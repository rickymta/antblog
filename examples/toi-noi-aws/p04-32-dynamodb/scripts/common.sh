#!/usr/bin/env bash
# Biến dùng chung. Đặt DDB_ENDPOINT=http://localhost:8000 để chạy với DynamoDB Local thay vì AWS.
set -euo pipefail

export AWS_REGION="${AWS_REGION:-ap-southeast-1}"
export AWS_DEFAULT_REGION="$AWS_REGION"
export AWS_PAGER=""
TABLE="${TABLE:-toinoi-lab-dispatch}"

EP=()
if [[ -n "${DDB_ENDPOINT:-}" ]]; then
  EP=(--endpoint-url "$DDB_ENDPOINT")
fi

ddb() {
  aws dynamodb "$@" ${EP[@]+"${EP[@]}"}
}

# Thời điểm cách bây giờ N giây, dạng ISO 8601 UTC (chạy được trên GNU date và BSD date).
iso_ago() {
  date -u -d "$1 seconds ago" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -v-"$1"S +%Y-%m-%dT%H:%M:%SZ
}

# Epoch giây của N giây nữa (dùng cho expires_at).
epoch_in() {
  echo $(($(date +%s) + $1))
}
