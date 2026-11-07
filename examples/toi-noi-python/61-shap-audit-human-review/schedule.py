"""Mô phỏng công suất người duyệt: mỗi ngày làm việc từ 08:00 đến 17:00 giờ Việt Nam, tối đa `per_day` đề xuất, cách đều nhau.

Hàm thuần (không đụng cơ sở dữ liệu): trả `decided_at` của từng đề xuất được duyệt, theo hai chính sách chọn việc
- `priority`: lấy đề xuất có chi phí kỳ vọng lớn nhất trong số đã vào hàng đợi;
- `fifo`: lấy đề xuất vào sớm nhất.
Cùng lịch này được dùng để chạy thật trên PostgreSQL và để so các mức công suất.
"""

import heapq
from dataclasses import dataclass
from datetime import datetime, timedelta

from synthetic_fraud import START

WORK_START_H, WORK_HOURS = 8, 9


@dataclass(frozen=True)
class Item:
    idx: int
    enqueued_at: datetime
    priority: float


def day_start(day: int) -> datetime:
    """00:00 giờ Việt Nam của ngày `day` kể từ START (START là 00:00 ngày 02/03/2026 giờ Việt Nam)."""
    return START + timedelta(days=day)


def schedule_reviews(items: list[Item], *, per_day: int, policy: str, first_day: int, last_day: int) -> dict[int, datetime]:
    """Duyệt từ `first_day` đến hết `last_day` (tính cả). Đề xuất không có trong kết quả là còn tồn đọng."""
    if per_day <= 0:
        return {}
    if policy not in ("priority", "fifo"):
        raise ValueError(policy)
    pending = sorted(items, key=lambda i: i.enqueued_at)
    nxt, heap, out = 0, [], {}
    step = timedelta(hours=WORK_HOURS) / per_day
    for day in range(first_day, last_day + 1):
        base = day_start(day) + timedelta(hours=WORK_START_H)
        for k in range(per_day):
            t = base + step * k
            while nxt < len(pending) and pending[nxt].enqueued_at <= t:
                it = pending[nxt]
                heapq.heappush(heap, ((-it.priority if policy == "priority" else it.enqueued_at.timestamp()), it.idx))
                nxt += 1
            if heap:
                out[heapq.heappop(heap)[1]] = t
    return out
