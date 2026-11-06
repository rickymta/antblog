"""Bài 12: type hints, mypy, Protocol, generics. Chạy: main.py | main.py --verify."""

import argparse
import re
import subprocess
import sys
from annotationlib import Format, get_annotations
from datetime import UTC, datetime
from decimal import Decimal
from pathlib import Path
from typing import Any

import domain
from domain import (
    Driver,
    QuoteLine,
    Registry,
    Trip,
    booking_distance,
    cheapest,
    first,
    km_label,
    parse_booking_created,
    total_fare,
)

HERE = Path(__file__).resolve().parent
D = Decimal
CREATED = datetime(2026, 1, 1, 8, 0, tzinfo=UTC)

# Chuỗi JSON thật do Kafka giao tới (bài 38 sẽ làm phần đó). Thiếu `distance_km` ở bản hỏng.
GOOD_EVENT = (
    '{"event_id": "EV-0001", "event_type": "BookingCreated", "occurred_at": "2026-01-01T08:00:00Z",'
    ' "version": 1, "payload": {"trip_id": "TN-0001", "customer_id": "CU-001", "service": "ride",'
    ' "distance_km": "5"}}'
)
BAD_EVENT = GOOD_EVENT.replace(', "distance_km": "5"', "")


def make_trip(trip_id: str, km: str, service: domain.ServiceName = "ride") -> Trip:
    return Trip(trip_id, "CU-001", None, service, D(km), "requested", CREATED)


def lazy_annotation_probe() -> tuple[str, dict[str, Any], dict[str, str]]:
    """Định nghĩa hàm tham chiếu tên chưa tồn tại: 3.14 không lỗi lúc `def`, chỉ lỗi khi đọc.

    Dùng exec trên chuỗi cố định để mypy không phàn nàn về tên `Voucher` chưa khai báo.
    Trả về: kết quả đọc khi chưa có Voucher, annotation (Format.VALUE) sau khi đã khai báo
    Voucher, và dạng chuỗi (Format.STRING).
    """
    namespace: dict[str, Any] = {}
    exec("def apply_voucher(voucher: Voucher) -> None: ...", namespace)
    func = namespace["apply_voucher"]  # đến đây chưa có lỗi nào
    try:
        func.__annotations__
        outcome = "đọc được"
    except NameError as exc:
        outcome = f"NameError: {exc}"
    as_text = get_annotations(func, format=Format.STRING)  # không cần tên tồn tại
    exec("class Voucher: ...", namespace)  # khai báo SAU khi hàm đã tồn tại
    resolved = get_annotations(func, format=Format.VALUE)
    return outcome, resolved, as_text


def demo() -> None:
    print("== 1. Kiểu cơ bản và X | None ==")
    t1, t2 = make_trip("TN-0001", "5"), make_trip("TN-0002", "1.5", "food")
    for trip in (t1, t2):
        print(f"{trip.trip_id}: {km_label(trip.distance_km)} {trip.service} -> {trip.fare} VND, driver_id={trip.driver_id}")
    assigned = t1.with_driver(Driver("DR-001", "Nguyễn Văn A", "Hà Nội"))
    print(f"with_driver -> {assigned.driver_id}; bản gốc vẫn {t1.driver_id}")

    print("== 2. Generics ==")
    print(f"first(['TN-0001', 'TN-0002']) = {first(['TN-0001', 'TN-0002'])}")
    trips: Registry[Trip] = Registry("chuyến")
    trips.add(t1.trip_id, t1)
    trips.add(t2.trip_id, t2)
    print(f"Registry[Trip] có {len(trips.values())} chuyến; get('TN-0009') = {trips.get('TN-0009')}")

    print("== 3. Protocol SupportsFare ==")
    best = cheapest(trips.values())
    print(f"cheapest(Trip) = {best.trip_id} ({best.fare} VND)")
    lines = [QuoteLine("5 km", D("25500")), QuoteLine("1,5 km", D("12000"))]
    print(f"cheapest(QuoteLine) = {cheapest(lines).label}; total_fare(trips) = {total_fare(trips.values())}")

    print("== 4. TypedDict qua JSON ==")
    event = parse_booking_created(GOOD_EVENT)
    print(f"{event['event_type']} v{event['version']}: {event['payload']['trip_id']} {booking_distance(event)} km")
    bad = parse_booking_created(BAD_EVENT)  # mypy không biết chuỗi này thiếu khóa
    try:
        booking_distance(bad)
    except KeyError as exc:
        print(f"parse không lỗi, dùng mới lỗi: KeyError {exc}")

    print("== 5. Annotation lười (PEP 649) ==")
    print(f"Trip.with_driver (Trip trả về chính nó, Driver khai báo sau): {get_annotations(Trip.with_driver, format=Format.STRING)}")
    outcome, resolved, as_text = lazy_annotation_probe()
    print(f"def apply_voucher(voucher: Voucher) chạy qua; đọc __annotations__ -> {outcome}")
    print(f"Format.STRING không cần tên tồn tại: {as_text}")
    print(f"khai báo class Voucher sau đó, Format.VALUE giải được: voucher -> {resolved['voucher'].__name__}, return -> {resolved['return']}")


def run_mypy(target: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, "-m", "mypy", target, "--strict", "--no-color-output"],
        capture_output=True,
        text=True,
        cwd=HERE,
    )


def marked_lines(path: Path) -> set[int]:
    """Số dòng có chú thích `# E:` trong broken.py: những dòng mypy bắt buộc phải báo."""
    return {no for no, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1) if "# E:" in line}


def reported_lines(output: str) -> set[int]:
    return {int(m.group(1)) for m in re.finditer(r"^broken\.py:(\d+): error:", output, re.MULTILINE)}


def verify() -> None:
    t1, t2 = make_trip("TN-0001", "5"), make_trip("TN-0002", "1.5", "food")
    registry: Registry[Trip] = Registry("chuyến")
    registry.add("TN-0001", t1)
    good, bad = parse_booking_created(GOOD_EVENT), parse_booking_created(BAD_EVENT)
    missing_key = False
    try:
        booking_distance(bad)
    except KeyError:
        missing_key = True
    outcome, resolved, as_text = lazy_annotation_probe()
    forward = get_annotations(Trip.with_driver, format=Format.VALUE)
    strict_main = run_mypy("main.py")
    strict_broken = run_mypy("broken.py")
    expected_lines = marked_lines(HERE / "broken.py")
    # km_label(2.5) vẫn chạy được lúc runtime: Python không kiểm kiểu
    runtime_accepts_float = km_label(2.5) == "2.5 km"  # type: ignore[arg-type]

    checks: list[tuple[str, bool]] = [
        ("TN-0001 5 km => 25500", t1.fare == D("25500")),
        ("TN-0002 1,5 km => 12000", t2.fare == D("12000")),
        ("with_driver trả bản sao, bản gốc giữ None", t1.with_driver(Driver("DR-001", "A", "Hà Nội")).driver_id == "DR-001" and t1.driver_id is None),
        ("first lấy phần tử đầu", first(["TN-0001", "TN-0002"]) == "TN-0001"),
        ("first danh sách rỗng raise ValueError", _raises(ValueError, first, [])),
        ("Registry.get khóa lạ => None", registry.get("TN-0009") is None),
        ("Registry.require khóa lạ raise KeyError", _raises(KeyError, registry.require, "TN-0009")),
        ("Registry.add trùng raise ValueError", _raises(ValueError, registry.add, "TN-0001", t1)),
        ("cheapest(Trip) chọn TN-0002", cheapest([t1, t2]).trip_id == "TN-0002"),
        ("cheapest(QuoteLine) cùng Protocol", cheapest([QuoteLine("a", D("2")), QuoteLine("b", D("1"))]).label == "b"),
        ("total_fare(Trip) = 37500", total_fare([t1, t2]) == D("37500")),
        ("TypedDict: đọc đúng khóa", booking_distance(good) == D("5") and good["payload"]["trip_id"] == "TN-0001"),
        ("TypedDict không kiểm lúc chạy: thiếu khóa chỉ lộ khi dùng", missing_key),
        ("annotation lười: đọc tên chưa có => NameError", outcome == "NameError: name 'Voucher' is not defined"),
        ("Format.STRING không cần tên tồn tại", as_text == {"voucher": "Voucher", "return": "None"}),
        ("khai báo Voucher sau: Format.VALUE giải được", resolved["voucher"].__name__ == "Voucher" and resolved["return"] is None),
        ("tham chiếu tiến Driver / Trip không cần nháy", forward == {"driver": Driver, "return": Trip}),
        ("Python chạy km_label(2.5) dù sai kiểu", runtime_accepts_float),
        ("mypy --strict main.py (đi theo import sang domain.py): exit 0", strict_main.returncode == 0),
        ("mypy --strict broken.py: exit 1, 9 lỗi", strict_broken.returncode == 1 and "Found 9 errors in 1 file" in strict_broken.stdout),
        ("broken.py: lỗi đúng các dòng đánh dấu # E:", reported_lines(strict_broken.stdout) == expected_lines),
    ]
    passed = sum(1 for _, ok in checks if ok)
    for name, ok in checks:
        if not ok:
            print(f"  ca sai: {name}")
    if passed != len(checks):
        print(f"FAIL {passed}/{len(checks)}")
        sys.exit(1)
    print(f"OK {passed}/{len(checks)}")


def _raises(exc_type: type[BaseException], func: Any, *args: object) -> bool:
    try:
        func(*args)
    except exc_type:
        return True
    return False


def main() -> None:
    parser = argparse.ArgumentParser(description="Type hints, mypy, Protocol, generics cho mã nghiệp vụ Tới Nơi")
    parser.add_argument("--verify", action="store_true", help="kiểm các kết quả kỳ vọng, gồm chạy mypy thật")
    args = parser.parse_args()
    if args.verify:
        verify()
    else:
        demo()


if __name__ == "__main__":
    main()
