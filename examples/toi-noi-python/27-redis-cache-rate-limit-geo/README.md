# Lab 27 - Redis: cache báo giá, rate limit và GEO vị trí tài xế

Bài viết: `api/seed/content/python-redis-cache-rate-limit-geo.md`.

- `main.py` (`redis.asyncio`, `decode_responses=True`): cache-aside cho báo giá với TTL (`SET ... PX`), chuẩn hóa km trước khi vào key lẫn khi tính giá; chống dội cache nguội bằng khóa `SET NX PX` kiểm lại sau khi giành khóa, tách thời gian chờ (`wait_ms`, hết thì `QuoteUnavailable`) khỏi tuổi thọ khóa (`lock_ms`); rate limit cửa sổ cố định (`INCR` + `EXPIRE NX` trong MULTI/EXEC) và token bucket bằng Lua (`register_script`, không ghi lùi `ts` khi đồng hồ client lệch); so sánh `GET`/`SET` với Lua khi 50 coroutine đua nhau; `GEOADD`/`GEOSEARCH` cho 25.000 tài xế giả quanh Hà Nội, đối chiếu với haversine tính tay, so sánh ba cách nạp (từng lệnh, pipeline, GEOADD nhiều thành viên) và dựng lại chỉ mục sau khi mất key.
- Redis chỉ là cache/chỉ mục: nguồn sự thật (bảng giá, vị trí gốc tài xế) là danh sách trong bộ nhớ đóng vai database. Dữ liệu tổng hợp, seed cố định.

## Hạ tầng cần

Redis trong hạ tầng dùng chung (`redis://localhost:63790/0`, ghi đè bằng `TOINOI_REDIS_URL`):

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
```

Mọi key của lab có tiền tố `lab27:` và bị xóa (SCAN + UNLINK) khi chạy xong. Cần Redis 7.0 trở lên (`EXPIRE ... NX`).

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/27-redis-cache-rate-limit-geo examples/toi-noi-python/27-redis-cache-rate-limit-geo/main.py
uv run --project examples/toi-noi-python/27-redis-cache-rate-limit-geo examples/toi-noi-python/27-redis-cache-rate-limit-geo/main.py --verify
```

Kết quả `--verify` kỳ vọng: `OK 44/44`, exit code 0 (thời gian chạy tùy máy, cỡ 10 đến 25 giây, vì cách nạp từng lệnh 25.000 vòng đi-về là một phần của phép đo; `--verify` chỉ kiểm số đếm và bất biến, không kiểm số mili giây). Các con số mili giây in ra phụ thuộc máy; các số đếm thì lặp lại được. Khi chưa có Redis: `FAIL: không nối được Redis tại <url> (docker compose …)` và exit code 1.
