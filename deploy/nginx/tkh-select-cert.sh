#!/bin/sh
# Ghi /etc/nginx/tkh-ssl.conf trỏ tới chứng chỉ đang dùng:
#   - Let's Encrypt (/etc/letsencrypt/live/$DOMAIN/) nếu certbot đã xin được;
#   - ngược lại chứng chỉ tự ký tạm (tạo một lần) để nginx vẫn khởi động được.
# In "changed" khi đổi nguồn hoặc nội dung chứng chỉ (vòng lặp reload dựa vào đó để nạp lại ngay).
set -eu

: "${DOMAIN:?Thiếu biến DOMAIN}"
LE_DIR="/etc/letsencrypt/live/${DOMAIN}"
SELF_DIR=/etc/nginx/selfsigned
OUT=/etc/nginx/tkh-ssl.conf

if [ -s "$LE_DIR/fullchain.pem" ] && [ -s "$LE_DIR/privkey.pem" ]; then
  cert="$LE_DIR/fullchain.pem"
  key="$LE_DIR/privkey.pem"
else
  if [ ! -s "$SELF_DIR/fullchain.pem" ]; then
    mkdir -p "$SELF_DIR"
    openssl req -x509 -nodes -newkey rsa:2048 -days 30 \
      -subj "/CN=${DOMAIN}" -addext "subjectAltName=DNS:${DOMAIN}${ADMIN_DOMAIN:+,DNS:${ADMIN_DOMAIN}}" \
      -keyout "$SELF_DIR/privkey.pem" -out "$SELF_DIR/fullchain.pem" >/dev/null 2>&1
    chmod 600 "$SELF_DIR/privkey.pem"
  fi
  cert="$SELF_DIR/fullchain.pem"
  key="$SELF_DIR/privkey.pem"
fi

# Dòng cuối ghi dấu vân tay chứng chỉ: cùng đường dẫn mà nội dung đổi (staging → thật, vừa gia hạn)
# thì tệp cũng đổi ⇒ vòng lặp nạp lại nginx ngay thay vì chờ mốc 6 giờ.
fp=$(openssl x509 -in "$cert" -noout -fingerprint -sha256 2>/dev/null | cut -d= -f2 || true)
printf 'ssl_certificate %s;\nssl_certificate_key %s;\n# sha256 %s\n' "$cert" "$key" "${fp:-?}" > "$OUT.tmp"
if [ -f "$OUT" ] && cmp -s "$OUT.tmp" "$OUT"; then
  rm -f "$OUT.tmp"
else
  mv "$OUT.tmp" "$OUT"
  echo "changed"
fi
