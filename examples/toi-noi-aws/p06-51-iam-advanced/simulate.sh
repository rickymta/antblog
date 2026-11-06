#!/usr/bin/env bash
# Mô phỏng ABAC, permission boundary và điều kiện tạo role bằng `aws iam simulate-custom-policy`.
# Chỉ ĐỌC: không tạo role hay policy nào. Cần quyền iam:SimulateCustomPolicy.
# Dùng: ./simulate.sh [profile]
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
profile="${1:-toinoi-lab}"
export AWS_PAGER=""
account="123456789012"
fn="arn:aws:lambda:ap-southeast-1:${account}:function:toinoi-lab-dispatch-assign"
queue="arn:aws:sqs:ap-southeast-1:${account}:toinoi-lab-dispatch-offers"
fnrole="arn:aws:iam::${account}:role/toinoi-lab-fn-checker"
boundary_arn="arn:aws:iam::${account}:policy/toinoi-lab-lambda-dev-boundary"
failures=0

# Mỗi ca: tên|identity policy|boundary (hoặc -)|action|resource|context (name=type=value, cách nhau bằng ';')|mong đợi
cases=(
  "ABAC cùng team gọi được function|abac-team-lambda.json|-|lambda:InvokeFunction|$fn|aws:PrincipalTag/Team=string=dispatch;aws:ResourceTag/Team=string=dispatch|allowed"
  "ABAC khác team bị chặn|abac-team-lambda.json|-|lambda:InvokeFunction|$fn|aws:PrincipalTag/Team=string=booking;aws:ResourceTag/Team=string=dispatch|implicitDeny"
  "ABAC tạo function với tag đúng team|abac-team-lambda.json|-|lambda:CreateFunction|$fn|aws:PrincipalTag/Team=string=dispatch;aws:RequestTag/Team=string=dispatch|allowed"
  "ABAC tạo function mạo tag team khác|abac-team-lambda.json|-|lambda:CreateFunction|$fn|aws:PrincipalTag/Team=string=dispatch;aws:RequestTag/Team=string=booking|implicitDeny"
  "ABAC gỡ tag Team bị Deny|abac-team-lambda.json|-|lambda:UntagResource|$fn|aws:PrincipalTag/Team=string=dispatch;aws:TagKeys=stringList=Team|explicitDeny"
  "Boundary: identity Allow * nhưng gửi SQS toinoi-* vẫn được|example-broad-identity.json|lambda-dev-boundary.json|sqs:SendMessage|$queue||allowed"
  "Boundary: ngoài trần thì implicitDeny (ec2)|example-broad-identity.json|lambda-dev-boundary.json|ec2:RunInstances|*||implicitDeny"
  "Boundary: iam:* bị Deny rõ ràng|example-broad-identity.json|lambda-dev-boundary.json|iam:CreateUser|arn:aws:iam::${account}:user/x||explicitDeny"
  "Tạo role có gắn đúng boundary|developer-create-role.json|-|iam:CreateRole|$fnrole|iam:PermissionsBoundary=string=$boundary_arn|allowed"
  "Tạo role không gắn boundary|developer-create-role.json|-|iam:CreateRole|$fnrole||implicitDeny"
  "Tạo role gắn boundary khác|developer-create-role.json|-|iam:CreateRole|$fnrole|iam:PermissionsBoundary=string=arn:aws:iam::aws:policy/AdministratorAccess|implicitDeny"
  "Gỡ boundary khỏi role bị Deny|developer-create-role.json|-|iam:DeleteRolePermissionsBoundary|$fnrole||explicitDeny"
  "Sửa chính policy boundary bị Deny|developer-create-role.json|-|iam:CreatePolicyVersion|$boundary_arn||explicitDeny"
)

for entry in "${cases[@]}"; do
  IFS='|' read -r name policy boundary action resource contexts expect <<<"$entry"
  args=(--policy-input-list "$(cat "$here/policies/$policy")"
    --action-names "$action"
    --resource-arns "$resource"
    --query 'EvaluationResults[0].EvalDecision'
    --output text
    --profile "$profile")
  if [ "$boundary" != "-" ]; then
    args+=(--permissions-boundary-policy-input-list "$(cat "$here/policies/$boundary")")
  fi
  if [ -n "$contexts" ]; then
    IFS=';' read -r -a pairs <<<"$contexts"
    entries=()
    for pair in "${pairs[@]}"; do
      IFS='=' read -r key type value <<<"$pair"
      entries+=("ContextKeyName=$key,ContextKeyValues=$value,ContextKeyType=$type")
    done
    args+=(--context-entries "${entries[@]}")
  fi
  got="$(aws iam simulate-custom-policy "${args[@]}")"
  if [ "$got" = "$expect" ]; then
    status="OK  "
  else
    status="SAI "
    failures=$((failures + 1))
  fi
  printf '%s %-62s -> %s (mong đợi %s)\n' "$status" "$name" "$got" "$expect"
done

if [ "$failures" -eq 0 ]; then
  echo "Mọi kết quả khớp mong đợi."
else
  echo "$failures kết quả không khớp mong đợi." >&2
  exit 1
fi
