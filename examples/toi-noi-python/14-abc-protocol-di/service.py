"""Dịch vụ báo giá: bản gắn cứng phụ thuộc (trước) và bản nhận phụ thuộc qua constructor (sau)."""

from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from decimal import Decimal

from fare import STANDARD, ServiceKind, calculate_fare
from ports import Clock, FarePolicyProvider

QUOTE_TTL = timedelta(minutes=5)


# Bản rút gọn của bài 13: bài đó mới thiết kế đầy đủ Money và Quote.
@dataclass(frozen=True)
class Money:
    amount: Decimal
    currency: str = "VND"


@dataclass(frozen=True)
class Quote:
    trip_distance_km: Decimal
    fare: Money
    expires_at: datetime  # aware

    def is_valid(self, now: datetime) -> bool:
        """Còn hiệu lực khi now < expires_at: đúng mốc 5 phút là hết (cùng biên với bài 13)."""
        if now.tzinfo is None:
            raise ValueError("now cần datetime có múi giờ (aware)")
        return now < self.expires_at


class HardcodedQuoteService:
    """TRƯỚC: tự gọi đồng hồ thật và dùng chính sách toàn cục; muốn test hạn phải chờ 5 phút."""

    def quote(self, distance_km: Decimal) -> Quote:
        fare = calculate_fare(distance_km, STANDARD)
        return Quote(distance_km, Money(fare), datetime.now(UTC) + QUOTE_TTL)


class QuoteService:
    """SAU: phụ thuộc đến từ ngoài qua constructor, nên thay được từng cái."""

    def __init__(self, policies: FarePolicyProvider, clock: Clock, ttl: timedelta = QUOTE_TTL) -> None:
        self._policies = policies
        self._clock = clock
        self._ttl = ttl

    def quote(self, city: str, distance_km: Decimal, *, service: ServiceKind = ServiceKind.RIDE) -> Quote:
        policy = self._policies.policy_for(city)
        fare = calculate_fare(distance_km, policy, service=service)
        return Quote(distance_km, Money(fare), self._clock.now() + self._ttl)

    def is_expired(self, quote: Quote) -> bool:
        return not quote.is_valid(self._clock.now())
