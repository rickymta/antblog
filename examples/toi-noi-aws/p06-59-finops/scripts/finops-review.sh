#!/usr/bin/env bash
# Chuẩn bị số liệu cho buổi review FinOps tháng. CHỈ chạy lệnh đọc, không tạo hay mua gì.
# Lưu ý: mỗi lần gọi Cost Explorer API có thể bị tính phí nhỏ (xem trang giá).
# Dùng: AWS_PROFILE=lab scripts/finops-review.sh [START END]  (YYYY-MM-DD, END không bao gồm)
set -euo pipefail  # lệnh lỗi trong run() đã có "|| echo" nên không làm dừng script

START="${1:-$(date -u -v-1m +%Y-%m-01 2>/dev/null || date -u -d 'last month' +%Y-%m-01)}"
END="${2:-$(date -u +%Y-%m-01)}"
echo "Khoảng thời gian: $START -> $END"

run() {
  echo
  echo "== $1"
  shift
  "$@" || echo "(bỏ qua: lệnh lỗi hoặc dịch vụ chưa bật opt-in)"
}

run "Chi phí theo tag Service" \
  aws ce get-cost-and-usage \
  --time-period "Start=$START,End=$END" --granularity MONTHLY \
  --metrics UnblendedCost --group-by Type=TAG,Key=Service \
  --query 'ResultsByTime[].Groups[].[Keys[0],Metrics.UnblendedCost.Amount]' --output table

run "Savings Plans coverage" \
  aws ce get-savings-plans-coverage \
  --time-period "Start=$START,End=$END" --granularity MONTHLY \
  --query 'SavingsPlansCoverages[].Coverage.CoveragePercentage' --output text

run "Savings Plans utilization" \
  aws ce get-savings-plans-utilization \
  --time-period "Start=$START,End=$END" \
  --query 'Total.Utilization.UtilizationPercentage' --output text

run "Khuyến nghị mua Compute Savings Plans (30 ngày gần nhất, 1 năm, no upfront)" \
  aws ce get-savings-plans-purchase-recommendation \
  --savings-plans-type COMPUTE_SP --term-in-years ONE_YEAR \
  --payment-option NO_UPFRONT --lookback-period-in-days THIRTY_DAYS \
  --query 'SavingsPlansPurchaseRecommendation.SavingsPlansPurchaseRecommendationSummary' --output json

run "Cost Optimization Hub: 20 khuyến nghị đầu" \
  aws cost-optimization-hub list-recommendations --region us-east-1 --max-results 20 --output json

echo
echo "Xong. Ghi kết quả vào bảng review: chi phí mỗi chuyến, coverage, utilization, 2-3 việc giảm."
