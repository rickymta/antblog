"""LOC-52: ETA bằng LightGBM học PHẦN DƯ của route-engine, so với baseline và so với dự đoán thẳng tổng thời gian.

Chạy từ gốc repo (cần chạy LOC-51 trước để có artifacts/features):
  uv run --project examples/toi-noi-location/python/p06 python examples/toi-noi-location/python/p06/loc52_eta_lightgbm.py

CẢNH BÁO TRUNG THỰC: dữ liệu do simulator của lab sinh ra. Khoảng cách giữa «route-engine tin» và «thế giới thật» (hiệu ứng vùng, giờ,
ngày, nhiễu) là do lab TỰ DỰNG, độ lớn do tham số `gap` của loc51_xuat_du_lieu quyết định. Mức giảm MAE ở đây đo mức mô hình khôi phục
phần hiệu ứng ẩn đó, không phải bằng chứng về một thành phố thật. Mục cuối chạy lại với các `gap` khác nhau.
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
from p06lib.common import ART, FEATURES, ZONE_NAMES, cluster_bootstrap_diff, pct, uptime, vn, zone_of

warnings.filterwarnings("ignore")
DAY0 = "2026-10-05"
NO_GPS_FEATURES = [f for f in FEATURES if f not in ("hist_speed_o", "hist_speed_d")]


def load_features(art: Path):
    feat = pl.read_parquet(art / "features" / "legs_features.parquet")
    lab = pl_.labelled(feat)
    hidden = pl.read_parquet(art / "data" / "hidden" / "hidden_truth.parquet")
    lab = lab.join(hidden, on="leg_id", how="left")
    return lab


def splits(lab: pl.DataFrame):
    return tuple(lab.filter(pl.col("split") == s) for s in ("train", "val", "test"))


def hour_baselines(tr: pl.DataFrame):
    """Hai baseline «sửa theo giờ», học trên tập huấn luyện: cộng trung bình phần dư theo giờ, hoặc nhân tỉ lệ Σthật/ΣETA theo giờ."""
    add = tr.group_by("hour").agg((pl.col("actual_s") - pl.col("route_eta_s")).mean().alias("add_s"))
    mul = tr.group_by("hour").agg((pl.col("actual_s").sum() / pl.col("route_eta_s").sum()).alias("mul"))
    return add, mul


def pred_hour_add(df, add):
    return (df.join(add, on="hour", how="left").select((pl.col("route_eta_s") + pl.col("add_s").fill_null(0.0)).alias("p")))["p"].to_numpy()


def pred_hour_mul(df, mul):
    return (df.join(mul, on="hour", how="left").select((pl.col("route_eta_s") * pl.col("mul").fill_null(1.0)).alias("p")))["p"].to_numpy()


def stats(err):
    a = np.abs(err)
    return {"mae": a.mean(), "median": float(np.median(a)), "p90": pct(a, 90), "p99": pct(a, 99), "under": float(-np.mean(err))}


def fmt_row(name, e):
    s = stats(e)
    return f"  {name:<46} MAE {s['mae']:7.2f}  |sai số| trung vị {s['median']:6.2f}  p90 {s['p90']:7.2f}  p99 {s['p99']:7.2f}  thật − dự đoán (TB; dương = dự đoán thiếu) {s['under']:+7.2f}"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--art", default=str(ART))
    ap.add_argument("--skip-sensitivity", action="store_true")
    a = ap.parse_args()
    art = Path(a.art)
    t0 = time.time()
    print("== LOC-52: ETA bằng LightGBM học phần dư của route-engine (trên dữ liệu mô phỏng) ==")
    print("DỮ LIỆU TỔNG HỢP: mô hình khôi phục máy sinh của lab; khoảng cách «route-engine tin» và «thế giới thật» do lab tự dựng (tham số gap).")
    print(f"uptime: {uptime()}")
    lab = load_features(art)
    tr, va, te = splits(lab)
    print(f"chặng có nhãn: huấn luyện {vn(tr.height)} (ngày {tr['day'].min()}–{tr['day'].max()}), kiểm định {vn(va.height)} (ngày {va['day'].min()}–{va['day'].max()}), kiểm tra {vn(te.height)} (ngày {te['day'].min()}–{te['day'].max()})")
    print(f"đặc trưng ({len(FEATURES)}): {', '.join(FEATURES)}")
    manifest = json.loads((art / "data" / "hidden" / "manifest.json").read_text())
    print(f"(sự thật ẩn, chỉ để chấm điểm) gap = {manifest['gap']}, σ vùng {manifest['zone_sigma']}, σ ngày {manifest['day_sigma']}, σ nhiễu mỗi chặng {manifest['leg_noise_sigma']}")

    # ---- baseline --------------------------------------------------------------------------------------------------
    add, mul = hour_baselines(tr)
    y = te["actual_s"].to_numpy()
    eta_te = te["route_eta_s"].to_numpy()
    e0 = eta_te - y
    e1a = pred_hour_add(te, add) - y
    e1b = pred_hour_mul(te, mul) - y
    oracle = te["noiseless_s"].to_numpy() - y  # dự đoán đúng hiệu ứng ẩn, chỉ còn nhiễu mỗi chặng mà không ai đoán được

    # ---- chọn tham số trên kiểm định -----------------------------------------------------------------------------------
    print(f"\n-- 1. chọn tham số trên tập KIỂM ĐỊNH (lưới {len(eta.GRID)} cấu hình khai báo trước; tập kiểm tra chưa dùng) --")
    yv = va["actual_s"].to_numpy()
    res = []
    for g in eta.GRID:
        mr = eta.fit_residual(tr, va, FEATURES, **g)
        mt = eta.fit_total(tr, va, FEATURES, **g)
        er = eta.mae(eta.predict_total(mr, va, FEATURES), yv)
        et = eta.mae(eta.predict_total(mt, va, FEATURES, residual=False), yv)
        res.append((g, er, et, mr.best_iteration_, mt.best_iteration_))
        print(f"  num_leaves {g['num_leaves']:>2}  min_child_samples {g['min_child_samples']:>3}:  phần dư MAE {er:7.3f} (cây {mr.best_iteration_:>4})   tổng MAE {et:7.3f} (cây {mt.best_iteration_:>4})")
    best_r = min(res, key=lambda r: r[1])
    best_t = min(res, key=lambda r: r[2])
    print(f"  chọn (theo MAE kiểm định): phần dư {best_r[0]}; tổng {best_t[0]}")
    print(f"  MAE kiểm định của baseline: route-engine {eta.mae(va['route_eta_s'].to_numpy(), yv):.3f}; sửa theo giờ (nhân) {eta.mae(pred_hour_mul(va, mul), yv):.3f}; (cộng) {eta.mae(pred_hour_add(va, add), yv):.3f}")
    mr = eta.fit_residual(tr, va, FEATURES, **best_r[0])
    mt = eta.fit_total(tr, va, FEATURES, **best_t[0])
    e_res = eta.predict_total(mr, te, FEATURES) - y
    e_tot = eta.predict_total(mt, te, FEATURES, residual=False) - y
    # lưu mô hình phần dư đã chọn cho LOC-53 (xuất ONNX) và LOC-54 (so sánh); tệp do chính script này sinh, tin cậy được
    import pickle

    (art / "model").mkdir(parents=True, exist_ok=True)
    with open(art / "model" / "eta_resid_lgbm.pkl", "wb") as fh:
        pickle.dump(mr, fh)
    (art / "model" / "best_params.json").write_text(json.dumps({"residual": best_r[0], "total": best_t[0]}))

    # ---- kết quả trên kiểm tra --------------------------------------------------------------------------------------
    print(f"\n-- 2. tập KIỂM TRA ({vn(te.height)} chặng, ngày {te['day'].min()}–{te['day'].max()}), xem một lần --")
    for nm, e in [
        ("route-engine một mình", e0),
        ("sửa theo giờ: cộng trung bình phần dư", e1a),
        ("sửa theo giờ: nhân tỉ lệ Σthật/ΣETA", e1b),
        ("LightGBM dự đoán TỔNG thời gian", e_tot),
        ("LightGBM dự đoán PHẦN DƯ của route-engine", e_res),
        ("(sàn nhiễu) đoán đúng hiệu ứng ẩn, còn nhiễu chặng", oracle),
    ]:
        print(fmt_row(nm, e))
    clusters = (te["day"].cast(pl.Int64) * 100 + te["hour"].cast(pl.Int64)).to_numpy()
    print("\n  hiệu MAE (B − A), bootstrap theo cụm giờ-ngày (2.000 lần, phân vị 2,5–97,5); âm = B tốt hơn:")
    for an, ea, bn, eb in [
        ("route-engine", e0, "sửa theo giờ (nhân)", e1b),
        ("route-engine", e0, "LightGBM phần dư", e_res),
        ("sửa theo giờ (nhân)", e1b, "LightGBM phần dư", e_res),
        ("LightGBM tổng", e_tot, "LightGBM phần dư", e_res),
    ]:
        o, lo, hi = cluster_bootstrap_diff(ea, eb, clusters)
        tag = "TỐT HƠN" if hi < 0 else ("XẤU HƠN" if lo > 0 else "không phân biệt được")
        print(f"    {an:>22} → {bn:<22} {o:+8.3f} s  [{lo:+.3f}, {hi:+.3f}]  {tag}")
    m0, mo, mr_ = np.abs(e0).mean(), np.abs(oracle).mean(), np.abs(e_res).mean()
    print(f"  phần «có thể sửa» (MAE route-engine − sàn nhiễu) = {m0 - mo:.2f} s; mô hình phần dư lấy lại {m0 - mr_:.2f} s = {100 * (m0 - mr_) / (m0 - mo):.1f}% của phần đó")
    print(f"  mô hình tốt hơn route-engine ở {100 * np.mean(np.abs(e_res) < np.abs(e0)):.1f}% chặng, tệ hơn ở {100 * np.mean(np.abs(e_res) > np.abs(e0)):.1f}%")

    # ---- theo giờ, vùng, loại chặng -----------------------------------------------------------------------------------
    print("\n-- 3. MAE theo khung giờ (giờ khởi hành, giờ Việt Nam; s) --")
    hrs = te["hour"].to_numpy()
    print("  giờ      n  route-engine  sửa-giờ(nhân)  LGBM-tổng  LGBM-phần dư  sàn nhiễu  (phần dư − route-engine)")
    worse_hours = []
    for h in range(24):
        m = hrs == h
        if m.sum() < 30:
            continue
        r0, r1, rt, rr, ro = (np.abs(x[m]).mean() for x in (e0, e1b, e_tot, e_res, oracle))
        if rr > r0:
            worse_hours.append(h)
        print(f"  {h:>3} {m.sum():>6}  {r0:>12.2f}  {r1:>13.2f}  {rt:>9.2f}  {rr:>12.2f}  {ro:>9.2f}  {rr - r0:+10.2f}")
    print(f"  giờ mà mô hình phần dư KHÔNG tốt hơn route-engine: {worse_hours if worse_hours else 'không có'}")
    print("\n-- 4. MAE theo vùng của điểm xuất phát (khoảng cách tới Chợ Bến Thành) và theo loại chặng --")
    zone = zone_of(te["o_dist_centre_m"].to_numpy())
    print("  nhóm                          n  route-engine  sửa-giờ(nhân)  LGBM-tổng  LGBM-phần dư  sàn nhiễu  (phần dư − route-engine)")
    groups = [(ZONE_NAMES[z], zone == z) for z in range(3)]
    kind = te["leg_kind"].to_numpy()
    groups += [("chặng đón (tài xế → điểm đón)", kind == 0), ("chặng có khách", kind == 1)]
    wk = te["is_weekend"].to_numpy()
    groups += [("ngày thường", wk == 0), ("cuối tuần", wk == 1)]
    eta_bucket = pl.Series(eta_te)
    for lo, hi in [(0, 120), (120, 300), (300, 600), (600, 1e9)]:
        groups.append((f"ETA route-engine {lo}–{hi if hi < 1e8 else '∞'} s", (eta_te >= lo) & (eta_te < hi)))
    for nm, m in groups:
        r0, r1, rt, rr, ro = (np.abs(x[m]).mean() for x in (e0, e1b, e_tot, e_res, oracle))
        print(f"  {nm:<28} {m.sum():>6}  {r0:>12.2f}  {r1:>13.2f}  {rt:>9.2f}  {rr:>12.2f}  {ro:>9.2f}  {rr - r0:+10.2f}")
    # ai được lợi, ai chịu thiệt: chặng nào mô hình làm tệ đi
    d = np.abs(e_res) - np.abs(e0)
    print(f"\n  phân bố (|sai số| mô hình − |sai số| route-engine) theo chặng: p5 {pct(d, 5):+.1f} s, p50 {pct(d, 50):+.1f} s, p95 {pct(d, 95):+.1f} s, tệ nhất {d.max():+.1f} s, tốt nhất {d.min():+.1f} s")
    big0 = np.abs(e0) > 300
    print(f"  chặng route-engine sai hơn 300 s ({vn(big0.sum())} chặng): MAE route-engine {np.abs(e0[big0]).mean():.1f} s → phần dư {np.abs(e_res[big0]).mean():.1f} s (đuôi vẫn là nhiễu mỗi chặng, mô hình không đoán được)")
    for nm, e in [("route-engine", e0), ("sửa theo giờ", e1b), ("LGBM phần dư", e_res), ("sàn nhiễu", oracle)]:
        print(f"  |sai số| p95 / p99 ({nm}): {pct(np.abs(e), 95):.1f} / {pct(np.abs(e), 99):.1f} s")

    # ---- tầm quan trọng đặc trưng ---------------------------------------------------------------------------------------
    imp = mr.booster_.feature_importance(importance_type="gain")
    order = np.argsort(-imp)
    tot = imp.sum()
    print("\n-- 5. tầm quan trọng (gain) của mô hình phần dư --")
    for i in order[:10]:
        print(f"  {FEATURES[i]:<18} {100 * imp[i] / tot:5.1f}%")
    print("  Đặc trưng mô tả chính xác thứ lab đã giấu (vùng, giờ, tâm kẹt) nên nằm trên cùng là điều dễ hiểu: mô hình đang khôi phục máy sinh của lab.")

    # ---- đường cong học: phần dư có giúp khi ít dữ liệu không --------------------------------------------------------
    print("\n-- 6. đường cong học: huấn luyện trên N ngày cuối của tập huấn luyện (cùng cấu hình của mô hình phần dư đã chọn) --")
    print("   số ngày  chặng huấn luyện   MAE LGBM-tổng   MAE LGBM-phần dư   (phần dư − tổng)")
    for nd in (1, 2, 3, 5, 8, 14):
        sub = tr.filter(pl.col("day") >= 14 - nd)
        m_r = eta.fit_residual(sub, va, FEATURES, **best_r[0])
        m_t = eta.fit_total(sub, va, FEATURES, **best_r[0])
        er = eta.mae(eta.predict_total(m_r, te, FEATURES), y)
        et = eta.mae(eta.predict_total(m_t, te, FEATURES, residual=False), y)
        print(f"   {nd:>7}  {vn(sub.height):>16}   {et:>13.2f}   {er:>16.2f}   {er - et:+14.2f}")

    # ---- độ nhạy với khoảng cách gap -------------------------------------------------------------------------------------
    if not a.skip_sensitivity:
        print("\n-- 7. độ nhạy: cùng pipeline, dữ liệu sinh lại với các gap khác (đặc trưng KHÔNG gồm GPS vì chạy `--no-gps`) --")
        print("   gap = độ lớn khoảng cách «thế giới thật» so với route-engine, do lab tự chọn; gap 0 = thế giới thật chỉ khác route-engine ở nhiễu mỗi chặng")
        print("   gap  chặng test   MAE route-engine  sửa-giờ(nhân)  LGBM phần dư  sàn nhiễu   giảm so với route-engine   % phần có thể sửa")
        gap_dirs = sorted(p for p in art.glob("gap_*") if (p / "legs.parquet").exists())
        rows = []
        main_dir = art / "data"
        for d in [*gap_dirs, main_dir]:
            mf = json.loads((d / "hidden" / "manifest.json").read_text())
            legs = pl.read_parquet(d / "legs.parquet")
            trips = pl.read_parquet(d / "trips.parquet")
            base = pl_.leg_base_features(legs, trips, DAY0)
            empty = pl.DataFrame(schema={"day": pl.Int32, "hour": pl.Int32, "cell": pl.Utf8, "dist_m": pl.Float64, "dt_s": pl.Float64, "n_seg": pl.UInt32})
            f = pl_.time_split(pl_.add_history(base, empty, 7))
            hid = pl.read_parquet(d / "hidden" / "hidden_truth.parquet")
            l2 = pl_.labelled(f).join(hid, on="leg_id", how="left")
            t2, v2, s2 = splits(l2)
            add2, mul2 = hour_baselines(t2)
            m2 = eta.fit_residual(t2, v2, NO_GPS_FEATURES, **best_r[0])
            y2 = s2["actual_s"].to_numpy()
            e0b = s2["route_eta_s"].to_numpy() - y2
            e1 = pred_hour_mul(s2, mul2) - y2
            er = eta.predict_total(m2, s2, NO_GPS_FEATURES) - y2
            eo = s2["noiseless_s"].to_numpy() - y2
            rows.append((mf["gap"], s2.height, np.abs(e0b).mean(), np.abs(e1).mean(), np.abs(er).mean(), np.abs(eo).mean()))
        for g, n, a0, a1, ar, ao in sorted(rows):
            frac = 100 * (a0 - ar) / (a0 - ao) if a0 - ao > 1e-9 else float("nan")
            print(f"  {g:>4.1f} {vn(n):>10}  {a0:>16.2f}  {a1:>13.2f}  {ar:>12.2f}  {ao:>9.2f}   {a0 - ar:>10.2f} s ({100 * (a0 - ar) / a0:5.1f}%)   {frac:>10.1f}%")
        print("   → mức giảm MAE tăng theo gap vì lab cho phép nó tăng; nó không nói gì về mức gap của một thành phố thật.")

    print(f"\ntổng {time.time() - t0:.0f} s; uptime: {uptime()}")


if __name__ == "__main__":
    main()
