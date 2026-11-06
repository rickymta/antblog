"""Structured logging: mỗi dòng log là một object JSON, kèm request_id lấy từ contextvars."""

import json
import logging
import sys
from collections.abc import Callable
from contextvars import ContextVar
from datetime import UTC, datetime
from typing import TextIO

from app.settings import Settings

# "-" nghĩa là dòng log phát sinh ngoài một request (lúc khởi động, tắt service).
request_id_var: ContextVar[str] = ContextVar("request_id", default="-")

# Thuộc tính sẵn có của LogRecord: mọi thuộc tính KHÁC là trường do `extra={...}` thêm vào.
_STANDARD_ATTRS = set(logging.makeLogRecord({}).__dict__) | {"message", "asctime"}


class RequestIdFilter(logging.Filter):
    """Gắn request_id hiện hành vào mọi record đi qua handler, kể cả log của thư viện."""

    def filter(self, record: logging.LogRecord) -> bool:
        record.request_id = request_id_var.get()
        return True


class JsonFormatter(logging.Formatter):
    def __init__(self, service: str) -> None:
        super().__init__()
        self.service = service

    def format(self, record: logging.LogRecord) -> str:
        data: dict[str, object] = {
            "ts": datetime.fromtimestamp(record.created, UTC).isoformat(timespec="milliseconds"),
            "level": record.levelname,
            "logger": record.name,
            "service": self.service,
            "request_id": getattr(record, "request_id", "-"),
            "event": record.getMessage(),
        }
        for key, value in record.__dict__.items():
            if key not in _STANDARD_ATTRS and key != "request_id":
                data[key] = value
        if record.exc_info:
            data["exc"] = self.formatException(record.exc_info)
        # ensure_ascii=False: giữ nguyên tiếng Việt; default=str: Decimal, datetime không làm vỡ log.
        return json.dumps(data, ensure_ascii=False, default=str)


def configure_logging(settings: Settings, stream: TextIO | None = None) -> Callable[[], None]:
    """Gắn handler JSON vào root logger; trả về hàm gỡ để test và lifespan dọn sạch."""
    handler = logging.StreamHandler(stream or sys.stdout)
    handler.setFormatter(JsonFormatter(settings.service_name))
    handler.addFilter(RequestIdFilter())
    root = logging.getLogger()
    previous_level = root.level
    root.addHandler(handler)
    root.setLevel(settings.log_level)

    def restore() -> None:
        root.removeHandler(handler)
        root.setLevel(previous_level)

    return restore
