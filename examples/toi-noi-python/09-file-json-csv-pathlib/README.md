# Lab 09 - File, JSON, CSV và pathlib

Bài viết: `api/seed/content/python-file-json-csv-pathlib.md`.

Chương trình đọc `data/trips.csv` (30 chuyến giả, thời điểm UTC) và `data/cities.json` (6 thành phố Tới Nơi), tính cước từng chuyến đã hoàn thành theo mô hình chung (12.000 VND gồm 2 km đầu, 4.500 VND mỗi km vượt, làm tròn `ROUND_HALF_UP`), gom theo thành phố rồi ghi `out/summary.json` và `out/completed_trips.csv`. Thư mục `out/` được tạo khi chạy và nằm trong `.gitignore` của lab.

Chạy từ thư mục gốc repository (hoặc bất kỳ thư mục nào, vì đường dẫn neo theo `__file__`):

```bash
uv run --project examples/toi-noi-python/09-file-json-csv-pathlib examples/toi-noi-python/09-file-json-csv-pathlib/main.py
uv run --project examples/toi-noi-python/09-file-json-csv-pathlib examples/toi-noi-python/09-file-json-csv-pathlib/main.py --demo
uv run --project examples/toi-noi-python/09-file-json-csv-pathlib examples/toi-noi-python/09-file-json-csv-pathlib/main.py --verify
```

Kết quả kỳ vọng: lệnh đầu in 6 dòng thành phố và `Tổng doanh thu chuyến hoàn thành: 641700 VND`; `--verify` in `OK 15/15` (không ghi vào `out/`, dùng thư mục tạm).
