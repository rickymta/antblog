"""Integration test cho TripRepository trên PostgreSQL thật (container hoặc compose)."""

from datetime import UTC
from decimal import Decimal

import psycopg
import pytest
from psycopg import errors as pg_errors

from trips_db import DriverBusy, InvalidTransition, TripRepository

D = Decimal


def make_trip(trips: TripRepository, trip_id: str = "TN-0001", distance: str = "2.500") -> None:
    trips.create(trip_id, "CU-001", "ride", D(distance), D("14250"))


def queue_trip(trips: TripRepository, trip_id: str, distance: str = "3") -> None:
    """Tạo chuyến rồi đưa sang `searching` (trạng thái duy nhất cho phép gán tài xế)."""
    trips.create(trip_id, "CU-001", "ride", D(distance), D("16500"))
    trips.start_searching(trip_id)


def test_roundtrip_keeps_decimal_and_aware_time(trips: TripRepository) -> None:
    make_trip(trips)

    row = trips.get("TN-0001")

    assert row is not None
    assert row.distance_km == D("2.500")          # numeric: không trôi như float
    assert row.fare_vnd == D("14250")
    assert row.status == "requested"
    assert row.created_at.tzinfo is not None      # timestamptz trả về datetime aware
    assert row.created_at.astimezone(UTC).tzinfo is UTC


def test_every_case_starts_in_an_empty_schema(conn: psycopg.Connection, pg_schema: str) -> None:
    # Mọi ca dùng cùng mã TN-0001 mà không đụng nhau vì mỗi ca có schema riêng.
    assert conn.execute("SELECT current_schema()").fetchone()[0] == pg_schema
    assert conn.execute("SELECT count(*) FROM trips").fetchone()[0] == 0


@pytest.mark.parametrize("distance", ["-0.001", "100.001"])
def test_distance_outside_domain_is_rejected_by_check(trips: TripRepository, distance: str) -> None:
    with pytest.raises(pg_errors.CheckViolation):
        make_trip(trips, distance=distance)


def test_trip_id_format_is_enforced(trips: TripRepository) -> None:
    with pytest.raises(pg_errors.CheckViolation):
        make_trip(trips, trip_id="TN-1")


def test_duplicate_trip_id_is_unique_violation(trips: TripRepository) -> None:
    make_trip(trips)
    with pytest.raises(pg_errors.UniqueViolation) as excinfo:
        make_trip(trips)
    assert excinfo.value.sqlstate == "23505"


def test_driver_cannot_hold_two_active_trips(trips: TripRepository) -> None:
    queue_trip(trips, "TN-0001")
    queue_trip(trips, "TN-0002")
    trips.assign_driver("TN-0001", "DR-001")

    with pytest.raises(DriverBusy):
        trips.assign_driver("TN-0002", "DR-001")

    after = trips.get("TN-0002")
    assert (after.driver_id, after.status) == (None, "searching")   # lệnh bị từ chối không để lại dấu vết


def test_second_driver_cannot_overwrite_an_accepted_trip(trips: TripRepository) -> None:
    # Lỗi mở bài: hai tài xế cùng nhận TN-0001; UPDATE không có điều kiện trạng thái sẽ ghi đè im lặng.
    queue_trip(trips, "TN-0001")
    trips.assign_driver("TN-0001", "DR-001")

    with pytest.raises(InvalidTransition):
        trips.assign_driver("TN-0001", "DR-002")

    assert trips.get("TN-0001").driver_id == "DR-001"


def test_cannot_assign_a_driver_before_the_trip_is_searching(trips: TripRepository) -> None:
    make_trip(trips, "TN-0001")                      # còn requested, chưa searching

    with pytest.raises(InvalidTransition):
        trips.assign_driver("TN-0001", "DR-001")

    assert trips.get("TN-0001").status == "requested"


def test_driver_is_free_again_after_trip_completes(conn: psycopg.Connection, trips: TripRepository) -> None:
    queue_trip(trips, "TN-0001")
    queue_trip(trips, "TN-0002")
    trips.assign_driver("TN-0001", "DR-001")
    conn.execute("UPDATE trips SET status = 'completed' WHERE trip_id = 'TN-0001'")

    trips.assign_driver("TN-0002", "DR-001")        # chỉ mục partial không còn tính TN-0001

    assert trips.get("TN-0002").status == "accepted"


def test_skip_locked_gives_two_drivers_two_different_trips(connect) -> None:
    a, b = connect(), connect()
    queue_trip(TripRepository(a), "TN-0001")
    queue_trip(TripRepository(a), "TN-0002", "4")

    with a.transaction():
        first = TripRepository(a).claim_next_searching("DR-001")
        # Giao dịch của a còn mở, TN-0001 đang bị khóa; b không chờ mà lấy chuyến kế tiếp.
        with b.transaction():
            second = TripRepository(b).claim_next_searching("DR-002")

    assert (first, second) == ("TN-0001", "TN-0002")


def test_plain_for_update_would_wait_for_the_lock(connect) -> None:
    a, b = connect(), connect()
    TripRepository(a).create("TN-0001", "CU-001", "ride", D("3"), D("16500"))

    with a.transaction():
        a.execute("SELECT trip_id FROM trips WHERE trip_id = 'TN-0001' FOR UPDATE")
        with b.transaction():
            b.execute("SET LOCAL lock_timeout = '300ms'")
            with pytest.raises(pg_errors.LockNotAvailable):
                b.execute("SELECT trip_id FROM trips WHERE trip_id = 'TN-0001' FOR UPDATE")
