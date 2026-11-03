#!/bin/sh
# Tạo .env từ .env.example và sinh ngẫu nhiên mọi secret còn trống. Không bao giờ ghi đè .env đã có.
# Chạy trên server, tại thư mục gốc repo tech-blog:  sh deploy/init-env.sh
set -eu
cd "$(dirname "$0")/.."

if [ -f .env ]; then
  echo ".env đã tồn tại — không ghi đè. Xoá hoặc đổi tên nếu muốn tạo lại." >&2
  exit 1
fi

rand() { openssl rand -base64 "$1" | tr -d '\n/+=' | cut -c1-"$2"; }

umask 077
sed \
  -e "s|^JWT_SECRET=$|JWT_SECRET=$(rand 64 64)|" \
  -e "s|^MONGO_ROOT_PASSWORD=$|MONGO_ROOT_PASSWORD=$(rand 32 32)|" \
  -e "s|^MONGO_APP_PASSWORD=$|MONGO_APP_PASSWORD=$(rand 32 32)|" \
  -e "s|^MONGO_INSPECTOR_PASSWORD=$|MONGO_INSPECTOR_PASSWORD=$(rand 32 24)|" \
  -e "s|^SEED_ADMIN_PASSWORD=$|SEED_ADMIN_PASSWORD=$(rand 24 20)|" \
  .env.example > .env

echo "Đã tạo .env (quyền 600). Còn phải điền tay: DOMAIN, ACME_EMAIL, CF_API_TOKEN, WG_SERVER_URL, SEED_ADMIN_EMAIL."
