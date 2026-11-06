#!/usr/bin/env bash
set -euo pipefail
# Tải một "ảnh" giả lên rồi tạo presigned URL GET: hợp lệ, không chữ ký, và hết hạn.
# Lưu ý: `aws s3 presign` chỉ tạo URL GET. URL PUT cần SDK, xem 02b-presign-put.sh.
source "$(dirname "$0")/common.sh"

TRIP_ID="${TRIP_ID:-0b6f0d52-6c1c-4f7b-9d57-3d2c1c6a1001}"
KEY="proof/${TRIP_ID}/delivered.jpg"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
printf 'anh-chung-tu-gia-lap' > "$TMP/photo.jpg"

log "Upload $KEY bằng quyền của bạn"
aws s3api put-object --bucket "$BUCKET" --key "$KEY" --body "$TMP/photo.jpg" \
  --content-type image/jpeg --query VersionId --output text

BASE="https://${BUCKET}.s3.${AWS_REGION}.amazonaws.com/${KEY}"

log "1) Không có chữ ký: mong đợi 403"
curl -sS -o /dev/null -w '%{http_code}\n' "$BASE"

log "2) Presigned URL GET, hiệu lực 300 giây: mong đợi 200"
URL="$(aws s3 presign "s3://${BUCKET}/${KEY}" --expires-in 300 --region "$AWS_REGION")"
curl -sS -o /dev/null -w '%{http_code}\n' "$URL"

log "3) Presigned URL hiệu lực 5 giây, đợi 8 giây: mong đợi 403 (Request has expired)"
SHORT="$(aws s3 presign "s3://${BUCKET}/${KEY}" --expires-in 5 --region "$AWS_REGION")"
sleep 8
curl -sS -w '\nHTTP %{http_code}\n' "$SHORT" | head -n 6
