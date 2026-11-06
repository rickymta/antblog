"""Cố ý sai: adapter đặt tên phương thức khác port. mypy phải bắt ở chỗ ghép (verify.sh kiểm)."""

from kyc_service.api.container import build_container
from kyc_service.domain.kyc import DocumentImage, OcrResult


class WrongOcr:
    def read_document(self, document: DocumentImage) -> OcrResult:  # sai tên: port gọi `read`
        raise NotImplementedError


build_container(ocr=WrongOcr())
