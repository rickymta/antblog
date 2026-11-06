"""Bài 50: liveness và chống giả mạo cơ bản qua chất lượng ảnh.

Chạy:   uv run --project examples/toi-noi-python/50-liveness-anti-spoofing examples/toi-noi-python/50-liveness-anti-spoofing/main.py
Verify: ... main.py --verify

Lab KHÔNG làm liveness thật. Nó đo blur, độ sáng, tương phản, kích thước mặt và mẫu lặp trong phổ tần số
trên ảnh MÔ PHỎNG (khuôn mặt Olivetti đặt vào khung rồi "chụp" bằng code), để thấy cơ chế và giới hạn.
Cần mạng ở lần chạy đầu: scikit-learn tải Olivetti (tải 4,26 MB, bản cache 1,4 MB) vào ~/scikit_learn_data.
"""

import argparse
import os
import statistics
import sys
import time
from collections import Counter
from collections.abc import Callable
from typing import Any

import cv2
import numpy as np

from capture import (
    Box,
    Capture,
    Kind,
    capture,
    load_faces,
    make_scene,
    screen_grid,
    to_camera,
)
from quality import (
    Assessment,
    Metrics,
    Next,
    Thresholds,
    Verdict,
    assess,
    calibrate,
    measure,
    moire_peak,
    route,
    sharpness,
)

KINDS = list(Kind)


def seed_of(kind: Kind, index: int, base: int = 1_000_000) -> int:
    """Seed riêng cho mỗi (kiểu, ảnh): lặp lại được, không trùng giữa các bộ."""
    return base * (KINDS.index(kind) + 1) + index


def measure_all(faces: np.ndarray, target: np.ndarray, kind: Kind, indices: range | list[int], base: int) -> list[Metrics]:
    out = []
    for i in indices:
        c: Capture = capture(faces[i], int(target[i]), kind, seed_of(kind, i, base))
        out.append(measure(c.image, c.box))
    return out


def split_indices() -> tuple[list[int], list[int]]:
    """Mỗi người 10 ảnh: ảnh 0-4 để hiệu chỉnh ngưỡng, ảnh 5-9 để thử."""
    cal = [i for i in range(400) if i % 10 < 5]
    test = [i for i in range(400) if i % 10 >= 5]
    return cal, test


def rate(items: list[Assessment], pred: Callable[[Assessment], bool]) -> float:
    return sum(pred(a) for a in items) / len(items)


def flagged(a: Assessment) -> bool:
    return a.verdict is not Verdict.OK


def summarize(t: Thresholds, per_kind: dict[Kind, list[Metrics]]) -> dict[Kind, list[Assessment]]:
    return {k: [assess(m, t) for m in ms] for k, ms in per_kind.items()}


def noise_demo(faces: np.ndarray) -> dict[str, float]:
    """Cùng một cảnh: nhiễu làm Laplacian thuần của ảnh MỜ vượt ảnh nét sạch; làm mượt trước thì không."""
    rng = np.random.default_rng(5)
    scene, box = make_scene(faces[0], rng)

    def shot(sigma_blur: float, sigma_noise: float) -> np.ndarray:
        img = cv2.GaussianBlur(scene, (0, 0), sigma_blur) + np.random.default_rng(6).normal(0, sigma_noise, scene.shape)
        return np.clip(img, 0, 255).astype(np.uint8)

    sharp_clean, blur_noisy = shot(0.5, 0.0), shot(3.5, 3.0)
    return {
        "plain_sharp_clean": sharpness(sharp_clean, box, smooth=0),
        "plain_blur_noisy": sharpness(blur_noisy, box, smooth=0),
        "smooth_sharp_clean": sharpness(sharp_clean, box),
        "smooth_blur_noisy": sharpness(blur_noisy, box),
    }


def blur_sensitivity(faces: np.ndarray, t: Thresholds, test: list[int]) -> list[tuple[float, float, float]]:
    """Chụp màn hình có lưới rõ nhưng ống kính nhòe thêm: mẫu lặp còn lộ ra không, ảnh còn bị gắn cờ không?"""
    rows = []
    for sigma in (0.5, 1.0, 1.5, 2.0, 3.0):
        verdicts = []
        for i in test:
            rng = np.random.default_rng(3_000_000 + i)
            scene, box = make_scene(faces[i], rng)
            scene = screen_grid(scene, rng, depth=0.3)
            img = to_camera(scene, rng, blur=sigma)
            verdicts.append(assess(measure(img, box), t))
        moire = sum("moire_pattern" in a.reasons for a in verdicts) / len(verdicts)
        any_flag = rate(verdicts, flagged)
        rows.append((sigma, moire, any_flag))
    return rows


def split_study(faces: np.ndarray, target: np.ndarray) -> dict[str, Any]:
    """Một cách chia người cho một con số. Ta in cả phân bố qua nhiều cách chia để không suy kết luận từ một lần chia.

    Đo 400 ảnh genuine một lần, rồi chỉ chọn lại ảnh nào dùng để hiệu chỉnh và ảnh nào để thử.
    """
    g = measure_all(faces, target, Kind.GENUINE, range(400), base=7_000_000)
    person = np.asarray(target)

    def frr(cal: np.ndarray, test: np.ndarray) -> float:
        t = calibrate([g[i] for i in cal])
        return rate([assess(g[i], t) for i in test], flagged)

    def spread(values: list[float]) -> dict[str, float]:
        p10, p50, p90 = np.percentile(values, [10, 50, 90])
        return {"p10": float(p10), "p50": float(p50), "p90": float(p90), "max": float(max(values))}

    first, second = np.flatnonzero(person < 20), np.flatnonzero(person >= 20)
    rng = np.random.default_rng(50)
    same, cross = [], []
    for _ in range(200):
        pick = np.concatenate([rng.permutation(np.flatnonzero(person == p))[:5] for p in range(40)])
        mask = np.zeros(400, bool)
        mask[pick] = True
        same.append(frr(np.flatnonzero(mask), np.flatnonzero(~mask)))
        cal_persons = rng.permutation(40)[:20]
        in_cal = np.isin(person, cal_persons)
        cross.append(frr(np.flatnonzero(in_cal), np.flatnonzero(~in_cal)))
    brightness = [float(np.median([g[i].brightness for i in np.flatnonzero(person == p)])) for p in range(40)]
    return {
        "ordered": frr(first, second),
        "reversed": frr(second, first),
        "same": spread(same),
        "cross": spread(cross),
        "darkest": sorted(int(p) for p in np.argsort(brightness)[:3]),
    }


def time_per_image(faces: np.ndarray, target: np.ndarray) -> tuple[dict[str, float], tuple[float, float, float]]:
    """Thời gian mỗi ảnh 320x320 (không tính sinh ảnh), lấy lần nhanh nhất trong 7 lượt: máy bận làm lượt nào đó chậm đi."""
    caps = [capture(faces[i], int(target[i]), Kind.GENUINE, 4_000_000 + i) for i in range(0, 400, 4)]

    def best_ms(fn: Callable[[np.ndarray, Box], object]) -> float:
        runs = []
        for _ in range(7):
            t0 = time.perf_counter()
            for c in caps:
                fn(c.image, c.box)
            runs.append((time.perf_counter() - t0) / len(caps) * 1000)
        return min(runs)

    return {"sharpness": best_ms(sharpness), "moire_peak": best_ms(moire_peak), "measure": best_ms(measure)}, os.getloadavg()


def fmt_pct(x: float) -> str:
    return f"{x * 100:5.1f}%"


def run(verify: bool) -> int:
    faces, target = load_faces()
    cal_idx, test_idx = split_indices()
    print(f"Olivetti: {faces.shape[0]} ảnh {faces.shape[1]}x{faces.shape[2]}, {len(set(target.tolist()))} người, "
          f"hiệu chỉnh trên {len(cal_idx)} ảnh genuine, thử trên {len(test_idx)} ảnh mỗi kiểu")

    # --- 1. Nhiễu và Laplacian
    print("\n== 1. Laplacian thuần có bị nhiễu đánh lừa không")
    nd = noise_demo(faces)
    print(f"  ảnh nét, không nhiễu : thuần {nd['plain_sharp_clean']:7.2f} | làm mượt {nd['smooth_sharp_clean']:6.2f}")
    print(f"  ảnh mờ, thêm nhiễu   : thuần {nd['plain_blur_noisy']:7.2f} | làm mượt {nd['smooth_blur_noisy']:6.2f}")

    # --- 2. Ngưỡng
    cal_metrics = measure_all(faces, target, Kind.GENUINE, cal_idx, base=9_000_000)
    t = calibrate(cal_metrics)
    print("\n== 2. Ngưỡng hiệu chỉnh từ ảnh genuine (phân vị 1% / 99%)")
    print(f"  sharpness >= {t.sharpness_min:.2f}   brightness {t.brightness_min:.1f}..{t.brightness_max:.1f}   "
          f"contrast >= {t.contrast_min:.1f}   face_ratio >= {t.face_ratio_min:.2f}   moire_peak <= {t.moire_peak_max:.2f}")

    # --- 3. Bảng theo kiểu
    per_kind = {k: measure_all(faces, target, k, test_idx, base=1_000_000) for k in KINDS}
    results = summarize(t, per_kind)
    print("\n== 3. Kết quả theo kiểu ảnh (200 ảnh mỗi kiểu, người đã gặp lúc hiệu chỉnh)")
    print(f"  {'kiểu':<15}{'ok':>5}{'recapture':>11}{'review':>8}{'gắn cờ':>9}   trung vị sharp / sáng / t.phản / moire")
    for k in KINDS:
        c = Counter(a.verdict for a in results[k])
        ms = per_kind[k]
        med = [statistics.median(getattr(m, f) for m in ms) for f in ("sharpness", "brightness", "contrast", "moire_peak")]
        print(f"  {k.value:<15}{c[Verdict.OK]:>5}{c[Verdict.RECAPTURE]:>11}{c[Verdict.REVIEW]:>8}"
              f"{fmt_pct(rate(results[k], flagged)):>9}   {med[0]:6.2f} {med[1]:6.1f} {med[2]:6.1f} {med[3]:5.2f}")
    print("  lý do gắn cờ (một ảnh có thể có nhiều lý do):")
    for k in KINDS:
        reasons = Counter(r for a in results[k] for r in a.reasons)
        print(f"    {k.value:<15}{dict(reasons) or '-'}")

    # --- 4. Ngưỡng chuyển sang người khác
    study = split_study(faces, target)
    print("\n== 4. Ngưỡng chuyển sang người khác: ảnh genuine bị gắn cờ (400 ảnh, mỗi người 10 ảnh)")
    print(f"  chia theo thứ tự, hiệu chỉnh người 0-19, thử 20-39 : {fmt_pct(study['ordered'])}")
    print(f"  chia theo thứ tự ngược, hiệu chỉnh 20-39, thử 0-19 : {fmt_pct(study['reversed'])}")
    for key, label in (("same", "cùng người (5 ảnh hiệu chỉnh, 5 ảnh thử, đủ 40 người)"), ("cross", "rời người (20 người hiệu chỉnh, 20 người thử)")):
        v = study[key]
        print(f"  200 cách chia ngẫu nhiên {label}:")
        print(f"    trung vị {fmt_pct(v['p50'])}  p10 {fmt_pct(v['p10'])}  p90 {fmt_pct(v['p90'])}  lớn nhất {fmt_pct(v['max'])}")
    print(f"  3 người có mặt tối nhất (độ sáng trung vị thấp nhất): {study['darkest']} -> "
          f"{sum(p >= 20 for p in study['darkest'])} trong 3 người thuộc nhóm 20-39")

    # --- 5. Độ nhạy theo độ nhòe ống kính
    sens = blur_sensitivity(faces, t, test_idx)
    print("\n== 5. Chụp màn hình lưới rõ, tổng độ nhòe sigma tăng dần")
    for sigma, moire, any_flag in sens:
        print(f"  sigma {sigma:3.1f}: cờ moire {fmt_pct(moire)}   gắn cờ bất kỳ lý do {fmt_pct(any_flag)}")

    # --- 6. Thời gian
    ms, load = time_per_image(faces, target)
    print("\n== 6. Thời gian mỗi ảnh 320x320 (lượt nhanh nhất trong 7)")
    print(f"  sharpness {ms['sharpness']:.2f} ms | moire_peak {ms['moire_peak']:.2f} ms | cả 5 chỉ số {ms['measure']:.2f} ms; "
          f"load average {load[0]:.2f} {load[1]:.2f} {load[2]:.2f}")

    if not verify:
        return 0
    return verify_all(t, results, study, nd, sens)


def verify_all(t: Thresholds, results: dict[Kind, list[Assessment]],
               study: dict[str, Any], nd: dict[str, float], sens: list[tuple[float, float, float]]) -> int:
    checks: list[tuple[str, bool]] = []

    def check(name: str, ok: bool) -> None:
        checks.append((name, bool(ok)))

    def r(kind: Kind, pred: Callable[[Assessment], bool]) -> float:
        return rate(results[kind], pred)

    def is_recapture(a: Assessment) -> bool:
        return a.verdict is Verdict.RECAPTURE

    def is_review(a: Assessment) -> bool:
        return a.verdict is Verdict.REVIEW

    def has(name: str) -> Callable[[Assessment], bool]:
        return lambda a: name in a.reasons

    check("laplacian thuần: ảnh mờ có nhiễu cao hơn ảnh nét sạch (bị đánh lừa)", nd["plain_blur_noisy"] > nd["plain_sharp_clean"])
    check("laplacian làm mượt: ảnh nét sạch cao hơn ảnh mờ có nhiễu ít nhất 3 lần", nd["smooth_sharp_clean"] > 3 * nd["smooth_blur_noisy"])
    check("ngưỡng sắp đúng thứ tự (sáng min < max)", t.brightness_min < t.brightness_max)
    check("genuine cùng nhóm người: gắn cờ <= 15%", r(Kind.GENUINE, flagged) <= 0.15)
    check("blur: >= 95% bị yêu cầu chụp lại", r(Kind.BLUR, is_recapture) >= 0.95 and r(Kind.BLUR, has("blur")) >= 0.95)
    check("dark: >= 95% bị yêu cầu chụp lại, cờ dark", r(Kind.DARK, is_recapture) >= 0.95 and r(Kind.DARK, has("dark")) >= 0.95)
    check("bright: 100% có cờ bright", r(Kind.BRIGHT, has("bright")) == 1.0)
    check("small_face: 100% có cờ small_face", r(Kind.SMALL_FACE, has("small_face")) == 1.0)
    check("screen_moire: >= 95% vào hàng duyệt", r(Kind.SCREEN_MOIRE, is_review) >= 0.95)
    check("screen_weak: >= 90% vào hàng duyệt", r(Kind.SCREEN_WEAK, is_review) >= 0.90)
    check("print_halftone: >= 95% vào hàng duyệt", r(Kind.PRINT_HALFTONE, is_review) >= 0.95)
    check("giới hạn: screen_clean gần như không bị bắt (gắn cờ <= 20%, cờ moire <= 2%)",
          r(Kind.SCREEN_CLEAN, flagged) <= 0.20 and r(Kind.SCREEN_CLEAN, has("moire_pattern")) <= 0.02)
    check("giới hạn: p90 các cách chia rời người cao hơn p90 các cách chia cùng người",
          study["cross"]["p90"] > study["same"]["p90"])
    check("giới hạn: cách chia theo thứ tự (0-19 / 20-39) là ngoại lệ, cao hơn trung vị rời người ít nhất gấp đôi",
          study["ordered"] >= 2 * study["cross"]["p50"])
    check("route: ok đi tiếp; recapture được thử lại tới giới hạn rồi chuyển needs_review; review chuyển needs_review",
          route(Verdict.OK, 1) is Next.PROCEED and route(Verdict.RECAPTURE, 2) is Next.RETAKE
          and route(Verdict.RECAPTURE, 3) is Next.NEEDS_REVIEW and route(Verdict.REVIEW, 1) is Next.NEEDS_REVIEW)
    check("tổng độ nhòe 3,0: cờ moire thấp hơn độ nhòe 0,5 ít nhất 50 điểm phần trăm", sens[0][1] - sens[-1][1] >= 0.5)
    check("Verdict không có giá trị live/real", {v.value for v in Verdict} == {"ok", "recapture", "review"})

    # Bất biến của chỉ số trên tín hiệu dựng tay.
    _, xx = np.indices((320, 320))
    stripes = (128 + 60 * np.sin(2 * np.pi * xx / 3.0)).astype(np.uint8)
    noise = np.random.default_rng(0).normal(128, 20, (320, 320)).clip(0, 255).astype(np.uint8)
    box = (80, 80, 160, 160)
    check("moire_peak: sọc tuần hoàn cao hơn nhiễu trắng ít nhất 1,0 (log10)", moire_peak(stripes, box) - moire_peak(noise, box) >= 1.0)
    flat = np.full((320, 320), 120, np.uint8)
    check("sharpness: ảnh phẳng bằng 0", sharpness(flat, box) == 0.0)

    # Cùng seed, cùng kết quả.
    faces, target = load_faces()
    c1 = capture(faces[3], int(target[3]), Kind.SCREEN_MOIRE, 42)
    c2 = capture(faces[3], int(target[3]), Kind.SCREEN_MOIRE, 42)
    check("cùng seed cho cùng ảnh và cùng chỉ số", np.array_equal(c1.image, c2.image) and measure(c1.image, c1.box) == measure(c2.image, c2.box))

    failed = [name for name, ok in checks if not ok]
    print()
    for name in failed:
        print(f"SAI: {name}")
    n_ok = len(checks) - len(failed)
    if failed:
        print(f"FAIL {n_ok}/{len(checks)}")
        return 1
    print(f"OK {n_ok}/{len(checks)}")
    return 0


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--verify", action="store_true", help="kiểm các bất biến, in OK n/n")
    args = parser.parse_args()
    sys.exit(run(args.verify))


if __name__ == "__main__":
    main()
