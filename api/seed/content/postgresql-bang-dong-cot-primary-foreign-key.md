---
slug: postgresql-bang-dong-cot-primary-foreign-key
title: "Bảng, dòng, cột và khoá trong nghiệp vụ chuyến xe"
summary: "Khách gọi tổng đài hỏi về «chuyến đồ ăn lúc 12 giờ trưa hôm qua» — fixture Tới Nơi có hai chuyến như thế. Học bảng, dòng, cột, primary key và foreign key để luôn tìm đúng một chuyến."
category: postgresql
level: beginner
tags: [postgresql, pg-003, primary-key, foreign-key, data-model, toi-noi]
language: vi
---

Tổng đài Tới Nơi nhận cuộc gọi: «Tôi đặt đồ ăn lúc 12 giờ trưa hôm qua, chuyến đó bị sao vậy?». Nhân viên mở dữ liệu ngày 02/11 và thấy **hai** chuyến đồ ăn cùng đặt đúng 12:00 — một ở Hà Nội đã huỷ, một ở Đà Nẵng đã hoàn thành. Giờ đặt không đủ để chỉ ra chuyến nào; tên khách cũng không đủ, vì người trùng tên là chuyện thường.

Câu hỏi «Làm sao chỉ ra đúng **một** chuyến, rồi từ đó tới đúng **một** khách, **một** tài xế?» chính là câu hỏi về khoá — chủ đề của bài này.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-003 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a>, <a href="/articles/postgresql-cai-dat-lab-psql-ket-noi">PG-002: Dựng môi trường lab</a>. Lab: <code>examples/toi-noi-postgresql/p01</code> (<code>01-schema.sql</code>, <code>02-fixture-valid.sql</code>, <code>03-fixture-invalid.sql</code>). Sau bài này, bạn vẽ được quan hệ khách–chuyến–tài xế và nhận diện một dòng bằng khoá thay vì bằng tên hay giờ.</p>
</div>

## 1. Một dòng là gì: bắt đầu từ «một chuyến»

Trong PostgreSQL, dữ liệu nằm trong **table** (bảng). Mỗi bảng có một danh sách **column** (cột) cố định và chứa nhiều **row** (dòng). Câu hỏi quan trọng nhất khi thiết kế bảng không phải danh sách cột mà là: **một dòng của bảng này là gì?** Người thiết kế dữ liệu gọi đó là *grain* — độ mịn của bảng.

| Bảng của lab P01 | Một dòng là | Số dòng trong fixture |
|---|---|---:|
| `cities` | một thành phố | 3 |
| `customers` | một khách | 6 |
| `drivers` | một tài xế | 6 |
| `merchants` | một quán | 3 |
| `trips` | một chuyến (đặt xe, đồ ăn hoặc bưu kiện) | 15 |
| `trip_events` | một lần chuyến đổi trạng thái | 0 (ghi từ PG-006) |

Vì một dòng `trips` là một chuyến, chuyến chỉ có một trạng thái hiện tại; chuỗi «requested → searching → accepted…» không nhét được vào một dòng nên mới có `trip_events`. Trộn hai grain trong một bảng là nguồn lỗi mà [PG-008](/articles/postgresql-mo-hinh-quan-he-chuan-hoa) sẽ mổ xẻ.

Thứ tự dòng trong bảng **không mang ý nghĩa** (muốn có thứ tự phải yêu cầu rõ — PG-007), nên «dòng thứ năm» không phải cách gọi một chuyến.

## 2. Cột: tên, kiểu và có được để trống

Mỗi cột có tên, **data type** (kiểu dữ liệu) và quy định có được để trống. Cuối `01-schema.sql`, lab hỏi catalog chuẩn SQL `information_schema.columns` để in cấu trúc `trips` (psql có lệnh `\d trips`, nhưng đó là lệnh của công cụ chứ không phải SQL). Kết quả chạy thật, chỉ giữ 8 trong 13 dòng:

```text
 column_name  |        data_type         | is_nullable |  column_default   | is_identity
--------------+--------------------------+-------------+-------------------+-------------
 id           | uuid                     | NO          | gen_random_uuid() | NO
 code         | bigint                   | NO          |                   | YES
 kind         | text                     | NO          |                   | NO
 customer_id  | uuid                     | NO          |                   | NO
 driver_id    | uuid                     | YES         |                   | NO
 status       | text                     | NO          | 'requested'::text | NO
 fare         | integer                  | NO          |                   | NO
 accepted_at  | timestamp with time zone | YES         |                   | NO
 …
(13 rows)
```

- **`is_nullable`**: cột có được chứa `NULL` — giá trị «không có / chưa biết». `driver_id` là `YES` vì chuyến vừa đặt chưa có tài xế; `customer_id` là `NO` vì không chuyến nào thiếu khách.
- **`column_default`**: giá trị tự điền khi câu lệnh thêm dòng không nhắc tới cột, ví dụ trạng thái mặc định `requested`.
- **`data_type`**: giới hạn giá trị hợp lệ — `fare` là số nguyên, không chứa được chữ «45.000đ». Vì sao chọn các kiểu này là chủ đề của [PG-004](/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid).

## 3. Nhận diện một dòng: khoá và primary key

Một **key** (khoá) là tập cột mà giá trị không bao giờ trùng giữa hai dòng — biết giá trị khoá là chỉ ra đúng một dòng. Một bảng có thể có vài khoá ứng viên (candidate key): với `customers`, cả `id` lẫn `phone` đều không trùng.

Ta chọn **một** khoá ứng viên làm **primary key** (khoá chính). Tài liệu PostgreSQL yêu cầu giá trị primary key *«both unique and not null»* — không trùng và không `NULL`. Một bảng có tối đa một primary key; các khoá còn lại khai báo bằng `UNIQUE`. PostgreSQL tự tạo một unique index B-tree cho primary key — index là chuyện của Phần 3.

Lab đặt tên ràng buộc theo tiền tố `pk_`, `uq_`, `fk_`, `ck_` để thông báo lỗi chỉ thẳng vào quy tắc. Database thật sự **chặn** dữ liệu vi phạm khoá — hai câu trong `03-fixture-invalid.sql` thêm thành phố trùng `id` và khách trùng số điện thoại:

```text
INSERT INTO cities (id, code, name) VALUES (1, 'XX', 'Trùng id 1');
ERROR:  duplicate key value violates unique constraint "pk_cities"
DETAIL:  Key (id)=(1) already exists.

ERROR:  duplicate key value violates unique constraint "uq_customers_phone"
DETAIL:  Key (phone)=(0900000001) already exists.
```

Cả hai cùng mã `23505` (unique_violation): primary key và unique dùng chung cơ chế «không trùng»; primary key còn cấm `NULL` và là khoá mà bảng khác trỏ tới. Cách viết các câu lệnh này nằm ở [PG-005](/articles/postgresql-create-table-constraints) và PG-006.

**Vì sao không lấy số điện thoại làm primary key?** `phone` là **natural key** (khoá tự nhiên — có nghĩa ngoài đời): khách đổi số, nhà mạng cấp lại số cũ cho người khác, và mọi chuyến trỏ tới số đó sẽ phải sửa theo. Tới Nơi dùng **surrogate key** (khoá thay thế — không mang nghĩa nghiệp vụ) `id uuid`, giữ `phone` là `UNIQUE`. Đây là đánh đổi, không phải luật: `cities` dùng `id smallint` do quản trị viên đặt, vì danh mục nhỏ và gần như không đổi.

## 4. Hai cách gọi một chuyến: `id` uuid và `code`

| Cột | Ràng buộc | Ví dụ | Ai dùng |
|---|---|---|---|
| `id` | `pk_trips` | `70000000-0000-4000-8000-000000100004` | ứng dụng, bảng khác qua foreign key |
| `code` | `uq_trips_code` | `100004` | khách, tổng đài, tài xế — đọc được qua điện thoại |

Tổng đài chỉ cần hỏi mã chuyến rồi «đi theo khoá». Ba truy vấn dưới đây chạy trên lab sau khi chạy các tệp `00`→`02`, mỗi truy vấn lọc bằng một khoá nên trả **đúng một dòng**:

```sql
SELECT code, customer_id, driver_id FROM trips WHERE code = 100004;
SELECT phone, full_name FROM customers WHERE id = 'c0000000-0000-4000-8000-000000000005';
SELECT code, full_name, city_id FROM drivers WHERE id = 'd0000000-0000-4000-8000-000000008005';
```

```text
  code  |             customer_id              |              driver_id
--------+--------------------------------------+--------------------------------------
 100004 | c0000000-0000-4000-8000-000000000005 | d0000000-0000-4000-8000-000000008005
(1 row)

   phone    |  full_name
------------+-------------
 0900000005 | Võ Minh Đức
(1 row)

 code |  full_name  | city_id
------+-------------+---------
 8005 | Lý Thị Oanh |       3
(1 row)
```

Chuyến 100004 là của khách Võ Minh Đức, tài xế Lý Thị Oanh (mã 8005) ở Đà Nẵng. Phần 2 gộp ba bước này thành một câu `JOIN`; bài này cố ý tách ra để thấy mỗi bước chỉ là «đi theo một khoá».

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">UUID trong fixture được viết cho dễ đọc</span>
<p>Fixture ghi sẵn UUID có đuôi trùng mã (<code>…100004</code> ↔ 100004) chỉ để kết quả ổn định. Trong hệ thống thật, <code>id</code> ngẫu nhiên, không liên quan tới <code>code</code>: đừng suy cái này từ cái kia. <code>code</code> cũng không liên tục — chuyến tạo sau trong lab nhận 100029 chứ không phải 100015 (PG-004, PG-006 giải thích), nên mã dùng để gọi tên, không dùng để đếm.</p>
</div>

## 5. Foreign key: nối chuyến với khách, tài xế, quán

Để database bảo đảm `trips.customer_id` **trỏ tới một khách có thật**, lab khai báo **foreign key** (khoá ngoại) `fk_trips_customer FOREIGN KEY (customer_id) REFERENCES customers(id)`; `trips` là bảng con, `customers` là bảng cha. Theo tài liệu, foreign key phải trỏ tới primary key hoặc cột có ràng buộc unique — nếu không, «đúng một dòng» mất nghĩa.

Foreign key chặn cả hai chiều. Hai ví dụ dưới đây là thêm chuyến cho khách không tồn tại, và xoá thành phố Đà Nẵng khi còn tài xế trỏ tới (cả hai từ `03-fixture-invalid.sql`):

```text
ERROR:  insert or update on table "trips" violates foreign key constraint "fk_trips_customer"
DETAIL:  Key (customer_id)=(c0000000-0000-4000-8000-000000000099) is not present in table "customers".

DELETE FROM cities WHERE id = 3;
ERROR:  update or delete on table "cities" violates foreign key constraint "fk_drivers_city" on table "drivers"
DETAIL:  Key (id)=(3) is still referenced from table "drivers".
```

Cả hai mang mã `23503` (foreign_key_violation). Lỗi thứ hai do hành vi mặc định `NO ACTION`: từ chối xoá dòng cha còn dòng con. `CASCADE` xoá luôn dòng con — với dữ liệu chuyến và tiền, thường là điều ta **không** muốn.

**NULL trong cột foreign key.** `driver_id` được phép `NULL`; theo tài liệu, dòng có `NULL` ở cột tham chiếu không bị foreign key kiểm tra. Fixture có ba chuyến như vậy (100006 huỷ khi chưa ai nhận, 100013 `searching`, 100014 `requested`). Còn gán một tài xế không tồn tại cho chuyến 100012 thì bị chặn ngay với `fk_trips_driver`. Vậy quy tắc «từ `accepted` trở đi phải có tài xế» không do foreign key quyết định; lab giao nó cho CHECK `ck_trips_assigned_has_driver` (PG-005).

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Foreign key chỉ kiểm «có tồn tại», không kiểm «có hợp lý»</span>
<p>Chạy thử trong giao dịch rồi huỷ: gán tài xế 8003 (Hà Nội, <code>city_id</code> 2) cho chuyến 100012 ở TP.HCM — câu lệnh <b>thành công</b>, trả về <code>100012 | 1 | d0000000-…-000000008003</code>. Tài xế có thật nên foreign key hài lòng; «tài xế khác thành phố» là quy tắc nhiều bảng mà foreign key đơn không diễn đạt được. Lab bắt nó bằng kiểm tra B1 của <code>07-checks.sql</code>; PG-008 bàn cách khoá bằng schema.</p>
</div>

## 6. Sơ đồ quan hệ khách–chuyến–tài xế

Gom các foreign key lại, ta có sơ đồ của P01. Mỗi đường nối đi từ bảng cha (đầu «1») tới bảng con (đầu «n»); nét đứt là foreign key được phép `NULL`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 520" role="img" aria-label="Sơ đồ quan hệ lab P01: cities một-nhiều với trips, drivers, merchants; customers một-nhiều với trips; drivers và merchants nối trips bằng khoá ngoại có thể NULL; trips một-nhiều với trip_events.">
<rect x="355" y="16" width="210" height="64" rx="8" class="f-surface s-line"/>
<text x="460" y="42" text-anchor="middle" class="t-mono t-b">cities</text>
<text x="460" y="64" text-anchor="middle" class="t-mono t-10 t-pk">PK id smallint · UQ code</text>
<rect x="20" y="180" width="220" height="96" rx="8" class="f-surface s-line"/>
<text x="36" y="206" class="t-mono t-b">customers</text>
<text x="36" y="232" class="t-mono t-10 t-pk">PK id uuid</text>
<text x="36" y="254" class="t-mono t-10">UQ phone</text>
<rect x="340" y="150" width="240" height="210" rx="8" class="f-pk-soft s-pk" stroke-width="1.8"/>
<text x="356" y="176" class="t-mono t-b t-pk">trips</text>
<text x="356" y="200" class="t-mono t-10 t-pk">PK id uuid</text>
<text x="356" y="220" class="t-mono t-10">UQ code bigint</text>
<text x="356" y="244" class="t-mono t-10">FK customer_id</text>
<text x="356" y="264" class="t-mono t-10">FK driver_id (NULL được)</text>
<text x="356" y="284" class="t-mono t-10">FK merchant_id (NULL được)</text>
<text x="356" y="304" class="t-mono t-10">FK city_id</text>
<text x="356" y="330" class="t-small">kind, status, fare,</text>
<text x="356" y="348" class="t-small">requested_at, accepted_at…</text>
<rect x="680" y="150" width="220" height="96" rx="8" class="f-surface s-line"/>
<text x="696" y="176" class="t-mono t-b">drivers</text>
<text x="696" y="200" class="t-mono t-10 t-pk">PK id uuid</text>
<text x="696" y="220" class="t-mono t-10">UQ code · FK city_id</text>
<rect x="680" y="284" width="220" height="76" rx="8" class="f-surface s-line"/>
<text x="696" y="310" class="t-mono t-b">merchants</text>
<text x="696" y="334" class="t-mono t-10 t-pk">PK id uuid</text>
<text x="696" y="352" class="t-mono t-10">FK city_id</text>
<rect x="340" y="420" width="240" height="80" rx="8" class="f-surface s-line"/>
<text x="356" y="446" class="t-mono t-b">trip_events</text>
<text x="356" y="470" class="t-mono t-10 t-pk">PK id bigint</text>
<text x="356" y="488" class="t-mono t-10">FK trip_id</text>
<line x1="460" y1="80" x2="460" y2="150" class="s-line" stroke-width="1.6"/>
<text x="468" y="98" class="t-small">1</text>
<text x="468" y="144" class="t-small">n</text>
<path d="M565 64 H790 V150" class="f-none s-line" stroke-width="1.6"/>
<text x="574" y="58" class="t-small">1</text>
<text x="798" y="144" class="t-small">n</text>
<path d="M565 40 H910 V322 H900" class="f-none s-line" stroke-width="1.6"/>
<text x="574" y="34" class="t-small">1</text>
<text x="886" y="316" class="t-small">n</text>
<line x1="240" y1="228" x2="340" y2="228" class="s-line" stroke-width="1.6"/>
<text x="248" y="220" class="t-small">1</text>
<text x="326" y="220" class="t-small">n</text>
<line x1="680" y1="198" x2="580" y2="198" class="s-warn dash" stroke-width="1.6"/>
<text x="642" y="190" class="t-small t-warn">0..1</text>
<text x="588" y="190" class="t-small">n</text>
<line x1="680" y1="322" x2="580" y2="322" class="s-warn dash" stroke-width="1.6"/>
<text x="642" y="314" class="t-small t-warn">0..1</text>
<text x="588" y="314" class="t-small">n</text>
<line x1="460" y1="360" x2="460" y2="420" class="s-line" stroke-width="1.6"/>
<text x="468" y="378" class="t-small">1</text>
<text x="468" y="414" class="t-small">n</text>
</svg>
</div>
<figcaption><b>Hình 1 · Quan hệ giữa sáu bảng của lab P01.</b> Foreign key luôn nằm ở phía «n». Nét đứt: chuyến có thể chưa có tài xế, và chỉ đơn đồ ăn mới có quán.</figcaption>
</figure>

Đọc thành câu: một khách có nhiều chuyến (khách `0900000001` có 3 chuyến trong fixture), một tài xế có nhiều chuyến theo thời gian, một chuyến có nhiều sự kiện. Hai điểm hay bị vẽ sai:

- Foreign key luôn ở phía «n»: không có cột `trip_ids` nào trong `customers`.
- «Tài xế – chuyến» là một–nhiều **theo thời gian**. Quy tắc «tại một thời điểm, tài xế chỉ chạy một chuyến» không nằm trong sơ đồ; lab kiểm tra bằng B3 của `07-checks.sql`, còn tranh chấp thật là chủ đề Phần 4.

## 7. Lỗi thường gặp và đánh đổi

- **Nhận diện bằng thuộc tính không phải khoá** (giờ đặt, họ tên, «dòng thứ mấy») — hai chuyến 12:00 của fixture là ví dụ thật.
- **Lấy natural key dễ đổi làm primary key** — số điện thoại, biển số, email nên là `UNIQUE`.
- **Tin foreign key bảo đảm dữ liệu hợp lý, hoặc dùng `ON DELETE CASCADE` cho dữ liệu có giá trị lịch sử** — xoá một khách mà mất mọi chuyến là sự cố.
- **Nghĩ primary key quyết định dòng nằm ở đâu trên đĩa** — với PostgreSQL thì không; xem [Clustered index thật và clustered index nhìn thấy](/articles/heap-ctid-va-primary-key) khi tới Phần 3.

Đánh đổi: `uuid` cho phép ứng dụng tự sinh id trước khi ghi và không lộ số lượng chuyến, nhưng khó đọc — nên cần thêm `code`. Mỗi foreign key là một lần kiểm tra khi ghi dòng con và khi xoá dòng cha; PostgreSQL cũng **không** tự tạo index cho cột tham chiếu (lab tạo `ix_trips_driver_id` bằng tay). Phần 3 sẽ đo chi phí này, và đây không phải lý do để bỏ foreign key.

## 8. Bài tập

**Bài 1.** Tới Nơi thêm bảng `ratings`: khách chấm 1–5 sao sau chuyến, mỗi chuyến **tối đa một** đánh giá. Grain là gì? Cột nào cần foreign key, làm sao chặn hai đánh giá cho cùng chuyến?

<details><summary><b>Lời giải</b></summary><p>Một dòng là một đánh giá của một chuyến. <code>trip_id</code> có foreign key tới <code>trips(id)</code> và <code>NOT NULL</code>. Đặt <code>UNIQUE (trip_id)</code> — hoặc dùng luôn <code>trip_id</code> làm primary key vì quan hệ chuyến–đánh giá là một–(0..1). Không thêm <code>customer_id</code>: suy ra được qua chuyến, thêm vào là lặp dữ liệu (PG-008). «Chỉ chuyến completed mới được đánh giá» là quy tắc nhiều bảng, foreign key không kiểm được.</p></details>

**Bài 2.** Không chạy lệnh, hãy đoán mã lỗi và tên ràng buộc trên fixture: (a) thêm tài xế mới mã 8002; (b) đặt `merchant_id` của chuyến 100002 thành UUID quán không tồn tại; (c) xoá khách id `c0000000-0000-4000-8000-000000000001`.

<details><summary><b>Lời giải</b></summary><p>(a) <code>23505</code> <code>uq_drivers_code</code>. (b) <code>23503</code> <code>fk_trips_merchant</code>. (c) <code>23503</code> <code>fk_trips_customer</code> — khách còn chuyến 100000, 100006, 100012, hành vi mặc định <code>NO ACTION</code> chặn xoá. Tự kiểm tra: chép câu lệnh vào một tệp, đặt dòng <code>-- expect-error: …</code> phía trên mỗi câu, chạy <code>node run.mjs</code> sau <code>00</code>→<code>02</code>.</p></details>

## 9. Đọc tiếp

- [PG-004: Chọn kiểu dữ liệu cho tiền, thời gian và định danh](/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid) — vì sao `id` là `uuid`, `fare` là `integer`, thời điểm là `timestamptz`.
- [PG-005: CREATE TABLE và ràng buộc dữ liệu](/articles/postgresql-create-table-constraints) — viết các khoá thành DDL.
- Tài liệu PostgreSQL 17: [Constraints](https://www.postgresql.org/docs/17/ddl-constraints.html), [information_schema.columns](https://www.postgresql.org/docs/17/infoschema-columns.html), [pg_constraint](https://www.postgresql.org/docs/17/catalog-pg-constraint.html).

Bài tiếp theo: [PG-004: Chọn kiểu dữ liệu cho tiền, thời gian và định danh](/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid)
