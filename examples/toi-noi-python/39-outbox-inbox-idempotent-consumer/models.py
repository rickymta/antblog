"""Bốn bảng của lab 39, tất cả trong schema `lab39`.

Phía phát (service chuyến): `trips`, `outbox`.  Phía nhận (service thanh toán tài xế):
`inbox`, `payout_lines`.  Lab đặt cả hai phía trong một database cho gọn; ở production mỗi
service sở hữu database riêng và chỉ nói chuyện với nhau qua Kafka.
"""

from datetime import datetime
from enum import StrEnum
from decimal import Decimal
from typing import Any

from sqlalchemy import BigInteger, DateTime, Identity, MetaData, Numeric, String
from sqlalchemy.dialects.postgresql import JSONB
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column

from settings import SCHEMA


class TripStatus(StrEnum):
    IN_TRANSIT = "in_transit"
    COMPLETED = "completed"


class Base(DeclarativeBase):
    metadata = MetaData(schema=SCHEMA)


class Trip(Base):
    __tablename__ = "trips"

    trip_id: Mapped[str] = mapped_column(String(16), primary_key=True)
    customer_id: Mapped[str] = mapped_column(String(16))
    driver_id: Mapped[str] = mapped_column(String(16))
    service: Mapped[str] = mapped_column(String(16), default="ride")  # ServiceKind
    distance_km: Mapped[Decimal] = mapped_column(Numeric(7, 3))
    fare: Mapped[Decimal | None] = mapped_column(Numeric(12, 0))
    status: Mapped[str] = mapped_column(String(16), default=TripStatus.IN_TRANSIT)
    completed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))


class OutboxEvent(Base):
    """Ý định phát một sự kiện, ghi cùng transaction với thay đổi nghiệp vụ.

    `seq` cho thứ tự phát; `event_id` là danh tính ổn định của sự kiện (consumer dùng nó để khử
    trùng); `published_at` NULL nghĩa là chưa phát; `lease_until` là «thuê» của một dispatcher.
    """

    __tablename__ = "outbox"

    seq: Mapped[int] = mapped_column(BigInteger, Identity(), primary_key=True)
    event_id: Mapped[str] = mapped_column(String(36), unique=True)
    topic: Mapped[str] = mapped_column(String(128))
    message_key: Mapped[str] = mapped_column(String(64))
    envelope: Mapped[dict[str, Any]] = mapped_column(JSONB)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True))
    published_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    lease_until: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    attempts: Mapped[int] = mapped_column(default=0)


class InboxEntry(Base):
    """Dấu «consumer này đã xử lý sự kiện này». Khóa chính ghép (consumer, event_id)."""

    __tablename__ = "inbox"

    consumer: Mapped[str] = mapped_column(String(64), primary_key=True)
    event_id: Mapped[str] = mapped_column(String(36), primary_key=True)
    received_at: Mapped[datetime] = mapped_column(DateTime(timezone=True))


class ScoreRequest(Base):
    """Hiệu ứng của consumer fraud-scoring: một yêu cầu chấm điểm cho chuyến vừa xong."""

    __tablename__ = "score_requests"

    id: Mapped[int] = mapped_column(Identity(), primary_key=True)
    consumer: Mapped[str] = mapped_column(String(64))
    trip_id: Mapped[str] = mapped_column(String(16))
    event_id: Mapped[str] = mapped_column(String(36))


class PayoutLine(Base):
    """Hiệu ứng phía consumer: một dòng ghi nhận tiền chuyến cho tài xế.

    Cố ý KHÔNG có UNIQUE trên trip_id: để thấy chuyện gì xảy ra khi không có inbox. Ở production
    ta nên thêm khóa tự nhiên làm lớp phòng thủ thứ hai (xem phần bài tập).
    """

    __tablename__ = "payout_lines"

    id: Mapped[int] = mapped_column(Identity(), primary_key=True)
    consumer: Mapped[str] = mapped_column(String(64))
    driver_id: Mapped[str] = mapped_column(String(16))
    trip_id: Mapped[str] = mapped_column(String(16))
    amount: Mapped[Decimal] = mapped_column(Numeric(12, 0))
    event_id: Mapped[str] = mapped_column(String(36))
