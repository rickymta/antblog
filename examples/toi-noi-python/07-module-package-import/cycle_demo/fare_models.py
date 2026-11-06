"""fare_models.py: import ngược fare_pricing -> tạo vòng. Cách sửa: tách phần dùng chung ra module thứ ba."""

from cycle_demo.fare_pricing import quote  # noqa: F401  (cố ý tạo vòng)


def money(amount: int) -> int:
    return amount
