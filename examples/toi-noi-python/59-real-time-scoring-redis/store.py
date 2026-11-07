"""Feature store ONLINE trên Redis: ghi trạng thái từ event, đọc đặc trưng cho một chuyến trong một vòng mạng.

Cùng 30 đặc trưng `FEATURES` với bản theo lô (features.py, polars) nhưng tính từ trạng thái trong Redis.
Hai kiểu key (mọi key có tiền tố `lab59:`):

    lab59:win:dev:{DEV}      sorted set, score = paid_at (micro giây từ epoch), member = "trip|khách|newuser"
    lab59:win:cust:{CU}      sorted set, member = "trip|promo"        (giữ 30 ngày)
    lab59:win:drv:{DR}       sorted set, member = "trip"              (giữ 24 giờ)
    lab59:win:pair:{CU}|{DR} sorted set, member = "trip"              (giữ 7 ngày)
    lab59:feat:{CU}          hash: prior_trips (số chuyến của khách đã ghi, không hết hạn), last_us (paid_at mới nhất của khách)
    lab59:feat:{DR}          hash: paid_us, done_us, end_lat, end_lng (chuyến cuối của tài xế) và prev_* (chuyến liền trước nó)
    lab59:meta               hash: applied_until (paid_at lớn nhất đã ghi), history_from (paid_at nhỏ nhất đã ghi)

Hash là trạng thái MỚI NHẤT; sorted set thì đọc được theo thời điểm. Một chuyến đã được Updater ghi rồi mới được chấm sẽ
thấy chính mình trong hash: `read_features` phát hiện điều đó (last_us của khách hoặc paid_us của tài xế bằng paid_at của
chuyến) và trừ đi, dùng prev_* cho vị trí cuối. Nếu đã có chuyến SAU nó (last_us hoặc paid_us lớn hơn) thì hash không còn
khôi phục được: trạng thái trả về là "ahead" và người gọi không được chấm bằng nó.

Đặc trưng cửa sổ («bao nhiêu chuyến trong 1 giờ trước») phụ thuộc thời điểm hỏi, nên không lưu thành một con số
trong hash: ta lưu thời điểm từng chuyến rồi đếm theo [t - W, t) lúc đọc. Redis không phải nguồn sự thật
(bài 27): mất dữ liệu thì dựng lại bằng cách phát lại event theo thứ tự `paid_at`.
"""

import math
from collections import defaultdict
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from typing import Any, NamedTuple
from zoneinfo import ZoneInfo

import redis.asyncio as redis

from features import BASE_FEATURES, FEATURES, IMPOSSIBLE_KMH

PREFIX = "lab59:"
EPOCH = datetime(1970, 1, 1, tzinfo=UTC)
LOCAL_TZ = ZoneInfo("Asia/Ho_Chi_Minh")
US_H = 3_600_000_000
US_D = 24 * US_H
KEEP_DEV, KEEP_CUST, KEEP_DRV, KEEP_PAIR = 7 * US_D, 30 * US_D, US_D, 7 * US_D  # đúng bằng cửa sổ dài nhất của từng khóa
SEP = "|"


def to_us(iso: str) -> int:
    """Chuỗi ISO UTC của envelope -> micro giây từ epoch (số nguyên, không sai số dấu phẩy động)."""
    return (datetime.fromisoformat(iso) - EPOCH) // timedelta(microseconds=1)


def haversine_km(lat1: float, lng1: float, lat2: float, lng2: float) -> float:
    p1, p2 = math.radians(lat1), math.radians(lat2)
    a = math.sin((p2 - p1) / 2) ** 2 + math.cos(p1) * math.cos(p2) * math.sin(math.radians(lng2 - lng1) / 2) ** 2
    return 2 * 6371.0 * math.asin(math.sqrt(a))


@dataclass(frozen=True)
class ClosedTrip:
    """Một chuyến đã đủ ba event (đặt, hoàn tất, thu tiền): đầu vào của chấm điểm, chấm tại `paid_us`."""

    trip_id: str
    customer_id: str
    device_id: str
    promo_code: str | None
    pickup_lat: float
    pickup_lng: float
    booked_us: int
    driver_id: str
    driver_device_id: str
    start_lat: float
    start_lng: float
    end_lat: float
    end_lng: float
    distance_km: float
    duration_s: int
    completed_us: int
    amount_vnd: int
    discount_vnd: int
    paid_us: int

    @classmethod
    def from_events(cls, booking: dict[str, Any], done: dict[str, Any], pay: dict[str, Any]) -> "ClosedTrip":
        b, d, p = booking["payload"], done["payload"], pay["payload"]
        return cls(
            trip_id=b["trip_id"], customer_id=b["customer_id"], device_id=b["device_id"], promo_code=b.get("promo_code"),
            pickup_lat=b["pickup_lat"], pickup_lng=b["pickup_lng"], booked_us=to_us(booking["occurred_at"]),
            driver_id=d["driver_id"], driver_device_id=d["driver_device_id"], start_lat=d["start_lat"], start_lng=d["start_lng"],
            end_lat=d["end_lat"], end_lng=d["end_lng"], distance_km=d["distance_km"], duration_s=d["duration_s"],
            completed_us=to_us(done["occurred_at"]), amount_vnd=p["amount_vnd"], discount_vnd=p["discount_vnd"],
            paid_us=to_us(pay["occurred_at"]),
        )


class TripAssembler:
    """Gom ba envelope cùng trip_id thành một ClosedTrip. Trạng thái chờ nằm trong bộ nhớ tiến trình: khởi động lại là mất."""

    def __init__(self) -> None:
        self._pending: dict[str, dict[str, dict[str, Any]]] = defaultdict(dict)

    def add(self, env: dict[str, Any]) -> ClosedTrip | None:
        slot = self._pending[env["payload"]["trip_id"]]
        slot[env["event_type"]] = env
        if len(slot) < 3:
            return None
        del self._pending[env["payload"]["trip_id"]]
        return ClosedTrip.from_events(slot["BookingCreated"], slot["TripCompleted"], slot["PaymentCaptured"])

    @property
    def pending(self) -> int:
        return len(self._pending)


# Ghi MỘT chuyến vào mọi key trong một script Lua: Redis chạy liền một mạch nên hoặc đủ sáu key cùng đổi, hoặc không đổi gì,
# và ghi lại cùng một chuyến (Kafka giao ít nhất một lần) là no-op. Điểm số (paid_at) và mốc cắt đi vào dưới dạng CHUỖI từ
# Python: Lua đổi số thành chuỗi bằng "%.14g", sẽ làm tròn một mốc 16 chữ số micro giây.
_RECORD_LUA = """
local score = ARGV[1]
if redis.call('ZADD', KEYS[2], score, ARGV[3]) == 0 then return 0 end   -- đã ghi chuyến này rồi
redis.call('ZADD', KEYS[1], score, ARGV[2])
redis.call('ZADD', KEYS[3], score, ARGV[4])
redis.call('ZADD', KEYS[4], score, ARGV[4])
for i = 1, 4 do
  redis.call('ZREMRANGEBYSCORE', KEYS[i], '-inf', '(' .. ARGV[4 + i])   -- bỏ mục cũ hơn cửa sổ dài nhất
  redis.call('PEXPIRE', KEYS[i], ARGV[8 + i])                           -- khóa không còn hoạt động thì tự biến mất
end
redis.call('HINCRBY', KEYS[5], 'prior_trips', 1)
local cl = redis.call('HGET', KEYS[5], 'last_us')
if (not cl) or tonumber(cl) <= tonumber(score) then redis.call('HSET', KEYS[5], 'last_us', score) end
local last = redis.call('HGET', KEYS[6], 'paid_us')
if (not last) or tonumber(last) <= tonumber(score) then                 -- chuyến đến muộn không ghi đè chuyến cuối
  if last then                                                          -- giữ chuyến cuối cũ làm «chuyến liền trước»
    local old = redis.call('HMGET', KEYS[6], 'paid_us', 'done_us', 'end_lat', 'end_lng')
    redis.call('HSET', KEYS[6], 'prev_paid_us', old[1], 'prev_done_us', old[2], 'prev_end_lat', old[3], 'prev_end_lng', old[4])
  end
  redis.call('HSET', KEYS[6], 'paid_us', score, 'done_us', ARGV[13], 'end_lat', ARGV[14], 'end_lng', ARGV[15])
end
local mark = redis.call('HGET', KEYS[7], 'applied_until')
if (not mark) or tonumber(mark) < tonumber(score) then redis.call('HSET', KEYS[7], 'applied_until', score) end
redis.call('HSETNX', KEYS[7], 'history_from', score)                    -- chuyến đầu tiên store thấy
return 1
"""


def key_dev(device_id: str) -> str:
    return f"{PREFIX}win:dev:{device_id}"


def key_cust(customer_id: str) -> str:
    return f"{PREFIX}win:cust:{customer_id}"


def key_drv(driver_id: str) -> str:
    return f"{PREFIX}win:drv:{driver_id}"


def key_pair(customer_id: str, driver_id: str) -> str:
    return f"{PREFIX}win:pair:{customer_id}{SEP}{driver_id}"


def key_feat(entity_id: str) -> str:
    return f"{PREFIX}feat:{entity_id}"


KEY_META = f"{PREFIX}meta"
MIN_HISTORY_US = 30 * US_D  # cửa sổ dài nhất (khách, 30 ngày): store phải có ít nhất chừng này lịch sử mới chấm được

# Feature store này phủ cả 30 cột của FEATURES, theo ba nhóm. Bản online của bài 55 (online.py) chỉ có nhóm giữa (14 cột)
# và thiếu `cust_promo_share_30d`; lab 59 bổ sung cột đó và hai cờ dẫn xuất. Mô hình đòi cột nào ngoài tập này thì
# `missing_for` báo tên cột và Scorer từ chối khởi tạo, thay vì điền 0 trong im lặng.
FROM_EVENT = tuple(BASE_FEATURES)  # 13 cột tính từ chính ba event của chuyến, không cần Redis
FROM_STATE = (  # 14 cột của online.py (bài 55): đọc từ trạng thái trong Redis
    "dev_trips_1h", "dev_trips_24h", "cust_trips_1h", "cust_trips_24h", "drv_trips_1h", "drv_trips_24h",
    "dev_customers_24h", "dev_customers_7d", "pair_trips_7d", "dev_newuser_7d", "cust_prior_trips",
    "drv_jump_km", "drv_gap_s", "drv_jump_speed_kmh",
)
ADDED_IN_59 = ("cust_promo_share_30d", "impossible_speed", "impossible_jump")  # 1 cột cửa sổ mới, 2 cờ dẫn xuất
COVERED = frozenset(FROM_EVENT + FROM_STATE + ADDED_IN_59)


class FeatureRead(NamedTuple):
    features: dict[str, float]
    applied_until_us: int | None  # paid_at lớn nhất đã ghi vào store; None = store trống (mới khởi động hoặc mất dữ liệu)
    history_from_us: int | None = None  # paid_at nhỏ nhất đã ghi: store có bao nhiêu lịch sử
    state: str = "before"  # "before": chuyến chưa được ghi; "latest": đã ghi và là chuyến mới nhất (đã trừ chính nó); "ahead": đã có chuyến sau nó


class FeatureStore:
    """Ghi (`record_trip`) và đọc (`read_features`) đặc trưng online. Một client `redis.asyncio` dùng chung, có timeout socket."""

    def __init__(self, client: redis.Redis) -> None:
        self.r = client
        self._record = client.register_script(_RECORD_LUA)

    def _record_args(self, t: ClosedTrip) -> tuple[list[str], list[str]]:
        s = t.paid_us
        keys = [key_dev(t.device_id), key_cust(t.customer_id), key_drv(t.driver_id), key_pair(t.customer_id, t.driver_id),
                key_feat(t.customer_id), key_feat(t.driver_id), KEY_META]
        newuser = 1 if t.promo_code == "NEWUSER50" else 0
        args = [
            str(s), SEP.join((t.trip_id, t.customer_id, str(newuser))), SEP.join((t.trip_id, str(int(t.promo_code is not None)))), t.trip_id,
            str(s - KEEP_DEV), str(s - KEEP_CUST), str(s - KEEP_DRV), str(s - KEEP_PAIR),
            *(str(k // 1000 + 3_600_000) for k in (KEEP_DEV, KEEP_CUST, KEEP_DRV, KEEP_PAIR)),  # PEXPIRE: cửa sổ + 1 giờ
            str(t.completed_us), repr(t.end_lat), repr(t.end_lng),
        ]
        return keys, args

    @staticmethod
    def missing_for(feature_list: Any) -> list[str]:
        """Cột mô hình cần mà store không tính được (rỗng nếu đủ)."""
        return [f for f in feature_list if f not in COVERED]

    async def record_trip(self, t: ClosedTrip) -> bool:
        """Ghi chuyến vào store. Trả True nếu là chuyến mới, False nếu đã ghi rồi (no-op)."""
        keys, args = self._record_args(t)
        return bool(await self._record(keys=keys, args=args))

    async def record_many(self, trips: list[ClosedTrip]) -> int:
        """Ghi một lô theo thứ tự đã cho trong MỘT pipeline (một vòng mạng cho cả lô). Trả số chuyến mới."""
        pipe = self.r.pipeline(transaction=False)
        for t in trips:
            keys, args = self._record_args(t)
            await self._record(keys=keys, args=args, client=pipe)  # với pipeline: chỉ xếp hàng, chưa chạy
        return sum(int(x) for x in await pipe.execute())

    async def read_features(self, t: ClosedTrip) -> FeatureRead:
        """30 đặc trưng của chuyến `t` tại thời điểm `t.paid_us`: trạng thái TRƯỚC chuyến, cộng chính ba event của nó.

        Một pipeline = một vòng mạng cho cả chín lệnh đọc. Cận trên loại trừ chỉ loại chính chuyến khỏi các sorted set;
        hash `prior_trips` và vị trí cuối là trạng thái mới nhất nên cần xử lý riêng nếu Updater đã ghi chuyến này
        (xem `state` của kết quả)."""
        s = t.paid_us
        before = f"({s}"  # cận trên loại trừ: cửa sổ [s - W, s), giống closed="left" của bản polars
        pipe = self.r.pipeline(transaction=False)
        pipe.zrangebyscore(key_dev(t.device_id), s - KEEP_DEV, s, withscores=True)
        pipe.zrangebyscore(key_cust(t.customer_id), s - KEEP_CUST, before, withscores=True)
        pipe.zcount(key_drv(t.driver_id), s - US_H, before)
        pipe.zcount(key_drv(t.driver_id), s - US_D, before)
        pipe.zcount(key_pair(t.customer_id, t.driver_id), s - 7 * US_D, before)
        pipe.hmget(key_feat(t.customer_id), "prior_trips", "last_us")
        pipe.hmget(key_feat(t.driver_id), "paid_us", "done_us", "end_lat", "end_lng", "prev_done_us", "prev_end_lat", "prev_end_lng")
        pipe.hmget(KEY_META, "applied_until", "history_from")
        pipe.zscore(key_cust(t.customer_id), SEP.join((t.trip_id, str(int(t.promo_code is not None)))))  # chính chuyến này đã được ghi chưa
        dev, cust, drv_1h, drv_24h, pair_7d, cu, drv, meta, mine = await pipe.execute()
        prior_n = int(cu[0]) if cu[0] is not None else 0
        cust_last = int(cu[1]) if cu[1] is not None else None
        drv_paid = int(drv[0]) if drv[0] is not None else None
        last, state = drv[1:4], "before"
        if (cust_last is not None and cust_last > s) or (drv_paid is not None and drv_paid > s):
            state = "ahead"  # đã có chuyến sau: hash không khôi phục được, người gọi phải bỏ kết quả
        elif mine is not None:  # Updater đã ghi chính chuyến này và nó là chuyến mới nhất: trừ nó khỏi hash
            state = "latest"
            prior_n -= 1
            if drv_paid == s:  # chuyến cuối của tài xế chính là chuyến này: dùng chuyến liền trước nó
                last = drv[4:7]
        feats = compute_features(t, dev, cust, drv_1h, drv_24h, pair_7d, prior_n, last)
        return FeatureRead(feats, int(meta[0]) if meta[0] else None, int(meta[1]) if meta[1] else None, state)

    async def clear(self) -> int:
        """Xóa mọi key `lab59:*` (dọn sau khi chạy). Trả số key đã xóa."""
        n, batch = 0, []
        async for k in self.r.scan_iter(match=f"{PREFIX}*", count=2000):
            batch.append(k)
            if len(batch) >= 2000:
                n += await self.r.unlink(*batch)
                batch = []
        if batch:
            n += await self.r.unlink(*batch)
        return n


def compute_features(t: ClosedTrip, dev, cust, drv_1h: int, drv_24h: int, pair_7d: int, prior: int, last) -> dict[str, float]:
    """Thuần: từ kết quả thô của Redis ra 30 đặc trưng, đúng định nghĩa và đúng tên của `features.FEATURES`.

    dev:  [(member, score)] của thiết bị trong [s - 7d, s], gồm cả mục bằng s;  cust: [(member, score)] trong [s - 30d, s)
    prior: số chuyến trước đó của khách (đã trừ chính chuyến nếu nó đã được ghi); last: (done_us, end_lat, end_lng) của chuyến liền trước của tài xế
    """
    s = t.paid_us
    # ---- theo từng chuyến (không cần Redis) ----
    hav = haversine_km(t.start_lat, t.start_lng, t.end_lat, t.end_lng)
    gap_pickup = haversine_km(t.pickup_lat, t.pickup_lng, t.start_lat, t.start_lng)
    speed = hav / (t.duration_s / 3600)
    hour = (EPOCH + timedelta(microseconds=t.booked_us)).astimezone(LOCAL_TZ).hour
    f: dict[str, float] = {
        "distance_km": t.distance_km, "duration_s": float(t.duration_s), "amount_vnd": float(t.amount_vnd),
        "discount_ratio": t.discount_vnd / (t.amount_vnd + t.discount_vnd),
        "has_promo": float(t.promo_code is not None), "has_newuser_promo": float(t.promo_code == "NEWUSER50"),
        "hour_local": float(hour), "is_night": float(hour < 6), "same_device": float(t.device_id == t.driver_device_id),
        "hav_start_end_km": hav, "start_pickup_gap_km": gap_pickup, "route_ratio": hav / t.distance_km, "speed_kmh": speed,
    }
    # ---- thiết bị: một lần đọc 7 ngày cho năm đặc trưng ----
    dev_rows = [(m.split(SEP), int(sc)) for m, sc in dev]
    prior_dev = [(m, sc) for m, sc in dev_rows if sc < s]
    f["dev_trips_1h"] = float(sum(1 for _, sc in prior_dev if sc >= s - US_H))
    f["dev_trips_24h"] = float(sum(1 for _, sc in prior_dev if sc >= s - US_D))
    f["dev_newuser_7d"] = float(sum(int(m[2]) for m, _ in prior_dev))
    # khách khác nhau trên thiết bị, gồm cả khách của chuyến hiện tại (đó là «khách mới trên thiết bị»)
    f["dev_customers_24h"] = float(len({m[1] for m, sc in prior_dev if sc >= s - US_D} | {t.customer_id}))
    f["dev_customers_7d"] = float(len({m[1] for m, _ in prior_dev} | {t.customer_id}))
    # ---- khách ----
    cust_rows = [(m.split(SEP), int(sc)) for m, sc in cust]
    f["cust_trips_1h"] = float(sum(1 for _, sc in cust_rows if sc >= s - US_H))
    f["cust_trips_24h"] = float(sum(1 for _, sc in cust_rows if sc >= s - US_D))
    f["cust_prior_trips"] = float(prior)
    promos = sum(int(m[1]) for m, _ in cust_rows)
    f["cust_promo_share_30d"] = promos / len(cust_rows) if cust_rows else 0.0
    # ---- tài xế, cặp ----
    f["drv_trips_1h"], f["drv_trips_24h"], f["pair_trips_7d"] = float(drv_1h), float(drv_24h), float(pair_7d)
    prev_done, prev_lat, prev_lng = last
    if prev_done is None:
        jump = gap = jump_speed = 0.0
    else:
        started = t.completed_us - t.duration_s * 1_000_000
        jump = haversine_km(float(prev_lat), float(prev_lng), t.start_lat, t.start_lng)
        gap = float(max(60, (started - int(prev_done)) // 1_000_000))
        jump_speed = jump / (gap / 3600)
    f["drv_jump_km"], f["drv_gap_s"], f["drv_jump_speed_kmh"] = jump, gap, jump_speed
    f["impossible_speed"], f["impossible_jump"] = float(speed > IMPOSSIBLE_KMH), float(jump_speed > IMPOSSIBLE_KMH)
    return {name: f[name] for name in FEATURES}
