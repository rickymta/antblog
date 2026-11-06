# Bài 18 · functools, itertools và pipeline xử lý lô chuyến

Lab của bài `python-functools-itertools-pipeline`: 10.000 chuyến giả (6 thành phố, `Random(seed)`), pipeline generator lọc chuyến tính cước được, tính cước, gom doanh thu theo thành phố; `partial`, `reduce`, `lru_cache`/`cache`, `groupby`, `chain.from_iterable`, `batched`, `accumulate`, `islice`. Không dùng pandas.

```bash
uv run --project examples/toi-noi-python/18-functools-itertools-pipeline examples/toi-noi-python/18-functools-itertools-pipeline/main.py
uv run --project examples/toi-noi-python/18-functools-itertools-pipeline examples/toi-noi-python/18-functools-itertools-pipeline/main.py --verify
```

- Cố định theo seed: 8.466 chuyến completed, 8.365 chuyến tính cước được, 101 chuyến completed bị loại vì khoảng cách ngoài miền, tổng doanh thu 338.740.950 VND.
- Thay đổi theo máy: thời gian chạy có/không cache và số byte `tracemalloc`. Bộ nhớ đo ở hai chế độ: không cache (`calculate_fare` thẳng) và cache lạnh (`cache_clear()` trước mỗi lần đo); chi phí riêng của cache được in tách ra.
- `--verify` phải in `OK 21/21` và thoát mã 0; sai thì in tên ca sai, `FAIL n/m` và thoát mã 1.

Python 3.14 (`itertools.batched` có từ 3.12), không cần mạng, không có thư viện ngoài.
