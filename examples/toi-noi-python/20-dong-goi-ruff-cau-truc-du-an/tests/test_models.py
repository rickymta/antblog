from dataclasses import FrozenInstanceError, replace
from datetime import UTC, datetime, timedelta
from decimal import Decimal

import pytest

from toinoi_pricing import ALLOWED_TRANSITIONS, Money, Quote, TripStatus, can_transition

D = Decimal
NOW = datetime(2026, 11, 6, 3, 0, tzinfo=UTC)


def test_money_add_same_currency() -> None:
    assert Money(D("12000")) + Money(D("3000")) == Money(D("15000"))


def test_money_add_rejects_other_currency() -> None:
    with pytest.raises(ValueError, match="không cộng được"):
        Money(D("1")) + Money(D("1"), "USD")


def test_money_add_rejects_non_money() -> None:
    with pytest.raises(TypeError):
        Money(D("1")) + 1


def test_money_rejects_float_amount() -> None:
    with pytest.raises(TypeError, match="amount phải là Decimal"):
        Money(2.5)  # type: ignore[arg-type]


@pytest.mark.parametrize(
    ("amount", "message"),
    [
        (D("NaN"), "hữu hạn"),
        (D("Infinity"), "hữu hạn"),
        (D("-1"), "không được âm"),
        (D("0.5"), "đồng lẻ"),
    ],
    ids=["nan", "inf", "am", "dong-le"],
)
def test_money_rejects_invalid_vnd_amount(amount: Decimal, message: str) -> None:
    with pytest.raises(ValueError, match=message):
        Money(amount)


@pytest.mark.parametrize(
    "currency", ["vnd", "VN", "VNDD", "ĐÔN"], ids=["thuong", "ngan", "dai", "non-ascii"]
)
def test_money_rejects_invalid_currency(currency: str) -> None:
    with pytest.raises(ValueError, match="currency phải gồm 3 chữ in hoa"):
        Money(D("1"), currency)


def test_money_normalizes_vnd_and_formats() -> None:
    money = Money(D("14250.0"))
    assert money.amount.as_tuple().exponent == 0  # 14250.0 -> 14250
    assert str(money) == "14.250 VND"
    assert str(Money(D("1.25"), "USD")) == "1.25 USD"  # tiền khác VND giữ phần lẻ


def test_money_is_frozen_and_hashable() -> None:
    money = Money(D("12000"))
    with pytest.raises(FrozenInstanceError):
        money.amount = D("1")  # type: ignore[misc]
    assert {money, Money(D("12000"))} == {money}
    assert replace(money, currency="USD") != money


@pytest.mark.parametrize(
    ("offset", "valid"),
    [(timedelta(seconds=-1), True), (timedelta(0), False), (timedelta(seconds=1), False)],
    ids=["1s-truoc-han", "dung-han", "1s-sau-han"],
)
def test_quote_validity_boundary(offset: timedelta, valid: bool) -> None:
    quote = Quote(D("5"), Money(D("25500")), NOW)
    assert quote.is_valid(NOW + offset) is valid


def test_quote_rejects_naive_datetimes() -> None:
    with pytest.raises(ValueError, match="aware"):
        Quote(D("5"), Money(D("25500")), datetime(2026, 11, 6, 3, 0))  # noqa: DTZ001
    quote = Quote(D("5"), Money(D("25500")), NOW)
    with pytest.raises(ValueError, match="aware"):
        quote.is_valid(datetime(2026, 11, 6, 3, 0))  # noqa: DTZ001


@pytest.mark.parametrize("distance", [D("-0.001"), D("100.001")], ids=["am", "qua-100"])
def test_quote_rejects_distance_out_of_range(distance: Decimal) -> None:
    with pytest.raises(ValueError, match="ngoài miền"):
        Quote(distance, Money(D("12000")), NOW)


def test_transition_table_has_ten_allowed_pairs() -> None:
    pairs = [(a, b) for a, targets in ALLOWED_TRANSITIONS.items() for b in targets]
    assert len(pairs) == 10  # 6 bước thuận + 4 lần hủy
    assert can_transition(TripStatus.ARRIVING, TripStatus.CANCELLED)
    assert not can_transition(TripStatus.IN_TRANSIT, TripStatus.CANCELLED)
    assert not can_transition(TripStatus.COMPLETED, TripStatus.REQUESTED)


def test_status_values_are_lowercase_strings() -> None:
    assert TripStatus.PICKED_UP.value == "picked_up"
    assert str(TripStatus.PICKED_UP) == "picked_up"
