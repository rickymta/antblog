"""Hợp đồng sự kiện gửi ra ngoài (integration event): tách khỏi domain event.

Domain event là chuyện nội bộ, đổi tự do khi refactor. Integration event là hợp đồng với service
khác (C#, analytics): tên trường, kiểu và `version` chỉ đổi theo quy tắc ở đây. Hàm
`scored_envelope` là chỗ DUY NHẤT biết cả hai phía.

Quy ước dây (chốt cho cả loạt bài Python): JSON, tên trường `snake_case`, envelope
`{event_id, event_type, occurred_at, version, payload}` như bài 38, thời gian ISO 8601 UTC.
Consumer C# đọc bằng `JsonNamingPolicy.SnakeCaseLower`.

Phiên bản (`version` trong envelope, số nguyên chỉ tăng):
- 1: application_id, driver_id, risk_score, review_reasons.
- 2: thêm face_score và ocr_min_confidence (cả hai tùy chọn). Thêm trường tùy chọn là tương thích.
Đổi tên, xóa trường, hoặc đổi nghĩa là KHÔNG tương thích: phải mở topic mới (`...v2`).
"""

from typing import Any

from kyc_service.domain.events import DomainEvent, KycApplicationScored

EVENT_TYPE = "KycApplicationScored"
TOPIC = "kyc.application-scored.v1"  # hậu tố là thế hệ tương thích của topic, không phải `version`
SUPPORTED_VERSIONS = (1, 2)


class UnmappedDomainEvent(TypeError):
    """Domain event chưa có ánh xạ sang integration event: lỗi lập trình, không được nuốt."""


def scored_envelope(
    event: KycApplicationScored, *, event_id: str, version: int = 2
) -> dict[str, Any]:
    """Domain event -> envelope. `version` chọn hình dạng payload (producer đang ở hợp đồng nào)."""
    if version not in SUPPORTED_VERSIONS:
        raise ValueError(f"version {version} chưa được định nghĩa")
    payload: dict[str, Any] = {
        "application_id": event.application_id,
        "driver_id": event.driver_id,
        "risk_score": event.risk_score,
        "review_reasons": [r.value for r in event.review_reasons],
    }
    if version >= 2:
        payload["face_score"] = event.face_score
        payload["ocr_min_confidence"] = event.ocr_min_confidence
    return {
        "event_id": event_id,
        "event_type": EVENT_TYPE,
        "occurred_at": event.occurred_at.isoformat(),
        "version": version,
        "payload": payload,
    }


def to_envelope(event: DomainEvent, *, event_id: str, version: int) -> dict[str, Any]:
    if isinstance(event, KycApplicationScored):
        return scored_envelope(event, event_id=event_id, version=version)
    raise UnmappedDomainEvent(f"chưa có integration event cho {type(event).__name__}")


def partition_key(event: DomainEvent) -> str:
    """Khóa Kafka: các thông điệp cùng khóa vào cùng partition.

    Kafka chỉ giữ thứ tự GỬI trong một partition, không giữ thứ tự TẠO sự kiện: hai dispatcher
    cùng chạy có thể gửi ngược, và dispatcher chết trước khi đánh dấu có thể gửi lại (1, 2, 1, 2).
    Muốn thứ tự thì chỉ một dispatcher cho mỗi khóa, hoặc consumer tự kiểm (version, `occurred_at`).
    """
    if isinstance(event, KycApplicationScored):
        return event.application_id
    raise UnmappedDomainEvent(f"chưa có khóa cho {type(event).__name__}")
