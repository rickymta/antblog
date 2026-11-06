#!/usr/bin/env bash
# Lab bài 17: kiểm pattern ở máy, put-events, đọc queue đích, replay.
# Phần test-event-pattern gọi API EventBridge nhưng không tạo tài nguyên.
set -euo pipefail

export AWS_REGION="${AWS_REGION:-ap-southeast-1}"

echo "== 1. test-event-pattern: sự kiện mẫu có khớp pattern không?"
aws events test-event-pattern \
  --event-pattern file://event-pattern-completed.json \
  --event file://sample-event-completed.json

echo "== 2. put-events: 2 TripCompleted và 1 TripCancelled"
aws events put-events --entries file://put-events.json \
  --query '{Failed:FailedEntryCount,Ids:Entries[].EventId}'

echo "== 3. Nhận từ queue settlement (HCM 52500 và chuyến huỷ khớp, HN không khớp)"
QUEUE_URL="$(terraform output -raw settlement_queue_url)"
aws sqs receive-message --queue-url "$QUEUE_URL" \
  --max-number-of-messages 10 --wait-time-seconds 20 \
  --query 'Messages[].Body' --output text

echo "== 4. Replay 1 giờ gần nhất từ archive về chính bus này"
ARCHIVE_ARN="$(terraform output -raw archive_arn)"
BUS_ARN="$(terraform output -raw event_bus_arn)"
START="$(date -u -v-1H +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -d '1 hour ago' +%Y-%m-%dT%H:%M:%SZ)"
END="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
aws events start-replay \
  --replay-name "toinoi-lab-replay-$(date +%s)" \
  --event-source-arn "$ARCHIVE_ARN" \
  --destination "{\"Arn\":\"$BUS_ARN\"}" \
  --event-start-time "$START" \
  --event-end-time "$END"
echo "Replay chạy lại mọi rule khớp: consumer phải idempotent. Archive cần vài phút để có dữ liệu."
