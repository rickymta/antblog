#!/usr/bin/env bash
# Ba access pattern của dispatch.
source "$(dirname "$0")/common.sh"

driver="${1:-d001}"
cell="${2:-w3gvk}"
now_epoch="$(date +%s)"

echo "== 1. Vị trí mới nhất của $driver (đọc ngược sort key, lấy một dòng)"
ddb query --table-name "$TABLE" \
  --key-condition-expression "pk = :p AND begins_with(sk, :l)" \
  --expression-attribute-values "{\":p\":{\"S\":\"DRIVER#$driver\"},\":l\":{\"S\":\"LOC#\"}}" \
  --no-scan-index-forward --limit 1 \
  --query 'Items[0].[sk.S,lat.N,lng.N]' --output text

echo "== 2. Lịch sử 10 phút gần đây của $driver (BETWEEN trên sort key, lọc bản ghi đã hết hạn nhưng TTL chưa xoá)"
ddb query --table-name "$TABLE" \
  --key-condition-expression "pk = :p AND sk BETWEEN :a AND :b" \
  --filter-expression "expires_at > :now" \
  --expression-attribute-values "{\":p\":{\"S\":\"DRIVER#$driver\"},\":a\":{\"S\":\"LOC#$(iso_ago 600)\"},\":b\":{\"S\":\"LOC#$(iso_ago 0)\"},\":now\":{\"N\":\"$now_epoch\"}}" \
  --query 'Count' --output text

echo "== 3. Tài xế đang rảnh trong ô $cell (GSI gsi1), rồi BatchGetItem để lấy chi tiết từ bảng gốc"
ids="$(ddb query --table-name "$TABLE" --index-name gsi1 \
  --key-condition-expression "gsi1pk = :p" \
  --expression-attribute-values "{\":p\":{\"S\":\"CITY#hcm#AVAILABLE#$cell\"}}" \
  --query 'Items[].pk.S' --output text)"
echo "$ids"

keys="$(for pk in $ids; do printf '{"pk":{"S":"%s"},"sk":{"S":"STATE"}}\n' "$pk"; done | jq -s '.')"
ddb batch-get-item --request-items "{\"$TABLE\":{\"Keys\":$keys}}" \
  --query "Responses.\"$TABLE\"[].[pk.S,status.S,lat.N,lng.N]" --output text
