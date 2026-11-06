#!/bin/sh
# Chạy toàn bộ lab phần 01 (bài 01–10). Chạy từ gốc repository.
set -eu
cd "$(dirname "$0")/../.."
for dir in examples/toi-noi-rust/0[1-9]-* examples/toi-noi-rust/10-*; do
  [ -f "$dir/Cargo.toml" ] || continue
  echo "\n> $dir"
  if grep -q '^\[\[test\]\]\|#\[test\]' -r "$dir/src" "$dir/tests" 2>/dev/null; then
    cargo test --quiet --manifest-path "$dir/Cargo.toml"
  fi
  if grep -q -- '--verify' "$dir/src/main.rs" 2>/dev/null; then
    cargo run --quiet --manifest-path "$dir/Cargo.toml" -- --verify
  fi
done
echo "\nPASS: examples/toi-noi-rust bài 01–10."
