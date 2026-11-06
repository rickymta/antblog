# Lab 14: ABC, Protocol, composition và dependency injection

Bài viết: `api/seed/content/python-abc-protocol-composition-dependency-injection.md`.

- `fare.py`: `ServiceKind` (StrEnum), `FarePolicy` và `calculate_fare` (hàm thuần, copy rút gọn từ bài 10 và 13).
- `ports.py`: Protocol `Clock`, `FarePolicyProvider`; `SystemClock`, `FixedClock`.
- `providers.py`: ABC `PolicyProviderBase` (template method) với `DictPolicyProvider`, `JsonFilePolicyProvider` (đọc `policies.json`).
- `service.py`: `HardcodedQuoteService` (trước) và `QuoteService` nhận phụ thuộc qua constructor (sau); `Money`, `Quote` (có `is_valid(now)`, `now < expires_at`) rút gọn.
- `tests/test_quote_service.py`: 12 test bằng đối tượng giả tự viết, không mock framework.
- `broken.py`: 5 lỗi kiểu cố ý (dòng kết thúc bằng chú thích «E:») để mypy bắt. Không chạy file này.
- `policies.json`: chính sách mặc định (khóa `_note` ghi chú giả định) và một thử nghiệm giá 5.000 VND/km vượt ở Đà Nẵng (dữ liệu học tập).

Cần Python 3.14 và uv; lần đầu uv tải pytest, mypy nên cần mạng. Lab đã chạy với CPython 3.14.7, mypy 2.4.1, pytest 9.

```bash
L=examples/toi-noi-python/14-abc-protocol-di
uv run --project $L $L/main.py
uv run --project $L $L/main.py --verify
uv run --project $L pytest $L -q                          # kỳ vọng: 12 passed
uv run --project $L mypy $L/main.py $L/tests --strict     # kỳ vọng: Success: no issues found in 2 source files
uv run --project $L mypy $L/broken.py --strict            # kỳ vọng: Found 5 errors in 1 file
```

Kết quả kỳ vọng của `--verify`: dòng cuối `OK 21/21`, exit code 0 (đã gồm chạy mypy và pytest thật). Có ca sai thì in tên ca, `FAIL n/21` (n là số ca đạt) và exit code 1.
