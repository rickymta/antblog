"""Bài 23: Lock, Queue, Semaphore - tránh nhận trùng chuyến.

Chạy:   uv run --project examples/toi-noi-python/23-lock-queue-semaphore examples/toi-noi-python/23-lock-queue-semaphore/main.py
Verify: thêm --verify ở cuối lệnh trên. Chỉ in bảng race: thêm --race-table (chạy được trên 3.14t).

Mọi thứ nằm trong MỘT process và trong bộ nhớ. Khóa trong bài này không bảo vệ được dữ liệu
khi có hai server; trường hợp đó cần giao dịch/điều kiện ở database (bài 25).
Các con số logic (số người thắng, peak) lặp lại được; số giây đo được dao động theo máy.
"""

import asyncio
import dis
import os
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from enum import StrEnum


class TripStatus(StrEnum):
    REQUESTED = "requested"
    SEARCHING = "searching"
    ACCEPTED = "accepted"
    ARRIVING = "arriving"
    PICKED_UP = "picked_up"
    IN_TRANSIT = "in_transit"
    COMPLETED = "completed"
    CANCELLED = "cancelled"


@dataclass
class Trip:
    trip_id: str
    driver_id: str | None = None
    status: TripStatus = TripStatus.SEARCHING


# --- 1. Race check-then-act với asyncio ----------------------------------------------------
async def accept_unsafe(trip: Trip, driver_id: str) -> bool:
    """SAI: kiểm tra, nhường quyền ở `await`, rồi mới ghi. Giữa hai bước, trạng thái có thể đã đổi."""
    if trip.driver_id is not None:  # check
        return False
    await asyncio.sleep(0)  # giả lập việc chờ I/O (xác minh tài xế, ghi log, gọi service khác)
    trip.driver_id = driver_id  # act: có thể ghi đè người đã nhận
    trip.status = TripStatus.ACCEPTED
    return True


async def accept_locked(trip: Trip, driver_id: str, lock: asyncio.Lock) -> bool:
    """Đúng: kiểm tra VÀ ghi nằm trong cùng một vùng lock, kể cả đoạn await ở giữa."""
    async with lock:
        if trip.driver_id is not None:
            return False
        await asyncio.sleep(0)
        trip.driver_id = driver_id
        trip.status = TripStatus.ACCEPTED
        return True


def driver_ids(count: int) -> list[str]:
    return [f"DR-{i:03d}" for i in range(1, count + 1)]


async def race_asyncio(count: int, *, locked: bool) -> tuple[int, Trip]:
    trip = Trip("TN-0001")
    lock = asyncio.Lock()
    if locked:
        coros = [accept_locked(trip, d, lock) for d in driver_ids(count)]
    else:
        coros = [accept_unsafe(trip, d) for d in driver_ids(count)]
    results = await asyncio.gather(*coros)
    return sum(results), trip


async def lock_is_not_reentrant() -> str:
    """asyncio.Lock không tái nhập: giữ lock rồi xin lại sẽ chờ mãi; asyncio.timeout cắt cho thấy."""
    lock = asyncio.Lock()
    async with lock:
        try:
            async with asyncio.timeout(0.2):
                async with lock:
                    return "lấy được lock lần hai"
        except TimeoutError:
            return "TimeoutError: lần xin thứ hai chờ mãi"
    return "không tới được"


# --- 2. Race với thread --------------------------------------------------------------------
def accept_thread_unsafe(trip: Trip, driver_id: str, barrier: threading.Barrier) -> bool:
    """Check-then-act có lịch xen kẽ ĐƯỢC ĐIỀU KHIỂN: mọi thread đọc xong rồi mới cùng ghi."""
    if trip.driver_id is not None:
        return False
    barrier.wait(timeout=5)  # điểm đồng bộ: thay cho việc cầu may bộ lập lịch
    trip.driver_id = driver_id
    trip.status = TripStatus.ACCEPTED
    return True


def accept_thread_locked(trip: Trip, driver_id: str, lock: threading.Lock) -> bool:
    with lock:
        if trip.driver_id is not None:
            return False
        time.sleep(0.005)  # giữ lock lâu hơn để cửa sổ race rộng; kết quả vẫn đúng
        trip.driver_id = driver_id
        trip.status = TripStatus.ACCEPTED
        return True


def race_threads(count: int, *, locked: bool) -> tuple[int, Trip]:
    trip = Trip("TN-0001")
    barrier = threading.Barrier(count)
    lock = threading.Lock()
    with ThreadPoolExecutor(max_workers=count) as pool:
        if locked:
            futures = [pool.submit(accept_thread_locked, trip, d, lock) for d in driver_ids(count)]
        else:
            futures = [pool.submit(accept_thread_unsafe, trip, d, barrier) for d in driver_ids(count)]
        wins = sum(f.result() for f in futures)
    return wins, trip


def audit(driver_id: str) -> None:
    """Một lời gọi hàm Python rỗng, đại diện cho việc ghi log/kiểm tra nhỏ giữa check và act."""


def accept_thread_no_sync(trip: Trip, driver_id: str) -> bool:
    """Cùng logic sai nhưng KHÔNG có điểm đồng bộ và không có đoạn chờ giữa check và act."""
    if trip.driver_id is not None:
        return False
    trip.driver_id = driver_id
    return True


def accept_thread_no_sync_call(trip: Trip, driver_id: str) -> bool:
    """Như trên, chỉ thêm MỘT lời gọi hàm giữa hai dòng."""
    if trip.driver_id is not None:
        return False
    audit(driver_id)
    trip.driver_id = driver_id
    return True


def broken_trips(*, with_call: bool, interval: float | None, trips: int = 2000, threads: int = 4) -> int:
    """Chạy code sai không đồng bộ `trips` chuyến; đếm chuyến có hơn một người thắng.

    `interval` đặt sys.setswitchinterval (None: giữ mặc định 5 ms) rồi khôi phục.
    """
    accept = accept_thread_no_sync_call if with_call else accept_thread_no_sync
    old = sys.getswitchinterval()
    if interval is not None:
        sys.setswitchinterval(interval)
    broken = 0
    try:
        with ThreadPoolExecutor(max_workers=threads) as pool:
            for n in range(trips):
                trip = Trip(f"TN-{n:04d}")
                wins = sum(pool.map(lambda d: accept(trip, d), driver_ids(threads)))
                broken += wins > 1
    finally:
        sys.setswitchinterval(old)
    return broken


RACE_VARIANTS = (
    ("không có lời gọi, interval mặc định", dict(with_call=False, interval=None)),
    ("không có lời gọi, interval 1e-6 s", dict(with_call=False, interval=1e-6)),
    ("có một lời gọi hàm, interval mặc định", dict(with_call=True, interval=None)),
    ("có một lời gọi hàm, interval 1e-6 s", dict(with_call=True, interval=1e-6)),
)


def path_when_free(func) -> str:
    """Các lệnh bytecode chạy khi chuyến còn trống: từ LOAD_ATTR đọc driver_id tới STORE_ATTR ghi.

    Đây là chi tiết cài đặt của CPython (đổi theo phiên bản), chỉ dùng để quan sát.
    """
    ops = [i.opname for i in dis.get_instructions(func)]
    i_load, i_ret, i_store = ops.index("LOAD_ATTR"), ops.index("RETURN_VALUE"), ops.index("STORE_ATTR")
    return " ".join(ops[i_load : i_load + 2] + ops[i_ret + 1 : i_store + 1])


def race_table(rounds: int = 5) -> dict[str, list[int]]:
    """Mỗi biến thể chạy `rounds` lượt x 2000 chuyến x 4 thread; in số chuyến hỏng mỗi lượt."""
    gil = sys._is_gil_enabled()
    print(f"Python {sys.version.split()[0]}, GIL {'bật' if gil else 'tắt (free-threaded)'}, "
          f"load average đầu: {os.getloadavg()[0]:.1f}")
    print("bytecode khi chuyến còn trống:")
    print("  không có lời gọi:", path_when_free(accept_thread_no_sync))
    print("  có một lời gọi  :", path_when_free(accept_thread_no_sync_call))
    print(f"{'biến thể (2000 chuyến x 4 thread)':<42}{'số chuyến hỏng qua ' + str(rounds) + ' lượt'}")
    table: dict[str, list[int]] = {}
    for name, kwargs in RACE_VARIANTS:
        counts = [broken_trips(**kwargs) for _ in range(rounds)]
        table[name] = counts
        print(f"{name:<42}{counts}")
    return table


# --- 3. Semaphore: giới hạn số lời gọi đồng thời ----------------------------------------------
@dataclass
class Gauge:
    """Đếm số lời gọi đang chạy cùng lúc và đỉnh của nó."""

    in_flight: int = 0
    peak: int = 0


async def call_pricing(gauge: Gauge, limiter: asyncio.Semaphore | None) -> None:
    async def work() -> None:
        gauge.in_flight += 1
        gauge.peak = max(gauge.peak, gauge.in_flight)
        await asyncio.sleep(0.05)  # một lời gọi sang dịch vụ báo giá
        gauge.in_flight -= 1

    if limiter is None:
        await work()
    else:
        async with limiter:  # vào được khi còn "giấy phép"; ra khỏi khối là trả lại
            await work()


async def fan_out(calls: int, limit: int | None) -> tuple[int, float]:
    gauge = Gauge()
    limiter = asyncio.Semaphore(limit) if limit else None
    start = time.perf_counter()
    await asyncio.gather(*(call_pricing(gauge, limiter) for _ in range(calls)))
    return gauge.peak, time.perf_counter() - start


# --- 4. Queue: producer/consumer có backpressure -------------------------------------------------
@dataclass
class PipelineResult:
    processed: int
    peak_qsize: int
    producer_done_s: float
    all_done_s: float
    consumers_stopped: int


async def run_pipeline(
    items: int, *, maxsize: int, consumers: int = 3, use_shutdown: bool = False
) -> PipelineResult:
    """Producer đẩy yêu cầu tìm tài xế vào queue; `consumers` worker xử lý mỗi yêu cầu 0,02 s.

    maxsize > 0: queue đầy thì `await put()` treo producer lại (backpressure).
    Dừng: sentinel None mỗi consumer một cái, hoặc Queue.shutdown() (3.13+).
    """
    queue: asyncio.Queue[str | None] = asyncio.Queue(maxsize=maxsize)
    processed = 0
    stopped = 0
    peak = 0
    start = time.perf_counter()
    producer_done = 0.0

    async def producer() -> None:
        nonlocal peak, producer_done
        for n in range(1, items + 1):
            await queue.put(f"TN-{n:04d}")  # đầy thì chờ ở đây
            peak = max(peak, queue.qsize())
        if use_shutdown:
            queue.shutdown()  # consumer xử lý nốt phần còn lại rồi nhận QueueShutDown
        else:
            for _ in range(consumers):
                await queue.put(None)  # sentinel: "hết việc"
        producer_done = time.perf_counter() - start

    async def consumer() -> None:
        nonlocal processed, stopped
        while True:
            try:
                item = await queue.get()
            except asyncio.QueueShutDown:
                break
            try:
                if item is None:  # nhận sentinel thì thoát vòng lặp
                    break
                await asyncio.sleep(0.02)  # xử lý một yêu cầu
                processed += 1
            finally:
                queue.task_done()  # mỗi get() phải có đúng một task_done()
        stopped += 1

    workers = [asyncio.create_task(consumer()) for _ in range(consumers)]
    prod = asyncio.create_task(producer())
    await prod
    await queue.join()  # chờ mọi phần tử đã get() đều đã task_done()
    await asyncio.gather(*workers)
    return PipelineResult(processed, peak, producer_done, time.perf_counter() - start, stopped)


# --- Chạy và in kết quả ----------------------------------------------------------------------
async def run_async_demos() -> dict[str, object]:
    out: dict[str, object] = {}

    print("== 1. Race với asyncio: hai tài xế cùng nhận TN-0001 ==")
    wins2, trip2 = await race_asyncio(2, locked=False)
    print(f"không khóa, 2 tài xế : {wins2} người báo thắng, driver cuối = {trip2.driver_id}")
    wins20, trip20 = await race_asyncio(20, locked=False)
    print(f"không khóa, 20 tài xế: {wins20} người báo thắng, driver cuối = {trip20.driver_id}")
    wins_l, trip_l = await race_asyncio(20, locked=True)
    print(f"asyncio.Lock, 20 tài xế: {wins_l} người thắng, driver = {trip_l.driver_id}, status = {trip_l.status}")
    out.update(wins2=wins2, wins20=wins20, wins_l=wins_l, trip_l=trip_l)
    reentrant = await lock_is_not_reentrant()
    print("xin lock hai lần trong cùng coroutine:", reentrant)
    out["reentrant"] = reentrant
    return out


async def run_semaphore_queue_demos() -> dict[str, object]:
    out: dict[str, object] = {}
    print("\n== 3. Semaphore: 100 lời gọi báo giá, mỗi lời 0,05 s ==")
    peak_free, t_free = await fan_out(100, None)
    peak_lim, t_lim = await fan_out(100, 10)
    print(f"không giới hạn  : đỉnh đồng thời {peak_free:>3}, mất {t_free:.2f} s")
    print(f"Semaphore(10)   : đỉnh đồng thời {peak_lim:>3}, mất {t_lim:.2f} s")
    out.update(peak_free=peak_free, peak_lim=peak_lim, t_free=t_free, t_lim=t_lim)

    print("\n== 4. Queue: 30 yêu cầu, 3 consumer, mỗi yêu cầu 0,02 s ==")
    unb = await run_pipeline(30, maxsize=0)
    bnd = await run_pipeline(30, maxsize=5)
    shut = await run_pipeline(30, maxsize=5, use_shutdown=True)
    header = f"{'cấu hình':<26}{'xử lý':>6}{'peak qsize':>12}{'producer xong':>15}{'tất cả xong':>13}"
    print(header)
    for name, r in (("maxsize=0 (không giới hạn)", unb), ("maxsize=5 + sentinel", bnd), ("maxsize=5 + shutdown()", shut)):
        print(f"{name:<26}{r.processed:>6}{r.peak_qsize:>12}{r.producer_done_s:>13.2f} s{r.all_done_s:>11.2f} s")
    out.update(unb=unb, bnd=bnd, shut=shut)
    return out


def run_thread_demos() -> dict[str, object]:
    out: dict[str, object] = {}
    print("\n== 2. Race với thread ==")
    wins_u, trip_u = race_threads(8, locked=False)
    print(f"không khóa, 8 thread (có Barrier): {wins_u} người báo thắng, driver cuối = {trip_u.driver_id}")
    wins_l, trip_l = race_threads(8, locked=True)
    print(f"threading.Lock, 8 thread          : {wins_l} người thắng, driver = {trip_l.driver_id}")
    print()
    out.update(wins_u=wins_u, wins_l=wins_l, table=race_table())
    return out


def verify(r: dict[str, object]) -> None:
    checks: list[tuple[str, bool]] = []

    def check(name: str, ok: bool) -> None:
        checks.append((name, bool(ok)))

    check("asyncio không khóa: 2 tài xế đều báo thắng", r["wins2"] == 2)
    check("asyncio không khóa: 20 tài xế đều báo thắng", r["wins20"] == 20)
    check("asyncio.Lock: đúng 1 người thắng", r["wins_l"] == 1)
    check("asyncio.Lock: chuyến chuyển sang accepted",
          r["trip_l"].status is TripStatus.ACCEPTED and r["trip_l"].driver_id is not None)
    check("asyncio.Lock không tái nhập (lần xin thứ hai hết giờ)", r["reentrant"].startswith("TimeoutError"))
    check("thread không khóa + Barrier: 8 người đều báo thắng", r["wins_u"] == 8)
    check("threading.Lock: đúng 1 người thắng", r["wins_l_thread"] == 1)
    check("không Semaphore: đỉnh đồng thời = 100", r["peak_free"] == 100)
    check("Semaphore(10): đỉnh đồng thời = 10", r["peak_lim"] == 10)
    check("Semaphore(10) chậm hơn ~10 lần (>= 0,45 s)", r["t_lim"] >= 0.45)
    for key, name in (("unb", "maxsize=0"), ("bnd", "maxsize=5 + sentinel"), ("shut", "maxsize=5 + shutdown")):
        check(f"{name}: xử lý đủ 30 yêu cầu", r[key].processed == 30)
        check(f"{name}: cả 3 consumer đều dừng", r[key].consumers_stopped == 3)
    check("maxsize=5: peak qsize không vượt 5 (cả hai cách dừng)", r["bnd"].peak_qsize <= 5 and r["shut"].peak_qsize <= 5)
    check("maxsize=0: backlog lớn hơn 5 (producer không bị chặn)", r["unb"].peak_qsize > 5)
    check("backpressure: producer bị giữ lại (>= 0,05 s) khi queue đầy", r["bnd"].producer_done_s >= 0.05)
    check("không backpressure: producer xong gần như tức thì (< 0,05 s)", r["unb"].producer_done_s < 0.05)

    bad = [name for name, ok in checks if not ok]
    for name in bad:
        print(f"SAI: {name}")
    if bad:
        print(f"FAIL {len(checks) - len(bad)}/{len(checks)}")
        sys.exit(1)
    print(f"OK {len(checks)}/{len(checks)}")


def main() -> None:
    if "--race-table" in sys.argv[1:]:  # chỉ in bảng race (chạy được cả trên build free-threaded)
        race_table()
        return
    results: dict[str, object] = {}
    results.update(asyncio.run(run_async_demos()))
    thread = run_thread_demos()
    results.update(wins_u=thread["wins_u"], wins_l_thread=thread["wins_l"])
    results.update(asyncio.run(run_semaphore_queue_demos()))
    if "--verify" in sys.argv[1:]:
        print()
        verify(results)


if __name__ == "__main__":
    main()
