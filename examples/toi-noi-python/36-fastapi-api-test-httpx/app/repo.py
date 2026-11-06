"""Kho chuyến. Giao diện là Protocol để test thay bằng bản giả (bài 14)."""

from datetime import UTC, datetime
from decimal import Decimal
from typing import Protocol

from .domain import ServiceKind, Trip, TripStatus


class RepositoryUnavailable(Exception):
    """Kho tạm thời không dùng được (mất kết nối, hết giờ)."""


class TripRepository(Protocol):
    async def get(self, trip_id: str) -> Trip | None: ...
    async def save(self, trip: Trip) -> None: ...


class InMemoryTripRepository:
    def __init__(self, trips: list[Trip]) -> None:
        self._trips = {t.trip_id: t for t in trips}

    async def get(self, trip_id: str) -> Trip | None:
        return self._trips.get(trip_id)

    async def save(self, trip: Trip) -> None:
        self._trips[trip.trip_id] = trip


def seed_trips() -> list[Trip]:
    created = datetime(2026, 11, 6, 2, 0, tzinfo=UTC)
    return [
        Trip("TN-0001", "CU-001", "DR-001", ServiceKind.RIDE, Decimal("5"), TripStatus.ACCEPTED, created),
        Trip("TN-0002", "CU-002", None, ServiceKind.FOOD, Decimal("2.5"), TripStatus.REQUESTED, created),
        Trip("TN-0003", "CU-001", "DR-002", ServiceKind.PARCEL, Decimal("12"), TripStatus.IN_TRANSIT, created),
    ]
