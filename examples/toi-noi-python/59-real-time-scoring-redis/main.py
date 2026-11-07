"""Bài 59 - Chấm điểm gian lận thời gian thực: feature store trên Redis, ngân sách 50 ms, fallback, đo p50/p99.

Chạy từ gốc repository:
    uv run --project examples/toi-noi-python/59-real-time-scoring-redis \
        examples/toi-noi-python/59-real-time-scoring-redis/main.py
    ... main.py --verify    # kiểm các bất biến in trong bài, in OK n/n

Cần Redis trong hạ tầng dùng chung (docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait) và
`brew install libomp` trên macOS (LightGBM). Mọi key của lab có tiền tố `lab59:` và bị xóa khi chạy xong.
Dữ liệu là TỔNG HỢP (synthetic_fraud.py, giống bài 54 đến 58); điểm rủi ro là đầu vào cho người duyệt, không tự khóa tài khoản.
Số đo thời gian phụ thuộc máy và tải lúc đo (lab in load average); `--verify` chỉ kiểm bất biến và hành vi fallback bằng
độ trễ tiêm vào, không kiểm tốc độ tuyệt đối.
"""

import asyncio
import math
import os
import platform
import sys
import time
from collections import Counter
from dataclasses import replace
from importlib.metadata import version
from types import SimpleNamespace

import lightgbm
import numpy as np
import polars as pl
import redis.asyncio as redis
from redis.exceptions import RedisError

import train
from dataset import VALID_START_DAY
from faults import BlockingModel, InjectedStore
from features import FEATURES, day_of
from scorer import Breaker, Scorer
from store import ADDED_IN_59, COVERED, FROM_EVENT, FROM_STATE, KEY_META, US_D, US_H, ClosedTrip, FeatureStore, TripAssembler, key_dev, key_feat
from synthetic_fraud import START_US, iter_envelopes

REDIS_URL = os.environ.get("TOINOI_REDIS_URL", "redis://localhost:63790/0")
DAY_US = US_D
SAMPLE_EVERY = 20  # trong ngày valid, cứ 20 chuyến thì đo một chuyến (đo từng chuyến một, giữa các lần ghi)
FAST_SOCKET_S = 0.250  # lưới an toàn cho kết nối treo; ngân sách 50 ms do asyncio.timeout giữ, không phải socket
PACED_RPS, PACED_SECONDS = 70, 10  # 70 chuyến/giây là cao điểm của Tới Nơi (bài du-an-mau-toi-noi)
BURST = 200
POOL = 16  # số kết nối Redis của client đọc
CAP = 8  # số lời gọi chấm điểm cùng lúc tối đa của một tiến trình (xem CPU mỗi chuyến ở bước [4])


def say(*parts: object) -> None:
    print(*parts)


class Checker:
    def __init__(self) -> None:
        self.total = 0
        self.failures: list[str] = []

    def check(self, name: str, ok: bool, detail: object = "") -> None:
        self.total += 1
        if not ok:
            self.failures.append(name)
            print(f"SAI: {name} {detail}")


def load() -> str:
    return " ".join(f"{x:.2f}" for x in os.getloadavg())


def pcts(xs: list[float]) -> tuple[float, float, float, float]:
    a = np.asarray(xs)
    return (*(float(v) for v in np.percentile(a, [50, 90, 99])), float(a.max()))


def row(label: str, xs: list[float]) -> str:
    if not xs:
        return f"{label:<44} {0:>6,}  (không có mẫu)"
    p50, p90, p99, mx = pcts(xs)
    return f"{label:<44} {len(xs):>6,} {p50:>8.2f} {p90:>8.2f} {p99:>8.2f} {mx:>9.2f}"


HEADER = f"{'giai đoạn (ms)':<44} {'mẫu':>6} {'p50':>8} {'p90':>8} {'p99':>8} {'max':>9}"


class FakeClock:
    """Đồng hồ giả cho cầu dao: test không cần ngủ."""

    def __init__(self) -> None:
        self.now = 1000.0

    def __call__(self) -> float:
        return self.now


def make_client(*, socket_timeout: float, pool: int | None = None) -> redis.Redis:
    """Client Redis. `pool=None`: pool mặc định (mở thêm kết nối khi cần, không giới hạn). `pool=n`: tối đa n kết nối,
    lời gọi thứ n+1 chờ tối đa một ngân sách rồi lỗi thay vì mở kết nối mới."""
    if pool is None:
        return redis.Redis.from_url(REDIS_URL, decode_responses=True, socket_timeout=socket_timeout, socket_connect_timeout=1.0)
    p = redis.BlockingConnectionPool.from_url(
        REDIS_URL, max_connections=pool, timeout=0.050, decode_responses=True, socket_timeout=socket_timeout, socket_connect_timeout=1.0)
    return redis.Redis(connection_pool=p)


async def warm(client: redis.Redis, n: int) -> None:
    """Mở sẵn n kết nối: lời gọi đầu tiên của một kết nối mới tốn thêm một lần bắt tay TCP và HELLO. Thử lại vài lần
    vì máy bận có thể làm bắt tay quá hạn."""
    for _ in range(10):
        out = await asyncio.gather(*(client.ping() for _ in range(n)), return_exceptions=True)
        if not any(isinstance(x, BaseException) for x in out):
            return
        await asyncio.sleep(0.1)
    raise RuntimeError("không mở sẵn được các kết nối Redis")


def pool_state(client: redis.Redis) -> str:
    """«n kết nối (k đang nối)» của pool: đọc thuộc tính nội bộ của redis-py 8.1.0 để biết pool nóng hay nguội."""
    pool = client.connection_pool
    conns = list(pool._available_connections) + list(pool._in_use_connections)
    return f"{len(conns)} kết nối, {sum(1 for x in conns if x.is_connected)} đang nối, tối đa {pool.max_connections}"


async def lag_monitor(samples: list[float], stop: asyncio.Event) -> None:
    """Đo độ trễ của chính event loop: ngủ 5 ms rồi xem thực tế thức dậy trễ bao nhiêu."""
    while not stop.is_set():
        a = time.perf_counter()
        await asyncio.sleep(0.005)
        samples.append((time.perf_counter() - a - 0.005) * 1000)


def fresh_trips(trips: list[ClosedTrip], n: int) -> list[ClosedTrip]:
    """n chuyến MỚI (mã mới, chưa ghi vào store, paid_at sau mọi chuyến đã ghi) lấy hình dạng từ các chuyến cuối bộ dữ liệu.
    Chấm chúng là đúng tình huống phục vụ: trạng thái trong store chưa có chuyến nào của chính chúng."""
    base = trips[-1].paid_us
    out = []
    for i in range(n):
        src = trips[-3000 + i % 3000]
        new_paid = base + (i + 1) * 1_000_000
        d = new_paid - src.paid_us
        out.append(replace(src, trip_id=f"TN-F{i:07d}", paid_us=new_paid, booked_us=src.booked_us + d, completed_us=src.completed_us + d))
    return out


async def count_keys(r: redis.Redis) -> int:
    return sum([1 async for _ in r.scan_iter(match="lab59:*", count=5000)])


# ---- [1] Môi trường, dữ liệu, mô hình --------------------------------------------------------


async def step_env(c: Checker):
    say("[1] Môi trường, dữ liệu và mô hình")
    slow = make_client(socket_timeout=5.0)
    try:
        await slow.ping()
    except (RedisError, OSError):
        print(f"FAIL: không nối được Redis tại {REDIS_URL} (docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait)")
        sys.exit(1)
    info = await slow.info("server")
    say(f"Python {platform.python_version()}, redis-py {version('redis')}, "
        f"Redis server {info['redis_version']}, lightgbm {lightgbm.__version__}, numpy {np.__version__}, polars {pl.__version__}")
    say(f"máy: {os.cpu_count()} lõi, load average 1/5/15 phút: {load()}")
    store = FeatureStore(slow)
    left = await store.clear()
    if left:
        say(f"đã xóa {left:,} key `lab59:*` còn sót từ lần chạy trước")
    t0 = time.perf_counter()
    events, table = train.load_table()
    model = train.ensure_model(table)
    say(f"events {events.height:,} dòng, bảng đặc trưng theo lô {table.height:,} chuyến, mô hình {model.version} "
        f"({time.perf_counter() - t0:.1f} giây, gồm dựng bảng và huấn luyện nếu `out/` chưa có); ngưỡng p toàn cục {model.threshold:.3f}")
    c.check("nối được Redis", True)
    c.check("600.000 event và 200.000 chuyến như bài 54", (events.height, table.height) == (600_000, 200_000), (events.height, table.height))
    c.check("mô hình nạp được, 30 đặc trưng", len(model.meta["feature_list"]) == 30 == len(FEATURES))
    say(f"feature store phủ {len(COVERED)}/30 cột: {len(FROM_EVENT)} từ chính các event của chuyến, {len(FROM_STATE)} từ trạng thái Redis "
        f"(đúng bộ online.py của bài 55), {len(ADDED_IN_59)} bổ sung ở bài này ({', '.join(ADDED_IN_59)}); cột mô hình cần mà store thiếu: {store.missing_for(model.feature_list)}")
    c.check("store phủ đủ cột của mô hình", not store.missing_for(model.feature_list) and len(COVERED) == 30)
    return slow, store, events, table, model


# ---- [2] Từ envelope đến ClosedTrip ------------------------------------------------------------


def step_assemble(events: pl.DataFrame, c: Checker) -> list[ClosedTrip]:
    say("\n[2] Từ envelope Kafka đến chuyến đã đủ ba event")
    t0 = time.perf_counter()
    asm, trips = TripAssembler(), []
    for _topic, env in iter_envelopes(events):
        done = asm.add(env)
        if done is not None:
            trips.append(done)
    say(f"600.000 envelope -> {len(trips):,} chuyến, {asm.pending} chuyến còn thiếu event ({time.perf_counter() - t0:.1f} giây)")
    ordered = all(a.paid_us <= b.paid_us for a, b in zip(trips, trips[1:]))
    gaps = [b.paid_us - a.paid_us for a, b in zip(trips, trips[1:])]
    say(f"khoảng cách lớn nhất giữa hai lần thu tiền liên tiếp: {max(gaps) / 60_000_000:.0f} phút; "
        f"{sum(g > 600_000_000 for g in gaps)} khoảng dài hơn 10 phút (ban đêm của bộ tổng hợp)")
    c.check("200.000 chuyến đủ ba event, không chuyến nào treo", (len(trips), asm.pending) == (200_000, 0), (len(trips), asm.pending))
    c.check("chuyến đóng theo thứ tự paid_at", ordered)
    return trips


# ---- [3] Phát lại vào Redis, đối chiếu online với theo lô, đo từng chuyến -------------------------


async def step_replay(slow, store: FeatureStore, trips, table: pl.DataFrame, model, c: Checker):
    say("\n[3] Phát lại vào Redis; từ ngày 40, đo một chuyến trong mỗi 20 chuyến trước khi ghi nó")
    r0 = int((await slow.info("memory"))["used_memory"])
    valid_us = START_US + VALID_START_DAY * DAY_US
    i_valid = next(i for i, t in enumerate(trips) if t.paid_us >= valid_us)
    t0 = time.perf_counter()
    for i in range(0, i_valid, 500):
        await store.record_many(trips[i:min(i + 500, i_valid)])
    took = time.perf_counter() - t0
    say(f"ghi {i_valid:,} chuyến trước ngày 40: {took:.1f} giây ({i_valid / took:,.0f} chuyến/giây, 500 chuyến mỗi pipeline, 1 script Lua mỗi chuyến)")

    ref = {r["trip_id"]: r for r in table.select("trip_id", *FEATURES).iter_rows(named=True)}
    valid = table.filter(day_of("paid_at") >= VALID_START_DAY)
    p_off = dict(zip(valid["trip_id"].to_list(), model.predict_proba(valid)))  # điểm theo lô của cùng mô hình

    fast = FeatureStore(make_client(socket_timeout=FAST_SOCKET_S, pool=POOL))
    await warm(fast.r, POOL)
    ping_client = make_client(socket_timeout=FAST_SOCKET_S, pool=1)  # client riêng, không bao giờ bị hủy giữa chừng
    await warm(ping_client, 1)
    generous = Scorer(store, model, budget_s=5.0)  # chỉ để đối chiếu điểm, không có áp lực thời gian
    budgeted = Scorer(fast, model)  # ngân sách 50 ms thật, pool 16 kết nối
    bad_feat, n_cmp, n_feat_cmp, max_dp, n_score_cmp = Counter(), 0, 0, 0.0, 0
    bad_after, n_after_state = 0, Counter()
    results, parity_degraded, pending = [], 0, []
    pings = {"riêng": [], "sau ok": [], "sau timeout": []}
    lags: list[float] = []
    stop = asyncio.Event()
    monitor = asyncio.create_task(lag_monitor(lags, stop))
    try:
        t0 = time.perf_counter()
        for k, t in enumerate(trips[i_valid:]):
            if k % SAMPLE_EVERY:
                pending.append(t)
                continue
            if pending:
                await store.record_many(pending)
                pending = []
            got = await store.read_features(t)  # trạng thái TRƯỚC chuyến t
            n_cmp += 1
            for f in FEATURES:
                n_feat_cmp += 1
                if not math.isclose(got.features[f], ref[t.trip_id][f], rel_tol=1e-9, abs_tol=1e-9):
                    bad_feat[f] += 1
            g = await generous.score(t)
            if g.degraded:
                parity_degraded += 1
            else:
                n_score_cmp += 1
                max_dp = max(max_dp, abs(g.score - p_off[t.trip_id]))
            a = time.perf_counter()
            await ping_client.ping()
            pings["riêng"].append((time.perf_counter() - a) * 1000)
            res = await budgeted.score(t)
            results.append(res)
            a = time.perf_counter()  # PING trên chính pool vừa chấm điểm: nếu lời gọi vừa bị hủy thì có thể phải nối lại
            await fast.r.ping()
            pings["sau timeout" if res.reason == "timeout" else "sau ok"].append((time.perf_counter() - a) * 1000)
            # Updater ghi chuyến NGAY, rồi chấm lại: kết quả phải y hệt lần đọc trước khi ghi (chuyến không thấy chính nó)
            await store.record_trip(t)
            again = await store.read_features(t)
            n_after_state[again.state] += 1
            if again.features != got.features:
                bad_after += 1
        if pending:
            await store.record_many(pending)
        took = time.perf_counter() - t0
        say(f"ngày 40 đến 60: {len(trips) - i_valid:,} chuyến ghi, {n_cmp:,} chuyến đo ({took:.1f} giây; load average {load()})")
    finally:
        stop.set()
        await monitor
        await fast.r.aclose()
        await ping_client.aclose()
        generous.close()
        budgeted.close()

    r1 = int((await slow.info("memory"))["used_memory"])
    n_keys = await count_keys(slow)
    say(f"đối chiếu đặc trưng online (Redis) với theo lô (polars): {n_cmp:,} chuyến x 30 đặc trưng = {n_feat_cmp:,} giá trị, "
        f"{sum(bad_feat.values())} giá trị lệch (rel_tol 1e-9)")
    say(f"đối chiếu điểm online với điểm theo lô của cùng mô hình: {n_score_cmp:,} chuyến, chênh lớn nhất {max_dp:.2e}")
    say(f"ghi chuyến rồi đọc lại ngay: {n_cmp - bad_after:,}/{n_cmp:,} chuyến cho đặc trưng y hệt lần đọc trước khi ghi; trạng thái {dict(n_after_state)}")
    say(f"Redis sau khi phát lại: {n_keys:,} key, used_memory tăng {(r1 - r0) / 2**20:.1f} MiB (máy dùng chung, số xấp xỉ)")
    c.check("online khớp theo lô trên mọi đặc trưng", not bad_feat, dict(bad_feat))
    c.check("điểm online khớp điểm theo lô", n_score_cmp > 3_000 and max_dp < 1e-9, (n_score_cmp, max_dp))
    c.check("bộ đối chiếu không bị degraded giữa chừng", parity_degraded == 0, parity_degraded)
    c.check("chuyến đã ghi rồi mới đọc: đặc trưng y hệt lúc chưa ghi (không thấy chính nó)", bad_after == 0 and n_after_state == {"latest": n_cmp}, (bad_after, dict(n_after_state)))
    c.check("mọi lần chấm có kết quả", len(results) == n_cmp and all(0.0 <= x.score <= 1.0 for x in results))
    return results, pings, lags, p_off


# ---- [4] Độ trễ ---------------------------------------------------------------------------------


def report_sequential(results, pings, lags, c: Checker) -> None:
    ok = [r for r in results if not r.degraded]
    late = [r for r in results if r.reason == "timeout"]
    say("\n[4] Độ trễ end-to-end (ngân sách 50 ms, mỗi lần một chuyến, không có chuyến nào khác đang chạy)")
    say(HEADER)
    say(row("PING, client riêng (không bao giờ bị hủy)", pings["riêng"]))
    say(row("PING trên pool chấm điểm, sau lời gọi ok", pings["sau ok"]))
    say(row("PING trên pool chấm điểm, sau lời gọi timeout", pings["sau timeout"]))
    say(row("độ trễ event loop (ngủ 5 ms, thức trễ)", lags))
    say(row("đọc Redis + dựng đặc trưng (chỉ chuyến ok)", [r.read_ms for r in ok]))
    say(row("mô hình + chuyển luồng (chỉ chuyến ok)", [r.model_ms for r in ok]))
    say(row("tổng score(), mọi chuyến", [r.elapsed_ms for r in results]))
    say(row("tổng score(), chuyến ok", [r.elapsed_ms for r in ok]))
    say(row("tổng score(), chuyến timeout", [r.elapsed_ms for r in late]))
    by = Counter(r.reason or "ok" for r in results)
    say(f"kết quả: {dict(by)}; load average lúc đo {load()}")
    c.check("mọi kết quả ok hoặc degraded có lý do", all((r.reason is None) != r.degraded for r in results))


async def paced(scorer: Scorer, trips, rps: int, seconds: int):
    """Mở vòng: chuyến thứ i phải đến đúng lịch t0 + i / rps, không chờ chuyến trước xong. Độ trễ tính TỪ LỊCH (không phải
    từ lúc coroutine thực sự bắt đầu), để không giấu việc người gọi bị trễ lịch (coordinated omission)."""
    n = rps * seconds
    t0 = time.perf_counter()
    starts: list[float] = []
    abs_starts: list[float] = []

    async def one(i: int):
        scheduled = t0 + i / rps
        await asyncio.sleep(max(0.0, scheduled - time.perf_counter()))
        abs_starts.append(time.perf_counter())
        starts.append(abs_starts[-1] - scheduled)
        res = await scorer.score(trips[i])
        return res, (time.perf_counter() - scheduled) * 1000

    out = await asyncio.gather(*(one(i) for i in range(n)))
    return out, [x * 1000 for x in starts], (n - 1) / (max(abs_starts) - min(abs_starts))


async def burst_once(label: str, client: redis.Redis, model, trips, **kw) -> Counter:
    fast = FeatureStore(client)
    scorer = Scorer(fast, model, **kw)
    before = pool_state(client)
    t0 = time.perf_counter()
    res = await asyncio.gather(*(scorer.score(t) for t in trips))
    wall = (time.perf_counter() - t0) * 1000
    scorer.close()
    by = Counter(r.reason or "ok" for r in res)
    errors = Counter(r.detail for r in res if r.reason == "store_error")
    p50, _, p99, _ = pcts([r.elapsed_ms for r in res])
    say(f"  {label}")
    say(f"    pool trước: {before}; sau: {pool_state(client)}")
    say(f"    xong sau {wall:.0f} ms; p50 {p50:.1f}, p99 {p99:.1f} ms; {dict(by)}" + (f"; lỗi store: {dict(errors)}" if errors else ""))
    return by


async def step_load(store: FeatureStore, model, trips, c: Checker) -> None:
    say("\nCùng bộ chấm, với các chuyến MỚI (chưa ghi vào store, paid_at sau mọi chuyến đã ghi)")
    fresh = fresh_trips(trips, 700 + 500 + 3 * BURST)
    fast_client = make_client(socket_timeout=FAST_SOCKET_S, pool=POOL)
    await warm(fast_client, POOL)
    fast = FeatureStore(fast_client)
    try:
        n = 500
        cpu0, w0 = time.process_time(), time.perf_counter()
        calm = Scorer(fast, model, budget_s=5.0)
        for t in fresh[700:700 + n]:
            await calm.score(t)
        cpu = (time.process_time() - cpu0) / n * 1000
        wall = (time.perf_counter() - w0) / n * 1000
        calm.close()
        say(f"CPU mỗi chuyến, một tiến trình, {n} chuyến tuần tự, ngân sách nới 5 giây: {cpu:.2f} ms CPU "
            f"({wall:.2f} ms thời gian trôi qua); CPU chiếm {cpu / wall:.0%} thời gian trôi qua, phần còn lại chưa tách "
            f"(chờ I/O, chờ được cấp CPU); nếu chỉ tính CPU thì tối đa khoảng {1000 / cpu:,.0f} chuyến/giây")
        c.check("chấm tuần tự 500 chuyến mới không degraded", calm.stats["degraded"] == 0, dict(calm.stats))

        say(f"{PACED_RPS} chuyến/giây trong {PACED_SECONDS} giây (lịch cố định, không chờ nhau; pool {POOL} kết nối đã mở sẵn); load average trước đo {load()}")
        scorer = Scorer(fast, model)
        out, lag_start, rate = await paced(scorer, fresh[:PACED_RPS * PACED_SECONDS], PACED_RPS, PACED_SECONDS)
        scorer.close()
        say(HEADER)
        say(row("từ lịch đến xong, mọi chuyến", [ms for _, ms in out]))
        say(row("lệch lịch lúc bắt đầu gọi", lag_start))
        say(f"tốc độ phát thực tế {rate:.1f} chuyến/giây (đặt {PACED_RPS}); kết quả: {dict(Counter(r.reason or 'ok' for r, _ in out))}; load average sau đo {load()}")
        c.check("70 chuyến/giây: mọi lời gọi có kết quả", len(out) == PACED_RPS * PACED_SECONDS)

        say(f"{BURST} chuyến đến cùng một lúc, mỗi cấu hình dùng client Redis mới (đổi một thứ so với cấu hình đầu), load average {load()}:")
        burst = fresh[-3 * BURST:]
        results = {}
        for label, kw, pool, warm_n in (
            (f"A. pool {POOL} kết nối, mở sẵn {POOL}", {}, POOL, POOL),
            ("B. như A nhưng chưa mở sẵn kết nối nào", {}, POOL, 0),
            (f"C. như A, thêm tối đa {CAP} lời gọi cùng lúc", {"max_in_flight": CAP}, POOL, POOL),
            ("D. pool mặc định của redis-py, chưa mở sẵn", {}, None, 0),
        ):
            client = make_client(socket_timeout=FAST_SOCKET_S, pool=pool)
            if warm_n:
                await warm(client, warm_n)
            try:
                results[label[0]] = await burst_once(label, client, model, burst[:BURST], **kw)
            finally:
                await client.aclose()
        c.check("đợt đồng thời: mọi lời gọi có kết quả", all(sum(x.values()) == BURST for x in results.values()))
        c.check("giới hạn lời gọi cùng lúc: ít nhất BURST - CAP lời gọi bị từ chối ngay", results["C"]["overloaded"] >= BURST - CAP, dict(results["C"]))
    finally:
        await fast_client.aclose()


# ---- [5] Khi hạ tầng hỏng: độ trễ tiêm vào, không dựa vào tốc độ máy -------------------------------


async def step_faults(slow, store: FeatureStore, model, trips, p_off, c: Checker) -> None:
    say("\n[5] Khi store chậm, kẹt hoặc mất: độ trễ tiêm vào (kết quả không phụ thuộc tốc độ máy)")
    t = trips[-1]
    base = await Scorer(store, model, budget_s=5.0).score(t)
    c.check("không lỗi: không degraded, có điểm", not base.degraded and base.reason is None, base)
    say(f"bình thường:               score={base.score:.6f} degraded={base.degraded} flagged={base.flagged} (chuyến cuối, đã được Updater ghi; điểm theo lô {p_off[t.trip_id]:.6f})")
    c.check("chuyến đã ghi vào store vẫn ra đúng điểm theo lô (không thấy chính nó)", abs(base.score - p_off[t.trip_id]) < 1e-9, (base.score, p_off[t.trip_id]))
    seen_later = {x.customer_id for x in trips[-20_000:]}  # khách còn chuyến ở cuối bộ dữ liệu
    old = next(x for x in trips if x.paid_us >= START_US + 31 * DAY_US and x.customer_id in seen_later)  # sau ngày 31: đủ lịch sử
    r_old = await Scorer(store, model, budget_s=5.0).score(old)
    say(f"chấm lại một chuyến cũ (khách đã có chuyến sau): degraded={r_old.degraded} reason={r_old.reason}")
    c.check("chuyến cũ đã có chuyến sau: state_ahead thay vì điểm sai", (r_old.degraded, r_old.reason) == (True, "state_ahead"), r_old)

    hang = InjectedStore(store, hang=True)
    r = await asyncio.wait_for(Scorer(hang, model, budget_s=0.050).score(t), timeout=5)
    say(f"store kẹt vĩnh viễn:       score={r.score:.6f} degraded={r.degraded} reason={r.reason} sau {r.elapsed_ms:.0f} ms; lời gọi bị hủy: {hang.cancelled}")
    c.check("store kẹt: degraded/timeout, điểm mặc định, không gắn cờ", (r.degraded, r.reason, r.flagged) == (True, "timeout", False) and r.score == model.meta["metrics_valid"]["base_rate"], r)
    c.check("store kẹt: lời gọi store bị hủy đúng một lần", (hang.calls, hang.cancelled) == (1, 1), (hang.calls, hang.cancelled))
    c.check("store kẹt: trả lời trước khi store xong (store không bao giờ xong)", r.elapsed_ms < 2_000, r.elapsed_ms)

    slow_store = InjectedStore(store, delay_s=0.300)
    r1 = await Scorer(slow_store, model, budget_s=0.050).score(t)
    r2 = await Scorer(slow_store, model, budget_s=2.0).score(t)
    say(f"chậm 300 ms, ngân sách 50 ms: degraded={r1.degraded} reason={r1.reason}; ngân sách 2 s: degraded={r2.degraded}, score={r2.score:.6f}")
    c.check("chậm hơn ngân sách thì degraded", r1.degraded and r1.reason == "timeout")
    c.check("chậm nhưng còn trong ngân sách thì vẫn ra điểm thật", not r2.degraded and r2.score == base.score, (r2.degraded, r2.score, base.score))

    mid = InjectedStore(store, delay_s=0.050)
    s_wide = Scorer(mid, model, budget_s=2.0)
    a, b = await s_wide.score(t, deadline_s=0.005), await s_wide.score(t)
    say(f"người gọi chỉ còn 5 ms (deadline_s) so với không đặt: degraded={a.degraded} và {b.degraded}")
    c.check("deadline của người gọi nhỏ hơn ngân sách thì thắng", a.degraded and a.reason == "timeout" and not b.degraded, (a.reason, b.degraded))

    down = FeatureStore(redis.Redis.from_url("redis://localhost:1/0", decode_responses=True, socket_timeout=0.05, socket_connect_timeout=0.05))
    try:
        r = await Scorer(down, model, budget_s=0.050).score(t)
    finally:
        await down.r.aclose()
    say(f"Redis không nghe ở cổng:   degraded={r.degraded} reason={r.reason} sau {r.elapsed_ms:.0f} ms")
    c.check("Redis không nối được: degraded/store_error", (r.degraded, r.reason) == (True, "store_error"), r)

    # ---- cầu dao với đồng hồ giả
    clock = FakeClock()
    stuck = InjectedStore(store, hang=True)
    sc = Scorer(stuck, model, budget_s=0.020, breaker=Breaker(failures=3, cooldown_s=2.0, clock=clock))
    reasons = [(await sc.score(t)).reason for _ in range(3)]
    calls_before = stuck.calls
    open_r = await sc.score(t)
    calls_open = stuck.calls
    clock.now += 2.1
    probe = await sc.score(t)  # hết thời gian chờ: đúng một lời gọi thăm dò, vẫn kẹt nên mở lại
    right_after = await sc.score(t)
    sc.store, sc.budget_s = store, 2.0  # store hồi phục
    clock.now += 2.1
    healed = await sc.score(t)
    again = await sc.score(t)
    say(f"cầu dao (3 lần hỏng thì mở, chờ 2 giây, đồng hồ giả): {reasons} -> {open_r.reason} (store không bị gọi: {calls_open == calls_before})")
    say(f"  sau 2,1 giây: thăm dò -> {probe.reason}; ngay sau đó -> {right_after.reason}; store hồi phục, sau 2,1 giây nữa -> {healed.reason or 'ok'}, rồi {again.reason or 'ok'}")
    c.check("3 lần timeout liên tiếp mở cầu dao", reasons == ["timeout"] * 3 and open_r.reason == "breaker_open", (reasons, open_r.reason))
    c.check("cầu dao mở: không gọi store", calls_open == calls_before, (calls_open, calls_before))
    c.check("hết thời gian chờ: một lời gọi thăm dò", probe.reason == "timeout" and stuck.calls == calls_before + 1 and right_after.reason == "breaker_open", (probe.reason, right_after.reason))
    c.check("store hồi phục: thăm dò thành công, cầu dao đóng", healed.reason is None and again.reason is None, (healed.reason, again.reason))

    # ---- timeout không ngắt được code CPU chạy thẳng trong event loop (ngân sách và độ chặn đủ lớn để máy bận không làm lệch)
    inline_model, off_model = BlockingModel(model, block_s=1.0), BlockingModel(model, block_s=1.0)
    ri = await Scorer(store, inline_model, budget_s=0.250, offload_model=False).score(t)
    ro = await Scorer(store, off_model, budget_s=0.250, offload_model=True).score(t)
    for _ in range(300):  # luồng không thể bị hủy: chờ nó chạy hết rồi mới sang bước sau
        if off_model.finished:
            break
        await asyncio.sleep(0.02)
    say(f"mô hình chặn 1.000 ms, ngân sách 250 ms: chạy thẳng trong loop -> degraded={ri.degraded} sau {ri.elapsed_ms:.0f} ms; "
        f"to_thread -> degraded={ro.degraded} sau {ro.elapsed_ms:.0f} ms (luồng vẫn chạy hết: {off_model.finished == 1})")
    c.check("mô hình chạy thẳng trong loop: timeout không có tác dụng", not ri.degraded and ri.elapsed_ms >= 950, (ri.degraded, ri.elapsed_ms))
    c.check("mô hình trong to_thread: timeout đúng hạn", ro.degraded and ro.reason == "timeout" and ro.elapsed_ms < 950, (ro.degraded, ro.reason, ro.elapsed_ms))
    c.check("luồng bị bỏ lại vẫn chạy hết (không hủy được luồng)", off_model.finished == 1, off_model.finished)

    # ---- nguồn đặc trưng trả NaN: mô hình từ chối, người gọi vẫn nhận câu trả lời
    r = await Scorer(InjectedStore(store, nan_feature="speed_kmh"), model, budget_s=2.0).score(t)
    say(f"đặc trưng speed_kmh là NaN: degraded={r.degraded} reason={r.reason}")
    c.check("đầu vào NaN: degraded/model_error, không ném lỗi", (r.degraded, r.reason) == (True, "model_error") and "NaN" in (r.detail or ""), r)
    r = await Scorer(InjectedStore(store, drop_feature="cust_promo_share_30d"), model, budget_s=2.0).score(t)
    say(f"nguồn thiếu cột cust_promo_share_30d: degraded={r.degraded} reason={r.reason} ({r.detail})")
    c.check("thiếu cột lúc chấm: model_error nêu tên cột, không điền 0", r.reason == "model_error" and "cust_promo_share_30d" in (r.detail or ""), r)
    wider = SimpleNamespace(feature_list=[*model.feature_list, "dev_fraud_known"], meta=model.meta, version="x")
    try:
        Scorer(store, wider)
        refused = ""
    except ValueError as e:
        refused = str(e)
    say(f"mô hình đòi thêm cột dev_fraud_known: Scorer từ chối khởi tạo: {refused}")
    c.check("mô hình đòi cột store không phủ: Scorer từ chối và nêu tên cột", "dev_fraud_known" in refused, refused)

    # ---- giới hạn lời gọi cùng lúc: từ chối ngay thay vì để mọi người cùng quá hạn
    held = InjectedStore(store, hang=True)
    capped = Scorer(held, model, budget_s=0.050, max_in_flight=8)
    res = await asyncio.wait_for(asyncio.gather(*(capped.score(x) for x in trips[-20:])), timeout=10)
    by = Counter(x.reason for x in res)
    say(f"20 lời gọi cùng lúc vào store kẹt, tối đa 8 cùng lúc: {dict(by)}; store chỉ bị gọi {held.calls} lần")
    c.check("tối đa 8 lời gọi cùng lúc: 8 vào store, 12 bị từ chối ngay", (by["timeout"], by["overloaded"], held.calls) == (8, 12, 8), (dict(by), held.calls))

    # ---- cầu dao chỉ đếm lỗi của store: store khỏe, mô hình chậm thì không mở
    slow_model = BlockingModel(model, block_s=1.2)
    sc = Scorer(store, slow_model, budget_s=0.400, breaker=Breaker(failures=3, cooldown_s=2.0))
    reasons = [(await sc.score(t)).reason for _ in range(5)]
    for _ in range(200):
        if slow_model.finished >= 5:
            break
        await asyncio.sleep(0.02)
    say(f"store khỏe, mô hình chậm 1,2 s, ngân sách 0,4 s, 5 lời gọi: {reasons}")
    c.check("mô hình chậm không làm cầu dao mở", "breaker_open" not in reasons and reasons == ["timeout"] * 5, reasons)

    # ---- giới hạn lời gọi tính cả luồng mô hình còn chạy sau khi lời gọi hết hạn
    stuck_model = BlockingModel(model, block_s=1.2)
    sc = Scorer(store, stuck_model, budget_s=0.400, max_in_flight=1)
    reasons = [(await sc.score(t)).reason for _ in range(5)]
    stray_now = sc._stray
    for _ in range(200):
        if stuck_model.finished >= 1:
            break
        await asyncio.sleep(0.02)
    await asyncio.sleep(0.05)
    say(f"max_in_flight=1, mô hình chặn 1,2 s, ngân sách 0,4 s, 5 lời gọi nối tiếp: {reasons}; luồng bị bỏ lại {stray_now}, mô hình được bắt đầu {stuck_model.started} lần; sau khi luồng xong: {sc._stray}")
    c.check("luồng còn chạy vẫn tính vào giới hạn: lời gọi sau bị từ chối, không thêm luồng", reasons == ["timeout"] + ["overloaded"] * 4 and stuck_model.started == 1 and sc._stray == 0, (reasons, stuck_model.started, sc._stray))

    # ---- một dependency kẹt không giữ người gọi quá ngân sách
    hang_all = InjectedStore(store, hang=True)
    sc = Scorer(hang_all, model, budget_s=0.050)
    t0 = time.perf_counter()
    res = await asyncio.wait_for(asyncio.gather(*(sc.score(x) for x in trips[-BURST:])), timeout=10)
    wall = (time.perf_counter() - t0) * 1000
    say(f"{BURST} lời gọi cùng lúc vào store kẹt: tất cả degraded={all(x.degraded for x in res)}, {hang_all.cancelled} lời gọi store bị hủy, xong sau {wall:.0f} ms")
    c.check("200 lời gọi vào store kẹt: tất cả degraded/timeout và bị hủy", all(x.reason == "timeout" for x in res) and hang_all.cancelled == BURST, (Counter(x.reason for x in res), hang_all.cancelled))
    c.check("200 lời gọi vào store kẹt: không ai chờ vô hạn", wall < 5_000, wall)


# ---- [6] Đường biên của store: cửa sổ, ghi trùng, đến muộn, store trống --------------------------------


def mk(trip_id: str, paid_us: int, **kw) -> ClosedTrip:
    base = dict(
        trip_id=trip_id, customer_id="CU-T0001", device_id="DEV-T0001", promo_code=None, pickup_lat=21.0, pickup_lng=105.8,
        booked_us=paid_us - 30 * 60 * 1_000_000, driver_id="DR-T0001", driver_device_id="DEV-T0002", start_lat=21.0, start_lng=105.8,
        end_lat=21.05, end_lng=105.85, distance_km=7.0, duration_s=900, completed_us=paid_us - 60 * 1_000_000,
        amount_vnd=60_000, discount_vnd=0, paid_us=paid_us,
    )
    return ClosedTrip(**{**base, **kw})


async def step_edges(slow, store: FeatureStore, model, c: Checker) -> None:
    say("\n[6] Đường biên của store (chuyến giả trên khách CU-T0001, thiết bị DEV-T0001, tài xế DR-T0001)")
    await store.clear()
    T0 = START_US + 30 * DAY_US
    a = mk("TN-T000001", T0)
    c.check("ghi chuyến mới trả True", await store.record_trip(a) is True)
    again = await store.record_trip(a)
    say(f"ghi lại cùng chuyến: {again} (no-op); hash khách: {await slow.hgetall(key_feat('CU-T0001'))}")
    lua_num = await slow.eval("return tostring(tonumber(ARGV[1]) - 5)", 0, "1773000000123456")
    lua_cat = await slow.eval("return '(' .. ARGV[1]", 0, "1773000000123456")
    say(f"Lua của Redis: tostring(tonumber('1773000000123456') - 5) = {lua_num}; chuỗi nối '(' .. ARGV[1] = {lua_cat}")
    c.check("Lua làm tròn số 16 chữ số khi đổi sang chuỗi, chuỗi từ ARGV thì không", lua_num != "1773000000123451" and lua_cat == "(1773000000123456", (lua_num, lua_cat))
    c.check("ghi trùng là no-op", again is False and (await slow.hget(key_feat("CU-T0001"), "prior_trips")) == "1")

    edge = await store.read_features(mk("TN-T000002", T0 + US_H))
    past = await store.read_features(mk("TN-T000003", T0 + US_H + 1))
    same = await store.read_features(mk("TN-T000004", T0))
    say(f"cận cửa sổ 1 giờ: chuyến trước đúng 1 giờ -> dev_trips_1h={edge.features['dev_trips_1h']:.0f}; "
        f"trễ thêm 1 micro giây -> {past.features['dev_trips_1h']:.0f}; cùng timestamp -> {same.features['dev_trips_1h']:.0f}")
    c.check("cận dưới của cửa sổ là bao gồm", edge.features["dev_trips_1h"] == 1.0 and edge.features["dev_trips_24h"] == 1.0)
    c.check("quá cửa sổ 1 micro giây thì không đếm, vẫn trong 24 giờ", past.features["dev_trips_1h"] == 0.0 and past.features["dev_trips_24h"] == 1.0)
    c.check("cùng timestamp không thấy nhau (cận trên loại trừ)", same.features["dev_trips_1h"] == 0.0 and same.features["cust_prior_trips"] == 1.0)

    b = mk("TN-T000005", T0 + 600 * 1_000_000, end_lat=21.10, end_lng=105.90)
    await store.record_trip(b)
    late = mk("TN-T000006", T0 + 300 * 1_000_000, end_lat=10.0, end_lng=106.0)  # đến muộn, paid_at cũ hơn b
    await store.record_trip(late)
    last_paid = await slow.hget(key_feat("DR-T0001"), "paid_us")
    c.check("chuyến đến muộn không ghi đè vị trí cuối của tài xế", last_paid == str(b.paid_us), last_paid)
    nxt = await store.read_features(mk("TN-T000007", T0 + 3600 * 1_000_000, start_lat=21.10, start_lng=105.90, duration_s=600,
                                      completed_us=T0 + 3600 * 1_000_000 - 1_000_000))
    say(f"chuyến đến muộn (paid_at cũ hơn): vị trí cuối vẫn của chuyến mới nhất; nhảy GPS của chuyến kế = {nxt.features['drv_jump_km']:.2f} km")
    c.check("nhảy GPS tính từ chuyến mới nhất, không phải chuyến đến muộn", nxt.features["drv_jump_km"] < 1.0, nxt.features["drv_jump_km"])

    far = mk("TN-T000008", T0 + 8 * DAY_US)
    await store.record_trip(far)
    card = await slow.zcard(key_dev("DEV-T0001"))
    ttl_ms = await slow.pttl(key_dev("DEV-T0001"))
    say(f"sau khi ghi một chuyến cách 8 ngày: zset thiết bị còn {card} mục (cửa sổ dài nhất 7 ngày); PTTL = {ttl_ms / 3_600_000:.1f} giờ")
    c.check("mục cũ hơn cửa sổ dài nhất bị cắt", card == 1, card)
    c.check("khóa có TTL (không hoạt động thì tự biến mất)", 0 < ttl_ms <= 7 * US_D // 1000 + 3_600_000, ttl_ms)

    # ---- store mới dựng lại: chưa đủ 30 ngày lịch sử thì chưa chấm
    await store.clear()
    T1 = START_US + 50 * DAY_US
    c1 = mk("TN-T000010", T1)
    await store.record_trip(c1)
    builder = Scorer(store, model, budget_s=2.0)
    early = await builder.score(mk("TN-T000011", T1 + US_H))
    later = await builder.score(mk("TN-T000012", T1 + 31 * DAY_US))
    say(f"store chỉ có 1 giờ lịch sử: degraded={early.degraded} reason={early.reason}; sau 31 ngày lịch sử: degraded={later.degraded} reason={later.reason}")
    c.check("chưa đủ 30 ngày lịch sử: degraded/store_building", (early.degraded, early.reason) == (True, "store_building"), early)
    c.check("đủ 30 ngày lịch sử: chấm được", not later.degraded, later)

    # ---- Updater ghi trước, chấm sau: không thấy chính nó; có chuyến sau thì báo ahead
    await store.clear()
    await store.record_trip(mk("TN-T000013", T1, end_lat=21.20, end_lng=105.95))
    p = mk("TN-T000020", T1 + 1_200 * 1_000_000, start_lat=21.0, start_lng=105.8)
    before_p = await store.read_features(p)
    await store.record_trip(p)
    after_p = await store.read_features(p)
    await store.record_trip(mk("TN-T000021", T1 + 1_800 * 1_000_000))
    ahead_p = await store.read_features(p)
    say(f"ghi trước, đọc sau: cust_prior_trips {before_p.features['cust_prior_trips']:.0f} -> {after_p.features['cust_prior_trips']:.0f}, "
        f"drv_jump_km {before_p.features['drv_jump_km']:.2f} -> {after_p.features['drv_jump_km']:.2f}; trạng thái {before_p.state} -> {after_p.state}; "
        f"sau khi có chuyến sau nó: {ahead_p.state}")
    c.check("ghi trước đọc sau: đặc trưng y hệt lúc chưa ghi", before_p.features == after_p.features and (before_p.state, after_p.state) == ("before", "latest") and before_p.features["drv_jump_km"] > 1.0, (before_p.state, after_p.state))
    c.check("đã có chuyến sau nó: state ahead", ahead_p.state == "ahead", ahead_p.state)

    # ---- store trống: không phải «khách mới»
    await store.clear()
    scorer = Scorer(store, model, budget_s=2.0)
    cold = await scorer.score(a)
    say(f"store trống (vừa xóa hết): degraded={cold.degraded} reason={cold.reason}; meta còn không: {await slow.exists(KEY_META)}")
    c.check("store trống: degraded/store_cold chứ không chấm điểm trên đặc trưng toàn 0", (cold.degraded, cold.reason) == (True, "store_cold"), cold)


# ---- main ----------------------------------------------------------------------------------------


async def run(verify: bool) -> int:
    c = Checker()
    slow, store, events, table, model = await step_env(c)
    try:
        trips = step_assemble(events, c)
        results, pings, lags, p_off = await step_replay(slow, store, trips, table, model, c)
        report_sequential(results, pings, lags, c)
        await step_load(store, model, trips, c)
        await step_faults(slow, store, model, trips, p_off, c)
        await step_edges(slow, store, model, c)
    finally:
        n = await store.clear()
        left = await count_keys(slow)
        say(f"\nDọn: đã xóa {n:,} key `lab59:*`, còn lại {left}")
        c.check("dọn sạch key lab59:*", left == 0, left)
        await slow.aclose()
    ok = c.total - len(c.failures)
    if verify:
        if c.failures:
            print(f"\nKiểm chứng: FAIL {ok}/{c.total}")
            return 1
        print(f"\nKiểm chứng: OK {ok}/{c.total}")
    return 0


if __name__ == "__main__":
    sys.exit(asyncio.run(run("--verify" in sys.argv)))
