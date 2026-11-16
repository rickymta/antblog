---
slug: mssql-select-loc-sap-xep-top-offset
title: "SELECT: lọc, sắp xếp, TOP và OFFSET/FETCH cho danh sách chuyến Tới Nơi"
summary: "Hai nhân viên CSKH đếm chuyến ngày 02/11 ra 8 và 9, trang danh sách đổi thứ tự giữa hai lần xem, tìm khách có tên bắt đầu bằng chữ P ra 0 người dù có chị Phạm Thu Dung. Bài đi qua WHERE, ORDER BY, TOP, OFFSET/FETCH, NULL và collation tiếng Việt trên SQL Server 2022 để mỗi câu hỏi có đúng một đáp án."
category: mssql
level: intermediate
tags: [mssql, ms-007, sql-server-2022, select, where, order-by, top, offset-fetch, collation, toi-noi]
language: vi
---

Sáng 03/11, trưởng ca CSKH của Tới Nơi nhờ hai nhân viên đếm số chuyến khách đặt trong **ngày 02/11** để đối chiếu khiếu nại. Một người báo 8, người kia báo 9. Cả hai chạy T-SQL trên cùng database, cùng bảng `toinoi.trips`, không ai gõ sai tên cột.

Cùng sáng đó, màn hình "chuyến theo loại dịch vụ" của đội vận hành, mỗi trang 3 dòng, cho thấy trang 1 là 100003, 100002, 100004, trong khi danh sách đầy đủ lại xếp 100002 lên đầu. Còn một nhân viên khác gõ `WHERE full_name LIKE N'P%'` để tìm khách có tên bắt đầu bằng chữ P và nhận về 0 dòng, dù chị Phạm Thu Dung có trong bảng.

Không câu nào báo lỗi. SQL Server trả đúng những gì câu lệnh yêu cầu; chỉ là câu lệnh chưa nói rõ "ngày 02/11" theo độ lệch giờ nào, hai chuyến cùng loại thì chuyến nào đứng trước, và collation tiếng Việt coi "Ph" là gì. Bài này đi qua các mệnh đề của một câu đọc dữ liệu và chỉ ra chỗ mỗi mệnh đề có thể đúng cú pháp mà sai nghiệp vụ.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-007 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode">MS-004 · kiểu dữ liệu cho tiền, thời gian và chuỗi tiếng Việt</a>, <a href="/articles/mssql-insert-update-delete-output">MS-006 · INSERT, UPDATE, DELETE và OUTPUT</a>. Lab: <code>examples/toi-noi-mssql/p01</code> (tệp <code>05-select.sql</code>). Sau bài này, bạn lọc được chuyến theo trạng thái và theo khoảng thời gian giờ Việt Nam, sắp xếp theo thứ tự xác định, phân trang đúng và hiểu collation tiếng Việt đổi kết quả so sánh chuỗi thế nào.</p>
</div>

## 1. Dữ liệu dùng trong bài

Mọi ví dụ chạy trên fixture của lab: 15 chuyến cố định (ảnh chụp 08:30 ngày 03/11/2026, giờ VN) cộng chuyến **100032** do vòng đời ở [MS-006](/articles/mssql-insert-update-delete-output) tạo ra, thành 16 chuyến. Tái lập:

```bash
cd examples/toi-noi-mssql
node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql \
  p01/03-fixture-invalid.sql p01/04-lifecycle.sql p01/05-select.sql
```

Kết quả bên dưới chép từ `p01/OUTPUT.txt` (SQL Server 2022 16.0.4325.4, RTM-CU28). Câu thử thêm ngoài `05-select.sql` được ghi rõ là đã chạy, trên `ToiNoi` hoặc trên một bản sao cùng dữ liệu. Mọi mốc thời gian trong fixture là `datetimeoffset(3)` ghi kèm độ lệch `+07:00`; database dùng collation `Vietnamese_100_CI_AS`.

Fixture cài sẵn vài "cái bẫy": 100003 và 100004 cùng đặt lúc 12:00 ngày 02/11; 100008 đặt 00:10 ngày 03/11 giờ VN, tức vẫn là ngày 02/11 theo UTC; 100006, 100013, 100014 chưa có tài xế nên `accepted_at` là NULL; tài xế 8006 chưa từng chạy chuyến nào.

## 2. Một câu SELECT được đọc theo thứ tự nào?

Ở dạng cơ bản, câu `SELECT` có các phần viết theo thứ tự cố định:

```sql
SELECT TOP (10) code, kind, city_id, fare   -- danh sách cột muốn nhận, tối đa 10 dòng
FROM toinoi.trips                           -- đọc từ bảng nào
WHERE status = 'completed'                  -- chỉ giữ dòng thoả điều kiện
ORDER BY code;                              -- sắp xếp kết quả
```

Thứ tự **viết** khác thứ tự **ý nghĩa**. Về logic, SQL Server hiểu câu lệnh như sau: lấy các dòng của `FROM` → giữ dòng thoả `WHERE` → tính danh sách cột của `SELECT` (kể cả tên đặt lại bằng `AS`) → sắp theo `ORDER BY` → cắt bằng `TOP` hoặc `OFFSET … FETCH`. Đó là cách *đọc nghĩa*; cách thực thi thật do bộ tối ưu truy vấn chọn và Phần 3 sẽ xem nó qua execution plan.

Thứ tự logic giải thích một lỗi người mới hay gặp: tên đặt bằng `AS` chưa tồn tại lúc `WHERE` chạy, nhưng đã có lúc `ORDER BY` chạy (đã chạy):

```sql
SELECT code, fare / 1000 AS nghin_dong FROM toinoi.trips WHERE nghin_dong > 60 ORDER BY code;
```

```text
Msg 207, Level 16, State 1
Invalid column name 'nghin_dong'.
```

Lọc theo cột gốc thì chạy, và `ORDER BY` dùng được tên mới: `… WHERE fare > 60000 ORDER BY nghin_dong DESC, code` trả 6 dòng, đầu tiên là `100005|126`. (`fare` là `int` nên `fare / 1000` là phép chia nguyên, chỉ dùng để hiển thị, đừng dùng để tính tiền.)

Nên liệt kê cột thay vì `SELECT *`: không kéo cột thừa và không đổi nghĩa khi bảng thêm cột ([MS-009](/articles/mssql-schema-migration-tuong-thich) sẽ thêm `vehicle_plate` vào `drivers`).

## 3. WHERE: chỉ giữ những dòng cần

**`WHERE`** nhận một biểu thức điều kiện (predicate) và giữ dòng nào biểu thức ra TRUE. Dòng ra FALSE **hoặc UNKNOWN** (kết quả khi so sánh với NULL) đều bị loại.

Câu S1 của `05-select.sql` lấy các chuyến chưa kết thúc:

```sql
SELECT code, kind, status, fare
FROM toinoi.trips
WHERE status NOT IN ('completed', 'cancelled')
ORDER BY requested_at DESC, code DESC;
```

```text
code|kind|status|fare
----|----|------|----
100014|ride|requested|48000
100013|food|searching|85000
100012|ride|accepted|39000
100011|parcel|arriving|27000
100010|ride|in_transit|98000
100009|food|picked_up|64000
PASS S1 số chuyến đang chạy = 6
```

Chuỗi phải khớp giá trị lưu; gõ sai chính tả không báo lỗi, chỉ ra ít dòng hơn. CHECK `ck_trips_status` của MS-005 giữ cột trong 8 giá trị đã biết, nên danh sách `IN` luôn đối chiếu được với tài liệu nghiệp vụ.

### AND được ưu tiên trước OR

Muốn lấy "chuyến hoàn thành hoặc bị huỷ, ở TP. Hồ Chí Minh (`city_id = 1`)" mà viết thiếu ngoặc:

```sql
SELECT COUNT(*) FROM toinoi.trips
WHERE status = 'completed' OR status = 'cancelled' AND city_id = 1;
```

SQL Server hiểu thành `status = 'completed' OR (status = 'cancelled' AND city_id = 1)` và đếm ra **9** (đã chạy), gồm cả chuyến hoàn thành ở Hà Nội và Đà Nẵng. Thêm ngoặc `(status = 'completed' OR status = 'cancelled') AND city_id = 1` thì còn **5**: 100000, 100002, 100006, 100007, 100008. Hễ trộn `AND` với `OR` thì đặt ngoặc, kể cả khi bạn nhớ thứ tự ưu tiên.

## 4. Lọc theo khoảng thời gian giờ Việt Nam

Quay lại câu hỏi "chuyến đặt trong ngày 02/11". Với Tới Nơi, "ngày" luôn là **ngày theo giờ Việt Nam**: khách đặt lúc 00:10 sáng 03/11 không nghĩ mình đặt "ngày 02/11".

### Khoảng nửa hở from, to)

Cách viết khuyến nghị là **khoảng nửa hở**: lấy mốc đầu (gồm), bỏ mốc cuối (không gồm), cả hai mốc ghi rõ độ lệch `+07:00`. Câu S2:

```sql
SELECT code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE requested_at >= '2026-11-02 00:00:00 +07:00'
  AND requested_at <  '2026-11-03 00:00:00 +07:00'
ORDER BY requested_at, code;
```

```text
code|requested_at
----|------------
100000|2026-11-02 07:58:00.000 +07:00
100001|2026-11-02 09:15:00.000 +07:00
100002|2026-11-02 11:05:00.000 +07:00
100003|2026-11-02 12:00:00.000 +07:00
100004|2026-11-02 12:00:00.000 +07:00
100005|2026-11-02 17:30:00.000 +07:00
100006|2026-11-02 18:00:00.000 +07:00
100007|2026-11-02 23:50:00.000 +07:00
PASS S2 chuyến đặt ngày 02/11 giờ VN = 8
```

`datetimeoffset` so sánh theo **khoảnh khắc** (quy về UTC), nên hằng số mang `+07:00` cho kết quả đúng bất kể dữ liệu được ghi với độ lệch nào. Vì sao nửa hở tốt hơn hai cách quen tay?

| Cách viết | Vấn đề |
|---|---|
| `BETWEEN '…02 00:00 +07:00' AND '…02 23:59:59 +07:00'` | Bỏ sót chuyến lúc `23:59:59.500`: `datetimeoffset(3)` lưu tới mili giây |
| `BETWEEN '…02 00:00 +07:00' AND '…03 00:00 +07:00'` | `BETWEEN` **gồm cả hai đầu**; chuyến đúng 00:00 ngày 03/11 bị đếm vào cả hai ngày |
| `>= '…02 00:00 +07:00' AND < '…03 00:00 +07:00'` | Các ngày nối nhau không chồng, không hở |

Kiểm cái bẫy thứ hai trên một giá trị (đã chạy): với `@x = '2026-11-03 00:00:00 +07:00'`, cả `@x BETWEEN <ngày 02>` lẫn `@x BETWEEN <ngày 03>` đều ra 1. Một khoảnh khắc thuộc hai ngày, và báo cáo theo ngày cộng lại lớn hơn tổng thật.

### Vì sao người thứ hai đếm ra 9?

SQL Server không có "múi giờ của phiên" như PostgreSQL; sai lệch đến từ hai chỗ khác.

**Mốc không ghi độ lệch.** Chuỗi `'2026-11-02 00:00:00'` khi chuyển sang `datetimeoffset` được gán độ lệch **+00:00** (đã chạy: `CAST('2026-11-02 00:00:00' AS datetimeoffset(3))` ra `2026-11-02 00:00:00.000 +00:00`). Câu lọc như S2 nhưng bỏ `+07:00` ở hai mốc đếm ra **9** (đã chạy): nó đang lọc ngày 02/11 theo UTC, nên 100008 (17:10 UTC ngày 02) lọt vào.

**Cắt ngày bằng CAST.** Câu S3 so hai cách lấy "ngày" từ cột:

```sql
SELECT (SELECT COUNT(*) FROM toinoi.trips
        WHERE CAST(requested_at AS date) = '2026-11-02')                          AS ngay_02_theo_gia_tri_luu,
       (SELECT COUNT(*) FROM toinoi.trips
        WHERE CAST(SWITCHOFFSET(requested_at, '+00:00') AS date) = '2026-11-02') AS ngay_02_theo_utc;
```

```text
ngay_02_theo_gia_tri_luu|ngay_02_theo_utc
------------------------|----------------
8|9
```

`CAST(… AS date)` lấy phần ngày của giá trị **như đã lưu**, tức theo độ lệch đi kèm từng dòng. Fixture ghi mọi mốc bằng `+07:00` nên kết quả là 8. `SWITCHOFFSET(requested_at, '+00:00')` đổi cách biểu diễn sang UTC trước khi cắt nên ra 9. Điều nguy hiểm là cột 8 chỉ đúng **nhờ** dữ liệu được ghi với `+07:00`. Cùng một khoảnh khắc có thể lưu theo nhiều độ lệch (đã chạy: `'2026-11-03 00:10 +07:00'` và `SWITCHOFFSET` của nó sang `+00:00` bằng nhau khi so sánh, nhưng `CAST AS date` ra 2026-11-03 và 2026-11-02). Lab đặt `TZ: Asia/Ho_Chi_Minh` cho container nên `SYSDATETIMEOFFSET()` trả `+07:00` (`DATEPART(TZOFFSET, SYSDATETIMEOFFSET())` = 420); một server đặt múi giờ UTC sẽ ghi mặc định `+00:00`, và `CAST AS date` trên dữ liệu đó trả ngày UTC.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Quy tắc lọc ngày của Tới Nơi</span>
<p>Mốc lọc luôn ghi rõ <code>+07:00</code> (hoặc ứng dụng truyền tham số kiểu <code>datetimeoffset</code>), luôn dạng nửa hở <code>&gt;= from AND &lt; to</code>, không bọc cột trong <code>CAST</code>. Đổi ngày giờ giữa các múi giờ bằng <code>AT TIME ZONE</code> và gom theo khung thời gian là chủ đề của <a href="/articles/mssql-ngay-gio-at-time-zone-date-bucket">MS-016</a> ở Phần 2.</p>
</div>

## 5. ORDER BY: sắp xếp và tie-breaker

Không có `ORDER BY`, SQL Server trả dòng theo thứ tự **không xác định**. Tài liệu `ORDER BY` nói rõ thứ tự của kết quả chỉ được đảm bảo khi có mệnh đề này. Nhiều người đoán rằng bảng `trips` clustered theo `code` (MS-003) thì sẽ trả theo `code`. Chạy `SELECT TOP (5) code FROM toinoi.trips;` trên `ToiNoi` (đã chạy) cho kết quả khác hẳn:

```text
code
----
100006
100013
100014
100000
100008
```

Câu này chỉ cần cột `code`, và mọi index nonclustered đều mang theo clustered key `code`, nên bộ tối ưu được phép đọc một index nhỏ hơn thay cho cả bảng. `SET SHOWPLAN_TEXT ON` cho thấy lần chạy này quét index `ix_trips_driver_id`, nên ba chuyến chưa có tài xế (NULL) đứng đầu. Thêm cột `kind, fare` vào cùng câu thì lại ra 100000, 100001, 100002… Thứ tự là hệ quả của cách đọc đang được chọn, không phải cam kết.

`ORDER BY` nhận một hay nhiều khoá sắp xếp, mỗi khoá có chiều riêng: `ASC` (tăng, mặc định) hoặc `DESC` (giảm). `ORDER BY requested_at DESC, code` nghĩa là giờ đặt giảm dần, **code tăng dần**: chiều không lan sang khoá sau.

### Khoá hoà nhau thì sao?

Khi nhiều dòng bằng nhau ở mọi khoá sắp xếp, SQL Server được phép trả chúng theo thứ tự bất kỳ. Câu S4 lấy hai chuyến cùng 12:00 và thêm `code` làm khoá phụ:

```sql
SELECT code, kind, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE requested_at >= '2026-11-02 11:00 +07:00' AND requested_at < '2026-11-02 13:00 +07:00'
ORDER BY requested_at, code;
```

```text
code|kind|requested_at
----|----|------------
100002|food|2026-11-02 11:05:00.000 +07:00
100003|food|2026-11-02 12:00:00.000 +07:00
100004|food|2026-11-02 12:00:00.000 +07:00
```

Khoá phụ có giá trị duy nhất được thêm vào cuối như vậy gọi là **tie-breaker**. Thiếu nó, chuyện thật sự xảy ra trên lab: cùng `ORDER BY kind`, danh sách đầy đủ và trang đầu 3 dòng cho thứ tự khác nhau (đã chạy, lặp lại được trên `ToiNoi` và trên bản sao cùng dữ liệu):

```text
-- SELECT code, kind FROM toinoi.trips ORDER BY kind;   (5 dòng food đầu tiên)
100002|food
100003|food
100004|food
100009|food
100013|food

-- SELECT TOP (3) code, kind FROM toinoi.trips ORDER BY kind;
100003|food
100002|food
100004|food
```

Đây chính là màn hình của đội vận hành ở đầu bài. Mỗi câu đều sắp đúng theo `kind`; với dòng hoà, câu có `TOP` được phép chọn một cách thực thi khác và trả thứ tự khác. Sửa bằng `ORDER BY kind, code`. Một tập khoá sắp xếp gọi là **xác định** (deterministic) khi không hai dòng nào bằng nhau trên toàn bộ tập khoá: `(kind, code)` xác định vì riêng `code` đã duy nhất (`uq_trips_code`), còn `(kind, status)` thì không.

### NULL đứng ở đâu?

SQL Server coi NULL là **giá trị nhỏ nhất** khi sắp xếp: đứng **đầu** khi `ASC`, đứng cuối khi `DESC`. Câu S7, năm chuyến đầu theo giờ nhận:

```sql
SELECT TOP (5) code, status, CAST(accepted_at AS varchar(34)) AS accepted_at
FROM toinoi.trips
ORDER BY accepted_at, code;
```

```text
code|status|accepted_at
----|------|-----------
100006|cancelled|NULL
100013|searching|NULL
100014|requested|NULL
100000|completed|2026-11-02 08:00:30.000 +07:00
100001|completed|2026-11-02 09:17:00.000 +07:00
```

Ba chuyến chưa ai nhận chiếm đầu danh sách. SQL Server 2022 **không có** cú pháp `NULLS FIRST`/`NULLS LAST`. Muốn đẩy NULL xuống cuối, thêm một biểu thức sắp xếp phụ đặt trước:

```sql
SELECT TOP (5) code, status, CAST(accepted_at AS varchar(34)) AS accepted_at
FROM toinoi.trips
ORDER BY CASE WHEN accepted_at IS NULL THEN 1 ELSE 0 END, accepted_at, code;
```

```text
code|status|accepted_at
----|------|-----------
100000|completed|2026-11-02 08:00:30.000 +07:00
100001|completed|2026-11-02 09:17:00.000 +07:00
100002|completed|2026-11-02 11:06:00.000 +07:00
100003|cancelled|2026-11-02 12:01:00.000 +07:00
100004|completed|2026-11-02 12:02:00.000 +07:00
```

Chiều ngược lại cũng cần nhớ: `ORDER BY accepted_at DESC, code` trên các chuyến sáng 03/11 đặt 100013 và 100014 (NULL) ở **cuối** (đã chạy). Người chuyển từ PostgreSQL sẽ thấy ngược hẳn thói quen cũ.

## 6. TOP và TOP … WITH TIES

**`TOP (n)`** lấy tối đa `n` dòng đầu của kết quả đã sắp. Nên luôn viết `n` trong ngoặc (dạng không ngoặc chỉ còn để tương thích) và luôn đi kèm `ORDER BY`: không có `ORDER BY` thì câu vẫn chạy, nhưng "n dòng đầu" là n dòng tuỳ ý. Câu S5:

```sql
SELECT TOP (3) code, fare FROM toinoi.trips ORDER BY fare DESC, code;

SELECT TOP (1) WITH TIES code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE requested_at >= '2026-11-02 12:00 +07:00'
ORDER BY requested_at;
```

```text
code|fare
----|----
100005|126000
100010|98000
100013|85000
code|requested_at
----|------------
100003|2026-11-02 12:00:00.000 +07:00
100004|2026-11-02 12:00:00.000 +07:00
PASS S5 TOP (1) WITH TIES lúc 12:00 = 2
```

**`WITH TIES`** lấy thêm mọi dòng bằng dòng cuối cùng trên khoá `ORDER BY`. "Chuyến đặt sớm nhất từ 12:00" có hai đáp án ngang nhau, và `TOP (1) WITH TIES` trả cả hai thay vì chọn ngẫu nhiên một. Câu này **bắt buộc** có `ORDER BY`; thiếu thì lỗi 1062 "The TOP N WITH TIES clause is not allowed without a corresponding ORDER BY clause" (đã chạy). Dùng `WITH TIES` khi câu hỏi nghiệp vụ là "những ai đứng đầu"; dùng tie-breaker khi cần đúng `n` dòng.

## 7. OFFSET/FETCH và phân trang

**`OFFSET k ROWS FETCH NEXT n ROWS ONLY`** là cú pháp phân trang chuẩn SQL: bỏ qua `k` dòng đầu, lấy `n` dòng tiếp theo. Trang `p` (đánh từ 1), mỗi trang `n` dòng là `OFFSET (p - 1) * n ROWS FETCH NEXT n ROWS ONLY`. Câu S6, trang 2 với 5 dòng mỗi trang:

```sql
SELECT code, status
FROM toinoi.trips
ORDER BY code
OFFSET 5 ROWS FETCH NEXT 5 ROWS ONLY;
```

```text
code|status
----|------
100005|completed
100006|cancelled
100007|completed
100008|completed
100009|picked_up
```

`OFFSET/FETCH` là một phần của mệnh đề `ORDER BY`, nên thiếu `ORDER BY` là lỗi cú pháp (đã chạy: `Msg 102 … Incorrect syntax near '0'`). Một câu cũng không được dùng cả `TOP` lẫn `OFFSET` (lỗi 10741). Cú pháp ép bạn sắp xếp, nhưng không ép thứ tự đó **xác định**: `ORDER BY kind OFFSET 0 ROWS FETCH NEXT 3 ROWS ONLY` trả đúng 100003, 100002, 100004 như câu `TOP (3)` ở mục 5 (đã chạy). Thêm `code` làm tie-breaker thì trang 1 là 100002, 100003, 100004 và trang 2 là 100009, 100013, 100001: mỗi dòng có một vị trí duy nhất, các trang ghép lại vừa khít.

`OFFSET` còn hai giới hạn mà tie-breaker không chữa được:

- **Chi phí:** dòng bị `OFFSET` bỏ qua vẫn phải được đọc và sắp trong server; trang thứ 1.000 của lịch sử chuyến là cái giá thật.
- **Dữ liệu đổi liên tục:** giữa hai lần bấm "trang sau", một chuyến mới chen vào đầu danh sách thì mọi dòng dịch xuống một bậc, người dùng thấy lại một dòng của trang trước.

Cả hai được giải quyết bằng **phân trang keyset** ("lấy các dòng đứng sau dòng cuối tôi đã thấy") ở MS-018 (Phần 2). Ở Phần 1, `OFFSET/FETCH` có tie-breaker là đủ cho danh sách ngắn.

## 8. NULL trong WHERE: = NULL và NOT IN

So sánh bất kỳ giá trị nào với NULL bằng `=`, `<>`, `<`… đều ra UNKNOWN, và `WHERE` loại dòng UNKNOWN. Câu S8:

```sql
SELECT (SELECT COUNT(*) FROM toinoi.trips WHERE driver_id = NULL)  AS bang_null,
       (SELECT COUNT(*) FROM toinoi.trips WHERE driver_id IS NULL) AS is_null;
```

```text
bang_null|is_null
---------|-------
0|3
```

`= NULL` luôn ra 0 dòng; `IS NULL` ra đúng 3 chuyến chưa có tài xế (100006, 100013, 100014).

Cái bẫy thứ hai khó thấy hơn. Đội vận hành muốn tìm **tài xế chưa từng chạy chuyến nào**:

```sql
SELECT COUNT(*) AS not_in FROM toinoi.drivers
WHERE id NOT IN (SELECT driver_id FROM toinoi.trips);

SELECT d.code AS not_exists FROM toinoi.drivers AS d
WHERE NOT EXISTS (SELECT 1 FROM toinoi.trips AS t WHERE t.driver_id = d.id);
```

```text
not_in
------
0
not_exists
----------
8006
```

`NOT IN (danh sách)` tương đương `id <> a AND id <> b AND …`. Danh sách `driver_id` có NULL (ba chuyến chưa có tài xế), nên với mọi tài xế, một vế là `id <> NULL`, ra UNKNOWN, và cả biểu thức không bao giờ ra TRUE. Kết quả 0 dòng, không lỗi nào. `NOT EXISTS` hỏi "có dòng nào khớp không", không so sánh với NULL, nên trả đúng tài xế 8006. Với truy vấn con, dùng `NOT EXISTS`; NULL, `ISNULL`/`COALESCE` và truy vấn con được bàn kỹ ở MS-011 và MS-014.

## 9. Collation tiếng Việt: so sánh và sắp xếp chuỗi

**Collation** là bộ quy tắc so sánh và sắp xếp chuỗi: chữ hoa có bằng chữ thường không, dấu có được tính không, chữ nào đứng trước chữ nào. Database `ToiNoi` dùng `Vietnamese_100_CI_AS`: quy tắc tiếng Việt, **CI** (case-insensitive) không phân biệt hoa thường, **AS** (accent-sensitive) phân biệt dấu. Hậu tố `_AI` (accent-insensitive) thì bỏ qua dấu.

Trong bảng chữ cái tiếng Việt, ă, â, đ, ê, ô, ơ, ư là **chữ cái riêng**, còn huyền, sắc, hỏi, ngã, nặng là **dấu thanh**. Câu S9 sắp cùng tám tên theo hai collation:

```text
thu_tu_vietnamese|thu_tu_latin1
-----------------|-------------
An, Ăn, Ân, Bình, Dung, Duy, Đức, Em|An, Ân, Ăn, Bình, Đức, Dung, Duy, Em
```

`Vietnamese_100_CI_AS` xếp A < Ă < Â và D < Đ < E, đúng thứ tự từ điển tiếng Việt. `Latin1_General_100_CI_AS` coi Đ là biến thể của D, Ă và Â là biến thể của A, nên "Đức" đứng trước "Dung".

### Phụ âm ghép là một chữ

Collation tiếng Việt còn coi các phụ âm ghép như ch, gi, kh, ng, nh, ph, th, tr là **một chữ cái**. Hệ quả thấy được ngay khi sắp xếp (đã chạy):

```text
vn: Nam, Nụ, Ngân, Nhung, Oanh          latin: Nam, Ngân, Nhung, Nụ, Oanh
vn: Tâm, Tú, Thu, Trang, Uyên           latin: Tâm, Thu, Trang, Tú, Uyên
```

và nguy hiểm hơn, với `LIKE`. Đây là câu tìm khách "tên bắt đầu bằng chữ P" ở đầu bài (đã chạy trên `ToiNoi`):

```text
WHERE full_name LIKE N'P%'                                    → 0 khách
WHERE full_name COLLATE Latin1_General_100_CI_AS LIKE N'P%'   → 1 khách (Phạm Thu Dung)
```

Với `Vietnamese_100_CI_AS`, "Phạm" bắt đầu bằng chữ "Ph", không phải chữ "P". Tương tự `N'Ngô' LIKE N'N%'` và `N'Trần' LIKE N'T%'` đều sai, còn `LIKE N'Ng%'`, `LIKE N'Ph%'` thì đúng. Tìm theo tiền tố là cả âm tiết (`LIKE N'Nguyễn %'` ở S10) không bị ảnh hưởng. Nếu nghiệp vụ thật sự cần "chữ cái Latin đầu tiên là P", ghi rõ `COLLATE Latin1_General_100_CI_AS` cho phép so sánh đó và chấp nhận cái giá ở mục 10.

### Hoa thường và dấu

Phần sau của S9 so tên khách "Nguyễn Văn An" với các cách gõ khác:

```text
khong_phan_biet_hoa|bo_dau_CI_AS|bo_thanh_vn_AI|bo_het_dau_vn_AI|bo_het_dau_latin_AI
-------------------|------------|--------------|----------------|-------------------
1|0|1|0|1
```

- `N'nguyễn văn an'` khớp: CI bỏ qua hoa thường.
- `N'Nguyen Van An'` không khớp dưới `_AS`: dấu được tính.
- Dưới `Vietnamese_100_CI_AI`, `N'Nguyên Văn An'` **khớp** (chỉ khác dấu thanh ngã) nhưng `N'Nguyen Van An'` **không khớp**: `_AI` bỏ qua dấu thanh, còn ê với e vẫn là hai chữ cái khác nhau.
- Dưới `Latin1_General_100_CI_AI`, `N'Nguyen Van An'` khớp: collation Latin coi mọi dấu là "accent".

Tương tự (đã chạy): `N'Đức' = N'Duc'` ra 0 với `Vietnamese_100_CI_AI` và 1 với `Latin1_General_100_CI_AI`; `N'Hà' = N'Ha'` ra 1 với `Vietnamese_100_CI_AI`. Ô tìm kiếm "gõ không dấu vẫn ra" vì thế không làm được chỉ bằng `_AI` của collation tiếng Việt; cách làm đúng (cột chuẩn hoá, full-text) nằm ngoài Phần 1.

## 10. LIKE và ký tự đại diện

`LIKE` so chuỗi với một mẫu: `%` khớp chuỗi bất kỳ (kể cả rỗng), `_` khớp đúng một ký tự, `[abc]` hoặc `[0-9]` khớp một ký tự trong tập. Câu S10:

```sql
SELECT phone, full_name FROM toinoi.customers WHERE full_name LIKE N'Nguyễn %';
SELECT phone, full_name FROM toinoi.customers WHERE full_name LIKE N'%Hà';
```

```text
phone|full_name
-----|---------
0900000001|Nguyễn Văn An
phone|full_name
-----|---------
0900000006|Đặng Ngọc Hà
```

Thêm vài mẫu (đã chạy): `LIKE N'%Ha'` ra 0 dòng vì collation phân biệt dấu, còn `full_name COLLATE Vietnamese_100_CI_AI LIKE N'%Ha'` ra Đặng Ngọc Hà; `LIKE N'Tr_n %'` ra Trần Thị Bình; `phone LIKE '090000000[1-3]'` ra ba số 0900000001 tới 0900000003. Lớp ký tự cũng là cách MS-005 viết CHECK định dạng số điện thoại.

**Ghi chú về hiệu năng (chi tiết ở Phần 3).** Một điều kiện gọi là **SARGable** khi bộ tối ưu dùng được index để tìm thẳng tới vùng dữ liệu cần (seek) thay vì đọc hết. So sánh trực tiếp trên cột (`requested_at >= … AND requested_at < …`, `LIKE N'Nguyễn %'`) thường giữ được khả năng đó. Bọc cột trong hàm hay phép ép kiểu (`CAST(requested_at AS date)`, `full_name COLLATE …`) và mẫu bắt đầu bằng `%` (`LIKE N'%Hà'`) thường làm mất nó. Bảng 16 dòng của lab không cho thấy khác biệt; MS-023 và MS-024 sẽ đo bằng execution plan.

## 11. Làm trong DBStudio

- `05-select.sql` có nhiều câu SELECT trong một batch; mỗi result set hiện ở một tab kết quả riêng, các dòng `PASS …` (in bằng `PRINT`) nằm ở tab **Thông điệp**.
- `Ctrl+Enter` chạy batch `GO` đang chứa con trỏ hoặc đoạn đang chọn: bôi đen một câu SELECT để chạy riêng nó. `Ctrl+.` huỷ câu đang chạy khi lỡ quét một bảng lớn.
- Nhân viên CSKH chỉ cần đọc thì nên dùng kết nối gắn chế độ "Chỉ đọc": DBStudio từ chối mọi câu ghi trên kết nối đó. Cách tạo kết nối xem ở [MS-002](/articles/mssql-cai-dat-docker-dbstudio).

## 12. So với PostgreSQL

Cùng câu hỏi, cùng dữ liệu với [PG-007](/articles/postgresql-select-filter-order-limit); các chỗ khác nhau thật sự:

| Chủ đề | PostgreSQL 17 | SQL Server 2022 |
|---|---|---|
| Giới hạn số dòng | `LIMIT n OFFSET k`, `ORDER BY` không bắt buộc | `TOP (n)` hoặc `OFFSET k ROWS FETCH NEXT n ROWS ONLY` (bắt buộc `ORDER BY`); không dùng chung hai cách |
| Lấy cả các dòng hoà | `FETCH FIRST n ROWS WITH TIES` | `TOP (n) WITH TIES` |
| NULL khi sắp xếp | NULL lớn nhất: cuối khi `ASC`; có `NULLS FIRST/LAST` | NULL nhỏ nhất: đầu khi `ASC`; không có `NULLS LAST`, dùng `CASE` |
| Kiểu thời điểm | `timestamptz` lưu UTC, hiển thị và hiểu chuỗi theo `TimeZone` của phiên | `datetimeoffset` lưu kèm độ lệch; chuỗi không ghi độ lệch được hiểu là `+00:00`; `CAST AS date` theo độ lệch đã lưu |
| Collation của lab | `C` (so theo byte, Đ đứng sau mọi chữ Latin) | `Vietnamese_100_CI_AS` (đúng thứ tự tiếng Việt, coi ph, ng, tr… là một chữ) |
| Đếm có điều kiện | `count(*) FILTER (WHERE …)` | `SUM(CASE WHEN … THEN 1 ELSE 0 END)` |

## 13. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Hai người đếm cùng một ngày ra số khác nhau | Mốc không ghi độ lệch (hiểu là `+00:00`), hoặc `CAST AS date` trên dữ liệu ghi bằng nhiều độ lệch | Mốc ghi `+07:00`, khoảng nửa hở |
| Báo cáo theo ngày cộng lại lớn hơn tổng | `BETWEEN` gồm mốc cuối | `>= from AND < to` |
| `WHERE x = NULL` luôn rỗng | So sánh với NULL ra UNKNOWN | `IS NULL` |
| "Tài xế chưa chạy chuyến nào" ra 0 | `NOT IN` gặp NULL trong truy vấn con | `NOT EXISTS` |
| Lọc "SG" mà ra chuyến Hà Nội | Trộn `AND`/`OR` không ngoặc | Đặt ngoặc |
| Trang 1 khác thứ tự với danh sách đầy đủ | `ORDER BY` không xác định | Thêm tie-breaker duy nhất |
| NULL chiếm đầu danh sách | `ASC` đặt NULL đầu tiên | `ORDER BY CASE WHEN x IS NULL THEN 1 ELSE 0 END, x` |
| `LIKE N'P%'` không ra "Phạm" | Collation tiếng Việt coi "Ph" là một chữ | `LIKE N'Ph%'`, hoặc ghi rõ collation Latin cho phép so sánh đó |
| Invalid column name với tên vừa đặt bằng `AS` | `WHERE` chạy trước danh sách cột | Lọc theo cột gốc |

## 14. Đánh đổi

- **Liệt kê cột hay `SELECT *`:** liệt kê dài hơn nhưng ổn định khi schema đổi. Dùng `*` để khám phá, còn trong mã ứng dụng thì liệt kê cột.
- **Tie-breaker `code` hay `id`:** cả hai duy nhất; `code` tăng theo thời điểm tạo nên thứ tự có nghĩa với người vận hành và trùng clustered key, còn `id` là GUID ngẫu nhiên.
- **Nửa hở hay hàm trên cột:** `CAST(SWITCHOFFSET(requested_at, '+07:00') AS date) = '2026-11-02'` cũng ra 8 và dễ đọc, nhưng biến đổi cột ở mỗi dòng; khoảng nửa hở để nguyên cột, có lợi khi có index.
- **`TOP` hay `OFFSET/FETCH`:** `TOP` gọn cho "n dòng đầu", có `WITH TIES`; `OFFSET/FETCH` cho phân trang và ép có `ORDER BY`.
- **`OFFSET` hay keyset:** `OFFSET` đơn giản, nhảy được tới trang bất kỳ; keyset rẻ và ổn định hơn nhưng chỉ đi "trang sau/trang trước".
- **Collation tiếng Việt hay Latin:** `Vietnamese_100` sắp đúng thói quen người Việt nhưng đổi nghĩa `LIKE` với phụ âm ghép; Latin thì ngược lại. Chọn ở mức database (MS-004), và chỉ ghi `COLLATE` trong câu lệnh khi có lý do rõ.

## 15. Bài tập

**Bài 1.** Đội quán ăn muốn xem các đơn đồ ăn đã hoàn thành có cước từ 50.000 đ trở lên, cước cao nhất lên đầu, thứ tự phải xác định. Viết câu lệnh và dự đoán số dòng trước khi chạy.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT code, city_id, fare, CAST(completed_at AS varchar(34)) AS completed_at
FROM toinoi.trips
WHERE kind = 'food' AND status = 'completed' AND fare &gt;= 50000
ORDER BY fare DESC, code;</code></pre>
<p>Kết quả đã chạy: 2 dòng, <code>100002|1|62000</code> (SG) rồi <code>100004|3|55000</code> (Đà Nẵng). Đơn 100003 (71.000 đ) bị loại vì đã huỷ; <code>code</code> giữ thứ tự cố định khi hai đơn cùng cước.</p>
</details>

**Bài 2.** Lấy các chuyến đặt trong khung **12:00 đến trước 18:00 ngày 02/11 giờ VN**. Chuyến 100006 đặt đúng 18:00 có thuộc khung không? Câu lệnh phải cho cùng kết quả dù dữ liệu được ghi với độ lệch nào.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT code, kind, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE requested_at &gt;= '2026-11-02 12:00:00 +07:00'
  AND requested_at &lt;  '2026-11-02 18:00:00 +07:00'
ORDER BY requested_at, code;</code></pre>
<p>Kết quả đã chạy: 100003, 100004 (cùng 12:00, mốc đầu được gồm) và 100005 (17:30). Chuyến 100006 lúc 18:00 thuộc khung kế tiếp vì mốc cuối không gồm. Hai mốc ghi <code>+07:00</code> và <code>datetimeoffset</code> so sánh theo khoảnh khắc, nên một dòng lưu <code>2026-11-02 05:00:00 +00:00</code> vẫn được tính đúng là 12:00 giờ VN.</p>
</details>

**Bài 3.** Điều phối viên tìm tài xế họ Ngô bằng `WHERE full_name LIKE N'N%'` trên bảng `toinoi.drivers`. Dự đoán kết quả, giải thích, rồi sửa câu lệnh. Sau đó lấy **2 chuyến chưa kết thúc có cước cao nhất**; khi nào nên dùng `TOP (2) WITH TIES` thay cho tie-breaker?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT code, full_name FROM toinoi.drivers WHERE full_name LIKE N'Ng%';

SELECT TOP (2) code, status, fare FROM toinoi.trips
WHERE status NOT IN ('completed', 'cancelled')
ORDER BY fare DESC, code;</code></pre>
<p>Đã chạy: <code>LIKE N'N%'</code> ra 0 dòng vì <code>Vietnamese_100_CI_AS</code> coi "Ng" là một chữ cái; <code>LIKE N'Ng%'</code> ra <code>8003|Ngô Quang Minh</code>. Câu thứ hai ra 100010 (<code>in_transit</code>, 98.000 đ) và 100013 (<code>searching</code>, 85.000 đ). Hôm nay không có hai chuyến cùng cước ở ranh giới nên <code>TOP (2) WITH TIES … ORDER BY fare DESC</code> cũng ra đúng hai dòng đó. Nếu chuyến thứ ba cũng 85.000 đ, <code>WITH TIES</code> trả 3 dòng (đúng khi câu hỏi là "những chuyến cước cao nhất"), còn tie-breaker <code>code</code> trả đúng 2 dòng cố định (đúng khi màn hình chỉ có 2 ô).</p>
</details>

## 16. Đọc tiếp

- Microsoft Learn: [SELECT - ORDER BY Clause](https://learn.microsoft.com/sql/t-sql/queries/select-order-by-clause-transact-sql) (gồm OFFSET/FETCH), [TOP](https://learn.microsoft.com/sql/t-sql/queries/top-transact-sql), [WHERE](https://learn.microsoft.com/sql/t-sql/queries/where-transact-sql), [LIKE](https://learn.microsoft.com/sql/t-sql/language-elements/like-transact-sql), [NULL and UNKNOWN](https://learn.microsoft.com/sql/t-sql/language-elements/null-and-unknown-transact-sql), [datetimeoffset](https://learn.microsoft.com/sql/t-sql/data-types/datetimeoffset-transact-sql), [SWITCHOFFSET](https://learn.microsoft.com/sql/t-sql/functions/switchoffset-transact-sql), [Collation and Unicode support](https://learn.microsoft.com/sql/relational-databases/collations/collation-and-unicode-support).
- Cùng chủ đề trên PostgreSQL: [PG-007 · SELECT đầu tiên: lọc, sắp xếp và giới hạn](/articles/postgresql-select-filter-order-limit).
- Lab: `examples/toi-noi-mssql/p01/05-select.sql` và kết quả đầy đủ ở `p01/OUTPUT.txt`.

Bài tiếp theo: [MS-008 · Chuẩn hoá và ranh giới mô hình dữ liệu](/articles/mssql-mo-hinh-quan-he-chuan-hoa)
