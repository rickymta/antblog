#!/usr/bin/env bash
# Hàm dùng chung. File này được `source`, không chạy trực tiếp.
# shellcheck disable=SC2034,SC2054
set -euo pipefail
export AWS_PAGER=""

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ ! -f "$HERE/env.sh" ]]; then
  echo "Thiếu env.sh. Chạy: cp env.sh.example env.sh rồi sửa giá trị." >&2
  exit 1
fi
# shellcheck source=/dev/null
source "$HERE/env.sh"

export AWS_DEFAULT_REGION="${AWS_REGION:?AWS_REGION chưa đặt}"
if [[ -z "${AWS_PROFILE:-}" ]]; then unset AWS_PROFILE; fi

# Tên tài nguyên theo quy ước toinoi-<env>-<service>. Lab bài 10 dùng lại đúng các tên này.
ALB_NAME="toinoi-lab-quote-alb"
TG_NAME="toinoi-lab-quote-tg"
ASG_NAME="toinoi-lab-quote-asg"
LT_NAME="toinoi-lab-quote-lt"
# Hai security group do lab bài 04 tạo (sg-alb, sg-app); lab này dùng lại, không tạo mới và không xoá.
SG_ALB_NAME="toinoi-lab-alb"
SG_APP_NAME="toinoi-lab-app"
TAG_SHORT=(Key=Project,Value=ToiNoi Key=Env,Value=lab Key=Service,Value=quote Key=Owner,Value=platform)
TAG_SPEC_JSON='{Key=Project,Value=ToiNoi},{Key=Env,Value=lab},{Key=Service,Value=quote},{Key=Owner,Value=platform}'

log() { printf '\n== %s\n' "$*"; }

confirm_identity() {
  log "Danh tính đang dùng"
  aws sts get-caller-identity --output table
  if [[ "${ASSUME_YES:-0}" != "1" ]]; then
    read -r -p "Đây có phải tài khoản LAB của bạn không? Gõ 'yes' để tiếp tục: " ans
    [[ "$ans" == "yes" ]] || { echo "Dừng." >&2; exit 1; }
  fi
}

# ID từ lab bài 04: biến nào để trống trong env.sh thì tra theo tag Name mà create-vpc.sh hoặc Terraform bài 04 đặt.
lab_subnet_ids() { # <tên subnet...>
  local name ids=()
  for name in "$@"; do
    ids+=("$(aws ec2 describe-subnets --filters "Name=vpc-id,Values=$VPC_ID" "Name=tag:Name,Values=$name" \
      --query 'Subnets[0].SubnetId' --output text | sed 's/^None$//')")
  done
  echo "${ids[*]}"
}
VPC_ID="${VPC_ID:-$(aws ec2 describe-vpcs --filters Name=tag:Name,Values=toinoi-lab-vpc Name=tag:Project,Values=ToiNoi \
  --query 'Vpcs[0].VpcId' --output text | sed 's/^None$//')}"
if [[ -n "$VPC_ID" ]]; then
  PUBLIC_SUBNET_IDS="${PUBLIC_SUBNET_IDS:-$(lab_subnet_ids toinoi-lab-public-a toinoi-lab-public-b)}"
  APP_SUBNET_IDS="${APP_SUBNET_IDS:-$(lab_subnet_ids toinoi-lab-app-a toinoi-lab-app-b)}"
fi

sg_id() { # $1 = tên SG; in ID hoặc rỗng
  aws ec2 describe-security-groups \
    --filters "Name=group-name,Values=$1" "Name=vpc-id,Values=${VPC_ID:-none}" \
    --query 'SecurityGroups[0].GroupId' --output text | sed 's/^None$//'
}
alb_arn() {
  aws elbv2 describe-load-balancers --names "$ALB_NAME" \
    --query 'LoadBalancers[0].LoadBalancerArn' --output text 2>/dev/null | sed 's/^None$//' || true
}
tg_arn() {
  aws elbv2 describe-target-groups --names "$TG_NAME" \
    --query 'TargetGroups[0].TargetGroupArn' --output text 2>/dev/null | sed 's/^None$//' || true
}
