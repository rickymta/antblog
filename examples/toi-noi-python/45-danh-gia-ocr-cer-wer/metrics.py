"""CER và WER: định nghĩa bằng khoảng cách Levenshtein, đối chiếu với jiwer.

CER = (S + D + I) / N, với N là số ký tự của chuỗi THAM CHIẾU (đáp án), S là số ký tự bị thay,
D số bị xóa (đáp án có mà OCR không có), I số bị chèn (OCR có mà đáp án không có). WER giống hệt
nhưng đơn vị là từ. Gộp nhiều chuỗi (corpus-level): cộng S, D, I và N của mọi chuỗi rồi chia một
lần, KHÔNG lấy trung bình các CER riêng, vì trung bình riêng để chuỗi ngắn lấn át chuỗi dài.

CER có thể vượt 1 (chèn nhiều hơn độ dài đáp án) và tính chuỗi Unicode theo từng mã, nên
'ễ' dạng NFD (hai mã) và NFC (một mã) cho CER khác 0: luôn NFC cả hai phía trước khi đo.
"""

import unicodedata
from collections.abc import Sequence
from dataclasses import dataclass

import jiwer


def nfc(text: str) -> str:
    return unicodedata.normalize("NFC", text)


def edit_distance(ref: Sequence[str], hyp: Sequence[str]) -> int:
    """Levenshtein tối thiểu số phép thay/xóa/chèn để biến `hyp` thành `ref` (quy hoạch động, O(n*m))."""
    previous = list(range(len(hyp) + 1))
    for i, r in enumerate(ref, start=1):
        current = [i]
        for j, h in enumerate(hyp, start=1):
            current.append(min(previous[j] + 1,  # xóa r khỏi tham chiếu: OCR thiếu ký tự
                               current[j - 1] + 1,  # chèn h: OCR thừa ký tự
                               previous[j - 1] + (r != h)))  # thay (hoặc khớp nếu r == h)
        previous = current
    return previous[-1]


def cer_by_hand(refs: Sequence[str], hyps: Sequence[str]) -> float:
    """CER corpus-level tính tay bằng edit_distance, để đối chiếu với jiwer."""
    # jiwer cắt khoảng trắng hai đầu mỗi chuỗi trước khi đếm; làm giống để hai bên cùng định nghĩa.
    errors = sum(edit_distance(nfc(r).strip(), nfc(h).strip()) for r, h in zip(refs, hyps, strict=True))
    return errors / sum(len(nfc(r).strip()) for r in refs)


def wer_by_hand(refs: Sequence[str], hyps: Sequence[str]) -> float:
    errors = sum(edit_distance(nfc(r).split(), nfc(h).split()) for r, h in zip(refs, hyps, strict=True))
    return errors / sum(len(nfc(r).split()) for r in refs)


@dataclass(frozen=True)
class ErrorCounts:
    """Phân rã lỗi mức ký tự: hits + substitutions + deletions = N (số ký tự tham chiếu)."""

    hits: int
    substitutions: int
    deletions: int
    insertions: int

    @property
    def reference_chars(self) -> int:
        return self.hits + self.substitutions + self.deletions

    @property
    def cer(self) -> float:
        return (self.substitutions + self.deletions + self.insertions) / self.reference_chars


def char_errors(refs: Sequence[str], hyps: Sequence[str]) -> ErrorCounts:
    """Đếm S/D/I mức ký tự bằng jiwer. refs không được chứa chuỗi rỗng (jiwer từ chối)."""
    out = jiwer.process_characters([nfc(r) for r in refs], [nfc(h) for h in hyps])
    return ErrorCounts(out.hits, out.substitutions, out.deletions, out.insertions)


def cer(refs: Sequence[str], hyps: Sequence[str]) -> float:
    return float(jiwer.cer([nfc(r) for r in refs], [nfc(h) for h in hyps]))


def wer(refs: Sequence[str], hyps: Sequence[str]) -> float:
    return float(jiwer.wer([nfc(r) for r in refs], [nfc(h) for h in hyps]))
