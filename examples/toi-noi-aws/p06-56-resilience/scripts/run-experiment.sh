#!/usr/bin/env bash
# Chạy thí nghiệm FIS từ template của lab và theo dõi trạng thái. KHÔNG chạy tự động.
# Dùng: scripts/run-experiment.sh <experiment-template-id>
set -euo pipefail

TEMPLATE_ID="${1:?cần experiment template id (terraform output -raw fis_experiment_template_id)}"

EXPERIMENT_ID="$(aws fis start-experiment \
  --experiment-template-id "$TEMPLATE_ID" \
  --tags Project=ToiNoi,Env=lab \
  --query 'experiment.id' --output text)"
echo "Đã bắt đầu thí nghiệm $EXPERIMENT_ID"

while true; do
  STATUS="$(aws fis get-experiment --id "$EXPERIMENT_ID" \
    --query 'experiment.state.status' --output text)"
  echo "$(date -u +%H:%M:%S) $STATUS"
  case "$STATUS" in
    completed | stopped | failed | cancelled) break ;;
  esac
  sleep 15
done

# Dừng sớm bằng tay nếu cần: aws fis stop-experiment --id "$EXPERIMENT_ID"
