"""Giải thích từng điểm gian lận bằng SHAP: LightGBM gốc -> đóng góp của từng đặc trưng -> `reasons[]` cho người duyệt.

Điều cần nhớ khi đọc kết quả:
  - SHAP giải thích điểm THÔ (margin, log-odds) CHƯA HIỆU CHỈNH của LightGBM trong Pipeline: tổng các đóng góp cộng
    `base_value` bằng điểm thô. Điểm gửi cho người duyệt là xác suất p ĐÃ HIỆU CHỈNH (bài 57), một hàm đơn điệu của điểm
    thô; SHAP không giải thích p trực tiếp, và tổng đóng góp không cộng thẳng ra log-odds của p (xem `log_odds_map`).
  - Đóng góp lớn nghĩa là «mô hình dựa vào đặc trưng này để chấm cao», không phải «đặc trưng này gây ra gian lận».
  - Đặc trưng tương quan (dev_trips_1h và dev_trips_24h) chia nhau công; xem `GROUPS` để gộp theo họ tín hiệu.
"""

import warnings
from dataclasses import dataclass

import numpy as np
import polars as pl
import shap

from features import FEATURES
from model_io import ScoringModel

# Họ tín hiệu: gộp các đặc trưng cùng nhìn một dấu vết, để lý do ổn định hơn khi hai đặc trưng chia nhau công.
GROUPS: dict[str, tuple[str, ...]] = {
    "hành trình": ("distance_km", "duration_s", "hav_start_end_km", "route_ratio", "speed_kmh", "impossible_speed"),
    "GPS tài xế": ("start_pickup_gap_km", "drv_jump_km", "drv_gap_s", "drv_jump_speed_kmh", "impossible_jump"),
    "khuyến mãi": ("amount_vnd", "discount_ratio", "has_promo", "has_newuser_promo", "cust_promo_share_30d", "dev_newuser_7d"),
    "thiết bị": ("dev_trips_1h", "dev_trips_24h", "dev_customers_24h", "dev_customers_7d", "same_device"),
    "nhịp đặt": ("hour_local", "is_night", "cust_trips_1h", "cust_trips_24h", "drv_trips_1h", "drv_trips_24h",
                        "pair_trips_7d", "cust_prior_trips"),
}
FEATURE_GROUP = {f: g for g, fs in GROUPS.items() for f in fs}

# Câu lý do, điền giá trị THẬT của chuyến. Chỉ dùng số từ đặc trưng, không có số giấy tờ hay thông tin cá nhân.
TEMPLATES: dict[str, str] = {
    "distance_km": "quãng đường khai báo {v:.1f} km",
    "duration_s": "thời lượng chuyến {m:.0f} phút",
    "amount_vnd": "cước đã thu {vnd} VND",
    "discount_ratio": "khuyến mãi chiếm {pct:.0f} % cước gốc",
    "has_promo": "chuyến dùng mã khuyến mãi",
    "has_newuser_promo": "chuyến dùng mã NEWUSER50",
    "hour_local": "đặt lúc {v:.0f} giờ địa phương",
    "is_night": "đặt ban đêm (0 đến 6 giờ)",
    "same_device": "khách đặt bằng chính điện thoại của tài xế",
    "hav_start_end_km": "điểm cuối chỉ cách điểm đầu {v:.2f} km theo đường chim bay",
    "start_pickup_gap_km": "GPS tài xế lúc bắt đầu cách điểm đón khai báo {v:.1f} km",
    "route_ratio": "đường chim bay chỉ bằng {v:.2f} lần quãng đường khai báo",
    "speed_kmh": "tốc độ trung bình trong chuyến {v:.0f} km/h",
    "dev_trips_1h": "thiết bị đã có {v:.0f} chuyến trong 1 giờ trước",
    "dev_trips_24h": "thiết bị đã có {v:.0f} chuyến trong 24 giờ trước",
    "cust_trips_1h": "khách đã có {v:.0f} chuyến trong 1 giờ trước",
    "cust_trips_24h": "khách đã có {v:.0f} chuyến trong 24 giờ trước",
    "drv_trips_1h": "tài xế đã có {v:.0f} chuyến trong 1 giờ trước",
    "drv_trips_24h": "tài xế đã có {v:.0f} chuyến trong 24 giờ trước",
    "dev_customers_24h": "{v:.0f} tài khoản khách khác nhau trên thiết bị trong 24 giờ (kể cả chuyến này)",
    "dev_customers_7d": "{v:.0f} tài khoản khách khác nhau trên thiết bị trong 7 ngày (kể cả chuyến này)",
    "pair_trips_7d": "cặp khách-tài xế đã đi {v:.0f} chuyến trong 7 ngày trước",
    "cust_prior_trips": "khách đã có {v:.0f} chuyến trước đó",
    "cust_promo_share_30d": "{pct:.0f} % chuyến 30 ngày trước của khách dùng khuyến mãi",
    "dev_newuser_7d": "mã NEWUSER50 đã dùng {v:.0f} lần trên thiết bị trong 7 ngày trước",
    "drv_jump_km": "tài xế nhảy {v:.1f} km so với điểm kết thúc chuyến trước",
    "drv_gap_s": "cách chuyến trước của tài xế {m:.0f} phút",
    "drv_jump_speed_kmh": "nhảy vị trí tương đương {v:.0f} km/h giữa hai chuyến của tài xế",
    "impossible_speed": "tốc độ trong chuyến vượt 120 km/h",
    "impossible_jump": "nhảy vị trí giữa hai chuyến vượt 120 km/h",
}
assert set(TEMPLATES) == set(FEATURES) == set(FEATURE_GROUP), "mỗi đặc trưng cần đúng một câu lý do và một họ"


def describe(feature: str, value: float) -> str:
    vnd = f"{value:,.0f}".replace(",", ".")
    return TEMPLATES[feature].format(v=value, m=value / 60.0, pct=100.0 * value, vnd=vnd)


@dataclass(frozen=True, slots=True)
class Reason:
    feature: str
    value: float
    shap: float  # đóng góp vào điểm thô của mô hình gốc, thang log-odds
    text: str

    def as_dict(self) -> dict:
        return {"feature": self.feature, "value": round(self.value, 4), "shap": round(self.shap, 4), "text": self.text}


class Explainer:
    """Bọc `shap.TreeExplainer` quanh LightGBM nằm trong `CalibratedClassifierCV(FrozenEstimator(Pipeline))` của bài 57.

    Dùng `ScoringModel.booster`, `.preprocess`, `.raw_margin` và `.contributions` của model_io (bài 57); `contributions`
    của model_io là TreeSHAP gốc của LightGBM (`pred_contrib=True`), được dùng để đối chiếu với gói `shap`.
    """

    def __init__(self, scoring: ScoringModel) -> None:
        self.scoring = scoring
        self.calibrator = scoring.model.calibrated_classifiers_[0].calibrators[0]
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", UserWarning)  # shap 0.54 cảnh báo về dạng đầu ra của LightGBM nhị phân
            self.tree = shap.TreeExplainer(scoring.booster)
        self.base_value = float(np.ravel(self.tree.expected_value)[-1])

    def _matrix(self, features: pl.DataFrame) -> np.ndarray:
        return self.scoring.preprocess.transform(features.select(self.scoring.feature_list))

    def contributions(self, features: pl.DataFrame) -> np.ndarray:
        """Ma trận (n, 30) đóng góp vào điểm THÔ (log-odds trước hiệu chỉnh), đúng thứ tự `feature_list`."""
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", UserWarning)
            values = self.tree.shap_values(self._matrix(features))
        if isinstance(values, list):  # phiên bản shap cũ trả [lớp 0, lớp 1]
            values = values[-1]
        values = np.asarray(values)
        if values.shape != (features.height, len(self.scoring.feature_list)):
            raise ValueError(f"dạng đầu ra SHAP bất ngờ: {values.shape}")
        return values

    def native_contributions(self, features: pl.DataFrame) -> tuple[np.ndarray, np.ndarray]:
        """TreeSHAP của chính LightGBM qua model_io: (đóng góp (n, 30), hằng số nền (n,))."""
        full = self.scoring.contributions(features)
        return full[:, :-1], full[:, -1]

    def raw_score(self, features: pl.DataFrame) -> np.ndarray:
        return self.scoring.raw_margin(features)

    def calibrated(self, features: pl.DataFrame) -> np.ndarray:
        """Điểm thô -> bộ hiệu chỉnh sigmoid của scikit-learn; phải bằng `scoring.predict_proba`."""
        return self.calibrator.predict(self.raw_score(features))

    def log_odds_map(self) -> tuple[float, float]:
        """(hệ số, hằng số) của log-odds(p) = hằng số + hệ số x điểm thô. Đúng cho bộ hiệu chỉnh SIGMOID chạy trên điểm thô
        (bản này); isotonic hay sigmoid chạy trên xác suất sẽ không có dạng này. Lấy từ thuộc tính `a_`, `b_` của scikit-learn."""
        return -float(self.calibrator.a_), -float(self.calibrator.b_)

    def reasons(self, features: pl.DataFrame, *, k: int = 3, min_shap: float = 0.05) -> list[list[Reason]]:
        """Với mỗi chuyến: tối đa k đặc trưng đẩy điểm LÊN nhiều nhất (đóng góp > min_shap), kèm câu giải thích."""
        names = self.scoring.feature_list
        contrib = self.contributions(features)
        x = features.select(names).to_numpy()
        out = []
        for i in range(features.height):
            order = np.argsort(-contrib[i])[:k]
            out.append([Reason(names[j], float(x[i, j]), float(contrib[i, j]), describe(names[j], float(x[i, j])))
                        for j in order if contrib[i, j] > min_shap])
        return out

    def group_contributions(self, contrib: np.ndarray) -> dict[str, np.ndarray]:
        idx = {f: i for i, f in enumerate(self.scoring.feature_list)}
        return {g: contrib[:, [idx[f] for f in fs]].sum(axis=1) for g, fs in GROUPS.items()}
