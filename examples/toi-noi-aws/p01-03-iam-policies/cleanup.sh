#!/usr/bin/env bash
# Dọn lab bài 03: gỡ policy khỏi role rồi xoá policy và role.
# MẶC ĐỊNH CHỈ IN LỆNH (dry-run). Thêm --apply để xoá thật.
# Dùng: ./cleanup.sh [profile] [--apply]
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

role="toinoi-lab-delivery-photo"
policy="toinoi-lab-delivery-photo-rw"
account="$(aws sts get-caller-identity --profile "$profile" --query Account --output text)"
policy_arn="arn:aws:iam::${account}:policy/${policy}"

if [ "$apply" -eq 1 ]; then
  aws iam detach-role-policy --role-name "$role" --policy-arn "$policy_arn" --profile "$profile"
  aws iam delete-policy --policy-arn "$policy_arn" --profile "$profile"
  aws iam delete-role --role-name "$role" --profile "$profile"
  echo "Đã xoá role $role và policy $policy_arn."
else
  echo "Dry-run: sẽ chạy các lệnh sau (thêm --apply để xoá thật)."
  echo "  aws iam detach-role-policy --role-name $role --policy-arn $policy_arn --profile $profile"
  echo "  aws iam delete-policy --policy-arn $policy_arn --profile $profile"
  echo "  aws iam delete-role --role-name $role --profile $profile"
fi
