"""Pipeline của bài 68 chạy trên PostgreSQL thật: transaction là transaction của cơ sở dữ liệu."""

import warnings
from typing import Any

import pytest
from pydantic import TypeAdapter
from sqlalchemy import text
from sqlalchemy.engine import Engine

from kyc_service.api.container import Container, FakeAdapterWarning, build_container
from kyc_service.application.behaviors import validation_behavior
from kyc_service.application.commands import RecordReviewDecision, SubmitDriverApplication
from kyc_service.application.errors import ApplicationNotFound, ValidationFailed
from kyc_service.application.handlers import GetApplicationHandler
from kyc_service.application.mediator import Mediator
from kyc_service.application.queries import GetApplication, ListReviewQueue
from kyc_service.domain.errors import InvalidStatusTransition
from kyc_service.domain.kyc import KycApplication, RiskScore
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.fake_services import FakeFaceMatch, FakeOcr, FakeRiskScore
from kyc_service.infrastructure.sql.read_model import SqlAlchemyReadModel
from kyc_service.infrastructure.sql.unit_of_work import make_session_factory

from .conftest import NOW, submit_for


def stored(engine: Engine) -> list[tuple[str, str]]:
    with engine.connect() as conn:
        found = conn.execute(
            text("SELECT application_id, status FROM lab70.kyc_applications ORDER BY 1")
        )
        return [(r[0], r[1]) for r in found]


def test_submit_commits_one_real_transaction(
    container: Container, engine: Engine, tx_events: list[str]
) -> None:
    view = container.mediator.send(submit_for("DR-001"))
    assert (view.application_id, view.status) == ("KYC-0001", "needs_review")
    assert tx_events == ["begin", "commit"]
    assert stored(engine) == [("KYC-0001", "needs_review")]


def test_invalid_command_never_touches_the_database(
    container: Container, engine: Engine, tx_events: list[str]
) -> None:
    with pytest.raises(ValidationFailed):
        container.mediator.send(SubmitDriverApplication("DR-1", "", "x", "y", True))
    assert tx_events == []
    assert stored(engine) == []


def test_unexpected_error_rolls_back_and_the_sequence_keeps_the_gap(
    engine: Engine, clean: None, tx_events: list[str]
) -> None:
    class Exploding:
        def score(self, application: KycApplication) -> RiskScore:
            raise RuntimeError("bug")

    broken = build_container(engine, risk=Exploding(), clock=FixedClock(NOW), log=lambda _: None)
    with pytest.raises(RuntimeError):
        broken.mediator.send(submit_for("DR-001"))
    assert stored(engine) == []
    assert tx_events[-1] == "rollback"
    healthy = build_container(engine, clock=FixedClock(NOW), log=lambda _: None)
    assert healthy.mediator.send(submit_for("DR-001")).application_id == "KYC-0002"


def test_expected_port_failure_still_commits_the_application(
    engine: Engine, clean: None, tx_events: list[str]
) -> None:
    c = build_container(engine, ocr=FakeOcr(available=False), clock=FixedClock(NOW))
    view = c.mediator.send(submit_for("DR-001"))
    assert (view.status, view.pending_check) == ("submitted", "kyc-ocr")
    assert tx_events == ["begin", "commit"]
    assert stored(engine) == [("KYC-0001", "submitted")]


def test_face_down_keeps_the_ocr_results_in_the_database(engine: Engine, clean: None) -> None:
    c = build_container(engine, face=FakeFaceMatch(available=False), clock=FixedClock(NOW))
    c.mediator.send(submit_for("DR-001"))
    assert c.mediator.send(GetApplication("KYC-0001")).status == "ocr_done"
    with engine.connect() as conn:
        n = conn.execute(text("SELECT count(*) FROM lab70.kyc_ocr_results")).scalar_one()
    assert n == 2


def test_reviewer_decides_once_and_the_second_decision_rolls_back(
    container: Container, tx_events: list[str]
) -> None:
    container.mediator.send(submit_for("DR-001"))
    done = container.mediator.send(RecordReviewDecision("KYC-0001", "RV-001", "approved", "khớp"))
    assert (done.status, done.reviewed_by) == ("approved", "RV-001")
    tx_events.clear()
    with pytest.raises(InvalidStatusTransition):
        container.mediator.send(RecordReviewDecision("KYC-0001", "RV-001", "rejected", "đổi ý"))
    assert tx_events == ["begin", "rollback"]
    assert container.mediator.send(GetApplication("KYC-0001")).status == "approved"


def test_unknown_application_is_not_found(container: Container) -> None:
    with pytest.raises(ApplicationNotFound):
        container.mediator.send(GetApplication("KYC-9999"))


def test_review_queue_only_lists_needs_review_in_id_order(container: Container) -> None:
    for driver in ("DR-001", "DR-002", "DR-003"):
        container.mediator.send(submit_for(driver))
    container.mediator.send(RecordReviewDecision("KYC-0002", "RV-001", "rejected", "OCR mờ"))
    queue = container.mediator.send(ListReviewQueue())
    assert [v.application_id for v in queue] == ["KYC-0001", "KYC-0003"]


def test_query_opens_no_write_transaction(container: Container, tx_events: list[str]) -> None:
    container.mediator.send(submit_for("DR-001"))
    tx_events.clear()
    container.mediator.send(GetApplication("KYC-0001"))
    assert "commit" not in tx_events  # đọc bằng Session riêng, không qua Unit of Work của command


def test_ocr_rows_keep_the_join_key_to_the_ocr_document_store(
    container: Container, engine: Engine
) -> None:
    container.mediator.send(submit_for("DR-001"))
    with engine.connect() as conn:
        rows = conn.execute(
            text(
                "SELECT kind, model_version, length(image_sha256)"
                " FROM lab70.kyc_ocr_results ORDER BY kind"
            )
        ).all()
    assert [tuple(r) for r in rows] == [("cccd", "fake-ocr-1", 64), ("gplx", "fake-ocr-1", 64)]


def test_default_adapters_are_announced_and_explicit_ones_are_not(engine: Engine) -> None:
    with pytest.warns(FakeAdapterWarning, match="adapter giả cho: ocr, face, risk"):
        build_container(engine, clock=FixedClock(NOW), log=lambda _: None)
    with warnings.catch_warnings():
        warnings.simplefilter("error", FakeAdapterWarning)
        build_container(
            engine,
            ocr=FakeOcr(),
            face=FakeFaceMatch(),
            risk=FakeRiskScore(),
            clock=FixedClock(NOW),
            log=lambda _: None,
        )


def test_handler_receives_the_validated_message_not_the_raw_one(container: Container) -> None:
    raw = SubmitDriverApplication(
        driver_id="DR-001",
        cccd_ref="  synthetic/cccd/DR-001.png  ",  # khoảng trắng thừa: chỉ bản đã validate được cắt
        gplx_ref="synthetic/gplx/DR-001.png",
        selfie_ref="synthetic/selfie/DR-001.png",
        face_consent=True,
    )
    assert container.mediator.send(raw).status == "needs_review"  # bản thô làm OCR không thấy ảnh


def test_consent_given_as_string_is_rejected_before_the_handler(
    container: Container, engine: Engine, tx_events: list[str]
) -> None:
    raw = SubmitDriverApplication(
        "DR-001",
        "synthetic/cccd/DR-001.png",
        "synthetic/gplx/DR-001.png",
        "synthetic/selfie/DR-001.png",
        "false",  # type: ignore[arg-type]
    )
    with pytest.raises(ValidationFailed) as info:
        container.mediator.send(raw)
    assert [(e.field, e.code) for e in info.value.errors] == [("face_consent", "bool_type")]
    assert tx_events == []
    assert stored(engine) == []


def test_validation_builds_one_type_adapter_per_message_type(engine: Engine, clean: None) -> None:
    built: list[type] = []

    def counting(message_type: type) -> TypeAdapter[Any]:
        built.append(message_type)
        return TypeAdapter(message_type)

    mediator = Mediator([validation_behavior(counting)])
    mediator.register(
        GetApplication, GetApplicationHandler(SqlAlchemyReadModel(make_session_factory(engine)))
    )
    for _ in range(3):
        with pytest.raises(ApplicationNotFound):
            mediator.send(GetApplication("KYC-9999"))
    assert built == [GetApplication]


def test_application_id_has_no_upper_limit(container: Container) -> None:
    with pytest.raises(ApplicationNotFound):
        container.mediator.send(GetApplication("KYC-10000"))  # đúng hình dạng, chỉ là chưa có


def test_negative_cosine_reaches_the_reviewer(engine: Engine, clean: None) -> None:
    c = build_container(
        engine,
        face=FakeFaceMatch(score_override=-0.4),
        clock=FixedClock(NOW),
        log=lambda _: None,
    )
    view = c.mediator.send(submit_for("DR-001"))
    assert view.face_score == -0.4
    assert view.risk_score is not None
    assert 0.0 <= view.risk_score <= 1.0
