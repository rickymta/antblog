#!/usr/bin/env bash
# Kiểm tra phần P05 (LOC-41…LOC-50): fmt, clippy, test của matcher, simulator, geo-engine và các lab chạy không cần bản đồ thật, không cần mạng.
# In PASS nếu tất cả qua.
#
# Chỉ kiểm các crate của P05 bằng `-p` (không chạm crate của phần khác trong workspace, vì crate phần khác có thể đang dở).
# Các biến môi trường: SKIP_DOTNET=1 bỏ bước contract test C# (LOC-46) nếu máy không có .NET SDK.
#
# Cố ý KHÔNG chạy ở đây, nên chạy riêng và ghi vào results/p05.md:
#   - mọi lab trên bản đồ TP.HCM thật (cần data/hcm-graph.bin do lab LOC-06 biên dịch) với 20 seed đánh giá: mỗi lab mất vài phút đến hàng chục phút
#   - đo độ trễ (SuggestMatch trên HCM, ngân sách ghép lô): số thời gian ở đây không có giá trị so sánh
#   - chạy lại OR-Tools của LOC-50 (python/loc50, ~7 phút) và sinh lại fixtures/loc50/*.json; ở đây chỉ kiểm môi trường uv + ortools nạp được ngoại tuyến
#   - các số «tốt hơn / xấu hơn» của bài: verify chỉ kiểm bất biến (hợp lệ, tất định, Hungarian ≤ tham lam, cắt tỉa ≥ tối ưu, thứ tự lấy-trước-giao), không kiểm kết luận thống kê
# Mọi lab thiếu dữ liệu mà không có cờ tường minh sẽ thoát mã 2 kèm thông báo, không bao giờ bỏ qua âm thầm.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
manifest="$here/rust/Cargo.toml"
pkgs=(-p matcher -p simulator -p geo-engine)

step() { printf '\n==> %s\n' "$*"; }
command -v cargo >/dev/null || { echo "LỖI: không thấy cargo trên PATH (xem README, mục Môi trường)"; exit 2; }

step "cargo fmt --check (matcher, simulator)"
cargo fmt --manifest-path "$manifest" -p matcher -p simulator --check

step "rustfmt --check (geo-engine/src/matching.rs, labs/src/p05.rs, lab loc41…loc50; không đụng file của phần khác)"
rustfmt --edition 2024 --check "$here/rust/crates/geo-engine/src/matching.rs" "$here/rust/labs/src/p05.rs" \
  "$here"/rust/labs/src/bin/loc4[1-9]_*.rs "$here"/rust/labs/src/bin/loc50_*.rs

step "cargo clippy (-D warnings, mọi target, matcher + simulator + geo-engine)"
cargo clippy --manifest-path "$manifest" "${pkgs[@]}" --all-targets -- -D warnings

step "cargo clippy (mười lab của P05: không được có cảnh báo nào nằm trong file lab của P05)"
labs_bins=(loc41_gan_nhat_sai_o_dau loc42_ghep_theo_lo_hungarian loc43_cua_so_lo loc44_cat_tia_h3_ma_tran_thua
  loc45_chi_phi_cong_bang_rui_ro_huy loc46_goi_y_sang_dispatch loc47_chain_dispatch loc48_lech_tuyen_dinh_tuyen_lai
  loc49_tai_xe_ranh_di_dau loc50_nhieu_diem_dung)
bin_args=()
for b in "${labs_bins[@]}"; do bin_args+=(--bin "$b"); done
# Không dùng `-D warnings` ở đây: lệnh đó cũng áp lên crate `pickup` và thư viện labs/p04 của P04 (đang dở, ngoài phạm vi P05) và làm hỏng bước này.
# Thay vào đó lọc cảnh báo theo đường dẫn: file lab và p05.rs của P05 phải sạch.
clippy_out="$(mktemp)"
cargo clippy --manifest-path "$manifest" -p labs "${bin_args[@]}" 2> "$clippy_out" || { cat "$clippy_out"; echo "LỖI: clippy không biên dịch được labs"; exit 1; }
if grep -E -- '--> .*labs/src/(bin/loc4[1-9]_|bin/loc50_|p05\.rs)' "$clippy_out"; then
  grep -E -B6 -A6 -- '--> .*labs/src/(bin/loc4[1-9]_|bin/loc50_|p05\.rs)' "$clippy_out"
  echo "LỖI: clippy có cảnh báo trong lab P05"; exit 1
fi
rm -f "$clippy_out"
echo "ok (không có cảnh báo trong lab P05)"

step "cargo test (unit, proptest: hợp lệ, Hungarian ≤ tham lam, cắt tỉa ≥ tối ưu, lấy trước giao sau; baseline simulator byte-identical; gRPC SuggestMatch trong tiến trình)"
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

run loc41_gan_nhat_sai_o_dau --fixture --seeds 2 --trials 100 --jobs 2
expect loc41_gan_nhat_sai_o_dau 'FNV-1a = 1e0c1a5b30a19582'   # baseline của P03 trên fixture, ghi trước khi thêm điểm cắm P05
expect loc41_gan_nhat_sai_o_dau 'ví dụ cụ thể'
run loc42_ghep_theo_lo_hungarian --fixture --seeds 2 --batches 30 --jobs 2
expect loc42_ghep_theo_lo_hungarian 'số lần Hungarian khác vét cạn = 0'
run loc42_ghep_theo_lo_hungarian --fixture --bo-roi --seeds 2 --jobs 2
expect loc42_ghep_theo_lo_hungarian 'từng bị khách đặt sau vượt' 2
run loc43_cua_so_lo --fixture --seeds 2 --tuning-seeds 2 --jobs 2
expect loc43_cua_so_lo 'chọn theo tiêu chí đặt trước'
run loc44_cat_tia_h3_ma_tran_thua --fixture --seeds 2 --instances 2 --jobs 2
expect loc44_cat_tia_h3_ma_tran_thua 'cắt tỉa r8, k-ring 2, 8 gần nhất|H3 r8, k-ring 2, 8 gần nhất' 2
run loc45_chi_phi_cong_bang_rui_ro_huy --fixture --seeds 2 --jobs 2
expect loc45_chi_phi_cong_bang_rui_ro_huy 'v1-all'
run loc45_chi_phi_cong_bang_rui_ro_huy --fixture --hinh-dang-lo --thu-lo --variant 'thường' --seeds 2 --jobs 2
expect loc45_chi_phi_cong_bang_rui_ro_huy 'n ≥ 2 khách × m ≥ 2 xe: 0;'   # ghép ngay không bao giờ có lô nhiều khách nhiều xe
if command -v dotnet >/dev/null && [ "${SKIP_DOTNET:-0}" != "1" ]; then
  run loc46_goi_y_sang_dispatch --fixture
  expect loc46_goi_y_sang_dispatch 'KẾT QUẢ: [0-9]+ PASS, 0 FAIL'
else
  step "lab: loc46_goi_y_sang_dispatch (--no-dotnet: không có dotnet hoặc SKIP_DOTNET=1; chỉ kiểm máy chủ Rust trong tiến trình)"
  cargo run --manifest-path "$manifest" --release -q -p labs --bin loc46_goi_y_sang_dispatch -- --fixture --no-dotnet > "$out/loc46.txt"
  echo "ok (contract test C# ĐÃ BỎ QUA)"
fi
run loc47_chain_dispatch --fixture --seeds 2 --jobs 2
expect loc47_chain_dispatch 'không chain \(tuyệt đối'
run loc48_lech_tuyen_dinh_tuyen_lai --fixture --tune 10 --eval 20 --hard 5
run loc49_tai_xe_ranh_di_dau --fixture --seeds 2 --jobs 2
expect loc49_tai_xe_ranh_di_dau 'Pearson'
run loc50_nhieu_diem_dung --fixture
expect loc50_nhieu_diem_dung 'n = [0-9]+ cặp'

step "python/loc50: uv + Python 3.13 + ortools nạp được ngoại tuyến (không giải lại, ~7 phút)"
if command -v uv >/dev/null; then
  (cd "$here/python/loc50" && uv run --offline --python 3.13 python -c "import ortools, sys; print('ortools', ortools.__version__, 'python', sys.version.split()[0])")
else
  echo "BỎ QUA: không thấy uv"
fi

printf '\nĐã bỏ qua có chủ đích: lab trên bản đồ TP.HCM với 20 seed đánh giá, đo độ trễ, chạy lại OR-Tools (xem đầu file).\n'
echo "PASS"
