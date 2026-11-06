"""Cước học tập của Tới Nơi (sao chép từ bài 10): 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt."""

from decimal import ROUND_HALF_UP, Decimal
from enum import StrEnum
from typing import NamedTuple


class ServiceKind(StrEnum):
    RIDE = "ride"
    FOOD = "food"
    PARCEL = "parcel"


class FarePolicy(NamedTuple):
    base_fare: Decimal
    included_km: Decimal
    price_per_extra_km: Decimal


BASE_FARE = Decimal("12000")
INCLUDED_KM = Decimal("2")
PRICE_PER_EXTRA_KM = Decimal("4500")
STANDARD = FarePolicy(BASE_FARE, INCLUDED_KM, PRICE_PER_EXTRA_KM)


def calculate_fare(distance_km: Decimal, policy: FarePolicy = STANDARD, *, service: ServiceKind = ServiceKind.RIDE) -> Decimal:
    ServiceKind(service)  # dịch vụ lạ ném ValueError
    if not Decimal("0") <= distance_km <= Decimal("100"):
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    extra_km = max(Decimal("0"), distance_km - policy.included_km)
    total = policy.base_fare + extra_km * policy.price_per_extra_km
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)
