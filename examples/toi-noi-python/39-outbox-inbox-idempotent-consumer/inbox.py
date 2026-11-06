"""Phía nhận: consumer ghi inbox và hiệu ứng trong CÙNG MỘT transaction."""

import json
from datetime import datetime
from decimal import Decimal

from sqlalchemy import Engine
from sqlalchemy.dialects.postgresql import insert
from sqlalchemy.orm import Session

from models import InboxEntry, PayoutLine, ScoreRequest

PROCESSED = "processed"
DUPLICATE = "duplicate"


def handle_trip_completed(
    engine: Engine,
    consumer: str,
    raw: bytes,
    *,
    now: datetime,
    use_inbox: bool = True,
    effect: str = "payout",
    fail_after_effect: bool = False,
) -> str:
    """Xử lý một thông điệp TripCompleted. Trả `processed` hoặc `duplicate`.

    Với inbox: INSERT inbox ... ON CONFLICT DO NOTHING. Không chèn được nghĩa là (consumer, event_id)
    đã có, đã xử lý, bỏ qua. Chèn được thì ghi hiệu ứng, cả hai cùng commit hoặc cùng rollback.
    """
    envelope = json.loads(raw)
    payload = envelope["payload"]
    with Session(engine) as session, session.begin():
        if use_inbox:
            inserted = session.execute(
                insert(InboxEntry)
                .values(consumer=consumer, event_id=envelope["event_id"], received_at=now)
                .on_conflict_do_nothing()
                .returning(InboxEntry.event_id)
            ).scalar_one_or_none()
            if inserted is None:
                return DUPLICATE
        if effect == "payout":  # service thanh toán: ghi tiền chuyến cho tài xế
            session.add(
                PayoutLine(
                    consumer=consumer,
                    driver_id=payload["driver_id"],
                    trip_id=payload["trip_id"],
                    amount=Decimal(payload["fare"]),
                    event_id=envelope["event_id"],
                )
            )
        else:  # service fraud-scoring: xếp một yêu cầu chấm điểm cho chuyến
            session.add(ScoreRequest(consumer=consumer, trip_id=payload["trip_id"], event_id=envelope["event_id"]))
        if fail_after_effect:
            raise RuntimeError("mô phỏng lỗi sau khi ghi hiệu ứng, trước khi commit")
    return PROCESSED
