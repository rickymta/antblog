# Lab 07: module, package và import

Bài viết: `api/seed/content/python-module-package-import.md`.

Gói `toinoi_fare` (layout `src/`) gồm `models.py`, `pricing.py`, `__init__.py`, `__main__.py`. `main.py` ở gốc lab import gói bằng import tuyệt đối. `pyproject.toml` khai báo `[build-system]` (uv_build) và `[tool.uv] package = true` để uv cài gói vào môi trường ở chế độ editable. Gói `cycle_demo/` chứa hai module cố ý import vòng (không thuộc `toinoi_fare`), chạy từ gốc lab bằng `import cycle_demo.fare_pricing`.

## Chạy

```bash
uv run --project examples/toi-noi-python/07-module-package-import examples/toi-noi-python/07-module-package-import/main.py
uv run --project examples/toi-noi-python/07-module-package-import python -m toinoi_fare 2.5
```

## Verify

```bash
uv run --project examples/toi-noi-python/07-module-package-import examples/toi-noi-python/07-module-package-import/main.py --verify
```

Kết quả kỳ vọng: dòng cuối `OK 18/18`, exit code 0. Có ca sai thì in tên ca, dòng `FAIL n/18` và exit code 1.
