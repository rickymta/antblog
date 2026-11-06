"""Bài 24: SQLAlchemy 2 và psycopg 3 với PostgreSQL: mapping, migration Alembic.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify
Dọn:    uv run --project <lab> <lab>/main.py --cleanup   (DROP SCHEMA lab24 CASCADE)

Cần hạ tầng dùng chung: docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Dữ liệu là dữ liệu giả; mọi thứ nằm trong schema `lab24` và bị xóa khi chạy xong (trừ --keep).
"""

import argparse
import asyncio
import logging
import sys
from contextlib import contextmanager
from datetime import UTC, datetime
from decimal import ROUND_HALF_UP, Decimal
from pathlib import Path
from zoneinfo import ZoneInfo

from alembic import command
from alembic.config import Config
from alembic.runtime.migration import MigrationContext
from alembic.script import ScriptDirectory
from sqlalchemy import Engine, create_engine, inspect, select, text
from sqlalchemy.dialects.postgresql import psycopg as pg_psycopg
from sqlalchemy.exc import IntegrityError, OperationalError
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.orm import Session, selectinload

from models import Customer, ServiceKind, Trip, TripStatus
from settings import PG_URL, SCHEMA, sqlalchemy_url

HERE = Path(__file__).resolve().parent
VN = ZoneInfo("Asia/Ho_Chi_Minh")

# Cước học tập của Tới Nơi (bài 03); copy lại để lab tự chứa.
BASE_FARE = Decimal("12000")
INCLUDED_KM = Decimal("2")
PRICE_PER_EXTRA_KM = Decimal("4500")


def calculate_fare(distance_km: Decimal, *, service: str = "ride") -> Decimal:
    if not Decimal("0") <= distance_km <= Decimal("100"):
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    extra = max(Decimal("0"), distance_km - INCLUDED_KM) * PRICE_PER_EXTRA_KM
    return (BASE_FARE + extra).quantize(Decimal("1"), rounding=ROUND_HALF_UP)


# ---------------------------------------------------------------- tiện ích hạ tầng


class SqlRecorder(logging.Handler):
    """Ghi lại các câu SQL mà engine phát ra (cùng nguồn với `echo=True`).

    `create_engine(url, echo=True)` chỉ là cách tắt để bật logger `sqlalchemy.engine`
    ở mức INFO; ở đây ta bật logger đó nhưng gom vào danh sách thay vì in ra màn hình.
    """

    def __init__(self) -> None:
        super().__init__(logging.INFO)
        self.messages: list[str] = []

    def emit(self, record: logging.LogRecord) -> None:
        self.messages.append(record.getMessage())

    @property
    def statements(self) -> list[str]:
        """Chỉ các câu DML/SELECT (bỏ BEGIN, COMMIT, dòng tham số, dòng cache)."""
        verbs = ("SELECT", "INSERT", "UPDATE", "DELETE")
        return [m for m in self.messages if m.lstrip().startswith(verbs)]

    @property
    def params(self) -> list[str]:
        return [m for m in self.messages if m.startswith(("[generated", "[cached", "[raw sql"))]


@contextmanager
def record_sql():
    handler = SqlRecorder()
    logger = logging.getLogger("sqlalchemy.engine")
    old_level = logger.level
    logger.setLevel(logging.INFO)
    logger.addHandler(handler)
    try:
        yield handler
    finally:
        logger.removeHandler(handler)
        logger.setLevel(old_level)


def alembic_config() -> Config:
    cfg = Config(str(HERE / "alembic.ini"))
    cfg.attributes["configure_logger"] = False  # giữ output của lab gọn; CLI vẫn có log
    return cfg


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


def current_revision(engine: Engine) -> str | None:
    with engine.connect() as conn:
        return MigrationContext.configure(conn, opts={"version_table_schema": SCHEMA}).get_current_revision()


def seed(session: Session) -> None:
    """Thêm 3 khách và 4 chuyến giả. Giờ ghi UTC aware; created_at TN-0003/4 để server_default."""
    customers = [
        Customer(customer_id="CU-001", full_name="Nguyễn Văn A"),
        Customer(customer_id="CU-002", full_name="Trần Thị B"),
        Customer(customer_id="CU-003", full_name="Lê Văn C"),  # chưa có chuyến nào
    ]
    specs = [
        ("TN-0001", "CU-001", ServiceKind.RIDE, "2.5"),
        ("TN-0002", "CU-001", ServiceKind.FOOD, "5"),
        ("TN-0003", "CU-002", ServiceKind.PARCEL, "0"),
        ("TN-0004", "CU-002", ServiceKind.RIDE, "2.001"),
    ]
    trips = []
    for trip_id, customer_id, service, km in specs:
        distance = Decimal(km)
        trips.append(
            Trip(
                trip_id=trip_id,
                customer_id=customer_id,
                service=service,
                distance_km=distance,
                fare=calculate_fare(distance, service=service),
            )
        )
    trips[0].created_at = datetime(2026, 11, 6, 3, 0, tzinfo=UTC)  # 10:00 giờ Việt Nam
    session.add_all(customers)
    session.add_all(trips)


# ---------------------------------------------------------------- các bước demo


async def async_demo() -> tuple[int, str, str]:
    """Cùng mô hình, engine bất đồng bộ: `postgresql+psycopg://` dùng được cho cả hai kiểu."""
    engine = create_async_engine(sqlalchemy_url())
    try:
        async with AsyncSession(engine) as session:
            trips = (await session.scalars(select(Trip).order_by(Trip.trip_id))).all()
            customer = await session.scalar(select(Customer).where(Customer.customer_id == "CU-001"))
            try:
                customer.trips  # tải lười cần I/O ngầm, mà coroutine không được phép chặn
                lazy_error = "không lỗi"
            except Exception as exc:  # noqa: BLE001 - ta muốn tên loại lỗi gốc
                # SQLAlchemy bọc lỗi trong StatementError; `.orig` giữ MissingGreenlet.
                lazy_error = type(getattr(exc, "orig", exc)).__name__
            loaded = await session.scalar(
                select(Customer)
                .options(selectinload(Customer.trips))
                .where(Customer.customer_id == "CU-001")
            )
            return len(trips), lazy_error, ",".join(t.trip_id for t in loaded.trips)
    finally:
        await engine.dispose()


def run(verify: bool, keep: bool) -> None:
    results: list[tuple[str, bool]] = []

    def check(name: str, got: object, expected: object) -> None:
        ok = got == expected
        results.append((name, ok))
        if not ok:
            print(f"SAI: {name}: nhận {got!r}, kỳ vọng {expected!r}")

    engine = make_engine()
    cfg = alembic_config()
    try:
        # 1. Migration: bắt đầu từ schema trống, `alembic upgrade head` dựng bảng.
        drop_schema(engine)
        command.upgrade(cfg, "head")
        head = ScriptDirectory.from_config(cfg).get_current_head()
        revision = current_revision(engine)
        print(f"alembic upgrade head -> revision hiện tại {revision}, head của script {head}")
        check("revision hiện tại là head", (revision, head), ("0002", "0002"))

        # 2. Database thật khớp với mapping? Hỏi catalog, không hỏi lại model.
        insp = inspect(engine)
        columns = {
            c["name"]: c["type"].compile(dialect=engine.dialect)
            for c in insp.get_columns("trips", schema=SCHEMA)
        }
        print("Cột trips trong PostgreSQL:", columns)
        check("kiểu cột fare", columns["fare"], "NUMERIC(12, 0)")
        check("kiểu cột created_at", columns["created_at"], "TIMESTAMP WITH TIME ZONE")
        index_names = {i["name"] for i in insp.get_indexes("trips", schema=SCHEMA)}
        check("index của migration 0002", "ix_trips_customer_created" in index_names, True)
        command.check(cfg)  # ném CommandError nếu models.py và database lệch nhau
        check("alembic check: không còn thay đổi chờ migration", True, True)

        # 3. Session + Unit of Work: add_all rồi commit một lần.
        with Session(engine) as session, session.begin():
            seed(session)
        print("Đã thêm 3 khách và 4 chuyến bằng một transaction.")

        # 4. select() 2.0 và SQL sinh ra.
        stmt = (
            select(Trip)
            .where(Trip.status == TripStatus.REQUESTED, Trip.customer_id == "CU-001")
            .order_by(Trip.created_at.desc())
        )
        sql_text = str(stmt.compile(dialect=pg_psycopg.dialect()))
        print("SQL sinh ra từ select(Trip):")
        print(sql_text)
        check("SQL có điều kiện status", "lab24.trips.status = %(status_1)s" in sql_text, True)
        check("SQL có ORDER BY DESC", "ORDER BY lab24.trips.created_at DESC" in sql_text, True)

        with Session(engine) as session:
            with record_sql() as rec:
                rows = session.scalars(stmt).all()
            # Dòng log thứ hai của mỗi câu là tham số; chúng đi riêng, không nối vào chuỗi SQL.
            print("Tham số gửi kèm:", rec.params[0].split("] ", 1)[1] if rec.params else "?")
            check("hai chuyến requested của CU-001", [t.trip_id for t in rows], ["TN-0002", "TN-0001"])

            first = session.get(Trip, "TN-0001")
            print(
                f"TN-0001: fare={first.fare!r}, distance={first.distance_km!r}, "
                f"status={first.status!r}, created_at={first.created_at.astimezone(VN):%Y-%m-%d %H:%M %Z}"
            )
            check("fare là Decimal đúng giá trị", first.fare, Decimal("14250"))
            check("distance_km giữ 3 chữ số lẻ", str(first.distance_km), "2.500")
            check("status được map về StrEnum", first.status, TripStatus.REQUESTED)
            check("created_at aware, cùng thời điểm", first.created_at, datetime(2026, 11, 6, 3, 0, tzinfo=UTC))
            check("created_at có offset", first.created_at.utcoffset() is not None, True)

            # Identity map: cùng khóa chính trong một session là cùng một object, không thêm SQL.
            with record_sql() as rec:
                again = session.get(Trip, "TN-0001")
            check("identity map: cùng object", again is first, True)
            check("identity map: không thêm SQL", len(rec.statements), 0)

            # Unit of Work: đổi thuộc tính, flush phát UPDATE.
            first.status = TripStatus.SEARCHING
            with record_sql() as rec:
                session.flush()
            print("flush sau khi đổi status:", rec.statements[0])
            check("flush phát đúng một UPDATE", len(rec.statements), 1)
            session.rollback()  # không giữ thay đổi demo

        # 5. N+1: tải lười vs selectinload.
        with Session(engine) as session:
            with record_sql() as rec:
                for customer in session.scalars(select(Customer).order_by(Customer.customer_id)):
                    _ = len(customer.trips)
            lazy_count = len(rec.statements)
        with Session(engine) as session:
            with record_sql() as rec:
                for customer in session.scalars(
                    select(Customer).options(selectinload(Customer.trips)).order_by(Customer.customer_id)
                ):
                    _ = len(customer.trips)
            eager_count = len(rec.statements)
        print(f"3 khách, đọc .trips của từng người: lazy = {lazy_count} câu SQL, selectinload = {eager_count} câu")
        check("lazy load: 1 + 3 câu", lazy_count, 4)
        check("selectinload: 2 câu", eager_count, 2)

        # selectinload chia lô: mỗi câu SELECT ... IN (...) mang tối đa 500 khóa. Thêm 1.498 khách
        # (tổng 1.501) để thấy số câu SQL tăng theo lô chứ không theo số khách.
        with Session(engine) as session, session.begin():
            session.add_all(
                Customer(customer_id=f"CU-{n:04d}", full_name=f"Khách giả {n}") for n in range(1000, 2498)
            )
        with Session(engine) as session:
            with record_sql() as rec:
                customers = session.scalars(
                    select(Customer).options(selectinload(Customer.trips))
                ).all()
            batch_count = len(rec.statements)
        print(f"{len(customers)} khách với selectinload: {batch_count} câu SQL (1 câu lấy khách + các lô tối đa 500 khóa)")
        check("selectinload chia lô 500 khóa: 1.501 khách ra 5 câu", (len(customers), batch_count), (1501, 5))

        # 5b. native_enum=False chỉ đổi cách lưu thành VARCHAR, không tạo CHECK: database vẫn
        # nhận một status lạ nếu ta ghi thẳng bằng SQL (ORM thì chặn ở phía Python).
        with Session(engine) as session:
            session.execute(
                text(
                    f"INSERT INTO {SCHEMA}.trips (trip_id, customer_id, service, distance_km, fare, status) "
                    "VALUES ('TN-0098', 'CU-001', 'ride', 1, 12000, 'bogus')"
                )
            )
            bogus = session.execute(
                text(f"SELECT status FROM {SCHEMA}.trips WHERE trip_id = 'TN-0098'")
            ).scalar_one()
            session.rollback()
        print(f"INSERT thẳng status='bogus' vào VARCHAR: PostgreSQL nhận ({bogus!r}), vì không có CHECK")
        check("không có CHECK trên status", bogus, "bogus")

        # 6. Ràng buộc nằm ở database: khóa ngoại sai bị PostgreSQL từ chối (SQLSTATE 23503).
        with Session(engine) as session:
            session.add(
                Trip(
                    trip_id="TN-0099",
                    customer_id="CU-999",
                    service=ServiceKind.RIDE,
                    distance_km=Decimal("1"),
                    fare=Decimal("12000"),
                )
            )
            try:
                session.commit()
                sqlstate = "không lỗi"
            except IntegrityError as exc:
                sqlstate = exc.orig.sqlstate  # exc.orig là exception gốc của psycopg
                session.rollback()
        print(f"Chuyến của khách không tồn tại -> IntegrityError, SQLSTATE {sqlstate}")
        check("khóa ngoại sai: SQLSTATE 23503", sqlstate, "23503")

        # 7. Async engine.
        n_trips, lazy_error, eager_ids = asyncio.run(async_demo())
        print(f"Async: {n_trips} chuyến; tải lười trong async -> {lazy_error}; selectinload -> {eager_ids}")
        check("async đọc đủ 4 chuyến", n_trips, 4)
        check("async: tải lười ném MissingGreenlet", lazy_error, "MissingGreenlet")
        check("async: selectinload chạy được", eager_ids, "TN-0001,TN-0002")

        # 8. Alembic downgrade/upgrade: migration đi hai chiều.
        command.downgrade(cfg, "-1")
        idx_after_down = {i["name"] for i in inspect(engine).get_indexes("trips", schema=SCHEMA)}
        rev_after_down = current_revision(engine)
        command.upgrade(cfg, "head")
        idx_after_up = {i["name"] for i in inspect(engine).get_indexes("trips", schema=SCHEMA)}
        print(f"downgrade -1 -> revision {rev_after_down}, index còn: {sorted(idx_after_down)}")
        print(f"upgrade head -> revision {current_revision(engine)}, index còn: {sorted(idx_after_up)}")
        check("downgrade -1 bỏ index", (rev_after_down, "ix_trips_customer_created" in idx_after_down), ("0001", False))
        check("upgrade head dựng lại index", "ix_trips_customer_created" in idx_after_up, True)
        with Session(engine) as session:
            check("dữ liệu còn nguyên sau downgrade/upgrade", len(session.scalars(select(Trip)).all()), 4)
    finally:
        if keep:
            print(f"--keep: giữ schema {SCHEMA}")
        else:
            drop_schema(engine)
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
    parser.add_argument("--keep", action="store_true", help="không xóa schema lab24 khi xong")
    parser.add_argument("--cleanup", action="store_true", help="chỉ xóa schema lab24 rồi thoát")
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
