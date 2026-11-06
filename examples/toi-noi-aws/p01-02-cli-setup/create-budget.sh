#!/usr/bin/env bash
# Tạo budget chi phí hằng tháng cho tài khoản lab, cảnh báo qua email.
# MẶC ĐỊNH CHỈ IN LỆNH (dry-run). Thêm --apply để tạo thật.
# Dùng: BUDGET_EMAIL=ban@example.com [BUDGET_LIMIT_USD=10] ./create-budget.sh [profile] [--apply]
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
profile="toinoi-lab"
apply=0
for arg in "$@"; do
  case "$arg" in
    --apply) apply=1 ;;
    -*) echo "Tuỳ chọn không hợp lệ: $arg" >&2; exit 2 ;;
    *) profile="$arg" ;;
  esac
done

email="${BUDGET_EMAIL:-}"
limit="${BUDGET_LIMIT_USD:-10}"
export AWS_PAGER=""

if [ -z "$email" ]; then
  echo "Cần đặt BUDGET_EMAIL (email nhận cảnh báo)." >&2
  exit 2
fi
if ! [[ "$email" =~ ^[^@[:space:]/\"\&]+@[^@[:space:]/\"\&]+$ ]]; then
  echo "BUDGET_EMAIL không giống địa chỉ email." >&2
  exit 2
fi
if ! [[ "$limit" =~ ^[0-9]+$ ]] || [ "$limit" -lt 1 ]; then
  echo "BUDGET_LIMIT_USD phải là số nguyên dương." >&2
  exit 2
fi
if [ "$apply" -eq 1 ] && [[ "$email" == *@toinoi.example ]]; then
  echo "toinoi.example chỉ là tên miền minh hoạ; dùng email thật của bạn." >&2
  exit 2
fi

account="$(aws sts get-caller-identity --profile "$profile" --query Account --output text)"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
sed "s/\"Amount\": \"10\"/\"Amount\": \"$limit\"/" "$here/budget.json" >"$tmp/budget.json"
sed "s/lab-owner@toinoi.example/$email/g" "$here/notifications.json" >"$tmp/notifications.json"

cmd=(aws budgets create-budget
  --account-id "$account"
  --budget "file://$tmp/budget.json"
  --notifications-with-subscribers "file://$tmp/notifications.json"
  --profile "$profile")

if [ "$apply" -eq 1 ]; then
  "${cmd[@]}"
  echo "Đã tạo budget toinoi-lab-monthly (ngưỡng ${limit} USD) cho account $account."
  echo "Dọn lại bằng: ./delete-budget.sh $profile --apply"
else
  echo "Dry-run: sẽ chạy lệnh sau (thêm --apply để tạo thật)."
  printf '  '; printf '%q ' "${cmd[@]}"
  printf '\n\nbudget.json:\n'
  cat "$tmp/budget.json"
fi
