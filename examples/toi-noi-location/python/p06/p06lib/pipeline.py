"""LOC-51: pipeline Polars từ Parquet lịch sử GPS (lược đồ LOC-20 v1) tới đặc trưng chặng theo giờ và ô H3.

Ba quy tắc xuyên suốt, mỗi quy tắc có test:
1. **Khử trùng** theo `(driver_ref, event_time, seq)`: giao hàng ít nhất một lần nên có dòng trùng.
2. **Không rò rỉ tương lai**: đặc trưng lịch sử của một chặng chỉ dùng dữ liệu của các ngày *trước* ngày khởi hành (cửa sổ trượt
   `lagged_sums`), không bao giờ dùng cùng ngày hay ngày sau.
3. **Chia theo thời gian**, không chia ngẫu nhiên.

Dữ liệu GPS bị giảm mẫu nên khoảng cách thời gian giữa hai dòng KHÔNG đều (xem `proto/location/v1/gps-history-parquet.md`):
tốc độ theo ô được tính bằng `Σ quãng / Σ thời gian` của các đoạn giữa hai điểm liên tiếp, có trọng số thời gian, không phải trung bình
cột `speed_dmps` (NULL là «không báo», 0 là «đứng yên»: hai thứ khác nhau, không được `fill_null(0)`).
"""

from __future__ import annotations

from pathlib import Path

import h3
import numpy as np
import polars as pl

from .common import CENTRE, FEATURES, TEST_DAYS, TRAIN_DAYS, VAL_DAYS, haversine_m

H3_RES = 7
GPS_KEY = ["driver_ref", "event_time", "seq"]
VN_OFFSET = pl.duration(hours=7)
MAX_SEGMENT_S = 90  # đoạn giữa hai điểm cách nhau quá 90 s (nhịp tim 60 s) không phải di chuyển liên tục
EARTH_R = 6_371_008.8


# ---------------------------------------------------------------------------------------------------------------------
# Đọc, khử trùng
# ---------------------------------------------------------------------------------------------------------------------


def scan_gps(root: Path) -> pl.LazyFrame:
    """Đọc cả cây `date=…/city=…` (cột `date`, `city` chỉ có trong đường dẫn nên phải bật `hive_partitioning`)."""
    return pl.scan_parquet(str(root / "**" / "*.parquet"), hive_partitioning=True)


def dedupe_gps(lf: pl.LazyFrame) -> pl.LazyFrame:
    """Bỏ dòng trùng (giao ít nhất một lần). Giữ dòng đầu tiên; các bản trùng có cùng nội dung."""
    return lf.unique(subset=GPS_KEY, keep="first", maintain_order=False)


def local_parts(col: str) -> list[pl.Expr]:
    """Ngày (kiểu Date), giờ theo giờ Việt Nam của một cột thời gian UTC."""
    lt = pl.col(col).dt.replace_time_zone(None) + VN_OFFSET
    return [lt.dt.date().alias("local_date"), lt.dt.hour().cast(pl.Int32).alias("hour")]


def hav_expr(lat1: pl.Expr, lng1: pl.Expr, lat2: pl.Expr, lng2: pl.Expr) -> pl.Expr:
    p1, p2 = lat1.radians(), lat2.radians()
    a = ((p2 - p1) / 2).sin() ** 2 + p1.cos() * p2.cos() * ((lng2 - lng1).radians() / 2).sin() ** 2
    return 2 * EARTH_R * a.sqrt().arcsin()


def day_expr(date_col: pl.Expr, day0: str) -> pl.Expr:
    from datetime import date

    return (date_col - pl.lit(date.fromisoformat(day0))).dt.total_days().cast(pl.Int32)


def cells_of_buckets(buckets: pl.DataFrame, res: int = H3_RES) -> pl.DataFrame:
    """Ô H3 của tâm mỗi ô lưới 0,001° (≈ 111 m). Tính một lần cho vài chục nghìn ô lưới thay vì cho hàng triệu điểm GPS;
    sai lệch tối đa ≈ 78 m ở biên ô H3 độ phân giải 7 (cạnh ô ≈ 1,4 km)."""
    xs, ys = buckets["bx"].to_list(), buckets["by"].to_list()
    cells = [h3.latlng_to_cell(x / 1000.0, y / 1000.0, res) for x, y in zip(xs, ys)]
    return buckets.with_columns(pl.Series("cell", cells, dtype=pl.Utf8))


# ---------------------------------------------------------------------------------------------------------------------
# Từ GPS tới bảng (ngày, giờ, ô)
# ---------------------------------------------------------------------------------------------------------------------


def gps_segments(gps: pl.LazyFrame) -> pl.LazyFrame:
    """Đoạn giữa hai điểm GPS liên tiếp của cùng tài xế, cùng chuyến: quãng (m), thời gian (s), giờ và ngày địa phương của điểm đầu."""
    g = (
        gps.select(
            "driver_ref",
            "trip_id",
            "event_time",
            (pl.col("lat_e7") / 1e7).alias("lat"),
            (pl.col("lng_e7") / 1e7).alias("lng"),
        )
        .sort(["driver_ref", "event_time"])
        .with_columns(
            pl.col("event_time").shift(1).over(["driver_ref", "trip_id"]).alias("pt"),
            pl.col("lat").shift(1).over(["driver_ref", "trip_id"]).alias("plat"),
            pl.col("lng").shift(1).over(["driver_ref", "trip_id"]).alias("plng"),
        )
        .filter(pl.col("pt").is_not_null())
        .with_columns(
            ((pl.col("event_time") - pl.col("pt")).dt.total_milliseconds() / 1000.0).alias("dt_s"),
            hav_expr(pl.col("plat"), pl.col("plng"), pl.col("lat"), pl.col("lng")).alias("dist_m"),
            *local_parts("pt"),
            (pl.col("plat") * 1000).round().cast(pl.Int32).alias("bx"),
            (pl.col("plng") * 1000).round().cast(pl.Int32).alias("by"),
        )
        .filter((pl.col("dt_s") > 0) & (pl.col("dt_s") <= MAX_SEGMENT_S))
    )
    return g


def cell_hour_table(gps: pl.LazyFrame, day0: str) -> pl.DataFrame:
    """Σ quãng, Σ thời gian, số đoạn theo (ngày, giờ, ô H3 độ phân giải 7)."""
    seg = gps_segments(gps)
    agg = (
        seg.group_by(["local_date", "hour", "bx", "by"])
        .agg(pl.col("dist_m").sum(), pl.col("dt_s").sum(), pl.len().alias("n_seg"))
        .collect()
    )
    buckets = cells_of_buckets(agg.select("bx", "by").unique())
    out = (
        agg.join(buckets, on=["bx", "by"])
        .group_by(["local_date", "hour", "cell"])
        .agg(pl.col("dist_m").sum(), pl.col("dt_s").sum(), pl.col("n_seg").sum())
        .with_columns(day_expr(pl.col("local_date"), day0).alias("day"))
        .drop("local_date")
        .sort(["day", "hour", "cell"])
    )
    return out


# ---------------------------------------------------------------------------------------------------------------------
# Cửa sổ trượt không nhìn thấy tương lai
# ---------------------------------------------------------------------------------------------------------------------


def lagged_sums(
    tbl: pl.DataFrame,
    keys: list[str],
    values: list[str],
    window: int = 7,
    include_current: bool = False,
) -> pl.DataFrame:
    """Với mỗi `day` D và khoá `keys`, tổng các `values` của các ngày D−window … D−1 (CHỈ các ngày trước, không gồm ngày D).

    Cài đặt bằng cách nhân bản bảng với độ trễ k = 1…window rồi cộng dồn: một dòng của ngày S chỉ rơi vào ngày đích S + k > S,
    nên không thể rò rỉ dữ liệu của cùng ngày hay ngày sau. Đây là chỗ duy nhất tạo đặc trưng lịch sử, và có test.

    `include_current=True` (độ trễ bắt đầu từ 0) CỐ Ý tạo lỗi «quên dịch cửa sổ một ngày» để lab đo hậu quả; không dùng để huấn luyện thật.
    """
    start = 0 if include_current else 1
    shifted = [tbl.with_columns((pl.col("day") + k).alias("day")) for k in range(start, window + 1)]
    return (
        pl.concat(shifted)
        .group_by(["day", *keys])
        .agg([pl.col(v).sum() for v in values])
    )


# ---------------------------------------------------------------------------------------------------------------------
# Chặng: nối chuyến, đặc trưng cơ bản
# ---------------------------------------------------------------------------------------------------------------------


def cell_of_lat_lng(lat: np.ndarray, lng: np.ndarray, res: int = H3_RES) -> list[str]:
    return [h3.latlng_to_cell(float(a), float(b), res) for a, b in zip(lat, lng)]


def leg_base_features(legs: pl.DataFrame, trips: pl.DataFrame, day0: str) -> pl.DataFrame:
    """Nối `legs` với `trips` theo `trip_id` và dựng đặc trưng chỉ dựa vào thông tin biết lúc xe KHỞI HÀNH (không dùng
    thời gian thật, GPS của chính chặng, hay mốc đến/đi của chuyến)."""
    t = trips.filter(pl.col("trip_id").is_not_null()).select("trip_id", "request_time")
    assert t["trip_id"].n_unique() == t.height, "trip_id trùng trong bảng chuyến"
    df = (
        legs.join(t, on="trip_id", how="left")
        .with_columns(*local_parts("depart_time"))
        .with_columns(
            day_expr(pl.col("local_date"), day0).alias("day"),
            (pl.col("hour") + (pl.col("depart_time").dt.replace_time_zone(None) + VN_OFFSET).dt.minute() / 60.0).alias("tod"),
            (pl.col("local_date").dt.weekday() - 1).cast(pl.Int32).alias("dow"),  # 0 = thứ Hai
            (pl.col("o_lat_e7") / 1e7).alias("o_lat"),
            (pl.col("o_lng_e7") / 1e7).alias("o_lng"),
            (pl.col("d_lat_e7") / 1e7).alias("d_lat"),
            (pl.col("d_lng_e7") / 1e7).alias("d_lng"),
            ((pl.col("depart_time") - pl.col("request_time")).dt.total_milliseconds() / 1000.0).alias("req_to_depart_s"),
        )
        .with_columns((pl.col("dow") >= 5).cast(pl.Int32).alias("is_weekend"))
    )
    o_lat, o_lng, d_lat, d_lng = (df[c].to_numpy() for c in ("o_lat", "o_lng", "d_lat", "d_lng"))
    crow = haversine_m(o_lat, o_lng, d_lat, d_lng)
    df = df.with_columns(
        pl.Series("crow_m", crow),
        pl.Series("o_dist_centre_m", haversine_m(o_lat, o_lng, CENTRE[0], CENTRE[1])),
        pl.Series("d_dist_centre_m", haversine_m(d_lat, d_lng, CENTRE[0], CENTRE[1])),
        pl.Series("o_cell", cell_of_lat_lng(o_lat, o_lng), dtype=pl.Utf8),
        pl.Series("d_cell", cell_of_lat_lng(d_lat, d_lng), dtype=pl.Utf8),
    ).with_columns((pl.col("route_dist_m") / pl.max_horizontal(pl.col("crow_m"), pl.lit(1.0))).alias("detour"))
    # nhãn (chỉ có ở dữ liệu quá khứ; chặng huỷ giữa đường không có nhãn nên bị loại, xem `labelled`)
    df = df.with_columns(
        (pl.col("actual_s") - pl.col("route_eta_s")).alias("resid_s"),
        (pl.col("actual_s") / pl.col("route_eta_s")).alias("ratio"),
    )
    return df


def labelled(df: pl.DataFrame) -> pl.DataFrame:
    """Chặng có nhãn dùng được: không huỷ giữa đường, có thời gian thật, ETA dương.

    Cảnh báo thiên lệch: chặng đón bị khách huỷ giữa đường bị loại, và chúng thường là chặng dài. Nhãn vì thế thiên về chặng ngắn."""
    return df.filter(
        (~pl.col("cancelled")) & pl.col("actual_s").is_not_null() & (pl.col("route_eta_s") > 0)
    )


# ---------------------------------------------------------------------------------------------------------------------
# Đặc trưng lịch sử theo (giờ, ô) và chia theo thời gian
# ---------------------------------------------------------------------------------------------------------------------

MIN_SEG = 5  # cần ít nhất chừng này đoạn GPS (hoặc chặng) trong cửa sổ mới tin
MIN_LEGS = 3


def speed_table(cell_hour: pl.DataFrame, window: int = 7, include_current: bool = False) -> pl.DataFrame:
    """Tốc độ (m/s) theo (day, hour, cell) từ `window` ngày TRƯỚC `day`: Σ quãng / Σ thời gian."""
    lag = lagged_sums(cell_hour, ["hour", "cell"], ["dist_m", "dt_s", "n_seg"], window, include_current)
    return lag.with_columns(
        pl.when((pl.col("n_seg") >= MIN_SEG) & (pl.col("dt_s") > 0))
        .then(pl.col("dist_m") / pl.col("dt_s"))
        .otherwise(None)
        .alias("speed")
    ).select("day", "hour", "cell", "speed")


def resid_table(lab: pl.DataFrame, window: int = 7, include_current: bool = False) -> pl.DataFrame:
    """Trung bình (thời gian thật / ETA route-engine − 1) theo (day, hour, o_cell) từ `window` ngày TRƯỚC `day`.
    Đây là mã hoá theo nhãn (target encoding); dùng đúng cách nhờ chỉ nhìn các ngày trước."""
    g = (
        lab.group_by(["day", "hour", "o_cell"])
        .agg((pl.col("ratio") - 1.0).sum().alias("s"), pl.len().alias("n"))
        .rename({"o_cell": "cell"})
    )
    lag = lagged_sums(g, ["hour", "cell"], ["s", "n"], window, include_current)
    return lag.with_columns(
        pl.when(pl.col("n") >= MIN_LEGS).then(pl.col("s") / pl.col("n")).otherwise(None).alias("hist_resid_o")
    ).select("day", "hour", "cell", "hist_resid_o")


def add_history(df: pl.DataFrame, cell_hour: pl.DataFrame, window: int = 7, include_current: bool = False) -> pl.DataFrame:
    """Gắn `hist_speed_o`, `hist_speed_d`, `hist_resid_o` (cửa sổ `window` ngày trước, không có ngày hiện tại).
    `include_current=True` là lỗi có chủ đích (xem `lagged_sums`)."""
    sp = speed_table(cell_hour, window, include_current)
    rs = resid_table(labelled(df), window, include_current)
    out = (
        df.join(sp.rename({"cell": "o_cell", "speed": "hist_speed_o"}), on=["day", "hour", "o_cell"], how="left")
        .join(sp.rename({"cell": "d_cell", "speed": "hist_speed_d"}), on=["day", "hour", "d_cell"], how="left")
        .join(rs.rename({"cell": "o_cell"}), on=["day", "hour", "o_cell"], how="left")
    )
    return out


def frozen_history(cell_hour: pl.DataFrame, lab: pl.DataFrame, day_lo: int, day_hi: int) -> tuple[pl.DataFrame, pl.DataFrame]:
    """Bảng tra «đóng băng» theo (hour, cell) gộp các ngày `day_lo…day_hi`: thứ mà service sẽ nạp (có phiên bản)
    để tính đặc trưng lịch sử cho chặng của những ngày mà cửa sổ trượt chưa có dữ liệu."""
    ch = cell_hour.filter(pl.col("day").is_between(day_lo, day_hi)).group_by(["hour", "cell"]).agg(
        pl.col("dist_m").sum(), pl.col("dt_s").sum(), pl.col("n_seg").sum()
    )
    sp = ch.with_columns(
        pl.when((pl.col("n_seg") >= MIN_SEG) & (pl.col("dt_s") > 0)).then(pl.col("dist_m") / pl.col("dt_s")).otherwise(None).alias("speed")
    ).select("hour", "cell", "speed")
    g = (
        lab.filter(pl.col("day").is_between(day_lo, day_hi))
        .group_by(["hour", "o_cell"])
        .agg((pl.col("ratio") - 1.0).sum().alias("s"), pl.len().alias("n"))
        .with_columns(pl.when(pl.col("n") >= MIN_LEGS).then(pl.col("s") / pl.col("n")).otherwise(None).alias("hist_resid_o"))
        .select("hour", pl.col("o_cell").alias("cell"), "hist_resid_o")
    )
    return sp, g


def apply_frozen(df: pl.DataFrame, sp: pl.DataFrame, rs: pl.DataFrame) -> pl.DataFrame:
    return (
        df.join(sp.rename({"cell": "o_cell", "speed": "hist_speed_o"}), on=["hour", "o_cell"], how="left")
        .join(sp.rename({"cell": "d_cell", "speed": "hist_speed_d"}), on=["hour", "d_cell"], how="left")
        .join(rs.rename({"cell": "o_cell"}), on=["hour", "o_cell"], how="left")
    )


def time_split(df: pl.DataFrame) -> pl.DataFrame:
    """Cột `split` theo ngày: huấn luyện / kiểm định / kiểm tra, không giao nhau và theo đúng thứ tự thời gian."""
    d = pl.col("day")
    return df.with_columns(
        pl.when(d.is_between(*TRAIN_DAYS))
        .then(pl.lit("train"))
        .when(d.is_between(*VAL_DAYS))
        .then(pl.lit("val"))
        .when(d.is_between(*TEST_DAYS))
        .then(pl.lit("test"))
        .otherwise(pl.lit("later"))
        .alias("split")
    )


# ---------------------------------------------------------------------------------------------------------------------
# Rò rỉ có chủ đích (để đo hậu quả, KHÔNG dùng để huấn luyện thật)
# ---------------------------------------------------------------------------------------------------------------------


def add_leaky_target_encoding(df: pl.DataFrame) -> pl.DataFrame:
    """Rò rỉ: trung bình (ratio − 1) theo (giờ, ô xuất phát) tính trên TOÀN BỘ dữ liệu, gồm cả chính dòng đó, ngày hiện tại và tương lai."""
    lab = labelled(df)
    g = (
        lab.group_by(["hour", "o_cell"])
        .agg((pl.col("ratio") - 1.0).mean().alias("leak_resid_o"))
    )
    return df.join(g, on=["hour", "o_cell"], how="left")


def feature_matrix(df: pl.DataFrame, features: list[str] | None = None) -> np.ndarray:
    cols = features or FEATURES
    return df.select([pl.col(c).cast(pl.Float64) for c in cols]).to_numpy().astype(np.float32)


def trip_own_gps_speed(gps: pl.LazyFrame) -> pl.DataFrame:
    """Σ quãng / Σ thời gian GPS của CHÍNH chuyến (cả hai chặng). Thông tin này chỉ có SAU khi chuyến xong:
    dùng nó làm đặc trưng ETA là rò rỉ nhãn trực tiếp (xem LOC-51 mục 4, rò rỉ D)."""
    seg = gps_segments(gps).group_by("trip_id").agg(pl.col("dist_m").sum().alias("d"), pl.col("dt_s").sum().alias("t"))
    return seg.with_columns((pl.col("d") / pl.col("t")).alias("leak_trip_speed")).select("trip_id", "leak_trip_speed").collect()
