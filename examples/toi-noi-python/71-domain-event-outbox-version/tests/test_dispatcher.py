"""Dispatcher trên PostgreSQL và Kafka thật: phát, đánh dấu, crash sinh trùng, thuê dòng."""

import asyncio
import json
from datetime import timedelta

import pytest
from consumers.scored_consumer import (
    SUPPORTED_BY_NEW_CONSUMER,
    SUPPORTED_BY_OLD_CONSUMER,
    EnvelopeError,
    decode,
)
from sqlalchemy.engine import Engine

from kyc_service.api.container import build_container
from kyc_service.infrastructure.clocks import ManualClock
from kyc_service.infrastructure.ids import SequentialIds
from kyc_service.infrastructure.kafka.admin import make_producer, read_all
from kyc_service.infrastructure.kafka.dispatcher import (
    OutboxDispatcher,
    SimulatedCrash,
    claim_batch,
)
from kyc_service.infrastructure.sql.unit_of_work import make_session_factory

from .conftest import NOW, outbox_rows, submit_for

pytestmark = pytest.mark.usefixtures("clean")


def seed(
    engine: Engine,
    topic: str,
    drivers: list[str],
    *,
    version: int = 2,
    ids: SequentialIds | None = None,
) -> None:
    c = build_container(
        engine,
        clock=ManualClock(NOW),
        ids=ids or SequentialIds(),
        emit_version=version,
        topic=topic,
        log=lambda _: None,
    )
    for driver in drivers:
        c.mediator.send(submit_for(driver))


async def dispatch(engine: Engine, clock: ManualClock, **kwargs: int | None) -> tuple[int, int]:
    producer = await make_producer()
    try:
        d = OutboxDispatcher(make_session_factory(engine), producer, clock)
        report = await d.dispatch_once(**kwargs)
        return report.claimed, report.published
    finally:
        await producer.stop()


def test_dispatch_publishes_every_pending_row_once_and_marks_them(
    engine: Engine, topic: str
) -> None:
    seed(engine, topic, ["DR-001", "DR-002", "DR-004"])
    clock = ManualClock(NOW)
    assert asyncio.run(dispatch(engine, clock)) == (3, 3)
    assert all(r["published_at"] is not None for r in outbox_rows(engine))
    assert asyncio.run(dispatch(engine, clock)) == (0, 0)  # đã phát hết: vòng sau không có gì
    messages = asyncio.run(read_all(topic))
    ids = [json.loads(m.value)["event_id"] for m in messages]
    assert sorted(ids) == [f"00000000-0000-4000-8000-00000000000{n}" for n in (1, 2, 3)]


def test_messages_on_the_wire_are_the_stored_envelopes(engine: Engine, topic: str) -> None:
    seed(engine, topic, ["DR-002"])
    asyncio.run(dispatch(engine, ManualClock(NOW)))
    (message,) = asyncio.run(read_all(topic))
    assert json.loads(message.value) == outbox_rows(engine)[0]["envelope"]
    assert message.key == b"KYC-0001"  # khóa Kafka là mã hồ sơ


def test_crash_between_publish_and_mark_sends_the_same_event_id_twice(
    engine: Engine, topic: str
) -> None:
    seed(engine, topic, ["DR-001", "DR-002"])
    clock = ManualClock(NOW)
    with pytest.raises(SimulatedCrash):
        asyncio.run(dispatch(engine, clock, crash_after_publish=1))
    assert [r["published_at"] for r in outbox_rows(engine)] == [None, None]  # chưa kịp đánh dấu
    assert asyncio.run(dispatch(engine, clock)) == (
        0,
        0,
    )  # lease còn hiệu lực: chưa ai được phát lại
    clock.advance(timedelta(seconds=31))
    assert asyncio.run(dispatch(engine, clock)) == (2, 2)
    ids = [json.loads(m.value)["event_id"] for m in asyncio.run(read_all(topic))]
    assert len(ids) == 3  # ba thông điệp trên topic...
    assert len(set(ids)) == 2  # ...nhưng chỉ hai sự kiện: một event_id đến hai lần


def test_two_dispatchers_claim_disjoint_rows(engine: Engine, topic: str) -> None:
    seed(engine, topic, ["DR-001", "DR-002", "DR-004", "DR-005"])
    factory = make_session_factory(engine)
    lease = timedelta(seconds=30)
    with factory() as a, a.begin(), factory() as b, b.begin():
        first = claim_batch(a, now=NOW, lease=lease, limit=2)  # A giữ khóa dòng, chưa commit
        second = claim_batch(b, now=NOW, lease=lease, limit=2)  # B bỏ qua dòng đang khóa
    assert len(first) == len(second) == 2
    assert {c.seq for c in first}.isdisjoint({c.seq for c in second})


def test_claim_respects_the_lease_and_attempts_count(engine: Engine, topic: str) -> None:
    seed(engine, topic, ["DR-001"])
    factory = make_session_factory(engine)
    lease = timedelta(seconds=30)
    with factory() as s, s.begin():
        assert len(claim_batch(s, now=NOW, lease=lease, limit=10)) == 1
    with factory() as s, s.begin():
        assert claim_batch(s, now=NOW + timedelta(seconds=10), lease=lease, limit=10) == []
    with factory() as s, s.begin():
        assert len(claim_batch(s, now=NOW + timedelta(seconds=31), lease=lease, limit=10)) == 1
    assert outbox_rows(engine)[0]["attempts"] == 2


def test_rollout_order_old_consumer_rejects_v2_new_consumer_reads_both(
    engine: Engine, topic: str
) -> None:
    ids = SequentialIds()  # dùng chung: event_id không được trùng giữa hai producer
    seed(engine, topic, ["DR-001", "DR-002"], version=1, ids=ids)  # producer chưa nâng cấp
    seed(engine, topic, ["DR-004", "DR-005"], version=2, ids=ids)  # producer đã nâng cấp
    asyncio.run(dispatch(engine, ManualClock(NOW)))
    raws = [m.value for m in asyncio.run(read_all(topic))]
    assert len(raws) == 4

    def outcome(raw: bytes, supported: frozenset[int]) -> str:
        try:
            return f"ok v{decode(raw, supported=supported).version}"
        except EnvelopeError as exc:
            return exc.reason

    old = sorted(outcome(r, SUPPORTED_BY_OLD_CONSUMER) for r in raws)
    new = sorted(outcome(r, SUPPORTED_BY_NEW_CONSUMER) for r in raws)
    assert old == ["ok v1", "ok v1", "unsupported_version", "unsupported_version"]
    assert new == ["ok v1", "ok v1", "ok v2", "ok v2"]
