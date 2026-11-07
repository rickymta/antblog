"""LightGBM cho ETA: dự đoán PHẦN DƯ (thời gian thật − ETA của route-engine) hay tổng thời gian. Dữ liệu mô phỏng: chỉ chứng minh phương pháp."""

from __future__ import annotations

import lightgbm as lgb
import numpy as np
import polars as pl

from .common import FEATURES, haversine_m  # noqa: F401

SEED = 7

# Cấu hình cố định, khai báo TRƯỚC khi đo. Cùng một cấu hình dùng cho mọi biến thể so sánh.
BASE_PARAMS = dict(
    objective="regression_l1",  # chỉ số báo cáo là MAE nên tối ưu trực tiếp độ lệch tuyệt đối
    learning_rate=0.05,
    n_estimators=2000,
    num_leaves=31,
    min_child_samples=50,
    subsample=0.8,
    subsample_freq=1,
    colsample_bytree=0.8,
    reg_lambda=1.0,
    random_state=SEED,
    n_jobs=4,
    deterministic=True,  # cùng dữ liệu, cùng seed ⇒ cùng cây (kể cả khi chạy đa luồng)
    force_row_wise=True,
    verbose=-1,
)

# Lưới nhỏ, khai báo trước; chọn theo MAE trên tập kiểm định (ngày 10–11), tập kiểm tra chỉ xem một lần.
GRID = [dict(num_leaves=nl, min_child_samples=mc) for nl in (15, 31, 63) for mc in (20, 100)]


def fit(Xtr, ytr, Xva, yva, eval_metric="l1", **over) -> lgb.LGBMRegressor:
    params = {**BASE_PARAMS, **over}
    m = lgb.LGBMRegressor(**params)
    m.fit(
        Xtr,
        ytr,
        eval_X=[(Xva, yva)][0][0],
        eval_y=yva,
        eval_metric=eval_metric,
        callbacks=[lgb.early_stopping(50, verbose=False)],
    )
    return m


def mae(a, b) -> float:
    return float(np.mean(np.abs(np.asarray(a, float) - np.asarray(b, float))))


def arrays(df: pl.DataFrame, features: list[str]):
    X = df.select([pl.col(c).cast(pl.Float64) for c in features]).to_numpy().astype(np.float32)
    return X, df["route_eta_s"].to_numpy(), df["actual_s"].to_numpy()


def fit_residual(train: pl.DataFrame, val: pl.DataFrame, features: list[str], **over) -> lgb.LGBMRegressor:
    """Huấn luyện trên `actual − route_eta`. Dự đoán tổng = `route_eta + dự đoán phần dư`."""
    Xt, et, at = arrays(train, features)
    Xv, ev, av = arrays(val, features)
    return fit(Xt, at - et, Xv, av - ev, **over)


def fit_total(train: pl.DataFrame, val: pl.DataFrame, features: list[str], **over) -> lgb.LGBMRegressor:
    """Huấn luyện thẳng trên `actual` (route_eta vẫn là một đặc trưng, nhưng mô hình phải tự học lại thang thời gian)."""
    Xt, et, at = arrays(train, features)
    Xv, ev, av = arrays(val, features)
    return fit(Xt, at, Xv, av, **over)


def predict_total(model: lgb.LGBMRegressor, df: pl.DataFrame, features: list[str], residual: bool = True) -> np.ndarray:
    X, eta, _ = arrays(df, features)
    p = model.predict(X)
    return eta + p if residual else p
