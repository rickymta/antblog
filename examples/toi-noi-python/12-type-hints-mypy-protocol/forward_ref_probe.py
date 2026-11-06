"""Thử tham chiếu tiến không dấu nháy: chạy bằng 3.14 và 3.13 để thấy khác biệt (không dùng trong --verify)."""

import sys


class Trip:
    def with_driver(self, driver: Driver) -> Trip:  # Driver khai báo SAU; Trip là lớp đang định nghĩa
        return self


class Driver: ...


print(f"Python {sys.version_info.major}.{sys.version_info.minor}: định nghĩa xong, không lỗi")
