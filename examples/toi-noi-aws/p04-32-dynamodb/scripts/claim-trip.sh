#!/usr/bin/env bash
# Nhận chuyến: ba thao tác trong một transaction. Chạy hai lần với hai tài xế khác nhau để thấy chỉ một người thắng.
# Dùng: ./claim-trip.sh <trip-id> <driver-id> <idempotency-token>
source "$(dirname "$0")/common.sh"

trip="${1:?trip id, ví dụ t9001}"
driver="${2:?driver id, ví dụ d001}"
token="${3:?idempotency token, ví dụ claim-t9001-d001}"
now="$(iso_ago 0)"
exp="$(epoch_in 86400)"

items=$(cat <<JSON
[
  {"Update": {
    "TableName": "$TABLE",
    "Key": {"pk": {"S": "TRIP#$trip"}, "sk": {"S": "META"}},
    "UpdateExpression": "SET #s = :accepted, driver_id = :d, accepted_at = :now",
    "ConditionExpression": "#s = :searching AND attribute_not_exists(driver_id)",
    "ExpressionAttributeNames": {"#s": "status"},
    "ExpressionAttributeValues": {":accepted": {"S": "ACCEPTED"}, ":searching": {"S": "SEARCHING"}, ":d": {"S": "$driver"}, ":now": {"S": "$now"}}
  }},
  {"Update": {
    "TableName": "$TABLE",
    "Key": {"pk": {"S": "DRIVER#$driver"}, "sk": {"S": "STATE"}},
    "UpdateExpression": "SET #s = :busy, trip_id = :t REMOVE gsi1pk, gsi1sk",
    "ConditionExpression": "#s = :available",
    "ExpressionAttributeNames": {"#s": "status"},
    "ExpressionAttributeValues": {":busy": {"S": "BUSY"}, ":available": {"S": "AVAILABLE"}, ":t": {"S": "$trip"}}
  }},
  {"Put": {
    "TableName": "$TABLE",
    "Item": {"pk": {"S": "IDEMP#$token"}, "sk": {"S": "ACCEPT"}, "trip_id": {"S": "$trip"}, "driver_id": {"S": "$driver"}, "expires_at": {"N": "$exp"}},
    "ConditionExpression": "attribute_not_exists(pk)"
  }}
]
JSON
)

# --client-request-token: gọi lại cùng token và cùng nội dung trong 10 phút không áp dụng lần hai.
if ddb transact-write-items --transact-items "$items" --client-request-token "$token" >/dev/null; then
  echo "OK: $driver nhận chuyến $trip."
else
  echo "Thất bại: chuyến đã có người nhận, hoặc tài xế không còn AVAILABLE, hoặc token đã dùng. Xem CancellationReasons ở trên."
  exit 1
fi
