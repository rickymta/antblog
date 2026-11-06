"""Bài 45: đánh giá OCR - CER/WER, bộ test vàng, ngưỡng đưa người duyệt, regression.

Chạy:            uv run --project <lab> <lab>/main.py
Verify:          uv run --project <lab> <lab>/main.py --verify
Ghi baseline mới: uv run --project <lab> <lab>/main.py --update-baseline   (chỉ khi CỐ Ý đổi pipeline)

Cần Tesseract (brew install tesseract tesseract-lang, gói `vie`). Không cần mạng, không cần
hạ tầng. Thẻ là dữ liệu giả; ảnh, manifest và cache OCR nằm trong out/ (gitignore).
"""

import random
import sys
import unicodedata
from collections import Counter

import evaluation as ev
import metrics
from ocr import TesseractMissing, tesseract_version
from postprocess import FieldResult, normalize_text

THRESHOLDS = (0.0, 0.5, 0.7, 0.8, 0.9, 0.95)


def show_metric_definitions() -> dict[str, object]:
    print("[1] CER và WER: định nghĩa trên ví dụ nhỏ")
    out: dict[str, object] = {}
    kitten = metrics.char_errors(["kitten"], ["sitting"])
    print(f"  'kitten' -> 'sitting': S={kitten.substitutions} D={kitten.deletions} I={kitten.insertions} "
          f"N={kitten.reference_chars}, CER {kitten.cer:.3f} (jiwer {metrics.cer(['kitten'], ['sitting']):.3f}, "
          f"tay {metrics.cer_by_hand(['kitten'], ['sitting']):.3f})")
    out["kitten"] = (kitten.cer, metrics.cer_by_hand(["kitten"], ["sitting"]))
    ref, hyp = "NGUYỄN VĂN A", "NGUYEN VAN A"
    print(f"  mất dấu {ref!r} -> {hyp!r}: CER {metrics.cer([ref], [hyp]):.3f}, WER {metrics.wer([ref], [hyp]):.3f}")
    out["diacritic"] = (metrics.cer([ref], [hyp]), metrics.wer([ref], [hyp]))
    decomposed = unicodedata.normalize("NFD", ref)
    print(f"  cùng chữ, NFC so với NFD: {metrics.edit_distance(ref, decomposed)} phép sửa trên {len(ref)} ký tự, "
          f"CER thô {metrics.edit_distance(ref, decomposed) / len(ref):.3f}, "
          f"sau NFC cả hai phía {metrics.cer([ref], [decomposed]):.3f}")
    out["nfc"] = (metrics.edit_distance(ref, decomposed), metrics.cer([ref], [decomposed]))
    refs, hyps = ["099085791907", "D"], ["099085791907", "O"]
    mean_of_cers = (metrics.cer(refs[:1], hyps[:1]) + metrics.cer(refs[1:], hyps[1:])) / 2
    print(f"  gộp 2 trường (12 ký tự đúng, 1 ký tự sai): CER corpus {metrics.cer(refs, hyps):.3f}, "
          f"trung bình hai CER {mean_of_cers:.3f}")
    out["corpus"] = (metrics.cer(refs, hyps), mean_of_cers)
    return out


def show_dataset(items: list[ev.Item]) -> None:
    counts = Counter((it.doc_type, it.variant) for it in items)
    print(f"[2] Bộ test vàng: {len(items)} thẻ, seed {ev.FIRST_SEED}..{ev.FIRST_SEED + len(items) - 1}, "
          f"manifest {(ev.OUT / 'manifest.json').relative_to(ev.OUT.parent)}")
    for doc in ("cccd", "gplx"):
        print(f"  {doc}: " + ", ".join(f"{v} {counts[(doc, v)]}" for v in ("clean", "blur", "rotate", "dark")))
    print(f"  tổng số trường: {sum(len(ev.DOCS[it.doc_type][1](it.seed)) for it in items)}")


def show_by_field(items: list[ev.Item], results: list, pages: list[str]) -> dict[str, object]:
    print("[3] Theo trường (CER của chữ OCR thô của trường; 'đúng' là sau hậu xử lý, tên xét không dấu)")
    raw = ev.cer_by_field(items, results, stage="raw")
    post = ev.cer_by_field(items, results, stage="post")
    wers = ev.wer_by_field(items, results, stage="raw")
    print(f"  {'trường':15} {'N ký tự':>8} {'S':>4} {'D':>4} {'I':>4} {'CER thô':>8} {'WER thô':>8} {'đúng':>9}")
    for name, c in raw.items():
        total = sum(name in r.fields for r in results)
        right = sum(ev.is_correct(name, r.fields[name].value, it.truth[name])
                    for it, r in zip(items, results, strict=True) if name in r.fields)
        print(f"  {name:15} {c.reference_chars:8d} {c.substitutions:4d} {c.deletions:4d} {c.insertions:4d} "
              f"{c.cer:8.4f} {wers[name]:8.4f} {right:4d}/{total:<4d}")
    print("  id_number: chữ OCR thô khác đáp án, và kết quả sau hậu xử lý")
    for it, r in zip(items, results, strict=True):
        f = r.fields.get("id_number")
        if f is not None and (normalize_text(f.raw or "") != it.truth["id_number"] or f.value != it.truth["id_number"]):
            print(f"    {it.doc_type}-{it.seed}-{it.variant:6} đúng {it.truth['id_number']!r} thô {f.raw!r} -> {f.value!r} "
                  f"confidence {f.confidence:.2f} vấn đề={list(f.issues)}")
    strict = ev.cer_by_field(items, results, stage="raw", strict_names=True)["full_name"]
    strict_wer = ev.wer_by_field(items, results, stage="raw", strict_names=True)["full_name"]
    print(f"  CER thô toàn bộ {ev.overall_cer(raw):.4f}; sau hậu xử lý {ev.overall_cer(post):.4f}")
    sub, dele, ins = (sum(getattr(c, k) for c in raw.values()) for k in ("substitutions", "deletions", "insertions"))
    empties = [(it, name) for it, r in zip(items, results, strict=True) for name, f in r.fields.items() if f.raw is None]
    empty_chars = sum(len(ev.fold(it.truth[name]) if name == "full_name" else it.truth[name]) for it, name in empties)
    by_variant_empty = Counter(it.variant for it, _ in empties)
    print(f"  lỗi ký tự: thay {sub}, xóa {dele}, chèn {ins}; {len(empties)} trường không lấy được chữ nào "
          f"(theo ảnh: {dict(by_variant_empty)}) gây {empty_chars}/{dele} ký tự xóa")
    nones = [(it, name) for it, r in zip(items, results, strict=True) for name, f in r.fields.items() if f.value is None]
    none_chars = sum(len(ev.fold(it.truth[name]) if name == "full_name" else it.truth[name]) for it, name in nones)
    print(f"  sau hậu xử lý có {len(nones)} trường value None ({none_chars} ký tự đáp án), CER sau hậu xử lý tính là xóa hết")
    print(f"  tên có dấu: CER {strict.cer:.4f}, WER {strict_wer:.4f} (không dấu: CER {raw['full_name'].cer:.4f}, "
          f"WER {wers['full_name']:.4f})")
    hit, total_fields = ev.page_recall(items, pages)
    empty, present = ev.empty_but_present(items, results, pages)
    print(f"  độ phủ trang (đáp án có trong chữ cả trang, không qua bước tìm trường): {hit}/{total_fields}; "
          f"trong {empty} trường bước tìm trường để trống, {present} có đáp án nằm sẵn trong trang")
    return {"raw": raw, "post": post, "strict_name": strict, "page_hit": hit, "page_total": total_fields}


def show_by_variant(items: list[ev.Item], results: list) -> dict[str, tuple[float, int, int]]:
    print("[4] Theo chất lượng ảnh")
    rows = ev.by_variant(items, results, stage="raw")
    for variant, (cer, right, total) in rows.items():
        print(f"  {variant:7} CER thô {cer:.4f}   trường đúng {right}/{total}")
    return rows


def show_gate(items: list[ev.Item], results: list) -> list[ev.GateStats]:
    print("[5] Ngưỡng confidence để tự nhận (trường không trích được hoặc có vấn đề luôn vào hàng chờ)")
    print("  Đơn vị: 'trường' là một trường; 'thẻ' vào hàng chờ nếu BẤT KỲ trường nào bị cờ (người duyệt mở cả thẻ)")
    print(f"  {'ngưỡng':>6} {'trường vào hàng chờ':>20} {'sai lọt (tự nhận)':>18} {'sai bị chặn':>12} "
          f"{'thẻ vào hàng chờ':>17} {'thẻ tự nhận có sai':>19}")
    stats = []
    for t in THRESHOLDS:
        g = ev.gate_stats(items, results, t)
        stats.append(g)
        print(f"  {t:6.2f} {g.field_review_rate:19.1%} {g.auto_wrong:10d}/{g.auto_fields:<7d} "
              f"{g.wrong_caught:11.1%} {g.card_review_rate:16.1%} {g.auto_cards_with_wrong:12d}/{g.auto_cards}")
    print(f"  Các trường SAI mà vẫn được tự nhận ở ngưỡng {ev.THRESHOLD}:")
    escaped_in_queue = 0
    for it, r in zip(items, results, strict=True):
        card_in_queue = any(ev.field_needs_review(f, ev.THRESHOLD) for f in r.fields.values())
        for name, f in r.fields.items():
            if not ev.is_correct(name, f.value, it.truth[name]) and not ev.field_needs_review(f, ev.THRESHOLD):
                escaped_in_queue += card_in_queue
                print(f"    {it.doc_type}-{it.seed}-{it.variant:6} {name:12} đúng {it.truth[name]!r:16} "
                      f"đọc {f.value!r:16} confidence {f.confidence:.2f}  thẻ đã vào hàng chờ: "
                      f"{'có' if card_in_queue else 'không'}")
    at = ev.gate_stats(items, results, ev.THRESHOLD)
    print(f"  Trong {at.auto_wrong} trường sai lọt, {escaped_in_queue} thuộc thẻ đã vào hàng chờ vì trường khác; "
          f"chỉ {at.auto_cards_with_wrong} thẻ lọt hẳn qua cổng mà có trường sai")
    # Tỉ lệ thẻ vào hàng chờ không do ngưỡng quyết định: xem theo biến thể ảnh và theo lý do.
    queue_by_variant = {v: [0, 0] for v in ("clean", "blur", "rotate", "dark")}
    cards_with_none = cards_with_issue_only = cards_low_only = 0
    clean_min_conf = []
    for it, r in zip(items, results, strict=True):
        flagged = [ev.field_needs_review(f, ev.THRESHOLD) for f in r.fields.values()]
        queue_by_variant[it.variant][1] += 1
        if any(flagged):
            queue_by_variant[it.variant][0] += 1
            if any(f.value is None for f in r.fields.values()):
                cards_with_none += 1
            elif any(f.issues for f in r.fields.values()):
                cards_with_issue_only += 1
            else:
                cards_low_only += 1
        if it.variant == "clean":
            clean_min_conf.append(min(f.confidence for f in r.fields.values()))
    in_queue = sum(v[0] for v in queue_by_variant.values())
    print(f"  Thẻ vào hàng chờ ở ngưỡng {ev.THRESHOLD} theo ảnh: "
          + ", ".join(f"{v} {a}/{b}" for v, (a, b) in queue_by_variant.items()))
    print(f"  Trong {in_queue} thẻ đó: có trường không trích được {cards_with_none}, chỉ có issue {cards_with_issue_only}, "
          f"chỉ confidence thấp {cards_low_only}")
    print(f"  confidence nhỏ nhất trên mỗi thẻ sạch: từ {min(clean_min_conf):.2f} đến {max(clean_min_conf):.2f}")
    return stats


def show_regression(items: list[ev.Item], results: list, pages: list[str]) -> dict[str, object]:
    print("[6] Regression so với baseline.json")
    current = ev.snapshot(items, results, pages)
    baseline = ev.load_baseline()
    env_problems = ev.check_environment(current["environment"], baseline["environment"])
    print(f"  môi trường: {current['environment']['tesseract']}, mô hình vie {current['environment']['traineddata_sha256']['vie']}, "
          f"font {current['environment']['font']['name']} {current['environment']['font']['sha256']} -> "
          f"{'cùng baseline' if not env_problems else 'LỆCH baseline'}")
    for line in env_problems:
        print(f"    LỆCH: {line}")

    def summary(snap: dict[str, object]) -> str:
        return (f"CER thô {snap['cer_raw_overall']}, độ phủ trang {snap['page_recall']}, trường đúng "
                f"{snap['fields_correct']}/{snap['fields_total']}, thẻ vào hàng chờ {snap['card_review_rate']:.0%}, "
                f"sai lọt {snap['auto_wrong_fields']} trường / {snap['auto_wrong_cards']} thẻ")

    print(f"  baseline:     {summary(baseline)}")
    failures = ev.check_regression(current, baseline)
    print(f"  bản hiện tại: {summary(current)} -> {'ĐẠT' if not failures else 'FAIL'}")
    bad = ev.snapshot(items, ev.run_pipeline(items, transform=ev.downscale_half),
                      ev.page_texts(items, transform=ev.downscale_half))
    bad_failures = ev.check_regression(bad, baseline)
    print(f"  thử phá 1 (thu ảnh còn một nửa trước OCR): {summary(bad)}")
    for line in bad_failures:
        print(f"    FAIL: {line}")
    broken = ev.snapshot(items, ev.drop_values(results), pages)
    broken_failures = ev.check_regression(broken, baseline)
    print(f"  thử phá 2 (hậu xử lý trả None cho các thẻ vốn đã vào hàng chờ): {summary(broken)}")
    for line in broken_failures:
        print(f"    FAIL: {line}")
    return {"current": current, "baseline": baseline, "failures": failures, "bad_failures": bad_failures,
            "broken_failures": broken_failures, "env_problems": env_problems}


def demo() -> dict[str, object]:
    print(f"Tesseract: {tesseract_version()}; ngôn ngữ vie+eng, psm 6")
    out: dict[str, object] = {"defs": show_metric_definitions()}
    items = ev.build_dataset()
    show_dataset(items)
    results = ev.run_pipeline(items)
    pages = ev.page_texts(items)
    out.update(items=items, results=results, pages=pages)
    out["by_field"] = show_by_field(items, results, pages)
    out["by_variant"] = show_by_variant(items, results)
    out["gate"] = show_gate(items, results)
    out["regression"] = show_regression(items, results, pages)
    return out


# ---- verify -------------------------------------------------------------------------------------


def verify() -> None:
    out = demo()
    print("\n--verify")
    checks: list[tuple[str, bool]] = []

    def check(name: str, ok: bool) -> None:
        checks.append((name, bool(ok)))

    kitten, tay = out["defs"]["kitten"]
    check("CER 'kitten' -> 'sitting' = 3/6", kitten == 0.5 and tay == 0.5)
    cer_d, wer_d = out["defs"]["diacritic"]
    check("mất dấu: CER 2/12, WER 2/3", abs(cer_d - 2 / 12) < 1e-9 and abs(wer_d - 2 / 3) < 1e-9)
    distance, after = out["defs"]["nfc"]
    check("NFC so NFD lệch 5 phép sửa (Ễ: 3, Ă: 2), sau chuẩn hóa CER = 0", distance == 5 and after == 0.0)
    corpus, mean = out["defs"]["corpus"]
    check("CER corpus 1/13, khác trung bình hai CER 0,5", abs(corpus - 1 / 13) < 1e-9 and mean == 0.5)

    rng = random.Random(45)
    alphabet = "abcdễ 01"
    pairs = [("".join(rng.choices(alphabet, k=rng.randint(1, 12))).strip() or "a",
              "".join(rng.choices(alphabet, k=rng.randint(0, 12)))) for _ in range(300)]
    refs, hyps = [p[0] for p in pairs], [p[1] for p in pairs]
    check("CER tay trùng jiwer trên 300 cặp ngẫu nhiên", abs(metrics.cer_by_hand(refs, hyps) - metrics.cer(refs, hyps)) < 1e-9)
    words_refs = [" ".join(r) or "a" for r in refs]
    words_hyps = [" ".join(h) for h in hyps]
    check("WER tay trùng jiwer trên 300 cặp ngẫu nhiên",
          abs(metrics.wer_by_hand(words_refs, words_hyps) - metrics.wer(words_refs, words_hyps)) < 1e-9)
    counts = metrics.char_errors(refs, hyps)
    check("hits + S + D = tổng ký tự tham chiếu", counts.reference_chars == sum(len(metrics.nfc(r)) for r in refs))

    ok_field = FieldResult("sex", "Nam", "Nam", 0.95)
    check("gate: trường tốt, confidence cao -> tự nhận", not ev.field_needs_review(ok_field, 0.8))
    check("gate: confidence dưới ngưỡng -> hàng chờ", ev.field_needs_review(FieldResult("sex", "Nam", "Nam", 0.79), 0.8))
    check("gate: confidence bằng ngưỡng -> tự nhận", not ev.field_needs_review(FieldResult("sex", "Nam", "Nam", 0.8), 0.8))
    check("gate: có issue -> hàng chờ dù confidence cao",
          ev.field_needs_review(FieldResult("id_number", "1", "1", 0.99, issues=("century_digit_mismatch:0",)), 0.8))
    check("gate: value None -> hàng chờ", ev.field_needs_review(FieldResult("sex", None, None, 0.0), 0.0))

    base = {"cer_raw_by_field": {"a": 0.10, "b": 0.0}, "page_recall": 0.94, "fields_correct": 213, "fields_total": 250,
            "auto_wrong_fields": 4, "auto_wrong_cards": 1, "card_review_rate": 0.40}

    def with_changes(**kw: object) -> dict[str, object]:
        merged = {**base, **kw}
        return merged

    check("regression: giống baseline -> đạt", ev.check_regression(base, base) == [])
    check("regression: CER +0,02 vẫn đạt (đúng biên)",
          ev.check_regression(with_changes(cer_raw_by_field={"a": 0.12, "b": 0.0}), base) == [])
    check("regression: CER +0,03 -> fail",
          len(ev.check_regression(with_changes(cer_raw_by_field={"a": 0.13, "b": 0.0}), base)) == 1)
    check("regression: thêm một trường sai lọt -> fail",
          len(ev.check_regression(with_changes(auto_wrong_fields=5), base)) == 1)
    check("regression: sai lọt giảm -> đạt", ev.check_regression(with_changes(auto_wrong_fields=3), base) == [])
    check("regression: trường đúng giảm 1 -> fail (không dung sai)",
          len(ev.check_regression(with_changes(fields_correct=212), base)) == 1)
    check("regression: trường đúng tăng -> đạt", ev.check_regression(with_changes(fields_correct=214), base) == [])
    check("regression: thêm một thẻ lọt hẳn -> fail", len(ev.check_regression(with_changes(auto_wrong_cards=2), base)) == 1)
    check("regression: độ phủ trang giảm 0,03 -> fail", len(ev.check_regression(with_changes(page_recall=0.91), base)) == 1)
    env = {"tesseract": "t 5", "traineddata_sha256": {"vie": "a"}, "font": {"name": "Arial.ttf", "sha256": "f"}}
    check("môi trường giống nhau -> không báo", ev.check_environment(env, env) == [])
    check("môi trường đổi mô hình ngôn ngữ -> báo riêng",
          len(ev.check_environment({**env, "traineddata_sha256": {"vie": "b"}}, env)) == 1)
    check("regression: hàng chờ +0,11 -> fail",
          len(ev.check_regression(with_changes(card_review_rate=0.51), base)) == 1)

    items, results = out["items"], out["results"]
    check("bộ vàng có 50 thẻ, 250 trường", len(items) == 50 and sum(len(r.fields) for r in results) == 250)
    gates = out["gate"]
    check("ngưỡng tăng thì trường vào hàng chờ không giảm",
          all(a.field_review_rate <= b.field_review_rate + 1e-12 for a, b in zip(gates, gates[1:], strict=False)))
    check("ngưỡng tăng thì sai lọt không tăng",
          all(a.auto_wrong >= b.auto_wrong for a, b in zip(gates, gates[1:], strict=False)))
    check("ngưỡng 0 vẫn có trường vào hàng chờ (trường không trích được)", gates[0].field_review_rate > 0)
    check("môi trường (Tesseract, mô hình vie, font) trùng với lúc ghi baseline", out["regression"]["env_problems"] == [])
    check("so với baseline.json: không có fail", out["regression"]["failures"] == [])
    check("hậu xử lý hỏng ở thẻ vốn đã vào hàng chờ chỉ bị luật «trường đúng» bắt",
          len(out["regression"]["broken_failures"]) == 1 and "số trường đúng" in out["regression"]["broken_failures"][0])
    check("độ phủ trang không thấp hơn số trường đúng", out["by_field"]["page_hit"] >= out["regression"]["current"]["fields_correct"])
    check("thu ảnh còn một nửa bị bắt bởi regression gate", len(out["regression"]["bad_failures"]) >= 3)

    failed = [name for name, ok in checks if not ok]
    for name in failed:
        print(f"SAI: {name}")
    passed = len(checks) - len(failed)
    if failed:
        print(f"FAIL {passed}/{len(checks)}")
        sys.exit(1)
    print(f"OK {passed}/{len(checks)}")


def update_baseline() -> None:
    items = ev.build_dataset()
    snap = ev.snapshot(items, ev.run_pipeline(items), ev.page_texts(items))
    ev.save_baseline(snap)
    print(f"đã ghi {ev.BASELINE_FILE}: {snap}")


if __name__ == "__main__":
    try:
        if "--update-baseline" in sys.argv:
            update_baseline()
        elif "--verify" in sys.argv:
            verify()
        else:
            demo()
    except FileNotFoundError as exc:
        if exc.filename == str(ev.BASELINE_FILE):
            print("FAIL: chưa có baseline.json (chạy --update-baseline một lần)")
        else:
            raise
        sys.exit(1)
    except TesseractMissing as exc:
        print(f"FAIL: {exc}")
        sys.exit(1)
