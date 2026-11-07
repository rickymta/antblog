---
slug: python-mongodb-pymongo-motor-document-ocr
title: "MongoDB với pymongo/motor: lưu kết quả OCR dạng document"
summary: "Bài 26: lưu kết quả OCR giấy tờ thành document MongoDB bằng AsyncMongoClient của PyMongo: insert, find, unique index, validator, explain và schema_version; Motor đã deprecated. Có lab chạy thật."
category: python-intermediate
level: intermediate
tags: [python, toi-noi, mongodb, pymongo, ocr]
language: vi
---

Tài xế DR-001 nộp ảnh CCCD và GPLX để đăng ký. Dịch vụ `kyc-ocr` của [Tới Nơi](/articles/du-an-mau-toi-noi) đọc ra số giấy tờ, họ tên, ngày sinh và trả về từng trường kèm độ tin cậy. Hồ sơ KYC-0001 vì thế có hai kết quả với hai bộ trường khác nhau: CCCD có quê quán, GPLX có hạng bằng và ngày hết hạn. Tuần sau nhóm đổi mô hình OCR: thêm một trường, đổi tên một trường. Người duyệt hồ sơ vẫn phải mở đúng kết quả của đúng phiên bản mô hình, và hệ thống vẫn phải trả lời được câu «những kết quả nào đọc kém, dưới ngưỡng tin cậy?».

Đây là bài 26 của lộ trình **Python trung bình qua Tới Nơi**. Bài dùng MongoDB 8 trong hạ tầng dùng chung (`infra/README.md` của loạt bài), cần quen asyncio ở bài 21 và đã thấy cách PostgreSQL lưu dữ liệu ở [bài 24](/articles/python-sqlalchemy-psycopg-postgresql-mapping-migration). Dữ liệu hoàn toàn tổng hợp: số CCCD giả bắt đầu bằng `099`, GPLX giả bắt đầu bằng `99`, tên giả. Kết quả OCR và độ tin cậy chỉ là đầu vào cho người duyệt, không tự động là quyết định xác minh.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một đồng nghiệp đề xuất: «Dùng MongoDB cho kết quả OCR, vì mỗi loại giấy tờ một kiểu nên khỏi cần schema.» Nhận định này đúng đến đâu?</p>
<details><summary>Xem đáp án</summary><p><b>Chỉ đúng một nửa.</b> Hình dạng dữ liệu đúng là thay đổi theo loại giấy tờ, và đó là lý do hợp lý để chọn document. Nhưng «khỏi cần schema» thì sai: schema vẫn tồn tại, chỉ chuyển từ bảng sang code đọc dữ liệu và sang validator của collection. Không khai báo, một độ tin cậy 1,7 sẽ lọt vào dữ liệu và chỉ lộ ra lúc đọc; bài chạy đúng tình huống này với validator. Còn trường bị đổi tên giữa hai đời mô hình thì validator của lab chưa chặn (nó chỉ kiểm <code>fields</code> là object), nên được xử lý bằng <code>schema_version</code> ở cuối bài.</p></details>
</div>

## Khi nào chọn document thay vì bảng

Câu hỏi nên đặt không phải «SQL hay NoSQL», mà là **đơn vị đọc và ghi** của dữ liệu. Một kết quả OCR được ghi một lần sau khi mô hình chạy xong, đọc nguyên khối để hiện cạnh ảnh giấy tờ, hiếm khi sửa từng trường, và chỉ nối với phần còn lại của hệ thống qua mã hồ sơ `kyc_id`. Với dạng dữ liệu ấy, một document tự nhiên hơn việc xé kết quả thành nhiều dòng rồi ghép lại.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Một collection chứa hai document: kết quả CCCD và kết quả GPLX. Hai document có chung phần vỏ cố định gồm mã hồ sơ, loại giấy tờ, độ tin cậy, phiên bản mô hình, phiên bản schema và thời điểm tạo; phần fields thì khác nhau theo loại giấy tờ."><rect x="20" y="16" width="840" height="248" rx="12" class="f-bg s-line"/><text x="36" y="42" class="t-mono">lab26_ocr_documents</text><rect x="40" y="58" width="400" height="190" rx="10" class="f-surface s-line"/><text x="56" y="82" class="t-b">document_type: cccd</text><rect x="56" y="94" width="368" height="56" rx="8" class="f-phys-soft s-phys"/><text x="68" y="116" class="t-small">kyc_id, document_type, confidence,</text><text x="68" y="136" class="t-small">model_version, schema_version, created_at</text><rect x="56" y="162" width="368" height="74" rx="8" class="f-warn-soft s-warn"/><text x="68" y="184" class="t-b">fields</text><text x="68" y="204" class="t-small">id_number, full_name, date_of_birth,</text><text x="68" y="222" class="t-small">sex, hometown_city (5 trường của CCCD)</text><rect x="460" y="58" width="380" height="190" rx="10" class="f-surface s-line"/><text x="476" y="82" class="t-b">document_type: gplx</text><rect x="476" y="94" width="348" height="56" rx="8" class="f-phys-soft s-phys"/><text x="488" y="116" class="t-small">kyc_id, document_type, confidence,</text><text x="488" y="136" class="t-small">model_version, schema_version, created_at</text><rect x="476" y="162" width="348" height="74" rx="8" class="f-warn-soft s-warn"/><text x="488" y="184" class="t-b">fields</text><text x="488" y="204" class="t-small">license_number, full_name, license_class, expiry_date</text><text x="488" y="222" class="t-small">(4 trường của GPLX)</text></svg>
</div>
<figcaption>Hình 1. Hai document cùng collection. Phần vỏ cố định là thứ ta lọc, sắp xếp và đánh index; phần fields đổi theo loại giấy tờ và chỉ được đọc cùng cả document.</figcaption>
</figure>

Bảng dưới so sánh hai cách lưu cho bài toán này. Đây là so sánh cho **kết quả OCR**, không phải phán quyết chung cho mọi dữ liệu của Tới Nơi.

| Tiêu chí | Bảng PostgreSQL | Document MongoDB |
| --- | --- | --- |
| Hình dạng thay đổi theo loại giấy tờ | Một bảng thưa nhiều cột NULL, hoặc thêm bảng mỗi khi có loại mới | Mỗi document mang đúng trường của nó |
| Thêm trường khi đổi mô hình OCR | Cần migration có phối hợp giữa code và schema (thêm cột nullable thì nhẹ, đổi tên hay tách cột thì không) | Ghi document mới với `schema_version` mới; bản cũ vẫn đọc được |
| Đọc cả kết quả để hiện cạnh ảnh | Nhiều dòng hoặc nhiều bảng cần ghép | Một lần `find_one` |
| Ràng buộc nhiều dòng (số dư, chuyển trạng thái chuyến) | Mạnh, là thế mạnh của bảng | Yếu hơn; không phải việc của bài này |
| Truy vấn nối nhiều thực thể, báo cáo bất kỳ | SQL và JOIN | Không phải điểm mạnh |

Hai điều kiện đảo ngược lựa chọn. Nếu dữ liệu là tiền, số dư hay trạng thái chuyến, nơi nhiều dòng phải đổi cùng nhau hoặc không đổi gì, hãy ở lại PostgreSQL với transaction (bài 25). Và nếu đội đã vận hành PostgreSQL tốt mà chỉ cần cất một khối JSON, cột `jsonb` rẻ hơn nhiều so với vận hành thêm một hệ cơ sở dữ liệu. Tới Nơi chọn MongoDB cho document OCR theo nguyên tắc «service sở hữu dữ liệu của mình»: `kyc-ocr` có kho riêng và chỉ giữ `kyc_id` làm khóa tham chiếu.

## Kết nối bằng AsyncMongoClient, và chuyện Motor

PyMongo từ bản 4.13 (14/5/2025) có API async ổn định là `pymongo.AsyncMongoClient`. Tài liệu migration của PyMongo mô tả nó là sự hợp nhất của PyMongo và Motor, cài đặt asyncio trực tiếp thay vì đẩy việc mạng sang một thread pool như Motor. Trang chủ tài liệu Motor ghi rõ Motor sẽ deprecated từ 14/5/2026 và chỉ nhận sửa lỗi nghiêm trọng đến 14/5/2027. Kiểm lại ngày 6/11/2026, mốc đầu đã qua, nên **dự án mới dùng `AsyncMongoClient`** và lab này không cài Motor. Dự án cũ đang dùng Motor thì thường chỉ cần đổi `AsyncIOMotorClient` thành `AsyncMongoClient` và đổi câu `import` sang `pymongo`, theo hướng dẫn migration.

Vài khác biệt nên biết, cũng theo hướng dẫn đó. `AsyncMongoClient` không an toàn trên nhiều thread, chỉ dùng trong một event loop. `cursor.to_list(0)` không hợp lệ, phải gọi `to_list(None)`. `find()` là hàm thường trả về cursor để lặp bằng `async for`, còn các hàm chạm mạng như `insert_one`, `find_one`, `count_documents` là coroutine phải `await`. Lúc viết lab ta vấp đúng chỗ này: `aggregate()` cũng là coroutine trả về cursor, nên `col.aggregate(...).to_list()` báo `'coroutine' object has no attribute 'to_list'`. Nếu codebase đồng bộ và đơn giản, PyMongo đồng bộ vẫn hợp lệ; tài liệu gợi ý dùng nó cùng `run_in_executor` khi chỉ cần tương thích asyncio ([bài 21](/articles/python-asyncio-coroutine-task-timeout-cancellation) đã nói `to_thread`).

Kết nối của lab (trích `main.py`):

```python
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
```

Ba điểm. Driver kết nối lười, nên lệnh `ping` ngay đầu buộc lỗi hạ tầng lộ ra sớm. `serverSelectionTimeoutMS=2000` rút thời gian chờ chọn server từ mặc định 30 giây xuống 2 giây. `tz_aware=True` làm `datetime` đọc ra có `tzinfo`; thiếu nó, phép so với `datetime` aware của bài 04 sẽ nổ.

## Ghi và đọc: insert, find, projection, update

Document OCR của lab (mô tả lại `make_document`) gồm phần vỏ cố định và `fields` theo loại. CCCD dùng bộ tên trường chung của loạt bài OCR: `full_name`, `id_number`, `date_of_birth`, `sex`, `hometown_city`. Trên thẻ ngày sinh in `dd/mm/yyyy` và OCR trả đúng chuỗi đó; khi lưu, lab chuyển sang ISO `yyyy-mm-dd` bằng `to_iso_date`. Số CCCD giả có cấu trúc: `099` (mã tỉnh không tồn tại), một số giới tính và thế kỷ (thế kỷ 20: nam 0, nữ 1; thế kỷ 21: nam 2, nữ 3), hai số năm sinh, sáu số ngẫu nhiên; `make_id_number` sinh nó từ `sex` và năm sinh. `confidence` của cả document là **giá trị nhỏ nhất** trong `field_confidence`: một trường đọc kém là đủ để cả hồ sơ cần người xem. Collection được tạo trước kèm validator, nên mọi lần ghi đều bị kiểm hình dạng (trích):

```python
OCR_VALIDATOR = {
    "$jsonSchema": {
        "bsonType": "object",
        "required": ["kyc_id", "document_type", "fields", "confidence",
                     "model_version", "schema_version", "created_at"],
        "properties": {
            "kyc_id": {"bsonType": "string", "pattern": "^KYC-[0-9]{4}$"},
            "document_type": {"enum": ["cccd", "gplx"]},
            "confidence": {"bsonType": "double", "minimum": 0, "maximum": 1},
            "schema_version": {"bsonType": "int", "minimum": 1, "maximum": 2},
            "created_at": {"bsonType": "date"},
        },
    }
}
# trong run():
#   await col.database.create_collection(COLLECTION, validator=OCR_VALIDATOR)
```

Lab sinh 1.000 hồ sơ, mỗi hồ sơ một CCCD và một GPLX, cộng 5 document cũ từ mô hình đời trước (phần cuối bài). `insert_one` trả `inserted_id`; không có `_id` thì driver tự thêm một `ObjectId`. `insert_many` gửi cả lô trong ít vòng đi-về. Đọc hàng đợi duyệt (trích `review_queue`):

```python
async def review_queue(col, limit):
    """Hàng đợi duyệt: độ tin cậy dưới ngưỡng, thấp nhất trước. Projection chỉ lấy trường cần."""
    cursor = (
        col.find({"confidence": {"$lt": REVIEW_THRESHOLD}},
                 {"_id": 0, "kyc_id": 1, "document_type": 1, "confidence": 1})
        .sort("confidence", ASCENDING)
        .limit(limit)
    )
    return await cursor.to_list(length=limit)
```

Tham số thứ hai của `find` là **projection**: chỉ trả các trường cần, đỡ kéo cả `fields` về khi chỉ cần dựng danh sách. Cursor là async iterator; `to_list(length=…)` gom thành list khi biết trước độ dài hợp lý. Lệnh chạy ở mục «Chạy và kiểm chứng» cho thấy document của `KYC-0001` và hàng đợi duyệt (trích):

```text
== 2. Đọc: find_one, projection, sort, limit ==
{
  "kyc_id": "KYC-0001",
  "document_type": "cccd",
  "fields": {
    "id_number": "099083519476",
    "full_name": "Nguyễn Văn A",
    "date_of_birth": "1983-10-14",
    "sex": "Nam",
    "hometown_city": "Nha Trang"
  },
  "field_confidence": {
    "id_number": 0.893,
    ...
  },
  "confidence": 0.887,
  "model_version": "demo-ocr-1.0",
  "schema_version": 2,
  "created_at": "2026-11-01 02:00:30+00:00"
}

Hàng đợi duyệt (confidence < 0.8, thấp nhất trước, 3 dòng đầu):
  {'kyc_id': 'KYC-0054', 'document_type': 'gplx', 'confidence': 0.305}
  {'kyc_id': 'KYC-0828', 'document_type': 'cccd', 'confidence': 0.311}
  {'kyc_id': 'KYC-0093', 'document_type': 'cccd', 'confidence': 0.322}
Tổng số document dưới ngưỡng: 77
```

Với 2.005 document, 77 nằm dưới ngưỡng 0,8: khoảng 4%, đúng tỷ lệ dữ liệu tổng hợp cố ý tạo ra. Ngưỡng và tỷ lệ thật do nghiệp vụ và đánh giá mô hình quyết định.

Hai ràng buộc làm cho dữ liệu đáng tin hơn. Thứ nhất, một **unique index** trên `(kyc_id, document_type, model_version)`: cùng một hồ sơ, cùng loại giấy tờ, cùng phiên bản mô hình chỉ có một kết quả. Chèn trùng bị từ chối bằng `DuplicateKeyError` (mã lỗi 11000), tương đương lỗi SQLSTATE 23505 ở bài 25. Chạy lại OCR bằng mô hình mới là **document mới**, không ghi đè kết quả cũ, nên lịch sử còn nguyên để so sánh. Thứ hai, validator từ chối `confidence = 1.7` bằng `WriteError` mã 121.

```python
# (trích)
duplicate = {k: v for k, v in original.items() if k != "_id"}
try:
    await col.insert_one(duplicate)
except DuplicateKeyError as exc:
    code = exc.code            # 11000
...
bad = {**duplicate, "model_version": "demo-ocr-bad", "confidence": 1.7}
try:
    await col.insert_one(bad)
except WriteError as exc:
    bad_code = exc.code        # 121
```

```text
== 3. Ràng buộc: unique index và validator ==
chèn trùng (kyc_id, document_type, model_version) -> DuplicateKeyError, code 11000
chèn confidence = 1.7 -> WriteError, code 121 (DocumentValidationFailure)
bản mới nhất của KYC-0001 cccd: demo-ocr-1.1
```

Lấy «bản mới nhất» dùng `find_one({...}, sort=[("created_at", DESCENDING)])`; mục index dưới đây cho thấy kế hoạch của nó.

## Index: tin vào explain, không tin cảm giác

Truy vấn hàng đợi duyệt lọc theo `confidence`. Không có index, MongoDB phải đọc mọi document rồi mới biết cái nào dưới ngưỡng. `explain()` cho biết kế hoạch thực sự được chọn; lab rút ra giai đoạn lá của kế hoạch và số document đã phải kiểm (trích):

```python
def plan_summary(explain):
    """(giai đoạn lá của kế hoạch thắng, số document phải kiểm, số document trả về)."""
    plan = explain["queryPlanner"]["winningPlan"]
    plan = plan.get("queryPlan", plan)
    while "inputStage" in plan:
        plan = plan["inputStage"]
    stats = explain["executionStats"]
    return plan["stage"], stats["totalDocsExamined"], stats["nReturned"]

before = plan_summary(await col.find(query).explain())
await col.create_index([("confidence", ASCENDING)], name="ix_confidence")
after = plan_summary(await col.find(query).explain())
```

```text
== 4. Index và explain ==
không có index confidence: COLLSCAN, kiểm 2006 document, trả 77
có index confidence:       IXSCAN, kiểm 77 document, trả 77
tìm theo kyc_id + document_type: IXSCAN, kiểm 1 document, trả 1
bản mới nhất, chưa có ix_latest: SORT -> FETCH -> IXSCAN
bản mới nhất, có ix_latest:      LIMIT -> FETCH -> IXSCAN
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Cùng truy vấn độ tin cậy dưới ngưỡng và cùng trả về 77 document: khi quét cả collection phải kiểm 2.006 document, khi có index chỉ kiểm 77 document."><text x="20" y="52" class="t-b">Không có index</text><rect x="230" y="30" width="620" height="40" rx="8" class="f-warn-soft s-warn"/><text x="246" y="55" class="t-small">COLLSCAN: kiểm 2.006 document</text><text x="20" y="122" class="t-b">Có index</text><rect x="230" y="100" width="26" height="40" rx="6" class="f-phys-soft s-phys"/><text x="270" y="125" class="t-small">IXSCAN: kiểm 77 document</text><text x="230" y="178" class="t-small">Cả hai cùng trả về 77 document; độ dài thanh tỷ lệ với số document phải kiểm.</text></svg>
</div>
<figcaption>Hình 2. Khối lượng việc của server khi lọc theo độ tin cậy, trước và sau khi có index.</figcaption>
</figure>

Số 2.006 là 2.005 document ban đầu cộng một bản kết quả mô hình mới chèn ở mục trên. Dòng thứ ba cho thấy truy vấn theo `kyc_id` và `document_type` đã được phục vụ bởi chính unique index ở trên: index ghép dùng được cho truy vấn khớp **tiền tố** của nó (`kyc_id`, rồi `document_type`), nên ta không cần thêm index riêng. Quy tắc tương tự B-tree trong PostgreSQL. Hai dòng cuối là truy vấn «bản mới nhất»: chỉ với unique index, MongoDB tìm đúng các bản của hồ sơ rồi phải `SORT` theo `created_at`; sau khi thêm `ix_latest` (ghép `kyc_id`, `document_type`, `created_at` giảm dần), thứ tự đã nằm sẵn trong index nên `SORT` biến mất và `LIMIT` dừng khi đủ một document.

Cái giá của index là mỗi lần ghi phải cập nhật thêm, và index chiếm bộ nhớ. Lab chỉ có 2.005 document nên ta đo **khối lượng việc** (số document kiểm), không đo thời gian. Ở đây tỉ lệ chọn lọc là 77 trên 2.006 (khoảng 4%); khi collection lớn lên, khối lượng phải kiểm của truy vấn không có index lớn lên theo cả collection, còn truy vấn có index chỉ lớn lên theo số document khớp.

## Thời gian và phiên bản schema

**Thời gian.** BSON lưu thời điểm với độ chính xác **mili giây**. Python có micro giây, nên phần lẻ hơn mili giây bị cắt khi ghi:

```text
== 5. Thời gian: UTC aware và độ chính xác mili giây ==
gửi vào : 2026-11-03T02:00:00.123456+00:00
đọc ra  : 2026-11-03T02:00:00.123000+00:00 (MongoDB giữ mili giây, bỏ phần micro giây)
giờ Việt Nam: 2026-11-03T09:00:00.123000+07:00
```

Hệ quả: đừng so `created_at` đọc từ DB bằng `==` với giá trị Python còn micro giây, và đừng dựa vào micro giây để phân biệt hai sự kiện. Cách lưu vẫn như bài 04: UTC aware, đổi sang `Asia/Ho_Chi_Minh` khi hiển thị, như dòng cuối.

**Phiên bản schema.** Mô hình OCR đời trước (`demo-ocr-0.9`) ghi `fields.name` và không có độ tin cậy từng trường; đời mới ghi `fields.full_name` kèm `field_confidence`. Collection đang có lẫn hai hình dạng, đánh dấu bằng `schema_version`. Có hai cách xử lý, và thường dùng cả hai. **Upgrade on read**: hàm thuần chuẩn hóa mọi document về hình dạng mới ngay khi đọc, ứng dụng chỉ làm việc với một hình dạng. **Migrate**: một lần `update_many` với aggregation pipeline đổi tên trường và đặt `field_confidence` thành `null` ngay trong server, không phải kéo dữ liệu về Python.

```python
# (trích)
def normalize_to_v2(doc):
    """Upgrade on read: ứng dụng luôn làm việc với hình dạng v2, DB có thể còn lẫn v1."""
    if doc["schema_version"] >= 2:
        return doc
    fields = dict(doc["fields"])
    fields["full_name"] = fields.pop("name")
    # v1 chưa từng có độ tin cậy từng trường: trả None, không bịa số.
    return {**doc, "fields": fields, "field_confidence": None, "schema_version": 2}

result = await col.update_many(
    {"schema_version": 1},
    [{"$set": {"fields.full_name": "$fields.name", "field_confidence": None, "schema_version": 2}},
     {"$unset": "fields.name"}],
)
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Document phiên bản 1 trong database được hàm normalize_to_v2 chuyển thành hình dạng phiên bản 2 trong bộ nhớ khi đọc; update_many chạy một lần để chuyển hẳn dữ liệu trong database sang phiên bản 2."><defs><marker id="py26-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="250" height="90" rx="10" class="f-warn-soft s-warn"/><text x="36" y="72" class="t-b">DB: schema v1</text><text x="36" y="98" class="t-small">fields.name</text><text x="36" y="116" class="t-small">không có field_confidence</text><path d="M270 85 L330 85" class="f-none s-phys" stroke-width="2" marker-end="url(#py26-0-arrow)"/><text x="278" y="72" class="t-small">đọc</text><rect x="330" y="40" width="270" height="90" rx="10" class="f-pk-soft s-pk"/><text x="346" y="72" class="t-b">Ứng dụng: luôn v2</text><text x="346" y="98" class="t-small">normalize_to_v2()</text><text x="346" y="116" class="t-small">fields.full_name</text><rect x="20" y="160" width="250" height="60" rx="10" class="f-phys-soft s-phys"/><text x="36" y="196" class="t-b">DB: schema v2</text><path d="M145 130 L145 160" class="f-none s-phys" stroke-width="2" marker-end="url(#py26-0-arrow)"/><text x="160" y="150" class="t-small">update_many, một lần</text><text x="330" y="196" class="t-small">Sau migrate, bước chuẩn hóa còn là lưới an toàn cho document lạc hậu.</text></svg>
</div>
<figcaption>Hình 3. Hai đường đi của schema_version: chuẩn hóa tại lúc đọc, và chuyển hẳn dữ liệu trong database.</figcaption>
</figure>

```text
== 6. schema_version: upgrade on read rồi migrate ==
v1 trong DB    : {'id_number': '099090679108', 'name': 'Hoàng Văn E', 'date_of_birth': '1990-01-01'}
v1 đọc thành v2: {'id_number': '099090679108', 'date_of_birth': '1990-01-01', 'full_name': 'Hoàng Văn E'}
update_many (pipeline) -> sửa 5 document
Sau migrate, theo schema_version: {2: 2007}
document v2 thiếu khóa field_confidence: 0
field_confidence của document đã migrate: None
```

Sau migrate các document cũ có `field_confidence` bằng `null`, vì mô hình cũ chưa từng tính nó: dữ liệu không tự sinh ra từ hư không. Đặt khóa rõ ràng là chủ ý: nếu pipeline chỉ đổi `schema_version` mà quên khóa này, document tự nhận là v2 nhưng `doc["field_confidence"]` sẽ `KeyError`, và `normalize_to_v2` (chỉ chuẩn hóa v1) không cứu được. Lab kiểm không còn document nào thiếu khóa (dòng thứ ba của output trên). Ứng dụng phải chấp nhận `None` và không được suy diễn rằng thiếu nghĩa là tin cậy cao. Nếu muốn chắc, cho OCR lại các document này bằng mô hình mới; vì unique index có `model_version`, kết quả mới là document mới.

## Chạy và kiểm chứng

Cần hạ tầng dùng chung đang chạy (`docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait`). Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/26-mongodb-pymongo-document-ocr examples/toi-noi-python/26-mongodb-pymongo-document-ocr/main.py
uv run --project examples/toi-noi-python/26-mongodb-pymongo-document-ocr examples/toi-noi-python/26-mongodb-pymongo-document-ocr/main.py --verify
```

Lệnh đầu in toàn bộ các khối output ở trên (trích từng phần). Lệnh `--verify` chạy lại cùng kịch bản, so 37 kết quả kỳ vọng và dọn collection `lab26_ocr_documents` khi xong:

```text
OK 37/37
```

Khi chưa bật MongoDB, `--verify` in `FAIL: không nối được MongoDB tại mongodb://localhost:27019/toinoi (docker compose …)` và thoát mã 1. Môi trường chạy: Python 3.14.7, PyMongo 4.19.0, MongoDB 8.3.13 trong container `mongo:8`.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `'coroutine' object has no attribute 'to_list'` | `aggregate()` của `AsyncCollection` là coroutine: `cursor = await col.aggregate(...)` rồi mới `await cursor.to_list(...)` |
| `ServerSelectionTimeoutError` | Container Mongo đã `healthy` chưa, cổng 27019 có đúng không, `TOINOI_MONGO_URL` có bị đặt sai không |
| `can't compare offset-naive and offset-aware datetimes` | Client chưa đặt `tz_aware=True`, hoặc đang so datetime đọc từ DB với `datetime.now()` không múi giờ |
| `WriteError` mã 121 | Document sai hình dạng so với validator; xem `exc.details["errInfo"]` để biết thuộc tính nào |
| `explain()` hiện `COLLSCAN` | Chưa có index, hoặc index ghép không khớp tiền tố của bộ lọc |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Một server MongoDB đơn lẻ, không replica set, nên ví dụ không nói gì về độ bền khi ghi (write concern), failover hay transaction nhiều document. 2.005 document quá nhỏ để chứng minh hiệu năng. Dữ liệu OCR thật chứa thông tin cá nhân: cần che số giấy tờ, giới hạn quyền đọc, đặt thời hạn lưu và ghi nhật ký truy cập, những việc thuộc [bài 47](/articles/python-luu-ket-qua-ocr-mongodb-pii-retention). Độ tin cậy trong lab là số tổng hợp, không đại diện cho mô hình OCR thật nào.</p>
</div>

**Bài tập:** (1) Trong `ix_latest`, vì sao `created_at` đứng cuối và giảm dần, thay vì đứng đầu? Nếu đảo thứ tự thì truy vấn «bản mới nhất của một hồ sơ» còn dùng index tốt không? (2) Mô hình OCR kế tiếp tách `full_name` thành `last_name` và `first_name` (schema_version 3). Hãy phác các bước đưa thay đổi này vào mà không dừng hệ thống.

<details><summary>Xem lời giải</summary><p>(1) Quy tắc chung: các trường lọc bằng đứng trước, trường sắp xếp đứng sau. Với <code>(kyc_id, document_type, created_at giảm dần)</code>, hai trường đầu chọn đúng một khoảng liền trong index và trong khoảng đó các bản đã xếp từ mới tới cũ, nên đọc một phần tử là đủ (<code>LIMIT → FETCH → IXSCAN</code> trong output). Nếu đặt <code>created_at</code> đứng đầu, index xếp mọi document của mọi hồ sơ theo thời gian; muốn tìm hồ sơ <code>KYC-0001</code> MongoDB phải duyệt theo thời gian và lọc từng khóa, nên không còn dùng được cho việc chọn hồ sơ. Với vài phiên bản mỗi hồ sơ, lợi ích nhỏ; mỗi index thêm là một khoản chi phí ghi, nên đo trước khi thêm. (2) Nâng <code>maximum</code> của <code>schema_version</code> trong validator lên 3 (dùng lệnh <code>collMod</code>); triển khai code <b>đọc được cả v2 và v3</b> (<code>normalize_to_v3</code>) trước; rồi mới triển khai code ghi v3; cuối cùng chạy <code>update_many</code> với pipeline (hoặc theo lô nếu cần kiểm từng trường hợp). Thứ tự «đọc được trước, ghi sau, migrate cuối» cho bản cũ và bản mới của service sống chung lúc triển khai.</p></details>

Bài trước: [bài 25](/articles/python-transaction-optimistic-concurrency-idempotency). Bài tiếp: [bài 27](/articles/python-redis-cache-rate-limit-geo).

## Tài liệu gốc

- [PyMongo: AsyncMongoClient](https://pymongo.readthedocs.io/en/stable/api/pymongo/asynchronous/mongo_client.html).
- [PyMongo: AsyncCollection](https://pymongo.readthedocs.io/en/stable/api/pymongo/asynchronous/collection.html).
- [PyMongo Driver: Migrate to PyMongo Async](https://www.mongodb.com/docs/languages/python/pymongo-driver/current/reference/migration/).
- [Motor: thông báo deprecation](https://motor.readthedocs.io/en/stable/).
- [MongoDB: Schema Validation](https://www.mongodb.com/docs/manual/core/schema-validation/).
- [MongoDB: Analyze Query Plan](https://www.mongodb.com/docs/manual/tutorial/analyze-query-plan/).
