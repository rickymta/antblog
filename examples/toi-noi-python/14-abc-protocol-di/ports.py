"""Hai «cổng» mà QuoteService cần từ bên ngoài, khai báo bằng Protocol (khớp theo cấu trúc)."""

from datetime import UTC, datetime, timedelta
from typing import Protocol, runtime_checkable

from fare import FarePolicy


@runtime_checkable
class Clock(Protocol):
    """Ai có `now() -> datetime` (aware, UTC) đều là Clock; không cần kế thừa gì."""

    def now(self) -> datetime: ...


class FarePolicyProvider(Protocol):
    """Cho biết chính sách cước áp dụng ở một thành phố."""

    def policy_for(self, city: str) -> FarePolicy: ...


class SystemClock:
    """Đồng hồ thật, dùng ở môi trường chạy."""

    def now(self) -> datetime:
        return datetime.now(UTC)


class FixedClock:
    """Đồng hồ giả cho test: đứng yên cho tới khi ta tua."""

    def __init__(self, instant: datetime) -> None:
        if instant.tzinfo is None:
            raise ValueError("FixedClock cần datetime có múi giờ (aware)")
        self._instant = instant

    def now(self) -> datetime:
        return self._instant

    def advance(self, delta: timedelta) -> None:
        self._instant += delta
