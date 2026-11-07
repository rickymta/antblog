#!/usr/bin/env bash
# Kiểm tra phần Rust của P02 (LOC-15, 16, 17, 18): fmt, clippy, test, và các lab chạy offline trên đồ thị fixture. In PASS nếu tất cả qua.
#
# Chỉ kiểm các crate của phần này (`-p map-match -p location-store` và các binary loc15…loc18 của `labs`), vì crate của việc khác
# (route-engine, simulator, geo-engine) có thể đang viết dở. `clippy --no-deps` để không lint chúng.
#
# Cố ý KHÔNG chạy ở đây (cần dữ liệu, hạ tầng hoặc chạy lâu), lệnh nằm trong results/p02-rust.md:
#   - lab trên đồ thị HCM thật (data/hcm-graph.bin, 126.699 nút): ở đây dùng --fixture với số chuyến nhỏ
#   - test tích hợp Kafka:   TOI_NOI_KAFKA=1 cargo test -p location-store --test kafka     (cần: docker compose ... up -d --wait kafka)
#   - test tích hợp Redis:   TOI_NOI_REDIS=1 cargo test -p location-store --test redis     (cần: docker compose ... up -d --wait redis)
#   - loc18 với Redis và Kafka thật: ... --bin loc18_kho_vi_tri -- --redis --kafka (lab chạy ~1–2 phút ở 25.000 tài xế)
#   - fmt/clippy/test toàn workspace (gồm crate của việc khác) và cargo bench
# Không có TOI_NOI_KAFKA=1 hay TOI_NOI_REDIS=1 thì hai test tích hợp tự in «bỏ qua» và qua; đặt biến mà hạ tầng không chạy thì thất bại.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
manifest="$here/rust/Cargo.toml"
labs_bins=(loc15_kalman loc16_hmm loc17_online loc18_kho_vi_tri)
lab_files=(labs/src/p02.rs labs/src/bin/loc15_kalman.rs labs/src/bin/loc16_hmm.rs labs/src/bin/loc17_online.rs labs/src/bin/loc18_kho_vi_tri.rs)

step() { printf '\n==> %s\n' "$*"; }
fail() { echo "FAIL: $*" >&2; exit 1; }

command -v cargo >/dev/null || { echo "LỖI: không thấy cargo trên PATH (xem README, mục Môi trường)"; exit 2; }

step "cargo fmt --check (map-match, location-store)"
cargo fmt --manifest-path "$manifest" -p map-match -p location-store --check

step "rustfmt --check (các lab P02)"
(cd "$here/rust" && rustfmt --edition 2024 --check "${lab_files[@]}")

step "cargo clippy -D warnings (map-match, location-store, mọi target)"
cargo clippy --manifest-path "$manifest" --no-deps -p map-match -p location-store --all-targets -- -D warnings

step "cargo clippy -D warnings (labs: lib và loc15…loc18)"
bin_args=(); for b in "${labs_bins[@]}"; do bin_args+=(--bin "$b"); done
cargo clippy --manifest-path "$manifest" --no-deps -p labs --lib "${bin_args[@]}" -- -D warnings

step "cargo test (unit, proptest, test ca khó trên fixture, doctest; test Kafka/Redis tự bỏ qua nếu không đặt biến)"
cargo test --manifest-path "$manifest" -p map-match -p location-store

run_lab() {
  local name="$1" marker="$2"; shift 2
  step "lab: $name $*"
  local out
  out="$(cargo run --manifest-path "$manifest" --release -q -p labs --bin "$name" -- "$@")" || fail "$name thoát với mã lỗi"
  grep -q -- "$marker" <<<"$out" || fail "$name: thiếu dòng «$marker» trong output"
  echo "ok"
}

run_lab loc15_kalman "chọn: accel_sigma" --fixture --trips 6
run_lab loc16_hmm "HMM + Viterbi offline" --fixture --trips 6
run_lab loc17_online "online, trễ 3 điểm" --fixture --trips 6 --per-kind 3 --drivers 50
run_lab loc18_kho_vi_tri "0/200 truy vấn khác kết quả" --fixture --drivers 3000 --seconds 1

cat <<'EOT'

Đã bỏ qua có chủ đích: lab trên đồ thị HCM thật (chỉ chạy --fixture, số chuyến nhỏ); test tích hợp Kafka (TOI_NOI_KAFKA=1) và Redis
(TOI_NOI_REDIS=1); loc18 với --redis --kafka; fmt/clippy/test toàn workspace và crate của việc khác (route-engine, simulator, geo-engine); cargo bench.
Số trong bài lấy từ results/p02-rust.md, không lấy từ lần chạy này.
EOT
echo "PASS"
