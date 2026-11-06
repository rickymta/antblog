from datetime import UTC, datetime, timedelta
from decimal import Decimal

import pytest

from toinoi_pricing import STANDARD, FarePolicy, ServiceKind, calculate_fare, make_quote

D = Decimal

# Bảng ca biên của bài 03 và 10; giá kỳ vọng tính tay từ đặc tả.
BOUNDARIES = [
    (D("0"), D("12000")),
    (D("1.999"), D("12000")),
    (D("2"), D("12000")),
    (D("2.001"), D("12005")),
    (D("2.499"), D("14246")),
    (D("2.5"), D("14250")),
    (D("5"), D("25500")),
    (D("100"), D("453000")),
]


@pytest.mark.parametrize("service", list(ServiceKind))
@pytest.mark.parametrize(
    ("distance", "expected"),
    BOUNDARIES,
    ids=[f"{distance}km" for distance, _ in BOUNDARIES],
)
def test_fare_at_boundaries(distance: Decimal, expected: Decimal, service: ServiceKind) -> None:
    assert calculate_fare(distance, service=service) == expected


@pytest.mark.parametrize("bad", [D("-0.001"), D("100.001"), D("NaN"), D("Infinity")])
def test_out_of_range_is_rejected(bad: Decimal) -> None:
    with pytest.raises(ValueError, match="ngoài miền"):
        calculate_fare(bad)


def test_replacement_policy_changes_the_price() -> None:
    policy = FarePolicy(price_per_extra_km=D("5000"))
    assert calculate_fare(D("4"), policy) == D("22000")
    assert calculate_fare(D("4")) == D("21000")  # chính sách chuẩn: 12.000 + 2 x 4.500


def test_make_quote_expires_after_ttl() -> None:
    now = datetime(2026, 11, 6, 3, 0, tzinfo=UTC)
    quote = make_quote(D("2.5"), now=now)
    assert quote.fare.amount == D("14250")
    assert quote.expires_at == now + timedelta(minutes=5)
    assert quote.is_valid(now)


def test_standard_policy_values() -> None:
    assert FarePolicy(D("12000"), D("2"), D("4500")) == STANDARD


def test_unknown_service_is_rejected() -> None:
    with pytest.raises(ValueError, match="bike"):
        calculate_fare(D("1"), service="bike")  # type: ignore[arg-type]


def test_make_quote_rejects_out_of_range_distance() -> None:
    with pytest.raises(ValueError, match="ngoài miền"):
        make_quote(D("100.001"), now=datetime(2026, 11, 6, 3, 0, tzinfo=UTC))
