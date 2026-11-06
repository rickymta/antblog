"""Các lỗi cố ý để mypy --strict bắt. Dòng nào kết thúc bằng chú thích «E:» (sau dấu thăng) PHẢI bị báo.

Đừng chạy file này: nó chỉ để mypy đọc.
"""

from ports import SystemClock
from providers import DictPolicyProvider, PolicyProviderBase
from service import QuoteService


class BadClock:
    """Cùng tên phương thức `now` nhưng khác chữ ký và kiểu trả về."""

    def now(self, tz: str) -> str:
        return tz


class WrongReturnProvider:
    """Có `policy_for` nhưng trả sai kiểu."""

    def policy_for(self, city: str) -> str:
        return city


class IncompleteProvider(PolicyProviderBase):
    """Quên cài phương thức trừu tượng `lookup`."""


a = QuoteService(object(), SystemClock())  # E: object không có policy_for
b = QuoteService(DictPolicyProvider({}), BadClock())  # E: BadClock không thỏa Clock
c = QuoteService(WrongReturnProvider(), SystemClock())  # E: policy_for trả str thay vì FarePolicy
d = PolicyProviderBase()  # E: không khởi tạo trực tiếp lớp trừu tượng
e = IncompleteProvider()  # E: thiếu lookup
