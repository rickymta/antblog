# Lab 25 · Transaction, optimistic concurrency và idempotency: đặt chuyến an toàn

Đi kèm bài `python-transaction-optimistic-concurrency-idempotency`.

- `models.py`: `Trip` (cột `version`), `BookingRequest` (UNIQUE `(customer_id, idempotency_key)`), `VersionedTrip` (ORM `version_id_col`), schema `lab25`.
- `booking.py`: `run_in_transaction` (retry có giới hạn khi 40001/40P01), `accept_trip` (`UPDATE ... WHERE version = :v`), `book_trip` (idempotent, bắt đúng SQLSTATE 23505 của đúng constraint).
- `main.py`: tái hiện rollback, `READ COMMITTED` và `REPEATABLE READ`, mất cập nhật bằng hai session, optimistic concurrency, `StaleDataError`, idempotency và cuộc đua hai thread cùng khóa. Cuối cùng `DROP SCHEMA lab25 CASCADE`.

Cần hạ tầng dùng chung (PostgreSQL), chạy từ thư mục gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/25-transaction-optimistic-idempotency examples/toi-noi-python/25-transaction-optimistic-idempotency/main.py
uv run --project examples/toi-noi-python/25-transaction-optimistic-idempotency examples/toi-noi-python/25-transaction-optimistic-idempotency/main.py --verify
```

Kết quả kỳ vọng: `--verify` kết thúc bằng `OK 24/24` (exit 0). Khi không nối được PostgreSQL, lab in `FAIL: không nối được PostgreSQL tại <url> ...` và exit 1. `--keep` giữ schema `lab25` sau khi chạy; `--cleanup` chỉ xóa schema.

Các ca xen kẽ giữa hai người dùng được dựng bằng điểm chèn (`before_write`, `after_lookup`) và `threading.Barrier`, không dùng `sleep`, nên kết quả lặp lại được. Mã chuyến sinh từ sequence nên ca thua cuộc đua để lại một khoảng trống số; lab kiểm điều đó có chủ đích.

Phiên bản đã chạy (xem `uv.lock`): SQLAlchemy 2.1.8, psycopg 3.3.8, Python 3.14.7, PostgreSQL 17.
