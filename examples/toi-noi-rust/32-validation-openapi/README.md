# Lab 32: validation, Problem Details và OpenAPI

Hợp đồng HTTP `/v1` của API sổ cái: validate thủ công gom lỗi, lỗi `application/problem+json` (RFC 9457), ánh xạ `LedgerError` sang HTTP và tài liệu OpenAPI 3.1 sinh bằng `utoipa`. Sổ cái là adapter trong bộ nhớ (`ledger-core` của lab 20), nên không cần PostgreSQL, Redis hay Kafka.

```bash
cargo run --manifest-path examples/toi-noi-rust/32-validation-openapi/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/32-validation-openapi/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/32-validation-openapi/Cargo.toml
```

Chứng minh: lệch trả 422, xung đột key trả 409, JSON hỏng trả 400 dạng Problem Details qua extractor `ApiJson`, `errors` liệt kê đủ mọi trường sai, tài liệu `/api-docs/openapi.json` có đủ đường dẫn và Swagger UI chạy. Server bind `127.0.0.1:0`; `--verify` và `cargo test` chạy song song được.
