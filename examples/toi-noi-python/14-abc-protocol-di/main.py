"""Bài 14: ABC, Protocol, composition và dependency injection. Chạy: main.py | main.py --verify."""

import argparse
import re
import subprocess
import sys
import time
from datetime import UTC, datetime, timedelta
from decimal import Decimal
from pathlib import Path
from typing import Any

from fare import ServiceKind
from ports import Clock, FixedClock, SystemClock
from providers import DictPolicyProvider, JsonFilePolicyProvider, PolicyProviderBase
from service import HardcodedQuoteService, QuoteService

HERE = Path(__file__).resolve().parent
D = Decimal
START = datetime(2026, 1, 1, 8, 0, tzinfo=UTC)


class IncompleteProvider(PolicyProviderBase):
    """Lớp con quên cài `lookup`: ABC chặn ngay lúc khởi tạo."""


class WrongSignatureClock:
    """Có tên `now` nhưng sai chữ ký: isinstance với runtime_checkable vẫn nói «đúng»."""

    def now(self, tz: str) -> str:
        return tz


def build_service(clock: Clock) -> QuoteService:
    """Composition root: nơi DUY NHẤT chọn cài đặt cụ thể và nối chúng lại với nhau."""
    return QuoteService(JsonFilePolicyProvider(HERE / "policies.json"), clock)


def message_of(func: Any, *args: object) -> str:
    """Gọi func và trả thông điệp của exception bay ra (dạng «TênLỗi: thông điệp»)."""
    try:
        func(*args)
    except Exception as exc:
        return f"{type(exc).__name__}: {exc}"
    return "không có lỗi"


def duck_typing_late_error() -> str:
    """Không khai báo kiểu: nhận `object()` làm provider vẫn dựng được; lỗi chỉ lộ khi gọi."""
    wrong: Any = object()
    service = QuoteService(wrong, SystemClock())  # dựng xong bình thường
    return message_of(service.quote, "Hà Nội", D("5"))


def demo() -> None:
    print("== 1. Duck typing: lỗi đến muộn ==")
    print(duck_typing_late_error())

    print("== 2. Trước: phụ thuộc gắn cứng ==")
    hardcoded = HardcodedQuoteService()
    first = hardcoded.quote(D("5"))
    time.sleep(0.01)
    second = hardcoded.quote(D("5"))
    print(f"hai báo giá 5 km cùng giá {first.fare.amount} nhưng expires_at khác nhau: {first.expires_at != second.expires_at}")

    print("== 3. Sau: tiêm đồng hồ và nguồn chính sách ==")
    clock = FixedClock(START)
    service = build_service(clock)
    for city in ("Hà Nội", "Đà Nẵng", "Cần Thơ"):
        print(f"{city}: 5 km -> {service.quote(city, D('5')).fare.amount} VND")
    quote = service.quote("Hà Nội", D("5"))
    print(f"expires_at = {quote.expires_at.isoformat()}")
    for elapsed in (timedelta(minutes=4, seconds=59), timedelta(seconds=1)):
        clock.advance(elapsed)
        print(f"sau khi tua thêm {elapsed}: hết hạn = {service.is_expired(quote)}")

    print("== 4. ABC chặn lớp thiếu phương thức ==")
    print(message_of(PolicyProviderBase))
    print(message_of(IncompleteProvider))

    print("== 5. Protocol: isinstance chỉ kiểm tên ==")
    print(f"isinstance(FixedClock, Clock) = {isinstance(FixedClock(START), Clock)}")
    print(f"isinstance(object(), Clock) = {isinstance(object(), Clock)}")
    print(f"isinstance(WrongSignatureClock(), Clock) = {isinstance(WrongSignatureClock(), Clock)}  (mypy mới bắt được)")


def run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run([sys.executable, "-m", *args], capture_output=True, text=True, cwd=HERE)


def marked_lines(path: Path) -> set[int]:
    return {no for no, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1) if "# E:" in line}


def reported_lines(output: str) -> set[int]:
    return {int(m.group(1)) for m in re.finditer(r"^broken\.py:(\d+): error:", output, re.MULTILINE)}


def verify() -> None:
    clock = FixedClock(START)
    service = build_service(clock)
    quote = service.quote("Hà Nội", D("5"))
    hardcoded = HardcodedQuoteService()
    h1 = hardcoded.quote(D("5"))
    time.sleep(0.01)
    h2 = hardcoded.quote(D("5"))
    dict_provider = DictPolicyProvider({})
    clock.advance(timedelta(minutes=4, seconds=59))
    expired_at_4m59 = service.is_expired(quote)
    clock.advance(timedelta(seconds=1))
    expired_at_5m00 = service.is_expired(quote)
    mypy_main = run("mypy", "main.py", "tests", "--strict", "--no-color-output")
    mypy_broken = run("mypy", "broken.py", "--strict", "--no-color-output")
    pytest_run = run("pytest", "-q", "-p", "no:cacheprovider")
    expected_lines = marked_lines(HERE / "broken.py")

    checks: list[tuple[str, bool]] = [
        ("duck typing: dựng được nhưng gọi mới AttributeError", "AttributeError" in duck_typing_late_error()),
        ("gắn cứng: hai báo giá cùng giá 25500", h1.fare.amount == h2.fare.amount == D("25500")),
        ("gắn cứng: expires_at không lặp lại được", h1.expires_at != h2.expires_at),
        ("Hà Nội 5 km => 25500", quote.fare.amount == D("25500")),
        ("Đà Nẵng 5 km => 27000 (policies.json)", service.quote("Đà Nẵng", D("5")).fare.amount == D("27000")),
        ("Cần Thơ không có riêng => mặc định 25500", service.quote("Cần Thơ", D("5")).fare.amount == D("25500")),
        ("FixedClock: expires_at = 08:05 UTC", quote.expires_at == START + timedelta(minutes=5)),
        ("Quote.is_valid: +4m59s còn hiệu lực", quote.is_valid(START + timedelta(minutes=4, seconds=59))),
        ("Quote.is_valid: đúng +5m00s là hết", not quote.is_valid(START + timedelta(minutes=5))),
        ("ServiceKind: 3 dịch vụ cùng giá 25500", {service.quote("Hà Nội", D("5"), service=k).fare.amount for k in ServiceKind} == {D("25500")}),
        ("tua 4m59s: chưa hết hạn", not expired_at_4m59),
        ("tua thêm 1s: hết hạn", expired_at_5m00),
        ("ABC: khởi tạo trực tiếp bị TypeError", message_of(PolicyProviderBase).startswith("TypeError")),
        ("ABC: lớp con thiếu lookup bị TypeError", "lookup" in message_of(IncompleteProvider)),
        ("ABC: DictPolicyProvider dùng chung luật mặc định", dict_provider.policy_for("Hà Nội") == dict_provider.policy_for("Nha Trang")),
        ("isinstance(Clock): FixedClock đúng, object() sai", isinstance(FixedClock(START), Clock) and not isinstance(object(), Clock)),
        ("isinstance(Clock) chỉ kiểm tên: sai chữ ký vẫn True", isinstance(WrongSignatureClock(), Clock)),
        ("mypy --strict main.py tests: exit 0", mypy_main.returncode == 0),
        ("mypy --strict broken.py: exit 1, 5 lỗi", mypy_broken.returncode == 1 and "Found 5 errors in 1 file" in mypy_broken.stdout),
        ("broken.py: lỗi đúng các dòng đánh dấu # E:", reported_lines(mypy_broken.stdout) == expected_lines),
        ("pytest: 12 passed", pytest_run.returncode == 0 and "12 passed" in pytest_run.stdout),
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
    parser = argparse.ArgumentParser(description="ABC, Protocol và dependency injection cho báo giá Tới Nơi")
    parser.add_argument("--verify", action="store_true", help="kiểm các kết quả kỳ vọng, gồm chạy mypy và pytest thật")
    args = parser.parse_args()
    if args.verify:
        verify()
    else:
        demo()


if __name__ == "__main__":
    main()
