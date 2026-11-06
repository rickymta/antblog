# Lab 04 · Chuỗi, f-string, ngày giờ và múi giờ

Đi kèm bài `python-chuoi-f-string-ngay-gio-timezone`. Minh họa `str` và `bytes` với tiếng Việt (`encode/decode`, `errors=`, NFC/NFD), f-string và format spec, `format_vnd` tự viết (quantize HALF_UP trước, không dùng `locale`), `strip/split/join`, `datetime` aware và `zoneinfo`, `replace` khác `astimezone`, ISO 8601, `strftime/strptime`, `timedelta`, ngày nghiệp vụ theo giờ Việt Nam, và một ví dụ template string (t-string, PEP 750, Python 3.14). Thời điểm trong ví dụ là cố định để kết quả tái lập.

Lưu ý: `zoneinfo` đọc cơ sở dữ liệu múi giờ của hệ điều hành. Trên Windows hoặc image container tối giản có thể cần thêm gói `tzdata`; lab này không thêm dependency nên chạy tốt trên macOS và Linux có tzdata.

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/04-chuoi-ngay-gio-timezone examples/toi-noi-python/04-chuoi-ngay-gio-timezone/main.py
uv run --project examples/toi-noi-python/04-chuoi-ngay-gio-timezone examples/toi-noi-python/04-chuoi-ngay-gio-timezone/main.py --verify
```

## Kết quả kỳ vọng

- Lệnh đầu in, trong số các dòng khác, `'Tới Nơi': 7 ký tự, 10 byte UTF-8`, `format_vnd(12004.5) -> 12.005`, `+07:00: 2026-11-06T08:30:00+07:00`, `thời lượng: 22 phút 15 giây` và bảng ba chuyến với ngày UTC khác ngày Việt Nam.
- Lệnh `--verify` kết thúc bằng `OK 39/39` (exit code 0); nếu có ca sai thì in dòng `SAI: …`, rồi `FAIL n/m` và thoát với mã 1.
