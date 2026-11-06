# Lab 11 · Class, dataclass, property và encapsulation

Đi kèm bài `python-class-dataclass-property-encapsulation`. Dựng `Trip` bằng class thường (`TripPlain`), rồi `@dataclass` với `@property` chỉ đọc, thuộc tính «riêng theo quy ước» `_status`, máy trạng thái 8 trạng thái (`ALLOWED_TRANSITIONS` dùng chuỗi, bài 13 mới đổi sang Enum) và `advance()` từ chối chuyển sai. Phần cuối đo `__slots__` bằng `tracemalloc`.

## Chạy

Từ thư mục gốc repository (không cần mạng, không cần thư viện ngoài):

```bash
uv run --project examples/toi-noi-python/11-class-dataclass-property examples/toi-noi-python/11-class-dataclass-property/main.py
uv run --project examples/toi-noi-python/11-class-dataclass-property examples/toi-noi-python/11-class-dataclass-property/main.py --verify
```

## Kết quả kỳ vọng

- Lệnh đầu in năm phần: class thường, dataclass, property, máy trạng thái, `__slots__`. Hai con số byte của phần cuối phụ thuộc phiên bản CPython và nền tảng (lần chạy trong bài: 112 và 72 byte mỗi đối tượng trên Python 3.14.7, macOS arm64).
- `--verify` kết thúc bằng `OK 36/36` (exit code 0); ca sai thì in `SAI: …`, rồi `FAIL n/m` (n là số ca đạt) và thoát với mã 1. `--verify` không so số byte cụ thể, chỉ so «có slots thì nhỏ hơn».
