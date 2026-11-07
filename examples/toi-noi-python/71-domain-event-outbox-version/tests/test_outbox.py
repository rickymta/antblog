"""Outbox trong Unit of Work trên PostgreSQL thật: cùng transaction, đúng hợp đồng."""

import json
from typing import Any

import pytest
from sqlalchemy import event, text
from sqlalchemy.engine import Engine
from sqlalchemy.exc import OperationalError

from kyc_service.api.container import Container, build_container
from kyc_service.application.commands import ResumeScreening
from kyc_service.application.errors import UnitOfWorkNotStarted
from kyc_service.application.integration_events import UnmappedDomainEvent
from kyc_service.domain.events import DomainEvent
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.fake_services import FakeOcr
from kyc_service.infrastructure.ids import SequentialIds
from kyc_service.infrastructure.sql.outbox import OutboxEventPublisher

from .conftest import NOW, outbox_rows, submit_for

pytestmark = pytest.mark.usefixtures("clean")


def test_one_scored_application_writes_exactly_one_outbox_row(
    container: Container, engine: Engine
) -> None:
    container.mediator.send(submit_for("DR-002"))
    (row,) = outbox_rows(engine)
    assert row["event_id"] == "00000000-0000-4000-8000-000000000001"
    assert row["message_key"] == "KYC-0001"
    assert row["published_at"] is None
    env: dict[str, Any] = row["envelope"]
    assert env["event_id"] == row["event_id"]  # id trong envelope là id đã lưu, không sinh lại
    assert (env["event_type"], env["version"]) == ("KycApplicationScored", 2)
    assert env["occurred_at"] == "2026-11-06T03:00:00+00:00"
    assert env["payload"] == {
        "application_id": "KYC-0001",
        "driver_id": "DR-002",
        "risk_score": 0.35,
        "review_reasons": ["ocr_low_confidence", "face_low_score", "risk_high"],
        "face_score": 0.71,
        "ocr_min_confidence": 0.58,
    }


def test_producer_pinned_to_version_1_writes_the_v1_payload(engine: Engine) -> None:
    c = build_container(engine, clock=FixedClock(NOW), ids=SequentialIds(), emit_version=1)
    c.mediator.send(submit_for("DR-002"))
    (row,) = outbox_rows(engine)
    assert row["envelope"]["version"] == 1
    assert set(row["envelope"]["payload"]) == {
        "application_id",
        "driver_id",
        "risk_score",
        "review_reasons",
    }


def test_application_without_a_score_writes_no_outbox_row(
    container: Container, engine: Engine
) -> None:
    container.mediator.send(submit_for("DR-003", consent=False))
    assert outbox_rows(engine) == []


def test_resume_writes_the_row_only_when_scoring_completes(engine: Engine) -> None:
    ocr = FakeOcr(available=False)
    c = build_container(engine, emit_version=2, ocr=ocr, clock=FixedClock(NOW), ids=SequentialIds())
    c.mediator.send(submit_for("DR-001"))
    assert outbox_rows(engine) == []
    ocr.available = True
    c.mediator.send(ResumeScreening("KYC-0001"))
    assert len(outbox_rows(engine)) == 1


def test_rows_get_increasing_seq_and_distinct_event_ids(
    container: Container, engine: Engine
) -> None:
    for driver in ("DR-001", "DR-002", "DR-004"):
        container.mediator.send(submit_for(driver))
    rows = outbox_rows(engine)
    assert [r["seq"] for r in rows] == sorted(r["seq"] for r in rows)
    assert len({r["event_id"] for r in rows}) == 3
    assert [r["message_key"] for r in rows] == ["KYC-0001", "KYC-0002", "KYC-0003"]


def test_commit_failure_loses_the_application_and_the_outbox_row_together(
    container: Container, engine: Engine
) -> None:
    def fail_on_commit(conn: object) -> None:
        raise OperationalError("COMMIT", {}, Exception("mất kết nối lúc commit"))

    event.listen(engine, "commit", fail_on_commit)
    try:
        with pytest.raises(OperationalError):
            container.mediator.send(submit_for("DR-001"))
    finally:
        event.remove(engine, "commit", fail_on_commit)
    with engine.connect() as conn:
        apps = conn.execute(text("SELECT count(*) FROM lab71.kyc_applications")).scalar_one()
    assert (apps, outbox_rows(engine)) == (0, [])


def test_unexpected_error_after_the_row_is_written_rolls_the_row_back(engine: Engine) -> None:
    """Dòng outbox đã nằm trong Session; lỗi ở bước sau (ở đây: hook commit) cuốn nó đi cùng."""
    c = build_container(engine, emit_version=2, clock=FixedClock(NOW), ids=SequentialIds())

    def boom(conn: object) -> None:
        raise RuntimeError("lỗi lập trình lúc commit")

    event.listen(engine, "commit", boom)
    try:
        with pytest.raises(RuntimeError):
            c.mediator.send(submit_for("DR-001"))
    finally:
        event.remove(engine, "commit", boom)
    assert outbox_rows(engine) == []


def test_publisher_refuses_to_run_outside_a_transaction(container: Container) -> None:
    publisher = OutboxEventPublisher(
        container.uow, topic="x", emit_version=2, ids=SequentialIds(), clock=FixedClock(NOW)
    )

    class Dummy(DomainEvent):
        pass

    with pytest.raises(UnitOfWorkNotStarted):
        publisher.publish([Dummy(occurred_at=NOW)])


def test_publisher_refuses_domain_events_it_cannot_map(
    container: Container, engine: Engine
) -> None:
    publisher = OutboxEventPublisher(
        container.uow, topic="x", emit_version=2, ids=SequentialIds(), clock=FixedClock(NOW)
    )

    class Dummy(DomainEvent):
        pass

    with pytest.raises(UnmappedDomainEvent), container.uow.begin():
        publisher.publish([Dummy(occurred_at=NOW)])
    assert outbox_rows(engine) == []


def test_envelope_stored_as_jsonb_survives_a_json_round_trip(
    container: Container, engine: Engine
) -> None:
    container.mediator.send(submit_for("DR-002"))
    (row,) = outbox_rows(engine)
    text_form = json.dumps(row["envelope"], ensure_ascii=False, separators=(",", ":"))
    assert json.loads(text_form) == row["envelope"]
