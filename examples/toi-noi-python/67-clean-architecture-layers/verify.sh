#!/usr/bin/env sh
# Chuỗi kiểm chứng lab 67: ruff, mypy strict, import-linter, pytest, --verify.
# Thành công in "OK n/n"; thất bại in "FAIL n/m" (n là số bước đạt) rồi thoát mã khác 0.
set -eu
lab="$(cd "$(dirname "$0")" && pwd)"
total=7
n=0
trap 'rc=$?; [ "$rc" -eq 0 ] || echo "FAIL $((n - 1))/$total (dừng ở bước $n)"' EXIT
step() {
  n=$((n + 1))
  echo "== $n. $*"
}

step ruff format --check
uv run --project "$lab" ruff format --check "$lab"
step ruff check
uv run --project "$lab" ruff check "$lab"
step mypy --strict
uv run --project "$lab" mypy --config-file "$lab/pyproject.toml" "$lab/src" "$lab/tests"
step "mypy phải bắt adapter sai chữ ký (demo-errors)"
types="$(uv run --project "$lab" mypy --config-file "$lab/pyproject.toml" "$lab/demo-errors/bad_adapter.py" || true)"
echo "$types"
echo "$types" | grep -q 'expected "OcrPort | None"'
echo "$types" | grep -q "^Found 1 error"
step import-linter
(cd "$lab" && uv run --project "$lab" lint-imports)
step pytest
uv run --project "$lab" pytest "$lab" -q
step main.py --verify
uv run --project "$lab" "$lab/main.py" --verify
echo "OK $n/$total"
