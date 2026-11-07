"""Ba adapter giả cho kyc-ocr, kyc-face, fraud-scoring. Adapter thật nằm ở P05–P07.

Adapter chịu hai việc: gọi service (ở đây tra bảng) và dịch lỗi của service về `PortFailure`, để
application không phải biết service nói giao thức gì. Cờ `available=False` mô phỏng service tắt.
"""

from kyc_service.application.errors import InvalidServiceResponse, ServiceUnavailable
from kyc_service.domain.errors import DomainError
from kyc_service.domain.kyc import (
    DocumentImage,
    FaceMatch,
    KycApplication,
    OcrResult,
    RiskScore,
)
from kyc_service.infrastructure.fixtures import FACE_FIXTURES, OCR_FIXTURES


class FakeOcr:
    def __init__(self, *, available: bool = True) -> None:
        self.available = available
        self.calls = 0

    def read(self, document: DocumentImage) -> OcrResult:
        self.calls += 1
        if not self.available:
            raise ServiceUnavailable("kyc-ocr")
        try:
            kind, fields, confidence = OCR_FIXTURES[document.image_ref]
            if kind is not document.kind:
                raise InvalidServiceResponse(
                    "kyc-ocr", f"ảnh {document.image_ref} không phải {document.kind}"
                )
            return OcrResult(kind, fields, confidence)
        except KeyError as exc:
            raise InvalidServiceResponse("kyc-ocr", f"không có ảnh {document.image_ref}") from exc
        except DomainError as exc:
            raise InvalidServiceResponse("kyc-ocr", str(exc)) from exc


class FakeFaceMatch:
    def __init__(self, *, available: bool = True, score_override: float | None = None) -> None:
        self.available = available
        self.score_override = score_override  # mô phỏng service trả điểm sai miền
        self.calls = 0

    def compare(self, document: DocumentImage, selfie_ref: str) -> FaceMatch:
        self.calls += 1
        if not self.available:
            raise ServiceUnavailable("kyc-face")
        try:
            raw = FACE_FIXTURES[selfie_ref] if self.score_override is None else self.score_override
            return FaceMatch(raw)
        except KeyError as exc:
            raise InvalidServiceResponse("kyc-face", f"không có ảnh {selfie_ref}") from exc
        except DomainError as exc:
            raise InvalidServiceResponse("kyc-face", str(exc)) from exc


class FakeRiskScore:
    """Điểm rủi ro giả: OCR, khuôn mặt càng kém tin cậy thì càng cao. Chỉ để chạy luồng."""

    def __init__(self, *, available: bool = True) -> None:
        self.available = available
        self.calls = 0

    def score(self, application: KycApplication) -> RiskScore:
        self.calls += 1
        if not self.available:
            raise ServiceUnavailable("fraud-scoring")
        assert application.face_match is not None  # use case chỉ gọi sau bước khuôn mặt
        lowest_ocr = min(r.confidence for r in application.ocr_results)
        face = max(application.face_match.score, 0.0)  # cosine âm coi như 0 tương đồng
        reasons = []
        if lowest_ocr < 0.7:
            reasons.append(f"OCR độ tin cậy thấp ({lowest_ocr:.2f})")
        if face < 0.8:
            reasons.append(f"điểm khuôn mặt thấp ({application.face_match.score:.2f})")
        return RiskScore(round(1 - (lowest_ocr + face) / 2, 2), tuple(reasons))
