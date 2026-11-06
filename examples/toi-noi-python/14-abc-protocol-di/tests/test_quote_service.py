"""Test QuoteService bằng đối tượng giả tự viết: không cần mock framework, không monkeypatch."""

from datetime import UTC, datetime, timedelta
from decimal import Decimal
from pathlib import Path

import pytest

from fare import STANDARD, FarePolicy, ServiceKind
from ports import Clock, FixedClock
from providers import DictPolicyProvider, JsonFilePolicyProvider, PolicyProviderBase
from service import QuoteService

D = Decimal
START = datetime(2026, 1, 1, 8, 0, tzinfo=UTC)
POLICY_5000 = FarePolicy(price_per_extra_km=D("5000"))


class RecordingPolicyProvider:
    """Đối tượng giả: KHÔNG kế thừa gì, chỉ có `policy_for` đúng chữ ký, vậy là thỏa Protocol."""

    def __init__(self, policy: FarePolicy) -> None:
        self.policy = policy
        self.cities: list[str] = []

    def policy_for(self, city: str) -> FarePolicy:
        self.cities.append(city)
        return self.policy


def test_quote_uses_policy_from_provider_and_city() -> None:
    provider = RecordingPolicyProvider(POLICY_5000)
    service = QuoteService(provider, FixedClock(START))

    quote = service.quote("Đà Nẵng", D("5"))

    assert provider.cities == ["Đà Nẵng"]
    assert quote.fare.amount == D("27000")  # 12.000 + 3 km x 5.000


def test_service_kind_is_accepted_and_does_not_change_price() -> None:
    service = QuoteService(DictPolicyProvider({}), FixedClock(START))

    fares = {kind: service.quote("Hà Nội", D("5"), service=kind).fare.amount for kind in ServiceKind}

    assert fares == {ServiceKind.RIDE: D("25500"), ServiceKind.FOOD: D("25500"), ServiceKind.PARCEL: D("25500")}


def test_quote_is_valid_until_exactly_expires_at() -> None:
    service = QuoteService(DictPolicyProvider({}), FixedClock(START))
    quote = service.quote("Hà Nội", D("5"))

    assert quote.is_valid(START + timedelta(minutes=4, seconds=59))
    assert not quote.is_valid(START + timedelta(minutes=5))
    with pytest.raises(ValueError, match="aware"):
        quote.is_valid(datetime(2026, 1, 1, 8, 0))


def test_expires_at_comes_from_injected_clock() -> None:
    service = QuoteService(DictPolicyProvider({}), FixedClock(START))

    quote = service.quote("Hà Nội", D("5"))

    assert quote.expires_at == START + timedelta(minutes=5)


@pytest.mark.parametrize(
    ("elapsed", "expired"),
    [
        (timedelta(minutes=4, seconds=59), False),
        (timedelta(minutes=5), True),
        (timedelta(minutes=6), True),
    ],
    ids=["4m59s", "5m00s", "6m"],
)
def test_quote_expiry_without_waiting(elapsed: timedelta, expired: bool) -> None:
    clock = FixedClock(START)
    service = QuoteService(DictPolicyProvider({}), clock)
    quote = service.quote("Hà Nội", D("5"))

    clock.advance(elapsed)

    assert service.is_expired(quote) is expired


def test_unknown_city_falls_back_to_default_policy() -> None:
    provider = DictPolicyProvider({"Đà Nẵng": POLICY_5000})

    assert provider.policy_for("Cần Thơ") == STANDARD
    assert provider.policy_for("Đà Nẵng") == POLICY_5000


def test_json_file_provider_reads_policies(tmp_path: Path) -> None:
    path = tmp_path / "policies.json"
    path.write_text(
        '{"default": {"base_fare": "12000", "included_km": "2", "price_per_extra_km": "4500"},'
        ' "cities": {"Đà Nẵng": {"base_fare": "12000", "included_km": "2", "price_per_extra_km": "5000"}}}',
        encoding="utf-8",
    )

    provider = JsonFilePolicyProvider(path)

    assert provider.policy_for("Đà Nẵng") == POLICY_5000
    assert provider.policy_for("Nha Trang") == STANDARD


def test_abstract_base_cannot_be_instantiated() -> None:
    with pytest.raises(TypeError, match="abstract"):
        PolicyProviderBase()  # type: ignore[abstract]


def test_fixed_clock_requires_aware_datetime() -> None:
    with pytest.raises(ValueError, match="aware"):
        FixedClock(datetime(2026, 1, 1, 8, 0))


def test_clock_is_checked_by_name_only_at_runtime() -> None:
    assert isinstance(FixedClock(START), Clock)
    assert not isinstance(object(), Clock)
