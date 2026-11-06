"""Strategy: cùng một việc «đưa khoảng cách, nhận cước», nhiều cách thực hiện thay được cho nhau."""

from collections.abc import Callable
from decimal import Decimal
from typing import Protocol, runtime_checkable

from domain import STANDARD, FarePolicy, ServiceKind, calculate_fare

# 1) Strategy là một HÀM: nhận km, trả cước. Không cần interface hay lớp cha.
type FareStrategy = Callable[[Decimal], Decimal]


def standard(service: ServiceKind = ServiceKind.RIDE, policy: FarePolicy = STANDARD) -> FareStrategy:
    """Closure giữ `service` và `policy`; trả về hàm strategy.

    `service` được chuyển tiếp cho calculate_fare; hiện tại nó chưa làm đổi giá.
    """

    def strategy(distance_km: Decimal) -> Decimal:
        return calculate_fare(distance_km, policy, service=service)

    return strategy


def audited(inner: FareStrategy, log: list[str]) -> FareStrategy:
    """Hàm bậc cao: bọc strategy khác, ghi một dòng mỗi lần tính. Giá không đổi."""

    def strategy(distance_km: Decimal) -> Decimal:
        fare = inner(distance_km)
        log.append(f"{distance_km} km -> {fare} VND")
        return fare

    return strategy


class CachedFare:
    """Strategy dạng LỚP vì có trạng thái riêng (bộ nhớ đệm, bộ đếm trúng cache).

    `__call__` làm instance dùng được ở mọi chỗ cần `FareStrategy`.
    """

    def __init__(self, inner: FareStrategy) -> None:
        self._inner = inner
        self._cache: dict[Decimal, Decimal] = {}
        self.hits = 0

    def __call__(self, distance_km: Decimal) -> Decimal:
        if distance_km in self._cache:
            self.hits += 1
        else:
            self._cache[distance_km] = self._inner(distance_km)
        return self._cache[distance_km]


# 2) Khi strategy có nhiều thành viên đi cùng nhau, dùng Protocol (cấu trúc, không cần kế thừa).
@runtime_checkable
class FareCalculator(Protocol):
    label: str

    def calculate(self, distance_km: Decimal) -> Decimal: ...


class PolicyCalculator:
    """Không kế thừa FareCalculator; chỉ cần có `label` và `calculate`."""

    def __init__(self, label: str, policy: FarePolicy = STANDARD) -> None:
        self.label = label
        self._policy = policy

    def calculate(self, distance_km: Decimal) -> Decimal:
        return calculate_fare(distance_km, self._policy)


def as_strategy(calculator: FareCalculator) -> FareStrategy:
    """Method gắn với instance cũng là một callable: hai dạng Strategy nối được với nhau."""
    return calculator.calculate


# 3) Chọn strategy theo dịch vụ: một dict thay cho chuỗi if/elif.
def build_strategies(audit_log: list[str]) -> dict[ServiceKind, FareStrategy]:
    return {
        ServiceKind.RIDE: standard(ServiceKind.RIDE),
        ServiceKind.FOOD: audited(standard(ServiceKind.FOOD), audit_log),
        ServiceKind.PARCEL: CachedFare(standard(ServiceKind.PARCEL)),
    }


def quote_for(
    strategies: dict[ServiceKind, FareStrategy], service: ServiceKind, distance_km: Decimal
) -> Decimal:
    return strategies[service](distance_km)
