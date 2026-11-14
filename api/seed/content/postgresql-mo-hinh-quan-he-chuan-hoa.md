---
slug: postgresql-mo-hinh-quan-he-chuan-hoa
title: "Chuẩn hoá và ranh giới mô hình dữ liệu"
summary: "Khách Tới Nơi đổi số điện thoại, tổng đài vẫn gọi số cũ vì bảng phẳng chỉ sửa một dòng. Bài này dùng chuẩn hoá 1NF–3NF để tìm dữ kiện lặp, xác định khi nào snapshot là đúng và đặt ranh giới chuyến – sự kiện."
category: postgresql
level: advanced
tags: [postgresql, pg-008, normalization, data-modeling, foreign-key, denormalization, toi-noi]
language: vi
---

Đội CSKH của Tới Nơi muốn «một bảng duy nhất»: mỗi dòng một chuyến, có sẵn tên và số điện thoại khách, tên tài xế, thành phố, quán, cả lịch sử trạng thái — xuất Excel là xong, cần thì sửa thẳng.

Ba tuần sau, khách Nguyễn Văn An đổi số điện thoại; nhân viên sửa ở chuyến gần nhất, và tối đó tổng đài gọi xác nhận một chuyến cũ — vào số cũ. Không câu lệnh nào lỗi; database chỉ đang giữ **hai sự thật khác nhau về cùng một người**. Bài này dùng chuẩn hoá để gọi tên lỗi đó, rồi đi tới câu hỏi khó hơn: khi nào giữ bản sao là thiết kế đúng, và đâu là ranh giới giữa chuyến với sự kiện của chuyến.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-008 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/postgresql-bang-dong-cot-primary-foreign-key">PG-003 · bảng, dòng, cột và khoá</a>, <a href="/articles/postgresql-create-table-constraints">PG-005 · CREATE TABLE và ràng buộc</a>, <a href="/articles/postgresql-select-filter-order-limit">PG-007 · SELECT, lọc, sắp xếp</a>. Lab: <code>examples/toi-noi-postgresql/p01</code> (schema <code>01</code>, kiểm tra <code>07</code>). Sau bài này, bạn giải thích được bất thường cập nhật và đánh đổi khi tách bảng.</p>
</div>

## 1. Bảng phẳng: một dòng biết mọi thứ

Dựng bảng phẳng mà CSKH muốn trong schema nháp `phang`, 5 chuyến chép từ fixture. Lab không có tệp này: chép lần lượt các khối SQL của mục 1, 3, 4 vào một tệp rồi chạy `node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql <tệp>.sql` trong `examples/toi-noi-postgresql`. Kết quả trong bài là lần chạy thật trên PGlite (PostgreSQL 17.5), đã bỏ dòng `-- [lab]`.

```sql
CREATE SCHEMA phang;
SET search_path TO phang;

CREATE TABLE trip_flat (
  trip_code      bigint PRIMARY KEY,
  kind           text NOT NULL,
  status         text NOT NULL,
  fare           integer NOT NULL,
  requested_at   timestamptz NOT NULL,
  customer_phone text NOT NULL,
  customer_name  text NOT NULL,
  driver_code    integer,
  driver_name    text,
  city_code      text NOT NULL,
  city_name      text NOT NULL,
  merchant_name  text,
  status_history text
);

INSERT INTO trip_flat VALUES
  (100000, 'ride', 'completed', 45000, '2026-11-02 07:58+07', '0900000001', 'Nguyễn Văn An', 8001, 'Bùi Văn Khoa', 'SG', 'TP. Hồ Chí Minh', NULL,
   'requested,searching,accepted,arriving,picked_up,in_transit,completed'),
  (100002, 'food', 'completed', 62000, '2026-11-02 11:05+07', '0900000002', 'Trần Thị Bình', 8002, 'Hồ Thị Lan', 'SG', 'TP. Hồ Chí Minh', 'Cơm Tấm Phố Lab',
   'requested,searching,accepted,arriving,picked_up,in_transit,completed'),
  (100003, 'food', 'cancelled', 71000, '2026-11-02 12:00+07', '0900000003', 'Lê Hoàng Cường', 8003, 'Ngô Quang Minh', 'HN', 'Hà Nội', 'Bún Chả Ngõ Mẫu',
   'requested,searching,accepted,cancelled'),
  (100006, 'ride', 'cancelled', 52000, '2026-11-02 18:00+07', '0900000001', 'Nguyễn Văn An', NULL, NULL, 'SG', 'TP. Hồ Chí Minh', NULL,
   'requested,searching,cancelled'),
  (100012, 'ride', 'accepted', 39000, '2026-11-03 08:20+07', '0900000001', 'Nguyễn Văn An', 8001, 'Bùi Văn Khoa', 'SG', 'TP. Hồ Chí Minh', NULL,
   'requested,searching,accepted');

SELECT trip_code, customer_name, customer_phone, driver_name, city_name, merchant_name
FROM trip_flat
ORDER BY trip_code;
```

```text
 trip_code | customer_name  | customer_phone |  driver_name   |    city_name    |  merchant_name
-----------+----------------+----------------+----------------+-----------------+-----------------
    100000 | Nguyễn Văn An  | 0900000001     | Bùi Văn Khoa   | TP. Hồ Chí Minh |
    100002 | Trần Thị Bình  | 0900000002     | Hồ Thị Lan     | TP. Hồ Chí Minh | Cơm Tấm Phố Lab
    100003 | Lê Hoàng Cường | 0900000003     | Ngô Quang Minh | Hà Nội          | Bún Chả Ngõ Mẫu
    100006 | Nguyễn Văn An  | 0900000001     |                | TP. Hồ Chí Minh |
    100012 | Nguyễn Văn An  | 0900000001     | Bùi Văn Khoa   | TP. Hồ Chí Minh |
(5 rows)
```

Chỉ 5 dòng mà «TP. Hồ Chí Minh» lặp 4 lần, khách An 3 lần; với 2 triệu chuyến/ngày của hồ sơ Tới Nơi, mỗi khách quen lặp hàng trăm lần.

## 2. Phụ thuộc hàm: ngôn ngữ để nói «dữ kiện này ở đâu»

**Phụ thuộc hàm** (functional dependency), viết `X → Y`: hai dòng cùng giá trị `X` thì **buộc** cùng giá trị `Y`. Đây là phát biểu về **nghiệp vụ**; dữ liệu đang có chỉ phản bác được một phụ thuộc, không chứng minh được nó.

Theo nghiệp vụ Tới Nơi, `trip_flat` có: `trip_code →` mọi cột (khoá của bảng); `customer_phone → customer_name` (khách đăng nhập bằng số, một số là một khách); `driver_code → driver_name`; `city_code → city_name`. Còn tên quán **không** xác định được quán — hai quán khác nhau có thể trùng tên. Bảng phẳng không chỉ lặp dữ liệu mà còn **thiếu định danh**: muốn biết hai đơn có cùng một quán không, ta chỉ có cái tên.

**Chuẩn hoá** (normalization) là tách bảng sao cho mỗi dữ kiện nằm đúng một chỗ — cột phụ thuộc vào cái gì thì nằm trong bảng có khoá là cái đó — để đổi một sự thật chỉ phải sửa **một** dòng.

## 3. Ba bất thường của bảng phẳng

Thuật ngữ kinh điển là **bất thường cập nhật** (update anomaly), gồm ba dạng: sửa, thêm, xoá.

### 3.1. Sửa: hai sự thật về một người

```sql
UPDATE trip_flat SET customer_phone = '0900000011' WHERE trip_code = 100012;

SELECT trip_code, customer_name, customer_phone
FROM trip_flat
WHERE customer_name = 'Nguyễn Văn An'
ORDER BY trip_code;
```

```text
UPDATE 1
 trip_code | customer_name | customer_phone
-----------+---------------+----------------
    100000 | Nguyễn Văn An | 0900000001
    100006 | Nguyễn Văn An | 0900000001
    100012 | Nguyễn Văn An | 0900000011
(3 rows)
```

Khách An giờ có hai số; không ràng buộc nào chặn được vì bảng không có khái niệm «khách». Sửa cho đủ cũng khó: tìm theo tên thì trúng người trùng tên, theo số cũ thì sót dòng đã sửa dở. Trong schema lab, số điện thoại sống ở **một** dòng của `customers`; chuyến chỉ giữ `customer_id`:

```sql
SET search_path TO toinoi;
UPDATE customers SET phone = '0900000011'
WHERE id = 'c0000000-0000-4000-8000-000000000001'
RETURNING full_name, phone;
```

Kết quả: một dòng `Nguyễn Văn An | 0900000011`, `UPDATE 1`. Mọi chuyến của khách An «thấy» số mới ngay vì không chuyến nào cất số điện thoại, và `uq_customers_phone`, `ck_customers_phone_format` (PG-005) bảo vệ số mới — bảng phẳng không có chỗ đặt hai ràng buộc đó.

### 3.2. Thêm: muốn có tài xế phải bịa ra chuyến

Tài xế 8006 Trịnh Văn Phúc mới tuyển, chưa chạy chuyến nào. Khoá duy nhất của bảng phẳng là `trip_code`:

```sql
SET search_path TO phang;
-- Lỗi cố ý. Dòng dưới là chỉ thị cho runner của lab; psql coi là chú thích.
-- expect-error: 23502 trip_code
INSERT INTO trip_flat (driver_code, driver_name, kind, status, fare, requested_at,
                       customer_phone, customer_name, city_code, city_name)
VALUES (8006, 'Trịnh Văn Phúc', 'ride', 'requested', 0, '2026-11-03 10:00+07', '?', '?', 'DN', 'Đà Nẵng');
```

```text
ERROR:  null value in column "trip_code" of relation "trip_flat" violates not-null constraint
DETAIL:  Failing row contains (null, ride, requested, 0, 2026-11-03 10:00:00+07, ?, ?, 8006, Trịnh Văn Phúc, DN, Đà Nẵng, null, null).
```

Muốn lưu, ai đó sẽ bịa mã chuyến và khách `'?'`, rồi mọi báo cáo phải nhớ loại dòng giả. Trong lab, `drivers` là bảng riêng.

### 3.3. Xoá: dọn chuyến huỷ, mất luôn quán

```sql
DELETE FROM trip_flat WHERE status = 'cancelled'
RETURNING trip_code, customer_name, merchant_name;
```

```text
 trip_code | customer_name  |  merchant_name
-----------+----------------+-----------------
    100003 | Lê Hoàng Cường | Bún Chả Ngõ Mẫu
    100006 | Nguyễn Văn An  |
(2 rows)

DELETE 2
```

Sau đó lọc theo tên quán Bún Chả Ngõ Mẫu hay tên khách Lê Hoàng Cường đều trả `(0 rows)`: quán và khách biến mất vì đơn duy nhất của họ là đơn huỷ. Ở schema lab thì ngược lại — FK còn chặn xoá khách đang có chuyến: `DELETE FROM customers WHERE id = 'c0000000-…-000000000003'` báo `violates foreign key constraint "fk_trips_customer"` (đã chạy thật).

## 4. 1NF, 2NF, 3NF ở mức đủ dùng

Ba dạng chuẩn đầu là ba câu hỏi kiểm tra, mỗi câu sửa một loại lặp.

### 1NF — mỗi ô một giá trị

**Dạng chuẩn 1** (first normal form): mỗi ô một giá trị đơn, không danh sách hay nhóm lặp trong một cột. `status_history` vi phạm. Hỏi «chuyến nào từng được nhận» phải dò chuỗi:

```sql
SELECT trip_code, status_history
FROM trip_flat
WHERE status_history LIKE '%accepted%'
ORDER BY trip_code;
```

```text
 trip_code |                            status_history
-----------+----------------------------------------------------------------------
    100000 | requested,searching,accepted,arriving,picked_up,in_transit,completed
    100002 | requested,searching,accepted,arriving,picked_up,in_transit,completed
    100012 | requested,searching,accepted
(3 rows)
```

(Không có 100003 vì đơn huỷ đã bị xoá ở 3.3.) Đúng hôm nay, nhưng chuỗi không có giờ của từng bước, không CHECK được từng trạng thái thuộc 8 giá trị, và `LIKE` sẽ khớp nhầm khi có trạng thái mới chứa chữ `accepted`. Sửa: mỗi lần đổi trạng thái là **một dòng** — bảng `trip_events` của lab, có `ck_trip_events_status` và `occurred_at` riêng.

PostgreSQL có mảng và JSONB nên một ô nhiều giá trị không bị cấm về kỹ thuật; câu hỏi 1NF thực dụng là *có ai cần lọc, ràng buộc hay tham chiếu từng phần tử không* — có thì tách dòng (JSONB: PG-053, Phần 6).

### 2NF — phụ thuộc vào toàn bộ khoá

Giả sử sửa 1NF vội: một dòng mỗi (chuyến, trạng thái), khoá ghép `(trip_code, status)`, giữ nguyên cột khách, tài xế, cước. Khi đó `customer_name` chỉ phụ thuộc `trip_code` — **một phần** khoá — nên lặp ở cả 7 dòng sự kiện của một chuyến. **Dạng chuẩn 2** yêu cầu mọi cột không thuộc khoá phụ thuộc vào **toàn bộ** khoá; dạng chuẩn này chỉ có nghĩa với khoá nhiều cột. Sửa: cột của chuyến ở bảng chuyến, bảng sự kiện chỉ giữ `trip_id`, `status`, `occurred_at`.

### 3NF — không phụ thuộc bắc cầu

`trip_code → driver_code` và `driver_code → driver_name`, nên `driver_name` phụ thuộc `trip_code` qua trung gian — **phụ thuộc bắc cầu** (transitive dependency). **Dạng chuẩn 3** yêu cầu cột không thuộc khoá chỉ phụ thuộc vào khoá. Mỗi chuỗi bắc cầu tách thành một bảng:

| Chuỗi trong `trip_flat` | Bảng trong lab | Chuyến giữ |
|---|---|---|
| `customer_phone → customer_name` | `customers` (`uq_customers_phone`) | `customer_id` |
| `driver_code → driver_name` | `drivers` (`uq_drivers_code`) | `driver_id` (NULL khi chưa ai nhận) |
| `city_code → city_name` | `cities` (`uq_cities_code`) | `city_id` |
| tên quán (thiếu định danh) | `merchants` (`id` uuid) | `merchant_id` (chỉ đơn đồ ăn) |
| `status_history` | `trip_events` | — sự kiện trỏ về chuyến |

Đó chính là `01-schema.sql`. Dạng chuẩn cao hơn (BCNF, 4NF…) xử lý trường hợp hiếm hơn; với Tới Nơi, 3NF cộng thói quen «mọi thực thể có định danh riêng» là đủ. Ghép lại thành dòng phẳng là việc của JOIN (PG-013, Phần 2).

## 5. Dữ liệu lặp còn sót trong schema lab

Schema chuẩn hoá vẫn có chỗ một dữ kiện xuất hiện hai lần: `trips.city_id` cùng tồn tại với `merchants.city_id` và `drivers.city_id`. Fixture sạch, nhưng không gì *bắt buộc* quán của đơn ở cùng thành phố với đơn. CHECK không giúp được: tài liệu PostgreSQL nói rõ CHECK không tham chiếu được dữ liệu ngoài dòng đang kiểm. Lab đã đo: script sinh dữ liệu của hồ sơ Tới Nơi (thu nhỏ 2.000 chuyến) qua mọi CHECK nhưng trượt 07-B1 ở 1.638 chuyến (tài xế khác thành phố) và 07-B2 ở 780 chuyến (quán khác thành phố), vì script chọn tài xế, quán ngẫu nhiên.

### FK hai cột cho quán

Dùng FK trên **cặp** `(merchant_id, city_id)`. Cột được FK tham chiếu phải là khoá chính hoặc có UNIQUE, nên cần `UNIQUE (id, city_id)` cho `merchants` — dư về logic (`id` đã duy nhất) nhưng là điều kiện để tham chiếu cặp cột. Chạy sau `00 → 02`:

```sql
SET search_path TO toinoi;
ALTER TABLE merchants ADD CONSTRAINT uq_merchants_id_city UNIQUE (id, city_id);
ALTER TABLE trips ADD CONSTRAINT fk_trips_merchant_city
  FOREIGN KEY (merchant_id, city_id) REFERENCES merchants (id, city_id);

-- Đơn đồ ăn ở SG (city 1) đặt quán Bún Chả Ngõ Mẫu ở Hà Nội (city 2)
INSERT INTO trips (kind, city_id, customer_id, merchant_id, fare, requested_at, updated_at)
VALUES ('food', 1, 'c0000000-0000-4000-8000-000000000001',
        'e0000000-0000-4000-8000-000000000002', 50000,
        '2026-11-03 10:00:00+07', '2026-11-03 10:00:00+07');
```

```text
ALTER TABLE
ALTER TABLE
ERROR:  insert or update on table "trips" violates foreign key constraint "fk_trips_merchant_city"
DETAIL:  Key (merchant_id, city_id)=(e0000000-0000-4000-8000-000000000002, 1) is not present in table "merchants".
```

Mặc định, dòng có một cột FK là NULL không bị kiểm tra, nên FK này không đụng `ride`/`parcel` (`merchant_id` NULL); luật «chỉ đơn đồ ăn có quán» vẫn do `ck_trips_merchant_only_food` giữ. Cái giá là một UNIQUE (kèm index) trên `merchants` — đáng, vì quán gần như không đổi thành phố.

### Cùng kỹ thuật cho tài xế thì vỡ

Áp y hệt cho `drivers` (`uq_drivers_id_city` và `fk_trips_driver_city` trên `(driver_id, city_id)`), rồi tài xế 8001 chuyển từ SG ra Hà Nội:

```sql
UPDATE drivers SET city_id = 2 WHERE code = 8001;
```

```text
ERROR:  update or delete on table "drivers" violates foreign key constraint "fk_trips_driver_city" on table "trips"
DETAIL:  Key (id, city_id)=(d0000000-0000-4000-8000-000000008001, 1) is still referenced from table "trips".
```

Tài xế không được đổi thành phố vì từng chạy chuyến ở SG. «Chữa» bằng `ON UPDATE CASCADE` (sao giá trị mới sang dòng tham chiếu) còn tệ hơn. Tạo lại FK có `ON UPDATE CASCADE` rồi chạy lại câu `UPDATE` (lần này `UPDATE 1`): `SELECT code, status, city_id FROM trips WHERE driver_id = 'd0000000-…-000000008001'` trả ba chuyến 100000, 100008 (`completed`) và 100012 (`accepted`), cả ba đều đổi từ `city_id = 1` thành **2**. Chuyến 100000 chở khách ở TP. Hồ Chí Minh sáng 02/11 giờ bị ghi là ở Hà Nội — lịch sử bị viết lại, doanh thu theo thành phố của tháng trước đổi theo.

Bài học nằm ở nghĩa của cột: **`trips.city_id` và `drivers.city_id` không phải cùng một dữ kiện.** Cột thứ nhất là nơi chuyến diễn ra, cố định từ lúc đặt. Cột thứ hai là nơi tài xế *đang* đăng ký, đổi theo thời gian. Quy tắc «tài xế chỉ nhận chuyến ở thành phố mình đăng ký» chỉ đúng **tại thời điểm nhận**, nên phải kiểm tra ở bước nhận chuyến, trong cùng giao dịch với `UPDATE` nhận chuyến (Phần 4), cộng một truy vấn đối soát kiểu 07-B1 — viết sao cho không báo nhầm chuyến cũ (bài tập 3).

### Dữ liệu suy ra: `drivers.status = 'busy'`

`busy` suy được từ `trips` (tài xế có chuyến `accepted` … `in_transit`). Lab vẫn lưu nó để lọc nhanh tài xế rảnh; PG-006 đổi nó bằng tay khi nhận và hoàn thành chuyến, quên một bước là lệch — 07-B4 bắt lỗi đó. Đây là phi chuẩn hoá *có chủ đích*.

## 6. Khi nào chấp nhận phi chuẩn hoá

**Phi chuẩn hoá** (denormalization) là cố ý giữ một dữ kiện ở nhiều chỗ. Trước khi gọi một cột như vậy là phi chuẩn hoá, hãy hỏi: nó là *bản sao của cùng dữ kiện*, hay là *một dữ kiện khác* trông giống?

### Snapshot giá cước: dữ kiện tại thời điểm

`trips.fare` là cước đã báo và khách đã chấp nhận lúc đặt: chuyến 100000 là 45.000 đ, chuyến 100008 là 0 đ (khuyến mãi). Bỏ `fare` để tính lại từ bảng giá cho «đúng chuẩn hoá» là sai: (1) bảng giá tăng ngày 03/11 thì cước chuyến 02/11 *không được* đổi — khách đã trả; (2) cước còn phụ thuộc quãng đường, phụ phí, khuyến mãi — không bảng giá nào tính ra 0 đ của 100008; (3) đối soát, hoàn tiền cần đúng con số lúc đặt. `fare` không phụ thuộc hàm vào `(city_id, kind)`; nó phụ thuộc vào **chuyến** — đúng chỗ của nó.

Muốn biết cước được tính từ phiên bản giá nào, giữ bảng giá **có phiên bản** thay vì sửa đè. Bản phác (rút gọn, ràng buộc chưa đặt tên theo quy ước lab; số giá là **giả định**, không có trong hồ sơ):

```sql
CREATE TABLE fare_rule_versions (
  id         integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  city_id    smallint NOT NULL REFERENCES cities (id),
  kind       text NOT NULL CHECK (kind IN ('ride', 'food', 'parcel')),
  base_fare  integer NOT NULL CHECK (base_fare >= 0),
  per_km     integer NOT NULL CHECK (per_km >= 0),
  valid_from timestamptz NOT NULL,
  UNIQUE (city_id, kind, valid_from)
);

INSERT INTO fare_rule_versions (city_id, kind, base_fare, per_km, valid_from) VALUES
  (1, 'ride', 12000, 4500, '2026-10-01 00:00+07'),
  (1, 'ride', 13000, 5000, '2026-11-03 00:00+07');

-- Phiên bản áp dụng cho chuyến đặt 07:58 ngày 02/11 ở SG
SELECT id, base_fare, per_km, valid_from
FROM fare_rule_versions
WHERE city_id = 1 AND kind = 'ride' AND valid_from <= '2026-11-02 07:58+07'
ORDER BY valid_from DESC
LIMIT 1;
```

```text
 id | base_fare | per_km |       valid_from
----+-----------+--------+------------------------
  1 |     12000 |   4500 | 2026-10-01 00:00:00+07
(1 row)
```

Câu cuối dùng đúng kỹ thuật PG-007. Bảng giá có phiên bản trả lời «áp giá nào»; `fare` trả lời «khách trả bao nhiêu». Cần cả hai thì giữ cả hai — chuyến có thể thêm `fare_rule_version_id` để kiểm toán. Lập luận này cũng áp dụng cho chứng từ — xem bài tập 2.

### Bản sao để đọc nhanh: được, nếu có luật

`drivers.status`, bộ đếm, cột tổng hợp là bản sao thật sự — suy ra được từ nguồn. Chấp nhận khi lợi ích đọc rõ ràng và đủ bốn điều; với `drivers.status`: nguồn dữ liệu gốc xác định (chuyến đang chạy trong `trips`), đường ghi duy nhất cùng giao dịch với nguồn (bước nhận/hoàn thành — giao dịch ở Phần 4), truy vấn kiểm tra lệch (07-B4), và dựng lại được từ nguồn khi lệch.

Bảng phẳng của CSKH cũng hợp lệ *ở dạng này*: một view hay materialized view dựng từ bảng chuẩn hoá, chỉ để đọc (PG-056, Phần 6) — không phải nơi sửa số điện thoại.

## 7. Ranh giới mô hình: chuyến và sự kiện chuyến

Lab tách một chuyến thành hai bảng có hai vòng đời khác nhau. Một dòng `trips` là trạng thái **hiện tại**, bị `UPDATE` khoảng 8 lần mỗi chuyến (hồ sơ: ≈ 560 UPDATE/giây lúc cao điểm), trả lời «chuyến nào đang chạy, tài xế nào đang bận». Một dòng `trip_events` là **một lần** đổi trạng thái, chỉ `INSERT` (hồ sơ: ~16 triệu dòng/ngày), trả lời «chuyện gì đã xảy ra, lúc nào» cho khiếu nại và đối soát.

Chuyến 100029 ở PG-006 có đúng một dòng `trips` (`completed`) và bảy dòng `trip_events`, từ `requested` lúc 09:00:00 tới `completed` lúc 09:31:00 (phần `04-lifecycle.sql` của `OUTPUT.txt`).

Chỉ có `trips` thì mất lịch sử. Chỉ có `trip_events` (hướng event sourcing) thì câu «chuyến nào đang `searching`» phải dựng lại trạng thái từ sự kiện — làm được, nhưng làm mọi truy vấn đọc phức tạp hơn để đổi lấy thứ Tới Nơi chưa cần.

Tách hai bảng tạo ra một **bất biến xuyên ranh giới**: sự kiện mới nhất phải khớp trạng thái hiện tại. Không ràng buộc khai báo nào diễn đạt được; lab giữ nó bằng cách ghi `UPDATE trips` và `INSERT trip_events` cùng nhau (PG-006) và kiểm bằng 07-B7. Ranh giới cũng nói ai sở hữu ai: sự kiện thuộc về chuyến (`fk_trip_events_trip`), nên xoá chuyến còn sự kiện bị chặn. Thanh toán, đánh giá sau này là bảng riêng trỏ về chuyến, không phải thêm cột vào `trips`.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Ba câu hỏi trước khi thêm một cột</span>
<p>(1) Cột phụ thuộc vào khoá nào — có phải khoá của bảng này? (2) Giá trị gốc đổi thì dòng này có <b>phải</b> đổi theo? Phải ⇒ tham chiếu; không được ⇒ snapshot. (3) Ai ghi cột này, truy vấn nào phát hiện khi nó lệch?</p>
</div>

## 8. Lỗi thường gặp

- **Chuẩn hoá theo dữ liệu mẫu thay vì nghiệp vụ:** hôm nay không có hai quán trùng tên không có nghĩa tên quán là khoá.
- **Gọi mọi bản sao là phi chuẩn hoá** rồi xoá snapshot như `fare`; hoặc ngược lại, **giữ bản sao không có truy vấn kiểm tra**.
- **Dùng `ON UPDATE CASCADE` cho hết lỗi FK:** nó sao giá trị mới vào cả dòng lịch sử.
- **Coi CHECK là đủ:** quy tắc nhiều dòng, nhiều bảng hay theo thời gian cần FK, kiểm tra trong giao dịch hoặc truy vấn đối soát.

## 9. Đánh đổi

| Lựa chọn | Được | Mất |
|---|---|---|
| Chuẩn hoá tới 3NF (thay bảng phẳng làm nguồn) | Mỗi sự thật sửa một chỗ; ràng buộc đặt đúng bảng | Đọc dạng phẳng cần JOIN |
| FK hai cột `(merchant_id, city_id)` | DB chặn quán khác thành phố | Thêm UNIQUE + index trên `merchants` |
| Bản sao suy ra (`busy`) | Lọc nhanh tài xế rảnh | Cần đường ghi duy nhất và truy vấn kiểm tra |
| Tách `trips` / `trip_events` | Trạng thái hiện tại rẻ, lịch sử đầy đủ | Bất biến xuyên hai bảng, phải ghi cùng nhau |

## 10. Bài tập

**Bài 1.** Đội tuyển dụng gửi bảng `driver_onboarding(driver_code, driver_name, city_code, city_name, document_type, document_number, verified_at)`, mỗi dòng một giấy tờ của tài xế. Liệt kê phụ thuộc hàm, chỉ ra vi phạm 2NF/3NF và đề xuất cách tách.

<details><summary><b>Lời giải</b></summary>
<p>Khoá: <code>(driver_code, document_type)</code> (giả định mỗi loại giấy tờ một bản). Phụ thuộc: <code>driver_code → driver_name, city_code</code>; <code>city_code → city_name</code>; <code>(driver_code, document_type) → document_number, verified_at</code>. <code>driver_name</code>, <code>city_code</code> chỉ phụ thuộc một phần khoá ⇒ vi phạm 2NF; <code>city_name</code> bắc cầu qua <code>city_code</code> ⇒ vi phạm 3NF. Tách: dùng <code>drivers</code>, <code>cities</code> của lab, thêm <code>driver_documents(driver_id, document_type, document_number, verified_at)</code> có FK về <code>drivers</code> và UNIQUE <code>(driver_id, document_type)</code>. Nhiều bản cùng loại theo thời gian thì khoá phải gồm mốc hiệu lực — hỏi chủ nghiệp vụ.</p>
</details>

**Bài 2.** Kế toán muốn biên lai in tên khách. Ba tháng sau khách đổi tên; biên lai in lại giữ tên cũ hay tên mới? Đề xuất mô hình cho từng câu trả lời.

<details><summary><b>Lời giải</b></summary>
<p>Câu hỏi nghiệp vụ/pháp lý, phải hỏi trước. Biên lai là chứng từ đã phát hành ⇒ tên lúc phát hành là dữ kiện của chứng từ: snapshot trong bảng biên lai (vd. <code>customer_name_at_issue</code>), không sửa khi hồ sơ đổi. Chỉ là bản in tiện lợi ⇒ tham chiếu <code>customer_id</code>. Đừng chép <code>full_name</code> vào <code>trips</code> khi chưa quyết định nó mang nghĩa nào.</p>
</details>

**Bài 3.** Sau khi tài xế 8001 chuyển ra Hà Nội (không dùng FK hai cột), truy vấn 07-B1 của lab báo gì? Sửa để chỉ báo trường hợp thật sự cần xử lý.

<details><summary><b>Lời giải</b></summary>
<p>Chạy thật sau <code>00 → 02</code> và <code>UPDATE drivers SET city_id = 2 WHERE code = 8001</code>: 07-B1 trả 3 dòng — 100000, 100008, 100012 — vì nó so thành phố <i>hiện tại</i> của tài xế với mọi chuyến, kể cả chuyến đã xong. Chỉ xét chuyến đang chạy:</p>
<pre><code class="language-sql">SELECT t.code, t.status, t.city_id AS thanh_pho_chuyen, d.code AS tai_xe, d.city_id AS thanh_pho_tai_xe
FROM trips AS t
JOIN drivers AS d ON d.id = t.driver_id
WHERE d.city_id &lt;&gt; t.city_id
  AND t.status IN ('accepted', 'arriving', 'picked_up', 'in_transit')
ORDER BY t.code;</code></pre>
<p>Kết quả: 1 dòng — 100012 (<code>accepted</code>, chuyến SG, tài xế giờ thuộc HN). Đây là lỗi thật: đổi thành phố khi tài xế còn chuyến đang chạy; quy trình chuyển thành phố nên chặn việc đó. (JOIN học ở Phần 2; câu này giữ kiểu viết của <code>07-checks.sql</code>.)</p>
</details>

## 11. Đọc tiếp

- [PostgreSQL 17 · Constraints](https://www.postgresql.org/docs/17/ddl-constraints.html): FK nhiều cột, cột được tham chiếu phải là khoá hoặc UNIQUE, xử lý NULL, `CASCADE`, giới hạn của CHECK.
- [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi): số liệu tải của `trips` và `trip_events`.
- Lab: `p01/01-schema.sql`, `p01/07-checks.sql` (nhóm B), `p01/README.md` mục «Tư liệu cho PG-008».

Bài tiếp theo: [PG-009 · Thay đổi schema mà dữ liệu cũ vẫn đọc được](/articles/postgresql-schema-migration-tuong-thich).
