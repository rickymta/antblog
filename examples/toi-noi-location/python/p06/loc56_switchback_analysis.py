"""LOC-56: vì sao A/B theo khách thường sai trên thị trường hai phía, và switchback có khá hơn không (trên dữ liệu mô phỏng).

Chạy từ gốc repo (cần chạy `loc56_ab_switchback` của Rust trước, mỗi thay đổi và bộ giả định một thư mục):
  uv run --project examples/toi-noi-location/python/p06 python examples/toi-noi-location/python/p06/loc56_switchback_analysis.py

So hai thiết kế với **sự thật toàn cục** (mọi người được thay đổi trừ mọi người không được, cùng seed, có cặp):
  A/B theo khách: gán ngẫu nhiên 50% khách; ước lượng = trung bình nhóm được − nhóm không (SE coi khách độc lập).
  Switchback: gán theo (vùng, khung thời gian); ước lượng cùng công thức, SE bền theo cụm đơn vị.
Báo cáo cả khi switchback KHÔNG tốt hơn. Hai thay đổi (`priority`, `eta`) và hai bộ giả định do lab chọn.
"""

from __future__ import annotations

import argparse
from pathlib import Path

import numpy as np
import polars as pl

from p06lib.common import ART, ZONE_NAMES, haversine_m, paired_ci, uptime, vn, zone_of, CENTRE, t_crit_975
from p06lib import experiments as ex

DEFAULT_DIRS = ["priority_thua_xe", "eta_thua_xe", "priority_thuong", "eta_thuong"]
DELTA = {"completed": 0.02, "wait_cens_s": 15.0}

# Chỉ số bảo vệ (guardrail): ngưỡng ĐẶT TRƯỚC, mang tính minh hoạ. «hại» = phía xấu của KTC 95% vượt ngưỡng.
# (tên cột, mô tả, ngưỡng, hướng xấu: +1 nếu tăng là xấu, −1 nếu giảm là xấu, tương đối?)
GUARDRAILS = [
    ("completion_rate", "KHÁCH: tỉ lệ hoàn thành", -0.01, -1, False),
    ("cancel_rate", "KHÁCH: tỉ lệ huỷ sau khi gán", 0.01, +1, False),
    ("unserved_rate", "KHÁCH: tỉ lệ hết kiên nhẫn chưa có xe", 0.01, +1, False),
    ("wait_mean", "KHÁCH: thời gian chờ TB của chuyến hoàn thành (s)", 30.0, +1, False),
    ("trips_gini", "TÀI XẾ: Gini số chuyến mỗi tài xế", 0.02, +1, False),
    ("zero_trip_share", "TÀI XẾ: tỉ lệ tài xế không có chuyến nào", 0.02, +1, False),
    ("idle_gap_p95_s", "TÀI XẾ: khoảng rảnh liên tiếp dài nhất, p95 (s)", 0.10, +1, True),
    ("empty_m_per_request", "TÀI XẾ: km rỗng tổng mỗi khách đặt (m)", 0.05, +1, True),
    ("busy_share", "TÀI XẾ: tỉ lệ thời gian bận (thu nhập gián tiếp)", -0.02, -1, False),
]


def mean_or_nan(series) -> float:
    """Trung bình của một cột polars; rỗng thì NaN (nhóm vùng có thể không có khách ở một seed nhỏ)."""
    v = series.mean()
    return float("nan") if v is None else float(v)


def fmt(x, d=3):
    return f"{x:+.{d}f}"


def analyse(dirpath: Path):
    name = dirpath.name
    cust, runs = ex.load(dirpath)
    seeds = sorted(cust["seed"].unique().to_list())
    n_req = cust.filter((pl.col("arm") == "off") & (pl.col("seed") == seeds[0])).height
    print(f"\n{'=' * 118}\n=== {name}: {len(seeds)} seed đánh giá {seeds[0]}…{seeds[-1]}, ~{n_req} khách/seed, sáu cách gán ===\n{'=' * 118}")
    desc = {"priority": "khách được thay đổi xếp hàng TRƯỚC khi tài xế rảnh (bật cho mọi người thì thứ tự giống hệt baseline: sự thật toàn cục = 0 theo cấu trúc)",
            "eta": "khách được thay đổi chọn tài xế gần nhất theo ETA đường bộ, khách khác theo chim bay (cải thiện thật, nhỏ)"}
    print("thay đổi:", desc[name.split("_")[0]])
    ids_units = {a: cust.filter((pl.col("arm") == a) & (pl.col("seed") == seeds[0]))
                 .select((pl.col("zone") * 1000 + pl.col("window")).n_unique()).item() for a in ex.SB_ARMS}
    print("đơn vị switchback mỗi seed (vùng × khung có khách): " + ", ".join(f"{a} {u}" for a, u in ids_units.items())
          + f"; tỉ lệ khách ở nhóm được thay đổi: ab {cust.filter(pl.col('arm') == 'ab')['treated'].mean():.3f}, "
          + ", ".join(f"{a} {cust.filter(pl.col('arm') == a)['treated'].mean():.3f}" for a in ex.SB_ARMS))
    results = {}
    for metric, mdesc in ex.METRICS.items():
        seeds_, truth, est = ex.summarize(cust, metric)
        results[metric] = (truth, est)
        d = 4 if metric == "completed" else 2
        md, lo, hi, _, _ = paired_ci(np.zeros_like(truth), truth)
        print(f"\n-- {metric}: {mdesc} --")
        print(f"  SỰ THẬT toàn cục (mọi người được − không ai được, có cặp {len(seeds_)} seed): {fmt(md, d)} [KTC 95% {fmt(lo, d)}, {fmt(hi, d)}]; độ lệch chuẩn giữa seed {truth.std(ddof=1):.{d}f}")
        print(f"  {'ước lượng':<18} {'TB ước lượng':>10}  {'thiên lệch so với sự thật [KTC 95%]':>40}  {'RMSE':>8}  {'KTC phủ':>9}  {'báo «có»':>9}  {'sai dấu':>9}")
        for nm, (e, s, g) in est.items():
            print(ex.report_row(nm, e, s, g, truth, d))
        print("  (KTC 95% mỗi seed = ước lượng ± t(G−1)·SE bền theo G cụm; cột «KTC phủ»: tỉ lệ seed mà KTC đó chứa sự thật của seed; «báo có»: tỉ lệ seed mà KTC 95% không chứa 0; «sai dấu»: tỉ lệ seed mà ước lượng trái dấu sự thật, n/a khi sự thật bằng 0)")

    # ---- ai được, ai chịu: theo vùng của điểm ghim -----------------------------------------------------------------------------
    if "pin_lat" in cust.columns:
        ring = zone_of(haversine_m(cust["pin_lat"].to_numpy(), cust["pin_lng"].to_numpy(), CENTRE[0], CENTRE[1]))
        c2 = cust.with_columns(pl.Series("ring", ring))
        print("\n-- ai được, ai chịu thiệt: theo vùng của điểm ghim (sự thật toàn cục, có cặp; và A/B ngây thơ) --")
        print(f"  {'vùng':<22} {'chỉ số':<12} {'sự thật toàn cục [KTC]':>30}  {'A/B ngây thơ [KTC]':>30}")
        for z in range(3):
            for metric in ("completed", "wait_cens_s"):
                tz, az = [], []
                for s in seeds:
                    d = c2.filter((pl.col("seed") == s) & (pl.col("ring") == z) & pl.col(metric).is_not_nan())
                    on, off = d.filter(pl.col("arm") == "on"), d.filter(pl.col("arm") == "off")
                    tz.append(mean_or_nan(on[metric]) - mean_or_nan(off[metric]))
                    ab = d.filter(pl.col("arm") == "ab")
                    az.append(mean_or_nan(ab.filter(pl.col("treated") == 1)[metric]) - mean_or_nan(ab.filter(pl.col("treated") == 0)[metric]))
                dd = 4 if metric == "completed" else 2
                a = paired_ci(np.zeros(len(tz)), np.array(tz))
                b = paired_ci(np.zeros(len(az)), np.array(az))
                print(f"  {ZONE_NAMES[z]:<22} {metric:<12} {fmt(a[0], dd):>10} [{fmt(a[1], dd)}, {fmt(a[2], dd)}]   {fmt(b[0], dd):>10} [{fmt(b[1], dd)}, {fmt(b[2], dd)}]")

    # ---- bảo vệ khách và tài xế: chỉ đo được ở cấp thị trường ---------------------------------------------------------------------
    runs = runs.with_columns((pl.col("completed") / pl.col("requests")).alias("completion_rate"))
    print("\n-- chỉ số bảo vệ cho cả KHÁCH và TÀI XẾ: thị trường «mọi người được» so với «không ai được», có cặp theo seed --")
    print("   Chỉ số phía tài xế (và chỉ số tổng hợp) chỉ đo được ở cấp thị trường, vì tài xế phục vụ cả hai nhóm khách: A/B theo khách không tách được chúng.")
    print(f"  {'chỉ số':<58} {'off (TB)':>10} {'on − off [KTC 95%]':>30} {'ngưỡng hại':>11}  kết luận")
    for col, label, thr, bad, rel in GUARDRAILS:
        a = runs.filter(pl.col("arm") == "off").sort("seed")[col].to_numpy().astype(float)
        b = runs.filter(pl.col("arm") == "on").sort("seed")[col].to_numpy().astype(float)
        md, lo, hi, _, _ = paired_ci(a, b)
        if rel:
            base = float(np.mean(a))
            md, lo, hi = md / base, lo / base, hi / base
        worst = hi if bad > 0 else lo
        tripped = (worst > thr) if bad > 0 else (worst < thr)
        unit = "%" if rel else ""
        d = 3 if not rel else 3
        print(f"  {label:<58} {np.mean(a):>10.3f} {fmt(md * (100 if rel else 1), 2 if rel else d):>10}{unit} [{fmt(lo * (100 if rel else 1), 2 if rel else d)}, {fmt(hi * (100 if rel else 1), 2 if rel else d)}]{unit:>1} {('+' if bad > 0 else '−')}{thr * (100 if rel else 1):>8.2f}{unit:<2}  {'DỪNG VÌ HẠI (phía xấu của KTC vượt ngưỡng)' if tripped else 'chưa thấy hại'}")

    # ---- tiêu chí dừng ---------------------------------------------------------------------------------------------------------------
    print("\n-- tiêu chí dừng: cần bao nhiêu «giờ thí nghiệm» (mỗi giờ ~ một seed) để phân biệt được hiệu quả δ, công suất 80%, α 5% --")
    print(f"  {'ước lượng':<18} " + "  ".join(f"{m}: SD/giờ, số giờ cần cho δ={DELTA[m]}" for m in DELTA))
    for nm in ["ab (theo khách)", *ex.SB_ARMS]:
        parts = []
        for m, dl in DELTA.items():
            sd = float(np.nanstd(results[m][1][nm][0], ddof=1))
            parts.append(f"{sd:>8.4f} → {ex.seeds_needed(sd, dl):>7.1f}")
        print(f"  {nm:<18} " + "    ".join(parts))
    print("  (SD là độ lệch chuẩn của ước lượng giữa các seed. Con số này giả định ước lượng không thiên lệch; xem bảng thiên lệch ở trên.)")
    print("  KTC 95% của TRUNG BÌNH ước lượng khi dùng n giờ đầu tiên (completed):")
    truth = results["completed"][0]
    for nm in ["ab (theo khách)", "sb_t10"]:
        e = results["completed"][1][nm][0]
        print(f"    {nm:<18} " + "; ".join(f"n={n}: {fmt(m, 4)} [{fmt(lo, 4)}, {fmt(hi, 4)}]" for n, m, lo, hi in ex.cumulative_ci(e)))
    print(f"    {'sự thật':<18} " + "; ".join(f"n={n}: {fmt(m, 4)} [{fmt(lo, 4)}, {fmt(hi, 4)}]" for n, m, lo, hi in ex.cumulative_ci(truth)))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dirs", nargs="*", default=DEFAULT_DIRS)
    ap.add_argument("--root", default=str(ART / "loc56"))
    a = ap.parse_args()
    print("== LOC-56: A/B theo khách so với switchback, đối chiếu sự thật toàn cục (trên dữ liệu mô phỏng) ==")
    print(f"uptime: {uptime()}")
    for d in a.dirs:
        p = Path(a.root) / d
        if not (p / "customers.csv").exists():
            print(f"\n(bỏ qua {d}: chưa có dữ liệu, chạy loc56_ab_switchback trước)")
            continue
        analyse(p)
    print(f"\nuptime: {uptime()}")


if __name__ == "__main__":
    main()
