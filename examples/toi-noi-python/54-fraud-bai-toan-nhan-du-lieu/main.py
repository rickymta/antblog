"""Bài 54 - Gian lận trong Tới Nơi: loại hình, nhãn, dữ liệu và metric.

Chạy từ gốc repository:
    uv run --project examples/toi-noi-python/54-fraud-bai-toan-nhan-du-lieu \
        examples/toi-noi-python/54-fraud-bai-toan-nhan-du-lieu/main.py
    ... main.py --verify    # kiểm các bất biến in trong bài, in OK n/n

Mọi dữ liệu là TỔNG HỢP (synthetic_fraud.py). Mọi con số chỉ nói về bộ này, không đại diện cho dữ liệu thật.
Không cần Docker, không cần mạng khi chạy.
"""

import json
import os
import sys
import time
from pathlib import Path
from zoneinfo import ZoneInfo

import numpy as np
import polars as pl
from sklearn.metrics import average_precision_score, roc_auc_score

import metrics
from dataset import (
    CUTOFF_DAY,
    RAW_FEATURES,
    TRAIN_END_DAY,
    VALID_START_DAY,
    at_day,
    fit_scorer,
    known_label,
    trip_table,
)
from synthetic_fraud import EVENT_SCHEMA, FRAUD_TYPES, START, FraudData, iter_envelopes, make_events

SEED = 42
N_TRIPS = 200_000
OUT = Path(__file__).parent / "out"
REVIEW_COST_VND = 8_000  # giả định: 5 phút người duyệt + chậm chi trả cho một chuyến bị gắn cờ nhầm
HANDLING_COST_VND = 20_000  # giả định: xử lý hoàn tiền/khiếu nại cho một chuyến gian lận bị bỏ sót
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


def vn_day(day: float) -> str:
    return at_day(day).astimezone(ZoneInfo("Asia/Ho_Chi_Minh")).strftime("%d/%m/%Y %H:%M")


# ---- [1] Bộ dữ liệu ------------------------------------------------------------------------


def step_dataset(c: Checker) -> tuple[FraudData, float]:
    say("\n[1] Bộ dữ liệu tổng hợp")
    t0 = time.perf_counter()
    data = make_events(seed=SEED, n_trips=N_TRIPS)
    took = time.perf_counter() - t0
    ev, lab = data.events, data.labels
    say(f"make_events(seed={SEED}, n_trips={N_TRIPS:,}) mất {took:.1f} giây; load average 1/5/15 phút: "
        + " ".join(f"{x:.2f}" for x in os.getloadavg()))
    say(f"events: {ev.height:,} dòng x {ev.width} cột; labels: {lab.height:,} dòng x {lab.width} cột")
    say("cột events:", ", ".join(f"{n}:{d}" for n, d in ev.schema.items()))
    lo, hi = ev["occurred_at"].min(), ev["occurred_at"].max()
    say(f"occurred_at: {lo:%Y-%m-%d %H:%M} đến {hi:%Y-%m-%d %H:%M} UTC; tăng dần: {ev['occurred_at'].is_sorted()}")
    per_type = ev.group_by("event_type").len().sort("event_type")
    say("số event theo loại:", dict(zip(per_type["event_type"], per_type["len"], strict=True)))
    say(f"khách {ev['customer_id'].n_unique():,}; tài xế {ev['driver_id'].drop_nulls().n_unique():,}; "
        f"thiết bị khách {ev['device_id'].drop_nulls().n_unique():,}")
    counts = lab.group_by("fraud_type").len()
    by_type = dict(zip(counts["fraud_type"], counts["len"], strict=True))
    say(f"{'fraud_type':12s}{'chuyến':>9s}{'tỉ lệ':>9s}")
    for name in FRAUD_TYPES:
        say(f"{name:12s}{by_type[name]:9,d}{by_type[name] / lab.height:9.2%}")
    rate = lab.filter(pl.col("is_fraud")).height / lab.height
    say(f"tỉ lệ gian lận chung: {rate:.2%}")
    topic, env = next(iter_envelopes(ev.head(1)))
    say(f"envelope đầu tiên (topic {topic}):")
    say(json.dumps(env, ensure_ascii=False))

    c.check("events có đúng các cột và kiểu trong schema", dict(ev.schema) == EVENT_SCHEMA, ev.schema)
    c.check("events không có cột nhãn", not ({"fraud_type", "is_fraud", "label_known_at"} & set(ev.columns)))
    c.check("occurred_at tăng dần", bool(ev["occurred_at"].is_sorted()))
    c.check("mỗi chuyến đúng 3 event, đủ ba loại",
            ev.group_by("trip_id").agg(n=pl.len(), k=pl.col("event_type").n_unique())
            .filter((pl.col("n") != 3) | (pl.col("k") != 3)).height == 0)
    c.check("event_id không trùng", ev["event_id"].n_unique() == ev.height)
    c.check("năm giá trị fraud_type", set(by_type) == set(FRAUD_TYPES), set(by_type))
    c.check("tỉ lệ gian lận khoảng 2 %", 0.018 <= rate <= 0.022, rate)
    c.check("labels có một dòng mỗi chuyến", lab.height == N_TRIPS and lab["trip_id"].n_unique() == N_TRIPS)
    return data, took


# ---- [2] Tín hiệu từng loại ----------------------------------------------------------------


def step_signals(t: pl.DataFrame, c: Checker) -> None:
    say("\n[2] Tín hiệu của từng loại gian lận (đặc trưng theo từng chuyến)")
    clean = t.filter(pl.col("fraud_type") == "none")
    rules = [
        ("ghost_trip", "route_ratio < 0.4", pl.col("route_ratio") < 0.4),
        ("gps_spoof", "start_pickup_gap_km > 1", pl.col("start_pickup_gap_km") > 1),
        ("promo_abuse", "promo_code == NEWUSER50", pl.col("has_newuser_promo") == 1),
        ("collusion", "device_id == driver_device_id", pl.col("same_device") == 1),
    ]
    say(f"{'loại':12s}{'luật':32s}{'chuyến loại':>12s}{'bắt được':>10s}{'sạch bị bắt':>12s}{'precision':>10s}")
    stats = {}
    for name, text, expr in rules:
        sub = t.filter(pl.col("fraud_type") == name)
        hit, fp = sub.filter(expr).height, clean.filter(expr).height
        stats[name] = (hit / sub.height, hit / (hit + fp))
        say(f"{name:12s}{text:32s}{sub.height:12,d}{hit / sub.height:10.1%}{fp:12,d}{hit / (hit + fp):10.1%}")
    c.check("luật GPS bắt hầu hết gps_spoof", stats["gps_spoof"][0] > 0.9, stats["gps_spoof"])
    c.check("luật tỉ lệ đường bắt đa số ghost_trip nhưng không hoàn hảo", 0.4 < stats["ghost_trip"][0] < 0.9, stats["ghost_trip"])
    c.check("luật khuyến mãi precision thấp vì khách mới hợp lệ cũng dùng mã", stats["promo_abuse"][1] < 0.4, stats["promo_abuse"])
    c.check("luật cùng thiết bị chỉ bắt một phần collusion", 0.1 < stats["collusion"][0] < 0.6, stats["collusion"])

    # Tín hiệu cần nhìn nhiều chuyến. Đếm trên TOÀN BỘ bộ dữ liệu nên chỉ để mô tả: bài 55 làm đúng theo thời điểm.
    pair = t.group_by("customer_id", "driver_id").agg(pair_n=pl.len())
    dev = t.group_by("device_id").agg(dev_customers=pl.col("customer_id").n_unique())
    w = t.join(pair, on=["customer_id", "driver_id"]).join(dev, on="device_id")
    say("tín hiệu nhiều chuyến (đếm trên cả tập, chỉ để mô tả):")
    say(f"{'loại':12s}{'cặp khách-tài xế >= 3 chuyến':>32s}{'thiết bị >= 3 khách':>22s}")
    multi = {}
    for name in FRAUD_TYPES:
        sub = w.filter(pl.col("fraud_type") == name)
        multi[name] = ((sub["pair_n"] >= 3).mean(), (sub["dev_customers"] >= 3).mean())
        say(f"{name:12s}{multi[name][0]:32.1%}{multi[name][1]:22.1%}")
    c.check("cặp lặp lại tách collusion khỏi chuyến sạch", multi["collusion"][0] > 0.8 and multi["none"][0] < 0.1, multi)
    c.check("nhiều khách chung thiết bị tách promo_abuse khỏi chuyến sạch", multi["promo_abuse"][1] > 0.5 and multi["none"][1] < 0.02, multi)


# ---- [3] Nhãn trễ --------------------------------------------------------------------------


def step_label_delay(t: pl.DataFrame, c: Checker) -> None:
    say("\n[3] Nhãn trễ (label delay)")
    lab = t.select("trip_id", "paid_at", "paid_day", "fraud_type", "is_fraud", "label_known_at").with_columns(
        delay_days=(pl.col("label_known_at") - pl.col("paid_at")).dt.total_seconds() / 86400
    )
    say(f"{'loại':12s}{'trung vị (ngày)':>16s}{'p90 (ngày)':>12s}{'<= 7 ngày':>11s}{'<= 14 ngày':>12s}{'<= 30 ngày':>12s}")
    med = {}
    for name in FRAUD_TYPES[1:]:
        d = lab.filter(pl.col("fraud_type") == name)["delay_days"]
        med[name] = d.median()
        say(f"{name:12s}{d.median():16.1f}{d.quantile(0.9):12.1f}{(d <= 7).mean():11.1%}{(d <= 14).mean():12.1%}{(d <= 30).mean():12.1%}")
    say(f"{'none':12s}{'30.0':>16s}   (chuyến sạch chỉ được xác nhận sạch khi hết hạn khiếu nại 30 ngày)")

    fraud = lab.filter(pl.col("is_fraud"))
    for cut in (30, 40):
        done = fraud.filter(pl.col("paid_day") < cut)
        known = known_label(done.select("trip_id", "is_fraud", "label_known_at"), CUTOFF_DAY)["known_fraud"].sum()
        say(f"chuyến gian lận thanh toán trước ngày {cut}: {done.height:,}; biết được vào ngày {CUTOFF_DAY}: {known:,} ({known / done.height:.1%})")
    recent = fraud.filter((pl.col("paid_day") >= CUTOFF_DAY - 7) & (pl.col("paid_day") < CUTOFF_DAY))
    k_recent = known_label(recent.select("trip_id", "is_fraud", "label_known_at"), CUTOFF_DAY)["known_fraud"].sum()
    say(f"chuyến gian lận của 7 ngày ngay trước ngày {CUTOFF_DAY}: {recent.height:,}; biết được: {k_recent:,} ({k_recent / recent.height:.1%})")
    done = fraud.filter(pl.col("paid_day") < TRAIN_END_DAY).select("trip_id", "fraud_type", "is_fraud", "label_known_at")
    kn = known_label(done, CUTOFF_DAY)
    mature = {n: kn.filter(pl.col("fraud_type") == n)["known_fraud"].mean() for n in FRAUD_TYPES[1:]}
    say(f"train (trước ngày {TRAIN_END_DAY}), độ chín của nhãn gian lận ở ngày {CUTOFF_DAY} theo loại: "
        + ", ".join(f"{n} {v:.1%}" for n, v in mature.items()))
    say("chuyến sạch chưa hết hạn khiếu nại 30 ngày vẫn mang nhãn 0 lúc đó, nên khoảng chờ 10 ngày không chữa loại điều tra lâu")
    c.check("gian lận phát hiện nhanh nhất là promo_abuse, chậm nhất là collusion", min(med, key=med.get) == "promo_abuse" and max(med, key=med.get) == "collusion", med)
    c.check("label_known_at luôn sau paid_at", bool((lab["delay_days"] > 0).all()))
    c.check("7 ngày gần nhất mới biết dưới nửa số gian lận", k_recent / recent.height < 0.5, k_recent / recent.height)
    c.check("khoảng chờ 10 ngày vẫn để collusion chín kém nhất", min(mature, key=mature.get) == "collusion" and mature["collusion"] < 0.8, mature)


# ---- [4] Chia theo thời gian ---------------------------------------------------------------


def x_of(df: pl.DataFrame) -> np.ndarray:
    return df.select(RAW_FEATURES).to_numpy()


def blacklist_eval(train: pl.DataFrame, test: pl.DataFrame) -> tuple[int, float, float]:
    """Danh sách đen thiết bị: thiết bị từng có gian lận ở train thì gắn cờ mọi chuyến của nó ở test."""
    bad = set(train.filter(pl.col("is_fraud"))["device_id"])
    flag = test["device_id"].is_in(list(bad)).to_numpy()
    y = test["is_fraud"].to_numpy()
    tp, fp = int((flag & y).sum()), int((flag & ~y).sum())
    return len(bad), tp / max(1, int(y.sum())), tp / max(1, tp + fp)


def step_split(t: pl.DataFrame, c: Checker) -> tuple[pl.DataFrame, pl.DataFrame]:
    say("\n[4] Chia train/valid")
    train = t.filter(pl.col("paid_day") < TRAIN_END_DAY)
    valid = t.filter(pl.col("paid_day") >= VALID_START_DAY)
    say(f"train: thanh toán trước ngày {TRAIN_END_DAY} ({vn_day(0)} đến {vn_day(TRAIN_END_DAY)}): {train.height:,} chuyến, "
        f"{train['is_fraud'].sum():,} gian lận ({train['is_fraud'].mean():.2%})")
    say(f"valid: từ ngày {VALID_START_DAY} ({vn_day(VALID_START_DAY)}): {valid.height:,} chuyến, "
        f"{valid['is_fraud'].sum():,} gian lận ({valid['is_fraud'].mean():.2%}); ngày {TRAIN_END_DAY} đến {VALID_START_DAY} bỏ trống làm khoảng chờ nhãn")

    # So sánh công bằng: hai cách chia dùng CÙNG cỡ train (mọi chuyến trước ngày 40) và cùng cỡ test (valid).
    pre40 = t.filter(pl.col("paid_day") < VALID_START_DAY)
    shuffled = t.sample(fraction=1.0, shuffle=True, seed=SEED)
    r_train, r_test = shuffled.head(pre40.height), shuffled.slice(pre40.height, valid.height)
    rows = {}
    say(f"cùng cỡ: train {pre40.height:,} chuyến, test {valid.height:,} chuyến")
    say(f"{'cách chia':30s}{'thiết bị đen':>13s}{'recall':>9s}{'precision':>11s}{'PR-AUC (LR)':>13s}")
    for name, (a, b) in {"ngẫu nhiên": (r_train, r_test), "theo thời gian": (pre40, valid)}.items():
        n_bad, rec, prec = blacklist_eval(a, b)
        model = fit_scorer(x_of(a), a["is_fraud"].to_numpy())
        pr = metrics.average_precision(b["is_fraud"].to_numpy(), model.predict_proba(x_of(b))[:, 1])
        rows[name] = (rec, prec, pr)
        say(f"{name:30s}{n_bad:13,d}{rec:9.1%}{prec:11.1%}{pr:13.3f}")
    # Hai biến thể khác của cách chia theo thời gian: có khoảng chờ, và nhãn chỉ biết đến ngày 40.
    known = known_label(train.select("trip_id", "is_fraud", "label_known_at"), CUTOFF_DAY)
    bad_known = set(train.filter(known["known_fraud"])["device_id"])
    flag = valid["device_id"].is_in(list(bad_known)).to_numpy()
    yv = valid["is_fraud"].to_numpy()
    n_bad, rec, prec = blacklist_eval(train, valid)
    say(f"{'theo thời gian, train < 30':30s}{n_bad:13,d}{rec:9.1%}{prec:11.1%}{'':>13s}")
    say(f"{'... nhãn chỉ biết ở ngày 40':30s}{len(bad_known):13,d}{(flag & yv).sum() / yv.sum():9.1%}"
        f"{(flag & yv).sum() / max(1, flag.sum()):11.1%}{'':>13s}")
    seen = set(pre40["device_id"])
    bad_all = set(pre40.filter(pl.col("is_fraud"))["device_id"])
    fraud_v = valid.filter(pl.col("is_fraud"))
    not_black = 1 - fraud_v["device_id"].is_in(list(bad_all)).mean()
    unseen = 1 - fraud_v["device_id"].is_in(list(seen)).mean()
    say(f"gian lận ở valid: {not_black:.1%} nằm trên thiết bị không có trong danh sách đen; {unseen:.1%} nằm trên thiết bị chưa từng xuất hiện trước ngày {VALID_START_DAY}")

    # Tạo tác của bộ sinh: quần thể khách gần như cố định, nên tỉ lệ chuyến sạch dùng NEWUSER50 giảm dần theo thời gian.
    say("tỉ lệ chuyến sạch dùng NEWUSER50 và PR-AUC (LR, train < 30 chấm trên từng giai đoạn 10 ngày):")
    model30 = fit_scorer(x_of(train), train["is_fraud"].to_numpy())
    period = {}
    for lo in range(0, 60, 10):
        part = t.filter((pl.col("paid_day") >= lo) & (pl.col("paid_day") < lo + 10 if lo < 50 else pl.col("paid_day") >= lo))
        share = part.filter(~pl.col("is_fraud"))["has_newuser_promo"].mean()
        pr_p = metrics.average_precision(part["is_fraud"].to_numpy(), model30.predict_proba(x_of(part))[:, 1])
        period[lo] = (share, pr_p)
        say(f"  ngày {lo:2d} đến {lo + 10:2d}: sạch dùng NEWUSER50 {share:6.1%}, PR-AUC {pr_p:.3f}")
    drop = [i for i, f in enumerate(RAW_FEATURES) if f not in ("has_newuser_promo", "discount_ratio")]
    no_promo = {}
    for name, (a, b) in {"ngẫu nhiên": (r_train, r_test), "theo thời gian": (pre40, valid)}.items():
        mdl = fit_scorer(x_of(a)[:, drop], a["is_fraud"].to_numpy())
        no_promo[name] = metrics.average_precision(b["is_fraud"].to_numpy(), mdl.predict_proba(x_of(b)[:, drop])[:, 1])
    say(f"bỏ hai cột has_newuser_promo, discount_ratio: PR-AUC (LR) ngẫu nhiên {no_promo['ngẫu nhiên']:.3f}, theo thời gian {no_promo['theo thời gian']:.3f}")
    c.check("cùng cỡ train: chia ngẫu nhiên làm danh sách đen thiết bị trông tốt hơn chia theo thời gian",
            rows["ngẫu nhiên"][0] > rows["theo thời gian"][0] + 0.15, rows)
    c.check("quá nửa gian lận ở valid không nằm trong danh sách đen", not_black > 0.5, not_black)
    c.check("tỉ lệ chuyến sạch dùng NEWUSER50 giảm hẳn theo thời gian (tạo tác của bộ sinh)", period[50][0] < 0.5 * period[0][0], period)
    c.check("không chuyến nào của train nằm sau valid", train["paid_at"].max() < valid["paid_at"].min())
    return train, valid


# ---- [5] Metric trên dữ liệu mất cân bằng --------------------------------------------------


def step_metrics(train: pl.DataFrame, valid: pl.DataFrame, c: Checker) -> tuple[np.ndarray, np.ndarray]:
    say("\n[5] Metric trên tập valid")
    y = valid["is_fraud"].to_numpy()
    model = fit_scorer(x_of(train), train["is_fraud"].to_numpy())
    score = model.predict_proba(x_of(valid))[:, 1]
    rate = y.mean()

    never = metrics.confusion(y, np.zeros(len(y)), 0.5)
    say(f"luôn đoán «sạch»:  accuracy {never.accuracy:.4f}, recall {never.recall:.2f} (tỉ lệ gian lận {rate:.4f})")
    rand = np.random.default_rng(0).random(len(y))
    r_pr, r_roc = metrics.average_precision(y, rand), metrics.roc_auc(y, rand)
    say(f"điểm ngẫu nhiên:  PR-AUC {r_pr:.4f}, ROC-AUC {r_roc:.4f}")
    pr, roc = metrics.average_precision(y, score), metrics.roc_auc(y, score)
    say(f"hồi quy logistic:  PR-AUC {pr:.4f}, ROC-AUC {roc:.4f}")
    say(f"đối chiếu scikit-learn: average_precision {average_precision_score(y, score):.4f}, roc_auc {roc_auc_score(y, score):.4f}")
    cf = metrics.confusion(y, score, 0.5)
    say(f"ngưỡng 0.5: TP {cf.tp}, FP {cf.fp}, FN {cf.fn}, TN {cf.tn:,}; precision {cf.precision:.3f}, recall {cf.recall:.3f}, "
        f"F1 {cf.f1:.3f}, accuracy {cf.accuracy:.4f}")

    keep = (y | (np.random.default_rng(1).random(len(y)) < 0.1))  # giữ mọi gian lận, 10 % chuyến sạch
    pr_sub, roc_sub = metrics.average_precision(y[keep], score[keep]), metrics.roc_auc(y[keep], score[keep])
    say(f"cùng điểm, chỉ giữ 10% chuyến sạch ({keep.sum():,} dòng, tỉ lệ gian lận {y[keep].mean():.1%}): "
        f"PR-AUC {pr_sub:.4f}, ROC-AUC {roc_sub:.4f}")

    say(f"{'loại':12s}{'chuyến':>8s}{'recall@0.5':>12s}{'recall@0.1':>12s}{'PR-AUC':>9s}")
    by = {}
    for name in FRAUD_TYPES[1:]:
        m = (valid["fraud_type"] == name).to_numpy()
        both = m | ~y
        by[name] = ((score[m] >= 0.5).mean(), metrics.average_precision(m[both], score[both]))
        say(f"{name:12s}{m.sum():8d}{(score[m] >= 0.5).mean():12.1%}{(score[m] >= 0.1).mean():12.1%}{by[name][1]:9.3f}")

    c.check("luôn đoán sạch: accuracy cao, recall 0", never.accuracy > 0.97 and never.recall == 0.0, never)
    c.check("điểm ngẫu nhiên: PR-AUC xấp xỉ tỉ lệ gian lận, ROC-AUC xấp xỉ 0,5",
            abs(r_pr - rate) < 0.01 and abs(r_roc - 0.5) < 0.03, (r_pr, r_roc))
    c.check("average_precision tự viết khớp scikit-learn", abs(pr - average_precision_score(y, score)) < 1e-9)
    c.check("roc_auc tự viết khớp scikit-learn", abs(roc - roc_auc_score(y, score)) < 1e-9)
    c.check("ROC-AUC cao hơn hẳn PR-AUC trên dữ liệu mất cân bằng", roc > 0.9 and roc - pr > 0.1, (roc, pr))
    c.check("bỏ bớt chuyến sạch: ROC-AUC gần như giữ, PR-AUC tăng mạnh", abs(roc_sub - roc) < 0.02 and pr_sub > pr + 0.1, (roc, roc_sub, pr, pr_sub))
    c.check("collusion bị bỏ sót nhiều hơn ghost_trip nếu chỉ có đặc trưng theo từng chuyến", by["collusion"][0] < by["ghost_trip"][0] - 0.3, by)
    return score, y


# ---- [6] Chi phí hai chiều và ngưỡng ------------------------------------------------------


def step_cost(valid: pl.DataFrame, score: np.ndarray, c: Checker) -> None:
    say("\n[6] Chi phí hai chiều và ngưỡng (giả định minh họa)")
    say(f"mỗi chuyến bị gắn cờ (kể cả gian lận thật): {REVIEW_COST_VND:,} VND phí duyệt; mỗi chuyến gian lận bị bỏ sót: số tiền chuyến + {HANDLING_COST_VND:,} VND xử lý")
    say("giả định: gắn cờ là chặn hết thiệt hại (người duyệt không sai); amount_vnd đại diện cho thiệt hại (với promo_abuse thiệt hại thật là phần giảm)")
    y = valid["is_fraud"].to_numpy()
    cost_fn = valid["amount_vnd"].to_numpy().astype(float) + HANDLING_COST_VND
    half = (valid["paid_day"] < 50).to_numpy()  # nửa đầu để chọn ngưỡng, nửa sau để báo cáo
    ya, sa, ca = y[half], score[half], cost_fn[half]
    yb, sb, cb = y[~half], score[~half], cost_fn[~half]
    thr, _ = metrics.best_threshold(ya, sa, ca, REVIEW_COST_VND)
    say(f"chọn ngưỡng trên valid nửa đầu (40 đến 50): {thr:.3f}; báo cáo trên nửa sau: {len(yb):,} chuyến, {yb.sum()} gian lận")
    say(f"{'chính sách':26s}{'gắn cờ':>8s}{'TP':>6s}{'FP':>7s}{'FN':>6s}{'chi phí (VND)':>16s}")
    for name, th in (("không gắn cờ gì", np.inf), ("gắn cờ tất cả", -np.inf), ("ngưỡng 0.5", 0.5), (f"ngưỡng chọn theo chi phí {thr:.3f}", thr)):
        cf = metrics.confusion(yb, sb, th)
        cost = metrics.total_cost(yb, sb, th, cb, REVIEW_COST_VND)
        say(f"{name:26s}{cf.tp + cf.fp:8,d}{cf.tp:6d}{cf.fp:7d}{cf.fn:6d}{cost:16,.0f}")
    k = max(1, int(0.01 * len(yb)))
    top = np.argsort(-sb, kind="stable")[:k]
    mean_fn = float(cb[yb].mean())
    say(f"mốc lý thuyết nếu điểm là xác suất đã hiệu chỉnh: phí duyệt / chi phí bỏ sót trung bình = {REVIEW_COST_VND:,} / {mean_fn:,.0f} = {REVIEW_COST_VND / mean_fn:.3f}; "
        f"điểm của hồi quy logistic ở đây chưa hiệu chỉnh (điểm trung bình {score.mean():.4f}, tỉ lệ gian lận {y.mean():.4f})")
    best_b, cost_b = metrics.best_threshold(yb, sb, cb, REVIEW_COST_VND)
    say(f"(đối chiếu, ngưỡng tốt nhất nếu chọn ngay trên nửa sau: {best_b:.3f}, chi phí {cost_b:,.0f}; không dùng được ngoài đời)")
    say(f"người duyệt chỉ xem được 1% chuyến ({k} chuyến điểm cao nhất): bắt {yb[top].sum()} trong {yb.sum()} gian lận, precision {yb[top].mean():.1%}")
    best_cost = metrics.total_cost(yb, sb, thr, cb, REVIEW_COST_VND)
    c.check("ngưỡng chọn theo chi phí nhỏ hơn 0,5 khi bỏ sót đắt hơn gắn cờ nhầm", thr < 0.5, thr)
    c.check("ngưỡng chọn theo chi phí rẻ hơn ngưỡng 0.5 và rẻ hơn không làm gì", best_cost < metrics.total_cost(yb, sb, 0.5, cb, REVIEW_COST_VND)
            and best_cost < metrics.total_cost(yb, sb, np.inf, cb, REVIEW_COST_VND), best_cost)
    c.check("gắn cờ tất cả đắt hơn không làm gì", metrics.total_cost(yb, sb, -np.inf, cb, REVIEW_COST_VND) > metrics.total_cost(yb, sb, np.inf, cb, REVIEW_COST_VND))


# ---- [7] Nhãn trễ làm gì với việc đo và huấn luyện ----------------------------------------


def step_delay_effect(train: pl.DataFrame, valid: pl.DataFrame, t: pl.DataFrame, score: np.ndarray, c: Checker) -> None:
    say("\n[7] Nhãn trễ làm sai lệch việc đo như thế nào")
    y = valid["is_fraud"].to_numpy()
    end_day = (t["paid_at"].max() - START).total_seconds() / 86400
    asof = known_label(valid.select("trip_id", "is_fraud", "label_known_at"), end_day)["known_fraud"].to_numpy()
    say(f"valid có {y.sum():,} chuyến gian lận thật; tại thời điểm dữ liệu kết thúc (ngày {end_day:.1f}) mới biết {asof.sum():,} ({asof.sum() / y.sum():.1%})")
    pr_final, pr_asof = metrics.average_precision(y, score), metrics.average_precision(asof, score)
    cf_f, cf_a = metrics.confusion(y, score, 0.5), metrics.confusion(asof, score, 0.5)
    say(f"{'nhãn dùng để đo':24s}{'PR-AUC':>8s}{'precision@0.5':>15s}{'recall@0.5':>12s}")
    say(f"{'nhãn cuối cùng':24s}{pr_final:8.3f}{cf_f.precision:15.3f}{cf_f.recall:12.3f}")
    say(f"{'nhãn biết đến lúc đó':24s}{pr_asof:8.3f}{cf_a.precision:15.3f}{cf_a.recall:12.3f}")

    vb = valid.filter(pl.col("paid_day") >= 50)
    yb, xb = vb["is_fraud"].to_numpy(), x_of(vb)
    pre40 = t.filter(pl.col("paid_day") < CUTOFF_DAY)
    def fit_eval(df: pl.DataFrame, y_train: np.ndarray) -> float:
        return metrics.average_precision(yb, fit_scorer(x_of(df), y_train).predict_proba(xb)[:, 1])
    k_train = known_label(train.select("trip_id", "is_fraud", "label_known_at"), CUTOFF_DAY)["known_fraud"].to_numpy()
    k_pre40 = known_label(pre40.select("trip_id", "is_fraud", "label_known_at"), CUTOFF_DAY)["known_fraud"].to_numpy()
    res = {
        "train < ngày 30, nhãn cuối": fit_eval(train, train["is_fraud"].to_numpy()),
        "train < ngày 30, nhãn biết ở ngày 40": fit_eval(train, k_train),
        "train < ngày 40, nhãn cuối": fit_eval(pre40, pre40["is_fraud"].to_numpy()),
        "train < ngày 40, nhãn biết ở ngày 40": fit_eval(pre40, k_pre40),
    }
    say("PR-AUC trên valid ngày 50 trở đi (nhãn cuối), khi huấn luyện bằng:")
    for name, v in res.items():
        say(f"  {name:38s}{v:8.3f}")
    c.check("nhãn chưa chín làm PR-AUC đo được thấp hơn hẳn thực tế", pr_asof < pr_final - 0.1, (pr_asof, pr_final))
    c.check("nhãn chưa chín làm precision đo được thấp hơn thực tế", cf_a.precision < cf_f.precision - 0.1, (cf_a.precision, cf_f.precision))
    c.check("với bộ chấm điểm nền, thiếu nhãn ở đuôi train chỉ đổi PR-AUC dưới 0,05", max(res.values()) - min(res.values()) < 0.05, res)


# ---- [8] Parquet ---------------------------------------------------------------------------


def step_parquet(data: FraudData, c: Checker) -> None:
    say("\n[8] Ghi Parquet vào out/")
    OUT.mkdir(exist_ok=True)
    for name, df in (("events", data.events), ("labels", data.labels)):
        path = OUT / f"{name}.parquet"
        df.write_parquet(path)
        back = pl.read_parquet(path)
        say(f"out/{name}.parquet: {path.stat().st_size / 2**20:.1f} MiB; đọc lại giống hệt: {back.equals(df)}")
        c.check(f"{name}.parquet đọc lại giống hệt", back.equals(df))


# ---- Hằng đẳng thức của bộ sinh -----------------------------------------------------------


def step_generator(c: Checker) -> None:
    a = make_events(seed=7, n_trips=6_000)
    b = make_events(seed=7, n_trips=6_000)
    d = make_events(seed=8, n_trips=6_000)
    c.check("cùng seed cho đúng cùng dữ liệu", a.events.equals(b.events) and a.labels.equals(b.labels))
    c.check("khác seed cho dữ liệu khác", not a.events.equals(d.events))
    c.check("n_trips nhỏ vẫn đủ năm loại", set(a.labels["fraud_type"].unique()) == set(FRAUD_TYPES))


def step_identity(t: pl.DataFrame, c: Checker) -> None:
    num = t["customer_id"].str.slice(3).cast(pl.Int64).to_numpy().astype(float)
    auc = metrics.roc_auc(t["is_fraud"].to_numpy(), num)
    say(f"ROC-AUC khi dùng số thứ tự trong customer_id làm điểm gian lận: {auc:.3f} (0.5 là không biết gì)")
    c.check("số thứ tự trong ID khách không dự đoán được gian lận", abs(auc - 0.5) < 0.03, auc)


def run() -> int:
    c = Checker()
    data, _ = step_dataset(c)
    labels = data.labels
    t = trip_table(data.events).join(labels, on="trip_id", how="inner")
    c.check("bảng chuyến đủ 200.000 dòng", t.height == N_TRIPS)
    c.check("booked < completed < paid trong mọi chuyến",
            t.filter((pl.col("booked_at") >= pl.col("completed_at")) | (pl.col("completed_at") >= pl.col("paid_at"))).height == 0)
    step_identity(t, c)
    step_signals(t, c)
    step_label_delay(t, c)
    train, valid = step_split(t, c)
    score, _ = step_metrics(train, valid, c)
    step_cost(valid, score, c)
    step_delay_effect(train, valid, t, score, c)
    step_parquet(data, c)
    step_generator(c)

    ok = c.total - len(c.failures)
    if c.failures:
        print(f"FAIL {ok}/{c.total}")
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
