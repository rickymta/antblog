# Lab 63: đóng gói `ledger-service` vào container

Binary `ledger-service` nhỏ (axum: `/healthz`, `/readyz`, `GET /v1/fare?km=5`; PostgreSQL tuỳ chọn qua `LEDGER_PG_URL`) và bốn Dockerfile để so: `Dockerfile.single` (một stage, mốc phản mẫu), `Dockerfile` (cargo-chef; target `slim`, `slim-shell`, mặc định `distroless`), `Dockerfile.musl` (alpine + scratch), `Dockerfile.size` (đo núm LTO/strip). Không phụ thuộc lab khác.

```bash
cargo run  --manifest-path examples/toi-noi-rust/63-container/Cargo.toml                 # kể lại từng bước
cargo run  --manifest-path examples/toi-noi-rust/63-container/Cargo.toml -- --verify     # không cần Docker, in VERIFY OK
cargo test --manifest-path examples/toi-noi-rust/63-container/Cargo.toml
# Phần Docker: build ~10 image/stage, chạy container tạm, đo kích thước, cache, SIGTERM (vài chục phút khi máy tải)
export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'
cargo run  --manifest-path examples/toi-noi-rust/63-container/Cargo.toml -- --verify --with-docker
```

Chứng minh: image một stage lớn hơn và chạy root; cargo-chef giữ lớp `cook` khi chỉ đổi một dòng code; context quên `.dockerignore` nặng cỡ nào; `USER` không-root; `HEALTHCHECK` bằng subcommand của binary (distroless không có shell); ENTRYPOINT dạng exec thoát mã 0, dạng shell bị SIGKILL (137); `/readyz` 503 khi DB chết nhưng `/healthz` vẫn 200.
Kích thước là `Size` của `docker image inspect`; với containerd image store đó là tổng layer đã nén, không phải dung lượng trên đĩa. Image và container của lab có tiền tố `lab-63-`, bị xoá khi xong kể cả khi lỗi. Lab không `docker system prune` nên cache build của BuildKit còn lại (xem `docker buildx du`). Chưa làm: build đa kiến trúc, registry, quét lỗ hổng, ký image.
