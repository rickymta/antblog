"""Bài 01 - Python 3.14 và uv: chương trình tính cước đầu tiên của Tới Nơi.

Mô hình minh họa: không phải bảng giá vận hành thực tế.
Chạy:   uv run --project examples/toi-noi-python/01-python-va-uv examples/toi-noi-python/01-python-va-uv/main.py
Kiểm tra: thêm --verify ở cuối lệnh.
"""

import sys
from decimal import Decimal

# Quy tắc cước: 12.000 VND gồm 2 km đầu, mỗi km vượt thêm 4.500 VND.
BASE_FARE = Decimal("12000")
INCLUDED_KM = Decimal("2")
PRICE_PER_EXTRA_KM = Decimal("4500")


def calculate_fare(distance_km: Decimal) -> Decimal:
    """Tính cước cho một quãng đường. Bài 02 sẽ giải thích Decimal và làm tròn."""
    extra_km = max(Decimal("0"), distance_km - INCLUDED_KM)
    return BASE_FARE + extra_km * PRICE_PER_EXTRA_KM


def main() -> None:
    # Mặc định 5 km; có thể truyền khoảng cách khác làm đối số đầu tiên để tự kiểm bằng tay.
    distance_km = Decimal(sys.argv[1]) if len(sys.argv) > 1 else Decimal("5")
    total = calculate_fare(distance_km)
    print(f"Quãng đường: {distance_km} km")
    print(f"Cước minh họa: {total} VND")


def verify() -> None:
    """So các kết quả kỳ vọng in trong bài; thoát mã 1 khi có giá trị sai."""
    cases = [
        ("0", "12000"),
        ("2", "12000"),
        ("2.5", "14250.0"),
        ("5", "25500"),
        ("-1", "12000"),  # khoảng cách âm: chương trình vẫn chạy, kết quả vô nghĩa
    ]
    ok = 0
    for km, expected in cases:
        actual = str(calculate_fare(Decimal(km)))
        if actual == expected:
            ok += 1
        else:
            print(f"ca sai: {km} km -> {actual}, kỳ vọng {expected}")
    # Bài tập: giá km vượt 5.000, đi 4 km phải trả 22.000.
    exercise = BASE_FARE + max(Decimal("0"), Decimal("4") - INCLUDED_KM) * Decimal("5000")
    total_checks = len(cases) + 1
    if exercise == Decimal("22000"):
        ok += 1
    else:
        print(f"ca sai: bài tập -> {exercise}, kỳ vọng 22000")
    if ok != total_checks:
        print(f"FAIL {ok}/{total_checks}")
        sys.exit(1)
    print(f"OK {ok}/{total_checks}")


if __name__ == "__main__":
    if "--verify" in sys.argv[1:]:
        verify()
    else:
        main()
