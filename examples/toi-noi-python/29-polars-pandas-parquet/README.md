# Lab 29 - polars, pandas và Parquet: phân tích 2 triệu chuyến mỗi ngày

Bài viết: `api/seed/content/python-polars-pandas-parquet-phan-tich-chuyen.md`.

`main.py` sinh 2.000.000 chuyến giả của một ngày (seed cố định, 6 thành phố, 3 dịch vụ, tải đỉnh sáng và chiều), ghi `out/trips.csv` và `out/trips.parquet`, rồi đo: kích thước, thời gian đọc, truy vấn lazy `scan_parquet`, pandas so với polars (thời gian và bộ nhớ, mỗi kịch bản một tiến trình), ngoại suy lên 730 triệu dòng/năm và bẫy cộng Int32.

Cần uv và Python 3.14; lần đầu cần mạng để tải numpy, polars, pandas, pyarrow. Không cần Docker hay dịch vụ nào. Dữ liệu sinh vào `out/` (gitignore, khoảng 200 MiB).

```bash
uv run --project examples/toi-noi-python/29-polars-pandas-parquet examples/toi-noi-python/29-polars-pandas-parquet/main.py
uv run --project examples/toi-noi-python/29-polars-pandas-parquet examples/toi-noi-python/29-polars-pandas-parquet/main.py --verify
uv run --project examples/toi-noi-python/29-polars-pandas-parquet examples/toi-noi-python/29-polars-pandas-parquet/main.py --rows 500000
```

`--verify` kiểm 17 bất biến (số dòng, cước số nguyên khớp `Decimal`, Parquet đọc lại bằng nhau, polars lazy == eager == streaming == pandas, Parquet nhỏ hơn CSV ít nhất 3 lần...) và in `OK 17/17`. Mọi thời gian và kích thước trong bài là số đo trên một máy cụ thể (macOS, Apple silicon, 10 lõi); máy khác cho số khác, tỉ lệ giữa các dòng mới là thứ đáng so. Tổng doanh thu và số chuyến mỗi nhóm phụ thuộc phiên bản numpy (bộ sinh số ngẫu nhiên `Generator` không cam kết giữ nguyên dòng số giữa các phiên bản), nên `--verify` không ghim các tổng đó.
