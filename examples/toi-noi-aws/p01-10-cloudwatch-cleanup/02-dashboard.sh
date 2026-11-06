#!/usr/bin/env bash
set -euo pipefail
# Tạo dashboard cho service quote từ dashboard.json.tpl. Dashboard tuỳ chỉnh tính tiền theo tháng
# (có hạn mức miễn phí, xem trang CloudWatch pricing); xoá nó khi không dùng.
source "$(dirname "$0")/common.sh"
confirm_identity

LB="$(alb_dim)"; TG="$(tg_dim)"
[[ -n "$LB" && -n "$TG" ]] || { echo "Cần ALB và target group của lab 08." >&2; exit 1; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
sed -e "s|__REGION__|$AWS_REGION|g" -e "s|__LB__|$LB|g" -e "s|__TG__|$TG|g" \
    -e "s|__ASG__|$ASG_NAME|g" -e "s|__DB__|$DB_ID|g" "$HERE/dashboard.json.tpl" > "$TMP/dashboard.json"
aws cloudwatch put-dashboard --dashboard-name "$DASHBOARD_NAME" --dashboard-body "file://$TMP/dashboard.json"
echo "Mở: https://${AWS_REGION}.console.aws.amazon.com/cloudwatch/home?region=${AWS_REGION}#dashboards/dashboard/${DASHBOARD_NAME}"
