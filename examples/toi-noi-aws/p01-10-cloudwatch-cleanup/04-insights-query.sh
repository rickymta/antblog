#!/usr/bin/env bash
# shellcheck disable=SC2016  # dấu backtick trong JMESPath là cố ý
set -euo pipefail
# Chạy một truy vấn Logs Insights: 20 dòng ERROR gần nhất trong 1 giờ qua. Mỗi truy vấn tính tiền theo dữ liệu log được quét.
source "$(dirname "$0")/common.sh"
END="$(date +%s)"; START=$((END - 3600))
QID="$(aws logs start-query --log-group-name "$LOG_GROUP" --start-time "$START" --end-time "$END" \
  --query-string 'fields @timestamp, @message | filter @message like /ERROR/ | sort @timestamp desc | limit 20' \
  --query 'queryId' --output text)"
echo "Query: $QID"
for _ in $(seq 1 20); do
  ST="$(aws logs get-query-results --query-id "$QID" --query status --output text)"
  [[ "$ST" == "Complete" ]] && break
  [[ "$ST" == "Failed" || "$ST" == "Cancelled" ]] && { echo "Truy vấn $ST" >&2; exit 1; }
  sleep 2
done
aws logs get-query-results --query-id "$QID" \
  --query 'results[*][?field==`@message`].value' --output text
