#!/usr/bin/env sh
# Chạy cả chuỗi kiểm chứng của bài 20. Gọi từ thư mục gốc repository hoặc bất kỳ đâu.
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
step pytest
uv run --project "$lab" pytest "$lab" -q
step toinoi-fare 2.5
out="$(uv run --project "$lab" toinoi-fare 2.5)"
echo "$out"
[ "$out" = "2.5 km => 14250 VND" ]

# uv build: wheel phải có py.typed, entry point đúng, và chạy được trong môi trường cô lập.
step uv build + cài wheel vào môi trường cô lập
rm -rf "$lab/out"
uv build --project "$lab" --out-dir "$lab/out" >/dev/null 2>&1
wheel="$(ls "$lab"/out/*.whl)"
unzip -l "$wheel" | grep -q "toinoi_pricing/py.typed"
unzip -p "$wheel" '*/entry_points.txt' | grep -q "toinoi-fare = toinoi_pricing.cli:main"
echo "wheel: $(basename "$wheel")"
isolated="$(uv run --no-project --isolated --with "$wheel" toinoi-fare 2.5)"
echo "$isolated"
[ "$isolated" = "2.5 km => 14250 VND" ]

# Các công cụ phải BẮT được file cố ý sai trong demo-errors/ (đếm đúng số lỗi).
step demo-errors: công cụ phải bắt được lỗi
lint="$(uv run --project "$lab" ruff check "$lab/demo-errors/bad_lint.py" --output-format concise || true)"
echo "$lint" | grep "^Found"
echo "$lint" | grep -q "^Found 5 errors"
fmt="$(uv run --project "$lab" ruff format --check "$lab/demo-errors/bad_format.py" 2>&1 || true)"
echo "$fmt" | grep "^1 file would be reformatted"
types="$(uv run --project "$lab" mypy --config-file "$lab/pyproject.toml" "$lab/demo-errors/bad_types.py" || true)"
echo "$types" | grep "^Found"
echo "$types" | grep -q "^Found 3 errors"
echo "OK $n/$total"
