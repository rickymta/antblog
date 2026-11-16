-- P02 · 12-group-by-rollup.sql — MS-012 `mssql-group-by-rollup-grouping-sets`
-- Tổng hợp: COUNT, SUM, GROUP BY, HAVING, ROLLUP, CUBE và GROUPING SETS —
-- báo cáo theo thành phố/dịch vụ có dòng tổng và MẪU SỐ RÕ RÀNG.
--
-- Quy ước báo cáo tháng 10/2026 (giống lab PostgreSQL PG-012):
--   * kỳ báo cáo = chuyến ĐẶT trong [2026-10-01 00:00, 2026-10-31 00:00) giờ VN
--     ⇒ 6 chuyến đang chạy (đặt sau nửa đêm 31/10) nằm ngoài kỳ;
--   * "đã kết thúc" = completed + cancelled — mẫu số của tỉ lệ huỷ;
--   * "doanh thu" = Σ cước chuyến completed (chưa đối chiếu tiền thật — MS-013/MS-020).
--
-- Chạy sau 00 → 02. File chỉ ĐỌC dữ liệu.

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

-- ---------------------------------------------------------------------------
-- 1. COUNT(*) đếm DÒNG; COUNT(cột) đếm giá trị KHÁC NULL; COUNT(DISTINCT cột)
--    đếm giá trị khác nhau. COUNT_BIG giống COUNT nhưng trả bigint.
SELECT COUNT(*)                    AS so_chuyen,
       COUNT(driver_id)            AS co_tai_xe,
       COUNT(merchant_id)          AS co_quan,
       COUNT(DISTINCT driver_id)   AS so_tai_xe_khac_nhau,
       COUNT(DISTINCT customer_id) AS so_khach_khac_nhau,
       COUNT_BIG(*)                AS so_chuyen_big
FROM toinoi.trips;

DECLARE @tx int = (SELECT COUNT(DISTINCT driver_id) FROM toinoi.trips),
        @kh int = (SELECT COUNT(DISTINCT customer_id) FROM toinoi.trips);
EXEC lab.expect_value N'1 số tài xế khác nhau', @tx, N'52';
EXEC lab.expect_value N'1 số khách khác nhau', @kh, N'576';

-- Kiểu kết quả: KHÁC PostgreSQL, SUM và AVG trên int vẫn trả int.
DECLARE @k_sum sysname, @k_avg sysname, @k_cnt sysname, @k_big sysname;
SELECT @k_sum = CAST(SQL_VARIANT_PROPERTY(SUM(fare), 'BaseType') AS sysname),
       @k_avg = CAST(SQL_VARIANT_PROPERTY(AVG(fare), 'BaseType') AS sysname),
       @k_cnt = CAST(SQL_VARIANT_PROPERTY(COUNT(*), 'BaseType') AS sysname),
       @k_big = CAST(SQL_VARIANT_PROPERTY(COUNT_BIG(*), 'BaseType') AS sysname)
FROM toinoi.trips;
SELECT @k_sum AS kieu_sum, @k_avg AS kieu_avg, @k_cnt AS kieu_count, @k_big AS kieu_count_big;
EXEC lab.expect_value N'1 kiểu SUM(int)', @k_sum, N'int';
EXEC lab.expect_value N'1 kiểu AVG(int)', @k_avg, N'int';
EXEC lab.expect_value N'1 kiểu COUNT_BIG', @k_big, N'bigint';
GO

-- SUM(int) trả int ⇒ tràn khi tổng vượt 2.147.483.647 (lỗi 8115). "Ước doanh
-- thu năm = tháng 10 × 12" vượt ngưỡng đó.
EXEC lab.expect_error N'1 SUM(fare * 12) tràn int',
     N'SELECT SUM(fare * 12) FROM toinoi.trips
       WHERE status = ''completed''
         AND requested_at >= ''2026-10-01 00:00 +07:00'' AND requested_at < ''2026-10-31 00:00 +07:00''',
     8115, N'int';

DECLARE @nam bigint = (SELECT SUM(CAST(fare AS bigint) * 12) FROM toinoi.trips
                       WHERE status = 'completed'
                         AND requested_at >= '2026-10-01 00:00 +07:00' AND requested_at < '2026-10-31 00:00 +07:00');
EXEC lab.expect_value N'1 SUM(CAST(fare AS bigint) * 12)', @nam, N'2187444000';
GO

-- ---------------------------------------------------------------------------
-- 2. Bảng chính của tháng: thành phố × dịch vụ. Mỗi cột ghi rõ mẫu số.
--    SQL Server không có FILTER (WHERE …) ⇒ SUM(CASE WHEN … THEN 1 ELSE 0 END).
SELECT c.code                                                       AS tp,
       t.kind,
       COUNT(*)                                                     AS da_ket_thuc,
       SUM(CASE WHEN t.status = 'completed' THEN 1 ELSE 0 END)      AS hoan_thanh,
       SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END)      AS huy,
       CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END)
            / COUNT(*) AS decimal(4, 1))                            AS ty_le_huy_pct,   -- huỷ / đã kết thúc
       SUM(CASE WHEN t.status = 'completed' THEN t.fare END)        AS doanh_thu,
       CAST(ROUND(AVG(CASE WHEN t.status = 'completed' THEN CAST(t.fare AS decimal(12, 2)) END), 0)
            AS int)                                                 AS cuoc_tb          -- / hoàn thành
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY c.code, t.kind
ORDER BY c.code, t.kind;

DECLARE @n int, @sg_parcel decimal(4, 1);
SELECT @n = COUNT(*) FROM toinoi.trips
WHERE requested_at >= '2026-10-01 00:00 +07:00' AND requested_at < '2026-10-31 00:00 +07:00';
SELECT @sg_parcel = CAST(100.0 * SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) AS decimal(4, 1))
FROM toinoi.trips
WHERE city_id = 1 AND kind = 'parcel'
  AND requested_at >= '2026-10-01 00:00 +07:00' AND requested_at < '2026-10-31 00:00 +07:00';
EXEC lab.expect_value N'2 chuyến trong kỳ', @n, N'3168';
EXEC lab.expect_value N'2 tỉ lệ huỷ SG parcel', @sg_parcel, N'21.9';
GO

-- ---------------------------------------------------------------------------
-- 3. Ba lỗi mẫu số trên cùng một dữ liệu, cộng thêm lỗi riêng của SQL Server:
--    a) chia số nguyên: int / int là phép chia NGUYÊN ⇒ 0 hoặc cắt phần lẻ;
--    b) cước TB tính trên MỌI chuyến (gồm chuyến huỷ — chưa ai trả tiền);
--       b0) AVG(fare) với fare int CẮT phần lẻ (không làm tròn);
--    c) tỉ lệ huỷ chia cho số chuyến HOÀN THÀNH thay vì số chuyến đã kết thúc.
SELECT c.code                                                                        AS tp,
       SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*)            AS a_chia_nguyen,
       100 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*)      AS a_pct_nguyen,
       CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END)
            / COUNT(*) AS decimal(5, 2))                                             AS a_pct_dung,
       AVG(t.fare)                                                                   AS b_avg_int_moi_chuyen,
       CAST(ROUND(AVG(CAST(t.fare AS decimal(12, 2))), 0) AS int)                    AS b_cuoc_tb_sai,
       CAST(ROUND(AVG(CASE WHEN t.status = 'completed' THEN CAST(t.fare AS decimal(12, 2)) END), 0)
            AS int)                                                                  AS b_cuoc_tb_dung,
       CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END)
            / SUM(CASE WHEN t.status = 'completed' THEN 1 ELSE 0 END) AS decimal(5, 2)) AS c_pct_sai_mau
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY c.code
ORDER BY c.code;

DECLARE @a int, @a_dung decimal(5, 2), @b_sai int, @b_dung int, @b_int int;
SELECT @a = 100 * SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*),
       @a_dung = CAST(100.0 * SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) AS decimal(5, 2)),
       @b_int = AVG(fare),
       @b_sai = CAST(ROUND(AVG(CAST(fare AS decimal(12, 2))), 0) AS int),
       @b_dung = CAST(ROUND(AVG(CASE WHEN status = 'completed' THEN CAST(fare AS decimal(12, 2)) END), 0) AS int)
FROM toinoi.trips
WHERE city_id = 1
  AND requested_at >= '2026-10-01 00:00 +07:00' AND requested_at < '2026-10-31 00:00 +07:00';
EXEC lab.expect_value N'3a SG tỉ lệ huỷ chia nguyên', @a, N'14';
EXEC lab.expect_value N'3a SG tỉ lệ huỷ đúng', @a_dung, N'14.10';
EXEC lab.expect_value N'3b SG cước TB gồm chuyến huỷ', @b_sai, N'68269';
EXEC lab.expect_value N'3b SG cước TB đúng', @b_dung, N'67451';

-- AVG(int) CẮT phần lẻ, không làm tròn: Đà Nẵng 68.010,6… → 68010.
DECLARE @dn_int int, @dn_round int;
SELECT @dn_int = AVG(fare), @dn_round = CAST(ROUND(AVG(CAST(fare AS decimal(12, 2))), 0) AS int)
FROM toinoi.trips
WHERE city_id = 3
  AND requested_at >= '2026-10-01 00:00 +07:00' AND requested_at < '2026-10-31 00:00 +07:00';
EXEC lab.expect_value N'3b0 DN AVG(fare) int (cắt)', @dn_int, N'68010';
EXEC lab.expect_value N'3b0 DN AVG decimal làm tròn', @dn_round, N'68011';
GO

-- ---------------------------------------------------------------------------
-- 4. Mọi cột trong SELECT phải nằm trong GROUP BY hoặc trong hàm tổng hợp.
EXEC lab.expect_error N'4 cột không nhóm, không tổng hợp',
     N'SELECT c.code, t.kind, COUNT(*) FROM toinoi.trips AS t JOIN toinoi.cities AS c ON c.id = t.city_id GROUP BY c.code',
     8120, N'kind';

-- KHÁC PostgreSQL: nhóm theo KHOÁ CHÍNH cũng không cho chọn cột khác của bảng
-- đó. SQL Server không suy phụ thuộc hàm ⇒ liệt kê đủ cột trong GROUP BY.
EXEC lab.expect_error N'4 nhóm theo khoá chính vẫn lỗi',
     N'SELECT ci.id, ci.code, ci.name, COUNT(*) FROM toinoi.trips AS t JOIN toinoi.cities AS ci ON ci.id = t.city_id GROUP BY ci.id',
     8120, N'code';

SELECT ci.id, ci.code, ci.name, COUNT(*) AS so_chuyen
FROM toinoi.trips AS t JOIN toinoi.cities AS ci ON ci.id = t.city_id
GROUP BY ci.id, ci.code, ci.name
ORDER BY ci.id;
GO

-- ---------------------------------------------------------------------------
-- 5. WHERE lọc DÒNG trước khi gom nhóm; HAVING lọc NHÓM sau khi gom.
--    Nhóm thành phố × dịch vụ có tỉ lệ huỷ trên 15 %. HAVING không dùng được
--    tên cột đặt ở SELECT (lỗi 207) ⇒ viết lại biểu thức.
SELECT c.code AS tp, t.kind, COUNT(*) AS da_ket_thuc,
       CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) AS decimal(4, 1)) AS ty_le_huy_pct
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY c.code, t.kind
HAVING 100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) > 15
ORDER BY ty_le_huy_pct DESC, c.code, t.kind;

EXEC lab.expect_error N'5 HAVING dùng tên cột của SELECT',
     N'SELECT kind, COUNT(*) AS so_chuyen FROM toinoi.trips GROUP BY kind HAVING so_chuyen > 500',
     207, N'so_chuyen';

-- Tài xế hoàn thành từ 63 chuyến trở lên trong tháng. Điều kiện trên DÒNG
-- (status = completed) ở WHERE; điều kiện trên NHÓM (COUNT >= 63) ở HAVING.
SELECT d.code AS tai_xe, c.code AS tp, COUNT(*) AS hoan_thanh, SUM(t.fare) AS doanh_thu
FROM toinoi.trips AS t
JOIN toinoi.drivers AS d ON d.id = t.driver_id
JOIN toinoi.cities AS c ON c.id = d.city_id
WHERE t.status = 'completed'
  AND t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY d.code, c.code
HAVING COUNT(*) >= 63
ORDER BY hoan_thanh DESC, tai_xe;

-- STRING_AGG gom danh sách đó thành MỘT dòng mỗi thành phố, thứ tự xác định
-- bằng WITHIN GROUP (ORDER BY …).
SELECT tp, COUNT(*) AS so_tai_xe,
       STRING_AGG(CONVERT(varchar(10), tai_xe), ', ') WITHIN GROUP (ORDER BY tai_xe) AS danh_sach
FROM (SELECT d.code AS tai_xe, c.code AS tp
      FROM toinoi.trips AS t
      JOIN toinoi.drivers AS d ON d.id = t.driver_id
      JOIN toinoi.cities AS c ON c.id = d.city_id
      WHERE t.status = 'completed'
        AND t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
      GROUP BY d.code, c.code
      HAVING COUNT(*) >= 63) AS x
GROUP BY tp
ORDER BY tp;

DECLARE @ds varchar(200) =
   (SELECT STRING_AGG(CONVERT(varchar(10), tai_xe), ', ') WITHIN GROUP (ORDER BY tai_xe)
    FROM (SELECT d.code AS tai_xe
          FROM toinoi.trips AS t JOIN toinoi.drivers AS d ON d.id = t.driver_id
          WHERE t.status = 'completed'
            AND t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
          GROUP BY d.code
          HAVING COUNT(*) >= 63) AS x);
EXEC lab.expect_value N'5 tài xế >= 63 chuyến', @ds, N'8101, 8107, 8108, 8109, 8120, 8124, 8210';
GO

-- ---------------------------------------------------------------------------
-- 6. Tổng phụ và tổng cộng trong MỘT câu: ROLLUP (tp, kind) sinh các nhóm
--    (tp, kind), (tp) và (). Cột bị "cuộn" hiện NULL — GROUPING(cột) = 1.
SELECT CASE WHEN GROUPING(c.code) = 1 THEN N'TỔNG' ELSE c.code END              AS tp,
       CASE WHEN GROUPING(t.kind) = 1 THEN N'(mọi dịch vụ)' ELSE t.kind END     AS kind,
       SUM(CASE WHEN t.status = 'completed' THEN 1 ELSE 0 END)                  AS hoan_thanh,
       SUM(CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END)             AS doanh_thu
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY ROLLUP (c.code, t.kind)
ORDER BY GROUPING(c.code), c.code, GROUPING(t.kind), t.kind;

-- Tổng cộng của ROLLUP phải bằng một truy vấn tổng độc lập (đối chiếu chéo).
DECLARE @r_ht int, @r_dt int, @d_ht int, @d_dt int, @so_dong int;
SELECT @r_ht = MAX(CASE WHEN g = 3 THEN ht END), @r_dt = MAX(CASE WHEN g = 3 THEN dt END), @so_dong = COUNT(*)
FROM (SELECT GROUPING_ID(c.code, t.kind) AS g,
             SUM(CASE WHEN t.status = 'completed' THEN 1 ELSE 0 END) AS ht,
             SUM(CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END) AS dt
      FROM toinoi.trips AS t JOIN toinoi.cities AS c ON c.id = t.city_id
      WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
      GROUP BY ROLLUP (c.code, t.kind)) AS x;
SELECT @d_ht = COUNT(*), @d_dt = SUM(fare) FROM toinoi.trips
WHERE status = 'completed'
  AND requested_at >= '2026-10-01 00:00 +07:00' AND requested_at < '2026-10-31 00:00 +07:00';
EXEC lab.expect_value N'6 ROLLUP số dòng (9 + 3 + 1)', @so_dong, N'13';
EXEC lab.expect_value N'6 ROLLUP tổng hoàn thành', @r_ht, N'2707';
EXEC lab.expect_value N'6 ROLLUP tổng doanh thu', @r_dt, N'182287000';
EXEC lab.expect_value N'6 truy vấn độc lập: hoàn thành', @d_ht, N'2707';
EXEC lab.expect_value N'6 truy vấn độc lập: doanh thu', @d_dt, N'182287000';
GO

-- NULL của ROLLUP và NULL của dữ liệu: khu đón ở Đà Nẵng (4 chuyến mất GPS).
-- Gắn nhãn bằng ISNULL ⇒ HAI dòng "TỔNG"; dùng GROUPING() thì đúng.
SELECT ISNULL(z.code, 'TONG')                                                  AS nhan_isnull,
       CASE WHEN GROUPING(z.code) = 1 THEN N'TỔNG'
            ELSE COALESCE(z.code, N'không rõ (mất GPS)') END                   AS nhan_grouping,
       COUNT(*)                                                                AS so_chuyen
FROM toinoi.trips AS t
LEFT JOIN toinoi.areas AS z ON z.id = t.pickup_area_id
WHERE t.city_id = 3
GROUP BY ROLLUP (z.code)
ORDER BY GROUPING(z.code), z.code;

DECLARE @hai_tong int = (SELECT COUNT(*) FROM (
          SELECT ISNULL(z.code, 'TONG') AS nhan
          FROM toinoi.trips AS t LEFT JOIN toinoi.areas AS z ON z.id = t.pickup_area_id
          WHERE t.city_id = 3
          GROUP BY ROLLUP (z.code)) AS x WHERE nhan = 'TONG');
EXEC lab.expect_value N'6 số dòng nhãn TONG khi dùng ISNULL', @hai_tong, N'2';
GO

-- ---------------------------------------------------------------------------
-- 7. GROUPING SETS: chọn đúng các cấp tổng cần — ở đây (tp, kind), (tp),
--    (kind) và (). CUBE (tp, kind) sinh đúng bốn cấp đó. GROUPING_ID ghép các
--    bit GROUPING thành một số: 0 = chi tiết, 1 = theo tp, 2 = theo kind, 3 = tổng.
SELECT GROUPING_ID(c.code, t.kind)                                              AS cap,
       CASE WHEN GROUPING(c.code) = 1 THEN N'*' ELSE c.code END                 AS tp,
       CASE WHEN GROUPING(t.kind) = 1 THEN N'*' ELSE t.kind END                 AS kind,
       COUNT(*)                                                                 AS da_ket_thuc,
       CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) AS decimal(4, 1)) AS ty_le_huy_pct
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY GROUPING SETS ((c.code, t.kind), (c.code), (t.kind), ())
HAVING GROUPING_ID(c.code, t.kind) > 0
ORDER BY cap, tp, kind;

DECLARE @gs int, @cube int, @tong_pct decimal(5, 2);
SELECT @gs = COUNT(*) FROM (SELECT 1 AS x FROM toinoi.trips AS t JOIN toinoi.cities AS c ON c.id = t.city_id
      WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
      GROUP BY GROUPING SETS ((c.code, t.kind), (c.code), (t.kind), ())) AS g;
SELECT @cube = COUNT(*) FROM (SELECT 1 AS x FROM toinoi.trips AS t JOIN toinoi.cities AS c ON c.id = t.city_id
      WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
      GROUP BY CUBE (c.code, t.kind)) AS g;
SELECT @tong_pct = CAST(100.0 * SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) AS decimal(5, 2))
FROM toinoi.trips
WHERE requested_at >= '2026-10-01 00:00 +07:00' AND requested_at < '2026-10-31 00:00 +07:00';
EXEC lab.expect_value N'7 GROUPING SETS số dòng (9 + 3 + 3 + 1)', @gs, N'16';
EXEC lab.expect_value N'7 CUBE số dòng', @cube, N'16';
EXEC lab.expect_value N'7 tỉ lệ huỷ cả tháng', @tong_pct, N'14.55';

-- Tỉ lệ cấp trên KHÔNG phải trung bình cộng các tỉ lệ con (nhóm 72 chuyến
-- nặng ngang nhóm 717 chuyến).
DECLARE @tb_ti_le decimal(5, 2) = (
    SELECT CAST(AVG(pct) AS decimal(5, 2))
    FROM (SELECT 100.0 * SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) AS pct
          FROM toinoi.trips
          WHERE requested_at >= '2026-10-01 00:00 +07:00' AND requested_at < '2026-10-31 00:00 +07:00'
          GROUP BY city_id, kind) AS x);
EXEC lab.expect_value N'7 trung bình cộng 9 tỉ lệ (sai)', @tb_ti_le, N'14.35';
GO

-- ---------------------------------------------------------------------------
-- 8. Đánh giá theo thành phố: HAI mẫu số khác nhau trong cùng một dòng.
--    * độ phủ  = số chuyến có dòng đánh giá / số chuyến hoàn thành
--    * điểm TB = trung bình trên các lượt CÓ sao (AVG bỏ qua NULL)
SELECT c.code                                                                   AS tp,
       COUNT(*)                                                                 AS hoan_thanh,
       COUNT(r.trip_id)                                                         AS co_danh_gia,
       CAST(100.0 * COUNT(r.trip_id) / COUNT(*) AS decimal(4, 1))               AS do_phu_pct,
       COUNT(r.stars)                                                           AS co_sao,
       CAST(AVG(CAST(r.stars AS decimal(9, 4))) AS decimal(4, 2))               AS diem_tb
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
  AND t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY c.code
ORDER BY c.code;

DECLARE @dn decimal(4, 2) = (SELECT CAST(AVG(CAST(r.stars AS decimal(9, 4))) AS decimal(4, 2))
                             FROM toinoi.trips AS t JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
                             WHERE t.status = 'completed' AND t.city_id = 3
                               AND t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00');
EXEC lab.expect_value N'8 điểm TB Đà Nẵng', @dn, N'4.30';
GO

-- ---------------------------------------------------------------------------
-- 9. COUNT(DISTINCT …) không cộng được qua các nhóm: khách đi cả ride lẫn
--    food bị đếm ở cả hai dòng. ROLLUP tính lại từ dòng gốc ⇒ dòng TỔNG đúng.
SELECT CASE WHEN GROUPING(t.kind) = 1 THEN N'TỔNG' ELSE t.kind END            AS kind,
       COUNT(DISTINCT t.customer_id)                                           AS so_khach,
       COUNT(*)                                                                AS hoan_thanh,
       CAST(1.0 * COUNT(*) / COUNT(DISTINCT t.customer_id) AS decimal(4, 2))   AS chuyen_moi_khach
FROM toinoi.trips AS t
WHERE t.status = 'completed'
  AND t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY ROLLUP (t.kind)
ORDER BY GROUPING(t.kind), t.kind;

DECLARE @cong int, @that int;
SELECT @cong = SUM(CASE WHEN g = 0 THEN kh END), @that = MAX(CASE WHEN g = 1 THEN kh END)
FROM (SELECT GROUPING(t.kind) AS g, COUNT(DISTINCT t.customer_id) AS kh
      FROM toinoi.trips AS t
      WHERE t.status = 'completed'
        AND t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
      GROUP BY ROLLUP (t.kind)) AS x;
EXEC lab.expect_value N'9 cộng khách ba dịch vụ', @cong, N'1103';
EXEC lab.expect_value N'9 khách thật (dòng TỔNG)', @that, N'569';
GO

-- ===========================================================================
-- Bài tập (lời giải trong bài viết)
-- ===========================================================================

-- BT1. Bảng theo dịch vụ cho cả nước, có dòng tổng bằng ROLLUP.
SELECT CASE WHEN GROUPING(kind) = 1 THEN N'TỔNG' ELSE kind END                AS kind,
       COUNT(*)                                                                AS da_ket_thuc,
       SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END)                   AS hoan_thanh,
       SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END)                   AS huy,
       CAST(100.0 * SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) AS decimal(4, 1)) AS ty_le_huy_pct,
       SUM(CASE WHEN status = 'completed' THEN fare ELSE 0 END)                AS doanh_thu,
       CAST(ROUND(AVG(CASE WHEN status = 'completed' THEN CAST(fare AS decimal(12, 2)) END), 0) AS int) AS cuoc_tb
FROM toinoi.trips
WHERE requested_at >= '2026-10-01 00:00 +07:00' AND requested_at < '2026-10-31 00:00 +07:00'
GROUP BY ROLLUP (kind)
ORDER BY GROUPING(kind), kind;

DECLARE @food_pct decimal(4, 1) = (SELECT CAST(100.0 * SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) AS decimal(4, 1))
                                   FROM toinoi.trips WHERE kind = 'food'
                                     AND requested_at >= '2026-10-01 00:00 +07:00' AND requested_at < '2026-10-31 00:00 +07:00');
EXEC lab.expect_value N'BT1 tỉ lệ huỷ food', @food_pct, N'16.0';

DECLARE @tb3 decimal(4, 1) = (
    SELECT CAST(AVG(pct) AS decimal(4, 1))
    FROM (SELECT 100.0 * SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) AS pct
          FROM toinoi.trips
          WHERE requested_at >= '2026-10-01 00:00 +07:00' AND requested_at < '2026-10-31 00:00 +07:00'
          GROUP BY kind) AS x);
EXEC lab.expect_value N'BT1 trung bình cộng ba tỉ lệ (sai)', @tb3, N'14.8';
GO

-- BT2. Quán có ít nhất 50 đơn trong tháng và tỉ lệ huỷ từ 20 % trở lên.
SELECT m.name AS quan, c.code AS tp, COUNT(*) AS so_don,
       CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) AS decimal(4, 1)) AS ty_le_huy_pct
FROM toinoi.trips AS t
JOIN toinoi.merchants AS m ON m.id = t.merchant_id
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY m.id, m.name, c.code
HAVING COUNT(*) >= 50
   AND 100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) >= 20
ORDER BY ty_le_huy_pct DESC, quan;

DECLARE @so_quan int = (SELECT COUNT(*) FROM (
          SELECT m.id
          FROM toinoi.trips AS t JOIN toinoi.merchants AS m ON m.id = t.merchant_id
          WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
          GROUP BY m.id
          HAVING COUNT(*) >= 50
             AND 100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) >= 20) AS x);
EXEC lab.expect_value N'BT2 số quán', @so_quan, N'4';
GO

-- BT3. Tài xế có điểm TB dưới 4,0, chỉ xét người có ít nhất 35 lượt CÓ SAO.
SELECT d.code AS tai_xe, COUNT(*) AS hoan_thanh, COUNT(r.stars) AS co_sao,
       CAST(AVG(CAST(r.stars AS decimal(9, 4))) AS decimal(4, 2)) AS diem_tb
FROM toinoi.trips AS t
JOIN toinoi.drivers AS d ON d.id = t.driver_id
LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
  AND t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY d.code
HAVING COUNT(r.stars) >= 35 AND AVG(CAST(r.stars AS decimal(9, 4))) < 4
ORDER BY diem_tb, tai_xe;

DECLARE @dung int, @sai_mau int;
SELECT @dung = SUM(CASE WHEN co_sao >= 35 AND tb_dec < 4 THEN 1 ELSE 0 END),
       @sai_mau = SUM(CASE WHEN hoan_thanh >= 35 AND tb_dec < 4 THEN 1 ELSE 0 END)
FROM (SELECT d.code, COUNT(*) AS hoan_thanh, COUNT(r.stars) AS co_sao,
             AVG(CAST(r.stars AS decimal(9, 4))) AS tb_dec
      FROM toinoi.trips AS t
      JOIN toinoi.drivers AS d ON d.id = t.driver_id
      LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
      WHERE t.status = 'completed'
        AND t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
      GROUP BY d.code) AS x;
EXEC lab.expect_value N'BT3 đúng: >= 35 lượt có sao, TB < 4', @dung, N'4';
EXEC lab.expect_value N'BT3 sai: ngưỡng trên COUNT(*)', @sai_mau, N'6';
GO
