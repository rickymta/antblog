"""Kiểu dữ liệu chung và phép đo cho mọi engine OCR của bài 43.

Ba engine trả về ba dạng khác nhau (Tesseract: TSV theo từ, EasyOCR: đa giác + chữ + điểm,
PaddleOCR: dict có rec_texts, rec_scores, rec_boxes). Ta đưa tất cả về `OcrLine`, rồi chỉ
so sánh ở dạng đó. File này không import thư viện OCR nào.

Hàm `levenshtein`, `cer` tự viết: bài 41 đã định nghĩa CER theo khoảng cách Levenshtein và
bài 45 sẽ đổi sang `jiwer`; ở đây giữ bản ngắn để lab không phụ thuộc thêm gói nào.
"""

import json
import os
import resource
import unicodedata
from dataclasses import dataclass
from pathlib import Path
from typing import Any

FIELDS = ("id_number", "date_of_birth", "full_name", "sex", "hometown_city")


@dataclass(frozen=True)
class OcrLine:
    """Một dòng chữ engine đọc được. `confidence` đã đưa về 0..1; `box` là (x0, y0, x1, y1)."""

    text: str
    confidence: float
    box: tuple[int, int, int, int]


class OcrError(Exception):
    """Engine không đọc được ảnh (file hỏng, không phải ảnh, định dạng lạ)."""


class EngineUnavailable(Exception):
    """Engine chưa cài hoặc không chạy được trên Python này; thông báo nói cách cài."""


# ---- Chuẩn hóa chữ ---------------------------------------------------------------------


def nfc(text: str) -> str:
    """NFC để «ễ» là một ký tự thay vì «e» cộng hai dấu rời; gọn khoảng trắng hai đầu và giữa."""
    return " ".join(unicodedata.normalize("NFC", text).split())


def nfc_chars(text: str) -> str:
    """NFC nhưng giữ nguyên khoảng trắng: dùng khi so từng ký tự với bảng ký tự của mô hình."""
    return unicodedata.normalize("NFC", text)


def fold(text: str) -> str:
    """Bỏ dấu tiếng Việt (Đ/đ không phân rã nên đổi tay). Dùng để tách lỗi dấu khỏi lỗi chữ."""
    text = nfc(text).replace("Đ", "D").replace("đ", "d")
    decomposed = unicodedata.normalize("NFD", text)
    return "".join(ch for ch in decomposed if not unicodedata.combining(ch))


# ---- Khoảng cách chỉnh sửa -------------------------------------------------------------


def levenshtein(a: str, b: str) -> int:
    """Số phép thay, xóa, chèn tối thiểu để biến a thành b."""
    if len(a) < len(b):
        a, b = b, a
    previous = list(range(len(b) + 1))
    for i, ca in enumerate(a, start=1):
        current = [i]
        for j, cb in enumerate(b, start=1):
            current.append(min(previous[j] + 1, current[j - 1] + 1, previous[j - 1] + (ca != cb)))
        previous = current
    return previous[-1]


def cer(truth: str, hypothesis: str) -> float:
    """Character error rate: levenshtein chia độ dài đáp án (đáp án rỗng thì không định nghĩa)."""
    if not truth:
        raise ValueError("đáp án rỗng: CER không xác định")
    return levenshtein(truth, hypothesis) / len(truth)


def best_substring_distance(truth: str, text: str) -> int:
    """Khoảng cách nhỏ nhất giữa `truth` và MỌI đoạn con liên tiếp của `text`.

    Là Levenshtein «nửa toàn cục»: đầu và cuối của `text` được bỏ miễn phí, còn `truth` phải
    khớp trọn. Ta cần vậy vì engine đọc cả dòng («Họ và tên: NGUYỄN VĂN A») còn đáp án chỉ là
    giá trị trường. Dòng không chứa giá trị thì khoảng cách vẫn nhỏ hơn len(truth) nếu có vài
    ký tự trùng ăn may; vì thế bảng kết quả luôn kèm tỉ lệ khớp nguyên văn.
    """
    previous = list(range(len(truth) + 1))  # cột: đã dùng i ký tự đáp án; hàng đầu: text rỗng
    best = previous[-1]
    for ch in text:
        current = [0]  # bỏ miễn phí phần đầu text
        for i, tc in enumerate(truth, start=1):
            current.append(min(previous[i] + 1, current[i - 1] + 1, previous[i - 1] + (tc != ch)))
        previous = current
        best = min(best, previous[-1])
    return best


def reading_order(lines: list[OcrLine]) -> list[OcrLine]:
    """Từ trên xuống, cùng hàng thì trái sang phải (hàng = tâm dọc lệch dưới nửa chiều cao dòng)."""
    return sorted(lines, key=lambda ln: ((ln.box[1] + ln.box[3]) // 2 // 20, ln.box[0]))


def candidates(lines: list[OcrLine]) -> list[str]:
    """Các đoạn chữ để dò giá trị trường: từng dòng, và từng cặp dòng liền nhau.

    Cặp dòng cho phép nhặt lại giá trị mà engine cắt làm đôi (vd «NGUYỄN» rồi «VĂN A»).
    """
    ordered = [nfc(ln.text) for ln in reading_order(lines)]
    pairs = [f"{a} {b}" for a, b in zip(ordered, ordered[1:], strict=False)]
    return ordered + pairs


def field_distance(truth: str, texts: list[str], *, fold_marks: bool = False) -> int:
    """Khoảng cách nhỏ nhất từ giá trị trường tới mọi đoạn ứng viên (không có ứng viên: len)."""
    target = fold(truth) if fold_marks else nfc(truth)
    if not texts:
        return len(target)
    return min(best_substring_distance(target, fold(t) if fold_marks else t) for t in texts)


@dataclass(frozen=True)
class FieldScore:
    """Điểm một trường trên một ảnh."""

    cer: float  # khoảng cách tốt nhất / len(đáp án), giữ dấu
    exact: bool  # có đoạn khớp nguyên văn (giữ dấu)
    exact_folded: bool  # khớp nguyên văn khi bỏ dấu cả hai phía


def score_fields(truth: dict[str, str], lines: list[OcrLine]) -> dict[str, FieldScore]:
    texts = candidates(lines)
    scores: dict[str, FieldScore] = {}
    for name in FIELDS:
        value = truth[name]
        d = field_distance(value, texts)
        scores[name] = FieldScore(
            cer=d / len(nfc(value)),
            exact=d == 0,
            exact_folded=field_distance(value, texts, fold_marks=True) == 0,
        )
    return scores


# ---- Đo tài nguyên ---------------------------------------------------------------------


def cpu_seconds() -> float:
    """CPU (user + system) của tiến trình này CỘNG các tiến trình con đã kết thúc.

    Tesseract chạy trong tiến trình con (tính qua RUSAGE_CHILDREN), EasyOCR và PaddleOCR chạy
    trong tiến trình này (RUSAGE_SELF, gồm mọi thread). Hiệu hai lần gọi là CPU của đoạn giữa.
    """
    own = resource.getrusage(resource.RUSAGE_SELF)
    kids = resource.getrusage(resource.RUSAGE_CHILDREN)
    return own.ru_utime + own.ru_stime + kids.ru_utime + kids.ru_stime


def load_average() -> float:
    return os.getloadavg()[0]


# ---- File kết quả ----------------------------------------------------------------------


def lines_to_json(lines: list[OcrLine]) -> list[dict[str, Any]]:
    return [{"text": ln.text, "confidence": round(ln.confidence, 4), "box": list(ln.box)} for ln in lines]


def lines_from_json(rows: list[dict[str, Any]]) -> list[OcrLine]:
    return [OcrLine(r["text"], float(r["confidence"]), tuple(r["box"])) for r in rows]  # type: ignore[arg-type]


def write_json(path: Path, data: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=1), encoding="utf-8")
