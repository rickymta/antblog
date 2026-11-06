#!/usr/bin/env bash
# Kiểm tra phần P04 (LOC-31…LOC-40): fmt, clippy, test của geocode và pickup (test golden của pickup giữ mã băm baseline trên fixture, nên bắt được mọi thay đổi baseline của simulator),
# các lab chạy trên fixture không cần mạng hay bản đồ thật, và Python LOC-39 nếu đã có venv. In PASS nếu tất cả qua.
#
# Chỉ kiểm các crate của P04 bằng `-p` (crate của phần khác, ví dụ matcher của P05 và phần P05 trong simulator, có thể đang dở; nên không chạy fmt/test của `simulator` ở đây).
#
# Cố ý KHÔNG chạy ở đây, nên chạy riêng và ghi vào results/p04.md:
#   - mọi lab trên bản đồ TP.HCM thật (cần data/hcm-graph.bin) và lab geocode trên PBF thật (cần data/vietnam-latest.osm.pbf)
#   - quét trọng số, so sánh cặp 20 seed, độ nhạy giả định, dải seed xác nhận: tốn nhiều phút CPU, và số trên fixture không có giá trị thống kê
#   - đo độ trễ p99 (máy dùng chung, số thời gian trên fixture không so sánh được)
#   - Python LOC-39 trên lịch sử simulator thật (chỉ chạy `--selftest` bằng dữ liệu bịa); `uv sync` cần mạng lần đầu
# Mọi lab dùng dữ liệu thiếu mà không có cờ tường minh sẽ thoát mã 2 kèm thông báo, không bao giờ bỏ qua âm thầm.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
manifest="$here/rust/Cargo.toml"
pkgs=(-p geocode -p pickup)

step() { printf '\n==> %s\n' "$*"; }

command -v cargo >/dev/null || { echo "LỖI: không thấy cargo trên PATH (xem README, mục Môi trường)"; exit 2; }

step "cargo fmt --check (geocode, pickup)"
cargo fmt --manifest-path "$manifest" "${pkgs[@]}" --check

step "rustfmt --check (lab loc31…loc40 và labs/src/p04.rs; không đụng lab của phần khác)"
rustfmt --edition 2024 --check "$here"/rust/labs/src/bin/loc3[1-9]_*.rs "$here"/rust/labs/src/bin/loc40_*.rs "$here/rust/labs/src/p04.rs"

step "cargo clippy (-D warnings, mọi target, geocode và pickup)"
cargo clippy --manifest-path "$manifest" "${pkgs[@]}" --all-targets -- -D warnings

step "cargo clippy (-D warnings, chín lab P04; --no-deps để không bị lỗi của crate khác làm hỏng)"
labs_bins=(loc31_geocode loc32_ham_muc_tieu loc33_sinh_ung_vien loc34_phia_duong loc35_xep_hang loc36_diem_hen_dong
  loc37_hem_cong_sanh loc38_diem_tra loc39_hoc_diem_don loc40_cuoc_quote_tong_hop)
bin_args=()
for b in "${labs_bins[@]}"; do bin_args+=(--bin "$b"); done
cargo clippy --manifest-path "$manifest" -p labs --no-deps "${bin_args[@]}" -- -D warnings

step "cargo test (unit, proptest bất biến «không tệ hơn điểm ghim», baseline giống từng byte, fixture; geocode, pickup)"
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

run loc31_geocode --fixture
expect loc31_geocode 'kiểm tra: NFC và NFD cho cùng khoá chuẩn hoá = true'
expect loc31_geocode 'kiểm tra: độ tin cậy luôn trong \[0, 1\] trên mọi ca thử = true'
expect loc31_geocode 'kiểm tra: mọi gợi ý bắt đầu bằng khoá tiền tố = true'
expect loc31_geocode 'kiểm tra: mọi truy vấn có độ tin cậy ≤ trần của trạng thái hành chính = true'
expect loc31_geocode 'kiểm tra: tên quận/phường số cũ không có trong bảng ánh xạ đều bị gắn OldUnmapped = true'
run loc32_ham_muc_tieu --fixture --jobs 4 --seeds 4 --tuning-seeds 2
expect loc32_ham_muc_tieu 'bất biến «không tệ hơn ghim»: true'
expect loc32_ham_muc_tieu 'CHÍNH SÁCH `p04-v1`'
run loc33_sinh_ung_vien --fixture --n 60
expect loc33_sinh_ung_vien 'ứng viên đầu ra vi phạm ràng buộc cứng .* = 0; ứng viên đi bộ ước lượng quá 250 m = 0'
run loc34_phia_duong --fixture --n 30 --seeds 3 --jobs 3
expect loc34_phia_duong 'top 4 \+ ghim'
expect loc34_phia_duong '\(a\) J đầy đủ'
run loc35_xep_hang --fixture --n 40 --threads 2
expect loc35_xep_hang 'số lần ứng viên tốt nhất có J lớn hơn J điểm ghim = 0'
run loc36_diem_hen_dong --fixture --seeds 3 --per-seed 4 --jobs 3
expect loc36_diem_hen_dong 'tình huống: bình thường'
run loc37_hem_cong_sanh --fixture --n 30
expect loc37_hem_cong_sanh 'nguồn vị trí = Pin'
run loc38_diem_tra --fixture --seeds 3 --jobs 3
expect loc38_diem_tra 'chỉ điểm trả J_trả đầy đủ'
run loc39_hoc_diem_don --fixture --export --history-seeds 3 --jobs 3 --out "$out/loc39"
expect loc39_hoc_diem_don 'đã ghi [0-9]+ dòng'
run loc40_cuoc_quote_tong_hop --fixture --seeds 3 --jobs 3 --skip-sensitivity
expect loc40_cuoc_quote_tong_hop 'KHÔNG TĂNG GIÁ LÉN'

if [ -d "$here/python/loc39/.venv" ]; then
  step "python LOC-39: selftest DBSCAN bằng dữ liệu bịa"
  (cd "$here/python/loc39" && uv run --offline python loc39_dbscan.py --selftest | tee "$out/loc39_py.txt")
  grep -q 'selftest OK' "$out/loc39_py.txt" || { echo "LỖI: selftest Python không in OK"; exit 1; }
else
  printf '\nBỏ qua Python LOC-39: chưa có python/loc39/.venv (chạy `cd python/loc39 && uv sync` một lần).\n'
fi

printf '\nĐã bỏ qua có chủ đích: mọi lab trên bản đồ TP.HCM thật, lab geocode trên PBF thật, quét trọng số, so sánh cặp 20 seed và độ nhạy giả định, đo p99, Python LOC-39 trên lịch sử simulator thật (xem đầu file).\n'
echo "PASS"
