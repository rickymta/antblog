"""Chấm điểm giả cho chuyến (quy tắc đơn giản) và backend giả có thể lỗi/chậm theo kế hoạch của lab.

Điểm ở đây là ĐẦU VÀO cho người duyệt, không phải kết luận gian lận.
"""

import asyncio
from decimal import Decimal, InvalidOperation

from redis.asyncio import Redis

from app.settings import BACKEND_TIMEOUT_S, BOOM, FAULT, HANG, RUNS, SLOW


class TransientScoringError(Exception):
    """Backend chấm điểm tạm thời lỗi (503, timeout): thử lại có thể thành công."""


class InvalidTripError(Exception):
    """Đầu vào sai: thử lại bao nhiêu lần cũng vậy, đưa thẳng vào dead-letter."""


MODEL_VERSION = "rules-2026.11.1"  # cùng quy ước với service fraud-scoring ở bài 37


def compute_score(distance_km: Decimal) -> dict[str, object]:
    score = Decimal("0.05") + Decimal("0.008") * distance_km
    reasons: list[str] = []  # mã ổn định cho máy và người duyệt, không phải câu văn (như bài 37)
    if distance_km >= 50:
        score += Decimal("0.4")
        reasons.append("long_trip")
    return {"score": str(min(score, Decimal("1")).quantize(Decimal("0.001"))), "reasons": reasons, "model_version": MODEL_VERSION}


def parse_distance(raw: str) -> Decimal:
    try:
        km = Decimal(raw)
    except InvalidOperation as exc:
        raise InvalidTripError(f"distance_km không phải số: {raw!r}") from exc
    if not km.is_finite():  # "NaN", "sNaN", "Infinity" đều là Decimal hợp lệ nhưng không phải khoảng cách
        raise InvalidTripError(f"distance_km không hữu hạn: {raw!r}")
    if not Decimal("0") <= km <= Decimal("100"):
        raise InvalidTripError(f"khoảng cách ngoài miền 0..100 km: {km}")
    return km


async def call_backend(redis: Redis, trip_id: str, distance_km: str) -> dict[str, object]:
    """Gọi «dịch vụ chấm điểm». Kế hoạch lỗi/chậm nằm trong Redis để lab điều khiển từ bên ngoài."""
    km = parse_distance(distance_km)  # lỗi vĩnh viễn: bắt trước khi tốn một lần gọi backend
    runs = int(await redis.get(RUNS + trip_id) or 0)  # đã tăng ở đầu job
    try:
        # Backend phải có hạn chót RIÊNG, ngắn hơn job_timeout của arq. Job quá job_timeout bị arq
        # kết thúc ở trạng thái thất bại, không retry và không đi qua code dead-letter của ta.
        async with asyncio.timeout(BACKEND_TIMEOUT_S):
            if runs <= int(await redis.get(HANG + trip_id) or 0):
                await asyncio.sleep(30)  # backend treo
            slow = await redis.get(SLOW + trip_id)
            if slow:
                await asyncio.sleep(float(slow))
    except TimeoutError as exc:
        raise TransientScoringError(f"backend quá {BACKEND_TIMEOUT_S} giây ở lần chạy {runs}") from exc
    if await redis.get(BOOM + trip_id):
        raise RuntimeError("lỗi không lường trước trong backend")
    if runs <= int(await redis.get(FAULT + trip_id) or 0):
        raise TransientScoringError(f"backend 503 ở lần chạy {runs}")
    return compute_score(km)
