from datetime import UTC, datetime, timedelta


class SystemClock:
    def now(self) -> datetime:
        return datetime.now(UTC)


class FixedClock:
    """Giờ cố định cho test và demo để output lặp lại được."""

    def __init__(self, at: datetime) -> None:
        self._at = at

    def now(self) -> datetime:
        return self._at


class ManualClock:
    """Đồng hồ chỉ nhích khi ta bảo: cho test hết hạn «thuê» dòng outbox mà không cần `sleep`."""

    def __init__(self, at: datetime) -> None:
        self._at = at

    def now(self) -> datetime:
        return self._at

    def advance(self, by: timedelta) -> None:
        self._at += by
