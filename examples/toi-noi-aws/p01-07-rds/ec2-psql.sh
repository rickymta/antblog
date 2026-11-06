#!/usr/bin/env bash
# Chạy TRÊN instance EC2 (qua Session Manager), không chạy trên máy bạn.
# Dùng: ./ec2-psql.sh <endpoint> <secret-arn> [file.sql]
# Cần: AWS CLI v2 và python3 (có sẵn trên AL2023) và psql: sudo dnf install -y postgresql17
# (nếu tên gói khác, tìm bằng: dnf search postgresql). Máy cần đường ra qua NAT để tải gói.
set -euo pipefail
ENDPOINT="${1:?thiếu endpoint}"; SECRET_ARN="${2:?thiếu secret arn}"; SQL_FILE="${3:-}"
REGION="${AWS_REGION:-ap-southeast-1}"

command -v psql >/dev/null || { echo "Chưa có psql. Cài client PostgreSQL bằng dnf." >&2; exit 1; }

SECRET_JSON="$(aws secretsmanager get-secret-value --region "$REGION" \
  --secret-id "$SECRET_ARN" --query SecretString --output text)"
PGPASSWORD="$(printf '%s' "$SECRET_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin)["password"])')"
DB_USER="$(printf '%s' "$SECRET_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin)["username"])')"
export PGPASSWORD

CONN="host=$ENDPOINT port=5432 dbname=tripsdb user=$DB_USER sslmode=require"
if [[ -n "$SQL_FILE" ]]; then
  psql "$CONN" -v ON_ERROR_STOP=1 -f "$SQL_FILE"
else
  psql "$CONN"
fi
