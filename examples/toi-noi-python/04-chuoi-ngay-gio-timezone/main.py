"""Bài 04: chuỗi, f-string, ngày giờ và múi giờ - hiển thị đúng, lưu đúng.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Thời điểm trong ví dụ là cố định để kết quả tái lập; dữ liệu là giả.
"""

import html
import sys
import tempfile
import unicodedata
import warnings
from datetime import UTC, date, datetime, timedelta, timezone
from decimal import ROUND_HALF_UP, Decimal
from pathlib import Path
from string.templatelib import Interpolation, Template
from zoneinfo import ZoneInfo

APP_NAME = "Tới Nơi"
VN_TZ = ZoneInfo("Asia/Ho_Chi_Minh")


# ---------- str, bytes, Unicode ----------
def text_vs_bytes(text: str) -> tuple[int, int, bytes]:
    """Trả về (số ký tự, số byte UTF-8, bytes). str đếm ký tự, bytes đếm byte."""
    raw = text.encode("utf-8")
    return len(text), len(raw), raw


def normalized_forms(text: str) -> tuple[bool, str]:
    """NFC và NFD là hai cách biểu diễn cùng một chữ có dấu.

    Trả về (hai dạng có bằng nhau không, chuỗi sau khi chuẩn hóa về NFC).
    """
    nfd = unicodedata.normalize("NFD", text)
    nfc = unicodedata.normalize("NFC", nfd)
    return text == nfd, nfc


def decode_ascii_strict(raw: bytes) -> str:
    """errors='strict' (mặc định): byte không hợp lệ làm decode ném UnicodeDecodeError."""
    try:
        return raw.decode("ascii")
    except UnicodeDecodeError as exc:
        return f"UnicodeDecodeError: {exc.reason}"


def file_roundtrip(text: str) -> str:
    """Ghi rồi đọc lại file văn bản. Luôn nêu encoding='utf-8' ở cả hai phía."""
    with tempfile.TemporaryDirectory() as folder:
        path = Path(folder) / "note.txt"
        path.write_text(text, encoding="utf-8")
        return path.read_text(encoding="utf-8")


# ---------- f-string và số tiền VND ----------
def format_vnd(amount: Decimal) -> str:
    """14250 -> '14.250'. Làm tròn HALF_UP trước, rồi đổi dấu phẩy thành dấu chấm.

    Phải quantize trước: spec `.0f` làm tròn theo context của Decimal (HALF_EVEN),
    nên 12004.5 sẽ ra '12,004'. Không dùng module locale: kết quả phụ thuộc máy chạy.
    """
    rounded = amount.quantize(Decimal("1"), rounding=ROUND_HALF_UP)
    return f"{rounded:,.0f}".replace(",", ".")


def receipt_line(label: str, amount: Decimal, width: int = 32) -> str:
    """Căn lề bằng format spec: nhãn căn trái, số tiền căn phải."""
    money = format_vnd(amount) + " VND"
    return f"{label:<{width - 12}}{money:>12}"


def format_spec_examples() -> list[tuple[str, str]]:
    """Một số format spec thường dùng: (spec, kết quả)."""
    n = 14250
    price = Decimal("2.5")
    return [
        ("{n:,}", f"{n:,}"),
        ("{n:_}", f"{n:_}"),
        ("{n:>10,}", f"{n:>10,}"),
        ("{n:+,}", f"{n:+,}"),
        ("{price:.2f}", f"{price:.2f}"),
        ("{APP_NAME:*^11}", f"{APP_NAME:*^11}"),
        ("{APP_NAME!r}", f"{APP_NAME!r}"),
        ("{n=}", f"{n=}"),
        ("{{n}}", f"{{n}}"),
    ]


# ---------- strip / split / join ----------
def parse_trip_line(line: str) -> tuple[str, str, Decimal]:
    """'  TN-0001 ; ride ; 5.0  ' -> ('TN-0001', 'ride', Decimal('5.0')).

    strip bỏ khoảng trắng hai đầu, split tách theo dấu ';'. Mỗi bước tạo str mới.
    """
    parts = [p.strip() for p in line.strip().split(";")]
    if len(parts) != 3:
        raise ValueError(f"cần 3 trường, nhận {len(parts)}: {line!r}")
    trip_id, service, km = parts
    return trip_id, service, Decimal(km)


def join_trip_ids(ids: list[str]) -> str:
    return ", ".join(ids)


# ---------- datetime: naive, aware, UTC, múi giờ ----------
def require_aware(value: datetime) -> datetime:
    """Một chỗ kiểm duy nhất: datetime phải aware (có tzinfo và utcoffset)."""
    if value.tzinfo is None or value.utcoffset() is None:
        raise ValueError("cần datetime aware; datetime naive không biết mình ở múi giờ nào")
    return value


def now_utc() -> datetime:
    """'Bây giờ' đúng cách: aware, UTC. (timezone.utc và UTC là cùng một đối tượng.)"""
    return datetime.now(timezone.utc)


def to_local(instant: datetime) -> datetime:
    """Đổi một thời điểm aware sang giờ Việt Nam. Từ chối datetime naive."""
    return require_aware(instant).astimezone(VN_TZ)


def to_storage(instant: datetime) -> str:
    """Chuỗi ISO 8601 theo quy ước lưu UTC: luôn kết thúc bằng '+00:00'."""
    return require_aware(instant).astimezone(UTC).isoformat()


def from_storage(text: str) -> datetime:
    """Đọc lại chuỗi ISO 8601 có offset. Kết quả là datetime aware."""
    return require_aware(datetime.fromisoformat(text))


def duration_text(start: datetime, end: datetime) -> str:
    """Thời lượng chuyến: 'X phút Y giây'. Phép trừ hai datetime aware cho timedelta."""
    delta = require_aware(end) - require_aware(start)
    if delta < timedelta(0):
        raise ValueError("thời điểm kết thúc trước thời điểm bắt đầu")
    minutes, seconds = divmod(int(delta.total_seconds()), 60)
    return f"{minutes} phút {seconds} giây"


def naive_compare_fails() -> bool:
    """So sánh thứ tự (<) giữa naive và aware ném TypeError."""
    aware = datetime(2026, 11, 6, 1, 30, tzinfo=UTC)
    naive = datetime(2026, 11, 6, 1, 30)
    try:
        aware < naive  # noqa: B015 - cố ý so sánh để quan sát lỗi
    except TypeError:
        return True
    return False


def naive_equals_aware() -> bool:
    """Nhưng `==` giữa naive và aware KHÔNG ném lỗi: luôn trả False, âm thầm."""
    aware = datetime(2026, 11, 6, 1, 30, tzinfo=UTC)
    naive = datetime(2026, 11, 6, 1, 30)
    return aware == naive


def utcnow_is_deprecated() -> bool:
    """datetime.utcnow() trả về datetime NAIVE và phát DeprecationWarning."""
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        value = datetime.utcnow()  # noqa: DTZ003 - cố ý gọi để quan sát cảnh báo
    return value.tzinfo is None and any(
        issubclass(w.category, DeprecationWarning) for w in caught
    )


def attach_vs_convert() -> tuple[str, str]:
    """Gắn nhãn múi giờ (replace) khác với đổi múi giờ (astimezone).

    Người dùng gõ 08:30 giờ Việt Nam (naive). Đúng: gắn nhãn VN_TZ bằng replace.
    Sai: gắn nhãn UTC rồi astimezone, vì nhãn UTC đã nói dối về giờ gốc.
    """
    typed = datetime(2026, 11, 6, 8, 30)  # naive: người dùng gõ giờ địa phương
    right = typed.replace(tzinfo=VN_TZ).astimezone(UTC)
    wrong = typed.replace(tzinfo=UTC).astimezone(VN_TZ)
    return right.isoformat(), wrong.isoformat()


# ---------- hiển thị và đọc giờ địa phương ----------
def format_local(instant: datetime) -> str:
    """Hiển thị cho khách: dd/mm/yyyy HH:MM theo giờ Việt Nam (spec datetime của f-string)."""
    local = to_local(instant)
    return f"{local:%d/%m/%Y %H:%M}"


def parse_local_input(text: str) -> datetime:
    """Đọc giờ khách gõ 'dd/mm/yyyy HH:MM' (giờ Việt Nam) thành datetime aware."""
    naive = datetime.strptime(text, "%d/%m/%Y %H:%M")
    return naive.replace(tzinfo=VN_TZ)


def local_date(instant: datetime) -> date:
    """Ngày theo giờ Việt Nam: đổi múi giờ TRƯỚC khi lấy .date()."""
    return to_local(instant).date()


SAMPLE_TRIPS = [
    ("TN-0001", datetime(2026, 11, 5, 23, 30, tzinfo=UTC)),
    ("TN-0002", datetime(2026, 11, 5, 17, 0, tzinfo=UTC)),
    ("TN-0003", datetime(2026, 11, 5, 16, 59, tzinfo=UTC)),
]


def trip_date_rows() -> list[tuple[str, str, str, str, str]]:
    """(mã chuyến, UTC, giờ VN, ngày UTC, ngày VN) cho từng chuyến mẫu."""
    rows = []
    for trip_id, instant in SAMPLE_TRIPS:
        rows.append((
            trip_id,
            instant.strftime("%d/%m %H:%M"),
            to_local(instant).strftime("%d/%m %H:%M"),
            instant.date().isoformat(),
            local_date(instant).isoformat(),
        ))
    return rows


# ---------- template string (t-string, PEP 750, Python 3.14) ----------
def render_html(template: Template) -> str:
    """Hàm xử lý t-string: escape HTML cho phần nội suy, giữ nguyên phần văn bản tĩnh."""
    out: list[str] = []
    for item in template:
        if isinstance(item, Interpolation):
            out.append(html.escape(str(item.value)))
        else:
            out.append(item)
    return "".join(out)


# ---------- chương trình minh họa ----------
def main() -> None:
    n_chars, n_bytes, raw = text_vs_bytes(APP_NAME)
    print(f"{APP_NAME!r}: {n_chars} ký tự, {n_bytes} byte UTF-8")
    print(f"bytes: {raw!r}; raw[0] = {raw[0]} (int, không phải str)")
    print(f"decode UTF-8: {raw.decode('utf-8')}")
    print(f"decode sai (latin-1): {raw.decode('latin-1')!r}")
    print(f"decode ascii strict: {decode_ascii_strict(raw)}")
    print(f"decode ascii replace: {raw.decode('ascii', errors='replace')!r}")
    print(f"casefold: {APP_NAME.casefold()}; file round-trip: {file_roundtrip(APP_NAME)}")
    same, nfc = normalized_forms(APP_NAME)
    print(f"NFC == NFD? {same}; NFC: {nfc}")
    print()

    total = Decimal("14250")
    print(f"{{total:,.0f}} -> {total:,.0f}")
    print(f"{{Decimal('12004.5'):,.0f}} -> {Decimal('12004.5'):,.0f}")
    print(f"format_vnd(12004.5) -> {format_vnd(Decimal('12004.5'))}")
    print(f"format_vnd -> {format_vnd(total)}")
    print(receipt_line("Cước 2,5 km", total))
    print(receipt_line("Cước 5 km", Decimal("25500")))
    for spec, result in format_spec_examples():
        print(f"  {spec:<16} -> {result!r}")
    print()

    trip_id, service, km = parse_trip_line("  TN-0001 ; ride ; 5.0  ")
    print(f"parse: {trip_id!r} {service!r} {km}")
    print("join:", join_trip_ids(["TN-0001", "TN-0002", "TN-0003"]))
    print()

    start_utc = datetime(2026, 11, 6, 1, 30, tzinfo=UTC)
    local = to_local(start_utc)
    print("UTC   :", start_utc.isoformat())
    print("+07:00:", local.isoformat())
    print("cùng thời điểm?", start_utc == local)
    print("lưu    :", to_storage(local))
    print("đọc lại:", from_storage(to_storage(local)).isoformat())
    print("hiển thị:", format_local(start_utc))
    print("đọc giờ khách gõ:", parse_local_input("06/11/2026 08:30").isoformat())
    end_utc = start_utc + timedelta(minutes=22, seconds=15)
    print("thời lượng:", duration_text(start_utc, end_utc))
    print()

    print("naive < aware lỗi?", naive_compare_fails())
    print("naive == aware là:", naive_equals_aware())
    print("utcnow() deprecated và naive?", utcnow_is_deprecated())
    right, wrong = attach_vs_convert()
    print("08:30 gắn VN_TZ rồi đổi UTC:", right)
    print("08:30 gắn UTC rồi đổi VN   :", wrong)
    print()

    print("mã       UTC          giờ VN       ngày UTC    ngày VN")
    for trip, utc_text, vn_text, utc_day, vn_day in trip_date_rows():
        print(f"{trip}  {utc_text:<11}  {vn_text:<11}  {utc_day}  {vn_day}")
    print()

    customer = "<b>An</b>"
    print("t-string:", render_html(t"Khách: {customer}"))


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

    start_utc = datetime(2026, 11, 6, 1, 30, tzinfo=UTC)
    local = to_local(start_utc)
    customer = "<b>An</b>"
    same, nfc = normalized_forms(APP_NAME)
    raw = text_vs_bytes(APP_NAME)[2]

    # str, bytes, Unicode
    check("độ dài str và bytes", text_vs_bytes(APP_NAME)[:2], (7, 10))
    check("decode UTF-8 khớp", raw.decode("utf-8"), APP_NAME)
    check("index bytes trả int", raw[0], 84)
    check("latin-1 làm hỏng chữ", raw.decode("latin-1"), "Tá»\x9bi NÆ¡i")
    check("ascii strict ném lỗi", decode_ascii_strict(raw),
          "UnicodeDecodeError: ordinal not in range(128)")
    check("ascii replace", raw.decode("ascii", errors="replace"), "T���i N��i")
    check("casefold", APP_NAME.casefold(), "tới nơi")
    check("file round-trip UTF-8", file_roundtrip(APP_NAME), APP_NAME)
    check("NFC khác NFD", same, False)
    check("chuẩn hóa về NFC", nfc, APP_NAME)

    # f-string, format spec
    check("spec .0f làm tròn HALF_EVEN", f"{Decimal('12004.5'):,.0f}", "12,004")
    check("format_vnd 14250", format_vnd(Decimal("14250")), "14.250")
    check("format_vnd 453000", format_vnd(Decimal("453000")), "453.000")
    check("format_vnd 12004.5", format_vnd(Decimal("12004.5")), "12.005")
    check("format_vnd 1234567.5 (bài tập)", format_vnd(Decimal("1234567.5")), "1.234.568")
    check("receipt_line", receipt_line("Cước", Decimal("12000")), "Cước" + " " * 18 + "12.000 VND")
    check("format spec", dict(format_spec_examples()), {
        "{n:,}": "14,250", "{n:_}": "14_250", "{n:>10,}": "    14,250", "{n:+,}": "+14,250",
        "{price:.2f}": "2.50", "{APP_NAME:*^11}": "**Tới Nơi**", "{APP_NAME!r}": "'Tới Nơi'",
        "{n=}": "n=14250", "{{n}}": "{n}",
    })

    # strip, split, join
    check("parse_trip_line", parse_trip_line("  TN-0001 ; ride ; 5.0  "),
          ("TN-0001", "ride", Decimal("5.0")))
    check("join", join_trip_ids(["TN-0001", "TN-0002"]), "TN-0001, TN-0002")

    # datetime
    check("timezone.utc là UTC", timezone.utc is UTC, True)
    check("now_utc là aware UTC", now_utc().utcoffset(), timedelta(0))
    check("UTC -> +07:00", local.isoformat(), "2026-11-06T08:30:00+07:00")
    check("cùng instant", start_utc == local, True)
    check("offset +07:00", local.utcoffset(), timedelta(hours=7))
    check("lưu UTC", to_storage(local), "2026-11-06T01:30:00+00:00")
    check("round-trip", from_storage(to_storage(local)) == start_utc, True)
    check("hiển thị dd/mm/yyyy", format_local(start_utc), "06/11/2026 08:30")
    check("đọc giờ khách gõ", parse_local_input("06/11/2026 08:30").isoformat(),
          "2026-11-06T08:30:00+07:00")
    check("thời lượng", duration_text(start_utc, start_utc + timedelta(minutes=22, seconds=15)),
          "22 phút 15 giây")
    check("bài tập: 23:30 UTC -> giờ VN",
          to_local(from_storage("2026-11-06T23:30:00+00:00")).isoformat(),
          "2026-11-07T06:30:00+07:00")
    check("naive < aware lỗi", naive_compare_fails(), True)
    check("naive == aware là False", naive_equals_aware(), False)
    check("utcnow deprecated", utcnow_is_deprecated(), True)
    check("replace vs astimezone", attach_vs_convert(),
          ("2026-11-06T01:30:00+00:00", "2026-11-06T15:30:00+07:00"))
    check("bảng ngày chuyến", trip_date_rows(), [
        ("TN-0001", "05/11 23:30", "06/11 06:30", "2026-11-05", "2026-11-06"),
        ("TN-0002", "05/11 17:00", "06/11 00:00", "2026-11-05", "2026-11-06"),
        ("TN-0003", "05/11 16:59", "05/11 23:59", "2026-11-05", "2026-11-05"),
    ])

    # t-string
    check("t-string escape", render_html(t"Khách: {customer}"), "Khách: &lt;b&gt;An&lt;/b&gt;")

    # Các ca phải bị từ chối bằng ValueError.
    rejects = [
        ("to_local từ chối naive", lambda: to_local(datetime(2026, 11, 6, 8, 30))),
        ("to_storage từ chối naive", lambda: to_storage(datetime(2026, 11, 6, 8, 30))),
        ("from_storage từ chối thiếu offset", lambda: from_storage("2026-11-06T08:30:00")),
    ]
    for name, call in rejects:
        try:
            call()
            outcome = "không lỗi"
        except ValueError:
            outcome = "ValueError"
        check(name, outcome, "ValueError")

    if failures:
        print(f"FAIL {total - len(failures)}/{total}")
        sys.exit(1)
    print(f"OK {total}/{total}")


if __name__ == "__main__":
    if "--verify" in sys.argv[1:]:
        verify()
    else:
        main()
