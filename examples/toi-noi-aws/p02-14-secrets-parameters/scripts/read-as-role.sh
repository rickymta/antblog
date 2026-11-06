#!/usr/bin/env bash
# Assume role chỉ đọc rồi đọc cấu hình: chứng minh quyền theo ARN, tag và KMS hoạt động.
# Không in giá trị secret, chỉ in tên khoá JSON.
set -euo pipefail

: "${ENV:=lab}"
: "${AWS_REGION:=ap-southeast-1}"
ROLE_ARN="${1:?Cách dùng: read-as-role.sh <reader_role_arn>}"

CREDS="$(aws sts assume-role --role-arn "${ROLE_ARN}" --role-session-name toinoi-config-reader \
  --query 'Credentials.[AccessKeyId,SecretAccessKey,SessionToken]' --output text)"
read -r AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_SESSION_TOKEN <<<"${CREDS}"
export AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_SESSION_TOKEN
unset AWS_PROFILE

echo "== Danh tính hiện tại =="
aws sts get-caller-identity --query Arn --output text

echo "== Parameter dưới /toinoi/${ENV}/booking (SecureString đã giải mã, che giá trị) =="
aws ssm get-parameters-by-path --path "/toinoi/${ENV}/booking" --recursive --with-decryption \
  --query 'Parameters[].[Name,Type]' --output table

echo "== Khoá trong secret DB =="
aws secretsmanager get-secret-value --secret-id "toinoi/${ENV}/booking/db" \
  --query SecretString --output text | jq -r 'keys[]'

echo "== Thử đọc secret không thuộc service này (phải báo lỗi AccessDenied) =="
aws secretsmanager get-secret-value --secret-id "toinoi/${ENV}/dispatch/db" 2>&1 | head -n 2 || true
