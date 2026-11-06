"""Bài 26: lưu kết quả OCR giấy tờ dạng document bằng PyMongo async (AsyncMongoClient).

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify

Cần MongoDB trong hạ tầng dùng chung:
    docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Lab chỉ đụng vào collection `lab26_ocr_documents` và xóa nó khi chạy xong.

Dữ liệu là tổng hợp: CCCD giả bắt đầu 099, GPLX giả bắt đầu 99, tên giả. Kết quả OCR (và
độ tin cậy đi kèm) là đầu vào cho người duyệt hồ sơ, không tự động là quyết định xác minh.
"""

import asyncio
import json
import os
import random
import sys
from datetime import UTC, date, datetime, timedelta
from typing import Any
from zoneinfo import ZoneInfo

from pymongo import ASCENDING, DESCENDING, AsyncMongoClient
from pymongo.asynchronous.collection import AsyncCollection
from pymongo.errors import DuplicateKeyError, PyMongoError, WriteError

MONGO_URL = os.environ.get("TOINOI_MONGO_URL", "mongodb://localhost:27019/toinoi")
COLLECTION = "lab26_ocr_documents"
REVIEW_THRESHOLD = 0.8  # dưới ngưỡng này, hồ sơ vào hàng đợi cho người duyệt
SEED = 26
KYC_COUNT = 1000  # mỗi hồ sơ có một CCCD và một GPLX -> 2.000 document
HCM = ZoneInfo("Asia/Ho_Chi_Minh")
CITIES = ("Hà Nội", "TP.HCM", "Đà Nẵng", "Hải Phòng", "Cần Thơ", "Nha Trang")
NAMES_BY_SEX = {
    "Nam": ("Nguyễn Văn A", "Lê Văn C", "Hoàng Văn E"),
    "Nữ": ("Trần Thị B", "Phạm Thị D", "Vũ Thị F"),
}
NAMES = NAMES_BY_SEX["Nam"] + NAMES_BY_SEX["Nữ"]  # dùng cho GPLX, không ghi giới tính

# $jsonSchema chặn những document sai hình dạng ngay tại server. Đây vẫn là "schema",
# chỉ là schema được khai báo ở collection thay vì ở bảng.
OCR_VALIDATOR: dict[str, Any] = {
    "$jsonSchema": {
        "bsonType": "object",
        "required": [
            "kyc_id", "document_type", "fields", "confidence",
            "model_version", "schema_version", "created_at",
        ],
        "properties": {
            "kyc_id": {"bsonType": "string", "pattern": "^KYC-[0-9]{4}$"},
            "document_type": {"enum": ["cccd", "gplx"]},
            "fields": {"bsonType": "object"},
            "confidence": {"bsonType": "double", "minimum": 0, "maximum": 1},
            "schema_version": {"bsonType": "int", "minimum": 1, "maximum": 2},
            "created_at": {"bsonType": "date"},
        },
    }
}

QUIET = False  # --verify chỉ in dòng SAI và kết luận


def say(*parts: object) -> None:
    if not QUIET:
        print(*parts)


class Checker:
    """Gom các so sánh kỳ vọng; verify dùng tổng số và danh sách ca sai."""

    def __init__(self) -> None:
        self.total = 0
        self.failures: list[str] = []

    def check(self, name: str, got: object, expected: object) -> None:
        self.total += 1
        if got != expected:
            self.failures.append(name)
            print(f"SAI: {name}: nhận {got!r}, kỳ vọng {expected!r}")


def show(doc: dict[str, Any]) -> str:
    """In document gọn: bỏ _id (ObjectId đổi mỗi lần chạy), datetime thành chuỗi."""
    return json.dumps({k: v for k, v in doc.items() if k != "_id"},
                      ensure_ascii=False, indent=2, default=str)


# ---------------------------------------------------------------- dữ liệu tổng hợp


def make_id_number(rng: random.Random, sex: str, birth_year: int) -> str:
    """CCCD giả 12 số: `099` (mã tỉnh không tồn tại) + 1 số giới tính/thế kỷ + 2 số năm + 6 số ngẫu nhiên.

    Thế kỷ 20: nam 0, nữ 1; thế kỷ 21: nam 2, nữ 3.
    """
    century_digit = (0 if birth_year < 2000 else 2) + (0 if sex == "Nam" else 1)
    return f"099{century_digit}{birth_year % 100:02d}" + "".join(rng.choices("0123456789", k=6))


def to_iso_date(card_text: str) -> str:
    """Trên thẻ ngày sinh in dd/mm/yyyy (OCR đọc đúng chuỗi đó); khi lưu ta chuyển sang ISO yyyy-mm-dd."""
    day, month, year = card_text.split("/")
    return date(int(year), int(month), int(day)).isoformat()


def make_document(rng: random.Random, n: int, document_type: str) -> dict[str, Any]:
    """Một kết quả OCR schema_version 2: có độ tin cậy từng trường, `confidence` = nhỏ nhất."""
    if document_type == "cccd":
        sex = rng.choice(("Nam", "Nữ"))
        year = rng.randint(1970, 2003)
        card_date = f"{rng.randint(1, 28):02d}/{rng.randint(1, 12):02d}/{year}"
        fields = {
            "id_number": make_id_number(rng, sex, year),
            "full_name": rng.choice(NAMES_BY_SEX[sex]),
            "date_of_birth": to_iso_date(card_date),
            "sex": sex,
            "hometown_city": rng.choice(CITIES),
        }
    else:
        fields = {
            "license_number": "99" + "".join(rng.choices("0123456789", k=10)),
            "full_name": rng.choice(NAMES),
            "license_class": rng.choice(("A1", "A2", "B1", "B2")),
            "expiry_date": f"{rng.randint(2027, 2035)}-{rng.randint(1, 12):02d}-{rng.randint(1, 28):02d}",
        }
    # Khoảng 4% hồ sơ có một trường đọc kém: đó là hàng đợi cho người duyệt.
    low_field = rng.choice(list(fields)) if rng.random() < 0.04 else None
    field_confidence = {
        name: round(rng.uniform(0.30, 0.78) if name == low_field else rng.uniform(0.86, 0.995), 3)
        for name in fields
    }
    return {
        "kyc_id": f"KYC-{n:04d}",
        "document_type": document_type,
        "fields": fields,
        "field_confidence": field_confidence,
        "confidence": min(field_confidence.values()),
        "model_version": "demo-ocr-1.0",
        "schema_version": 2,
        "created_at": datetime(2026, 11, 1, 2, 0, tzinfo=UTC) + timedelta(seconds=30 * n),
    }


def make_legacy_documents() -> list[dict[str, Any]]:
    """Năm document schema_version 1 từ mô hình cũ: một trường `name`, không có field_confidence."""
    rng = random.Random(SEED + 1)
    return [
        {
            "kyc_id": f"KYC-{1000 + i:04d}",
            "document_type": "cccd",
            "fields": {
                "id_number": make_id_number(rng, "Nam", 1990),
                "name": rng.choice(NAMES),
                "date_of_birth": "1990-01-01",
            },
            "confidence": round(rng.uniform(0.85, 0.99), 3),
            "model_version": "demo-ocr-0.9",
            "schema_version": 1,
            "created_at": datetime(2026, 6, 1, 2, 0, tzinfo=UTC) + timedelta(minutes=i),
        }
        for i in range(1, 6)
    ]


def normalize_to_v2(doc: dict[str, Any]) -> dict[str, Any]:
    """Upgrade on read: ứng dụng luôn làm việc với hình dạng v2, DB có thể còn lẫn v1."""
    if doc["schema_version"] >= 2:
        return doc
    fields = dict(doc["fields"])
    fields["full_name"] = fields.pop("name")
    # v1 chưa từng có độ tin cậy từng trường: trả None, không bịa số.
    return {**doc, "fields": fields, "field_confidence": None, "schema_version": 2}


# ---------------------------------------------------------------- truy vấn


def plan_summary(explain: dict[str, Any]) -> tuple[str, int, int]:
    """(giai đoạn lá của kế hoạch thắng, số document phải kiểm, số document trả về)."""
    plan = explain["queryPlanner"]["winningPlan"]
    plan = plan.get("queryPlan", plan)
    while "inputStage" in plan:
        plan = plan["inputStage"]
    stats = explain["executionStats"]
    return plan["stage"], stats["totalDocsExamined"], stats["nReturned"]


def plan_stages(explain: dict[str, Any]) -> list[str]:
    """Chuỗi giai đoạn của kế hoạch thắng, từ ngoài vào trong (vd. SORT, FETCH, IXSCAN)."""
    plan = explain["queryPlanner"]["winningPlan"]
    plan = plan.get("queryPlan", plan)
    stages = [plan["stage"]]
    while "inputStage" in plan:
        plan = plan["inputStage"]
        stages.append(plan["stage"])
    return stages


async def review_queue(col: AsyncCollection[dict[str, Any]], limit: int) -> list[dict[str, Any]]:
    """Hàng đợi duyệt: độ tin cậy dưới ngưỡng, thấp nhất trước. Projection chỉ lấy trường cần."""
    cursor = (
        col.find({"confidence": {"$lt": REVIEW_THRESHOLD}},
                 {"_id": 0, "kyc_id": 1, "document_type": 1, "confidence": 1})
        .sort("confidence", ASCENDING)
        .limit(limit)
    )
    return await cursor.to_list(length=limit)


# ---------------------------------------------------------------- các phần của bài


async def connect() -> AsyncMongoClient[dict[str, Any]]:
    # tz_aware=True: datetime đọc ra có tzinfo (mặc định PyMongo trả datetime không múi giờ).
    client: AsyncMongoClient[dict[str, Any]] = AsyncMongoClient(
        MONGO_URL, tz_aware=True, serverSelectionTimeoutMS=2000
    )
    try:
        await client.admin.command("ping")
    except PyMongoError:
        await client.close()
        print("FAIL: không nối được MongoDB tại "
              f"{MONGO_URL} (docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait)")
        sys.exit(1)
    return client


async def part_insert(col: AsyncCollection[dict[str, Any]], c: Checker) -> None:
    say("== 1. Chèn: insert_one và insert_many ==")
    rng = random.Random(SEED)
    docs = [make_document(rng, n, t) for n in range(1, KYC_COUNT + 1) for t in ("cccd", "gplx")]

    first = await col.insert_one(docs[0])
    say("insert_one ->", type(first.inserted_id).__name__, "(driver tự thêm _id)")
    many = await col.insert_many(docs[1:])
    say("insert_many ->", len(many.inserted_ids), "document")
    legacy = await col.insert_many(make_legacy_documents())
    say("insert_many (schema v1) ->", len(legacy.inserted_ids), "document")

    c.check("insert_one trả ObjectId", type(first.inserted_id).__name__, "ObjectId")
    c.check("insert_many 1999", len(many.inserted_ids), 1999)
    c.check("tổng document", await col.count_documents({}), 2005)
    say("count_documents ->", await col.count_documents({}))


async def part_read(col: AsyncCollection[dict[str, Any]], c: Checker) -> None:
    say("\n== 2. Đọc: find_one, projection, sort, limit ==")
    doc = await col.find_one({"kyc_id": "KYC-0001", "document_type": "cccd"})
    assert doc is not None
    say(show(doc))
    c.check("find_one KYC-0001 cccd", doc["fields"]["id_number"].startswith("099"), True)
    c.check("CCCD có 5 trường chung", sorted(doc["fields"]),
            ["date_of_birth", "full_name", "hometown_city", "id_number", "sex"])
    fields = doc["fields"]
    century_digit = int(fields["id_number"][3])
    c.check("id_number: 099 + giới tính/thế kỷ + 2 số năm",
            (len(fields["id_number"]), century_digit % 2 == (0 if fields["sex"] == "Nam" else 1),
             fields["id_number"][4:6] == fields["date_of_birth"][2:4]),
            (12, True, True))
    c.check("date_of_birth lưu ISO", date.fromisoformat(fields["date_of_birth"]).isoformat(),
            fields["date_of_birth"])
    c.check("dd/mm/yyyy trên thẻ -> ISO", to_iso_date("05/03/1990"), "1990-03-05")
    c.check("confidence là min field_confidence", doc["confidence"], min(doc["field_confidence"].values()))

    queue = await review_queue(col, 3)
    say("\nHàng đợi duyệt (confidence < 0.8, thấp nhất trước, 3 dòng đầu):")
    for row in queue:
        say(" ", row)
    low_total = await col.count_documents({"confidence": {"$lt": REVIEW_THRESHOLD}})
    say("Tổng số document dưới ngưỡng:", low_total)
    c.check("hàng đợi sắp tăng dần", [r["confidence"] for r in queue],
            sorted(r["confidence"] for r in queue))
    c.check("projection bỏ fields", all("fields" not in r for r in queue), True)
    c.check("số document dưới ngưỡng", low_total, 77)

    # Ba truy vấn đếm chạy đồng thời trên cùng client (cùng pool kết nối).
    cccd, gplx, legacy = await asyncio.gather(
        col.count_documents({"document_type": "cccd"}),
        col.count_documents({"document_type": "gplx"}),
        col.count_documents({"schema_version": 1}),
    )
    say(f"gather: cccd={cccd}, gplx={gplx}, schema v1={legacy}")
    c.check("đếm theo loại", (cccd, gplx, legacy), (1005, 1000, 5))


async def part_constraints(col: AsyncCollection[dict[str, Any]], c: Checker) -> None:
    say("\n== 3. Ràng buộc: unique index và validator ==")
    await col.create_index(
        [("kyc_id", ASCENDING), ("document_type", ASCENDING), ("model_version", ASCENDING)],
        unique=True, name="uq_kyc_type_model",
    )
    original = await col.find_one({"kyc_id": "KYC-0001", "document_type": "cccd"})
    assert original is not None
    duplicate = {k: v for k, v in original.items() if k != "_id"}
    try:
        await col.insert_one(duplicate)
        code: int | None = None
    except DuplicateKeyError as exc:
        code = exc.code
    say("chèn trùng (kyc_id, document_type, model_version) -> DuplicateKeyError, code", code)
    c.check("DuplicateKeyError code 11000", code, 11000)

    bad = {**duplicate, "model_version": "demo-ocr-bad", "confidence": 1.7}
    try:
        await col.insert_one(bad)
        bad_code: int | None = None
    except WriteError as exc:
        bad_code = exc.code
    say("chèn confidence = 1.7 -> WriteError, code", bad_code, "(DocumentValidationFailure)")
    c.check("WriteError code 121", bad_code, 121)

    # Cùng hồ sơ, mô hình mới hơn: đây là document mới, không ghi đè kết quả cũ.
    newer = {**duplicate, "model_version": "demo-ocr-1.1",
             "created_at": datetime(2026, 11, 2, 2, 0, tzinfo=UTC)}
    await col.insert_one(newer)
    latest = await col.find_one({"kyc_id": "KYC-0001", "document_type": "cccd"},
                                sort=[("created_at", DESCENDING)])
    assert latest is not None
    say("bản mới nhất của KYC-0001 cccd:", latest["model_version"])
    c.check("bản mới nhất theo created_at", latest["model_version"], "demo-ocr-1.1")
    history = await col.count_documents({"kyc_id": "KYC-0001", "document_type": "cccd"})
    c.check("lịch sử còn 2 bản", history, 2)


async def part_index(col: AsyncCollection[dict[str, Any]], c: Checker) -> None:
    say("\n== 4. Index và explain ==")
    query = {"confidence": {"$lt": REVIEW_THRESHOLD}}

    before = plan_summary(await col.find(query).explain())
    say(f"không có index confidence: {before[0]}, kiểm {before[1]} document, trả {before[2]}")
    await col.create_index([("confidence", ASCENDING)], name="ix_confidence")
    after = plan_summary(await col.find(query).explain())
    say(f"có index confidence:       {after[0]}, kiểm {after[1]} document, trả {after[2]}")

    by_kyc = plan_summary(await col.find({"kyc_id": "KYC-0500", "document_type": "gplx"}).explain())
    say(f"tìm theo kyc_id + document_type: {by_kyc[0]}, kiểm {by_kyc[1]} document, trả {by_kyc[2]}")

    # Truy vấn «bản mới nhất của một hồ sơ và loại giấy tờ»: có giai đoạn SORT cho tới khi có index
    # ghép kết thúc bằng created_at giảm dần.
    latest = {"kyc_id": "KYC-0001", "document_type": "cccd"}
    sort_before = plan_stages(await col.find(latest).sort("created_at", DESCENDING).limit(1).explain())
    await col.create_index(
        [("kyc_id", ASCENDING), ("document_type", ASCENDING), ("created_at", DESCENDING)],
        name="ix_latest",
    )
    sort_after = plan_stages(await col.find(latest).sort("created_at", DESCENDING).limit(1).explain())
    say("bản mới nhất, chưa có ix_latest:", " -> ".join(sort_before))
    say("bản mới nhất, có ix_latest:     ", " -> ".join(sort_after))

    c.check("trước index: COLLSCAN", before[0], "COLLSCAN")
    c.check("bản mới nhất: có SORT khi chưa có ix_latest", sort_before[0], "SORT")
    c.check("bản mới nhất: không còn SORT khi có ix_latest", "SORT" in sort_after, False)
    c.check("trước index: kiểm cả collection", before[1], 2006)
    c.check("sau index: IXSCAN", after[0], "IXSCAN")
    c.check("sau index: chỉ kiểm document trả về", after[1], after[2])
    c.check("unique index phục vụ truy vấn theo kyc_id", (by_kyc[0], by_kyc[1]), ("IXSCAN", 1))


async def part_datetime(col: AsyncCollection[dict[str, Any]], c: Checker) -> None:
    say("\n== 5. Thời gian: UTC aware và độ chính xác mili giây ==")
    sent = datetime(2026, 11, 3, 2, 0, 0, 123456, tzinfo=UTC)
    doc = {**(await col.find_one({"kyc_id": "KYC-0002", "document_type": "gplx"}) or {}),
           "model_version": "demo-ocr-ms", "created_at": sent}
    doc.pop("_id")
    await col.insert_one(doc)
    back = await col.find_one({"model_version": "demo-ocr-ms"})
    assert back is not None
    got = back["created_at"]
    say("gửi vào :", sent.isoformat())
    say("đọc ra  :", got.isoformat(), "(MongoDB giữ mili giây, bỏ phần micro giây)")
    say("giờ Việt Nam:", got.astimezone(HCM).isoformat())
    c.check("created_at có tzinfo", got.tzinfo is not None, True)
    c.check("micro giây bị cắt còn mili giây", got.microsecond, 123000)
    c.check("cùng thời điểm", got == sent.replace(microsecond=123000), True)
    c.check("giờ hiển thị +07:00", got.astimezone(HCM).isoformat(), "2026-11-03T09:00:00.123000+07:00")


async def part_schema_version(col: AsyncCollection[dict[str, Any]], c: Checker) -> None:
    say("\n== 6. schema_version: upgrade on read rồi migrate ==")
    old = await col.find_one({"schema_version": 1}, sort=[("kyc_id", ASCENDING)])
    assert old is not None
    say("v1 trong DB    :", old["fields"])
    upgraded = normalize_to_v2(old)
    say("v1 đọc thành v2:", upgraded["fields"])
    c.check("v1 có trường name", "name" in old["fields"], True)
    c.check("upgrade on read đổi tên trường", sorted(upgraded["fields"]),
            ["date_of_birth", "full_name", "id_number"])

    # Migrate một lần bằng update với aggregation pipeline: đổi tên trường ngay trong server.
    result = await col.update_many(
        {"schema_version": 1},
        [{"$set": {"fields.full_name": "$fields.name", "field_confidence": None, "schema_version": 2}},
         {"$unset": "fields.name"}],
    )
    say("update_many (pipeline) -> sửa", result.modified_count, "document")
    # Với AsyncCollection, aggregate() là coroutine trả về cursor: phải await rồi mới to_list.
    agg_cursor = await col.aggregate(
        [{"$group": {"_id": "$schema_version", "n": {"$sum": 1}}}, {"$sort": {"_id": 1}}]
    )
    groups = await agg_cursor.to_list(length=None)
    say("Sau migrate, theo schema_version:", {g["_id"]: g["n"] for g in groups})
    migrated = await col.find_one({"kyc_id": old["kyc_id"], "model_version": "demo-ocr-0.9"})
    assert migrated is not None
    c.check("modified_count", result.modified_count, 5)
    c.check("chỉ còn schema_version 2", {g["_id"]: g["n"] for g in groups}, {2: 2007})
    c.check("trường name đã biến mất", "name" in migrated["fields"], False)
    c.check("full_name giữ giá trị cũ", migrated["fields"]["full_name"], old["fields"]["name"])
    # Sau migrate mọi document v2 đều có khóa field_confidence (giá trị null cho document cũ),
    # nên đọc doc["field_confidence"] không bao giờ KeyError.
    missing_key = await col.count_documents({"field_confidence": {"$exists": False}})
    say("document v2 thiếu khóa field_confidence:", missing_key)
    say("field_confidence của document đã migrate:", migrated["field_confidence"])
    c.check("không document nào thiếu khóa field_confidence", missing_key, 0)
    c.check("document migrate có field_confidence None", migrated["field_confidence"], None)
    c.check("normalize_to_v2 không đổi document v2", normalize_to_v2(migrated), migrated)


async def run() -> int:
    c = Checker()
    client = await connect()
    try:
        col = client.get_default_database("toinoi")[COLLECTION]
        await col.drop()
        await col.database.create_collection(COLLECTION, validator=OCR_VALIDATOR)
        await part_insert(col, c)
        await part_read(col, c)
        await part_constraints(col, c)
        await part_index(col, c)
        await part_datetime(col, c)
        await part_schema_version(col, c)
    finally:
        await client.get_default_database("toinoi")[COLLECTION].drop()  # dọn dẹp
        await client.close()

    if c.failures:
        print(f"FAIL {c.total - len(c.failures)}/{c.total}")
        return 1
    if QUIET:
        print(f"OK {c.total}/{c.total}")
    else:
        say(f"\nĐã dọn collection {COLLECTION}. Kiểm chứng: {c.total}/{c.total} ca đạt.")
    return 0


def main() -> None:
    global QUIET
    QUIET = "--verify" in sys.argv[1:]
    sys.exit(asyncio.run(run()))


if __name__ == "__main__":
    main()
