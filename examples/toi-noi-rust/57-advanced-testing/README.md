# Lab 57: kiểm thử nâng cao (mutation, fuzz, snapshot, test tất định)

Không cần PostgreSQL, Kafka hay Redis; `--verify` và `cargo test` chỉ cần Rust stable. Chạy từ gốc repo:

```bash
cargo run  --manifest-path examples/toi-noi-rust/57-advanced-testing/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/57-advanced-testing/Cargo.toml
```

Công cụ ngoài (`cargo install cargo-mutants cargo-fuzz`; fuzz cần nightly, chỉ riêng bước này):

```bash
cd examples/toi-noi-rust/57-advanced-testing
cargo mutants --file src/split.rs -- --test split_basic   # bộ test đầu: 3 mutant sống
cargo mutants --file src/split.rs                         # sau khi thêm test: 0 mutant sống
mkdir -p fuzz/corpus/parse_line && cargo +nightly fuzz run parse_line fuzz/corpus/parse_line fuzz/seeds -- -max_total_time=60
cargo +nightly fuzz run parse_line_v1 tests/data/crash_v1_char_boundary.bin   # tái hiện crash
```

Lab chứng minh: mutant sống sót chỉ ra test thiếu; input tối thiểu của fuzz thành test hồi quy; snapshot `insta` có redaction cho trường đổi mỗi lần chạy; `HashMap` không sắp thứ tự làm snapshot flaky; clock giả và seed cố định cho test tất định. Parser sao kê là bản nhỏ tự chứa của bài 47 (không dùng `ledger-core`). Không commit `fuzz/corpus`, `fuzz/artifacts`, `target/`.
