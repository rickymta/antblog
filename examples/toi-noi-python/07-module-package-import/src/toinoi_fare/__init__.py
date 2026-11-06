"""toinoi_fare: gói tính cước Tới Nơi. File này chạy một lần khi `import toinoi_fare`."""

from .models import DEFAULT_POLICY, FarePolicy, Quote
from .pricing import calculate_fare, quote

# __all__ khai báo bề mặt công khai của gói (cũng là thứ `from toinoi_fare import *` lấy).
__all__ = ["DEFAULT_POLICY", "FarePolicy", "Quote", "calculate_fare", "quote"]
