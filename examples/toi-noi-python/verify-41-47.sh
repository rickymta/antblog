#!/usr/bin/env sh
# Chạy toàn bộ lab phần 05 (bài 41–47). Cần uv, Python 3.14, Tesseract có gói vie và hạ tầng chung:
#   docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
set -eu
root="$(cd "$(dirname "$0")" && pwd)"
for lab in 41-kyc-bai-toan-du-lieu-metric 42-opencv-canh-perspective-deskew 43-tesseract-easyocr-paddleocr \
           44-hau-xu-ly-ocr-regex-checksum 45-danh-gia-ocr-cer-wer 46-ocr-service-fastapi-worker \
           47-luu-ket-qua-ocr-mongodb-pii; do
  echo "== $lab"
  uv run --project "$root/$lab" "$root/$lab/main.py" --verify
done
