#!/usr/bin/env bash
# Kiểm tra phần P03 (LOC-21…LOC-30): fmt, clippy, test của route-engine, simulator, geo-engine và các lab chạy không cần bản đồ thật. In PASS nếu tất cả qua.
#
# Chỉ kiểm các crate của P03 bằng `-p` (không chạm crate khác trong workspace, vì crate của phần khác có thể đang dở).
#
# Cố ý KHÔNG chạy ở đây, nên chạy riêng và ghi vào results/p03.md:
#   - mọi lab trên bản đồ TP.HCM thật (cần data/hcm-graph.bin, do lab LOC-06 biên dịch từ data/vietnam-latest.osm.pbf)
#   - client C# (dotnet/GeoEngineClient): cần .NET SDK, NuGet (mạng lần đầu) và một máy chủ geo-engine đang chạy
#   - binary máy chủ `geo-engine` (chỉ test trong tiến trình qua cổng loopback, không phơi cổng ra ngoài)
#   - đo hiệu năng nghiêm túc: số thời gian ở đây (fixture nhỏ, máy dùng chung) không có giá trị so sánh
# Test gRPC của geo-engine chạy máy chủ và client trong cùng tiến trình qua 127.0.0.1 (không cần mạng ngoài).
# Mọi lab dùng dữ liệu thiếu mà không có cờ tường minh sẽ thoát mã 2 kèm thông báo, không bao giờ bỏ qua âm thầm.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
manifest="$here/rust/Cargo.toml"
pkgs=(-p route-engine -p simulator -p geo-engine)

step() { printf '\n==> %s\n' "$*"; }

command -v cargo >/dev/null || { echo "LỖI: không thấy cargo trên PATH (xem README, mục Môi trường)"; exit 2; }

step "cargo fmt --check (route-engine, simulator, geo-engine)"
cargo fmt --manifest-path "$manifest" "${pkgs[@]}" --check

step "rustfmt --check (lab loc21…loc30 và labs/src/p03.rs; không đụng lab của phần khác)"
rustfmt --edition 2024 --check "$here"/rust/labs/src/bin/loc2[1-9]_*.rs "$here"/rust/labs/src/bin/loc30_*.rs "$here/rust/labs/src/p03.rs"

step "cargo clippy (-D warnings, mọi target, ba crate của P03)"
cargo clippy --manifest-path "$manifest" "${pkgs[@]}" --all-targets -- -D warnings

step "cargo clippy (-D warnings, mười lab của P03)"
labs_bins=(loc21_dijkstra_astar loc22_hai_chieu_alt loc23_re_u_turn loc24_profile loc25_contraction_hierarchies
  loc26_ma_tran loc27_toc_do_theo_gio loc28_routing_provider loc29_dich_vu_dinh_tuyen loc30_mo_phong)
bin_args=()
for b in "${labs_bins[@]}"; do bin_args+=(--bin "$b"); done
cargo clippy --manifest-path "$manifest" -p labs "${bin_args[@]}" -- -D warnings

step "cargo test (unit, proptest so với Dijkstra, tất định của mô phỏng, gRPC trong tiến trình)"
cargo test --manifest-path "$manifest" "${pkgs[@]}"

out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT
run() { # run <bin> [args...]: chạy lab trên fixture, lưu output, thoát nếu lỗi
  local bin="$1"; shift
  step "lab: $bin $*"
  cargo run --manifest-path "$manifest" --release -q -p labs --bin "$bin" -- "$@" > "$out/$bin.txt"
  echo "ok ($(wc -l < "$out/$bin.txt") dòng)"
}
expect() { # expect <bin> <mẫu grep -E> [số lần tối thiểu]
  local n; n="$(grep -cE "$2" "$out/$1.txt" || true)"
  if [ "$n" -lt "${3:-1}" ]; then echo "LỖI: output của $1 thiếu dòng khớp «$2» (có $n, cần ≥ ${3:-1})"; exit 1; fi
}

run loc21_dijkstra_astar --fixture --queries 100
expect loc21_dijkstra_astar 'chi phí khác Dijkstra CSR: Vec<Vec>: 0, petgraph dijkstra: 0, petgraph astar: 0'
expect loc21_dijkstra_astar 'số cạnh vi phạm h > w = 0' 2
run loc22_hai_chieu_alt --fixture --queries 100
expect loc22_hai_chieu_alt 'dừng đúng khác Dijkstra: 0/'
expect loc22_hai_chieu_alt 'hai chiều, dừng ở nút gặp đầu tiên: chi phí 13'
run loc23_re_u_turn --fixture --queries 100
expect loc23_re_u_turn 'RẼ VÀO CHỖ BỊ CẤM' 2
run loc24_profile --fixture --queries 100
run loc25_contraction_hierarchies --fixture --queries 400
expect loc25_contraction_hierarchies 'CH của lab so với Dijkstra: 0 khác biệt'
expect loc25_contraction_hierarchies 'CH của lab so với fast_paths: 0 khác biệt'
run loc26_ma_tran --fixture --repeats 3
expect loc26_ma_tran 'tổng = 0'
expect loc26_ma_tran '0 ô vi phạm'
run loc27_toc_do_theo_gio --fixture --queries 100
expect loc27_toc_do_theo_gio 'CCH tuỳ biến so với Dijkstra: 0 khác biệt' 4
run loc28_routing_provider --fixture --queries 100
expect loc28_routing_provider 'cặp vi phạm hợp đồng: 0/50' 3
run loc29_dich_vu_dinh_tuyen --fixture --requests 300 --clients 16
expect loc29_dich_vu_dinh_tuyen 'Route lạnh'
run loc30_mo_phong --fixture --seeds 1
expect loc30_mo_phong 'báo cáo giống từng byte: true; nhật ký chuyến .* giống từng byte: true; nhật ký sự kiện .* giống từng byte: true'

printf '\nĐã bỏ qua có chủ đích: lab trên bản đồ TP.HCM thật, client C# (cần máy chủ và NuGet), binary máy chủ geo-engine, đo hiệu năng (xem đầu file).\n'
echo "PASS"
