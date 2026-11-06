# Lab 36: Kafka với rdkafka (consume TripCompleted, produce LedgerEntryPosted)

Cần Kafka trong `examples/toi-noi-rust/infra` đã bật. Sổ cái là `Ledger` trong bộ nhớ của `ledger-core` (lab 20), đứng thay cho PostgreSQL để lab tập trung vào Kafka.

```bash
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run --manifest-path examples/toi-noi-rust/36-kafka-rdkafka/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/36-kafka-rdkafka/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/36-kafka-rdkafka/Cargo.toml
```

Chứng minh, mỗi kịch bản trên Kafka thật: `FutureProducer` + `StreamConsumer` với `enable.auto.commit=false` và commit sau khi ghi; at-least-once (xử lý xong, chưa commit, khởi động lại: nhận lại, bút toán chỉ một nhờ `idempotency_key`); key partition (`tripId` so với id tài khoản); consumer group và rebalance (hai consumer, in partition được gán); thông điệp hỏng/version lạ bị bỏ qua mà không kẹt partition (DLQ ở bài 37); không commit không làm consumer đọc lại (phải `seek`); hết giờ chờ thì lỗi `Timeout` rõ ràng. Topic `lab.36.<uuid>.*` và group `lab-36-<uuid>*` được `AdminClient` tạo và xoá mỗi ca, nên chạy song song được. Thiếu `TOI_NOI_KAFKA` hoặc không nối được broker thì FAIL, không skip.

Phụ thuộc chính: `rdkafka` 0.39 (librdkafka 2.12.1 build kèm, không cần cmake), `tokio`, `serde`/`serde_json`, `jiff`, `uuid`, `anyhow`, `thiserror`, `ledger-core`.
