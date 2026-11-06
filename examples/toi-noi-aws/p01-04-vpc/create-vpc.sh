#!/usr/bin/env bash
# Dựng VPC 10.20.0.0/16 của Tới Nơi: 6 subnet (public, private-app, private-data) ở 2 AZ,
# Internet Gateway, route table và 3 security group (sg-alb -> sg-app:8080 -> sg-db:5432).
# MẶC ĐỊNH CHỈ IN LỆNH (dry-run). Các lệnh chỉ-đọc (lấy tên AZ) vẫn được gọi thật.
#
# Dùng: ./create-vpc.sh [profile] [--apply] [--with-nat] [--with-nacl]
#   --apply      tạo thật
#   --with-nat   thêm 1 NAT Gateway + Elastic IP (TÍNH TIỀN theo giờ và theo GB)
#   --with-nacl  gắn NACL tuỳ chỉnh cho 2 subnet private-data (chưa kiểm chứng với RDS)
set -euo pipefail

profile="toinoi-lab"
region="ap-southeast-1"
apply=0
with_nat=0
with_nacl=0
for arg in "$@"; do
  case "$arg" in
    --apply) apply=1 ;;
    --with-nat) with_nat=1 ;;
    --with-nacl) with_nacl=1 ;;
    -*) echo "Tuỳ chọn không hợp lệ: $arg" >&2; exit 2 ;;
    *) profile="$arg" ;;
  esac
done
export AWS_PAGER=""

vpc_name="toinoi-lab-vpc"

# show <đối số...>: in lệnh dễ đọc, chỉ đặt nháy đơn quanh đối số có ký tự đặc biệt.
show() {
  printf '  aws' >&2
  local a quote="'\\''"
  for a in "$@"; do
    if [[ "$a" =~ ^[A-Za-z0-9_./:=@%+,-]+$ ]]; then
      printf ' %s' "$a" >&2
    else
      printf " '%s'" "${a//\'/$quote}" >&2
    fi
  done
  printf '\n' >&2
}

# aws_do <giá trị giả khi dry-run> <đối số của aws...>: chạy thật khi --apply, nếu không thì in lệnh.
aws_do() {
  local placeholder="$1"
  shift
  if [ "$apply" -eq 1 ]; then
    aws "$@" --profile "$profile" --region "$region"
  else
    show "$@"
    printf '%s\n' "$placeholder"
  fi
}

tags() {
  printf 'ResourceType=%s,Tags=[{Key=Name,Value=%s},{Key=Project,Value=ToiNoi},{Key=Env,Value=lab},{Key=Service,Value=network},{Key=Owner,Value=platform}]' "$1" "$2"
}

# Hai AZ đầu tiên đang sẵn sàng (nhãn zone khác nhau giữa các tài khoản nên phải hỏi).
read -r az1 az2 < <(aws ec2 describe-availability-zones --profile "$profile" --region "$region" \
  --filters Name=zone-type,Values=availability-zone Name=state,Values=available \
  --query 'AvailabilityZones[0:2].ZoneName' --output text)
echo "AZ dùng: $az1, $az2 (region $region). Chế độ: $([ "$apply" -eq 1 ] && echo THẬT || echo dry-run)"

vpc="$(aws_do vpc-PLACEHOLDER ec2 create-vpc --cidr-block 10.20.0.0/16 \
  --tag-specifications "$(tags vpc "$vpc_name")" --query Vpc.VpcId --output text)"
aws_do - ec2 modify-vpc-attribute --vpc-id "$vpc" --enable-dns-hostnames '{"Value":true}' >/dev/null

mk_subnet() { # <tên> <cidr> <az>
  aws_do "subnet-$1" ec2 create-subnet --vpc-id "$vpc" --cidr-block "$2" --availability-zone "$3" \
    --tag-specifications "$(tags subnet "toinoi-lab-$1")" --query Subnet.SubnetId --output text
}
pub_a="$(mk_subnet public-a 10.20.0.0/24 "$az1")"
pub_b="$(mk_subnet public-b 10.20.1.0/24 "$az2")"
app_a="$(mk_subnet app-a 10.20.10.0/24 "$az1")"
app_b="$(mk_subnet app-b 10.20.11.0/24 "$az2")"
data_a="$(mk_subnet data-a 10.20.20.0/24 "$az1")"
data_b="$(mk_subnet data-b 10.20.21.0/24 "$az2")"

# Internet Gateway và route table public.
igw="$(aws_do igw-PLACEHOLDER ec2 create-internet-gateway \
  --tag-specifications "$(tags internet-gateway toinoi-lab-igw)" --query InternetGateway.InternetGatewayId --output text)"
aws_do - ec2 attach-internet-gateway --internet-gateway-id "$igw" --vpc-id "$vpc" >/dev/null

mk_rt() { # <tên>
  aws_do "rtb-$1" ec2 create-route-table --vpc-id "$vpc" \
    --tag-specifications "$(tags route-table "toinoi-lab-$1")" --query RouteTable.RouteTableId --output text
}
rt_public="$(mk_rt public)"
aws_do - ec2 create-route --route-table-id "$rt_public" --destination-cidr-block 0.0.0.0/0 --gateway-id "$igw" >/dev/null
aws_do - ec2 associate-route-table --route-table-id "$rt_public" --subnet-id "$pub_a" >/dev/null
aws_do - ec2 associate-route-table --route-table-id "$rt_public" --subnet-id "$pub_b" >/dev/null

# Route table private-app mỗi AZ một cái; chỉ có đường ra Internet khi dùng --with-nat.
rt_app_a="$(mk_rt app-a)"
rt_app_b="$(mk_rt app-b)"
aws_do - ec2 associate-route-table --route-table-id "$rt_app_a" --subnet-id "$app_a" >/dev/null
aws_do - ec2 associate-route-table --route-table-id "$rt_app_b" --subnet-id "$app_b" >/dev/null

# Route table private-data: chỉ có luật local, không có đường ra Internet.
rt_data="$(mk_rt data)"
aws_do - ec2 associate-route-table --route-table-id "$rt_data" --subnet-id "$data_a" >/dev/null
aws_do - ec2 associate-route-table --route-table-id "$rt_data" --subnet-id "$data_b" >/dev/null

if [ "$with_nat" -eq 1 ]; then
  echo "NAT Gateway và Elastic IP TÍNH TIỀN theo giờ; nhớ chạy cleanup.sh sau khi thử."
  eip="$(aws_do eipalloc-PLACEHOLDER ec2 allocate-address --domain vpc \
    --tag-specifications "$(tags elastic-ip toinoi-lab-nat-eip)" --query AllocationId --output text)"
  nat="$(aws_do nat-PLACEHOLDER ec2 create-nat-gateway --subnet-id "$pub_a" --allocation-id "$eip" \
    --tag-specifications "$(tags natgateway toinoi-lab-nat-a)" --query NatGateway.NatGatewayId --output text)"
  aws_do - ec2 wait nat-gateway-available --nat-gateway-ids "$nat" >/dev/null
  # Một NAT duy nhất phục vụ cả hai AZ: rẻ cho lab, nhưng mất AZ chứa NAT là app ở AZ kia mất đường ra.
  aws_do - ec2 create-route --route-table-id "$rt_app_a" --destination-cidr-block 0.0.0.0/0 --nat-gateway-id "$nat" >/dev/null
  aws_do - ec2 create-route --route-table-id "$rt_app_b" --destination-cidr-block 0.0.0.0/0 --nat-gateway-id "$nat" >/dev/null
fi

# Security group: sg-alb -> sg-app:8080 -> sg-db:5432.
mk_sg() { # <tên> <mô tả>
  aws_do "sg-$1" ec2 create-security-group --group-name "toinoi-lab-$1" --description "$2" --vpc-id "$vpc" \
    --tag-specifications "$(tags security-group "toinoi-lab-$1")" --query GroupId --output text
}
sg_alb="$(mk_sg alb 'Toi Noi lab: ALB nhan HTTPS tu Internet')"
sg_app="$(mk_sg app 'Toi Noi lab: service app, chi nhan tu ALB')"
sg_db="$(mk_sg db 'Toi Noi lab: PostgreSQL, chi nhan tu app')"
aws_do - ec2 authorize-security-group-ingress --group-id "$sg_alb" --protocol tcp --port 443 --cidr 0.0.0.0/0 >/dev/null
aws_do - ec2 authorize-security-group-ingress --group-id "$sg_app" --protocol tcp --port 8080 --source-group "$sg_alb" >/dev/null
aws_do - ec2 authorize-security-group-ingress --group-id "$sg_db" --protocol tcp --port 5432 --source-group "$sg_app" >/dev/null

if [ "$with_nacl" -eq 1 ]; then
  # NACL stateless cho subnet private-data: chỉ nhận 5432 từ hai subnet app, trả lời qua cổng tạm.
  nacl="$(aws_do acl-PLACEHOLDER ec2 create-network-acl --vpc-id "$vpc" \
    --tag-specifications "$(tags network-acl toinoi-lab-data)" --query NetworkAcl.NetworkAclId --output text)"
  aws_do - ec2 create-network-acl-entry --network-acl-id "$nacl" --ingress --rule-number 100 \
    --protocol tcp --port-range From=5432,To=5432 --cidr-block 10.20.10.0/23 --rule-action allow >/dev/null
  aws_do - ec2 create-network-acl-entry --network-acl-id "$nacl" --egress --rule-number 100 \
    --protocol tcp --port-range From=1024,To=65535 --cidr-block 10.20.10.0/23 --rule-action allow >/dev/null
  for subnet in "$data_a" "$data_b"; do
    assoc="$(aws_do assoc-PLACEHOLDER ec2 describe-network-acls --filters "Name=association.subnet-id,Values=$subnet" \
      --query "NetworkAcls[0].Associations[?SubnetId=='$subnet'].NetworkAclAssociationId | [0]" --output text)"
    aws_do - ec2 replace-network-acl-association --association-id "$assoc" --network-acl-id "$nacl" >/dev/null
  done
fi

echo
echo "VPC $vpc: public=($pub_a,$pub_b) app=($app_a,$app_b) data=($data_a,$data_b)"
echo "Security group: alb=$sg_alb app=$sg_app db=$sg_db"
[ "$apply" -eq 1 ] || echo "Đây là dry-run; thêm --apply để tạo thật. Dọn lại bằng ./cleanup.sh $profile --apply"
