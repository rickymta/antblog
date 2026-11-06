#!/usr/bin/env sh
# Chạy toàn bộ lab phần 04 (bài 31–40). Cần uv, Python 3.14, Docker và hạ tầng chung:
#   docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
set -eu
root="$(cd "$(dirname "$0")" && pwd)"
for lab in 31-fastapi-endpoint-bao-gia 32-fastapi-validation-problem-details 33-fastapi-dependency-middleware-logging \
           34-fastapi-jwt-authorization 35-fastapi-upload-background-streaming; do
  echo "== $lab"
  uv run --project "$root/$lab" "$root/$lab/main.py" --verify
done
echo "== 36-fastapi-api-test-httpx"
uv run --project "$root/36-fastapi-api-test-httpx" pytest "$root/36-fastapi-api-test-httpx" -q
for lab in 37-grpc-fraud-scoring 38-kafka-aiokafka-producer-consumer 39-outbox-inbox-idempotent-consumer \
           40-worker-arq-retry-dead-letter; do
  echo "== $lab"
  uv run --project "$root/$lab" "$root/$lab/main.py" --verify
done
