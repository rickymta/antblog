"""Hồ sơ KYC của tài xế: trạng thái, giấy tờ, kết quả kiểm tra, quyết định của người duyệt.

Trạng thái và bảng chuyển lấy từ bài 41. Kết quả OCR, điểm khuôn mặt, điểm rủi ro chỉ là đầu vào
cho người duyệt: `approved` và `rejected` chỉ đến từ `needs_review`.
"""

import re
from collections.abc import Mapping
from dataclasses import dataclass, field
from datetime import datetime
from enum import StrEnum
from types import MappingProxyType
from typing import Self

from kyc_service.domain.errors import DomainError, InvalidStatusTransition


class KycStatus(StrEnum):
    SUBMITTED = "submitted"  # tài xế đã nộp ảnh, chưa service nào xử lý
    OCR_DONE = "ocr_done"  # kyc-ocr đã đọc xong các trường
    FACE_CHECKED = "face_checked"  # kyc-face đã so ảnh chân dung với selfie
    RISK_SCORED = "risk_scored"  # fraud-scoring đã chấm điểm rủi ro
    NEEDS_REVIEW = "needs_review"  # đang chờ người duyệt xem
    APPROVED = "approved"
    REJECTED = "rejected"


S = KycStatus

# Trạng thái -> các trạng thái được đi tiếp. Bước nào không đủ tin cậy đều có đường sang
# NEEDS_REVIEW; APPROVED và REJECTED chỉ đến từ NEEDS_REVIEW (máy không tự quyết định một người).
ALLOWED: dict[KycStatus, frozenset[KycStatus]] = {
    S.SUBMITTED: frozenset({S.OCR_DONE, S.NEEDS_REVIEW}),
    S.OCR_DONE: frozenset({S.FACE_CHECKED, S.NEEDS_REVIEW}),
    S.FACE_CHECKED: frozenset({S.RISK_SCORED, S.NEEDS_REVIEW}),
    S.RISK_SCORED: frozenset({S.NEEDS_REVIEW}),
    S.NEEDS_REVIEW: frozenset({S.APPROVED, S.REJECTED}),
    S.APPROVED: frozenset(),
    S.REJECTED: frozenset(),
}


def advance(current: KycStatus, target: KycStatus) -> KycStatus:
    """Trả về `target` nếu chuyển hợp lệ, không thì ném InvalidStatusTransition."""
    if target not in ALLOWED[current]:
        raise InvalidStatusTransition(
            f"không được chuyển hồ sơ KYC từ {current.value} sang {target.value}"
        )
    return target


class DocumentKind(StrEnum):
    CCCD = "cccd"
    GPLX = "gplx"


# Mã hồ sơ không ngừng nhận ở KYC-9999: `next_id` in 5 chữ số trở lên khi sequence vượt 9999.
# Dùng [0-9] chứ không dùng \d: \d của `re` nhận cả chữ số Unicode (Arabic-Indic...).
APPLICATION_ID = re.compile(r"KYC-[0-9]{4,}")
DRIVER_ID = re.compile(r"DR-[0-9]{3}")
REVIEWER_ID = re.compile(r"RV-[0-9]{3}")


def _require_id(pattern: re.Pattern[str], value: str, name: str) -> str:
    if not pattern.fullmatch(value):
        raise DomainError(f"{name} sai định dạng: {value!r}")
    return value


def _require_range(value: float, low: float, high: float, name: str) -> float:
    if not low <= value <= high:  # NaN cũng rơi vào nhánh sai
        raise DomainError(f"{name} ngoài miền {low:g}..{high:g}: {value}")
    return value


def _require_unit_interval(value: float, name: str) -> float:
    return _require_range(value, 0.0, 1.0, name)


_DIGITS = re.compile(r"[0-9]{8,}")
_MASKED = re.compile(r"[0-9]{3}\*+[0-9]{2}")
# Trường OCR là số định danh: vào aggregate (và xuống cơ sở dữ liệu) chỉ dạng đã che.
PII_FIELDS = ("id_number", "license_number")


def mask_id_number(id_number: str) -> str:
    """Số giấy tờ chỉ ra ngoài dạng mặt nạ `099*******83`: giữ 3 số đầu và 2 số cuối.

    Khác `mask_digits` của bài 47 (từ chối chuỗi ngắn, lẫn chữ, chữ số không phải ASCII): ở đây
    chuỗi lạ bị che hết thành dấu `*` để hồ sơ vẫn tới người duyệt thay vì làm hỏng lệnh nộp.
    Chuỗi đã che giữ nguyên, nên che hai lần cho cùng kết quả.
    """
    if _MASKED.fullmatch(id_number):
        return id_number
    if not _DIGITS.fullmatch(id_number):
        return "*" * len(id_number)
    return id_number[:3] + "*" * (len(id_number) - 5) + id_number[-2:]


@dataclass(frozen=True)
class DocumentImage:
    """Tham chiếu tới ảnh giấy tờ đã lưu (khóa object), không phải byte ảnh."""

    kind: DocumentKind
    image_ref: str

    def __post_init__(self) -> None:
        if not self.image_ref.strip():
            raise DomainError("image_ref không được rỗng")


@dataclass(frozen=True)
class OcrResult:
    """Các trường OCR đọc được từ một giấy tờ, kèm độ tin cậy 0..1. Đầu vào cho người duyệt."""

    kind: DocumentKind
    fields: Mapping[str, str]
    confidence: float
    # Khóa nối sang document OCR của bài 47: (kyc_id, loại giấy tờ, model_version, image_sha256).
    model_version: str = "unknown"
    image_sha256: str | None = None

    def __post_init__(self) -> None:
        _require_unit_interval(self.confidence, "confidence")
        # Che số NGAY khi kết quả OCR vào miền: mọi nơi phía sau (adapter rủi ro, kho, view)
        # chỉ thấy dạng đã che, dù hồ sơ vừa qua OCR hay vừa nạp lại từ cơ sở dữ liệu.
        masked = {k: mask_id_number(v) if k in PII_FIELDS else v for k, v in self.fields.items()}
        object.__setattr__(self, "fields", MappingProxyType(masked))

    def __deepcopy__(self, memo: dict[int, object]) -> Self:
        return self  # bất biến nên chia sẻ được; mappingproxy cũng không deepcopy được


@dataclass(frozen=True)
class FaceMatch:
    """Điểm cosine -1..1 giữa ảnh trên giấy tờ và selfie (đúng miền cosine của SFace, bài 49).

    Có thể âm. Không phải xác suất và không phải kết luận «cùng người».
    """

    score: float

    def __post_init__(self) -> None:
        _require_range(self.score, -1.0, 1.0, "score khuôn mặt")


@dataclass(frozen=True)
class RiskScore:
    """Điểm rủi ro 0..1 kèm lý do để người duyệt đọc. Không phải kết luận gian lận.

    Engine bài 56 chấm trên thang 0..100: chia 100 khi đưa vào đây (`review_at: 40` là 0,40).
    """

    value: float
    reasons: tuple[str, ...] = ()

    def __post_init__(self) -> None:
        _require_unit_interval(self.value, "điểm rủi ro")


@dataclass(frozen=True)
class ReviewDecision:
    reviewer_id: str
    decision: KycStatus
    reason: str
    decided_at: datetime


@dataclass
class KycApplication:
    """Aggregate: một hồ sơ KYC (`KYC-0001`) của một tài xế, gồm đúng hai giấy tờ CCCD và GPLX."""

    application_id: str
    driver_id: str
    documents: tuple[DocumentImage, DocumentImage]
    selfie_ref: str
    face_consent: bool
    submitted_at: datetime
    status: KycStatus = KycStatus.SUBMITTED
    ocr_results: tuple[OcrResult, ...] = ()
    face_match: FaceMatch | None = None
    risk_score: RiskScore | None = None
    note: str | None = None  # vì sao hồ sơ vào hàng chờ duyệt
    review: ReviewDecision | None = field(default=None)

    @classmethod
    def submit(
        cls,
        *,
        application_id: str,
        driver_id: str,
        documents: tuple[DocumentImage, ...],
        selfie_ref: str,
        face_consent: bool,
        now: datetime,
    ) -> Self:
        """Tạo hồ sơ mới ở trạng thái SUBMITTED; mọi quy tắc đầu vào kiểm ở đây."""
        _require_id(APPLICATION_ID, application_id, "mã hồ sơ")
        _require_id(DRIVER_ID, driver_id, "mã tài xế")
        if now.tzinfo is None:
            raise DomainError("submitted_at phải có múi giờ (UTC)")
        kinds = sorted(d.kind for d in documents)
        if kinds != [DocumentKind.CCCD, DocumentKind.GPLX]:
            raise DomainError("một hồ sơ gồm đúng hai giấy tờ: một CCCD và một GPLX")
        if not isinstance(face_consent, bool):
            raise DomainError("face_consent phải là bool, không phải chuỗi hay số")
        if not selfie_ref.strip():
            raise DomainError("selfie_ref không được rỗng")
        by_kind = {d.kind: d for d in documents}
        return cls(
            application_id=application_id,
            driver_id=driver_id,
            documents=(by_kind[DocumentKind.CCCD], by_kind[DocumentKind.GPLX]),
            selfie_ref=selfie_ref,
            face_consent=face_consent,
            submitted_at=now,
        )

    @property
    def id_document(self) -> DocumentImage:
        """Giấy tờ có ảnh chân dung dùng để so khớp khuôn mặt: CCCD."""
        return self.documents[0]

    def record_ocr(self, results: tuple[OcrResult, ...]) -> None:
        if sorted(r.kind for r in results) != sorted(d.kind for d in self.documents):
            raise DomainError("kết quả OCR phải đủ một kết quả cho mỗi giấy tờ của hồ sơ")
        self.status = advance(self.status, KycStatus.OCR_DONE)
        self.ocr_results = results

    def record_face(self, match: FaceMatch) -> None:
        self.status = advance(self.status, KycStatus.FACE_CHECKED)
        self.face_match = match

    def record_risk(self, score: RiskScore) -> None:
        self.status = advance(self.status, KycStatus.RISK_SCORED)
        self.risk_score = score

    def send_to_review(self, reason: str) -> None:
        self.status = advance(self.status, KycStatus.NEEDS_REVIEW)
        self.note = reason

    def decide(self, *, reviewer_id: str, decision: KycStatus, reason: str, now: datetime) -> None:
        """Người duyệt quyết định: chỉ NEEDS_REVIEW nhận, chỉ tới approved hoặc rejected."""
        _require_id(REVIEWER_ID, reviewer_id, "mã người duyệt")
        if decision not in (KycStatus.APPROVED, KycStatus.REJECTED):
            raise DomainError(
                f"quyết định phải là approved hoặc rejected, không phải {decision.value}"
            )
        if not reason.strip():
            raise DomainError("quyết định phải có lý do")
        self.status = advance(self.status, decision)
        self.review = ReviewDecision(reviewer_id, decision, reason, now)
