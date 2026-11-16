---
slug: mssql-phan-trang-keyset
title: "Phân trang lịch sử chuyến: OFFSET/FETCH và keyset trên SQL Server"
summary: "Kế toán của một khách doanh nghiệp Tới Nơi đối chiếu hoá đơn và thấy màn Lịch sử chuyến thiếu đúng hai chuyến đặt cùng giây. Bài so OFFSET/FETCH với phân trang keyset theo (requested_at, code), viết điều kiện trang kế tiếp khi SQL Server không có so sánh bộ giá trị, rồi chứng minh ghép mọi trang lại thì không mất, không lặp dòng."
category: mssql
level: advanced
tags: [mssql, ms-018, sql-server-2022, t-sql, pagination, keyset, offset-fetch, order-by, toi-noi]
language: vi
---

"Cửa hàng Phụ Kiện Lab" là khách doanh nghiệp của Tới Nơi: hệ thống bán hàng của họ đặt chuyến bưu kiện qua API, nhiều khi cả lô cùng lúc. Cuối tháng, kế toán của cửa hàng lật màn **Lịch sử chuyến** (mới nhất trước, 5 chuyến một trang) để đối chiếu với hoá đơn và báo thiếu **hai chuyến**. Hai chuyến đó có trong database, có trong hoá đơn, nhưng không trang nào hiện chúng. Không có lỗi nào được ghi lại.

Nguyên nhân nằm ở câu "lấy trang kế tiếp". Bài này đi từ `OFFSET/FETCH` ([MS-007](/articles/mssql-select-loc-sap-xep-top-offset)) tới **phân trang keyset**, viết điều kiện trang kế tiếp bằng T-SQL khi SQL Server không có cú pháp so sánh bộ giá trị như PostgreSQL, và **chứng minh** điều kiện đó bằng cách đi hết mọi trang.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-018 · Phần 2/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/mssql-select-loc-sap-xep-top-offset">MS-007 · SELECT: lọc, sắp xếp, TOP và OFFSET/FETCH</a>, <a href="/articles/mssql-window-function-window-clause">MS-015 · Window function và mệnh đề WINDOW</a>. Lab: <code>examples/toi-noi-mssql/p02</code> (tệp <code>18-phan-trang-keyset.sql</code>). Sau bài này, bạn viết được điều kiện trang kế tiếp đúng với khoá sắp xếp trùng, kể cả khi hai cột sắp ngược chiều, và có một vòng lặp kiểm chứng cho nó.</p>
</div>

## 1. Dữ liệu: 18 chuyến, hai lô cùng giây

Khách doanh nghiệp `cb000000-…-000000000001` có 18 chuyến bưu kiện ở Sài Gòn trong fixture P02. Đánh số vị trí bằng `ROW_NUMBER()` để có **bảng đối chiếu**: đây là đáp án mà mọi cách phân trang phải tái tạo được.

```sql
SELECT ROW_NUMBER() OVER (ORDER BY requested_at DESC, code DESC)              AS vi_tri,
       (ROW_NUMBER() OVER (ORDER BY requested_at DESC, code DESC) - 1) / 5 + 1 AS trang,
       code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY requested_at DESC, code DESC;
```

```text
vi_tri|trang|code|requested_at
------|-----|----|------------
1|1|102897|2026-10-28 10:30:00.000 +07:00
2|1|102611|2026-10-25 14:10:00.000 +07:00
3|1|102149|2026-10-21 08:50:00.000 +07:00
4|1|101885|2026-10-18 14:30:00.000 +07:00
5|1|101884|2026-10-18 14:30:00.000 +07:00
6|2|101883|2026-10-18 14:30:00.000 +07:00
7|2|101882|2026-10-18 14:30:00.000 +07:00
8|2|101555|2026-10-15 16:45:00.000 +07:00
9|2|101224|2026-10-12 11:05:00.000 +07:00
10|2|100976|2026-10-10 09:00:00.000 +07:00
11|3|100975|2026-10-10 09:00:00.000 +07:00
12|3|100974|2026-10-10 09:00:00.000 +07:00
13|3|100973|2026-10-10 09:00:00.000 +07:00
14|3|100972|2026-10-10 09:00:00.000 +07:00
15|3|100971|2026-10-10 09:00:00.000 +07:00
16|4|100669|2026-10-07 09:20:00.000 +07:00
17|4|100500|2026-10-05 15:40:00.000 +07:00
18|4|100130|2026-10-02 10:15:00.000 +07:00
```

Hai lô API đặt cùng giây: 10/10 09:00:00 có 6 chuyến (100971–100976), 18/10 14:30:00 có 4 chuyến (101882–101885). Cả hai **vắt qua ranh giới trang**. Đó không phải trường hợp hiếm: trong fixture, 305 chuyến thuộc 148 nhóm trùng giờ đặt. `datetimeoffset(3)` lưu tới mili giây, nhưng lô gửi qua API, dữ liệu nhập từ file hay đồng hồ phía máy khách vẫn tạo ra giờ trùng. Đừng dựa vào "hiếm khi trùng".

## 2. OFFSET/FETCH: đúng khi dữ liệu đứng yên

Trang `p` bằng `OFFSET (p - 1) * 5 ROWS FETCH NEXT 5 ROWS ONLY`. Với `code` làm tie-breaker, trang 2 khớp bảng đối chiếu:

```sql
SELECT code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY requested_at DESC, code DESC
OFFSET 5 ROWS FETCH NEXT 5 ROWS ONLY;
```

```text
code|requested_at
----|------------
101883|2026-10-18 14:30:00.000 +07:00
101882|2026-10-18 14:30:00.000 +07:00
101555|2026-10-15 16:45:00.000 +07:00
101224|2026-10-12 11:05:00.000 +07:00
100976|2026-10-10 09:00:00.000 +07:00
```

`OFFSET` đếm **vị trí**, nên nó có hai cái giá:

- **Chi phí tăng theo độ sâu.** Để trả trang thứ 1.000, server phải tạo ra 4.995 dòng đầu theo đúng thứ tự rồi bỏ đi. Fixture 18 dòng không đủ để đo điều này; Phần 3 (MS-021 trở đi) đọc execution plan và đo trên dữ liệu lớn.
- **Trôi khi dữ liệu thay đổi.** Một chuyến mới chen vào đầu danh sách giữa hai lần tải trang thì mọi dòng dịch xuống một bậc (mục 8).

## 3. Keyset: "lấy tiếp sau dòng cuối tôi đã thấy"

**Phân trang keyset** (còn gọi *seek pagination*) không đếm vị trí. Trang sau bắt đầu **ngay sau dòng cuối** của trang trước, xác định bằng giá trị khoá sắp xếp của dòng đó, gọi là **con trỏ**. Ứng dụng giữ con trỏ (thường mã hoá thành một chuỗi trong phản hồi API) và gửi lại khi người dùng bấm "Xem thêm". Con trỏ ở đây là giá trị dữ liệu, không phải `DECLARE CURSOR` của T-SQL: thứ đó giữ trạng thái trong một phiên, không hợp với web nhiều phiên.

Ba điều kiện để keyset đúng:

1. **Thứ tự toàn phần:** `ORDER BY` trên một khoá **duy nhất** và **NOT NULL**, để mỗi dòng có đúng một vị trí.
2. **Điều kiện "sau con trỏ"** dùng **đúng khoá đó, đúng chiều, so sánh chặt**.
3. Mọi trang dùng **cùng** `ORDER BY` và **cùng** cỡ trang.

`requested_at` một mình vi phạm điều kiện 1. Khoá đúng là cặp `(requested_at, code)`: `code` duy nhất nên cặp cũng duy nhất. Trang 1 không có con trỏ, dùng `TOP (5)`:

```sql
SELECT TOP (5) code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY requested_at DESC, code DESC;
```

Kết quả là 5 dòng đầu của bảng đối chiếu (102897 … 101884). Con trỏ ứng dụng giữ lại là khoá của dòng cuối: **(2026-10-18 14:30:00.000 +07:00, 101884)**.

## 4. Con trỏ một cột: mất dòng hoặc lặp dòng

Đội web chỉ giữ giờ đặt và hỏi "các chuyến cũ hơn":

```sql
DECLARE @con_tro_at datetimeoffset(3) = '2026-10-18 14:30:00 +07:00';

SELECT TOP (5) code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND requested_at < @con_tro_at
ORDER BY requested_at DESC, code DESC;
```

```text
code|requested_at
----|------------
101555|2026-10-15 16:45:00.000 +07:00
101224|2026-10-12 11:05:00.000 +07:00
100976|2026-10-10 09:00:00.000 +07:00
100975|2026-10-10 09:00:00.000 +07:00
100974|2026-10-10 09:00:00.000 +07:00
PASS con trỏ một cột, < = 101555,101224,100976,100975,100974
```

101883 và 101882 cùng giờ với con trỏ nên `<` loại chúng, và không trang nào sau đó còn cơ hội lấy lại. Đây chính là hai chuyến kế toán không tìm thấy. Không lỗi, không cảnh báo.

Sửa vội thành `<=` thì trang 2 bắt đầu bằng hai chuyến của trang 1:

```text
PASS con trỏ một cột, <= = 101885,101884,101883,101882,101555
```

Lặp dòng còn tệ hơn khi **một lô cùng giây dài hơn cỡ trang**: dòng cuối trang vẫn mang cùng giờ, con trỏ không nhích, "Xem thêm" trả mãi cùng một trang. Với khoá không duy nhất, **không có** dấu so sánh nào đúng: `<` mất phần còn lại của nhóm trùng, `<=` lặp phần đã thấy.

## 5. Điều kiện trang kế tiếp khi không có so sánh bộ giá trị

Cặp `(requested_at, code)` được so theo **thứ tự từ điển**: so cột đầu, chỉ khi cột đầu bằng nhau mới so cột sau. PostgreSQL viết thẳng được `(requested_at, code) < (x, y)` (*row value comparison*, so sánh bộ giá trị). SQL Server 2022 **không có** cú pháp này:

```text
PASS so sánh bộ (requested_at, code) < (…) — lỗi 4145: An expression of non-boolean type specified in a context where a condition is expected, near ','.
```

Viết tách theo đúng định nghĩa:

```text
(a, b) < (x, y)   ⇔   a < x   OR   (a = x AND b < y)
```

```sql
DECLARE @con_tro_at datetimeoffset(3) = '2026-10-18 14:30:00 +07:00',
        @con_tro_code bigint = 101884;

SELECT TOP (5) code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at < @con_tro_at
       OR (requested_at = @con_tro_at AND code < @con_tro_code))
ORDER BY requested_at DESC, code DESC;
```

```text
code|requested_at
----|------------
101883|2026-10-18 14:30:00.000 +07:00
101882|2026-10-18 14:30:00.000 +07:00
101555|2026-10-15 16:45:00.000 +07:00
101224|2026-10-12 11:05:00.000 +07:00
100976|2026-10-10 09:00:00.000 +07:00
PASS keyset trang 2 = 101883,101882,101555,101224,100976
```

Trang 2 lấy đúng phần còn lại của lô 18/10. Con trỏ mới là (2026-10-10 09:00:00.000 +07:00, 100976), và trang 3 bắt đầu đúng ở giữa lô 10/10:

```text
PASS keyset trang 3 = 100975,100974,100973,100972,100971
```

Ba chi tiết dễ sai khi chép sang code ứng dụng:

- **Dấu ngoặc.** `AND` được ưu tiên trước `OR` (MS-007). Bỏ cặp ngoặc ngoài cùng thì nhánh `OR (requested_at = @x AND code < @y)` không còn điều kiện khách hàng, và sẽ trả chuyến của khách khác đặt đúng giây đó. Trên fixture, câu thiếu ngoặc tình cờ vẫn ra 13 dòng như câu đúng (lab có kiểm cả hai), vì không khách nào khác đặt lúc 18/10 14:30:00. Test trên dữ liệu này không bắt được lỗi; chỉ đọc code mới thấy.
- **Kiểu của con trỏ.** Giữ đúng `datetimeoffset(3)` cùng độ lệch múi giờ. Con trỏ bị cắt xuống giây (`14:30:00` thay vì `14:30:00.437`) sẽ lặp hoặc mất dòng y như mục 4. `datetimeoffset` so sánh theo thời điểm UTC, nên `07:30 +00:00` và `14:30 +07:00` là cùng một con trỏ.
- **Hai cột, hai tham số.** Cột phụ phải là cột duy nhất và NOT NULL (ở đây `code`, khoá của clustered index). Một cột có NULL làm `=` và `<` ra UNKNOWN, và dòng đó biến mất khỏi mọi trang.

## 6. Hai cột sắp ngược chiều

Giả sử màn hình muốn giờ đặt mới trước, nhưng trong cùng một giây thì **mã nhỏ trước** (thứ tự hệ thống của cửa hàng gửi lên): `ORDER BY requested_at DESC, code ASC`. Dấu so sánh của mỗi cột đi theo chiều của chính cột đó: `DESC` ⇒ `<`, `ASC` ⇒ `>`. Trang 1 theo thứ tự này là 102897, 102611, 102149, 101882, 101883; con trỏ là (18/10 14:30, 101883):

```sql
SELECT TOP (5) code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at < @con_tro_at                                  -- DESC ⇒ <
       OR (requested_at = @con_tro_at AND code > @con_tro_code))   -- ASC  ⇒ >
ORDER BY requested_at DESC, code ASC;
```

```text
code|requested_at
----|------------
101884|2026-10-18 14:30:00.000 +07:00
101885|2026-10-18 14:30:00.000 +07:00
101555|2026-10-15 16:45:00.000 +07:00
101224|2026-10-12 11:05:00.000 +07:00
100971|2026-10-10 09:00:00.000 +07:00
```

Ở PostgreSQL, trường hợp này buộc phải viết tách vì so sánh bộ chỉ có một chiều. Ở SQL Server, dạng viết tách là dạng duy nhất, nên trộn chiều chỉ là đổi một dấu. Bài tập 3 chứng minh nó bằng vòng lặp ở mục 7.

## 7. Chứng minh điều kiện trang kế tiếp

Một ví dụ trang 2 đúng chưa phải bằng chứng. Bằng chứng là: **đi hết mọi trang** bằng chính điều kiện đó, rồi kiểm hợp các trang đúng bằng toàn bộ lịch sử, không dòng nào lặp, không dòng nào thiếu, đúng thứ tự.

Lab PostgreSQL làm việc này bằng recursive CTE. Ở SQL Server, phần đệ quy của CTE cấm `TOP` và `OFFSET` (MS-019 giải thích recursive CTE):

```text
PASS TOP trong phần đệ quy của CTE — lỗi 461: The TOP or OFFSET operator is not allowed in the recursive part of a recursive common table expression 'trang'.
```

Nên lab dùng vòng lặp `WHILE` của T-SQL, mô phỏng đúng việc ứng dụng làm: lấy trang, đọc dòng cuối làm con trỏ, lấy trang tiếp, cho tới khi trang trả 0 dòng:

```sql
DECLARE @trang TABLE (so_trang int, vi_tri int, code bigint, requested_at datetimeoffset(3));
DECLARE @so_trang int = 0, @n int = 1,
        @con_tro_at datetimeoffset(3) = NULL, @con_tro_code bigint = NULL;

WHILE @n > 0
BEGIN
    SET @so_trang += 1;

    INSERT INTO @trang (so_trang, vi_tri, code, requested_at)
    SELECT TOP (5) @so_trang, ROW_NUMBER() OVER (ORDER BY requested_at DESC, code DESC), code, requested_at
    FROM toinoi.trips
    WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
      AND (@con_tro_at IS NULL                                     -- trang 1: chưa có con trỏ
           OR requested_at < @con_tro_at
           OR (requested_at = @con_tro_at AND code < @con_tro_code))
    ORDER BY requested_at DESC, code DESC;
    SET @n = @@ROWCOUNT;

    -- Con trỏ mới = dòng cuối của trang vừa lấy.
    SELECT TOP (1) @con_tro_at = requested_at, @con_tro_code = code
    FROM @trang WHERE so_trang = @so_trang
    ORDER BY vi_tri DESC;
END;
```

```text
so_trang|ma
--------|--
1|102897,102611,102149,101885,101884
2|101883,101882,101555,101224,100976
3|100975,100974,100973,100972,100971
4|100669,100500,100130
PASS số dòng mỗi trang = 1:5 2:5 3:5 4:3
tong_ma_qua_cac_trang|ma_khac_nhau|so_chuyen
---------------------|------------|---------
18|18|18
PASS tổng mã / mã khác nhau = 18/18
PASS dòng lệch giữa ghép trang và bảng đối chiếu = 0
```

Bốn trang 5/5/5/3; 18 mã qua các trang, 18 mã khác nhau, đúng bằng số chuyến; và một `FULL JOIN` theo vị trí giữa trang ghép lại với bảng đối chiếu ở mục 1 cho 0 dòng lệch. Cả hai lô cùng giây đều nằm trọn, đúng thứ tự, dù vắt qua ranh giới trang. Vòng lặp kết thúc vì trang thứ năm trả 0 dòng. Lab chạy cùng vòng lặp với hai con trỏ một cột của mục 4, có chốt tối đa 8 trang:

```text
PASS con trỏ một cột < qua mọi trang = 3 trang có dòng, 13 mã khác nhau
PASS con trỏ một cột <= (chốt 8 trang) = 8 trang, 40 dòng, 14 mã khác nhau
PASS con trỏ <=: trang 4 … 8 = 4:100976,100975,100974,100973,100972 5:100976,100975,100974,100973,100972 …
```

Với `<`, vòng lặp dừng êm sau 3 trang và mất 5 chuyến. Với `<=`, lô 10/10 có 6 chuyến, dài hơn cỡ trang, nên từ trang 4 con trỏ không nhích nữa: vòng lặp chỉ dừng nhờ chốt, và 4 chuyến cũ nhất không bao giờ hiện. Đó là lý do một kiểm tra như thế này nên chạy trong test của API phân trang, trên fixture có khoá trùng cố ý.

## 8. Dữ liệu đổi giữa hai lần tải trang

Đây là chỗ `OFFSET` thua rõ nhất. Lab mô phỏng: khách đã xem trang 1, rồi đặt thêm một chuyến (trong `BEGIN TRANSACTION … ROLLBACK`):

```text
chuyen_moi
----------
103174
```

Trang 2 tải bằng `OFFSET 5 ROWS FETCH NEXT 5 ROWS ONLY`:

```text
code|requested_at
----|------------
101884|2026-10-18 14:30:00.000 +07:00
101883|2026-10-18 14:30:00.000 +07:00
101882|2026-10-18 14:30:00.000 +07:00
101555|2026-10-15 16:45:00.000 +07:00
101224|2026-10-12 11:05:00.000 +07:00
PASS keyset sau khi có chuyến mới = 101883,101882,101555,101224,100976
```

Chuyến mới chen vào đầu danh sách, mọi dòng dịch xuống một bậc, nên 101884 (vừa thấy ở trang 1) hiện lại, còn 100976 bị đẩy sang trang sau. Trang 2 bằng keyset với con trỏ (18/10 14:30, 101884) vẫn ra đúng 5 chuyến như trước: điều kiện gắn vào **giá trị** của dòng đã thấy, không vào số thứ tự. Xoá dòng cũng vậy: `OFFSET` làm mất một dòng ở ranh giới, keyset thì không.

Sau `ROLLBACK`, khách vẫn 18 chuyến, nhưng `IDENT_CURRENT('toinoi.trips')` là 103174: IDENTITY không trả lại giá trị đã cấp ([MS-006](/articles/mssql-insert-update-delete-output)). Lab chạy `DBCC CHECKIDENT ('toinoi.trips', RESEED, 103173)` để các tệp sau thấy đúng trạng thái fixture. Đó là việc của lab, không phải việc nên làm trên hệ thống thật.

Keyset không biến danh sách thành ảnh chụp cố định. Chuyến mới hơn trang 1 không hiện cho tới khi người dùng tải lại từ đầu; với lịch sử "mới nhất trước", đó là hành vi mong muốn. Nhưng một dòng được chèn **với giờ cũ** (nhập bù, đồng bộ trễ) nằm sau con trỏ sẽ xuất hiện ở trang sau, còn nằm trước con trỏ thì người đang lật trang không thấy. Mỗi trang là một câu lệnh riêng nhìn dữ liệu ở thời điểm của nó; muốn mọi trang cùng nhìn một thời điểm là chuyện isolation level và snapshot của Phần 4.

## 9. Vì sao keyset rẻ: chỉ nêu ý chính

Với `OFFSET k`, server tạo ra k dòng đầu rồi bỏ đi. Với keyset, nếu có index theo đúng thứ tự `(customer_id, requested_at, code)`, server có thể bắt đầu đọc **ngay tại con trỏ** và dừng sau 5 dòng, nên trang thứ 1.000 tốn gần bằng trang 1. Schema P01 có `ix_trips_customer_requested ON toinoi.trips (customer_id, requested_at DESC)`, không ghi `code`. Index đó đã đủ chưa (bảng có clustered index trên `code`), execution plan trông ra sao, và đo thế nào cho đúng là nội dung Phần 3. Bài này không đo hiệu năng.

## 10. Làm trong DBStudio

- Mục 7 là một batch `GO` duy nhất (biến `@trang`, `@con_tro_at` chỉ sống trong batch đó): đặt con trỏ trong batch và bấm `Ctrl+Enter`. Hai result set (danh sách trang và dòng tổng) hiện ở hai tab kết quả riêng; các dòng `PASS …` nằm ở tab **Thông điệp**.
- Mục 8 có `INSERT`. Nếu chạy tay từng câu thay vì cả batch, bấm "Bắt đầu" trên thanh công cụ transaction trước, và kết thúc bằng "Rollback".
- Muốn gửi bảng đối chiếu cho kế toán khách hàng, export result set của mục 1 thay vì chụp màn hình.

## 11. So với PostgreSQL

Cùng dữ liệu, cùng 18 chuyến với [PG-018](/articles/postgresql-pagination-keyset):

| Chủ đề | PostgreSQL 17 | SQL Server 2022 |
|---|---|---|
| Lấy n dòng | `LIMIT n [OFFSET k]` | `TOP (n)` hoặc `OFFSET k ROWS FETCH NEXT n ROWS ONLY` (bắt buộc `ORDER BY`) |
| Điều kiện sau con trỏ | `(a, b) < (x, y)` | Không có so sánh bộ (lỗi 4145); viết `a < x OR (a = x AND b < y)` |
| Hai cột ngược chiều | Phải viết tách | Như mọi trường hợp: đổi dấu của cột đó |
| Đi hết mọi trang trong một câu | Recursive CTE + `LATERAL … LIMIT` | Phần đệ quy cấm `TOP`/`OFFSET` (lỗi 461); dùng vòng lặp `WHILE` |
| Chèn rồi ROLLBACK | Sequence đã tiêu một số | IDENTITY đã tiêu một số |

## 12. Lỗi thường gặp và đánh đổi

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Thiếu vài dòng ở ranh giới trang, không lỗi | Con trỏ chỉ có cột không duy nhất, so `<` | Thêm khoá phụ duy nhất vào con trỏ và `ORDER BY` |
| Dòng của trang trước hiện lại; "Xem thêm" kẹt | So `<=` với khoá không duy nhất | Khoá duy nhất, so sánh chặt |
| Lỗi 4145 khi chép code PostgreSQL | So sánh bộ giá trị | Viết tách `a < x OR (a = x AND b < y)` |
| Trả cả chuyến của khách khác | Thiếu ngoặc quanh khối `OR` | Bọc cả điều kiện con trỏ trong ngoặc |
| Trang trước trả về trang đầu | Quên đảo chiều sắp khi đi lùi | `>` + `ASC` bên trong, sắp lại `DESC` bên ngoài (bài tập 1) |
| Lặp/mất dòng sau khi app "tối ưu" con trỏ | Con trỏ cắt bớt mili giây hoặc mất độ lệch múi giờ | Giữ nguyên `datetimeoffset(3)` trong con trỏ |

Đánh đổi:

- **Keyset không nhảy được tới "trang 37".** Nó chỉ đi tới, đi lùi từ một con trỏ. Màn lịch sử chuyến và API cuộn vô hạn hợp với keyset; bảng quản trị cần số trang thì vẫn dùng `OFFSET` cho vài trang đầu.
- **Con trỏ lộ khoá.** Con trỏ gồm giờ đặt và mã chuyến; API công khai nên mã hoá (và ký) nó, không để client tự dựng.
- **Đổi thứ tự sắp là đổi con trỏ.** Thêm tuỳ chọn "sắp theo cước" nghĩa là một khoá mới `(fare, code)`, một điều kiện mới và có thể một index mới.

## 13. Bài tập

**Bài 1.** Nút "Trang trước": người dùng đang ở trang 3, dòng đầu trang là (2026-10-10 09:00:00 +07:00, 100975). Viết câu lấy đúng trang 2.

<details><summary><b>Lời giải</b></summary>
<p>Đảo cả dấu so sánh lẫn chiều sắp để lấy 5 dòng ngay <i>trước</i> con trỏ, rồi sắp lại theo thứ tự hiển thị:</p>
<pre><code class="language-sql">SELECT code, requested_at
FROM (SELECT TOP (5) code, requested_at
      FROM toinoi.trips
      WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
        AND (requested_at &gt; @dau_at OR (requested_at = @dau_at AND code &gt; @dau_code))
      ORDER BY requested_at ASC, code ASC) AS truoc
ORDER BY requested_at DESC, code DESC;</code></pre>
<p>Lab: <code>PASS Bài 1: trang trước của trang 3 = 101883,101882,101555,101224,100976</code>, đúng trang 2. Giữ <code>DESC</code> bên trong thì <code>TOP (5)</code> lấy 5 dòng mới nhất trong mọi dòng phía trước, tức trang 1.</p>
</details>

**Bài 2.** Làm sao biết có nên hiện nút "Xem thêm" mà không cần `COUNT(*)`? Áp dụng cho con trỏ của trang 2 (100976) và trang 3 (100971).

<details><summary><b>Lời giải</b></summary>
<p>Lấy <code>TOP (6)</code> (cỡ trang + 1), hiển thị 5 dòng đầu; có dòng thứ 6 thì còn trang. Sau con trỏ 100976 truy vấn trả 6 dòng ⇒ còn trang 4; sau con trỏ 100971 chỉ còn 3 dòng ⇒ trang 4 là trang cuối. Lab: <code>PASS Bài 2: TOP (6) ở trang 3 / trang 4 = 6/3</code>.</p>
</details>

**Bài 3.** Chứng minh điều kiện ở mục 6 (`requested_at DESC, code ASC`) bằng vòng lặp của mục 7.

<details><summary><b>Lời giải</b></summary>
<p>Đổi ba chỗ trong vòng lặp: <code>ORDER BY requested_at DESC, code ASC</code> (cả trong <code>ROW_NUMBER()</code>) và <code>code &gt; @con_tro_code</code>. Lab ghép các trang ra <code>102897,102611,102149,101882,101883,101884,101885,101555,101224,100971,100972,100973,100974,100975,100976,100669,100500,100130</code> và <code>PASS Bài 3: tổng mã / mã khác nhau = 18/18</code>: trong mỗi lô cùng giây, mã tăng dần như yêu cầu.</p>
</details>

## 14. Đọc tiếp

- Trước đó: [MS-017 · Upsert: MERGE và các mẫu an toàn hơn](/articles/mssql-upsert-merge).
- `TOP`, `OFFSET/FETCH`, tie-breaker và thứ tự ưu tiên `AND`/`OR`: [MS-007](/articles/mssql-select-loc-sap-xep-top-offset). `ROW_NUMBER()`: [MS-015](/articles/mssql-window-function-window-clause).
- Execution plan và đo chi phí của `OFFSET`: MS-021 (Phần 3). Snapshot và isolation level: Phần 4.
- Cùng bài toán trên PostgreSQL: [PG-018 · Phân trang lịch sử chuyến bằng keyset](/articles/postgresql-pagination-keyset).
- Microsoft Learn: [ORDER BY clause (OFFSET và FETCH)](https://learn.microsoft.com/sql/t-sql/queries/select-order-by-clause-transact-sql), [TOP](https://learn.microsoft.com/sql/t-sql/queries/top-transact-sql), [WHILE](https://learn.microsoft.com/sql/t-sql/language-elements/while-transact-sql), [WITH common_table_expression](https://learn.microsoft.com/sql/t-sql/queries/with-common-table-expression-transact-sql), [DBCC CHECKIDENT](https://learn.microsoft.com/sql/t-sql/database-console-commands/dbcc-checkident-transact-sql).

Bài tiếp theo: [MS-019 · Recursive CTE với cây khu vực](/articles/mssql-recursive-cte-cay-khu-vuc)
