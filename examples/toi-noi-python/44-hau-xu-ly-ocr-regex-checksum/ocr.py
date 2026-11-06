"""Gọi Tesseract và đọc kết quả dạng TSV: từng từ kèm khung chữ nhật và confidence.

Tesseract là chương trình ngoài (`brew install tesseract tesseract-lang`, cần gói `vie`).
Lab gọi bằng subprocess thay vì thư viện bọc, để thấy đúng lệnh và đúng đầu ra thô.
Kết quả được cache trong out/ocr_cache/ theo hash của (ảnh, ngôn ngữ, psm, phiên bản Tesseract,
nội dung file `<lang>.traineddata`): đổi mô hình ngôn ngữ thì cache cũ không bị dùng nhầm.
"""

import hashlib
import io
import os
import re
import subprocess
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path

from PIL import Image

CACHE_DIR = Path(__file__).parent / "out" / "ocr_cache"


@dataclass(frozen=True)
class Word:
    """Một từ Tesseract nhận ra. `conf` đã đổi về 0..1; `line` là khóa (block, par, line)."""

    text: str
    conf: float
    left: int
    top: int
    width: int
    height: int
    line: tuple[int, int, int]

    @property
    def bottom(self) -> int:
        return self.top + self.height


class TesseractMissing(RuntimeError):
    """Không chạy được Tesseract, hoặc thiếu gói ngôn ngữ."""


@lru_cache(maxsize=1)
def tesseract_version() -> str:
    try:
        out = subprocess.run(["tesseract", "--version"], capture_output=True, text=True, check=True)
    except (FileNotFoundError, subprocess.CalledProcessError) as exc:
        raise TesseractMissing("không chạy được tesseract (brew install tesseract tesseract-lang)") from exc
    return (out.stdout or out.stderr).splitlines()[0].strip()


@lru_cache(maxsize=1)
def tessdata_dir() -> Path:
    """Thư mục mô hình ngôn ngữ, lấy từ dòng đầu của `tesseract --list-langs`."""
    tesseract_version()
    out = subprocess.run(["tesseract", "--list-langs"], capture_output=True, text=True)
    first = (out.stdout or out.stderr).splitlines()[0] if (out.stdout or out.stderr) else ""
    match = re.search(r'"([^"]+)"', first)
    if match is None:
        raise TesseractMissing(f"không đọc được thư mục tessdata từ `tesseract --list-langs`: {first!r}")
    return Path(match.group(1))


@lru_cache(maxsize=None)
def traineddata_hashes(lang: str) -> dict[str, str]:
    """SHA-256 (16 ký tự đầu) của từng file `<ngôn ngữ>.traineddata`. Thiếu file thì báo rõ."""
    hashes = {}
    for part in lang.split("+"):
        path = tessdata_dir() / f"{part}.traineddata"
        if not path.exists():
            raise TesseractMissing(f"thiếu {path} (brew install tesseract-lang, gói `{part}`)")
        hashes[part] = hashlib.sha256(path.read_bytes()).hexdigest()[:16]
    return hashes


def parse_tsv(tsv: str) -> list[Word]:
    """Đọc TSV của Tesseract, chỉ giữ dòng level 5 (từ) có chữ. conf -1 nghĩa là không có."""
    words: list[Word] = []
    for row in tsv.splitlines()[1:]:  # bỏ dòng tiêu đề
        cols = row.split("\t")
        if len(cols) < 12 or cols[0] != "5":
            continue
        text = cols[11].strip()
        if not text:
            continue
        words.append(Word(
            text=text, conf=max(float(cols[10]), 0.0) / 100.0,
            left=int(cols[6]), top=int(cols[7]), width=int(cols[8]), height=int(cols[9]),
            line=(int(cols[2]), int(cols[3]), int(cols[4])),
        ))
    return words


def ocr_words(image: Image.Image, *, lang: str = "vie+eng", psm: int = 6, use_cache: bool = True) -> list[Word]:
    """OCR một ảnh PIL, trả danh sách từ. psm 6 = «một khối chữ đồng nhất»."""
    buffer = io.BytesIO()
    image.save(buffer, format="PNG")
    png = buffer.getvalue()
    model = ",".join(f"{k}={v}" for k, v in traineddata_hashes(lang).items())
    key = hashlib.sha256(png + f"|{lang}|{psm}|{tesseract_version()}|{model}".encode()).hexdigest()[:24]
    cache_file = CACHE_DIR / f"{key}.tsv"
    if use_cache and cache_file.exists():
        return parse_tsv(cache_file.read_text(encoding="utf-8"))
    # OMP_THREAD_LIMIT chỉ có tác dụng với bản Tesseract build kèm OpenMP. Bản Homebrew đang dùng
    # KHÔNG có OpenMP (`tesseract --version` không có dòng «Found OpenMP»), nên biến này ở đây
    # không làm gì; giữ lại để lab vẫn đúng trên môi trường có OpenMP.
    env = {**os.environ, "OMP_THREAD_LIMIT": "1"}
    try:
        run = subprocess.run(["tesseract", "stdin", "stdout", "-l", lang, "--psm", str(psm), "tsv"],
                             input=png, capture_output=True, env=env, check=True)
    except subprocess.CalledProcessError as exc:
        detail = exc.stderr.decode("utf-8", "replace").strip()
        raise TesseractMissing(f"tesseract lỗi với -l {lang}: {detail}") from exc
    tsv = run.stdout.decode("utf-8")
    if use_cache:
        CACHE_DIR.mkdir(parents=True, exist_ok=True)
        cache_file.write_text(tsv, encoding="utf-8")
    return parse_tsv(tsv)
