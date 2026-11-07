"""Lỗi của miền nghiệp vụ. Kế thừa ValueError để mã gọi đã biết bắt ValueError vẫn chạy."""


class DomainError(ValueError):
    """Quy tắc nghiệp vụ bị vi phạm."""


class InvalidStatusTransition(DomainError):
    """Hồ sơ KYC không được đi từ trạng thái này sang trạng thái kia."""
