# Lab 02 · Biến, kiểu dữ liệu và Decimal

Code của bài `python-bien-kieu-du-lieu-decimal`: biến là tên gắn với đối tượng, `int`/`float`/`Decimal`, `quantize` với `ROUND_HALF_UP`, chuyển đổi kiểu, truthiness, `None`.

Chạy từ gốc repository:

```bash
uv run --project examples/toi-noi-python/02-bien-kieu-du-lieu-decimal examples/toi-noi-python/02-bien-kieu-du-lieu-decimal/main.py
uv run --project examples/toi-noi-python/02-bien-kieu-du-lieu-decimal examples/toi-noi-python/02-bien-kieu-du-lieu-decimal/main.py --verify
```

Chương trình in 30 dòng đánh số; các dòng này khớp với phần «Kết quả» trong bài. `--verify` so từng dòng với danh sách `EXPECTED`, in `OK 30/30` và thoát mã 1 nếu có dòng khác.

Bảng kiểm bằng tay (dòng 22–25): 0 km → 12000, 2 km → 12000, 2,5 km → 14250, 5 km → 25500 VND.
