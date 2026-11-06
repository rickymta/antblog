"""Bộ test vàng, chạy OCR, tính CER/WER theo trường, ngưỡng đưa người duyệt và so với baseline.

Bộ test vàng là bộ thẻ GIẢ sinh bằng code: đáp án (`truth`) là đúng những chữ đã vẽ lên thẻ nên
không cần gán nhãn tay. Seed của bộ này (từ 1000) khác seed đã dùng khi chỉnh luật hậu xử lý ở
bài 44 (1 đến 8), để số đo không bị «học thuộc». Bộ này cũng KHÁC bộ ảnh của lab 41 và 42 (seed,
GPLX, tham số biến thể), nên số CER ở đây không so chéo với các bài đó.
"""

import dataclasses
import hashlib
import json
import os
from collections import defaultdict
from collections.abc import Callable
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from datetime import date
from pathlib import Path

from PIL import Image

import metrics
from ocr import Word, ocr_words, tesseract_version, traineddata_hashes
from postprocess import CardResult, FieldResult, fold, normalize_text, parse_card
from synthetic_card import (VARIANTS, degrade, font_path, make_card_fields, make_id_card,
                            make_license_card, make_license_fields)

OUT = Path(__file__).parent / "out"
BASELINE_FILE = Path(__file__).parent / "baseline.json"
TODAY = date(2026, 11, 6)
FIRST_SEED = 1000
DEFAULT_N = 50
THRESHOLD = 0.8  # confidence tối thiểu để một trường được tự nhận (xem bài về cách chọn)
MAX_CER_INCREASE = 0.02  # regression: CER một trường tăng quá 2 điểm phần trăm là fail
MAX_REVIEW_RATE_INCREASE = 0.10

DOCS: dict[str, tuple[Callable[[int], Image.Image], Callable[[int], dict[str, str]]]] = {
    "cccd": (make_id_card, make_card_fields),
    "gplx": (make_license_card, make_license_fields),
}


@dataclass(frozen=True)
class Item:
    doc_type: str
    seed: int
    variant: str
    image: str  # đường dẫn tương đối so với out/
    truth: dict[str, str]


def build_dataset(n: int = DEFAULT_N) -> list[Item]:
    """Sinh n thẻ (xen kẽ CCCD và GPLX, quay vòng 4 biến thể), ghi ảnh và out/manifest.json
    (đường dẫn ảnh, loại giấy tờ, seed, biến thể và đáp án của từng thẻ)."""
    (OUT / "images").mkdir(parents=True, exist_ok=True)
    items = []
    for i in range(n):
        doc = "cccd" if i % 2 == 0 else "gplx"
        variant = VARIANTS[(i // 2) % len(VARIANTS)]
        seed = FIRST_SEED + i
        make_card, make_fields = DOCS[doc]
        relative = f"images/{doc}-{seed}-{variant}.png"
        degrade(make_card(seed), variant, seed).save(OUT / relative)
        items.append(Item(doc, seed, variant, relative, make_fields(seed)))
    manifest = [{"image": it.image, "doc_type": it.doc_type, "seed": it.seed, "variant": it.variant,
                 "truth": it.truth} for it in items]
    (OUT / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=1), encoding="utf-8")
    return items


def run_pipeline(items: list[Item], *, transform: Callable[[Image.Image], Image.Image] | None = None,
                 lang: str = "vie+eng", psm: int = 6) -> list[CardResult]:
    """OCR rồi hậu xử lý từng ảnh. `transform` cho phép chen một bước tiền xử lý (hoặc làm hỏng
    có chủ ý) trước OCR; mỗi lần gọi Tesseract là một tiến trình riêng nên chạy song song được."""

    def one(item: Item) -> CardResult:
        return parse_card(_words(item, transform, lang, psm), doc_type=item.doc_type, today=TODAY)

    with ThreadPoolExecutor(max_workers=os.process_cpu_count() or 1) as pool:
        return list(pool.map(one, items))


def _words(item: Item, transform: Callable[[Image.Image], Image.Image] | None, lang: str, psm: int) -> list[Word]:
    with Image.open(OUT / item.image) as opened:
        image = opened.convert("RGB")
    if transform is not None:
        image = transform(image)
    return ocr_words(image, lang=lang, psm=psm)


def page_texts(items: list[Item], *, transform: Callable[[Image.Image], Image.Image] | None = None,
               lang: str = "vie+eng", psm: int = 6) -> list[str]:
    """Chữ của CẢ TRANG (mọi từ Tesseract đọc được), không qua bước tìm trường. Dùng cache OCR
    nên gần như không tốn thêm thời gian sau `run_pipeline`."""
    with ThreadPoolExecutor(max_workers=os.process_cpu_count() or 1) as pool:
        return list(pool.map(lambda it: " ".join(w.text for w in _words(it, transform, lang, psm)), items))


def _squash(text: str, *, fold_it: bool) -> str:
    text = fold(text) if fold_it else metrics.nfc(text)
    return "".join(text.split())


def page_recall(items: list[Item], pages: list[str]) -> tuple[int, int]:
    """Độ phủ trang, KHÔNG phụ thuộc bước tìm trường: đáp án của trường có xuất hiện (bỏ khoảng
    trắng; tên so không dấu) ở đâu đó trong chữ cả trang không. Trả (số trường có, tổng trường).
    Chưa kiểm đúng vị trí, nên là cận trên của những gì bước tìm trường có thể lấy được."""
    hit = total = 0
    for item, page in zip(items, pages, strict=True):
        for name, truth in item.truth.items():
            total += 1
            hit += _squash(truth, fold_it=name == "full_name") in _squash(page, fold_it=name == "full_name")
    return hit, total


def empty_but_present(items: list[Item], results: list[CardResult], pages: list[str]) -> tuple[int, int]:
    """Trong các trường bước tìm trường để trống (raw None), bao nhiêu có đáp án nằm sẵn trong
    chữ cả trang? Trả (số trường trống, số trường trống mà đáp án có trong trang)."""
    empty = present = 0
    for item, result, page in zip(items, results, pages, strict=True):
        for name, field in result.fields.items():
            if field.raw is None:
                empty += 1
                present += _squash(item.truth[name], fold_it=name == "full_name") in _squash(page, fold_it=name == "full_name")
    return empty, present


def drop_values(results: list[CardResult], *, threshold: float = 0.8) -> list[CardResult]:
    """Làm hỏng HẬU XỬ LÝ có chủ ý (không đụng OCR): bỏ `value` của mọi trường ở các thẻ VỐN ĐÃ vào
    hàng chờ, như một regex sửa sai làm parser trả None. Chữ OCR thô giữ nguyên, tỉ lệ thẻ vào hàng
    chờ không đổi, nên CER thô và các luật về cổng không thấy gì: chỉ số trường đúng tụt."""
    out = []
    for result in results:
        if any(field_needs_review(f, threshold) for f in result.fields.values()):
            fields = {n: dataclasses.replace(f, value=None) for n, f in result.fields.items()}
            result = dataclasses.replace(result, fields=fields)
        out.append(result)
    return out


# ---- Đúng/sai và CER theo trường -----------------------------------------------------------------


def is_correct(name: str, value: str | None, truth: str) -> bool:
    """Trường đúng khi giá trị khớp đáp án. Tên so không dấu (Tesseract với font này mất dấu chữ
    hoa); mọi trường khác phải khớp nguyên văn: số sai một chữ số là sai cả số."""
    if value is None:
        return False
    return fold(value) == fold(truth) if name == "full_name" else value == truth


def hypothesis(field: FieldResult, *, stage: str) -> str:
    """Chuỗi đem so với đáp án. stage='raw': các từ OCR nối lại, chưa hậu xử lý;
    stage='post': giá trị sau hậu xử lý, rỗng nếu không trích được (tính là xóa hết)."""
    if stage == "raw":
        return normalize_text(field.raw or "")
    return field.value or ""


def cer_by_field(items: list[Item], results: list[CardResult], *, stage: str,
                 strict_names: bool = False) -> dict[str, metrics.ErrorCounts]:
    """CER corpus-level theo tên trường. Tên mặc định so không dấu (như is_correct);
    strict_names=True so có dấu để thấy phần lỗi do mất dấu."""
    refs: dict[str, list[str]] = defaultdict(list)
    hyps: dict[str, list[str]] = defaultdict(list)
    for item, result in zip(items, results, strict=True):
        for name, field in result.fields.items():
            ref, hyp = item.truth[name], hypothesis(field, stage=stage)
            if name == "full_name" and not strict_names:
                ref, hyp = fold(ref), fold(hyp)
            refs[name].append(ref)
            hyps[name].append(hyp)
    return {name: metrics.char_errors(refs[name], hyps[name]) for name in sorted(refs)}


def wer_by_field(items: list[Item], results: list[CardResult], *, stage: str,
                 strict_names: bool = False) -> dict[str, float]:
    """WER corpus-level theo tên trường (đơn vị là từ). Trường một từ thì WER là tỉ lệ trường sai."""
    refs: dict[str, list[str]] = defaultdict(list)
    hyps: dict[str, list[str]] = defaultdict(list)
    for item, result in zip(items, results, strict=True):
        for name, field in result.fields.items():
            ref, hyp = item.truth[name], hypothesis(field, stage=stage)
            if name == "full_name" and not strict_names:
                ref, hyp = fold(ref), fold(hyp)
            refs[name].append(ref)
            hyps[name].append(hyp)
    return {name: metrics.wer(refs[name], hyps[name]) for name in sorted(refs)}


def overall_cer(counts: dict[str, metrics.ErrorCounts]) -> float:
    errors = sum(c.substitutions + c.deletions + c.insertions for c in counts.values())
    return errors / sum(c.reference_chars for c in counts.values())


def by_variant(items: list[Item], results: list[CardResult], *, stage: str) -> dict[str, tuple[float, int, int]]:
    """Theo biến thể ảnh: (CER, số trường đúng sau hậu xử lý, tổng trường)."""
    out = {}
    for variant in VARIANTS:
        picked = [(it, r) for it, r in zip(items, results, strict=True) if it.variant == variant]
        counts = cer_by_field([p[0] for p in picked], [p[1] for p in picked], stage=stage)
        total = sum(len(r.fields) for _, r in picked)
        right = sum(is_correct(n, f.value, it.truth[n]) for it, r in picked for n, f in r.fields.items())
        out[variant] = (overall_cer(counts), right, total)
    return out


# ---- Ngưỡng đưa người duyệt ----------------------------------------------------------------------


def field_needs_review(field: FieldResult, threshold: float) -> bool:
    """Không dùng đáp án: chỉ thứ có lúc chạy thật. Trường vào hàng chờ nếu không trích được,
    có vấn đề từ bước kiểm tra, hoặc confidence (min của các từ) dưới ngưỡng."""
    return field.value is None or bool(field.issues) or field.confidence < threshold


@dataclass(frozen=True)
class GateStats:
    threshold: float
    fields: int
    auto_fields: int
    auto_wrong: int  # trường SAI mà vẫn được tự nhận: loại lỗi phải về 0 hoặc gần 0
    wrong: int  # tổng trường sai
    cards: int
    auto_cards: int  # thẻ mà MỌI trường đều được tự nhận
    auto_cards_with_wrong: int

    @property
    def field_review_rate(self) -> float:
        return 1 - self.auto_fields / self.fields

    @property
    def card_review_rate(self) -> float:
        return 1 - self.auto_cards / self.cards

    @property
    def wrong_caught(self) -> float:
        return 1 - self.auto_wrong / self.wrong if self.wrong else 1.0


def gate_stats(items: list[Item], results: list[CardResult], threshold: float) -> GateStats:
    fields = auto = auto_wrong = wrong = auto_cards = auto_cards_wrong = 0
    for item, result in zip(items, results, strict=True):
        card_auto = True
        card_wrong = False
        for name, field in result.fields.items():
            fields += 1
            bad = not is_correct(name, field.value, item.truth[name])
            review = field_needs_review(field, threshold)
            wrong += bad
            if review:
                card_auto = False
            else:
                auto += 1
                auto_wrong += bad
                card_wrong = card_wrong or bad
        if card_auto:
            auto_cards += 1
            auto_cards_wrong += card_wrong
    return GateStats(threshold, fields, auto, auto_wrong, wrong, len(items), auto_cards, auto_cards_wrong)


# ---- Baseline và regression ----------------------------------------------------------------------


def environment() -> dict[str, object]:
    """Những thứ ngoài code mà số đo phụ thuộc: phiên bản Tesseract, nội dung file mô hình ngôn ngữ
    (hash), và font vẽ thẻ. Lệch một trong ba thì so với baseline không còn công bằng."""
    font = font_path()
    font_hash = hashlib.sha256(Path(font).read_bytes()).hexdigest()[:16] if font else None
    return {"tesseract": tesseract_version(), "traineddata_sha256": traineddata_hashes("vie+eng"),
            "font": {"name": Path(font).name if font else None, "sha256": font_hash}}


def check_environment(current: dict[str, object], baseline: dict[str, object]) -> list[str]:
    """Báo RIÊNG sự lệch môi trường, tách khỏi lỗi regression: số khác có thể do môi trường, không
    do code. Rỗng nghĩa là cùng môi trường với lúc ghi baseline."""
    return [f"môi trường lệch baseline ở `{key}`: baseline {baseline.get(key)!r}, hiện tại {current[key]!r}"
            for key in current if current[key] != baseline.get(key)]


def snapshot(items: list[Item], results: list[CardResult], pages: list[str],
             threshold: float = THRESHOLD) -> dict[str, object]:
    """Các số đưa vào baseline.json. CER là CER của chữ OCR THÔ của từng trường (stage='raw'), vì
    CER sau hậu xử lý coi trường bị bỏ trống là xóa hết nên phạt nặng đúng chỗ ta CHỦ Ý không đoán;
    thay vào đó `fields_correct` đo thẳng kết quả sau hậu xử lý. `page_recall` đo chữ cả trang, không
    qua bước tìm trường. CER làm tròn 4 chữ số để file ổn định."""
    counts = cer_by_field(items, results, stage="raw")
    strict_name = cer_by_field(items, results, stage="raw", strict_names=True)["full_name"]
    gate = gate_stats(items, results, threshold)
    correct = sum(is_correct(n, f.value, it.truth[n]) for it, r in zip(items, results, strict=True)
                  for n, f in r.fields.items())
    recall_hit, recall_total = page_recall(items, pages)
    return {
        "environment": environment(),
        "ocr": {"lang": "vie+eng", "psm": 6},
        "dataset": {"n": len(items), "first_seed": FIRST_SEED},
        "threshold": threshold,
        "cer_raw_overall": round(overall_cer(counts), 4),
        "cer_raw_by_field": {**{name: round(c.cer, 4) for name, c in counts.items()},
                             "full_name_diacritics": round(strict_name.cer, 4)},
        "page_recall": round(recall_hit / recall_total, 4),
        "fields_correct": correct,
        "fields_total": gate.fields,
        "card_review_rate": round(gate.card_review_rate, 4),
        "auto_wrong_fields": gate.auto_wrong,
        "auto_wrong_cards": gate.auto_cards_with_wrong,
    }


def check_regression(current: dict[str, object], baseline: dict[str, object], *,
                     max_cer_increase: float = MAX_CER_INCREASE,
                     max_review_rate_increase: float = MAX_REVIEW_RATE_INCREASE) -> list[str]:
    """Trả danh sách lý do fail (rỗng = đạt). Sáu luật, mỗi luật canh một loại thoái lui:
    1. CER chữ thô của một trường (kể cả tên có dấu) tăng quá `max_cer_increase`: OCR xấu đi.
    2. Độ phủ trang giảm quá `max_cer_increase`: chữ cả trang xấu đi, không phụ thuộc bước tìm trường.
    3. Số trường ĐÚNG sau hậu xử lý giảm (không dung sai): lỗi ở bước tìm trường, sửa, kiểm tra.
    4. Số trường sai mà vẫn được tự nhận tăng (không dung sai).
    5. Số THẺ có trường sai mà cả thẻ vẫn được tự nhận (không ai duyệt) tăng (không dung sai).
    6. Tỉ lệ thẻ vào hàng chờ tăng quá `max_review_rate_increase`: hệ thống kém hữu ích đi.
    """
    failures = []
    base_fields: dict[str, float] = baseline["cer_raw_by_field"]  # type: ignore[assignment]
    cur_fields: dict[str, float] = current["cer_raw_by_field"]  # type: ignore[assignment]
    for name, base_cer in base_fields.items():
        delta = cur_fields.get(name, 1.0) - base_cer
        if delta > max_cer_increase + 1e-9:
            failures.append(f"CER {name} tăng {base_cer:.4f} -> {cur_fields.get(name, 1.0):.4f} (+{delta:.4f})")
    recall_drop = baseline["page_recall"] - current["page_recall"]  # type: ignore[operator]
    if recall_drop > max_cer_increase + 1e-9:
        failures.append(f"độ phủ trang giảm {baseline['page_recall']:.4f} -> {current['page_recall']:.4f}")
    if current["fields_correct"] < baseline["fields_correct"]:  # type: ignore[operator]
        failures.append(f"số trường đúng sau hậu xử lý giảm {baseline['fields_correct']} -> "
                        f"{current['fields_correct']} (trên {baseline['fields_total']})")
    if current["auto_wrong_fields"] > baseline["auto_wrong_fields"]:  # type: ignore[operator]
        failures.append(f"trường sai được tự nhận tăng {baseline['auto_wrong_fields']} -> "
                        f"{current['auto_wrong_fields']}")
    if current["auto_wrong_cards"] > baseline["auto_wrong_cards"]:  # type: ignore[operator]
        failures.append(f"thẻ lọt hẳn qua cổng mà có trường sai tăng {baseline['auto_wrong_cards']} -> "
                        f"{current['auto_wrong_cards']}")
    review_delta = current["card_review_rate"] - baseline["card_review_rate"]  # type: ignore[operator]
    if review_delta > max_review_rate_increase + 1e-9:
        failures.append(f"tỉ lệ thẻ vào hàng chờ tăng {baseline['card_review_rate']:.2f} -> "
                        f"{current['card_review_rate']:.2f}")
    return failures


def load_baseline() -> dict[str, object]:
    return json.loads(BASELINE_FILE.read_text(encoding="utf-8"))


def save_baseline(snap: dict[str, object]) -> None:
    BASELINE_FILE.write_text(json.dumps(snap, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def downscale_half(image: Image.Image) -> Image.Image:
    """«Tối ưu» có chủ ý làm hỏng: thu ảnh còn một nửa để OCR nhanh hơn. Dùng để thử regression."""
    return image.resize((image.width // 2, image.height // 2), Image.Resampling.BICUBIC)
