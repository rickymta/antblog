"""Bài 41: bài toán KYC tài xế - luồng nghiệp vụ, dữ liệu tổng hợp và metric.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Cần Tesseract (brew install tesseract tesseract-lang, gói `vie`). Không cần mạng, không cần
hạ tầng Docker. Ảnh và manifest sinh ra nằm trong out/ (gitignore).
"""

import os
import subprocess
import sys
import tempfile
from collections import defaultdict
from collections.abc import Callable, Sequence
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import numpy as np
import PIL
from PIL import Image, ImageFilter

import baseline
import dataset
import metrics
from kyc import ALLOWED, PRODUCED_BY, KycStatus, advance
from metrics import MISSING, FieldPrediction
from synthetic_card import CARD_HEIGHT, CARD_WIDTH, DOC_TYPES, VARIANTS, font_has_vietnamese, make_document

OUT = Path(__file__).parent / "out"
WORKERS = 4
REVIEW_THRESHOLD = 0.80
THRESHOLDS = (0.5, 0.6, 0.7, 0.8, 0.9)
PSM_SWEEP = ("3", "4", "6", "11")
BLUR_SWEEP = (0.0, 1.0, 1.5, 2.0)  # bán kính Gaussian (điểm ảnh) cho thí nghiệm mất nét

Predictions = dict[str, dict[str, FieldPrediction]]  # sample_id -> trường -> dự đoán


# ---- Chạy baseline và gom số ---------------------------------------------------------------


def run_baseline(samples: Sequence[dataset.Sample], images: dict[str, Path], psm: str = baseline.PSM) -> Predictions:
    """Chạy Tesseract trên từng ảnh (song song WORKERS tiến trình con) rồi tách trường."""

    def one(sample: dataset.Sample) -> dict[str, FieldPrediction]:
        return baseline.extract_fields(baseline.ocr_lines(images[sample.sample_id], psm=psm), sample.doc_type)

    with ThreadPoolExecutor(WORKERS) as pool:
        results = list(pool.map(one, samples))
    return {s.sample_id: r for s, r in zip(samples, results, strict=True)}


def field_pairs(samples: Sequence[dataset.Sample], preds: Predictions) -> list[tuple[str, str]]:
    """Cặp (đáp án, OCR đọc) của mọi trường của mọi mẫu; trường vắng mặt thành chuỗi rỗng."""
    return [(truth, preds[s.sample_id].get(name, MISSING).text) for s in samples for name, truth in s.fields.items()]


def group_by(samples: Sequence[dataset.Sample], key: Callable[[dataset.Sample], str]) -> dict[str, list[dataset.Sample]]:
    groups: dict[str, list[dataset.Sample]] = defaultdict(list)
    for s in samples:
        groups[key(s)].append(s)
    return groups


def gate_stats(samples: Sequence[dataset.Sample], preds: Predictions, threshold: float) -> dict[str, int]:
    """Cổng duyệt, đơn vị là giấy tờ: bao nhiêu giấy tờ cần người duyệt, bao nhiêu lọt qua nhưng có trường sai."""
    flagged = [metrics.needs_review(s.fields, preds[s.sample_id], threshold) for s in samples]
    wrong = [not metrics.sample_is_correct(s.fields, preds[s.sample_id]) for s in samples]
    return {
        "n": len(samples),
        "review": sum(flagged),
        "passed": len(samples) - sum(flagged),
        "passed_wrong": sum(1 for f, w in zip(flagged, wrong, strict=True) if not f and w),
        "review_correct": sum(1 for f, w in zip(flagged, wrong, strict=True) if f and not w),
    }


def gate_stats_by_dossier(samples: Sequence[dataset.Sample], preds: Predictions, threshold: float) -> dict[str, int]:
    """Cổng duyệt, đơn vị là hồ sơ KYC (hai giấy tờ cùng `kyc_id`): hồ sơ cần duyệt nếu có ít nhất một
    giấy tờ cần duyệt; hồ sơ lọt mà sai nếu không cần duyệt nhưng có ít nhất một giấy tờ sai."""
    dossiers = group_by(samples, lambda s: s.kyc_id)
    flagged = [any(metrics.needs_review(s.fields, preds[s.sample_id], threshold) for s in docs) for docs in dossiers.values()]
    wrong = [any(not metrics.sample_is_correct(s.fields, preds[s.sample_id]) for s in docs) for docs in dossiers.values()]
    return {
        "n": len(dossiers),
        "review": sum(flagged),
        "passed": len(dossiers) - sum(flagged),
        "passed_wrong": sum(1 for f, w in zip(flagged, wrong, strict=True) if not f and w),
        "review_correct": sum(1 for f, w in zip(flagged, wrong, strict=True) if f and not w),
    }


def field_table(samples: Sequence[dataset.Sample], preds: Predictions) -> dict[str, dict[str, float]]:
    """Precision, recall và CER theo từng trường (gộp mọi biến thể, mọi loại giấy tờ có trường đó).

    predicted = số lần OCR đưa ra chuỗi không rỗng; correct = số lần khớp chính xác đáp án.
    precision = correct / predicted; recall = correct / tổng số lần trường đó xuất hiện.
    """
    rows: dict[str, dict[str, float]] = {}
    for name in sorted({name for s in samples for name in s.fields}):
        total = predicted = correct = 0
        pairs = []
        for s in samples:
            if name not in s.fields:
                continue
            pred = preds[s.sample_id].get(name, MISSING)
            total += 1
            predicted += bool(pred.text)
            correct += metrics.exact_match(s.fields[name], pred.text)
            pairs.append((s.fields[name], pred.text))
        rows[name] = {
            "total": total, "predicted": predicted, "correct": correct,
            "precision": correct / predicted if predicted else 0.0,
            "recall": correct / total,
            "cer": metrics.corpus_cer(pairs),
            "cer_nomarks": metrics.corpus_cer([(metrics.strip_marks(a), metrics.strip_marks(b)) for a, b in pairs]),
        }
    return rows


# ---- Các bước của demo ---------------------------------------------------------------------


def step_dataset(samples: Sequence[dataset.Sample]) -> None:
    print("\n[1] Bộ dữ liệu tổng hợp")
    print(f"font tiếng Việt hệ thống: {'có' if font_has_vietnamese() else 'không (chữ bị bỏ dấu)'}")
    print(f"{len(samples)} ảnh = {len({s.kyc_id for s in samples})} hồ sơ x {len(DOC_TYPES)} giấy tờ; "
          f"manifest: out/{dataset.MANIFEST_NAME}")
    print(f"{'':6s}" + "".join(f"{v:>8s}" for v in VARIANTS) + f"{'cộng':>8s}")
    for doc in DOC_TYPES:
        counts = [sum(1 for s in samples if s.doc_type == doc and s.variant == v) for v in VARIANTS]
        print(f"{doc:6s}" + "".join(f"{c:8d}" for c in counts) + f"{sum(counts):8d}")
    first = samples[0]
    print(f"ví dụ {first.sample_id} ({first.doc_type}, {first.variant}): {first.fields}")
    for variant in VARIANTS:
        sample = next(s for s in samples if s.variant == variant)
        with Image.open(OUT / sample.image) as img:
            print(f"  {variant:7s} {sample.sample_id} kích thước {img.size} {img.mode}")


def step_metrics() -> None:
    print("\n[2] Metric: Levenshtein, CER, WER")
    ref, hyp = "NGUYỄN VĂN A", "NGUYEN VAN A"
    print(f"ref={ref!r} hyp={hyp!r}")
    print(f"  levenshtein ký tự = {metrics.levenshtein(ref, hyp)}, CER = {metrics.cer(ref, hyp):.3f}")
    print(f"  levenshtein từ    = {metrics.levenshtein(ref.split(), hyp.split())}, WER = {metrics.wer(ref, hyp):.3f}")
    print(f"  CER sau khi bỏ dấu của cả hai = {metrics.cer(metrics.strip_marks(ref), metrics.strip_marks(hyp)):.3f}")
    pairs = [("C", "B"), ("NGUYỄN VĂN A", "NGUYỄN VĂN A")]
    mean = sum(metrics.cer(r, h) for r, h in pairs) / len(pairs)
    print(f"hai trường (sai 1/1 ký tự, sai 0/12 ký tự): trung bình CER = {mean:.3f}, "
          f"CER gộp = {metrics.corpus_cer(pairs):.3f}")
    print(f"ref='A' hyp='ABCD': CER = {metrics.cer('A', 'ABCD'):.1f} (có thể lớn hơn 1)")


def wrong_examples(samples: Sequence[dataset.Sample], preds: Predictions, name: str) -> list[str]:
    """Các dòng mô tả những lần trường `name` bị đọc sai hoặc vắng: mã ảnh, biến thể, đáp án -> OCR đọc."""
    lines = []
    for s in samples:
        if name not in s.fields:
            continue
        pred = preds[s.sample_id].get(name, MISSING)
        if not metrics.exact_match(s.fields[name], pred.text):
            lines.append(f"  {s.sample_id} {s.variant:6s} {s.fields[name]!r} -> {pred.text!r} (tin cậy {pred.confidence:.2f})")
    return lines


def step_baseline(samples: Sequence[dataset.Sample], preds: Predictions) -> None:
    print(f"\n[3] Baseline {baseline.tesseract_version()}, -l {baseline.LANG} --psm {baseline.PSM}, không tiền xử lý")
    print(f"{'biến thể':9s}{'ảnh':>4s}{'CER':>8s}{'CER bỏ dấu':>12s}{'WER':>8s}{'trường vắng':>13s}{'giấy tờ đúng hết':>18s}")
    groups = group_by(samples, lambda s: s.variant)
    for variant in (*VARIANTS, "tất cả"):
        group = samples if variant == "tất cả" else groups[variant]
        pairs = field_pairs(group, preds)
        folded = [(metrics.strip_marks(a), metrics.strip_marks(b)) for a, b in pairs]
        missing = sum(1 for _, b in pairs if not b)
        correct = sum(metrics.sample_is_correct(s.fields, preds[s.sample_id]) for s in group)
        print(f"{variant:9s}{len(group):4d}{metrics.corpus_cer(pairs):8.3f}{metrics.corpus_cer(folded):12.3f}"
              f"{metrics.corpus_wer(pairs):8.3f}{missing:7d}/{len(pairs):<5d}{correct:11d}/{len(group):<6d}")

    print("\nTheo trường (gộp 50 ảnh)")
    print(f"{'trường':15s}{'có':>5s}{'đưa ra':>8s}{'đúng':>6s}{'precision':>11s}{'recall':>8s}{'CER':>8s}{'CER bỏ dấu':>12s}")
    for name, row in field_table(samples, preds).items():
        print(f"{name:15s}{row['total']:5.0f}{row['predicted']:8.0f}{row['correct']:6.0f}"
              f"{row['precision']:11.3f}{row['recall']:8.3f}{row['cer']:8.3f}{row['cer_nomarks']:12.3f}")
    for name, limit in (("license_class", 9), ("full_name", 10)):
        print(f"\nTrường {name} sai (tối đa {limit} ca đầu theo mã ảnh)")
        for line in wrong_examples(samples, preds, name)[:limit]:
            print(line)


def step_gate(samples: Sequence[dataset.Sample], preds: Predictions) -> None:
    print("\n[4] Cổng chuyển người duyệt (thiếu trường, hoặc trường có độ tin cậy dưới ngưỡng), đơn vị giấy tờ")
    print(f"{'ngưỡng':>7s}{'cần duyệt':>11s}{'tỉ lệ':>8s}{'lọt qua':>9s}{'lọt mà sai':>12s}{'duyệt mà đúng':>15s}")
    for threshold in THRESHOLDS:
        g = gate_stats(samples, preds, threshold)
        print(f"{threshold:7.2f}{g['review']:11d}{g['review'] / g['n']:8.0%}{g['passed']:9d}"
              f"{g['passed_wrong']:12d}{g['review_correct']:15d}")
    print(f"\nTheo biến thể, ngưỡng {REVIEW_THRESHOLD:.2f}, đơn vị giấy tờ")
    print(f"{'biến thể':9s}{'ảnh':>4s}{'cần duyệt':>11s}{'lọt mà sai':>12s}")
    for variant, group in group_by(samples, lambda s: s.variant).items():
        g = gate_stats(group, preds, REVIEW_THRESHOLD)
        print(f"{variant:9s}{g['n']:4d}{g['review']:11d}{g['passed_wrong']:12d}")
    print("\nGộp theo hồ sơ KYC (25 hồ sơ; cần duyệt nếu có ít nhất một trong hai giấy tờ cần duyệt)")
    print(f"{'ngưỡng':>7s}{'cần duyệt':>11s}{'tỉ lệ':>8s}{'lọt qua':>9s}{'lọt mà sai':>12s}{'duyệt mà đúng':>15s}")
    for threshold in THRESHOLDS:
        g = gate_stats_by_dossier(samples, preds, threshold)
        print(f"{threshold:7.2f}{g['review']:11d}{g['review'] / g['n']:8.0%}{g['passed']:9d}"
              f"{g['passed_wrong']:12d}{g['review_correct']:15d}")


def step_psm(samples: Sequence[dataset.Sample], images: dict[str, Path]) -> dict[str, float]:
    """CER của cùng bộ ảnh với bốn chế độ phân trang (`--psm`): baseline phụ thuộc cấu hình."""
    print("\n[5] Cùng 50 ảnh, đổi chế độ phân trang --psm (CER gộp; 3 = tự động, 4 = một cột, 6 = một khối, 11 = chữ rải rác)")
    print(f"{'psm':>4s}" + "".join(f"{v:>9s}" for v in VARIANTS) + f"{'tất cả':>9s}")
    overall = {}
    groups = group_by(samples, lambda s: s.variant)
    for psm in PSM_SWEEP:
        preds = run_baseline(samples, images, psm)
        cells = [metrics.corpus_cer(field_pairs(groups[v], preds)) for v in VARIANTS]
        overall[psm] = metrics.corpus_cer(field_pairs(samples, preds))
        print(f"{psm:>4s}" + "".join(f"{c:9.3f}" for c in cells) + f"{overall[psm]:9.3f}")
    return overall


def blur_sweep(samples: Sequence[dataset.Sample], workdir: Path) -> dict[float, tuple[float, int, int]]:
    """CER, số trường vắng và số giấy tờ đúng hết của 50 giấy tờ sạch khi tăng độ mờ Gaussian."""
    results = {}
    for radius in BLUR_SWEEP:
        images = {}
        for s in samples:
            card = make_document(s.doc_type, dataset.seed_of(s))
            if radius:
                card = card.filter(ImageFilter.GaussianBlur(radius))
            images[s.sample_id] = workdir / f"{s.sample_id}_r{radius}.png"
            card.save(images[s.sample_id])
        preds = run_baseline(samples, images)
        pairs = field_pairs(samples, preds)
        results[radius] = (metrics.corpus_cer(pairs), sum(1 for _, b in pairs if not b),
                           sum(metrics.sample_is_correct(s.fields, preds[s.sample_id]) for s in samples))
    return results


def step_blur(samples: Sequence[dataset.Sample]) -> dict[float, tuple[float, int, int]]:
    print("\n[6] Độ mờ tăng dần trên 50 giấy tờ sạch (Gaussian, bán kính theo điểm ảnh)")
    print(f"{'bán kính':>9s}{'CER':>8s}{'trường vắng':>13s}{'giấy tờ đúng hết':>18s}")
    with tempfile.TemporaryDirectory() as tmp:
        sweep = blur_sweep(samples, Path(tmp))
    total_fields = sum(len(s.fields) for s in samples)
    for radius, (cer_value, missing, exact) in sweep.items():
        print(f"{radius:9.1f}{cer_value:8.3f}{missing:7d}/{total_fields:<5d}{exact:11d}/{len(samples)}")
    return sweep


def step_status() -> list[str]:
    print("\n[7] Luồng trạng thái hồ sơ KYC")
    path = [KycStatus.SUBMITTED, KycStatus.OCR_DONE, KycStatus.FACE_CHECKED, KycStatus.RISK_SCORED,
            KycStatus.NEEDS_REVIEW, KycStatus.APPROVED]
    current = path[0]
    for target in path[1:]:
        by = PRODUCED_BY.get(target, "người duyệt" if target in (KycStatus.APPROVED, KycStatus.REJECTED) else "quy tắc")
        current = advance(current, target)
        print(f"  -> {current.value:13s} ({by})")
    errors = []
    for start, target in ((KycStatus.SUBMITTED, KycStatus.APPROVED), (KycStatus.RISK_SCORED, KycStatus.REJECTED),
                          (KycStatus.RISK_SCORED, KycStatus.APPROVED), (KycStatus.APPROVED, KycStatus.SUBMITTED)):
        try:
            advance(start, target)
        except ValueError as exc:
            errors.append(str(exc))
            print(f"  từ chối: {exc}")
    return errors


def demo() -> dict[str, object]:
    print(f"pillow {PIL.__version__}, numpy {np.__version__}")
    samples = dataset.build_dataset(OUT)
    images = {s.sample_id: OUT / s.image for s in samples}
    step_dataset(samples)
    step_metrics()
    preds = run_baseline(samples, images)
    step_baseline(samples, preds)
    step_gate(samples, preds)
    psm_cer = step_psm(samples, images)
    sweep = step_blur(samples)
    errors = step_status()
    return {"samples": samples, "preds": preds, "psm": psm_cer, "sweep": sweep, "errors": errors}


# ---- --verify ------------------------------------------------------------------------------

# TSV giả: dòng 1 có từ conf -1 (bỏ) và hai từ conf 50, 90; dòng 2 một từ conf 96.
_FAKE_TSV = "\n".join([
    "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext",
    "4\t1\t1\t1\t1\t0\t0\t0\t10\t10\t-1\t",
    "5\t1\t1\t1\t1\t1\t0\t0\t10\t10\t50.0\tSố",
    "5\t1\t1\t1\t1\t2\t20\t0\t10\t10\t90.0\t/No.:",
    "5\t1\t1\t1\t1\t3\t40\t0\t10\t10\t-1\t",
    "5\t1\t1\t1\t2\t1\t0\t30\t10\t10\t96.0\t099085791907",
])


def _century_ok(fields: dict[str, str]) -> bool:
    number, year = fields["id_number"], int(fields["date_of_birth"][-4:])
    male = fields["sex"] == "Nam"
    expected = (0 if male else 1) if year < 2000 else (2 if male else 3)
    return (len(number) == 12 and number.isdigit() and number.startswith("099")
            and int(number[3]) == expected and number[4:6] == f"{year % 100:02d}")


def _refused(current: KycStatus, target: KycStatus, **kwargs: bool) -> bool:
    try:
        advance(current, target, **kwargs)
    except ValueError:
        return True
    return False


def verify(facts: dict[str, object]) -> int:
    samples: list[dataset.Sample] = facts["samples"]  # type: ignore[assignment]
    preds: Predictions = facts["preds"]  # type: ignore[assignment]
    sweep: dict[float, tuple[float, int, int]] = facts["sweep"]  # type: ignore[assignment]
    psm_cer: dict[str, float] = facts["psm"]  # type: ignore[assignment]
    status_errors: list[str] = facts["errors"]  # type: ignore[assignment]
    checks: list[tuple[str, bool]] = []

    def check(name: str, ok: bool) -> None:
        checks.append((name, bool(ok)))

    # Bộ dữ liệu
    by_doc = {d: [s for s in samples if s.doc_type == d] for d in DOC_TYPES}
    counts = {v: sum(1 for s in samples if s.variant == v) for v in VARIANTS}
    check("50 ảnh, 25 hồ sơ", len(samples) == 50 and len({s.kyc_id for s in samples}) == 25)
    check("25 CCCD + 25 GPLX", len(by_doc["cccd"]) == 25 and len(by_doc["gplx"]) == 25)
    check("số ảnh theo biến thể", counts == {"clean": 13, "blur": 13, "rotate": 12, "dark": 12})
    check("mọi cặp (loại giấy tờ, biến thể) có mặt",
          all(any(s.doc_type == d and s.variant == v for s in samples) for d in DOC_TYPES for v in VARIANTS))
    check("số CCCD: 12 số, 099, giới tính/thế kỷ, năm sinh", all(_century_ok(s.fields) for s in by_doc["cccd"]))
    check("số GPLX: 12 số bắt đầu 99, hạng hợp lệ",
          all(len(s.fields["license_number"]) == 12 and s.fields["license_number"].isdigit()
              and s.fields["license_number"].startswith("99")
              and s.fields["license_class"] in ("A1", "A2", "B1", "B2", "C", "D") for s in by_doc["gplx"]))
    check("CCCD và GPLX cùng hồ sơ trùng họ tên, ngày sinh",
          all(a.fields["full_name"] == b.fields["full_name"] and a.fields["date_of_birth"] == b.fields["date_of_birth"]
              for a, b in zip(by_doc["cccd"], by_doc["gplx"], strict=True)))
    check("ảnh lặp lại được theo seed",
          all(np.array_equal(np.asarray(dataset.render(s)), np.asarray(dataset.render(s))) for s in samples[:8]))
    loaded = dataset.load_manifest(OUT)
    check("manifest đọc lại đúng kế hoạch, đủ file ảnh",
          loaded == samples and all((OUT / s.image).exists() for s in loaded))
    sizes: dict[str, set[tuple[int, int]]] = {}
    for s in samples:
        with Image.open(OUT / s.image) as img:
            sizes.setdefault(s.variant, set()).add(img.size)
    check("ảnh sạch, mờ, tối cỡ thẻ; ảnh xoay lớn hơn",
          all(sizes[v] == {(CARD_WIDTH, CARD_HEIGHT)} for v in ("clean", "blur", "dark"))
          and all(w > CARD_WIDTH and h > CARD_HEIGHT for w, h in sizes["rotate"]))

    # Metric
    check("Levenshtein kitten/sitting = 3", metrics.levenshtein("kitten", "sitting") == 3)
    check("Levenshtein rỗng, đồng nhất, đối xứng",
          metrics.levenshtein("", "abc") == 3 and metrics.levenshtein("abc", "abc") == 0
          and metrics.levenshtein("kitten", "sitting") == metrics.levenshtein("sitting", "kitten"))
    check("CER NGUYỄN VĂN A = 2/12", abs(metrics.cer("NGUYỄN VĂN A", "NGUYEN VAN A") - 2 / 12) < 1e-9)
    check("WER cùng cặp = 2/3", abs(metrics.wer("NGUYỄN VĂN A", "NGUYEN VAN A") - 2 / 3) < 1e-9)
    check("CER có thể lớn hơn 1", metrics.cer("A", "ABCD") == 3.0)
    try:
        metrics.cer("", "x")
        empty_raises = False
    except ValueError:
        empty_raises = True
    check("ref rỗng thì CER ném ValueError", empty_raises)
    check("CER gộp khác trung bình CER",
          abs(metrics.corpus_cer([("C", "B"), ("NGUYỄN VĂN A", "NGUYỄN VĂN A")]) - 1 / 13) < 1e-9)
    check("NFC và NFD cùng một chữ thì CER 0", metrics.cer("Ê", "Ê") == 0.0)
    check("strip_marks", metrics.strip_marks("NGUYỄN ĐẶNG Hà Nội") == "NGUYEN DANG Ha Noi")
    check("exact_match bỏ khoảng trắng thừa", metrics.exact_match("LÊ VĂN C", " LÊ  VĂN C "))
    fp = FieldPrediction
    ok_preds = {"a": fp("x", 0.95), "b": fp("y", 0.90)}
    check("cổng: đủ trường, tin cậy cao thì không cần duyệt", not metrics.needs_review(["a", "b"], ok_preds))
    check("cổng: thiếu trường thì cần duyệt", metrics.needs_review(["a", "b", "c"], ok_preds))
    check("cổng: một trường dưới ngưỡng thì cần duyệt",
          metrics.needs_review(["a", "b"], {"a": fp("x", 0.95), "b": fp("y", 0.79)}))
    check("cổng: trường rỗng dù tin cậy cao thì cần duyệt", metrics.needs_review(["a"], {"a": fp("", 0.99)}))

    # Trạng thái
    check("đủ 7 trạng thái đúng từ điển miền",
          [s.value for s in KycStatus] == ["submitted", "ocr_done", "face_checked", "risk_scored",
                                           "needs_review", "approved", "rejected"]
          and set(ALLOWED) == set(KycStatus))
    check("rejected chỉ đến từ needs_review",
          [s for s in KycStatus if KycStatus.REJECTED in ALLOWED[s]] == [KycStatus.NEEDS_REVIEW])
    check("approved và rejected là trạng thái cuối", not ALLOWED[KycStatus.APPROVED] and not ALLOWED[KycStatus.REJECTED])
    check("bốn bước sai đều bị từ chối, kể cả duyệt tự động khi chưa bật", len(status_errors) == 4)
    check("duyệt tự động chỉ đi được khi truyền auto_approve=True, và chỉ cho đúng một cạnh",
          advance(KycStatus.RISK_SCORED, KycStatus.APPROVED, auto_approve=True) == KycStatus.APPROVED
          and not any(KycStatus.APPROVED in ALLOWED[s] for s in (KycStatus.SUBMITTED, KycStatus.OCR_DONE,
                                                                  KycStatus.FACE_CHECKED, KycStatus.RISK_SCORED))
          and _refused(KycStatus.OCR_DONE, KycStatus.APPROVED, auto_approve=True))

    # Baseline Tesseract
    check("đọc TSV: gom từ thành dòng, độ tin cậy nhỏ nhất",
          baseline.parse_tsv(_FAKE_TSV) == [baseline.OcrLine("Số /No.:", 0.5), baseline.OcrLine("099085791907", 0.96)])
    found = baseline.extract_fields([baseline.OcrLine("S6 /No.:", 0.9), baseline.OcrLine("099085791907", 0.9)], "cccd")
    check("tách trường chịu được nhãn đọc sai dấu", found["id_number"].text == "099085791907")
    check("không thấy nhãn thì trường vắng", found["full_name"] == MISSING)
    check("nhãn ở dòng cuối (không có dòng giá trị) thì vắng",
          baseline.extract_fields([baseline.OcrLine("Ngày sinh:", 0.9)], "cccd")["date_of_birth"] == MISSING)
    groups = group_by(samples, lambda s: s.variant)
    rates = {v: metrics.corpus_cer(field_pairs(g, preds)) for v, g in groups.items()}
    check("baseline: CER sạch < mờ < xoay", rates["clean"] < rates["blur"] < rates["rotate"])
    check("baseline: ảnh tối vẫn đọc tốt (CER < 0,1)", rates["dark"] < 0.1)
    table = field_table(samples, preds)
    check("0 <= recall <= precision <= 1 ở mọi trường", all(0 <= r["recall"] <= r["precision"] <= 1 for r in table.values()))
    check("liệt kê ca sai khớp bảng theo trường (đưa ra sai hoặc vắng = có - đúng)",
          all(len(wrong_examples(samples, preds, n)) == int(r["total"] - r["correct"]) for n, r in table.items()))
    gates = [gate_stats(samples, preds, t) for t in THRESHOLDS]
    dossier_gates = [gate_stats_by_dossier(samples, preds, t) for t in THRESHOLDS]
    check("gộp theo hồ sơ: 25 hồ sơ, duyệt + lọt = 25, số hồ sơ cần duyệt không vượt số giấy tờ cần duyệt",
          all(d["n"] == 25 and d["review"] + d["passed"] == 25 and d["review"] <= g["review"]
              and 2 * d["review"] >= g["review"] for d, g in zip(dossier_gates, gates, strict=True)))
    check("ngưỡng cao hơn thì không ít giấy tờ cần duyệt hơn", all(a["review"] <= b["review"] for a, b in zip(gates, gates[1:], strict=False)))
    check("cổng nhất quán: duyệt + lọt = tổng, lọt mà sai <= lọt",
          all(g["review"] + g["passed"] == g["n"] and g["passed_wrong"] <= g["passed"] for g in gates))
    check("mờ nặng (2,0) làm CER và số trường vắng cao hơn mờ nhẹ (1,0)",
          sweep[1.0][0] < sweep[2.0][0] and sweep[1.0][1] < sweep[2.0][1])

    check("chọn --psm làm CER gộp đổi hơn 0,05 (baseline phụ thuộc cấu hình)",
          max(psm_cer.values()) - min(psm_cer.values()) > 0.05)
    check("psm 4 của baseline nằm trong bảng", baseline.PSM in psm_cer)

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
