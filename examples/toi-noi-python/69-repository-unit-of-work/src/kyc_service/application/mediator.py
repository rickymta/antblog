"""Mediator tự viết: tìm handler theo kiểu của thông điệp rồi chạy qua pipeline behavior.

Không có reflection, không quét package, không container: handler đăng ký tường minh một lần
khi khởi động (composition root).
"""

from collections.abc import Callable, Sequence
from typing import Any, Protocol

from kyc_service.application.messages import Message

type Next = Callable[[Any], Any]
"""Bước kế tiếp trong pipeline: nhận thông điệp, trả kết quả."""

type Behavior = Callable[[Any, Next], Any]
"""Một lớp bọc quanh handler: nhận thông điệp và `next`, quyết định có gọi `next` hay không."""


class Handler[M, R](Protocol):
    def handle(self, message: M) -> R: ...


class HandlerNotFound(LookupError):
    pass


class DuplicateHandler(ValueError):
    pass


def _wrap(behavior: Behavior, next_step: Next) -> Next:
    # Hàm riêng (không viết lambda ngay trong vòng for) để mỗi lớp bọc giữ đúng
    # `behavior` và `next_step` của nó; closure trong vòng lặp sẽ bắt biến cuối cùng.
    return lambda message: behavior(message, next_step)


class Mediator:
    def __init__(self, behaviors: Sequence[Behavior] = ()) -> None:
        self._behaviors = tuple(behaviors)
        self._pipelines: dict[type, Next] = {}

    def register[M: Message[Any], R](self, message_type: type[M], handler: Handler[M, R]) -> None:
        """Gắn đúng một handler cho một kiểu thông điệp và dựng sẵn pipeline của nó."""
        if message_type in self._pipelines:
            raise DuplicateHandler(f"{message_type.__name__} đã có handler")
        pipeline: Next = handler.handle
        # Duyệt từ trong ra ngoài: behavior đầu danh sách bọc ngoài cùng.
        for behavior in reversed(self._behaviors):
            pipeline = _wrap(behavior, pipeline)
        self._pipelines[message_type] = pipeline

    def send[R](self, message: Message[R]) -> R:
        """Gửi thông điệp; dispatch theo kiểu chính xác, không theo kiểu cha."""
        pipeline = self._pipelines.get(type(message))
        if pipeline is None:
            raise HandlerNotFound(f"chưa đăng ký handler cho {type(message).__name__}")
        result: R = pipeline(message)
        return result
