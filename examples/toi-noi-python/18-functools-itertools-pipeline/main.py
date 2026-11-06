"""Bài 18: functools, itertools và pipeline generator.

Bài toán: một lô 10.000 chuyến giả (6 thành phố) cần được lọc chuyến tính cước được,
tính cước, rồi gom doanh thu theo thành phố, mà không dựng list trung gian cho từng bước.
Không dùng pandas: chỉ thư viện chuẩn.

Chạy:     uv run --project examples/toi-noi-python/18-functools-itertools-pipeline examples/toi-noi-python/18-functools-itertools-pipeline/main.py
Kiểm tra: thêm --verify ở cuối.
"""

import sys
import time
import tracemalloc
from collections import defaultdict
from collections.abc import Callable, Iterable, Iterator
from decimal import ROUND_HALF_UP, Decimal
from enum import StrEnum
from functools import cache, lru_cache, partial, reduce
from itertools import accumulate, batched, chain, groupby, islice
from operator import add, attrgetter
from random import Random
from typing import Any, NamedTuple

TRIPS = 10_000
BATCH = 1_000  # mỗi lô 1.000 chuyến: một lần ghi database trong thực tế
SEED = 20260618
MAX_KM = Decimal("100")

CITIES = ("Hà Nội", "TP.HCM", "Đà Nẵng", "Hải Phòng", "Cần Thơ", "Nha Trang")
CITY_EDGES = (25, 60, 72, 82, 90, 100)  # ngưỡng tích lũy (%): TP.HCM và Hà Nội đông nhất

# Giá trị lấy từ lần chạy thật với SEED trên; --verify so lại các con số này.
EXPECTED_STATUS = {"cancelled": 1_224, "completed": 8_466, "in_transit": 310}
EXPECTED_BILLABLE = 8_365
EXPECTED_REJECTED_DISTANCE = 101  # completed nhưng khoảng cách ngoài miền 0..100 km
EXPECTED_REVENUE = Decimal("338740950")  # VND, tổng cước của các chuyến tính cước được


# ---------------------------------------------------------------- miền (rút gọn từ bài 10)

class ServiceKind(StrEnum):
    RIDE = "ride"
    FOOD = "food"
    PARCEL = "parcel"


class FarePolicy(NamedTuple):
    base_fare: Decimal = Decimal("12000")
    included_km: Decimal = Decimal("2")
    price_per_extra_km: Decimal = Decimal("4500")


STANDARD = FarePolicy()
POLICY_5000 = FarePolicy(price_per_extra_km=Decimal("5000"))


def calculate_fare(
    distance_km: Decimal,
    policy: FarePolicy = STANDARD,
    *,
    service: ServiceKind = ServiceKind.RIDE,
) -> Decimal:
    """Cước = giá cơ bản + phần vượt x giá mỗi km vượt, làm tròn ROUND_HALF_UP."""
    if not distance_km.is_finite() or not Decimal("0") <= distance_km <= MAX_KM:
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    ServiceKind(service)  # ép về Enum: dịch vụ lạ bị ValueError ngay tại đây
    extra_km = max(Decimal("0"), distance_km - policy.included_km)
    total = policy.base_fare + extra_km * policy.price_per_extra_km
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)


class TripStatus(StrEnum):
    REQUESTED = "requested"
    SEARCHING = "searching"
    ACCEPTED = "accepted"
    ARRIVING = "arriving"
    PICKED_UP = "picked_up"
    IN_TRANSIT = "in_transit"
    COMPLETED = "completed"
    CANCELLED = "cancelled"


class Trip(NamedTuple):
    trip_id: str
    city: str
    service: ServiceKind
    distance_km: Decimal
    status: TripStatus


class PricedTrip(NamedTuple):
    trip_id: str
    city: str
    service: ServiceKind
    distance_km: Decimal
    fare: Decimal


class CityStat(NamedTuple):
    trips: int
    revenue: Decimal
    average: Decimal  # cước trung bình, làm tròn ROUND_HALF_UP về đồng


# ---------------------------------------------------------------- nguồn dữ liệu giả (generator)

def generate_trips(n: int = TRIPS, seed: int = SEED) -> Iterator[Trip]:
    """Sinh n chuyến giả bằng Random(seed).random() để kết quả lặp lại được."""
    rng = Random(seed)
    for i in range(1, n + 1):
        pct = rng.random() * 100
        city = next(c for c, edge in zip(CITIES, CITY_EDGES) if pct < edge)
        r = rng.random()
        service = ServiceKind.RIDE if r < 0.70 else ServiceKind.FOOD if r < 0.90 else ServiceKind.PARCEL
        if rng.random() < 0.012:  # khoảng 1,2% khoảng cách hỏng (lỗi nhập, GPS nhảy)
            km = Decimal(1001 + int(rng.random() * 1500)) / 10  # 100,1 .. 250,0
        else:
            km = Decimal(5 + int(rng.random() ** 3 * 295)) / 10  # 0,5 .. 30,0, nghiêng về chuyến ngắn
        s = rng.random()
        status = TripStatus.COMPLETED if s < 0.85 else TripStatus.CANCELLED if s < 0.97 else TripStatus.IN_TRANSIT
        yield Trip(f"TN-{i:04d}", city, service, km, status)


# ---------------------------------------------------------------- functools.partial

def partial_demo() -> dict[str, Any]:
    fare_food = partial(calculate_fare, service=ServiceKind.FOOD)  # cố định tham số từ khóa
    fare_5000 = partial(calculate_fare, policy=POLICY_5000)
    nested = partial(fare_5000, service=ServiceKind.PARCEL)  # partial của partial được làm phẳng
    # Tạo nhiều partial trong comprehension: mỗi cái giữ ĐÚNG giá trị s của vòng lặp của nó
    per_service = [partial(calculate_fare, service=s) for s in ServiceKind]
    return {
        "food_5km": fare_food(Decimal("5")),
        "p5000_5km": fare_5000(Decimal("5")),
        "func_is_original": fare_food.func is calculate_fare,
        "keywords": fare_food.keywords,
        "nested_keywords": nested.keywords,
        "nested_func_is_original": nested.func is calculate_fare,
        "has___name__": hasattr(fare_food, "__name__"),
        "per_service_keywords": [p.keywords for p in per_service],
    }


# ---------------------------------------------------------------- reduce và compose

def compose(*steps: Callable[[Any], Any]) -> Callable[[Any], Any]:
    """compose(f, g, h)(x) == h(g(f(x))): nối các bước theo thứ tự đọc từ trái sang phải."""
    return reduce(lambda f, g: lambda x: g(f(x)), steps)


def reduce_demo(priced: list[PricedTrip]) -> dict[str, Any]:
    fares = [p.fare for p in priced]
    try:
        reduce(add, [])
        empty_error = None
    except TypeError as exc:
        empty_error = str(exc)
    return {
        "reduce_equals_sum": reduce(add, fares, Decimal(0)) == sum(fares, Decimal(0)),
        "reduce_max_equals_max": reduce(max, fares) == max(fares),
        "empty_with_initial": reduce(add, [], Decimal(0)),
        "empty_without_initial": empty_error,
        "compose_order": compose(lambda x: x + 1, lambda x: x * 10)(1),  # (1+1)*10 = 20, không phải 1*10+1
    }


# ---------------------------------------------------------------- lru_cache / cache

fare_cached = lru_cache(maxsize=1024)(calculate_fare)  # tương đương @lru_cache(maxsize=1024) trên def

ACTIVE_POLICY: dict[str, FarePolicy] = {"policy": STANDARD}


@cache  # cache == lru_cache(maxsize=None): không giới hạn số khóa
def quote_active(km: Decimal) -> Decimal:
    """SAI có chủ đích: kết quả phụ thuộc ACTIVE_POLICY, một đầu vào không nằm trong khóa cache."""
    return calculate_fare(km, ACTIVE_POLICY["policy"])


@cache
def quote_with_policy(km: Decimal, policy: FarePolicy) -> Decimal:
    """ĐÚNG: mọi thứ ảnh hưởng kết quả đều là tham số (FarePolicy là NamedTuple nên hashable)."""
    return calculate_fare(km, policy)


def cache_demo(billable_trips: list[Trip]) -> dict[str, Any]:
    fare_cached.cache_clear()
    for t in billable_trips:
        fare_cached(t.distance_km, service=t.service)
    info = fare_cached.cache_info()
    distinct = len({(t.distance_km, t.service) for t in billable_trips})

    quote_active.cache_clear()
    quote_with_policy.cache_clear()
    ACTIVE_POLICY["policy"] = STANDARD
    before = quote_active(Decimal("5"))
    ACTIVE_POLICY["policy"] = POLICY_5000  # đổi chính sách giữa chừng
    stale = quote_active(Decimal("5"))  # vẫn trả giá cũ: khóa chỉ có km
    ACTIVE_POLICY["policy"] = STANDARD
    try:
        fare_cached([Decimal("5")])  # type: ignore[arg-type]
        unhashable = None
    except TypeError as exc:
        unhashable = str(exc)
    return {
        "hits": info.hits,
        "misses": info.misses,
        "currsize": info.currsize,
        "calls": len(billable_trips),
        "distinct_keys": distinct,
        "stale_before": before,
        "stale_after_policy_change": stale,
        "correct_standard": quote_with_policy(Decimal("5"), STANDARD),
        "correct_5000": quote_with_policy(Decimal("5"), POLICY_5000),
        "unhashable_error": unhashable,
    }


# ---------------------------------------------------------------- các bước pipeline (generator)

def is_billable(t: Trip) -> bool:
    """Tính cước được: chuyến đã hoàn thành và khoảng cách trong miền 0..100 km."""
    return t.status is TripStatus.COMPLETED and Decimal("0") <= t.distance_km <= MAX_KM


def price_one(fare_fn: Callable[..., Decimal], t: Trip) -> PricedTrip:
    return PricedTrip(t.trip_id, t.city, t.service, t.distance_km, fare_fn(t.distance_km, service=t.service))


def make_pipeline(fare_fn: Callable[..., Decimal] = fare_cached) -> Callable[[Iterable[Trip]], Iterator[PricedTrip]]:
    """lọc -> tính cước, nối bằng compose. Chưa chạy gì cho tới khi có người duyệt kết quả."""
    return compose(
        partial(filter, is_billable),  # Iterable[Trip] -> Iterator[Trip]
        partial(map, partial(price_one, fare_fn)),  # Iterator[Trip] -> Iterator[PricedTrip]
    )


def counted[T](source: Iterable[T], counter: list[int]) -> Iterator[T]:
    """Bọc một nguồn để đếm số phần tử đã bị kéo."""
    for item in source:
        counter[0] += 1
        yield item


def stats_of(rows: list[PricedTrip]) -> CityStat:
    revenue = sum((p.fare for p in rows), Decimal(0))
    average = (revenue / len(rows)).quantize(Decimal("1"), rounding=ROUND_HALF_UP)
    return CityStat(len(rows), revenue, average)


def by_city_groupby(priced: Iterable[PricedTrip]) -> dict[str, CityStat]:
    """groupby chỉ gom phần tử LIỀN NHAU cùng khóa, nên phải sorted theo cùng khóa trước."""
    ordered = sorted(priced, key=attrgetter("city"))
    return {city: stats_of(list(rows)) for city, rows in groupby(ordered, key=attrgetter("city"))}


def by_city_one_pass(priced: Iterable[PricedTrip]) -> dict[str, CityStat]:
    """Một lượt, không sort: giữ trips và revenue theo từng khóa."""
    count: defaultdict[str, int] = defaultdict(int)
    revenue: defaultdict[str, Decimal] = defaultdict(Decimal)
    for p in priced:
        count[p.city] += 1
        revenue[p.city] += p.fare
    return {
        city: CityStat(count[city], revenue[city], (revenue[city] / count[city]).quantize(Decimal("1"), rounding=ROUND_HALF_UP))
        for city in count
    }


def process_batch(batch: tuple[Trip, ...]) -> list[PricedTrip]:
    """Xử lý một lô: ở production đây là chỗ ghi database một lần cho cả lô."""
    return list(make_pipeline()(batch))


def pipeline_demo() -> dict[str, Any]:
    pipeline = make_pipeline(fare_cached)

    # Lười: kéo 3 chuyến đầu chỉ làm nguồn bị kéo vài phần tử, không phải 10.000
    pulled = [0]
    first3 = list(islice(pipeline(counted(generate_trips(), pulled)), 3))

    status_count: dict[str, int] = defaultdict(int)
    for t in generate_trips():
        status_count[t.status.value] += 1
    billable_count = sum(1 for t in generate_trips() if is_billable(t))
    rejected_distance = sum(
        1 for t in generate_trips() if t.status is TripStatus.COMPLETED and not is_billable(t)
    )

    priced = list(pipeline(generate_trips()))
    one_pass = by_city_one_pass(priced)
    grouped = by_city_groupby(priced)

    # groupby chưa sort: nhóm trùng khóa xuất hiện nhiều lần
    unsorted_groups = [k for k, _ in groupby(priced, key=attrgetter("city"))]
    # Nhóm con của groupby bị "dùng chung" iterator: list() sang nhóm sau làm nhóm trước cạn
    held = list(groupby(sorted(priced, key=attrgetter("city")), key=attrgetter("city")))
    first_group_after_advance = list(held[0][1])

    # Theo lô: batched -> process_batch -> chain.from_iterable trải phẳng trở lại
    batches = list(batched(generate_trips(), BATCH))
    per_batch = [process_batch(b) for b in batches]
    revenues = [sum((p.fare for p in rows), Decimal(0)) for rows in per_batch]
    running = list(accumulate(revenues))
    flattened = list(chain.from_iterable(per_batch))

    total = sum((p.fare for p in priced), Decimal(0))
    return {
        "first3": first3,
        "pulled_for_first3": pulled[0],
        "status_count": dict(sorted(status_count.items())),
        "billable": billable_count,
        "rejected_distance": rejected_distance,
        "priced": len(priced),
        "total_revenue": total,
        "grouped": grouped,
        "same_as_one_pass": grouped == one_pass,
        "unsorted_group_count": len(unsorted_groups),
        "sorted_group_count": len(grouped),
        "first_group_after_advance": first_group_after_advance,
        "batch_sizes": [len(b) for b in batches],
        "revenues": revenues,
        "running": running,
        "flatten_equals_direct": flattened == priced,
    }


# ---------------------------------------------------------------- đo bộ nhớ: pipeline vs list từng bước

def peak_memory(fn: Callable[[], Any]) -> tuple[Any, int]:
    tracemalloc.start()
    try:
        result = fn()
        _, peak = tracemalloc.get_traced_memory()
    finally:
        tracemalloc.stop()
    return result, peak


def aggregate_streaming(fare_fn: Callable[..., Decimal]) -> dict[str, CityStat]:
    return by_city_one_pass(make_pipeline(fare_fn)(generate_trips()))  # không list trung gian


def aggregate_materialized(fare_fn: Callable[..., Decimal]) -> dict[str, CityStat]:
    trips = list(generate_trips())
    billable = [t for t in trips if is_billable(t)]
    priced = [price_one(fare_fn, t) for t in billable]
    return by_city_one_pass(priced)


def memory_demo() -> dict[str, int]:
    """Đỉnh bộ nhớ của pipeline và của list từng bước, có và không có cache.

    Không cache: dùng thẳng calculate_fare. Có cache: cache_clear() trước mỗi lần đo để cache lạnh,
    nếu không các khóa nhớ từ lần trước làm số đo lệch. Phần chênh giữa hai dòng streaming là chi phí
    của chính cache (khóa, kết quả, danh sách liên kết).
    """
    _, stream_plain = peak_memory(lambda: aggregate_streaming(calculate_fare))
    _, list_plain = peak_memory(lambda: aggregate_materialized(calculate_fare))
    fare_cached.cache_clear()
    _, stream_cold = peak_memory(lambda: aggregate_streaming(fare_cached))
    fare_cached.cache_clear()
    _, list_cold = peak_memory(lambda: aggregate_materialized(fare_cached))
    return {"stream_plain": stream_plain, "list_plain": list_plain, "stream_cold": stream_cold, "list_cold": list_cold}


def time_best_of(fn: Callable[[], Any], repeat: int = 5) -> float:
    best = float("inf")
    for _ in range(repeat):
        start = time.perf_counter()
        fn()
        best = min(best, time.perf_counter() - start)
    return best


def dots(n: int | Decimal) -> str:
    return f"{n:,}".replace(",", ".")


# ---------------------------------------------------------------- in kết quả và --verify

def show() -> None:
    p = partial_demo()
    print("== functools.partial ==")
    for k, v in p.items():
        print(f"{k:24}:", v)

    billable = [t for t in generate_trips() if is_billable(t)]
    priced = list(make_pipeline(fare_cached)(generate_trips()))
    r = reduce_demo(priced)
    print("== reduce, compose ==")
    for k, v in r.items():
        print(f"{k:24}:", v)

    c = cache_demo(billable)
    print("== lru_cache / cache ==")
    for k, v in c.items():
        print(f"{k:26}:", v)
    t_raw = time_best_of(lambda: [calculate_fare(t.distance_km, service=t.service) for t in billable])

    def cached_run() -> list[Decimal]:
        fare_cached.cache_clear()
        return [fare_cached(t.distance_km, service=t.service) for t in billable]

    t_cached = time_best_of(cached_run)
    print(f"{len(billable)} lần tính cước: không cache {t_raw * 1000:.1f} ms | có cache (kể cả {c['misses']} lần miss) {t_cached * 1000:.1f} ms")

    d = pipeline_demo()
    print("== pipeline ==")
    print("3 chuyến đầu được tính cước:", [(x.trip_id, x.city, str(x.fare)) for x in d["first3"]])
    print("số chuyến nguồn bị kéo để lấy 3 chuyến đó:", d["pulled_for_first3"])
    print("theo trạng thái:", d["status_count"])
    print(f"tính cước được: {d['billable']} | completed nhưng khoảng cách ngoài miền: {d['rejected_distance']}")
    print(f"{'thành phố':10} {'số chuyến':>10} {'doanh thu':>14} {'cước TB':>9}")
    for city, s in d["grouped"].items():
        print(f"{city:10} {s.trips:>10} {dots(s.revenue):>14} {dots(s.average):>9}")
    print("tổng doanh thu:", dots(d["total_revenue"]), "VND")
    print("groupby == một lượt defaultdict:", d["same_as_one_pass"])
    print(f"groupby chưa sort: {d['unsorted_group_count']} nhóm | đã sort: {d['sorted_group_count']} nhóm")
    print("nhóm đầu sau khi groupby đã đi tiếp:", d["first_group_after_advance"])
    print("số chuyến mỗi lô:", d["batch_sizes"])
    print("doanh thu 3 lô đầu:", [dots(x) for x in d["revenues"][:3]])
    print("cộng dồn 3 lô đầu:", [dots(x) for x in d["running"][:3]])
    print("trải phẳng các lô == chạy thẳng:", d["flatten_equals_direct"])

    m = memory_demo()
    print("đỉnh bộ nhớ (tracemalloc), 10.000 chuyến:")
    print(f"  không cache : pipeline {m['stream_plain'] / 1024:,.1f} KiB | list từng bước {m['list_plain'] / 1024:,.0f} KiB"
          f" | tỷ lệ {m['list_plain'] / m['stream_plain']:.0f} lần")
    print(f"  cache lạnh  : pipeline {m['stream_cold'] / 1024:,.0f} KiB | list từng bước {m['list_cold'] / 1024:,.0f} KiB"
          f" | tỷ lệ {m['list_cold'] / m['stream_cold']:.0f} lần")
    print(f"  chi phí riêng của cache (811 khóa): khoảng {(m['stream_cold'] - m['stream_plain']) / 1024:,.0f} KiB")
    print("(thời gian và bộ nhớ thay đổi theo máy; số chuyến và doanh thu theo seed thì cố định)")


def verify() -> None:
    p = partial_demo()
    billable = [t for t in generate_trips() if is_billable(t)]
    priced = list(make_pipeline(fare_cached)(generate_trips()))
    r = reduce_demo(priced)
    c = cache_demo(billable)
    d = pipeline_demo()
    m = memory_demo()
    manual = [
        PricedTrip(t.trip_id, t.city, t.service, t.distance_km, calculate_fare(t.distance_km, service=t.service))
        for t in generate_trips()
        if is_billable(t)
    ]
    checks = [
        ("dữ liệu giả lặp lại được", d["status_count"] == EXPECTED_STATUS and sum(d["status_count"].values()) == TRIPS),
        ("partial cố định tham số", p["food_5km"] == Decimal("25500") and p["p5000_5km"] == Decimal("27000")
         and p["func_is_original"] and p["keywords"] == {"service": ServiceKind.FOOD}),
        ("partial của partial được làm phẳng", p["nested_func_is_original"]
         and p["nested_keywords"] == {"policy": POLICY_5000, "service": ServiceKind.PARCEL} and not p["has___name__"]),
        ("partial trong comprehension không late binding",
         p["per_service_keywords"] == [{"service": ServiceKind.RIDE}, {"service": ServiceKind.FOOD}, {"service": ServiceKind.PARCEL}]),
        ("reduce == sum, max", r["reduce_equals_sum"] and r["reduce_max_equals_max"]),
        ("reduce rỗng cần giá trị đầu", r["empty_with_initial"] == Decimal("0")
         and r["empty_without_initial"] == "reduce() of empty iterable with no initial value"),
        ("compose áp dụng từ trái sang phải", r["compose_order"] == 20),
        ("pipeline == viết tay", priced == manual and d["priced"] == EXPECTED_BILLABLE == len(billable)),
        ("lọc loại đúng chuyến lỗi khoảng cách", d["billable"] == EXPECTED_BILLABLE and d["rejected_distance"] == EXPECTED_REJECTED_DISTANCE),
        ("pipeline lười: lấy 3 chuyến chỉ kéo vài phần tử", 3 <= d["pulled_for_first3"] < 10),
        ("cache: hit + miss == số lần gọi", c["hits"] + c["misses"] == c["calls"] and c["misses"] == c["distinct_keys"] == c["currsize"]),
        ("cache sai khi kết quả phụ thuộc biến ngoài khóa", c["stale_before"] == c["stale_after_policy_change"] == Decimal("25500")
         and c["correct_standard"] == Decimal("25500") and c["correct_5000"] == Decimal("27000")),
        ("cache cần đối số hashable", c["unhashable_error"] == "unhashable type: 'list'"),
        ("groupby cần sort", d["unsorted_group_count"] > 6 and d["sorted_group_count"] == 6),
        ("nhóm con của groupby cạn khi đi tiếp", d["first_group_after_advance"] == []),
        ("groupby == một lượt defaultdict", d["same_as_one_pass"]),
        ("tổng doanh thu khớp", d["total_revenue"] == EXPECTED_REVENUE == sum((s.revenue for s in d["grouped"].values()), Decimal(0))
         and sum(s.trips for s in d["grouped"].values()) == EXPECTED_BILLABLE),
        ("batched thành 10 lô", d["batch_sizes"] == [BATCH] * 10),
        ("accumulate ra cộng dồn, chain.from_iterable trải phẳng", d["running"][-1] == d["total_revenue"] and d["flatten_equals_direct"]),
        ("pipeline streaming tốn ít bộ nhớ hơn list từng bước (không cache)", m["list_plain"] > 50 * m["stream_plain"]),
        ("cache tốn bộ nhớ riêng nhưng pipeline vẫn nhẹ hơn list (cache lạnh)", m["stream_cold"] > m["stream_plain"] and m["list_cold"] > 3 * m["stream_cold"]),
    ]
    failed = [name for name, ok in checks if not ok]
    passed = len(checks) - len(failed)
    if failed:
        print("Ca sai:", ", ".join(failed))
        print(f"FAIL {passed}/{len(checks)}")
        sys.exit(1)
    print(f"OK {passed}/{len(checks)}")


def main() -> None:
    if "--verify" in sys.argv:
        verify()
    else:
        show()


if __name__ == "__main__":
    main()
