"""Bài 42: phát hiện cạnh, biến đổi phối cảnh và deskew ảnh giấy tờ.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Cần Tesseract (brew install tesseract tesseract-lang, gói `vie`). Không cần mạng, không cần
hạ tầng Docker. Ảnh chụp giả và manifest sinh ra nằm trong out/ (gitignore).
"""

import json
import math
import os
import random
import subprocess
import sys
import tempfile
from collections import defaultdict
from collections.abc import Callable, Sequence
from concurrent.futures import ThreadPoolExecutor
from dataclasses import asdict, dataclass
from pathlib import Path

import cv2
import numpy as np
import PIL
from PIL import Image

import baseline
import metrics
import vision
from metrics import MISSING, FieldPrediction
from synthetic_card import (CARD_HEIGHT, CARD_WIDTH, DOC_TYPES, font_has_vietnamese, homography_coeffs,
                            make_document, make_document_fields, photograph)

OUT = Path(__file__).parent / "out"
WORKERS = 4
DRIVERS = 25
BASE_SEED = 2026
CONDITIONS = ("skew", "tilt", "hard", "cutoff")
# Thứ tự điều kiện theo chỉ số ảnh (mod 7): 15 skew, 14 tilt, 14 hard, 7 cutoff trong 50 ảnh.
CONDITION_CYCLE = ("skew", "tilt", "hard", "skew", "tilt", "hard", "cutoff")
TABLES = ("go_toi", "ban_xam", "ban_sang")  # ba mặt bàn cho ảnh có nền bàn
GEOMETRY = ("raw", "deskew", "warp")  # hình học: nguyên ảnh, chỉ xoay, nắn phối cảnh
# Tăng cường ảnh sau khi nắn, mỗi cột đổi một thứ so với cột kề: CLAHE (clahe), ngưỡng thích nghi với C = 10
# hoặc 15 (thr), lọc trung vị 3x3 (med), và các tổ hợp.
ENHANCE = ("clahe", "thr10", "thr15", "med+thr10", "med+thr15", "clahe+thr10", "clahe+thr15", "med+clahe")
PIPELINES = (*GEOMETRY, *ENHANCE)
ALT_LANG, ALT_PSM = "vie+eng", "6"  # cấu hình Tesseract thứ hai, dùng ở bài 43 trở đi
DESCRIPTIONS = {
    "skew": "xoay trong mặt phẳng 8-20 độ, nền phẳng sáng",
    "tilt": "nghiêng phối cảnh 15-30 độ mỗi hướng, xoay tới 10 độ, ba mặt bàn",
    "hard": "như tilt nhưng tối (x0,55), bóng chạy ngang, mờ nhẹ, nhiễu mạnh",
    "cutoff": "nghiêng nhẹ, thẻ bị đẩy sang phải nên mép phải ra ngoài khung",
}


@dataclass(frozen=True)
class Sample:
    sample_id: str
    kyc_id: str
    doc_type: str
    condition: str
    scene: str
    roll: float
    pitch: float
    yaw: float
    image: str
    fields: dict[str, str]
    corners: list[list[float]]  # bốn góc thật của thẻ trong ảnh: TL, TR, BR, BL


# ---- Sinh bộ ảnh chụp giả ------------------------------------------------------------------


def photo_params(index: int, seed: int) -> dict[str, object]:
    """Tham số «chụp» của ảnh thứ `index` (đếm từ 0), cố định theo seed."""
    rng = random.Random(seed * 101 + index)
    condition = CONDITION_CYCLE[index % len(CONDITION_CYCLE)]
    sign = lambda: rng.choice((-1.0, 1.0))  # noqa: E731
    table = TABLES[index % len(TABLES)]
    if condition == "skew":
        return {"condition": condition, "scene": "phang", "roll": sign() * rng.uniform(8, 20), "pitch": 0.0,
                "yaw": 0.0, "kwargs": {}}
    if condition == "cutoff":
        return {"condition": condition, "scene": table, "roll": sign() * rng.uniform(0, 8),
                "pitch": sign() * rng.uniform(0, 10), "yaw": sign() * rng.uniform(0, 10),
                "kwargs": {"offset": (rng.uniform(250, 350), 0.0)}}
    tilt = {"condition": condition, "scene": table, "roll": sign() * rng.uniform(0, 10),
            "pitch": sign() * rng.uniform(15, 30), "yaw": sign() * rng.uniform(15, 30), "kwargs": {}}
    if condition == "hard":
        tilt["kwargs"] = {"brightness": 0.55, "shade": 0.35, "blur": 1.0, "noise": 6.0}
    return tilt


def build_photos(out_dir: Path) -> tuple[list[Sample], dict[str, np.ndarray]]:
    """Sinh 50 ảnh chụp (25 hồ sơ x CCCD, GPLX) vào `out_dir/photos/` và `manifest.json`.

    Trả thêm các thẻ sạch (không biến dạng) của chính 50 giấy tờ, làm mốc trần cho CER.
    """
    (out_dir / "photos").mkdir(parents=True, exist_ok=True)
    samples = []
    for k in range(1, DRIVERS + 1):
        seed = BASE_SEED + k
        for j, doc_type in enumerate(DOC_TYPES):
            index = 2 * (k - 1) + j
            params = photo_params(index, seed)
            card = make_document(doc_type, seed)
            photo = photograph(card, seed, roll=params["roll"], pitch=params["pitch"], yaw=params["yaw"],  # type: ignore[arg-type]
                               scene=params["scene"], **params["kwargs"])  # type: ignore[arg-type]
            sample_id = f"IMG-{index + 1:03d}"
            relative = f"photos/{sample_id}.jpg"
            photo.image.save(out_dir / relative, quality=92)  # JPEG như ảnh tải lên thật
            samples.append(Sample(sample_id, f"KYC-{k:04d}", doc_type, str(params["condition"]), str(params["scene"]),
                                  round(float(params["roll"]), 2), round(float(params["pitch"]), 2),  # type: ignore[arg-type]
                                  round(float(params["yaw"]), 2), relative, make_document_fields(doc_type, seed),  # type: ignore[arg-type]
                                  [[round(x, 1), round(y, 1)] for x, y in photo.corners]))
    manifest = {"seed": BASE_SEED, "generator": "synthetic_card.py", "vietnamese_font": font_has_vietnamese(),
                "count": len(samples), "items": [asdict(s) for s in samples]}
    (out_dir / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    return samples, {s.sample_id: np.asarray(make_document(s.doc_type, BASE_SEED + int(s.kyc_id[4:]))) for s in samples}


# ---- Các đường xử lý ảnh --------------------------------------------------------------------


def read_bgr(path: Path) -> np.ndarray:
    image = cv2.imread(str(path), cv2.IMREAD_COLOR)
    if image is None:
        raise FileNotFoundError(path)
    return image


def pipeline_image(name: str, bgr: np.ndarray) -> np.ndarray:
    """Ảnh sau mỗi đường xử lý (đưa vào Tesseract)."""
    if name == "raw":
        return bgr
    if name == "deskew":
        return vision.deskew(bgr)[0]
    warped, _ = vision.rectify(bgr)
    if name == "warp":
        return warped
    gray = vision.to_gray(warped)
    median = vision.denoise_median(gray)
    steps = {
        "clahe": lambda: vision.clahe(gray),
        "thr10": lambda: vision.adaptive_binary(gray, c=10),
        "thr15": lambda: vision.adaptive_binary(gray, c=15),
        "med+thr10": lambda: vision.adaptive_binary(median, c=10),
        "med+thr15": lambda: vision.adaptive_binary(median, c=15),
        "clahe+thr10": lambda: vision.adaptive_binary(vision.clahe(gray), c=10),
        "clahe+thr15": lambda: vision.adaptive_binary(vision.clahe(gray), c=15),
        "med+clahe": lambda: vision.clahe(median),
    }
    if name in steps:
        return steps[name]()
    raise ValueError(f"đường xử lý lạ: {name}")


def ocr_all(samples: Sequence[Sample], paths: dict[str, Path], *, lang: str = baseline.LANG,
            psm: str = baseline.PSM) -> dict[str, dict[str, FieldPrediction]]:
    def one(sample: Sample) -> dict[str, FieldPrediction]:
        return baseline.extract_fields(baseline.ocr_lines(paths[sample.sample_id], lang=lang, psm=psm), sample.doc_type)

    with ThreadPoolExecutor(WORKERS) as pool:
        results = list(pool.map(one, samples))
    return {s.sample_id: r for s, r in zip(samples, results, strict=True)}


def run_pipelines(samples: Sequence[Sample], out_dir: Path, workdir: Path) -> tuple[
        dict[str, dict[str, dict[str, FieldPrediction]]], dict[str, dict[str, Path]]]:
    """Chạy mọi đường xử lý cho 50 ảnh: trả (pipeline -> sample_id -> trường -> dự đoán, pipeline -> sample_id -> file ảnh)."""
    results, all_paths = {}, {}
    for name in PIPELINES:
        paths = {}
        for s in samples:
            if name == "raw":
                paths[s.sample_id] = out_dir / s.image  # đưa nguyên file JPEG, không mã hóa lại
                continue
            paths[s.sample_id] = workdir / f"{name.replace('+', '_')}_{s.sample_id}.png"
            cv2.imwrite(str(paths[s.sample_id]), pipeline_image(name, read_bgr(out_dir / s.image)))
        results[name] = ocr_all(samples, paths)
        all_paths[name] = paths
    return results, all_paths


def reference_cer(samples: Sequence[Sample], cards: dict[str, np.ndarray], workdir: Path) -> float:
    """CER của chính 50 giấy tờ khi ảnh sạch, không biến dạng: mốc trần cho mọi đường xử lý."""
    paths = {}
    for s in samples:
        paths[s.sample_id] = workdir / f"ref_{s.sample_id}.png"
        Image.fromarray(cards[s.sample_id]).save(paths[s.sample_id])
    return metrics.corpus_cer(field_pairs(samples, ocr_all(samples, paths)))


# ---- Gom số ---------------------------------------------------------------------------------


def field_pairs(samples: Sequence[Sample], preds: dict[str, dict[str, FieldPrediction]]) -> list[tuple[str, str]]:
    return [(truth, preds[s.sample_id].get(name, MISSING).text) for s in samples for name, truth in s.fields.items()]


def group_by(samples: Sequence[Sample], key: Callable[[Sample], str]) -> dict[str, list[Sample]]:
    groups: dict[str, list[Sample]] = defaultdict(list)
    for s in samples:
        groups[key(s)].append(s)
    return groups


def corner_error(detected: np.ndarray, truth: Sequence[Sequence[float]]) -> float:
    """Khoảng cách trung bình (điểm ảnh) giữa 4 góc tìm được và 4 góc thật, cùng thứ tự TL, TR, BR, BL."""
    return float(np.mean(np.linalg.norm(np.asarray(detected, dtype=np.float64) - np.asarray(truth), axis=1)))


def detect_all(samples: Sequence[Sample], out_dir: Path) -> dict[str, np.ndarray | None]:
    return {s.sample_id: vision.find_card_quad(read_bgr(out_dir / s.image)) for s in samples}


def mean_gray_error(a: np.ndarray, b: np.ndarray) -> float:
    return float(np.abs(cv2.cvtColor(a, cv2.COLOR_RGB2GRAY).astype(np.float32)
                        - cv2.cvtColor(b, cv2.COLOR_RGB2GRAY).astype(np.float32)).mean())


def rotated_rect_demo(angles: Sequence[float]) -> list[tuple[float, float, float]]:
    """Vẽ hình chữ nhật 600 x 380 xoay theo từng góc (dương = theo chiều kim đồng hồ trên màn hình);
    trả (góc thật, góc thô của minAreaRect, góc chuẩn hóa của lab)."""
    rows = []
    for angle in angles:
        image = np.zeros((900, 900), dtype=np.uint8)
        box = cv2.boxPoints(((450.0, 450.0), (600.0, 380.0), angle)).astype(np.int32)
        cv2.fillPoly(image, [box], 255)
        # ink_mask coi điểm TỐI là mực, nên đảo: nền sáng, hình chữ nhật tối.
        raw, normalized = vision.min_area_rect_angle(255 - image)
        rows.append((angle, raw, normalized))
    return rows


# ---- Các bước của demo ---------------------------------------------------------------------


def step_photos(samples: Sequence[Sample]) -> None:
    print("\n[1] Bộ ảnh chụp giả")
    print(f"font tiếng Việt hệ thống: {'có' if font_has_vietnamese() else 'không (chữ bị bỏ dấu)'}")
    print(f"{len(samples)} ảnh JPEG, mỗi ảnh {PIL.Image.open(OUT / samples[0].image).size}; manifest: out/manifest.json")
    print(f"{'điều kiện':10s}{'cccd':>6s}{'gplx':>6s}{'cộng':>6s}  mô tả")
    for condition in CONDITIONS:
        group = [s for s in samples if s.condition == condition]
        cccd = sum(1 for s in group if s.doc_type == "cccd")
        print(f"{condition:10s}{cccd:6d}{len(group) - cccd:6d}{len(group):6d}  {DESCRIPTIONS[condition]}")
    example = next(s for s in samples if s.condition == "tilt")
    print(f"ví dụ {example.sample_id} ({example.condition}, nền {example.scene}): roll={example.roll}, "
          f"pitch={example.pitch}, yaw={example.yaw}; góc thật TL, TR, BR, BL = {example.corners}")


def frame_corners(sample: Sample) -> np.ndarray:
    """Bốn góc của khung xanh in trên thẻ (nằm trong mép thẻ 13 điểm ảnh) sau khi chụp: đưa góc của
    khung trên thẻ sạch qua cùng phép phối cảnh đã biến thẻ thành ảnh chụp."""
    card = np.array([[0, 0], [CARD_WIDTH, 0], [CARD_WIDTH, CARD_HEIGHT], [0, CARD_HEIGHT]], dtype=np.float32)
    matrix = cv2.getPerspectiveTransform(card, np.array(sample.corners, dtype=np.float32))
    frame = np.array([[[13, 13], [CARD_WIDTH - 13, 13], [CARD_WIDTH - 13, CARD_HEIGHT - 13], [13, CARD_HEIGHT - 13]]],
                     dtype=np.float32)
    return cv2.perspectiveTransform(frame, matrix)[0]


def step_detect(samples: Sequence[Sample], quads: dict[str, np.ndarray | None]) -> dict[str, tuple[float, float]]:
    print("\n[2] Canny + findContours + tứ giác lớn nhất")
    print(f"{'điều kiện':10s}{'tìm thấy':>10s}{'lệch mép thẻ TB':>17s}{'tối đa':>8s}{'lệch khung xanh TB':>20s}")
    stats = {}
    for condition in CONDITIONS:
        group = [s for s in samples if s.condition == condition]
        found = [s for s in group if quads[s.sample_id] is not None]
        edge = [corner_error(quads[s.sample_id], s.corners) for s in found]  # type: ignore[arg-type]
        frame = [corner_error(quads[s.sample_id], frame_corners(s)) for s in found]  # type: ignore[arg-type]
        stats[condition] = (float(np.mean(edge)) if edge else math.nan, float(np.mean(frame)) if frame else math.nan)
        print(f"{condition:10s}{len(found):6d}/{len(group):<3d}"
              + (f"{np.mean(edge):14.1f} px{max(edge):8.1f}{np.mean(frame):17.1f} px" if edge else f"{'-':>17s}{'-':>8s}{'-':>20s}"))
    return stats


def step_warp(samples: Sequence[Sample], cards: dict[str, np.ndarray], out_dir: Path) -> float:
    print("\n[4] Nắn phối cảnh bằng góc thật (kiểm hình học, chưa liên quan tới Canny)")
    errors = []
    for s in samples:
        if s.condition not in ("skew", "tilt"):
            continue  # hard đổi độ sáng, cutoff có góc ngoài khung: sai khác xám không còn đo hình học
        warped = vision.warp_card(read_bgr(out_dir / s.image), np.array(s.corners, dtype=np.float32))
        errors.append(mean_gray_error(cv2.cvtColor(warped, cv2.COLOR_BGR2RGB), cards[s.sample_id]))
    mean = float(np.mean(errors))
    print(f"{len(errors)} ảnh skew và tilt: sai khác xám trung bình giữa thẻ nắn lại và thẻ gốc = {mean:.1f} / 255 "
          f"(lớn nhất {max(errors):.1f})")
    return mean


def step_deskew(samples: Sequence[Sample], out_dir: Path) -> float:
    print("\n[5] Deskew bằng minAreaRect")
    print("hình chữ nhật vẽ sẵn xoay theo góc đã biết (dương = theo chiều kim đồng hồ):")
    print(f"{'góc thật':>10s}{'góc thô minAreaRect':>21s}{'góc chuẩn hóa':>15s}")
    for angle, raw, normalized in rotated_rect_demo((-20.0, -10.0, 0.0, 10.0, 20.0)):
        print(f"{angle:10.1f}{raw:21.1f}{normalized:15.1f}")
    by_condition: dict[str, list[tuple[float, float]]] = defaultdict(list)
    ink_share: dict[str, list[float]] = defaultdict(list)
    for s in samples:
        gray = cv2.cvtColor(read_bgr(out_dir / s.image), cv2.COLOR_BGR2GRAY)
        by_condition[s.condition].append((vision.min_area_rect_angle(gray)[1], s.roll))
        ink_share[s.condition].append(float((vision.ink_mask(gray) > 0).mean()))
    print(f"{'điều kiện':10s}{'|góc ước lượng| TB':>20s}{'|góc xoay thật| TB':>21s}{'sai số TB':>11s}{'«mực» chiếm':>13s}")
    for condition in CONDITIONS:
        pairs = by_condition[condition]
        print(f"{condition:10s}{np.mean([abs(a) for a, _ in pairs]):20.2f}{np.mean([abs(r) for _, r in pairs]):21.2f}"
              f"{np.mean([abs(a - r) for a, r in pairs]):11.2f}{np.mean(ink_share[condition]):13.1%}")
    return float(np.mean([abs(a - r) for a, r in by_condition["skew"]]))


def step_guard(samples: Sequence[Sample], out_dir: Path, workdir: Path) -> dict[str, float]:
    """Thử bỏ kiểm tra «góc sát mép khung» trên các ảnh cutoff: tìm thấy tứ giác sai, và OCR ra sao."""
    group = [s for s in samples if s.condition == "cutoff"]
    paths, errors = {}, []
    for s in group:
        bgr = read_bgr(out_dir / s.image)
        quad = vision.find_card_quad(bgr, edge_margin=None)
        if quad is None:
            image = bgr
        else:
            errors.append(corner_error(quad, s.corners))
            image = vision.warp_card(bgr, quad)
        paths[s.sample_id] = workdir / f"guard_{s.sample_id}.png"
        cv2.imwrite(str(paths[s.sample_id]), image)
    cer = metrics.corpus_cer(field_pairs(group, ocr_all(group, paths)))
    print("\n[3] Bỏ kiểm tra góc sát mép khung trên ảnh cutoff")
    print(f"tìm thấy {len(errors)}/{len(group)} tứ giác, lệch góc thật từ {min(errors):.1f} đến {max(errors):.1f} px; "
          f"CER sau nắn = {cer:.3f}")
    return {"found": float(len(errors)), "min_error": min(errors), "cer": cer}


def highpass_noise(gray: np.ndarray) -> float:
    """Độ lệch chuẩn của phần «tần số cao» (ảnh trừ bản làm mờ hộp 5x5) trong vùng trống bên phải
    thẻ, nơi không có chữ: ước lượng thô của nhiễu hạt."""
    patch = gray[170:470, 650:980].astype(np.float32)
    return float((patch - cv2.blur(patch, (5, 5))).std())


def step_noise(samples: Sequence[Sample], out_dir: Path) -> dict[str, float]:
    print("\n[6] Nhiễu hạt trong vùng trống của thẻ sau khi nắn (ảnh hard)")
    stages: dict[str, list[float]] = defaultdict(list)
    for s in (x for x in samples if x.condition == "hard"):
        warped, quad = vision.rectify(read_bgr(out_dir / s.image))
        if quad is None:
            continue
        gray = vision.to_gray(warped)
        stages["xám sau nắn"].append(highpass_noise(gray))
        stages["sau CLAHE"].append(highpass_noise(vision.clahe(gray)))
        stages["sau lọc trung vị"].append(highpass_noise(vision.denoise_median(gray)))
    means = {name: float(np.mean(values)) for name, values in stages.items()}
    print(f"{len(stages['xám sau nắn'])} ảnh hard nắn được: " + ", ".join(f"{name} {value:.2f}" for name, value in means.items()))
    return means


def groups_of(samples: Sequence[Sample]) -> dict[str, list[Sample]]:
    """Các hàng của bảng: bốn điều kiện, rồi «tất cả» (50 ảnh) và «không cutoff» (43 ảnh)."""
    groups = {c: [s for s in samples if s.condition == c] for c in CONDITIONS}
    groups["tất cả"] = list(samples)
    groups["không cutoff"] = [s for s in samples if s.condition != "cutoff"]
    return groups


def print_cer_table(title: str, columns: Sequence[str], samples: Sequence[Sample],
                    results: dict[str, dict[str, dict[str, FieldPrediction]]], width: int = 13) -> dict[str, dict[str, float]]:
    print(title)
    print(f"{'điều kiện':13s}{'ảnh':>4s}" + "".join(f"{c:>{width}s}" for c in columns))
    cells = {}
    for name, group in groups_of(samples).items():
        row = {c: metrics.corpus_cer(field_pairs(group, results[c])) for c in columns}
        cells[name] = row
        print(f"{name:13s}{len(group):4d}" + "".join(f"{row[c]:{width}.3f}" for c in columns))
    return cells


def table_cer(samples: Sequence[Sample], results: dict[str, dict[str, dict[str, FieldPrediction]]],
              alt_results: dict[str, dict[str, dict[str, FieldPrediction]]], reference: float,
              ) -> dict[str, dict[str, dict[str, float]]]:
    print(f"\n[7] Hình học: CER gộp theo trường, Tesseract -l {baseline.LANG} --psm {baseline.PSM} "
          f"(ảnh sạch không biến dạng: {reference:.3f})")
    geometry = print_cer_table("", GEOMETRY, samples, results)
    print("\nSố giấy tờ đọc đúng hết các trường (trên 50 giấy tờ)")
    print(f"{'điều kiện':13s}{'ảnh':>4s}" + "".join(f"{c:>13s}" for c in GEOMETRY))
    for name, group in groups_of(samples).items():
        print(f"{name:13s}{len(group):4d}" + "".join(
            f"{sum(metrics.sample_is_correct(s.fields, results[c][s.sample_id]) for s in group):13d}" for c in GEOMETRY))
    enhance = print_cer_table(f"\n[8] Tăng cường ảnh sau nắn (Tesseract -l {baseline.LANG} --psm {baseline.PSM}), mỗi cột đổi một thứ",
                              ("warp", *ENHANCE), samples, results, width=12)
    alt = print_cer_table(f"\n[9] Cùng ba đường hình học, Tesseract -l {ALT_LANG} --psm {ALT_PSM}", GEOMETRY, samples, alt_results)
    return {"geometry": geometry, "enhance": enhance, "alt": alt}


def demo() -> dict[str, object]:
    print(f"pillow {PIL.__version__}, numpy {np.__version__}, opencv {cv2.__version__}")
    samples, cards = build_photos(OUT)
    step_photos(samples)
    quads = detect_all(samples, OUT)
    detect_stats = step_detect(samples, quads)
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        guard = step_guard(samples, OUT, work)
        warp_mae = step_warp(samples, cards, OUT)
        skew_error = step_deskew(samples, OUT)
        noise = step_noise(samples, OUT)
        results, paths = run_pipelines(samples, OUT, work)
        alt_results = {name: ocr_all(samples, paths[name], lang=ALT_LANG, psm=ALT_PSM) for name in GEOMETRY}
        reference = reference_cer(samples, cards, work)
    cells = table_cer(samples, results, alt_results, reference)
    return {"samples": samples, "cards": cards, "quads": quads, "detect_stats": detect_stats, "guard": guard,
            "warp_mae": warp_mae, "skew_error": skew_error, "noise": noise, "results": results, "alt_results": alt_results,
            "reference": reference, "cells": cells}


# ---- --verify ------------------------------------------------------------------------------


def verify(facts: dict[str, object]) -> int:
    samples: list[Sample] = facts["samples"]  # type: ignore[assignment]
    quads: dict[str, np.ndarray | None] = facts["quads"]  # type: ignore[assignment]
    all_cells: dict[str, dict[str, dict[str, float]]] = facts["cells"]  # type: ignore[assignment]
    cells, enhance, alt = all_cells["geometry"], all_cells["enhance"], all_cells["alt"]
    reference: float = facts["reference"]  # type: ignore[assignment]
    results = facts["results"]
    checks: list[tuple[str, bool]] = []

    def check(name: str, ok: bool) -> None:
        checks.append((name, bool(ok)))

    # Bộ ảnh
    counts = {c: sum(1 for s in samples if s.condition == c) for c in CONDITIONS}
    check("50 ảnh, 25 hồ sơ", len(samples) == 50 and len({s.kyc_id for s in samples}) == 25)
    check("số ảnh theo điều kiện 15/14/14/7", counts == {"skew": 15, "tilt": 14, "hard": 14, "cutoff": 7})
    check("sinh lại cùng seed ra cùng thông số",
          all(photo_params(i, BASE_SEED + 1 + i // 2) == photo_params(i, BASE_SEED + 1 + i // 2) for i in range(50)))
    loaded = json.loads((OUT / "manifest.json").read_text(encoding="utf-8"))["items"]
    check("manifest khớp danh sách mẫu và đủ file", [Sample(**i) for i in loaded] == samples
          and all((OUT / s.image).exists() for s in samples))
    inside = lambda c: all(0 <= x <= 1600 and 0 <= y <= 1200 for x, y in c)  # noqa: E731
    check("góc thật nằm trong khung, trừ ảnh cutoff (cắt mép)",
          all(inside(s.corners) for s in samples if s.condition != "cutoff")
          and all(not inside(s.corners) for s in samples if s.condition == "cutoff"))

    # Hình học
    src = np.array([[0, 0], [CARD_WIDTH, 0], [CARD_WIDTH, CARD_HEIGHT], [0, CARD_HEIGHT]], dtype=np.float64)
    dst = np.array(samples[1].corners, dtype=np.float64)
    a, b, c, d, e, f, g, h = homography_coeffs(dst, src)
    mapped = [((a * x + b * y + c) / (g * x + h * y + 1), (d * x + e * y + f) / (g * x + h * y + 1)) for x, y in dst]
    check("homography_coeffs đưa góc đích về góc nguồn (sai số < 1e-6)", np.allclose(mapped, src, atol=1e-6))
    check("order_corners sắp TL, TR, BR, BL dù đưa vào lộn xộn",
          np.allclose(vision.order_corners(np.array([[900, 700], [100, 100], [100, 650], [880, 120]])),
                      [[100, 100], [880, 120], [900, 700], [100, 650]]))
    rows = rotated_rect_demo((-20.0, -10.0, 0.0, 10.0, 20.0))
    check("góc chuẩn hóa khớp góc vẽ (sai số < 1 độ) cả hai chiều xoay",
          all(abs(normalized - angle) < 1.0 for angle, _, normalized in rows))
    check("warp bằng góc thật khôi phục thẻ gốc (sai khác xám TB < 15)", facts["warp_mae"] < 15)  # type: ignore[operator]

    # Phát hiện
    uncut = [s for s in samples if s.condition != "cutoff"]
    found = [s for s in uncut if quads[s.sample_id] is not None]
    check("tìm được tứ giác ở ít nhất 90% ảnh không bị cắt", len(found) >= 0.9 * len(uncut))
    check("tứ giác tìm được lệch góc thật dưới 25 px",
          all(corner_error(quads[s.sample_id], s.corners) < 25 for s in found))  # type: ignore[arg-type]
    check("mọi ảnh bị cắt mép đều bị từ chối (None), không nắn sai",
          all(quads[s.sample_id] is None for s in samples if s.condition == "cutoff"))
    guard: dict[str, float] = facts["guard"]  # type: ignore[assignment]
    check("bỏ kiểm tra sát mép: cả 7 ảnh cutoff đều ra tứ giác nhưng lệch góc thật trên 25 px",
          guard["found"] == 7 and guard["min_error"] > 25)
    clipped = np.full((600, 800, 3), 60, dtype=np.uint8)
    cv2.rectangle(clipped, (200, -100), (700, 450), (230, 230, 230), -1)  # thẻ vượt mép trên khung
    check("thẻ vượt mép khung: có kiểm tra thì None, tắt kiểm tra thì ra tứ giác có góc trên mép",
          vision.find_card_quad(clipped) is None
          and (loose := vision.find_card_quad(clipped, edge_margin=None)) is not None and loose[:, 1].min() == 0)
    table_only = np.full((600, 800, 3), 90, dtype=np.uint8)
    cv2.randn(table_only, 90, 4)
    check("ảnh không có thẻ thì trả None", vision.find_card_quad(table_only) is None)
    clean_card = cv2.cvtColor(np.asarray(make_document("cccd", BASE_SEED + 1)), cv2.COLOR_RGB2BGR)
    warped_again, quad_again = vision.rectify(clean_card)
    check("thẻ đã thẳng, không có nền: tứ giác tìm được là khung xanh in trên thẻ (cách mép ảnh 10 điểm)",
          quad_again is not None and np.allclose(quad_again, [[10, 10], [1001, 10], [1001, 627], [10, 627]], atol=4)
          and warped_again.shape == clean_card.shape)
    check("warp ra đúng cỡ thẻ chuẩn", vision.warp_card(clean_card, np.array([[0, 0], [1011, 0], [1011, 637], [0, 637]])).shape[:2] == (CARD_HEIGHT, CARD_WIDTH))
    straight_angle = vision.min_area_rect_angle(cv2.cvtColor(clean_card, cv2.COLOR_BGR2GRAY))[1]
    check("thẻ thẳng ước lượng góc nghiêng ~0 (|góc| < 0,5)", abs(straight_angle) < 0.5)
    check("góc ước lượng ảnh skew sai số TB < 1 độ", facts["skew_error"] < 1.0)  # type: ignore[operator]

    # OCR trước/sau
    check("CER ảnh sạch (mốc trần) < 0,05", reference < 0.05)
    check("tilt: nắn phối cảnh giảm CER ít nhất một nửa", cells["tilt"]["warp"] < 0.5 * cells["tilt"]["raw"])
    check("hard: nắn phối cảnh giảm CER ít nhất một nửa", cells["hard"]["warp"] < 0.5 * cells["hard"]["raw"])
    check("skew: deskew giảm CER ít nhất một nửa", cells["skew"]["deskew"] < 0.5 * cells["skew"]["raw"])
    check("cutoff: warp rơi về ảnh gốc nên CER bằng raw", abs(cells["cutoff"]["warp"] - cells["cutoff"]["raw"]) < 1e-9)
    check("đủ cột và 6 hàng ở ba bảng CER",
          all(set(r) == set(GEOMETRY) for r in cells.values()) and all(set(r) == {"warp", *ENHANCE} for r in enhance.values())
          and all(set(r) == set(GEOMETRY) for r in alt.values()) and len(cells) == len(enhance) == len(alt) == 6)
    stats: dict[str, tuple[float, float]] = facts["detect_stats"]  # type: ignore[assignment]
    check("nền phẳng sáng (skew): tứ giác bám khung xanh in trên thẻ hơn mép thẻ", stats["skew"][1] < stats["skew"][0])
    check("nền bàn (tilt): tứ giác bám mép thẻ hơn khung xanh", stats["tilt"][0] < stats["tilt"][1])
    noise: dict[str, float] = facts["noise"]  # type: ignore[assignment]
    check("CLAHE làm nhiễu hạt trong vùng trống tăng ít nhất 2 lần, lọc trung vị làm giảm",
          noise["sau CLAHE"] > 2 * noise["xám sau nắn"] and noise["sau lọc trung vị"] < noise["xám sau nắn"])
    check("CLAHE rồi ngưỡng (C = 10 và C = 15) làm hỏng OCR trên bộ này (CER gộp > 0,5)",
          enhance["tất cả"]["clahe+thr10"] > 0.5 and enhance["tất cả"]["clahe+thr15"] > 0.5)
    check("ngưỡng một mình: C = 10 hỏng (> 0,2), C = 15 cứu được (< 0,2); lọc trung vị cứu được cả C = 10",
          enhance["tất cả"]["thr10"] > 0.2 and enhance["tất cả"]["thr15"] < 0.2 and enhance["tất cả"]["med+thr10"] < 0.2)
    check("lọc trung vị rồi ngưỡng (C = 10) tốt hơn ảnh gốc ở mọi điều kiện",
          all(enhance[c]["med+thr10"] < cells[c]["raw"] for c in CONDITIONS))
    check("cấu hình thứ hai (vie+eng, psm 6) có đủ 50 dự đoán cho ba đường hình học",
          all(len(facts["alt_results"][g]) == 50 for g in GEOMETRY))  # type: ignore[index]
    check("mọi dự đoán đủ 50 ảnh", all(len(results[p]) == 50 for p in PIPELINES))  # type: ignore[index]

    failed = [name for name, ok in checks if not ok]
    for name in failed:
        print(f"SAI: {name}")
    passed = len(checks) - len(failed)
    if failed:
        print(f"FAIL {passed}/{len(checks)}")
        return 1
    print(f"OK {passed}/{len(checks)}")
    return 0


def _languages() -> set[str]:
    out = subprocess.run([baseline.TESSERACT, "--list-langs"], capture_output=True, text=True, check=True)
    return set((out.stdout + out.stderr).split())


def main() -> int:
    if not baseline.tesseract_available():
        print("FAIL: không tìm thấy tesseract (brew install tesseract tesseract-lang)")
        return 1
    if baseline.LANG not in _languages():
        print(f"FAIL: tesseract thiếu gói ngôn ngữ {baseline.LANG} (brew install tesseract-lang)")
        return 1
    facts = demo()
    if "--verify" in sys.argv:
        print()
        return verify(facts)
    return 0


if __name__ == "__main__":
    print(f"load average (1 phút): {os.getloadavg()[0]:.1f}")
    sys.exit(main())
