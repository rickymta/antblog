#!/usr/bin/env bash
# Hàm dùng chung. File này được `source`, không chạy trực tiếp.
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

log() { printf '\n== %s\n' "$*"; }

# In danh tính đang dùng và hỏi xác nhận trước khi tạo/xoá tài nguyên.
# Đặt ASSUME_YES=1 để bỏ qua câu hỏi (ví dụ khi chạy lại trong tài khoản lab của bạn).
confirm_identity() {
  log "Danh tính đang dùng"
  aws sts get-caller-identity --output table
  if [[ "${ASSUME_YES:-0}" != "1" ]]; then
    read -r -p "Đây có phải tài khoản LAB của bạn không? Gõ 'yes' để tiếp tục: " ans
    [[ "$ans" == "yes" ]] || { echo "Dừng." >&2; exit 1; }
  fi
}

ACCOUNT_ID="$(aws sts get-caller-identity --query Account --output text)"
export ACCOUNT_ID
BUCKET="${BUCKET:-toinoi-lab-delivery-photos-${ACCOUNT_ID}}"
export BUCKET
WEB_ORIGIN="${WEB_ORIGIN:-https://app.toinoi.example}"
APP_ROLE_ARN="${APP_ROLE_ARN:-}"
