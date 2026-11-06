"""Outbox: ghi sự kiện vào bảng `outbox` BÊN TRONG transaction của command.

`OutboxEventPublisher` là adapter của port `EventPublisher`. Nó dùng Session của Unit of Work đang
mở, nên dòng outbox commit hoặc rollback cùng hồ sơ: không còn «sự kiện ma» (hồ sơ rollback mà
sự kiện đã đi) lẫn sự kiện mất (hồ sơ commit mà sự kiện không bao giờ đi).
"""

from collections.abc import Sequence

from kyc_service.application.integration_events import partition_key, to_envelope
from kyc_service.application.ports import Clock, IdGenerator
from kyc_service.domain.events import DomainEvent
from kyc_service.infrastructure.sql.tables import OutboxRow
from kyc_service.infrastructure.sql.unit_of_work import SqlAlchemyUnitOfWork


class OutboxEventPublisher:
    def __init__(
        self,
        uow: SqlAlchemyUnitOfWork,
        *,
        topic: str,
        emit_version: int,
        ids: IdGenerator,
        clock: Clock,
    ) -> None:
        self._uow = uow
        self._topic = topic
        self._emit_version = emit_version
        self._ids = ids
        self._clock = clock

    def publish(self, events: Sequence[DomainEvent]) -> None:
        session = self._uow.session  # ném UnitOfWorkNotStarted nếu gọi ngoài transaction
        for event in events:
            event_id = self._ids.new_id()  # đặt id một lần, lưu cùng dòng outbox
            session.add(
                OutboxRow(
                    event_id=event_id,
                    topic=self._topic,
                    message_key=partition_key(event),
                    envelope=to_envelope(event, event_id=event_id, version=self._emit_version),
                    created_at=self._clock.now(),
                )
            )
        session.flush()
