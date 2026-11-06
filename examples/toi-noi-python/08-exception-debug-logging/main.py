"""Bài 08: exception, debug và logging. Chạy: main.py [mét] [--buggy] [--unhandled] [--pdb] | --verify."""

import argparse
import logging
import subprocess
import sys
from decimal import Decimal, InvalidOperation
from pathlib import Path

import fare

HERE = Path(__file__).resolve().parent


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Báo giá Tới Nơi từ số mét của thiết bị")
    parser.add_argument("metres", nargs="?", help="khoảng cách thiết bị gửi (mét), ví dụ 2500")
    parser.add_argument("--buggy", action="store_true", help="bật lỗi đổi đơn vị cố ý")
    parser.add_argument("--unhandled", action="store_true", help="không bắt lỗi: xem traceback thô")
    parser.add_argument("--pdb", action="store_true", help="dừng bằng breakpoint() trong quote_from_metres()")
    parser.add_argument("--log-level", default="INFO", help="DEBUG, INFO, WARNING, ERROR")
    parser.add_argument("--verify", action="store_true", help="kiểm các kết quả kỳ vọng")
    return parser


def run_once(raw: str, *, buggy: bool, unhandled: bool, use_pdb: bool) -> None:
    if unhandled:
        total = fare.quote_from_metres(raw, buggy=buggy, pause=use_pdb)  # lỗi nào cũng bay thẳng ra, in traceback
        print(f"{raw} m => {total} VND")
        return
    print(f"{raw} m => {fare.handle_request(raw, buggy=buggy, pause=use_pdb)}")


def demo(args: argparse.Namespace) -> None:
    inputs = [args.metres] if args.metres is not None else ["2500", "-100", "abc"]
    for raw in inputs:
        run_once(raw, buggy=args.buggy, unhandled=args.unhandled, use_pdb=args.pdb)


class ListHandler(logging.Handler):
    """Handler gom LogRecord vào list để --verify kiểm tra log mà không phụ thuộc stderr."""

    def __init__(self) -> None:
        super().__init__(level=logging.DEBUG)
        self.records: list[logging.LogRecord] = []

    def emit(self, record: logging.LogRecord) -> None:
        self.records.append(record)


def raises(exc_type: type[BaseException], func, *args, **kwargs) -> BaseException | None:
    """Gọi func; trả về exception nếu đúng loại exc_type, None nếu không có lỗi; lỗi khác thì bay tiếp."""
    try:
        func(*args, **kwargs)
    except exc_type as exc:
        return exc
    return None


def run_python(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run([sys.executable, *args], capture_output=True, text=True, cwd=HERE)


def verify() -> None:
    q = fare.quote_from_metres
    handler = ListHandler()
    fare.logger.addHandler(handler)
    fare.logger.setLevel(logging.DEBUG)
    results = [
        fare.handle_request("2500"),
        fare.handle_request("-100"),
        fare.handle_request("abc"),
    ]
    fare.logger.removeHandler(handler)
    levels = [r.levelname for r in handler.records]
    exception_records = [r for r in handler.records if r.exc_info]

    out_of_range_neg = raises(fare.DistanceOutOfRangeError, q, "-100")
    out_of_range_big = raises(fare.DistanceOutOfRangeError, q, "100001")
    device_error = raises(fare.DeviceDataError, q, "abc")
    assert_off = run_python("-O", "-c", "from fare import unsafe_guard; print(unsafe_guard(-5))")
    assert_on = run_python("-c", "from fare import unsafe_guard; unsafe_guard(-5)")

    checks: list[tuple[str, bool]] = [
        ("0 m => 12000", q("0") == Decimal("12000")),
        ("2000 m => 12000", q("2000") == Decimal("12000")),
        ("2500 m => 14250", q("2500") == Decimal("14250")),
        ("5000 m => 25500", q("5000") == Decimal("25500")),
        ("--buggy: 2500 m bị tính 25500 (sai nhưng không lỗi)", q("2500", buggy=True) == Decimal("25500")),
        ("FareError là ValueError", issubclass(fare.FareError, ValueError)),
        ("-100 m raise DistanceOutOfRangeError", out_of_range_neg is not None),
        ("100001 m raise DistanceOutOfRangeError", out_of_range_big is not None),
        ("lỗi miền là FareError (bắt được bằng lớp cha)", isinstance(out_of_range_neg, fare.FareError)),
        ("'abc' raise DeviceDataError", device_error is not None),
        ("raise from: __cause__ là InvalidOperation", isinstance(device_error.__cause__, InvalidOperation)),
        ("handle_request: 2500 => 14250 VND", results[0] == "14250 VND"),
        ("handle_request: -100 => REJECT", results[1] == "REJECT"),
        ("handle_request: abc => DEVICE_ERROR", results[2] == "DEVICE_ERROR"),
        ("finally chạy đủ 3 lần (3 dòng DEBUG kết thúc)", sum("kết thúc" in r.getMessage() for r in handler.records) == 3),
        ("có 1 WARNING và 1 ERROR", levels.count("WARNING") == 1 and levels.count("ERROR") == 1),
        ("logger.exception đính kèm traceback", len(exception_records) == 1),
        ("logger tên theo module", fare.logger.name == "fare"),
        ("python -O: assert bị bỏ qua, -5 lọt qua", assert_off.returncode == 0 and assert_off.stdout.strip() == "-5"),
        ("python thường: assert raise AssertionError", "AssertionError" in assert_on.stderr),
        ("mask_phone không lộ số giữa", fare.mask_phone("0912345678") == "09******78"),
    ]
    passed = sum(1 for _, ok in checks if ok)
    for name, ok in checks:
        if not ok:
            print(f"  ca sai: {name}")
    if passed != len(checks):
        print(f"FAIL {passed}/{len(checks)}")
        sys.exit(1)
    print(f"OK {passed}/{len(checks)}")


def main() -> None:
    args = build_parser().parse_args()
    if args.verify:
        verify()
        return
    # Cấu hình logging đúng một lần, ở điểm vào của chương trình (không ở trong module thư viện).
    logging.basicConfig(
        level=args.log_level.upper(),
        format="%(asctime)s %(levelname)-7s %(name)s: %(message)s",
    )
    demo(args)


if __name__ == "__main__":
    main()
