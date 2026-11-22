#!/usr/bin/env bash
# Kiểm tra nhanh cấu hình của dịch vụ bao-gia trước khi khởi động.
set -eu
thu_muc="$(cd "$(dirname "$0")" && pwd)"
bang_gia="$thu_muc/bang-gia.csv"

if [ ! -r "$bang_gia" ]; then
  echo "LOI: khong doc duoc $bang_gia" >&2
  exit 1
fi

so_dich_vu=$(($(wc -l < "$bang_gia") - 1))
echo "bao-gia: bang gia co $so_dich_vu loai dich vu"
echo "bao-gia: cau hinh hop le"
