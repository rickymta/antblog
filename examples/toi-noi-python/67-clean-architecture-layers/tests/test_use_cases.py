"""Test use case bằng adapter giả, không HTTP, không cơ sở dữ liệu."""

import pytest

from kyc_service.api.container import Container, build_container
from kyc_service.application.errors import ApplicationNotFound
from kyc_service.domain.errors import InvalidStatusTransition
from kyc_service.domain.kyc import KycApplication, KycStatus, RiskScore
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.fake_services import FakeFaceMatch, FakeOcr, FakeRiskScore
from kyc_service.infrastructure.memory_repository import InMemoryKycRepository

from .conftest import NOW, request_for


def test_happy_path_ends_in_needs_review(container: Container) -> None:
    view = container.submit.execute(request_for("DR-001"))
    assert view.status == "needs_review"
    assert view.application_id == "KYC-0001"
    assert view.pending_check is None
    assert view.face_score == 0.88


def test_without_consent_face_and_risk_are_not_called(repo: InMemoryKycRepository) -> None:
    face, risk = FakeFaceMatch(), FakeRiskScore()
    c = build_container(repository=repo, face=face, risk=risk, clock=FixedClock(NOW))
    view = c.submit.execute(request_for("DR-003", consent=False))
    assert (face.calls, risk.calls) == (0, 0)
    assert view.status == "needs_review"
    assert view.note == "chưa có đồng ý so khớp khuôn mặt"


def test_service_down_keeps_application_at_last_reached_status(repo: InMemoryKycRepository) -> None:
    c = build_container(repository=repo, risk=FakeRiskScore(available=False), clock=FixedClock(NOW))
    view = c.submit.execute(request_for("DR-001"))
    assert (view.status, view.pending_check) == ("face_checked", "fraud-scoring")
    stored = repo.get("KYC-0001")
    assert stored is not None and stored.status is KycStatus.FACE_CHECKED


def test_repository_returns_copies(repo: InMemoryKycRepository, container: Container) -> None:
    container.submit.execute(request_for("DR-001"))
    loaded = repo.get("KYC-0001")
    assert loaded is not None
    loaded.note = "sửa mà không save"
    again = repo.get("KYC-0001")
    assert again is not None and again.note != "sửa mà không save"


def test_unexpected_error_leaves_submitted_application_behind(repo: InMemoryKycRepository) -> None:
    class Exploding:
        def score(self, application: KycApplication) -> RiskScore:
            raise RuntimeError("bug")

    c = build_container(repository=repo, risk=Exploding(), clock=FixedClock(NOW))
    with pytest.raises(RuntimeError):
        c.submit.execute(request_for("DR-001"))
    left = repo.get("KYC-0001")
    assert left is not None and left.status is KycStatus.SUBMITTED  # giới hạn: chưa có transaction


def test_review_flow(container: Container) -> None:
    container.submit.execute(request_for("DR-002"))
    view = container.review.execute(
        "KYC-0001", reviewer_id="RV-001", decision=KycStatus.REJECTED, reason="OCR mờ, selfie lệch"
    )
    assert (view.status, view.reviewed_by) == ("rejected", "RV-001")
    with pytest.raises(InvalidStatusTransition):
        container.review.execute(
            "KYC-0001", reviewer_id="RV-001", decision=KycStatus.APPROVED, reason="đổi ý"
        )


def test_unknown_application(container: Container) -> None:
    with pytest.raises(ApplicationNotFound):
        container.get.execute("KYC-9999")
    with pytest.raises(ApplicationNotFound):
        container.review.execute(
            "KYC-9999", reviewer_id="RV-001", decision=KycStatus.APPROVED, reason="x"
        )


def test_ocr_down_does_not_call_later_ports(repo: InMemoryKycRepository) -> None:
    face = FakeFaceMatch()
    c = build_container(
        repository=repo, ocr=FakeOcr(available=False), face=face, clock=FixedClock(NOW)
    )
    c.submit.execute(request_for("DR-001"))
    assert face.calls == 0


def test_negative_cosine_reaches_reviewer_and_risk_stays_in_range(
    repo: InMemoryKycRepository,
) -> None:
    c = build_container(
        repository=repo, face=FakeFaceMatch(score_override=-0.4), clock=FixedClock(NOW)
    )
    view = c.submit.execute(request_for("DR-001"))
    assert view.face_score == -0.4
    assert view.risk_score is not None
    assert 0.0 <= view.risk_score <= 1.0


def test_build_container_warns_about_fake_adapters() -> None:
    with pytest.warns(UserWarning, match="adapter giả cho: repository, ocr, face, risk"):
        build_container(clock=FixedClock(NOW))
