-- P02 · 11-null-case-isnull-coalesce.sql — MS-011 `mssql-null-case-isnull-coalesce`
-- NULL, CASE, ISNULL và COALESCE: phân biệt THIẾU DỮ LIỆU, KHÔNG ÁP DỤNG và
-- GIÁ TRỊ 0 trên SQL Server 2022.
--
-- Trong dữ liệu Tới Nơi, một ô trống có thể mang ba nghĩa khác nhau:
--   * chưa có / thiếu   — chuyến đang tìm tài xế; khách bỏ trống phần sao;
--                          app tài xế mất GPS nên không biết khu đón;
--   * không áp dụng     — đơn đặt xe thì không có quán; chuyến huỷ không được
--                          đánh giá;
--   * và số 0 THẬT      — chuyến khuyến mãi cước 0 đ; khách chọn "không tip".
-- SQL chỉ có một NULL cho hai nghĩa đầu; nghĩa nào phải suy từ các cột khác.
--
-- Chạy sau 00 → 02. File chỉ ĐỌC dữ liệu (bảng tạm #… tự mất khi hết phiên).
-- Dòng "Warning: Null value is eliminated…" là cảnh báo của SQL Server khi
-- hàm tổng hợp bỏ qua NULL (ANSI_WARNINGS ON) — mục 4 của bài giải thích.

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

-- ---------------------------------------------------------------------------
-- 1. So sánh với NULL không ra TRUE hay FALSE mà ra UNKNOWN. T-SQL không có
--    kiểu boolean để in ra, nên dùng CASE để "nhìn" kết quả của điều kiện.
--    IS [NOT] DISTINCT FROM có từ SQL Server 2022.
SELECT CASE WHEN NULL = NULL THEN 'TRUE' WHEN NOT (NULL = NULL) THEN 'FALSE' ELSE 'UNKNOWN' END AS null_bang_null,
       CASE WHEN NULL <> 1   THEN 'TRUE' WHEN NOT (NULL <> 1)   THEN 'FALSE' ELSE 'UNKNOWN' END AS null_khac_1,
       CASE WHEN NULL IS NULL THEN 'TRUE' ELSE 'FALSE' END                                       AS null_is_null,
       CASE WHEN NULL IS DISTINCT FROM NULL THEN 'TRUE' ELSE 'FALSE' END                         AS null_distinct_null,
       CASE WHEN 1 IS DISTINCT FROM NULL THEN 'TRUE' ELSE 'FALSE' END                            AS mot_distinct_null;

DECLARE @bang_null int = (SELECT COUNT(*) FROM toinoi.trips WHERE driver_id = NULL),
        @is_null   int = (SELECT COUNT(*) FROM toinoi.trips WHERE driver_id IS NULL);
SELECT @bang_null AS bang_null, @is_null AS is_null;
EXEC lab.expect_value N'1 driver_id = NULL', @bang_null, N'0';
EXEC lab.expect_value N'1 driver_id IS NULL', @is_null, N'297';
GO

-- SET ANSI_NULLS OFF (đã lỗi thời, sẽ thành lỗi ở bản sau) làm `= NULL` khớp
-- NULL. Chỉ để biết khi gặp mã cũ; mặc định của mọi driver hiện đại là ON.
SET ANSI_NULLS OFF;
DECLARE @ansi_off int = (SELECT COUNT(*) FROM toinoi.trips WHERE driver_id = NULL);
SET ANSI_NULLS ON;
EXEC lab.expect_value N'1 ANSI_NULLS OFF: driver_id = NULL', @ansi_off, N'297';
GO

-- ---------------------------------------------------------------------------
-- 2. Một NULL, nhiều nghĩa: đọc driver_id cùng với status bằng CASE.
--    CASE dừng ở nhánh ĐẦU TIÊN đúng; nhánh ELSE bắt dữ liệu sai quy tắc.
--    SQL Server không cho GROUP BY theo số thứ tự cột ⇒ gom trong bảng dẫn xuất.
SELECT y_nghia_cot_driver_id, COUNT(*) AS so_chuyen
FROM (SELECT CASE
               WHEN driver_id IS NOT NULL                 THEN N'có tài xế'
               WHEN status IN ('requested', 'searching')  THEN N'chưa có — đang tìm tài xế'
               WHEN status = 'cancelled'                  THEN N'không áp dụng — huỷ trước khi có người nhận'
               ELSE N'SAI QUY TẮC — lẽ ra phải có tài xế'
             END AS y_nghia_cot_driver_id
      FROM toinoi.trips) AS x
GROUP BY y_nghia_cot_driver_id
ORDER BY so_chuyen DESC;

-- Đảo thứ tự nhánh: đặt 'cancelled' lên đầu ⇒ nhóm "huỷ trước khi nhận" nuốt
-- cả 166 chuyến huỷ SAU khi đã có tài xế.
DECLARE @dung int = (SELECT COUNT(*) FROM toinoi.trips WHERE driver_id IS NULL AND status = 'cancelled');
DECLARE @sai  int = (SELECT SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END) FROM toinoi.trips);
EXEC lab.expect_value N'2 huỷ trước khi nhận (nhánh đúng thứ tự)', @dung, N'295';
EXEC lab.expect_value N'2 huỷ trước khi nhận (nhánh cancelled đặt đầu)', @sai, N'461';

-- CASE dạng đơn giản so sánh bằng `=` ⇒ WHEN NULL không bao giờ khớp.
DECLARE @case_don_gian int = (SELECT SUM(CASE stars WHEN NULL THEN 1 ELSE 0 END) FROM toinoi.trip_ratings);
DECLARE @case_tim_kiem int = (SELECT SUM(CASE WHEN stars IS NULL THEN 1 ELSE 0 END) FROM toinoi.trip_ratings);
EXEC lab.expect_value N'2 CASE stars WHEN NULL', @case_don_gian, N'0';
EXEC lab.expect_value N'2 CASE WHEN stars IS NULL', @case_tim_kiem, N'153';
GO

-- ---------------------------------------------------------------------------
-- 3. Đánh giá có BỐN trường hợp, hai trong số đó cùng ra NULL sau LEFT JOIN:
--    "không có dòng đánh giá" và "có dòng nhưng stars NULL". Phân biệt bằng
--    cột KHÔNG BAO GIỜ NULL của bảng bên phải (khoá chính r.trip_id).
SELECT tinh_trang_danh_gia, COUNT(*) AS so_chuyen
FROM (SELECT CASE
               WHEN t.status = 'cancelled' THEN N'không áp dụng — chuyến huỷ'
               WHEN r.trip_id IS NULL      THEN N'thiếu — chưa đánh giá'
               WHEN r.stars IS NULL        THEN N'thiếu — mở màn đánh giá, bỏ trống sao'
               ELSE N'có sao'
             END AS tinh_trang_danh_gia
      FROM toinoi.trips AS t
      LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
      WHERE t.status IN ('completed', 'cancelled')) AS x
GROUP BY tinh_trang_danh_gia
ORDER BY so_chuyen DESC;

DECLARE @chua_danh_gia int = (SELECT COUNT(*) FROM toinoi.trips AS t
                              LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
                              WHERE t.status = 'completed' AND r.trip_id IS NULL);
EXEC lab.expect_value N'3 chuyến hoàn thành chưa đánh giá', @chua_danh_gia, N'816';
GO

-- ---------------------------------------------------------------------------
-- 4. Hàm tổng hợp BỎ QUA NULL — và AVG trên cột số nguyên trả SỐ NGUYÊN.
--    stars là tinyint ⇒ AVG(stars) trả int, cắt phần thập phân: 4,2186 → 4.
--    ISNULL(stars, 0) biến "không biết" thành "0 sao" và kéo điểm xuống.
SELECT COUNT(*)                                                         AS so_danh_gia,
       COUNT(stars)                                                     AS co_sao,
       AVG(stars)                                                       AS avg_int,
       CAST(AVG(CAST(stars AS decimal(9, 4))) AS decimal(4, 2))         AS diem_tb_dung,
       CAST(AVG(CAST(ISNULL(stars, 0) AS decimal(9, 4))) AS decimal(4, 2)) AS diem_tb_sai
FROM toinoi.trip_ratings;

DECLARE @avg_int int, @dung decimal(4, 2), @sai decimal(4, 2), @kieu sysname;
SELECT @avg_int = AVG(stars),
       @dung = CAST(AVG(CAST(stars AS decimal(9, 4))) AS decimal(4, 2)),
       @sai  = CAST(AVG(CAST(ISNULL(stars, 0) AS decimal(9, 4))) AS decimal(4, 2)),
       @kieu = CAST(SQL_VARIANT_PROPERTY(AVG(stars), 'BaseType') AS sysname)
FROM toinoi.trip_ratings;
EXEC lab.expect_value N'4 AVG(stars) kiểu', @kieu, N'int';
EXEC lab.expect_value N'4 AVG(stars) cắt thành số nguyên', @avg_int, N'4';
EXEC lab.expect_value N'4 điểm TB đúng (decimal)', @dung, N'4.22';
EXEC lab.expect_value N'4 điểm TB sai (ISNULL 0)', @sai, N'3.88';

-- Ba điều kiện "= 5", "<> 5" và "IS NULL" mới phủ hết.
SELECT COUNT(*)                                         AS tong,
       SUM(CASE WHEN stars = 5 THEN 1 ELSE 0 END)       AS nam_sao,
       SUM(CASE WHEN stars <> 5 THEN 1 ELSE 0 END)      AS khac_nam_sao,
       SUM(CASE WHEN stars IS NULL THEN 1 ELSE 0 END)   AS khong_ro
FROM toinoi.trip_ratings;

-- Tập rỗng: trừ COUNT, hàm tổng hợp trả NULL khi không có dòng nào.
DECLARE @n int, @s int;
SELECT @n = COUNT(*), @s = SUM(fare) FROM toinoi.trips
WHERE city_id = 3 AND requested_at >= '2026-10-20 00:00 +07:00' AND requested_at < '2026-10-21 00:00 +07:00';
EXEC lab.expect_value N'4 DN 20/10 COUNT(*)', @n, N'0';
EXEC lab.expect_value N'4 DN 20/10 SUM(fare)', @s, NULL;
GO

-- ---------------------------------------------------------------------------
-- 5. Tip: NULL (bỏ qua bước tip) KHÁC 0 (chủ động chọn không tip). Câu hỏi
--    nghiệp vụ quyết định mẫu số. AVG(int) lại cắt phần lẻ: 2.942,89 → 2.942.
SELECT COUNT(*)                                                    AS so_danh_gia,
       COUNT(tip)                                                  AS co_tra_loi_tip,
       SUM(CASE WHEN tip = 0 THEN 1 ELSE 0 END)                    AS chon_0d,
       SUM(tip)                                                    AS tong_tip,
       CAST(ROUND(AVG(CAST(tip AS decimal(12, 2))), 0) AS int)            AS tb_tren_nguoi_tra_loi,
       CAST(ROUND(AVG(CAST(ISNULL(tip, 0) AS decimal(12, 2))), 0) AS int) AS tb_tren_moi_luot,
       AVG(ISNULL(tip, 0))                                         AS tb_moi_luot_avg_int
FROM toinoi.trip_ratings;

DECLARE @tb1 int, @tb2 int, @tb2_int int;
SELECT @tb1 = ROUND(AVG(CAST(tip AS decimal(12, 2))), 0),
       @tb2 = ROUND(AVG(CAST(ISNULL(tip, 0) AS decimal(12, 2))), 0),
       @tb2_int = AVG(ISNULL(tip, 0))
FROM toinoi.trip_ratings;
EXEC lab.expect_value N'5 tip TB trên người trả lời', @tb1, N'4122';
EXEC lab.expect_value N'5 tip TB trên mỗi lượt (làm tròn)', @tb2, N'2943';
EXEC lab.expect_value N'5 tip TB trên mỗi lượt (AVG int)', @tb2_int, N'2942';

-- NULL "lây" qua phép cộng: SUM(fare + tip) bỏ luôn cước của lượt bỏ qua tip.
SELECT SUM(t.fare) + SUM(r.tip)          AS cong_hai_tong,
       SUM(t.fare + r.tip)               AS tong_cua_tong_dong,
       SUM(t.fare + ISNULL(r.tip, 0))    AS tong_dong_isnull
FROM toinoi.trips AS t
JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id;

DECLARE @a int, @b int;
SELECT @a = SUM(t.fare) + SUM(r.tip), @b = SUM(t.fare + r.tip)
FROM toinoi.trips AS t JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id;
EXEC lab.expect_value N'5 cộng hai tổng', @a, N'134142000';
EXEC lab.expect_value N'5 SUM(fare + tip)', @b, N'98163000';
GO

-- ---------------------------------------------------------------------------
-- 6. Cước 0 đ là giá trị THẬT (khuyến mãi). "Cước TB của chuyến có thu tiền"
--    loại 0 bằng NULLIF(fare, 0) — NULLIF(a, b) trả NULL khi a = b.
SELECT COUNT(*)                                                 AS chuyen_hoan_thanh,
       SUM(CASE WHEN fare = 0 THEN 1 ELSE 0 END)                AS cuoc_0d,
       AVG(fare)                                                        AS avg_int,
       CAST(ROUND(AVG(CAST(fare AS decimal(12, 2))), 0) AS int)            AS tb_moi_chuyen,
       CAST(ROUND(AVG(CAST(NULLIF(fare, 0) AS decimal(12, 2))), 0) AS int) AS tb_chuyen_co_thu
FROM toinoi.trips
WHERE status = 'completed';

DECLARE @tb_all int, @tb_co_thu int;
SELECT @tb_all = ROUND(AVG(CAST(fare AS decimal(12, 2))), 0),
       @tb_co_thu = ROUND(AVG(CAST(NULLIF(fare, 0) AS decimal(12, 2))), 0)
FROM toinoi.trips WHERE status = 'completed';
EXEC lab.expect_value N'6 cước TB mọi chuyến hoàn thành', @tb_all, N'67339';
EXEC lab.expect_value N'6 cước TB chuyến có thu', @tb_co_thu, N'68119';
GO

-- ---------------------------------------------------------------------------
-- 7. ISNULL và COALESCE KHÔNG giống nhau.
--  7a. Kiểu trả về: ISNULL lấy kiểu của đối số ĐẦU; COALESCE lấy kiểu có độ
--      ưu tiên cao nhất. areas.code là varchar(12) ⇒ ISNULL cắt chuỗi còn 12
--      ký tự VÀ ép về varchar (mất chữ tiếng Việt ngoài bảng mã 1258).
SELECT t.code,
       ISNULL(z.code, N'không rõ (mất GPS)')                                  AS khu_isnull,
       COALESCE(z.code, N'không rõ (mất GPS)')                                AS khu_coalesce,
       CAST(SQL_VARIANT_PROPERTY(ISNULL(z.code, N'không rõ (mất GPS)'), 'BaseType') AS sysname)   AS kieu_isnull,
       CAST(SQL_VARIANT_PROPERTY(COALESCE(z.code, N'không rõ (mất GPS)'), 'BaseType') AS sysname) AS kieu_coalesce
FROM toinoi.trips AS t
LEFT JOIN toinoi.areas AS z ON z.id = t.pickup_area_id
WHERE t.code = (SELECT MIN(code) FROM toinoi.trips WHERE pickup_area_id IS NULL);

DECLARE @isn nvarchar(50), @coa nvarchar(50);
SELECT @isn = ISNULL(z.code, N'không rõ (mất GPS)'), @coa = COALESCE(z.code, N'không rõ (mất GPS)')
FROM toinoi.trips AS t LEFT JOIN toinoi.areas AS z ON z.id = t.pickup_area_id
WHERE t.code = 100072;
DECLARE @len int = LEN(@isn);
EXEC lab.expect_value N'7a ISNULL cắt còn 12 ký tự', @len, N'12';
EXEC lab.expect_value N'7a COALESCE giữ nguyên chuỗi', @coa, N'không rõ (mất GPS)';

--  7b. Chữ vào cột số: cả hai đều lỗi 245 (ISNULL ép chữ về int của đối số
--      đầu; COALESCE chọn int vì int ưu tiên cao hơn nvarchar). Phải đổi số
--      sang chữ trước.
EXEC lab.expect_error N'7b ISNULL(d.code, chữ)',
     N'SELECT ISNULL(d.code, N''đang tìm…'') FROM toinoi.trips AS t LEFT JOIN toinoi.drivers AS d ON d.id = t.driver_id',
     245, N'đang tìm';
EXEC lab.expect_error N'7b COALESCE(d.code, chữ)',
     N'SELECT COALESCE(d.code, N''đang tìm…'') FROM toinoi.trips AS t LEFT JOIN toinoi.drivers AS d ON d.id = t.driver_id',
     245, N'đang tìm';

--  7c. Tính NULL của cột kết quả: ISNULL với đối số thứ hai khác NULL ⇒ cột
--      NOT NULL; COALESCE luôn được coi là có thể NULL. Thấy rõ khi SELECT INTO
--      (và với computed column, view có index).
SELECT TOP (0) ISNULL(tip, 0) AS tip_isnull, COALESCE(tip, 0) AS tip_coalesce
INTO #tip_nullability
FROM toinoi.trip_ratings;

SELECT c.name AS cot, c.is_nullable
FROM tempdb.sys.columns AS c
WHERE c.object_id = OBJECT_ID(N'tempdb..#tip_nullability')
ORDER BY c.column_id;

DECLARE @n1 int = (SELECT CAST(is_nullable AS int) FROM tempdb.sys.columns
                   WHERE object_id = OBJECT_ID(N'tempdb..#tip_nullability') AND name = N'tip_isnull'),
        @n2 int = (SELECT CAST(is_nullable AS int) FROM tempdb.sys.columns
                   WHERE object_id = OBJECT_ID(N'tempdb..#tip_nullability') AND name = N'tip_coalesce');
EXEC lab.expect_value N'7c ISNULL(tip, 0) is_nullable', @n1, N'0';
EXEC lab.expect_value N'7c COALESCE(tip, 0) is_nullable', @n2, N'1';
DROP TABLE #tip_nullability;

--  7d. COALESCE nhận nhiều đối số; ISNULL chỉ nhận đúng hai (lỗi 174).
EXEC lab.expect_error N'7d ISNULL ba đối số', N'SELECT ISNULL(NULL, NULL, 1)', 174, N'isnull';
GO

-- ---------------------------------------------------------------------------
-- 8. COALESCE để HIỂN THỊ: mỗi chỗ trống được gắn đúng nghĩa của nó. Số tài
--    xế đổi sang chữ bằng CONVERT trước khi ghép với chữ (mục 7b).
SELECT t.code, t.kind, t.status,
       COALESCE(m.name,
                CASE WHEN t.kind = 'food' THEN N'LỖI: đơn đồ ăn thiếu quán'
                     ELSE N'— (không áp dụng)' END)                    AS quan,
       COALESCE(z.code, N'không rõ (mất GPS)')                         AS khu_don,
       COALESCE(CONVERT(nvarchar(10), d.code),
                CASE WHEN t.status IN ('requested', 'searching') THEN N'đang tìm…'
                     ELSE N'— (huỷ trước khi nhận)' END)               AS tai_xe
FROM toinoi.trips AS t
LEFT JOIN toinoi.merchants AS m ON m.id = t.merchant_id
LEFT JOIN toinoi.areas     AS z ON z.id = t.pickup_area_id
LEFT JOIN toinoi.drivers   AS d ON d.id = t.driver_id
WHERE t.code IN (SELECT MIN(code) FROM toinoi.trips WHERE pickup_area_id IS NULL
                 UNION ALL SELECT MIN(code) FROM toinoi.trips WHERE kind = 'food' AND status = 'completed'
                 UNION ALL SELECT MIN(code) FROM toinoi.trips WHERE status = 'cancelled' AND driver_id IS NULL
                 UNION ALL SELECT MAX(code) FROM toinoi.trips WHERE status = 'searching'
                 UNION ALL SELECT MIN(code) FROM toinoi.trips WHERE fare = 0)
ORDER BY t.code;

-- IIF(điều_kiện, a, b) là cách viết gọn của CASE hai nhánh. Điều kiện UNKNOWN
-- rơi vào nhánh b — như ELSE của CASE.
DECLARE @iif int = (SELECT SUM(IIF(stars >= 4, 0, 1)) FROM toinoi.trip_ratings);
EXEC lab.expect_value N'8 IIF(stars >= 4, 0, 1): dưới 4 sao + NULL', @iif, N'456';
GO

-- ---------------------------------------------------------------------------
-- 9. `<>` cũng bỏ rơi NULL. "Chuyến KHÔNG đón ở khu SG-TT-01 (id 111)":
--    `<> 111` không đếm 36 chuyến mất GPS; IS DISTINCT FROM 111 (SQL Server
--    2022) đếm cả chúng.
SELECT COUNT(*)                                                        AS tong,
       SUM(CASE WHEN pickup_area_id = 111 THEN 1 ELSE 0 END)           AS o_111,
       SUM(CASE WHEN pickup_area_id <> 111 THEN 1 ELSE 0 END)          AS khac_111,
       SUM(CASE WHEN pickup_area_id IS DISTINCT FROM 111 THEN 1 ELSE 0 END) AS khac_biet_111
FROM toinoi.trips;

DECLARE @khac int = (SELECT COUNT(*) FROM toinoi.trips WHERE pickup_area_id <> 111),
        @dist int = (SELECT COUNT(*) FROM toinoi.trips WHERE pickup_area_id IS DISTINCT FROM 111),
        @cu   int = (SELECT COUNT(*) FROM toinoi.trips WHERE ISNULL(pickup_area_id, -1) <> 111);
EXEC lab.expect_value N'9 pickup_area_id <> 111', @khac, N'2976';
EXEC lab.expect_value N'9 IS DISTINCT FROM 111', @dist, N'3012';
EXEC lab.expect_value N'9 cách cũ ISNULL(…, -1) <> 111', @cu, N'3012';

GO

-- ---------------------------------------------------------------------------
-- 10. Chia cho 0 ⇒ lỗi 8134, cả câu hỏng. Ngày 20/10 Đà Nẵng có bão, 0 chuyến.
EXEC lab.expect_error N'10 tỉ lệ huỷ 20/10 chia cho 0',
     N'SELECT c.code, COUNT(t.id) AS tong,
              100 * SUM(CASE WHEN t.status = ''cancelled'' THEN 1 ELSE 0 END) / COUNT(t.id) AS ty_le_huy_pct
       FROM toinoi.cities AS c
       LEFT JOIN toinoi.trips AS t
         ON t.city_id = c.id
        AND t.requested_at >= ''2026-10-20 00:00 +07:00'' AND t.requested_at < ''2026-10-21 00:00 +07:00''
       GROUP BY c.code
       ORDER BY c.code',
     8134, N'Divide by zero';

-- Mã cũ đặt SET ANSI_WARNINGS OFF + ARITHABORT OFF thì chia cho 0 ra NULL
-- kèm cảnh báo thay vì lỗi — lỗi bị giấu. Đừng dùng; viết NULLIF có chủ đích.
SET ANSI_WARNINGS OFF; SET ARITHABORT OFF;
DECLARE @chia0 int = 100 / 0;
SET ANSI_WARNINGS ON; SET ARITHABORT ON;
EXEC lab.expect_value N'10 ANSI_WARNINGS OFF: 100 / 0', @chia0, NULL;
GO

-- NULLIF(mẫu, 0) ⇒ tỉ lệ của DN là NULL = "không xác định". 100.0 (decimal)
-- thay vì 100 (int) để tránh chia nguyên: 100 * 5 / 37 = 13, không phải 13,5.
SELECT c.code,
       COUNT(t.id)                                                            AS tong,
       100 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / NULLIF(COUNT(t.id), 0) AS pct_chia_nguyen,
       CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END)
            / NULLIF(COUNT(t.id), 0) AS decimal(4, 1))                       AS ty_le_huy_pct,
       COALESCE(CONVERT(nvarchar(10), CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END)
                                           / NULLIF(COUNT(t.id), 0) AS decimal(4, 1))),
                N'không có chuyến')                                           AS hien_thi
FROM toinoi.cities AS c
LEFT JOIN toinoi.trips AS t
  ON t.city_id = c.id
 AND t.requested_at >= '2026-10-20 00:00 +07:00' AND t.requested_at < '2026-10-21 00:00 +07:00'
GROUP BY c.code
ORDER BY c.code;

DECLARE @hn decimal(4, 1), @sg decimal(4, 1), @dn_null int;
SELECT @hn = MAX(CASE WHEN code = 'HN' THEN pct END),
       @sg = MAX(CASE WHEN code = 'SG' THEN pct END),
       @dn_null = SUM(CASE WHEN code = 'DN' AND pct IS NULL THEN 1 ELSE 0 END)
FROM (SELECT c.code,
             CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END)
                  / NULLIF(COUNT(t.id), 0) AS decimal(4, 1)) AS pct
      FROM toinoi.cities AS c
      LEFT JOIN toinoi.trips AS t
        ON t.city_id = c.id
       AND t.requested_at >= '2026-10-20 00:00 +07:00' AND t.requested_at < '2026-10-21 00:00 +07:00'
      GROUP BY c.code) AS x;
EXEC lab.expect_value N'10 tỉ lệ huỷ HN 20/10', @hn, N'13.5';
EXEC lab.expect_value N'10 tỉ lệ huỷ SG 20/10', @sg, N'14.3';
EXEC lab.expect_value N'10 tỉ lệ huỷ DN 20/10 là NULL', @dn_null, N'1';
GO

-- ===========================================================================
-- Bài tập (lời giải trong bài viết)
-- ===========================================================================

-- BT1. Dịch merchant_id thành ba nghĩa rồi đếm.
SELECT y_nghia_merchant_id, COUNT(*) AS so_chuyen
FROM (SELECT CASE
               WHEN merchant_id IS NOT NULL THEN N'có quán'
               WHEN kind <> 'food'          THEN N'không áp dụng — không phải đơn đồ ăn'
               ELSE N'LỖI — đơn đồ ăn thiếu quán'
             END AS y_nghia_merchant_id
      FROM toinoi.trips) AS x
GROUP BY y_nghia_merchant_id
ORDER BY so_chuyen DESC;

DECLARE @loi int = (SELECT COUNT(*) FROM toinoi.trips WHERE merchant_id IS NULL AND kind = 'food');
EXEC lab.expect_value N'BT1 đơn đồ ăn thiếu quán', @loi, N'0';
GO

-- BT2. Cước + tip của chuyến hoàn thành tháng 10 theo thành phố.
SELECT c.code AS tp,
       SUM(t.fare + r.tip)                  AS sai_cong_trong_sum,
       SUM(t.fare) + ISNULL(SUM(r.tip), 0)  AS cuoc_cong_tip
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
  AND t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY c.code
ORDER BY c.code;

DECLARE @sg_sai int, @sg_dung int;
SELECT @sg_sai = SUM(t.fare + r.tip), @sg_dung = SUM(t.fare) + ISNULL(SUM(r.tip), 0)
FROM toinoi.trips AS t
LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed' AND t.city_id = 1
  AND t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00';
EXEC lab.expect_value N'BT2 SG SUM(fare + tip)', @sg_sai, N'49340000';
EXEC lab.expect_value N'BT2 SG cước + tip', @sg_dung, N'95650000';
GO

-- BT3. Bảng ngày 20/10: mỗi thành phố một dòng, không ghi số 0 giả.
SELECT c.code AS tp, COUNT(t.id) AS hoan_thanh, COUNT(r.stars) AS co_sao,
       CAST(AVG(CAST(r.stars AS decimal(9, 4))) AS decimal(4, 2)) AS diem_tb,
       COALESCE(CONVERT(nvarchar(10), CAST(AVG(CAST(r.stars AS decimal(9, 4))) AS decimal(4, 2))),
                CASE WHEN COUNT(t.id) = 0 THEN N'không có chuyến'
                     ELSE N'chưa có lượt chấm sao' END) AS hien_thi
FROM toinoi.cities AS c
LEFT JOIN toinoi.trips AS t
  ON t.city_id = c.id AND t.status = 'completed'
 AND t.requested_at >= '2026-10-20 00:00 +07:00' AND t.requested_at < '2026-10-21 00:00 +07:00'
LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
GROUP BY c.code
ORDER BY c.code;

DECLARE @hn_tb decimal(4, 2), @sg_tb decimal(4, 2);
SELECT @hn_tb = CAST(AVG(CASE WHEN t.city_id = 2 THEN CAST(r.stars AS decimal(9, 4)) END) AS decimal(4, 2)),
       @sg_tb = CAST(AVG(CASE WHEN t.city_id = 1 THEN CAST(r.stars AS decimal(9, 4)) END) AS decimal(4, 2))
FROM toinoi.trips AS t
JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
  AND t.requested_at >= '2026-10-20 00:00 +07:00' AND t.requested_at < '2026-10-21 00:00 +07:00';
EXEC lab.expect_value N'BT3 HN điểm TB 20/10', @hn_tb, N'4.23';
EXEC lab.expect_value N'BT3 SG điểm TB 20/10', @sg_tb, N'4.39';
GO
