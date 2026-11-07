"""Tiêm lỗi vào store và mô hình để kiểm hành vi fallback mà không phụ thuộc tốc độ máy."""

import asyncio
import time
from typing import Any


class InjectedStore:
    """Bọc một store: trước khi đọc thì chờ `delay_s` giây, kẹt vĩnh viễn nếu `hang`, trả một đặc trưng NaN hoặc làm mất một cột. Đếm số lời gọi bị hủy."""

    def __init__(self, inner: Any, *, delay_s: float = 0.0, hang: bool = False, nan_feature: str | None = None,
                 drop_feature: str | None = None) -> None:
        self.inner, self.delay_s, self.hang, self.nan_feature, self.drop_feature = inner, delay_s, hang, nan_feature, drop_feature
        self.calls = 0
        self.cancelled = 0

    def missing_for(self, feature_list: Any) -> list[str]:
        return self.inner.missing_for(feature_list)

    async def read_features(self, trip: Any):
        self.calls += 1
        try:
            if self.hang:
                await asyncio.Event().wait()  # không bao giờ được set: mô phỏng một kết nối treo
            if self.delay_s:
                await asyncio.sleep(self.delay_s)
        except asyncio.CancelledError:
            self.cancelled += 1
            raise
        got = await self.inner.read_features(trip)
        if self.nan_feature:  # nguồn đặc trưng hỏng: một giá trị thành NaN
            got = got._replace(features={**got.features, self.nan_feature: float("nan")})
        if self.drop_feature:  # nguồn đặc trưng thiếu một cột
            got = got._replace(features={k: v for k, v in got.features.items() if k != self.drop_feature})
        return got


class BlockingModel:
    """Bọc mô hình: `score_one` ngủ `block_s` giây bằng time.sleep (chặn luồng đang chạy nó) rồi mới chấm."""

    def __init__(self, inner: Any, *, block_s: float) -> None:
        self.inner, self.block_s = inner, block_s
        self.meta, self.version, self.feature_list = inner.meta, inner.version, inner.feature_list
        self.finished = 0
        self.started = 0

    def flag(self, p: Any, amount_vnd: Any) -> Any:
        return self.inner.flag(p, amount_vnd)

    def score_one(self, features: Any) -> float:
        self.started += 1
        time.sleep(self.block_s)
        self.finished += 1
        return self.inner.score_one(features)
