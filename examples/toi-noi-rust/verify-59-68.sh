#!/bin/sh
# Chạy toàn bộ lab phần 07 (bài 59–68). Chạy từ gốc repository; cần hạ tầng trong infra/.
set -eu
cd "$(dirname "$0")/../.."
: "${TOI_NOI_PG_URL:?Đặt TOI_NOI_PG_URL theo examples/toi-noi-rust/infra/README.md}"
: "${TOI_NOI_REDIS:?Đặt TOI_NOI_REDIS theo examples/toi-noi-rust/infra/README.md}"
export TOI_NOI_KAFKA="${TOI_NOI_KAFKA:-127.0.0.1:9092}"
for dir in examples/toi-noi-rust/59-* examples/toi-noi-rust/6[0-8]-*; do
  [ -f "$dir/Cargo.toml" ] || continue
  echo "\n> $dir"
  cargo test --quiet --manifest-path "$dir/Cargo.toml"
  cargo run --quiet --manifest-path "$dir/Cargo.toml" -- --verify
done
echo "\nPASS: examples/toi-noi-rust bài 59–68."
