"""Kho chuyến giả trong bộ nhớ. `Database` đóng vai pool kết nối; `TripSession` là một phiên ngắn hạn
cho mỗi request. Hai bộ đếm cho phép bài quan sát vòng đời dependency mà không cần database thật."""

import logging
from dataclasses import dataclass, field
from decimal import Decimal

from app.domain import ServiceKind, Trip, TripStatus

log = logging.getLogger("toinoi.repo")


def seed_trips() -> dict[str, Trip]:
    trips = [
        Trip("TN-0001", "CU-001", "DR-001", ServiceKind.RIDE, Decimal("2.5"), TripStatus.IN_TRANSIT),
        Trip("TN-0002", "CU-002", "DR-002", ServiceKind.FOOD, Decimal("5"), TripStatus.COMPLETED),
        Trip("TN-0003", "CU-001", "DR-003", ServiceKind.PARCEL, Decimal("12"), TripStatus.COMPLETED),
    ]
    return {t.trip_id: t for t in trips}


@dataclass
class Database:
    trips: dict[str, Trip]
    opened_total: int = 0  # tổng số phiên đã mở từ lúc service chạy
    open_now: int = 0  # số phiên đang mở (phải về 0 khi không có request)
    closed: bool = field(default=False)

    def open_session(self, *, fail_commit: bool = False) -> "TripSession":
        self.opened_total += 1
        self.open_now += 1
        return TripSession(self, session_no=self.opened_total, fail_commit=fail_commit)

    def close(self) -> None:
        self.closed = True


class TripSession:
    def __init__(self, db: Database, session_no: int, *, fail_commit: bool = False) -> None:
        self._db = db
        self.session_no = session_no
        self._open = True
        self._fail_commit = fail_commit  # lab: giả lập commit thất bại để so hai scope

    def get_trip(self, trip_id: str) -> Trip | None:
        if not self._open:
            raise RuntimeError("phiên đã đóng")
        return self._db.trips.get(trip_id)

    def commit(self) -> None:
        if self._fail_commit:
            raise RuntimeError("commit giả lập thất bại")

    def close(self) -> None:
        if self._open:
            self._open = False
            self._db.open_now -= 1
