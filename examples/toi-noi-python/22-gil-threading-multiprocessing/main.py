"""Bài 22: GIL, threading, multiprocessing và free-threaded 3.14.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify
Bản free-threaded (cần `uv python install 3.14t`):
        uv run --no-project --python 3.14t <lab>/main.py

So sánh tuần tự, ThreadPoolExecutor, ProcessPoolExecutor (và InterpreterPoolExecutor của
3.14) trên ba loại việc của dịch vụ kyc-ocr: chờ I/O, vòng lặp Python thuần, nén bằng zlib.
Số đo phụ thuộc máy và tải máy lúc chạy; lab in cả cấu hình lẫn load average để đọc cho đúng.
"""

import os
import platform
import statistics
import subprocess
import sys
import sysconfig
import time
from collections.abc import Callable, Sequence
from concurrent.futures import (
    Executor,
    InterpreterPoolExecutor,
    ProcessPoolExecutor,
    ThreadPoolExecutor,
)
import multiprocessing as mp
from dataclasses import dataclass

import workload


@dataclass(frozen=True)
class Config:
    """Quy mô một lượt đo. FULL là số in trong bài; QUICK dành cho --verify."""

    tasks: int  # số ảnh mỗi lô
    io_delay: float  # giây chờ «storage» cho mỗi ảnh
    pixels: int  # số điểm ảnh mỗi ảnh
    repeats: int  # số lần đo, lấy trung vị


FULL = Config(tasks=20, io_delay=0.1, pixels=workload.PAGE_PIXELS, repeats=3)
QUICK = Config(tasks=10, io_delay=0.05, pixels=1_000_000, repeats=1)

WORKERS = os.process_cpu_count() or 1


def gil_enabled() -> bool:
    """True nếu GIL đang bật. Build thường luôn True; build 3.14t thường False."""
    return sys._is_gil_enabled()


def core_kinds() -> str:
    """Số lõi hiệu năng (P) và tiết kiệm điện (E) trên Apple Silicon; máy khác thì nói không rõ.

    Máy có lõi E thì chạy 10 tiến trình CPU-bound không bao giờ được 10 lần một lõi P.
    """
    def sysctl(name: str) -> str | None:
        try:
            out = subprocess.run(["sysctl", "-n", name], capture_output=True, text=True, check=True)
        except (OSError, subprocess.CalledProcessError):
            return None
        return out.stdout.strip()

    performance, efficiency = sysctl("hw.perflevel0.logicalcpu"), sysctl("hw.perflevel1.logicalcpu")
    if performance is None:
        return "không rõ (không đọc được sysctl hw.perflevel)"
    return f"{performance} lõi P + {efficiency or 0} lõi E"


def describe_environment() -> None:
    """In môi trường đo: không có dòng này thì con số không đọc được."""
    start = mp.get_start_method()
    print(f"Máy: {platform.platform()}, os.cpu_count()={os.cpu_count()}, process_cpu_count()={WORKERS}")
    print(f"Python {platform.python_version()} ({platform.python_implementation()}), "
          f"Py_GIL_DISABLED={sysconfig.get_config_var('Py_GIL_DISABLED')}, GIL bật: {gil_enabled()}")
    print(f"start method: {start}, switch interval: {sys.getswitchinterval()} s")
    print(f"lõi theo loại: {core_kinds()}")
    print(f"load average 1 phút lúc bắt đầu: {os.getloadavg()[0]:.2f} (số lõi: {os.cpu_count()}); "
          "mỗi hàng đo in thêm load trước và sau, vì tải máy đổi giữa các hàng")


def run_sequential(fn: Callable[[int], int], ids: Sequence[int]) -> list[int]:
    return [fn(i) for i in ids]


def run_pool(make: Callable[[], Executor], fn: Callable[[int], int], ids: Sequence[int]) -> list[int]:
    """Tạo pool, chạy cả lô, đóng pool: thời gian gồm cả khởi động (cold)."""
    with make() as executor:
        return list(executor.map(fn, ids))  # map giữ thứ tự đầu vào


def timed(job: Callable[[], list[int]], repeats: int) -> tuple[float, list[int]]:
    """Trung vị thời gian tường (perf_counter) của `repeats` lần chạy, và kết quả lần cuối."""
    samples: list[float] = []
    result: list[int] = []
    for _ in range(repeats):
        start = time.perf_counter()
        result = job()
        samples.append(time.perf_counter() - start)
    return statistics.median(samples), result


def warm_pool_time(make: Callable[[], Executor], fn: Callable[[int], int],
                   ids: Sequence[int], repeats: int) -> tuple[float, list[int]]:
    """Đo khi pool đã khởi động sẵn: khởi động mọi worker trước, chỉ đo phần chạy lô."""
    with make() as executor:
        # Mỗi tác vụ ngủ 0,3 s nên pool buộc phải bật đủ WORKERS worker trước khi ta bấm giờ.
        list(executor.map(workload.fetch_page, range(WORKERS), [0.3] * WORKERS))
        return timed(lambda: list(executor.map(fn, ids)), repeats)


Row = tuple[float, list[int], float, float]  # giây, kết quả, load trước, load sau


def with_load(job: Callable[[], tuple[float, list[int]]]) -> Row:
    """Chạy một hàng đo và ghi load average 1 phút ngay trước và ngay sau hàng đó."""
    before = os.getloadavg()[0]
    seconds, result = job()
    return seconds, result, before, os.getloadavg()[0]


def benchmark(name: str, fn: Callable[[int], int], ids: Sequence[int], cfg: Config,
              *, with_processes: bool = True, with_interpreters: bool = True) -> dict[str, Row]:
    """Đo một loại việc trên mọi mô hình; trả {tên mô hình: (giây, kết quả, load trước, load sau)}."""
    rows: dict[str, Row] = {}
    rows["tuần tự"] = with_load(lambda: timed(lambda: run_sequential(fn, ids), cfg.repeats))
    rows[f"thread x{WORKERS}"] = with_load(lambda: timed(
        lambda: run_pool(lambda: ThreadPoolExecutor(WORKERS), fn, ids), cfg.repeats))
    if with_processes:
        rows[f"process x{WORKERS} (gồm khởi động)"] = with_load(lambda: timed(
            lambda: run_pool(lambda: ProcessPoolExecutor(WORKERS), fn, ids), cfg.repeats))
        rows[f"process x{WORKERS} (pool sẵn)"] = with_load(lambda: warm_pool_time(
            lambda: ProcessPoolExecutor(WORKERS), fn, ids, cfg.repeats))
    if with_interpreters:
        rows[f"interpreter x{WORKERS} (pool sẵn)"] = with_load(lambda: warm_pool_time(
            lambda: InterpreterPoolExecutor(WORKERS), fn, ids, cfg.repeats))
    print(f"\n{name}: {len(ids)} ảnh, trung vị {cfg.repeats} lần")
    base = rows["tuần tự"][0]
    for label, (seconds, _, before, after) in rows.items():
        print(f"  {label:<36} {seconds:7.3f} s   x{base / seconds:5.2f}   (load {before:5.1f} -> {after:5.1f})")
    return rows


def make_jobs(cfg: Config) -> list[tuple[str, Callable[[int], int]]]:
    """Ba loại việc với tham số của cấu hình. Dùng functools.partial để hàm vẫn pickle được."""
    from functools import partial

    return [
        ("I/O: chờ storage", partial(workload.fetch_page, delay=cfg.io_delay)),
        ("CPU: vòng lặp Python (binarize)", partial(workload.binarize_page, pixels=cfg.pixels)),
        ("CPU trong C: zlib.compress", partial(workload.compress_page, pixels=cfg.pixels)),
    ]


def run_all(cfg: Config) -> None:
    ids = list(range(1, cfg.tasks + 1))
    describe_environment()
    for name, fn in make_jobs(cfg):
        benchmark(name, fn, ids, cfg)


def verify() -> None:
    """Kiểm kết quả đúng (so với tuần tự) và các hành vi lỗi đã in trong bài.

    Không kiểm tốc độ nào: số giây và tỉ lệ phụ thuộc máy và tải máy lúc chạy (đã thấy một
    kiểm «thread không nhanh hơn tuần tự» trượt khi load average lên hàng chục), nên chúng
    chỉ được in ra ở lệnh chạy thường, kèm load trước và sau mỗi hàng.
    """
    failures: list[str] = []
    total = 0

    def check(name: str, ok: bool, detail: str = "") -> None:
        nonlocal total
        total += 1
        if not ok:
            failures.append(name)
            print(f"SAI: {name} {detail}".rstrip())

    cfg = QUICK
    ids = list(range(1, cfg.tasks + 1))
    jobs = dict(make_jobs(cfg))
    cpu_fn = jobs["CPU: vòng lặp Python (binarize)"]
    zlib_fn = jobs["CPU trong C: zlib.compress"]

    # 1. Giá trị cố định: binarize_page(1) với seed 1 là 1_006_619 điểm tối trên 2.000.000.
    check("binarize_page(1)", workload.binarize_page(1) == 1_006_619,
          f"nhận {workload.binarize_page(1)}")

    # 2. Mọi mô hình cho đúng kết quả của bản tuần tự, đúng thứ tự, cho cả hai loại việc CPU.
    for fn_name, fn in (("binarize", cpu_fn), ("zlib", zlib_fn)):
        expected = run_sequential(fn, ids)
        for label, make in [("thread", lambda: ThreadPoolExecutor(WORKERS)),
                            ("process", lambda: ProcessPoolExecutor(WORKERS)),
                            ("interpreter", lambda: InterpreterPoolExecutor(WORKERS))]:
            check(f"{fn_name}: kết quả {label} == tuần tự", run_pool(make, fn, ids) == expected)

    # 3. Hàm lambda không gửi được sang process: PicklingError nổi lên ở future.result().
    with ProcessPoolExecutor(2) as executor:
        future = executor.submit(lambda page_id: page_id, 1)
        try:
            future.result()
            kind = "không lỗi"
        except Exception as exc:  # noqa: BLE001 - cố ý bắt rộng để ghi lại loại lỗi
            kind = type(exc).__name__
    check("lambda -> PicklingError", kind == "PicklingError", f"nhận {kind}")

    # 4. Ngoại lệ trong worker được ném lại ở tiến trình chính, giữ nguyên loại và thông điệp.
    with ProcessPoolExecutor(2) as executor:
        try:
            list(executor.map(workload.fail_on_odd, [2, 4, 5]))
            got = "không lỗi"
        except ValueError as exc:
            got = str(exc)
    check("ngoại lệ từ worker", got == "ảnh 5 hỏng", f"nhận {got!r}")

    # 5. GIL bật/tắt khớp với loại build (trừ khi ép bằng PYTHON_GIL).
    if not sysconfig.get_config_var("Py_GIL_DISABLED"):
        check("build thường thì GIL bật", gil_enabled())
    else:
        check("build free-threaded chạy được", isinstance(gil_enabled(), bool))

    if failures:
        print(f"FAIL {total - len(failures)}/{total}")
        sys.exit(1)
    print(f"OK {total}/{total}")


if __name__ == "__main__":
    if "--verify" in sys.argv:
        verify()
    else:
        run_all(FULL)
