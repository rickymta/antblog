#!/usr/bin/env bash
set -euo pipefail
# Kích hoạt cost allocation tag và tạo budget tháng cảnh báo qua email.
# Cost Explorer và Budgets là API toàn cầu đặt ở us-east-1. Kích hoạt tag chỉ làm được ở tài khoản quản lý
# (hoặc tài khoản đơn lẻ), và tag chỉ xuất hiện để kích hoạt sau khi đã có tài nguyên mang tag đó trong dữ liệu billing.
source "$(dirname "$0")/common.sh"
confirm_identity
: "${ALERT_EMAIL:?}" "${BUDGET_AMOUNT_USD:?}"

log "Kích hoạt cost allocation tag (có thể báo lỗi nếu tag chưa xuất hiện trong billing, hãy chạy lại sau ~24 giờ)"
aws ce update-cost-allocation-tags-status --region us-east-1 --cost-allocation-tags-status \
  TagKey=Project,Status=Active TagKey=Env,Status=Active TagKey=Service,Status=Active \
  TagKey=Owner,Status=Active TagKey=CostCenter,Status=Active || \
  echo "(chưa kích hoạt được: xem README mục cost allocation tag)"

log "Budget $BUDGET_NAME: ngưỡng ${BUDGET_AMOUNT_USD} USD/tháng, email ở 80% thực tế và 100% dự báo"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
sed -e "s|__BUDGET_NAME__|$BUDGET_NAME|g" -e "s|__AMOUNT__|$BUDGET_AMOUNT_USD|g" "$HERE/budget.json.tpl" > "$TMP/budget.json"
sed -e "s|__EMAIL__|$ALERT_EMAIL|g" "$HERE/budget-notifications.json.tpl" > "$TMP/notifications.json"
if aws budgets describe-budget --region us-east-1 --account-id "$ACCOUNT_ID" --budget-name "$BUDGET_NAME" >/dev/null 2>&1; then
  echo "Budget $BUDGET_NAME đã có (thường do lab bài 02 tạo): giữ nguyên, không tạo trùng."
else
  aws budgets create-budget --region us-east-1 --account-id "$ACCOUNT_ID" \
    --budget "file://$TMP/budget.json" --notifications-with-subscribers "file://$TMP/notifications.json"
fi
aws budgets describe-budgets --region us-east-1 --account-id "$ACCOUNT_ID" \
  --query 'Budgets[].[BudgetName,BudgetLimit.Amount,TimeUnit]' --output table
