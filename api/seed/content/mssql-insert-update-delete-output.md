---
slug: mssql-insert-update-delete-output
title: "INSERT, UPDATE, DELETE và mệnh đề OUTPUT: một chuyến Tới Nơi trọn vòng đời trên SQL Server"
summary: "Khách đặt xong phải thấy ngay mã chuyến, tài xế bấm nhận trễ phải biết mình thua, mỗi lần đổi trạng thái phải có một dòng lịch sử. Bài dùng INSERT, UPDATE, DELETE với OUTPUT inserted/deleted, biến bảng và transaction để đưa một chuyến qua 7 trạng thái, giải thích vì sao mã chuyến là 100032 và vì sao OUTPUT INTO thẳng bảng lịch sử bị từ chối."
category: mssql
level: intermediate
tags: [mssql, ms-006, sql-server-2022, dml, insert, update, delete, output, identity, toi-noi]
language: vi
---

09:00 ngày 03/11, chị Phạm Thu Dung đặt một chuyến xe máy ở Hà Nội, cước 42.000 đ. Ngay khi bấm "Đặt", app phải hiện **mã chuyến**: con số do database cấp, ứng dụng không tự biết. Một phút sau, tài xế 8003 bấm "Nhận"; hai giây sau nữa, tài xế 8001 cũng bấm "Nhận" cùng chuyến. App của tài xế 8001 phải báo "chuyến đã có người nhận", không được lặng lẽ thay tài xế 8003.

Đội backend còn một yêu cầu thứ ba: mỗi lần chuyến đổi trạng thái phải có đúng một dòng trong bảng lịch sử `trip_events`, không thiếu, không thừa. Người từng dùng PostgreSQL sẽ nghĩ ngay tới `UPDATE … RETURNING` kết hợp `INSERT`. Trên SQL Server, cách viết gần nhất là `UPDATE … OUTPUT … INTO toinoi.trip_events`, và SQL Server từ chối nó với lỗi 333.

Bài này đưa một chuyến đi hết vòng đời bằng `INSERT`, `UPDATE`, `DELETE` kèm mệnh đề `OUTPUT`, đọc số dòng bị ảnh hưởng để biết ai thắng, và ghép thay đổi của chuyến với dòng lịch sử trong cùng một transaction.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-006 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/mssql-create-table-constraints">MS-005 · CREATE TABLE, ràng buộc và IDENTITY</a>. Lab: <code>examples/toi-noi-mssql/p01</code> (tệp <code>04-lifecycle.sql</code>). Sau bài này, bạn đưa được một chuyến qua đủ vòng đời bằng INSERT/UPDATE/DELETE … OUTPUT, ghi lịch sử trong cùng transaction và biết một UPDATE ảnh hưởng 0 dòng nói lên điều gì.</p>
</div>

## 1. Database trả lời gì sau một câu ghi?

`INSERT`, `UPDATE`, `DELETE` thuộc nhóm **DML** (Data Manipulation Language, câu lệnh thao tác dữ liệu). Mỗi câu chạy xong, SQL Server cho ứng dụng biết hai thứ:

- **Số dòng bị ảnh hưởng.** Trong T-SQL, giá trị này nằm ở hàm hệ thống `@@ROWCOUNT` ngay sau câu lệnh. Công cụ in nó ra thành thông điệp: `sqlcmd` in `(1 rows affected)`, DBStudio hiện "Ảnh hưởng 1 dòng". Driver của ứng dụng (ADO.NET `ExecuteNonQuery`, JDBC `executeUpdate`…) trả nó về dưới dạng số nguyên. Mọi tệp của lab mở đầu bằng `SET NOCOUNT ON`: lệnh này chỉ tắt thông điệp "rows affected" cho gọn kết quả, còn `@@ROWCOUNT` vẫn được cập nhật bình thường.
- **Kết quả của mệnh đề `OUTPUT`** (nếu có): một tập dòng giống kết quả của SELECT, tính trên đúng những dòng câu lệnh vừa ghi.

`OUTPUT` đọc từ hai bảng ảo mà SQL Server dựng cho câu lệnh đang chạy: **`inserted`** chứa dòng ở trạng thái *sau* khi ghi, **`deleted`** chứa dòng ở trạng thái *trước* khi ghi.

| Câu lệnh | `inserted` | `deleted` | Ghi chú |
|---|---|---|---|
| `INSERT` | Dòng vừa thêm, gồm cả giá trị database tự điền (`DEFAULT`, IDENTITY) | Không có | Thay cho "INSERT rồi SELECT lại" |
| `UPDATE` | Giá trị **mới** | Giá trị **cũ** | Một dòng OUTPUT thấy được cả trước và sau |
| `DELETE` | Không có | Nội dung dòng **vừa bị xoá** | Bản ghi cuối cùng trước khi dòng biến mất |

`OUTPUT` có hai dạng. `OUTPUT <danh sách cột>` trả kết quả thẳng về client. `OUTPUT <danh sách cột> INTO <bảng hoặc biến bảng>` ghi kết quả vào một bảng. Một câu lệnh được phép có cả hai, như ở bước đặt chuyến dưới đây.

## 2. Chạy lab

Lab P01 dựng database `ToiNoi` (collation `Vietnamese_100_CI_AS`) trên SQL Server 2022 trong Docker. Cách dựng container và kết nối DBStudio nằm ở [MS-002](/articles/mssql-cai-dat-docker-dbstudio). Chuỗi tệp cho bài này:

```bash
cd examples/toi-noi-mssql
npm run db:up
node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql p01/03-fixture-invalid.sql p01/04-lifecycle.sql
```

`npm run p01` chạy đủ 8 tệp và in kết quả giống `p01/OUTPUT.txt`. Mọi kết quả trong bài chép từ tệp đó (SQL Server 2022 16.0.4325.4, RTM-CU28) hoặc từ câu thử thêm mà bài ghi rõ là đã chạy. `00-reset.sql` xoá database `ToiNoi` cũ rồi tạo lại, nên chạy lại bao nhiêu lần cũng ra cùng mã chuyến và cùng mốc thời gian: GUID và thời điểm trong fixture đều ghi sẵn, ứng dụng thật mới dùng `NEWID()` và `SYSDATETIMEOFFSET()`.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỉ chạy trên database lab</span>
<p><code>00-reset.sql</code> đặt database về <code>SINGLE_USER WITH ROLLBACK IMMEDIATE</code> trước khi xoá: mọi phiên đang dùng <code>ToiNoi</code>, kể cả tab DBStudio đang mở transaction, bị cắt ngang. Đừng chạy nó trên server có dữ liệu thật.</p>
</div>

## 3. INSERT … OUTPUT: khách đặt chuyến

Câu đặt chuyến chỉ ghi những gì ứng dụng đã biết. `status` lấy `DEFAULT 'requested'`, `driver_id` và `accepted_at` để NULL, `code` do IDENTITY cấp. Bước 1 của `04-lifecycle.sql`:

```sql
DECLARE @moi TABLE (code bigint);
BEGIN TRANSACTION;
INSERT INTO toinoi.trips (id, kind, city_id, customer_id, fare, requested_at, updated_at)
OUTPUT inserted.code INTO @moi (code)
OUTPUT inserted.code, inserted.status, inserted.fare, inserted.requested_at
VALUES ('71000000-0000-4000-8000-000000000001', 'ride', 2,
        (SELECT id FROM toinoi.customers WHERE phone = '0900000004'),
        42000, '2026-11-03 09:00:00 +07:00', '2026-11-03 09:00:00 +07:00');
INSERT INTO toinoi.trip_events (trip_id, status, occurred_at)
VALUES ('71000000-0000-4000-8000-000000000001', 'requested', '2026-11-03 09:00:00 +07:00');
COMMIT TRANSACTION;
```

```text
code|status|fare|requested_at
----|------|----|------------
100032|requested|42000|2026-11-03 09:00:00.000 +07:00
PASS mã chuyến mới (100014 + 17 giá trị bị tiêu ở 03 + 1) = 100032
```

Hai mệnh đề `OUTPUT` làm hai việc khác nhau. Mệnh đề thứ nhất cất mã chuyến vào **biến bảng** `@moi`: một biến kiểu bảng chỉ sống trong batch hiện tại, đủ để script dùng lại mã ở câu sau. Mệnh đề thứ hai trả về app đúng những gì app cần hiển thị: mã 100032 và trạng thái `requested` do `DEFAULT` điền. App không phải đoán, cũng không phải gửi thêm câu SELECT.

`BEGIN TRANSACTION … COMMIT TRANSACTION` gói hai câu INSERT thành một đơn vị: chuyến và dòng lịch sử đầu tiên cùng được ghi hoặc cùng không. Mục 4 giải thích vì sao lab phải bật thêm `SET XACT_ABORT ON` để điều đó thật sự đúng.

### Vì sao 100032 chứ không phải 100015?

Fixture hợp lệ có 15 chuyến, mã 100000 đến 100014 (`PASS IDENT_CURRENT(trips) = 100014` trong phần `02`). Sau đó, tệp `03-fixture-invalid.sql` của [MS-005](/articles/mssql-create-table-constraints) chạy 34 câu sai, mỗi câu trong một transaction riêng rồi ROLLBACK. Một số câu INSERT vào `trips` đã kịp xin số từ IDENTITY trước khi bị từ chối, và **SQL Server không trả lại số đã cấp**, kể cả khi transaction bị huỷ.

Để biết chính xác câu nào tiêu số, tôi chạy `00 → 03` trên một bản sao database (`Scratch_ms006`) và in `IDENT_CURRENT(N'toinoi.trips')` trước mỗi câu của `03`:

| Câu trong `03` | Lỗi | Tiêu số IDENTITY? |
|---|---|---|
| N2 thiếu cước, N3 `status` NULL tường minh | 515 (NOT NULL) | Có, 2 số |
| F1 khách không tồn tại | 547 (FOREIGN KEY) | Có, 1 số |
| C8 … C17 vi phạm CHECK của `trips` | 547 (CHECK) | Có, 10 số |
| T3 cước vượt `int`, T4 `'45.000đ'`, T5 ngày 31/11, T6 GUID sai | 8115, 245, 241, 8169 (chuyển kiểu) | Có, 4 số |
| I1 tự ghi giá trị vào cột `code` | 544 (IDENTITY_INSERT) | Không: câu bị từ chối trước khi chạy |
| Các câu trên bảng khác (`cities`, `customers`, `drivers`) | | Không liên quan tới `trips` |

Tổng cộng 2 + 1 + 10 + 4 = **17** giá trị, từ 100015 tới 100031. Chuyến của chị Dung nhận số kế tiếp: 100032. Chứng minh ngược: chạy `00 → 01 → 02 → 04` trên bản sao, bỏ `03`, chuyến mới nhận **100015** (đã chạy).

Một chi tiết đáng nhớ khi so với PostgreSQL: ở lab PG, câu lỗi chuyển kiểu (`'45.000đ'`, ngày không tồn tại) không tiêu số sequence. Trên SQL Server 2022, bốn câu tương ứng (T3–T6) vẫn tiêu số. Kết luận chung cho cả hai engine không đổi: **mã IDENTITY dùng để tra cứu** ("cho tôi xem chuyến 100032"), không dùng để đếm số chuyến hay kiểm tra "có mất chuyến nào không". Tài liệu thuộc tính IDENTITY của Microsoft nói rõ giá trị không được tái sử dụng khi transaction bị rollback và dãy số có thể có khoảng trống.

### SCOPE_IDENTITY hay OUTPUT?

Nhiều code .NET cũ lấy mã vừa sinh bằng `SELECT SCOPE_IDENTITY()`. SQL Server có ba hàm dễ nhầm:

- `SCOPE_IDENTITY()`: giá trị IDENTITY **cuối cùng** sinh ra trong phiên hiện tại, trong phạm vi hiện tại (không tính trigger).
- `@@IDENTITY`: giá trị cuối cùng trong phiên, **kể cả** giá trị do trigger sinh ở bảng khác. Thêm một trigger ghi log là kết quả có thể đổi.
- `IDENT_CURRENT(N'bảng')`: giá trị cuối cùng của bảng, **của mọi phiên**. Chỉ dùng để quan sát như lab, không dùng để lấy mã cho dòng mình vừa thêm.

`SCOPE_IDENTITY()` chỉ trả một giá trị. Thử INSERT hai chuyến trong một câu, trong transaction rồi ROLLBACK, trên bản sao database sau `04`:

```sql
DECLARE @m TABLE (id uniqueidentifier, code bigint);
BEGIN TRAN;
INSERT INTO toinoi.trips (kind, city_id, customer_id, fare, requested_at, updated_at)
OUTPUT inserted.id, inserted.code INTO @m
VALUES ('ride', 1, 'c0000000-0000-4000-8000-000000000001', 30000, '2026-11-03 10:00 +07:00', '2026-11-03 10:00 +07:00'),
       ('ride', 1, 'c0000000-0000-4000-8000-000000000002', 31000, '2026-11-03 10:01 +07:00', '2026-11-03 10:01 +07:00');
SELECT CAST(SCOPE_IDENTITY() AS bigint) AS scope_identity, CAST(IDENT_CURRENT('toinoi.trips') AS bigint) AS ident_current;
SELECT code FROM @m ORDER BY code;
ROLLBACK;
SELECT CAST(IDENT_CURRENT('toinoi.trips') AS bigint) AS ident_current_sau_rollback, (SELECT COUNT(*) FROM @m) AS dong_trong_bien_bang;
```

```text
scope_identity|ident_current
--------------|-------------
100035|100035
code
----
100034
100035
ident_current_sau_rollback|dong_trong_bien_bang
--------------------------|--------------------
100035|2
```

`SCOPE_IDENTITY()` chỉ cho biết 100035; `OUTPUT` cho cả hai mã, kèm `id` để ghép đúng mã với đúng dòng. Kết quả cuối còn cho thấy hai điều: ROLLBACK không trả lại 100034 và 100035, và **biến bảng không bị ROLLBACK** (vẫn còn 2 dòng). Tính chất thứ hai hữu ích khi muốn giữ lại thông tin để ghi log sau khi transaction đã huỷ.

Kết luận: với code mới, dùng `OUTPUT inserted.<cột>` để lấy giá trị database sinh ra. `SCOPE_IDENTITY()` vẫn đúng cho INSERT một dòng, nhưng chỉ biết IDENTITY, không biết `DEFAULT` hay giá trị khác.

## 4. OUTPUT INTO bảng lịch sử: vì sao bị từ chối

Tài xế chưa nhận, hệ thống chuyển chuyến sang `searching`. Cách viết "tự nhiên" là đổi trạng thái và ghi lịch sử bằng một câu:

```sql
UPDATE toinoi.trips SET status = 'searching'
OUTPUT inserted.id, inserted.status, inserted.updated_at INTO toinoi.trip_events (trip_id, status, occurred_at)
WHERE id = '71000000-0000-4000-8000-000000000001';
```

```text
PASS O1 OUTPUT INTO bảng có CHECK/khoá ngoại — lỗi 333: The target table 'toinoi.trip_events' of the OUTPUT INTO clause cannot have any enabled check constraints or any enabled rules. Found check constraint or rule 'ck_trip_events_status'.
```

Tài liệu mệnh đề OUTPUT liệt kê các điều kiện của bảng đích trong `OUTPUT INTO`: không được có trigger đang bật, không được có CHECK constraint đang bật, và **không được nằm ở bất kỳ đầu nào của một khoá ngoại**. `trip_events` vi phạm cả hai: có `ck_trip_events_status` và có `fk_trip_events_trip` trỏ tới `trips`. SQL Server báo lỗi về CHECK trước (333). Bỏ CHECK cũng chưa đủ. Trên bản sao database, tôi tạo một bảng chỉ có khoá ngoại tới `trips` và thử lại:

```text
Msg 332, Level 16, State 1
The target table 'dbo.ev_fk' of the OUTPUT INTO clause cannot be on either side of a (primary key, foreign key) relationship. Found reference constraint 'FK__ev_fk__trip_id__6D0D32F4'.
```

Gỡ ràng buộc khỏi bảng lịch sử để chiều theo cú pháp là đổi nhầm: CHECK và FK của [MS-005](/articles/mssql-create-table-constraints) mới là thứ giữ dữ liệu đúng. T-SQL còn một cách viết khác, lồng câu UPDATE vào `INSERT … SELECT … FROM (UPDATE … OUTPUT …) AS x` (composable DML); cách này cũng bị từ chối với lỗi 356 vì cùng lý do khoá ngoại (đã chạy trên bản sao).

### Mẫu đúng: OUTPUT INTO biến bảng, rồi INSERT, trong một transaction

Bước 2b của lab:

```sql
SET XACT_ABORT ON;   -- đặt ở đầu tệp: lỗi giữa transaction ⇒ huỷ cả transaction

DECLARE @ev TABLE (trip_id uniqueidentifier, status varchar(12), occurred_at datetimeoffset(3));
BEGIN TRANSACTION;
UPDATE toinoi.trips
SET status = 'searching', updated_at = '2026-11-03 09:00:05 +07:00'
OUTPUT inserted.id, inserted.status, inserted.updated_at INTO @ev
WHERE id = '71000000-0000-4000-8000-000000000001' AND status = 'requested';
INSERT INTO toinoi.trip_events (trip_id, status, occurred_at) SELECT trip_id, status, occurred_at FROM @ev;
COMMIT TRANSACTION;
```

```text
PASS requested → searching = 1
```

Biến bảng không có ràng buộc nào nên `OUTPUT INTO @ev` luôn hợp lệ. Câu INSERT thứ hai ghi vào `trip_events` theo đường bình thường, mọi CHECK và FK vẫn được kiểm. Nếu UPDATE không khớp dòng nào, `@ev` rỗng và INSERT thêm 0 dòng: không thể có dòng lịch sử cho một lần đổi trạng thái không xảy ra.

### Vì sao cần SET XACT_ABORT ON?

Mặc định (`XACT_ABORT OFF`), phần lớn lỗi runtime như vi phạm ràng buộc chỉ huỷ **câu lệnh** bị lỗi; transaction vẫn mở và các câu trước đó vẫn còn hiệu lực. Thử trên bản sao database: mở transaction, INSERT một dòng lịch sử hợp lệ, rồi một dòng gõ sai trạng thái (`'acepted'`):

```text
-- XACT_ABORT OFF
Msg 547 … conflicted with the CHECK constraint "ck_trip_events_status" …
The statement has been terminated.
OFF: trancount=1 so_dong=1

-- XACT_ABORT ON
Msg 547 … conflicted with the CHECK constraint "ck_trip_events_status" …
ON: trancount=0 so_dong=0
```

Với `OFF`, transaction vẫn mở (`@@TRANCOUNT = 1`) và dòng hợp lệ đầu tiên vẫn nằm trong đó. Script chạy tiếp tới `COMMIT` thì dòng ấy được ghi, dù câu thứ hai đã lỗi. Với `ON`, SQL Server huỷ cả transaction và dừng batch: không còn gì để commit nhầm. Đó là lý do mọi transaction nhiều câu của lab nằm sau `SET XACT_ABORT ON`. Bắt lỗi bằng `TRY…CATCH` và đọc `XACT_STATE()` là chủ đề của MS-031 ở Phần 4.

## 5. UPDATE có điều kiện: tài xế 8003 nhận chuyến

Mỗi bước của vòng đời là một UPDATE với điều kiện `WHERE` gồm hai phần: **dòng nào** (`id = …`) và **đang ở trạng thái nào** (`status = <trạng thái trước>`). Bước nhận chuyến còn kiểm `driver_id IS NULL`:

```sql
DECLARE @ev TABLE (trip_id uniqueidentifier, status varchar(12), occurred_at datetimeoffset(3));
BEGIN TRANSACTION;
UPDATE toinoi.trips
SET status = 'accepted',
    driver_id = (SELECT id FROM toinoi.drivers WHERE code = 8003),
    accepted_at = '2026-11-03 09:01:00 +07:00',
    updated_at  = '2026-11-03 09:01:00 +07:00'
OUTPUT inserted.id, inserted.status, inserted.updated_at INTO @ev
OUTPUT deleted.status AS truoc, inserted.status AS sau, inserted.accepted_at
WHERE id = '71000000-0000-4000-8000-000000000001' AND status = 'searching' AND driver_id IS NULL;
INSERT INTO toinoi.trip_events (trip_id, status, occurred_at) SELECT trip_id, status, occurred_at FROM @ev;
UPDATE toinoi.drivers SET status = 'busy' WHERE code = 8003 AND status = 'online';
DECLARE @busy int = @@ROWCOUNT;
COMMIT TRANSACTION;
```

```text
truoc|sau|accepted_at
-----|---|-----------
searching|accepted|2026-11-03 09:01:00.000 +07:00
PASS searching → accepted = 1
PASS tài xế 8003 online → busy = 1
```

`deleted.status` và `inserted.status` nằm trên cùng một dòng kết quả: trạng thái trước là `searching`, sau là `accepted`. Ở PostgreSQL 17, `RETURNING` của UPDATE chỉ thấy giá trị mới; trên SQL Server, `OUTPUT` cho cả hai mà không cần đọc trước.

Tài xế cũng chuyển `online → busy` bằng chính mẫu có điều kiện, và lab cất `@@ROWCOUNT` vào biến `@busy` **ngay sau** câu UPDATE. Mục sau giải thích vì sao phải làm vậy.

## 6. Tài xế 8001 bấm nhận trễ: 0 dòng, không lỗi

Hai giây sau, app của tài xế 8001 gửi đúng câu nhận chuyến, chỉ khác tài xế và thời điểm:

```sql
UPDATE toinoi.trips
SET status = 'accepted',
    driver_id = (SELECT id FROM toinoi.drivers WHERE code = 8001),
    accepted_at = '2026-11-03 09:01:02 +07:00',
    updated_at  = '2026-11-03 09:01:02 +07:00'
WHERE id = '71000000-0000-4000-8000-000000000001' AND status = 'searching' AND driver_id IS NULL;
EXEC lab.expect_value N'tài xế 8001 nhận trễ: số dòng đổi', @@ROWCOUNT, N'0';
```

```text
PASS tài xế 8001 nhận trễ: số dòng đổi = 0
```

Không có lỗi nào. Chuyến đã `accepted` nên điều kiện `status = 'searching'` sai, không dòng nào khớp `WHERE`, và SQL Server coi đó là một câu UPDATE thành công ảnh hưởng 0 dòng. Thêm `OUTPUT inserted.code, inserted.status` vào câu trên (đã chạy trên bản sao) thì client nhận một result set **chỉ có tiêu đề, không có dòng**; DBStudio hiện "Ảnh hưởng 0 dòng". Code ứng dụng chỉ bắt exception sẽ tưởng tài xế 8001 nhận chuyến thành công. App phải đọc số dòng bị ảnh hưởng (hoặc thấy OUTPUT rỗng) để báo "chuyến đã có người nhận".

### @@ROWCOUNT phải đọc ngay

`@@ROWCOUNT` phản ánh câu lệnh **vừa chạy xong**, và rất nhiều câu lệnh đặt lại nó, kể cả câu điều khiển. Thử trên bản sao database với một UPDATE khớp đúng 1 dòng:

```sql
UPDATE toinoi.trips SET updated_at = updated_at WHERE code = 100012;
IF @@ROWCOUNT = 1 PRINT CONCAT(N'IF thấy 1, rồi @@ROWCOUNT = ', @@ROWCOUNT);
```

```text
IF thấy 1, rồi @@ROWCOUNT = 0
```

Điều kiện `IF` đọc đúng 1, nhưng tới câu `PRINT` bên trong thì `@@ROWCOUNT` đã về 0. Quy tắc an toàn: cất vào biến ngay câu sau, `DECLARE @rc int = @@ROWCOUNT;`, rồi chỉ dùng biến. Với `OUTPUT … INTO @ev`, lab đếm `SELECT COUNT(*) FROM @ev` nên không phụ thuộc thứ tự câu lệnh.

**0 dòng có hai nguyên nhân** khác hẳn nhau: không có chuyến mang `id` đó, hoặc chuyến có nhưng đang ở trạng thái khác. Muốn giải thích cho người dùng, đọc lại theo `id`: không có dòng nghĩa là `id` sai; có dòng thì xem `status`. Câu đọc này chỉ để giải thích, quyết định ghi đã nằm trong điều kiện `WHERE`.

### Nếu bỏ điều kiện trạng thái

Cùng câu nhận chuyến nhưng chỉ có `WHERE code = 100012`. Chuyến 100012 của fixture đang `accepted` do tài xế 8001 nhận. Thử trong `BEGIN TRAN … ROLLBACK` trên bản sao database:

```sql
BEGIN TRAN;
UPDATE toinoi.trips
SET status = 'accepted', driver_id = (SELECT id FROM toinoi.drivers WHERE code = 8003),
    accepted_at = '2026-11-03 09:45:00 +07:00', updated_at = '2026-11-03 09:45:00 +07:00'
OUTPUT inserted.code, deleted.driver_id AS tai_xe_cu, inserted.driver_id AS tai_xe_moi
WHERE code = 100012;
ROLLBACK;
```

```text
code|tai_xe_cu|tai_xe_moi
----|---------|----------
100012|D0000000-0000-4000-8000-000000008001|D0000000-0000-4000-8000-000000008003
```

Câu lệnh thành công, không ràng buộc nào phản đối vì dòng mới vẫn hợp lệ theo mọi CHECK. Tài xế 8001 đang chạy chuyến thì bị thay bằng 8003. Đây là **lost update** (cập nhật bị mất): một lần ghi đè lên kết quả của lần ghi trước mà không ai hay. Ràng buộc chỉ kiểm từng dòng có hợp lệ không; việc chuyến đi đúng **bước** là trách nhiệm của điều kiện trong `WHERE`. (SQL Server hiển thị `uniqueidentifier` bằng chữ in hoa, nên GUID trong kết quả bắt đầu bằng `D0000000`.)

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Lab chạy một kết nối, các câu đi tuần tự: tài xế 8001 bấm "sau 2 giây" chứ không cùng khoảnh khắc. Hai phiên chạy song song thật sự, khoá dòng và isolation level là việc của Phần 4, với bài thực hành <a href="/articles/mssql-thuc-hanh-tranh-chap-nhan-chuyen">MS-040 · hai tài xế cùng nhận một chuyến</a>.</p>
</div>

## 7. Bốn bước còn lại và lịch sử bảy trạng thái

Các bước `arriving`, `picked_up`, `in_transit`, `completed` theo đúng một mẫu, nên lab gom chúng vào một vòng lặp: bảng `@steps` liệt kê "từ trạng thái nào, tới trạng thái nào, lúc mấy giờ".

```sql
WHILE @n <= 4
BEGIN
    SELECT @tu = tu, @toi = toi, @luc = luc FROM @steps WHERE n = @n;
    DELETE FROM @ev;
    BEGIN TRANSACTION;
    UPDATE toinoi.trips
    SET status = @toi,
        completed_at = IIF(@toi = 'completed', @luc, completed_at),
        updated_at = @luc
    OUTPUT inserted.id, inserted.status, inserted.updated_at INTO @ev
    WHERE id = '71000000-0000-4000-8000-000000000001' AND status = @tu;
    INSERT INTO toinoi.trip_events (trip_id, status, occurred_at) SELECT trip_id, status, occurred_at FROM @ev;
    COMMIT TRANSACTION;
    -- … kiểm số dòng trong @ev bằng lab.expect_value
    SET @n += 1;
END;
```

`IIF(@toi = 'completed', @luc, completed_at)` chỉ ghi `completed_at` ở bước cuối. Quên nó thì ràng buộc của MS-005 chặn ngay. Thử trên bản sao database với chuyến 100010 đang `in_transit`:

```sql
UPDATE toinoi.trips SET status = 'completed', updated_at = '2026-11-03 09:31:00 +07:00'
WHERE code = 100010 AND status = 'in_transit';
```

```text
Msg 547, Level 16, State 1
The UPDATE statement conflicted with the CHECK constraint "ck_trips_completed_at_by_status". The conflict occurred in database "Scratch_ms006", table "toinoi.trips".
The statement has been terminated.
```

Kết quả của vòng lặp và lịch sử cuối cùng:

```text
PASS accepted → arriving = 1
PASS arriving → picked_up = 1
PASS picked_up → in_transit = 1
PASS in_transit → completed = 1
id|status|occurred_at
--|------|-----------
1|requested|2026-11-03 09:00:00.000 +07:00
2|searching|2026-11-03 09:00:05.000 +07:00
3|accepted|2026-11-03 09:01:00.000 +07:00
4|arriving|2026-11-03 09:01:30.000 +07:00
5|picked_up|2026-11-03 09:07:00.000 +07:00
6|in_transit|2026-11-03 09:07:20.000 +07:00
7|completed|2026-11-03 09:31:00.000 +07:00
code|status|fare|so_phut
----|------|----|-------
100032|completed|42000|31
PASS số dòng trip_events = 7
```

Bảy trạng thái, bảy dòng lịch sử, chuyến dài 31 phút (`DATEDIFF(minute, requested_at, completed_at)`). Dòng lịch sử đầu tiên có `id = 1`: `03` không có câu nào ghi vào `trip_events`, và lỗi 333 ở mục 4 là lỗi biên dịch, câu lệnh chưa chạy nên không tiêu số IDENTITY của bảng này. Sau bước cuối, lab trả tài xế 8003 về `online` bằng cùng mẫu có điều kiện.

## 8. DELETE … OUTPUT, và vì sao Tới Nơi hầu như không xoá chuyến

Chuyến thật bị huỷ vẫn được giữ với trạng thái `cancelled`: cần cho đối soát, khiếu nại và thống kê tỷ lệ huỷ. `DELETE` chỉ dành cho dữ liệu tạo nhầm. Bước 6 của lab tạo một bản nháp giao bưu kiện đặt nhầm (mã **100033**) kèm một dòng lịch sử, rồi thử xoá thẳng chuyến:

```sql
DELETE FROM toinoi.trips WHERE id = '71000000-0000-4000-8000-000000000002';
```

```text
PASS D1 xoá chuyến còn lịch sử — lỗi 547: The DELETE statement conflicted with the REFERENCE constraint "fk_trip_events_trip". The conflict occurred in database "ToiNoi", table "toinoi.trip_events", column 'trip_id'.
```

Khoá ngoại bảo vệ lịch sử: còn dòng `trip_events` trỏ tới thì không xoá được chuyến (lab không dùng `ON DELETE CASCADE`). Muốn xoá thật thì xoá con trước, rồi tới cha, trong một transaction:

```sql
BEGIN TRANSACTION;
DELETE FROM toinoi.trip_events
OUTPUT deleted.trip_id, deleted.status
WHERE trip_id = '71000000-0000-4000-8000-000000000002';

DELETE FROM toinoi.trips
OUTPUT deleted.code, deleted.kind, deleted.status, deleted.fare
WHERE id = '71000000-0000-4000-8000-000000000002' AND status = 'requested';
IF @@ROWCOUNT = 1 COMMIT TRANSACTION; ELSE ROLLBACK TRANSACTION;
```

```text
trip_id|status
-------|------
71000000-0000-4000-8000-000000000002|requested
code|kind|status|fare
----|----|------|----
100033|parcel|requested|25000
PASS IDENT_CURRENT sau khi xoá bản nháp = 100033
PASS số dòng trips/trip_events = 16/7
```

`OUTPUT deleted.*` là bản ghi cuối cùng của dòng, có thể lưu vào nhật ký trước khi nó biến mất. Điều kiện `AND status = 'requested'` là lưới an toàn: gõ nhầm `id` của một chuyến đã có tài xế thì DELETE thứ hai ảnh hưởng 0 dòng, `IF @@ROWCOUNT = 1` sai và cả transaction bị ROLLBACK, kể cả các dòng lịch sử vừa xoá. Ở đây `@@ROWCOUNT` được đọc ngay trong điều kiện `IF`, trước mọi câu khác.

Cuối tệp: 16 chuyến (15 fixture + 100032), `trip_events` 7 dòng, nhưng `IDENT_CURRENT` vẫn là **100033**. Chuyến kế tiếp sẽ nhận 100034 (mục 3 đã thấy điều này trên bản sao); số 100033 không bao giờ được dùng lại.

## 9. Làm trong DBStudio

- **Chạy theo batch.** Trong query editor, `Ctrl+Enter` chạy batch `GO` đang chứa con trỏ (hoặc đoạn đang chọn), `Ctrl+Shift+Enter` là "Chạy cả script", `Ctrl+.` huỷ câu đang chạy. Biến như `@ev`, `@moi` chỉ sống trong batch khai báo nó: chạy riêng một câu INSERT dùng `@ev` mà không chọn cả batch thì gặp lỗi "Must declare the table variable".
- **Đọc kết quả.** Mỗi result set (ví dụ bảng `truoc|sau|accepted_at`) nằm trong một tab kết quả riêng; câu DML không có OUTPUT hiện "Ảnh hưởng N dòng". Các dòng `PASS …` của lab là `PRINT`, nên chúng nằm ở tab **Thông điệp**.
- **Thử rồi bỏ.** Thanh công cụ transaction có "Tự động commit" / "Bắt đầu" / "Commit" / "Rollback". Chế độ thủ công dùng `SET IMPLICIT_TRANSACTIONS ON`, nên các câu UPDATE thử nghiệm ở mục 6 có thể huỷ bằng nút "Rollback". Mỗi tab editor giữ một kết nối riêng (giữ `USE`, bảng `#temp` và transaction của tab đó); đóng tab khi còn transaction mở, DBStudio hỏi commit hay rollback.
- **Sửa trên lưới.** Lưới kết quả cho sửa trực tiếp trên bảng có khoá chính, và thay đổi được lưu trong transaction của tab. Đó vẫn là một câu UPDATE, chịu mọi CHECK, nhưng không ghi dòng `trip_events` nào. Đổi trạng thái chuyến luôn đi qua script như trên.
- **Kết nối sản xuất.** Trên kết nối gắn môi trường "Sản xuất", câu phá huỷ như `DELETE`/`UPDATE` không có `WHERE`, `DROP`, `TRUNCATE` mở hộp thoại "Xác nhận câu lệnh phá huỷ" và bắt gõ tên database; kết nối "Chỉ đọc" từ chối mọi câu ghi. Cách tạo kết nối xem ở [MS-002](/articles/mssql-cai-dat-docker-dbstudio).

## 10. So với PostgreSQL

Cùng vòng đời, cùng dữ liệu với [PG-006](/articles/postgresql-insert-update-delete-returning), nhưng cơ chế khác ở mấy chỗ:

| Chủ đề | PostgreSQL 17 | SQL Server 2022 |
|---|---|---|
| Lấy lại dòng vừa ghi | `RETURNING`, chỉ giá trị sau khi ghi (với UPDATE) | `OUTPUT inserted.*` và `deleted.*`, thấy cả trước lẫn sau |
| Đổi trạng thái + ghi lịch sử trong một câu | CTE ghi dữ liệu: `WITH done AS (UPDATE … RETURNING …) INSERT …` | Không được: `OUTPUT INTO` cấm bảng đích có CHECK/FK (333/332), composable DML cấm FK (356). Dùng biến bảng + transaction |
| Mỗi câu lẻ có tự commit không | Có (autocommit) | Có (autocommit), trừ khi bật `IMPLICIT_TRANSACTIONS` |
| Lỗi giữa transaction | Transaction chuyển sang trạng thái lỗi, mọi câu sau bị từ chối tới khi ROLLBACK | Mặc định chỉ huỷ câu lỗi, transaction vẫn mở; cần `SET XACT_ABORT ON` |
| Số dòng bị ảnh hưởng | Thẻ lệnh `UPDATE 0` | `@@ROWCOUNT`, bị đặt lại bởi câu kế tiếp |
| Câu lỗi chuyển kiểu có tiêu số IDENTITY | Không (trên lab PG) | Có (T3–T6 trên lab này) |

## 11. Lỗi thường gặp và đánh đổi

- **UPDATE/DELETE quên `WHERE`.** Câu đúng cú pháp và sửa mọi dòng: `UPDATE toinoi.trips SET fare = 0;` trong `BEGIN TRAN … ROLLBACK` trên bản sao trả `@@ROWCOUNT = 16`. Chạy SELECT với cùng `WHERE` trước, đọc số dòng sau, và để DBStudio canh kết nối sản xuất.
- **Bỏ qua số dòng.** Coi "không có exception" là thành công thì tài xế 8001 được báo nhận chuyến dù 0 dòng bị đổi.
- **Đọc `@@ROWCOUNT` muộn.** Một câu `IF`, `PRINT` hay `SET` chen giữa là giá trị đã khác. Cất vào biến ngay sau câu DML.
- **Đọc rồi mới ghi.** `SELECT status` thấy `searching` rồi `UPDATE … WHERE id = …` không kèm trạng thái: giữa hai câu, trạng thái có thể đã đổi. Đưa điều kiện vào chính câu UPDATE.
- **Transaction nhiều câu mà thiếu `XACT_ABORT ON`.** Câu thứ hai lỗi, câu thứ nhất vẫn được commit.
- **Gỡ CHECK/FK để dùng được `OUTPUT INTO`.** Đổi tính đúng của dữ liệu lấy một câu lệnh ngắn hơn. Dùng biến bảng.
- **Coi mã IDENTITY là liên tục.** Lỗi ràng buộc, lỗi chuyển kiểu, rollback, xoá đều để lại khoảng trống. Đếm chuyến bằng `COUNT(*)`.
- **`OUTPUT inserted.*` cho tiện.** Trả mọi cột và đổi theo khi bảng thêm cột. Liệt kê cột cần dùng. Tài liệu cũng lưu ý thứ tự dòng trong OUTPUT không được đảm bảo; cần thứ tự thì sắp lại khi đọc.
- **Xoá cứng hay giữ trạng thái huỷ.** Xoá làm bảng gọn nhưng mất dấu vết và đụng khoá ngoại; giữ `cancelled` tốn chỗ nhưng trả lời được câu hỏi đối soát. Với chuyến, Tới Nơi chọn giữ.
- **"Có thì sửa, chưa có thì thêm".** Câu hỏi upsert và `MERGE` để dành cho Phần 2 (MS-017).

## 12. Bài tập

**Bài 1.** Khách huỷ chuyến 100013 (đồ ăn, đang `searching`). Viết câu UPDATE huỷ chuyến chỉ khi chưa có ai nhận, trả về `code, kind, status, updated_at` bằng OUTPUT. Chạy hai lần liên tiếp thì lần hai ra gì? Áp câu đó lên chuyến 100012 (`accepted`) thì sao?

<details><summary><b>Lời giải</b></summary><pre><code class="language-sql">UPDATE toinoi.trips
SET status = 'cancelled', updated_at = '2026-11-03 08:40:00 +07:00'
OUTPUT inserted.code, inserted.kind, inserted.status, CAST(inserted.updated_at AS varchar(34)) AS updated_at
WHERE code = 100013 AND status IN ('requested', 'searching');</code></pre>
<p>Đã chạy trong <code>BEGIN TRAN … ROLLBACK</code> trên bản sao lab sau <code>04</code>: lần một trả <code>100013|food|cancelled|2026-11-03 08:40:00.000 +07:00</code>, <code>@@ROWCOUNT = 1</code>. Lần hai OUTPUT rỗng, <code>@@ROWCOUNT = 0</code>: chuyến đã <code>cancelled</code>, bấm huỷ hai lần không gây hại. Áp lên 100012 cũng 0 dòng: chuyến đã có tài xế thì không huỷ được theo luật "chưa ai nhận". Ứng dụng thật còn phải ghi một dòng <code>trip_events</code> trạng thái <code>cancelled</code>, nên dùng mẫu biến bảng + transaction ở mục 4.</p></details>

**Bài 2.** Tài xế bấm "Đã đón khách", app chạy `UPDATE … SET status = 'picked_up' … WHERE id = @id AND status = 'arriving'` và nhận về 0 dòng. Liệt kê các nguyên nhân có thể và cách app phân biệt chúng.

<details><summary><b>Lời giải</b></summary><p>(1) <code>@id</code> không tồn tại: lỗi dữ liệu phía app. (2) Chuyến đã <code>picked_up</code> hoặc đi xa hơn: thường do bấm hai lần hoặc mạng gửi lại, coi như đã xong. (3) Chuyến đã <code>cancelled</code>: báo tài xế. (4) Chuyến còn <code>accepted</code>: tài xế bỏ qua bước "đang tới đón", app phải gửi đúng bước trước. Phân biệt bằng <code>SELECT status FROM toinoi.trips WHERE id = @id</code>: 0 dòng là (1), còn lại xem <code>status</code>. Câu đọc này chỉ để giải thích, không thay điều kiện trong UPDATE.</p></details>

**Bài 3.** Chạy `node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql p01/04-lifecycle.sql` (bỏ `03`). Chuyến của chị Dung nhận mã nào, và hai kiểm tra nào của `04` báo FAIL? Giải thích.

<details><summary><b>Lời giải</b></summary><p>Đã chạy trên bản sao database: chuyến mới nhận <strong>100015</strong>, bản nháp nhận 100016. Hai kiểm tra FAIL là <code>mã chuyến mới … = 100015 (mong đợi 100032)</code> và <code>IDENT_CURRENT sau khi xoá bản nháp = 100016 (mong đợi 100033)</code>; mọi bước vòng đời vẫn PASS. Không có <code>03</code> thì không câu lỗi nào tiêu 17 giá trị IDENTITY trước <code>04</code>. Kiểm tra trong lab cố ý ghi cứng 100032 để chứng minh dãy mã có khoảng trống; code ứng dụng thì không bao giờ được giả định một mã cụ thể.</p></details>

## 13. Đọc tiếp

- Trước đó: [MS-005 · CREATE TABLE, ràng buộc và IDENTITY](/articles/mssql-create-table-constraints).
- Bảng `trip_events` tách khỏi `trips` và quan hệ chuyến – sự kiện: [MS-008](/articles/mssql-mo-hinh-quan-he-chuan-hoa). Fixture và bằng chứng kiểm tra gom thành bản bàn giao: [MS-010](/articles/mssql-thuc-hanh-schema-chuyen-xe).
- Cùng vòng đời trên PostgreSQL: [PG-006 · INSERT, UPDATE, DELETE và RETURNING](/articles/postgresql-insert-update-delete-returning).
- Microsoft Learn: [OUTPUT clause](https://learn.microsoft.com/sql/t-sql/queries/output-clause-transact-sql), [INSERT](https://learn.microsoft.com/sql/t-sql/statements/insert-transact-sql), [UPDATE](https://learn.microsoft.com/sql/t-sql/queries/update-transact-sql), [DELETE](https://learn.microsoft.com/sql/t-sql/statements/delete-transact-sql), [@@ROWCOUNT](https://learn.microsoft.com/sql/t-sql/functions/rowcount-transact-sql), [SCOPE_IDENTITY](https://learn.microsoft.com/sql/t-sql/functions/scope-identity-transact-sql), [SET XACT_ABORT](https://learn.microsoft.com/sql/t-sql/statements/set-xact-abort-transact-sql), [IDENTITY (Property)](https://learn.microsoft.com/sql/t-sql/statements/create-table-transact-sql-identity-property).

Bài tiếp theo: [MS-007 · SELECT: lọc, sắp xếp, TOP và OFFSET/FETCH](/articles/mssql-select-loc-sap-xep-top-offset)
