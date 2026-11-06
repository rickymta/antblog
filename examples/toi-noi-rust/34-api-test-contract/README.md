# Lab 34: API test với oneshot, PostgreSQL thật và hợp đồng với client C#

Cần PostgreSQL lab (`TOI_NOI_PG_URL`, xem `../infra/README.md`) và .NET SDK 10 cho phần C#. Chạy từ gốc repo:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run  --manifest-path examples/toi-noi-rust/34-api-test-contract/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/34-api-test-contract/Cargo.toml
cd examples/toi-noi-rust/34-api-test-contract/dotnet-client
dotnet run  --project src/LedgerClient          # đọc contracts/*.json, in CONTRACT OK (C#)
dotnet test tests/LedgerClient.Tests
```

Lab chứng minh: `oneshot` gọi `Router` không cần socket và server thật trên `127.0.0.1:0` khác nhau ở đâu; ca lỗi 401/403/409/422/400/415 trên PostgreSQL thật (schema `lab_<uuid>` mỗi ca, tự dọn cả khi test panic, trừ ca còn transaction mở: `lock_timeout` tránh treo, schema sót có dấu tuổi); snapshot OpenAPI bằng `insta` (`tests/snapshots/`, commit); hợp đồng lỗi cùng dạng bài 32 (`type`, slug, `instance`, `traceId`, `errors`, `/api-docs/openapi.json`); hợp đồng thủ công `contracts/post-entry.{request,response,unbalanced.problem}.json` mà test Rust và client C# cùng đọc. `-- --measure` đo chi phí hai đường vào (nên build `--release`). Không có Pact hay schema registry: đó là bước tiếp theo.
