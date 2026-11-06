"""Bài 21: asyncio - coroutine, Task, timeout và hủy: gọi dịch vụ báo giá.

Chạy:   uv run --project examples/toi-noi-python/21-asyncio-task-timeout examples/toi-noi-python/21-asyncio-task-timeout/main.py
Verify: thêm --verify ở cuối lệnh trên.

"Dịch vụ báo giá" ở đây là mô phỏng: độ trễ mạng được thay bằng asyncio.sleep với jitter
sinh từ random.Random(seed), nên các con số logic lặp lại được; chỉ thời gian đo
(giây) dao động nhẹ giữa các lần chạy, vì vậy --verify so sánh theo ngưỡng.
"""

import asyncio
import logging
import random
import re
import sys
import time
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from decimal import ROUND_HALF_UP, Decimal
from enum import StrEnum
from typing import NamedTuple

# --- Mô hình cước dùng chung (giữ nguyên tên từ bài 03) ----------------------------------
BASE_FARE = Decimal("12000")
INCLUDED_KM = Decimal("2")
PRICE_PER_EXTRA_KM = Decimal("4500")




class ServiceKind(StrEnum):
    RIDE = "ride"
    FOOD = "food"
    PARCEL = "parcel"


class FarePolicy(NamedTuple):
    base_fare: Decimal
    included_km: Decimal
    price_per_extra_km: Decimal


STANDARD = FarePolicy(BASE_FARE, INCLUDED_KM, PRICE_PER_EXTRA_KM)


def calculate_fare(
    distance_km: Decimal, policy: FarePolicy = STANDARD, *, service: ServiceKind = ServiceKind.RIDE
) -> Decimal:
    """Cước VND, làm tròn ROUND_HALF_UP. Miền 0..100 km, ngoài miền -> ValueError.

    Ba dịch vụ cùng một công thức (mô hình học tập), nên `service` chỉ để giữ chữ ký chung.
    """
    if not Decimal("0") <= distance_km <= Decimal("100"):
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    extra = max(Decimal("0"), distance_km - policy.included_km) * policy.price_per_extra_km
    return (policy.base_fare + extra).quantize(Decimal("1"), rounding=ROUND_HALF_UP)


@dataclass(frozen=True)
class Money:
    amount: Decimal
    currency: str = "VND"


@dataclass(frozen=True)
class Quote:
    trip_distance_km: Decimal
    fare: Money
    expires_at: datetime


# --- Dịch vụ báo giá mô phỏng ----------------------------------------------------------
def make_latencies(count: int, *, seed: int = 42) -> list[float]:
    """Độ trễ mỗi lần gọi: 0,15 s cộng jitter 0..0,10 s, cố định theo seed."""
    rng = random.Random(seed)
    return [0.15 + rng.random() * 0.10 for _ in range(count)]


async def fetch_quote(distance_km: Decimal, latency: float) -> Quote:
    """Một lần gọi dịch vụ báo giá: chờ `latency` giây (I/O giả) rồi tính cước.

    `await asyncio.sleep` trả quyền điều khiển cho event loop trong lúc chờ,
    nên các coroutine khác chạy tiếp được.
    """
    await asyncio.sleep(latency)
    fare = calculate_fare(distance_km)  # ValueError nếu ngoài miền
    return Quote(distance_km, Money(fare), datetime.now(UTC) + timedelta(minutes=5))


DISTANCES = [Decimal(x) for x in ("0", "2", "2.5", "3", "4", "5", "6.5", "8", "10", "12")]


# --- 1. Tuần tự và đồng thời -------------------------------------------------------------
async def quotes_sequential(latencies: list[float]) -> list[Quote]:
    """await từng lời gọi một: tổng thời gian ~ tổng độ trễ."""
    return [await fetch_quote(d, lat) for d, lat in zip(DISTANCES, latencies)]


async def quotes_gather(latencies: list[float]) -> list[Quote]:
    """gather bọc mỗi coroutine thành Task rồi chờ tất cả: tổng thời gian ~ độ trễ lớn nhất."""
    return list(await asyncio.gather(*(fetch_quote(d, lat) for d, lat in zip(DISTANCES, latencies))))


async def quotes_taskgroup(latencies: list[float]) -> list[Quote]:
    """TaskGroup (3.11+): tạo Task trong khối async with, thoát khối khi tất cả xong."""
    async with asyncio.TaskGroup() as tg:
        tasks = [tg.create_task(fetch_quote(d, lat)) for d, lat in zip(DISTANCES, latencies)]
    return [t.result() for t in tasks]


async def timed(coro) -> tuple[object, float]:
    start = time.perf_counter()
    result = await coro
    return result, time.perf_counter() - start


# --- 2. Khi một lời gọi lỗi: gather và TaskGroup khác nhau ---------------------------------
async def slow_sibling(events: list[str], name: str) -> str:
    """Lời gọi chậm 0,3 s; ghi lại nó kết thúc hay bị hủy."""
    try:
        await asyncio.sleep(0.3)
        events.append(f"{name}: chạy xong")
        return name
    except asyncio.CancelledError:
        events.append(f"{name}: bị hủy")
        raise  # luôn ném lại, xem mục «Hủy» trong bài


async def failing_quote() -> Quote:
    """Lời gọi hỏng: khoảng cách 150 km nằm ngoài miền nên ném ValueError sau 0,05 s."""
    return await fetch_quote(Decimal("150"), 0.05)


async def gather_failure() -> list[str]:
    sibling_events: list[str] = []
    lines: list[str] = []
    try:
        await asyncio.gather(
            failing_quote(), slow_sibling(sibling_events, "A"), slow_sibling(sibling_events, "B")
        )
    except ValueError as exc:
        lines.append(f"gather ném ValueError: {exc}")
    # gather không hủy các Task còn lại: chờ thêm để thấy chúng vẫn chạy.
    await asyncio.sleep(0.4)
    # Thứ tự A/B kết thúc hoặc bị hủy không được asyncio bảo đảm, nên sắp xếp trước khi in.
    return lines + sorted(sibling_events)


async def taskgroup_failure() -> list[str]:
    sibling_events: list[str] = []
    lines: list[str] = []
    try:
        async with asyncio.TaskGroup() as tg:
            tg.create_task(failing_quote())
            tg.create_task(slow_sibling(sibling_events, "A"))
            tg.create_task(slow_sibling(sibling_events, "B"))
    except* ValueError as group:  # except* bắt từng loại trong ExceptionGroup
        lines.append(f"TaskGroup ném ExceptionGroup: {[str(e) for e in group.exceptions]}")
    await asyncio.sleep(0.4)
    return lines + sorted(sibling_events)  # sắp xếp: thứ tự hủy A, B không được bảo đảm


# --- 3. Timeout --------------------------------------------------------------------------
async def quote_with_timeout(distance_km: Decimal, latency: float, limit: float) -> tuple[str, Decimal]:
    """Chờ tối đa `limit` giây; hết giờ thì dùng cước tính cục bộ (nguồn "fallback")."""
    try:
        async with asyncio.timeout(limit):
            quote = await fetch_quote(distance_km, latency)
            return "service", quote.fare.amount
    except TimeoutError:
        # asyncio.timeout hủy đoạn bên trong, rồi đổi CancelledError thành TimeoutError ở đây.
        return "fallback", calculate_fare(distance_km)


# --- 4. Hủy và dọn dẹp -------------------------------------------------------------------
async def hold_quote(trip_id: str, holds: set[str], log: list[str]) -> None:
    """Giữ chỗ giá cho chuyến trong khi chờ khách xác nhận; PHẢI nhả khi bị hủy."""
    holds.add(trip_id)
    try:
        await asyncio.sleep(10)  # chờ khách bấm đặt
    except asyncio.CancelledError:
        log.append(f"{trip_id}: nhận CancelledError, đang dọn")
        raise  # ném lại để Task kết thúc ở trạng thái cancelled
    finally:
        holds.discard(trip_id)  # chạy cả khi xong bình thường, lỗi hay bị hủy
        log.append(f"{trip_id}: đã nhả giữ chỗ")


async def swallow_cancel(log: list[str]) -> str:
    """Anti-pattern: nuốt CancelledError rồi chạy tiếp."""
    try:
        await asyncio.sleep(10)
    except asyncio.CancelledError:
        log.append("nuốt CancelledError, tiếp tục")
    return "vẫn trả kết quả"


async def stubborn_quote() -> str:
    """Anti-pattern: nuốt CancelledError do timeout bắn vào rồi chạy tiếp thêm 0,1 s."""
    try:
        await asyncio.sleep(1)
    except asyncio.CancelledError:
        pass  # nuốt: timeout tưởng đã cắt xong
    await asyncio.sleep(0.1)
    return "chạy tiếp sau hạn"


async def timeout_vs_swallow() -> dict[str, object]:
    """asyncio.timeout(0,1) bọc coroutine nuốt hủy: khối không ném TimeoutError."""
    start = time.perf_counter()
    raised = False
    result = ""
    try:
        async with asyncio.timeout(0.1) as scope:
            result = await stubborn_quote()
    except TimeoutError:
        raised = True
    return {
        "raised": raised,
        "expired": scope.expired(),
        "result": result,
        "elapsed": time.perf_counter() - start,
    }


async def cancellation_demo() -> dict[str, object]:
    holds: set[str] = set()
    log: list[str] = []
    task = asyncio.create_task(hold_quote("TN-0001", holds, log))
    await asyncio.sleep(0.1)
    held_before = sorted(holds)
    task.cancel()  # chỉ xin hủy; Task nhận CancelledError tại điểm await kế tiếp
    try:
        await task
    except asyncio.CancelledError:
        log.append("người gọi thấy CancelledError khi await task")
    swallow_log: list[str] = []
    bad = asyncio.create_task(swallow_cancel(swallow_log))
    await asyncio.sleep(0.05)
    bad.cancel()
    result = await bad
    return {
        "held_before": held_before,
        "held_after": sorted(holds),
        "log": log,
        "cancelled": task.cancelled(),
        "swallow_result": result,
        "swallow_cancelled": bad.cancelled(),
        "swallow_log": swallow_log,
        "timeout_swallow": await timeout_vs_swallow(),
    }


# --- 5. Không chặn event loop ------------------------------------------------------------
def legacy_blocking_call(seconds: float) -> None:
    """Giả lập SDK đồng bộ (hoặc tính toán nặng): time.sleep giữ nguyên thread."""
    time.sleep(seconds)


async def blocking_in_coroutine(seconds: float) -> None:
    legacy_blocking_call(seconds)  # SAI: gọi thẳng trong coroutine, event loop đứng


async def offloaded_to_thread(seconds: float) -> None:
    await asyncio.to_thread(legacy_blocking_call, seconds)  # chạy ở thread khác


async def heartbeat(stop: asyncio.Event, gaps: list[float], interval: float = 0.05) -> None:
    """Tick mỗi 0,05 s và ghi khoảng cách thật giữa hai tick: loop bị chặn thì khoảng cách phình ra."""
    loop = asyncio.get_running_loop()
    last = loop.time()
    while not stop.is_set():
        await asyncio.sleep(interval)
        now = loop.time()
        gaps.append(now - last)
        last = now


class SlowCallbackCollector(logging.Handler):
    """Gom cảnh báo "took N seconds" mà asyncio ghi ở chế độ debug."""

    def __init__(self) -> None:
        super().__init__()
        self.durations: list[float] = []

    def emit(self, record: logging.LogRecord) -> None:
        match = re.search(r"took ([0-9.]+) seconds", record.getMessage())
        if match:
            self.durations.append(float(match.group(1)))


async def max_gap_while(work) -> float:
    stop = asyncio.Event()
    gaps: list[float] = []
    beat = asyncio.create_task(heartbeat(stop, gaps))
    await asyncio.sleep(0.2)  # cho heartbeat chạy vài nhịp trước khi tác vụ chặn bắt đầu
    await work
    await asyncio.sleep(0.1)
    stop.set()
    await beat
    return max(gaps)


async def blocking_demo() -> dict[str, object]:
    collector = SlowCallbackCollector()
    logging.getLogger("asyncio").addHandler(collector)
    try:
        gap_blocked = await max_gap_while(blocking_in_coroutine(0.5))
        slow_blocked = list(collector.durations)
        collector.durations.clear()
        gap_thread = await max_gap_while(offloaded_to_thread(0.5))
        slow_thread = list(collector.durations)
    finally:
        logging.getLogger("asyncio").removeHandler(collector)
    return {
        "gap_blocked": gap_blocked,
        "gap_thread": gap_thread,
        "slow_blocked": slow_blocked,
        "slow_thread": slow_thread,
    }


# --- Chạy demo và in kết quả --------------------------------------------------------------
async def run_demos() -> dict[str, object]:
    results: dict[str, object] = {}
    latencies = make_latencies(len(DISTANCES))
    results["sum_latency"] = sum(latencies)
    results["max_latency"] = max(latencies)

    print("== 1. Tuần tự và đồng thời (10 lời gọi báo giá) ==")
    print(f"độ trễ giả: tổng {sum(latencies):.2f} s, lớn nhất {max(latencies):.2f} s")
    seq, t_seq = await timed(quotes_sequential(latencies))
    gat, t_gat = await timed(quotes_gather(latencies))
    grp, t_grp = await timed(quotes_taskgroup(latencies))
    print(f"tuần tự   : {t_seq:.2f} s")
    print(f"gather    : {t_gat:.2f} s")
    print(f"TaskGroup : {t_grp:.2f} s")
    fares = [q.fare.amount for q in gat]
    print("cước theo thứ tự đầu vào:", ", ".join(str(f) for f in fares))
    results.update(t_seq=t_seq, t_gat=t_gat, t_grp=t_grp, fares=fares)
    results["same_seq_gather"] = [q.fare for q in seq] == [q.fare for q in gat] == [q.fare for q in grp]

    print("\n== 2. Một lời gọi lỗi: gather và TaskGroup ==")
    g_events = await gather_failure()
    print("gather:")
    for line in g_events:
        print("  ", line)
    t_events = await taskgroup_failure()
    print("TaskGroup:")
    for line in t_events:
        print("  ", line)
    results.update(g_events=g_events, t_events=t_events)

    print("\n== 3. Timeout 0,30 s ==")
    fast, t_fast = await timed(quote_with_timeout(Decimal("5"), 0.1, 0.30))
    slow, t_slow = await timed(quote_with_timeout(Decimal("5"), 2.0, 0.30))
    print(f"dịch vụ nhanh (0,10 s): nguồn={fast[0]}, cước={fast[1]}, mất {t_fast:.2f} s")
    print(f"dịch vụ chậm  (2,00 s): nguồn={slow[0]}, cước={slow[1]}, mất {t_slow:.2f} s")
    results.update(fast=fast, slow=slow, t_slow=t_slow)

    print("\n== 4. Hủy và dọn dẹp ==")
    cancel = await cancellation_demo()
    print("đang giữ chỗ trước khi hủy:", cancel["held_before"])
    for line in cancel["log"]:  # type: ignore[union-attr]
        print("  ", line)
    print("đang giữ chỗ sau khi hủy  :", cancel["held_after"])
    print("task.cancelled() =", cancel["cancelled"])
    print("nuốt CancelledError ->", cancel["swallow_result"], "| cancelled() =", cancel["swallow_cancelled"])
    ts = cancel["timeout_swallow"]
    print(
        f"timeout 0,10 s bọc coroutine nuốt hủy -> TimeoutError: {ts['raised']}, "
        f"expired(): {ts['expired']}, kết quả: {ts['result']}, mất {ts['elapsed']:.2f} s"
    )
    results["cancel"] = cancel
    return results


def run_blocking() -> dict[str, object]:
    """Chạy riêng với debug=True để asyncio cảnh báo callback chạy quá 0,1 s."""
    print("\n== 5. Chặn event loop ==")
    out = asyncio.run(blocking_demo(), debug=True)
    print(f"gọi sync trực tiếp 0,5 s : khoảng cách tick lớn nhất {out['gap_blocked']:.2f} s (danh định 0,05 s)")
    print(f"asyncio.to_thread 0,5 s  : khoảng cách tick lớn nhất {out['gap_thread']:.2f} s")
    print(f"cảnh báo slow callback (debug): trực tiếp {out['slow_blocked']}, to_thread {out['slow_thread']}")
    return out


def verify(results: dict[str, object], blocking: dict[str, object]) -> None:
    checks: list[tuple[str, bool]] = []

    def check(name: str, ok: bool) -> None:
        checks.append((name, bool(ok)))

    # Phần logic: lặp lại tuyệt đối.
    check("cước 0/2/2,5/5 km = 12000/12000/14250/25500",
          [results["fares"][i] for i in (0, 1, 2, 5)] == [Decimal(x) for x in ("12000", "12000", "14250", "25500")])
    check("ba cách gọi cho cùng kết quả", results["same_seq_gather"])
    g, t = results["g_events"], results["t_events"]
    check("gather: ném ValueError ra ngoài", any(e.startswith("gather ném ValueError") for e in g))
    check("gather: hai lời gọi còn lại vẫn chạy xong", "A: chạy xong" in g and "B: chạy xong" in g)
    check("TaskGroup: ném ExceptionGroup chứa ValueError", any("ExceptionGroup" in e for e in t))
    check("TaskGroup: hai lời gọi còn lại bị hủy", "A: bị hủy" in t and "B: bị hủy" in t)
    check("timeout: dịch vụ nhanh dùng nguồn service", results["fast"] == ("service", Decimal("25500")))
    check("timeout: dịch vụ chậm chuyển sang fallback", results["slow"] == ("fallback", Decimal("25500")))
    c = results["cancel"]
    check("hủy: giữ chỗ được nhả", c["held_before"] == ["TN-0001"] and c["held_after"] == [])
    check("hủy: Task ở trạng thái cancelled", c["cancelled"] is True)
    check("hủy: nuốt CancelledError thì Task không còn cancelled",
          c["swallow_result"] == "vẫn trả kết quả" and c["swallow_cancelled"] is False)
    ts = c["timeout_swallow"]
    check("timeout bọc coroutine nuốt hủy: không có TimeoutError dù expired() là True",
          ts["raised"] is False and ts["expired"] is True and ts["result"] == "chạy tiếp sau hạn")
    check("timeout bọc coroutine nuốt hủy: khối vẫn chạy ~0,2 s (quá hạn 0,1 s)", ts["elapsed"] >= 0.19)
    # Phần thời gian: so theo ngưỡng rộng vì phụ thuộc máy.
    check("tuần tự >= tổng độ trễ", results["t_seq"] >= results["sum_latency"])
    check("gather < 1/3 thời gian tuần tự", results["t_gat"] < results["t_seq"] / 3)
    check("TaskGroup < 1/3 thời gian tuần tự", results["t_grp"] < results["t_seq"] / 3)
    check("timeout dừng sau 0,30 s và rất xa độ trễ 2 s của dịch vụ", 0.28 <= results["t_slow"] < 1.5)
    check("gọi sync trực tiếp làm loop đứng >= 0,45 s", blocking["gap_blocked"] >= 0.45)
    check("to_thread giữ tick đều: khoảng cách lớn nhất < nửa lúc chặn", blocking["gap_thread"] < blocking["gap_blocked"] / 2)
    check("debug mode cảnh báo callback chậm khi chặn loop", len(blocking["slow_blocked"]) >= 1)

    bad = [name for name, ok in checks if not ok]
    for name in bad:
        print(f"SAI: {name}")
    if bad:
        print(f"FAIL {len(checks) - len(bad)}/{len(checks)}")
        sys.exit(1)
    print(f"OK {len(checks)}/{len(checks)}")


def main() -> None:
    results = asyncio.run(run_demos())  # điểm vào chuẩn: tạo loop, chạy, đóng loop
    blocking = run_blocking()
    if "--verify" in sys.argv[1:]:
        print()
        verify(results, blocking)


if __name__ == "__main__":
    main()
