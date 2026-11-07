# Lab 59 · Chấm điểm thời gian thực: feature store trên Redis, ngân sách 50 ms và fallback

Đi kèm bài `python-real-time-scoring-feature-store-redis-latency`. Lab phát lại 200.000 chuyến của bộ dữ liệu gian lận tổng hợp (bài 54) vào Redis làm feature store online, đọc 30 đặc trưng của bài 55 cho từng chuyến, chấm điểm bằng mô hình của bài 57 trong ngân sách 50 ms, đo p50/p99 và kiểm hành vi khi store chậm, kẹt, mất hay trống. Điểm rủi ro là đầu vào cho người duyệt, không tự khóa tài khoản và không chặn Dispatch.

- `store.py`: `FeatureStore` (ghi một chuyến bằng một script Lua nguyên tử và idempotent; đọc 30 đặc trưng trong một pipeline chín lệnh; chuyến đã được ghi rồi mới đọc thì trừ chính nó, chuyến cũ đã có chuyến sau thì báo `ahead`), `ClosedTrip`, `TripAssembler` (gom ba envelope thành một chuyến), `compute_features` (thuần, không I/O).
- `scorer.py`: `Scorer.score()` với `asyncio.timeout`, `degraded` + `reason` (`timeout`, `store_error`, `store_cold`, `store_building`, `state_ahead`, `breaker_open`, `overloaded`, `model_error`), `deadline_s` của người gọi, `Breaker` (cầu dao chỉ đếm lỗi của store, đồng hồ tiêm vào), giới hạn lời gọi cùng lúc tính cả luồng mô hình còn chạy.
- `faults.py`: `InjectedStore` (độ trễ hoặc kẹt tiêm vào), `BlockingModel`.
- `train.py`: dựng bảng đặc trưng theo lô (`out/table.parquet`) và huấn luyện mô hình nếu `out/` chưa có (như bài 57, không dạy lại).
- `main.py`: sáu bước và `--verify`.
- `synthetic_fraud.py`, `dataset.py` copy NGUYÊN từ lab 54; `features.py`, `model.py`, `model_io.py` copy từ lab 57 (`features.py` giống lab 55).

Key Redis (mọi key có tiền tố `lab59:`, xóa khi chạy xong): `lab59:win:dev|cust|drv|pair:...` (sorted set, score = `paid_at` micro giây), `lab59:feat:{CU|DR}-...` (hash), `lab59:meta:applied_until`.

## Hạ tầng cần

Redis trong hạ tầng dùng chung (`redis://localhost:63790/0`, ghi đè bằng `TOINOI_REDIS_URL`):

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
```

Chưa có Redis: `FAIL: không nối được Redis tại <url> (docker compose ...)` và exit code 1. Trên macOS, LightGBM cần `brew install libomp`. Lần đầu cần mạng để tải thư viện và dựng bảng đặc trưng, huấn luyện mô hình (khoảng một phút, lưu `out/`).

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/59-real-time-scoring-redis examples/toi-noi-python/59-real-time-scoring-redis/main.py
uv run --project examples/toi-noi-python/59-real-time-scoring-redis examples/toi-noi-python/59-real-time-scoring-redis/main.py --verify
```

## Kết quả kỳ vọng

- `--verify` kết thúc bằng `Kiểm chứng: OK n/n` (exit 0); lệch thì in `SAI: <tên ca>` rồi `FAIL k/n` với k là số ca đạt.
- Thời gian chạy phụ thuộc tải máy: khoảng hai phút khi load average cỡ 100 trên 10 lõi, 9 đến 10 phút khi máy quá bận (phần lâu nhất là đo từng chuyến ở bước 3).
- `--verify` kiểm bất biến: đặc trưng online khớp bản polars trên 100.080 giá trị (kể cả khi chuyến đã được ghi rồi mới đọc), điểm online khớp điểm theo lô, cận cửa sổ, ghi trùng, chuyến đến muộn, store trống và đang dựng, chuyến cũ bị bỏ qua, và hành vi fallback bằng độ trễ tiêm vào và đồng hồ giả. Không kiểm tốc độ tuyệt đối; bảng p50/p99 phụ thuộc máy và tải lúc đo (lab in load average).
- Mọi dữ liệu là tổng hợp; số đo không đại diện cho dữ liệu thật. Mô hình chạy trong tiến trình Python (không qua gRPC); lab không đo mạng thật giữa Dispatch và service.
