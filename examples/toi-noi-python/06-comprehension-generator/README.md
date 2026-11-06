# Bài 06 · Comprehension, iterator và generator

Lab của bài `python-comprehension-generator-iterator`: giả lập 1 triệu điểm GPS tài xế (random có seed, khoảng 160 giây dữ liệu ở 6.250 dòng/giây), lọc điểm có tốc độ phi lý (> 120 km/h) bằng generator và so bộ nhớ với cách nạp hết vào list.

```bash
uv run --project examples/toi-noi-python/06-comprehension-generator examples/toi-noi-python/06-comprehension-generator/main.py
uv run --project examples/toi-noi-python/06-comprehension-generator examples/toi-noi-python/06-comprehension-generator/main.py --verify
```

- Lệnh đầu in các phần comprehension, iterator, generator, itertools, rồi so generator với list. Chạy mất vài giây.
- Cố định theo seed: 972 điểm phi lý trong 1.000.000 điểm; điểm phi lý đầu tiên ở seq 2306.
- Thay đổi theo máy: thời gian chạy và số byte `tracemalloc` (trên máy tác giả: generator vài KiB, list khoảng 229 MiB).
- `--verify` phải in `OK 15/15` và thoát mã 0; sai thì in `FAIL` và thoát mã 1.

Python 3.14 (`itertools.batched` có từ 3.12), không cần mạng, không có thư viện ngoài.
