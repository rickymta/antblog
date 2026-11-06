# Lab 30: criterion, dhat và đo ghi PostgreSQL

Dùng lại `ledger-core` (lab 20) làm workload. Chứng minh: `cargo bench` với criterion (`black_box`, `BenchmarkGroup`, `Throughput::Elements`), debug so với release (`lto`, `codegen-units`), đếm allocation bằng `dhat` (chỉ bật theo feature), và ghi 10.000 bút toán vào PostgreSQL thật bằng bốn cách.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'   # compose: infra/README.md
cargo run --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml
cargo bench --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml
cargo run --release --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml -- --timing
cargo run --release --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml -- --pg-bench
cargo run --release --features dhat-heap --bin alloc-profile --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml
```
`--verify` kiểm kết quả nghiệp vụ (không kiểm số đo tuyệt đối) và cần PostgreSQL: thiếu `TOI_NOI_PG_URL` thì thoát mã 1. Mỗi ca chạy trong schema `lab_<uuid>` riêng và xóa nó khi xong. Số đo chỉ có ý nghĩa trên máy đã đo, không phải SLA.
Dependency: `ledger-core` (path), `sqlx` 0.9, `tokio`, `uuid`, `dhat` (optional), `criterion` (dev). Dữ liệu hư cấu của Tới Nơi.
