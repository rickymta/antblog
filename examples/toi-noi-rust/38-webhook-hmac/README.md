# Lab 38: webhook VíXanh, chữ ký HMAC, chống replay, xử lý trễ và trùng

Chứng minh bằng chạy thật: endpoint axum `POST /webhooks/vixanh` đọc body thô (`Bytes`) rồi mới parse; header giả định của lab `X-ViXanh-Signature: t=<epoch>,v1=<hex>` ký `t.body` bằng HMAC-SHA256 (`hmac`, `sha2`), so thẻ bằng `verify_slice` (constant-time); cửa sổ thời gian 300 giây với đồng hồ cố định; `external_ref` là khóa chính nên webhook trùng chỉ xử lý một lần; webhook đến trước/sau event `TripCompleted` (lưu `pending`, trả 202); 5xx khi DB lỗi và VíXanh thử lại; hai khóa hợp lệ cùng lúc khi xoay vòng. Khóa và định dạng header là giá trị lab, không phải của PSP thật.

Cần PostgreSQL của `examples/toi-noi-rust/infra`. Từ gốc repository:

```bash
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run --manifest-path examples/toi-noi-rust/38-webhook-hmac/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/38-webhook-hmac/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/38-webhook-hmac/Cargo.toml
```

Mỗi thí nghiệm chạy trong schema `lab_<uuid>` riêng và server bind `127.0.0.1:0`, nên chạy lặp và song song được. Dùng lại `ledger-core` của lab 20 (đường dẫn tương đối). Log từ chối đi ra stderr; thêm `2>/dev/null` để đọc output sạch.
