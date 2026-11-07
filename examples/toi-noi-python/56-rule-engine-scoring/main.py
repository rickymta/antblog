"""Bài 56 - Rule engine và điểm rủi ro: Strategy, cấu hình YAML và giải thích được.

Chạy từ gốc repository:
    uv run --project examples/toi-noi-python/56-rule-engine-scoring examples/toi-noi-python/56-rule-engine-scoring/main.py
    ... main.py --verify    # kiểm các bất biến in trong bài, in OK n/n

Không cần Docker. Dữ liệu là TỔNG HỢP (synthetic_fraud.py, giống bài 54, 55): mọi con số chỉ nói về bộ này,
điểm rủi ro là đầu vào cho người duyệt chứ không phải quyết định khóa tài khoản.
Lần đầu lab dựng đặc trưng từ event (khoảng nửa phút) và lưu out/table.parquet; xóa out/ để dựng lại.
"""

import os
import sys
import time
from dataclasses import replace
from pathlib import Path

import numpy as np
import polars as pl
from sklearn.ensemble import HistGradientBoostingClassifier
from sklearn.linear_model import LogisticRegression

import metrics
from dataset import CUTOFF_DAY, at_day
from features import FEATURES, day_of
from prepare import load_table, split
from rules import (
    AGGREGATORS,
    ConfigError,
    RuleEngine,
    load_config,
    parse_config,
    rule_flags,
    score_frame,
)

HERE = Path(__file__).parent
REVIEW_COST_VND = 8_000  # giả định bài 54: một chuyến gắn cờ nhầm tốn 5 phút người duyệt
HANDLING_COST_VND = 20_000  # giả định bài 54: xử lý một chuyến gian lận bị bỏ sót, cộng tiền chuyến
TYPES = ("ghost_trip", "gps_spoof", "promo_abuse", "collusion")


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


def recall_by_type(fraud_type: np.ndarray, flag: np.ndarray) -> dict[str, float]:
    return {t: float(flag[fraud_type == t].mean()) for t in TYPES}


# ---- [1] Dữ liệu ----------------------------------------------------------------------------


def step_data(c: Checker):
    say("[1] Dữ liệu: bảng 30 đặc trưng đúng theo thời điểm (bài 55) nối nhãn (bài 54)")
    t0 = time.perf_counter()
    table = load_table()
    train, valid = split(table)
    say(f"bảng chuyến: {table.height:,} dòng x {table.width} cột ({time.perf_counter() - t0:.1f} giây, gồm dựng đặc trưng nếu chưa có out/table.parquet)")
    for name, d in (("train (trước ngày 30)", train), ("valid (từ ngày 40)", valid)):
        say(f"{name}: {d.height:,} chuyến, {int(d['is_fraud'].sum()):,} gian lận ({d['is_fraud'].mean():.2%})")
    c.check("bảng có 200.000 chuyến và đủ 30 đặc trưng", table.height == 200_000 and all(f in table.columns for f in FEATURES))
    c.check("valid có tỉ lệ gian lận 1,5 đến 2 %", 0.015 < valid["is_fraud"].mean() < 0.02, valid["is_fraud"].mean())
    return table, train, valid


# ---- [2] Từng rule ---------------------------------------------------------------------------


def step_rules(cfg, train: pl.DataFrame, valid: pl.DataFrame, c: Checker):
    say("\n[2] Từng rule đứng một mình (điểm trong YAML so với precision đo trên train, bằng nhãn cuối và nhãn biết ở ngày 40)")
    ftr, fva = rule_flags(train, cfg.rules), rule_flags(valid, cfg.rules)
    ytr, yva = train["is_fraud"].to_numpy(), valid["is_fraud"].to_numpy()
    yk = train.select((pl.col("is_fraud") & (pl.col("label_known_at") <= pl.lit(at_day(CUTOFF_DAY)))).alias("k"))["k"].to_numpy()
    say(f"{'rule':27s}{'điểm':>5s}{'train: bắn':>11s}{'precision':>10s}{'(nhãn ngày 40)':>15s}{'valid: bắn':>11s}{'precision':>10s}{'recall':>8s}")
    above, over_known = [], []
    for s_ in cfg.rules:
        a, b = ftr[s_.id].to_numpy(), fva[s_.id].to_numpy()
        p_tr = (a & ytr).sum() / max(1, a.sum())
        p_k = (a & yk).sum() / max(1, a.sum())
        p_va = (b & yva).sum() / max(1, b.sum())
        r_va = (b & yva).sum() / yva.sum()
        say(f"{s_.id:27s}{s_.points:5d}{int(a.sum()):11,d}{p_tr:10.3f}{p_k:15.3f}{int(b.sum()):11,d}{p_va:10.3f}{r_va:8.3f}")
        if s_.points > 100 * p_tr + 10:
            above.append((s_.id, s_.points, round(p_tr, 3)))
        if s_.points > 100 * p_k:
            over_known.append(s_.id)
        c.check(f"rule {s_.id} có bắn trên train và valid", a.sum() >= 20 and b.sum() >= 5, (int(a.sum()), int(b.sum())))
    say(f"điểm vượt 100 x precision theo nhãn biết ở ngày {CUTOFF_DAY}: {len(over_known)}/{len(cfg.rules)} rule; bảng chọn điểm bằng nhãn cuối cùng")
    c.check("điểm của mỗi rule không vượt 100 x precision đo trên train quá 10 điểm", not above, above)
    c.check("mọi rule có points không quá 95 (trần: không rule nào được tự coi là chắc chắn tuyệt đối)", all(s_.points <= 95 for s_ in cfg.rules), [s_.id for s_ in cfg.rules if s_.points > 95])
    n_hit = fva.select(pl.sum_horizontal(pl.all()).alias("n"))["n"].to_numpy()
    say("\nsố rule cùng bắn trên một chuyến (valid):")
    say(f"{'số rule':>8s}{'chuyến':>9s}{'gian lận':>10s}{'tỉ lệ':>8s}")
    rates = {}
    for k in range(0, 4):
        m = n_hit == k if k < 3 else n_hit >= k
        rates[k] = yva[m].mean()
        say(f"{(str(k) if k < 3 else '>= 3'):>8s}{int(m.sum()):9,d}{int(yva[m].sum()):10,d}{rates[k]:8.1%}")
    c.check("càng nhiều rule cùng bắn càng chắc gian lận", rates[0] < rates[1] < rates[2] < rates[3], rates)
    return ftr, fva


# ---- [3] Chọn ngưỡng của một rule -------------------------------------------------------------


def step_threshold(table: pl.DataFrame, train: pl.DataFrame, valid: pl.DataFrame, c: Checker) -> None:
    say("\n[3] Chọn ngưỡng của một rule: đo trên train, báo cáo trên valid")
    say(f"{'điều kiện':26s}{'train: bắn':>11s}{'precision':>10s}{'recall':>8s}{'valid: bắn':>11s}{'precision':>10s}{'recall':>8s}")
    res = {}
    for expr_text, expr in [
        ("dev_customers_24h >= 2", pl.col("dev_customers_24h") >= 2),
        ("dev_customers_24h >= 3", pl.col("dev_customers_24h") >= 3),
        ("dev_customers_24h >= 4", pl.col("dev_customers_24h") >= 4),
        ("dev_newuser_7d >= 1", pl.col("dev_newuser_7d") >= 1),
        ("dev_newuser_7d >= 2", pl.col("dev_newuser_7d") >= 2),
    ]:
        row = [f"{expr_text:26s}"]
        for d in (train, valid):
            flag = d.select(expr.alias("f"))["f"].to_numpy()
            y = d["is_fraud"].to_numpy()
            tp = int((flag & y).sum())
            prec = tp / max(1, int(flag.sum()))
            row.append(f"{int(flag.sum()):11,d}{prec:10.3f}{tp / y.sum():8.3f}")
            res[(expr_text, d is train)] = prec
        say("".join(row))
    # khởi động nguội: số chuyến đầu của khách giảm theo thời gian (quần thể khách gần như cố định trong bộ sinh)
    say("\nchuyến SẠCH theo giai đoạn (ngày tính từ đầu dữ liệu; ngày 30 đến 40 không thuộc train hay valid):")
    say(f"{'giai đoạn':>12s}{'chuyến sạch':>13s}{'chuyến đầu của khách':>22s}{'có NEWUSER50':>14s}{'tỉ lệ':>8s}")
    day_all = table.select(day_of("paid_at").alias("d"))["d"].to_numpy()
    clean = ~table["is_fraud"].to_numpy()
    first = table["cust_prior_trips"].to_numpy() == 0
    new = table["has_newuser_promo"].to_numpy() == 1
    buckets = {}
    for lo, hi in ((0, 10), (10, 20), (20, 30), (30, 40), (40, 50), (50, 60)):
        m = (day_all >= lo) & (day_all < hi) & clean
        n_first, n_first_new = int((m & first).sum()), int((m & first & new).sum())
        buckets[(lo, hi)] = (n_first, n_first_new / max(1, n_first))
        say(f"{f'{lo} đến {hi}':>12s}{int(m.sum()):13,d}{n_first:22,d}{n_first_new:14,d}{n_first_new / max(1, n_first):8.1%}")
    c.check("số chuyến đầu của khách ở 10 ngày đầu gấp trên 3 lần ở 10 ngày cuối (quần thể khách gần cố định)", buckets[(0, 10)][0] > 3 * buckets[(50, 60)][0], buckets)
    c.check("tỉ lệ NEWUSER50 trong chuyến đầu của khách sạch gần như không đổi (10 đến 22 %) ở mọi giai đoạn", all(0.10 < v[1] < 0.22 for v in buckets.values()), buckets)
    # một ngưỡng mà precision đo trên train không cho biết gì về valid
    gap = res[("dev_newuser_7d >= 1", False)] - res[("dev_newuser_7d >= 1", True)]
    c.check("dev_newuser_7d >= 1 lệch xa giữa train và valid (cao hơn ít nhất 0,2)", gap > 0.2, gap)
    c.check("dev_newuser_7d >= 2 có precision từ 0,85 trở lên ở cả train và valid", res[("dev_newuser_7d >= 2", True)] >= 0.85 and res[("dev_newuser_7d >= 2", False)] >= 0.85, res)
    c.check("nâng ngưỡng số khách trên thiết bị làm precision tăng, recall giảm",
            res[("dev_customers_24h >= 2", True)] < res[("dev_customers_24h >= 3", True)] < res[("dev_customers_24h >= 4", True)])


# ---- [4] Ba strategy gộp điểm ------------------------------------------------------------------


def threshold_report(y: np.ndarray, score: np.ndarray, amount: np.ndarray, day: np.ndarray):
    """Chọn ngưỡng theo chi phí trên nửa đầu valid (ngày 40 đến 50), báo cáo trên nửa sau (như bài 54)."""
    half = day < 50
    cost_fn = amount.astype(float) + HANDLING_COST_VND
    thr, _ = metrics.best_threshold(y[half], score[half], cost_fn[half], REVIEW_COST_VND)
    cf = metrics.confusion(y[~half], score[~half], thr)
    cost = metrics.total_cost(y[~half], score[~half], thr, cost_fn[~half], REVIEW_COST_VND)
    return thr, cf, cost, half


def step_aggregators(cfg, fva: pl.DataFrame, valid: pl.DataFrame, c: Checker):
    say("\n[4] Ba strategy gộp điểm trên cùng 11 rule (valid)")
    y = valid["is_fraud"].to_numpy()
    amount = valid["amount_vnd"].to_numpy()
    day = valid.select(day_of("paid_at").alias("d"))["d"].to_numpy()
    k = int(0.01 * len(y))
    say(f"{'aggregator':10s}{'PR-AUC':>8s}{'ROC-AUC':>9s}{'số mức điểm':>12s}{'top 1% bắt':>12s}{'ngưỡng chi phí':>16s}{'gắn cờ':>8s}{'TP':>5s}{'FP':>6s}{'chi phí (VND)':>15s}")
    scores, out = {}, {}
    for name in AGGREGATORS:
        s = score_frame(fva, cfg.rules, name).to_numpy()
        scores[name] = s
        top = np.argsort(-s, kind="stable")[:k]
        thr, cf, cost, _ = threshold_report(y, s, amount, day)
        out[name] = (metrics.average_precision(y, s), thr, cf, cost)
        say(f"{name:10s}{out[name][0]:8.3f}{metrics.roc_auc(y, s):9.3f}{len(set(s.tolist())):12d}{int(y[top].sum()):12,d}{thr:16.1f}{cf.tp + cf.fp:8,d}{cf.tp:5d}{cf.fp:6d}{cost:15,.0f}")
    say(f"(top 1% = {k} chuyến điểm cao nhất; chi phí tính trên nửa sau valid, ngày 50 trở đi, với {int((day >= 50).sum()):,} chuyến)")
    s_sum, s_max, s_nor = scores["sum"], scores["max"], scores["noisy_or"]
    n_hit = fva.select(pl.sum_horizontal(pl.all()).alias("n"))["n"].to_numpy()
    c.check("sum >= noisy_or >= max trên từng chuyến", bool(np.all(s_sum >= s_nor - 1e-9) and np.all(s_nor >= s_max - 1e-9)))
    c.check("chuyến có đúng một rule bắn: ba aggregator cho cùng điểm", bool(np.all(np.isclose(s_sum[n_hit == 1], s_max[n_hit == 1]) & np.isclose(s_nor[n_hit == 1], s_max[n_hit == 1]))))
    c.check("chuyến có từ hai rule bắn: sum và max khác nhau ở ít nhất một chuyến", bool(np.any(s_sum[n_hit >= 2] > s_max[n_hit >= 2] + 1e-9)))
    bins = [("0", s_nor == 0), ("(0, 20)", (s_nor > 0) & (s_nor < 20)), ("[20, 40)", (s_nor >= 20) & (s_nor < 40)), ("[40, 60)", (s_nor >= 40) & (s_nor < 60)),
            ("[60, 70)", (s_nor >= 60) & (s_nor < 70)), ("[70, 80)", (s_nor >= 70) & (s_nor < 80)), ("[80, 100]", s_nor >= 80)]
    say("\nđiểm noisy_or không phải xác suất: tỉ lệ gian lận thật theo khoảng điểm trên valid")
    say(f"{'khoảng điểm':>12s}{'chuyến':>9s}{'gian lận':>10s}{'tỉ lệ':>8s}")
    for name, m in bins:
        if m.any():
            say(f"{name:>12s}{int(m.sum()):9,d}{int(y[m].sum()):10,d}{y[m].mean():8.1%}")
    half = day < 50
    cost_fn = amount.astype(float) + HANDLING_COST_VND
    cost_none = metrics.total_cost(y[~half], s_nor[~half], np.inf, cost_fn[~half], REVIEW_COST_VND)
    c.check("rule engine rẻ hơn không làm gì", out["noisy_or"][3] < cost_none, (out["noisy_or"][3], cost_none))
    thr_nor = out["noisy_or"][1]
    c.check("review_at trong YAML nằm trong 20 điểm của ngưỡng chọn theo chi phí (noisy_or)", abs(thr_nor - cfg.review_at) <= 20, (thr_nor, cfg.review_at))
    c.check("PR-AUC của rule engine gấp trên 20 lần mốc cơ sở (tỉ lệ gian lận)", out["noisy_or"][0] > 20 * y.mean(), out["noisy_or"][0])
    say(f"không làm gì: chi phí {cost_none:,.0f} VND trên cùng nửa sau")
    late = day >= 50
    fl_late = s_nor[late] >= cfg.review_at
    y_late = y[late]
    tp_late = int((fl_late & y_late).sum())
    say(f"dải review (noisy_or, điểm >= {cfg.review_at:.0f}) trên nửa sau valid, nửa không dùng để chọn ngưỡng: gắn cờ {int(fl_late.sum()):,} trong {int(late.sum()):,} chuyến, "
        f"bắt {tp_late} trong {int(y_late.sum())} gian lận, precision {tp_late / max(1, fl_late.sum()):.3f}, recall {tp_late / y_late.sum():.3f}")
    c.check("trên nửa sau valid, dải review có precision cao hơn 10 lần tỉ lệ gian lận nền", tp_late / max(1, fl_late.sum()) > 10 * y_late.mean(), (tp_late, int(fl_late.sum())))
    return scores, out, cost_none


# ---- [5] Giải thích ---------------------------------------------------------------------------


def step_explain(engine: RuleEngine, valid: pl.DataFrame, c: Checker) -> None:
    say("\n[5] Lý do kèm điểm: những chuyến của valid, qua engine từng chuyến")
    rows = list(valid.select("trip_id", "fraud_type", *FEATURES).iter_rows(named=True))
    verdicts = [engine.score({f: r[f] for f in FEATURES}) for r in rows]
    picks = []
    for t in ("ghost_trip", "promo_abuse", "gps_spoof"):
        best = max((i for i, r in enumerate(rows) if r["fraud_type"] == t), key=lambda i: (len(verdicts[i].hits), -i))
        picks.append(best)
    clean_flagged = next(i for i, r in enumerate(rows) if r["fraud_type"] == "none" and verdicts[i].band == "review")
    missed = next(i for i, r in enumerate(rows) if r["fraud_type"] == "collusion" and verdicts[i].score == 0)
    for i in [*picks, clean_flagged, missed]:
        v = verdicts[i]
        say(f"{rows[i]['trip_id']}  fraud_type={rows[i]['fraud_type']}  điểm {v.score:.1f}  dải {v.band}")
        for line in v.reasons or ["(không rule nào bắn: engine không có gì để nói)"]:
            say(f"    - {line}")
    flagged = [v for v in verdicts if v.band != "none"]
    c.check("mọi chuyến vào dải watch/review đều có ít nhất một lý do", all(v.reasons for v in flagged))
    c.check("số lý do bằng số rule đã bắn", all(len(v.reasons) == len(v.hits) for v in verdicts))
    c.check("chuyến điểm 0 không có lý do", all(not v.reasons for v in verdicts if v.score == 0))
    return rows, verdicts


# ---- [6] Khi rule đủ, khi cần ML --------------------------------------------------------------


def step_vs_ml(cfg, ftr, fva, train: pl.DataFrame, valid: pl.DataFrame, scores, out, c: Checker) -> None:
    say("\n[6] Rule đủ đến đâu: so với một mô hình học trên cùng train")
    ytr, y = train["is_fraud"].to_numpy(), valid["is_fraud"].to_numpy()
    ftype = valid["fraud_type"].to_numpy()
    amount = valid["amount_vnd"].to_numpy()
    day = valid.select(day_of("paid_at").alias("d"))["d"].to_numpy()

    # (a) trọng số tay so với trọng số học từ chính các rule
    lr = LogisticRegression(C=10, max_iter=2000).fit(ftr.to_numpy().astype(float), ytr)
    p_lr = lr.predict_proba(fva.to_numpy().astype(float))[:, 1]
    pr_hand, pr_lr = out["noisy_or"][0], metrics.average_precision(y, p_lr)
    say(f"cùng 11 rule, trọng số tay (noisy_or): PR-AUC {pr_hand:.3f}; trọng số học bằng hồi quy logistic trên cờ rule: {pr_lr:.3f}")
    ids = [s_.id for s_ in cfg.rules]
    a_s, b_j, b_p = (ftr[i].to_numpy() for i in ("speed_impossible", "gps_jump_impossible", "pickup_far"))
    say(f"trên train: speed_impossible bắn {int(a_s.sum())} chuyến, trong đó {int((a_s & b_j).sum())} cũng bị gps_jump_impossible bắn và {int((a_s & b_p).sum())} cũng bị pickup_far bắn")
    coefs = {}
    for dropped in ((), ("gps_jump_impossible",), ("pickup_far",), ("gps_jump_impossible", "pickup_far")):
        keep = [i for i in ids if i not in dropped]
        m_ = LogisticRegression(C=10, max_iter=2000).fit(ftr.select(keep).to_numpy().astype(float), ytr)
        coefs[dropped] = float(m_.coef_[0][keep.index("speed_impossible")])
        say(f"  hệ số của speed_impossible khi bỏ {', '.join(dropped) if dropped else 'không rule nào'}: {coefs[dropped]:+.2f}")
    c.check("speed_impossible nằm gần trọn trong hai rule GPS khác (ít nhất 90 % số chuyến)", (a_s & (b_j | b_p)).sum() >= 0.9 * a_s.sum(), (int(a_s.sum()), int((a_s & (b_j | b_p)).sum())))
    c.check("bỏ cả hai rule trùng thì hệ số của speed_impossible đổi dấu thành dương", coefs[("gps_jump_impossible", "pickup_far")] > 0 and coefs[()] < coefs[("gps_jump_impossible", "pickup_far")], coefs)
    neg = [(s.id, round(float(w), 2)) for s, w in zip(cfg.rules, lr.coef_[0], strict=True) if w < 0]
    say(f"hệ số âm của hồi quy (rule làm điểm GIẢM dù đứng một mình precision cao): {neg}")

    # (b) mô hình trên 30 đặc trưng
    t0 = time.perf_counter()
    xtr_np, xva_np = train.select(FEATURES).to_numpy(), valid.select(FEATURES).to_numpy()
    hgb = HistGradientBoostingClassifier(max_iter=150, random_state=0).fit(xtr_np, ytr)
    p_ml = hgb.predict_proba(xva_np)[:, 1]
    say(f"HistGradientBoosting (tối đa 150 vòng, dừng sớm ở {hgb.n_iter_}; dừng sớm dùng 10 % train tách ngẫu nhiên; 30 đặc trưng) huấn luyện trong {time.perf_counter() - t0:.0f} giây; load average 1/5/15 phút: {load()}")
    pr_ml = metrics.average_precision(y, p_ml)
    say(f"PR-AUC trên valid: rule engine {pr_hand:.3f}, mô hình {pr_ml:.3f} (mốc cơ sở {y.mean():.4f})")
    hgb1 = HistGradientBoostingClassifier(max_iter=150, max_depth=1, random_state=0).fit(xtr_np, ytr)
    pr_d1 = metrics.average_precision(y, hgb1.predict_proba(xva_np)[:, 1])
    hgb_flags = HistGradientBoostingClassifier(max_iter=150, random_state=0).fit(ftr.to_numpy().astype(float), ytr)
    pr_flags = metrics.average_precision(y, hgb_flags.predict_proba(fva.to_numpy().astype(float))[:, 1])
    say(f"đổi một biến: cùng HistGradientBoosting nhưng chỉ cộng tính (max_depth=1, không tương tác giữa đặc trưng): PR-AUC {pr_d1:.3f}; "
        f"học trên đúng 11 cờ rule thay vì 30 đặc trưng: {pr_flags:.3f}")

    # (c) cùng công suất người duyệt: dải review của YAML gắn cờ bao nhiêu chuyến thì mô hình lấy đúng chừng ấy chuyến điểm cao nhất
    s = scores["noisy_or"]
    flag_rule = s >= cfg.review_at
    n_flag = int(flag_rule.sum())
    flag_ml = np.zeros(len(y), dtype=bool)
    flag_ml[np.argsort(-p_ml, kind="stable")[:n_flag]] = True
    say(f"\ncùng công suất người duyệt, toàn valid: dải review (điểm >= {cfg.review_at:.0f}) gắn cờ {n_flag:,} chuyến ({n_flag / len(y):.1%}); mô hình lấy {n_flag:,} chuyến điểm cao nhất")
    say(f"{'':12s}{'rule engine':>14s}{'mô hình':>10s}")
    say(f"{'bắt được':12s}{int((flag_rule & y).sum()):14d}{int((flag_ml & y).sum()):10d}   (trong {int(y.sum()):,} chuyến gian lận)")
    say(f"{'precision':12s}{(flag_rule & y).sum() / n_flag:14.3f}{(flag_ml & y).sum() / n_flag:10.3f}")
    r_rule, r_ml = recall_by_type(ftype, flag_rule), recall_by_type(ftype, flag_ml)
    say("recall theo loại:")
    for t in TYPES:
        say(f"  {t:12s}{r_rule[t]:12.1%}{r_ml[t]:10.1%}   ({int((ftype == t).sum())} chuyến)")
    # cùng số chuyến sạch bị gắn cờ nhầm: mô hình lấy nhiều chuyến nhất mà FP chưa vượt FP của rule
    fp_rule = int((flag_rule & ~y).sum())
    order = np.argsort(-p_ml, kind="stable")
    k_same = int(np.searchsorted(np.cumsum(~y[order]), fp_rule, side="right"))
    same_fp = np.zeros(len(y), dtype=bool)
    same_fp[order[:k_same]] = True
    say(f"\ncùng số chuyến sạch bị gắn cờ nhầm (FP = {fp_rule}): rule engine bắt {int((flag_rule & y).sum())}; mô hình gắn cờ {k_same:,} chuyến, bắt {int((same_fp & y).sum())}; "
        "recall theo loại: " + ", ".join(f"{t} {v:.1%}" for t, v in recall_by_type(ftype, same_fp).items()))
    say("PR-AUC theo loại (loại đó so với chuyến sạch; lúc chấm điểm hệ thống không biết loại):")
    ap = {}
    for t in TYPES:
        sel = (ftype == t) | ~y
        ap[t] = (metrics.average_precision(ftype[sel] == t, s[sel]), metrics.average_precision(ftype[sel] == t, p_ml[sel]))
        say(f"  {t:12s} rule engine {ap[t][0]:.3f}  mô hình {ap[t][1]:.3f}")
    # chi phí: ngưỡng của mỗi bên chọn trên nửa đầu valid, báo cáo trên nửa sau (như bài 54)
    _, cf_rule, cost_rule, _ = threshold_report(y, s, amount, day)
    thr_ml, cf_ml, cost_ml, _ = threshold_report(y, p_ml, amount, day)
    say(f"chi phí giả định trên nửa sau valid: rule engine gắn cờ {cf_rule.tp + cf_rule.fp:,} (TP {cf_rule.tp}, FP {cf_rule.fp}) tốn {cost_rule:,.0f} VND; "
        f"mô hình (ngưỡng {thr_ml:.3f}) gắn cờ {cf_ml.tp + cf_ml.fp:,} (TP {cf_ml.tp}, FP {cf_ml.fp}) tốn {cost_ml:,.0f} VND")

    c.check("mô hình học thắng rule engine tay ở PR-AUC với biên lớn (>= 0,10)", pr_ml > pr_hand + 0.10, (pr_ml, pr_hand))
    c.check("mô hình chỉ cộng tính (max_depth=1) kém mô hình đầy đủ ít nhất 0,03 PR-AUC", pr_ml - pr_d1 >= 0.03, (pr_ml, pr_d1))
    c.check("ở cùng FP, mô hình bắt nhiều hơn rule engine", int((same_fp & y).sum()) > int((flag_rule & y).sum()))
    c.check("trọng số học không hơn trọng số tay quá 0,1 PR-AUC (11 rule đã nắm gần hết thông tin của chúng)", pr_lr < pr_hand + 0.1, (pr_lr, pr_hand))
    c.check("rule engine bỏ sót phần lớn collusion (recall dưới 45 %)", r_rule["collusion"] < 0.45, r_rule["collusion"])
    c.check("mô hình bắt collusion nhiều gấp ít nhất 2 lần rule engine ở cùng công suất", r_ml["collusion"] > 2 * max(r_rule["collusion"], 0.01), (r_ml["collusion"], r_rule["collusion"]))
    c.check("rule engine bắt ít nhất 60 % gps_spoof", r_rule["gps_spoof"] >= 0.6, r_rule)
    c.check("AP theo loại của collusion: mô hình gấp ít nhất 3 lần rule engine", ap["collusion"][1] > 3 * ap["collusion"][0], ap)


# ---- [7] Cấu hình YAML ----------------------------------------------------------------------


BROKEN = """
version: 1
review_at: 60
watch_at: 80
aggregate: average
rules:
  - id: speed_impossible
    point: 70
    when: ["speed_kmh > 120"]
    reason: "tốc độ {speed_kmh:.0f} km/h"
  - id: speed_impossible
    points: 150
    when: ["speed_kph > 120", "dev_trips_1h >=> 2"]
    reason: "{dev_trips_1h} chuyến, {speed_kmh}"
"""


def step_config(cfg, fva: pl.DataFrame, valid: pl.DataFrame, c: Checker) -> None:
    say("\n[7] Cấu hình YAML: kiểm khi nạp, và đổi hành vi không sửa code")
    try:
        parse_config(BROKEN)
        errors = []
    except ConfigError as e:
        errors = e.errors
    say(f"một file cấu hình có nhiều lỗi: ConfigError báo {len(errors)} lỗi một lượt:")
    for e in errors:
        say(f"  - {e}")
    text = (HERE / "rules.yaml").read_text(encoding="utf-8")
    cfg_max = parse_config(text.replace("aggregate: noisy_or", "aggregate: max").replace("review_at: 40", "review_at: 60"))
    y = valid["is_fraud"].to_numpy()
    say(f"đổi hai dòng trong YAML (aggregate: noisy_or -> max, review_at: {cfg.review_at:.0f} -> {cfg_max.review_at:.0f}), không đụng code; dải review trên valid:")
    flagged = {}
    say(f"  {'aggregate / review_at':24s}{'gắn cờ':>8s}{'bắt':>6s}{'precision':>10s}")
    for agg in ("noisy_or", "max"):
        sc = score_frame(fva, cfg.rules, agg).to_numpy()
        for at in (40, 60):
            fl = sc >= at
            label = f"{agg} / {at}" + ("  (gốc)" if (agg, at) == ("noisy_or", 40) else "  (đã đổi)" if (agg, at) == ("max", 60) else "")
            flagged[label] = int(fl.sum())
            say(f"  {label:24s}{int(fl.sum()):8,d}{int((fl & y).sum()):6d}{(fl & y).sum() / max(1, fl.sum()):10.3f}")
    dup = "version: 1\nversion: 2\nreview_at: 60\nwatch_at: 30\naggregate: sum\nrules: []\n"
    try:
        parse_config(dup)
        dup_err = "không lỗi"
    except ConfigError as e:
        dup_err = "; ".join(e.errors)
    say(f"khóa trùng (version hai lần): {dup_err}")
    c.check("khóa trùng trong YAML bị từ chối", "trùng" in dup_err, dup_err)
    c.check("cấu hình hợp lệ nạp được 11 rule", len(cfg.rules) == 11)
    c.check("đổi YAML đổi số chuyến vào dải review mà không sửa code", len(set(flagged.values())) >= 3, flagged)
    c.check("cấu hình sai bị từ chối với đủ nhóm lỗi",
            len(errors) >= 8 and any("aggregate" in e for e in errors) and any("khóa không biết" in e for e in errors)
            and any("id trùng" in e for e in errors) and any("speed_kph" in e for e in errors) and any(">=>" in e for e in errors)
            and any("points" in e for e in errors) and any("watch_at" in e for e in errors) and any("reason dùng" in e for e in errors), errors)
    c.check("thẻ YAML nguy hiểm bị từ chối (SafeLoader), không chạy code",
            _rejects("version: !!python/object/apply:os.getcwd []\n"))
    return cfg_max


def _rejects(text: str) -> bool:
    try:
        parse_config(text)
    except ConfigError as e:
        return any("YAML" in m for m in e.errors)
    return False


# ---- [8] Đường theo lô khớp đường từng chuyến, và độ trễ ---------------------------------------


def step_parity(cfg, engine: RuleEngine, rows, verdicts, fva: pl.DataFrame, c: Checker) -> None:
    say("\n[8] Một định nghĩa, hai đường chạy: từng chuyến (Python) và theo lô (polars)")
    for name in AGGREGATORS:
        eng = RuleEngine.from_config(replace(cfg, aggregate=name))
        py = np.array([eng.score({f: r[f] for f in FEATURES}).score for r in rows])
        pl_score = score_frame(fva, cfg.rules, name).to_numpy()
        diff = float(np.abs(py - pl_score).max())
        say(f"aggregate={name:9s}: {len(rows):,} chuyến valid, chênh lớn nhất giữa hai đường {diff:.2e}")
        c.check(f"đường Python và đường polars khớp (aggregate={name})", diff < 1e-9, diff)
    sample = [{f: r[f] for f in FEATURES} for r in rows[:20_000]]
    t0 = time.perf_counter()
    for f in sample:
        engine.score(f)
    dt = time.perf_counter() - t0
    say(f"engine từng chuyến: {len(sample):,} chuyến trong {dt:.2f} giây = {dt / len(sample) * 1e6:.0f} micro giây mỗi chuyến; load average 1/5/15 phút: {load()}")
    say("(số này chỉ để biết cỡ: máy đang bận, và chưa tính việc lấy đặc trưng, thứ chiếm phần lớn ngân sách ở bài 59)")
    bands = {b: sum(1 for v in verdicts if v.band == b) for b in ("none", "watch", "review")}
    say(f"dải trên valid: {bands}")
    c.check("engine chỉ trả ba dải none, watch, review (không có dải tự khóa) và dùng cả ba", {v.band for v in verdicts} == {"none", "watch", "review"})


def main() -> int:
    verify = "--verify" in sys.argv
    c = Checker()
    cfg = load_config(HERE / "rules.yaml")
    engine = RuleEngine.from_config(cfg)
    table, train, valid = step_data(c)
    ftr, fva = step_rules(cfg, train, valid, c)
    step_threshold(table, train, valid, c)
    scores, out, _ = step_aggregators(cfg, fva, valid, c)
    rows, verdicts = step_explain(engine, valid, c)
    step_vs_ml(cfg, ftr, fva, train, valid, scores, out, c)
    step_config(cfg, fva, valid, c)
    step_parity(cfg, engine, rows, verdicts, fva, c)
    ok = c.total - len(c.failures)
    if c.failures:
        print(f"\nKiểm chứng: FAIL {ok}/{c.total}")
        return 1
    if verify:
        print(f"\nKiểm chứng: OK {ok}/{c.total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
