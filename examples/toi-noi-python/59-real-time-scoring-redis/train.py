"""Dựng bảng đặc trưng theo lô (nguồn «offline») và huấn luyện mô hình chấm điểm nếu `out/` chưa có.

Mô hình giống bài 57 (`model.fit_calibrated`): LightGBM 100 cây trên ngày 0 đến 22, hiệu chỉnh sigmoid trên ngày 22 đến 30;
ngưỡng gắn cờ toàn cục chọn trên bộ hiệu chỉnh. Bài 59 không dạy lại phần huấn luyện; file này chỉ để lab tự chứa:
chạy lần đầu sẽ sinh `out/model.joblib`, `out/model_meta.json` và `out/table.parquet`.
"""

from datetime import UTC, datetime
from pathlib import Path

import numpy as np
import polars as pl
from sklearn.metrics import average_precision_score

from dataset import TRAIN_END_DAY, VALID_START_DAY
from features import FEATURES, add_features, day_of, trip_table
from model import CAL_START_DAY, HANDLING_COST_VND, LGBM_PARAMS, REVIEW_COST_VND, best_cost_threshold, fit_calibrated
from model_io import ModelLoadError, load_model, save_model
from synthetic_fraud import make_events

SEED = 42
N_TRIPS = 200_000
OUT = Path(__file__).parent / "out"


def load_table() -> tuple[pl.DataFrame, pl.DataFrame]:
    """Trả (events, bảng đặc trưng theo lô). `events` luôn sinh lại (vài giây); bảng lưu ở `out/table.parquet`
    (dựng bằng polars, khoảng nửa phút), xóa thư mục `out/` để dựng lại."""
    data = make_events(seed=SEED, n_trips=N_TRIPS)
    path = OUT / "table.parquet"
    if path.exists():
        return data.events, pl.read_parquet(path)
    OUT.mkdir(exist_ok=True)
    table = add_features(trip_table(data.events)).join(data.labels, on="trip_id", how="left").sort("paid_at")
    table.write_parquet(path)
    return data.events, table


def ensure_model(table: pl.DataFrame):
    """Nạp `out/model.joblib` nếu có; chưa có (hoặc nạp không được) thì huấn luyện (khoảng nửa phút) rồi lưu."""
    try:
        return load_model(OUT)
    except ModelLoadError:
        pass
    day = table.select(day_of("paid_at").alias("d"))["d"].to_numpy()
    y = table["is_fraud"].to_numpy().astype(int)
    tr = np.flatnonzero(day < TRAIN_END_DAY)
    x = table.select(FEATURES)
    base, cal = fit_calibrated(x[tr], y[tr], day[tr])
    cal_idx = np.flatnonzero((day >= CAL_START_DAY) & (day < TRAIN_END_DAY))
    p_cal = cal.predict_proba(x[cal_idx])[:, 1]
    thr, _ = best_cost_threshold(y[cal_idx], p_cal, table["amount_vnd"].to_numpy()[cal_idx].astype(float))  # chọn trên bộ hiệu chỉnh, không nhìn valid
    v = np.flatnonzero(day >= VALID_START_DAY)
    p = cal.predict_proba(x[v])[:, 1]
    meta = {
        "created_at": datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "data": {"seed": SEED, "n_trips": N_TRIPS, "synthetic": True, "valid_from_day": VALID_START_DAY},
        "model": {"kind": "LGBMClassifier + CalibratedClassifierCV(sigmoid)", "params": LGBM_PARAMS},
        "decision": {"tuned_threshold": round(thr, 6), "review_cost_vnd": REVIEW_COST_VND, "handling_cost_vnd": HANDLING_COST_VND,
                     "note": "diem rui ro la dau vao cho nguoi duyet, khong tu khoa tai khoan"},
        "metrics_valid": {"pr_auc": round(float(average_precision_score(y[v], p)), 4), "base_rate": round(float(y[v].mean()), 5), "rows": int(v.size)},
    }
    save_model(cal, meta, OUT)
    return load_model(OUT)
