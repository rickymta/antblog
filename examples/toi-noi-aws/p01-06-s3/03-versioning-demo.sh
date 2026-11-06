#!/usr/bin/env bash
set -euo pipefail
# Quan sát điều gì xảy ra khi xoá object trong bucket bật versioning.
source "$(dirname "$0")/common.sh"

KEY="proof/demo-versioning/delivered.jpg"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

log "Ghi hai phiên bản của cùng một key"
printf 'v1' > "$TMP/v1"; printf 'v2' > "$TMP/v2"
V1="$(aws s3api put-object --bucket "$BUCKET" --key "$KEY" --body "$TMP/v1" --query VersionId --output text)"
V2="$(aws s3api put-object --bucket "$BUCKET" --key "$KEY" --body "$TMP/v2" --query VersionId --output text)"
echo "v1=$V1"; echo "v2=$V2"

log "Xoá object KHÔNG chỉ version: S3 chỉ thêm delete marker"
MARKER="$(aws s3api delete-object --bucket "$BUCKET" --key "$KEY" --query VersionId --output text)"
echo "delete marker=$MARKER"

log "GET key thường: mong đợi NoSuchKey"
aws s3api get-object --bucket "$BUCKET" --key "$KEY" "$TMP/out" 2>&1 | head -n 2 || true

log "Các phiên bản vẫn còn nguyên"
aws s3api list-object-versions --bucket "$BUCKET" --prefix "$KEY" \
  --query '{Versions:Versions[].[VersionId,IsLatest],DeleteMarkers:DeleteMarkers[].[VersionId,IsLatest]}'

log "Xoá chính delete marker: object sống lại với phiên bản v2"
aws s3api delete-object --bucket "$BUCKET" --key "$KEY" --version-id "$MARKER" >/dev/null
aws s3api get-object --bucket "$BUCKET" --key "$KEY" "$TMP/out" >/dev/null
echo "Nội dung hiện tại: $(cat "$TMP/out")"

log "Xoá vĩnh viễn v2 bằng cách chỉ đúng version id"
aws s3api delete-object --bucket "$BUCKET" --key "$KEY" --version-id "$V2" >/dev/null
aws s3api get-object --bucket "$BUCKET" --key "$KEY" "$TMP/out" >/dev/null
echo "Nội dung hiện tại: $(cat "$TMP/out") (phiên bản cũ nhất còn lại)"
