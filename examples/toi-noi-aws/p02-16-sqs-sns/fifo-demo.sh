#!/usr/bin/env bash
# Lab bài 16: FIFO queue với MessageGroupId và MessageDeduplicationId.
# Tạo một FIFO queue riêng, gửi hai lần cùng deduplication id, chỉ nhận được một message.
set -euo pipefail

export AWS_REGION="${AWS_REGION:-ap-southeast-1}"
NAME="toinoi-lab-trip-status.fifo"

QUEUE_URL="$(aws sqs create-queue \
  --queue-name "$NAME" \
  --attributes FifoQueue=true,VisibilityTimeout=60,ReceiveMessageWaitTimeSeconds=20 \
  --tags Project=ToiNoi,Env=lab,Service=booking,Owner=platform \
  --query QueueUrl --output text)"

for i in 1 2; do
  aws sqs send-message \
    --queue-url "$QUEUE_URL" \
    --message-group-id "trip-1001" \
    --message-deduplication-id "trip-1001-accepted" \
    --message-body '{"tripId":"1001","status":"accepted"}' \
    --query MessageId --output text
done

echo "== Gửi 2 lần cùng deduplication id, nhận về:"
aws sqs receive-message --queue-url "$QUEUE_URL" \
  --max-number-of-messages 10 --wait-time-seconds 5 \
  --query 'length(Messages)' --output text

echo "== Dọn FIFO queue"
aws sqs delete-queue --queue-url "$QUEUE_URL"
