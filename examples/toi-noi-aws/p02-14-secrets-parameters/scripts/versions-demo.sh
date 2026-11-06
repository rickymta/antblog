#!/usr/bin/env bash
# Mô phỏng thủ công việc rotation để thấy nhãn phiên bản AWSCURRENT và AWSPREVIOUS.
# Đây KHÔNG phải rotation thật (rotation thật cần hàm Lambda hoặc managed rotation, xem README).
set -euo pipefail

: "${ENV:=lab}"
SECRET_ID="toinoi/${ENV}/booking/db"
TMP="$(mktemp)"
chmod 600 "${TMP}"
trap 'rm -f "${TMP}"' EXIT

aws secretsmanager get-secret-value --secret-id "${SECRET_ID}" --query SecretString --output text \
  | jq --rawfile p <(aws secretsmanager get-random-password --password-length 32 --exclude-punctuation --query RandomPassword --output text | tr -d '\n') '.password = $p' > "${TMP}"

aws secretsmanager put-secret-value --secret-id "${SECRET_ID}" --secret-string "file://${TMP}" >/dev/null

echo "== Các phiên bản và nhãn =="
aws secretsmanager list-secret-version-ids --secret-id "${SECRET_ID}" \
  --query 'Versions[].[VersionId,VersionStages[0]]' --output table

echo "== Khoá trong AWSPREVIOUS (cửa sổ hai mật khẩu cùng tồn tại) =="
aws secretsmanager get-secret-value --secret-id "${SECRET_ID}" --version-stage AWSPREVIOUS \
  --query SecretString --output text | jq -r 'keys[]'
