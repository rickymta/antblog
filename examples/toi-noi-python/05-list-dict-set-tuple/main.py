"""Bài 05: list, dict, set, tuple, Counter, defaultdict, deque.

Dữ liệu giả của Tới Nơi: 6 chuyến TN-0001..TN-0006, 3 tài xế DR-001..DR-003,
6 thành phố. Mọi thứ nằm trong bộ nhớ của một process: tắt chương trình là mất,
không có transaction, không có khóa giữa nhiều tiến trình.

Chạy:     uv run --project examples/toi-noi-python/05-list-dict-set-tuple examples/toi-noi-python/05-list-dict-set-tuple/main.py
Kiểm tra: thêm --verify ở cuối. Đo thời gian nhỏ: thêm --bench.
"""

import copy
import sys
from collections import Counter, defaultdict, deque
from decimal import Decimal
from typing import NamedTuple


class Trip(NamedTuple):
    """Một chuyến là một tuple có tên: bất biến, truy cập bằng trip.city."""

    id: str
    city: str
    service: str
    driver: str | None  # None = chưa có tài xế nhận
    fare: Decimal
    status: str


# Sáu thành phố của Tới Nơi: mã -> tên. Dict giữ đúng thứ tự khai báo.
CITIES = {
    "SG": "TP. Hồ Chí Minh",
    "HN": "Hà Nội",
    "DN": "Đà Nẵng",
    "HP": "Hải Phòng",
    "CT": "Cần Thơ",
    "NT": "Nha Trang",
}

TRIPS = [
    Trip("TN-0001", "SG", "ride", "DR-001", Decimal("25500"), "completed"),
    Trip("TN-0002", "HN", "food", "DR-002", Decimal("12000"), "completed"),
    Trip("TN-0003", "SG", "parcel", "DR-001", Decimal("14250"), "in_transit"),
    Trip("TN-0004", "DN", "ride", "DR-003", Decimal("25500"), "accepted"),
    Trip("TN-0005", "SG", "food", None, Decimal("12000"), "searching"),
    Trip("TN-0006", "HP", "ride", None, Decimal("14250"), "requested"),
]

DRIVERS = {
    "DR-001": {"name": "Nguyễn Văn An", "city": "SG"},
    "DR-002": {"name": "Trần Thị Bình", "city": "HN"},
    "DR-003": {"name": "Lê Minh Châu", "city": "DN"},
}


def list_demo() -> dict:
    """List: có thứ tự, sửa được, cắt lát, sắp xếp theo key."""
    ids = [t.id for t in TRIPS]
    first_two = ids[:2]  # slicing: tạo list mới
    last = ids[-1]  # chỉ số âm đếm từ cuối

    queue_view = ids.copy()
    queue_view.append("TN-0007")  # thêm vào cuối
    queue_view.insert(0, "TN-0000")  # chèn đầu: dịch cả list
    popped_last = queue_view.pop()  # lấy phần tử cuối
    popped_first = queue_view.pop(0)  # lấy phần tử đầu: dịch cả list

    # sorted() trả list mới; key quyết định cách so sánh. (-fare, id): cước giảm dần, mã tăng dần.
    by_fare = [t.id for t in sorted(TRIPS, key=lambda t: (-t.fare, t.id))]
    return {
        "ids": ids,
        "first_two": first_two,
        "last": last,
        "popped_last": popped_last,
        "popped_first": popped_first,
        "after_pops": queue_view,
        "by_fare": by_fare,
    }


def tuple_demo() -> dict:
    """Tuple: bất biến, unpack, dùng làm khóa dict."""
    trip = TRIPS[0]
    trip_id, city, *_rest = trip  # unpack, *_rest nhận phần còn lại
    pickup = (10.7769, 106.7009)  # (vĩ độ, kinh độ)
    lat, lng = pickup
    try:
        pickup[0] = 0.0  # type: ignore[index]
        immutable = False
    except TypeError:
        immutable = True

    # Khóa ghép (thành phố, dịch vụ) -> số chuyến. Tuple hash được nên làm khóa được.
    by_city_service: dict[tuple[str, str], int] = {}
    for t in TRIPS:
        key = (t.city, t.service)
        by_city_service[key] = by_city_service.get(key, 0) + 1
    return {
        "trip_id": trip_id,
        "city": city,
        "lat": lat,
        "lng": lng,
        "immutable": immutable,
        "by_city_service": by_city_service,
    }


def dict_demo() -> dict:
    """Dict: tra theo khóa, giữ thứ tự chèn, get / setdefault / items."""
    found = DRIVERS["DR-001"]["name"]
    missing_get = DRIVERS.get("DR-999")  # None thay vì KeyError
    missing_default = DRIVERS.get("DR-999", {"name": "(không có)"})["name"]
    try:
        DRIVERS["DR-999"]
        raised = False
    except KeyError:
        raised = True

    # setdefault: lấy giá trị nếu khóa có, nếu chưa có thì chèn mặc định rồi trả về.
    shifts: dict[str, list[str]] = {}
    shifts.setdefault("DR-001", []).append("sáng")
    shifts.setdefault("DR-001", []).append("chiều")  # khóa đã có: không tạo list mới
    order = list(DRIVERS)  # thứ tự chèn từ Python 3.7
    pairs = [(code, d["city"]) for code, d in DRIVERS.items()]
    return {
        "found": found,
        "missing_get": missing_get,
        "missing_default": missing_default,
        "raised": raised,
        "shifts": shifts,
        "order": order,
        "pairs": pairs,
    }


def set_demo() -> dict:
    """Set: loại trùng, kiểm tra thuộc tập, phép tập hợp."""
    online = {"DR-001", "DR-002", "DR-002"}  # trùng bị loại ngay
    eligible = {"DR-002", "DR-003"}  # đủ điều kiện nhận chuyến
    return {
        "online_size": len(online),
        "has_dr001": "DR-001" in online,
        "dispatchable": sorted(online & eligible),  # giao: online VÀ đủ điều kiện
        "either": sorted(online | eligible),  # hợp
        "online_only": sorted(online - eligible),  # online nhưng không đủ điều kiện
        "exactly_one": sorted(online ^ eligible),  # đúng một trong hai tập
        "no_duplicate": len({t.id for t in TRIPS}) == len(TRIPS),
    }


def alias_demo() -> dict:
    """Gán tên không sao chép đối tượng; copy nông và deepcopy khác nhau ở phần lồng nhau."""
    a = ["TN-0001", "TN-0002"]
    b = a  # alias: hai tên, một list
    b.append("TN-0003")
    same_object = a is b

    original = {"driver": "DR-001", "trips": ["TN-0001"]}
    shallow = original.copy()  # sao chép lớp ngoài, list bên trong vẫn dùng chung
    deep = copy.deepcopy(original)  # sao chép toàn bộ
    shallow["trips"].append("TN-0003")
    deep["trips"].append("TN-0009")
    return {
        "a_after": a,
        "same_object": same_object,
        "original_trips": original["trips"],
        "shallow_shares": shallow["trips"] is original["trips"],
        "deep_shares": deep["trips"] is original["trips"],
        "deep_trips": deep["trips"],
    }


def hash_demo() -> dict:
    """Chỉ đối tượng hash được mới làm khóa dict hoặc phần tử set."""
    results = {}
    for name, value in [("tuple", ("SG", "ride")), ("str", "DR-001"), ("list", ["SG"])]:
        try:
            hash(value)
            results[name] = True
        except TypeError:
            results[name] = False
    # Tuple chứa list thì không hash được, dù tuple là bất biến.
    try:
        hash(("SG", ["ride"]))
        results["tuple_with_list"] = True
    except TypeError:
        results["tuple_with_list"] = False
    return results


def collections_demo() -> dict:
    """Counter, defaultdict(list), deque."""
    per_city = Counter(t.city for t in TRIPS)  # đếm chuyến theo thành phố
    per_city_all = {code: per_city[code] for code in CITIES}  # khóa vắng -> 0, không KeyError

    by_driver: defaultdict[str, list[str]] = defaultdict(list)
    waiting: deque[str] = deque()
    for t in TRIPS:
        if t.driver is None:
            waiting.append(t.id)  # chưa có tài xế: vào hàng đợi điều phối
        else:
            by_driver[t.driver].append(t.id)  # khóa mới tự tạo list rỗng

    served = []
    while waiting:
        served.append(waiting.popleft())  # FIFO: vào trước, ra trước
    return {
        "per_city": dict(per_city),
        "most_common": per_city.most_common(1),
        "per_city_all": per_city_all,
        "by_driver": dict(by_driver),
        "served": served,
        "empty_after": len(waiting),
    }


def exercise_demo() -> dict:
    """Đáp án bài tập trong bài viết."""
    top_service = Counter(t.service for t in TRIPS).most_common(1)  # bài tập 1
    a = [1, 2]
    c = [a, a]
    c[0].append(3)
    d = copy.deepcopy(c)  # bài tập 2: deepcopy nhớ đối tượng đã sao chép
    return {"top_service": top_service, "c": c, "deep_keeps_sharing": d[0] is d[1], "deep_detached": d[0] is not a}


def bench() -> None:
    """Đo nhỏ: tìm phần tử cuối trong list và set 20.000 mã. Số đo thay đổi theo máy."""
    from timeit import timeit

    codes = [f"DR-{i:05d}" for i in range(20_000)]
    code_set = set(codes)
    target = codes[-1]
    t_list = timeit(lambda: target in codes, number=500)
    t_set = timeit(lambda: target in code_set, number=500)
    print(f"[bench] 500 lần tìm phần tử cuối trong 20.000 mã: list {t_list * 1000:.2f} ms, set {t_set * 1000:.3f} ms")


def show() -> None:
    lst, tup, dct, st = list_demo(), tuple_demo(), dict_demo(), set_demo()
    print("== list ==")
    print("ids        :", lst["ids"])
    print("ids[:2]    :", lst["first_two"], "| ids[-1]:", lst["last"])
    print("pop cuối/đầu:", lst["popped_last"], lst["popped_first"], "->", lst["after_pops"])
    print("theo cước  :", lst["by_fare"])
    print("== tuple ==")
    print("unpack     :", tup["trip_id"], tup["city"], "| tọa độ:", tup["lat"], tup["lng"])
    print("gán phần tử bị TypeError:", tup["immutable"])
    print("khóa ghép  :", tup["by_city_service"])
    print("== dict ==")
    print("DR-001     :", dct["found"], "| get DR-999:", dct["missing_get"], "| mặc định:", dct["missing_default"])
    print("DRIVERS['DR-999'] gây KeyError:", dct["raised"])
    print("setdefault :", dct["shifts"], "| thứ tự:", dct["order"])
    print("items      :", dct["pairs"])
    print("== set ==")
    for key, value in st.items():
        print(f"{key:12}:", value)
    print("== alias ==")
    al = alias_demo()
    for key, value in al.items():
        print(f"{key:14}:", value)
    print("== hashable ==")
    print(hash_demo())
    print("== collections ==")
    for key, value in collections_demo().items():
        print(f"{key:12}:", value)
    print("(mọi thứ ở trên nằm trong RAM của process này: không bền, không transaction)")


def verify() -> None:
    lst, tup, dct, st = list_demo(), tuple_demo(), dict_demo(), set_demo()
    al, hs, co = alias_demo(), hash_demo(), collections_demo()
    ex = exercise_demo()
    checks = [
        ("slicing", lst["first_two"] == ["TN-0001", "TN-0002"] and lst["last"] == "TN-0006"),
        ("pop", lst["popped_last"] == "TN-0007" and lst["popped_first"] == "TN-0000" and lst["after_pops"] == lst["ids"]),
        ("sort key", lst["by_fare"] == ["TN-0001", "TN-0004", "TN-0003", "TN-0006", "TN-0002", "TN-0005"]),
        ("tuple unpack", tup["trip_id"] == "TN-0001" and tup["city"] == "SG" and tup["immutable"]),
        ("tuple key", tup["by_city_service"][("SG", "ride")] == 1 and len(tup["by_city_service"]) == 6),
        ("dict get", dct["found"] == "Nguyễn Văn An" and dct["missing_get"] is None and dct["raised"]),
        ("setdefault", dct["shifts"] == {"DR-001": ["sáng", "chiều"]} and dct["order"] == ["DR-001", "DR-002", "DR-003"]),
        ("set size", st["online_size"] == 2 and st["has_dr001"] and st["no_duplicate"]),
        ("set algebra", st["dispatchable"] == ["DR-002"] and st["online_only"] == ["DR-001"]
         and st["either"] == ["DR-001", "DR-002", "DR-003"] and st["exactly_one"] == ["DR-001", "DR-003"]),
        ("alias", al["same_object"] and al["a_after"] == ["TN-0001", "TN-0002", "TN-0003"]),
        ("copy vs deepcopy", al["shallow_shares"] and not al["deep_shares"]
         and al["original_trips"] == ["TN-0001", "TN-0003"] and al["deep_trips"] == ["TN-0001", "TN-0009"]),
        ("hashable", hs == {"tuple": True, "str": True, "list": False, "tuple_with_list": False}),
        ("Counter", co["per_city"] == {"SG": 3, "HN": 1, "DN": 1, "HP": 1} and co["most_common"] == [("SG", 3)]
         and co["per_city_all"]["CT"] == 0),
        ("defaultdict", co["by_driver"] == {"DR-001": ["TN-0001", "TN-0003"], "DR-002": ["TN-0002"], "DR-003": ["TN-0004"]}),
        ("deque FIFO", co["served"] == ["TN-0005", "TN-0006"] and co["empty_after"] == 0),
        ("bài tập 1", ex["top_service"] == [("ride", 3)]),
        ("bài tập 2", ex["c"] == [[1, 2, 3], [1, 2, 3]] and ex["deep_keeps_sharing"] and ex["deep_detached"]),
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
        return
    show()
    if "--bench" in sys.argv:
        bench()


if __name__ == "__main__":
    main()
