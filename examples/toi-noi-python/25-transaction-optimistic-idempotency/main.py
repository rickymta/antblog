"""Bài 25: transaction, optimistic concurrency và idempotency: đặt chuyến an toàn.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify
Dọn:    uv run --project <lab> <lab>/main.py --cleanup   (DROP SCHEMA lab25 CASCADE)

Cần hạ tầng dùng chung: docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Mọi thứ nằm trong schema `lab25`, bị xóa khi chạy xong (trừ --keep). Dữ liệu là dữ liệu giả.
"""

import argparse
import sys
import threading
from decimal import Decimal

from sqlalchemy import Engine, create_engine, func, select, text, update
from sqlalchemy.exc import DBAPIError, OperationalError
from sqlalchemy.orm import Session
from sqlalchemy.orm.exc import StaleDataError

from booking import (
    ConflictRetriesExhausted,
    IdempotencyConflict,
    TripNotOpen,
    accept_trip,
    book_trip,
    run_in_transaction,
)
from models import Base, BookingRequest, ServiceKind, Trip, TripStatus, VersionedTrip
from settings import PG_URL, SCHEMA, sqlalchemy_url

D = Decimal


def fail_no_db() -> None:
    print(
        f"FAIL: không nối được PostgreSQL tại {PG_URL} "
        "(docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait)"
    )
    sys.exit(1)


def make_engine() -> Engine:
    engine = create_engine(sqlalchemy_url(), pool_pre_ping=True)
    try:
        with engine.connect() as conn:
            conn.execute(text("SELECT 1"))
    except OperationalError:
        fail_no_db()
    return engine


def drop_schema(engine: Engine) -> None:
    with engine.begin() as conn:
        conn.execute(text(f'DROP SCHEMA IF EXISTS "{SCHEMA}" CASCADE'))


def create_schema(engine: Engine) -> None:
    """Bài 24 dùng Alembic; ở đây create_all cho gọn vì trọng tâm là transaction."""
    drop_schema(engine)
    with engine.begin() as conn:
        conn.execute(text(f'CREATE SCHEMA "{SCHEMA}"'))
    Base.metadata.create_all(engine)


def seed_trip(engine: Engine, trip_id: str, fare: str = "12000") -> None:
    with Session(engine) as session, session.begin():
        session.add(
            Trip(trip_id=trip_id, customer_id="CU-001", fare=D(fare), status=TripStatus.SEARCHING)
        )


def read_trip(engine: Engine, trip_id: str) -> Trip:
    with Session(engine) as session:
        trip = session.get(Trip, trip_id)
        session.expunge(trip)  # dùng được sau khi đóng session
        return trip


def run(verify: bool, keep: bool) -> None:
    results: list[tuple[str, bool]] = []

    def check(name: str, got: object, expected: object) -> None:
        ok = got == expected
        results.append((name, ok))
        if not ok:
            print(f"SAI: {name}: nhận {got!r}, kỳ vọng {expected!r}")

    engine = make_engine()
    try:
        create_schema(engine)

        # ---- 1. Transaction: tất cả hoặc không gì cả
        with engine.connect() as conn:
            level = conn.execute(text("SHOW transaction_isolation")).scalar_one()
        print(f"Isolation mặc định của PostgreSQL: {level}")
        check("isolation mặc định", level, "read committed")

        seen_inside = seen_other = None
        try:
            with Session(engine) as session, session.begin():
                session.add(Trip(trip_id="TN-0001", customer_id="CU-001", fare=D("12000")))
                session.flush()  # câu INSERT đã gửi, nhưng chưa COMMIT
                seen_inside = session.scalar(select(func.count()).select_from(Trip))
                with Session(engine) as other:  # một kết nối khác nhìn vào
                    seen_other = other.scalar(select(func.count()).select_from(Trip))
                raise RuntimeError("mô phỏng lỗi sau khi ghi chuyến")
        except RuntimeError:
            pass
        with Session(engine) as session:
            after = session.scalar(select(func.count()).select_from(Trip))
        print(f"Trong transaction thấy {seen_inside} chuyến; kết nối khác thấy {seen_other}; sau rollback còn {after}")
        check("trong transaction thấy chuyến vừa ghi", seen_inside, 1)
        check("kết nối khác không thấy dữ liệu chưa commit", seen_other, 0)
        check("rollback xóa sạch", after, 0)

        # ---- 2. Isolation level: hai người đọc cùng một dòng
        seed_trip(engine, "TN-0002", "12000")

        def change_fare_elsewhere(new_fare: str) -> None:
            with Session(engine) as other, other.begin():
                other.execute(update(Trip).where(Trip.trip_id == "TN-0002").values(fare=D(new_fare)))

        def read_fare(session: Session) -> Decimal:
            return session.scalar(select(Trip.fare).where(Trip.trip_id == "TN-0002"))

        with Session(engine) as session, session.begin():
            first = read_fare(session)
            change_fare_elsewhere("14250")
            second = read_fare(session)
        print(f"READ COMMITTED : đọc lần 1 = {first}, đọc lần 2 = {second} (non-repeatable read)")
        check("READ COMMITTED thấy giá trị mới ở lần đọc sau", (first, second), (D("12000"), D("14250")))

        with Session(engine) as session, session.begin():
            session.connection(execution_options={"isolation_level": "REPEATABLE READ"})
            first = read_fare(session)
            change_fare_elsewhere("15000")
            second = read_fare(session)
            sqlstate = message = None
            try:
                session.execute(update(Trip).where(Trip.trip_id == "TN-0002").values(fare=first + 500))
            except DBAPIError as exc:
                sqlstate = exc.orig.sqlstate
                message = exc.orig.diag.message_primary
        print(f"REPEATABLE READ: đọc lần 1 = {first}, đọc lần 2 = {second}, ghi chồng -> SQLSTATE {sqlstate}")
        print(f"Thông báo PostgreSQL cho 40001: {message}")
        check("REPEATABLE READ giữ nguyên snapshot", (first, second), (D("14250"), D("14250")))
        check("REPEATABLE READ: ghi chồng bị 40001", sqlstate, "40001")
        check(
            "thông báo của 40001",
            message,
            "could not serialize access due to concurrent update",
        )

        interference = {"left": 1}

        def raise_fare_by_500(session: Session) -> Decimal:
            current = read_fare(session)
            if interference["left"]:  # chỉ lần thử đầu có người chen vào
                interference["left"] -= 1
                change_fare_elsewhere(str(current + 1000))
            session.execute(update(Trip).where(Trip.trip_id == "TN-0002").values(fare=current + 500))
            return current + 500

        new_fare, attempts = run_in_transaction(engine, raise_fare_by_500, isolation_level="REPEATABLE READ")
        print(f"run_in_transaction thử lại toàn bộ khi 40001: dùng {attempts} lần, cước cuối = {new_fare}")
        check("retry transaction sau 40001", (attempts, new_fare), (2, D("16500")))

        # ---- 3. Lost update: không có version thì người ghi sau thắng, không ai biết
        seed_trip(engine, "TN-0003")
        outcomes = []
        with Session(engine) as a, Session(engine) as b:
            trip_a = a.get(Trip, "TN-0003")  # cả hai cùng thấy driver_id = None
            trip_b = b.get(Trip, "TN-0003")
            for who, session, trip in (("DR-001", a, trip_a), ("DR-002", b, trip_b)):
                if trip.driver_id is None:  # kiểm tra trên dữ liệu đã đọc: cả hai đều qua
                    trip.driver_id = who
                    session.commit()
                    outcomes.append(f"{who} commit thành công")
        final = read_trip(engine, "TN-0003")
        print(f"Không version: {', '.join(outcomes)}; driver cuối cùng = {final.driver_id}")
        check("lost update: cả hai đều tưởng đã nhận", len(outcomes), 2)
        check("lost update: DR-002 ghi đè DR-001", final.driver_id, "DR-002")

        # ---- 4. Optimistic concurrency bằng UPDATE ... WHERE version = :v
        seed_trip(engine, "TN-0004")

        def dr002_gets_it_first(attempt: int) -> None:
            accept_trip(engine, "TN-0004", "DR-002")

        try:
            accept_trip(engine, "TN-0004", "DR-001", before_write=dr002_gets_it_first)
            loser = "không lỗi"
        except TripNotOpen as exc:
            loser = f"TripNotOpen (driver hiện tại {exc.driver_id})"
        final = read_trip(engine, "TN-0004")
        print(f"Có version: DR-001 nhận {loser}; driver cuối = {final.driver_id}, version = {final.version}")
        check("optimistic: người chậm bị từ chối", loser, "TripNotOpen (driver hiện tại DR-002)")
        check("optimistic: một người thắng, version tăng", (final.driver_id, final.version), ("DR-002", 2))

        seed_trip(engine, "TN-0005")

        def someone_edits_fare(attempt: int) -> None:
            if attempt == 1:  # sửa một cột không liên quan, nhưng version vẫn đổi
                with Session(engine) as other, other.begin():
                    other.execute(
                        update(Trip).where(Trip.trip_id == "TN-0005").values(fare=D("13000"), version=Trip.version + 1)
                    )

        attempts = accept_trip(engine, "TN-0005", "DR-001", before_write=someone_edits_fare)
        final = read_trip(engine, "TN-0005")
        print(f"Xung đột không liên quan: nhận được ở lần thử {attempts}; driver = {final.driver_id}, version = {final.version}")
        check("xung đột vô hại: đọc lại rồi thành công", (attempts, final.driver_id, final.version), (2, "DR-001", 3))

        seed_trip(engine, "TN-0006")
        calls = []

        def always_edits(attempt: int) -> None:
            calls.append(attempt)
            with Session(engine) as other, other.begin():
                other.execute(update(Trip).where(Trip.trip_id == "TN-0006").values(version=Trip.version + 1))

        try:
            accept_trip(engine, "TN-0006", "DR-001", max_attempts=3, before_write=always_edits)
            exhausted = "không lỗi"
        except ConflictRetriesExhausted:
            exhausted = f"ConflictRetriesExhausted sau {len(calls)} lần"
        print(f"Version đổi liên tục: {exhausted}; driver = {read_trip(engine, 'TN-0006').driver_id}")
        check("retry có giới hạn", exhausted, "ConflictRetriesExhausted sau 3 lần")

        # ORM làm hộ phần "WHERE version = ?" nhờ version_id_col.
        with Session(engine) as session, session.begin():
            session.add(VersionedTrip(trip_id="TN-0007"))
        stale = "không lỗi"
        with Session(engine) as a, Session(engine) as b:
            va = a.get(VersionedTrip, "TN-0007")
            vb = b.get(VersionedTrip, "TN-0007")
            va.driver_id = "DR-001"
            a.commit()
            vb.driver_id = "DR-002"
            try:
                b.commit()
            except StaleDataError as exc:
                stale = type(exc).__name__
                b.rollback()
        with Session(engine) as session:
            vt = session.get(VersionedTrip, "TN-0007")
            print(f"version_id_col: người thứ hai nhận {stale}; driver = {vt.driver_id}, version = {vt.version}")
            check("ORM version_id_col ném StaleDataError", (stale, vt.driver_id, vt.version), ("StaleDataError", "DR-001", 2))

        # ---- 5. Idempotency: khóa + UNIQUE
        first = book_trip(engine, "CU-001", "req-1", D("5"))
        again = book_trip(engine, "CU-001", "req-1", D("5"))
        print(f"Gửi req-1 hai lần: lần 1 replayed={first.replayed}, lần 2 replayed={again.replayed}, cùng chuyến: {first.trip_id == again.trip_id}")
        check("lặp lại cùng khóa trả cùng chuyến", (first.replayed, again.replayed, first.trip_id == again.trip_id), (False, True, True))
        try:
            book_trip(engine, "CU-001", "req-1", D("6"))
            mismatch = "không lỗi"
        except IdempotencyConflict:
            mismatch = "IdempotencyConflict"
        print(f"Cùng khóa req-1 nhưng 6 km: {mismatch}")
        check("cùng khóa khác nội dung bị từ chối", mismatch, "IdempotencyConflict")

        try:
            book_trip(engine, "CU-001", "req-1", D("5"), service=ServiceKind.FOOD)
            service_mismatch = "không lỗi"
        except IdempotencyConflict:
            service_mismatch = "IdempotencyConflict"
        print(f"Cùng khóa req-1 nhưng đổi dịch vụ sang food: {service_mismatch}")
        check("khác dịch vụ cũng là khác nội dung", service_mismatch, "IdempotencyConflict")
        try:
            book_trip(engine, "CU-001", "req-far", D("100.5"))
            out_of_range = "không lỗi"
        except ValueError as exc:
            out_of_range = str(exc)
        print(f"Đặt 100,5 km: {out_of_range}")
        check("ngoài miền bị từ chối trước khi ghi", out_of_range, "khoảng cách ngoài miền 0..100 km: 100.5")

        # Hai request cùng khóa chạy đồng thời. Barrier giữ cả hai SAU bước kiểm tra "chưa có"
        # và TRƯỚC khi ghi, nên cả hai chắc chắn cùng thấy khóa chưa tồn tại.
        barrier = threading.Barrier(2)
        race: list = []
        errors: list[BaseException] = []

        def request() -> None:
            try:
                # timeout: nếu một thread chết trước hàng rào, thread kia báo lỗi thay vì treo mãi.
                race.append(
                    book_trip(
                        engine, "CU-001", "req-2", D("2.5"),
                        after_lookup=lambda: barrier.wait(timeout=5),
                    )
                )
            except BaseException as exc:  # noqa: BLE001 - gom lại để báo ở thread chính
                errors.append(exc)

        threads = [threading.Thread(target=request) for _ in range(2)]
        for t in threads:
            t.start()
        for t in threads:
            t.join(timeout=30)
        check("race: không thread nào gặp lỗi ngoài dự kiến", [repr(e) for e in errors], [])
        created = sum(not r.replayed for r in race)
        lost = sum(r.lost_race for r in race)
        same = len({r.trip_id for r in race}) == 1
        loser_conflict = next((r.conflict for r in race if r.lost_race), None)
        print(f"Thông báo PostgreSQL cho request thua: {loser_conflict}")
        print(f"Hai request req-2 đồng thời: {created} tạo chuyến, {lost} bị 23505 rồi đọc lại; cùng trip_id: {same}")
        check("race: một thắng, một thua nhận kết quả của người thắng", (created, lost, same), (1, 1, True))

        with Session(engine) as session:
            n_trips = session.scalar(
                select(func.count()).select_from(Trip).where(Trip.trip_id >= "TN-0100")
            )
            n_requests = session.scalar(select(func.count()).select_from(BookingRequest))
            used = session.execute(text(f"SELECT last_value FROM {SCHEMA}.trip_seq")).scalar_one()
        print(f"Trong database: {n_requests} bản ghi khóa, {n_trips} chuyến sinh bởi book_trip")
        check("không có chuyến mồ côi hay trùng", (n_requests, n_trips), (2, 2))
        print(
            f"Sequence đã cấp tới số {used} nhưng chỉ có {n_trips} chuyến: "
            "ca thua đã rollback chuyến ứng viên, số của nó không được dùng lại"
        )
        check("sequence không rollback: mã chuyến có khoảng trống", used, 102)
    finally:
        if keep:
            print(f"--keep: giữ schema {SCHEMA}")
        else:
            engine.dispose()
            cleanup_engine = create_engine(sqlalchemy_url())
            drop_schema(cleanup_engine)
            cleanup_engine.dispose()
            print(f"Đã dọn: DROP SCHEMA {SCHEMA} CASCADE")
        engine.dispose()

    failed = [name for name, ok in results if not ok]
    if verify:
        if failed:
            print(f"FAIL {len(results) - len(failed)}/{len(results)}")
            sys.exit(1)
        print(f"OK {len(results)}/{len(results)}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    parser.add_argument("--verify", action="store_true", help="so kết quả kỳ vọng, in OK n/n")
    parser.add_argument("--keep", action="store_true", help="không xóa schema lab25 khi xong")
    parser.add_argument("--cleanup", action="store_true", help="chỉ xóa schema lab25 rồi thoát")
    args = parser.parse_args()
    if args.cleanup:
        engine = make_engine()
        drop_schema(engine)
        engine.dispose()
        print(f"Đã dọn: DROP SCHEMA {SCHEMA} CASCADE")
        return
    run(verify=args.verify, keep=args.keep)


if __name__ == "__main__":
    main()
