#!/usr/bin/env sh
# Chạy toàn bộ lab phần 03 (bài 21–30). Cần uv, Python 3.14, Docker và hạ tầng chung:
#   docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
set -eu
root="$(cd "$(dirname "$0")" && pwd)"
for lab in 21-asyncio-task-timeout 22-gil-threading-multiprocessing 23-lock-queue-semaphore \
           24-sqlalchemy-postgresql-alembic 25-transaction-optimistic-idempotency \
           26-mongodb-pymongo-document-ocr 27-redis-cache-rate-limit-geo; do
  echo "== $lab"
  uv run --project "$root/$lab" "$root/$lab/main.py" --verify
done
echo "== 28-integration-test-testcontainers"
uv run --project "$root/28-integration-test-testcontainers" pytest "$root/28-integration-test-testcontainers" -q
for lab in 29-polars-pandas-parquet 30-numpy-pillow-opencv; do
  echo "== $lab"
  uv run --project "$root/$lab" "$root/$lab/main.py" --verify
done
