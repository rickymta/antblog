"""Miền báo giá của Tới Nơi (copy từ bài 31, thêm hai exception có kiểu). Không biết gì về HTTP.

Mô hình cước và thời hạn báo giá là giả định học tập, không phải bảng giá vận hành.
"""

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


class DistanceOutOfRange(ValueError):
    """Vẫn là ValueError với thông điệp cũ của loạt bài, nhưng có kiểu riêng để tầng HTTP nhận ra."""

    def __init__(self, distance_km: Decimal) -> None:
        super().__init__(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
        self.distance_km = distance_km


class QuoteNotFound(LookupError):
    def __init__(self, quote_id: str) -> None:
        super().__init__(f"không có báo giá {quote_id}")
        self.quote_id = quote_id


class FarePolicy(NamedTuple):
    base_fare: Decimal = Decimal("12000")
    included_km: Decimal = Decimal("2")
    price_per_extra_km: Decimal = Decimal("4500")


STANDARD = FarePolicy()


def calculate_fare(
    distance_km: Decimal,
    policy: FarePolicy = STANDARD,
    *,
    service: ServiceKind = ServiceKind.RIDE,
) -> Decimal:
    """Cước (VND), làm tròn ROUND_HALF_UP; ngoài miền 0..100 km thì DistanceOutOfRange (là ValueError)."""
    if not distance_km.is_finite() or not Decimal("0") <= distance_km <= MAX_KM:
        raise DistanceOutOfRange(distance_km)
    ServiceKind(service)  # dịch vụ lạ bị ValueError thường, không phải DistanceOutOfRange
    extra_km = max(Decimal("0"), distance_km - policy.included_km)
    total = policy.base_fare + extra_km * policy.price_per_extra_km
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)


@dataclass(frozen=True)
class Money:
    amount: Decimal
    currency: str = "VND"


@dataclass(frozen=True)
class Quote:
    """Báo giá có hạn: hết hạn thì báo lại, không sửa báo giá cũ."""

    trip_distance_km: Decimal
    fare: Money
    expires_at: datetime  # aware, UTC

    def is_valid(self, now: datetime) -> bool:
        return now < self.expires_at
