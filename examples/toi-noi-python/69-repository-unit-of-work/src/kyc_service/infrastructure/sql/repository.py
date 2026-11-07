"""KycRepository trên SQLAlchemy Session: kho theo aggregate, không phải kho tổng quát.

Chỉ có những thao tác mà use case cần (`next_id`, `add`, `get`, `save`, `list_by_status`). Không có
`find(**filters)` hay `query()` trả ra Session: mỗi truy vấn thêm vào là một quyết định có chủ ý.
Aggregate là dataclass thuần của domain; hàng ORM chỉ sống trong module này.
"""

from collections.abc import Mapping
from typing import Any

from sqlalchemy import select
from sqlalchemy.exc import IntegrityError
from sqlalchemy.orm import Session, selectinload
from sqlalchemy.orm.attributes import flag_modified
from sqlalchemy.orm.exc import StaleDataError

from kyc_service.application.errors import ConcurrencyConflict
from kyc_service.domain.kyc import (
    PII_FIELDS,
    DocumentImage,
    DocumentKind,
    FaceMatch,
    KycApplication,
    KycStatus,
    OcrResult,
    ReviewDecision,
    RiskScore,
    mask_id_number,
)
from kyc_service.infrastructure.sql.tables import (
    APPLICATION_SEQ,
    ApplicationRow,
    DocumentRow,
    OcrRow,
)

# Trường OCR là số định danh: vào cơ sở dữ liệu chỉ dạng đã che, bản đầy đủ không rời bộ nhớ.
UNIQUE_VIOLATION = "23505"

LOAD_ALL = (
    selectinload(ApplicationRow.documents),
    selectinload(ApplicationRow.ocr_results),
)


def protect(fields: Mapping[str, str]) -> dict[str, Any]:
    """Chốt thứ hai: `OcrResult` đã che số; đây chặn mọi đường khác ghi số rõ xuống bảng."""
    return {k: mask_id_number(v) if k in PII_FIELDS else v for k, v in fields.items()}


def to_domain(row: ApplicationRow) -> KycApplication:
    """Dựng aggregate từ hàng. Đi qua constructor nên bất biến của value object vẫn được kiểm."""
    docs = {d.kind: d for d in row.documents}
    cccd, gplx = docs[DocumentKind.CCCD.value], docs[DocumentKind.GPLX.value]
    review = None
    if row.reviewed_by is not None:
        assert row.review_decision is not None
        assert row.review_reason is not None
        assert row.reviewed_at is not None
        review = ReviewDecision(
            row.reviewed_by, KycStatus(row.review_decision), row.review_reason, row.reviewed_at
        )
    return KycApplication(
        application_id=row.application_id,
        driver_id=row.driver_id,
        documents=(
            DocumentImage(DocumentKind.CCCD, cccd.image_ref),
            DocumentImage(DocumentKind.GPLX, gplx.image_ref),
        ),
        selfie_ref=row.selfie_ref,
        face_consent=row.face_consent,
        submitted_at=row.submitted_at,
        status=KycStatus(row.status),
        ocr_results=tuple(
            OcrResult(DocumentKind(o.kind), o.fields, o.confidence, o.model_version, o.image_sha256)
            for o in row.ocr_results
        ),
        face_match=FaceMatch(row.face_score) if row.face_score is not None else None,
        risk_score=(
            RiskScore(row.risk_value, tuple(row.risk_reasons))
            if row.risk_value is not None
            else None
        ),
        note=row.note,
        review=review,
    )


def apply_to_row(row: ApplicationRow, app: KycApplication) -> None:
    """Chép trạng thái aggregate lên hàng. SQLAlchemy chỉ phát UPDATE cho cột thật sự đổi."""
    row.driver_id = app.driver_id
    row.status = app.status.value
    row.selfie_ref = app.selfie_ref
    row.face_consent = app.face_consent
    row.submitted_at = app.submitted_at
    row.note = app.note
    row.face_score = app.face_match.score if app.face_match else None
    row.risk_value = app.risk_score.value if app.risk_score else None
    row.risk_reasons = list(app.risk_score.reasons) if app.risk_score else []
    review = app.review
    row.reviewed_by = review.reviewer_id if review else None
    row.review_decision = review.decision.value if review else None
    row.review_reason = review.reason if review else None
    row.reviewed_at = review.decided_at if review else None

    wanted = {
        r.kind.value: (protect(r.fields), r.confidence, r.model_version, r.image_sha256)
        for r in app.ocr_results
    }
    current = {
        o.kind: (o.fields, o.confidence, o.model_version, o.image_sha256) for o in row.ocr_results
    }
    if wanted != current:  # chỉ thay hàng con khi có thay đổi thật
        row.ocr_results = [
            OcrRow(
                application_id=app.application_id,
                kind=k,
                fields=f,
                confidence=c,
                model_version=m,
                image_sha256=h,
            )
            for k, (f, c, m, h) in sorted(wanted.items())
        ]


class SqlAlchemyKycRepository:
    """Một kho cho MỘT transaction: nó giữ tham chiếu mạnh tới mọi hàng đã nạp.

    Identity map của Session chỉ giữ tham chiếu YẾU: hàng không ai cầm có thể bị thu gom ngay sau
    `get`, và lần `get` kế tiếp nạp lại version MỚI từ cơ sở dữ liệu, xóa mất version mà ta đã
    đọc. Khi đó `WHERE version = ...` luôn khớp và mất kiểm tra xung đột. Giữ hàng trong `_rows`
    bảo đảm `save` ghi với đúng version đã nạp.
    """

    def __init__(self, session: Session) -> None:
        self._session = session
        self._rows: dict[str, ApplicationRow] = {}

    def next_id(self) -> str:
        # nextval không thuộc transaction: rollback không trả lại số đã cấp.
        return f"KYC-{self._session.scalar(APPLICATION_SEQ.next_value()):04d}"

    def add(self, application: KycApplication) -> None:
        row = ApplicationRow(
            application_id=application.application_id,
            documents=[
                DocumentRow(
                    application_id=application.application_id,
                    kind=d.kind.value,
                    image_ref=d.image_ref,
                )
                for d in application.documents
            ],
            ocr_results=[],
            # Các cột còn lại do apply_to_row điền; khai báo tối thiểu để INSERT hợp lệ.
            driver_id=application.driver_id,
            status=application.status.value,
            selfie_ref=application.selfie_ref,
            face_consent=application.face_consent,
            submitted_at=application.submitted_at,
        )
        apply_to_row(row, application)
        self._session.add(row)
        self._rows[application.application_id] = row
        try:
            self._session.flush()  # đẩy INSERT ngay để lỗi ràng buộc hiện ở đây, không ở lúc commit
        except IntegrityError as exc:
            if getattr(exc.orig, "sqlstate", None) == UNIQUE_VIOLATION:
                raise ValueError(f"hồ sơ {application.application_id} đã tồn tại") from exc
            raise

    def get(self, application_id: str) -> KycApplication | None:
        row = self._rows.get(application_id)
        if row is None:
            row = self._session.get(ApplicationRow, application_id, options=LOAD_ALL)
            if row is None:
                return None
            self._rows[application_id] = row
        return to_domain(row)

    def save(self, application: KycApplication) -> None:
        row = self._rows.get(application.application_id)
        if row is None:
            raise ValueError(f"hồ sơ {application.application_id} chưa được add hoặc get")
        apply_to_row(row, application)
        # Mọi lần ghi aggregate đều tăng version, kể cả khi chỉ hàng con đổi: aggregate là một
        # đơn vị nhất quán, hai người cùng sửa nó phải xung đột.
        flag_modified(row, "status")
        try:
            self._session.flush()
        except StaleDataError as exc:
            raise ConcurrencyConflict(application.application_id) from exc

    def list_by_status(self, status: KycStatus) -> list[KycApplication]:
        rows = self._session.scalars(
            select(ApplicationRow)
            .where(ApplicationRow.status == status.value)
            .order_by(ApplicationRow.application_id)
            .options(*LOAD_ALL)
        ).all()
        for row in rows:
            self._rows.setdefault(row.application_id, row)
        return [to_domain(self._rows[r.application_id]) for r in rows]
