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

# Tên tài nguyên theo quy ước toinoi-<env>-<service>.
DB_ID="${DB_ID:-toinoi-lab-trips}"
DB_SUBNET_GROUP="toinoi-lab-trips"
DB_PARAM_GROUP="toinoi-lab-trips-pg"
DB_SG_NAME="toinoi-lab-db"   # sg-db do lab bài 04 tạo; lab này chỉ tạo khi chưa có
DB_NAME="tripsdb"
DB_USER="toinoi_admin"
ROLE_POLICY_NAME="toinoi-lab-read-db-secret"
COMMON_TAGS=(Key=Project,Value=ToiNoi Key=Env,Value=lab Key=Service,Value=trips Key=Owner,Value=platform)

log() { printf '\n== %s\n' "$*"; }

# ID từ lab bài 04: biến nào để trống trong env.sh thì tra theo tag Name mà create-vpc.sh hoặc Terraform bài 04 đặt.
none_to_empty() { sed 's/^None$//'; }
lab_vpc_id() {
  aws ec2 describe-vpcs --filters Name=tag:Name,Values=toinoi-lab-vpc Name=tag:Project,Values=ToiNoi \
    --query 'Vpcs[0].VpcId' --output text | none_to_empty
}
lab_subnet_ids() { # <tên subnet...>, ví dụ toinoi-lab-data-a toinoi-lab-data-b
  local name ids=()
  for name in "$@"; do
    ids+=("$(aws ec2 describe-subnets --filters "Name=vpc-id,Values=$VPC_ID" "Name=tag:Name,Values=$name" \
      --query 'Subnets[0].SubnetId' --output text | none_to_empty)")
  done
  echo "${ids[*]}"
}
lab_sg_id() { # <tên security group>
  aws ec2 describe-security-groups --filters "Name=group-name,Values=$1" "Name=vpc-id,Values=$VPC_ID" \
    --query 'SecurityGroups[0].GroupId' --output text | none_to_empty
}
VPC_ID="${VPC_ID:-$(lab_vpc_id)}"
if [[ -n "$VPC_ID" ]]; then
  DB_SUBNET_IDS="${DB_SUBNET_IDS:-$(lab_subnet_ids toinoi-lab-data-a toinoi-lab-data-b)}"
  SG_APP_ID="${SG_APP_ID:-$(lab_sg_id toinoi-lab-app)}"
fi

confirm_identity() {
  log "Danh tính đang dùng"
  aws sts get-caller-identity --output table
  if [[ "${ASSUME_YES:-0}" != "1" ]]; then
    read -r -p "Đây có phải tài khoản LAB của bạn không? Gõ 'yes' để tiếp tục: " ans
    [[ "$ans" == "yes" ]] || { echo "Dừng." >&2; exit 1; }
  fi
}

# Lấy ID security group của DB theo tên + VPC (rỗng nếu chưa có).
find_db_sg() { lab_sg_id "$DB_SG_NAME"; }
# Chỉ SG do CHÍNH lab này tạo (tag Service=trips); sg-db của bài 04 có tag Service=network và để bài 04 dọn.
find_db_sg_owned() {
  aws ec2 describe-security-groups \
    --filters "Name=group-name,Values=$DB_SG_NAME" "Name=vpc-id,Values=$VPC_ID" Name=tag:Service,Values=trips \
    --query 'SecurityGroups[0].GroupId' --output text | none_to_empty
}
