"""Chính sách cước Tới Nơi (mô hình học tập, copy rút gọn từ bài 10)."""

from decimal import ROUND_HALF_UP, Decimal
from enum import StrEnum
from typing import NamedTuple

MAX_KM = Decimal("100")


class ServiceKind(StrEnum):
    """Loại dịch vụ (bài 13). Mô hình học tập chưa đổi giá theo dịch vụ."""

    RIDE = "ride"
    FOOD = "food"
    PARCEL = "parcel"


class FarePolicy(NamedTuple):
    """Ba tham số của chính sách; bất biến nên chia sẻ giữa các đối tượng được."""

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
    """Cước một chuyến: hàm thuần, không đọc giờ, không đọc file, không biết thành phố."""
    if not Decimal("0") <= distance_km <= MAX_KM:
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    extra_km = max(Decimal("0"), distance_km - policy.included_km)
    total = policy.base_fare + extra_km * policy.price_per_extra_km
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)
