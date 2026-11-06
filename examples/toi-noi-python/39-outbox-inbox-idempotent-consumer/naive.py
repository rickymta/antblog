"""Hai cách ghi hai nơi KHÔNG qua outbox: để thấy cửa sổ dual-write mất hoặc sinh sự kiện «ma»."""

import json
from datetime import datetime

from aiokafka import AIOKafkaProducer
from sqlalchemy import Engine
from sqlalchemy.orm import Session

from fare import calculate_fare
from kafka_io import SimulatedCrash
from models import Trip, TripStatus
from outbox import trip_completed_envelope
from settings import NAIVE_TOPIC


async def _publish(producer: AIOKafkaProducer, trip: Trip, event_id: str, now: datetime) -> None:
    envelope = trip_completed_envelope(trip, event_id=event_id, occurred_at=now)
    await producer.send_and_wait(NAIVE_TOPIC, json.dumps(envelope).encode(), key=trip.trip_id.encode())


async def db_then_publish(
    engine: Engine, producer: AIOKafkaProducer, trip_id: str, *, now: datetime, crash_between: bool
) -> None:
    """Commit database trước, publish sau. Chết giữa hai bước: chuyến hoàn thành mà sự kiện mất."""
    with Session(engine, expire_on_commit=False) as session, session.begin():
        trip = session.get(Trip, trip_id)
        trip.status, trip.completed_at, trip.fare = TripStatus.COMPLETED, now, calculate_fare(trip.distance_km)
    if crash_between:
        raise SimulatedCrash("tiến trình chết sau COMMIT, trước khi publish")
    await _publish(producer, trip, "evt-naive-1", now)


async def publish_then_db(engine: Engine, producer: AIOKafkaProducer, trip_id: str, *, now: datetime) -> None:
    """Publish trước, commit database sau. Database lỗi: sự kiện «ma» về chuyến chưa hoàn thành."""
    with Session(engine) as session, session.begin():
        trip = session.get(Trip, trip_id)
        trip.fare = calculate_fare(trip.distance_km)
        await _publish(producer, trip, "evt-naive-ghost", now)  # thông điệp đã rời khỏi tiến trình
        raise SimulatedCrash("database lỗi sau khi đã publish")  # transaction rollback, Kafka thì không
