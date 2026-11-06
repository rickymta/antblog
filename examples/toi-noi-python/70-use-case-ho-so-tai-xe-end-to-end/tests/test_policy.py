"""Bảng ca cho ngưỡng: chỉ chọn LÝ DO vào hàng chờ, không bao giờ duyệt hay từ chối."""

import pytest

from kyc_service.domain.errors import DomainError
from kyc_service.domain.events import KycApplicationScored
from kyc_service.domain.kyc import (
    DocumentImage,
    DocumentKind,
    FaceMatch,
    KycApplication,
    KycStatus,
    OcrResult,
    RiskScore,
)
from kyc_service.domain.policy import ReviewPolicy
from kyc_service.domain.review import ReviewReason as R

from .conftest import NOW

type Inputs = tuple[float, float | None, float | None, bool]

POLICY = ReviewPolicy()  # ocr 0.80, khuôn mặt 0.75, rủi ro từ 0.30


def application(
    ocr_min: float, face: float | None, risk: float | None, *, consent: bool = True
) -> KycApplication:
    app = KycApplication.submit(
        application_id="KYC-0001",
        driver_id="DR-001",
        documents=(DocumentImage(DocumentKind.CCCD, "c"), DocumentImage(DocumentKind.GPLX, "g")),
        selfie_ref="s",
        face_consent=consent,
        now=NOW,
    )
    app.record_ocr(
        (OcrResult(DocumentKind.CCCD, {}, ocr_min), OcrResult(DocumentKind.GPLX, {}, 1.0))
    )
    if face is not None:
        app.record_face(FaceMatch(face))
    if risk is not None:
        app.record_risk(RiskScore(risk))
    return app


CASES = [
    # (ocr_min, face, risk, consent) -> lý do
    ((0.93, 0.88, 0.10, True), (R.ROUTINE,)),
    ((1.00, 1.00, 0.00, True), (R.ROUTINE,)),  # điểm tốt nhất vẫn vào hàng chờ
    ((0.80, 0.88, 0.10, True), (R.ROUTINE,)),  # đúng ngưỡng OCR: chưa gắn cờ
    ((0.79, 0.88, 0.10, True), (R.OCR_LOW_CONFIDENCE,)),
    ((0.93, 0.75, 0.10, True), (R.ROUTINE,)),  # đúng ngưỡng khuôn mặt: chưa gắn cờ
    ((0.93, 0.74, 0.10, True), (R.FACE_LOW_SCORE,)),
    ((0.93, 0.88, 0.29, True), (R.ROUTINE,)),
    ((0.93, 0.88, 0.30, True), (R.RISK_HIGH,)),  # đúng ngưỡng rủi ro: gắn cờ (>=)
    ((0.58, 0.71, 0.36, True), (R.OCR_LOW_CONFIDENCE, R.FACE_LOW_SCORE, R.RISK_HIGH)),
    ((0.99, None, None, False), (R.NO_FACE_CONSENT,)),
    ((0.50, None, None, False), (R.NO_FACE_CONSENT, R.OCR_LOW_CONFIDENCE)),
]


@pytest.mark.parametrize(("inputs", "expected"), CASES)
def test_reasons_table(inputs: Inputs, expected: tuple[R, ...]) -> None:
    ocr_min, face, risk, consent = inputs
    assert POLICY.reasons(application(ocr_min, face, risk, consent=consent)) == expected


@pytest.mark.parametrize(("inputs", "reasons"), CASES)
def test_no_input_ever_reaches_approved_or_rejected(inputs: Inputs, reasons: tuple[R, ...]) -> None:
    ocr_min, face, risk, consent = inputs
    app = application(ocr_min, face, risk, consent=consent)
    app.send_to_review(POLICY.reasons(app), now=NOW)
    assert app.status is KycStatus.NEEDS_REVIEW
    assert app.review is None


def test_thresholds_must_be_probabilities() -> None:
    with pytest.raises(DomainError):
        ReviewPolicy(face_min_score=1.2)
    with pytest.raises(DomainError):
        ReviewPolicy(risk_review_at=float("nan"))


def test_a_review_without_any_reason_is_refused() -> None:
    app = application(0.9, 0.9, 0.1)
    with pytest.raises(DomainError):
        app.send_to_review((), now=NOW)
    assert app.status is KycStatus.RISK_SCORED


def test_scored_event_is_raised_once_with_the_numbers_and_reasons() -> None:
    app = application(0.58, 0.71, 0.36)
    app.send_to_review(POLICY.reasons(app), now=NOW)
    (event,) = app.pull_events()
    assert event == KycApplicationScored(
        occurred_at=NOW,
        application_id="KYC-0001",
        driver_id="DR-001",
        risk_score=0.36,
        face_score=0.71,
        ocr_min_confidence=0.58,
        review_reasons=(R.OCR_LOW_CONFIDENCE, R.FACE_LOW_SCORE, R.RISK_HIGH),
    )
    assert app.pull_events() == ()  # lấy rồi là hết


def test_no_scored_event_without_consent() -> None:
    app = application(0.9, None, None, consent=False)
    app.send_to_review(POLICY.reasons(app), now=NOW)
    assert app.pull_events() == ()
