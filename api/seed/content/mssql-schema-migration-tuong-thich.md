---
slug: mssql-schema-migration-tuong-thich
title: "Thay đổi schema tương thích: WITH NOCHECK và ràng buộc tin cậy"
summary: "Tới Nơi cần bắt buộc biển số xe cho tài xế khi bảng đã có dữ liệu và app cũ vẫn chạy. Làm migration expand → backfill → contract trên SQL Server 2022: WITH NOCHECK, cờ is_not_trusted, WITH CHECK CHECK CONSTRAINT, khoá Sch-M và kế hoạch hoàn tác hai mức."
category: mssql
level: advanced
tags: [mssql, sql-server-2022, ms-009, migration, alter-table, constraint, expand-contract, toi-noi]
language: vi
---

Bộ phận an toàn của Tới Nơi chốt một yêu cầu mới: khách phải thấy **biển số xe** của tài xế trước khi lên xe, nên mọi tài xế đều phải có biển số đúng định dạng. Nghe như một dòng SQL: thêm cột `vehicle_plate varchar(12) NOT NULL` vào `toinoi.drivers`. Nhưng bảng đã có tài xế đang chạy chuyến, biển số của họ nằm trong hồ sơ giấy của bộ phận tuyển dụng, và app đăng ký tài xế bản cũ, chưa biết gì về biển số, vẫn chạy cho tới khi bản mới được triển khai hết.

Câu `ADD … NOT NULL` không có `DEFAULT` sẽ lỗi ngay vì dòng cũ không có giá trị; thêm kèm `DEFAULT '???'` thì dữ liệu rác hợp lệ mãi mãi. Bài này đổi schema theo từng bước mà ở mỗi bước dữ liệu cũ vẫn đọc được, app cũ vẫn chạy, luôn có đường lùi, và gặp một khái niệm riêng của SQL Server: ràng buộc **tin cậy** và **không tin cậy**.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-009 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/mssql-create-table-constraints">MS-005 · CREATE TABLE, ràng buộc và IDENTITY</a>, <a href="/articles/mssql-insert-update-delete-output">MS-006 · INSERT, UPDATE, DELETE và OUTPUT</a>, <a href="/articles/mssql-mo-hinh-quan-he-chuan-hoa">MS-008 · Chuẩn hoá và ranh giới mô hình</a>. Lab: <code>examples/toi-noi-mssql/p01</code> (tệp <code>06-migration.sql</code>, <code>06-migration-rollback.sql</code>). Sau bài này, bạn làm được một migration expand → backfill → contract, đọc được cờ <code>is_not_trusted</code> và có kế hoạch hoàn tác chạy lặp được.</p>
</div>

## 1. Migration tương thích ngược là gì?

**Schema migration** là một thay đổi có kiểm soát lên cấu trúc database: thêm cột, thêm ràng buộc, đổi kiểu, tách bảng. Các bài trước dựng schema trên database trống; từ đây schema đã có dữ liệu và người dùng.

Migration **tương thích ngược** (backward compatible) là migration mà sau khi chạy, ứng dụng **phiên bản cũ** vẫn đọc ghi được như trước. Triển khai hiếm khi tức thời: máy chủ được thay dần, app tài xế cập nhật theo từng người. Trong khoảng đó, bản cũ và bản mới **cùng chạy trên một schema**. Ba thứ phải sống sót qua mỗi bước:

| Cần giữ | Hỏng khi | Ví dụ với `drivers` |
|---|---|---|
| Dữ liệu cũ | Ràng buộc mới không khớp dòng đã có | `NOT NULL` trên cột mà 6 tài xế cũ chưa có giá trị |
| Ứng dụng cũ | Schema đòi thứ app cũ không gửi, hoặc mất thứ app cũ cần | Bắt buộc biển số khi app cũ chưa gửi; xoá cột app cũ đang đọc |
| Truy vấn đang chạy | Lệnh đổi schema giữ khoá nặng quá lâu | Quét cả bảng lớn trong khi mọi truy vấn khác phải chờ |

Thêm cột cho phép NULL là thay đổi **an toàn**. Xoá cột, đổi tên cột, thêm `NOT NULL` khi app cũ chưa ghi giá trị là thay đổi **phá vỡ** (breaking change): không bị cấm, chỉ phải tách ra và làm sau cùng.

## 2. Expand → backfill → contract

Cách tách đó có tên **expand–contract** (mở rộng rồi thu hẹp):

- **Expand**: thêm cấu trúc mới mà không ai bị ảnh hưởng (cột cho phép NULL, ràng buộc chưa kiểm dữ liệu cũ).
- **Backfill** (điền ngược): triển khai app mới dùng cấu trúc mới, rồi điền dữ liệu cho dòng cũ.
- **Contract**: khi mọi phiên bản app đã dùng cấu trúc mới và dữ liệu đã sạch, mới siết ràng buộc hoặc gỡ cấu trúc cũ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 900 250" role="img" aria-label="Ba giai đoạn expand, backfill và contract. App v1 ghi được ở hai giai đoạn đầu, bị từ chối với lỗi 515 ở giai đoạn contract. App v2 ghi biển số và chạy được ở cả ba giai đoạn.">
<rect x="180" y="16" width="225" height="46" rx="8" class="f-pk-soft s-pk"/><text x="292" y="36" text-anchor="middle" class="t-b">1 · Expand (E1–E3)</text><text x="292" y="53" text-anchor="middle" class="t-small">cột NULL + CHECK WITH NOCHECK</text>
<rect x="420" y="16" width="225" height="46" rx="8" class="f-pk-soft s-pk"/><text x="532" y="36" text-anchor="middle" class="t-b">2 · Backfill (B1–B3)</text><text x="532" y="53" text-anchor="middle" class="t-small">điền + WITH CHECK CHECK</text>
<rect x="660" y="16" width="225" height="46" rx="8" class="f-pk-soft s-pk"/><text x="772" y="36" text-anchor="middle" class="t-b">3 · Contract (C1–C2)</text><text x="772" y="53" text-anchor="middle" class="t-small">ALTER COLUMN NOT NULL</text>
<text x="20" y="100" class="t-b">Ràng buộc</text>
<rect x="180" y="78" width="225" height="36" rx="6" class="f-surface s-line"/><text x="292" y="101" text-anchor="middle" class="t-small">is_not_trusted = 1</text>
<rect x="420" y="78" width="225" height="36" rx="6" class="f-surface s-line"/><text x="532" y="101" text-anchor="middle" class="t-small">is_not_trusted = 0, NULL vẫn được</text>
<rect x="660" y="78" width="225" height="36" rx="6" class="f-surface s-line"/><text x="772" y="101" text-anchor="middle" class="t-small">CHECK tin cậy + NOT NULL</text>
<text x="20" y="150" class="t-b">App v1</text><text x="20" y="166" class="t-small">không gửi biển số</text>
<rect x="180" y="128" width="465" height="36" rx="6" class="f-phys-soft s-phys"/><text x="412" y="151" text-anchor="middle" class="t-phys">ghi được (cột mới nhận NULL)</text>
<rect x="660" y="128" width="225" height="36" rx="6" class="f-warn-soft s-warn"/><text x="772" y="151" text-anchor="middle" class="t-warn">bị từ chối · lỗi 515</text>
<text x="20" y="200" class="t-b">App v2</text><text x="20" y="216" class="t-small">luôn gửi biển số</text>
<rect x="180" y="178" width="705" height="36" rx="6" class="f-phys-soft s-phys"/><text x="532" y="201" text-anchor="middle" class="t-phys">ghi được ở mọi giai đoạn, phải triển khai xong TRƯỚC bước 3</text>
<text x="450" y="240" text-anchor="middle" class="t-small">Mỗi giai đoạn tự đứng được: dừng giữa chừng hệ thống vẫn chạy, quay lại giai đoạn trước vẫn được.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Expand – contract cho cột biển số.</b> Mã E/B/C khớp nhãn kiểm tra trong <code>06-migration.sql</code>.</figcaption>
</figure>

Lab chạy ba giai đoạn liền nhau để quan sát; ngoài đời, giai đoạn 2 và 3 có thể cách nhau nhiều ngày. `npm run p01` chạy `06` sau `00`…`05` và in 8 dòng `PASS`. Muốn đi từng bước, mở `06-migration.sql` trong query editor của DBStudio trên database vừa chạy `00`…`05`, đặt con trỏ vào một batch và bấm Ctrl+Enter: DBStudio chỉ chạy batch `GO` chứa con trỏ, nên bạn dừng được sau mỗi bước để xem catalog.

## 3. Expand: thêm cột cho phép NULL

```sql
ALTER TABLE toinoi.drivers ADD vehicle_plate varchar(12) NULL;
GO
-- Ứng dụng cũ vẫn thêm tài xế được (cột mới = NULL).
INSERT INTO toinoi.drivers (id, code, full_name, city_id, vehicle_type, status, created_at)
VALUES ('d0000000-0000-4000-8000-000000008007', 8007, N'Tạ Văn Quý', 1, 'bike', 'offline', '2026-11-03 10:00:00 +07:00');
EXEC lab.expect_value N'E1 app cũ thêm tài xế 8007', @@ROWCOUNT, N'1';
```

```text
PASS E1 app cũ thêm tài xế 8007 = 1
```

Câu `INSERT` của app cũ liệt kê cột tường minh nên không thấy gì khác. Code nào viết `SELECT *` thì bỗng nhận thêm một cột; đó là lý do ứng dụng không nên phụ thuộc vào `SELECT *`. Tài xế 8007 là dòng **mới** nhưng chưa có biển số: backfill phải xử lý cả dòng sinh ra trong giai đoạn expand.

Thêm cột cho phép NULL chỉ sửa **metadata** (thông tin mô tả bảng trong catalog), không viết lại dòng nào. Để thấy rõ, tôi đo trên một bảng nháp 500.000 dòng (7.070 page) trong database `Scratch_ms009`, cùng container lab (SQL Server 2022 CU28 Developer, Docker Desktop trên Windows 11, máy 16 CPU, container giới hạn 2 GB RAM). Mỗi lệnh chạy trong transaction; trước khi commit, đọc lượng log đã ghi từ `sys.dm_tran_database_transactions` và khoá đang giữ trên bảng từ `sys.dm_tran_locks`. Chạy bốn lần; log của lệnh nặng gần như không đổi, thời gian dao động:

| Lệnh trên bảng 500.000 dòng | Log ghi | Thời gian (4 lần) | Khoá trên bảng |
|---|---|---|---|
| `ADD note varchar(10) NULL` | 1.028–6.612 byte | 4–12 ms | Sch-M |
| `ADD is_verified bit NOT NULL DEFAULT 0` | 2.072–2.324 byte | 0–4 ms | Sch-M |
| `ADD ref uniqueidentifier NOT NULL DEFAULT NEWID()` | ≈ 110,8 MB | 2,5–4,3 s | Sch-M |

Dòng thứ hai là hành vi có ghi trong [tài liệu `ALTER TABLE`](https://learn.microsoft.com/sql/t-sql/statements/alter-table-transact-sql): từ SQL Server 2012, bản **Enterprise** thêm cột `NOT NULL` có `DEFAULT` là hằng số lúc chạy (runtime constant) mà không viết lại dòng; giá trị mặc định nằm trong metadata. Developer Edition có đủ tính năng Enterprise nên lab thấy được điều đó. Tài liệu gắn hành vi này với Enterprise, nên trên Standard đừng giả định nó, hãy đo trên bản sao trước. `NEWID()` cho mỗi dòng một giá trị khác nên buộc viết lại cả bảng: 110 MB log cho 500.000 dòng.

Cột thứ tư đáng chú ý hơn: **mọi** lệnh `ALTER TABLE` ở đây đều giữ khoá **Sch-M** (schema modification). Theo tài liệu, Sch-M ngăn mọi phiên khác tham chiếu tới bảng, kể cả `SELECT`, cho tới khi transaction kết thúc. Lệnh chỉ sửa metadata giữ nó vài mili giây; lệnh viết lại bảng giữ nó suốt thời gian viết.

## 4. WITH NOCHECK: chặn dữ liệu mới, chưa kiểm dữ liệu cũ

Ràng buộc định dạng biển số (hư cấu, đơn giản hoá: xe máy `59X1-123.45`, ô tô `30A-123.45`). T-SQL không có regex nên dùng `LIKE` với lớp ký tự, chạy trên collation nhị phân để chỉ nhận chữ hoa (collation `Vietnamese_100_CI_AS` của database không phân biệt hoa thường):

```sql
ALTER TABLE toinoi.drivers WITH NOCHECK
ADD CONSTRAINT ck_drivers_vehicle_plate_format CHECK (
    vehicle_plate COLLATE Latin1_General_BIN2 LIKE '[1-9][0-9][A-Z][0-9]-[0-9][0-9][0-9].[0-9][0-9]'
 OR vehicle_plate COLLATE Latin1_General_BIN2 LIKE '[1-9][0-9][A-Z]-[0-9][0-9][0-9].[0-9][0-9]');
GO
DECLARE @t nvarchar(5) = (SELECT CONVERT(nvarchar(5), is_not_trusted) FROM sys.check_constraints
                          WHERE name = N'ck_drivers_vehicle_plate_format');
EXEC lab.expect_value N'E2 ck_drivers_vehicle_plate_format is_not_trusted', @t, N'1';
EXEC lab.expect_error N'E3 app mới ghi biển sai định dạng',
     N'UPDATE toinoi.drivers SET vehicle_plate = ''59x1-12345'' WHERE code = 8001', 547, N'ck_drivers_vehicle_plate_format';
```

```text
PASS E2 ck_drivers_vehicle_plate_format is_not_trusted = 1
PASS E3 app mới ghi biển sai định dạng — lỗi 547: The UPDATE statement conflicted with the CHECK constraint "ck_drivers_vehicle_plate_format". The conflict occurred in database "ToiNoi", table "toinoi.drivers", column 'vehicle_plate'.
```

`WITH NOCHECK` bỏ qua lần quét dữ liệu hiện có, nhưng ràng buộc **áp cho mọi dòng thêm mới hoặc bị sửa từ giờ** (E3). Vì chưa ai chứng minh dữ liệu cũ thoả, SQL Server đánh dấu nó là **ràng buộc không tin cậy** (untrusted constraint): cột `is_not_trusted = 1` trong `sys.check_constraints` (FK có cột cùng tên trong `sys.foreign_keys`). CHECK vẫn chấp nhận khi biểu thức ra UNKNOWN (đã gặp ở [MS-005](/articles/mssql-create-table-constraints)), nên app cũ ghi biển số NULL vẫn qua, đúng ý ở giai đoạn này.

Nếu không ghi gì, `ADD CONSTRAINT` mặc định là `WITH CHECK`: quét toàn bảng để chứng minh mọi dòng thoả, dưới khoá Sch-M. Trên bảng lớn, `WITH NOCHECK` lúc thêm rồi kiểm sau là cách tách lần quét đó khỏi bước expand.

## 5. Backfill, và ba trạng thái của một ràng buộc

Biển số lấy từ hồ sơ giấy. Lab giả lập một biển của dữ liệu cũ bị nhập sai (chữ thường) bằng cách tạm **tắt** ràng buộc, như dữ liệu nhập từ trước khi có nó:

```sql
ALTER TABLE toinoi.drivers NOCHECK CONSTRAINT ck_drivers_vehicle_plate_format;
UPDATE d SET vehicle_plate = v.plate
FROM toinoi.drivers AS d
JOIN (VALUES (8001, '59X1-123.45'), (8002, '59X2-234.56'), (8003, '29B1-345.67'),
             (8004, '30A-456.78'),  (8005, '43c1-567.89'), (8006, '43A-678.90'),
             (8007, '59X3-789.01')) AS v(code, plate) ON v.code = d.code;
EXEC lab.expect_value N'B1 backfill biển số', @@ROWCOUNT, N'7';
ALTER TABLE toinoi.drivers CHECK CONSTRAINT ck_drivers_vehicle_plate_format;
```

```text
PASS B1 backfill biển số = 7
```

`UPDATE … FROM … JOIN (VALUES …)` lấy giá trị từ một bảng hằng qua điều kiện nối. Ở đây có ba câu dễ nhầm. Tôi dựng lại cùng chuỗi lệnh trong `Scratch_ms009` và đọc catalog sau mỗi bước:

| Sau lệnh | `is_disabled` | `is_not_trusted` | Ý nghĩa |
|---|---|---|---|
| `WITH NOCHECK ADD CONSTRAINT` | 0 | 1 | Đang bật, chặn dữ liệu mới; dữ liệu cũ chưa kiểm |
| `NOCHECK CONSTRAINT` | 1 | 1 | **Tắt hẳn**: dữ liệu sai mới cũng lọt |
| `CHECK CONSTRAINT` | 0 | 1 | Bật lại, nhưng **vẫn không tin cậy** |
| `WITH CHECK CHECK CONSTRAINT` (thành công) | 0 | 0 | Đã kiểm mọi dòng: tin cậy |

Bẫy nằm ở dòng thứ ba. Tài liệu `ALTER TABLE` ghi: nếu không chỉ định, `WITH CHECK` là mặc định cho ràng buộc **mới**, còn `WITH NOCHECK` là mặc định cho ràng buộc **bật lại**. Vì vậy `CHECK CONSTRAINT` trơn không quét dòng nào. Cụm `WITH CHECK CHECK CONSTRAINT` đọc lặp nhưng không thừa: `WITH CHECK` nghĩa là "kiểm dữ liệu hiện có", `CHECK CONSTRAINT` nghĩa là "bật ràng buộc".

Lab chạy đúng câu đó, và nó thất bại vì biển `43c1-567.89` của tài xế 8005:

```text
PASS B2 validate khi còn dữ liệu sai — lỗi 547: The ALTER TABLE statement conflicted with the CHECK constraint "ck_drivers_vehicle_plate_format". The conflict occurred in database "ToiNoi", table "toinoi.drivers", column 'vehicle_plate'.
code|vehicle_plate
----|-------------
8005|43c1-567.89
```

Thông báo 547 **không nói dòng nào** sai; lab tự tìm bằng `SELECT … WHERE NOT (…)` lặp lại biểu thức của CHECK (result set ngay sau dòng PASS). Có một cách không phải chép biểu thức: `DBCC CHECKCONSTRAINTS` quét các ràng buộc đang bật của bảng và liệt kê từng vi phạm. Chạy trên bản dựng lại trong `Scratch_ms009`, ngay trước bước sửa:

```sql
DBCC CHECKCONSTRAINTS (N'toinoi.drivers');
```

```text
Table|Constraint|Where
-----|----------|-----
[toinoi].[drivers]|[ck_drivers_vehicle_plate_format]|[vehicle_plate] = '43c1-567.89'
DBCC execution completed. If DBCC printed error messages, contact your system administrator.
```

Tìm dòng vi phạm **trước** khi chạy `WITH CHECK CHECK` trên bảng lớn: một lần validate thất bại là một lần quét toàn bảng dưới Sch-M bị bỏ phí. Sửa dòng gốc rồi validate lại:

```sql
UPDATE toinoi.drivers SET vehicle_plate = UPPER(vehicle_plate) WHERE code = 8005;
ALTER TABLE toinoi.drivers WITH CHECK CHECK CONSTRAINT ck_drivers_vehicle_plate_format;
```

```text
PASS B3 sau khi sửa và validate: is_not_trusted = 0
```

Lưu ý: `DBCC CHECKCONSTRAINTS` chỉ báo cáo. Trong `Scratch_ms009`, sau khi sửa biển 8005 và chạy lại `DBCC CHECKCONSTRAINTS … WITH NO_INFOMSGS` không báo vi phạm nào, `is_not_trusted` vẫn là `1`. Chỉ `WITH CHECK CHECK CONSTRAINT` mới đổi cờ.

## 6. Vì sao "tin cậy" quan trọng

Ràng buộc không tin cậy vẫn chặn dữ liệu mới, vậy để nguyên có sao không? Có, vì **query optimizer** (bộ tối ưu truy vấn) dùng ràng buộc tin cậy như một sự thật về dữ liệu để bỏ bớt việc. Tài liệu `ALTER TABLE` viết rằng optimizer không xét các ràng buộc định nghĩa `WITH NOCHECK` cho tới khi chúng được bật lại bằng `WITH CHECK CHECK CONSTRAINT`. Tôi so sánh estimated plan (`SET SHOWPLAN_TEXT ON`) trên `Scratch_ms009`, trước và sau khi làm `ck_drivers_vehicle_type` và `fk_trips_customer` mất tin cậy bằng `NOCHECK CONSTRAINT` rồi `CHECK CONSTRAINT`:

```sql
SELECT code FROM toinoi.drivers WHERE vehicle_type IN ('truck', 'bus');
SELECT t.code, t.fare FROM toinoi.trips AS t JOIN toinoi.customers AS c ON c.id = t.customer_id;
```

Khi cả hai ràng buộc tin cậy:

```text
  |--Constant Scan
  |--Clustered Index Scan(OBJECT:([Scratch_ms009].[toinoi].[trips].[uq_trips_code] AS [t]))
```

Khi cả hai không tin cậy:

```text
  |--Clustered Index Scan(OBJECT:([Scratch_ms009].[toinoi].[drivers].[pk_drivers]), WHERE:([Scratch_ms009].[toinoi].[drivers].[vehicle_type]='bus' OR [Scratch_ms009].[toinoi].[drivers].[vehicle_type]='truck'))
  |--Nested Loops(Inner Join, OUTER REFERENCES:([t].[customer_id]))
       |--Clustered Index Scan(OBJECT:([Scratch_ms009].[toinoi].[trips].[uq_trips_code] AS [t]))
       |--Clustered Index Seek(OBJECT:([Scratch_ms009].[toinoi].[customers].[pk_customers] AS [c]), SEEK:([c].[id]=[Scratch_ms009].[toinoi].[trips].[customer_id] as [t].[customer_id]) ORDERED FORWARD)
```

Câu thứ nhất: CHECK tin cậy nói `vehicle_type` chỉ là `bike` hoặc `car`, nên optimizer biết điều kiện `IN ('truck', 'bus')` không thể đúng và thay cả truy vấn bằng `Constant Scan` (trả rỗng, không đọc bảng). Câu thứ hai: FK tin cậy trên cột `NOT NULL` bảo đảm mỗi chuyến có đúng một khách, mà truy vấn không lấy cột nào của `customers`, nên optimizer bỏ hẳn phép nối. Không tin cậy thì cả hai lợi ích biến mất.

Hai giới hạn của phép thử này: bảng chỉ có vài dòng, nên đây là bằng chứng về **hình dạng plan**, không phải về thời gian; và với điều kiện đơn `vehicle_type = 'truck'`, SQL Server tự tham số hoá câu lệnh (plan hiện `[@1]` thay cho hằng) nên không thấy `Constant Scan` ở cả hai trường hợp. Phần 3 sẽ đọc plan kỹ trong DBStudio. Điều cần nhớ ở đây: K8 trong `07-checks.sql` đếm mọi CHECK/FK `is_disabled = 1 OR is_not_trusted = 1` trong schema `toinoi` và mong đợi 0, để không ai bỏ quên một `WITH NOCHECK`.

## 7. Contract: ALTER COLUMN NOT NULL

Đây là bước **phá vỡ** app v1, chỉ chạy khi mọi phiên bản app đã gửi biển số và không còn dòng NULL:

```sql
DECLARE @nulls int = (SELECT COUNT(*) FROM toinoi.drivers WHERE vehicle_plate IS NULL);
EXEC lab.expect_value N'C1 còn tài xế thiếu biển', @nulls, N'0';
ALTER TABLE toinoi.drivers ALTER COLUMN vehicle_plate varchar(12) NOT NULL;
GO
EXEC lab.expect_error N'C2 app cũ (không gửi biển số) bị chặn sau contract',
     N'INSERT INTO toinoi.drivers (code, full_name, city_id, vehicle_type) VALUES (8008, N''Mai Thị Sen'', 2, ''bike'')', 515, N'vehicle_plate';
SELECT code, vehicle_type, vehicle_plate FROM toinoi.drivers ORDER BY code;
```

```text
PASS C1 còn tài xế thiếu biển = 0
PASS C2 app cũ (không gửi biển số) bị chặn sau contract — lỗi 515: Cannot insert the value NULL into column 'vehicle_plate', table 'ToiNoi.toinoi.drivers'; column does not allow nulls. INSERT fails.
code|vehicle_type|vehicle_plate
----|------------|-------------
8001|bike|59X1-123.45
8002|bike|59X2-234.56
8003|bike|29B1-345.67
8004|car|30A-456.78
8005|bike|43C1-567.89
8006|car|43A-678.90
8007|bike|59X3-789.01
```

Với 7 dòng thì nhanh. Trên bảng nháp 500.000 dòng ở mục 3 (đã điền đủ giá trị), cùng câu `ALTER COLUMN … NOT NULL` ghi khoảng **76 MB** log trong 1,4–3,1 giây (4 lần đo), suốt thời gian đó giữ Sch-M. Lệnh phải kiểm từng dòng, và số log cho thấy nó còn chạm vào từng dòng. Tôi thử thêm một CHECK `vehicle_plate IS NOT NULL` đã tin cậy trước khi siết, như mẹo của PostgreSQL: vẫn khoảng 74 MB log và 4,3 giây. SQL Server 2022 không dùng CHECK đó để bỏ qua lần quét. Với bảng lớn, bước contract cần cửa sổ bảo trì, hoặc chấp nhận để cột `NULL` và giữ quy tắc bằng CHECK tin cậy `vehicle_plate IS NOT NULL`. Lần validate CHECK đó vẫn quét toàn bảng dưới Sch-M (mục 8), nhưng chỉ đọc, không ghi log theo từng dòng.

## 8. Bảng chi phí: lệnh nào nặng

| Lệnh | Quét / viết lại dòng? | Ghi chú |
|---|---|---|
| `ADD` cột cho phép NULL | Không, chỉ metadata | Đo: ~1 KB log |
| `ADD` cột `NOT NULL DEFAULT` hằng số | Không (Enterprise/Developer, theo tài liệu) | Đo: ~2 KB log |
| `ADD` cột `NOT NULL DEFAULT NEWID()` | Viết lại mọi dòng | Đo: ~110 MB log / 500.000 dòng |
| `ADD CONSTRAINT … CHECK` (mặc định `WITH CHECK`) | Quét toàn bảng, chỉ đọc | Đo: ~1,2 s, 3 KB log; thất bại nếu có dòng sai |
| `WITH NOCHECK ADD CONSTRAINT` | Không | Đo: 0 ms; ràng buộc không tin cậy |
| `WITH CHECK CHECK CONSTRAINT` | Quét toàn bảng, chỉ đọc | Đo: ~1,1 s; chỉ cách này đổi `is_not_trusted` về 0 |
| `ALTER COLUMN … NOT NULL` | Quét và chạm từng dòng | Đo: ~76 MB log / 500.000 dòng |
| `ALTER COLUMN … NULL` | Không | Đo: 596 byte log |
| `DROP COLUMN` | Không | Tài liệu: không thu hồi dung lượng cho tới khi rebuild bảng |

Mọi lệnh tôi đo đều giữ Sch-M, kể cả hai lệnh validate; khác nhau ở **thời gian giữ**. Đây là khác biệt lớn với PostgreSQL (mục 11): ở SQL Server, lần quét để làm ràng buộc thành tin cậy cũng chặn cả `SELECT` của phiên khác, nên trên bảng lớn nó cần cửa sổ bảo trì như `ALTER COLUMN`. `WITH NOCHECK` chỉ dời lần quét đó tới lúc bạn chọn, không làm nó nhẹ đi. Khi một transaction dài đang đọc `drivers`, câu `ALTER` phải đứng chờ, và các truy vấn đến sau có thể xếp hàng sau nó (Phần 4 tái hiện bằng hai phiên). `SET LOCK_TIMEOUT 5000` trước migration giới hạn thời gian chờ khoá ở 5 giây: thà `ALTER` thất bại rồi thử lại còn hơn cả hệ thống đứng.

## 9. Kế hoạch hoàn tác hai mức

Migration nào cũng cần trả lời trước: nếu hỏng thì lùi thế nào? `06-migration-rollback.sql` có **hai mức**; chọn mức nhẹ nhất đủ giải quyết sự cố.

```sql
-- Mức 1
IF COLUMNPROPERTY(OBJECT_ID(N'toinoi.drivers'), N'vehicle_plate', 'AllowsNull') = 0
    ALTER TABLE toinoi.drivers ALTER COLUMN vehicle_plate varchar(12) NULL;
GO
-- Mức 2: sao lưu trước khi xoá (SELECT … INTO tạo bảng mới).
IF COL_LENGTH(N'toinoi.drivers', N'vehicle_plate') IS NOT NULL
   AND OBJECT_ID(N'lab.backup_drivers_vehicle_plate') IS NULL
BEGIN
    EXEC (N'SELECT id, code, vehicle_plate, SYSDATETIMEOFFSET() AS backed_up_at
            INTO lab.backup_drivers_vehicle_plate
            FROM toinoi.drivers WHERE vehicle_plate IS NOT NULL;');
END;
GO
ALTER TABLE toinoi.drivers DROP CONSTRAINT IF EXISTS ck_drivers_vehicle_plate_format;
IF COL_LENGTH(N'toinoi.drivers', N'vehicle_plate') IS NOT NULL
    ALTER TABLE toinoi.drivers DROP COLUMN vehicle_plate;
```

| Mức | Làm gì | Khi nào | Mất dữ liệu? |
|---|---|---|---|
| 1 · Nới lỏng | `ALTER COLUMN … NULL`, giữ cột và CHECK | App cũ lỗi vì chưa gửi biển số | Không; chỉ sửa metadata (596 byte log khi đo) |
| 2 · Gỡ hẳn | Sao lưu sang `lab.backup_drivers_vehicle_plate`, gỡ CHECK rồi `DROP COLUMN` | Bỏ hẳn tính năng | Không, nếu bước sao lưu chạy trước |

Mỗi bước kiểm trạng thái trước khi làm (`COLUMNPROPERTY`, `COL_LENGTH`, `OBJECT_ID`, `DROP CONSTRAINT IF EXISTS`), nên script **chạy lặp được**. Câu `SELECT … INTO` nằm trong `EXEC (N'…')` vì SQL Server biên dịch cả batch trước khi chạy: lần chạy thứ hai, cột `vehicle_plate` đã mất, và câu tham chiếu tới nó trực tiếp sẽ lỗi biên dịch dù nằm trong nhánh `IF` không được chạy. Rollback không nằm trong `npm run p01`; `npm test` chạy nó hai lần liên tiếp và đòi cả hai lần đều in:

```text
PASS R1 cột vehicle_plate = không
PASS R2 số biển số đã sao lưu = 7
```

Tôi chạy lại đúng hai lần trên `Scratch_ms009` sau `06` và nhận cùng hai dòng ở mỗi lần.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chạy rollback trên môi trường thật bằng DBStudio</span>
<p>Đặt "Môi trường" của kết nối production là "Sản xuất". Khi đó DBStudio bắt gõ đúng tên database trong hộp "Xác nhận câu lệnh phá huỷ" trước khi chạy câu <code>ALTER … DROP</code> hoặc <code>DROP</code>, đúng loại câu ở mức 2. Kết nối chỉ dùng để đọc catalog thì bật thêm công tắc "Chỉ đọc". Chạy rollback trong transaction thủ công ("Bắt đầu" trên thanh transaction thay cho "Tự động commit"), xem kết quả rồi mới "Commit" hoặc "Rollback".</p>
</div>

## 10. Thứ tự triển khai với app cũ và app mới

| Bước | Schema | App đang chạy | Lùi bằng |
|---|---|---|---|
| 1 | Expand: cột NULL + CHECK `WITH NOCHECK` | v1 | Rollback mức 2 (chưa có dữ liệu cần giữ) |
| 2 | (không đổi) | v1 + v2 lẫn nhau, rồi chỉ v2 | Quay lại v1; cột vẫn nhận NULL |
| 3 | Backfill, sửa dòng sai, `WITH CHECK CHECK` | v2 | Dữ liệu điền thêm vô hại với v1 |
| 4 | Kiểm `IS NULL` = 0, rồi contract `NOT NULL` | Chỉ v2 | Rollback mức 1 trước, rồi mới quay lại v1 |
| 5 (nếu bỏ tính năng) | Rollback mức 2 | Phiên bản không đọc ghi `vehicle_plate` | Khôi phục từ bảng sao lưu (bài tập 2) |

Hai luật suy ra từ bảng: contract chỉ chạy khi **không còn máy chủ nào chạy v1**; và trước khi gỡ cột ở mức 2, phải gỡ mọi phiên bản app còn đọc ghi `vehicle_plate`. Nếu không, chính rollback là thay đổi phá vỡ.

## 11. So với PostgreSQL

Cùng ý tưởng, khác cơ chế. PostgreSQL thêm ràng buộc bằng `NOT VALID` rồi `VALIDATE CONSTRAINT`; SQL Server dùng `WITH NOCHECK` rồi `WITH CHECK CHECK CONSTRAINT`, và trạng thái nằm ở `is_not_trusted` thay cho `convalidated`. Ở PostgreSQL, `VALIDATE` lấy khoá nhẹ cho phép đọc ghi song song; ở SQL Server, lần quét `WITH CHECK CHECK` giữ Sch-M suốt thời gian quét (đo ở mục 8). PostgreSQL 12+ dùng CHECK `IS NOT NULL` đã validate để `SET NOT NULL` khỏi quét bảng; phép đo ở mục 7 cho thấy SQL Server 2022 không làm vậy. Ngược lại, SQL Server có cờ "tắt hẳn" (`NOCHECK CONSTRAINT`), thứ PostgreSQL không có cho CHECK. Bài cùng chủ đề: [PG-009 · Thay đổi schema mà dữ liệu cũ vẫn đọc được](/articles/postgresql-schema-migration-tuong-thich).

## 12. Lỗi thường gặp

- **Nạp dữ liệu với `NOCHECK CONSTRAINT ALL` rồi bật lại bằng `CHECK CONSTRAINT ALL`.** Mọi ràng buộc bật lại đều không tin cậy (bài tập 1). Luôn kết thúc bằng `WITH CHECK CHECK CONSTRAINT`.
- **Tưởng `DBCC CHECKCONSTRAINTS` sạch là xong.** Nó không đổi cờ `is_not_trusted`.
- **Gộp expand và contract vào một lần triển khai.** Migration chạy lúc app mới khởi động, app cũ còn chạy thì lỗi 515 ngay.
- **Quên dòng sinh ra trong giai đoạn expand.** Tài xế 8007 đăng ký sau bước 1 cũng cần biển số; kiểm `IS NULL` ngay trước contract.
- **Không đặt tên ràng buộc.** Tên tự sinh kiểu `CK__drivers__vehic__…` khác nhau giữa các môi trường, script rollback không gọi được. K9 trong `07` bắt lỗi này.
- **Rollback chưa từng chạy thử**, hoặc chỉ chạy được một lần.

## 13. Bài tập

**Bài 1.** Đội dữ liệu nạp lại chuyến cũ vào `toinoi.trips` với `ALTER TABLE toinoi.trips NOCHECK CONSTRAINT ALL`, xong thì chạy `ALTER TABLE toinoi.trips CHECK CONSTRAINT ALL` và báo "đã bật lại ràng buộc". K8 sẽ báo gì? Sửa thế nào?

<details><summary><b>Lời giải</b></summary>
<p>Đã chạy trên bản sao schema lab trong <code>Scratch_ms009</code> (không nạp thêm dữ liệu): sau hai câu đó, số CHECK và FK của <code>trips</code> có <code>is_not_trusted = 1</code> là <b>14</b> (10 CHECK + 4 FK), dù cờ <code>is_disabled</code> đều về 0. K8 sẽ FAIL. Sửa: <code>ALTER TABLE toinoi.trips WITH CHECK CHECK CONSTRAINT ALL;</code> quét lại và đưa số đó về <b>0</b>. Nếu có dòng sai, câu này lỗi 547 ở ràng buộc đầu tiên gặp; chạy <code>DBCC CHECKCONSTRAINTS (N'toinoi.trips')</code> trước để có danh sách đầy đủ.</p>
</details>

**Bài 2.** Sau sự cố, đội vận hành đã chạy rollback **mức 2** (cột bị gỡ). Hôm sau quyết định bật lại tính năng. Viết SQL đưa cột và biển số quay lại từ bản sao lưu, rồi nêu các bước còn lại.

<details><summary><b>Lời giải</b></summary>
<p>Đã chạy trên <code>Scratch_ms009</code> sau <code>06</code> và hai lần rollback:</p>
<pre><code class="language-sql">ALTER TABLE toinoi.drivers ADD vehicle_plate varchar(12) NULL;
GO
UPDATE d SET vehicle_plate = b.vehicle_plate
FROM toinoi.drivers AS d
JOIN lab.backup_drivers_vehicle_plate AS b ON b.id = d.id;</code></pre>
<p>Kết quả 7 dòng, không tài xế nào còn NULL; 8005 có <code>43C1-567.89</code> (bản đã sửa, vì sao lưu lấy dữ liệu sau migration). <code>GO</code> là bắt buộc: không có nó, SQL Server biên dịch câu <code>UPDATE</code> khi cột chưa tồn tại và báo lỗi. Tiếp theo: kiểm <code>WHERE vehicle_plate IS NULL</code> (tài xế đăng ký trong lúc cột bị gỡ cần nguồn khác), thêm lại CHECK <code>WITH NOCHECK</code> rồi <code>WITH CHECK CHECK CONSTRAINT</code>, cuối cùng contract. Không có bản sao lưu thì dữ liệu đã mất cùng <code>DROP COLUMN</code>; đó là lý do mức 1 là mặc định.</p>
</details>

**Bài 3.** Tới Nơi muốn đổi tên `drivers.full_name` thành `display_name`. Lập kế hoạch expand–contract (không dùng `sp_rename` một phát), ghi rõ phiên bản app nào chạy được ở mỗi bước.

<details><summary><b>Lời giải</b></summary>
<p>(1) Expand: thêm <code>display_name nvarchar(100) NULL</code>; v1 không ảnh hưởng. (2) v2 ghi <b>cả hai</b> cột, vẫn đọc <code>full_name</code>. (3) Backfill theo lô, điều kiện <code>display_name IS NULL</code>. (4) Thêm <code>ck_drivers_display_name_not_blank</code> <code>WITH NOCHECK</code> rồi <code>WITH CHECK CHECK CONSTRAINT</code>; kiểm NULL = 0 rồi <code>ALTER COLUMN … NOT NULL</code> trong cửa sổ bảo trì. (5) v3 đọc <code>display_name</code>, vẫn ghi cả hai. (6) Hết v1/v2: v4 ngừng ghi <code>full_name</code>; nới <code>full_name</code> thành NULL (chỉ metadata). (7) Contract: sao lưu nếu cần, gỡ <code>ck_drivers_full_name_not_blank</code> rồi <code>DROP COLUMN full_name</code>. Bước 1–6 lùi được bằng cách quay về app trước; chỉ bước 7 mất dữ liệu. <code>sp_rename</code> một phát làm mọi app cũ hỏng ngay.</p>
</details>

## 14. Đọc tiếp

- [Microsoft Learn · ALTER TABLE (Transact-SQL)](https://learn.microsoft.com/sql/t-sql/statements/alter-table-transact-sql): `WITH CHECK | WITH NOCHECK`, mặc định khi bật lại ràng buộc, thêm cột `NOT NULL` có `DEFAULT`, khoá Sch-M, `DROP COLUMN`.
- [Microsoft Learn · DBCC CHECKCONSTRAINTS](https://learn.microsoft.com/sql/t-sql/database-console-commands/dbcc-checkconstraints-transact-sql): kiểm một ràng buộc, một bảng, hoặc cả ràng buộc đang tắt với `ALL_CONSTRAINTS`.
- [Microsoft Learn · sys.check_constraints](https://learn.microsoft.com/sql/relational-databases/system-catalog-views/sys-check-constraints-transact-sql) và [sys.foreign_keys](https://learn.microsoft.com/sql/relational-databases/system-catalog-views/sys-foreign-keys-transact-sql): cột `is_disabled`, `is_not_trusted`.
- [Microsoft Learn · SET LOCK_TIMEOUT](https://learn.microsoft.com/sql/t-sql/statements/set-lock-timeout-transact-sql).
- Lab: `examples/toi-noi-mssql/p01/06-migration.sql`, `06-migration-rollback.sql`, kết quả thật trong `p01/OUTPUT.txt`.

Bài tiếp theo: [MS-010 · Thực hành: schema chuyến xe có kiểm tra ràng buộc](/articles/mssql-thuc-hanh-schema-chuyen-xe).
