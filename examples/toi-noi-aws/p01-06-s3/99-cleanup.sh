#!/usr/bin/env bash
set -euo pipefail
# Xoá bucket lab: phải xoá MỌI version và delete marker trước, rồi mới xoá bucket được.
# Việc xoá không thể hoàn tác. Chỉ chạy trong tài khoản lab và với đúng bucket của lab này.
source "$(dirname "$0")/common.sh"
confirm_identity

case "$BUCKET" in
  toinoi-lab-*) ;;
  *) echo "Từ chối: tên bucket '$BUCKET' không bắt đầu bằng toinoi-lab-" >&2; exit 1 ;;
esac

if ! aws s3api head-bucket --bucket "$BUCKET" 2>/dev/null; then
  echo "Bucket $BUCKET không tồn tại, không có gì để dọn."
  exit 0
fi

# $1 = Versions hoặc DeleteMarkers
purge() {
  local field="$1" out key vid
  while :; do
    out="$(aws s3api list-object-versions --bucket "$BUCKET" --max-items 500 \
      --query "${field}[].[Key,VersionId]" --output text)"
    if [[ -z "$out" || "$out" == "None" ]]; then break; fi
    while IFS=$'\t' read -r key vid; do
      [[ -n "$key" ]] || continue
      aws s3api delete-object --bucket "$BUCKET" --key "$key" --version-id "$vid" >/dev/null
    done <<<"$out"
  done
}

log "Xoá toàn bộ version"
purge Versions
log "Xoá toàn bộ delete marker"
purge DeleteMarkers
log "Xoá bucket"
aws s3api delete-bucket --bucket "$BUCKET"
echo "Đã xoá $BUCKET. Kiểm lại: aws s3api head-bucket --bucket $BUCKET (phải báo 404)."
