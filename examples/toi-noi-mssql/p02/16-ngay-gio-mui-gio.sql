-- P02 · 16-ngay-gio-mui-gio.sql — MS-016 `mssql-ngay-gio-at-time-zone-date-bucket`
-- Ngày giờ, AT TIME ZONE, DATETRUNC và DATE_BUCKET: bộ lọc NGÀY NGHIỆP VỤ
-- theo giờ Việt Nam có kiểm tra HAI BIÊN, và gom chuyến theo khung 15 phút.
--
-- Nhắc lại MS-004: datetimeoffset lưu giờ trên đồng hồ KÈM độ lệch so với UTC
-- và so sánh theo khoảnh khắc (quy về UTC). SQL Server không có "múi giờ của
-- phiên"; chuỗi không ghi độ lệch khi đổi sang datetimeoffset được gán +00:00.
-- "Ngày 15/10" của Tới Nơi là ngày theo giờ Việt Nam (+07:00, không có giờ mùa
-- hè): từ 2026-10-15 00:00 +07:00 tới TRƯỚC 2026-10-16 00:00 +07:00.
--
-- Fixture có 5 chuyến cài sát biên (02-kiem-tra-fixture.sql, H5):
--   101489  15/10 00:00:00     101502  15/10 06:59:59     101503  15/10 07:00:00
--   101594  15/10 23:59:59.5   101595  16/10 00:00:00   (giờ VN)
--
-- Chạy sau 00 → 02. File chỉ đọc dữ liệu; thí nghiệm ghi vào bảng tạm.
-- DATEFIRST và LANGUAGE được đặt tường minh để kết quả không phụ thuộc login.

SET NOCOUNT ON;
SET LANGUAGE us_english;   -- kéo theo DATEFIRST 7 (tuần bắt đầu Chủ nhật)
USE [$(LAB_DB)];
GO

-- ===========================================================================
-- 1. Một khoảnh khắc, nhiều cách biểu diễn
-- ===========================================================================
SELECT code,
       CAST(requested_at AS varchar(34))                                    AS nhu_da_luu,
       CAST(SWITCHOFFSET(requested_at, '+00:00') AS varchar(34))            AS switchoffset_utc,
       CAST(requested_at AT TIME ZONE 'UTC' AS varchar(34))                 AS at_time_zone_utc,
       CAST(requested_at AT TIME ZONE 'SE Asia Standard Time' AS varchar(34)) AS at_time_zone_vn,
       DATEPART(TZOFFSET, requested_at)                                     AS do_lech_phut,
       CAST(CAST(requested_at AS datetime2(3)) AS varchar(30))              AS bo_do_lech
FROM toinoi.trips
WHERE code = 101503;

DECLARE @v nvarchar(400);
SET @v = (SELECT CONCAT(CAST(SWITCHOFFSET(requested_at, '+00:00') AS varchar(34)), N' | ',
                        CAST(requested_at AT TIME ZONE 'SE Asia Standard Time' AS varchar(34)), N' | ',
                        DATEPART(TZOFFSET, requested_at))
          FROM toinoi.trips WHERE code = 101503);
EXEC lab.expect_value N'1a 101503 UTC | VN | độ lệch', @v,
     N'2026-10-15 00:00:00.000 +00:00 | 2026-10-15 07:00:00.000 +07:00 | 420';

-- Tên múi giờ là tên Windows (cả trên Linux); có 141 tên trên bản CU28.
SET @v = CONCAT((SELECT COUNT(*) FROM sys.time_zone_info), N' tên; VN ',
                (SELECT current_utc_offset FROM sys.time_zone_info WHERE name = 'SE Asia Standard Time'));
EXEC lab.expect_value N'1b sys.time_zone_info', @v, N'141 tên; VN +07:00';
GO

-- ===========================================================================
-- 2. Đếm chuyến "ngày 15/10" bằng 7 cách
-- ===========================================================================
SELECT
  SUM(CASE WHEN requested_at >= '2026-10-15 00:00:00 +07:00'
            AND requested_at <  '2026-10-16 00:00:00 +07:00' THEN 1 ELSE 0 END)                    AS a_nua_mo,
  SUM(CASE WHEN CAST(requested_at AT TIME ZONE 'SE Asia Standard Time' AS date) = '2026-10-15'
           THEN 1 ELSE 0 END)                                                                       AS b_doi_gio_vn_roi_cat,
  SUM(CASE WHEN CAST(requested_at AS date) = '2026-10-15' THEN 1 ELSE 0 END)                        AS c_cat_theo_do_lech_luu,
  SUM(CASE WHEN CAST(SWITCHOFFSET(requested_at, '+00:00') AS date) = '2026-10-15' THEN 1 ELSE 0 END) AS d_cat_theo_utc_SAI,
  SUM(CASE WHEN requested_at BETWEEN '2026-10-15 00:00:00 +07:00'
                                 AND '2026-10-15 23:59:59 +07:00' THEN 1 ELSE 0 END)               AS e_between_235959_SAI,
  SUM(CASE WHEN requested_at BETWEEN '2026-10-15 00:00:00 +07:00'
                                 AND '2026-10-16 00:00:00 +07:00' THEN 1 ELSE 0 END)               AS f_between_ngay_sau_SAI,
  SUM(CASE WHEN requested_at >= '2026-10-15' AND requested_at < '2026-10-16' THEN 1 ELSE 0 END)     AS g_moc_khong_do_lech_SAI
INTO #dem_15
FROM toinoi.trips;

SELECT * FROM #dem_15;

DECLARE @v nvarchar(400) = (SELECT CONCAT_WS(N'|', a_nua_mo, b_doi_gio_vn_roi_cat, c_cat_theo_do_lech_luu, d_cat_theo_utc_SAI,
                                             e_between_235959_SAI, f_between_ngay_sau_SAI, g_moc_khong_do_lech_SAI) FROM #dem_15);
EXEC lab.expect_value N'2a bảy cách đếm 15/10', @v, N'106|106|106|105|105|107|105';
DROP TABLE #dem_15;

-- Cách c chỉ đúng NHỜ dữ liệu ghi bằng +07:00. Cùng các khoảnh khắc ấy ghi
-- bằng +00:00 (như một job nhập từ hệ thống ghi UTC): khoảnh khắc không đổi,
-- cách a vẫn 106, cách c còn 105.
SELECT SWITCHOFFSET(requested_at, '+00:00') AS requested_at
INTO #ghi_utc
FROM toinoi.trips;

SET @v = CONCAT(
    (SELECT COUNT(*) FROM #ghi_utc WHERE requested_at >= '2026-10-15 00:00:00 +07:00' AND requested_at < '2026-10-16 00:00:00 +07:00'),
    N'|',
    (SELECT COUNT(*) FROM #ghi_utc WHERE CAST(requested_at AS date) = '2026-10-15'),
    N'|',
    (SELECT COUNT(*) FROM #ghi_utc WHERE CAST(requested_at AT TIME ZONE 'SE Asia Standard Time' AS date) = '2026-10-15'));
EXEC lab.expect_value N'2b dữ liệu ghi +00:00: nửa mở | CAST date | AT TIME ZONE rồi cắt', @v, N'106|105|106';
DROP TABLE #ghi_utc;
GO

-- ===========================================================================
-- 3. Kiểm tra HAI BIÊN trên 5 chuyến cài sẵn
-- ===========================================================================
SELECT code, CAST(requested_at AS varchar(34)) AS requested_at,
       CASE WHEN requested_at >= '2026-10-15 00:00:00 +07:00'
             AND requested_at <  '2026-10-16 00:00:00 +07:00' THEN 1 ELSE 0 END  AS a_nua_mo,
       CASE WHEN requested_at BETWEEN '2026-10-15 00:00:00 +07:00'
                              AND '2026-10-15 23:59:59 +07:00' THEN 1 ELSE 0 END AS e_between_235959,
       CASE WHEN requested_at BETWEEN '2026-10-15 00:00:00 +07:00'
                              AND '2026-10-16 00:00:00 +07:00' THEN 1 ELSE 0 END AS f_between_ngay_sau,
       CASE WHEN CAST(SWITCHOFFSET(requested_at, '+00:00') AS date) = '2026-10-15' THEN 1 ELSE 0 END AS d_ngay_utc
INTO #bien
FROM toinoi.trips
WHERE code IN (101489, 101502, 101503, 101594, 101595);

SELECT * FROM #bien ORDER BY code;

DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CONCAT(code, N':', a_nua_mo, e_between_235959, f_between_ngay_sau, d_ngay_utc), N' ')
                                   WITHIN GROUP (ORDER BY code) FROM #bien);
EXEC lab.expect_value N'3a năm chuyến biên (a e f d)', @v, N'101489:1110 101502:1110 101503:1111 101594:1011 101595:0011';
DROP TABLE #bien;
GO

-- Dựng hai biên từ một tham số NGÀY (thứ ứng dụng truyền vào).
DECLARE @ngay date = '2026-10-15';
DECLARE @tu  datetimeoffset(3) = CAST(@ngay AS datetime2(3)) AT TIME ZONE 'SE Asia Standard Time';
DECLARE @den datetimeoffset(3) = DATEADD(day, 1, @tu);
DECLARE @tu2 datetimeoffset(3) = TODATETIMEOFFSET(CAST(@ngay AS datetime2(3)), '+07:00');
SELECT CAST(@tu AS varchar(34)) AS tu, CAST(@tu2 AS varchar(34)) AS tu_cach_2, CAST(@den AS varchar(34)) AS den_truoc,
       (SELECT COUNT(*) FROM toinoi.trips WHERE requested_at >= @tu AND requested_at < @den) AS so_chuyen;

DECLARE @v nvarchar(400) = CONCAT(CAST(@tu AS varchar(34)), N' | ', CAST(@den AS varchar(34)), N' | ',
                                  (SELECT COUNT(*) FROM toinoi.trips WHERE requested_at >= @tu AND requested_at < @den),
                                  N' | ', CASE WHEN @tu = @tu2 THEN N'cùng khoảnh khắc' ELSE N'khác' END);
EXEC lab.expect_value N'3b biên từ tham số ngày', @v,
     N'2026-10-15 00:00:00.000 +07:00 | 2026-10-16 00:00:00.000 +07:00 | 106 | cùng khoảnh khắc';
GO

-- ===========================================================================
-- 4. Bẫy kiểu datetime cũ: 23:59:59.999 làm tròn sang ngày hôm sau
-- ===========================================================================
-- datetime chính xác tới 1/300 giây (đuôi .000/.003/.007): '…23:59:59.999'
-- được làm tròn thành 00:00:00.000 ngày 16. Thử trên bản sao giờ VN kiểu datetime.
SELECT CAST(SWITCHOFFSET(requested_at, '+07:00') AS datetime) AS requested_at_vn
INTO #kieu_cu
FROM toinoi.trips;

SELECT CONVERT(varchar(23), CAST('2026-10-15 23:59:59.999' AS datetime), 121) AS moc_999,
       CONVERT(varchar(23), CAST('2026-10-15 23:59:59.997' AS datetime), 121) AS moc_997,
       (SELECT COUNT(*) FROM #kieu_cu WHERE requested_at_vn BETWEEN '2026-10-15' AND '2026-10-15 23:59:59.999') AS between_999,
       (SELECT COUNT(*) FROM #kieu_cu WHERE requested_at_vn BETWEEN '2026-10-15' AND '2026-10-15 23:59:59.997') AS between_997,
       (SELECT COUNT(*) FROM #kieu_cu WHERE requested_at_vn >= '2026-10-15' AND requested_at_vn < '2026-10-16') AS nua_mo;

DECLARE @v nvarchar(400) = CONCAT(
    CONVERT(varchar(23), CAST('2026-10-15 23:59:59.999' AS datetime), 121), N' | ',
    (SELECT COUNT(*) FROM #kieu_cu WHERE requested_at_vn BETWEEN '2026-10-15' AND '2026-10-15 23:59:59.999'), N'/',
    (SELECT COUNT(*) FROM #kieu_cu WHERE requested_at_vn BETWEEN '2026-10-15' AND '2026-10-15 23:59:59.997'), N'/',
    (SELECT COUNT(*) FROM #kieu_cu WHERE requested_at_vn >= '2026-10-15' AND requested_at_vn < '2026-10-16'));
EXEC lab.expect_value N'4a datetime .999 / .997 / nửa mở', @v, N'2026-10-16 00:00:00.000 | 107/106/106';
DROP TABLE #kieu_cu;
GO

-- ===========================================================================
-- 5. DATEDIFF và DATEPART trên datetimeoffset
-- ===========================================================================
-- DATEDIFF đếm số RANH GIỚI vượt qua, tính trên giá trị UTC. DATEPART lấy phần
-- của giờ trên đồng hồ như đã lưu.
DECLARE @sang   datetimeoffset(3) = '2026-10-15 06:00:00 +07:00',
        @sang2  datetimeoffset(3) = '2026-10-15 08:00:00 +07:00',
        @khuya  datetimeoffset(3) = '2026-10-15 23:59:59.500 +07:00',
        @nuadem datetimeoffset(3) = '2026-10-16 00:00:00 +07:00';
SELECT DATEDIFF(day, @sang, @sang2)                         AS cung_ngay_vn_cach_2_gio,
       DATEDIFF(day, @khuya, @nuadem)                       AS khac_ngay_vn_cach_nua_giay,
       DATEDIFF(day, CAST(@khuya AS datetime2(3)), CAST(@nuadem AS datetime2(3))) AS bo_do_lech_roi_tinh,
       DATEPART(hour, @sang)                                AS gio_theo_gia_tri_luu;

DECLARE @v nvarchar(100) = CONCAT_WS(N'|', DATEDIFF(day, @sang, @sang2), DATEDIFF(day, @khuya, @nuadem),
                                     DATEDIFF(day, CAST(@khuya AS datetime2(3)), CAST(@nuadem AS datetime2(3))),
                                     DATEPART(hour, @sang));
EXEC lab.expect_value N'5a DATEDIFF theo UTC / DATEPART theo giá trị lưu', @v, N'1|0|1|6';
GO

-- ===========================================================================
-- 6. Gom theo ngày VN và ngày UTC; biên tháng
-- ===========================================================================
SELECT ngay,
       SUM(CASE WHEN theo = 'vn'  THEN 1 ELSE 0 END) AS theo_ngay_vn,
       SUM(CASE WHEN theo = 'utc' THEN 1 ELSE 0 END) AS theo_ngay_utc
FROM toinoi.trips AS t
CROSS APPLY (VALUES ('vn',  CAST(SWITCHOFFSET(t.requested_at, '+07:00') AS date)),
                    ('utc', CAST(SWITCHOFFSET(t.requested_at, '+00:00') AS date))) AS v (theo, ngay)
WHERE ngay BETWEEN '2026-10-14' AND '2026-10-16'
GROUP BY ngay
ORDER BY ngay;

DECLARE @v nvarchar(400) = (
    SELECT STRING_AGG(CONCAT(FORMAT(ngay, 'dd/MM'), N':', vn, N'/', utc), N' ') WITHIN GROUP (ORDER BY ngay)
    FROM (SELECT ngay, SUM(CASE WHEN theo = 'vn' THEN 1 ELSE 0 END) AS vn, SUM(CASE WHEN theo = 'utc' THEN 1 ELSE 0 END) AS utc
          FROM toinoi.trips AS t
          CROSS APPLY (VALUES ('vn',  CAST(SWITCHOFFSET(t.requested_at, '+07:00') AS date)),
                              ('utc', CAST(SWITCHOFFSET(t.requested_at, '+00:00') AS date))) AS v (theo, ngay)
          WHERE ngay BETWEEN '2026-10-14' AND '2026-10-16'
          GROUP BY ngay) AS x);
EXEC lab.expect_value N'6a ngày VN / ngày UTC', @v, N'14/10:100/102 15/10:106/105 16/10:103/104';

-- Biên tháng: 6 chuyến đặt 00:30–00:58 ngày 31/10 giờ VN là 30/10 theo UTC.
SET @v = CONCAT(
    (SELECT COUNT(*) FROM toinoi.trips
     WHERE requested_at >= '2026-10-01 00:00:00 +07:00' AND requested_at < '2026-10-31 00:00:00 +07:00'), N' / ',
    (SELECT COUNT(*) FROM toinoi.trips
     WHERE CAST(SWITCHOFFSET(requested_at, '+00:00') AS date) BETWEEN '2026-09-30' AND '2026-10-30'), N' / ',
    (SELECT COUNT(*) FROM toinoi.trips
     WHERE CAST(SWITCHOFFSET(requested_at, '+07:00') AS date) <> CAST(SWITCHOFFSET(requested_at, '+00:00') AS date)));
EXEC lab.expect_value N'6b kỳ 01–30/10 VN / ngày UTC tương ứng / chuyến lệch ngày', @v, N'3168 / 3174 / 383';
GO

-- ===========================================================================
-- 7. Giờ trong ngày theo giờ VN
-- ===========================================================================
SELECT DATEPART(hour, SWITCHOFFSET(requested_at, '+07:00')) AS gio_vn, COUNT(*) AS so_chuyen
FROM toinoi.trips
WHERE DATEPART(hour, SWITCHOFFSET(requested_at, '+07:00')) < 8
GROUP BY DATEPART(hour, SWITCHOFFSET(requested_at, '+07:00'))
ORDER BY gio_vn;

DECLARE @v nvarchar(400) = (
    SELECT STRING_AGG(CONCAT(gio, N':', n), N' ') WITHIN GROUP (ORDER BY gio)
    FROM (SELECT DATEPART(hour, SWITCHOFFSET(requested_at, '+07:00')) AS gio, COUNT(*) AS n
          FROM toinoi.trips
          WHERE DATEPART(hour, SWITCHOFFSET(requested_at, '+07:00')) < 8
          GROUP BY DATEPART(hour, SWITCHOFFSET(requested_at, '+07:00'))) AS x);
EXEC lab.expect_value N'7a chuyến theo giờ VN 0–7', @v, N'0:68 1:28 2:39 3:39 4:25 5:61 6:123 7:193';
GO

-- ===========================================================================
-- 8. DATETRUNC và DATE_BUCKET (SQL Server 2022)
-- ===========================================================================
-- Trên datetimeoffset, DATETRUNC cắt theo giờ trên đồng hồ như đã lưu (+07:00);
-- DATE_BUCKET không ghi origin thì chia khung tính từ 1900-01-01 00:00 +00:00,
-- tức theo UTC: "ngày" của nó bắt đầu lúc 07:00 giờ VN.
DECLARE @x datetimeoffset(3) = '2026-10-15 06:59:59.000 +07:00';
DECLARE @goc datetimeoffset(3) = '2026-01-01 00:00:00 +07:00';
SELECT CAST(DATETRUNC(day, @x) AS varchar(34))            AS datetrunc_day,
       CAST(DATE_BUCKET(day, 1, @x) AS varchar(34))       AS date_bucket_day,
       CAST(DATE_BUCKET(day, 1, @x, @goc) AS varchar(34)) AS date_bucket_day_goc_vn,
       CAST(DATE_BUCKET(minute, 15, @x) AS varchar(34))   AS date_bucket_15_phut,
       CAST(DATETRUNC(month, @x) AS varchar(34))          AS datetrunc_month,
       CAST(DATE_BUCKET(month, 1, @x) AS varchar(34))     AS date_bucket_month;

DECLARE @v nvarchar(400) = CONCAT_WS(N' | ', CAST(DATETRUNC(day, @x) AS varchar(34)), CAST(DATE_BUCKET(day, 1, @x) AS varchar(34)),
                                     CAST(DATE_BUCKET(day, 1, @x, @goc) AS varchar(34)), CAST(DATE_BUCKET(minute, 15, @x) AS varchar(34)),
                                     CAST(DATE_BUCKET(month, 1, @x) AS varchar(34)));
EXEC lab.expect_value N'8a DATETRUNC / DATE_BUCKET trên 06:59:59 +07:00', @v,
     N'2026-10-15 00:00:00.000 +07:00 | 2026-10-14 07:00:00.000 +07:00 | 2026-10-15 00:00:00.000 +07:00 | 2026-10-15 06:45:00.000 +07:00 | 2026-10-01 07:00:00.000 +07:00';

-- Đếm "ngày 15/10" bằng DATE_BUCKET(day) không origin ra số của khung
-- 07:00 15/10 → 07:00 16/10, không phải ngày VN.
SET @v = CONCAT(
    (SELECT COUNT(*) FROM toinoi.trips WHERE DATE_BUCKET(day, 1, requested_at) = '2026-10-15 07:00:00 +07:00'), N'/',
    (SELECT COUNT(*) FROM toinoi.trips WHERE DATE_BUCKET(day, 1, requested_at, @goc) = '2026-10-15 00:00:00 +07:00'), N'/',
    (SELECT COUNT(*) FROM toinoi.trips WHERE DATETRUNC(day, requested_at) = '2026-10-15 00:00:00 +07:00'));
EXEC lab.expect_value N'8b đếm 15/10: DATE_BUCKET không origin / có origin VN / DATETRUNC', @v, N'105/106/106';
GO

-- Khung 15 phút: giờ cao điểm sáng 15/10 (07:00–09:00 giờ VN). Lịch khung
-- bằng GENERATE_SERIES + LEFT JOIN để khung không có chuyến vẫn hiện.
DECLARE @tu datetimeoffset(3) = '2026-10-15 07:00:00 +07:00';
WITH khung AS (
    SELECT DATEADD(minute, 15 * value, @tu) AS bat_dau FROM GENERATE_SERIES(0, 7)
),
chuyen AS (
    SELECT DATE_BUCKET(minute, 15, requested_at) AS bat_dau, COUNT(*) AS so_chuyen
    FROM toinoi.trips
    WHERE requested_at >= @tu AND requested_at < DATEADD(hour, 2, @tu)
    GROUP BY DATE_BUCKET(minute, 15, requested_at)
)
SELECT FORMAT(k.bat_dau, 'HH:mm') AS khung_15_phut, COALESCE(c.so_chuyen, 0) AS so_chuyen
INTO #khung
FROM khung AS k
LEFT JOIN chuyen AS c ON c.bat_dau = k.bat_dau;

SELECT khung_15_phut, so_chuyen FROM #khung ORDER BY khung_15_phut;

DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CONCAT(khung_15_phut, N':', so_chuyen), N' ') WITHIN GROUP (ORDER BY khung_15_phut) FROM #khung);
EXEC lab.expect_value N'8c khung 15 phút 07:00–09:00 ngày 15/10', @v,
     N'07:00:3 07:15:1 07:30:1 07:45:3 08:00:1 08:15:1 08:30:1 08:45:3';
DROP TABLE #khung;
GO

-- Khung 15 phút đông nhất cả tháng (giờ VN), hoà thì khung sớm hơn.
DECLARE @v nvarchar(400) = (
    SELECT TOP (1) CONCAT(CAST(DATE_BUCKET(minute, 15, requested_at) AS varchar(34)), N' : ', COUNT(*))
    FROM toinoi.trips
    GROUP BY DATE_BUCKET(minute, 15, requested_at)
    ORDER BY COUNT(*) DESC, DATE_BUCKET(minute, 15, requested_at));
EXEC lab.expect_value N'8d khung 15 phút đông nhất', @v, N'2026-10-10 09:00:00.000 +07:00 : 7';
GO

-- ===========================================================================
-- 9. Thứ trong tuần phụ thuộc DATEFIRST
-- ===========================================================================
-- 15/10/2026 là thứ Năm. DATEPART(weekday) đánh số theo @@DATEFIRST của PHIÊN
-- (us_english: 7 = tuần bắt đầu Chủ nhật). DATETRUNC(week) cũng theo DATEFIRST;
-- iso_week và DATE_BUCKET(week) luôn bắt đầu thứ Hai.
DECLARE @d date = '2026-10-15';
DECLARE @v nvarchar(400);
SELECT @@DATEFIRST AS datefirst, DATEPART(weekday, @d) AS weekday, DATENAME(weekday, @d) AS ten_thu,
       DATETRUNC(week, @d) AS datetrunc_week, DATETRUNC(iso_week, @d) AS datetrunc_iso_week,
       DATE_BUCKET(week, 1, @d) AS date_bucket_week;
SET @v = CONCAT_WS(N'|', @@DATEFIRST, DATEPART(weekday, @d), DATETRUNC(week, @d), DATETRUNC(iso_week, @d), DATE_BUCKET(week, 1, @d));
EXEC lab.expect_value N'9a DATEFIRST 7', @v, N'7|5|2026-10-11|2026-10-12|2026-10-12';

SET DATEFIRST 1;
SET @v = CONCAT_WS(N'|', @@DATEFIRST, DATEPART(weekday, @d), DATETRUNC(week, @d), DATETRUNC(iso_week, @d), DATE_BUCKET(week, 1, @d));
EXEC lab.expect_value N'9b DATEFIRST 1', @v, N'1|4|2026-10-12|2026-10-12|2026-10-12';
SET DATEFIRST 7;

-- Đếm chuyến theo thứ, KHÔNG phụ thuộc DATEFIRST: số ngày kể từ 01/01/1900
-- (một ngày thứ Hai) chia lấy dư cho 7 ⇒ 0 = thứ Hai … 6 = Chủ nhật.
SELECT DATEDIFF(day, '1900-01-01', CAST(SWITCHOFFSET(requested_at, '+07:00') AS date)) % 7 AS thu_0_la_thu_hai,
       COUNT(*) AS so_chuyen
FROM toinoi.trips
GROUP BY DATEDIFF(day, '1900-01-01', CAST(SWITCHOFFSET(requested_at, '+07:00') AS date)) % 7
ORDER BY thu_0_la_thu_hai;
SET @v = (SELECT STRING_AGG(CONCAT(thu, N':', n), N' ') WITHIN GROUP (ORDER BY thu)
          FROM (SELECT DATEDIFF(day, '1900-01-01', CAST(SWITCHOFFSET(requested_at, '+07:00') AS date)) % 7 AS thu, COUNT(*) AS n
                FROM toinoi.trips
                GROUP BY DATEDIFF(day, '1900-01-01', CAST(SWITCHOFFSET(requested_at, '+07:00') AS date)) % 7) AS x);
EXEC lab.expect_value N'9c chuyến theo thứ (0 = thứ Hai)', @v, N'0:403 1:385 2:402 3:505 4:502 5:490 6:487';
GO

-- ===========================================================================
-- 10. Giờ mùa hè: vì sao múi giờ ghi bằng TÊN, không bằng độ lệch
-- ===========================================================================
-- New York lùi đồng hồ ngày 01/11/2026 (ngày đó dài 25 giờ). DATEADD trên
-- datetimeoffset giữ nguyên độ lệch -04:00 nên "cộng 1 ngày" ra 23:00 giờ địa
-- phương. Cộng trên giờ đồng hồ (datetime2) rồi AT TIME ZONE mới ra nửa đêm.
DECLARE @tz sysname = N'Eastern Standard Time';
DECLARE @bat_dau datetimeoffset(0) = CAST('2026-11-01 00:00:00' AS datetime2(0)) AT TIME ZONE @tz;
DECLARE @nua_dem_sau datetimeoffset(0) = DATEADD(day, 1, CAST('2026-11-01 00:00:00' AS datetime2(0))) AT TIME ZONE @tz;
SELECT CAST(@bat_dau AS varchar(34))                                   AS bat_dau,
       CAST(DATEADD(day, 1, @bat_dau) AS varchar(34))                  AS dateadd_giu_do_lech,
       CAST(DATEADD(day, 1, @bat_dau) AT TIME ZONE @tz AS varchar(34)) AS doi_ve_gio_dia_phuong,
       CAST(@nua_dem_sau AS varchar(34))                               AS cong_tren_dong_ho,
       DATEDIFF(hour, @bat_dau, @nua_dem_sau)                          AS so_gio_cua_ngay;

DECLARE @v nvarchar(400) = CONCAT_WS(N' | ', CAST(@bat_dau AS varchar(34)), CAST(DATEADD(day, 1, @bat_dau) AT TIME ZONE @tz AS varchar(34)),
                                     CAST(@nua_dem_sau AS varchar(34)), DATEDIFF(hour, @bat_dau, @nua_dem_sau));
EXEC lab.expect_value N'10a New York 01/11/2026', @v,
     N'2026-11-01 00:00:00 -04:00 | 2026-11-01 23:00:00 -05:00 | 2026-11-02 00:00:00 -05:00 | 25';

-- Giờ đồng hồ 01:30 sáng 01/11 xảy ra HAI lần ở New York; AT TIME ZONE với
-- datetime2 chọn độ lệch trước khi lùi (-04:00). Đi từ UTC thì phân biệt được.
SET @v = CONCAT_WS(N' | ',
    CAST(CAST('2026-11-01 01:30:00' AS datetime2(0)) AT TIME ZONE @tz AS varchar(34)),
    CAST(CAST('2026-11-01 05:30:00 +00:00' AS datetimeoffset(0)) AT TIME ZONE @tz AS varchar(34)),
    CAST(CAST('2026-11-01 06:30:00 +00:00' AS datetimeoffset(0)) AT TIME ZONE @tz AS varchar(34)));
EXEC lab.expect_value N'10b 01:30 xảy ra hai lần', @v,
     N'2026-11-01 01:30:00 -04:00 | 2026-11-01 01:30:00 -04:00 | 2026-11-01 01:30:00 -05:00';

-- Việt Nam không có giờ mùa hè: cộng 1 ngày và cộng 24 giờ trùng nhau.
DECLARE @vn datetimeoffset(0) = CAST('2026-10-15 00:00:00' AS datetime2(0)) AT TIME ZONE 'SE Asia Standard Time';
SET @v = CONCAT_WS(N' | ', CAST(DATEADD(day, 1, @vn) AS varchar(34)), CAST(DATEADD(hour, 24, @vn) AS varchar(34)));
EXEC lab.expect_value N'10c Việt Nam +1 ngày | +24 giờ', @v, N'2026-10-16 00:00:00 +07:00 | 2026-10-16 00:00:00 +07:00';
GO

-- ===========================================================================
-- 11. Lời giải bài tập
-- ===========================================================================
-- Bài 1. Doanh thu (cước chuyến hoàn thành) ngày 15/10 giờ VN, dựng biên từ
--        tham số ngày.
DECLARE @ngay date = '2026-10-15';
DECLARE @tu datetimeoffset(3) = CAST(@ngay AS datetime2(3)) AT TIME ZONE 'SE Asia Standard Time';
DECLARE @v nvarchar(400) = (
    SELECT CONCAT(COUNT(*), N' chuyến, ', SUM(fare), N' đ')
    FROM toinoi.trips
    WHERE status = 'completed' AND requested_at >= @tu AND requested_at < DATEADD(day, 1, @tu));
EXEC lab.expect_value N'BT1 doanh thu 15/10', @v, N'89 chuyến, 5862000 đ';
GO

-- Bài 2. Số chuyến Đà Nẵng 19–21/10 theo ngày VN, ngày bão vẫn có dòng 0.
DECLARE @v nvarchar(400) = (
    SELECT STRING_AGG(CONCAT(FORMAT(l.ngay, 'dd/MM'), N':', x.n), N' ') WITHIN GROUP (ORDER BY l.ngay)
    FROM (SELECT DATEADD(day, value, CAST('2026-10-19' AS date)) AS ngay FROM GENERATE_SERIES(0, 2)) AS l
    CROSS APPLY (SELECT COUNT(*) AS n FROM toinoi.trips AS t
                 WHERE t.city_id = 3
                   AND t.requested_at >= CAST(l.ngay AS datetime2(3)) AT TIME ZONE 'SE Asia Standard Time'
                   AND t.requested_at <  CAST(DATEADD(day, 1, l.ngay) AS datetime2(3)) AT TIME ZONE 'SE Asia Standard Time') AS x);
EXEC lab.expect_value N'BT2 DN 19–21/10', @v, N'19/10:13 20/10:0 21/10:17';
GO

-- Bài 3. Thời gian trung bình từ lúc đặt tới lúc hoàn thành theo dịch vụ (giây
--        → phút, 1 chữ số lẻ). DATEDIFF trên datetimeoffset tính theo UTC nên
--        không bị độ lệch ảnh hưởng.
DECLARE @v nvarchar(400) = (
    SELECT STRING_AGG(CONCAT(kind, N':', phut), N' ') WITHIN GROUP (ORDER BY kind)
    FROM (SELECT kind, CAST(ROUND(AVG(CAST(DATEDIFF(second, requested_at, completed_at) AS decimal(12, 2))) / 60, 1) AS decimal(6, 1)) AS phut
          FROM toinoi.trips WHERE status = 'completed' GROUP BY kind) AS x);
EXEC lab.expect_value N'BT3 phút đặt → hoàn thành', @v, N'food:29.0 parcel:28.0 ride:28.4';
GO
