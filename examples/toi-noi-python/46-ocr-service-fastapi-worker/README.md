# Lab 46 · Đóng gói OCR thành service: FastAPI, worker arq, timeout

Đi kèm bài `python-ocr-service-fastapi-worker-timeout`. API FastAPI nhận ảnh CCCD giả, xếp job vào hàng đợi arq trên Redis; worker arq chạy Tesseract như tiến trình con có thể bị kill; kết quả đọc lại qua `GET`. Không có trạng thái trong RAM của API.

- `app/main.py`: `create_app(settings)`, `POST /v1/kyc/ocr-jobs` (202, hoặc 200 nếu cùng `kyc_id` và cùng ảnh), `GET /v1/kyc/ocr-jobs/{job_id}`, `/healthz`. `describe_job` đọc trạng thái từ Redis trong một transaction.
- `app/jobs.py`: hàm job `ocr_document`, `WorkerSettings` (JSON thay pickle, `max_jobs`, `job_timeout`, `keep_result`).
- `app/ocr.py`: `run_tesseract` (asyncio subprocess, timeout kill tiến trình con), `parse_card` (regex rút gọn; bài 44 làm đầy đủ).
- `app/images.py`, `app/storage.py`, `app/problems.py`, `app/settings.py`: kiểm ảnh ở cửa, kho đối tượng giả (thư mục), lỗi Problem Details cùng dạng lab 32, cấu hình `TOINOI_*`.
- `timeouts_demo.py`: ba cách đặt timeout và độ trễ vòng lặp sự kiện. `loadtest.py`: tải thử bằng httpx (p50/p95). `harness.py`: ảnh thử, dọn Redis, tiến trình thật.
- `main.py`: demo, `--verify`, `--loadtest`, `--cleanup`. `synthetic_card.py`: thẻ giả, copy từ lab 30.

Cần Redis của hạ tầng dùng chung và Tesseract có ngôn ngữ `vie` và `eng` (`brew install tesseract tesseract-lang`). Từ thư mục gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/46-ocr-service-fastapi-worker examples/toi-noi-python/46-ocr-service-fastapi-worker/main.py
uv run --project examples/toi-noi-python/46-ocr-service-fastapi-worker examples/toi-noi-python/46-ocr-service-fastapi-worker/main.py --verify
uv run --project examples/toi-noi-python/46-ocr-service-fastapi-worker examples/toi-noi-python/46-ocr-service-fastapi-worker/main.py --loadtest 40 --concurrency 2,8,32
```

Chạy riêng từng thành phần (cổng 8046; tài liệu ở `/docs`):

```bash
uv run --directory examples/toi-noi-python/46-ocr-service-fastapi-worker arq app.jobs.WorkerSettings
uv run --directory examples/toi-noi-python/46-ocr-service-fastapi-worker uvicorn app.main:app --port 8046
curl -si -F kyc_id=KYC-0001 -F document_type=cccd -F "file=@card.png;type=image/png" localhost:8046/v1/kyc/ocr-jobs
```

## Kết quả kỳ vọng

- `--verify` kết thúc bằng `OK 42/42` (exit 0): 29 ca trong tiến trình (API qua `httpx.ASGITransport`, worker arq chạy theo lượt), 6 ca bộ tách trường và timeout, 7 ca với hai uvicorn và hai arq là tiến trình thật. Không kiểm số giây tuyệt đối. Khi không nối được Redis lab in `FAIL: không nối được Redis tại <url> ...` và exit 1; thiếu `tesseract` thì in `FAIL: không thấy tesseract ...`.
- Mọi key Redis có `lab46` và bị xóa khi chạy xong; `--cleanup` chỉ xóa key. Giới hạn ảnh 10 MiB (như bài 35), 50 triệu điểm ảnh. `OMP_THREAD_LIMIT=1` được đặt nhưng binary Tesseract Homebrew 5.5.4 không có OpenMP nên không tác dụng ở đây.
- Họ tên đọc đúng khi bỏ dấu nhưng thường sai dấu (Tesseract `vie+eng` đọc chữ hoa có dấu kém); số giấy tờ, giới tính, quê quán đọc đúng trên thẻ giả sạch, ngày sinh đúng 38 đến 40 trên 40 thẻ tùy mức tải thử. Cấu hình `vie+eng --psm 6`, đo trên thẻ phẳng chưa qua tiền xử lý. Đó là đầu vào cho người duyệt, không phải quyết định xác minh.
- Số đo p50/p95 phụ thuộc máy và tải máy lúc chạy; `--loadtest` in load average trước và sau khi đo.
- Đã chạy với Python 3.14.7, fastapi 0.162.0, starlette 1.10.0, uvicorn 0.54.0, arq 0.28.0, redis-py 5.3.1, Redis 7.4.12, pillow 13.0.0, Tesseract 5.5.4, macOS (Apple M1 Max).
