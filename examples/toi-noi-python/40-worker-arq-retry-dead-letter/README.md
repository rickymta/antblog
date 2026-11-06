# Lab 40 · Worker nền với arq: lịch, retry, dead-letter

Đi kèm bài `python-worker-arq-scheduling-retry-dead-letter` (bài tổng hợp P03–P04).

- `app/settings.py`: URL Redis, tên hàng đợi và key (`lab40:`), serializer JSON thay cho pickle mặc định của arq.
- `app/scoring.py`: chấm điểm giả bằng quy tắc; backend giả có thể lỗi hoặc chậm theo kế hoạch ghi trong Redis.
- `app/jobs.py`: `score_trip` (retry backoff, dead-letter, idempotent theo `trip_id`), `report_queue_health` (cron), `backoff_delay`.
- `app/worker.py`: `WorkerSettings` (max_tries, max_jobs, timeout, `job_completion_wait`, cron).
- `app/dlq.py`: xem và đưa lại (redrive) dead-letter.
- `main.py`: kịch bản đầy đủ, có khởi động một worker thật để thử SIGTERM.

Cần Redis của hạ tầng dùng chung, chạy từ thư mục gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/40-worker-arq-retry-dead-letter examples/toi-noi-python/40-worker-arq-retry-dead-letter/main.py
uv run --project examples/toi-noi-python/40-worker-arq-retry-dead-letter examples/toi-noi-python/40-worker-arq-retry-dead-letter/main.py --verify
```

Chạy riêng một worker (lệnh `arq` tìm module `app` theo thư mục hiện tại nên cần `--directory`):

```bash
uv run --directory examples/toi-noi-python/40-worker-arq-retry-dead-letter arq app.worker.WorkerSettings
```

Kết quả kỳ vọng: `--verify` kết thúc bằng `OK 26/26` (exit 0). Khi không nối được Redis, lab in `FAIL: không nối được Redis tại <url> ...` và exit 1. `--keep` giữ key sau khi chạy; `--cleanup` chỉ xóa key.

arq không cho đổi tiền tố key nội bộ (`arq:job:`, `arq:result:`...), nên lab đặt `job_id` có chữ `lab40` và dọn bằng hai mẫu `lab40:*` và `arq:*lab40*`. Backoff trong lab là mili-giây để chạy nhanh. `arq` đang ở chế độ chỉ bảo trì (README của dự án); bài viết nói rõ và nêu hướng thay thế.

Phiên bản đã chạy (xem `uv.lock`): Python 3.14.7, arq 0.28.0, redis-py 5.3.1, Redis 7.4.
