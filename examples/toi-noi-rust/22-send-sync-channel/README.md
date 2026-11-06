# Lab 22: Send, Sync, Arc<Mutex>, RwLock và channel

Thư viện `ledger_lab` và binary `ledger-cli`; dùng lại `Ledger` của `ledger-core` (lab 20), thêm `tokio` (`rt-multi-thread`, `macros`, `sync`, `time`).

```bash
cargo run --manifest-path examples/toi-noi-rust/22-send-sync-channel/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/22-send-sync-channel/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/22-send-sync-channel/Cargo.toml
cargo run --release --manifest-path examples/toi-noi-rust/22-send-sync-channel/Cargo.toml -- --bench   # đo Mutex với RwLock
```

Chứng minh: `Rc` trong `tokio::spawn` và `MutexGuard` giữ qua `.await` bị compiler từ chối (doc test `compile_fail`); guard của `std::sync::Mutex` qua `.await` trong `join!` trên runtime một luồng thì kẹt thật, bản `tokio::sync::Mutex` thì xong (`src/deadlock.rs`); 50 task ghi cùng `idempotency_key` chỉ ra một bút toán, qua `Arc<Mutex<Ledger>>` lẫn qua actor (`mpsc` bounded + `oneshot`); `broadcast` phát `LedgerEntryPosted` cho nhiều subscriber, subscriber chậm nhận `Lagged`; hàng đợi đầy cho `try_post` lỗi `Full` và `post().await` phải chờ. `--bench` in số đo thật trên máy chạy, không nằm trong `--verify`. Khóa trong process không chống trùng giữa nhiều instance (việc của PostgreSQL, bài 24). Không cần PostgreSQL/Redis. Dữ liệu hư cấu của Tới Nơi.
