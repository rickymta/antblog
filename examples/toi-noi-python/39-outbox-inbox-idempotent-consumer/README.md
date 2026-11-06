# Lab 39 · Outbox, Inbox và idempotent consumer

Đi kèm bài `python-outbox-inbox-idempotent-consumer`.

- `models.py`: `trips`, `outbox` (phía phát), `inbox`, `payout_lines`, `score_requests` (phía nhận), schema `lab39`.
- `outbox.py`: `complete_trip` (chuyến + outbox cùng transaction), `claim_batch` (thuê dòng bằng `FOR UPDATE SKIP LOCKED` và `lease_until`), `dispatch_once` (phát ngoài transaction rồi đánh dấu).
- `inbox.py`: `handle_trip_completed` (inbox `ON CONFLICT DO NOTHING` + hiệu ứng trong một transaction).
- `naive.py`: hai cách dual-write không qua outbox, để thấy mất sự kiện và sự kiện «ma».
- `kafka_io.py`: topic, producer, consumer commit tay (at-least-once), đếm và đọc lại topic.
- `main.py`: kịch bản đầy đủ; cuối cùng xóa schema `lab39` và topic `lab39.*`.

Cần hạ tầng dùng chung (PostgreSQL và Kafka), chạy từ thư mục gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/39-outbox-inbox-idempotent-consumer examples/toi-noi-python/39-outbox-inbox-idempotent-consumer/main.py
uv run --project examples/toi-noi-python/39-outbox-inbox-idempotent-consumer examples/toi-noi-python/39-outbox-inbox-idempotent-consumer/main.py --verify
```

Kết quả kỳ vọng: `--verify` kết thúc bằng `OK 21/21` (exit 0). Khi thiếu PostgreSQL hoặc Kafka, lab in `FAIL: không nối được <dịch vụ> tại <url> ...` và exit 1. `--keep` giữ schema và topic sau khi chạy; `--cleanup` chỉ dọn.

Thời gian được truyền vào (`now`) nên kết quả không phụ thuộc đồng hồ máy; không có `sleep` trong các ca. Topic: `lab39.trip.completed.v1` (3 partition) và `lab39.naive.trip.completed.v1`; consumer group `lab39-payout`, `lab39-fraud-scoring`, `lab39-naive-payout`.

Phiên bản đã chạy (xem `uv.lock`): Python 3.14.7, SQLAlchemy 2.1.8, psycopg 3.3.8, aiokafka 0.15.0, PostgreSQL 17, Kafka 3.9.
