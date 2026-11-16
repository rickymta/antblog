---
slug: mongodb-schema-validation-unique-index
title: "Schema validation với $jsonSchema và index unique: để MongoDB tự từ chối chuyến sai"
summary: "MongoDB không có CREATE TABLE: thiếu validator, collection trips nhận cả cước âm, trạng thái «done» hay chuyến hoàn thành mà không có giờ hoàn thành. Bài này viết validator $jsonSchema có 5 quy tắc liên trường, đọc lỗi 121 và errInfo, giữ khoá tự nhiên bằng index unique, và chỉ ra những gì vẫn lọt qua."
category: mongodb
level: intermediate
tags: [mongodb, mg-005, mongodb-8, schema-validation, jsonschema, unique-index, errinfo, toi-noi]
language: vi
---

Bộ phận vận hành Tới Nơi viết một script nhập lại các chuyến từ hệ cũ vào MongoDB. Script chạy xong không báo lỗi gì. Một tuần sau, báo cáo doanh thu có ba chuyến cước âm, một chuyến trạng thái `done` (không nằm trong 8 trạng thái của vòng đời), một chuyến `completed` nhưng không có `completed_at`, và hai khách hàng cùng số điện thoại `0900000001`. Không dòng nào trong số đó bị từ chối, vì collection MongoDB mặc định **nhận mọi document**: không có cột, không có kiểu cột, không có `CHECK`.

[MG-004](/articles/mongodb-kieu-bson-tien-thoi-gian-unicode) đã chốt data dictionary: `fare` là số nguyên đồng kiểu `int`/`long`, thời điểm là `Date`, định danh là UUID. Bài này biến bản thiết kế đó thành luật mà **database tự kiểm** ở mọi lần ghi, dù dữ liệu đến từ API, job, script hay người sửa tay trong công cụ GUI. Có hai công cụ: **validator** cho luật trong một document, và **index unique** cho luật «không trùng» giữa các document. Cuối bài là danh sách những gì cả hai vẫn để lọt.

<div class="tkh-callout">
<span class="tkh-callout-title">MG-005 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/mongodb-kieu-bson-tien-thoi-gian-unicode">MG-004: Kiểu BSON cho tiền, thời gian, chuỗi tiếng Việt và định danh</a>. Lab: <code>examples/toi-noi-mongodb/p01</code> (<code>01-schema.js</code>, <code>02-fixture-valid.js</code>, <code>03-fixture-invalid.js</code>, kết quả trong <code>OUTPUT.txt</code>). Sau bài này, bạn viết được validator <code>$jsonSchema</code> có quy tắc liên trường, đọc được lỗi 121 và <code>errInfo</code>, giữ khoá tự nhiên bằng index unique và biết những gì validator không bảo vệ được.</p>
</div>

## 1. Validator là gì

**Schema validation** là cơ chế MongoDB kiểm từng document khi ghi. Mỗi collection có thể có một **validator**: một biểu thức lọc giống điều kiện của `find()`. Document được ghi (insert, hoặc bản kết quả sau update) phải **khớp** biểu thức đó; không khớp thì lệnh ghi bị từ chối với lỗi **121** `DocumentValidationFailure`.

Validator khai báo khi tạo collection, hoặc gắn sau bằng lệnh `collMod`:

```js
db.createCollection('trips', {
  validator: { /* biểu thức lọc */ },
  validationLevel: 'strict',   // kiểm những document nào (mục 7)
  validationAction: 'error',   // không khớp thì từ chối hay chỉ ghi log (mục 7)
});
```

Biểu thức lọc có thể dùng hai lớp, và lab dùng cả hai:

1. **`$jsonSchema`**: mô tả hình dạng document theo kiểu JSON Schema: trường nào bắt buộc, kiểu BSON gì, giá trị trong tập nào, chuỗi theo mẫu nào, số trong khoảng nào, có cho trường lạ không.
2. **Toán tử truy vấn** (`$or`, `$and`, `$exists`, `$expr`…): diễn đạt **quy tắc liên trường**, ví dụ «đồ ăn thì phải có quán», mà JSON Schema viết rất vụng.

Validator **chỉ nhìn thấy document đang được ghi**. Nó không đọc được document khác trong cùng collection (nên không kiểm được «không trùng»), không đọc được collection khác (nên không kiểm được «khách có tồn tại không»). Hai giới hạn này quyết định cấu trúc phần còn lại của bài: luật trong một document thuộc về validator; luật «không trùng» thuộc về index unique; luật liên collection thuộc về ứng dụng và các phép kiểm định kỳ.

Sau `01-schema.js`, năm collection nghiệp vụ có validator; `counters` (bộ đếm mã chuyến) thì không:

```text
name|co_validator|validationLevel|validationAction
----|------------|---------------|----------------
cities|true|(mặc định)|(mặc định)
counters|false|(mặc định)|(mặc định)
customers|true|(mặc định)|(mặc định)
drivers|true|(mặc định)|(mặc định)
merchants|true|(mặc định)|(mặc định)
trips|true|strict|error
```

Bảng đọc từ `db.getCollectionInfos()`: `options.validator`, `options.validationLevel`, `options.validationAction`. «Mặc định» nghĩa là collection không ghi rõ tuỳ chọn, và giá trị mặc định chính là `strict` và `error`. Lab ghi rõ ở `trips` để người đọc script không phải nhớ.

## 2. `$jsonSchema`: kiểu, trường bắt buộc, enum, pattern, khoảng giá trị

Validator của `customers` là ví dụ gọn nhất:

```js
const uuid = { bsonType: 'binData' };
const date = { bsonType: 'date' };

db.createCollection('customers', {
  validator: { $jsonSchema: {
    bsonType: 'object', required: ['_id', 'phone', 'full_name', 'created_at'], additionalProperties: false,
    properties: {
      _id: uuid,
      phone: { bsonType: 'string', pattern: '^0[0-9]{9}$', description: 'số di động VN 10 chữ số, lưu dạng chuỗi' },
      full_name: { bsonType: 'string', minLength: 1, maxLength: 100 },
      created_at: date,
    },
  } },
});
```

Những từ khoá lab dùng:

| Từ khoá | Ý nghĩa | Ví dụ trong lab |
|---|---|---|
| `bsonType` | Kiểu BSON (một tên hoặc một mảng) | `fare: { bsonType: ['int', 'long'] }` |
| `required` | Danh sách trường phải có mặt | `trips`: 10 trường bắt buộc |
| `properties` | Luật cho từng trường | |
| `enum` | Giá trị phải thuộc tập liệt kê | `status`: 8 trạng thái |
| `pattern` | Chuỗi khớp biểu thức chính quy | `city_id: '^[A-Z]{2}$'` |
| `minimum`, `maximum` | Khoảng giá trị số (tính cả hai đầu) | `fare`: 0 … 10.000.000 |
| `minLength`, `maxLength` | Độ dài chuỗi theo code point | `full_name`: 1 … 100 |
| `maxItems`, `items` | Số phần tử và luật cho từng phần tử mảng | `events`: tối đa 20, mỗi phần tử có `status`, `at` |
| `additionalProperties: false` | Cấm trường không liệt kê trong `properties` | mọi collection nghiệp vụ |
| `description` | Chú thích, không ảnh hưởng kiểm tra | `phone` |

Phần cốt lõi của validator `trips` (lược bớt):

```js
const STATUSES = ['requested', 'searching', 'accepted', 'picked_up', 'in_transit', 'arriving', 'completed', 'cancelled'];

{ $jsonSchema: {
  bsonType: 'object',
  required: ['_id', 'code', 'kind', 'city_id', 'customer_id', 'status', 'fare', 'requested_at', 'updated_at', 'events'],
  additionalProperties: false,
  properties: {
    _id: uuid,
    code: { bsonType: 'int', minimum: 100000 },
    kind: { enum: ['ride', 'food', 'parcel'] },
    city_id: cityId,
    customer_id: uuid, driver_id: uuid, merchant_id: uuid,
    status: { enum: STATUSES },
    // Tiền: số nguyên đồng. int hoặc long, KHÔNG double (MG-004).
    fare: { bsonType: ['int', 'long'], minimum: 0, maximum: 10000000 },
    requested_at: date, accepted_at: date, completed_at: date, updated_at: date,
    events: {
      bsonType: 'array', maxItems: 20,
      items: {
        bsonType: 'object', required: ['status', 'at'], additionalProperties: false,
        properties: { status: { enum: STATUSES }, at: date, by: { enum: ['customer', 'driver', 'system'] } },
      },
    },
  },
} }
```

Năm chi tiết dễ sai khi tự viết:

- **Liệt kê `_id` trong `properties`** khi đặt `additionalProperties: false`. `_id` cũng là một trường; quên nó thì **mọi** document đều có «trường lạ» và bị từ chối.
- **Dùng `bsonType`, không dùng `type`.** `type` theo JSON Schema chỉ biết «number», không phân biệt `int` với `double`; `bsonType` phân biệt được, và đó là điều data dictionary cần.
- **`driver_id` không nằm trong `required`** vì chuyến chưa có tài xế thì trường này vắng. Nhưng khi có mặt, nó phải là `binData`; vì vậy `driver_id: null` bị từ chối (ca T12). Validator thể hiện đúng quy ước của lab: không áp dụng thì **vắng**, không ghi `null`.
- **`enum` phân biệt hoa thường và kiểu**: `'ONLINE'` không thuộc `['offline', 'online', 'busy']` (ca D3).
- **MongoDB chỉ hỗ trợ một phần JSON Schema** (draft 4, bỏ đi một số từ khoá như `$ref`, `$schema`, `default`, `definitions`, `format`). Muốn dùng lại một khối luật, lab khai báo hằng JavaScript (`uuid`, `date`, `cityId`) rồi chèn vào nhiều chỗ, như trên.

## 3. Quy tắc liên trường R1–R5

`$jsonSchema` kiểm từng trường riêng lẻ. Luật nghiệp vụ của chuyến lại nằm **giữa các trường**: có tài xế thì phải có giờ nhận, đã hoàn thành thì phải có giờ hoàn thành. Validator của `trips` đặt `$jsonSchema` và năm quy tắc vào một `$and` ở mức ngoài cùng:

```js
validator: {
  $and: [
    { $jsonSchema: { /* mục 2 */ } },                       // điều kiện #0
    // R1: đồ ăn ⇔ có quán.                                 // điều kiện #1
    { $or: [
      { kind: 'food', merchant_id: { $exists: true } },
      { kind: { $in: ['ride', 'parcel'] }, merchant_id: { $exists: false } },
    ] },
    // R2: từ accepted tới completed phải có tài xế; requested/searching thì chưa có; cancelled tuỳ.
    { $or: [                                                 // điều kiện #2
      { status: { $in: ['requested', 'searching'] }, driver_id: { $exists: false } },
      { status: { $in: ['accepted', 'picked_up', 'in_transit', 'arriving', 'completed'] }, driver_id: { $exists: true } },
      { status: 'cancelled' },
    ] },
    // R3: có tài xế ⇔ có accepted_at.                       // điều kiện #3
    { $or: [
      { driver_id: { $exists: true }, accepted_at: { $exists: true } },
      { driver_id: { $exists: false }, accepted_at: { $exists: false } },
    ] },
    // R4: completed ⇔ có completed_at.                      // điều kiện #4
    { $or: [
      { status: 'completed', completed_at: { $exists: true } },
      { status: { $ne: 'completed' }, completed_at: { $exists: false } },
    ] },
    // R5: thứ tự thời gian — so sánh hai trường của cùng document phải dùng $expr.
    { $expr: { $and: [                                       // điều kiện #5
      { $or: [{ $eq: [{ $type: '$accepted_at' }, 'missing'] }, { $gte: ['$accepted_at', '$requested_at'] }] },
      { $or: [{ $eq: [{ $type: '$completed_at' }, 'missing'] }, { $gte: ['$completed_at', '$accepted_at'] }] },
      { $gte: ['$updated_at', '$requested_at'] },
    ] } },
  ],
},
```

Cách đọc:

- **«A ⇔ B» viết thành `$or` của hai nhánh**: (A và B) hoặc (không A và không B). R1, R3, R4 đều theo mẫu này. Đọc từng nhánh như một câu: «là đồ ăn và có quán, hoặc là xe/bưu kiện và không có quán».
- **R2 có ba nhánh** vì `cancelled` là trạng thái «tuỳ»: chuyến 100003 bị huỷ sau khi tài xế đã nhận (còn `driver_id`), chuyến 100006 bị huỷ trước khi có người nhận (không có `driver_id`). Cả hai đều hợp lệ.
- **R5 phải dùng `$expr`.** Điều kiện truy vấn thông thường so sánh một trường với **hằng số** (`{ fare: { $gte: 0 } }`). So sánh **hai trường** của cùng document (`completed_at ≥ accepted_at`) cần `$expr`, nơi viết được biểu thức aggregation với `'$tên_trường'`.
- **Trong `$expr`, trường vắng không «tự đúng».** `$gte` so một Date với giá trị vắng vẫn trả về một kết quả theo thứ tự kiểu BSON chứ không bỏ qua; vì vậy R5 hỏi trước `{ $type: '$accepted_at' }` có phải `'missing'` không. Luật «chưa nhận thì không cần so» phải viết ra tường minh.
- **Thứ tự trong `$and` có nghĩa**: chỉ số 0 là `$jsonSchema`, chỉ số 1…5 trùng số quy tắc R1…R5. Lỗi 121 báo chỉ số điều kiện không thoả (mục 5), nên đặt thứ tự có chủ đích giúp đọc lỗi nhanh hơn nhiều.

Một quy tắc một điều kiện là nguyên tắc giống `CHECK` ở PostgreSQL: gộp R3 và R4 vào một `$or` lớn vẫn chạy, nhưng khi bị vi phạm thì không ai biết luật nào sai.

## 4. Chạy 03-fixture-invalid.js: mỗi ca sai, một mã lỗi

`03-fixture-invalid.js` dựng một document mẫu hợp lệ, ghi thử rồi xoá để chứng minh nó hợp lệ thật, sau đó mỗi ca sửa **đúng một chỗ**:

```js
const goodTrip = () => ({
  _id: UUID(), code: 199999, kind: 'ride', city_id: 'SG', customer_id: C(1), status: 'requested',
  fare: 45000, requested_at: vn('2026-11-03 08:40'), updated_at: vn('2026-11-03 08:40'), events: [],
});

expectError('T1 fare âm', () => db.trips.insertOne({ ...goodTrip(), fare: -1000 }), 121, 'minimum');
expectError('R4 completed thiếu completed_at', () => db.trips.insertOne({ ...goodTrip(),
  status: 'completed', driver_id: D(8003), accepted_at: vn('2026-11-03 08:41') }), 121, 4);
```

Hàm `expectError` của `lib/check.js` chỉ báo PASS khi lỗi có đúng mã mong đợi; tham số cuối là chuỗi phải xuất hiện trong chi tiết lỗi, hoặc là **chỉ số điều kiện `$and` duy nhất** không thoả. Ca sai «đúng lỗi nhưng sai lý do» vì thế không lọt qua. Một phần kết quả trong `OUTPUT.txt`:

```text
/* trips: kiểu và giá trị */
PASS T1 fare âm → lỗi 121 (minimum)
PASS T2 fare có phần lẻ 45000.5 → lỗi 121 (bsonType)
PASS T3 fare là chuỗi "45000" → lỗi 121 (bsonType)
PASS T4 fare là Decimal128 → lỗi 121 (bsonType)
PASS T5 kind taxi → lỗi 121 (enum)
PASS T6 status done → lỗi 121 (enum)
PASS T7 code dưới 100000 → lỗi 121 (minimum)
PASS T8 code trùng 100004 → lỗi 11000 (uq_trips_code)
PASS T9 customer_id là chuỗi UUID → lỗi 121 (customer_id)
PASS T10 thiếu events → lỗi 121 (events)
PASS T11 event có status lạ → lỗi 121 (enum)
PASS T12 driver_id null thay vì vắng → lỗi 121 (driver_id)

/* trips: quy tắc liên trường */
PASS R1a food không có quán → lỗi 121 (điều kiện #1)
PASS R1b ride có quán → lỗi 121 (điều kiện #1)
PASS R2a requested mà đã có tài xế → lỗi 121 (điều kiện #2)
PASS R2b accepted mà chưa có tài xế → lỗi 121 (điều kiện #2)
PASS R3 có tài xế, thiếu accepted_at → lỗi 121 (điều kiện #3)
PASS R4 completed thiếu completed_at → lỗi 121 (điều kiện #4)
PASS R5 hoàn thành trước khi nhận → lỗi 121 (điều kiện #5)
```

Cả file có 43 PASS, gồm 8 ca cho `customers` (thiếu `phone`, `phone` 9 chữ số, `phone` là số, trùng số điện thoại, tên rỗng, `created_at` là chuỗi, trường lạ `email`, trùng `_id`) và 5 ca cho `drivers`. Bảng cuối file xác nhận dữ liệu không đổi sau mọi ca sai: vẫn 3 thành phố, 6 khách, 6 tài xế, 3 quán, 15 chuyến.

Đáng chú ý là ca **R2a**: chuyến `requested` có `driver_id` **và** `accepted_at`. Nếu chỉ có `driver_id`, document vi phạm cả R2 lẫn R3; ca được dựng để chỉ R2 sai. Viết fixture sai cũng cần kỷ luật như viết luật: mỗi ca phải chứng minh đúng một luật hoạt động.

## 5. Đọc lỗi 121 và `errInfo`

Lỗi 121 có thông báo ngắn và không nói gì: `Document failed validation`. Thông tin thật nằm ở **`errInfo`**, có từ MongoDB 5.0. Ghi chuyến `completed` thiếu `completed_at` (ca R4) trên một database nháp có cùng validator, mongosh trả về:

```text
MongoServerError: Document failed validation
```

```js
{
  failingDocumentId: UUID('3fdc2d3a-c6a0-4816-a5b6-11b2aa661c99'),
  details: {
    operatorName: '$and',
    clausesNotSatisfied: [ {
      index: 4,
      details: {
        operatorName: '$or',
        clausesNotSatisfied: [
          { index: 0, details: { operatorName: '$and', clausesNotSatisfied: [ { index: 1, details: {
              operatorName: '$exists', specifiedAs: { completed_at: { '$exists': true } },
              reason: 'path does not exist' } } ] } },
          { index: 1, details: { operatorName: '$and', clausesNotSatisfied: [ { index: 0, details: {
              operatorName: '$ne', specifiedAs: { status: { '$ne': 'completed' } },
              reason: 'comparison succeeded', consideredValue: 'completed' } } ] } },
        ],
      },
    } ],
  },
}
```

Đọc từ ngoài vào trong:

1. `failingDocumentId`: `_id` của document bị từ chối. Với `insertMany` nhiều document, đây là cách biết document nào sai.
2. `details.operatorName: '$and'`, `clausesNotSatisfied: [{ index: 4 }]`: trong `$and` gốc, **chỉ điều kiện #4** không thoả, tức R4. `$jsonSchema` (#0) và R1, R2, R3, R5 đều đạt.
3. Bên trong R4 là `$or` có hai nhánh, cả hai đều sai: nhánh 0 sai vì `completed_at` không tồn tại (`path does not exist`); nhánh 1 sai vì `status` **bằng** `'completed'` nên `$ne` không đạt (`consideredValue: 'completed'`).

Lab tóm phần này thành ba dòng trong `OUTPUT.txt`:

```text
/* Chi tiết lỗi validation (errInfo.details) của R4 */
PASS R4 lần nữa, đọc chi tiết → lỗi 121
operatorName gốc: $and
các điều kiện $and không thoả: #4
```

Khi lỗi nằm trong `$jsonSchema`, chi tiết có thêm một tầng **`schemaRulesNotSatisfied`** chỉ đúng trường và từ khoá. Ca T2 (`fare: 45000.5`) trả về:

```js
details: {
  operatorName: '$and',
  clausesNotSatisfied: [ { index: 0, details: {
    operatorName: '$jsonSchema',
    schemaRulesNotSatisfied: [ {
      operatorName: 'properties',
      propertiesNotSatisfied: [ { propertyName: 'fare', details: [ {
        operatorName: 'bsonType', specifiedAs: { bsonType: ['int', 'long'] },
        reason: 'type did not match', consideredValue: 45000.5, consideredType: 'double',
      } ] } ],
    } ],
  } } ],
}
```

Câu trả lời đầy đủ nằm ngay trong lỗi: điều kiện #0, trường `fare`, luật `bsonType: ['int', 'long']`, giá trị nhận được `45000.5` kiểu `double`. Trong DBStudio 0.1.7, chạy câu `insertOne` của ca T2 trong query editor: lệnh bị từ chối và khung kết quả chỉ hiện `Document failed validation (code 121)`. DBStudio chưa hiển thị `errInfo`, nên muốn biết quy tắc nào bị vi phạm thì chạy lại trong mongosh như trên. DBStudio cũng chỉ hiểu một tập con cú pháp mongosh: `UUID()` không tham số bị báo `UUID: expects a string`, vì vậy câu lệnh trong ảnh ghi sẵn UUID cho `_id`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-schema-validation-unique-index/dbstudio-01.webp" alt="DBStudio chạy insertOne có fare 45000.5, lỗi Document failed validation code 121" width="1086" height="540" loading="lazy"></div><figcaption>DBStudio 0.1.7 · lỗi 121, chưa kèm <code>errInfo</code></figcaption></figure>

Ở ứng dụng, `errInfo` là dữ liệu cho **log và người vận hành**, không phải thông báo cho khách. Dịch vụ đặt chuyến nên ghi `errInfo` vào log có cấu trúc, trả mã lỗi nghiệp vụ ngắn cho client, và coi mọi lỗi 121 là **lỗi lập trình** (ứng dụng đã gửi dữ liệu mà chính nó lẽ ra phải kiểm trước), không phải lỗi người dùng nhập sai.

## 6. Update cũng bị kiểm

Validator không chỉ chạy lúc `insert`. Với `validationLevel: 'strict'`, mọi update được kiểm trên **document kết quả sau khi áp dụng thay đổi**, không phải trên riêng các trường bị sửa:

```text
/* update cũng bị kiểm */
PASS U1 $inc làm fare âm (chuyến 100014) → lỗi 121 (minimum)
PASS U2 $set completed cho chuyến đang chạy 100010 → lỗi 121 (điều kiện #4)
PASS U3 $set trùng phone khách 2 cho khách 3 → lỗi 11000 (uq_customers_phone)
PASS U4 $unset trường bắt buộc full_name → lỗi 121 (full_name)
```

- **U1**: `$inc: { fare: -100000 }` trên chuyến 100014 (cước 48.000 đ) cho ra −52.000. Lệnh chỉ chạm một trường, nhưng giá trị kết quả vi phạm `minimum: 0`.
- **U2**: `$set: { status: 'completed' }` cho chuyến 100010 đang `in_transit`. Riêng `status` vẫn hợp lệ theo `enum`; document kết quả thì vi phạm R4 vì không có `completed_at`. Muốn hoàn thành chuyến phải `$set` cả `status` lẫn `completed_at` trong **cùng một** lệnh update, và lệnh đó là nguyên tử trên một document ([MG-006](/articles/mongodb-insert-update-delete)).
- **U3**: update cũng bị index unique kiểm, nên trả 11000 thay vì 121.
- **U4**: `$unset` trường bắt buộc bị từ chối như một insert thiếu trường.

Bảng cuối file xác nhận chuyến 100014 vẫn có `fare` 48000 và chuyến 100010 vẫn `in_transit`: update bị từ chối không để lại thay đổi nào.

Cần phân biệt **bị từ chối** với **không khớp**. `updateOne({ code: 100010, status: 'arriving' }, …)` trên chuyến đang `in_transit` không báo lỗi; nó khớp 0 document và không ghi gì. Lỗi 121 nghĩa là có document khớp nhưng kết quả sau update không hợp lệ. MG-006 dùng cả hai cơ chế: điều kiện lọc giữ thứ tự chuyển trạng thái, validator giữ hình dạng document.

## 7. `validationLevel` và `validationAction`

Hai tuỳ chọn đi kèm validator quyết định **kiểm document nào** và **làm gì khi không khớp**:

| Tuỳ chọn | Giá trị | Ý nghĩa |
|---|---|---|
| `validationLevel` | `strict` (mặc định) | Kiểm mọi insert và mọi update |
| | `moderate` | Kiểm insert và update của document **đang hợp lệ**; document vốn đã không hợp lệ thì update không bị kiểm |
| | `off` | Không kiểm |
| `validationAction` | `error` (mặc định) | Từ chối lệnh ghi, lỗi 121 |
| | `warn` | Vẫn ghi, chỉ để lại cảnh báo trong log của server |

Lab chạy `strict` + `error` ở mọi nơi, trừ một bước trong migration. `moderate` và `warn` là công cụ cho lúc **đổi schema trên dữ liệu đã có**: gắn validator mới mà không làm hỏng ngay các lệnh update vào document cũ chưa kịp sửa, hoặc đo xem bao nhiêu document sẽ vi phạm trước khi bật thật. [MG-009](/articles/mongodb-schema-migration-tuong-thich) dùng `collMod` đổi validator của `drivers` qua các bước expand, backfill, contract, chuyển `validationAction` sang `warn` để đọc log cảnh báo (id 20294, `Document would fail validation`), rồi bật lại `error`.

Hai điều cần nhớ ngay từ bây giờ:

- Gắn validator mới bằng `collMod` **không kiểm lại** document đã có. Collection có thể chứa dữ liệu vi phạm validator hiện tại mà không ai biết. Kiểm toàn bộ bằng `db.trips.find({ $nor: [validator] })` hoặc `db.trips.validate()` (lab làm việc này ở `07-checks.js`).
- `warn` không phải chế độ «an toàn hơn»: dữ liệu sai vẫn vào, chỉ có dòng log. Để `warn` lâu dài là chấp nhận dữ liệu sai.

## 8. Index unique: giữ khoá tự nhiên

Validator không kiểm được «không có khách nào khác cùng số điện thoại» vì nó không nhìn thấy document khác. Việc đó thuộc về **index unique**: index từ chối mọi lệnh ghi tạo ra hai document có cùng giá trị khoá.

Mọi collection có sẵn index unique `_id_` trên `_id`. Định danh của lab là UUID ([MG-004](/articles/mongodb-kieu-bson-tien-thoi-gian-unicode)), không mang nghĩa nghiệp vụ; những giá trị con người dùng để nhận ra một thực thể là **khoá tự nhiên** (*natural key*) và cần index unique riêng:

```js
// _id luôn có index unique sẵn. Các index unique còn lại giữ "khoá tự nhiên".
db.customers.createIndex({ phone: 1 }, { unique: true, name: 'uq_customers_phone' });
db.drivers.createIndex({ code: 1 }, { unique: true, name: 'uq_drivers_code' });
db.trips.createIndex({ code: 1 }, { unique: true, name: 'uq_trips_code' });
// Index phục vụ truy vấn của P01 (Phần 3 đo và chọn index bài bản).
db.trips.createIndex({ customer_id: 1, requested_at: -1 }, { name: 'ix_trips_customer_requested' });
db.trips.createIndex({ driver_id: 1, status: 1 }, { name: 'ix_trips_driver_status' });
```

Kết quả sau `01-schema.js`, 10 index kể cả `_id_`:

```text
collection|name|key|unique
----------|----|---|------
cities|_id_|{"_id":1}|true
customers|_id_|{"_id":1}|true
customers|uq_customers_phone|{"phone":1}|true
drivers|_id_|{"_id":1}|true
drivers|uq_drivers_code|{"code":1}|true
merchants|_id_|{"_id":1}|true
trips|_id_|{"_id":1}|true
trips|uq_trips_code|{"code":1}|true
trips|ix_trips_customer_requested|{"customer_id":1,"requested_at":-1}|false
trips|ix_trips_driver_status|{"driver_id":1,"status":1}|false
```

Trong DBStudio, mở `trips` › **Chỉ mục** trên cây đối tượng để thấy bốn index. Cây không đánh dấu index nào là unique; double-click một index để mở script DDL mà DBStudio sinh cho cả collection. Dòng `createIndex` của `uq_trips_code` có `"unique":true`, còn dòng đầu là `createCollection` kèm toàn bộ validator:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-schema-validation-unique-index/dbstudio-02.webp" alt="Cây đối tượng DBStudio mở trips, Chỉ mục với bốn index và script DDL có createIndex unique true cho uq_trips_code" width="1150" height="560" loading="lazy"></div><figcaption>DBStudio · index của <code>trips</code> và script DDL do DBStudio sinh</figcaption></figure>

**Đặt tên index** theo quy ước giống lab PostgreSQL và SQL Server: `uq_<collection>_<trường>` cho unique, `ix_…` cho index phục vụ truy vấn. Tên mặc định của MongoDB (`code_1`, `customer_id_1_requested_at_-1`) đủ dùng, nhưng tên có chủ đích xuất hiện trong thông báo lỗi và cho biết ngay vì sao index tồn tại.

### Lỗi 11000

Vi phạm index unique trả lỗi **11000** (*duplicate key*), không phải 121. Ca T8 ghi chuyến mới với `code: 100004` đã có:

```text
MongoServerError: E11000 duplicate key error collection: toinoi.trips index: uq_trips_code dup key: { code: 100004 }
```

Thông báo nêu collection, **tên index** và giá trị trùng; đối tượng lỗi có thêm `keyPattern: { code: 1 }` và `keyValue: { code: 100004 }` để ứng dụng xử lý không cần phân tích chuỗi. Dịch vụ đăng ký khách bắt lỗi 11000 trên `uq_customers_phone` và trả «số điện thoại đã được đăng ký»; đây là lỗi nghiệp vụ hợp lệ, khác với 121 là lỗi lập trình. Cách đúng là **ghi rồi bắt lỗi**, không phải «tìm trước, chưa có thì ghi»: giữa lúc tìm và lúc ghi, một request khác có thể ghi cùng số điện thoại, và chỉ index unique chặn được trường hợp đó.

### Những điều cần biết về index unique

- **Trường vắng được coi là `null`.** Trong index unique thường, chỉ **một** document được phép thiếu trường khoá; document thứ hai thiếu trường sẽ trùng «`null`». Lab tránh được vì `phone` và `code` đều bắt buộc. Khoá chỉ áp dụng cho một phần document (ví dụ «mỗi tài xế chỉ một chuyến đang chạy») cần **partial index** (MG-025).
- **Tạo index unique trên collection đã có dữ liệu trùng sẽ thất bại.** Phải tìm và xử lý bản trùng trước (gom `$group` theo khoá, lọc `n > 1`).
- **So sánh theo byte và theo kiểu**, trừ khi index có collation. `'0900000001'` và `'0900000001 '` (thêm khoảng trắng) là hai giá trị khác nhau; validator `pattern` của lab chặn khoảng trắng trước khi index phải phân xử.
- Ở cụm **sharded** (Phần 6), index unique phải có khoá shard ở đầu; đây là lý do phải nghĩ về khoá tự nhiên trước khi chọn khoá shard.

## 9. Những gì vẫn lọt qua

Phần cuối `03-fixture-invalid.js` ghi nhận các trường hợp **không có lỗi nào**, dữ liệu sai đi thẳng vào database:

```text
/* Bẫy: KHÔNG có lỗi, dữ liệu sai lọt vào */
PASS B1 chuyến của khách không tồn tại đã vào = 1
PASS B2 tài xế ở thành phố không tồn tại đã vào = 1
PASS B3 collection "trip" (thiếu s) được tự tạo = true
PASS B4 fare âm lọt qua bypass = 1
```

- **B1: không có khoá ngoại.** `customer_id` là một UUID đúng kiểu nhưng không có khách nào mang `_id` đó. MongoDB không có `FOREIGN KEY`, và validator không đọc được collection `customers`.
- **B2: đúng mẫu nhưng không tồn tại.** `city_id: 'XX'` khớp `^[A-Z]{2}$`. Đổi `pattern` thành `enum: ['SG', 'HN', 'DN']` chặn được ca này, nhưng mỗi lần mở thành phố mới lại phải sửa validator; lab chọn kiểm định kỳ.
- **B3: gõ sai tên collection.** `db.trip.insertOne(…)` (thiếu chữ «s») không báo «không tồn tại» như SQL mà **tự tạo** collection `trip` không có validator, và document hợp lệ hay sai đều nằm ở đó. Phòng bằng cách để tên collection là hằng số trong code, tạo collection tường minh trong script schema, và ở P05 cấp cho tài khoản ứng dụng vai trò không có quyền tạo collection (MG-042).
- **B4: `bypassDocumentValidation`.** Tuỳ chọn này của lệnh ghi bỏ qua validator; `insertOne({ …, fare: -5 }, { bypassDocumentValidation: true })` thành công. Nó có lý do tồn tại (khôi phục dữ liệu cũ, `mongorestore` có tuỳ chọn tương ứng), nhưng cần quyền `bypassDocumentValidation`. Lab chưa bật xác thực nên ai cũng làm được; P05 thu hồi quyền này khỏi tài khoản ứng dụng.
- **B5: luật giữa hai collection.** Tài xế 8003 có `status: 'online'` trong `drivers` nhưng lại được gán vào một chuyến đang chạy trong `trips`. Validator của mỗi collection chỉ thấy document của chính nó. Lab không ghi thử ca này; `07-checks.js` kiểm bất biến «tài xế `busy` ⇔ có đúng một chuyến đang chạy» trên dữ liệu thật.

Mọi ca bẫy được dọn ngay sau khi ghi nhận. Lưới bắt những gì validator để lọt là **`07-checks.js`**: 12 truy vấn mong đợi 0 vi phạm, gồm tham chiếu bằng `$lookup` (thay cho khoá ngoại), thành phố của quán và tài xế, trạng thái tài xế, và `validate()` trên mọi collection. Cách thiết kế các phép kiểm này là chủ đề của [MG-008](/articles/mongodb-nhung-hay-tham-chieu) và [MG-010](/articles/mongodb-thuc-hanh-mo-hinh-chuyen-xe).

## 10. So với PostgreSQL / SQL Server

Bài [PG-005](/articles/postgresql-create-table-constraints) và [MS-005](/articles/mssql-create-table-constraints) dựng cùng bộ luật bằng ràng buộc khai báo. Cơ chế khác nhau ở nhiều chỗ:

| Chủ đề | PostgreSQL | SQL Server | MongoDB |
|---|---|---|---|
| Kiểu và bắt buộc | Kiểu cột + `NOT NULL` | Kiểu cột + `NOT NULL` | `bsonType` + `required` trong `$jsonSchema` |
| Luật trên một dòng/document | `CHECK` | `CHECK` | Validator (`$jsonSchema`, `$or`, `$expr`) |
| Số luật | Nhiều `CHECK`, mỗi cái một tên | Nhiều `CHECK`, mỗi cái một tên | Một validator cho cả collection; tách luật bằng `$and` |
| Không trùng | `UNIQUE` (ràng buộc) | `UNIQUE` (ràng buộc hoặc index) | Index unique |
| Khoá ngoại | `FOREIGN KEY` | `FOREIGN KEY` | Không có; kiểm bằng `$lookup` định kỳ |
| Mã lỗi vi phạm luật | SQLSTATE 23514 (`CHECK`), 23502 (`NOT NULL`) | 547 (`CHECK`, `FOREIGN KEY`), 515 (`NULL`) | 121, chi tiết trong `errInfo` |
| Mã lỗi trùng khoá | 23505 | 2627 (ràng buộc), 2601 (index) | 11000 |
| Biết luật nào sai | Tên ràng buộc trong thông báo | Tên ràng buộc trong thông báo | Chỉ số điều kiện trong `errInfo` |
| Thêm luật, chưa kiểm dữ liệu cũ | `NOT VALID` | `WITH NOCHECK` | `collMod` (không bao giờ kiểm dữ liệu cũ); `moderate` / `warn` |
| Ghi nhầm tên bảng | Lỗi «không tồn tại» | Lỗi «không tồn tại» | Tự tạo collection mới |

Hai khác biệt đáng nhớ nhất: MongoDB **không có khoá ngoại**, và luật **không có tên**. Ở PostgreSQL, thông báo lỗi nói `ck_trips_completed_at`; ở MongoDB, bạn tự gắn ý nghĩa cho «điều kiện #4» bằng thứ tự trong `$and` và comment trong script. Đó là lý do lab giữ R1…R5 trùng chỉ số với vị trí của chúng.

## 11. Những hiểu nhầm thường gặp

- **«MongoDB là schemaless nên không kiểm được dữ liệu.»** MongoDB không *bắt buộc* schema, nhưng validator kiểm được kiểu, trường bắt buộc, tập giá trị và luật liên trường ở mọi lệnh ghi.
- **«Validator chỉ chạy khi insert.»** Với `strict`, mọi update cũng bị kiểm, và kiểm trên document kết quả (U1, U2, U4).
- **«Gắn validator xong là dữ liệu cũ đã sạch.»** `collMod` không kiểm document đã có. Phải tự tìm bằng `$nor` hoặc `validate()`.
- **«Lỗi 121 không nói gì.»** Thông báo ngắn, nhưng `errInfo.details` chỉ ra điều kiện, trường, từ khoá và giá trị vi phạm.
- **«Unique kiểm được trong validator.»** Validator không nhìn thấy document khác. Chỉ index unique chặn được trùng, kể cả khi hai request ghi cùng lúc.
- **«`city_id` đúng pattern nghĩa là thành phố có thật.»** Không có khoá ngoại; tham chiếu phải kiểm bằng ứng dụng và phép kiểm định kỳ.
- **«Chưa có collection thì ghi sẽ báo lỗi.»** MongoDB tự tạo collection mới, không validator, không index.

## 12. Bài tập

**Bài 1.** Viết thêm điều kiện R6 cho validator `trips`: chuyến `cancelled` không được có `completed_at`. R6 có cần thiết không khi đã có R4? Nếu thêm vào cuối `$and`, nó mang chỉ số bao nhiêu?

<details><summary><b>Lời giải</b></summary><p>R4 đã nói «có <code>completed_at</code> ⇔ <code>status</code> là <code>completed</code>»: nhánh thứ hai <code>{ status: { $ne: 'completed' }, completed_at: { $exists: false } }</code> đã cấm <code>completed_at</code> với mọi trạng thái khác, kể cả <code>cancelled</code>. Vì vậy R6 thừa; thêm vào chỉ làm một document sai vi phạm hai điều kiện cùng lúc, khiến <code>errInfo</code> khó đọc hơn. Nếu vẫn viết, nó là <code>{ $or: [{ status: { $ne: 'cancelled' } }, { completed_at: { $exists: false } }] }</code> ở chỉ số 6. Bài học: trước khi thêm luật, kiểm xem luật cũ đã bao trùm nó chưa.</p></details>

**Bài 2.** Một lập trình viên gắn validator cho `customers` nhưng quên `_id` trong `properties`, vẫn giữ `additionalProperties: false`. Chuyện gì xảy ra với lệnh `insertOne` tiếp theo, và `errInfo` sẽ chỉ ra điều gì?

<details><summary><b>Lời giải</b></summary><p>Mọi lệnh insert đều bị lỗi 121, kể cả document hoàn toàn đúng, vì <code>_id</code> (do mongosh hoặc driver tự thêm nếu thiếu) là trường không liệt kê trong <code>properties</code>. <code>errInfo.details</code> có <code>schemaRulesNotSatisfied</code> với <code>operatorName: 'additionalProperties'</code> và <code>additionalProperties: ['_id']</code>. Sửa: thêm <code>_id</code> với kiểu phù hợp vào <code>properties</code>. Đây cũng là lý do <code>03-fixture-invalid.js</code> ghi thử document mẫu hợp lệ trước mọi ca sai.</p></details>

**Bài 3.** Đội ứng dụng muốn cho khách đăng ký bằng email **hoặc** số điện thoại, nên `phone` không còn bắt buộc. Họ bỏ `phone` khỏi `required` và giữ nguyên `uq_customers_phone`. Khách thứ hai đăng ký chỉ bằng email gặp lỗi gì? Sửa thế nào?

<details><summary><b>Lời giải</b></summary><p>Lỗi 11000 trên <code>uq_customers_phone</code> với <code>dup key: { phone: null }</code>: index unique coi trường vắng là <code>null</code>, nên chỉ một khách được phép không có số điện thoại. Sửa: tạo lại index dưới dạng partial, chỉ đưa vào index những document có <code>phone</code> kiểu chuỗi: <code>db.customers.createIndex({ phone: 1 }, { unique: true, name: 'uq_customers_phone', partialFilterExpression: { phone: { $type: 'string' } } })</code>, rồi thêm luật «phải có <code>phone</code> hoặc <code>email</code>» vào validator bằng <code>$or</code> với hai <code>$exists</code>. Partial index được bàn kỹ ở MG-025.</p></details>

**Bài 4.** Liệt kê ba cách một chuyến có `fare` âm có thể nằm trong collection `trips` dù validator có `minimum: 0` và `validationAction: 'error'`. Với mỗi cách, nêu cách phát hiện.

<details><summary><b>Lời giải</b></summary><p>(1) Ghi bằng <code>bypassDocumentValidation: true</code> (ca B4); phát hiện bằng <code>db.trips.find({ fare: { $lt: 0 } })</code> hoặc <code>validate()</code>, và phòng bằng cách không cấp quyền bypass cho tài khoản ứng dụng. (2) Document đã có trước khi validator được gắn hoặc siết lại bằng <code>collMod</code>, vì <code>collMod</code> không kiểm dữ liệu cũ; phát hiện bằng <code>db.trips.find({ $nor: [validator] })</code> với validator đọc từ <code>getCollectionInfos()</code>. (3) Validator từng ở <code>validationLevel: 'off'</code> hoặc <code>validationAction: 'warn'</code> trong một đợt migration; phát hiện bằng cùng truy vấn <code>$nor</code> và log cảnh báo 20294. Ngoài ra, ghi nhầm vào collection <code>trip</code> (B3) không làm bẩn <code>trips</code> nhưng làm mất dữ liệu khỏi báo cáo; kiểm danh sách collection bằng <code>db.getCollectionNames()</code>.</p></details>

## 13. Đọc tiếp

- [Schema Validation](https://www.mongodb.com/docs/manual/core/schema-validation/): tổng quan, cách xem và sửa validator.
- [`$jsonSchema`](https://www.mongodb.com/docs/manual/reference/operator/query/jsonSchema/): từ khoá được hỗ trợ và những gì bị bỏ so với JSON Schema draft 4.
- [Specify Validation Level for Existing Documents](https://www.mongodb.com/docs/manual/core/schema-validation/specify-validation-level/) và [Choose How to Handle Invalid Documents](https://www.mongodb.com/docs/manual/core/schema-validation/handle-invalid-documents/): `validationLevel`, `validationAction`.
- [Bypass Schema Validation](https://www.mongodb.com/docs/manual/core/schema-validation/bypass-document-validation/): khi nào và ai được bỏ qua validator.
- [Unique Indexes](https://www.mongodb.com/docs/manual/core/index-unique/): trường vắng, partial unique index, unique trên cụm sharded.
- [PG-005](/articles/postgresql-create-table-constraints) và [MS-005](/articles/mssql-create-table-constraints): cùng bộ luật bằng ràng buộc khai báo.

Bài tiếp theo: [MG-006 · insert, update, delete và toán tử cập nhật](/articles/mongodb-insert-update-delete).
