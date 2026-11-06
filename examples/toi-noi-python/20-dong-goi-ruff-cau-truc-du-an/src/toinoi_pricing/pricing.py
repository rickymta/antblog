"""Tính cước. Mô hình học tập: không phụ phí, thuế, giảm giá; ba dịch vụ cùng giá."""

from datetime import datetime, timedelta
from decimal import ROUND_HALF_UP, Decimal
from typing import NamedTuple

from toinoi_pricing.models import MAX_KM, Money, Quote, ServiceKind

QUOTE_TTL = timedelta(minutes=5)


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
    """Cước một chuyến, làm tròn cuối ROUND_HALF_UP về 1 VND.

    `distance_km` là Decimal: mypy chặn float/str ở chỗ gọi, nên không cần kiểm kiểu lúc chạy
    như bài 10. `service` chưa làm đổi giá.

    Raises:
        ValueError: ngoài miền 0..100 km hoặc không hữu hạn (NaN, Infinity).
    """
    if not distance_km.is_finite() or not Decimal("0") <= distance_km <= MAX_KM:
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    ServiceKind(service)  # dịch vụ lạ bị ValueError ngay tại đây, kể cả khi mypy bị bỏ qua
    extra_km = max(Decimal("0"), distance_km - policy.included_km)
    total = policy.base_fare + extra_km * policy.price_per_extra_km
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)


def make_quote(
    distance_km: Decimal,
    *,
    now: datetime,
    policy: FarePolicy = STANDARD,
    service: ServiceKind = ServiceKind.RIDE,
    ttl: timedelta = QUOTE_TTL,
) -> Quote:
    """Báo giá có hạn. `now` được truyền vào (không gọi đồng hồ ở đây) nên test cố định được."""
    fare = Money(calculate_fare(distance_km, policy, service=service))
    return Quote(distance_km, fare, now + ttl)
