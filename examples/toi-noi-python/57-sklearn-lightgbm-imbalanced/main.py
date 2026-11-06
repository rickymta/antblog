"""Bài 57 - scikit-learn và LightGBM trên dữ liệu gian lận mất cân bằng.

Chạy từ gốc repository:
    uv run --project examples/toi-noi-python/57-sklearn-lightgbm-imbalanced \
        examples/toi-noi-python/57-sklearn-lightgbm-imbalanced/main.py
    ... main.py --verify    # in OK n/n khi đạt; có ca sai thì in tên ca, FAIL n/m và exit 1 (cả khi không có --verify)

Không cần Docker. Trên macOS, LightGBM cần thư viện OpenMP: `brew install libomp`.
Dữ liệu là TỔNG HỢP (synthetic_fraud.py, giống bài 54 đến 56): mọi con số chỉ nói về bộ này, và điểm rủi ro là
đầu vào cho người duyệt chứ không phải quyết định khóa tài khoản. Lần đầu lab dựng đặc trưng từ event (khoảng nửa
phút mỗi seed, lưu out/table*.parquet) rồi lưu mô hình ra out/model.joblib và out/model_meta.json.

Nhãn: MỌI bước dùng nhãn cuối cùng (`is_fraud`), trừ bước [6] so sánh với nhãn biết tại ngày 40.
"""

import json
import os
import shutil
import sys
import tempfile
import time
from pathlib import Path

import lightgbm
import numpy as np
import polars as pl
import sklearn
from sklearn.calibration import CalibratedClassifierCV
from sklearn.frozen import FrozenEstimator
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import brier_score_loss, log_loss
from sklearn.model_selection import StratifiedKFold

import metrics
from dataset import CUTOFF_DAY, at_day
from features import FEATURES, day_of
from model import (
    CAL_START_DAY,
    HANDLING_COST_VND,
    LGBM_PARAMS,
    REVIEW_COST_VND,
    expected_cost_flag,
    fit_calibrated,
    best_cost_threshold,
    make_lgbm,
    make_pipeline,
    reference_distribution,
    time_folds,
    total_cost,
)
from model_io import META_FILE, MODEL_FILE, ModelInputError, ModelLoadError, load_model, save_model
from prepare import N_TRIPS, OUT, SEED, load_table, split

TYPES = ("ghost_trip", "gps_spoof", "promo_abuse", "collusion")
TREE_GRID = [25, 50, 100, 200, 300, 400]
SEEDS = (42, 7, 123)


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


def pr(y: np.ndarray, p: np.ndarray) -> float:
    return metrics.average_precision(y, p)


def clip(p: np.ndarray) -> np.ndarray:
    return np.clip(p, 1e-9, 1 - 1e-9)


BIN_EDGES = [0.0, 0.005, 0.02, 0.1, 0.5, 1.0 + 1e-9]


def reliability(y: np.ndarray, p: np.ndarray):
    """Chia dự đoán thành 5 khoảng, so trung bình dự đoán với tỉ lệ gian lận thật trong khoảng. Trả (bảng, ECE)."""
    rows, ece = [], 0.0
    idx = np.digitize(p, BIN_EDGES) - 1
    for b in range(len(BIN_EDGES) - 1):
        m = idx == b
        if m.any():
            rows.append((BIN_EDGES[b], min(BIN_EDGES[b + 1], 1.0), int(m.sum()), float(p[m].mean()), float(y[m].mean())))
            ece += m.mean() * abs(p[m].mean() - y[m].mean())
    return rows, float(ece)


def type_pr(ft: np.ndarray, y: np.ndarray, p: np.ndarray, t: str) -> float:
    sel = (ft == t) | (y == 0)
    return pr(ft[sel] == t, p[sel])


def prepare_data(table: pl.DataFrame) -> dict:
    """Tách train/valid theo bài 54 và gom mảng cần dùng. `yk` là nhãn biết tại ngày 40 (chỉ dùng ở bước [6])."""
    train, valid = split(table)
    yk = train.select((pl.col("is_fraud") & (pl.col("label_known_at") <= pl.lit(at_day(CUTOFF_DAY)))).alias("k"))["k"].to_numpy().astype(int)
    return {
        "train": train, "valid": valid,
        "xtr": train.select(FEATURES), "ytr": train["is_fraud"].to_numpy().astype(int), "yk": yk,
        "dtr": train.select(day_of("paid_at").alias("d"))["d"].to_numpy(), "atr": train["amount_vnd"].to_numpy().astype(float),
        "xva": valid.select(FEATURES), "yva": valid["is_fraud"].to_numpy().astype(int),
        "dva": valid.select(day_of("paid_at").alias("d"))["d"].to_numpy(), "amount": valid["amount_vnd"].to_numpy().astype(float),
        "ftype": valid["fraud_type"].to_numpy(),
    }


# ---- [1] Dữ liệu -----------------------------------------------------------------------------


def step_data(c: Checker):
    say("[1] Môi trường và dữ liệu")
    say(f"Python {sys.version.split()[0]}, scikit-learn {sklearn.__version__}, lightgbm {lightgbm.__version__}, numpy {np.__version__}, polars {pl.__version__}")
    t0 = time.perf_counter()
    table = load_table()
    d = prepare_data(table)
    train, valid, ytr, yva = d["train"], d["valid"], d["ytr"], d["yva"]
    say(f"bảng chuyến {table.height:,} dòng ({time.perf_counter() - t0:.1f} giây, gồm dựng đặc trưng nếu chưa có out/table.parquet)")
    say(f"train (ngày 0 đến 30): {train.height:,} chuyến, {ytr.sum():,} gian lận ({ytr.mean():.2%})")
    say(f"valid (từ ngày 40):    {valid.height:,} chuyến, {yva.sum():,} gian lận ({yva.mean():.2%}); ngày 30 đến 40 bỏ trống")
    say("nhãn: mọi bước dưới đây dùng nhãn CUỐI CÙNG, trừ bước [6]")
    c.check("train 100.022 và valid 66.704 chuyến như bài 54", (train.height, valid.height) == (100_022, 66_704), (train.height, valid.height))
    c.check("lightgbm nạp được trên Python 3.14", sys.version_info >= (3, 14))
    return d


# ---- [2] Pipeline và baseline tuyến tính -----------------------------------------------------


def step_pipeline(d: dict, c: Checker) -> None:
    say("\n[2] Pipeline: ColumnTransformer chọn cột theo tên, rồi mô hình")
    t0 = time.perf_counter()
    lr = make_pipeline(LogisticRegression(max_iter=2000), scale=True).fit(d["xtr"], d["ytr"])
    p_lr = lr.predict_proba(d["xva"])[:, 1]
    say(f"hồi quy logistic (log1p + StandardScaler, 30 đặc trưng, {time.perf_counter() - t0:.1f} giây): "
        f"PR-AUC {pr(d['yva'], p_lr):.3f}, ROC-AUC {metrics.roc_auc(d['yva'], p_lr):.3f} trên valid")
    valid = d["valid"]
    extra = lr.predict_proba(valid)[:, 1]  # nguyên bảng 35 cột: trip_id, nhãn... bị bỏ
    shuffled = lr.predict_proba(valid.select(list(reversed(FEATURES))))[:, 1]
    same_extra, same_shuffled = bool(np.array_equal(extra, p_lr)), bool(np.array_equal(shuffled, p_lr))
    say(f"đưa nguyên bảng {valid.width} cột (có trip_id, nhãn): kết quả giống hệt = {same_extra}; đảo thứ tự cột đặc trưng: giống hệt = {same_shuffled}")
    try:
        lr.predict_proba(d["xva"].drop("speed_kmh"))
        err = "không lỗi"
    except ValueError as e:
        err = type(e).__name__ + ": " + str(e).split(".")[0][:90]
    say(f"bỏ cột speed_kmh: {err}")
    c.check("pipeline bỏ cột thừa và không phụ thuộc thứ tự cột", same_extra and same_shuffled)
    c.check("thiếu một đặc trưng thì lỗi, không đoán", err != "không lỗi", err)


# ---- [3] Cross-validation --------------------------------------------------------------------


def step_cv(d: dict, c: Checker) -> int:
    say("\n[3] Cross-validation trên train: theo thời gian và ngẫu nhiên (nhãn cuối cùng)")
    xtr, ytr, dtr, train = d["xtr"], d["ytr"], d["dtr"], d["train"]
    t0 = time.perf_counter()
    say("theo thời gian: cửa sổ test 5 ngày, bỏ 2 ngày sát test, train mọi ngày trước đó (LightGBM 400 vòng, PR-AUC theo số vòng):")
    say(f"{'test (ngày)':>12s}{'train':>9s}{'test':>8s}{'gian lận':>10s}" + "".join(f"{k:>7d}" for k in TREE_GRID))
    grid, spans = [], []
    for tr_i, te_i in time_folds(dtr):
        pipe = make_pipeline(make_lgbm(n_estimators=max(TREE_GRID))).fit(xtr[tr_i], ytr[tr_i])
        z = pipe[:-1].transform(xtr[te_i])
        scores = [pr(ytr[te_i], pipe[-1].predict_proba(z, num_iteration=k)[:, 1]) for k in TREE_GRID]
        grid.append(scores)
        spans.append((tr_i, te_i))
        say(f"{f'{dtr[te_i].min():.0f} đến {dtr[te_i].max() + 1e-9:.0f}':>12s}{len(tr_i):9,d}{len(te_i):8,d}{ytr[te_i].mean():10.2%}" + "".join(f"{s:7.3f}" for s in scores))
    grid_a = np.array(grid)
    mean = grid_a.mean(axis=0)
    best = TREE_GRID[int(np.argmax(mean))]
    say(f"{'trung bình':>12s}{'':27s}" + "".join(f"{s:7.3f}" for s in mean))
    spread = float(mean[TREE_GRID.index(100):].max() - mean[TREE_GRID.index(100):].min())
    say(f"số vòng chọn theo trung bình PR-AUC các fold: {best}; từ 100 đến 400 vòng trung bình chỉ chênh {spread:.3f}, nên không đáng tinh chỉnh thêm")
    ki = TREE_GRID.index(best)
    fold_scores = grid_a[:, ki]

    # đổi MỘT biến: giữ train cố định ở fold đầu (26.693 chuyến), chấm cả bốn cửa sổ
    small = spans[0][0]
    fixed = make_pipeline(make_lgbm(n_estimators=best)).fit(xtr[small], ytr[small])
    fixed_scores = [pr(ytr[te], fixed.predict_proba(xtr[te])[:, 1]) for _, te in spans]
    say(f"\ncùng {best} vòng, train CỐ ĐỊNH ở fold đầu ({len(small):,} chuyến) so với train mở rộng, PR-AUC theo cửa sổ test:")
    say(f"{'':26s}" + "".join(f"{f'ngày {dtr[te].min():.0f}':>10s}" for _, te in spans))
    say(f"{'train cố định':26s}" + "".join(f"{s:10.3f}" for s in fixed_scores))
    say(f"{'train mở rộng (như trên)':26s}" + "".join(f"{s:10.3f}" for s in fold_scores))

    rs = []
    for a, b in StratifiedKFold(5, shuffle=True, random_state=0).split(xtr, ytr):
        m = make_pipeline(make_lgbm(n_estimators=best)).fit(xtr[a], ytr[a])
        rs.append(pr(ytr[b], m.predict_proba(xtr[b])[:, 1]))
    say(f"\nngẫu nhiên 5 fold (StratifiedKFold, {best} vòng, train {len(ytr) * 4 // 5:,} chuyến mỗi fold): " + " ".join(f"{s:.3f}" for s in rs) + f"; trung bình {np.mean(rs):.3f}")
    say(f"theo thời gian 4 fold ({best} vòng, train {min(len(a) for a, _ in spans):,} đến {max(len(a) for a, _ in spans):,} chuyến): " + " ".join(f"{s:.3f}" for s in fold_scores) + f"; trung bình {fold_scores.mean():.3f}")
    say("hai trung bình khác nhau ở HAI biến (cỡ train và cách chia), nên không so thẳng để kết luận về cách chia")
    say(f"({time.perf_counter() - t0:.0f} giây; load average 1/5/15 phút: {load()})")

    say("\nnhãn gian lận đã biết vào lúc bắt đầu cửa sổ test (trong số chuyến gian lận của phần train của fold):")
    fr = []
    for tr_i, te_i in spans:
        start = float(np.floor(dtr[te_i].min()))
        known = train[tr_i].select((pl.col("is_fraud") & (pl.col("label_known_at") <= pl.lit(at_day(start)))).sum().alias("k"))["k"][0]
        fr.append(known / ytr[tr_i].sum())
        say(f"  test từ ngày {start:.0f}: biết {known:,} trong {int(ytr[tr_i].sum()):,} chuyến gian lận ({known / ytr[tr_i].sum():.0%})")
    c.check("mọi fold theo thời gian: train kết thúc trước test ít nhất 2 ngày", all(dtr[a].max() + 2 <= dtr[b].min() + 1e-9 for a, b in spans))
    c.check("số vòng chọn nằm trong lưới và PR-AUC CV cao hơn 20 lần mốc cơ sở", best in TREE_GRID and fold_scores.mean() > 20 * ytr.mean(), (best, fold_scores.mean()))
    c.check("PR-AUC giữa các fold theo thời gian dao động ít nhất 0,02 (một lần chia là ít)", float(np.ptp(fold_scores)) >= 0.02, np.ptp(fold_scores))
    c.check("train cố định tụt theo thời gian: cửa sổ cuối thấp hơn cửa sổ đầu ít nhất 0,03", fixed_scores[0] - fixed_scores[-1] >= 0.03, fixed_scores)
    c.check("train mở rộng hơn train cố định ở cửa sổ cuối", fold_scores[-1] > fixed_scores[-1], (fold_scores[-1], fixed_scores[-1]))
    c.check("lúc bắt đầu test, chưa biết hết nhãn gian lận của phần train (dưới 90 % ở mọi fold)", all(f < 0.9 for f in fr), fr)
    c.check("lab chọn n_estimators của LGBM_PARAMS bằng đúng kết quả CV", best == LGBM_PARAMS["n_estimators"], (best, LGBM_PARAMS["n_estimators"]))
    return best


# ---- [4] Mất cân bằng: trọng số lớp ----------------------------------------------------------


def step_imbalance(d: dict, n_trees: int, c: Checker) -> float:
    say(f"\n[4] Trọng số lớp trên cùng train ({n_trees} vòng, đổi MỖI scale_pos_weight, mọi thứ khác giữ nguyên, nhãn cuối)")
    xtr, ytr, xva, yva = d["xtr"], d["ytr"], d["xva"], d["yva"]
    neg_pos = float((ytr == 0).sum() / (ytr == 1).sum())
    say(f"tỉ lệ sạch/gian lận ở train = {neg_pos:.1f}; tỉ lệ gian lận ở valid = {yva.mean():.4f}")
    say(f"{'scale_pos_weight':>17s}{'PR-AUC':>8s}{'ROC-AUC':>9s}{'logloss':>9s}{'Brier':>8s}{'p trung bình':>14s}{'cờ ở 0.5':>10s}{'precision':>10s}{'recall':>8s}")
    res, probs = {}, {}
    for spw in (1.0, 7.0, neg_pos):
        p = make_pipeline(make_lgbm(n_estimators=n_trees, scale_pos_weight=spw)).fit(xtr, ytr).predict_proba(xva)[:, 1]
        cf = metrics.confusion(yva, p, 0.5)
        res[spw] = dict(pr=pr(yva, p), roc=metrics.roc_auc(yva, p), ll=log_loss(yva, clip(p)), br=brier_score_loss(yva, p), mean=float(p.mean()), flag=cf.tp + cf.fp, prec=cf.precision, rec=cf.recall)
        probs[spw] = p
        r = res[spw]
        say(f"{spw:17.1f}{r['pr']:8.3f}{r['roc']:9.3f}{r['ll']:9.4f}{r['br']:8.4f}{r['mean']:14.4f}{r['flag']:10,d}{r['prec']:10.3f}{r['rec']:8.3f}")
    say("cùng số chuyến bị gắn cờ, nhưng lấy từ mô hình KHÔNG trọng số (chỉ hạ ngưỡng):")
    close = True
    for spw in (7.0, neg_pos):
        k = res[spw]["flag"]
        top = np.argsort(-probs[1.0], kind="stable")[:k]
        prec, rec = float(yva[top].mean()), float(yva[top].sum() / yva.sum())
        close = close and abs(rec - res[spw]["rec"]) < 0.05 and abs(prec - res[spw]["prec"]) < 0.05
        say(f"  scale_pos_weight={spw:5.1f} gắn cờ {k:,} chuyến (ngưỡng 0.5): precision {res[spw]['prec']:.3f}, recall {res[spw]['rec']:.3f}; "
            f"không trọng số, {k:,} chuyến điểm cao nhất: precision {prec:.3f}, recall {rec:.3f}")
    prs = [r["pr"] for r in res.values()]
    means = [r["mean"] for r in res.values()]
    c.check("trọng số lớp gần như không đổi PR-AUC (xếp hạng), chênh dưới 0,02", max(prs) - min(prs) < 0.02, prs)
    c.check("trọng số lớp đẩy xác suất trung bình lên, tăng dần theo scale_pos_weight", means[0] < means[1] < means[2], means)
    c.check("ở scale_pos_weight = sạch/gian lận, xác suất trung bình vượt tỉ lệ thật ít nhất 1,3 lần", means[2] > 1.3 * yva.mean(), (means[2], yva.mean()))
    c.check("logloss xấu đi khi trọng số lớp đủ lớn so với không trọng số", res[neg_pos]["ll"] > res[1.0]["ll"], (res[neg_pos]["ll"], res[1.0]["ll"]))
    c.check("hạ ngưỡng trên mô hình không trọng số cho precision và recall trong 0,05 so với mô hình có trọng số", close)
    return neg_pos


# ---- [5] Calibration -------------------------------------------------------------------------


def step_calibration(d: dict, n_trees: int, neg_pos: float, c: Checker):
    say(f"\n[5] Calibration: mô hình gốc huấn luyện ngày < {CAL_START_DAY}, bộ hiệu chỉnh là ngày {CAL_START_DAY} đến 30 (nhãn cuối, không chạm valid)")
    xtr, ytr, dtr, xva, yva = d["xtr"], d["ytr"], d["dtr"], d["xva"], d["yva"]
    a, b = np.flatnonzero(dtr < CAL_START_DAY), np.flatnonzero(dtr >= CAL_START_DAY)
    say(f"gốc: {len(a):,} chuyến ({ytr[a].sum():,} gian lận, {ytr[a].mean():.2%}); hiệu chỉnh: {len(b):,} chuyến ({ytr[b].sum():,} gian lận, {ytr[b].mean():.2%}); valid {yva.mean():.2%}")
    say("hai phần liền nhau, không có khoảng bỏ: chuyến cuối của phần gốc có nhãn chưa chín, và hiệu chỉnh thừa hưởng điều đó")
    say("chọn sigmoid TRƯỚC khi đọc valid: bộ hiệu chỉnh chỉ có vài trăm chuyến gian lận, hàm hai tham số đơn điệu ít rủi ro hơn bậc thang isotonic")
    say(f"{'mô hình':34s}{'PR-AUC':>8s}{'logloss':>9s}{'Brier':>8s}{'p trung bình':>14s}{'ECE':>8s}{'số mức điểm':>13s}")
    out, base_pipes, cal_models = {}, {}, {}
    for spw in (1.0, neg_pos):
        base, cal_sig = fit_calibrated(xtr, ytr, dtr, n_estimators=n_trees, scale_pos_weight=spw)
        cal_iso = CalibratedClassifierCV(FrozenEstimator(base), method="isotonic").fit(xtr[b], ytr[b])
        base_pipes[spw], cal_models[spw] = base, cal_sig
        variants = {"chưa hiệu chỉnh": base.predict_proba(xva)[:, 1], "sigmoid": cal_sig.predict_proba(xva)[:, 1], "isotonic": cal_iso.predict_proba(xva)[:, 1]}
        for name, p in variants.items():
            _, ece = reliability(yva, p)
            out[(spw, name)] = p
            label = f"scale_pos_weight={spw:.1f}, {name}"
            say(f"{label:34s}{pr(yva, p):8.3f}{log_loss(yva, clip(p)):9.4f}{brier_score_loss(yva, p):8.4f}{p.mean():14.4f}{ece:8.4f}{len(np.unique(p)):13,d}")
    raw1, sig1, iso1 = out[(1.0, "chưa hiệu chỉnh")], out[(1.0, "sigmoid")], out[(1.0, "isotonic")]
    rawn, sign = out[(neg_pos, "chưa hiệu chỉnh")], out[(neg_pos, "sigmoid")]
    rows_raw, _ = reliability(yva, raw1)
    rows_sig, _ = reliability(yva, sig1)
    rows_w, _ = reliability(yva, rawn)
    say("\nđộ tin cậy (reliability) trên valid, scale_pos_weight=1: p dự đoán trung bình so với tỉ lệ gian lận thật trong từng khoảng")
    say(f"{'khoảng p':>16s}{'chưa hiệu chỉnh:':>18s}{'p':>7s}{'thật':>8s}{'sigmoid:':>14s}{'p':>7s}{'thật':>8s}   (chuyến, p trung bình, tỉ lệ gian lận thật)")
    sig_by = {r[0]: r for r in rows_sig}
    for rr in rows_raw:
        rs = sig_by.get(rr[0])
        tail = f"{rs[2]:14,d}{rs[3]:7.4f}{rs[4]:8.4f}" if rs else ""
        say(f"{f'[{rr[0]:.3f}, {rr[1]:.3f})':>16s}{rr[2]:18,d}{rr[3]:7.4f}{rr[4]:8.4f}{tail}")
    say("scale_pos_weight=" + f"{neg_pos:.1f}, chưa hiệu chỉnh, cùng các khoảng: " + "; ".join(f"[{r[0]:.3f}, {r[1]:.3f}) p {r[3]:.3f} thật {r[4]:.3f} ({r[2]:,} chuyến)" for r in rows_w))

    c.check("sigmoid giảm logloss và Brier so với chưa hiệu chỉnh (scale_pos_weight=1)", log_loss(yva, clip(sig1)) < log_loss(yva, clip(raw1)) and brier_score_loss(yva, sig1) < brier_score_loss(yva, raw1))
    c.check("sau sigmoid, p trung bình nằm trong 10 % tỉ lệ gian lận thật của valid", abs(sig1.mean() - yva.mean()) < 0.1 * yva.mean(), (sig1.mean(), yva.mean()))
    c.check("sigmoid là hàm đơn điệu của điểm: PR-AUC đổi dưới 0,005", abs(pr(yva, sig1) - pr(yva, raw1)) < 0.005, (pr(yva, sig1), pr(yva, raw1)))
    c.check("isotonic gộp điểm thành bậc thang: ít mức điểm hơn hẳn", len(np.unique(iso1)) < 0.2 * len(np.unique(raw1)), (len(np.unique(iso1)), len(np.unique(raw1))))
    c.check("sigmoid kéo p trung bình của mô hình có trọng số về gần tỉ lệ thật", abs(sign.mean() - yva.mean()) < abs(rawn.mean() - yva.mean()), (sign.mean(), rawn.mean()))
    return base_pipes[1.0], cal_models[1.0], {"raw": raw1, "sigmoid": sig1, "isotonic": iso1}


# ---- [6] Nhãn cuối cùng so với nhãn biết ở ngày 40 -------------------------------------------


def step_labels(d: dict, c: Checker) -> None:
    say(f"\n[6] Nhãn cuối cùng so với nhãn biết tại ngày {CUTOFF_DAY} (huấn luyện VÀ hiệu chỉnh, đo trên valid bằng nhãn cuối)")
    xtr, ytr, yk, dtr, xva, yva, amount, ft = d["xtr"], d["ytr"], d["yk"], d["dtr"], d["xva"], d["yva"], d["amount"], d["ftype"]
    a, b = np.flatnonzero(dtr < CAL_START_DAY), np.flatnonzero(dtr >= CAL_START_DAY)
    say(f"gian lận biết được vào ngày {CUTOFF_DAY}: phần gốc {yk[a].sum():,}/{ytr[a].sum():,} ({yk[a].sum() / ytr[a].sum():.0%}), phần hiệu chỉnh {yk[b].sum():,}/{ytr[b].sum():,} ({yk[b].sum() / ytr[b].sum():.0%})")
    say(f"{'nhãn để huấn luyện + hiệu chỉnh':34s}{'PR-AUC':>8s}{'p trung bình':>14s}{'ECE':>8s}{'gắn cờ':>8s}{'chi phí kỳ vọng (VND)':>23s}{'collusion PR-AUC':>18s}")
    res = {}
    for name, y in (("nhãn cuối cùng", ytr), (f"nhãn biết ở ngày {CUTOFF_DAY}", yk)):
        _, cal = fit_calibrated(xtr, y, dtr)
        p = cal.predict_proba(xva)[:, 1]
        fl = expected_cost_flag(p, amount)
        _, ece = reliability(yva, p)
        res[name] = dict(pr=pr(yva, p), mean=float(p.mean()), ece=ece, cost=total_cost(fl, yva, amount), col=type_pr(ft, yva, p, "collusion"))
        r = res[name]
        say(f"{name:34s}{r['pr']:8.3f}{r['mean']:14.4f}{r['ece']:8.4f}{int(fl.sum()):8,d}{r['cost']:23,.0f}{r['col']:18.3f}")
    final, known = res["nhãn cuối cùng"], res[f"nhãn biết ở ngày {CUTOFF_DAY}"]
    c.check("nhãn biết ở ngày 40 làm p trung bình thấp hơn (gian lận chưa có nhãn bị dạy là sạch)", known["mean"] < final["mean"], (known["mean"], final["mean"]))
    c.check("nhãn biết ở ngày 40 làm chi phí kỳ vọng cao hơn nhãn cuối", known["cost"] > final["cost"], (known["cost"], final["cost"]))
    c.check("nhãn biết ở ngày 40 làm collusion (độ trễ dài nhất) tụt PR-AUC", known["col"] < final["col"], (known["col"], final["col"]))


# ---- [7] Nhiều seed ---------------------------------------------------------------------------


def step_seeds(n_trees: int, c: Checker) -> dict:
    say(f"\n[7] Lặp lại trên {len(SEEDS)} seed của bộ sinh ({', '.join(map(str, SEEDS))}); mô hình, siêu tham số, cách chia giữ nguyên (nhãn cuối)")
    say(f"{'seed':>5s}{'gian lận train/hiệu chỉnh/valid':>34s}{'PR toàn train':>15s}{'PR gốc<22':>11s}{'ECE chưa':>10s}{'ECE sigmoid':>13s}{'chi phí sigmoid':>17s}{'chi phí chưa':>14s}{'chênh':>8s}")
    rows, types = {}, {}
    for seed in SEEDS:
        dd = prepare_data(load_table(seed=seed))
        xtr, ytr, dtr, xva, yva, amount, ft = dd["xtr"], dd["ytr"], dd["dtr"], dd["xva"], dd["yva"], dd["amount"], dd["ftype"]
        a, b = np.flatnonzero(dtr < CAL_START_DAY), np.flatnonzero(dtr >= CAL_START_DAY)
        full = make_pipeline(make_lgbm(n_estimators=n_trees)).fit(xtr, ytr).predict_proba(xva)[:, 1]
        base, cal = fit_calibrated(xtr, ytr, dtr, n_estimators=n_trees)
        raw, sig = base.predict_proba(xva)[:, 1], cal.predict_proba(xva)[:, 1]
        lr = make_pipeline(LogisticRegression(max_iter=2000), scale=True).fit(xtr[a], ytr[a]).predict_proba(xva)[:, 1]
        c_sig, c_raw = total_cost(expected_cost_flag(sig, amount), yva, amount), total_cost(expected_cost_flag(raw, amount), yva, amount)
        _, e_raw = reliability(yva, raw)
        _, e_sig = reliability(yva, sig)
        rows[seed] = dict(full=pr(yva, full), base=pr(yva, raw), sig=pr(yva, sig), e_raw=e_raw, e_sig=e_sig, c_sig=c_sig, c_raw=c_raw, ratio=c_raw / c_sig - 1)
        types[seed] = {t: (type_pr(ft, yva, sig, t), type_pr(ft, yva, lr, t)) for t in TYPES}
        r = rows[seed]
        say(f"{seed:5d}{f'{ytr.mean():.2%} / {ytr[b].mean():.2%} / {yva.mean():.2%}':>34s}{r['full']:15.3f}{r['base']:11.3f}{r['e_raw']:10.4f}{r['e_sig']:13.4f}{r['c_sig']:17,.0f}{r['c_raw']:14,.0f}{r['ratio']:8.1%}")
    say("chi phí: quy tắc chi phí kỳ vọng trên TOÀN valid (không tinh chỉnh ngưỡng); 'chênh' = chi phí p chưa hiệu chỉnh / chi phí p đã hiệu chỉnh - 1")
    say("\nPR-AUC theo loại (LightGBM+sigmoid / hồi quy logistic, cùng train ngày < 22):")
    say(f"{'loại':12s}" + "".join(f"{f'seed {s}':>16s}" for s in SEEDS))
    for t in TYPES:
        say(f"{t:12s}" + "".join(f"{f'{types[s][t][0]:.3f} / {types[s][t][1]:.3f}':>16s}" for s in SEEDS))
    ratios = [rows[s]["ratio"] for s in SEEDS]
    c.check("sigmoid giảm ECE ở mọi seed", all(rows[s]["e_sig"] < rows[s]["e_raw"] for s in SEEDS), rows)
    c.check("sigmoid đổi PR-AUC dưới 0,005 ở mọi seed", all(abs(rows[s]["sig"] - rows[s]["base"]) < 0.005 for s in SEEDS), rows)
    c.check("chênh chi phí của p chưa hiệu chỉnh so với đã hiệu chỉnh dao động trên 3 điểm phần trăm giữa các seed (không kết luận từ một seed)", max(ratios) - min(ratios) > 0.03, ratios)
    c.check("LightGBM hơn hồi quy logistic rõ ở gps_spoof và collusion ở mọi seed", all(types[s][t][0] > types[s][t][1] + 0.05 for s in SEEDS for t in ("gps_spoof", "collusion")), types)
    return {"rows": rows, "types": types}


# ---- [8] Ngưỡng ------------------------------------------------------------------------------


def step_threshold(d: dict, base, cal, p_raw: np.ndarray, p_cal: np.ndarray, c: Checker):
    say("\n[8] Ngưỡng: chọn trên BỘ HIỆU CHỈNH (ngày 22 đến 30), báo cáo trên toàn valid (nhãn cuối)")
    xtr, ytr, dtr, atr = d["xtr"], d["ytr"], d["dtr"], d["atr"]
    yva, amount = d["yva"], d["amount"]
    a, b = np.flatnonzero(dtr < CAL_START_DAY), np.flatnonzero(dtr >= CAL_START_DAY)
    say(f"valid: {len(yva):,} chuyến, {yva.sum():,} gian lận; chi phí = {REVIEW_COST_VND:,} VND x số chuyến gắn cờ (kể cả gian lận thật) + (tiền chuyến + {HANDLING_COST_VND:,} VND) x gian lận bỏ sót")
    lr = make_pipeline(LogisticRegression(max_iter=2000), scale=True).fit(xtr[a], ytr[a])
    p_lr = lr.predict_proba(d["xva"])[:, 1]
    thr, _ = best_cost_threshold(ytr[b], cal.predict_proba(xtr[b])[:, 1], atr[b])
    thr_lr, _ = best_cost_threshold(ytr[b], lr.predict_proba(xtr[b])[:, 1], atr[b])
    k = int(0.01 * len(yva))
    top = np.zeros(len(yva), dtype=bool)
    top[np.argsort(-p_cal, kind="stable")[:k]] = True
    policies = [
        ("không gắn cờ gì", np.zeros(len(yva), dtype=bool)),
        ("ngưỡng 0.5", p_cal >= 0.5),
        (f"ngưỡng chọn trên bộ hiệu chỉnh ({thr:.3f})", p_cal >= thr),
        ("chi phí kỳ vọng, p đã hiệu chỉnh", expected_cost_flag(p_cal, amount)),
        ("chi phí kỳ vọng, p CHƯA hiệu chỉnh", expected_cost_flag(p_raw, amount)),
        (f"người duyệt chỉ xem {k} chuyến (1 %)", top),
        (f"hồi quy logistic, ngưỡng trên bộ hiệu chỉnh ({thr_lr:.3f})", p_lr >= thr_lr),
    ]
    say(f"{'chính sách':52s}{'gắn cờ':>8s}{'TP':>6s}{'FP':>6s}{'FN':>6s}{'chi phí (VND)':>16s}")
    by = {}
    yb = yva.astype(bool)
    for name, fl in policies:
        by[name] = (int(fl.sum()), int((fl & yb).sum()), int((fl & ~yb).sum()), int((~fl & yb).sum()), total_cost(fl, yva, amount))
        say(f"{name:52s}{by[name][0]:8,d}{by[name][1]:6d}{by[name][2]:6d}{by[name][3]:6d}{by[name][4]:16,.0f}")
    none_cost = by["không gắn cờ gì"][4]
    tuned = by[f"ngưỡng chọn trên bộ hiệu chỉnh ({thr:.3f})"][4]
    c.check("ngưỡng chọn trên bộ hiệu chỉnh rẻ hơn không làm gì và rẻ hơn ngưỡng 0.5", tuned < min(none_cost, by["ngưỡng 0.5"][4]), by)
    c.check("quy tắc chi phí kỳ vọng (không tuning) rẻ hơn không làm gì", by["chi phí kỳ vọng, p đã hiệu chỉnh"][4] < none_cost)
    c.check("ngưỡng chọn theo chi phí thấp hơn 0,5", thr < 0.5, thr)
    c.check("LightGBM đã hiệu chỉnh rẻ hơn hồi quy logistic ở ngưỡng chọn trên bộ hiệu chỉnh", tuned < by[f"hồi quy logistic, ngưỡng trên bộ hiệu chỉnh ({thr_lr:.3f})"][4])
    return thr, p_lr


# ---- [9] Theo loại và đặc trưng quan trọng ---------------------------------------------------


def step_types(d: dict, base, p_cal: np.ndarray, p_lr: np.ndarray, c: Checker) -> dict[str, float]:
    say("\n[9] PR-AUC theo loại gian lận (valid) và đặc trưng quan trọng")
    yva, ft = d["yva"], d["ftype"]
    say(f"{'loại':12s}{'chuyến':>8s}{'LightGBM+sigmoid':>18s}{'hồi quy logistic':>18s}")
    by_type = {}
    for t in TYPES:
        by_type[t] = type_pr(ft, yva, p_cal, t)
        say(f"{t:12s}{int((ft == t).sum()):8d}{by_type[t]:18.3f}{type_pr(ft, yva, p_lr, t):18.3f}")
    gain = base[-1].booster_.feature_importance("gain")
    top = np.argsort(-gain)[:8]
    say("8 đặc trưng có tổng gain lớn nhất: " + ", ".join(f"{FEATURES[i]} {gain[i] / gain.sum():.1%}" for i in top))
    c.check("tổng gain của 8 đặc trưng đầu dưới 80 %, tức nhiều đặc trưng cùng đóng góp", gain[top].sum() / gain.sum() < 0.8, gain[top].sum() / gain.sum())
    c.check("đặc trưng đa chuyến (cửa sổ thời gian) có mặt trong 8 đặc trưng đầu", any(FEATURES[i].startswith(("dev_", "pair_", "cust_", "drv_")) for i in top))
    return by_type


# ---- [10] Lưu và nạp ---------------------------------------------------------------------------


def step_save(d: dict, cal, p_cal: np.ndarray, thr: float, n_trees: int, by_type: dict[str, float], c: Checker) -> None:
    say("\n[10] Lưu mô hình: joblib + metadata")
    yva, xva = d["yva"], d["xva"]
    meta = {
        "created_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "data": {"synthetic": True, "seed": SEED, "n_trips": N_TRIPS, "base_train_days": [0, CAL_START_DAY],
                 "calibration_days": [CAL_START_DAY, 30], "gap_days": [30, 40], "valid_from_day": 40, "labels": "final"},
        "model": {"kind": "LGBMClassifier + CalibratedClassifierCV(sigmoid)", "params": {**LGBM_PARAMS, "n_estimators": n_trees, "scale_pos_weight": 1.0}},
        "decision": {"tuned_threshold": round(float(thr), 6), "threshold_chosen_on": "calibration days 22-30", "review_cost_vnd": REVIEW_COST_VND,
                     "handling_cost_vnd": HANDLING_COST_VND, "note": "diem rui ro la dau vao cho nguoi duyet, khong tu khoa tai khoan"},
        "metrics_valid": {"pr_auc": round(pr(yva, p_cal), 4), "roc_auc": round(metrics.roc_auc(yva, p_cal), 4),
                          "brier": round(float(brier_score_loss(yva, p_cal)), 5), "logloss": round(float(log_loss(yva, clip(p_cal))), 5),
                          "base_rate": round(float(yva.mean()), 5), "rows": int(len(yva))},
        "metrics_valid_by_type": {t: round(v, 4) for t, v in by_type.items()},
        "reference_distribution": reference_distribution(d["xtr"]),
    }
    full = save_model(cal, meta, OUT)
    size = (OUT / MODEL_FILE).stat().st_size
    say(f"đã ghi out/{MODEL_FILE} ({size / 1024:.0f} KiB) và out/{META_FILE}; model_version = {full['model_version']}")
    say("khóa của metadata: " + ", ".join(sorted(full)))
    say("metadata (trích): " + json.dumps({k: full[k] for k in ("model_version", "metrics_valid", "metrics_valid_by_type")}, ensure_ascii=False))
    loaded = load_model(OUT)
    again = loaded.predict_proba(xva)
    one = loaded.score_one(xva.row(0, named=True))
    say(f"nạp lại: {len(again):,} dự đoán giống hệt lúc huấn luyện = {bool(np.array_equal(again, p_cal))}; score_one(chuyến đầu valid) = {one:.6f} (hàng loạt: {again[0]:.6f}); ngưỡng trong meta {loaded.threshold:.3f}")
    c.check("nạp lại cho dự đoán giống hệt", bool(np.array_equal(again, p_cal)))
    c.check("score_one khớp predict_proba", abs(one - again[0]) < 1e-12)
    c.check("meta có danh sách 30 đặc trưng đúng thứ tự FEATURES", full["feature_list"] == FEATURES and len(FEATURES) == 30)
    c.check("meta có phân phối tham chiếu cho cả 30 đặc trưng và metric theo bốn loại", len(full["reference_distribution"]) == 30 and len(full["metrics_valid_by_type"]) == 4)

    contrib = loaded.contributions(xva.head(200))
    margin = loaded.raw_margin(xva.head(200))
    gap = float(np.abs(contrib.sum(axis=1) - margin).max())
    say(f"đóng góp từng đặc trưng (pred_contrib): tổng mỗi dòng khớp điểm thô trước hiệu chỉnh, lệch lớn nhất {gap:.1e} trên 200 chuyến (đây là log-odds của LightGBM, không phải p)")
    c.check("tổng đóng góp bằng điểm thô (margin) của booster", gap < 1e-9, gap)

    bad_inputs = {"None": None, "NaN": float("nan"), "chuỗi": "abc"}
    row = xva.row(0, named=True)
    refused = {}
    for name, v in bad_inputs.items():
        try:
            loaded.score_one({**row, "drv_jump_km": v})
            refused[name] = "không lỗi"
        except ModelInputError as e:
            refused[name] = str(e)
    try:
        loaded.score_one({k: v for k, v in row.items() if k != "speed_kmh"})
        refused["thiếu khóa"] = "không lỗi"
    except ModelInputError as e:
        refused["thiếu khóa"] = str(e)
    say("score_one từ chối đầu vào xấu (ModelInputError):")
    for name, msg in refused.items():
        say(f"  {name}: {msg}")
    c.check("score_one từ chối None, NaN, chuỗi và thiếu khóa", all(v != "không lỗi" for v in refused.values()), refused)

    tmp = Path(tempfile.mkdtemp(prefix="lab57-"))
    try:
        errors = {}
        for name in ("tamper", "features", "libs", "json", "keys", "trunc"):
            shutil.copytree(OUT, tmp / name, ignore=shutil.ignore_patterns("table*.parquet"))
        blob = (tmp / "tamper" / MODEL_FILE).read_bytes()
        (tmp / "tamper" / MODEL_FILE).write_bytes(blob[:-1] + bytes([blob[-1] ^ 1]))
        mj = json.loads((tmp / "features" / META_FILE).read_text())
        mj["feature_list"] = mj["feature_list"][:-1]
        (tmp / "features" / META_FILE).write_text(json.dumps(mj))
        mj = json.loads((tmp / "libs" / META_FILE).read_text())
        mj["libraries"]["lightgbm"] = "3.3.5"
        (tmp / "libs" / META_FILE).write_text(json.dumps(mj))
        (tmp / "json" / META_FILE).write_text("{bad")
        mj = json.loads((tmp / "keys" / META_FILE).read_text())
        del mj["artifact"]
        (tmp / "keys" / META_FILE).write_text(json.dumps(mj))
        (tmp / "trunc" / MODEL_FILE).write_bytes((tmp / "trunc" / MODEL_FILE).read_bytes()[:1000])
        mj = json.loads((tmp / "trunc" / META_FILE).read_text())
        from model_io import sha256_of
        mj["artifact"]["sha256"] = sha256_of(tmp / "trunc" / MODEL_FILE)
        (tmp / "trunc" / META_FILE).write_text(json.dumps(mj))
        for name in errors_names(tmp):
            try:
                load_model(tmp / name)
                errors[name] = "không lỗi"
            except ModelLoadError as e:
                errors[name] = str(e)
        say("nạp file bị sửa (mọi lỗi đều là ModelLoadError):")
        for name, label in (("tamper", "đổi 1 bit trong model.joblib"), ("features", "bỏ 1 đặc trưng khỏi meta"), ("libs", "meta ghi lightgbm 3.3.5"),
                            ("json", "meta là JSON hỏng"), ("keys", "meta thiếu khóa artifact"), ("trunc", "model.joblib cắt cụt, sha256 sửa cho khớp")):
            say(f"  {label}: {errors[name]}")
        relaxed = load_model(tmp / "libs", strict=False)
        c.check("sha256 phát hiện file bị đổi", "sha256" in errors["tamper"], errors["tamper"])
        c.check("danh sách đặc trưng khác thì từ chối", "đặc trưng" in errors["features"], errors["features"])
        c.check("phiên bản lightgbm khác minor thì từ chối, trừ khi strict=False", "lightgbm" in errors["libs"] and relaxed.version == full["model_version"], errors["libs"])
        c.check("meta JSON hỏng, thiếu khóa, joblib cắt cụt đều ra ModelLoadError", all(errors[n] != "không lỗi" for n in ("json", "keys", "trunc")), errors)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    leftovers = sorted(p.name for p in OUT.iterdir() if p.name.startswith(".") or p.suffix == ".tmp")
    c.check("save_model không để lại file tạm", not leftovers, leftovers)


def errors_names(tmp: Path) -> list[str]:
    return sorted(p.name for p in tmp.iterdir())


def main() -> int:
    verify = "--verify" in sys.argv
    c = Checker()
    d = step_data(c)
    step_pipeline(d, c)
    n_trees = step_cv(d, c)
    neg_pos = step_imbalance(d, n_trees, c)
    base, cal, ps = step_calibration(d, n_trees, neg_pos, c)
    step_labels(d, c)
    step_seeds(n_trees, c)
    thr, p_lr = step_threshold(d, base, cal, ps["raw"], ps["sigmoid"], c)
    by_type = step_types(d, base, ps["sigmoid"], p_lr, c)
    step_save(d, cal, ps["sigmoid"], thr, n_trees, by_type, c)
    ok = c.total - len(c.failures)
    if c.failures:
        print(f"\nKiểm chứng: FAIL {ok}/{c.total}")
        return 1
    if verify:
        print(f"\nKiểm chứng: OK {ok}/{c.total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
