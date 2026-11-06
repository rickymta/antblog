#!/usr/bin/env bash
# Thử cache từ một máy trong VPC (ví dụ EC2 vào bằng Session Manager, bài 05).
# Cần redis-cli hoặc valkey-cli hỗ trợ TLS. Đặt CLI=valkey-cli nếu dùng valkey-cli.
# Nếu cache có AUTH token: export REDISCLI_AUTH=<token> trước khi chạy (không truyền token qua tham số dòng lệnh).
set -euo pipefail

HOST="${1:?Cách dùng: smoke-test.sh <endpoint>}"
CLI_BIN="${CLI:-redis-cli}"
HERE="$(cd "$(dirname "$0")" && pwd)"
r() { "${CLI_BIN}" -h "${HOST}" -p 6379 --tls "$@"; }

echo "== PING =="
r PING

echo "== Cache báo giá: key quote:v7:hcm:ride:5, TTL 60 giây cộng ngẫu nhiên 0-14 giây =="
TTL=$((60 + RANDOM % 15))
r SET quote:v7:hcm:ride:5 '{"fare_vnd":58000,"currency":"VND"}' EX "${TTL}"
r GET quote:v7:hcm:ride:5
r TTL quote:v7:hcm:ride:5

echo "== Session tài xế, TTL 1800 giây =="
r SET driver:session:drv-1001 '{"city":"hcm","status":"online"}' EX 1800
r TTL driver:session:drv-1001

echo "== Rate limit cửa sổ cố định: 5 request/10 giây, gọi 7 lần =="
WINDOW="$(($(date +%s) / 10))"
for i in 1 2 3 4 5 6 7; do
  printf 'lần %s: ' "${i}"
  r --eval "${HERE}/rate-limit.lua" "rl:user-42:${WINDOW}" , 10 5 | tr '\n' ' '
  echo
done

echo "== Rate limit cửa sổ trượt: 3 request/10 giây, gọi 5 lần =="
for i in 1 2 3 4 5; do
  NOW="$(date +%s%3N)"
  printf 'lần %s: ' "${i}"
  r --eval "${HERE}/sliding-window.lua" "rl:sw:user-42" , "${NOW}" 10000 3 "${NOW}-${RANDOM}" | tr '\n' ' '
  echo
done

echo "== Thống kê hit/miss (dùng để tính tỉ lệ cache hit) =="
r INFO stats | grep -E 'keyspace_hits|keyspace_misses|evicted_keys'
