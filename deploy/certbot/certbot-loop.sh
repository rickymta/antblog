#!/bin/sh
# Xin và tự gia hạn MỘT chứng chỉ Let's Encrypt cho $DOMAIN + $ADMIN_DOMAIN (+ CERT_EXTRA_DOMAINS)
# bằng thử thách DNS-01 qua API Cloudflare.
# Chạy được cả khi bản ghi DNS đang bật proxy Cloudflare (đám mây cam) — không cần mở cổng cho Let's Encrypt.
# Token Cloudflare chỉ cần quyền Zone › DNS › Edit trên đúng zone của tên miền.
set -eu

: "${DOMAIN:?Thiếu biến DOMAIN}"

# Chưa có token/email (vd domain chưa chuyển sang Cloudflare): KHÔNG thoát — thoát thì restart policy chạy lại
# liên tục. Nằm chờ tới khi điền .env rồi `docker compose up -d certbot` (tạo lại container với biến mới).
if [ -z "${CF_API_TOKEN:-}" ] || [ -z "${ACME_EMAIL:-}" ]; then
  echo "certbot: chưa có CF_API_TOKEN hoặc ACME_EMAIL trong .env — tạm dừng. nginx vẫn chạy với chứng chỉ tự ký." >&2
  echo "certbot: điền đủ rồi chạy: docker compose up -d certbot" >&2
  while :; do sleep 86400; done
fi

umask 077
CREDS=/tmp/cloudflare.ini
printf 'dns_cloudflare_api_token = %s\n' "$CF_API_TOKEN" > "$CREDS"

STAGING_FLAG=""
if [ "${ACME_STAGING:-0}" = "1" ]; then
  STAGING_FLAG="--staging"
  echo "certbot: đang dùng môi trường STAGING của Let's Encrypt (chứng chỉ không được trình duyệt tin)."
fi

DOMAIN_ARGS="-d $DOMAIN"
[ -n "${ADMIN_DOMAIN:-}" ] && DOMAIN_ARGS="$DOMAIN_ARGS -d $ADMIN_DOMAIN"
for d in $(echo "${CERT_EXTRA_DOMAINS:-}" | tr ',' ' '); do
  DOMAIN_ARGS="$DOMAIN_ARGS -d $d"
done

issue() {
  # shellcheck disable=SC2086
  certbot certonly --non-interactive --agree-tos --email "$ACME_EMAIL" \
    --dns-cloudflare --dns-cloudflare-credentials "$CREDS" --dns-cloudflare-propagation-seconds 30 \
    --cert-name "$DOMAIN" --keep-until-expiring --expand $STAGING_FLAG $DOMAIN_ARGS
}

# Mỗi lần container khởi động: xin mới, hoặc MỞ RỘNG chứng chỉ khi danh sách tên miền đổi
# (đã đủ tên và chưa tới hạn ⇒ certbot không làm gì). Lỗi (token sai, DNS chưa trỏ…) ⇒ chờ 1 giờ rồi thử lại,
# KHÔNG thoát — container thoát rồi khởi động lại liên tục sẽ dính giới hạn tần suất của Let's Encrypt.
until issue; do
  echo "certbot: xin chứng chỉ thất bại, thử lại sau 1 giờ (xem lỗi ở trên)." >&2
  sleep 3600
done
echo "certbot: chứng chỉ $DOMAIN sẵn sàng ($DOMAIN_ARGS). nginx tự nạp trong ≤ 5 phút."

# Gia hạn: certbot chỉ gia hạn khi còn < 30 ngày. nginx nạp lại chứng chỉ mới mỗi 6 giờ.
while :; do
  certbot renew --non-interactive $STAGING_FLAG || echo "certbot: gia hạn lỗi, thử lại sau 12 giờ." >&2
  sleep 43200
done
