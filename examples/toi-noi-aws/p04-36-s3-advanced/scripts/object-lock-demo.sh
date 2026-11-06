#!/usr/bin/env bash
# Thử Object Lock trên bucket hoá đơn của lab: ghi object có retention, thử xoá, đặt legal hold, bypass governance.
# Chạy SAU `terraform apply`. Script chỉ ghi vài byte và hỏi xác nhận trước khi chạy.
# Chạy trong tài khoản lab: object ở chế độ COMPLIANCE sẽ KHÔNG xoá được cho tới hết hạn, kể cả root.
set -euo pipefail

: "${AWS_PROFILE:?Đặt AWS_PROFILE trỏ vào tài khoản lab}"
REGION="${AWS_REGION:-ap-southeast-1}"
ACCOUNT_ID="$(aws sts get-caller-identity --query Account --output text)"
BUCKET="${BUCKET:-toinoi-lab-invoices-${ACCOUNT_ID}}"
KEY="invoices/2026/11/inv-100123.txt"

case "$BUCKET" in
  toinoi-lab-*) ;;
  *) echo "Từ chối: bucket phải bắt đầu bằng toinoi-lab-" >&2; exit 1 ;;
esac

echo "Tài khoản: ${ACCOUNT_ID}, bucket: ${BUCKET}"
read -r -p "Tiếp tục ghi object thử vào bucket này? [y/N] " ans
[ "$ans" = "y" ] || exit 0

RETAIN_UNTIL="$(date -u -v+1d +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -d '+1 day' +%Y-%m-%dT%H:%M:%SZ)"
TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT
echo "hoa don thu nghiem" >"$TMP"

echo "1) Ghi object với retention GOVERNANCE tới ${RETAIN_UNTIL}"
VERSION_ID="$(aws s3api put-object --region "$REGION" --bucket "$BUCKET" --key "$KEY" --body "$TMP" \
  --object-lock-mode GOVERNANCE --object-lock-retain-until-date "$RETAIN_UNTIL" \
  --query VersionId --output text)"
echo "   VersionId: ${VERSION_ID}"

echo "2) Xem retention"
aws s3api get-object-retention --region "$REGION" --bucket "$BUCKET" --key "$KEY" --version-id "$VERSION_ID"

echo "3) Thử xoá đúng version: phải bị AccessDenied"
if aws s3api delete-object --region "$REGION" --bucket "$BUCKET" --key "$KEY" --version-id "$VERSION_ID"; then
  echo "   Bất ngờ: xoá được. Kiểm tra lại cấu hình Object Lock." >&2
else
  echo "   Đúng như mong đợi: bị từ chối."
fi

echo "4) Đặt legal hold (không có ngày hết hạn)"
aws s3api put-object-legal-hold --region "$REGION" --bucket "$BUCKET" --key "$KEY" --version-id "$VERSION_ID" \
  --legal-hold Status=ON

echo "5) Gỡ legal hold, rồi xoá bằng quyền bypass governance (cần s3:BypassGovernanceRetention)"
aws s3api put-object-legal-hold --region "$REGION" --bucket "$BUCKET" --key "$KEY" --version-id "$VERSION_ID" \
  --legal-hold Status=OFF
aws s3api delete-object --region "$REGION" --bucket "$BUCKET" --key "$KEY" --version-id "$VERSION_ID" \
  --bypass-governance-retention
echo "Xong. Nếu bucket đã bật replication, bản sao ở region thứ hai cũng cần dọn (xem README)."
