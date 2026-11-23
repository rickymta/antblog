#!/bin/sh
# Đầu Phần 2: Bình bật xoay vòng log cho bao-gia và chép log hai ngày trước sang server.
# Chạy với quyền root; chạy lại nhiều lần vẫn cho cùng kết quả.
set -eu
nguon=/usr/share/toinoi-lab/p02
dich=/var/log/toinoi

install -o baogia -g toinoi -m 644 "$nguon/bao-gia-2026-11-20.log" "$dich/bao-gia.log.1"
install -o baogia -g toinoi -m 644 "$nguon/bao-gia-2026-11-19.log.gz" "$dich/bao-gia.log.2.gz"
touch -d '2026-11-20 11:16:34' "$dich/bao-gia.log.1"
touch -d '2026-11-19 11:07:06' "$dich/bao-gia.log.2.gz"
touch -d '2026-11-21 12:17:05' "$dich"
