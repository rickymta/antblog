"""Vận hành dead-letter: xem và đưa lại hàng đợi (redrive) sau khi nguyên nhân đã được sửa."""

import json
from typing import Any

from arq.connections import ArqRedis

from app.settings import DLQ


async def peek(redis: ArqRedis) -> list[dict[str, Any]]:
    return [json.loads(raw) for raw in await redis.lrange(DLQ, 0, -1)]


async def redrive(redis: ArqRedis) -> list[str]:
    """Đưa lại hàng đợi các phần tử hết lượt thử (`max_tries`); lỗi `permanent` ở yên đó.

    Job_id cũ không dùng lại được: arq từ chối enqueue khi còn khóa kết quả của job_id đó, nên
    ta enqueue với job_id MỚI. Idempotency thật vẫn do hiệu ứng trong job bảo vệ (SET NX).
    """
    requeued: list[str] = []
    for raw in await redis.lrange(DLQ, 0, -1):
        entry = json.loads(raw)
        if entry["reason"] != "max_tries":
            continue
        job = await redis.enqueue_job(
            "score_trip", entry["trip_id"], entry["distance_km"], _job_id=f"{entry['job_id']}:redrive"
        )
        if job is not None:
            await redis.lrem(DLQ, 1, raw)  # chỉ gỡ khỏi dead-letter khi đã enqueue được
            requeued.append(job.job_id)
    return requeued
