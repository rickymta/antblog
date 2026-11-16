---
slug: mongodb-find-loc-sap-xep-collation
title: "find trong MongoDB: lọc, projection, sắp xếp, phân trang và collation cho danh sách chuyến Tới Nơi"
summary: "Hai nhân viên chăm sóc khách hàng đếm chuyến ngày 02/11 ra 8 và 9, trang danh sách đổi thứ tự giữa hai lần xem, tìm «nguyen van an» ra 0 người. Bài đi qua bộ lọc, projection, sort, skip/limit, truy vấn mảng, null và trường vắng, collation tiếng Việt trên MongoDB 8.0 để mỗi câu hỏi có đúng một đáp án."
category: mongodb
level: intermediate
tags: [mongodb, mg-007, mongodb-8, find, projection, sort, pagination, collation, toi-noi]
language: vi
---

Sáng 03/11, trưởng nhóm chăm sóc khách hàng của Tới Nơi hỏi một câu đơn giản: «Hôm qua, ngày 02/11, có bao nhiêu chuyến?» Hai nhân viên cùng mở DBStudio, cùng gõ một câu `find` trên collection `trips`. Một người báo **8**, người kia báo **9**. Cả hai đều không gặp lỗi nào. Cùng buổi sáng, màn hình danh sách chuyến đổi chỗ hai chuyến 100003 và 100004 giữa hai lần bấm «Tải lại», còn ô tìm khách gõ «nguyen van an» thì không ra ai, dù anh Nguyễn Văn An là khách quen.

Ba chuyện, một nguyên nhân chung: câu truy vấn trả lời đúng câu hỏi *được viết ra*, không phải câu hỏi *trong đầu người viết*. Bài này đi qua `find` của MongoDB — bộ lọc, projection, sắp xếp, phân trang, truy vấn mảng, `null`, collation — với dữ liệu thật của lab, và chỉ ra những chỗ MongoDB trả về 0 hay một con số sai mà không báo gì.

<div class="tkh-callout">
<span class="tkh-callout-title">MG-007 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/mongodb-insert-update-delete">MG-006 · insert, update, delete và toán tử cập nhật</a> và <a href="/articles/mongodb-kieu-bson-tien-thoi-gian-unicode">MG-004 · Kiểu BSON cho tiền, thời gian, chuỗi tiếng Việt</a>. Lab: <code>examples/toi-noi-mongodb/p01</code> (tệp <code>05-find.js</code>, chạy sau <code>04</code> nên có 16 chuyến). Sau bài này, bạn lọc được theo ngày giờ Việt Nam, sắp xếp có thứ tự xác định, phân biệt <code>null</code> với trường vắng và chọn collation phù hợp để tìm tên tiếng Việt.</p>
</div>

## 1. find gồm những phần nào

`find` nhận hai tham số và trả về một **cursor** (con trỏ duyệt kết quả):

```js
db.trips.find(
  { city_id: 'SG', status: { $in: ['completed', 'cancelled'] } },   // bộ lọc (filter)
  { _id: 0, code: 1, status: 1, fare: 1 },                          // projection
).sort({ code: 1 }).skip(0).limit(20);
```

- **Bộ lọc** (filter): document mô tả điều kiện. Nhiều trường trong một bộ lọc nghĩa là **AND**. `{}` hoặc bỏ trống là lấy tất cả.
- **Projection**: chọn trường trả về (mục 5).
- **`sort`, `skip`, `limit`**: gắn vào cursor. Thứ tự bạn viết không quan trọng — MongoDB luôn sắp xếp trước, rồi bỏ qua `skip` document, rồi lấy tối đa `limit` document.

Câu trên chạy trên lab (sau `npm run p01`) trả về 6 chuyến ở TP. Hồ Chí Minh đã hoàn thành hoặc bị huỷ:

```text
[
  { code: 100000, status: 'completed', fare: 45000 },
  { code: 100002, status: 'completed', fare: 62000 },
  { code: 100006, status: 'cancelled', fare: 52000 },
  { code: 100007, status: 'completed', fare: 30000 },
  { code: 100008, status: 'completed', fare: 0 },
  { code: 100015, status: 'completed', fare: 41000 }
]
```

(mongosh in mỗi trường một dòng; bài gộp lại cho gọn.) Các toán tử so sánh hay dùng:

| Toán tử | Nghĩa | Ví dụ |
|---|---|---|
| `{ f: v }` hoặc `$eq` | bằng | `{ city_id: 'SG' }` |
| `$ne` | khác (kể cả khi **không có** trường — mục 7) | `{ status: { $ne: 'cancelled' } }` |
| `$gt`, `$gte`, `$lt`, `$lte` | lớn hơn, ≥, nhỏ hơn, ≤ | `{ fare: { $gte: 100000 } }` |
| `$in`, `$nin` | thuộc / không thuộc danh sách | `{ kind: { $in: ['food', 'parcel'] } }` |
| `$exists` | có / không có trường | `{ driver_id: { $exists: false } }` |
| `$or`, `$and`, `$nor` | kết hợp nhiều điều kiện | `{ $or: [{ kind: 'food' }, { fare: { $gte: 100000 } }] }` → 6 chuyến |

Toán tử đầy đủ và `$expr` là bài MG-011 ở Phần 2. Bài này tập trung vào những chỗ dễ sai.

## 2. Lọc theo ngày giờ Việt Nam

Lab lưu mọi thời điểm là BSON `Date` — một mốc tuyệt đối tính theo UTC (MG-004). «Ngày 02/11» của nhân viên Tới Nơi là ngày theo **giờ Việt Nam** (UTC+7): từ 00:00 ngày 02/11 tới trước 00:00 ngày 03/11. Cách viết đúng là **khoảng nửa mở** `[đầu ngày, đầu ngày hôm sau)`: `$gte` cho mốc đầu, `$lt` cho mốc cuối.

```js
const from = vn('2026-11-02 00:00'), to = vn('2026-11-03 00:00');
db.trips.find(
  { requested_at: { $gte: from, $lt: to } },
  { _id: 0, code: 1, status: 1, requested_at: 1 },
).sort({ requested_at: 1, code: 1 });
```

```text
code|status|requested_at
----|------|------------
100000|completed|2026-11-02 07:58:00 +07:00
100001|completed|2026-11-02 09:15:00 +07:00
100002|completed|2026-11-02 11:05:00 +07:00
100003|cancelled|2026-11-02 12:00:00 +07:00
100004|completed|2026-11-02 12:00:00 +07:00
100005|completed|2026-11-02 17:30:00 +07:00
100006|cancelled|2026-11-02 18:00:00 +07:00
100007|completed|2026-11-02 23:50:00 +07:00
(8 document)
PASS chuyến ngày 02/11 giờ VN = 8
```

`vn()` là hàm của lab (`lib/check.js`). Trong query editor của DBStudio hay mongosh thông thường, viết mốc kèm độ lệch múi giờ: `ISODate('2026-11-02T00:00:00+07:00')` — mongosh đổi nó thành `ISODate('2026-11-01T17:00:00.000Z')`, đúng 00:00 giờ Việt Nam. Câu sau cho cùng 8 chuyến:

```js
db.trips.find(
  { requested_at: { $gte: ISODate('2026-11-02T00:00:00+07:00'), $lt: ISODate('2026-11-03T00:00:00+07:00') } },
  { _id: 0, code: 1, status: 1, requested_at: 1 },
).sort({ requested_at: 1, code: 1 });
```

Lưu ý DBStudio (cũng như mongosh) hiển thị `requested_at` theo UTC: chuyến 100000 hiện `2026-11-02T00:58:00.000Z`, tức 07:58 giờ Việt Nam.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-find-loc-sap-xep-collation/dbstudio-01.webp" alt="DBStudio chạy find các chuyến ngày 02/11 giờ Việt Nam, 8 dòng từ 100000 tới 100007, requested_at theo UTC" width="1086" height="665" loading="lazy"></div><figcaption>DBStudio · 8 chuyến của ngày 02/11 giờ VN, hiển thị theo UTC</figcaption></figure>

Vì sao nửa mở? Viết `$lte: ISODate('2026-11-02T23:59:59+07:00')` bỏ sót chuyến đặt lúc 23:59:59.500, vì `Date` có độ phân giải mili giây. `$lt` đầu ngày hôm sau không có khe hở nào, và các ngày liền nhau ghép lại không trùng, không sót.

### Người đếm ra 9 đã làm gì

Nhân viên thứ hai lấy «ngày» theo UTC. Có hai cách hay gặp. Cách một: viết mốc không có múi giờ, `ISODate('2026-11-02')` và `ISODate('2026-11-03')` — mongosh hiểu là 00:00 **UTC**, tức 07:00 sáng giờ Việt Nam. Chạy trên lab, câu đó trả về **9**. Cách hai: so ngày bằng `$dateToString` mà không truyền `timezone`. Lab đếm theo cả hai kiểu:

```text
theo_utc|theo_gio_vn
--------|-----------
9|8
(1 document)
PASS ngày UTC đếm thừa 100008 (00:10 ngày 03/11 giờ VN) = 9 vs 8
```

Chuyến 100008 đặt lúc 00:10 ngày 03/11 giờ Việt Nam, tức 17:10 ngày 02/11 UTC: theo UTC nó thuộc ngày 02/11. Ngược lại, chuyến đặt từ 00:00 đến 06:59 sáng 02/11 giờ Việt Nam sẽ bị tính sang ngày 01/11. Fixture không có chuyến nào trong khung đó, nên lần này chỉ thừa một chuyến; ngày có chuyến đêm thì sai cả hai đầu. Gom theo ngày, tuần, khung 15 phút đúng múi giờ bằng `$dateTrunc` là bài MG-016.

### So khác kiểu: không lỗi, 0 kết quả

Hai lỗi thứ ba và thứ tư trong `05-find.js` còn khó thấy hơn:

```text
PASS requested_at >= "2026-11-02" = 0
PASS customer_id = chuỗi UUID = 0
PASS customer_id = UUID(...) = 4
```

`{ requested_at: { $gte: '2026-11-02' } }` so một `Date` với một **chuỗi**. MongoDB không đổi kiểu và không báo lỗi: với các toán tử so sánh, nó chỉ so giá trị **cùng nhóm kiểu BSON** (type bracketing), nên không document nào khớp. Tương tự, `customer_id` lưu kiểu UUID (binary subtype 4); so với chuỗi `'c0000000-0000-4000-8000-000000000001'` ra 0, phải viết `UUID('c0000000-…-000000000001')` mới thấy 4 chuyến của anh An (100000, 100006, 100012, 100015).

Đây là chỗ một câu truy vấn sai trông giống hệt «không có dữ liệu». Khi kết quả bằng 0 ngoài dự đoán, kiểm kiểu trước: `typeof` trong mongosh, hoặc mở document trong DBStudio và xem kiểu của trường.

## 3. Sắp xếp phải có thứ tự xác định

`sort({ requested_at: 1 })` sắp tăng dần, `-1` giảm dần. Chuyến 100003 và 100004 **cùng** đặt lúc 12:00 ngày 02/11. Chỉ sắp theo `requested_at`, MongoDB không hứa hai chuyến này đứng theo thứ tự nào: lần này 100003 trước, lần sau có thể ngược lại, nhất là khi kế hoạch truy vấn đổi (dùng index khác, dữ liệu được ghi lại). Đó là lý do màn hình danh sách «nhảy» giữa hai lần tải. Thêm **khoá phụ** (tie-breaker) duy nhất:

```js
db.trips.find({ requested_at: vn('2026-11-02 12:00') }, { _id: 0, code: 1, requested_at: 1 })
  .sort({ requested_at: 1, code: 1 });
```

```text
code|requested_at
----|------------
100003|2026-11-02 12:00:00 +07:00
100004|2026-11-02 12:00:00 +07:00
(2 document)
```

`code` có index unique, nên cặp `(requested_at, code)` không bao giờ trùng — thứ tự được xác định hoàn toàn. Quy tắc chung: khoá sắp xếp cuối cùng phải là một trường duy nhất (`code` hoặc `_id`).

Hai chi tiết khác:

- **Không có `sort` thì không có thứ tự.** Thứ tự trả về khi không sort phụ thuộc cách MongoDB đọc dữ liệu (thứ tự lưu, index được chọn). Nó có thể *trông* như thứ tự thêm vào trên lab nhỏ, nhưng không được bảo đảm.
- **Sort trên nhiều document không có index phải sắp trong bộ nhớ**, có giới hạn bộ nhớ riêng. Index hỗ trợ sắp xếp và quy tắc ESR là chủ đề của Phần 3 (MG-022, MG-023).

## 4. Phân trang bằng skip và limit

Trang 2, mỗi trang 5 chuyến, mới nhất trước: bỏ 5 document đầu, lấy 5 document tiếp theo.

```js
db.trips.find({}, { _id: 0, code: 1, requested_at: 1 })
  .sort({ requested_at: -1, code: -1 }).skip(5).limit(5);
```

```text
code|requested_at
----|------------
100010|2026-11-03 08:05:00 +07:00
100009|2026-11-03 08:00:00 +07:00
100008|2026-11-03 00:10:00 +07:00
100007|2026-11-02 23:50:00 +07:00
100006|2026-11-02 18:00:00 +07:00
(5 document)
```

Trang `p` (đánh từ 1), mỗi trang `n` document: `.skip((p - 1) * n).limit(n)`. Trên lab, trang 1 là 100015 → 100011, trang 3 là 100005 → 100001, trang 4 chỉ còn 100000. Ở trang 3, 100004 đứng trước 100003 vì cả hai khoá đều giảm dần — thứ tự đó cố định nhờ khoá phụ `code: -1`.

`skip` có hai giới hạn mà khoá phụ không chữa được:

- **Chi phí:** document bị `skip` vẫn phải được server duyệt qua rồi bỏ đi. Trang thứ 1.000 của lịch sử chuyến tốn gần bằng đọc 1.000 trang.
- **Dữ liệu đổi giữa hai lần xem:** một chuyến mới chen vào đầu danh sách đẩy mọi document lùi một vị trí; người đang xem trang 2 sẽ thấy lại một chuyến đã xem ở trang 1.

Cách chữa là **phân trang keyset**: «lấy các chuyến đứng sau chuyến cuối cùng tôi đã thấy», bằng điều kiện trên chính khoá sắp xếp thay cho `skip`. MG-018 ở Phần 2 chứng minh điều kiện trang kế tiếp khi khoá sắp xếp có giá trị trùng. Với danh sách ngắn ở Phần 1, `skip`/`limit` có khoá phụ là đủ.

## 5. Projection: trường lồng nhau và cắt mảng

Projection trả lời «trả về những trường nào». Hai kiểu, không trộn được:

- **Bao gồm** (`1`): chỉ trả các trường được liệt kê, cộng `_id` (trừ khi ghi `_id: 0`).
- **Loại trừ** (`0`): trả mọi trường trừ các trường được liệt kê.

Ngoại lệ duy nhất là `_id: 0` đi được với kiểu bao gồm. Viết `{ code: 1, events: 0 }` thì MongoDB báo lỗi.

Trường lồng nhau dùng **dot notation** (cú pháp dấu chấm). Danh sách quán chỉ cần tên món, không cần giá và SKU:

```js
db.merchants.find({}, { _id: 0, name: 1, 'menu.name': 1 }).sort({ name: 1 });
```

```text
name|menu
----|----
Bún Chả Ngõ Mẫu|[{"name":"Bún chả suất thường"},{"name":"Nem rán (2 chiếc)"}]
Cơm Tấm Phố Lab|[{"name":"Cơm tấm sườn"},{"name":"Cơm tấm sườn bì chả"}]
Mì Quảng Thử Nghiệm|[{"name":"Mì Quảng gà"},{"name":"Mì Quảng tôm thịt"}]
(3 document)
```

`'menu.name'` áp dụng cho từng phần tử của mảng `menu`: kết quả vẫn là mảng, mỗi phần tử chỉ còn `name`.

Với mảng dài như lịch sử trạng thái, **`$slice`** lấy một phần: số dương lấy từ đầu, số âm lấy từ cuối. Hai sự kiện cuối của chuyến 100015:

```js
db.trips.find({ code: 100015 }, { _id: 0, code: 1, events: { $slice: -2 } });
```

```text
code|events
----|------
100015|[{"status":"arriving","at":{"$date":"2026-11-03T02:08:00Z"},"by":"system"},{"status":"completed","at":{"$date":"2026-11-03T02:11:00Z"},"by":"driver"}]
(1 document)
```

(Cột `at` hiện theo UTC: 02:08Z là 09:08 giờ Việt Nam.) Projection còn nhận biểu thức aggregation — MG-006 đã dùng `n_events: { $size: '$events' }` trong `findOneAndUpdate`.

Projection giảm dữ liệu gửi qua mạng, nhưng **không** làm MongoDB đọc ít hơn từ đĩa: server vẫn nạp cả document rồi mới cắt. Truy vấn chỉ đọc index mà không chạm document (covered query) là MG-024.

## 6. Truy vấn trên mảng

Bộ lọc dot notation trên mảng khớp khi **có ít nhất một phần tử** thoả:

```text
PASS quán có món dưới 45.000 đ = 2
```

`{ 'menu.price': { $lt: 45000 } }` khớp Bún Chả Ngõ Mẫu (nem rán 20.000 đ) và Mì Quảng Thử Nghiệm (mì Quảng gà 40.000 đ). Cơm Tấm Phố Lab có món rẻ nhất 45.000 đ nên không khớp.

Bẫy xuất hiện khi có **hai điều kiện** trên cùng mảng. Câu hỏi: «quán nào có món gà giá từ 45.000 đ?». Viết bằng hai điều kiện dot notation (chạy thêm trên lab):

```js
db.merchants.find({ 'menu.name': /gà/, 'menu.price': { $gte: 45000 } }, { _id: 0, name: 1 });
```

```text
[ { name: 'Mì Quảng Thử Nghiệm' } ]
```

Sai: món gà của quán này giá 40.000 đ. Điều kiện `/gà/` khớp phần tử «Mì Quảng gà», điều kiện giá khớp phần tử *khác* là «Mì Quảng tôm thịt» 45.000 đ. Mỗi điều kiện chỉ cần một phần tử nào đó thoả, không nhất thiết cùng phần tử. Muốn **cùng một phần tử** thoả mọi điều kiện, dùng `$elemMatch`:

```js
db.merchants.find({ menu: { $elemMatch: { name: /gà/, price: { $gte: 45000 } } } }, { _id: 0, name: 1 });
```

```text
[]
```

Đúng: không quán nào có món gà từ 45.000 đ. Lab dùng `$elemMatch` cho lịch sử chuyến — «chuyến có sự kiện `accepted` do tài xế thực hiện»:

```text
PASS chuyến có sự kiện accepted bởi driver = 1
```

Chỉ chuyến 100015 có lịch sử đầy đủ; 15 chuyến của fixture có `events` rỗng (dữ liệu chuyển từ hệ cũ chỉ có trạng thái cuối, xem `02-fixture-valid.js`).

Muốn trả về **chỉ phần tử khớp** thay vì cả mảng, projection có toán tử vị trí `'menu.$': 1`: chạy `find({ 'menu.price': { $lt: 45000 } }, { _id: 0, name: 1, 'menu.$': 1 })` trên lab, Bún Chả chỉ còn món nem rán, Mì Quảng chỉ còn món gà. Lọc và biến đổi mảng phức tạp hơn (`$filter`, `$map`, `$unwind`) là MG-014.

## 7. null, trường vắng và cái bẫy của $ne

Lab không bao giờ ghi `driver_id: null` — validator cấm. Chuyến chưa có tài xế đơn giản là **không có** trường `driver_id` (MG-003). Vậy truy vấn `{ driver_id: null }` trả gì?

```text
code|status
----|------
100006|cancelled
100013|searching
100014|requested
(3 document)
PASS {driver_id: null} khớp cả trường vắng = 3
PASS {driver_id: {$exists: false}} = 3
```

`{ f: null }` khớp document có `f` bằng `null` **hoặc không có `f`**. Trên lab hai tập này trùng nhau vì không có `null` thật nào. Khi cần phân biệt:

| Bộ lọc | Khớp |
|---|---|
| `{ driver_id: null }` | giá trị `null` **hoặc** trường vắng |
| `{ driver_id: { $exists: false } }` | chỉ trường vắng |
| `{ driver_id: { $type: 'null' } }` | chỉ giá trị `null` (lab: 0) |
| `{ driver_id: { $exists: true } }` | có trường, kể cả khi bằng `null` (lab: 13) |

### $ne và $nin khớp cả document không có trường

Câu hỏi: «các chuyến **không** do tài xế 8001 chạy». Tài xế 8001 có ba chuyến (100000, 100008, 100012) trong 16 chuyến.

```text
PASS $ne tài xế 8001 (gồm cả 3 chuyến chưa có tài xế) = 13
PASS $ne 8001 và có tài xế = 10
```

`{ driver_id: { $ne: UUID('…8001') } }` trả **13**: 10 chuyến của tài xế khác **cộng 3 chuyến chưa có tài xế**. Với MongoDB, document không có `driver_id` thì «`driver_id` khác 8001» là đúng. `$nin` cũng vậy: `$nin` với 8001 và 8002 trả 10 trên lab, gồm cả ba chuyến không tài xế. Muốn «chuyến có tài xế, và tài xế không phải 8001», thêm `$exists: true` — kết quả 10.

Đây là **cái bẫy ngược** với SQL. Trong PostgreSQL hay SQL Server, `driver_id <> 8001` loại các dòng `driver_id IS NULL` (so sánh với NULL cho UNKNOWN), nên câu tương ứng trả 10; và `NOT IN (…)` với danh sách chứa `NULL` còn trả **0 dòng**. Người chuyển từ SQL sang thường quên điều kiện `$exists`; người chuyển từ MongoDB sang SQL thì quên `OR driver_id IS NULL`. Cả hai đều không có lỗi để nhắc.

## 8. Collation: sắp xếp và tìm tên tiếng Việt

**Collation** là bộ quy tắc so sánh và sắp xếp chuỗi theo ngôn ngữ. Không chỉ định gì, MongoDB so chuỗi theo **byte** (simple binary comparison). Danh sách khách sắp theo tên:

```text
so byte (mặc định): Lê Hoàng Cường · Nguyễn Văn An · Phạm Thu Dung · Trần Thị Bình · Võ Minh Đức · Đặng Ngọc Hà
collation vi      : Đặng Ngọc Hà · Lê Hoàng Cường · Nguyễn Văn An · Phạm Thu Dung · Trần Thị Bình · Võ Minh Đức
PASS Đặng đứng cuối khi so byte = Đặng Ngọc Hà
PASS collation vi: Đ đứng sau D, trước L ⇒ Đặng đầu danh sách = Đặng Ngọc Hà
```

So byte, chữ `Đ` (U+0110) có mã lớn hơn mọi chữ Latin không dấu, nên chị Đặng Ngọc Hà đứng sau anh Võ Minh Đức. Với `collation({ locale: 'vi' })`, MongoDB dùng quy tắc tiếng Việt của thư viện ICU: `Đ` là chữ cái đứng ngay sau `D`. Không có khách nào tên bắt đầu bằng A, B, C hay D, nên chị Hà lên đầu danh sách. Trong DBStudio:

```js
db.customers.find({}, { _id: 0, full_name: 1 }).sort({ full_name: 1 }).collation({ locale: 'vi' });
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-find-loc-sap-xep-collation/dbstudio-02.webp" alt="DBStudio chạy find khách hàng sắp xếp với collation vi, Đặng Ngọc Hà đứng đầu" width="1086" height="595" loading="lazy"></div><figcaption>DBStudio · sắp xếp với <code>collation({ locale: 'vi' })</code> (thêm <code>phone</code> vào projection)</figcaption></figure>

### Strength: phân biệt tới mức nào

Tham số **`strength`** quyết định collation coi những khác biệt nào là đáng kể. Với tiếng Việt, ICU chia như sau:

| Mức | Phân biệt | Ví dụ với `locale: 'vi'` |
|---|---|---|
| 1 (primary) | chỉ **chữ cái gốc** | `a` ≠ `ă` ≠ `â`, `d` ≠ `đ`; nhưng `văn` = `vẵn` = `VĂN` |
| 2 (secondary) | thêm **dấu thanh** | `Nguyễn` ≠ `Nguyên`; hoa = thường |
| 3 (tertiary, mặc định) | thêm **hoa/thường** | `An` ≠ `an` |

Điểm mấu chốt: trong tiếng Việt, **ă, â, ê, ô, ơ, ư, đ là chữ cái riêng**, không phải «a có dấu». Chỉ năm dấu thanh (sắc, huyền, hỏi, ngã, nặng) là dấu phụ. Bảng đếm khách theo từng cách gõ của `05-find.js`:

```text
truy_van|khong_collation|vi_1|en_1
--------|---------------|----|----
nguyen van an|0|0|1
nguyên văn an|0|1|1
NGUYỄN VĂN AN|0|1|1
dang ngoc ha|0|0|1
(4 document)
PASS vi strength 1: "nguyen van an" KHÔNG khớp (văn ≠ van) = 0
PASS vi strength 1: bỏ dấu thanh và hoa/thường = 1
PASS en strength 1: bỏ mọi dấu ⇒ tìm không dấu = 1
```

- **Không collation**: chỉ khớp chuỗi giống từng byte. Cả bốn cách gõ đều ra 0.
- **`vi`, strength 1**: «nguyên văn an» (sai dấu thanh, viết thường) khớp, nhưng «nguyen van an» **không** khớp, vì `van` ≠ `văn` ở mức chữ cái. Đúng với tiếng Việt, nhưng không giải được bài toán «gõ không dấu».
- **`en`, strength 1**: quy tắc tiếng Anh coi mọi dấu là dấu phụ, kể cả `ă`, `đ`. «nguyen van an» và «dang ngoc ha» đều khớp. Đây mới là thứ ô tìm khách cần.

Chạy thêm trên lab với `vi` strength 2: «NGUYỄN VĂN AN» khớp (bỏ qua hoa/thường), «Nguyen Van An» không khớp (dấu thanh và chữ cái đều khác). Chọn collation theo **câu hỏi nghiệp vụ**, không theo tên ngôn ngữ: sắp xếp danh sách cho người Việt đọc thì `vi`; tìm kiếm chấp nhận gõ không dấu thì cần quy tắc bỏ dấu như `en` strength 1 — và chấp nhận nó coi «Đức» = «Duc».

Biểu thức chính quy **không** dùng collation: `{ full_name: /^Nguy/ }` so theo ký tự, phân biệt dấu và hoa/thường (lab: 1 khách).

### Collation và index

Collation truyền vào truy vấn có tác dụng cho chính truy vấn đó. Collection và index cũng có collation mặc định riêng, đặt lúc tạo. Một quy tắc quan trọng: **muốn dùng index cho phép so sánh chuỗi, truy vấn phải dùng đúng collation của index**. Index `uq_customers_phone` của lab có collation mặc định (so byte); truy vấn trên `phone` kèm `collation({ locale: 'vi' })` không dùng được index đó cho điều kiện chuỗi. Đo bằng `explain`, và thiết kế index có collation cho tìm kiếm tiếng Việt, là MG-023 và MG-029 ở Phần 3.

## 9. Đếm: countDocuments và estimatedDocumentCount

```text
countDocuments|estimatedDocumentCount
--------------|----------------------
16|16
(1 document)
```

Hai con số bằng nhau trên lab, nhưng chúng làm việc khác nhau:

- **`countDocuments(filter)`** thực sự chạy truy vấn (bên trong là aggregation `$match` + `$group`) và đếm document khớp bộ lọc. Chính xác, nhận `limit`/`skip`, nhưng tốn công tương đương một lần duyệt.
- **`estimatedDocumentCount()`** đọc số lượng từ **metadata** của collection, **không nhận bộ lọc**. Rất nhanh, nhưng có thể lệch, ví dụ sau khi `mongod` tắt không sạch, hoặc trên cụm sharding khi còn document mồ côi đang di chuyển giữa các shard.

Dùng `estimatedDocumentCount` cho con số hiển thị «khoảng bao nhiêu chuyến», `countDocuments` cho mọi chỗ cần đúng (đối soát, kiểm tra). Phương thức cũ `cursor.count()` đã bị khuyến cáo không dùng; đừng chép nó từ các ví dụ cũ.

## 10. So với PostgreSQL và SQL Server

| Việc | PostgreSQL / SQL Server | MongoDB |
|---|---|---|
| Chọn cột / trường | `SELECT code, status` | Projection `{ code: 1, status: 1 }`; `_id` tự có trừ khi `_id: 0` |
| Cắt kết quả | `LIMIT/OFFSET`, `TOP`, `OFFSET … FETCH` | `.limit()`, `.skip()` |
| So khác kiểu | Lỗi hoặc ép kiểu ngầm | Không lỗi, không ép kiểu, 0 kết quả |
| Lọc «khác X» | `<>` loại dòng `NULL` (10) | `$ne` gồm cả document thiếu trường (13) |
| `NOT IN` danh sách có `NULL` | 0 dòng | `$nin` vẫn trả document thiếu trường |
| Điều kiện trên tập con | JOIN bảng con, `EXISTS` | Dot notation (phần tử bất kỳ) hoặc `$elemMatch` (cùng phần tử) |
| Collation | Gắn với cột/database (SQL Server: `Vietnamese_100_CI_AS`); PostgreSQL theo cột hoặc `COLLATE` | Mặc định so byte; truyền theo truy vấn hoặc đặt cho collection/index |

## 11. Những hiểu nhầm thường gặp

- **«Mốc `ISODate('2026-11-02')` là đầu ngày 02/11.»** Là 00:00 UTC, tức 07:00 sáng giờ Việt Nam. Luôn viết kèm `+07:00` hoặc tính mốc ở ứng dụng.
- **«Truy vấn ra 0 nghĩa là không có dữ liệu.»** Có thể bạn đang so `Date` với chuỗi, UUID với chuỗi, số với chuỗi. MongoDB không báo lỗi khác kiểu.
- **«Sort theo thời gian là đủ.»** Thời gian trùng nhau thường xuyên ở tải cao. Luôn thêm khoá phụ duy nhất.
- **«`{ f: null }` tìm document có `f` bằng null.»** Nó tìm cả document không có `f`. Dùng `$exists`/`$type` khi cần phân biệt.
- **«`$ne` giống `<>` của SQL.»** `$ne` khớp cả document thiếu trường.
- **«Hai điều kiện dot notation trên một mảng áp dụng cho cùng phần tử.»** Mỗi điều kiện có thể khớp một phần tử khác; dùng `$elemMatch`.
- **«Collation `vi` strength 1 cho phép gõ không dấu.»** Nó bỏ dấu thanh và hoa/thường, nhưng giữ ă, â, ê, ô, ơ, ư, đ là chữ cái riêng.

## 12. Bài tập

**Bài 1.** Viết câu `find` đếm số chuyến **hoàn thành** trong ngày 03/11 theo giờ Việt Nam, chỉ dùng `ISODate` (không dùng `vn()`). Trên lab sau `npm run p01`, kết quả là bao nhiêu?

<details><summary><b>Lời giải</b></summary><p><code>db.trips.countDocuments({ status: 'completed', requested_at: { $gte: ISODate('2026-11-03T00:00:00+07:00'), $lt: ISODate('2026-11-04T00:00:00+07:00') } })</code>. Các chuyến đặt trong ngày 03/11 giờ Việt Nam đã hoàn thành là 100008 (00:10) và 100015 (08:40), nên kết quả là 2. Chuyến 100007 hoàn thành lúc 00:20 ngày 03/11 nhưng được <em>đặt</em> lúc 23:50 ngày 02/11, nên không thuộc ngày 03/11 nếu lọc theo <code>requested_at</code>. Lọc theo <code>completed_at</code> thì là câu hỏi khác, với đáp án khác — hãy hỏi lại nghiệp vụ muốn ngày nào.</p></details>

**Bài 2.** Đội vận hành cần «các chuyến chưa có tài xế, hoặc có tài xế nhưng không phải 8001 và 8002». Viết bộ lọc ngắn nhất và giải thích vì sao không cần `$exists`.

<details><summary><b>Lời giải</b></summary><p><code>{ driver_id: { $nin: [UUID('d0000000-0000-4000-8000-000000008001'), UUID('d0000000-0000-4000-8000-000000008002')] } }</code>. <code>$nin</code> vốn khớp cả document không có <code>driver_id</code>, nên đúng yêu cầu mà không cần thêm điều kiện. Trên lab kết quả là 10 (7 chuyến của tài xế khác + 3 chuyến chưa có tài xế). Nếu yêu cầu chỉ là «có tài xế nhưng không phải 8001 và 8002» thì phải thêm <code>$exists: true</code>.</p></details>

**Bài 3.** Viết truy vấn tìm chuyến có sự kiện `accepted` xảy ra **sau** 08:40:30 ngày 03/11 (giờ Việt Nam). Vì sao `{ 'events.status': 'accepted', 'events.at': { $gt: … } }` có thể cho kết quả sai?

<details><summary><b>Lời giải</b></summary><p><code>db.trips.find({ events: { $elemMatch: { status: 'accepted', at: { $gt: ISODate('2026-11-03T08:40:30+07:00') } } } })</code> — trên lab trả chuyến 100015 (accepted lúc 08:41). Hai điều kiện dot notation có thể khớp hai phần tử khác nhau: một chuyến nhận lúc 08:00 nhưng hoàn thành lúc 09:00 sẽ khớp nhầm, vì sự kiện <code>completed</code> thoả điều kiện thời gian còn sự kiện <code>accepted</code> thoả điều kiện trạng thái.</p></details>

**Bài 4.** Ô tìm khách cần: gõ «duc» ra anh Võ Minh Đức khi tìm theo **họ tên đầy đủ** «vo minh duc», và danh sách kết quả sắp theo thứ tự tiếng Việt. Chọn collation cho từng việc và giải thích.

<details><summary><b>Lời giải</b></summary><p>So khớp: <code>{ locale: 'en', strength: 1 }</code>, vì quy tắc <code>vi</code> coi <code>đ</code> ≠ <code>d</code> và <code>ư</code> ≠ <code>u</code> ở mọi mức strength, nên «vo minh duc» không khớp. Sắp xếp: <code>{ locale: 'vi' }</code>, để Đ đứng sau D. Một truy vấn <code>find</code> chỉ nhận một collation, nên hai việc này thường tách ra: tìm bằng một collation (hoặc một trường đã bỏ dấu sẵn, có index riêng — MG-029), rồi sắp kết quả bằng collation khác. Tìm theo một phần tên («duc») thì so bằng không dùng được; cần text search hoặc trường bỏ dấu, cũng ở MG-029.</p></details>

## 13. Đọc tiếp

- [db.collection.find()](https://www.mongodb.com/docs/manual/reference/method/db.collection.find/) — bộ lọc, projection, cursor.
- [Query an Array of Embedded Documents](https://www.mongodb.com/docs/manual/tutorial/query-array-of-documents/) — dot notation và `$elemMatch`.
- [Query for Null or Missing Fields](https://www.mongodb.com/docs/manual/tutorial/query-for-null-fields/) — `null`, `$exists`, `$type`.
- [Comparison/Sort Order](https://www.mongodb.com/docs/manual/reference/bson-type-comparison-order/) — so sánh giữa các kiểu BSON.
- [Collation](https://www.mongodb.com/docs/manual/reference/collation/) — `locale`, `strength`, collation của index.
- [db.collection.countDocuments()](https://www.mongodb.com/docs/manual/reference/method/db.collection.countDocuments/) và [estimatedDocumentCount()](https://www.mongodb.com/docs/manual/reference/method/db.collection.estimatedDocumentCount/).
- [PG-007](/articles/postgresql-select-filter-order-limit) và [MS-007](/articles/mssql-select-loc-sap-xep-top-offset): cùng các câu hỏi trên PostgreSQL và SQL Server.

Bài tiếp theo: [MG-008 · Nhúng hay tham chiếu: mô hình document cho Tới Nơi](/articles/mongodb-nhung-hay-tham-chieu).
