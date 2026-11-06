"""LOC-51: pipeline Polars từ Parquet GPS (LOC-20) tới đặc trưng chặng theo giờ và ô H3. Trên dữ liệu mô phỏng.

Chạy từ gốc repo:
  uv run --project examples/toi-noi-location/python/p06 python examples/toi-noi-location/python/p06/loc51_polars_pipeline.py

Làm: đọc Parquet theo hợp đồng v1, khử trùng, nối chuyến, dựng đặc trưng theo (giờ, ô H3) chỉ từ ngày trước, chia theo thời gian, rồi
ĐO hậu quả của bốn kiểu rò rỉ (A cửa sổ quên dịch một ngày, B chia ngẫu nhiên, C mã hoá theo nhãn trên toàn bộ dữ liệu, D nối GPS của
chính chuyến). Có kiểm chứng bằng phép thử: xáo nhãn của tương lai thì đặc trưng của quá khứ không đổi.

Dữ liệu do simulator sinh, nên mọi điểm MAE ở đây đo mức mô hình khôi phục máy sinh dữ liệu của lab, không phải độ chính xác ngoài đời.
"""

from __future__ import annotations

import argparse
import time
import warnings
from pathlib import Path

import numpy as np
import polars as pl

from p06lib import eta, pipeline as pl_
from p06lib.common import ART, FEATURES, FROZEN_DAYS, TEST_DAYS, uptime, vn

warnings.filterwarnings("ignore")
DAY0 = "2026-10-05"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=str(ART / "data"))
    ap.add_argument("--eval", default=str(ART / "eval"))
    ap.add_argument("--out", default=str(ART / "features"))
    a = ap.parse_args()
    data, eval_dir, out = Path(a.data), Path(a.eval), Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    t0 = time.time()
    print("== LOC-51: pipeline Polars từ Parquet GPS tới đặc trưng chặng (trên dữ liệu mô phỏng) ==")
    print(f"polars {pl.__version__}; uptime: {uptime()}")
    print(f"thư mục dữ liệu: {data}")

    # ---- 1. đọc và khử trùng ---------------------------------------------------------------------------------------
    print("\n-- 1. đọc theo hợp đồng Parquet v1, khử trùng theo (driver_ref, event_time, seq) --")
    gps = pl_.scan_gps(data / "history-pseudo")
    raw = gps.select(pl.len()).collect().item()
    dd = pl_.dedupe_gps(gps)
    n = dd.select(pl.len()).collect().item()
    print(f"GPS: {vn(raw)} dòng đọc được; {vn(raw - n)} dòng trùng bị bỏ ({100 * (raw - n) / raw:.3f}%); còn {vn(n)}")
    trips = pl.read_parquet(data / "trips.parquet")
    legs = pl.read_parquet(data / "legs.parquet")
    sch = gps.collect_schema()
    print("lược đồ đọc được (date, city lấy từ đường dẫn):", ", ".join(f"{k}:{v}" for k, v in sch.items()))
    files = list((data / "history-pseudo").rglob("*.parquet"))
    print(f"số file: {vn(len(files))}; trong đó file «phát lại» (replay): {sum('replay' in f.name for f in files)}")
    nl = dd.select(
        pl.len().alias("n"),
        pl.col("accuracy_dm").is_null().sum().alias("accuracy_null"),
        pl.col("speed_dmps").is_null().sum().alias("speed_null"),
        (pl.col("speed_dmps") == 0).sum().alias("speed_zero"),
        pl.col("heading_ddeg").is_null().sum().alias("heading_null"),
        pl.col("trip_id").is_null().sum().alias("trip_null"),
    ).collect().row(0, named=True)
    print(
        f"NULL khác 0: speed_dmps NULL {vn(nl['speed_null'])} ({100 * nl['speed_null'] / nl['n']:.2f}%) còn = 0 (đứng yên) "
        f"{vn(nl['speed_zero'])} ({100 * nl['speed_zero'] / nl['n']:.2f}%); accuracy_dm NULL {vn(nl['accuracy_null'])}; "
        f"heading_ddeg NULL {vn(nl['heading_null'])}; trip_id NULL {vn(nl['trip_null'])}"
    )
    print("lý do giữ điểm (sample_reason):", dict(dd.group_by("sample_reason").len().sort("len", descending=True).collect().iter_rows()))
    gaps = (
        dd.sort(["driver_ref", "event_time"])
        .with_columns(((pl.col("event_time") - pl.col("event_time").shift(1).over(["driver_ref", "trip_id"])).dt.total_milliseconds() / 1000.0).alias("gap_s"))
        .filter(pl.col("gap_s").is_not_null())
        .select(
            pl.col("gap_s").quantile(0.05).alias("p05"),
            pl.col("gap_s").median().alias("p50"),
            pl.col("gap_s").quantile(0.95).alias("p95"),
            pl.col("gap_s").max().alias("max"),
        )
        .collect()
        .row(0, named=True)
    )
    print("khoảng cách thời gian giữa hai dòng liên tiếp cùng chuyến (giây), KHÔNG đều vì giảm mẫu: " + ", ".join(f"{k} {v:.1f}" for k, v in gaps.items()))

    # ---- 2. tốc độ theo ô: đúng và sai -----------------------------------------------------------------------------
    print("\n-- 2. tốc độ trung bình: Σ quãng / Σ thời gian so với trung bình cột speed_dmps (m/s) --")
    sp = (
        dd.with_columns(pl_.local_parts("event_time"))
        .group_by("hour")
        .agg(
            (pl.col("speed_dmps").mean() / 10).alias("naive_skip_null"),
            (pl.col("speed_dmps").fill_null(0).mean() / 10).alias("naive_fill0"),
        )
        .sort("hour")
        .collect()
    )
    cell_hour = pl_.cell_hour_table(dd, DAY0)
    tw = cell_hour.group_by("hour").agg((pl.col("dist_m").sum() / pl.col("dt_s").sum()).alias("time_weighted")).sort("hour")
    cmp = sp.join(tw, on="hour").filter(pl.col("hour").is_in([3, 8, 13, 18]))
    print("giờ  trung bình speed_dmps (bỏ NULL)  fill_null(0)  có trọng số thời gian")
    for r in cmp.iter_rows(named=True):
        print(f"{r['hour']:>3}  {r['naive_skip_null']:>30.2f}  {r['naive_fill0']:>12.2f}  {r['time_weighted']:>20.2f}")
    print(f"bảng (ngày, giờ, ô H3 độ phân giải {pl_.H3_RES}): {vn(cell_hour.height)} dòng; {cell_hour['cell'].n_unique()} ô; {cell_hour['day'].n_unique()} ngày")

    # ---- 3. nối chuyến, đặc trưng -----------------------------------------------------------------------------------
    print("\n-- 3. nối chặng với chuyến, đặc trưng biết lúc xe khởi hành --")
    base = pl_.leg_base_features(legs, trips, DAY0)
    lab = pl_.labelled(base)
    print(f"chặng: {vn(legs.height)}; bỏ chặng huỷ giữa đường (không có nhãn): {vn(legs.height - lab.height)}; còn {vn(lab.height)}")
    print(
        f"bảng trips: {vn(trips.height)} yêu cầu, {vn(trips.filter(pl.col('simulated')).height)} thuộc ngày có mô phỏng, "
        f"{vn(trips['trip_id'].is_not_null().sum())} có tài xế"
    )
    feat = pl_.time_split(pl_.add_history(base, cell_hour, 7))
    labf = pl_.labelled(feat)
    cnt = labf.group_by("split").agg(pl.len().alias("n"), pl.col("day").min().alias("ngày đầu"), pl.col("day").max().alias("ngày cuối")).sort("ngày đầu")
    print("chia theo thời gian (ngày 0 = thứ Hai 2026-10-05):")
    print(cnt)
    tr_end = labf.filter(pl.col("split") == "train")["depart_time"].max()
    va_beg = labf.filter(pl.col("split") == "val")["depart_time"].min()
    va_end = labf.filter(pl.col("split") == "val")["depart_time"].max()
    te_beg = labf.filter(pl.col("split") == "test")["depart_time"].min()
    assert tr_end < va_beg and va_end < te_beg, "các tập phải nối tiếp nhau theo thời gian"
    print(f"kiểm tra: huấn luyện kết thúc {tr_end} < kiểm định bắt đầu {va_beg}; kiểm định kết thúc {va_end} < kiểm tra bắt đầu {te_beg}")
    miss = {c: float(labf[c].is_null().mean()) for c in ("hist_speed_o", "hist_speed_d", "hist_resid_o")}
    print("tỉ lệ thiếu đặc trưng lịch sử (null; LightGBM xử lý được): " + ", ".join(f"{k} {100 * v:.1f}%" for k, v in miss.items()))
    for d in (0, 1, 7, TEST_DAYS[0], TEST_DAYS[1]):
        r = labf.filter(pl.col("day") == d)
        print(f"  ngày {d:>2}: hist_resid_o có ở {100 * (1 - r['hist_resid_o'].null_count() / r.height):.1f}% chặng, hist_speed_o có ở {100 * (1 - r['hist_speed_o'].null_count() / r.height):.1f}%")

    # ---- 4. rò rỉ ---------------------------------------------------------------------------------------------------
    print("\n-- 4. đo hậu quả của rò rỉ (cùng LightGBM, cùng tham số, mục tiêu = phần dư của route-engine) --")
    leak_a = pl_.time_split(pl_.add_history(base, cell_hour, 7, include_current=True))
    leak_c = pl_.time_split(pl_.add_leaky_target_encoding(pl_.add_history(base, cell_hour, 7)))
    own = pl_.trip_own_gps_speed(dd)
    leak_d = pl_.time_split(pl_.add_history(base, cell_hour, 7)).join(own, on="trip_id", how="left")

    def score(df, cols, label):
        d = pl_.labelled(df)
        tr, va, te = (d.filter(pl.col("split") == s) for s in ("train", "val", "test"))
        m = eta.fit_residual(tr, va, cols)
        p = eta.predict_total(m, te, cols)
        e = eta.mae(p, te["actual_s"].to_numpy())
        print(f"  {label:<70} MAE kiểm tra {e:7.2f} s  (số cây: {m.best_iteration_})")
        return e, m

    te0 = labf.filter(pl.col("split") == "test")
    base_mae = eta.mae(te0["route_eta_s"].to_numpy(), te0["actual_s"].to_numpy())
    print(f"  {'route-engine một mình (không học gì)':<70} MAE kiểm tra {base_mae:7.2f} s")
    clean, m_clean = score(feat, FEATURES, "SẠCH: cửa sổ 7 ngày TRƯỚC, chia theo thời gian")
    r_a, _ = score(leak_a, FEATURES, "A: cửa sổ quên dịch một ngày (gồm ngày hiện tại)")
    r_c, _ = score(leak_c, FEATURES + ["leak_resid_o"], "C: mã hoá theo nhãn trên TOÀN BỘ dữ liệu (cả tương lai)")
    r_d, _ = score(leak_d, FEATURES + ["leak_trip_speed"], "D: nối GPS của chính chuyến (tốc độ trung bình cả chuyến)")
    print(f"  mức giảm MAE so với route-engine một mình (cùng tập kiểm tra): SẠCH {base_mae - clean:+.2f} s; A {base_mae - r_a:+.2f} s; C {base_mae - r_c:+.2f} s; D {base_mae - r_d:+.2f} s")
    # B: chia ngẫu nhiên, đặc trưng sạch. Chấm trên các dòng ngẫu nhiên giữ lại của ngày 17–20, so cùng các dòng đó với mô hình SẠCH
    d = labf.filter(pl.col("day") <= TEST_DAYS[1])
    u = np.random.default_rng(eta.SEED).random(d.height)
    tr, va, te = d.filter(pl.Series(u < 0.7)), d.filter(pl.Series((u >= 0.7) & (u < 0.85))), d.filter(pl.Series(u >= 0.85))
    m = eta.fit_residual(tr, va, FEATURES)
    late = te.filter(pl.col("day") >= TEST_DAYS[0])
    yl = late["actual_s"].to_numpy()
    e_rand = eta.mae(eta.predict_total(m, late, FEATURES), yl)
    e_clean_same = eta.mae(eta.predict_total(m_clean, late, FEATURES), yl)
    e_route_same = eta.mae(late["route_eta_s"].to_numpy(), yl)
    print(f"  B: chia NGẪU NHIÊN 70/15/15 các chặng ngày 0–20 (số cây: {m.best_iteration_}); chấm trên {vn(late.height)} dòng ngẫu nhiên giữ lại của ngày 17–20:")
    print(f"     route-engine một mình {e_route_same:7.2f} s; mô hình SẠCH (chia theo thời gian) {e_clean_same:7.2f} s; mô hình B {e_rand:7.2f} s  → B «đẹp hơn» SẠCH {e_clean_same - e_rand:+.2f} s trên cùng các dòng")

    # ---- 5. kiểm chứng không rò rỉ ----------------------------------------------------------------------------------
    print(f"\n-- 5. kiểm chứng không rò rỉ: xáo nhãn của các ngày ≥ {TEST_DAYS[0]} rồi dựng lại đặc trưng của ngày {TEST_DAYS[0]} --")
    d_t = TEST_DAYS[0]
    day_a = feat.filter(pl.col("day") == d_t).select("leg_id", "hist_resid_o", "hist_speed_o")
    late = base.filter(pl.col("day") >= d_t)
    shuffled = late.with_columns(pl.col("ratio").shuffle(seed=1).alias("ratio"))
    base2 = pl.concat([base.filter(pl.col("day") < d_t), shuffled.select(base.columns)])
    feat2 = pl_.add_history(base2, cell_hour, 7)
    day_b = feat2.filter(pl.col("day") == d_t).select("leg_id", pl.col("hist_resid_o").alias("hr2"), pl.col("hist_speed_o").alias("hs2"))
    j = day_a.join(day_b, on="leg_id")
    diff = j.filter(((pl.col("hist_resid_o").fill_null(-9) - pl.col("hr2").fill_null(-9)).abs() > 1e-12) | ((pl.col("hist_speed_o").fill_null(-9) - pl.col("hs2").fill_null(-9)).abs() > 1e-12)).height
    print(f"  {vn(j.height)} chặng ngày {d_t}: số chặng có đặc trưng lịch sử thay đổi khi xáo nhãn từ ngày {d_t} trở đi = {diff} (phải bằng 0)")
    assert diff == 0
    nxt = d_t + 1
    a13 = feat.filter(pl.col("day") == nxt).select("leg_id", "hist_resid_o")
    b13 = feat2.filter(pl.col("day") == nxt).select("leg_id", pl.col("hist_resid_o").alias("hr2"))
    d13 = a13.join(b13, on="leg_id").filter((pl.col("hist_resid_o").fill_null(-9) - pl.col("hr2").fill_null(-9)).abs() > 1e-12).height
    print(f"  (đối chứng) ngày {nxt} dùng nhãn ngày {d_t} làm lịch sử nên ĐƯỢC phép đổi: {vn(d13)} chặng ngày {nxt} đổi giá trị")

    # ---- 6. bảng tra đóng băng cho ngày chưa có cửa sổ trượt ---------------------------------------------------------
    print("\n-- 6. bảng tra đóng băng cho các ngày sau dữ liệu (service nạp bảng có phiên bản) --")
    sp_f, rs_f = pl_.frozen_history(cell_hour, lab, *FROZEN_DAYS)
    print(f"  bảng tốc độ (giờ, ô): {vn(sp_f.height)} dòng; bảng phần dư (giờ, ô): {vn(rs_f.height)} dòng; gộp ngày {FROZEN_DAYS[0]}–{FROZEN_DAYS[1]}")
    ev_legs = pl.read_parquet(eval_dir / "legs.parquet")
    ev_trips = pl.read_parquet(eval_dir / "trips.parquet")
    ev = pl_.apply_frozen(pl_.leg_base_features(ev_legs, ev_trips, DAY0), sp_f, rs_f)
    print(f"  chặng đánh giá: {vn(ev.height)} (ngày {ev['day'].min()} tới {ev['day'].max()}); có nhãn {vn(pl_.labelled(ev).height)}")

    # ---- 7. ghi -----------------------------------------------------------------------------------------------------
    feat.write_parquet(out / "legs_features.parquet")
    ev.write_parquet(out / "eval_features.parquet")
    cell_hour.write_parquet(out / "cell_hour.parquet")
    sp_f.write_parquet(out / "frozen_speed.parquet")
    rs_f.write_parquet(out / "frozen_resid.parquet")
    print(f"\nđã ghi {out}/{{legs_features,eval_features,cell_hour,frozen_speed,frozen_resid}}.parquet; tổng {time.time() - t0:.0f} s")
    print(f"uptime: {uptime()}")


if __name__ == "__main__":
    main()
