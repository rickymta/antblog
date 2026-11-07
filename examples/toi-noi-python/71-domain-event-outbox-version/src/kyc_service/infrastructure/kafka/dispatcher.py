"""Dispatcher: đọc outbox và đẩy ra Kafka, NGOÀI mọi transaction (xem bài 39 về cơ chế thuê dòng).

Ba bước mỗi vòng: thuê dòng (một câu UPDATE, commit ngay) -> phát -> đánh dấu đã phát. Chết giữa
bước phát và bước đánh dấu thì lần sau phát lại cùng `event_id`: Kafka nhận trùng, nên consumer
phải khử trùng (inbox, bài 39). Đó là giá của at-least-once.
"""

import json
from dataclasses import dataclass
from datetime import timedelta
from typing import Any

from aiokafka import AIOKafkaProducer
from sqlalchemy import select, update
from sqlalchemy.orm import Session, sessionmaker

from kyc_service.application.ports import Clock
from kyc_service.infrastructure.sql.tables import OutboxRow


class SimulatedCrash(Exception):
    """Mô phỏng tiến trình chết đúng giữa hai bước (chỉ dùng trong lab)."""


@dataclass(frozen=True)
class Claimed:
    seq: int
    event_id: str
    topic: str
    key: str
    envelope: dict[str, Any]


@dataclass(frozen=True)
class DispatchReport:
    claimed: int
    published: int


def claim_batch(session: Session, *, now: Any, lease: timedelta, limit: int) -> list[Claimed]:
    """«Thuê» tối đa `limit` dòng chưa phát và chưa bị dispatcher khác thuê.

    `FOR UPDATE SKIP LOCKED` làm hai dispatcher chạy đồng thời nhận hai tập dòng rời nhau;
    `lease_until` giữ chỗ sau khi transaction thuê commit.
    """
    pending = (
        select(OutboxRow.seq)
        .where(OutboxRow.published_at.is_(None))
        .where((OutboxRow.lease_until.is_(None)) | (OutboxRow.lease_until < now))
        .order_by(OutboxRow.seq)
        .limit(limit)
        .with_for_update(skip_locked=True)
    )
    rows = session.execute(
        update(OutboxRow)
        .where(OutboxRow.seq.in_(pending))
        .values(lease_until=now + lease, attempts=OutboxRow.attempts + 1)
        .returning(
            OutboxRow.seq,
            OutboxRow.event_id,
            OutboxRow.topic,
            OutboxRow.message_key,
            OutboxRow.envelope,
        )
    ).all()
    # RETURNING không bảo đảm thứ tự: sắp lại theo seq trước khi phát.
    return sorted((Claimed(*row) for row in rows), key=lambda c: c.seq)


class OutboxDispatcher:
    def __init__(
        self,
        session_factory: sessionmaker[Session],
        producer: AIOKafkaProducer,
        clock: Clock,
        *,
        lease: timedelta = timedelta(seconds=30),
        batch: int = 100,
    ) -> None:
        self._factory = session_factory
        self._producer = producer
        self._clock = clock
        self._lease = lease
        self._batch = batch

    async def dispatch_once(self, *, crash_after_publish: int | None = None) -> DispatchReport:
        now = self._clock.now()
        with self._factory() as session, session.begin():
            claimed = claim_batch(session, now=now, lease=self._lease, limit=self._batch)
        # Transaction thuê đã commit: từ đây không giữ khóa hay transaction nào khi gọi Kafka.
        published: list[int] = []
        for item in claimed:
            value = json.dumps(item.envelope, ensure_ascii=False, separators=(",", ":")).encode()
            await self._producer.send_and_wait(item.topic, value, key=item.key.encode())
            published.append(item.seq)
            if crash_after_publish is not None and len(published) >= crash_after_publish:
                raise SimulatedCrash(
                    f"chết sau khi phát {len(published)} thông điệp, trước khi đánh dấu"
                )
        if published:
            with self._factory() as session, session.begin():
                session.execute(
                    update(OutboxRow).where(OutboxRow.seq.in_(published)).values(published_at=now)
                )
        return DispatchReport(claimed=len(claimed), published=len(published))
