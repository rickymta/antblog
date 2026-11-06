#!/usr/bin/env bash
# Minh hoạ usage plan + API key cho đối tác quán. Chỉ dùng được với REST API (HTTP API không có usage plan).
# Cần một REST API và stage đã deploy; điền REST_API_ID và STAGE trước khi chạy trong tài khoản lab.
set -euo pipefail

export AWS_REGION="${AWS_REGION:-ap-southeast-1}"
REST_API_ID="${REST_API_ID:?Đặt REST_API_ID của REST API đã có}"
STAGE="${STAGE:-prod}"

PLAN_ID="$(aws apigateway create-usage-plan \
  --name toinoi-partner-basic \
  --throttle burstLimit=20,rateLimit=10 \
  --quota limit=100000,period=MONTH \
  --api-stages apiId="$REST_API_ID",stage="$STAGE" \
  --tags Project=ToiNoi,Env=lab,Service=gateway,Owner=platform \
  --query id --output text)"

KEY_ID="$(aws apigateway create-api-key \
  --name toinoi-partner-quan-an-001 --enabled \
  --query id --output text)"

aws apigateway create-usage-plan-key \
  --usage-plan-id "$PLAN_ID" --key-id "$KEY_ID" --key-type API_KEY

echo "Plan: $PLAN_ID, key: $KEY_ID. Lấy giá trị key để gửi cho đối tác:"
echo "aws apigateway get-api-key --api-key $KEY_ID --include-value --query value --output text"
