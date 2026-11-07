#!/usr/bin/env sh
# Chạy toàn bộ lab phần 07 (bài 54–61). Cần uv, Python 3.14, libomp trên macOS (brew install libomp)
# và hạ tầng chung: docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
# Lab 58 chạy nhiều phút vì lặp 3 seed.
set -eu
root="$(cd "$(dirname "$0")" && pwd)"
for lab in 54-fraud-bai-toan-nhan-du-lieu 55-feature-engineering-event 56-rule-engine-scoring \
           57-sklearn-lightgbm-imbalanced 58-networkx-graph-cau-ket 59-real-time-scoring-redis \
           60-model-registry-ab-drift 61-shap-audit-human-review; do
  echo "== $lab"
  uv run --project "$root/$lab" "$root/$lab/main.py" --verify
done
for lab in 56-rule-engine-scoring 57-sklearn-lightgbm-imbalanced 60-model-registry-ab-drift 61-shap-audit-human-review; do
  echo "== pytest $lab"
  uv run --project "$root/$lab" pytest "$root/$lab" -q
done
