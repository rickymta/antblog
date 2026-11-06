#!/usr/bin/env bash
# Lab bài 16: publish BookingCreated lên SNS rồi nhận từ hai queue.
# Chạy SAU khi `terraform apply` xong, với profile AWS của tài khoản lab.
set -euo pipefail

export AWS_REGION="${AWS_REGION:-ap-southeast-1}"

TOPIC_ARN="$(terraform output -raw topic_arn)"
DISPATCH_URL="$(terraform output -json queue_urls | jq -r '."dispatch-push"')"
STATS_URL="$(terraform output -json queue_urls | jq -r '.stats')"

publish() { # $1 = city, $2 = service, $3 = tripId
  aws sns publish \
    --topic-arn "$TOPIC_ARN" \
    --message "{\"type\":\"BookingCreated\",\"tripId\":\"$3\",\"city\":\"$1\",\"service\":\"$2\"}" \
    --message-attributes "{\"city\":{\"DataType\":\"String\",\"StringValue\":\"$1\"},\"service\":{\"DataType\":\"String\",\"StringValue\":\"$2\"},\"eventId\":{\"DataType\":\"String\",\"StringValue\":\"evt-$3\"}}" \
    --query MessageId --output text
}

echo "== Publish 3 chuyến: HCM/ride, HN/food, DN/parcel"
publish HCM ride 1001
publish HN food 1002
publish DN parcel 1003

echo "== Queue stats nhận đủ 3 message (long polling 20 giây)"
aws sqs receive-message --queue-url "$STATS_URL" \
  --max-number-of-messages 10 --wait-time-seconds 20 \
  --message-attribute-names All \
  --query 'Messages[].Body' --output text

echo "== Queue dispatch-push chỉ nhận HCM/ride và HN/food (không có DN/parcel)"
aws sqs receive-message --queue-url "$DISPATCH_URL" \
  --max-number-of-messages 10 --wait-time-seconds 20 \
  --message-attribute-names All \
  --query 'Messages[].Body' --output text

echo "== Message vừa nhận ẩn trong visibility timeout; chưa delete nên sẽ hiện lại sau khi hết hạn."
echo "   Dùng 'aws sqs delete-message --queue-url ... --receipt-handle ...' để xác nhận đã xử lý."
