"""Test domain thuần: không port, không adapter, không framework."""

import itertools

import pytest

from kyc_service.domain.errors import DomainError, InvalidStatusTransition
from kyc_service.domain.kyc import (
    ALLOWED,
    DocumentImage,
    DocumentKind,
    FaceMatch,
    KycApplication,
    KycStatus,
    OcrResult,
    RiskScore,
    advance,
    mask_id_number,
)

from .conftest import NOW


def make_application() -> KycApplication:
    return KycApplication.submit(
        application_id="KYC-0001",
        driver_id="DR-001",
        documents=(DocumentImage(DocumentKind.GPLX, "g"), DocumentImage(DocumentKind.CCCD, "c")),
        selfie_ref="s",
        face_consent=True,
        now=NOW,
    )


@pytest.mark.parametrize(("current", "target"), list(itertools.product(KycStatus, KycStatus)))
def test_transition_matrix_matches_table(current: KycStatus, target: KycStatus) -> None:
    if target in ALLOWED[current]:
        assert advance(current, target) is target
    else:
        with pytest.raises(InvalidStatusTransition):
            advance(current, target)


def test_approved_and_rejected_only_come_from_needs_review() -> None:
    sources = {s for s, targets in ALLOWED.items() if KycStatus.APPROVED in targets}
    assert sources == {KycStatus.NEEDS_REVIEW}
    sources = {s for s, targets in ALLOWED.items() if KycStatus.REJECTED in targets}
    assert sources == {KycStatus.NEEDS_REVIEW}


def test_submit_orders_documents_cccd_first() -> None:
    app = make_application()
    assert [d.kind for d in app.documents] == [DocumentKind.CCCD, DocumentKind.GPLX]
    assert app.status is KycStatus.SUBMITTED


@pytest.mark.parametrize(
    "kinds",
    [
        (DocumentKind.CCCD,),
        (DocumentKind.CCCD, DocumentKind.CCCD),
        (DocumentKind.GPLX,) * 2,
        (DocumentKind.CCCD, DocumentKind.GPLX, DocumentKind.GPLX),
    ],
    ids=["một", "hai-cccd", "hai-gplx", "ba"],
)
def test_application_needs_exactly_one_cccd_and_one_gplx(kinds: tuple[DocumentKind, ...]) -> None:
    docs = tuple(DocumentImage(k, f"ref{i}") for i, k in enumerate(kinds))
    with pytest.raises(DomainError, match="đúng hai giấy tờ"):
        KycApplication.submit(
            application_id="KYC-0001",
            driver_id="DR-001",
            documents=docs,
            selfie_ref="s",
            face_consent=True,
            now=NOW,
        )


def test_decide_before_needs_review_is_rejected() -> None:
    app = make_application()
    with pytest.raises(InvalidStatusTransition):
        app.decide(reviewer_id="RV-001", decision=KycStatus.APPROVED, reason="ép", now=NOW)
    assert app.status is KycStatus.SUBMITTED


@pytest.mark.parametrize("value", [-0.01, 1.01, float("nan")])
def test_scores_must_be_in_unit_interval(value: float) -> None:
    with pytest.raises(DomainError):
        RiskScore(value)
    with pytest.raises(DomainError):
        OcrResult(DocumentKind.CCCD, {}, value)


def test_ocr_fields_are_read_only() -> None:
    result = OcrResult(DocumentKind.CCCD, {"full_name": "Nguyễn Văn A"}, 0.9)
    with pytest.raises(TypeError):
        result.fields["full_name"] = "x"  # type: ignore[index]


@pytest.mark.parametrize(
    ("raw", "masked"), [("099090000183", "099*******83"), ("1234567", "*******")]
)
def test_mask_id_number(raw: str, masked: str) -> None:
    assert mask_id_number(raw) == masked


@pytest.mark.parametrize("value", [-1.0, -0.4, 0.0, 1.0])
def test_face_match_is_cosine_in_minus_one_to_one(value: float) -> None:
    assert FaceMatch(value).score == value


@pytest.mark.parametrize("value", [-1.01, 1.01, float("nan")])
def test_face_match_rejects_outside_cosine_range(value: float) -> None:
    with pytest.raises(DomainError):
        FaceMatch(value)


def test_application_id_keeps_accepting_after_9999() -> None:
    app = KycApplication.submit(
        application_id="KYC-10000",
        driver_id="DR-001",
        documents=(DocumentImage(DocumentKind.CCCD, "c"), DocumentImage(DocumentKind.GPLX, "g")),
        selfie_ref="s",
        face_consent=True,
        now=NOW,
    )
    assert app.application_id == "KYC-10000"


@pytest.mark.parametrize("bad", ["KYC-١٢٣٤", "KYC-123", "kyc-0001"])
def test_application_id_rejects_unicode_digits_and_short_ids(bad: str) -> None:
    with pytest.raises(DomainError, match="mã hồ sơ"):
        KycApplication.submit(
            application_id=bad,
            driver_id="DR-001",
            documents=(
                DocumentImage(DocumentKind.CCCD, "c"),
                DocumentImage(DocumentKind.GPLX, "g"),
            ),
            selfie_ref="s",
            face_consent=True,
            now=NOW,
        )


def test_face_consent_must_be_a_real_bool() -> None:
    with pytest.raises(DomainError, match="face_consent"):
        KycApplication.submit(
            application_id="KYC-0001",
            driver_id="DR-001",
            documents=(
                DocumentImage(DocumentKind.CCCD, "c"),
                DocumentImage(DocumentKind.GPLX, "g"),
            ),
            selfie_ref="s",
            face_consent="false",  # type: ignore[arg-type]
            now=NOW,
        )
