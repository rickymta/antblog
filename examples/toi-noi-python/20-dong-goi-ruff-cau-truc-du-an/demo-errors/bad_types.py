from decimal import Decimal

from toinoi_pricing import calculate_fare


def quote_text() -> str:
    fare = calculate_fare(2.5)
    return fare


def first_digit(text: str | None) -> str:
    return text[0]
