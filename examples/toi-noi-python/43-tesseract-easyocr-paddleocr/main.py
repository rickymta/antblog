"""Bài 43: Tesseract, EasyOCR và PaddleOCR trên cùng một bộ thẻ căn cước giả.

Chạy (từ gốc repository):
  uv run --project <lab> <lab>/main.py                       Tesseract: quét -l và --psm, đo, ghi kết quả
  uv run --project <lab> --extra easyocr <lab>/main.py --engine easyocr
  uv run --project <lab>/paddle <lab>/main.py --engine paddleocr        (Python 3.13, xem README)
  uv run --project <lab>/paddle <lab>/main.py --engine paddleocr-v5
  uv run --project <lab> <lab>/main.py --compare             chỉ in bảng từ các file out/results-*.json
  uv run --project <lab> <lab>/main.py --verify              chỉ cần Tesseract, in OK n/n

Ảnh, manifest và kết quả nằm trong out/ (gitignore). EasyOCR và PaddleOCR tải mô hình ở lần chạy
đầu (cần mạng) vào cache của chính thư viện, không vào repository.
"""

import argparse
import sys
import time
from pathlib import Path

from compare import (
    best_line_for,
    format_report,
    load_all,
    run_engine,
    save_results,
    summarize,
)
from dataset import VARIANTS, Sample, build_dataset
from engines import ENGINE_NAMES, TesseractEngine, make_engine
from ocr_common import EngineUnavailable, cpu_seconds, load_average, score_fields

OUT = Path(__file__).parent / "out"
LANGS = ("eng", "vie", "vie+eng")
PSMS = (3, 4, 6, 11)
PSM_NAMES = {3: "tự phân trang", 4: "một cột chữ", 6: "một khối chữ", 11: "chữ rời rạc"}


def sweep_tesseract(samples: list[Sample]) -> dict[tuple[str, int], dict[str, float]]:
    """Chạy Tesseract với mọi cặp (-l, --psm) trên cả bộ ảnh. Trả CER trung bình theo biến thể."""
    table: dict[tuple[str, int], dict[str, float]] = {}
    for lang in LANGS:
        for psm in PSMS:
            engine = TesseractEngine(lang, psm)
            by_variant: dict[str, list[float]] = {v: [] for v in VARIANTS}
            exact = exact_folded = 0
            for s in samples:
                scores = score_fields(s.fields, engine.recognize(s.path))
                by_variant[s.variant].append(sum(sc.cer for sc in scores.values()) / len(scores))
                exact += sum(sc.exact for sc in scores.values())
                exact_folded += sum(sc.exact_folded for sc in scores.values())
            cells = {v: sum(x) / len(x) for v, x in by_variant.items()}
            cells["tất cả"] = sum(sum(x) for x in by_variant.values()) / len(samples)
            cells["đúng %"] = 100 * exact / (len(samples) * 5)
            cells["đúng bỏ dấu %"] = 100 * exact_folded / (len(samples) * 5)
            table[lang, psm] = cells
    return table


def print_sweep(table: dict[tuple[str, int], dict[str, float]]) -> None:
    print(f"{'-l':<8}{'--psm':<7}" + "".join(f"{c:>10}" for c in (*VARIANTS, "tất cả", "đúng %", "bỏ dấu %")))
    for (lang, psm), cells in table.items():
        row = "".join(f"{cells[v]:>10.3f}" for v in (*VARIANTS, "tất cả"))
        print(f"{lang:<8}{psm:<7}{row}{cells['đúng %']:>10.0f}{cells['đúng bỏ dấu %']:>10.0f}")


def show_words(sample: Sample) -> None:
    """Dạng gốc của Tesseract: từng TỪ có hộp (trái, trên, rộng, cao) và confidence 0..100.

    In các từ nằm trong dải số căn cước đến họ tên (y từ 200 đến 320 trên thẻ không xoay).
    """
    import pytesseract
    from PIL import Image

    data = pytesseract.image_to_data(Image.open(sample.path), lang="vie+eng", config="--psm 6",
                                     output_type=pytesseract.Output.DICT)
    print(f"{'từ':<14}{'conf':>6}{'trái':>6}{'trên':>6}{'rộng':>6}{'cao':>5}")
    for i, word in enumerate(data["text"]):
        if word.strip() and 200 <= data["top"][i] <= 320:
            print(f"{word:<14}{float(data['conf'][i]):>6.1f}{data['left'][i]:>6}{data['top'][i]:>6}"
                  f"{data['width'][i]:>6}{data['height'][i]:>5}")


def show_same_image(results: list[dict], sample: Sample) -> None:
    """Cùng một ảnh, dòng chứa họ tên theo từng engine: chữ, confidence (0..1), hộp (x0,y0,x1,y1)."""
    from ocr_common import lines_from_json

    print(f"đáp án full_name = {sample.fields['full_name']!r}")
    for result in results:
        row = next(r for r in result["images"] if r["id"] == sample.sample_id)
        line = best_line_for(sample.fields["full_name"], lines_from_json(row["lines"]))
        if line is None:
            print(f"  {result['engine']:<14} (không có dòng nào)")
        else:
            print(f"  {result['engine']:<14} {line.text!r:<30} conf={line.confidence:.2f} box={line.box}")


def measure(name: str, samples: list[Sample]) -> dict:
    """Tạo engine (đo thời gian và CPU khởi tạo), chạy bộ ảnh, ghi out/results-<name>.json."""
    t0, c0 = time.perf_counter(), cpu_seconds()
    engine = make_engine(name)
    init_seconds, init_cpu = time.perf_counter() - t0, cpu_seconds() - c0
    result = run_engine(engine, samples, init_seconds=init_seconds, init_cpu=init_cpu)
    save_results(OUT, result)
    return result


def demo(engine_name: str) -> int:
    samples = build_dataset(OUT)
    print(f"[1] Bộ ảnh: {len(samples)} ảnh = {len(samples) // len(VARIANTS)} thẻ x {len(VARIANTS)} biến thể {VARIANTS}")
    print(f"    ví dụ đáp án {samples[0].sample_id}: {samples[0].fields}")
    if engine_name == "tesseract":
        print("\n[2] Tesseract trả về từng từ (--psm 6, -l vie+eng), ảnh s1-clean:")
        show_words(samples[0])
        print("\n[3] Quét -l x --psm trên cả bộ: CER trung bình theo biến thể (thấp là tốt), "
              "và % trường khớp nguyên văn")
        print_sweep(sweep_tesseract(samples))
    print(f"\n[4] Đo {engine_name} trên {len(samples)} ảnh (tải máy lúc bắt đầu: {load_average():.1f})")
    try:
        result = measure(engine_name, samples)
    except EngineUnavailable as exc:
        print(f"    KHÔNG CHẠY ĐƯỢC {engine_name}: {exc}")
        return 1
    print(f"    {result['info']}, Python {result['python']}, tải {result['load_before']} -> {result['load_after']}")
    results = load_all(OUT, ENGINE_NAMES)
    print("\n[5] Cùng một ảnh, các engine đã chạy:")
    show_same_image(results, samples[0])
    print("\n[6] So sánh các engine đã có file kết quả:\n")
    print(format_report(results, samples))
    return 0


# ---- verify ----------------------------------------------------------------------------


def verify() -> int:
    import json

    import numpy as np
    from PIL import Image

    from dataset import dataset_digest, make_variant, non_ascii_chars
    from ocr_common import (
        OcrError,
        OcrLine,
        best_substring_distance,
        cer,
        fold,
        levenshtein,
        lines_from_json,
        nfc,
    )
    from synthetic_card import make_id_card

    passed = 0
    failed: list[str] = []
    total = 0

    def check(name: str, ok: bool) -> None:
        nonlocal passed, total
        total += 1
        if ok:
            passed += 1
        else:
            failed.append(name)

    samples = build_dataset(OUT)
    by_id = {s.sample_id: s for s in samples}
    check("bộ ảnh có 24 ảnh và đủ file", len(samples) == 24 and all(s.path.exists() for s in samples))
    check("manifest.json đủ 24 mục", len(json.loads((OUT / "manifest.json").read_text(encoding="utf-8"))) == 24)
    clean_fields = [s.fields for s in samples if s.variant == "clean"]
    check("sáu thẻ phủ đủ sáu tên và sáu thành phố, mỗi thứ một lần",
          len({f["full_name"] for f in clean_fields}) == 6 and len({f["hometown_city"] for f in clean_fields}) == 6)
    accents = non_ascii_chars(samples)
    check("đáp án có chữ hoa dấu chồng (Ầ, Ễ) và Đ", all(ch in accents for ch in "ẦỄĐ"))
    first = dataset_digest(samples)
    check("sinh lại cùng seed cho cùng ảnh", dataset_digest(build_dataset(OUT)) == first)

    card = make_id_card(1)
    sizes = {v: make_variant(card, v, 1).size for v in VARIANTS}
    clean_px = np.asarray(card, dtype=np.float32)
    check("ảnh xoay lớn hơn ảnh gốc (expand=True)", sizes["rotate"][0] > card.size[0] and sizes["rotate"][1] > card.size[1])
    check("ảnh tối có độ sáng trung bình thấp hơn nửa ảnh gốc",
          float(np.asarray(make_variant(card, "dark", 1), dtype=np.float32).mean()) < 0.5 * float(clean_px.mean()))
    check("ảnh mờ khác ảnh gốc", not np.array_equal(np.asarray(make_variant(card, "blur", 1)), np.asarray(card)))

    check("levenshtein kitten/sitting = 3", levenshtein("kitten", "sitting") == 3)
    check("levenshtein chuỗi rỗng", levenshtein("", "abc") == 3 and levenshtein("abc", "") == 3)
    check("cer 1 lỗi trên 4 ký tự = 0,25", cer("1234", "1284") == 0.25)
    try:
        cer("", "x")
        check("cer đáp án rỗng ném ValueError", False)
    except ValueError:
        check("cer đáp án rỗng ném ValueError", True)
    check("đoạn con khớp trọn trong dòng dài", best_substring_distance("099182", "So/No.: 099182839347") == 0)
    check("đoạn con sai một ký tự", best_substring_distance("099182", "099132839347") == 1)
    check("fold bỏ dấu cả Đ", fold("NGUYỄN VĂN ĐỨC") == "NGUYEN VAN DUC")
    check("nfc ghép dấu rời", nfc("Ê") == "Ê" and len(nfc("Ê")) == 1)

    tess = TesseractEngine()
    check("Tesseract từ 5 trở lên", int(tess.version.split(".")[0]) >= 5)
    clean = [s for s in samples if s.variant == "clean"]
    scored = {s.sample_id: score_fields(s.fields, tess.recognize(s.path)) for s in clean}
    check("thẻ sạch: số căn cước đúng nguyên văn ở mọi thẻ", all(sc["id_number"].exact for sc in scored.values()))
    check("thẻ sạch: ngày sinh đúng nguyên văn ở mọi thẻ", all(sc["date_of_birth"].exact for sc in scored.values()))
    check("thẻ sạch: CER trung bình dưới 0,1",
          sum(f.cer for sc in scored.values() for f in sc.values()) / (5 * len(scored)) < 0.1)

    eng_only = TesseractEngine("eng", 6)
    with_vie = sum(score_fields(s.fields, tess.recognize(s.path))["hometown_city"].exact for s in clean)
    without_vie = sum(score_fields(s.fields, eng_only.recognize(s.path))["hometown_city"].exact for s in clean)
    check("gói vie giúp đọc đúng dấu ở quê quán nhiều hơn chỉ eng", with_vie > without_vie)

    blur = [s for s in samples if s.variant == "blur"]
    psm3, psm6 = TesseractEngine("vie+eng", 3), tess
    cer3 = sum(f.cer for s in blur for f in score_fields(s.fields, psm3.recognize(s.path)).values())
    cer6 = sum(f.cer for s in blur for f in score_fields(s.fields, psm6.recognize(s.path)).values())
    check("ảnh mờ: --psm 6 cho CER thấp hơn --psm 3", cer6 < cer3)

    lines = tess.recognize(by_id["s1-rotate"].path)
    width, height = Image.open(by_id["s1-rotate"].path).size
    check("mọi dòng có hộp trong ảnh, confidence 0..1, chữ không rỗng",
          bool(lines) and all(0 <= ln.box[0] < ln.box[2] <= width and 0 <= ln.box[1] < ln.box[3] <= height
                              and 0.0 <= ln.confidence <= 1.0 and ln.text.strip() for ln in lines))

    blank = OUT / "blank.png"
    Image.new("RGB", (400, 300), (255, 255, 255)).save(blank)
    blank_lines = tess.recognize(blank)
    blank_scores = score_fields(samples[0].fields, blank_lines)
    check("ảnh trắng: không có dòng, CER mỗi trường bằng 1, không trường nào đúng",
          blank_lines == [] and all(sc.cer == 1.0 and not sc.exact for sc in blank_scores.values()))

    corrupt = OUT / "corrupt.png"
    corrupt.write_bytes(b"\x89PNG\r\n\x1a\n" + b"khong phai anh")
    for label, path in (("file hỏng", corrupt), ("file không tồn tại", OUT / "khong-co.png")):
        try:
            tess.recognize(path)
            check(f"{label} ném OcrError", False)
        except OcrError:
            check(f"{label} ném OcrError", True)

    saved_module = sys.modules.get("easyocr", ...)
    sys.modules["easyocr"] = None  # type: ignore[assignment]  # giả lập chưa cài easyocr
    try:
        make_engine("easyocr")
        check("thiếu easyocr ném EngineUnavailable có hướng dẫn --extra", False)
    except EngineUnavailable as exc:
        check("thiếu easyocr ném EngineUnavailable có hướng dẫn --extra", "--extra easyocr" in str(exc))
    finally:
        if saved_module is ...:
            del sys.modules["easyocr"]
        else:
            sys.modules["easyocr"] = saved_module
    try:
        make_engine("khong-co")
        check("tên engine lạ ném ValueError", False)
    except ValueError:
        check("tên engine lạ ném ValueError", True)

    fake = {"engine": "tesseract", "info": {}, "python": "3.14.7", "machine": "máy thử", "cpu_count": 1,
            "load_before": 0.0, "load_after": 0.0, "init_seconds": 0.0, "init_cpu_seconds": 0.0, "model_mb": 1.0,
            "images": [{"id": s.sample_id, "seconds": 0.5, "cpu_seconds": 0.5,
                        "lines": [{"text": s.fields["id_number"], "confidence": 0.9, "box": [0, 0, 10, 10]}]}
                       for s in samples]}
    summary = summarize(fake, samples)
    check("tổng hợp giả: id_number đúng 100%, họ tên đúng 0%",
          summary["exact_pct"]["id_number"] == 100 and summary["exact_pct"]["full_name"] == 0)
    report = format_report([fake], samples)
    check("bảng chỉ có engine đã chạy", "tesseract" in report and "easyocr" not in report and "paddle" not in report)
    check("lines_from_json giữ nguyên dòng", lines_from_json(fake["images"][0]["lines"])
          == [OcrLine(samples[0].fields["id_number"], 0.9, (0, 0, 10, 10))])

    for name in failed:
        print(f"SAI: {name}")
    if failed:
        print(f"FAIL {passed}/{total}")
        return 1
    print(f"OK {passed}/{total}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--engine", choices=ENGINE_NAMES, default="tesseract")
    parser.add_argument("--compare", action="store_true", help="chỉ in bảng từ out/results-*.json")
    parser.add_argument("--threshold", type=float, default=0.8, help="ngưỡng confidence của bảng 4 (mặc định 0.8)")
    parser.add_argument("--verify", action="store_true", help="kiểm các điều bài nêu, chỉ cần Tesseract")
    args = parser.parse_args()
    if args.verify:
        return verify()
    if args.compare:
        samples = build_dataset(OUT)
        print(format_report(load_all(OUT, ENGINE_NAMES), samples, args.threshold))
        return 0
    return demo(args.engine)


if __name__ == "__main__":
    sys.exit(main())
