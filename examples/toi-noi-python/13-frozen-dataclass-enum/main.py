"""Bài 13: frozen dataclass, Enum, equality và immutability - thiết kế Money, Quote.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Mô hình cước và thời hạn báo giá là giả định học tập, không phải bảng giá vận hành.
"""

import json
import sys
from dataclasses import FrozenInstanceError, dataclass, replace
from datetime import UTC, datetime, timedelta
from decimal import ROUND_HALF_UP, Decimal
from enum import Enum, StrEnum
from typing import NamedTuple
from zoneinfo import ZoneInfo

VN_TZ = ZoneInfo("Asia/Ho_Chi_Minh")
MAX_KM = Decimal("100")
QUOTE_TTL = timedelta(minutes=5)  # giả định học tập: báo giá sống 5 phút


# ---------- 1. Enum: tên cho một tập giá trị đóng ----------
class ServiceKind(StrEnum):
    RIDE = "ride"
    FOOD = "food"
    PARCEL = "parcel"


class TripStatus(StrEnum):
    """Định nghĩa chuẩn của 8 trạng thái chuyến (giá trị chữ thường)."""

    REQUESTED = "requested"
    SEARCHING = "searching"
    ACCEPTED = "accepted"
    ARRIVING = "arriving"
    PICKED_UP = "picked_up"
    IN_TRANSIT = "in_transit"
    COMPLETED = "completed"
    CANCELLED = "cancelled"

    # Enum cũng là class: gắn hành vi vào ngay tại trạng thái.
    def can_go_to(self, new_status: TripStatus) -> bool:
        return new_status in ALLOWED_TRANSITIONS[self]

    @property
    def is_final(self) -> bool:
        return not ALLOWED_TRANSITIONS[self]


# Cùng bảng chuyển của bài 11, nhưng khóa và giá trị là thành viên Enum thay vì chuỗi.
ALLOWED_TRANSITIONS: dict[TripStatus, tuple[TripStatus, ...]] = {
    TripStatus.REQUESTED: (TripStatus.SEARCHING, TripStatus.CANCELLED),
    TripStatus.SEARCHING: (TripStatus.ACCEPTED, TripStatus.CANCELLED),
    TripStatus.ACCEPTED: (TripStatus.ARRIVING, TripStatus.CANCELLED),
    TripStatus.ARRIVING: (TripStatus.PICKED_UP, TripStatus.CANCELLED),
    TripStatus.PICKED_UP: (TripStatus.IN_TRANSIT,),
    TripStatus.IN_TRANSIT: (TripStatus.COMPLETED,),
    TripStatus.COMPLETED: (),
    TripStatus.CANCELLED: (),
}


class PlainService(Enum):
    """Enum thường để đối chiếu: thành viên KHÔNG bằng chuỗi giá trị của nó."""

    RIDE = "ride"


# ---------- 2. NamedTuple: chính sách cước (giữ nguyên từ bài 10) ----------
class FarePolicy(NamedTuple):
    base_fare: Decimal = Decimal("12000")
    included_km: Decimal = Decimal("2")
    price_per_extra_km: Decimal = Decimal("4500")


STANDARD = FarePolicy()


def calculate_fare(
    distance_km: Decimal,
    policy: FarePolicy = STANDARD,
    *,
    service: ServiceKind = ServiceKind.RIDE,
) -> Decimal:
    """Cước (VND), làm tròn ROUND_HALF_UP; ngoài miền 0..100 km thì ValueError."""
    if not Decimal("0") <= distance_km <= MAX_KM:
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    ServiceKind(service)  # ép về Enum: dịch vụ lạ bị ValueError ngay tại đây
    extra_km = max(Decimal("0"), distance_km - policy.included_km)
    total = policy.base_fare + extra_km * policy.price_per_extra_km
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)


# ---------- 3. Frozen dataclass: value object Money ----------
@dataclass(frozen=True)
class Money:
    """Số tiền + loại tiền. Bất biến; bằng nhau khi cùng amount và currency."""

    amount: Decimal
    currency: str = "VND"

    def __post_init__(self) -> None:
        if not isinstance(self.amount, Decimal):
            raise TypeError(f"amount phải là Decimal, nhận {type(self.amount).__name__}")
        if not self.amount.is_finite():
            raise ValueError(f"amount phải hữu hạn: {self.amount}")
        if self.amount < 0:
            raise ValueError(f"amount không được âm: {self.amount}")
        if len(self.currency) != 3 or not (self.currency.isascii() and self.currency.isalpha() and self.currency.isupper()):
            raise ValueError(f"currency phải gồm 3 chữ in hoa: {self.currency!r}")
        if self.currency == "VND":
            if self.amount != self.amount.to_integral_value():
                raise ValueError(f"VND không có đồng lẻ: {self.amount}")
            # Chuẩn hóa 12000.0 -> 12000. Frozen cấm self.amount = ... kể cả trong
            # __post_init__, nên ta gán qua object.__setattr__.
            object.__setattr__(self, "amount", self.amount.quantize(Decimal("1")))

    def __add__(self, other: object) -> Money:
        if not isinstance(other, Money):
            return NotImplemented
        if other.currency != self.currency:
            raise ValueError(f"không cộng được {self.currency} với {other.currency}")
        return Money(self.amount + other.amount, self.currency)

    def __str__(self) -> str:
        if self.currency == "VND":
            return f"{self.amount:,.0f}".replace(",", ".") + " VND"
        return f"{self.amount} {self.currency}"


# ---------- 4. Frozen dataclass: Quote có hạn ----------
@dataclass(frozen=True)
class Quote:
    """Báo giá tại một thời điểm. Hết hạn thì phải báo lại, không sửa báo giá cũ."""

    trip_distance_km: Decimal
    fare: Money
    expires_at: datetime  # aware

    def __post_init__(self) -> None:
        if self.expires_at.tzinfo is None:
            raise ValueError("expires_at phải là datetime aware")
        if not Decimal("0") <= self.trip_distance_km <= MAX_KM:
            raise ValueError(f"khoảng cách ngoài miền 0..100 km: {self.trip_distance_km}")

    def is_valid(self, now: datetime) -> bool:
        """Còn hiệu lực khi now < expires_at. `now` truyền vào: bài 14 sẽ thay bằng Clock."""
        if now.tzinfo is None:
            raise ValueError("now phải là datetime aware")
        return now < self.expires_at

    @classmethod
    def issue(
        cls,
        distance_km: Decimal,
        *,
        issued_at: datetime,
        service: ServiceKind = ServiceKind.RIDE,
        ttl: timedelta = QUOTE_TTL,
    ) -> Quote:  # Python 3.14: annotation lười (PEP 649/749), không cần viết "Quote" trong ngoặc kép
        return cls(distance_km, Money(calculate_fare(distance_km, service=service)), issued_at + ttl)


# ---------- 5. Giới hạn của frozen: nông, và không chống cố tình ----------
@dataclass(frozen=True)
class LabelledQuote:
    """Frozen nhưng chứa list: bản thân field không gán lại được, nhưng list vẫn sửa được."""

    quote: Quote
    labels: list[str]


@dataclass(frozen=True)
class LabelledQuoteSafe:
    """Sửa bằng cách đổi list thành tuple: bất biến đến tận bên trong."""

    quote: Quote
    labels: tuple[str, ...]


# ---------- equality: NamedTuple so như tuple, dataclass so theo kiểu ----------
class PointNT(NamedTuple):
    x: int
    y: int


class SizeNT(NamedTuple):
    w: int
    h: int


@dataclass(frozen=True)
class PointDC:
    x: int
    y: int


@dataclass(frozen=True)
class SizeDC:
    w: int
    h: int


# ---------- demo ----------
T0 = datetime(2026, 11, 6, 1, 30, tzinfo=UTC)


def error_of(action) -> str:
    """Chạy action; trả «TênLỗi: thông điệp» hoặc 'không lỗi'."""
    try:
        action()
    except Exception as exc:  # noqa: BLE001 - demo muốn thấy mọi loại lỗi
        return f"{type(exc).__name__}: {exc}"
    return "không lỗi"


def demo() -> None:
    print("== 1. Enum ==")
    print("TripStatus('requested') is TripStatus.REQUESTED:", TripStatus("requested") is TripStatus.REQUESTED)
    print("TripStatus('flying'):", error_of(lambda: TripStatus("flying")))
    print("TripStatus.PICKED_UP == 'picked_up':", TripStatus.PICKED_UP == "picked_up")
    print("PlainService.RIDE == 'ride':", PlainService.RIDE == "ride")
    print("f-string:", f"{TripStatus.PICKED_UP}", "| name:", TripStatus.PICKED_UP.name)
    print("json.dumps:", json.dumps({"status": TripStatus.REQUESTED, "service": ServiceKind.FOOD}))
    print("số trạng thái:", len(TripStatus), "| trạng thái cuối:", [s.value for s in TripStatus if s.is_final])
    print("ACCEPTED -> ARRIVING:", TripStatus.ACCEPTED.can_go_to(TripStatus.ARRIVING),
          "| PICKED_UP -> CANCELLED:", TripStatus.PICKED_UP.can_go_to(TripStatus.CANCELLED))
    print("gán lại thành viên:", error_of(lambda: setattr(TripStatus, "REQUESTED", "x")))

    print("\n== 2. Money (frozen dataclass) ==")
    m = Money(Decimal("14250"))
    print("repr:", repr(m), "| str:", m)
    print("m.amount = 1:", error_of(lambda: setattr(m, "amount", Decimal("1"))))
    a, b = Money(Decimal("12000")), Money(Decimal("12000.0"))
    print("12000 == 12000.0:", a == b, "| cùng hash:", hash(a) == hash(b), "| repr b:", repr(b))
    print("is:", a is b, "| dùng làm khóa dict:", {a: "cơ bản"}[b])
    print("cộng:", Money(Decimal("12000")) + Money(Decimal("2250")))
    print("sum:", sum([Money(Decimal("12000")), Money(Decimal("14250"))], Money(Decimal("0"))))
    print("VND + USD:", error_of(lambda: Money(Decimal("1")) + Money(Decimal("1"), "USD")))
    print("âm:", error_of(lambda: Money(Decimal("-1"))))
    print("đồng lẻ:", error_of(lambda: Money(Decimal("1.5"))))
    print("float:", error_of(lambda: Money(1.5)))  # type: ignore[arg-type]
    print("replace hợp lệ:", replace(m, amount=Decimal("20000")), "| m vẫn là", m)
    print("replace sai:", error_of(lambda: replace(m, amount=Decimal("-5"))))

    print("\n== 3. Quote có hạn ==")
    q = Quote.issue(Decimal("2.5"), issued_at=T0)
    print("quote:", q.fare, "đến", q.expires_at.astimezone(VN_TZ).strftime("%H:%M:%S %z"))
    for label, delta in [("+4 phút 59 giây", timedelta(minutes=4, seconds=59)),
                         ("+5 phút đúng", timedelta(minutes=5)),
                         ("+5 phút 1 giây", timedelta(minutes=5, seconds=1))]:
        print(f"is_valid({label}):", q.is_valid(T0 + delta))
    print("now naive:", error_of(lambda: q.is_valid(datetime(2026, 11, 6, 1, 31))))
    copy = replace(q, expires_at=q.expires_at + QUOTE_TTL)
    print("replace chỉ tạo bản sao, giá vẫn là giá cũ:", copy.fare, "| hết hạn", copy.expires_at.astimezone(UTC).strftime("%H:%M"),
          "| bản gốc", q.expires_at.astimezone(UTC).strftime("%H:%M"))
    same_instant = datetime(2026, 11, 6, 8, 35, tzinfo=VN_TZ)  # = 01:35 UTC
    print("cùng thời điểm, khác múi giờ:", q == replace(q, expires_at=same_instant),
          "| cùng hash:", hash(q) == hash(replace(q, expires_at=same_instant)))

    print("\n== 4. NamedTuple hay frozen dataclass ==")
    base, included, per_km = STANDARD  # NamedTuple giải nén được
    print("giải nén:", base, included, per_km)
    print("STANDARD == tuple thường:", STANDARD == (Decimal("12000"), Decimal("2"), Decimal("4500")))
    print("PointNT(1, 2) == SizeNT(1, 2):", PointNT(1, 2) == SizeNT(1, 2))
    print("PointDC(1, 2) == SizeDC(1, 2):", PointDC(1, 2) == SizeDC(1, 2))
    print("STANDARD.base_fare = 1:", error_of(lambda: setattr(STANDARD, "base_fare", 1)))
    print("_replace:", STANDARD._replace(price_per_extra_km=Decimal("5000")).price_per_extra_km)
    print("tuple(Money):", error_of(lambda: tuple(m)))  # type: ignore[call-overload]

    print("\n== 5. Giới hạn của frozen ==")
    lq = LabelledQuote(q, ["ride"])
    lq.labels.append("priority")  # field không gán lại được, nhưng list bên trong vẫn sửa được
    print("labels sau append:", lq.labels)
    print("hash(LabelledQuote):", error_of(lambda: hash(lq)))
    print("replace dùng chung list:", replace(lq, quote=q).labels is lq.labels)
    safe = LabelledQuoteSafe(q, ("ride",))
    print("hash(LabelledQuoteSafe) chạy được:", isinstance(hash(safe), int))
    victim = Money(Decimal("1000"))
    object.__setattr__(victim, "amount", Decimal("-1"))  # vòng qua frozen, không ai chặn
    print("object.__setattr__ vòng qua frozen:", repr(victim))


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

    D = Decimal

    # Enum
    check("parse trả đúng thành viên", TripStatus("requested") is TripStatus.REQUESTED, True)
    check("trạng thái lạ", error_of(lambda: TripStatus("flying")), "ValueError: 'flying' is not a valid TripStatus")
    check("StrEnum == chuỗi", TripStatus.PICKED_UP == "picked_up", True)
    check("Enum thường != chuỗi", PlainService.RIDE == "ride", False)
    check("f-string ra giá trị", f"{TripStatus.PICKED_UP}", "picked_up")
    check("json.dumps StrEnum", json.dumps({"s": TripStatus.REQUESTED}), '{"s": "requested"}')
    check("đủ 8 trạng thái", len(TripStatus), 8)
    check("giá trị chữ thường theo thứ tự", [s.value for s in TripStatus],
          ["requested", "searching", "accepted", "arriving", "picked_up", "in_transit", "completed", "cancelled"])
    check("trạng thái cuối", [s.value for s in TripStatus if s.is_final], ["completed", "cancelled"])
    check("bốn trạng thái hủy được",
          [s.value for s in TripStatus if s.can_go_to(TripStatus.CANCELLED)],
          ["requested", "searching", "accepted", "arriving"])
    check("khóa chuỗi vẫn tra được bảng chuyển", ALLOWED_TRANSITIONS["requested"],  # type: ignore[index]
          (TripStatus.SEARCHING, TripStatus.CANCELLED))
    check("không gán lại thành viên", error_of(lambda: setattr(TripStatus, "REQUESTED", "x")),
          "AttributeError: cannot reassign member 'REQUESTED'")
    check("dịch vụ lạ", error_of(lambda: ServiceKind("taxi")), "ValueError: 'taxi' is not a valid ServiceKind")

    # Money
    m = Money(D("14250"))
    check("repr Money", repr(m), "Money(amount=Decimal('14250'), currency='VND')")
    check("str Money", str(m), "14.250 VND")
    check("frozen: không gán field", error_of(lambda: setattr(m, "amount", D("1"))),
          "FrozenInstanceError: cannot assign to field 'amount'")
    a, b = Money(D("12000")), Money(D("12000.0"))
    check("12000 == 12000.0", a == b, True)
    check("hash bằng nhau", hash(a) == hash(b), True)
    check("chuẩn hóa amount", repr(b), "Money(amount=Decimal('12000'), currency='VND')")
    check("dùng làm khóa dict", {a: "cơ bản"}[b], "cơ bản")
    check("khác currency thì khác", Money(D("1"), "VND") == Money(D("1"), "USD"), False)
    check("cộng", Money(D("12000")) + Money(D("2250")), Money(D("14250")))
    check("sum có giá trị đầu",
          sum([Money(D("12000")), Money(D("14250"))], Money(D("0"))), Money(D("26250")))
    check("cộng khác currency", error_of(lambda: Money(D("1")) + Money(D("1"), "USD")),
          "ValueError: không cộng được VND với USD")
    check("số âm", error_of(lambda: Money(D("-1"))), "ValueError: amount không được âm: -1")
    check("đồng lẻ", error_of(lambda: Money(D("1.5"))), "ValueError: VND không có đồng lẻ: 1.5")
    check("float bị từ chối", error_of(lambda: Money(1.5)),  # type: ignore[arg-type]
          "TypeError: amount phải là Decimal, nhận float")
    check("NaN bị từ chối", error_of(lambda: Money(D("NaN"))), "ValueError: amount phải hữu hạn: NaN")
    check("currency sai", error_of(lambda: Money(D("1"), "vnd")), "ValueError: currency phải gồm 3 chữ in hoa: 'vnd'")
    check("currency có ký hiệu", error_of(lambda: Money(D("1"), "U$D")), "ValueError: currency phải gồm 3 chữ in hoa: 'U$D'")
    check("replace tạo bản mới", replace(m, amount=D("20000")), Money(D("20000")))
    check("replace không đổi bản gốc", m, Money(D("14250")))
    check("replace chạy lại validation", error_of(lambda: replace(m, amount=D("-5"))),
          "ValueError: amount không được âm: -5")
    check("Money + int", error_of(lambda: m + 1),  # type: ignore[operator]
          "TypeError: unsupported operand type(s) for +: 'Money' and 'int'")

    # Quote
    q = Quote.issue(D("2.5"), issued_at=T0)
    check("quote fare 2,5 km", q.fare, Money(D("14250")))
    check("quote hết hạn sau 5 phút", q.expires_at, T0 + timedelta(minutes=5))
    check("còn hạn lúc +4:59", q.is_valid(T0 + timedelta(minutes=4, seconds=59)), True)
    check("hết hạn đúng lúc +5:00", q.is_valid(T0 + timedelta(minutes=5)), False)
    check("hết hạn lúc +5:01", q.is_valid(T0 + timedelta(minutes=5, seconds=1)), False)
    check("now naive", error_of(lambda: q.is_valid(datetime(2026, 11, 6, 1, 31))),
          "ValueError: now phải là datetime aware")
    check("expires_at naive", error_of(lambda: Quote(D("1"), Money(D("12000")), datetime(2026, 11, 6))),
          "ValueError: expires_at phải là datetime aware")
    check("khoảng cách ngoài miền", error_of(lambda: Quote.issue(D("100.5"), issued_at=T0)),
          "ValueError: khoảng cách ngoài miền 0..100 km: 100.5")
    check("quote frozen", error_of(lambda: setattr(q, "fare", Money(D("1")))),
          "FrozenInstanceError: cannot assign to field 'fare'")
    copy = replace(q, expires_at=q.expires_at + QUOTE_TTL)
    check("replace tạo bản sao, bản gốc không đổi", (copy.expires_at, q.expires_at),
          (T0 + timedelta(minutes=10), T0 + timedelta(minutes=5)))
    check("replace giữ nguyên giá cũ", copy.fare, q.fare)
    same_instant = replace(q, expires_at=datetime(2026, 11, 6, 8, 35, tzinfo=VN_TZ))
    check("cùng thời điểm khác múi giờ: bằng nhau", q == same_instant, True)
    check("cùng thời điểm khác múi giờ: cùng hash", hash(q) == hash(same_instant), True)
    for km, expected in [("0", "12000"), ("2", "12000"), ("2.5", "14250"), ("5", "25500")]:
        check(f"cước {km} km", Quote.issue(D(km), issued_at=T0).fare.amount, D(expected))

    # NamedTuple và dataclass
    check("giải nén NamedTuple", tuple(STANDARD), (D("12000"), D("2"), D("4500")))
    check("NamedTuple == tuple thường", STANDARD == (D("12000"), D("2"), D("4500")), True)
    check("NamedTuple khác kiểu vẫn bằng", PointNT(1, 2) == SizeNT(1, 2), True)
    check("dataclass khác kiểu thì khác", PointDC(1, 2) == SizeDC(1, 2), False)
    check("NamedTuple bất biến", error_of(lambda: setattr(STANDARD, "base_fare", 1)),
          "AttributeError: can't set attribute")
    check("_replace", STANDARD._replace(price_per_extra_km=D("5000")),
          FarePolicy(D("12000"), D("2"), D("5000")))
    check("chính sách thay thế: 4 km", calculate_fare(D("4"), STANDARD._replace(price_per_extra_km=D("5000"))), D("22000"))
    check("Money không lặp được", error_of(lambda: tuple(m)),  # type: ignore[call-overload]
          "TypeError: 'Money' object is not iterable")

    # giới hạn của frozen
    lq = LabelledQuote(q, ["ride"])
    lq.labels.append("priority")
    check("list trong frozen vẫn sửa được", lq.labels, ["ride", "priority"])
    check("replace là bản sao nông", replace(lq, quote=q).labels is lq.labels, True)
    check("frozen chứa list không hash được", error_of(lambda: hash(lq)), "TypeError: unhashable type: 'list'")
    check("frozen chứa tuple hash được", isinstance(hash(LabelledQuoteSafe(q, ("ride",))), int), True)
    victim = Money(D("1000"))
    object.__setattr__(victim, "amount", D("-1"))
    check("object.__setattr__ vòng qua frozen", victim.amount, D("-1"))

    if failures:
        print(f"FAIL {total - len(failures)}/{total}")
        sys.exit(1)
    print(f"OK {total}/{total}")


if __name__ == "__main__":
    if "--verify" in sys.argv[1:]:
        verify()
    else:
        demo()
