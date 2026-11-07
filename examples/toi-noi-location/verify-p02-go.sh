#!/usr/bin/env bash
# Kiểm tra phần Go của P02 (LOC-11, 12, 13, 14, 19, 20): vet, gofmt, test -race, và các lab chạy offline.
#
# Không bật Docker. Các việc KHÔNG nằm trong script này (cần Kafka/Docker hoặc chạy lâu):
#   - test có Kafka: đặt TOI_NOI_KAFKA=1 sau khi `docker compose -f infra/docker-compose.yml up -d --wait`
#   - đo 25.000 kết nối và sweep tải (LOC-12), pause Kafka (LOC-13), độ trễ đầu cuối qua hub (LOC-19):
#     lệnh nằm trong results/p02-go.md
#   - sinh lại protobuf: VERIFY_PROTO=1 (cần mạng lần đầu để tải buf)
set -euo pipefail
export PATH="/usr/local/go/bin:$PATH"
cd "$(dirname "$0")/go"
fail() { echo "FAIL: $*" >&2; exit 1; }

echo "== gofmt"
bad="$(gofmt -l .)"
[ -z "$bad" ] || fail "gofmt chưa sạch: $bad"

echo "== go vet"
go vet ./...

echo "== go test -race"
go test -race -count=1 ./...

echo "== LOC-11: race detector bắt được lỗi, bản sửa thì sạch"
out="$(go run -race ./cmd/loc11_nhap_mon race-bad 2>&1 || true)"
echo "$out" | grep -q "WARNING: DATA RACE" || fail "race-bad không bị -race bắt"
go run -race ./cmd/loc11_nhap_mon race-fixed | grep -q "channel: đếm được 800000" || fail "race-fixed sai kết quả"
go run -race ./cmd/loc11_nhap_mon pipeline | grep -q "hoàn tất" || fail "pipeline không hoàn tất"

echo "== LOC-13: thí nghiệm tắt êm và hàng đợi (rút gọn)"
go run ./cmd/loc13_backpressure --mode shutdown | tail -3
go run ./cmd/loc13_backpressure --variant coalesce --phases 2:10000,6:1000,3:12000 | grep '^S'

echo "== LOC-20: giảm mẫu, Parquet, retention (rút gọn)"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
go run ./cmd/history-writer demo --out "$tmp/h" --drivers 300 --minutes 5 --retention-span-days 40 | grep -E 'giảm mẫu|sai số|đã xoá'

echo "== LOC-12/14: ingest (sink discard) + loadgen nhỏ, có điểm xấu"
go build -o "$tmp/ingest" ./cmd/location-ingest
go build -o "$tmp/loadgen" ./cmd/loadgen
"$tmp/ingest" --sink discard --listen 127.0.0.1:18081 >"$tmp/ingest.log" 2>&1 &
pid=$!
trap 'kill $pid 2>/dev/null || true; rm -rf "$tmp"' EXIT
for _ in $(seq 1 40); do curl -sf http://127.0.0.1:18081/healthz >/dev/null && break; sleep 0.25; done
"$tmp/loadgen" --url ws://127.0.0.1:18081/ws --stats http://127.0.0.1:18081/debug/stats --drivers 300 --rate 150 \
  --batch 3 --duration 8s --warmup 8s --report-every 4s --fault-rate 0.02 --reorder-fraction 0.05 --mock-fraction 0.02 | grep '^STEP' | tail -1
stats="$(curl -s http://127.0.0.1:18081/debug/stats)"
echo "$stats" | grep -q '"TELEPORT"\|POINT_FLAG_TELEPORT' || fail "không thấy cờ teleport trong thống kê ingest"

if [ "${VERIFY_PROTO:-0}" = "1" ]; then
  echo "== protobuf sinh lại có khớp file đã commit không"
  before="$(shasum gen/location/v1/location.pb.go)"
  ../proto/generate.sh >/dev/null
  [ "$before" = "$(shasum gen/location/v1/location.pb.go)" ] || fail "location.pb.go khác sau khi sinh lại"
fi

if [ -n "${TOI_NOI_KAFKA:-}" ]; then
  echo "== test Kafka (TOI_NOI_KAFKA=${TOI_NOI_KAFKA})"
  go test -race -count=1 ./internal/kafkax
fi

echo "PASS"
