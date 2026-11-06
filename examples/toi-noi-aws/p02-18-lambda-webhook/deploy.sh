#!/usr/bin/env bash
# Lab bài 18: tạo hàng đợi, secret, hai role, hai hàm Lambda và Function URL bằng AWS CLI.
# CHỈ chạy trong tài khoản lab. Script dừng ngay khi có lỗi (set -e); dọn bằng ./cleanup.sh.
set -euo pipefail

# Chạy từ thư mục của lab dù gọi script từ đâu: mọi đường dẫn bên dưới là tương đối.
cd "$(dirname "$0")"

export AWS_REGION="${AWS_REGION:-ap-southeast-1}"
ENV_NAME="${ENV_NAME:-lab}"
PREFIX="toinoi-${ENV_NAME}"
TAGS_KV="Project=ToiNoi,Env=${ENV_NAME},Service=payments,Owner=platform"
RUNTIME="${RUNTIME:-nodejs22.x}"
ACCOUNT_ID="$(aws sts get-caller-identity --query Account --output text)"

mkdir -p build

echo "== 1. SQS: queue sự kiện thanh toán và DLQ"
DLQ_URL="$(aws sqs create-queue --queue-name "${PREFIX}-payment-events-dlq" \
  --attributes MessageRetentionPeriod=1209600 --tags "${TAGS_KV}" --query QueueUrl --output text)"
DLQ_ARN="$(aws sqs get-queue-attributes --queue-url "$DLQ_URL" --attribute-names QueueArn --query Attributes.QueueArn --output text)"

# Visibility timeout 60 giây = 6 lần timeout của hàm consumer (10 giây).
cat > build/queue-attributes.json <<JSON
{
  "VisibilityTimeout": "60",
  "ReceiveMessageWaitTimeSeconds": "20",
  "RedrivePolicy": "{\"deadLetterTargetArn\":\"${DLQ_ARN}\",\"maxReceiveCount\":\"5\"}"
}
JSON
QUEUE_URL="$(aws sqs create-queue --queue-name "${PREFIX}-payment-events" \
  --attributes file://build/queue-attributes.json --tags "${TAGS_KV}" --query QueueUrl --output text)"
QUEUE_ARN="$(aws sqs get-queue-attributes --queue-url "$QUEUE_URL" --attribute-names QueueArn --query Attributes.QueueArn --output text)"

echo "== 2. Secrets Manager: secret ký webhook (giá trị ngẫu nhiên, không in ra)"
SECRET_NAME="toinoi/${ENV_NAME}/vixanh-webhook-secret"
# Ghi giá trị ra tệp quyền 600 rồi truyền bằng file:// để secret không hiện trong danh sách tiến trình.
(umask 077 && openssl rand -hex 32 | tr -d '\n' > build/webhook-secret.txt)
SECRET_ARN="$(aws secretsmanager create-secret --name "$SECRET_NAME" \
  --secret-string file://build/webhook-secret.txt \
  --tags Key=Project,Value=ToiNoi Key=Env,Value="${ENV_NAME}" Key=Service,Value=payments Key=Owner,Value=platform \
  --query ARN --output text)"
rm -f build/webhook-secret.txt

make_role() { # $1 = tên role, $2 = tệp policy mẫu
  local role="$1" tpl="$2"
  sed -e "s|\${QUEUE_ARN}|${QUEUE_ARN}|g" -e "s|\${SECRET_ARN}|${SECRET_ARN}|g" "$tpl" > "build/${role}.json"
  aws iam create-role --role-name "$role" \
    --assume-role-policy-document file://trust-policy.json \
    --tags Key=Project,Value=ToiNoi Key=Env,Value="${ENV_NAME}" Key=Service,Value=payments Key=Owner,Value=platform >/dev/null
  aws iam attach-role-policy --role-name "$role" \
    --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole
  aws iam put-role-policy --role-name "$role" --policy-name inline-least-privilege \
    --policy-document "file://build/${role}.json"
}

echo "== 3. IAM: một role cho mỗi hàm (quyền tối thiểu)"
make_role "${PREFIX}-vixanh-webhook" webhook-permissions.json
make_role "${PREFIX}-payment-consumer" consumer-permissions.json
echo "Chờ IAM lan truyền..." && sleep 15

echo "== 4. Đóng gói và tạo hàm webhook"
(cd src/webhook && zip -q -r ../../build/webhook.zip .)
aws lambda create-function \
  --function-name "${PREFIX}-vixanh-webhook" \
  --runtime "$RUNTIME" --architectures arm64 \
  --handler index.handler \
  --zip-file fileb://build/webhook.zip \
  --role "arn:aws:iam::${ACCOUNT_ID}:role/${PREFIX}-vixanh-webhook" \
  --timeout 10 --memory-size 256 \
  --environment "Variables={QUEUE_URL=${QUEUE_URL},SECRET_ID=${SECRET_NAME}}" \
  --tags "${TAGS_KV}" >/dev/null

echo "== 5. Function URL cho webhook (AuthType NONE: bảo vệ bằng chữ ký HMAC trong hàm)"
FUNCTION_URL="$(aws lambda create-function-url-config \
  --function-name "${PREFIX}-vixanh-webhook" --auth-type NONE \
  --query FunctionUrl --output text)"
aws lambda add-permission --function-name "${PREFIX}-vixanh-webhook" \
  --statement-id FunctionURLAllowPublicAccess \
  --action lambda:InvokeFunctionUrl --principal "*" \
  --function-url-auth-type NONE >/dev/null
# Function URL công khai mới cần thêm quyền InvokeFunction giới hạn cho lời gọi qua URL.
aws lambda add-permission --function-name "${PREFIX}-vixanh-webhook" \
  --statement-id FunctionURLInvokeFunction \
  --action lambda:InvokeFunction --principal "*" \
  --invoked-via-function-url >/dev/null

echo "== 6. Hàm consumer và event source mapping từ SQS"
(cd src/consumer && zip -q -r ../../build/consumer.zip .)
aws lambda create-function \
  --function-name "${PREFIX}-payment-consumer" \
  --runtime "$RUNTIME" --architectures arm64 \
  --handler index.handler \
  --zip-file fileb://build/consumer.zip \
  --role "arn:aws:iam::${ACCOUNT_ID}:role/${PREFIX}-payment-consumer" \
  --timeout 10 --memory-size 256 \
  --tags "${TAGS_KV}" >/dev/null

aws lambda create-event-source-mapping \
  --function-name "${PREFIX}-payment-consumer" \
  --event-source-arn "$QUEUE_ARN" \
  --batch-size 10 --maximum-batching-window-in-seconds 5 \
  --function-response-types ReportBatchItemFailures \
  --scaling-config MaximumConcurrency=5 >/dev/null

cat <<MSG

Xong. Function URL: ${FUNCTION_URL}
Thử webhook (lấy secret bằng quyền admin của bạn, không dán secret vào lịch sử shell chung):
  SECRET="\$(aws secretsmanager get-secret-value --secret-id ${SECRET_NAME} --query SecretString --output text)"
  SIG="\$(node sign.mjs "\$SECRET" sample-webhook.json)"
  curl -sS -X POST "${FUNCTION_URL}" -H "x-vixanh-signature: \$SIG" --data-binary @sample-webhook.json
  curl -sS -X POST "${FUNCTION_URL}" -H "x-vixanh-signature: t=1,v1=00" --data-binary @sample-webhook.json
Xem log consumer: aws logs tail /aws/lambda/${PREFIX}-payment-consumer --since 5m
MSG
