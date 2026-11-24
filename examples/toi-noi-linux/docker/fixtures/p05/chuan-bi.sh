#!/bin/sh
# Đầu Phần 5: dữ liệu cho các bài mạng, đĩa và giám sát. Chạy với quyền root trên toinoi-app-01;
# chạy lại vẫn cho cùng kết quả.
set -eu
nguon=/usr/share/toinoi-lab/p05

# LNX-043: một file log gỡ lỗi ai đó bật rồi quên tắt, 400 MB.
log=/srv/toinoi/bao-gia/shared/logs/debug-2026-11.log
[ -e "$log" ] || fallocate -l 400M "$log"
chown baogia:toinoi "$log"
chmod 640 "$log"
touch -d '2026-11-23 23:59:00' "$log"

# LNX-047: "ổ đĩa mới" mà nhà cung cấp gắn thêm (một file ảnh 512 MB, gắn thành loop device).
install -d -m 700 /srv/dia
[ -e /srv/dia/du-lieu-01.img ] || truncate -s 512M /srv/dia/du-lieu-01.img
