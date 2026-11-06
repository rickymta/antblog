"""Port: những gì use case cần từ thế giới bên ngoài, do application khai báo.

Là `Protocol` nên adapter ở infrastructure khớp theo cấu trúc, không cần import hay kế thừa gì.
"""

from contextlib import AbstractContextManager
from datetime import datetime
from typing import Protocol

from kyc_service.application.views import ApplicationView
from kyc_service.domain.kyc import (
    DocumentImage,
    FaceMatch,
    KycApplication,
    KycStatus,
    OcrResult,
    RiskScore,
)


class OcrPort(Protocol):
    """kyc-ocr: đọc các trường từ một ảnh giấy tờ."""

    def read(self, document: DocumentImage) -> OcrResult: ...


class FaceMatchPort(Protocol):
    """kyc-face: so ảnh chân dung trên giấy tờ với selfie."""

    def compare(self, document: DocumentImage, selfie_ref: str) -> FaceMatch: ...


class RiskScorePort(Protocol):
    """fraud-scoring: chấm điểm rủi ro cho cả hồ sơ."""

    def score(self, application: KycApplication) -> RiskScore: ...


class KycRepository(Protocol):
    """Lưu và nạp aggregate KycApplication."""

    def next_id(self) -> str: ...

    def add(self, application: KycApplication) -> None: ...

    def get(self, application_id: str) -> KycApplication | None: ...

    def save(self, application: KycApplication) -> None: ...

    def list_by_status(self, status: KycStatus) -> list[KycApplication]: ...


class Clock(Protocol):
    def now(self) -> datetime:
        """Giờ hiện tại, aware, UTC."""
        ...


class UnitOfWork(Protocol):
    """Một transaction và các kho dùng chung nó. Thay `TransactionManager` của bài 68.

    `begin()` mở transaction; `applications` chỉ dùng được BÊN TRONG khối `with`, vì nó là kho
    gắn với transaction đang mở. Dùng ngoài khối thì ném `UnitOfWorkNotStarted`.
    """

    @property
    def applications(self) -> KycRepository: ...

    def begin(self) -> AbstractContextManager[None]:
        """Thoát khối `with` bình thường là commit; thoát bằng exception là rollback."""
        ...


class ApplicationReadModel(Protocol):
    """Phía đọc của CQRS: trả thẳng view, không nạp aggregate."""

    def get(self, application_id: str) -> ApplicationView | None: ...

    def review_queue(self) -> tuple[ApplicationView, ...]:
        """Hồ sơ đang needs_review, theo mã hồ sơ."""
        ...
