"""Miền Tới Nơi dùng trong lab: cước và báo giá (bài 31, 32), trạng thái chuyến (bài 13). Tự chứa, chép từ các lab trước."""

from dataclasses import dataclass
from datetime import datetime, timedelta
from decimal import ROUND_HALF_UP, Decimal
from enum import StrEnum
from typing import NamedTuple

MAX_KM = Decimal("100")
QUOTE_TTL = timedelta(minutes=5)  # giả định học tập: báo giá sống 5 phút


class ServiceKind(StrEnum):
    RIDE = "ride"
    FOOD = "food"
    PARCEL = "parcel"


class TripStatus(StrEnum):
    REQUESTED = "requested"
    SEARCHING = "searching"
    ACCEPTED = "accepted"
    ARRIVING = "arriving"
    PICKED_UP = "picked_up"
    IN_TRANSIT = "in_transit"
    COMPLETED = "completed"
    CANCELLED = "cancelled"


# Chỉ các trạng thái này mới hủy được (từ điển miền).
CANCELLABLE = {TripStatus.REQUESTED, TripStatus.SEARCHING, TripStatus.ACCEPTED, TripStatus.ARRIVING}


class DistanceOutOfRange(ValueError):
    def __init__(self, distance_km: Decimal) -> None:
        super().__init__(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
        self.distance_km = distance_km


class QuoteNotFound(LookupError):
    def __init__(self, quote_id: str) -> None:
        super().__init__(f"không có báo giá {quote_id}")
        self.quote_id = quote_id


class TripNotFound(LookupError):
    def __init__(self, trip_id: str) -> None:
        super().__init__(f"không có chuyến {trip_id}")
        self.trip_id = trip_id


class InvalidTransition(Exception):
    pass


class FarePolicy(NamedTuple):
    base_fare: Decimal = Decimal("12000")
    included_km: Decimal = Decimal("2")
    price_per_extra_km: Decimal = Decimal("4500")


STANDARD = FarePolicy()


def calculate_fare(distance_km: Decimal, policy: FarePolicy = STANDARD, *,
                   service: ServiceKind = ServiceKind.RIDE) -> Decimal:
    """Cước (VND), làm tròn ROUND_HALF_UP; ngoài miền thì DistanceOutOfRange, dịch vụ lạ thì ValueError."""
    if not distance_km.is_finite() or not Decimal("0") <= distance_km <= MAX_KM:
        raise DistanceOutOfRange(distance_km)
    ServiceKind(service)  # dịch vụ lạ bị ValueError thường, không phải DistanceOutOfRange
    extra_km = max(Decimal("0"), distance_km - policy.included_km)
    total = policy.base_fare + extra_km * policy.price_per_extra_km
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)


@dataclass
class Trip:
    trip_id: str
    customer_id: str
    driver_id: str | None
    service: ServiceKind
    distance_km: Decimal
    status: TripStatus
    created_at: datetime

    def cancel(self) -> None:
        if self.status not in CANCELLABLE:
            raise InvalidTransition(f"không thể hủy chuyến đang ở trạng thái {self.status.value}")
        self.status = TripStatus.CANCELLED
