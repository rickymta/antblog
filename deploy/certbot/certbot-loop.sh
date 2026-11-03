#!/bin/sh
# Xin và tự gia hạn chứng chỉ Let's Encrypt cho $DOMAIN bằng thử thách DNS-01 qua API Cloudflare.
# Chạy được cả khi bản ghi DNS đang bật proxy Cloudflare (đám mây cam) — không cần mở cổng cho Let's Encrypt.
# Token Cloudflare chỉ cần quyền Zone › DNS › Edit trên đúng zone của tên miền.
set -eu

: "${DOMAIN:?Thiếu biến DOMAIN}"
: "${ACME_EMAIL:?Thiếu biến ACME_EMAIL}"
: "${CF_API_TOKEN:?Thiếu biến CF_API_TOKEN}"

umask 077
CREDS=/tmp/cloudflare.ini
printf 'dns_cloudflare_api_token = %s\n' "$CF_API_TOKEN" > "$CREDS"

STAGING_FLAG=""
if [ "${ACME_STAGING:-0}" = "1" ]; then
  STAGING_FLAG="--staging"
  echo "certbot: đang dùng môi trường STAGING của Let's Encrypt (chứng chỉ không được trình duyệt tin)."
fi

DOMAIN_ARGS="-d $DOMAIN"
for d in $(echo "${CERT_EXTRA_DOMAINS:-}" | tr ',' ' '); do
  DOMAIN_ARGS="$DOMAIN_ARGS -d $d"
done

issue() {
  # shellcheck disable=SC2086
  certbot certonly --non-interactive --agree-tos --email "$ACME_EMAIL" \
    --dns-cloudflare --dns-cloudflare-credentials "$CREDS" --dns-cloudflare-propagation-seconds 30 \
    --cert-name "$DOMAIN" --keep-until-expiring $STAGING_FLAG $DOMAIN_ARGS
}

# Lần đầu: thử tới khi được. Lỗi (token sai, DNS chưa trỏ…) ⇒ chờ 1 giờ rồi thử lại,
# KHÔNG thoát — container thoát rồi khởi động lại liên tục sẽ dính giới hạn tần suất của Let's Encrypt.
until [ -s "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" ]; do
  if issue; then
    echo "certbot: đã có chứng chỉ cho $DOMAIN. nginx sẽ tự nạp trong ≤ 5 phút."
  else
    echo "certbot: xin chứng chỉ thất bại, thử lại sau 1 giờ (xem lỗi ở trên)." >&2
    sleep 3600
  fi
done

# Gia hạn: certbot chỉ gia hạn khi còn < 30 ngày. nginx nạp lại chứng chỉ mới mỗi 6 giờ.
while :; do
  certbot renew --non-interactive $STAGING_FLAG || echo "certbot: gia hạn lỗi, thử lại sau 12 giờ." >&2
  sleep 43200
done
