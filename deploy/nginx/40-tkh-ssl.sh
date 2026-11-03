#!/bin/sh
# Chạy bởi entrypoint của ảnh nginx chính thức (sau envsubst template), TRƯỚC khi nginx khởi động.
# 1. Chọn chứng chỉ lần đầu.
# 2. Chạy nền vòng lặp: mỗi 5 phút kiểm tra certbot đã có chứng chỉ thật chưa (có ⇒ reload ngay);
#    mỗi 6 giờ reload để nạp chứng chỉ vừa được gia hạn.
set -eu

/usr/local/bin/tkh-select-cert.sh >/dev/null
echo "tkh: dùng $(head -n1 /etc/nginx/tkh-ssl.conf)"

(
  elapsed=0
  while :; do
    sleep 300
    elapsed=$((elapsed + 300))
    if [ "$(/usr/local/bin/tkh-select-cert.sh 2>/dev/null || true)" = "changed" ] || [ "$elapsed" -ge 21600 ]; then
      elapsed=0
      { nginx -t -q && nginx -s reload && echo "tkh: đã nạp lại nginx ($(head -n1 /etc/nginx/tkh-ssl.conf))"; } \
        || echo "tkh: CẢNH BÁO — nạp lại nginx thất bại, vẫn chạy cấu hình cũ" >&2
    fi
  done
) &
