#!/usr/bin/env bash
# Nối vào RDS Proxy bằng token IAM (iam_auth = REQUIRED). Chạy trên máy trong VPC (EC2 bài 05) có role với policy rds-db:connect.
# Dùng: ./connect-via-proxy.sh <proxy-endpoint> [toinoi_app]
# Cần psql. Không in token ra màn hình.
set -euo pipefail

host="${1:?proxy endpoint (output proxy_endpoint hoặc proxy_read_only_endpoint)}"
user="${2:-toinoi_app}"
region="${AWS_REGION:-ap-southeast-1}"

token="$(aws rds generate-db-auth-token --hostname "$host" --port 5432 --region "$region" --username "$user")"

# sslmode=require: proxy bắt buộc TLS (require_tls = true). Production nên dùng verify-full với bundle chứng chỉ.
PGPASSWORD="$token" psql "host=$host port=5432 dbname=tripsdb user=$user sslmode=require" \
  -c "select inet_server_addr() as server, pg_is_in_recovery() as is_reader, now();"
