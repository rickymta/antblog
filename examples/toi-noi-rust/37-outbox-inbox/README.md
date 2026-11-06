# Lab 37: outbox, inbox và idempotent consumer (PostgreSQL + Kafka)

Chứng minh bằng chạy thật: ghi DB rồi gửi Kafka (tiến trình con tự thoát bằng `process::exit(137)` giữa hai bước, giả lập bị kill) làm mất sự kiện, gửi Kafka rồi ghi DB lỗi để lại sự kiện ma; outbox ghi cùng transaction sqlx với bút toán; relay `SELECT ... FOR UPDATE SKIP LOCKED` rồi đánh dấu `published_at`; relay chết sau khi gửi thì gửi lại; hai relay song song không trùng nhờ khóa nhưng nhiều worker đảo thứ tự; inbox `event_id` unique cùng transaction với hiệu ứng; DLQ `lab.37.<uuid>.dlq` sau 3 lần thử, partition không kẹt.

Cần PostgreSQL và Kafka của `examples/toi-noi-rust/infra`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
export TOI_NOI_KAFKA='127.0.0.1:9092'
cargo run --manifest-path examples/toi-noi-rust/37-outbox-inbox/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/37-outbox-inbox/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/37-outbox-inbox/Cargo.toml
```

Mỗi ca dùng schema `lab_<uuid>` và topic `lab.37.<uuid>.*` riêng, xoá cuối ca, nên chạy lặp và song song được. Thiếu biến môi trường thì lab thoát mã 1 và nói cách bật compose. Dùng lại `ledger-core` của lab 20. Lần build đầu biên dịch librdkafka từ nguồn (cần `make` và trình biên dịch C), mất vài phút.
