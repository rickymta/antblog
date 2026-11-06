"""Bài 03: điều kiện, vòng lặp và hàm - tách quy tắc tính cước.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Mô hình cước là giả định học tập, không phải bảng giá vận hành của Tới Nơi:
cả ba dịch vụ (ride, food, parcel) dùng cùng một công thức giá.
"""

import sys
from decimal import ROUND_HALF_UP, Decimal

# Hằng số của quy tắc cước (giả định học tập).
BASE_FARE = Decimal("12000")  # giá cơ bản, đã gồm INCLUDED_KM đầu
INCLUDED_KM = Decimal("2")
PRICE_PER_EXTRA_KM = Decimal("4500")
MIN_KM = Decimal("0")
MAX_KM = Decimal("100")

# Ba loại dịch vụ của Tới Nơi. Ở mô hình học tập này chúng cùng giá.
SERVICES = ("ride", "food", "parcel")


def fare_breakdown(
    distance_km: Decimal, *, service: str = "ride"
) -> tuple[Decimal, Decimal]:
    """Trả về (giá cơ bản, tiền phần vượt), chưa làm tròn.

    Hàm chỉ tính và trả kết quả, không in gì ra màn hình.
    Ném ValueError khi khoảng cách hoặc dịch vụ nằm ngoài miền cho phép.
    """
    # Chained comparison: đọc như toán học, 0 <= km <= 100.
    if not MIN_KM <= distance_km <= MAX_KM:
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    if service not in SERVICES:
        raise ValueError(f"service không hợp lệ: {service!r}")

    if distance_km <= INCLUDED_KM:
        extra = Decimal("0")
    else:
        extra = (distance_km - INCLUDED_KM) * PRICE_PER_EXTRA_KM
    return BASE_FARE, extra


def calculate_fare(distance_km: Decimal, *, service: str = "ride") -> Decimal:
    """Tính cước (VND), làm tròn ROUND_HALF_UP về 1 đồng.

    `service` là keyword-only: bắt buộc gọi calculate_fare(d, service="food").
    """
    base, extra = fare_breakdown(distance_km, service=service)
    return (base + extra).quantize(Decimal("1"), rounding=ROUND_HALF_UP)


def format_vnd(amount: Decimal) -> str:
    """12000 -> '12.000'. Làm tròn HALF_UP trước vì spec `.0f` làm tròn HALF_EVEN.

    Dùng dấu chấm phân cách nghìn, không dựa vào locale (xem bài 04).
    """
    rounded = amount.quantize(Decimal("1"), rounding=ROUND_HALF_UP)
    return f"{rounded:,.0f}".replace(",", ".")


def classify_fare(total: Decimal) -> str:
    """if/elif/else: gắn nhãn mức cước để minh họa rẽ nhánh nhiều bậc."""
    if total <= BASE_FARE:
        return "cơ bản"
    elif total <= Decimal("50000"):
        return "trung bình"
    else:
        return "đường dài"


def max_distance_for_budget(budget: Decimal, *, service: str = "ride") -> Decimal:
    """Vòng while: tăng 0,5 km một lần tới khi vượt ngân sách hoặc hết miền."""
    step = Decimal("0.5")
    distance = Decimal("0")
    while distance + step <= MAX_KM:
        if calculate_fare(distance + step, service=service) > budget:
            break  # bước kế tiếp vượt ngân sách: dừng, giữ khoảng cách hiện tại
        distance += step
    return distance


def append_bad(item: str, bucket: list[str] = []) -> list[str]:  # noqa: B006
    """CẢNH BÁO: giá trị mặc định list được tạo MỘT lần lúc def, dùng chung mọi lần gọi."""
    bucket.append(item)
    return bucket


def append_good(item: str, bucket: list[str] | None = None) -> list[str]:
    """Cách đúng: mặc định None, tạo list mới trong thân hàm."""
    if bucket is None:
        bucket = []
    bucket.append(item)
    return bucket


def print_fare_table() -> None:
    """Vòng for lồng nhau với enumerate: in bảng cước, bỏ qua dòng ngoài miền."""
    distances = ["-1", "0", "2", "2.5", "5", "100", "100.5"]
    for service in SERVICES:
        print(f"[{service}]")
        for i, text in enumerate(distances, start=1):
            km = Decimal(text)
            if not MIN_KM <= km <= MAX_KM:
                print(f"  {i}. {text:>5} km  TỪ CHỐI (ngoài miền)")
                continue  # bỏ qua phần còn lại của lần lặp này
            total = calculate_fare(km, service=service)
            print(f"  {i}. {text:>5} km  {format_vnd(total):>8} VND  {classify_fare(total)}")


def demo_mutable_default() -> None:
    print("Mặc định list bị dùng chung:")
    print("  append_bad('a')  ->", append_bad("a"))
    print("  append_bad('b')  ->", append_bad("b"))
    print("  append_good('a') ->", append_good("a"))
    print("  append_good('b') ->", append_good("b"))


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

    money_cases = [
        # (km, service, kỳ vọng)
        ("0", "ride", "12000"),
        ("1.999", "ride", "12000"),
        ("2", "ride", "12000"),
        ("2.001", "ride", "12005"),
        ("2.499", "ride", "14246"),
        ("2.5", "ride", "14250"),
        ("5", "ride", "25500"),
        ("100", "ride", "453000"),
        ("5", "food", "25500"),
        ("5", "parcel", "25500"),
        ("0", "food", "12000"),
        ("2.5", "parcel", "14250"),
    ]
    for km, service, expected in money_cases:
        check(f"{km} km {service}", calculate_fare(Decimal(km), service=service), Decimal(expected))

    # Các ca phải bị từ chối bằng ValueError (kèm thông điệp thống nhất).
    for km, service in [("-0.001", "ride"), ("100.001", "ride"), ("5", "taxi")]:
        try:
            calculate_fare(Decimal(km), service=service)
        except ValueError as exc:
            check(f"từ chối {km} km {service}", str(exc).split(":")[0],
                  "service không hợp lệ" if service == "taxi" else "khoảng cách ngoài miền 0..100 km")
        else:
            check(f"từ chối {km} km {service}", "không lỗi", "ValueError")

    check("fare_breakdown", fare_breakdown(Decimal("5"), service="food"),
          (Decimal("12000"), Decimal("13500")))

    try:
        calculate_fare(Decimal("5"), "food")  # type: ignore[misc]
        keyword_only = False
    except TypeError:
        keyword_only = True
    check("service là keyword-only", keyword_only, True)

    first = append_good("a")
    second = append_good("b")
    check("append_good độc lập", (first, second), (["a"], ["b"]))

    bad_first = append_bad("a")
    bad_second = append_bad("b")
    check("append_bad dùng chung list", (bad_second, bad_first is bad_second), (["a", "b"], True))

    check("ngân sách 30.000", max_distance_for_budget(Decimal("30000")), Decimal("6.0"))
    check("ngân sách 20.000", max_distance_for_budget(Decimal("20000"), service="food"),
          Decimal("3.5"))

    check("format_vnd làm tròn HALF_UP", format_vnd(Decimal("12004.5")), "12.005")

    if failures:
        print(f"FAIL {total - len(failures)}/{total}")
        sys.exit(1)
    print(f"OK {total}/{total}")


def main() -> None:
    print("Bảng cước (giả định học tập):")
    print_fare_table()
    print()
    base, extra = fare_breakdown(Decimal("5"), service="food")
    print(f"Tách 5 km food: cơ bản={base}, phần vượt={extra}")
    print(f"Ngân sách 30.000 VND đi tối đa {max_distance_for_budget(Decimal('30000'))} km (ride)")
    print(f"Ngân sách 20.000 VND đi tối đa {max_distance_for_budget(Decimal('20000'))} km (ride)")
    print()
    demo_mutable_default()


if __name__ == "__main__":
    if "--verify" in sys.argv[1:]:
        verify()
    else:
        main()
