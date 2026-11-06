"""Ba cách «đặt timeout» cho OCR và thứ còn lại sau khi hết hạn. Dùng cho phần 3 của main.py.

Mỗi hàm trả một dict số liệu; main.py in ra và --verify kiểm bất biến (thứ tự, còn sống hay không),
không kiểm số giây tuyệt đối vì máy chạy lab có thể đang bận.
"""

import asyncio
import contextlib
import os
import resource
import subprocess
import time
from concurrent.futures import ProcessPoolExecutor
from concurrent.futures import TimeoutError as FutureTimeout

from app.ocr import OcrTimeout, run_tesseract
from app.settings import ocr_env

TESS_ARGS = ["stdin", "stdout", "-l", "vie+eng", "--psm", "6", "tsv"]


def pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)  # tín hiệu 0 không gửi gì, chỉ kiểm tiến trình còn tồn tại
    except ProcessLookupError:
        return False
    return True


def blocking_ocr(binary: str, image: bytes, pids: list[int] | None = None) -> tuple[float, float]:
    """Gọi Tesseract đồng bộ. Trả về (lúc bắt đầu, lúc xong) theo time.time() để so thứ tự giữa các tiến trình."""
    started = time.time()
    proc = subprocess.Popen([binary, *TESS_ARGS], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, env=ocr_env())
    if pids is not None:
        pids.append(proc.pid)
    proc.communicate(image)
    return started, time.time()


async def wait_for_to_thread(binary: str, image: bytes, timeout: float) -> dict[str, object]:
    """Cách 1: asyncio.wait_for(asyncio.to_thread(...)). Coroutine ngừng chờ; thread và Tesseract vẫn chạy."""
    pids: list[int] = []
    started = time.perf_counter()
    try:
        await asyncio.wait_for(asyncio.to_thread(blocking_ocr, binary, image, pids), timeout)
    except TimeoutError:
        pass
    gave_up_s = time.perf_counter() - started
    for _ in range(200):  # thread có thể chưa kịp sinh tiến trình con khi ta đã bỏ cuộc
        if pids:
            break
        await asyncio.sleep(0.01)
    alive_after_timeout = pid_alive(pids[0])
    while pid_alive(pids[0]):
        await asyncio.sleep(0.01)
    return {"gave_up_s": gave_up_s, "child_alive_after_timeout": alive_after_timeout,
            "child_finished_after_s": time.perf_counter() - started}


def pool_result_timeout(binary: str, image: bytes, timeout: float) -> dict[str, object]:
    """Cách 2: ProcessPoolExecutor(1) và future.result(timeout). Hết hạn: cancel() không hủy việc đang chạy,
    nên yêu cầu thứ hai phải chờ hết việc cũ (worker duy nhất vẫn bận).

    Pool được làm ấm bằng một việc rỗng trước khi đo. Bản đầu không làm vậy: tiến trình worker (spawn trên macOS) còn
    đang nạp lại main.py khi hết timeout, Tesseract chỉ bắt đầu sau đó khoảng một giây, và phần lớn «chờ» là khởi động."""
    with ProcessPoolExecutor(max_workers=1) as pool:
        warm = time.perf_counter()
        pool.submit(abs, 0).result()
        warmup_s = time.perf_counter() - warm
        first = pool.submit(blocking_ocr, binary, image)
        try:
            first.result(timeout=timeout)
        except FutureTimeout:
            pass
        gave_up_at = time.time()
        cancelled = first.cancel()  # đã chạy rồi nên False
        second = pool.submit(blocking_ocr, binary, image)  # một yêu cầu lành, gửi ngay sau khi bỏ cuộc
        first_start, first_end = first.result()
        second_start, second_end = second.result()
    return {"cancel_returned": cancelled, "warmup_s": warmup_s, "first_started_before_give_up": first_start < gave_up_at,
            "first_end_after_give_up_s": first_end - gave_up_at,
            "second_started_after_first_ended": second_start >= first_end - 0.001,
            "second_waited_s": second_start - gave_up_at}


async def kill_on_timeout(binary: str, image: bytes, timeout: float) -> dict[str, object]:
    """Cách 3 (của lab): run_tesseract kill tiến trình con khi hết hạn, rồi nhận ngay việc kế tiếp."""
    started = time.perf_counter()
    try:
        await run_tesseract(binary, image, timeout=timeout)
        raise AssertionError("OCR xong trước timeout: đặt timeout nhỏ hơn")
    except OcrTimeout as exc:
        pid = exc.pid
    gave_up_s = time.perf_counter() - started
    alive = pid_alive(pid)
    again = time.perf_counter()
    await run_tesseract(binary, image, timeout=60)  # việc kế tiếp không phải chờ gì
    return {"gave_up_s": gave_up_s, "child_alive_after_timeout": alive,
            "next_job_s": time.perf_counter() - again}


def child_pids() -> list[int]:
    """PID các tiến trình `tesseract` là con trực tiếp của tiến trình này (pgrep -P ... -x tesseract). Lọc theo tên vì
    ProcessPoolExecutor ở bước trước để lại một tiến trình con khác (resource tracker của multiprocessing)."""
    out = subprocess.run(["pgrep", "-P", str(os.getpid()), "-x", "tesseract"], capture_output=True, text=True).stdout
    return [int(x) for x in out.split()]


async def cancel_kills_child(binary: str, image: bytes) -> dict[str, object]:
    """arq hủy job quá job_timeout bằng cách cancel coroutine (asyncio.wait_for). `finally` của run_tesseract phải
    kill tiến trình con trong trường hợp đó, không chỉ khi hết `timeout` của chính nó."""
    task = asyncio.create_task(run_tesseract(binary, image, timeout=60))
    pids: list[int] = []
    for _ in range(300):  # chờ tiến trình con xuất hiện
        await asyncio.sleep(0.01)
        pids = child_pids()
        if pids:
            break
    task.cancel()
    with contextlib.suppress(asyncio.CancelledError):
        await task
    return {"children_seen": len(pids), "alive_after_cancel": [pid_alive(p) for p in pids]}


async def max_loop_lag_ms(work, tick: float = 0.01) -> tuple[float, float]:
    """Chạy `work()` trong khi một «nhịp tim» 10 ms đo độ trễ lớn nhất của vòng lặp sự kiện.
    Trả về (độ trễ lớn nhất ms, thời gian `work` chạy ms)."""
    worst = 0.0
    stop = False

    async def heartbeat() -> None:
        nonlocal worst
        while not stop:
            before = time.perf_counter()
            await asyncio.sleep(tick)
            worst = max(worst, time.perf_counter() - before - tick)

    task = asyncio.create_task(heartbeat())
    await asyncio.sleep(tick * 3)
    started = time.perf_counter()
    await work()
    elapsed = time.perf_counter() - started
    stop = True
    await task
    return worst * 1000, elapsed * 1000


async def loop_lag_compare(binary: str, image: bytes) -> dict[str, float]:
    """subprocess.run trong coroutine chặn vòng lặp suốt thời gian OCR; asyncio subprocess thì không."""

    async def blocking() -> None:
        subprocess.run([binary, *TESS_ARGS], input=image, capture_output=True, env=ocr_env(), check=True)

    async def non_blocking() -> None:
        await run_tesseract(binary, image, timeout=60)

    blocked_lag, blocked_ms = await max_loop_lag_ms(blocking)
    free_lag, free_ms = await max_loop_lag_ms(non_blocking)
    return {"blocking_lag_ms": blocked_lag, "blocking_run_ms": blocked_ms,
            "async_lag_ms": free_lag, "async_run_ms": free_ms}


def cpu_per_ocr(binary: str, image: bytes, *, omp_limit: str | None, runs: int = 5) -> tuple[float, float]:
    """(giây tường, giây CPU) trung bình mỗi lần OCR, chạy tuần tự. CPU lấy từ rusage của các tiến trình con.
    omp_limit=None: để Tesseract tự chọn số luồng OpenMP; "1": một luồng (khuyến nghị của FAQ khi chạy
    mỗi lõi một tiến trình)."""
    env = {k: v for k, v in os.environ.items() if k != "OMP_THREAD_LIMIT"}
    if omp_limit is not None:
        env["OMP_THREAD_LIMIT"] = omp_limit
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    started = time.perf_counter()
    for _ in range(runs):
        subprocess.run([binary, *TESS_ARGS], input=image, capture_output=True, env=env, check=True)
    wall = (time.perf_counter() - started) / runs
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    cpu = ((after.ru_utime - before.ru_utime) + (after.ru_stime - before.ru_stime)) / runs
    return wall, cpu
