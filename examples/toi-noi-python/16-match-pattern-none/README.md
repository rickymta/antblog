# Lab 16 · Pattern matching và xử lý None

Đi kèm bài `python-match-pattern-matching-none`. Lab tự chứa, không cần hạ tầng, không cần mạng khi chạy (lần resolve đầu cần mạng để tải `mypy`).

- `trip.py`: `TripStatus(StrEnum)`, `ServiceKind`, `Trip` dataclass đủ bảy trường theo từ điển miền (không phải `Trip` đóng gói của bài 11).
- `matching.py`: `describe` (class pattern), `apply_command` (sequence, or-pattern, guard), `next_step_label` (`assert_never`), `handle_event` (mapping pattern), `driver_label`/`patch_trip` (None, sentinel), `TypeIs`/`TypeGuard`.
- `unsafe_example.py`: code cố ý sai để xem mypy báo gì; `main.py` không import file này.
- `main.py`: các demo in trong bài và chế độ `--verify`.

Phiên bản đã chạy: Python 3.14.7, mypy 2.4.1 (xem `uv.lock`).

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/16-match-pattern-none examples/toi-noi-python/16-match-pattern-none/main.py
uv run --project examples/toi-noi-python/16-match-pattern-none examples/toi-noi-python/16-match-pattern-none/main.py --verify
uv run --project examples/toi-noi-python/16-match-pattern-none mypy examples/toi-noi-python/16-match-pattern-none/main.py --strict
uv run --project examples/toi-noi-python/16-match-pattern-none mypy examples/toi-noi-python/16-match-pattern-none/unsafe_example.py --strict
```

## Kết quả kỳ vọng

- Lệnh đầu in năm nhóm demo: mô tả 10 chuyến, đi qua 7 trạng thái, định tuyến 8 envelope, các bẫy cú pháp, và `None` so với sentinel.
- `--verify` kết thúc bằng `OK 56/56` (exit code 0). Nếu lệch: in `SAI: …`, rồi `FAIL n/m` (n là số ca đạt, m là tổng) và exit 1.
- `mypy main.py --strict`: `Success: no issues found`. `mypy unsafe_example.py --strict`: 2 lỗi (`driver_id` có thể là `None`, `assert_never` còn trạng thái chưa xử lý) và 4 dòng `Revealed type`. Thông báo `assert_never` liệt kê trạng thái theo thứ tự khai báo khi chạy với `--no-incremental`; với cache mypy đã có, từng thấy thứ tự khác.
