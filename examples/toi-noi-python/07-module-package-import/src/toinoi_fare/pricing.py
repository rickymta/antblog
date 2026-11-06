"""pricing.py: quy tắc tính cước. Nhập models bằng import tương đối (cùng gói)."""

from decimal import ROUND_HALF_UP, Decimal

from .models import DEFAULT_POLICY, FarePolicy, Quote


def calculate_fare(distance_km: Decimal | str | int, policy: FarePolicy = DEFAULT_POLICY) -> Decimal:
    """Tính cước (VND) cho quãng đường km, làm tròn ROUND_HALF_UP về 1 đồng."""
    distance = Decimal(distance_km)
    if distance < 0 or distance > policy.max_km:
        raise ValueError(f"khoảng cách ngoài miền 0..{policy.max_km} km: {distance}")
    extra = max(Decimal(0), distance - policy.included_km)
    raw = policy.base_fare + extra * policy.price_per_extra_km
    return raw.quantize(Decimal("1"), rounding=ROUND_HALF_UP)


def quote(distance_km: Decimal | str | int, policy: FarePolicy = DEFAULT_POLICY) -> Quote:
    """Đóng gói khoảng cách và cước thành một Quote; phần tính nằm ở calculate_fare."""
    return Quote(distance_km=Decimal(distance_km), total_vnd=calculate_fare(distance_km, policy))
