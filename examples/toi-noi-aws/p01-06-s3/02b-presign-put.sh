#!/usr/bin/env bash
set -euo pipefail
# Tài xế upload ảnh bằng presigned URL PUT. Cần python3 và boto3 (pip install boto3).
source "$(dirname "$0")/common.sh"

if ! python3 -c 'import boto3' 2>/dev/null; then
  echo "Chưa có boto3. Cài bằng: python3 -m pip install --user boto3" >&2
  exit 1
fi

TRIP_ID="${TRIP_ID:-0b6f0d52-6c1c-4f7b-9d57-3d2c1c6a1002}"
KEY="proof/${TRIP_ID}/pickup.jpg"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
printf 'anh-lay-hang-gia-lap' > "$TMP/photo.jpg"

URL="$(python3 "$HERE/presign_put.py" "$BUCKET" "$KEY" 300)"

log "PUT đúng Content-Type: mong đợi 200"
curl -sS -o /dev/null -w '%{http_code}\n' -X PUT -H 'Content-Type: image/jpeg' \
  --data-binary "@$TMP/photo.jpg" "$URL"

log "PUT sai Content-Type: mong đợi 403 (SignatureDoesNotMatch)"
curl -sS -o /dev/null -w '%{http_code}\n' -X PUT -H 'Content-Type: text/plain' \
  --data-binary "@$TMP/photo.jpg" "$URL"
