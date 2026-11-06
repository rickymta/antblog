"""Cố ý SAI để xem mypy --strict báo gì. KHÔNG nằm trong luồng chạy của main.py.

    uv run --project <lab> mypy <lab>/unsafe_example.py --strict
"""

from typing import assert_never, reveal_type

from matching import has_usable_driver, is_assigned
from trip import Trip, TripStatus


def driver_initial(trip: Trip) -> str:
    return trip.driver_id[0]  # lỗi: driver_id có thể là None


def label_missing_case(status: TripStatus) -> str:
    match status:
        case TripStatus.REQUESTED:
            return "chờ"
        case TripStatus.COMPLETED:
            return "xong"
        case _:
            assert_never(status)  # lỗi: còn 6 trạng thái chưa xử lý


def narrowing(driver_id: str | None) -> None:
    if is_assigned(driver_id):
        reveal_type(driver_id)  # TypeIs: nhánh if là str
    else:
        reveal_type(driver_id)  # TypeIs: nhánh else là None
    if has_usable_driver(driver_id):
        reveal_type(driver_id)  # TypeGuard: nhánh if là str
    else:
        reveal_type(driver_id)  # TypeGuard: nhánh else vẫn str | None
