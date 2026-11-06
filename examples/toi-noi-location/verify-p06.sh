#!/usr/bin/env bash
# Kiểm tra toàn phần P06 (LOC-51 … LOC-56): pytest, test và lint Rust của eta-model và simulator (bằng -p), rồi chạy cả sáu bài trên
# dữ liệu NHỎ sinh từ đồ thị tổng hợp (fixture). Không dùng mạng ngoài khi chạy (uv cần môi trường đã đồng bộ), không cần data/hcm-graph.bin.
#
# Chạy từ gốc repo:   bash examples/toi-noi-location/verify-p06.sh
#
# KHÔNG chạy (chỉ có trong results/p06.md):
#   - sinh bộ dữ liệu đầy đủ trên bản đồ TP.HCM (21 ngày × 24 giờ, 7,6 triệu dòng GPS) và các số MAE, hiệu chuẩn, dự báo ghi trong bài;
#   - chạy lại LOC-49 và LOC-56 trên HCM với 20–40 seed đánh giá (ở đây chỉ vài seed trên fixture để thử đường ống);
#   - đo p99 và bộ nhớ của tract và ort trên mô hình thật (ở đây chỉ vài trăm lần gọi trên mô hình fixture);
#   - các lần chạy độ nhạy theo gap.
# Với dữ liệu fixture, số MAE hay độ phủ KHÔNG có ý nghĩa; script chỉ kiểm đường ống chạy hết và các bất biến (không rò rỉ, so số học, fallback, sự thật toàn cục).

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
LAB="$ROOT/examples/toi-noi-location"
PYDIR="$LAB/python/p06"
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
MANIFEST="$LAB/rust/Cargo.toml"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
step() { echo; echo "==> $*"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
py() { (cd "$PYDIR" && uv run python "$@"); }
cargo_labs() { cargo "$1" --manifest-path "$MANIFEST" -p labs "${@:2}"; }

ORT_LIB="$(ls "$PYDIR"/.venv/lib/python3.13/site-packages/onnxruntime/capi/libonnxruntime*.dylib 2>/dev/null | head -1 || true)"
if [ -n "$ORT_LIB" ]; then export ORT_DYLIB_PATH="$ORT_LIB"; echo "ort dùng thư viện của gói Python onnxruntime: $ORT_LIB"; else echo "CẢNH BÁO: không thấy libonnxruntime trong .venv, các bước ort sẽ báo «chưa kiểm»"; fi

step "uv sync (môi trường Python 3.13 của riêng P06)"
(cd "$PYDIR" && uv sync --quiet)

step "pytest (dữ liệu thử nhỏ, không mạng)"
(cd "$PYDIR" && uv run pytest -q)

step "cargo fmt --check (eta-model, simulator) và rustfmt --check (các lab P06)"
cargo fmt --manifest-path "$MANIFEST" -p eta-model -p simulator --check
for f in p06.rs bin/loc51_xuat_du_lieu.rs bin/loc53_so_sanh_tract_ort.rs bin/loc53_fallback.rs bin/loc55_tai_xe_theo_du_bao.rs bin/loc56_ab_switchback.rs; do
  rustfmt --edition 2024 --check "$LAB/rust/labs/src/$f"
done

step "cargo clippy -D warnings (eta-model, simulator bằng -p; các lab P06)"
cargo clippy --manifest-path "$MANIFEST" -p eta-model -p simulator --all-targets -- -D warnings
# `-D warnings` áp cho cả crate nội bộ mà labs phụ thuộc (của phần khác, có thể đang viết dở), nên ở đây chỉ coi cảnh báo nằm trong file của P06 là lỗi
cargo_labs clippy --bin loc51_xuat_du_lieu --bin loc53_so_sanh_tract_ort --bin loc53_fallback --bin loc55_tai_xe_theo_du_bao --bin loc56_ab_switchback 2>&1 | tee "$TMP/clippy-labs.txt" | grep -E "^(warning|error)" | head -5 || true
if grep -E -- "--> .*labs/src/(p06\.rs|bin/loc5[1356]_)" "$TMP/clippy-labs.txt"; then fail "clippy báo cảnh báo trong file lab P06"; fi

step "cargo test -p eta-model -p simulator (gồm: baseline simulator giữ nguyên từng byte, điểm cắm Truth/Observer)"
cargo test --manifest-path "$MANIFEST" -p eta-model -p simulator 2>&1 | tee "$TMP/cargo-test.txt" | grep -E "^test result|Running|BỎ QUA|FAILED" || true
grep -q "FAILED" "$TMP/cargo-test.txt" && fail "có test Rust hỏng"
if [ -z "$ORT_LIB" ]; then echo "LƯU Ý: không có ORT_DYLIB_PATH nên test ort không chạy (chưa kiểm ort)."; fi

step "build release các lab P06"
cargo_labs build --release --bin loc51_xuat_du_lieu --bin loc53_so_sanh_tract_ort --bin loc53_fallback --bin loc55_tai_xe_theo_du_bao --bin loc56_ab_switchback
BIN="$LAB/rust/target/release"

step "LOC-51: xuất dữ liệu fixture (21 ngày × 2 giờ + yêu cầu tới ngày 41) và pipeline Polars"
"$BIN/loc51_xuat_du_lieu" --fixture --days 21 --demand-days 42 --hours 7-8 --jobs 4 --out "$TMP/data" > "$TMP/loc51_rs.txt"
grep -E "yêu cầu đặt xe|chặng xe" "$TMP/loc51_rs.txt"
"$BIN/loc51_xuat_du_lieu" --fixture --mode eval --hours 8 --seeds 20 --jobs 4 --out "$TMP/eval" > /dev/null
py loc51_polars_pipeline.py --data "$TMP/data" --eval "$TMP/eval" --out "$TMP/features" | tee "$TMP/loc51_py.txt" | grep -E "trùng bị bỏ|phải bằng 0|SẠCH|tổng"
grep -q "(phải bằng 0)" "$TMP/loc51_py.txt" || fail "thiếu phép thử không rò rỉ"

step "LOC-52: LightGBM phần dư (không chạy độ nhạy gap)"
py loc52_eta_lightgbm.py --art "$TMP" --skip-sensitivity | tee "$TMP/loc52.txt" | grep -E "LightGBM dự đoán|route-engine một mình|sàn nhiễu" | head -5

step "LOC-53: xuất ONNX bằng onnxmltools, chạy bằng tract và ort trong Rust, fallback"
py loc53_onnx_export.py --art "$TMP" | grep -E "ghi eta-resid|max \|Δ\|"
"$BIN/loc53_so_sanh_tract_ort" --backend tract --version eta-resid-v1-cls --model-dir "$TMP/model" --calls 300 | tee "$TMP/tract.txt" | grep -E "so với|p50"
awk '/tract \(một dòng\) so với onnxruntime Python/ { for (i = 1; i <= NF; i++) if ($i == "max") { v = $(i+2); sub(/e/, "e", v); print v; exit } }' "$TMP/tract.txt" > "$TMP/delta.txt"
python3 - "$TMP/delta.txt" <<'EOF' || fail "tract lệch onnxruntime Python quá 1e-3"
import sys
v = float(open(sys.argv[1]).read().strip())
assert abs(v) < 1e-3, v
print(f"tract so với onnxruntime Python: max |Δ| = {v:.2e} (< 1e-3)")
EOF
if [ -n "$ORT_LIB" ]; then
  "$BIN/loc53_so_sanh_tract_ort" --backend ort --version eta-resid-v1 --model-dir "$TMP/model" --calls 300 | grep -E "so với|p50"
else
  echo "ort: CHƯA KIỂM (không có libonnxruntime)"
fi
"$BIN/loc53_fallback" --model-dir "$TMP/model" | tee "$TMP/fallback.txt" | grep -E "FALLBACK|số ETA không hữu hạn"
grep -q "số ETA không hữu hạn trả ra: 0" "$TMP/fallback.txt" || fail "dịch vụ trả ETA không hữu hạn"

step "LOC-54: P50/P90, hiệu chuẩn, meet_gap_s phát lại"
py loc54_quantile_calibration.py --art "$TMP" | grep -E "mô hình P50|mô hình P90|phân vị chéo"

step "LOC-55: dự báo nhu cầu, rồi chạy lại LOC-49 (vài seed trên fixture)"
py loc55_demand_forecast.py --art "$TMP" | grep -E "LightGBM Poisson|đã ghi"
"$BIN/loc55_tai_xe_theo_du_bao" --fixture --art "$TMP" --seeds 3 --jobs 4 --only thu_tu_13h | grep -E "===|hiệu =|độ khớp|dự báo LightGBM" | head -8

step "LOC-56: A/B so với switchback (fixture), phân tích Python"
"$BIN/loc56_ab_switchback" --fixture --treatment priority --tag fx --out "$TMP/loc56/priority_fx" | tail -3
py loc56_switchback_analysis.py --root "$TMP/loc56" --dirs priority_fx | tee "$TMP/loc56.txt" | grep -E "SỰ THẬT|ab \(theo khách\)|sb_"
grep -q "+0.0000 \[KTC 95% +0.0000, +0.0000\]" "$TMP/loc56.txt" || fail "sự thật toàn cục của thay đổi priority phải bằng đúng 0"

echo
echo "PASS: P06 (pytest, Rust -p eta-model và simulator, sáu bài chạy trên fixture)."
echo "KHÔNG chạy: bộ dữ liệu HCM đầy đủ và mọi số trong results/p06.md; LOC-49/LOC-56 trên HCM; p99 và bộ nhớ tract/ort trên mô hình thật; chạy độ nhạy gap."
