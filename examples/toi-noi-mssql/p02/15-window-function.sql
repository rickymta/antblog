-- P02 · 15-window-function.sql — MS-015 `mssql-window-function-window-clause`
-- Window function và mệnh đề WINDOW (SQL Server 2022): LUỸ KẾ, XẾP HẠNG, SO
-- SÁNH VỚI DÒNG TRƯỚC — với frame (khung) ghi rõ.
--
-- Hàm cửa sổ tính trên một tập dòng "xung quanh" dòng hiện tại mà KHÔNG gộp
-- các dòng lại như GROUP BY. OVER (…) có ba phần:
--   PARTITION BY  — chia nhóm, mỗi nhóm tính riêng
--   ORDER BY      — thứ tự trong nhóm
--   frame         — dòng nào trong nhóm được đưa vào phép tính. Có ORDER BY mà
--                   không ghi frame thì mặc định là
--                   RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW.
-- SQL Server 2022: RANGE chỉ nhận UNBOUNDED và CURRENT ROW (không có
-- "RANGE 6 PRECEDING" hay khoảng thời gian), không có GROUPS.
--
-- Chạy sau 00 → 02. File chỉ đọc dữ liệu; mục 7 đổi mức tương thích của
-- database sang 150 rồi đặt lại 160 ngay trong cùng batch.

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

-- ===========================================================================
-- 1. Doanh thu theo ngày của Đà Nẵng quanh ngày bão 20/10 (0 chuyến)
-- ===========================================================================
-- Lịch đủ 30 ngày bằng GENERATE_SERIES (SQL Server 2022) + LEFT JOIN ⇒ ngày bão
-- có dòng doanh thu 0. Cột tb7_rows_thua tính cùng frame ROWS nhưng trên dữ
-- liệu THƯA (chỉ ngày có chuyến) — ngày 20/10 không có dòng nên ra NULL, và
-- "7 dòng" quanh đó trải trên 8 ngày lịch.
-- Hàm cửa sổ tính trên CẢ THÁNG ở truy vấn trong; truy vấn ngoài mới lọc ngày
-- cần in (cửa sổ tính SAU WHERE của cùng cấp truy vấn).
-- AVG trên int trả int (cắt phần lẻ) ⇒ ép decimal trước khi lấy trung bình.
WITH lich AS (
    SELECT DATEADD(day, value, CAST('2026-10-01' AS date)) AS ngay
    FROM GENERATE_SERIES(0, 29)
),
theo_ngay AS (
    SELECT CAST(SWITCHOFFSET(requested_at, '+07:00') AS date) AS ngay,
           COUNT(*) AS so_chuyen, SUM(fare) AS doanh_thu
    FROM toinoi.trips
    WHERE city_id = 3 AND status = 'completed'
    GROUP BY CAST(SWITCHOFFSET(requested_at, '+07:00') AS date)
),
thua AS (
    SELECT ngay, so_chuyen, doanh_thu,
           AVG(CAST(doanh_thu AS decimal(18, 2))) OVER (ORDER BY ngay ROWS BETWEEN 6 PRECEDING AND CURRENT ROW) AS tb7
    FROM theo_ngay
),
day_du AS (
    SELECT l.ngay,
           COALESCE(d.so_chuyen, 0) AS so_chuyen,
           COALESCE(d.doanh_thu, 0) AS doanh_thu,
           SUM(COALESCE(d.doanh_thu, 0)) OVER (ORDER BY l.ngay ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS luy_ke,
           CAST(ROUND(AVG(CAST(COALESCE(d.doanh_thu, 0) AS decimal(18, 2)))
                      OVER (ORDER BY l.ngay ROWS BETWEEN 6 PRECEDING AND CURRENT ROW), 0) AS int) AS tb7_lich_day,
           CAST(ROUND(d.tb7, 0) AS int) AS tb7_rows_thua
    FROM lich AS l
    LEFT JOIN thua AS d ON d.ngay = l.ngay
)
SELECT ngay, so_chuyen, doanh_thu, luy_ke, tb7_lich_day, tb7_rows_thua
INTO #dn
FROM day_du;

SELECT ngay, so_chuyen, doanh_thu, luy_ke, tb7_lich_day, tb7_rows_thua
FROM #dn
WHERE ngay BETWEEN '2026-10-17' AND '2026-10-24'
ORDER BY ngay;

DECLARE @v nvarchar(400);
SET @v = (SELECT STRING_AGG(CONCAT(FORMAT(ngay, 'dd/MM'), N'=', tb7_lich_day, N'/', COALESCE(CAST(tb7_rows_thua AS varchar(12)), 'NULL')), N' ')
                 WITHIN GROUP (ORDER BY ngay)
          FROM #dn WHERE ngay BETWEEN '2026-10-19' AND '2026-10-21');
EXEC lab.expect_value N'1a TB7 lịch đủ / ROWS thưa 19–21/10', @v, N'19/10=828571/828571 20/10=672571/NULL 21/10=677286/797143';
SET @v = (SELECT CONCAT(COUNT(*), N' ngày, luỹ kế cuối ', MAX(luy_ke)) FROM #dn);
EXEC lab.expect_value N'1b lịch DN', @v, N'30 ngày, luỹ kế cuối 26416000';
GO

-- RANGE với độ lệch (cách PostgreSQL dùng cho "7 ngày lịch") không có ở SQL
-- Server 2022: lỗi 4194.
EXEC lab.expect_error N'1c RANGE 6 PRECEDING',
     N'SELECT ngay, SUM(doanh_thu) OVER (ORDER BY ngay RANGE BETWEEN 6 PRECEDING AND CURRENT ROW)
       FROM (VALUES (CAST(''2026-10-01'' AS date), 1)) AS s (ngay, doanh_thu);',
     4194, N'RANGE is only supported with UNBOUNDED and CURRENT ROW';

-- GROUPS cũng chưa có: lỗi cú pháp.
EXEC lab.expect_error N'1d GROUPS',
     N'SELECT SUM(fare) OVER (ORDER BY code GROUPS BETWEEN 1 PRECEDING AND CURRENT ROW) FROM toinoi.trips;',
     102, N'GROUPS';
GO

-- ===========================================================================
-- 2. Frame mặc định khi thứ tự có HOÀ
-- ===========================================================================
-- 18 chuyến của khách doanh nghiệp; lô API 10/10 09:00:00 có 6 chuyến cùng
-- giây. ORDER BY requested_at không ghi frame ⇒ RANGE: mọi chuyến cùng giây
-- nhận CÙNG giá trị (cả lô). ROWS + tie-breaker `code` ⇒ tăng từng dòng.
SELECT code, CAST(requested_at AS varchar(34)) AS requested_at, fare,
       COUNT(*)  OVER (ORDER BY requested_at)                               AS dem_mac_dinh,
       SUM(fare) OVER (ORDER BY requested_at)                               AS luy_ke_mac_dinh,
       COUNT(*)  OVER (ORDER BY requested_at, code ROWS UNBOUNDED PRECEDING) AS dem_rows,
       SUM(fare) OVER (ORDER BY requested_at, code ROWS UNBOUNDED PRECEDING) AS luy_ke_rows
INTO #dn_doanh_nghiep
FROM toinoi.trips
WHERE customer_id = 'CB000000-0000-4000-8000-000000000001';

SELECT TOP (10) code, requested_at, fare, dem_mac_dinh, luy_ke_mac_dinh, dem_rows, luy_ke_rows
FROM #dn_doanh_nghiep
ORDER BY requested_at, code;

DECLARE @v nvarchar(400);
SET @v = (SELECT STRING_AGG(CONCAT(code, N':', dem_mac_dinh, N'/', dem_rows), N' ') WITHIN GROUP (ORDER BY code)
          FROM #dn_doanh_nghiep WHERE code BETWEEN 100669 AND 101224);
EXEC lab.expect_value N'2a đếm mặc định / ROWS quanh lô 10/10', @v,
     N'100669:3/3 100971:9/4 100972:9/5 100973:9/6 100974:9/7 100975:9/8 100976:9/9 101224:10/10';
SET @v = (SELECT CONCAT(MIN(luy_ke_mac_dinh), N'/', MAX(luy_ke_mac_dinh)) FROM #dn_doanh_nghiep WHERE code BETWEEN 100971 AND 100976);
EXEC lab.expect_value N'2b luỹ kế mặc định cả lô 10/10 (min/max)', @v, N'462000/462000';
SET @v = (SELECT CONCAT(COUNT(*), N' chuyến, tổng ', MAX(luy_ke_rows)) FROM #dn_doanh_nghiep);
EXEC lab.expect_value N'2c khách doanh nghiệp', @v, N'18 chuyến, tổng 809000';
DROP TABLE #dn_doanh_nghiep;
GO

-- ===========================================================================
-- 3. Xếp hạng tài xế theo số chuyến hoàn thành — mệnh đề WINDOW
-- ===========================================================================
-- WINDOW (SQL Server 2022, mức tương thích 160) đặt tên cho một định nghĩa
-- cửa sổ dùng lại nhiều lần. Một cửa sổ có tên còn làm NỀN cho cửa sổ khác:
-- OVER (w ORDER BY …) thêm ORDER BY vào w.
-- row_number đánh số liên tục (hoà thì theo tie-breaker), rank nhảy cóc sau
-- nhóm hoà (1,1,3), dense_rank không nhảy (1,1,2). NTILE(4) chia mỗi thành phố
-- thành 4 nhóm gần bằng nhau.
SELECT c.code AS tp, d.code AS tai_xe, COUNT(*) AS hoan_thanh,
       ROW_NUMBER() OVER (theo_tp ORDER BY COUNT(*) DESC, d.code) AS so_thu_tu,
       RANK()       OVER xep_hang AS hang,
       DENSE_RANK() OVER xep_hang AS hang_lien_tuc,
       NTILE(4)     OVER (theo_tp ORDER BY COUNT(*) DESC, d.code) AS nhom_4
INTO #hang
FROM toinoi.trips AS t
JOIN toinoi.drivers AS d ON d.id = t.driver_id
JOIN toinoi.cities  AS c ON c.id = d.city_id
WHERE t.status = 'completed'
GROUP BY c.code, d.code
WINDOW theo_tp  AS (PARTITION BY c.code),
       xep_hang AS (theo_tp ORDER BY COUNT(*) DESC);

-- Lọc "top 3" ở truy vấn NGOÀI (hàm cửa sổ không dùng được trong WHERE).
SELECT tp, tai_xe, hoan_thanh, so_thu_tu, hang, hang_lien_tuc
FROM #hang
WHERE hang_lien_tuc <= 3
ORDER BY tp, so_thu_tu;

DECLARE @v nvarchar(400);
SET @v = (SELECT STRING_AGG(CONCAT(tp, N'-', tai_xe, N':', hoan_thanh, N':', so_thu_tu, N'/', hang, N'/', hang_lien_tuc), N' ')
                 WITHIN GROUP (ORDER BY tp, so_thu_tu)
          FROM #hang WHERE tp = 'SG' AND hang_lien_tuc <= 3);
EXEC lab.expect_value N'3a SG row_number/rank/dense_rank', @v,
     N'SG-8101:68:1/1/1 SG-8109:68:2/1/1 SG-8120:66:3/3/2 SG-8108:64:4/4/3 SG-8124:64:5/4/3';
SET @v = (SELECT CONCAT(SUM(CASE WHEN tp = 'DN' THEN 1 ELSE 0 END), N'/', SUM(CASE WHEN tp = 'HN' THEN 1 ELSE 0 END), N'/',
                        SUM(CASE WHEN tp = 'SG' THEN 1 ELSE 0 END))
          FROM #hang WHERE hang_lien_tuc <= 3);
EXEC lab.expect_value N'3b số người dense_rank<=3 DN/HN/SG', @v, N'4/3/5';

-- NTILE trên 24 tài xế SG ⇒ 4 nhóm × 6 người.
SELECT nhom_4, COUNT(*) AS so_tai_xe, MIN(hoan_thanh) AS it_nhat, MAX(hoan_thanh) AS nhieu_nhat
FROM #hang WHERE tp = 'SG'
GROUP BY nhom_4 ORDER BY nhom_4;
SET @v = (SELECT STRING_AGG(CONCAT(nhom_4, N':', n), N' ') WITHIN GROUP (ORDER BY nhom_4)
          FROM (SELECT nhom_4, COUNT(*) AS n FROM #hang WHERE tp = 'SG' GROUP BY nhom_4) AS x);
EXEC lab.expect_value N'3c NTILE(4) SG', @v, N'1:6 2:6 3:6 4:6';
DROP TABLE #hang;
GO

-- Hàm cửa sổ trong WHERE: lỗi 4108.
EXEC lab.expect_error N'3d hàm cửa sổ trong WHERE',
     N'SELECT d.code FROM toinoi.drivers AS d WHERE ROW_NUMBER() OVER (ORDER BY d.code) <= 3;',
     4108, N'Windowed functions can only appear in the SELECT or ORDER BY';
GO

-- ===========================================================================
-- 4. So sánh với chuyến TRƯỚC / SAU: LAG và LEAD
-- ===========================================================================
-- Khách 0910000016 — người đi nhiều nhất — 6 chuyến đầu tháng. Dòng đầu không
-- có chuyến trước ⇒ LAG trả NULL.
SELECT TOP (6) t.code, CAST(t.requested_at AS varchar(34)) AS requested_at, t.kind,
       LAG(t.kind) OVER w                                        AS dich_vu_truoc,
       DATEDIFF(minute, LAG(t.requested_at) OVER w, t.requested_at) AS phut_tu_chuyen_truoc,
       LEAD(t.code) OVER w                                       AS chuyen_sau,
       ROW_NUMBER() OVER w                                       AS chuyen_thu
FROM toinoi.trips AS t
JOIN toinoi.customers AS c ON c.id = t.customer_id
WHERE c.phone = '0910000016'
WINDOW w AS (ORDER BY t.requested_at, t.code)
ORDER BY t.requested_at, t.code;

DECLARE @v nvarchar(400);
SET @v = (SELECT STRING_AGG(CONCAT(code, N':', COALESCE(CAST(phut AS varchar(10)), 'NULL')), N' ') WITHIN GROUP (ORDER BY n)
          FROM (SELECT t.code, ROW_NUMBER() OVER w AS n,
                       DATEDIFF(minute, LAG(t.requested_at) OVER w, t.requested_at) AS phut
                FROM toinoi.trips AS t JOIN toinoi.customers AS c ON c.id = t.customer_id
                WHERE c.phone = '0910000016'
                WINDOW w AS (ORDER BY t.requested_at, t.code)) AS x
          WHERE n <= 6);
EXEC lab.expect_value N'4a LAG phút giữa hai chuyến', @v,
     N'100008:NULL 100149:1833 100215:1099 100259:358 100277:239 100369:1146';
GO

-- ===========================================================================
-- 5. FIRST_VALUE, bẫy LAST_VALUE, và IGNORE NULLS (SQL Server 2022)
-- ===========================================================================
-- Frame mặc định chỉ tới dòng hiện tại (và các dòng hoà) ⇒ "giá trị cuối" là
-- chính dòng hiện tại. Muốn chuyến cuối tháng phải mở frame tới cuối nhóm.
SELECT TOP (4) t.code,
       FIRST_VALUE(t.code) OVER (ORDER BY t.requested_at, t.code) AS chuyen_dau,
       LAST_VALUE(t.code)  OVER (ORDER BY t.requested_at, t.code) AS last_value_sai,
       LAST_VALUE(t.code)  OVER (ORDER BY t.requested_at, t.code
                                 ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS chuyen_cuoi
FROM toinoi.trips AS t
JOIN toinoi.customers AS c ON c.id = t.customer_id
WHERE c.phone = '0910000016'
ORDER BY t.requested_at, t.code;

DECLARE @v nvarchar(400);
SET @v = (SELECT STRING_AGG(CONCAT(code, N':', sai, N'/', dung), N' ') WITHIN GROUP (ORDER BY n)
          FROM (SELECT t.code,
                       ROW_NUMBER() OVER (ORDER BY t.requested_at, t.code) AS n,
                       LAST_VALUE(t.code) OVER (ORDER BY t.requested_at, t.code) AS sai,
                       LAST_VALUE(t.code) OVER (ORDER BY t.requested_at, t.code
                                                ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS dung
                FROM toinoi.trips AS t JOIN toinoi.customers AS c ON c.id = t.customer_id
                WHERE c.phone = '0910000016') AS x
          WHERE n <= 3);
EXEC lab.expect_value N'5a LAST_VALUE sai / đúng', @v, N'100008:100008/103123 100149:100149/103123 100215:100215/103123';

-- "So với ngày CÓ dữ liệu gần nhất trước đó" ở Đà Nẵng: trên lịch có NULL
-- (không COALESCE), LAG(…) ngày 21/10 trỏ vào 20/10 = NULL; LAST_VALUE …
-- IGNORE NULLS bỏ qua dòng NULL và lấy 19/10.
SELECT ngay, doanh_thu,
       LAG(doanh_thu) OVER (ORDER BY ngay)                                  AS lag_thuong,
       LAST_VALUE(doanh_thu) IGNORE NULLS
           OVER (ORDER BY ngay ROWS BETWEEN UNBOUNDED PRECEDING AND 1 PRECEDING) AS ngay_co_du_lieu_truoc
INTO #dn_null
FROM (SELECT ngay, NULLIF(doanh_thu, 0) AS doanh_thu FROM #dn) AS s;

SELECT ngay, doanh_thu, lag_thuong, ngay_co_du_lieu_truoc
FROM #dn_null WHERE ngay BETWEEN '2026-10-19' AND '2026-10-22' ORDER BY ngay;

SET @v = (SELECT STRING_AGG(CONCAT(FORMAT(ngay, 'dd/MM'), N'=', COALESCE(CAST(lag_thuong AS varchar(12)), 'NULL'), N'/',
                                   COALESCE(CAST(ngay_co_du_lieu_truoc AS varchar(12)), 'NULL')), N' ') WITHIN GROUP (ORDER BY ngay)
          FROM #dn_null WHERE ngay BETWEEN '2026-10-20' AND '2026-10-21');
EXEC lab.expect_value N'5b LAG / LAST_VALUE IGNORE NULLS 20–21/10', @v, N'20/10=637000/637000 21/10=NULL/637000';

-- LAG … IGNORE NULLS cũng chạy trên bản CU28 của lab: ngày 21/10 lấy 19/10.
SET @v = (SELECT CAST(x.truoc AS varchar(12))
          FROM (SELECT ngay, LAG(doanh_thu) IGNORE NULLS OVER (ORDER BY ngay) AS truoc
                FROM (SELECT ngay, NULLIF(doanh_thu, 0) AS doanh_thu FROM #dn) AS s) AS x
          WHERE x.ngay = '2026-10-21');
EXEC lab.expect_value N'5c LAG IGNORE NULLS 21/10', @v, N'637000';
DROP TABLE #dn_null;
GO

-- ===========================================================================
-- 6. Tỉ trọng: cửa sổ RỖNG `OVER ()` = toàn bộ kết quả, sau GROUP BY
-- ===========================================================================
SELECT c.code AS tp,
       SUM(t.fare)                                                                   AS doanh_thu,
       SUM(SUM(t.fare)) OVER ()                                                      AS tong,
       CAST(ROUND(100.0 * SUM(t.fare) / SUM(SUM(t.fare)) OVER (), 2) AS decimal(5, 2)) AS ty_trong_pct
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.status = 'completed'
GROUP BY c.code
ORDER BY doanh_thu DESC, tp;

DECLARE @v nvarchar(400);
SET @v = (SELECT STRING_AGG(CONCAT(tp, N':', pct), N' ') WITHIN GROUP (ORDER BY tp)
          FROM (SELECT c.code AS tp,
                       CAST(ROUND(100.0 * SUM(t.fare) / SUM(SUM(t.fare)) OVER (), 2) AS decimal(5, 2)) AS pct
                FROM toinoi.trips AS t JOIN toinoi.cities AS c ON c.id = t.city_id
                WHERE t.status = 'completed' GROUP BY c.code) AS x);
EXEC lab.expect_value N'6 tỉ trọng doanh thu', @v, N'DN:14.49 HN:34.56 SG:50.95';
GO

-- ===========================================================================
-- 7. WINDOW và GENERATE_SERIES cần mức tương thích 160
-- ===========================================================================
-- Database của lab tạo trên SQL Server 2022 nên ở mức 160. Database nâng cấp
-- từ bản cũ thường giữ mức cũ: thử ở 150 rồi đặt lại 160.
-- Mức tương thích mới áp dụng từ BATCH SAU câu ALTER (đã thử: đổi và dùng
-- trong cùng batch thì câu sau vẫn chạy theo mức cũ), nên mỗi bước một batch.
ALTER DATABASE CURRENT SET COMPATIBILITY_LEVEL = 150;
GO
EXEC lab.expect_error N'7a WINDOW ở mức 150',
     N'SELECT TOP (1) code, ROW_NUMBER() OVER w AS n FROM toinoi.trips WINDOW w AS (ORDER BY code);',
     102, N'Incorrect syntax';
EXEC lab.expect_error N'7b GENERATE_SERIES ở mức 150',
     N'SELECT TOP (1) value FROM GENERATE_SERIES(1, 3);',
     208, N'GENERATE_SERIES';
-- IGNORE NULLS và DATE_BUCKET không phụ thuộc mức tương thích (vẫn chạy ở 150).
DECLARE @v nvarchar(100) = (
    SELECT TOP (1) CONCAT(LAST_VALUE(x) IGNORE NULLS OVER (ORDER BY n ROWS UNBOUNDED PRECEDING), N'/',
                          FORMAT(DATE_BUCKET(minute, 15, CAST('2026-10-15 07:44' AS datetime2(0))), 'HH:mm'))
    FROM (VALUES (1, 10), (2, NULL)) AS s (n, x)
    ORDER BY n DESC);
EXEC lab.expect_value N'7c IGNORE NULLS + DATE_BUCKET ở mức 150', @v, N'10/07:30';
GO
ALTER DATABASE CURRENT SET COMPATIBILITY_LEVEL = 160;
GO
DECLARE @v nvarchar(10) = (SELECT CAST(compatibility_level AS nvarchar(10)) FROM sys.databases WHERE name = DB_NAME());
EXEC lab.expect_value N'7d mức tương thích sau khi đặt lại', @v, N'160';
GO

-- ===========================================================================
-- 8. Lời giải bài tập
-- ===========================================================================
-- Bài 1. Một chuyến cước cao nhất mỗi thành phố: ROW_NUMBER (đúng 1 dòng, hoà
--        thì mã nhỏ hơn) và RANK (mọi chuyến hoà hạng 1).
DECLARE @v nvarchar(400);
SET @v = (SELECT STRING_AGG(CONCAT(tp, N':', code), N' ') WITHIN GROUP (ORDER BY tp, code)
          FROM (SELECT c.code AS tp, t.code,
                       ROW_NUMBER() OVER (PARTITION BY t.city_id ORDER BY t.fare DESC, t.code) AS n
                FROM toinoi.trips AS t JOIN toinoi.cities AS c ON c.id = t.city_id
                WHERE t.status = 'completed') AS x
          WHERE n = 1);
EXEC lab.expect_value N'BT1 ROW_NUMBER = 1', @v, N'DN:101355 HN:101699 SG:101601';
SET @v = (SELECT STRING_AGG(CONCAT(c.code, N':', m.cuoc_max), N' ') WITHIN GROUP (ORDER BY c.code)
          FROM toinoi.cities AS c
          CROSS APPLY (SELECT MAX(fare) AS cuoc_max FROM toinoi.trips AS t
                       WHERE t.city_id = c.id AND t.status = 'completed') AS m);
EXEC lab.expect_value N'BT1 cước tối đa mỗi thành phố', @v, N'DN:150000 HN:149000 SG:150000';
SET @v = (SELECT STRING_AGG(CONCAT(tp, N':', code), N' ') WITHIN GROUP (ORDER BY tp, code)
          FROM (SELECT c.code AS tp, t.code,
                       RANK() OVER (PARTITION BY t.city_id ORDER BY t.fare DESC) AS h
                FROM toinoi.trips AS t JOIN toinoi.cities AS c ON c.id = t.city_id
                WHERE t.status = 'completed') AS x
          WHERE h = 1);
EXEC lab.expect_value N'BT1 RANK = 1', @v,
     N'DN:101355 DN:101431 HN:101699 HN:101913 HN:102014 HN:102037 SG:101601 SG:101725 SG:101741 SG:101892 SG:101984';

-- Bài 2. Khách 0910000016: khoảng nghỉ dài nhất giữa hai chuyến liên tiếp
--        trong tháng (LEAD lấy giờ đặt của chuyến SAU).
SET @v = (SELECT TOP (1) CONCAT(code, N'→', chuyen_sau, N':', phut, N' phút')
          FROM (SELECT t.code,
                       LEAD(t.code) OVER w AS chuyen_sau,
                       DATEDIFF(minute, t.requested_at, LEAD(t.requested_at) OVER w) AS phut
                FROM toinoi.trips AS t JOIN toinoi.customers AS c ON c.id = t.customer_id
                WHERE c.phone = '0910000016'
                WINDOW w AS (ORDER BY t.requested_at, t.code)) AS x
          WHERE phut IS NOT NULL
          ORDER BY phut DESC, code);
EXEC lab.expect_value N'BT2 khoảng nghỉ dài nhất', @v, N'102372→102664:3477 phút';

-- Bài 3. Tỉ trọng từng dịch vụ TRONG mỗi thành phố (cửa sổ có PARTITION BY).
SET @v = (SELECT STRING_AGG(CONCAT(tp, N'-', kind, N':', pct), N' ') WITHIN GROUP (ORDER BY tp, kind)
          FROM (SELECT c.code AS tp, t.kind,
                       CAST(ROUND(100.0 * SUM(t.fare) / SUM(SUM(t.fare)) OVER (PARTITION BY c.code), 2) AS decimal(5, 2)) AS pct
                FROM toinoi.trips AS t JOIN toinoi.cities AS c ON c.id = t.city_id
                WHERE t.status = 'completed' GROUP BY c.code, t.kind) AS x
          WHERE tp = 'DN');
EXEC lab.expect_value N'BT3 tỉ trọng dịch vụ ở DN', @v, N'DN-food:56.41 DN-parcel:12.80 DN-ride:30.79';

DROP TABLE #dn;
GO
