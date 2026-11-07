"""LOC-54: độ tin cậy của ETA. Hồi quy phân vị P50 và P90 của phần dư route-engine, kiểm tra hiệu chuẩn (độ phủ thực nghiệm theo nhóm),
app nên hiện con số nào, và ảnh hưởng tới `meet_gap_s` (phát lại trên các chặng đón của 20 seed đánh giá).

Chạy từ gốc repo (sau LOC-51, LOC-52):
  uv run --project examples/toi-noi-location/python/p06 python examples/toi-noi-location/python/p06/loc54_quantile_calibration.py

Dữ liệu mô phỏng; nhiễu mỗi chặng là log-chuẩn do lab đặt (σ = 0,12) nên P90 «đúng» là một hàm trơn của thời gian. Thật ngoài đời đuôi nặng hơn.
`meet_gap_s` ở đây được PHÁT LẠI từ thời gian thật của simulator theo một giả định hành vi khách, không phải chạy lại simulator.
"""

from __future__ import annotations

import argparse
import json
import time
import warnings
from pathlib import Path

import numpy as np
import polars as pl

from p06lib import eta, pipeline as pl_
from p06lib.common import ART, FEATURES, ZONE_NAMES, paired_ci, pct, uptime, vn, zone_of, t_crit_975

warnings.filterwarnings("ignore")
WALK_SPEED = 1.3  # m/s, customer.walk_speed_mps của kịch bản gốc
ALPHAS = (0.5, 0.9)


def pinball(y, p, a):
    d = y - p
    return float(np.mean(np.maximum(a * d, (a - 1) * d)))


def cluster_ci(hit: np.ndarray, clusters: np.ndarray, n_boot=2000, seed=3):
    """Độ phủ và KTC 95% (bootstrap theo cụm giờ-ngày)."""
    uniq, inv = np.unique(clusters, return_inverse=True)
    k = len(uniq)
    s = np.bincount(inv, weights=hit.astype(float), minlength=k)
    n = np.bincount(inv, minlength=k).astype(float)
    rng = np.random.default_rng(seed)
    idx = rng.integers(0, k, size=(n_boot, k))
    c = s[idx].sum(1) / np.maximum(n[idx].sum(1), 1)
    lo, hi = np.percentile(c, [2.5, 97.5])
    return float(hit.mean()), float(lo), float(hi)


def buckets(df: pl.DataFrame):
    hour = df["hour"].to_numpy()
    zone = zone_of(df["o_dist_centre_m"].to_numpy())
    kind = df["leg_kind"].to_numpy()
    wk = df["is_weekend"].to_numpy()
    eta_s = df["route_eta_s"].to_numpy()
    g = [("tất cả", np.ones(len(df), bool))]
    g += [("giờ 00–05", hour <= 5), ("giờ 06–09", (hour >= 6) & (hour <= 9)), ("giờ 10–16", (hour >= 10) & (hour <= 16)),
          ("giờ 17–20", (hour >= 17) & (hour <= 20)), ("giờ 21–23", hour >= 21)]
    g += [(ZONE_NAMES[z], zone == z) for z in range(3)]
    g += [("chặng đón", kind == 0), ("chặng có khách", kind == 1), ("ngày thường", wk == 0), ("cuối tuần", wk == 1)]
    for lo, hi in [(0, 120), (120, 300), (300, 600), (600, 1e9)]:
        g.append((f"ETA gốc {lo}–{hi if hi < 1e8 else '∞'} s", (eta_s >= lo) & (eta_s < hi)))
    return g


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--art", default=str(ART))
    a = ap.parse_args()
    art = Path(a.art)
    t0 = time.time()
    print("== LOC-54: độ tin cậy của ETA, P50/P90 và hiệu chuẩn (trên dữ liệu mô phỏng) ==")
    print("DỮ LIỆU TỔNG HỢP: nhiễu mỗi chặng là log-chuẩn σ = 0,12 do lab đặt; hiệu chuẩn tốt ở đây chưa nói gì về hiệu chuẩn ngoài đời.")
    print(f"uptime: {uptime()}")
    lab = pl_.labelled(pl.read_parquet(art / "features" / "legs_features.parquet"))
    tr, va, te = (lab.filter(pl.col("split") == s) for s in ("train", "val", "test"))
    params = json.loads((art / "model" / "best_params.json").read_text())["residual"]
    print(f"chặng: huấn luyện {vn(tr.height)}, kiểm định {vn(va.height)}, kiểm tra {vn(te.height)}; cấu hình cây (chọn ở LOC-52): {params}")

    # ---- huấn luyện ---------------------------------------------------------------------------------------------------
    models = {}
    for al in ALPHAS:
        Xt, et, at = eta.arrays(tr, FEATURES)
        Xv, ev, av = eta.arrays(va, FEATURES)
        models[al] = eta.fit(Xt, at - et, Xv, av - ev, eval_metric="quantile", objective="quantile", alpha=al, **params)
        print(f"  phân vị {al}: {models[al].best_iteration_} cây")

    def predict(df, al):
        X, e, _ = eta.arrays(df, FEATURES)
        return e + models[al].predict(X)

    y = te["actual_s"].to_numpy()
    eta_te = te["route_eta_s"].to_numpy()
    p50, p90 = predict(te, 0.5), predict(te, 0.9)
    crossing = float(np.mean(p90 < p50))
    p90 = np.maximum(p90, p50)  # sửa quantile chéo nhau
    print(f"\nphân vị chéo nhau (P90 < P50) trước khi sửa: {100 * crossing:.2f}% dòng kiểm tra; đã nâng P90 bằng P50 ở các dòng đó")

    # baseline P90
    ratio = tr["actual_s"].to_numpy() / tr["route_eta_s"].to_numpy()
    r90 = float(np.quantile(ratio, 0.9))
    rh = {h: float(np.quantile(g["actual_s"].to_numpy() / g["route_eta_s"].to_numpy(), 0.9)) for (h,), g in tr.group_by("hour")}
    base_const = eta_te * r90
    base_hour = eta_te * np.array([rh.get(h, r90) for h in te["hour"].to_numpy()])
    # recalibrate (split conformal) trên kiểm định
    pv90 = np.maximum(predict(va, 0.9), predict(va, 0.5))
    score = va["actual_s"].to_numpy() - pv90
    nv = len(score)
    c = float(np.quantile(score, min(1.0, np.ceil((nv + 1) * 0.9) / nv)))
    p90c = p90 + c
    print(f"baseline P90: ETA × {r90:.4f} (phân vị 90 của tỉ lệ thật/ETA trên huấn luyện); ETA × hệ số theo giờ; mô hình; mô hình + hiệu chỉnh conformal c = {c:+.2f} s (tính trên kiểm định)")

    clusters = (te["day"].cast(pl.Int64) * 100 + te["hour"].cast(pl.Int64)).to_numpy()
    print(f"\n-- 1. hiệu chuẩn trên tập KIỂM TRA ({vn(te.height)} chặng, ngày {te['day'].min()}–{te['day'].max()}): độ phủ P(thật ≤ dự đoán) --")
    print("   mục tiêu: P50 → 50%, P90 → 90%; KTC 95% bootstrap theo cụm giờ-ngày")
    for nm, p, tgt in [("route-engine một mình (coi là P50)", eta_te, 0.5), ("mô hình P50", p50, 0.5), ("ETA × hệ số hằng (P90 baseline)", base_const, 0.9),
                       ("ETA × hệ số theo giờ (P90 baseline)", base_hour, 0.9), ("mô hình P90", p90, 0.9), ("mô hình P90 + conformal", p90c, 0.9)]:
        cv, lo, hi = cluster_ci((y <= p), clusters)
        print(f"  {nm:<40} độ phủ {100 * cv:5.1f}% [{100 * lo:.1f}, {100 * hi:.1f}] (mục tiêu {100 * tgt:.0f}%)   pinball {pinball(y, p, tgt):7.2f} s   độ rộng TB (P − P50 mô hình) {np.mean(p - p50):+7.2f} s")
    print(f"  pinball của mô hình P90 so với baseline hằng: {pinball(y, p90, 0.9):.2f} vs {pinball(y, base_const, 0.9):.2f}; theo giờ {pinball(y, base_hour, 0.9):.2f}")

    print("\n-- 2. độ phủ theo nhóm (P90 mô hình; cột cuối: P90 sau conformal) --")
    print(f"  {'nhóm':<24} {'n':>7} {'độ phủ P50':>11} {'độ phủ P90':>11} [KTC 95%]        {'lệch so với 90%':>16} {'ETA×hằng':>10} {'sau conformal':>14}")
    worst = (0.0, "")
    rows = []
    for nm, m in buckets(te):
        if m.sum() < 100:
            continue
        c50 = np.mean(y[m] <= p50[m])
        cv, lo, hi = cluster_ci((y[m] <= p90[m]), clusters[m])
        cb = np.mean(y[m] <= base_const[m])
        cc = np.mean(y[m] <= p90c[m])
        rows.append((nm, m.sum(), c50, cv, lo, hi, cb, cc))
        if nm != "tất cả" and abs(cv - 0.9) > abs(worst[0]):
            worst = (cv - 0.9, nm)
        print(f"  {nm:<24} {m.sum():>7} {100 * c50:>10.1f}% {100 * cv:>10.1f}% [{100 * lo:.1f}, {100 * hi:.1f}] {100 * (cv - 0.9):>+15.1f}% {100 * cb:>9.1f}% {100 * cc:>13.1f}%")
    print(f"  nhóm lệch xa 90% nhất: {worst[1]} ({100 * worst[0]:+.1f} điểm phần trăm)")
    print("  theo từng ngày kiểm tra (cú sốc theo ngày của thế giới ẩn làm độ phủ của cả ngày lệch cùng chiều, không mô hình nào biết trước):")
    dayk = te["day"].to_numpy()
    for d in sorted(set(dayk.tolist())):
        m = dayk == d
        print(f"    ngày {d} (thứ {['Hai', 'Ba', 'Tư', 'Năm', 'Sáu', 'Bảy', 'Chủ nhật'][d % 7]}): n = {vn(m.sum())}; P50 phủ {100 * np.mean(y[m] <= p50[m]):.1f}%; P90 phủ {100 * np.mean(y[m] <= p90[m]):.1f}%")
    print("  độ tin cậy theo thập phân vị của P90 dự đoán (độ phủ mỗi thập phân vị; hiệu chuẩn tốt ⇒ mọi thập phân vị gần 90%):")
    edges = np.quantile(p90, np.linspace(0, 1, 11))
    cov = []
    for i in range(10):
        m = (p90 >= edges[i]) & (p90 <= edges[i + 1]) if i == 9 else (p90 >= edges[i]) & (p90 < edges[i + 1])
        cov.append(100 * np.mean(y[m] <= p90[m]))
    print("   " + " ".join(f"{x:5.1f}" for x in cov) + f"   (P90 dự đoán từ {edges[0]:.0f} s tới {edges[-1]:.0f} s)")

    # ---- kiểm tra ngoài thời gian trên 20 seed đánh giá ------------------------------------------------------------------------
    ev = pl_.labelled(pl.read_parquet(art / "features" / "eval_features.parquet"))
    print(f"\n-- 3. kiểm tra ngoài thời gian: {vn(ev.height)} chặng của 20 seed đánh giá (ngày {ev['day'].min()}–{ev['day'].max()}, giờ 8, 13, 18; lịch sử lấy từ bảng đóng băng ngày 14–20) --")
    ye = ev["actual_s"].to_numpy()
    e50, e90 = predict(ev, 0.5), predict(ev, 0.9)
    e90 = np.maximum(e90, e50)
    cl_e = (ev["day"].cast(pl.Int64) * 100 + ev["hour"].cast(pl.Int64)).to_numpy()
    for nm, p, tgt in [("mô hình P50", e50, 0.5), ("mô hình P90", e90, 0.9), ("mô hình P90 + conformal", e90 + c, 0.9)]:
        cv, lo, hi = cluster_ci((ye <= p), cl_e)
        print(f"  {nm:<28} độ phủ {100 * cv:5.1f}% [{100 * lo:.1f}, {100 * hi:.1f}] (mục tiêu {100 * tgt:.0f}%)")
    for h in (8, 13, 18):
        m = ev["hour"].to_numpy() == h
        print(f"    giờ {h:>2}: P50 phủ {100 * np.mean(ye[m] <= e50[m]):.1f}%, P90 phủ {100 * np.mean(ye[m] <= e90[m]):.1f}% (n = {vn(m.sum())})")

    # ---- con số nào để hiện -------------------------------------------------------------------------------------------------------
    print("\n-- 4. con số nào hiện cho khách (chặng đón của tập kiểm tra; «hiện E» nghĩa là app nói xe tới sau E giây) --")
    pk = te["leg_kind"].to_numpy() == 0
    print(f"  {'hiện':<26} {'xe tới trễ hơn số hiện':>24} {'trễ TB khi trễ (s)':>20} {'sớm TB khi sớm (s)':>20} {'hiện TB (s)':>12}")
    for nm, p in [("ETA route-engine", eta_te), ("P50 mô hình", p50), ("P90 mô hình", p90)]:
        late = y[pk] > p[pk]
        print(f"  {nm:<26} {100 * late.mean():>23.1f}% {np.mean((y - p)[pk][late]):>20.1f} {np.mean((p - y)[pk][~late]):>20.1f} {np.mean(p[pk]):>12.1f}")

    # ---- meet_gap_s phát lại ----------------------------------------------------------------------------------------------------------
    print("\n-- 5. ảnh hưởng tới meet_gap_s: PHÁT LẠI trên chặng đón hoàn thành của 20 seed đánh giá (không chạy lại simulator) --")
    trips = pl.read_parquet(art / "eval" / "trips.parquet").filter(pl.col("trip_id").is_not_null()).select("trip_id", "request_time", "walk_m")
    pk_e = ev.filter(pl.col("leg_kind") == 0).join(trips, on="trip_id", how="left", suffix="_t")
    ok = pk_e["walk_m"].is_not_null()
    pk_e = pk_e.filter(ok)
    pe50, pe90 = np.maximum(predict(pk_e, 0.5), 0), np.maximum(predict(pk_e, 0.9), predict(pk_e, 0.5))
    depart = (pk_e["depart_time"].dt.timestamp("ms").to_numpy()) / 1000.0
    req = (pk_e["request_time_t"].dt.timestamp("ms").to_numpy()) / 1000.0
    walk_s = pk_e["walk_m"].to_numpy() / WALK_SPEED
    T = depart + pk_e["actual_s"].to_numpy()  # giờ xe thật sự tới điểm đón
    seed_idx = pk_e["day"].to_numpy() - 28
    policies = {
        "baseline (khách đứng chờ ở ghim)": req + walk_s,
        "hiện ETA route-engine": np.maximum(req + walk_s, depart + pk_e["route_eta_s"].to_numpy()),
        "hiện P50 mô hình": np.maximum(req + walk_s, depart + pe50),
        "hiện P90 mô hình": np.maximum(req + walk_s, depart + pe90),
    }
    print(f"  {vn(pk_e.height)} chặng đón; giả định: khách tới điểm đón lúc max(giờ đặt + thời gian đi bộ, giờ gán + ETA đã hiện), đi bộ {WALK_SPEED} m/s; huỷ không phụ thuộc ETA hiện ra")
    print("  (mô hình huỷ của simulator dùng ETA kế hoạch chứ không dùng con số hiện cho khách, nên hiện P90 lớn hơn KHÔNG làm tăng huỷ ở đây: giới hạn lớn của phép phát lại)")
    seeds = sorted(set(seed_idx.tolist()))

    def per_seed(f):
        return {k: np.array([f(C[seed_idx == s], T[seed_idx == s]) for s in seeds]) for k, C in policies.items()}

    metrics = [
        ("meet_gap_s trung bình", lambda C, T: np.mean(np.abs(T - C))),
        ("meet_gap_s p95", lambda C, T: pct(np.abs(T - C), 95)),
        ("khách chờ ở ghim TB (s)", lambda C, T: np.mean(np.maximum(0, T - C))),
        ("tài xế chờ khách TB (s)", lambda C, T: np.mean(np.maximum(0, C - T))),
        ("% chuyến tài xế chờ > 60 s", lambda C, T: 100 * np.mean(C - T > 60)),
        ("% chuyến khách chờ > 120 s", lambda C, T: 100 * np.mean(T - C > 120)),
    ]
    ref = "hiện ETA route-engine"
    print(f"  {'chỉ số':<30} {'chính sách':<34} {'TB 20 seed':>11} {'hiệu so với «' + ref + '» [KTC 95%]':>46} {'thấp hơn/cao hơn':>17}")
    for mname, f in metrics:
        vals = per_seed(f)
        for pn, v in vals.items():
            if pn == ref:
                print(f"  {mname:<30} {pn:<34} {v.mean():>11.2f} {'(mốc so sánh)':>46}")
            else:
                md, lo, hi, nb_lo, nb_hi = paired_ci(vals[ref], v)
                tag = "TỐT HƠN" if hi < 0 else ("XẤU HƠN" if lo > 0 else "không phân biệt")
                print(f"  {'':<30} {pn:<34} {v.mean():>11.2f} {md:>+14.2f} [{lo:+.2f}, {hi:+.2f}] {tag:<16} {nb_lo:>3}/{nb_hi:<3}")
        print()
    print("  (hiệu = chính sách − hiện ETA route-engine; âm là thấp hơn. «TỐT HƠN/XẤU HƠN» chỉ nói chỉ số thấp hơn/cao hơn, không nói ai chịu: khách chờ ở ghim và tài xế chờ khách là hai bên khác nhau.)")
    print("  Chưa đo: điểm hẹn động của LOC-36 (không cắm vào phép phát lại), hành vi huỷ phụ thuộc con số hiện, khách thật đọc con số khác với mô hình đơn giản này.")
    print(f"\ntổng {time.time() - t0:.0f} s; uptime: {uptime()}")


if __name__ == "__main__":
    main()
