"""DTO trả ra ngoài application: không để aggregate rò sang api."""

from dataclasses import dataclass

from kyc_service.domain.kyc import KycApplication, KycStatus, mask_id_number

# Bước tự động kế tiếp của một hồ sơ còn dở: service nào chưa kiểm được. Suy ra từ trạng thái nên
# đọc lại hồ sơ (GET) cũng thấy, không chỉ lần phản hồi 202 đầu tiên.
NEXT_CHECK: dict[str, str] = {
    KycStatus.SUBMITTED.value: "kyc-ocr",
    KycStatus.OCR_DONE.value: "kyc-face",
    KycStatus.FACE_CHECKED.value: "fraud-scoring",
}


@dataclass(frozen=True)
class OcrView:
    kind: str
    id_number_masked: str | None
    confidence: float


@dataclass(frozen=True)
class ApplicationView:
    application_id: str
    driver_id: str
    status: str
    ocr: tuple[OcrView, ...]
    face_score: float | None
    risk_score: float | None
    risk_reasons: tuple[str, ...]
    note: str | None
    reviewed_by: str | None
    pending_check: str | None = None  # service chưa kiểm được; hồ sơ vẫn được nhận

    @classmethod
    def of(cls, app: KycApplication, *, pending_check: str | None = None) -> ApplicationView:
        return cls(
            application_id=app.application_id,
            driver_id=app.driver_id,
            status=app.status.value,
            ocr=tuple(
                OcrView(
                    kind=r.kind.value,
                    id_number_masked=(
                        mask_id_number(r.fields["id_number"]) if "id_number" in r.fields else None
                    ),
                    confidence=r.confidence,
                )
                for r in app.ocr_results
            ),
            face_score=app.face_match.score if app.face_match else None,
            risk_score=app.risk_score.value if app.risk_score else None,
            risk_reasons=app.risk_score.reasons if app.risk_score else (),
            note=app.note,
            reviewed_by=app.review.reviewer_id if app.review else None,
            pending_check=pending_check or NEXT_CHECK.get(app.status.value),
        )
