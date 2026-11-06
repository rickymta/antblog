"""State: luật «được làm gì» của chuyến, viết theo hai cách cho cùng một vòng đời 8 trạng thái."""

from dataclasses import dataclass, field
from typing import ClassVar

from domain import TripStatus

# ---------- Cách 1: bảng chuyển (dict) ----------
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


@dataclass
class TableTrip:
    trip_id: str
    driver_id: str | None = None
    status: TripStatus = TripStatus.REQUESTED

    def move_to(self, new_status: TripStatus) -> None:
        if new_status not in ALLOWED_TRANSITIONS[self.status]:
            raise ValueError(f"{self.trip_id}: không chuyển {self.status} -> {new_status}")
        # Luật riêng của một trạng thái bắt đầu chen vào bảng dưới dạng if:
        if new_status is TripStatus.ACCEPTED and self.driver_id is None:
            raise ValueError(f"{self.trip_id}: chưa có tài xế nhận chuyến")
        self.status = new_status


# ---------- Cách 2: mỗi trạng thái một lớp ----------
class TripState:
    """Lớp cha: mặc định cho cả 8 trạng thái. Lớp con chỉ ghi đè cái khác biệt.

    Trong Python 3.14, annotation `Trip` bên dưới không cần ngoặc kép dù lớp
    Trip khai báo ở dưới (PEP 649: annotation được tính khi cần).
    """

    status: ClassVar[TripStatus]
    forward: ClassVar[TripStatus | None] = None  # bước thuận kế tiếp
    cancellable: ClassVar[bool] = False

    def message(self, trip: Trip) -> str:
        raise NotImplementedError

    def advance(self, trip: Trip) -> TripStatus:
        if self.forward is None:
            raise ValueError(f"{trip.trip_id}: {self.status} là trạng thái cuối")
        return self.forward

    def cancel(self, trip: Trip) -> TripStatus:
        if not self.cancellable:
            raise ValueError(f"{trip.trip_id}: không hủy được khi {self.status}")
        return TripStatus.CANCELLED


class Requested(TripState):
    status = TripStatus.REQUESTED
    forward = TripStatus.SEARCHING
    cancellable = True

    def message(self, trip: Trip) -> str:
        return "Đã nhận yêu cầu đặt chuyến"


class Searching(TripState):
    status = TripStatus.SEARCHING
    forward = TripStatus.ACCEPTED
    cancellable = True

    def message(self, trip: Trip) -> str:
        return "Đang tìm tài xế"

    def advance(self, trip: Trip) -> TripStatus:
        # Luật riêng của trạng thái này nằm trong lớp của nó, không rải khắp nơi.
        if trip.driver_id is None:
            raise ValueError(f"{trip.trip_id}: chưa có tài xế nhận chuyến")
        return super().advance(trip)


class Accepted(TripState):
    status = TripStatus.ACCEPTED
    forward = TripStatus.ARRIVING
    cancellable = True

    def message(self, trip: Trip) -> str:
        return f"Tài xế {trip.driver_id} đã nhận chuyến"


class Arriving(TripState):
    status = TripStatus.ARRIVING
    forward = TripStatus.PICKED_UP
    cancellable = True

    def message(self, trip: Trip) -> str:
        return f"Tài xế {trip.driver_id} đang đến điểm đón"


class PickedUp(TripState):
    status = TripStatus.PICKED_UP
    forward = TripStatus.IN_TRANSIT

    def message(self, trip: Trip) -> str:
        return "Đã đón khách"


class InTransit(TripState):
    status = TripStatus.IN_TRANSIT
    forward = TripStatus.COMPLETED

    def message(self, trip: Trip) -> str:
        return "Đang trên đường tới điểm đến"


class Completed(TripState):
    status = TripStatus.COMPLETED

    def message(self, trip: Trip) -> str:
        return "Chuyến đã hoàn tất"


class Cancelled(TripState):
    status = TripStatus.CANCELLED

    def message(self, trip: Trip) -> str:
        return "Chuyến đã hủy"


# Các lớp trạng thái không giữ dữ liệu riêng nên mỗi loại chỉ cần một instance dùng chung.
STATES: dict[TripStatus, TripState] = {
    cls.status: cls()
    for cls in (Requested, Searching, Accepted, Arriving, PickedUp, InTransit, Completed, Cancelled)
}


@dataclass
class Trip:
    trip_id: str
    driver_id: str | None = None
    _state: TripState = field(
        default_factory=lambda: STATES[TripStatus.REQUESTED], init=False, repr=False
    )

    @property
    def status(self) -> TripStatus:
        return self._state.status

    @property
    def message(self) -> str:
        return self._state.message(self)

    def advance(self) -> None:
        self._state = STATES[self._state.advance(self)]

    def cancel(self) -> None:
        self._state = STATES[self._state.cancel(self)]
