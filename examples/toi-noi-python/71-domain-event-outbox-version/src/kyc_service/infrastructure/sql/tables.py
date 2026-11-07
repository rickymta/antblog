"""Bảng PostgreSQL của aggregate KycApplication (schema `lab71`). Chỉ infrastructure biết chúng.

Aggregate gồm một hàng gốc `kyc_applications` và các hàng con (`kyc_documents`, `kyc_ocr_results`).
Cả ba được nạp và ghi cùng nhau qua repository; không ai ngoài repository đọc hàng con.
"""

from datetime import datetime
from typing import Any

from sqlalchemy import (
    BigInteger,
    CheckConstraint,
    DateTime,
    Float,
    ForeignKey,
    Identity,
    Index,
    MetaData,
    Sequence,
    String,
    Text,
    text,
)
from sqlalchemy.dialects.postgresql import JSONB
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship

from kyc_service.domain.kyc import KycStatus
from kyc_service.infrastructure.sql.settings import SCHEMA


class Base(DeclarativeBase):
    # Mọi bảng và sequence nằm trong schema lab71 để dọn được bằng DROP SCHEMA ... CASCADE.
    metadata = MetaData(schema=SCHEMA)


# Sinh mã hồ sơ KYC-0001, KYC-0002... Sequence KHÔNG rollback: transaction lỗi để lại lỗ.
APPLICATION_SEQ = Sequence("kyc_application_seq", metadata=Base.metadata)

_STATUSES = ", ".join(f"'{s.value}'" for s in KycStatus)


class ApplicationRow(Base):
    __tablename__ = "kyc_applications"
    __table_args__ = (
        CheckConstraint(f"status IN ({_STATUSES})", name="ck_status_known"),
        # Chốt thứ hai cho quy tắc của domain: có người duyệt khi và chỉ khi đã approved/rejected.
        CheckConstraint(
            "(status IN ('approved', 'rejected')) = (reviewed_by IS NOT NULL)",
            name="ck_decision_has_reviewer",
        ),
        Index("ix_kyc_applications_status", "status", "application_id"),
    )

    application_id: Mapped[str] = mapped_column(String(16), primary_key=True)  # KYC-0001
    driver_id: Mapped[str] = mapped_column(String(16), index=True)  # DR-001
    status: Mapped[str] = mapped_column(String(16))
    selfie_ref: Mapped[str] = mapped_column(String(256))
    face_consent: Mapped[bool]
    submitted_at: Mapped[datetime] = mapped_column(DateTime(timezone=True))
    note: Mapped[str | None] = mapped_column(Text)
    queue_reasons: Mapped[list[str]] = mapped_column(JSONB, default=list)  # mã ReviewReason
    face_score: Mapped[float | None] = mapped_column(Float(53))
    risk_value: Mapped[float | None] = mapped_column(Float(53))
    risk_reasons: Mapped[list[str]] = mapped_column(JSONB, default=list)
    reviewed_by: Mapped[str | None] = mapped_column(String(16))  # RV-001
    review_decision: Mapped[str | None] = mapped_column(String(16))
    review_reason: Mapped[str | None] = mapped_column(Text)
    reviewed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    # Phiên bản cho optimistic concurrency: mỗi UPDATE thêm 1 và kèm `WHERE version = <đã nạp>`.
    version: Mapped[int] = mapped_column()

    __mapper_args__ = {"version_id_col": version}  # noqa: RUF012 - cách khai báo của SQLAlchemy

    # lazy="raise": chạm vào hàng con mà quên nạp (selectinload) thì lỗi ngay, không âm thầm
    # phát N câu SELECT. Mọi truy vấn phải nói rõ nó cần hàng con nào.
    documents: Mapped[list[DocumentRow]] = relationship(
        cascade="all, delete-orphan", lazy="raise", order_by="DocumentRow.kind"
    )
    ocr_results: Mapped[list[OcrRow]] = relationship(
        cascade="all, delete-orphan", lazy="raise", order_by="OcrRow.kind"
    )


class DocumentRow(Base):
    __tablename__ = "kyc_documents"

    application_id: Mapped[str] = mapped_column(
        ForeignKey(f"{SCHEMA}.kyc_applications.application_id"), primary_key=True
    )
    kind: Mapped[str] = mapped_column(String(8), primary_key=True)  # cccd | gplx
    image_ref: Mapped[str] = mapped_column(String(256))


class OcrRow(Base):
    __tablename__ = "kyc_ocr_results"

    application_id: Mapped[str] = mapped_column(
        ForeignKey(f"{SCHEMA}.kyc_applications.application_id"), primary_key=True
    )
    kind: Mapped[str] = mapped_column(String(8), primary_key=True)
    # Số giấy tờ đã che (`099*******83`) trước khi vào đây; xem repository.
    fields: Mapped[dict[str, Any]] = mapped_column(JSONB)
    confidence: Mapped[float] = mapped_column(Float(53))
    # Khóa nối sang document OCR của bài 47 (cùng application_id và kind).
    model_version: Mapped[str] = mapped_column(String(64))
    image_sha256: Mapped[str | None] = mapped_column(String(64))


class OutboxRow(Base):
    """Ý định phát một sự kiện, ghi cùng transaction với thay đổi nghiệp vụ (xem bài 39).

    `seq` cho thứ tự phát; `event_id` là danh tính ổn định của sự kiện (consumer khử trùng bằng
    nó); `published_at` NULL nghĩa là chưa phát; `lease_until` là «thuê» của một dispatcher.
    """

    __tablename__ = "outbox"
    __table_args__ = (
        Index("ix_outbox_pending", "seq", postgresql_where=text("published_at IS NULL")),
    )

    seq: Mapped[int] = mapped_column(BigInteger, Identity(), primary_key=True)
    event_id: Mapped[str] = mapped_column(String(36), unique=True)
    topic: Mapped[str] = mapped_column(String(128))
    message_key: Mapped[str] = mapped_column(String(64))
    envelope: Mapped[dict[str, Any]] = mapped_column(JSONB)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True))
    published_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    lease_until: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    attempts: Mapped[int] = mapped_column(default=0)
