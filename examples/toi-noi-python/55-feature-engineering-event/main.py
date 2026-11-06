"""Bài 55 - Feature engineering từ event Kafka: velocity, thiết bị, GPS và leakage.

Chạy từ gốc repository:
    uv run --project examples/toi-noi-python/55-feature-engineering-event \
        examples/toi-noi-python/55-feature-engineering-event/main.py
    ... main.py --verify    # kiểm các bất biến in trong bài, in OK n/n

Cần Kafka trong hạ tầng dùng chung (localhost:9094, ghi đè bằng TOINOI_KAFKA_BOOTSTRAP):
    docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Topic của lab có tiền tố `lab55.` và bị xóa khi chạy xong. Dữ liệu là TỔNG HỢP (synthetic_fraud.py, giống bài 54);
mọi con số chỉ nói về bộ này.
"""

import asyncio
import math
import os
import sys
import time
from datetime import timedelta
from pathlib import Path
from zoneinfo import ZoneInfo

import numpy as np
import polars as pl
from sklearn.ensemble import HistGradientBoostingClassifier
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import average_precision_score, roc_auc_score
from sklearn.pipeline import Pipeline
from sklearn.preprocessing import StandardScaler

import stream
from features import (
    BASE_FEATURES,
    FEATURES,
    GPS_FEATURES,
    IMPOSSIBLE_KMH,
    WINDOW_FEATURES,
    add_device_fraud_history,
    add_features,
    add_future_window,
    add_global_aggregates,
    add_global_aggregates_serving,
    add_hour_bucket,
    day_of,
    haversine_expr,
    trip_table,
)
from online import ONLINE_FEATURES, OnlineFeatures
from synthetic_fraud import FRAUD_TYPES, START, haversine_km, iter_envelopes, make_events

SEED = 42
N_TRIPS = 200_000
TRAIN_END_DAY = 30  # như bài 54: train < ngày 30, valid từ ngày 40
VALID_START_DAY = 40
STREAM_DAYS = 10  # phần event đưa qua Kafka và qua bộ tính online
OUT = Path(__file__).parent / "out"
QUIET = False


def say(*parts: object) -> None:
    if not QUIET:
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


# ---- [1] Từ event đến bảng chuyến ----------------------------------------------------------


def step_trips(c: Checker):
    say("\n[1] Từ event đến bảng chuyến")
    t0 = time.perf_counter()
    data = make_events(seed=SEED, n_trips=N_TRIPS)
    say(f"make_events(seed={SEED}, n_trips={N_TRIPS:,}): {data.events.height:,} event trong {time.perf_counter() - t0:.1f} giây")
    t0 = time.perf_counter()
    trips = trip_table(data.events)
    say(f"trip_table: nối 3 loại event theo trip_id -> {trips.height:,} chuyến x {trips.width} cột trong {time.perf_counter() - t0:.2f} giây")
    say("cột:", ", ".join(trips.columns))
    c.check("mỗi chuyến một dòng đủ ba event", trips.height == N_TRIPS and trips["trip_id"].n_unique() == N_TRIPS)
    c.check("bảng chuyến sắp theo paid_at", bool(trips["paid_at"].is_sorted()))
    return data, trips


# ---- [2] Cửa sổ thời gian ------------------------------------------------------------------


def step_windows(trips: pl.DataFrame, labels: pl.DataFrame, c: Checker) -> pl.DataFrame:
    say("\n[2] Đặc trưng cửa sổ thời gian (polars rolling, closed=\"left\")")
    t0 = time.perf_counter()
    feat = add_features(trips)
    took = time.perf_counter() - t0
    say(f"add_features: {len(FEATURES)} đặc trưng cho {feat.height:,} chuyến trong {took:.2f} giây; load average 1/5/15 phút: {load()}")
    full = feat.join(labels, on="trip_id").sort("paid_at")
    cols = ["dev_trips_1h", "drv_trips_1h", "dev_customers_7d", "pair_trips_7d", "dev_newuser_7d", "cust_promo_share_30d"]
    table = full.group_by("fraud_type").agg([pl.col(x).mean() for x in cols]).sort("fraud_type")
    say(f"{'loại':12s}" + "".join(f"{x:>21s}" for x in cols))
    for row in table.iter_rows(named=True):
        say(f"{row['fraud_type']:12s}" + "".join(f"{row[x]:21.3f}" for x in cols))

    ghost_dev = (full.filter(pl.col("fraud_type") == "ghost_trip").group_by("device_id").len().sort("len", descending=True)["device_id"][0])
    burst = full.filter(pl.col("device_id") == ghost_dev).sort("paid_at").head(8)
    say(f"\nmột thiết bị gian lận ({ghost_dev}), 8 chuyến đầu theo paid_at (giờ Việt Nam):")
    say(f"{'trip_id':12s}{'paid_at':>21s}{'dev_trips_1h':>14s}{'dev_customers_24h':>19s}{'pair_trips_7d':>15s}{'fraud_type':>13s}")
    for r in burst.iter_rows(named=True):
        local = r["paid_at"].astimezone(ZoneInfo("Asia/Ho_Chi_Minh")).strftime("%d/%m %H:%M:%S")
        say(f"{r['trip_id']:12s}{local:>21s}{r['dev_trips_1h']:14d}{r['dev_customers_24h']:19d}{r['pair_trips_7d']:15d}{r['fraud_type']:>13s}")

    by = {r["fraud_type"]: r for r in table.iter_rows(named=True)}
    c.check("chuyến của thiết bị gian lận dồn dập hơn hẳn chuyến sạch", by["ghost_trip"]["dev_trips_1h"] > 20 * by["none"]["dev_trips_1h"], by["ghost_trip"])
    c.check("promo_abuse: nhiều khách trên một thiết bị trong 7 ngày", by["promo_abuse"]["dev_customers_7d"] > 2.5 and by["none"]["dev_customers_7d"] < 1.1)
    c.check("collusion: cặp khách-tài xế lặp lại", by["collusion"]["pair_trips_7d"] > 10 * by["none"]["pair_trips_7d"], by["collusion"])
    first = full.group_by("device_id").agg(pl.col("dev_trips_1h").first())
    c.check("chuyến đầu tiên của mỗi thiết bị có dev_trips_1h = 0", int(first["dev_trips_1h"].max()) == 0)
    try:
        add_features(trips.sort("booked_at"))
        refused = False
    except ValueError:
        refused = True
    c.check("add_features từ chối bảng chưa sắp theo paid_at (rolling().over() sẽ sai âm thầm)", refused)
    # Đo cái giá nếu không có hàng rào: cùng biểu thức rolling().over() trên bảng sắp theo booked_at, polars không báo lỗi.
    expr = pl.len().rolling("paid_at", period="1h", closed="left").over("device_id")
    wrong_unsorted = (
        trips.sort("booked_at").with_columns(n=expr).select("trip_id", "n")
        .join(trips.sort("paid_at").with_columns(n=expr).select("trip_id", "n"), on="trip_id", suffix="_ok")
        .filter(pl.col("n") != pl.col("n_ok")).height
    )
    say(f"đối chứng: dev_trips_1h tính trên bảng sắp theo booked_at, không báo lỗi gì, lệch {wrong_unsorted:,}/{trips.height:,} chuyến")
    c.check("rolling().over() trên bảng chưa sắp cho giá trị sai mà không báo lỗi", wrong_unsorted > 0, wrong_unsorted)
    c.check("không đặc trưng nào bị null", all(full[x].null_count() == 0 for x in FEATURES), [x for x in FEATURES if full[x].null_count()])
    c.check("cửa sổ [t - 1h, t) không đếm chính dòng hiện tại", int(full["dev_trips_1h"].min()) == 0 and int(full["drv_trips_1h"].min()) == 0)
    return full


# ---- [3] Haversine và GPS -----------------------------------------------------------------


def step_gps(full: pl.DataFrame, c: Checker) -> None:
    say("\n[3] Haversine và tốc độ phi lý")
    hn_hcm = haversine_km(21.0285, 105.8542, 10.7769, 106.7009)
    say(f"Hà Nội - TP.HCM (tâm thành phố, đường chim bay): {hn_hcm:.0f} km; tại 120 km/h mất {hn_hcm / IMPOSSIBLE_KMH:.1f} giờ")
    expr_km = full.select(haversine_expr("start_lat", "start_lng", "end_lat", "end_lng").alias("k"))["k"].to_numpy()
    np_km = haversine_km(full["start_lat"].to_numpy(), full["start_lng"].to_numpy(), full["end_lat"].to_numpy(), full["end_lng"].to_numpy())
    diff = float(np.max(np.abs(expr_km - np_km)))
    say(f"biểu thức polars so với numpy trên {full.height:,} chuyến: lệch lớn nhất {diff:.2e} km")
    say(f"{'loại':12s}{'chuyến':>9s}{'speed_kmh>120':>15s}{'jump>120 km/h':>15s}{'một trong hai':>15s}")
    rates = {}
    for name in FRAUD_TYPES:
        sub = full.filter(pl.col("fraud_type") == name)
        either = ((sub["impossible_speed"] == 1) | (sub["impossible_jump"] == 1)).mean()
        rates[name] = (sub["impossible_speed"].mean(), sub["impossible_jump"].mean(), either)
        say(f"{name:12s}{sub.height:9,d}{rates[name][0]:15.1%}{rates[name][1]:15.1%}{either:15.1%}")
    clean = full.filter(pl.col("fraud_type") == "none")
    fp_n = clean.filter((pl.col("impossible_speed") == 1) | (pl.col("impossible_jump") == 1)).height
    spoof = full.filter(pl.col("fraud_type") == "gps_spoof")
    n_spoof = spoof.height
    caught = spoof.filter((pl.col("impossible_speed") == 1) | (pl.col("impossible_jump") == 1)).height
    say(f"chuyến sạch bị gắn cờ: {fp_n} trên {clean.height:,}; gps_spoof bị gắn cờ: {caught} trên {n_spoof}")
    others = [n for n in FRAUD_TYPES if n not in ("none", "gps_spoof")]
    c.check("biểu thức haversine của polars khớp numpy", diff < 1e-9, diff)
    c.check("Hà Nội - TP.HCM khoảng 1.140 km", 1_100 < hn_hcm < 1_180, hn_hcm)
    c.check("tốc độ phi lý gần như chỉ xuất hiện ở gps_spoof", rates["gps_spoof"][2] > 0.3 and rates["none"][2] < 0.001 and all(rates[n][2] == 0 for n in others), rates)
    c.check("luật tốc độ không bắt hết gps_spoof (loại dịch gần vài km vẫn lọt)", rates["gps_spoof"][2] < 0.8, rates["gps_spoof"])


# ---- [4] Leakage ---------------------------------------------------------------------------


def hgb(random_state: int = 0) -> HistGradientBoostingClassifier:
    """Mặc định của scikit-learn: tối đa 150 vòng; `early_stopping="auto"` bật dừng sớm vì train có hơn 10.000 dòng
    (tách ngẫu nhiên 10 % train làm tập kiểm), nên số vòng thật nằm ở `n_iter_`."""
    return HistGradientBoostingClassifier(max_iter=150, learning_rate=0.1, random_state=random_state)


LEAK_COLS = ["dev_trips_next_1h", "dev_trips_this_hour", "dev_customers_total", "pair_trips_total"]


def leak_frame(events: pl.DataFrame, labels: pl.DataFrame) -> pl.DataFrame:
    """Đặc trưng sạch cộng các đặc trưng rò rỉ, tính từ `events` và `labels` đã cho (labels có thể đã bị cắt)."""
    t = add_global_aggregates(add_hour_bucket(add_future_window(add_features(trip_table(events)))))
    return add_device_fraud_history(t, labels, honest=False)


def step_prefix(events: pl.DataFrame, labels: pl.DataFrame, c: Checker) -> None:
    """Phép thử cắt cụt: đặc trưng của một chuyến tính chỉ từ thông tin có đến lúc nó được thu tiền phải giống tính trên toàn bộ."""
    say("\n[4a] Phép thử cắt cụt: đặc trưng có dùng thông tin tương lai không?")
    full = leak_frame(events, labels).sort("paid_at")
    candidates = full.filter(pl.col("dev_trips_next_1h") >= 1)
    sample = candidates.gather_every(max(1, candidates.height // 40)).head(40)
    clean_diff = dict.fromkeys(FEATURES, 0)
    leak_diff = dict.fromkeys(LEAK_COLS, 0)
    naive_events_only = naive_labels_cut = 0
    for row in sample.iter_rows(named=True):
        cut = events.filter(pl.col("occurred_at") <= row["paid_at"])
        lab_known = labels.filter(pl.col("label_known_at") <= row["paid_at"])
        got = leak_frame(cut, labels).filter(pl.col("trip_id") == row["trip_id"]).row(0, named=True)  # chỉ cắt event
        got_known = leak_frame(cut, lab_known).filter(pl.col("trip_id") == row["trip_id"]).row(0, named=True)  # cắt cả nhãn
        for x in FEATURES:
            if not math.isclose(got[x], row[x], rel_tol=1e-9, abs_tol=1e-9):
                clean_diff[x] += 1
        for x in LEAK_COLS:
            leak_diff[x] += got[x] != row[x]
    # Mẫu thứ hai cho lịch sử gian lận: 40 chuyến thuộc thiết bị đã có chuyến gian lận trước đó (nhãn cuối).
    cand2 = full.filter(pl.col("dev_fraud_naive") > 0)
    sample2 = cand2.gather_every(max(1, cand2.height // 40)).head(40)
    for row in sample2.iter_rows(named=True):
        cut = events.filter(pl.col("occurred_at") <= row["paid_at"])
        lab_known = labels.filter(pl.col("label_known_at") <= row["paid_at"])
        got = leak_frame(cut, labels).filter(pl.col("trip_id") == row["trip_id"]).row(0, named=True)
        got_known = leak_frame(cut, lab_known).filter(pl.col("trip_id") == row["trip_id"]).row(0, named=True)
        naive_events_only += got["dev_fraud_naive"] != row["dev_fraud_naive"]
        naive_labels_cut += got_known["dev_fraud_naive"] != row["dev_fraud_naive"]
    n = sample.height
    bad_clean = {k: v for k, v in clean_diff.items() if v}
    say(f"{n} chuyến mà thiết bị còn chuyến sau trong vòng 1 giờ; mỗi chuyến tính lại từ event có occurred_at <= paid_at của nó")
    say(f"  {len(FEATURES)} đặc trưng sạch khác kết quả tính trên toàn bộ: {bad_clean or 'không đặc trưng nào'}")
    say("  đặc trưng nhìn tương lai khác kết quả: " + ", ".join(f"{x} {leak_diff[x]}/{n}" for x in LEAK_COLS))
    n2 = sample2.height
    say(f"  dev_fraud_naive, trên {n2} chuyến thuộc thiết bị đã có gian lận trước đó: cắt event, giữ nguyên nhãn: khác {naive_events_only}/{n2}; "
        f"cắt cả nhãn theo label_known_at: khác {naive_labels_cut}/{n2}")
    c.check("đặc trưng sạch bất biến khi cắt event ở paid_at", not bad_clean, bad_clean)
    c.check("cửa sổ nhìn trước đổi khi cắt event", leak_diff["dev_trips_next_1h"] == n and leak_diff["dev_trips_this_hour"] > 0, leak_diff)
    c.check("tổng hợp cả bảng đổi khi cắt event", leak_diff["dev_customers_total"] + leak_diff["pair_trips_total"] > 0, leak_diff)
    c.check("chỉ cắt event thì không thấy rò rỉ nhãn; phải cắt cả nhãn", naive_labels_cut > naive_events_only, (naive_events_only, naive_labels_cut))


def lr() -> Pipeline:
    return Pipeline([("scale", StandardScaler()), ("lr", LogisticRegression(max_iter=2000))])


def step_leak_models(full: pl.DataFrame, labels: pl.DataFrame, c: Checker) -> None:
    say("\n[4b] Leakage làm số đo offline lệch với thứ phục vụ thật nhìn thấy")
    base = add_global_aggregates_serving(add_global_aggregates(add_hour_bucket(add_future_window(full.drop("fraud_type", "is_fraud", "label_known_at")))))
    base = add_device_fraud_history(add_device_fraud_history(base, labels, honest=False), labels, honest=True)
    base = base.join(labels, on="trip_id").with_columns(
        # phiên bản "phục vụ" của đặc trưng hàng giờ: chỉ đếm tới thời điểm hiện tại trong giờ
        dev_trips_this_hour_serving=pl.int_range(1, pl.len() + 1).over(["device_id", pl.col("paid_at").dt.truncate("1h")]),
        dev_trips_next_1h_serving=pl.lit(0),
        paid_day=day_of(),
    ).sort("paid_at")
    train = base.filter(pl.col("paid_day") < TRAIN_END_DAY)
    valid = base.filter(pl.col("paid_day") >= VALID_START_DAY)
    y_tr, y_va = train["is_fraud"].to_numpy(), valid["is_fraud"].to_numpy()
    fraud_v = valid.filter(pl.col("is_fraud"))
    naive_pos = (fraud_v["dev_fraud_naive"] > 0).mean()
    known_pos = (fraud_v["dev_fraud_known"] > 0).mean()
    say(f"train: {train.height:,} chuyến (ngày < {TRAIN_END_DAY}); valid: {valid.height:,} chuyến (ngày >= {VALID_START_DAY}, {int(y_va.sum()):,} gian lận)")
    say(f"chuyến gian lận ở valid có dev_fraud_naive > 0: {naive_pos:.1%}; có dev_fraud_known > 0: {known_pos:.1%}")

    def run(extra_train: list[str], extra_serving: list[str] | None):
        model = hgb().fit(train.select(FEATURES + extra_train).to_numpy(), y_tr)
        off = average_precision_score(y_va, model.predict_proba(valid.select(FEATURES + extra_train).to_numpy())[:, 1])
        srv = None
        if extra_serving is not None:
            srv = average_precision_score(y_va, model.predict_proba(valid.select(FEATURES + extra_serving).to_numpy())[:, 1])
        return model, off, srv

    m0, pr0, _ = run([], None)
    _, pr1_off, pr1_srv = run(["dev_trips_next_1h", "dev_trips_this_hour"], ["dev_trips_next_1h_serving", "dev_trips_this_hour_serving"])
    _, pr2_off, pr2_srv = run(["dev_customers_total", "pair_trips_total"], ["dev_customers_total_serving", "pair_trips_total_serving"])
    _, pr3_off, pr3_srv = run(["dev_fraud_naive"], ["dev_fraud_known"])
    _, pr4, _ = run(["dev_fraud_known"], None)
    s0 = m0.predict_proba(valid.select(FEATURES).to_numpy())[:, 1]
    say(f"HistGradientBoosting mặc định: tối đa 150 vòng, mô hình sạch dừng sớm ở vòng {m0.n_iter_} (n_iter_); dừng sớm dùng 10 % train tách ngẫu nhiên")
    say(f"{'mô hình (HistGradientBoosting)':50s}{'PR-AUC offline':>16s}{'PR-AUC phục vụ':>16s}")
    say(f"{'đặc trưng sạch (' + str(len(FEATURES)) + ')':50s}{pr0:16.3f}{pr0:16.3f}")
    say(f"{'+ cửa sổ nhìn trước (next_1h, this_hour)':50s}{pr1_off:16.3f}{pr1_srv:16.3f}")
    say(f"{'+ tổng hợp cả bảng theo khóa (2 cột)':50s}{pr2_off:16.3f}{pr2_srv:16.3f}")
    say(f"{'+ lịch sử gian lận thiết bị, nhãn tính ngay':50s}{pr3_off:16.3f}{pr3_srv:16.3f}")
    say(f"{'+ lịch sử gian lận, chỉ nhãn đã biết (cả hai nơi)':50s}{pr4:16.3f}{pr4:16.3f}")
    noise = [average_precision_score(y_va, hgb(rs).fit(train.select(FEATURES).to_numpy(), y_tr).predict_proba(valid.select(FEATURES).to_numpy())[:, 1])
             for rs in range(5)]
    say(f"độ nhiễu của chính mô hình sạch khi chỉ đổi random_state (0 đến 4): PR-AUC từ {min(noise):.3f} đến {max(noise):.3f}")
    say(f"mô hình sạch: ROC-AUC {roc_auc_score(y_va, s0):.3f}")

    # Bảng 2x2: đổi đặc trưng (13 theo từng chuyến so với đủ 30) và đổi mô hình (hồi quy logistic so với cây tăng cường).
    def fit_pr(model, cols: list[str]) -> float:
        model.fit(train.select(cols).to_numpy(), y_tr)
        return average_precision_score(y_va, model.predict_proba(valid.select(cols).to_numpy())[:, 1])

    grid = {("HistGradientBoosting", 13): fit_pr(hgb(), BASE_FEATURES), ("HistGradientBoosting", 30): pr0,
            ("hồi quy logistic", 13): fit_pr(lr(), BASE_FEATURES), ("hồi quy logistic", 30): fit_pr(lr(), FEATURES)}
    say(f"PR-AUC trên valid: {'':22s}{'13 đặc trưng theo từng chuyến':>32s}{'30 đặc trưng':>14s}")
    for name in ("hồi quy logistic", "HistGradientBoosting"):
        say(f"{'':26s}{name:22s}{grid[(name, 13)]:12.3f}{'':20s}{grid[(name, 30)]:.3f}")
    say(f"{'loại':12s}{'chuyến valid':>13s}{'recall@top1%':>14s}")
    top = np.argsort(-s0, kind="stable")[: int(0.01 * len(s0))]
    flagged = np.zeros(len(s0), dtype=bool)
    flagged[top] = True
    rec = {}
    for name in FRAUD_TYPES[1:]:
        m = (valid["fraud_type"] == name).to_numpy()
        rec[name] = float(flagged[m].mean())
        say(f"{name:12s}{int(m.sum()):13d}{rec[name]:14.1%}")
    c.check("đặc trưng sạch: PR-AUC trên valid cao hơn hẳn bộ chấm nền bài 54 (khoảng 0,54)", pr0 > 0.75, pr0)
    c.check("cửa sổ nhìn trước: PR-AUC offline không báo động", abs(pr1_off - pr0) < 0.03, (pr1_off, pr0))
    c.check("tổng hợp cả bảng: offline đẹp hơn phục vụ", pr2_off > pr2_srv + 0.01, (pr2_off, pr2_srv))
    c.check("nhãn tính ngay: offline đẹp hơn phục vụ rõ", pr3_off > pr3_srv + 0.03, (pr3_off, pr3_srv))
    c.check("nhãn tính ngay: phục vụ tệ hơn mô hình không dùng nó", pr3_srv < pr0, (pr3_srv, pr0))
    c.check("lịch sử chỉ gồm nhãn đã biết, dùng cả hai nơi: không thua mô hình sạch", pr4 > pr0 - 0.01, (pr4, pr0))
    c.check("lịch sử gian lận tính ngay nhiều hơn lịch sử nhãn đã biết", naive_pos > known_pos, (naive_pos, known_pos))
    c.check("30 đặc trưng hơn hẳn 13 đặc trưng ở cả hai mô hình",
            grid[("HistGradientBoosting", 30)] > grid[("HistGradientBoosting", 13)] + 0.1
            and grid[("hồi quy logistic", 30)] > grid[("hồi quy logistic", 13)] + 0.1, grid)
    c.check("chuyến của ngày valid vẫn đủ gian lận để đo", y_va.sum() > 500)
    c.check("đặc trưng nhiều chuyến cứu collusion: recall@top1% trên 0,1", rec["collusion"] > 0.1, rec)


# ---- [5] Online từ event, và Kafka ---------------------------------------------------------


def compare(online: dict[str, dict[str, float]], batch: pl.DataFrame) -> tuple[dict[str, int], list[str]]:
    """Số chuyến lệch theo từng đặc trưng, và danh sách chuyến có ít nhất một đặc trưng lệch (online so với theo lô)."""
    ref = batch.filter(pl.col("trip_id").is_in(list(online))).select("trip_id", *ONLINE_FEATURES)
    bad = dict.fromkeys(ONLINE_FEATURES, 0)
    wrong: list[str] = []
    for row in ref.iter_rows(named=True):
        got = online[row["trip_id"]]
        diff = [x for x in ONLINE_FEATURES if not math.isclose(got[x], row[x], rel_tol=1e-9, abs_tol=1e-9)]
        for x in diff:
            bad[x] += 1
        if diff:
            wrong.append(row["trip_id"])
    return bad, wrong


def replay(envelopes: list[dict]) -> tuple[dict[str, dict[str, float]], OnlineFeatures]:
    proc = OnlineFeatures()
    out = {}
    for env in envelopes:
        res = proc.on_envelope(env)
        if res:
            out[res[0]] = res[1]
    return out, proc


TOPIC_ORDER = ("BookingCreated", "TripCompleted", "PaymentCaptured")


def interleave(by_type: dict[str, list[dict]], batch: int) -> list[dict]:
    """Phát lại xen kẽ cố định: lấy `batch` bản ghi của mỗi topic theo vòng, như một consumer đọc theo lô cỡ `batch`."""
    pos = dict.fromkeys(TOPIC_ORDER, 0)
    out: list[dict] = []
    while any(pos[k] < len(by_type[k]) for k in TOPIC_ORDER):
        for k in TOPIC_ORDER:
            out.extend(by_type[k][pos[k]:pos[k] + batch])
            pos[k] += batch
    return out


def step_online(events: pl.DataFrame, full: pl.DataFrame, c: Checker) -> None:
    say(f"\n[5] Tính online từ envelope (event của {STREAM_DAYS} ngày đầu)")
    sub = events.filter(pl.col("occurred_at") < START + timedelta(days=STREAM_DAYS))
    envs = [env for _, env in iter_envelopes(sub)]
    t0 = time.perf_counter()
    online, proc = replay(envs)
    took = time.perf_counter() - t0
    say(f"{len(envs):,} envelope -> {len(online):,} chuyến đã đủ ba event, {len(proc.pending)} chuyến còn thiếu event; {took:.2f} giây ({len(envs) / took:,.0f} envelope/giây)")
    bad, wrong = compare(online, full)
    say(f"so với tính theo lô trên toàn bộ dữ liệu: {len(wrong)} chuyến lệch (trên {len(online):,}, {len(ONLINE_FEATURES)} đặc trưng, đủ cả {len(FEATURES)} đặc trưng của features.py)")
    c.check("online theo thứ tự thời gian khớp tính theo lô", not wrong, {k: v for k, v in bad.items() if v})
    c.check("bộ tính online phủ đủ mọi đặc trưng của features.py", set(ONLINE_FEATURES) == set(FEATURES), set(FEATURES) ^ set(ONLINE_FEATURES))
    c.check("có đủ chuyến để so", len(online) > 5_000, len(online))

    say("\nQua Kafka thật (3 topic, mỗi topic 1 partition, một consumer đọc cả ba):")
    records, timing = asyncio.run(stream.roundtrip(sub))
    by_topic = {t: sum(r.topic == t for r in records) for t in sorted({r.topic for r in records})}
    say(f"gửi {int(timing['sent']):,} event trong {timing['produce_s']:.1f} giây, đọc lại {len(records):,} bản ghi trong {timing['consume_s']:.1f} giây; load average: {load()}")
    say("bản ghi theo topic:", by_topic)
    arrival = [r.envelope for r in records]
    idx: dict[str, dict[str, int]] = {}
    for i, r in enumerate(records):
        idx.setdefault(r.envelope["payload"]["trip_id"], {})[r.envelope["event_type"]] = i
    complete = [v for v in idx.values() if len(v) == 3]
    pay_first = sum(v["PaymentCaptured"] < v["TripCompleted"] for v in complete)
    say(f"{pay_first:,}/{len(complete):,} chuyến có PaymentCaptured được đọc TRƯỚC TripCompleted của chính nó")

    paid = dict(full.select("trip_id", "paid_at").iter_rows())

    def inversions_of(order: list[str]) -> int:
        return sum(paid[a] > paid[b] for a, b in zip(order, order[1:], strict=False))

    arr_online, arr_proc = replay(arrival)
    inv = inversions_of(arr_proc.closed_order)
    bad_arr, wrong_arr = compare(arr_online, full)
    kinds = full.filter(pl.col("trip_id").is_in(wrong_arr)).group_by("fraud_type").len().sort("len", descending=True)
    say(f"xử lý đúng THỨ TỰ ĐỌC của lần chạy này: {inv:,}/{len(arr_proc.closed_order) - 1:,} cặp chuyến đóng liên tiếp bị đảo so với paid_at; "
        f"{len(wrong_arr)} chuyến có đặc trưng lệch ({dict(zip(kinds['fraud_type'], kinds['len'], strict=True))}); theo đặc trưng:")
    for x, v in bad_arr.items():
        if v:
            say(f"  {x:20s}{v:7,d} chuyến lệch")
    # Số chuyến lệch phụ thuộc cách consumer chia lô, và thứ tự đọc đổi theo từng lần chạy: phát lại xen kẽ cố định để đo được.
    by_type = {k: [r.envelope for r in records if r.envelope["event_type"] == k] for k in TOPIC_ORDER}
    say("phát lại xen kẽ cố định (lấy N bản ghi của mỗi topic theo vòng), số chuyến có đặc trưng lệch so với bản theo lô:")
    wrong_by_batch = {}
    for batch in (10, 50, 500, 2000, 100_000):
        res, _ = replay(interleave(by_type, batch))
        wrong_by_batch[batch] = len(compare(res, full)[1])
        say(f"  lô {batch:7,d}: {wrong_by_batch[batch]:5,d} chuyến lệch")
    srt_online, _ = replay(sorted(arrival, key=lambda e: e["occurred_at"]))
    bad_srt, wrong_srt = compare(srt_online, full)
    say(f"sắp lại theo occurred_at rồi xử lý: {len(wrong_srt)} chuyến lệch ({len(srt_online):,} chuyến)")
    c.check("Kafka giao đủ mọi event", len(records) == int(timing["sent"]) == sub.height, (len(records), timing["sent"], sub.height))
    c.check("mỗi topic đủ số event của loại đó", sum(by_topic.values()) == sub.height)
    c.check("đọc nhiều topic làm đảo thứ tự giữa các event", pay_first > 0 and inv > 0, (pay_first, inv))
    c.check("phát lại xen kẽ theo lô nhỏ làm đặc trưng lệch", wrong_by_batch[10] > 0, wrong_by_batch)
    c.check("sắp lại theo thời gian (watermark vô hạn) khớp tính theo lô", not wrong_srt, {k: v for k, v in bad_srt.items() if v})
    by_id = lambda e: e["event_id"]  # noqa: E731
    c.check("bản ghi qua Kafka giải mã giống envelope gốc", sorted(arrival, key=by_id) == sorted(envs, key=by_id))


# ---- [6] Lưu feature -----------------------------------------------------------------------


def step_parquet(full: pl.DataFrame, c: Checker) -> None:
    say("\n[6] Lưu feature Parquet")
    OUT.mkdir(exist_ok=True)
    path = OUT / "features.parquet"
    out = full.select("trip_id", "paid_at", *FEATURES)
    out.write_parquet(path)
    back = pl.read_parquet(path)
    say(f"out/features.parquet: {out.height:,} dòng x {out.width} cột, {path.stat().st_size / 2**20:.1f} MiB; đọc lại giống hệt: {back.equals(out)}")
    c.check("features.parquet đọc lại giống hệt", back.equals(out))
    c.check("features.parquet không có cột nhãn", not ({"fraud_type", "is_fraud", "label_known_at"} & set(back.columns)))
    c.check("danh sách đặc trưng gồm nhóm cửa sổ và nhóm GPS", set(WINDOW_FEATURES + GPS_FEATURES) <= set(FEATURES))


def run() -> int:
    c = Checker()
    data, trips = step_trips(c)
    full = step_windows(trips, data.labels, c)
    step_gps(full, c)
    step_prefix(data.events, data.labels, c)
    step_leak_models(full, data.labels, c)
    step_online(data.events, full, c)
    step_parquet(full, c)

    if c.failures:
        print(f"FAIL {c.total - len(c.failures)}/{c.total}")
        return 1
    if QUIET:
        print(f"OK {c.total}/{c.total}")
    else:
        say(f"\nKiểm chứng: OK {c.total}/{c.total}")
    return 0


def main() -> None:
    global QUIET
    QUIET = "--verify" in sys.argv[1:]
    sys.exit(run())


if __name__ == "__main__":
    main()
