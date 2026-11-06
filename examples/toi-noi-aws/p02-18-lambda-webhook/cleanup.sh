#!/usr/bin/env bash
# Dọn mọi tài nguyên do deploy.sh tạo. Lỗi "không tồn tại" được bỏ qua (|| true) để chạy lại an toàn.
set -euo pipefail

# Chạy từ thư mục của lab để `rm -rf build` chỉ xoá build/ của lab.
cd "$(dirname "$0")"

export AWS_REGION="${AWS_REGION:-ap-southeast-1}"
ENV_NAME="${ENV_NAME:-lab}"
PREFIX="toinoi-${ENV_NAME}"

for fn in "${PREFIX}-payment-consumer" "${PREFIX}-vixanh-webhook"; do
  for uuid in $(aws lambda list-event-source-mappings --function-name "$fn" --query 'EventSourceMappings[].UUID' --output text 2>/dev/null || true); do
    aws lambda delete-event-source-mapping --uuid "$uuid" >/dev/null || true
  done
  aws lambda delete-function-url-config --function-name "$fn" 2>/dev/null || true
  aws lambda delete-function --function-name "$fn" 2>/dev/null || true
done

for role in "${PREFIX}-vixanh-webhook" "${PREFIX}-payment-consumer"; do
  aws iam delete-role-policy --role-name "$role" --policy-name inline-least-privilege 2>/dev/null || true
  aws iam detach-role-policy --role-name "$role" \
    --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole 2>/dev/null || true
  aws iam delete-role --role-name "$role" 2>/dev/null || true
done

for q in "${PREFIX}-payment-events" "${PREFIX}-payment-events-dlq"; do
  if url="$(aws sqs get-queue-url --queue-name "$q" --query QueueUrl --output text 2>/dev/null)"; then
    aws sqs delete-queue --queue-url "$url" || true
  fi
done

# --force-delete-without-recovery xoá ngay secret của lab (không có thời gian chờ khôi phục).
aws secretsmanager delete-secret --secret-id "toinoi/${ENV_NAME}/vixanh-webhook-secret" \
  --force-delete-without-recovery 2>/dev/null || true

aws logs delete-log-group --log-group-name "/aws/lambda/${PREFIX}-vixanh-webhook" 2>/dev/null || true
aws logs delete-log-group --log-group-name "/aws/lambda/${PREFIX}-payment-consumer" 2>/dev/null || true
rm -rf build
echo "Đã dọn xong."
