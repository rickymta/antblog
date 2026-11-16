---
slug: mongodb-insert-update-delete
title: "insert, update, delete và toán tử cập nhật: một chuyến Tới Nơi trọn vòng đời trên MongoDB"
summary: "Khách đặt xong phải thấy ngay mã chuyến, tài xế bấm nhận trễ phải biết mình thua, mỗi lần đổi trạng thái phải để lại lịch sử. Bài dùng insertOne, updateOne có điều kiện, $push, findOneAndUpdate và bộ đếm mã để đưa chuyến 100015 qua 7 trạng thái, giải thích matchedCount 0, vì sao replaceOne làm mất trường và vì sao deleteOne không bị khoá ngoại nào chặn."
category: mongodb
level: intermediate
tags: [mongodb, mg-006, mongodb-8, insert, update, delete, findoneandupdate, update-operators, toi-noi]
language: vi
---

08:40 ngày 03/11, anh Nguyễn Văn An đặt một chuyến xe máy ở TP. Hồ Chí Minh. Ngay khi bấm «Đặt», app phải hiện **mã chuyến** — con số để anh đọc cho tổng đài nếu có chuyện, và app không tự biết con số đó. Một phút sau, tài xế 8007 bấm «Nhận»; một giây sau nữa, tài xế 8003 cũng bấm «Nhận» cùng chuyến. App của 8003 phải báo «chuyến đã có người nhận», không được lặng lẽ ghi đè tài xế 8007. Và mỗi lần chuyến đổi trạng thái, bộ phận chăm sóc khách hàng muốn thấy một dòng lịch sử: ai đổi, lúc nào.

Series [PostgreSQL](/articles/postgresql-insert-update-delete-returning) và [SQL Server](/articles/mssql-insert-update-delete-output) đã giải bài toán này bằng `INSERT`/`UPDATE` kèm `RETURNING`/`OUTPUT`, bảng lịch sử riêng và transaction. Trên MongoDB, công cụ khác hẳn: không có IDENTITY, không có khoá ngoại, nhưng lịch sử có thể nằm ngay trong document chuyến. Bài này đưa chuyến **100015** đi hết vòng đời trong lab, đọc kỹ từng con số MongoDB trả về sau mỗi lệnh ghi, và chỉ ra chỗ nào một lệnh là nguyên tử, chỗ nào không.

<div class="tkh-callout">
<span class="tkh-callout-title">MG-006 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/mongodb-schema-validation-unique-index">MG-005 · Schema validation với $jsonSchema và index unique</a>. Lab: <code>examples/toi-noi-mongodb/p01</code> (tệp <code>04-lifecycle.js</code>, kết quả trong <code>OUTPUT.txt</code>). Sau bài này, bạn đưa được một chuyến qua đủ vòng đời bằng update có điều kiện và <code>$push</code>, cấp mã bằng bộ đếm, đọc đúng <code>matchedCount</code>/<code>modifiedCount</code> và biết vì sao xoá trong MongoDB không bị chặn như có khoá ngoại.</p>
</div>

## 1. MongoDB trả lời gì sau một lệnh ghi

Mỗi phương thức ghi của collection trả về một **kết quả ghi** (write result) — một object nhỏ cho ứng dụng biết server đã làm gì. Đây là thứ thay cho thẻ lệnh `UPDATE 1` của PostgreSQL hay «rows affected» của SQL Server:

| Phương thức | Kết quả trả về | Ghi chú |
|---|---|---|
| `insertOne(doc)` | `acknowledged`, `insertedId` | `_id` của document vừa thêm (do bạn đặt, hoặc `ObjectId` tự sinh) |
| `insertMany([...])` | `acknowledged`, `insertedIds` | Object ánh xạ vị trí trong mảng → `_id` |
| `updateOne` / `updateMany` / `replaceOne` | `matchedCount`, `modifiedCount`, `upsertedCount`, id của document upsert | **`matchedCount: 0` không phải lỗi** |
| `deleteOne` / `deleteMany` | `deletedCount` | Xoá 0 document cũng không phải lỗi |
| `findOneAndUpdate` / `findOneAndReplace` / `findOneAndDelete` | **Chính document** (trước hoặc sau khi ghi), hoặc `null` | Thay cho `RETURNING` / `OUTPUT` |

Hai cặp số dễ nhầm nhất:

- **`matchedCount`**: số document khớp bộ lọc.
- **`modifiedCount`**: số document thật sự thay đổi. Nếu giá trị mới giống hệt giá trị cũ, MongoDB không ghi gì và không tính vào con số này (mục 8).

`acknowledged: true` nghĩa là server đã xác nhận lệnh ghi theo **write concern** (mức xác nhận ghi) đang dùng. Lab dùng mặc định; chọn `w`, `j` cho dữ liệu tiền là chủ đề của MG-032 ở Phần 4. Id của document được upsert: mongosh in ra dưới tên `insertedId`, driver Node.js gọi là `upsertedId` — cùng một thông tin.

Chạy lại toàn bộ lab P01 (dữ liệu sẽ bị xoá và dựng lại):

```bash
cd examples/toi-noi-mongodb
npm run db:up
npm run p01
```

Tệp `04-lifecycle.js` chạy sau fixture hợp lệ (`02`) và fixture sai (`03`). Các khối kết quả dưới đây chép từ `OUTPUT.txt`; khối nào chạy thêm ngoài lab, bài ghi rõ.

## 2. insertOne và insertMany: thêm tài xế 8007

Tài xế Mai Văn Quý vừa được duyệt hồ sơ lúc 08:31. `insertOne` thêm một document; `_id` do ứng dụng đặt sẵn (UUID, xem MG-004):

```js
const ins = db.drivers.insertOne({
  _id: UUID('d0000000-0000-4000-8000-000000008007'), code: 8007, full_name: 'Mai Văn Quý',
  city_id: 'SG', vehicle_type: 'bike', status: 'online', created_at: vn('2026-11-03 08:31'),
});
```

```text
PASS acknowledged = true
PASS insertedId = d0000000-0000-4000-8000-000000008007
```

Gõ cùng lệnh trực tiếp trong mongosh (hoặc query editor của DBStudio), bạn thấy object đầy đủ. Khối dưới chạy trên database nháp `scratch_c`, không phải `toinoi`:

```text
{
  acknowledged: true,
  insertedId: UUID('d0000000-0000-4000-8000-000000008007')
}
```

`vn()` là hàm của lab (`lib/check.js`): nhận giờ Việt Nam, trả về BSON `Date` (UTC bên trong). Validator của `drivers` (MG-005) vẫn kiểm document này như mọi lần ghi khác.

`insertMany` nhận một mảng và trả về `insertedIds`. Khi không có `_id`, driver tự sinh `ObjectId` (chạy trên `scratch_c`; giá trị trên máy bạn sẽ khác):

```text
{
  acknowledged: true,
  insertedIds: {
    '0': ObjectId('6afb0f6a97616173250953e8'),
    '1': ObjectId('6afb0f6a97616173250953e9')
  }
}
```

**`insertMany` không phải «tất cả hoặc không gì cả».** Mặc định `ordered: true`: MongoDB thêm lần lượt và **dừng ở lỗi đầu tiên**, các document đã thêm trước đó vẫn ở lại. Thử thêm ba document `_id` 2, 1, 3 vào collection đã có `_id: 1` (chạy trên `scratch_c`):

```text
MongoBulkWriteError 11000 | E11000 duplicate key error collection: scratch_c.t index: _id_ dup key: { _id: 1 }
{"insertedCount":1,"matchedCount":0,"modifiedCount":0,"deletedCount":0,"upsertedCount":0,"upsertedIds":{},"insertedIds":{"0":2}}
```

Document 2 đã vào, document 1 bị từ chối, document 3 **không được thử**. Với `ordered: false`, MongoDB thử hết rồi báo mọi lỗi một lần — MG-009 dùng cách này khi backfill. Muốn nhiều document cùng thành công hoặc cùng huỷ, bạn cần transaction (Phần 4).

## 3. Mã chuyến từ bộ đếm: vì sao là 100015

MongoDB không có `IDENTITY` hay sequence. `_id` của chuyến là UUID — tốt cho máy, nhưng không ai đọc được qua điện thoại. Mã chuyến `code` (6 chữ số, có index unique `uq_trips_code`) cần một nguồn cấp số tăng dần. Lab dùng mẫu **counter** (bộ đếm): collection `counters` có một document cho mỗi dãy số, fixture `02` đặt sẵn `{ _id: 'trip_code', seq: 100014 }` bằng mã lớn nhất đã có.

```js
const counter = db.counters.findOneAndUpdate(
  { _id: 'trip_code' },
  { $inc: { seq: 1 } },
  { returnDocument: 'after', upsert: true },
);
const code = counter.seq;
```

```text
PASS mã chuyến mới = 100015
```

Ba chi tiết làm mẫu này đúng:

1. **`$inc` chạy trên server và là nguyên tử** với document `trip_code`. Hai request đặt chuyến cùng lúc không thể cùng đọc 100014 rồi cùng ghi 100015 như khi ứng dụng tự «đọc, cộng một, ghi lại».
2. **`returnDocument: 'after'`** trả về document *sau* khi tăng. Mặc định là `'before'`: bạn sẽ nhận 100014 — mã đã thuộc về chuyến khác.
3. **`upsert: true`** tạo document nếu chưa có. `$inc` trên trường chưa tồn tại coi giá trị cũ là 0, nên bộ đếm mới bắt đầu từ **1**. Chạy thử trên `scratch_c` với collection rỗng:

```text
{
  _id: 'trip_code',
  seq: 1
}
```

Gõ sai tên bộ đếm (`'trip_cdoe'`) là âm thầm có một dãy mới bắt đầu từ 1. Ở lab, validator `trips` (`code` tối thiểu 100000) sẽ chặn chuyến mang mã 1, nên lỗi lộ ra ngay ở lệnh insert kế tiếp. Không có validator thì bạn không được cảnh báo.

### Không có «lỗ hổng» do lệnh lỗi — nhưng vẫn có thể có

Ở lab PostgreSQL, chuyến tương ứng nhận mã **100029**; ở SQL Server là **100032**. Lý do: các câu INSERT sai trong tệp `03` đã xin số từ IDENTITY/sequence trước khi bị ràng buộc từ chối, và database không trả lại số đã cấp. Ở MongoDB, 37 ca sai trong `03-fixture-invalid.js` không đụng tới `counters` — bộ đếm chỉ tăng khi ứng dụng *gọi nó*. Vì vậy mã mới là **100015**, liền ngay sau 100014. Bỏ `03` khỏi lượt chạy, kết quả vẫn vậy.

Điều đó không có nghĩa dãy số luôn liền. Bộ đếm và chuyến là **hai document khác nhau, hai lệnh ghi riêng**. Nếu tăng bộ đếm xong mà insert chuyến thất bại, số đã cấp bị mất. Thử trên `scratch_c` với validator chép từ `toinoi.trips`, bộ đếm đang ở 100015; lần đầu chèn chuyến có cước âm:

```text
ma 100016 -> 121 Document failed validation
ma 100017 -> true
[ 100017 ]
```

Mã 100016 không thuộc chuyến nào. Kết luận giống hai series kia: `code` để **tra cứu** («cho tôi xem chuyến 100015»), không dùng để đếm chuyến hay kiểm tra «có mất chuyến nào không».

Một hệ quả khác: mọi chuyến mới trên toàn hệ thống cùng ghi vào **một document** `trip_code`. Đó là điểm nóng tiềm năng khi tải cao (hồ sơ Tới Nơi ước tính cao điểm khoảng 70 chuyến/giây). Bài này không đo; Phần 3 và Phần 4 mới có số liệu đo thật và các cách giảm tải như cấp mã theo khối.

## 4. Update có điều kiện: chuyển đúng bước, người đến sau khớp 0

Khách đặt chuyến bằng `insertOne`, kèm sự kiện đầu tiên trong mảng `events`:

```js
const id = UUID('70000000-0000-4000-8000-000000100015');
const t0 = vn('2026-11-03 08:40');
db.trips.insertOne({
  _id: id, code, kind: 'ride', city_id: 'SG', customer_id: C(1), status: 'requested',
  fare: 41000, requested_at: t0, updated_at: t0,
  events: [{ status: 'requested', at: t0, by: 'customer' }],
});
```

Chuyến chưa có tài xế nên **không có** trường `driver_id` — không ghi `null` (MG-003, validator cấm). Mỗi bước sau đó là một `updateOne` do hàm `step()` của lab tạo:

```js
function step(from, to, at, by, extra = {}) {
  const when = vn(at);
  return db.trips.updateOne(
    { _id: id, status: from },                                  // dòng nào + đang ở trạng thái nào
    { $set: { status: to, updated_at: when, ...extra },
      $push: { events: { status: to, at: when, by } } },
  );
}
```

Bộ lọc có hai phần: **document nào** (`_id`) và **đang ở trạng thái nào** (`status: from`). Bước đầu, hệ thống chuyển chuyến sang `searching`:

```text
PASS requested → searching: matched/modified = 1/1
```

Rồi hai tài xế cùng bấm nhận. 8007 bấm lúc 08:41:00, 8003 lúc 08:41:01, cả hai gửi cùng một lệnh `step('searching', 'accepted', …)`, chỉ khác `driver_id`:

```text
PASS 8007 nhận: matched = 1
PASS 8003 nhận muộn: matched (không ghi gì) = 0
PASS tài xế của chuyến = d0000000-0000-4000-8000-000000008007
```

Lệnh của 8003 **không có lỗi nào**. Chuyến đã là `accepted`, nên bộ lọc `{ status: 'searching' }` không còn khớp, MongoDB không ghi gì và trả về:

```text
{
  acknowledged: true,
  insertedId: null,
  matchedCount: 0,
  modifiedCount: 0,
  upsertedCount: 0
}
```

(Khối trên chạy lại trên `scratch_c` để in object đầy đủ; trong lab, `expectValue` chỉ in con số.) Trong query editor của DBStudio, chạy lại câu nhận chuyến của 8003 sau khi lab đã xong. DBStudio 0.1.7 không in object kết quả mà chỉ báo **Ảnh hưởng 0 dòng**: cùng một sự thật (không document nào bị sửa), nhưng không tách `matchedCount` với `modifiedCount`. Muốn phân biệt "không khớp" với "khớp nhưng giá trị không đổi", hãy đọc object kết quả trong mongosh hoặc driver:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-insert-update-delete/dbstudio-01.webp" alt="DBStudio chạy updateOne nhận chuyến muộn của tài xế 8003, kết quả Ảnh hưởng 0 dòng" width="1086" height="780" loading="lazy"></div><figcaption>DBStudio · update có điều kiện không khớp: Ảnh hưởng 0 dòng</figcaption></figure>

Ứng dụng phải **tự đọc `matchedCount`** để báo «chuyến đã có người nhận». Code chỉ bắt exception sẽ tưởng 8003 nhận thành công.

`matchedCount: 0` có hai nguyên nhân khác nhau: không có document mang `_id` đó, hoặc document có nhưng đã ở trạng thái khác. Muốn giải thích cho người dùng, đọc lại theo `_id` — nhưng chỉ để *giải thích*; quyết định ghi đã nằm trong bộ lọc. Cuối `04`, lab thử huỷ chuyến 100015 khi nó đã hoàn thành, với `from = 'searching'`:

```text
PASS huỷ chuyến đã hoàn thành: matched = 0
```

**Nếu bỏ `status` khỏi bộ lọc**, lệnh của 8003 khớp theo `_id`, ghi đè `driver_id` và `accepted_at` của 8007, đẩy thêm một sự kiện `accepted` thứ hai — và trả về `matchedCount: 1` như mọi lần thành công. Đây là lỗi **mất cập nhật** (lost update) kinh điển; điều kiện trạng thái trong bộ lọc chặn nó mà không cần khoá hay transaction.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Lab chạy tuần tự</span>
<p>Trong <code>04-lifecycle.js</code>, hai lệnh nhận chuyến chạy lần lượt trong một kết nối, nên bài chỉ chứng minh <em>logic</em> của bộ lọc. Khi hai lệnh thật sự đến cùng lúc, MongoDB vẫn bảo đảm mỗi lệnh ghi trên <em>một document</em> là nguyên tử: lệnh thứ hai kiểm bộ lọc trên trạng thái mà lệnh thứ nhất đã ghi. Kịch bản đồng thời thật, chạy lặp nhiều lần và kiểm bất biến, là MG-040 ở Phần 4.</p>
</div>

## 5. Toán tử cập nhật

Tham số thứ hai của `updateOne`/`updateMany` là một **document cập nhật** gồm các **toán tử cập nhật** (update operator) — khoá bắt đầu bằng `$` mô tả *thay đổi*, không phải trạng thái cuối:

| Toán tử | Làm gì | Dùng trong lab |
|---|---|---|
| `$set` | Gán giá trị cho trường; tạo trường nếu chưa có | `status`, `updated_at`, `driver_id`, `accepted_at` |
| `$inc` | Cộng (hoặc trừ, với số âm) vào trường số; trường chưa có coi như 0 | Bộ đếm `seq` |
| `$push` | Thêm phần tử vào cuối mảng | Sự kiện vào `events` |
| `$unset` | Xoá trường khỏi document | Rollback biển số ở MG-009 |
| `$setOnInsert` | Chỉ gán khi lệnh upsert **tạo mới** document; cập nhật thường thì bỏ qua | Bộ đếm `parcel_code` (mục 8) |

Ngoài ra còn `$min`/`$max` (chỉ ghi khi giá trị mới nhỏ hơn/lớn hơn), `$addToSet` (thêm nếu chưa có), `$pull` (bỏ phần tử khớp điều kiện), `$currentDate`… Một lệnh kết hợp được nhiều toán tử, miễn chúng không cùng đụng một trường.

Quên toán tử — truyền thẳng `{ status: 'accepted' }` — thì mongosh từ chối ngay ở phía client (chạy trên `scratch_c`):

```text
MongoInvalidArgumentError | Update document requires atomic operators
```

Đây là hàng rào có chủ đích: một document không có `$` nghĩa là **thay toàn bộ**, việc của `replaceOne` (mục 9).

`$unset` xoá hẳn trường, khác với `$set: { x: null }` (trường vẫn còn, giá trị `null`). Trên `scratch_c`, `updateOne({ _id: 2 }, { $unset: { b: '' } })` biến `{ _id: 2, a: 1, b: 2 }` thành `{ _id: 2, a: 1 }`. Giá trị bạn đưa cho `$unset` không quan trọng; quy ước hay dùng là chuỗi rỗng.

Một chi tiết nhỏ khi đọc document sau nhiều lần `$set`: **trường mới được thêm vào cuối document**. Đọc chuyến 100015 sau khi lab chạy xong, `driver_id`, `accepted_at`, `completed_at` nằm *sau* mảng `events`, dù các chuyến của fixture có chúng ở giữa. Thứ tự trường không ảnh hưởng truy vấn hay validator; đừng viết code dựa vào nó.

## 6. Một document, một thao tác nguyên tử

Mỗi bước của `step()` làm **hai việc** trong một lệnh: đổi `status` bằng `$set` và thêm sự kiện bằng `$push`. Vì cả hai nằm trong cùng một document, MongoDB bảo đảm chúng **cùng xảy ra hoặc cùng không**: không ai đọc được chuyến đã `accepted` mà lịch sử chưa có dòng `accepted`, và sập server giữa chừng cũng không để lại trạng thái nửa vời. Tài liệu MongoDB gọi đây là tính nguyên tử ở mức một document (single-document atomicity).

Validator của `trips` (MG-005) kiểm document **sau khi áp dụng** cả hai toán tử. Chuyển sang `accepted` mà quên `driver_id` hay `accepted_at` trong `extra` thì lệnh bị từ chối với lỗi 121 (điều kiện liên trường #2, #3), và cả `$push` lẫn `$set` đều không được ghi.

So với hai series quan hệ: PostgreSQL và SQL Server giữ lịch sử ở bảng `trip_events` riêng, nên mỗi bước là **một UPDATE `trips` + một INSERT `trip_events`**, và phải bọc trong transaction để không lệch nhau. MongoDB làm được điều tương tự mà không cần transaction — *vì* chúng ta đã chọn nhúng lịch sử vào chuyến. Lựa chọn đó có giá: mảng `events` lớn dần theo số bước, và mọi cách đọc lịch sử phải đi qua document chuyến. MG-008 cân nhắc khi nào nhúng là đúng.

### Tài xế chuyển busy: document khác, không nguyên tử

Ngay sau khi nhận chuyến, lab chuyển tài xế 8007 sang `busy`:

```js
db.drivers.updateOne({ _id: D(8007), status: 'online' }, { $set: { status: 'busy' } });
```

Lệnh này cũng có điều kiện, nhưng nó ghi vào **một document khác, trong collection khác**. Giữa lệnh nhận chuyến và lệnh này, nếu ứng dụng sập, chuyến 100015 đã `accepted` với tài xế 8007 trong khi 8007 vẫn `online` — và có thể được gán thêm chuyến khác. Tệp `07-checks.js` có hẳn một kiểm tra cho bất biến «tài xế `busy` ⇔ có đúng một chuyến đang chạy» để bắt trường hợp này. Muốn hai document đổi cùng nhau, bạn cần transaction nhiều document hoặc thiết kế khác — chủ đề của MG-034 và MG-039 ở Phần 4.

## 7. findOneAndUpdate: nhận lại document vừa ghi

Ba bước giữa (`picked_up`, `in_transit`, `arriving`) dùng `step()` như trên. Bước cuối, app tài xế cần hiển thị luôn chuyến vừa hoàn thành: cước, giờ xong, số sự kiện. Thay vì `updateOne` rồi `findOne` (thêm một vòng gọi mạng, và có thể đọc phải một lần ghi khác xen giữa), lab dùng `findOneAndUpdate`:

```js
const done = db.trips.findOneAndUpdate(
  { _id: id, status: 'arriving' },
  { $set: { status: 'completed', completed_at: vn('2026-11-03 09:11'), updated_at: vn('2026-11-03 09:11') },
    $push: { events: { status: 'completed', at: vn('2026-11-03 09:11'), by: 'driver' } } },
  { returnDocument: 'after',
    projection: { _id: 0, code: 1, status: 1, fare: 1, completed_at: 1, n_events: { $size: '$events' } } },
);
```

```text
code|status|fare|completed_at|n_events
----|------|----|------------|--------
100015|completed|41000|2026-11-03 09:11:00 +07:00|7
(1 document)
```

`projection` chọn trường trả về như của `find` (MG-007), và từ MongoDB 4.4 nhận được cả biểu thức aggregation: `n_events` là `$size` của mảng `events`, tính trên document *sau* khi đã `$push`. Đây là phiên bản MongoDB của `UPDATE … RETURNING` (PostgreSQL) và `OUTPUT inserted.*` (SQL Server). Khác biệt: `findOneAndUpdate` chỉ làm việc với **một** document; không có «`updateMany` trả về mọi document đã sửa».

Khi bộ lọc không khớp, `findOneAndUpdate` trả về `null` (chạy trên `scratch_c`) — với phương thức này, `null` đóng vai `matchedCount: 0`.

### Dòng thời gian của chuyến 100015

Mở mảng `events` thành từng dòng bằng `$unwind` (aggregation, học kỹ ở Phần 2):

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
PASS số sự kiện = 7
PASS phút từ đặt tới hoàn thành = 31
```

Bảy sự kiện, đặt lúc 08:40, hoàn thành lúc 09:11: **31 phút**. Không có sự kiện `accepted` thứ hai của 8003 — bằng chứng lệnh nhận muộn không ghi gì. Trong DBStudio, chạy `db.trips.find({ code: 100015 }, { _id: 0, code: 1, status: 1, driver_id: 1, events: 1 })` và xem ở chế độ **Tài liệu**: mảng `events` nằm ngay trong chuyến, bảy phần tử theo đúng thứ tự `$push`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-insert-update-delete/dbstudio-02.webp" alt="DBStudio hiện chuyến 100015 với mảng events bảy phần tử từ requested tới completed" width="1086" height="1075" loading="lazy"></div><figcaption>DBStudio · bảy sự kiện nhúng trong chuyến 100015 (giờ UTC); <code>driver_id</code> nằm cuối vì được <code>$set</code> sau</figcaption></figure>

Sau khi chuyến xong, lab trả tài xế về `online` — lại một `updateOne` có điều kiện `status: 'busy'` trên document tài xế, với cùng giới hạn đã nói ở mục 6.

## 8. updateMany, modifiedCount 0 và upsert

`updateMany` áp dụng một document cập nhật cho **mọi** document khớp bộ lọc. Lab cố tình đặt lại trạng thái `offline` cho tài xế Đà Nẵng đang `offline`:

```js
db.drivers.updateMany({ city_id: 'DN', status: 'offline' }, { $set: { status: 'offline' } });
```

```text
PASS updateMany: matched/modified (giá trị không đổi ⇒ không sửa) = 1/0
```

Tài xế 8006 khớp (`matchedCount: 1`) nhưng giá trị mới bằng giá trị cũ, nên MongoDB không ghi (`modifiedCount: 0`). PostgreSQL và SQL Server báo **1 dòng** cho cùng câu lệnh: chúng vẫn ghi lại dòng dù giá trị không đổi. Hệ quả thực tế: muốn biết «bộ lọc có tìm thấy document không», đọc `matchedCount`; muốn biết «có gì thật sự đổi không», đọc `modifiedCount`. Dùng nhầm `modifiedCount` để kiểm tra «chuyến có tồn tại không» sẽ báo sai khi lệnh lặp lại một giá trị đã có — chuyện rất thường gặp khi request được gửi lại.

**Upsert** (update + insert) là tuỳ chọn `upsert: true`: nếu không document nào khớp, MongoDB tạo document mới từ các điều kiện bằng của bộ lọc cộng với document cập nhật. Lab tạo thử bộ đếm cho bưu kiện:

```js
db.counters.updateOne({ _id: 'parcel_code' }, { $setOnInsert: { seq: 500000 } }, { upsert: true });
```

```text
PASS upsert tạo bộ đếm mới: upsertedCount = 1
```

`$setOnInsert` chỉ có tác dụng khi document được tạo mới. Chạy lệnh tương tự lần thứ hai (trên `scratch_c`, với `seq: 1`) không ghi đè giá trị đã có:

```text
{
  acknowledged: true,
  insertedId: null,
  matchedCount: 1,
  modifiedCount: 0,
  upsertedCount: 0
}
```

Đó chính là cách «tạo nếu chưa có, giữ nguyên nếu đã có» mà không cần đọc trước. Một lưu ý quan trọng: hai upsert đồng thời với cùng bộ lọc **có thể cùng không tìm thấy** document và cùng tạo mới. Tài liệu MongoDB khuyên đặt **index unique** trên trường của bộ lọc để lần thứ hai bị từ chối thay vì tạo bản trùng; ở đây `_id` vốn đã unique. Nhập dữ liệu bằng upsert và `$merge` là bài MG-017.

## 9. replaceOne: thay cả document, quên trường là mất trường

`replaceOne(filter, doc)` thay **toàn bộ** nội dung document khớp bằng `doc` (giữ nguyên `_id`). Không có toán tử nào: trường bạn không viết ra sẽ biến mất. Lab giả lập một màn hình «sửa tên khách» viết vội: đọc khách 6, rồi thay bằng document chỉ có số điện thoại và họ tên:

```js
const before = db.customers.findOne({ _id: C(6) });
db.customers.replaceOne({ _id: C(6) }, { phone: before.phone, full_name: 'Đặng Ngọc Hà' });
```

```text
PASS replaceOne thiếu created_at → lỗi 121 (created_at)
```

Validator của `customers` yêu cầu `created_at`, nên lệnh bị từ chối. Chạy lại trên `scratch_c` với validator chép từ `toinoi.customers` để xem `errInfo` đầy đủ (rút gọn phần `specifiedAs`):

```text
MongoServerError 121 Document failed validation
{
  failingDocumentId: UUID('c0000000-0000-4000-8000-000000000006'),
  details: {
    operatorName: '$jsonSchema',
    schemaRulesNotSatisfied: [
      {
        operatorName: 'required',
        ...
        missingProperties: [
          'created_at'
        ]
      }
    ]
  }
}
```

Validator đã cứu dữ liệu ở đây. Trên collection không có validator, cùng lệnh **thành công** và ngày tạo tài khoản mất vĩnh viễn — thử trên `scratch_c`, `replaceOne({ _id: 1 }, { status: 'x' })` trả `matchedCount: 1, modifiedCount: 1` và document chỉ còn `{ _id: 1, status: 'x' }`. Quy tắc thực tế: sửa vài trường thì dùng `updateOne` + `$set`; chỉ dùng `replaceOne` khi ứng dụng thật sự sở hữu và gửi lên **trọn** document.

## 10. Xoá: không có khoá ngoại nào chặn

Khách 6 (chị Đặng Ngọc Hà) có hai chuyến: 100007 và 100014. Trong PostgreSQL hay SQL Server, khoá ngoại `trips.customer_id → customers` chặn câu `DELETE` khách này. MongoDB **không có khoá ngoại** (MG-005). Lab xoá bằng `findOneAndDelete`, phương thức xoá một document và trả lại nội dung của nó:

```js
const removed = db.customers.findOneAndDelete({ _id: C(6) });
```

```text
PASS đã xoá khách = Đặng Ngọc Hà
PASS chuyến trỏ tới khách vừa xoá = 2
```

Xoá thành công, và hai chuyến giờ là **chuyến mồ côi** (orphan): `customer_id` trỏ tới một khách không còn tồn tại. Không lỗi, không cảnh báo. `deleteOne` cũng cho kết quả tương tự, chỉ trả về `{ acknowledged: true, deletedCount: 1 }`.

Vì `findOneAndDelete` trả lại nguyên document đã xoá, lab khôi phục được đúng như cũ — cùng `_id`, cùng `created_at`:

```js
db.customers.insertOne(removed);
```

```text
PASS khôi phục khách 6 = 1
```

Đây là một lý do nên dùng `findOneAndDelete` cho thao tác xoá do người vận hành thực hiện: giữ lại bản đã xoá trong log hay collection lưu trữ thì «xoá nhầm» còn đường lui. Khôi phục ở quy mô lớn hơn là việc của backup và oplog (Phần 5).

### Xoá an toàn ở tầng ứng dụng — và khe hở của nó

Khi database không giữ tham chiếu, ứng dụng phải tự kiểm. Lab viết một hàm mẫu:

```js
function deleteCustomerIfUnused(cid) {
  if (db.trips.countDocuments({ customer_id: cid }, { limit: 1 }) > 0) return 'bị chặn: còn chuyến';
  return `đã xoá ${db.customers.deleteOne({ _id: cid }).deletedCount}`;
}
```

```text
PASS xoá an toàn khách 6 = bị chặn: còn chuyến
```

`{ limit: 1 }` cho `countDocuments` dừng ngay khi thấy một chuyến — chỉ cần biết «có hay không». Nhưng hàm này **không an toàn khi chạy đồng thời**: giữa lúc đếm (0 chuyến) và lúc xoá, một request đặt chuyến mới cho khách đó có thể chen vào, và bạn lại có chuyến mồ côi. Khoá ngoại của database quan hệ đóng khe hở này; ở MongoDB, bạn cần transaction, xoá mềm (đánh dấu `deleted_at` thay vì xoá) hoặc chấp nhận và kiểm định kỳ như `07-checks.js` làm bằng `$lookup`. Phần 4 quay lại bài toán này.

Cuối `04`, số document khớp đúng kỳ vọng — thêm một tài xế, thêm một chuyến, khách giữ nguyên:

```text
customers|drivers|trips|counter
---------|-------|-----|-------
6|7|16|100015
(1 document)
```

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">deleteMany({}) xoá sạch collection</span>
<p>Bộ lọc rỗng <code>{}</code> khớp mọi document. <code>deleteMany({})</code> không hỏi lại và không có khoá ngoại nào ngăn. Trước khi xoá theo bộ lọc, chạy <code>countDocuments</code> với đúng bộ lọc đó và so với con số bạn nghĩ; trong DBStudio, kiểm database đang chọn trước khi chạy.</p>
</div>

## 11. So với PostgreSQL và SQL Server

| Việc | PostgreSQL | SQL Server | MongoDB (lab này) |
|---|---|---|---|
| Lấy giá trị database tạo ra | `RETURNING` | `OUTPUT inserted.*` | `insertedId(s)`; `findOneAndUpdate` cho giá trị khác |
| Mã tăng dần | IDENTITY/sequence: 100029, số bị tiêu bởi INSERT lỗi | IDENTITY: 100032 | Bộ đếm `$inc`: 100015; chỉ mất số khi tăng xong mà insert lỗi |
| «Không khớp gì» | `UPDATE 0` | 0 rows affected | `matchedCount: 0` |
| Ghi giá trị trùng giá trị cũ | `UPDATE 1` | 1 row affected | `matchedCount: 1`, `modifiedCount: 0` |
| Đổi trạng thái + ghi lịch sử | UPDATE + INSERT trong transaction | UPDATE + INSERT trong transaction (OUTPUT INTO bị chặn bởi CHECK/FK) | Một `updateOne` với `$set` + `$push`, nguyên tử, không cần transaction |
| Thay cả dòng/document | Không có lệnh tương ứng; UPDATE giữ cột không nhắc tới | Như PostgreSQL | `replaceOne`: trường không nhắc tới bị mất |
| Upsert | `INSERT … ON CONFLICT` | `MERGE` hoặc kiểm rồi ghi | `upsert: true`, nên có index unique |
| Xoá cha còn con | Khoá ngoại chặn | Khoá ngoại chặn | Không chặn, để lại document mồ côi |

Điểm đáng nhớ: MongoDB dời một phần trách nhiệm từ database sang **thiết kế document** (nhúng để có nguyên tử) và sang **ứng dụng** (đọc kết quả ghi, tự giữ tham chiếu).

## 12. Những hiểu nhầm thường gặp

- **«Không có exception là đã ghi.»** `updateOne` khớp 0 document vẫn thành công. Luôn đọc `matchedCount` (hoặc kiểm `null` với `findOneAnd*`) ở mọi lệnh ghi có điều kiện.
- **«`modifiedCount: 0` nghĩa là không tìm thấy.»** Có thể đã tìm thấy nhưng giá trị không đổi. Kiểm sự tồn tại bằng `matchedCount`.
- **«`findOneAndUpdate` trả về document mới.»** Mặc định trả về bản **trước** khi sửa. Với bộ đếm, quên `returnDocument: 'after'` là cấp trùng mã.
- **«`insertMany` lỗi thì không có gì được thêm.»** Các document trước lỗi đầu tiên đã ở lại. Cần tất cả hoặc không gì cả thì dùng transaction.
- **«MongoDB có transaction nên cứ bọc mọi thứ trong transaction.»** Khi dữ liệu cần đổi cùng nhau nằm trong một document, một lệnh update đã nguyên tử; transaction tốn thêm chi phí và có giới hạn riêng (MG-034, MG-039).
- **«Đã nhúng lịch sử thì mọi thứ về chuyến đều nguyên tử.»** Chỉ những gì nằm trong document chuyến. Trạng thái tài xế, bộ đếm mã, thông tin khách nằm ở document khác.
- **«Xoá khách đang có chuyến sẽ bị báo lỗi.»** Không có khoá ngoại: xoá thành công, chuyến thành mồ côi.

## 13. Bài tập

**Bài 1.** Hai lệnh sau chạy trên chuyến 100015 sau khi lab đã xong. Mỗi lệnh trả về `matchedCount` và `modifiedCount` bao nhiêu? (a) `updateOne({ code: 100015, status: 'completed' }, { $set: { fare: 41000 } })`; (b) `updateOne({ code: 100015, status: 'arriving' }, { $set: { status: 'completed' } })`.

<details><summary><b>Lời giải</b></summary><p>(a) <code>1/0</code>: chuyến đang <code>completed</code> nên khớp, nhưng cước đã là 41000, không có gì đổi. (b) <code>0/0</code>: chuyến không còn ở <code>arriving</code>, bộ lọc không khớp. Cả hai đều không ném lỗi; chỉ (b) cho biết «bước chuyển này không còn hợp lệ».</p></details>

**Bài 2.** Một lập trình viên viết bộ đếm như sau: `const c = db.counters.findOne({ _id: 'trip_code' }); db.counters.updateOne({ _id: 'trip_code' }, { $set: { seq: c.seq + 1 } }); return c.seq + 1;`. Chỉ ra lỗi khi hai request đặt chuyến chạy cùng lúc và sửa lại bằng một lệnh.

<details><summary><b>Lời giải</b></summary><p>Hai request có thể cùng đọc <code>seq = 100015</code>, cùng ghi 100016 và cùng trả về 100016: hai chuyến xin cùng một mã, một trong hai sẽ bị index unique <code>uq_trips_code</code> từ chối (lỗi 11000). Sửa: <code>db.counters.findOneAndUpdate({ _id: 'trip_code' }, { $inc: { seq: 1 } }, { returnDocument: 'after' }).seq</code> — đọc và tăng trong một thao tác nguyên tử trên server.</p></details>

**Bài 3.** Viết lệnh cho khách huỷ một chuyến **chỉ khi** nó còn ở `requested` hoặc `searching`, đồng thời ghi sự kiện `cancelled` do `customer` thực hiện lúc 09:00 ngày 03/11 (giờ VN). Ứng dụng nhận về `matchedCount: 0` thì nên báo khách điều gì?

<details><summary><b>Lời giải</b></summary><p><code>db.trips.updateOne({ _id: id, status: { $in: ['requested', 'searching'] } }, { $set: { status: 'cancelled', updated_at: vn('2026-11-03 09:00') }, $push: { events: { status: 'cancelled', at: vn('2026-11-03 09:00'), by: 'customer' } } })</code>. <code>matchedCount: 0</code> nghĩa là chuyến không tồn tại hoặc đã có tài xế nhận (hoặc đã kết thúc); đọc lại theo <code>_id</code> để báo đúng, ví dụ «tài xế đã nhận chuyến, vui lòng huỷ qua tổng đài». Quyết định không cho huỷ đã nằm sẵn trong bộ lọc.</p></details>

**Bài 4.** Đội vận hành muốn một nút «Sửa thông tin khách» gửi lên toàn bộ form (số điện thoại, họ tên). Vì sao `replaceOne` với dữ liệu form là lựa chọn nguy hiểm, và nên viết lệnh thế nào?

<details><summary><b>Lời giải</b></summary><p>Form không có <code>created_at</code> (và mọi trường thêm sau này, ví dụ <code>schema_version</code> ở MG-009), nên <code>replaceOne</code> sẽ xoá các trường đó. Lab có validator nên bị lỗi 121; không có validator thì dữ liệu mất mà không ai hay. Dùng <code>updateOne({ _id }, { $set: { phone, full_name } })</code>: chỉ đổi đúng hai trường form sở hữu.</p></details>

## 14. Đọc tiếp

- [MongoDB CRUD Operations](https://www.mongodb.com/docs/manual/crud/) — tổng quan insert, find, update, delete.
- [Update Operators](https://www.mongodb.com/docs/manual/reference/operator/update/) — danh sách đầy đủ `$set`, `$inc`, `$push`, `$setOnInsert`…
- [db.collection.findOneAndUpdate()](https://www.mongodb.com/docs/manual/reference/method/db.collection.findOneAndUpdate/) — `returnDocument`, `projection`, `upsert`.
- [db.collection.updateOne()](https://www.mongodb.com/docs/manual/reference/method/db.collection.updateOne/) — kết quả trả về và hành vi upsert, kể cả lưu ý về index unique.
- [Atomicity and Transactions](https://www.mongodb.com/docs/manual/core/write-operations-atomicity/) — nguyên tử ở mức một document.
- [PG-006](/articles/postgresql-insert-update-delete-returning) và [MS-006](/articles/mssql-insert-update-delete-output): cùng vòng đời chuyến trên PostgreSQL và SQL Server.

Bài tiếp theo: [MG-007 · find: lọc, projection, sắp xếp, phân trang và collation](/articles/mongodb-find-loc-sap-xep-collation).
