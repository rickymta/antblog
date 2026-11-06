"""Pipeline scikit-learn + LightGBM cho điểm gian lận: dựng mô hình, chia theo thời gian, hiệu chỉnh, quyết định theo chi phí."""

from collections.abc import Iterator, Sequence

import lightgbm as lgb
import numpy as np
import polars as pl
from sklearn.calibration import CalibratedClassifierCV
from sklearn.compose import ColumnTransformer
from sklearn.frozen import FrozenEstimator
from sklearn.pipeline import Pipeline
from sklearn.preprocessing import FunctionTransformer, StandardScaler

from features import FEATURES

# Siêu tham số LightGBM của lab. `n_estimators = 100` chọn bằng cross-validation theo thời gian ở main.py (không nhìn valid).
# n_jobs, deterministic và force_row_wise để kết quả lặp lại được giữa các lần chạy trên cùng máy.
LGBM_PARAMS = dict(
    n_estimators=100, learning_rate=0.1, num_leaves=31, n_jobs=4,
    deterministic=True, force_row_wise=True, random_state=0, verbose=-1,
)
REVIEW_COST_VND = 8_000  # giả định bài 54: một chuyến gắn cờ nhầm tốn 5 phút người duyệt
HANDLING_COST_VND = 20_000  # giả định bài 54: bỏ sót một chuyến gian lận tốn tiền chuyến + chừng này
CAL_START_DAY = 22  # mô hình gốc: ngày < 22; hiệu chỉnh: ngày 22 đến 30


def make_pipeline(estimator, *, scale: bool = False, features: Sequence[str] = FEATURES) -> Pipeline:
    """ColumnTransformer chọn đúng các đặc trưng THEO TÊN (bỏ cột khác), rồi tới mô hình.

    scale=True thêm log1p + StandardScaler cho mô hình tuyến tính; cây quyết định không cần (log1p đơn điệu).
    """
    numeric = (
        Pipeline([("log1p", FunctionTransformer(np.log1p)), ("scale", StandardScaler())]) if scale else "passthrough"
    )
    select = ColumnTransformer([("features", numeric, list(features))], remainder="drop", verbose_feature_names_out=False)
    return Pipeline([("select", select), ("model", estimator)])


def make_lgbm(**overrides) -> lgb.LGBMClassifier:
    """LightGBM với LGBM_PARAMS (mặc định đúng 100 vòng đã chọn); `overrides` ghi đè từng tham số."""
    return lgb.LGBMClassifier(**{**LGBM_PARAMS, **overrides})


def fit_calibrated(
    x: pl.DataFrame, y: np.ndarray, day: np.ndarray, *, cal_start_day: float = CAL_START_DAY, method: str = "sigmoid", **lgbm_overrides
) -> tuple[Pipeline, CalibratedClassifierCV]:
    """Mô hình gốc huấn luyện trên ngày < cal_start_day, rồi hiệu chỉnh trên các ngày còn lại của `x`.

    `FrozenEstimator` bảo CalibratedClassifierCV không huấn luyện lại mô hình gốc. Trả (mô hình gốc, mô hình đã hiệu chỉnh).
    `x` là DataFrame có đủ cột FEATURES; `y` nhãn 0/1; `day` ngày thanh toán (tính từ đầu dữ liệu) của từng dòng.
    """
    a, b = np.flatnonzero(day < cal_start_day), np.flatnonzero(day >= cal_start_day)
    base = make_pipeline(make_lgbm(**lgbm_overrides)).fit(x[a], y[a])
    cal = CalibratedClassifierCV(FrozenEstimator(base), method=method).fit(x[b], y[b])
    return base, cal


def time_folds(day: np.ndarray, *, n_splits: int = 4, test_days: int = 5, gap_days: int = 2, end_day: int = 30) -> Iterator[tuple[np.ndarray, np.ndarray]]:
    """Cross-validation mở rộng dần theo thời gian: test là một cửa sổ `test_days` ngày, train là mọi ngày trước đó
    trừ `gap_days` ngày sát test. Không bao giờ train trên tương lai."""
    for i in range(n_splits):
        start = end_day - (n_splits - i) * test_days
        yield np.flatnonzero(day < start - gap_days), np.flatnonzero((day >= start) & (day < start + test_days))


def total_cost(flag: np.ndarray, y: np.ndarray, amount_vnd: np.ndarray) -> float:
    """Tổng chi phí giả định của một chính sách gắn cờ.

    chi phí = số chuyến gắn cờ x 8.000 + chuyến gian lận bỏ sót x (tiền chuyến + 20.000).
    Phí duyệt tính cho MỌI chuyến gắn cờ, kể cả gian lận thật. Hai giả định: (1) gắn cờ chặn được toàn bộ thiệt hại,
    tức người duyệt không sai; (2) `amount_vnd` đại diện cho thiệt hại của chuyến (với promo_abuse, thiệt hại thật
    chỉ là phần giảm giá)."""
    flag, y = np.asarray(flag, dtype=bool), np.asarray(y, dtype=bool)
    missed = ~flag & y
    return float(REVIEW_COST_VND * flag.sum() + (np.asarray(amount_vnd, dtype=float)[missed] + HANDLING_COST_VND).sum())


def best_cost_threshold(y: np.ndarray, p: np.ndarray, amount_vnd: np.ndarray) -> tuple[float, float]:
    """Ngưỡng p làm `total_cost` nhỏ nhất, chỉ cắt giữa hai điểm khác nhau. Trả (ngưỡng, chi phí)."""
    y = np.asarray(y, dtype=bool)
    order = np.argsort(-p, kind="stable")
    ps, ys = p[order], y[order]
    miss_cost = (np.asarray(amount_vnd, dtype=float)[order] + HANDLING_COST_VND) * ys
    k = np.arange(len(ps) + 1)
    cost_k = REVIEW_COST_VND * k + np.r_[miss_cost.sum(), miss_cost.sum() - np.cumsum(miss_cost)]
    cut = np.r_[True, ps[:-1] != ps[1:], True]  # chỉ được cắt giữa hai điểm khác nhau (k = 0..n)
    best = int(np.argmin(np.where(cut, cost_k, np.inf)))
    return (float("inf") if best == 0 else float(ps[best - 1])), float(cost_k[best])


def expected_cost_flag(p: np.ndarray, amount_vnd: np.ndarray) -> np.ndarray:
    """Gắn cờ khi p * (tiền chuyến + 20.000) > 8.000: chi phí kỳ vọng của bỏ sót vượt phí duyệt.

    Đúng với mô hình chi phí của `total_cost` (phí duyệt trả cho mọi chuyến gắn cờ). Chỉ có nghĩa khi `p` là xác suất
    đã hiệu chỉnh (calibrated)."""
    return np.asarray(p, dtype=float) * (np.asarray(amount_vnd, dtype=float) + HANDLING_COST_VND) > REVIEW_COST_VND


# ---- Phân phối tham chiếu cho PSI (bài 60 so phân phối đặc trưng lúc phục vụ với lúc huấn luyện) -------------------


def psi_edges(values: np.ndarray, bins: int = 10) -> np.ndarray:
    """Mốc cắt cho PSI: phân vị bên trong (đã bỏ trùng) cộng, với MỖI mốc e, mốc ngay dưới nó (`nextafter(e, -inf)`).

    Khoảng đóng bên phải (x <= mốc) tách «<= 0» khỏi «> 0» ở cờ hiếm. Mốc phụ ngay dưới e tách thêm khoảng chỉ chứa
    giá trị bằng e, nên khối giá trị trùng (cờ gần như toàn số 1, đặc trưng hằng số) không bị gộp với các giá trị thấp hơn:
    PSI thấy lệch về CẢ HAI phía. Với đặc trưng liên tục khoảng phụ rỗng ở cả hai tập và không đổi PSI."""
    q = np.unique(np.quantile(values, np.linspace(0, 1, bins + 1)[1:-1]))
    return np.unique(np.concatenate([q, np.nextafter(q, -np.inf)]))


def reference_distribution(x: pl.DataFrame, features: Sequence[str] = FEATURES, *, bins: int = 10) -> dict[str, dict[str, list[float]]]:
    """Với mỗi đặc trưng: các mốc cắt (xem `psi_edges`) và tỉ lệ dòng mỗi khoảng trên dữ liệu huấn luyện."""
    out = {}
    for f in features:
        v = x[f].to_numpy().astype(float)
        edges = psi_edges(v, bins)
        counts = np.bincount(np.searchsorted(edges, v, side="left"), minlength=len(edges) + 1)
        out[f] = {"edges": edges.tolist(), "fractions": (counts / len(v)).tolist()}
    return out


def psi(reference: dict[str, list[float]], values: np.ndarray, *, eps: float = 1e-4) -> float:
    """Population Stability Index của `values` so với một mục của `reference_distribution` (cùng quy ước khoảng đóng bên phải)."""
    edges = np.asarray(reference["edges"])
    counts = np.bincount(np.searchsorted(edges, np.asarray(values, dtype=float), side="left"), minlength=len(edges) + 1)
    actual = np.clip(counts / max(1, counts.sum()), eps, None)
    expected = np.clip(np.asarray(reference["fractions"]), eps, None)
    return float(np.sum((actual - expected) * np.log(actual / expected)))
