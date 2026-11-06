#!/usr/bin/env bash
# Mô phỏng quyền của hai policy bằng `aws iam simulate-custom-policy`.
# Chỉ ĐỌC: không tạo role, policy hay bucket nào. Cần quyền iam:SimulateCustomPolicy.
# Dùng: ./simulate.sh [profile]
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
profile="${1:-toinoi-lab}"
export AWS_PAGER=""
bucket="arn:aws:s3:::toinoi-lab-delivery-photos"
failures=0

# policy|action|resource|s3:prefix (rỗng nếu không cần)|kết quả mong đợi
cases=(
  "delivery-photo-policy.json|s3:PutObject|$bucket/proof/2026/trip-1001.jpg||allowed"
  "delivery-photo-policy.json|s3:GetObject|$bucket/proof/2026/trip-1001.jpg||allowed"
  "delivery-photo-policy.json|s3:PutObject|$bucket/other/x.jpg||implicitDeny"
  "delivery-photo-policy.json|s3:DeleteObject|$bucket/proof/2026/trip-1001.jpg||implicitDeny"
  "delivery-photo-policy.json|s3:PutObject|arn:aws:s3:::toinoi-lab-other-bucket/proof/x.jpg||implicitDeny"
  "delivery-photo-policy.json|s3:ListBucket|$bucket|proof/2026/|allowed"
  "delivery-photo-policy.json|s3:ListBucket|$bucket|other/|implicitDeny"
  "too-broad-policy.json|s3:DeleteObject|$bucket/proof/2026/trip-1001.jpg||allowed"
  "too-broad-policy.json|s3:PutObject|arn:aws:s3:::toinoi-lab-other-bucket/proof/x.jpg||allowed"
)

for entry in "${cases[@]}"; do
  IFS='|' read -r policy action resource prefix expect <<<"$entry"
  args=(--policy-input-list "$(cat "$here/$policy")"
    --action-names "$action"
    --resource-arns "$resource"
    --query 'EvaluationResults[0].EvalDecision'
    --output text
    --profile "$profile")
  if [ -n "$prefix" ]; then
    args+=(--context-entries "ContextKeyName=s3:prefix,ContextKeyValues=$prefix,ContextKeyType=string")
  fi
  got="$(aws iam simulate-custom-policy "${args[@]}")"
  if [ "$got" = "$expect" ]; then
    status="OK  "
  else
    status="SAI "
    failures=$((failures + 1))
  fi
  printf '%s %-26s %-18s %-60s -> %s (mong đợi %s)\n' "$status" "$policy" "$action" "${resource#arn:aws:s3:::}${prefix:+ [prefix=$prefix]}" "$got" "$expect"
done

if [ "$failures" -eq 0 ]; then
  echo "Mọi kết quả khớp mong đợi."
else
  echo "$failures kết quả không khớp mong đợi." >&2
  exit 1
fi
