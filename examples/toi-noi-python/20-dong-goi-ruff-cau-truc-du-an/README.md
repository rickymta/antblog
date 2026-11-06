# Lab 20: đóng gói, ruff, mypy và cấu trúc dự án

Bài viết: `api/seed/content/python-dong-goi-ruff-pre-commit-cau-truc-du-an.md` (bài tổng hợp P01 và P02).

Gói `toinoi_pricing` (src layout, build backend `uv_build`):

- `src/toinoi_pricing/models.py`: `ServiceKind`, `TripStatus`, `ALLOWED_TRANSITIONS`, `Money` và `Quote` (frozen, kiểm tra đầu vào chép từ lab 13).
- `src/toinoi_pricing/pricing.py`: `FarePolicy`, `calculate_fare`, `make_quote`.
- `src/toinoi_pricing/cli.py`: lệnh `toinoi-fare` (entry point `[project.scripts]`); `__main__.py` cho `python -m`.
- `src/toinoi_pricing/py.typed`: đánh dấu gói có kiểu (PEP 561).
- `tests/`: pytest cho pricing, models, cli (60 test).
- `pyproject.toml`: metadata, entry point, `[dependency-groups] dev`, cấu hình `[tool.ruff]`, `[tool.mypy]`, `[tool.pytest.ini_options]`.
- `demo-errors/`: file cố ý sai (`bad_lint.py`, `bad_format.py`, `bad_types.py`) và `consumer.py` (mã của người dùng gói), bị loại khỏi lệnh kiểm chung; `verify.sh` đếm đúng số lỗi mà ruff và mypy bắt được.
- `.pre-commit-config.yaml`: cấu hình mẫu (ruff-format, ruff-check, mypy, vài hook chung). Lab không cài hook.

Chạy từ thư mục gốc repository:

```bash
L=examples/toi-noi-python/20-dong-goi-ruff-cau-truc-du-an
uv run --project $L ruff format --check $L
uv run --project $L ruff check $L
uv run --project $L mypy --config-file $L/pyproject.toml $L/src $L/tests
uv run --project $L pytest $L -q
uv run --project $L toinoi-fare 2.5
sh $L/verify.sh
```

Kết quả kỳ vọng: `9 files already formatted` (8 file `.py` và `README.md`), `All checks passed!`, mypy `Success: no issues found in 8 source files`, pytest `60 passed`, `toinoi-fare 2.5` in `2.5 km => 14250 VND`; `verify.sh` kết thúc bằng `OK 7/7`. Thất bại thì `verify.sh` in `FAIL n/7` (n là số bước đạt) và thoát mã khác 0.

## Đóng gói

```bash
uv build --project $L --out-dir $L/out
uv run --no-project --isolated --with $L/out/*.whl toinoi-fare 2.5
```

`out/` bị gitignore; lần đầu cần mạng để uv tải build backend. `verify.sh` bước 6 chạy đúng hai lệnh này.

## Thử py.typed

Với wheel có `py.typed`, mypy phân tích gói và bắt lỗi trong `demo-errors/consumer.py` (gán `Decimal` cho biến `str`):

```bash
uv run --no-project --isolated --with $L/out/*.whl --with mypy mypy $L/demo-errors/consumer.py
```

Kết quả: `error: Incompatible types in assignment (expression has type "Decimal", variable has type "str")  [assignment]`. Để thấy phiên bản thiếu `py.typed`, chép lab ra chỗ khác, xóa file rồi build lại:

```bash
cp -R $L /tmp/lab20np && rm -rf /tmp/lab20np/.venv /tmp/lab20np/out /tmp/lab20np/src/toinoi_pricing/py.typed
uv build --project /tmp/lab20np --out-dir /tmp/lab20np/out
uv run --no-project --isolated --with /tmp/lab20np/out/*.whl --with mypy mypy $L/demo-errors/consumer.py
```

Kết quả: `error: Skipping analyzing "toinoi_pricing": module is installed, but missing library stubs or py.typed marker  [import-untyped]`.

## Thử pre-commit (không cài hook vào repo này)

`pre-commit install` ghi vào `.git/hooks` nên lab không làm. Để thử cấu hình, chép lab ra một thư mục tạm và tạo repo ở đó (cần mạng để tải hook của ruff lần đầu):

```bash
cp -R $L /tmp/lab20 && cd /tmp/lab20 && rm -rf .venv out
git init -q && git add -A && uvx pre-commit run --all-files
```

Kết quả kỳ vọng: bảy dòng `Passed` (ruff format, ruff check, mypy --strict, end-of-file, trailing whitespace, check toml, check yaml).
