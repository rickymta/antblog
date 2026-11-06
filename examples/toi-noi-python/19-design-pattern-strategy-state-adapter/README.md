# Lab 19: Strategy, State và Adapter kiểu Python

Bài viết: `api/seed/content/python-design-pattern-strategy-state-adapter.md`.

- `domain.py`: `ServiceKind`, `TripStatus`, `FarePolicy`, `calculate_fare` (copy từ các bài trước).
- `strategy.py`: Strategy bằng hàm (`standard`, `audited`), bằng lớp có trạng thái (`CachedFare`), bằng `Protocol` (`FareCalculator`), chọn theo dịch vụ bằng dict.
- `state.py`: vòng đời 8 trạng thái viết hai cách, bảng chuyển (`ALLOWED_TRANSITIONS`, `TableTrip`) và mỗi trạng thái một lớp (`TripState`, `Trip`).
- `adapter.py`: SDK bản đồ giả (`MapVietClient`, trả mét kiểu float) và `MapVietAdapter` thành `RoutingService` trả km `Decimal`.
- `main.py`: demo và `--verify`.

Chạy từ thư mục gốc repository (không cần mạng khi chạy; lần đầu uv tải mypy):

```bash
uv run --project examples/toi-noi-python/19-design-pattern-strategy-state-adapter examples/toi-noi-python/19-design-pattern-strategy-state-adapter/main.py
uv run --project examples/toi-noi-python/19-design-pattern-strategy-state-adapter examples/toi-noi-python/19-design-pattern-strategy-state-adapter/main.py --verify
uv run --project examples/toi-noi-python/19-design-pattern-strategy-state-adapter mypy --config-file examples/toi-noi-python/19-design-pattern-strategy-state-adapter/pyproject.toml examples/toi-noi-python/19-design-pattern-strategy-state-adapter/*.py
```

Kết quả kỳ vọng: `--verify` in dòng cuối `OK 26/26`, exit code 0; có ca sai thì in `SAI: <tên ca>`, dòng `FAIL n/26` (n là số ca đạt) và exit code 1. mypy in `Success: no issues found in 5 source files`.
