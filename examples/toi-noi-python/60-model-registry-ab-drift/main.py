"""Bài 60 - Model registry, shadow mode, A/B theo hash, drift (PSI), lịch huấn luyện lại và rollback.

Chạy từ gốc repository:
    uv run --project examples/toi-noi-python/60-model-registry-ab-drift \
        examples/toi-noi-python/60-model-registry-ab-drift/main.py
    ... main.py --verify    # kiểm các bất biến in trong bài, in OK n/n

Không cần Docker. Trên macOS, LightGBM cần OpenMP: `brew install libomp`.
Dữ liệu là TỔNG HỢP (synthetic_fraud.py, giống bài 54 đến 59): mọi con số chỉ nói về bộ này, và điểm rủi ro là đầu vào
cho người duyệt chứ không phải quyết định khóa tài khoản. Hai kịch bản có chủ ý đặt vào dữ liệu (nhãn «kịch bản» trong
output): nguồn GPS đổi ở mục drift, và việc promote nhầm ở mục rollback. Nhãn dùng để đánh giá là nhãn cuối; v3 là bản duy
nhất huấn luyện bằng nhãn biết tại ngày 50.
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

import lightgbm
import numpy as np
import polars as pl
import sklearn

import metrics
from costs import trip_costs
from drift import PSI_ALERT, PSI_WARN, RetrainPolicy, psi, retrain_decision
from features import FEATURES
from model import HANDLING_COST_VND, REVIEW_COST_VND, expected_cost_flag
from prepare import N_TRIPS, OUT, SEED, load_table
from registry import Registry, RegistryError
from routing import BUCKETS, ShadowRouter, assign_arm, bucket, decide
from synthetic_fraud import make_events
from train import V1, V2, V3, V3F, VX, gain_share, impact_share, in_memory, paid_day, train

HERE = Path(__file__).parent
REG_DIR = OUT / "registry"
SHADOW_LOG = OUT / "shadow_log.jsonl"
WINDOW_FROM = 50  # từ ngày 50: v1 đang phục vụ, v2 và v3 chạy shadow, A/B
HOLDOUT = (40, 50)  # số đo ghi vào manifest
EXPERIMENT = "fraud-v2-vs-v1"
BATCH = 1_000
SEEDS = (42, 7, 123)
HIST_FEATURES = {"cust_prior_trips", "cust_promo_share_30d"}  # hai đặc trưng dựa vào lịch sử khách


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


def pr(y: np.ndarray, p: np.ndarray) -> float:
    return metrics.average_precision(y, p)


def load_customers() -> pl.DataFrame:
    """trip_id -> customer_id, device_id (lấy từ BookingCreated, bảng đặc trưng không giữ hai cột này). Lưu out/customers.parquet."""
    path = OUT / "customers.parquet"
    if path.exists():
        return pl.read_parquet(path)
    events = make_events(seed=SEED, n_trips=N_TRIPS).events
    df = events.filter(pl.col("event_type") == "BookingCreated").select("trip_id", "customer_id", "device_id")
    OUT.mkdir(exist_ok=True)
    df.write_parquet(path)
    return df


# ---- [1] Dữ liệu ------------------------------------------------------------------------------


def step_data(c: Checker) -> dict:
    say("[1] Môi trường và dữ liệu")
    say(f"Python {sys.version.split()[0]}, scikit-learn {sklearn.__version__}, lightgbm {lightgbm.__version__}, numpy {np.__version__}, polars {pl.__version__}")
    t0 = time.perf_counter()
    table = load_table()
    customers = load_customers()
    day = paid_day(table)
    window = table.filter(pl.Series(day >= WINDOW_FROM)).join(customers, on="trip_id", how="left", maintain_order="left")
    yw = window["is_fraud"].to_numpy().astype(int)
    say(f"bảng chuyến {table.height:,} dòng ({time.perf_counter() - t0:.1f} giây, gồm dựng đặc trưng nếu chưa có out/table.parquet)")
    say(f"cửa sổ chạy thử: ngày {WINDOW_FROM} đến 60: {window.height:,} chuyến, {window['customer_id'].n_unique():,} khách, "
        f"{yw.sum()} gian lận ({yw.mean():.2%})")
    c.check("bảng 200.000 chuyến, cửa sổ ngày 50+ không có chuyến thiếu khách", table.height == 200_000 and window["customer_id"].null_count() == 0)
    return {"table": table, "day": day, "window": window, "yw": yw}


# ---- [2] Huấn luyện ba phiên bản và đăng ký -----------------------------------------------------


def step_train(d: dict, c: Checker) -> tuple[Registry, dict, dict]:
    say("\n[2] Ba phiên bản: huấn luyện và đăng ký vào registry (thư mục)")
    table, day = d["table"], d["day"]
    shutil.rmtree(REG_DIR, ignore_errors=True)
    reg = Registry(REG_DIR)
    hold = (day >= HOLDOUT[0]) & (day < HOLDOUT[1])
    hx = table.filter(pl.Series(hold))
    hy = hx["is_fraud"].to_numpy().astype(int)
    say(f"holdout ghi vào manifest: ngày {HOLDOUT[0]} đến {HOLDOUT[1]}, {len(hy):,} chuyến, {hy.sum()} gian lận, nhãn cuối")
    say(f"{'bản':>4s}{'huấn luyện đến':>16s}{'nhãn':>8s}{'chuyến':>9s}{'gian lận':>10s}{'ngưỡng':>8s}{'PR-AUC holdout':>16s}{'chồng train':>13s}{'KiB':>6s}  version")
    versions, models = {}, {}
    parent = None
    t0 = time.perf_counter()
    for spec in (V1, V2, V3):
        cal, meta = train(table, spec)
        p = cal.predict_proba(hx.select(FEATURES))[:, 1]
        overlaps = HOLDOUT[0] < spec.end_day
        holdout = {"days": list(HOLDOUT), "rows": int(len(hy)), "pr_auc": round(pr(hy, p), 4), "overlaps_train": overlaps}
        version = reg.register(cal, meta, actor="svc:fraud-trainer", parent=parent, holdout=holdout, note=spec.note)
        parent = parent or version  # v2 và v3 cùng là con của v1 (champion lúc huấn luyện)
        versions[spec.name] = version
        models[spec.name] = reg.load(version)
        m = reg.manifest(version)
        say(f"{spec.name:>4s}{f'ngày {spec.end_day}':>16s}{spec.labels:>8s}{meta['trained_on']['rows']:>9,d}"
            f"{int(round(meta['trained_on']['train_fraud_rate'] * meta['trained_on']['rows'])):>10,d}{models[spec.name].threshold:>8.3f}{holdout['pr_auc']:>16.4f}"
            f"{'có' if overlaps else 'không':>13s}{m['bytes'] / 1024:>6.0f}  {version}")
    say(f"(huấn luyện và đăng ký ba bản: {time.perf_counter() - t0:.0f} giây; load average 1/5/15 phút: {load_avg()})")
    say("manifest của v1:")
    say(json.dumps(reg.manifest(versions["v1"]), ensure_ascii=False, indent=2, sort_keys=True))
    say("thư mục registry:")
    for p in sorted(REG_DIR.rglob("*")):
        if p.is_file() and not p.name.startswith("."):
            say(f"  {p.relative_to(OUT)}  ({p.stat().st_size:,} byte)")
    h = {n: reg.manifest(v)["holdout"] for n, v in versions.items()}
    c.check("ba phiên bản có sha256 khác nhau", len({reg.manifest(v)["sha256"] for v in versions.values()}) == 3)
    c.check("holdout của v1 và v2 không chồng train, của v3 chồng", not h["v1"]["overlaps_train"] and not h["v2"]["overlaps_train"] and h["v3"]["overlaps_train"])
    c.check("PR-AUC holdout của v1 cao hơn 20 lần tỉ lệ gian lận nền", h["v1"]["pr_auc"] > 20 * hy.mean(), (h["v1"]["pr_auc"], hy.mean()))
    c.check("model_version khớp tên thư mục và registry nạp lại được", all(models[n].version == versions[n] for n in versions))
    c.check("mỗi bản có ngưỡng riêng trong meta, nằm trong (0, 1)", all(0 < models[n].threshold < 1 for n in models))
    return reg, versions, models


# ---- [3] Registry: promote, rollback và những lần từ chối ---------------------------------------


def expect_error(fn, needle: str) -> str:
    try:
        fn()
    except RegistryError as e:
        return str(e) if needle in str(e) else f"lỗi khác: {e}"
    return "KHÔNG LỖI"


def step_registry(reg: Registry, versions: dict, c: Checker) -> None:
    say("\n[3] Registry: promote, và những lần từ chối")
    v1, v2, v3 = versions["v1"], versions["v2"], versions["v3"]
    reg.promote(v1, actor="oncall-ml", reason="bản đầu tiên, cùng công thức mô hình bài 57")
    say(f"promote v1 -> current = {reg.current()}")
    tmp = Path(tempfile.mkdtemp(prefix="lab60-"))
    try:
        sandbox = tmp / "registry"
        shutil.copytree(REG_DIR, sandbox)
        sb = Registry(sandbox)
        cases: list[tuple[str, str]] = []

        def case(name: str, fn, needle: str) -> None:
            got = expect_error(fn, needle)
            cases.append((name, got))
            say(f"  {name}: {got}")

        case("promote v3 (holdout chồng train)", lambda: sb.promote(v3, actor="oncall-ml", reason="thử"), "chồng lên")
        blob = (sandbox / v2 / "model.joblib").read_bytes()
        (sandbox / v2 / "model.joblib").write_bytes(blob[:-1] + bytes([blob[-1] ^ 1]))
        case("promote v2 sau khi đổi 1 bit trong file", lambda: sb.promote(v2, actor="oncall-ml", reason="thử"), "sha256")
        (sandbox / v2 / "model.joblib").write_bytes(blob)
        mf = sandbox / v2 / "manifest.json"
        data = json.loads(mf.read_text())
        data["holdout"]["pr_auc"] = 0.5
        mf.write_text(json.dumps(data))
        case("promote bản có PR-AUC holdout 0,5 (sửa manifest bản sao)", lambda: sb.promote(v2, actor="oncall-ml", reason="thử"), "kém champion")
        data["holdout"]["pr_auc"] = 0.89
        data["holdout"]["days"] = [41, 50]
        mf.write_text(json.dumps(data))
        case("promote bản có holdout khác ngày với champion (sửa manifest bản sao)", lambda: sb.promote(v2, actor="oncall-ml", reason="thử"), "khác ngày")
        data["holdout"] = None
        mf.write_text(json.dumps(data))
        case("promote bản không có số đo holdout", lambda: sb.promote(v2, actor="oncall-ml", reason="thử"), "chưa có số đo")
        case("promote với lý do rỗng", lambda: sb.promote(v2, actor="oncall-ml", reason="  "), "lý do")
        case("promote phiên bản không tồn tại", lambda: sb.promote("fraud-lgbm-00000000", actor="oncall-ml", reason="thử"), "không có phiên bản")
        case("rollback khi chỉ có một bản", lambda: sb.rollback(actor="oncall-ml", reason="thử"), "không có bản trước")
        (sandbox / "current.json").write_text(json.dumps({"version": v3, "since": "giả lập chết giữa hai bước"}))
        case("promote khi con trỏ lệch lịch sử (giả lập chết sau khi đổi con trỏ)", lambda: sb.promote(v2, actor="oncall-ml", reason="thử"), "lệch lịch sử")
        case("rollback khi con trỏ lệch lịch sử", lambda: sb.rollback(actor="oncall-ml", reason="thử"), "lệch lịch sử")
        empty = Registry(tmp / "empty")
        loaded = reg.load(v1)
        case("đăng ký lại cùng mô hình (cùng sha256)", lambda: empty.register(loaded.model, loaded.meta, actor="x") and empty.register(loaded.model, loaded.meta, actor="x"), "đã có")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    names = ("chặn holdout chồng train", "chặn file bị sửa", "chặn bản kém champion", "chặn holdout khác ngày", "chặn bản không có số đo",
             "chặn lý do rỗng", "chặn phiên bản không có", "chặn rollback khi chưa có bản trước", "chặn promote khi con trỏ lệch lịch sử",
             "chặn rollback khi con trỏ lệch lịch sử", "registry không ghi đè phiên bản")
    for (_, got), label in zip(cases, names):
        c.check(label, not got.startswith(("KHÔNG", "lỗi khác")), got)
    c.check("current vẫn là v1 sau mọi lần từ chối ở bản sao", reg.current() == v1)


# ---- [4] Shadow mode -----------------------------------------------------------------------------


class BrokenScorer:
    """Mô hình thử hỏng: ném lỗi ở mọi lần chấm."""

    version = "v-broken"
    threshold = 0.5

    def predict_proba(self, features: pl.DataFrame) -> np.ndarray:
        raise ValueError("mô hình thử hỏng")


class WrongLengthScorer:
    """Mô hình thử trả sai độ dài: thiếu một điểm so với số chuyến."""

    version = "v-wrong-length"
    threshold = 0.5

    def predict_proba(self, features: pl.DataFrame) -> np.ndarray:
        return np.zeros(features.height - 1)


def step_shadow(d: dict, reg: Registry, versions: dict, models: dict, c: Checker) -> dict:
    say(f"\n[4] Shadow mode: ngày {WINDOW_FROM} đến 60, v1 quyết định, v2 và v3 chỉ ghi điểm")
    w, yw = d["window"], d["yw"]
    amount = w["amount_vnd"].to_numpy().astype(float)
    champion = reg.load_current()
    v1, v2, v3 = versions["v1"], versions["v2"], versions["v3"]
    SHADOW_LOG.unlink(missing_ok=True)
    router = ShadowRouter(champion, {v2: models["v2"], v3: models["v3"]}, SHADOW_LOG)
    t0 = time.perf_counter()
    flags = np.concatenate([router.decide(w[i:i + BATCH]) for i in range(0, w.height, BATCH)])
    t_all = time.perf_counter() - t0
    t0 = time.perf_counter()
    p_alone = champion.predict_proba(w)
    t_champ = time.perf_counter() - t0
    flags_alone = decide(p_alone, amount)
    log = pl.read_ndjson(SHADOW_LOG)
    say(f"đã chấm {w.height:,} chuyến theo lô {BATCH:,}; log {SHADOW_LOG.relative_to(OUT.parent)}: {log.height:,} dòng, {SHADOW_LOG.stat().st_size / 1e6:.1f} MB")
    say("một dòng log: " + SHADOW_LOG.read_text(encoding="utf-8").splitlines()[0].replace(v1, "v1").replace(v2, "v2").replace(v3, "v3"))
    say(f"quyết định của router có shadow giống hệt quyết định của champion chạy một mình: {bool(np.array_equal(flags, flags_alone))} ({int(flags.sum())} cờ theo quy tắc chi phí kỳ vọng)")
    say(f"thời gian chấm {w.height:,} chuyến: champion một mình {t_champ:.2f} giây; champion + 2 shadow + ghi log {t_all:.2f} giây (load average 1/5/15 phút: {load_avg()})")

    rng = np.random.default_rng(0)
    sample = rng.choice(w.height, size=100, replace=False)
    one = np.array([champion.score_one(w.row(int(i), named=True)) for i in sample])
    max_diff = float(np.abs(one - p_alone[sample]).max())
    say(f"champion: score_one trên 100 chuyến ngẫu nhiên lệch điểm theo lô tối đa {max_diff:.2e}")

    tmpdir = Path(tempfile.mkdtemp(prefix="lab60-"))
    bad_router = ShadowRouter(champion, {"v-broken": BrokenScorer()}, tmpdir / "bad.jsonl")
    bad_flags = np.concatenate([bad_router.decide(w[i:i + BATCH]) for i in range(0, w.height, BATCH)])
    bad_first = (tmpdir / "bad.jsonl").read_text(encoding="utf-8").splitlines()[0]
    len_router = ShadowRouter(champion, {"v-wrong-length": WrongLengthScorer()}, tmpdir / "len.jsonl")
    len_flags = np.concatenate([len_router.decide(w[i:i + BATCH]) for i in range(0, w.height, BATCH)])
    log_router = ShadowRouter(champion, {v2: models["v2"]}, tmpdir / "khong-co-thu-muc" / "x.jsonl")  # thư mục không tồn tại: ghi log lỗi
    log_flags = np.concatenate([log_router.decide(w[i:i + BATCH]) for i in range(0, w.height, BATCH)])
    n_batches = -(-w.height // BATCH)
    shutil.rmtree(tmpdir, ignore_errors=True)
    say(f"shadow hỏng: quyết định vẫn giống hệt = {bool(np.array_equal(bad_flags, flags_alone))}, {bad_router.shadow_errors['v-broken']} lô lỗi được ghi; dòng log đầu: "
        + bad_first.replace(v1, "v1")[:150])
    say(f"shadow trả sai độ dài: quyết định giống hệt = {bool(np.array_equal(len_flags, flags_alone))}, {len_router.shadow_errors['v-wrong-length']} lô lỗi; "
        f"ghi log vào thư mục không có: quyết định giống hệt = {bool(np.array_equal(log_flags, flags_alone))}, {log_router.log_errors} lô lỗi log")

    joined = w.select("trip_id").join(log, on="trip_id", how="left", maintain_order="left")
    p = {n: joined[f"p_{v}"].to_numpy() for n, v in (("v1", v1), ("v2", v2), ("v3", v3))}
    base = float(yw.mean())
    thr = {n: models[n].threshold for n in p}
    say(f"\nso sánh trên nhãn cuối (cửa sổ {w.height:,} chuyến, {int(yw.sum())} gian lận, tỉ lệ nền {base:.4f}):")
    say(f"{'bản':>4s}{'PR-AUC':>8s}{'p trung bình':>14s}{'p99':>8s}{'PSI điểm':>10s}")
    res = {}
    for n in ("v1", "v2", "v3"):
        r = dict(pr=pr(yw, p[n]), mean=float(p[n].mean()), p99=float(np.quantile(p[n], 0.99)), psi=psi(p["v1"], p[n]))
        for key, f in (("ec", expected_cost_flag(p[n], amount)), ("v1thr", p[n] >= thr["v1"]), ("own", p[n] >= thr[n])):
            r[f"rate_{key}"] = float(f.mean())
            r[f"cost_{key}"] = float(trip_costs(f, yw, amount).sum())
        res[n] = r
        say(f"{n:>4s}{r['pr']:8.4f}{r['mean']:14.4f}{r['p99']:8.4f}{r['psi']:10.3f}")
    say(f"\nquyết định và chi phí (triệu VND; phí duyệt {REVIEW_COST_VND:,} cho mọi cờ, bỏ sót = tiền chuyến + {HANDLING_COST_VND:,}). Quy tắc chi phí kỳ vọng là quyết định chính; hai ngưỡng để so sánh:")
    say(f"{'bản':>4s}{'quy tắc chi phí kỳ vọng':>33s}{'ngưỡng v1':>31s}{'ngưỡng riêng của bản':>38s}")
    say(f"{'':>4s}{'cờ/1000':>12s}{'chi phí':>10s}{'':>9s}{'cờ/1000':>10s}{'chi phí':>10s}{'':>9s}{'ngưỡng':>9s}{'cờ/1000':>10s}{'chi phí':>10s}")
    for n in ("v1", "v2", "v3"):
        r = res[n]
        say(f"{n:>4s}{1000 * r['rate_ec']:>12.1f}{r['cost_ec'] / 1e6:>10.2f}{'':>9s}{1000 * r['rate_v1thr']:>10.1f}{r['cost_v1thr'] / 1e6:>10.2f}{'':>9s}{thr[n]:>9.3f}{1000 * r['rate_own']:>10.1f}{r['cost_own'] / 1e6:>10.2f}")
    tot = metrics.total_cost(yw, p["v1"], thr["v1"], amount + HANDLING_COST_VND, REVIEW_COST_VND)
    c.check("quyết định có shadow giống hệt quyết định champion một mình", bool(np.array_equal(flags, flags_alone)))
    c.check("log shadow có đủ một dòng mỗi chuyến, mỗi dòng đủ ba điểm", log.height == w.height and all(log[f"p_{v}"].null_count() == 0 for v in (v1, v2, v3)))
    c.check("champion: score_one khớp chấm theo lô (lệch dưới 1e-9)", max_diff < 1e-9, max_diff)
    c.check("shadow hỏng không đổi quyết định, lỗi được ghi", bool(np.array_equal(bad_flags, flags_alone)) and bad_router.shadow_errors["v-broken"] == n_batches and "ValueError" in bad_first)
    c.check("shadow trả sai độ dài không đổi quyết định, lỗi được đếm", bool(np.array_equal(len_flags, flags_alone)) and len_router.shadow_errors["v-wrong-length"] == n_batches)
    c.check("log shadow không ghi được không đổi quyết định, lỗi log được đếm", bool(np.array_equal(log_flags, flags_alone)) and log_router.log_errors == n_batches)
    c.check("tổng chi phí từng chuyến khớp metrics.total_cost", abs(res["v1"]["cost_v1thr"] - tot) < 1e-6, (res["v1"]["cost_v1thr"], tot))
    c.check("v3 xếp hạng không tệ hơn v1 quá 0,02 PR-AUC (PR-AUC không thấy lỗi của v3)", res["v3"]["pr"] > res["v1"]["pr"] - 0.02, (res["v3"]["pr"], res["v1"]["pr"]))
    c.check("v3 gắn cờ ít hơn 0,75 lần v1 theo quy tắc chi phí kỳ vọng và p trung bình thấp hơn 0,6 lần tỉ lệ nền", res["v3"]["rate_ec"] < 0.75 * res["v1"]["rate_ec"] and res["v3"]["mean"] < 0.6 * base, (res["v3"], base))
    c.check("v3 đắt hơn v1 ít nhất 1,15 lần theo quy tắc chi phí kỳ vọng, ngưỡng của v1 và ngưỡng riêng",
            all(res["v3"][f"cost_{k}"] > 1.15 * res["v1"][f"cost_{k}"] for k in ("ec", "v1thr", "own")), res)
    c.check("p99 của v3 nằm dưới ngưỡng của v1 còn p99 của v1 và v2 nằm trên", res["v3"]["p99"] < thr["v1"] < min(res["v1"]["p99"], res["v2"]["p99"]), {n: res[n]["p99"] for n in res})
    return {"p": p, "res": res, "log": log, "thr": thr}


# ---- [5] A/B theo hash ---------------------------------------------------------------------------


def per1000(cost: np.ndarray, mask: np.ndarray) -> float:
    return 1000.0 * float(cost[mask].sum()) / max(1, int(mask.sum()))


def cluster_ci(cost: np.ndarray, inv: np.ndarray, t_mask: np.ndarray, c_mask: np.ndarray, *, boots: int = 1000, seed: int = 0):
    """Khoảng tin cậy 95 % của (chi phí/1000 chuyến nhóm treatment) - (nhóm control), bootstrap THEO KHÁCH (chuyến của cùng khách không độc lập)."""
    rng = np.random.default_rng(seed)

    def stats(mask):
        cs, ns = np.bincount(inv[mask], weights=cost[mask]), np.bincount(inv[mask])
        keep = ns > 0
        return cs[keep], ns[keep]

    (ct, nt), (cc, nc) = stats(t_mask), stats(c_mask)
    out = np.empty(boots)
    for b in range(boots):
        i, j = rng.integers(0, len(ct), len(ct)), rng.integers(0, len(cc), len(cc))
        out[b] = 1000 * ct[i].sum() / nt[i].sum() - 1000 * cc[j].sum() / nc[j].sum()
    return float(np.percentile(out, 2.5)), float(np.percentile(out, 97.5))


def step_ab(d: dict, sh: dict, c: Checker) -> None:
    say(f"\n[5] A/B theo hash khách: thí nghiệm «{EXPERIMENT}», 50 % khách vào v2 (treatment), 50 % còn lại giữ v1 (control)")
    w, yw = d["window"], d["yw"]
    amount = w["amount_vnd"].to_numpy().astype(float)
    cust = w["customer_id"].to_numpy()
    uniq, inv = np.unique(cust, return_inverse=True)
    t0 = time.perf_counter()
    arm_of = {u: assign_arm(u, EXPERIMENT, treatment_share=0.5) for u in uniq}
    arm = np.array([arm_of[u] for u in cust])
    t_mask, c_mask = arm == "treatment", arm == "control"
    n_t, n_c = sum(a == "treatment" for a in arm_of.values()), sum(a == "control" for a in arm_of.values())
    say(f"chia {len(uniq):,} khách ({time.perf_counter() - t0:.1f} giây): treatment {n_t:,} khách / {int(t_mask.sum()):,} chuyến / {int(yw[t_mask].sum())} gian lận; "
        f"control {n_c:,} khách / {int(c_mask.sum()):,} chuyến / {int(yw[c_mask].sum())} gian lận")

    probe = "from routing import assign_arm; print(''.join('T' if assign_arm(f'CU-{i:05d}', 'fraud-v2-vs-v1', treatment_share=0.5) == 'treatment' else 'C' for i in range(40)))"
    outs = []
    for seed in ("1", "2"):
        r = subprocess.run([sys.executable, "-c", probe], cwd=HERE, capture_output=True, text=True, env={**os.environ, "PYTHONHASHSEED": seed, "PYTHONPATH": str(HERE)}, check=True)
        outs.append(r.stdout.strip())
    here = "".join("T" if assign_arm(f"CU-{i:05d}", EXPERIMENT, treatment_share=0.5) == "treatment" else "C" for i in range(40))
    say(f"40 khách đầu CU-00000..CU-00039 qua hai tiến trình mới với PYTHONHASHSEED khác nhau: {outs[0] == outs[1] == here} ({here})")

    other = np.array([bucket(u, "fraud-promo-copy-v1") < 0.5 * BUCKETS for u in uniq])
    mine = np.array([arm_of[u] == "treatment" for u in uniq])
    joint = float((mine & other).mean())
    say(f"hai thí nghiệm khác tên: khách cùng ở treatment của cả hai = {joint:.3f} (độc lập sẽ là {mine.mean() * other.mean():.3f})")

    # mỗi nhóm quyết định bằng quy tắc chi phí kỳ vọng với điểm của mô hình nhóm đó
    cost1 = trip_costs(expected_cost_flag(sh["p"]["v1"], amount), yw, amount)
    cost2 = trip_costs(expected_cost_flag(sh["p"]["v2"], amount), yw, amount)
    cost_arm = np.where(t_mask, cost2, cost1)
    ct, cc = per1000(cost_arm, t_mask), per1000(cost_arm, c_mask)
    lo, hi = cluster_ci(cost_arm, inv, t_mask, c_mask)
    say(f"\nchi phí (VND) trên 1.000 chuyến, treatment dùng v2, control dùng v1, cùng quy tắc chi phí kỳ vọng: treatment {ct:,.0f}, control {cc:,.0f}, chênh {ct - cc:+,.0f}")
    say(f"khoảng tin cậy 95 % của chênh (bootstrap theo khách, 1.000 lần): [{lo:+,.0f}, {hi:+,.0f}]")

    diffs = []
    for k in range(100):
        m = np.array([bucket(u, f"aa-{k}") < 0.5 * BUCKETS for u in uniq])[inv]
        diffs.append(per1000(cost1, m) - per1000(cost1, ~m))
    diffs = np.array(diffs)
    say(f"A/A (cả hai nhóm dùng v1, 100 cách chia): chênh có độ lệch chuẩn {diffs.std():,.0f}, 95 % nằm trong [{np.percentile(diffs, 2.5):+,.0f}, {np.percentile(diffs, 97.5):+,.0f}]")
    gain = (cost1.sum() - cost2.sum()) / len(yw) * 1000
    say(f"toàn cửa sổ, nếu mọi khách dùng v2 thay v1: chi phí/1.000 chuyến thay đổi {-gain:+,.0f} (v1 {1000 * cost1.sum() / len(yw):,.0f})")
    half = (hi - lo) / 2
    scale = (half / max(abs(gain), 1.0)) ** 2
    power80 = scale * ((1.96 + 0.8416) / 1.96) ** 2  # n tăng theo bình phương: để power 80 % cần (z_0,975 + z_0,8)^2 lần cỡ của «nửa độ rộng bằng chênh» (power chừng 50 %)
    say(f"nửa độ rộng khoảng tin cậy {half:,.0f} so với chênh thật {abs(gain):,.0f}: nửa độ rộng bằng chênh (power chừng 50 %) cần chừng {scale:,.0f} lần lưu lượng 10 ngày này, "
        f"power 80 % cần chừng {power80:,.0f} lần, tức {power80 * len(yw) / 1e6:,.1f} triệu chuyến (ngoại suy thô: chuyến độc lập, chênh không đổi; ở lab chênh thật đổi dấu theo seed)")

    dev = w.select("device_id", "customer_id").with_columns(arm=pl.Series(arm)).unique().group_by("device_id").agg(
        n_cust=pl.col("customer_id").n_unique(), n_arm=pl.col("arm").n_unique())
    multi = dev.filter(pl.col("n_cust") >= 2)
    both = multi.filter(pl.col("n_arm") == 2).height
    say(f"thiết bị có từ 2 khách trở lên: {multi.height:,}; trong đó {both:,} thiết bị ({both / max(1, multi.height):.0%}) có khách ở cả hai nhóm")
    c.check("tỉ lệ khách vào treatment nằm trong 0,45 đến 0,55", 0.45 < n_t / (n_t + n_c) < 0.55, n_t / (n_t + n_c))
    c.check("chia nhóm giống hệt qua tiến trình mới với PYTHONHASHSEED khác", outs[0] == outs[1] == here, outs)
    c.check("hai thí nghiệm khác tên chia gần độc lập", abs(joint - mine.mean() * other.mean()) < 0.02, joint)
    c.check("mỗi khách chỉ ở một nhóm, mọi chuyến của khách cùng nhóm", all(len(set(arm[inv == i])) == 1 for i in range(0, len(uniq), 97)))
    c.check("khoảng tin cậy của chênh v2 - v1 chứa 0 (10 ngày không đủ phân biệt)", lo < 0 < hi, (lo, hi))
    c.check("độ lệch chuẩn A/A lớn hơn 1/3 chênh v2 so v1 trên toàn cửa sổ (nhiễu chia khách cỡ hiệu ứng)", diffs.std() > abs(gain) / 3, (diffs.std(), gain))
    c.check("có thiết bị nhiều khách nằm ở cả hai nhóm (nhiễu giao thoa tồn tại)", both > 0, both)


# ---- [6] Drift ----------------------------------------------------------------------------------


def feat_arrays(df: pl.DataFrame) -> dict[str, np.ndarray]:
    return {f: df[f].to_numpy().astype(float) for f in FEATURES}


def psi_summary(ref: dict[str, np.ndarray], cur: pl.DataFrame):
    vals = {f: psi(ref[f], cur[f].to_numpy().astype(float)) for f in FEATURES}
    return vals, sorted(vals.items(), key=lambda kv: -kv[1])


def gps_x3_from(table: pl.DataFrame, day: np.ndarray, from_day: float) -> pl.DataFrame:
    """Kịch bản: từ ngày `from_day` nguồn GPS đổi, start_pickup_gap_km lớn gấp 3 (chỉ sửa đúng cột này)."""
    return table.with_columns(
        pl.when(pl.Series(day >= from_day)).then(pl.col("start_pickup_gap_km") * 3.0).otherwise(pl.col("start_pickup_gap_km")).alias("start_pickup_gap_km"))


def broken_source_costs(table: pl.DataFrame, day: np.ndarray, models_v1, models_v2, amount: np.ndarray, y: np.ndarray, w: pl.DataFrame) -> dict[str, float]:
    """Chi phí (quy tắc chi phí kỳ vọng) trên cửa sổ ngày 50+ với nguồn cũ và nguồn đổi; vx học đến ngày 50 trên dữ liệu có lỗi từ ngày 30."""
    wi = w.with_columns(pl.col("start_pickup_gap_km") * 3.0)
    cal, _ = train(gps_x3_from(table, day, 30), VX)
    p_vx = cal.predict_proba(wi.select(FEATURES))[:, 1]
    out = {}
    for name, p in (("v1 nguồn cũ", models_v1.predict_proba(w)), ("v1 nguồn đổi", models_v1.predict_proba(wi)),
                    ("v2 nguồn đổi", models_v2.predict_proba(wi)), ("vx nguồn đổi", p_vx)):
        out[name] = float(trip_costs(expected_cost_flag(p, amount), y, amount).sum())
    return out


def step_drift(d: dict, models: dict, sh: dict, c: Checker) -> dict:
    say("\n[6] Drift: PSI từng đặc trưng so với dữ liệu huấn luyện của v1 (ngày 0 đến 30)")
    table, day, w, yw = d["table"], d["day"], d["window"], d["yw"]
    ref_df = table.filter(pl.Series(day < 30))
    ref = feat_arrays(ref_df)
    imp = impact_share(models["v1"], ref_df)
    gain = gain_share(models["v1"])
    rng = np.random.default_rng(0)
    idx = rng.permutation(ref_df.height)
    a, b = ref_df[idx[:23_000]], ref_df[idx[23_000:46_000]]
    noise = max(psi(a[f].to_numpy().astype(float), b[f].to_numpy().astype(float)) for f in FEATURES)
    say(f"nhiễu nền: PSI lớn nhất giữa hai nửa ngẫu nhiên 23.000 dòng của chính tập tham chiếu = {noise:.4f}")
    say(f"{'cửa sổ (ngày)':>16s}{'chuyến':>8s}{'PR-AUC v1':>11s}{'cờ/1000':>9s}{'>=0,10':>8s}{'>=0,25':>8s}  ba đặc trưng PSI lớn nhất")
    weekly, weekly_pr, wk = {}, [], []
    for lo, hi in ((40, 47), (47, 54), (54, 60.01)):
        cur = table.filter(pl.Series((day >= lo) & (day < hi)))
        wk.append(cur)
        y = cur["is_fraud"].to_numpy().astype(int)
        p = models["v1"].predict_proba(cur)
        vals, top = psi_summary(ref, cur)
        weekly[(lo, hi)] = (vals, top)
        weekly_pr.append(pr(y, p))
        say(f"{f'{lo} đến {hi:.0f}':>16s}{cur.height:8,d}{weekly_pr[-1]:11.3f}{1000 * expected_cost_flag(p, cur['amount_vnd'].to_numpy().astype(float)).mean():9.1f}"
            f"{sum(v >= PSI_WARN for v in vals.values()):8d}{sum(v >= PSI_ALERT for v in vals.values()):8d}  "
            + ", ".join(f"{k} {v:.3f}" for k, v in top[:3]))
    say(f"cust_prior_trips đếm số chuyến trước đó của khách từ đầu log (không có cửa sổ): trung bình {ref['cust_prior_trips'].mean():.1f} ở tập tham chiếu, "
        + ", ".join(f"{x['cust_prior_trips'].mean():.1f}" for x in wk) + " ở ba tuần")
    top_imp = sorted(imp.items(), key=lambda kv: -kv[1])
    rank = [k for k, _ in top_imp].index("cust_prior_trips") + 1
    say(f"độ quan trọng của cust_prior_trips trong v1: {imp['cust_prior_trips']:.1%} tổng mean|SHAP| (hạng {rank}/30), nhưng chỉ {gain['cust_prior_trips']:.1%} gain; "
        "ba đặc trưng đầu theo mean|SHAP|: " + ", ".join(f"{k} {v:.1%}" for k, v in top_imp[:3]))
    first = [(x["cust_prior_trips"] == 0).mean() for x in [ref_df, *wk]]
    say("tỉ lệ chuyến đầu tiên của khách (cust_prior_trips = 0): " + f"{first[0]:.1%} ở tập tham chiếu, " + ", ".join(f"{v:.1%}" for v in first[1:]) + " ở ba tuần")
    clean = [x.filter(~pl.col("is_fraud")) for x in [ref_df, *wk]]
    nu = [x["has_newuser_promo"].mean() for x in clean]
    say("hai tạo tác của bộ sinh dữ liệu (không phải hiện tượng thật):")
    say("  (a) tỉ lệ chuyến SẠCH dùng NEWUSER50: " + f"{nu[0]:.1%} ở tập tham chiếu, " + ", ".join(f"{v:.1%}" for v in nu[1:]) + " ở ba tuần (quần thể khách gần như cố định nên khách mới cạn dần)")
    wc = w.filter(~pl.col("is_fraud"))["cust_promo_share_30d"]
    wf = w.filter(pl.col("is_fraud"))["cust_promo_share_30d"]
    pos_clean, pos_fraud = float((wc > 0).mean()), float((wf > 0).mean())
    say(f"  (b) cust_promo_share_30d > 0 ở cửa sổ ngày 50+: {pos_clean:.1%} chuyến sạch, {pos_fraud:.1%} chuyến gian lận (khách gian lận không dùng mã nào ngoài NEWUSER50 của promo_abuse)")
    sparse = np.r_[np.zeros(9_990), np.ones(10)]
    shifted = np.r_[np.zeros(5_000), np.ones(5_000)]
    p_sparse = psi(sparse, shifted)
    p_low = psi(np.r_[np.ones(9_990), np.zeros(10)], np.r_[np.ones(500), np.zeros(500)])
    say(f"PSI hai phía cho cờ hiếm: tham chiếu 99,9 % số 0, hiện tại 50 % số 1: {p_sparse:.2f}; tham chiếu 99,9 % số 1, hiện tại 50 % số 0: {p_low:.2f}")

    say("\nkịch bản (đặt vào dữ liệu, không quan sát): từ ngày 50 nguồn GPS đổi, start_pickup_gap_km của mọi chuyến lớn gấp 3")
    wi = w.with_columns(pl.col("start_pickup_gap_km") * 3.0)
    vals_w, _ = psi_summary(ref, w)
    vals_i, _ = psi_summary(ref, wi)
    amount = w["amount_vnd"].to_numpy().astype(float)
    say(f"{'':14s}{'PSI start_pickup_gap_km':>25s}{'đặc trưng >=0,25':>18s}{'PR-AUC v1':>11s}{'cờ v1':>7s}{'chi phí v1':>13s}{'chi phí v2':>13s}")
    out = {}
    for label, df, vals in (("nguồn cũ", w, vals_w), ("nguồn đổi (x3)", wi, vals_i)):
        p1, p2 = models["v1"].predict_proba(df), models["v2"].predict_proba(df)
        f1, f2 = expected_cost_flag(p1, amount), expected_cost_flag(p2, amount)
        o = dict(psi=vals["start_pickup_gap_km"], n_alert=sum(v >= PSI_ALERT for v in vals.values()), pr=pr(yw, p1), flags=int(f1.sum()),
                 cost1=float(trip_costs(f1, yw, amount).sum()), cost2=float(trip_costs(f2, yw, amount).sum()))
        out[label] = o
        say(f"{label:14s}{o['psi']:25.3f}{o['n_alert']:18d}{o['pr']:11.3f}{o['flags']:7,d}{o['cost1']:13,.0f}{o['cost2']:13,.0f}")
    bs = broken_source_costs(table, day, models["v1"], models["v2"], amount, yw, w)
    say("huấn luyện lại thì sao? vx học đến ngày 50 (nhãn cuối) trên dữ liệu có lỗi nguồn từ ngày 30; chi phí (triệu VND) trên cửa sổ ngày 50+, quy tắc chi phí kỳ vọng:")
    say("  " + ", ".join(f"{k} {v / 1e6:.2f}" for k, v in bs.items()))
    say("(v2 học trước lúc nguồn đổi nên không cứu được; vx học luôn dữ liệu lỗi nên số đo hồi phục, nhưng nó đã học một nguồn sai: vẫn phải tìm và sửa nguồn)")

    c.check("PSI của một mẫu với chính nó bằng 0", psi(ref["speed_kmh"], ref["speed_kmh"]) == 0.0)
    c.check("nhiễu nền PSI dưới 0,01 ở cỡ cửa sổ một tuần", noise < 0.01, noise)
    c.check("PSI bắt được cờ hiếm đổi ở cả hai phía (vượt 0,25)", p_sparse > PSI_ALERT and p_low > PSI_ALERT, (p_sparse, p_low))
    c.check("cust_prior_trips vượt 0,25 ở cả ba tuần (bộ đếm tích lũy tự trôi)", all(weekly[k][0]["cust_prior_trips"] > PSI_ALERT for k in weekly))
    c.check("ngoài hai đặc trưng lịch sử khách, mọi đặc trưng dưới 0,10 ở cả ba tuần", all(max(v for k, v in weekly[k_][0].items() if k not in HIST_FEATURES) < PSI_WARN for k_ in weekly))
    c.check("cust_promo_share_30d lệch nhẹ: giữa 0,05 và 0,25 ở cả ba tuần", all(0.05 < weekly[k_][0]["cust_promo_share_30d"] < PSI_ALERT for k_ in weekly))
    c.check("cust_prior_trips thuộc nhóm đặc trưng tác động mạnh nhất (hạng 3 trở lên theo mean|SHAP|) dù gain thấp hơn nhiều", rank <= 3 and imp["cust_prior_trips"] > 2 * gain["cust_prior_trips"], (rank, imp["cust_prior_trips"], gain["cust_prior_trips"]))
    c.check("tỉ lệ chuyến đầu tiên của khách giảm hơn 3 lần từ tham chiếu đến tuần cuối", first[0] > 3 * first[3], first)
    c.check("tạo tác (a): tỉ lệ NEWUSER50 của chuyến sạch ở tuần cuối thấp hơn nửa tham chiếu", nu[3] < 0.5 * nu[0], nu)
    c.check("tạo tác (b): chuyến sạch có cust_promo_share_30d > 0 nhiều hơn hẳn chuyến gian lận", pos_clean > 2 * pos_fraud, (pos_clean, pos_fraud))
    c.check("kịch bản GPS: PSI của start_pickup_gap_km vượt 0,25, ở nguồn cũ dưới 0,1", out["nguồn đổi (x3)"]["psi"] > PSI_ALERT and out["nguồn cũ"]["psi"] < PSI_WARN, out)
    c.check("kịch bản GPS: số cờ của v1 tăng ít nhất 1,3 lần", out["nguồn đổi (x3)"]["flags"] > 1.3 * out["nguồn cũ"]["flags"], out)
    c.check("kịch bản GPS: v2 (học trước lúc nguồn đổi) đắt hơn ít nhất 1,3 lần", out["nguồn đổi (x3)"]["cost2"] > 1.3 * out["nguồn cũ"]["cost2"], out)
    c.check("kịch bản GPS: vx (học trên dữ liệu có lỗi) rẻ hơn v1 nguồn đổi ít nhất 1,3 lần", bs["vx nguồn đổi"] * 1.3 < bs["v1 nguồn đổi"], bs)
    return {"gs": imp, "weekly": weekly, "vals_i": vals_i}


# ---- [7] Lịch huấn luyện lại -----------------------------------------------------------------------


def step_policy(dr: dict, sh: dict, c: Checker) -> RetrainPolicy:
    say("\n[7] Chính sách huấn luyện lại: tín hiệu -> một hành động")
    pol = RetrainPolicy()
    say(f"chính sách (giả định của lab): tuổi tối đa {pol.max_age_days:.0f} ngày; PSI >= {PSI_ALERT} ở bất kỳ đặc trưng nào là điều tra, từ {PSI_WARN} là theo dõi (độ quan trọng mean|SHAP| chỉ để xếp thứ tự); "
        f"PR-AUC tụt quá {pol.max_pr_auc_drop}; tỉ lệ cờ ngoài {pol.flag_rate_band}; rollback nếu vừa promote trong {pol.rollback_window_days:.0f} ngày")
    gs = dr["gs"]
    nat = dr["weekly"][(47, 54)][0]
    r = sh["res"]
    v3_ratio = r["v3"]["rate_ec"] / r["v1"]["rate_ec"]
    scenarios = [
        ("ngày 54, tuần bình thường (v1 24 ngày tuổi)", dict(age_days=24, psi_by_feature=nat, pr_auc_drop=0.0, flag_rate_ratio=1.0), "investigate"),
        ("ngày 60, đúng hạn lịch (v1 30 ngày tuổi)", dict(age_days=30, psi_by_feature=nat, pr_auc_drop=0.0, flag_rate_ratio=1.0), "retrain"),
        ("ngày 55, nguồn GPS đổi (kịch bản mục 6)", dict(age_days=25, psi_by_feature=dr["vals_i"], pr_auc_drop=None, flag_rate_ratio=None), "investigate"),
        ("v3 vừa promote 3 ngày trước", dict(age_days=10, psi_by_feature={}, pr_auc_drop=None, flag_rate_ratio=v3_ratio, days_since_promote=3), "rollback"),
        ("v1 mất 0,05 PR-AUC trên nhãn đã chín", dict(age_days=20, psi_by_feature={}, pr_auc_drop=0.05, flag_rate_ratio=1.0), "retrain"),
        ("chỉ cust_promo_share_30d lệch nhẹ (PSI của tuần 47 đến 54)", dict(age_days=10, psi_by_feature={"cust_promo_share_30d": nat["cust_promo_share_30d"]}, pr_auc_drop=0.0, flag_rate_ratio=1.0), "monitor"),
    ]
    got = []
    for name, kw, expected in scenarios:
        dec = retrain_decision(pol, impact_share=gs, **kw)
        got.append(dec.action)
        say(f"  {name}: {dec.action}")
        for reason in dec.reasons:
            say(f"      - {reason}")
    for (name, _, expected), action in zip(scenarios, got):
        c.check(f"chính sách: {name} -> {expected}", action == expected, action)
    c.check("đặc trưng bị lỗi nguồn (start_pickup_gap_km) có tác động mean|SHAP| thấp hơn cust_prior_trips nhiều lần mà vẫn bị điều tra", gs["start_pickup_gap_km"] * 2 < gs["cust_prior_trips"], (gs["start_pickup_gap_km"], gs["cust_prior_trips"]))
    return pol


# ---- [8] Promote, promote nhầm và rollback ---------------------------------------------------------------


def step_rollback(d: dict, reg: Registry, versions: dict, models: dict, gs: dict, pol: RetrainPolicy, c: Checker) -> None:
    say("\n[8] Promote v2, rồi promote nhầm v3 (kịch bản), phát hiện và rollback")
    w = d["window"]
    v2, v3 = versions["v2"], versions["v3"]
    p2_before = models["v2"].predict_proba(w)
    reg.promote(v2, actor="oncall-ml", reason="holdout 40-50 không kém v1, shadow 50-60 cùng thứ bậc, A/B chưa phân biệt được")
    say(f"promote v2 (qua cổng) -> current = {'v2' if reg.current() == v2 else reg.current()}")
    bad = expect_error(lambda: reg.promote(v3, actor="dev-in-a-hurry", reason="muốn dùng dữ liệu mới nhất"), "chồng lên")
    say(f"promote v3 không ép: {bad}")
    reg.promote(v3, actor="dev-in-a-hurry", reason="muốn dùng dữ liệu mới nhất", force=True)
    say(f"promote v3 với force=True và lý do -> current = {'v3' if reg.current() == v3 else reg.current()}")

    day_w = paid_day(w)
    before, after = day_w < 57, day_w >= 57
    serving = reg.load_current()
    s_after, s_before = w.filter(pl.Series(after)), w.filter(pl.Series(before))
    a_after, a_before = (x["amount_vnd"].to_numpy().astype(float) for x in (s_after, s_before))
    p_after = serving.predict_proba(s_after)
    p_before = models["v2"].predict_proba(s_before)
    p_same = models["v2"].predict_proba(s_after)  # champion cũ giữ chạy shadow cùng ba ngày
    rate_before = float(expected_cost_flag(p_before, a_before).mean())
    rate_after = float(expected_cost_flag(p_after, a_after).mean())
    rate_same = float(expected_cost_flag(p_same, a_after).mean())
    ratio = rate_after / rate_same
    say(f"tỉ lệ cờ ngày 57-60: v3 phục vụ {1000 * rate_after:.1f}/1000; v2 chấm cùng ba ngày đó (shadow) {1000 * rate_same:.1f}/1000; tỉ lệ {ratio:.2f}")
    say(f"so với v2 ngày 50-57 ({1000 * rate_before:.1f}/1000) tỉ lệ là {rate_after / rate_before:.2f}: mức chuẩn khác ngày lẫn thêm biến động của chính dữ liệu")
    dec = retrain_decision(pol, age_days=10, psi_by_feature={}, impact_share=gs, pr_auc_drop=None, flag_rate_ratio=ratio, days_since_promote=3)
    say(f"chính sách: {dec.action} ({dec.reasons[0]})")
    t0 = time.perf_counter()
    target = reg.rollback(actor="oncall-ml", reason="tỉ lệ cờ giảm hơn một nửa sau promote v3; nghi nhãn chưa chín trong huấn luyện")
    now = reg.load_current()
    t_roll = time.perf_counter() - t0
    p2_after = now.predict_proba(w)
    say(f"rollback -> current = {'v2' if target == v2 else target}; đổi con trỏ và nạp lại mất {1000 * t_roll:.1f} ms; điểm của bản đang phục vụ giống hệt điểm v2 trước đó: {bool(np.array_equal(p2_after, p2_before))}")
    say(f"v3 vẫn còn trong registry để điều tra: {v3 in reg.versions()}; số bản: {len(reg.versions())}")
    say("lịch sử (history.jsonl, rút gọn):")
    for e in reg.history():
        tail = f" <- {e['previous'][-8:] if e.get('previous') else '-'}" if e["event"] == "promoted" else (f" -> {e['to'][-8:]}" if e["event"] == "rolled_back" else "")
        forced = " [ÉP QUA CỔNG]" if e.get("forced_over") else ""
        say(f"  {e['event']:11s} ...{e['version'][-8:]}{tail} bởi {e['actor']}{forced}" + (f": {e['reason']}" if e.get("reason") else ""))
    hist = reg.history()
    c.check("v3 chỉ qua cổng khi force=True, và lịch sử ghi lại việc ép", any(e["event"] == "promoted" and e["version"] == v3 and e["forced_over"] for e in hist))
    c.check("tỉ lệ cờ sau khi v3 phục vụ ngoài dải của chính sách", not pol.flag_rate_band[0] <= ratio <= pol.flag_rate_band[1], ratio)
    c.check("chính sách đề nghị rollback", dec.action == "rollback", dec)
    c.check("rollback đưa current về v2, không xóa v3", target == v2 and reg.current() == v2 and v3 in reg.versions())
    c.check("sau rollback điểm của bản phục vụ giống hệt v2 (bit)", bool(np.array_equal(p2_after, p2_before)))
    c.check("lịch sử có đủ 3 đăng ký, 3 promote, 1 rollback theo thứ tự", [e["event"] for e in hist] == ["registered"] * 3 + ["promoted", "promoted", "promoted", "rolled_back"], [e["event"] for e in hist])
    c.check("con trỏ current.json khớp ngăn xếp phát lại từ lịch sử", reg.current() == reg._stack()[-1])


# ---- [9] Lặp lại với ba seed -------------------------------------------------------------------------


def step_seeds(c: Checker) -> None:
    say(f"\n[9] Lặp lại kết luận chính với seed {', '.join(map(str, SEEDS))} (dữ liệu sinh lại hoàn toàn; không registry, không A/B)")
    rows, drift_rows = [], []
    for seed in SEEDS:
        t0 = time.perf_counter()
        table = load_table(seed=seed)
        day = paid_day(table)
        w = table.filter(pl.Series(day >= WINDOW_FROM))
        y = w["is_fraud"].to_numpy().astype(int)
        amount = w["amount_vnd"].to_numpy().astype(float)
        ref_df = table.filter(pl.Series(day < 30))
        ref = feat_arrays(ref_df)
        trained, scoring = {}, {}
        for spec in (V1, V2, V3, V3F):
            cal, meta = train(table, spec)
            trained[spec.name] = (cal, float(meta["decision"]["tuned_threshold"]))
            scoring[spec.name] = in_memory(cal, meta, f"{spec.name}-seed{seed}")
        thr1 = trained["v1"][1]
        ps = {}
        for name, (cal, thr_own) in trained.items():
            p = cal.predict_proba(w.select(FEATURES))[:, 1]
            ps[name] = p
            fe, f1, fo = expected_cost_flag(p, amount), p >= thr1, p >= thr_own
            rows.append(dict(seed=seed, name=name, pr=pr(y, p), re=1000 * fe.mean(), ce=float(trip_costs(fe, y, amount).sum()),
                             r1=1000 * f1.mean(), c1=float(trip_costs(f1, y, amount).sum()), ro=1000 * fo.mean(), co=float(trip_costs(fo, y, amount).sum())))
        vals, _ = psi_summary(ref, w)
        wi = w.with_columns(pl.col("start_pickup_gap_km") * 3.0)
        pi = trained["v1"][0].predict_proba(wi.select(FEATURES))[:, 1]
        valsi, _ = psi_summary(ref, wi)
        imp = impact_share(scoring["v1"], ref_df)
        top_imp = sorted(imp, key=imp.get, reverse=True)
        bs = broken_source_costs(table, day, scoring["v1"], scoring["v2"], amount, y, w)
        fe1, fei = expected_cost_flag(ps["v1"], amount), expected_cost_flag(pi, amount)
        drift_rows.append(dict(seed=seed, prior=vals["cust_prior_trips"], other=max(v for k, v in vals.items() if k not in HIST_FEATURES),
                               gps=valsi["start_pickup_gap_km"], flags=float(fei.sum() / max(1, fe1.sum())),
                               cost=float(trip_costs(fei, y, amount).sum() / trip_costs(fe1, y, amount).sum()),
                               psi2=psi(ps["v1"], ps["v2"]), psi3=psi(ps["v1"], ps["v3"]), imp=imp["cust_prior_trips"], rank=top_imp.index("cust_prior_trips") + 1, bs=bs))
        say(f"  seed {seed}: xong sau {time.perf_counter() - t0:.0f} giây (load average 1/5/15 phút: {load_avg()})")
    say("\nquy tắc chi phí kỳ vọng (cờ/1000 và chi phí triệu VND), rồi ngưỡng của v1, rồi ngưỡng riêng của từng bản (v3f: như v3 nhưng nhãn cuối):")
    say(f"{'seed':>5s}{'bản':>4s}{'PR-AUC':>8s}{'cờ/1000':>9s}{'chi phí':>9s}{'ngưỡng v1':>11s}{'cờ/1000':>9s}{'chi phí':>9s}{'cờ/1000':>9s}{'chi phí':>9s}")
    for r in rows:
        say(f"{r['seed']:5d}{r['name']:>4s}{r['pr']:8.4f}{r['re']:9.1f}{r['ce'] / 1e6:9.2f}{'':>11s}{r['r1']:9.1f}{r['c1'] / 1e6:9.2f}{r['ro']:9.1f}{r['co'] / 1e6:9.2f}")
    by = {(r["seed"], r["name"]): r for r in rows}
    say(f"\n{'seed':>5s}{'PSI cust_prior_trips':>22s}{'hạng mean|SHAP|':>17s}{'tác động':>10s}{'PSI lớn nhất còn lại':>22s}{'PSI GPS x3':>12s}{'cờ v1 x lần':>13s}{'chi phí v1 x lần':>18s}")
    for r in drift_rows:
        say(f"{r['seed']:5d}{r['prior']:22.3f}{r['rank']:17d}{r['imp']:10.1%}{r['other']:22.3f}{r['gps']:12.3f}{r['flags']:13.2f}{r['cost']:18.2f}")
    say("\nPSI của phân phối điểm so với v1, và huấn luyện lại sau khi nguồn đổi (chi phí triệu VND, quy tắc chi phí kỳ vọng):")
    say(f"{'seed':>5s}{'PSI điểm v2':>13s}{'PSI điểm v3':>13s}{'v1 nguồn cũ':>13s}{'v1 nguồn đổi':>14s}{'v2 nguồn đổi':>14s}{'vx nguồn đổi':>14s}")
    for r in drift_rows:
        b = r["bs"]
        say(f"{r['seed']:5d}{r['psi2']:13.3f}{r['psi3']:13.3f}{b['v1 nguồn cũ'] / 1e6:13.2f}{b['v1 nguồn đổi'] / 1e6:14.2f}{b['v2 nguồn đổi'] / 1e6:14.2f}{b['vx nguồn đổi'] / 1e6:14.2f}")
    for s in SEEDS:
        v1, v3, v3f = by[(s, "v1")], by[(s, "v3")], by[(s, "v3f")]
        c.check(f"seed {s}: v3 gắn cờ ít hơn 0,75 lần v1 theo quy tắc chi phí kỳ vọng", v3["re"] < 0.75 * v1["re"], (v3["re"], v1["re"]))
        c.check(f"seed {s}: v3 đắt hơn v1 ít nhất 1,15 lần theo quy tắc chi phí kỳ vọng, ngưỡng của v1 và ngưỡng riêng", v3["ce"] > 1.15 * v1["ce"] and v3["c1"] > 1.15 * v1["c1"] and v3["co"] > 1.15 * v1["co"], (v3, v1))
        c.check(f"seed {s}: PR-AUC của v3 không thấp hơn v1 quá 0,02", v3["pr"] > v1["pr"] - 0.02, (v3["pr"], v1["pr"]))
        c.check(f"seed {s}: đổi đúng một biến (nhãn cuối thay vì nhãn biết tại ngày 50) đưa số cờ của v3f về 0,8 đến 1,3 lần v1 và rẻ hơn v3 ít nhất 1,15 lần",
                0.8 * v1["re"] < v3f["re"] < 1.3 * v1["re"] and v3["ce"] > 1.15 * v3f["ce"], (v3f, v3, v1))
    for r in drift_rows:
        c.check(f"seed {r['seed']}: cust_prior_trips vượt 0,25, đặc trưng còn lại dưới 0,10, GPS x3 vượt 0,25", r["prior"] > PSI_ALERT and r["other"] < PSI_WARN and r["gps"] > PSI_ALERT, r)
        c.check(f"seed {r['seed']}: GPS x3 làm số cờ của v1 tăng hơn 1,3 lần và chi phí tăng hơn 1,3 lần", r["flags"] > 1.3 and r["cost"] > 1.3, r)
        c.check(f"seed {r['seed']}: cust_prior_trips thuộc ba đặc trưng tác động mạnh nhất (mean|SHAP|)", r["rank"] <= 3, r["rank"])
        b = r["bs"]
        c.check(f"seed {r['seed']}: vx (học trên dữ liệu có lỗi) rẻ hơn v1 nguồn đổi ít nhất 1,3 lần, v2 (học trước lúc đổi) thì không", b["vx nguồn đổi"] * 1.3 < b["v1 nguồn đổi"] and b["v2 nguồn đổi"] > 1.3 * b["v1 nguồn cũ"], b)
    better = [by[(s, "v2")]["pr"] > by[(s, "v1")]["pr"] for s in SEEDS]
    say(f"\nv2 (huấn luyện lại, nhiều dữ liệu hơn) có PR-AUC cao hơn v1 ở {sum(better)}/{len(SEEDS)} seed: " + ", ".join(f"{by[(s, 'v2')]['pr'] - by[(s, 'v1')]['pr']:+.4f}" for s in SEEDS))
    diffs = [by[(s, "v2")]["ce"] - by[(s, "v1")]["ce"] for s in SEEDS]
    say("chi phí của v2 trừ v1 theo quy tắc chi phí kỳ vọng (triệu VND): " + ", ".join(f"{x / 1e6:+.2f}" for x in diffs))


def main() -> int:
    verify = "--verify" in sys.argv
    c = Checker()
    d = step_data(c)
    reg, versions, models = step_train(d, c)
    step_registry(reg, versions, c)
    sh = step_shadow(d, reg, versions, models, c)
    step_ab(d, sh, c)
    dr = step_drift(d, models, sh, c)
    pol = step_policy(dr, sh, c)
    step_rollback(d, reg, versions, models, dr["gs"], pol, c)
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
