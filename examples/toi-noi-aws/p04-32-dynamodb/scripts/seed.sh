#!/usr/bin/env bash
# Nạp dữ liệu mẫu: 3 tài xế ở TP.HCM, lịch sử vị trí, và 2 chuyến đang tìm tài xế.
source "$(dirname "$0")/common.sh"

exp_loc="$(epoch_in 86400)" # lịch sử vị trí sống 24 giờ
exp_state="$(epoch_in 86400)"
cell="w3gvk" # ô geohash độ chính xác 5, ví dụ minh hoạ cho khu trung tâm TP.HCM

put_state() { # id status lat lng
  local id="$1" status="$2" lat="$3" lng="$4" item
  if [[ "$status" == "AVAILABLE" ]]; then
    item=$(cat <<JSON
{"pk":{"S":"DRIVER#$id"},"sk":{"S":"STATE"},"city":{"S":"hcm"},"status":{"S":"$status"},
 "lat":{"N":"$lat"},"lng":{"N":"$lng"},"cell":{"S":"$cell"},"updated_at":{"S":"$(iso_ago 2)"},
 "gsi1pk":{"S":"CITY#hcm#AVAILABLE#$cell"},"gsi1sk":{"S":"$id"},"expires_at":{"N":"$exp_state"}}
JSON
)
  else
    item=$(cat <<JSON
{"pk":{"S":"DRIVER#$id"},"sk":{"S":"STATE"},"city":{"S":"hcm"},"status":{"S":"$status"},
 "lat":{"N":"$lat"},"lng":{"N":"$lng"},"cell":{"S":"$cell"},"updated_at":{"S":"$(iso_ago 2)"},
 "expires_at":{"N":"$exp_state"}}
JSON
)
  fi
  ddb put-item --table-name "$TABLE" --item "$item"
}

put_loc() { # id seconds_ago lat lng
  local id="$1" ago="$2" lat="$3" lng="$4" ts
  ts="$(iso_ago "$ago")"
  ddb put-item --table-name "$TABLE" --item \
    "{\"pk\":{\"S\":\"DRIVER#$id\"},\"sk\":{\"S\":\"LOC#$ts\"},\"lat\":{\"N\":\"$lat\"},\"lng\":{\"N\":\"$lng\"},\"speed\":{\"N\":\"28\"},\"expires_at\":{\"N\":\"$exp_loc\"}}"
}

put_state d001 AVAILABLE 10.7769 106.7009
put_state d002 AVAILABLE 10.7801 106.6992
put_state d003 BUSY 10.7712 106.7040

# Lịch sử của d001: 20 điểm, cách nhau 60 giây, trải 20 phút
for i in $(seq 0 19); do
  put_loc d001 $((i * 60 + 2)) "10.77$((i % 10))" "106.70$((i % 10))"
done

for t in t9001 t9002; do
  ddb put-item --table-name "$TABLE" --item \
    "{\"pk\":{\"S\":\"TRIP#$t\"},\"sk\":{\"S\":\"META\"},\"status\":{\"S\":\"SEARCHING\"},\"city\":{\"S\":\"hcm\"},\"expires_at\":{\"N\":\"$exp_state\"}}"
done
echo "Đã nạp dữ liệu mẫu vào $TABLE."
