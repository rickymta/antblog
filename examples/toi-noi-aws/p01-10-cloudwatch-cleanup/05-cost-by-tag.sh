#!/usr/bin/env bash
set -euo pipefail
# Chi phí 7 ngày gần nhất nhóm theo tag Project. Mỗi lần gọi API Cost Explorer có thể bị tính phí nhỏ, đừng đặt vào vòng lặp.
source "$(dirname "$0")/common.sh"
END="$(date -u +%F)"
START="$(date -u -d '-7 days' +%F 2>/dev/null || date -u -v-7d +%F)"
aws ce get-cost-and-usage --region us-east-1 \
  --time-period "Start=$START,End=$END" --granularity DAILY --metrics UnblendedCost \
  --group-by Type=TAG,Key=Project \
  --query 'ResultsByTime[].[TimePeriod.Start,Groups[].[Keys[0],Metrics.UnblendedCost.Amount]]' --output json
