#!/usr/bin/env bash
# Gửi N bản ghi vị trí mẫu bằng PutRecords. Partition key là driverId để cùng tài xế vào cùng shard.
# Dùng: ./put-sample.sh [số bản ghi, mặc định 20, tối đa 500 mỗi lệnh PutRecords]
source "$(dirname "$0")/common.sh"

n="${1:-20}"
cities=(hcm hn dn)
records="[]"

for i in $(seq 1 "$n"); do
  driver="$(printf 'd%03d' $((i % 50 + 1)))"
  city="${cities[$((i % 3))]}"
  json="$(jq -nc --arg d "$driver" --arg c "$city" --argjson ts "$(($(date +%s) * 1000 + i))" \
    '{driver_id:$d, trip_id:"t9001", city:$c, lat:10.7769, lng:106.7009, speed:28.5, ts_ms:$ts}')"
  records="$(jq -c --arg k "$driver" --arg data "$json" \
    '. + [{PartitionKey:$k, Data:($data|@base64)}]' <<<"$records")"
done

# Data đã mã hoá base64, đúng định dạng đầu vào mặc định của AWS CLI cho kiểu blob.
aws kinesis put-records --stream-name "$STREAM" --records "$records" \
  --query '{Gui:length(Records), Loi:FailedRecordCount}' --output json
