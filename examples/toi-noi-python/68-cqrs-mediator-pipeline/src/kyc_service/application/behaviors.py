"""Pipeline behavior: việc cắt ngang (log, validate, transaction) viết một lần, không lặp trong
từng handler. Mỗi behavior là hàm `(message, next) -> kết quả`; không gọi `next` là chặn lại."""

from collections.abc import Callable
from dataclasses import asdict
from typing import Any

from pydantic import TypeAdapter, ValidationError

from kyc_service.application.errors import FieldError, ValidationFailed
from kyc_service.application.mediator import Behavior, Next
from kyc_service.application.messages import Command
from kyc_service.application.ports import TransactionManager


def logging_behavior(log: Callable[[str], None]) -> Behavior:
    """Ghi vào, ra, hoặc lỗi của mọi thông điệp. Ghi TÊN thông điệp, không ghi nội dung (PII)."""

    def behavior(message: Any, next_step: Next) -> Any:
        name = type(message).__name__
        log(f"> {name}")
        try:
            result = next_step(message)
        except Exception as exc:
            log(f"! {name} {type(exc).__name__}")
            raise
        log(f"< {name} ok")
        return result

    return behavior


def validation_behavior(
    adapter_factory: Callable[[type], TypeAdapter[Any]] = TypeAdapter,
) -> Behavior:
    """Kiểm hình dạng thông điệp theo chú thích kiểu của dataclass, gom MỌI lỗi một lượt.

    Handler nhận bản ĐÃ VALIDATE (đã cắt khoảng trắng, đã ép về đúng kiểu), không phải thông
    điệp gốc của người gọi. Chỉ kiểm hình dạng; quy tắc nghiệp vụ như «hai giấy tờ khác loại»
    vẫn nằm ở domain. `adapter_factory` để test đếm số lần dựng bộ kiểm.
    """
    adapters: dict[type, TypeAdapter[Any]] = {}

    def behavior(message: Any, next_step: Next) -> Any:
        message_type = type(message)
        adapter = adapters.get(message_type)
        if adapter is None:  # dựng đúng một lần cho mỗi kiểu thông điệp
            adapter = adapters[message_type] = adapter_factory(message_type)
        try:
            validated = adapter.validate_python(asdict(message))
        except ValidationError as exc:
            # Không chép `input` của pydantic vào lỗi: có thể là dữ liệu nhạy cảm.
            raise ValidationFailed(
                [
                    FieldError(".".join(str(p) for p in e["loc"]), e["type"], e["msg"])
                    for e in exc.errors()
                ]
            ) from None
        return next_step(validated)

    return behavior


def transaction_behavior(transactions: TransactionManager) -> Behavior:
    """Chỉ command mới mở transaction; query đi thẳng."""

    def behavior(message: Any, next_step: Next) -> Any:
        if not isinstance(message, Command):
            return next_step(message)
        with transactions.begin():
            return next_step(message)

    return behavior
