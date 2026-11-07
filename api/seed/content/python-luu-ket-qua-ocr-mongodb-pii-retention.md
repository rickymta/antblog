---
slug: python-luu-ket-qua-ocr-mongodb-pii-retention
title: "Lưu kết quả OCR: document MongoDB, mặt nạ PII, versioning và retention"
summary: "Bài 47: lưu kết quả OCR vào MongoDB có chính sách PII. Che số giấy tờ, băm HMAC có khóa bí mật để đối chiếu, TTL index cho bản gốc, audit ai xem, migration schema v2 (bài 26) sang v3 đọc được cả hai."
category: python-advanced
level: advanced
tags: [python, toi-noi, mongodb, pii, retention]
language: vi
---

Service OCR của bài 46 giữ kết quả trong Redis 15 phút. Người duyệt hồ sơ của [Tới Nơi](/articles/du-an-mau-toi-noi) cần xem nó hôm sau, tuần sau, cạnh các lần OCR bằng mô hình mới hơn, nên nó phải sang một kho bền. Nhưng thứ cần lưu là số CCCD, họ tên, ngày sinh của người thật, và «lưu» kéo theo bốn câu hỏi: lưu **trường nào** và ở dạng nào, **bao lâu**, **ai** được xem, và **dữ liệu cũ** ra sao khi đổi cách lưu.

Đây là bài 47 của lộ trình **Python nâng cao qua Tới Nơi**, bài cuối của phần OCR giấy tờ. Bài nối tiếp [bài 26](/articles/python-mongodb-pymongo-motor-document-ocr) (document OCR, `schema_version`, bộ trường `full_name`, `id_number`...) và nhận đầu vào là kết quả job của [bài 46](/articles/python-ocr-service-fastapi-worker-timeout). Dữ liệu hoàn toàn tổng hợp. Kết quả OCR là đầu vào cho người duyệt, không tự động là quyết định xác minh.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Nhóm bảo mật đề xuất: «Đừng lưu số CCCD, lưu <code>sha256(số)</code>. Hash không đảo ngược được, nên lộ dữ liệu cũng không sao.» Số CCCD gồm 12 chữ số. Đề xuất này an toàn đến đâu?</p>
<details><summary>Xem đáp án</summary><p><b>Gần như không an toàn.</b> Kẻ tấn công không cần đảo hash: chỉ cần thử mọi số rồi băm so sánh. Số CCCD có cấu trúc (mã tỉnh, một số giới tính và thế kỷ, hai số năm sinh), và giới tính cùng ngày sinh thường nằm rõ trong chính document, nên chỉ còn 6 chữ số ngẫu nhiên, tức một triệu khả năng. Bài chạy đúng cuộc tấn công này: dò ra số thật chỉ sau vài giây, kể cả trên một máy rất bận. Muốn hash có ích thì phải có khóa bí mật, đặt tách khỏi cơ sở dữ liệu.</p></details>
</div>

## Ba loại dữ liệu, ba vòng đời

Lab chia kết quả OCR theo **vòng đời**: phần ai cũng cần và sống lâu tách khỏi phần nhạy cảm, hết hạn sớm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Kết quả job OCR của bài 46 đi qua save_ocr_result. Bước một ghi lab47_ocr_raw gồm số giấy tờ rõ, văn bản thô và khóa ảnh, có TTL 30 ngày. Bước hai ghi lab47_ocr_documents gồm số đã che, hash HMAC, ngày sinh ISO, độ tin cậy và con trỏ raw_id, không có TTL. view_document ghi lab47_audit_log, có TTL 400 ngày, trước khi trả dữ liệu cho người xem."><defs><marker id="py47-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="115" width="170" height="90" rx="10" class="f-pk-soft s-pk"/><text x="34" y="142" class="t-b">Kết quả job OCR</text><text x="34" y="164" class="t-small">fields, confidence,</text><text x="34" y="182" class="t-small">raw_text, model_version</text><path d="M190 140 L248 70" class="f-none s-phys" stroke-width="2" marker-end="url(#py47-0-arrow)"/><path d="M190 160 L248 160" class="f-none s-phys" stroke-width="2" marker-end="url(#py47-0-arrow)"/><rect x="250" y="20" width="300" height="100" rx="10" class="f-warn-soft s-warn"/><text x="264" y="46" class="t-b">lab47_ocr_raw (ghi trước)</text><text x="264" y="68" class="t-small">số giấy tờ rõ, văn bản thô, khóa ảnh</text><text x="264" y="86" class="t-small">TTL: expire_at = created_at + 30 ngày</text><text x="264" y="104" class="t-small">hết hạn thì tự biến mất</text><rect x="250" y="125" width="300" height="100" rx="10" class="f-phys-soft s-phys"/><text x="264" y="151" class="t-b">lab47_ocr_documents (ghi sau)</text><text x="264" y="173" class="t-small">số đã che + hash HMAC, ngày sinh ISO,</text><text x="264" y="191" class="t-small">confidence, model_version, raw_id</text><text x="264" y="209" class="t-small">không TTL: sống tới khi có yêu cầu xóa</text><rect x="250" y="230" width="300" height="90" rx="10" class="f-pk-soft s-pk"/><text x="264" y="256" class="t-b">lab47_audit_log</text><text x="264" y="278" class="t-small">ai xem gì, lúc nào, lý do; không chứa số</text><text x="264" y="296" class="t-small">TTL: expire_at = at + 400 ngày</text><path d="M550 175 L610 175" class="f-none s-phys" stroke-width="2" marker-end="url(#py47-0-arrow)"/><path d="M550 70 L614 128" class="f-none s-phys dash" stroke-width="2" marker-end="url(#py47-0-arrow)"/><rect x="612" y="115" width="250" height="120" rx="10" class="f-pk-soft s-pk"/><text x="626" y="142" class="t-b">view_document</text><text x="626" y="164" class="t-small">mặc định: chỉ dạng che</text><text x="626" y="182" class="t-small">reveal: cần scope kyc:reveal</text><text x="626" y="200" class="t-small">và lý do; đọc raw nếu còn</text><text x="626" y="218" class="t-small">ghi audit TRƯỚC khi trả</text><path d="M740 235 L560 285" class="f-none s-phys dash" stroke-width="2" marker-end="url(#py47-0-arrow)"/></svg>
</div>
<figcaption>Hình 1. Ba collection, ba vòng đời. Số giấy tờ rõ chỉ nằm ở collection có hạn dùng ngắn; collection sống lâu chỉ có dạng che và dạng băm.</figcaption>
</figure>

Một document trong `lab47_ocr_documents` sau khi lưu kết quả thật của bài 46 (thẻ giả seed 7), trích từ output của `main.py`:

```json
{
  "kyc_id": "KYC-0007",
  "document_type": "cccd",
  "model_version": "tesseract-5.5.4+vie+eng/psm6",
  "image_sha256": "dec89321f6b5dcdba573fe24133ca6de1870045570ca746a870f2038e8d7edd9",
  "schema_version": 3,
  "pii_key_id": "lab-1",
  "fields": {
    "date_of_birth": "1990-03-13",
    "full_name": "NGUYEN VAN A",
    "sex": "Nam",
    "hometown_city": "Cần Thơ",
    "id_number_masked": "099*******83",
    "id_number_hash": "81b1f40d9f35d5dbdf25e373ee29548d3dd0ffb306e507ac3e354f6d2388aef2"
  },
  "field_confidence": {
    "id_number": 0.965,
    "date_of_birth": 0.966,
    "full_name": 0.952,
    "sex": 0.964,
    "hometown_city": 0.966
  },
  "confidence": 0.952,
  "needs_review": false,
  "missing_fields": [],
  "raw_id": "6aede4b855d30ccc5b495ec5",
  "created_at": "2026-11-06 14:31:20+00:00"
}
```

So với bài 26: ngày sinh `13/03/1990` trên thẻ được đổi sang ISO, `confidence` là nhỏ nhất của các trường, và có thêm `image_sha256`, `missing_fields`. Mới là `id_number_masked`, `id_number_hash`, `pii_key_id`, `raw_id`: số CCCD rõ **không còn** trong document. Số rõ, văn bản thô và khóa ảnh nằm ở document tương ứng của `lab47_ocr_raw`, có `expire_at` đúng 30 ngày sau `created_at`. Họ tên và ngày sinh vẫn rõ vì người duyệt cần chúng và chúng ít nhạy cảm hơn số giấy tờ (vẫn là dữ liệu cá nhân); đó là quyết định cần pháp chế ký, và bài này không phải tư vấn pháp lý.

Về số phiên bản, bài nối tiếp dãy của bài 26: **v2** là document còn `fields.id_number` rõ (hình dạng sau migration của bài 26), **v3** là dạng đã che. Document v1 của bài 26 phải qua migration của bài 26 trước; `normalize_to_v3` gặp v1 ném `ValueError` thay vì đoán.

## Che số và băm có khóa

Che số giữ ba số đầu và hai số cuối: `099078777631` thành `099*******31`. Bản đầu của lab giữ 4 đầu 4 cuối; review chỉ ra rằng khi kẻ tấn công biết thêm ngày sinh và giới tính (đang lưu rõ) thì chỉ còn 100 ứng viên. Với mặt nạ hiện tại lab đếm được **10.000** ứng viên: vẫn lộ một phần, nên mặt nạ chỉ để người duyệt nhận ra hồ sơ, không phải biện pháp bảo vệ. `mask_digits` từ chối chuỗi ngắn, chuỗi lẫn chữ và chữ số không phải ASCII (lab thử cả chữ số Arabic-Indic). Băm để đối chiếu dùng HMAC-SHA-256 (`app/pii.py`):

```python
def pii_hash(value: str, document_type: str, key: PiiKey) -> str:
    return hmac.new(key.key, f"{document_type}\x1f{value}".encode(), hashlib.sha256).hexdigest()
```

Khóa HMAC này là **khóa bí mật chung của hệ thống** (hay gọi là pepper), không phải salt: salt công khai và khác nhau mỗi bản ghi, còn ở đây cùng số phải ra cùng hash thì mới tra index được, nên khóa phải chung và phải giữ kín. Vì vậy mã dùng tên `PiiKey`, `pii_hash_key`, biến môi trường `TOINOI_PII_HASH_KEY`, tối thiểu 32 ký tự (production: 32 byte ngẫu nhiên). Hash kèm loại giấy tờ để CCCD và GPLX trùng chữ số không trùng hash; `pii_key_id` ghi trong document để xoay khóa. `PiiSettings` không có giá trị mặc định cho khóa, kiểu `SecretStr` với `hide_input_in_errors`: thiếu hoặc ngắn thì `ValidationError` mà không in lại giá trị; `PiiKey` có `__repr__` che khóa. Khóa trong `main.py` là **giá trị của lab**, công khai trong mã nguồn; production lấy từ secret manager (Vault, AWS Secrets Manager, Azure Key Vault...) và đặt tách khỏi cơ sở dữ liệu. Thí nghiệm sau cho thấy vì sao «tách» là cốt lõi:

```text
SHA-256 trơn, biết 6 số đầu: dò lại được 099090159083 sau 2.49 giây (tối đa 10^6 lần thử)
cùng bộ dò nhưng giá trị là HMAC có khóa mà kẻ tấn công không có: None sau 13.23 giây
nếu lộ cả khóa lẫn dữ liệu: dò ra 099090159083 sau 7.34 giây
```

Dòng đầu là đáp án của câu hỏi mở bài (thời gian tính tới lúc gặp số đúng, tối đa 10^6 lần thử). Dòng hai: không có khóa thì thử hết một triệu khả năng mà không ra gì. Dòng ba: **lộ cả khóa lẫn dữ liệu thì HMAC không cứu được**, nên khóa không được nằm cạnh dữ liệu: cùng bản sao lưu, cùng log, cùng repository.

## Lưu idempotent, lưu từng phiên bản mô hình và từng ảnh

Khóa của một kết quả là `(kyc_id, document_type, model_version, image_sha256)`, và `raw` dùng cùng khóa. Bản đầu thiếu `image_sha256`: ảnh chụp lại với cùng mô hình đụng khóa cũ và bị coi là «đã có», tức mất kết quả mới. Giờ nó là document mới, bản cũ còn, «bản mới nhất» là bản có `created_at` lớn nhất (đồng hạng theo `_id`). `save_ocr_result` tìm document theo khóa và trả ngay nếu có, **không đụng tới `raw`** (nếu `raw` đã hết hạn thì lưu lại không dựng `raw` mới trong khi `raw_id` vẫn trỏ cái cũ); rồi biến đổi dữ liệu (che, băm, đổi ngày), ghi `raw`, ghi document. Biến đổi chạy trước mọi lần ghi để đầu vào xấu không để lại `raw` mồ côi; chết giữa hai lần ghi thì `raw` mồ côi tự hết hạn nhờ TTL. Chạy lại an toàn nhờ unique index:

```python
try:
    await db[DOCS].insert_one(doc)
    return SaveResult(doc, True)
except DuplicateKeyError:
    existing = await db[DOCS].find_one(id_key)
    return SaveResult(existing, False)   # đã có đúng kết quả này: trả lại bản cũ
```

OCR không đọc ra số, số chỉ 11 chữ số, hay ngày `31/02/1990` không có thật đều là việc của người duyệt: lab **lưu lại** document với `needs_review: true`, trường đó `null` và tên nó nằm trong `missing_fields`, thay vì từ chối và mất kết quả.

Hình dạng document được canh ở **chính collection** bằng `$jsonSchema`. Trong thời gian chuyển đổi validator nhận cả v2 và v3 (`oneOf`); v3 đóng kín ở cả hai cấp, chỉ nhận trường đã khai, hash đúng mẫu 64 chữ số hex (`app/documents.py`, trích):

```python
"fields": {
    "bsonType": "object",
    "additionalProperties": False,           # không nhận id_number rõ, so_cccd hay chữ lạ
    "properties": {"id_number_masked": {"bsonType": ["string", "null"], "pattern": "^[0-9]{3}[*]{7}[0-9]{2}$"},
                   "id_number_hash": {"bsonType": ["string", "null"], "pattern": "^[0-9a-f]{64}$"}, ...},
    "anyOf": [{"required": ["id_number_masked", "id_number_hash"]},
              {"required": ["license_number_masked", "license_number_hash"]}],
}
```

Lab thử sáu document sai và cả sáu bị từ chối với mã `121`: số rõ trong `fields`, trường lạ trong `fields`, trường lạ ngoài `fields` (`raw_text`), hash rỗng, mặt nạ sai dạng, thiếu `pii_key_id`.

Đối chiếu theo số giấy tờ đi qua hash và **cũng bị ghi audit**: `lookup_by_number` cần scope `kyc:view`, ghi hành động `lookup` (không ghi số được tra), rồi băm số bằng từng khóa còn hiệu lực và tra `ix_id_hash`; kết quả chỉ có dạng che. Lab cho `KYC-0007` và `KYC-2001` cùng số: tra ra bốn document (hai hồ sơ, vì `KYC-0007` có ba), `explain` báo `IXSCAN` trên `ix_id_hash`, kiểm 4 document, trả 4. Xoay khóa có cái giá: tra bằng khóa mới một mình không thấy bản ghi cũ, nên phải **giữ khóa cũ trong danh sách tra** tới khi bản ghi cũ hết vòng đời; băm lại chỉ làm được cho bản còn `raw`, vì khi `raw` đã hết hạn thì số rõ không còn ở đâu.

## Retention bằng TTL index

**TTL index** là index trên một trường kiểu date; MongoDB xóa document khi thời điểm đó đã qua. Lab đặt `expire_at` riêng cho từng document và `expireAfterSeconds=0`, tức «xóa đúng lúc `expire_at`». Cách này cho mỗi document một hạn riêng và cho phép **giữ lại** một document (vì lý do pháp lý) bằng cách gỡ trường `expire_at`:

```python
await raw.create_index("expire_at", expireAfterSeconds=0, name="ttl_raw_expire_at")
```

Ba điều thường làm người ta thất vọng, và lab thử cả ba. Một: bộ quét TTL của `mongod` chạy theo chu kỳ (mặc định 60 giây), nên «hết hạn lúc 10:00» không có nghĩa là «biến mất lúc 10:00». Để quan sát, lab tạm đặt `ttlMonitorSleepSecs` về 1 giây, và tham số này áp cho **cả `mongod`**: mọi lab khác dùng chung container cũng bị quét nhanh trong lúc đó. Nó không nằm trong trang tham số chính thức của MongoDB; lab dùng vì `setParameter` đổi được lúc chạy (đã thử trên 8.3.13). Lab trả lại bằng `finally` và bắt SIGTERM (gửi SIGTERM giữa chừng: tham số về 60, collection được xóa). Bị SIGKILL thì tham số kẹt ở 1; lần chạy sau thấy giá trị không lớn hơn 1, cảnh báo và trả về 60. Hai: trường phải là **kiểu date**; `expire_at` là chuỗi ISO không bao giờ hết hạn, không có cảnh báo nào. Ba: TTL chỉ xóa document trong collection đó, không biết gì về ảnh trong kho. Năm document thử:

| Document | `expire_at` | Kết quả |
| --- | --- | --- |
| A | đã qua 5 giây, kiểu date | bị xóa |
| B | 1 ngày sau | còn |
| C | 3 giây sau, nhưng bị gỡ trường `expire_at` ngay | còn (giữ lại) |
| D | chuỗi ISO của hôm qua | còn (sai kiểu, không bao giờ hết hạn) |
| E | 3 giây sau, không gỡ gì | bị xóa (đối chứng cho C) |

E chứng minh C còn lại là do ta gỡ `expire_at` chứ không phải bộ quét chưa chạy. Về ảnh: `raw` chỉ giữ khóa đối tượng, nên khi TTL xóa document, **ảnh trong kho vẫn còn** và thành ảnh mồ côi; retention thật phải đặt luật vòng đời ở chính kho (S3, Azure Blob lifecycle) với cùng thời hạn. Bản sao lưu cũng giữ dữ liệu cũ tới khi nó hết hạn.

## Audit: ai xem gì

Mọi lần đọc đi qua `view_document` hoặc `lookup_by_number`: kiểm quyền, **ghi audit**, rồi mới dựng kết quả. Nếu kho audit hỏng, hàm ném ngoại lệ và không byte dữ liệu nào được trả: đóng cửa khi audit hỏng, vì mở cửa là cách dễ nhất để mất dấu vết đúng lúc cần (`app/access.py`, trích):

```python
if reveal and ("kyc:reveal" not in actor.scopes or not reason or len(reason.strip()) < REVEAL_MIN_REASON):
    await audit.record(actor=actor, action="reveal_denied", ...)
    raise RevealDenied("cần scope kyc:reveal và lý do tối thiểu ...")

await audit.record(actor=actor, action="reveal" if reveal else "view", ...)   # TRƯỚC khi trả
document = normalize_to_v3(latest, key)
if not reveal:
    return View(document, None, False)
if latest["schema_version"] < 3:      # v2 chưa migrate: số rõ còn ngay trong document
    return View(document, {name: latest["fields"][name]}, False)
raw = await db[RAW].find_one({"_id": document["raw_id"]})
```

Nếu `raw` không còn, kết quả báo `raw_expired` thay vì lỗi; nếu OCR vốn không đọc ra số thì không có gì để trả; document v2 chưa migrate có số rõ ngay trong nó nên reveal trả được. Nhật ký ghi `actor` (`DP-001` điều phối viên; `DP-002` chỉ được xem; `DP-003` không có scope), `action`, `kyc_id`, loại giấy tờ, lý do và thời gian đến mili giây. Cả những lần **bị từ chối** cũng được ghi, vì chúng là tín hiệu quan trọng nhất. Audit không chứa số giấy tờ, kể cả dạng che (lab quét toàn bộ nhật ký). `reason` là chữ tự do nên người gọi có thể gõ cả số vào đó; lab chỉ kiểm độ dài, production cần che mẫu số trong lý do. Audit sống lâu hơn dữ liệu nó mô tả (400 ngày so với 30; giá trị lab), và production phải để nó **chỉ ghi thêm** (tài khoản ứng dụng chỉ có quyền `insert`): nếu dịch vụ ghi audit cũng sửa được audit thì nó chỉ là log.

## Migration schema v2 sang v3

Ta có sáu document v2 của bài 26 mang số rõ: ba cái cũ hơn 30 ngày, ba cái mới. Cần cả hai cách của bài 26: **upgrade on read** (`normalize_to_v3` tính số che và hash từ số rõ của v2 mà không ghi lại, nên tầng trên chỉ thấy dạng che trong lúc chưa migrate) và migrate hàng loạt. Cách sau không viết được bằng aggregation pipeline như bài 26 vì phải tính HMAC với khóa bí mật, nên chạy trong Python theo lô. Với mỗi document, ghi `raw` trước rồi mới sửa document chính; `raw` có `expire_at = created_at + 30 ngày`, **không phải** `now + 30 ngày`, để migration không kéo dài thời hạn lưu của số rõ cũ (`app/documents.py`, trích):

```python
result = await docs.update_one(
    {"_id": doc["_id"], "schema_version": 2},       # bản chạy sau thấy 0 khớp, không sửa lần hai
    {"$set": {"fields": fields, "schema_version": SCHEMA_VERSION, "pii_key_id": key.key_id,
              "image_sha256": None, "raw_id": raw["_id"], "missing_fields": missing,
              "needs_review": doc["confidence"] < REVIEW_THRESHOLD or bool(missing)}})
modified += result.modified_count
```

Ảnh gốc của v2 không còn biết nên `image_sha256` là `null` (validator v3 cho phép đúng trường hợp này). Bộ lọc `schema_version: 2` làm migration chạy lại và chạy đồng thời được: lab chạy hai tác vụ cùng lúc trong **một** tiến trình, lần này chúng báo sửa 6 và 0 document (cách chia có thể khác giữa các lần, tổng luôn đúng sáu). Sau đó không còn document v2, không còn trường `id_number`, và quét cả collection không thấy sáu số rõ. Câu đó chỉ nói về collection chính: số rõ vẫn nằm trong `raw` tới hạn và trong bản sao lưu cũ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Migration một document v2. Bước một: ghi raw bằng setOnInsert với expire_at bằng created_at cộng 30 ngày. Bước hai: sửa document chính với bộ lọc schema_version bằng 2, đổi thành v3 gồm số đã che, hash, raw_id. Document cũ hơn 30 ngày sinh ra raw đã quá hạn nên TTL dọn ngay sau đó; document mới thì raw còn hạn. Trong lúc chưa migrate, upgrade on read trả dạng v3 mà không ghi gì."><defs><marker id="py47-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="95" width="190" height="100" rx="10" class="f-warn-soft s-warn"/><text x="34" y="122" class="t-b">Document v2 (bài 26)</text><text x="34" y="144" class="t-small">fields.id_number rõ</text><text x="34" y="162" class="t-small">schema_version = 2</text><text x="34" y="180" class="t-small">created_at: cũ hoặc mới</text><path d="M210 125 L268 70" class="f-none s-phys" stroke-width="2" marker-end="url(#py47-1-arrow)"/><rect x="270" y="30" width="280" height="80" rx="10" class="f-phys-soft s-phys"/><text x="284" y="56" class="t-b">1. raw (setOnInsert)</text><text x="284" y="78" class="t-small">plain: số rõ</text><text x="284" y="96" class="t-small">expire_at = created_at + 30 ngày</text><path d="M210 165 L268 215" class="f-none s-phys" stroke-width="2" marker-end="url(#py47-1-arrow)"/><rect x="270" y="180" width="280" height="80" rx="10" class="f-phys-soft s-phys"/><text x="284" y="206" class="t-b">2. sửa document chính</text><text x="284" y="228" class="t-small">lọc schema_version = 2, mỗi bản chạy</text><text x="284" y="246" class="t-small">chỉ sửa được một lần; số rõ rời đây</text><path d="M550 70 L610 70" class="f-none s-phys" stroke-width="2" marker-end="url(#py47-1-arrow)"/><rect x="612" y="30" width="250" height="80" rx="10" class="f-pk-soft s-pk"/><text x="626" y="56" class="t-b">TTL dọn raw</text><text x="626" y="78" class="t-small">cũ hơn 30 ngày: quá hạn ngay,</text><text x="626" y="96" class="t-small">bị xóa; mới hơn: ở lại tới hạn</text><path d="M550 220 L610 220" class="f-none s-phys" stroke-width="2" marker-end="url(#py47-1-arrow)"/><rect x="612" y="180" width="250" height="80" rx="10" class="f-pk-soft s-pk"/><text x="626" y="206" class="t-b">Document v3</text><text x="626" y="228" class="t-small">số đã che, hash, pii_key_id,</text><text x="626" y="246" class="t-small">raw_id; validator chỉ còn v3</text><text x="20" y="222" class="t-small">chưa migrate thì upgrade on read:</text><text x="20" y="240" class="t-small">trả dạng v3, không ghi DB</text></svg>
</div>
<figcaption>Hình 2. Migration một document. Số rõ chỉ rời khỏi collection chính sau khi đã có chỗ ở trong collection có hạn dùng.</figcaption>
</figure>

Migration và retention giao nhau: ba document cũ hơn 30 ngày sinh ra ba `raw` đã quá hạn ngay lúc tạo và TTL dọn chúng, ba `raw` còn lại ở lại. Trước khi siết validator bằng `collMod` chỉ còn v3, lab kiểm không còn document nào dưới v3; sau đó ghi một document v2 bị từ chối (mã 121).

## Chạy và kiểm chứng

Cần MongoDB của hạ tầng dùng chung. Lab đụng vào ba collection `lab47_*` (xóa khi xong) và, ở phần TTL, tham số `ttlMonitorSleepSecs` của cả `mongod`. Từ gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/47-luu-ket-qua-ocr-mongodb-pii examples/toi-noi-python/47-luu-ket-qua-ocr-mongodb-pii/main.py
```

Phần retention của demo (trích):

```text
index TTL của raw: 0 giây sau expire_at
ttlMonitorSleepSecs tạm đặt 1 giây: True
tính từ lúc chèn: A (đã quá hạn, kiểu date) biến mất sau 1.2 giây; E (hết hạn sau 3 giây) sau 3.3 giây
còn lại: ['KYC-9002', 'KYC-9003', 'KYC-9004'] (B còn hạn, C đã gỡ expire_at, D có expire_at là chuỗi)
```

A biến mất sau 1,2 giây (một chu kỳ quét 1 giây), E sau 3,3 giây, sát hạn 3 giây của nó, và đúng B, C, D ở lại. Nhật ký audit của demo (10 dòng; bỏ `expire_at`):

Kiểm chứng đầy đủ là chế độ `--verify`: 53 ca, chạy trên máy rất bận (load average ở dòng đầu). Output nguyên văn:

```bash
uv run --project examples/toi-noi-python/47-luu-ket-qua-ocr-mongodb-pii examples/toi-noi-python/47-luu-ket-qua-ocr-mongodb-pii/main.py --verify
```

```text
MongoDB mongodb://localhost:27019/toinoi; khóa băm của lab (không dùng ngoài lab); load average 1/5/15 phút: 333.74 265.88 199.72
  đạt  mask 12 chữ số: 3 đầu + 7 sao + 2 cuối, giữ độ dài
  đạt  mask từ chối chuỗi ngắn, chữ lẫn số, chữ số Arabic-Indic, chuỗi rỗng
  đạt  mặt nạ cộng ngày sinh và giới tính còn 10.000 ứng viên chứa số thật (không phải 100 như mặt nạ 4+4)
  đạt  băm: cùng đầu vào ra cùng giá trị; đổi loại giấy tờ hoặc khóa thì khác
  đạt  repr của PiiKey không lộ khóa
  đạt  thiếu khóa hoặc khóa dưới 32 ký tự: ValidationError, không có mặc định, lỗi không in lại giá trị
  đạt  SHA-256 không khóa của số CCCD bị dò ra bằng 10^6 lần thử
  đạt  kẻ tấn công có dữ liệu mà không có khóa: không dò ra
  đạt  lộ cả khóa lẫn dữ liệu thì dò ra như thường: khóa phải ở nơi khác cơ sở dữ liệu
  đạt  document mới: schema_version 3, created True
  đạt  số giấy tờ trong document chỉ còn dạng che: 099*******83
  đạt  toàn bộ document không chứa số giấy tờ rõ ở bất kỳ trường nào
  đạt  ngày sinh dd/mm/yyyy trên thẻ -> ISO; confidence là nhỏ nhất; mã mô hình và hash ảnh được giữ
  đạt  raw: số rõ + văn bản thô + khóa ảnh, expire_at là kiểu date, đúng created_at + 30 ngày
  đạt  validator v3 từ chối số rõ, trường lạ (trong và ngoài fields), hash rỗng, mặt nạ sai dạng, thiếu pii_key_id (mã 121)
  đạt  lưu lại cùng kết quả sau khi raw đã hết hạn: không thêm document, không dựng lại raw
  đạt  mô hình mới là document mới, bản cũ còn
  đạt  chụp lại ảnh (hash khác) với cùng mô hình: document mới, không mất kết quả của ảnh trước; bản mới nhất là bản chụp lại
  đạt  GPLX cùng cách: license_number_masked theo mẫu, không còn license_number
  đạt  kết quả không đọc ra số giấy tờ: vẫn được lưu, needs_review true, missing_fields ghi rõ, số che và hash là null
  đạt  số 11 chữ số và ngày 31/02/1990: lưu với needs_review, thiếu id_number và date_of_birth, không ném lỗi
  đạt  không có raw mồ côi sau các lần lưu (kể cả đầu vào xấu)
  đạt  cùng số CCCD ở hai hồ sơ: tra theo hash thấy cả hai hồ sơ (bốn document, vì KYC-0007 có ba)
  đạt  kết quả đối chiếu chỉ có dạng che
  đạt  số không có trong hệ thống: 0 kết quả
  đạt  tra theo hash dùng index ix_id_hash (IXSCAN), chỉ kiểm document trả về
  đạt  sau khi xoay khóa, tra bằng khóa mới một mình không thấy bản ghi cũ; phải giữ khóa cũ trong danh sách để tra
  đạt  tra số không có scope kyc:view: PermissionError (và ghi lookup_denied)
  đạt  raw có TTL index trên expire_at với expireAfterSeconds = 0
  đạt  audit có TTL index riêng, dài hơn raw
  đạt  document quá hạn (kiểu date) bị TTL xóa
  đạt  document còn hạn, document đã gỡ expire_at (giữ lại) và document có expire_at kiểu chuỗi đều KHÔNG bị xóa
  đạt  đối chứng: document hết hạn sau 3 giây cũng bị xóa, nên việc C còn lại là do gỡ expire_at
  đạt  xem mặc định: số đã che, không có số rõ
  đạt  xem số rõ có scope kyc:reveal và lý do: trả số rõ từ raw
  đạt  xem số rõ thiếu scope hoặc lý do ngắn/trống: RevealDenied, không trả gì
  đạt  audit không ghi được: ngoại lệ nổi lên, không có dữ liệu nào được trả (fail closed)
  đạt  audit ghi đủ 10 dòng theo thứ tự: 4 lookup, 1 lookup_denied, view, reveal, 3 reveal_denied
  đạt  dòng audit có actor, kyc_id, lý do; hết hạn sau 400 ngày; không chứa số giấy tờ (rõ hay che)
  đạt  bản gốc đã hết hạn: vẫn xem được dạng che; xin số rõ nhận raw_expired thay vì lỗi
  đạt  trước migrate: 6 document v2 của bài 26 mang số rõ (validator đang cho phép v2)
  đạt  upgrade on read: v2 đọc ra hình dạng v3 đã che, số rõ không qua tầng trên; DB chưa đổi
  đạt  view_document trên document v2 chưa migrate: vẫn trả dạng che
  đạt  reveal trên document v2 chưa migrate: số rõ còn trong document nên được trả, không báo raw_expired sai
  đạt  document v1 của bài 26 chưa qua migration v1 sang v2: ValueError rõ ràng, không đoán
  đạt  hai tác vụ migrate đồng thời: tổng số document được sửa đúng 6, không bản nào sửa lần hai
  đạt  sau migrate: không còn v2, không còn trường id_number, không số rõ nào trong cả collection
  đạt  document đã migrate bằng đúng kết quả của upgrade on read (che, băm, cờ needs_review)
  đạt  migrate đưa số rõ vào raw với expire_at = created_at + 30 ngày (không kéo dài thời hạn lưu cũ)
  đạt  ba document cũ hơn 30 ngày sinh ra raw đã quá hạn; ba document mới thì chưa
  đạt  TTL dọn ba raw quá hạn do migrate tạo ra; ba raw còn hạn ở lại
  đạt  trước khi siết validator: không còn document nào dưới v3
  đạt  siết validator chỉ còn v3: ghi v2 (số rõ) bị từ chối (mã 121)
OK 53/53
```

Để thấy validator thật sự làm việc, ta phá có chủ ý: bỏ dòng `"additionalProperties": False` ở cấp `fields`. Đúng một ca đỏ, và `[None, None, 121, 121, 121, 121]` nói rằng hai document, một mang `id_number` rõ, một mang `so_cccd`, đã lọt vào collection:

```text
  SAI  validator v3 từ chối số rõ, trường lạ (trong và ngoài fields), hash rỗng, mặt nạ sai dạng, thiếu pii_key_id (mã 121)  [[None, None, 121, 121, 121, 121]]
FAIL 52/53
```

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Document hết hạn vẫn còn sau nhiều phút | `expire_at` có phải kiểu date không; bộ quét TTL mặc định 60 giây |
| TTL xóa document nhưng ảnh vẫn còn trong kho | TTL không biết kho đối tượng; đặt luật vòng đời ở kho |
| Tra theo số không ra gì sau khi đổi khóa băm | Tra bằng cả khóa cũ lẫn mới; băm lại chỉ làm được cho bản còn `raw` |
| `WriteError` mã 121 khi lưu | Document khác `$jsonSchema`: số rõ hoặc trường lạ, hash sai mẫu, thiếu `pii_key_id` hoặc `raw_id` |
| `ttlMonitorSleepSecs` còn 1 sau lần chạy bị ngắt | Lần chạy sau tự cảnh báo và trả về 60 |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Một MongoDB đơn lẻ, không replica set: ví dụ không nói gì về độ bền khi ghi hay failover, và ta dựa vào thứ tự ghi cùng idempotency thay vì transaction. Khóa băm là chuỗi công khai trong <code>main.py</code>; production dùng secret manager và xoay khóa theo lịch. Mặt nạ để lộ ba số đầu và hai số cuối, và băm không thay thế mã hóa: ai có cả khóa lẫn dữ liệu vẫn dò lại được số giấy tờ, nên dữ liệu nhạy cảm còn cần mã hóa khi lưu và kiểm soát truy cập. Các hạn 30 và 400 ngày là giá trị lab, thời hạn thật do pháp chế quyết định; bài này không phải tư vấn pháp lý. Lab không có xóa theo yêu cầu của chủ thể dữ liệu (kể cả sao lưu), và audit chưa được bảo vệ ở mức quyền cơ sở dữ liệu. Số liệu chạy trên macOS Apple M1 Max rất bận, Python 3.14.7, pymongo 4.19.0, MongoDB 8.3.13.</p>
</div>

**Bài tập 1:** `KYC-0007` có ba document nên một lần tra theo số trả ba dòng của cùng hồ sơ. Người duyệt chỉ muốn bản mới nhất mỗi hồ sơ. Bạn đổi `find_by_number` thế nào?

**Bài tập 2:** Pháp chế yêu cầu giữ số CCCD của một hồ sơ đang bị khiếu nại quá 30 ngày. Làm thế nào bằng thứ lab đã có, và quên thì có hại gì?

<details><summary>Xem lời giải</summary><p><b>1.</b> Aggregation: <code>$match</code> theo loại, <code>pii_key_id</code> và hash (dùng <code>ix_id_hash</code>), <code>$sort</code> theo <code>created_at</code> giảm dần, <code>$group</code> theo <code>kyc_id</code> với <code>$first</code>; hoặc chọn bản mới nhất mỗi <code>kyc_id</code> trong Python khi mỗi số chỉ có vài document. <b>2.</b> Gỡ trường <code>expire_at</code> của đúng document <code>raw</code> đó bằng <code>$unset</code>, như document C trong thí nghiệm TTL. Quên đặt lại <code>expire_at</code> khi vụ việc kết thúc thì số rõ nằm đó vô thời hạn, trái chính sách.</p></details>

Bài trước: [bài 46](/articles/python-ocr-service-fastapi-worker-timeout). Bài tiếp: [bài 48](/articles/python-face-detection-landmarks-mediapipe-opencv).

## Tài liệu gốc

- [MongoDB: TTL Indexes](https://www.mongodb.com/docs/manual/core/index-ttl/).
- [MongoDB: Schema Validation, $jsonSchema](https://www.mongodb.com/docs/manual/core/schema-validation/).
- [MongoDB: collMod](https://www.mongodb.com/docs/manual/reference/command/collmod/).
- [PyMongo: AsyncMongoClient](https://pymongo.readthedocs.io/en/stable/api/pymongo/asynchronous/mongo_client.html).
- [Python: hmac](https://docs.python.org/3.14/library/hmac.html).
- [Pydantic Settings: secrets](https://pydantic.dev/docs/validation/latest/concepts/pydantic_settings/).
