"""Cấu hình worker arq. Chạy bằng: arq app.worker.WorkerSettings (từ thư mục lab)."""

import os

from arq import cron
from arq.worker import func

from app.jobs import report_queue_health, score_trip
from app.settings import MAX_TRIES, QUEUE, deserialize, redis_settings, serialize


class WorkerSettings:
    functions = [func(score_trip, max_tries=MAX_TRIES)]
    # Theo lịch: mỗi phút vào giây 0 và 30, cộng một lần ngay khi worker khởi động.
    cron_jobs = [cron(report_queue_health, name="lab40:report_queue_health", second={0, 30}, run_at_startup=True)]
    redis_settings = redis_settings()
    queue_name = QUEUE
    max_tries = MAX_TRIES
    max_jobs = 5  # tối đa 5 job chạy đồng thời trong một worker
    job_timeout = 10  # giây; quá hạn thì job bị hủy
    keep_result = 600  # giữ kết quả 10 phút
    poll_delay = 0.1
    # SIGTERM: ngừng nhận job mới, cho job đang chạy tối đa ngần này giây rồi mới hủy.
    job_completion_wait = int(os.environ.get("LAB40_GRACE_S", "10"))
    job_serializer = serialize
    job_deserializer = deserialize
