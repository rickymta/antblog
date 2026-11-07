"""LOC-55: dự báo nhu cầu theo ô H3 (độ phân giải 8), kiểm tra theo thời gian, ô thưa, xuất bảng cho LOC-49 (trên dữ liệu mô phỏng).

Chạy từ gốc repo (sau LOC-51 sinh dữ liệu):
  uv run --project examples/toi-noi-location/python/p06 python examples/toi-noi-location/python/p06/loc55_demand_forecast.py

Chia theo thời gian: huấn luyện ngày 14–27, kiểm định 28–34 (chọn số cây), KIỂM TRA 35–41 (xem một lần). Đặc trưng của ngày d chỉ dùng dữ
liệu ngày ≤ d − 7 (dự báo trước một tuần), nên bảng dự báo cho ngày 42 trở đi dựng được từ dữ liệu có sẵn.
Nhu cầu do máy sinh của lab tạo ra (giờ, điểm nóng, cuối tuần, cú sốc theo ngày, sự kiện ẩn). Mô hình khôi phục máy sinh đó, không phải nhu cầu thật.
"""

from __future__ import annotations

import argparse
import json
import time
import warnings
from pathlib import Path

import numpy as np
import polars as pl

from p06lib import demand as dm
from p06lib.common import ART, cluster_bootstrap_diff, paired_ci, uptime, vn
from p06lib.demand import FEATURE_NAMES, LAG7, LAG_MEAN, SHARE_X_CITY

warnings.filterwarnings("ignore")
TRAIN = list(range(14, 28))
VAL = list(range(28, 35))
TEST = list(range(35, 42))
# giờ nhẹ tải và vừa tải (13h, 20h) để tài xế còn rảnh mà gợi ý; thêm 18h (cao điểm, thành phố quá tải) để thấy khi nào gợi ý vô nghĩa
SCENARIOS = [(44, 13, "thu_tu_13h"), (44, 20, "thu_tu_20h"), (47, 20, "thu_bay_20h"), (44, 18, "thu_tu_18h")]


def metrics(y, mu, clusters):
    return {
        "dev": dm.poisson_deviance(y, mu),
        "mae": float(np.mean(np.abs(y - mu))),
        "rmse": float(np.sqrt(np.mean((y - mu) ** 2))),
        "bias": float(np.mean(mu - y)),
    }


def parent_agg(meta, vals, cells):
    """Gộp dự báo/thực tế lên ô cha (độ phân giải 7): MAE theo ô cha-ngày-giờ."""
    import h3

    parent = np.array([int(h3.str_to_int(h3.cell_to_parent(c, 7))) for c in cells])
    cid = parent[meta[:, 0]]
    key = (cid.astype(np.int64) * 1000 + meta[:, 1] * 24 + meta[:, 2])
    uniq, inv = np.unique(key, return_inverse=True)
    return uniq, inv


def topk_recall(meta, y, mu, k=10, min_total=15):
    """Với mỗi (ngày, giờ) có ≥ min_total yêu cầu: tỉ lệ yêu cầu rơi vào k ô được dự báo cao nhất (so với k ô thật cao nhất)."""
    key = meta[:, 1] * 24 + meta[:, 2]
    order = np.argsort(key, kind="stable")
    key_s, y_s, mu_s = key[order], y[order], mu[order]
    bounds = np.flatnonzero(np.diff(key_s)) + 1
    cap_pred, cap_best = [], []
    for ys, ms in zip(np.split(y_s, bounds), np.split(mu_s, bounds)):
        if ys.sum() < min_total:
            continue
        cap_pred.append(ys[np.argsort(-ms, kind="stable")[:k]].sum() / ys.sum())
        cap_best.append(np.sort(ys)[::-1][:k].sum() / ys.sum())
    return float(np.mean(cap_pred)), float(np.mean(cap_best)), len(cap_pred)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--art", default=str(ART))
    a = ap.parse_args()
    art = Path(a.art)
    t0 = time.time()
    print("== LOC-55: dự báo nhu cầu theo ô H3 (trên dữ liệu mô phỏng) ==")
    print("DỮ LIỆU TỔNG HỢP: nhu cầu do máy sinh của lab tạo ra; mô hình khôi phục máy sinh đó, không chứng minh gì về nhu cầu thật.")
    print(f"uptime: {uptime()}")
    trips = pl.read_parquet(art / "data" / "trips.parquet")
    manifest = json.loads((art / "data" / "hidden" / "manifest.json").read_text())
    req = dm.request_cells(trips)
    n_days = int(req["day"].max()) + 1
    print(f"yêu cầu đặt xe: {vn(req.height)} trong {n_days} ngày (ngày 0 = thứ Hai 2026-10-05); ô H3 độ phân giải {dm.H3_RES} (cạnh ≈ 461 m)")
    panel = dm.Panel(req, (0, 27), n_days)
    C = len(panel.cells)
    test_req = req.filter(pl.col("day").is_between(TEST[0], TEST[-1]))
    outside = float(panel.outside[TEST[0] : TEST[-1] + 1].sum())
    print(f"tập ô đã biết (có yêu cầu trong ngày 0–27): {C}; yêu cầu ngày kiểm tra rơi ngoài tập ô đó: {int(outside)} / {vn(test_req.height)} ({100 * outside / test_req.height:.2f}%)")
    print(f"chia theo thời gian: huấn luyện ngày {TRAIN[0]}–{TRAIN[-1]}, kiểm định {VAL[0]}–{VAL[-1]}, kiểm tra {TEST[0]}–{TEST[-1]}; đặc trưng chỉ từ ngày ≤ d − 7")
    ev_days = [e["day"] for e in manifest["hidden_events"]]
    ev_in = [e for e in manifest["hidden_events"] if TEST[0] <= e["day"] <= TEST[-1]]
    print(f"(sự thật ẩn, chỉ để chấm điểm) sự kiện ẩn trong ngày kiểm tra: {[(e['day'], f'điểm nóng {e['hotspot']}', f'{e['from_h']}–{e['to_h']}h ×{e['mult']}') for e in ev_in]}; tất cả {len(ev_days)} sự kiện ở ngày {ev_days}")

    Xtr, ytr, mtr = panel.features(TRAIN)
    Xva, yva, mva = panel.features(VAL)
    Xte, yte, mte = panel.features(TEST)
    print(f"số dòng (ô × giờ): huấn luyện {vn(len(ytr))}, kiểm định {vn(len(yva))}, kiểm tra {vn(len(yte))}; tỉ lệ dòng có nhu cầu > 0 trong kiểm tra: {100 * np.mean(yte > 0):.1f}% (đa số ô-giờ bằng 0: dữ liệu thưa)")
    m = dm.fit_poisson(Xtr, ytr, Xva, yva)
    print(f"LightGBM Poisson: {m.best_iteration_} cây (dừng sớm theo kiểm định)")

    mu_model = m.predict(Xte)
    preds = {
        "cùng giờ tuần trước (baseline)": Xte[:, LAG7],
        "trung bình 2 tuần trước": Xte[:, LAG_MEAN],
        "tỉ phần ô × tổng thành phố tuần trước": Xte[:, SHARE_X_CITY],
        "LightGBM Poisson": mu_model,
    }
    clusters = (mte[:, 1] * 24 + mte[:, 2]).astype(np.int64)
    print(f"\n-- 1. tập KIỂM TRA ({vn(len(yte))} ô-giờ; {vn(int(yte.sum()))} yêu cầu): sai số theo ô-giờ --")
    print(f"  {'phương pháp':<42} {'độ lệch Poisson':>16} {'MAE':>8} {'RMSE':>8} {'thiên lệch (dự báo − thật)':>27} {'tổng dự báo / tổng thật':>24}")
    for nm, mu in preds.items():
        s = metrics(yte, mu, clusters)
        print(f"  {nm:<42} {s['dev']:>16.4f} {s['mae']:>8.4f} {s['rmse']:>8.4f} {s['bias']:>+27.5f} {mu.sum() / yte.sum():>24.3f}")
    print("  (độ lệch Poisson: dự báo kẹp dưới 0,01 vì baseline hay dự báo đúng 0; thấp hơn là tốt)")

    # hiệu có cặp, bootstrap theo cụm (ngày, giờ)
    print("\n  hiệu MAE và hiệu độ lệch giữa mô hình và baseline «cùng giờ tuần trước», bootstrap theo cụm ngày-giờ (2.000 lần):")
    base = preds["cùng giờ tuần trước (baseline)"]
    obs, lo, hi = cluster_bootstrap_diff(yte - base, yte - mu_model, clusters)
    print(f"    MAE (mô hình − baseline): {obs:+.5f} [{lo:+.5f}, {hi:+.5f}]  {'TỐT HƠN' if hi < 0 else 'XẤU HƠN' if lo > 0 else 'không phân biệt được'}")
    dev_row = lambda mu: 2 * (np.where(yte > 0, yte * np.log(np.maximum(yte, 1e-12) / np.maximum(mu, 0.01)), 0.0) - (yte - np.maximum(mu, 0.01)))
    dd = dev_row(mu_model) - dev_row(base)
    uniq, inv = np.unique(clusters, return_inverse=True)
    sums, cnt = np.bincount(inv, weights=dd), np.bincount(inv).astype(float)
    rng = np.random.default_rng(5)
    idx = rng.integers(0, len(uniq), size=(2000, len(uniq)))
    bs = sums[idx].sum(1) / cnt[idx].sum(1)
    lo, hi = np.percentile(bs, [2.5, 97.5])
    print(f"    độ lệch Poisson (mô hình − baseline): {dd.mean():+.5f} [{lo:+.5f}, {hi:+.5f}]  {'TỐT HƠN' if hi < 0 else 'XẤU HƠN' if lo > 0 else 'không phân biệt được'}")

    # ---- cấp ô cha và tổng thành phố ---------------------------------------------------------------------------------------------
    print("\n-- 2. gộp lên cấp thô hơn (cùng các dự báo): ô cha H3 độ phân giải 7, và tổng cả thành phố theo giờ --")
    uniq_p, inv_p = parent_agg(mte, yte, panel.cells)
    print(f"  {'phương pháp':<42} {'MAE ô cha-giờ':>14} {'MAE tổng thành phố-giờ':>24} {'sai số tương đối tổng (|dự báo−thật|/thật)':>46}")
    for nm, mu in preds.items():
        yp, mp = np.bincount(inv_p, weights=yte), np.bincount(inv_p, weights=mu)
        yc, mc = np.bincount(inv, weights=yte), np.bincount(inv, weights=mu)
        print(f"  {nm:<42} {np.mean(np.abs(yp - mp)):>14.4f} {np.mean(np.abs(yc - mc)):>24.3f} {100 * np.sum(np.abs(yc - mc)) / np.sum(yc):>45.1f}%")

    # ---- ô thưa ------------------------------------------------------------------------------------------------------------------------
    vol = panel.N[:, TRAIN[0] : TRAIN[-1] + 1, :].sum((1, 2))
    per_day = vol / len(TRAIN)
    strata = [("ô thưa: < 1 yêu cầu/ngày", per_day < 1), ("ô vừa: 1–5 yêu cầu/ngày", (per_day >= 1) & (per_day < 5)), ("ô dày: ≥ 5 yêu cầu/ngày", per_day >= 5)]
    print("\n-- 3. ô thưa: chia ô theo nhu cầu trung bình trong cửa sổ huấn luyện (ngày 14–27) --")
    print(f"  {'nhóm':<28} {'số ô':>5} {'% yêu cầu kiểm tra':>19}   " + "   ".join(f"{n[:16]:>16} MAE / lệch" for n in ["baseline tuần trước", "tỉ phần×tổng", "LightGBM"]))
    cell_of_row = mte[:, 0]
    for nm, msk in strata:
        rows = msk[cell_of_row]
        share = yte[rows].sum() / yte.sum()
        cols = []
        for key in ["cùng giờ tuần trước (baseline)", "tỉ phần ô × tổng thành phố tuần trước", "LightGBM Poisson"]:
            mu = preds[key][rows]
            cols.append(f"{np.mean(np.abs(yte[rows] - mu)):.4f} / {dm.poisson_deviance(yte[rows], mu):.4f}")
        print(f"  {nm:<28} {int(msk.sum()):>5} {100 * share:>18.1f}%   " + "   ".join(f"{c:>26}" for c in cols))
    pos = yte > 0
    print(f"  baseline «tuần trước» mù ở ô-giờ có nhu cầu: lag7 = 0 ở {100 * np.mean(base[pos] == 0):.1f}% số ô-giờ có yêu cầu (mô hình dự báo > 0,01 ở {100 * np.mean(mu_model[pos] > 0.01):.1f}% số đó)")
    print("  top-10 ô: tỉ lệ yêu cầu rơi vào 10 ô được dự báo cao nhất (so với 10 ô thật cao nhất), trung bình các giờ có ≥ 15 yêu cầu:")
    for nm, mu in preds.items():
        cp, cb, nh = topk_recall(mte, yte, mu)
        print(f"    {nm:<42} {100 * cp:5.1f}%  (trần {100 * cb:.1f}%, {nh} giờ)")

    # ---- sự kiện ẩn ---------------------------------------------------------------------------------------------------------------
    print("\n-- 4. cú sốc không dự báo được: ô-giờ trong sự kiện ẩn so với phần còn lại của kiểm tra --")
    dayk, hourk = mte[:, 1], mte[:, 2]
    evmask = np.zeros(len(yte), bool)
    for e in ev_in:
        evmask |= (dayk == e["day"]) & (hourk >= e["from_h"]) & (hourk < e["to_h"])
    if evmask.any():
        short = ["tuần trước", "TB 2 tuần", "tỉ phần×tổng", "LightGBM"]
        for nm, msk in [("trong sự kiện (cả thành phố, giờ có sự kiện)", evmask), ("ngoài sự kiện", ~evmask)]:
            print(f"  {nm:<46}: {vn(int(msk.sum()))} ô-giờ; tổng thật {int(yte[msk].sum())}; " + "; ".join(f"{sn} MAE {np.mean(np.abs(yte[msk] - v[msk])):.4f}" for sn, v in zip(short, preds.values())))
        # ô bị ảnh hưởng: ô nằm trong top theo nhu cầu ngày đó
        print("  (chỉ ghi chú: sự kiện nhân nhu cầu ×3 ở một điểm nóng trong 4 giờ tối; không có đặc trưng nào biết trước, nên mọi phương pháp đều trượt ở đó)")
    else:
        print("  không có sự kiện ẩn trong các ngày kiểm tra")

    # ---- xuất bảng cho LOC-49 -----------------------------------------------------------------------------------------------------
    out_dir = art / "demand"
    out_dir.mkdir(parents=True, exist_ok=True)
    print("\n-- 5. xuất bảng nhu cầu cho LOC-49 (ô H3 độ phân giải 8; per_hour = số yêu cầu kỳ vọng trong một giờ) --")
    train_share = panel.N[:, 14:28, :].sum((1, 2))
    train_share = train_share / train_share.sum()
    entries = []
    for day, hour, name in SCENARIOS:
        X, _, meta = panel.features([day])
        sel = meta[:, 2] == hour
        mu = m.predict(X[sel])
        lag7 = X[sel][:, LAG7]
        total = float(mu.sum())
        tabs = {
            "model": mu,
            "lastweek": lag7,
            # bảng cố định kiểu LOC-49: chỉ phụ thuộc vị trí (tỉ phần nhu cầu cả ngày-giờ), cùng tổng với dự báo của mô hình để không bị thiệt về mức
            "fixed": train_share * total,
        }
        files = {}
        for k, v in tabs.items():
            path = out_dir / f"{name}_{k}.csv"
            lines = ["cell,per_hour"] + [f"{panel.cells[i]},{v[i]:.6f}" for i in range(C) if v[i] > 0]
            path.write_text("\n".join(lines) + "\n")
            files[k] = str(path.relative_to(art))
        entries.append({"name": name, "day": day, "hour": hour, "tables": files})
        print(f"  ngày {day} (thứ {['Hai', 'Ba', 'Tư', 'Năm', 'Sáu', 'Bảy', 'Chủ nhật'][day % 7]}) {hour:02d}h: dự báo mô hình tổng {total:.1f} yêu cầu/giờ trên {int((mu > 0.01).sum())} ô có dự báo > 0,01; tuần trước {lag7.sum():.0f}; bảng cố định cùng tổng")
        top = np.argsort(-mu)[:3]
        print("    3 ô cao nhất theo mô hình: " + ", ".join(f"{panel.cells[i]} ({mu[i]:.2f}/giờ; tuần trước {lag7[i]:.0f})" for i in top))
    ev_future = [(e["day"], e["hotspot"], e["from_h"], e["to_h"]) for e in manifest["hidden_events"] if e["day"] >= 42]
    (out_dir / "scenarios.json").write_text(json.dumps({"res": dm.H3_RES, "world_seed": manifest["world_seed"], "scenarios": entries}, indent=2))
    print(f"  đã ghi {out_dir}/scenarios.json và các bảng CSV; sự kiện ẩn rơi vào ngày ≥ 42 (chỉ để chấm điểm): {ev_future}")
    print(f"\ntổng {time.time() - t0:.0f} s; uptime: {uptime()}")


if __name__ == "__main__":
    main()
