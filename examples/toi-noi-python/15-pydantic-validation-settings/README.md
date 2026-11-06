# Lab 15 · Pydantic v2: validate hồ sơ tài xế và cấu hình ứng dụng

Đi kèm bài `python-pydantic-validation-settings`. Lab tự chứa, không cần hạ tầng, không cần mạng khi chạy (lần resolve đầu cần mạng để tải `pydantic`, `pydantic-settings`).

- `models.py`: `DriverProfile(BaseModel)` (field validator, model validator, alias, `repr=False` cho số giấy tờ), `QuoteRequest` (lax, từ chối float bằng validator `before`), `StrictQuoteBody`/`StrictQuoteRequest` (strict), `FrozenDriverProfile` và `updated()` (đổi hồ sơ an toàn), `error_report()` đổi `ValidationError` thành danh sách lỗi phẳng, `DriverRecord` là dataclass để so sánh.
- `settings.py`: `AppSettings(BaseSettings)` đọc `TOINOI_*` (`TOINOI_PG_URL`, `TOINOI_QUOTE_TTL_SECONDS`, ...), `SecretStr` cho chuỗi nối có mật khẩu.
- `main.py`: các demo in trong bài và chế độ `--verify`.

Dữ liệu là giả: CCCD bắt đầu `099`, GPLX bắt đầu `99`, điện thoại `0900 000 001`. Phiên bản đã chạy: Python 3.14.7, pydantic 2.14.3, pydantic-settings 2.16.2 (xem `uv.lock`).

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/15-pydantic-validation-settings examples/toi-noi-python/15-pydantic-validation-settings/main.py
uv run --project examples/toi-noi-python/15-pydantic-validation-settings examples/toi-noi-python/15-pydantic-validation-settings/main.py --verify
```

## Kết quả kỳ vọng

- Lệnh đầu in các nhóm demo: dataclass nhận rác còn `BaseModel` báo lỗi; hồ sơ hợp lệ, `model_dump`; năm lỗi cùng lúc có `field`/`code`/`message`; JSON lax, strict và từ chối float; gán lại thuộc tính (giới hạn của `validate_assignment`, `frozen`); lỗi không lộ số giấy tờ (`hide_input_in_errors`); `BaseSettings` (mặc định, env, `.env`, `SecretStr`, cấu hình sai).
- `--verify` kết thúc bằng `OK 53/53` (exit code 0). Nếu lệch: in `SAI: …`, rồi `FAIL n/m` (n là số ca đạt, m là tổng) và exit 1.
- `--verify` tự gỡ mọi biến `TOINOI_*` đang có trên máy trong lúc chạy, nên kết quả không phụ thuộc môi trường của bạn.
