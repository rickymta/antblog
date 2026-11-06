"""Bài 27: Redis cho Tới Nơi - cache báo giá, rate limit và GEO vị trí tài xế (redis-py async).

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Cần Redis trong hạ tầng dùng chung:
    docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Mọi key của lab có tiền tố `lab27:` và bị xóa khi chạy xong. Redis ở đây là bộ nhớ đệm và
chỉ mục tra cứu; nguồn sự thật (bảng giá, vị trí gốc của tài xế) luôn nằm ở chỗ khác.
Dữ liệu tổng hợp, seed cố định. Thời gian đo phụ thuộc máy; kết quả đếm thì lặp lại được.
"""

import asyncio
import json
import math
import os
import platform
import random
import sys
import time
from collections.abc import Awaitable, Callable
from decimal import ROUND_HALF_UP, Decimal
from enum import StrEnum
from typing import NamedTuple
from uuid import uuid4

import redis.asyncio as redis
from redis.exceptions import RedisError

REDIS_URL = os.environ.get("TOINOI_REDIS_URL", "redis://localhost:63790/0")
PREFIX = "lab27:"
SEED = 27

# ---------------------------------------------------------------- cước (sao chép từ bài 10)


class FarePolicy(NamedTuple):
    base_fare: Decimal = Decimal("12000")
    included_km: Decimal = Decimal("2")
    price_per_extra_km: Decimal = Decimal("4500")


STANDARD = FarePolicy()


class ServiceKind(StrEnum):
    RIDE = "ride"
    FOOD = "food"
    PARCEL = "parcel"


def calculate_fare(
    distance_km: Decimal, policy: FarePolicy = STANDARD, *, service: ServiceKind = ServiceKind.RIDE
) -> Decimal:
    if not Decimal("0") <= distance_km <= Decimal("100"):
        raise ValueError(f"khoảng cách ngoài miền 0..100 km: {distance_km}")
    extra_km = max(Decimal("0"), distance_km - policy.included_km)
    total = policy.base_fare + extra_km * policy.price_per_extra_km
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)


# ---------------------------------------------------------------- tiện ích của lab

QUIET = False


def say(*parts: object) -> None:
    if not QUIET:
        print(*parts)


class Checker:
    def __init__(self) -> None:
        self.total = 0
        self.failures: list[str] = []

    def check(self, name: str, got: object, expected: object) -> None:
        self.total += 1
        if got != expected:
            self.failures.append(name)
            print(f"SAI: {name}: nhận {got!r}, kỳ vọng {expected!r}")


async def timed[T](awaitable: Awaitable[T]) -> tuple[T, float]:
    """Chạy một awaitable và trả (kết quả, mili giây)."""
    start = time.perf_counter()
    result = await awaitable
    return result, (time.perf_counter() - start) * 1000


async def delete_prefix(r: redis.Redis) -> int:
    """Xóa mọi key `lab27:*` bằng SCAN (không dùng KEYS vì nó chặn server khi nhiều key)."""
    deleted = 0
    batch: list[str] = []
    async for key in r.scan_iter(match=f"{PREFIX}*", count=1000):
        batch.append(key)
        if len(batch) >= 1000:
            deleted += await r.unlink(*batch)
            batch.clear()
    if batch:
        deleted += await r.unlink(*batch)
    return deleted


# ---------------------------------------------------------------- 1. cache-aside cho báo giá


class QuoteSource:
    """Giả lập dịch vụ báo giá chậm (nguồn sự thật) và đếm số lần bị gọi."""

    def __init__(self, delay_s: float = 0.05) -> None:
        self.delay_s = delay_s
        self.calls = 0

    async def fetch(self, service: ServiceKind, distance_km: Decimal) -> dict[str, str]:
        self.calls += 1
        await asyncio.sleep(self.delay_s)
        fare = calculate_fare(distance_km, service=service)
        # Decimal không có trong JSON: lưu chuỗi, đọc lại bằng Decimal(...) khi cần tính.
        return {"service": service.value, "distance_km": str(distance_km), "fare": str(fare),
                "currency": "VND", "policy": "v1"}


class QuoteUnavailable(Exception):
    """Chờ người khác dựng cache quá lâu: caller nên trả 503 hoặc giá cũ, không tự đổ về nguồn."""


def normalize_km(distance_km: Decimal) -> Decimal:
    """Làm tròn km về mét (3 chữ số thập phân) TRƯỚC khi vào key và TRƯỚC khi tính giá.

    Hai khoảng cách cùng key phải ra cùng giá; nếu key dùng km đã làm tròn mà nguồn tính theo km gốc,
    2.5004 và 2.5001 dùng chung key 2.500 nhưng giá lệch nhau và khách nhận giá của người khác.
    """
    return distance_km.quantize(Decimal("0.001"), rounding=ROUND_HALF_UP)


def quote_key(service: ServiceKind, distance_km: Decimal) -> str:
    # km đã chuẩn hóa: 2.5 và 2.50 ra cùng một key. Phiên bản chính sách (v1) nằm trong key:
    # đổi bảng giá thì đổi v2, key cũ tự chết theo TTL.
    return f"{PREFIX}quote:v1:{service.value}:{normalize_km(distance_km)}"


async def get_quote(
    r: redis.Redis, source: QuoteSource, service: ServiceKind, distance_km: Decimal,
    *, ttl_ms: int = 30_000,
) -> tuple[dict[str, str], str]:
    """Cache-aside: đọc cache; hụt thì hỏi nguồn rồi ghi cache kèm TTL."""
    km = normalize_km(distance_km)  # chuẩn hóa một lần, dùng cho cả key lẫn nguồn
    key = quote_key(service, km)
    cached = await r.get(key)
    if cached is not None:
        return json.loads(cached), "hit"
    quote = await source.fetch(service, km)
    await r.set(key, json.dumps(quote), px=ttl_ms)
    return quote, "miss"


# Chỉ xóa khóa nếu giá trị vẫn là token của ta (tránh xóa nhầm khóa của người khác).
UNLOCK_LUA = """
if redis.call('GET', KEYS[1]) == ARGV[1] then
    return redis.call('DEL', KEYS[1])
end
return 0
"""


async def get_quote_single_flight(
    r: redis.Redis, source: QuoteSource, service: ServiceKind, distance_km: Decimal,
    *, ttl_ms: int = 30_000, lock_ms: int = 2_000, wait_ms: int = 3_000,
) -> tuple[dict[str, str], str]:
    """Như get_quote nhưng chỉ MỘT caller hỏi nguồn khi cache nguội; số còn lại chờ rồi đọc cache.

    lock_ms (tuổi thọ khóa) phải lớn hơn thời gian nguồn trả lời lúc chậm; wait_ms (thời gian caller
    chịu chờ) là con số khác. Hết wait_ms thì ném QuoteUnavailable thay vì tự gọi nguồn: để mọi
    caller đã chờ cùng đổ về nguồn chính là cache stampede.
    """
    km = normalize_km(distance_km)
    key = quote_key(service, km)
    lock_key = f"{key}:lock"
    give_up_at = time.monotonic() + wait_ms / 1000
    while True:
        cached = await r.get(key)
        if cached is not None:
            return json.loads(cached), "hit"
        token = uuid4().hex
        if await r.set(lock_key, token, nx=True, px=lock_ms):  # SET NX PX = giành khóa có hạn
            try:
                # Kiểm lại cache sau khi giành khóa: người giữ khóa trước có thể vừa ghi xong
                # (và nhả khóa) giữa lúc ta đọc hụt và lúc ta giành được khóa.
                cached = await r.get(key)
                if cached is not None:
                    return json.loads(cached), "hit"
                quote = await source.fetch(service, km)
                await r.set(key, json.dumps(quote), px=ttl_ms)
                return quote, "miss"
            finally:
                await r.eval(UNLOCK_LUA, 1, lock_key, token)
        if time.monotonic() > give_up_at:
            raise QuoteUnavailable(f"chờ quá {wait_ms} ms: {key}")
        await asyncio.sleep(0.01)


async def part_cache(r: redis.Redis, c: Checker) -> None:
    say("== 1. Cache-aside cho báo giá ==")
    source = QuoteSource()
    km = Decimal("2.5")
    ride = ServiceKind.RIDE

    (quote, state1), ms1 = await timed(get_quote(r, source, ride, km))
    (_, state2), ms2 = await timed(get_quote(r, source, ride, km))
    key = quote_key(ride, km)
    say(f"lần 1: {state1:<4} {ms1:6.1f} ms  -> {quote}")
    say(f"lần 2: {state2:<4} {ms2:6.1f} ms  (nguồn bị gọi {source.calls} lần)")
    say(f"key = {key}")
    say(f"PTTL = {await r.pttl(key)} ms (còn lại trước khi key tự xóa)")
    c.check("lần 1 hụt, lần 2 trúng", (state1, state2), ("miss", "hit"))
    c.check("nguồn chỉ bị gọi một lần", source.calls, 1)
    c.check("giá 2,5 km", Decimal(quote["fare"]), Decimal("14250"))
    c.check("hit nhanh hơn miss", ms2 < ms1, True)
    c.check("key chuẩn hóa km", quote_key(ride, Decimal("2.50")), quote_key(ride, Decimal("2.5")))
    c.check("TTL còn trong 30 giây", 0 < await r.pttl(key) <= 30_000, True)

    # TTL ngắn: hết hạn thì lần đọc sau là hụt và nguồn bị gọi lại.
    calls_before = source.calls
    food = ServiceKind.FOOD
    await get_quote(r, source, food, Decimal("5"), ttl_ms=150)
    _, state_fresh = await get_quote(r, source, food, Decimal("5"), ttl_ms=150)
    await asyncio.sleep(0.2)
    _, state_expired = await get_quote(r, source, food, Decimal("5"), ttl_ms=150)
    say(f"TTL 150 ms: ngay sau ghi -> {state_fresh}; sau 200 ms -> {state_expired}")
    c.check("TTL hết hạn", (state_fresh, state_expired), ("hit", "miss"))
    c.check("nguồn gọi thêm 2 lần", source.calls - calls_before, 2)

    # Cùng key thì phải cùng giá: nguồn tính trên km đã chuẩn hóa, không phải km gốc của từng caller.
    parcel = ServiceKind.PARCEL
    q_a, _ = await get_quote(r, source, parcel, Decimal("4.0004"))
    q_b, state_b = await get_quote(r, source, parcel, Decimal("4.0001"))
    q_c, state_c = await get_quote(r, source, parcel, Decimal("4.0006"))
    say(f"4,0004 km -> {q_a['fare']} ({q_a['distance_km']}); 4,0001 km -> {q_b['fare']} ({state_b}); "
        f"4,0006 km -> {q_c['fare']} ({q_c['distance_km']}, {state_c})")
    c.check("hai km gần nhau cùng key, cùng giá", (q_a["fare"], q_b["fare"], state_b),
            ("21000", "21000", "hit"))
    c.check("km làm tròn sang mét kế tiếp: key và giá mới", (q_c["distance_km"], q_c["fare"], state_c),
            ("4.001", "21005", "miss"))

    # 20 yêu cầu đồng thời cho cùng một key nguội: cache-aside ngây thơ hỏi nguồn 20 lần.
    await r.delete(key)
    naive_source = QuoteSource()
    await asyncio.gather(*(get_quote(r, naive_source, ride, km) for _ in range(20)))
    say(f"20 yêu cầu đồng thời, key nguội, cache-aside thường: nguồn bị gọi {naive_source.calls} lần")

    await r.delete(key)
    guarded_source = QuoteSource()
    results = await asyncio.gather(
        *(get_quote_single_flight(r, guarded_source, ride, km) for _ in range(20))
    )
    states = sorted({state for _, state in results})
    say(f"20 yêu cầu đồng thời, có khóa single-flight:          nguồn bị gọi {guarded_source.calls} lần")
    c.check("không khóa: 20 lần gọi nguồn", naive_source.calls, 20)
    c.check("có khóa: 1 lần gọi nguồn", guarded_source.calls, 1)
    c.check("mọi caller đều nhận đúng giá", {q["fare"] for q, _ in results}, {"14250"})
    c.check("trạng thái chỉ có hit/miss", states, ["hit", "miss"])
    c.check("khóa đã được trả", await r.exists(f"{key}:lock"), 0)

    # Nguồn chậm hơn tuổi thọ khóa (0,3 s so với 0,1 s) và chậm hơn thời gian chịu chờ (0,15 s):
    # số lần gọi nguồn bị chặn bởi khóa, các caller chờ quá lâu nhận QuoteUnavailable thay vì tự gọi nguồn.
    slow_source = QuoteSource(delay_s=0.3)
    outcomes = await asyncio.gather(
        *(get_quote_single_flight(r, slow_source, ride, Decimal("7"), lock_ms=100, wait_ms=150)
          for _ in range(20)),
        return_exceptions=True,
    )
    unavailable = sum(isinstance(o, QuoteUnavailable) for o in outcomes)
    served = sum(isinstance(o, tuple) for o in outcomes)
    say(f"nguồn chậm 0,3 s, khóa 0,1 s, chờ tối đa 0,15 s: nguồn bị gọi {slow_source.calls} lần, "
        f"{served} caller có giá, {unavailable} caller nhận QuoteUnavailable")
    c.check("nguồn chậm: số lần gọi nguồn bị chặn", slow_source.calls <= 3, True)
    c.check("nguồn chậm: phần lớn caller nhận QuoteUnavailable", unavailable >= 15, True)
    c.check("nguồn chậm: không caller nào gặp lỗi khác", unavailable + served, 20)


# ---------------------------------------------------------------- 2. rate limit


async def fixed_window_allow(
    r: redis.Redis, client_id: str, *, limit: int, window_s: int, now: float
) -> bool:
    """INCR + EXPIRE NX trong một MULTI/EXEC. `now` truyền vào để ca thử lặp lại được."""
    window_id = int(now // window_s)
    key = f"{PREFIX}rl:fw:{client_id}:{window_id}"
    async with r.pipeline(transaction=True) as pipe:
        pipe.incr(key)
        pipe.expire(key, window_s, nx=True)  # chỉ đặt TTL lần đầu của cửa sổ
        count, _ = await pipe.execute()
    return count <= limit


# Token bucket: đọc trạng thái, nạp lại token theo thời gian trôi, trừ phí, ghi lại: tất cả
# trong một script nên atomic. Đơn vị là mili-token (1 token = 1000): với tốc độ nạp nguyên, phép tính
# là số nguyên nên state lưu gọn ('0', '1000'). Số trong Lua vẫn là double; tốc độ nạp lẻ có sai số nhỏ.
TOKEN_BUCKET_LUA = """
local capacity = tonumber(ARGV[1]) * 1000
local refill_per_s = tonumber(ARGV[2])
local now_ms = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) * 1000
local state = redis.call('HMGET', KEYS[1], 'milli', 'ts')
local milli = tonumber(state[1])
local ts = tonumber(state[2])
if milli == nil then
    milli = capacity
    ts = now_ms
end
local elapsed = math.max(0, now_ms - ts)
ts = math.max(ts, now_ms)  -- đồng hồ client lệch lùi: không ghi lùi ts, kẻo lần sau nạp thừa token
milli = math.min(capacity, milli + elapsed * refill_per_s)
local allowed = 0
if milli >= cost then
    milli = milli - cost
    allowed = 1
end
redis.call('HSET', KEYS[1], 'milli', milli, 'ts', ts)
redis.call('PEXPIRE', KEYS[1], math.ceil(capacity / refill_per_s) + 1000)
return {allowed, tostring(milli)}
"""

TokenBucket = Callable[..., Awaitable[tuple[bool, float]]]


def make_token_bucket(r: redis.Redis) -> TokenBucket:
    script = r.register_script(TOKEN_BUCKET_LUA)  # nạp một lần, sau đó gọi bằng EVALSHA

    async def allow(client_id: str, *, capacity: int, refill_per_s: float, now_ms: int, cost: int = 1
                    ) -> tuple[bool, float]:
        allowed, milli = await script(
            keys=[f"{PREFIX}rl:tb:{client_id}"], args=[capacity, refill_per_s, now_ms, cost]
        )
        return bool(allowed), float(milli) / 1000

    return allow


async def naive_allow(r: redis.Redis, key: str) -> bool:
    """SAI: đọc rồi ghi là hai lệnh; giữa chúng coroutine khác chen vào."""
    tokens = int(await r.get(key) or 0)
    if tokens <= 0:
        return False
    await asyncio.sleep(0)  # chỗ xử lý nghiệp vụ nào đó: nhường quyền cho coroutine khác
    await r.set(key, tokens - 1)
    return True


async def part_rate_limit(r: redis.Redis, c: Checker) -> None:
    say("\n== 2. Rate limit ==")
    # Cửa sổ cố định: tối đa 5 yêu cầu mỗi 60 giây cho một khách.
    at_end = [await fixed_window_allow(r, "CU-001", limit=5, window_s=60, now=59.9) for _ in range(6)]
    at_start = [await fixed_window_allow(r, "CU-001", limit=5, window_s=60, now=60.1) for _ in range(6)]
    say("cửa sổ cố định, 5 yêu cầu/60 s, t=59,9 s: ", at_end)
    say("cửa sổ cố định, 5 yêu cầu/60 s, t=60,1 s: ", at_start)
    burst = sum(at_end) + sum(at_start)
    say(f"-> {burst} yêu cầu được nhận trong 0,2 giây quanh ranh giới cửa sổ (giới hạn danh nghĩa 5/60 s)")
    c.check("cửa sổ 1: 5 qua, 1 chặn", at_end, [True] * 5 + [False])
    c.check("cửa sổ 2: 5 qua, 1 chặn", at_start, [True] * 5 + [False])
    c.check("burst gấp đôi ở ranh giới", burst, 10)
    ttl = await r.ttl(f"{PREFIX}rl:fw:CU-001:0")
    c.check("key cửa sổ có TTL", 0 < ttl <= 60, True)

    # Token bucket: dung lượng 5, nạp 1 token/giây. Mốc thời gian mô phỏng (mili giây).
    allow = make_token_bucket(r)
    base = 1_000_000
    plan = [(0, 8), (2_000, 3), (12_000, 7)]
    say("\ntoken bucket (dung lượng 5, nạp 1 token/s), mô phỏng thời gian:")
    outcomes: list[tuple[int, int]] = []
    for offset, count in plan:
        results = [await allow("CU-002", capacity=5, refill_per_s=1, now_ms=base + offset)
                   for _ in range(count)]
        passed = sum(1 for ok, _ in results if ok)
        left = results[-1][1]
        outcomes.append((passed, count - passed))
        say(f"  t=+{offset / 1000:4.1f} s: {count} yêu cầu -> {passed} qua, "
            f"{count - passed} chặn, còn {left:g} token")
    c.check("bucket đầy: 5 qua 3 chặn", outcomes[0], (5, 3))
    c.check("sau 2 s: 2 token mới", outcomes[1], (2, 1))
    c.check("nghỉ lâu chỉ đầy tới dung lượng", outcomes[2], (5, 2))
    state = await r.hgetall(f"{PREFIX}rl:tb:CU-002")
    say("trạng thái trong Redis:", state, "| PTTL", await r.pttl(f"{PREFIX}rl:tb:CU-002"), "ms")
    c.check("state có milli và ts", sorted(state), ["milli", "ts"])

    # Một instance có đồng hồ chậm 1 giây gọi sau: thời gian đi lùi không được tạo thêm token.
    drained = [await allow("CU-004", capacity=5, refill_per_s=1, now_ms=base + 10_000) for _ in range(5)]
    back, _ = await allow("CU-004", capacity=5, refill_per_s=1, now_ms=base + 9_000)
    later = [(await allow("CU-004", capacity=5, refill_per_s=1, now_ms=base + 11_000))[0] for _ in range(2)]
    say(f"đồng hồ lùi 1 s: gọi lúc t-1 s -> {back}; gọi lúc t+1 s -> {later} (chỉ 1 token mới)")
    c.check("xô cạn rồi", [ok for ok, _ in drained], [True] * 5)
    c.check("đồng hồ lùi không nạp token", back, False)
    c.check("sau 1 s thật chỉ có 1 token", later, [True, False])

    # Đua giữa các caller: 50 coroutine cùng xin token khi bucket có 5.
    capacity = 5
    race_key = f"{PREFIX}rl:race"
    await r.set(race_key, capacity)
    naive_results = await asyncio.gather(*(naive_allow(r, race_key) for _ in range(50)))
    naive_passed = sum(naive_results)
    lua_results = await asyncio.gather(
        *(allow("CU-003", capacity=capacity, refill_per_s=1, now_ms=base) for _ in range(50))
    )
    lua_passed = sum(1 for ok, _ in lua_results if ok)
    say(f"\n50 yêu cầu đồng thời, dung lượng {capacity}: "
        f"GET rồi SET cho qua {naive_passed}, Lua cho qua {lua_passed}")
    c.check("GET/SET cho lọt quá dung lượng", naive_passed > capacity, True)
    c.check("Lua cho qua đúng dung lượng", lua_passed, capacity)


# ---------------------------------------------------------------- 3. GEO vị trí tài xế

HANOI = (21.0285, 105.8542)  # (vĩ độ, kinh độ) quanh Hồ Gươm
EARTH_RADIUS_M = 6_372_797.560856  # bán kính Redis dùng khi tính khoảng cách GEO
DRIVER_COUNT = 25_000
GEO_KEY = f"{PREFIX}geo:drivers:hanoi"


class Driver(NamedTuple):
    driver_id: str
    lat: float
    lon: float


def make_drivers(count: int = DRIVER_COUNT) -> list[Driver]:
    """Tài xế rải đều theo diện tích trong đĩa bán kính 15 km quanh trung tâm Hà Nội."""
    rng = random.Random(SEED)
    drivers = []
    for i in range(1, count + 1):
        radius_km = 15 * math.sqrt(rng.random())
        angle = rng.uniform(0, 2 * math.pi)
        lat = HANOI[0] + radius_km * math.cos(angle) / 111.32
        lon = HANOI[1] + radius_km * math.sin(angle) / (111.32 * math.cos(math.radians(HANOI[0])))
        drivers.append(Driver(f"DR-{i:05d}", round(lat, 6), round(lon, 6)))
    return drivers


def haversine_km(lat1: float, lon1: float, lat2: float, lon2: float) -> float:
    p1, p2 = math.radians(lat1), math.radians(lat2)
    a = (math.sin((p2 - p1) / 2) ** 2
         + math.cos(p1) * math.cos(p2) * math.sin(math.radians(lon2 - lon1) / 2) ** 2)
    return 2 * EARTH_RADIUS_M * math.asin(math.sqrt(a)) / 1000


def nearest_bruteforce(drivers: list[Driver], lat: float, lon: float, count: int) -> list[tuple[str, float]]:
    ranked = sorted((haversine_km(lat, lon, d.lat, d.lon), d.driver_id) for d in drivers)
    return [(driver_id, dist) for dist, driver_id in ranked[:count]]


async def load_drivers_pipeline(r: redis.Redis, key: str, drivers: list[Driver], chunk: int = 5_000) -> None:
    """GEOADD mỗi tài xế một lệnh nhưng gửi theo lô qua pipeline (không cần chờ từng phản hồi)."""
    for start in range(0, len(drivers), chunk):
        async with r.pipeline(transaction=False) as pipe:
            for d in drivers[start:start + chunk]:
                pipe.geoadd(key, (d.lon, d.lat, d.driver_id))
            await pipe.execute()


async def find_nearest(
    r: redis.Redis, lat: float, lon: float, *, radius_km: float, count: int
) -> list[tuple[str, float]]:
    found = await r.geosearch(GEO_KEY, longitude=lon, latitude=lat, radius=radius_km, unit="km",
                              sort="ASC", count=count, withdist=True)
    return [(member, float(dist)) for member, dist in found]


async def part_geo(r: redis.Redis, c: Checker) -> None:
    say("\n== 3. GEO: 25.000 tài xế quanh Hà Nội ==")
    drivers = make_drivers()

    # Cách 1: mỗi tài xế một vòng đi-về tới Redis. Cách 2: pipeline. Cách 3: một GEOADD nhiều thành viên.
    seq_key = f"{PREFIX}geo:seq"
    start = time.perf_counter()
    for d in drivers:
        await r.geoadd(seq_key, (d.lon, d.lat, d.driver_id))
    seq_ms = (time.perf_counter() - start) * 1000
    start = time.perf_counter()
    await load_drivers_pipeline(r, GEO_KEY, drivers)
    pipe_ms = (time.perf_counter() - start) * 1000
    bulk_key = f"{PREFIX}geo:bulk"
    start = time.perf_counter()
    for i in range(0, len(drivers), 5_000):
        values = [v for d in drivers[i:i + 5_000] for v in (d.lon, d.lat, d.driver_id)]
        await r.geoadd(bulk_key, values)
    bulk_ms = (time.perf_counter() - start) * 1000
    say(f"nạp {len(drivers)} tài xế, mỗi lệnh một vòng đi-về : {seq_ms:8.0f} ms")
    say(f"nạp {len(drivers)} tài xế, pipeline lô 5.000 lệnh   : {pipe_ms:8.0f} ms")
    say(f"nạp {len(drivers)} tài xế, 5 lệnh GEOADD x 5.000    : {bulk_ms:8.0f} ms")

    say("kiểu key:", await r.type(GEO_KEY), "| ZCARD =", await r.zcard(GEO_KEY))
    score = await r.zscore(GEO_KEY, "DR-00001")
    say(f"điểm (score) của DR-00001 trong sorted set: {score:.0f} (geohash 52 bit)")
    c.check("GEO là sorted set", await r.type(GEO_KEY), "zset")
    c.check("25.000 thành viên", await r.zcard(GEO_KEY), DRIVER_COUNT)
    c.check("ba cách nạp cho cùng kết quả", (await r.zcard(seq_key), await r.zcard(bulk_key)),
            (DRIVER_COUNT, DRIVER_COUNT))

    pos = (await r.geopos(GEO_KEY, "DR-00001"))[0]
    assert pos is not None
    say(f"vị trí gốc DR-00001: ({drivers[0].lon}, {drivers[0].lat}); Redis trả: ({pos[0]:.6f}, {pos[1]:.6f})")
    c.check("GEOPOS sai lệch dưới 1e-5 độ",
            abs(pos[0] - drivers[0].lon) < 1e-5 and abs(pos[1] - drivers[0].lat) < 1e-5, True)

    # Khách ở gần Hồ Gươm. Tìm 5 tài xế gần nhất trong 3 km; đối chiếu với tính tay trên 25.000 điểm.
    cust_lat, cust_lon = 21.0290, 105.8520
    nearest, search_ms = await timed(find_nearest(r, cust_lat, cust_lon, radius_km=3, count=5))
    expected = nearest_bruteforce(drivers, cust_lat, cust_lon, 5)
    say(f"\nGEOSEARCH 5 tài xế gần nhất trong 3 km ({search_ms:.1f} ms):")
    for (driver_id, dist), (_, ref) in zip(nearest, expected, strict=True):
        say(f"  {driver_id}  {dist:.4f} km   (tính tay haversine: {ref:.4f} km)")
    c.check("cùng 5 mã tài xế như tính tay", [m for m, _ in nearest], [m for m, _ in expected])
    c.check("khoảng cách lệch dưới 1 m",
            all(abs(a[1] - b[1]) < 0.001 for a, b in zip(nearest, expected, strict=True)), True)

    in_radius = await find_nearest(r, cust_lat, cust_lon, radius_km=2, count=DRIVER_COUNT)
    brute_in_radius = sum(1 for d in drivers if haversine_km(cust_lat, cust_lon, d.lat, d.lon) <= 2)
    say(f"số tài xế trong 2 km: Redis {len(in_radius)}, tính tay {brute_in_radius}")
    c.check("số tài xế trong 2 km", len(in_radius), brute_in_radius)

    # Tài xế di chuyển: GEOADD lại cùng member là cập nhật; không phát sinh thành viên mới.
    nearest_id = nearest[0][0]
    await r.geoadd(GEO_KEY, (105.9000, 21.1000, nearest_id))
    moved = await find_nearest(r, cust_lat, cust_lon, radius_km=3, count=5)
    say(f"\n{nearest_id} đi xa: top 5 mới bắt đầu bằng {moved[0][0]}; ZCARD vẫn {await r.zcard(GEO_KEY)}")
    c.check("cập nhật vị trí không thêm member", await r.zcard(GEO_KEY), DRIVER_COUNT)
    c.check("tài xế đã đi xa rời top 5", nearest_id not in [m for m, _ in moved], True)

    # Tài xế offline: GEO không tự hết hạn từng member, phải ZREM. Redis không có TTL cho member.
    await r.zrem(GEO_KEY, moved[0][0])
    c.check("ZREM giảm một thành viên", await r.zcard(GEO_KEY), DRIVER_COUNT - 1)

    # Redis mất dữ liệu (restart không bật persistence, FLUSHALL, failover): dựng lại từ nguồn sự thật.
    await r.delete(GEO_KEY)
    empty = await find_nearest(r, cust_lat, cust_lon, radius_km=3, count=5)
    rebuild_ms = await timed(load_drivers_pipeline(r, GEO_KEY, drivers))
    rebuilt = await find_nearest(r, cust_lat, cust_lon, radius_km=3, count=5)
    say(f"mất key: tìm thấy {len(empty)} tài xế; dựng lại từ nguồn gốc mất {rebuild_ms[1]:.0f} ms, "
        f"top 5 trùng ban đầu: {[m for m, _ in rebuilt] == [m for m, _ in nearest]}")
    c.check("key mất thì tìm không ra gì", empty, [])
    c.check("dựng lại cho cùng kết quả", [m for m, _ in rebuilt], [m for m, _ in nearest])


# ---------------------------------------------------------------- chạy


async def connect() -> redis.Redis:
    r = redis.from_url(REDIS_URL, decode_responses=True, socket_connect_timeout=2)
    try:
        await r.ping()
    except (RedisError, OSError):
        await r.aclose()
        print(f"FAIL: không nối được Redis tại {REDIS_URL} "
              "(docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait)")
        sys.exit(1)
    return r


async def run() -> int:
    c = Checker()
    r = await connect()
    try:
        load = os.getloadavg()
        say(f"Máy đo: {platform.platform()}, {os.cpu_count()} CPU, "
            f"load average {load[0]:.2f} {load[1]:.2f} {load[2]:.2f}; Redis trong container Docker")
        await delete_prefix(r)
        await part_cache(r, c)
        await part_rate_limit(r, c)
        await part_geo(r, c)
    finally:
        removed = await delete_prefix(r)
        leftover = [k async for k in r.scan_iter(match=f"{PREFIX}*")]
        await r.aclose()
    c.check("không còn key lab27:*", leftover, [])

    if c.failures:
        print(f"FAIL {c.total - len(c.failures)}/{c.total}")
        return 1
    if QUIET:
        print(f"OK {c.total}/{c.total}")
    else:
        say(f"\nĐã dọn {removed} key lab27:*. Kiểm chứng: {c.total}/{c.total} ca đạt.")
    return 0


def main() -> None:
    global QUIET
    QUIET = "--verify" in sys.argv[1:]
    sys.exit(asyncio.run(run()))


if __name__ == "__main__":
    main()
