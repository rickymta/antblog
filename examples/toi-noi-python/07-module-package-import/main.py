"""Bài 07: module, package và import. Chạy: main.py (demo) hoặc main.py --verify."""

import contextlib
import importlib
import io
import subprocess
import sys
import tempfile
from decimal import Decimal
from pathlib import Path

# Import tuyệt đối từ gói đã được uv cài (editable) vào môi trường của project.
import toinoi_fare
from toinoi_fare import calculate_fare, quote


def import_experiment() -> dict[str, object]:
    """Tạo một module tạm, import hai lần, quan sát sys.path và sys.modules."""
    result: dict[str, object] = {}

    # 1) Chưa có trong sys.path: Python tìm không ra -> ModuleNotFoundError.
    try:
        importlib.import_module("probe_mod")
    except ModuleNotFoundError as exc:
        result["error"] = str(exc)

    with tempfile.TemporaryDirectory() as tmp:
        (Path(tmp) / "probe_mod.py").write_text(
            "import sys\n"
            'print("probe_mod: thân module đang chạy; trong sys.modules:", __name__ in sys.modules)\n'
            "VALUE = 42\n",
            encoding="utf-8",
        )
        sys.path.insert(0, tmp)  # 2) thêm thư mục vào sys.path
        importlib.invalidate_caches()  # file mới tạo: xóa cache của bộ tìm module
        buffer = io.StringIO()
        try:
            with contextlib.redirect_stdout(buffer):
                first = importlib.import_module("probe_mod")  # chạy thân module
                second = importlib.import_module("probe_mod")  # lấy từ sys.modules
            output = buffer.getvalue()
            result["body_runs"] = output.count("thân module đang chạy")
            result["in_modules_while_running"] = output.strip().endswith("True")
            result["same_object"] = first is second
            result["cached"] = sys.modules["probe_mod"] is first
        finally:
            sys.path.remove(tmp)
            sys.modules.pop("probe_mod", None)
    return result


def run_module_entry(km: str) -> str:
    """Chạy `python -m toinoi_fare <km>` trong tiến trình con, trả về dòng in ra."""
    done = subprocess.run(
        [sys.executable, "-m", "toinoi_fare", km],
        capture_output=True, text=True, check=True,
    )
    return done.stdout.strip()


def last_error_line(args: list[str], cwd: Path | None = None) -> str:
    """Chạy `python <args>` trong tiến trình con, trả về dòng cuối của stderr (dòng thông báo lỗi)."""
    done = subprocess.run([sys.executable, *args], capture_output=True, text=True, cwd=cwd)
    return done.stderr.strip().splitlines()[-1] if done.returncode != 0 else ""


HERE = Path(__file__).resolve().parent


def cycle_error() -> str:
    """Gói cycle_demo có hai module import lẫn nhau: import vòng. Chạy từ gốc lab."""
    return last_error_line(["-c", "import cycle_demo.fare_pricing"], cwd=HERE)


def script_relative_error() -> str:
    """Chạy thẳng file trong gói như script: import tương đối mất 'gói cha'."""
    return last_error_line([str(HERE / "src" / "toinoi_fare" / "pricing.py")])


def demo() -> None:
    print(f"__name__ của main.py : {__name__}")
    print(f"__name__ của gói     : {toinoi_fare.__name__}")
    print(f"__name__ của pricing : {quote.__module__}")
    for km in ("0", "2", "2.5", "5"):
        result = quote(km)
        print(f"{result.distance_km} km => {result.total_vnd} VND")
    exp = import_experiment()
    print(f"import probe_mod khi chưa có trong sys.path: {exp['error']}")
    print(f"thân probe_mod chạy {exp['body_runs']} lần sau 2 lần import")
    print(f"hai lần import cùng một object: {exp['same_object']}")
    print(f"probe_mod đã nằm trong sys.modules khi thân module chạy: {exp['in_modules_while_running']}")
    print(f"python -m toinoi_fare 2.5 => {run_module_entry('2.5')}")
    print(f"chạy pricing.py như script: {script_relative_error()}")
    # Dòng lỗi thật có thêm đường dẫn tuyệt đối ở cuối; cắt bớt để dễ đọc.
    print(f"import vòng: {cycle_error().split(' (/')[0]}")


def verify() -> None:
    exp = import_experiment()
    checks: list[tuple[str, bool]] = [
        ("0 km => 12000", quote("0").total_vnd == Decimal("12000")),
        ("2 km => 12000", quote("2").total_vnd == Decimal("12000")),
        ("2.5 km => 14250", quote("2.5").total_vnd == Decimal("14250")),
        ("5 km => 25500", quote("5").total_vnd == Decimal("25500")),
        ("khoảng cách âm bị từ chối", _rejects("-1")),
        ("quote() dùng calculate_fare", quote("2.5").total_vnd == calculate_fare("2.5")),
        ("quote ở gói và ở pricing là cùng object", toinoi_fare.quote is toinoi_fare.pricing.quote),
        ("pricing nằm trong sys.modules", "toinoi_fare.pricing" in sys.modules),
        ("gói có tên toinoi_fare", toinoi_fare.__name__ == "toinoi_fare"),
        ("main.py chạy với __name__ == '__main__'", __name__ == "__main__"),
        ("thiếu module -> ModuleNotFoundError", exp["error"] == "No module named 'probe_mod'"),
        ("thân module chạy đúng 1 lần", exp["body_runs"] == 1),
        ("import lần 2 trả về object đã cache", exp["same_object"] is True and exp["cached"] is True),
        ("python -m toinoi_fare 2.5", run_module_entry("2.5") == "2.5 km => 14250 VND"),
        ("chạy file trong gói như script -> lỗi import tương đối",
         "attempted relative import with no known parent package" in script_relative_error()),
        ("import vòng -> 'partially initialized module'", "partially initialized module" in cycle_error()),
        ("module vào sys.modules trước khi thân chạy xong", exp["in_modules_while_running"] is True),
        ("calculate_fare(100) hợp lệ, 100.001 bị từ chối", calculate_fare("100") == Decimal("453000") and _rejects("100.001")),
    ]
    passed = sum(1 for _, ok in checks if ok)
    for name, ok in checks:
        if not ok:
            print(f"  ca sai: {name}")
    if passed != len(checks):
        print(f"FAIL {passed}/{len(checks)}")
        sys.exit(1)
    print(f"OK {passed}/{len(checks)}")


def _rejects(distance: str) -> bool:
    """True nếu calculate_fare từ chối khoảng cách này bằng ValueError."""
    try:
        calculate_fare(distance)
    except ValueError:
        return True
    return False


if __name__ == "__main__":
    verify() if "--verify" in sys.argv[1:] else demo()
