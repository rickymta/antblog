"""Miền nghiệp vụ của Tới Nơi: kiểu dữ liệu, không có logic tính cước."""

from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal
from enum import StrEnum

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


# Chuyển hợp lệ: đi thuận từng bước; hủy được từ requested/searching/accepted/arriving.
ALLOWED_TRANSITIONS: dict[TripStatus, frozenset[TripStatus]] = {
    TripStatus.REQUESTED: frozenset({TripStatus.SEARCHING, TripStatus.CANCELLED}),
    TripStatus.SEARCHING: frozenset({TripStatus.ACCEPTED, TripStatus.CANCELLED}),
    TripStatus.ACCEPTED: frozenset({TripStatus.ARRIVING, TripStatus.CANCELLED}),
    TripStatus.ARRIVING: frozenset({TripStatus.PICKED_UP, TripStatus.CANCELLED}),
    TripStatus.PICKED_UP: frozenset({TripStatus.IN_TRANSIT}),
    TripStatus.IN_TRANSIT: frozenset({TripStatus.COMPLETED}),
    TripStatus.COMPLETED: frozenset(),
    TripStatus.CANCELLED: frozenset(),
}


def can_transition(current: TripStatus, new: TripStatus) -> bool:
    return new in ALLOWED_TRANSITIONS[current]


@dataclass(frozen=True)
class Money:
    """Số tiền + loại tiền. Bất biến; bằng nhau khi cùng amount và currency.

    Phần kiểm tra trong __post_init__ chép từ lab 13.
    """

    amount: Decimal
    currency: str = "VND"

    def __post_init__(self) -> None:
        if not isinstance(self.amount, Decimal):
            raise TypeError(f"amount phải là Decimal, nhận {type(self.amount).__name__}")
        if not self.amount.is_finite():
            raise ValueError(f"amount phải hữu hạn: {self.amount}")
        if self.amount < 0:
            raise ValueError(f"amount không được âm: {self.amount}")
        if len(self.currency) != 3 or not (self.currency.isascii() and self.currency.isupper()):
            raise ValueError(f"currency phải gồm 3 chữ in hoa: {self.currency!r}")
        if self.currency == "VND":
            if self.amount != self.amount.to_integral_value():
                raise ValueError(f"VND không có đồng lẻ: {self.amount}")
            # Chuẩn hóa 12000.0 -> 12000. Frozen cấm self.amount = ... kể cả trong
            # __post_init__, nên ta gán qua object.__setattr__.
            object.__setattr__(self, "amount", self.amount.quantize(Decimal("1")))

    def __add__(self, other: object) -> Money:
        if not isinstance(other, Money):
            return NotImplemented
        if other.currency != self.currency:
            raise ValueError(f"không cộng được {self.currency} với {other.currency}")
        return Money(self.amount + other.amount, self.currency)

    def __str__(self) -> str:
        if self.currency == "VND":
            return f"{self.amount:,.0f}".replace(",", ".") + " VND"
        return f"{self.amount} {self.currency}"


@dataclass(frozen=True)
class Quote:
    """Báo giá có hạn, chép từ lab 13. Thời điểm luôn aware (lưu UTC)."""

    trip_distance_km: Decimal
    fare: Money
    expires_at: datetime

    def __post_init__(self) -> None:
        if self.expires_at.tzinfo is None:
            raise ValueError("expires_at phải là datetime aware")
        if not Decimal("0") <= self.trip_distance_km <= MAX_KM:
            raise ValueError(f"khoảng cách ngoài miền 0..100 km: {self.trip_distance_km}")

    def is_valid(self, now: datetime) -> bool:
        """Còn hiệu lực khi now < expires_at."""
        if now.tzinfo is None:
            raise ValueError("now phải là datetime aware")
        return now < self.expires_at
