---
slug: mssql-create-table-constraints
title: "CREATE TABLE, ràng buộc và IDENTITY"
summary: "Một bản app tài xế ghi chuyến completed mà quên giờ hoàn thành; một script migration không xoá được DEFAULT vì tên tự sinh khác nhau giữa hai môi trường. Viết DDL cho Tới Nơi trên SQL Server với PK, FK, UNIQUE, CHECK, DEFAULT đặt tên, IDENTITY, rồi đọc lỗi 515, 547, 2627, 544."
category: mssql
level: intermediate
tags: [ms-005, mssql, sql-server-2022, ddl, constraint, check, identity, error-number, toi-noi]
language: vi
---

Thứ Hai, nhóm vận hành Tới Nơi thấy báo cáo «thời gian trung bình mỗi chuyến» của tuần trước tụt bất thường. Truy ngược lại mới biết: một bản phát hành của app tài xế đổi chuyến sang `completed` nhưng quên ghi `completed_at`, nên hàng nghìn chuyến «đã xong» không có giờ xong. Cùng tuần, script migration chạy trơn tru ở môi trường thử nhưng hỏng trên môi trường thật với lỗi «không tìm thấy ràng buộc `DF__trips__statu__3B75D760`»: tên DEFAULT do SQL Server tự sinh, và mỗi môi trường sinh một tên khác. (Cả hai sự cố là **giả định** để minh hoạ; Tới Nơi là hệ thống hư cấu.)

Mỗi ứng dụng chỉ kiểm tra phần của mình; thứ duy nhất mọi đường ghi đều phải đi qua là database. Bài này biến quy tắc nghiệp vụ của chuyến thành **ràng buộc** có tên trong `CREATE TABLE` trên SQL Server 2022, rồi cố tình ghi 34 câu dữ liệu sai để đọc từng số hiệu lỗi.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-005 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/mssql-bang-khoa-chinh-clustered-index">MS-003: Bảng, khoá chính và clustered index</a>, <a href="/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode">MS-004: Kiểu dữ liệu cho tiền, thời gian, chuỗi tiếng Việt và định danh</a>. Lab: <code>examples/toi-noi-mssql/p01</code> (<code>01-schema.sql</code>, <code>03-fixture-invalid.sql</code>, <code>04-lifecycle.sql</code>, <code>07-checks.sql</code>). Sau bài này, bạn viết được DDL có PK, FK, UNIQUE, CHECK, DEFAULT đặt tên, dùng IDENTITY đúng cách và đọc được lỗi 515, 547, 2627 khi dữ liệu sai bị chặn.</p>
</div>

## 1. CREATE TABLE gồm những gì

`CREATE TABLE` thuộc nhóm **DDL** (Data Definition Language, câu lệnh định nghĩa cấu trúc), khác với INSERT/UPDATE/DELETE là câu lệnh thao tác dữ liệu. Một định nghĩa bảng trên SQL Server có ba lớp:

- **Cột và kiểu**: `phone varchar(10)`, `fare int`… (đã chọn ở MS-004).
- **DEFAULT**: giá trị dùng khi câu INSERT không nhắc tới cột. Trên SQL Server, DEFAULT là một **ràng buộc có tên**, một đối tượng trong catalog như CHECK hay UNIQUE.
- **Ràng buộc** (constraint): điều kiện mà mọi dòng trong bảng phải thoả. SQL Server kiểm tra khi thêm hoặc sửa dòng; với khoá ngoại thì kiểm cả khi xoá dòng đang được tham chiếu. Câu lệnh vi phạm bị huỷ, bảng giữ nguyên.

Bảng `customers` trong `01-schema.sql` có đủ ba lớp:

```sql
CREATE TABLE toinoi.customers (
    id         uniqueidentifier  NOT NULL CONSTRAINT df_customers_id DEFAULT NEWSEQUENTIALID(),
    phone      varchar(10)       NOT NULL,
    full_name  nvarchar(100)     NOT NULL,
    created_at datetimeoffset(3) NOT NULL CONSTRAINT df_customers_created_at DEFAULT SYSDATETIMEOFFSET(),
    CONSTRAINT pk_customers PRIMARY KEY CLUSTERED (id),
    CONSTRAINT uq_customers_phone UNIQUE (phone),
    CONSTRAINT ck_customers_phone_format CHECK (phone LIKE '0[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]'),
    CONSTRAINT ck_customers_full_name_not_blank CHECK (TRIM(full_name) <> N'')
);
```

`NOT NULL` và `CONSTRAINT df_… DEFAULT …` viết ngay trên dòng cột: đó là **ràng buộc cột** (column constraint). Bốn dòng `CONSTRAINT …` đứng riêng là **ràng buộc bảng** (table constraint), viết được cho nhiều cột. DEFAULT chỉ viết được ở dạng ràng buộc cột trong `CREATE TABLE`; lab để mọi ràng buộc còn lại ở dạng bảng cho dễ đặt tên đồng loạt.

## 2. Các loại ràng buộc và số hiệu lỗi

| Loại | Chặn điều gì | Ví dụ trong lab | Số hiệu lỗi khi vi phạm |
|---|---|---|---|
| `NOT NULL` | Thiếu giá trị bắt buộc | `trips.fare`, `customers.phone` | 515 |
| `PRIMARY KEY` | Thiếu hoặc trùng định danh dòng | `pk_trips (id)` | 2627 (trùng) |
| `UNIQUE` | Hai dòng trùng khoá nghiệp vụ | `uq_customers_phone`, `uq_trips_code` | 2627 |
| `FOREIGN KEY` | Tham chiếu tới dòng không tồn tại, hoặc xoá dòng còn bị tham chiếu | `fk_trips_customer` | 547 |
| `CHECK` | Giá trị trái quy tắc nghiệp vụ trên một dòng | `ck_trips_fare_non_negative` | 547 |
| `DEFAULT` | Không chặn gì; điền giá trị khi câu lệnh bỏ qua cột | `df_trips_status` | — |

SQL Server báo lỗi bằng **số hiệu lỗi** (error number), kèm **mức nghiêm trọng** (severity, 16 là lỗi do câu lệnh của người dùng) và một số **state**: dòng đầu của lỗi có dạng `Msg 5074, Level 16, State 1` (ví dụ đầy đủ ở mục 3). Vài điểm khác với điều người quen database khác hay nghĩ:

- **547 dùng chung cho FOREIGN KEY và CHECK.** Phân biệt bằng nội dung thông báo và tên ràng buộc, nên tên ràng buộc càng quan trọng.
- **Trùng một unique index** (tạo bằng `CREATE UNIQUE INDEX` chứ không phải ràng buộc) cho lỗi **2601** thay vì 2627; thử trên database nháp: `Cannot insert duplicate key row in object 'dbo.t_uq' with unique index 'ux_t_uq_phone'`.
- **UNIQUE của SQL Server coi hai NULL là trùng nhau**: cột `email NULL` có ràng buộc UNIQUE chỉ nhận **một** dòng NULL; dòng NULL thứ hai bị chặn với 2627 và giá trị `(<NULL>)`. Muốn nhiều dòng trống mà các email có giá trị vẫn không trùng, dùng **filtered index** `CREATE UNIQUE INDEX … WHERE email IS NOT NULL` (đã thử: ba dòng, hai NULL, được nhận; email trùng bị chặn với 2601).

Bảng `trips` gom đủ các loại. Trích phần ràng buộc (danh sách cột đầy đủ ở `01-schema.sql`):

```sql
CREATE TABLE toinoi.trips (
    id           uniqueidentifier  NOT NULL CONSTRAINT df_trips_id DEFAULT NEWID(),
    code         bigint            IDENTITY(100000, 1) NOT NULL,
    kind         varchar(10)       NOT NULL,
    status       varchar(12)       NOT NULL CONSTRAINT df_trips_status DEFAULT 'requested',
    fare         int               NOT NULL,              -- đồng
    -- … city_id, customer_id, driver_id, merchant_id, các mốc thời gian
    CONSTRAINT pk_trips PRIMARY KEY NONCLUSTERED (id),
    CONSTRAINT uq_trips_code UNIQUE CLUSTERED (code),
    CONSTRAINT fk_trips_customer FOREIGN KEY (customer_id) REFERENCES toinoi.customers (id),
    CONSTRAINT fk_trips_driver   FOREIGN KEY (driver_id)   REFERENCES toinoi.drivers (id),
    -- … fk_trips_city, fk_trips_merchant cùng dạng
    CONSTRAINT ck_trips_kind CHECK (kind IN ('ride', 'food', 'parcel')),
    CONSTRAINT ck_trips_fare_non_negative CHECK (fare >= 0),
    CONSTRAINT ck_trips_completed_at_by_status CHECK (
        (status = 'completed' AND completed_at IS NOT NULL) OR (status <> 'completed' AND completed_at IS NULL)),
    CONSTRAINT ck_trips_accepted_after_requested CHECK (accepted_at >= requested_at)
    -- … tổng cộng 10 CHECK, các CHECK còn lại ở mục 4
);
```

Vì sao khoá chính `NONCLUSTERED` và clustered index nằm trên `code` đã được giải thích ở [MS-003](/articles/mssql-bang-khoa-chinh-clustered-index).

## 3. Đặt tên mọi ràng buộc, kể cả DEFAULT

Nếu không ghi `CONSTRAINT tên`, SQL Server tự sinh tên. Tạo một bảng nháp viết như hồ sơ Tới Nơi (không đặt tên) trên database nháp `Scratch_ms003` rồi đọc catalog `sys.objects`:

```sql
CREATE TABLE dbo.nhap_khong_ten (
  id     int          NOT NULL PRIMARY KEY,
  code   bigint       UNIQUE,
  kind   varchar(10)  NOT NULL CHECK (kind IN ('ride', 'food', 'parcel')),
  status varchar(12)  NOT NULL DEFAULT 'requested',
  fare   int          NOT NULL CHECK (fare >= 0)
);
SELECT name, type_desc FROM sys.objects
WHERE parent_object_id = OBJECT_ID('dbo.nhap_khong_ten') ORDER BY type_desc, name;
```

```text
name|type_desc
----|---------
CK__nhap_khong__fare__49C3F6B7|CHECK_CONSTRAINT
CK__nhap_khong__kind__47DBAE45|CHECK_CONSTRAINT
DF__nhap_khon__statu__48CFD27E|DEFAULT_CONSTRAINT
PK__nhap_kho__3213E83F5CBBA9F9|PRIMARY_KEY_CONSTRAINT
UQ__nhap_kho__357D4CF9C9B68F1E|UNIQUE_CONSTRAINT
```

Phần đuôi hex (`49C3F6B7`…) sinh ngẫu nhiên khi tạo bảng, nên chạy cùng script trên hai database sẽ ra hai tên khác nhau. Hậu quả hiện ra khi migration cần gỡ cột `status`:

```text
ALTER TABLE dbo.nhap_khong_ten DROP COLUMN status;
Msg 5074, Level 16, State 1
The object 'DF__nhap_khon__statu__48CFD27E' is dependent on column 'status'.
Msg 4922, Level 16, State 9
ALTER TABLE DROP COLUMN status failed because one or more objects access this column.
```

Phải `DROP CONSTRAINT` DEFAULT trước, mà muốn vậy thì phải biết tên của nó trên **từng** môi trường. Script đúng ở môi trường thử sẽ hỏng ở môi trường thật, đúng sự cố mở bài. Lab đặt tên mọi ràng buộc theo quy ước `pk_<bảng>`, `fk_<bảng>_<cột>`, `uq_<bảng>_<cột>`, `ck_<bảng>_<quy tắc>`, `df_<bảng>_<cột>`. Lợi ích:

1. **Thông báo lỗi chỉ thẳng vào quy tắc**: `ck_trips_completed_at_by_status` nói ngay «trạng thái và giờ hoàn thành lệch nhau».
2. **Ứng dụng ánh xạ lỗi theo tên**: 2627 + `uq_customers_phone` ⇒ «Số điện thoại đã đăng ký».
3. **Migration gọi đúng tên** trên mọi môi trường (MS-009).
4. **Kiểm thử ghi tên mong đợi**: `lab.expect_error … 547, N'ck_trips_kind'`; ràng buộc khác bắt nhầm là FAIL.

Catalog còn có cột `is_system_named` cho biết tên do SQL Server sinh hay do người viết đặt; với bốn DEFAULT của `trips` trên database `ToiNoi`, cột này đều là `0`. Kiểm tra K9 của `07-checks.sql` dò tên sai quy ước trên mọi ràng buộc (kết quả `PASS K9 tên ràng buộc sai quy ước = 0`) và kiểm kê theo bảng:

```text
bang|pk|fk|uq|ck|df
----|--|--|--|--|--
cities|1|0|1|2|0
customers|1|0|1|2|2
drivers|1|1|1|5|3
merchants|1|1|0|1|2
trip_events|1|1|0|1|1
trips|1|4|1|10|4
```

`NOT NULL` không có trong bảng kiểm kê: trên SQL Server, NOT NULL là thuộc tính của cột chứ không phải đối tượng có tên. Lỗi 515 vì thế chỉ nêu **tên cột**.

## 4. Viết CHECK trong T-SQL

### Không có kiểu boolean: viết lại «A ⇔ B»

T-SQL không có kiểu boolean, nên một điều kiện không phải là giá trị để so sánh tiếp. Cách viết gọn `(kind = 'food') = (merchant_id IS NOT NULL)` quen thuộc ở PostgreSQL bị từ chối ngay khi biên dịch (thử trên database nháp):

```text
Msg 102, Level 15, State 1
Incorrect syntax near '='.
```

Quy tắc «A khi và chỉ khi B» phải viết thành hai trường hợp được phép: `(A AND B) OR (NOT A AND NOT B)`. Quy tắc «nếu A thì B» viết là `NOT A OR B`:

| Quy tắc nghiệp vụ | Dạng logic | CHECK trong lab |
|---|---|---|
| Đơn đồ ăn có quán, dịch vụ khác không có quán | A ⇔ B | `(kind = 'food' AND merchant_id IS NOT NULL) OR (kind <> 'food' AND merchant_id IS NULL)` |
| Có tài xế khi và chỉ khi có giờ nhận | A ⇔ B | `(driver_id IS NULL AND accepted_at IS NULL) OR (driver_id IS NOT NULL AND accepted_at IS NOT NULL)` |
| Chưa ai nhận thì chưa có tài xế | A ⇒ B | `status NOT IN ('requested', 'searching') OR driver_id IS NULL` |
| Từ accepted tới completed phải có tài xế | A ⇒ B | `status NOT IN ('accepted', …, 'completed') OR driver_id IS NOT NULL` |

Khi viết «NOT A», hãy chắc A không bao giờ ra UNKNOWN. Ở đây `kind` và `status` là NOT NULL, các vế còn lại là `IS NULL` / `IS NOT NULL`, nên không có kẽ hở. `cancelled` không thuộc danh sách nào ở hàng 3–4, nên huỷ trước hay sau khi có tài xế đều hợp lệ (chuyến 100006 và 100003 của fixture).

### CHECK chỉ từ chối khi biểu thức ra FALSE

So sánh với NULL cho ra UNKNOWN, và CHECK **chấp nhận** UNKNOWN. Thử trên database nháp, bảng có `CHECK (accepted_at >= requested_at)`:

```sql
INSERT dbo.t_ck VALUES ('2026-11-03 09:00 +07:00', NULL);
PRINT CONCAT(N'chèn accepted_at NULL: ', @@ROWCOUNT, N' dòng');
```

```text
chèn accepted_at NULL: 1 dòng
```

Nhờ vậy `ck_trips_accepted_after_requested` tự bỏ qua chuyến chưa ai nhận. Mặt trái: CHECK kiểu so sánh **không bao giờ** buộc cột phải có giá trị. Quy tắc «phải có» phải viết tường minh bằng `IS NULL` / `IS NOT NULL`, hoặc dùng NOT NULL.

### Định dạng chuỗi: LIKE và collation nhị phân

SQL Server 2022 chưa có hàm biểu thức chính quy. `LIKE` với lớp ký tự `[0-9]` đủ cho số điện thoại 10 chữ số. Với mã thành phố hai chữ in hoa, có một bẫy: collation `Vietnamese_100_CI_AS` của database không phân biệt hoa thường, nên `[A-Z]` khớp cả chữ thường. Thử trên lab:

```sql
SELECT IIF('sg' LIKE '[A-Z][A-Z]', 1, 0) AS ci_khop_chu_thuong,
       IIF('sg' COLLATE Latin1_General_BIN2 LIKE '[A-Z][A-Z]', 1, 0) AS bin2_khop_chu_thuong,
       IIF('SG' COLLATE Latin1_General_BIN2 LIKE '[A-Z][A-Z]', 1, 0) AS bin2_khop_chu_hoa;
```

```text
ci_khop_chu_thuong|bin2_khop_chu_thuong|bin2_khop_chu_hoa
------------------|--------------------|-----------------
1|0|1
```

Collation nhị phân `Latin1_General_BIN2` so theo mã ký tự, nên `[A-Z]` chỉ còn là 26 chữ in hoa. Vì vậy lab viết `CHECK (code COLLATE Latin1_General_BIN2 LIKE '[A-Z][A-Z]')`, và câu C1 ghi mã `'ct'` bị chặn.

### Chuỗi rỗng, khoảng trắng và TRIM

`ck_customers_full_name_not_blank CHECK (TRIM(full_name) <> N'')` chặn tên rỗng hoặc toàn khoảng trắng. Có một sắc thái của SQL Server: khi so sánh `=` / `<>`, khoảng trắng cuối chuỗi bị bỏ qua. Thử trên lab:

```text
ba_dau_cach_so_voi_rong|len_ba_dau_cach|byte_ba_dau_cach|trim_tab|tab_so_voi_rong
-----------------------|---------------|----------------|--------|---------------
bằng|0|6|khác|khác rỗng
```

`N'   ' = N''` cho kết quả **bằng** và `LEN(N'   ')` là 0, dù chuỗi chiếm 6 byte. Nghĩa là riêng `full_name <> N''` đã chặn được tên toàn dấu cách; `TRIM` ở đây nói rõ ý định và giữ đúng nghĩa nếu sau này quy tắc đổi sang so `LEN` hay đổi collation. Cả hai đều **không** chặn ký tự tab: `TRIM` mặc định chỉ cắt dấu cách, và `NCHAR(9) <> N''` là đúng. Muốn chặn cả tab, xuống dòng thì dùng dạng `TRIM(N' ' + NCHAR(9) + NCHAR(10) + NCHAR(13) FROM full_name)`; thử trên database nháp, CHECK viết theo dạng này chặn tên chỉ gồm tab và dấu cách (lỗi 547), còn nhận `N' An '`.

### Mỗi CHECK một quy tắc, và CHECK chỉ thấy dòng đang ghi

Một dòng vi phạm cùng lúc hai CHECK (loại `taxi` và cước âm, thử trên database nháp) chỉ được báo **một** lỗi:

```text
lỗi 547: The INSERT statement conflicted with the CHECK constraint "ck_t_hai_kind". The conflict occurred in database "Scratch_ms003", table "dbo.t_hai", column 'kind'.
```

Tài liệu không hứa thứ tự kiểm các CHECK, nên đừng viết code dựa vào việc ràng buộc nào được báo. Hệ quả thiết kế: gộp nhiều quy tắc vào một CHECK lớn thì thông báo lỗi không cho biết quy tắc nào hỏng. Lab tách mỗi CHECK một quy tắc, và mỗi câu sai ở `03` chỉ vi phạm đúng một quy tắc.

«Tài xế của chuyến phải cùng thành phố với chuyến» cần đọc bảng `drivers`, CHECK không làm được. Gọi hàm vô hướng tự viết trong CHECK về kỹ thuật là chạy được nhưng chỉ kiểm dòng đang ghi, không kiểm lại khi bảng kia đổi. Lab kiểm các quy tắc liên bảng bằng truy vấn trong `07-checks.sql` (K1–K7) và bàn cách mô hình hoá ở [MS-008](/articles/mssql-mo-hinh-quan-he-chuan-hoa).

## 5. IDENTITY: số do database cấp, không liên tục

`code bigint IDENTITY(100000, 1)` là cột **IDENTITY**: SQL Server tự cấp giá trị bắt đầu từ 100000, mỗi dòng tăng 1. Mỗi bảng có tối đa một cột IDENTITY. Ghi tay giá trị cho cột này bị chặn (câu I1):

```text
PASS I1 tự ghi mã chuyến — lỗi 544: Cannot insert explicit value for identity column in table 'trips' when IDENTITY_INSERT is set to OFF.
```

`SET IDENTITY_INSERT <bảng> ON` cho phép ghi tay trong phiên hiện tại, dùng khi chuyển dữ liệu từ hệ cũ; nó cần quyền sửa bảng và mỗi phiên chỉ bật được cho một bảng tại một thời điểm.

**IDENTITY không bảo đảm liên tục.** Thử trên database nháp: chèn một dòng hợp lệ, một dòng bị CHECK chặn, một dòng trong transaction bị ROLLBACK, rồi một dòng hợp lệ nữa:

```text
code|fare
----|----
100000|1000
100003|3000
```

Hai giá trị 100001 và 100002 đã bị tiêu và không được trả lại. Cùng hiện tượng trong lab: sau 34 câu sai của `03`, chuyến mới ở `04-lifecycle.sql` nhận mã 100032 chứ không phải 100015:

```text
PASS mã chuyến mới (100014 + 17 giá trị bị tiêu ở 03 + 1) = 100032
```

Tài liệu còn nêu thêm nguồn gây lỗ hổng: SQL Server cấp trước một khối giá trị trong bộ nhớ để ghi nhanh, và khi server khởi động lại đột ngột, phần chưa dùng bị bỏ. Kết luận: dùng `code` để gọi tên chuyến, không dùng để đếm, và không viết logic «mã kế tiếp là MAX + 1». `IDENT_CURRENT(N'toinoi.trips')` cho biết giá trị đã cấp gần nhất; `02-fixture-valid.sql` kiểm nó bằng 100014 ngay sau khi nạp.

Trong lưới dữ liệu của DBStudio, cột IDENTITY và cột tính toán là chỉ đọc; sửa trực tiếp trên lưới chỉ làm được với bảng có khoá chính (hoặc unique index trên cột NOT NULL), mọi bảng của lab đều thoả điều kiện này.

## 6. Chạy 03-fixture-invalid.sql: mỗi câu sai, một số hiệu lỗi

`03-fixture-invalid.sql` có 34 câu sai. Mỗi câu chạy qua thủ tục `lab.expect_error` (tạo ở `00-reset.sql`): mở transaction, chạy câu lệnh, ROLLBACK, rồi in `PASS` nếu lỗi đúng số hiệu và thông báo nhắc đúng tên ràng buộc hoặc tên cột. Câu sai mà chạy được là `FAIL`, tức schema thiếu một quy tắc. Đây là kiểm thử cho chính schema.

Trong DBStudio, mở tệp vào query editor và bấm **Ctrl+Shift+Enter** (Chạy cả script) để chạy lần lượt từng batch `GO`. Các dòng `PASS`/`FAIL` là `PRINT`, nên hiện ở tab **Thông điệp**. Chạy bằng runner của lab cho cùng kết quả như `OUTPUT.txt`.

| Nhóm | Số câu | Số hiệu lỗi |
|---|---:|---|
| NOT NULL | 3 | 515 |
| PRIMARY KEY / UNIQUE | 4 | 2627 |
| FOREIGN KEY | 3 | 547 |
| CHECK | 17 | 547 |
| IDENTITY ghi tay | 1 | 544 |
| Kiểu, độ dài, chuyển đổi | 6 | 2628, 220, 8115, 245, 241, 8169 (MS-004) |

Trích kết quả theo nhóm từ `OUTPUT.txt`:

**NOT NULL**: ghi NULL tường minh bị chặn kể cả khi cột có DEFAULT; DEFAULT chỉ dùng khi câu lệnh không nhắc tới cột.

```text
PASS N1 khách thiếu số điện thoại — lỗi 515: Cannot insert the value NULL into column 'phone', table 'ToiNoi.toinoi.customers'; column does not allow nulls. INSERT fails.
PASS N3 đặt NULL tường minh cho status (DEFAULT không cứu) — lỗi 515: Cannot insert the value NULL into column 'status', table 'ToiNoi.toinoi.trips'; column does not allow nulls. INSERT fails.
```

**UNIQUE / PRIMARY KEY**: thông báo nêu cả giá trị bị trùng. Câu U4 dùng mã `'HN'` in hoa để chỉ vi phạm UNIQUE; mã `'hn'` sẽ vi phạm thêm CHECK định dạng.

```text
PASS U2 trùng số điện thoại — lỗi 2627: Violation of UNIQUE KEY constraint 'uq_customers_phone'. Cannot insert duplicate key in object 'toinoi.customers'. The duplicate key value is (0900000001).
```

**FOREIGN KEY**: hai chiều, và chiều xoá được gọi là «REFERENCE constraint».

```text
PASS F1 chuyến của khách không tồn tại — lỗi 547: The INSERT statement conflicted with the FOREIGN KEY constraint "fk_trips_customer". The conflict occurred in database "ToiNoi", table "toinoi.customers", column 'id'.
PASS F3 xoá khách đang có chuyến — lỗi 547: The DELETE statement conflicted with the REFERENCE constraint "fk_trips_customer". The conflict occurred in database "ToiNoi", table "toinoi.trips", column 'customer_id'.
```

**CHECK**: đúng sự cố mở bài. Số `+84…` bị từ chối để UNIQUE trên một dạng chuẩn còn ý nghĩa; chuyến huỷ mà có mốc hoàn thành bị chặn; C17 cho thấy `datetimeoffset` so sánh theo UTC. CHECK một cột nêu tên cột, CHECK nhiều cột thì không:

```text
PASS C3 số điện thoại dạng +84 — lỗi 547: The INSERT statement conflicted with the CHECK constraint "ck_customers_phone_format". The conflict occurred in database "ToiNoi", table "toinoi.customers", column 'phone'.
PASS C16 huỷ mà có mốc hoàn thành — lỗi 547: The INSERT statement conflicted with the CHECK constraint "ck_trips_completed_at_by_status". The conflict occurred in database "ToiNoi", table "toinoi.trips".
PASS C17 nhận chuyến trước giờ đặt (khác múi giờ) — lỗi 547: The INSERT statement conflicted with the CHECK constraint "ck_trips_accepted_after_requested". The conflict occurred in database "ToiNoi", table "toinoi.trips".
```

Cuối tệp, số dòng giữ nguyên như sau bước `02`: không câu sai nào để lại dấu vết.

```text
PASS số dòng cities/customers/drivers/merchants/trips = 3/6/6/3/15
```

Ứng dụng nên xử lý lỗi theo **số hiệu + tên ràng buộc**, không theo câu chữ: thông báo đổi theo ngôn ngữ của phiên đăng nhập, số hiệu thì không. 547 + `ck_trips_completed_at_by_status` nên được coi là lỗi lập trình cần cảnh báo, còn 2627 + `uq_customers_phone` là lỗi người dùng cần báo lại trên form.

## 7. Chú thích cột bằng MS_Description

Data dictionary của MS-004 nên nằm cạnh dữ liệu. SQL Server không có `COMMENT ON`; chú thích được lưu thành **extended property** (thuộc tính mở rộng gắn vào đối tượng trong catalog), theo quy ước tên `MS_Description` mà nhiều công cụ đọc. `01-schema.sql` gọi `sys.sp_addextendedproperty` cho từng bảng và cột:

```sql
EXEC sys.sp_addextendedproperty @name = N'MS_Description',
     @value = N'Cước tính bằng đồng (VND), số nguyên không âm',
     @level0type = N'SCHEMA', @level0name = N'toinoi',
     @level1type = N'TABLE',  @level1name = N'trips',
     @level2type = N'COLUMN', @level2name = N'fare';
```

Đọc lại bằng catalog `sys.extended_properties` (chạy trên database `ToiNoi`, giữ 8 trong 13 dòng):

```sql
SELECT c.name AS cot, CAST(ep.value AS nvarchar(200)) AS mo_ta
FROM sys.columns AS c
LEFT JOIN sys.extended_properties AS ep
       ON ep.major_id = c.object_id AND ep.minor_id = c.column_id AND ep.name = N'MS_Description'
WHERE c.object_id = OBJECT_ID(N'toinoi.trips')
ORDER BY c.column_id;
```

```text
cot|mo_ta
---|-----
id|Định danh kỹ thuật uniqueidentifier; khoá chính NONCLUSTERED
code|Mã chuyến IDENTITY tăng dần, clustered key; có thể nhảy cóc
kind|NULL
…
status|NULL
fare|Cước tính bằng đồng (VND), số nguyên không âm
requested_at|Thời điểm khách đặt (datetimeoffset)
accepted_at|Thời điểm tài xế nhận; NULL khi chưa ai nhận
completed_at|Thời điểm hoàn thành; chỉ có khi status = completed
```

Sáu cột `kind`, `city_id`, `customer_id`, `driver_id`, `merchant_id`, `status` chưa có chú thích, khoảng trống bạn sẽ lấp ở bài tập 3. Giá trị chú thích là `sql_variant`, nên phải `CAST` sang `nvarchar` khi đọc.

## 8. So với PostgreSQL

Bài [PG-005](/articles/postgresql-create-table-constraints) dùng cùng quy tắc nghiệp vụ; các điểm thật sự khác khi viết trên SQL Server:

| Chủ đề | PostgreSQL | SQL Server |
|---|---|---|
| CHECK «A ⇔ B» | `(A) = (B)` | `(A AND B) OR (NOT A AND NOT B)`; không có kiểu boolean |
| Định dạng chuỗi | Regex `~` | `LIKE` + lớp ký tự, `COLLATE Latin1_General_BIN2` khi cần phân biệt hoa thường |
| DEFAULT | Thuộc tính cột, không có tên | Ràng buộc có tên; tên tự sinh có đuôi ngẫu nhiên |
| UNIQUE và NULL | Mặc định nhiều NULL hợp lệ | Chỉ một NULL; dùng filtered unique index |
| Mã lỗi | SQLSTATE 5 ký tự (`23505`, `23514`…) | Số hiệu (2627, 547…); FK và CHECK chung 547 |
| Ghi tay cột identity | `OVERRIDING SYSTEM VALUE` | `SET IDENTITY_INSERT … ON` |
| Chú thích | `COMMENT ON` | Extended property `MS_Description` |

## 9. Lỗi thường gặp và đánh đổi

- **Không đặt tên ràng buộc, nhất là DEFAULT**: migration hỏng ở môi trường khác.
- **Tưởng CHECK chặn NULL** (`CHECK (fare >= 0)` để lọt NULL) và **gộp nhiều luật vào một CHECK**.
- **`LIKE '[A-Z]'` dưới collation CI**: chữ thường lọt qua.
- **UNIQUE trên cột cho phép NULL**: dòng NULL thứ hai bị chặn ngoài ý muốn.
- **Tin IDENTITY liên tục**: báo cáo «thiếu chuyến» vì mã nhảy cóc.
- **Bật `CASCADE` cho tiện**: xoá một thành phố thử nghiệm có thể kéo theo chuyến. `NO ACTION` mặc định buộc xoá có chủ đích.
- **Danh sách giá trị sẽ đổi**: thêm trạng thái mới phải sửa CHECK, một thay đổi schema có quy trình riêng ([MS-009](/articles/mssql-schema-migration-tuong-thich)). Tám trạng thái gần như cố định hợp với CHECK; danh mục hay đổi hợp với bảng danh mục + khoá ngoại.
- **Chi phí ghi**: mỗi lần ghi đều phải kiểm ràng buộc; UNIQUE/PK dựa vào index, FK phải tra bảng cha. Phần 3 sẽ đo phần index thay vì đoán.

## 10. Bài tập

**Bài 1.** Nghiệp vụ yêu cầu thêm: «chuyến `cancelled` không được có `completed_at`». Có cần ràng buộc mới không?

<details><summary><b>Lời giải</b></summary><p>Không cần. <code>ck_trips_completed_at_by_status</code> đã cho phép đúng hai trường hợp: <code>completed</code> có <code>completed_at</code>, và mọi trạng thái khác không có <code>completed_at</code>. Câu C16 của <code>03-fixture-invalid.sql</code> chính là phép thử này và bị chặn với 547. Thêm một CHECK trùng ý chỉ làm thông báo lỗi khó đoán hơn (hai tên cùng đúng).</p></details>

**Bài 2.** Thêm cột `email` cho `customers`: không bắt buộc, không trùng giữa các khách có email, luôn lưu chữ thường. Viết định nghĩa cột, ràng buộc và index có tên.

<details><summary><b>Lời giải</b></summary><pre><code class="language-sql">ALTER TABLE toinoi.customers ADD email varchar(254) NULL
    CONSTRAINT ck_customers_email_lower CHECK (email = LOWER(email) COLLATE Latin1_General_BIN2);
CREATE UNIQUE INDEX ux_customers_email ON toinoi.customers (email) WHERE email IS NOT NULL;</code></pre>
<p>Không dùng ràng buộc <code>UNIQUE</code>, vì trên SQL Server nó chỉ cho một dòng NULL (mục 2); filtered unique index cho nhiều khách chưa có email. Trùng email báo 2601 chứ không phải 2627. CHECK phải so dưới collation nhị phân: với <code>Vietnamese_100_CI_AS</code>, <code>'Binh@x' = 'binh@x'</code> là đúng, nên <code>email = LOWER(email)</code> không chặn được gì. Đã thử trên database nháp (bảng thu nhỏ có cùng cột, CHECK và index): hai khách email NULL được nhận; <code>'Lan@toinoi.test'</code> bị chặn với 547 <code>ck_…_email_lower</code>; còn <code>'Binh@toinoi.test'</code> khi đã có <code>'binh@toinoi.test'</code> bị index chặn trước với 2601, vì index dưới collation CI cũng coi hai giá trị là trùng. K9 của <code>07-checks.sql</code> chỉ dò tên ràng buộc, không dò tên index, nên tiền tố <code>ux_</code> không làm K9 đỏ.</p></details>

**Bài 3.** Thêm chú thích `MS_Description` cho `trips.status`, rồi đoán số hiệu lỗi của từng câu, không chạy thử: (a) INSERT tài xế mã 8002; (b) INSERT chuyến `food` không có quán; (c) INSERT chuyến với `fare = '54k'`; (d) DELETE khách `0900000001`; (e) INSERT chuyến ghi `code = 100100`.

<details><summary><b>Lời giải</b></summary><pre><code class="language-sql">EXEC sys.sp_addextendedproperty @name = N'MS_Description',
     @value = N'Trạng thái hiện tại; 8 giá trị của vòng đời, mặc định requested',
     @level0type = N'SCHEMA', @level0name = N'toinoi', @level1type = N'TABLE', @level1name = N'trips',
     @level2type = N'COLUMN', @level2name = N'status';</code></pre>
<p>(a) 2627 <code>uq_drivers_code</code>. (b) 547 <code>ck_trips_merchant_only_food</code>. (c) 245, lỗi chuyển đổi kiểu, xảy ra trước mọi ràng buộc. (d) 547 «REFERENCE constraint» <code>fk_trips_customer</code>. (e) 544, cột IDENTITY không nhận giá trị ghi tay khi <code>IDENTITY_INSERT</code> đang OFF. Đối chiếu với các câu U3, C11, T4, F3, I1 trong <code>OUTPUT.txt</code>.</p></details>

## 11. Đọc tiếp

- Trước đó: [MS-004 · Kiểu dữ liệu cho tiền, thời gian, chuỗi tiếng Việt và định danh](/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode).
- Thêm ràng buộc trên bảng đang có dữ liệu (`WITH NOCHECK`, ràng buộc không tin cậy): [MS-009](/articles/mssql-schema-migration-tuong-thich).
- Gom DDL, fixture sai và bằng chứng kiểm tra thành bản bàn giao: [MS-010](/articles/mssql-thuc-hanh-schema-chuyen-xe).
- Microsoft Learn (SQL Server 2022): [CREATE TABLE](https://learn.microsoft.com/sql/t-sql/statements/create-table-transact-sql), [Unique constraints and check constraints](https://learn.microsoft.com/sql/relational-databases/tables/unique-constraints-and-check-constraints), [Primary and foreign key constraints](https://learn.microsoft.com/sql/relational-databases/tables/primary-and-foreign-key-constraints), [IDENTITY (property)](https://learn.microsoft.com/sql/t-sql/statements/create-table-transact-sql-identity-property), [SET IDENTITY_INSERT](https://learn.microsoft.com/sql/t-sql/statements/set-identity-insert-transact-sql), [TRIM](https://learn.microsoft.com/sql/t-sql/functions/trim-transact-sql), [sp_addextendedproperty](https://learn.microsoft.com/sql/relational-databases/system-stored-procedures/sp-addextendedproperty-transact-sql), [Database engine events and errors](https://learn.microsoft.com/sql/relational-databases/errors-events/database-engine-events-and-errors).

Bài tiếp theo: [MS-006 · INSERT, UPDATE, DELETE và mệnh đề OUTPUT](/articles/mssql-insert-update-delete-output)
