#!/bin/sh
# Chạy toàn bộ lab phần 02 (bài 11–20). Chạy từ gốc repository.
set -eu
cd "$(dirname "$0")/../.."
for dir in examples/toi-noi-rust/1[1-9]-* examples/toi-noi-rust/20-*; do
  [ -f "$dir/Cargo.toml" ] || continue
  echo "\n> $dir"
  cargo test --quiet --manifest-path "$dir/Cargo.toml"
  cargo run --quiet --manifest-path "$dir/Cargo.toml" -- --verify
done
echo "\nPASS: examples/toi-noi-rust bài 11–20."
