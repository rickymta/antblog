# Lab 38 - Kafka với aiokafka: producer, consumer group, commit offset, schema version

Bài viết: `api/seed/content/python-kafka-aiokafka-producer-consumer-schema.md`.

- `envelope.py`: sự kiện `BookingCreated` trong envelope JSON `{event_id, event_type, occurred_at, version, payload}`; `encode`/`decode`, version 1 và 2 được hỗ trợ, version lạ hoặc không phải JSON bị từ chối bằng `EnvelopeError` có mã lý do.
- `main.py` (`aiokafka`), 7 phần trên topic `lab38.booking.created.v1` (3 partition): producer key = `customer_id` (idempotent, `acks=all`); không key làm mất thứ tự theo khách; gửi từng cái so với gửi theo lô; schema version qua Kafka; consumer group `lab38-fraud-scoring` với `enable_auto_commit=False` và commit sau xử lý, tái hiện xử lý trùng khi sập trước commit; commit trước xử lý làm mất sự kiện; hai consumer cùng group chia partition.

## Hạ tầng cần

Kafka trong hạ tầng dùng chung (`localhost:9094`, ghi đè bằng `TOINOI_KAFKA_BOOTSTRAP`):

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
```

Lab tự tạo bốn topic `lab38.*` (xóa leftover từ lần chạy hỏng trước) và xóa chúng khi chạy xong. Lab không xóa consumer group `lab38-*`: admin client của aiokafka không có lệnh xóa group. Khi chưa có Kafka: `FAIL: không nối được Kafka tại <url> (docker compose ...)` và exit code 1.

## Chạy

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/38-kafka-aiokafka-producer-consumer examples/toi-noi-python/38-kafka-aiokafka-producer-consumer/main.py
uv run --project examples/toi-noi-python/38-kafka-aiokafka-producer-consumer examples/toi-noi-python/38-kafka-aiokafka-producer-consumer/main.py --verify
```

Kết quả `--verify` kỳ vọng: `OK 26/26`, exit code 0. Một lần chạy mất khoảng nửa phút, phần lớn là chờ: broker đợi 3 giây (`group.initial.rebalance.delay.ms`) khi một group đang rỗng có thành viên đầu tiên, và mỗi lần đọc xong lab chờ vài giây không có tin mới để biết đã hết dữ liệu. Số mili giây và số sự kiện/giây phụ thuộc máy; số sự kiện, ánh xạ khách -> partition (hash key xác định) và số lần xử lý trùng thì lặp lại được; phân công partition cho E và F thì không (đổi theo từng lần chạy), lab chỉ kiểm hai tập rời nhau và đủ 3.
