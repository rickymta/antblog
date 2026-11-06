#!/usr/bin/env bash
# Tạo bảng bằng CLI, tương đương Terraform. Tiện cho DynamoDB Local; trên AWS thật nên dùng Terraform.
source "$(dirname "$0")/common.sh"

ddb create-table --cli-input-json "file://$(dirname "$0")/../table-definition.json" >/dev/null
ddb wait table-exists --table-name "$TABLE"
ddb update-time-to-live --table-name "$TABLE" \
  --time-to-live-specification "Enabled=true,AttributeName=expires_at" >/dev/null
echo "Đã tạo $TABLE (GSI gsi1, TTL expires_at, Streams bật)."
