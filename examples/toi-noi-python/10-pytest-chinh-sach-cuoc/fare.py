"""Chính sách cước Tới Nơi (mô hình học tập, không phải bảng giá vận hành).

Cước = giá cơ bản + phần vượt x giá mỗi km vượt, làm tròn cuối ROUND_HALF_UP.
"""

from decimal import ROUND_HALF_UP, Decimal
from typing import NamedTuple

SERVICES = ("ride", "food", "parcel")
MAX_KM = Decimal("100")


class FarePolicy(NamedTuple):
    """Ba tham số của chính sách; bất biến, nên dùng chung giữa các test được."""

    base_fare: Decimal = Decimal("12000")
    included_km: Decimal = Decimal("2")
    price_per_extra_km: Decimal = Decimal("4500")


STANDARD = FarePolicy()


def calculate_fare(
    distance_km: Decimal | int,
    policy: FarePolicy = STANDARD,
    *,
    service: str = "ride",
) -> Decimal:
    """Tính cước cho một chuyến.

    Nhận Decimal hoặc int; từ chối float và str vì float làm hỏng độ chính xác
    của tiền. Raises:
        TypeError: kiểu của distance_km không được hỗ trợ.
        ValueError: ngoài miền 0..100 km, không hữu hạn, hoặc dịch vụ lạ.
    """
    if isinstance(distance_km, bool) or not isinstance(distance_km, (Decimal, int)):
        raise TypeError(f"distance_km phải là Decimal hoặc int, nhận {type(distance_km).__name__}")
    distance = Decimal(distance_km)
    if not distance.is_finite() or not Decimal("0") <= distance <= MAX_KM:
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    if service not in SERVICES:
        raise ValueError(f"service không hợp lệ: {service!r}")

    extra_km = max(Decimal("0"), distance - policy.included_km)
    total = policy.base_fare + extra_km * policy.price_per_extra_km
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)
