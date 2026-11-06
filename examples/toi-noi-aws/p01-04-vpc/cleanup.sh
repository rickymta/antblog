#!/usr/bin/env bash
# Dọn VPC toinoi-lab-vpc (lab bài 04) theo thứ tự phụ thuộc, tìm tài nguyên bằng tag.
# MẶC ĐỊNH CHỈ IN LỆNH XOÁ (dry-run); các lệnh chỉ-đọc vẫn được gọi thật. Thêm --apply để xoá.
# Dùng: ./cleanup.sh [profile] [--apply]
# Nếu còn ENI (ALB, RDS, EC2 của bài sau) trong subnet thì xoá subnet sẽ lỗi: dọn các lab đó trước.
set -euo pipefail

profile="toinoi-lab"
region="ap-southeast-1"
apply=0
for arg in "$@"; do
  case "$arg" in
    --apply) apply=1 ;;
    -*) echo "Tuỳ chọn không hợp lệ: $arg" >&2; exit 2 ;;
    *) profile="$arg" ;;
  esac
done
export AWS_PAGER=""

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

# Lệnh chỉ-đọc: luôn chạy thật.
ro() { aws "$@" --profile "$profile" --region "$region"; }
# Lệnh ghi: chỉ chạy khi --apply.
rw() {
  if [ "$apply" -eq 1 ]; then
    aws "$@" --profile "$profile" --region "$region"
  else
    show "$@"
  fi
}

vpc="$(ro ec2 describe-vpcs \
  --filters Name=tag:Name,Values=toinoi-lab-vpc Name=tag:Project,Values=ToiNoi \
  --query 'Vpcs[0].VpcId' --output text)"
if [ "$vpc" = "None" ] || [ -z "$vpc" ]; then
  echo "Không tìm thấy VPC toinoi-lab-vpc; không có gì để dọn."
  exit 0
fi
echo "Dọn VPC $vpc ($([ "$apply" -eq 1 ] && echo THẬT || echo dry-run))"

# 1. NAT Gateway và Elastic IP của chúng.
nats="$(ro ec2 describe-nat-gateways \
  --filter "Name=vpc-id,Values=$vpc" Name=state,Values=pending,available \
  --query 'NatGateways[].NatGatewayId' --output text)"
eips="$(ro ec2 describe-nat-gateways \
  --filter "Name=vpc-id,Values=$vpc" Name=state,Values=pending,available \
  --query 'NatGateways[].NatGatewayAddresses[].AllocationId' --output text)"
for nat in $nats; do
  rw ec2 delete-nat-gateway --nat-gateway-id "$nat" >/dev/null
  if [ "$apply" -eq 1 ]; then
    aws ec2 wait nat-gateway-deleted --nat-gateway-ids "$nat" --profile "$profile" --region "$region"
  fi
done
for eip in $eips; do
  rw ec2 release-address --allocation-id "$eip"
done

# 2. Subnet (xoá subnet cũng gỡ các liên kết route table và NACL của nó).
for subnet in $(ro ec2 describe-subnets --filters "Name=vpc-id,Values=$vpc" \
  --query 'Subnets[].SubnetId' --output text); do
  rw ec2 delete-subnet --subnet-id "$subnet"
done

# shellcheck disable=SC2016  # dấu backtick trong JMESPath là cố ý, không phải command substitution
# 3. Route table không phải main (luật và liên kết đi cùng).
for rt in $(ro ec2 describe-route-tables --filters "Name=vpc-id,Values=$vpc" \
  --query 'RouteTables[?length(Associations[?Main==`true`])==`0`].RouteTableId' --output text); do
  rw ec2 delete-route-table --route-table-id "$rt"
done

# 4. NACL tuỳ chỉnh (không phải mặc định).
# shellcheck disable=SC2016
for acl in $(ro ec2 describe-network-acls --filters "Name=vpc-id,Values=$vpc" \
  --query 'NetworkAcls[?IsDefault==`false`].NetworkAclId' --output text); do
  rw ec2 delete-network-acl --network-acl-id "$acl"
done

# 5. Internet Gateway.
for igw in $(ro ec2 describe-internet-gateways --filters "Name=attachment.vpc-id,Values=$vpc" \
  --query 'InternetGateways[].InternetGatewayId' --output text); do
  rw ec2 detach-internet-gateway --internet-gateway-id "$igw" --vpc-id "$vpc"
  rw ec2 delete-internet-gateway --internet-gateway-id "$igw"
done

# 6. Security group: db trước (tham chiếu app), rồi app (tham chiếu alb), rồi alb.
for name in toinoi-lab-db toinoi-lab-app toinoi-lab-alb; do
  sg="$(ro ec2 describe-security-groups \
    --filters "Name=vpc-id,Values=$vpc" "Name=group-name,Values=$name" \
    --query 'SecurityGroups[0].GroupId' --output text)"
  if [ "$sg" != "None" ] && [ -n "$sg" ]; then
    rw ec2 delete-security-group --group-id "$sg"
  fi
done

# 7. VPC.
rw ec2 delete-vpc --vpc-id "$vpc"
[ "$apply" -eq 1 ] && echo "Đã dọn xong." || echo "Dry-run xong; thêm --apply để xoá thật."
