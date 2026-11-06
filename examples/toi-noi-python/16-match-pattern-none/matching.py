"""match/case cho chuyến và sự kiện, cùng cách xử lý None đúng chỗ."""

from dataclasses import replace
from enum import Enum
from typing import Any, Final, TypeGuard, TypeIs, assert_never

from trip import Trip, TripStatus


class InvalidTransition(ValueError):
    """Lệnh không hợp lệ với trạng thái hiện tại."""


# --- 1. class pattern: mô tả chuyến, kể cả dữ liệu lỗi ----------------------------------
def describe(trip: Trip | None) -> str:
    match trip:
        case None:  # `is None`: chuyến không tồn tại
            return "Không tìm thấy chuyến"
        case Trip(status=TripStatus.CANCELLED):
            return "Chuyến đã hủy"
        case Trip(status=TripStatus.REQUESTED | TripStatus.SEARCHING, driver_id=None):
            return "Đang tìm tài xế"
        case Trip(status=TripStatus.REQUESTED | TripStatus.SEARCHING, driver_id=str()):
            return "Dữ liệu lỗi: chưa nhận chuyến mà đã có tài xế"
        case Trip(driver_id=None):  # tới đây trạng thái từ accepted trở đi
            return "Dữ liệu lỗi: chuyến đã nhận mà thiếu tài xế"
        case Trip(driver_id=str() as driver) if not driver.strip():
            return "Dữ liệu lỗi: driver_id trống"
        case Trip(status=TripStatus.IN_TRANSIT, distance_km=km, driver_id=driver) if km > 20:
            return f"Đang chạy đường dài {km} km, tài xế {driver}"
        case Trip(status=status, driver_id=driver):
            return f"Tài xế {driver}: {status.value}"
        case _:  # mypy biết trip đã hết khả năng: assert_never báo nếu thêm trường hợp mà quên xử lý
            assert_never(trip)


# --- 2. sequence + or-pattern + guard: chuyển trạng thái rõ ràng ------------------------
def apply_command(trip: Trip, command: tuple[object, ...]) -> Trip:
    """Trả Trip mới (không sửa tại chỗ). Lệnh không khớp trạng thái -> InvalidTransition."""
    match trip.status, command:
        case TripStatus.REQUESTED, ("search",):
            return replace(trip, status=TripStatus.SEARCHING)
        case TripStatus.SEARCHING, ("assign", str() as driver_id) if driver_id.strip():
            return replace(trip, status=TripStatus.ACCEPTED, driver_id=driver_id)
        case TripStatus.ACCEPTED, ("arrive",):
            return replace(trip, status=TripStatus.ARRIVING)
        case TripStatus.ARRIVING, ("pickup",):
            return replace(trip, status=TripStatus.PICKED_UP)
        case TripStatus.PICKED_UP, ("depart",):
            return replace(trip, status=TripStatus.IN_TRANSIT)
        case TripStatus.IN_TRANSIT, ("complete",):
            return replace(trip, status=TripStatus.COMPLETED)
        case (
            TripStatus.REQUESTED
            | TripStatus.SEARCHING
            | TripStatus.ACCEPTED
            | TripStatus.ARRIVING
        ), ("cancel",):
            return replace(trip, status=TripStatus.CANCELLED)
        case _:
            raise InvalidTransition(f"{trip.status.value} không nhận lệnh {command!r}")


def next_step_label(status: TripStatus) -> str:
    """Duyệt đủ 8 trạng thái; thiếu một case thì mypy --strict báo ở assert_never."""
    match status:
        case TripStatus.REQUESTED:
            return "chờ ghép tài xế"
        case TripStatus.SEARCHING:
            return "chờ tài xế nhận"
        case TripStatus.ACCEPTED:
            return "tài xế đang tới điểm đón"
        case TripStatus.ARRIVING:
            return "tài xế sắp tới điểm đón"
        case TripStatus.PICKED_UP:
            return "đã đón khách"
        case TripStatus.IN_TRANSIT:
            return "đang trên đường"
        case TripStatus.COMPLETED | TripStatus.CANCELLED:
            return "đã kết thúc"
        case _:
            assert_never(status)


# --- 3. mapping pattern: định tuyến envelope sự kiện ------------------------------------
def handle_event(envelope: dict[str, Any]) -> str:
    """Định tuyến theo hình dạng envelope. Đây là routing, không phải validation:
    kiểm tra đầy đủ kiểu và miền là việc của pydantic ở biên (bài 15)."""
    match envelope:
        case {
            "event_type": "BookingCreated",
            "version": 1,
            "payload": {"trip_id": str() as trip_id, "service": "ride" | "food" | "parcel" as service},
        }:
            return f"tạo chuyến {trip_id} ({service})"
        case {
            "event_type": "TripCompleted",
            "version": 1,
            "payload": {"trip_id": str() as trip_id, "driver_id": str() as driver_id},
        }:
            return f"chuyến {trip_id} hoàn tất, tài xế {driver_id}"
        case {
            "event_type": "TripCompleted",
            "version": 1,
            "payload": {"trip_id": str() as trip_id, "driver_id": None},
        }:  # khóa CÓ mặt và là null: khác với khóa vắng mặt
            return f"cách ly {trip_id}: TripCompleted có driver_id null"
        case {"event_type": str() as event_type, "version": int() as version} if version > 1:
            return f"bỏ qua {event_type} v{version}: phiên bản mới hơn consumer hiểu"
        case {"event_type": str() as event_type, **rest}:  # **rest gom các khóa còn lại
            return f"cách ly {event_type}: payload sai hợp đồng (khóa: {sorted(rest)})"
        case _:
            return "cách ly: không phải envelope"


# --- 4. None: ba trạng thái, sentinel và narrowing ----------------------------------------
def driver_label_truthy(trip: Trip) -> str:
    """Cách viết hay gặp nhưng sai: truthiness gộp None và chuỗi rỗng."""
    return trip.driver_id or "chưa có tài xế"


def driver_label(trip: Trip) -> str:
    """Tách rõ: None (chưa có), "" hoặc toàn khoảng trắng (dữ liệu lỗi), chuỗi hợp lệ."""
    match trip.driver_id:
        case None:
            return "chưa có tài xế"
        case str() as driver if not driver.strip():
            return "dữ liệu lỗi"
        case str() as driver:
            return driver


class _Unset(Enum):
    UNSET = "UNSET"


# Sentinel: khác với None. `driver_id=None` nghĩa là "bỏ tài xế", bỏ trống tham số nghĩa là "giữ nguyên".
UNSET: Final = _Unset.UNSET


def patch_trip(
    trip: Trip,
    *,
    driver_id: str | None | _Unset = UNSET,
    status: TripStatus | _Unset = UNSET,
) -> Trip:
    changes: dict[str, Any] = {}
    if driver_id is not UNSET:
        changes["driver_id"] = driver_id
    if status is not UNSET:
        changes["status"] = status
    return replace(trip, **changes)


def is_assigned(driver_id: str | None) -> TypeIs[str]:
    """TypeIs hợp lệ vì kết quả đúng khi và chỉ khi giá trị là str (thu hẹp cả nhánh else)."""
    return driver_id is not None


def has_usable_driver(driver_id: str | None) -> TypeGuard[str]:
    """Phải là TypeGuard: "" cũng là str nhưng trả False, nên nhánh else KHÔNG được thu hẹp thành None."""
    return driver_id is not None and driver_id.strip() != ""
