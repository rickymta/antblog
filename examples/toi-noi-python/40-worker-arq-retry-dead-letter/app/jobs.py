"""Các job của worker: chấm điểm chuyến (retry + dead-letter + idempotent) và báo cáo định kỳ."""

import json
from datetime import UTC, datetime
from random import Random
from typing import Any

from arq import Retry
from redis.asyncio import Redis

from app.scoring import InvalidTripError, TransientScoringError, call_backend
from app.settings import BACKOFF_BASE_S, BACKOFF_CAP_S, DLQ, EFFECTS, HEALTH, MAX_TRIES, QUEUE, RUNS, SCORE


def backoff_delay(job_try: int, *, rng: Random | None = None) -> float:
    """Chờ bao lâu trước lần chạy tiếp theo: base * 2^(lần_vừa_thất_bại - 1), có trần.

    `rng` truyền vào thì dùng «full jitter» (ngẫu nhiên trong [0, delay]) để các job lỗi cùng lúc
    không cùng quay lại một thời điểm; mặc định không jitter để lab lặp lại được.
    """
    delay = min(BACKOFF_CAP_S, BACKOFF_BASE_S * 2 ** (job_try - 1))
    return rng.uniform(0, delay) if rng else delay


async def dead_letter(
    redis: Redis, ctx: dict[str, Any], trip_id: str, distance_km: str, reason: str, error: Exception
) -> None:
    entry = {
        "trip_id": trip_id,
        "distance_km": distance_km,  # đủ đối số để đưa lại hàng đợi sau khi sửa nguyên nhân
        "job_id": ctx["job_id"],
        "job_try": ctx["job_try"],
        "reason": reason,
        "error": f"{type(error).__name__}: {error}",
        "failed_at": datetime.now(UTC).isoformat(),
    }
    await redis.lpush(DLQ, json.dumps(entry, ensure_ascii=False))


async def score_trip(ctx: dict[str, Any], trip_id: str, distance_km: str) -> dict[str, Any]:
    """Chấm điểm một chuyến. Chạy lại bao nhiêu lần cũng chỉ có MỘT hiệu ứng cho mỗi trip_id."""
    redis: Redis = ctx["redis"]
    job_try: int = ctx["job_try"]
    await redis.incr(RUNS + trip_id)

    # Lớp 2a (lớp 1 là _job_id lúc enqueue): đã có kết quả bền cho chuyến này thì trả lại, không gọi backend.
    existing = await redis.get(SCORE + trip_id)
    if existing is not None:
        return {**json.loads(existing), "duplicate": True}

    try:
        result = await call_backend(redis, trip_id, distance_km)
    except InvalidTripError as exc:
        await dead_letter(redis, ctx, trip_id, distance_km, "permanent", exc)  # không thử lại: vô ích
        raise
    except TransientScoringError as exc:
        if job_try >= MAX_TRIES:
            # Lần cuối: arq sẽ không gọi lại hàm này (job_try > max_tries bị từ chối ngay),
            # nên đây là cơ hội duy nhất của ta để ghi dead-letter.
            await dead_letter(redis, ctx, trip_id, distance_km, "max_tries", exc)
            raise
        raise Retry(defer=backoff_delay(job_try)) from exc
    except Exception as exc:
        # Lỗi chưa phân loại: arq không thử lại và không báo ai, nên ta tự ghi dead-letter trước khi ném.
        await dead_letter(redis, ctx, trip_id, distance_km, "unexpected", exc)
        raise

    # Lớp 2b: SET NX. Hai job cùng chuyến chạy đồng thời thì đúng một job ghi được.
    if await redis.set(SCORE + trip_id, json.dumps(result), nx=True):
        await redis.incr(EFFECTS)
    return result


async def report_queue_health(ctx: dict[str, Any]) -> None:
    """Job theo lịch (cron): ghi nhanh độ dài hàng đợi và dead-letter."""
    redis: Redis = ctx["redis"]
    snapshot = {
        "queued": await redis.zcard(QUEUE),
        "dead_letters": await redis.llen(DLQ),
        "at": datetime.now(UTC).isoformat(),
    }
    await redis.set(HEALTH, json.dumps(snapshot))
