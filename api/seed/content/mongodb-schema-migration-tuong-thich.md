---
slug: mongodb-schema-migration-tuong-thich
title: "Đổi schema khi không có ALTER TABLE: thêm biển số xe cho tài xế Tới Nơi"
summary: "Tới Nơi bắt buộc mọi tài xế phải có biển số xe, trong khi collection drivers đã có dữ liệu và app cũ vẫn chạy. MongoDB không có ALTER TABLE: bài này đổi validator và document theo expand → backfill → kiểm → contract, dùng validationLevel/validationAction, schema_version và một kịch bản hoàn tác chạy lặp được."
category: mongodb
level: advanced
tags: [mongodb, mg-009, migration, collmod, validation, schema-versioning, expand-contract, toi-noi]
language: vi
---

Bộ phận an toàn của Tới Nơi chốt yêu cầu: khách phải thấy **biển số xe** trước khi lên xe, nên mọi tài xế đều phải có biển số đúng định dạng. Đây là đúng yêu cầu mà series PostgreSQL đã xử lý ở [PG-009](/articles/postgresql-schema-migration-tuong-thich). Một bạn trong đội nghe xong thì nói: "MongoDB là schemaless, cứ ghi thêm trường `vehicle_plate` là xong, cần gì migration."

Câu đó sai ở hai chỗ. Thứ nhất, collection `drivers` của lab có validator với `additionalProperties: false`: ghi thêm một trường lạ sẽ nhận lỗi 121 ngay. Thứ hai, kể cả nới validator, bảy tài xế đang có trong collection không tự có biển số. App đăng ký tài xế bản cũ vẫn đang chạy và chưa biết biển số là gì, còn danh sách biển số do bộ phận tuyển dụng gửi sang có ít nhất một dòng viết sai. Bài này đổi "schema" của `drivers` theo từng bước, mỗi bước đều kiểm được và có đường lùi.

<div class="tkh-callout">
<span class="tkh-callout-title">MG-009 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/mongodb-schema-validation-unique-index">MG-005 · Schema validation với <code>$jsonSchema</code></a>, <a href="/articles/mongodb-insert-update-delete">MG-006 · insert, update, delete</a>, <a href="/articles/mongodb-nhung-hay-tham-chieu">MG-008 · Nhúng hay tham chiếu</a>. Lab: <code>examples/toi-noi-mongodb/p01</code> (<code>06-migration.js</code>, <code>06-migration-rollback.js</code>). Sau bài này, bạn thực hiện được một migration expand → backfill → contract bằng <code>collMod</code>, chọn đúng <code>validationLevel</code>/<code>validationAction</code> cho từng giai đoạn, đánh dấu phiên bản bằng <code>schema_version</code> và hoàn tác được.</p>
</div>

## 1. "Schema" trong MongoDB là hai thứ

PostgreSQL có một catalog duy nhất mô tả bảng: thêm cột bằng `ALTER TABLE` là mọi dòng, cũ và mới, đều có cột đó (có thể mang giá trị NULL). MongoDB không có câu lệnh tương đương, vì "schema" của một collection thực chất gồm **hai thứ độc lập**:

1. **Hình dạng của các document đang nằm trên đĩa.** Mỗi document tự mang tên trường và kiểu của mình. Không có nơi nào ghi "collection này có những cột gì".
2. **Validator** của collection: bộ luật kiểm mỗi lần ghi, lưu trong tuỳ chọn của collection (MG-005).

Đổi một thứ **không** đổi thứ kia:

| Bạn làm | Document cũ | Validator |
|---|---|---|
| `collMod` đặt validator mới | Không bị sửa, không bị quét, kể cả khi vi phạm luật mới | Đổi ngay |
| `updateMany` thêm trường cho mọi document | Đổi | Không đổi; còn có thể bị chính validator chặn |
| Ứng dụng bắt đầu ghi trường mới | Chỉ document được ghi mới có trường | Không đổi; trường lạ bị chặn nếu `additionalProperties: false` |

Hệ quả là migration trong MongoDB luôn có một khoảng thời gian mà **document cũ và document mới có hình dạng khác nhau cùng tồn tại**. Việc của người thiết kế là làm cho khoảng thời gian ấy an toàn và kiểm được, thay vì giả vờ nó không có.

`06-migration.js` không chép tay validator từ `01-schema.js` mà đọc nó từ catalog, rồi dựng phiên bản mới dựa trên đó:

```js
const PLATE = { bsonType: 'string', pattern: '^[0-9]{2}[A-Z][A-Z0-9]?-[0-9]{3}\\.[0-9]{2}$', description: 'biển số dạng 59X3-123.45 / 51G-123.45' };

// Đọc validator hiện tại từ catalog (không chép tay lại 01-schema.js).
const v1 = db.getCollectionInfos({ name: 'drivers' })[0].options.validator.$jsonSchema;
const clone = (o) => EJSON.parse(EJSON.stringify(o));

function schemaV2({ requirePlate }) {
  const s = clone(v1);
  s.properties.vehicle_plate = PLATE;
  s.properties.schema_version = { bsonType: 'int', enum: [2] };
  if (requirePlate) s.required = [...s.required, 'vehicle_plate', 'schema_version'];
  return { $jsonSchema: s };
}
```

Định dạng biển số của lab là "2 số tỉnh + seri 1–2 ký tự + `-` + 3 số + `.` + 2 số", cùng biểu thức với lab PostgreSQL. Đây không phải quy chuẩn biển số đầy đủ của Việt Nam.

## 2. Kế hoạch: expand → backfill → kiểm → contract

Cách tách bước giống hệt PG-009, chỉ khác công cụ:

| Bước | Validator `drivers` | Document | App v1 (không biết biển số) | App v2 (luôn gửi biển số) |
|---|---|---|---|---|
| 1 · Expand | Cho phép `vehicle_plate` và `schema_version`, **chưa bắt buộc** | Chưa đổi | Ghi được | Ghi được |
| 2 · Backfill | Như bước 1 | Tài xế cũ được điền biển số theo lô | Ghi được | Ghi được |
| 3 · Kiểm | Như bước 1 | Đếm document không thoả validator đích = 0 | Ghi được | Ghi được |
| 4 · Contract | Bắt buộc hai trường mới, `strict` + `error` | Không đổi | **Bị từ chối** khi tạo tài xế | Ghi được |

Mỗi bước tự đứng được: dừng ở giữa thì hệ thống vẫn chạy. Bước 4 là bước duy nhất phá vỡ app v1, nên chỉ chạy khi app v2 đã triển khai xong ở mọi nơi. Trong lab, `06-migration.js` chạy cả bốn bước liền nhau; ở hệ thật, giữa bước 1 và bước 4 có thể là nhiều ngày.

## 3. Bước 1 · Expand: cho phép trường mới

**`collMod`** là lệnh đổi tuỳ chọn của một collection đã có: validator, `validationLevel`, `validationAction` và một số thiết lập khác. Lệnh này chỉ đổi metadata, không đọc hay sửa document nào.

```js
db.runCommand({ collMod: 'drivers', validator: schemaV2({ requirePlate: false }), validationLevel: 'strict', validationAction: 'error' });
// Phiên bản ứng dụng cũ (chưa biết biển số) vẫn ghi được:
db.drivers.updateOne({ code: 8003 }, { $set: { status: 'online' } });
print('PASS app cũ cập nhật tài xế 8003 không có biển số');
// Phiên bản mới ghi tài xế có biển số:
db.drivers.insertOne({
  _id: D(8008), code: 8008, full_name: 'Kiều Thị Sen', city_id: 'HN', vehicle_type: 'car', status: 'offline',
  created_at: vn('2026-11-03 09:30'), vehicle_plate: '30K-456.78', schema_version: 2,
});
expectError('biển số sai định dạng bị chặn ngay từ expand', () => db.drivers.updateOne({ code: 8008 }, { $set: { vehicle_plate: '30K45678' } }), 121, 'pattern');
```

```text
/* Bước 1 · expand: cho phép vehicle_plate, chưa bắt buộc */
PASS app cũ cập nhật tài xế 8003 không có biển số
PASS biển số sai định dạng bị chặn ngay từ expand → lỗi 121 (pattern)
```

Ba điều xảy ra cùng lúc ở bước này. App v1 cập nhật tài xế 8003 (chưa có biển số) vẫn được, vì `vehicle_plate` chưa nằm trong `required`. App v2 tạo tài xế 8008 kèm biển số được, vì trường mới đã có trong `properties` nên `additionalProperties: false` không chặn nữa. Và biển số sai định dạng bị chặn **ngay từ bây giờ**: luật định dạng áp dụng cho mọi lần ghi có trường đó, dù trường chưa bắt buộc. Đây là điểm tương đương với `CHECK … NOT VALID` của PostgreSQL: dữ liệu mới đã sạch, dữ liệu cũ tính sau.

## 4. Bước 2 · Backfill: điền biển số cho tài xế cũ

**Backfill** (điền ngược) là ghi giá trị trường mới cho các document có từ trước. Biển số lấy từ hệ thống đăng ký xe cũ, và như mọi dữ liệu nguồn thật, nó có một dòng bẩn: tài xế 8004 được ghi là `' 30h-135.79 '`, có khoảng trắng và chữ thường.

```js
const source = [
  [8001, '59X3-123.45'], [8002, '59X1-678.90'], [8003, '29B1-246.80'],
  [8004, ' 30h-135.79 '], [8005, '43D1-112.23'], [8006, '43A-998.87'], [8007, '59X2-555.01'],
];
try {
  db.drivers.bulkWrite(
    source.map(([code, plate]) => ({ updateOne: {
      filter: { code, vehicle_plate: { $exists: false } },
      update: { $set: { vehicle_plate: plate, schema_version: 2 } },
    } })),
    { ordered: false },
  );
  print('FAIL lô đầu không có lỗi, mong đợi dòng 8004 bị chặn');
} catch (e) {
  expectValue('lần 1: số tài xế đã ghi biển số', e.result.modifiedCount, 6);
  expectValue('lần 1: lỗi tại vị trí (mã lỗi)', e.writeErrors.map((w) => `${w.index} (${w.code})`).join(', '), '3 (121)');
}
```

```text
/* Bước 2 · backfill theo lô (bulkWrite), dữ liệu nguồn có dòng bẩn */
PASS lần 1: số tài xế đã ghi biển số = 6
PASS lần 1: lỗi tại vị trí (mã lỗi) = 3 (121)
PASS còn thiếu biển số = 8004
```

### `bulkWrite` với `ordered: false`

**`bulkWrite`** gửi nhiều thao tác ghi trong một lệnh. Mặc định (`ordered: true`), server dừng ở thao tác lỗi đầu tiên và bỏ qua mọi thao tác phía sau. Với `ordered: false`, server **chạy hết lô**; thao tác lỗi không chặn các thao tác khác. Cuối lô, mongosh ném `MongoBulkWriteError` mang theo hai thứ:

- `e.result.modifiedCount`: số document đã sửa thành công, ở đây là **6**;
- `e.writeErrors`: danh sách lỗi theo **vị trí trong lô**. Vị trí `3` (đếm từ 0) là dòng 8004, mã **121** vì `' 30h-135.79 '` không khớp pattern.

Đây là hành vi mong muốn cho backfill: một dòng bẩn không được làm hỏng sáu dòng sạch, và lỗi phải được báo rõ để xử lý riêng. Đừng bắt lỗi rồi bỏ qua: `catch` ở đây kiểm đúng số lượng và vị trí lỗi.

### Xử lý dòng lỗi và backfill idempotent

Dòng lỗi được chuẩn hoá (cắt khoảng trắng, viết hoa) rồi ghi lại:

```js
for (const [code, raw] of source.filter(([c]) => left.includes(c))) {
  const plate = raw.trim().toUpperCase();
  db.drivers.updateOne({ code, vehicle_plate: { $exists: false } }, { $set: { vehicle_plate: plate, schema_version: 2 } });
}
```

```text
PASS 8004 sau chuẩn hoá = 30H-135.79
PASS backfill lần 2 (idempotent) = 0
```

Chuẩn hoá ở đây là quyết định nghiệp vụ, không phải mẹo kỹ thuật: lab chỉ sửa khoảng trắng và chữ hoa vì chắc chắn chúng không đổi nghĩa. Một biển số thiếu chữ số thì phải hỏi lại nguồn, không được đoán.

Dòng cuối là điều quan trọng nhất của backfill. Bộ lọc `vehicle_plate: { $exists: false }` làm cho mỗi thao tác chỉ khớp tài xế **chưa có** biển số. Chạy lại toàn bộ lô lần thứ hai sửa 0 document. Thao tác như vậy gọi là **idempotent**: chạy một lần hay nhiều lần cho cùng kết quả. Với collection lớn, backfill chạy theo nhiều lô và có thể bị ngắt giữa chừng (mất kết nối, khởi động lại máy). Bộ lọc idempotent cho phép chạy lại từ đầu mà không ghi đè dữ liệu đã đúng, cũng không ghi đè biển số mà app v2 vừa ghi cho tài xế mới.

Lab có 7 tài xế nên một lô là đủ. Với hàng chục nghìn tài xế, chia lô theo `_id` (lấy N document chưa có trường, ghi, lặp lại) và giới hạn tốc độ để không tranh tài nguyên với tải thật.

## 5. Bước 3 · Kiểm: còn bao nhiêu document không đạt luật đích?

Trước khi siết luật, phải chứng minh dữ liệu cũ đã đạt luật mới. Validator là một biểu thức truy vấn, nên dùng chính nó để tìm document **không** thoả bằng `$nor`:

```js
const target = schemaV2({ requirePlate: true });
expectValue('drivers không thoả validator đích', db.drivers.countDocuments({ $nor: [target] }), 0);
```

```text
/* Bước 3 · document không thoả validator đích */
PASS drivers không thoả validator đích = 0
```

`$nor: [target]` khớp document mà `target` **không** khớp. Kết quả 0 nghĩa là áp validator đích bây giờ sẽ không có document nào "nằm ngoài luật". Kiểm bằng **đúng biểu thức** sẽ được đặt vào `collMod` ở bước 4, không phải bằng một truy vấn viết tay "tương đương", để tránh lệch giữa điều được kiểm và điều được áp.

### Nếu bỏ qua bước kiểm thì sao?

`collMod` không quét document cũ, nên nó vui vẻ nhận validator mà dữ liệu hiện tại không đạt. Để thấy hậu quả, mình chép 8 tài xế sau migration sang database nháp `scratch_d`, xoá biển số của 3 tài xế (8004, 8005, 8006), rồi áp validator đích đọc từ `toinoi.drivers`. Lab MongoDB 8.0.38, database nháp đã xoá sau khi chạy:

```text
collMod xong; $nor = 3
validate valid = true nNonCompliantDocuments = 3 warnings = ["Detected one or more documents not compliant with the collection's schema. Check logs for log id 5363500."]
strict: update 8006 lỗi 121
```

`collMod` thành công không một lời cảnh báo. `$nor` đếm được 3 document vi phạm. Lệnh **`validate()`** (kiểm tra cấu trúc collection, từ MongoDB 5.0 kiêm đếm document không thoả validator) báo `nNonCompliantDocuments: 3` nhưng vẫn `valid: true`: document vi phạm validator không phải là collection hỏng, chỉ là một cảnh báo kèm log id 5363500. Đừng chỉ nhìn `valid`.

Dòng cuối mới là hậu quả thật: với `validationLevel: 'strict'`, app v1 chỉ đổi `status` của tài xế 8006 cũng bị từ chối, vì document **sau khi sửa** vẫn thiếu biển số. Siết luật trước khi backfill xong là làm hỏng cả những thao tác chẳng liên quan gì tới biển số.

## 6. `validationLevel` và `validationAction`

Hai tuỳ chọn của collection quyết định validator được áp **cho những lần ghi nào** và **làm gì khi vi phạm**:

| Tuỳ chọn | Giá trị | Ý nghĩa |
|---|---|---|
| `validationLevel` | `strict` (mặc định) | Kiểm mọi insert và mọi update |
| | `moderate` | Kiểm mọi insert; kiểm update **chỉ khi document trước khi sửa đã hợp lệ**. Document đang vi phạm thì sửa không bị kiểm |
| | `off` | Không kiểm gì |
| `validationAction` | `error` (mặc định) | Từ chối thao tác ghi, lỗi 121 |
| | `warn` | Cho ghi, chỉ ghi một dòng cảnh báo vào log của server |

### `moderate`: không chặn sửa document cũ

Tiếp tục thí nghiệm trong `scratch_d`, đổi sang `moderate` rồi thử ba thao tác:

```text
moderate: update 8006 matched/modified 1/1
moderate: unset plate của 8001 (đang hợp lệ) lỗi 121
moderate: insert thiếu biển số lỗi 121
```

Tài xế 8006 (đang thiếu biển số) đổi trạng thái được, vì document đang vi phạm nên update không bị kiểm. Tài xế 8001 (đang hợp lệ) thì không được xoá biển số: document hợp lệ phải giữ hợp lệ. Tài xế mới thiếu biển số vẫn bị chặn, vì insert luôn bị kiểm.

`moderate` hữu ích khi buộc phải siết luật trước khi backfill xong, ví dụ chặn dữ liệu mới sai ngay lập tức trong khi việc dọn dữ liệu cũ kéo dài nhiều tuần. Đổi lại, nó không cho biết document nào còn vi phạm; vẫn phải đếm bằng `$nor` hoặc `validate()`. Lab không cần `moderate` vì backfill xong trước khi siết.

### `warn`: quan sát trước khi siết

`validationAction: 'warn'` cho một giai đoạn quan sát: áp validator đích, để ứng dụng chạy như thật, rồi đọc log xem có thao tác nào lẽ ra bị từ chối. Lab thử bằng cách chèn một tài xế thiếu biển số:

```js
db.runCommand({ collMod: 'drivers', validator: target, validationLevel: 'strict', validationAction: 'warn' });
const warned = { _id: UUID(), code: 8099, full_name: 'Thiếu Biển Số', city_id: 'SG', vehicle_type: 'bike', status: 'offline', created_at: vn('2026-11-03 09:40') };
db.drivers.insertOne(warned);
const logLine = db.adminCommand({ getLog: 'global' }).log.map((l) => JSON.parse(l)).reverse()
  .find((l) => l.id === 20294 && l.attr?.namespace === 'toinoi.drivers');
expectValue('log id 20294 "Document would fail validation"', logLine?.msg, 'Document would fail validation');
db.drivers.deleteOne({ _id: warned._id });
```

```text
/* validationAction "warn": ghi được, chỉ để lại log */
PASS log id 20294 "Document would fail validation" = Document would fail validation
```

Insert thành công; server để lại dòng log có cấu trúc với `id` **20294** và `msg` `"Document would fail validation"`, kèm namespace `toinoi.drivers`. Trong hệ thật, bạn lọc log theo id này (hoặc đưa vào hệ thống giám sát) trong vài ngày; khi không còn dòng nào từ ứng dụng, bước contract an toàn. Lab xoá tài xế thử ngay sau đó.

`warn` không phải trạng thái để lại lâu dài: validator "chỉ cảnh báo" là validator không bảo vệ gì cả, và log có thể bị xoay vòng trước khi ai đọc.

## 7. Bước 4 · Contract: bắt buộc biển số

Khi app v2 đã triển khai hết và dữ liệu đã sạch, áp validator đích với `strict` + `error`:

```js
db.runCommand({ collMod: 'drivers', validator: target, validationLevel: 'strict', validationAction: 'error' });
expectError('tài xế mới thiếu biển số', () => db.drivers.insertOne(warned), 121, 'vehicle_plate');
expectError('$unset biển số của 8001', () => db.drivers.updateOne({ code: 8001 }, { $unset: { vehicle_plate: '' } }), 121, 'vehicle_plate');
```

```text
/* Bước 4 · contract: bắt buộc biển số, strict + error */
PASS tài xế mới thiếu biển số → lỗi 121 (vehicle_plate)
PASS $unset biển số của 8001 → lỗi 121 (vehicle_plate)

/* validate: kiểm toàn bộ collection với validator hiện tại */
valid|nrecords|nNonCompliantDocuments
-----|--------|----------------------
true|8|0
(1 document)
PASS nNonCompliantDocuments = 0
```

`validate()` sau contract là bằng chứng thứ hai, độc lập với `$nor` ở bước 3: 8 document, 0 không thoả. Trên collection lớn `validate()` quét toàn bộ dữ liệu và index, nên chạy ngoài giờ cao điểm.

Kết quả cuối của migration:

```text
/* Tài xế sau migration */
code|full_name|vehicle_type|vehicle_plate|schema_version
----|---------|------------|-------------|--------------
8001|Bùi Văn Khoa|bike|59X3-123.45|2
8002|Hồ Thị Lan|bike|59X1-678.90|2
8003|Ngô Quang Minh|bike|29B1-246.80|2
8004|Đỗ Thanh Nam|car|30H-135.79|2
8005|Lý Thị Oanh|bike|43D1-112.23|2
8006|Trịnh Văn Phúc|car|43A-998.87|2
8007|Mai Văn Quý|bike|59X2-555.01|2
8008|Kiều Thị Sen|car|30K-456.78|2
(8 document)
```

Trong DBStudio, chạy `db.drivers.find({}, { _id: 0, code: 1, vehicle_plate: 1, schema_version: 1 }).sort({ code: 1 })`: cả 8 tài xế đều có `vehicle_plate` và `schema_version: 2`. Nhìn kỹ, thứ tự hai trường mới ở 8008 khác các tài xế còn lại.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-schema-migration-tuong-thich/dbstudio-01.webp" alt="DBStudio liệt kê 8 tài xế sau migration, mỗi tài xế có vehicle_plate và schema_version 2" width="1086" height="665" loading="lazy"></div><figcaption>DBStudio · 8 tài xế sau migration; 8008 có thứ tự trường khác</figcaption></figure>

Một chi tiết nhỏ khi nhìn từng document: thứ tự trường không giống nhau. Đọc trên lab:

```text
_id,code,full_name,city_id,vehicle_type,status,created_at,schema_version,vehicle_plate
_id,code,full_name,city_id,vehicle_type,status,created_at,vehicle_plate,schema_version
```

Dòng đầu là tài xế 8001 (được backfill bằng `$set`), dòng sau là 8008 (được app v2 `insertOne`). Khi `$set` thêm nhiều trường mới, MongoDB thêm chúng theo thứ tự chữ cái của tên trường; còn `insertOne` giữ đúng thứ tự trong object bạn gửi. Ứng dụng không nên phụ thuộc vào thứ tự trường, và công cụ GUI có thể hiện cột theo thứ tự khác nhau tuỳ document được đọc đầu tiên.

## 8. `schema_version`: ứng dụng đọc được cả hai hình dạng

Trường `schema_version: 2` không phục vụ validator; nó phục vụ **ứng dụng**. Đây là **schema versioning pattern**: mỗi document tự ghi mình thuộc phiên bản hình dạng nào, và code đọc xử lý từng phiên bản một cách tường minh. Document không có trường này được hiểu là phiên bản 1.

Trong khoảng giữa expand và contract, app v2 phải đọc được cả hai. Một đoạn phác thảo phía ứng dụng (không có trong lab):

```js
function driverView(d) {
  const version = d.schema_version ?? 1;
  switch (version) {
    case 1: return { ...d, vehicle_plate: null, plate_status: 'chưa cập nhật' };
    case 2: return { ...d, plate_status: 'đã có' };
    default: throw new Error(`schema_version ${version} chưa được hỗ trợ — app cần nâng cấp`);
  }
}
```

Nhánh `default` quan trọng không kém: app cũ gặp document của phiên bản tương lai phải báo lỗi rõ ràng, không được đọc sai. Với `schema_version`, câu hỏi "còn bao nhiêu document ở phiên bản cũ" cũng chỉ là một truy vấn: `db.drivers.countDocuments({ schema_version: { $exists: false } })`.

Lab chọn backfill **chủ động** (*eager*): một script điền hết rồi mới siết. Cách còn lại là **lười** (*lazy*): ứng dụng nâng document lên phiên bản mới khi có dịp ghi vào nó. Cách lười hợp với collection rất lớn và trường không bắt buộc, nhưng không bao giờ "xong" nếu có document không ai ghi tới, nên không dùng được cho một trường **bắt buộc** như biển số. Cuối cùng vẫn cần một lần backfill để dọn phần đuôi.

## 9. Hoàn tác: thứ tự không được đảo

Migration nào cũng cần đường lùi, viết và chạy thử **trước** khi cần. `06-migration-rollback.js` đưa `drivers` về hình dạng v1 theo bốn bước:

```js
// 1. nới validator: biển số không bắt buộc, nhưng vẫn được phép có
const lenient = EJSON.parse(EJSON.stringify(cur));
lenient.required = v1.required;
db.runCommand({ collMod: 'drivers', validator: { $jsonSchema: lenient }, validationLevel: 'strict', validationAction: 'error' });

// 2. sao lưu biển số (upsert theo _id, chạy lại không nhân đôi)
const backup = db.drivers.find({ vehicle_plate: { $exists: true } }, { vehicle_plate: 1, code: 1 }).toArray();
for (const d of backup) {
  db.drivers_plate_backup.updateOne({ _id: d._id }, { $set: { code: d.code, vehicle_plate: d.vehicle_plate, saved_at: new Date() } }, { upsert: true });
}

// 3. bỏ hai trường
const unset = db.drivers.updateMany({}, { $unset: { vehicle_plate: '', schema_version: '' } });

// 4. đặt lại validator v1 (không có hai trường mới, additionalProperties: false)
db.runCommand({ collMod: 'drivers', validator: { $jsonSchema: v1 }, validationLevel: 'strict', validationAction: 'error' });
```

Vì sao đúng thứ tự này?

- **Nới trước khi `$unset`.** Validator đích bắt buộc biển số; bước 4 của migration đã cho thấy `$unset` biển số bị lỗi 121. Không nới thì bước 3 bị chính validator chặn.
- **Nới chứ không về thẳng v1.** Nếu đặt validator v1 ngay (không có `vehicle_plate` trong `properties`, `additionalProperties: false`), cả 8 document đang có biển số lập tức vi phạm. Với `strict`, mọi lần app v2 sửa tài xế (đổi trạng thái `busy`, `online`) đều bị từ chối cho tới khi `$unset` xong. Validator "nới" vẫn cho phép biển số tồn tại, nên app v2 vẫn chạy trong lúc hoàn tác.
- **Sao lưu trước khi xoá.** `$unset` không lấy lại được. Biển số đã thu thập là dữ liệu nghiệp vụ; lý do hoàn tác thường là lỗi code, không phải vì dữ liệu sai. Sao lưu bằng upsert theo `_id` nên chạy lại không tạo bản trùng.
- **Về v1 sau cùng**, khi không còn document nào mang trường mới.

Tài xế 8008 do app v2 tạo được giữ lại, chỉ mất hai trường. Script kiểm bốn điều sau khi chạy:

```js
expectValue('R1 còn tài xế có vehicle_plate', db.drivers.countDocuments({ vehicle_plate: { $exists: true } }), 0);
expectValue('R2 số biển số đã sao lưu', db.drivers_plate_backup.countDocuments(), 8);
expectValue('R3 validator không còn vehicle_plate', 'vehicle_plate' in db.getCollectionInfos({ name: 'drivers' })[0].options.validator.$jsonSchema.properties, false);
expectValue('R4 drivers không thoả validator v1', db.drivers.validate().nNonCompliantDocuments, 0);
```

Bộ test của lab (`npm test`) chạy script hoàn tác **hai lần liên tiếp** sau `npm run p01` và yêu cầu cả hai lần đều thoát mã 0 và in `PASS R1 còn tài xế có vehicle_plate = 0`, `PASS R2 số biển số đã sao lưu = 8`; R3 và R4 được in cùng dạng `PASS`/`FAIL` để người chạy đọc. Lần thứ hai không còn biển số để sao lưu hay xoá, nhưng kết quả cuối vẫn như lần đầu: script hoàn tác cũng idempotent. Script này không nằm trong `npm run p01`; test chạy lại toàn bộ p01 sau đó để database `toinoi` trở về trạng thái đầy đủ.

## 10. So với PostgreSQL và SQL Server

| Chủ đề | PostgreSQL | SQL Server | MongoDB |
|---|---|---|---|
| Thêm "cột" | `ALTER TABLE … ADD COLUMN`, mọi dòng có cột (NULL) | `ALTER TABLE … ADD` | Không có; sửa validator (`collMod`) và document (backfill) riêng rẽ |
| Luật mới chưa kiểm dữ liệu cũ | `ADD CONSTRAINT … NOT VALID` | `WITH NOCHECK` (ràng buộc bị đánh dấu `is_not_trusted`) | Mặc định: `collMod` không bao giờ quét dữ liệu cũ |
| Kiểm dữ liệu cũ | `VALIDATE CONSTRAINT`, lỗi không nói dòng nào | `WITH CHECK CHECK CONSTRAINT` | `countDocuments({ $nor: [validator] })` trả đúng document vi phạm; `validate()` đếm `nNonCompliantDocuments` |
| Cho sửa dòng cũ đang vi phạm | Không có tương đương | Không có tương đương | `validationLevel: 'moderate'` |
| Chỉ cảnh báo, không chặn | Không có | Không có | `validationAction: 'warn'`, log id 20294 |
| Đổi schema trong transaction | DDL có transaction, rollback được | Phần lớn DDL có transaction | `collMod` không hoàn tác bằng transaction; cần script hoàn tác |

Điểm khác cốt lõi: ở PostgreSQL, sau khi `VALIDATE` thành công thì database **cam kết** mọi dòng đạt luật, và catalog ghi nhận điều đó. Ở MongoDB, validator chỉ là cổng kiểm lúc ghi; không có trạng thái "đã được kiểm toàn bộ" nào được lưu. Bằng chứng nằm ở việc bạn chạy `$nor` hoặc `validate()`, và chạy lại sau mỗi lần nghi ngờ (`bypassDocumentValidation`, restore từ bản sao lưu cũ, nhập dữ liệu hàng loạt).

## 11. Những hiểu nhầm thường gặp

- **"Schemaless nên không cần migration."** Không có `ALTER TABLE` không có nghĩa không có schema; nó có nghĩa document cũ và mới có thể khác hình dạng mà không ai biết. Migration là cách làm cho sự khác biệt đó có kế hoạch và kiểm được.
- **"`collMod` thành công nghĩa là dữ liệu đã đạt luật mới."** `collMod` không quét document nào. Thí nghiệm ở mục 5 nhận validator khi 3 document đang vi phạm.
- **"`validate()` trả `valid: true` là ổn."** Document không thoả validator chỉ làm tăng `nNonCompliantDocuments` và thêm cảnh báo; `valid` vẫn `true`.
- **"`ordered: false` để nhanh hơn."** Lý do chính trong migration là để một dòng lỗi không chặn các dòng sau, và để nhận đủ danh sách lỗi theo vị trí. Vẫn phải bắt `MongoBulkWriteError` và xử lý từng lỗi.
- **"Backfill chạy một lần là đủ."** Backfill thật bị ngắt, chạy lại, chạy song song với app v2. Không có bộ lọc kiểu `$exists: false` thì lần chạy lại ghi đè dữ liệu mới hơn.
- **"Để `warn` cho an toàn."** `warn` không chặn gì. Chỉ dùng nó cho giai đoạn quan sát có thời hạn và có người đọc log.
- **"Hoàn tác là chạy migration theo chiều ngược."** Thứ tự bước hoàn tác phải xét lại theo validator đang áp: về thẳng v1 trước khi `$unset` sẽ chặn chính app v2 đang chạy.

## 12. Bài tập

**Bài 1.** Một đồng nghiệp đề xuất gộp migration thành hai câu: `collMod` validator đích (bắt buộc biển số), rồi `bulkWrite` backfill. Dữ liệu và app như trong bài. Điều gì xảy ra trong khoảng giữa hai câu, và với dòng 8004?

<details><summary><b>Lời giải</b></summary><p>Giữa hai câu, 7 tài xế cũ đều vi phạm validator. Với <code>strict</code> + <code>error</code>, mọi update của app v1 lên tài xế cũ (đổi <code>status</code>) bị lỗi 121, giống thí nghiệm <code>strict: update 8006 lỗi 121</code>; app v1 cũng không tạo được tài xế mới. Backfill vẫn chạy được vì document sau khi <code>$set</code> đạt luật, nhưng dòng 8004 bị lỗi 121 và tài xế 8004 tiếp tục vi phạm cho tới khi có người chuẩn hoá tay, nghĩa là trong suốt thời gian đó tài xế 8004 không đổi được trạng thái. Đúng thứ tự expand → backfill → kiểm → contract tránh cả hai vấn đề.</p></details>

**Bài 2.** Viết lại backfill cho trường hợp collection có 60.000 tài xế (số tài xế đăng ký theo hồ sơ Tới Nơi), mỗi lô 500, chạy lại được khi bị ngắt giữa chừng. Biển số lấy từ một hàm `lookupPlate(code)` có thể trả chuỗi bẩn hoặc `null`.

<details><summary><b>Lời giải</b></summary><p>Phác thảo (không có trong lab):</p>
<pre><code class="language-js">const PLATE_RE = /^[0-9]{2}[A-Z][A-Z0-9]?-[0-9]{3}\.[0-9]{2}$/;
let lastId = null, skipped = [];
while (true) {
  const filter = { vehicle_plate: { $exists: false }, ...(lastId ? { _id: { $gt: lastId } } : {}) };
  const batch = db.drivers.find(filter, { code: 1 }).sort({ _id: 1 }).limit(500).toArray();
  if (batch.length === 0) break;
  lastId = batch.at(-1)._id;
  const ops = [];
  for (const d of batch) {
    const plate = lookupPlate(d.code)?.trim().toUpperCase();
    if (!plate || !PLATE_RE.test(plate)) { skipped.push(d.code); continue; }
    ops.push({ updateOne: { filter: { _id: d._id, vehicle_plate: { $exists: false } },
                            update: { $set: { vehicle_plate: plate, schema_version: 2 } } } });
  }
  if (ops.length) try { db.drivers.bulkWrite(ops, { ordered: false }); }
                  catch (e) { e.writeErrors.forEach((w) =&gt; skipped.push(batch[w.index]?.code)); }
}</code></pre>
<p>Điểm chính: phân trang theo <code>_id</code> (không dùng <code>skip</code>), bộ lọc <code>$exists: false</code> ở cả truy vấn lấy lô và trong từng <code>updateOne</code> (app v2 có thể vừa ghi biển số), kiểm định dạng trước khi gửi để giảm lỗi 121, và giữ danh sách <code>skipped</code> cho người xử lý. Lưu ý chỉ số <code>w.index</code> là vị trí trong <code>ops</code>, không phải trong <code>batch</code> khi có dòng đã bị bỏ qua; bản thật nên giữ mảng mã tương ứng với <code>ops</code>. Sau cùng vẫn phải kiểm bằng <code>$nor</code> trước contract.</p></details>

**Bài 3.** Bạn bắt buộc phải chặn biển số sai định dạng của tài xế **mới** ngay hôm nay, nhưng backfill cho tài xế cũ cần hai tuần vì phải xác minh từng người. Chọn `validator`, `validationLevel`, `validationAction` cho hai tuần đó và giải thích rủi ro còn lại.

<details><summary><b>Lời giải</b></summary><p>Cách 1 (như lab): validator ở dạng expand (<code>vehicle_plate</code> có pattern nhưng chưa bắt buộc), <code>strict</code> + <code>error</code>. Biển số sai bị chặn ở mọi lần ghi, nhưng tài xế <b>mới</b> vẫn có thể được tạo không có biển số. Cách 2: validator đích (bắt buộc) với <code>moderate</code> + <code>error</code>. Insert tài xế mới thiếu hoặc sai biển số bị chặn; tài xế cũ đang vi phạm vẫn đổi trạng thái được. Rủi ro của cách 2: update lên tài xế cũ không bị kiểm gì cả, kể cả một <code>$set</code> làm sai trường khác; và không có gì báo khi số tài xế vi phạm giảm về 0, nên phải theo dõi <code>countDocuments({ $nor: [target] })</code> hằng ngày rồi chuyển sang <code>strict</code>. Cách 2 chỉ dùng được khi app v2 đã triển khai xong, vì app v1 sẽ không tạo được tài xế nào.</p></details>

**Bài 4.** Sau khi chạy rollback, bộ phận an toàn quyết định triển khai lại biển số. Làm sao lấy lại 8 biển số mà không cần nguồn từ bộ phận tuyển dụng? Viết câu lệnh.

<details><summary><b>Lời giải</b></summary><p>Chạy lại bước 1 (expand), rồi backfill từ <code>drivers_plate_backup</code> thay cho danh sách nguồn, vẫn giữ bộ lọc idempotent:</p>
<pre><code class="language-js">db.drivers_plate_backup.find().forEach((b) =&gt;
  db.drivers.updateOne({ _id: b._id, vehicle_plate: { $exists: false } },
                       { $set: { vehicle_plate: b.vehicle_plate, schema_version: 2 } }));</code></pre>
<p>Biển số trong bản sao lưu đã qua chuẩn hoá (8004 là <code>30H-135.79</code>) nên không gặp lại lỗi pattern. Sau đó kiểm <code>$nor</code> rồi contract như cũ. Đây là lý do rollback sao lưu theo <code>_id</code> thay vì xoá thẳng.</p></details>

## 13. Đọc tiếp

- [Schema Validation](https://www.mongodb.com/docs/manual/core/schema-validation/): validator, mức và hành động.
- [Specify Validation Level for Existing Documents](https://www.mongodb.com/docs/manual/core/schema-validation/specify-validation-level/): `strict` và `moderate`.
- [Choose How to Handle Invalid Documents](https://www.mongodb.com/docs/manual/core/schema-validation/handle-invalid-documents/): `error` và `warn`.
- [`collMod`](https://www.mongodb.com/docs/manual/reference/command/collMod/): đổi validator trên collection đã có.
- [`db.collection.bulkWrite()`](https://www.mongodb.com/docs/manual/reference/method/db.collection.bulkWrite/): `ordered`, xử lý lỗi.
- [`validate`](https://www.mongodb.com/docs/manual/reference/command/validate/): `nNonCompliantDocuments`.
- [Schema Versioning pattern](https://www.mongodb.com/docs/manual/data-modeling/design-patterns/data-versioning/schema-versioning/).
- [PG-009 · Thay đổi schema mà dữ liệu cũ vẫn đọc được](/articles/postgresql-schema-migration-tuong-thich): cùng yêu cầu với `NOT VALID` → `VALIDATE`.

Bài tiếp theo: [MG-010 · Thực hành: mô hình chuyến xe có kiểm tra ràng buộc](/articles/mongodb-thuc-hanh-mo-hinh-chuyen-xe).
