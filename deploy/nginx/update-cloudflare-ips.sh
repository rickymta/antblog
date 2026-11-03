#!/bin/sh
# Tải lại danh sách dải IP của Cloudflare vào cloudflare-realip.conf. Chạy trên server rồi
# `docker compose build nginx && docker compose up -d nginx`.
set -eu
cd "$(dirname "$0")"
v4=$(curl -fsS https://www.cloudflare.com/ips-v4)
v6=$(curl -fsS https://www.cloudflare.com/ips-v6)
{
  echo '# Lấy IP thật của người dùng khi đi qua Cloudflare (mức http, nạp từ conf.d).'
  echo "# Sinh tự động bởi update-cloudflare-ips.sh lúc $(date -u +%Y-%m-%dT%H:%MZ)."
  echo 'real_ip_header CF-Connecting-IP;'
  echo 'real_ip_recursive on;'
  echo
  echo '# IPv4'
  for ip in $v4; do echo "set_real_ip_from $ip;"; done
  echo
  echo '# IPv6'
  for ip in $v6; do echo "set_real_ip_from $ip;"; done
} > cloudflare-realip.conf.tmp
mv cloudflare-realip.conf.tmp cloudflare-realip.conf
echo "Đã cập nhật $(grep -c set_real_ip_from cloudflare-realip.conf) dải IP."
