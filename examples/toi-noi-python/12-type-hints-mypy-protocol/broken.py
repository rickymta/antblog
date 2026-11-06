"""Các lỗi cố ý để mypy --strict bắt. Dòng nào kết thúc bằng chú thích «E:» (sau dấu thăng) PHẢI bị báo đúng một lỗi.

Đừng chạy file này: nó chỉ để mypy đọc (vài dòng sẽ lỗi lúc chạy). Chỉ riêng `km_label(2.5)`
sẽ chạy trót lọt, vì Python không kiểm kiểu.
"""

from datetime import UTC, datetime
from decimal import Decimal

from domain import Driver, Registry, Trip, cheapest, km_label, parse_booking_created

trip = Trip("TN-0001", "CU-001", None, "ride", Decimal("5"), "requested", datetime(2026, 1, 1, tzinfo=UTC))
driver = Driver("DR-001", "Nguyễn Văn A", "Hà Nội")
trips: Registry[Trip] = Registry("chuyến")

label = km_label(2.5)  # E: float không phải Decimal
name = trip.driver_id.upper()  # E: driver_id có thể là None
trips.add("DR-001", driver)  # E: kho Registry[Trip] không nhận Driver
bad_trip = Trip(trip_id="TN-0002")  # E: thiếu đối số bắt buộc
wrong_service = Trip("TN-0003", "CU-001", None, "taxi", Decimal("1"), "requested", trip.created_at)  # E: "taxi" ngoài Literal
winner = cheapest([driver])  # E: Driver không có thuộc tính fare


def show_all(items: list[object]) -> None:
    for item in items:
        print(item)


my_trips = [trip]
show_all(my_trips)  # E: list[Trip] không phải list[object] (list bất biến)
event = parse_booking_created("{}")
print(event["payload"]["distance"])  # E: payload không có khóa "distance"


def helper(x):  # E: --strict yêu cầu annotation đầy đủ
    return x
