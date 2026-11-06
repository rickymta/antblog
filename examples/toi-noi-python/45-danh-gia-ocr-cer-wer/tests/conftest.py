import pytest

import evaluation as ev
from ocr import TesseractMissing


@pytest.fixture(scope="session")
def golden():
    """Bộ test vàng, kết quả pipeline và chữ cả trang: chạy OCR một lần cho cả phiên test."""
    try:
        items = ev.build_dataset()
        return items, ev.run_pipeline(items), ev.page_texts(items)
    except TesseractMissing as exc:
        # Không skip: regression test mà lặng lẽ bị bỏ qua thì không bảo vệ được gì.
        pytest.fail(f"cần Tesseract và gói ngôn ngữ vie để chạy bộ test vàng: {exc}")
