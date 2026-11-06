#!/bin/sh
# Chạy khi server đã mở với 2 worker:
#   uv run --project examples/toi-noi-python/31-fastapi-endpoint-bao-gia uvicorn app.main:app \
#       --app-dir examples/toi-noi-python/31-fastapi-endpoint-bao-gia --port 8031 --workers 2
# 12 POST song song (mỗi lần một kết nối mới), rồi 6 GET QT-0001. Kết quả phụ thuộc cách hệ điều hành
# chia kết nối cho hai worker nên số lần trùng có thể khác; một worker duy nhất thì không bao giờ trùng.
URL=${1:-http://127.0.0.1:8031}
fmt='import sys,json; d=json.load(sys.stdin); print(d["quote_id"], d["distance_km"], "km, fare", d["fare"])'
for i in 1 2 3 4 5 6 7 8 9 10 11 12; do
  ( curl -s -m 10 -X POST "$URL/v1/quotes" -H "Content-Type: application/json" \
      -d "{\"distance_km\": \"$((i + 2))\"}" | python3 -c "$fmt" ) &
done | sort
wait
echo "--- GET QT-0001, 6 lần"
for i in 1 2 3 4 5 6; do curl -s -m 10 "$URL/v1/quotes/QT-0001" | python3 -c "$fmt"; done
