-- P02 · 14-subquery-exists-cte.sql — MS-014 `mssql-subquery-exists-cte-temp-table`
-- Subquery, EXISTS, CTE và bảng tạm: tách truy vấn đối soát thành các BƯỚC
-- KIỂM CHỨNG ĐƯỢC.
--
-- Chạy sau 00 → 02 (npm run p02 làm sẵn). File chỉ ĐỌC dữ liệu: bảng tạm
-- #… và biến bảng @… mất khi phiên kết thúc; đoạn ghi duy nhất nằm trong
-- BEGIN TRAN … ROLLBACK và chỉ ghi vào bảng tạm.

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

-- ===========================================================================
-- 1. Subquery vô hướng và subquery tương quan
-- ===========================================================================
DECLARE @v nvarchar(100);

-- AVG trên cột int trả về int: phần lẻ bị CẮT, không làm tròn.
SELECT AVG(fare)                           AS tb_kieu_int,
       AVG(CAST(fare AS decimal(12, 2)))   AS tb_kieu_decimal
FROM toinoi.trips
WHERE status = 'completed';

SET @v = (SELECT CONCAT(AVG(fare), N' / ', AVG(CAST(fare AS decimal(12, 2))))
          FROM toinoi.trips WHERE status = 'completed');
EXEC lab.expect_value N'1a AVG int / decimal', @v, N'67339 / 67339.120797';

-- Subquery TƯƠNG QUAN: "chuyến hoàn thành có cước cao hơn trung bình của
-- thành phố mình". Đặt subquery trong WHERE thì chạy được…
SELECT c.code AS tp, COUNT(*) AS tren_tb_thanh_pho
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.status = 'completed'
  AND t.fare > (SELECT AVG(t2.fare) FROM toinoi.trips AS t2
                WHERE t2.city_id = t.city_id AND t2.status = 'completed')
GROUP BY c.code
ORDER BY c.code;

-- … nhưng đặt subquery bên trong SUM thì SQL Server từ chối (lỗi 130).
EXEC lab.expect_error N'1b subquery trong hàm gộp',
     N'SELECT c.code, COUNT(*) AS hoan_thanh,
             SUM(CASE WHEN t.fare > (SELECT AVG(t2.fare) FROM toinoi.trips AS t2
                                     WHERE t2.city_id = t.city_id AND t2.status = ''completed'')
                      THEN 1 ELSE 0 END) AS tren_tb
      FROM toinoi.trips AS t JOIN toinoi.cities AS c ON c.id = t.city_id
      WHERE t.status = ''completed''
      GROUP BY c.code;',
     130, N'Cannot perform an aggregate function';

-- Sửa: tính trung bình thành phố ở một bảng dẫn xuất rồi JOIN, phép so sánh
-- trong SUM chỉ còn là cột.
SELECT c.code AS tp,
       COUNT(*)                                          AS hoan_thanh,
       SUM(CASE WHEN t.fare > tb.tb THEN 1 ELSE 0 END)   AS tren_tb_thanh_pho,
       MIN(tb.tb)                                        AS tb_thanh_pho
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
JOIN (SELECT city_id, AVG(CAST(fare AS decimal(12, 2))) AS tb
      FROM toinoi.trips WHERE status = 'completed' GROUP BY city_id) AS tb ON tb.city_id = t.city_id
WHERE t.status = 'completed'
GROUP BY c.code
ORDER BY c.code;

SET @v = (SELECT STRING_AGG(CONCAT(x.tp, N':', x.n), N' ') WITHIN GROUP (ORDER BY x.tp)
          FROM (SELECT c.code AS tp, SUM(CASE WHEN t.fare > tb.tb THEN 1 ELSE 0 END) AS n
                FROM toinoi.trips AS t
                JOIN toinoi.cities AS c ON c.id = t.city_id
                JOIN (SELECT city_id, AVG(CAST(fare AS decimal(12, 2))) AS tb
                      FROM toinoi.trips WHERE status = 'completed' GROUP BY city_id) AS tb ON tb.city_id = t.city_id
                WHERE t.status = 'completed'
                GROUP BY c.code) AS x);
EXEC lab.expect_value N'1c trên TB thành phố', @v, N'DN:190 HN:436 SG:636';

-- Subquery vô hướng trả HƠN 1 dòng: lỗi lúc CHẠY (512), không phải lúc viết.
-- Chuyến 100042 có 2 giao dịch (thẻ lỗi rồi thu lại).
EXEC lab.expect_error N'1d subquery vô hướng trả 2 dòng',
     N'SELECT code, (SELECT p.amount FROM toinoi.payments AS p WHERE p.trip_id = t.id) AS so_tien
       FROM toinoi.trips AS t WHERE t.code = 100042;',
     512, N'Subquery returned more than 1 value';

-- Cùng câu với chuyến chỉ có 1 giao dịch thì chạy êm — lỗi chỉ lộ ra khi dữ
-- liệu đổi.
SET @v = (SELECT (SELECT p.amount FROM toinoi.payments AS p WHERE p.trip_id = t.id)
          FROM toinoi.trips AS t WHERE t.code = 100041);
EXEC lab.expect_value N'1e cùng câu, chuyến 100041 (1 giao dịch)', @v, N'127000';
GO

-- ===========================================================================
-- 2. IN, EXISTS và JOIN cho câu hỏi "có hay không": khách từng huỷ chuyến
-- ===========================================================================
SELECT (SELECT COUNT(*) FROM toinoi.customers AS c
        WHERE c.id IN (SELECT t.customer_id FROM toinoi.trips AS t WHERE t.status = 'cancelled')) AS dung_in,
       (SELECT COUNT(*) FROM toinoi.customers AS c
        WHERE EXISTS (SELECT 1 FROM toinoi.trips AS t
                      WHERE t.customer_id = c.id AND t.status = 'cancelled'))                  AS dung_exists,
       (SELECT COUNT(*) FROM toinoi.customers AS c
        JOIN toinoi.trips AS t ON t.customer_id = c.id AND t.status = 'cancelled')               AS dung_join;

DECLARE @v nvarchar(100) = (
    SELECT CONCAT(
        (SELECT COUNT(*) FROM toinoi.customers AS c
         WHERE c.id IN (SELECT t.customer_id FROM toinoi.trips AS t WHERE t.status = 'cancelled')), N' / ',
        (SELECT COUNT(*) FROM toinoi.customers AS c
         WHERE EXISTS (SELECT 1 FROM toinoi.trips AS t WHERE t.customer_id = c.id AND t.status = 'cancelled')), N' / ',
        (SELECT COUNT(*) FROM toinoi.customers AS c
         JOIN toinoi.trips AS t ON t.customer_id = c.id AND t.status = 'cancelled')));
EXEC lab.expect_value N'2 IN / EXISTS / JOIN', @v, N'266 / 266 / 461';
GO

-- ===========================================================================
-- 3. BẪY NOT IN + NULL: "tài xế chưa chạy chuyến nào"
-- ===========================================================================
-- trips.driver_id có 297 giá trị NULL ⇒ `id NOT IN (…, NULL, …)` không bao giờ
-- ra TRUE ⇒ 0 dòng, không lỗi, không cảnh báo.
DECLARE @v nvarchar(100);
SET @v = (SELECT COUNT(*) FROM toinoi.trips WHERE driver_id IS NULL);
EXEC lab.expect_value N'3a driver_id NULL', @v, N'297';

SELECT d.code AS not_in
FROM toinoi.drivers AS d
WHERE d.id NOT IN (SELECT t.driver_id FROM toinoi.trips AS t)
ORDER BY d.code;

SET @v = (SELECT COUNT(*) FROM toinoi.drivers AS d
          WHERE d.id NOT IN (SELECT t.driver_id FROM toinoi.trips AS t));
EXEC lab.expect_value N'3b NOT IN', @v, N'0';

-- NOT EXISTS: đúng với mọi trường hợp — 8219, 8220 vừa đăng ký 29/10.
SELECT d.code AS not_exists, CAST(d.created_at AS varchar(34)) AS created_at
FROM toinoi.drivers AS d
WHERE NOT EXISTS (SELECT 1 FROM toinoi.trips AS t WHERE t.driver_id = d.id)
ORDER BY d.code;

SET @v = (SELECT STRING_AGG(CAST(d.code AS varchar(10)), ',') WITHIN GROUP (ORDER BY d.code)
          FROM toinoi.drivers AS d
          WHERE NOT EXISTS (SELECT 1 FROM toinoi.trips AS t WHERE t.driver_id = d.id));
EXEC lab.expect_value N'3c NOT EXISTS', @v, N'8219,8220';

-- NOT IN chỉ an toàn khi subquery chắc chắn không có NULL.
SET @v = (SELECT COUNT(*) FROM toinoi.drivers AS d
          WHERE d.id NOT IN (SELECT t.driver_id FROM toinoi.trips AS t WHERE t.driver_id IS NOT NULL));
EXEC lab.expect_value N'3d NOT IN + IS NOT NULL', @v, N'2';
GO

-- ===========================================================================
-- 4. CTE: đặt tên cho từng bước đối soát tháng 10
-- ===========================================================================
--   b1  phạm vi    : chuyến đặt trong [01/10, 31/10) giờ VN, đã kết thúc
--   b2  giao dịch  : thực thu mỗi chuyến = Σ charge − Σ refund + Σ adjustment (succeeded)
--   b3  ghép       : b1 LEFT JOIN b2 — phải giữ ĐÚNG số dòng của b1
--   b4  lệch       : thực thu ≠ phải thu (completed: cước; cancelled: 0)
-- Câu kiểm kê in số dòng từng bước.
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

-- Danh sách lệch và hai dòng tổng. CTE chỉ sống trong MỘT câu lệnh, nên câu
-- này phải viết lại cả chuỗi.
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
SELECT ma, status, phai_thu, thuc_thu, chenh
FROM (SELECT 1 AS nhom, CAST(code AS nvarchar(10)) AS ma, status, phai_thu, thuc_thu,
             thuc_thu - phai_thu AS chenh FROM b4
      UNION ALL
      SELECT 2, N'TỔNG b4', NULL, SUM(phai_thu), SUM(thuc_thu), SUM(thuc_thu - phai_thu) FROM b4
      UNION ALL
      SELECT 3, N'TỔNG b3', NULL, SUM(CAST(phai_thu AS bigint)), SUM(CAST(thuc_thu AS bigint)),
             SUM(thuc_thu - phai_thu) FROM b3) AS u
ORDER BY nhom, ma;
GO

-- Kiểm kê bằng assertion (cùng chuỗi CTE).
DECLARE @v nvarchar(200);
WITH b1 AS (
    SELECT t.id, t.code, t.status,
           CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END AS phai_thu
    FROM toinoi.trips AS t
    WHERE t.status IN ('completed', 'cancelled')
      AND t.requested_at >= '2026-10-01 00:00:00 +07:00'
      AND t.requested_at <  '2026-10-31 00:00:00 +07:00'
),
b2 AS (
    SELECT p.trip_id, SUM(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END) AS thuc_thu
    FROM toinoi.payments AS p WHERE p.status = 'succeeded' GROUP BY p.trip_id
),
b3 AS (
    SELECT b1.id, b1.code, b1.status, b1.phai_thu, COALESCE(b2.thuc_thu, 0) AS thuc_thu
    FROM b1 LEFT JOIN b2 ON b2.trip_id = b1.id
),
b4 AS (SELECT * FROM b3 WHERE thuc_thu <> phai_thu)
SELECT @v = CONCAT((SELECT COUNT(*) FROM b1), N'/', (SELECT COUNT(*) FROM b2), N'/',
                   (SELECT COUNT(*) FROM b3), N'/', (SELECT COUNT(DISTINCT id) FROM b3), N'/',
                   (SELECT COUNT(*) FROM b4), N' chênh b4=', (SELECT SUM(thuc_thu - phai_thu) FROM b4),
                   N' chênh b3=', (SELECT SUM(thuc_thu - phai_thu) FROM b3),
                   N' mã=', (SELECT STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY code) FROM b4));
EXEC lab.expect_value N'4 kiểm kê b1/b2/b3/b3 khác nhau/b4',
     @v, N'3168/2688/3168/3168/7 chênh b4=268000 chênh b3=268000 mã=100724,101084,101310,101987,102443,102642,102739';
GO

-- ===========================================================================
-- 5. CTE KHÔNG được lưu lại: mỗi lần tham chiếu là một lần tính
-- ===========================================================================
-- CTE `r` sinh 3 số ngẫu nhiên. Nếu SQL Server tính `r` một lần rồi dùng lại,
-- tự JOIN r với chính nó trên v phải ra 3 dòng. Thực tế: 0 — hai tham chiếu là
-- hai lần tính, ra hai bộ số khác nhau. (Xác suất hai số CHECKSUM(NEWID())
-- trùng nhau là khoảng 1/4 tỉ — không đáng kể.)
DECLARE @v nvarchar(100);
WITH r AS (SELECT CHECKSUM(NEWID()) AS v FROM (VALUES (1), (2), (3)) AS s (n))
SELECT @v = COUNT(*) FROM r AS a JOIN r AS b ON a.v = b.v;
EXEC lab.expect_value N'5a CTE tự JOIN (tính 2 lần)', @v, N'0';

-- Cùng dữ liệu, lưu vào bảng tạm trước: tính đúng một lần.
SELECT CHECKSUM(NEWID()) AS v INTO #r FROM (VALUES (1), (2), (3)) AS s (n);
SELECT @v = COUNT(*) FROM #r AS a JOIN #r AS b ON a.v = b.v;
EXEC lab.expect_value N'5b bảng tạm tự JOIN (tính 1 lần)', @v, N'3';
DROP TABLE #r;
GO

-- ===========================================================================
-- 6. Bảng tạm #… và biến bảng @…
-- ===========================================================================
-- 6a. Lưu bước b3 vào bảng tạm MỘT lần, các câu kiểm và danh sách đọc lại nó.
DECLARE @v nvarchar(200);

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

SELECT COUNT(*) AS so_dong, COUNT(DISTINCT id) AS so_chuyen,
       SUM(CASE WHEN thuc_thu <> phai_thu THEN 1 ELSE 0 END) AS so_lech,
       SUM(thuc_thu - phai_thu) AS tong_chenh
FROM #doi_soat;

SET @v = (SELECT CONCAT(COUNT(*), N'/', SUM(CASE WHEN thuc_thu <> phai_thu THEN 1 ELSE 0 END), N'/',
                        SUM(thuc_thu - phai_thu)) FROM #doi_soat);
EXEC lab.expect_value N'6a #doi_soat dòng/lệch/chênh', @v, N'3168/7/268000';

-- 6b. Phạm vi: bảng tạm thấy được trong SQL động và thủ tục con do phiên gọi;
--     biến bảng chỉ thấy trong batch khai báo nó.
DECLARE @bien TABLE (id uniqueidentifier PRIMARY KEY, phai_thu int, thuc_thu int);
INSERT INTO @bien (id, phai_thu, thuc_thu) SELECT id, phai_thu, thuc_thu FROM #doi_soat;

CREATE TABLE #dem (n int);
INSERT INTO #dem EXEC sys.sp_executesql N'SELECT COUNT(*) FROM #doi_soat;';
SET @v = (SELECT n FROM #dem);
EXEC lab.expect_value N'6b SQL động đọc được #doi_soat', @v, N'3168';
DROP TABLE #dem;

EXEC lab.expect_error N'6c SQL động không thấy biến bảng',
     N'SELECT COUNT(*) FROM @bien;', 1087, N'Must declare the table variable';

-- 6d. Giao dịch: ROLLBACK huỷ dòng ghi vào bảng tạm, KHÔNG huỷ dòng ghi vào
--     biến bảng.
CREATE TABLE #nhat_ky (ghi_chu nvarchar(50));
DECLARE @nhat_ky TABLE (ghi_chu nvarchar(50));
BEGIN TRANSACTION;
    INSERT INTO #nhat_ky VALUES (N'bắt đầu đối soát');
    INSERT INTO @nhat_ky VALUES (N'bắt đầu đối soát');
ROLLBACK TRANSACTION;
SELECT (SELECT COUNT(*) FROM #nhat_ky) AS bang_tam_sau_rollback,
       (SELECT COUNT(*) FROM @nhat_ky) AS bien_bang_sau_rollback;
SET @v = CONCAT((SELECT COUNT(*) FROM #nhat_ky), N' / ', (SELECT COUNT(*) FROM @nhat_ky));
EXEC lab.expect_value N'6d sau ROLLBACK: #tạm / @biến', @v, N'0 / 1';
DROP TABLE #nhat_ky;
GO

-- 6e. Statistics và ước lượng số dòng. Cùng 3.168 dòng, cùng điều kiện
--     "chênh lệch khác 0 hoặc phải thu > 100.000": đọc ước lượng trong
--     execution plan đã cache (EstimateRows của toán tử quét bảng).
--     Câu chạy bằng SQL động có đánh dấu riêng theo tên database để tìm lại.
DECLARE @danh_dau nvarchar(100) = N'/*ms014-' + DB_NAME();
DECLARE @sql nvarchar(max) = N'
DECLARE @bien TABLE (id uniqueidentifier PRIMARY KEY, phai_thu int, thuc_thu int);
INSERT INTO @bien SELECT id, phai_thu, thuc_thu FROM #doi_soat;
SELECT COUNT(*) ' + @danh_dau + N'-bien*/ FROM @bien WHERE phai_thu > 100000;
SELECT COUNT(*) ' + @danh_dau + N'-bien-cu*/ FROM @bien WHERE phai_thu > 100000
OPTION (USE HINT (''DISABLE_DEFERRED_COMPILATION_TV''));
SELECT COUNT(*) ' + @danh_dau + N'-tam*/ FROM #doi_soat WHERE phai_thu > 100000;';
EXEC sys.sp_executesql @sql;

WITH XMLNAMESPACES (DEFAULT 'http://schemas.microsoft.com/sqlserver/2004/07/showplan'),
ke_hoach AS (
    SELECT SUBSTRING(st.text, qs.statement_start_offset / 2 + 1,
                     (CASE qs.statement_end_offset WHEN -1 THEN DATALENGTH(st.text)
                           ELSE qs.statement_end_offset END - qs.statement_start_offset) / 2 + 1) AS cau,
           CAST(tp.query_plan AS xml) AS kh
    FROM sys.dm_exec_query_stats AS qs
    CROSS APPLY sys.dm_exec_sql_text(qs.sql_handle) AS st
    CROSS APPLY sys.dm_exec_text_query_plan(qs.plan_handle, qs.statement_start_offset, qs.statement_end_offset) AS tp
    WHERE st.text LIKE N'%' + @danh_dau + N'%' AND st.text NOT LIKE N'%XMLNAMESPACES%'
)
SELECT DISTINCT   -- chạy lab nhiều lần trên cùng database ⇒ nhiều bản kế hoạch giống nhau
       CASE WHEN cau LIKE N'%-bien-cu%' THEN N'biến bảng, tắt deferred compilation'
            WHEN cau LIKE N'%-bien*/%' THEN N'biến bảng (mặc định, mức tương thích 160)'
            ELSE N'bảng tạm #doi_soat' END                                                   AS doi_tuong,
       kh.value('(//RelOp[@LogicalOp="Clustered Index Scan" or @LogicalOp="Table Scan"]/@TableCardinality)[1]', 'float') AS so_dong_biet,
       kh.value('(//RelOp[@LogicalOp="Clustered Index Scan" or @LogicalOp="Table Scan"]/@EstimateRows)[1]', 'float')     AS uoc_luong
INTO #uoc_luong
FROM ke_hoach
WHERE cau LIKE N'SELECT COUNT(*)%';

SELECT doi_tuong, so_dong_biet, uoc_luong FROM #uoc_luong ORDER BY doi_tuong;

DECLARE @v nvarchar(200) = (SELECT STRING_AGG(CONCAT(CAST(so_dong_biet AS int), N'→', CAST(ROUND(uoc_luong, 1) AS decimal(9, 1))), N' | ')
                                   WITHIN GROUP (ORDER BY doi_tuong)
                            FROM #uoc_luong);
EXEC lab.expect_value N'6e ước lượng: bảng tạm | biến bảng | biến bảng cũ', @v, N'3168→439.0 | 3168→950.4 | 0→1.0';
DROP TABLE #uoc_luong;
GO

-- ===========================================================================
-- 7. Lời giải bài tập
-- ===========================================================================
-- Bài 1. Khách có ít nhất một chuyến hoàn thành nhưng chưa đánh giá chuyến nào.
DECLARE @v nvarchar(200);
SET @v = (SELECT COUNT(*) FROM toinoi.customers AS c
          WHERE EXISTS (SELECT 1 FROM toinoi.trips AS t
                        WHERE t.customer_id = c.id AND t.status = 'completed')
            AND NOT EXISTS (SELECT 1 FROM toinoi.trips AS t
                            JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
                            WHERE t.customer_id = c.id));
EXEC lab.expect_value N'BT1 khách hoàn thành nhưng chưa đánh giá', @v, N'32';

-- Bài 2. Gom các chuyến lệch theo trạng thái, đọc lại #doi_soat của mục 6:
--        GO kết thúc batch (biến @… mất) nhưng bảng tạm sống tới hết PHIÊN.
SELECT status, COUNT(*) AS so_chuyen_lech, SUM(thuc_thu - phai_thu) AS chenh
FROM #doi_soat
WHERE thuc_thu <> phai_thu
GROUP BY status
ORDER BY status;

SET @v = (SELECT STRING_AGG(CONCAT(status, N':', n, N':', chenh), N' ') WITHIN GROUP (ORDER BY status)
          FROM (SELECT status, COUNT(*) AS n, SUM(thuc_thu - phai_thu) AS chenh
                FROM #doi_soat WHERE thuc_thu <> phai_thu GROUP BY status) AS x);
EXEC lab.expect_value N'BT2 lệch theo trạng thái', @v, N'cancelled:2:24000 completed:5:244000';

-- Bài 3. "Tài xế chưa chạy chuyến nào" bằng EXCEPT: phép tập hợp coi hai NULL
--        là bằng nhau và chỉ trả giá trị của vế trái, nên không gặp bẫy NOT IN.
SET @v = (SELECT STRING_AGG(CAST(d.code AS varchar(10)), ',') WITHIN GROUP (ORDER BY d.code)
          FROM (SELECT id FROM toinoi.drivers EXCEPT SELECT driver_id FROM toinoi.trips) AS x
          JOIN toinoi.drivers AS d ON d.id = x.id);
EXEC lab.expect_value N'BT3 EXCEPT', @v, N'8219,8220';

DROP TABLE #doi_soat;
GO
