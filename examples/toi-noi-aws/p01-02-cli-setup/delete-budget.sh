#!/usr/bin/env bash
# Xoá budget toinoi-lab-monthly (dọn lab bài 02). MẶC ĐỊNH CHỈ IN LỆNH; thêm --apply để xoá thật.
# Dùng: ./delete-budget.sh [profile] [--apply]
set -euo pipefail

profile="toinoi-lab"
apply=0
for arg in "$@"; do
  case "$arg" in
    --apply) apply=1 ;;
    -*) echo "Tuỳ chọn không hợp lệ: $arg" >&2; exit 2 ;;
    *) profile="$arg" ;;
  esac
done
export AWS_PAGER=""

account="$(aws sts get-caller-identity --profile "$profile" --query Account --output text)"
cmd=(aws budgets delete-budget
  --account-id "$account"
  --budget-name toinoi-lab-monthly
  --profile "$profile")

if [ "$apply" -eq 1 ]; then
  "${cmd[@]}"
  echo "Đã xoá budget toinoi-lab-monthly."
else
  echo "Dry-run: sẽ chạy lệnh sau (thêm --apply để xoá thật)."
  printf '  '; printf '%q ' "${cmd[@]}"
  printf '\n'
fi
