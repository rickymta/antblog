"""Client Python thật cho module PyO3 `toi_noi_fee` (dựng bằng maturin develop)."""
import sys

import toi_noi_fee as fee


def check(condition: bool, what: str) -> None:
    if not condition:
        raise AssertionError(what)
    print(f"  ok  {what}")


def raises(exc_type, fn, *args):
    try:
        fn(*args)
    except BaseException as e:  # PanicException không kế thừa Exception nên bắt BaseException
        return e if isinstance(e, exc_type) else None
    return None


def main() -> None:
    print(f"Python {sys.version.split()[0]}, module {fee.__name__}")

    print("== 1. Gọi hàm, nhận struct ==")
    s = fee.split_fare(25_500)
    print(f"      {s!r}")
    check(isinstance(s, fee.FeeSplit), "split_fare trả đối tượng FeeSplit của Rust")
    check((s.fare_dong, s.platform_fee_dong, s.driver_net_dong) == (25_500, 5_100, 20_400),
          "cước 25.500, phí 5.100, tài xế 20.400")
    check(s == fee.split_fare(25_500), "so sánh theo giá trị (eq)")
    check(fee.fare_dong_for_km(5) == 25_500, "fare_dong_for_km(5) = 25.500")

    print("== 2. Ranh giới kiểu: lỗi thành exception ==")
    e = raises(ValueError, fee.split_fare, -1)
    check(e is not None, "cước âm -> ValueError")
    print(f"      {e}")
    check(raises(OverflowError, fee.split_fare, 2**63) is not None,
          "int Python vượt i64 -> OverflowError, không bị cắt lặng lẽ")
    check(raises(TypeError, fee.split_fare, 25_500.5) is not None,
          "float vào chỗ i64 -> TypeError (không có đường cho tiền float lọt vào)")
    check(raises(OverflowError, fee.fare_dong_for_km, -1) is not None, "km âm -> OverflowError (u32)")
    try:
        s.fare_dong = 1
    except AttributeError:
        print("  ok  FeeSplit là frozen: gán thuộc tính bị từ chối")
    else:
        raise AssertionError("FeeSplit phải frozen")

    print("== 3. Panic trong Rust ==")
    e = raises(BaseException, fee.panic_for_test)
    check(e is not None and type(e).__name__ == "PanicException", "panic Rust -> PanicException")
    check(not isinstance(e, Exception), "PanicException kế thừa BaseException, `except Exception` không nuốt nó")
    check(fee.split_fare(25_500).driver_net_dong == 20_400, "sau panic, tiến trình Python vẫn sống và gọi tiếp được")

    print("PYTHON OK")


if __name__ == "__main__":
    main()
