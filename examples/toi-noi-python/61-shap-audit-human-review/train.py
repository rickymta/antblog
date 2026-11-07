"""Mô hình của bài 57, huấn luyện lại tại chỗ nếu out/model.joblib chưa có (lab tự chứa, không import từ lab khác).

Công thức đúng bài 57, qua `model.fit_calibrated`: LightGBM 100 vòng không trọng số lớp, huấn luyện trên chuyến thanh toán
trước ngày 22, hiệu chỉnh sigmoid trên ngày 22 đến 30 (`FrozenEstimator` giữ nguyên mô hình gốc). Ngưỡng gắn cờ chọn theo
chi phí trên chính bộ hiệu chỉnh (không nhìn cửa sổ đánh giá từ ngày 50). `ensure_model` chỉ huấn luyện khi chưa có file hợp lệ.
"""

from pathlib import Path

import numpy as np
import polars as pl
from sklearn.calibration import CalibratedClassifierCV

import metrics
from features import FEATURES, day_of
from model import CAL_START_DAY, HANDLING_COST_VND, LGBM_PARAMS, REVIEW_COST_VND, fit_calibrated
from model_io import ModelLoadError, ScoringModel, load_model, save_model

TRAIN_END_DAY = 30


def train(table: pl.DataFrame) -> tuple[CalibratedClassifierCV, dict]:
    day = table.select(day_of("paid_at").alias("d"))["d"].to_numpy()
    use = np.flatnonzero(day < TRAIN_END_DAY)
    x = table.select(FEATURES)[use]
    y = table["is_fraud"].to_numpy().astype(int)[use]
    _, cal = fit_calibrated(x, y, day[use], cal_start_day=CAL_START_DAY)
    b = day[use] >= CAL_START_DAY
    amount = table["amount_vnd"].to_numpy().astype(float)[use][b]
    thr, _ = metrics.best_threshold(y[b], cal.predict_proba(x.filter(pl.Series(b)))[:, 1], amount + HANDLING_COST_VND, REVIEW_COST_VND)
    meta = {
        "data": {"synthetic": True, "base_train_days": [0, CAL_START_DAY], "calibration_days": [CAL_START_DAY, TRAIN_END_DAY]},
        "model": {"kind": "LGBMClassifier + CalibratedClassifierCV(sigmoid)", "params": dict(LGBM_PARAMS)},
        "decision": {"tuned_threshold": round(float(thr), 6), "threshold_chosen_on": "bộ hiệu chỉnh (ngày 22 đến 30)",
                     "review_cost_vnd": REVIEW_COST_VND, "handling_cost_vnd": HANDLING_COST_VND,
                     "note": "diem rui ro la dau vao cho nguoi duyet, khong tu khoa tai khoan"},
    }
    return cal, meta


def in_memory(cal: CalibratedClassifierCV, meta: dict, name: str) -> ScoringModel:
    """Bọc mô hình chưa lưu thành ScoringModel (dùng để lặp lại phép đo trên seed khác, không ghi file)."""
    return ScoringModel(cal, {**meta, "feature_list": list(FEATURES), "model_version": name})


def ensure_model(table: pl.DataFrame, out: Path) -> tuple[ScoringModel, bool]:
    """(mô hình, vừa huấn luyện?). File thiếu hoặc không qua kiểm của `load_model` thì huấn luyện lại."""
    try:
        return load_model(out), False
    except ModelLoadError:
        cal, meta = train(table)
        save_model(cal, meta, out)
        return load_model(out), True
