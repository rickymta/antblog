"""Baseline OCR: Tesseract chạy nguyên ảnh, tách trường bằng nhãn in trên thẻ.

Cố ý đơn giản để có một mốc so sánh: không tiền xử lý ảnh (bài 42), không regex hay sửa lỗi
OCR (bài 44), không chọn mô hình (bài 43). Mọi thứ tiếp theo của phần 05 được đo so với số này.
"""

import csv
import io
import os
import shutil
import subprocess
from dataclasses import dataclass
from difflib import SequenceMatcher
from pathlib import Path

from metrics import MISSING, FieldPrediction, strip_marks

TESSERACT = shutil.which("tesseract")
LANG = "vie"
PSM = "4"  # cấu hình riêng của bài 41 và 42 (bài 43 trở đi dùng vie+eng, --psm 6); main.py đo thêm 3, 6, 11

# Nhãn in trên thẻ (đúng như trong synthetic_card.py) dùng để tìm dòng chứa giá trị.
LABELS: dict[str, dict[str, str]] = {
    "cccd": {"id_number": "Số / No.:", "full_name": "Họ và tên:", "date_of_birth": "Ngày sinh:",
             "sex": "Giới tính:", "hometown_city": "Quê quán:"},
    "gplx": {"license_number": "Số / No.:", "full_name": "Họ và tên:", "date_of_birth": "Ngày sinh:",
             "license_class": "Hạng / Class:", "expiry_date": "Có giá trị đến / Expires:"},
}
LABEL_MATCH_THRESHOLD = 0.7  # độ giống tối thiểu (SequenceMatcher) để coi một dòng là nhãn


@dataclass(frozen=True)
class OcrLine:
    text: str
    confidence: float  # nhỏ nhất trong các từ của dòng, 0..1


def tesseract_available() -> bool:
    return TESSERACT is not None


def tesseract_version() -> str:
    out = subprocess.run([TESSERACT, "--version"], capture_output=True, text=True, check=True)
    return (out.stdout or out.stderr).splitlines()[0]


def parse_tsv(tsv: str) -> list[OcrLine]:
    """Gom các từ (level 5) của đầu ra TSV thành dòng theo (block, paragraph, line).

    Cột TSV: level, page, block, par, line, word, left, top, width, height, conf, text.
    Từ không đọc được có conf -1 hoặc text rỗng thì bị bỏ.
    """
    words: dict[tuple[str, str, str], list[tuple[str, float]]] = {}
    reader = csv.reader(io.StringIO(tsv), delimiter="\t", quoting=csv.QUOTE_NONE)
    next(reader, None)  # dòng tiêu đề
    for row in reader:
        if len(row) < 12 or row[0] != "5" or not row[11].strip():
            continue
        conf = float(row[10])
        if conf < 0:
            continue
        words.setdefault((row[2], row[3], row[4]), []).append((row[11].strip(), conf / 100.0))
    return [OcrLine(" ".join(w for w, _ in ws), min(c for _, c in ws)) for ws in words.values()]


def ocr_lines(image_path: Path, *, lang: str = LANG, psm: str = PSM) -> list[OcrLine]:
    """Chạy `tesseract <ảnh> stdout -l <lang> --psm <psm> tsv` và trả các dòng chữ."""
    if TESSERACT is None:
        raise RuntimeError("không tìm thấy tesseract (brew install tesseract tesseract-lang)")
    env = {**os.environ, "OMP_THREAD_LIMIT": "1"}  # mỗi tiến trình một luồng: ta tự chạy song song nhiều ảnh
    out = subprocess.run([TESSERACT, str(image_path), "stdout", "-l", lang, "--psm", psm, "tsv"],
                         capture_output=True, encoding="utf-8", env=env, timeout=120, check=True)
    return parse_tsv(out.stdout)


def _fold(text: str) -> str:
    """Chữ thường, không dấu, chỉ chữ và số: để so nhãn không bị lỗi dấu hay dấu câu của OCR."""
    return "".join(ch for ch in strip_marks(text).lower() if ch.isalnum())


def extract_fields(lines: list[OcrLine], doc_type: str) -> dict[str, FieldPrediction]:
    """Với mỗi trường: tìm dòng giống nhãn nhất, lấy dòng ngay sau nó làm giá trị.

    Không tìm được nhãn đủ giống thì trường vắng mặt. Độ tin cậy của trường là độ tin cậy nhỏ
    nhất trong các từ của dòng giá trị: một từ yếu kéo cả trường xuống.
    """
    preds: dict[str, FieldPrediction] = {}
    folded = [_fold(line.text) for line in lines]
    for name, label in LABELS[doc_type].items():
        target = _fold(label)
        scores = [SequenceMatcher(None, f, target).ratio() for f in folded]
        best = max(range(len(lines)), key=scores.__getitem__, default=None)
        if best is None or scores[best] < LABEL_MATCH_THRESHOLD or best + 1 >= len(lines):
            preds[name] = MISSING
            continue
        value = lines[best + 1]
        preds[name] = FieldPrediction(value.text, value.confidence)
    return preds
