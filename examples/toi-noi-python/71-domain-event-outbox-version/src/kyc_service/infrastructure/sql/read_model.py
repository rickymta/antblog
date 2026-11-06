"""Phía đọc trên PostgreSQL: SELECT thẳng ra view, không dựng aggregate.

Mỗi lần đọc dùng một Session ngắn riêng, không đi qua Unit of Work của command: query không mở
transaction ghi, và không dùng chung Session với command đang chạy.
"""

from sqlalchemy import select
from sqlalchemy.orm import Session, selectinload, sessionmaker

from kyc_service.application.views import NEXT_CHECK, ApplicationView, OcrView
from kyc_service.domain.kyc import KycStatus, mask_id_number
from kyc_service.infrastructure.sql.tables import ApplicationRow


def to_view(row: ApplicationRow) -> ApplicationView:
    return ApplicationView(
        application_id=row.application_id,
        driver_id=row.driver_id,
        status=row.status,
        ocr=tuple(
            OcrView(
                kind=o.kind,
                id_number_masked=(
                    mask_id_number(o.fields["id_number"]) if "id_number" in o.fields else None
                ),
                confidence=o.confidence,
            )
            for o in row.ocr_results
        ),
        face_score=row.face_score,
        risk_score=row.risk_value,
        risk_reasons=tuple(row.risk_reasons),
        queue_reasons=tuple(row.queue_reasons),
        note=row.note,
        reviewed_by=row.reviewed_by,
        pending_check=NEXT_CHECK.get(row.status),
    )


class SqlAlchemyReadModel:
    def __init__(self, session_factory: sessionmaker[Session]) -> None:
        self._factory = session_factory

    def get(self, application_id: str) -> ApplicationView | None:
        with self._factory() as session:
            row = session.get(
                ApplicationRow, application_id, options=[selectinload(ApplicationRow.ocr_results)]
            )
            return to_view(row) if row else None

    def review_queue(self) -> tuple[ApplicationView, ...]:
        with self._factory() as session:
            rows = session.scalars(
                select(ApplicationRow)
                .where(ApplicationRow.status == KycStatus.NEEDS_REVIEW.value)
                .order_by(ApplicationRow.application_id)
                .options(selectinload(ApplicationRow.ocr_results))  # không nạp documents
            ).all()
            return tuple(to_view(r) for r in rows)
