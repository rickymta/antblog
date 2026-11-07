"""Đặc trưng gian lận từ log event, tính theo lô bằng polars, ĐÚNG THEO THỜI ĐIỂM.

Quy tắc điểm thời gian (as-of): mỗi chuyến được chấm điểm ở thời điểm `PaymentCaptured` (paid_at).
Đặc trưng của chuyến chỉ được dùng thông tin có trước thời điểm đó: các chuyến TRƯỚC đã thu tiền
(cửa sổ [t - W, t), không gồm chính nó) cộng với ba event của chính chuyến. Chuyến đang chạy mà chưa thu
tiền chưa được tính; đó là đơn giản hóa của lab.

Phần cuối file là các đặc trưng RÒ RỈ (dùng thông tin tương lai hoặc nhãn biết trễ), chỉ để tái hiện lỗi.
"""

import warnings

import polars as pl

from synthetic_fraud import START

TZ = "Asia/Ho_Chi_Minh"
IMPOSSIBLE_KMH = 120.0

BASE_FEATURES = [
    "distance_km", "duration_s", "amount_vnd", "discount_ratio", "has_promo", "has_newuser_promo",
    "hour_local", "is_night", "same_device",
    "hav_start_end_km", "start_pickup_gap_km", "route_ratio", "speed_kmh",
]
WINDOW_FEATURES = [
    "dev_trips_1h", "dev_trips_24h", "cust_trips_1h", "cust_trips_24h", "drv_trips_1h", "drv_trips_24h",
    "dev_customers_24h", "dev_customers_7d", "pair_trips_7d",
    "cust_prior_trips", "cust_promo_share_30d", "dev_newuser_7d",
]
GPS_FEATURES = ["drv_jump_km", "drv_gap_s", "drv_jump_speed_kmh", "impossible_speed", "impossible_jump"]
FEATURES = BASE_FEATURES + WINDOW_FEATURES + GPS_FEATURES


def haversine_expr(lat1: str, lng1: str, lat2: str, lng2: str) -> pl.Expr:
    """Khoảng cách đường chim bay (km) giữa hai cặp cột (độ), bán kính Trái Đất 6371 km."""
    p1, p2 = pl.col(lat1).radians(), pl.col(lat2).radians()
    dphi = p2 - p1
    dlmb = (pl.col(lng2) - pl.col(lng1)).radians()
    a = (dphi / 2).sin() ** 2 + p1.cos() * p2.cos() * (dlmb / 2).sin() ** 2
    return 2 * 6371.0 * a.sqrt().arcsin()


def trip_table(events: pl.DataFrame) -> pl.DataFrame:
    """Nối ba loại event theo trip_id thành một dòng mỗi chuyến, sắp theo thời điểm chấm điểm paid_at."""
    def only(kind: str, cols: list[str], renames: dict[str, str]) -> pl.DataFrame:
        return events.filter(pl.col("event_type") == kind).select(["trip_id", "occurred_at", *cols]).rename(renames)

    booking = only("BookingCreated", ["customer_id", "device_id", "service", "city", "pickup_lat", "pickup_lng", "promo_code"],
                   {"occurred_at": "booked_at"})
    done = only("TripCompleted", ["driver_id", "driver_device_id", "start_lat", "start_lng", "end_lat", "end_lng",
                                  "distance_km", "duration_s"], {"occurred_at": "completed_at"})
    pay = only("PaymentCaptured", ["amount_vnd", "discount_vnd"], {"occurred_at": "paid_at"})
    return (
        booking.join(done, on="trip_id", how="inner").join(pay, on="trip_id", how="inner")
        .with_columns(started_at=pl.col("completed_at") - pl.duration(seconds=pl.col("duration_s")))
        .sort("paid_at")
    )


def add_features(trips: pl.DataFrame) -> pl.DataFrame:
    """Thêm toàn bộ FEATURES. `trips` phải sắp theo paid_at (trip_table đã làm).

    polars 2.1.0 không báo lỗi khi `rolling(...).over(khóa)` chạy trên bảng chưa sắp theo cột chỉ mục: nó trả
    giá trị sai âm thầm. Vì vậy hàm tự kiểm và từ chối thay vì đoán.
    """
    if not trips["paid_at"].is_sorted():
        raise ValueError("add_features cần bảng sắp theo paid_at (trips.sort('paid_at')); rolling().over() không tự báo lỗi")
    def count(key: str | list[str], period: str) -> pl.Expr:
        # cửa sổ [t - period, t): closed="left" loại chính dòng hiện tại
        return pl.len().rolling("paid_at", period=period, closed="left").over(key)

    def distinct(col: str, key: str, period: str) -> pl.Expr:
        # closed="both": gồm cả chuyến hiện tại, để "khách mới trên thiết bị" tự được tính
        return pl.col(col).n_unique().rolling("paid_at", period=period, closed="both").over(key)

    t = trips.with_columns(
        hav_start_end_km=haversine_expr("start_lat", "start_lng", "end_lat", "end_lng"),
        start_pickup_gap_km=haversine_expr("pickup_lat", "pickup_lng", "start_lat", "start_lng"),
        hour_local=pl.col("booked_at").dt.convert_time_zone(TZ).dt.hour().cast(pl.Float64),
        has_promo=pl.col("promo_code").is_not_null().cast(pl.Float64),
        has_newuser_promo=(pl.col("promo_code") == "NEWUSER50").fill_null(False).cast(pl.Float64),
        same_device=(pl.col("device_id") == pl.col("driver_device_id")).cast(pl.Float64),
    ).with_columns(
        speed_kmh=pl.col("hav_start_end_km") / (pl.col("duration_s") / 3600),
        route_ratio=pl.col("hav_start_end_km") / pl.col("distance_km"),
        discount_ratio=pl.col("discount_vnd") / (pl.col("amount_vnd") + pl.col("discount_vnd")),
        is_night=(pl.col("hour_local") < 6).cast(pl.Float64),
    )
    # ---- cửa sổ thời gian ----
    t = t.with_columns(
        dev_trips_1h=count("device_id", "1h"), dev_trips_24h=count("device_id", "24h"),
        cust_trips_1h=count("customer_id", "1h"), cust_trips_24h=count("customer_id", "24h"),
        drv_trips_1h=count("driver_id", "1h"), drv_trips_24h=count("driver_id", "24h"),
        dev_customers_24h=distinct("customer_id", "device_id", "24h"),
        dev_customers_7d=distinct("customer_id", "device_id", "7d"),
        pair_trips_7d=count(["customer_id", "driver_id"], "7d"),
        cust_prior_trips=pl.int_range(pl.len()).over("customer_id"),
        dev_newuser_7d=pl.col("has_newuser_promo").sum().rolling("paid_at", period="7d", closed="left").over("device_id"),
        _promo_30d=pl.col("has_promo").sum().rolling("paid_at", period="30d", closed="left").over("customer_id"),
        _trips_30d=count("customer_id", "30d"),
    ).with_columns(
        cust_promo_share_30d=pl.when(pl.col("_trips_30d") > 0).then(pl.col("_promo_30d") / pl.col("_trips_30d")).otherwise(0.0)
    )
    # ---- GPS: nhảy vị trí so với chuyến trước của cùng tài xế ----
    prev = pl.col("driver_id")
    t = t.with_columns(
        _prev_lat=pl.col("end_lat").shift(1).over(prev), _prev_lng=pl.col("end_lng").shift(1).over(prev),
        _prev_done=pl.col("completed_at").shift(1).over(prev),
    ).with_columns(
        drv_jump_km=haversine_expr("_prev_lat", "_prev_lng", "start_lat", "start_lng").fill_null(0.0),
        # chờ tối thiểu 60 giây để nhiễu GPS vài chục mét không thành "hàng nghìn km/h"
        drv_gap_s=(pl.col("started_at") - pl.col("_prev_done")).dt.total_seconds().clip(lower_bound=60).fill_null(0).cast(pl.Float64),
    ).with_columns(
        drv_jump_speed_kmh=pl.when(pl.col("drv_gap_s") > 0).then(pl.col("drv_jump_km") / (pl.col("drv_gap_s") / 3600)).otherwise(0.0),
    ).with_columns(
        impossible_speed=(pl.col("speed_kmh") > IMPOSSIBLE_KMH).cast(pl.Float64),
        impossible_jump=(pl.col("drv_jump_speed_kmh") > IMPOSSIBLE_KMH).cast(pl.Float64),
    )
    return t.drop([c for c in t.columns if c.startswith("_")])


# ---- Đặc trưng RÒ RỈ: chỉ để tái hiện lỗi, không dùng trong mô hình sạch --------------------


def add_future_window(t: pl.DataFrame) -> pl.DataFrame:
    """Rò rỉ 1: đếm chuyến cùng thiết bị trong (t, t + 1h], tức là NHÌN VỀ TƯƠNG LAI (offset 0, period 1 giờ).

    Cửa sổ bắt đầu sau t nên không gồm chính dòng hiện tại.
    """
    t = t.sort("paid_at")
    return t.with_columns(
        dev_trips_next_1h=pl.len().rolling("paid_at", period="1h", offset="0s", closed="right").over("device_id")
    )


def add_hour_bucket(t: pl.DataFrame) -> pl.DataFrame:
    """Rò rỉ 1b: lỗi hay gặp, gom theo giờ lịch bằng group_by_dynamic rồi join ngược. Mỗi chuyến nhận tổng của CẢ giờ."""
    t = t.sort("paid_at")  # group_by_dynamic đòi dữ liệu đã sắp theo cột chỉ mục
    per_hour = t.group_by_dynamic("paid_at", every="1h", group_by="device_id").agg(dev_trips_this_hour=pl.len())
    return t.with_columns(_hour=pl.col("paid_at").dt.truncate("1h")).join(
        per_hour.rename({"paid_at": "_hour"}), on=["device_id", "_hour"], how="left"
    ).drop("_hour")


def add_global_aggregates(t: pl.DataFrame) -> pl.DataFrame:
    """Rò rỉ 2: gom TOÀN BỘ bảng theo khóa rồi gắn lại, không có cửa sổ thời gian.

    Mỗi chuyến nhận số khách khác nhau của thiết bị và số chuyến của cặp khách-tài xế tính trên cả 60 ngày,
    gồm cả những chuyến chưa xảy ra lúc chấm điểm.
    """
    return t.with_columns(
        dev_customers_total=pl.col("customer_id").n_unique().over("device_id"),
        pair_trips_total=pl.len().over(["customer_id", "driver_id"]),
    )


def add_global_aggregates_serving(t: pl.DataFrame) -> pl.DataFrame:
    """Phiên bản phục vụ của add_global_aggregates: đếm dồn từ đầu tới chuyến hiện tại, không thấy tương lai."""
    t = t.sort("paid_at")
    return t.with_columns(
        dev_customers_total_serving=pl.col("customer_id").n_unique().rolling("paid_at", period="61d", closed="both").over("device_id"),
        pair_trips_total_serving=pl.int_range(1, pl.len() + 1).over(["customer_id", "driver_id"]),
    )


def add_device_fraud_history(t: pl.DataFrame, labels: pl.DataFrame, *, honest: bool) -> pl.DataFrame:
    """Số chuyến gian lận TRƯỚC ĐÓ trên cùng thiết bị.

    honest=False: nhãn được tính ngay từ lúc thanh toán (nhãn cuối cùng, như nhìn lại sau khi đã điều tra xong).
    honest=True:  nhãn chỉ được tính từ lúc `label_known_at`, đúng với thứ ta biết tại thời điểm chấm.
    Dòng hiện tại không bao giờ tự đếm mình (allow_exact_matches=False).
    """
    arrival = pl.col("label_known_at") if honest else pl.col("paid_at")
    arrivals = (
        t.select("trip_id", "device_id", "paid_at").join(labels.filter(pl.col("is_fraud")), on="trip_id")
        .select("device_id", ts=arrival).sort("ts")
        .with_columns(dev_fraud_before=pl.int_range(1, pl.len() + 1).over("device_id"))
    )
    name = "dev_fraud_known" if honest else "dev_fraud_naive"
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")  # polars không kiểm được độ sắp xếp khi có `by`; hai bảng đã sắp theo thời gian
        joined = t.sort("paid_at").join_asof(
            arrivals.rename({"dev_fraud_before": name}), left_on="paid_at", right_on="ts", by="device_id",
            strategy="backward", allow_exact_matches=False,
        )
    return joined.with_columns(pl.col(name).fill_null(0)).drop("ts")


def day_of(col: str = "paid_at") -> pl.Expr:
    return (pl.col(col) - pl.lit(START)).dt.total_seconds() / 86400

