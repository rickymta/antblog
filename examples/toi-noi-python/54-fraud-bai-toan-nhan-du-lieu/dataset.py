"""Từ log event đến bảng chuyến, nhãn theo thời điểm biết, chia theo thời gian và bộ chấm điểm nền.

Bảng chuyến ở đây chỉ có đặc trưng THEO TỪNG CHUYẾN (không cửa sổ thời gian, không đồ thị);
đặc trưng nhiều chuyến là việc của bài 55.
"""

from datetime import timedelta

import numpy as np
import polars as pl
from sklearn.linear_model import LogisticRegression
from sklearn.pipeline import Pipeline
from sklearn.preprocessing import StandardScaler

from synthetic_fraud import START, haversine_km

TZ = "Asia/Ho_Chi_Minh"
RAW_FEATURES = [
    "log_speed_kmh",  # đường chim bay start -> end chia thời lượng chuyến
    "log_start_pickup_gap_km",  # GPS tài xế lúc bắt đầu cách điểm đón khách khai báo
    "route_ratio",  # đường chim bay start -> end chia quãng đường khai báo
    "discount_ratio",  # phần giảm giá / cước gốc
    "log_distance_km",
    "log_duration_s",
    "is_night",  # 0 đến 5 giờ sáng giờ địa phương
    "same_device",  # khách đặt bằng chính điện thoại của tài xế
    "has_newuser_promo",
]
TRAIN_END_DAY = 30  # train: chuyến thanh toán trước ngày 30 (tính từ START)
VALID_START_DAY = 40  # valid: từ ngày 40; ngày 30 đến 40 là khoảng chờ cho nhãn chín
CUTOFF_DAY = 40  # ngày ta "huấn luyện": chỉ nhãn có label_known_at <= ngày này mới biết


def at_day(day: float):
    """Thời điểm UTC cách START `day` ngày."""
    return START + timedelta(days=day)


def trip_table(events: pl.DataFrame) -> pl.DataFrame:
    """Nối ba loại sự kiện theo trip_id thành một dòng mỗi chuyến, kèm đặc trưng theo từng chuyến."""
    def only(kind: str, cols: list[str], renames: dict[str, str]) -> pl.DataFrame:
        return events.filter(pl.col("event_type") == kind).select(["trip_id", "occurred_at", *cols]).rename(renames)

    booking = only("BookingCreated", ["customer_id", "device_id", "service", "city", "pickup_lat", "pickup_lng", "promo_code"],
                   {"occurred_at": "booked_at"})
    done = only("TripCompleted", ["driver_id", "driver_device_id", "start_lat", "start_lng", "end_lat", "end_lng",
                                  "distance_km", "duration_s"], {"occurred_at": "completed_at"})
    pay = only("PaymentCaptured", ["amount_vnd", "discount_vnd"], {"occurred_at": "paid_at"})
    t = booking.join(done, on="trip_id", how="inner").join(pay, on="trip_id", how="inner").sort("paid_at")
    hav_se = haversine_km(t["start_lat"].to_numpy(), t["start_lng"].to_numpy(), t["end_lat"].to_numpy(), t["end_lng"].to_numpy())
    gap = haversine_km(t["pickup_lat"].to_numpy(), t["pickup_lng"].to_numpy(), t["start_lat"].to_numpy(), t["start_lng"].to_numpy())
    return t.with_columns(
        pl.Series("hav_start_end_km", hav_se),
        pl.Series("start_pickup_gap_km", gap),
    ).with_columns(
        speed_kmh=pl.col("hav_start_end_km") / (pl.col("duration_s") / 3600),
        route_ratio=pl.col("hav_start_end_km") / pl.col("distance_km"),
        discount_ratio=pl.col("discount_vnd") / (pl.col("amount_vnd") + pl.col("discount_vnd")),
        same_device=(pl.col("device_id") == pl.col("driver_device_id")).cast(pl.Float64),
        is_night=(pl.col("booked_at").dt.convert_time_zone(TZ).dt.hour() < 6).cast(pl.Float64),
        has_newuser_promo=(pl.col("promo_code") == "NEWUSER50").fill_null(False).cast(pl.Float64),
        paid_day=(pl.col("paid_at") - pl.lit(START)).dt.total_seconds() / 86400,
    ).with_columns(
        log_speed_kmh=pl.col("speed_kmh").clip(0, 2000).log1p(),
        log_start_pickup_gap_km=pl.col("start_pickup_gap_km").log1p(),
        log_distance_km=pl.col("distance_km").log(),
        log_duration_s=pl.col("duration_s").cast(pl.Float64).log(),
    )


def known_label(labels: pl.DataFrame, cutoff_day: float) -> pl.DataFrame:
    """Nhãn nhìn thấy được vào ngày cutoff: chuyến gian lận mà label_known_at chưa tới thì vẫn là 0."""
    return labels.with_columns(
        known_fraud=pl.col("is_fraud") & (pl.col("label_known_at") <= pl.lit(at_day(cutoff_day)))
    )


def fit_scorer(x: np.ndarray, y: np.ndarray) -> Pipeline:
    """Bộ chấm điểm nền: hồi quy logistic trên đặc trưng theo từng chuyến, đủ đơn giản để không che lỗi dữ liệu."""
    pipe = Pipeline([("scale", StandardScaler()), ("lr", LogisticRegression(max_iter=1000))])
    return pipe.fit(x, y)
