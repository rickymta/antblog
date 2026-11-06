"""Tính đặc trưng ONLINE từ từng envelope Kafka, cùng định nghĩa với features.py (polars, theo lô).

Mỗi khóa (thiết bị, khách, tài xế, cặp khách-tài xế) giữ một danh sách (thời điểm, giá trị) sắp theo thời gian.
Một chuyến "đóng" khi đủ ba event (BookingCreated, TripCompleted, PaymentCaptured); lúc đó ta đọc cửa sổ rồi
mới ghi chuyến vào trạng thái, nên đặc trưng không bao giờ thấy chính nó hay chuyến sau nó.
"""

import math
from bisect import bisect_left, bisect_right
from datetime import UTC, datetime, timedelta
from typing import Any

EPOCH = datetime(1970, 1, 1, tzinfo=UTC)
US = {"1h": 3_600_000_000, "24h": 86_400_000_000, "7d": 7 * 86_400_000_000, "30d": 30 * 86_400_000_000}
IMPOSSIBLE_KMH = 120.0
# Đủ 30 đặc trưng của features.FEATURES (verify kiểm hai danh sách bằng nhau). Giờ địa phương = UTC + 7, Việt Nam không có giờ mùa hè.
ONLINE_FEATURES = [
    "distance_km", "duration_s", "amount_vnd", "discount_ratio", "has_promo", "has_newuser_promo",
    "hour_local", "is_night", "same_device",
    "hav_start_end_km", "start_pickup_gap_km", "route_ratio", "speed_kmh",
    "dev_trips_1h", "dev_trips_24h", "cust_trips_1h", "cust_trips_24h", "drv_trips_1h", "drv_trips_24h",
    "dev_customers_24h", "dev_customers_7d", "pair_trips_7d",
    "cust_prior_trips", "cust_promo_share_30d", "dev_newuser_7d",
    "drv_jump_km", "drv_gap_s", "drv_jump_speed_kmh", "impossible_speed", "impossible_jump",
]


def to_us(iso: str) -> int:
    """Chuỗi ISO UTC của envelope -> micro giây từ epoch (số nguyên, không sai số dấu phẩy động)."""
    return (datetime.fromisoformat(iso) - EPOCH) // timedelta(microseconds=1)


def haversine_km(lat1: float, lng1: float, lat2: float, lng2: float) -> float:
    p1, p2 = math.radians(lat1), math.radians(lat2)
    a = math.sin((p2 - p1) / 2) ** 2 + math.cos(p1) * math.cos(p2) * math.sin(math.radians(lng2 - lng1) / 2) ** 2
    return 2 * 6371.0 * math.asin(math.sqrt(a))


class Window:
    """Danh sách (ts, giá trị) sắp theo ts; chèn đúng chỗ kể cả khi event đến lệch thứ tự."""

    __slots__ = ("ts", "vals")

    def __init__(self) -> None:
        self.ts: list[int] = []
        self.vals: list[Any] = []

    def add(self, ts: int, val: Any = 1) -> None:
        i = bisect_right(self.ts, ts)
        self.ts.insert(i, ts)
        self.vals.insert(i, val)

    def count_before(self, t: int, width: int) -> int:
        """Số mục trong [t - width, t)."""
        return bisect_left(self.ts, t) - bisect_left(self.ts, t - width)

    def sum_before(self, t: int, width: int) -> int:
        return sum(self.vals[bisect_left(self.ts, t - width):bisect_left(self.ts, t)])

    def distinct_upto(self, t: int, width: int) -> int:
        """Số giá trị khác nhau trong [t - width, t] (gồm cả mục tại t)."""
        return len(set(self.vals[bisect_left(self.ts, t - width):bisect_right(self.ts, t)]))


class OnlineFeatures:
    def __init__(self) -> None:
        self.dev, self.cust, self.drv, self.pair = ({} for _ in range(4))
        self.dev_cust: dict[str, Window] = {}
        self.dev_new: dict[str, Window] = {}
        self.cust_promo: dict[str, Window] = {}
        self.prior_trips: dict[str, int] = {}
        self.last_trip: dict[str, tuple[int, float, float]] = {}  # tài xế -> (completed_us, end_lat, end_lng)
        self.pending: dict[str, dict[str, dict[str, Any]]] = {}
        self.closed_order: list[str] = []

    def on_envelope(self, env: dict[str, Any]) -> tuple[str, dict[str, float]] | None:
        """Nhận một envelope; trả (trip_id, đặc trưng) khi chuyến vừa đủ ba event, ngược lại None."""
        p = env["payload"]
        slot = self.pending.setdefault(p["trip_id"], {})
        slot[env["event_type"]] = {**p, "_us": to_us(env["occurred_at"])}
        if len(slot) < 3:
            return None
        del self.pending[p["trip_id"]]
        self.closed_order.append(p["trip_id"])
        return p["trip_id"], self._close(slot["BookingCreated"], slot["TripCompleted"], slot["PaymentCaptured"])

    def _close(self, b: dict[str, Any], c: dict[str, Any], pay: dict[str, Any]) -> dict[str, float]:
        t = pay["_us"]
        dev, cust, drv = b["device_id"], b["customer_id"], c["driver_id"]
        pair = f"{cust}|{drv}"
        w = {name: m.setdefault(key, Window()) for name, m, key in
             (("dev", self.dev, dev), ("cust", self.cust, cust), ("drv", self.drv, drv), ("pair", self.pair, pair),
              ("dev_cust", self.dev_cust, dev), ("dev_new", self.dev_new, dev), ("cust_promo", self.cust_promo, cust))}
        newuser = 1 if b.get("promo_code") == "NEWUSER50" else 0
        has_promo = 1 if b.get("promo_code") is not None else 0
        hav = haversine_km(c["start_lat"], c["start_lng"], c["end_lat"], c["end_lng"])
        speed = hav / (c["duration_s"] / 3600)
        n30 = w["cust"].count_before(t, US["30d"])
        hour_local = float(((b["_us"] // 3_600_000_000) + 7) % 24)
        f = {
            "distance_km": c["distance_km"], "duration_s": c["duration_s"], "amount_vnd": pay["amount_vnd"],
            "discount_ratio": pay["discount_vnd"] / (pay["amount_vnd"] + pay["discount_vnd"]),
            "has_promo": has_promo, "has_newuser_promo": newuser, "hour_local": hour_local,
            "is_night": 1.0 if hour_local < 6 else 0.0, "same_device": 1.0 if dev == c["driver_device_id"] else 0.0,
            "hav_start_end_km": hav, "start_pickup_gap_km": haversine_km(b["pickup_lat"], b["pickup_lng"], c["start_lat"], c["start_lng"]),
            "route_ratio": hav / c["distance_km"], "speed_kmh": speed, "impossible_speed": 1.0 if speed > IMPOSSIBLE_KMH else 0.0,
            "cust_promo_share_30d": w["cust_promo"].sum_before(t, US["30d"]) / n30 if n30 > 0 else 0.0,
            "dev_trips_1h": w["dev"].count_before(t, US["1h"]), "dev_trips_24h": w["dev"].count_before(t, US["24h"]),
            "cust_trips_1h": w["cust"].count_before(t, US["1h"]), "cust_trips_24h": w["cust"].count_before(t, US["24h"]),
            "drv_trips_1h": w["drv"].count_before(t, US["1h"]), "drv_trips_24h": w["drv"].count_before(t, US["24h"]),
            "pair_trips_7d": w["pair"].count_before(t, US["7d"]),
            "dev_newuser_7d": w["dev_new"].sum_before(t, US["7d"]),
            "cust_prior_trips": self.prior_trips.get(cust, 0),
        }
        w["dev_cust"].add(t, cust)  # chuyến hiện tại tự được tính vào "khách trên thiết bị"
        f["dev_customers_24h"] = w["dev_cust"].distinct_upto(t, US["24h"])
        f["dev_customers_7d"] = w["dev_cust"].distinct_upto(t, US["7d"])
        started = c["_us"] - c["duration_s"] * 1_000_000
        prev = self.last_trip.get(drv)
        if prev is None:
            f["drv_jump_km"] = f["drv_gap_s"] = f["drv_jump_speed_kmh"] = f["impossible_jump"] = 0.0
        else:
            jump = haversine_km(prev[1], prev[2], c["start_lat"], c["start_lng"])
            gap = max(60, (started - prev[0]) // 1_000_000)
            f["drv_jump_km"], f["drv_gap_s"], f["drv_jump_speed_kmh"] = jump, float(gap), jump / (gap / 3600)
        f["impossible_jump"] = 1.0 if f["drv_jump_speed_kmh"] > IMPOSSIBLE_KMH else 0.0
        # ghi chuyến vào trạng thái SAU khi đã đọc xong đặc trưng
        for name in ("dev", "cust", "drv", "pair"):
            w[name].add(t)
        w["dev_new"].add(t, newuser)
        w["cust_promo"].add(t, has_promo)
        self.prior_trips[cust] = self.prior_trips.get(cust, 0) + 1
        self.last_trip[drv] = (c["_us"], c["end_lat"], c["end_lng"])
        return {k: float(v) for k, v in f.items()}
