#!/bin/sh
# Đầu Phần 4: Bình đưa bản 1.5.0 của bao-gia lên server (chưa chuyển current, chưa chạy) và cài
# vài script của nhóm vào /usr/local/bin. Chạy với quyền root; chạy lại vẫn cho cùng kết quả.
set -eu
nguon=/usr/share/toinoi-lab/p04
rel=/srv/toinoi/bao-gia/releases/1.5.0

install -d -o an -g toinoi -m 755 "$rel"
install -o an -g toinoi -m 750 "$nguon/release-1.5.0/bao-gia" "$rel/bao-gia"
install -o an -g toinoi -m 640 "$nguon/release-1.5.0/bang-gia.csv" "$rel/bang-gia.csv"
install -o an -g toinoi -m 750 "$nguon/release-1.5.0/kiem-tra-cau-hinh.sh" "$rel/kiem-tra-cau-hinh.sh"
touch -d '2026-11-23 08:40:00' "$rel/bao-gia" "$rel/bang-gia.csv" "$rel/kiem-tra-cau-hinh.sh" "$rel"

for f in tinh-lai-bao-cao xuat-du-lieu xuat-chuyen; do
  install -o root -g root -m 755 "$nguon/$f" "/usr/local/bin/$f"
done
