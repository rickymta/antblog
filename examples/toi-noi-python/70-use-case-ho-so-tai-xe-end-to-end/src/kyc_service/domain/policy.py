"""Ngưỡng và lý do vào hàng chờ. Ngưỡng KHÔNG quyết định duyệt hay từ chối.

Máy luôn dừng ở `needs_review`; ngưỡng chỉ chọn LÝ DO vào hàng chờ để người duyệt biết nhìn gì
trước. Giá trị mặc định ở đây là ví dụ cho lab, không rút ra từ dữ liệu của bài này: ngưỡng thật
phải đo trên chính nguồn điểm của adapter thật (OCR bài 45, khuôn mặt bài 49, rủi ro bài 56).
"""

from dataclasses import dataclass

from kyc_service.domain.errors import DomainError
from kyc_service.domain.kyc import KycApplication
from kyc_service.domain.review import ReviewReason


@dataclass(frozen=True)
class ReviewPolicy:
    ocr_min_confidence: float = 0.80  # giấy tờ có confidence thấp hơn mức này bị gắn cờ
    face_min_score: float = 0.75  # điểm khuôn mặt thấp hơn mức này bị gắn cờ
    risk_review_at: float = 0.30  # điểm rủi ro từ mức này trở lên bị gắn cờ

    def __post_init__(self) -> None:
        # Điểm khuôn mặt là cosine (-1..1); OCR và rủi ro nằm trong 0..1.
        for name, low in (
            ("ocr_min_confidence", 0.0),
            ("face_min_score", -1.0),
            ("risk_review_at", 0.0),
        ):
            if not low <= getattr(self, name) <= 1.0:
                raise DomainError(f"ngưỡng {name} ngoài miền {low:g}..1: {getattr(self, name)}")

    def reasons(self, application: KycApplication) -> tuple[ReviewReason, ...]:
        """Các lý do vào hàng chờ, tính từ những gì hồ sơ đã có. Luôn có ít nhất một lý do."""
        found: list[ReviewReason] = []
        if not application.face_consent:
            found.append(ReviewReason.NO_FACE_CONSENT)
        if application.ocr_results and (
            min(r.confidence for r in application.ocr_results) < self.ocr_min_confidence
        ):
            found.append(ReviewReason.OCR_LOW_CONFIDENCE)
        if application.face_match and application.face_match.score < self.face_min_score:
            found.append(ReviewReason.FACE_LOW_SCORE)
        if application.risk_score and application.risk_score.value >= self.risk_review_at:
            found.append(ReviewReason.RISK_HIGH)
        return tuple(found) or (ReviewReason.ROUTINE,)
