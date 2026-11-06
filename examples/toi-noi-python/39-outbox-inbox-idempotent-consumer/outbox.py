"""Phía phát: ghi chuyến + outbox cùng transaction, và dispatcher đẩy outbox ra Kafka."""

import json
import uuid
from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import Any

from aiokafka import AIOKafkaProducer
from sqlalchemy import Engine, select, update
from sqlalchemy.orm import Session

from fare import calculate_fare
from kafka_io import SimulatedCrash
from models import OutboxEvent, Trip, TripStatus
from settings import TOPIC


class TripNotInTransit(Exception):
    pass


def trip_completed_envelope(trip: Trip, *, event_id: str, occurred_at: datetime) -> dict[str, Any]:
    """Envelope theo từ điển miền: event_id, event_type, occurred_at (UTC), version, payload."""
    return {
        "event_id": event_id,
        "event_type": "TripCompleted",
        "occurred_at": occurred_at.isoformat(),
        "version": 1,
        "payload": {
            "trip_id": trip.trip_id,
            "customer_id": trip.customer_id,
            "driver_id": trip.driver_id,
            "fare": str(trip.fare),  # Decimal đi qua JSON dưới dạng chuỗi
            "currency": "VND",
        },
    }


def complete_trip(engine: Engine, trip_id: str, *, now: datetime, fail_before_commit: bool = False) -> str:
    """Hoàn thành chuyến VÀ ghi ý định phát sự kiện trong CÙNG MỘT transaction. Trả event_id."""
    with Session(engine) as session, session.begin():
        trip = session.get(Trip, trip_id, with_for_update=True)
        if trip is None or trip.status != TripStatus.IN_TRANSIT:
            raise TripNotInTransit(trip_id)
        trip.status = TripStatus.COMPLETED
        trip.completed_at = now
        trip.fare = calculate_fare(trip.distance_km)
        event_id = str(uuid.uuid4())  # sinh MỘT lần, lưu cùng outbox; retry không bao giờ sinh id mới
        session.add(
            OutboxEvent(
                event_id=event_id,
                topic=TOPIC,
                message_key=trip.trip_id,  # cùng chuyến luôn vào cùng partition, giữ thứ tự theo chuyến
                envelope=trip_completed_envelope(trip, event_id=event_id, occurred_at=now),
                created_at=now,
            )
        )
        if fail_before_commit:
            raise RuntimeError("mô phỏng lỗi trước khi commit")
    return event_id


@dataclass(frozen=True)
class Claimed:
    seq: int
    event_id: str
    topic: str
    key: str
    envelope: dict[str, Any]


def claim_batch(session: Session, *, now: datetime, lease: timedelta, limit: int) -> list[Claimed]:
    """«Thuê» tối đa `limit` dòng chưa phát và chưa bị dispatcher khác thuê.

    Một câu UPDATE duy nhất: `FOR UPDATE SKIP LOCKED` làm hai dispatcher chạy đồng thời nhận
    hai tập dòng rời nhau, `lease_until` giữ chỗ sau khi transaction này commit.
    """
    pending = (
        select(OutboxEvent.seq)
        .where(OutboxEvent.published_at.is_(None))
        .where((OutboxEvent.lease_until.is_(None)) | (OutboxEvent.lease_until < now))
        .order_by(OutboxEvent.seq)
        .limit(limit)
        .with_for_update(skip_locked=True)
    )
    rows = session.execute(
        update(OutboxEvent)
        .where(OutboxEvent.seq.in_(pending))
        .values(lease_until=now + lease, attempts=OutboxEvent.attempts + 1)
        .returning(OutboxEvent.seq, OutboxEvent.event_id, OutboxEvent.topic, OutboxEvent.message_key, OutboxEvent.envelope)
    ).all()
    # RETURNING không bảo đảm thứ tự: sắp lại theo seq trước khi phát.
    return sorted((Claimed(*row) for row in rows), key=lambda c: c.seq)


@dataclass(frozen=True)
class DispatchReport:
    claimed: int
    published: int


async def dispatch_once(
    engine: Engine,
    producer: AIOKafkaProducer,
    *,
    now: datetime,
    lease: timedelta = timedelta(seconds=30),
    batch: int = 100,
    crash_after_publish: int | None = None,
) -> DispatchReport:
    """Một vòng của dispatcher: thuê dòng -> phát (ngoài transaction) -> đánh dấu đã phát.

    `crash_after_publish=k` mô phỏng chết sau khi phát k thông điệp và trước khi đánh dấu:
    đó chính là cửa sổ tạo ra thông điệp trùng (at-least-once).
    """
    with Session(engine) as session, session.begin():
        claimed = claim_batch(session, now=now, lease=lease, limit=batch)
    # Transaction thuê đã commit; từ đây không giữ khóa hay transaction nào trong lúc gọi Kafka.
    published: list[int] = []
    for item in claimed:
        value = json.dumps(item.envelope, ensure_ascii=False, separators=(",", ":")).encode()
        await producer.send_and_wait(item.topic, value, key=item.key.encode())
        published.append(item.seq)
        if crash_after_publish is not None and len(published) >= crash_after_publish:
            raise SimulatedCrash(f"dispatcher chết sau khi phát {len(published)} thông điệp, trước khi đánh dấu")
    if published:
        with Session(engine) as session, session.begin():
            session.execute(update(OutboxEvent).where(OutboxEvent.seq.in_(published)).values(published_at=now))
    return DispatchReport(claimed=len(claimed), published=len(published))
