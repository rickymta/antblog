"""Bài 17: decorator và context manager.

Bài toán: gọi dịch vụ báo giá có thể lỗi tạm thời (retry), đo thời gian mỗi lần gọi (timed),
và gán tài xế cho chuyến theo kiểu «hoặc xong hết, hoặc không đổi gì» (transaction giả trên dict).
Mọi thời gian chờ đều đi qua một Timer (đồng hồ đo và chờ) thay thế được, nên lab không ngủ thật khi kiểm thử.

Chạy:     uv run --project examples/toi-noi-python/17-decorator-context-manager examples/toi-noi-python/17-decorator-context-manager/main.py
Kiểm tra: thêm --verify ở cuối.
"""

import copy
import functools
import logging
import sys
import time
from collections.abc import Callable, Iterator
from contextlib import ExitStack, contextmanager, suppress
from decimal import ROUND_HALF_UP, Decimal
from enum import StrEnum
from typing import Any, NamedTuple, Protocol

# ---------------------------------------------------------------- miền (rút gọn từ bài 10 và 13)

MAX_KM = Decimal("100")


class ServiceKind(StrEnum):
    RIDE = "ride"
    FOOD = "food"
    PARCEL = "parcel"


class FarePolicy(NamedTuple):
    base_fare: Decimal = Decimal("12000")
    included_km: Decimal = Decimal("2")
    price_per_extra_km: Decimal = Decimal("4500")


STANDARD = FarePolicy()


def calculate_fare(
    distance_km: Decimal,
    policy: FarePolicy = STANDARD,
    *,
    service: ServiceKind = ServiceKind.RIDE,
) -> Decimal:
    """Cước = giá cơ bản + phần vượt x giá mỗi km vượt, làm tròn ROUND_HALF_UP."""
    if not distance_km.is_finite() or not Decimal("0") <= distance_km <= MAX_KM:
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    ServiceKind(service)  # ép về Enum: dịch vụ lạ bị ValueError ngay tại đây
    extra_km = max(Decimal("0"), distance_km - policy.included_km)
    total = policy.base_fare + extra_km * policy.price_per_extra_km
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)


class TripStatus(StrEnum):
    REQUESTED = "requested"
    SEARCHING = "searching"
    ACCEPTED = "accepted"
    ARRIVING = "arriving"
    PICKED_UP = "picked_up"
    IN_TRANSIT = "in_transit"
    COMPLETED = "completed"
    CANCELLED = "cancelled"


# ---------------------------------------------------------------- đồng hồ thay thế được

class Timer(Protocol):
    """Hai việc decorator cần từ thời gian: đọc đồng hồ đo và chờ."""

    def perf_counter(self) -> float: ...
    def sleep(self, seconds: float) -> None: ...


class SystemTimer:
    def perf_counter(self) -> float:
        return time.perf_counter()

    def sleep(self, seconds: float) -> None:
        time.sleep(seconds)


class FakeTimer:
    """Đồng hồ giả: sleep không chờ thật, chỉ ghi lại và cộng vào thời gian đã trôi."""

    def __init__(self) -> None:
        self.now = 0.0
        self.sleeps: list[float] = []

    def perf_counter(self) -> float:
        return self.now

    def sleep(self, seconds: float) -> None:
        self.sleeps.append(seconds)
        self.now += seconds

    def advance(self, seconds: float) -> None:
        self.now += seconds


SYSTEM_TIMER = SystemTimer()

# ---------------------------------------------------------------- logging có thể bắt lại

LOGGER = logging.getLogger("toinoi.lab17")
LOGGER.setLevel(logging.DEBUG)
LOGGER.propagate = False  # không tràn ra root logger: output của lab chỉ gồm những gì ta in
LOGGER.addHandler(logging.NullHandler())


class ListHandler(logging.Handler):
    def __init__(self, sink: list[str]) -> None:
        super().__init__()
        self.sink = sink

    def emit(self, record: logging.LogRecord) -> None:
        self.sink.append(f"{record.levelname} {record.getMessage()}")


@contextmanager
def capture_logs() -> Iterator[list[str]]:
    """Gắn handler tạm; luôn gỡ ra khi rời khối with, kể cả khi có lỗi."""
    lines: list[str] = []
    handler = ListHandler(lines)
    LOGGER.addHandler(handler)
    try:
        yield lines
    finally:
        LOGGER.removeHandler(handler)


# ---------------------------------------------------------------- closure

def make_fare_quoter(policy: FarePolicy) -> Callable[[Decimal], Decimal]:
    """Hàm trả về hàm: quote nhớ `policy` của lần gọi đã tạo ra nó (closure)."""

    def quote(distance_km: Decimal) -> Decimal:
        return calculate_fare(distance_km, policy)

    return quote


def closure_demo() -> dict[str, Any]:
    policy_5000 = FarePolicy(price_per_extra_km=Decimal("5000"))
    quote_5000 = make_fare_quoter(policy_5000)
    # Bẫy kinh điển: closure giữ BIẾN, không giữ giá trị tại lúc tạo.
    late: list[Callable[[], int]] = [lambda: i for i in range(3)]
    fixed: list[Callable[[], int]] = [lambda i=i: i for i in range(3)]  # type: ignore[misc]  # mặc định i=i chốt giá trị ngay lúc định nghĩa
    return {
        "quote_standard_5km": make_fare_quoter(STANDARD)(Decimal("5")),
        "quote_5000_5km": quote_5000(Decimal("5")),
        "free_vars": quote_5000.__code__.co_freevars,
        "captured_policy_is_5000": quote_5000.__closure__[0].cell_contents == policy_5000,  # type: ignore[index]
        "late_binding": [f() for f in late],
        "fixed_binding": [f() for f in fixed],
    }


# ---------------------------------------------------------------- decorator cơ bản và functools.wraps

CALL_LOG: list[str] = []


def record_naive[**P, R](func: Callable[P, R]) -> Callable[P, R]:
    def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
        CALL_LOG.append(func.__name__)
        return func(*args, **kwargs)

    return wrapper  # quên functools.wraps: wrapper mang tên và docstring của chính nó


def record[**P, R](func: Callable[P, R]) -> Callable[P, R]:
    @functools.wraps(func)  # sao chép __name__, __doc__, __module__... và đặt __wrapped__
    def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
        CALL_LOG.append(func.__name__)
        return func(*args, **kwargs)

    return wrapper


def price_trip(distance_km: Decimal) -> Decimal:
    """Báo giá một chuyến theo chính sách chuẩn."""
    return calculate_fare(distance_km)


def wraps_demo() -> dict[str, Any]:
    CALL_LOG.clear()
    naive = record_naive(price_trip)  # `@record_naive` trên dòng def chỉ là cú pháp cho dòng này
    good = record(price_trip)
    result = good(Decimal("2.5"))
    naive(Decimal("2.5"))
    return {
        "naive_name": naive.__name__,
        "naive_doc": naive.__doc__,
        "good_name": good.__name__,
        "good_doc": good.__doc__,
        "good_wrapped_is_original": good.__wrapped__ is price_trip,  # type: ignore[attr-defined]
        "result": result,
        "call_log": list(CALL_LOG),
    }


# ---------------------------------------------------------------- decorator có tham số: retry

class PricingUnavailable(Exception):
    """Lỗi tạm thời của dịch vụ báo giá (kiểu HTTP 503): đáng thử lại."""


def retry[**P, R](
    times: int = 3,
    delay: float = 0.0,
    backoff: float = 1.0,
    *,
    on: tuple[type[BaseException], ...] = (PricingUnavailable,),
    timer: Timer = SYSTEM_TIMER,
) -> Callable[[Callable[P, R]], Callable[P, R]]:
    """Gọi tối đa `times` lần (tính cả lần đầu), chỉ thử lại khi gặp lỗi trong `on`.

    Chờ delay, delay*backoff, delay*backoff^2... giữa các lần. Hết lượt thì ném lại lỗi cuối.
    """
    if isinstance(times, bool) or not isinstance(times, int):
        raise TypeError("retry cần ngoặc: viết @retry(...) hoặc @retry(), không viết @retry trần")
    if times < 1:
        raise ValueError(f"times phải >= 1, nhận {times}")

    def decorator(func: Callable[P, R]) -> Callable[P, R]:
        @functools.wraps(func)
        def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
            for attempt in range(1, times + 1):
                try:
                    return func(*args, **kwargs)
                except on as exc:
                    if attempt == times:
                        raise  # hết lượt: trả lại đúng lỗi gốc, không bọc
                    wait = delay * backoff ** (attempt - 1)
                    LOGGER.warning("%s lần %d/%d lỗi (%s), chờ %.1f s", func.__name__, attempt, times, exc, wait)
                    timer.sleep(wait)
            raise AssertionError("không tới được đây")  # để mypy biết mọi nhánh đều return/raise

        return wrapper

    return decorator


class FlakyPricing:
    """Dịch vụ báo giá giả: `fail_first` lần gọi đầu ném PricingUnavailable, sau đó mới trả giá."""

    def __init__(self, fail_first: int, timer: FakeTimer, latency: float = 0.2) -> None:
        self.fail_first = fail_first
        self.timer = timer
        self.latency = latency
        self.calls = 0

    def price(self, distance_km: Decimal) -> Decimal:
        self.calls += 1
        self.timer.advance(self.latency)  # mỗi lần gọi tốn latency giây (thời gian giả)
        if self.calls <= self.fail_first:
            raise PricingUnavailable(f"503 lần {self.calls}")
        return calculate_fare(distance_km)


def retry_demo() -> dict[str, Any]:
    out: dict[str, Any] = {}
    km = Decimal("2.5")

    # Lỗi 2 lần rồi thành công: chờ 0,5 s rồi 1,0 s (backoff x2)
    timer = FakeTimer()
    backend = FlakyPricing(fail_first=2, timer=timer)
    with capture_logs() as lines:
        quote = retry(times=3, delay=0.5, backoff=2.0, timer=timer)(backend.price)
        out["ok_result"] = quote(km)
    out["ok_calls"], out["ok_sleeps"], out["ok_logs"] = backend.calls, timer.sleeps, lines

    # Hết lượt: lỗi cuối được ném lại nguyên vẹn
    timer = FakeTimer()
    backend = FlakyPricing(fail_first=5, timer=timer)
    with capture_logs():
        quote = retry(times=3, delay=0.5, timer=timer)(backend.price)
        try:
            quote(km)
            out["exhausted_error"] = None
        except PricingUnavailable as exc:
            out["exhausted_error"] = str(exc)
    out["exhausted_calls"], out["exhausted_sleeps"] = backend.calls, timer.sleeps

    # Lỗi nghiệp vụ (ValueError) không nằm trong `on`: không thử lại, gọi đúng 1 lần
    timer = FakeTimer()
    backend = FlakyPricing(fail_first=0, timer=timer)
    quote = retry(times=3, delay=0.5, timer=timer)(backend.price)
    try:
        quote(Decimal("150"))
        out["domain_error"] = None
    except ValueError as exc:
        out["domain_error"] = str(exc)
    out["domain_calls"], out["domain_sleeps"] = backend.calls, timer.sleeps

    # Quên ngoặc: @retry trần truyền hàm vào chỗ của `times`
    try:
        retry(backend.price)  # type: ignore[arg-type]
        out["bare_error"] = None
    except TypeError as exc:
        out["bare_error"] = str(exc)
    return out


# ---------------------------------------------------------------- timed và thứ tự chồng decorator

def timed[**P, R](*, timer: Timer = SYSTEM_TIMER, label: str | None = None) -> Callable[[Callable[P, R]], Callable[P, R]]:
    """Ghi log thời gian mỗi lần gọi, kể cả khi hàm ném lỗi (finally)."""

    def decorator(func: Callable[P, R]) -> Callable[P, R]:
        name = label or func.__name__

        @functools.wraps(func)
        def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
            start = timer.perf_counter()
            try:
                return func(*args, **kwargs)
            finally:
                LOGGER.info("%s mất %.3f s", name, timer.perf_counter() - start)

        return wrapper

    return decorator


def stacking_demo() -> dict[str, Any]:
    km = Decimal("2.5")
    out: dict[str, Any] = {}
    for order in ("timed_outside", "timed_inside"):
        timer = FakeTimer()
        backend = FlakyPricing(fail_first=2, timer=timer)
        retrying = retry(times=3, delay=0.5, backoff=2.0, timer=timer)
        timing = timed(timer=timer, label="price")
        # `@` chỉ là cú pháp: decorator nằm gần def nhất được áp dụng trước.
        quote = timing(retrying(backend.price)) if order == "timed_outside" else retrying(timing(backend.price))
        with capture_logs() as lines:
            quote(km)
        out[order] = [line for line in lines if line.startswith("INFO")]
    return out


# ---------------------------------------------------------------- context manager: transaction giả

type TripDb = dict[str, dict[str, dict[str, str | int | None]]]


def new_db() -> TripDb:
    return {
        "trips": {
            "TN-0001": {"status": TripStatus.REQUESTED.value, "driver_id": None},
            "TN-0002": {"status": TripStatus.REQUESTED.value, "driver_id": None},
        },
        "drivers": {
            "DR-001": {"open_trips": 0},
            "DR-002": {"open_trips": 0},
        },
    }


@contextmanager
def transaction(db: TripDb) -> Iterator[TripDb]:
    """Transaction giả: làm việc trên bản sao; êm thì commit (ghi đè db), lỗi thì rollback."""
    working = copy.deepcopy(db)
    try:
        yield working
    except Exception:
        LOGGER.info("rollback")
        raise  # rollback xong vẫn ném lỗi lên: người gọi phải biết việc đã thất bại
    else:
        db.clear()
        db.update(working)  # commit
        LOGGER.info("commit")


def assign_driver(db: TripDb, trip_id: str, driver_id: str) -> None:
    with transaction(db) as tx:
        trip = tx["trips"][trip_id]
        trip["driver_id"] = driver_id
        trip["status"] = TripStatus.ACCEPTED.value
        tx["drivers"][driver_id]["open_trips"] += 1  # type: ignore[operator]  # KeyError nếu tài xế lạ


def assign_driver_no_tx(db: TripDb, trip_id: str, driver_id: str) -> None:
    """Cùng việc nhưng ghi thẳng vào db: lỗi giữa chừng để lại trạng thái dở dang."""
    trip = db["trips"][trip_id]
    trip["driver_id"] = driver_id
    trip["status"] = TripStatus.ACCEPTED.value
    db["drivers"][driver_id]["open_trips"] += 1  # type: ignore[operator]


def transaction_demo() -> dict[str, Any]:
    out: dict[str, Any] = {}
    db = new_db()
    with capture_logs() as lines:
        assign_driver(db, "TN-0001", "DR-001")
        out["after_commit"] = copy.deepcopy(db["trips"]["TN-0001"]), copy.deepcopy(db["drivers"]["DR-001"])

        before = copy.deepcopy(db)
        try:
            assign_driver(db, "TN-0002", "DR-999")  # tài xế không tồn tại
            out["rollback_error"] = None
        except KeyError as exc:
            out["rollback_error"] = repr(exc)
        out["db_unchanged_after_rollback"] = db == before
        out["trip2_after_rollback"] = dict(db["trips"]["TN-0002"])
    out["tx_logs"] = lines

    db2 = new_db()
    try:
        assign_driver_no_tx(db2, "TN-0002", "DR-999")
    except KeyError:
        pass
    out["trip2_without_tx"] = dict(db2["trips"]["TN-0002"])
    return out


# ---------------------------------------------------------------- context manager dạng class và ExitStack

class DriverBusy(Exception):
    pass


class DriverLock:
    """Giữ chỗ một tài xế trong tập `busy`; trả lại khi rời khối with dù có lỗi.

    Chỉ an toàn khi dùng tuần tự trong một luồng: `in busy` rồi `add` không nguyên tử.
    Nhiều luồng cần thêm threading.Lock (bài 23).
    """

    def __init__(self, busy: set[str], driver_id: str) -> None:
        self.busy = busy
        self.driver_id = driver_id

    def __enter__(self) -> str:
        if self.driver_id in self.busy:
            raise DriverBusy(self.driver_id)
        self.busy.add(self.driver_id)
        return self.driver_id

    def __exit__(self, exc_type: object, exc: object, tb: object) -> None:
        # Trả None (falsy): exception đang bay vẫn tiếp tục bay. Trả True mới là nuốt lỗi.
        self.busy.discard(self.driver_id)


def lock_demo() -> dict[str, Any]:
    out: dict[str, Any] = {}
    busy: set[str] = set()
    with DriverLock(busy, "DR-001") as driver_id:
        out["held_inside"] = (driver_id, sorted(busy))
    out["released_after"] = sorted(busy)

    try:
        with DriverLock(busy, "DR-001"):
            raise RuntimeError("lỗi giữa chừng")
    except RuntimeError:
        pass
    out["released_after_error"] = sorted(busy)

    # ExitStack: số tài nguyên chỉ biết lúc chạy. Một cái lỗi thì những cái đã lấy được trả lại.
    busy = {"DR-002"}  # DR-002 đang bận ở nơi khác
    try:
        with ExitStack() as stack:
            for driver in ("DR-001", "DR-002", "DR-003"):
                stack.enter_context(DriverLock(busy, driver))
        out["stack_error"] = None
    except DriverBusy as exc:
        out["stack_error"] = f"DriverBusy({exc})"
    out["busy_after_failed_stack"] = sorted(busy)

    busy = set()
    with ExitStack() as stack:
        for driver in ("DR-001", "DR-002", "DR-003"):
            stack.enter_context(DriverLock(busy, driver))
        out["stack_all_held"] = sorted(busy)
    out["stack_all_released"] = sorted(busy)

    # suppress: nuốt đúng loại lỗi chỉ định, mọi lỗi khác vẫn bay lên
    status: dict[str, str] = {}
    reached: list[str] = []
    with suppress(KeyError):
        status["TN-9999"]  # noqa: B018  # KeyError: thoát khỏi khối with ngay tại đây
        reached.append("dòng sau lỗi")  # không bao giờ chạy
    reached.append("sau with")
    out["suppress_reached"] = reached
    try:
        with suppress(KeyError):
            int("abc")  # ValueError không nằm trong danh sách nuốt
    except ValueError:
        out["suppress_other_error_propagates"] = True
    return out


@contextmanager
def tracer(name: str, log: list[str]) -> Iterator[None]:
    log.append(f"enter {name}")
    try:
        yield
    finally:
        log.append(f"exit {name}")


@contextmanager
def leaky(log: list[str]) -> Iterator[None]:
    log.append("open")
    yield
    log.append("close")  # không có try/finally: lỗi trong khối with làm dòng này không chạy


@contextmanager
def safe(log: list[str]) -> Iterator[None]:
    log.append("open")
    try:
        yield
    finally:
        log.append("close")


def generator_cm_demo() -> dict[str, Any]:
    order: list[str] = []
    with tracer("A", order), tracer("B", order):
        order.append("body")

    leaky_log: list[str] = []
    safe_log: list[str] = []
    for cm, log in ((leaky, leaky_log), (safe, safe_log)):
        try:
            with cm(log):
                raise RuntimeError("lỗi trong thân with")
        except RuntimeError:
            pass
    return {"order": order, "leaky": leaky_log, "safe": safe_log}


# ---------------------------------------------------------------- in kết quả và --verify

def show() -> None:
    c = closure_demo()
    print("== closure ==")
    for k, v in c.items():
        print(f"{k:24}:", v)

    w = wraps_demo()
    print("== functools.wraps ==")
    for k, v in w.items():
        print(f"{k:24}:", v)

    r = retry_demo()
    print("== retry ==")
    for k, v in r.items():
        print(f"{k:24}:", v)

    s = stacking_demo()
    print("== thứ tự chồng decorator ==")
    for k, v in s.items():
        print(f"{k:14}:", v)

    t = transaction_demo()
    print("== transaction giả ==")
    for k, v in t.items():
        print(f"{k:28}:", v)

    lk = lock_demo()
    print("== context manager dạng class, ExitStack ==")
    for k, v in lk.items():
        print(f"{k:24}:", v)

    g = generator_cm_demo()
    print("== @contextmanager: thứ tự thoát, try/finally ==")
    for k, v in g.items():
        print(f"{k:8}:", v)


def verify() -> None:
    c, w, r, s = closure_demo(), wraps_demo(), retry_demo(), stacking_demo()
    t, lk, g = transaction_demo(), lock_demo(), generator_cm_demo()
    checks = [
        ("closure nhớ policy", c["quote_standard_5km"] == Decimal("25500") and c["quote_5000_5km"] == Decimal("27000")
         and c["free_vars"] == ("policy",) and c["captured_policy_is_5000"]),
        ("closure late binding", c["late_binding"] == [2, 2, 2] and c["fixed_binding"] == [0, 1, 2]),
        ("thiếu wraps mất tên", w["naive_name"] == "wrapper" and w["naive_doc"] is None),
        ("wraps giữ tên, doc, __wrapped__", w["good_name"] == "price_trip" and w["good_doc"] is not None
         and w["good_wrapped_is_original"] and w["result"] == Decimal("14250") and w["call_log"] == ["price_trip", "price_trip"]),
        ("retry thành công lần 3", r["ok_result"] == Decimal("14250") and r["ok_calls"] == 3 and r["ok_sleeps"] == [0.5, 1.0]),
        ("retry ghi log mỗi lần lỗi", len(r["ok_logs"]) == 2 and all(line.startswith("WARNING") for line in r["ok_logs"])),
        ("retry hết lượt ném lại lỗi cuối", r["exhausted_error"] == "503 lần 3" and r["exhausted_calls"] == 3
         and r["exhausted_sleeps"] == [0.5, 0.5]),
        ("retry không thử lại lỗi nghiệp vụ", r["domain_error"] == "khoảng cách ngoài miền 0..100 km: 150"
         and r["domain_calls"] == 1 and r["domain_sleeps"] == []),
        ("@retry trần báo TypeError", r["bare_error"] is not None and "ngoặc" in r["bare_error"]),
        ("timed ngoài retry đo tổng", s["timed_outside"] == ["INFO price mất 2.100 s"]),
        ("timed trong retry đo từng lần", s["timed_inside"] == ["INFO price mất 0.200 s"] * 3),
        ("transaction commit", t["after_commit"] == ({"status": "accepted", "driver_id": "DR-001"}, {"open_trips": 1})),
        ("transaction rollback giữ nguyên db", t["rollback_error"] == "KeyError('DR-999')" and t["db_unchanged_after_rollback"]
         and t["trip2_after_rollback"] == {"status": "requested", "driver_id": None}),
        ("log commit rồi rollback", t["tx_logs"] == ["INFO commit", "INFO rollback"]),
        ("không transaction thì dở dang", t["trip2_without_tx"] == {"status": "accepted", "driver_id": "DR-999"}),
        ("DriverLock trả chỗ", lk["held_inside"] == ("DR-001", ["DR-001"]) and lk["released_after"] == []
         and lk["released_after_error"] == []),
        ("ExitStack trả phần đã lấy khi lỗi", lk["stack_error"] == "DriverBusy(DR-002)" and lk["busy_after_failed_stack"] == ["DR-002"]),
        ("ExitStack giữ và trả cả ba", lk["stack_all_held"] == ["DR-001", "DR-002", "DR-003"] and lk["stack_all_released"] == []
         ),
        ("suppress chỉ nuốt loại lỗi chỉ định", lk["suppress_reached"] == ["sau with"] and lk["suppress_other_error_propagates"]),
        ("thoát theo thứ tự ngược", g["order"] == ["enter A", "enter B", "body", "exit B", "exit A"]),
        ("thiếu try/finally thì bỏ dọn dẹp", g["leaky"] == ["open"] and g["safe"] == ["open", "close"]),
    ]
    failed = [name for name, ok in checks if not ok]
    passed = len(checks) - len(failed)
    if failed:
        print("Ca sai:", ", ".join(failed))
        print(f"FAIL {passed}/{len(checks)}")
        sys.exit(1)
    print(f"OK {passed}/{len(checks)}")


def main() -> None:
    if "--verify" in sys.argv:
        verify()
    else:
        show()


if __name__ == "__main__":
    main()
