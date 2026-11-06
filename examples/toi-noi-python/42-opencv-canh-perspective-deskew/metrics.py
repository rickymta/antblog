"""Metric cho OCR giấy tờ: khoảng cách Levenshtein, CER, WER, precision theo trường, cổng duyệt.

Chỉ dùng thư viện chuẩn. Bài 45 thay phần CER/WER bằng `jiwer` và dùng cùng định nghĩa.
"""

import re
import unicodedata
from collections.abc import Callable, Iterable, Sequence
from dataclasses import dataclass


def normalize(text: str) -> str:
    """NFC, bỏ khoảng trắng đầu cuối, gộp khoảng trắng liên tiếp. Không đổi hoa thường hay dấu."""
    return re.sub(r"\s+", " ", unicodedata.normalize("NFC", text)).strip()


def strip_marks(text: str) -> str:
    """Bỏ dấu tiếng Việt (Đ/đ không phân rã nên đổi tay): để biết bao nhiêu lỗi chỉ là mất dấu."""
    text = unicodedata.normalize("NFC", text).replace("Đ", "D").replace("đ", "d")
    return "".join(ch for ch in unicodedata.normalize("NFD", text) if not unicodedata.combining(ch))


def levenshtein(ref: Sequence[str], hyp: Sequence[str]) -> int:
    """Số phép sửa ít nhất (thay, xóa, chèn, mỗi phép giá 1) để biến `hyp` thành `ref`.

    Quy hoạch động hai hàng: `prev[j]` là khoảng cách giữa `i - 1` phần tử đầu của ref và `j`
    phần tử đầu của hyp. Chạy trên chuỗi ký tự cho CER, trên list từ cho WER.
    """
    prev = list(range(len(hyp) + 1))
    for i, r in enumerate(ref, start=1):
        cur = [i]
        for j, h in enumerate(hyp, start=1):
            cur.append(min(prev[j] + 1,  # xóa r khỏi ref (hyp thiếu một phần tử)
                           cur[j - 1] + 1,  # chèn: hyp thừa một phần tử
                           prev[j - 1] + (r != h)))  # khớp (0) hoặc thay (1)
        prev = cur
    return prev[-1]


def cer(ref: str, hyp: str) -> float:
    """Character error rate = số phép sửa ký tự / độ dài ref. Có thể lớn hơn 1 nếu hyp dài hơn nhiều."""
    ref, hyp = normalize(ref), normalize(hyp)
    if not ref:
        raise ValueError("CER không xác định khi chuỗi tham chiếu rỗng")
    return levenshtein(ref, hyp) / len(ref)


def wer(ref: str, hyp: str) -> float:
    """Word error rate: cùng công thức, đơn vị là từ tách theo khoảng trắng."""
    ref_words, hyp_words = normalize(ref).split(), normalize(hyp).split()
    if not ref_words:
        raise ValueError("WER không xác định khi chuỗi tham chiếu rỗng")
    return levenshtein(ref_words, hyp_words) / len(ref_words)


def corpus_rate(pairs: Iterable[tuple[str, str]], unit: Callable[[str], Sequence[str]] = list) -> float:
    """Tỉ lệ lỗi gộp (micro): tổng phép sửa chia tổng độ dài ref, không lấy trung bình các tỉ lệ.

    Trung bình tỉ lệ cho mỗi chuỗi một phiếu bầu như nhau, nên trường ngắn («C», «Nam») nặng
    ngang trường dài. Cách gộp làm mỗi ký tự một phiếu.
    """
    edits = total = 0
    for ref, hyp in pairs:
        ref_units = unit(normalize(ref))
        edits += levenshtein(ref_units, unit(normalize(hyp)))
        total += len(ref_units)
    if total == 0:
        raise ValueError("không có gì để đo")
    return edits / total


def corpus_cer(pairs: Iterable[tuple[str, str]]) -> float:
    return corpus_rate(pairs, list)


def corpus_wer(pairs: Iterable[tuple[str, str]]) -> float:
    return corpus_rate(pairs, str.split)


@dataclass(frozen=True)
class FieldPrediction:
    """Một trường OCR đọc được: chữ và độ tin cậy 0..1 (rỗng, 0.0 nếu không tìm thấy)."""

    text: str
    confidence: float


MISSING = FieldPrediction("", 0.0)


def exact_match(truth: str, pred: str) -> bool:
    return normalize(truth) == normalize(pred)


def sample_is_correct(truth: dict[str, str], preds: dict[str, FieldPrediction]) -> bool:
    """Đúng khi mọi trường khớp chính xác. Một ký tự sai trong số giấy tờ là sai cả giấy tờ đó."""
    return all(exact_match(value, preds.get(name, MISSING).text) for name, value in truth.items())


def needs_review(expected_fields: Iterable[str], preds: dict[str, FieldPrediction], threshold: float = 0.80) -> bool:
    """Cổng chuyển người duyệt: thiếu một trường hoặc có trường tin cậy dưới ngưỡng.

    Cổng chỉ nhìn thứ OCR tự báo (có đọc được không, độ tin cậy), không biết đáp án. Độ tin cậy
    của Tesseract là điểm xếp hạng chứ không phải xác suất đã hiệu chỉnh.
    """
    for name in expected_fields:
        pred = preds.get(name, MISSING)
        if not pred.text or pred.confidence < threshold:
            return True
    return False
