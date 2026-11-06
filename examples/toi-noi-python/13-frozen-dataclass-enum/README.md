# Lab 13 · Frozen dataclass, Enum, equality và immutability

Đi kèm bài `python-frozen-dataclass-enum-immutability`. Lab này không có `Trip` (trạng thái gắn vào `Trip` ở lab 16). Thiết kế `TripStatus` và `ServiceKind` bằng `StrEnum`, value object `Money` và `Quote` bằng `@dataclass(frozen=True)` (có `__post_init__`, `__add__`, `is_valid(now)`, `dataclasses.replace`), đối chiếu với `NamedTuple` (`FarePolicy` giữ nguyên từ bài 10) và chỉ ra giới hạn của `frozen` (nông, không chống cố tình).

## Chạy

Từ thư mục gốc repository (không cần mạng, không cần thư viện ngoài):

```bash
uv run --project examples/toi-noi-python/13-frozen-dataclass-enum examples/toi-noi-python/13-frozen-dataclass-enum/main.py
uv run --project examples/toi-noi-python/13-frozen-dataclass-enum examples/toi-noi-python/13-frozen-dataclass-enum/main.py --verify
```

## Kết quả kỳ vọng

- Lệnh đầu in năm phần: Enum, Money, Quote có hạn, NamedTuple so với frozen dataclass, giới hạn của frozen. Không có số đo phụ thuộc máy; mọi dòng lặp lại được.
- `--verify` kết thúc bằng `OK 64/64` (exit code 0); ca sai thì in `SAI: …`, rồi `FAIL n/m` (n là số ca đạt) và thoát với mã 1.
- Thời hạn báo giá 5 phút (`QUOTE_TTL`) là giả định học tập.
