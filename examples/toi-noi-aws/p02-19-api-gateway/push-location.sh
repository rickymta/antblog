#!/usr/bin/env bash
# Đẩy một vị trí tài xế tới một kết nối WebSocket (mô phỏng service tracking).
# Cách dùng: ./push-location.sh <connectionId>
# Lấy connectionId từ bảng toinoi-lab-ws-connections (aws dynamodb scan) sau khi mở kết nối bằng wscat.
set -euo pipefail

export AWS_REGION="${AWS_REGION:-ap-southeast-1}"
CONNECTION_ID="${1:?Cách dùng: $0 <connectionId>}"
ENDPOINT="$(terraform output -raw websocket_management_endpoint)"

aws apigatewaymanagementapi post-to-connection \
  --endpoint-url "$ENDPOINT" \
  --connection-id "$CONNECTION_ID" \
  --cli-binary-format raw-in-base64-out \
  --data "$(printf '{"tripId":"1001","lat":10.7769,"lng":106.7009,"ts":%s}' "$(date +%s)")"
