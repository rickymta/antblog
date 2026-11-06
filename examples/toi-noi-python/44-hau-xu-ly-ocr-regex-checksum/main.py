"""Bài 44: hậu xử lý OCR - chuẩn hóa Unicode, regex có tên, sửa lỗi theo ngữ cảnh, kiểm cấu trúc.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Cần Tesseract (brew install tesseract tesseract-lang, gói `vie`). Không cần mạng, không cần
hạ tầng. Mọi thẻ là dữ liệu giả sinh bằng Pillow; ảnh và cache OCR nằm trong out/ (gitignore).
"""

import os
import sys
import unicodedata
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from datetime import date

from rapidfuzz import fuzz, process

from ocr import TesseractMissing, Word, ocr_words, tesseract_version
from postprocess import (ACCEPTED_PREFIXES, CITIES, FUZZY_CITY_MARGIN, FUZZY_CITY_MIN, NAME_RE, CardResult,
                         check_id_structure, fix_digits, fold, luhn_valid, match_city,
                         normalize_text, parse_card, parse_class, parse_date, parse_id_number,
                         parse_name)
from synthetic_card import (VARIANTS, degrade, make_card_fields, make_id_card, make_license_card,
                            make_license_fields)

TODAY = date(2026, 11, 6)  # cố định để kết quả không phụ thuộc ngày chạy
SEEDS = range(1, 9)  # 8 seed x 2 loại giấy tờ x 4 biến thể = 64 ảnh
DOCS = {"cccd": (make_id_card, make_card_fields), "gplx": (make_license_card, make_license_fields)}


@dataclass(frozen=True)
class Sample:
    doc_type: str
    seed: int
    variant: str
    truth: dict[str, str]
    words: list[Word]


def build_samples() -> list[Sample]:
    """Sinh 64 ảnh, OCR bằng Tesseract (song song vì mỗi lần gọi là một tiến trình riêng)."""
    jobs = [(doc, seed, variant) for doc in DOCS for seed in SEEDS for variant in VARIANTS]

    def run(job: tuple[str, int, str]) -> Sample:
        doc, seed, variant = job
        make_card, make_fields = DOCS[doc]
        image = degrade(make_card(seed), variant, seed)
        return Sample(doc, seed, variant, make_fields(seed), ocr_words(image))

    with ThreadPoolExecutor(max_workers=os.process_cpu_count() or 1) as pool:
        return list(pool.map(run, jobs))


# ---- Các phần của bài ---------------------------------------------------------------------------


def show_tsv_excerpt(samples: list[Sample]) -> None:
    """Vài từ của thẻ seed 7 sạch, đúng như `ocr.py` đọc ra từ TSV của Tesseract."""
    sample = next(s for s in samples if (s.doc_type, s.seed, s.variant) == ("cccd", 7, "clean"))
    wanted = {sample.truth["id_number"], sample.truth["date_of_birth"], sample.truth["sex"]}
    print("[0] Từ Tesseract nhận ra (cccd-7-clean, trích): dòng (block, par, line), khung, conf, chữ")
    print(f"  {'dòng':10} {'left':>5} {'top':>5} {'rộng':>5} {'cao':>4} {'conf':>10}  chữ")
    for w in sample.words:
        if w.text in wanted:
            print(f"  {w.line!s:10} {w.left:5d} {w.top:5d} {w.width:5d} {w.height:4d} {w.conf * 100:10.6f}  {w.text}")


def show_one_card(samples: list[Sample]) -> dict[str, object]:
    """Một thẻ nghiêng từ đầu đến cuối: chữ OCR thô, giá trị sau hậu xử lý, cờ."""
    sample = next(s for s in samples if (s.doc_type, s.seed, s.variant) == ("cccd", 7, "rotate"))
    result = parse_card(sample.words, doc_type=sample.doc_type, today=TODAY)
    print(f"[1] Một thẻ từ đầu đến cuối: {sample.doc_type}-{sample.seed}-{sample.variant}")
    for name, field in result.fields.items():
        print(f"  {name:14} thô {field.raw!r:16} -> {field.value!r:16} confidence {field.confidence:.2f}"
              f"  sửa={list(field.corrections)} vấn đề={list(field.issues)}")
    return {"card_ok": result.ok, "fields": result.fields, "truth": sample.truth}


def show_unicode() -> dict[str, bool]:
    print("[2] Unicode: cùng một chữ, hai dãy mã")
    composed = "NGUYỄN"
    decomposed = unicodedata.normalize("NFD", composed)
    print(f"  NFC: {len(composed)} mã, NFD: {len(decomposed)} mã, bằng nhau: {composed == decomposed}")
    nfd_name = unicodedata.normalize("NFD", "NGUYỄN VĂN A")
    print(f"  NAME_RE khớp NFC: {NAME_RE.fullmatch('NGUYỄN VĂN A') is not None}, "
          f"khớp NFD: {NAME_RE.fullmatch(nfd_name) is not None}")
    messy = "  Nguy\u1ec5n\u00a0V\u0103n\u200b  A  "  # NBSP và ký tự rộng bằng 0 lẫn trong chuỗi
    print(f"  normalize_text({messy!r}) -> {normalize_text(messy)!r}")
    print(f"  fold('Đà Nẵng') -> {fold('Đà Nẵng')!r}")
    return {
        "nfc_ne_nfd": composed != decomposed,
        "regex_fails_on_nfd": NAME_RE.fullmatch(nfd_name) is None,
        "regex_ok_after_normalize": NAME_RE.fullmatch(normalize_text(nfd_name)) is not None,
        "normalize_messy": normalize_text(messy) == "Nguyễn Văn A",
        "fold_dj": fold("Đà Nẵng") == "da nang",
    }


def show_context_fixes() -> dict[str, bool]:
    print("[3] Sửa ký tự dễ nhầm theo ngữ cảnh trường")
    cases = [
        ("id_number", "O99O9OI59O83", parse_id_number),
        ("id_number", "0990 9015 9083", parse_id_number),
        ("date_of_birth", "l3-O3-199O", parse_date),
        ("date_of_birth", "13 / 03 / 1 990", parse_date),
        ("date_of_birth", "48/10/1985", parse_date),
        ("full_name", "NGUY3N V4N A", parse_name),
        ("full_name", "NGUYEN VAN5", parse_name),
        ("full_name", "NGUY0N V1N A", parse_name),
        ("license_class", "Al", parse_class),
        ("license_class", "Cc", parse_class),
    ]
    results = {}
    for field, raw, parser in cases:
        value, notes, issues = parser(raw)
        print(f"  {field:14} {raw!r:18} -> {value!r:16} sửa={notes} vấn đề={issues}")
        results[f"{field}:{raw}"] = value is not None and not issues
    return results


def show_id_structure() -> dict[str, object]:
    print("[4] Kiểm cấu trúc số CCCD giả (không phải checksum)")
    good = "099085791907"  # nam, sinh 18/10/1985: thế kỷ 20 nam -> 0, năm 85
    print(f"  {good}, nam, 18/10/1985 -> {check_id_structure(good, '18/10/1985', 'Nam')}")
    print(f"  {good}, nữ,  18/10/1985 -> {check_id_structure(good, '18/10/1985', 'Nữ')}")
    print(f"  {good}, nam, 18/10/1995 -> {check_id_structure(good, '18/10/1995', 'Nam')}")
    print(f"  {good}, nam, 18/10/2005 -> {check_id_structure(good, '18/10/2005', 'Nam')}")
    # Thay MỖI chữ số bằng 9 chữ số khác: 12 vị trí x 9 = 108 phép sai một chữ số.
    detected_by_position = [0] * 12
    for pos in range(12):
        for digit in "0123456789":
            if digit == good[pos]:
                continue
            wrong = good[:pos] + digit + good[pos + 1:]
            if check_id_structure(wrong, "18/10/1985", "Nam"):
                detected_by_position[pos] += 1
    total = sum(detected_by_position)
    print(f"  sai một chữ số: bắt được {total}/108; theo vị trí 1..12 = {detected_by_position}")
    by_list = sum(detected_by_position[:3])
    by_cross = sum(detected_by_position[3:6])
    print(f"  trong {total}: {by_list} do mã tỉnh nằm ngoài danh sách cho phép {sorted(ACCEPTED_PREFIXES)}, "
          f"{by_cross} do đối chiếu giới tính và năm sinh (giả định hai thông tin này đọc đúng)")
    valid_card, invalid_card = "4111111111111111", "4111111111111112"
    print(f"  Luhn (số thẻ thanh toán thử): {valid_card} -> {luhn_valid(valid_card)}, "
          f"{invalid_card} -> {luhn_valid(invalid_card)}")
    return {"detected_total": total, "by_position": detected_by_position, "by_list": by_list, "by_cross": by_cross,
            "good_ok": check_id_structure(good, "18/10/1985", "Nam") == [],
            "luhn": luhn_valid(valid_card) and not luhn_valid(invalid_card),
            "sex_mismatch": check_id_structure(good, "18/10/1985", "Nữ") == ["century_digit_mismatch:0"],
            "century_mismatch": check_id_structure(good, "18/10/2005", "Nam")
            == ["century_digit_mismatch:0", "year_digits_mismatch:85"]}


def show_fuzzy() -> dict[str, object]:
    print(f"[5] rapidfuzz với danh sách đóng (tối thiểu {FUZZY_CITY_MIN:.0f} điểm, hơn hạng nhì "
          f"{FUZZY_CITY_MARGIN:.0f} điểm)")
    out: dict[str, object] = {}
    for text in ("CAN THO", "Da Nang", "TP HCM", "Nha Trag", "Ha Nam", "Ho Chi Minh"):
        city, score, second = match_city(text)
        print(f"  {text!r:14} -> {city!r:12} điểm {score:5.1f}, hạng nhì {second:5.1f}")
        out[text] = city
    # Nếu bỏ ngưỡng và biên: 'Ha Nam' (địa danh khác, không có trong danh sách) bị gán nhầm.
    best = process.extractOne(fold("Ha Nam"), {c: fold(c) for c in CITIES}, scorer=fuzz.ratio)
    print(f"  không có ngưỡng: 'Ha Nam' -> {best[2]!r} ({best[1]:.1f} điểm)")
    out["no_threshold_wrong"] = best[2]
    typed = "Lê Văn Cường"
    for ocr_name in ("LE VAN CUONG", "LE VAN C", "TRAN THI B"):
        print(f"  tên OCR {ocr_name!r} so với hồ sơ {typed!r}: {fuzz.ratio(fold(ocr_name), fold(typed)):.1f}")
    out["name_match"] = (round(fuzz.ratio(fold("LE VAN CUONG"), fold(typed)), 1),
                         round(fuzz.ratio(fold("TRAN THI B"), fold(typed)), 1))
    return out


def same(name: str, value: str | None, truth: str) -> bool:
    """Tên so không dấu: Tesseract với font này làm mất dấu chữ hoa (xem bài), và so tên với hồ sơ
    cũng nên không dấu. Các trường khác phải khớp nguyên văn."""
    if value is None:
        return False
    return fold(value) == fold(truth) if name == "full_name" else value == truth


def classify(name: str, field_value: str | None, issues: tuple[str, ...], truth: str) -> str:
    """correct: đúng; flagged: sai nhưng có vấn đề/thiếu giá trị (người duyệt thấy);
    silent: sai mà KHÔNG có vấn đề nào, đây là loại nguy hiểm."""
    if same(name, field_value, truth):
        return "correct" if not issues else "correct_flagged"
    return "flagged" if (field_value is None or issues) else "silent"


def evaluate(samples: list[Sample]) -> dict[str, object]:
    parsed: list[tuple[Sample, CardResult]] = [
        (s, parse_card(s.words, doc_type=s.doc_type, today=TODAY)) for s in samples]
    raw_correct = post_correct = total = 0
    by_variant: dict[str, list[int]] = {v: [0, 0, 0] for v in VARIANTS}  # raw, post, tổng
    kinds: Counter[str] = Counter()
    silent: list[tuple[str, str, str, str, float]] = []
    name_exact = name_total = 0
    for sample, result in parsed:
        for name, field in result.fields.items():
            truth = sample.truth[name]
            total += 1
            by_variant[sample.variant][2] += 1
            if name == "full_name":
                name_total += 1
                name_exact += field.value == truth
            if field.raw is not None and same(name, normalize_text(field.raw), truth):
                raw_correct += 1
                by_variant[sample.variant][0] += 1
            if same(name, field.value, truth):
                post_correct += 1
                by_variant[sample.variant][1] += 1
            kind = classify(name, field.value, field.issues, truth)
            kinds[kind] += 1
            if kind == "silent":
                silent.append((f"{sample.doc_type}-{sample.seed}-{sample.variant}", name, truth,
                               field.value or "", field.confidence))
    return {"parsed": parsed, "total": total, "raw_correct": raw_correct, "post_correct": post_correct,
            "by_variant": by_variant, "kinds": kinds, "silent": silent,
            "name_exact": name_exact, "name_total": name_total}


def show_batch(samples: list[Sample]) -> dict[str, object]:
    print(f"[6] Cả bộ: {len(samples)} ảnh = {len(SEEDS)} seed x {len(DOCS)} loại giấy tờ x {len(VARIANTS)} biến thể")
    ev = evaluate(samples)
    total = ev["total"]
    print(f"  trường đúng (tên xét không dấu): chỉ nối từ OCR {ev['raw_correct']}/{total}, "
          f"sau hậu xử lý {ev['post_correct']}/{total}")
    print("  theo biến thể (chỉ nối từ -> hậu xử lý, trên tổng):")
    for variant, (raw, post, count) in ev["by_variant"].items():
        print(f"    {variant:7} {raw:3} -> {post:3} / {count}")
    print(f"  tên đúng NGUYÊN VĂN (có dấu): {ev['name_exact']}/{ev['name_total']}")
    rescued: Counter[str] = Counter()
    broken = 0
    for sample, result in ev["parsed"]:
        for name, field in result.fields.items():
            raw_ok = field.raw is not None and same(name, normalize_text(field.raw), sample.truth[name])
            post_ok = same(name, field.value, sample.truth[name])
            broken += raw_ok and not post_ok
            if post_ok and not raw_ok:
                first = field.corrections[0] if field.corrections else ""
                rescued["fuzzy với danh sách đóng" if first.startswith("fuzzy:")
                        else "đổi ký tự dễ nhầm" if "->" in first
                        else "bỏ dấu cách chèn thừa"] += 1
    print(f"  trường được hậu xử lý cứu (thô sai, sau đúng): {sum(rescued.values())} = {dict(rescued)}; "
          f"trường bị hậu xử lý làm hỏng: {broken}")
    scans = [(sample.truth[name], field.value) for sample, result in ev["parsed"]
             for name, field in result.fields.items() if "scan_without_label" in field.corrections]
    scan_ok = sum(truth == value for truth, value in scans)
    print(f"  quét cả trang tìm số 12 chữ số khi mất nhãn: {len(scans)} lần, đúng {scan_ok} lần")
    ev["scan_total"], ev["scan_ok"] = len(scans), scan_ok
    ev["rescued"], ev["broken"] = rescued, broken
    kinds = ev["kinds"]
    print(f"  phân loại trường: đúng không cờ {kinds['correct']}, đúng có cờ {kinds['correct_flagged']}, "
          f"sai có cờ/thiếu {kinds['flagged']}, sai KHÔNG cờ {kinds['silent']}")
    silent_by_field = Counter(name for _, name, _, _, _ in ev["silent"])
    print(f"  sai không cờ theo trường: {dict(silent_by_field)}")
    for case, name, truth, value, conf in ev["silent"]:
        print(f"    {case:14} {name:10} đúng {truth!r:14} đọc {value!r:16} confidence {conf:.2f}")
    low = sum(1 for *_, conf in ev["silent"] if conf < 0.8)
    print(f"  trong số đó confidence < 0,80: {low}/{len(ev['silent'])}")
    return ev


def show_confidence(ev: dict[str, object]) -> dict[str, object]:
    print("[7] Confidence của trường = nhỏ nhất của các từ")
    best = None
    for sample, result in ev["parsed"]:
        for name, field in result.fields.items():
            if len(field.word_confs) >= 3 and field.value is not None:
                spread = max(field.word_confs) - min(field.word_confs)
                if best is None or spread > best[0]:
                    best = (spread, sample, name, field)
    assert best is not None
    _, sample, name, field = best
    mean = sum(field.word_confs) / len(field.word_confs)
    print(f"  {sample.doc_type}-{sample.seed}-{sample.variant} {name}: {field.raw!r}")
    print(f"    confidence từng từ {[round(c, 2) for c in field.word_confs]}")
    print(f"    min {min(field.word_confs):.2f}, trung bình {mean:.2f}")
    return {"min_le_mean": min(field.word_confs) <= mean, "min_is_field": field.confidence == min(field.word_confs)}


def demo() -> dict[str, object]:
    print(f"Tesseract: {tesseract_version()}; ngôn ngữ vie+eng, psm 6; Python {sys.version.split()[0]}")
    out: dict[str, object] = {}
    samples = build_samples()
    show_tsv_excerpt(samples)
    out["one_card"] = show_one_card(samples)
    out["unicode"] = show_unicode()
    out["fixes"] = show_context_fixes()
    out["structure"] = show_id_structure()
    out["fuzzy"] = show_fuzzy()
    ev = show_batch(samples)
    out["batch"] = ev
    out["confidence"] = show_confidence(ev)
    return out


# ---- verify -------------------------------------------------------------------------------------


def verify() -> None:
    out = demo()
    print("\n--verify")
    checks: list[tuple[str, bool]] = []

    def check(name: str, ok: bool) -> None:
        checks.append((name, bool(ok)))

    for key, ok in out["unicode"].items():
        check(f"unicode: {key}", ok)
    expect_fix = {
        "id_number:O99O9OI59O83": True, "id_number:0990 9015 9083": True,
        "date_of_birth:l3-O3-199O": True, "date_of_birth:13 / 03 / 1 990": True,
        "date_of_birth:48/10/1985": False,  # ngày không tồn tại phải bị cờ
        "full_name:NGUY3N V4N A": False,  # '3' và '4' không có bảng ngược: để người duyệt
        "full_name:NGUYEN VAN5": False,  # '5' ở cuối từ, chỉ có chữ cái bên trái: không đoán thành 'S'
        "full_name:NGUY0N V1N A": True,  # '0' và '1' nằm giữa hai chữ cái: đổi thành 'O' và 'I'
        "license_class:Al": True, "license_class:Cc": False,
    }
    for key, expected in expect_fix.items():
        check(f"sửa ngữ cảnh: {key} -> ok={expected}", out["fixes"][key] == expected)
    check("O99O9OI59O83 đọc lại đúng 099090159083", parse_id_number("O99O9OI59O83")[0] == "099090159083")
    check("VAN5 không bị đổi thành VANS: ra digit_in_name", parse_name("NGUYEN VAN5") == (None, [], ["digit_in_name"]))
    check("'NGUY0N V1N A' đọc lại 'NGUYON VIN A'", parse_name("NGUY0N V1N A")[0] == "NGUYON VIN A")
    check("fix_digits không đụng chữ số có sẵn", fix_digits("0123456789") == ("0123456789", []))
    structure = out["structure"]
    check("cấu trúc: số hợp lệ không bị cờ", structure["good_ok"])
    check("cấu trúc: sai giới tính bị cờ", structure["sex_mismatch"])
    check("cấu trúc: sai thế kỷ/năm bị cờ", structure["century_mismatch"])
    check("cấu trúc: sai một chữ số bắt được đúng 54/108", structure["detected_total"] == 54)
    check("cấu trúc: 6 chữ số cuối không bao giờ bị bắt",
          structure["by_position"][6:] == [0] * 6 and all(n == 9 for n in structure["by_position"][:6]))
    check("cấu trúc: 27 phép bắt nhờ danh sách mã tỉnh, 27 nhờ đối chiếu thẻ",
          (structure["by_list"], structure["by_cross"]) == (27, 27))
    check("Luhn nhận số thử đúng, loại số sai", structure["luhn"])
    check("mã tỉnh chấp nhận trong lab chỉ là 099", ACCEPTED_PREFIXES == {"099"})
    fuzzy = out["fuzzy"]
    check("fuzzy: 'CAN THO' -> 'Cần Thơ'", fuzzy["CAN THO"] == "Cần Thơ")
    check("fuzzy: 'Da Nang' -> 'Đà Nẵng'", fuzzy["Da Nang"] == "Đà Nẵng")
    check("fuzzy: 'TP HCM' -> 'TP.HCM'", fuzzy["TP HCM"] == "TP.HCM")
    check("fuzzy: 'Ha Nam' bị từ chối nhờ ngưỡng", fuzzy["Ha Nam"] is None)
    check("fuzzy: không ngưỡng thì 'Ha Nam' bị gán nhầm", fuzzy["no_threshold_wrong"] == "Hà Nội")
    check("fuzzy: tên OCR khớp hồ sơ cao hơn tên lạ", fuzzy["name_match"][0] > fuzzy["name_match"][1] + 30)
    batch = out["batch"]
    check("batch: hậu xử lý không làm tụt số trường đúng", batch["post_correct"] >= batch["raw_correct"])
    check("batch: có ảnh sạch đọc đúng số CCCD",
          any(f.value == s.truth[n] for s, r in batch["parsed"] if s.variant == "clean"
              for n, f in r.fields.items() if n == "id_number"))
    check("batch: không trường nào đúng ở bản thô mà sai sau hậu xử lý", batch["broken"] == 0)
    check("batch: quét cả trang tìm số được dùng và đa số đúng", 0 < batch["scan_ok"] <= batch["scan_total"] and batch["scan_ok"] * 2 > batch["scan_total"])
    check("batch: số trường theo phân loại cộng đủ tổng", sum(batch["kinds"].values()) == batch["total"])
    check("batch: parse_card không nâng ngoại lệ trên ảnh xấu (dark/blur/rotate)", batch["total"] == 320)
    check("confidence trường là min của các từ", out["confidence"]["min_is_field"])

    failed = [name for name, ok in checks if not ok]
    for name in failed:
        print(f"SAI: {name}")
    passed = len(checks) - len(failed)
    if failed:
        print(f"FAIL {passed}/{len(checks)}")
        sys.exit(1)
    print(f"OK {passed}/{len(checks)}")


if __name__ == "__main__":
    try:
        if "--verify" in sys.argv:
            verify()
        else:
            demo()
    except TesseractMissing as exc:
        print(f"FAIL: {exc}")
        sys.exit(1)
