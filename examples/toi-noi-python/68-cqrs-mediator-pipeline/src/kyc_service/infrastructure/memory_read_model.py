"""Phía đọc trong bộ nhớ. Lab dùng chung kho dữ liệu với phía ghi; việc tách nằm ở PORT.

Khi lên PostgreSQL (bài 69) adapter này đổi thành SELECT thẳng ra view, không dựng aggregate.
"""

from kyc_service.application.views import ApplicationView
from kyc_service.domain.kyc import KycStatus
from kyc_service.infrastructure.memory_repository import InMemoryKycRepository


class InMemoryReadModel:
    def __init__(self, repository: InMemoryKycRepository) -> None:
        self._repository = repository

    def get(self, application_id: str) -> ApplicationView | None:
        found = self._repository.get(application_id)
        return ApplicationView.of(found) if found else None

    def review_queue(self) -> tuple[ApplicationView, ...]:
        return tuple(
            ApplicationView.of(a)
            for a in self._repository.all_rows()
            if a.status is KycStatus.NEEDS_REVIEW
        )
