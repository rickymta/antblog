"""Cấu hình của lab 40: Redis dùng chung, mọi key của lab có tiền tố `lab40:`."""

import json
import os
from typing import Any

from arq import create_pool
from arq.connections import ArqRedis, RedisSettings

REDIS_URL = os.environ.get("TOINOI_REDIS_URL", "redis://localhost:63790/0")
PREFIX = "lab40:"
QUEUE = PREFIX + "queue"  # sorted set của arq: job_id -> thời điểm được phép chạy
DLQ = PREFIX + "dlq"  # dead-letter: list Redis, mỗi phần tử là một JSON
HEALTH = PREFIX + "health"  # cron ghi vào đây
SCORE = PREFIX + "score:"  # + trip_id -> kết quả chấm điểm (hiệu ứng bền của job)
EFFECTS = PREFIX + "effects"  # đếm số lần hiệu ứng THẬT sự xảy ra
RUNS = PREFIX + "runs:"  # + trip_id -> số lần hàm job được gọi (để lab quan sát)
FAULT = PREFIX + "fault:"  # + trip_id -> số lần gọi đầu tiên của backend giả sẽ lỗi (99 = luôn lỗi)
SLOW = PREFIX + "slow:"  # + trip_id -> số giây backend giả chậm
HANG = PREFIX + "hang:"  # + trip_id -> số lần chạy đầu tiên backend giả treo (ngủ 30 giây)
BOOM = PREFIX + "boom:"  # + trip_id -> có khóa này thì backend giả ném lỗi không lường trước

MAX_TRIES = 4  # tổng số lần chạy tối đa của một job (lần đầu + 3 lần thử lại)
BACKOFF_BASE_S = 0.2  # lab dùng số nhỏ để chạy nhanh; production tính bằng giây đến phút
BACKOFF_CAP_S = 2.0
BACKEND_TIMEOUT_S = 2.0  # phải NGẮN HƠN job_timeout (10 giây) của worker, xem app/worker.py


def redis_settings() -> RedisSettings:
    return RedisSettings.from_dsn(REDIS_URL)


# arq mặc định serialize bằng pickle; giải pickle dữ liệu từ Redis là chạy mã của người ghi vào Redis.
# Lab dùng JSON: job chỉ mang chuỗi, xem được bằng redis-cli, và không có đường thực thi mã.
def serialize(data: Any) -> bytes:
    return json.dumps(data, default=repr).encode()  # exception không phải JSON: lưu repr


def deserialize(raw: bytes) -> Any:
    return json.loads(raw)


async def make_pool() -> ArqRedis:
    return await create_pool(
        redis_settings(),
        default_queue_name=QUEUE,
        job_serializer=serialize,
        job_deserializer=deserialize,
    )
