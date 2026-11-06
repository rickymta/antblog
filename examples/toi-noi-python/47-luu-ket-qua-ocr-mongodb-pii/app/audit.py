"""Nhật ký truy cập: ai xem document nào, lúc nào, với lý do gì. Không chứa số giấy tờ (kể cả dạng che).

`reason` là chữ tự do do người gọi nhập, nên có thể chứa số giấy tờ nếu họ gõ vào; lab không lọc (chỉ yêu cầu độ dài),
production cần chặn hoặc che mẫu số trong lý do."""

from dataclasses import dataclass, field
from datetime import UTC, datetime, timedelta
from typing import Any, Protocol

from pymongo.asynchronous.collection import AsyncCollection

AUDIT_RETENTION = timedelta(days=400)  # giá trị lab; dài hơn raw vì audit phải sống lâu hơn dữ liệu nó mô tả
REVEAL_MIN_REASON = 10  # lý do xem số rõ ngắn hơn ngần này ký tự bị từ chối


@dataclass(frozen=True)
class Actor:
    """Người gọi, sau khi bài 34 đã xác thực và cấp scope. `DP-001`: điều phối viên (người duyệt hồ sơ)."""

    actor_id: str
    scopes: frozenset[str] = field(default_factory=frozenset)


class AuditSink(Protocol):
    async def record(self, *, actor: Actor, action: str, kyc_id: str, document_type: str,
                     reason: str | None, now: datetime | None = None) -> None: ...


class MongoAuditSink:
    def __init__(self, collection: AsyncCollection[dict[str, Any]]) -> None:
        self.collection = collection

    async def record(self, *, actor: Actor, action: str, kyc_id: str, document_type: str,
                     reason: str | None, now: datetime | None = None) -> None:
        now = now or datetime.now(UTC)
        at = now.replace(microsecond=now.microsecond // 1000 * 1000)  # MongoDB chỉ giữ mili giây
        await self.collection.insert_one({
            "at": at, "expire_at": at + AUDIT_RETENTION, "actor": actor.actor_id, "action": action,
            "kyc_id": kyc_id, "document_type": document_type, "reason": reason})
