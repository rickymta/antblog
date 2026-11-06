"""Miền chuyến xe của bài 16, theo từ điển miền (lab tự chứa; không mang máy trạng thái đóng gói của bài 11)."""

from dataclasses import dataclass
from datetime import UTC, datetime
from decimal import Decimal
from enum import StrEnum


class TripStatus(StrEnum):
    REQUESTED = "requested"
    SEARCHING = "searching"
    ACCEPTED = "accepted"
    ARRIVING = "arriving"
    PICKED_UP = "picked_up"
    IN_TRANSIT = "in_transit"
    COMPLETED = "completed"
    CANCELLED = "cancelled"


class ServiceKind(StrEnum):
    RIDE = "ride"
    FOOD = "food"
    PARCEL = "parcel"


@dataclass
class Trip:
    """Thứ tự trường cũng là `__match_args__`, nên `Trip("TN-0001", ...)` dùng được trong pattern.

    driver_id là None cho tới khi có tài xế nhận chuyến: None ở đây mang nghĩa nghiệp vụ.
    """

    trip_id: str
    customer_id: str
    driver_id: str | None
    service: ServiceKind
    distance_km: Decimal
    status: TripStatus
    created_at: datetime


def make_trip(
    status: TripStatus = TripStatus.REQUESTED,
    driver_id: str | None = None,
    distance_km: str = "5",
    trip_id: str = "TN-0001",
) -> Trip:
    return Trip(
        trip_id=trip_id,
        customer_id="CU-001",
        driver_id=driver_id,
        service=ServiceKind.RIDE,
        distance_km=Decimal(distance_km),
        status=status,
        created_at=datetime(2026, 11, 6, 3, 0, tzinfo=UTC),
    )
