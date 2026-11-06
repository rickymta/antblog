"""Bài 58 - Đồ thị networkx: gắn cờ nhóm khách - tài xế - thiết bị cho người duyệt.

Chạy từ gốc repository:
    uv run --project examples/toi-noi-python/58-networkx-graph-cau-ket \
        examples/toi-noi-python/58-networkx-graph-cau-ket/main.py
    ... main.py --verify    # kiểm các bất biến in trong bài, in OK n/n

Không cần Docker, không cần mạng sau lần cài đầu. Dữ liệu là TỔNG HỢP (synthetic_fraud.py, giống bài 54); mọi con số
chỉ nói về bộ này. Nhãn (fraud_type) chỉ dùng để ĐÁNH GIÁ và để huấn luyện bộ phân loại ở bước 5; `graph.py` dựng đồ thị
mà không đọc nhãn. Nhóm bị gắn cờ là đầu vào cho người duyệt, không phải kết luận cấu kết.
"""

import hashlib
import json
import math
import os
import statistics
import subprocess
import sys
import time
import tracemalloc
from datetime import timedelta
from itertools import product

import networkx as nx
import numpy as np
import polars as pl
from sklearn.ensemble import HistGradientBoostingClassifier
from sklearn.metrics import average_precision_score, roc_auc_score

import dataset
import graph as gr
from dataset import TRAIN_END_DAY, VALID_START_DAY
from features import FEATURES, add_features, day_of, trip_table
from synthetic_fraud import FRAUD_TYPES, START, FraudData, make_events

SEED = 42
N_TRIPS = 200_000
LAST_DAY = 59  # ngày cuối có chuyến thanh toán (60 ngày: 0 đến 59)
TUNE_FIRST_DAY = 7  # quét lưới luật nhóm trên ảnh chụp ngày 7 đến 29: đã có ít nhất 7 ngày lịch sử
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


def pct(x: float) -> str:
    return f"{x:.1%}"


def wilson(k: int, n: int, z: float = 1.96) -> tuple[float, float]:
    """Khoảng tin cậy Wilson 95 % cho tỉ lệ k/n."""
    p = k / n
    centre = (p + z * z / (2 * n)) / (1 + z * z / n)
    half = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / (1 + z * z / n)
    return centre - half, centre + half


def make_full(seed: int) -> tuple[FraudData, pl.DataFrame]:
    """Sinh dữ liệu theo seed rồi dựng bảng chuyến kèm 30 đặc trưng cửa sổ, nhãn cuối và paid_day, sắp theo paid_at."""
    data = make_events(seed=seed, n_trips=N_TRIPS)
    full = add_features(trip_table(data.events)).join(data.labels, on="trip_id").with_columns(paid_day=day_of()).sort("paid_at")
    return data, full


# ---- Chỉ số theo loại gian lận (cùng cách đo với bài 54 và 55) -----------------------------


def type_metrics(valid: pl.DataFrame, score: np.ndarray) -> dict:
    """PR-AUC tổng, ROC-AUC, và theo từng loại: PR-AUC (loại đó so với chuyến sạch) và recall@top 1 % điểm cao nhất."""
    y = valid["is_fraud"].to_numpy()
    top = np.zeros(len(score), dtype=bool)
    top[np.argsort(-score, kind="stable")[: int(0.01 * len(score))]] = True
    by = {}
    for name in FRAUD_TYPES[1:]:
        m = (valid["fraud_type"] == name).to_numpy()
        both = m | ~y
        by[name] = {"ap": float(average_precision_score(m[both], score[both])), "rec1": float(top[m].mean())}
    return {"pr": float(average_precision_score(y, score)), "roc": float(roc_auc_score(y, score)), "by": by}


def hgb() -> HistGradientBoostingClassifier:
    return HistGradientBoostingClassifier(max_iter=150, learning_rate=0.1, random_state=0)


# ---- [1] Dữ liệu và mốc tham chiếu ---------------------------------------------------------


def step_baselines(c: Checker):
    say("\n[1] Dữ liệu và mốc tham chiếu (cùng cách đo với bài 54, 55: train trước ngày 30, valid từ ngày 40)")
    t0 = time.perf_counter()
    data = make_events(seed=SEED, n_trips=N_TRIPS)
    say(f"make_events(seed={SEED}, n_trips={N_TRIPS:,}): {data.events.height:,} event, {data.labels.height:,} nhãn trong {time.perf_counter() - t0:.1f} giây")
    full = add_features(trip_table(data.events)).join(data.labels, on="trip_id").with_columns(paid_day=day_of()).sort("paid_at")
    train, valid = full.filter(pl.col("paid_day") < TRAIN_END_DAY), full.filter(pl.col("paid_day") >= VALID_START_DAY)
    y_va = valid["is_fraud"].to_numpy()
    n_col = int((valid["fraud_type"] == "collusion").sum())
    say(f"valid: {valid.height:,} chuyến, {int(y_va.sum()):,} gian lận ({pct(y_va.mean())}), trong đó collusion {n_col}")

    # Mốc 1: hồi quy logistic 9 đặc trưng theo từng chuyến (bài 54)
    base = dataset.trip_table(data.events).join(data.labels, on="trip_id")
    b_tr, b_va = base.filter(pl.col("paid_day") < TRAIN_END_DAY), base.filter(pl.col("paid_day") >= VALID_START_DAY)

    def x_of(df: pl.DataFrame) -> np.ndarray:
        return df.select(dataset.RAW_FEATURES).to_numpy()

    lr = dataset.fit_scorer(x_of(b_tr), b_tr["is_fraud"].to_numpy())
    m_lr = type_metrics(b_va, lr.predict_proba(x_of(b_va))[:, 1])
    # Mốc 2: HistGradientBoosting 30 đặc trưng cửa sổ (bài 55)
    t0 = time.perf_counter()
    model = hgb().fit(train.select(FEATURES).to_numpy(), train["is_fraud"].to_numpy())
    s_hgb = model.predict_proba(valid.select(FEATURES).to_numpy())[:, 1]
    m_hgb = type_metrics(valid, s_hgb)
    say(f"{'mô hình':44s}{'PR-AUC':>8s}{'ROC-AUC':>9s}{'collusion PR-AUC':>18s}{'collusion recall@1%':>21s}")
    say(f"{'hồi quy logistic, 9 đặc trưng theo từng chuyến':44s}{m_lr['pr']:8.3f}{m_lr['roc']:9.3f}{m_lr['by']['collusion']['ap']:18.3f}{'-':>21s}")
    say(f"{'HistGradientBoosting, 30 đặc trưng cửa sổ':44s}{m_hgb['pr']:8.3f}{m_hgb['roc']:9.3f}{m_hgb['by']['collusion']['ap']:18.3f}{pct(m_hgb['by']['collusion']['rec1']):>21s}")
    say(f"(huấn luyện HistGradientBoosting {time.perf_counter() - t0:.0f} giây; load average 1/5/15 phút: {load()})")
    say(f"HistGradientBoosting: tối đa 150 vòng, dừng sớm ở {model.n_iter_} (n_iter_); dừng sớm dùng 10 % train tách ngẫu nhiên")
    # Hai tạo tác của bộ sinh (không sửa synthetic_fraud.py): ảnh hưởng mọi valid của phần 07
    clean = full.filter(pl.col("fraud_type") == "none")
    early, late = clean.filter(pl.col("paid_day") < 10), clean.filter(pl.col("paid_day") >= 50)
    non_promo_fraud = full.filter(pl.col("is_fraud") & (pl.col("fraud_type") != "promo_abuse"))
    say(f"tạo tác của bộ sinh: chuyến sạch dùng NEWUSER50: ngày 0-9 {early['has_newuser_promo'].mean():.1%}, ngày 50-59 {late['has_newuser_promo'].mean():.1%} (quần thể khách gần như cố định); "
        f"chuyến gian lận không phải promo_abuse có mã khuyến mãi: {non_promo_fraud['has_promo'].mean():.1%}")
    c.check("valid có 1.141 chuyến gian lận cỡ bài 54 (khoảng 1.000 đến 1.300)", 1_000 < y_va.sum() < 1_300, int(y_va.sum()))
    c.check("tạo tác bộ sinh: tỉ lệ chuyến sạch dùng NEWUSER50 giảm hẳn theo thời gian", early["has_newuser_promo"].mean() > 3 * late["has_newuser_promo"].mean(), (early["has_newuser_promo"].mean(), late["has_newuser_promo"].mean()))
    c.check("mốc 1: hồi quy logistic gần như mù với collusion (PR-AUC dưới 0,1)", m_lr["by"]["collusion"]["ap"] < 0.1, m_lr["by"]["collusion"])
    c.check("mốc 2: 30 đặc trưng cửa sổ cho PR-AUC trên 0,8", m_hgb["pr"] > 0.8, m_hgb["pr"])
    return data, full, train, valid, m_lr, m_hgb


# ---- Sự thật nền (CHỈ để đánh giá, không đi vào đồ thị) ---------------------------------------


def ring_truth(full: pl.DataFrame) -> dict[str, int]:
    """Tài khoản -> số hiệu nhóm cấu kết thật: thành phần liên thông của đồ thị các chuyến collusion (khách - tài xế)."""
    rg = nx.Graph()
    rg.add_edges_from(full.filter(pl.col("fraud_type") == "collusion").select("customer_id", "driver_id").unique().iter_rows())
    comps = sorted(nx.connected_components(rg), key=lambda s: min(s))
    return {x: i for i, comp in enumerate(comps) for x in comp}


# ---- [2] Từ đồ thị thô đến nhóm -------------------------------------------------------------


def largest(g: nx.Graph) -> tuple[int, int]:
    comps = list(nx.connected_components(g))
    return len(comps), max((len(x) for x in comps), default=0)


def step_stages(full: pl.DataFrame, ring_of: dict[str, int], c: Checker):
    day = 50
    say(f"\n[2] Từ đồ thị thô đến nhóm: ảnh chụp đầu ngày {day}, cửa sổ {gr.WINDOW_DAYS} ngày")
    w = gr.window_slice(full, day)
    all_pairs = w.group_by("customer_id", "driver_id").agg(trips=pl.len())
    ring_cust = {x for x in w.filter(pl.col("fraud_type") == "collusion")["customer_id"].unique()}
    say(f"cửa sổ: {w.height:,} chuyến, {all_pairs.height:,} cặp khách-tài xế khác nhau, {len(ring_cust)} khách thuộc nhóm cấu kết thật (nhãn, chỉ để đánh giá)")
    say("cột «khách ring» đếm khách cấu kết thật nằm trong thành phần lớn nhất của giai đoạn đó")

    def row(name: str, g: nx.Graph):
        n_comp, big = largest(g)
        comp = max(nx.connected_components(g), key=len)
        say(f"{name:46s}{g.number_of_edges():9,d}{g.number_of_nodes():9,d}{n_comp:12,d}{big:10,d}{len(comp & ring_cust):12d}")
        return g.number_of_edges(), n_comp, big

    say(f"{'giai đoạn':46s}{'cạnh':>9s}{'nút':>9s}{'thành phần':>12s}{'lớn nhất':>10s}{'khách ring':>12s}")
    g1 = nx.Graph()
    g1.add_edges_from(all_pairs.select("customer_id", "driver_id").iter_rows())
    e1, n1, big1 = row("1. mọi cặp khách-tài xế", g1)
    pairs, shared = gr.edge_tables(w)
    g2 = nx.Graph()
    g2.add_edges_from(pairs.select("customer_id", "driver_id").iter_rows())
    e2, n2, big2 = row(f"2. chỉ giữ cặp từ {gr.STRONG_MIN_TRIPS} chuyến", g2)
    g3 = gr.build_graph(pairs, shared)
    e3, n3, big3 = row("3. thêm thiết bị dùng chung từ 2 tài khoản", g3)
    groups = gr.split_groups(g3)
    sizes = sorted((len(x) for x in groups), reverse=True)
    n_big = sum(len(x) > gr.MAX_GROUP_NODES for x in nx.connected_components(g3))
    say(f"4. tách {n_big} thành phần lớn hơn {gr.MAX_GROUP_NODES} nút bằng Louvain: {len(groups):,} nhóm, nhóm lớn nhất {sizes[0]} nút")
    c.check("đồ thị thô: một thành phần nuốt phần lớn khu vực (hơn 1.000 nút)", big1 > 1_000, big1)
    c.check("lọc cặp mạnh bằng polars giữ dưới 10 % số cạnh", e2 < 0.1 * e1, (e2, e1))
    c.check("sau khi lọc và thêm thiết bị vẫn còn thành phần trên 100 nút (cần tách)", big3 > 100, big3)
    c.check("Louvain tách mọi thành phần lớn: nhóm lớn nhất nhỏ hơn thành phần lớn nhất", sizes[0] < big3, (sizes[0], big3))
    return g3


# ---- [3] Đặc trưng nhóm: nhóm cấu kết khác nhóm bình thường ở đâu --------------------------------


def with_truth(groups: pl.DataFrame, ring_of: dict[str, int]) -> pl.DataFrame:
    """Gắn nhãn đánh giá cho nhóm (chỉ để đánh giá, không đi vào đồ thị).

    ring_share: tỉ lệ khách + tài xế của nhóm thuộc nhóm cấu kết thật; is_ring: từ 50 % (nhóm «thuần»);
    has_ring: chứa ít nhất 3 tài khoản cấu kết thật, trong đó có ít nhất 2 khách (người duyệt mở nhóm sẽ gặp nhóm cấu kết, dù lẫn khách thường).
    """
    share, has = [], []
    for m in groups["members"].to_list():
        acct = [x for x in m if not x.startswith("DEV")]
        ring = [x for x in acct if x in ring_of]
        share.append(len(ring) / max(1, len(acct)))
        has.append(len(ring) >= 3 and sum(x.startswith("CU") for x in ring) >= 2)
    return groups.with_columns(pl.Series("ring_share", share), pl.Series("has_ring", has)).with_columns(is_ring=pl.col("ring_share") >= 0.5)


def step_group_stats(groups: pl.DataFrame, c: Checker) -> None:
    g = groups.filter(pl.col("size") >= 3)
    say(f"\n[3] Thống kê nhóm qua {groups['day'].n_unique()} ảnh chụp (ngày 1 đến {LAST_DAY}), chỉ nhóm từ 3 nút ({g.height:,} trên {groups.height:,} nhóm-ngày)")
    cols = ["size", "customers", "drivers", "devices", "density", "cycle_rank", "top_betweenness"]
    tab = g.group_by("is_ring").agg(n=pl.len(), **{x: pl.col(x).median() for x in cols}).sort("is_ring")
    say(f"{'loại nhóm':16s}{'số nhóm':>9s}" + "".join(f"{x:>17s}" for x in cols))
    med = {}
    for r in tab.iter_rows(named=True):
        name = "cấu kết thật" if r["is_ring"] else "còn lại"
        med[r["is_ring"]] = r
        say(f"{name:16s}{r['n']:9,d}" + "".join(f"{r[x]:17.3f}" for x in cols))
    say("(trung vị theo nhóm-ngày; một nhóm tồn tại nhiều ngày được đếm mỗi ngày một lần)")
    c.check("nhóm cấu kết thật có nhiều chu trình hơn hẳn (trung vị cycle_rank gấp 3 lần)", med[True]["cycle_rank"] >= 3 * max(med[False]["cycle_rank"], 1), (med[True]["cycle_rank"], med[False]["cycle_rank"]))
    c.check("nhóm cấu kết thật lớn hơn hẳn nhóm còn lại (trung vị số nút gấp 2 lần)", med[True]["size"] >= 2 * med[False]["size"], (med[True]["size"], med[False]["size"]))


# ---- [4] Luật gắn cờ nhóm và tìm lại nhóm cấu kết thật ---------------------------------------


def evaluate_rule(g: pl.DataFrame, rule: gr.Rule) -> tuple[int, int, float, float]:
    """(số nhóm bị cờ, số nhóm «thuần» đúng, precision, recall) trên nhóm-ngày; «thuần» = is_ring."""
    flagged = g.filter(gr.suspect_expr(rule))
    tp = int(flagged["is_ring"].sum())
    prec = tp / flagged.height if flagged.height else 0.0
    rec = tp / max(1, int(g["is_ring"].sum()))
    return flagged.height, tp, prec, rec


def tune_rule(g: pl.DataFrame) -> list[tuple[float, gr.Rule]]:
    """Quét lưới trên bảng nhóm của train; chọn theo F1 trên nhóm-ngày. Trả danh sách (F1, luật) từ cao xuống thấp."""
    out = []
    for cr, dens, bt in product((2, 3, 4, 5), (0.0, 0.1, 0.15, 0.2, 0.25), (1.0, 0.7, 0.6, 0.55)):
        rule = gr.Rule(min_cycle_rank=cr, min_density=dens, max_top_betweenness=bt)
        _, _, p, r = evaluate_rule(g, rule)
        out.append((2 * p * r / (p + r) if p + r else 0.0, rule))
    return sorted(out, key=lambda x: -x[0])


def ring_detection(groups: pl.DataFrame, ring_of: dict[str, int], full: pl.DataFrame) -> pl.DataFrame:
    """Với mỗi nhóm cấu kết thật: ngày đầu tiên một nhóm bị gắn cờ chứa từ 2 khách của nó, và số chuyến thanh toán sau đó."""
    col = full.filter(pl.col("fraud_type") == "collusion").with_columns(ring=pl.col("customer_id").replace_strict(ring_of, return_dtype=pl.Int64))
    info = col.group_by("ring").agg(trips=pl.len(), first_day=pl.col("paid_day").min().floor(), customers=pl.col("customer_id").n_unique(),
                                    trips_train=(pl.col("paid_day") < TRAIN_END_DAY).sum(), trips_valid=(pl.col("paid_day") >= VALID_START_DAY).sum())
    cust_ring = {x: r for x, r in ring_of.items() if x.startswith("CU")}
    first_flag: dict[int, int] = {}
    flagged = groups.filter(pl.col("suspect") & pl.col("has_ring")).sort("day")
    for day, members in flagged.select("day", "members").iter_rows():
        counts: dict[int, int] = {}
        for x in members:
            if x in cust_ring:
                counts[cust_ring[x]] = counts.get(cust_ring[x], 0) + 1
        for r, k in counts.items():
            if k >= 2:
                first_flag.setdefault(r, day)
    rows = []
    for r in info.sort("first_day", "ring").iter_rows(named=True):
        fd = first_flag.get(r["ring"])
        after = int(col.filter((pl.col("ring") == r["ring"]) & (pl.col("paid_day") >= fd)).height) if fd is not None else 0
        rows.append({"ring": r["ring"], "customers": r["customers"], "trips": r["trips"], "first_day": int(r["first_day"]),
                     "flag_day": fd, "lag": None if fd is None else fd - int(r["first_day"]), "trips_after": after,
                     "trips_train": r["trips_train"], "trips_valid": r["trips_valid"]})
    return pl.DataFrame(rows, schema={"ring": pl.Int64, "customers": pl.Int64, "trips": pl.Int64, "first_day": pl.Int64,
                                      "flag_day": pl.Int64, "lag": pl.Int64, "trips_after": pl.Int64,
                                      "trips_train": pl.Int64, "trips_valid": pl.Int64})


def diagnose_rings(groups: pl.DataFrame, ring_of: dict[str, int], rings: pl.DataFrame) -> dict[int, dict[str, int]]:
    """Với nhóm cấu kết bị bỏ sót hoặc gắn cờ muộn (trễ trên 14 ngày): mỗi ngày lấy nhóm chứa nhiều khách của nó nhất (từ 2 khách),
    đếm điều kiện nào của luật làm nhóm đó trượt. «khác» là trượt số tài xế, số khách hoặc chu trình."""
    cust_ring = {x: r for x, r in ring_of.items() if x.startswith("CU")}
    late = rings.filter(pl.col("flag_day").is_null() | (pl.col("lag") > 14))
    wanted = set(late["ring"].to_list())
    best: dict[tuple[int, int], tuple[int, dict]] = {}
    for row in groups.iter_rows(named=True):
        counts: dict[int, int] = {}
        for x in row["members"]:
            r = cust_ring.get(x)
            if r in wanted:
                counts[r] = counts.get(r, 0) + 1
        for r, k in counts.items():
            if k >= 2 and ((r, row["day"]) not in best or k > best[(r, row["day"])][0]):
                best[(r, row["day"])] = (k, row)
    R = gr.RULE
    say("\nnhóm bị bỏ sót hoặc gắn cờ muộn (trễ trên 14 ngày): mỗi ngày lấy nhóm chứa nhiều khách của nó nhất, đếm điều kiện làm nó trượt")
    say(f"{'nhóm':>5s}{'chuyến':>8s}{'ngày có nhóm':>14s}{'đạt luật':>10s}{'chỉ density':>13s}{'chỉ betw':>10s}{'density+betw':>14s}{'khác':>6s}")
    out: dict[int, dict[str, int]] = {}
    for r in late.iter_rows(named=True):
        tally = {"pass": 0, "density": 0, "betw": 0, "both": 0, "other": 0}
        for (rr, _day), (_k, g) in best.items():
            if rr != r["ring"]:
                continue
            other = g["drivers"] < R.min_drivers or g["customers"] < R.min_customers or g["cycle_rank"] < R.min_cycle_rank
            d_bad, b_bad = g["density"] < R.min_density, g["top_betweenness"] > R.max_top_betweenness
            tally["other" if other else "both" if d_bad and b_bad else "density" if d_bad else "betw" if b_bad else "pass"] += 1
        out[r["ring"]] = tally
        say(f"{r['ring']:5d}{r['trips']:8d}{sum(tally.values()):14d}{tally['pass']:10d}{tally['density']:13d}{tally['betw']:10d}{tally['both']:14d}{tally['other']:6d}")
    return out


def step_rule(groups: pl.DataFrame, full: pl.DataFrame, ring_of: dict[str, int], c: Checker) -> dict:
    say("\n[4] Luật gắn cờ nhóm: chọn ngưỡng trên train (sự thật nền chỉ từ chuyến train), báo cáo trên valid")
    # Sự thật nền để CHỌN ngưỡng chỉ lấy từ chuyến thanh toán trước ngày 30 (nhãn cuối); valid dùng sự thật nền của cả kỳ để đo
    ring_train = ring_truth(full.filter(pl.col("paid_day") < TRAIN_END_DAY))
    tr_all = groups.filter((pl.col("day") >= TUNE_FIRST_DAY) & (pl.col("day") < TRAIN_END_DAY)).drop("ring_share", "has_ring", "is_ring")
    tr = with_truth(tr_all, ring_train)
    va = groups.filter(pl.col("day") >= VALID_START_DAY)
    say(f"nhóm-ngày thật trong train (ngày {TUNE_FIRST_DAY} đến {TRAIN_END_DAY - 1}, sự thật nền từ train): {int(tr['is_ring'].sum())}; trong valid (ngày {VALID_START_DAY} đến {LAST_DAY}): {int(va['is_ring'].sum())}")
    ranked = tune_rule(tr)

    def f1_of(g: pl.DataFrame, rule: gr.Rule) -> float:
        _, _, p, r = evaluate_rule(g, rule)
        return 2 * p * r / (p + r) if p + r else 0.0

    say("quét lưới trên train (3 luật điểm F1 cao nhất; các cột sau là kết quả của đúng luật đó trên valid):")
    say(f"{'cycle_rank':>11s}{'density':>9s}{'betw<=':>8s}{'F1 train':>10s}{'cờ valid':>10s}{'đúng':>7s}{'precision':>11s}{'recall':>8s}{'F1 valid':>10s}")
    for f1, rule in ranked[:3]:
        n, tp, p, r = evaluate_rule(va, rule)
        say(f"{rule.min_cycle_rank:11d}{rule.min_density:9.2f}{rule.max_top_betweenness:8.2f}{f1:10.3f}{n:10d}{tp:7d}{p:11.3f}{r:8.3f}{f1_of(va, rule):10.3f}")
    say(f"luật cố định trong graph.py: {gr.RULE}")
    rank = next(i for i, (_, r) in enumerate(ranked) if r == gr.RULE) + 1
    sens = {}
    for a, b in ((TUNE_FIRST_DAY, 18), (19, TRAIN_END_DAY - 1)):
        rk = tune_rule(with_truth(tr_all.filter(pl.col("day").is_between(a, b)), ring_train))
        sens[(a, b)] = next(i for i, (_, r) in enumerate(rk) if r == gr.RULE) + 1
    say(f"độ nhạy của việc chọn: F1 train hạng 1 {ranked[0][0]:.3f}, hạng 2 {ranked[1][0]:.3f} (hơn nhau {ranked[0][0] - ranked[1][0]:.3f}); "
        f"chọn chỉ trên ngày {TUNE_FIRST_DAY} đến 18, RULE đứng hạng {sens[(TUNE_FIRST_DAY, 18)]}/80; chỉ trên ngày 19 đến {TRAIN_END_DAY - 1}, hạng {sens[(19, TRAIN_END_DAY - 1)]}/80")
    say(f"{'bước thêm điều kiện (luật cố định)':44s}{'cờ train':>9s}{'prec':>7s}{'rec':>7s}{'cờ valid':>10s}{'prec':>7s}{'rec':>7s}")
    steps = [("từ 2 tài xế và 3 khách", gr.Rule(min_cycle_rank=0, min_density=0.0, max_top_betweenness=1.0)),
             (f"+ cycle_rank >= {gr.RULE.min_cycle_rank}", gr.Rule(min_density=0.0, max_top_betweenness=1.0)),
             (f"+ density >= {gr.RULE.min_density}", gr.Rule(max_top_betweenness=1.0)),
             (f"+ top_betweenness <= {gr.RULE.max_top_betweenness} (= RULE)", gr.RULE)]
    res = {}
    for name, rule in steps:
        a, b = evaluate_rule(tr, rule), evaluate_rule(va, rule)
        res[name] = (a, b)
        say(f"{name:44s}{a[0]:9d}{a[2]:7.3f}{a[3]:7.3f}{b[0]:10d}{b[2]:7.3f}{b[3]:7.3f}")
    final_tr, final_va = res[steps[-1][0]]
    loose = va.filter(gr.suspect_expr())
    say(f"precision nới trên valid (nhóm bị cờ có chứa từ 3 tài khoản cấu kết thật): {pct(loose['has_ring'].mean())}; "
        f"độ thuần trung vị của nhóm bị cờ (tỉ lệ tài khoản thuộc nhóm cấu kết): {loose['ring_share'].median():.2f}")
    flagged_valid = groups.filter(pl.col("suspect") & (pl.col("day") >= VALID_START_DAY)).sort("day")
    flagged_days = flagged_valid.group_by("day").len()["len"]
    seen_sets: set[frozenset[str]] = set()
    for members in flagged_valid["members"].to_list():
        seen_sets.add(frozenset(members))
    n_days_valid = groups.filter(pl.col("day") >= VALID_START_DAY)["day"].n_unique()
    say(f"nhóm-ngày bị gắn cờ trong valid: {flagged_valid.height}, mỗi ngày trung bình {flagged_days.mean():.1f}, lớn nhất {flagged_days.max()}; "
        f"nhưng chỉ {len(seen_sets)} tập thành viên khác nhau, tức trung bình {len(seen_sets) / n_days_valid:.1f} tập mới mỗi ngày")
    say("recall của luật theo giai đoạn (sự thật nền của cả kỳ, chỉ để mô tả):")
    say(f"{'ngày':>9s}{'nhóm-ngày thật':>16s}{'cờ':>6s}{'precision':>11s}{'recall':>8s}")
    for a, b in ((7, 14), (15, 22), (23, 29), (30, 39), (40, 49), (50, 59)):
        g_ = groups.filter(pl.col("day").is_between(a, b))
        n_, _tp, p_, r_ = evaluate_rule(g_, gr.RULE)
        say(f"{a:>4d}-{b:<4d}{int(g_['is_ring'].sum()):16d}{n_:6d}{p_:11.3f}{r_:8.3f}")

    rings = ring_detection(groups, ring_of, full)
    say("\ntừng nhóm cấu kết thật (nhãn chỉ để đánh giá): ngày đầu tiên có chuyến cấu kết, ngày nhóm bị gắn cờ lần đầu, số chuyến ở train (trước ngày 30) và valid (từ ngày 40)")
    say(f"{'nhóm':>5s}{'khách':>7s}{'chuyến':>8s}{'ngày đầu':>10s}{'ngày cờ':>9s}{'trễ (ngày)':>12s}{'chuyến sau cờ':>15s}{'train':>7s}{'valid':>7s}")
    for r in rings.iter_rows(named=True):
        say(f"{r['ring']:5d}{r['customers']:7d}{r['trips']:8d}{r['first_day']:10d}{'-' if r['flag_day'] is None else r['flag_day']:>9}"
            f"{'-' if r['lag'] is None else r['lag']:>12}{r['trips_after']:15d}{r['trips_train']:7d}{r['trips_valid']:7d}")
    hit = rings.filter(pl.col("flag_day").is_not_null())
    after, total = int(hit["trips_after"].sum()), int(rings["trips"].sum())
    lo, hi = wilson(hit.height, rings.height)
    say(f"bắt được {hit.height}/{rings.height} nhóm (khoảng tin cậy Wilson 95 %: {lo:.3f} đến {hi:.3f}); độ trễ trung vị {statistics.median(hit['lag'].to_list()):.0f} ngày; "
        f"{after}/{total} chuyến cấu kết ({after / total:.1%}) thanh toán từ ngày nhóm bị gắn cờ trở đi")
    seen_train = rings.filter(pl.col("trips_train") > 0)
    unseen = rings.filter(pl.col("trips_train") == 0)
    active = rings.filter(pl.col("trips_valid") > 0)
    say(f"nhóm đã có chuyến ở train (ngưỡng được chọn khi đã thấy chúng): {seen_train.height}/{rings.height}; chưa có chuyến nào ở train: {unseen.height}, "
        f"bị bắt {unseen['flag_day'].is_not_null().sum()}; nhóm còn hoạt động ở valid: {active.height}, trong đó đã có ở train: {active.filter(pl.col('trips_train') > 0).height}")
    col_dev = full.filter(pl.col("fraud_type") == "collusion").with_columns(ring=pl.col("customer_id").replace_strict(ring_of, return_dtype=pl.Int64))
    devs = {r: set(g["device_id"].to_list()) for (r,), g in col_dev.group_by("ring")}
    shared_dev = [(a, b) for a in sorted(devs) for b in sorted(devs) if a < b and devs[a] & devs[b]]
    say(f"cặp nhóm cấu kết thật dùng chung ít nhất một thiết bị khách: {shared_dev}")
    diagnose_rings(groups, ring_of, rings)
    c.check("luật chọn bằng lưới trên train: precision trên valid từ 0,7", final_va[2] >= 0.7, final_va)
    c.check("precision nới (nhóm chứa nhóm cấu kết) cao hơn precision thuần", loose["has_ring"].mean() > final_va[2], (loose["has_ring"].mean(), final_va))
    c.check("mỗi điều kiện thêm vào làm precision train tăng", res[steps[0][0]][0][2] < res[steps[1][0]][0][2] < res[steps[3][0]][0][2], res)
    c.check("luật cố định là luật có F1 train cao nhất của lưới", gr.RULE == ranked[0][1], ranked[:3])
    c.check("suspect_expr khớp is_suspect trong snapshot", int((groups["suspect"] != groups.select(gr.suspect_expr().alias("s"))["s"]).sum()) == 0)
    c.check("bắt được từ 60 % số nhóm cấu kết thật (số nhóm do bộ sinh quyết định, không ghim)", rings.height >= 10 and hit.height >= 0.6 * rings.height, (hit.height, rings.height))
    c.check("nhóm bị gắn cờ có độ trễ dương hoặc bằng 0, không có nhóm bị gắn cờ trước chuyến đầu", bool((hit["lag"] >= 0).all()), hit)
    c.check("vẫn còn chuyến cấu kết sau ngày gắn cờ (đủ để người duyệt can thiệp)", after > 0.3 * total, (after, total))
    c.check("mỗi ngày valid chỉ có vài chục nhóm cần xem, không phải hàng trăm", flagged_days.max() < 40, flagged_days.max())
    c.check("nhóm-ngày bị cờ lặp lại: số tập thành viên khác nhau ít hơn số nhóm-ngày", len(seen_sets) < flagged_valid.height, (len(seen_sets), flagged_valid.height))
    return {"rank": rank, "valid": final_va, "rings": rings.height, "hit": hit.height}


def step_exercise(c: Checker) -> None:
    """Hai hình mẫu của hình 1 dựng bằng tay: lưới K(2,3) bị cờ, hình sao thì không (đáp án bài tập 1)."""
    empty = pl.DataFrame(schema={"account": pl.String, "device": pl.String})
    mesh = pl.DataFrame([{"customer_id": f"CU-0000{i}", "driver_id": f"DR-0000{j}", "trips": 2} for i in (1, 2, 3) for j in (1, 2)])
    star = pl.DataFrame([{"customer_id": f"CU-0000{i}", "driver_id": "DR-00001", "trips": 4} for i in range(1, 6)])
    say("\nHai hình mẫu dựng bằng tay (không qua dữ liệu):")
    got = {}
    for name, pairs in (("2 tài xế x 3 khách, mỗi cặp 2 chuyến", mesh), ("1 tài xế x 5 khách, mỗi cặp 4 chuyến", star)):
        g = gr.build_graph(pairs, empty)
        st, _ = gr.group_stats(g, frozenset(g.nodes))
        got[name] = (st, gr.is_suspect(st))
        say(f"  {name}: {st['size']} nút, {st['edges']} cạnh, mật độ {st['density']:.2f}, chu trình {st['cycle_rank']}, betweenness lớn nhất {st['top_betweenness']:.2f} -> bị cờ: {gr.is_suspect(st)}")
    (m_st, m_flag), (s_st, s_flag) = got.values()
    c.check("lưới 2 tài xế x 3 khách có 2 chu trình và bị gắn cờ", m_st["cycle_rank"] == 2 and m_flag, m_st)
    c.check("hình sao 1 tài xế x 5 khách có 0 chu trình và không bị gắn cờ", s_st["cycle_rank"] == 0 and not s_flag, s_st)


# ---- [5] Đặc trưng đồ thị trong mô hình ---------------------------------------------------------


def step_model(full, train, valid, nodes, pairs, m_hgb, c: Checker):
    say("\n[5] Đặc trưng đồ thị trong mô hình (HistGradientBoosting, cùng siêu tham số, cùng train/valid)")
    x = gr.add_graph_features(full, nodes, pairs)
    tr, va = x.filter(pl.col("paid_day") < TRAIN_END_DAY), x.filter(pl.col("paid_day") >= VALID_START_DAY)
    y_tr = tr["is_fraud"].to_numpy()
    cols_g = FEATURES + gr.GRAPH_FEATURES
    say(f"{len(gr.GRAPH_FEATURES)} đặc trưng đồ thị: {', '.join(gr.GRAPH_FEATURES)}")
    say("trung bình theo loại gian lận trên valid (đặc trưng từ ảnh chụp đầu ngày):")
    show = ["gc_group_size", "gc_cycle_rank", "gd_betweenness", "g_edge_trips", "g_suspect_group"]
    tab = va.group_by("fraud_type").agg([pl.col(k).mean() for k in show]).sort("fraud_type")
    say(f"{'loại':12s}" + "".join(f"{k:>18s}" for k in show))
    means = {}
    for r in tab.iter_rows(named=True):
        means[r["fraud_type"]] = r
        say(f"{r['fraud_type']:12s}" + "".join(f"{r[k]:18.3f}" for k in show))

    t0 = time.perf_counter()
    m_g = hgb().fit(tr.select(cols_g).to_numpy(), y_tr)
    say(f"HistGradientBoosting có đặc trưng đồ thị dừng sớm ở {m_g.n_iter_} vòng (tối đa 150)")
    s_g = m_g.predict_proba(va.select(cols_g).to_numpy())[:, 1]
    r_g = type_metrics(va, s_g)
    only = hgb().fit(tr.select(gr.GRAPH_FEATURES).to_numpy(), y_tr)
    r_only = type_metrics(va, only.predict_proba(va.select(gr.GRAPH_FEATURES).to_numpy())[:, 1])
    say(f"\n{'mô hình':34s}{'PR-AUC':>8s}{'collusion':>11s}{'recall@1%':>11s}{'promo_abuse':>13s}{'recall@1%':>11s}{'ghost_trip':>12s}{'gps_spoof':>11s}")

    def line(name: str, m: dict) -> None:
        b = m["by"]
        say(f"{name:34s}{m['pr']:8.3f}{b['collusion']['ap']:11.3f}{pct(b['collusion']['rec1']):>11s}{b['promo_abuse']['ap']:13.3f}{pct(b['promo_abuse']['rec1']):>11s}"
            f"{b['ghost_trip']['ap']:12.3f}{b['gps_spoof']['ap']:11.3f}")

    line("30 đặc trưng cửa sổ (mốc, bài 55)", m_hgb)
    line("+ 14 đặc trưng đồ thị", r_g)
    line("chỉ 14 đặc trưng đồ thị", r_only)
    say(f"(huấn luyện hai mô hình trong {time.perf_counter() - t0:.0f} giây; load average: {load()})")

    # Luật nhóm đứng một mình, không học gì
    flag = va["g_suspect_group"].to_numpy() == 1
    col = (va["fraud_type"] == "collusion").to_numpy()
    y = va["is_fraud"].to_numpy()
    say(f"luật nhóm một mình, không mô hình: gắn cờ {int(flag.sum())} chuyến; {int((flag & col).sum())}/{int(col.sum())} chuyến collusion "
        f"(recall {pct((flag & col).sum() / col.sum())}); {pct((flag & y).sum() / max(1, flag.sum()))} số chuyến bị cờ là gian lận (mọi loại), "
        f"{pct((flag & col).sum() / max(1, flag.sum()))} là collusion")
    kinds = va.filter(pl.col("g_suspect_group") == 1).group_by("fraud_type").len().sort("len", descending=True)
    say("  chuyến bị cờ theo loại:", dict(zip(kinds["fraud_type"], kinds["len"], strict=True)))

    c_base, c_g = m_hgb["by"]["collusion"], r_g["by"]["collusion"]
    c.check("collusion: đặc trưng đồ thị nâng PR-AUC theo loại thêm ít nhất 0,1", c_g["ap"] > c_base["ap"] + 0.1, (c_base, c_g))
    c.check("collusion: recall@1% tăng thêm ít nhất 10 điểm phần trăm", c_g["rec1"] > c_base["rec1"] + 0.10, (c_base, c_g))
    c.check("PR-AUC tổng không tụt quá 0,01 khi thêm đặc trưng đồ thị", r_g["pr"] > m_hgb["pr"] - 0.01, (m_hgb["pr"], r_g["pr"]))
    c.check("collusion: chỉ đặc trưng đồ thị đã tốt hơn mốc logistic nhiều lần", r_only["by"]["collusion"]["ap"] > 5 * 0.026, r_only["by"]["collusion"])
    c.check("đặc trưng collusion sáng ở cờ nhóm (gấp 10 lần chuyến sạch)", means["collusion"]["g_suspect_group"] > 10 * means["none"]["g_suspect_group"], (means["collusion"], means["none"]))
    c.check("không đặc trưng đồ thị nào bị null", all(x[k].null_count() == 0 for k in gr.GRAPH_FEATURES))
    return x, r_g


# ---- [6] Rò rỉ: đồ thị nhìn tương lai --------------------------------------------------------------


def snapshot_signature(s: gr.Snapshot) -> str:
    """Dấu vân tay của ảnh chụp: nút (kèm số hiệu nhóm, bậc, core, betweenness làm tròn 9 chữ số) và thành viên từng nhóm."""
    rows = [(r["node"], r["group_id"], r["degree"], r["core"], round(r["betweenness"], 9), r["g_size"], r["g_cycle_rank"],
             round(r["g_density"], 9), r["g_suspect"]) for r in s.nodes.sort("node").iter_rows(named=True)]
    members = sorted((r["day"], tuple(r["members"])) for r in s.groups.iter_rows(named=True))
    return hashlib.sha256(repr((rows, members)).encode()).hexdigest()[:16]


def digest_in_subprocess(hash_seed: int) -> str:
    """Dựng lại ảnh chụp ngày 50 trong một tiến trình mới với PYTHONHASHSEED khác (`main.py --digest`)."""
    env = {**os.environ, "PYTHONHASHSEED": str(hash_seed)}
    out = subprocess.run([sys.executable, __file__, "--digest"], env=env, capture_output=True, text=True, check=True)
    return out.stdout.strip()


def step_leak(full, x, r_g, c: Checker) -> tuple[dict, dict]:
    say("\n[6] Rò rỉ: phép thử cắt cụt và đồ thị dựng một lần trên cả 60 ngày")
    days = (45, 50, 55)
    same_cut, same_label, diff_leaky = [], [], []
    labels_cols = ["fraud_type", "is_fraud", "label_known_at"]
    for d in days:
        cut = full.filter(pl.col("paid_at") < START + timedelta(days=d))
        honest_full = snapshot_signature(gr.snapshot(full, d))
        same_cut.append(snapshot_signature(gr.snapshot(cut, d)) == honest_full)
        same_label.append(snapshot_signature(gr.snapshot(full.drop(labels_cols), d)) == honest_full)
        diff_leaky.append(snapshot_signature(gr.snapshot(cut, d, lookahead_days=3)) != snapshot_signature(gr.snapshot(full, d, lookahead_days=3)))
    say(f"ảnh chụp ngày {days}: dựng từ event cắt ở đầu ngày giống dựng từ toàn bộ? {same_cut}")
    say(f"  giống nhau khi bỏ hết cột nhãn khỏi bảng chuyến? {same_label}")
    say(f"  nếu cửa sổ kết thúc muộn hơn 3 ngày (nhìn tương lai), phép cắt cụt có làm kết quả đổi (lộ lỗi)? {diff_leaky}")
    here = snapshot_signature(gr.snapshot(full, 50))
    other = {seed: digest_in_subprocess(seed) for seed in (1, 2)}
    say(f"ảnh chụp ngày 50 dựng lại trong tiến trình mới với PYTHONHASHSEED=1 và 2: dấu vân tay {other[1]}, {other[2]}; tiến trình này {here}")
    c.check("ảnh chụp không phụ thuộc PYTHONHASHSEED (hai tiến trình mới khớp tiến trình này)", other[1] == other[2] == here, (other, here))
    c.check("ảnh chụp bất biến khi cắt event ở đầu ngày (3 ngày thử)", all(same_cut), same_cut)
    c.check("ảnh chụp không đọc nhãn: bỏ cột nhãn không đổi gì", all(same_label), same_label)
    c.check("cửa sổ nhìn tương lai bị phép thử cắt cụt phát hiện ở cả 3 ngày", all(diff_leaky), diff_leaky)

    # Đồ thị dựng MỘT LẦN trên cả 60 ngày, mọi chuyến dùng chung ảnh chụp đó
    t0 = time.perf_counter()
    big = gr.snapshot(full, LAST_DAY + 2, window_days=LAST_DAY + 2)
    xl = gr.add_graph_features(full, big.nodes, big.pairs, fixed_day=LAST_DAY + 2).with_columns(paid_day=day_of())
    say(f"đồ thị toàn kỳ: {big.graph.number_of_edges():,} cạnh, {big.groups.height:,} nhóm, dựng trong {time.perf_counter() - t0:.1f} giây")
    cols_g = FEATURES + gr.GRAPH_FEATURES
    tr_l, va_l = xl.filter(pl.col("paid_day") < TRAIN_END_DAY), xl.filter(pl.col("paid_day") >= VALID_START_DAY)
    leaky = hgb().fit(tr_l.select(cols_g).to_numpy(), tr_l["is_fraud"].to_numpy())
    off = type_metrics(va_l, leaky.predict_proba(va_l.select(cols_g).to_numpy())[:, 1])
    va_h = x.filter(pl.col("paid_day") >= VALID_START_DAY)
    srv = type_metrics(va_h, leaky.predict_proba(va_h.select(cols_g).to_numpy())[:, 1])
    say(f"{'mô hình':44s}{'PR-AUC':>8s}{'collusion PR-AUC':>18s}")
    say(f"{'đồ thị theo ngày, đo như phục vụ (bước 5)':44s}{r_g['pr']:8.3f}{r_g['by']['collusion']['ap']:18.3f}")
    say(f"{'đồ thị toàn kỳ, offline (valid cũng toàn kỳ)':44s}{off['pr']:8.3f}{off['by']['collusion']['ap']:18.3f}")
    say(f"{'đồ thị toàn kỳ, phục vụ (valid theo ngày)':44s}{srv['pr']:8.3f}{srv['by']['collusion']['ap']:18.3f}")
    c.check("đồ thị toàn kỳ: offline đẹp hơn phục vụ thật (PR-AUC chênh trên 0,03)", off["pr"] > srv["pr"] + 0.03, (off["pr"], srv["pr"]))
    c.check("đồ thị toàn kỳ: offline đẹp hơn cả đồ thị theo ngày đo trung thực", off["pr"] > r_g["pr"] + 0.01, (off["pr"], r_g["pr"]))
    return off, srv


def check_edge_feature(full: pl.DataFrame, x: pl.DataFrame, c: Checker) -> None:
    """Đối chiếu độc lập g_edge_trips: đếm lại từ bảng chuyến, cặp có từ 2 chuyến TRƯỚC đầu ngày trong 28 ngày, ghi lại số chuyến."""
    bad = total = 0
    for d in (5, 20, 45, 59):
        hist = gr.window_slice(full, d).group_by("customer_id", "driver_id").agg(expect=pl.len()).filter(pl.col("expect") >= gr.STRONG_MIN_TRIPS)
        today = x.filter(pl.col("day") == d).select("trip_id", "customer_id", "driver_id", "g_edge_trips")
        got = today.join(hist, on=["customer_id", "driver_id"], how="left").with_columns(pl.col("expect").fill_null(0).cast(pl.Float64))
        bad += int((got["g_edge_trips"] != got["expect"]).sum())
        total += got.height
    say(f"\nđối chiếu độc lập g_edge_trips với đếm lại từ bảng chuyến (ngày 5, 20, 45, 59): {bad} lệch trên {total:,} chuyến")
    c.check("g_edge_trips khớp phép đếm lại độc lập từ bảng chuyến", bad == 0 and total > 1_000, (bad, total))


# ---- [7] Quy mô ----------------------------------------------------------------------------------


def graph_cost(pairs: pl.DataFrame) -> tuple[nx.Graph, float, float, float]:
    """Dựng đồ thị đúng như build_graph (chuỗi ID, thuộc tính weight/kind trên cạnh, kind trên nút); trả (đồ thị, giây dựng, giây thành phần, byte mỗi cạnh).

    Bộ nhớ đo bằng tracemalloc với chuỗi ID sinh NGAY TRONG phiên đo, vì ID là chuỗi riêng của đồ thị."""
    def build() -> nx.Graph:
        g = nx.Graph()
        g.add_edges_from((c, d, {"weight": n, "kind": "trip"}) for c, d, n in pairs.select("customer_id", "driver_id", "trips").iter_rows())
        nx.set_node_attributes(g, {n: gr.kind_of(n) for n in g}, "kind")
        return g

    t0 = time.perf_counter()
    g = build()
    t_build = time.perf_counter() - t0
    t0 = time.perf_counter()
    nx.number_connected_components(g)
    t_comp = time.perf_counter() - t0
    del g
    tracemalloc.start()
    g = build()
    mem = tracemalloc.get_traced_memory()[0]
    tracemalloc.stop()
    return g, t_build, t_comp, mem / g.number_of_edges()


def step_scale(full, c: Checker) -> None:
    say(f"\n[7] Quy mô: networkx giữ đồ thị trong bộ nhớ Python; load average lúc đo: {load()}")
    say("đồ thị dựng như build_graph (chuỗi ID, thuộc tính trên cạnh và nút), đo bộ nhớ bằng tracemalloc:")
    say(f"{'đồ thị':>12s}{'cạnh':>10s}{'nút':>9s}{'dựng (giây)':>13s}{'thành phần (giây)':>19s}{'byte/cạnh':>11s}{'giữ lại':>9s}")
    rows = {}
    for days in (7, 14, 28, 56):
        w = gr.window_slice(full, 58, days)
        pairs = w.group_by("customer_id", "driver_id").agg(trips=pl.len())
        g, t_build, t_comp, bpe = graph_cost(pairs)
        p_, s_ = gr.edge_tables(w)
        keep = (p_.height + s_.height) / pairs.height
        rows[days] = (g.number_of_edges(), bpe, keep)
        say(f"{'thô ' + str(days) + ' ngày':>12s}{g.number_of_edges():10,d}{g.number_of_nodes():9,d}{t_build:13.3f}{t_comp:19.3f}{bpe:11.0f}{keep:9.1%}")
    w28 = gr.window_slice(full, 58, 28)
    pairs, shared = gr.edge_tables(w28)
    tracemalloc.start()
    t0 = time.perf_counter()
    gf = gr.build_graph(pairs, shared)
    t_f = time.perf_counter() - t0
    mem_f = tracemalloc.get_traced_memory()[0]
    tracemalloc.stop()
    bpe_f = mem_f / gf.number_of_edges()
    say(f"{'lọc 28 ngày':>12s}{gf.number_of_edges():10,d}{gf.number_of_nodes():9,d}{t_f:13.3f}{'-':>19s}{bpe_f:11.0f}   (giây dựng đo trong phiên tracemalloc)")
    per_day, year = 2_000_000, 730_000_000
    lo, hi = sorted((rows[28][1], rows[56][1]))
    say(f"ngoại suy tuyến tính, KHÔNG đo: cửa sổ 28 ngày ở 2 triệu chuyến/ngày tối đa {28 * per_day / 1e6:.0f} triệu cạnh ≈ {28 * per_day * lo / 2**30:.0f} đến {28 * per_day * hi / 2**30:.0f} GiB; "
        f"730 triệu cạnh/năm ≈ {year * lo / 2**30:.0f} đến {year * hi / 2**30:.0f} GiB (byte/cạnh của đồ thị thô 56 và 28 ngày)")
    say(f"đồ thị đã lọc: {28 * per_day * rows[28][2]:,.0f} cạnh sau lọc ở 56 triệu cạnh thô (giữ {rows[28][2]:.1%}) x {bpe_f:.0f} byte ≈ {28 * per_day * rows[28][2] * bpe_f / 2**30:.1f} GiB")
    say(f"tỉ lệ cạnh giữ lại theo cửa sổ: {', '.join(f'{d} ngày {rows[d][2]:.1%}' for d in rows)}")

    # Betweenness: đắt O(V*E); xấp xỉ bằng lấy mẫu k nút nguồn
    s = gr.snapshot(full, 58)
    comp = max(nx.connected_components(s.graph), key=len)
    sg = gr.ordered_subgraph(s.graph, comp)  # có thứ tự: lấy mẫu k nút dùng thứ tự nút, nên phải ổn định giữa các lần chạy
    t0 = time.perf_counter()
    exact = nx.betweenness_centrality(sg)
    t_exact = time.perf_counter() - t0
    t0 = time.perf_counter()
    approx = nx.betweenness_centrality(sg, k=50, seed=0)
    t_approx = time.perf_counter() - t0
    top_e = {k for k, _ in sorted(exact.items(), key=lambda kv: -kv[1])[:20]}
    top_a = {k for k, _ in sorted(approx.items(), key=lambda kv: -kv[1])[:20]}
    say(f"betweenness trên thành phần lớn nhất ({sg.number_of_nodes()} nút, {sg.number_of_edges()} cạnh): chính xác {t_exact:.2f} giây, "
        f"lấy mẫu k=50 {t_approx:.2f} giây; trùng {len(top_e & top_a)}/20 nút đứng đầu")
    c.check("cạnh tăng theo cửa sổ (đúng thứ bậc)", all(rows[a][0] < rows[b][0] for a, b in ((7, 14), (14, 28), (28, 56))), [rows[d][0] for d in rows])
    c.check("mỗi cạnh tốn từ 100 đến 3.000 byte trong networkx (kể cả chuỗi ID và thuộc tính)", all(100 < r[1] < 3_000 for r in rows.values()) and 100 < bpe_f < 3_000, [round(r[1]) for r in rows.values()])
    c.check("đo đủ chuỗi và thuộc tính cho thấy nhiều byte mỗi cạnh hơn 300", rows[28][1] > 300, rows[28][1])
    c.check("lọc cạnh bằng polars giữ dưới 10 % cạnh thô ở mọi cửa sổ", all(r[2] < 0.1 for r in rows.values()), {d: round(r[2], 3) for d, r in rows.items()})
    c.check("betweenness lấy mẫu trùng ít nhất 10/20 nút đầu bản chính xác", len(top_e & top_a) >= 10, len(top_e & top_a))


# ---- [8] Hàng đợi người duyệt ---------------------------------------------------------------------


def review_record(row: dict) -> dict:
    """Bản ghi một nhóm cho người duyệt: nêu bằng chứng, KHÔNG kết luận."""
    return {
        # số hiệu nhóm trong ảnh chụp đổi mỗi ngày; mã ổn định lấy theo thành viên có ID nhỏ nhất
        "group_id": f"G-{min(row['members'])}",
        "snapshot_day": row["day"],
        "status": "needs_review",
        "members": {"customers": row["customers"], "drivers": row["drivers"], "devices": row["devices"]},
        "member_ids": sorted(row["members"]),
        "trips_in_window": row["trips"],
        "reasons": [
            f"{row['drivers']} tài xế và {row['customers']} khách nối với nhau bằng các cặp từ {gr.STRONG_MIN_TRIPS} chuyến trở lên",
            f"{row['cycle_rank']} chu trình độc lập, mật độ {row['density']:.2f}, không phải một hub duy nhất (betweenness lớn nhất {row['top_betweenness']:.2f})",
            f"{row['devices']} thiết bị dùng chung từ 2 tài khoản",
        ],
        "note": "Gắn cờ nhóm để người duyệt xem; không phải kết luận cấu kết. Cần xem chuyến, thiết bị và nhãn của người duyệt trước khi hành động.",
    }


def step_review(groups: pl.DataFrame, c: Checker) -> None:
    say("\n[8] Hàng đợi người duyệt: nhóm bị gắn cờ ở ảnh chụp ngày cuối")
    last = groups.filter((pl.col("day") == LAST_DAY) & pl.col("suspect")).sort("trips", descending=True)
    say(f"ngày {LAST_DAY}: {last.height} nhóm bị gắn cờ; bản ghi của nhóm có nhiều chuyến nhất:")
    rec = review_record(last.row(0, named=True))
    say(json.dumps(rec, ensure_ascii=False, indent=2))
    ring_share = last["ring_share"].to_list()
    say(f"(đối chiếu nhãn, không đưa vào bản ghi: {sum(s >= 0.5 for s in ring_share)}/{last.height} nhóm là nhóm cấu kết thật)")
    c.check("bản ghi duyệt có trạng thái needs_review và không có kết luận", rec["status"] == "needs_review" and "verdict" not in rec and "is_fraud" not in rec)
    c.check("bản ghi duyệt liệt kê đủ thành viên và mã nhóm lấy theo thành viên", len(rec["member_ids"]) == last.row(0, named=True)["size"] and rec["group_id"] == f"G-{rec['member_ids'][0]}")
    c.check("ngày cuối có nhóm bị gắn cờ", last.height > 0, last.height)


# ---- [9] Độ nhạy theo seed -------------------------------------------------------------------------


def seed_summary(seed: int) -> dict:
    """Chạy lại chuỗi đo chính trên dữ liệu sinh với `seed` khác: luật nhóm, nhóm cấu kết thật, mô hình có/không đặc trưng đồ thị, đồ thị toàn kỳ."""
    _, full = make_full(seed)
    ring_of = ring_truth(full)
    nodes, groups, pairs = gr.snapshots(full, range(1, LAST_DAY + 1))
    groups = with_truth(groups, ring_of)
    ring_train = ring_truth(full.filter(pl.col("paid_day") < TRAIN_END_DAY))
    tr_all = groups.filter((pl.col("day") >= TUNE_FIRST_DAY) & (pl.col("day") < TRAIN_END_DAY)).drop("ring_share", "has_ring", "is_ring")
    ranked = tune_rule(with_truth(tr_all, ring_train))
    va = groups.filter(pl.col("day") >= VALID_START_DAY)
    rank = next(i for i, (_, r) in enumerate(ranked) if r == gr.RULE) + 1
    _, _, prec, rec = evaluate_rule(va, gr.RULE)
    rings = ring_detection(groups, ring_of, full)
    x = gr.add_graph_features(full, nodes, pairs)
    cols_g = FEATURES + gr.GRAPH_FEATURES
    tr, va_x = x.filter(pl.col("paid_day") < TRAIN_END_DAY), x.filter(pl.col("paid_day") >= VALID_START_DAY)
    y_tr = tr["is_fraud"].to_numpy()
    base = type_metrics(va_x, hgb().fit(tr.select(FEATURES).to_numpy(), y_tr).predict_proba(va_x.select(FEATURES).to_numpy())[:, 1])
    withg = type_metrics(va_x, hgb().fit(tr.select(cols_g).to_numpy(), y_tr).predict_proba(va_x.select(cols_g).to_numpy())[:, 1])
    big = gr.snapshot(full, LAST_DAY + 2, window_days=LAST_DAY + 2)
    xl = gr.add_graph_features(full, big.nodes, big.pairs, fixed_day=LAST_DAY + 2).with_columns(paid_day=day_of())
    tr_l, va_l = xl.filter(pl.col("paid_day") < TRAIN_END_DAY), xl.filter(pl.col("paid_day") >= VALID_START_DAY)
    leaky = hgb().fit(tr_l.select(cols_g).to_numpy(), tr_l["is_fraud"].to_numpy())
    off = type_metrics(va_l, leaky.predict_proba(va_l.select(cols_g).to_numpy())[:, 1])
    srv = type_metrics(va_x, leaky.predict_proba(va_x.select(cols_g).to_numpy())[:, 1])
    return {"rings": rings.height, "hit": int(rings["flag_day"].is_not_null().sum()), "rank": rank, "prec": prec, "rec": rec,
            "base": base, "withg": withg, "off": off, "srv": srv}


def step_seeds(main_summary: dict, c: Checker) -> None:
    say("\n[9] Độ nhạy theo seed của bộ sinh (cùng quy trình, nhãn cuối ở mọi bước)")
    results = {SEED: main_summary}
    for seed in (7, 123):
        t0 = time.perf_counter()
        results[seed] = seed_summary(seed)
        say(f"(seed {seed}: {time.perf_counter() - t0:.0f} giây; load average: {load()})")
    say(f"{'seed':>5s}{'nhóm thật':>10s}{'bắt':>5s}{'RULE hạng':>10s}{'prec valid':>11s}{'rec valid':>10s}"
        f"{'PR-AUC cửa sổ':>15s}{'+đồ thị':>9s}{'collusion cửa sổ':>18s}{'+đồ thị':>9s}{'toàn kỳ offline':>17s}{'phục vụ':>9s}")
    for seed, r in results.items():
        say(f"{seed:5d}{r['rings']:10d}{r['hit']:5d}{r['rank']:10d}{r['prec']:11.3f}{r['rec']:10.3f}{r['base']['pr']:15.3f}{r['withg']['pr']:9.3f}"
            f"{r['base']['by']['collusion']['ap']:18.3f}{r['withg']['by']['collusion']['ap']:9.3f}{r['off']['pr']:17.3f}{r['srv']['pr']:9.3f}")
    for seed, r in results.items():
        c.check(f"seed {seed}: đặc trưng đồ thị nâng PR-AUC theo loại của collusion thêm ít nhất 0,05", r["withg"]["by"]["collusion"]["ap"] > r["base"]["by"]["collusion"]["ap"] + 0.05, r)
        c.check(f"seed {seed}: đồ thị toàn kỳ đẹp hơn khi phục vụ thật (PR-AUC offline lớn hơn)", r["off"]["pr"] > r["srv"]["pr"], r)
        c.check(f"seed {seed}: luật bắt từ 60 % số nhóm cấu kết", r["hit"] >= 0.6 * r["rings"] and r["rings"] >= 10, (r["hit"], r["rings"]))


# ---- Chạy ------------------------------------------------------------------------------------------


def run() -> int:
    c = Checker()
    data, full, train, valid, m_lr, m_hgb = step_baselines(c)
    ring_of = ring_truth(full)
    step_stages(full, ring_of, c)

    t0 = time.perf_counter()
    nodes, groups, pairs = gr.snapshots(full, range(1, LAST_DAY + 1))
    groups = with_truth(groups, ring_of)
    say(f"\ndựng {LAST_DAY} ảnh chụp ngày 1 đến {LAST_DAY} (mỗi ngày một đồ thị cửa sổ {gr.WINDOW_DAYS} ngày): {groups.height:,} nhóm-ngày, "
        f"{nodes.height:,} nút-ngày trong {time.perf_counter() - t0:.0f} giây; load average: {load()}")
    step_group_stats(groups, c)
    rule_info = step_rule(groups, full, ring_of, c)
    step_exercise(c)
    x, r_g = step_model(full, train, valid, nodes, pairs, m_hgb, c)
    off, srv = step_leak(full, x, r_g, c)
    check_edge_feature(full, x, c)
    step_scale(full, c)
    step_review(groups, c)
    step_seeds({"rings": rule_info["rings"], "hit": rule_info["hit"], "rank": rule_info["rank"], "prec": rule_info["valid"][2], "rec": rule_info["valid"][3],
                "base": m_hgb, "withg": r_g, "off": off, "srv": srv}, c)

    s1, s2 = gr.snapshot(full, 50), gr.snapshot(full, 50)
    c.check("cùng đầu vào, cùng seed Louvain cho cùng nhóm (tái lập được)", snapshot_signature(s1) == snapshot_signature(s2))

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
    if "--digest" in sys.argv[1:]:  # chỉ để phép thử PYTHONHASHSEED trong --verify gọi lại
        data = make_events(seed=SEED, n_trips=N_TRIPS)
        full = add_features(trip_table(data.events)).join(data.labels, on="trip_id").sort("paid_at")
        print(snapshot_signature(gr.snapshot(full, 50)))
        sys.exit(0)
    QUIET = "--verify" in sys.argv[1:]
    sys.exit(run())


if __name__ == "__main__":
    main()
