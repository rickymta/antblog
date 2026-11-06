"""Bài 06: comprehension, iterator, generator, itertools.

Bài toán: Tới Nơi nhận khoảng 6.250 dòng vị trí tài xế mỗi giây lúc cao điểm
(25.000 tài xế, mỗi tài xế gửi một lần mỗi 4 giây). Ta giả lập 1 triệu điểm GPS
(khoảng 160 giây dữ liệu) bằng generator, lọc điểm có tốc độ phi lý mà không
nạp cả triệu điểm vào list, rồi đo bộ nhớ bằng tracemalloc.

Chạy:     uv run --project examples/toi-noi-python/06-comprehension-generator examples/toi-noi-python/06-comprehension-generator/main.py
Kiểm tra: thêm --verify ở cuối.
"""

import sys
import time
import tracemalloc
from collections import defaultdict
from collections.abc import Iterator
from decimal import Decimal
from itertools import batched, chain, groupby, islice
from random import Random
from typing import NamedTuple

POINTS = 1_000_000  # số điểm GPS giả lập
SEED = 20260614  # seed cố định: mỗi lần chạy ra đúng cùng một dòng dữ liệu
DRIVERS_ONLINE = 25_000
MAX_PLAUSIBLE_KMH = 120  # trên ngưỡng này coi là phi lý với xe máy/ô tô trong thành phố
EXPECTED_ABSURD = 972  # số điểm phi lý trong 1 triệu điểm với SEED trên (lấy từ lần chạy thật)


class Trip(NamedTuple):
    id: str
    city: str
    driver: str | None
    fare: Decimal


TRIPS = [
    Trip("TN-0001", "SG", "DR-001", Decimal("25500")),
    Trip("TN-0002", "HN", "DR-002", Decimal("12000")),
    Trip("TN-0003", "SG", "DR-001", Decimal("14250")),
    Trip("TN-0004", "DN", "DR-003", Decimal("25500")),
    Trip("TN-0005", "SG", None, Decimal("12000")),
    Trip("TN-0006", "HP", None, Decimal("14250")),
]


class GpsPoint(NamedTuple):
    seq: int
    driver_no: int  # số thứ tự tài xế giả lập, 1..25.000 (không phải mã DR-001 của dữ liệu mẫu)
    speed_kmh: float
    lat: float
    lng: float


def gps_stream(n: int, seed: int = SEED) -> Iterator[GpsPoint]:
    """Generator: mỗi lần được hỏi mới sinh MỘT điểm. Chỉ dùng Random.random()
    (kết quả theo seed ổn định) để số liệu trong bài lặp lại được."""
    rng = Random(seed)
    for seq in range(n):
        driver_no = 1 + int(rng.random() * DRIVERS_ONLINE)
        if rng.random() < 0.001:  # khoảng 0,1% điểm lỗi: tốc độ 150-400 km/h
            speed = 150 + rng.random() * 250
        else:
            speed = rng.random() * 60  # 0-60 km/h
        lat = 10.70 + rng.random() * 0.20
        lng = 106.60 + rng.random() * 0.20
        yield GpsPoint(seq, driver_no, speed, lat, lng)


def is_absurd(p: GpsPoint) -> bool:
    return p.speed_kmh > MAX_PLAUSIBLE_KMH


# ---------------------------------------------------------------- comprehension

def comprehension_demo() -> dict:
    expensive = [t.id for t in TRIPS if t.fare >= Decimal("14250")]  # list comprehension
    fare_by_id = {t.id: t.fare for t in TRIPS}  # dict comprehension
    cities = {t.city for t in TRIPS}  # set comprehension: tự loại trùng

    # Cách 1: gói hết vào một dòng lồng nhau. Ngắn nhưng khó đọc và quét TRIPS nhiều lần.
    one_liner = {
        d: [t.id for t in TRIPS if t.driver == d and t.fare >= Decimal("14250")]
        for d in {t.driver for t in TRIPS if t.driver}
    }
    # Cách 2: vòng for thường. Dài hơn nhưng đọc từ trên xuống và quét một lần.
    loop: defaultdict[str, list[str]] = defaultdict(list)
    for t in TRIPS:
        if t.driver and t.fare >= Decimal("14250"):
            loop[t.driver].append(t.id)
    # Cách 1 còn tạo khóa cho tài xế không có chuyến nào đạt ngưỡng (list rỗng); cách 2 không.
    same = {d: ids for d, ids in one_liner.items() if ids} == dict(loop)
    return {
        "expensive": expensive,
        "fare_by_id": {k: str(v) for k, v in fare_by_id.items()},
        "cities": sorted(cities),
        "loop": dict(loop),
        "same_result": same,
    }


# ---------------------------------------------------------------- iterator protocol

def iterator_demo() -> dict:
    ids = ["TN-0001", "TN-0002"]
    it = iter(ids)  # iterable -> iterator
    first, second = next(it), next(it)
    try:
        next(it)
        stopped = False
    except StopIteration:
        stopped = True

    # Đây là điều vòng for làm ngầm: gọi iter() một lần, rồi next() tới khi StopIteration.
    manual = []
    it2 = iter(ids)
    while True:
        try:
            manual.append(next(it2))
        except StopIteration:
            break
    return {
        "first": first,
        "second": second,
        "stopped": stopped,
        "manual_equals_for": manual == [i for i in ids],
        "list_is_iterator": iter(ids) is ids,  # list là iterable, không phải iterator
        "iterator_is_itself": iter(it2) is it2,  # iterator trả về chính nó
    }


# ---------------------------------------------------------------- generator function

def trip_ids(log: list[str]):
    """Generator function: thân hàm chạy từng đoạn, dừng ở mỗi yield."""
    for t in TRIPS[:3]:
        log.append(f"yield {t.id}")
        yield t.id


def generator_demo() -> dict:
    log: list[str] = []
    gen = trip_ids(log)
    before = list(log)  # gọi trip_ids() chưa chạy dòng nào trong thân hàm
    a = next(gen)
    after_one = list(log)
    rest = list(gen)  # kéo nốt phần còn lại

    squares = (n * n for n in range(1, 4))  # generator expression
    first_sum = sum(squares)
    second_sum = sum(squares)  # đã cạn: tiêu thụ một lần
    return {
        "log_before": before,
        "first": a,
        "log_after_one": after_one,
        "rest": rest,
        "first_sum": first_sum,
        "second_sum": second_sum,
    }


# ---------------------------------------------------------------- itertools

def itertools_demo() -> dict:
    head = [p.seq for p in islice(gps_stream(POINTS), 3)]  # chỉ lấy 3 điểm đầu của 1 triệu
    chained = list(chain(["TN-0001"], ["TN-0002", "TN-0003"]))  # nối nhiều iterable
    batches = list(batched((t.id for t in TRIPS), 4))  # itertools.batched: có trong stdlib từ 3.12

    # groupby chỉ gom các phần tử LIỀN NHAU có cùng khóa, nên phải sort theo khóa trước.
    unsorted_groups = [(city, len(list(g))) for city, g in groupby(TRIPS, key=lambda t: t.city)]
    sorted_groups = [
        (city, len(list(g))) for city, g in groupby(sorted(TRIPS, key=lambda t: t.city), key=lambda t: t.city)
    ]
    return {
        "head": head,
        "chained": chained,
        "batches": batches,
        "unsorted_groups": unsorted_groups,
        "sorted_groups": sorted_groups,
    }


# ---------------------------------------------------------------- tổng hợp trên generator

def aggregate_demo() -> dict:
    n = 100_000
    top_speed = max(p.speed_kmh for p in gps_stream(n))
    any_absurd = any(is_absurd(p) for p in gps_stream(n))
    all_in_city = all(10.70 <= p.lat <= 10.90 for p in gps_stream(n))

    # any() dừng ngay khi gặp True: đếm xem nó đã kéo bao nhiêu điểm.
    pulled = 0

    def counted() -> Iterator[GpsPoint]:
        nonlocal pulled
        for p in gps_stream(POINTS):
            pulled += 1
            yield p

    found = any(is_absurd(p) for p in counted())
    return {
        "top_speed": round(top_speed, 2),
        "any_absurd": any_absurd,
        "all_in_city": all_in_city,
        "any_found": found,
        "points_pulled_by_any": pulled,
    }


# ---------------------------------------------------------------- bài toán chính

def count_absurd_stream(n: int = POINTS) -> int:
    """Duyệt n điểm, tại mọi thời điểm chỉ giữ một điểm đang xử lý."""
    return sum(1 for p in gps_stream(n) if is_absurd(p))


def first_absurd(k: int = 3, n: int = POINTS) -> list[tuple[int, int, float]]:
    return [(p.seq, p.driver_no, round(p.speed_kmh, 1)) for p in islice(filter(is_absurd, gps_stream(n)), k)]


def absurd_via_list(n: int = POINTS) -> int:
    points = list(gps_stream(n))  # nạp hết n điểm vào RAM
    return len([p for p in points if is_absurd(p)])


def peak_memory(fn, *args) -> tuple[object, int]:
    """Chạy fn dưới tracemalloc, trả (kết quả, số byte đỉnh)."""
    tracemalloc.start()
    try:
        result = fn(*args)
        _, peak = tracemalloc.get_traced_memory()
    finally:
        tracemalloc.stop()
    return result, peak


def elapsed(fn, *args) -> tuple[object, float]:
    start = time.perf_counter()
    result = fn(*args)
    return result, time.perf_counter() - start


def human(n_bytes: int) -> str:
    if n_bytes >= 1024 * 1024:
        return f"{n_bytes / 1024 / 1024:,.1f} MiB"
    return f"{n_bytes / 1024:,.1f} KiB"


def show() -> None:
    c = comprehension_demo()
    print("== comprehension ==")
    print("list :", c["expensive"])
    print("dict :", c["fare_by_id"])
    print("set  :", c["cities"])
    print("một dòng == vòng for:", c["same_result"], "| kết quả:", c["loop"])

    i = iterator_demo()
    print("== iterator ==")
    print(i)

    g = generator_demo()
    print("== generator ==")
    for key, value in g.items():
        print(f"{key:14}:", value)

    t = itertools_demo()
    print("== itertools ==")
    for key, value in t.items():
        print(f"{key:16}:", value)

    a = aggregate_demo()
    print("== sum/max/any/all ==")
    for key, value in a.items():
        print(f"{key:21}:", value)

    print(f"== {POINTS:,} điểm GPS giả lập (seed {SEED}) ==".replace(",", "."))
    seconds_of_data = POINTS / 6250
    print(f"tương đương {seconds_of_data:.0f} giây dữ liệu ở 6.250 dòng/giây")
    print("3 điểm phi lý đầu (seq, tài xế, km/h):", first_absurd())

    points_list = list(gps_stream(POINTS))
    gen = gps_stream(POINTS)
    print(
        "sys.getsizeof (chỉ đo vỏ ngoài): list",
        f"{sys.getsizeof(points_list):,} byte | generator {sys.getsizeof(gen)} byte".replace(",", "."),
    )
    del points_list, gen

    n_stream, t_stream = elapsed(count_absurd_stream)
    n_list, t_list = elapsed(absurd_via_list)
    _, m_stream = peak_memory(count_absurd_stream)
    _, m_list = peak_memory(absurd_via_list)
    print(f"generator: {n_stream} điểm phi lý | đỉnh bộ nhớ {human(m_stream)} | {t_stream:.2f} s (không đo bộ nhớ)")
    print(f"list     : {n_list} điểm phi lý | đỉnh bộ nhớ {human(m_list)} | {t_list:.2f} s (không đo bộ nhớ)")
    print(f"tỷ lệ bộ nhớ list/generator: {m_list / m_stream:.0f} lần")
    print("(thời gian và bộ nhớ thay đổi theo máy; số điểm phi lý theo seed thì cố định)")


def verify() -> None:
    c, i, g = comprehension_demo(), iterator_demo(), generator_demo()
    t, a = itertools_demo(), aggregate_demo()
    n_stream = count_absurd_stream()
    n_list = absurd_via_list(200_000)
    n_stream_small = count_absurd_stream(200_000)
    _, m_stream = peak_memory(count_absurd_stream, 200_000)
    _, m_list = peak_memory(absurd_via_list, 200_000)
    checks = [
        ("list comprehension", c["expensive"] == ["TN-0001", "TN-0003", "TN-0004", "TN-0006"]),
        ("dict/set comprehension", c["fare_by_id"]["TN-0003"] == "14250" and c["cities"] == ["DN", "HN", "HP", "SG"]),
        ("một dòng == vòng for", c["same_result"] and c["loop"] == {"DR-001": ["TN-0001", "TN-0003"], "DR-003": ["TN-0004"]}),
        ("next / StopIteration", i["first"] == "TN-0001" and i["second"] == "TN-0002" and i["stopped"]),
        ("iter protocol", i["manual_equals_for"] and not i["list_is_iterator"] and i["iterator_is_itself"]),
        ("generator lười", g["log_before"] == [] and g["log_after_one"] == ["yield TN-0001"] and g["rest"] == ["TN-0002", "TN-0003"]),
        ("generator dùng một lần", g["first_sum"] == 14 and g["second_sum"] == 0),
        ("islice / chain", t["head"] == [0, 1, 2] and t["chained"] == ["TN-0001", "TN-0002", "TN-0003"]),
        ("batched", t["batches"] == [("TN-0001", "TN-0002", "TN-0003", "TN-0004"), ("TN-0005", "TN-0006")]),
        ("groupby cần sort", len(t["unsorted_groups"]) == 6 and t["sorted_groups"] == [("DN", 1), ("HN", 1), ("HP", 1), ("SG", 3)]),
        ("any dừng sớm", a["any_found"] and a["points_pulled_by_any"] < 10_000 and a["any_absurd"]),
        ("max / all", a["top_speed"] > MAX_PLAUSIBLE_KMH and a["all_in_city"]),
        ("1 triệu điểm, số điểm phi lý", n_stream == EXPECTED_ABSURD),
        ("generator == list", n_stream_small == n_list),
        ("generator giữ bộ nhớ nhỏ", m_stream < 100_000 and m_list > 100 * m_stream),
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
