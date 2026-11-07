"""Bài 61 - SHAP, nhật ký quyết định và hàng đợi người duyệt cho điểm gian lận.

Chạy từ gốc repository (cần PostgreSQL của hạ tầng chung):
    docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
    uv run --project examples/toi-noi-python/61-shap-audit-human-review \
        examples/toi-noi-python/61-shap-audit-human-review/main.py
    ... main.py --verify    # kiểm các bất biến in trong bài, in OK n/n

Hàng đợi và nhật ký nằm ở schema `lab61` của PostgreSQL, được tạo lúc bắt đầu và XÓA khi xong (kể cả khi lỗi).
Trên macOS, LightGBM cần OpenMP: `brew install libomp`.
Dữ liệu là TỔNG HỢP (giống bài 54 đến 60). Người duyệt RV-001 trong lab là MÔ PHỎNG: nó chốt theo nhãn cuối của bộ sinh
(luôn đúng), nên lab kiểm đường ống và các ràng buộc, không đo độ chính xác của người thật. Điểm rủi ro là đầu vào cho người
duyệt; chỉ người duyệt mới chốt `confirmed_fraud` hoặc `cleared`. SHAP giải thích điểm thô CHƯA hiệu chỉnh của mô hình, không phải p.
"""

import os
import sys
import time
from datetime import timedelta
from pathlib import Path

import lightgbm
import numpy as np
import polars as pl
import psycopg
import shap
import sklearn

from explain import FEATURE_GROUP, GROUPS, Explainer
from features import FEATURES, day_of
from model import HANDLING_COST_VND, REVIEW_COST_VND, expected_cost_flag
from pii import scrub
from prepare import OUT, load_table
from review import COMPOSE, PG_URL, REVIEWER_ID, SCHEMA, SYSTEM_ACTOR, AlreadyDecided, Flag, ReviewStore, connect
from rules import RuleEngine, load_config
from schedule import Item, day_start, schedule_reviews
from train import ensure_model, in_memory, train

HERE = Path(__file__).parent
WINDOW_FROM = 50
PER_DAY = 70  # công suất người duyệt dùng cho lần chạy thật trên PostgreSQL
SLA = timedelta(hours=24)
TYPES = ("ghost_trip", "gps_spoof", "promo_abuse", "collusion")
SEEDS = (42, 7, 123)

# Dấu vết mong đợi của từng loại, lấy từ hình 1 của bài 54 (cột «một chuyến» và cột «nhiều chuyến»), KHÔNG lấy từ mô hình.
EXPECTED = {
    "ghost_trip": {"route_ratio", "hav_start_end_km", "speed_kmh", "duration_s", "distance_km", "dev_trips_1h", "dev_trips_24h",
                   "pair_trips_7d", "dev_customers_24h"},
    "gps_spoof": {"start_pickup_gap_km", "drv_jump_km", "drv_jump_speed_kmh", "drv_gap_s", "impossible_jump"},
    "promo_abuse": {"has_newuser_promo", "has_promo", "discount_ratio", "amount_vnd", "dev_newuser_7d", "dev_customers_7d", "dev_customers_24h"},
    "collusion": {"same_device", "pair_trips_7d"},
}


# Bản đồ HẸP: chỉ các đặc trưng được nêu thẳng trong hình 1 bài 54 (cột «một chuyến» và «nhiều chuyến»), không gồm đặc trưng phụ như speed_kmh,
# duration_s, amount_vnd. Dùng để kiểm EXPECTED (rộng hơn) có làm số trúng đẹp lên không.
NARROW = {
    "ghost_trip": {"route_ratio", "hav_start_end_km", "dev_trips_1h", "dev_trips_24h", "pair_trips_7d"},
    "gps_spoof": {"start_pickup_gap_km", "drv_jump_km", "drv_jump_speed_kmh"},
    "promo_abuse": {"has_newuser_promo", "discount_ratio", "dev_newuser_7d", "dev_customers_7d"},
    "collusion": {"same_device", "pair_trips_7d"},
}


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


def load_avg() -> str:
    return " ".join(f"{x:.2f}" for x in os.getloadavg())


def fail_no_db() -> None:
    print(f"FAIL: không nối được PostgreSQL tại {PG_URL} ({COMPOSE})")
    sys.exit(1)


def rank(x: np.ndarray) -> np.ndarray:
    return np.argsort(np.argsort(x)).astype(float)


def random_hit(n_expected: int, k: int = 3, n_features: int = 30) -> float:
    """Xác suất k đặc trưng bốc ngẫu nhiên có ít nhất một đặc trưng nằm trong tập mong đợi."""
    return 1 - float(np.prod([(n_features - n_expected - m) / (n_features - m) for m in range(k)]))


# ---- Đo chất lượng lý do (dùng cho seed 42 và hai seed lặp lại) ---------------------------------------


def explain_stats(sm, ex: Explainer, fl: pl.DataFrame, p_fl: np.ndarray, train_ref: pl.DataFrame, rng: np.random.Generator) -> dict:
    """Trên các chuyến bị gắn cờ: lý do SHAP có chỉ đúng dấu vết của loại gian lận không, và xóa lý do thì điểm tụt bao nhiêu."""
    names = sm.feature_list
    contrib = ex.contributions(fl)
    ftype = fl["fraud_type"].to_numpy()
    tp = np.flatnonzero(ftype != "none")
    gain = sm.booster.feature_importance("gain")
    order = np.argsort(-gain)
    g1, g3 = {names[order[0]]}, {names[j] for j in order[:3]}
    per_type, l1, gl1, l3, gl3 = {}, [], [], [], []
    for t in TYPES:
        sel = tp[ftype[tp] == t]
        exp = EXPECTED[t]
        h1 = np.array([names[int(np.argmax(contrib[i]))] in exp for i in sel])
        h3 = np.array([bool({names[j] for j in np.argsort(-contrib[i])[:3]} & exp) for i in sel])
        per_type[t] = dict(n=len(sel), top1=float(h1.mean()), top3=float(h3.mean()), g1=bool(g1 & exp), g3=bool(g3 & exp), rnd=random_hit(len(exp)))
        l1 += list(h1)
        l3 += list(h3)
        gl1 += [bool(g1 & exp)] * len(sel)
        gl3 += [bool(g3 & exp)] * len(sel)
    l1, gl1, l3, gl3 = map(np.array, (l1, gl1, l3, gl3))
    b1, b3 = [], []
    for _ in range(1000):
        s = rng.integers(0, len(tp), len(tp))
        b1.append(l1[s].mean() - gl1[s].mean())
        b3.append(l3[s].mean() - gl3[s].mean())
    # xóa lý do: đặt đặc trưng về trung vị của train rồi chấm lại
    med = np.array([train_ref[f].median() for f in names])
    xs = fl.select(names).to_numpy()[tp]

    def rescore(cols: np.ndarray) -> np.ndarray:
        x = xs.copy()
        for r_, cc in enumerate(cols):
            x[r_, cc] = med[cc]
        return sm.predict_proba(pl.DataFrame(x, schema=names))

    top1 = np.array([[int(np.argmax(contrib[i]))] for i in tp])
    top3 = np.array([np.argsort(-contrib[i])[:3] for i in tp])
    rnd1 = [rescore(rng.integers(0, 30, size=(len(tp), 1))) for _ in range(10)]
    rnd3 = [rescore(np.array([rng.choice(30, 3, replace=False) for _ in tp])) for _ in range(10)]
    r1, r3 = rescore(top1), rescore(top3)
    amt = fl["amount_vnd"].to_numpy().astype(float)[tp]
    kept = lambda p_: float(np.mean(expected_cost_flag(p_, amt)))  # noqa: E731 - còn bị gắn cờ theo quy tắc chi phí kỳ vọng
    narrow1 = np.array([names[int(np.argmax(contrib[i]))] in NARROW[ftype[i]] for i in tp])
    narrow3 = np.array([bool({names[j] for j in np.argsort(-contrib[i])[:3]} & NARROW[ftype[i]]) for i in tp])
    rnd_narrow3 = float(np.mean([random_hit(len(NARROW[ftype[i]])) for i in tp]))
    return dict(narrow1=float(narrow1.mean()), narrow3=float(narrow3.mean()), rnd_narrow3=rnd_narrow3,
        contrib=contrib, tp=tp, per_type=per_type, g1=g1, g3=g3,
        local1=float(l1.mean()), glob1=float(gl1.mean()), local3=float(l3.mean()), glob3=float(gl3.mean()),
        ci1=tuple(np.percentile(b1, [2.5, 97.5])), ci3=tuple(np.percentile(b3, [2.5, 97.5])),
        p0=float(p_fl[tp].mean()), keep0=kept(p_fl[tp]),
        del1=float(r1.mean()), keep1=kept(r1), rnd1=float(np.mean([r.mean() for r in rnd1])), rkeep1=float(np.mean([kept(r) for r in rnd1])),
        del3=float(r3.mean()), keep3=kept(r3), rnd3=float(np.mean([r.mean() for r in rnd3])), rkeep3=float(np.mean([kept(r) for r in rnd3])),
    )


# ---- [1] Môi trường, dữ liệu, mô hình --------------------------------------------------------------


def step_setup(conn: psycopg.Connection, c: Checker) -> dict:
    say("[1] Môi trường, dữ liệu, mô hình")
    version = conn.execute("SHOW server_version").fetchone()["server_version"]
    say(f"Python {sys.version.split()[0]}, shap {shap.__version__}, lightgbm {lightgbm.__version__}, scikit-learn {sklearn.__version__}, "
        f"numpy {np.__version__}, polars {pl.__version__}, psycopg {psycopg.__version__}, PostgreSQL {version}")
    t0 = time.perf_counter()
    table = load_table()
    scoring, trained = ensure_model(table, OUT)
    say(f"bảng chuyến {table.height:,} dòng; mô hình {scoring.version} ({'vừa huấn luyện' if trained else 'nạp từ out/'}, {time.perf_counter() - t0:.1f} giây); "
        f"quy tắc gắn cờ: p x (tiền chuyến + {HANDLING_COST_VND:,}) > {REVIEW_COST_VND:,} (ngưỡng p chung {scoring.threshold:.3f} chỉ để so sánh)")
    day = table.select(day_of("paid_at").alias("d"))["d"].to_numpy()
    window = table.filter(pl.Series(day >= WINDOW_FROM))
    train_ref = table.filter(pl.Series(day < 30))
    p = scoring.predict_proba(window)
    flagged = scoring.flag(p, window['amount_vnd'].to_numpy().astype(float))
    n_thr = int((p >= scoring.threshold).sum())
    y = window["is_fraud"].to_numpy().astype(bool)
    say(f"cửa sổ ngày {WINDOW_FROM} đến 60: {window.height:,} chuyến, {int(y.sum())} gian lận; {int(flagged.sum())} chuyến vào hàng đợi (ngưỡng chung sẽ cho {n_thr}) "
        f"({int((flagged & y).sum())} gian lận thật, precision {(flagged & y).sum() / flagged.sum():.3f}, recall {(flagged & y).sum() / y.sum():.3f}); nhãn dùng là nhãn cuối")
    c.check("mô hình nạp được, cửa sổ 33.416 chuyến", window.height == 33_416 and scoring.version.startswith("fraud-lgbm-"))
    return {"table": table, "window": window, "train": train_ref, "scoring": scoring, "p": p, "flagged": flagged, "y": y}


# ---- [2] SHAP --------------------------------------------------------------------------------------


def step_shap(d: dict, c: Checker) -> Explainer:
    say("\n[2] SHAP: TreeExplainer trên LightGBM gốc")
    window, scoring, p = d["window"], d["scoring"], d["p"]
    names = scoring.feature_list
    ex = Explainer(scoring)
    t0 = time.perf_counter()
    contrib = ex.contributions(window)
    t_all = time.perf_counter() - t0
    raw = ex.raw_score(window)
    err_add = float(np.abs(ex.base_value + contrib.sum(axis=1) - raw).max())
    native, base_native = ex.native_contributions(window)
    err_native = float(np.abs(native - contrib).max())
    cal = ex.calibrated(window)
    err_cal = float(np.abs(cal - p).max())
    k, b0 = ex.log_odds_map()
    logit = np.log(p / (1 - p))
    err_aff = float(np.abs(logit - (b0 + k * raw)).max())
    say(f"{window.height:,} chuyến x {len(names)} đặc trưng trong {t_all:.2f} giây; base_value = {ex.base_value:.3f} (log-odds)")
    say(f"base_value + tổng đóng góp so với điểm thô của LightGBM: lệch tối đa {err_add:.1e}")
    say(f"gói shap so với TreeSHAP gốc của LightGBM (ScoringModel.contributions, bài 57): lệch đóng góp tối đa {err_native:.1e}")
    say(f"điểm thô qua bộ hiệu chỉnh sigmoid so với predict_proba của mô hình lưu: lệch tối đa {err_cal:.1e}")
    say(f"log-odds của p so với {b0:.3f} + {k:.3f} x điểm thô: lệch tối đa {err_aff:.1e}")
    i = int(np.argmax(d["flagged"] & d["y"]))
    say(f"một chuyến bị gắn cờ: điểm thô {raw[i]:.3f}, p gửi cho người duyệt {p[i]:.4f} (log-odds {logit[i]:.3f}); SHAP cộng ra điểm thô {raw[i]:.3f}, không phải {logit[i]:.3f}")
    one = window[:200]
    t0 = time.perf_counter()
    for j in range(200):
        ex.contributions(one[j:j + 1])
    t_one = (time.perf_counter() - t0) / 200
    fl = window.filter(pl.Series(d["flagged"]))
    t0 = time.perf_counter()
    ex.contributions(fl)
    t_fl = time.perf_counter() - t0
    say(f"thời gian: từng chuyến {1000 * t_one:.1f} ms (trung bình 200 lần gọi); cả lô {fl.height} chuyến bị gắn cờ {1000 * t_fl:.0f} ms (load average 1/5/15 phút: {load_avg()})")
    mean_abs = np.abs(contrib).mean(axis=0)
    gain = scoring.booster.feature_importance("gain")
    corr = float(np.corrcoef(rank(mean_abs), rank(gain))[0, 1])
    top_s, top_g = np.argsort(-mean_abs)[:6], np.argsort(-gain)[:6]
    say("6 đặc trưng đầu theo trung bình |SHAP| (cả cửa sổ): " + ", ".join(f"{names[j]} {mean_abs[j]:.2f}" for j in top_s))
    say("6 đặc trưng đầu theo gain khi huấn luyện:             " + ", ".join(f"{names[j]} {gain[j] / gain.sum():.1%}" for j in top_g))
    say(f"tương quan hạng giữa hai cách xếp (30 đặc trưng): {corr:.2f}")
    c.check("base_value + tổng SHAP bằng điểm thô của LightGBM (lệch dưới 1e-6)", err_add < 1e-6, err_add)
    c.check("gói shap khớp TreeSHAP gốc của LightGBM (dưới 1e-6) và cùng hằng số nền", err_native < 1e-6 and abs(base_native[0] - ex.base_value) < 1e-6, (err_native, base_native[0], ex.base_value))
    c.check("điểm thô qua hiệu chỉnh sigmoid khớp predict_proba của mô hình lưu (dưới 1e-9)", err_cal < 1e-9, err_cal)
    c.check("log-odds của p là hàm affine của điểm thô (dưới 1e-6): SHAP nhân hệ số này mới ra thang của p", err_aff < 1e-6, err_aff)
    c.check("tổng SHAP không cộng ra log-odds của p (hệ số khác 1, hằng số khác 0)", abs(k - 1) > 0.01 and abs(b0) > 0.1, (k, b0))
    c.check("SHAP ra đúng 30 cột theo feature_list", contrib.shape == (window.height, 30) and names == list(FEATURES))
    d["contrib"], d["raw"] = contrib, raw
    return ex


# ---- [3] Lý do cho người duyệt ---------------------------------------------------------------------------


def step_reasons(d: dict, ex: Explainer, c: Checker) -> dict:
    say("\n[3] Từ SHAP đến reasons[]")
    window, scoring, p = d["window"], d["scoring"], d["p"]
    names = scoring.feature_list
    idx = np.flatnonzero(d["flagged"])
    fl = window[idx]
    reasons = ex.reasons(fl)
    ftype = fl["fraud_type"].to_numpy()
    for t in ("ghost_trip", "gps_spoof", "collusion"):
        j = int(np.argmax(ftype == t))
        say(f"ví dụ {t}, p = {p[idx[j]]:.3f}:")
        for r in reasons[j]:
            say(f"    {r.shap:+.2f}  {r.text}")
    j = int(np.argmax(ftype == "none"))
    say(f"ví dụ chuyến sạch bị gắn cờ nhầm, p = {p[idx[j]]:.3f}:")
    for r in reasons[j]:
        say(f"    {r.shap:+.2f}  {r.text}")
    c.check("mọi chuyến bị gắn cờ có ít nhất một lý do", all(len(r) >= 1 for r in reasons))
    c.check("lý do sắp theo đóng góp giảm dần và đều dương", all(all(a.shap >= b.shap > 0 for a, b in zip(r, r[1:])) and all(x.shap > 0 for x in r) for r in reasons))

    rng = np.random.default_rng(0)
    st = explain_stats(scoring, ex, fl, p[idx], d["train"], rng)
    tp = st["tp"]
    say(f"\nlý do có chỉ đúng cơ chế? Chuyến gian lận bị gắn cờ: {len(tp)}. Đặc trưng SHAP đứng đầu (top-1) hoặc trong ba đứng đầu (top-3) có nằm trong dấu vết mong đợi của loại đó (hình 1 bài 54)?")
    say(f"{'loại':12s}{'chuyến':>8s}{'top-1 SHAP':>12s}{'top-1 gain':>12s}{'top-3 SHAP':>12s}{'top-3 gain':>12s}{'top-3 ngẫu nhiên':>18s}")
    for t in TYPES:
        r = st["per_type"][t]
        say(f"{t:12s}{r['n']:8d}{r['top1']:12.1%}{(100 if r['g1'] else 0):11d} %{r['top3']:12.1%}{(100 if r['g3'] else 0):11d} %{r['rnd']:18.1%}")
    say(f"«gain» là bộ đặc trưng cố định cho mọi chuyến: top-1 = {', '.join(sorted(st['g1']))}; top-3 = {', '.join(sorted(st['g3']))} (xếp theo gain khi huấn luyện)")
    say(f"gộp bốn loại: top-1 SHAP {st['local1']:.1%} so với top-1 gain {st['glob1']:.1%}, chênh {st['local1'] - st['glob1']:+.1%} (khoảng tin cậy 95 % [{st['ci1'][0]:+.1%}, {st['ci1'][1]:+.1%}]); "
        f"top-3 SHAP {st['local3']:.1%} so với top-3 gain {st['glob3']:.1%}, chênh {st['local3'] - st['glob3']:+.1%} ([{st['ci3'][0]:+.1%}, {st['ci3'][1]:+.1%}]) (bootstrap 1.000 lần)")
    say("dấu vết «mong đợi» là bản đồ ta viết từ hình 1 bài 54, rộng hơn hình ở vài đặc trưng phụ; kết quả với bản đồ hẹp in ngay dưới.")
    say(f"bản đồ HẸP (chỉ đặc trưng nêu thẳng trong hình 1 bài 54, bỏ speed_kmh, duration_s, amount_vnd...): top-1 SHAP {st['narrow1']:.1%}, top-3 SHAP {st['narrow3']:.1%}, top-3 ngẫu nhiên {st['rnd_narrow3']:.1%}")

    say(f"\nxóa lý do (đặt về trung vị train), trên {len(tp)} chuyến gian lận bị gắn cờ; p trung bình trước khi xóa {st['p0']:.3f}, còn bị gắn cờ {st['keep0']:.0%}:")
    say(f"{'xóa':26s}{'p trung bình':>14s}{'còn gắn cờ':>12s}")
    say(f"{'đặc trưng lý do số 1':26s}{st['del1']:14.3f}{st['keep1']:12.0%}")
    say(f"{'1 đặc trưng ngẫu nhiên':26s}{st['rnd1']:14.3f}{st['rkeep1']:12.0%}  (trung bình 10 lần bốc)")
    say(f"{'ba đặc trưng lý do đầu':26s}{st['del3']:14.3f}{st['keep3']:12.0%}")
    say(f"{'3 đặc trưng ngẫu nhiên':26s}{st['rnd3']:14.3f}{st['rkeep3']:12.0%}  (trung bình 10 lần bốc)")

    groups = ex.group_contributions(st["contrib"][tp])
    gnames = list(groups)
    top_g = np.argmax(np.column_stack([groups[g] for g in gnames]), axis=1)
    say("\nhọ tín hiệu có đóng góp lớn nhất, theo loại gian lận (số chuyến):")
    say(f"{'loại':12s}" + "".join(f"{n:>12s}" for n in gnames))
    for t in TYPES:
        sel = ftype[tp] == t
        say(f"{t:12s}" + "".join(f"{int((top_g[sel] == k).sum()):12d}" for k in range(len(gnames))))

    # SHAP lộ ra mô hình dựa vào một tạo tác của bộ sinh dữ liệu: cust_promo_share_30d > 0 «tố» chuyến sạch
    contrib_w = d["contrib"]
    j_ps = names.index("cust_promo_share_30d")
    v = window["cust_promo_share_30d"].to_numpy()
    yw = d["y"]
    say(f"\ncust_promo_share_30d trong cửa sổ: > 0 ở {np.mean(v[~yw] > 0):.1%} chuyến sạch và {np.mean(v[yw] > 0):.1%} chuyến gian lận (khách gian lận không dùng mã nào ngoài NEWUSER50 của promo_abuse: tạo tác của bộ sinh);")
    say(f"đóng góp SHAP trung bình của nó: {contrib_w[v > 0, j_ps].mean():+.2f} khi > 0, {contrib_w[v == 0, j_ps].mean():+.2f} khi = 0")
    c.check("top-1 SHAP trúng dấu vết mong đợi nhiều hơn top-1 gain cố định (khoảng tin cậy dưới > 0)", st["ci1"][0] > 0, st["ci1"])
    c.check("top-3 SHAP trúng dấu vết mong đợi trên 90 % và cao hơn ngẫu nhiên ở mọi loại", st["local3"] > 0.9 and all(st["per_type"][t]["top3"] > st["per_type"][t]["rnd"] for t in TYPES), st["per_type"])
    c.check("xóa ba lý do đầu hạ p trung bình sâu hơn xóa ba đặc trưng ngẫu nhiên", st["del3"] < st["rnd3"] - 0.05, (st["del3"], st["rnd3"]))
    c.check("xóa lý do số 1 hạ p trung bình sâu hơn xóa một đặc trưng ngẫu nhiên", st["del1"] < st["rnd1"], (st["del1"], st["rnd1"]))
    c.check("mọi đặc trưng thuộc đúng một họ tín hiệu", set(FEATURE_GROUP) == set(names) and sum(len(x) for x in GROUPS.values()) == 30)
    c.check("tạo tác: cust_promo_share_30d > 0 nhiều hơn hẳn ở chuyến sạch, và SHAP của nó âm khi > 0", np.mean(v[~yw] > 0) > 2 * np.mean(v[yw] > 0) and contrib_w[v > 0, j_ps].mean() < 0)
    d["idx"], d["fl"], d["reasons"] = idx, fl, reasons
    return d


# ---- [4] Hàng đợi trên PostgreSQL ------------------------------------------------------------------------


def build_flags(d: dict, version: str):
    fl, reasons, p, idx = d["fl"], d["reasons"], d["p"], d["idx"]
    cfg = load_config(HERE / "rules.yaml")
    engine = RuleEngine.from_config(cfg)
    flags, hits_n = [], 0
    paid = fl["paid_at"].to_list()
    amount = fl["amount_vnd"].to_list()
    rows = fl.select(FEATURES).to_dicts()
    for k in range(fl.height):
        v = engine.score(rows[k])
        hits_n += len(v.hits)
        score = float(p[idx[k]])
        flags.append(Flag(
            trip_id=fl["trip_id"][k], model_version=version, rules_version=cfg.version, score=score,
            amount_vnd=int(amount[k]), priority=round(score * (amount[k] + HANDLING_COST_VND), 2),
            reasons={"model": [r.as_dict() for r in reasons[k]],
                     "rules": [{"rule_id": h.rule_id, "points": h.points, "text": h.reason} for h in v.hits],
                     "rules_score": round(v.score, 1)},
            enqueued_at=paid[k], due_at=paid[k] + SLA))
    return flags, cfg.version, hits_n


def step_queue(conn: psycopg.Connection, d: dict, c: Checker) -> dict:
    say(f"\n[4] Hàng đợi người duyệt trên PostgreSQL, schema {SCHEMA}")
    store = ReviewStore(conn)
    store.create_schema()
    flags, rules_version, hits_n = build_flags(d, d["scoring"].version)
    say(f"dựng {len(flags)} đề xuất: reasons của mô hình (SHAP) cộng lý do của rule engine bài 56 ({hits_n} lượt rule bắn, phiên bản cấu hình {rules_version})")
    t0 = time.perf_counter()
    ids = [store.enqueue(f) for f in flags]
    say(f"đã ghi {sum(i is not None for i in ids)} đề xuất và {sum(i is not None for i in ids)} sự kiện flagged ({time.perf_counter() - t0:.1f} giây)")
    again = [store.enqueue(f) for f in flags[:50]]
    n = conn.execute(f"SELECT count(*) AS n FROM {SCHEMA}.review_queue").fetchone()["n"]
    say(f"chạy lại 50 đề xuất đầu (như consumer nhận trùng): {sum(i is None for i in again)} bị bỏ qua, hàng đợi vẫn {n} dòng")
    row = conn.execute(f"SELECT queue_id, trip_id, model_version, rules_version, score, priority, status, enqueued_at, due_at, reasons FROM {SCHEMA}.review_queue ORDER BY priority DESC LIMIT 1").fetchone()
    say("một dòng review_queue (ưu tiên cao nhất, rút gọn lý do):")
    say(f"  queue_id={row['queue_id']} trip_id={row['trip_id']} model={row['model_version']} rules_version={row['rules_version']} score={row['score']} "
        f"priority={row['priority']} status={row['status']}")
    say(f"  enqueued_at={row['enqueued_at'].isoformat()} due_at={row['due_at'].isoformat()}")
    for r in row["reasons"]["model"]:
        say(f"  model  {r['shap']:+.2f} {r['feature']}={r['value']}: {r['text']}")
    for r in row["reasons"]["rules"][:2]:
        say(f"  rule   {r['points']:>3d} {r['rule_id']}: {r['text']}")
    c.check("hàng đợi có đủ đề xuất, chạy lại không nhân đôi", n == len(flags) and all(i is None for i in again))
    c.check("mỗi đề xuất có đúng một sự kiện flagged do dịch vụ chấm điểm ghi", conn.execute(
        f"SELECT count(*) AS n FROM {SCHEMA}.decision_log WHERE event='flagged' AND actor=%s", (SYSTEM_ACTOR,)).fetchone()["n"] == len(flags))

    items = [Item(k, flags[k].enqueued_at, flags[k].priority) for k in range(len(flags))]
    sched = schedule_reviews(items, per_day=PER_DAY, policy="priority", first_day=WINDOW_FROM, last_day=59)
    fraud = d["fl"]["is_fraud"].to_numpy().astype(bool)
    t0 = time.perf_counter()
    for k, at in sorted(sched.items(), key=lambda kv: kv[1]):
        store.decide(ids[k], REVIEWER_ID, "confirmed_fraud" if fraud[k] else "cleared", "mô phỏng: chốt theo nhãn cuối", at)
    status = {r["status"]: r["n"] for r in conn.execute(f"SELECT status, count(*) AS n FROM {SCHEMA}.review_queue GROUP BY status")}
    say(f"RV-001 (mô phỏng, {PER_DAY} đề xuất mỗi ngày làm việc 08:00-17:00, ưu tiên theo chi phí kỳ vọng) chốt {len(sched)} đề xuất trong {time.perf_counter() - t0:.1f} giây: {status}")
    c.check("số đề xuất đã chốt khớp lịch mô phỏng, phần còn lại là tồn đọng", status.get("confirmed_fraud", 0) + status.get("cleared", 0) == len(sched) and status.get("needs_review", 0) == len(flags) - len(sched))
    c.check("confirmed_fraud khớp số chuyến gian lận trong số đã chốt", status.get("confirmed_fraud", 0) == int(sum(fraud[k] for k in sched)))
    return {"store": store, "flags": flags, "ids": ids, "items": items, "sched": sched, "fraud": fraud, "n_decided_before": len(sched)}


# ---- [5] Ràng buộc: ai được chốt, sửa, xóa -------------------------------------------------------------


def expect_db_error(conn: psycopg.Connection, sql: str, params: tuple, kind, needle: str = "") -> str:
    try:
        conn.execute(sql, params)
    except kind as e:
        msg = str(e).splitlines()[0]
        return msg if needle in msg else f"lỗi khác: {msg}"
    except psycopg.Error as e:
        return f"loại lỗi khác {type(e).__name__}: {str(e).splitlines()[0]}"
    return "KHÔNG LỖI"


def step_guards(conn: psycopg.Connection, q: dict, c: Checker) -> None:
    say("\n[5] Ràng buộc ở cơ sở dữ liệu: ai được chốt, sửa, xóa, ghi log")
    store, ids, sched = q["store"], q["ids"], q["sched"]
    open_id = next(i for k, i in enumerate(ids) if k not in sched)
    decided_id = conn.execute(f"SELECT queue_id FROM {SCHEMA}.review_queue WHERE status = 'confirmed_fraud' ORDER BY queue_id LIMIT 1").fetchone()["queue_id"]
    now = day_start(60)
    E = psycopg.errors
    cases: list[tuple[str, str, str]] = []

    def case(label: str, short: str, got: str) -> None:
        cases.append((label, short, got))
        say(f"  {label}: {got}")

    conn.execute(f"INSERT INTO {SCHEMA}.reviewer (reviewer_id, display_name, active) VALUES ('RV-009', 'Người duyệt đã nghỉ', false)")
    oq = conn.execute(f"SELECT * FROM {SCHEMA}.review_queue WHERE queue_id=%s", (open_id,)).fetchone()
    dq = conn.execute(f"SELECT * FROM {SCHEMA}.review_queue WHERE queue_id=%s", (decided_id,)).fetchone()
    upd = f"UPDATE {SCHEMA}.review_queue SET status=%s, decided_by=%s, decided_at=%s WHERE queue_id=%s"
    case("dịch vụ chấm điểm tự chốt confirmed_fraud", "dịch vụ tự chốt", expect_db_error(conn, upd, ("confirmed_fraud", SYSTEM_ACTOR, now, open_id), (E.ForeignKeyViolation, E.RaiseException)))
    case("người duyệt chưa đăng ký (RV-002) chốt cleared", "người duyệt chưa đăng ký", expect_db_error(conn, upd, ("cleared", "RV-002", now, open_id), (E.ForeignKeyViolation, E.RaiseException)))
    case("người duyệt đã nghỉ (RV-009, active = false) chốt cleared", "người duyệt không hoạt động", expect_db_error(conn, upd, ("cleared", "RV-009", now, open_id), E.RaiseException, "không hoạt động"))
    case("confirmed_fraud mà không ghi ai chốt", "chốt không có người chốt", expect_db_error(conn, upd, ("confirmed_fraud", None, now, open_id), E.CheckViolation))
    case("chốt với thời điểm trước lúc vào hàng đợi", "chốt trước lúc vào hàng đợi", expect_db_error(conn, upd, ("cleared", REVIEWER_ID, oq["enqueued_at"].replace(year=2000), open_id), E.CheckViolation))
    case("sửa trực tiếp đề xuất đã chốt (confirmed_fraud -> cleared)", "sửa đề xuất đã chốt", expect_db_error(conn, f"UPDATE {SCHEMA}.review_queue SET status='cleared' WHERE queue_id=%s", (decided_id,), E.RaiseException, "đã chốt"))
    case("đổi điểm của đề xuất đang chờ", "đổi điểm đề xuất", expect_db_error(conn, f"UPDATE {SCHEMA}.review_queue SET score=0.99 WHERE queue_id=%s", (open_id,), E.RaiseException, "không đổi nội dung"))
    case("đổi mức ưu tiên của đề xuất đang chờ", "đổi ưu tiên đề xuất", expect_db_error(conn, f"UPDATE {SCHEMA}.review_queue SET priority=0 WHERE queue_id=%s", (open_id,), E.RaiseException, "không đổi nội dung"))
    case("xóa dòng review_queue", "xóa review_queue", expect_db_error(conn, f"DELETE FROM {SCHEMA}.review_queue WHERE queue_id=%s", (open_id,), E.RaiseException, "không cho xóa"))
    ins_q = (f"INSERT INTO {SCHEMA}.review_queue (trip_id, model_version, rules_version, score, amount_vnd, priority, reasons, status, enqueued_at, due_at, decided_at, decided_by) "
             "VALUES ('TN-9999999', 'm', 1, 0.5, 1, 1, '{}', 'confirmed_fraud', %s, %s, %s, %s)")
    case("INSERT thẳng một đề xuất đã ở trạng thái confirmed_fraud", "INSERT đề xuất đã chốt", expect_db_error(conn, ins_q, (oq["enqueued_at"], oq["due_at"], oq["enqueued_at"], REVIEWER_ID), E.RaiseException, "chỉ được vào ở needs_review"))
    case("sửa decision_log", "sửa decision_log", expect_db_error(conn, f"UPDATE {SCHEMA}.decision_log SET note='x' WHERE queue_id=%s", (decided_id,), E.RaiseException, "chỉ ghi thêm"))
    case("xóa decision_log", "xóa decision_log", expect_db_error(conn, f"DELETE FROM {SCHEMA}.decision_log WHERE queue_id=%s", (decided_id,), E.RaiseException, "chỉ ghi thêm"))
    case("TRUNCATE decision_log", "TRUNCATE decision_log", expect_db_error(conn, f"TRUNCATE {SCHEMA}.decision_log", (), E.RaiseException, "chỉ ghi thêm"))
    ins = (f"INSERT INTO {SCHEMA}.decision_log (at, event, queue_id, trip_id, actor, model_version, rules_version, score) VALUES (%s, %s, %s, %s, %s, %s, %s, %s)")
    ok_vals = (oq["queue_id"], oq["trip_id"], None, oq["model_version"], oq["rules_version"], oq["score"])

    def log_args(event: str, actor: str, *, at=None, score=None, queue_id=None, trip=None):
        return (at or oq["enqueued_at"], event, queue_id or oq["queue_id"], trip or oq["trip_id"], actor, oq["model_version"], oq["rules_version"], score if score is not None else oq["score"])

    case("ghi sự kiện confirmed_fraud với actor là dịch vụ chấm điểm", "actor sai cho quyết định", expect_db_error(conn, ins, log_args("confirmed_fraud", SYSTEM_ACTOR), (E.CheckViolation, E.RaiseException)))
    case("ghi sự kiện flagged với actor là người duyệt", "actor sai cho flagged", expect_db_error(conn, ins, log_args("flagged", REVIEWER_ID), E.CheckViolation))
    case("ghi hai lần cùng sự kiện flagged cho một đề xuất", "trùng sự kiện", expect_db_error(conn, ins, log_args("flagged", SYSTEM_ACTOR), E.UniqueViolation))
    case("ghi log có điểm khác điểm trong hàng đợi (log giả mạo)", "log lệch điểm", expect_db_error(conn, ins, log_args("flagged", SYSTEM_ACTOR, score=0.01), E.RaiseException, "không khớp đề xuất"))
    case("ghi log chốt cleared cho đề xuất đã là confirmed_fraud", "log chốt ngược trạng thái", expect_db_error(
        conn, ins, (dq["decided_at"], "cleared", dq["queue_id"], dq["trip_id"], REVIEWER_ID, dq["model_version"], dq["rules_version"], dq["score"]), E.RaiseException, "không khớp quyết định"))
    case("ghi log chốt cho đề xuất còn needs_review (chưa có quyết định trong hàng đợi)", "log chốt không có quyết định", expect_db_error(conn, ins, log_args("cleared", REVIEWER_ID, at=now), E.RaiseException, "không khớp quyết định"))
    try:
        store.decide(decided_id, REVIEWER_ID, "cleared", "", now)
        again = "KHÔNG LỖI"
    except AlreadyDecided as e:
        again = f"AlreadyDecided: {e}"
    case("hai lần chốt cùng đề xuất qua store.decide", "chốt hai lần", again)
    del ok_vals
    for label, short, got in cases:
        c.check(f"chặn: {short}", not got.startswith(("KHÔNG", "lỗi khác", "loại lỗi khác")), got)

    note_in = "CCCD 099123456789 dùng ở hai tài khoản, khách gọi từ 0900 000 001 hoặc 09 91 23 45 67 89, cước 125.000.000 VND ngày 2026-03-02 10:00"
    store.decide(open_id, REVIEWER_ID, "confirmed_fraud", note_in, now)
    stored = conn.execute(f"SELECT decision_note FROM {SCHEMA}.review_queue WHERE queue_id=%s", (open_id,)).fetchone()["decision_note"]
    logged = conn.execute(f"SELECT note FROM {SCHEMA}.decision_log WHERE queue_id=%s AND event='confirmed_fraud'", (open_id,)).fetchone()["note"]
    say(f"\nghi chú người duyệt gõ:  {note_in}")
    say(f"ghi chú được lưu:        {stored}")
    # quét: mọi trường văn bản đã lưu phải bất động với scrub (không còn gì để che), kể cả chuỗi số xen dấu phân cách
    texts = [r["decision_note"] or "" for r in conn.execute(f"SELECT decision_note FROM {SCHEMA}.review_queue")]
    texts += [r["reasons"] for r in conn.execute(f"SELECT reasons::text AS reasons FROM {SCHEMA}.review_queue")]
    texts += [(r["note"] or "") + (r["reasons"] or "") for r in conn.execute(f"SELECT note, reasons::text AS reasons FROM {SCHEMA}.decision_log")]
    leaks = sum(1 for t in texts if scrub(t) != t)
    say(f"quét {len(texts):,} trường văn bản của cả hai bảng bằng chính hàm che (kể cả số xen dấu cách, chấm, gạch): {leaks} trường còn có thể che thêm")
    c.check("ghi chú được che trước khi lưu (099*******89, 090*****01), tiền có dấu chấm ngăn nghìn và ngày giờ giữ nguyên", stored == scrub(note_in) and stored.count("099*******89") == 2 and "090*****01" in stored and "125.000.000" in stored and "2026-03-02 10:00" in stored, stored)
    c.check("nhật ký lưu đúng chuỗi đã che", logged == stored)
    c.check("không trường văn bản nào còn chuỗi số định danh có thể che thêm", leaks == 0, leaks)
    bad = conn.execute(f"""SELECT count(*) AS n FROM {SCHEMA}.review_queue q JOIN {SCHEMA}.decision_log l ON l.queue_id = q.queue_id AND l.event = q.status
                           WHERE q.status <> 'needs_review' AND (l.actor <> q.decided_by OR l.at <> q.decided_at)""").fetchone()["n"]
    nlog = conn.execute(f"""SELECT count(*) AS n FROM {SCHEMA}.review_queue q WHERE (SELECT count(*) FROM {SCHEMA}.decision_log l WHERE l.queue_id = q.queue_id)
                            <> CASE WHEN q.status = 'needs_review' THEN 1 ELSE 2 END""").fetchone()["n"]
    c.check("nhật ký khớp bảng hàng đợi: người chốt, thời điểm, và 1 hoặc 2 sự kiện mỗi đề xuất", bad == 0 and nlog == 0, (bad, nlog))
    q["n_decided_total"] = q["n_decided_before"] + 1  # thêm đề xuất vừa dùng để thử che ghi chú
    q["n_guard_cases"] = len(cases)


# ---- [6] Đo từ nhật ký --------------------------------------------------------------------------------


def money_share(items, fraud, loss, per_day, policy, first=50, last=59):
    s = schedule_reviews(items, per_day=per_day, policy=policy, first_day=first, last_day=last)
    caught = [k for k in s if fraud[k]]
    return s, float(loss[caught].sum() / loss[fraud].sum()), len(caught)


def step_measure(conn: psycopg.Connection, d: dict, q: dict, c: Checker) -> None:
    say("\n[6] Đo từ chính nhật ký: SLA, công suất, và dải điểm nào còn đáng duyệt")
    flags, items, fraud = q["flags"], q["items"], q["fraud"]
    end = day_start(60)
    rows = conn.execute(f"""SELECT count(*) AS n,
        count(*) FILTER (WHERE status <> 'needs_review') AS decided,
        count(*) FILTER (WHERE (status <> 'needs_review' AND decided_at > due_at) OR (status = 'needs_review' AND %s > due_at)) AS breach,
        percentile_cont(0.5) WITHIN GROUP (ORDER BY extract(epoch FROM decided_at - enqueued_at) / 3600) AS p50,
        percentile_cont(0.95) WITHIN GROUP (ORDER BY extract(epoch FROM decided_at - enqueued_at) / 3600) AS p95
        FROM {SCHEMA}.review_queue""", (end,)).fetchone()
    say(f"lần chạy thật ({PER_DAY}/ngày, ưu tiên; gồm 1 đề xuất chốt ở mục 5): {rows['n']} đề xuất, {rows['decided']} đã chốt, tồn đọng {rows['n'] - rows['decided']}; "
        f"vi phạm SLA 24 giờ: {rows['breach']} ({rows['breach'] / rows['n']:.0%}); thời gian chờ của đề xuất đã chốt: trung vị {rows['p50']:.1f} giờ, p95 {rows['p95']:.1f} giờ")
    say(f"đề xuất vào hàng đợi trung bình {len(flags) / 10:.1f} mỗi ngày; công suất {PER_DAY}/ngày")

    loss = np.array([f.amount_vnd + HANDLING_COST_VND for f in flags], float)
    say(f"\nđổi công suất người duyệt (mô phỏng cùng {len(flags)} đề xuất, cùng lịch 08:00-17:00 mọi ngày dương lịch, kể cả cuối tuần; «tiền bắt được» = tổng (tiền chuyến + xử lý) của chuyến gian lận đã được duyệt, GIẢ SỬ duyệt trễ vẫn chặn đủ thiệt hại; «chờ p95» chỉ tính đề xuất đã được duyệt nên bị cắt khi còn tồn đọng):")
    say(f"{'công suất/ngày':>15s}{'chính sách':>11s}{'đã duyệt':>9s}{'tồn đọng':>9s}{'vi phạm SLA':>12s}{'chờ p95 đã duyệt (giờ)':>24s}{'gian lận bắt':>13s}{'tiền bắt được':>14s}")
    table = {}
    for per_day, policy in ((40, "priority"), (50, "priority"), (50, "fifo"), (65, "priority"), (65, "fifo"), (PER_DAY, "priority"), (100, "priority")):
        s, money, caught = money_share(items, fraud, loss, per_day, policy)
        waits = np.array([(t - items[k].enqueued_at).total_seconds() / 3600 for k, t in s.items()])
        breach = sum(1 for k in range(len(items)) if (s[k] > items[k].enqueued_at + SLA if k in s else end > items[k].enqueued_at + SLA))
        table[(per_day, policy)] = dict(done=len(s), breach=breach, caught=caught, money=money)
        say(f"{per_day:15d}{policy:>11s}{len(s):9d}{len(items) - len(s):9d}{breach:12d}{float(np.percentile(waits, 95)):24.1f}{caught:7d}/{int(fraud.sum())}{money:14.1%}")
    sim = table[(PER_DAY, "priority")]
    c.check("mô phỏng ở công suất của lần chạy thật khớp số đã chốt (cộng 1 đề xuất ở mục 5) và số vi phạm SLA đo từ PostgreSQL",
            sim["done"] + 1 == rows["decided"] and sim["breach"] == rows["breach"] - 0 or sim["done"] + 1 == rows["decided"] and abs(sim["breach"] - rows["breach"]) <= 1, (sim, dict(rows)))
    c.check("công suất thấp hơn lượng đề xuất thì tồn đọng và vi phạm SLA tăng", table[(40, "priority")]["breach"] > table[(100, "priority")]["breach"] and table[(40, "priority")]["done"] < table[(100, "priority")]["done"])
    c.check("công suất 100/ngày không bắt ít tiền hơn 40/ngày", table[(100, "priority")]["money"] >= table[(40, "priority")]["money"])

    sc = np.array([f.score for f in flags])
    say(f"\ndải điểm của các đề xuất (nhãn cuối; phí duyệt {REVIEW_COST_VND:,} VND cho MỌI đề xuất, bỏ sót = tiền chuyến + {HANDLING_COST_VND:,} VND):")
    say(f"{'dải p':>12s}{'đề xuất':>9s}{'gian lận':>9s}{'precision':>10s}{'tiền chặn được':>16s}{'chi phí duyệt':>15s}{'lãi ròng':>12s}")
    nets = []
    edges = [float(sc.min()), 0.3, 0.6, 1.01]
    for lo, hi in zip(edges, edges[1:]):
        m = (sc >= lo) & (sc < hi)
        gain, cost = float(loss[m & fraud].sum()), float(REVIEW_COST_VND * m.sum())
        nets.append(gain - cost)
        say(f"{f'[{lo:.3f},{min(hi, 1):.1f})':>12s}{int(m.sum()):9d}{int((m & fraud).sum()):9d}{(m & fraud).sum() / max(1, m.sum()):10.3f}{gain:16,.0f}{cost:15,.0f}{gain - cost:12,.0f}")
    be = {a: REVIEW_COST_VND / (a + HANDLING_COST_VND) for a in (30_000, 120_000, 500_000)}
    say("điểm hòa vốn p* = phí duyệt / (tiền chuyến + xử lý), vì phí duyệt trả cho mọi cờ: " + ", ".join(f"chuyến {a:,} VND -> {v:.3f}" for a, v in be.items()))
    flag_rate = len(flags) / d["window"].height
    hours = flag_rate * 2_000_000 * 5 / 60
    say(f"ngoại suy thô theo tỉ lệ: ở 2 triệu chuyến/ngày, cùng tỉ lệ gắn cờ {flag_rate:.2%} cho {flag_rate * 2_000_000:,.0f} đề xuất/ngày, "
        f"tức {hours:,.0f} giờ duyệt/ngày nếu mỗi đề xuất 5 phút (cửa sổ này có {d['window'].height / 10:,.0f} chuyến/ngày)")
    c.check("dải điểm cao nhất có lãi ròng dương lớn nhất", nets[-1] > 0 and nets[-1] == max(nets), nets)
    c.check("điểm hòa vốn thấp hơn khi chuyến đắt hơn", be[500_000] < be[120_000] < be[30_000])


# ---- [7] Lặp lại với ba seed ---------------------------------------------------------------------------


def breach_count(items: list, sched: dict, end) -> int:
    return sum(1 for k in range(len(items)) if (sched[k] > items[k].enqueued_at + SLA if k in sched else end > items[k].enqueued_at + SLA))


def p95_wait(items: list, sched: dict) -> float:
    return float(np.percentile([(t - items[k].enqueued_at).total_seconds() / 3600 for k, t in sched.items()], 95))


def step_seeds(c: Checker) -> None:
    say(f"\n[7] Lặp lại phép đo lý do và công suất với seed {', '.join(map(str, SEEDS))} (dữ liệu và mô hình sinh lại; không PostgreSQL)")
    say(f"{'seed':>5s}{'cờ':>6s}{'gian lận':>9s}{'precision':>10s}{'top-1 SHAP':>11s}{'top-1 gain':>11s}{'CI chênh':>20s}{'xóa 3 lý do':>12s}{'xóa 3 ngẫu nhiên':>17s}"
        f"{'tiền 50/ngày ưu tiên':>22s}{'FIFO':>7s}{'tiền 65/ngày ưu tiên':>22s}{'FIFO':>7s}")
    out, sla_rows, band_rows, narrow_rows = [], [], [], []
    end = day_start(60)
    for seed in SEEDS:
        table = load_table(seed=seed)
        day = table.select(day_of("paid_at").alias("d"))["d"].to_numpy()
        w = table.filter(pl.Series(day >= WINDOW_FROM))
        cal, meta = train(table)
        sm = in_memory(cal, meta, f"seed{seed}")
        p = sm.predict_proba(w)
        fl_mask = sm.flag(p, w["amount_vnd"].to_numpy().astype(float))
        fl = w.filter(pl.Series(fl_mask))
        ex = Explainer(sm)
        st = explain_stats(sm, ex, fl, p[fl_mask], table.filter(pl.Series(day < 30)), np.random.default_rng(seed))
        fraud = fl["is_fraud"].to_numpy().astype(bool)
        pri = p[fl_mask] * (fl["amount_vnd"].to_numpy() + HANDLING_COST_VND)
        paid = fl["paid_at"].to_list()
        items = [Item(k, paid[k], float(pri[k])) for k in range(fl.height)]
        loss = fl["amount_vnd"].to_numpy().astype(float) + HANDLING_COST_VND
        res = {}
        for per_day in (50, 65):
            for policy in ("priority", "fifo"):
                sched, money, _ = money_share(items, fraud, loss, per_day, policy)
                res[(per_day, policy)] = (money, breach_count(items, sched, end), p95_wait(items, sched))
        out.append(dict(seed=seed, st=st, m50=(res[(50, "priority")][0], res[(50, "fifo")][0]), m65=(res[(65, "priority")][0], res[(65, "fifo")][0])))
        sla_rows.append((seed, fl.height / 10, res))
        sc = p[fl_mask]
        edges = [float(sc.min()), 0.3, 0.6, 1.01]
        nets = []
        for lo_, hi_ in zip(edges, edges[1:]):
            m = (sc >= lo_) & (sc < hi_)
            nets.append(float(loss[m & fraud].sum() - REVIEW_COST_VND * m.sum()))
        band_rows.append((seed, nets))
        narrow_rows.append((seed, st["narrow1"], st["narrow3"], st["rnd_narrow3"], st["local3"]))
        lo, hi = st["ci1"]
        say(f"{seed:5d}{fl.height:6d}{int(fraud.sum()):9d}{fraud.mean():10.3f}{st['local1']:11.1%}{st['glob1']:11.1%}{f'[{lo:+.1%}, {hi:+.1%}]':>20s}"
            f"{st['del3']:12.3f}{st['rnd3']:17.3f}{res[(50, 'priority')][0]:22.1%}{res[(50, 'fifo')][0]:7.1%}{res[(65, 'priority')][0]:22.1%}{res[(65, 'fifo')][0]:7.1%}")
    say("\nSLA 24 giờ theo công suất (số đề xuất quá hạn; p95 chờ chỉ tính đề xuất đã được duyệt), ưu tiên / FIFO:")
    say(f"{'seed':>5s}{'đề xuất/ngày':>14s}{'quá hạn 50 ưu tiên':>20s}{'FIFO':>6s}{'quá hạn 65 ưu tiên':>20s}{'FIFO':>6s}{'p95 chờ 65 ưu tiên':>20s}{'FIFO':>7s}")
    for seed, per_day, res in sla_rows:
        say(f"{seed:5d}{per_day:14.1f}{res[(50, 'priority')][1]:20d}{res[(50, 'fifo')][1]:6d}{res[(65, 'priority')][1]:20d}{res[(65, 'fifo')][1]:6d}"
            f"{res[(65, 'priority')][2]:20.1f}{res[(65, 'fifo')][2]:7.1f}")
    say("\nlãi ròng theo dải điểm (triệu VND, nhãn cuối): dải thấp nhất, 0,3 đến 0,6, từ 0,6")
    for seed, nets in band_rows:
        say(f"{seed:5d}" + "".join(f"{x / 1e6:12.2f}" for x in nets))
    say("\nbản đồ HẸP (chỉ đặc trưng nêu thẳng trong hình 1 bài 54): top-1 SHAP, top-3 SHAP, top-3 ngẫu nhiên, và top-3 SHAP theo bản đồ rộng để so:")
    for seed, n1, n3, rn, l3 in narrow_rows:
        say(f"{seed:5d}{n1:10.1%}{n3:10.1%}{rn:10.1%}{l3:10.1%}")
    for r in out:
        st = r["st"]
        c.check(f"seed {r['seed']}: top-1 SHAP trúng dấu vết nhiều hơn top-1 gain cố định (khoảng tin cậy dưới > 0)", st["ci1"][0] > 0, st["ci1"])
        c.check(f"seed {r['seed']}: xóa ba lý do đầu hạ p sâu hơn xóa ba đặc trưng ngẫu nhiên ít nhất 0,05", st["del3"] < st["rnd3"] - 0.05, (st["del3"], st["rnd3"]))
        c.check(f"seed {r['seed']}: xếp theo chi phí kỳ vọng bắt không ít tiền hơn FIFO ở 50 và 65 đề xuất/ngày", r["m50"][0] >= r["m50"][1] and r["m65"][0] >= r["m65"][1], (r["m50"], r["m65"]))
        c.check(f"seed {r['seed']}: bản đồ hẹp, top-3 SHAP vẫn trúng cao hơn ngẫu nhiên ít nhất 10 điểm phần trăm", st["narrow3"] > st["rnd_narrow3"] + 0.10, (st["narrow3"], st["rnd_narrow3"]))


def main() -> int:
    verify = "--verify" in sys.argv
    c = Checker()
    try:
        conn = connect()
        conn.execute("SELECT 1")
    except psycopg.Error:
        fail_no_db()
    try:
        d = step_setup(conn, c)
        ex = step_shap(d, c)
        d = step_reasons(d, ex, c)
        q = step_queue(conn, d, c)
        step_guards(conn, q, c)
        step_measure(conn, d, q, c)
    finally:
        store = ReviewStore(conn)
        store.drop_schema()
        gone = not store.schema_exists()
        conn.close()
    say(f"\nDọn: DROP SCHEMA {SCHEMA} CASCADE; schema còn tồn tại = {not gone}")
    c.check("schema lab61 đã được xóa khi xong", gone)
    step_seeds(c)
    ok = c.total - len(c.failures)
    if verify:
        if c.failures:
            print(f"\nKiểm chứng: FAIL {ok}/{c.total}")
            return 1
        print(f"\nKiểm chứng: OK {ok}/{c.total}")
    elif c.failures:
        print(f"\nCó {len(c.failures)} ca sai ({ok}/{c.total} đạt)")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
