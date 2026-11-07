"""Huấn luyện một phiên bản bộ chấm điểm theo công thức của bài 57, với ngày kết thúc và cách lấy nhãn do ta chọn.

Công thức (giữ nguyên bài 57, qua `model.fit_calibrated`): LightGBM 100 vòng không trọng số lớp, huấn luyện trên chuyến
thanh toán trước `end_day - 8`, rồi hiệu chỉnh sigmoid trên 8 ngày cuối của phần dữ liệu. Ngưỡng gắn cờ của mỗi phiên bản
chọn theo chi phí trên chính bộ hiệu chỉnh của nó (không nhìn holdout). Ba phiên bản của bài 60 chỉ khác `end_day` và nhãn:

    v1: end_day 30, nhãn cuối   (cùng công thức mô hình của bài 57)
    v2: end_day 40, nhãn cuối   (huấn luyện lại theo lịch, trước ngày 50 có 10 ngày chờ cho nhãn chín)
    v3: end_day 50, nhãn biết vào ngày 50, KHÔNG chờ  (huấn luyện lại «trên dữ liệu mới nhất»: lỗi nhãn chưa chín của bài 54)
    v3f: đối chứng của v3, đổi ĐÚNG MỘT biến: cùng end_day 50 nhưng nhãn cuối (chỉ dùng ở bước lặp lại ba seed)
"""

from dataclasses import dataclass
from typing import Literal

import numpy as np
import polars as pl
from sklearn.calibration import CalibratedClassifierCV

import metrics
from dataset import at_day
from features import FEATURES, day_of
from model import HANDLING_COST_VND, LGBM_PARAMS, REVIEW_COST_VND, fit_calibrated
from model_io import ScoringModel

CAL_SPAN_DAYS = 8


@dataclass(frozen=True)
class Spec:
    name: str
    end_day: int  # train dùng chuyến thanh toán trước ngày này (tính từ START)
    labels: Literal["final", "known"]  # nhãn cuối, hoặc nhãn đã BIẾT vào ngày `end_day`
    note: str = ""


V1 = Spec("v1", 30, "final", "cùng công thức mô hình bài 57")
V2 = Spec("v2", 40, "final", "huấn luyện lại theo lịch, chờ nhãn chín")
V3 = Spec("v3", 50, "known", "huấn luyện lại trên dữ liệu mới nhất, nhãn chưa chín")
V3F = Spec("v3f", 50, "final", "đối chứng của v3: cùng dữ liệu đến ngày 50, chỉ đổi nhãn sang nhãn cuối")
VX = Spec("vx", 50, "final", "huấn luyện lại sau khi nguồn GPS đổi: học luôn dữ liệu lỗi từ ngày 30")


def paid_day(table: pl.DataFrame) -> np.ndarray:
    return table.select(day_of("paid_at").alias("d"))["d"].to_numpy()


def labels_for(table: pl.DataFrame, spec: Spec) -> np.ndarray:
    """Nhãn 0/1 dùng để huấn luyện. `known`: chuyến gian lận mà `label_known_at` chưa tới ngày `end_day` vẫn là 0."""
    if spec.labels == "final":
        return table["is_fraud"].to_numpy().astype(int)
    known = pl.col("is_fraud") & (pl.col("label_known_at") <= pl.lit(at_day(spec.end_day)))
    return table.select(known.alias("k"))["k"].to_numpy().astype(int)


def train(table: pl.DataFrame, spec: Spec) -> tuple[CalibratedClassifierCV, dict]:
    """Trả (mô hình đã hiệu chỉnh, meta). Meta chưa có sha256/phiên bản thư viện: `save_model` thêm."""
    day = paid_day(table)
    y = labels_for(table, spec)
    use = np.flatnonzero(day < spec.end_day)
    x = table.select(FEATURES)[use]
    d, yy = day[use], y[use]
    cal_start = spec.end_day - CAL_SPAN_DAYS
    _, cal = fit_calibrated(x, yy, d, cal_start_day=cal_start)
    # ngưỡng theo chi phí trên bộ hiệu chỉnh của chính mô hình này (nhãn đúng như lúc huấn luyện)
    b = d >= cal_start
    amount = table["amount_vnd"].to_numpy().astype(float)[use][b]
    thr, _ = metrics.best_threshold(yy[b], cal.predict_proba(x.filter(pl.Series(b)))[:, 1], amount + HANDLING_COST_VND, REVIEW_COST_VND)
    meta = {
        "spec": spec.name,
        "trained_on": {
            "synthetic": True, "end_day": spec.end_day, "base_days": [0, cal_start],
            "calibration_days": [cal_start, spec.end_day], "labels": spec.labels,
            "train_fraud_rate": round(float(yy.mean()), 5), "rows": int(len(use)),
        },
        "model": {"kind": "LGBMClassifier + CalibratedClassifierCV(sigmoid)", "params": {**LGBM_PARAMS}},
        "decision": {"tuned_threshold": round(float(thr), 6), "threshold_chosen_on": "bộ hiệu chỉnh của chính phiên bản này",
                     "review_cost_vnd": REVIEW_COST_VND, "handling_cost_vnd": HANDLING_COST_VND,
                     "note": "diem rui ro la dau vao cho nguoi duyet, khong tu khoa tai khoan"},
    }
    return cal, meta


def impact_share(scoring: ScoringModel, df: pl.DataFrame, *, n: int = 10_000, seed: int = 0) -> dict[str, float]:
    """Phần của từng đặc trưng trong tổng mean|đóng góp| lên điểm thô, trên một mẫu `df`.

    Đóng góp lấy từ TreeSHAP của LightGBM qua `ScoringModel.contributions` (cột cuối là hằng số nền, bỏ đi). Đây là thước đo
    «mô hình dựa vào đặc trưng này bao nhiêu khi chấm», dùng thay gain lúc huấn luyện vì gain có thể xếp một đặc trưng rất thấp
    dù nó đẩy điểm nhiều (bài 61)."""
    sample = df.sample(n=min(n, df.height), seed=seed)
    contrib = np.abs(scoring.contributions(sample)[:, :-1]).mean(axis=0)
    return {f: float(v / contrib.sum()) for f, v in zip(scoring.feature_list, contrib)}


def gain_share(scoring: ScoringModel) -> dict[str, float]:
    """Phần gain lúc huấn luyện, chỉ để so với impact_share."""
    gain = scoring.booster.feature_importance("gain")
    return {f: float(g / gain.sum()) for f, g in zip(scoring.feature_list, gain)}


def in_memory(cal: CalibratedClassifierCV, meta: dict, name: str) -> ScoringModel:
    """Bọc mô hình chưa lưu thành ScoringModel (để lặp lại phép đo trên seed khác, không ghi file)."""
    return ScoringModel(cal, {**meta, "feature_list": list(FEATURES), "model_version": name})
