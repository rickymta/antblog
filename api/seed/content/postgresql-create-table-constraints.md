---
slug: postgresql-create-table-constraints
title: "CREATE TABLE và ràng buộc dữ liệu: để database tự từ chối chuyến sai"
summary: "Một bản app tài xế ghi chuyến completed mà quên giờ hoàn thành, báo cáo Tới Nơi lệch cả tuần. Viết DDL với primary key, foreign key, unique, check, not null có đặt tên rõ ràng, rồi đọc từng SQLSTATE khi dữ liệu sai bị chặn."
category: postgresql
level: intermediate
tags: [postgresql, pg-005, ddl, constraint, check, foreign-key, sqlstate, toi-noi]
language: vi
---

Thứ Hai, nhóm vận hành Tới Nơi thấy báo cáo «thời gian trung bình mỗi chuyến» của tuần trước tụt bất thường. Truy ngược lại mới biết: một bản phát hành của app tài xế đổi chuyến sang `completed` nhưng quên ghi `completed_at`. Hàng nghìn chuyến «đã xong» không có giờ xong, phép tính bỏ qua chúng nên con số trông đẹp một cách giả tạo. Cùng tuần, job nhập đơn từ một đối tác ghi số điện thoại dạng `+84…`, trong khi app khách ghi `0…` — một khách thành hai hồ sơ. (Sự cố là **giả định** để minh hoạ; Tới Nơi là hệ thống hư cấu.)

Mỗi ứng dụng chỉ kiểm tra phần của mình; thứ duy nhất mọi đường ghi đều phải đi qua là database. Bài này biến các quy tắc nghiệp vụ của chuyến thành **ràng buộc** trong `CREATE TABLE`, rồi cố tình ghi 33 câu dữ liệu sai để xem PostgreSQL từ chối từng câu ra sao.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-005 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-bang-dong-cot-primary-foreign-key">PG-003 · Bảng, dòng, cột và khoá</a>, <a href="/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid">PG-004 · Kiểu dữ liệu cho tiền, thời gian, định danh</a>. Lab: <code>examples/toi-noi-postgresql/p01</code> (tệp <code>01-schema.sql</code>, <code>03-fixture-invalid.sql</code>). Sau bài này, bạn viết được DDL với primary key, foreign key, unique, check và not null, đặt tên ràng buộc có quy ước và đọc được SQLSTATE khi dữ liệu sai bị chặn.</p>
</div>

## 1. CREATE TABLE gồm những gì

`CREATE TABLE` thuộc nhóm **DDL** (Data Definition Language — câu lệnh định nghĩa cấu trúc), khác với INSERT/UPDATE/DELETE là câu lệnh thao tác dữ liệu. Một định nghĩa bảng có ba lớp:

- **Cột và kiểu**: `phone text`, `fare integer`… (đã chọn ở PG-004).
- **Giá trị mặc định** (`DEFAULT`): dùng khi câu INSERT không nhắc tới cột.
- **Ràng buộc** (constraint): điều kiện mà mọi dòng trong bảng phải thoả. PostgreSQL kiểm tra ràng buộc mỗi khi thêm hoặc sửa dòng; với khoá ngoại thì kiểm tra cả khi xoá dòng đang được tham chiếu. Câu lệnh vi phạm bị huỷ, bảng giữ nguyên.

Bảng `customers` của lab có đủ cả ba lớp:

```sql
CREATE TABLE customers (
  id         uuid        NOT NULL DEFAULT gen_random_uuid(),
  phone      text        NOT NULL,
  full_name  text        NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_customers PRIMARY KEY (id),
  CONSTRAINT uq_customers_phone UNIQUE (phone),
  CONSTRAINT ck_customers_phone_format CHECK (phone ~ '^0[0-9]{9}$'),
  CONSTRAINT ck_customers_full_name_not_blank CHECK (btrim(full_name) <> '')
);
```

`NOT NULL` viết ngay trên dòng cột — **ràng buộc cột** (column constraint). Bốn dòng `CONSTRAINT …` đứng riêng là **ràng buộc bảng** (table constraint), viết được cho nhiều cột. NOT NULL luôn là ràng buộc cột; lab để mọi ràng buộc còn lại ở dạng bảng cho dễ đặt tên đồng loạt.

`~` so khớp biểu thức chính quy (`'^0[0-9]{9}$'`: số 0 rồi đúng 9 chữ số); `btrim()` cắt khoảng trắng hai đầu.

## 2. Năm loại ràng buộc trong schema của Tới Nơi

| Loại | Chặn điều gì | Ví dụ trong lab | SQLSTATE khi vi phạm |
|---|---|---|---|
| `NOT NULL` | Thiếu giá trị bắt buộc | `trips.city_id`, `customers.full_name` | `23502` not_null_violation |
| `UNIQUE` | Hai dòng trùng khoá nghiệp vụ | `uq_customers_phone`, `uq_drivers_code`, `uq_trips_code` | `23505` unique_violation |
| `PRIMARY KEY` | Thiếu hoặc trùng định danh dòng | `pk_trips (id)` | `23505` (trùng) / `23502` (NULL) |
| `FOREIGN KEY` | Tham chiếu tới dòng không tồn tại, hoặc xoá dòng còn bị tham chiếu | `fk_trips_customer`, `fk_trip_events_trip` | `23503` foreign_key_violation |
| `CHECK` | Giá trị trái quy tắc nghiệp vụ trên một dòng | `ck_trips_fare_non_negative`, `ck_trips_status` | `23514` check_violation |

**SQLSTATE** là mã lỗi năm ký tự theo chuẩn SQL. Hai ký tự đầu là nhóm: `23` là nhóm «Integrity Constraint Violation» (vi phạm toàn vẹn), `22` là nhóm lỗi dữ liệu như sai định dạng.

Vài điểm cần nhớ:

- **Primary key = unique + not null**, mỗi bảng tối đa một. PostgreSQL tự tạo một unique index để thực thi ràng buộc này (UNIQUE cũng vậy); index là chủ đề của Phần 3.
- **UNIQUE mặc định coi hai NULL là khác nhau**: cột `email` cho phép trống thì nhiều khách cùng không có email vẫn hợp lệ. Từ PostgreSQL 15 có `UNIQUE NULLS NOT DISTINCT` nếu muốn chỉ cho phép một NULL.
- **Khoá ngoại mặc định là `NO ACTION`** khi xoá dòng cha: còn dòng con thì báo lỗi. Có thể chọn `RESTRICT`, `CASCADE` (xoá theo), `SET NULL`…; Tới Nơi giữ mặc định.

Bảng `trips` gom đủ năm loại. Trích phần ràng buộc (danh sách cột đầy đủ ở `01-schema.sql`):

```sql
CREATE TABLE trips (
  id           uuid        NOT NULL DEFAULT gen_random_uuid(),
  code         bigint      GENERATED ALWAYS AS IDENTITY (START WITH 100000),
  kind         text        NOT NULL,
  -- … city_id, customer_id, driver_id, merchant_id, status, fare, các mốc thời gian
  CONSTRAINT pk_trips PRIMARY KEY (id),
  CONSTRAINT uq_trips_code UNIQUE (code),
  CONSTRAINT fk_trips_customer FOREIGN KEY (customer_id) REFERENCES customers (id),
  CONSTRAINT fk_trips_driver   FOREIGN KEY (driver_id)   REFERENCES drivers (id),
  -- … fk_trips_city, fk_trips_merchant cùng dạng
  CONSTRAINT ck_trips_kind CHECK (kind IN ('ride', 'food', 'parcel')),
  CONSTRAINT ck_trips_fare_non_negative CHECK (fare >= 0),
  CONSTRAINT ck_trips_completed_at_by_status CHECK ((status = 'completed') = (completed_at IS NOT NULL)),
  CONSTRAINT ck_trips_accepted_after_requested CHECK (accepted_at >= requested_at)
  -- … tổng cộng 10 CHECK, các CHECK còn lại ở mục 4
);
```

`driver_id` và `merchant_id` không có NOT NULL: chuyến đang tìm tài xế chưa có tài xế, chuyến đặt xe không có quán. Khoá ngoại bỏ qua giá trị NULL — chỉ khi có giá trị thì giá trị đó mới phải tồn tại ở bảng cha.

## 3. Đặt tên ràng buộc: pk_, fk_, uq_, ck_

Nếu không ghi `CONSTRAINT tên`, PostgreSQL sẽ tự chọn tên. Hãy tạo thử một bảng nháp viết như hồ sơ Tới Nơi (không đặt tên) rồi xem catalog `pg_constraint` — bảng hệ thống liệt kê ràng buộc:

```sql
CREATE TABLE nhap_khong_ten (
  id    integer PRIMARY KEY,
  code  bigint  UNIQUE,
  kind  text    NOT NULL CHECK (kind IN ('ride', 'food', 'parcel')),
  fare  integer NOT NULL CHECK (fare >= 0)
);

SELECT conname, contype
FROM pg_constraint
WHERE conrelid = 'nhap_khong_ten'::regclass
ORDER BY conname;
```

```text
          conname          | contype
---------------------------+---------
 nhap_khong_ten_code_key   | u
 nhap_khong_ten_fare_check | c
 nhap_khong_ten_kind_check | c
 nhap_khong_ten_pkey       | p
(4 rows)
```

Tên tự sinh đọc được, nhưng tài liệu không cam kết cách sinh, và không cho biết quy tắc nào bị vi phạm. Lab đặt tên mọi ràng buộc theo quy ước `pk_<bảng>`, `fk_<bảng>_<cột>`, `uq_<bảng>_<cột>`, `ck_<bảng>_<quy tắc>`. Lợi ích cụ thể:

1. **Thông báo lỗi chỉ thẳng vào quy tắc**: `ck_trips_completed_at_by_status` nói ngay «trạng thái và giờ hoàn thành lệch nhau».
2. **Ứng dụng ánh xạ lỗi theo tên**: `23505` + `uq_customers_phone` ⇒ «Số điện thoại đã đăng ký».
3. **Migration gọi đúng tên** khi gỡ hay thay ràng buộc (PG-009).
4. **Kiểm thử ghi tên mong đợi**: `-- expect-error: 23505 uq_customers_phone`; ràng buộc khác bắt nhầm là test đỏ.

Chạy cùng truy vấn với `conrelid = 'trips'::regclass` trên schema của lab cho ra 16 dòng: 10 `ck_trips_…`, 4 `fk_trips_…`, `pk_trips` và `uq_trips_code`.

Không thấy NOT NULL ở đây: PostgreSQL 17 không lưu NOT NULL vào `pg_constraint` và không cho đặt tên nó. Lỗi NOT NULL vì thế chỉ nêu **tên cột**; muốn biết cột nào bắt buộc, đọc `information_schema.columns.is_nullable` (lab làm ở cuối `01-schema.sql`). PostgreSQL 18 thay đổi điểm này; lab chạy bản 17 nên chưa kiểm.

## 4. Viết CHECK cho quy tắc nghiệp vụ

### CHECK đạt khi biểu thức là true hoặc NULL

Tài liệu nói rõ một ràng buộc kiểm tra được thoả mãn «if the check expression evaluates to true or the null value». So sánh với NULL ra NULL, không ra false:

```sql
SELECT NULL::timestamptz >= TIMESTAMPTZ '2026-11-03 09:00:00+07' AS ket_qua_so_sanh,
       (NULL::timestamptz >= TIMESTAMPTZ '2026-11-03 09:00:00+07') IS NULL AS la_null;
```

```text
 ket_qua_so_sanh | la_null
-----------------+---------
                 | t
(1 row)
```

Nhờ vậy `ck_trips_accepted_after_requested CHECK (accepted_at >= requested_at)` tự bỏ qua chuyến chưa ai nhận. Mặt trái: CHECK kiểu so sánh **không bao giờ** buộc cột phải có giá trị. Quy tắc «phải có» phải viết tường minh bằng `IS NULL` / `IS NOT NULL`, hoặc dùng NOT NULL.

### Hai mẫu viết quy tắc

| Quy tắc nghiệp vụ | Dạng logic | CHECK trong lab |
|---|---|---|
| Đơn đồ ăn có quán, dịch vụ khác không có quán | A ⇔ B → `(A) = (B)` | `(kind = 'food') = (merchant_id IS NOT NULL)` |
| Có tài xế khi và chỉ khi có giờ nhận | A ⇔ B | `(driver_id IS NULL) = (accepted_at IS NULL)` |
| Chưa ai nhận thì chưa có tài xế | A ⇒ B → `NOT A OR B` | `status NOT IN ('requested','searching') OR driver_id IS NULL` |
| Từ accepted tới completed phải có tài xế | A ⇒ B | `status NOT IN ('accepted', …, 'completed') OR driver_id IS NOT NULL` |

Các vế đều là `IS NULL`/`IS NOT NULL` hoặc so sánh trên cột NOT NULL, nên không ra NULL — không có kẽ hở. `cancelled` không thuộc danh sách nào ở hàng 3–4, nên huỷ trước hay sau khi có tài xế đều hợp lệ (chuyến 100006 và 100003 của fixture).

### Mỗi CHECK một quy tắc

Một dòng vi phạm cùng lúc hai CHECK — loại chuyến `taxi` và cước âm:

```sql
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000099', 'taxi', 1, 'c0000000-0000-4000-8000-000000000001', NULL, NULL,
        'requested', -1000, '2026-11-03 09:00:00+07', NULL, NULL, '2026-11-03 09:00:00+07');
```

```text
ERROR:  new row for relation "trips" violates check constraint "ck_trips_fare_non_negative"
DETAIL:  Failing row contains (f0000000-0000-4000-8000-000000000099, 100015, taxi, 1, …, requested, -1000, …).
```

Câu lệnh bị huỷ và thông báo chỉ nêu **một** ràng buộc. Trên lab, tên được báo là tên đứng trước theo thứ tự chữ cái (`fare` trước `kind`), nhưng tài liệu **không hứa** thứ tự kiểm — đừng viết code dựa vào nó. Hệ quả thiết kế: nếu gộp nhiều quy tắc vào một CHECK lớn, thông báo lỗi không cho biết quy tắc nào hỏng. Lab vì thế tách mỗi CHECK một quy tắc, và mỗi câu sai ở `03` chỉ vi phạm đúng một quy tắc để lỗi báo ra không bị lẫn lộn.

### CHECK chỉ nhìn thấy dòng đang ghi

«Tài xế của chuyến phải cùng thành phố với chuyến» cần đọc bảng `drivers` — CHECK không làm được. Tài liệu PostgreSQL nói rõ không hỗ trợ CHECK tham chiếu dữ liệu khác ngoài dòng đang được kiểm; cách lách bằng một function có thể chạy được lúc thử nhưng không được bảo đảm, và có thể làm hỏng lúc dump/restore. Các quy tắc liên bảng như vậy được kiểm tra bằng truy vấn ở `07-checks.sql` (PG-010) và bàn ở [PG-008](/articles/postgresql-mo-hinh-quan-he-chuan-hoa).

## 5. Chạy 03-fixture-invalid.sql: mỗi câu sai, một SQLSTATE

```bash
cd examples/toi-noi-postgresql
node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql p01/03-fixture-invalid.sql
```

Tệp có 33 câu sai, mỗi câu kèm dòng chú thích `-- expect-error: <SQLSTATE> <tên>` phía trên. Runner của lab coi lỗi khớp là đạt; câu sai mà chạy được là FAIL — tức schema thiếu một quy tắc. Đây là kiểm thử cho chính schema. (Với psql thật, dòng đó chỉ là chú thích; đừng bật `ON_ERROR_STOP` cho tệp này.)

| Nhóm | Số câu | SQLSTATE |
|---|---:|---|
| NOT NULL | 3 | 23502 |
| UNIQUE / PRIMARY KEY | 4 | 23505 |
| FOREIGN KEY | 3 | 23503 |
| CHECK | 17 | 23514 |
| IDENTITY ghi tay | 1 | 428C9 |
| Sai kiểu / định dạng | 5 | 22P02, 22003, 22008, 22007 |

Đầu tệp có một câu đối chứng hợp lệ (chạy trong `BEGIN … ROLLBACK`, giao dịch học ở Phần 4); mỗi câu sai về chuyến chỉ đổi đúng một giá trị so với nó, nên lỗi chắc chắn do giá trị bị đổi. Trích kết quả theo nhóm (bỏ dòng chú thích của runner):

**NOT NULL** — ghi NULL tường minh bị chặn kể cả khi cột có DEFAULT; DEFAULT chỉ dùng khi câu lệnh không nhắc tới cột:

```text
ERROR:  null value in column "full_name" of relation "customers" violates not-null constraint
DETAIL:  Failing row contains (f0000000-0000-4000-8000-000000000001, 0900000011, null, 2026-11-03 09:00:00+07).

ERROR:  null value in column "status" of relation "trips" violates not-null constraint
```

**UNIQUE / PRIMARY KEY** — `DETAIL` cho biết giá trị nào đã tồn tại (trùng `pk_cities` cũng ra đúng dạng này):

```text
ERROR:  duplicate key value violates unique constraint "uq_customers_phone"
DETAIL:  Key (phone)=(0900000001) already exists.
```

**FOREIGN KEY** — hai chiều: thêm dòng con trỏ tới dòng cha không tồn tại, và xoá dòng cha còn dòng con:

```text
ERROR:  insert or update on table "trips" violates foreign key constraint "fk_trips_customer"
DETAIL:  Key (customer_id)=(c0000000-0000-4000-8000-000000000099) is not present in table "customers".

DELETE FROM cities WHERE id = 3;
ERROR:  update or delete on table "cities" violates foreign key constraint "fk_drivers_city" on table "drivers"
DETAIL:  Key (id)=(3) is still referenced from table "drivers".
```

**CHECK** — đúng sự cố mở bài. Số `+84…` bị từ chối để UNIQUE trên một dạng chuẩn còn ý nghĩa; chuyến 100012 bị đổi sang `completed` mà quên giờ hoàn thành:

```text
ERROR:  new row for relation "customers" violates check constraint "ck_customers_phone_format"
DETAIL:  Failing row contains (f0000000-0000-4000-8000-000000000008, +84900000001, Nguyễn Văn An, 2026-11-03 09:00:00+07).

UPDATE trips SET status = 'completed' WHERE code = 100012;
ERROR:  new row for relation "trips" violates check constraint "ck_trips_completed_at_by_status"
DETAIL:  Failing row contains (70000000-0000-4000-8000-000000100012, 100012, ride, 1, …, completed, 39000, 2026-11-03 08:20:00+07, 2026-11-03 08:21:00+07, null, …).
```

**IDENTITY và sai kiểu** — mã chuyến do database cấp; còn lỗi sai kiểu thì phát sinh ngay khi đọc giá trị, trước mọi ràng buộc:

```text
ERROR:  cannot insert a non-DEFAULT value into column "code"
DETAIL:  Column "code" is an identity column defined as GENERATED ALWAYS.
HINT:  Use OVERRIDING SYSTEM VALUE to override.

ERROR:  invalid input syntax for type integer: "45.000đ"
ERROR:  integer out of range
ERROR:  date/time field value out of range: "2026-02-30 09:00:00+07"
```

Cuối tệp, số dòng vẫn giữ nguyên như sau bước `02`: không câu sai nào để lại dấu vết.

```text
 cities | customers | drivers | trips
--------+-----------+---------+-------
      3 |         6 |       6 |    15
(1 row)
```

Hai điều đáng để ý. Thứ nhất, trong `OUTPUT.txt` các dòng chuyến bị từ chối mang mã 100016, 100017, 100019… (phần `DETAIL` in cả dòng): số đã được IDENTITY cấp trước khi ràng buộc từ chối dòng, và không được trả lại — đó là lý do chuyến kế tiếp ở PG-006 nhận mã 100029 chứ không phải 100015. Thứ hai, có một lỗi **không** báo gì: `'03/11/2026'::date` cho ra `2026-03-11` (DateStyle `ISO, MDY`), không ràng buộc nào chặn được ngày đúng định dạng nhưng sai ý. Luôn ghi ngày giờ theo ISO 8601.

## 6. Vì sao ràng buộc phải ở database, không chỉ ở ứng dụng

Việc kiểm tra trong ứng dụng vẫn cần để báo lỗi sớm, dễ hiểu ngay trên form — nhưng không đủ:

| Câu hỏi | Chỉ kiểm ở ứng dụng | Thêm ràng buộc ở database |
|---|---|---|
| Ai ghi vào `trips`? | App khách, app tài xế, công cụ CSKH, job nhập đối tác, script migration, kỹ sư trực sửa tay — mỗi đường tự kiểm | Mọi đường đều qua cùng một chốt |
| Hai yêu cầu đăng ký cùng số điện thoại cùng lúc | Cả hai «kiểm tra chưa tồn tại» rồi cùng ghi ⇒ trùng | `uq_customers_phone` cho đúng một câu thành công; câu kia nhận `23505` (cơ chế đồng thời: Phần 4) |
| Bản phát hành có lỗi | Dữ liệu sai lọt vào bảng mà không ai hay, chỉ phát hiện khi báo cáo lệch | Câu ghi báo lỗi ngay, lỗi lộ ra trong log và test |
| Ứng dụng xử lý lỗi thế nào | Tự định nghĩa mã lỗi riêng | Dựa vào SQLSTATE + tên ràng buộc |

Về điểm cuối, tài liệu PostgreSQL khuyên ứng dụng kiểm **mã lỗi** thay vì đọc nội dung thông báo: mã ít đổi giữa các phiên bản và không phụ thuộc ngôn ngữ thông báo. `23514` + `ck_trips_completed_at_by_status` nên được ứng dụng coi là lỗi lập trình cần cảnh báo, không phải lỗi người dùng.

## 7. Lỗi thường gặp và đánh đổi

- **Tưởng CHECK chặn NULL** (`CHECK (fare >= 0)` để lọt NULL) và **gộp nhiều luật vào một CHECK** (lỗi không chỉ ra luật nào hỏng) — hai lỗi đã gặp ở mục 4.
- **CHECK đọc bảng khác qua function**: không được hỗ trợ; dùng FK, truy vấn kiểm tra hoặc sửa mô hình (PG-008). Dữ liệu sinh theo script của hồ sơ dự án (bản thu nhỏ 2.000 chuyến) vượt qua mọi CHECK, nhưng có 1.638 chuyến mà tài xế ở thành phố khác với chuyến — chỉ truy vấn ở `07` phát hiện.
- **Danh sách giá trị sẽ đổi**: thêm trạng thái mới phải sửa ràng buộc, một thay đổi schema có quy trình riêng ([PG-009](/articles/postgresql-schema-migration-tuong-thich)). Phương án khác là bảng danh mục + khoá ngoại: sửa dữ liệu thay vì DDL, đổi lại thêm một bảng và một phép nối. Tám trạng thái gần như cố định hợp với CHECK; danh mục hay đổi hợp với bảng.
- **Bật `CASCADE` cho tiện**: xoá một thành phố thử nghiệm có thể kéo theo chuyến và lịch sử. `NO ACTION` mặc định buộc xoá có chủ đích.
- **Chi phí ghi**: mỗi lần ghi đều phải kiểm tra ràng buộc — CHECK chỉ tính trên dòng, UNIQUE/PK dựa vào index, FK phải tra bảng cha. Với 560 UPDATE/giây vào `trips` lúc cao điểm, đây là cái giá có thật; Phần 3 sẽ đo phần index thay vì đoán.

## 8. Bài tập

**Bài 1.** Nghiệp vụ yêu cầu thêm: «chuyến `cancelled` không được có `completed_at`». Có cần thêm ràng buộc mới không? Thử bằng `UPDATE trips SET completed_at = '2026-11-02 18:05:00+07' WHERE code = 100006;` (chuyến 100006 đang `cancelled`).

<details><summary><b>Lời giải</b></summary><p>Không cần. <code>ck_trips_completed_at_by_status</code> đã viết <code>(status = 'completed') = (completed_at IS NOT NULL)</code>: mọi trạng thái khác <code>completed</code> đều buộc <code>completed_at</code> là NULL. Câu thử bị chặn (đã chạy trên lab):</p>
<pre><code class="language-text">ERROR:  new row for relation "trips" violates check constraint "ck_trips_completed_at_by_status"
DETAIL:  Failing row contains (70000000-0000-4000-8000-000000100006, 100006, ride, 1, …, cancelled, 52000, 2026-11-02 18:00:00+07, null, 2026-11-02 18:05:00+07, …).</code></pre>
<p>Thêm một CHECK trùng ý chỉ làm thông báo lỗi khó đoán hơn (hai tên cùng đúng).</p></details>

**Bài 2.** Thêm cột `email` cho `customers`: không bắt buộc, không trùng, luôn lưu chữ thường. Viết định nghĩa cột và ràng buộc có tên. Hai khách cùng chưa có email có vi phạm không?

<details><summary><b>Lời giải</b></summary><pre><code class="language-sql">email text,
CONSTRAINT uq_customers_email UNIQUE (email),
CONSTRAINT ck_customers_email_lower CHECK (email = lower(email))</code></pre>
<p>Không NOT NULL vì email không bắt buộc. Nhiều dòng <code>email</code> NULL vẫn hợp lệ: UNIQUE mặc định coi các NULL là khác nhau, còn CHECK ra NULL nên đạt. Đã thử trên lab (thêm cột bằng <code>ALTER TABLE</code> trong giao dịch rồi huỷ): đặt NULL cho hai khách → <code>UPDATE 2</code>; ghi <code>'Binh@toinoi.test'</code> → <code>23514 ck_customers_email_lower</code>. CHECK này buộc ứng dụng chuẩn hoá trước khi ghi.</p></details>

**Bài 3.** Không chạy thử, hãy đoán SQLSTATE của từng câu: (a) INSERT tài xế mã 8002; (b) INSERT chuyến `food` không có quán; (c) INSERT chuyến với `fare = '54k'`; (d) DELETE khách `c0000000-…-000000000001`; (e) INSERT chuyến ghi `code = 100100`.

<details><summary><b>Lời giải</b></summary><p>(a) <code>23505 uq_drivers_code</code>. (b) <code>23514 ck_trips_merchant_only_food</code>. (c) <code>22P02</code> — lỗi đọc giá trị, xảy ra trước mọi ràng buộc. (d) <code>23503</code> — khách còn chuyến tham chiếu qua <code>fk_trips_customer</code>. (e) <code>428C9</code> — cột IDENTITY <code>GENERATED ALWAYS</code> không nhận giá trị ghi tay. Đối chiếu với các câu tương ứng trong <code>03-fixture-invalid.sql</code> và <code>OUTPUT.txt</code>.</p></details>

## 9. Đọc tiếp

- Trước đó: [PG-004 · Chọn kiểu dữ liệu cho tiền, thời gian và định danh](/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid).
- Đổi ràng buộc trên bảng đang có dữ liệu (`NOT VALID` → `VALIDATE`): [PG-009](/articles/postgresql-schema-migration-tuong-thich).
- Gom DDL, fixture sai và bằng chứng kiểm tra thành bản bàn giao: [PG-010](/articles/postgresql-thuc-hanh-schema-chuyen-xe).
- Tài liệu PostgreSQL 17: [Constraints](https://www.postgresql.org/docs/17/ddl-constraints.html), [CREATE TABLE](https://www.postgresql.org/docs/17/sql-createtable.html), [Mã lỗi SQLSTATE](https://www.postgresql.org/docs/17/errcodes-appendix.html).

Bài tiếp theo: [PG-006 · Thêm, sửa, xoá và đọc kết quả với RETURNING](/articles/postgresql-insert-update-delete-returning)
