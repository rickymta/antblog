#!/usr/bin/env bash
# Ghi một toạ độ mới: một dòng lịch sử (TTL 24 giờ) và cập nhật STATE. Không đổi gsi1pk nên không ghi vào GSI.
# Dùng: ./put-location.sh <driver-id> <lat> <lng>
source "$(dirname "$0")/common.sh"

driver="${1:?driver id}"
lat="${2:?lat}"
lng="${3:?lng}"
ts="$(iso_ago 0)"
exp="$(epoch_in 86400)"

ddb put-item --table-name "$TABLE" --item \
  "{\"pk\":{\"S\":\"DRIVER#$driver\"},\"sk\":{\"S\":\"LOC#$ts\"},\"lat\":{\"N\":\"$lat\"},\"lng\":{\"N\":\"$lng\"},\"expires_at\":{\"N\":\"$exp\"}}"

ddb update-item --table-name "$TABLE" \
  --key "{\"pk\":{\"S\":\"DRIVER#$driver\"},\"sk\":{\"S\":\"STATE\"}}" \
  --update-expression "SET lat = :la, lng = :ln, updated_at = :t" \
  --expression-attribute-values "{\":la\":{\"N\":\"$lat\"},\":ln\":{\"N\":\"$lng\"},\":t\":{\"S\":\"$ts\"}}" >/dev/null
echo "Đã ghi vị trí của $driver lúc $ts."
