#!/usr/bin/env sh
# Chạy toàn bộ lab phần 06 (bài 48–53). Cần uv, Python 3.14, hạ tầng chung cho bài 51
# (docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait) và mạng lần đầu để tải dataset, mô hình.
set -eu
root="$(cd "$(dirname "$0")" && pwd)"
for lab in 48-face-detection-landmarks 49-face-embedding-cosine-threshold 50-liveness-anti-spoofing \
           51-vector-search-pgvector-faiss 52-face-match-grpc-service 53-face-matching-danh-gia-bias-privacy; do
  echo "== $lab"
  uv run --project "$root/$lab" "$root/$lab/main.py" --verify
done
