#!/usr/bin/env sh
# Chạy toàn bộ lab phần 01 (bài 01–10). Cần uv và Python 3.14 (uv python install 3.14).
set -eu
root="$(cd "$(dirname "$0")" && pwd)"
for lab in 01-python-va-uv 02-bien-kieu-du-lieu-decimal 03-dieu-kien-vong-lap-ham 04-chuoi-ngay-gio-timezone \
           05-list-dict-set-tuple 06-comprehension-generator 07-module-package-import 08-exception-debug-logging \
           09-file-json-csv-pathlib; do
  echo "== $lab"
  uv run --project "$root/$lab" "$root/$lab/main.py" --verify
done
echo "== 10-pytest-chinh-sach-cuoc"
uv run --project "$root/10-pytest-chinh-sach-cuoc" pytest "$root/10-pytest-chinh-sach-cuoc" -q
