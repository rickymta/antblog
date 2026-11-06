#!/usr/bin/env bash
# Tạo role toinoi-lab-delivery-photo, customer managed policy và gắn chúng với nhau.
# MẶC ĐỊNH CHỈ IN LỆNH (dry-run). Thêm --apply để tạo thật.
# Dùng: ./create-role.sh [profile] [--apply]
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
export AWS_PAGER=""

role="toinoi-lab-delivery-photo"
policy="toinoi-lab-delivery-photo-rw"
account="$(aws sts get-caller-identity --profile "$profile" --query Account --output text)"
policy_arn="arn:aws:iam::${account}:policy/${policy}"
tags=("Key=Project,Value=ToiNoi" "Key=Env,Value=lab" "Key=Service,Value=delivery-photo" "Key=Owner,Value=platform")

cmds=(
  "aws iam create-role --role-name $role --assume-role-policy-document file://$here/trust-policy-delivery-photo.json --tags ${tags[*]} --profile $profile"
  "aws iam create-policy --policy-name $policy --policy-document file://$here/delivery-photo-policy.json --tags ${tags[*]} --profile $profile"
  "aws iam attach-role-policy --role-name $role --policy-arn $policy_arn --profile $profile"
)

if [ "$apply" -eq 1 ]; then
  aws iam create-role --role-name "$role" \
    --assume-role-policy-document "file://$here/trust-policy-delivery-photo.json" \
    --tags "${tags[@]}" --profile "$profile"
  aws iam create-policy --policy-name "$policy" \
    --policy-document "file://$here/delivery-photo-policy.json" \
    --tags "${tags[@]}" --profile "$profile"
  aws iam attach-role-policy --role-name "$role" --policy-arn "$policy_arn" --profile "$profile"
  echo "Đã tạo role $role và policy $policy_arn."
  echo "Dọn lại bằng: ./cleanup.sh $profile --apply"
else
  echo "Dry-run: sẽ chạy các lệnh sau (thêm --apply để tạo thật)."
  printf '  %s\n' "${cmds[@]}"
fi
