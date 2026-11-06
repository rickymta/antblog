"""Lỗi ở ranh giới application."""


class ApplicationNotFound(LookupError):
    def __init__(self, application_id: str) -> None:
        super().__init__(f"không có hồ sơ {application_id}")
        self.application_id = application_id


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
