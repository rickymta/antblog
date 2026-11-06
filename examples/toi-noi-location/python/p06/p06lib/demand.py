"""LOC-55: dự báo nhu cầu theo ô H3. Bảng đếm (ô, ngày, giờ), đặc trưng chỉ từ dữ liệu cách ≥ 7 ngày, baseline «cùng giờ tuần trước»,
mô hình LightGBM Poisson, chấm điểm theo thời gian. Dữ liệu mô phỏng: nhu cầu do máy sinh của lab tạo ra, kể cả «sự kiện ẩn» không dự báo được."""

from __future__ import annotations

import h3
import lightgbm as lgb
import numpy as np
import polars as pl

from .common import CENTRE, haversine_m

H3_RES = 8
DAYS_PER_WEEK = 7


def request_cells(trips: pl.DataFrame, day0: str = "2026-10-05") -> pl.DataFrame:
    """Mỗi yêu cầu đặt xe một dòng: ngày (đếm từ ngày 0), giờ địa phương, ô H3 độ phân giải 8 của điểm ghim."""
    from datetime import date

    from .pipeline import VN_OFFSET

    d0 = date.fromisoformat(day0)
    lt = pl.col("request_time").dt.replace_time_zone(None) + VN_OFFSET
    df = trips.select(
        (lt.dt.date() - pl.lit(d0)).dt.total_days().cast(pl.Int32).alias("day"),
        lt.dt.hour().cast(pl.Int32).alias("hour"),
        (pl.col("pin_lat_e7") / 1e7).alias("lat"),
        (pl.col("pin_lng_e7") / 1e7).alias("lng"),
    )
    lat, lng = df["lat"].to_numpy(), df["lng"].to_numpy()
    cells = [h3.latlng_to_cell(float(a), float(b), H3_RES) for a, b in zip(lat, lng)]
    return df.with_columns(pl.Series("cell", cells, dtype=pl.Utf8))


class Panel:
    """Tensor đếm N[ô, ngày, giờ] cho tập ô cố định (các ô có yêu cầu trong cửa sổ huấn luyện)."""

    def __init__(self, req: pl.DataFrame, universe_days: tuple[int, int], n_days: int):
        known = req.filter(pl.col("day").is_between(*universe_days))["cell"].unique().sort().to_list()
        self.cells = known
        self.idx = {c: i for i, c in enumerate(known)}
        self.n_days = n_days
        self.N = np.zeros((len(known), n_days, 24), dtype=np.float32)
        self.outside = np.zeros((n_days, 24), dtype=np.float32)  # yêu cầu rơi vào ô ngoài tập đã biết
        g = req.group_by(["cell", "day", "hour"]).len()
        for c, d, h, n in g.iter_rows():
            if d >= n_days:
                continue
            i = self.idx.get(c)
            if i is None:
                self.outside[d, h] += n
            else:
                self.N[i, d, h] += n
        ll = np.array([h3.cell_to_latlng(c) for c in known])
        self.lat, self.lng = ll[:, 0], ll[:, 1]
        self.dist_centre = haversine_m(self.lat, self.lng, CENTRE[0], CENTRE[1])
        self.city = self.N.sum(0)  # [ngày, giờ] (chỉ gồm ô đã biết)

    # ---- đặc trưng: chỉ dùng dữ liệu của ngày ≤ d − 7 -----------------------------------------------------------------------
    def features(self, days: list[int]):
        """Ma trận đặc trưng cho mọi (ô, ngày ∈ days, giờ). Chỉ dùng độ trễ 7 và 14 ngày (và cửa sổ kết thúc ở d − 7)."""
        C = len(self.cells)
        rows, y, meta = [], [], []
        for d in days:
            assert d - 14 >= 0, "cần ít nhất 14 ngày lịch sử"
            l7, l14 = self.N[:, d - 7, :], self.N[:, d - 14, :]
            city7, city14 = self.city[d - 7, :], self.city[d - 14, :]
            # tổng nhu cầu mỗi ô trong 14 ngày kết thúc ở d − 7 (cửa sổ [d−20, d−7] cắt ở 0)
            lo = max(0, d - 20)
            vol = self.N[:, lo : d - 6, :].sum((1, 2))
            share = vol / max(vol.sum(), 1.0)
            hour_share = self.N[:, lo : d - 6, :].sum(1)  # [ô, giờ]
            hour_share = hour_share / np.maximum(hour_share.sum(0, keepdims=True), 1.0)
            dow = d % 7
            for h in range(24):
                X = np.column_stack(
                    [
                        np.full(C, h, np.float32),
                        np.full(C, dow, np.float32),
                        np.full(C, float(dow >= 5), np.float32),
                        self.lat.astype(np.float32),
                        self.lng.astype(np.float32),
                        self.dist_centre.astype(np.float32),
                        l7[:, h],
                        l14[:, h],
                        (l7[:, h] + l14[:, h]) / 2,
                        np.full(C, city7[h], np.float32),
                        np.full(C, (city7[h] + city14[h]) / 2, np.float32),
                        vol.astype(np.float32),
                        share.astype(np.float32),
                        hour_share[:, h].astype(np.float32),
                        (share * city7[h]).astype(np.float32),
                    ]
                )
                rows.append(X)
                y.append(self.N[:, d, h] if d < self.n_days else np.full(C, np.nan, np.float32))
                meta.append(np.column_stack([np.arange(C), np.full(C, d), np.full(C, h)]))
        return np.vstack(rows), np.concatenate(y), np.vstack(meta).astype(np.int64)


FEATURE_NAMES = [
    "hour", "dow", "is_weekend", "cell_lat", "cell_lng", "cell_dist_centre_m", "lag7", "lag14", "lag_mean", "city_lag7",
    "city_lag_mean", "cell_vol_14d", "cell_share", "cell_hour_share", "share_x_city_lag7",
]
LAG7, LAG_MEAN, SHARE_X_CITY = 6, 8, 14


def poisson_deviance(y, mu, eps=0.01):
    """Độ lệch Poisson trung bình trên mỗi ô-giờ (thấp hơn là tốt). `mu` kẹp dưới `eps` vì baseline hay dự báo đúng 0."""
    mu = np.maximum(mu, eps)
    term = np.where(y > 0, y * np.log(np.maximum(y, 1e-12) / mu), 0.0)
    return float(2 * np.mean(term - (y - mu)))


def fit_poisson(Xtr, ytr, Xva, yva, **over) -> lgb.LGBMRegressor:
    p = dict(objective="poisson", learning_rate=0.05, n_estimators=1500, num_leaves=15, min_child_samples=50, subsample=0.8,
             subsample_freq=1, colsample_bytree=0.8, reg_lambda=1.0, random_state=7, n_jobs=4, deterministic=True, force_row_wise=True, verbose=-1)
    p.update(over)
    m = lgb.LGBMRegressor(**p)
    m.fit(Xtr, ytr, eval_X=Xva, eval_y=yva, eval_metric="poisson", callbacks=[lgb.early_stopping(50, verbose=False)])
    return m
