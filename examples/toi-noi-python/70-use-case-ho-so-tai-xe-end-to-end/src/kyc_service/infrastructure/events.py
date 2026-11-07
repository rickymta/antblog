"""Adapter nhận domain event. Lab 70 chỉ ghi lại trong bộ nhớ; bài 71 thay bằng outbox."""

from collections.abc import Sequence

from kyc_service.domain.events import DomainEvent


class RecordingEventPublisher:
    """Ghi các event đã nhận để test và demo đọc lại. Không bền: tắt tiến trình là mất."""

    def __init__(self) -> None:
        self.received: list[DomainEvent] = []

    def publish(self, events: Sequence[DomainEvent]) -> None:
        self.received.extend(events)
