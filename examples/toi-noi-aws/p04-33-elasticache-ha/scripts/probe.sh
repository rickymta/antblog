#!/usr/bin/env bash
# Đo cửa sổ lỗi mà client thấy khi failover: ghi SET mỗi 200 ms, báo thời điểm lỗi đầu tiên và lần thành công trở lại.
# Chạy trên máy trong VPC trong khi ở cửa sổ khác chạy test-failover.sh.
# Dùng: ENDPOINT=<configuration endpoint> ./probe.sh [số giây, mặc định 120]
source "$(dirname "$0")/common.sh"

duration="${1:-120}"
end=$(($(date +%s) + duration))
first_fail=""
last_fail=""
fails=0
total=0

while [[ "$(date +%s)" -lt "$end" ]]; do
  total=$((total + 1))
  # Khoá có hash tag để biết chắc nó thuộc một shard cố định: {probe}
  if ! out="$(cli SET "{probe}:last" "$(date +%s)" EX 60 2>&1)" || [[ "$out" != "OK" ]]; then
    fails=$((fails + 1))
    [[ -z "$first_fail" ]] && first_fail="$(date +%s)"
    last_fail="$(date +%s)"
  fi
  sleep 0.2
done

echo "Tổng $total lần ghi, $fails lần lỗi."
if [[ -n "$first_fail" ]]; then
  echo "Cửa sổ lỗi: $((last_fail - first_fail + 1)) giây (từ lỗi đầu tới lỗi cuối)."
else
  echo "Không thấy lỗi nào."
fi
