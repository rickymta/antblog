---
slug: mongodb-document-bson-id
title: "Document, BSON và _id: một chuyến xe trông thế nào"
summary: "Tổng đài Tới Nơi cần chỉ ra đúng một chuyến trong hai chuyến đồ ăn cùng đặt lúc 12:00 và đọc được mọi thông tin của nó trong một lần mở. Bài này mổ document chuyến 100004: BSON khác JSON ở đâu, _id có những luật gì, vì sao lab dùng UUID, và vì sao trường vắng khác null."
category: mongodb
level: beginner
tags: [mongodb, mg-003, document, bson, objectid, uuid, null, toi-noi]
language: vi
---

Tổng đài Tới Nơi nhận cuộc gọi: "Tôi đặt đồ ăn lúc 12 giờ trưa hôm qua, chuyến đó bị sao vậy?" Dữ liệu ngày 02/11 có **hai** chuyến đồ ăn cùng đặt đúng 12:00: chuyến 100003 ở Hà Nội đã huỷ, chuyến 100004 ở Đà Nẵng đã hoàn thành. Giờ đặt không đủ để chỉ ra chuyến nào. Khi đã biết đó là chuyến 100004, nhân viên mở nó ra và thấy cả chuyến trong **một document**: khách, tài xế, quán, cước, các mốc thời gian.

Bài này đọc kỹ document đó từng trường. Đọc được một document là kỹ năng nền cho cả series: phải biết mỗi giá trị có kiểu gì, trường nào bắt buộc, trường nào có thể vắng, và cái gì định danh document.

<div class="tkh-callout">
<span class="tkh-callout-title">MG-003 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/mongodb-la-gi-va-vai-tro-trong-toi-noi">MG-001</a>, <a href="/articles/mongodb-cai-dat-docker-dbstudio">MG-002: Dựng lab</a>. Lab: <code>examples/toi-noi-mongodb/p01</code> (<code>01-schema.js</code>, <code>02-fixture-valid.js</code>, <code>05-find.js</code>). Sau bài này, bạn đọc được một document lồng nhau, biết BSON khác JSON ở đâu, chọn được <code>_id</code> có lý do và phân biệt được trường vắng với <code>null</code>.</p>
</div>

## 1. Một chuyến nhìn dưới dạng document

Cuối `02-fixture-valid.js`, lab in chuyến 100004 bằng `printjson(db.trips.findOne({ code: 100004 }))` (trích `OUTPUT.txt`):

```text
{
  _id: UUID('70000000-0000-4000-8000-000000100004'),
  code: 100004,
  kind: 'food',
  city_id: 'DN',
  customer_id: UUID('c0000000-0000-4000-8000-000000000005'),
  driver_id: UUID('d0000000-0000-4000-8000-000000008005'),
  merchant_id: UUID('e0000000-0000-4000-8000-000000000003'),
  status: 'completed',
  fare: 55000,
  requested_at: ISODate('2026-11-02T05:00:00.000Z'),
  accepted_at: ISODate('2026-11-02T05:02:00.000Z'),
  completed_at: ISODate('2026-11-02T05:35:00.000Z'),
  updated_at: ISODate('2026-11-02T05:35:00.000Z'),
  events: []
}
```

Một **document** là một tập cặp **trường: giá trị** (*field: value*). Tên trường là chuỗi; giá trị có thể là số, chuỗi, ngày giờ, định danh nhị phân, một document con hay một mảng. Đọc từng trường:

| Trường | Giá trị | Kiểu BSON | Ý nghĩa nghiệp vụ |
|---|---|---|---|
| `_id` | `UUID('70000000-…-000000100004')` | `binData` (subtype 4) | Định danh của document, ứng dụng dùng |
| `code` | `100004` | `int` | Mã chuyến cho người đọc: khách, tổng đài, tài xế |
| `kind` | `'food'` | `string` | Loại dịch vụ: `ride`, `food` hoặc `parcel` |
| `city_id` | `'DN'` | `string` | Tham chiếu tới `cities._id` |
| `customer_id`, `driver_id`, `merchant_id` | `UUID(…)` | `binData` | Tham chiếu tới khách, tài xế, quán (không có khoá ngoại, MG-001) |
| `status` | `'completed'` | `string` | Trạng thái hiện tại, một trong 8 giá trị |
| `fare` | `55000` | `int` | Cước, số nguyên đồng |
| `requested_at` … `updated_at` | `ISODate(…)` | `date` | Các mốc thời gian, lưu theo UTC |
| `events` | `[]` | `array` | Lịch sử trạng thái nhúng trong chuyến |

Cột "Kiểu BSON" không đoán từ cách in: chạy toán tử `$type` trên chính document đó trong lab cho ra:

```js
db.trips.aggregate([{ $match: { code: 100004 } }, { $project: {
  _id: { $type: '$_id' }, code: { $type: '$code' }, kind: { $type: '$kind' },
  customer_id: { $type: '$customer_id' }, fare: { $type: '$fare' },
  requested_at: { $type: '$requested_at' }, events: { $type: '$events' } } }])
```

```text
[
  {
    _id: 'binData',
    code: 'int',
    kind: 'string',
    customer_id: 'binData',
    fare: 'int',
    requested_at: 'date',
    events: 'array'
  }
]
```

Ba chi tiết đáng chú ý:

- **Giờ hiển thị lệch 7 tiếng.** Chuyến được đặt lúc 12:00 giờ Việt Nam, nhưng `requested_at` in ra `05:00:00.000Z`. Chữ `Z` nghĩa là UTC: BSON `Date` lưu một thời điểm tuyệt đối, không kèm múi giờ. 12:00 ở Việt Nam (UTC+7) chính là 05:00 UTC. Cách lọc "chuyến ngày 02/11 theo giờ Việt Nam" là bài [MG-007](/articles/mongodb-find-loc-sap-xep-collation).
- **`events` rỗng.** 15 chuyến trong fixture được coi là dữ liệu chuyển từ hệ cũ, chỉ có trạng thái cuối. Chuyến mới tạo ở `04-lifecycle.js` mới có lịch sử đầy đủ (mục 6).
- **Không có tên khách hay tên quán.** Document chỉ giữ định danh `customer_id`, `merchant_id`; tên nằm ở collection `customers`, `merchants`. Nhúng gì, tham chiếu gì là quyết định thiết kế của [MG-008](/articles/mongodb-nhung-hay-tham-chieu).

Trong DBStudio, chạy `db.trips.find({ code: 100004 })` trong query editor rồi chọn chế độ **Tài liệu** ở khung kết quả, bạn thấy đúng các trường trên. DBStudio hiện mỗi giá trị kèm kiểu của nó theo Extended JSON: UUID thành `{"$uuid": …}`, ngày giờ thành `{"$date": "…Z"}`. Giờ hiển thị là UTC, nên 12:00 giờ Việt Nam hiện là `05:00:00.000Z`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-document-bson-id/dbstudio-01.webp" alt="DBStudio hiện document chuyến 100004 ở chế độ Tài liệu với $uuid và $date" width="1086" height="680" loading="lazy"></div><figcaption>DBStudio · chuyến 100004 ở chế độ Tài liệu</figcaption></figure>

## 2. BSON không phải JSON

Cách mongosh in document trông giống JSON, nhưng `UUID(…)` và `ISODate(…)` không phải cú pháp JSON. MongoDB lưu và truyền document dưới dạng **BSON** (*Binary JSON*): định dạng nhị phân, mỗi giá trị đi kèm một byte kiểu. JSON chỉ có sáu loại giá trị (chuỗi, số, true/false, null, object, mảng); BSON có thêm số nguyên 32 và 64 bit, `double`, `Decimal128`, `date`, `ObjectId`, `binData`, `timestamp`… và phân biệt chúng.

Sự khác nhau lộ ra ngay khi chuyển document 100004 sang JSON thuần bằng `JSON.stringify` trong mongosh:

```text
{"_id":"70000000-0000-4000-8000-000000100004","code":100004,"kind":"food","city_id":"DN","customer_id":"c0000000-0000-4000-8000-000000000005","driver_id":"d0000000-0000-4000-8000-000000008005","merchant_id":"e0000000-0000-4000-8000-000000000003","status":"completed","fare":55000,"requested_at":"2026-11-02T05:00:00.000Z","accepted_at":"2026-11-02T05:02:00.000Z","completed_at":"2026-11-02T05:35:00.000Z","updated_at":"2026-11-02T05:35:00.000Z","events":[]}
```

UUID và ngày giờ đều thành **chuỗi**. Ai nhận chuỗi này rồi ghi ngược vào MongoDB mà không chuyển kiểu sẽ được document có `customer_id` là chuỗi, và mọi truy vấn so với `UUID(…)` không còn khớp. Lab đã đo đúng bẫy này ở `05-find.js` (trích `OUTPUT.txt`):

```text
/* Sai: so UUID với chuỗi — không lỗi, 0 kết quả */
PASS customer_id = chuỗi UUID = 0
PASS customer_id = UUID(...) = 4
```

Không lỗi, chỉ ra 0. Validator của `trips` chặn được việc **ghi** chuỗi vào `customer_id` (ca T9 trong `03-fixture-invalid.js`), nhưng không chặn được việc **truy vấn** sai kiểu.

Khi cần một dạng văn bản giữ đủ kiểu, MongoDB dùng **Extended JSON**. Dạng *canonical* của cùng document (`EJSON.stringify(doc, { relaxed: false })`), chạy trên lab:

```text
{"_id":{"$binary":{"base64":"cAAAAAAAQACAAAAAABAABA==","subType":"04"}},"code":{"$numberInt":"100004"},"kind":"food","city_id":"DN","customer_id":{"$binary":{"base64":"wAAAAAAAQACAAAAAAAAABQ==","subType":"04"}},"driver_id":{"$binary":{"base64":"0AAAAAAAQACAAAAAAACABQ==","subType":"04"}},"merchant_id":{"$binary":{"base64":"4AAAAAAAQACAAAAAAAAAAw==","subType":"04"}},"status":"completed","fare":{"$numberInt":"55000"},"requested_at":{"$date":{"$numberLong":"1793595600000"}},"accepted_at":{"$date":{"$numberLong":"1793595720000"}},"completed_at":{"$date":{"$numberLong":"1793597700000"}},"updated_at":{"$date":{"$numberLong":"1793597700000"}},"events":[]}
```

Dài, nhưng không mất gì: UUID là 16 byte nhị phân subtype `04`, `55000` được đánh dấu là `$numberInt` (Int32), ngày là số mili giây kể từ 01/01/1970 UTC. Đây là dạng nên dùng khi xuất dữ liệu để nhập lại. Công cụ GUI và `mongoexport` thường cho chọn giữa dạng *relaxed* (dễ đọc hơn) và *canonical*.

| | JSON | BSON |
|---|---|---|
| Dạng | Văn bản | Nhị phân, có độ dài và byte kiểu cho từng giá trị |
| Số | Một loại "number" | `int` (32 bit), `long` (64 bit), `double`, `Decimal128` |
| Ngày giờ | Không có, thường dùng chuỗi | `date` (mili giây UTC) |
| Định danh nhị phân | Không có | `binData` (UUID là subtype 4), `ObjectId` |
| Dùng ở | API, file cấu hình | Lưu trữ và giao tiếp của MongoDB |

Kích thước thật của document 100004 khi lưu dạng BSON, đo bằng `$bsonSize`:

```text
[
  {
    code: 100004,
    bytes: 302,
    n_fields: 14
  },
  {
    code: 100014,
    bytes: 193,
    n_fields: 10
  },
  {
    code: 100015,
    bytes: 668,
    n_fields: 13
  }
]
```

302 byte cho 14 trường. Tên trường cũng được lưu **trong từng document**, nên riêng tên `requested_at` đã tốn 13 byte (12 ký tự và một byte kết thúc) ở mỗi chuyến; với 730 triệu chuyến mỗi năm theo hồ sơ Tới Nơi, tên trường dài cũng là chi phí. Chuyến 100015 nặng hơn gấp đôi vì có 7 sự kiện trong `events`. Kiểu số và ngày giờ nào dùng cho tiền, cho thời điểm là bài [MG-004](/articles/mongodb-kieu-bson-tien-thoi-gian-unicode).

## 3. `_id`: bốn luật không đổi

Mọi document trong một collection thường đều có trường **`_id`**, và MongoDB giữ bốn luật cho nó. Cả bốn được thử trong một database nháp trên lab (không đụng `toinoi`).

**Luật 1: luôn có.** Chèn document không có `_id`, MongoDB tự sinh một **ObjectId**:

```js
db.thu.insertOne({ ghi_chu: 'không ghi _id' })
```

```text
{
  acknowledged: true,
  insertedId: ObjectId('6afb0f85addf42e10623edaf')
}
```

ObjectId là giá trị 12 byte, 4 byte đầu là thời điểm tạo tính bằng giây. Lấy lại thời điểm đó bằng `getTimestamp()`:

```text
2026-11-16T14:13:25.000Z
```

Tiện để biết document được tạo lúc nào, nhưng cũng có nghĩa là ObjectId **lộ thời điểm tạo** và tăng dần theo thời gian.

**Luật 2: duy nhất trong collection.** Mỗi collection có sẵn một index unique tên `_id_` trên `_id` (bảng index của `01-schema.js` liệt kê nó ở mọi collection). Chèn trùng `_id`:

```text
11000  E11000 duplicate key error collection: scratch_a.thu index: _id_ dup key: { _id: ObjectId('6afb0f85addf42e10623edaf') }
```

Mã lỗi **11000** là lỗi trùng khoá, giống ca C8 của lab: `PASS C8 _id trùng khách 1 → lỗi 11000 (_id_)`.

**Luật 3: không đổi được.** Một khi document đã có `_id`, không update nào sửa được nó:

```js
db.thu.updateOne({ _id: id }, { $set: { _id: new ObjectId() } })
```

```text
66  Performing an update on the path '_id' would modify the immutable field '_id'
```

Mã **66** (*ImmutableField*). Muốn "đổi `_id`" chỉ có cách chèn document mới rồi xoá document cũ, và mọi document khác đang tham chiếu tới `_id` cũ sẽ thành mồ côi. Vì vậy `_id` phải là thứ **không bao giờ cần đổi**.

**Luật 4: không phải mảng.** `_id` nhận hầu hết kiểu BSON (chuỗi, số, ObjectId, UUID, cả document con), trừ mảng:

```text
53  The '_id' value cannot be of type array
```

### Lab chọn `_id` thế nào

| Collection | `_id` | Ví dụ | Vì sao |
|---|---|---|---|
| `cities` | Mã chuỗi ngắn | `'DN'` | Khoá tự nhiên ổn định, ngắn, dễ đọc; danh sách thành phố nhỏ và do Tới Nơi tự đặt mã |
| `customers`, `drivers`, `merchants`, `trips` | UUID (`binData` subtype 4) | `UUID('70000000-…-000000100004')` | Sinh được ở ứng dụng trước khi ghi, không lộ thời điểm hay số lượng, **cùng giá trị với lab PostgreSQL/SQL Server** để đối chiếu ba engine |
| `counters` | Tên bộ đếm | `'trip_code'` | Mỗi bộ đếm là một document, gọi bằng tên |

Chuyến còn có `code` kiểu `int` với index unique `uq_trips_code`. `_id` là định danh cho máy; `code` là định danh cho người, đọc được qua điện thoại. Đây là cùng ý tưởng `id` uuid + `code` của [PG-003](/articles/postgresql-bang-dong-cot-primary-foreign-key), chỉ khác ở chỗ MongoDB không có kiểu sequence: `code` được cấp bằng collection `counters` (MG-006).

Fixture ghi sẵn UUID có đuôi trùng mã (`…100004` ↔ chuyến 100004) **chỉ để kết quả in ra dễ đọc và ổn định**. Trong hệ thật, UUID ngẫu nhiên và không liên quan gì tới `code`; đừng suy cái này từ cái kia.

Vì sao không dùng ObjectId mặc định cho mọi thứ? ObjectId hoàn toàn hợp lệ và là lựa chọn phổ biến. Lab chọn UUID vì hai lý do cụ thể: dữ liệu phải trùng với hai lab quan hệ, và định danh khách, chuyến không nên tiết lộ thời điểm tạo cho ai nhìn thấy nó. Trade-off chi tiết giữa ObjectId và UUID (kích thước, thứ tự, index) là một phần của MG-004.

## 4. Trường vắng khác `null`

Chuyến 100014 vừa được đặt, chưa có tài xế. Document của nó, đọc từ lab:

```text
{
  _id: UUID('70000000-0000-4000-8000-000000100014'),
  code: 100014,
  kind: 'ride',
  city_id: 'SG',
  customer_id: UUID('c0000000-0000-4000-8000-000000000006'),
  status: 'requested',
  fare: 48000,
  requested_at: ISODate('2026-11-03T01:29:30.000Z'),
  updated_at: ISODate('2026-11-03T01:29:30.000Z'),
  events: []
}
```

Không có `driver_id`, `merchant_id`, `accepted_at`, `completed_at`: **trường vắng** (*missing field*), không phải `driver_id: null`. Trong cùng collection, document có 10, 12, 13 hay 14 trường tuỳ trạng thái và loại dịch vụ. Đếm trên lab:

```text
[
  { _id: 10, n: 2 },
  { _id: 11, n: 1 },
  { _id: 12, n: 3 },
  { _id: 13, n: 8 },
  { _id: 14, n: 2 }
]
```

(`_id` ở đây là số trường, `n` là số chuyến; kết quả đã gộp dòng cho gọn.) Ở bảng quan hệ, chuyến chưa có tài xế vẫn có cột `driver_id`, giá trị `NULL`. Ở MongoDB, lab quy ước **không áp dụng thì không ghi**, và validator cấm `null` để quy ước đó không bị phá. Ca T12 của lab (trích `OUTPUT.txt`):

```text
PASS T12 driver_id null thay vì vắng → lỗi 121 (driver_id)
```

`driver_id` được khai báo kiểu `binData`; `null` không phải `binData` nên bị từ chối với mã **121** (vi phạm validation). Các quy tắc liên trường của `trips` cũng viết theo sự có mặt của trường: "trạng thái `requested` thì `driver_id` **không tồn tại**" (`$exists: false`).

Trong DBStudio, lấy bốn chuyến 100011–100014 bằng `find` có projection rồi xem ở chế độ **Tài liệu**: mỗi document có hình dạng riêng. 100011 (bưu kiện) và 100012 (đặt xe) có `driver_id` nhưng không có `merchant_id`; 100013 (đồ ăn đang tìm tài xế) có `merchant_id` mà chưa có `driver_id`; 100014 vừa đặt thì không có cả hai. Trường không áp dụng thì vắng mặt, không phải `null`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-document-bson-id/dbstudio-02.webp" alt="DBStudio hiện bốn document chuyến 100011 đến 100014 với các trường khác nhau: có hoặc không có driver_id, merchant_id" width="1086" height="1000" loading="lazy"></div><figcaption>DBStudio · bốn chuyến, bốn hình dạng document: trường không áp dụng thì vắng mặt</figcaption></figure>

### Vì sao phân biệt quan trọng: truy vấn `null`

Lọc `{ driver_id: null }` khớp **cả** document có `driver_id: null` **lẫn** document không có trường `driver_id`. Trong lab không có `null` nào, nên ba chuyến khớp đều là chuyến vắng trường (trích `OUTPUT.txt` của `05-find.js`):

```text
/* null và trường vắng */
code|status
----|------
100006|cancelled
100013|searching
100014|requested
(3 document)
PASS {driver_id: null} khớp cả trường vắng = 3
PASS {driver_id: {$exists: false}} = 3
```

Để thấy khác biệt, thử trên database nháp với ba document: một có `driver_id: null`, một không có trường, một có giá trị:

```text
{driver_id:null}: 2
{driver_id:{$exists:false}}: 1
{driver_id:{$type:"null"}}: 1
```

`{ driver_id: null }` gộp hai trường hợp; `$exists: false` chỉ lấy trường vắng; `$type: "null"` chỉ lấy giá trị `null` tường minh. Nếu dữ liệu có lẫn cả hai, báo cáo "chuyến chưa có tài xế" có thể đếm khác nhau tuỳ người viết truy vấn. Quy ước một cách biểu diễn (lab chọn trường vắng) và để validator giữ quy ước đó là cách tránh tận gốc. Toán tử `$ne` còn một bẫy tương tự, để dành cho MG-007.

## 5. Thứ tự trường

Trong BSON, các trường của document **có thứ tự**: thứ tự lúc ghi. Fixture của lab cố ý sắp trường theo một thứ tự đọc được (định danh, tham chiếu, trạng thái, tiền, thời gian, lịch sử), và mọi chuyến in ra theo đúng thứ tự đó. Ba hệ quả, thử trên database nháp:

**`_id` luôn được đưa lên đầu.** Chèn `{ b: 2, _id: 5, a: 1 }`, đọc lại:

```text
{
  _id: 5,
  b: 2,
  a: 1
}
```

**`$set` giữ chỗ trường đã có, thêm trường mới vào cuối.** Sau `updateOne({ _id: 5 }, { $set: { b: 20, c: 3 } })`:

```text
{
  _id: 5,
  b: 20,
  a: 1,
  c: 3
}
```

**So sánh cả document con thì thứ tự có nghĩa.** Với `_id: { city: 'SG', code: 1 }`:

```text
find object đúng thứ tự: 1
find object đảo thứ tự: 0
find theo trường con: 1
```

Lọc `{ _id: { code: 1, city: 'SG' } }` không khớp, vì so sánh bằng trên document con là so từng byte, kể cả thứ tự trường. Lọc theo từng trường con (`'_id.code': 1, '_id.city': 'SG'`) thì thứ tự không quan trọng. Đây là lý do nên tránh `_id` dạng document con nếu không thật sự cần, và lọc theo từng trường bằng ký pháp dấu chấm khi làm việc với document lồng nhau.

Ứng dụng không nên dựa vào thứ tự trường cho logic nghiệp vụ; nhưng biết thứ tự được giữ giúp bạn đọc dữ liệu nhanh hơn và hiểu vì sao hai truy vấn "trông giống nhau" ra kết quả khác nhau.

## 6. Document lồng nhau và mảng

Document 100004 có `events` rỗng; hai chỗ khác trong lab cho thấy sức mạnh thật của mô hình document.

**Menu nhúng trong quán.** Quán ở Đà Nẵng, đọc từ lab:

```text
{
  _id: UUID('e0000000-0000-4000-8000-000000000003'),
  city_id: 'DN',
  name: 'Mì Quảng Thử Nghiệm',
  created_at: ISODate('2026-07-01T01:00:00.000Z'),
  menu: [
    {
      sku: 'MQ-GA',
      name: 'Mì Quảng gà',
      price: 40000
    },
    {
      sku: 'MQ-TOM',
      name: 'Mì Quảng tôm thịt',
      price: 45000
    }
  ]
}
```

`menu` là **mảng các document con**. Mở quán là có menu, không cần đọc thêm collection nào. Validator giới hạn mỗi món phải có `sku`, `name`, `price` kiểu `int`, và menu tối đa 200 món.

**Lịch sử trạng thái nhúng trong chuyến.** Chuyến 100015 tạo ở `04-lifecycle.js` đi qua 7 trạng thái; mỗi lần đổi trạng thái là một lệnh `updateOne` vừa `$set` trạng thái mới vừa `$push` một phần tử vào `events`, nguyên tử trong một document (MG-001). Mảng đó, in thành bảng (trích `OUTPUT.txt`):

```text
/* Lịch sử trạng thái nhúng trong chuyến */
i|status|at|by
-|------|--|--
0|requested|2026-11-03 08:40:00 +07:00|customer
1|searching|2026-11-03 08:40:05 +07:00|system
2|accepted|2026-11-03 08:41:00 +07:00|driver
3|picked_up|2026-11-03 08:47:00 +07:00|driver
4|in_transit|2026-11-03 08:47:30 +07:00|driver
5|arriving|2026-11-03 09:08:00 +07:00|system
6|completed|2026-11-03 09:11:00 +07:00|driver
(7 document)
```

Lab SQL Server cần hẳn bảng `trip_events` và một transaction cho việc này.

### Giới hạn 16 MB

Nhúng có trần: **một document BSON tối đa 16 MB** (`maxBsonObjectSize: 16777216` trong `db.hello()`, MG-001). Thử chèn một document vượt trần trên database nháp:

```text
10334 MongoServerError object to insert too large. size in bytes: 16777242, max size: 16777216
```

Một chuyến 668 byte còn rất xa trần. Nguy hiểm là mảng **lớn dần không giới hạn**: nếu nhúng mọi vị trí GPS tài xế gửi về (mỗi 4 giây theo hồ sơ Tới Nơi) vào chuyến, một chuyến dài sẽ phình rất nhanh, mỗi lần ghi phải viết lại document lớn hơn. Vì vậy validator của `trips` giới hạn `events` tối đa 20 phần tử, và vị trí GPS không nằm trong chuyến. Khi nào nhúng, khi nào tách là toàn bộ bài [MG-008](/articles/mongodb-nhung-hay-tham-chieu).

## 7. So với PostgreSQL / SQL Server

| Khái niệm | PostgreSQL / SQL Server | MongoDB |
|---|---|---|
| Một bản ghi | Dòng; mọi dòng cùng danh sách cột | Document; số trường có thể khác nhau giữa các document |
| Kiểu dữ liệu | Khai báo ở cột, mọi dòng giống nhau | Gắn với **từng giá trị**; validator mới ép kiểu chung |
| Khoá chính | `PRIMARY KEY` tự chọn tên cột, đổi được bằng `UPDATE` | Luôn là `_id`, unique, **không đổi được** |
| Định danh tự sinh | `IDENTITY`, sequence, `gen_random_uuid()` | `ObjectId` do driver/server sinh khi thiếu `_id` |
| "Không có giá trị" | `NULL` | Trường vắng hoặc `null`, hai thứ khác nhau; lab chỉ dùng trường vắng |
| Thứ tự cột / trường | Không mang ý nghĩa trong kết quả | Được giữ; ảnh hưởng so sánh bằng trên document con |
| Dữ liệu con | Bảng riêng + khoá ngoại, hoặc cột `jsonb` / JSON | Document con và mảng là kiểu hạng nhất |
| Giới hạn một bản ghi | Phụ thuộc trang, giá trị lớn đẩy ra ngoài (TOAST, LOB) | 16 MB cho cả document |

## 8. Những hiểu nhầm thường gặp

- **"Document là JSON."** Document là BSON; JSON thuần làm mất kiểu UUID, ngày giờ, phân biệt int/double. Xuất để nhập lại thì dùng Extended JSON canonical.
- **"`_id` phải là ObjectId."** `_id` nhận hầu hết kiểu trừ mảng. ObjectId chỉ là mặc định khi bạn không chỉ định.
- **"Đổi `_id` bằng `$set` là được."** Không: lỗi 66. Chọn `_id` từ thứ không bao giờ thay đổi; số điện thoại khách không đủ điều kiện, vì khách đổi số được.
- **"`{ x: null }` tìm document có `x` bằng `null`."** Nó khớp cả document không có `x`. Dùng `$exists` hoặc `$type: "null"` khi cần tách.
- **"Có `$jsonSchema` rồi thì mọi document cùng hình dạng."** Validator giới hạn hình dạng, không làm chúng giống nhau: trường không bắt buộc vẫn được vắng.

## 9. Bài tập

**Bài 1.** Không chạy lệnh, liệt kê các trường mà document chuyến 100013 (đồ ăn ở Hà Nội, đang `searching`) **phải có** và **không được có**, dựa trên quy tắc trong bài và `01-schema.js`. Sau đó kiểm bằng `db.trips.findOne({ code: 100013 })`.

<details><summary><b>Lời giải</b></summary><p>Phải có: <code>_id</code>, <code>code</code>, <code>kind</code> (<code>'food'</code>), <code>city_id</code> (<code>'HN'</code>), <code>customer_id</code>, <code>merchant_id</code> (đồ ăn phải có quán), <code>status</code>, <code>fare</code>, <code>requested_at</code>, <code>updated_at</code>, <code>events</code>. Không được có: <code>driver_id</code> và <code>accepted_at</code> (đang <code>searching</code> thì chưa có tài xế), <code>completed_at</code> (chưa hoàn thành). Tổng 11 trường, khớp với nhóm "11 trường: 1 chuyến" trong bảng đếm ở mục 4.</p></details>

**Bài 2.** Một script nhập dữ liệu đọc chuyến từ một API trả JSON thuần rồi `insertOne` thẳng vào `trips`. Hai trường nào sẽ gây lỗi trước tiên, mã lỗi gì, và sửa ở đâu?

<details><summary><b>Lời giải</b></summary><p>JSON thuần biến UUID và ngày giờ thành chuỗi. Validator yêu cầu <code>_id</code>, <code>customer_id</code>… là <code>binData</code> và <code>requested_at</code>, <code>updated_at</code> là <code>date</code>, nên lệnh ghi bị từ chối với lỗi 121. Sửa ở script nhập: chuyển chuỗi UUID bằng <code>UUID('…')</code> và chuỗi thời điểm bằng <code>new Date('…')</code> trước khi ghi, hoặc nhận dữ liệu dạng Extended JSON. Không được tắt validator cho "nhanh".</p></details>

**Bài 3.** Đồng nghiệp đề xuất dùng số điện thoại làm `_id` của khách "cho khỏi cần index unique riêng". Nêu hai vấn đề.

<details><summary><b>Lời giải</b></summary><p>(1) <code>_id</code> không đổi được (lỗi 66), trong khi khách có thể đổi số điện thoại; đổi số sẽ phải chèn document mới, xoá document cũ và sửa mọi chuyến đang trỏ tới khách. (2) Số điện thoại là dữ liệu cá nhân; dùng làm định danh thì nó lan ra mọi collection tham chiếu, log và URL. Lab giữ <code>_id</code> là UUID và giữ tính duy nhất của số điện thoại bằng index <code>uq_customers_phone</code>.</p></details>

**Bài 4.** Viết hai truy vấn đếm "chuyến chưa có tài xế" cho lab: một đúng với quy ước của lab, một an toàn cả khi dữ liệu cũ lỡ có `driver_id: null`. Kết quả trên lab là bao nhiêu?

<details><summary><b>Lời giải</b></summary><p>Theo quy ước: <code>db.trips.countDocuments({ driver_id: { $exists: false } })</code>. An toàn với cả hai cách biểu diễn: <code>db.trips.countDocuments({ driver_id: null })</code>, vì nó khớp cả trường vắng lẫn <code>null</code>. Trên lab cả hai ra 3 (chuyến 100006, 100013, 100014), như <code>OUTPUT.txt</code> của <code>05-find.js</code>. Nếu hai con số khác nhau trên dữ liệu thật, đó là dấu hiệu có <code>null</code> lọt vào và cần dọn.</p></details>

## 10. Đọc tiếp

- [Documents](https://www.mongodb.com/docs/manual/core/document/): cấu trúc document, trường `_id`, ký pháp dấu chấm, thứ tự trường.
- [BSON Types](https://www.mongodb.com/docs/manual/reference/bson-types/): danh sách kiểu và tên dùng với `$type`.
- [ObjectId](https://www.mongodb.com/docs/manual/reference/method/ObjectId/): cấu trúc 12 byte và `getTimestamp()`.
- [MongoDB Extended JSON (v2)](https://www.mongodb.com/docs/manual/reference/mongodb-extended-json/): dạng canonical và relaxed.
- [Query for Null or Missing Fields](https://www.mongodb.com/docs/manual/tutorial/query-for-null-fields/): `null`, `$exists`, `$type`.
- [MongoDB Limits and Thresholds](https://www.mongodb.com/docs/manual/reference/limits/): giới hạn 16 MB và các giới hạn khác.
- [PG-003](/articles/postgresql-bang-dong-cot-primary-foreign-key): bảng, dòng, khoá và `id`/`code` trong series PostgreSQL.

Bài tiếp theo: [MG-004 · Kiểu BSON cho tiền, thời gian, chuỗi tiếng Việt và định danh](/articles/mongodb-kieu-bson-tien-thoi-gian-unicode).
