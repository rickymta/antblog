"""Bảng huấn luyện: 30 đặc trưng đúng theo thời điểm (bài 55) nối với nhãn, tách train/valid (bài 54).

Dữ liệu sinh bằng `make_events(seed=42, n_trips=200_000)`; đặc trưng tính bằng `features.add_features`. Kết quả
lưu `out/table.parquet` để các lần chạy sau không phải tính lại; xóa thư mục `out/` để tính lại từ đầu.
"""

from pathlib import Path

import numpy as np
import polars as pl

from dataset import CUTOFF_DAY, TRAIN_END_DAY, VALID_START_DAY, at_day
from features import FEATURES, add_features, day_of, trip_table
from synthetic_fraud import make_events

SEED = 42
N_TRIPS = 200_000
OUT = Path(__file__).parent / "out"

# Cột giữ lại ngoài 30 đặc trưng: khóa, thời điểm chấm và nhãn (không bao giờ là đặc trưng). `amount_vnd` đã là một đặc trưng, dùng luôn cho chi phí.
KEYS = ["trip_id", "paid_at"]
LABELS = ["fraud_type", "is_fraud", "label_known_at"]


def build_table() -> pl.DataFrame:
    """Sinh event, dựng bảng chuyến, thêm đặc trưng, nối nhãn. Sắp theo paid_at."""
    data = make_events(seed=SEED, n_trips=N_TRIPS)
    feat = add_features(trip_table(data.events))
    return feat.select([*KEYS, *FEATURES]).join(data.labels, on="trip_id").sort("paid_at")


def load_table(*, cache: bool = True) -> pl.DataFrame:
    path = OUT / "table.parquet"
    if cache and path.exists():
        return pl.read_parquet(path)
    table = build_table()
    if cache:
        OUT.mkdir(exist_ok=True)
        table.write_parquet(path)
    return table


def split(table: pl.DataFrame) -> tuple[pl.DataFrame, pl.DataFrame]:
    """Train: thanh toán trước ngày 30. Valid: từ ngày 40. Ngày 30 đến 40 bỏ trống cho nhãn chín (bài 54)."""
    day = day_of("paid_at")
    train = table.filter(day < TRAIN_END_DAY)
    valid = table.filter(day >= VALID_START_DAY)
    return train, valid


def xy(frame: pl.DataFrame) -> tuple[pl.DataFrame, np.ndarray]:
    """Đặc trưng (DataFrame giữ tên cột) và nhãn nhị phân."""
    return frame.select(FEATURES), frame["is_fraud"].to_numpy().astype(int)


__all__ = ["CUTOFF_DAY", "FEATURES", "at_day", "build_table", "load_table", "split", "xy"]
