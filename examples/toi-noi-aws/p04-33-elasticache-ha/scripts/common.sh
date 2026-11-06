#!/usr/bin/env bash
# Biến dùng chung.
set -euo pipefail

export AWS_REGION="${AWS_REGION:-ap-southeast-1}"
export AWS_DEFAULT_REGION="$AWS_REGION"
export AWS_PAGER=""
RG="${RG:-toinoi-lab-ha}"

# Tìm client dòng lệnh: valkey-cli hoặc redis-cli (cluster mode cần cờ -c).
if command -v valkey-cli >/dev/null 2>&1; then
  CLI=valkey-cli
else
  CLI=redis-cli
fi

# Cờ kết nối: REDIS_TLS=0 để thử với Redis cục bộ không TLS.
CLI_ARGS=()
if [[ "${REDIS_TLS:-1}" == "1" ]]; then
  CLI_ARGS+=(--tls)
fi
if [[ "${REDIS_CLUSTER:-1}" == "1" ]]; then
  CLI_ARGS+=(-c)
fi

cli() {
  # shellcheck disable=SC2086
  "$CLI" -h "${ENDPOINT:?đặt ENDPOINT=<configuration endpoint>}" -p "${PORT:-6379}" ${CLI_ARGS[@]+"${CLI_ARGS[@]}"} "$@"
}
