"""LOC-56: ước lượng hiệu quả của một thay đổi trên thị trường hai phía từ dữ liệu của `loc56_ab_switchback` (Rust).

Cách dựng: cùng seed ⇒ cùng dòng khách, nên chạy «mọi người được» (`on`) và «không ai được» (`off`) cho ra chênh lệch toàn cục có cặp,
dùng làm sự thật. Các cách gán còn lại (`ab`, `sb_*`) chỉ cho thấy một nửa khách được thay đổi; ước lượng của chúng được so với sự thật đó.

Mọi số trên dữ liệu mô phỏng. Cả hai thay đổi đều do lab chọn; `priority` chọn vì nhiễu giữa nhóm rõ nhất.
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import polars as pl

from .common import paired_ci, t_crit_975

METRICS = {
    "completed": "tỉ lệ hoàn thành chuyến (cao hơn là tốt)",
    "wait_cens_s": "thời gian chờ có chặn, giây (thấp hơn là tốt): chuyến xong tính tới lúc xe tới; huỷ tính tới lúc huỷ; hết kiên nhẫn tính 300 s",
}
SB_ARMS = ["sb_zt", "sb_t10", "sb_t20"]


def load(dirpath) -> tuple[pl.DataFrame, pl.DataFrame]:
    c = pl.read_csv(f"{dirpath}/customers.csv")
    r = pl.read_csv(f"{dirpath}/runs.csv")
    return c, r


def ols_binary(y: np.ndarray, t: np.ndarray, cluster: np.ndarray | None):
    """Hồi quy y ~ a + b·t. Trả (b = hiệu trung bình có trọng số khách, sai số chuẩn bền theo cụm, số cụm).
    `cluster = None`: mỗi khách một cụm (độc lập, như phân tích A/B ngây thơ). Công thức CR1: G/(G−1) · (N−1)/(N−2)."""
    n = len(y)
    tb = t.mean()
    sxx = np.sum((t - tb) ** 2)
    if sxx == 0 or n < 3:
        return float("nan"), float("nan"), 0
    b = np.sum((t - tb) * (y - y.mean())) / sxx
    a = y.mean() - b * tb
    e = y - a - b * t
    s = (t - tb) * e
    if cluster is None:
        g = n
        meat = np.sum(s**2)
    else:
        uniq, inv = np.unique(cluster, return_inverse=True)
        g = len(uniq)
        sums = np.bincount(inv, weights=s, minlength=g)
        meat = np.sum(sums**2)
    if g < 3:
        return float(b), float("nan"), g
    var = meat / sxx**2 * (g / (g - 1)) * ((n - 1) / (n - 2))
    return float(b), float(np.sqrt(var)), g


@dataclass
class SeedEst:
    seed: int
    truth: float
    ab: tuple[float, float, int]
    sb: dict[str, tuple[float, float, int]]
    n: int


def estimates_for_seed(cust: pl.DataFrame, seed: int, metric: str) -> SeedEst:
    """Sự thật toàn cục và các ước lượng của một seed, cho một chỉ số."""
    d = cust.filter((pl.col("seed") == seed) & pl.col(metric).is_not_null() & pl.col(metric).is_not_nan())
    on = d.filter(pl.col("arm") == "on").sort("customer")
    off = d.filter(pl.col("arm") == "off").sort("customer")
    keep = set(on["customer"].to_list()) & set(off["customer"].to_list())
    on = on.filter(pl.col("customer").is_in(keep))
    off = off.filter(pl.col("customer").is_in(keep))
    truth = float(on[metric].mean() - off[metric].mean())
    ab = d.filter(pl.col("arm") == "ab")
    ab_est = ols_binary(ab[metric].to_numpy().astype(float), ab["treated"].to_numpy().astype(float), None)
    sb = {}
    for arm in SB_ARMS:
        x = d.filter(pl.col("arm") == arm)
        cl = (x["zone"].to_numpy().astype(np.int64) * 1000 + x["window"].to_numpy()).astype(np.int64)
        sb[arm] = ols_binary(x[metric].to_numpy().astype(float), x["treated"].to_numpy().astype(float), cl)
    return SeedEst(seed, truth, ab_est, sb, on.height)


def summarize(cust: pl.DataFrame, metric: str):
    """Trả danh sách (tên, ước lượng từng seed, SE từng seed, số cụm) cùng sự thật từng seed."""
    seeds = sorted(cust["seed"].unique().to_list())
    rows = [estimates_for_seed(cust, s, metric) for s in seeds]
    truth = np.array([r.truth for r in rows])
    out = {"ab (theo khách)": (np.array([r.ab[0] for r in rows]), np.array([r.ab[1] for r in rows]), np.array([r.ab[2] for r in rows]))}
    for arm in SB_ARMS:
        out[arm] = (np.array([r.sb[arm][0] for r in rows]), np.array([r.sb[arm][1] for r in rows]), np.array([r.sb[arm][2] for r in rows]))
    return seeds, truth, out


def report_row(name: str, est: np.ndarray, se: np.ndarray, g: np.ndarray, truth: np.ndarray, digits: int = 3):
    """Một dòng so với sự thật: trung bình ước lượng, thiên lệch có cặp [KTC t 95%], RMSE, độ phủ KTC 95% trong từng seed, % seed sai dấu."""
    ok = np.isfinite(est) & np.isfinite(truth)
    if not ok.any():
        return f"  {name:<18} {'n/a (không đủ đơn vị để ước lượng)':>60}"
    bias_md, lo, hi, _, _ = paired_ci(truth[ok], est[ok])  # est − truth
    rmse = float(np.sqrt(np.mean((est[ok] - truth[ok]) ** 2)))
    n_clusters_ok = np.isfinite(se)
    # KTC 95% của từng seed: est ± t(G−1)·se với G = số cụm (đơn vị switchback, hoặc số khách ở A/B); ít cụm thì t lớn
    crit = np.array([t_crit_975(int(x) - 1) if x > 1 else np.nan for x in g])
    # độ phủ: KTC đó có chứa sự thật của seed đó không
    cover = np.mean(np.abs(est[n_clusters_ok] - truth[n_clusters_ok]) <= crit[n_clusters_ok] * se[n_clusters_ok]) if n_clusters_ok.any() else float("nan")
    sign_wrong = np.mean(np.sign(est[ok]) != np.sign(truth[ok])) if np.abs(truth[ok]).max() > 0 else float("nan")
    # «báo có hiệu quả khi không có»: KTC 95% của seed không chứa 0
    sig = np.mean(np.abs(est[n_clusters_ok]) > crit[n_clusters_ok] * se[n_clusters_ok]) if n_clusters_ok.any() else float("nan")
    f = f"{{:+.{digits}f}}"
    sw = "n/a" if not np.isfinite(sign_wrong) else f"{100 * sign_wrong:.0f}%"
    return (
        f"  {name:<18} {f.format(np.nanmean(est)):>10}  {f.format(bias_md):>10} [{f.format(lo)}, {f.format(hi)}]  {rmse:>8.{digits}f}  "
        f"{100 * cover:>8.0f}%  {100 * sig:>9.0f}%  {sw:>9}"
    )


def seeds_needed(sd: float, delta: float, power_z: float = 0.8416, alpha_z: float = 1.96) -> float:
    """Số «giờ thí nghiệm» (seed) cần để phát hiện hiệu quả `delta` với công suất 80%, khi độ lệch chuẩn của ước lượng mỗi giờ là `sd`."""
    if delta <= 0 or not np.isfinite(sd):
        return float("nan")
    return float(((alpha_z + power_z) * sd / delta) ** 2)


def cumulative_ci(est: np.ndarray, ns=(5, 10, 20, 40)):
    """KTC t 95% của trung bình ước lượng khi dùng n giờ thí nghiệm đầu tiên."""
    out = []
    for n in ns:
        if n > len(est):
            continue
        x = est[:n]
        m = float(x.mean())
        h = t_crit_975(n - 1) * float(x.std(ddof=1)) / np.sqrt(n)
        out.append((n, m, m - h, m + h))
    return out
