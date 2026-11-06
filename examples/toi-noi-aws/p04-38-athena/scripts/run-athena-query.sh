#!/usr/bin/env bash
# Chạy một câu SQL trong workgroup của lab và in số byte Athena đã quét (căn cứ tính tiền).
# Dùng: AWS_PROFILE=<lab> ./run-athena-query.sh "SELECT count(*) FROM trips_curated WHERE dt='2026-03-02'"
set -euo pipefail

: "${AWS_PROFILE:?Đặt AWS_PROFILE trỏ vào tài khoản lab}"
SQL="${1:?Cần câu SQL làm tham số đầu tiên}"
REGION="${AWS_REGION:-ap-southeast-1}"
WORKGROUP="${WORKGROUP:-toinoi-lab-analytics}"
DATABASE="${DATABASE:-toinoi_lab_analytics}"

QID="$(aws athena start-query-execution --region "$REGION" \
  --work-group "$WORKGROUP" \
  --query-execution-context "Database=${DATABASE}" \
  --query-string "$SQL" \
  --query QueryExecutionId --output text)"
echo "QueryExecutionId: ${QID}"

while true; do
  STATE="$(aws athena get-query-execution --region "$REGION" --query-execution-id "$QID" \
    --query QueryExecution.Status.State --output text)"
  case "$STATE" in
    SUCCEEDED) break ;;
    FAILED|CANCELLED)
      aws athena get-query-execution --region "$REGION" --query-execution-id "$QID" \
        --query QueryExecution.Status.StateChangeReason --output text >&2
      exit 1
      ;;
  esac
  sleep 2
done

SCANNED="$(aws athena get-query-execution --region "$REGION" --query-execution-id "$QID" \
  --query QueryExecution.Statistics.DataScannedInBytes --output text)"
echo "Đã quét: ${SCANNED} byte"
aws athena get-query-results --region "$REGION" --query-execution-id "$QID" --max-items 20 --output table
