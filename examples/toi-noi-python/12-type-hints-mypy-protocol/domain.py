"""Bài 12: hợp đồng kiểu cho mã nghiệp vụ Tới Nơi (mô hình học tập).

File này là thư viện nhỏ có kiểu; main.py chạy thử, broken.py chứa các lỗi cố ý để mypy bắt.
"""

import json
from collections.abc import Sequence
from dataclasses import dataclass, replace
from datetime import datetime
from decimal import ROUND_HALF_UP, Decimal
from typing import Literal, NamedTuple, NotRequired, Protocol, TypedDict

# --- Bí danh kiểu bằng câu lệnh `type` (PEP 695) -------------------------------------------
type TripId = str  # mã chuyến, ví dụ "TN-0001"
type ServiceName = Literal["ride", "food", "parcel"]  # bài 13 đổi thành ServiceKind(StrEnum)

BASE_FARE = Decimal("12000")
INCLUDED_KM = Decimal("2")
PRICE_PER_EXTRA_KM = Decimal("4500")
MAX_KM = Decimal("100")


def calculate_fare(distance_km: Decimal, *, service: ServiceName = "ride") -> Decimal:
    """Cước một chuyến (bài 03). `service` chưa đổi giá: mô hình học tập không có phụ phí."""
    if not Decimal("0") <= distance_km <= MAX_KM:
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    extra_km = max(Decimal("0"), distance_km - INCLUDED_KM)
    return (BASE_FARE + extra_km * PRICE_PER_EXTRA_KM).quantize(Decimal("1"), rounding=ROUND_HALF_UP)


def km_label(distance: Decimal) -> str:
    """Nhãn hiển thị khoảng cách. Chữ ký nói rõ: nhận Decimal, không nhận float."""
    return f"{distance} km"


# --- Dataclass có kiểu: `X | None` buộc người gọi xử lý «chưa có tài xế» ---------------------
# Bản rút gọn để tập trung vào kiểu: status là str thường, không mang máy trạng thái của bài 11.
@dataclass
class Trip:
    trip_id: TripId
    customer_id: str
    driver_id: str | None
    service: ServiceName
    distance_km: Decimal
    status: str
    created_at: datetime

    @property
    def fare(self) -> Decimal:
        return calculate_fare(self.distance_km, service=self.service)

    # `Driver` được khai báo SAU lớp này và `Trip` chính là lớp đang định nghĩa: từ Python 3.14
    # (PEP 649/749) vẫn không cần đặt dấu nháy quanh tên kiểu.
    def with_driver(self, driver: Driver) -> Trip:
        return replace(self, driver_id=driver.driver_id)


@dataclass
class Driver:
    driver_id: str
    full_name: str
    city: str


class QuoteLine(NamedTuple):
    """Dòng báo giá: kiểu khác Trip nhưng cũng có `fare` (dùng cho ví dụ Protocol)."""

    label: str
    fare: Decimal


# --- Generics bằng cú pháp PEP 695: tham số kiểu nằm ngay sau tên ---------------------------
def first[T](items: Sequence[T]) -> T:
    """Phần tử đầu; kiểu trả về đi theo kiểu phần tử của danh sách truyền vào."""
    if not items:
        raise ValueError("danh sách rỗng")
    return items[0]


class Registry[T]:
    """Kho trong bộ nhớ chứa đúng một loại đối tượng, tra theo mã."""

    def __init__(self, label: str) -> None:
        self._label = label
        self._items: dict[str, T] = {}

    def add(self, key: str, item: T) -> None:
        if key in self._items:
            raise ValueError(f"{self._label} đã có: {key}")
        self._items[key] = item

    def get(self, key: str) -> T | None:
        return self._items.get(key)

    def require(self, key: str) -> T:
        item = self._items.get(key)
        if item is None:
            raise KeyError(f"{self._label} không tồn tại: {key}")
        return item

    def values(self) -> list[T]:
        return list(self._items.values())


# --- Protocol: khớp theo cấu trúc, không cần kế thừa ----------------------------------------
class SupportsFare(Protocol):
    """Bất cứ thứ gì có thuộc tính `fare` đọc được kiểu Decimal."""

    @property
    def fare(self) -> Decimal: ...


def cheapest[T: SupportsFare](items: Sequence[T]) -> T:
    """`T: SupportsFare` là cận trên: T phải thỏa Protocol, và kết quả vẫn mang kiểu T."""
    return min(items, key=lambda item: item.fare)


def total_fare(items: Sequence[SupportsFare]) -> Decimal:
    """Sequence là covariant (hợp biến): list[Trip] truyền được vào Sequence[SupportsFare]."""
    return sum((item.fare for item in items), Decimal("0"))


# --- TypedDict: hình dạng của dict JSON (sự kiện BookingCreated) ----------------------------
class BookingCreatedPayload(TypedDict):
    trip_id: TripId
    customer_id: str
    service: ServiceName
    distance_km: str  # Decimal đi qua JSON dưới dạng chuỗi (bài 09)
    note: NotRequired[str]


class Envelope[P](TypedDict):
    event_id: str
    event_type: str
    occurred_at: str
    version: int
    payload: P


def parse_booking_created(raw: str) -> Envelope[BookingCreatedPayload]:
    """json.loads trả Any, nên mypy tin lời khai của ta: TypedDict KHÔNG kiểm dữ liệu lúc chạy."""
    event: Envelope[BookingCreatedPayload] = json.loads(raw)
    return event


def booking_distance(event: Envelope[BookingCreatedPayload]) -> Decimal:
    return Decimal(event["payload"]["distance_km"])
