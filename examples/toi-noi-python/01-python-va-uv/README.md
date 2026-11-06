# Lab 01 · Python 3.14 và uv: chương trình tính cước đầu tiên

Code của bài `python-va-uv-tu-source-code-den-chuong-trinh`. Mô hình cước minh họa: 12.000 VND gồm 2 km đầu, 4.500 VND mỗi km vượt; không phụ phí, không làm tròn (bài 02 xử lý).

Chạy từ gốc repository (cần [uv](https://docs.astral.sh/uv/); uv tự tải Python 3.14 nếu máy chưa có):

```bash
uv run --project examples/toi-noi-python/01-python-va-uv examples/toi-noi-python/01-python-va-uv/main.py
# thử khoảng cách khác (tự kiểm bằng tay):
uv run --project examples/toi-noi-python/01-python-va-uv examples/toi-noi-python/01-python-va-uv/main.py 2.5
uv run --project examples/toi-noi-python/01-python-va-uv examples/toi-noi-python/01-python-va-uv/main.py --verify
```

Kết quả kỳ vọng:

```text
Quãng đường: 5 km
Cước minh họa: 25500 VND
```

`--verify` kiểm các mốc 0 / 2 / 2,5 / 5 km, trường hợp khoảng cách âm (-1 km vẫn ra 12000, minh họa «chạy được vẫn có thể sai») và bài tập (4 km, 5.000 VND/km vượt = 22000), rồi in `OK 6/6`; thoát mã 1 nếu sai.
