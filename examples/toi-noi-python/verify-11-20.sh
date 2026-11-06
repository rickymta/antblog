#!/usr/bin/env sh
# Chạy toàn bộ lab phần 02 (bài 11–20). Cần uv và Python 3.14 (uv python install 3.14); lần đầu cần mạng để uv tải mypy/pytest/pydantic.
set -eu
root="$(cd "$(dirname "$0")" && pwd)"
for lab in 11-class-dataclass-property 12-type-hints-mypy-protocol 13-frozen-dataclass-enum 14-abc-protocol-di \
           15-pydantic-validation-settings 16-match-pattern-none 17-decorator-context-manager \
           18-functools-itertools-pipeline 19-design-pattern-strategy-state-adapter; do
  echo "== $lab"
  uv run --project "$root/$lab" "$root/$lab/main.py" --verify
done
echo "== 20-dong-goi-ruff-cau-truc-du-an"
sh "$root/20-dong-goi-ruff-cau-truc-du-an/verify.sh"
