"""Bài 48: face detection và landmarks: tìm khuôn mặt trong ảnh giấy tờ và selfie.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Cần mạng ở lần chạy đầu để tải mô hình (YuNet, BlazeFace, FaceLandmarker) vào ~/.cache/toinoi-python/models
và bộ Olivetti qua sklearn. Ảnh người không được commit; ảnh sinh ra nằm trong out/ (gitignore).
Kết quả là đầu vào cho người duyệt hồ sơ KYC, không phải quyết định xác minh.
"""

import os
import platform
import statistics
import sys
import time
from collections import Counter
from pathlib import Path

import cv2
import mediapipe
import numpy as np

import faces
import synth
from faces import (
    ARCFACE_112,
    BlazeFaceDetector,
    Detector,
    HaarDetector,
    MediaPipeLandmarker,
    YuNetDetector,
    align_face,
    check_portrait,
    face_quality,
    iou,
    roll_degrees,
    similarity_transform,
)
from models import BLAZEFACE, BLAZEFACE_FULL, CACHE_DIR, FACE_LANDMARKER, YUNET, ensure_model

OUT = Path(__file__).parent / "out"
PERSONS = range(0, 400, 10)  # ảnh đầu tiên của mỗi người trong 40 người
HIT_IOU = 0.4  # trúng nếu IoU với ô đã dán >= 0.4 (ô dán gồm cả tóc nên không thể tiến sát 1)
Sample = tuple[np.ndarray, tuple[int, int, int, int]]


def hits(detector: Detector, samples: list[Sample]) -> tuple[int, int, int]:
    """(số ảnh trúng, số ảnh có hộp, tổng số mặt thừa).

    «Có hộp» trả lời câu hỏi «bộ phát hiện có tìm ra mặt không»; «trúng» còn đòi hộp khớp ô đáp án (IoU >= 0.4),
    nên khi ô đáp án là hộp bao của ô vuông đã xoay thì trúng <= có hộp. Mặt thừa = số hộp ngoài hộp khớp nhất."""
    hit = seen = extra = 0
    for image, truth in samples:
        found = detector.detect(image)
        if not found:
            continue
        seen += 1
        best = max(found, key=lambda f: iou(f.box, truth))
        hit += iou(best.box, truth) >= HIT_IOU
        extra += len(found) - 1
    return hit, seen, extra


def median_roll_error(detector: Detector, images: list[np.ndarray], truth: float) -> tuple[int, float]:
    """(số ảnh có landmark, trung vị |roll ước lượng - roll thật|, độ)."""
    errors = []
    for image in images:
        found = detector.detect(image)
        if found and found[0].landmarks is not None:
            errors.append(abs(roll_degrees(found[0].landmarks) - truth))
    return len(errors), (statistics.median(errors) if errors else float("nan"))


def residual_roll(aligner: Detector, checker: MediaPipeLandmarker, images: list[np.ndarray]) -> float:
    """Căn chỉnh bằng landmark của `aligner`, rồi đo lại độ nghiêng trên crop bằng `checker`: trung vị |roll|."""
    residuals = []
    for image in images:
        found = aligner.detect(image)
        if not found or found[0].landmarks is None:
            continue
        crop = cv2.resize(align_face(image, found[0].landmarks), (224, 224), interpolation=cv2.INTER_CUBIC)
        again = checker.detect(crop)
        if again and again[0].landmarks is not None:
            residuals.append(abs(roll_degrees(again[0].landmarks)))
    return statistics.median(residuals) if residuals else float("nan")


def flagged_share(images: list[np.ndarray], detector: Detector, reason: str) -> tuple[float, float]:
    """(tỉ lệ ảnh bị gắn `reason` trên tổng số ảnh, trung vị chỉ số tương ứng)."""
    hit = 0
    values = []
    for image in images:
        result = check_portrait(image, detector)
        hit += reason in result.reasons
        if result.quality is not None:
            values.append(result.quality.blur if reason == "blurry" else result.quality.brightness)
    return hit / len(images), (statistics.median(values) if values else float("nan"))


def demo() -> dict[str, object]:
    OUT.mkdir(exist_ok=True)
    facts: dict[str, object] = {}

    print("[1] Môi trường")
    load = os.getloadavg()[0]
    print(f"Python {platform.python_version()}, OpenCV {cv2.__version__}, MediaPipe {mediapipe.__version__}, "
          f"NumPy {np.__version__}")
    print(f"load average 1 phút lúc chạy: {load:.2f} (số lõi: {os.cpu_count()})")
    facts["load"] = load

    print(f"\n[2] Mô hình và dữ liệu (cache mô hình: {CACHE_DIR})")
    for model in (YUNET, BLAZEFACE, BLAZEFACE_FULL, FACE_LANDMARKER):
        path = ensure_model(model)
        print(f"  {model.name:<32} {path.stat().st_size:>9,} byte  {model.url}")
    print(f"  Haar: {Path(cv2.data.haarcascades).name}/haarcascade_frontalface_default.xml (có sẵn trong gói OpenCV)")
    images, labels = synth.load_olivetti()
    cache_file = Path.home() / "scikit_learn_data" / "olivetti_py3.pkz"
    print(f"  Olivetti: {images.shape[0]} ảnh {images.shape[1]}x{images.shape[2]} xám, {len(set(labels.tolist()))} người; "
          f"nguồn {synth.OLIVETTI_URL}")
    print(f"  (sklearn lưu {cache_file.name}: {cache_file.stat().st_size:,} byte trong ~/scikit_learn_data)")
    facts["olivetti_shape"] = images.shape
    facts["olivetti_people"] = len(set(labels.tolist()))
    facts["models_ok"] = True  # ensure_model đã kiểm kích thước và SHA-256, sai thì ném lỗi

    haar, yunet = HaarDetector(), YuNetDetector()
    blaze, landmarker = BlazeFaceDetector(), MediaPipeLandmarker()
    blaze_full = BlazeFaceDetector(BLAZEFACE_FULL, name="blazeface-full")
    detectors: list[Detector] = [haar, yunet, blaze, blaze_full, landmarker]

    # --- [3] So sánh bộ phát hiện ----------------------------------------------------------
    print("\n[3] Phát hiện trên 40 người (mỗi người một ảnh); trúng = IoU >= 0.4 với ô đã dán")
    sets: dict[str, list[Sample]] = {
        "chân dung trên thẻ": [synth.portrait_on_card(images[i], i) for i in PERSONS],
        "selfie mặt 200 px": [synth.selfie(images[i], seed=i, face_px=200) for i in PERSONS],
        "selfie mặt 80 px": [synth.selfie(images[i], seed=i, face_px=80) for i in PERSONS],
        "selfie mặt 40 px": [synth.selfie(images[i], seed=i, face_px=40) for i in PERSONS],
        "nghiêng 15 độ": [synth.selfie(images[i], seed=i, roll_deg=15) for i in PERSONS],
        "nghiêng 30 độ": [synth.selfie(images[i], seed=i, roll_deg=30) for i in PERSONS],
        "nghiêng 60 độ": [synth.selfie(images[i], seed=i, roll_deg=60) for i in PERSONS],
    }
    base = sets["selfie mặt 200 px"]
    sets["mờ Gauss sigma 3"] = [(synth.gaussian_blur(img, 3), box) for img, box in base]
    sets["tối gamma 3"] = [(synth.adjust_gamma(img, 3), box) for img, box in base]
    print(f"{'điều kiện':<22}" + "".join(f"{d.name:>15}" for d in detectors) + "   (trúng / có hộp / mặt thừa, trên 40 ảnh)")
    table: dict[str, dict[str, int]] = {}
    seen_table: dict[str, dict[str, int]] = {}
    for name, samples in sets.items():
        row = {d.name: hits(d, samples) for d in detectors}
        table[name] = {k: v[0] for k, v in row.items()}
        seen_table[name] = {k: v[1] for k, v in row.items()}
        print(f"{name:<22}" + "".join(f"{f'{h}/{n}/{x}':>15}" for h, n, x in row.values()))
    facts["table"], facts["seen"] = table, seen_table

    no_faces = [synth.no_face(s) for s in range(40)]
    false_alarms = {d.name: sum(len(d.detect(img)) for img in no_faces) for d in detectors}
    print(f"40 ảnh không có mặt (thẻ trống, nền phòng): số mặt báo nhầm {false_alarms}")
    facts["false_alarms"] = false_alarms

    load_before = os.getloadavg()[0]
    timing = {}
    for d in detectors:
        samples_ms = []
        for img, _ in base:
            t0 = time.perf_counter()
            d.detect(img)
            samples_ms.append((time.perf_counter() - t0) * 1000)
        timing[d.name] = statistics.median(samples_ms)
    print("thời gian trung vị mỗi ảnh 640x480 (ms), CPU, một luồng gọi: "
          + ", ".join(f"{k} {v:.1f}" for k, v in timing.items())
          + f"   (load {load_before:.1f} -> {os.getloadavg()[0]:.1f})")
    facts["timing"] = timing

    two = [synth.two_faces(images[i], images[i + 10], seed=i) for i in PERSONS[:-1]]
    counts = Counter(len(yunet.detect(img)) for img in two)
    print(f"ảnh hai người cạnh nhau (39 ảnh): YuNet báo số mặt {dict(sorted(counts.items()))}")
    facts["two_faces"] = counts

    # --- [4] Landmark và căn chỉnh -----------------------------------------------------------
    print("\n[4] Landmark 5 điểm và căn chỉnh theo mẫu ArcFace 112x112")
    print("sai số góc nghiêng: trung vị |roll ước lượng - roll thật| (độ), trước khi căn chỉnh;")
    print("rồi độ nghiêng còn lại sau khi căn chỉnh, đo lại bằng FaceLandmarker trên crop")
    print(f"{'roll thật':>10}{'yunet':>10}{'mp 478':>10}{'còn lại (YuNet)':>20}{'còn lại (MP)':>16}{'YuNet tìm thấy':>17}")
    roll_rows = {}
    for truth in (-30, -15, 0, 15, 30, 40):
        rolled = [synth.selfie(images[i], seed=i, roll_deg=truth)[0] for i in PERSONS]
        n_y, err_y = median_roll_error(yunet, rolled, truth)
        n_m, err_m = median_roll_error(landmarker, rolled, truth)
        res_y = residual_roll(yunet, landmarker, rolled)
        res_m = residual_roll(landmarker, landmarker, rolled)
        roll_rows[truth] = (err_y, err_m, res_y, res_m, n_y, n_m)
        print(f"{truth:>10}{err_y:>10.2f}{err_m:>10.2f}{res_y:>20.2f}{res_m:>16.2f}{f'{n_y}/40':>17}")
    facts["roll_rows"] = roll_rows

    # Phép đồng dạng khôi phục đúng một phép biến đổi đã biết (kiểm toán học, không cần ảnh).
    theta = np.deg2rad(25.0)
    rot = np.array([[np.cos(theta), -np.sin(theta)], [np.sin(theta), np.cos(theta)]])
    moved = 0.7 * (ARCFACE_112 @ rot.T) + np.array([30.0, -12.0])
    matrix = similarity_transform(moved, ARCFACE_112)
    back = cv2.transform(moved[None], matrix)[0]
    recover_err = float(np.abs(back - ARCFACE_112).max())
    print(f"similarity_transform khôi phục mẫu ArcFace từ bản đã xoay 25 độ, co 0.7, dịch: sai số lớn nhất {recover_err:.1e} px")
    facts["recover_err"] = recover_err

    # Lưu minh họa: thẻ có hộp và landmark; selfie nghiêng 30 độ cùng hai crop căn chỉnh.
    card, _ = sets["chân dung trên thẻ"][0]
    drawn = card.copy()
    colors = {"haar": (0, 160, 255), "yunet": (0, 200, 0), "blazeface": (255, 120, 0), "blazeface-full": (200, 200, 0), "mp-landmarker": (200, 0, 200)}
    for d in detectors:
        for f in d.detect(card):
            x, y, w, h = f.box
            cv2.rectangle(drawn, (x, y), (x + w, y + h), colors[d.name], 2)
            if f.landmarks is not None and d.name == "yunet":
                for px, py in f.landmarks:
                    cv2.circle(drawn, (int(px), int(py)), 4, (0, 0, 255), -1)
    cv2.imwrite(str(OUT / "card_detections.png"), drawn)
    tilted = synth.selfie(images[0], seed=0, roll_deg=30)[0]
    panels = [cv2.resize(tilted, (448, 336))]
    for d in (yunet, landmarker):
        found = d.detect(tilted)
        crop = align_face(tilted, found[0].landmarks) if found and found[0].landmarks is not None else np.zeros((112, 112, 3), np.uint8)
        panels.append(cv2.resize(crop, (336, 336), interpolation=cv2.INTER_CUBIC))
    cv2.imwrite(str(OUT / "aligned_roll30.png"), np.hstack(panels))

    # --- [5] Chất lượng ảnh ----------------------------------------------------------------
    print("\n[5] Chất lượng trên vùng lõi của crop căn chỉnh")
    print(f"ngưỡng hiện dùng: blur (phương sai Laplacian) < {faces.MIN_BLUR_VAR:.0f} là mờ; "
          f"độ sáng < {faces.MIN_BRIGHTNESS:.0f} là tối, > {faces.MAX_BRIGHTNESS:.0f} là cháy sáng")
    selfies = [img for img, _ in base]
    cards = [img for img, _ in sets["chân dung trên thẻ"]]
    # Vì sao đo vùng lõi: ở thẻ, crop 112 x 112 còn chứa viền khung ảnh, chữ in và mép thẻ.
    medians: dict[str, tuple[float, float]] = {}
    clean_min: dict[str, float] = {}
    for label, group in (("selfie", selfies), ("thẻ", cards)):
        whole, core, brightness = [], [], []
        for image in group:
            found = yunet.detect(image)
            crop = align_face(image, found[0].landmarks)
            whole.append(face_quality(crop, core=False).blur)
            core_q = face_quality(crop)
            core.append(core_q.blur)
            brightness.append(core_q.brightness)
        medians[label] = (statistics.median(whole), statistics.median(core))
        clean_min[label] = min(core)
        print(f"ảnh sạch, {label}: trung vị phương sai Laplacian đo cả crop {medians[label][0]:.1f}, đo vùng lõi "
              f"{medians[label][1]:.1f}; vùng lõi nhỏ nhất {min(core):.1f}; độ sáng trong [{min(brightness):.1f}, {max(brightness):.1f}]")
        clean_min[label + "_bright"] = min(brightness)
        clean_min[label + "_bright_max"] = max(brightness)
    facts["core_medians"], facts["clean_min"] = medians, clean_min
    ladder: dict[str, dict[float, tuple[float, float]]] = {"blur": {}, "gamma": {}}
    print(f"{'làm mờ Gauss':<16}{'sigma':>7}{'trung vị blur selfie':>24}{'bị gắn mờ':>12}{'trung vị blur thẻ':>22}{'bị gắn mờ':>12}")
    for sigma in (0, 1, 2, 3, 4):
        s_imgs = selfies if sigma == 0 else [synth.gaussian_blur(i, sigma) for i in selfies]
        c_imgs = cards if sigma == 0 else [synth.gaussian_blur(i, sigma) for i in cards]
        s_share, s_med = flagged_share(s_imgs, yunet, "blurry")
        c_share, c_med = flagged_share(c_imgs, yunet, "blurry")
        ladder["blur"][sigma] = (s_med, s_share)
        print(f"{'':<16}{sigma:>7}{s_med:>24.1f}{s_share:>12.0%}{c_med:>22.1f}{c_share:>12.0%}")
    print(f"{'đổi gamma':<16}{'gamma':>7}{'trung vị sáng selfie':>24}{'gắn tối/cháy':>14}")
    for gamma in (0.5, 1, 2, 3, 4):
        g_imgs = [synth.adjust_gamma(i, gamma) for i in selfies]
        dark, med = flagged_share(g_imgs, yunet, "too_dark")
        bright, _ = flagged_share(g_imgs, yunet, "too_bright")
        ladder["gamma"][gamma] = (med, dark + bright)
        print(f"{'':<16}{gamma:>7}{med:>24.1f}{f'{dark:.0%} / {bright:.0%}':>14}")
    facts["ladder"] = ladder

    # --- [6] check_portrait: các ca và 40 thẻ -----------------------------------------------
    print("\n[6] check_portrait (YuNet): trạng thái và lý do chuyển người duyệt")
    cases: dict[str, np.ndarray] = {
        "thẻ sạch": cards[0],
        "thẻ mờ sigma 4": synth.gaussian_blur(cards[0], 4),
        "thẻ tối gamma 4": synth.adjust_gamma(cards[0], 4),
        "thẻ không có ảnh": synth.no_face(0),
        "hai người trong khung": two[0],
        "selfie mặt 60 px": synth.selfie(images[0], seed=0, face_px=60)[0],
        "selfie nghiêng 35 độ": synth.selfie(images[0], seed=0, roll_deg=35)[0],
        "ảnh đen hoàn toàn": np.zeros((480, 640, 3), np.uint8),
        "mặt bị cắt mép khung": synth.selfie(images[0], seed=0, center=(0, 240))[0],
        "ảnh xám 1 kênh": cv2.cvtColor(cards[0], cv2.COLOR_BGR2GRAY),
    }
    outcomes: dict[str, tuple[str, tuple[str, ...]]] = {}
    for name, image in cases.items():
        result = check_portrait(image, yunet)
        outcomes[name] = (result.status, result.reasons)
        roll_text = f"{result.roll:.1f}" if result.roll is not None else "-"
        print(f"  {name:<24}{result.status:<14}{', '.join(result.reasons) or '-':<26}roll {roll_text}")
    facts["outcomes"] = outcomes
    tilted = check_portrait(cases["selfie nghiêng 35 độ"], landmarker)
    outcomes["nghiêng 35 độ (FaceLandmarker)"] = (tilted.status, tilted.reasons)
    roll_text = f"{tilted.roll:.1f}" if tilted.roll is not None else "-"
    print(f"  {'nghiêng 35 độ, dùng FaceLandmarker':<36}{tilted.status:<14}{', '.join(tilted.reasons) or '-'} (roll {roll_text})")
    statuses = Counter(check_portrait(img, yunet).status for img in cards)
    print(f"40 thẻ sạch: {dict(statuses)}")
    facts["clean_cards"] = statuses
    small_reasons: Counter[str] = Counter()
    for img, _ in sets["selfie mặt 80 px"]:
        small_reasons.update(check_portrait(img, yunet).reasons or ("(không lý do)",))
    print(f"40 selfie mặt 80 px (YuNet tìm đủ 40/40), lý do gắn bởi check_portrait với MIN_FACE_PX = {faces.MIN_FACE_PX}: "
          f"{dict(small_reasons)}")
    facts["small_reasons"] = small_reasons

    print(f"\nẢnh đã lưu trong {OUT.name}/: card_detections.png, aligned_roll30.png")
    return facts


def verify() -> None:
    """So các kết quả kỳ vọng đã in trong bài; sai thì in FAIL và thoát với mã 1."""
    failures: list[str] = []
    total = 0

    def check(name: str, ok: bool, detail: str = "") -> None:
        nonlocal total
        total += 1
        if not ok:
            failures.append(name)
            print(f"SAI: {name} {detail}".rstrip())

    facts = demo()
    print()
    table: dict[str, dict[str, int]] = facts["table"]  # type: ignore[assignment]
    roll_rows = facts["roll_rows"]
    ladder = facts["ladder"]
    outcomes: dict[str, tuple[str, tuple[str, ...]]] = facts["outcomes"]  # type: ignore[assignment]
    assert isinstance(roll_rows, dict) and isinstance(ladder, dict)

    check("mô hình trong cache khớp kích thước và SHA-256", facts["models_ok"] is True)
    check("Olivetti: 400 ảnh 64x64, 40 người", facts["olivetti_shape"] == (400, 64, 64) and facts["olivetti_people"] == 40)
    check("similarity_transform khôi phục phép biến đổi đã biết", float(facts["recover_err"]) < 1e-6,  # type: ignore[arg-type]
          str(facts["recover_err"]))
    check("YuNet tìm được mặt trên >= 38/40 thẻ", table["chân dung trên thẻ"]["yunet"] >= 38,
          str(table["chân dung trên thẻ"]))
    check("YuNet tìm được mặt trên >= 38/40 selfie 200 px", table["selfie mặt 200 px"]["yunet"] >= 38)
    check("YuNet vẫn tìm được mặt 80 px (>= 38/40) khi BlazeFace gần như mất (<= 10/40)",
          table["selfie mặt 80 px"]["yunet"] >= 38 and table["selfie mặt 80 px"]["blazeface"] <= 10,
          str(table["selfie mặt 80 px"]))
    check("nghiêng 30 độ: Haar mất (<= 10/40), YuNet còn >= 30/40",
          table["nghiêng 30 độ"]["haar"] <= 10 and table["nghiêng 30 độ"]["yunet"] >= 30, str(table["nghiêng 30 độ"]))
    check("nghiêng 60 độ: YuNet mất (<= 10/40), FaceLandmarker còn >= 35/40",
          table["nghiêng 60 độ"]["yunet"] <= 10 and table["nghiêng 60 độ"]["mp-landmarker"] >= 35,
          str(table["nghiêng 60 độ"]))
    false_alarms: dict[str, int] = facts["false_alarms"]  # type: ignore[assignment]
    seen = facts["seen"]
    check("nghiêng 60 độ: 'có hộp' không nhỏ hơn 'trúng' ở mọi bộ, và BlazeFace có hộp nhiều hơn trúng",
          all(seen["nghiêng 60 độ"][k] >= table["nghiêng 60 độ"][k] for k in table["nghiêng 60 độ"])
          and seen["nghiêng 60 độ"]["blazeface"] > table["nghiêng 60 độ"]["blazeface"], f"{seen['nghiêng 60 độ']} so với {table['nghiêng 60 độ']}")  # type: ignore[index]
    check("BlazeFace full range tìm mặt 80 px tốt hơn bản short range", table["selfie mặt 80 px"]["blazeface-full"] > table["selfie mặt 80 px"]["blazeface"],
          str(table["selfie mặt 80 px"]))
    check("ảnh không có mặt: YuNet, Haar và FaceLandmarker không báo nhầm", all(
        false_alarms[n] == 0 for n in ("yunet", "haar", "mp-landmarker")), str(false_alarms))
    counts: Counter[int] = facts["two_faces"]  # type: ignore[assignment]
    check("ảnh hai người: YuNet báo đúng 2 mặt trên >= 90% số ảnh", counts[2] >= 0.9 * sum(counts.values()), str(dict(counts)))
    # Landmark: FaceLandmarker 478 điểm đo góc nghiêng chính xác hơn YuNet khi nghiêng nhiều.
    check("FaceLandmarker: sai số roll trung vị < 3 độ ở mọi mức nghiêng", all(row[1] < 3.0 for row in roll_rows.values()),
          str({k: round(v[1], 2) for k, v in roll_rows.items()}))
    check("YuNet ở roll +-30: sai số roll trung vị > 3 độ (đo lại, không giả định)",
          roll_rows[30][0] > 3.0 and roll_rows[-30][0] > 3.0, str((roll_rows[30][0], roll_rows[-30][0])))
    check("căn chỉnh bằng landmark FaceLandmarker: còn lại < 3 độ; bằng YuNet ở roll 30: còn nhiều hơn",
          roll_rows[30][3] < 3.0 and roll_rows[30][2] > roll_rows[30][3], str(roll_rows[30]))
    blur_ladder = ladder["blur"]
    gamma_ladder = ladder["gamma"]
    check("blur giảm đều theo sigma (trung vị phương sai Laplacian)", all(
        blur_ladder[a][0] > blur_ladder[b][0] for a, b in ((0, 1), (1, 2), (2, 3), (3, 4))), str(blur_ladder))
    check("độ sáng giảm đều theo gamma", all(
        gamma_ladder[a][0] > gamma_ladder[b][0] for a, b in ((0.5, 1), (1, 2), (2, 3), (3, 4))), str(gamma_ladder))
    check("ảnh sạch không bị gắn mờ hay tối (0%)", blur_ladder[0][1] == 0 and gamma_ladder[1][1] == 0)
    medians, clean_min = facts["core_medians"], facts["clean_min"]
    check("ngưỡng mờ nằm dưới mức nhỏ nhất của ảnh sạch (selfie và thẻ)", faces.MIN_BLUR_VAR < min(clean_min["selfie"], clean_min["thẻ"]),
          str(clean_min))
    check("khoảng sáng cho phép bao trọn ảnh sạch", faces.MIN_BRIGHTNESS < min(clean_min["selfie_bright"], clean_min["thẻ_bright"])
          and faces.MAX_BRIGHTNESS > max(clean_min["selfie_bright_max"], clean_min["thẻ_bright_max"]), str(clean_min))
    check("đo cả crop thẻ cao gấp > 5 lần đo vùng lõi (viền thẻ lọt vào), selfie thì không", medians["thẻ"][0] > 5 * medians["thẻ"][1]
          and medians["selfie"][0] < 2 * medians["selfie"][1], str(medians))
    check("mờ sigma 4: >= 80% bị gắn mờ", blur_ladder[4][1] >= 0.8, str(blur_ladder[4]))
    check("thẻ sạch: đạt face_checked", outcomes["thẻ sạch"] == ("face_checked", ()), str(outcomes["thẻ sạch"]))
    check("thẻ mờ: needs_review blurry", outcomes["thẻ mờ sigma 4"][0] == "needs_review" and "blurry" in outcomes["thẻ mờ sigma 4"][1])
    check("thẻ tối: needs_review too_dark", "too_dark" in outcomes["thẻ tối gamma 4"][1], str(outcomes["thẻ tối gamma 4"]))
    check("thẻ không có ảnh: no_face", outcomes["thẻ không có ảnh"] == ("needs_review", ("no_face",)))
    check("hai người: multiple_faces", "multiple_faces" in outcomes["hai người trong khung"][1])
    check("mặt 60 px: face_too_small", "face_too_small" in outcomes["selfie mặt 60 px"][1], str(outcomes["selfie mặt 60 px"]))
    check("nghiêng 35 độ: FaceLandmarker gắn tilted", "tilted" in outcomes["nghiêng 35 độ (FaceLandmarker)"][1],
          str(outcomes["nghiêng 35 độ (FaceLandmarker)"]))
    check("YuNet ở roll 40: báo roll thấp hơn thật >= 10 độ (bão hòa), trong khi FaceLandmarker sai < 3 độ",
          roll_rows[40][0] >= 10.0 and roll_rows[40][1] < 3.0, str(roll_rows[40]))
    check("mặt bị cắt mép khung: face_cut_off", "face_cut_off" in outcomes["mặt bị cắt mép khung"][1],
          str(outcomes["mặt bị cắt mép khung"]))
    check("ảnh xám 1 kênh cho cùng kết luận như thẻ sạch", outcomes["ảnh xám 1 kênh"][0] == outcomes["thẻ sạch"][0],
          str(outcomes["ảnh xám 1 kênh"]))
    for bad in (np.zeros((0, 0, 3), np.uint8), np.zeros((10, 10, 3), np.float32), np.zeros((10, 10, 2), np.uint8)):
        try:
            check_portrait(bad, YuNetDetector())
            check(f"ảnh không hợp lệ {bad.shape} {bad.dtype} bị từ chối", False)
        except ValueError:
            check(f"ảnh không hợp lệ {bad.shape} {bad.dtype} bị từ chối", True)
    check("ảnh đen: no_face, không ném lỗi", outcomes["ảnh đen hoàn toàn"] == ("needs_review", ("no_face",)))
    check("mọi ca lỗi đều sang người duyệt, không ca nào tự đạt",
          all(v[0] == "needs_review" for k, v in outcomes.items() if k not in ("thẻ sạch", "ảnh xám 1 kênh")))
    check("MIN_FACE_PX = 80: selfie mặt 80 px bị gắn face_too_small ở một phần ảnh (không phải 0, không phải tất cả)",
          0 < facts["small_reasons"]["face_too_small"] < 40, str(facts["small_reasons"]))  # type: ignore[index]
    check(">= 38/40 thẻ sạch đạt face_checked", facts["clean_cards"]["face_checked"] >= 38, str(facts["clean_cards"]))  # type: ignore[index]
    check("đã lưu ảnh minh họa vào out/", all((OUT / n).exists() for n in ("card_detections.png", "aligned_roll30.png")))

    if failures:
        print(f"FAIL {total - len(failures)}/{total}")
        sys.exit(1)
    print(f"OK {total}/{total}")


if __name__ == "__main__":
    if "--verify" in sys.argv:
        verify()
    else:
        demo()
