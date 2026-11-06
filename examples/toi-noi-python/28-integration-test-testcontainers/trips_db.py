"""Bài 28: lớp truy cập PostgreSQL của «dịch vụ chuyến» và kho document OCR trên MongoDB.

Luồng trạng thái rút gọn so với từ điển miền (8 trạng thái): lab chỉ cài requested -> searching ->
accepted; các bước sau (arriving, picked_up, in_transit, completed) test đặt thẳng bằng SQL.
Hai lớp này là thứ ta kiểm thử bằng database thật. Chúng cố ý nhỏ nhưng dùng đúng những
tính năng mà SQLite hay một kho trong bộ nhớ không có: kiểu numeric/timestamptz, CHECK,
chỉ mục unique có điều kiện (partial index) và FOR UPDATE SKIP LOCKED.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import UTC, datetime
from decimal import Decimal

import psycopg
from psycopg import errors as pg_errors
from psycopg.rows import dict_row
from pymongo.collection import Collection

# Chuyến đang chiếm tài xế: tài xế chỉ được ở một chuyến như vậy tại một thời điểm.
ACTIVE_STATUSES = ("accepted", "arriving", "picked_up", "in_transit")

# DDL chạy trong schema riêng của từng ca test (search_path trỏ tới schema đó),
# nên tên bảng không cần tiền tố schema.
DDL = """
CREATE TABLE trips (
    trip_id      text PRIMARY KEY CHECK (trip_id ~ '^TN-[0-9]{4}$'),
    customer_id  text NOT NULL,
    driver_id    text,
    service      text NOT NULL CHECK (service IN ('ride', 'food', 'parcel')),
    distance_km  numeric(6, 3) NOT NULL CHECK (distance_km BETWEEN 0 AND 100),
    fare_vnd     numeric(12, 0) NOT NULL CHECK (fare_vnd >= 0),
    status       text NOT NULL DEFAULT 'requested' CHECK (status IN (
                     'requested', 'searching', 'accepted', 'arriving',
                     'picked_up', 'in_transit', 'completed', 'cancelled')),
    created_at   timestamptz NOT NULL DEFAULT now()
);
-- Một tài xế chỉ có một chuyến đang chạy: ràng buộc nằm ở database, không chỉ ở code.
CREATE UNIQUE INDEX one_active_trip_per_driver
    ON trips (driver_id)
    WHERE driver_id IS NOT NULL
      AND status IN ('accepted', 'arriving', 'picked_up', 'in_transit');
"""


class DriverBusy(Exception):
    """Tài xế đang có một chuyến chưa xong."""


class InvalidTransition(Exception):
    """Chuyến không ở trạng thái xuất phát của bước chuyển (hoặc không tồn tại)."""


@dataclass(frozen=True)
class TripRow:
    trip_id: str
    customer_id: str
    driver_id: str | None
    service: str
    distance_km: Decimal
    fare_vnd: Decimal
    status: str
    created_at: datetime


class TripRepository:
    """Repository trên một kết nối psycopg 3 (autocommit; giao dịch mở bằng conn.transaction())."""

    def __init__(self, conn: psycopg.Connection) -> None:
        self._conn = conn

    def create(self, trip_id: str, customer_id: str, service: str,
               distance_km: Decimal, fare_vnd: Decimal) -> None:
        self._conn.execute(
            "INSERT INTO trips (trip_id, customer_id, service, distance_km, fare_vnd)"
            " VALUES (%s, %s, %s, %s, %s)",
            (trip_id, customer_id, service, distance_km, fare_vnd),
        )

    def get(self, trip_id: str) -> TripRow | None:
        with self._conn.cursor(row_factory=dict_row) as cur:
            cur.execute("SELECT * FROM trips WHERE trip_id = %s", (trip_id,))
            row = cur.fetchone()
        return TripRow(**row) if row else None

    def start_searching(self, trip_id: str) -> None:
        """requested -> searching. Sai trạng thái (hoặc không có chuyến) thì InvalidTransition."""
        self._transition(trip_id, "requested", "searching")

    def assign_driver(self, trip_id: str, driver_id: str) -> None:
        """searching -> accepted kèm tài xế.

        Điều kiện `status = 'searching'` nằm trong WHERE: chuyến đã có tài xế thì UPDATE không
        khớp dòng nào (rowcount 0) và ta ném InvalidTransition, không ghi đè im lặng.
        Chỉ mục unique có điều kiện chặn tài xế đang bận chuyến khác (DriverBusy).
        """
        try:
            self._transition(trip_id, "searching", "accepted", driver_id=driver_id)
        except pg_errors.UniqueViolation as exc:
            raise DriverBusy(f"{driver_id} đang có chuyến chưa xong") from exc

    def _transition(self, trip_id: str, from_status: str, to_status: str,
                    driver_id: str | None = None) -> None:
        cur = self._conn.execute(
            "UPDATE trips SET status = %s, driver_id = COALESCE(%s, driver_id)"
            " WHERE trip_id = %s AND status = %s",
            (to_status, driver_id, trip_id, from_status),
        )
        if cur.rowcount != 1:
            raise InvalidTransition(f"{trip_id}: không thể chuyển {from_status} -> {to_status}")

    def claim_next_searching(self, driver_id: str) -> str | None:
        """Nhận chuyến `searching` có mã nhỏ nhất mà không ai khác đang giữ khóa.

        FOR UPDATE SKIP LOCKED bỏ qua dòng đang bị giao dịch khác khóa, nên hai tài xế
        gọi cùng lúc nhận hai chuyến khác nhau thay vì chờ nhau hay nhận trùng.
        Phải gọi trong `with conn.transaction():` để khóa giữ đến khi commit.
        """
        with self._conn.cursor() as cur:
            cur.execute(
                "SELECT trip_id FROM trips WHERE status = 'searching'"
                " ORDER BY trip_id LIMIT 1 FOR UPDATE SKIP LOCKED"
            )
            row = cur.fetchone()
            if row is None:
                return None
            cur.execute(
                "UPDATE trips SET driver_id = %s, status = 'accepted' WHERE trip_id = %s",
                (driver_id, row[0]),
            )
            return row[0]


class OcrDocumentStore:
    """Kho document kết quả OCR trên một collection MongoDB (mỗi ca test một database riêng)."""

    def __init__(self, collection: Collection) -> None:
        self._col = collection
        # Mỗi hồ sơ KYC chỉ có một document cho mỗi loại giấy tờ.
        self._col.create_index([("kyc_id", 1), ("document_type", 1)], unique=True)

    def save(self, kyc_id: str, document_type: str, fields: dict, confidence: float) -> None:
        self._col.insert_one({
            "kyc_id": kyc_id,
            "document_type": document_type,
            "fields": fields,
            "confidence": confidence,
            "schema_version": 1,
            "created_at": datetime.now(UTC),
        })

    def find_by_id_number(self, id_number: str) -> list[dict]:
        return list(self._col.find({"fields.id_number": id_number}, {"_id": 0}))

    def get(self, kyc_id: str, document_type: str) -> dict | None:
        return self._col.find_one({"kyc_id": kyc_id, "document_type": document_type}, {"_id": 0})


__all__ = ["ACTIVE_STATUSES", "DDL", "DriverBusy", "InvalidTransition", "OcrDocumentStore",
           "TripRepository", "TripRow"]
