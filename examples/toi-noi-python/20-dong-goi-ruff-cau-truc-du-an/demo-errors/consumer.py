"""Mã của người DÙNG gói (không thuộc gói). Dùng để thử py.typed: xem README, mục «Thử py.typed»."""

from decimal import Decimal

from toinoi_pricing import calculate_fare

fare: str = calculate_fare(Decimal("2.5"))
