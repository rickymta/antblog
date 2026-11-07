#!/usr/bin/env bash
# Kiểm tra phần P01 (LOC-03…LOC-10): fmt, clippy, test, và mọi lab không cần PBF lẫn Redis. In PASS nếu tất cả qua.
#
# Cố ý KHÔNG chạy ở đây (cần dữ liệu hoặc hạ tầng ngoài, nên chạy riêng và ghi vào results/p01.md):
#   - loc06_osm_do_thi   cần data/vietnam-latest.osm.pbf (data/fetch.sh); ở đây chỉ chạy với --fixture
#   - loc04_khoang_cach  phần hệ số vòng đường dùng --fixture ở đây (bản HCM cần data/hcm-graph.bin)
#   - loc07_snap         chạy với --fixture ở đây (bản HCM cần data/hcm-graph.bin)
#   - loc09_gan_nhat     chạy với --no-redis ở đây (Redis cần docker compose ... up -d --wait redis)
#   - cargo bench        criterion mất vài phút; chạy riêng
# Mọi lab dùng dữ liệu thiếu mà không có cờ tường minh sẽ thoát mã 2 kèm thông báo, không bao giờ bỏ qua âm thầm.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
manifest="$here/rust/Cargo.toml"

step() { printf '\n==> %s\n' "$*"; }

command -v cargo >/dev/null || { echo "LỖI: không thấy cargo trên PATH (xem README, mục Môi trường)"; exit 2; }

step "cargo fmt --check"
cargo fmt --manifest-path "$manifest" --all --check

step "cargo clippy (-D warnings, mọi target)"
cargo clippy --manifest-path "$manifest" --workspace --all-targets -- -D warnings

step "cargo test (unit, proptest, doctest)"
cargo test --manifest-path "$manifest" --workspace

step "fixture sinh lại khớp từng byte"
fx="$here/fixtures/khu-mau.json"
before="$(shasum -a 256 "$fx" | cut -d' ' -f1)"
cargo run --manifest-path "$manifest" --release -q -p labs --bin gen_khu_mau >/dev/null
after="$(shasum -a 256 "$fx" | cut -d' ' -f1)"
if [ "$before" != "$after" ]; then
  echo "LỖI: fixtures/khu-mau.json khác bản sinh lại từ mã nguồn (đã ghi đè; xem git diff)"; exit 1
fi
echo "ok ($after)"

run() {
  step "lab: $*"
  cargo run --manifest-path "$manifest" --release -q -p labs --bin "$@" > /dev/null
  echo "ok"
}

run loc03_toa_do
run loc04_khoang_cach -- --fixture
run loc05_hinh_hoc
run loc06_osm_do_thi -- --fixture
run loc07_snap -- --fixture --queries 2000 --scan-queries 200
run loc08_h3
run loc09_gan_nhat -- --no-redis --scales 2500 --seconds 2
run loc10_tong_hop

printf '\nĐã bỏ qua có chủ đích: loc06 trên PBF thật, loc04/loc07 trên đồ thị HCM, loc09 với Redis, cargo bench (xem đầu file).\n'
echo "PASS"
