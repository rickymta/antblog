"""Integration test cho OcrDocumentStore trên MongoDB thật."""

import pytest
from pymongo.errors import DuplicateKeyError

from trips_db import OcrDocumentStore

# Số CCCD giả: 12 chữ số bắt đầu bằng 099 (mã tỉnh không tồn tại).
FIELDS = {"full_name": "Nguyễn Văn A", "id_number": "099000000001", "birth_date": "1990-01-02"}


def test_save_and_read_back(ocr_store: OcrDocumentStore) -> None:
    ocr_store.save("KYC-0001", "cccd", FIELDS, 0.93)

    doc = ocr_store.get("KYC-0001", "cccd")

    assert doc is not None
    assert doc["fields"]["full_name"] == "Nguyễn Văn A"   # tiếng Việt có dấu đi và về nguyên vẹn
    assert doc["confidence"] == 0.93
    assert doc["schema_version"] == 1


def test_created_at_comes_back_timezone_aware(ocr_store: OcrDocumentStore) -> None:
    ocr_store.save("KYC-0001", "cccd", FIELDS, 0.93)

    doc = ocr_store.get("KYC-0001", "cccd")

    # Mặc định PyMongo trả datetime naive; fixture tạo client với tz_aware=True để khớp quy ước UTC aware.
    assert doc["created_at"].tzinfo is not None


def test_unique_index_rejects_second_document_of_same_type(ocr_store: OcrDocumentStore) -> None:
    ocr_store.save("KYC-0001", "cccd", FIELDS, 0.93)

    with pytest.raises(DuplicateKeyError) as excinfo:
        ocr_store.save("KYC-0001", "cccd", FIELDS, 0.50)

    assert excinfo.value.code == 11000
    ocr_store.save("KYC-0001", "gplx", {"license_number": "990000000001"}, 0.88)  # loại khác thì được


def test_query_by_nested_field(ocr_store: OcrDocumentStore) -> None:
    ocr_store.save("KYC-0001", "cccd", FIELDS, 0.93)
    ocr_store.save("KYC-0002", "cccd", {**FIELDS, "id_number": "099000000002"}, 0.91)

    found = ocr_store.find_by_id_number("099000000002")

    assert [d["kyc_id"] for d in found] == ["KYC-0002"]
