"""models.py: dữ liệu và hằng số của mô hình cước Tới Nơi (không import module nào khác trong gói)."""

from decimal import Decimal
from typing import NamedTuple


class FarePolicy(NamedTuple):
    """Chính sách cước: tuple có tên, bất biến."""

    base_fare: Decimal
    included_km: Decimal
    price_per_extra_km: Decimal
    max_km: Decimal


class Quote(NamedTuple):
    """Kết quả một báo giá: khoảng cách đầu vào và số tiền VND đã làm tròn."""

    distance_km: Decimal
    total_vnd: Decimal


# Mô hình minh họa dùng chung cả loạt: không phải bảng giá vận hành thực tế.
DEFAULT_POLICY = FarePolicy(
    base_fare=Decimal("12000"),
    included_km=Decimal("2"),
    price_per_extra_km=Decimal("4500"),
    max_km=Decimal("100"),
)
