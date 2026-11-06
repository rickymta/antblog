#!/usr/bin/env bash
# Tạo instance toinoi-lab-quote: role + instance profile (Session Manager), security group KHÔNG có inbound,
# instance AL2023 (IMDSv2 bắt buộc, root gp3 mã hoá) chạy placeholder quote qua user data.
# MẶC ĐỊNH CHỈ IN LỆNH (dry-run). Các lệnh chỉ-đọc (tra VPC/subnet) vẫn được gọi thật.
#
# Dùng: ./create-instance.sh [profile] [--apply] [--use-lab-vpc] [--arm]
#   --apply        tạo thật
#   --use-lab-vpc  đặt vào subnet private-app và sg-app của VPC bài 04 (cần NAT hoặc VPC endpoint để SSM hoạt động)
#                  mặc định: default VPC, subnet mặc định đầu tiên (nhận IPv4 công khai, TÍNH TIỀN theo giờ)
#   --arm          Graviton (t4g.micro + AMI arm64); image nginx:stable-alpine có bản arm64
# Biến: INSTANCE_TYPE để đổi loại máy.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
profile="toinoi-lab"
region="ap-southeast-1"
apply=0
use_lab_vpc=0
arch="x86_64"
for arg in "$@"; do
  case "$arg" in
    --apply) apply=1 ;;
    --use-lab-vpc) use_lab_vpc=1 ;;
    --arm) arch="arm64" ;;
    -*) echo "Tuỳ chọn không hợp lệ: $arg" >&2; exit 2 ;;
    *) profile="$arg" ;;
  esac
done
export AWS_PAGER=""

if [ "$arch" = "arm64" ]; then
  instance_type="${INSTANCE_TYPE:-t4g.micro}"
  ami_param="/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-arm64"
else
  instance_type="${INSTANCE_TYPE:-t3.micro}"
  ami_param="/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64"
fi
role="toinoi-lab-quote-ec2"
sg_name="toinoi-lab-quote"

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
# aws_do <giá trị giả khi dry-run> <đối số aws...>
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
ro() { aws "$@" --profile "$profile" --region "$region"; }
tags() {
  printf 'ResourceType=%s,Tags=[{Key=Name,Value=%s},{Key=Project,Value=ToiNoi},{Key=Env,Value=lab},{Key=Service,Value=quote},{Key=Owner,Value=platform}]' "$1" "$2"
}
iam_tags=("Key=Project,Value=ToiNoi" "Key=Env,Value=lab" "Key=Service,Value=quote" "Key=Owner,Value=platform")

# 1. Mạng: chọn subnet và security group.
if [ "$use_lab_vpc" -eq 1 ]; then
  vpc="$(ro ec2 describe-vpcs --filters Name=tag:Name,Values=toinoi-lab-vpc Name=tag:Project,Values=ToiNoi \
    --query 'Vpcs[0].VpcId' --output text)"
  [ "$vpc" != "None" ] || { echo "Không thấy VPC toinoi-lab-vpc; chạy lab bài 04 trước." >&2; exit 1; }
  subnet="$(ro ec2 describe-subnets --filters "Name=vpc-id,Values=$vpc" Name=tag:Name,Values=toinoi-lab-app-a \
    --query 'Subnets[0].SubnetId' --output text)"
  sg="$(ro ec2 describe-security-groups --filters "Name=vpc-id,Values=$vpc" Name=group-name,Values=toinoi-lab-app \
    --query 'SecurityGroups[0].GroupId' --output text)"
  echo "Dùng VPC bài 04: $vpc, subnet $subnet, security group sg-app $sg"
  echo "Lưu ý: cần NAT Gateway (create-vpc.sh --with-nat) hoặc VPC endpoint, nếu không SSM Agent không ra được ngoài."
else
  vpc="$(ro ec2 describe-vpcs --filters Name=is-default,Values=true --query 'Vpcs[0].VpcId' --output text)"
  [ "$vpc" != "None" ] || { echo "Region này không có default VPC; dùng --use-lab-vpc." >&2; exit 1; }
  subnet="$(ro ec2 describe-subnets --filters "Name=vpc-id,Values=$vpc" Name=default-for-az,Values=true \
    --query 'Subnets[0].SubnetId' --output text)"
  echo "Dùng default VPC $vpc, subnet $subnet (có IPv4 công khai, tính tiền theo giờ)."
  sg="$(aws_do sg-PLACEHOLDER ec2 create-security-group --group-name "$sg_name" \
    --description "Toi Noi lab: quote, khong co inbound" --vpc-id "$vpc" \
    --tag-specifications "$(tags security-group "$sg_name")" --query GroupId --output text)"
fi

# 2. Role cho EC2 + instance profile, gắn policy Session Manager.
aws_do - iam create-role --role-name "$role" \
  --assume-role-policy-document "file://$here/trust-policy-ec2.json" --tags "${iam_tags[@]}" >/dev/null
aws_do - iam attach-role-policy --role-name "$role" \
  --policy-arn arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore >/dev/null
aws_do - iam create-instance-profile --instance-profile-name "$role" --tags "${iam_tags[@]}" >/dev/null
aws_do - iam add-role-to-instance-profile --instance-profile-name "$role" --role-name "$role" >/dev/null
if [ "$apply" -eq 1 ]; then
  # IAM cần vài giây để nhất quán trước khi EC2 thấy instance profile.
  aws iam wait instance-profile-exists --instance-profile-name "$role" --profile "$profile"
  sleep 10
fi

# 3. Instance: IMDSv2 bắt buộc, root gp3 mã hoá và xoá khi terminate, user data cài và chạy quote.
instance="$(aws_do i-PLACEHOLDER ec2 run-instances \
  --image-id "resolve:ssm:$ami_param" \
  --instance-type "$instance_type" \
  --subnet-id "$subnet" \
  --security-group-ids "$sg" \
  --iam-instance-profile "Name=$role" \
  --metadata-options HttpTokens=required,HttpEndpoint=enabled,HttpPutResponseHopLimit=1 \
  --block-device-mappings 'DeviceName=/dev/xvda,Ebs={VolumeSize=8,VolumeType=gp3,Encrypted=true,DeleteOnTermination=true}' \
  --user-data "file://$here/user-data.sh" \
  --tag-specifications "$(tags instance toinoi-lab-quote)" "$(tags volume toinoi-lab-quote-root)" \
  --query 'Instances[0].InstanceId' --output text)"

if [ "$apply" -eq 1 ]; then
  aws ec2 wait instance-running --instance-ids "$instance" --profile "$profile" --region "$region"
  echo "Instance $instance đang chạy. Đợi SSM Agent báo Online (thường 1-3 phút)..."
  for _ in $(seq 1 40); do
    status="$(ro ssm describe-instance-information --filters "Key=InstanceIds,Values=$instance" \
      --query 'InstanceInformationList[0].PingStatus' --output text 2>/dev/null || true)"
    [ "$status" = "Online" ] && break
    sleep 10
  done
  echo "PingStatus: ${status:-chưa thấy}. Kiểm service bằng ./check.sh $profile"
else
  echo
  echo "Đây là dry-run; thêm --apply để tạo thật. Dọn lại bằng ./cleanup.sh $profile --apply"
fi
