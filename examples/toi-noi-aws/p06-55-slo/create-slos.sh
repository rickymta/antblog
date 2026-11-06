#!/usr/bin/env bash
# Tạo hai SLO của API đặt chuyến bằng CloudWatch Application Signals.
# Mặc định CHỈ IN lệnh; thêm --apply để chạy. Cần service `toinoi-booking` đã được instrument
# (ADOT hoặc CloudWatch agent bật Application Signals) và đã xuất hiện trong trang Services.
# Dùng: ./create-slos.sh [profile] [--apply]
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
profile="toinoi-lab"
apply=""
for arg in "$@"; do
  case "$arg" in
    --apply) apply="yes" ;;
    *) profile="$arg" ;;
  esac
done
export AWS_PAGER=""

run() {
  if [ -n "$apply" ]; then
    "$@"
  else
    printf '   '
    printf '%q ' "$@"
    echo
  fi
}

if [ -z "$apply" ]; then
  echo "Chế độ in lệnh (chưa tạo gì). Thêm --apply để thực hiện."
fi

echo "== 0. Bật khám phá service của Application Signals (một lần mỗi tài khoản và region)"
run aws application-signals start-discovery --profile "$profile"

for slo in booking-availability booking-latency-p99; do
  echo "== Tạo SLO từ slo/$slo.json"
  run aws application-signals create-service-level-objective \
    --cli-input-json "file://$here/slo/$slo.json" --profile "$profile"
done

echo "== Kiểm: danh sách SLO và budget còn lại"
run aws application-signals list-service-level-objectives --profile "$profile"
