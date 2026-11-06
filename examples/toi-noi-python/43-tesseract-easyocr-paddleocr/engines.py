"""Ba engine OCR sau cùng một giao diện: `recognize(path) -> list[OcrLine]`.

Tesseract chạy qua tiến trình con `tesseract`; EasyOCR và PaddleOCR chạy trong tiến trình này
và chỉ được import khi dùng (import PyTorch hay PaddlePaddle mất vài giây và có thể không có).
"""

import importlib.metadata as metadata
import os
import subprocess
import sys
import warnings
from pathlib import Path
from typing import Protocol

import numpy as np
from PIL import Image, UnidentifiedImageError

from ocr_common import EngineUnavailable, OcrError, OcrLine, nfc_chars

ENGINE_NAMES = ("tesseract", "easyocr", "paddleocr", "paddleocr-v5")


class OcrEngine(Protocol):
    name: str

    def describe(self) -> dict[str, str]: ...

    def model_files(self) -> list[Path]: ...

    def missing_chars(self, chars: str) -> list[str] | None: ...

    def recognize(self, path: Path) -> list[OcrLine]: ...


def read_rgb(path: Path) -> Image.Image:
    """Mở ảnh bằng Pillow; mọi lỗi giải mã đổi thành OcrError có tên file."""
    try:
        with Image.open(path) as image:
            return image.convert("RGB")
    except (UnidentifiedImageError, OSError, SyntaxError) as exc:
        raise OcrError(f"không đọc được ảnh {path.name}: {exc}") from exc


def _version(package: str) -> str:
    try:
        return metadata.version(package)
    except metadata.PackageNotFoundError:
        return "chưa cài"


class TesseractEngine:
    """Tesseract qua `pytesseract.image_to_data`: mỗi TỪ có bounding box và confidence 0..100."""

    def __init__(self, lang: str = "vie+eng", psm: int = 6, *, name: str = "tesseract") -> None:
        import pytesseract

        self._pt = pytesseract
        self.lang, self.psm, self.name = lang, psm, name
        try:
            self.version = str(pytesseract.get_tesseract_version())
        except pytesseract.TesseractNotFoundError as exc:
            raise EngineUnavailable("không thấy lệnh tesseract (macOS: brew install tesseract tesseract-lang)") from exc
        missing = [code for code in lang.split("+") if code not in pytesseract.get_languages(config="")]
        if missing:
            raise EngineUnavailable(f"thiếu gói ngôn ngữ Tesseract: {', '.join(missing)} (brew install tesseract-lang)")

    def describe(self) -> dict[str, str]:
        return {"engine": f"Tesseract {self.version}", "config": f"-l {self.lang} --psm {self.psm}",
                "pytesseract": _version("pytesseract")}

    def model_files(self) -> list[Path]:
        """Gói ngôn ngữ cài cùng Tesseract (không tải lúc chạy): <tessdata>/<mã>.traineddata."""
        listing = subprocess.run(["tesseract", "--list-langs"], capture_output=True, text=True, check=True)
        folder = Path(listing.stdout.splitlines()[0].split('"')[1])
        return [folder / f"{code}.traineddata" for code in self.lang.split("+")]

    def missing_chars(self, chars: str) -> list[str] | None:
        return None  # unicharset nằm trong file traineddata, lab không giải nén; không kiểm thì không nói

    def recognize(self, path: Path) -> list[OcrLine]:
        image = read_rgb(path)
        data = self._pt.image_to_data(image, lang=self.lang, config=f"--psm {self.psm}",
                                      output_type=self._pt.Output.DICT)
        lines: dict[tuple[int, int, int], list[int]] = {}  # (block, đoạn, dòng) -> chỉ số từ
        for i, word in enumerate(data["text"]):
            if word.strip() and float(data["conf"][i]) >= 0:  # conf -1: hàng cấu trúc, không phải từ
                key = (data["block_num"][i], data["par_num"][i], data["line_num"][i])
                lines.setdefault(key, []).append(i)
        result = []
        for idx in lines.values():
            x0 = min(data["left"][i] for i in idx)
            y0 = min(data["top"][i] for i in idx)
            x1 = max(data["left"][i] + data["width"][i] for i in idx)
            y1 = max(data["top"][i] + data["height"][i] for i in idx)
            confidence = sum(float(data["conf"][i]) for i in idx) / len(idx) / 100.0  # trung bình từ
            result.append(OcrLine(" ".join(data["text"][i] for i in idx), confidence, (x0, y0, x1, y1)))
        return result


class EasyOcrEngine:
    """EasyOCR: mỗi DÒNG có đa giác 4 đỉnh, chữ và điểm 0..1. Mô hình tải lần đầu vào ~/.EasyOCR."""

    name = "easyocr"

    def __init__(self, lang: str = "vi") -> None:
        try:
            import easyocr
        except ImportError as exc:
            raise EngineUnavailable("chưa cài easyocr (thêm --extra easyocr vào lệnh uv run)") from exc
        warnings.filterwarnings("ignore", category=UserWarning)  # cảnh báo torch về MPS, quantize
        self.lang = lang
        self._reader = easyocr.Reader([lang], gpu=False, verbose=False)  # ép CPU để so công bằng

    def describe(self) -> dict[str, str]:
        import torch
        from easyocr.config import detection_models, recognition_models

        sources = [detection_models["craft"]["url"], recognition_models["gen2"]["latin_g2"]["url"]]
        return {"engine": f"EasyOCR {_version('easyocr')}", "config": f"Reader(['{self.lang}'], gpu=False)",
                "torch": torch.__version__, "mô hình tải từ": " ".join(sources)}

    def model_files(self) -> list[Path]:
        folder = Path.home() / ".EasyOCR" / "model"  # nơi EasyOCR tải mô hình ở lần đầu
        return [folder / "craft_mlt_25k.pth", folder / "latin_g2.pth"]  # phát hiện + nhận dạng («vi» dùng bộ latin)

    def missing_chars(self, chars: str) -> list[str]:
        """Ký tự của `chars` không có trong bảng ký tự của mô hình nhận dạng latin_g2."""
        from easyocr.config import recognition_models

        model = recognition_models["gen2"]["latin_g2"]
        known = set(nfc_chars(model["characters"] + model["symbols"]))
        return sorted(set(nfc_chars(chars)) - known)

    def recognize(self, path: Path) -> list[OcrLine]:
        array = np.asarray(read_rgb(path))
        result = []
        for polygon, text, confidence in self._reader.readtext(array):
            xs = [int(p[0]) for p in polygon]
            ys = [int(p[1]) for p in polygon]
            result.append(OcrLine(text, float(confidence), (min(xs), min(ys), max(xs), max(ys))))
        return result


class PaddleOcrEngine:
    """PaddleOCR 3.x: `predict` trả dict có rec_texts, rec_scores (0..1) và rec_boxes (x0, y0, x1, y1).

    Ta tắt ba mô-đun phụ (xoay trang, nắn phẳng, xoay dòng): chúng cần thêm mô hình và việc hình
    học đã là của bài 42, nên so với hai engine kia cho công bằng.
    """

    def __init__(self, lang: str = "vi", ocr_version: str | None = None, *, name: str = "paddleocr") -> None:
        os.environ.setdefault("PADDLE_PDX_DISABLE_MODEL_SOURCE_CHECK", "True")  # bỏ bước dò mạng
        try:
            from paddleocr import PaddleOCR
        except ImportError as exc:
            hint = ("paddlepaddle chưa có wheel cho Python 3.14: chạy qua project con paddle/ (Python 3.13)"
                    if sys.version_info >= (3, 14) else "chưa cài paddleocr và paddlepaddle")
            raise EngineUnavailable(hint) from exc
        self.name, self.lang, self.ocr_version = name, lang, ocr_version
        options = {"ocr_version": ocr_version} if ocr_version else {}
        self._ocr = PaddleOCR(lang=lang, use_doc_orientation_classify=False, use_doc_unwarping=False,
                              use_textline_orientation=False, **options)

    def describe(self) -> dict[str, str]:
        return {"engine": f"PaddleOCR {_version('paddleocr')}", "paddlepaddle": _version("paddlepaddle"),
                "config": f"lang='{self.lang}'" + (f", ocr_version='{self.ocr_version}'" if self.ocr_version else ""),
                "mô hình tải từ": "nguồn do PaddleX chọn (Hugging Face, ModelScope hoặc BOS)",
                "mô hình lưu ở": str(self.model_files()[0].parent)}

    def model_files(self) -> list[Path]:
        folder = Path.home() / ".paddlex" / "official_models"  # nơi PaddleX lưu mô hình đã tải
        if self.ocr_version == "PP-OCRv5":
            names = ("PP-OCRv5_server_det", "latin_PP-OCRv5_mobile_rec")
        else:  # mặc định của paddleocr 3.7 cho lang='vi': PP-OCRv6 cỡ medium
            names = ("PP-OCRv6_medium_det", "PP-OCRv6_medium_rec")
        return [folder / name for name in names]

    def missing_chars(self, chars: str) -> list[str] | None:
        """Ký tự của `chars` không có trong `character_dict` của mô hình nhận dạng (inference.yml)."""
        try:
            import yaml
        except ImportError:
            return None
        config = yaml.safe_load((self.model_files()[1] / "inference.yml").read_text(encoding="utf-8"))
        known = set(nfc_chars("".join(config["PostProcess"]["character_dict"])))
        return sorted(set(nfc_chars(chars)) - known)

    def recognize(self, path: Path) -> list[OcrLine]:
        bgr = np.ascontiguousarray(np.asarray(read_rgb(path))[..., ::-1])  # Paddle nhận BGR như OpenCV
        result = []
        for page in self._ocr.predict(bgr):
            for text, score, box in zip(page["rec_texts"], page["rec_scores"], page["rec_boxes"], strict=True):
                if text.strip():
                    result.append(OcrLine(text, float(score), tuple(int(v) for v in box)))  # type: ignore[arg-type]
        return result


def make_engine(name: str) -> OcrEngine:
    """Tạo engine theo tên. Thiếu thư viện thì ném EngineUnavailable có hướng dẫn."""
    if name == "tesseract":
        return TesseractEngine()
    if name == "easyocr":
        return EasyOcrEngine()
    if name == "paddleocr":
        return PaddleOcrEngine()
    if name == "paddleocr-v5":
        return PaddleOcrEngine(ocr_version="PP-OCRv5", name="paddleocr-v5")
    raise ValueError(f"engine lạ: {name} (chọn một trong {', '.join(ENGINE_NAMES)})")
