---
slug: postgresql-thuc-hanh-schema-chuyen-xe
title: "Thực hành: schema chuyến xe có kiểm tra ràng buộc"
summary: "Tới Nơi sắp đổ dữ liệu chuyến xe vào PostgreSQL và không ai chấp nhận câu “schema chạy rồi”. Bài thực hành cuối Phần 1: bàn giao DDL, fixture hợp lệ và sai, truy vấn bất biến cùng bằng chứng chạy lại được."
category: postgresql
level: advanced
tags: [postgresql, pg-010, thuc-hanh, schema, constraint, fixture, toi-noi]
language: vi
---

Đội nền tảng Tới Nơi chuẩn bị chuyển dữ liệu chuyến xe từ hệ cũ sang PostgreSQL. Trong buổi duyệt thiết kế, trưởng nhóm vận hành hỏi đúng một câu: "Làm sao tôi biết một chuyến `completed` mà không có tài xế sẽ không lọt vào?". Câu trả lời "schema chạy rồi, ứng dụng có kiểm tra" không được chấp nhận. Thứ cần bàn giao là **bằng chứng**: chạy từ database trống, dữ liệu đúng đi qua, từng loại dữ liệu sai bị chặn với đúng tên quy tắc, và những quy tắc schema chưa chặn được thì có truy vấn canh.

Đây là bài thực hành cuối Phần 1. Bạn nhận đề bài, tự làm, rồi mới mở lời giải mẫu dựa trên lab và tự chấm theo tiêu chí ở mục 4.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-010 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: toàn bộ PG-001 … PG-009 (danh sách ở mục 6), đặc biệt <a href="/articles/postgresql-create-table-constraints">CREATE TABLE và ràng buộc</a> và <a href="/articles/postgresql-mo-hinh-quan-he-chuan-hoa">Chuẩn hoá và ranh giới mô hình</a>. Lab: <code>examples/toi-noi-postgresql/p01</code>. Sau bài này, bạn bàn giao được DDL, fixture hợp lệ/sai và bằng chứng kiểm tra cho schema chuyến xe.</p>
</div>

## 1. Đề bài

### Bối cảnh

Schema phục vụ ba dịch vụ của [Tới Nơi](/articles/du-an-mau-toi-noi) — đặt xe (`ride`), giao đồ ăn (`food`), giao bưu kiện (`parcel`) — gồm sáu bảng: `cities`, `customers`, `drivers`, `merchants`, `trips`, `trip_events`. Tên bảng, cột và kiểu bám lược đồ lõi của hồ sơ dự án; bạn được thêm ràng buộc nhưng không đổi nghĩa cột. Tiền tính bằng đồng (số nguyên), thời điểm là khoảnh khắc tuyệt đối hiển thị theo `Asia/Ho_Chi_Minh`. Mọi dữ liệu là hư cấu.

### Quy tắc nghiệp vụ phải mã hoá

| Mã | Quy tắc |
|---|---|
| R1 | Thành phố có mã 2 chữ in hoa, duy nhất; tên không rỗng |
| R2 | Khách đăng nhập bằng số điện thoại dạng `0` + 9 chữ số, duy nhất; họ tên không rỗng (kể cả toàn khoảng trắng) |
| R3 | Tài xế có mã nội bộ dương, duy nhất, thuộc một thành phố có thật; loại xe `bike`/`car`; trạng thái `offline`/`online`/`busy`, mặc định `offline` |
| R4 | Quán thuộc một thành phố có thật; tên không rỗng |
| R5 | Chuyến có loại thuộc 3 dịch vụ, trạng thái thuộc 8 trạng thái vòng đời, mặc định `requested`; trỏ tới khách, thành phố (và tài xế, quán nếu có) có thật |
| R6 | Cước là số nguyên đồng, không âm (0 đ hợp lệ — khuyến mãi) |
| R7 | Đơn `food` phải có quán; loại khác không gắn quán |
| R8 | Có tài xế khi và chỉ khi có mốc nhận chuyến `accepted_at` |
| R9 | `requested`/`searching` chưa có tài xế; từ `accepted` tới `completed` phải có tài xế; `cancelled` tuỳ (huỷ trước hay sau khi nhận) |
| R10 | Có `completed_at` khi và chỉ khi trạng thái `completed` |
| R11 | `accepted_at` không sớm hơn `requested_at`; `completed_at` không sớm hơn `accepted_at` |
| R12 | Mã chuyến dễ đọc do database cấp từ 100000, không ghi tay |
| R13 | Lịch sử trạng thái chỉ thêm, trạng thái thuộc 8 giá trị, trỏ tới chuyến có thật |
| R14 | Quy tắc nhiều bảng: tài xế và quán cùng thành phố với chuyến; một tài xế tối đa một chuyến đang chạy; `busy` khớp với chuyến đang chạy; `updated_at` không sớm hơn mốc mới nhất của dòng |

### Sản phẩm bàn giao

| Mã | Sản phẩm | Yêu cầu tối thiểu |
|---|---|---|
| D1 | DDL | Sáu bảng; mọi ràng buộc được **đặt tên** theo quy ước `pk_/fk_/uq_/ck_`; chọn kiểu tiền, thời gian, định danh có lý do; chú thích `COMMENT ON` cho cột khó hiểu |
| D2 | Fixture hợp lệ | Chạy lại được từ trạng thái trống; giá trị cố định (UUID, thời gian có múi giờ); phủ đủ 8 trạng thái và 3 loại; có ca biên |
| D3 | Fixture sai | Mỗi câu vi phạm **đúng một** quy tắc, ghi trước SQLSTATE và tên ràng buộc mong đợi; mỗi quy tắc R1–R13 ít nhất một ca; có ca sai kiểu dữ liệu |
| D4 | Truy vấn bất biến | Mỗi truy vấn tìm dòng vi phạm, kết quả đúng là 0 dòng; phủ R14 và kiểm chính schema (ràng buộc chưa validate, tên sai quy ước) |
| D5 | Bằng chứng | Kết quả chạy từ database trống với tổng kết 0 lỗi ngoài kỳ vọng; ghi phiên bản PostgreSQL và những gì **chưa** kiểm |

Tuỳ chọn cộng điểm: một migration tương thích ngược kèm rollback theo [PG-009](/articles/postgresql-schema-migration-tuong-thich).

Chỉ dùng kiến thức Phần 1. Truy vấn của D4 cần `JOIN`, `NOT EXISTS`, `GROUP BY` (Phần 2 dạy kỹ) — nếu chưa tự tin, viết bằng lời từng truy vấn cần tìm gì, rồi đối chiếu với lời giải.

## 2. Gợi ý trước khi làm

Không cần xem lời giải, nhưng nên nhớ những điều Phần 1 đã chỉ ra:

- Viết quy tắc thành câu "nếu A thì B" trước, rồi mới thành SQL `NOT A OR B`. Quy tắc "khi và chỉ khi" viết được thành so sánh hai biểu thức boolean.
- CHECK **chấp nhận** khi biểu thức ra NULL. Quy tắc "phải có" cần `IS NULL`/`IS NOT NULL` tường minh.
- Mỗi CHECK giữ **một** quy tắc: khi một dòng vi phạm nhiều CHECK, PostgreSQL chỉ báo một tên, và tài liệu không hứa cái nào.
- Dòng gốc hợp lệ cho fixture sai: viết một câu INSERT hợp lệ, chạy thử rồi huỷ; mỗi ca sai chỉ đổi đúng một giá trị so với nó.
- `NOT NULL` ở PostgreSQL 17 không có tên ràng buộc — thông báo lỗi chỉ nêu tên cột.
- Viết ngày giờ theo ISO 8601 có múi giờ: `'03/11/2026'::date` không lỗi mà ra ngày 11 tháng 3.

Chạy lab và bộ test để có môi trường đối chiếu ([PG-002](/articles/postgresql-cai-dat-lab-psql-ket-noi) hướng dẫn chi tiết):

```bash
cd examples/toi-noi-postgresql
npm ci
npm run p01      # 00 → 07 trên PGlite (PostgreSQL 17.5)
npm test
```

## 3. Lời giải mẫu

Lời giải là chính các tệp của lab. Hãy mở từng phần **sau khi** đã có bản của bạn.

### 3.1 D1 — DDL (`01-schema.sql`)

<details><summary><b>Lời giải</b></summary>
<p>Bảng đối chiếu quy tắc → ràng buộc. Trước migration <code>06</code>, <code>pg_constraint</code> có 37 ràng buộc (6 PK, 7 FK, 4 UNIQUE, 20 CHECK), cộng các <code>NOT NULL</code> không nằm trong catalog đó ở bản 17.</p>
<table>
<thead><tr><th>Quy tắc</th><th>Ràng buộc</th></tr></thead>
<tbody>
<tr><td>R1</td><td><code>pk_cities</code>, <code>uq_cities_code</code>, <code>ck_cities_code_format</code> (<code>~ '^[A-Z]{2}$'</code>), <code>ck_cities_name_not_blank</code> (<code>btrim(name) &lt;&gt; ''</code>)</td></tr>
<tr><td>R2</td><td><code>uq_customers_phone</code>, <code>ck_customers_phone_format</code> (<code>^0[0-9]{9}$</code> — chỉ một dạng chuẩn để UNIQUE có nghĩa), <code>ck_customers_full_name_not_blank</code></td></tr>
<tr><td>R3</td><td><code>uq_drivers_code</code>, <code>ck_drivers_code_positive</code>, <code>fk_drivers_city</code>, <code>ck_drivers_vehicle_type</code>, <code>ck_drivers_status</code> + <code>DEFAULT 'offline'</code></td></tr>
<tr><td>R4</td><td><code>fk_merchants_city</code>, <code>ck_merchants_name_not_blank</code></td></tr>
<tr><td>R5</td><td><code>ck_trips_kind</code>, <code>ck_trips_status</code>, 4 FK <code>fk_trips_city/customer/driver/merchant</code></td></tr>
<tr><td>R6</td><td><code>fare integer</code> + <code>ck_trips_fare_non_negative</code></td></tr>
<tr><td>R7</td><td><code>ck_trips_merchant_only_food</code>: <code>(kind = 'food') = (merchant_id IS NOT NULL)</code></td></tr>
<tr><td>R8–R11</td><td><code>ck_trips_driver_accepted_together</code>, <code>ck_trips_unassigned_no_driver</code>, <code>ck_trips_assigned_has_driver</code>, <code>ck_trips_completed_at_by_status</code>, <code>ck_trips_accepted_after_requested</code>, <code>ck_trips_completed_after_accepted</code></td></tr>
<tr><td>R12</td><td><code>code bigint GENERATED ALWAYS AS IDENTITY (START WITH 100000)</code> + <code>uq_trips_code</code></td></tr>
<tr><td>R13</td><td><code>fk_trip_events_trip</code>, <code>ck_trip_events_status</code>; "chỉ thêm" là quy ước ứng dụng — P01 chưa có công cụ chặn UPDATE (quyền ở Phần 5)</td></tr>
<tr><td>R14</td><td>Không CHECK nào diễn đạt được (cần so nhiều dòng/bảng) ⇒ truy vấn ở D4</td></tr>
</tbody>
</table>
<p>Phần lõi của <code>trips</code> — để ý mỗi CHECK một quy tắc, và hai quy tắc so sánh thời gian tự bỏ qua dòng có mốc NULL:</p>
<pre><code class="language-sql">CONSTRAINT ck_trips_driver_accepted_together CHECK ((driver_id IS NULL) = (accepted_at IS NULL)),
CONSTRAINT ck_trips_unassigned_no_driver CHECK (
  status NOT IN ('requested', 'searching') OR driver_id IS NULL),
CONSTRAINT ck_trips_assigned_has_driver CHECK (
  status NOT IN ('accepted', 'arriving', 'picked_up', 'in_transit', 'completed') OR driver_id IS NOT NULL),
CONSTRAINT ck_trips_completed_at_by_status CHECK ((status = 'completed') = (completed_at IS NOT NULL)),
CONSTRAINT ck_trips_accepted_after_requested CHECK (accepted_at &gt;= requested_at),
CONSTRAINT ck_trips_completed_after_accepted CHECK (completed_at &gt;= accepted_at)</code></pre>
<p>Kiểu dữ liệu: tiền <code>integer</code> đồng (không <code>real</code>/<code>double</code> vì sai số nhị phân, không <code>money</code> vì phụ thuộc <code>lc_monetary</code>); thời điểm <code>timestamptz</code>; định danh <code>uuid</code> từ <code>gen_random_uuid()</code> kèm <code>code</code> dễ đọc. Lý do chi tiết ở <a href="/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid">PG-004</a>. Tệp còn có 16 <code>COMMENT ON</code> và ba index giống hồ sơ (Phần 3 mới đo chúng).</p>
</details>

### 3.2 D2 — Fixture hợp lệ (`00-reset.sql`, `02-fixture-valid.sql`)

<details><summary><b>Lời giải</b></summary>
<p><code>00-reset.sql</code> xoá và dựng lại schema riêng <code>toinoi</code> nên mọi tệp chạy lại được từ đầu. <code>02</code> nạp 3 thành phố, 6 khách, 6 tài xế, 3 quán và 15 chuyến — ảnh chụp lúc 08:30 ngày 03/11/2026 giờ VN. UUID và thời gian ghi sẵn (có <code>+07</code>) để kết quả giống nhau mọi lần chạy; mã chuyến không ghi tay mà để IDENTITY cấp theo thứ tự dòng.</p>
<p>Ca biên có chủ đích: 100003 huỷ <b>sau</b> khi nhận (giữ tài xế), 100006 huỷ <b>trước</b> khi có người nhận; 100003 và 100004 cùng giờ đặt (thử sắp xếp ở PG-007); 100007 đặt 23:50 ngày 02/11, 100008 đặt 00:10 ngày 03/11 giờ VN (vẫn là 02/11 theo UTC); 100008 cước 0 đ; bốn tài xế <code>busy</code> mỗi người đúng một chuyến đang chạy. Kiểm nhanh ngay trong tệp (truy vấn dùng <code>FILTER</code>/<code>GROUP BY</code> của Phần 2):</p>
<pre><code class="language-text">   status   | ride | food | parcel | tong
------------+------+------+--------+------
 requested  |    1 |    0 |      0 |    1
 searching  |    0 |    1 |      0 |    1
 accepted   |    1 |    0 |      0 |    1
 arriving   |    0 |    0 |      1 |    1
 picked_up  |    0 |    1 |      0 |    1
 in_transit |    1 |    0 |      0 |    1
 completed  |    3 |    2 |      2 |    7
 cancelled  |    1 |    1 |      0 |    2
(8 rows)</code></pre>
</details>

### 3.3 D3 — Fixture sai (`03-fixture-invalid.sql`)

<details><summary><b>Lời giải</b></summary>
<p>Tệp mở bằng một chuyến "gốc" hợp lệ chạy trong giao dịch rồi <code>ROLLBACK</code> (chứng minh dòng gốc đúng mà không để lại dữ liệu), sau đó 33 câu sai, mỗi câu có dòng <code>-- expect-error:</code> ngay phía trên:</p>
<table>
<thead><tr><th>Nhóm</th><th>Số ca</th><th>SQLSTATE</th><th>Ví dụ</th></tr></thead>
<tbody>
<tr><td>NOT NULL</td><td>3</td><td>23502</td><td>Khách thiếu họ tên; ghi <code>NULL</code> tường minh vào <code>status</code> dù có DEFAULT</td></tr>
<tr><td>UNIQUE / PK</td><td>4</td><td>23505</td><td>Trùng số điện thoại, mã tài xế, mã thành phố, id thành phố</td></tr>
<tr><td>FOREIGN KEY</td><td>3</td><td>23503</td><td>Thành phố không tồn tại; xoá thành phố còn được tham chiếu</td></tr>
<tr><td>CHECK</td><td>17</td><td>23514</td><td>Mỗi CHECK ít nhất một ca; R7 có hai ca (đồ ăn không quán, đặt xe có quán)</td></tr>
<tr><td>IDENTITY</td><td>1</td><td>428C9</td><td>Ghi tay vào <code>code</code></td></tr>
<tr><td>Sai kiểu / định dạng</td><td>5</td><td>22P02, 22003, 22007, 22008</td><td><code>'45.000đ'</code> vào cột số, 3 tỷ đồng vượt <code>integer</code>, ngày 30/02, <code>'hôm qua'</code></td></tr>
</tbody>
</table>
<p>Một ca tiêu biểu — tên ràng buộc trong thông báo chính là quy tắc bị vi phạm:</p>
<pre><code class="language-text">ERROR:  new row for relation "trips" violates check constraint "ck_trips_status"
DETAIL:  Failing row contains (f0000000-0000-4000-8000-000000000012, 100020, ride, 1, c0000000-0000-4000-8000-000000000001, null, null, done, 40000, 2026-11-03 09:00:00+07, null, null, 2026-11-03 09:00:00+07).</code></pre>
<p>Hai hiện tượng nên ghi vào hồ sơ bàn giao. (1) Mã trong <code>DETAIL</code> là 100020 dù chỉ có 15 chuyến: câu INSERT lỗi ở bước ràng buộc vẫn tiêu số của IDENTITY, nên chuyến hợp lệ tiếp theo ở <code>04</code> nhận mã <b>100029</b> — mã chuyến không bảo đảm liên tục. (2) Bẫy không báo lỗi: <code>'03/11/2026'::date</code> ra <code>2026-03-11</code> với <code>DateStyle</code> <code>ISO, MDY</code>. Cuối tệp, số dòng vẫn là 3/6/6/15: không câu sai nào lọt.</p>
</details>

### 3.4 D4 — Truy vấn bất biến (`07-checks.sql`)

<details><summary><b>Lời giải</b></summary>
<p>13 truy vấn, mỗi truy vấn kỳ vọng <code>(0 rows)</code>, chia ba nhóm:</p>
<table>
<thead><tr><th>Nhóm</th><th>Truy vấn</th><th>Vì sao cần</th></tr></thead>
<tbody>
<tr><td>A · Quy tắc đã có ràng buộc</td><td>A1 mồ côi, A2 lịch sử mồ côi, A3 trạng thái ↔ mốc, A4 trùng khoá</td><td>Bằng chứng độc lập với DDL; hữu ích khi ràng buộc thêm bằng <code>NOT VALID</code> hoặc dữ liệu nhập từ nguồn chưa có ràng buộc</td></tr>
<tr><td>B · Quy tắc CHECK không diễn đạt được</td><td>B1 tài xế khác thành phố, B2 quán khác thành phố, B3 tài xế hai chuyến đang chạy, B4 <code>busy</code> lệch, B5 <code>updated_at</code> sớm hơn mốc, B6 nghi đặt trùng, B7 lịch sử không khớp chuyến</td><td>Phủ R14 — chỗ dữ liệu sai lọt vào thật</td></tr>
<tr><td>C · Bằng chứng về schema</td><td>C1 ràng buộc chưa validate, C2 tên sai quy ước; C3 kiểm kê, C4 số dòng (hai truy vấn này in số, không kỳ vọng 0)</td><td>Không ai "quên" một <code>NOT VALID</code> hay đặt tên tuỳ tiện</td></tr>
</tbody>
</table>
<p>Kiểm kê C3 sau toàn bộ lab (đã có migration <code>06</code>, nên <code>drivers</code> có 5 CHECK):</p>
<pre><code class="language-text">    bang     | p | f | u | c
-------------+---+---+---+----
 cities      | 1 | 0 | 1 |  2
 customers   | 1 | 0 | 1 |  2
 drivers     | 1 | 1 | 1 |  5
 merchants   | 1 | 1 | 0 |  1
 trip_events | 1 | 1 | 0 |  1
 trips       | 1 | 4 | 1 | 10
(6 rows)</code></pre>
<p>Vì sao nhóm B quan trọng: người dựng lab đã chạy script sinh dữ liệu của hồ sơ dự án (thu nhỏ còn 2.000 chuyến) qua schema này — qua mọi CHECK, nhưng trượt B1 ở 1.638 chuyến, B2 ở 780 và B5 ở 1.980 (số ghi trong <code>p01/README.md</code>). Dữ liệu "qua được ràng buộc" chưa chắc đúng nghiệp vụ.</p>
</details>

### 3.5 D5 — Bằng chứng bàn giao

<details><summary><b>Lời giải</b></summary>
<p>Chạy từ database trống (PGlite tạo database mới trong bộ nhớ mỗi lần), tổng kết cuối của <code>npm run p01</code>:</p>
<pre><code class="language-text">-- [lab] Câu lệnh đã chạy: 176
-- [lab] Lỗi mong đợi (expect-error): 38
-- [lab] Kiểm số dòng (expect-rows): 56
-- [lab] FAIL: 0
-- [lab] KẾT QUẢ: ĐẠT</code></pre>
<p>38 lỗi mong đợi = 33 ở <code>03</code> + 2 ở <code>04</code> (vòng đời) + 3 ở <code>06</code> (migration). Runner coi câu lẽ ra phải lỗi mà lại chạy được là <code>FAIL</code> — đó chính là cách phát hiện schema thiếu quy tắc. <code>npm test</code> báo <code>pass 6</code>, <code>fail 0</code>: gồm so <code>OUTPUT.txt</code> từng byte, đủ 9 SQLSTATE ở <code>03</code>, mọi truy vấn <code>07</code> trả 0 dòng, số dòng cuối 3/6/7/3/16/7, và chuỗi rollback → rollback → <code>06</code> → <code>07</code> của migration.</p>
<p>Phần "chưa kiểm" phải ghi thẳng: mới chạy trên PGlite 0.3.16 (PostgreSQL 17.5), <b>chưa</b> chạy trên server PostgreSQL thật; một kết nối nên không có bằng chứng nào về khoá hay đồng thời; quy tắc R13 "chỉ thêm" và R14 chỉ được canh bằng truy vấn, không bị chặn lúc ghi.</p>
</details>

## 4. Tiêu chí tự chấm

Mỗi tiêu chí 10 điểm. Từ 80 điểm là đạt để sang Phần 2; tiêu chí 1, 4 và 8 là bắt buộc.

| # | Tiêu chí | Đạt khi |
|---|---|---|
| 1 | Chạy lại được | Từ trạng thái trống, chạy toàn bộ hai lần liên tiếp đều cho cùng kết quả |
| 2 | Đặt tên ràng buộc | Mọi PK/FK/UNIQUE/CHECK có tên đúng quy ước; truy vấn kiểu C2 trả 0 dòng |
| 3 | Kiểu dữ liệu có lý do | Tiền, thời điểm, định danh được chọn và giải thích; không `float` cho tiền, không `timestamp` không múi giờ cho mốc sự kiện |
| 4 | Phủ quy tắc | Mỗi R1–R13 có ít nhất một ràng buộc, hoặc ghi rõ vì sao chỉ canh bằng truy vấn |
| 5 | Một CHECK một quy tắc | Không CHECK nào gộp hai quy tắc khiến thông báo lỗi mơ hồ |
| 6 | Fixture hợp lệ đủ ca | 8 trạng thái, 3 loại, có ca biên (cước 0, huỷ trước/sau nhận, qua nửa đêm giờ VN) |
| 7 | Fixture sai chính xác | Mỗi câu vi phạm đúng một quy tắc, kỳ vọng ghi cả SQLSTATE và tên ràng buộc (hoặc tên cột với 23502) |
| 8 | Không câu sai nào lọt | Số dòng sau fixture sai không đổi; không có ca "lẽ ra lỗi mà chạy được" |
| 9 | Truy vấn bất biến | Phủ R14 và hai kiểm tra về schema, đều trả 0 dòng trên fixture |
| 10 | Bằng chứng trung thực | Có output thật, phiên bản, và danh sách điều chưa kiểm |

## 5. Bẫy hay gặp khi tự làm

- **Fixture sai vi phạm hai quy tắc cùng lúc.** Ví dụ `status = 'done'` kèm tài xế: thông báo có thể nêu một CHECK khác với CHECK bạn định thử. Khi viết lab, người dựng đã gặp đúng chuyện này và phải tách CHECK.
- **Kỳ vọng tên cho lỗi NOT NULL.** Bản 17 không có tên ràng buộc NOT NULL; `pg_constraint` cũng không liệt kê chúng.
- **Tin rằng mã chuyến liên tục.** Đừng viết kiểm tra kiểu "mã kế tiếp phải là 100015".
- **Fixture phụ thuộc giờ chạy.** Dùng `now()` trong fixture làm kết quả mỗi lần một khác; ghi thời gian tường minh có `+07`.
- **Gọi một CHECK là "đã chặn" khi nó ra NULL.** `accepted_at >= requested_at` không chặn chuyến thiếu `accepted_at`; quy tắc "phải có" là việc của R8–R10.
- **Bỏ quên nhóm B.** Schema qua mọi CHECK vẫn có thể chứa tài xế Hà Nội chạy chuyến ở Đà Nẵng.

## 6. Tổng kết Phần 1

| Bài | Bạn đã có gì cho sản phẩm cuối phần |
|---|---|
| [PG-001 · PostgreSQL là gì và nằm ở đâu trong Tới Nơi?](/articles/postgresql-la-gi-va-vai-tro-trong-toi-noi) | Phân biệt DBMS, database, schema, bảng; vì sao dữ liệu chuyến cần một nguồn sự thật |
| [PG-002 · Dựng môi trường lab](/articles/postgresql-cai-dat-lab-psql-ket-noi) | Lab chạy được, ghi phiên bản và cách dọn — nền của D5 |
| [PG-003 · Bảng, dòng, cột và khoá](/articles/postgresql-bang-dong-cot-primary-foreign-key) | Quan hệ khách–chuyến–tài xế, PK/FK, `uuid` và `code` |
| [PG-004 · Kiểu dữ liệu cho tiền, thời gian, định danh](/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid) | Data dictionary có đơn vị, NULL, múi giờ — tiêu chí 3 |
| [PG-005 · CREATE TABLE và ràng buộc](/articles/postgresql-create-table-constraints) | DDL có ràng buộc đặt tên và fixture sai theo SQLSTATE — D1, D3 |
| [PG-006 · INSERT, UPDATE, DELETE với RETURNING](/articles/postgresql-insert-update-delete-returning) | Vòng đời một chuyến trên fixture chạy lại được |
| [PG-007 · SELECT: lọc, sắp xếp, giới hạn](/articles/postgresql-select-filter-order-limit) | Lọc theo giờ VN nửa hở, thứ tự xác định |
| [PG-008 · Chuẩn hoá và ranh giới mô hình](/articles/postgresql-mo-hinh-quan-he-chuan-hoa) | Hiểu vì sao R14 tồn tại và đánh đổi khi tách bảng |
| [PG-009 · Thay đổi schema tương thích](/articles/postgresql-schema-migration-tuong-thich) | Migration expand–contract và rollback — phần cộng điểm |

Sản phẩm của phần đúng như khung đã đặt: schema tối thiểu cho khách, tài xế và chuyến, fixture nhỏ, và kiểm tra dữ liệu sai. Những gì **chưa** có cũng rõ: chưa có truy vấn tổng hợp và đối soát (Phần 2), chưa đo index hay kế hoạch thực thi (Phần 3), chưa có hai phiên tranh nhau một chuyến (Phần 4).

## 7. Bài tập mở rộng

**Bài 1.** Biến B2 (quán khác thành phố) từ truy vấn canh thành ràng buộc schema, áp lên bảng `trips` đã có dữ liệu theo cách của PG-009, và viết một ca sai chứng minh nó chặn được.

<details><summary><b>Lời giải</b></summary>
<p>FK hai cột cần phía được tham chiếu có UNIQUE trên đúng hai cột đó. Không có trong lab; đã chạy sau <code>npm run p01</code> bằng <code>node run.mjs p01 &lt;tệp&gt;</code>:</p>
<pre><code class="language-sql">SET search_path TO toinoi;
ALTER TABLE merchants ADD CONSTRAINT uq_merchants_id_city UNIQUE (id, city_id);
ALTER TABLE trips
  ADD CONSTRAINT fk_trips_merchant_city FOREIGN KEY (merchant_id, city_id)
  REFERENCES merchants (id, city_id) NOT VALID;
ALTER TABLE trips VALIDATE CONSTRAINT fk_trips_merchant_city;
-- expect-error: 23503 fk_trips_merchant_city
INSERT INTO trips (id, kind, city_id, customer_id, merchant_id, status, fare, requested_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000040', 'food', 2, 'c0000000-0000-4000-8000-000000000001',
        'e0000000-0000-4000-8000-000000000001', 'requested', 40000,
        '2026-11-03 09:00:00+07', '2026-11-03 09:00:00+07');</code></pre>
<pre><code class="language-text">ERROR:  insert or update on table "trips" violates foreign key constraint "fk_trips_merchant_city"
DETAIL:  Key (merchant_id, city_id)=(e0000000-0000-4000-8000-000000000001, 2) is not present in table "merchants".</code></pre>
<p>Ba câu <code>ALTER</code> đều thành công vì fixture sạch. Chuyến đặt xe và bưu kiện có <code>merchant_id</code> NULL nên FK không kiểm (FK mặc định bỏ qua khi một cột khoá là NULL) — đúng ý, vì R7 đã lo phần đó. Đánh đổi: thêm một UNIQUE (kèm index) trên <code>merchants</code>, và quán đổi thành phố thì phải xử lý cả chuyến cũ. B1 làm tương tự với <code>drivers</code>, nhưng cần hỏi nghiệp vụ: tài xế chuyển thành phố thì chuyến cũ có còn "đúng" không?</p>
</details>

**Bài 2.** Script sinh dữ liệu của hồ sơ dự án qua mọi CHECK nhưng trượt B1, B2 và B5 (mục 3.4). Không cần chạy lại: đọc script trong [hồ sơ dự án](/articles/du-an-mau-toi-noi) và chỉ ra nguyên nhân từng lỗi, rồi đề xuất cách sửa script.

<details><summary><b>Lời giải</b></summary>
<p>B1: tài xế được chọn ngẫu nhiên trong toàn bộ mảng tài xế, không lọc theo <code>city_id</code> của chuyến. B2: quán cũng chọn ngẫu nhiên toàn quốc. B5: script đặt <code>updated_at = requested_at</code> trong khi <code>accepted_at</code> = giờ đặt + 2 phút và <code>completed_at</code> = giờ đặt + 25 phút, nên <code>updated_at</code> sớm hơn mốc mới nhất. Sửa: chọn tài xế và quán trong cùng thành phố với chuyến; đặt <code>updated_at</code> bằng mốc mới nhất của dòng (<code>greatest(requested_at, accepted_at, completed_at)</code>). Bài học cho bàn giao: bộ dữ liệu lớn dùng để đo hiệu năng ở Phần 3 cũng phải chạy qua <code>07</code>, nếu không phép đo dựa trên dữ liệu không thể xảy ra.</p>
</details>

## 8. Đọc tiếp

- [Tài liệu PostgreSQL 17 — Constraints](https://www.postgresql.org/docs/17/ddl-constraints.html): CHECK, NOT NULL, UNIQUE, PRIMARY KEY, FOREIGN KEY.
- [Tài liệu PostgreSQL 17 — PostgreSQL Error Codes](https://www.postgresql.org/docs/17/errcodes-appendix.html): bảng SQLSTATE dùng trong D3.
- Lab: `examples/toi-noi-postgresql/p01` — `README.md`, các tệp `00`…`07`, `OUTPUT.txt`, `test/p01.test.mjs`.

Bài tiếp theo — mở đầu Phần 2 · SQL cho nghiệp vụ: [PG-011: NULL, CASE và COALESCE trong dữ liệu chuyến](/articles/postgresql-null-case-coalesce)
