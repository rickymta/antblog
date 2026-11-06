"""Miền chung của lab 19 (copy từ các bài trước, giữ nguyên tên).

Cước = giá cơ bản + phần vượt x giá mỗi km vượt, làm tròn cuối ROUND_HALF_UP.
Mô hình học tập: không phụ phí, thuế, giảm giá; ba dịch vụ cùng giá.
"""

from decimal import ROUND_HALF_UP, Decimal
from enum import StrEnum
from typing import NamedTuple

MAX_KM = Decimal("100")


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
    """Cước một chuyến. `service` có trong chữ ký để chỗ gọi không phải đổi khi sau này
    có quy tắc riêng từng dịch vụ; hiện tại nó chưa làm đổi giá (ba dịch vụ cùng giá)."""
    if not distance_km.is_finite() or not Decimal("0") <= distance_km <= MAX_KM:
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    extra_km = max(Decimal("0"), distance_km - policy.included_km)
    total = policy.base_fare + extra_km * policy.price_per_extra_km
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)
