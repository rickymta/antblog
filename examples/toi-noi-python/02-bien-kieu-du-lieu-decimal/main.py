"""Bài 02 - Biến, kiểu dữ liệu và Decimal: biểu diễn tiền và khoảng cách.

Mô hình minh họa: không phải bảng giá vận hành thực tế.
Chạy:     uv run --project examples/toi-noi-python/02-bien-kieu-du-lieu-decimal examples/toi-noi-python/02-bien-kieu-du-lieu-decimal/main.py
Kiểm tra: thêm --verify ở cuối lệnh (so từng dòng in ra với EXPECTED).
"""

import sys
from decimal import ROUND_HALF_UP, Decimal, InvalidOperation

# Quy tắc cước: 12.000 VND gồm 2 km đầu, mỗi km vượt thêm 4.500 VND.
BASE_FARE = Decimal("12000")
INCLUDED_KM = Decimal("2")
PRICE_PER_EXTRA_KM = Decimal("4500")
ONE_VND = Decimal("1")


def raw_fare(distance_km: Decimal) -> Decimal:
    """Cước thô: giữ nguyên độ chính xác, chưa làm tròn."""
    extra_km = max(Decimal("0"), distance_km - INCLUDED_KM)
    return BASE_FARE + extra_km * PRICE_PER_EXTRA_KM


def calculate_fare(distance_km: Decimal) -> Decimal:
    """Tiền phải trả: làm tròn đến 1 VND, điểm giữa làm tròn ra xa số 0."""
    return raw_fare(distance_km).quantize(ONE_VND, rounding=ROUND_HALF_UP)


def parse_decimal(text: str) -> Decimal:
    """Đổi chuỗi ngoài hệ thống thành Decimal hữu hạn; mọi lỗi quy về ValueError."""
    try:
        value = Decimal(text)
    except InvalidOperation as exc:  # InvalidOperation KHÔNG phải ValueError
        raise ValueError(f"không phải số thập phân: {text!r}") from exc
    if not value.is_finite():  # Decimal chấp nhận cả 'NaN' và 'Infinity'
        raise ValueError(f"số không hữu hạn: {text!r}")
    return value


def report() -> list[str]:
    """Sinh toàn bộ các dòng kết quả in trong bài."""
    out: list[str] = []

    # 1. Biến là tên gắn với đối tượng; kiểu thuộc về đối tượng.
    trip_id = "TN-0001"
    out.append(f"1. {trip_id} -> {type(trip_id).__name__}")
    value = 5
    out.append(f"2. value = 5 -> {type(value).__name__}")
    value = "5"  # gắn lại cùng tên vào một đối tượng khác kiểu: hợp lệ trong Python
    out.append(f"3. value = '5' -> {type(value).__name__}")
    a = [1, 2]
    b = a  # b là tên thứ hai của cùng một đối tượng, không phải bản sao
    b.append(3)
    out.append(f"4. a={a}, a is b: {a is b}")

    # 2. int không giới hạn; phép chia luôn có hai dạng.
    out.append(f"5. 2**100 = {2**100}")
    out.append(f"6. 5 / 2 = {5 / 2}; 5 // 2 = {5 // 2}")

    # 3. float là nhị phân xấp xỉ; Decimal giữ chữ số thập phân.
    out.append(f"7. float: 0.1 + 0.2 = {0.1 + 0.2}; == 0.3 là {0.1 + 0.2 == 0.3}")
    dec_sum = Decimal("0.1") + Decimal("0.2")
    out.append(f"8. Decimal: 0.1 + 0.2 = {dec_sum}; == 0.3 là {dec_sum == Decimal('0.3')}")
    out.append(f"9. Decimal(0.1) = {Decimal(0.1)}")
    out.append(f"10. Decimal('0.1') = {Decimal('0.1')}")

    # 4. Làm tròn tiền: nêu rõ chế độ, không dựa vào mặc định.
    half = Decimal("14254.5")
    out.append(f"11. mặc định (HALF_EVEN): {half.quantize(ONE_VND)}")
    out.append(f"12. ROUND_HALF_UP: {half.quantize(ONE_VND, rounding=ROUND_HALF_UP)}")

    # 5. Chuyển đổi kiểu: dữ liệu ngoài hệ thống luôn là chuỗi.
    out.append(f"13. int('5') = {int('5')}; Decimal('4.5') = {Decimal('4.5')}")
    try:
        int("2.5")
    except ValueError as exc:
        out.append(f"14. int('2.5') -> ValueError: {exc}")
    try:
        Decimal("abc")
    except InvalidOperation:
        out.append("15. Decimal('abc') -> InvalidOperation")
    try:
        Decimal("1.5") + 0.5  # type: ignore[operator]
    except TypeError:
        out.append("16. Decimal + float -> TypeError")
    out.append(f"17. int(Decimal('2.9')) = {int(Decimal('2.9'))} (cắt về 0, không làm tròn)")

    # 6. bool, truthiness và None.
    falsy = [bool(0), bool(Decimal("0")), bool(""), bool([]), bool(None)]
    out.append(f"18. falsy: {falsy}")
    out.append(f"19. bool('False') = {bool('False')}")
    driver_id = None
    out.append(f"20. driver_id is None: {driver_id is None}")

    # 7. Cước 2,501 km: cước thô và tiền phải trả.
    d = Decimal("2.501")
    out.append(f"21. {d} km: raw={raw_fare(d)}, fare={calculate_fare(d)}")

    # 8. Bảng kiểm bằng tay.
    for number, km in enumerate(("0", "2", "2.5", "5"), start=22):
        out.append(f"{number}. {km} km -> {calculate_fare(Decimal(km))} VND")

    # 9. Cổng vào dữ liệu: gom lỗi parse về một loại ngoại lệ.
    out.append(f"26. InvalidOperation là ValueError: {issubclass(InvalidOperation, ValueError)}")
    for number, text in enumerate(("2.5", "abc", "NaN"), start=27):
        try:
            out.append(f"{number}. parse_decimal({text!r}) = {parse_decimal(text)}")
        except ValueError as exc:
            out.append(f"{number}. parse_decimal({text!r}) -> ValueError: {exc}")

    # Đáp án bài tập 1: 2,499 km.
    e = Decimal("2.499")
    out.append(f"30. {e} km: raw={raw_fare(e)}, fare={calculate_fare(e)}")
    return out


EXPECTED = [
    "1. TN-0001 -> str",
    "2. value = 5 -> int",
    "3. value = '5' -> str",
    "4. a=[1, 2, 3], a is b: True",
    "5. 2**100 = 1267650600228229401496703205376",
    "6. 5 / 2 = 2.5; 5 // 2 = 2",
    "7. float: 0.1 + 0.2 = 0.30000000000000004; == 0.3 là False",
    "8. Decimal: 0.1 + 0.2 = 0.3; == 0.3 là True",
    "9. Decimal(0.1) = 0.1000000000000000055511151231257827021181583404541015625",
    "10. Decimal('0.1') = 0.1",
    "11. mặc định (HALF_EVEN): 14254",
    "12. ROUND_HALF_UP: 14255",
    "13. int('5') = 5; Decimal('4.5') = 4.5",
    "14. int('2.5') -> ValueError: invalid literal for int() with base 10: '2.5'",
    "15. Decimal('abc') -> InvalidOperation",
    "16. Decimal + float -> TypeError",
    "17. int(Decimal('2.9')) = 2 (cắt về 0, không làm tròn)",
    "18. falsy: [False, False, False, False, False]",
    "19. bool('False') = True",
    "20. driver_id is None: True",
    "21. 2.501 km: raw=14254.500, fare=14255",
    "22. 0 km -> 12000 VND",
    "23. 2 km -> 12000 VND",
    "24. 2.5 km -> 14250 VND",
    "25. 5 km -> 25500 VND",
    "26. InvalidOperation là ValueError: False",
    "27. parse_decimal('2.5') = 2.5",
    "28. parse_decimal('abc') -> ValueError: không phải số thập phân: 'abc'",
    "29. parse_decimal('NaN') -> ValueError: số không hữu hạn: 'NaN'",
    "30. 2.499 km: raw=14245.500, fare=14246",
]


def verify() -> None:
    """So từng dòng in ra với kết quả in trong bài; thoát mã 1 khi có dòng sai."""
    actual = report()
    ok = 0
    for index, expected in enumerate(EXPECTED):
        got = actual[index] if index < len(actual) else "(thiếu dòng)"
        if got == expected:
            ok += 1
        else:
            print(f"ca sai, dòng {index + 1}: nhận '{got}', kỳ vọng '{expected}'")
    total = len(EXPECTED)
    if len(actual) != total:
        print(f"ca sai: số dòng in ra là {len(actual)}, kỳ vọng {total}")
        ok = min(ok, total - 1)
    if ok != total:
        print(f"FAIL {ok}/{total}")
        sys.exit(1)
    print(f"OK {ok}/{total}")


if __name__ == "__main__":
    if "--verify" in sys.argv[1:]:
        verify()
    else:
        print("\n".join(report()))
