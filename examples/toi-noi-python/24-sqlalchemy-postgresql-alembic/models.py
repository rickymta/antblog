"""Mapping ORM 2.0 cho hai bảng của Tới Nơi: customers và trips (schema lab24)."""

from datetime import datetime
from decimal import Decimal
from enum import StrEnum

from sqlalchemy import DateTime, Enum, ForeignKey, Index, MetaData, Numeric, String, func
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship

from settings import SCHEMA


class ServiceKind(StrEnum):
    RIDE = "ride"
    FOOD = "food"
    PARCEL = "parcel"


class TripStatus(StrEnum):
    REQUESTED = "requested"
    SEARCHING = "searching"
    ACCEPTED = "accepted"
    ARRIVING = "arriving"
    PICKED_UP = "picked_up"
    IN_TRANSIT = "in_transit"
    COMPLETED = "completed"
    CANCELLED = "cancelled"


def _enum_column(enum_cls: type[StrEnum]) -> Enum:
    """Lưu StrEnum thành VARCHAR chứa *giá trị* ("picked_up"), không phải tên ("PICKED_UP").

    native_enum=False: không tạo kiểu ENUM của PostgreSQL (khó đổi bằng migration).
    """
    return Enum(
        enum_cls,
        native_enum=False,
        length=16,
        values_callable=lambda e: [member.value for member in e],
    )


class Base(DeclarativeBase):
    # Mọi bảng nằm trong schema lab24 để lab dọn được bằng DROP SCHEMA ... CASCADE.
    metadata = MetaData(schema=SCHEMA)
    # Cho phép viết Mapped[TripStatus] và Mapped[ServiceKind] thẳng trên annotation.
    type_annotation_map = {
        TripStatus: _enum_column(TripStatus),
        ServiceKind: _enum_column(ServiceKind),
    }


class Customer(Base):
    __tablename__ = "customers"

    customer_id: Mapped[str] = mapped_column(String(16), primary_key=True)  # CU-001
    full_name: Mapped[str] = mapped_column(String(100))

    # Quan hệ 1-n; mặc định tải lười (lazy="select"): chạm .trips mới phát SQL.
    trips: Mapped[list["Trip"]] = relationship(back_populates="customer")


class Trip(Base):
    __tablename__ = "trips"
    # Truy vấn "chuyến của một khách, mới nhất trước" dùng index này (thêm ở migration 0002).
    __table_args__ = (Index("ix_trips_customer_created", "customer_id", "created_at"),)

    trip_id: Mapped[str] = mapped_column(String(16), primary_key=True)  # TN-0001
    customer_id: Mapped[str] = mapped_column(ForeignKey(f"{SCHEMA}.customers.customer_id"))
    driver_id: Mapped[str | None] = mapped_column(String(16))  # NULL tới khi có người nhận
    service: Mapped[ServiceKind]
    distance_km: Mapped[Decimal] = mapped_column(Numeric(7, 3))
    fare: Mapped[Decimal] = mapped_column(Numeric(12, 0))  # VND, không có phần lẻ
    status: Mapped[TripStatus] = mapped_column(default=TripStatus.REQUESTED)
    # timestamptz: PostgreSQL lưu một thời điểm tuyệt đối; ta luôn ghi giờ UTC aware.
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now()
    )

    customer: Mapped[Customer] = relationship(back_populates="trips")
