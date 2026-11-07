"""Lỗi ở ranh giới application."""

from dataclasses import dataclass


class ApplicationNotFound(LookupError):
    def __init__(self, application_id: str) -> None:
        super().__init__(f"không có hồ sơ {application_id}")
        self.application_id = application_id


class ConcurrencyConflict(Exception):
    """Hồ sơ đã bị giao dịch khác sửa giữa lúc ta nạp và lúc ta ghi (optimistic concurrency)."""

    def __init__(self, application_id: str) -> None:
        super().__init__(f"hồ sơ {application_id} vừa bị cập nhật bởi giao dịch khác")
        self.application_id = application_id


class UnitOfWorkNotStarted(RuntimeError):
    """Dùng kho ngoài khối `with uow.begin()`: không có transaction nào đang mở."""


class NestedUnitOfWork(RuntimeError):
    """`begin()` lồng nhau: mỗi command chỉ có một transaction."""


class PortFailure(Exception):
    """Một service ngoài (qua port) không cho kết quả dùng được. Adapter dịch lỗi của nó về đây."""

    reason = "failure"

    def __init__(self, service: str, detail: str = "") -> None:
        super().__init__(f"{service}: {self.reason} {detail}".strip())
        self.service = service


class ServiceUnavailable(PortFailure):
    reason = "unavailable"


class InvalidServiceResponse(PortFailure):
    reason = "invalid_response"


@dataclass(frozen=True)
class FieldError:
    field: str
    code: str
    message: str


class ValidationFailed(Exception):
    """Command/query sai hình dạng. Không chứa giá trị đầu vào: có thể là dữ liệu nhạy cảm."""

    def __init__(self, errors: list[FieldError]) -> None:
        super().__init__(f"{len(errors)} lỗi trong thông điệp")
        self.errors = errors
