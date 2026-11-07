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
def test_risk_and_ocr_scores_must_be_in_unit_interval(value: float) -> None:
    with pytest.raises(DomainError):
        RiskScore(value)
    with pytest.raises(DomainError):
        OcrResult(DocumentKind.CCCD, {}, value)


@pytest.mark.parametrize("value", [-1.0, -0.12, 0.0, 0.88, 1.0])
def test_face_score_is_a_cosine_so_it_may_be_negative(value: float) -> None:
    assert FaceMatch(value).score == value


@pytest.mark.parametrize("value", [-1.01, 1.01, float("nan"), float("inf")])
def test_face_score_outside_cosine_range_is_rejected(value: float) -> None:
    with pytest.raises(DomainError):
        FaceMatch(value)


@pytest.mark.parametrize("application_id", ["KYC-0001", "KYC-9999", "KYC-10000", "KYC-123456"])
def test_application_id_does_not_stop_at_9999(application_id: str) -> None:
    app = KycApplication.submit(
        application_id=application_id,
        driver_id="DR-001",
        documents=(DocumentImage(DocumentKind.CCCD, "c"), DocumentImage(DocumentKind.GPLX, "g")),
        selfie_ref="s",
        face_consent=True,
        now=NOW,
    )
    assert app.application_id == application_id


@pytest.mark.parametrize("application_id", ["KYC-001", "KYC-٠٠٠١", "kyc-0001", "KYC-0001 "])
def test_application_id_rejects_short_and_non_ascii_digits(application_id: str) -> None:
    with pytest.raises(DomainError):
        KycApplication.submit(
            application_id=application_id,
            driver_id="DR-001",
            documents=(
                DocumentImage(DocumentKind.CCCD, "c"),
                DocumentImage(DocumentKind.GPLX, "g"),
            ),
            selfie_ref="s",
            face_consent=True,
            now=NOW,
        )


def test_ocr_fields_are_read_only() -> None:
    result = OcrResult(DocumentKind.CCCD, {"full_name": "Nguyễn Văn A"}, 0.9)
    with pytest.raises(TypeError):
        result.fields["full_name"] = "x"  # type: ignore[index]


@pytest.mark.parametrize(
    ("raw", "masked"),
    [
        ("099090000183", "099*******83"),
        ("099*******83", "099*******83"),  # che hai lần cho cùng kết quả
        ("1234567", "*******"),  # ngắn: che hết
        ("0990900001AB", "************"),  # lẫn chữ: che hết
        ("٠٩٩٠٩٠٠٠٠١٨٣", "************"),  # chữ số Arabic-Indic: che hết
    ],
)
def test_mask_id_number(raw: str, masked: str) -> None:
    assert mask_id_number(raw) == masked
