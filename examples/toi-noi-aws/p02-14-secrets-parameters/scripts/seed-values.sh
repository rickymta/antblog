#!/usr/bin/env bash
# Đặt giá trị thật cho SecureString và secret DB sau khi Terraform đã tạo "vỏ".
# Mật khẩu sinh ngẫu nhiên bằng Secrets Manager, ghi qua file tạm (quyền 600) để không lộ trong lịch sử shell
# hay danh sách tiến trình. Chạy trong tài khoản lab. Script KHÔNG in mật khẩu.
set -euo pipefail

: "${ENV:=lab}"
: "${AWS_REGION:=ap-southeast-1}"
PREFIX="/toinoi/${ENV}/booking"
SECRET_ID="toinoi/${ENV}/booking/db"
KEY_ALIAS="alias/toinoi-${ENV}-config"

TMP="$(mktemp)"
chmod 600 "${TMP}"
trap 'rm -f "${TMP}"' EXIT

# SecureString: khoá API giả lập của cổng thanh toán VíXanh (giá trị sinh ngẫu nhiên, không phải khoá thật).
aws secretsmanager get-random-password --password-length 40 --exclude-punctuation \
  --query RandomPassword --output text | tr -d "\n" > "${TMP}"
aws ssm put-parameter --name "${PREFIX}/vixanh-api-key" --type SecureString \
  --key-id "${KEY_ALIAS}" --value "file://${TMP}" --overwrite

# Secret DB dạng JSON chuẩn mà các hàm rotation mẫu của Secrets Manager mong đợi.
# --rawfile đọc mật khẩu từ process substitution: không đưa mật khẩu vào tham số dòng lệnh.
jq -n --rawfile p <(aws secretsmanager get-random-password --password-length 32 --exclude-punctuation \
  --query RandomPassword --output text | tr -d '\n') '{
  engine: "postgres",
  username: "booking_app",
  password: $p,
  host: "toinoi-lab-db.example.internal",
  port: 5432,
  dbname: "toinoi"
}' > "${TMP}"

aws secretsmanager put-secret-value --secret-id "${SECRET_ID}" --secret-string "file://${TMP}" \
  --query '{Version:VersionId,Stages:VersionStages}'

echo "Đã đặt giá trị. Host trong secret là giá trị giả: thay bằng endpoint RDS thật của bạn nếu cần kết nối."
