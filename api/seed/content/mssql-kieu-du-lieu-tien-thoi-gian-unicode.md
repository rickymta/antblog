---
slug: mssql-kieu-du-lieu-tien-thoi-gian-unicode
title: "Kiểu dữ liệu cho tiền, thời gian, chuỗi tiếng Việt và định danh"
summary: "Kế toán Tới Nơi cần tổng cước ngày 02/11, CSKH cần in đúng tên «Nguyễn … Đà Nẵng». Trên SQL Server, chọn sai kiểu là cước lệch vì money và float, chuyến 00:10 rơi sang ngày khác, chữ ễ thành dấu hỏi. Bài này lập data dictionary cho bảng trips."
category: mssql
level: intermediate
tags: [ms-004, mssql, sql-server-2022, data-type, datetimeoffset, nvarchar, collation, uniqueidentifier, toi-noi]
language: vi
---

Kế toán Tới Nơi hỏi: «Tổng cước các chuyến đặt trong ngày 02/11 là bao nhiêu?». Cùng lúc, bộ phận CSKH phàn nàn rằng báo cáo xuất từ một bảng tạm in tên khách thành «Nguy?n Văn An». Ba lỗi có thể xảy ra mà **không có thông báo lỗi nào**: cước lưu bằng `money` hoặc `float` cộng chia ra số lẻ không khớp với cổng thanh toán; «ngày» tính theo UTC gom nhầm chuyến 100008 khách đặt lúc 00:10 sáng 03/11 giờ Việt Nam vào ngày 02/11; chuỗi tiếng Việt đi qua một cột `varchar` mất dấu ngã.

Chọn kiểu dữ liệu là quyết định xem database chặn sai sót nào và để lọt sai sót nào. Bài này chọn kiểu cho tiền, thời điểm, chuỗi và định danh của bảng `trips` trên SQL Server 2022, kèm thí nghiệm chạy thật, rồi gom quyết định vào một data dictionary.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-004 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/mssql-bang-khoa-chinh-clustered-index">MS-003: Bảng, khoá chính và clustered index</a>. Lab: <code>examples/toi-noi-mssql/p01</code> (<code>01-schema.sql</code>, <code>03-fixture-invalid.sql</code>, <code>05-select.sql</code>, thí nghiệm <code>thu-nghiem/ms004-kieu-du-lieu.sql</code>). Sau bài này, bạn lập được data dictionary có đơn vị, nghĩa của NULL, múi giờ và collation cho từng cột.</p>
</div>

## 1. Chọn kiểu là chọn miền giá trị, đơn vị và nghĩa của NULL

Trước khi gõ tên kiểu cho một cột, hãy trả lời ba câu:

1. **Miền giá trị** (domain) là gì? Số nguyên hay có phần lẻ, lớn tới đâu, chữ nào được phép?
2. **Đơn vị** là gì? Đồng hay nghìn đồng, giờ Việt Nam hay UTC?
3. **NULL nghĩa là gì?** «Chưa xảy ra», «không áp dụng» hay «không biết», và có được phép không?

Kiểu dữ liệu là lớp kiểm tra đầu tiên: giá trị ngoài miền của kiểu bị từ chối trước mọi ràng buộc. Sáu câu sai kiểu trong `03-fixture-invalid.sql` (kết quả trong `OUTPUT.txt`):

| Giá trị gửi vào | Cột | Số hiệu lỗi | Thông báo (rút gọn) |
|---|---|---:|---|
| 101 ký tự | `full_name nvarchar(100)` | 2628 | `String or binary data would be truncated in table 'ToiNoi.toinoi.customers', column 'full_name'` |
| `40000` | `cities.id smallint` | 220 | `Arithmetic overflow error for data type smallint, value = 40000` |
| `2147483648` | `fare int` | 8115 | `Arithmetic overflow error converting expression to data type int` |
| `'45.000đ'` | `fare int` | 245 | `Conversion failed when converting the varchar value '45.000đ' to data type int` |
| `'2026-11-31 09:00 +07:00'` | `requested_at datetimeoffset(3)` | 241 | `Conversion failed when converting date and/or time from character string` |
| `'khach-001'` | `customer_id uniqueidentifier` | 8169 | `Conversion failed when converting from a character string to uniqueidentifier` |

Lỗi 2628 có từ SQL Server 2019: thông báo nêu tên bảng, tên cột và giá trị bị cắt, thay cho thông báo 8152 cũ không nói cột nào. Nếu các cột này đều là `nvarchar(max)`, cả «45.000đ» lẫn ngày 31/11 đều được nhận. Kiểu càng sát miền giá trị thật, database càng chặn được nhiều.

## 2. Tiền: số nguyên đồng, không phải `money` hay `float`

Tiền Việt Nam không có đơn vị lẻ dưới đồng trong nghiệp vụ của Tới Nơi, nên lab lưu cước là **số nguyên đồng**: `fare int NOT NULL`, 45.000 đ lưu là `45000`. Các lựa chọn khác trên SQL Server (kích thước lưu trữ theo tài liệu; `int`, `bigint`, `money` đã kiểm bằng `DATALENGTH` trên lab):

| Kiểu | Byte | Chính xác? | Với cước Tới Nơi |
|---|---:|---|---|
| `int` (đồng) | 4 | có | tối đa 2.147.483.647 đ một giá trị, dư cho một chuyến |
| `bigint` (đồng) | 8 | có | cần cho **tổng**, không cần cho một chuyến |
| `decimal(p,s)` | 5–17 | có | cần khi có phần lẻ thật (tỷ lệ phí, ngoại tệ); `decimal(19,0)` tốn 9 byte |
| `money` | 8 | có, nhưng cố định 4 chữ số lẻ | phép chia làm tròn ở chữ số thứ tư |
| `float` / `real` | 8 / 4 | **không** | sai số nhị phân, cộng dồn lệch |

Thí nghiệm `thu-nghiem/ms004-kieu-du-lieu.sql` chia đều 100.000 đ cho ba người bằng `money` và `int`, và cộng `0.1` mười lần bằng `float`:

```sql
DECLARE @m money = 100000, @f float = 0, @i int = 0;
WHILE @i < 10 BEGIN SET @f += 0.1; SET @i += 1; END;
SELECT @m / 3                        AS money_chia_3,
       (@m / 3) * 3                  AS money_nhan_lai,
       100000 / 3                    AS int_chia_3,
       100000 % 3                    AS int_du,
       IIF(@f = 1.0, N'bằng', N'KHÁC') AS float_10_lan_0_1_so_voi_1;
```

```text
money_chia_3|money_nhan_lai|int_chia_3|int_du|float_10_lan_0_1_so_voi_1
------------|--------------|----------|------|-------------------------
33333.3333|99999.9999|33333|1|KHÁC
```

- `money` cắt kết quả ở chữ số lẻ thứ tư: nhân lại không về 100.000 mà thành 99.999,9999 đ, một con số không ai trả được.
- `float` cộng mười lần 0,1 không ra đúng 1. Hiển thị làm tròn che được con số, nhưng phép so sánh và đối soát thì không.
- `int` chia cho `int` cắt phần lẻ: ba phần 33.333 đ thiếu 1 đ, và `%` cho biết phần dư. Cách này **buộc** bạn viết quy tắc «phần dư 1 đ thuộc về ai». `decimal` cũng không thoát được quyết định đó: `CAST(100000 AS decimal(12,0)) / 3` cho `33333.333333` (SQL Server tự chọn 6 chữ số lẻ cho kết quả phép chia), vẫn phải làm tròn về đồng.

**`SUM(int)` trả về `int` và có thể tràn.** Đây là điểm khác PostgreSQL. Tài liệu `SUM` ghi kiểu trả về của đầu vào `int` là `int`. Thí nghiệm cộng hai cước 2 tỷ đồng (giá trị đặt ra để thấy giới hạn):

```sql
DECLARE @x TABLE (fare int);
INSERT INTO @x VALUES (2000000000), (2000000000);
BEGIN TRY
    SELECT SUM(fare) AS tong_int FROM @x;
END TRY
BEGIN CATCH
    PRINT CONCAT(N'PASS SUM(int) tràn — lỗi ', ERROR_NUMBER(), N': ', ERROR_MESSAGE());
END CATCH;
SELECT SUM(CAST(fare AS bigint)) AS tong_bigint FROM @x;
```

```text
PASS SUM(int) tràn — lỗi 8115: Arithmetic overflow error converting expression to data type int.
tong_bigint
-----------
4000000000
```

Với 2 triệu chuyến/ngày (hồ sơ Tới Nơi) và cước trung bình **giả định** 100.000 đ, tổng một ngày là 200 tỷ đồng, gấp gần trăm lần giới hạn `int`. Quy tắc của lab: **cột** cước là `int`, **phép cộng** báo cáo luôn viết `SUM(CAST(fare AS bigint))`. Trong DBStudio, dòng `PRINT` của khối `CATCH` hiện ở tab **Thông điệp**, còn `tong_bigint` là một result set riêng.

## 3. Thời điểm: `datetimeoffset(3)`, không phải `datetime`

SQL Server có nhiều kiểu ngày giờ. Ba kiểu liên quan tới mốc sự kiện:

| Kiểu | Byte (đo trên lab) | Độ chính xác | Mang múi giờ? |
|---|---:|---|---|
| `datetime` | 8 | làm tròn tới bội của 1/300 giây (.000, .003, .007) | không |
| `datetime2(3)` | 7 | 1 mili giây | không |
| `datetimeoffset(3)` | 9 | 1 mili giây | có, lưu kèm **độ lệch** so với UTC (`+07:00`) |

`datetime` là kiểu cũ, vẫn gặp nhiều trong hệ thống có sẵn. Thí nghiệm cho thấy nó làm mất mili giây:

```text
datetime_lam_tron|datetime2_3
-----------------|-----------
2026-11-03 09:00:00.000|2026-11-03 09:00:00.001
```

`datetime2` chính xác nhưng chỉ là **ngày + giờ trên mặt đồng hồ**, không biết là giờ ở đâu. `datetimeoffset` lưu khoảnh khắc kèm độ lệch, và **so sánh theo UTC**: cùng một khoảnh khắc viết ở hai độ lệch thì bằng nhau.

```sql
DECLARE @vn datetimeoffset(3) = '2026-11-03 09:00:00 +07:00',
        @utc datetimeoffset(3) = '2026-11-03 02:00:00 +00:00';
SELECT IIF(@vn = @utc, N'bằng', N'khác') AS so_sanh,
       CAST(SWITCHOFFSET(@vn, '+00:00') AS varchar(34)) AS doi_sang_utc,
       CAST(@utc AT TIME ZONE 'SE Asia Standard Time' AS varchar(34)) AS doi_sang_gio_vn;
```

```text
so_sanh|doi_sang_utc|doi_sang_gio_vn
-------|------------|---------------
bằng|2026-11-03 02:00:00.000 +00:00|2026-11-03 09:00:00.000 +07:00
```

CHECK `ck_trips_accepted_after_requested` của lab dựa vào đúng tính chất này: câu C17 trong `03-fixture-invalid.sql` ghi giờ nhận `01:59 +00:00` cho chuyến đặt lúc `09:00 +07:00`, tức nhận trước khi đặt một phút, và bị chặn với lỗi 547 dù hai chuỗi trông như «01:59 sau 09:00».

Vì vậy mọi mốc sự kiện của Tới Nơi là `datetimeoffset(3)`, DEFAULT là `SYSDATETIMEOFFSET()`, và mọi giá trị trong fixture ghi rõ `+07:00`. Ba chữ số lẻ (mili giây) là đủ cho nghiệp vụ và tiết kiệm 1 byte so với mặc định 7 chữ số.

## 4. Ngày nghiệp vụ, AT TIME ZONE và bẫy DATEFORMAT

`AT TIME ZONE` đổi một giá trị sang múi giờ khác. Trên SQL Server, tên múi giờ là **tên múi giờ Windows**, kể cả khi SQL Server chạy trên Linux như lab này. Giờ Việt Nam là `SE Asia Standard Time`; catalog `sys.time_zone_info` liệt kê 141 tên trên bản CU28 của lab. Tên IANA quen thuộc của Linux bị từ chối:

```text
SELECT CAST('2026-11-03 09:00:00' AS datetime2(0)) AT TIME ZONE 'Asia/Ho_Chi_Minh';
Msg 9820, Level 16, State 1
The time zone parameter 'Asia/Ho_Chi_Minh' provided to AT TIME ZONE clause is invalid.
```

`AT TIME ZONE` có hai hành vi tuỳ kiểu đầu vào. Với `datetimeoffset`, biểu thức **đổi** sang múi giờ đích. Với `datetime2` (không có độ lệch), biểu thức chỉ **gắn** độ lệch của múi giờ đó vào, giữ nguyên giờ trên đồng hồ:

```text
gan_gio_vn|gan_utc
----------|-------
2026-11-03 09:00:00 +07:00|2026-11-03 09:00:00 +00:00
```

Cùng 09:00 nhưng hai kết quả cách nhau 7 tiếng. Nếu cột là `datetime2` và không ai ghi lại giá trị đó là giờ ở đâu, không có cách nào biết dòng nào đúng.

**Ngày của một khoảnh khắc phụ thuộc múi giờ.** `CAST(… AS date)` trên `datetimeoffset` lấy ngày theo **độ lệch đang lưu**; muốn ngày theo UTC thì đổi trước. Trên database `ToiNoi`:

```sql
SELECT code, requested_at,
       CAST(requested_at AT TIME ZONE 'UTC' AS date) AS ngay_utc,
       CAST(requested_at AS date)                    AS ngay_theo_gia_tri_luu
FROM toinoi.trips WHERE code IN (100007, 100008) ORDER BY code;
```

```text
code|requested_at|ngay_utc|ngay_theo_gia_tri_luu
----|------------|--------|---------------------
100007|2026-11-02 23:50:00.000 +07:00|2026-11-02|2026-11-02
100008|2026-11-03 00:10:00.000 +07:00|2026-11-02|2026-11-03
```

Chuyến 100008 thuộc ngày 03/11 theo giờ Việt Nam nhưng ngày 02/11 theo UTC. `05-select.sql` đếm cả fixture theo hai cách và ra `8|9`: 8 chuyến «ngày 02/11» theo giá trị lưu (+07:00), 9 chuyến theo UTC. «Ngày theo giá trị lưu» chỉ đúng khi **mọi** dòng được ghi với `+07:00`; dữ liệu từ nguồn ghi UTC sẽ phá giả định đó. Cách viết bộ lọc ngày an toàn (khoảng nửa mở theo giờ Việt Nam) nằm ở [MS-007](/articles/mssql-select-loc-sap-xep-top-offset).

**Bẫy DATEFORMAT không báo lỗi.** Cuối `03-fixture-invalid.sql`, cùng chuỗi `'03/11/2026'` được đọc dưới hai thiết lập `SET DATEFORMAT` của phiên:

```text
PASS B1 '03/11/2026' với DATEFORMAT dmy = 2026-11-03
PASS B2 '03/11/2026' với DATEFORMAT mdy = 2026-03-11
```

Thiết lập phiên đổi, ngày đổi theo, và không ràng buộc nào chặn được ngày đúng định dạng nhưng sai ý. Kiểu cũ còn tệ hơn: chạy thử trên lab, với `SET DATEFORMAT dmy`, chuỗi `'2026-11-03 09:00:00'` ép sang `datetime` cho ra **2026-03-11**, trong khi cùng chuỗi ép sang `datetime2` vẫn là 2026-11-03, và dạng có chữ `T` (`'2026-11-03T09:00:00'`) đúng với cả hai. Quy tắc của lab: viết ngày giờ dạng `'YYYY-MM-DD hh:mm:ss +07:00'` cho `datetimeoffset`, và ứng dụng gửi ngày giờ bằng **tham số có kiểu** chứ không ghép chuỗi.

## 5. Chuỗi tiếng Việt: `nvarchar`, tiền tố `N'…'` và collation

SQL Server có hai họ kiểu chuỗi:

- **`nvarchar(n)`**: Unicode (UTF-16), `n` tính theo cặp byte; chữ tiếng Việt thường gặp tốn 2 byte mỗi ký tự.
- **`varchar(n)`**: mỗi ký tự được mã hoá theo **code page** (bảng mã một byte) mà **collation** quy định. Collation là bộ quy tắc của cột hay database về bảng mã, cách so sánh và sắp xếp chuỗi (phân biệt hoa thường, dấu…).

Database `ToiNoi` dùng collation `Vietnamese_100_CI_AS`, ứng với code page 1258 (đọc bằng `COLLATIONPROPERTY(N'Vietnamese_100_CI_AS', 'CodePage')`). Code page 1258 có sẵn các chữ như `ă`, `đ`, `à`, nhưng không có dạng dựng sẵn của nhiều chữ mang hai dấu như `ễ`, `ẵ`. Chuỗi viết **không có tiền tố `N`** là `varchar`, nên bị đổi sang code page 1258 ngay từ lúc đọc câu lệnh. Thí nghiệm ghi cùng một tên vào biến bảng có một cột `varchar` và một cột `nvarchar`:

```sql
DECLARE @t TABLE (nhan varchar(30), v_varchar varchar(40), v_nvarchar nvarchar(40));
INSERT INTO @t VALUES
    ('literal có N',   N'Nguyễn Văn An — Đà Nẵng',  N'Nguyễn Văn An — Đà Nẵng'),
    ('literal không N', 'Nguyễn Văn An — Đà Nẵng',   'Nguyễn Văn An — Đà Nẵng');
SELECT nhan, v_varchar, v_nvarchar,
       DATALENGTH(v_nvarchar) AS byte_nvarchar, LEN(v_nvarchar) AS ky_tu
FROM @t;
```

```text
nhan|v_varchar|v_nvarchar|byte_nvarchar|ky_tu
----|---------|----------|-------------|-----
literal có N|Nguy?n Văn An — Đà N?ng|Nguyễn Văn An — Đà Nẵng|46|23
literal không N|Nguy?n Văn An — Đà N?ng|Nguy?n Văn An — Đà N?ng|46|23
```

Đọc kỹ hai điều:

1. **Cột `varchar` làm mất chữ dù chuỗi có `N`**: `ễ` và `ẵ` thành `?` khi giá trị được đổi sang code page của cột. Không có lỗi, không có cảnh báo.
2. **Cột `nvarchar` cũng mất chữ nếu chuỗi thiếu `N`**: chữ đã hỏng từ lúc là `varchar`, đổi sang Unicode sau đó không khôi phục được. Cột `nvarchar` vẫn tốn 46 byte cho 23 ký tự, nhưng hai trong số đó là dấu hỏi.

Vì thế lab dùng `nvarchar` cho mọi chữ người đọc (tên, tên quán, tên thành phố) và luôn viết `N'…'`; `varchar` chỉ dành cho mã ASCII máy đọc: `kind`, `status`, `phone`. Phía ứng dụng, tham số chuỗi phải khai báo kiểu `nvarchar` (ADO.NET gửi `string` là `nvarchar` mặc định); khai báo nhầm `varchar` sẽ mất chữ y như trên.

**Collation UTF-8 (từ SQL Server 2019).** Collation có hậu tố `_UTF8` cho phép `varchar` lưu Unicode bằng UTF-8: chữ ASCII 1 byte, chữ có dấu 2–3 byte. Với tên `Nguyễn Văn An` (13 ký tự):

```text
byte_utf16|byte_utf8|gia_tri_utf8
----------|---------|------------
26|16|Nguyễn Văn An
```

Tiết kiệm đáng kể với chữ Việt nhiều ký tự ASCII, nhưng `n` trong `varchar(n)` bây giờ là **số byte**: chạy thử trên lab, `ễ` chiếm 3 byte, `Nguyễn` chiếm 8 byte, và `CAST(N'Nguyễn Văn An' COLLATE Vietnamese_100_CI_AS_SC_UTF8 AS varchar(10))` chỉ còn `Nguyễn V`. Lab chưa dùng UTF-8 cho cột vì `nvarchar` đơn giản hơn khi tính độ dài theo ký tự; nếu đổi sang, phải tính lại độ dài mọi cột.

Collation còn quyết định so sánh và sắp xếp: `_CI` không phân biệt hoa thường, `_AS` phân biệt dấu. Hệ quả cho CHECK định dạng (vì sao `'sg' LIKE '[A-Z][A-Z]'` vẫn khớp) nằm ở [MS-005](/articles/mssql-create-table-constraints); thứ tự «An, Ăn, Ân…» khi sắp xếp nằm ở MS-007.

## 6. Định danh: `uniqueidentifier`, NEWID, NEWSEQUENTIALID và IDENTITY

**`uniqueidentifier` cho `id`.** Kiểu này lưu một GUID 128 bit (16 byte, đo bằng `DATALENGTH(NEWID())` trên lab). Lưu bằng `uniqueidentifier` thay vì chuỗi vừa gọn vừa được kiểm định dạng (`'khach-001'` bị chặn với lỗi 8169 ở mục 1). Hai hàm sinh giá trị:

- **`NEWID()`**: GUID ngẫu nhiên, gọi được ở bất kỳ đâu. Lab dùng cho `trips.id` (`df_trips_id`).
- **`NEWSEQUENTIALID()`**: GUID tăng dần trên một máy, **chỉ** dùng được trong DEFAULT của cột. Lab dùng cho `customers`, `drivers`, `merchants`, nơi `id` là clustered key.

Thí nghiệm kiểm cả hai tính chất:

```text
ket_qua
-------
PASS NEWSEQUENTIALID tăng dần theo thứ tự chèn
PASS NEWSEQUENTIALID ngoài DEFAULT — lỗi 302: The newsequentialid() built-in function can only be used in a DEFAULT expression for a column of type 'uniqueidentifier' in a CREATE TABLE or ALTER TABLE statement. It cannot be combined with other operators to form a complex scalar expression.
```

Giá trị tăng dần là lý do bảng clustered theo `NEWSEQUENTIALID()` gần như không phân mảnh, trong khi `NEWID()` làm bảng tốn thêm khoảng 55% số trang (thí nghiệm ở [MS-003](/articles/mssql-bang-khoa-chinh-clustered-index)). Cái giá: tài liệu cảnh báo giá trị của `NEWSEQUENTIALID()` đoán được, nên không dùng nó cho định danh cần giữ bí mật, và dãy có thể bắt đầu lại từ giá trị thấp hơn sau khi khởi động lại máy. `trips.id` có thể lộ ra ứng dụng khách nên dùng `NEWID()`, và vì không phải clustered key nên sự ngẫu nhiên chỉ ảnh hưởng index `pk_trips`.

**IDENTITY cho `code`.** `code bigint IDENTITY(100000, 1)` tự cấp số từ 100000, mỗi lần tăng 1. Số không liên tục: câu INSERT lỗi hoặc bị ROLLBACK vẫn tiêu số. Chi tiết và lỗi 544 khi ghi tay ở [MS-005](/articles/mssql-create-table-constraints).

**`int` hay `bigint` cho số tăng dần?** Tính bằng số liệu của hồ sơ Tới Nơi: `trips` thêm khoảng 730 triệu dòng/năm, nên một cột `int` (tối đa 2.147.483.647) cạn sau chưa tới 3 năm; `trip_events` khoảng 16 triệu dòng/ngày, cạn sau khoảng 134 ngày. Lab dùng `bigint` cho cả hai. Ngược lại, `cities.id` là `smallint` (tối đa 32.767) vì Tới Nơi chỉ có vài thành phố, và `drivers.code` là `int`.

## 7. NULL: «chưa có», không phải 0

NULL không phải 0 hay chuỗi rỗng; NULL cho biết giá trị **không có**. Mỗi cột cho phép NULL phải ghi rõ nghĩa:

- `accepted_at IS NULL` = **chưa có ai nhận**. Đừng thay bằng ngày giả như `1900-01-01` (giá trị mà `datetime` trả về khi ép chuỗi rỗng): mọi phép tính thời gian chờ sẽ sai.
- `fare` là `NOT NULL`, và **0 đ là giá trị hợp lệ**: chuyến 100008 miễn phí do khuyến mãi. «Cước 0 đ» và «chưa biết cước» là hai sự thật khác nhau.
- `merchant_id IS NULL` = **không áp dụng** (không phải đơn đồ ăn), khác «chưa biết quán».

NULL còn ảnh hưởng tới ràng buộc: CHECK chỉ từ chối khi biểu thức ra FALSE, nên `CHECK (accepted_at >= requested_at)` cho qua chuyến chưa có `accepted_at`. MS-005 trình bày cách viết quy tắc «phải có» bằng `IS NULL` / `IS NOT NULL`.

## 8. Data dictionary cho bảng trips

**Data dictionary** (từ điển dữ liệu) là bảng mô tả từng cột: kiểu, đơn vị, miền giá trị, NULL nghĩa là gì, múi giờ, collation. Đây là sản phẩm của bài:

| Cột | Kiểu | Đơn vị / miền | NULL nghĩa là | Ghi chú |
|---|---|---|---|---|
| `id` | `uniqueidentifier` | GUID ngẫu nhiên, `NEWID()` | không cho phép | khoá chính NONCLUSTERED |
| `code` | `bigint IDENTITY(100000,1)` | tăng dần, có thể nhảy cóc | không cho phép | clustered key; mã cho khách và CSKH |
| `kind` | `varchar(10)` | `ride` / `food` / `parcel` (ASCII) | không cho phép | |
| `city_id` | `smallint` | id trong `cities` | không cho phép | |
| `customer_id` | `uniqueidentifier` | id trong `customers` | không cho phép | |
| `driver_id` | `uniqueidentifier` | id trong `drivers` | chưa có tài xế nhận | |
| `merchant_id` | `uniqueidentifier` | id trong `merchants` | không áp dụng (không phải `food`) | |
| `status` | `varchar(12)` | 8 trạng thái vòng đời (ASCII) | không cho phép | mặc định `requested` |
| `fare` | `int` | **đồng**, ≥ 0 | không cho phép | 0 đ = miễn phí; tổng dùng `SUM(CAST(fare AS bigint))` |
| `requested_at` | `datetimeoffset(3)` | khoảnh khắc khách đặt, ghi `+07:00` | không cho phép | ngày nghiệp vụ theo giờ Việt Nam |
| `accepted_at` | `datetimeoffset(3)` | khoảnh khắc tài xế nhận | chưa ai nhận | ≥ `requested_at` |
| `completed_at` | `datetimeoffset(3)` | khoảnh khắc hoàn thành | chưa hoàn thành | chỉ có khi `completed` |
| `updated_at` | `datetimeoffset(3)` | lần sửa gần nhất | không cho phép | |

Bảng `customers` thêm một dòng đáng chú ý: `full_name nvarchar(100)`, chữ người đọc, collation của database `Vietnamese_100_CI_AS`. Từ điển nên nằm gần dữ liệu: lab ghi phần quan trọng nhất vào catalog dưới dạng extended property `MS_Description`, cách ghi và đọc lại ở [MS-005](/articles/mssql-create-table-constraints).

## 9. So với PostgreSQL

Bài [PG-004](/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid) có cùng câu hỏi nhưng vài câu trả lời khác hẳn:

| Chủ đề | PostgreSQL | SQL Server |
|---|---|---|
| Tổng của cột số nguyên 4 byte | `sum(integer)` trả `bigint` | `SUM(int)` trả `int`, tràn với lỗi 8115 |
| Kiểu tiền có sẵn | `money` phụ thuộc `lc_monetary` | `money` cố định 4 chữ số lẻ |
| Thời điểm có múi giờ | `timestamptz` lưu UTC, **không** giữ độ lệch gốc | `datetimeoffset` lưu **kèm** độ lệch, so sánh theo UTC |
| Tên múi giờ | IANA (`Asia/Ho_Chi_Minh`) | Windows (`SE Asia Standard Time`), cả trên Linux |
| Chuỗi | `text` luôn là UTF-8 của database | `nvarchar` (UTF-16) hoặc `varchar` theo code page / UTF-8 của collation |
| Định danh | `uuid` + `gen_random_uuid()` | `uniqueidentifier` + `NEWID()` / `NEWSEQUENTIALID()` |
| Định dạng ngày mơ hồ | `DateStyle` | `SET DATEFORMAT`, và `datetime` hiểu sai cả dạng `YYYY-MM-DD` |

## 10. Lỗi thường gặp và đánh đổi

- **Tiền bằng `float` hoặc `money`**: sai số và làm tròn không thấy khi hiển thị, chỉ lộ ra lúc đối soát.
- **`SUM(fare)` không ép `bigint`**: chạy tốt trên dữ liệu thử, tràn khi dữ liệu thật đủ lớn.
- **`datetime` / `datetime2` cho mốc sự kiện**: mất mili giây hoặc mất múi giờ; dữ liệu hai nguồn lệch 7 tiếng không tách lại được.
- **Tên múi giờ IANA trong `AT TIME ZONE`**: lỗi 9820; dùng tên trong `sys.time_zone_info`.
- **Chuỗi ngày theo định dạng địa phương** (`'03/11/2026'`): kết quả phụ thuộc thiết lập phiên.
- **Thiếu `N'…'`, cột `varchar` cho chữ người đọc, tham số ứng dụng khai báo `varchar`**: dấu ngã, dấu hỏi biến mất mà không có lỗi.
- **Coi NULL như 0 hoặc dùng giá trị giả**.

Đánh đổi chính: số nguyên đồng gọn và nhanh nhưng buộc quy tắc làm tròn phải viết ra; `nvarchar` tốn gấp đôi `varchar` với chữ ASCII nhưng không bao giờ mất chữ; `uniqueidentifier` tiện cho hệ phân tán nhưng tốn 16 byte và phải chọn đúng hàm sinh.

## 11. Bài tập

**Bài 1.** Tới Nơi thêm bảng `payments`: số tiền khách trả, tiền tip (có thể không có), phí cổng thanh toán theo tỷ lệ 1,5% và thời điểm cổng xác nhận (có thể chưa có), cùng mã giao dịch của cổng dạng chữ số và chữ cái Latin. Lập data dictionary: kiểu, đơn vị, NULL nghĩa là gì.

<details><summary><b>Lời giải</b></summary><p>Gợi ý một phương án: <code>amount int NOT NULL</code> (đồng, ≥ 0); <code>tip int NOT NULL</code> với DEFAULT 0 nếu «không tip» chỉ là 0 đ, hoặc cho NULL nếu cần phân biệt «khách chưa chọn tip», và ghi rõ lựa chọn. Tỷ lệ phí là phần lẻ thật: <code>decimal(5,4)</code> lưu <code>0.0150</code>; tiền phí tính ra phải làm tròn về đồng theo quy tắc viết ra, không dùng <code>money</code>. <code>confirmed_at datetimeoffset(3) NULL</code>, NULL = cổng chưa xác nhận. Mã giao dịch là chuỗi ASCII máy đọc: <code>varchar(n)</code> với <code>n</code> theo tài liệu của cổng. Báo cáo tổng tiền dùng <code>SUM(CAST(amount AS bigint))</code>.</p></details>

**Bài 2.** Một job nhập dữ liệu ghi `requested_at` vào một cột `datetime2(3)` bằng chuỗi `'2026-11-03 01:00:00'`, lấy từ hệ thống đối tác ghi giờ UTC. Một báo cáo sau đó viết `requested_at AT TIME ZONE 'SE Asia Standard Time'`. Kết quả là gì, và sai ở đâu?

<details><summary><b>Lời giải</b></summary><p>Với <code>datetime2</code>, <code>AT TIME ZONE</code> chỉ gắn độ lệch của múi giờ đích (như thí nghiệm ở mục 4), nên kết quả là <code>2026-11-03 01:00:00.000 +07:00</code>: báo cáo hiểu chuyến được đặt lúc 1 giờ sáng giờ Việt Nam, trong khi thực tế là 8 giờ sáng. Cách đúng: gắn độ lệch gốc trước rồi mới đổi, <code>(requested_at AT TIME ZONE 'UTC') AT TIME ZONE 'SE Asia Standard Time'</code>, cho <code>2026-11-03 08:00:00.000 +07:00</code> (cả hai kết quả đã chạy thử trên lab với biến <code>datetime2(3)</code>). Tốt hơn nữa là lưu bằng <code>datetimeoffset(3)</code> ngay từ đầu để giá trị tự mang độ lệch.</p></details>

**Bài 3.** Hồ sơ Tới Nơi lưu toạ độ tài xế (`driver_locations.lat`, `lng`) bằng `float`. Có mâu thuẫn với lời khuyên «không dùng số thực cho tiền» không?

<details><summary><b>Lời giải</b></summary><p>Không. Toạ độ GPS là phép đo vốn đã có sai số vài mét; sai số làm tròn nhị phân nhỏ hơn rất nhiều bậc, và không ai so bằng tuyệt đối hay cộng dồn toạ độ để đối soát. Tiền thì phải chính xác tới từng đồng và phải khớp với hệ khác. Chọn kiểu theo miền giá trị và cách dùng, không theo một luật chung «cấm float».</p></details>

## 12. Đọc tiếp

- [MS-005: CREATE TABLE, ràng buộc và IDENTITY](/articles/mssql-create-table-constraints): biến data dictionary thành DDL có NOT NULL, UNIQUE, CHECK, DEFAULT và `MS_Description`.
- [MS-007: SELECT: lọc, sắp xếp, TOP và OFFSET/FETCH](/articles/mssql-select-loc-sap-xep-top-offset): lọc theo khoảng thời gian giờ Việt Nam và thứ tự theo collation.
- Microsoft Learn (SQL Server 2022): [int, bigint, smallint, tinyint](https://learn.microsoft.com/sql/t-sql/data-types/int-bigint-smallint-and-tinyint-transact-sql), [decimal và numeric](https://learn.microsoft.com/sql/t-sql/data-types/decimal-and-numeric-transact-sql), [money và smallmoney](https://learn.microsoft.com/sql/t-sql/data-types/money-and-smallmoney-transact-sql), [float và real](https://learn.microsoft.com/sql/t-sql/data-types/float-and-real-transact-sql), [SUM](https://learn.microsoft.com/sql/t-sql/functions/sum-transact-sql), [datetimeoffset](https://learn.microsoft.com/sql/t-sql/data-types/datetimeoffset-transact-sql), [datetime2](https://learn.microsoft.com/sql/t-sql/data-types/datetime2-transact-sql), [AT TIME ZONE](https://learn.microsoft.com/sql/t-sql/queries/at-time-zone-transact-sql), [SET DATEFORMAT](https://learn.microsoft.com/sql/t-sql/statements/set-dateformat-transact-sql), [nchar và nvarchar](https://learn.microsoft.com/sql/t-sql/data-types/nchar-and-nvarchar-transact-sql), [Collation and Unicode support](https://learn.microsoft.com/sql/relational-databases/collations/collation-and-unicode-support), [uniqueidentifier](https://learn.microsoft.com/sql/t-sql/data-types/uniqueidentifier-transact-sql), [NEWSEQUENTIALID](https://learn.microsoft.com/sql/t-sql/functions/newsequentialid-transact-sql).

Bài tiếp theo: [MS-005 · CREATE TABLE, ràng buộc và IDENTITY](/articles/mssql-create-table-constraints)
