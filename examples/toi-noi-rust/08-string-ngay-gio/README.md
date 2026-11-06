# Bài 08: String, &str, ngày giờ UTC và định dạng tiền

Lab của bài [rust-string-str-ngay-gio-dinh-dang-tien](../../../api/seed/content/rust-string-str-ngay-gio-dinh-dang-tien.md).

```bash
cargo run --manifest-path examples/toi-noi-rust/08-string-ngay-gio/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/08-string-ngay-gio/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/08-string-ngay-gio/Cargo.toml
```

Lab chứng minh: `String` sở hữu vùng heap còn `&str` chỉ mượn; `len()` đếm byte, `chars().count()` đếm ký tự; cắt chuỗi sai biên UTF-8 là panic; `split_once(':')` tách mã tài khoản; `impl Display for Money` in `25.500 VND`; `jiff::Timestamp` lưu UTC và hiển thị `Asia/Ho_Chi_Minh` (+07:00).
Dependency duy nhất: `jiff` 0.2.x (cần dữ liệu múi giờ IANA của hệ điều hành, hoặc bật feature `tzdb-bundle-always`).
