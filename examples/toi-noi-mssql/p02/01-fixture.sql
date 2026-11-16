-- P02 · 01-fixture.sql — bộ dữ liệu 30 ngày, sinh TẤT ĐỊNH bằng T-SQL.
--
-- Ảnh chụp lúc 2026-10-31 01:00 giờ VN:
--   * chuyến đặt từ 2026-10-01 00:00 tới 2026-10-30 23:59:59 (+07:00) đều đã
--     kết thúc (completed / cancelled);
--   * 6 chuyến đặt sau nửa đêm 31/10 đang chạy, mỗi chuyến một trạng thái.
-- Mọi tên, số điện thoại, quán, khu vực đều HƯ CẤU.
--
-- CÙNG dữ liệu với lab PostgreSQL (examples/toi-noi-postgresql/p02/01-fixture.sql):
-- cùng hàm băm, cùng công thức, cùng cách làm tròn (lab.p02_phut). Đã đối chiếu
-- từng chuyến với lab PostgreSQL; 02-kiem-tra-fixture.sql giữ các số tổng.
--
-- Tất định: KHÔNG dùng NEWID()/RAND(). Mọi lựa chọn "ngẫu nhiên" lấy từ hàm băm
-- số học lab.p02_hash(n, salt) — chỉ để sinh dữ liệu lab; cuối file bị xoá.
--
-- Mã chuyến ghi thẳng (SET IDENTITY_INSERT ON), tăng theo giờ đặt, như nạp lịch
-- sử từ hệ cũ giữ nguyên mã. id = 70000000-…-<mã> chỉ để dễ đối chiếu.

SET NOCOUNT ON;
SET XACT_ABORT ON;
USE [$(LAB_DB)];
GO

-- ---------------------------------------------------------------------------
-- Hàm băm sinh dữ liệu: nhân Knuth (2654435761) trên 31 bit thấp, lấy 16 bit
-- cao của tích 32 bit ⇒ 0..65535. Mọi phép tính trong bigint (tích lớn nhất
-- ≈ 5,7·10^18 < 9,2·10^18), không tràn. Scalar UDF của SQL Server 2019+ được
-- "inline" vào truy vấn nên gọi hàng chục nghìn lần vẫn nhanh.
CREATE FUNCTION lab.p02_hash (@n bigint, @salt int)
RETURNS int
WITH SCHEMABINDING
AS
BEGIN
    RETURN CAST(((((@n * 1000003 + @salt * 7919) % 2147483648) * 2654435761) % 4294967296) / 65536 AS int);
END;
GO

-- Phút đặt trong giờ: floor((p − truoc) / w · 60) với p = p_tu / p_mau.
-- Bản PostgreSQL tính bằng numeric: mỗi phép chia LÀM TRÒN tới 16 hoặc 20 chữ
-- số thập phân (quy tắc select_div_scale), nên khi kết quả đúng là số nguyên
-- (vd. 50) nó có thể ra 49,999… và floor còn 49. Hàm này tái hiện đúng các bước
-- làm tròn đó bằng decimal(38, 0) để mọi chuyến có cùng phút đặt với lab
-- PostgreSQL (khác biệt thật: 1 chuyến, mã 101126). Ngoài mục đích đối chiếu,
-- đây KHÔNG phải cách nên tính thời gian.
CREATE FUNCTION lab.p02_phut (@p_tu int, @p_mau int, @truoc int, @w int)
RETURNS int
WITH SCHEMABINDING
AS
BEGIN
    DECLARE @rs int = CASE WHEN @p_tu <= @p_mau THEN 20 ELSE 16 END;
    DECLARE @n decimal(38, 0) = CAST(@p_tu AS decimal(38, 0)) * POWER(CAST(10 AS decimal(38, 0)), @rs);
    DECLARE @r decimal(38, 0) = @n % @p_mau;
    DECLARE @p decimal(38, 0) = (@n - @r) / @p_mau + CASE WHEN 2 * @r >= @p_mau THEN 1 ELSE 0 END;   -- p, 10^rs
    DECLARE @v decimal(38, 0) = (@p - CAST(@truoc AS decimal(38, 0)) * POWER(CAST(10 AS decimal(38, 0)), @rs))
                                * POWER(CAST(10 AS decimal(38, 0)), 20 - @rs);                       -- p − truoc, 10^20
    SET @r = @v % @w;
    DECLARE @q decimal(38, 0) = (@v - @r) / @w + CASE WHEN 2 * @r >= @w THEN 1 ELSE 0 END;           -- ÷ w, 10^20
    DECLARE @m decimal(38, 0) = @q * 60;
    RETURN CAST((@m - @m % POWER(CAST(10 AS decimal(38, 0)), 20)) / POWER(CAST(10 AS decimal(38, 0)), 20) AS int);
END;
GO

-- ---------------------------------------------------------------------------
-- Danh mục
INSERT INTO toinoi.cities (id, code, name) VALUES
    (1, 'SG', N'TP. Hồ Chí Minh'),
    (2, 'HN', N'Hà Nội'),
    (3, 'DN', N'Đà Nẵng');

-- Cây khu vực (hư cấu): 3 thành phố, 10 vùng, 21 khu. Vùng HN-NA mới mở, CHƯA
-- có khu nào (lá ở cấp vùng) — dùng ở MS-019.
INSERT INTO toinoi.areas (id, parent_id, code, name, level, city_id) VALUES
    (1,   NULL, 'SG',       N'TP. Hồ Chí Minh', 'city',     1),
    (2,   NULL, 'HN',       N'Hà Nội',          'city',     2),
    (3,   NULL, 'DN',       N'Đà Nẵng',         'city',     3),
    (11,  1,    'SG-TT',    N'Trung tâm',       'district', NULL),
    (12,  1,    'SG-DO',    N'Phía Đông',       'district', NULL),
    (13,  1,    'SG-TA',    N'Phía Tây',        'district', NULL),
    (14,  1,    'SG-NA',    N'Phía Nam',        'district', NULL),
    (21,  2,    'HN-TT',    N'Trung tâm',       'district', NULL),
    (22,  2,    'HN-TA',    N'Phía Tây',        'district', NULL),
    (23,  2,    'HN-BA',    N'Phía Bắc',        'district', NULL),
    (24,  2,    'HN-NA',    N'Phía Nam (mở 11/2026)', 'district', NULL),
    (31,  3,    'DN-TT',    N'Trung tâm',       'district', NULL),
    (32,  3,    'DN-BI',    N'Ven biển',        'district', NULL),
    (111, 11,   'SG-TT-01', N'Khu 01', 'zone', NULL),
    (112, 11,   'SG-TT-02', N'Khu 02', 'zone', NULL),
    (113, 11,   'SG-TT-03', N'Khu 03', 'zone', NULL),
    (121, 12,   'SG-DO-01', N'Khu 01', 'zone', NULL),
    (122, 12,   'SG-DO-02', N'Khu 02', 'zone', NULL),
    (123, 12,   'SG-DO-03', N'Khu 03', 'zone', NULL),
    (131, 13,   'SG-TA-01', N'Khu 01', 'zone', NULL),
    (132, 13,   'SG-TA-02', N'Khu 02', 'zone', NULL),
    (141, 14,   'SG-NA-01', N'Khu 01', 'zone', NULL),
    (142, 14,   'SG-NA-02', N'Khu 02', 'zone', NULL),
    (211, 21,   'HN-TT-01', N'Khu 01', 'zone', NULL),
    (212, 21,   'HN-TT-02', N'Khu 02', 'zone', NULL),
    (213, 21,   'HN-TT-03', N'Khu 03', 'zone', NULL),
    (221, 22,   'HN-TA-01', N'Khu 01', 'zone', NULL),
    (222, 22,   'HN-TA-02', N'Khu 02', 'zone', NULL),
    (231, 23,   'HN-BA-01', N'Khu 01', 'zone', NULL),
    (232, 23,   'HN-BA-02', N'Khu 02', 'zone', NULL),
    (311, 31,   'DN-TT-01', N'Khu 01', 'zone', NULL),
    (312, 31,   'DN-TT-02', N'Khu 02', 'zone', NULL),
    (321, 32,   'DN-BI-01', N'Khu 01', 'zone', NULL),
    (322, 32,   'DN-BI-02', N'Khu 02', 'zone', NULL);

-- 600 khách cá nhân: 1–300 SG, 301–510 HN, 511–600 DN (thành phố cư trú chỉ
-- dùng để sinh chuyến). Số 0910000001 … 0910000600. Thêm 1 khách doanh nghiệp
-- đặt bưu kiện qua API (MS-018). GENERATE_SERIES có từ SQL Server 2022.
INSERT INTO toinoi.customers (id, phone, full_name, created_at)
SELECT CAST('c1000000-0000-4000-8000-' + RIGHT('000000000000' + CAST(g.value AS varchar(12)), 12) AS uniqueidentifier),
       '091' + RIGHT('0000000' + CAST(g.value AS varchar(7)), 7),
       CHOOSE(1 + g.value % 12, N'Nguyễn', N'Trần', N'Lê', N'Phạm', N'Hoàng', N'Huỳnh', N'Phan', N'Vũ', N'Võ', N'Đặng', N'Bùi', N'Đỗ')
         + N' ' + CHOOSE(1 + (g.value / 12) % 8, N'Văn', N'Thị', N'Minh', N'Thu', N'Quốc', N'Ngọc', N'Hữu', N'Thanh')
         + N' ' + CHOOSE(1 + (g.value / 96) % 20, N'An', N'Bình', N'Châu', N'Dũng', N'Giang', N'Hải', N'Khánh', N'Linh', N'Mai', N'Nam',
                         N'Oanh', N'Phong', N'Quân', N'Sơn', N'Tâm', N'Uyên', N'Việt', N'Xuân', N'Yến', N'Hạnh'),
       DATEADD(hour, g.value * 7, CAST('2026-01-01 08:00:00 +07:00' AS datetimeoffset(3)))
FROM GENERATE_SERIES(1, 600) AS g
UNION ALL
SELECT 'cb000000-0000-4000-8000-000000000001', '0919990001', N'Cửa hàng Phụ Kiện Lab (KHDN)', '2026-03-15 09:00:00 +07:00';

-- Tài xế: SG 8101–8124, HN 8201–8220, DN 8301–8310. 8219, 8220 vừa đăng ký
-- 29/10, CHƯA chạy chuyến nào (MS-014: bẫy NOT IN). 8101, 8102, 8201, 8301 đang
-- có chuyến dở lúc chụp ⇒ busy.
INSERT INTO toinoi.drivers (id, code, full_name, city_id, vehicle_type, status, created_at)
SELECT CAST('d1000000-0000-4000-8000-' + RIGHT('000000000000' + CAST(d.code AS varchar(12)), 12) AS uniqueidentifier),
       d.code,
       CHOOSE(1 + d.code % 10, N'Bùi', N'Hồ', N'Ngô', N'Đỗ', N'Lý', N'Trịnh', N'Mai', N'Tạ', N'Lương', N'Đinh')
         + N' ' + CHOOSE(1 + (d.code / 10) % 6, N'Văn', N'Thị', N'Quang', N'Thanh', N'Đức', N'Kim')
         + N' ' + CHOOSE(1 + (d.code / 7) % 12, N'Khoa', N'Lan', N'Minh', N'Nam', N'Oanh', N'Phúc', N'Quý', N'Sang', N'Tú', N'Vinh', N'Hưng', N'Lộc'),
       d.city_id,
       CASE WHEN d.code % 4 = 0 THEN 'car' ELSE 'bike' END,
       CASE WHEN d.code IN (8101, 8102, 8201, 8301) THEN 'busy'
            WHEN d.code IN (8219, 8220) THEN 'offline'
            WHEN lab.p02_hash(d.code, 30) % 3 = 0 THEN 'online'
            ELSE 'offline' END,
       CASE WHEN d.code IN (8219, 8220) THEN CAST('2026-10-29 09:00:00 +07:00' AS datetimeoffset(3))
            ELSE DATEADD(day, d.code % 100, CAST('2026-06-01 09:00:00 +07:00' AS datetimeoffset(3))) END
FROM (SELECT 8100 + value AS code, CAST(1 AS smallint) AS city_id FROM GENERATE_SERIES(1, 24)
      UNION ALL SELECT 8200 + value, 2 FROM GENERATE_SERIES(1, 20)
      UNION ALL SELECT 8300 + value, 3 FROM GENERATE_SERIES(1, 10)) AS d;

-- Quán: SG 101–112, HN 201–208, DN 301–305.
INSERT INTO toinoi.merchants (id, city_id, name, created_at)
SELECT CAST('e1000000-0000-4000-8000-' + RIGHT('000000000000' + CAST(m.n AS varchar(12)), 12) AS uniqueidentifier),
       CAST(m.n / 100 AS smallint),
       CHOOSE(1 + m.n % 10, N'Cơm Tấm', N'Bún Bò', N'Phở', N'Bánh Mì', N'Trà Sữa', N'Gà Rán', N'Bún Chả', N'Mì Quảng', N'Bánh Xèo', N'Cháo')
         + N' Lab ' + CAST(m.n AS nvarchar(5)),
       '2026-05-01 08:00:00 +07:00'
FROM (SELECT 100 + value AS n FROM GENERATE_SERIES(1, 12)
      UNION ALL SELECT 200 + value FROM GENERATE_SERIES(1, 8)
      UNION ALL SELECT 300 + value FROM GENERATE_SERIES(1, 5)) AS m;
GO

-- ---------------------------------------------------------------------------
-- Kế hoạch chuyến (bảng tạm #p02_plan): mỗi dòng một chuyến với đủ thuộc tính
-- và các mốc thời gian, kể cả mốc không lưu trong trips (đón khách) — để sinh
-- trips, trip_events, trip_ratings, payments nhất quán.
--
-- Số chuyến mỗi (ngày, thành phố) = cơ sở (SG 50 · HN 35 · DN 15) × 1,2 nếu là
-- thứ Bảy/Chủ nhật, cộng dao động −2…+2. Ngày 20/10 Đà Nẵng có bão: 0 chuyến.
-- Giờ đặt rải theo hồ sơ giờ trong ngày (tổng trọng số 100), làm tròn tới PHÚT.
--
-- Vị trí i trong ngày có "phần trăm" p = (i + 0,5) · 100 / so_chuyen, viết
-- p = (2i + 1) · 50 / so_chuyen để so với ranh giới giờ bằng số nguyên chính
-- xác; phút trong giờ do lab.p02_phut tính (xem chú thích ở hàm).
WITH gio AS (
    SELECT h, w FROM (VALUES (0, 2), (1, 1), (2, 1), (3, 1), (4, 1), (5, 2), (6, 4), (7, 6), (8, 6), (9, 4),
                             (10, 4), (11, 8), (12, 9), (13, 5), (14, 3), (15, 3), (16, 4), (17, 7), (18, 9),
                             (19, 7), (20, 5), (21, 4), (22, 2), (23, 2)) AS v (h, w)
),
gio_luy_ke AS (
    SELECT h, w, SUM(w) OVER (ORDER BY h ROWS UNBOUNDED PRECEDING) - w AS truoc FROM gio
),
tp AS (
    SELECT city_id, co_so FROM (VALUES (1, 50), (2, 35), (3, 15)) AS v (city_id, co_so)
),
o AS (                                   -- số chuyến của từng (ngày, thành phố)
    SELECT g.value AS d, DATEADD(day, g.value, CAST('2026-10-01' AS date)) AS ngay, tp.city_id,
           CASE WHEN tp.city_id = 3 AND g.value = 19 THEN 0           -- 20/10, Đà Nẵng
                -- 1900-01-01 là thứ Hai ⇒ (số ngày kể từ đó) % 7: 5 = thứ Bảy, 6 = Chủ nhật
                ELSE tp.co_so * (CASE WHEN DATEDIFF(day, '19000101', DATEADD(day, g.value, CAST('2026-10-01' AS date))) % 7 >= 5
                                      THEN 12 ELSE 10 END) / 10
                     + (g.value * 7 + tp.city_id * 3) % 5 - 2
           END AS so_chuyen
    FROM GENERATE_SERIES(0, 29) AS g CROSS JOIN tp
),
sinh AS (                                -- chuyến thường: vị trí i trong ngày
    SELECT o.d * 1000 + o.city_id * 300 + i.value AS k, o.city_id, o.ngay,
           (2 * i.value + 1) * 50 AS p_tu, o.so_chuyen AS p_mau          -- p = p_tu / p_mau
    FROM o JOIN GENERATE_SERIES(0, 200) AS i ON i.value < o.so_chuyen
),
thuong AS (
    SELECT s.k, CAST(s.city_id AS smallint) AS city_id,
           TODATETIMEOFFSET(
               DATEADD(minute, lab.p02_phut(s.p_tu, s.p_mau, gl.truoc, gl.w),
                   DATEADD(hour, gl.h, CAST(s.ngay AS datetime2(3)))), '+07:00') AS requested_at,
           CAST(NULL AS varchar(10)) AS kind_dat, CAST(NULL AS varchar(20)) AS ket_cuc_dat,
           CAST(NULL AS uniqueidentifier) AS customer_dat, CAST(NULL AS int) AS driver_code_dat,
           CAST('thuong' AS varchar(10)) AS nguon
    FROM sinh AS s
    JOIN gio_luy_ke AS gl ON s.p_tu >= gl.truoc * s.p_mau AND s.p_tu < (gl.truoc + gl.w) * s.p_mau
),
dac_biet AS (
    SELECT k, CAST(city_id AS smallint) AS city_id, CAST(requested_at AS datetimeoffset(3)) AS requested_at,
           CAST(kind_dat AS varchar(10)) AS kind_dat, CAST(ket_cuc_dat AS varchar(20)) AS ket_cuc_dat,
           CAST(customer_dat AS uniqueidentifier) AS customer_dat, driver_code_dat, CAST(nguon AS varchar(10)) AS nguon
    FROM (VALUES
    -- MS-016: chuyến ở sát biên ngày 15/10 (giờ VN).
    (900001, 1, '2026-10-15 00:00:00 +07:00',     'ride', 'completed', NULL, NULL, 'bien'),
    (900002, 1, '2026-10-15 06:59:59 +07:00',     'ride', 'completed', NULL, NULL, 'bien'),
    (900003, 1, '2026-10-15 07:00:00 +07:00',     'ride', 'completed', NULL, NULL, 'bien'),
    (900004, 1, '2026-10-15 23:59:59.500 +07:00', 'ride', 'completed', NULL, NULL, 'bien'),
    (900005, 1, '2026-10-16 00:00:00 +07:00',     'ride', 'completed', NULL, NULL, 'bien'),
    -- MS-018: 18 chuyến bưu kiện của khách doanh nghiệp; hai lô đặt qua API cùng
    -- một thời điểm (6 chuyến lúc 10/10 09:00, 4 chuyến lúc 18/10 14:30).
    (900201, 1, '2026-10-02 10:15:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900202, 1, '2026-10-05 15:40:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900203, 1, '2026-10-07 09:20:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900204, 1, '2026-10-10 09:00:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900205, 1, '2026-10-10 09:00:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900206, 1, '2026-10-10 09:00:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900207, 1, '2026-10-10 09:00:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900208, 1, '2026-10-10 09:00:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900209, 1, '2026-10-10 09:00:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900210, 1, '2026-10-12 11:05:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900211, 1, '2026-10-15 16:45:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900212, 1, '2026-10-18 14:30:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900213, 1, '2026-10-18 14:30:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900214, 1, '2026-10-18 14:30:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900215, 1, '2026-10-18 14:30:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900216, 1, '2026-10-21 08:50:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900217, 1, '2026-10-25 14:10:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    (900218, 1, '2026-10-28 10:30:00 +07:00', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
    -- Sáu chuyến đang chạy lúc chụp 2026-10-31 01:00, mỗi chuyến một trạng thái.
    (900101, 1, '2026-10-31 00:30:00 +07:00', 'ride',   'in_transit', NULL, 8101, 'dang_chay'),
    (900102, 2, '2026-10-31 00:38:00 +07:00', 'food',   'picked_up',  NULL, 8201, 'dang_chay'),
    (900103, 1, '2026-10-31 00:46:00 +07:00', 'parcel', 'arriving',   NULL, 8102, 'dang_chay'),
    (900104, 3, '2026-10-31 00:50:00 +07:00', 'ride',   'accepted',   NULL, 8301, 'dang_chay'),
    (900105, 2, '2026-10-31 00:55:00 +07:00', 'food',   'searching',  NULL, NULL, 'dang_chay'),
    (900106, 1, '2026-10-31 00:58:00 +07:00', 'ride',   'requested',  NULL, NULL, 'dang_chay')
    ) AS v (k, city_id, requested_at, kind_dat, ket_cuc_dat, customer_dat, driver_code_dat, nguon)
),
tat_ca AS (
    SELECT * FROM thuong UNION ALL SELECT * FROM dac_biet
),
-- Thông số theo thành phố: số tài xế đang hoạt động, số quán, số khu, dải khách.
thong_so AS (
    SELECT * FROM (VALUES (CAST(1 AS smallint), 24, 12, 10, 1, 300),
                          (CAST(2 AS smallint), 18,  8,  7, 301, 210),
                          (CAST(3 AS smallint), 10,  5,  4, 511, 90)) AS v (city_id, so_tai_xe, so_quan, so_khu, kh_dau, kh_so)
),
b1 AS (                                  -- loại dịch vụ và kết cục
    SELECT t.*, ts.so_tai_xe, ts.so_quan, ts.so_khu, ts.kh_dau, ts.kh_so,
           COALESCE(t.kind_dat, CASE WHEN lab.p02_hash(t.k, 1) % 100 < 40 THEN 'ride'
                                     WHEN lab.p02_hash(t.k, 1) % 100 < 85 THEN 'food'
                                     ELSE 'parcel' END) AS kind,
           COALESCE(t.ket_cuc_dat, CASE WHEN lab.p02_hash(t.k, 2) % 1000 < 90  THEN 'huy_truoc_nhan'
                                        WHEN lab.p02_hash(t.k, 2) % 1000 < 150 THEN 'huy_sau_nhan'
                                        ELSE 'completed' END) AS ket_cuc
    FROM tat_ca AS t JOIN thong_so AS ts ON ts.city_id = t.city_id
),
b2 AS (                                  -- ai, ở đâu, bao nhiêu tiền
    SELECT b1.*,
           COALESCE(customer_dat,
               CAST('c1000000-0000-4000-8000-' + RIGHT('000000000000' + CAST(kh_dau +
                   CASE WHEN lab.p02_hash(k, 5) % 5 = 0 THEN lab.p02_hash(k, 17) % (kh_so / 10)   -- 20 %: khách đi nhiều
                        ELSE lab.p02_hash(k, 17) % kh_so END AS varchar(12)), 12) AS uniqueidentifier)) AS customer_id,
           CASE WHEN ket_cuc IN ('huy_truoc_nhan', 'requested', 'searching') THEN NULL
                ELSE CAST('d1000000-0000-4000-8000-' + RIGHT('000000000000' + CAST(COALESCE(driver_code_dat,
                        city_id * 100 + 8000 + 1 + lab.p02_hash(k, 6) % so_tai_xe) AS varchar(12)), 12) AS uniqueidentifier)
           END AS driver_id,
           CASE WHEN kind = 'food'
                THEN CAST('e1000000-0000-4000-8000-' + RIGHT('000000000000' + CAST(city_id * 100 + 1 + lab.p02_hash(k, 7) % so_quan AS varchar(12)), 12) AS uniqueidentifier)
           END AS merchant_id,
           CASE WHEN lab.p02_hash(k, 11) % 100 = 0 THEN NULL                       -- 1 %: mất GPS
                ELSE CASE city_id
                       WHEN 1 THEN CHOOSE(1 + lab.p02_hash(k, 10) % so_khu, 111, 112, 113, 121, 122, 123, 131, 132, 141, 142)
                       WHEN 2 THEN CHOOSE(1 + lab.p02_hash(k, 10) % so_khu, 211, 212, 213, 221, 222, 231, 232)
                       ELSE        CHOOSE(1 + lab.p02_hash(k, 10) % so_khu, 311, 312, 321, 322) END
           END AS pickup_area_id,
           CASE kind
             WHEN 'ride' THEN CASE WHEN lab.p02_hash(k, 4) % 100 < 2 THEN 0      -- khuyến mãi 0 đ
                                   ELSE 12000 + lab.p02_hash(k, 3) % 89 * 1000 END
             WHEN 'food' THEN 25000 + lab.p02_hash(k, 3) % 126 * 1000
             ELSE             18000 + lab.p02_hash(k, 3) % 63 * 1000
           END AS fare,
           20 + lab.p02_hash(k, 8) % 280  AS giay_cho_nhan,      -- đặt → tài xế nhận
           3 + lab.p02_hash(k, 9) % 8     AS phut_toi_don,       -- nhận → đón/lấy hàng
           5 + lab.p02_hash(k, 15) % 30   AS phut_di,            -- đón → hoàn thành
           lab.p02_hash(k, 16)            AS h_huy
    FROM b1
),
b3 AS (                                  -- các mốc thời gian
    SELECT b2.*,
           CASE WHEN driver_id IS NOT NULL THEN DATEADD(second, giay_cho_nhan, requested_at) END AS accepted_at,
           CASE WHEN ket_cuc IN ('completed', 'picked_up', 'in_transit')
                THEN DATEADD(minute, phut_toi_don, DATEADD(second, giay_cho_nhan, requested_at)) END AS picked_up_at
    FROM b2
)
SELECT ROW_NUMBER() OVER (ORDER BY requested_at, k) + 99999 AS code,
       k, nguon, kind, city_id, customer_id, driver_id, merchant_id, pickup_area_id, fare, ket_cuc,
       CASE WHEN ket_cuc IN ('huy_truoc_nhan', 'huy_sau_nhan') THEN 'cancelled' ELSE ket_cuc END AS status,
       requested_at, accepted_at, picked_up_at,
       CASE WHEN ket_cuc = 'completed' THEN DATEADD(minute, phut_di, picked_up_at) END AS completed_at,
       CASE ket_cuc
         WHEN 'huy_truoc_nhan' THEN DATEADD(second, 60 + h_huy % 300, requested_at)
         WHEN 'huy_sau_nhan'   THEN DATEADD(second, 30 + h_huy % 450, accepted_at)
       END AS cancelled_at,
       -- Bước cuối của vòng đời (1 requested … 7 completed); chuyến huỷ thêm bước 8.
       CASE ket_cuc WHEN 'requested' THEN 1 WHEN 'searching' THEN 2 WHEN 'accepted' THEN 3
                    WHEN 'arriving' THEN 4 WHEN 'picked_up' THEN 5 WHEN 'in_transit' THEN 6
                    WHEN 'completed' THEN 7 WHEN 'huy_truoc_nhan' THEN 2 WHEN 'huy_sau_nhan' THEN 4
       END AS buoc_cuoi
INTO #p02_plan
FROM b3;

DECLARE @n nvarchar(20) = CONVERT(nvarchar(20), (SELECT COUNT(*) FROM #p02_plan));
EXEC lab.expect_value N'kế hoạch: số chuyến', @n, N'3174';
SET @n = CONVERT(nvarchar(20), (SELECT COUNT(*) - COUNT(DISTINCT code) FROM #p02_plan));
EXEC lab.expect_value N'kế hoạch: mã trùng', @n, N'0';

-- ---------------------------------------------------------------------------
-- Chuyến. updated_at = mốc cuối cùng của vòng đời.
SET IDENTITY_INSERT toinoi.trips ON;
INSERT INTO toinoi.trips (id, code, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                          requested_at, accepted_at, completed_at, updated_at, pickup_area_id)
SELECT CAST('70000000-0000-4000-8000-' + RIGHT('000000000000' + CAST(code AS varchar(12)), 12) AS uniqueidentifier),
       code, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
       requested_at, accepted_at, completed_at,
       CASE ket_cuc
         WHEN 'requested'  THEN requested_at
         WHEN 'searching'  THEN DATEADD(second, 2, requested_at)
         WHEN 'accepted'   THEN accepted_at
         WHEN 'arriving'   THEN DATEADD(second, 10, accepted_at)
         WHEN 'picked_up'  THEN picked_up_at
         WHEN 'in_transit' THEN DATEADD(second, 30, picked_up_at)
         WHEN 'completed'  THEN completed_at
         ELSE cancelled_at
       END,
       pickup_area_id
FROM #p02_plan
ORDER BY code;
SET @n = CONVERT(nvarchar(20), @@ROWCOUNT);
SET IDENTITY_INSERT toinoi.trips OFF;
EXEC lab.expect_value N'trips đã nạp', @n, N'3174';

-- Sau khi chèn tường minh giá trị lớn hơn, SQL Server tự đẩy IDENTITY lên giá
-- trị lớn nhất: chuyến tạo mới sẽ nhận 103174.
SET @n = CONVERT(nvarchar(20), IDENT_CURRENT(N'toinoi.trips'));
EXEC lab.expect_value N'IDENT_CURRENT(trips) sau khi nạp', @n, N'103173';

-- ---------------------------------------------------------------------------
-- Lịch sử trạng thái: bước 1..buoc_cuoi; chuyến huỷ thêm dòng `cancelled`.
-- Thêm theo (mã chuyến, thứ tự bước) ⇒ id tăng tất định.
INSERT INTO toinoi.trip_events (trip_id, status, occurred_at)
SELECT CAST('70000000-0000-4000-8000-' + RIGHT('000000000000' + CAST(p.code AS varchar(12)), 12) AS uniqueidentifier),
       e.status, e.occurred_at
FROM #p02_plan AS p
CROSS APPLY (VALUES
    (1, 'requested',  p.requested_at),
    (2, 'searching',  DATEADD(second, 2, p.requested_at)),
    (3, 'accepted',   p.accepted_at),
    (4, 'arriving',   DATEADD(second, 10, p.accepted_at)),
    (5, 'picked_up',  p.picked_up_at),
    (6, 'in_transit', DATEADD(second, 30, p.picked_up_at)),
    (7, 'completed',  p.completed_at),
    (8, 'cancelled',  p.cancelled_at)
) AS e (buoc, status, occurred_at)
WHERE e.buoc <= p.buoc_cuoi OR (e.buoc = 8 AND p.status = 'cancelled')
ORDER BY p.code, e.buoc;
SET @n = CONVERT(nvarchar(20), @@ROWCOUNT);
EXEC lab.expect_value N'trip_events đã nạp', @n, N'20685';

-- ---------------------------------------------------------------------------
-- Đánh giá: ~70 % chuyến hoàn thành. ~6 % bỏ trống sao; tip ~30 % bỏ qua
-- (NULL), ~45 % chọn 0 đ, còn lại 5.000 / 10.000 / 20.000 đ.
INSERT INTO toinoi.trip_ratings (trip_id, stars, tip, rated_at)
SELECT t.id,
       CASE WHEN x.v < 6 THEN NULL WHEN x.v < 62 THEN 5 WHEN x.v < 86 THEN 4
            WHEN x.v < 93 THEN 3 WHEN x.v < 97 THEN 2 ELSE 1 END,
       CASE WHEN x.tp < 30 THEN NULL WHEN x.tp < 75 THEN 0 WHEN x.tp < 90 THEN 5000
            WHEN x.tp < 97 THEN 10000 ELSE 20000 END,
       DATEADD(minute, 1 + lab.p02_hash(t.code, 18) % 10, t.completed_at)
FROM toinoi.trips AS t
CROSS APPLY (SELECT lab.p02_hash(t.code, 13) % 100 AS v, lab.p02_hash(t.code, 14) % 100 AS tp) AS x
WHERE t.status = 'completed' AND lab.p02_hash(t.code, 12) % 100 < 70
ORDER BY t.code;
SET @n = CONVERT(nvarchar(20), @@ROWCOUNT);
EXEC lab.expect_value N'trip_ratings đã nạp', @n, N'1891';

-- ---------------------------------------------------------------------------
-- Thanh toán bình thường (trước khi cài sai lệch):
--   * chuyến hoàn thành, cước > 0: tiền mặt ~40 %, ví ~35 %, thẻ ~25 %;
--     ví/thẻ ~4 %: lần đầu failed, 1 phút sau thu lại thành công;
--     ví ~5 % (cước > 40.000): trả một phần bằng ví, phần còn lại bằng thẻ;
--   * chuyến cước 0 đ: không có giao dịch nào;
--   * chuyến huỷ SAU khi nhận, trả thẻ, ~50 %: đã trừ lúc nhận rồi hoàn đủ;
--   * chuyến đang chạy: chưa có giao dịch.
SELECT t.code, t.id AS trip_id, t.status, t.fare, t.accepted_at, t.completed_at, t.updated_at,
       CASE WHEN lab.p02_hash(t.code, 20) % 100 < 40 THEN 'cash'
            WHEN lab.p02_hash(t.code, 20) % 100 < 75 THEN 'wallet'
            ELSE 'card' END AS method,
       CASE WHEN lab.p02_hash(t.code, 21) % 100 < 4 THEN 1 ELSE 0 END AS lan_dau_loi,
       CASE WHEN lab.p02_hash(t.code, 22) % 100 < 5 THEN 1 ELSE 0 END AS tra_mot_phan,
       10000 + lab.p02_hash(t.code, 24) % 3 * 5000 AS phan_the,
       CASE WHEN lab.p02_hash(t.code, 23) % 100 < 50 THEN 1 ELSE 0 END AS tru_truoc
INTO #p02_pay_plan
FROM toinoi.trips AS t
WHERE t.status IN ('completed', 'cancelled');

INSERT INTO toinoi.payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
SELECT trip_id, kind, method, status, amount,
       CASE method WHEN 'wallet' THEN 'VX-' WHEN 'card' THEN 'TH-' END + CAST(code AS varchar(12)) + '-' + hau_to,
       paid_at
FROM (
    -- (a) lần thu lỗi trước đó
    SELECT code, 1 AS thu_tu, trip_id, 'charge' AS kind, method, 'failed' AS status, fare AS amount,
           '1' AS hau_to, completed_at AS paid_at
    FROM #p02_pay_plan
    WHERE status = 'completed' AND fare > 0 AND method <> 'cash' AND lan_dau_loi = 1
    UNION ALL
    -- (b) lần thu thành công (trả một phần: phần ví)
    SELECT code, 2, trip_id, 'charge',
           CASE WHEN tmp_hop_le = 1 THEN 'wallet' ELSE method END, 'succeeded',
           CASE WHEN tmp_hop_le = 1 THEN fare - phan_the ELSE fare END,
           CASE WHEN lan_dau_loi = 1 AND method <> 'cash' THEN '2' ELSE '1' END,
           DATEADD(minute, CASE WHEN lan_dau_loi = 1 AND method <> 'cash' THEN 1 ELSE 0 END, completed_at)
    FROM (SELECT *, CASE WHEN method = 'wallet' AND tra_mot_phan = 1 AND lan_dau_loi = 0 AND fare > 40000
                         THEN 1 ELSE 0 END AS tmp_hop_le
          FROM #p02_pay_plan) AS x
    WHERE status = 'completed' AND fare > 0
    UNION ALL
    -- (c) trả một phần: phần thẻ
    SELECT code, 3, trip_id, 'charge', 'card', 'succeeded', phan_the, '2', DATEADD(second, 20, completed_at)
    FROM #p02_pay_plan
    WHERE status = 'completed' AND fare > 40000 AND method = 'wallet' AND tra_mot_phan = 1 AND lan_dau_loi = 0
    UNION ALL
    -- (d) chuyến huỷ sau khi nhận: trừ tiền thẻ lúc nhận…
    SELECT code, 4, trip_id, 'charge', 'card', 'succeeded', fare, '1', accepted_at
    FROM #p02_pay_plan
    WHERE status = 'cancelled' AND accepted_at IS NOT NULL AND method = 'card' AND tru_truoc = 1 AND fare > 0
    UNION ALL
    -- (e) … rồi hoàn đủ lúc huỷ
    SELECT code, 5, trip_id, 'refund', 'card', 'succeeded', fare, 'R', updated_at
    FROM #p02_pay_plan
    WHERE status = 'cancelled' AND accepted_at IS NOT NULL AND method = 'card' AND tru_truoc = 1 AND fare > 0
) AS g
ORDER BY code, thu_tu;
SET @n = CONVERT(nvarchar(20), @@ROWCOUNT);
EXEC lab.expect_value N'payments thường đã nạp', @n, N'2821';

-- ---------------------------------------------------------------------------
-- Các ca "khó nhưng ĐÚNG" — đối soát không được báo nhầm (MS-020):
--   A1 103165: tiền mặt thu thiếu 5.000 đ, CSKH trừ ví 5.000 đ một giờ sau
--      (đã sang ngày 31/10) ⇒ thực thu vẫn = cước, nhưng tiền nằm ở hai ngày.
--   A2 102235: cước sửa từ 82.000 xuống 72.000; ví đã trừ 82.000 nên CSKH ghi
--      adjustment −10.000 ⇒ thực thu = cước mới.
--   P1 103162: một giao dịch thẻ `pending` thừa bên cạnh lần thu thành công.
UPDATE toinoi.payments SET amount = 23000
WHERE trip_id = '70000000-0000-4000-8000-000000103165' AND kind = 'charge' AND method = 'cash' AND amount = 28000;
SET @n = CONVERT(nvarchar(20), @@ROWCOUNT);
EXEC lab.expect_value N'A1 103165 thu thiếu', @n, N'1';

UPDATE toinoi.payments SET amount = 82000
WHERE trip_id = '70000000-0000-4000-8000-000000102235' AND kind = 'charge' AND method = 'wallet' AND amount = 72000;
SET @n = CONVERT(nvarchar(20), @@ROWCOUNT);
EXEC lab.expect_value N'A2 102235 ví trừ cước cũ', @n, N'1';

INSERT INTO toinoi.payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
SELECT t.id, v.kind, v.method, v.status, v.amount, v.provider_ref, DATEADD(minute, v.tre_phut, t.completed_at)
FROM (VALUES
    (102235, 'adjustment', 'wallet', 'succeeded', -10000, 'VX-102235-A', 120),
    (103165, 'adjustment', 'wallet', 'succeeded',   5000, 'VX-103165-A', 60),
    (103162, 'charge',     'card',   'pending',    57000, 'TH-103162-P', 2)
) AS v (code, kind, method, status, amount, provider_ref, tre_phut)
JOIN toinoi.trips AS t ON t.code = v.code
ORDER BY v.code;
SET @n = CONVERT(nvarchar(20), @@ROWCOUNT);
EXEC lab.expect_value N'ca khó: giao dịch thêm', @n, N'3';

-- ---------------------------------------------------------------------------
-- SAI LỆCH CÀI SẴN cho bài thực hành đối soát (MS-020) — đúng 7 chuyến:
--   D1 100724 THIẾU THANH TOÁN  tài xế không ghi nhận tiền mặt (xoá giao dịch)
--   D2 101084 THIẾU THANH TOÁN  thẻ bị từ chối, không ai thu lại
--   D3 101310 THU TRÙNG         ví trừ hai lần, 3 giây sau
--   D4 101987 HOÀN KHÔNG CÓ THU chuyến huỷ trước khi nhận nhưng ví lại hoàn 72.000
--   D5 102443 LỆCH SỐ TIỀN      tài xế nhập 45.000 thay vì 54.000 (đảo chữ số)
--   D6 102642 LỆCH SỐ TIỀN      thẻ trừ 380.000 thay vì 38.000 (thừa một số 0)
--   D7 102739 THU CHUYẾN HUỶ    chuyến huỷ sau khi nhận nhưng ví vẫn bị trừ
DELETE FROM toinoi.payments WHERE trip_id = '70000000-0000-4000-8000-000000100724';
SET @n = CONVERT(nvarchar(20), @@ROWCOUNT);
EXEC lab.expect_value N'D1 100724', @n, N'1';

UPDATE toinoi.payments SET status = 'failed'
WHERE trip_id = '70000000-0000-4000-8000-000000101084' AND status = 'succeeded';
SET @n = CONVERT(nvarchar(20), @@ROWCOUNT);
EXEC lab.expect_value N'D2 101084', @n, N'1';

UPDATE toinoi.payments
SET amount = CASE trip_id WHEN '70000000-0000-4000-8000-000000102443' THEN 45000
                          WHEN '70000000-0000-4000-8000-000000102642' THEN 380000 END
WHERE trip_id IN ('70000000-0000-4000-8000-000000102443', '70000000-0000-4000-8000-000000102642')
  AND kind = 'charge' AND status = 'succeeded';
SET @n = CONVERT(nvarchar(20), @@ROWCOUNT);
EXEC lab.expect_value N'D5, D6', @n, N'2';

INSERT INTO toinoi.payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
SELECT t.id, v.kind, 'wallet', 'succeeded', v.amount, v.provider_ref,
       CASE v.code
         WHEN 101310 THEN DATEADD(second, 3, (SELECT p.paid_at FROM toinoi.payments AS p
                                              WHERE p.trip_id = t.id AND p.status = 'succeeded'))
         WHEN 101987 THEN DATEADD(hour, 1, t.updated_at)
         WHEN 102739 THEN t.accepted_at
       END
FROM (VALUES
    (101310, 'charge', 85000, 'VX-101310-2'),
    (101987, 'refund', 72000, 'VX-101987-R'),
    (102739, 'charge', 96000, 'VX-102739-1')
) AS v (code, kind, amount, provider_ref)
JOIN toinoi.trips AS t ON t.code = v.code
ORDER BY v.code;
SET @n = CONVERT(nvarchar(20), @@ROWCOUNT);
EXEC lab.expect_value N'D3, D4, D7', @n, N'3';
GO

-- ---------------------------------------------------------------------------
-- Dọn công cụ sinh dữ liệu: dữ liệu đã nằm trong bảng.
DROP TABLE IF EXISTS #p02_pay_plan;
DROP TABLE IF EXISTS #p02_plan;
DROP FUNCTION lab.p02_hash;
DROP FUNCTION lab.p02_phut;
GO

-- Số dòng sau khi nạp (02-kiem-tra-fixture.sql mô tả chi tiết hình dạng).
SELECT (SELECT COUNT(*) FROM toinoi.cities)       AS cities,
       (SELECT COUNT(*) FROM toinoi.areas)        AS areas,
       (SELECT COUNT(*) FROM toinoi.customers)    AS customers,
       (SELECT COUNT(*) FROM toinoi.drivers)      AS drivers,
       (SELECT COUNT(*) FROM toinoi.merchants)    AS merchants,
       (SELECT COUNT(*) FROM toinoi.trips)        AS trips,
       (SELECT COUNT(*) FROM toinoi.trip_events)  AS trip_events,
       (SELECT COUNT(*) FROM toinoi.trip_ratings) AS trip_ratings,
       (SELECT COUNT(*) FROM toinoi.payments)     AS payments;
DECLARE @tong nvarchar(20) = CONVERT(nvarchar(20), (SELECT SUM(CAST(fare AS bigint)) FROM toinoi.trips WHERE status = 'completed'));
EXEC lab.expect_value N'doanh thu chuyến hoàn thành (đ)', @tong, N'182287000';
GO
