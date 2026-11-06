#!/usr/bin/env bash
set -euo pipefail
# Cấp cho role của EC2 quyền đọc ĐÚNG secret mật khẩu của DB này (quyền tối thiểu), bằng inline policy.
source "$(dirname "$0")/common.sh"
if [[ -z "${EC2_ROLE_NAME:-}" ]]; then echo "EC2_ROLE_NAME trống, bỏ qua."; exit 0; fi
confirm_identity

SECRET_ARN="$(aws rds describe-db-instances --db-instance-identifier "$DB_ID" \
  --query 'DBInstances[0].MasterUserSecret.SecretArn' --output text)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
sed "s|__SECRET_ARN__|$SECRET_ARN|g" "$HERE/policy-read-db-secret.json.tpl" > "$TMP/policy.json"
aws iam put-role-policy --role-name "$EC2_ROLE_NAME" \
  --policy-name "$ROLE_POLICY_NAME" --policy-document "file://$TMP/policy.json"
echo "Đã gắn inline policy $ROLE_POLICY_NAME vào role $EC2_ROLE_NAME."
echo "Secret ARN: $SECRET_ARN"
