"""Fixture dùng chung cho mọi file test trong thư mục tests/."""

from decimal import Decimal

import pytest

from fare import STANDARD, FarePolicy


@pytest.fixture(scope="session")
def standard_policy() -> FarePolicy:
    # FarePolicy là NamedTuple bất biến nên dùng chung cả phiên test là an toàn.
    return STANDARD


@pytest.fixture
def policy_5000() -> FarePolicy:
    """Chính sách thay thế: giá mỗi km vượt 5.000 thay vì 4.500."""
    return FarePolicy(price_per_extra_km=Decimal("5000"))
