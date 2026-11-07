"""Consumer tham chiếu của KycApplicationScored: một service khác (ví dụ analytics) đọc topic.

Nằm NGOÀI package `kyc_service` có chủ ý: consumer không dùng chung mã với producer, chỉ dùng
chung HỢP ĐỒNG (envelope và payload ghi trong `integration_events.py` và trong bài). Quy tắc đọc
giống bài 38: kiểm `version` TRƯỚC payload, bỏ qua trường lạ, trường tùy chọn vắng là `None`.
"""

import json
from dataclasses import dataclass
from datetime import datetime

EVENT_TYPE = "KycApplicationScored"
SUPPORTED_BY_OLD_CONSUMER = frozenset({1})  # bản chưa nâng cấp, chỉ hiểu version 1
SUPPORTED_BY_NEW_CONSUMER = frozenset({1, 2})


@dataclass(frozen=True)
class Scored:
    event_id: str
    version: int
    occurred_at: datetime
    application_id: str
    driver_id: str
    risk_score: float
    review_reasons: tuple[str, ...]
    face_score: float | None = None  # có từ version 2
    ocr_min_confidence: float | None = None  # có từ version 2


def _number(value: object, name: str) -> float:
    if isinstance(value, bool) or not isinstance(value, int | float):
        raise TypeError(f"{name} phải là số")
    return float(value)


class EnvelopeError(ValueError):
    """Thông điệp không đọc được; `reason` là mã ổn định cho nhật ký và metric."""

    def __init__(self, reason: str, detail: str = "") -> None:
        super().__init__(f"{reason}: {detail}" if detail else reason)
        self.reason = reason


def decode(raw: bytes, *, supported: frozenset[int]) -> Scored:
    """Bytes -> Scored; mọi lỗi định dạng thành EnvelopeError, không để lọt exception lạ."""
    try:
        env = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise EnvelopeError("invalid_json", str(exc)) from None
    if not isinstance(env, dict):
        raise EnvelopeError("invalid_envelope", "không phải object")
    if env.get("event_type") != EVENT_TYPE:
        raise EnvelopeError("wrong_event_type", repr(env.get("event_type")))
    version = env.get("version")
    # Kiểm kiểu trước: True == 1 lọt qua `in`, list/dict không băm được.
    if type(version) is not int or version not in supported:
        # Kiểm TRƯỚC khi đọc payload: payload của version lạ có thể đã đổi nghĩa.
        raise EnvelopeError("unsupported_version", repr(version))
    try:
        p = env["payload"]
        reasons = p["review_reasons"]
        if not isinstance(reasons, list) or not all(isinstance(r, str) for r in reasons):
            raise TypeError("review_reasons phải là danh sách chuỗi")
        for name in ("application_id", "driver_id"):
            if not isinstance(p[name], str):
                raise TypeError(f"{name} phải là chuỗi")
        for name in ("face_score", "ocr_min_confidence"):  # tùy chọn: vắng hoặc null là None
            if p.get(name) is not None and not isinstance(p[name], int | float):
                raise TypeError(f"{name} phải là số")
        return Scored(
            event_id=env["event_id"],
            version=version,
            occurred_at=datetime.fromisoformat(env["occurred_at"]),
            application_id=p["application_id"],
            driver_id=p["driver_id"],
            risk_score=_number(p["risk_score"], "risk_score"),
            review_reasons=tuple(reasons),
            face_score=p.get("face_score"),  # v1 không có -> None; trường lạ khác bị bỏ qua
            ocr_min_confidence=p.get("ocr_min_confidence"),
        )
    except (KeyError, TypeError, ValueError) as exc:
        raise EnvelopeError("invalid_payload", f"{type(exc).__name__}: {exc}") from None
