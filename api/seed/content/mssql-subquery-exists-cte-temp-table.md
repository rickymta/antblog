---
slug: mssql-subquery-exists-cte-temp-table
title: "Subquery, EXISTS, CTE và bảng tạm: tách câu đối soát tháng 10 của Tới Nơi thành từng bước kiểm chứng được"
summary: "Câu đối soát tháng 10 báo 7 chuyến lệch tiền nhưng không ai chứng minh được nó đúng, câu \"tài xế chưa chạy chuyến nào\" trả 0 dòng dù có hai người như vậy, còn câu đếm theo thành phố bị SQL Server từ chối với lỗi 130. Bài đi qua subquery, IN và EXISTS, bẫy NOT IN với NULL, chuỗi CTE có câu kiểm kê cho từng bước, rồi so bảng tạm #… với biến bảng @… trên SQL Server 2022."
category: mssql
level: intermediate
tags: [mssql, ms-014, sql-server-2022, t-sql, subquery, exists, cte, temp-table, table-variable, toi-noi]
language: vi
---

Cuối tháng 10, đội tài chính Tới Nơi chạy câu đối soát "chuyến nào có tiền thực thu khác tiền phải thu". Câu lệnh lồng ba tầng subquery và trả về **7 chuyến**. Kế toán trưởng hỏi ba câu: phạm vi gồm bao nhiêu chuyến, bước ghép với giao dịch có làm mất hay nhân đôi chuyến nào không, và 7 dòng đó có giải thích hết chênh lệch của cả tháng không. Không ai trả lời được, vì câu lệnh chỉ có một đầu ra.

Cùng tuần, bộ phận tuyển dụng hỏi "tài xế nào đã đăng ký nhưng chưa chạy chuyến nào". Câu `NOT IN` của họ trả **0 dòng**, và họ kết luận ai cũng đã chạy. Thật ra có hai tài xế như vậy. Một nhân viên phân tích khác viết câu đếm "chuyến có cước cao hơn trung bình thành phố" thì SQL Server không chạy mà báo lỗi 130.

Bài này đi qua các dạng subquery, phân biệt `IN` với `EXISTS`, giải thích vì sao `NOT IN` gặp NULL thì cho kết quả sai mà không báo lỗi, rồi viết lại câu đối soát bằng **CTE**: mỗi bước có tên và có một con số để đối chiếu. Phần cuối so hai chỗ lưu kết quả trung gian của SQL Server là bảng tạm `#…` và biến bảng `@…`.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-014 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/mssql-null-case-isnull-coalesce">MS-011 · NULL, CASE, ISNULL và COALESCE</a>, <a href="/articles/mssql-join-apply-cardinality">MS-013 · JOIN, APPLY và số dòng sau khi nối</a>, và mục NULL trong WHERE của <a href="/articles/mssql-select-loc-sap-xep-top-offset">MS-007</a>. Lab: <code>examples/toi-noi-mssql/p02</code> (tệp <code>14-subquery-exists-cte.sql</code>). Sau bài này, bạn tách được một truy vấn đối soát thành các bước có câu kiểm cho từng bước, và chọn được giữa CTE, bảng tạm và biến bảng.</p>
</div>

## 1. Dữ liệu dùng trong bài

Fixture Phần 2 là ảnh chụp lúc **01:00 ngày 31/10/2026 giờ VN**, gồm 3.174 chuyến. Kỳ tháng 10 (đặt trong `[01/10, 31/10)` giờ VN) có **3.168 chuyến đã kết thúc**; 2.707 chuyến hoàn thành với tổng cước 182.287.000 đ. Cột `trips.driver_id` có 297 giá trị NULL (chuyến huỷ trước khi có người nhận và chuyến đang tìm tài xế). Thực thu của một chuyến = Σ charge − Σ refund + Σ adjustment, chỉ tính giao dịch `succeeded`.

```bash
cd examples/toi-noi-mssql
LAB_DB=ToiNoi_thu node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
  p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/14-subquery-exists-cte.sql
```

Kết quả bên dưới chép từ lần chạy trên SQL Server 2022 16.0.4325.4 (RTM-CU28); tệp 14 có 21 dòng `PASS`, 0 `FAIL`. Runner in cột ngăn bằng `|`. Fixture này cho ra đúng từng dòng như fixture PostgreSQL Phần 2, nên các con số đối chiếu được với [PG-014](/articles/postgresql-subquery-exists-cte).

## 2. Subquery vô hướng và subquery tương quan

**Subquery** (truy vấn con) là một `SELECT` trong ngoặc, đặt trong câu lệnh khác. Đặt ở chỗ cần một giá trị (danh sách cột, vế phải của `=`/`>`), nó là subquery **vô hướng** (scalar) và phải trả tối đa 1 dòng × 1 cột. Đặt sau `IN`/`EXISTS`, nó trả một tập dòng. Đặt trong `FROM`, nó là **bảng dẫn xuất** (derived table) như bước gom giao dịch ở MS-013.

Subquery vô hướng đơn giản nhất là một hằng số tính từ dữ liệu, ví dụ cước trung bình của chuyến hoàn thành. Ngay ở đây SQL Server đã khác PostgreSQL một chỗ:

```text
tb_kieu_int|tb_kieu_decimal
-----------|---------------
67339|67339.120797
```

`AVG` trên cột `int` trả về `int`: phần lẻ bị **cắt**, không làm tròn. Muốn trung bình có phần thập phân, ép kiểu trước khi gộp: `AVG(CAST(fare AS decimal(12, 2)))`.

Khi subquery tham chiếu cột của câu bên ngoài, nó là **subquery tương quan** (correlated subquery): về nghĩa, nó được tính lại cho từng dòng bên ngoài. Câu "chuyến hoàn thành có cước cao hơn trung bình *của thành phố mình*":

```sql
SELECT c.code AS tp, COUNT(*) AS tren_tb_thanh_pho
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.status = 'completed'
  AND t.fare > (SELECT AVG(t2.fare) FROM toinoi.trips AS t2
                WHERE t2.city_id = t.city_id AND t2.status = 'completed')
GROUP BY c.code
ORDER BY c.code;
```

```text
tp|tren_tb_thanh_pho
--|-----------------
DN|190
HN|436
SG|636
```

Ở đây `AVG` vẫn cắt phần lẻ (TB của SG là 67.450,98 nhưng subquery trả 67450), song cước luôn là bội của 1.000 đ nên phép so sánh `>` không đổi kết quả. Đừng dựa vào may mắn đó cho cột tiền có số lẻ.

### Lỗi 130: subquery bên trong hàm gộp

Người phân tích muốn có cả tổng số chuyến lẫn số chuyến trên trung bình trong cùng một dòng, nên đưa subquery vào trong `SUM(CASE …)` như cách quen làm với `count(*) FILTER` của PostgreSQL:

```sql
SELECT c.code, COUNT(*) AS hoan_thanh,
       SUM(CASE WHEN t.fare > (SELECT AVG(t2.fare) FROM toinoi.trips AS t2
                               WHERE t2.city_id = t.city_id AND t2.status = 'completed')
                THEN 1 ELSE 0 END) AS tren_tb
FROM toinoi.trips AS t JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.status = 'completed'
GROUP BY c.code;
```

```text
PASS 1b subquery trong hàm gộp — lỗi 130: Cannot perform an aggregate function on an expression containing an aggregate or a subquery.
```

SQL Server không cho đối số của hàm gộp chứa subquery. Cách sửa là tính trung bình thành phố **một lần** trong bảng dẫn xuất rồi JOIN vào, để biểu thức trong `SUM` chỉ còn so hai cột:

```sql
SELECT c.code AS tp,
       COUNT(*)                                        AS hoan_thanh,
       SUM(CASE WHEN t.fare > tb.tb THEN 1 ELSE 0 END) AS tren_tb_thanh_pho,
       MIN(tb.tb)                                      AS tb_thanh_pho
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
JOIN (SELECT city_id, AVG(CAST(fare AS decimal(12, 2))) AS tb
      FROM toinoi.trips WHERE status = 'completed' GROUP BY city_id) AS tb ON tb.city_id = t.city_id
WHERE t.status = 'completed'
GROUP BY c.code
ORDER BY c.code;
```

```text
tp|hoan_thanh|tren_tb_thanh_pho|tb_thanh_pho
--|----------|-----------------|------------
DN|389|190|67907.455012
HN|941|436|66940.488841
SG|1377|636|67450.980392
```

Cùng 190/436/636 như câu tương quan, và câu này còn cho thấy mẫu số.

### Lỗi 512: subquery vô hướng trả hơn một dòng

Subquery vô hướng chỉ được trả tối đa một dòng, nhưng SQL Server kiểm tra điều này **lúc chạy**, trên dữ liệu thật. Lấy "số tiền giao dịch" của chuyến 100042 (thẻ lỗi rồi thu lại, nên có 2 giao dịch):

```sql
SELECT code, (SELECT p.amount FROM toinoi.payments AS p WHERE p.trip_id = t.id) AS so_tien
FROM toinoi.trips AS t WHERE t.code = 100042;
```

```text
PASS 1d subquery vô hướng trả 2 dòng — lỗi 512: Subquery returned more than 1 value. This is not permitted when the subquery follows =, !=, <, <= , >, >= or when the subquery is used as an expression.
```

Cùng câu với chuyến 100041 (1 giao dịch) chạy êm và trả 127000. Đây là loại lỗi nguy hiểm: báo cáo chạy đúng nhiều tháng, rồi hỏng vào ngày đầu tiên có một chuyến thu hai lần. Khi một chuyến có thể có nhiều dòng, nói rõ cần dòng nào (`SUM`, `MAX`, hoặc `TOP (1) … ORDER BY` có tie-breaker) thay vì giả định chỉ có một.

## 3. IN, EXISTS và JOIN cho câu hỏi "có hay không"

Câu hỏi: **bao nhiêu khách từng huỷ ít nhất một chuyến?** Ba cách viết:

```sql
SELECT (SELECT COUNT(*) FROM toinoi.customers AS c
        WHERE c.id IN (SELECT t.customer_id FROM toinoi.trips AS t WHERE t.status = 'cancelled')) AS dung_in,
       (SELECT COUNT(*) FROM toinoi.customers AS c
        WHERE EXISTS (SELECT 1 FROM toinoi.trips AS t
                      WHERE t.customer_id = c.id AND t.status = 'cancelled'))                  AS dung_exists,
       (SELECT COUNT(*) FROM toinoi.customers AS c
        JOIN toinoi.trips AS t ON t.customer_id = c.id AND t.status = 'cancelled')               AS dung_join;
```

```text
dung_in|dung_exists|dung_join
-------|-----------|---------
266|266|461
```

`IN` và `EXISTS` cùng trả **266**: mỗi khách được tính một lần dù huỷ bao nhiêu chuyến. `JOIN` trả **461** vì mỗi chuyến huỷ tạo một dòng, đúng hiện tượng nở dòng của MS-013. Với câu hỏi "có hay không", `EXISTS` nói đúng ý nhất: nó chỉ hỏi có dòng khớp hay không, dừng ở dòng khớp đầu tiên và không bao giờ nhân bản dòng bên ngoài. `SELECT 1` trong `EXISTS` chỉ là quy ước; danh sách cột ở đó không được dùng.

Về hiệu năng, bộ tối ưu của SQL Server thường biến cả `IN (subquery)` lẫn `EXISTS` thành cùng một phép **semi join**, nên đừng chọn giữa hai cách vì tin đồn "EXISTS nhanh hơn". Phần 3 sẽ xem điều đó trong execution plan. Lý do thật để chọn `EXISTS` nằm ở mục sau.

## 4. Bẫy NOT IN gặp NULL

Bộ phận tuyển dụng hỏi tài xế nào chưa chạy chuyến nào:

```sql
SELECT d.code AS not_in
FROM toinoi.drivers AS d
WHERE d.id NOT IN (SELECT t.driver_id FROM toinoi.trips AS t)
ORDER BY d.code;

SELECT d.code AS not_exists, CAST(d.created_at AS varchar(34)) AS created_at
FROM toinoi.drivers AS d
WHERE NOT EXISTS (SELECT 1 FROM toinoi.trips AS t WHERE t.driver_id = d.id)
ORDER BY d.code;
```

```text
not_in
------
not_exists|created_at
----------|----------
8219|2026-10-29 09:00:00.000 +07:00
8220|2026-10-29 09:00:00.000 +07:00
```

`x NOT IN (a, b, NULL)` nghĩa là `x <> a AND x <> b AND x <> NULL`. Vế cuối luôn ra UNKNOWN, nên cả biểu thức không bao giờ ra TRUE, và `WHERE` loại mọi dòng. Danh sách `driver_id` có 297 NULL, nên câu `NOT IN` trả 0 dòng mà không lỗi, không cảnh báo. `NOT EXISTS` hỏi "có chuyến nào của tài xế này không" và không so sánh với NULL, nên trả đúng 8219 và 8220, hai tài xế vừa đăng ký ngày 29/10.

`NOT IN` chỉ an toàn khi subquery chắc chắn không có NULL: thêm `WHERE t.driver_id IS NOT NULL` vào subquery thì câu trả đúng 2 dòng (lab kiểm bằng assertion `3d`). Nhưng "chắc chắn" là một giả định về schema có thể đổi sau này: hôm nay cột `NOT NULL`, mai một migration cho phép NULL, và câu báo cáo cũ lặng lẽ trả 0 dòng. Quy tắc của lab: với subquery, viết `NOT EXISTS`; chỉ dùng `NOT IN` với danh sách hằng số viết tay.

## 5. CTE: đặt tên cho từng bước đối soát

**CTE** (Common Table Expression) là một truy vấn có tên, khai báo bằng `WITH` ngay trước câu lệnh dùng nó. Các CTE trong cùng một `WITH` cách nhau bằng dấu phẩy, và CTE sau được đọc CTE trước. Viết lại câu đối soát thành bốn bước:

| Bước | Ý nghĩa | Con số phải đúng |
|---|---|---|
| `b1` phạm vi | Chuyến đặt trong `[01/10, 31/10)` giờ VN, đã kết thúc | 3.168 (đếm độc lập ở `02-kiem-tra-fixture.sql`) |
| `b2` giao dịch | Thực thu mỗi chuyến, chỉ `succeeded` | Mỗi `trip_id` một dòng |
| `b3` ghép | `b1 LEFT JOIN b2` | **Bằng số dòng của `b1`**, không mất, không nhân |
| `b4` lệch | Thực thu ≠ phải thu | Tổng chênh của `b4` = tổng chênh của cả `b3` |

```sql
WITH b1 AS (
    SELECT t.id, t.code, t.status, t.fare,
           CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END AS phai_thu
    FROM toinoi.trips AS t
    WHERE t.status IN ('completed', 'cancelled')
      AND t.requested_at >= '2026-10-01 00:00:00 +07:00'
      AND t.requested_at <  '2026-10-31 00:00:00 +07:00'
),
b2 AS (
    SELECT p.trip_id,
           SUM(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END) AS thuc_thu
    FROM toinoi.payments AS p
    WHERE p.status = 'succeeded'
    GROUP BY p.trip_id
),
b3 AS (
    SELECT b1.id, b1.code, b1.status, b1.phai_thu, COALESCE(b2.thuc_thu, 0) AS thuc_thu
    FROM b1 LEFT JOIN b2 ON b2.trip_id = b1.id
),
b4 AS (
    SELECT * FROM b3 WHERE thuc_thu <> phai_thu
)
SELECT (SELECT COUNT(*) FROM b1)           AS b1_pham_vi,
       (SELECT COUNT(*) FROM b2)           AS b2_chuyen_co_gd,
       (SELECT COUNT(*) FROM b3)           AS b3_sau_ghep,
       (SELECT COUNT(DISTINCT id) FROM b3) AS b3_chuyen_khac_nhau,
       (SELECT COUNT(*) FROM b4)           AS b4_lech;
```

```text
b1_pham_vi|b2_chuyen_co_gd|b3_sau_ghep|b3_chuyen_khac_nhau|b4_lech
----------|---------------|-----------|-------------------|-------
3168|2688|3168|3168|7
```

Câu kiểm kê này trả lời được hai câu đầu của kế toán trưởng. Phạm vi 3.168 khớp con số đếm độc lập. `b3` có 3.168 dòng và 3.168 chuyến khác nhau: phép ghép không làm mất chuyến (nhờ `LEFT JOIN`) và không nhân đôi chuyến (nhờ `b2` đã gom về một dòng mỗi chuyến). `b2` chỉ có 2.688 chuyến: chuyến huỷ trước khi có tài xế, chuyến cước 0 đ và chuyến chỉ có giao dịch lỗi không có dòng nào ở bước này, và `LEFT JOIN` giữ chúng lại với thực thu 0.

Câu thứ ba cần danh sách và hai dòng tổng. CTE chỉ sống trong **một câu lệnh**, nên câu in danh sách phải viết lại cả chuỗi `WITH` (trong lab là nguyên văn), chỉ thay câu `SELECT` cuối:

```sql
SELECT ma, status, phai_thu, thuc_thu, chenh
FROM (SELECT 1 AS nhom, CAST(code AS nvarchar(10)) AS ma, status, phai_thu, thuc_thu,
             thuc_thu - phai_thu AS chenh FROM b4
      UNION ALL
      SELECT 2, N'TỔNG b4', NULL, SUM(phai_thu), SUM(thuc_thu), SUM(thuc_thu - phai_thu) FROM b4
      UNION ALL
      SELECT 3, N'TỔNG b3', NULL, SUM(CAST(phai_thu AS bigint)), SUM(CAST(thuc_thu AS bigint)),
             SUM(thuc_thu - phai_thu) FROM b3) AS u
ORDER BY nhom, ma;
```

```text
ma|status|phai_thu|thuc_thu|chenh
--|------|--------|--------|-----
100724|completed|76000|0|-76000
101084|completed|98000|0|-98000
101310|completed|85000|170000|85000
101987|cancelled|0|-72000|-72000
102443|completed|54000|45000|-9000
102642|completed|38000|380000|342000
102739|cancelled|0|96000|96000
TỔNG b4|NULL|351000|619000|268000
TỔNG b3|NULL|182287000|182555000|268000
```

Dòng `TỔNG b4` và `TỔNG b3` cùng chênh **+268.000 đ**: bảy chuyến giải thích toàn bộ chênh lệch của tháng, không còn khoản lệch nào nằm ngoài danh sách. `SUM(CAST(… AS bigint))` ở dòng tổng `b3` là thói quen từ MS-004: tổng tiền cả tháng có thể vượt giới hạn của `int`.

Câu kiểm bắt được lỗi **cấu trúc** (mất dòng, nhân dòng, chênh lệch không giải thích được), không bắt được lỗi **định nghĩa**. Nếu ai đó quên điều kiện `status = 'succeeded'` ở `b2`, mọi số kiểm vẫn khớp với nhau nhưng thực thu sai. Vì vậy mỗi bước cần một con số đối chiếu được với nguồn **độc lập** (như 3.168 của bước `b1`), không chỉ với bước liền trước.

## 6. CTE không phải bảng: mỗi lần tham chiếu là một lần tính

Câu kiểm kê ở trên tham chiếu `b1` ba lần (trực tiếp, qua `b3`, qua `b4`). SQL Server **không lưu** kết quả của CTE: CTE giống một view chỉ sống trong một câu lệnh, và bộ tối ưu gộp định nghĩa của nó vào chỗ dùng. T-SQL không có tuỳ chọn kiểu `MATERIALIZED` của PostgreSQL để ép tính một lần.

Thí nghiệm nhỏ cho thấy điều đó. CTE `r` sinh 3 số ngẫu nhiên; nếu `r` được tính một lần rồi dùng lại, tự JOIN `r` với chính nó phải ra 3 dòng:

```sql
WITH r AS (SELECT CHECKSUM(NEWID()) AS v FROM (VALUES (1), (2), (3)) AS s (n))
SELECT COUNT(*) FROM r AS a JOIN r AS b ON a.v = b.v;               -- 0

SELECT CHECKSUM(NEWID()) AS v INTO #r FROM (VALUES (1), (2), (3)) AS s (n);
SELECT COUNT(*) FROM #r AS a JOIN #r AS b ON a.v = b.v;             -- 3
```

```text
PASS 5a CTE tự JOIN (tính 2 lần) = 0
PASS 5b bảng tạm tự JOIN (tính 1 lần) = 3
```

Hai tham chiếu tới CTE là hai lần tính, ra hai bộ số khác nhau. Với dữ liệu thường, hai lần tính cho cùng kết quả nếu không ai ghi vào bảng trong lúc câu lệnh chạy (chuyện đồng thời để Phần 4), nhưng **công** thì nhân lên: bước `b2` gom toàn bộ bảng `payments` có thể chạy lại ở mỗi chỗ dùng. Khi một bước tốn kém được dùng nhiều lần, hoặc nhiều câu lệnh cần cùng một kết quả trung gian, hãy lưu nó lại.

## 7. Bảng tạm #… và biến bảng @…

SQL Server có hai chỗ lưu kết quả trung gian trong phiên, cả hai đều nằm trong database hệ thống `tempdb`:

- **Bảng tạm** (temporary table) có tên bắt đầu bằng `#`, tạo bằng `CREATE TABLE #x` hoặc `SELECT … INTO #x`. Bảng tồn tại tới khi phiên đóng hoặc bị `DROP`.
- **Biến bảng** (table variable) khai báo bằng `DECLARE @x TABLE (…)`, sống trong **batch** khai báo nó, như mọi biến T-SQL.

Lưu bước `b3` vào bảng tạm một lần, thêm khoá chính, rồi các câu kiểm và danh sách đọc lại nó:

```sql
SELECT t.id, t.code, t.status,
       CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END AS phai_thu,
       COALESCE(g.thuc_thu, 0)                                AS thuc_thu
INTO #doi_soat
FROM toinoi.trips AS t
LEFT JOIN (SELECT p.trip_id, SUM(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END) AS thuc_thu
           FROM toinoi.payments AS p WHERE p.status = 'succeeded' GROUP BY p.trip_id) AS g
       ON g.trip_id = t.id
WHERE t.status IN ('completed', 'cancelled')
  AND t.requested_at >= '2026-10-01 00:00:00 +07:00'
  AND t.requested_at <  '2026-10-31 00:00:00 +07:00';

ALTER TABLE #doi_soat ADD PRIMARY KEY (id);
```

```text
so_dong|so_chuyen|so_lech|tong_chenh
-------|---------|-------|----------
3168|3168|7|268000
```

Khoá chính trên bảng tạm vừa là index vừa là một câu kiểm: nếu bước ghép lỡ nhân dòng, `ADD PRIMARY KEY` báo lỗi trùng khoá ngay. (Không đặt tên cho ràng buộc trên bảng tạm: tên ràng buộc phải duy nhất trong `tempdb`, hai phiên cùng chạy script sẽ đụng nhau.)

Bốn khác biệt giữa hai loại, đều đã chạy trong lab:

| | Bảng tạm `#doi_soat` | Biến bảng `@bien` |
|---|---|---|
| Phạm vi | Cả phiên, qua nhiều batch; thấy được trong SQL động và thủ tục do phiên gọi | Chỉ batch khai báo; SQL động báo lỗi 1087 `Must declare the table variable "@bien"` |
| `ROLLBACK` | Huỷ dòng đã ghi | **Không** huỷ: dòng ghi trong transaction vẫn còn |
| Statistics | Có; ước lượng 439 dòng cho điều kiện `phai_thu > 100000`, đúng bằng thực tế | Không có; ước lượng 950,4 (30 % của 3.168) |
| Index | `CREATE INDEX`, `ALTER TABLE … ADD` sau khi tạo | Chỉ khai báo cùng bảng (`PRIMARY KEY`, `UNIQUE`, `INDEX` inline) |

Hàng `ROLLBACK` là chỗ biến bảng có ích thật: ghi nhật ký vào biến bảng trong transaction, rollback xong vẫn đọc được nhật ký để báo lỗi.

```text
bang_tam_sau_rollback|bien_bang_sau_rollback
---------------------|----------------------
0|1
```

### Statistics và ước lượng số dòng

**Statistics** là bản tóm tắt phân bố giá trị của một cột, bộ tối ưu dùng nó để ước lượng một điều kiện giữ lại bao nhiêu dòng, rồi chọn cách JOIN và cấp bộ nhớ theo ước lượng đó. Lab chạy cùng câu `COUNT(*) … WHERE phai_thu > 100000` trên ba đối tượng chứa cùng 3.168 dòng, rồi đọc ước lượng trong execution plan đã cache:

```text
doi_tuong|so_dong_biet|uoc_luong
---------|------------|---------
bảng tạm #doi_soat|3168.0|439.0
biến bảng (mặc định, mức tương thích 160)|3168.0|950.39999999999998
biến bảng, tắt deferred compilation|0.0|1.0
```

- Bảng tạm có statistics tự tạo trên cột `phai_thu`, nên ước lượng **439**, đúng bằng số dòng thật.
- Biến bảng không có statistics. Từ mức tương thích 150 (SQL Server 2019), tính năng *table variable deferred compilation* hoãn biên dịch câu lệnh tới lần chạy đầu, nên bộ tối ưu biết bảng có 3.168 dòng, nhưng vẫn phải **đoán** tỉ lệ cho điều kiện: 950,4.
- Tắt tính năng đó bằng `OPTION (USE HINT ('DISABLE_DEFERRED_COMPILATION_TV'))` thì quay về hành vi cũ: ước lượng **1 dòng** cho bảng 3.168 dòng. Database nâng cấp từ bản cũ, còn ở mức tương thích 140 trở xuống, vẫn chạy theo cách này.

Ước lượng lệch 2 lần không hỏng câu đếm này, nhưng khi biến bảng là một vế JOIN với bảng lớn, nó có thể khiến bộ tối ưu chọn cách JOIN tồi. Mặt kia: statistics của bảng tạm được cập nhật khi dữ liệu đổi nhiều, và mỗi lần như vậy câu lệnh dùng nó có thể phải biên dịch lại (recompile). Biến bảng không có statistics nên không gây recompile vì lý do này. Phần 3 đo các hệ quả đó bằng execution plan.

## 8. Làm trong DBStudio

- Mở `14-subquery-exists-cte.sql`, bấm `Ctrl+Shift+Enter` để chạy cả script: mỗi result set hiện thành một kết quả riêng, các dòng `PASS …` nằm ở tab **Thông điệp**.
- Mỗi tab query giữ **một kết nối riêng**, nên `#doi_soat` tạo ở batch đầu vẫn còn khi bạn đặt con trỏ vào batch sau và bấm `Ctrl+Enter`. Tab khác (phiên khác) không thấy bảng tạm đó; đóng tab là bảng tạm mất. Biến bảng thì mất ngay khi batch khai báo nó chạy xong.
- Muốn gửi danh sách 7 chuyến lệch cho đội tài chính, export result set đó ra CSV hoặc XLSX.

## 9. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| `NOT IN` trả 0 dòng, không lỗi | Subquery có NULL | `NOT EXISTS` |
| Lỗi 512 sau nhiều tháng chạy ổn | Subquery vô hướng gặp chuyến có 2 dòng | Nói rõ dòng nào: `SUM`, `MAX`, `TOP (1) … ORDER BY` |
| Lỗi 130 | Subquery bên trong `SUM`/`COUNT` | Bảng dẫn xuất hoặc CTE rồi JOIN |
| Đếm khách ra 461 thay vì 266 | JOIN 1–n cho câu hỏi "có hay không" | `EXISTS` |
| Trung bình mất phần lẻ | `AVG` trên `int` trả `int` | `AVG(CAST(x AS decimal(…)))` |
| `Invalid object name 'b3'` ở câu thứ hai | CTE chỉ sống trong một câu lệnh | Viết lại `WITH` hoặc lưu vào bảng tạm |
| `Must declare the table variable` trong SQL động | Biến bảng không sang được `sp_executesql` | Bảng tạm, hoặc truyền table-valued parameter |
| Nhật ký mất sau `ROLLBACK` | Ghi vào bảng tạm | Ghi vào biến bảng |

## 10. Đánh đổi

- **CTE hay subquery lồng:** cùng sức mạnh, cùng kế hoạch thực thi trong phần lớn trường hợp; CTE đọc từ trên xuống và có tên để kiểm kê.
- **CTE hay bảng tạm:** CTE không tốn chỗ trong `tempdb` và bộ tối ưu thấy cả câu một lượt; bảng tạm tính một lần, có statistics, có index, kiểm được bằng khoá chính, nhưng tốn ghi vào `tempdb` và có thể gây recompile.
- **Bảng tạm hay biến bảng:** biến bảng gọn và không bị `ROLLBACK` cuốn theo, nhưng không có statistics; bảng tạm ngược lại. Quy tắc của lab: vài chục dòng tham số hoặc nhật ký phải sống sót sau `ROLLBACK` thì dùng biến bảng; vài nghìn dòng trở lên đem JOIN với bảng lớn thì dùng bảng tạm. Đừng chọn theo câu "biến bảng nằm trong RAM": cả hai đều ở `tempdb`.
- **Câu kiểm kê trong cùng script hay để riêng:** để cùng script thì mỗi lần chạy báo cáo là một lần kiểm; cái giá là tính lại các bước (hoặc thêm một bảng tạm).

## 11. So với PostgreSQL

Cùng câu hỏi, cùng dữ liệu với [PG-014](/articles/postgresql-subquery-exists-cte), các con số khớp nhau (266, 8219/8220, 3.168/2.688/3.168/3.168/7, +268.000). Khác biệt thật:

| Chủ đề | PostgreSQL 17 | SQL Server 2022 |
|---|---|---|
| Subquery trả > 1 dòng | Lỗi `21000` | Lỗi 512 |
| Subquery trong hàm gộp | Cho phép (`count(*) FILTER (WHERE x > (SELECT …))`) | Lỗi 130; dùng bảng dẫn xuất + JOIN |
| `avg` trên số nguyên | Trả `numeric` có phần lẻ | `AVG(int)` trả `int`, cắt phần lẻ |
| CTE dùng nhiều lần | Từ bản 12 có `MATERIALIZED` / `NOT MATERIALIZED` | Không có tuỳ chọn; luôn gộp vào câu, muốn tính một lần thì dùng bảng tạm |
| N dòng đầu mỗi nhóm | `LATERAL` | `CROSS APPLY` / `OUTER APPLY` (MS-013) |
| Kết quả trung gian trong phiên | `CREATE TEMP TABLE` | `#bang_tam`, biến bảng `@bien` |

## 12. Bài tập

**Bài 1.** Đội chăm sóc khách hàng muốn gửi lời mời đánh giá cho những khách đã có ít nhất một chuyến hoàn thành nhưng **chưa đánh giá chuyến nào**. Đếm số khách đó; câu lệnh không được nhân dòng.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT COUNT(*) FROM toinoi.customers AS c
WHERE EXISTS (SELECT 1 FROM toinoi.trips AS t
              WHERE t.customer_id = c.id AND t.status = 'completed')
  AND NOT EXISTS (SELECT 1 FROM toinoi.trips AS t
                  JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
                  WHERE t.customer_id = c.id);</code></pre>
<p>Kết quả đã chạy: <b>32</b> khách (assertion <code>BT1</code> trong lab). Hai điều kiện là hai câu hỏi "có hay không" độc lập, nên mỗi vế là một <code>EXISTS</code>; JOIN thẳng <code>customers</code> với <code>trips</code> sẽ nhân dòng theo số chuyến.</p>
</details>

**Bài 2.** Kế toán muốn biết 7 chuyến lệch chia theo trạng thái ra sao. Dùng `#doi_soat` của mục 7 (vẫn còn sau `GO`?), gom theo `status`, và kiểm rằng tổng hai nhóm vẫn là +268.000 đ.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT status, COUNT(*) AS so_chuyen_lech, SUM(thuc_thu - phai_thu) AS chenh
FROM #doi_soat
WHERE thuc_thu &lt;&gt; phai_thu
GROUP BY status
ORDER BY status;</code></pre>
<pre><code class="language-text">status|so_chuyen_lech|chenh
------|--------------|-----
cancelled|2|24000
completed|5|244000</code></pre>
<p>2 chuyến huỷ (+24.000) và 5 chuyến hoàn thành (+244.000), cộng lại +268.000 đ. <code>GO</code> kết thúc batch nên biến <code>@…</code> mất, nhưng bảng tạm sống tới hết phiên, nên câu này chạy được ở batch sau (assertion <code>BT2</code>).</p>
</details>

**Bài 3.** Viết lại câu "tài xế chưa chạy chuyến nào" bằng `EXCEPT` (lấy `drivers.id` trừ đi `trips.driver_id`). Câu này có gặp bẫy NULL như `NOT IN` không? Vì sao?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT d.code
FROM (SELECT id FROM toinoi.drivers
      EXCEPT
      SELECT driver_id FROM toinoi.trips) AS x
JOIN toinoi.drivers AS d ON d.id = x.id
ORDER BY d.code;</code></pre>
<p>Kết quả đã chạy: 8219, 8220 (assertion <code>BT3</code>). <code>EXCEPT</code> là phép tập hợp: nó so sánh <em>dòng</em> và coi hai NULL là giống nhau, không tính <code>id &lt;&gt; NULL</code> như <code>NOT IN</code>. NULL ở vế phải chỉ loại được NULL ở vế trái, mà <code>drivers.id</code> không có NULL. Nhược điểm: <code>EXCEPT</code> chỉ trả các cột được so sánh và tự loại dòng trùng, nên muốn thêm cột khác phải JOIN lại như trên.</p>
</details>

## 13. Đọc tiếp

- Microsoft Learn: [Subqueries (SQL Server)](https://learn.microsoft.com/sql/relational-databases/performance/subqueries), [EXISTS](https://learn.microsoft.com/sql/t-sql/language-elements/exists-transact-sql), [IN](https://learn.microsoft.com/sql/t-sql/language-elements/in-transact-sql), [WITH common_table_expression](https://learn.microsoft.com/sql/t-sql/queries/with-common-table-expression-transact-sql), [table (Transact-SQL)](https://learn.microsoft.com/sql/t-sql/data-types/table-transact-sql) (biến bảng), [CREATE TABLE](https://learn.microsoft.com/sql/t-sql/statements/create-table-transact-sql) (mục bảng tạm), [Intelligent query processing](https://learn.microsoft.com/sql/relational-databases/performance/intelligent-query-processing) (table variable deferred compilation), [EXCEPT and INTERSECT](https://learn.microsoft.com/sql/t-sql/language-elements/set-operators-except-and-intersect-transact-sql).
- Cùng chủ đề trên PostgreSQL: [PG-014 · Subquery, EXISTS và CTE](/articles/postgresql-subquery-exists-cte).
- Lab: `examples/toi-noi-mssql/p02/14-subquery-exists-cte.sql`. Execution plan và statistics được đo kỹ ở MS-021 (Phần 3) và MS-025 (Phần 3).

Bài tiếp theo: [MS-015 · Window function và mệnh đề WINDOW (SQL Server 2022)](/articles/mssql-window-function-window-clause)
