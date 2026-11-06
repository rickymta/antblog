#!/usr/bin/env bash
set -euo pipefail
# Tạo bucket ảnh chứng từ giao hàng với cấu hình an toàn. Chạy lại được (idempotent ở mức hợp lý).
source "$(dirname "$0")/common.sh"
confirm_identity

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

log "Tạo bucket $BUCKET ở $AWS_REGION"
if aws s3api head-bucket --bucket "$BUCKET" 2>/dev/null; then
  echo "Bucket đã tồn tại và thuộc tài khoản này, bỏ qua bước tạo."
else
  # Ngoài us-east-1 bắt buộc có LocationConstraint.
  aws s3api create-bucket --bucket "$BUCKET" \
    --create-bucket-configuration "LocationConstraint=$AWS_REGION"
fi

log "Block Public Access (bốn cờ), ACL tắt, mã hoá SSE-S3"
aws s3api put-public-access-block --bucket "$BUCKET" \
  --public-access-block-configuration \
  BlockPublicAcls=true,IgnorePublicAcls=true,BlockPublicPolicy=true,RestrictPublicBuckets=true
aws s3api put-bucket-ownership-controls --bucket "$BUCKET" \
  --ownership-controls 'Rules=[{ObjectOwnership=BucketOwnerEnforced}]'
aws s3api put-bucket-encryption --bucket "$BUCKET" \
  --server-side-encryption-configuration \
  'Rules=[{ApplyServerSideEncryptionByDefault={SSEAlgorithm=AES256}}]'

log "Bật versioning và lifecycle"
aws s3api put-bucket-versioning --bucket "$BUCKET" \
  --versioning-configuration Status=Enabled
aws s3api put-bucket-lifecycle-configuration --bucket "$BUCKET" \
  --lifecycle-configuration "file://$HERE/lifecycle.json"

log "CORS cho web $WEB_ORIGIN"
sed "s|__WEB_ORIGIN__|$WEB_ORIGIN|g" "$HERE/cors.json.tpl" > "$TMP/cors.json"
aws s3api put-bucket-cors --bucket "$BUCKET" --cors-configuration "file://$TMP/cors.json"

log "Bucket policy"
if [[ -n "${APP_ROLE_ARN:-}" ]]; then
  TPL="$HERE/bucket-policy-app-role.json.tpl"
else
  echo "APP_ROLE_ARN trống: chỉ áp policy chặn HTTP không mã hoá."
  TPL="$HERE/bucket-policy-tls-only.json.tpl"
fi
sed -e "s|__BUCKET__|$BUCKET|g" -e "s|__APP_ROLE_ARN__|${APP_ROLE_ARN:-}|g" "$TPL" > "$TMP/policy.json"
aws s3api put-bucket-policy --bucket "$BUCKET" --policy "file://$TMP/policy.json"

log "Tag"
aws s3api put-bucket-tagging --bucket "$BUCKET" --tagging \
  'TagSet=[{Key=Project,Value=ToiNoi},{Key=Env,Value=lab},{Key=Service,Value=delivery-photo},{Key=Owner,Value=platform}]'

log "Kiểm tra lại cấu hình"
aws s3api get-public-access-block --bucket "$BUCKET"
aws s3api get-bucket-versioning --bucket "$BUCKET"
aws s3api get-bucket-lifecycle-configuration --bucket "$BUCKET" --query 'Rules[].ID'
echo "Xong. Bucket: $BUCKET"
