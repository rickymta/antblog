"""Bài 16: match/case và xử lý None - chuyển trạng thái chuyến rõ ràng.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify
"""

import sys
from typing import Any

from matching import (
    UNSET,
    InvalidTransition,
    apply_command,
    describe,
    driver_label,
    driver_label_truthy,
    handle_event,
    has_usable_driver,
    is_assigned,
    next_step_label,
    patch_trip,
)
from trip import Trip, TripStatus, make_trip

S = TripStatus

# Ca mô tả chuyến: (nhãn ca, chuyến hoặc None).
DESCRIBE_CASES: list[tuple[str, Trip | None]] = [
    ("không có chuyến", None),
    ("requested, chưa tài xế", make_trip()),
    ("searching, chưa tài xế", make_trip(S.SEARCHING)),
    ("requested mà có tài xế", make_trip(S.REQUESTED, "DR-001")),
    ("accepted, DR-001", make_trip(S.ACCEPTED, "DR-001")),
    ("accepted, thiếu tài xế", make_trip(S.ACCEPTED)),
    ("accepted, driver_id trống", make_trip(S.ACCEPTED, "  ")),
    ("in_transit 25 km", make_trip(S.IN_TRANSIT, "DR-001", "25")),
    ("in_transit 5 km", make_trip(S.IN_TRANSIT, "DR-001", "5")),
    ("cancelled", make_trip(S.CANCELLED)),
]

EVENT_CASES: list[tuple[str, dict[str, Any]]] = [
    ("BookingCreated v1", {"event_id": "e1", "event_type": "BookingCreated", "version": 1,
                           "payload": {"trip_id": "TN-0001", "service": "food", "note": "thừa khóa"}}),
    ("TripCompleted v1", {"event_type": "TripCompleted", "version": 1,
                          "payload": {"trip_id": "TN-0001", "driver_id": "DR-001"}}),
    ("driver_id là null", {"event_type": "TripCompleted", "version": 1,
                           "payload": {"trip_id": "TN-0002", "driver_id": None}}),
    ("driver_id vắng mặt", {"event_type": "TripCompleted", "version": 1,
                            "payload": {"trip_id": "TN-0003"}}),
    ("version 2", {"event_type": "TripCompleted", "version": 2, "payload": {}}),
    ("service lạ", {"event_type": "BookingCreated", "version": 1,
                    "payload": {"trip_id": "TN-0004", "service": "taxi"}}),
    ("version là True", {"event_type": "BookingCreated", "version": True,
                         "payload": {"trip_id": "TN-0005", "service": "ride"}}),
    ("không có event_type", {"type": "x"}),
]


def show(title: str) -> None:
    print(f"\n== {title} ==")


def demo_describe() -> None:
    show("class pattern: mô tả chuyến")
    for name, trip in DESCRIBE_CASES:
        print(f"{name:<26} -> {describe(trip)}")


def demo_transitions() -> None:
    show("chuyển trạng thái bằng match")
    trip = make_trip()
    for command in [("search",), ("assign", "DR-001"), ("arrive",), ("pickup",),
                    ("depart",), ("complete",)]:
        trip = apply_command(trip, command)
        print(f"{command!r:<22} -> {trip.status.value:<10} driver={trip.driver_id}")
    for status, command in [(S.COMPLETED, ("cancel",)), (S.SEARCHING, ("assign", "")),
                            (S.REQUESTED, ("assign", "DR-001")), (S.PICKED_UP, ("cancel",))]:
        try:
            apply_command(make_trip(status), command)
        except InvalidTransition as exc:
            print(f"từ chối: {exc}")


def demo_events() -> None:
    show("mapping pattern: định tuyến sự kiện")
    for name, envelope in EVENT_CASES:
        print(f"{name:<20} -> {handle_event(envelope)}")


def demo_traps() -> None:
    show("bẫy thường gặp")
    source = "def f(s):\n    match s:\n        case ACCEPTED:\n            return 1\n        case S.B:\n            return 2\n"
    try:
        compile(source, "<demo>", "exec")
    except SyntaxError as exc:
        print("tên trần trong case:", exc.msg)
    namespace: dict[str, Any] = {}
    exec("def f(s):\n    match s:\n        case ACCEPTED:\n            return 'khớp'\n", namespace)
    print("tên trần khớp mọi giá trị:", namespace["f"]("bất kỳ"))
    match True:
        case 1:
            print("literal 1 khớp True")
    match 1.0:
        case 1:
            print("literal 1 khớp 1.0")
    match "cancel":
        case ("cancel",):
            print("chuỗi khớp sequence pattern")
        case _:
            print("chuỗi KHÔNG khớp sequence pattern")
    match ["cancel"]:
        case ("cancel",):
            print("list khớp pattern viết bằng tuple")


def demo_none() -> None:
    show("None: ba trạng thái của driver_id")
    for label, driver in [("None", None), ("chuỗi rỗng", ""), ("DR-001", "DR-001")]:
        trip = make_trip(S.ACCEPTED, driver)
        print(f"{label:<11} truthy={driver_label_truthy(trip)!r:<20} match={driver_label(trip)!r}")
    show("sentinel UNSET khác None")
    trip = make_trip(S.ACCEPTED, "DR-001")
    print("patch không tham số     :", patch_trip(trip).driver_id)
    print("patch driver_id=None    :", patch_trip(trip, driver_id=None).driver_id)
    print("patch driver_id='DR-002':", patch_trip(trip, driver_id="DR-002").driver_id)
    print("UNSET                   :", repr(UNSET))


def verify() -> None:
    """So các kết quả kỳ vọng đã in trong bài; sai thì in FAIL và thoát với mã 1."""
    failures: list[str] = []
    total = 0

    def check(name: str, got: object, expected: object) -> None:
        nonlocal total
        total += 1
        if got != expected:
            failures.append(name)
            print(f"SAI: {name}: nhận {got!r}, kỳ vọng {expected!r}")

    expected_describe = [
        "Không tìm thấy chuyến",
        "Đang tìm tài xế",
        "Đang tìm tài xế",
        "Dữ liệu lỗi: chưa nhận chuyến mà đã có tài xế",
        "Tài xế DR-001: accepted",
        "Dữ liệu lỗi: chuyến đã nhận mà thiếu tài xế",
        "Dữ liệu lỗi: driver_id trống",
        "Đang chạy đường dài 25 km, tài xế DR-001",
        "Tài xế DR-001: in_transit",
        "Chuyến đã hủy",
    ]
    for (name, trip), want in zip(DESCRIBE_CASES, expected_describe, strict=True):
        check(f"describe: {name}", describe(trip), want)

    # đi hết 7 trạng thái hợp lệ
    trip = make_trip()
    path: list[str] = [trip.status.value]
    for command in [("search",), ("assign", "DR-001"), ("arrive",), ("pickup",), ("depart",), ("complete",)]:
        trip = apply_command(trip, command)
        path.append(trip.status.value)
    check("đường đi 7 trạng thái", path,
          ["requested", "searching", "accepted", "arriving", "picked_up", "in_transit", "completed"])
    check("tài xế sau assign", trip.driver_id, "DR-001")

    # hủy từ 4 trạng thái được phép
    for status in (S.REQUESTED, S.SEARCHING, S.ACCEPTED, S.ARRIVING):
        check(f"hủy từ {status.value}", apply_command(make_trip(status), ("cancel",)).status, S.CANCELLED)
    # lệnh bị từ chối
    rejected: list[tuple[TripStatus, tuple[object, ...]]] = [
        (S.COMPLETED, ("cancel",)), (S.PICKED_UP, ("cancel",)), (S.IN_TRANSIT, ("cancel",)),
        (S.SEARCHING, ("assign", "")), (S.SEARCHING, ("assign", "  ")),
        (S.SEARCHING, ("assign", None)), (S.REQUESTED, ("assign", "DR-001")),
        (S.REQUESTED, ("depart",)), (S.CANCELLED, ("search",)),
    ]
    for status, bad_command in rejected:
        try:
            apply_command(make_trip(status), bad_command)
            got = "không lỗi"
        except InvalidTransition:
            got = "InvalidTransition"
        check(f"từ chối {status.value} {bad_command!r}", got, "InvalidTransition")
    check("lệnh dạng list vẫn khớp", apply_command(make_trip(), ["search"]).status, S.SEARCHING)  # type: ignore[arg-type]
    check("không sửa Trip gốc", make_trip().status, S.REQUESTED)
    original = make_trip()
    apply_command(original, ("search",))
    check("apply_command không sửa tại chỗ", original.status, S.REQUESTED)

    # next_step_label phủ đủ 8 trạng thái
    check("next_step_label đủ 8", sum(1 for s in TripStatus if next_step_label(s)), 8)

    # sự kiện
    expected_events = [
        "tạo chuyến TN-0001 (food)",
        "chuyến TN-0001 hoàn tất, tài xế DR-001",
        "cách ly TN-0002: TripCompleted có driver_id null",
        "cách ly TripCompleted: payload sai hợp đồng (khóa: ['payload', 'version'])",
        "bỏ qua TripCompleted v2: phiên bản mới hơn consumer hiểu",
        "cách ly BookingCreated: payload sai hợp đồng (khóa: ['payload', 'version'])",
        "tạo chuyến TN-0005 (ride)",
        "cách ly: không phải envelope",
    ]
    expected_by_name = dict(zip((n for n, _ in EVENT_CASES), expected_events, strict=True))
    for name, envelope in EVENT_CASES:
        check(f"event: {name}", handle_event(envelope), expected_by_name[name])

    # bẫy cú pháp
    try:
        compile("def f(s):\n    match s:\n        case ACCEPTED:\n            return 1\n        case 2:\n            return 2\n",
                "<demo>", "exec")
        msg = "không lỗi"
    except SyntaxError as exc:
        msg = exc.msg
    check("tên trần trong case là SyntaxError", msg, "name capture 'ACCEPTED' makes remaining patterns unreachable")
    ns: dict[str, Any] = {}
    exec("def f(s):\n    match s:\n        case ACCEPTED:\n            return 'khớp'\n", ns)
    check("tên trần khớp mọi giá trị", (ns["f"]("a"), ns["f"](None)), ("khớp", "khớp"))
    sequence_hit = False
    match "cancel":
        case ("cancel",):
            sequence_hit = True
        case _:
            pass
    check("chuỗi không phải sequence pattern", sequence_hit, False)
    one_hit = False
    match True:
        case 1:
            one_hit = True
    check("literal 1 khớp True", one_hit, True)
    float_hit = False
    match 1.0:
        case 1:
            float_hit = True
    check("literal 1 khớp 1.0", float_hit, True)

    # class pattern theo vị trí (dataclass tự sinh __match_args__) và so sánh StrEnum với chuỗi
    check("__match_args__", Trip.__match_args__[:3], ("trip_id", "customer_id", "driver_id"))
    positional = ""
    match make_trip(S.SEARCHING, None, trip_id="TN-0009"):
        case Trip(trip_id, _, None):
            positional = trip_id
    check("class pattern theo vị trí", positional, "TN-0009")
    string_hit = False
    match "accepted":
        case TripStatus.ACCEPTED:
            string_hit = True
    check("chuỗi 'accepted' khớp TripStatus.ACCEPTED", string_hit, True)
    default_hit = False
    match 2:
        case 1:
            pass
        case _ if default_hit:
            pass
        case _:
            default_hit = True
    check("guard sai thì thử case kế tiếp", default_hit, True)

    # None
    check("truthy gộp rỗng với None", driver_label_truthy(make_trip(S.ACCEPTED, "")), "chưa có tài xế")
    check("match tách None", driver_label(make_trip(S.ACCEPTED, None)), "chưa có tài xế")
    check("match tách rỗng", driver_label(make_trip(S.ACCEPTED, "")), "dữ liệu lỗi")
    check("match chuỗi hợp lệ", driver_label(make_trip(S.ACCEPTED, "DR-001")), "DR-001")
    base = make_trip(S.ACCEPTED, "DR-001")
    check("patch không tham số giữ nguyên", patch_trip(base).driver_id, "DR-001")
    check("patch None bỏ tài xế", patch_trip(base, driver_id=None).driver_id, None)
    check("patch đổi tài xế", patch_trip(base, driver_id="DR-002").driver_id, "DR-002")
    check("patch status giữ driver", (patch_trip(base, status=S.ARRIVING).status, patch_trip(base, status=S.ARRIVING).driver_id),
          (S.ARRIVING, "DR-001"))
    check("TypeIs", (is_assigned(None), is_assigned("DR-001"), is_assigned("")), (False, True, True))
    check("TypeGuard", (has_usable_driver(None), has_usable_driver(""), has_usable_driver("DR-001")), (False, False, True))

    if failures:
        print(f"FAIL {total - len(failures)}/{total}")
        sys.exit(1)
    print(f"OK {total}/{total}")


def main() -> None:
    demo_describe()
    demo_transitions()
    demo_events()
    demo_traps()
    demo_none()


if __name__ == "__main__":
    if "--verify" in sys.argv[1:]:
        verify()
    else:
        main()
