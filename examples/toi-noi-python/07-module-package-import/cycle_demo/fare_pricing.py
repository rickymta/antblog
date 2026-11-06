"""fare_pricing.py: import fare_models, trong khi fare_models lại import ngược fare_pricing."""

from cycle_demo.fare_models import money


def quote() -> int:
    return money(12000)
