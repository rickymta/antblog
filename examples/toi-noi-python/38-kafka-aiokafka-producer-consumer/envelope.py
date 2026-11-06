"""Hợp đồng sự kiện BookingCreated: envelope JSON có `version`, người đọc khoan dung.

Envelope: {"event_id", "event_type", "occurred_at" (ISO UTC), "version", "payload": {...}}.
- version 1: payload có trip_id, customer_id, service, distance_km, requested_at.
- version 2: thêm device_id (tùy chọn). Thêm field tùy chọn là thay đổi tương thích.
- Consumer đọc được version trong SUPPORTED; version lạ bị từ chối có lý do, không đoán.
"""

import json
import random
import uuid
from dataclasses import dataclass
from datetime import UTC, datetime
from decimal import Decimal, InvalidOperation
from enum import StrEnum

EVENT_TYPE = "BookingCreated"
TOPIC = "booking.created.v1"  # lab đặt thêm tiền tố "lab38." khi chạy
SUPPORTED = frozenset({1, 2})


class ServiceKind(StrEnum):
    RIDE = "ride"
    FOOD = "food"
    PARCEL = "parcel"


@dataclass(frozen=True)
class BookingCreated:
    trip_id: str
    customer_id: str
    service: ServiceKind
    distance_km: Decimal
    requested_at: datetime
    device_id: str | None = None  # có từ version 2


@dataclass(frozen=True)
class Decoded:
    event_id: str
    version: int
    occurred_at: datetime
    event: BookingCreated


class EnvelopeError(ValueError):
    """Thông điệp không đọc được; `reason` là mã ổn định cho nhật ký và metric."""

    def __init__(self, reason: str, detail: str = "") -> None:
        super().__init__(f"{reason}: {detail}" if detail else reason)
        self.reason = reason


def make_event_id(rng: random.Random) -> str:
    """UUID v4 sinh từ rng có seed để output của lab lặp lại được. Production dùng uuid4()."""
    return str(uuid.UUID(int=rng.getrandbits(128), version=4))


def encode(event: BookingCreated, *, event_id: str, occurred_at: datetime, version: int = 1) -> bytes:
    payload = {
        "trip_id": event.trip_id,
        "customer_id": event.customer_id,
        "service": event.service.value,
        "distance_km": str(event.distance_km),  # tiền/độ dài chính xác đi dưới dạng chuỗi
        "requested_at": event.requested_at.astimezone(UTC).isoformat(),
    }
    if version >= 2 and event.device_id is not None:
        payload["device_id"] = event.device_id
    envelope = {
        "event_id": event_id,
        "event_type": EVENT_TYPE,
        "occurred_at": occurred_at.astimezone(UTC).isoformat(),
        "version": version,
        "payload": payload,
    }
    return json.dumps(envelope, ensure_ascii=False, separators=(",", ":")).encode("utf-8")


def decode(raw: bytes) -> Decoded:
    """Bytes -> Decoded; mọi lỗi định dạng thành EnvelopeError, không để lọt exception lạ."""
    try:
        env = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise EnvelopeError("invalid_json", str(exc)) from None
    if not isinstance(env, dict):
        raise EnvelopeError("invalid_envelope", "không phải object")
    if env.get("event_type") != EVENT_TYPE:
        raise EnvelopeError("wrong_event_type", repr(env.get("event_type")))
    version = env.get("version")
    # Kiểm kiểu trước: list/dict không băm được (TypeError khi tra SUPPORTED), True == 1 lọt qua.
    if type(version) is not int or version not in SUPPORTED:
        # Kiểm TRƯỚC khi đọc payload: payload của version lạ có thể đã đổi nghĩa.
        raise EnvelopeError("unsupported_version", repr(version))
    try:
        p = env["payload"]
        event = BookingCreated(
            trip_id=p["trip_id"],
            customer_id=p["customer_id"],
            service=ServiceKind(p["service"]),
            distance_km=Decimal(p["distance_km"]),
            requested_at=datetime.fromisoformat(p["requested_at"]),
            device_id=p.get("device_id"),  # v1 không có -> None; field lạ khác bị bỏ qua
        )
        return Decoded(
            event_id=env["event_id"],
            version=version,
            occurred_at=datetime.fromisoformat(env["occurred_at"]),
            event=event,
        )
    except (KeyError, TypeError, ValueError, InvalidOperation) as exc:
        raise EnvelopeError("invalid_payload", f"{type(exc).__name__}: {exc}") from None
