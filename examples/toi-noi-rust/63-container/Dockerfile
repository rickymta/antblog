# Đa stage với cargo-chef. Ba target dùng chung một builder:
#   --target slim        debian:bookworm-slim, có shell và `id`
#   --target slim-shell  cùng image nhưng ENTRYPOINT dạng shell: ví dụ phản mẫu cho SIGTERM
#   (mặc định)           distroless, không shell, user nonroot
FROM lukemathwalker/cargo-chef:latest-rust-1.98-bookworm AS chef
WORKDIR /app

# Stage 1: chỉ đọc Cargo.toml/Cargo.lock/cấu trúc crate, ra recipe.json. Đổi code không đổi recipe.
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: biên dịch DEPENDENCY từ recipe (layer này được cache), rồi mới copy code và build crate.
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --no-default-features --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked --no-default-features --bin ledger-service

# Runtime 1: debian slim. Cần tạo user, vì mặc định là root.
FROM debian:bookworm-slim AS slim
RUN useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin app
COPY --from=builder /app/target/release/ledger-service /app/ledger-service
ENV LEDGER_ADDR=0.0.0.0:8080
USER 10001:10001
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/app/ledger-service", "healthcheck"]
ENTRYPOINT ["/app/ledger-service"]
CMD ["serve"]

# Phản mẫu: ENTRYPOINT dạng shell chạy `/bin/sh -c ...`, sh làm PID 1 và không chuyển SIGTERM cho con.
FROM slim AS slim-shell
ENTRYPOINT /app/ledger-service serve

# Runtime 2 (mặc định): distroless, không shell, không package manager, user nonroot (uid 65532).
FROM gcr.io/distroless/cc-debian12:nonroot AS distroless
COPY --from=builder /app/target/release/ledger-service /app/ledger-service
ENV LEDGER_ADDR=0.0.0.0:8080
USER nonroot:nonroot
EXPOSE 8080
# Không có shell và curl, nên binary tự có subcommand `healthcheck`.
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/app/ledger-service", "healthcheck"]
ENTRYPOINT ["/app/ledger-service"]
CMD ["serve"]
