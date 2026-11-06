"""Chạy Tesseract như tiến trình con có thể bị hủy, và đọc các trường từ kết quả TSV.

Phần quan trọng nhất của bài là `run_tesseract`: timeout ở đây DỪNG việc (kill tiến trình con),
chứ không chỉ ngừng chờ. Phần `parse_card` là bản rút gọn dùng regex; bài 44 làm đầy đủ
(chuẩn hóa Unicode, regex có tên, sửa lỗi OCR, rapidfuzz).
"""

import asyncio
import difflib
import re
import shutil
import unicodedata
from dataclasses import dataclass

from .settings import ocr_env

LANGS = "vie+eng"
PSM = 6  # một khối chữ đồng nhất: hợp với thẻ có các dòng nhãn và giá trị
CITIES = ("Hà Nội", "TP.HCM", "Đà Nẵng", "Hải Phòng", "Cần Thơ", "Nha Trang")


class OcrTimeout(Exception):
    """Tesseract chạy quá hạn và đã bị kill. `pid` để kiểm tra tiến trình con không còn."""

    def __init__(self, timeout: float, pid: int) -> None:
        super().__init__(f"tesseract quá {timeout:g} giây (pid {pid}), đã kill")
        self.timeout, self.pid = timeout, pid


class OcrFailed(Exception):
    """Tesseract thoát với mã khác 0 (ảnh không đọc được, thiếu dữ liệu ngôn ngữ...)."""


def find_tesseract() -> str:
    path = shutil.which("tesseract")
    if path is None:
        raise FileNotFoundError("không thấy `tesseract` trên PATH (brew install tesseract tesseract-lang)")
    return path


async def tesseract_version(binary: str) -> str:
    proc = await asyncio.create_subprocess_exec(
        binary, "--version", stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.STDOUT)
    out, _ = await proc.communicate()
    return out.decode().splitlines()[0].removeprefix("tesseract ").strip()  # "5.5.4"


async def run_tesseract(binary: str, image: bytes, *, timeout: float, langs: str = LANGS, psm: int = PSM) -> str:
    """Trả về TSV của Tesseract. Ảnh đi qua stdin nên không có file tạm để dọn.

    Hủy ở HAI chỗ: hết `timeout` (OcrTimeout) và khi coroutine bị cancel từ ngoài (arq hủy job quá
    job_timeout, worker nhận SIGTERM). `finally` kill tiến trình con trong cả hai trường hợp.
    """
    proc = await asyncio.create_subprocess_exec(
        binary, "stdin", "stdout", "-l", langs, "--psm", str(psm), "tsv",
        stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE,
        env=ocr_env())
    try:
        async with asyncio.timeout(timeout):
            out, err = await proc.communicate(image)
    except TimeoutError:
        raise OcrTimeout(timeout, proc.pid) from None
    finally:
        if proc.returncode is None:  # còn chạy: timeout hoặc bị cancel
            proc.kill()
            await asyncio.shield(proc.wait())  # thu xác để không để lại tiến trình zombie
    if proc.returncode != 0:
        raise OcrFailed(err.decode(errors="replace").strip()[:200] or f"mã thoát {proc.returncode}")
    return out.decode()


# ---------------------------------------------------------------- đọc TSV thành trường


@dataclass(frozen=True)
class Token:
    text: str
    conf: float  # 0..1


def fold(text: str) -> str:
    """Chữ thường, bỏ dấu: để so nhãn («Họ và tên» đọc thành «Ho va tên» vẫn khớp)."""
    text = text.replace("Đ", "D").replace("đ", "d")
    return "".join(c for c in unicodedata.normalize("NFD", text) if not unicodedata.combining(c)).lower()


def tsv_lines(tsv: str) -> list[list[Token]]:
    """Gom các từ (level 5 của TSV) theo dòng; bỏ ô trống và ô conf = -1."""
    lines: dict[tuple[str, str, str], list[Token]] = {}
    for row in tsv.splitlines()[1:]:
        cols = row.split("\t")
        if len(cols) == 12 and cols[11].strip() and float(cols[10]) >= 0:
            lines.setdefault((cols[2], cols[3], cols[4]), []).append(Token(cols[11].strip(), float(cols[10]) / 100))
    return list(lines.values())


def _value_after(lines: list[list[Token]], label: str) -> list[Token]:
    """Các token ở dòng ngay sau dòng nhãn (nhãn so sau khi bỏ dấu)."""
    for i, line in enumerate(lines[:-1]):
        if label in fold(" ".join(t.text for t in line)):
            return lines[i + 1]
    return []


def _field(tokens: list[Token], text: str | None = None) -> tuple[str, float] | None:
    """(giá trị, độ tin cậy nhỏ nhất của các token). `text` là giá trị đã chuẩn hóa; không có token thì không có trường."""
    if not tokens:
        return None
    return (text if text is not None else " ".join(t.text for t in tokens),
            round(min(t.conf for t in tokens), 3))


def _regex_field(lines: list[list[Token]], pattern: str) -> tuple[str, float] | None:
    for line in lines:
        for token in line:
            if re.fullmatch(pattern, token.text):
                return token.text, round(token.conf, 3)
    return None


def parse_card(tsv: str) -> tuple[dict[str, str | None], dict[str, float]]:
    """Trường của CCCD giả: số 12 chữ số, ngày dd/mm/yyyy, tên, giới tính, quê quán.

    Độ tin cậy của một trường là nhỏ nhất trong các token của nó (như bài 26). Trường không tìm được
    là None với độ tin cậy 0: để người duyệt thấy, không bịa.
    """
    lines = tsv_lines(tsv)
    found: dict[str, tuple[str, float] | None] = {
        "id_number": _regex_field(lines, r"\d{12}"),
        "date_of_birth": _regex_field(lines, r"\d{2}/\d{2}/\d{4}"),
    }
    name_tokens = [t for t in _value_after(lines, "ho va ten") if t.text.replace(".", "").isalpha()]
    found["full_name"] = _field(name_tokens)
    sex_tokens = _value_after(lines, "gioi tinh")
    sex_text = " ".join(t.text for t in sex_tokens)
    sex_value = "Nam" if fold(sex_text).startswith("nam") else "Nữ" if fold(sex_text).startswith("nu") else None
    # Giới tính và quê quán chỉ nhận giá trị thuộc danh sách đã biết; chữ rác thì để trống (None, độ tin cậy 0)
    # cho người duyệt thấy, thay vì nhận chữ rác với độ tin cậy cao của chính token rác đó.
    found["sex"] = _field(sex_tokens, sex_value) if sex_value is not None else None
    city_tokens = _value_after(lines, "que quan")
    city_text = " ".join(t.text for t in city_tokens)
    # Chọn thành phố gần nhất trong danh sách 6 thành phố của Tới Nơi (difflib; bài 44 dùng rapidfuzz).
    match = difflib.get_close_matches(fold(city_text), [fold(c) for c in CITIES], n=1, cutoff=0.6)
    city_value = CITIES[[fold(c) for c in CITIES].index(match[0])] if match else None
    found["hometown_city"] = _field(city_tokens, city_value) if city_value is not None else None

    fields = {k: (v[0] if v else None) for k, v in found.items()}
    confidence = {k: (v[1] if v else 0.0) for k, v in found.items()}
    return fields, confidence
