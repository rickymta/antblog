#!/usr/bin/env bash
# Tìm ai đã xoá VPC Flow Logs trong 90 ngày qua, bằng CloudTrail event history (không cần CloudTrail Lake).
# Chỉ ĐỌC. Event history chỉ giữ management event 90 ngày và theo từng region.
# Dùng: ./who-disabled-flow-logs.sh [profile] [region]
set -euo pipefail

profile="${1:-toinoi-lab}"
region="${2:-ap-southeast-1}"
export AWS_PAGER=""

aws cloudtrail lookup-events --profile "$profile" --region "$region" \
  --lookup-attributes AttributeKey=EventName,AttributeValue=DeleteFlowLogs \
  --max-results 50 --output json |
  jq -r '.Events[] | .CloudTrailEvent | fromjson |
    [.eventTime, (.userIdentity.arn // .userIdentity.type), (.userIdentity.accessKeyId // "-"),
     .sourceIPAddress, .userAgent, (.errorCode // "OK")] | @tsv'

echo "Cột: thời gian, principal, access key, IP nguồn, user agent, kết quả. Không có dòng nào nghĩa là không có DeleteFlowLogs trong 90 ngày ở region này."
