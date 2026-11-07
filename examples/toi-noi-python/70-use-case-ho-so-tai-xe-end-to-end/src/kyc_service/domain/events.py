"""Domain event: điều đã xảy ra trong miền, đặt tên ở thì quá khứ, do aggregate tạo ra.

Domain event là chuyện nội bộ của service (tên, trường đổi tự do khi refactor). Hợp đồng gửi ra
service khác là integration event, tách riêng ở bài 71.
"""

from dataclasses import dataclass
from datetime import datetime

from kyc_service.domain.review import ReviewReason


@dataclass(frozen=True, kw_only=True)
class DomainEvent:
    occurred_at: datetime


@dataclass(frozen=True, kw_only=True)
class KycApplicationScored(DomainEvent):
    """Hồ sơ đã chấm điểm đủ (OCR, khuôn mặt, rủi ro) và vào hàng chờ duyệt.

    Không phải quyết định: `approved` và `rejected` chỉ đến từ người duyệt.
    """

    application_id: str
    driver_id: str
    risk_score: float
    face_score: float
    ocr_min_confidence: float
    review_reasons: tuple[ReviewReason, ...]
