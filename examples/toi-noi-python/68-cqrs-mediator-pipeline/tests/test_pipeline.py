"""Test pipeline thật (log, validate, transaction) trên service KYC."""

from typing import Any

import pytest
from pydantic import TypeAdapter

from kyc_service.api.container import Container, build_container
from kyc_service.application.behaviors import validation_behavior
from kyc_service.application.commands import RecordReviewDecision, SubmitDriverApplication
from kyc_service.application.errors import ApplicationNotFound, ValidationFailed
from kyc_service.application.handlers import GetApplicationHandler
from kyc_service.application.mediator import Mediator
from kyc_service.application.queries import GetApplication, ListReviewQueue
from kyc_service.domain.errors import InvalidStatusTransition
from kyc_service.domain.kyc import KycApplication, RiskScore
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.fake_services import FakeFaceMatch, FakeOcr
from kyc_service.infrastructure.memory_read_model import InMemoryReadModel

from .conftest import NOW, submit_for


def test_command_runs_log_validate_transaction(container: Container, events: list[str]) -> None:
    view = container.mediator.send(submit_for("DR-001"))
    assert view.status == "needs_review"
    assert events == ["> SubmitDriverApplication", "< SubmitDriverApplication ok"]
    assert container.transactions.events == ["begin", "commit"]


def test_query_does_not_open_a_transaction(container: Container) -> None:
    container.mediator.send(submit_for("DR-001"))
    container.transactions.events.clear()
    assert container.mediator.send(GetApplication("KYC-0001")).driver_id == "DR-001"
    assert container.transactions.events == []


def test_invalid_command_never_reaches_transaction_or_handler(
    container: Container, events: list[str]
) -> None:
    bad = SubmitDriverApplication("DR-1", "", "x", "y", True)
    with pytest.raises(ValidationFailed) as info:
        container.mediator.send(bad)
    assert [(e.field, e.code) for e in info.value.errors] == [
        ("driver_id", "string_pattern_mismatch"),
        ("cccd_ref", "string_too_short"),
    ]
    assert container.transactions.events == []
    assert container.repository.count() == 0
    assert events == ["> SubmitDriverApplication", "! SubmitDriverApplication ValidationFailed"]


def test_validation_error_does_not_contain_the_input(container: Container) -> None:
    with pytest.raises(ValidationFailed) as info:
        container.mediator.send(submit_for("BI-MAT-12345"))
    assert "BI-MAT-12345" not in repr(info.value.errors)


def test_unexpected_error_rolls_back(events: list[str]) -> None:
    class Exploding:
        def score(self, application: KycApplication) -> RiskScore:
            raise RuntimeError("bug")

    c = build_container(risk=Exploding(), clock=FixedClock(NOW), log=events.append)
    with pytest.raises(RuntimeError):
        c.mediator.send(submit_for("DR-001"))
    assert c.repository.count() == 0  # khác lab 67: không còn hồ sơ mồ côi
    assert c.transactions.events == ["begin", "rollback"]
    assert events[-1] == "! SubmitDriverApplication RuntimeError"


def test_rollback_does_not_rewind_the_id_sequence(events: list[str]) -> None:
    class ExplodingOnce:
        def __init__(self) -> None:
            self.armed = True

        def score(self, application: KycApplication) -> RiskScore:
            if self.armed:
                self.armed = False
                raise RuntimeError("bug")
            return RiskScore(0.1)

    c = build_container(risk=ExplodingOnce(), clock=FixedClock(NOW), log=events.append)
    with pytest.raises(RuntimeError):
        c.mediator.send(submit_for("DR-001"))
    assert c.mediator.send(submit_for("DR-001")).application_id == "KYC-0002"


def test_port_failure_is_handled_and_commits(events: list[str]) -> None:
    c = build_container(ocr=FakeOcr(available=False), clock=FixedClock(NOW), log=events.append)
    view = c.mediator.send(submit_for("DR-001"))
    assert (view.status, view.pending_check) == ("submitted", "kyc-ocr")
    assert c.transactions.events == ["begin", "commit"]
    assert c.repository.count() == 1


def test_face_down_keeps_ocr_results(events: list[str]) -> None:
    c = build_container(
        face=FakeFaceMatch(available=False), clock=FixedClock(NOW), log=events.append
    )
    view = c.mediator.send(submit_for("DR-001"))
    assert (view.status, view.pending_check, len(view.ocr)) == ("ocr_done", "kyc-face", 2)


def test_review_flow_and_rejected_second_decision(container: Container) -> None:
    container.mediator.send(submit_for("DR-001"))
    approve = RecordReviewDecision("KYC-0001", "RV-001", "approved", "khớp")
    assert container.mediator.send(approve).status == "approved"
    container.transactions.events.clear()
    with pytest.raises(InvalidStatusTransition):
        container.mediator.send(RecordReviewDecision("KYC-0001", "RV-001", "rejected", "đổi ý"))
    assert container.transactions.events == ["begin", "rollback"]
    assert container.mediator.send(GetApplication("KYC-0001")).status == "approved"


def test_review_decision_must_be_approved_or_rejected(container: Container) -> None:
    container.mediator.send(submit_for("DR-001"))
    with pytest.raises(ValidationFailed) as info:
        container.mediator.send(RecordReviewDecision("KYC-0001", "RV-001", "needs_review", "x"))  # type: ignore[arg-type]
    assert info.value.errors[0].field == "decision"


def test_unknown_application(container: Container) -> None:
    with pytest.raises(ApplicationNotFound):
        container.mediator.send(GetApplication("KYC-9999"))


def test_review_queue_lists_only_needs_review(container: Container) -> None:
    for driver in ("DR-001", "DR-002", "DR-003"):
        container.mediator.send(submit_for(driver))
    container.mediator.send(RecordReviewDecision("KYC-0002", "RV-001", "rejected", "OCR mờ"))
    queue = container.mediator.send(ListReviewQueue())
    assert [v.application_id for v in queue] == ["KYC-0001", "KYC-0003"]


def test_handler_receives_the_validated_message_not_the_raw_one(container: Container) -> None:
    raw = SubmitDriverApplication(
        driver_id="DR-001",
        cccd_ref="  synthetic/cccd/DR-001.png  ",  # khoảng trắng thừa: chỉ bản đã validate được cắt
        gplx_ref="synthetic/gplx/DR-001.png",
        selfie_ref="synthetic/selfie/DR-001.png",
        face_consent=True,
    )
    view = container.mediator.send(raw)
    assert view.status == "needs_review"  # bản thô sẽ làm OCR không tìm thấy ảnh


def test_consent_given_as_string_is_rejected_before_the_handler(container: Container) -> None:
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
    assert container.repository.count() == 0
    assert container.transactions.events == []


def test_validation_builds_one_type_adapter_per_message_type() -> None:
    built: list[type] = []

    def counting(message_type: type) -> TypeAdapter[Any]:
        built.append(message_type)
        return TypeAdapter(message_type)

    c = build_container(clock=FixedClock(NOW), log=lambda _: None)
    mediator = Mediator([validation_behavior(counting)])
    mediator.register(GetApplication, GetApplicationHandler(InMemoryReadModel(c.repository)))
    for _ in range(3):
        with pytest.raises(ApplicationNotFound):
            mediator.send(GetApplication("KYC-9999"))
    assert built == [GetApplication]


def test_application_id_has_no_upper_limit(container: Container) -> None:
    with pytest.raises(ApplicationNotFound):
        container.mediator.send(GetApplication("KYC-10000"))  # hợp lệ về hình dạng, chỉ là chưa có


def test_negative_cosine_reaches_reviewer(events: list[str]) -> None:
    c = build_container(
        face=FakeFaceMatch(score_override=-0.4), clock=FixedClock(NOW), log=events.append
    )
    view = c.mediator.send(submit_for("DR-001"))
    assert view.face_score == -0.4
    assert view.risk_score is not None
    assert 0.0 <= view.risk_score <= 1.0


def test_build_container_warns_about_fake_adapters() -> None:
    with pytest.warns(UserWarning, match="adapter giả cho: repository, ocr, face, risk"):
        build_container(clock=FixedClock(NOW))
