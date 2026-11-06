# Lab 29: cấu hình, secret và tracing có cấu trúc

Service sổ cái tối giản nạp `Settings` (crate `config`, mật khẩu là `secrecy::SecretString`), validate lúc khởi động
và log JSON bằng `tracing` + `tracing-subscriber`. Dùng lại `ledger-core` của lab 20. Không cần PostgreSQL.

```bash
# Thiếu mật khẩu: thoát mã 2, in mọi vấn đề (fail fast)
cargo run --manifest-path examples/toi-noi-rust/29-config-tracing/Cargo.toml
# Chạy thật, log JSON ra stdout (biến môi trường ghi đè file config/default.toml)
LEDGER__DATABASE__PASSWORD=lab-only-secret LEDGER__DATABASE__MAX_CONNECTIONS=20 \
  cargo run --manifest-path examples/toi-noi-rust/29-config-tracing/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/29-config-tracing/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/29-config-tracing/Cargo.toml
```

Chứng minh: env thắng file; gom nhiều lỗi cấu hình rồi thoát mã 2; mật khẩu không vào `Debug` hay log; `request_id` trong span
đi theo event con và qua `tokio::spawn` khi có `.instrument(Span::current())` (mất khi quên); giữ `span.enter()` qua `.await` gán nhầm
ngữ cảnh; `#[instrument]` trần làm lọt PII còn `skip` thì không. Test đọc log từ bộ đệm.
