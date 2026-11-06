#!/usr/bin/env sh
# Chạy toàn bộ lab phần 09 (bài 67–71). Cần uv, Python 3.14, hạ tầng chung cho bài 69–71
# (docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait) và .NET SDK 10 cho ca C# của bài 71.
set -eu
root="$(cd "$(dirname "$0")" && pwd)"
for lab in 67-clean-architecture-layers 68-cqrs-mediator-pipeline 69-repository-unit-of-work \
           70-use-case-ho-so-tai-xe-end-to-end 71-domain-event-outbox-version; do
  echo "== $lab"
  sh "$root/$lab/verify.sh"
done
