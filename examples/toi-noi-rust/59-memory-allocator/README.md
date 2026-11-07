# Lab 59: bộ nhớ không GC, allocator và so với .NET

Dùng lại `ledger-core` (lab 20) làm khối lượng việc: ghi 10.000 bút toán. Chứng minh: `Drop` tất định và RAII (thứ tự drop, giao dịch rollback kể cả khi panic, `MutexGuard`), đếm allocation bằng global allocator tự viết (`Box`, `Vec` tăng trưởng, `String`, 90.026 block cho 10.000 bút toán), `Cow`, `'static`, HRTB, struct tự tham chiếu và `Pin`. `dotnet-compare/` chạy cùng khối lượng việc bằng C# để so allocated bytes, số lần GC và RSS.

```bash
cargo run --manifest-path examples/toi-noi-rust/59-memory-allocator/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/59-memory-allocator/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/59-memory-allocator/Cargo.toml
cargo run --release --manifest-path examples/toi-noi-rust/59-memory-allocator/Cargo.toml -- --workload
cargo run --release --bin alloc-bench --manifest-path examples/toi-noi-rust/59-memory-allocator/Cargo.toml -- --threads 4 --rounds 20   # thêm --features mimalloc hoặc jemalloc
dotnet run --project examples/toi-noi-rust/59-memory-allocator/dotnet-compare -c Release -- --rounds 20
```
`--verify` kiểm số cấp phát (không kiểm thời gian) và in `VERIFY OK`. Số đo thời gian và RSS chỉ có ý nghĩa trên máy đã đo, không phải SLA; máy dùng chung nên nhiễu. RSS đỉnh đo bằng `/usr/bin/time -l` trên binary đã build (`target/dotnet-compare/bin/LedgerMemory/release/LedgerMemory`), không qua `dotnet run`. Cả `mimalloc` và `jemalloc` đều biên dịch C; `jemalloc` chậm hơn rõ rệt. Mọi số đo là macOS. Dependency: `ledger-core` (path), `mimalloc`, `tikv-jemallocator` (cả hai optional theo feature). Dữ liệu hư cấu của Tới Nơi.
