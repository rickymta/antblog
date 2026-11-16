---
slug: mongodb-thuc-hanh-mo-hinh-chuyen-xe
title: "Thực hành: mô hình chuyến xe trên MongoDB có validator, fixture và kiểm tra bất biến"
summary: "Tới Nơi sắp đưa dữ liệu chuyến xe lên MongoDB 8.0 và trưởng nhóm vận hành không chấp nhận câu “MongoDB không cần schema”. Bài thực hành cuối Phần 1: bàn giao collection và validator, fixture hợp lệ/sai, 12 kiểm tra bất biến liên collection, đối soát doanh thu và bằng chứng chạy lại được."
category: mongodb
level: advanced
tags: [mongodb, mg-010, thuc-hanh, validator, fixture, lookup, toi-noi]
language: vi
---

Đội nền tảng Tới Nơi chuẩn bị đưa dữ liệu chuyến xe lên MongoDB 8.0. Trong buổi duyệt thiết kế, trưởng nhóm vận hành đặt đúng câu hỏi từng đặt cho đội PostgreSQL: "Làm sao tôi biết một chuyến `completed` mà không có tài xế sẽ không lọt vào?". Rồi chị hỏi thêm câu mà đội PostgreSQL không phải trả lời: "MongoDB không có khoá ngoại. Vậy ai bảo đảm chuyến không trỏ tới một khách đã bị xoá?"

Câu trả lời "MongoDB linh hoạt, ứng dụng tự kiểm" không được chấp nhận. Thứ cần bàn giao là **bằng chứng**: chạy từ database trống, dữ liệu đúng đi qua, từng loại dữ liệu sai bị chặn với đúng mã lỗi và đúng điều kiện, những gì validator **không** chặn được thì ghi rõ và có truy vấn canh, và con số doanh thu khớp tới từng đồng.

Đây là bài thực hành cuối Phần 1. Bạn nhận đề bài, tự làm, rồi mới mở lời giải mẫu dựa trên lab và tự chấm theo tiêu chí ở mục 5.

<div class="tkh-callout">
<span class="tkh-callout-title">MG-010 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: toàn bộ MG-001 … MG-009 (danh sách ở mục 8), đặc biệt <a href="/articles/mongodb-schema-validation-unique-index">MG-005 · Schema validation và index unique</a> và <a href="/articles/mongodb-nhung-hay-tham-chieu">MG-008 · Nhúng hay tham chiếu</a>. Lab: <code>examples/toi-noi-mongodb/p01</code> (8 file, <code>OUTPUT.txt</code>, <code>test/p01.test.mjs</code>). Sau bài này, bạn bàn giao được collection, validator, fixture hợp lệ/sai, kiểm tra bất biến và bằng chứng chạy cho mô hình chuyến xe trên MongoDB.</p>
</div>

## 1. Đề bài

### Bối cảnh

Mô hình phục vụ ba dịch vụ của [Tới Nơi](/articles/du-an-mau-toi-noi): đặt xe (`ride`), giao đồ ăn (`food`), giao bưu kiện (`parcel`). Dữ liệu giống lab PostgreSQL và SQL Server để đối chiếu: 3 thành phố, 6 khách, 6 tài xế, 3 quán, 15 chuyến. Nhưng mô hình phải theo cơ chế của MongoDB, không phải bản dịch bảng sang collection:

- lịch sử trạng thái **nhúng** trong chuyến, menu **nhúng** trong quán (MG-008);
- khách, tài xế, quán **tham chiếu** bằng UUID; thành phố tham chiếu bằng mã hai chữ;
- tiền là số nguyên đồng (BSON `int`/`long`), thời điểm là BSON `Date`, hiển thị theo giờ Việt Nam (MG-004);
- trường không áp dụng thì **vắng mặt**, không ghi `null` (MG-003).

Mọi tên người, số điện thoại, biển số đều hư cấu.

### Quy tắc nghiệp vụ phải mã hoá

| Mã | Quy tắc |
|---|---|
| Q1 | Thành phố có mã 2 chữ in hoa, tên không rỗng, múi giờ `Asia/Ho_Chi_Minh` |
| Q2 | Khách có số điện thoại dạng chuỗi `0` + 9 chữ số, **duy nhất**; họ tên không rỗng |
| Q3 | Tài xế có mã nội bộ số nguyên 1000–99999, **duy nhất**; loại xe `bike`/`car`; trạng thái `offline`/`online`/`busy` |
| Q4 | Quán có menu tối đa 200 món, mỗi món có `sku`, tên, giá nguyên không âm |
| Q5 | Chuyến có loại thuộc 3 dịch vụ, trạng thái thuộc 8 trạng thái vòng đời, mã số nguyên ≥ 100000 **duy nhất** |
| Q6 | Cước là số nguyên đồng, 0 … 10.000.000 (0 đ hợp lệ: khuyến mãi); không double, không chuỗi, không Decimal128 |
| Q7 | Không có trường lạ; trường vắng không được thay bằng `null` |
| Q8 | Đồ ăn có quán; loại khác không có quán |
| Q9 | `requested`/`searching` chưa có tài xế; `accepted` … `completed` phải có tài xế; `cancelled` tuỳ |
| Q10 | Có tài xế khi và chỉ khi có `accepted_at`; có `completed_at` khi và chỉ khi `completed` |
| Q11 | `accepted_at` ≥ `requested_at`, `completed_at` ≥ `accepted_at`, `updated_at` ≥ `requested_at` |
| Q12 | Lịch sử trạng thái nhúng tối đa 20 sự kiện; sự kiện cuối khớp trạng thái hiện tại; thời gian tăng dần |
| Q13 | Mọi tham chiếu (khách, tài xế, quán, thành phố) trỏ tới document có thật; quán và tài xế cùng thành phố với chuyến |
| Q14 | Tài xế `busy` khi và chỉ khi có đúng một chuyến đang chạy; mã chuyến cấp từ bộ đếm, bộ đếm không tụt sau mã lớn nhất |

### Sản phẩm bàn giao

| Mã | Sản phẩm | Yêu cầu tối thiểu |
|---|---|---|
| D1 | Collection, validator, index | Validator `$jsonSchema` cho 5 collection nghiệp vụ, quy tắc liên trường của `trips` tách thành từng điều kiện đánh số trong `$and`; index unique cho khoá tự nhiên; ghi lý do chọn kiểu |
| D2 | Fixture hợp lệ | Chạy lại được từ database trống; UUID và thời gian cố định; phủ 8 trạng thái × 3 loại; có ca biên (cước 0, huỷ trước/sau khi nhận, qua nửa đêm giờ VN) |
| D3 | Fixture sai | Mỗi ca vi phạm **đúng một** quy tắc, ghi trước mã lỗi (121 hoặc 11000) và quy tắc/điều kiện mong đợi; có ca update; ghi lại các **bẫy** MongoDB không báo lỗi |
| D4 | Kiểm tra bất biến | Mỗi kiểm tra trả danh sách vi phạm, đúng là rỗng; phủ Q12–Q14 và kiểm toàn bộ document với validator hiện tại; có đối soát doanh thu và kiểm kê |
| D5 | Bằng chứng | Kết quả chạy từ database trống, tổng kết 0 FAIL, so khớp với kết quả đã lưu; ghi phiên bản MongoDB và những gì **chưa** kiểm |

Tuỳ chọn cộng điểm: một migration expand → backfill → contract kèm hoàn tác chạy lặp được, theo [MG-009](/articles/mongodb-schema-migration-tuong-thich).

## 2. Gợi ý trước khi làm

- **Chọn trước: luật nào validator giữ, luật nào truy vấn canh.** Validator chỉ nhìn một document. Q13 và Q14 cần đọc collection khác nên không thể nằm trong validator; Q12 một phần (thời gian tăng dần) rất khó diễn đạt trong `$jsonSchema`.
- **Viết quy tắc liên trường thành `$or` của các trường hợp hợp lệ**, mỗi quy tắc một phần tử trong `$and` gốc. Khi vi phạm, `errInfo.details.clausesNotSatisfied` cho biết **chỉ số** điều kiện, nên fixture sai kiểm được "đúng điều kiện #4" chứ không chỉ "lỗi 121".
- **So hai trường của cùng document phải dùng `$expr`.** `$jsonSchema` không so được `completed_at` với `accepted_at`. Nhớ xử lý trường vắng: `$type` của trường vắng là `'missing'`.
- **`additionalProperties: false` tính cả `_id`.** Quên khai `_id` trong `properties` thì không ghi được document nào.
- **Số trong mongosh.** Số nguyên viết thẳng được lưu là `int`; `45000.5` là `double`. Kiểm cả hai chiều trong fixture sai.
- **Document mẫu phải hợp lệ thật.** Chèn thử document mẫu rồi xoá trước khi viết ca sai; nếu mẫu sai sẵn, mọi ca sai đều "PASS" vô nghĩa.
- **Một ca sai, một vi phạm.** `status: 'completed'` không kèm tài xế vi phạm cả #2, #3 và #4. Muốn thử #4 thì phải thêm tài xế và `accepted_at`.

Chạy lab để có môi trường đối chiếu ([MG-002](/articles/mongodb-cai-dat-docker-dbstudio) hướng dẫn chi tiết):

```sh
cd examples/toi-noi-mongodb
npm run db:up        # MongoDB 8.0 replica set một node trong Docker, chờ "healthy"
npm run p01          # 00 → 07 trên database toinoi mới
npm test             # chạy lại p01, so OUTPUT.txt, thử rollback, thí nghiệm MG-004
```

Lab không bật xác thực và chỉ mở cổng `27018` trên `127.0.0.1`; Phần 5 mới bật xác thực.

## 3. Lời giải mẫu

### 3.1 D1 · Collection, validator, index (`01-schema.js`)

<details><summary><b>Lời giải</b></summary>
<p>Sáu collection: <code>cities</code>, <code>customers</code>, <code>drivers</code>, <code>merchants</code>, <code>trips</code> có validator; <code>counters</code> (bộ đếm mã chuyến) không có. Validator của <code>trips</code> là một <code>$and</code> năm phần tử sau <code>$jsonSchema</code>:</p>
<table>
<thead><tr><th>Điều kiện</th><th>Quy tắc</th><th>Cách viết</th></tr></thead>
<tbody>
<tr><td>#0</td><td>Q5, Q6, Q7, Q12 (trần 20)</td><td><code>$jsonSchema</code>: <code>required</code>, <code>enum</code>, <code>bsonType: ['int','long']</code> cho <code>fare</code>, <code>additionalProperties: false</code>, <code>events.maxItems: 20</code></td></tr>
<tr><td>#1</td><td>Q8</td><td><code>$or</code>: food có <code>merchant_id</code> / ride, parcel không có</td></tr>
<tr><td>#2</td><td>Q9</td><td><code>$or</code> ba nhánh theo nhóm trạng thái</td></tr>
<tr><td>#3</td><td>Q10 (tài xế ⇔ <code>accepted_at</code>)</td><td><code>$or</code> hai nhánh có/không</td></tr>
<tr><td>#4</td><td>Q10 (<code>completed</code> ⇔ <code>completed_at</code>)</td><td><code>$or</code> hai nhánh</td></tr>
<tr><td>#5</td><td>Q11</td><td><code>$expr</code>, mỗi so sánh bỏ qua khi trường vắng</td></tr>
</tbody>
</table>
<p>Index unique giữ khoá tự nhiên, hai index thường phục vụ truy vấn P01 (Phần 3 mới đo và chọn index bài bản):</p>
<pre><code class="language-text">collection|name|key|unique
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
(10 document)
PASS số index (kể cả _id) = 10</code></pre>
<p>Lý do chọn kiểu (data dictionary rút gọn): <code>_id</code> UUID (binary subtype 4) để cùng giá trị với lab PostgreSQL/SQL Server; <code>code</code> int cho người đọc, cấp bằng <code>counters</code> + <code>findOneAndUpdate</code> <code>$inc</code> vì MongoDB không có IDENTITY/SEQUENCE; tiền <code>int</code>/<code>long</code> vì cộng chính xác và <code>$sum</code> tự lên <code>long</code> khi cần (MG-004); thời điểm <code>date</code> (UTC); <code>phone</code> chuỗi để giữ số 0 đầu.</p>
</details>

### 3.2 D2 · Fixture hợp lệ (`02-fixture-valid.js`)

<details><summary><b>Lời giải</b></summary>
<p>UUID ghi sẵn theo mẫu dễ đọc (<code>c…</code> khách, <code>d…800x</code> tài xế, <code>e…</code> quán, <code>7…100000</code> chuyến), thời gian viết theo giờ VN qua <code>vn()</code>. Ảnh chụp dữ liệu lúc 08:30 ngày 03/11: ngày 02/11 đã xong, sáng 03/11 còn 6 chuyến đang chạy ở 6 trạng thái khác nhau. Phủ đủ 8 trạng thái và 3 loại:</p>
<pre><code class="language-text">_id|ride|food|parcel|tong
---|----|----|------|----
accepted|1|0|0|1
arriving|0|0|1|1
cancelled|1|1|0|2
completed|3|2|2|7
in_transit|1|0|0|1
picked_up|0|1|0|1
requested|1|0|0|1
searching|0|1|0|1
(8 document)
PASS số trạng thái khác nhau = 8</code></pre>
<p>Ca biên: 100008 cước 0 đ; 100003 huỷ <b>sau</b> khi tài xế nhận (giữ <code>driver_id</code>, <code>accepted_at</code>), 100006 huỷ <b>trước</b> (không có hai trường đó); 100007 đặt 23:50 ngày 02/11 và 100008 đặt 00:10 ngày 03/11 giờ VN, vẫn là ngày 02/11 theo UTC; 100003 và 100004 cùng giờ đặt để thử khoá phụ khi sắp xếp. Tài xế đang có chuyến chạy thì <code>busy</code>, để Q14 đúng ngay từ đầu. Bộ đếm bắt đầu ở 100014.</p>
</details>

### 3.3 D3 · Fixture sai và bẫy (`03-fixture-invalid.js`)

<details><summary><b>Lời giải</b></summary>
<p>Ba document mẫu (khách, tài xế, chuyến) được chèn thử rồi xoá, in <code>PASS ba document mẫu hợp lệ</code>. Sau đó 37 ca sai, mỗi ca sửa đúng một chỗ của mẫu:</p>
<table>
<thead><tr><th>Nhóm</th><th>Số ca</th><th>Ví dụ</th></tr></thead>
<tbody>
<tr><td><code>customers</code></td><td>8</td><td>C3 phone là số → 121 (<code>bsonType</code>); C4 trùng phone → 11000 (<code>uq_customers_phone</code>); C7 trường lạ <code>email</code> → 121 (<code>additionalProperties</code>)</td></tr>
<tr><td><code>drivers</code></td><td>5</td><td>D3 <code>ONLINE</code> viết hoa → 121 (<code>enum</code>); D4 mã 8099.5 → 121 (<code>bsonType</code>)</td></tr>
<tr><td><code>trips</code>: kiểu và giá trị</td><td>12</td><td>T2 cước 45000.5, T3 chuỗi, T4 Decimal128 đều → 121 (<code>bsonType</code>); T12 <code>driver_id: null</code> thay vì vắng → 121</td></tr>
<tr><td><code>trips</code>: liên trường</td><td>7</td><td>R1a … R5, mỗi ca kiểm đúng một điều kiện #1 … #5</td></tr>
<tr><td>Đọc <code>errInfo.details</code></td><td>1</td><td>R4 lần nữa: <code>operatorName</code> gốc <code>$and</code>, điều kiện không thoả <code>#4</code></td></tr>
<tr><td>Update cũng bị kiểm</td><td>4</td><td>U1 <code>$inc</code> làm cước âm; U2 <code>$set</code> completed cho chuyến đang chạy → điều kiện #4; U3 trùng phone; U4 <code>$unset</code> trường bắt buộc</td></tr>
</tbody>
</table>
<p>Trích phần liên trường của <code>OUTPUT.txt</code>:</p>
<pre><code class="language-text">PASS R1a food không có quán → lỗi 121 (điều kiện #1)
PASS R1b ride có quán → lỗi 121 (điều kiện #1)
PASS R2a requested mà đã có tài xế → lỗi 121 (điều kiện #2)
PASS R2b accepted mà chưa có tài xế → lỗi 121 (điều kiện #2)
PASS R3 có tài xế, thiếu accepted_at → lỗi 121 (điều kiện #3)
PASS R4 completed thiếu completed_at → lỗi 121 (điều kiện #4)
PASS R5 hoàn thành trước khi nhận → lỗi 121 (điều kiện #5)</code></pre>
<p><code>expectError</code> trong <code>lib/check.js</code> chỉ PASS khi điều kiện được chỉ định là điều kiện <b>duy nhất</b> không thoả. Nhờ vậy ca sai nào vô tình vi phạm hai quy tắc sẽ FAIL, buộc người viết sửa ca.</p>
<p>Phần quan trọng không kém là bốn <b>bẫy</b>: MongoDB không báo lỗi, dữ liệu sai lọt vào. Lab ghi nhận rồi dọn đi:</p>
<pre><code class="language-text">PASS B1 chuyến của khách không tồn tại đã vào = 1
PASS B2 tài xế ở thành phố không tồn tại đã vào = 1
PASS B3 collection "trip" (thiếu s) được tự tạo = true
PASS B4 fare âm lọt qua bypass = 1</code></pre>
<p>B1, B2: không có khoá ngoại. B3: ghi vào tên collection gõ sai thì MongoDB tự tạo collection mới không có validator. B4: <code>bypassDocumentValidation</code> bỏ qua validator với người có quyền đó. Cuối tệp, số document vẫn là 3/6/6/3/15, cước 100014 vẫn 48.000, 100010 vẫn <code>in_transit</code>. Tổng của tệp: 1 + 37 + 4 + 1 = <b>43 PASS</b>.</p>
</details>

### 3.4 Vòng đời và migration (`04`, `05`, `06`)

<details><summary><b>Lời giải</b></summary>
<p><code>04-lifecycle.js</code> thêm tài xế 8007, cấp mã <b>100015</b> từ bộ đếm, đưa chuyến mới qua 7 trạng thái bằng update có điều kiện + <code>$push</code> (một thao tác nguyên tử mỗi bước), cho hai tài xế tranh nhận (người sau khớp 0), hoàn thành sau 31 phút với 7 sự kiện. Phần xoá khách cho thấy không có gì chặn: xoá khách 6 để lại 2 chuyến mồ côi, khôi phục từ <code>findOneAndDelete</code>. Sau <code>04</code> có 7 tài xế, 16 chuyến.</p>
<p><code>06-migration.js</code> là phần cộng điểm: thêm <code>vehicle_plate</code> + <code>schema_version</code> cho tài xế theo MG-009, có tài xế 8008 do "app v2" tạo. Sau <code>06</code> có <b>8 tài xế</b>, đều có biển số. <code>06-migration-rollback.js</code> không chạy trong <code>npm run p01</code>; bộ test chạy nó hai lần liên tiếp.</p>
</details>

### 3.5 D4 · Kiểm tra bất biến, đối soát, kiểm kê (`07-checks.js`)

<details><summary><b>Lời giải</b></summary>
<p>Mọi kiểm tra đi qua một hàm: chạy pipeline, mong đợi 0 document, in bảng vi phạm nếu có.</p>
<pre><code class="language-js">function check(label, coll, pipeline) {
  const bad = db.getCollection(coll).aggregate(pipeline).toArray();
  expectValue(label, bad.length, 0);
  if (bad.length) table(bad);
}</code></pre>
<table>
<thead><tr><th>Kiểm tra</th><th>Quy tắc</th><th>Vì sao không để validator làm</th></tr></thead>
<tbody>
<tr><td>K1–K3 chuyến trỏ tới khách/tài xế/quán không tồn tại</td><td>Q13</td><td>Không có khoá ngoại; validator không đọc collection khác</td></tr>
<tr><td>K4 <code>city_id</code> của chuyến, tài xế, quán không có trong <code>cities</code> (3 lần)</td><td>Q13</td><td>Pattern <code>^[A-Z]{2}$</code> nhận cả <code>'XX'</code> (bẫy B2)</td></tr>
<tr><td>K5 quán khác thành phố, K6 tài xế khác thành phố</td><td>Q13</td><td>Cần <code>$lookup</code></td></tr>
<tr><td>K7 <code>busy</code> ⇔ đúng một chuyến đang chạy</td><td>Q14</td><td>Trạng thái tài xế nằm ở document khác (MG-008, bản sao suy ra)</td></tr>
<tr><td>K8 sự kiện cuối khác trạng thái, K9 sự kiện không theo thứ tự thời gian</td><td>Q12</td><td>Diễn đạt được bằng <code>$expr</code> nhưng nặng và khó đọc; kiểm định kỳ là đủ vì đường ghi duy nhất là <code>step()</code></td></tr>
<tr><td>K10 bộ đếm ≥ mã chuyến lớn nhất</td><td>Q14</td><td>Bộ đếm là collection khác</td></tr>
<tr><td>K11 tổng document không thoả validator hiện tại</td><td>Q1–Q12</td><td>Bắt dữ liệu lọt qua <code>bypassDocumentValidation</code>, restore, đổi validator</td></tr>
<tr><td>K12 đủ index unique ngoài <code>_id</code></td><td>Q2, Q3, Q5</td><td>Index bị xoá thì luật duy nhất biến mất mà không ai biết</td></tr>
</tbody>
</table>
<p>Kết quả trên lab:</p>
<pre><code class="language-text">PASS K1 chuyến trỏ tới khách không tồn tại = 0
PASS K2 chuyến trỏ tới tài xế không tồn tại = 0
PASS K3 chuyến trỏ tới quán không tồn tại = 0
PASS K4 trips.city_id không có trong cities = 0
PASS K4 drivers.city_id không có trong cities = 0
PASS K4 merchants.city_id không có trong cities = 0
PASS K5 quán khác thành phố với chuyến đồ ăn = 0
PASS K6 tài xế khác thành phố với chuyến = 0
PASS K7 tài xế: busy ⇔ đúng một chuyến đang chạy = 0
PASS K8 sự kiện cuối khác trạng thái hiện tại = 0
PASS K9 sự kiện không theo thứ tự thời gian = 0
PASS K10 bộ đếm ≥ mã chuyến lớn nhất = true

collection|valid|nrecords|nNonCompliantDocuments
----------|-----|--------|----------------------
cities|true|3|0
customers|true|6|0
drivers|true|8|0
merchants|true|3|0
trips|true|16|0
(5 document)
PASS K11 tổng document không thoả validator = 0
PASS K12 index unique ngoài _id = customers.uq_customers_phone, drivers.uq_drivers_code, trips.uq_trips_code</code></pre>
<p>12 mã kiểm tra nhưng <b>14 dòng PASS</b>, vì K4 chạy cho ba collection. Cộng dòng đối soát doanh thu bên dưới là <b>15 PASS</b> của <code>07</code>.</p>
</details>

#### Đối soát doanh thu

Kiểm tra bất biến trả lời "dữ liệu có hợp lệ không"; đối soát trả lời "con số có khớp không". `07-checks.js` cộng cước chuyến `completed` theo thành phố, nối tên thành phố bằng `$lookup`, và in luôn **kiểu BSON** của tổng:

```js
db.trips.aggregate([
  { $match: { status: 'completed' } },
  { $group: { _id: '$city_id', so_chuyen: { $sum: 1 }, doanh_thu: { $sum: '$fare' } } },
  { $lookup: { from: 'cities', localField: '_id', foreignField: '_id', as: 'c' } },
  { $project: { _id: 0, city: '$_id', ten: { $first: '$c.name' }, so_chuyen: 1, doanh_thu: 1, kieu_tong: { $type: '$doanh_thu' } } },
  { $sort: { city: 1 } },
]);
```

```text
city|ten|so_chuyen|doanh_thu|kieu_tong
----|---|---------|---------|---------
DN|Đà Nẵng|1|55000|int
HN|Hà Nội|2|164000|int
SG|TP. Hồ Chí Minh|5|178000|int
(3 document)
PASS tổng doanh thu chuyến hoàn thành (đ) = 397000
```

Dán cùng pipeline vào query editor của DBStudio, chạy trên database `toinoi`, khung kết quả (chế độ **Tài liệu**) hiện đúng ba document DN/HN/SG, `kieu_tong` đều là `int`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-thuc-hanh-mo-hinh-chuyen-xe/dbstudio-01.webp" alt="DBStudio chạy pipeline doanh thu theo thành phố: DN 55000, HN 164000, SG 178000, kiểu int" width="1086" height="800" loading="lazy"></div><figcaption>DBStudio · đối soát doanh thu chuyến hoàn thành theo thành phố</figcaption></figure>

Đối chiếu tay: Đà Nẵng 55.000 đ (chuyến 100004); Hà Nội 38.000 + 126.000 = 164.000 đ (100001, 100005); TP. Hồ Chí Minh 45.000 + 62.000 + 30.000 + 0 + 41.000 = 178.000 đ (100000, 100002, 100007, 100008, 100015). Tổng **397.000 đ** trên 8 chuyến hoàn thành. Hai chuyến huỷ (100003 cước 71.000 đ, 100006 cước 52.000 đ) và sáu chuyến đang chạy không được cộng. Cột `kieu_tong` là `int`: tổng còn nhỏ nên `$sum` giữ `int`; khi vượt giới hạn int32, `$sum` tự chuyển lên `long` thay vì tràn (MG-004). Nếu một ngày cột này hiện `double`, nghĩa là có cước kiểu double lọt vào (qua bypass) và K11 phải báo trước đó.

#### Kiểm kê cuối phần

```text
cities|customers|drivers|merchants|trips|counters
------|---------|-------|---------|-----|--------
3|6|8|3|16|1
(1 document)
collection|count|avgObjSize|nindexes
----------|-----|----------|--------
trips|16|283|4
```

3 thành phố, 6 khách, 8 tài xế (6 fixture + 8007 ở `04` + 8008 ở `06`), 3 quán, 16 chuyến, 1 bộ đếm. `avgObjSize` 283 byte, xa trần 16 MB (MG-008).

### 3.6 D5 · Bằng chứng bàn giao

<details><summary><b>Lời giải</b></summary>
<p>Chạy từ database trống (<code>00-reset.js</code> xoá database <code>toinoi</code> và in phiên bản server), tổng kết cuối của <code>npm run p01</code>:</p>
<pre><code class="language-text">// [lab] File đã chạy: 8
// [lab] PASS: 115
// [lab] FAIL: 0
// [lab] KẾT QUẢ: ĐẠT</code></pre>
<p>Dòng đầu của <code>OUTPUT.txt</code> ghi môi trường: <code>8.0.38|rs0|PRIMARY|toinoi|8.0</code> (phiên bản, replica set, vai trò, database, FCV). Ảnh Docker được ghim theo digest để ai chạy cũng ra cùng bản.</p>
<p><code>npm test</code> (<code>test/p01.test.mjs</code>) có 4 test, theo README lab đều pass trên môi trường đã kiểm: (1) thí nghiệm kiểu BSON của MG-004 đạt 13 PASS; (2) <code>p01</code> chạy đủ 8 file, 115 PASS, 0 FAIL và kết quả <b>khớp từng dòng</b> với <code>OUTPUT.txt</code> (chỉ bỏ dòng "Đã xoá database toinoi cũ"); (3) script hoàn tác của <code>06</code> chạy hai lần liên tiếp, lần nào cũng có R1 = 0 và R2 = 8; (4) chạy lại <code>p01</code> lần cuối để database trở về trạng thái đầy đủ.</p>
<p>Phần "chưa kiểm" phải ghi thẳng, xem mục 7.</p>
</details>

## 4. Validator bảo đảm được gì, không bảo đảm gì

Đây là bảng nên đặt ở trang đầu hồ sơ bàn giao, vì trưởng nhóm vận hành cần nó hơn mọi chi tiết khác:

| Loại luật | Validator + index unique giữ được? | Trong lab |
|---|---|---|
| Kiểu BSON, trường bắt buộc, `enum`, khoảng giá trị, pattern | Có, ở mọi insert và update | T1–T12, C1–C7, D2–D5 |
| Cấm trường lạ, cấm `null` thay cho trường vắng | Có (`additionalProperties: false`, không khai `null` trong kiểu) | C7, T12 |
| Luật liên trường trong **một** document | Có (`$or`, `$expr` trong `$and`) | R1a–R5, U2 |
| Giới hạn kích thước mảng nhúng | Có (`maxItems`) | `events` ≤ 20, `menu` ≤ 200 |
| Giá trị duy nhất giữa các document | Có, bằng index unique (lỗi 11000), không phải validator | C4, D1, T8, U3 |
| Tham chiếu tới document có thật | **Không** | Bẫy B1, B2; xoá khách ở `04`; canh bằng K1–K4 |
| Luật giữa nhiều document, nhiều collection | **Không** | K5–K7, K10 |
| Duy nhất trong mảng của một document (theo một trường con) | **Không** (`uniqueItems` so cả phần tử) | Bài tập MG-008 |
| Chuyển trạng thái đúng thứ tự (`searching` → `accepted`, không ngược) | **Không**; giữ bằng bộ lọc `status: from` của update | `04`, "huỷ chuyến đã hoàn thành: matched = 0" |
| Ghi vào đúng collection | **Không**; tên sai tạo collection mới | Bẫy B3 |
| Người có quyền bypass | **Không** | Bẫy B4; canh bằng K11 |
| Dữ liệu có từ trước khi đổi validator | **Không**, `collMod` không quét | MG-009; canh bằng `$nor` và `validate()` |

Kết luận cho câu hỏi của trưởng nhóm vận hành: chuyến `completed` không có tài xế **bị chặn ở mọi lần ghi** (điều kiện #2 và #4), trừ người có quyền bypass, và K11 bắt được cả trường hợp đó. Còn chuyến trỏ tới khách đã bị xoá thì **không bị chặn**; K1 phát hiện nó, và thao tác xoá khách phải đi qua một đường kiểm tra ở ứng dụng.

## 5. Tiêu chí tự chấm

Mỗi tiêu chí 10 điểm. Từ 80 điểm là đạt để sang Phần 2; tiêu chí 1, 4 và 8 là bắt buộc.

| # | Tiêu chí | Đạt khi |
|---|---|---|
| 1 | Chạy lại được | Từ database trống, chạy toàn bộ hai lần liên tiếp cho cùng kết quả, so khớp được với kết quả đã lưu |
| 2 | Validator đủ lớp | 5 collection có validator; `additionalProperties: false`; quy tắc liên trường tách thành từng điều kiện đánh số |
| 3 | Kiểu có lý do | Tiền `int`/`long` (không double, không chuỗi); thời điểm `date`; định danh UUID/`int` được giải thích; trường không áp dụng thì vắng |
| 4 | Phủ quy tắc | Mỗi Q1–Q14 được validator, index unique hoặc kiểm tra giữ; quy tắc nào chỉ được canh thì ghi rõ trong bảng mục 4 |
| 5 | Nhúng/tham chiếu có lý do | Mỗi mảng nhúng có giới hạn (`maxItems`); mỗi tham chiếu có kiểm tra `$lookup` |
| 6 | Fixture hợp lệ đủ ca | 8 trạng thái, 3 loại, ca biên cước 0, huỷ trước/sau nhận, qua nửa đêm giờ VN |
| 7 | Fixture sai chính xác | Mỗi ca vi phạm đúng một quy tắc; kỳ vọng ghi mã lỗi và quy tắc/điều kiện; có ca update |
| 8 | Không ca sai nào lọt, bẫy được ghi lại | Số document không đổi sau fixture sai; các bẫy không báo lỗi được ghi nhận và dọn đi |
| 9 | Kiểm tra bất biến | Phủ Q12–Q14, kiểm validator toàn bộ và index unique; tất cả trả 0; có đối soát tiền khớp tay |
| 10 | Bằng chứng trung thực | Kết quả chạy thật, phiên bản server, danh sách điều chưa kiểm |

## 6. Bẫy hay gặp khi tự làm

- **Ca sai vi phạm hai điều kiện.** Như gợi ý ở mục 2: `status: 'completed'` không kèm tài xế vi phạm ba điều kiện cùng lúc. Kiểm `clausesNotSatisfied` có đúng một phần tử.
- **Kiểm "lỗi 121" mà không kiểm vì sao.** Một validator viết sai có thể từ chối mọi thứ; mọi ca sai đều "PASS". Document mẫu hợp lệ và kiểm chi tiết lỗi là hai lớp chống chuyện đó.
- **Dùng `null` cho trường chưa có.** Validator của lab cấm, và truy vấn `{ driver_id: null }` khớp cả trường vắng (MG-007): hai nghĩa lẫn vào nhau.
- **Fixture phụ thuộc giờ chạy.** `new Date()` trong fixture làm `OUTPUT.txt` mỗi lần một khác; lab chỉ dùng `new Date()` ở bản sao lưu của rollback, nơi không in ra.
- **Tin rằng validator chặn tham chiếu sai.** `bsonType: 'binData'` chỉ nói kiểu. Thiếu K1–K4 là thiếu thứ thay cho khoá ngoại.
- **Quên rằng tên collection sai không báo lỗi.** Một chữ `trip` thay cho `trips` trong code ghi là dữ liệu đi vào một collection không validator. Kiểm danh sách collection trong kiểm kê.
- **Tin `valid: true` của `validate()`.** Đọc `nNonCompliantDocuments` (MG-009).

## 7. Bàn giao

| Hạng mục | Nằm ở | Bằng chứng | Chưa kiểm / giới hạn |
|---|---|---|---|
| Collection, validator, index | `p01/01-schema.js` | 10 index, 5 collection có validator, `trips` strict/error | Validator đọc lại từ catalog ở `06`, không chép tay |
| Fixture hợp lệ | `p01/02-fixture-valid.js` | 7 PASS, bảng 8 trạng thái × 3 loại | 15 chuyến cũ có `events` rỗng (chuyển từ hệ cũ) |
| Fixture sai và bẫy | `p01/03-fixture-invalid.js` | 43 PASS: 37 ca sai, 4 bẫy | Bẫy chỉ được ghi nhận, không bị chặn |
| Vòng đời | `p01/04-lifecycle.js` | Mã 100015, 7 sự kiện, 31 phút | Một kết nối: "hai tài xế tranh nhận" là tuần tự, chưa có tranh chấp thật (Phần 4) |
| Migration + hoàn tác | `p01/06-migration.js`, `06-migration-rollback.js` | 12 PASS; rollback R1–R4 PASS hai lần | Backfill một lô; chưa đo trên dữ liệu lớn |
| Kiểm tra bất biến, đối soát | `p01/07-checks.js` | 15 PASS, 397.000 đ, kiểm kê 3/6/8/3/16 | Chạy theo yêu cầu, chưa được lên lịch |
| Kết quả chuẩn | `p01/OUTPUT.txt` | `npm test` so khớp từng dòng | |
| Môi trường | `docker-compose.yml`, README lab | MongoDB 8.0.38, FCV 8.0, replica set một node | Chưa bật xác thực (Phần 5); một node, chưa có failover (Phần 6) |

Những gì **chưa** có cũng phải nằm trong hồ sơ: chưa có truy vấn báo cáo đầy đủ (Phần 2), chưa đo index bằng `explain` (Phần 3), chưa có hai phiên ghi đồng thời thật và transaction nhiều document (Phần 4), chưa có phân quyền để hạn chế ai được dùng `bypassDocumentValidation` (Phần 5). Lab chạy với dữ liệu nhỏ; mọi kết luận về hiệu năng đều chưa được rút ra.

## 8. Tổng kết Phần 1

| Bài | Bạn đã có gì cho sản phẩm cuối phần |
|---|---|
| [MG-001 · MongoDB là gì và nằm ở đâu trong Tới Nơi?](/articles/mongodb-la-gi-va-vai-tro-trong-toi-noi) | Instance, database, collection, document; replica set; MongoDB giữ bản gốc của dữ liệu nào |
| [MG-002 · Dựng MongoDB 8.0 bằng Docker và kết nối bằng DBStudio](/articles/mongodb-cai-dat-docker-dbstudio) | Lab chạy được, ghi phiên bản và cách dọn — nền của D5 |
| [MG-003 · Document, BSON và `_id`](/articles/mongodb-document-bson-id) | Đọc chuyến 100004 dạng document, trường vắng khác `null` |
| [MG-004 · Kiểu BSON cho tiền, thời gian, chuỗi tiếng Việt và định danh](/articles/mongodb-kieu-bson-tien-thoi-gian-unicode) | Data dictionary: `int`/`long` cho tiền, `Date`, UUID — tiêu chí 3 |
| [MG-005 · Schema validation với `$jsonSchema` và index unique](/articles/mongodb-schema-validation-unique-index) | Validator có điều kiện đánh số, lỗi 121 và 11000 — D1, D3 |
| [MG-006 · insert, update, delete và toán tử cập nhật](/articles/mongodb-insert-update-delete) | Vòng đời chuyến bằng update có điều kiện, bộ đếm mã |
| [MG-007 · find: lọc, projection, sắp xếp, phân trang và collation](/articles/mongodb-find-loc-sap-xep-collation) | Lọc theo ngày giờ VN, thứ tự xác định |
| [MG-008 · Nhúng hay tham chiếu](/articles/mongodb-nhung-hay-tham-chieu) | Lý do nhúng `events` và `menu`, tham chiếu phần còn lại; K1–K7 |
| [MG-009 · Đổi schema khi không có ALTER TABLE](/articles/mongodb-schema-migration-tuong-thich) | Migration expand → contract và hoàn tác — phần cộng điểm |

Sản phẩm của phần đúng như khung đã đặt: collection cho thành phố, khách, tài xế, quán (menu nhúng) và chuyến (lịch sử nhúng); validator `$jsonSchema` cộng quy tắc liên trường; index unique; fixture hợp lệ/sai; kiểm tra bất biến liên collection. Người đã đọc [PG-010](/articles/postgresql-thuc-hanh-schema-chuyen-xe) sẽ thấy cùng đề bài, cùng dữ liệu, cùng con số 397.000 đ, nhưng ranh giới giữa "database chặn" và "truy vấn canh" nằm ở chỗ khác hẳn.

## 9. Bài tập mở rộng

Mỗi bài theo cùng một khuôn: thêm **một quy tắc**, một **ca sai** chứng minh nó chặn hoặc phát hiện được, và một **kiểm tra** trên dữ liệu hiện có.

**Bài 1.** Validator hiện chỉ yêu cầu `updated_at ≥ requested_at`. Thêm quy tắc: `updated_at` cũng không sớm hơn `accepted_at` và `completed_at` (khi có). Áp lên `trips` đã có dữ liệu theo cách của MG-009.

<details><summary><b>Lời giải</b></summary><p>Quy tắc mới là điều kiện #6 trong <code>$and</code>:</p>
<pre><code class="language-js">const r6 = { $expr: { $and: [
  { $or: [{ $eq: [{ $type: '$accepted_at' }, 'missing'] }, { $gte: ['$updated_at', '$accepted_at'] }] },
  { $or: [{ $eq: [{ $type: '$completed_at' }, 'missing'] }, { $gte: ['$updated_at', '$completed_at'] }] },
] } };
db.trips.countDocuments({ $nor: [r6] });   // kiểm dữ liệu cũ TRƯỚC khi collMod</code></pre>
<p>Trên lab, truy vấn <code>$nor</code> trả <b>0</b>, nên có thể <code>collMod</code> với <code>$and: [...cũ, r6]</code>. Đã thử trong database nháp <code>scratch_d</code> (lab 8.0.38, xoá sau khi chạy): nạp đủ 16 chuyến của lab qua validator mới không lỗi; ca sai "<code>completed</code> có tài xế, <code>accepted_at</code> 08:41, <code>completed_at</code> 09:00, <code>updated_at</code> 08:41" bị từ chối với lỗi 121, điều kiện <b>#6</b> là điều kiện duy nhất không thoả. Thêm ca này vào <code>03</code> với <code>expectError(..., 121, 6)</code>.</p></details>

**Bài 2.** Thêm quy tắc: tài xế không thể nhận chuyến trước thời điểm tài xế được tạo (`trips.accepted_at ≥ drivers.created_at`). Quy tắc này đặt ở validator được không? Viết kiểm tra K13.

<details><summary><b>Lời giải</b></summary><p>Không: <code>drivers.created_at</code> nằm ở collection khác, validator của <code>trips</code> không đọc được. Kiểm bằng <code>$lookup</code>:</p>
<pre><code class="language-js">check('K13 chuyến được nhận trước khi tài xế tồn tại', 'trips', [
  { $match: { driver_id: { $exists: true } } },
  { $lookup: { from: 'drivers', localField: 'driver_id', foreignField: '_id', as: 'd',
               pipeline: [{ $project: { created_at: 1 } }] } },
  { $match: { $expr: { $lt: ['$accepted_at', { $first: '$d.created_at' }] } } },
  { $project: { _id: 0, code: 1 } },
]);</code></pre>
<p>Trên lab trả 0 document. Ca sai không chèn được qua validator nên phải viết dạng "bẫy": chèn tài xế mới có <code>created_at</code> 09:00 ngày 03/11 và một chuyến của tài xế đó có <code>accepted_at</code> 08:00, chạy K13 và mong đợi 1 vi phạm, rồi dọn cả hai. Lưu ý: chuyến có tài xế trỏ tới tài xế không tồn tại thì <code>$first</code> trả về giá trị vắng và so sánh không báo; K2 đã lo trường hợp đó.</p></details>

**Bài 3.** Thêm quy tắc: nếu chuyến có lịch sử thì sự kiện đầu tiên phải là `requested`. Đặt ở validator hay kiểm tra? Viết cả hai phần.

<details><summary><b>Lời giải</b></summary><p>Validator diễn đạt được vì chỉ cần nhìn một document: điều kiện #7 <code>{ $or: [{ events: { $size: 0 } }, { 'events.0.status': 'requested' }] }</code>. <code>$nor</code> trên dữ liệu lab trả 0 (15 chuyến có <code>events</code> rỗng, 100015 bắt đầu bằng <code>requested</code>). Trong <code>scratch_d</code>, ca sai "<code>status: 'requested'</code>, <code>events: [{ status: 'searching', at }]</code>" bị từ chối với lỗi 121, điều kiện <b>#7</b>. Ca này không vi phạm #0 vì <code>searching</code> là trạng thái hợp lệ trong <code>enum</code>. Kiểm tra định kỳ vẫn nên có (dùng chính biểu thức với <code>$nor</code>), vì dữ liệu có thể vào bằng bypass; K11 đã bao phần đó nếu quy tắc nằm trong validator.</p></details>

**Bài 4.** Viết lại bảng mục 4 cho hệ thống của bạn: liệt kê năm luật quan trọng nhất của dữ liệu bạn đang quản lý, đánh dấu luật nào database giữ, luật nào chỉ được canh, và ai được quyền đi vòng qua (bypass, sửa tay, nhập hàng loạt).

<details><summary><b>Lời giải</b></summary><p>Không có đáp án duy nhất. Bảng tốt có ba đặc điểm: (1) mỗi luật "chỉ được canh" có tên truy vấn kiểm tra, tần suất chạy và người nhận cảnh báo; (2) mỗi đường đi vòng có tên quyền hoặc quy trình cụ thể (role nào có <code>bypassDocumentValidation</code>, script nhập dữ liệu nào chạy với quyền đó); (3) ghi rõ cái gì chưa được kiểm, thay vì để trống. Nếu một luật quan trọng nằm ở cột "không" mà không có kiểm tra nào, đó là việc đầu tiên cần làm.</p></details>

## 10. Đọc tiếp

- [Schema Validation](https://www.mongodb.com/docs/manual/core/schema-validation/): `$jsonSchema`, quy tắc bằng toán tử truy vấn, `errInfo`.
- [`$jsonSchema`](https://www.mongodb.com/docs/manual/reference/operator/query/jsonSchema/): từ khoá được hỗ trợ và chỗ khác với JSON Schema chuẩn.
- [`$lookup`](https://www.mongodb.com/docs/manual/reference/operator/aggregation/lookup/) và [`validate`](https://www.mongodb.com/docs/manual/reference/command/validate/).
- [Unique Indexes](https://www.mongodb.com/docs/manual/core/index-unique/).
- [PG-010 · Thực hành: schema chuyến xe có kiểm tra ràng buộc](/articles/postgresql-thuc-hanh-schema-chuyen-xe): cùng đề bài trên PostgreSQL.
- Lab: `examples/toi-noi-mongodb/p01`: `README.md` (bảng khác biệt có chủ đích so với PostgreSQL/SQL Server), các tệp `00` … `07`, `OUTPUT.txt`, `test/p01.test.mjs`.

Bài tiếp theo: Phần 2 · Truy vấn và aggregation cho nghiệp vụ, mở đầu bằng MG-011 về toán tử truy vấn (sắp ra mắt).
