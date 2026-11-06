# Lab 35: gRPC với tonic (GetBalance, PostEntry) và client .NET

Server tonic cho `ledger.v1.Ledger` (`proto/ledger/v1/ledger.proto`): `GetBalance`, `PostEntry` với số tiền `int64 amount_dong`.
Lab chứng minh: sinh code bằng `tonic-prost-build` + `protoc-bin-vendored` (máy không cần `protoc`), map `LedgerError` sang `Status`,
deadline (`grpc-timeout`), interceptor đọc `authorization` và `x-request-id`, và client C# thật (`dotnet-client/`) gọi vào server Rust.

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'   # hạ tầng: infra/README.md
cargo run --manifest-path examples/toi-noi-rust/35-tonic-grpc/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/35-tonic-grpc/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/35-tonic-grpc/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/35-tonic-grpc/Cargo.toml -- --with-dotnet   # cần .NET SDK 10
```

`--verify` và `cargo test` bind `127.0.0.1:0`, dùng schema PostgreSQL `lab_<uuid>` riêng và dọn khi xong. `--with-dotnet` chạy
`dotnet run --project dotnet-client` với địa chỉ server thật, rồi chạy thêm kịch bản deadline từ phía C#; không nằm trong `--verify` vì cần .NET SDK. Thư mục `bin/`, `obj/` của .NET không commit.
