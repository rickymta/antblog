"""Bài 49: embedding khuôn mặt, cosine similarity, ngưỡng, FAR/FRR và ROC trên Olivetti.

Chạy:   uv run --project <lab> <lab>/main.py [--with-arcface]
Verify: uv run --project <lab> <lab>/main.py --verify [--with-arcface]

Mặc định chỉ dùng SFace (OpenCV Zoo, Apache 2.0) và YuNet (MIT). Cờ --with-arcface thêm ArcFace R50 của InsightFace
làm hàng so sánh: mô hình đó chỉ được dùng cho nghiên cứu phi thương mại, lab in cảnh báo trước khi tải gói 288 MB.
Cần mạng ở lần chạy đầu: mô hình vào ~/.cache/toinoi-python/models và bộ Olivetti qua sklearn. Không commit ảnh
người hay mô hình; embedding sinh ra (là dữ liệu sinh trắc học) nằm trong out/ và đã gitignore.
Điểm so khớp là đầu vào cho người duyệt hồ sơ KYC, không phải quyết định xác minh.
"""

import os
import platform
import statistics
import sys
import time
from pathlib import Path

import cv2
import numpy as np
import onnxruntime as ort
from sklearn.datasets import fetch_olivetti_faces

from degrade import HEAVY, LIGHT, Degradation, degrade
from embedders import SFace
from facepipe import FacePreparer
from metrics import (auc, best_threshold_by_cost, cosine_matrix, equal_error_rate, expected_cost, far_frr,
                     split_scores, suggest, threshold_at_far, threshold_at_frr)
from models import ARCFACE_R50, CACHE_DIR, SFACE, YUNET, ensure_model

OUT = Path(__file__).parent / "out"
OLIVETTI_URL = "https://ndownloader.figshare.com/files/5976027"  # sklearn.datasets._olivetti_faces.FACES
BATCH = 32
# Giả định chi phí để minh họa cơ chế; vận hành thật phải thay bằng số của chính họ.
C_FA, C_FR, P_IMPOSTOR = 100.0, 1.0, 0.02
TARGET_FAR = 1e-3
RANDOM_SPLITS = 300
ARCFACE_WARNING = (
    "CẢNH BÁO GIẤY PHÉP: mã nguồn InsightFace theo MIT, nhưng README của họ ghi các mô hình huấn luyện sẵn, trong đó "
    "gói buffalo_l (file w600k_r50.onnx), chỉ dùng cho nghiên cứu phi thương mại. Cờ --with-arcface tải gói này "
    "trực tiếp từ trang phát hành của họ về máy bạn để so thang điểm; nếu bạn chạy trong dự án có thu phí, hãy hỏi "
    "người phụ trách pháp lý, hoặc bỏ cờ này và chỉ chạy SFace."
)


def embed_found(embedder, crops: list[np.ndarray | None]) -> np.ndarray:
    """Ma trận (N, dim); ảnh không có crop (không lấy được mặt) để hàng NaN, không bị nhúng."""
    found = [k for k, crop in enumerate(crops) if crop is not None]
    out = np.full((len(crops), embedder.dim), np.nan, dtype=np.float32)
    for start in range(0, len(found), BATCH):
        chunk = found[start:start + BATCH]
        out[chunk] = embedder.embed([crops[k] for k in chunk])
    return out


def summarize(scores: np.ndarray, labels: np.ndarray, *, symmetric: bool, miss_rate: float = 0.0) -> dict[str, float]:
    genuine, impostor = split_scores(scores, labels, labels, symmetric=symmetric, allow_missing=True)
    eer, eer_t = equal_error_rate(genuine, impostor)
    t3, t2 = threshold_at_far(impostor, TARGET_FAR), threshold_at_far(impostor, 1e-2)
    far3, frr3 = far_frr(genuine, impostor, np.array([t3]))
    _, frr2 = far_frr(genuine, impostor, np.array([t2]))
    return {"n_g": len(genuine), "n_i": len(impostor), "auc": auc(genuine, impostor), "eer": eer, "eer_t": eer_t,
            "t_far": t3, "far": float(far3[0]), "frr": float(frr3[0]), "tar2": 1.0 - float(frr2[0]),
            "tar3": 1.0 - float(frr3[0]), "miss": miss_rate,
            # FRR của cả hệ thống: ảnh không lấy được mặt cũng không qua được và phải chuyển người duyệt.
            "frr_sys": 1.0 - (1.0 - miss_rate) * (1.0 - float(frr3[0]))}


def quantile_row(label: str, group: str, x: np.ndarray) -> str:
    q = np.quantile(x, [0, 0.05, 0.5, 0.95, 1.0])
    return f"  {label:<12}{group:<10}{len(x):>8,}" + "".join(f"{v:>8.3f}" for v in q)


def impostor_structure(scores: np.ndarray, labels: np.ndarray, t: float) -> tuple[int, int]:
    """(số cặp khác người có điểm >= t, số CẶP NGƯỜI khác nhau mà các cặp đó thuộc về) trên ma trận đối xứng."""
    rows, cols = np.triu_indices(len(labels), k=1)
    a, b = labels[rows], labels[cols]
    chosen = (a != b) & (scores[rows, cols].astype(np.float64) >= t)  # float64: ngưỡng là float64, điểm là float32
    return int(chosen.sum()), len({tuple(sorted(pair)) for pair in zip(a[chosen].tolist(), b[chosen].tolist())})


def _split_once(scores: np.ndarray, labels: np.ndarray, people_a: np.ndarray, people_b: np.ndarray) -> dict[str, float]:
    """Chọn ngưỡng ở FAR <= 1e-3 trên nhóm người A, đo trên nhóm người B (hai nhóm không chung người nào)."""
    rows = {}
    for key, people in (("a", people_a), ("b", people_b)):
        idx = np.flatnonzero(np.isin(labels, people))
        rows[key] = split_scores(scores[np.ix_(idx, idx)], labels[idx], labels[idx], symmetric=False, allow_missing=True)
    t = threshold_at_far(rows["a"][1], TARGET_FAR)
    far_a, frr_a = far_frr(*rows["a"], np.array([t]))
    far_b, frr_b = far_frr(*rows["b"], np.array([t]))
    return {"t": t, "far_a": float(far_a[0]), "frr_a": float(frr_a[0]), "far_b": float(far_b[0]),
            "frr_b": float(frr_b[0]), "n_i_b": len(rows["b"][1]), "disjoint": not (set(people_a) & set(people_b))}


def split_study(scores: np.ndarray, labels: np.ndarray, draws: int = RANDOM_SPLITS, seed: int = 0) -> dict[str, object]:
    """Hai chiều của cách chia 20/20 theo chỉ số, cộng nhiều cách chia ngẫu nhiên rời người: không kết luận từ một lần chia."""
    first, second = np.arange(20), np.arange(20, 40)
    forward, backward = _split_once(scores, labels, first, second), _split_once(scores, labels, second, first)
    rng = np.random.default_rng(seed)
    runs = [_split_once(scores, labels, *np.split(rng.permutation(40), 2)) for _ in range(draws)]
    far_b = np.array([r["far_b"] for r in runs])
    frr_b = np.array([r["frr_b"] for r in runs])
    return {"forward": forward, "backward": backward, "far_q": np.quantile(far_b, [0.05, 0.5, 0.95]),
            "frr_q": np.quantile(frr_b, [0.05, 0.5, 0.95]), "share_exceed": float((far_b > TARGET_FAR).mean()),
            "draws": draws, "all_disjoint": all(r["disjoint"] for r in runs)}


def bootstrap(scores: np.ndarray, labels: np.ndarray, fixed_t: float, draws: int = 200, seed: int = 0) -> dict[str, float]:
    """Bootstrap theo NGƯỜI: lấy lại 40 người có hoàn lại. Mỗi lần ghi (a) FRR và EER khi CHỌN LẠI ngưỡng ở FAR <= 1e-3
    (FAR khi đó luôn ở mục tiêu nên không có thông tin), và (b) FAR, FRR tại ngưỡng CỐ ĐỊNH fixed_t.

    Một người bị bốc hai lần thì hai bản sao là hai nhóm; cặp giữa hai bản sao của cùng một người bị loại
    (thực ra là cùng người, không tính là impostor). Hàng NaN (không lấy được mặt) bị bỏ khỏi cặp.
    """
    rng = np.random.default_rng(seed)
    frrs, eers, fars_fixed, frrs_fixed = [], [], [], []
    for _ in range(draws):
        people = rng.integers(0, 40, size=40)
        idx = np.concatenate([np.flatnonzero(labels == p) for p in people])
        copy = np.repeat(np.arange(40), 10)
        origin = labels[idx]
        sub = scores[np.ix_(idx, idx)]
        valid = ~np.isnan(sub)
        same_copy = copy[:, None] == copy[None, :]
        same_origin = origin[:, None] == origin[None, :]
        eye = np.eye(len(idx), dtype=bool)
        genuine = sub[same_copy & ~eye & valid].astype(np.float64)
        impostor = sub[~same_origin & valid].astype(np.float64)
        t = threshold_at_far(impostor, TARGET_FAR)
        frrs.append(float(far_frr(genuine, impostor, np.array([t]))[1][0]))
        eers.append(equal_error_rate(genuine, impostor)[0])
        far_f, frr_f = far_frr(genuine, impostor, np.array([fixed_t]))
        fars_fixed.append(float(far_f[0]))
        frrs_fixed.append(float(frr_f[0]))
    q = lambda xs: [float(v) for v in np.quantile(xs, [0.025, 0.5, 0.975])]  # noqa: E731
    return {"draws": draws, "frr": q(frrs), "eer": q(eers), "far_fixed": q(fars_fixed), "frr_fixed": q(frrs_fixed)}


def demo(with_arcface: bool = False) -> dict[str, object]:
    OUT.mkdir(exist_ok=True)
    facts: dict[str, object] = {}

    print("[1] Môi trường")
    load = os.getloadavg()[0]
    print(f"Python {platform.python_version()}, NumPy {np.__version__}, onnxruntime {ort.__version__}, "
          f"OpenCV {cv2.__version__}")
    print(f"providers onnxruntime có sẵn: {ort.get_available_providers()}; ArcFace (nếu bật) dùng CPUExecutionProvider")
    print(f"load average 1 phút lúc chạy: {load:.2f} (số lõi: {os.cpu_count()})")
    facts["load"] = load

    print(f"\n[2] Mô hình và dữ liệu (cache mô hình: {CACHE_DIR})")
    models = [YUNET, SFACE]
    if with_arcface:
        print(f"  {ARCFACE_WARNING}")
        models.append(ARCFACE_R50)
    for model in models:
        path = ensure_model(model)
        print(f"  {model.name:<22}{path.stat().st_size:>13,} byte  {model.url}"
              + (f"  (gói {model.zip_size:,} byte)" if model.zip_size else ""))
    data = fetch_olivetti_faces()
    images = np.rint(data.images * 255).astype(np.uint8)
    labels = data.target.astype(np.int64)
    print(f"  Olivetti: {images.shape[0]} ảnh {images.shape[1]}x{images.shape[2]} xám, {len(set(labels.tolist()))} người x "
          f"{int(np.bincount(labels).min())} ảnh; nguồn {OLIVETTI_URL}")
    print("  Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn.")
    cache_file = Path.home() / "scikit_learn_data" / "olivetti_py3.pkz"
    print(f"  (sklearn tải olivettifaces.mat, khoảng 4,26 MB, lưu lại {cache_file.name}: {cache_file.stat().st_size:,} byte "
          "trong ~/scikit_learn_data)")
    facts["models_ok"] = True

    embedders: list = [SFace()]
    if with_arcface:
        from embedders import ArcFaceR50  # chỉ nạp khi bật cờ

        embedders.append(ArcFaceR50())

    print("\n[3] Căn chỉnh khuôn mặt (YuNet 5 landmark -> mẫu ArcFace 112x112); ảnh không lấy được mặt KHÔNG được nhúng")
    prep = FacePreparer()
    levels: dict[str, Degradation | None] = {"sạch": None, "nhẹ": LIGHT, "nặng": HEAVY}
    crops: dict[str, list[np.ndarray | None]] = {}
    miss: dict[str, int] = {}
    for name, level in levels.items():
        before = prep.detect_failures
        t0 = time.perf_counter()
        crops[name] = [prep.aligned(img if level is None else degrade(img, level, seed=k)) for k, img in enumerate(images)]
        ms = (time.perf_counter() - t0) / len(images) * 1000
        miss[name] = prep.detect_failures - before
        print(f"  mức {name:<5}: YuNet không lấy được mặt ở {miss[name]}/{len(images)} ảnh ({miss[name] / len(images):.1%}); "
              f"những ảnh này chuyển needs_review và không có embedding; {ms:.1f} ms/ảnh (gồm dò mặt)")
    facts["miss"] = miss
    if miss["sạch"]:
        raise RuntimeError("ảnh tham chiếu sạch phải lấy được mặt")
    resized = [prep.resize_only(img) for img in images]
    geometry = [g for g in (prep.geometry(img) for img in images) if g is not None]
    eye_resize = statistics.median(g[0] for g in geometry)
    outside = statistics.median(g[1] for g in geometry)
    template_eye = float(np.linalg.norm(np.array([73.5318, 51.5014]) - np.array([38.2946, 51.6963])))
    print(f"  «chỉ resize» 64 -> 112 để hai mắt cách {eye_resize:.1f} px trên 112, mẫu ArcFace chỉ {template_eye:.1f} px "
          f"(mặt to hơn mẫu {eye_resize / template_eye:.2f} lần); crop căn chỉnh có trung vị {outside:.1%} điểm ảnh nằm ngoài ảnh gốc (viền lặp)")
    facts["eye_resize"], facts["template_eye"], facts["outside"] = eye_resize, template_eye, outside

    print("\n[4] Embedding (vector chuẩn hóa L2)")
    clean = {e.name: embed_found(e, crops["sạch"]) for e in embedders}
    plain = {e.name: embed_found(e, resized) for e in embedders}
    one = crops["sạch"][0]
    assert one is not None
    for e in embedders:
        e.embed([one])  # khởi động
        times = []
        for _ in range(15):
            t0 = time.perf_counter()
            e.embed([one])
            times.append((time.perf_counter() - t0) * 1000)
        norms = np.linalg.norm(clean[e.name], axis=1)
        print(f"  {e.name:<12} shape {clean[e.name].shape}, dtype {clean[e.name].dtype}, chuẩn L2 trong "
              f"[{norms.min():.6f}, {norms.max():.6f}], một ảnh trung vị {statistics.median(times):.1f} ms "
              f"(load {os.getloadavg()[0]:.1f})")
        np.save(OUT / f"embeddings_{e.name}.npy", clean[e.name])
    np.save(OUT / "labels.npy", labels)
    print(f"  đã lưu {', '.join(f'embeddings_{e.name}.npy' for e in embedders)}, labels.npy vào {OUT.name}/")
    facts["clean"], facts["labels"] = clean, labels
    facts["norm_dev"] = max(float(np.abs(np.linalg.norm(v, axis=1) - 1).max()) for v in clean.values())
    again = embedders[0].embed([one])[0]
    facts["repeat_cos"] = float(again @ clean[embedders[0].name][0])

    # --- [5] Phân bố genuine/impostor, ROC, so sánh có/không căn chỉnh --------------------------
    print("\n[5] Phân bố điểm cosine, Olivetti sạch so với sạch (mọi cặp không thứ tự)")
    print(f"  {'mô hình':<12}{'nhóm':<10}{'số cặp':>8}{'min':>8}{'p05':>8}{'p50':>8}{'p95':>8}{'max':>8}")
    clean_sum: dict[str, dict[str, float]] = {}
    plain_sum: dict[str, dict[str, float]] = {}
    clean_scores: dict[str, np.ndarray] = {}
    structure: dict[str, tuple[int, int]] = {}
    fixed: dict[str, dict[float, tuple[float, float]]] = {}
    for e in embedders:
        s = cosine_matrix(clean[e.name], clean[e.name])
        clean_scores[e.name] = s
        g, i = split_scores(s, labels, labels, symmetric=True)
        print(quantile_row(e.name, "genuine", g))
        print(quantile_row(e.name, "impostor", i))
        clean_sum[e.name] = summarize(s, labels, symmetric=True)
        plain_sum[e.name] = summarize(cosine_matrix(plain[e.name], plain[e.name]), labels, symmetric=True)
        structure[e.name] = impostor_structure(s, labels, clean_sum[e.name]["t_far"])
        fixed[e.name] = {}
        for t in (0.3, 0.5):
            far, frr = far_frr(g, i, np.array([t]))
            fixed[e.name][t] = (float(far[0]), float(frr[0]))
    facts["clean_sum"], facts["plain_sum"], facts["clean_scores"] = clean_sum, plain_sum, clean_scores
    facts["structure"], facts["fixed"] = structure, fixed
    print("\n  ROC: TAR = 1 - FRR theo FAR. Ngưỡng ở FAR <= 1e-3 tính trên chính các cặp khác người này")
    print(f"  {'mô hình':<12}{'đầu vào':<12}{'AUC':>9}{'EER':>8}{'TAR@FAR 1%':>12}{'TAR@FAR 0.1%':>14}{'ngưỡng':>9}{'FAR':>10}")
    for e in embedders:
        for label, summary in (("căn chỉnh", clean_sum[e.name]), ("chỉ resize", plain_sum[e.name])):
            print(f"  {e.name:<12}{label:<12}{summary['auc']:>9.5f}{summary['eer']:>8.4f}{summary['tar2']:>12.4f}"
                  f"{summary['tar3']:>14.4f}{summary['t_far']:>9.3f}{summary['far']:>10.5f}")
    print("\n  Ngưỡng cố định thường thấy trong ví dụ trên mạng (bộ sạch):")
    for e in embedders:
        print("  " + f"{e.name:<12}" + "; ".join(
            f"ngưỡng {t}: FAR {far:.4f}, FRR {frr:.4f}" for t, (far, frr) in fixed[e.name].items()))
    print("\n  Các cặp khác người vượt ngưỡng FAR <= 1e-3 không độc lập: chúng dồn vào ít cặp người")
    for e in embedders:
        pairs, groups = structure[e.name]
        print(f"  {e.name:<12}{pairs} cặp ảnh vượt ngưỡng {clean_sum[e.name]['t_far']:.3f}, thuộc {groups}/780 cặp người")

    # --- [6] Ba kịch bản: sạch, thẻ nhẹ, thẻ nặng ----------------------------------------------
    print("\n[6] «Thẻ vs selfie»: probe xuống cấp (mờ, tối, nhiễu, JPEG), reference sạch; mọi cặp có thứ tự i != j")
    for level in (LIGHT, HEAVY):
        print(f"  mức {level.name}: Gauss sigma {level.sigma}, gamma {level.gamma}, nhiễu {level.noise}, JPEG {level.jpeg_quality}")
    scen_sum: dict[str, dict[str, dict[str, float]]] = {}
    scen_scores: dict[str, dict[str, np.ndarray]] = {}
    for e in embedders:
        scen_sum[e.name], scen_scores[e.name] = {}, {}
        for name in levels:
            probe = clean[e.name] if name == "sạch" else embed_found(e, crops[name])
            s = cosine_matrix(probe, clean[e.name])  # hàng NaN = probe không lấy được mặt
            scen_scores[e.name][name] = s
            scen_sum[e.name][name] = summarize(s, labels, symmetric=False, miss_rate=miss[name] / len(images))
    facts["scen_sum"], facts["scen_scores"] = scen_sum, scen_scores
    print(f"  {'mô hình':<12}{'kịch bản':<10}{'hụt mặt':>9}{'genuine':>9}{'impostor':>10}{'AUC':>9}{'EER':>8}"
          f"{'TAR@1%':>8}{'TAR@0.1%':>10}{'ngưỡng':>8}{'FRR hệ thống':>14}")
    for e in embedders:
        for name in levels:
            r = scen_sum[e.name][name]
            print(f"  {e.name:<12}{name:<10}{miss[name]:>9}{r['n_g']:>9,.0f}{r['n_i']:>10,.0f}{r['auc']:>9.5f}{r['eer']:>8.4f}"
                  f"{r['tar2']:>8.4f}{r['tar3']:>10.4f}{r['t_far']:>8.3f}{r['frr_sys']:>14.4f}")
    print("  (AUC, EER, TAR, ngưỡng: chỉ trên ảnh lấy được mặt. FRR hệ thống = FRR ở FAR <= 1e-3 cộng thêm các ảnh không lấy được mặt)")
    print("\n  Ngưỡng chọn trên kịch bản sạch rồi dùng nguyên cho kịch bản nặng (chỉ ảnh lấy được mặt):")
    carry: dict[str, tuple[float, float, float]] = {}
    for e in embedders:
        t = scen_sum[e.name]["sạch"]["t_far"]
        g, i = split_scores(scen_scores[e.name]["nặng"], labels, labels, symmetric=False, allow_missing=True)
        far, frr = far_frr(g, i, np.array([t]))
        carry[e.name] = (t, float(far[0]), float(frr[0]))
        print(f"  {e.name:<12}ngưỡng {t:.3f}: FAR {far[0]:.5f}, FRR {frr[0]:.4f}")
    facts["carry"] = carry

    # --- [7] Chọn ngưỡng theo chi phí và ba vùng gợi ý ----------------------------------------
    print(f"\n[7] Chọn ngưỡng theo chi phí (giả định: lượt mạo danh {P_IMPOSTOR:.0%}, nhầm nhận = {C_FA:.0f} đơn vị, "
          f"nhầm từ chối = {C_FR:.0f} đơn vị); chỉ ảnh lấy được mặt")
    print(f"  {'mô hình':<12}{'kịch bản':<10}{'cách chọn':<18}{'ngưỡng':>8}{'FAR':>10}{'FRR':>8}{'chi phí/lượt':>14}")
    cost_rows: dict[tuple[str, str, str], tuple[float, float, float, float]] = {}
    for e in embedders:
        for name in ("sạch", "nặng"):
            g, i = split_scores(scen_scores[e.name][name], labels, labels, symmetric=False, allow_missing=True)
            t_cost, _ = best_threshold_by_cost(g, i, c_fa=C_FA, c_fr=C_FR, p_impostor=P_IMPOSTOR)
            picks = {"tối thiểu chi phí": t_cost, "EER": scen_sum[e.name][name]["eer_t"],
                     "FAR <= 1e-3": scen_sum[e.name][name]["t_far"]}
            for how, t in picks.items():
                far, frr = far_frr(g, i, np.array([t]))
                cost = float(expected_cost(far[0], frr[0], c_fa=C_FA, c_fr=C_FR, p_impostor=P_IMPOSTOR))
                cost_rows[(e.name, name, how)] = (t, float(far[0]), float(frr[0]), cost)
                print(f"  {e.name:<12}{name:<10}{how:<18}{t:>8.3f}{far[0]:>10.5f}{frr[0]:>8.4f}{cost:>14.5f}")
    facts["cost_rows"] = cost_rows

    main_model = embedders[0].name
    g, i = split_scores(scen_scores[main_model]["nặng"], labels, labels, symmetric=False, allow_missing=True)
    t_high = threshold_at_far(i, TARGET_FAR)
    t_low = threshold_at_frr(g, 0.01)
    print(f"\n  Ba vùng gợi ý cho người duyệt ({main_model}, kịch bản nặng, ảnh lấy được mặt): t_high = {t_high:.3f} "
          f"(FAR <= 1e-3), t_low = {t_low:.3f} (FRR <= 1%)")
    zones = ("likely_match", "unclear", "likely_mismatch")
    zone_share = {}
    for label, scores in (("genuine", g), ("impostor", i)):
        counts = [sum(suggest(s, t_low=t_low, t_high=t_high) == z for s in scores) for z in zones]
        zone_share[label] = [c / len(scores) for c in counts]
        print(f"  {label:<10}" + "".join(f"{z} {c / len(scores):>7.2%}   " for z, c in zip(zones, counts)))
    unclear = (1 - P_IMPOSTOR) * zone_share["genuine"][1] + P_IMPOSTOR * zone_share["impostor"][1]
    queue = unclear * (1 - miss["nặng"] / len(images)) + miss["nặng"] / len(images)
    print(f"  với {P_IMPOSTOR:.0%} lượt mạo danh, vùng unclear chiếm {unclear:.2%} số ảnh lấy được mặt; cộng {miss['nặng']}/{len(images)} "
          f"ảnh không lấy được mặt thì hàng người duyệt chiếm khoảng {queue:.2%} số lượt")
    facts["zone_share"], facts["t_low"], facts["t_high"], facts["queue"] = zone_share, t_low, t_high, queue

    # --- [8] Người rời nhau, nhiều cách chia và bootstrap theo người -------------------------
    print(f"\n[8] Ngưỡng chọn ở FAR <= 1e-3 trên 20 người, đo trên 20 người khác (kịch bản nặng); hai chiều rồi {RANDOM_SPLITS} cách chia ngẫu nhiên")
    studies: dict[str, dict[str, object]] = {}
    for e in embedders:
        st = split_study(scen_scores[e.name]["nặng"], labels)
        studies[e.name] = st
        for direction, key in (("người 0-19 -> 20-39", "forward"), ("người 20-39 -> 0-19", "backward")):
            r = st[key]  # type: ignore[assignment]
            print(f"  {e.name:<12}{direction}: ngưỡng {r['t']:.3f}; nhóm chọn FAR {r['far_a']:.5f} FRR {r['frr_a']:.4f}"
                  f" | nhóm chưa thấy FAR {r['far_b']:.5f} FRR {r['frr_b']:.4f}")
        fq, rq = st["far_q"], st["frr_q"]  # type: ignore[assignment]
        print(f"  {e.name:<12}{RANDOM_SPLITS} cách chia: FAR nhóm chưa thấy p05 {fq[0]:.5f}, trung vị {fq[1]:.5f}, p95 {fq[2]:.5f}; "
              f"vượt 0.001 ở {st['share_exceed']:.0%} số lần; FRR p05 {rq[0]:.4f}, trung vị {rq[1]:.4f}, p95 {rq[2]:.4f}")
    facts["studies"] = studies
    boots = {}
    for e in embedders:
        b = bootstrap(scen_scores[e.name]["nặng"], labels, scen_sum[e.name]["nặng"]["t_far"])
        boots[e.name] = b
        print(f"  {e.name:<12}bootstrap {b['draws']} lần theo người, khoảng [2.5%, trung vị, 97.5%]:")
        print(f"    chọn lại ngưỡng ở FAR <= 1e-3: FRR {b['frr'][0]:.4f} / {b['frr'][1]:.4f} / {b['frr'][2]:.4f}; "
              f"EER {b['eer'][0]:.4f} / {b['eer'][1]:.4f} / {b['eer'][2]:.4f}")
        print(f"    giữ ngưỡng cố định {scen_sum[e.name]['nặng']['t_far']:.3f}: FAR {b['far_fixed'][0]:.5f} / {b['far_fixed'][1]:.5f} / "
              f"{b['far_fixed'][2]:.5f}; FRR {b['frr_fixed'][0]:.4f} / {b['frr_fixed'][1]:.4f} / {b['frr_fixed'][2]:.4f}")
    facts["boots"] = boots

    # --- [9] Ca đặc biệt ---------------------------------------------------------------------
    print("\n[9] Ca đặc biệt")
    edge: dict[str, bool] = {}
    for label, img in (("ảnh đen", np.zeros((64, 64), np.uint8)),
                       ("nhiễu trắng", np.random.default_rng(0).integers(0, 256, (64, 64), dtype=np.uint8))):
        edge[label] = prep.aligned(img) is None
        print(f"  {label}: YuNet {'không lấy được mặt: chuyển needs_review, không có embedding' if edge[label] else 'lấy được mặt'}")
    same = float(clean[main_model][0] @ clean[main_model][0])
    nan_rows = int(np.isnan(scen_scores[main_model]["nặng"]).all(axis=1).sum())
    print(f"  một ảnh so với chính nó: cosine {same:.6f} (bị loại khỏi nhóm genuine)")
    print(f"  kịch bản nặng có {nan_rows} hàng NaN trong ma trận điểm (ảnh không lấy được mặt); split_scores từ chối chúng nếu không có allow_missing=True")
    try:
        split_scores(scen_scores[main_model]["nặng"], labels, labels, symmetric=False)
        rejected = False
    except ValueError:
        rejected = True
    facts["edge"], facts["self_cos"], facts["nan_rows"], facts["nan_rejected"] = edge, same, nan_rows, rejected
    print(f"\nEmbedding đã lưu trong {OUT.name}/ (sinh trắc học: không commit, theo bài 53 phải có retention).")
    facts["models"] = [e.name for e in embedders]
    return facts


def verify(with_arcface: bool) -> None:
    """So các kết quả kỳ vọng đã in trong bài; sai thì in FAIL và thoát với mã 1."""
    failures: list[str] = []
    total = 0

    def check(name: str, ok: bool, detail: str = "") -> None:
        nonlocal total
        total += 1
        if not ok:
            failures.append(name)
            print(f"SAI: {name} {detail}".rstrip())

    f = demo(with_arcface)
    print()
    clean_sum, plain_sum = f["clean_sum"], f["plain_sum"]
    scen_sum, scen_scores = f["scen_sum"], f["scen_scores"]
    labels, miss = f["labels"], f["miss"]
    cost_rows, studies, boots = f["cost_rows"], f["studies"], f["boots"]
    n_images = len(labels)
    check("ảnh sạch lấy được mặt ở 400/400", miss["sạch"] == 0, str(miss))
    check("YuNet hụt nhiều hơn khi ảnh xuống cấp nhiều hơn (sạch < nhẹ < nặng)", miss["sạch"] < miss["nhẹ"] < miss["nặng"], str(miss))
    check("số hàng NaN trong ma trận kịch bản nặng đúng bằng số ảnh không lấy được mặt", f["nan_rows"] == miss["nặng"],
          f"{f['nan_rows']} so với {miss['nặng']}")
    check("split_scores từ chối hàng NaN nếu không có allow_missing=True", f["nan_rejected"] is True)
    check("kích thước cặp: 1.800 genuine và 78.000 impostor (sạch, không thứ tự)", all(
        s["n_g"] == 1800 and s["n_i"] == 78000 for s in clean_sum.values()))
    check("kịch bản sạch có thứ tự: 3.600 genuine và 156.000 impostor", all(
        d["sạch"]["n_g"] == 3600 and d["sạch"]["n_i"] == 156000 for d in scen_sum.values()))
    check("kịch bản nặng: đúng 9 genuine và 390 impostor cho mỗi ảnh lấy được mặt", all(
        d["nặng"]["n_g"] == 9 * (n_images - miss["nặng"]) and d["nặng"]["n_i"] == 390 * (n_images - miss["nặng"])
        for d in scen_sum.values()), str({k: (v["nặng"]["n_g"], v["nặng"]["n_i"]) for k, v in scen_sum.items()}))
    check("embedding đã chuẩn hóa L2", f["norm_dev"] < 1e-5, str(f["norm_dev"]))
    check("«chỉ resize» làm mặt to hơn mẫu ArcFace > 1.2 lần và crop căn chỉnh có 20% đến 80% điểm viền lặp",
          f["eye_resize"] > 1.2 * f["template_eye"] and 0.2 < f["outside"] < 0.8, f"{f['eye_resize']:.1f} so với {f['template_eye']:.1f}; {f['outside']:.3f}")
    for name in f["models"]:
        check(f"{name}: căn chỉnh tốt hơn chỉ resize (EER nhỏ hơn)", clean_sum[name]["eer"] < plain_sum[name]["eer"],
              f"{clean_sum[name]['eer']:.4f} so với {plain_sum[name]['eer']:.4f}")
        check(f"{name}: EER tăng dần sạch < nhẹ < nặng",
              scen_sum[name]["sạch"]["eer"] < scen_sum[name]["nhẹ"]["eer"] < scen_sum[name]["nặng"]["eer"],
              str({k: round(v["eer"], 4) for k, v in scen_sum[name].items()}))
        check(f"{name}: AUC bộ sạch > 0.99", clean_sum[name]["auc"] > 0.99, str(clean_sum[name]["auc"]))
        check(f"{name}: TAR ở FAR 1% >= TAR ở FAR 0.1% mọi kịch bản", all(r["tar2"] >= r["tar3"] for r in scen_sum[name].values()))
        for scenario, r in scen_sum[name].items():
            check(f"{name}/{scenario}: ngưỡng ở FAR<=1e-3 cho FAR đo được <= 1e-3", r["far"] <= 1e-3, str(r["far"]))
        check(f"{name}: FRR hệ thống (cả ảnh không lấy được mặt) lớn hơn FRR trên ảnh lấy được mặt ở kịch bản nặng",
              scen_sum[name]["nặng"]["frr_sys"] > scen_sum[name]["nặng"]["frr"] + 0.01, str(scen_sum[name]["nặng"]))
        check(f"{name}: chọn ngưỡng ở kịch bản sạch rồi dùng cho kịch bản nặng làm FRR tăng",
              f["carry"][name][2] > scen_sum[name]["sạch"]["frr"], str(f["carry"][name]))
        for scenario in ("sạch", "nặng"):
            best = cost_rows[(name, scenario, "tối thiểu chi phí")][3]
            others = [cost_rows[(name, scenario, how)][3] for how in ("EER", "FAR <= 1e-3")]
            check(f"{name}/{scenario}: ngưỡng tối thiểu chi phí không đắt hơn EER hay FAR<=1e-3", best <= min(others) + 1e-12,
                  f"{best} so với {others}")
        st = studies[name]
        check(f"{name}: các cách chia 20/20 đều rời người", st["forward"]["disjoint"] and st["backward"]["disjoint"] and st["all_disjoint"])
        fq = st["far_q"]
        check(f"{name}: phân vị FAR nhóm chưa thấy có thứ tự p05 <= trung vị <= p95", fq[0] <= fq[1] <= fq[2], str(fq))
        check(f"{name}: p95 FAR trên người chưa thấy vượt mục tiêu 0.001 (ngưỡng chọn trên nhóm này không bảo đảm FAR ở nhóm khác)",
              fq[2] > TARGET_FAR, str(fq))
        b = boots[name]
        check(f"{name}: khoảng bootstrap có thứ tự lo <= trung vị <= hi", all(
            v[0] <= v[1] <= v[2] for v in (b["frr"], b["eer"], b["far_fixed"], b["frr_fixed"])), str(b))
        pairs, groups = f["structure"][name]
        check(f"{name}: các cặp khác người vượt ngưỡng dồn vào ít cặp người (số cặp người < số cặp ảnh)", groups < pairs and groups < 780,
              f"{pairs} cặp ảnh, {groups} cặp người")
    if with_arcface:
        t1, t2 = clean_sum["sface"]["t_far"], clean_sum["arcface_r50"]["t_far"]
        check("ngưỡng của hai mô hình khác nhau hơn 0.1 (không dùng chung được)", abs(t1 - t2) > 0.1, f"{t1:.3f} vs {t2:.3f}")
    check("cùng ảnh embed hai lần cho cosine > 0.9999 (suy luận xác định)", f["repeat_cos"] > 0.9999, str(f["repeat_cos"]))
    check("ảnh so với chính nó cosine ~ 1 và không nằm trong genuine", abs(f["self_cos"] - 1) < 1e-5
          and all(split_scores(s, labels, labels, symmetric=True)[0].max() < 1 - 1e-6 for s in f["clean_scores"].values()))
    for label, not_found in f["edge"].items():
        check(f"{label}: YuNet không lấy được mặt nên không có embedding (không lùi âm thầm)", not_found is True)
    zs = f["zone_share"]
    check("ba vùng: kẻ mạo danh vào likely_match <= 0.11%, người thật vào likely_mismatch <= 1.1%",
          zs["impostor"][0] <= 0.0011 and zs["genuine"][2] <= 0.011, str(zs))
    check("hàng người duyệt (unclear + ảnh không lấy được mặt) lớn hơn riêng vùng unclear", f["queue"] > (
        0.98 * zs["genuine"][1] + 0.02 * zs["impostor"][1]), str(f["queue"]))
    check("đã lưu embedding .npy và nạp lại đúng (hàng NaN chỉ ở ảnh không lấy được mặt)", all(
        np.array_equal(np.load(OUT / f"embeddings_{n}.npy"), f["clean"][n]) for n in f["models"]))

    if failures:
        print(f"FAIL {total - len(failures)}/{total}")
        sys.exit(1)
    print(f"OK {total}/{total}")


if __name__ == "__main__":
    flag = "--with-arcface" in sys.argv
    if "--verify" in sys.argv:
        verify(flag)
    else:
        demo(flag)
