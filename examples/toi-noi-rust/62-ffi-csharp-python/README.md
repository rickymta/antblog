# Lab 62: unsafe và FFI, gọi thư viện tính phí từ C# (P/Invoke) và Python (PyO3)

Workspace ba crate quanh `ledger-core` (lab 20, không sửa): `fee-ffi` là `cdylib` xuất hàm `extern "C"` (`#[unsafe(no_mangle)]`, struct `#[repr(C)]`, header do `cbindgen` sinh); `fee-py` là module Python bằng PyO3; gốc workspace là binary `--verify`. Client C# thật ở `dotnet-client/` (`LibraryImport`, `SafeHandle`), client Python ở `python-client/`.

```bash
cargo run  --manifest-path examples/toi-noi-rust/62-ffi-csharp-python/Cargo.toml                  # kể lại từng bước
cargo run  --manifest-path examples/toi-noi-rust/62-ffi-csharp-python/Cargo.toml -- --verify      # chỉ cần Rust, in VERIFY OK
cargo test --manifest-path examples/toi-noi-rust/62-ffi-csharp-python/Cargo.toml -p fee-ffi       # 10 test, chạy được dưới Miri
cargo run  --manifest-path examples/toi-noi-rust/62-ffi-csharp-python/Cargo.toml -- --with-dotnet # cần .NET SDK 10
cargo run  --manifest-path examples/toi-noi-rust/62-ffi-csharp-python/Cargo.toml -- --with-python # tạo venv, pip install maturin (cần mạng), maturin develop
```

`--verify` kiểm chữ ký và bố cục struct, số liệu `TN-0001` (25.500 / 5.100 / 20.400), mã lỗi thay vì panic, vòng đời handle (double free và dùng sau free trả `BadHandle`; tham số con trỏ chỉ dùng để tra địa chỉ), `catch_unwind`, tiến trình con abort khi `extern "C"` panic không được chặn, header cbindgen khớp `fee-ffi/include/fee_ffi.h` và symbol xuất ra bằng `nm` (máy không có `nm` thì in dòng BỎ QUA). Các cờ `--with-*` thoát mã khác 0 khi lỗi. Venv của `--with-python` mặc định nằm trong `target/`; đặt `FEE_PY_VENV` để dùng chỗ khác. Không cài gì vào hệ thống; `bin/`, `obj/`, `target/`, `.so`, `.dylib` không commit.
