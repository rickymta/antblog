---
slug: mssql-mo-hinh-quan-he-chuan-hoa
title: "Chuẩn hoá và ranh giới mô hình dữ liệu"
summary: "Khách Tới Nơi đổi số điện thoại, tổng đài vẫn gọi số cũ vì bảng phẳng chỉ sửa một dòng. Bài này dùng 1NF–3NF trên SQL Server 2022 để tìm dữ kiện lặp, rồi chọn cách giữ quy tắc liên bảng: FK hai cột, filtered index, trigger hay truy vấn đối soát."
category: mssql
level: advanced
tags: [mssql, sql-server-2022, ms-008, normalization, data-modeling, filtered-index, trigger, toi-noi]
language: vi
---

Đội CSKH của Tới Nơi muốn "một bảng duy nhất": mỗi dòng một chuyến, có sẵn tên và số điện thoại khách, tên tài xế, thành phố, quán, cả chuỗi lịch sử trạng thái. Xuất Excel là xong, cần thì sửa thẳng trong bảng.

Ba tuần sau, khách Nguyễn Văn An đổi số điện thoại. Nhân viên sửa ở chuyến gần nhất, và tối đó tổng đài gọi xác nhận một chuyến cũ vào số cũ. Không câu lệnh nào lỗi; database chỉ đang giữ **hai sự thật khác nhau về cùng một người**. Bài này dùng chuẩn hoá để gọi tên lỗi đó, rồi đi tới phần khó hơn mà SQL Server buộc ta phải chọn: những quy tắc nằm giữa nhiều bảng thì giữ bằng gì, khi CHECK chỉ nhìn thấy một dòng.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-008 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/mssql-bang-khoa-chinh-clustered-index">MS-003 · bảng, khoá chính và clustered index</a>, <a href="/articles/mssql-create-table-constraints">MS-005 · CREATE TABLE và ràng buộc</a>, <a href="/articles/mssql-insert-update-delete-output">MS-006 · INSERT, UPDATE, DELETE và OUTPUT</a>. Lab: <code>examples/toi-noi-mssql/p01</code> (schema <code>01</code>, kiểm tra <code>07</code>). Sau bài này, bạn giải thích được bất thường cập nhật, biết dữ kiện nào được phép lặp và chọn được cách giữ quy tắc liên bảng trên SQL Server.</p>
</div>

## 1. Bảng phẳng: một dòng biết mọi thứ

Lab không có tệp cho bảng phẳng, nên bài dựng nó trong một **database nháp** riêng, `Scratch_ms008`, để không đụng vào `ToiNoi`. Mở kết nối tới lab như ở [MS-002](/articles/mssql-cai-dat-docker-dbstudio), dán các khối SQL của mục 1 và 3 vào query editor của DBStudio rồi bấm Ctrl+Shift+Enter ("Chạy cả script"); mỗi `SELECT` hiện thành một result set riêng. Kết quả trong bài là lần chạy thật bằng `sqlcmd` trên SQL Server 2022 16.0.4325.4 (RTM-CU28); dòng `Msg` đã bỏ tên máy chủ.

```sql
CREATE DATABASE Scratch_ms008 COLLATE Vietnamese_100_CI_AS;
GO
USE Scratch_ms008;
GO
CREATE SCHEMA phang;
GO
CREATE TABLE phang.trip_flat (
    trip_code      bigint            NOT NULL PRIMARY KEY,
    kind           varchar(10)       NOT NULL,
    status         varchar(12)       NOT NULL,
    fare           int               NOT NULL,
    requested_at   datetimeoffset(3) NOT NULL,
    customer_phone varchar(10)       NOT NULL,
    customer_name  nvarchar(100)     NOT NULL,
    driver_code    int               NULL,
    driver_name    nvarchar(100)     NULL,
    city_code      char(2)           NOT NULL,
    city_name      nvarchar(100)     NOT NULL,
    merchant_name  nvarchar(200)     NULL,
    status_history varchar(200)      NULL
);
INSERT INTO phang.trip_flat VALUES
 (100000, 'ride', 'completed', 45000, '2026-11-02 07:58 +07:00', '0900000001', N'Nguyễn Văn An', 8001, N'Bùi Văn Khoa', 'SG', N'TP. Hồ Chí Minh', NULL,
  'requested,searching,accepted,arriving,picked_up,in_transit,completed'),
 (100002, 'food', 'completed', 62000, '2026-11-02 11:05 +07:00', '0900000002', N'Trần Thị Bình', 8002, N'Hồ Thị Lan', 'SG', N'TP. Hồ Chí Minh', N'Cơm Tấm Phố Lab',
  'requested,searching,accepted,arriving,picked_up,in_transit,completed'),
 (100003, 'food', 'cancelled', 71000, '2026-11-02 12:00 +07:00', '0900000003', N'Lê Hoàng Cường', 8003, N'Ngô Quang Minh', 'HN', N'Hà Nội', N'Bún Chả Ngõ Mẫu',
  'requested,searching,accepted,cancelled'),
 (100006, 'ride', 'cancelled', 52000, '2026-11-02 18:00 +07:00', '0900000001', N'Nguyễn Văn An', NULL, NULL, 'SG', N'TP. Hồ Chí Minh', NULL,
  'requested,searching,cancelled'),
 (100012, 'ride', 'accepted', 39000, '2026-11-03 08:20 +07:00', '0900000001', N'Nguyễn Văn An', 8001, N'Bùi Văn Khoa', 'SG', N'TP. Hồ Chí Minh', NULL,
  'requested,searching,accepted');

SELECT trip_code, customer_name, customer_phone, driver_name, city_name, merchant_name
FROM phang.trip_flat ORDER BY trip_code;
```

```text
trip_code|customer_name|customer_phone|driver_name|city_name|merchant_name
---------|-------------|--------------|-----------|---------|-------------
100000|Nguyễn Văn An|0900000001|Bùi Văn Khoa|TP. Hồ Chí Minh|NULL
100002|Trần Thị Bình|0900000002|Hồ Thị Lan|TP. Hồ Chí Minh|Cơm Tấm Phố Lab
100003|Lê Hoàng Cường|0900000003|Ngô Quang Minh|Hà Nội|Bún Chả Ngõ Mẫu
100006|Nguyễn Văn An|0900000001|NULL|TP. Hồ Chí Minh|NULL
100012|Nguyễn Văn An|0900000001|Bùi Văn Khoa|TP. Hồ Chí Minh|NULL
```

Chỉ 5 dòng mà "TP. Hồ Chí Minh" lặp 4 lần, khách An 3 lần. Với khoảng 2 triệu chuyến mỗi ngày trong [hồ sơ Tới Nơi](/articles/du-an-mau-toi-noi), mỗi khách quen lặp hàng trăm lần.

## 2. Phụ thuộc hàm: ngôn ngữ để nói "dữ kiện này ở đâu"

**Phụ thuộc hàm** (functional dependency), viết `X → Y`: hai dòng cùng giá trị `X` thì **buộc** cùng giá trị `Y`. Đây là phát biểu về **nghiệp vụ**; dữ liệu đang có chỉ phản bác được một phụ thuộc, không chứng minh được nó.

Theo nghiệp vụ Tới Nơi, `trip_flat` có: `trip_code →` mọi cột (khoá của bảng); `customer_phone → customer_name` (khách đăng nhập bằng số, một số là một khách); `driver_code → driver_name`; `city_code → city_name`. Tên quán thì **không** xác định được quán, vì hai quán khác nhau có thể trùng tên. Bảng phẳng không chỉ lặp dữ liệu mà còn **thiếu định danh**: muốn biết hai đơn có cùng một quán không, ta chỉ có cái tên.

**Chuẩn hoá** (normalization) là tách bảng sao cho mỗi dữ kiện nằm đúng một chỗ: cột phụ thuộc vào cái gì thì nằm trong bảng có khoá là cái đó. Nhờ vậy, đổi một sự thật chỉ phải sửa **một** dòng.

## 3. Ba bất thường của bảng phẳng

Thuật ngữ kinh điển là **bất thường cập nhật** (update anomaly), gồm ba dạng: sửa, thêm, xoá.

### 3.1. Sửa: hai sự thật về một người

```sql
UPDATE phang.trip_flat SET customer_phone = '0900000011' WHERE trip_code = 100012;
SELECT trip_code, customer_name, customer_phone FROM phang.trip_flat
WHERE customer_name = N'Nguyễn Văn An' ORDER BY trip_code;
```

```text
trip_code|customer_name|customer_phone
---------|-------------|--------------
100000|Nguyễn Văn An|0900000001
100006|Nguyễn Văn An|0900000001
100012|Nguyễn Văn An|0900000011
```

Khách An giờ có hai số, và không ràng buộc nào chặn được vì bảng không có khái niệm "khách". Sửa cho đủ cũng khó: tìm theo tên thì trúng người trùng tên, theo số cũ thì sót dòng đã sửa dở. Trong schema lab, số điện thoại nằm ở **một** dòng của `customers`; chuyến chỉ giữ `customer_id`. Chạy trên bản sao schema lab trong database nháp, bọc trong transaction rồi `ROLLBACK`:

```sql
BEGIN TRAN;
UPDATE toinoi.customers SET phone = '0900000011'
OUTPUT inserted.full_name, deleted.phone AS so_cu, inserted.phone AS so_moi
WHERE id = 'c0000000-0000-4000-8000-000000000001';
ROLLBACK;
```

```text
full_name|so_cu|so_moi
---------|-----|------
Nguyễn Văn An|0900000001|0900000011
```

Mệnh đề `OUTPUT` ([MS-006](/articles/mssql-insert-update-delete-output)) cho thấy cả giá trị trước và sau trong một câu. Mọi chuyến của khách An "thấy" số mới ngay, vì không chuyến nào cất số điện thoại; `uq_customers_phone` và `ck_customers_phone_format` ([MS-005](/articles/mssql-create-table-constraints)) bảo vệ số mới. Bảng phẳng không có chỗ đặt hai ràng buộc đó.

### 3.2. Thêm: muốn có tài xế phải bịa ra chuyến

Tài xế 8006 Trịnh Văn Phúc mới tuyển, chưa chạy chuyến nào. Khoá duy nhất của bảng phẳng là `trip_code`:

```sql
INSERT INTO phang.trip_flat (driver_code, driver_name, kind, status, fare, requested_at,
                             customer_phone, customer_name, city_code, city_name)
VALUES (8006, N'Trịnh Văn Phúc', 'ride', 'requested', 0, '2026-11-03 10:00 +07:00', '?', N'?', 'DN', N'Đà Nẵng');
```

```text
Msg 515, Level 16, State 2, Line 2
Cannot insert the value NULL into column 'trip_code', table 'Scratch_ms008.phang.trip_flat'; column does not allow nulls. INSERT fails.
The statement has been terminated.
```

Muốn lưu, ai đó sẽ bịa mã chuyến và khách `'?'`, rồi mọi báo cáo phải nhớ loại dòng giả. Trong lab, `drivers` là bảng riêng: tài xế 8006 tồn tại mà không cần chuyến nào.

### 3.3. Xoá: dọn chuyến huỷ, mất luôn quán

```sql
DELETE FROM phang.trip_flat
OUTPUT deleted.trip_code, deleted.customer_name, deleted.merchant_name
WHERE status = 'cancelled';
SELECT COUNT(*) AS con_quan_bun_cha FROM phang.trip_flat WHERE merchant_name = N'Bún Chả Ngõ Mẫu';
SELECT COUNT(*) AS con_khach_cuong  FROM phang.trip_flat WHERE customer_name = N'Lê Hoàng Cường';
```

```text
trip_code|customer_name|merchant_name
---------|-------------|-------------
100003|Lê Hoàng Cường|Bún Chả Ngõ Mẫu
100006|Nguyễn Văn An|NULL
con_quan_bun_cha
----------------
0
con_khach_cuong
---------------
0
```

Quán Bún Chả Ngõ Mẫu và khách Lê Hoàng Cường biến mất, vì đơn duy nhất của họ là đơn huỷ. Ở schema lab thì ngược lại: FK còn chặn xoá khách đang có chuyến. Ca F3 của `03-fixture-invalid.sql` trong `OUTPUT.txt`:

```text
PASS F3 xoá khách đang có chuyến — lỗi 547: The DELETE statement conflicted with the REFERENCE constraint "fk_trips_customer". The conflict occurred in database "ToiNoi", table "toinoi.trips", column 'customer_id'.
```

## 4. 1NF, 2NF, 3NF ở mức đủ dùng

Ba dạng chuẩn đầu là ba câu hỏi kiểm tra, mỗi câu sửa một loại lặp.

**1NF — mỗi ô một giá trị.** **Dạng chuẩn 1** (first normal form) yêu cầu mỗi ô chứa một giá trị đơn, không danh sách hay nhóm lặp trong một cột. `status_history` vi phạm. Hỏi "chuyến nào từng được nhận" phải dò chuỗi:

```sql
SELECT trip_code, status_history FROM phang.trip_flat
WHERE status_history LIKE '%accepted%' ORDER BY trip_code;
```

```text
trip_code|status_history
---------|--------------
100000|requested,searching,accepted,arriving,picked_up,in_transit,completed
100002|requested,searching,accepted,arriving,picked_up,in_transit,completed
100012|requested,searching,accepted
```

(Không có 100003 vì đơn huỷ đã bị xoá ở 3.3.) Kết quả đúng hôm nay, nhưng chuỗi không có giờ của từng bước, không CHECK được từng trạng thái thuộc 8 giá trị, và `LIKE` sẽ khớp nhầm khi có trạng thái mới chứa chữ `accepted`. Cách sửa: mỗi lần đổi trạng thái là **một dòng**, chính là bảng `trip_events` của lab với `ck_trip_events_status` và `occurred_at` riêng. SQL Server không có kiểu mảng; JSON lưu trong `nvarchar` là lựa chọn khi không ai cần lọc hay ràng buộc từng phần tử (MS-053, Phần 6).

**2NF — phụ thuộc vào toàn bộ khoá.** Giả sử sửa 1NF vội: một dòng mỗi (chuyến, trạng thái), khoá ghép `(trip_code, status)`, giữ nguyên cột khách, tài xế, cước. Khi đó `customer_name` chỉ phụ thuộc `trip_code`, tức **một phần** khoá, nên lặp ở cả 7 dòng sự kiện của một chuyến. **Dạng chuẩn 2** yêu cầu mọi cột không thuộc khoá phụ thuộc vào **toàn bộ** khoá; nó chỉ có nghĩa với khoá nhiều cột.

**3NF — không phụ thuộc bắc cầu.** `trip_code → driver_code` và `driver_code → driver_name`, nên `driver_name` phụ thuộc `trip_code` qua trung gian: **phụ thuộc bắc cầu** (transitive dependency). **Dạng chuẩn 3** yêu cầu cột không thuộc khoá chỉ phụ thuộc vào khoá. Mỗi chuỗi bắc cầu tách thành một bảng:

| Chuỗi trong `trip_flat` | Bảng trong lab | Chuyến giữ |
|---|---|---|
| `customer_phone → customer_name` | `customers` (`uq_customers_phone`) | `customer_id` |
| `driver_code → driver_name` | `drivers` (`uq_drivers_code`) | `driver_id` (NULL khi chưa ai nhận) |
| `city_code → city_name` | `cities` (`uq_cities_code`) | `city_id` |
| tên quán (thiếu định danh) | `merchants` (`id` uniqueidentifier) | `merchant_id` (chỉ đơn đồ ăn) |
| `status_history` | `trip_events` | sự kiện trỏ về chuyến qua `fk_trip_events_trip` |

Đó chính là `01-schema.sql`. Dạng chuẩn cao hơn (BCNF, 4NF…) xử lý những trường hợp hiếm hơn; với Tới Nơi, 3NF cộng thói quen "mọi thực thể có định danh riêng" là đủ. Ghép lại thành dòng phẳng là việc của JOIN (MS-013, Phần 2).

## 5. Chỗ chuẩn hoá dừng lại: dữ liệu suy ra và lịch sử

Schema lab không chuẩn hoá tuyệt đối. Có ba chỗ cố ý giữ dữ kiện "trông như lặp", và mỗi chỗ có một kiểm tra trong `07-checks.sql`.

**`trips.fare` là snapshot, không phải bản sao.** Cước 45.000 đ của chuyến 100000 là con số khách đã chấp nhận lúc đặt; chuyến 100008 là 0 đ vì khuyến mãi. Bảng giá tăng ngày mai thì cước hôm qua *không được* đổi. `fare` phụ thuộc vào **chuyến**, nên nằm đúng chỗ. Muốn biết cước được tính từ phiên bản giá nào thì giữ bảng giá có phiên bản, không xoá `fare`.

**`drivers.status` là dữ liệu suy ra.** `busy` suy được từ `trips` (tài xế có chuyến `accepted` … `in_transit`). Lab vẫn lưu nó để lọc nhanh tài xế rảnh, và [MS-006](/articles/mssql-insert-update-delete-output) đổi nó cùng transaction với bước nhận chuyến. Đây là **phi chuẩn hoá** (denormalization) có chủ đích: cố ý giữ một dữ kiện ở hai chỗ để đọc nhanh hơn. Cái giá là nguy cơ lệch. K4 trong `07-checks.sql` bắt đúng chuyện đó; thử cho chuyến 100012 hoàn thành mà quên trả tài xế 8001 về `online` (trong transaction rồi `ROLLBACK`):

```sql
BEGIN TRAN;
UPDATE toinoi.trips SET status = 'completed', completed_at = '2026-11-03 08:45 +07:00',
                        updated_at = '2026-11-03 08:45 +07:00'
WHERE code = 100012;
SELECT d.code, d.status FROM toinoi.drivers AS d
WHERE (d.status = 'busy' AND NOT EXISTS (
          SELECT 1 FROM toinoi.trips AS t WHERE t.driver_id = d.id
            AND t.status IN ('accepted', 'arriving', 'picked_up', 'in_transit')))
   OR (d.status <> 'busy' AND EXISTS (
          SELECT 1 FROM toinoi.trips AS t WHERE t.driver_id = d.id
            AND t.status IN ('accepted', 'arriving', 'picked_up', 'in_transit')));
ROLLBACK;
```

```text
code|status
----|------
8001|busy
```

Không ràng buộc nào báo lỗi; chỉ truy vấn canh mới thấy. Bản sao suy ra được chấp nhận khi có đủ bốn điều: nguồn dữ liệu gốc xác định (chuyến đang chạy trong `trips`), một đường ghi duy nhất cùng transaction với nguồn, truy vấn kiểm lệch (K4) và cách dựng lại từ nguồn khi lệch.

**`trips.status` và `trip_events`: hai vòng đời.** Một dòng `trips` là trạng thái **hiện tại**, bị `UPDATE` khoảng 8 lần mỗi chuyến; một dòng `trip_events` là **một lần** đổi trạng thái, chỉ `INSERT`. Chuyến 100032 ở `04-lifecycle.sql` có đúng một dòng `trips` (`completed`) và bảy dòng `trip_events`, từ `requested` lúc 09:00:00 tới `completed` lúc 09:31:00. Chỉ giữ `trips` thì mất lịch sử; chỉ giữ `trip_events` thì câu "chuyến nào đang `searching`" phải dựng lại trạng thái từ sự kiện. Tách hai bảng tạo ra một bất biến xuyên ranh giới: sự kiện mới nhất phải khớp trạng thái hiện tại. K5 kiểm nó bằng `CROSS APPLY (SELECT TOP (1) … ORDER BY e.occurred_at DESC, e.id DESC)`; trên dữ liệu lab, `OUTPUT.txt` báo `PASS K5 sự kiện cuối lệch trạng thái chuyến = 0`.

## 6. Quy tắc liên bảng: CHECK chỉ nhìn thấy một dòng

Schema chuẩn hoá vẫn có chỗ một dữ kiện xuất hiện hai lần: `trips.city_id` cùng tồn tại với `drivers.city_id` và `merchants.city_id`. Ba quy tắc trong `07-checks.sql` vì thế nằm **giữa** nhiều dòng hoặc nhiều bảng:

- **K1**: tài xế nhận chuyến cùng thành phố với chuyến.
- **K2**: đơn đồ ăn đặt quán cùng thành phố với chuyến.
- **K3**: một tài xế tối đa một chuyến đang chạy (`accepted` … `in_transit`).

CHECK constraint của SQL Server chỉ đánh giá các cột của dòng đang ghi, nên không viết thẳng được ba quy tắc này. SQL Server cho năm hướng; phần dưới chạy từng hướng trên bản sao schema lab trong `Scratch_ms008` (dựng bằng `01` và `02` với tên database đổi lại).

### 6.1. FK hai cột: hợp với quán (K2)

FK có thể trỏ tới **cặp** cột `(merchant_id, city_id)`. Phía được tham chiếu phải có khoá chính hoặc UNIQUE trên đúng cặp đó, nên cần thêm `UNIQUE (id, city_id)` cho `merchants`. Về logic thì dư (`id` đã duy nhất), nhưng đó là điều kiện để FK tham chiếu được.

```sql
ALTER TABLE toinoi.merchants ADD CONSTRAINT uq_merchants_id_city UNIQUE (id, city_id);
ALTER TABLE toinoi.trips ADD CONSTRAINT fk_trips_merchant_city
    FOREIGN KEY (merchant_id, city_id) REFERENCES toinoi.merchants (id, city_id);
GO
-- Đơn đồ ăn ở SG (city 1) đặt quán Bún Chả Ngõ Mẫu ở Hà Nội (city 2)
INSERT INTO toinoi.trips (kind, city_id, customer_id, merchant_id, fare, requested_at, updated_at)
VALUES ('food', 1, 'c0000000-0000-4000-8000-000000000001',
        'e0000000-0000-4000-8000-000000000002', 50000,
        '2026-11-03 10:00 +07:00', '2026-11-03 10:00 +07:00');
```

```text
Msg 547, Level 16, State 1, Line 1
The INSERT statement conflicted with the FOREIGN KEY constraint "fk_trips_merchant_city". The conflict occurred in database "Scratch_ms008", table "toinoi.merchants".
The statement has been terminated.
```

FK nhiều cột không kiểm dòng có một cột khoá là NULL, nên chuyến `ride`/`parcel` (`merchant_id` NULL) không bị ảnh hưởng; luật "chỉ đơn đồ ăn có quán" vẫn do `ck_trips_merchant_only_food` giữ. Cái giá là một UNIQUE kèm index trên `merchants`. Đáng, vì quán gần như không đổi thành phố.

### 6.2. Cùng kỹ thuật cho tài xế thì vỡ (K1)

Áp y hệt cho `drivers` (`uq_drivers_id_city` và `fk_trips_driver_city`), rồi tài xế 8001 chuyển từ SG ra Hà Nội:

```sql
UPDATE toinoi.drivers SET city_id = 2 WHERE code = 8001;
```

```text
Msg 547, Level 16, State 1, Line 1
The UPDATE statement conflicted with the REFERENCE constraint "fk_trips_driver_city". The conflict occurred in database "Scratch_ms008", table "toinoi.trips".
The statement has been terminated.
```

Tài xế không được đổi thành phố vì từng chạy chuyến ở SG. "Chữa" bằng `ON UPDATE CASCADE` còn tệ hơn. Tạo lại FK có `ON UPDATE CASCADE`, chạy lại câu `UPDATE` trong transaction:

```text
code|status|city_id
----|------|-------
100000|completed|2
100008|completed|2
100012|accepted|2
```

Chuyến 100000 chở khách ở TP. Hồ Chí Minh sáng 02/11 giờ bị ghi là ở Hà Nội: lịch sử bị viết lại, doanh thu theo thành phố của ngày đó đổi theo. Bài học nằm ở nghĩa của cột: **`trips.city_id` và `drivers.city_id` không phải cùng một dữ kiện.** Cột thứ nhất là nơi chuyến diễn ra, cố định từ lúc đặt; cột thứ hai là nơi tài xế *đang* đăng ký, đổi theo thời gian. K1 chỉ đúng **tại thời điểm nhận chuyến**, nên phải kiểm ở bước nhận chuyến.

### 6.3. Filtered unique index: hợp với K3

**Filtered index** (index có điều kiện lọc) chỉ chứa các dòng thoả mệnh đề `WHERE` của nó. Gắn thêm `UNIQUE`, ta có "duy nhất trong tập con": mỗi `driver_id` xuất hiện tối đa một lần **trong số chuyến đang chạy**, còn chuyến đã xong thì không tính.

```sql
CREATE UNIQUE NONCLUSTERED INDEX ux_trips_driver_running
    ON toinoi.trips (driver_id)
    WHERE status IN ('accepted', 'arriving', 'picked_up', 'in_transit');
GO
SELECT name, has_filter, filter_definition FROM sys.indexes WHERE name = 'ux_trips_driver_running';
GO
-- Tài xế 8001 đang chạy 100012 (accepted); gán thêm chuyến 100014
UPDATE toinoi.trips
SET status = 'accepted', driver_id = 'd0000000-0000-4000-8000-000000008001',
    accepted_at = '2026-11-03 08:30 +07:00', updated_at = '2026-11-03 08:30 +07:00'
WHERE code = 100014;
```

```text
name|has_filter|filter_definition
----|----------|-----------------
ux_trips_driver_running|1|([status] IN ('accepted', 'arriving', 'picked_up', 'in_transit'))
Msg 2601, Level 14, State 1, Line 2
Cannot insert duplicate key row in object 'toinoi.trips' with unique index 'ux_trips_driver_running'. The duplicate key value is (d0000000-0000-4000-8000-000000008001).
The statement has been terminated.
```

Cùng câu đó cho tài xế 8003 (đang `online`, không có chuyến chạy) nhận chuyến 100013 thì đổi được 1 dòng. Lỗi là **2601** (unique index), không phải 2627 như UNIQUE constraint ở MS-005: ràng buộc UNIQUE không có mệnh đề `WHERE`, nên quy tắc này chỉ viết được bằng index. Vì index được cập nhật trong cùng câu lệnh ghi, hai phiên không thể cùng chèn hai khoá trùng; P04 sẽ xem phiên thứ hai bị chặn rồi nhận lỗi ra sao.

Một bẫy có thật: ghi vào bảng có filtered index đòi một số tuỳ chọn `SET` đúng giá trị. Phiên `sqlcmd` mặc định `QUOTED_IDENTIFIER OFF` (lab chạy với `-I` để bật nó), và khi tắt thì mọi câu ghi vào `trips` đều hỏng, kể cả câu không đụng tới `driver_id`:

```text
Msg 1934, Level 16, State 1, Line 2
UPDATE failed because the following SET options have incorrect settings: 'QUOTED_IDENTIFIER'. Verify that SET options are correct for use with indexed views and/or indexes on computed columns and/or filtered indexes and/or query notifications and/or XML data type methods and/or spatial index operations.
```

Trước khi thêm filtered index vào bảng nóng, hãy chắc mọi đường ghi (ứng dụng, job, script vận hành) đều chạy với tuỳ chọn mặc định của driver hiện đại.

### 6.4. Trigger: diễn đạt được mọi thứ, với cái giá ẩn

**Trigger** là đoạn T-SQL tự chạy sau (hoặc thay cho) `INSERT`/`UPDATE`/`DELETE`, đọc các dòng vừa ghi qua bảng ảo `inserted` và `deleted`. Trigger cho K1:

```sql
CREATE TRIGGER toinoi.tr_trips_driver_same_city
ON toinoi.trips
AFTER INSERT, UPDATE
AS
BEGIN
    SET NOCOUNT ON;
    IF NOT UPDATE(driver_id) AND NOT UPDATE(city_id) RETURN;
    IF EXISTS (SELECT 1 FROM inserted AS i
               JOIN toinoi.drivers AS d ON d.id = i.driver_id
               WHERE d.city_id <> i.city_id)
        THROW 50001, N'K1: tài xế khác thành phố với chuyến', 1;
END;
```

Cho tài xế 8003 (Hà Nội) nhận chuyến 100014 (SG):

```text
Msg 50001, Level 16, State 1, Procedure tr_trips_driver_same_city, Line 11
K1: tài xế khác thành phố với chuyến
```

Chuyến 100014 vẫn ở `requested`, vì `THROW` trong trigger huỷ câu lệnh cùng transaction chứa nó. Trigger viết theo tập `inserted` nên đúng với câu ghi nhiều dòng. Nhưng phải trả giá: logic nằm ngoài chỗ người đọc DDL thường nhìn; trigger này chỉ canh phía `trips`, đổi thành phố ở `drivers` vẫn lọt nếu không có trigger thứ hai; mỗi câu ghi `trips` tốn thêm một lần đọc `drivers`. Trigger đọc bảng khác cũng chịu chung vấn đề đồng thời với mọi kiểm tra "đọc rồi ghi" (Phần 4).

### 6.5. CHECK gọi scalar UDF: trông gọn, giữ không chặt

CHECK không tham chiếu được bảng khác, nhưng gọi được **hàm vô hướng do người dùng định nghĩa** (scalar UDF), và hàm thì đọc được bảng khác:

```sql
CREATE FUNCTION toinoi.fn_driver_city (@driver_id uniqueidentifier)
RETURNS smallint
AS
BEGIN
    RETURN (SELECT city_id FROM toinoi.drivers WHERE id = @driver_id);
END;
GO
ALTER TABLE toinoi.trips ADD CONSTRAINT ck_trips_driver_same_city
    CHECK (driver_id IS NULL OR toinoi.fn_driver_city(driver_id) = city_id);
```

Gán 8003 cho 100014 bị chặn với lỗi 547 quen thuộc, nêu tên `ck_trips_driver_same_city`. Rồi chuyển tài xế 8001 ra Hà Nội và chạy lại K1 (trong transaction):

```text
đổi thành phố 8001: 1 dòng
k1_vi_pham
----------
3
```

CHECK chỉ chạy khi **dòng của `trips`** bị ghi; đổi `drivers` không kích hoạt nó, nên ba chuyến vi phạm nằm yên trong khi ràng buộc vẫn báo "tin cậy". Hàm chạy một lần cho mỗi dòng được ghi, đọc dữ liệu theo isolation level của phiên, nên hai phiên đồng thời có thể cùng thấy trạng thái cũ. Dùng CHECK + UDF chỉ khi chấp nhận được cả ba điều đó; với K1 thì không.

### 6.6. Chọn gì cho Tới Nơi

| Quy tắc | Cách giữ chính | Vì sao | Lưới an toàn |
|---|---|---|---|
| K2 quán cùng thành phố | FK hai cột `(merchant_id, city_id)` | Quán gần như không đổi thành phố; database chặn mọi đường ghi | K2 trong `07` |
| K1 tài xế cùng thành phố | Kiểm trong transaction nhận chuyến | Đúng tại thời điểm nhận, không đúng mãi mãi | K1 chỉ xét chuyến đang chạy (bài tập 2) |
| K3 một chuyến đang chạy | Filtered unique index | Database chặn kể cả khi hai phiên tranh nhau | K3 trong `07` |
| K4 `busy` khớp chuyến | Cập nhật cùng transaction (MS-006) | Dữ liệu suy ra, chỉ có một đường ghi | K4 trong `07` |
| K5 sự kiện cuối khớp trạng thái | `UPDATE trips` + `INSERT trip_events` cùng transaction | Hai bảng, hai vòng đời | K5 trong `07` |

Lab P01 giữ các quy tắc này bằng truy vấn ở `07` và chưa thêm FK hai cột hay filtered index; đó là bài tập mở rộng của MS-010. Bảng phẳng của CSKH cũng có chỗ hợp lệ: một view, hoặc indexed view cho báo cáo (MS-056, Phần 6), dựng từ bảng chuẩn hoá và chỉ để đọc, không phải nơi sửa số điện thoại.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Ba câu hỏi trước khi thêm một cột</span>
<p>(1) Cột phụ thuộc vào khoá nào, có phải khoá của bảng này? (2) Giá trị gốc đổi thì dòng này có <b>phải</b> đổi theo? Phải ⇒ tham chiếu; không được ⇒ snapshot. (3) Ai ghi cột này, và truy vấn nào phát hiện khi nó lệch?</p>
</div>

## 7. So với PostgreSQL

Cơ chế chuẩn hoá giống nhau; khác ở công cụ giữ quy tắc liên bảng. Ý "duy nhất trong tập con" ở PostgreSQL là partial unique index (`CREATE UNIQUE INDEX … WHERE`), không đòi tuỳ chọn `SET` nào khi ghi; SQL Server gọi là filtered index và có bẫy `QUOTED_IDENTIFIER` ở 6.3. Ở cả hai engine, CHECK không đọc được bảng khác. Bài cùng chủ đề bên series PostgreSQL: [PG-008 · Chuẩn hoá và ranh giới mô hình dữ liệu](/articles/postgresql-mo-hinh-quan-he-chuan-hoa).

## 8. Lỗi thường gặp

- **Chuẩn hoá theo dữ liệu mẫu thay vì nghiệp vụ.** Hôm nay không có hai quán trùng tên không có nghĩa tên quán là khoá.
- **Gọi mọi bản sao là phi chuẩn hoá** rồi xoá snapshot như `fare`; hoặc ngược lại, **giữ bản sao không có truy vấn kiểm tra** như K4.
- **Dùng `ON UPDATE CASCADE` cho hết lỗi FK.** Nó chép giá trị mới vào cả dòng lịch sử.
- **Tin CHECK + UDF như tin FK.** Ràng buộc chỉ canh một phía; phía còn lại đổi thì không ai báo.
- **Thêm filtered index mà không kiểm tuỳ chọn `SET` của mọi đường ghi.** Lỗi 1934 xuất hiện ở job chạy đêm chứ không ở máy lập trình viên.

## 9. Đánh đổi

| Lựa chọn | Được | Mất |
|---|---|---|
| Chuẩn hoá tới 3NF | Mỗi sự thật sửa một chỗ; ràng buộc đặt đúng bảng | Đọc dạng phẳng cần JOIN |
| FK hai cột | Database chặn mọi đường ghi | Thêm UNIQUE + index; đổi giá trị cha bị chặn |
| Filtered unique index | Chặn cả khi đồng thời | Ràng buộc nằm trong index; đòi tuỳ chọn `SET` đúng |
| Trigger | Diễn đạt được quy tắc tuỳ ý | Logic ẩn, phải canh mọi bảng liên quan, tốn thêm đọc |
| CHECK + scalar UDF | Cú pháp gọn | Chỉ canh một phía, chạy theo từng dòng, không an toàn khi đồng thời |
| Kiểm trong transaction + truy vấn đối soát | Linh hoạt, đúng theo thời điểm | Phụ thuộc kỷ luật của mọi đường ghi |

## 10. Bài tập

**Bài 1.** Đội tuyển dụng gửi bảng `driver_onboarding(driver_code, driver_name, city_code, city_name, document_type, document_number, verified_at)`, mỗi dòng một giấy tờ của tài xế. Liệt kê phụ thuộc hàm, chỉ ra vi phạm 2NF/3NF và đề xuất cách tách.

<details><summary><b>Lời giải</b></summary>
<p>Khoá: <code>(driver_code, document_type)</code> (giả định mỗi loại giấy tờ một bản). Phụ thuộc: <code>driver_code → driver_name, city_code</code>; <code>city_code → city_name</code>; <code>(driver_code, document_type) → document_number, verified_at</code>. <code>driver_name</code>, <code>city_code</code> chỉ phụ thuộc một phần khoá ⇒ vi phạm 2NF; <code>city_name</code> bắc cầu qua <code>city_code</code> ⇒ vi phạm 3NF. Tách: dùng <code>drivers</code>, <code>cities</code> của lab, thêm <code>driver_documents(driver_id, document_type, document_number, verified_at)</code> có <code>fk_driver_documents_driver</code> và <code>uq_driver_documents_driver_type UNIQUE (driver_id, document_type)</code>. Nếu nghiệp vụ giữ nhiều bản cùng loại theo thời gian thì khoá phải gồm mốc hiệu lực; hỏi chủ nghiệp vụ trước.</p>
</details>

**Bài 2.** Tài xế 8001 chuyển ra Hà Nội (không dùng FK hai cột). Truy vấn K1 của `07-checks.sql` báo gì? Sửa để chỉ báo trường hợp thật sự cần xử lý.

<details><summary><b>Lời giải</b></summary>
<p>Đã chạy trên bản sao schema lab, trong transaction rồi <code>ROLLBACK</code>: sau <code>UPDATE toinoi.drivers SET city_id = 2 WHERE code = 8001</code>, K1 nguyên bản đếm <b>3</b>, vì nó so thành phố <i>hiện tại</i> của tài xế với mọi chuyến, kể cả chuyến đã xong. Chỉ xét chuyến đang chạy:</p>
<pre><code class="language-sql">SELECT t.code, t.status, t.city_id AS thanh_pho_chuyen, d.code AS tai_xe, d.city_id AS thanh_pho_tai_xe
FROM toinoi.trips AS t
JOIN toinoi.drivers AS d ON d.id = t.driver_id
WHERE d.city_id &lt;&gt; t.city_id
  AND t.status IN ('accepted', 'arriving', 'picked_up', 'in_transit')
ORDER BY t.code;</code></pre>
<pre><code class="language-text">code|status|thanh_pho_chuyen|tai_xe|thanh_pho_tai_xe
----|------|----------------|------|----------------
100012|accepted|1|8001|2</code></pre>
<p>Đây là lỗi thật: đổi thành phố khi tài xế còn chuyến đang chạy. Quy trình chuyển thành phố nên chặn việc đó, hoặc chờ chuyến xong.</p>
</details>

**Bài 3.** Đã có `ux_trips_driver_running`. Đồng nghiệp đề xuất bỏ cột `drivers.status` và bỏ K4 "vì index đã giữ quy tắc". Đồng ý không?

<details><summary><b>Lời giải</b></summary>
<p>Không. Index giữ K3 (tối đa một chuyến đang chạy), không nói gì về <code>drivers.status</code>. Bỏ cột thì mất khả năng phân biệt <code>offline</code> với <code>online</code>: tài xế không có chuyến vẫn có thể đang nghỉ, và đó là dữ kiện gốc, không suy ra được từ <code>trips</code>. Chỉ riêng giá trị <code>busy</code> là suy ra. Có hai hướng hợp lệ: giữ cột và giữ K4 làm lưới an toàn; hoặc chỉ lưu <code>offline/online</code> và tính "bận" bằng <code>EXISTS</code> trên chuyến đang chạy, trả giá bằng truy vấn đọc đắt hơn ở màn hình điều phối. Đo trước khi chọn (Phần 3).</p>
</details>

## 11. Đọc tiếp

- [Microsoft Learn · Unique constraints and check constraints](https://learn.microsoft.com/sql/relational-databases/tables/unique-constraints-and-check-constraints): CHECK đánh giá theo dòng, các giới hạn đã biết.
- [Microsoft Learn · Primary and foreign key constraints](https://learn.microsoft.com/sql/relational-databases/tables/primary-and-foreign-key-constraints): FK nhiều cột, `CASCADE`.
- [Microsoft Learn · Create filtered indexes](https://learn.microsoft.com/sql/relational-databases/indexes/create-filtered-indexes): điều kiện lọc được phép và tuỳ chọn `SET` bắt buộc.
- [Microsoft Learn · DML triggers](https://learn.microsoft.com/sql/relational-databases/triggers/dml-triggers): bảng `inserted`/`deleted`, trigger AFTER và INSTEAD OF.
- Lab: `examples/toi-noi-mssql/p01/01-schema.sql`, `07-checks.sql` (K1–K7), kết quả thật trong `p01/OUTPUT.txt`.

Bài tiếp theo: [MS-009 · Thay đổi schema tương thích: WITH NOCHECK và ràng buộc tin cậy](/articles/mssql-schema-migration-tuong-thich).
