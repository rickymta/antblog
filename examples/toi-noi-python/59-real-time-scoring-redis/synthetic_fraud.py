"""Bộ dữ liệu gian lận TỔNG HỢP của Tới Nơi, dùng chung cho các bài 54 đến 61.

Mọi thứ ở đây là giả: khách, tài xế, thiết bị, tọa độ, số tiền và cả tỉ lệ gian lận đều do
bộ sinh số ngẫu nhiên có seed tạo ra. Mọi con số đo trên bộ này KHÔNG đại diện cho dữ liệu thật
của bất kỳ nền tảng nào. Các bài sau copy nguyên file này, không sửa.

    from synthetic_fraud import make_events
    data = make_events(seed=42, n_trips=200_000)   # FraudData(events, labels)

Ba bảng quan trọng nhất là `events` (log sự kiện, tăng dần theo thời gian), `labels` (nhãn, nằm
RIÊNG, biết trễ) và hàm `iter_envelopes` (đổi một dòng `events` thành envelope Kafka).

------------------------------------------------------------------------------------------
events: 1 dòng = 1 sự kiện, sắp theo occurred_at tăng dần. 3 loại sự kiện cho mỗi chuyến.
Cột không thuộc loại sự kiện đó là null. KHÔNG có cột nhãn.

  cột               kiểu                  có ở sự kiện        ý nghĩa
  event_id          String                cả ba               UUID v4 giả (dạng 8-4-4-4-12)
  event_type        String                cả ba               BookingCreated | TripCompleted | PaymentCaptured
  occurred_at       Datetime[us, UTC]     cả ba               thời điểm xảy ra
  version           Int8                  cả ba               1
  trip_id           String                cả ba               TN-0000001 (đánh số theo giờ đặt)
  customer_id       String                cả ba               CU-00001
  device_id         String                BookingCreated      DEV-00001, thiết bị của khách khi đặt
  service           String                BookingCreated      ride | food | parcel
  city              String                BookingCreated      một trong 6 thành phố của Tới Nơi
  pickup_lat/_lng   Float64               BookingCreated      điểm đón khách khai báo
  promo_code        String                BookingCreated      mã khuyến mãi hoặc null
  driver_id         String                TripCompleted       DR-00001
  driver_device_id  String                TripCompleted       DEV-00001, thiết bị của tài xế
  start_lat/_lng    Float64               TripCompleted       GPS của tài xế lúc bắt đầu chuyến
  end_lat/_lng      Float64               TripCompleted       GPS của tài xế lúc kết thúc
  distance_km       Float64               TripCompleted       quãng đường khai báo (đồng hồ cước)
  duration_s        Int32                 TripCompleted       thời lượng chuyến, giây
  amount_vnd        Int64                 PaymentCaptured     số tiền đã thu (sau khuyến mãi), VND nguyên
  discount_vnd      Int64                 PaymentCaptured     phần khuyến mãi đã trừ, VND nguyên

labels: 1 dòng = 1 chuyến (200_000 dòng với n_trips mặc định), tách khỏi events vì nhãn thật
chỉ BIẾT SAU chuyến một khoảng trễ.

  trip_id           String                khóa nối với events
  fraud_type        String                none | ghost_trip | gps_spoof | promo_abuse | collusion
  is_fraud          Boolean               fraud_type != "none"
  label_known_at    Datetime[us, UTC]     lúc nhãn này được biết: chuyến gian lận bị phát hiện sau
                                          vài ngày đến vài tuần; chuyến sạch chỉ "xác nhận sạch" khi
                                          hết hạn khiếu nại 30 ngày

Bộ sinh: 60 ngày từ 02/03/2026 (giờ Việt Nam), 6 thành phố, khoảng 2 % chuyến gian lận chia
thành bốn loại. Mọi chuyến đều hoàn tất (không có chuyến hủy, để bảng gọn). ID khách, tài xế,
thiết bị được xáo ngẫu nhiên nên số thứ tự trong ID không tiết lộ ai gian lận.
"""

import json
import math
from collections.abc import Iterator
from datetime import UTC, datetime
from typing import Any, NamedTuple

import numpy as np
import polars as pl

# ---- Hằng số miền -------------------------------------------------------------------------

FRAUD_TYPES = ("none", "ghost_trip", "gps_spoof", "promo_abuse", "collusion")
EVENT_TYPES = ("BookingCreated", "TripCompleted", "PaymentCaptured")
TOPICS = {  # envelope từ từ điển miền; lab thêm tiền tố "labNN." khi chạy
    "BookingCreated": "booking.created.v1",
    "TripCompleted": "trip.completed.v1",
    "PaymentCaptured": "payment.captured.v1",
}
# Gốc thời gian: 00:00 ngày 02/03/2026 giờ Việt Nam = 17:00 ngày 01/03/2026 UTC.
START = datetime(2026, 3, 1, 17, 0, tzinfo=UTC)
START_US = int(START.timestamp()) * 1_000_000
DAYS = 60
DAY_S = 86_400

FRAUD_RATE = 0.02
FRAUD_SPLIT = {"ghost_trip": 0.35, "promo_abuse": 0.30, "gps_spoof": 0.20, "collusion": 0.15}

# (tên, vĩ độ, kinh độ tâm, trọng số, bán kính khu phục vụ km)
CITIES = (
    ("Hà Nội", 21.0285, 105.8542, 0.28, 14.0),
    ("TP.HCM", 10.7769, 106.7009, 0.34, 16.0),
    ("Đà Nẵng", 16.0544, 108.2022, 0.12, 9.0),
    ("Hải Phòng", 20.8449, 106.6881, 0.10, 9.0),
    ("Cần Thơ", 10.0452, 105.7469, 0.08, 8.0),
    ("Nha Trang", 12.2388, 109.1967, 0.08, 8.0),
)
CITY_P = np.array([c[3] for c in CITIES])
SERVICES = ("ride", "food", "parcel")
SERVICE_P = np.array([0.60, 0.30, 0.10])
# Trọng số theo giờ địa phương 0..23: đỉnh sáng 7-9 giờ và chiều 17-19 giờ.
HOUR_W = np.array([1, 1, 1, 1, 1, 2, 4, 9, 10, 7, 5, 5, 6, 5, 4, 4, 6, 9, 10, 8, 5, 3, 2, 1], dtype=float)
HOUR_P = HOUR_W / HOUR_W.sum()

# Mô hình cước học tập của loạt bài (bài 02/03): 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt.
BASE_FARE = 12_000
INCLUDED_KM = 2.0
PRICE_PER_EXTRA_KM = 4_500
# Khuyến mãi: tên -> (tỉ lệ giảm, trần giảm VND)
PROMOS = {"": (0.0, 0), "NEWUSER50": (0.50, 30_000), "WEEKEND10": (0.10, 15_000),
          "FOOD15": (0.15, 20_000), "COMEBACK20": (0.20, 25_000)}
PROMO_NAMES = tuple(PROMOS)
# Độ trễ nhãn (ngày): trung vị và độ lệch log của phân phối lognormal; chuyến sạch xác nhận sau 30 ngày.
LABEL_DELAY = {"ghost_trip": (6.0, 0.7), "gps_spoof": (8.0, 0.7), "promo_abuse": (3.0, 0.6), "collusion": (16.0, 0.6)}
CLEAN_CONFIRM_DAYS = 30.0
KM_PER_DEG = 111.32


class FraudData(NamedTuple):
    events: pl.DataFrame
    labels: pl.DataFrame


def haversine_km(lat1: Any, lng1: Any, lat2: Any, lng2: Any) -> Any:
    """Khoảng cách đường chim bay giữa hai điểm (km), bán kính Trái Đất 6371 km; nhận mảng numpy."""
    p1, p2 = np.radians(lat1), np.radians(lat2)
    dphi = p2 - p1
    dlmb = np.radians(lng2) - np.radians(lng1)
    a = np.sin(dphi / 2) ** 2 + np.cos(p1) * np.cos(p2) * np.sin(dlmb / 2) ** 2
    return 2 * 6371.0 * np.arcsin(np.sqrt(a))


def fare_vnd(distance_km: np.ndarray) -> np.ndarray:
    """Cước theo mô hình học tập, làm tròn nửa đồng lên (ROUND_HALF_UP), ra VND nguyên."""
    raw = BASE_FARE + np.maximum(0.0, distance_km - INCLUDED_KM) * PRICE_PER_EXTRA_KM
    return np.floor(raw + 0.5).astype(np.int64)


# ---- Quần thể ------------------------------------------------------------------------------


class _World:
    """Danh sách khách, tài xế, thiết bị. Chỉ số là số nguyên; ID chữ chỉ gán ở bước cuối."""

    def __init__(self, rng: np.random.Generator, n_trips: int) -> None:
        self.rng = rng
        n_c = max(2_000, n_trips // 4)
        n_d = max(60, n_trips // 170)
        self.cust_city = list(rng.choice(len(CITIES), size=n_c, p=CITY_P))
        self.cust_dev = list(range(n_c))
        self.cust_w = rng.lognormal(0.0, 1.2, size=n_c)  # vài khách đi rất nhiều
        self.drv_city = list(rng.choice(len(CITIES), size=n_d, p=CITY_P))
        self.drv_dev = list(range(n_c, n_c + n_d))
        self.drv_w = rng.lognormal(0.0, 0.6, size=n_d)
        self.n_dev = n_c + n_d
        self.n_normal_cust = n_c
        # 3 % khách dùng chung thiết bị theo cặp (gia đình, máy cũ), cùng thành phố nếu có thể.
        share = rng.choice(n_c, size=int(n_c * 0.03) // 2 * 2, replace=False)
        share = share[np.argsort(np.array(self.cust_city)[share], kind="stable")]
        for a, b in zip(share[0::2], share[1::2], strict=True):
            self.cust_dev[b] = self.cust_dev[a]
        # 0,4 % khách ngồi chung một thiết bị theo nhóm 3-6 người (quán cà phê, máy bảng dùng chung).
        group = rng.choice(n_c, size=int(n_c * 0.004), replace=False)
        i = 0
        while i < len(group):
            k = int(rng.integers(3, 7))
            for c in group[i + 1:i + k]:
                self.cust_dev[c] = self.cust_dev[group[i]]
            i += k
        # 8 % khách có tài xế quen: đặt lại nhiều lần, hợp lệ nhưng cùng cặp.
        self.fav = {}
        by_city = {ci: [d for d in range(n_d) if self.drv_city[d] == ci] for ci in range(len(CITIES))}
        for c in rng.choice(n_c, size=int(n_c * 0.08), replace=False):
            pool = by_city[int(self.cust_city[c])]
            if pool:
                self.fav[int(c)] = int(rng.choice(pool))

    def new_device(self) -> int:
        self.n_dev += 1
        return self.n_dev - 1

    def new_customers(self, city: int, device: int | list[int], n: int) -> list[int]:
        out = []
        for i in range(n):
            self.cust_city.append(city)
            self.cust_dev.append(device if isinstance(device, int) else device[i % len(device)])
            out.append(len(self.cust_city) - 1)
        return out

    def drivers_in(self, city: int) -> np.ndarray:
        return np.array([d for d, c in enumerate(self.drv_city) if c == city])


# ---- Các dòng chuyến (trước khi thành event) ----------------------------------------------

_FIELDS = ("kind", "cust", "drv", "city", "t0", "service", "dist", "px", "py", "dx", "dy", "dur",
           "promo", "dev_c", "dev_d", "spoof")


def _rows(**cols: Any) -> dict[str, np.ndarray]:
    """Gói các cột (vô hướng thì lặp lại cho đủ độ dài) thành dict mảng numpy."""
    m = max(len(v) for v in cols.values() if not np.isscalar(v))
    out = {}
    for name in _FIELDS:
        v = cols.get(name, -1 if name in ("dev_c", "dev_d") else 0)
        out[name] = np.full(m, v) if np.isscalar(v) else np.asarray(v)
    return out


def _sample_t0(rng: np.random.Generator, m: int) -> np.ndarray:
    """m thời điểm đặt chuyến (giây từ START), ngày đều, giờ theo đỉnh sáng chiều."""
    day = rng.integers(0, DAYS, size=m)
    hour = rng.choice(24, size=m, p=HOUR_P)
    return day * DAY_S + hour * 3600 + rng.random(m) * 3600


def _disc(rng: np.random.Generator, m: int, radius: float) -> tuple[np.ndarray, np.ndarray]:
    r = radius * np.sqrt(rng.random(m))
    th = rng.random(m) * 2 * math.pi
    return r * np.cos(th), r * np.sin(th)


def _shape(rng: np.random.Generator, m: int, radius: Any) -> tuple[np.ndarray, ...]:
    """Hình dạng của một chuyến bình thường: quãng đường khai báo, thời lượng, điểm đón và điểm đến (km, tọa độ cục bộ).

    Mọi loại gian lận dùng lại hàm này rồi chỉ ghi đè đúng cái chúng làm giả, để các cột còn lại
    không tự tố cáo người gian lận.
    """
    dist = np.clip(np.exp(rng.normal(math.log(4.5), 0.65, m)), 0.6, 28.0)
    detour = rng.random(m) < 0.06  # đường vòng thật: đường chim bay ngắn hơn nhiều
    ratio = np.where(detour, rng.uniform(0.15, 0.45, m), rng.uniform(0.45, 0.97, m))
    speed = np.clip(rng.normal(26, 5, m), 12, 45)
    dur = (dist / speed * 3600 + rng.uniform(60, 240, m)).astype(np.int64)
    px, py = _disc(rng, m, radius)
    th = rng.random(m) * 2 * math.pi
    straight = dist * ratio
    return dist, dur, px, py, px + straight * np.cos(th), py + straight * np.sin(th)


def _normal_trips(rng: np.random.Generator, w: _World, m: int) -> dict[str, np.ndarray]:
    city = rng.choice(len(CITIES), size=m, p=CITY_P)
    cust = np.empty(m, dtype=np.int64)
    drv = np.empty(m, dtype=np.int64)
    cust_city = np.array(w.cust_city[: w.n_normal_cust])
    drv_city = np.array(w.drv_city)
    for ci in range(len(CITIES)):
        sel = np.flatnonzero(city == ci)
        for pool_city, weights, target in ((cust_city, w.cust_w, cust), (drv_city, w.drv_w, drv)):
            idx = np.flatnonzero(pool_city == ci)
            cum = np.cumsum(weights[idx])
            target[sel] = idx[np.searchsorted(cum, rng.random(len(sel)) * cum[-1])]
    # khách có tài xế quen: 55 % số chuyến của họ đi với tài xế đó
    for i in np.flatnonzero(np.isin(cust, list(w.fav))):
        if rng.random() < 0.55:
            drv[i] = w.fav[int(cust[i])]
    dist, dur, px, py, dx, dy = _shape(rng, m, np.array([c[4] for c in CITIES])[city])
    # 0,25 % chuyến mà tài xế tự đặt xe bằng điện thoại của mình: cùng thiết bị, vô hại
    dev_c = np.where(rng.random(m) < 0.0025, -2, -1)  # -2: dùng thiết bị của tài xế, điền ở bước sau
    spoof = np.zeros(m, dtype=np.int64)
    glitch = rng.random(m)  # GPS cũ/lệch của chuyến sạch: dương tính giả hợp lý
    spoof[glitch < 0.0055] = 5  # lệch 0,5 đến 3 km (GPS nhảy cóc giữa các trạm phát)
    spoof[glitch < 0.0015] = 3  # lệch 3 đến 30 km
    spoof[glitch < 0.0002] = 4  # lệch hàng trăm km (GPS nhớ nhầm thành phố)
    return _rows(
        kind=0, cust=cust, drv=drv, city=city, t0=_sample_t0(rng, m),
        service=rng.choice(3, size=m, p=SERVICE_P), dist=dist, px=px, py=py, dx=dx, dy=dy, dur=dur,
        promo=np.zeros(m, dtype=np.int64), dev_c=dev_c, dev_d=np.full(m, -1), spoof=spoof,
    )


def _group_bookings(rng: np.random.Generator, w: _World, base: dict[str, np.ndarray], n_groups: int, ks: np.ndarray) -> dict[str, np.ndarray]:
    """Đặt xe cho cả nhóm: cùng một khách, cùng điện thoại, thêm 1-3 chuyến trong vài phút với tài xế khác.

    Chuyến sạch nhưng nhìn từ ngoài giống một đợt dồn dập: dương tính giả hợp lý cho đặc trưng đếm chuyến.
    """
    seeds = np.repeat(rng.choice(len(base["kind"]), size=n_groups, replace=False), ks)
    out = {k: v[seeds].copy() for k, v in base.items()}
    m = len(seeds)
    out["t0"] = out["t0"] + rng.uniform(5, 300, m)
    drv_city = np.array(w.drv_city)
    for ci in range(len(CITIES)):
        sel = np.flatnonzero(out["city"] == ci)
        idx = np.flatnonzero(drv_city == ci)
        out["drv"][sel] = rng.choice(idx, size=len(sel))
    for key in ("px", "py"):
        out[key] = out[key] + rng.normal(0, 0.05, m)
    out["dx"] = out["dx"] + rng.normal(0, 0.05, m)
    out["dy"] = out["dy"] + rng.normal(0, 0.05, m)
    out["spoof"] = np.zeros(m, dtype=np.int64)
    out["promo"] = np.zeros(m, dtype=np.int64)
    out["dev_c"] = np.full(m, -1)
    out["dev_d"] = np.full(m, -1)
    return out


def _ghost_trips(rng: np.random.Generator, w: _World, total: int) -> list[dict[str, np.ndarray]]:
    """Chuyến ảo: một người điều khiển cả tài xế lẫn các tài khoản khách mới.

    Ba kiểu người làm: 35 % "lẻ" (mỗi chuyến một tài khoản, một điện thoại, cách nhau nhiều ngày, chỉ còn dấu vết
    trên chính chuyến); 35 % dùng chung một điện thoại cho 2-5 tài khoản; 30 % mỗi tài khoản một điện thoại.
    Đợt dồn dập có 3-9 chuyến, một nửa sát nhau (vài phút), một nửa thưa (10 đến 60 phút).
    """
    out, made = [], 0
    while made < total:
        city = int(rng.choice(len(CITIES), p=CITY_P))
        radius = CITIES[city][4]
        mode = rng.random()
        drivers = w.drivers_in(city)
        if mode < 0.35:
            k = min(int(rng.integers(1, 4)), total - made)
            for _ in range(k):
                dist, dur, px, py, dx, dy = _shape(rng, 1, radius)
                obvious = rng.random() < 0.4
                if obvious:
                    dist, dur = rng.uniform(3, 14, 1), rng.integers(40, 240, 1)
                    dx, dy = px + rng.normal(0, 0.03, 1), py + rng.normal(0, 0.03, 1)
                else:
                    dist = dist * rng.uniform(1.2, 1.9)
                acc = w.new_customers(city, w.new_device(), 1)
                out.append(_rows(kind=1, cust=acc, drv=int(rng.choice(drivers)), city=city,
                                 t0=np.array([rng.uniform(0, (DAYS - 1) * DAY_S)]), service=rng.choice(3, size=1, p=SERVICE_P),
                                 dist=dist, px=px, py=py, dx=dx, dy=dy, dur=dur))
                made += 1
            continue
        drv = int(rng.choice(drivers))
        n_acc = int(rng.integers(2, 6))
        shared = mode < 0.70
        dev = w.new_device()
        accounts = w.new_customers(city, dev if shared else [w.new_device() for _ in range(n_acc)], n_acc)
        for _ in range(int(rng.integers(1, 4))):
            k = min(int(rng.integers(3, 10)), total - made)
            if k <= 0:
                break
            night = rng.random() < 0.35
            hour = int(rng.integers(0, 6)) if night else int(rng.integers(0, 24))
            t0 = int(rng.integers(0, DAYS - 1)) * DAY_S + hour * 3600 + rng.random() * 3600
            tight = rng.random() < 0.5
            t = t0 + np.cumsum(rng.uniform(90, 600, k) if tight else rng.uniform(600, 3600, k))
            dist, dur, px, py, dx, dy = _shape(rng, k, radius)
            obvious = rng.random(k) < 0.3  # chuyến "không di chuyển": điểm đến trùng điểm đón, quãng đường phình ra
            dist = np.where(obvious, rng.uniform(3, 14, k), dist * rng.uniform(1.2, 1.9, k))
            dur = np.where(obvious, rng.integers(40, 240, k), dur)
            dx = np.where(obvious, px + rng.normal(0, 0.03, k), dx)
            dy = np.where(obvious, py + rng.normal(0, 0.03, k), dy)
            out.append(_rows(
                kind=1, cust=rng.choice(accounts, size=k), drv=drv, city=city, t0=t,
                service=rng.choice(3, size=k, p=SERVICE_P), dist=dist, px=px, py=py, dx=dx, dy=dy, dur=dur,
                dev_d=np.where((rng.random(k) < 0.4) & shared, dev, -1),  # 40 %: tài xế dùng chính điện thoại đó
            ))
            made += k
    return out


def _promo_trips(rng: np.random.Generator, w: _World, total: int) -> list[dict[str, np.ndarray]]:
    """Lạm dụng khuyến mãi: một người mở nhiều tài khoản mới, mỗi tài khoản dùng NEWUSER50.

    35 % dùng một điện thoại cho 5-12 tài khoản; 35 % dùng 2-4 điện thoại cho 4-10 tài khoản;
    30 % mỗi tài khoản một điện thoại riêng (3-8 tài khoản), không để lại dấu thiết bị dùng chung.
    Mỗi tài khoản dùng mã đúng một lần, chuyến đi như chuyến bình thường, chỉ khác mã khuyến mãi.
    """
    out, made = [], 0
    code = PROMO_NAMES.index("NEWUSER50")
    while made < total:
        city = int(rng.choice(len(CITIES), p=CITY_P))
        mode = rng.random()
        if mode < 0.35:
            n_acc, n_dev = int(rng.integers(5, 13)), 1
        elif mode < 0.70:
            n_acc, n_dev = int(rng.integers(4, 11)), int(rng.integers(2, 5))
        else:
            n_acc = int(rng.integers(3, 9))
            n_dev = n_acc
        accounts = w.new_customers(city, [w.new_device() for _ in range(n_dev)], n_acc)
        span = rng.uniform(4, 12)  # cả nhóm hoạt động trong một cửa sổ vài ngày
        day0 = rng.uniform(0, DAYS - span)
        drivers = w.drivers_in(city)
        for acc in accounts:
            if made >= total:
                break
            k = 1  # mỗi tài khoản chỉ dùng mã đúng một lần, như khách mới hợp lệ
            t = (day0 + rng.uniform(0, span, k)) * DAY_S + rng.choice(24, size=k, p=HOUR_P) * 3600
            dist, dur, px, py, dx, dy = _shape(rng, k, CITIES[city][4])
            out.append(_rows(
                kind=3, cust=acc, drv=rng.choice(drivers, size=k), city=city, t0=t,
                service=rng.choice(3, size=k, p=SERVICE_P), dist=dist, px=px, py=py, dx=dx, dy=dy, dur=dur,
                promo=np.full(k, code),
            ))
            made += k
    return out


def _collusion_trips(rng: np.random.Generator, w: _World, total: int) -> list[dict[str, np.ndarray]]:
    """Cấu kết: 2-3 tài xế và 3-6 tài khoản khách mới chạy qua lại trong 2 đến 5 tuần để nhận thưởng.

    Nửa số chuyến là chuyến ngắn 0,8-3 km (chạy cho đủ chỉ tiêu thưởng), nửa còn lại giống chuyến thường.
    """
    out, made = [], 0
    while made < total:
        city = int(rng.choice(len(CITIES), p=CITY_P))
        drivers = rng.choice(w.drivers_in(city), size=int(rng.integers(2, 4)), replace=False)
        devs = [w.new_device(), w.new_device()]
        customers = w.new_customers(city, devs, int(rng.integers(3, 7)))
        k = min(int(rng.integers(30, 61)), total - made)
        span = rng.uniform(14, 35)
        day0 = rng.uniform(0, DAYS - span)
        t = (day0 + rng.uniform(0, span, k)) * DAY_S + rng.choice(24, size=k, p=HOUR_P) * 3600
        dist, dur, px, py, dx, dy = _shape(rng, k, CITIES[city][4])
        short = rng.random(k) < 0.5
        s_dist = rng.uniform(0.8, 3.0, k)
        s_dur = (s_dist / rng.uniform(18, 34, k) * 3600 + rng.uniform(60, 240, k)).astype(np.int64)
        ratio = rng.uniform(0.6, 0.9, k)
        th = rng.random(k) * 2 * math.pi
        dx = np.where(short, px + s_dist * ratio * np.cos(th), dx)
        dy = np.where(short, py + s_dist * ratio * np.sin(th), dy)
        dist, dur = np.where(short, s_dist, dist), np.where(short, s_dur, dur)
        dev_c = np.where(rng.random(k) < 0.25, -2, -1)  # 25 %: khách đặt bằng điện thoại của chính tài xế
        out.append(_rows(
            kind=4, cust=rng.choice(customers, size=k), drv=rng.choice(drivers, size=k), city=city, t0=t,
            service=rng.choice(3, size=k, p=SERVICE_P), dist=dist, px=px, py=py, dx=dx, dy=dy, dur=dur, dev_c=dev_c,
        ))
        made += k
    return out


def _spoof_trips(rng: np.random.Generator, w: _World, total: int) -> list[dict[str, np.ndarray]]:
    """GPS giả: tài xế bật ứng dụng giả vị trí theo phiên 4-9 chuyến liên tiếp; điểm bắt đầu bị dịch."""
    out, made = [], 0
    while made < total:
        city = int(rng.choice(len(CITIES), p=CITY_P))
        drv = int(rng.choice(w.drivers_in(city)))
        cust_pool = np.flatnonzero(np.array(w.cust_city[: w.n_normal_cust]) == city)
        for _ in range(int(rng.integers(2, 5))):
            k = min(int(rng.integers(4, 10)), total - made)
            if k <= 0:
                break
            t = int(rng.integers(0, DAYS - 1)) * DAY_S + int(rng.integers(7, 22)) * 3600 + np.cumsum(rng.uniform(600, 2400, k))
            dist, dur, px, py, dx, dy = _shape(rng, k, CITIES[city][4])
            out.append(_rows(
                kind=2, cust=rng.choice(cust_pool, size=k), drv=drv, city=city, t0=t,
                service=rng.choice(3, size=k, p=SERVICE_P), dist=dist, px=px, py=py, dx=dx, dy=dy, dur=dur,
                spoof=np.where(rng.random(k) < 0.4, 2, 1),  # 2 = dịch xa (cả trăm km), 1 = dịch gần (dưới 6 km)
            ))
            made += k
    return out


# ---- Ghép thành bảng event ----------------------------------------------------------------


def _local_to_latlng(city: np.ndarray, x: np.ndarray, y: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    lat0 = np.array([c[1] for c in CITIES])[city]
    lng0 = np.array([c[2] for c in CITIES])[city]
    return lat0 + y / KM_PER_DEG, lng0 + x / (KM_PER_DEG * np.cos(np.radians(lat0)))


def _shift(lat: np.ndarray, lng: np.ndarray, dx_km: np.ndarray, dy_km: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    return lat + dy_km / KM_PER_DEG, lng + dx_km / (KM_PER_DEG * np.cos(np.radians(lat)))


def _driver_schedule(t: np.ndarray, drv: np.ndarray, dur: np.ndarray, travel_s: np.ndarray) -> np.ndarray:
    """Giờ bắt đầu chuyến sao cho mỗi tài xế kịp chạy hết chuyến trước và di chuyển tới điểm đón (30 km/h).

    s_k = max(t_k, s_{k-1} + dur_{k-1} + travel_k + 20). Đặt C_k = tổng khoảng cách tối thiểu tích lũy
    thì s_k - C_k = max(t_k - C_k, s_{k-1} - C_{k-1}): một phép max tích lũy theo từng tài xế.
    `t`, `drv`, `dur`, `travel_s` đã sắp theo (drv, t).
    """
    first = np.r_[True, drv[1:] != drv[:-1]]
    gap = np.r_[0.0, dur[:-1] + travel_s[1:] + 20.0]
    gap[first] = 0.0
    csum = np.cumsum(gap)
    base = np.maximum.accumulate(np.where(first, csum, 0.0))
    c = csum - base
    gid = np.cumsum(first)
    big = 1e8
    return c + np.maximum.accumulate(t - c + gid * big) - gid * big


def _ids(prefix: str, perm_ids: np.ndarray, width: int) -> np.ndarray:
    return np.array([f"{prefix}-{int(v) + 1:0{width}d}" for v in perm_ids], dtype=object)


def _uuids(rng: np.random.Generator, n: int) -> np.ndarray:
    hi = rng.integers(0, 2**63, size=n, dtype=np.int64).tolist()
    lo = rng.integers(0, 2**63, size=n, dtype=np.int64).tolist()
    out = np.empty(n, dtype=object)
    for i, (a, b) in enumerate(zip(hi, lo, strict=True)):
        h = f"{a:016x}{b:016x}"
        out[i] = f"{h[:8]}-{h[8:12]}-4{h[13:16]}-{'89ab'[int(h[16], 16) & 3]}{h[17:20]}-{h[20:]}"
    return out


def _us(seconds: np.ndarray) -> pl.Series:
    return pl.Series(START_US + np.round(seconds * 1e6).astype(np.int64)).cast(pl.Datetime("us", "UTC"))


def make_events(seed: int = 42, n_trips: int = 200_000) -> FraudData:
    """Sinh n_trips chuyến, trả về (events, labels). Cùng seed và n_trips cho đúng cùng dữ liệu."""
    rng = np.random.default_rng(seed)
    n_fraud = round(FRAUD_RATE * n_trips)
    n_by = {k: round(n_fraud * v) for k, v in FRAUD_SPLIT.items()}
    n_by["ghost_trip"] += n_fraud - sum(n_by.values())
    w = _World(rng, n_trips)
    n_normal = n_trips - n_fraud
    n_groups = int(0.002 * n_normal)  # 0,2 % chuyến sạch là đặt xe cho cả nhóm, thêm 1-3 chuyến mỗi nhóm
    ks = rng.integers(1, 4, size=n_groups)
    base = _normal_trips(rng, w, n_normal - int(ks.sum()))
    parts = [base, _group_bookings(rng, w, base, n_groups, ks)]
    parts += _ghost_trips(rng, w, n_by["ghost_trip"])
    parts += _promo_trips(rng, w, n_by["promo_abuse"])
    parts += _collusion_trips(rng, w, n_by["collusion"])
    parts += _spoof_trips(rng, w, n_by["gps_spoof"])
    t = {k: np.concatenate([p[k] for p in parts]) for k in _FIELDS}
    m = len(t["kind"])

    # Khuyến mãi của chuyến sạch: chuyến đầu của khách có 15 % dùng NEWUSER50; còn lại 9 % dùng mã thường.
    normal = t["kind"] == 0
    order = np.argsort(t["t0"], kind="stable")
    seen: set[int] = set()
    first_trip = np.zeros(m, dtype=bool)
    for i in order:
        if normal[i] and int(t["cust"][i]) not in seen:
            first_trip[i] = True
            seen.add(int(t["cust"][i]))
    u = rng.random(m)
    promo = t["promo"].copy()
    promo[normal & first_trip & (u < 0.15)] = PROMO_NAMES.index("NEWUSER50")
    other = normal & ~first_trip & (u < 0.09)
    food = t["service"] == SERVICES.index("food")
    pick = rng.random(m)
    promo[other & food & (pick < 0.6)] = PROMO_NAMES.index("FOOD15")
    promo[other & ~(food & (pick < 0.6))] = np.where(pick[other & ~(food & (pick < 0.6))] < 0.5,
                                                    PROMO_NAMES.index("WEEKEND10"), PROMO_NAMES.index("COMEBACK20"))

    # Tọa độ thật rồi tọa độ GPS quan sát được.
    plat, plng = _local_to_latlng(t["city"], t["px"], t["py"])
    elat, elng = _local_to_latlng(t["city"], t["dx"], t["dy"])
    th = rng.random(m) * 2 * math.pi
    jitter = rng.normal(0, 0.025, size=(2, m))  # sai số GPS thường: khoảng 25 m
    off = np.zeros(m)
    off = np.where(t["spoof"] == 1, rng.uniform(0.3, 6.0, m), off)
    off = np.where(t["spoof"] == 5, rng.uniform(0.5, 3.0, m), off)
    off = np.where(t["spoof"] == 2, rng.uniform(100, 700, m), off)
    off = np.where(t["spoof"] == 3, rng.uniform(3.0, 30.0, m), off)
    off = np.where(t["spoof"] == 4, rng.uniform(100, 800, m), off)
    slat, slng = _shift(plat, plng, jitter[0] + off * np.cos(th), jitter[1] + off * np.sin(th))
    elat, elng = _shift(elat, elng, rng.normal(0, 0.025, m), rng.normal(0, 0.025, m))

    # Lịch của tài xế theo vị trí THẬT (điểm đến chuyến trước -> điểm đón chuyến này).
    wait = rng.uniform(90, 480, m)
    drv_order = np.lexsort((t["t0"], t["drv"]))
    d_s = t["drv"][drv_order]
    travel = np.r_[0.0, haversine_km(elat[drv_order][:-1], elng[drv_order][:-1], plat[drv_order][1:], plng[drv_order][1:])]
    travel_s = travel / 30.0 * 3600.0
    start_sorted = _driver_schedule((t["t0"] + wait)[drv_order], d_s, t["dur"][drv_order].astype(float), travel_s)
    started = np.empty(m)
    started[drv_order] = start_sorted
    booked = t["t0"] + rng.random(m)  # lẻ giây để tránh trùng timestamp
    completed = started + t["dur"]
    paid = completed + rng.uniform(2, 30, m)

    # Số tiền
    dist = np.round(t["dist"], 2)
    fare = fare_vnd(dist)
    pct = np.array([PROMOS[n][0] for n in PROMO_NAMES])[promo]
    cap = np.array([PROMOS[n][1] for n in PROMO_NAMES])[promo]
    discount = np.minimum(cap, np.floor(fare * pct)).astype(np.int64)
    amount = fare - discount

    # Đánh số chuyến theo giờ đặt; xáo ID khách, tài xế, thiết bị.
    rank = np.empty(m, dtype=np.int64)
    rank[np.argsort(booked, kind="stable")] = np.arange(m)
    trip_id = _ids("TN", rank, 7)
    n_cust, n_drv = len(w.cust_city), len(w.drv_city)
    cust_perm = rng.permutation(n_cust)
    drv_perm = rng.permutation(n_drv)
    dev_perm = rng.permutation(w.n_dev)
    width_c, width_d = max(5, len(str(n_cust))), max(5, len(str(w.n_dev)))
    customer_id = _ids("CU", cust_perm[t["cust"]], width_c)
    driver_id = _ids("DR", drv_perm[t["drv"]], max(5, len(str(n_drv))))
    cust_dev = np.array(w.cust_dev)[t["cust"]]
    drv_dev = np.array(w.drv_dev)[t["drv"]]
    dev_c = np.where(t["dev_c"] == -2, drv_dev, np.where(t["dev_c"] >= 0, t["dev_c"], cust_dev))
    dev_d = np.where(t["dev_d"] >= 0, t["dev_d"], drv_dev)
    device_id = _ids("DEV", dev_perm[dev_c], width_d)
    driver_device_id = _ids("DEV", dev_perm[dev_d], width_d)

    city_name = np.array([c[0] for c in CITIES], dtype=object)[t["city"]]
    service = np.array(SERVICES, dtype=object)[t["service"]]
    promo_code = np.array([n or None for n in PROMO_NAMES], dtype=object)[promo]
    ev_ids = _uuids(rng, 3 * m)

    def frame(kind: str, seconds: np.ndarray, off_i: int, cols: dict[str, Any]) -> pl.DataFrame:
        base = {
            "event_id": pl.Series(ev_ids[off_i * m:(off_i + 1) * m].tolist(), dtype=pl.String),
            "event_type": pl.Series([kind] * m, dtype=pl.String),
            "occurred_at": _us(seconds), "version": pl.Series([1] * m, dtype=pl.Int8),
            "trip_id": pl.Series(trip_id.tolist(), dtype=pl.String),
            "customer_id": pl.Series(customer_id.tolist(), dtype=pl.String),
        }
        return pl.DataFrame({**base, **cols})

    booking = frame("BookingCreated", booked, 0, {
        "device_id": pl.Series(device_id.tolist(), dtype=pl.String),
        "service": pl.Series(service.tolist(), dtype=pl.String),
        "city": pl.Series(city_name.tolist(), dtype=pl.String),
        "pickup_lat": plat, "pickup_lng": plng,
        "promo_code": pl.Series(promo_code.tolist(), dtype=pl.String),
    })
    done = frame("TripCompleted", completed, 1, {
        "driver_id": pl.Series(driver_id.tolist(), dtype=pl.String),
        "driver_device_id": pl.Series(driver_device_id.tolist(), dtype=pl.String),
        "start_lat": slat, "start_lng": slng, "end_lat": elat, "end_lng": elng,
        "distance_km": dist, "duration_s": pl.Series(t["dur"].astype(np.int32)),
    })
    pay = frame("PaymentCaptured", paid, 2, {"amount_vnd": amount, "discount_vnd": discount})
    events = (
        pl.concat([booking, done, pay], how="diagonal")
        .sort("occurred_at", maintain_order=True)
        .select(EVENT_COLUMNS)
        .cast(EVENT_SCHEMA)
    )

    # Nhãn và độ trễ
    kind = np.array(FRAUD_TYPES, dtype=object)[t["kind"]]
    delay_days = np.full(m, CLEAN_CONFIRM_DAYS)
    for ki, name in enumerate(FRAUD_TYPES):
        if ki == 0:
            continue
        sel = t["kind"] == ki
        med, sigma = LABEL_DELAY[name]
        delay_days[sel] = np.minimum(55.0, med * np.exp(rng.normal(0, sigma, int(sel.sum()))))
    labels = pl.DataFrame({
        "trip_id": pl.Series(trip_id.tolist(), dtype=pl.String),
        "fraud_type": pl.Series(kind.tolist(), dtype=pl.String),
        "is_fraud": pl.Series((t["kind"] != 0).tolist(), dtype=pl.Boolean),
        "label_known_at": _us(paid + delay_days * DAY_S),
    }).sort("trip_id").cast(LABEL_SCHEMA)
    return FraudData(events, labels)


EVENT_SCHEMA = {
    "event_id": pl.String, "event_type": pl.String, "occurred_at": pl.Datetime("us", "UTC"), "version": pl.Int8,
    "trip_id": pl.String, "customer_id": pl.String, "device_id": pl.String, "service": pl.String, "city": pl.String,
    "pickup_lat": pl.Float64, "pickup_lng": pl.Float64, "promo_code": pl.String,
    "driver_id": pl.String, "driver_device_id": pl.String,
    "start_lat": pl.Float64, "start_lng": pl.Float64, "end_lat": pl.Float64, "end_lng": pl.Float64,
    "distance_km": pl.Float64, "duration_s": pl.Int32,
    "amount_vnd": pl.Int64, "discount_vnd": pl.Int64,
}
EVENT_COLUMNS = list(EVENT_SCHEMA)
LABEL_SCHEMA = {
    "trip_id": pl.String, "fraud_type": pl.String, "is_fraud": pl.Boolean,
    "label_known_at": pl.Datetime("us", "UTC"),
}


# ---- Envelope Kafka ------------------------------------------------------------------------


def iter_envelopes(events: pl.DataFrame) -> Iterator[tuple[str, dict[str, Any]]]:
    """(topic, envelope) cho từng dòng events, theo thứ tự thời gian.

    Envelope: {"event_id", "event_type", "occurred_at" (ISO UTC), "version": 1, "payload": {...}};
    payload chỉ có các trường không null của loại sự kiện đó.
    """
    payload_cols = [c for c in EVENT_COLUMNS if c not in ("event_id", "event_type", "occurred_at", "version")]
    for row in events.iter_rows(named=True):
        payload = {c: row[c] for c in payload_cols if row[c] is not None}
        yield TOPICS[row["event_type"]], {
            "event_id": row["event_id"],
            "event_type": row["event_type"],
            "occurred_at": row["occurred_at"].astimezone(UTC).isoformat(),
            "version": row["version"],
            "payload": payload,
        }


def encode_envelope(envelope: dict[str, Any]) -> bytes:
    return json.dumps(envelope, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
