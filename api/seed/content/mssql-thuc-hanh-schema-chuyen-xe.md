---
slug: mssql-thuc-hanh-schema-chuyen-xe
title: "Thực hành: schema chuyến xe có kiểm tra ràng buộc"
summary: "Tới Nơi sắp đổ dữ liệu chuyến xe vào SQL Server 2022 và không ai chấp nhận câu “schema chạy rồi”. Bài thực hành cuối Phần 1: bàn giao DDL, fixture hợp lệ và sai theo số hiệu lỗi, truy vấn bất biến liên bảng, DBCC CHECKCONSTRAINTS và bằng chứng chạy lại được."
category: mssql
level: advanced
tags: [mssql, sql-server-2022, ms-010, thuc-hanh, schema, constraint, fixture, toi-noi]
language: vi
---

Đội nền tảng Tới Nơi chuẩn bị chuyển dữ liệu chuyến xe từ hệ cũ sang SQL Server 2022. Trong buổi duyệt thiết kế, trưởng nhóm vận hành hỏi đúng một câu: "Làm sao tôi biết một chuyến `completed` mà không có tài xế sẽ không lọt vào?". Câu trả lời "schema chạy rồi, ứng dụng có kiểm tra" không được chấp nhận. Thứ cần bàn giao là **bằng chứng**: chạy từ database trống, dữ liệu đúng đi qua, từng loại dữ liệu sai bị chặn với đúng số hiệu lỗi và đúng tên quy tắc, còn những quy tắc schema chưa chặn được thì có truy vấn canh.

Đây là bài thực hành cuối Phần 1. Bạn nhận đề bài, tự làm, rồi mới mở lời giải mẫu dựa trên lab và tự chấm theo tiêu chí ở mục 4.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-010 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: toàn bộ MS-001 … MS-009 (danh sách ở mục 6), đặc biệt <a href="/articles/mssql-create-table-constraints">MS-005 · CREATE TABLE, ràng buộc và IDENTITY</a> và <a href="/articles/mssql-mo-hinh-quan-he-chuan-hoa">MS-008 · Chuẩn hoá và ranh giới mô hình</a>. Lab: <code>examples/toi-noi-mssql/p01</code>. Sau bài này, bạn bàn giao được DDL, fixture hợp lệ/sai, truy vấn bất biến và bằng chứng chạy cho schema chuyến xe trên SQL Server.</p>
</div>

## 1. Đề bài

### Bối cảnh

Schema phục vụ ba dịch vụ của [Tới Nơi](/articles/du-an-mau-toi-noi), gồm đặt xe (`ride`), giao đồ ăn (`food`) và giao bưu kiện (`parcel`), với sáu bảng trong schema `toinoi`: `cities`, `customers`, `drivers`, `merchants`, `trips`, `trip_events`. Tên bảng và cột bám lược đồ lõi của hồ sơ dự án; bạn được thêm ràng buộc nhưng không đổi nghĩa cột. Tiền tính bằng đồng (số nguyên), thời điểm là `datetimeoffset` ghi rõ độ lệch `+07:00`, database dùng collation `Vietnamese_100_CI_AS`. Mọi tên người, số điện thoại, biển hiệu là hư cấu.

### Quy tắc nghiệp vụ phải mã hoá

| Mã | Quy tắc |
|---|---|
| R1 | Thành phố có mã 2 chữ **in hoa**, duy nhất; tên không rỗng |
| R2 | Khách đăng nhập bằng số điện thoại dạng `0` + 9 chữ số, duy nhất; họ tên không rỗng (kể cả toàn khoảng trắng) |
| R3 | Tài xế có mã nội bộ dương, duy nhất, thuộc một thành phố có thật; loại xe `bike`/`car`; trạng thái `offline`/`online`/`busy`, mặc định `offline` |
| R4 | Quán thuộc một thành phố có thật; tên không rỗng |
| R5 | Chuyến có loại thuộc 3 dịch vụ, trạng thái thuộc 8 trạng thái vòng đời, mặc định `requested`; trỏ tới khách, thành phố (và tài xế, quán nếu có) có thật |
| R6 | Cước là số nguyên đồng, không âm (0 đ hợp lệ, dùng cho khuyến mãi) |
| R7 | Đơn `food` phải có quán; loại khác không gắn quán |
| R8 | Có tài xế khi và chỉ khi có mốc nhận chuyến `accepted_at` |
| R9 | `requested`/`searching` chưa có tài xế; từ `accepted` tới `completed` phải có tài xế; `cancelled` tuỳ trường hợp (huỷ trước hay sau khi nhận) |
| R10 | Có `completed_at` khi và chỉ khi trạng thái `completed` |
| R11 | `accepted_at` không sớm hơn `requested_at`; `completed_at` không sớm hơn `accepted_at` (so theo khoảnh khắc, kể cả khi hai mốc ghi khác độ lệch) |
| R12 | Mã chuyến dễ đọc do database cấp từ 100000, không ghi tay; bảng `trips` lưu theo thứ tự mã này |
| R13 | Lịch sử trạng thái chỉ thêm, trạng thái thuộc 8 giá trị, trỏ tới chuyến có thật |
| R14 | Quy tắc nhiều bảng: tài xế và quán cùng thành phố với chuyến; một tài xế tối đa một chuyến đang chạy; `busy` khớp với chuyến đang chạy; sự kiện cuối khớp trạng thái chuyến; `updated_at` không sớm hơn mốc khác của dòng |

### Sản phẩm bàn giao

| Mã | Sản phẩm | Yêu cầu tối thiểu |
|---|---|---|
| D1 | DDL | Sáu bảng; mọi PK, FK, UNIQUE, CHECK **và DEFAULT** được đặt tên theo quy ước `pk_/fk_/uq_/ck_/df_`; chọn kiểu cho tiền, thời gian, chuỗi tiếng Việt, định danh và nêu lý do; chọn clustered key cho `trips`; chú thích `MS_Description` cho cột khó hiểu |
| D2 | Fixture hợp lệ | Chạy lại được từ database trống; giá trị cố định (GUID, thời điểm có độ lệch); phủ đủ 8 trạng thái và 3 loại; có ca biên |
| D3 | Fixture sai | Mỗi câu vi phạm **đúng một** quy tắc, ghi trước số hiệu lỗi và tên ràng buộc (hoặc tên cột) mong đợi; mỗi R1–R13 ít nhất một ca; có ca sai kiểu dữ liệu |
| D4 | Truy vấn bất biến | Mỗi truy vấn đếm dòng vi phạm, kết quả đúng là 0; phủ R14 và kiểm chính schema (ràng buộc tắt hoặc không tin cậy, tên sai quy ước, FK thiếu index) |
| D5 | Bằng chứng | Kết quả chạy từ database trống, tổng kết 0 FAIL; ghi phiên bản SQL Server (`ProductVersion`, CU) và những gì **chưa** kiểm |

Tuỳ chọn cộng điểm: một migration tương thích ngược kèm rollback chạy lặp được theo [MS-009](/articles/mssql-schema-migration-tuong-thich).

Chỉ dùng kiến thức Phần 1. Truy vấn của D4 cần `JOIN`, `EXISTS`, `GROUP BY`, `CROSS APPLY` (Phần 2 dạy kỹ). Nếu chưa tự tin, viết bằng lời từng truy vấn cần tìm gì, rồi đối chiếu với lời giải.

## 2. Gợi ý trước khi làm

Không cần xem lời giải, nhưng nên nhớ những điều Phần 1 đã chỉ ra:

- Viết quy tắc thành câu "nếu A thì B" trước, rồi mới thành `NOT A OR B`. T-SQL không có kiểu boolean, nên "A khi và chỉ khi B" viết thành `(A AND B) OR (NOT A AND NOT B)`.
- CHECK **chấp nhận** khi biểu thức ra UNKNOWN. Quy tắc "phải có" cần `IS NULL`/`IS NOT NULL` tường minh.
- Mỗi CHECK giữ **một** quy tắc, để thông báo 547 nêu đúng tên quy tắc bị vi phạm.
- Database không phân biệt hoa thường: `'[A-Z][A-Z]'` khớp cả `'sg'`. Muốn chỉ nhận chữ in hoa, ép `COLLATE Latin1_General_BIN2` ngay trong CHECK.
- Chuỗi tiếng Việt luôn có tiền tố `N'…'`; thiếu nó, chữ bị đổi sang code page trước khi lưu ([MS-004](/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode)).
- `NOT NULL` không có tên ràng buộc: lỗi 515 chỉ nêu tên cột. `DEFAULT` thì có, và phải đặt tên, nếu không SQL Server sinh tên kiểu `DF__trips__statu__…`.
- Viết ngày giờ dạng ISO `'YYYY-MM-DD hh:mm:ss +07:00'`. `'03/11/2026'` không lỗi, nhưng ra ngày khác nhau tuỳ `DATEFORMAT` của phiên.
- Viết một câu `INSERT` hợp lệ làm dòng gốc; mỗi ca sai chỉ đổi đúng một giá trị so với nó, và chạy trong transaction rồi `ROLLBACK` để dữ liệu không đổi.

Chạy lab để có môi trường đối chiếu ([MS-002](/articles/mssql-cai-dat-docker-dbstudio) hướng dẫn chi tiết):

```sh
cd examples/toi-noi-mssql
npm run db:up        # SQL Server 2022 trong Docker, chờ healthcheck
npm run p01          # 00 → 07 trên database ToiNoi mới
npm test             # so với OUTPUT.txt, chạy rollback hai lần, thí nghiệm MS-004
```

Muốn làm bằng giao diện: trong DBStudio, mở tệp của bạn trong query editor và bấm Ctrl+Shift+Enter ("Chạy cả script"). Mỗi `SELECT` hiện thành một result set; các dòng `PASS`/`FAIL` mà thủ tục kiểm tra in bằng `PRINT` nằm ở tab "Thông điệp".

## 3. Lời giải mẫu

Lời giải là chính các tệp của lab. Hãy mở từng phần **sau khi** đã có bản của bạn.

### 3.1 D1 — DDL (`01-schema.sql`)

<details><summary><b>Lời giải</b></summary>
<p>Bảng đối chiếu quy tắc → ràng buộc. Sau toàn bộ lab (kể cả migration <code>06</code>), kiểm kê của <code>07</code> đếm 6 PK, 7 FK, 4 UNIQUE, 21 CHECK và 12 DEFAULT, tất cả đều có tên.</p>
<table>
<thead><tr><th>Quy tắc</th><th>Ràng buộc</th></tr></thead>
<tbody>
<tr><td>R1</td><td><code>pk_cities</code>, <code>uq_cities_code</code>, <code>ck_cities_code_format</code> (<code>code COLLATE Latin1_General_BIN2 LIKE '[A-Z][A-Z]'</code>), <code>ck_cities_name_not_blank</code> (<code>TRIM(name) &lt;&gt; N''</code>)</td></tr>
<tr><td>R2</td><td><code>uq_customers_phone</code>, <code>ck_customers_phone_format</code> (<code>LIKE '0[0-9]…'</code> 9 lớp <code>[0-9]</code>; <code>varchar(10)</code> chặn chuỗi dài hơn), <code>ck_customers_full_name_not_blank</code></td></tr>
<tr><td>R3</td><td><code>uq_drivers_code</code>, <code>ck_drivers_code_positive</code>, <code>fk_drivers_city</code>, <code>ck_drivers_vehicle_type</code>, <code>ck_drivers_status</code> + <code>df_drivers_status</code></td></tr>
<tr><td>R4</td><td><code>fk_merchants_city</code>, <code>ck_merchants_name_not_blank</code></td></tr>
<tr><td>R5</td><td><code>ck_trips_kind</code>, <code>ck_trips_status</code> + <code>df_trips_status</code>, 4 FK <code>fk_trips_city/customer/driver/merchant</code></td></tr>
<tr><td>R6</td><td><code>fare int</code> + <code>ck_trips_fare_non_negative</code></td></tr>
<tr><td>R7</td><td><code>ck_trips_merchant_only_food</code>: <code>(kind = 'food' AND merchant_id IS NOT NULL) OR (kind &lt;&gt; 'food' AND merchant_id IS NULL)</code></td></tr>
<tr><td>R8–R11</td><td><code>ck_trips_driver_accepted_together</code>, <code>ck_trips_unassigned_no_driver</code>, <code>ck_trips_assigned_has_driver</code>, <code>ck_trips_completed_at_by_status</code>, <code>ck_trips_accepted_after_requested</code>, <code>ck_trips_completed_after_accepted</code></td></tr>
<tr><td>R12</td><td><code>code bigint IDENTITY(100000, 1)</code> + <code>uq_trips_code UNIQUE CLUSTERED</code>; khoá chính <code>pk_trips</code> trên <code>id</code> là NONCLUSTERED</td></tr>
<tr><td>R13</td><td><code>fk_trip_events_trip</code>, <code>ck_trip_events_status</code>; "chỉ thêm" là quy ước ứng dụng, P01 chưa có công cụ chặn UPDATE (quyền ở Phần 5)</td></tr>
<tr><td>R14</td><td>Không CHECK một dòng nào diễn đạt được ⇒ truy vấn ở D4 (MS-008 bàn các lựa chọn khác)</td></tr>
</tbody>
</table>
<p>Quyết định đặc thù SQL Server nằm ở R12. Clustered index quyết định thứ tự lưu của cả bảng (<a href="/articles/mssql-bang-khoa-chinh-clustered-index">MS-003</a>); khoá GUID ngẫu nhiên làm clustered key thì mỗi chuyến mới chèn vào giữa bảng, nên lab cluster theo <code>code</code> tăng dần. Tệp tự in lại cấu trúc để đối chiếu:</p>
<pre><code class="language-text">index_name|type_desc|la_pk|duy_nhat|cot_khoa
----------|---------|-----|--------|--------
uq_trips_code|CLUSTERED|0|1|code
pk_trips|NONCLUSTERED|1|1|id
ix_trips_city_requested|NONCLUSTERED|0|0|city_id, requested_at
ix_trips_customer_requested|NONCLUSTERED|0|0|customer_id, requested_at
ix_trips_driver_id|NONCLUSTERED|0|0|driver_id</code></pre>
<p>Kiểu dữ liệu: tiền <code>int</code> đồng (không <code>money</code> vì 4 chữ số lẻ và phép chia làm tròn bất ngờ, không <code>float</code> vì sai số nhị phân); thời điểm <code>datetimeoffset(3)</code>; chữ người đọc <code>nvarchar</code>, mã máy đọc <code>varchar</code>; định danh <code>uniqueidentifier</code> kèm <code>code</code> dễ đọc. Lý do chi tiết ở <a href="/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode">MS-004</a>. Tệp còn ghi 15 chú thích bằng extended property <code>MS_Description</code>. Muốn so DDL của bạn với bản sinh tự động, mở tab ERD của DBStudio và xem panel "DDL" của từng bảng; DDL DBStudio sinh ra chạy lại được và xếp theo thứ tự phụ thuộc.</p>
</details>

### 3.2 D2 — Fixture hợp lệ (`00-reset.sql`, `02-fixture-valid.sql`)

<details><summary><b>Lời giải</b></summary>
<p><code>00-reset.sql</code> đóng mọi phiên đang dùng <code>ToiNoi</code>, xoá và tạo lại database, nên mọi tệp chạy lại được từ đầu. Tệp đó cũng tạo schema <code>lab</code> với hai thủ tục <code>lab.expect_error</code> và <code>lab.expect_value</code>, mỗi lần gọi in một dòng <code>PASS</code> hoặc <code>FAIL</code>. <code>02</code> nạp 3 thành phố, 6 khách, 6 tài xế, 3 quán và 15 chuyến: ảnh chụp lúc 08:30 ngày 03/11/2026 giờ VN. GUID ghi sẵn, mọi mốc thời gian có <code>+07:00</code>; mã chuyến không ghi tay mà để IDENTITY cấp theo thứ tự dòng, và tệp kiểm điều đó:</p>
<pre><code class="language-text">PASS trips sau khi nạp = 15
PASS chuyến có id lệch mã = 0
PASS IDENT_CURRENT(trips) = 100014
status|ride|food|parcel|tong
------|----|----|------|----
requested|1|0|0|1
searching|0|1|0|1
accepted|1|0|0|1
arriving|0|0|1|1
picked_up|0|1|0|1
in_transit|1|0|0|1
completed|3|2|2|7
cancelled|1|1|0|2</code></pre>
<p>Ca biên có chủ đích: 100003 huỷ <b>sau</b> khi nhận (giữ tài xế), 100006 huỷ <b>trước</b> khi có người nhận; 100003 và 100004 cùng giờ đặt 12:00 (thử sắp xếp ở <a href="/articles/mssql-select-loc-sap-xep-top-offset">MS-007</a>); 100007 đặt 23:50 ngày 02/11, 100008 đặt 00:10 ngày 03/11 giờ VN (vẫn là 02/11 theo UTC); 100008 cước 0 đ; bốn tài xế <code>busy</code> mỗi người đúng một chuyến đang chạy. Lịch sử <code>trip_events</code> của 15 chuyến này cố ý không nạp (dữ liệu chuyển từ hệ cũ chỉ có trạng thái cuối); <code>04-lifecycle.sql</code> ghi lịch sử đầy đủ cho chuyến mới.</p>
</details>

### 3.3 D3 — Fixture sai (`03-fixture-invalid.sql`)

<details><summary><b>Lời giải</b></summary>
<p>34 câu sai, mỗi câu gọi qua <code>lab.expect_error</code> với số hiệu lỗi và tên mong đợi. Thủ tục chạy câu lệnh trong transaction riêng rồi luôn <code>ROLLBACK</code>, nên dù câu lệnh có lỡ thành công, dữ liệu vẫn nguyên; khi đó dòng in ra là <code>FAIL … câu lệnh KHÔNG lỗi</code>.</p>
<table>
<thead><tr><th>Nhóm</th><th>Số ca</th><th>Số hiệu lỗi</th><th>Ví dụ</th></tr></thead>
<tbody>
<tr><td>NOT NULL</td><td>3</td><td>515</td><td>Khách thiếu số điện thoại; ghi <code>NULL</code> tường minh vào <code>status</code> dù có DEFAULT</td></tr>
<tr><td>PK / UNIQUE</td><td>4</td><td>2627</td><td>Trùng id và mã thành phố, số điện thoại, mã tài xế</td></tr>
<tr><td>FOREIGN KEY</td><td>3</td><td>547</td><td>Khách không tồn tại; xoá khách đang có chuyến</td></tr>
<tr><td>CHECK</td><td>17</td><td>547</td><td>Mỗi CHECK ít nhất một ca; R7 có hai ca; C17 nhận chuyến "trước giờ đặt" khi ghi khác độ lệch</td></tr>
<tr><td>IDENTITY</td><td>1</td><td>544</td><td>Ghi tay vào <code>code</code></td></tr>
<tr><td>Kiểu, độ dài, chuyển đổi</td><td>6</td><td>2628, 220, 8115, 245, 241, 8169</td><td>Tên 101 ký tự, <code>city_id</code> vượt <code>smallint</code>, cước vượt <code>int</code>, <code>'45.000đ'</code>, ngày 31/11, GUID sai dạng</td></tr>
</tbody>
</table>
<p>Một ca tiêu biểu: tên ràng buộc trong thông báo chính là quy tắc bị vi phạm.</p>
<pre><code class="language-text">PASS C17 nhận chuyến trước giờ đặt (khác múi giờ) — lỗi 547: The INSERT statement conflicted with the CHECK constraint "ck_trips_accepted_after_requested". The conflict occurred in database "ToiNoi", table "toinoi.trips".</code></pre>
<p>Ba hiện tượng nên ghi vào hồ sơ bàn giao. (1) Bẫy không báo lỗi: cùng chuỗi <code>'03/11/2026'</code> ra <code>2026-11-03</code> với <code>DATEFORMAT dmy</code> và <code>2026-03-11</code> với <code>mdy</code> (B1, B2). (2) Cuối tệp, số dòng vẫn là <code>3/6/6/3/15</code>: không câu sai nào lọt. (3) Các câu <code>INSERT</code> lỗi vẫn tiêu số IDENTITY, nên chuyến hợp lệ tiếp theo ở <code>04</code> nhận mã <b>100032</b>, không phải 100015: <code>PASS mã chuyến mới (100014 + 17 giá trị bị tiêu ở 03 + 1) = 100032</code>. Mã chuyến không bảo đảm liên tục. Tổng cộng <code>03</code> in 37 dòng <code>PASS</code> (34 ca sai, 2 ca bẫy, 1 kiểm số dòng).</p>
</details>

### 3.4 D4 — Truy vấn bất biến (`07-checks.sql`)

<details><summary><b>Lời giải</b></summary>
<p>Mỗi kiểm tra đếm số dòng <b>vi phạm</b> và gọi <code>lab.expect_value</code> với giá trị mong đợi 0. Chạy sau <code>00</code> → <code>06</code>:</p>
<pre><code class="language-text">PASS K1 tài xế khác thành phố với chuyến = 0
PASS K2 quán khác thành phố với chuyến = 0
PASS K3 tài xế có hơn một chuyến đang chạy = 0
PASS K4 drivers.status lệch chuyến đang chạy = 0
PASS K5 sự kiện cuối lệch trạng thái chuyến = 0
PASS K6 sự kiện trước giờ đặt = 0
PASS K7 updated_at nhỏ hơn mốc khác = 0
PASS K8 ràng buộc tắt / không tin cậy = 0
PASS K9 tên ràng buộc sai quy ước = 0</code></pre>
<table>
<thead><tr><th>Nhóm</th><th>Kiểm tra</th><th>Vì sao cần</th></tr></thead>
<tbody>
<tr><td>Quy tắc CHECK không diễn đạt được</td><td>K1–K7</td><td>Phủ R14: nhiều bảng (K1, K2, K4), nhiều dòng (K3, K5), giữa hai bảng theo thời gian (K6), giữa các cột mốc (K7)</td></tr>
<tr><td>Sức khoẻ của ràng buộc</td><td>K8, K9</td><td>K8 đếm CHECK/FK có <code>is_disabled = 1</code> hoặc <code>is_not_trusted = 1</code>: không ai quên một <code>WITH NOCHECK</code> (<a href="/articles/mssql-schema-migration-tuong-thich">MS-009</a>). K9 bắt tên tự sinh kiểu <code>DF__trips__statu__…</code></td></tr>
<tr><td>Quyết định cần người</td><td>K10, K11</td><td>Không có giá trị mong đợi cố định hoặc không đếm được bằng một số</td></tr>
</tbody>
</table>
<p>K7 viết <code>updated_at &lt; requested_at OR updated_at &lt; accepted_at OR updated_at &lt; completed_at</code>; so sánh với mốc NULL ra UNKNOWN, nên chuyến chưa nhận hay chưa xong không bị báo nhầm.</p>
<p><b>K10 — khoá ngoại thiếu index.</b> SQL Server không tự tạo index cho cột FK. Truy vấn liệt kê, không phán đúng sai:</p>
<pre><code class="language-text">khoa_ngoai|cot|co_index
----------|---|--------
fk_drivers_city|city_id|KHÔNG
fk_merchants_city|city_id|KHÔNG
fk_trip_events_trip|trip_id|có
fk_trips_city|city_id|có
fk_trips_customer|customer_id|có
fk_trips_driver|driver_id|có
fk_trips_merchant|merchant_id|KHÔNG</code></pre>
<p>Ba dòng <code>KHÔNG</code> là ba quyết định cần ghi vào hồ sơ. Thiếu index ở cột con thì mỗi lần xoá hoặc đổi khoá ở bảng cha, SQL Server phải quét bảng con để kiểm còn dòng tham chiếu hay không. <code>fk_trips_merchant</code> đáng xử lý nhất: <code>trips</code> là bảng lớn nhất, và màn hình quán cần "đơn của quán X"; chỉ đơn <code>food</code> có quán, nên một filtered index <code>WHERE merchant_id IS NOT NULL</code> nhỏ hơn index thường. <code>fk_drivers_city</code> và <code>fk_merchants_city</code> trỏ tới danh mục 3 dòng hiếm khi xoá; thêm index chỉ khi có truy vấn cần, ví dụ điều phối "tài xế rảnh ở thành phố X" có lẽ cần index <code>(city_id, status)</code> chứ không phải <code>(city_id)</code>. Đây là phán đoán, chưa phải số liệu đo: Phần 3 sẽ đo lợi ích và chi phí ghi. Muốn gửi danh sách này cho người duyệt, export result set ra CSV hoặc XLSX từ DBStudio.</p>
<p><b>K11 — <code>DBCC CHECKCONSTRAINTS WITH ALL_CONSTRAINTS, NO_INFOMSGS</code>.</b> Quét lại mọi CHECK và FK của database trên dữ liệu hiện có, kể cả ràng buộc đang tắt; không in gì nghĩa là không dòng nào vi phạm, rồi tệp <code>PRINT</code> dòng <code>PASS K11 DBCC CHECKCONSTRAINTS không báo vi phạm</code>. K11 là bằng chứng độc lập với DDL: hữu ích sau khi nạp dữ liệu với ràng buộc tắt. Nhưng nó không biến ràng buộc thành tin cậy (MS-009), nên vẫn cần K8.</p>
<p>Kiểm kê cuối, sau migration <code>06</code> (nên <code>drivers</code> có 5 CHECK):</p>
<pre><code class="language-text">bang|pk|fk|uq|ck|df
----|--|--|--|--|--
cities|1|0|1|2|0
customers|1|0|1|2|2
drivers|1|1|1|5|3
merchants|1|1|0|1|2
trip_events|1|1|0|1|1
trips|1|4|1|10|4
PASS số dòng cuối cities/customers/drivers/merchants/trips/trip_events = 3/6/7/3/16/7</code></pre>
</details>

### 3.5 D5 — Bằng chứng bàn giao

<details><summary><b>Lời giải</b></summary>
<p>Chạy từ database trống (<code>00</code> xoá và tạo lại <code>ToiNoi</code>), tổng kết cuối của <code>npm run p01</code> trong <code>OUTPUT.txt</code>:</p>
<pre><code class="language-text">-- [lab] File đã chạy: 8
-- [lab] PASS: 80
-- [lab] FAIL: 0
-- [lab] KẾT QUẢ: ĐẠT</code></pre>
<p>80 PASS = 7 ở <code>02</code> + 37 ở <code>03</code> + 14 ở <code>04</code> (vòng đời, gồm lỗi 333 khi <code>OUTPUT INTO</code> thẳng bảng có CHECK và lỗi 547 khi xoá chuyến còn lịch sử) + 3 ở <code>05</code> + 8 ở <code>06</code> (migration) + 11 ở <code>07</code>. Runner đếm dòng bắt đầu bằng <code>PASS </code>/<code>FAIL </code>, và với <code>sqlcmd -b</code>, một lỗi không được bắt sẽ dừng tệp và runner báo FAIL, nên lỗi ngoài kỳ vọng không thể lẫn vào kết quả. <code>test/p01.test.mjs</code> có ba test: chạy lại <code>p01</code> và so từng dòng với <code>OUTPUT.txt</code>; chạy rollback của <code>06</code> hai lần liên tiếp; chạy thí nghiệm kiểu dữ liệu của MS-004. Phiên bản ghi ngay đầu <code>OUTPUT.txt</code>:</p>
<pre><code class="language-text">phien_ban|cu|edition
---------|--|-------
16.0.4325.4|CU28|Developer Edition (64-bit)</code></pre>
<p>Phần "chưa kiểm" phải ghi thẳng: mới chạy trên Developer Edition trong Docker trên Linux, chưa trên Standard hay trên Windows; mỗi tệp là một phiên, nên không có bằng chứng nào về khoá hay đồng thời; R13 "chỉ thêm" và R14 chỉ được canh bằng truy vấn, không bị chặn lúc ghi; dữ liệu chỉ có 16 chuyến, chưa nói gì về hiệu năng.</p>
</details>

## 4. Tiêu chí tự chấm

Mỗi tiêu chí 10 điểm. Từ 80 điểm là đạt để sang Phần 2; tiêu chí 1, 4 và 8 là bắt buộc.

| # | Tiêu chí | Đạt khi |
|---|---|---|
| 1 | Chạy lại được | Từ database trống, chạy toàn bộ hai lần liên tiếp đều cho cùng kết quả |
| 2 | Đặt tên ràng buộc | Mọi PK/FK/UNIQUE/CHECK/DEFAULT có tên đúng quy ước; truy vấn kiểu K9 trả 0 |
| 3 | Kiểu dữ liệu có lý do | Tiền, thời điểm, chuỗi tiếng Việt, định danh được chọn và giải thích; không `float`/`money` cho tiền, không `datetime` cho mốc sự kiện |
| 4 | Phủ quy tắc | Mỗi R1–R13 có ít nhất một ràng buộc, hoặc ghi rõ vì sao chỉ canh bằng truy vấn |
| 5 | Một CHECK một quy tắc | Không CHECK nào gộp hai quy tắc khiến thông báo 547 mơ hồ |
| 6 | Fixture hợp lệ đủ ca | 8 trạng thái, 3 loại, có ca biên (cước 0, huỷ trước/sau khi nhận, qua nửa đêm giờ VN, cùng giờ đặt) |
| 7 | Fixture sai chính xác | Mỗi câu vi phạm đúng một quy tắc; kỳ vọng ghi cả số hiệu lỗi và tên ràng buộc (hoặc tên cột với 515, 2628) |
| 8 | Không câu sai nào lọt | Số dòng sau fixture sai không đổi; không có ca "lẽ ra lỗi mà chạy được" |
| 9 | Truy vấn bất biến | Phủ R14, có K8 và K9 trả 0, có danh sách FK thiếu index kèm quyết định |
| 10 | Bằng chứng trung thực | Có kết quả chạy thật, phiên bản và CU, danh sách điều chưa kiểm |

## 5. Bẫy hay gặp khi tự làm

- **Fixture sai vi phạm hai quy tắc cùng lúc.** Ví dụ thêm thành phố mã `'hn'`: vừa trùng `uq_cities_code` (collation CI coi `'hn'` = `'HN'`) vừa sai `ck_cities_code_format`. Lab dùng `'HN'` cho ca trùng và `'ct'` cho ca chữ thường.
- **Quên collation khi viết CHECK định dạng.** `code LIKE '[A-Z][A-Z]'` trên collation CI nhận cả chữ thường; ca sai C1 sẽ "lẽ ra lỗi mà chạy được".
- **Thiếu `N` trước chuỗi tiếng Việt** trong fixture. Không lỗi nào báo; chữ "ễ" thành "?".
- **Tin rằng mã chuyến liên tục.** Đừng viết kiểm tra kiểu "mã kế tiếp phải là 100015".
- **`OUTPUT … INTO` thẳng vào `trip_events`.** Bảng đích của `OUTPUT INTO` không được có CHECK hay FK đang bật (lỗi 333 trong `04`); ghi qua biến bảng rồi `INSERT`.
- **Bật lại ràng buộc sau khi nạp dữ liệu bằng `CHECK CONSTRAINT` trơn.** Ràng buộc vẫn không tin cậy; K8 sẽ FAIL.
- **Fixture phụ thuộc giờ chạy.** `SYSDATETIMEOFFSET()` trong fixture làm kết quả mỗi lần một khác; ghi thời điểm tường minh có `+07:00`.
- **Bỏ quên K1–K7.** Schema qua mọi CHECK vẫn có thể chứa tài xế Hà Nội chạy chuyến ở Đà Nẵng.

## 6. Tổng kết Phần 1

| Bài | Bạn đã có gì cho sản phẩm cuối phần |
|---|---|
| [MS-001 · SQL Server 2022 là gì và nằm ở đâu trong Tới Nơi?](/articles/mssql-la-gi-va-vai-tro-trong-toi-noi) | Phân biệt instance, database, schema, bảng; edition; SQL Server trên Linux và container |
| [MS-002 · Dựng SQL Server 2022 bằng Docker và kết nối bằng DBStudio](/articles/mssql-cai-dat-docker-dbstudio) | Lab chạy được, đọc phiên bản, dọn môi trường: nền của D5 |
| [MS-003 · Bảng, khoá chính và clustered index](/articles/mssql-bang-khoa-chinh-clustered-index) | Chọn khoá chính và clustered key cho `trips`: R12 |
| [MS-004 · Kiểu dữ liệu cho tiền, thời gian, chuỗi tiếng Việt và định danh](/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode) | Data dictionary có đơn vị, độ lệch giờ, `N'…'`: tiêu chí 3 |
| [MS-005 · CREATE TABLE, ràng buộc và IDENTITY](/articles/mssql-create-table-constraints) | DDL có ràng buộc đặt tên, fixture sai theo số hiệu lỗi: D1, D3 |
| [MS-006 · INSERT, UPDATE, DELETE và mệnh đề OUTPUT](/articles/mssql-insert-update-delete-output) | Vòng đời một chuyến, ghi lịch sử cùng transaction: nguồn của K4, K5 |
| [MS-007 · SELECT: lọc, sắp xếp, TOP và OFFSET/FETCH](/articles/mssql-select-loc-sap-xep-top-offset) | Lọc theo giờ VN nửa mở, thứ tự xác định, collation tiếng Việt |
| [MS-008 · Chuẩn hoá và ranh giới mô hình dữ liệu](/articles/mssql-mo-hinh-quan-he-chuan-hoa) | Hiểu vì sao R14 tồn tại và các cách giữ quy tắc liên bảng |
| [MS-009 · Thay đổi schema tương thích](/articles/mssql-schema-migration-tuong-thich) | Expand → backfill → contract, ràng buộc tin cậy, rollback: phần cộng điểm và K8 |
| MS-010 · Bài này | Bàn giao D1–D5 có bằng chứng |

Sản phẩm của phần này đúng như khung đã đặt: schema tối thiểu cho khách, tài xế, quán, chuyến và lịch sử trạng thái, fixture hợp lệ và sai, kiểm tra bất biến, tất cả chạy trên SQL Server 2022 thật. Những gì **chưa** có cũng được nêu rõ: chưa có truy vấn tổng hợp và đối soát (Phần 2), chưa đo index hay execution plan (Phần 3), chưa có hai phiên tranh nhau một chuyến (Phần 4).

Nếu bạn học cả series PostgreSQL, bài cùng vai trò ở đó là [PG-010 · Thực hành: schema chuyến xe có kiểm tra ràng buộc](/articles/postgresql-thuc-hanh-schema-chuyen-xe). Cùng dữ liệu, nhưng khác ở những chỗ đáng ghi vào hồ sơ: SQL Server kiểm lỗi theo số hiệu (515, 2627, 547…) thay cho SQLSTATE, chú thích bằng `MS_Description` thay cho `COMMENT ON`, và ràng buộc chưa kiểm dữ liệu cũ là `WITH NOCHECK` (`is_not_trusted`) thay cho `NOT VALID`.

## 7. Bài tập mở rộng

**Bài 1.** Biến K2 (quán khác thành phố) từ truy vấn canh thành ràng buộc schema, áp lên `trips` đã có dữ liệu theo cách của MS-009, và viết một ca sai theo kiểu `lab.expect_error` chứng minh nó chặn được.

<details><summary><b>Lời giải</b></summary>
<p>Không có trong lab; đã chạy trên bản sao schema lab sau <code>01</code> → <code>02</code> trong database nháp <code>Scratch_ms008</code>:</p>
<pre><code class="language-sql">ALTER TABLE toinoi.merchants ADD CONSTRAINT uq_merchants_id_city UNIQUE (id, city_id);
ALTER TABLE toinoi.trips WITH NOCHECK ADD CONSTRAINT fk_trips_merchant_city
    FOREIGN KEY (merchant_id, city_id) REFERENCES toinoi.merchants (id, city_id);
ALTER TABLE toinoi.trips WITH CHECK CHECK CONSTRAINT fk_trips_merchant_city;
SELECT name, is_not_trusted FROM sys.foreign_keys WHERE name = N'fk_trips_merchant_city';
EXEC lab.expect_error N'X1 đơn đồ ăn SG đặt quán Hà Nội',
     N'INSERT INTO toinoi.trips (kind, city_id, customer_id, merchant_id, fare) VALUES (''food'', 1, ''c0000000-0000-4000-8000-000000000001'', ''e0000000-0000-4000-8000-000000000002'', 50000)',
     547, N'fk_trips_merchant_city';</code></pre>
<pre><code class="language-text">name|is_not_trusted
----|--------------
fk_trips_merchant_city|0
PASS X1 đơn đồ ăn SG đặt quán Hà Nội — lỗi 547: The INSERT statement conflicted with the FOREIGN KEY constraint "fk_trips_merchant_city". The conflict occurred in database "Scratch_ms008", table "toinoi.merchants".</code></pre>
<p>Fixture sạch nên <code>WITH CHECK CHECK</code> thành công và K8 vẫn 0. Chuyến <code>ride</code>/<code>parcel</code> có <code>merchant_id</code> NULL nên FK không kiểm, đúng ý vì R7 đã lo phần đó. Đánh đổi: thêm một UNIQUE (kèm index) trên <code>merchants</code>, và quán đổi thành phố thì phải xử lý cả chuyến cũ. Đừng làm y hệt cho tài xế: MS-008 mục 6.2 cho thấy FK hai cột chặn tài xế chuyển thành phố.</p>
</details>

**Bài 2.** Biến K3 (một tài xế tối đa một chuyến đang chạy) thành ràng buộc lúc ghi, kèm một ca sai. Ghi rõ điều kiện vận hành mà thay đổi này đòi hỏi.

<details><summary><b>Lời giải</b></summary>
<p>Filtered unique index (MS-008 mục 6.3), chạy trên cùng bản sao:</p>
<pre><code class="language-sql">CREATE UNIQUE INDEX ux_trips_driver_running ON toinoi.trips (driver_id)
    WHERE status IN ('accepted', 'arriving', 'picked_up', 'in_transit');
GO
EXEC lab.expect_error N'X2 tài xế 8001 nhận chuyến thứ hai khi đang chạy 100012',
     N'UPDATE toinoi.trips SET status = ''accepted'', driver_id = ''d0000000-0000-4000-8000-000000008001'', accepted_at = ''2026-11-03 08:30 +07:00'', updated_at = ''2026-11-03 08:30 +07:00'' WHERE code = 100014',
     2601, N'ux_trips_driver_running';</code></pre>
<pre><code class="language-text">PASS X2 tài xế 8001 nhận chuyến thứ hai khi đang chạy 100012 — lỗi 2601: Cannot insert duplicate key row in object 'toinoi.trips' with unique index 'ux_trips_driver_running'. The duplicate key value is (d0000000-0000-4000-8000-000000008001).</code></pre>
<p>Số hiệu là 2601 (unique index), không phải 2627 (UNIQUE constraint). Điều kiện vận hành: mọi phiên ghi <code>trips</code> phải chạy với <code>QUOTED_IDENTIFIER ON</code> và các tuỳ chọn <code>SET</code> mà filtered index đòi hỏi, nếu không mọi câu ghi lỗi 1934; lab chạy <code>sqlcmd -I</code> vì lý do này. K9 không bắt tên index, nên đặt tên theo quy ước <code>ux_</code> là việc của người review.</p>
</details>

**Bài 3.** Quyết định cho ba dòng `KHÔNG` của K10: tạo index nào, rồi chạy lại K10. Truy vấn K10 có đủ để kiểm FK hai cột ở bài 1 không?

<details><summary><b>Lời giải</b></summary>
<p>Một phương án (chạy trên cùng bản sao, sau bài 1 và 2): <code>CREATE INDEX ix_trips_merchant_id ON toinoi.trips (merchant_id) WHERE merchant_id IS NOT NULL;</code> và <code>CREATE INDEX ix_drivers_city_id ON toinoi.drivers (city_id);</code>, để <code>fk_merchants_city</code> lại vì bảng quán nhỏ. K10 chạy lại:</p>
<pre><code class="language-text">khoa_ngoai|cot|co_index
----------|---|--------
fk_drivers_city|city_id|có
fk_merchants_city|city_id|KHÔNG
fk_trip_events_trip|trip_id|có
fk_trips_city|city_id|có
fk_trips_customer|customer_id|có
fk_trips_driver|driver_id|có
fk_trips_merchant|merchant_id|có
fk_trips_merchant_city|merchant_id|có
fk_trips_merchant_city|city_id|có</code></pre>
<p>K10 xét từng cột FK riêng lẻ, chỉ hỏi "có index nào bắt đầu bằng cột này không". Với FK hai cột nó báo "có" cho cả hai cột, nhờ hai index khác nhau (<code>city_id</code> đứng đầu <code>ix_trips_city_requested</code>), dù không index nào có đúng cặp <code>(merchant_id, city_id)</code>. Index lọc trên <code>merchant_id</code> vẫn đủ để tìm chuyến của một quán; nhưng hãy ghi giới hạn này cạnh truy vấn, và khi schema có FK nhiều cột thì viết lại K10 để so cả cặp cột. <code>DBCC CHECKCONSTRAINTS WITH ALL_CONSTRAINTS, NO_INFOMSGS</code> chạy sau các thay đổi này không báo vi phạm.</p>
</details>

## 8. Đọc tiếp

- [Microsoft Learn · Unique constraints and check constraints](https://learn.microsoft.com/sql/relational-databases/tables/unique-constraints-and-check-constraints) và [Primary and foreign key constraints](https://learn.microsoft.com/sql/relational-databases/tables/primary-and-foreign-key-constraints).
- [Microsoft Learn · DBCC CHECKCONSTRAINTS](https://learn.microsoft.com/sql/t-sql/database-console-commands/dbcc-checkconstraints-transact-sql): `ALL_CONSTRAINTS`, `ALL_ERRORMSGS`, kết quả dạng Table/Constraint/Where.
- [Microsoft Learn · sys.check_constraints](https://learn.microsoft.com/sql/relational-databases/system-catalog-views/sys-check-constraints-transact-sql): cột `is_disabled`, `is_not_trusted` dùng trong K8.
- Lab: `examples/toi-noi-mssql/p01`: `README.md`, các tệp `00` … `07`, `OUTPUT.txt`, `test/p01.test.mjs`.

Bài tiếp theo, mở đầu Phần 2 · T-SQL cho nghiệp vụ (đang viết): NULL, CASE, ISNULL và COALESCE trong dữ liệu chuyến.
