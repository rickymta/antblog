"""Document kết quả OCR trong MongoDB: schema v2 (bài 26, số giấy tờ rõ) và v3 (đã che + băm): lưu, đọc, migrate.

Ba collection, mỗi cái một vòng đời dữ liệu:
- lab47_ocr_documents: kết quả đã che số giấy tờ, sống lâu (không TTL trong lab; xóa theo yêu cầu là việc khác).
- lab47_ocr_raw:       số giấy tờ rõ + văn bản thô + khóa ảnh, có TTL (retention ngắn).
- lab47_audit_log:     ai xem gì, có TTL dài hơn (xem app/audit.py).
"""

import re
from dataclasses import dataclass
from datetime import UTC, date, datetime, timedelta
from typing import Any

from pymongo import ASCENDING, DESCENDING
from pymongo.asynchronous.collection import AsyncCollection
from pymongo.asynchronous.database import AsyncDatabase
from pymongo.errors import DuplicateKeyError

from .pii import ID_FIELD, PiiKey, mask_digits, pii_hash

DOCS, RAW, AUDIT = "lab47_ocr_documents", "lab47_ocr_raw", "lab47_audit_log"
RAW_RETENTION = timedelta(days=30)  # giá trị lab; thời hạn thật do pháp chế và chính sách dữ liệu quyết định
SCHEMA_VERSION = 3  # bài 26 dừng ở 2: v2 còn số rõ trong fields
REVIEW_THRESHOLD = 0.8  # cùng ngưỡng của bài 26 và bài 46
MASK_PATTERN = "^[0-9]{3}[*]{7}[0-9]{2}$"  # 12 chữ số: 3 đầu, 7 sao, 2 cuối (CCCD và GPLX giả của loạt đều 12 số)
HASH_PATTERN = "^[0-9a-f]{64}$"
SHA256_PATTERN = HASH_PATTERN
ISO_DATE_PATTERN = "^[0-9]{4}-[0-9]{2}-[0-9]{2}$"
NUMBER_PATTERN = re.compile(r"[0-9]{12}")


def _nullable(bson_type: str, **extra: Any) -> dict[str, Any]:
    return {"bsonType": [bson_type, "null"], **extra}


# Tên trường được phép trong `fields` của v3: ngoài danh sách này (số rõ, `so_cccd`, chữ lạ) là bị từ chối.
V3_FIELD_PROPERTIES: dict[str, Any] = {
    "id_number_masked": _nullable("string", pattern=MASK_PATTERN),
    "id_number_hash": _nullable("string", pattern=HASH_PATTERN),
    "license_number_masked": _nullable("string", pattern=MASK_PATTERN),
    "license_number_hash": _nullable("string", pattern=HASH_PATTERN),
    "full_name": _nullable("string"),
    "date_of_birth": _nullable("string", pattern=ISO_DATE_PATTERN),
    "sex": _nullable("string"),
    "hometown_city": _nullable("string"),
    "license_class": _nullable("string"),
    "expiry_date": _nullable("string", pattern=ISO_DATE_PATTERN),
}


def documents_validator(*, allow_legacy: bool) -> dict[str, Any]:
    """$jsonSchema của collection chính. Trong thời gian chuyển đổi nhận cả v2 (bài 26) lẫn v3; migrate xong thì siết lại
    chỉ còn v3 (collMod) để không ai ghi số giấy tờ rõ vào đây nữa. v3 đóng kín: không nhận trường lạ ở cả hai cấp."""
    required = ["kyc_id", "document_type", "schema_version", "model_version", "image_sha256", "pii_key_id", "fields",
                "field_confidence", "confidence", "needs_review", "missing_fields", "raw_id", "created_at"]
    v3: dict[str, Any] = {
        "bsonType": "object",
        "additionalProperties": False,
        "required": required,
        "properties": {
            "_id": {},
            "kyc_id": {"bsonType": "string", "pattern": "^KYC-[0-9]{4}$"},
            "document_type": {"enum": ["cccd", "gplx"]},
            "schema_version": {"enum": [3]},
            "model_version": {"bsonType": "string"},
            "image_sha256": _nullable("string", pattern=SHA256_PATTERN),  # null: document chuyển từ v2, không còn biết ảnh
            "pii_key_id": {"bsonType": "string"},
            "fields": {
                "bsonType": "object",
                "additionalProperties": False,
                "properties": V3_FIELD_PROPERTIES,
                "anyOf": [{"required": ["id_number_masked", "id_number_hash"]},
                          {"required": ["license_number_masked", "license_number_hash"]}],
            },
            "field_confidence": {"bsonType": ["object", "null"]},
            "confidence": {"bsonType": "double", "minimum": 0, "maximum": 1},
            "needs_review": {"bsonType": "bool"},
            "missing_fields": {"bsonType": "array", "items": {"bsonType": "string"}},
            "raw_id": {"bsonType": "objectId"},
            "created_at": {"bsonType": "date"},
        },
    }
    legacy: dict[str, Any] = {
        "bsonType": "object",
        "required": ["kyc_id", "document_type", "fields", "confidence", "model_version", "schema_version", "created_at"],
        "properties": {"schema_version": {"enum": [2]}, "confidence": {"bsonType": "double", "minimum": 0, "maximum": 1}},
    }
    return {"$jsonSchema": {"bsonType": "object", "oneOf": [legacy, v3]} if allow_legacy else v3}


async def reset_and_setup(db: AsyncDatabase[dict[str, Any]], *, allow_legacy: bool = True) -> None:
    """XÓA rồi dựng lại ba collection và index. Chỉ dùng cho lab và kiểm thử; triển khai thật dùng migration, không drop."""
    for name in (DOCS, RAW, AUDIT):
        await db[name].drop()
    await db.create_collection(DOCS, validator=documents_validator(allow_legacy=allow_legacy))
    docs, raw = db[DOCS], db[RAW]
    # Một kết quả cho mỗi (hồ sơ, loại giấy tờ, phiên bản mô hình, ẢNH): chụp lại ảnh là kết quả mới, không đè kết quả
    # ảnh trước; lưu lại cùng ảnh là idempotent.
    unique_key = [("kyc_id", ASCENDING), ("document_type", ASCENDING), ("model_version", ASCENDING),
                  ("image_sha256", ASCENDING)]
    await docs.create_index(unique_key, unique=True, name="uq_kyc_type_model_image")
    await docs.create_index([("kyc_id", ASCENDING), ("document_type", ASCENDING), ("created_at", DESCENDING)],
                            name="ix_latest")
    # Đối chiếu theo số giấy tờ: tra theo giá trị băm, không bao giờ theo số rõ.
    await docs.create_index([("document_type", ASCENDING), ("fields.id_number_hash", ASCENDING)], name="ix_id_hash")
    await docs.create_index([("document_type", ASCENDING), ("fields.license_number_hash", ASCENDING)], name="ix_license_hash")
    await raw.create_index(unique_key, unique=True, name="uq_raw")
    # TTL: MongoDB xóa document khi `expire_at` (BẮT BUỘC kiểu date) đã qua; expireAfterSeconds=0 nghĩa là «đúng lúc đó».
    await raw.create_index("expire_at", expireAfterSeconds=0, name="ttl_raw_expire_at")
    # Audit sống lâu hơn dữ liệu nó mô tả, nên TTL dài hơn (xem app/audit.py).
    await db[AUDIT].create_index("expire_at", expireAfterSeconds=0, name="ttl_audit_expire_at")


def to_iso_date(card_text: str) -> str:
    """Trên thẻ ngày sinh in dd/mm/yyyy (OCR đọc đúng chuỗi đó); khi lưu ta chuyển sang ISO yyyy-mm-dd (bài 26).
    Ngày không có thật (31/02/1990) là ValueError."""
    day, month, year = card_text.split("/")
    return date(int(year), int(month), int(day)).isoformat()


@dataclass(frozen=True)
class SaveResult:
    document: dict[str, Any]
    created: bool  # False: kết quả này đã có, trả lại bản cũ


def protect_fields(
    document_type: str, fields: dict[str, Any], key: PiiKey, *, dob_on_card: bool,
) -> tuple[dict[str, Any], list[str], str | None]:
    """Thay số giấy tờ rõ bằng dạng che và dạng băm; các trường khác giữ nguyên.

    Trả về (fields mới, danh sách trường thiếu hoặc không hợp lệ, số rõ nếu hợp lệ). Số không đủ 12 chữ số hoặc ngày không
    có thật KHÔNG làm hàm lỗi: trường đó thành None và được ghi vào `missing_fields`, document vào hàng chờ duyệt. Mọi biến
    đổi chạy TRƯỚC khi ghi bất kỳ thứ gì, để một đầu vào xấu không để lại `raw` mồ côi.
    `dob_on_card`: ngày sinh đang là dd/mm/yyyy in trên thẻ; document v2 đã lưu ISO sẵn.
    """
    name = ID_FIELD[document_type]
    out = {k: v for k, v in fields.items() if k != name}
    missing = [k for k, v in fields.items() if k != name and v in (None, "")]
    number = fields.get(name)
    valid_number = number if isinstance(number, str) and NUMBER_PATTERN.fullmatch(number) else None
    out[f"{name}_masked"] = mask_digits(valid_number) if valid_number else None
    out[f"{name}_hash"] = pii_hash(valid_number, document_type, key) if valid_number else None
    if valid_number is None:
        missing.append(name)
    if dob_on_card and out.get("date_of_birth"):
        try:
            out["date_of_birth"] = to_iso_date(out["date_of_birth"])
        except ValueError:
            out["date_of_birth"] = None
            missing.append("date_of_birth")
    return out, sorted(set(missing)), valid_number


async def save_ocr_result(
    db: AsyncDatabase[dict[str, Any]], output: dict[str, Any], *, image_key: str, image_sha256: str, key: PiiKey,
    now: datetime | None = None,
) -> SaveResult:
    """Lưu một kết quả OCR (dạng job của bài 46). Khóa là (hồ sơ, loại, mô hình, sha256 ảnh).

    Thứ tự: kiểm document đã có (không đụng tới `raw` nếu có, kể cả khi `raw` đã hết hạn), biến đổi dữ liệu, ghi `raw`,
    ghi document. Chết giữa chừng thì `raw` mồ côi tự hết hạn bằng TTL; chạy lại là idempotent nhờ unique index."""
    now = (now or datetime.now(UTC)).replace(microsecond=0)
    kyc_id, document_type, model = output["kyc_id"], output["document_type"], output["model_version"]
    id_key = {"kyc_id": kyc_id, "document_type": document_type, "model_version": model, "image_sha256": image_sha256}
    existing = await db[DOCS].find_one(id_key)
    if existing is not None:
        return SaveResult(existing, False)

    fields, missing, number = protect_fields(document_type, output["fields"], key, dob_on_card=True)
    await db[RAW].update_one(id_key, {"$setOnInsert": {
        "image_key": image_key, "raw_text": output.get("raw_text"),
        "plain": {ID_FIELD[document_type]: number} if number else {},
        "created_at": now, "expire_at": now + RAW_RETENTION}}, upsert=True)
    raw = await db[RAW].find_one(id_key)
    assert raw is not None

    doc = {
        **id_key,
        "schema_version": SCHEMA_VERSION,
        "pii_key_id": key.key_id,
        "fields": fields,
        "field_confidence": output["field_confidence"],
        "confidence": float(output["confidence"]),
        # Thiếu hoặc sai số giấy tờ, ngày sinh... là việc của người duyệt: lưu lại và đánh dấu, không bỏ kết quả.
        "needs_review": bool(output["needs_review"]) or bool(missing),
        "missing_fields": missing,
        "raw_id": raw["_id"],
        "created_at": now,
    }
    try:
        await db[DOCS].insert_one(doc)
        return SaveResult(doc, True)
    except DuplicateKeyError:
        existing = await db[DOCS].find_one(id_key)
        assert existing is not None
        return SaveResult(existing, False)


def normalize_to_v3(doc: dict[str, Any], key: PiiKey) -> dict[str, Any]:
    """Upgrade on read: ứng dụng luôn thấy hình dạng v3 (đã che), kể cả khi DB còn document v2 của bài 26. Thao tác thuần,
    không ghi lại DB; số rõ của v2 không đi qua tầng trên. Document v1 của bài 26 phải qua migration của bài 26 trước."""
    version = doc["schema_version"]
    if version >= 3:
        return doc
    if version != 2:
        raise ValueError(f"schema_version {version} chưa hỗ trợ: chạy migration v1 sang v2 của bài 26 trước")
    fields, missing, _ = protect_fields(doc["document_type"], doc["fields"], key, dob_on_card=False)
    return {**doc, "schema_version": 3, "pii_key_id": key.key_id, "image_sha256": None, "raw_id": doc.get("raw_id"),
            "fields": fields, "missing_fields": missing, "field_confidence": doc.get("field_confidence"),
            "needs_review": doc["confidence"] < REVIEW_THRESHOLD or bool(missing)}


async def find_by_number(
    db: AsyncDatabase[dict[str, Any]], number: str, document_type: str, keys: list[PiiKey],
) -> list[dict[str, Any]]:
    """Các document cùng số giấy tờ: băm số cần tìm bằng từng khóa còn hiệu lực rồi tra index; kết quả chỉ có dạng che.
    Hàm trần, KHÔNG ghi audit: ứng dụng phải gọi qua `access.lookup_by_number`."""
    name = ID_FIELD[document_type]
    hits: list[dict[str, Any]] = []
    for key in keys:
        cursor = db[DOCS].find({"document_type": document_type, "pii_key_id": key.key_id,
                                f"fields.{name}_hash": pii_hash(number, document_type, key)},
                               {"_id": 0, "kyc_id": 1, "document_type": 1, f"fields.{name}_masked": 1, "created_at": 1})
        hits += await cursor.to_list(length=100)
    return hits


async def migrate_v2_to_v3(db: AsyncDatabase[dict[str, Any]], key: PiiKey, *, batch: int = 100) -> int:
    """Chuyển document v2 (bài 26) sang v3 theo lô. An toàn khi chạy lại hoặc chạy song song hai bản:
    1. ghi raw (setOnInsert) với expire_at = created_at + retention, tức số rõ vẫn nằm trong thời hạn lưu cũ;
    2. sửa document với bộ lọc `schema_version: 2`: bản chạy sau thấy 0 document khớp, không sửa lần hai.
    Trả về số document do lời gọi NÀY sửa. Ảnh gốc của v2 không còn được biết nên `image_sha256` là null."""
    docs: AsyncCollection[dict[str, Any]] = db[DOCS]
    modified = 0
    while True:
        old = await docs.find({"schema_version": 2}).limit(batch).to_list(length=batch)
        if not old:
            return modified
        for doc in old:
            document_type = doc["document_type"]
            fields, missing, number = protect_fields(document_type, doc["fields"], key, dob_on_card=False)
            raw_key = {"kyc_id": doc["kyc_id"], "document_type": document_type, "model_version": doc["model_version"],
                       "image_sha256": None}
            await db[RAW].update_one(raw_key, {"$setOnInsert": {
                "image_key": None, "raw_text": None, "plain": {ID_FIELD[document_type]: number} if number else {},
                "migrated_from": 2, "created_at": doc["created_at"], "expire_at": doc["created_at"] + RAW_RETENTION}},
                upsert=True)
            raw = await db[RAW].find_one(raw_key)
            assert raw is not None
            result = await docs.update_one(
                {"_id": doc["_id"], "schema_version": 2},
                {"$set": {"fields": fields, "schema_version": SCHEMA_VERSION, "pii_key_id": key.key_id,
                          "image_sha256": None, "raw_id": raw["_id"], "missing_fields": missing,
                          "field_confidence": doc.get("field_confidence"),
                          "needs_review": doc["confidence"] < REVIEW_THRESHOLD or bool(missing)}})
            modified += result.modified_count
