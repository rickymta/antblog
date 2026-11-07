"""Đường dẫn, hằng số, thống kê dùng chung. Mọi con số sinh ra từ dữ liệu mô phỏng của simulator (LOC-30/51)."""

from __future__ import annotations

import subprocess
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[1]  # python/p06
ART = ROOT / "artifacts"
LAB = ROOT.parents[1]  # examples/toi-noi-location

# Chợ Bến Thành: tâm của các vùng trung tâm / vành trong / ngoại vi, giống matcher::eval::Zones (P05)
CENTRE = (10.7725, 106.6980)
ZONE_NAMES = ["trung tâm (<3 km)", "vành trong (3–6 km)", "ngoại vi (≥6 km)"]

# Chia theo thời gian (ngày đếm từ ngày 0 = thứ Hai 2026-10-05). Huấn luyện / kiểm định (chọn tham số) / kiểm tra (báo cáo).
# 21 ngày mô phỏng: huấn luyện 2 tuần đầy đủ, kiểm định thứ Hai–thứ Tư, kiểm tra thứ Năm–Chủ nhật (2 ngày thường + 2 ngày cuối tuần).
TRAIN_DAYS = (0, 13)
VAL_DAYS = (14, 16)
TEST_DAYS = (17, 20)
FROZEN_DAYS = (14, 20)  # cửa sổ của bảng tra đóng băng cho các ngày sau dữ liệu (tuần cuối)
DAY0 = "2026-10-05"

# Đặc trưng đưa vào mô hình ETA, theo thứ tự cố định (thứ tự này là hợp đồng với phía Rust, xem eta-model).
FEATURES = [
    "route_eta_s",
    "route_dist_m",
    "n_edges",
    "leg_kind",
    "tod",
    "dow",
    "is_weekend",
    "o_lat",
    "o_lng",
    "d_lat",
    "d_lng",
    "crow_m",
    "detour",
    "o_dist_centre_m",
    "d_dist_centre_m",
    "req_to_depart_s",
    "hist_speed_o",
    "hist_speed_d",
    "hist_resid_o",
]


def vn(n) -> str:
    """Số nguyên kiểu Việt Nam: dấu chấm ngăn cách hàng nghìn."""
    return f"{int(n):,}".replace(",", ".")


def uptime() -> str:
    try:
        return subprocess.run(["uptime"], capture_output=True, text=True, check=True).stdout.strip()
    except Exception as e:  # pragma: no cover
        return f"(không chạy được uptime: {e})"


def haversine_m(lat1, lng1, lat2, lng2):
    """Khoảng cách chim bay (mét) trên mảng numpy, cùng công thức với geo_core::haversine_m (bán kính 6.371.008,8 m)."""
    r = 6_371_008.8
    p1, p2 = np.radians(lat1), np.radians(lat2)
    dphi = p2 - p1
    dl = np.radians(lng2) - np.radians(lng1)
    a = np.sin(dphi / 2) ** 2 + np.cos(p1) * np.cos(p2) * np.sin(dl / 2) ** 2
    return 2 * r * np.arcsin(np.sqrt(a))


def zone_of(dist_centre_m):
    d = np.asarray(dist_centre_m)
    return np.where(d < 3000, 0, np.where(d < 6000, 1, 2))


# ---- thống kê -------------------------------------------------------------------------------------------------------

_T975 = {
    1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365, 8: 2.306, 9: 2.262, 10: 2.228,
    11: 2.201, 12: 2.179, 13: 2.160, 14: 2.145, 15: 2.131, 16: 2.120, 17: 2.110, 18: 2.101, 19: 2.093,
    20: 2.086, 21: 2.080, 22: 2.074, 23: 2.069, 24: 2.064, 25: 2.060, 26: 2.056, 27: 2.052, 28: 2.048,
    29: 2.045, 30: 2.042,
}


def t_crit_975(df: int) -> float:
    """Giá trị tới hạn hai phía 95% của phân phối t Student; cùng bảng với matcher::eval::t_crit_975 (Rust, P05)."""
    if df <= 0:
        return float("nan")
    if df <= 30:
        return _T975[df]
    if df <= 40:
        return 2.021
    if df <= 60:
        return 2.000
    if df <= 120:
        return 1.980
    return 1.960


def paired_ci(a, b):
    """Hiệu có cặp b − a: (trung bình hiệu, KTC 95% dưới, trên, số cặp b<a, số cặp b>a). KTC theo t Student trên hiệu từng cặp."""
    a, b = np.asarray(a, float), np.asarray(b, float)
    ok = np.isfinite(a) & np.isfinite(b)
    d = (b - a)[ok]
    n = len(d)
    md = float(d.mean()) if n else float("nan")
    sd = float(d.std(ddof=1)) if n > 1 else float("nan")
    half = t_crit_975(n - 1) * sd / np.sqrt(n) if n > 1 else float("nan")
    return md, md - half, md + half, int((d < 0).sum()), int((d > 0).sum())


def verdict(lo: float, hi: float, lower_better: bool = True, md: float | None = None) -> str:
    if not (lo > 0 or hi < 0):
        return "không phân biệt được"
    mean = (lo + hi) / 2 if md is None else md
    return "TỐT HƠN" if (mean < 0) == lower_better else "XẤU HƠN"


def cluster_bootstrap_diff(err_a, err_b, clusters, n_boot=2000, seed=7):
    """Bootstrap theo cụm (cùng một giờ-ngày là một cụm) cho hiệu MAE: mean|err_b| − mean|err_a|.
    Trả (hiệu quan sát, KTC 95% phân vị 2,5–97,5). Cụm tránh coi hàng chục nghìn chặng cùng một giờ là độc lập."""
    err_a, err_b = np.abs(np.asarray(err_a, float)), np.abs(np.asarray(err_b, float))
    clusters = np.asarray(clusters)
    uniq, inv = np.unique(clusters, return_inverse=True)
    k = len(uniq)
    sa = np.bincount(inv, weights=err_a, minlength=k)
    sb = np.bincount(inv, weights=err_b, minlength=k)
    cnt = np.bincount(inv, minlength=k).astype(float)
    rng = np.random.default_rng(seed)
    idx = rng.integers(0, k, size=(n_boot, k))
    num = (sb[idx].sum(1) - sa[idx].sum(1)) / cnt[idx].sum(1)
    obs = float(err_b.mean() - err_a.mean())
    lo, hi = np.percentile(num, [2.5, 97.5])
    return obs, float(lo), float(hi)


def pct(x, p):
    x = np.sort(np.asarray(x, float))
    if len(x) == 0:
        return float("nan")
    idx = int(np.ceil(p / 100 * len(x))) - 1  # nearest-rank như simulator::metrics::percentile
    return float(x[max(0, min(idx, len(x) - 1))])


def day_index(dt_series_local_date, day0: str = DAY0):
    """Ngày dạng số đếm từ ngày 0 (polars Series kiểu Date → Int)."""
    import polars as pl
    from datetime import date

    d0 = date.fromisoformat(day0)
    return (dt_series_local_date - pl.lit(d0)).dt.total_days()
