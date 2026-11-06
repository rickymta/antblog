"""Bài 39: outbox, inbox và idempotent consumer trong Python.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify
Dọn:    uv run --project <lab> <lab>/main.py --cleanup   (DROP SCHEMA lab39 + xóa topic lab39.*)

Cần hạ tầng dùng chung (PostgreSQL và Kafka):
    docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Mọi thứ nằm trong schema `lab39` và các topic `lab39.*`, bị xóa khi chạy xong (trừ --keep).
Dữ liệu là dữ liệu giả; thời gian được truyền vào (`now`) nên kết quả lặp lại được.
"""

import argparse
import asyncio
import json
import sys
from datetime import UTC, datetime, timedelta
from decimal import Decimal

from sqlalchemy import Engine, create_engine, func, select, text
from sqlalchemy.exc import OperationalError
from sqlalchemy.orm import Session

import inbox
import kafka_io
import naive
import outbox
from kafka_io import SimulatedCrash
from models import Base, InboxEntry, OutboxEvent, PayoutLine, ScoreRequest, Trip
from settings import KAFKA_BOOTSTRAP, NAIVE_TOPIC, PG_URL, SCHEMA, TOPIC, sqlalchemy_url

D = Decimal
T0 = datetime(2026, 11, 6, 9, 0, tzinfo=UTC)
COMPOSE = "docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait"


def fail_no_service(name: str, url: str) -> None:
    print(f"FAIL: không nối được {name} tại {url} ({COMPOSE})")
    sys.exit(1)


def make_engine() -> Engine:
    engine = create_engine(sqlalchemy_url(), pool_pre_ping=True)
    try:
        with engine.connect() as conn:
            conn.execute(text("SELECT 1"))
    except OperationalError:
        fail_no_service("PostgreSQL", PG_URL)
    return engine


def drop_schema(engine: Engine) -> None:
    with engine.begin() as conn:
        conn.execute(text(f'DROP SCHEMA IF EXISTS "{SCHEMA}" CASCADE'))


def create_schema(engine: Engine) -> None:
    drop_schema(engine)
    with engine.begin() as conn:
        conn.execute(text(f'CREATE SCHEMA "{SCHEMA}"'))
    Base.metadata.create_all(engine)


def seed_trips(engine: Engine, trips: list[tuple[str, str, str, str]]) -> None:
    with Session(engine) as session, session.begin():
        for trip_id, customer_id, driver_id, km in trips:
            session.add(Trip(trip_id=trip_id, customer_id=customer_id, driver_id=driver_id, distance_km=D(km)))


def scalar(engine: Engine, stmt):
    with Session(engine) as session:
        return session.scalar(stmt)


def trip_status(engine: Engine, trip_id: str) -> str:
    return scalar(engine, select(Trip.status).where(Trip.trip_id == trip_id))


def count(engine: Engine, model, *where) -> int:
    return scalar(engine, select(func.count()).select_from(model).where(*where))


async def run(verify: bool, keep: bool) -> None:
    results: list[tuple[str, bool]] = []

    def check(name: str, got: object, expected: object) -> None:
        ok = got == expected
        results.append((name, ok))
        if not ok:
            print(f"SAI: {name}: nhận {got!r}, kỳ vọng {expected!r}")

    engine = make_engine()
    if not await kafka_io.check_kafka():
        fail_no_service("Kafka", KAFKA_BOOTSTRAP)
    producer = None
    try:
        create_schema(engine)
        await kafka_io.delete_topics(TOPIC, NAIVE_TOPIC)  # lần chạy trước có thể bị ngắt giữa chừng
        await asyncio.sleep(1)
        await kafka_io.create_topic(TOPIC)
        await kafka_io.create_topic(NAIVE_TOPIC)
        producer = await kafka_io.make_producer()
        seed_trips(
            engine,
            [
                ("TN-0101", "CU-001", "DR-001", "5"),
                ("TN-0102", "CU-002", "DR-002", "2.5"),
                ("TN-0001", "CU-001", "DR-001", "5"),
                ("TN-0002", "CU-002", "DR-002", "2.5"),
                ("TN-0003", "CU-003", "DR-003", "2"),
                ("TN-0004", "CU-001", "DR-001", "10"),
                ("TN-0005", "CU-004", "DR-002", "7.5"),
                ("TN-0006", "CU-005", "DR-004", "3"),
            ],
        )

        # ---- 1. Dual-write ngây thơ: hai cách, hai kiểu hỏng
        try:
            await naive.db_then_publish(engine, producer, "TN-0101", now=T0, crash_between=True)
        except SimulatedCrash:
            pass
        status, in_topic = trip_status(engine, "TN-0101"), await kafka_io.count_messages(NAIVE_TOPIC)
        print(f"Commit rồi publish, chết giữa chừng: chuyến TN-0101 = {status}, sự kiện trên topic = {in_topic}")
        check("commit trước, chết trước publish: chuyến xong nhưng không có sự kiện", (status, in_topic), ("completed", 0))

        try:
            await naive.publish_then_db(engine, producer, "TN-0102", now=T0)
        except SimulatedCrash:
            pass
        status, in_topic = trip_status(engine, "TN-0102"), await kafka_io.count_messages(NAIVE_TOPIC)
        print(f"Publish rồi commit, database lỗi: chuyến TN-0102 = {status}, sự kiện trên topic = {in_topic}")
        check("publish trước, database lỗi: có sự kiện cho chuyến chưa xong", (status, in_topic), ("in_transit", 1))

        # ---- 2. Outbox: chuyến và ý định phát cùng một transaction
        try:
            outbox.complete_trip(engine, "TN-0001", now=T0, fail_before_commit=True)
        except RuntimeError:
            pass
        status, rows = trip_status(engine, "TN-0001"), count(engine, OutboxEvent)
        print(f"Lỗi trước commit: chuyến TN-0001 = {status}, dòng outbox = {rows} (cả hai cùng rollback)")
        check("lỗi trước commit: không có chuyến xong, không có dòng outbox", (status, rows), ("in_transit", 0))

        event_ids = [outbox.complete_trip(engine, tid, now=T0) for tid in ("TN-0001", "TN-0002", "TN-0003", "TN-0004", "TN-0005")]
        pending = count(engine, OutboxEvent, OutboxEvent.published_at.is_(None))
        done = count(engine, Trip, Trip.status == "completed", Trip.trip_id < "TN-0100")
        print(f"Hoàn thành 5 chuyến: {done} chuyến completed, {pending} dòng outbox chờ phát")
        check("5 chuyến xong đi kèm 5 dòng outbox chờ phát", (done, pending), (5, 5))
        try:
            outbox.complete_trip(engine, "TN-0001", now=T0)
        except outbox.TripNotInTransit:
            pass
        check("hoàn thành lần hai bị từ chối, không sinh thêm dòng outbox", count(engine, OutboxEvent), 5)

        # ---- 3. Dispatcher: chết sau khi publish, trước khi đánh dấu
        try:
            await outbox.dispatch_once(engine, producer, now=T0, crash_after_publish=3)
        except SimulatedCrash:
            pass
        on_topic = await kafka_io.count_messages(TOPIC)
        marked = count(engine, OutboxEvent, OutboxEvent.published_at.is_not(None))
        print(f"Dispatcher chết sau 3 lần publish: topic có {on_topic} thông điệp, outbox đánh dấu đã phát = {marked}")
        check("chết giữa publish và đánh dấu: 3 trên topic, 0 đánh dấu", (on_topic, marked), (3, 0))

        report = await outbox.dispatch_once(engine, producer, now=T0 + timedelta(seconds=5))
        print(f"Chạy lại sau 5 giây (lease 30 giây chưa hết): thuê được {report.claimed} dòng")
        check("lease chưa hết hạn: dispatcher khác không thuê được dòng nào", report.claimed, 0)

        report = await outbox.dispatch_once(engine, producer, now=T0 + timedelta(seconds=60))
        envelopes = await kafka_io.read_all(TOPIC)
        distinct = {e["event_id"] for e in envelopes}
        marked = count(engine, OutboxEvent, OutboxEvent.published_at.is_not(None))
        print(f"Chạy lại sau 60 giây: thuê {report.claimed}, phát {report.published}; topic có {len(envelopes)} thông điệp, {len(distinct)} event_id khác nhau; đánh dấu đã phát = {marked}")
        check("lease hết hạn: phát lại cả 5 dòng", (report.claimed, report.published), (5, 5))
        check("topic có 8 thông điệp nhưng chỉ 5 sự kiện khác nhau (3 trùng)", (len(envelopes), len(distinct)), (8, 5))
        check("event_id trên topic đúng là event_id trong outbox", distinct, set(event_ids))
        check("outbox đánh dấu đủ 5 dòng đã phát", marked, 5)

        # ---- 4. Consumer KHÔNG có inbox: tiền chuyến bị trả hai lần
        now = T0 + timedelta(minutes=2)
        naive_outcomes = await kafka_io.consume(
            "lab39-naive-payout",
            TOPIC,
            lambda raw: asyncio.to_thread(inbox.handle_trip_completed, engine, "naive", raw, now=now, use_inbox=False),
            expected=8,
        )
        lines = count(engine, PayoutLine, PayoutLine.consumer == "naive")
        twice = scalar(
            engine,
            select(func.count()).select_from(
                select(PayoutLine.trip_id).where(PayoutLine.consumer == "naive").group_by(PayoutLine.trip_id).having(func.count() > 1).subquery()
            ),
        )
        print(f"Consumer không inbox: nhận {len(naive_outcomes)} thông điệp, ghi {lines} dòng tiền; {twice} chuyến bị trả hai lần")
        check("không inbox: 8 dòng tiền cho 5 chuyến, 3 chuyến bị trả hai lần", (lines, twice), (8, 3))

        # ---- 5. Consumer có inbox: cùng thông điệp, mỗi sự kiện một hiệu ứng
        outcomes = await kafka_io.consume(
            "lab39-payout",
            TOPIC,
            lambda raw: asyncio.to_thread(inbox.handle_trip_completed, engine, "payout", raw, now=now),
            expected=8,
        )
        lines = count(engine, PayoutLine, PayoutLine.consumer == "payout")
        entries = count(engine, InboxEntry, InboxEntry.consumer == "payout")
        print(f"Consumer có inbox: processed={outcomes.count(inbox.PROCESSED)}, duplicate={outcomes.count(inbox.DUPLICATE)}; dòng tiền = {lines}, dòng inbox = {entries}")
        check("có inbox: 5 processed, 3 duplicate", (outcomes.count(inbox.PROCESSED), outcomes.count(inbox.DUPLICATE)), (5, 3))
        check("có inbox: 5 dòng tiền, 5 dòng inbox", (lines, entries), (5, 5))
        total = scalar(engine, select(func.sum(PayoutLine.amount)).where(PayoutLine.consumer == "payout"))
        expected_total = sum(
            (scalar(engine, select(Trip.fare).where(Trip.trip_id == tid)) for tid in ("TN-0001", "TN-0002", "TN-0003", "TN-0004", "TN-0005")),
            D(0),
        )
        check("tổng tiền trả = tổng cước 5 chuyến", total, expected_total)

        other = await kafka_io.consume(
            "lab39-fraud-scoring",
            TOPIC,
            lambda raw: asyncio.to_thread(inbox.handle_trip_completed, engine, "fraud-scoring", raw, now=now, effect="score"),
            expected=8,
        )
        total_entries = count(engine, InboxEntry)
        scored = count(engine, ScoreRequest)
        print(f"Consumer thứ hai 'fraud-scoring': processed={other.count(inbox.PROCESSED)}, duplicate={other.count(inbox.DUPLICATE)}; tổng dòng inbox = {total_entries}, yêu cầu chấm điểm = {scored}")
        check("khóa inbox gồm cả consumer: consumer khác vẫn xử lý đủ 5 sự kiện", (other.count(inbox.PROCESSED), total_entries, scored), (5, 10, 5))

        # ---- 6. Chết giữa xử lý và commit offset: nhận lại, bỏ qua
        outbox.complete_trip(engine, "TN-0006", now=T0 + timedelta(minutes=3))
        await outbox.dispatch_once(engine, producer, now=T0 + timedelta(minutes=3))
        crashed = False
        try:
            await kafka_io.consume(
                "lab39-payout",
                TOPIC,
                lambda raw: asyncio.to_thread(inbox.handle_trip_completed, engine, "payout", raw, now=now),
                expected=1,
                crash_before_commit=True,
            )
        except SimulatedCrash:
            crashed = True
        lines_after_crash = count(engine, PayoutLine, PayoutLine.consumer == "payout")
        print(f"Consumer chết sau khi xử lý, trước khi commit offset: dòng tiền = {lines_after_crash}")
        redelivered = await kafka_io.consume(
            "lab39-payout",
            TOPIC,
            lambda raw: asyncio.to_thread(inbox.handle_trip_completed, engine, "payout", raw, now=now),
            expected=1,
        )
        lines_final = count(engine, PayoutLine, PayoutLine.consumer == "payout")
        print(f"Khởi động lại cùng group: nhận lại thông điệp, kết quả = {redelivered}; dòng tiền = {lines_final}")
        check("chết trước commit offset: hiệu ứng đã ghi 1 lần", (crashed, lines_after_crash), (True, 6))
        check("nhận lại sau khi restart: duplicate, không ghi thêm", (redelivered, lines_final), ([inbox.DUPLICATE], 6))

        # ---- 7. Lỗi trong transaction của consumer: inbox và hiệu ứng cùng rollback
        env_raw = (await kafka_io.read_all(TOPIC))[-1]
        raw = json.dumps({**env_raw, "event_id": "evt-fixture-1"}).encode()
        try:
            inbox.handle_trip_completed(engine, "fixture", raw, now=now, fail_after_effect=True)
        except RuntimeError:
            pass
        after_fail = (count(engine, InboxEntry, InboxEntry.consumer == "fixture"), count(engine, PayoutLine, PayoutLine.consumer == "fixture"))
        retry = inbox.handle_trip_completed(engine, "fixture", raw, now=now)
        print(f"Lỗi sau khi ghi hiệu ứng: (inbox, tiền) = {after_fail}; xử lý lại = {retry}")
        check("lỗi trong transaction: không còn dòng inbox nào, không có tiền", after_fail, (0, 0))
        check("xử lý lại sau lỗi: processed", retry, inbox.PROCESSED)

        # ---- 8. Hai dispatcher cùng lúc: SKIP LOCKED chia dòng, không thuê trùng
        for tid, cust in (("TN-0201", "CU-006"), ("TN-0202", "CU-006"), ("TN-0203", "CU-007"), ("TN-0204", "CU-007"), ("TN-0205", "CU-008")):
            seed_trips(engine, [(tid, cust, "DR-005", "4")])
            outbox.complete_trip(engine, tid, now=T0 + timedelta(minutes=4))
        later = T0 + timedelta(minutes=5)
        with Session(engine) as a, Session(engine) as b:  # mỗi session tự mở transaction ở lần dùng đầu
            first = outbox.claim_batch(a, now=later, lease=timedelta(seconds=30), limit=3)  # A chưa commit
            second = outbox.claim_batch(b, now=later, lease=timedelta(seconds=30), limit=3)  # B chạy khi A còn giữ khóa
            disjoint = {c.seq for c in first}.isdisjoint({c.seq for c in second})
            sizes = (len(first), len(second))  # thoát khối with = rollback cả hai, lease không giữ lại
        print(f"Hai dispatcher thuê đồng thời (limit 3): {sizes[0]} và {sizes[1]} dòng, không trùng nhau: {disjoint}")
        check("SKIP LOCKED: hai lô rời nhau, tổng 5 dòng chờ", (disjoint, sizes), (True, (3, 2)))
    except Exception as exc:  # noqa: BLE001  một bước hỏng không được che mất kết quả các ca đã chạy
        check(f"kịch bản chạy hết không ngoại lệ ({type(exc).__name__})", str(exc), "")
    finally:
        if producer is not None:
            await producer.stop()
        if keep:
            print(f"--keep: giữ schema {SCHEMA} và topic lab39.*")
        else:
            engine.dispose()
            cleanup = create_engine(sqlalchemy_url())
            drop_schema(cleanup)
            cleanup.dispose()
            await kafka_io.delete_topics(TOPIC, NAIVE_TOPIC)
            print(f"Đã dọn: DROP SCHEMA {SCHEMA} CASCADE, xóa topic {TOPIC} và {NAIVE_TOPIC}")
        engine.dispose()

    failed = [name for name, ok in results if not ok]
    if verify:
        if failed:
            print(f"FAIL {len(results) - len(failed)}/{len(results)}")
            sys.exit(1)
        print(f"OK {len(results)}/{len(results)}")


async def cleanup_only() -> None:
    engine = make_engine()
    drop_schema(engine)
    engine.dispose()
    await kafka_io.delete_topics(TOPIC, NAIVE_TOPIC)
    print(f"Đã dọn: DROP SCHEMA {SCHEMA} CASCADE, xóa topic {TOPIC} và {NAIVE_TOPIC}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    parser.add_argument("--verify", action="store_true", help="so kết quả kỳ vọng, in OK n/n")
    parser.add_argument("--keep", action="store_true", help="không xóa schema và topic khi xong")
    parser.add_argument("--cleanup", action="store_true", help="chỉ xóa schema và topic rồi thoát")
    args = parser.parse_args()
    if args.cleanup:
        asyncio.run(cleanup_only())
        return
    asyncio.run(run(verify=args.verify, keep=args.keep))


if __name__ == "__main__":
    main()
