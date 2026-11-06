"""Bài 11: class, dataclass, property và encapsulation - bảo vệ trạng thái chuyến.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Mô hình cước và máy trạng thái là giả định học tập. Trạng thái dùng chuỗi thường;
bài 13 mới đổi sang Enum.
"""

import sys
import tracemalloc
from dataclasses import dataclass, field
from datetime import UTC, datetime
from decimal import ROUND_HALF_UP, Decimal

# ---------- cước (rút gọn từ bài 03, cùng một công thức) ----------
BASE_FARE = Decimal("12000")
INCLUDED_KM = Decimal("2")
PRICE_PER_EXTRA_KM = Decimal("4500")
MAX_KM = Decimal("100")
SERVICES = ("ride", "food", "parcel")


def calculate_fare(distance_km: Decimal, *, service: str = "ride") -> Decimal:
    """Cước (VND), làm tròn ROUND_HALF_UP; ngoài miền 0..100 km thì ValueError."""
    if not Decimal("0") <= distance_km <= MAX_KM:
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    if service not in SERVICES:
        raise ValueError(f"service không hợp lệ: {service!r}")
    extra_km = max(Decimal("0"), distance_km - INCLUDED_KM)
    total = BASE_FARE + extra_km * PRICE_PER_EXTRA_KM
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)


# ---------- 1. Class thường: __init__ và self ----------
class TripPlain:
    """Phiên bản class thường. Mọi thuộc tính công khai, ai cũng gán được."""

    def __init__(self, trip_id: str, customer_id: str, service: str, distance_km: Decimal):
        # self là chính đối tượng đang được tạo; mỗi dòng gán tạo một thuộc tính riêng của nó.
        self.trip_id = trip_id
        self.customer_id = customer_id
        self.service = service
        self.distance_km = distance_km
        self.status = "requested"
        self.driver_id: str | None = None


# ---------- 2. Máy trạng thái: dict chuyển hợp lệ (dùng chuỗi, bài 13 đổi sang Enum) ----------
ALLOWED_TRANSITIONS: dict[str, tuple[str, ...]] = {
    "requested": ("searching", "cancelled"),
    "searching": ("accepted", "cancelled"),
    "accepted": ("arriving", "cancelled"),
    "arriving": ("picked_up", "cancelled"),
    "picked_up": ("in_transit",),
    "in_transit": ("completed",),
    "completed": (),
    "cancelled": (),
}


# ---------- 3. dataclass + property + __post_init__ ----------
@dataclass
class Trip:
    """Chuyến xe. Bên ngoài chỉ đọc status/driver_id/history; đổi trạng thái qua advance()."""

    # Thứ tự trường theo từ điển miền: trip_id, customer_id, driver_id, service, distance_km,
    # status, created_at. Trường init=False không nhận từ constructor: chuyến luôn bắt đầu
    # ở trạng thái đầu tiên, chưa có tài xế. Dấu _ đầu tên là quy ước «riêng»; Python không
    # chặn truy cập.
    trip_id: str
    customer_id: str
    _driver_id: str | None = field(default=None, init=False)
    service: str
    distance_km: Decimal
    _status: str = field(default="requested", init=False)
    created_at: datetime  # aware, UTC
    _history: list[str] = field(default_factory=lambda: ["requested"], init=False, repr=False)

    def __post_init__(self) -> None:
        # Chạy ngay sau __init__ do dataclass sinh: từ chối dữ liệu sai lúc tạo.
        if not self.trip_id.startswith("TN-"):
            raise ValueError(f"mã chuyến phải có dạng TN-0001: {self.trip_id!r}")
        if self.created_at.tzinfo is None:
            raise ValueError("created_at phải là datetime aware (UTC)")
        calculate_fare(self.distance_km, service=self.service)  # từ chối khoảng cách/dịch vụ sai

    @property
    def status(self) -> str:
        return self._status

    @property
    def driver_id(self) -> str | None:
        return self._driver_id

    @property
    def history(self) -> tuple[str, ...]:
        # Trả tuple (bản sao bất biến) để không lộ list nội bộ cho bên ngoài sửa.
        return tuple(self._history)

    @property
    def fare(self) -> Decimal:
        # Giá trị suy ra: tính theo yêu cầu nên không bao giờ lệch khỏi distance_km/service.
        return calculate_fare(self.distance_km, service=self.service)

    def advance(self, new_status: str, *, driver_id: str | None = None) -> None:
        """Chuyển trạng thái. Kiểm tra HẾT trước khi sửa: lỗi thì đối tượng không đổi."""
        if new_status not in ALLOWED_TRANSITIONS:
            raise ValueError(f"trạng thái không tồn tại: {new_status!r}")
        if new_status not in ALLOWED_TRANSITIONS[self._status]:
            raise ValueError(f"chuyến {self.trip_id}: không thể chuyển {self._status} -> {new_status}")
        if new_status == "accepted":
            if not driver_id or not driver_id.strip():
                raise ValueError(f"chuyến {self.trip_id}: nhận chuyến phải có driver_id")
        elif driver_id is not None:
            raise ValueError(f"driver_id chỉ đi kèm bước accepted, không đi kèm {new_status}")
        # Qua hết kiểm tra mới sửa.
        if new_status == "accepted":
            self._driver_id = driver_id
        self._status = new_status
        self._history.append(new_status)


# ---------- 4. __slots__: bỏ __dict__ của từng đối tượng ----------
@dataclass
class TripRow:
    trip_id: str
    customer_id: str
    service: str
    distance_km: Decimal
    status: str


@dataclass(slots=True)
class TripRowSlots:
    trip_id: str
    customer_id: str
    service: str
    distance_km: Decimal
    status: str


def bytes_per_instance(cls: type, count: int = 50_000) -> float:
    """Byte bộ nhớ cấp thêm cho mỗi đối tượng (tracemalloc), trừ phần con trỏ của chính
    cái list chứa. Các giá trị field dùng chung một object nên con số chỉ phản ánh
    phần khung đối tượng (đã gồm tiêu đề của bộ gom rác)."""
    trip_id, customer_id, service, km, status = "TN-0001", "CU-001", "ride", Decimal("2.5"), "requested"
    tracemalloc.start()
    before = tracemalloc.get_traced_memory()[0]
    rows = [cls(trip_id, customer_id, service, km, status) for _ in range(count)]
    after = tracemalloc.get_traced_memory()[0]
    tracemalloc.stop()
    return (after - before - sys.getsizeof(rows)) / count


# ---------- dữ liệu mẫu và demo ----------
T0 = datetime(2026, 11, 6, 1, 30, tzinfo=UTC)
HAPPY_PATH = ("searching", "accepted", "arriving", "picked_up", "in_transit", "completed")


def new_trip(trip_id: str = "TN-0001", km: str = "2.5", service: str = "ride") -> Trip:
    return Trip(trip_id, "CU-001", service, Decimal(km), T0)


def error_of(action) -> str:
    """Chạy action; trả «TênLỗi: thông điệp» hoặc 'không lỗi'."""
    try:
        action()
    except Exception as exc:  # noqa: BLE001 - demo muốn thấy mọi loại lỗi
        return f"{type(exc).__name__}: {exc}"
    return "không lỗi"


def mutable_default_error() -> str:
    """Định nghĩa dataclass có list mặc định: Python từ chối ngay lúc khai báo class."""

    def define() -> None:
        @dataclass
        class Bad:
            tags: list[str] = []  # type: ignore[assignment]  # noqa: RUF012

    return error_of(define)


def demo() -> None:
    print("== 1. class thường ==")
    a = TripPlain("TN-0001", "CU-001", "ride", Decimal("2.5"))
    b = TripPlain("TN-0001", "CU-001", "ride", Decimal("2.5"))
    print("a == b:", a == b, "(== mặc định so danh tính `is`, không so nội dung)")
    a.status = "completed"  # nhảy từ requested thẳng sang completed, không ai ngăn
    print("a.status sau khi gán thẳng:", a.status, "| a.driver_id:", a.driver_id)

    print("\n== 2. dataclass ==")
    t1, t2 = new_trip(), new_trip()
    print(t1)
    print("t1 == t2:", t1 == t2)
    print("hash(t1):", error_of(lambda: hash(t1)))
    print("list mặc định:", mutable_default_error())

    print("\n== 3. property ==")
    print("fare:", t1.fare, "| status:", t1.status, "| driver_id:", t1.driver_id)
    print("t1.fare = 1:", error_of(lambda: setattr(t1, "fare", 1)))
    print("t1.status = 'completed':", error_of(lambda: setattr(t1, "status", "completed")))
    t1._status = "completed"  # Python không chặn: «riêng» chỉ là quy ước
    print("sau t1._status = 'completed': status =", t1.status, "(quy ước, không phải khóa)")

    print("\n== 4. máy trạng thái ==")
    trip = new_trip("TN-0002")
    for step in HAPPY_PATH:
        trip.advance(step, driver_id="DR-001" if step == "accepted" else None)
    print("history:", " -> ".join(trip.history))
    bad = new_trip("TN-0003")
    print("requested -> completed:", error_of(lambda: bad.advance("completed")))
    print("trạng thái sau lỗi:", bad.status)
    print("accepted không driver:", error_of(lambda: _advance_to_accepted(bad)))
    print("trạng thái sau lỗi:", bad.status, "| driver_id:", bad.driver_id)
    print("trạng thái lạ:", error_of(lambda: bad.advance("flying")))

    print("\n== 5. __slots__ ==")
    plain_bytes = bytes_per_instance(TripRow)
    slots_bytes = bytes_per_instance(TripRowSlots)
    print(f"TripRow       : {plain_bytes:.0f} byte/đối tượng")
    print(f"TripRowSlots  : {slots_bytes:.0f} byte/đối tượng")
    print(f"tiết kiệm     : {100 * (1 - slots_bytes / plain_bytes):.0f}%")
    row = TripRowSlots("TN-0001", "CU-001", "ride", Decimal("2.5"), "requested")
    plain_row = TripRow("TN-0001", "CU-001", "ride", Decimal("2.5"), "requested")
    # sys.getsizeof chỉ đo khung đối tượng, không đo phần dữ liệu thuộc tính của bản không slots
    print("sys.getsizeof:", sys.getsizeof(plain_row), "(không slots) so với", sys.getsizeof(row), "(slots)")
    print("thêm thuộc tính lạ:", error_of(lambda: setattr(row, "note", "x")))


def _advance_to_accepted(trip: Trip) -> None:
    trip.advance("searching")
    trip.advance("accepted")  # thiếu driver_id


def verify() -> None:
    """So các kết quả kỳ vọng in trong bài; sai thì in FAIL và thoát với mã 1."""
    failures: list[str] = []
    total = 0

    def check(name: str, got: object, expected: object) -> None:
        nonlocal total
        total += 1
        if got != expected:
            failures.append(name)
            print(f"SAI: {name}: nhận {got!r}, kỳ vọng {expected!r}")

    # class thường: == so danh tính (is); gán thẳng bỏ qua mọi quy tắc
    a = TripPlain("TN-0001", "CU-001", "ride", Decimal("2.5"))
    b = TripPlain("TN-0001", "CU-001", "ride", Decimal("2.5"))
    check("class thường: == so danh tính (is)", a == b, False)
    a.status = "completed"
    check("class thường: gán thẳng status", a.status, "completed")

    # dataclass: __eq__ theo field, không hashable, list mặc định bị từ chối
    t1, t2 = new_trip(), new_trip()
    check("dataclass: == theo field", t1 == t2, True)
    check("dataclass: không hashable", error_of(lambda: hash(t1)), "TypeError: unhashable type: 'Trip'")
    check(
        "dataclass: list mặc định",
        mutable_default_error(),
        "ValueError: mutable default <class 'list'> for field tags is not allowed: use default_factory",
    )
    t2.advance("searching")
    check("dataclass: == gồm cả trạng thái", t1 == t2, False)

    # property: fare suy ra, chỉ đọc
    for km, expected in [("0", "12000"), ("2", "12000"), ("2.5", "14250"), ("5", "25500")]:
        check(f"fare {km} km", new_trip(km=km).fare, Decimal(expected))
    check("fare chỉ đọc", error_of(lambda: setattr(t1, "fare", 1)),
          "AttributeError: property 'fare' of 'Trip' object has no setter")
    check("status chỉ đọc", error_of(lambda: setattr(t1, "status", "completed")),
          "AttributeError: property 'status' of 'Trip' object has no setter")
    check("driver_id chỉ đọc", error_of(lambda: setattr(t1, "driver_id", "DR-009")),
          "AttributeError: property 'driver_id' of 'Trip' object has no setter")
    spare = new_trip("TN-0009")
    spare._status = "completed"  # quy ước, không phải khóa
    check("_status vẫn gán được (chỉ là quy ước)", spare.status, "completed")
    check("history là tuple", isinstance(t1.history, tuple), True)
    check("history không lộ list nội bộ", hasattr(t1.history, "append"), False)

    # máy trạng thái: đường đi trọn vẹn
    trip = new_trip("TN-0002")
    for step in HAPPY_PATH:
        trip.advance(step, driver_id="DR-001" if step == "accepted" else None)
    check("lịch sử đủ 7 trạng thái", trip.history, ("requested", *HAPPY_PATH))
    check("driver_id sau khi nhận", trip.driver_id, "DR-001")
    check("completed không còn đường ra", ALLOWED_TRANSITIONS["completed"], ())
    check("cancelled không còn đường ra", ALLOWED_TRANSITIONS["cancelled"], ())
    check("đủ 8 trạng thái", len(ALLOWED_TRANSITIONS), 8)
    check("mọi đích đều là trạng thái đã khai báo",
          all(dst in ALLOWED_TRANSITIONS for dsts in ALLOWED_TRANSITIONS.values() for dst in dsts), True)

    # chuyển sai: lỗi và không đổi trạng thái
    bad = new_trip("TN-0003")
    check("requested -> completed bị từ chối", error_of(lambda: bad.advance("completed")),
          "ValueError: chuyến TN-0003: không thể chuyển requested -> completed")
    check("lỗi không đổi trạng thái", (bad.status, bad.history), ("requested", ("requested",)))
    check("trạng thái lạ", error_of(lambda: bad.advance("flying")),
          "ValueError: trạng thái không tồn tại: 'flying'")
    check("accepted thiếu driver_id", error_of(lambda: _advance_to_accepted(bad)),
          "ValueError: chuyến TN-0003: nhận chuyến phải có driver_id")
    check("thiếu driver_id không đổi gì", (bad.status, bad.driver_id), ("searching", None))
    check("driver_id sai chỗ", error_of(lambda: bad.advance("cancelled", driver_id="DR-001")),
          "ValueError: driver_id chỉ đi kèm bước accepted, không đi kèm cancelled")

    # hủy: chỉ từ requested/searching/accepted/arriving
    cancellable = [s for s, dsts in ALLOWED_TRANSITIONS.items() if "cancelled" in dsts]
    check("4 trạng thái hủy được", cancellable, ["requested", "searching", "accepted", "arriving"])

    # __post_init__ từ chối dữ liệu sai lúc tạo
    check("mã chuyến sai", error_of(lambda: new_trip("XX-1")),
          "ValueError: mã chuyến phải có dạng TN-0001: 'XX-1'")
    check("datetime naive bị từ chối",
          error_of(lambda: Trip("TN-0001", "CU-001", "ride", Decimal("1"), datetime(2026, 11, 6))),
          "ValueError: created_at phải là datetime aware (UTC)")
    check("khoảng cách ngoài miền", error_of(lambda: new_trip(km="100.5")),
          "ValueError: khoảng cách ngoài miền 0..100 km: 100.5")

    # __slots__
    row = TripRowSlots("TN-0001", "CU-001", "ride", Decimal("2.5"), "requested")
    check("slots: không có __dict__", hasattr(row, "__dict__"), False)
    check("slots: thuộc tính lạ bị từ chối", error_of(lambda: setattr(row, "note", "x")),
          "AttributeError: 'TripRowSlots' object has no attribute 'note' and no __dict__ for setting new attributes")
    check("slots: vẫn có __eq__ theo field", row == TripRowSlots("TN-0001", "CU-001", "ride", Decimal("2.5"), "requested"), True)
    check("slots: nhỏ hơn không slots", bytes_per_instance(TripRowSlots) < bytes_per_instance(TripRow), True)

    if failures:
        print(f"FAIL {total - len(failures)}/{total}")
        sys.exit(1)
    print(f"OK {total}/{total}")


if __name__ == "__main__":
    if "--verify" in sys.argv[1:]:
        verify()
    else:
        demo()
