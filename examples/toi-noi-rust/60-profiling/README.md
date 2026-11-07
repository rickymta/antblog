# Lab 60: profiling sổ cái bằng samply, dhat và tokio-console

Ba cách cài cùng một kịch bản (ghi bút toán rồi đọc số dư): `SlowBook` (quét toàn sổ + `clone` + `format!`), `ScanBook` (hết `clone`/`format!`, vẫn quét), `IndexedBook` (số dư cộng dồn). Dùng lại `ledger-core` (lab 20). Không cần hạ tầng.

```bash
cargo run --manifest-path examples/toi-noi-rust/60-profiling/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/60-profiling/Cargo.toml
cargo run --release --manifest-path examples/toi-noi-rust/60-profiling/Cargo.toml -- --report --n 2000
cargo run --release --features dhat-heap --bin alloc-profile --manifest-path examples/toi-noi-rust/60-profiling/Cargo.toml
RUSTFLAGS="--cfg tokio_unstable" CARGO_TARGET_DIR=examples/toi-noi-rust/60-profiling/target/console cargo run --release --features console --bin console-probe --manifest-path examples/toi-noi-rust/60-profiling/Cargo.toml -- --verify
```
CPU profile (trong thư mục lab): `cargo install samply`, rồi `samply record --save-only --unstable-presymbolicate -o target/samply-slow.json.gz target/release/ledger-profiling --workload slow --n 1000` và `python3 scripts/samply_top.py target/samply-slow.json.gz --focus "SlowBook as"` (bảng gộp, không phải flamegraph; bỏ `--save-only` để mở Firefox Profiler). `--verify` chỉ kiểm bộ đếm (số lần gọi, số posting duyệt) và đáp án, không kiểm thời gian. Số đo chỉ có ý nghĩa trên máy đã đo, không phải SLA.
Dependency: `ledger-core` (path), `tokio`, `serde_json`; tùy chọn `dhat`, `console-subscriber`, `console-api`, `tonic`. Dữ liệu hư cấu của Tới Nơi.
