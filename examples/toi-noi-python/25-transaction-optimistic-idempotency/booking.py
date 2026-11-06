"""Các use case ghi dữ liệu của lab 25: transaction, optimistic concurrency, idempotency.

Mỗi hàm tự mở Session riêng, giống một request web thật: không dùng chung session giữa
hai request. Các tham số `before_write`/`after_lookup` là "điểm chèn" cho test: chúng cho
phép dựng đúng thứ tự xen kẽ giữa hai người dùng để tái hiện race, không có sleep ngẫu nhiên.
"""

from collections.abc import Callable
from dataclasses import dataclass
from decimal import ROUND_HALF_UP, Decimal
from typing import NamedTuple

from sqlalchemy import select, update
from sqlalchemy.engine import Engine
from sqlalchemy.exc import DBAPIError, IntegrityError
from sqlalchemy.orm import Session

from models import TRIP_SEQ, BookingRequest, ServiceKind, Trip, TripStatus

UNIQUE_VIOLATION = "23505"
RETRYABLE_SQLSTATES = {"40001", "40P01"}  # serialization_failure, deadlock_detected


class FarePolicy(NamedTuple):
    base_fare: Decimal
    included_km: Decimal
    price_per_extra_km: Decimal


STANDARD = FarePolicy(Decimal("12000"), Decimal("2"), Decimal("4500"))


def calculate_fare(
    distance_km: Decimal, policy: FarePolicy = STANDARD, *, service: ServiceKind = ServiceKind.RIDE
) -> Decimal:
    """Cước học tập (bài 03, copy lại cho lab tự chứa): miền 0..100 km, ROUND_HALF_UP.

    Ba dịch vụ cùng giá nên `service` chỉ để giữ chữ ký thống nhất với các bài khác.
    """
    if not Decimal("0") <= distance_km <= Decimal("100"):
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    extra = max(Decimal("0"), distance_km - policy.included_km) * policy.price_per_extra_km
    return (policy.base_fare + extra).quantize(Decimal("1"), rounding=ROUND_HALF_UP)


# ---------------------------------------------------------------- lỗi nghiệp vụ


class TripNotOpen(Exception):
    """Chuyến đã có người nhận (hoặc không còn ở trạng thái searching)."""

    def __init__(self, trip_id: str, driver_id: str | None, status: str) -> None:
        super().__init__(f"{trip_id} không còn nhận được: status={status}, driver={driver_id}")
        self.driver_id = driver_id


class ConflictRetriesExhausted(Exception):
    """Version đổi liên tục, hết số lần thử cho phép."""


class IdempotencyConflict(Exception):
    """Cùng khóa idempotency nhưng nội dung ý định khác."""


# ---------------------------------------------------------------- transaction + retry


def run_in_transaction[T](
    engine: Engine,
    work: Callable[[Session], T],
    *,
    isolation_level: str | None = None,
    max_attempts: int = 3,
) -> tuple[T, int]:
    """Chạy `work` trong một transaction; thử lại TOÀN BỘ transaction khi PostgreSQL báo
    serialization_failure (40001) hoặc deadlock (40P01). Lỗi khác ném ngay.

    Trả về (kết quả, số lần thử đã dùng). Số lần thử bị chặn: retry vô hạn là một dạng sự cố.
    """
    for attempt in range(1, max_attempts + 1):
        try:
            with Session(engine) as session, session.begin():
                if isolation_level:
                    # Phải đặt trước câu lệnh đầu tiên của transaction.
                    session.connection(execution_options={"isolation_level": isolation_level})
                return work(session), attempt
        except DBAPIError as exc:
            sqlstate = getattr(exc.orig, "sqlstate", None)
            if sqlstate not in RETRYABLE_SQLSTATES or attempt == max_attempts:
                raise
    raise AssertionError("không tới được")  # vòng lặp luôn return hoặc raise


# ---------------------------------------------------------------- optimistic concurrency


def accept_trip(
    engine: Engine,
    trip_id: str,
    driver_id: str,
    *,
    max_attempts: int = 3,
    before_write: Callable[[int], None] | None = None,
) -> int:
    """Tài xế nhận chuyến bằng UPDATE có điều kiện version. Trả về số lần thử đã dùng.

    Mỗi lần thử: đọc (driver, status, version) -> quyết định nghiệp vụ -> UPDATE ... WHERE version = <đã đọc>.
    rowcount == 0 nghĩa là có người sửa hàng này giữa lúc đọc và lúc ghi: ta ĐỌC LẠI và quyết định
    lại từ đầu, không ghi đè mù. Nếu lần đọc lại thấy chuyến đã có tài xế, đó là kết quả nghiệp vụ
    (TripNotOpen), không phải lý do để thử tiếp.
    """
    for attempt in range(1, max_attempts + 1):
        with Session(engine) as session, session.begin():
            row = session.execute(
                select(Trip.driver_id, Trip.status, Trip.version).where(Trip.trip_id == trip_id)
            ).one()
            if row.driver_id is not None or row.status != TripStatus.SEARCHING:
                raise TripNotOpen(trip_id, row.driver_id, row.status)
            if before_write:
                before_write(attempt)  # chỗ test chèn một người ghi khác
            result = session.execute(
                update(Trip)
                .where(Trip.trip_id == trip_id, Trip.version == row.version)
                .values(driver_id=driver_id, status=TripStatus.ACCEPTED, version=Trip.version + 1)
            )
            if result.rowcount == 1:
                return attempt
        # rowcount == 0: version đã đổi, vòng sau đọc lại.
    raise ConflictRetriesExhausted(f"{trip_id}: version đổi liên tục sau {max_attempts} lần thử")


# ---------------------------------------------------------------- idempotency


@dataclass(frozen=True)
class BookingResult:
    trip_id: str
    replayed: bool  # True nếu đây là lần lặp của một ý định đã có kết quả
    lost_race: bool = False  # True nếu ta bị 23505 và phải đọc lại kết quả của người thắng
    conflict: str | None = None  # "SQLSTATE: thông báo của PostgreSQL" khi lost_race


def _resolve(
    request: BookingRequest, distance_km: Decimal, service: ServiceKind, *, conflict: str | None
) -> BookingResult:
    # So khớp nội dung: cùng khóa mà ý định khác là lỗi của client, không phải "lặp lại".
    if (request.distance_km, request.service) != (distance_km, service):
        raise IdempotencyConflict(
            f"khóa {request.idempotency_key!r} đã dùng cho {request.distance_km} km {request.service}, "
            f"không phải {distance_km} km {service}"
        )
    return BookingResult(request.trip_id, replayed=True, lost_race=conflict is not None, conflict=conflict)


def _find_request(session: Session, customer_id: str, key: str) -> BookingRequest | None:
    return session.scalar(
        select(BookingRequest).where(
            BookingRequest.customer_id == customer_id, BookingRequest.idempotency_key == key
        )
    )


def book_trip(
    engine: Engine,
    customer_id: str,
    key: str,
    distance_km: Decimal,
    *,
    service: ServiceKind = ServiceKind.RIDE,
    after_lookup: Callable[[], None] | None = None,
) -> BookingResult:
    """Đặt chuyến idempotent: cùng (customer_id, key) luôn ra cùng một chuyến.

    Chuyến và bản ghi khóa được ghi trong CÙNG một transaction: không có trạng thái "có khóa
    mà mất chuyến" hay ngược lại.
    """
    fare = calculate_fare(distance_km, service=service)  # ngoài miền thì ValueError ngay, chưa tốn sequence
    with Session(engine) as session:
        existing = _find_request(session, customer_id, key)
        if existing is not None:
            return _resolve(existing, distance_km, service, conflict=None)

    if after_lookup:
        after_lookup()  # chỗ test giữ cả hai request lại, để cả hai cùng thấy "chưa có khóa"

    try:
        with Session(engine) as session, session.begin():
            trip_id = f"TN-{session.scalar(TRIP_SEQ.next_value()):04d}"
            session.add(
                Trip(
                    trip_id=trip_id,
                    customer_id=customer_id,
                    service=service,
                    status=TripStatus.REQUESTED,
                    fare=fare,
                )
            )
            session.flush()  # chuyến phải có trước khi khóa trỏ khóa ngoại tới nó
            session.add(
                BookingRequest(
                    customer_id=customer_id,
                    idempotency_key=key,
                    service=service,
                    distance_km=distance_km,
                    trip_id=trip_id,
                )
            )
        return BookingResult(trip_id, replayed=False)
    except IntegrityError as exc:
        # Chỉ chấp nhận ĐÚNG lỗi ta chờ: vi phạm UNIQUE của khóa idempotency. Mọi IntegrityError
        # khác (khóa ngoại, NOT NULL...) là bug thật và phải nổi lên.
        orig = exc.orig
        if getattr(orig, "sqlstate", None) != UNIQUE_VIOLATION or (
            orig.diag.constraint_name != "uq_booking_requests_customer_key"
        ):
            raise
        conflict = f"{orig.sqlstate}: {orig.diag.message_primary}"
    # Transaction của ta đã rollback (kể cả chuyến ứng viên). Người thắng đã commit: đọc lại.
    with Session(engine) as session:
        winner = _find_request(session, customer_id, key)
        if winner is None:  # 23505 mà không thấy khóa: không phải tình huống ta hiểu
            raise RuntimeError(f"23505 nhưng không thấy khóa {key!r}")
        return _resolve(winner, distance_km, service, conflict=conflict)
