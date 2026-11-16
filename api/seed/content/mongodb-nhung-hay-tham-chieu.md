---
slug: mongodb-nhung-hay-tham-chieu
title: "Nhúng hay tham chiếu: mô hình document cho chuyến, quán và khách của Tới Nơi"
summary: "Nên nhúng toàn bộ lịch sử chuyến vào document khách cho một lần đọc là đủ, hay tách sự kiện ra collection riêng như bảng trip_events? Bài này chọn nhúng hay tham chiếu theo cách dữ liệu được đọc và ghi, đo giới hạn 16 MB trên lab thật và kiểm tham chiếu bằng $lookup khi không có khoá ngoại."
category: mongodb
level: advanced
tags: [mongodb, mg-008, data-modeling, embedding, reference, lookup, denormalization, toi-noi]
language: vi
---

Buổi duyệt thiết kế màn hình "Lịch sử chuyến" của app Tới Nơi có hai đề xuất trái ngược nhau. Một bạn backend nói: "MongoDB là document database, vậy nhúng hết các chuyến vào document khách. Mở app ra, đọc đúng một document là có đủ lịch sử, khỏi phải nối." Một bạn khác vừa chuyển từ đội PostgreSQL sang thì muốn làm ngược lại: tách lịch sử trạng thái ra collection `trip_events` riêng, giống bảng `trip_events` bên PostgreSQL, "cho chuẩn".

Cả hai cách đều chạy được với 16 chuyến của lab. Nhưng anh Nguyễn Văn An đặt bốn chuyến trong hai ngày; với 2 triệu chuyến mỗi ngày của Tới Nơi, document khách theo đề xuất thứ nhất sẽ lớn lên mãi, còn đề xuất thứ hai biến một lần đổi trạng thái thành hai lần ghi vào hai collection. Bài này trả lời câu hỏi mà MongoDB bắt người thiết kế trả lời ngay từ đầu: **dữ liệu nào nhúng vào document, dữ liệu nào tham chiếu sang document khác**, và lấy gì thay khoá ngoại khi đã tham chiếu.

<div class="tkh-callout">
<span class="tkh-callout-title">MG-008 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/mongodb-document-bson-id">MG-003 · Document, BSON và <code>_id</code></a>, <a href="/articles/mongodb-schema-validation-unique-index">MG-005 · Schema validation và index unique</a>, <a href="/articles/mongodb-insert-update-delete">MG-006 · insert, update, delete</a>. Lab: <code>examples/toi-noi-mongodb/p01</code> (<code>01-schema.js</code>, <code>04-lifecycle.js</code>, <code>07-checks.js</code>). Sau bài này, bạn quyết định được nhúng hay tham chiếu dựa trên cách đọc và ghi, biết khi nào một mảng sẽ phình không giới hạn, và kiểm được tham chiếu bằng <code>$lookup</code>.</p>
</div>

## 1. Câu hỏi đúng: dữ liệu được đọc và ghi thế nào?

Trong MongoDB có hai cách nối hai mẩu dữ liệu liên quan:

- **Nhúng** (*embedding*): đặt dữ liệu con ngay trong document cha, dưới dạng document con hoặc mảng. Lịch sử trạng thái nằm trong chuyến: `trips.events`.
- **Tham chiếu** (*referencing*): document này chỉ giữ định danh của document kia, nằm ở collection khác. Chuyến giữ `customer_id`, còn thông tin khách nằm trong `customers`.

Series PostgreSQL bắt đầu thiết kế từ **dữ kiện**: dữ kiện nào phụ thuộc vào khoá nào, rồi chuẩn hoá để mỗi dữ kiện nằm đúng một chỗ ([PG-008](/articles/postgresql-mo-hinh-quan-he-chuan-hoa)). Cách đọc dữ liệu tính sau, bằng JOIN và index. MongoDB đảo thứ tự ấy: thiết kế bắt đầu từ **cách ứng dụng đọc và ghi** (*access pattern*), vì đơn vị mà MongoDB đọc, ghi nguyên tử và kiểm validator là **một document**.

Vậy trước khi vẽ collection, hãy liệt kê các thao tác nóng của Tới Nơi. Số liệu tải lấy từ [hồ sơ dự án](/articles/du-an-mau-toi-noi):

| Thao tác | Tần suất (hồ sơ) | Đọc/ghi những gì cùng lúc |
|---|---|---|
| Đổi trạng thái chuyến | ≈ 560 lần/giây lúc cao điểm | Trạng thái hiện tại + một sự kiện lịch sử, phải khớp nhau |
| Theo dõi chuyến đang chạy | Phần lớn trong ≈ 30.000 lượt đọc/giây | Chuyến, trạng thái, lịch sử, tài xế |
| Xem menu quán | Cũng nằm trong ≈ 30.000 lượt đọc/giây | Quán + toàn bộ món |
| Xem lịch sử chuyến của khách | Ít hơn nhiều, có phân trang | Các chuyến gần nhất của một khách |
| Khách đổi số điện thoại, tài xế đổi tên | Hiếm | Một khách, một tài xế |
| Báo cáo doanh thu theo thành phố | Theo lô, vài lần/ngày | Nhiều chuyến, gom theo thành phố |

Bảng này đã gợi ý gần hết câu trả lời: thứ gì luôn đọc cùng nhau và phải ghi cùng nhau thì nên nằm trong một document; thứ gì được nhiều nơi dùng chung và sửa độc lập thì nên đứng riêng.

## 2. Bốn câu hỏi cho mỗi quan hệ

Với mỗi cặp "cha – con", trả lời bốn câu theo thứ tự:

1. **Có luôn đọc cùng nhau không?** Mở chuyến là cần lịch sử; mở quán là cần menu. Nếu phần con thường được đọc một mình thì nhúng chỉ làm document cha nặng thêm.
2. **Có phải ghi cùng nhau, nguyên tử không?** Một thao tác ghi lên một document là nguyên tử, kể cả khi nó sửa nhiều trường và mảng lồng bên trong. Ghi hai document thì cần transaction (Phần 4) hoặc chấp nhận một khoảng lệch.
3. **Số phần tử con có giới hạn không?** Một chuyến đi qua tối đa vài trạng thái; một khách có thể đặt chuyến suốt nhiều năm.
4. **Phần con có được dùng chung hoặc sửa độc lập không?** Một khách xuất hiện trong nhiều chuyến; đổi số điện thoại không được phép đòi sửa từng chuyến.

Câu 1 và 2 trả lời "có", câu 3 trả lời "có giới hạn" và câu 4 trả lời "không" thì nhúng. Câu 3 hoặc câu 4 nghiêng về phía ngược lại thì tham chiếu. Áp vào lab P01:

| Quan hệ | Đọc cùng? | Ghi nguyên tử cùng? | Giới hạn | Dùng chung, sửa độc lập? | Quyết định trong lab |
|---|---|---|---|---|---|
| Chuyến → sự kiện trạng thái | Có | Có: trạng thái và sự kiện phải khớp | Vài phần tử; validator chặn ở 20 | Không | **Nhúng** `trips.events` |
| Quán → món | Có | Thường sửa cả menu một lần | Vài chục món; validator chặn ở 200 | Không (món thuộc một quán) | **Nhúng** `merchants.menu` |
| Chuyến → khách | Không phải lúc nào | Không | Một khách – rất nhiều chuyến, không giới hạn | Có | **Tham chiếu** `trips.customer_id` |
| Chuyến → tài xế | Thỉnh thoảng | Không (trạng thái tài xế là document khác) | Một tài xế – rất nhiều chuyến | Có | **Tham chiếu** `trips.driver_id` |
| Chuyến → quán | Thỉnh thoảng | Không | Một quán – rất nhiều chuyến | Có | **Tham chiếu** `trips.merchant_id` |
| Mọi thứ → thành phố | Hiếm khi cần tên | Không | 3 thành phố | Có | **Tham chiếu** bằng mã ngắn `city_id` (`'SG'`, `'HN'`, `'DN'`) |

Khách, tài xế, quán và chuyến dùng UUID làm `_id` (BSON binary subtype 4, [MG-004](/articles/mongodb-kieu-bson-tien-thoi-gian-unicode)); thành phố dùng chính mã hai chữ làm `_id`. Tham chiếu chỉ là một trường có cùng kiểu với `_id` bên kia, không có gì đặc biệt hơn.

## 3. Nhúng lịch sử trạng thái vào chuyến

### Một lần ghi, một document, nguyên tử

Bên PostgreSQL, một lần đổi trạng thái là `UPDATE trips` cộng `INSERT trip_events`, hai câu phải nằm trong cùng một transaction, và bất biến "sự kiện cuối khớp trạng thái" phải kiểm bằng truy vấn riêng. Trong lab MongoDB, cả hai việc là **một** `updateOne` (trích `04-lifecycle.js`):

```js
function step(from, to, at, by, extra = {}) {
  const when = vn(at);
  return db.trips.updateOne(
    { _id: id, status: from },
    { $set: { status: to, updated_at: when, ...extra }, $push: { events: { status: to, at: when, by } } },
  );
}
```

`$set` đổi trạng thái và `$push` thêm sự kiện trên cùng một document nên hoặc cả hai xảy ra, hoặc không gì xảy ra. Bộ lọc `status: from` làm cho hai tài xế bấm nhận cùng lúc chỉ có một người khớp ([MG-006](/articles/mongodb-insert-update-delete)). Chuyến 100015 đi hết vòng đời với bảy sự kiện nằm ngay trong chuyến:

```text
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

Đọc chuyến để hiển thị màn hình theo dõi là một lần đọc theo `_id`, không cần nối.

### Giới hạn bằng validator: `maxItems: 20`

"Có giới hạn" phải là luật mà database giữ, không chỉ là một giả định trong đầu người thiết kế. Validator của `trips` ở `01-schema.js` ghi rõ:

```js
events: {
  bsonType: 'array', maxItems: 20,
  items: {
    bsonType: 'object', required: ['status', 'at'], additionalProperties: false,
    properties: { status: { enum: STATUSES }, at: date, by: { enum: ['customer', 'driver', 'system'] } },
  },
},
```

Một chuyến bình thường có 7 sự kiện; 20 là trần rộng gấp gần ba lần nhưng vẫn nhỏ. Để thấy trần này hoạt động, mình chép chuyến 100015 sang database nháp `scratch_d`, tạo collection với **đúng validator đọc từ `toinoi.trips`**, rồi `$push` từng sự kiện cho tới khi bị từ chối. Database nháp đã được xoá sau khi đo. Lần chạy dùng lab MongoDB 8.0.38 (replica set một node trong Docker Desktop):

```js
const src = db.getSiblingDB('toinoi');
const s = db.getSiblingDB('scratch_d');
const val = src.getCollectionInfos({ name: 'trips' })[0].options.validator;
s.createCollection('trips', { validator: val, validationLevel: 'strict', validationAction: 'error' });
const t15 = src.trips.findOne({ code: 100015 });
s.trips.insertOne(t15);
// $push thêm từng sự kiện 'completed' (by: 'system') cho tới khi lỗi
```

```text
maxItems: events accepted = 20 err code 121
size with 20 events: 1419
```

Sự kiện thứ 21 bị từ chối với lỗi 121, document dừng ở 20 sự kiện và 1.419 byte. Phần `errInfo.details` (rút gọn, bỏ `consideredValue` dài 21 phần tử) chỉ đúng chỗ:

```text
{"operatorName":"maxItems","specifiedAs":{"maxItems":20},"reason":"array did not match specified length", … ,"numberOfItems":21}
```

Nếu một ngày nghiệp vụ thật sự cần nhiều sự kiện hơn (ví dụ chuyến nhiều điểm dừng), đó là lúc xem lại mô hình, không phải lúc lặng lẽ nâng `maxItems` lên 10.000.

### Kích thước thực tế

Kích thước document đo bằng toán tử `$bsonSize` trên lab (đọc, không ghi):

| Document | Số sự kiện | Kích thước BSON |
|---|---|---|
| Chuyến 100014 (`requested`, chưa có tài xế) | 0 | 193 byte |
| Chuyến 100004 (đồ ăn, `completed`) | 0 | 302 byte |
| Chuyến 100015 (đi hết vòng đời) | 7 | 668 byte |
| Bản sao 100015 trong `scratch_d`, đầy trần | 20 | 1.419 byte |

`07-checks.js` in thống kê của cả collection ở cuối lab:

```text
collection|count|avgObjSize|nindexes
----------|-----|----------|--------
trips|16|283|4
```

Trung bình 283 byte vì 15 chuyến của fixture có `events` rỗng (dữ liệu chuyển từ hệ cũ chỉ có trạng thái cuối). Ngay cả ở trần 20 sự kiện, một chuyến vẫn chưa tới 1,5 KB, cách giới hạn 16 MB hơn mười nghìn lần.

## 4. Nhúng menu vào quán

Menu là trường hợp nhúng thứ hai. Người dùng mở trang quán là cần cả menu; mỗi món chỉ thuộc một quán; quán sửa giá thường sửa vài món một lúc. Validator `merchants` giới hạn `menu` tối đa 200 món, mỗi món có `sku`, `name`, `price` (int, đồng). Một quán trong lab:

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

Chạy `db.merchants.find({ city_id: 'DN' })` trong query editor của DBStudio và xem ở chế độ **Tài liệu**, bạn thấy đúng hình dạng trên: một document quán, mảng món nằm bên trong.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-nhung-hay-tham-chieu/dbstudio-01.webp" alt="DBStudio hiện quán Mì Quảng Thử Nghiệm với mảng menu nhúng hai món" width="1086" height="700" loading="lazy"></div><figcaption>DBStudio · menu nhúng trong document quán</figcaption></figure>

Vì menu nằm trong quán, các câu hỏi về món là câu hỏi trên mảng của một document, không cần collection thứ hai:

```js
db.merchants.aggregate([
  { $project: { _id: 0, name: 1, so_mon: { $size: '$menu' }, mon_re_nhat: { $min: '$menu.price' } } },
  { $sort: { name: 1 } },
]);
```

```text
{"name":"Bún Chả Ngõ Mẫu","so_mon":2,"mon_re_nhat":20000}
{"name":"Cơm Tấm Phố Lab","so_mon":2,"mon_re_nhat":45000}
{"name":"Mì Quảng Thử Nghiệm","so_mon":2,"mon_re_nhat":40000}
```

Ba quán, mỗi quán 3 trường thông tin và 2 món, mỗi document khoảng 244–252 byte.

Nhúng menu không đúng vĩnh viễn. Nó sẽ sai khi một trong bốn câu hỏi đổi câu trả lời: chuỗi cửa hàng dùng chung một menu cho hàng trăm chi nhánh (dùng chung → tham chiếu), mỗi quán có hàng nghìn mặt hàng như siêu thị (mất giới hạn), hoặc tồn kho từng món đổi theo từng giây (ghi độc lập, rất thường xuyên). Khi đó món nên thành collection riêng, có `merchant_id`. Đánh giá lại thiết kế khi số liệu đổi chính là lý do validator ghi con số 200 ra giấy trắng mực đen.

## 5. Mảng không giới hạn và giới hạn 16 MB

### Một document có trần

Mỗi document BSON tối đa **16 MB** (16 MiB = 16.777.216 byte). Đây là giới hạn cứng của server, không phải cấu hình. Nhúng thứ gì không có giới hạn tự nhiên là đặt một quả bom hẹn giờ vào document cha.

Đề xuất "nhúng mọi chuyến vào khách" là đúng loại đó. Để có con số thật thay vì ước lượng, mình làm hai thí nghiệm trong `scratch_d` (không có validator), cùng lab 8.0.38 như trên, rồi xoá database nháp:

1. Một document khách `{ _id: 1, full_name: 'Nguyễn Văn An', trips: [] }`, `$push` vào `trips` từng lô 1.000 bản sao các chuyến của lab (xoay vòng 16 chuyến thật, mỗi chuyến khoảng 290 byte).
2. Bản sao chuyến 100015, `$push` vào `events` từng lô 10.000 rồi từng lô 100 sự kiện giống nhau `{ status: 'in_transit', at, by: 'driver' }` (một sự kiện đứng riêng là 56 byte).

```text
customer embed: trips = 57000 bytes = 16512621 err 10334  BSONObj size: 16802510 (0x10062CE) is invalid. Size must be between 0 and 16793600(16MB) First element: _id: 1
free fine: events = 268007 bytes = 16773593 err 10334
```

Document khách dừng ở **57.000 chuyến** (16.512.621 byte); lô 1.000 chuyến tiếp theo đẩy nó lên 16.802.510 byte và bị từ chối với lỗi **10334** (`BSONObjectTooLarge`). Document chuyến không có validator dừng ở **268.007 sự kiện** (16.773.593 byte). Con số 16.793.600 trong thông báo lớn hơn 16 MiB một chút vì server chừa 16 KB cho dữ liệu nội bộ; giới hạn dành cho document của bạn vẫn là 16 MiB.

57.000 chuyến nghe có vẻ rộng: một khách đặt 2 chuyến mỗi ngày sẽ mất khoảng 78 năm mới chạm trần. Nhưng 16 MB chỉ là **lỗi cuối cùng**. Vấn đề đến sớm hơn nhiều:

- **Đọc khách là đọc cả lịch sử.** Màn hình hồ sơ chỉ cần tên và số điện thoại vẫn kéo theo hàng trăm KB qua mạng và vào bộ nhớ đệm, trừ khi mọi truy vấn đều nhớ dùng projection.
- **Mọi lần đổi trạng thái chuyến đều ghi vào document khách.** Với 560 lượt đổi trạng thái/giây, document của khách hay đặt trở thành điểm nóng; document chuyến nhỏ bị thay bằng document khách lớn.
- **Tài xế và quán cũng cần chuyến.** Chuyến đã nằm trong khách thì không thể đồng thời nằm trong tài xế mà không nhân bản; "chuyến đang chạy của tài xế 8001" thành truy vấn quét mảng của mọi khách.
- **Index trên trường trong mảng là multikey**: mỗi phần tử tạo một khoá index (Phần 3). Mảng dài làm index phình theo.

Quan hệ khách → chuyến vì vậy là **tham chiếu từ phía "nhiều"**: chuyến giữ `customer_id`, khách không giữ danh sách chuyến. Câu "lịch sử chuyến của khách" được phục vụ bằng index có sẵn trong `01-schema.js`:

```js
db.trips.createIndex({ customer_id: 1, requested_at: -1 }, { name: 'ix_trips_customer_requested' });
```

Đặt danh sách id chuyến vào khách (`trip_ids: [...]`) cũng không giải quyết được gì: mảng vẫn không giới hạn, lại thêm một chỗ phải ghi đồng bộ. Quy tắc dễ nhớ: **tham chiếu đặt ở phía có nhiều phần tử**, phía "một" không giữ mảng.

### Mảng có giới hạn mềm thì sao?

Có quan hệ ở giữa: thường nhỏ nhưng thỉnh thoảng lớn, như đánh giá của một quán hay vị trí GPS của một chuyến dài. Hai mẫu thiết kế hay dùng, ở đây chỉ giới thiệu:

- **Subset pattern**: nhúng một phần nhỏ hay dùng (ví dụ 10 đánh giá mới nhất trong quán), phần còn lại ở collection riêng. `$push` với `$slice: -10` giữ mảng không vượt 10 phần tử.
- **Bucket pattern**: gom nhiều phần tử theo khung (ví dụ vị trí tài xế theo từng 5 phút) vào một document "xô", mỗi xô có trần. Vị trí tài xế (≈ 6.250 bản ghi/giây theo hồ sơ) và time series collection là chủ đề của MG-048 ở Phần 5.

## 6. Tham chiếu bằng UUID và không có khoá ngoại

### Thứ MongoDB không kiểm

Tham chiếu trong MongoDB chỉ là một giá trị. Validator của `trips` kiểm `customer_id` có kiểu `binData` (`bsonType: 'binData'`), nhưng **không** kiểm có khách nào mang `_id` ấy hay không: validator chỉ nhìn đúng một document, không đọc collection khác. Lab ghi lại hai bẫy trong `03-fixture-invalid.js` và một bẫy trong `04-lifecycle.js`:

```text
PASS B1 chuyến của khách không tồn tại đã vào = 1
PASS B2 tài xế ở thành phố không tồn tại đã vào = 1
```

```text
PASS đã xoá khách = Đặng Ngọc Hà
PASS chuyến trỏ tới khách vừa xoá = 2
```

B1: chuyến trỏ tới khách `…0000ff` không tồn tại vẫn được nhận. B2: `city_id: 'XX'` đúng pattern `^[A-Z]{2}$` nhưng không có thành phố XX. Còn `deleteOne` khách 6 thành công dù khách còn hai chuyến, để lại hai chuyến **mồ côi** (*orphan*). PostgreSQL và SQL Server chặn cả ba bằng FOREIGN KEY; MongoDB không có khái niệm khoá ngoại.

Bù lại có hai lớp: ứng dụng kiểm trước khi ghi (`deleteCustomerIfUnused` trong `04` đếm chuyến trước khi xoá, vẫn còn khe hở khi hai thao tác chạy đồng thời — Phần 4), và **truy vấn kiểm tra** chạy định kỳ để phát hiện những gì lọt qua.

### Kiểm tham chiếu bằng `$lookup`

**`$lookup`** là giai đoạn aggregation nối mỗi document với các document khớp ở collection khác, đặt kết quả vào một mảng. Mảng rỗng nghĩa là bên kia không có document nào khớp, tức tham chiếu bị gãy. `07-checks.js` đóng gói ý này thành một hàm dùng lại:

```js
// Bước $lookup chỉ lấy _id của bên được tham chiếu; mảng rỗng = mồ côi.
const missing = (from, local) => [
  { $match: { [local]: { $exists: true } } },
  { $lookup: { from, localField: local, foreignField: '_id', as: 'ref', pipeline: [{ $project: { _id: 1 } }] } },
  { $match: { ref: { $size: 0 } } },
  { $project: { _id: 0, code: 1, [local]: 1 } },
];
check('K1 chuyến trỏ tới khách không tồn tại', 'trips', missing('customers', 'customer_id'));
check('K2 chuyến trỏ tới tài xế không tồn tại', 'trips', missing('drivers', 'driver_id'));
check('K3 chuyến trỏ tới quán không tồn tại', 'trips', missing('merchants', 'merchant_id'));
for (const c of ['trips', 'drivers', 'merchants']) check(`K4 ${c}.city_id không có trong cities`, c, missing('cities', 'city_id'));
```

Ba chi tiết đáng chú ý. `$match: { [local]: { $exists: true } }` bỏ qua chuyến chưa có tài xế hoặc không có quán: trường vắng là hợp lệ, không phải tham chiếu gãy. `pipeline: [{ $project: { _id: 1 } }]` chỉ lấy `_id` của bên kia, vì kiểm tồn tại không cần cả document. Và `foreignField: '_id'` luôn có index sẵn, nên mỗi lần tra là một lần tìm theo index.

Hai kiểm tra tiếp theo dùng cùng kỹ thuật cho luật nghiệp vụ mà khoá ngoại thông thường cũng không diễn đạt được: quán và tài xế phải cùng thành phố với chuyến.

```js
check('K5 quán khác thành phố với chuyến đồ ăn', 'trips', [
  { $match: { kind: 'food' } },
  { $lookup: { from: 'merchants', localField: 'merchant_id', foreignField: '_id', as: 'm' } },
  { $match: { $expr: { $ne: ['$city_id', { $first: '$m.city_id' }] } } },
  { $project: { _id: 0, code: 1, city_id: 1, merchant_city: { $first: '$m.city_id' } } },
]);
```

Kết quả trên dữ liệu lab (trích `OUTPUT.txt`):

```text
PASS K1 chuyến trỏ tới khách không tồn tại = 0
PASS K2 chuyến trỏ tới tài xế không tồn tại = 0
PASS K3 chuyến trỏ tới quán không tồn tại = 0
PASS K4 trips.city_id không có trong cities = 0
PASS K4 drivers.city_id không có trong cities = 0
PASS K4 merchants.city_id không có trong cities = 0

/* Luật nghiệp vụ liên collection */
PASS K5 quán khác thành phố với chuyến đồ ăn = 0
PASS K6 tài xế khác thành phố với chuyến = 0
```

Mỗi kiểm tra trả về **danh sách vi phạm** và mong đợi danh sách rỗng. Khi có vi phạm, `check()` in luôn bảng document vi phạm, nên người trực biết ngay chuyến nào cần xử lý.

### `$lookup` để đọc, không chỉ để kiểm

`$lookup` cũng là cách đọc dữ liệu tham chiếu khi cần hiển thị. Ví dụ danh sách chuyến hoàn thành kèm tên và số điện thoại khách (truy vấn chỉ đọc, chạy trên lab):

```js
db.trips.aggregate([
  { $match: { status: 'completed' } },
  { $lookup: { from: 'customers', localField: 'customer_id', foreignField: '_id', as: 'khach' } },
  { $project: { _id: 0, code: 1, city_id: 1, fare: 1, khach: { $first: '$khach.full_name' }, phone: { $first: '$khach.phone' } } },
  { $sort: { code: 1 } },
]);
```

```text
{"code":100000,"city_id":"SG","fare":45000,"khach":"Nguyễn Văn An","phone":"0900000001"}
{"code":100001,"city_id":"HN","fare":38000,"khach":"Lê Hoàng Cường","phone":"0900000003"}
{"code":100002,"city_id":"SG","fare":62000,"khach":"Trần Thị Bình","phone":"0900000002"}
{"code":100004,"city_id":"DN","fare":55000,"khach":"Võ Minh Đức","phone":"0900000005"}
{"code":100005,"city_id":"HN","fare":126000,"khach":"Phạm Thu Dung","phone":"0900000004"}
{"code":100007,"city_id":"SG","fare":30000,"khach":"Đặng Ngọc Hà","phone":"0900000006"}
{"code":100008,"city_id":"SG","fare":0,"khach":"Trần Thị Bình","phone":"0900000002"}
{"code":100015,"city_id":"SG","fare":41000,"khach":"Nguyễn Văn An","phone":"0900000001"}
```

Trong DBStudio, ảnh dưới chạy cùng pipeline nhưng bỏ `phone` khỏi `$project` cho vừa khung. Ở chế độ **Bảng**, mỗi dòng là một document kết quả (DBStudio 0.1.7 chưa tách trường thành cột): đủ 8 chuyến hoàn thành, tên khách lấy từ `customers`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-nhung-hay-tham-chieu/dbstudio-02.webp" alt="DBStudio chạy aggregation $lookup từ trips sang customers, 8 dòng chuyến hoàn thành kèm tên khách" width="1086" height="665" loading="lazy"></div><figcaption>DBStudio · <code>$lookup</code> chuyến → khách, 8 chuyến hoàn thành</figcaption></figure>

`$first` lấy phần tử đầu của mảng `khach`; vì `_id` là duy nhất nên mảng có tối đa một phần tử. Nối từ phía "nhiều" sang phía "một" không làm tăng số document. Chiều ngược lại (từ khách sang chuyến) thì có, và đó là chủ đề của MG-013 ở Phần 2.

Đừng biến `$lookup` thành phản xạ cho mọi màn hình. Nếu thao tác nóng nhất của hệ thống phải `$lookup` mỗi lần, đó là tín hiệu mô hình đang đi ngược cách đọc: hoặc nhúng, hoặc chép sẵn vài trường cần hiển thị (mục 7).

## 7. Snapshot hay tham chiếu: chép dữ liệu có chủ đích

### `fare` là snapshot, không phải bản sao

`trips.fare` là cước khách đã chấp nhận lúc đặt: 45.000 đ cho chuyến 100000, 0 đ cho chuyến 100008 khuyến mãi. Bảng giá đổi vào ngày mai thì cước chuyến hôm qua **không được** đổi theo. PG-008 lập luận rằng `fare` là dữ kiện của chính chuyến, không phải bản sao của bảng giá; lập luận ấy giữ nguyên trong MongoDB. Lab giữ `fare` trên chuyến và không có collection bảng giá.

Gọi đúng tên: **snapshot** là giá trị chụp tại một thời điểm và *cố ý không đổi* khi nguồn đổi. **Tham chiếu** là trỏ tới giá trị *hiện tại* của nguồn. Câu hỏi thiết kế cho mỗi trường là: khi nguồn đổi, giá trị trong document này **có phải** đổi theo hay **không được** đổi theo?

### Chép tên khách vào chuyến: một lựa chọn thiết kế

Giả sử đội CSKH muốn màn hình danh sách chuyến hiện tên khách mà không cần `$lookup`. Có hai cách chép, nghĩa khác hẳn nhau (cả hai **không có trong lab**, chỉ là phác thảo):

```js
// (a) Snapshot: tên và số điện thoại TẠI THỜI ĐIỂM ĐẶT, không bao giờ sửa.
{ code: 100000, customer_id: UUID('c000…0001'), customer_at_booking: { full_name: 'Nguyễn Văn An', phone: '0900000001' }, … }

// (b) Extended reference: chép vài trường hay đọc của khách HIỆN TẠI, phải cập nhật khi khách đổi.
{ code: 100000, customer: { _id: UUID('c000…0001'), full_name: 'Nguyễn Văn An' }, … }
```

(a) hợp với biên lai, khiếu nại: "khách lúc đặt tên gì, gọi số nào". (b) là **extended reference pattern**: giữ `_id` để tham chiếu, cộng vài trường để khỏi nối khi hiển thị. Mẫu này hợp với trường **hiếm khi đổi** và **chấp nhận lệch một lúc**, như tên quán trên danh sách đơn.

Riêng số điện thoại cần thêm một câu hỏi: đó là dữ liệu cá nhân. Mỗi bản chép là thêm một nơi phải mã hoá, phân quyền và xoá khi khách yêu cầu (Phần 5). Lab chọn không chép gì của khách vào chuyến.

### Bất thường cập nhật khi chép sai nghĩa

Nếu chọn (b) mà khách đổi tên, phải sửa mọi chuyến của khách đó:

```js
db.trips.updateMany({ 'customer._id': cid }, { $set: { 'customer.full_name': newName } });
```

Câu này không nguyên tử với lần sửa `customers`: hai document ở hai collection khác nhau. Giữa hai câu, hoặc khi câu thứ hai lỗi giữa chừng, hệ thống giữ **hai sự thật khác nhau về cùng một người**, đúng **bất thường cập nhật** (*update anomaly*) mà PG-008 dựng lại trên bảng phẳng. Khác biệt là MongoDB không có chuẩn hoá nào ngăn bạn tạo ra tình huống đó; bạn phải tự chọn và tự canh.

Lab có một bản sao suy ra như vậy: `drivers.status = 'busy'` chép lại sự thật "tài xế đang có chuyến chạy" vốn nằm trong `trips`. Lab chấp nhận nó vì lọc tài xế rảnh là thao tác nóng, và bù bằng đủ bốn điều: nguồn gốc xác định (chuyến đang chạy), đường ghi rõ ràng (bước nhận và bước hoàn thành trong `04`), truy vấn phát hiện lệch (K7), và dựng lại được từ nguồn.

```text
PASS K7 tài xế: busy ⇔ đúng một chuyến đang chạy = 0
```

| Trường chép | Nghĩa | Khi nguồn đổi | Ai canh |
|---|---|---|---|
| `trips.fare` | Snapshot cước lúc đặt | Không đổi | Validator (kiểu, ≥ 0) |
| `drivers.status` | Bản sao suy ra từ chuyến đang chạy | Phải đổi theo | K7 trong `07-checks.js` |
| `customer_at_booking` (phác thảo) | Snapshot cho biên lai | Không đổi | Không cần kiểm lệch |
| `customer.full_name` (phác thảo) | Extended reference | Phải đổi theo, chấp nhận trễ | Cần thêm một kiểm tra kiểu K7 |

## 8. Áp vào hai đề xuất ban đầu

**Nhúng mọi chuyến vào khách** trượt ở câu 3 (không giới hạn: trần thật là khoảng 57.000 chuyến theo số liệu đo ở mục 5) và câu 4 (chuyến cũng thuộc về tài xế và quán). Nó biến 560 lượt ghi/giây vào chuyến nhỏ thành 560 lượt ghi/giây vào document khách lớn.

**Tách `events` ra collection riêng như `trip_events`** trượt ở câu 2. Một lần đổi trạng thái thành hai lần ghi vào hai collection; muốn trạng thái và lịch sử luôn khớp thì phải dùng transaction nhiều document cho thao tác nóng nhất của hệ thống. Lab có trần 20 sự kiện, nên lợi ích duy nhất của việc tách (lịch sử không giới hạn) không cần đến.

Có trường hợp tách lịch sử là đúng: khi lịch sử phải giữ lâu hơn nhiều so với chuyến (lưu trữ pháp lý trong khi chuyến được dọn sau 90 ngày), hoặc khi có hệ thống khác cần đọc dòng sự kiện. Phần 6 dùng change stream cho nhu cầu thứ hai mà vẫn giữ `events` nhúng.

## 9. So với PostgreSQL và SQL Server

| Chủ đề | PostgreSQL / SQL Server (lab P01) | MongoDB (lab P01) |
|---|---|---|
| Xuất phát điểm thiết kế | Dữ kiện và phụ thuộc hàm, chuẩn hoá tới 3NF | Cách đọc và ghi; đơn vị nguyên tử là document |
| Lịch sử trạng thái | Bảng `trip_events`, ghi cùng transaction với `UPDATE trips` | Mảng `trips.events`, một `updateOne` với `$set` + `$push` |
| Giới hạn số sự kiện | Không cần (bảng không có trần) | `maxItems: 20` trong validator |
| Menu | Bảng riêng | Mảng `merchants.menu`, `maxItems: 200` |
| Tham chiếu | FOREIGN KEY, database chặn khi ghi và khi xoá | Trường cùng kiểu với `_id`, không có gì chặn; kiểm bằng `$lookup` (K1–K6) |
| Tổng kích thước một bản ghi | Dòng lớn được tách ra lưu ngoài (TOAST, LOB) | Trần cứng 16 MB mỗi document |
| Đọc dạng phẳng cho CSKH | JOIN hoặc view | `$lookup` trong aggregation, hoặc chép trường có chủ đích |

Hai cách đều đúng trong hệ của nó. Chỗ người chuyển từ SQL sang hay vấp là mang thói quen "mỗi thực thể một bảng" sang MongoDB, rồi bù lại bằng `$lookup` và transaction ở mọi nơi; còn người chuyển chiều ngược lại hay nhúng mọi thứ rồi gặp mảng không giới hạn.

## 10. Những hiểu nhầm thường gặp

- **"MongoDB là document database nên cứ nhúng hết."** Nhúng chỉ đúng khi dữ liệu được đọc cùng, ghi cùng và có giới hạn. Khách → chuyến không thoả điều kiện nào trong số đó.
- **"Tham chiếu trong MongoDB là anti-pattern."** Lab tham chiếu năm quan hệ trên bảy. Tham chiếu đúng chỗ cộng index đúng chỗ là thiết kế bình thường.
- **"16 MB còn xa, chưa cần lo."** Giới hạn 16 MB là lỗi cuối cùng; chi phí đọc, ghi và index của document lớn đến sớm hơn nhiều. Đặt `maxItems` để luật giới hạn nằm trong database.
- **"Có `customer_id` kiểu UUID là có khoá ngoại."** Validator chỉ kiểm kiểu. Chuyến của khách không tồn tại vẫn vào (B1), xoá khách vẫn thành công (`04`). Cần kiểm bằng `$lookup`.
- **"Chép tên khách vào chuyến là phi chuẩn hoá, xấu."** Có thể là snapshot đúng nghĩa (biên lai) hoặc extended reference có chủ đích. Xấu là chép mà không quyết định nghĩa, và không có kiểm tra phát hiện lệch.
- **"`$lookup` chậm nên không bao giờ dùng."** Nối theo `_id` là tra index. Vấn đề chỉ xuất hiện khi thao tác nóng nhất phải nối nhiều lần, hoặc khi nối từ "một" sang "nhiều" không giới hạn.

## 11. Bài tập

**Bài 1.** Với mỗi dữ liệu sau, chọn nhúng hay tham chiếu và nêu câu hỏi nào trong bốn câu ở mục 2 quyết định: (a) địa chỉ đón và trả của một chuyến; (b) đánh giá 1–5 sao khách để lại cho tài xế sau chuyến; (c) danh sách giấy tờ (bằng lái, đăng ký xe) của tài xế, mỗi tài xế 2–4 giấy tờ.

<details><summary><b>Lời giải</b></summary><p>(a) Nhúng vào chuyến: luôn đọc cùng chuyến, ghi lúc đặt, đúng hai phần tử, không dùng chung; đây cũng là snapshot (khách đổi địa chỉ nhà không đổi điểm đón của chuyến cũ). (b) Nếu mỗi chuyến tối đa một đánh giá thì nhúng vào chuyến được (có giới hạn, gắn với chuyến); nhưng không nhúng vào tài xế, vì tài xế có số đánh giá không giới hạn (câu 3). Điểm trung bình của tài xế là bản sao suy ra, cần nguồn gốc và kiểm tra như K7. (c) Nhúng vào tài xế với <code>maxItems</code> nhỏ: có giới hạn, đọc cùng hồ sơ tài xế, không dùng chung. Nếu ảnh scan giấy tờ lớn thì chỉ nhúng thông tin, còn file để ở kho lưu trữ đối tượng và tham chiếu bằng khoá.</p></details>

**Bài 2.** Validator của `merchants` không chặn hai món trùng `sku` trong cùng một menu. Thêm `uniqueItems: true` vào mảng `menu` có giải quyết được không? Viết một truy vấn kiểm tra kiểu `07-checks.js` để phát hiện vi phạm.

<details><summary><b>Lời giải</b></summary><p>Không đủ. <code>uniqueItems</code> so sánh <b>cả phần tử</b>. Thử trong database nháp <code>scratch_d</code> trên lab 8.0.38: hai món <code>{ sku: 'CT-SUON', price: 45000 }</code> và <code>{ sku: 'CT-SUON', price: 47000 }</code> vẫn được nhận; chỉ hai món giống hệt nhau mới bị chặn với lỗi 121. Kiểm bằng aggregation, so số món với số <code>sku</code> khác nhau:</p>
<pre><code class="language-js">db.merchants.aggregate([
  { $project: { name: 1, n: { $size: '$menu' }, n_sku: { $size: { $setUnion: ['$menu.sku', []] } } } },
  { $match: { $expr: { $ne: ['$n', '$n_sku'] } } },
]);</code></pre>
<p>Trên lab, truy vấn trả 0 document. Đây là ví dụ cho ranh giới của validator: luật "duy nhất trong mảng của một document" không có trong <code>$jsonSchema</code>, và index unique trên <code>menu.sku</code> lại chặn trùng <b>giữa các quán</b>, không phải trong một quán.</p></details>

**Bài 3.** Đội tài chính đề xuất thêm vào mỗi khách trường `trip_ids` chứa id mọi chuyến của khách, "để đếm số chuyến nhanh". Chỉ ra hai vấn đề và đề xuất cách thay thế.

<details><summary><b>Lời giải</b></summary><p>(1) Mảng không giới hạn: cùng vấn đề của việc nhúng chuyến, chỉ chậm hơn vì mỗi phần tử nhỏ hơn. (2) Bản sao phải ghi đồng bộ: tạo chuyến là ghi hai document ở hai collection, không nguyên tử; lỗi giữa chừng làm <code>trip_ids</code> lệch với <code>trips</code>. Thay thế: đếm bằng <code>db.trips.countDocuments({ customer_id })</code>, có index <code>ix_trips_customer_requested</code> hỗ trợ. Nếu cần con số sẵn để hiển thị, giữ một bộ đếm <code>trip_count</code> tăng bằng <code>$inc</code> (bản sao suy ra) kèm một kiểm tra so nó với <code>countDocuments</code>, giống cách K7 canh <code>drivers.status</code>.</p></details>

**Bài 4.** Màn hình danh sách đơn của quán cần hiện tên tài xế và biển số. Thiết kế theo extended reference: chép trường nào vào chuyến, lúc nào ghi, và kiểm tra nào phát hiện lệch?

<details><summary><b>Lời giải</b></summary><p>Chép <code>driver: { _id, full_name, vehicle_plate }</code> vào chuyến ở đúng bước nhận chuyến (cùng <code>updateOne</code> với <code>status: 'accepted'</code>, nên nguyên tử với việc gán tài xế). Cần quyết định nghĩa: với chuyến đang chạy, quán và khách cần biển số <b>hiện tại</b>; với chuyến đã xong, biển số lúc chạy là snapshot hợp lý hơn. Một cách nhất quán là coi trường này là snapshot lúc nhận chuyến, và nếu tài xế đổi xe giữa chuyến thì cập nhật riêng các chuyến đang chạy của tài xế. Kiểm tra: <code>$lookup</code> chuyến đang chạy sang <code>drivers</code>, báo chuyến có <code>driver.vehicle_plate</code> khác biển số hiện tại. <code>vehicle_plate</code> chỉ có sau migration ở <a href="/articles/mongodb-schema-migration-tuong-thich">MG-009</a>.</p></details>

## 12. Đọc tiếp

- [Data Modeling](https://www.mongodb.com/docs/manual/data-modeling/): tổng quan, nhúng và tham chiếu.
- [Embedded Data Versus References](https://www.mongodb.com/docs/manual/data-modeling/concepts/embedding-vs-references/): tiêu chí chọn.
- [Building with Patterns / Schema Design Patterns](https://www.mongodb.com/docs/manual/data-modeling/design-patterns/): subset, extended reference, bucket, schema versioning.
- [`$lookup`](https://www.mongodb.com/docs/manual/reference/operator/aggregation/lookup/): cú pháp `localField`/`foreignField` có kèm `pipeline`.
- [MongoDB Limits and Thresholds](https://www.mongodb.com/docs/manual/reference/limits/): giới hạn 16 MB mỗi document.
- [PG-008 · Chuẩn hoá và ranh giới mô hình dữ liệu](/articles/postgresql-mo-hinh-quan-he-chuan-hoa): cùng câu hỏi, trả lời bằng chuẩn hoá và bảng `trip_events`.
- Lab: `p01/01-schema.js` (validator có `maxItems`), `p01/04-lifecycle.js`, `p01/07-checks.js` (K1–K7).

Bài tiếp theo: [MG-009 · Đổi schema khi không có ALTER TABLE](/articles/mongodb-schema-migration-tuong-thich).
