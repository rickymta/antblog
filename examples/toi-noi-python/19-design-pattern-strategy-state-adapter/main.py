"""Bài 19: Strategy, State, Adapter. Chạy: main.py (demo) hoặc main.py --verify."""

import sys
from collections.abc import Callable
from decimal import Decimal

from adapter import (
    MapVietAdapter,
    MapVietClient,
    Point,
    RoutingService,
    RoutingUnavailable,
    meters_to_km,
    quote_trip,
)
from domain import ServiceKind, TripStatus, calculate_fare
from state import ALLOWED_TRANSITIONS, STATES, TableTrip, Trip
from strategy import (
    CachedFare,
    FareCalculator,
    PolicyCalculator,
    as_strategy,
    audited,
    build_strategies,
    quote_for,
    standard,
)

D = Decimal

# Bảng ca biên của bài 03 và 10: (km, cước kỳ vọng).
GRID = [
    ("0", "12000"), ("1.999", "12000"), ("2", "12000"), ("2.001", "12005"),
    ("2.499", "14246"), ("2.5", "14250"), ("5", "25500"), ("100", "453000"),
]  # fmt: skip

# Tọa độ minh họa (lat, lng) và quãng đường SDK trả về, đơn vị mét.
ORIGIN: Point = (10.7769, 106.7009)
NEAR: Point = (10.7800, 106.7050)
MID: Point = (10.8000, 106.7200)
FAR: Point = (11.9404, 108.4583)
ROUTES: dict[tuple[Point, Point], float] = {
    (ORIGIN, NEAR): 2001.0,
    (ORIGIN, MID): 5230.0,
    (ORIGIN, FAR): 150000.0,
}


class FixedRouting:
    """Đối tượng giả cho test: không kế thừa gì, chỉ có đúng một method của RoutingService."""

    def __init__(self, km: str) -> None:
        self._km = Decimal(km)

    def distance_km(self, origin: Point, destination: Point) -> Decimal:
        return self._km


def run_trip(trip: Trip) -> list[str]:
    """Đi hết vòng đời thuận, trả danh sách status từng bước."""
    seen = [str(trip.status)]
    while trip.status is not TripStatus.COMPLETED:
        trip.advance()
        seen.append(str(trip.status))
    return seen


def reachable_by_classes(status: TripStatus) -> frozenset[TripStatus]:
    """Từ một trạng thái, các trạng thái đạt được bằng advance() hoặc cancel() của lớp trạng thái."""
    reach: set[TripStatus] = set()
    trip = Trip("TN-0001", driver_id="DR-001")
    state = STATES[status]
    for action in (state.advance, state.cancel):
        try:
            reach.add(action(trip))
        except ValueError:
            pass
    return frozenset(reach)


def raises(exc_type: type[Exception], fn: Callable[..., object], *args: object, contains: str = "") -> bool:
    """True nếu gọi fn(*args) ném exc_type có thông báo chứa `contains`."""
    try:
        fn(*args)
    except exc_type as exc:
        return contains in str(exc)
    return False


def advance_n(trip: Trip, times: int) -> Trip:
    for _ in range(times):
        trip.advance()
    return trip


def demo() -> None:
    print("== Strategy: ba dịch vụ, cùng cách tính, khác cách thực hiện ==")
    log: list[str] = []
    strategies = build_strategies(log)
    for service in ServiceKind:
        print(f"{service:7} 5 km -> {quote_for(strategies, service, D('5'))} VND")
    quote_for(strategies, ServiceKind.FOOD, D("2.5"))
    print(f"nhật ký của food: {log}")
    cached = CachedFare(standard())
    cached(D("5")), cached(D("5")), cached(D("2.5"))
    print(f"CachedFare sau 3 lần gọi (2 quãng đường khác nhau): hits={cached.hits}")
    calc = PolicyCalculator("cuoc-chuan")
    print(f"PolicyCalculator khớp Protocol: {isinstance(calc, FareCalculator)} ({calc.label})")

    print("== State: một chuyến đi hết vòng đời ==")
    trip = Trip("TN-0001")
    print(f"{trip.status}: {trip.message}")
    trip.advance()
    print(f"{trip.status}: {trip.message}")
    try:
        trip.advance()
    except ValueError as exc:
        print(f"searching -> accepted khi chưa có tài xế: {exc}")
    trip.driver_id = "DR-001"
    advance_n(trip, 1)
    print(f"{trip.status}: {trip.message}")
    advance_n(trip, 1)
    print(f"{trip.status}: {trip.message}")
    advance_n(trip, 2)
    try:
        trip.cancel()
    except ValueError as exc:
        print(f"hủy khi {trip.status}: {exc}")
    pairs = sum(len(v) for v in ALLOWED_TRANSITIONS.values())
    print(f"số cặp (từ, tới) được phép trong bảng chuyển: {pairs}")

    print("== Adapter: SDK trả mét, ta cần km Decimal ==")
    adapter = MapVietAdapter(MapVietClient(ROUTES))
    for dest in (NEAR, MID):
        km = adapter.distance_km(ORIGIN, dest)
        print(f"{km} km -> {quote_trip(adapter, ORIGIN, dest)} VND")
    try:
        adapter.distance_km(ORIGIN, (0.0, 0.0))
    except RoutingUnavailable as exc:
        print(f"RoutingUnavailable: {exc}")
    print(f"Decimal(2000.1)      = {Decimal(2000.1)}")
    print(f"Decimal('2000.1')    = {Decimal('2000.1')}")
    print(f"meters_to_km(2000.1) = {meters_to_km(2000.1)}")


def verify() -> None:
    log: list[str] = []
    strategies = build_strategies(log)
    checks: list[tuple[str, bool]] = []

    # --- Strategy ---
    for service in ServiceKind:
        ok = all(quote_for(strategies, service, D(km)) == D(price) for km, price in GRID)
        checks.append((f"strategy {service}: 8 ca biên cùng bảng giá", ok))
    expected_log = [f"{D(km)} km -> {D(price)} VND" for km, price in GRID]
    checks.append(("food ghi một dòng nhật ký mỗi lần tính", log == expected_log))
    cached = CachedFare(standard())
    cached(D("5")), cached(D("5")), cached(D("2.5"))
    checks.append(("CachedFare: 3 lần gọi, 2 quãng đường khác nhau -> hits == 1", cached.hits == 1))
    calc = PolicyCalculator("cuoc-chuan")
    checks.append(("PolicyCalculator khớp Protocol (không kế thừa)", isinstance(calc, FareCalculator)))
    checks.append(("as_strategy(calc) tính như standard()", as_strategy(calc)(D("5")) == standard()(D("5"))))
    checks.append(("strategy từ chối km ngoài miền",
                   raises(ValueError, strategies[ServiceKind.RIDE], D("100.001"), contains="ngoài miền")))
    side: list[str] = []
    checks.append(("audited() không đổi giá", audited(standard(), side)(D("5")) == D("25500") and len(side) == 1))

    # --- State ---
    expected_path = ["requested", "searching", "accepted", "arriving", "picked_up", "in_transit", "completed"]
    done = Trip("TN-0001", driver_id="DR-001")
    checks.append(("vòng đời thuận 7 trạng thái", run_trip(done) == expected_path))
    checks.append(("completed là cuối: advance() bị từ chối",
                   raises(ValueError, done.advance, contains="trạng thái cuối")))
    pairs_ok = all(reachable_by_classes(s) == ALLOWED_TRANSITIONS[s] for s in TripStatus)
    checks.append(("lớp trạng thái và bảng chuyển cho cùng tập chuyển (64 cặp)", pairs_ok))
    checks.append(("bảng chuyển có 10 cặp (6 thuận + 4 hủy)",
                   sum(len(v) for v in ALLOWED_TRANSITIONS.values()) == 10))
    no_driver = advance_n(Trip("TN-0002"), 1)
    checks.append(("searching -> accepted cần tài xế (lớp)",
                   raises(ValueError, no_driver.advance, contains="chưa có tài xế")))
    table = TableTrip("TN-0002", status=TripStatus.SEARCHING)
    checks.append(("searching -> accepted cần tài xế (bảng)",
                   raises(ValueError, table.move_to, TripStatus.ACCEPTED, contains="chưa có tài xế")))
    in_transit = advance_n(Trip("TN-0003", driver_id="DR-001"), 5)
    checks.append(("không hủy được khi in_transit",
                   raises(ValueError, in_transit.cancel, contains="không hủy được")))
    accepted = advance_n(Trip("TN-0004", driver_id="DR-001"), 2)
    accepted.cancel()
    checks.append(("hủy được khi accepted", accepted.status is TripStatus.CANCELLED))
    accepted_msg = advance_n(Trip("TN-0005", driver_id="DR-009"), 2).message
    checks.append(("thông báo theo trạng thái", accepted_msg == "Tài xế DR-009 đã nhận chuyến"))

    # --- Adapter ---
    adapter = MapVietAdapter(MapVietClient(ROUTES))
    checks.append(("adapter: 5230.0 m -> 5,23 km", adapter.distance_km(ORIGIN, MID) == D("5.23")))
    checks.append(("quote_trip qua adapter: 5,23 km -> 26535", quote_trip(adapter, ORIGIN, MID) == D("26535")))
    checks.append(("adapter: 2001.0 m -> 2,001 km -> 12005", quote_trip(adapter, ORIGIN, NEAR) == D("12005")))
    checks.append(("lỗi SDK được dịch thành RoutingUnavailable",
                   raises(RoutingUnavailable, adapter.distance_km, ORIGIN, (0.0, 0.0), contains="mã nhà cung cấp 404")))
    checks.append(("Decimal(float) lộ sai số, Decimal(str) thì không",
                   Decimal(2000.1) != D("2000.1") and meters_to_km(2000.1) == D("2.0001")))
    checks.append(("đối tượng giả khớp RoutingService, không cần mock",
                   quote_trip(FixedRouting("2.5"), ORIGIN, MID) == calculate_fare(D("2.5"))))
    checks.append(("150 km từ SDK vẫn bị miền 0..100 từ chối",
                   raises(ValueError, quote_trip, adapter, ORIGIN, FAR, contains="ngoài miền")))
    # Kiểu tĩnh: cả hai đều gán được cho RoutingService (mypy kiểm khi chạy mypy --strict).
    services: list[RoutingService] = [adapter, FixedRouting("1")]
    checks.append(("hai đối tượng cùng là RoutingService", len(services) == 2))

    failed = [name for name, ok in checks if not ok]
    for name in failed:
        print(f"SAI: {name}")
    if failed:
        print(f"FAIL {len(checks) - len(failed)}/{len(checks)}")
        sys.exit(1)
    print(f"OK {len(checks)}/{len(checks)}")


if __name__ == "__main__":
    verify() if "--verify" in sys.argv[1:] else demo()
