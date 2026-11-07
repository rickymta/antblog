"""Bộ chấm điểm có ngân sách thời gian: đọc đặc trưng từ Redis, chạy mô hình, và LUÔN trả lời đúng hạn.

Hợp đồng với người gọi (Dispatch hay luồng thu tiền): `score()` không ném lỗi và không chờ quá `budget_s`
(mặc định 50 ms). Khi không chấm được thì trả điểm mặc định kèm `degraded=True` và `reason`; điểm đó nghĩa là
«chưa có điểm», không phải «chuyến này sạch». Điểm rủi ro là đầu vào cho người duyệt, không tự khóa tài khoản.
"""

import asyncio
import threading
import time
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from collections.abc import Callable
from dataclasses import dataclass
from typing import Any

from redis.exceptions import RedisError

from store import MIN_HISTORY_US, ClosedTrip

BUDGET_S = 0.050


@dataclass(frozen=True)
class ScoreResult:
    trip_id: str
    score: float  # xác suất gian lận đã hiệu chỉnh, hoặc điểm mặc định khi degraded
    flagged: bool  # theo chi phí kỳ vọng (`model.flag`, bài 57); degraded thì luôn False
    degraded: bool
    reason: str | None  # None, "timeout", "store_error", "store_cold", "store_building", "state_ahead", "breaker_open", "overloaded", "model_error"
    model_version: str
    elapsed_ms: float
    read_ms: float = 0.0  # thời gian đọc Redis và dựng đặc trưng
    model_ms: float = 0.0  # thời gian chạy mô hình
    detail: str | None = None  # tên lớp lỗi khi reason == "store_error"; thông điệp khi "model_error"


class Breaker:
    """Cầu dao cho store: sau `failures` lần hỏng liên tiếp thì mở `cooldown_s` giây, trong lúc đó không gọi store nữa.

    Hết thời gian chờ thì cho ĐÚNG MỘT lời gọi thăm dò đi qua; thành công thì đóng lại. `clock` tiêm vào để test không cần ngủ."""

    def __init__(self, *, failures: int = 5, cooldown_s: float = 2.0, clock: Callable[[], float] = time.monotonic) -> None:
        self.failures, self.cooldown_s, self.clock = failures, cooldown_s, clock
        self._count = 0
        self._open_until = 0.0

    def allow(self) -> bool:
        if self._count < self.failures:
            return True
        if self.clock() >= self._open_until:
            self._open_until = self.clock() + self.cooldown_s  # lời gọi thăm dò; các lời gọi sau phải chờ tiếp
            return True
        return False

    def success(self) -> None:
        self._count = 0

    def failure(self) -> None:
        self._count += 1
        if self._count >= self.failures:
            self._open_until = self.clock() + self.cooldown_s


class Scorer:
    def __init__(self, store: Any, model: Any, *, budget_s: float = BUDGET_S, default_score: float | None = None,
                 breaker: Breaker | None = None, offload_model: bool = True, max_in_flight: int | None = None, workers: int = 8) -> None:
        self.store, self.model, self.budget_s, self.breaker, self.offload_model = store, model, budget_s, breaker, offload_model
        self.max_in_flight, self._in_flight, self._stray = max_in_flight, 0, 0
        self._lock = threading.Lock()
        self._pool = ThreadPoolExecutor(max_workers=workers, thread_name_prefix="score")
        missing = store.missing_for(model.feature_list)
        if missing:  # không bao giờ điền 0 cho cột store không tính được
            raise ValueError(f"feature store không phủ các cột mô hình cần: {missing}")
        # Điểm mặc định = tỉ lệ gian lận nền của mô hình: «không biết gì thêm». `flagged` luôn False khi degraded.
        self.default_score = default_score if default_score is not None else float(model.meta["metrics_valid"]["base_rate"])
        self.stats: Counter[str] = Counter()

    def close(self) -> None:
        self._pool.shutdown(wait=False, cancel_futures=True)

    def _unstray(self, _future: Any = None) -> None:
        with self._lock:
            self._stray -= 1

    def _degraded(self, trip: ClosedTrip, reason: str, t0: float, read_ms: float = 0.0, detail: str | None = None) -> ScoreResult:
        self.stats["degraded"] += 1
        self.stats[f"degraded:{reason}"] += 1
        return ScoreResult(trip.trip_id, self.default_score, False, True, reason, self.model.version,
                           (time.perf_counter() - t0) * 1000, read_ms, 0.0, detail)

    async def score(self, trip: ClosedTrip, *, deadline_s: float | None = None) -> ScoreResult:
        """Chấm một chuyến. `deadline_s` là thời gian người gọi còn chờ (vd. `context.time_remaining()` của gRPC)."""
        t0 = time.perf_counter()
        # Giới hạn tính cả luồng mô hình còn chạy sau khi lời gọi của nó đã hết hạn (`_stray`): luồng không hủy được.
        if self.max_in_flight is not None and self._in_flight + self._stray >= self.max_in_flight:
            return self._degraded(trip, "overloaded", t0)  # từ chối ngay: xếp hàng quá ngân sách chỉ tốn việc vô ích
        if self.breaker is not None and not self.breaker.allow():
            return self._degraded(trip, "breaker_open", t0)
        self._in_flight += 1
        try:
            return await self._run(trip, t0, self.budget_s if deadline_s is None else min(self.budget_s, deadline_s))
        finally:
            self._in_flight -= 1

    async def _run(self, trip: ClosedTrip, t0: float, budget: float) -> ScoreResult:
        read_ms = model_ms = 0.0
        store_answered = False  # cầu dao chỉ đếm lỗi của STORE, không đếm mô hình chậm
        try:
            async with asyncio.timeout(budget):  # chỉ ngắt được tại một `await`
                got = await self.store.read_features(trip)
                store_answered = True
                read_ms = (time.perf_counter() - t0) * 1000
                if got.applied_until_us is None:  # store trống: mọi đặc trưng sẽ là 0, điểm thấp giả
                    self._store_ok()
                    return self._degraded(trip, "store_cold", t0, read_ms)
                if got.history_from_us is None or trip.paid_us - got.history_from_us < MIN_HISTORY_US:
                    self._store_ok()  # store đang được dựng lại: cửa sổ 30 ngày chưa đủ lịch sử
                    return self._degraded(trip, "store_building", t0, read_ms)
                if got.state == "ahead":  # chuyến cũ đã có chuyến sau: hash không còn đúng theo thời điểm
                    self._store_ok()
                    return self._degraded(trip, "state_ahead", t0, read_ms)
                t1 = time.perf_counter()
                if self.offload_model:
                    p = await self._offload(got.features)  # CPU: ra khỏi event loop
                else:
                    p = self.model.score_one(got.features)  # chạy thẳng trong event loop: chặn mọi coroutine khác
                model_ms = (time.perf_counter() - t1) * 1000
        except TimeoutError:
            if self.breaker is not None:
                self.breaker.failure() if not store_answered else self.breaker.success()
            return self._degraded(trip, "timeout", t0, read_ms)
        except (RedisError, OSError) as e:
            if self.breaker is not None:
                self.breaker.failure()
            return self._degraded(trip, "store_error", t0, read_ms, type(e).__name__)
        except Exception as e:  # đầu vào mô hình bị từ chối (None, NaN, thiếu cột) hoặc lỗi mô hình: vẫn phải trả lời
            self._store_ok()
            return self._degraded(trip, "model_error", t0, read_ms, f"{type(e).__name__}: {e}")
        self._store_ok()
        self.stats["ok"] += 1
        flag = bool(self.model.flag(p, float(trip.amount_vnd)))
        return ScoreResult(trip.trip_id, p, flag, False, None, self.model.version, (time.perf_counter() - t0) * 1000, read_ms, model_ms)

    def _store_ok(self) -> None:
        if self.breaker is not None:
            self.breaker.success()

    async def _offload(self, features: Any) -> float:
        """Chạy mô hình trong executor riêng. Lời gọi hết hạn thì luồng vẫn chạy tiếp; đếm nó vào `_stray` tới khi xong."""
        cf = self._pool.submit(self.model.score_one, features)
        try:
            return await asyncio.wrap_future(cf)
        except BaseException:
            if not cf.done():  # luồng đang chạy hoặc xếp hàng chưa kịp hủy
                with self._lock:
                    self._stray += 1
                cf.add_done_callback(self._unstray)
            raise
