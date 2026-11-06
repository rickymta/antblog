"""Đọc document qua các cổng có ghi nhật ký: ghi audit TRƯỚC khi trả dữ liệu, và «đóng cửa khi audit hỏng»."""

from dataclasses import dataclass
from datetime import datetime
from typing import Any

from pymongo import DESCENDING
from pymongo.asynchronous.database import AsyncDatabase

from .audit import REVEAL_MIN_REASON, Actor, AuditSink
from .documents import DOCS, RAW, find_by_number, normalize_to_v3
from .pii import ID_FIELD, PiiKey


class RevealDenied(PermissionError):
    """Người gọi không có scope kyc:reveal hoặc không nêu lý do đủ dài."""


@dataclass(frozen=True)
class View:
    document: dict[str, Any]  # luôn là hình dạng v3, số giấy tờ đã che
    revealed: dict[str, str] | None  # số giấy tờ rõ khi reveal thành công
    raw_expired: bool  # xin reveal nhưng bản gốc đã hết hạn lưu


async def view_document(
    db: AsyncDatabase[dict[str, Any]], audit: AuditSink, key: PiiKey, *, kyc_id: str, document_type: str,
    actor: Actor, reveal: bool = False, reason: str | None = None, now: datetime | None = None,
) -> View:
    latest = await db[DOCS].find_one({"kyc_id": kyc_id, "document_type": document_type},
                                     sort=[("created_at", DESCENDING), ("_id", DESCENDING)])
    if latest is None:
        raise LookupError(f"không có kết quả OCR cho {kyc_id} ({document_type})")

    if reveal and ("kyc:reveal" not in actor.scopes or not reason or len(reason.strip()) < REVEAL_MIN_REASON):
        await audit.record(actor=actor, action="reveal_denied", kyc_id=kyc_id, document_type=document_type,
                           reason=reason, now=now)
        raise RevealDenied("cần scope kyc:reveal và lý do tối thiểu " + str(REVEAL_MIN_REASON) + " ký tự")

    # Ghi audit TRƯỚC. Nếu ghi lỗi, ngoại lệ nổi lên và không có byte dữ liệu nào được trả: fail closed.
    await audit.record(actor=actor, action="reveal" if reveal else "view", kyc_id=kyc_id,
                       document_type=document_type, reason=reason, now=now)
    document = normalize_to_v3(latest, key)
    if not reveal:
        return View(document, None, False)
    name = ID_FIELD[document_type]
    if latest["schema_version"] < 3:  # document v2 chưa migrate: số rõ còn ngay trong document, không có raw để hết hạn
        return View(document, {name: latest["fields"][name]}, False)
    raw = await db[RAW].find_one({"_id": document["raw_id"]})
    plain = (raw or {}).get("plain", {})
    if name not in plain:  # raw đã hết hạn, hoặc OCR vốn không đọc ra số
        return View(document, None, raw is None)
    return View(document, {name: plain[name]}, False)


async def lookup_by_number(
    db: AsyncDatabase[dict[str, Any]], audit: AuditSink, keys: list[PiiKey], *, number: str, document_type: str,
    actor: Actor, now: datetime | None = None,
) -> list[dict[str, Any]]:
    """Đối chiếu theo số giấy tờ. Ai tra số nào cũng là truy cập dữ liệu cá nhân nên cũng ghi audit (hành động `lookup`,
    KHÔNG ghi số được tra) và cũng cần scope kyc:view; thiếu scope thì ghi `lookup_denied` rồi từ chối."""
    if "kyc:view" not in actor.scopes:
        await audit.record(actor=actor, action="lookup_denied", kyc_id="-", document_type=document_type, reason=None, now=now)
        raise PermissionError("cần scope kyc:view")
    await audit.record(actor=actor, action="lookup", kyc_id="-", document_type=document_type, reason=None, now=now)
    return await find_by_number(db, number, document_type, keys)
