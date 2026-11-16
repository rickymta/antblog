---
slug: mongodb-kieu-bson-tien-thoi-gian-unicode
title: "Kiểu BSON cho tiền, thời gian, chuỗi tiếng Việt và định danh"
summary: "Kế toán Tới Nơi cần tổng cước ngày 02/11, CSKH cần tìm đúng khách «Nguyễn Văn An». Trên MongoDB, mỗi giá trị mang kiểu BSON riêng: cước lưu double cộng ra số lẻ, Date viết thiếu múi giờ lệch 7 tiếng, UUID lưu thành chuỗi thì tìm ra 0. Bài này lập data dictionary cho collection trips."
category: mongodb
level: intermediate
tags: [mongodb, mg-004, mongodb-8, bson, decimal128, date, utf-8, objectid, uuid, toi-noi]
language: vi
---

Kế toán Tới Nơi hỏi: «Tổng cước các chuyến đặt trong ngày 02/11 là bao nhiêu?». Một script nhập dữ liệu từ hệ cũ vừa chạy xong, và câu trả lời có thể sai theo ba cách mà **không có thông báo lỗi nào**: vài chuyến có cước `45000.5` lưu dạng số thực nên tổng ra số lẻ không khớp cổng thanh toán; vài chuyến có `requested_at` viết bằng `new Date('2026-11-02T07:58:00')` không ghi múi giờ, nên tuỳ máy chạy script mà thời điểm lệch 7 tiếng; còn `customer_id` của một số chuyến là chuỗi `'c0000000-…'` thay vì UUID, nên khi CSKH tìm «các chuyến của khách 1» thì những chuyến ấy biến mất khỏi kết quả.

PostgreSQL và SQL Server gắn kiểu vào **cột**: mọi dòng của cột `fare` cùng một kiểu, giá trị sai kiểu bị từ chối hoặc ép kiểu ngay lúc ghi. MongoDB gắn kiểu vào **từng giá trị** trong từng document. Hai chuyến có thể cùng có trường `fare` nhưng một chuyến là số nguyên 32 bit, chuyến kia là chuỗi. Vì vậy chọn kiểu ở MongoDB gồm hai việc: biết kiểu BSON nào đúng cho từng loại dữ liệu, và biết **công cụ nào đang thật sự tạo ra kiểu đó** (mongosh, driver của ứng dụng, script nhập liệu). Bài này làm cả hai cho tiền, thời điểm, chuỗi tiếng Việt và định danh, kèm thí nghiệm chạy thật trên MongoDB 8.0.38, rồi gom quyết định vào data dictionary của collection `trips`.

<div class="tkh-callout">
<span class="tkh-callout-title">MG-004 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/mongodb-document-bson-id">MG-003: Document, BSON và <code>_id</code></a>. Lab: <code>examples/toi-noi-mongodb/p01</code> (<code>01-schema.js</code>, <code>02-fixture-valid.js</code>, <code>03-fixture-invalid.js</code>, <code>05-find.js</code>, thí nghiệm <code>thu-nghiem/mg004-kieu-bson.js</code>). Sau bài này, bạn chọn được kiểu BSON cho tiền, thời điểm, chuỗi và định danh, biết mongosh và driver sinh ra kiểu nào, và lập được data dictionary có đơn vị, múi giờ và nghĩa của trường vắng.</p>
</div>

## 1. Kiểu nằm ở giá trị, không nằm ở collection

**BSON** (*Binary JSON*) là định dạng nhị phân MongoDB dùng để lưu document và truyền qua mạng ([MG-003](/articles/mongodb-document-bson-id)). Khác JSON chỉ có «số», «chuỗi», «true/false», BSON phân biệt nhiều kiểu hơn, và mỗi giá trị được ghi kèm một byte cho biết kiểu của nó. Những kiểu Tới Nơi dùng:

| Kiểu BSON | Tên trong `$type` | Kích thước giá trị | Tới Nơi dùng cho |
|---|---|---|---|
| 32-bit integer | `int` | 4 byte | cước, giá món, mã chuyến, mã tài xế |
| 64-bit integer | `long` | 8 byte | tổng tiền lớn, bộ đếm |
| Double | `double` | 8 byte | toạ độ GPS (Phần 3), **không** dùng cho tiền |
| Decimal128 | `decimal` | 16 byte | tỷ lệ, tiền có phần lẻ (mục 3) |
| String | `string` | độ dài UTF-8 + 5 byte | tên, số điện thoại, trạng thái |
| Date | `date` | 8 byte | mọi thời điểm |
| Binary data | `binData` | độ dài + 5 byte | UUID (subtype 4) |
| ObjectId | `objectId` | 12 byte | `_id` mặc định khi không chỉ định |
| Array, Object | `array`, `object` | tuỳ nội dung | `events`, `menu` |

Vì kiểu đi theo giá trị, câu hỏi «trường này kiểu gì» chỉ trả lời được bằng cách hỏi chính dữ liệu. Toán tử aggregation **`$type`** trả về tên kiểu BSON của một biểu thức. Chạy trên chuyến 100004 trong database `toinoi` sau khi đã chạy lab P01:

```js
db.trips.aggregate([
  { $match: { code: 100004 } },
  { $project: {
    _id: { $type: '$_id' }, code: { $type: '$code' }, kind: { $type: '$kind' },
    fare: { $type: '$fare' }, requested_at: { $type: '$requested_at' },
    customer_id: { $type: '$customer_id' }, driver_id: { $type: '$driver_id' },
    events: { $type: '$events' },
  } },
])
```

```text
_id|code|kind|fare|requested_at|customer_id|driver_id|events
---|----|----|----|------------|-----------|---------|------
binData|int|string|int|date|binData|binData|array
```

Trong DBStudio, dán câu lệnh trên vào query editor của kết nối `toinoi` và chạy; khung kết quả ở chế độ **Tài liệu** cho thấy mỗi trường ứng với một tên kiểu:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-kieu-bson-tien-thoi-gian-unicode/dbstudio-01.webp" alt="DBStudio chạy aggregation $type trên chuyến 100004, kết quả binData, int, string, int, date, binData, binData, array" width="1086" height="560" loading="lazy"></div><figcaption>DBStudio · kiểu BSON của từng trường trong chuyến 100004</figcaption></figure>

Câu hỏi hữu ích hơn là hỏi trên **toàn collection**: trường `fare` có bao nhiêu kiểu khác nhau? Gom theo `$type` là cách nhanh nhất để phát hiện dữ liệu nhập từ nguồn lạ:

```js
db.trips.aggregate([
  { $group: { _id: { $type: '$fare' }, so_chuyen: { $sum: 1 }, tong: { $sum: '$fare' } } },
])
```

```text
_id|so_chuyen|tong
---|---------|----
int|16|881000
```

Sau khi chạy đủ `00` → `07`, cả 16 chuyến có `fare` kiểu `int`. Kết quả có nhiều hơn một dòng (ví dụ thêm `double` hoặc `string`) là dấu hiệu cần điều tra trước khi tin bất kỳ con số tổng nào. Lab giữ được một dòng nhờ validator chặn sai kiểu ngay lúc ghi; [MG-005](/articles/mongodb-schema-validation-unique-index) viết validator đó. Bài này trả lời câu hỏi trước nó: kiểu nào là đúng.

## 2. Số: int, long, double, Decimal128 — và ai quyết định kiểu

BSON có bốn kiểu số:

| Kiểu | Miền giá trị | Chính xác? | Ghi chú |
|---|---|---|---|
| `int` | −2.147.483.648 … 2.147.483.647 | Có, với số nguyên | Đủ cho một cước (tối đa ~2,1 tỷ đ) |
| `long` | khoảng ±9,2 × 10¹⁸ | Có, với số nguyên | Tổng tiền, bộ đếm |
| `double` | khoảng ±1,8 × 10³⁰⁸, 15–17 chữ số có nghĩa | **Không** với số thập phân | Dấu phẩy động nhị phân IEEE 754 |
| `decimal` | 34 chữ số thập phân có nghĩa | Có, với số thập phân | Decimal128, IEEE 754-2008 |

Câu hỏi khó hơn là **ai chọn kiểu** khi bạn gõ `fare: 45000`. JavaScript chỉ có một kiểu số (double), nên mongosh và driver Node.js phải tự quyết định khi chuyển sang BSON. Thí nghiệm `thu-nghiem/mg004-kieu-bson.js` ghi sáu giá trị rồi hỏi lại kiểu:

```js
c.insertOne({ _id: 1, nho: 100000, lon: 2 ** 31, le: 45000.5,
              long: Long('3000000000'), int_ep: Int32(7), dec: Decimal128('0.1') });
c.aggregate([{ $project: { _id: 0, nho: { $type: '$nho' }, lon: { $type: '$lon' }, le: { $type: '$le' },
                           long: { $type: '$long' }, int_ep: { $type: '$int_ep' }, dec: { $type: '$dec' } } }]);
```

```text
nho|lon|le|long|int_ep|dec
---|---|--|----|------|---
int|double|double|long|int|decimal
```

Quy tắc của mongosh: số nguyên **vừa 32 bit** thành `int`; số nguyên lớn hơn **không** tự thành `long` mà thành `double`; số có phần lẻ thành `double`. Muốn `long` hay `decimal` phải viết rõ `Long('…')`, `Decimal128('…')`. Một hệ quả bất ngờ: `45000.0` trong JavaScript chính là số 45000, nên mongosh lưu nó thành `int`. Muốn lưu đúng một `double` có giá trị nguyên phải viết `Double(45000)`.

Driver của từng ngôn ngữ có quy tắc riêng: Node.js giống mongosh; Python gửi `int` thành int 32 hoặc 64 bit tuỳ độ lớn, `float` thành double; Java, C# gửi theo kiểu khai báo trong class (`int`, `long`, `double`, `decimal`). Vì vậy **cùng một trường có thể nhận kiểu khác nhau tuỳ đường ghi**: API viết bằng C# với `long Fare` ghi `long`, script vận hành trong mongosh ghi `int`, công cụ nhập CSV ghi `double`. Validator của lab chấp nhận `['int', 'long']` cho `fare` chính vì lý do này, và từ chối mọi kiểu khác. Cách cấu hình mapping kiểu trong driver là chủ đề của MG-056.

Hai chữ số nguyên khác kiểu vẫn so sánh được với nhau: `int` 45000 và `long` 45000 là cùng một giá trị khi lọc và sắp xếp (mục 8). Khác biệt chỉ nằm ở cách lưu và ở điều mà ứng dụng nhận lại khi đọc.

## 3. Tiền: số nguyên đồng, không phải double

Đồng Việt Nam không có đơn vị nhỏ hơn được dùng trong thanh toán, nên Tới Nơi lưu tiền là **số nguyên đồng**: `fare: 55000` nghĩa là 55.000 đ. Quy ước này giống lab PostgreSQL và SQL Server, và ghi rõ trong comment của validator:

```js
// Tiền: số nguyên đồng. int hoặc long, KHÔNG double (MG-004).
fare: { bsonType: ['int', 'long'], minimum: 0, maximum: 10000000 },
```

### Vì sao không double

`double` lưu số theo cơ số 2. Phần lẻ như 0,1 không biểu diễn chính xác được bằng cơ số 2, giống như 1/3 không viết chính xác được bằng số thập phân. Thí nghiệm cộng hai giá trị 0,1 và 0,2 bằng `$sum`, một lần với `double`, một lần với `Decimal128`:

```text
kieu|tong|bang_0_3
----|----|--------
decimal|0.3|true
double|0.30000000000000004|false
```

Với đồng Việt Nam, sai số này không xuất hiện khi mọi giá trị là số nguyên nhỏ: `double` biểu diễn chính xác mọi số nguyên tới 2⁵³. Vấn đề nằm ở chỗ khác: một cột tiền kiểu `double` **cho phép** giá trị `45000.5`, và một khi phần lẻ đã vào được thì các phép chia, phần trăm khuyến mãi, phí dịch vụ sẽ tạo ra số lẻ không ai làm tròn. Kiểu `int`/`long` biến quy ước «số nguyên đồng» thành điều database kiểm được: ca **T2** của `03-fixture-invalid.js` ghi `fare: 45000.5` và bị từ chối với lỗi 121 vì sai `bsonType`.

### `$sum` tự nâng lên long

Một cước vừa với `int`, nhưng tổng doanh thu một tháng thì không chắc. MongoDB không để tổng tràn số: khi cộng các `int` vượt quá 32 bit, `$sum` tự trả về `long`:

```js
c.insertMany([{ _id: 'big1', g: 2000000000 }, { _id: 'big2', g: 2000000000 }]);
c.aggregate([{ $group: { _id: null, s: { $sum: '$g' } } }, { $project: { s: 1, t: { $type: '$s' } } }]);
```

```text
tong|kieu
----|----
4000000000|long
```

Khi cộng lẫn kiểu, kết quả lấy kiểu «rộng» nhất: có một `double` thì tổng là `double`, có một `decimal` thì tổng là `decimal`. Đây là lý do thêm để giữ cước một kiểu thống nhất: một giá trị `double` lọt vào là đủ biến tổng doanh thu thành số thực. Nếu ứng dụng **lưu** một tổng (ví dụ bảng tổng hợp doanh thu theo ngày ở MG-017), hãy ghi rõ `Long(…)` hoặc dùng kiểu `long` của driver, đừng để giá trị đi qua `Number` của JavaScript rồi thành `double` khi vượt 32 bit.

### Khi nào Decimal128 là đúng

Validator của lab từ chối cả `Decimal128('45000')` (ca **T4**): cước là số nguyên đồng, cho phép thêm một kiểu chỉ làm truy vấn và mapping phức tạp hơn. Nhưng Decimal128 là lựa chọn đúng trong vài trường hợp:

- **Tiền có phần lẻ**: Tới Nơi mở rộng sang thị trường dùng USD, cước 3,75 USD. Có hai cách đúng: lưu số nguyên theo đơn vị nhỏ nhất (`375` cent, kiểu `int`/`long`) hoặc lưu `Decimal128('3.75')` kèm trường `currency`. Cách đầu đơn giản và nhanh; cách sau tiện khi đơn vị nhỏ nhất thay đổi theo đồng tiền.
- **Tỷ lệ và đơn giá**: tỷ giá, thuế suất, phần trăm hoa hồng của tài xế, đơn giá theo km như 4.250,5 đ/km. Kết quả cuối (số tiền phải trả) vẫn làm tròn về số nguyên đồng theo một quy tắc ghi rõ, rồi mới lưu vào `fare`.
- **Số đo cần đúng tới chữ số thập phân cuối**, khi so sánh bằng nhau là quan trọng.

Decimal128 tốn 16 byte, chậm hơn khi tính toán, và nhiều ngôn ngữ cần thư viện riêng để giữ đủ độ chính xác ở phía ứng dụng (JavaScript không có kiểu thập phân sẵn). Đừng dùng nó «cho chắc» khi số nguyên đã đủ.

## 4. Thời điểm: BSON Date luôn là UTC

Kiểu **Date** của BSON là số mili giây kể từ 1970-01-01 00:00:00 UTC, lưu trong 8 byte. Date **không chứa múi giờ**: nó chỉ là một khoảnh khắc trên trục thời gian. Hiển thị `05:00Z` hay `12:00 +07:00` là việc của công cụ đọc. Chuyến 100004 đặt lúc 12:00 trưa giờ Việt Nam ngày 02/11, và `printjson` của mongosh in nó theo UTC:

```text
requested_at: ISODate('2026-11-02T05:00:00.000Z'),
```

DBStudio và hàm `table()` của lab in giờ Việt Nam; giá trị lưu trong database là một.

### Bẫy: chuỗi ngày giờ không ghi múi giờ

Cái nguy hiểm không nằm ở lúc đọc mà ở lúc **tạo** giá trị Date. Thí nghiệm viết cùng một thời điểm «07:58 sáng 02/11» theo ba cách:

```js
const a = new Date('2026-11-02T07:58:00');       // không múi giờ
const b = new Date('2026-11-02T07:58:00+07:00'); // rõ ràng
const z = ISODate('2026-11-02T07:58:00');        // ISODate không múi giờ
```

```text
cach_viet|utc
---------|---
new Date('…07:58:00')|2026-11-02T00:58:00.000Z
new Date('…07:58:00+07:00')|2026-11-02T00:58:00.000Z
ISODate('…07:58:00')|2026-11-02T07:58:00.000Z
```

- `new Date('…07:58:00')` không có múi giờ được hiểu theo **giờ địa phương của tiến trình mongosh**. Container lab đặt `TZ=Asia/Ho_Chi_Minh` nên kết quả trùng cách viết rõ ràng. Chạy cùng script trên máy chủ đặt `TZ=UTC` (mặc định của phần lớn ảnh Docker và máy chủ cloud), kết quả sẽ là `07:58Z`: **cùng một dòng code, dữ liệu khác nhau tuỳ máy chạy**.
- `ISODate('…07:58:00')` không có múi giờ luôn được hiểu là **UTC**, nên lệch 7 tiếng so với ý định «07:58 giờ Việt Nam».

Quy tắc của lab: **mọi chuỗi thời điểm đều ghi rõ múi giờ**. `lib/check.js` có hàm `vn()` nhận giờ Việt Nam và luôn gắn `+07:00`:

```js
globalThis.vn = (s) => new Date(s.replace(' ', 'T') + (s.length === 16 ? ':00' : '') + '+07:00');
```

Ở ứng dụng, quy tắc tương ứng là: lấy thời điểm hiện tại từ đồng hồ hệ thống dưới dạng UTC, nhận thời điểm từ client theo ISO 8601 **có** offset (`2026-11-02T07:58:00+07:00` hoặc `…Z`), và từ chối chuỗi không có offset ở tầng API.

### Mili giây, không phải micro giây

Date chỉ giữ tới mili giây. Phần nhỏ hơn bị cắt khi tạo giá trị:

```js
new Date('2026-11-02T07:58:00.123456+07:00').getUTCMilliseconds()   // 123
```

Với Tới Nơi, mili giây là đủ. Nếu nguồn dữ liệu (ví dụ PostgreSQL `timestamptz` có micro giây) được đồng bộ sang MongoDB, hai bên sẽ lệch dưới 1 ms; so sánh bằng nhau giữa hai hệ thống phải tính đến điều này (MG-057).

### Hai thứ trông giống Date nhưng không phải

- **Chuỗi ngày giờ** như `'2026-11-03T08:00:00+07:00'`: so sánh theo thứ tự chữ, không theo thời gian; `$dateTrunc` và chỉ mục theo thời gian không dùng được. Ca **C6** của lab ghi `created_at` dạng chuỗi và bị validator từ chối.
- Kiểu BSON **Timestamp**: tên gần giống nhưng là kiểu nội bộ MongoDB dùng cho oplog (Phần 6). Ứng dụng không dùng nó để lưu thời điểm nghiệp vụ.

## 5. Ngày nghiệp vụ theo giờ Việt Nam

«Ngày 02/11» là khái niệm theo giờ Việt Nam: từ 00:00 +07:00 ngày 02/11 tới trước 00:00 +07:00 ngày 03/11, tức từ 17:00Z ngày 01/11 tới trước 17:00Z ngày 02/11. Mọi toán tử ngày giờ của aggregation **mặc định tính theo UTC**, và nhận tham số `timezone` để đổi:

```js
c.insertOne({ _id: 'time', at: new Date('2026-11-02T07:58:00+07:00') });
c.aggregate([{ $match: { _id: 'time' } }, { $project: { _id: 0,
  utc:     { $dateToString: { date: '$at', format: '%Y-%m-%d %H:%M' } },
  gio_vn:  { $dateToString: { date: '$at', format: '%Y-%m-%d %H:%M', timezone: 'Asia/Ho_Chi_Minh' } },
  ngay_vn: { $dateTrunc: { date: '$at', unit: 'day', timezone: 'Asia/Ho_Chi_Minh' } },
} }]);
```

```text
utc|gio_vn|ngay_vn
---|------|-------
2026-11-02 00:58|2026-11-02 07:58|2026-11-02 00:00:00 +07:00
```

`$dateTrunc` với `timezone` trả về **nửa đêm giờ Việt Nam** (giá trị lưu là `2026-11-01T17:00:00Z`), đúng mốc để gom chuyến theo ngày. Thiếu `timezone`, chuyến 100008 khách đặt lúc 00:10 sáng 03/11 giờ Việt Nam (17:10Z ngày 02/11) bị tính vào ngày 02/11. `05-find.js` của lab đếm đúng chênh lệch này: lọc theo ngày UTC ra 9 chuyến cho «ngày 02/11», lọc theo khoảng nửa mở giờ Việt Nam ra 8. Cách lọc và gom theo ngày được trình bày đầy đủ ở [MG-007](/articles/mongodb-find-loc-sap-xep-collation) và MG-016.

Tới Nơi chạy ở ba thành phố cùng múi giờ, nhưng lab vẫn ghi múi giờ vào dữ liệu: collection `cities` có trường `tz` (validator chỉ cho phép `'Asia/Ho_Chi_Minh'`). Báo cáo đọc múi giờ từ thành phố thay vì viết cứng `+07:00` trong pipeline; khi mở thêm thị trường, chỉ dữ liệu thay đổi. Không lưu múi giờ hay offset vào từng Date: Date là khoảnh khắc, múi giờ là thuộc tính của **nơi** diễn ra sự việc.

## 6. Chuỗi tiếng Việt: UTF-8, NFC và collation

Chuỗi BSON luôn là **UTF-8**. Khác SQL Server (phải chọn `nvarchar` và tiền tố `N'…'`), MongoDB không có chuỗi «không Unicode» để dùng nhầm: chữ «ễ» đi vào thế nào thì ra thế ấy. Nhưng UTF-8 mở ra ba câu hỏi khác.

### Độ dài: ký tự hay byte

Mỗi chữ cái tiếng Việt có dấu chiếm 2–3 byte trong UTF-8. MongoDB có hai toán tử đo độ dài:

```js
c.insertOne({ _id: 'str', s: 'Đà Nẵng', s2: 'Đà Nẵng'.normalize('NFD') });
c.aggregate([{ $match: { _id: 'str' } }, { $project: { _id: 0,
  cp: { $strLenCP: '$s' }, bytes: { $strLenBytes: '$s' },
  nfd_cp: { $strLenCP: '$s2' }, bang_nhau: { $eq: ['$s', '$s2'] } } }]);
```

```text
cp|bytes|nfd_cp|bang_nhau
--|-----|------|---------
7|11|10|false
```

`$strLenCP` đếm **code point** (điểm mã Unicode, gần với «ký tự» người đọc thấy): «Đà Nẵng» có 7. `$strLenBytes` đếm byte: 11. Giới hạn của MongoDB tính theo byte (document tối đa 16 MB, khoá index có giới hạn kích thước), còn giới hạn nghiệp vụ như «họ tên tối đa 100 ký tự» nên tính theo code point. Từ khoá `maxLength` trong `$jsonSchema` đếm theo code point, nên `full_name: { maxLength: 100 }` của lab nghĩa là 100 ký tự, không phải 100 byte.

### NFC và NFD: hai chuỗi nhìn giống nhau

Unicode cho phép viết «ẵ» theo hai cách: một code point dựng sẵn (dạng **NFC**), hoặc chữ «a» cộng các dấu kết hợp riêng (dạng **NFD**). Trên màn hình hai dạng giống hệt nhau, nhưng thí nghiệm cho thấy dạng NFD của «Đà Nẵng» dài 10 code point và **không bằng** dạng NFC. Hệ quả: khách gõ tên trên một bàn phím sinh NFD sẽ không tìm thấy bằng chuỗi NFC, index unique coi hai dạng là hai giá trị khác nhau, và `maxLength` đếm ra số khác. Thử nhanh trên database nháp với validator `maxLength: 7`: dạng NFC của «Đà Nẵng» được nhận, dạng NFD bị từ chối với lỗi 121.

MongoDB **không chuẩn hoá** chuỗi khi ghi. Việc đó thuộc ứng dụng: chuẩn hoá mọi chuỗi người dùng nhập về NFC ở tầng API trước khi ghi và trước khi tìm.

### Sắp xếp và tìm kiếm: collation

Mặc định MongoDB so sánh chuỗi theo **byte**. Với tiếng Việt, thứ tự byte sai ở chữ «Đ» (code point U+0110, xếp sau mọi chữ cái Latin cơ bản). `05-find.js` sắp xếp sáu khách theo tên:

```text
so byte (mặc định): Lê Hoàng Cường · Nguyễn Văn An · Phạm Thu Dung · Trần Thị Bình · Võ Minh Đức · Đặng Ngọc Hà
collation vi      : Đặng Ngọc Hà · Lê Hoàng Cường · Nguyễn Văn An · Phạm Thu Dung · Trần Thị Bình · Võ Minh Đức
```

**Collation** là bộ quy tắc so sánh chuỗi theo ngôn ngữ, chỉ định ở mức truy vấn, index hoặc collection. Với `locale: 'vi'`, «Đ» đứng sau «D» như từ điển tiếng Việt. Tham số `strength: 1` bỏ qua khác biệt cấp hai trở lên (dấu, hoa thường), nhưng «bỏ qua dấu» theo locale tiếng Việt khác với điều nhiều người mong đợi:

```text
truy_van|khong_collation|vi_1|en_1
--------|---------------|----|----
nguyen van an|0|0|1
nguyên văn an|0|1|1
NGUYỄN VĂN AN|0|1|1
dang ngoc ha|0|0|1
```

Với `vi` strength 1, **dấu thanh** (huyền, sắc, hỏi, ngã, nặng) và hoa thường được bỏ qua, nhưng ă, â, ê, ô, ơ, ư, đ là **chữ cái riêng** trong bảng chữ cái tiếng Việt. Vì thế «nguyên văn an» khớp «Nguyễn Văn An» còn «nguyen van an» thì không. Locale `en` strength 1 coi mọi dấu là dấu phụ và khớp cả dạng không dấu. Ô tìm kiếm «gõ không dấu» của ứng dụng cần một trong hai: collation `en` strength 1, hoặc một trường phụ lưu dạng không dấu đã chuẩn hoá. Lựa chọn này cùng text search tiếng Việt được bàn ở MG-007 và MG-029.

### Không phải mọi chuỗi đều là chữ

Số điện thoại `0900000001` lưu dạng **chuỗi**, không phải số: số 0 đầu có nghĩa, không ai cộng hai số điện thoại, và định dạng kiểm bằng `pattern: '^0[0-9]{9}$'`. Ca **C3** ghi `phone: 900000099` (số) và bị từ chối. Trạng thái (`completed`), loại dịch vụ (`food`), mã thành phố (`SG`) là chuỗi ASCII cố định, kiểm bằng `enum` hoặc `pattern`; chỉ tên người, tên quán, tên món mới là «chữ tiếng Việt» cần nghĩ tới NFC và collation.

## 7. Định danh: ObjectId hay UUID

Mọi document có trường `_id` duy nhất trong collection ([MG-003](/articles/mongodb-document-bson-id)). Không chỉ định `_id`, mongosh và driver tự sinh một **ObjectId**.

### ObjectId

ObjectId dài **12 byte**: 4 byte đầu là thời điểm tạo (giây kể từ 1970, UTC), 5 byte ngẫu nhiên theo tiến trình, 3 byte bộ đếm tăng dần. Vì có thời điểm ở đầu, ObjectId tăng gần như theo thứ tự tạo và đọc ngược ra được lúc tạo:

```js
ObjectId('6ae93b700000000000000001').getTimestamp()
```

```text
objectid|thoi_diem|bytes
--------|---------|-----
6ae93b700000000000000001|2026-11-03 08:40:00 +07:00|12
```

Ưu điểm: nhỏ, sinh ở client không cần hỏi server, gần tăng dần nên chèn vào index `_id` hiệu quả. Nhược điểm: lộ thời điểm tạo cho ai nhìn thấy id (ví dụ id khách hàng trong URL cho biết khách đăng ký lúc nào), thứ tự chỉ «gần đúng» giữa nhiều máy sinh id, và chỉ có nghĩa trong MongoDB: PostgreSQL hay hệ thống thanh toán bên ngoài không có kiểu ObjectId.

### UUID

**UUID** là định danh 128 bit, sinh được ở bất kỳ đâu mà gần như không trùng. Trong BSON, UUID lưu bằng kiểu **Binary data subtype 4**: 16 byte. Lab dùng UUID cho khách, tài xế, quán và chuyến, với cùng giá trị như lab PostgreSQL (`uuid`) và SQL Server (`uniqueidentifier`), để đối chiếu dữ liệu ba engine và để dịch vụ đặt chuyến tạo id trước khi ghi vào bất kỳ hệ thống nào.

Điều quan trọng là lưu UUID dưới dạng **binary**, không phải chuỗi 36 ký tự. Thí nghiệm lưu cùng một UUID theo hai cách:

```js
c.insertOne({ _id: 'ids', u: UUID('c0000000-0000-4000-8000-000000000001'),
                          s: 'c0000000-0000-4000-8000-000000000001' });
c.aggregate([{ $match: { _id: 'ids' } }, { $project: { _id: 0,
  u: { $type: '$u' }, u_bytes: { $binarySize: '$u' }, s: { $type: '$s' }, s_bytes: { $strLenBytes: '$s' } } }]);
```

```text
u|u_bytes|s|s_bytes
-|-------|-|-------
binData|16|string|36
```

Dạng chuỗi tốn hơn gấp đôi, ở cả document lẫn mọi index chứa nó. Tệ hơn, hai dạng **không bao giờ bằng nhau**: `db.lab_types.countDocuments({ u: 'c0000000-0000-4000-8000-000000000001' })` trả về **0**, không có lỗi nào. Một dịch vụ lưu `customer_id` dạng chuỗi và một dịch vụ khác tìm bằng `UUID(…)` sẽ không bao giờ thấy dữ liệu của nhau. Ca **T9** của lab ghi `customer_id` dạng chuỗi UUID và bị validator từ chối (`customer_id: { bsonType: 'binData' }`).

Một bẫy cũ cần biết khi gặp dữ liệu từ hệ thống đời trước: một số driver C#, Java, Python phiên bản cũ lưu UUID bằng **subtype 3** với thứ tự byte riêng của từng ngôn ngữ. Cùng một UUID ghi bằng hai driver cấu hình khác nhau có thể ra hai giá trị binary khác nhau. Driver hiện nay cho cấu hình `uuidRepresentation=standard` (subtype 4); hãy đặt nó tường minh trong chuỗi kết nối hoặc cấu hình client của mọi dịch vụ.

### Lab chọn gì

| Collection | `_id` | Lý do |
|---|---|---|
| `cities` | chuỗi mã `'SG'`, `'HN'`, `'DN'` | Danh mục nhỏ, mã tự nhiên ổn định, người đọc hiểu ngay |
| `customers`, `drivers`, `merchants`, `trips` | UUID (binData subtype 4) | Sinh ở ứng dụng, dùng chung với hệ thống khác, không lộ thời điểm |
| `counters` | chuỗi `'trip_code'` | Mỗi bộ đếm một document |

Chuyến còn có `code` kiểu `int` tăng dần (100000, 100001…) để khách và CSKH đọc qua điện thoại. MongoDB không có `IDENTITY` hay sequence; lab cấp mã bằng collection `counters` và `findOneAndUpdate` với `$inc` ([MG-006](/articles/mongodb-insert-update-delete)). ObjectId vẫn là lựa chọn tốt cho dữ liệu chỉ sống trong MongoDB như log hoặc sự kiện tạm: nhỏ hơn UUID và có sẵn thứ tự gần đúng theo thời gian.

## 8. So sánh khác kiểu: không lỗi, chỉ không khớp

Khi kiểu trong một trường không thống nhất, truy vấn **không báo lỗi**. Kết quả sai lặng lẽ. Thí nghiệm ghi bốn chuyến nháp với `fare` 45000 ở bốn dạng:

```js
c.insertMany([{ _id: 'f1', fare: 45000 }, { _id: 'f2', fare: '45000' },
              { _id: 'f3', fare: Long('45000') }, { _id: 'f4', fare: 45000.0 }]);
c.countDocuments({ fare: 45000 })            // 3
c.countDocuments({ fare: { $gt: 0 } })       // 3
c.countDocuments({ fare: { $type: 'string' } })  // 1
```

Bốn kiểu số (`int`, `long`, `double`, `decimal`) được so sánh theo **giá trị**: `{ fare: 45000 }` khớp mọi dạng số của 45000. Thử thêm trên database nháp với `Double(45000)` và `Decimal128('45000')`, cả năm dạng số đều khớp. Chuỗi `'45000'` thì không, vì MongoDB không ép chuỗi thành số.

Điều kiện `{ fare: { $gt: 0 } }` cũng bỏ qua chuỗi. MongoDB áp dụng **type bracketing**: toán tử so sánh chỉ so với giá trị cùng nhóm kiểu với toán hạng. Toán hạng là số, nên chỉ document có `fare` là số mới được xét. Một chuyến có `fare: '45000'` không lọt vào báo cáo «chuyến có thu tiền», không lọt vào `$sum` (chuỗi bị bỏ qua khi cộng), và không ai nhận được thông báo gì.

Khi **sắp xếp** một trường có nhiều kiểu, MongoDB dùng thứ tự kiểu BSON cố định: `null` trước, rồi các số, rồi chuỗi, rồi object, mảng, binary, ObjectId, boolean, Date… Một chuyến có `fare` là chuỗi luôn đứng sau mọi chuyến có `fare` là số, bất kể giá trị. Ba điều trên giải thích vì sao lab dựa vào validator: kiểm kiểu lúc ghi rẻ hơn rất nhiều so với đi tìm lỗi trong báo cáo.

## 9. Data dictionary cho collection trips

**Data dictionary** (từ điển dữ liệu) mô tả từng trường: kiểu, đơn vị, miền giá trị, và ở MongoDB thêm một cột quan trọng: **trường vắng nghĩa là gì**. Lab không ghi `null`: trường không áp dụng thì không có mặt trong document, và validator từ chối `null` (ca **T12** ghi `driver_id: null` bị lỗi 121, vì `null` không phải `binData`).

| Trường | Kiểu BSON | Đơn vị / miền | Vắng nghĩa là | Ghi chú |
|---|---|---|---|---|
| `_id` | `binData` subtype 4 | UUID sinh ở ứng dụng | không được vắng | index `_id_` có sẵn |
| `code` | `int` | ≥ 100000, tăng dần | không được vắng | cấp từ `counters`; index `uq_trips_code` |
| `kind` | `string` | `ride` / `food` / `parcel` | không được vắng | ASCII |
| `city_id` | `string` | 2 chữ hoa, `_id` trong `cities` | không được vắng | không có khoá ngoại (MG-005) |
| `customer_id` | `binData` subtype 4 | `_id` trong `customers` | không được vắng | |
| `driver_id` | `binData` subtype 4 | `_id` trong `drivers` | chưa có tài xế nhận | |
| `merchant_id` | `binData` subtype 4 | `_id` trong `merchants` | không áp dụng (không phải `food`) | |
| `status` | `string` | 8 trạng thái vòng đời | không được vắng | ASCII |
| `fare` | `int` hoặc `long` | **đồng**, 0 … 10.000.000 | không được vắng | 0 đ = chuyến miễn phí (100008); không `double`, không `decimal` |
| `requested_at` | `date` | khoảnh khắc khách đặt, UTC | không được vắng | ngày nghiệp vụ theo `cities.tz` |
| `accepted_at` | `date` | khoảnh khắc tài xế nhận | chưa ai nhận | ≥ `requested_at` |
| `completed_at` | `date` | khoảnh khắc hoàn thành | chưa hoàn thành | chỉ có khi `completed` |
| `updated_at` | `date` | lần sửa gần nhất | không được vắng | |
| `events` | `array` of `object` | `{ status, at: date, by }`, tối đa 20 phần tử | không được vắng (có thể rỗng) | lịch sử nhúng (MG-006, MG-008) |

Collection `customers` thêm hai dòng đáng chú ý: `phone` kiểu `string` theo mẫu `^0[0-9]{9}$`; `full_name` kiểu `string` tối đa 100 code point, chuẩn hoá NFC ở ứng dụng, sắp xếp bằng collation `vi`. Giá món trong `merchants.menu[].price` theo cùng quy ước với `fare`.

Bảng này là bản thiết kế; validator của [MG-005](/articles/mongodb-schema-validation-unique-index) biến nó thành điều database kiểm được. Hai thứ cần giữ khớp nhau: khi thêm trường mới, sửa cả hai.

## 10. So với PostgreSQL / SQL Server

Bài [PG-004](/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid) và [MS-004](/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode) trả lời cùng câu hỏi trên hai engine quan hệ. Những chỗ cơ chế khác thật sự:

| Chủ đề | PostgreSQL | SQL Server | MongoDB |
|---|---|---|---|
| Kiểu gắn với | Cột | Cột | Từng giá trị; validator giới hạn thêm |
| Ghi sai kiểu | Lỗi hoặc ép kiểu | Lỗi hoặc ép kiểu | Ghi được nếu không có validator |
| Tiền | `bigint` đồng | `int`/`bigint` đồng | `int`/`long` đồng; `Decimal128` khi có phần lẻ |
| Tổng tràn số | `sum(int)` trả `bigint` | Phải `CAST` sang `bigint` | `$sum` tự nâng lên `long` |
| Thời điểm | `timestamptz` (micro giây) | `datetimeoffset(3)` | `Date` (mili giây, UTC, không offset) |
| Ngày theo giờ VN | `AT TIME ZONE` | `AT TIME ZONE` | tham số `timezone` của toán tử aggregation |
| Chuỗi Unicode | `text` (UTF-8 theo database) | `nvarchar` + `N'…'` | Luôn UTF-8 |
| Collation | Theo cột / database | Theo cột / database | Theo truy vấn, index hoặc collection |
| UUID | `uuid` | `uniqueidentifier` | `binData` subtype 4 |
| Số tăng dần | identity / sequence | `IDENTITY` | Tự làm bằng `counters` + `$inc` |
| Không áp dụng | `NULL` | `NULL` | Trường vắng; `null` là một giá trị khác |

Khác biệt lớn nhất là dòng thứ hai. Ở PostgreSQL, kiểu cột là hàng rào có sẵn; ở MongoDB, hàng rào chỉ tồn tại khi bạn dựng nó bằng validator.

## 11. Những hiểu nhầm thường gặp

- **«MongoDB schemaless nên không cần chọn kiểu.»** Mỗi giá trị vẫn có kiểu, chỉ là không ai bắt các giá trị cùng trường phải cùng kiểu. Không chọn nghĩa là để mongosh, từng driver và từng script chọn hộ, mỗi nơi một kiểu.
- **«Gõ số lớn trong mongosh sẽ thành long.»** Số nguyên vượt 32 bit thành `double`. Muốn `long` phải viết `Long('…')`.
- **«`45000.0` là double.»** Trong mongosh, `45000.0` và `45000` là cùng một số JavaScript và được lưu thành `int`. Muốn `double` phải viết `Double(45000)`.
- **«Date lưu theo giờ máy chủ.»** Date luôn là UTC. Thứ phụ thuộc giờ máy là cách **chuỗi không có offset** được hiểu khi tạo Date bằng `new Date(…)` trong mongosh hoặc Node.js.
- **«Tìm không dấu chỉ cần collation `vi` strength 1.»** Locale `vi` bỏ qua dấu thanh nhưng giữ ă, â, ê, ô, ơ, ư, đ là chữ riêng; «nguyen van an» không khớp «Nguyễn Văn An».
- **«UUID dạng chuỗi hay binary cũng như nhau.»** Chuỗi tốn 36 byte thay vì 16, và truy vấn bằng dạng này không bao giờ khớp dữ liệu lưu bằng dạng kia, kể cả khi không có lỗi nào.
- **«Truy vấn sai kiểu sẽ báo lỗi.»** `{ fare: { $gt: 0 } }` lặng lẽ bỏ qua `fare` là chuỗi; `{ customer_id: '…' }` lặng lẽ trả 0.

## 12. Bài tập

Thí nghiệm của bài chạy bằng `npm run p01:thu-nghiem`. Lệnh này chạy `00-reset.js` trước (xoá database `toinoi`), nên sau khi thử xong hãy chạy lại `npm run p01`.

**Bài 1.** Một đồng nghiệp chạy trong mongosh: `db.daily_revenue.insertOne({ _id: '2026-11', total: 3000000000 })`, rồi một dịch vụ Java đọc `total` vào biến `long`. Trường `total` được lưu kiểu gì, có vấn đề gì không, và nên sửa câu lệnh ra sao?

<details><summary><b>Lời giải</b></summary><p>3.000.000.000 vượt giới hạn <code>int</code> (2.147.483.647), nên mongosh lưu thành <code>double</code>, không phải <code>long</code>. Giá trị này vẫn chính xác (số nguyên nhỏ hơn 2⁵³), nhưng kiểu sai: driver Java có thể từ chối đọc <code>double</code> vào trường <code>long</code> hoặc cần chuyển đổi, và các phép cộng tiếp theo trên <code>double</code> có thể sinh phần lẻ. Sửa: <code>total: Long('3000000000')</code>, hoặc tính bằng <code>$sum</code> trong pipeline rồi ghi bằng <code>$merge</code> để giữ kiểu <code>long</code>. Kiểm lại bằng <code>$type</code>.</p></details>

**Bài 2.** Viết pipeline đếm số chuyến và tổng cước theo **ngày đặt chuyến theo giờ Việt Nam**. Với dữ liệu lab, chuyến 100008 (đặt 00:10 sáng 03/11 giờ Việt Nam) phải rơi vào ngày nào? Bỏ tham số `timezone` thì kết quả đổi thế nào?

<details><summary><b>Lời giải</b></summary><p><code>db.trips.aggregate([{ $group: { _id: { $dateTrunc: { date: '$requested_at', unit: 'day', timezone: 'Asia/Ho_Chi_Minh' } }, so_chuyen: { $sum: 1 }, tong: { $sum: '$fare' } } }, { $sort: { _id: 1 } }])</code>. Chuyến 100008 rơi vào ngày 03/11 (nhóm có <code>_id</code> là nửa đêm 03/11 giờ Việt Nam, tức <code>2026-11-02T17:00:00Z</code>). Bỏ <code>timezone</code>, <code>$dateTrunc</code> cắt theo UTC: 100008 có <code>requested_at</code> là 17:10Z ngày 02/11 nên bị tính vào ngày 02/11, và nhóm 02/11 có 9 chuyến thay vì 8, giống phép đếm của <code>05-find.js</code>.</p></details>

**Bài 3.** Dịch vụ CSKH tìm chuyến bằng `db.trips.find({ customer_id: req.params.customerId })`, với `customerId` lấy từ URL dạng `c0000000-0000-4000-8000-000000000001`. Kết quả luôn rỗng dù khách có 3 chuyến. Giải thích và sửa. Nếu một phần dữ liệu cũ đã lưu `customer_id` dạng chuỗi, làm sao đếm được có bao nhiêu chuyến như vậy?

<details><summary><b>Lời giải</b></summary><p>Tham số URL là chuỗi; <code>customer_id</code> lưu dạng <code>binData</code> subtype 4. Chuỗi và binary không bao giờ bằng nhau, nên truy vấn khớp 0 mà không báo lỗi. Sửa: chuyển tham số thành UUID trước khi truy vấn (trong mongosh <code>UUID(id)</code>; trong driver dùng kiểu UUID/Binary của driver với <code>uuidRepresentation</code> chuẩn), và kiểm định dạng UUID ở tầng API để trả lỗi 400 thay vì danh sách rỗng. Đếm dữ liệu cũ: <code>db.trips.countDocuments({ customer_id: { $type: 'string' } })</code>, hoặc gom <code>{ $group: { _id: { $type: '$customer_id' }, n: { $sum: 1 } } }</code> để thấy mọi kiểu đang có.</p></details>

**Bài 4.** Tới Nơi thêm chính sách: tài xế nhận 80% cước, làm tròn xuống tới đồng; sắp tới mở thị trường Singapore tính cước bằng SGD có hai chữ số lẻ. Bổ sung data dictionary: trường nào thêm vào, kiểu gì, và vì sao?

<details><summary><b>Lời giải</b></summary><p>Tỷ lệ 80% là tham số của chính sách, không phải tiền: lưu ở collection cấu hình dưới dạng <code>Decimal128('0.80')</code> (hoặc số nguyên phần nghìn <code>800</code>) kèm thời điểm áp dụng. Tiền tài xế nhận (<code>driver_payout</code>) là số tiền thật: tính <code>floor(fare × 0.80)</code> rồi lưu <code>int</code>/<code>long</code> đồng, ghi rõ quy tắc làm tròn trong data dictionary. Với SGD, thêm trường <code>currency</code> (<code>string</code>, mã ISO 4217 như <code>VND</code>, <code>SGD</code>) và lưu cước theo đơn vị nhỏ nhất (cent, <code>int</code>/<code>long</code>), hoặc chuyển <code>fare</code> sang <code>Decimal128</code> cho mọi đồng tiền. Cách nào cũng phải cập nhật validator và mọi báo cáo cộng <code>fare</code>: không được cộng VND với SGD, nên gom theo <code>currency</code> trước khi <code>$sum</code>.</p></details>

## 13. Đọc tiếp

- [BSON Types](https://www.mongodb.com/docs/manual/reference/bson-types/): danh sách kiểu, số hiệu và tên dùng với `$type`.
- [Comparison/Sort Order](https://www.mongodb.com/docs/manual/reference/bson-type-comparison-order/): thứ tự giữa các kiểu khi so sánh và sắp xếp.
- [Model Monetary Data](https://www.mongodb.com/docs/manual/tutorial/model-monetary-data/): Decimal128 và cách lưu số nguyên theo đơn vị nhỏ nhất.
- [Date()](https://www.mongodb.com/docs/manual/reference/method/Date/) và [`$dateTrunc`](https://www.mongodb.com/docs/manual/reference/operator/aggregation/dateTrunc/): Date trong mongosh, cắt ngày theo múi giờ.
- [Collation](https://www.mongodb.com/docs/manual/reference/collation/): `locale`, `strength` và nơi chỉ định collation.
- [ObjectId](https://www.mongodb.com/docs/manual/reference/method/ObjectId/) và [UUID()](https://www.mongodb.com/docs/manual/reference/method/UUID/): cấu trúc ObjectId, tạo UUID subtype 4.
- [PG-004](/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid) và [MS-004](/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode): cùng data dictionary trên PostgreSQL và SQL Server.

Bài tiếp theo: [MG-005 · Schema validation với `$jsonSchema` và index unique](/articles/mongodb-schema-validation-unique-index).
