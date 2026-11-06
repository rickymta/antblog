"""Ba bảng của lab 25: trips (version thủ công), booking_requests (khóa idempotency), versioned_trips."""

from decimal import Decimal
from enum import StrEnum

from sqlalchemy import Enum, ForeignKey, MetaData, Numeric, Sequence, String, UniqueConstraint
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column

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
    """StrEnum lưu thành VARCHAR chứa giá trị ("picked_up"); không tạo ENUM native hay CHECK (xem lab 24)."""
    return Enum(
        enum_cls,
        native_enum=False,
        length=16,
        values_callable=lambda e: [member.value for member in e],
    )


class Base(DeclarativeBase):
    metadata = MetaData(schema=SCHEMA)
    type_annotation_map = {
        TripStatus: _enum_column(TripStatus),
        ServiceKind: _enum_column(ServiceKind),
    }


# Mã chuyến TN-0100, TN-0101... lấy từ sequence. Sequence không rollback: ca thua cuộc đua
# vẫn "tiêu" một số, nên mã chuyến có thể có khoảng trống. Mã không cần liên tục, chỉ cần duy nhất.
TRIP_SEQ = Sequence("trip_seq", start=100, metadata=Base.metadata)  # metadata để create_all tạo nó


class Trip(Base):
    __tablename__ = "trips"

    trip_id: Mapped[str] = mapped_column(String(16), primary_key=True)
    customer_id: Mapped[str] = mapped_column(String(16))
    driver_id: Mapped[str | None] = mapped_column(String(16))
    service: Mapped[ServiceKind] = mapped_column(default=ServiceKind.RIDE)
    status: Mapped[TripStatus] = mapped_column(default=TripStatus.REQUESTED)
    fare: Mapped[Decimal] = mapped_column(Numeric(12, 0))
    # Cột version cho optimistic concurrency: mọi UPDATE có điều kiện phải đi kèm version cũ.
    version: Mapped[int] = mapped_column(default=1, server_default="1")


class BookingRequest(Base):
    """Một ý định đặt chuyến, nhận diện bằng (customer_id, idempotency_key)."""

    __tablename__ = "booking_requests"
    # Ràng buộc UNIQUE là lớp quyết định cuối: hai request cùng khóa không cùng commit được.
    __table_args__ = (
        UniqueConstraint("customer_id", "idempotency_key", name="uq_booking_requests_customer_key"),
    )

    id: Mapped[int] = mapped_column(primary_key=True)  # PostgreSQL tự sinh (identity)
    customer_id: Mapped[str] = mapped_column(String(16))
    idempotency_key: Mapped[str] = mapped_column(String(64))
    service: Mapped[ServiceKind]  # cùng distance_km: nội dung ý định, để so khớp
    distance_km: Mapped[Decimal] = mapped_column(Numeric(7, 3))
    trip_id: Mapped[str] = mapped_column(ForeignKey(f"{SCHEMA}.trips.trip_id"))


class VersionedTrip(Base):
    """Bảng riêng để cho thấy ORM tự quản version nhờ `version_id_col`."""

    __tablename__ = "versioned_trips"

    trip_id: Mapped[str] = mapped_column(String(16), primary_key=True)
    driver_id: Mapped[str | None] = mapped_column(String(16))
    version: Mapped[int] = mapped_column(default=1)

    # ORM thêm "AND version = <cũ>" vào mọi UPDATE/DELETE và tự tăng version;
    # không khớp hàng nào thì ném StaleDataError.
    __mapper_args__ = {"version_id_col": version}
