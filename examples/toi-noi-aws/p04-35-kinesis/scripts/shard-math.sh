#!/usr/bin/env bash
# Tính số shard tối thiểu cho Kinesis Data Streams theo hai giới hạn ghi của mỗi shard:
# 1.000 bản ghi/giây và 1 MiB/giây. Dùng: ./shard-math.sh [tài xế] [chu kỳ giây] [byte mỗi bản ghi] [dự phòng %]
# Ví dụ Tới Nơi: ./shard-math.sh 25000 4 200 30   rồi ./shard-math.sh 50000 4 200 30
set -euo pipefail

drivers="${1:-25000}"
interval="${2:-4}"
bytes="${3:-200}"
headroom="${4:-30}"

ceil_div() { echo $((($1 + $2 - 1) / $2)); }

rps=$((drivers / interval))
by_records="$(ceil_div "$rps" 1000)"
by_bytes="$(ceil_div $((rps * bytes)) 1048576)"
need=$((by_records > by_bytes ? by_records : by_bytes))
with_headroom="$(ceil_div $((need * (100 + headroom))) 100)"
if ((by_records >= by_bytes)); then limit="số bản ghi/giây"; else limit="băng thông"; fi

echo "Tải ghi: $rps bản ghi/giây, khoảng $((rps * bytes)) byte/giây"
echo "Shard theo số bản ghi: $by_records; theo băng thông: $by_bytes"
echo "Tối thiểu: $need shard (giới hạn bởi $limit); cộng dự phòng $headroom%: $with_headroom shard"
