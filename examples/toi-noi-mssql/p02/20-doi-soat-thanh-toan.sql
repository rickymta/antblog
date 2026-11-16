-- P02 · 20-doi-soat-thanh-toan.sql — MS-020 `mssql-thuc-hanh-tsql-doi-soat`
-- Thực hành: truy vấn vận hành và ĐỐI SOÁT THANH TOÁN — bàn giao T-SQL, expected
-- output và giải thích chênh lệch.
--
-- Chạy sau 00 → 02 (dữ liệu không đổi; file chạy lại được nhiều lần):
--   node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
--                p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/20-doi-soat-thanh-toan.sql
--
-- ĐẶC TẢ ĐỐI SOÁT THÁNG 10/2026
--   Phạm vi     chuyến ĐẶT trong [2026-10-01 00:00, 2026-10-31 00:00) giờ VN
--               (+07:00) và đã kết thúc (completed / cancelled). Chuyến đang
--               chạy chưa đối soát.
--   Phải thu    completed ⇒ cước (fare); cancelled ⇒ 0 đ.
--   Thực thu    Σ charge − Σ refund + Σ adjustment, CHỈ giao dịch `succeeded`
--               (pending/failed không làm đổi tiền), KHÔNG lọc theo ngày giao
--               dịch: tiền của chuyến tính cho chuyến, dù thu sau nửa đêm.
--   Khớp        thực thu = phải thu.
--   Phân loại   theo thứ tự ưu tiên (dòng lệch nhận loại ĐẦU TIÊN đúng):
--     1 HOÀN KHÔNG CÓ THU   có refund thành công nhưng không có charge thành công
--     2 THU CHUYẾN HUỶ      chuyến huỷ, thực thu > 0
--     3 THIẾU THANH TOÁN    hoàn thành, cước > 0, không có charge thành công
--     4 THU TRÙNG           ≥ 2 charge thành công CÙNG số tiền (lặp yêu cầu)
--     5 LỆCH SỐ TIỀN        còn lại: có thu nhưng thực thu ≠ phải thu
--   Hai charge thành công KHÁC số tiền là trả một phần (ví + thẻ), KHÔNG phải
--   thu trùng.
--
-- Kết quả mong đợi (cài sẵn trong 01-fixture.sql): ĐÚNG 7 chuyến lệch, tổng
-- chênh +268.000 đ; KHÔNG báo nhầm các ca khó nhưng đúng.
--
-- Khác bản PostgreSQL: SQL Server không có view tạm ⇒ dùng VIEW thường trong
-- schema lab, tạo bằng CREATE OR ALTER (chạy lại không lỗi) và để lại sau file
-- để mở trong DBStudio. CREATE VIEW phải là câu đầu tiên của batch (lỗi 111).

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

-- ---------------------------------------------------------------------------
-- 0. SỔ ĐỐI SOÁT theo chuyến: MỘT định nghĩa dùng chung, một dòng một chuyến.
--    gd     = giao dịch GOM THEO CHUYẾN TRƯỚC rồi mới nối (không nhân dòng);
--    lap    = CROSS APPLY một truy vấn tổng hợp KHÔNG có GROUP BY ⇒ luôn trả
--             đúng một dòng, nên không làm mất chuyến nào.
CREATE OR ALTER VIEW lab.so_doi_soat
AS
WITH pham_vi AS (
    SELECT t.id, t.code, t.kind, t.city_id, t.status,
           CAST(SWITCHOFFSET(t.requested_at, '+07:00') AS date)       AS ngay,
           CAST(IIF(t.status = 'completed', t.fare, 0) AS bigint)     AS phai_thu
    FROM toinoi.trips AS t
    WHERE t.status IN ('completed', 'cancelled')
      AND t.requested_at >= '2026-10-01 00:00:00 +07:00'
      AND t.requested_at <  '2026-10-31 00:00:00 +07:00'
),
gd AS (
    SELECT p.trip_id,
           SUM(IIF(p.kind = 'charge' AND p.status = 'succeeded', 1, 0))                          AS so_thu_ok,
           SUM(IIF(p.kind = 'charge'     AND p.status = 'succeeded', CAST(p.amount AS bigint), 0)) AS tong_thu,
           SUM(IIF(p.kind = 'refund'     AND p.status = 'succeeded', CAST(p.amount AS bigint), 0)) AS tong_hoan,
           SUM(IIF(p.kind = 'adjustment' AND p.status = 'succeeded', CAST(p.amount AS bigint), 0)) AS tong_dieu_chinh,
           SUM(IIF(p.status <> 'succeeded', 1, 0))                                               AS so_gd_khong_thanh_cong,
           STRING_AGG(CONCAT(p.kind, '/', p.method, '/', p.status, '/', p.amount), ', ')
               WITHIN GROUP (ORDER BY p.paid_at, p.id)                                           AS chi_tiet
    FROM toinoi.payments AS p
    GROUP BY p.trip_id
)
SELECT pv.id, pv.code, pv.kind, pv.city_id, pv.status, pv.ngay, pv.phai_thu,
       ISNULL(g.so_thu_ok, 0)                                       AS so_thu_ok,
       lap.so_thu_lap_tien,
       ISNULL(g.tong_thu, 0)                                        AS tong_thu,
       ISNULL(g.tong_hoan, 0)                                       AS tong_hoan,
       ISNULL(g.tong_dieu_chinh, 0)                                 AS tong_dieu_chinh,
       ISNULL(g.tong_thu - g.tong_hoan + g.tong_dieu_chinh, 0)      AS thuc_thu,
       ISNULL(g.tong_thu - g.tong_hoan + g.tong_dieu_chinh, 0) - pv.phai_thu AS chenh,
       ISNULL(g.so_gd_khong_thanh_cong, 0)                          AS so_gd_khong_thanh_cong,
       -- COALESCE, không ISNULL: ISNULL lấy kiểu của tham số ĐẦU (varchar của
       -- STRING_AGG) nên chữ "ị" bị đổi thành "?" (xem kiểm tra V2 bên dưới).
       COALESCE(g.chi_tiet, N'(không có giao dịch)')                AS chi_tiet,
       CASE
         WHEN ISNULL(g.tong_thu - g.tong_hoan + g.tong_dieu_chinh, 0) = pv.phai_thu THEN NULL
         WHEN ISNULL(g.so_thu_ok, 0) = 0 AND g.tong_hoan > 0           THEN N'1 HOÀN KHÔNG CÓ THU'
         WHEN pv.status = 'cancelled'                                  THEN N'2 THU CHUYẾN HUỶ'
         WHEN pv.phai_thu > 0 AND ISNULL(g.so_thu_ok, 0) = 0           THEN N'3 THIẾU THANH TOÁN'
         WHEN lap.so_thu_lap_tien > 0                                  THEN N'4 THU TRÙNG'
         ELSE                                                               N'5 LỆCH SỐ TIỀN'
       END AS loai_lech
FROM pham_vi AS pv
LEFT JOIN gd AS g ON g.trip_id = pv.id
CROSS APPLY (SELECT COUNT(*) - COUNT(DISTINCT p.amount) AS so_thu_lap_tien
             FROM toinoi.payments AS p
             WHERE p.trip_id = pv.id AND p.kind = 'charge' AND p.status = 'succeeded') AS lap;
GO

-- CREATE VIEW không đứng đầu batch ⇒ lỗi 111 (vì thế phải có GO phía trên).
EXEC lab.expect_error N'V1 CREATE VIEW sau một câu khác trong cùng batch',
     N'SELECT 1 AS x; CREATE VIEW lab.v_thu AS SELECT 1 AS x;', 111, N'CREATE VIEW';

-- ISNULL trả về kiểu của tham số đầu: varchar ⇒ mất dấu tiếng Việt.
-- COALESCE trả về kiểu ưu tiên cao nhất (nvarchar) ⇒ giữ nguyên.
DECLARE @chi_tiet varchar(8000) = NULL;
DECLARE @isnull nvarchar(100) = ISNULL(@chi_tiet, N'(không có giao dịch)');
DECLARE @coalesce nvarchar(100) = COALESCE(@chi_tiet, N'(không có giao dịch)');
SELECT @isnull AS voi_isnull, @coalesce AS voi_coalesce;
EXEC lab.expect_value N'V2 ISNULL(varchar, N…)', @isnull, N'(không có giao d?ch)';
EXEC lab.expect_value N'V2 COALESCE(varchar, N…)', @coalesce, N'(không có giao dịch)';
GO

-- ---------------------------------------------------------------------------
-- 1. ĐỐI SOÁT TỔNG: số chuyến, tổng phải thu, tổng thực thu, chênh.
DECLARE @so int, @phai bigint, @thuc bigint, @chenh bigint;
SELECT @so = COUNT(*), @phai = SUM(phai_thu), @thuc = SUM(thuc_thu), @chenh = SUM(chenh)
FROM lab.so_doi_soat;
SELECT @so AS so_chuyen, @phai AS tong_phai_thu, @thuc AS tong_thuc_thu, @chenh AS chenh;
EXEC lab.expect_value N'1 số chuyến trong phạm vi', @so, N'3168';
EXEC lab.expect_value N'1 tổng phải thu (đ)', @phai, N'182287000';
EXEC lab.expect_value N'1 tổng thực thu (đ)', @thuc, N'182555000';
EXEC lab.expect_value N'1 chênh (đ)', @chenh, N'268000';

-- Sổ không được nở hay mất dòng: mỗi chuyến trong phạm vi đúng một dòng.
DECLARE @dong int, @khac int, @doc_lap int;
SELECT @dong = COUNT(*), @khac = COUNT(DISTINCT id) FROM lab.so_doi_soat;
SELECT @doc_lap = COUNT(*) FROM toinoi.trips
WHERE status IN ('completed', 'cancelled')
  AND requested_at >= '2026-10-01 00:00:00 +07:00' AND requested_at < '2026-10-31 00:00:00 +07:00';
SELECT @dong AS dong_so, @khac AS chuyen_khac_nhau, @doc_lap AS dem_doc_lap;
DECLARE @ba nvarchar(60) = CONCAT(@dong, '|', @khac, '|', @doc_lap);
EXEC lab.expect_value N'1 dòng sổ | chuyến khác nhau | đếm độc lập', @ba, N'3168|3168|3168';

-- Mọi giao dịch thuộc một chuyến TRONG sổ (không có tiền "trôi nổi").
DECLARE @troi int = (SELECT COUNT(*) FROM toinoi.payments AS p
                     WHERE NOT EXISTS (SELECT 1 FROM lab.so_doi_soat AS s WHERE s.id = p.trip_id));
EXEC lab.expect_value N'1 giao dịch ngoài sổ', @troi, N'0';

-- Cầu nối: cộng thẳng trên payments (chỉ succeeded) phải ra đúng tổng thực thu.
SELECT kind, COUNT(*) AS so_gd, SUM(CAST(amount AS bigint)) AS tong_tien
FROM toinoi.payments
WHERE status = 'succeeded'
GROUP BY kind
ORDER BY kind;
DECLARE @cau_noi bigint = (SELECT SUM(IIF(kind = 'refund', -CAST(amount AS bigint), CAST(amount AS bigint)))
                           FROM toinoi.payments WHERE status = 'succeeded');
EXEC lab.expect_value N'1 cầu nối charge − refund + adjustment (đ)', @cau_noi, N'182555000';
DECLARE @khong_ok nvarchar(100) = (SELECT STRING_AGG(CONCAT(status, ':', n), ', ') WITHIN GROUP (ORDER BY status)
                                   FROM (SELECT status, COUNT(*) AS n FROM toinoi.payments
                                         WHERE status <> 'succeeded' GROUP BY status) AS x);
EXEC lab.expect_value N'1 giao dịch không đổi tiền', @khong_ok, N'failed:96, pending:1';

-- SUM trên cột int trả về int: tổng vượt 2.147.483.647 thì lỗi 8115. Giả lập
-- "báo cáo 12 tháng" bằng cách nhân 12 lần tiền thu tháng 10 (183.640.000 đ).
EXEC lab.expect_error N'1 SUM(int) tràn khi cộng 12 tháng',
     N'DECLARE @x int; SELECT @x = SUM(p.amount) FROM toinoi.payments AS p CROSS JOIN (SELECT TOP (12) object_id FROM sys.all_objects) AS m WHERE p.kind = ''charge'' AND p.status = ''succeeded'';',
     8115, N'int';
DECLARE @ca_nam bigint = (SELECT SUM(CAST(p.amount AS bigint)) FROM toinoi.payments AS p
                          CROSS JOIN (SELECT TOP (12) object_id FROM sys.all_objects) AS m
                          WHERE p.kind = 'charge' AND p.status = 'succeeded');
EXEC lab.expect_value N'1 cùng tổng với CAST(… AS bigint)', @ca_nam, N'2203680000';
GO

-- ---------------------------------------------------------------------------
-- 2. ĐÁP ÁN: danh sách chênh lệch (expected output bàn giao).
SELECT s.code, s.ngay, c.code AS tp, s.kind, s.status,
       s.phai_thu, s.thuc_thu, s.chenh, s.loai_lech, s.chi_tiet
FROM lab.so_doi_soat AS s
JOIN toinoi.cities AS c ON c.id = s.city_id
WHERE s.loai_lech IS NOT NULL
ORDER BY s.code;

DECLARE @dap_an nvarchar(400) =
    (SELECT STRING_AGG(CONCAT(code, ':', LEFT(loai_lech, 1), ':', chenh), ', ') WITHIN GROUP (ORDER BY code)
     FROM lab.so_doi_soat WHERE loai_lech IS NOT NULL);
EXEC lab.expect_value N'2 đáp án mã:loại:chênh', @dap_an,
     N'100724:3:-76000, 101084:3:-98000, 101310:4:85000, 101987:1:-72000, 102443:5:-9000, 102642:5:342000, 102739:2:96000';

-- Theo loại, có dòng tổng (GROUPING SETS); dòng tổng = chênh của mục 1.
SELECT IIF(GROUPING(loai_lech) = 1, N'TỔNG', loai_lech) AS loai,
       COUNT(*) AS so_chuyen, SUM(chenh) AS tong_chenh
FROM lab.so_doi_soat
WHERE loai_lech IS NOT NULL
GROUP BY GROUPING SETS ((loai_lech), ())
ORDER BY GROUPING(loai_lech), loai_lech;

DECLARE @theo_loai nvarchar(400) =
    (SELECT STRING_AGG(CONCAT(IIF(g = 1, N'T', LEFT(loai_lech, 1)), ':', n, ':', tong), ', ')
                WITHIN GROUP (ORDER BY g, loai_lech)
     FROM (SELECT loai_lech, GROUPING(loai_lech) AS g, COUNT(*) AS n, SUM(chenh) AS tong
           FROM lab.so_doi_soat WHERE loai_lech IS NOT NULL
           GROUP BY GROUPING SETS ((loai_lech), ())) AS x);
EXEC lab.expect_value N'2 theo loại loại:số chuyến:chênh', @theo_loai,
     N'1:1:-72000, 2:1:96000, 3:2:-174000, 4:1:85000, 5:2:333000, T:7:268000';
GO

-- Bàn giao cho hệ khác dưới dạng JSON (FOR JSON PATH). Gán vào biến để kết quả
-- không bị sqlcmd/driver cắt thành nhiều dòng, rồi đọc ngược bằng OPENJSON để
-- chứng minh JSON giữ đủ 7 dòng và đúng tổng chênh.
DECLARE @json nvarchar(max) =
    (SELECT s.code AS ma_chuyen, s.ngay, c.code AS tp, s.kind AS dich_vu, s.status AS trang_thai,
            s.phai_thu, s.thuc_thu, s.chenh, s.loai_lech
     FROM lab.so_doi_soat AS s
     JOIN toinoi.cities AS c ON c.id = s.city_id
     WHERE s.loai_lech IS NOT NULL
     ORDER BY s.code
     FOR JSON PATH);
SELECT ISJSON(@json) AS hop_le, LEN(@json) AS do_dai, JSON_QUERY(@json, '$[0]') AS phan_tu_dau;
DECLARE @j_dong int, @j_chenh bigint;
SELECT @j_dong = COUNT(*), @j_chenh = SUM(chenh)
FROM OPENJSON(@json) WITH (ma_chuyen bigint, chenh bigint);
DECLARE @j nvarchar(60) = CONCAT(@j_dong, '|', @j_chenh);
EXEC lab.expect_value N'2 JSON đọc lại: số dòng | tổng chênh', @j, N'7|268000';
GO

-- ---------------------------------------------------------------------------
-- 3. ĐỐI SOÁT SỐ DÒNG: mọi chuyến trong phạm vi rơi vào ĐÚNG MỘT nhóm.
WITH x AS (
    SELECT tong_thu,
           CASE
             WHEN loai_lech IS NOT NULL                 THEN N'LỆCH'
             WHEN status = 'completed' AND phai_thu = 0 THEN N'KHỚP — hoàn thành cước 0 đ, không cần thu'
             WHEN status = 'completed'                  THEN N'KHỚP — hoàn thành, có thu'
             WHEN tong_hoan > 0                         THEN N'KHỚP — chuyến huỷ, đã hoàn đủ'
             ELSE                                            N'KHỚP — chuyến huỷ, không giao dịch'
           END AS nhom
    FROM lab.so_doi_soat
)
SELECT IIF(GROUPING(nhom) = 1, N'TỔNG', nhom) AS nhom, COUNT(*) AS so_chuyen, SUM(tong_thu) AS tong_tien_charge
FROM x
GROUP BY GROUPING SETS ((nhom), ())
ORDER BY GROUPING(nhom), COUNT(*) DESC;

DECLARE @nhom nvarchar(200) =
    (SELECT STRING_AGG(CONCAT(n, ':', tien), ', ') WITHIN GROUP (ORDER BY n DESC)
     FROM (SELECT COUNT(*) AS n, SUM(tong_thu) AS tien
           FROM lab.so_doi_soat
           GROUP BY CASE WHEN loai_lech IS NOT NULL THEN 0
                         WHEN status = 'completed' AND phai_thu = 0 THEN 1
                         WHEN status = 'completed' THEN 2
                         WHEN tong_hoan > 0 THEN 3 ELSE 4 END) AS x);
EXEC lab.expect_value N'3 nhóm số chuyến:tiền charge', @nhom,
     N'2671:181941000, 447:0, 31:0, 12:1008000, 7:691000';
GO

-- ---------------------------------------------------------------------------
-- 4. CA KHÓ NHƯNG ĐÚNG — không ca nào được báo lệch.
SELECT code, status, phai_thu, thuc_thu, loai_lech, chi_tiet
FROM lab.so_doi_soat
WHERE code IN (100042, 100048, 102235, 103162, 103165)
ORDER BY code;
DECLARE @kho int = (SELECT COUNT(*) FROM lab.so_doi_soat
                    WHERE code IN (100042, 100048, 102235, 103162, 103165) AND loai_lech IS NULL AND chenh = 0);
EXEC lab.expect_value N'4 năm ca khó không bị báo', @kho, N'5';

-- Chuyến huỷ đã trừ thẻ rồi hoàn đủ: 12 chuyến, không chuyến nào bị báo.
SELECT TOP (1) code, status, phai_thu, thuc_thu, loai_lech, chi_tiet
FROM lab.so_doi_soat
WHERE status = 'cancelled' AND tong_hoan > 0 AND loai_lech IS NULL
ORDER BY code;
DECLARE @hoan_du int = (SELECT COUNT(*) FROM lab.so_doi_soat
                        WHERE status = 'cancelled' AND so_thu_ok > 0 AND tong_hoan = tong_thu AND loai_lech IS NULL);
EXEC lab.expect_value N'4 chuyến huỷ đã hoàn đủ, không bị báo', @hoan_du, N'12';

-- Trả một phần (hai charge thành công KHÁC số tiền) — không phải thu trùng.
DECLARE @mot_phan int = (SELECT COUNT(*) FROM lab.so_doi_soat
                         WHERE so_thu_ok = 2 AND so_thu_lap_tien = 0 AND loai_lech IS NULL);
EXEC lab.expect_value N'4 chuyến trả một phần, không bị báo', @mot_phan, N'26';
GO

-- QUY TẮC NGÂY THƠ — hai cách "đơn giản" hay gặp:
--   A: cộng MỌI charge bất kể trạng thái, bỏ qua refund/adjustment, so với phải thu;
--   B: "chuyến hoàn thành có cước phải có đúng 1 charge thành công, chuyến huỷ
--      không có charge thành công nào".
SELECT SUM(IIF(x.tong_moi_charge <> s.phai_thu, 1, 0))                         AS ngay_tho_a,
       SUM(IIF((s.status = 'completed' AND s.phai_thu > 0 AND s.so_thu_ok <> 1)
            OR (s.status = 'cancelled' AND s.so_thu_ok > 0), 1, 0))           AS ngay_tho_b,
       SUM(IIF(s.loai_lech IS NOT NULL, 1, 0))                                 AS quy_tac_dung
FROM lab.so_doi_soat AS s
OUTER APPLY (SELECT ISNULL(SUM(CAST(p.amount AS bigint)), 0) AS tong_moi_charge
             FROM toinoi.payments AS p
             WHERE p.trip_id = s.id AND p.kind = 'charge') AS x;

DECLARE @a int, @b int, @b_trung int, @b_sot nvarchar(100);
SELECT @a = SUM(IIF(x.tong_moi_charge <> s.phai_thu, 1, 0))
FROM lab.so_doi_soat AS s
OUTER APPLY (SELECT ISNULL(SUM(CAST(p.amount AS bigint)), 0) AS tong_moi_charge
             FROM toinoi.payments AS p WHERE p.trip_id = s.id AND p.kind = 'charge') AS x;
SELECT @b = COUNT(*), @b_trung = SUM(IIF(loai_lech IS NOT NULL, 1, 0))
FROM lab.so_doi_soat
WHERE (status = 'completed' AND phai_thu > 0 AND so_thu_ok <> 1) OR (status = 'cancelled' AND so_thu_ok > 0);
SELECT @b_sot = STRING_AGG(CAST(code AS varchar(12)), ', ') WITHIN GROUP (ORDER BY code)
FROM lab.so_doi_soat
WHERE loai_lech IS NOT NULL
  AND NOT ((status = 'completed' AND phai_thu > 0 AND so_thu_ok <> 1) OR (status = 'cancelled' AND so_thu_ok > 0));
EXEC lab.expect_value N'4 quy tắc ngây thơ A báo', @a, N'115';
DECLARE @bb nvarchar(60) = CONCAT(@b, '|', @b_trung);
EXEC lab.expect_value N'4 quy tắc ngây thơ B báo | trong đó lệch thật', @bb, N'42|4';
EXEC lab.expect_value N'4 quy tắc ngây thơ B bỏ sót', @b_sot, N'101987, 102443, 102642';
GO

-- ---------------------------------------------------------------------------
-- 5. CHÊNH LỆCH THỜI ĐIỂM — không phải lỗi: theo ngày đặt chuyến (sổ) và theo
--    ngày giao dịch (paid_at giờ VN, như bảng kê của cổng thanh toán).
SELECT d.ngay,
       (SELECT SUM(s.thuc_thu) FROM lab.so_doi_soat AS s WHERE s.ngay = d.ngay) AS theo_ngay_dat_chuyen,
       (SELECT SUM(IIF(p.kind = 'refund', -CAST(p.amount AS bigint), CAST(p.amount AS bigint)))
          FROM toinoi.payments AS p
         WHERE p.status = 'succeeded'
           AND p.paid_at >= CAST(d.ngay AS datetime2(3)) AT TIME ZONE 'SE Asia Standard Time'
           AND p.paid_at <  CAST(DATEADD(day, 1, d.ngay) AS datetime2(3)) AT TIME ZONE 'SE Asia Standard Time'
       ) AS theo_ngay_giao_dich
FROM (VALUES (CAST('2026-10-15' AS date)), (CAST('2026-10-16' AS date))) AS d (ngay)
ORDER BY d.ngay;
GO

-- Đối chiếu bằng cách khác (SWITCHOFFSET rồi lấy ngày) cho cả hai ngày.
DECLARE @ngay nvarchar(200) =
    (SELECT STRING_AGG(CONCAT(ngay, ':', dat, ':', gd), ', ') WITHIN GROUP (ORDER BY ngay)
     FROM (SELECT d.ngay,
                  (SELECT SUM(s.thuc_thu) FROM lab.so_doi_soat AS s WHERE s.ngay = d.ngay) AS dat,
                  (SELECT SUM(IIF(p.kind = 'refund', -CAST(p.amount AS bigint), CAST(p.amount AS bigint)))
                     FROM toinoi.payments AS p
                    WHERE p.status = 'succeeded'
                      AND CAST(SWITCHOFFSET(p.paid_at, '+07:00') AS date) = d.ngay) AS gd
           FROM (VALUES (CAST('2026-10-15' AS date)), (CAST('2026-10-16' AS date))) AS d (ngay)) AS x);
EXEC lab.expect_value N'5 ngày:theo ngày đặt:theo ngày giao dịch', @ngay,
     N'2026-10-15:5862000:5823000, 2026-10-16:5244000:5228000';

-- Cả tháng: điều chỉnh +5.000 đ của 103165 (đặt 30/10) ghi lúc 00:05 ngày 31/10.
DECLARE @thang_dat bigint = (SELECT SUM(thuc_thu) FROM lab.so_doi_soat);
DECLARE @thang_gd bigint =
    (SELECT SUM(IIF(kind = 'refund', -CAST(amount AS bigint), CAST(amount AS bigint))) FROM toinoi.payments
     WHERE status = 'succeeded'
       AND paid_at >= '2026-10-01 00:00:00 +07:00' AND paid_at < '2026-10-31 00:00:00 +07:00');
DECLARE @sau bigint =
    (SELECT SUM(IIF(kind = 'refund', -CAST(amount AS bigint), CAST(amount AS bigint))) FROM toinoi.payments
     WHERE status = 'succeeded' AND paid_at >= '2026-10-31 00:00:00 +07:00');
SELECT @thang_dat AS theo_ngay_dat_chuyen, @thang_gd AS theo_ngay_giao_dich_thang_10, @sau AS ghi_nhan_tu_31_10;
SELECT t.code, p.kind, p.amount, CONVERT(varchar(34), SWITCHOFFSET(p.paid_at, '+07:00'), 121) AS paid_at_vn
FROM toinoi.payments AS p
JOIN toinoi.trips AS t ON t.id = p.trip_id
WHERE p.status = 'succeeded' AND p.paid_at >= '2026-10-31 00:00:00 +07:00'
ORDER BY p.paid_at, p.id;
DECLARE @thang nvarchar(60) = CONCAT(@thang_dat, '|', @thang_gd, '|', @sau);
EXEC lab.expect_value N'5 tháng: ngày đặt | ngày giao dịch | từ 31/10', @thang,
     N'182555000|182550000|5000';
GO

-- ---------------------------------------------------------------------------
-- 6. VIỆC CẦN THEO DÕI: giao dịch còn `pending` khi chuyến đã hoàn thành hơn
--    1 giờ (tính tới lúc chụp dữ liệu 2026-10-31 01:00 giờ VN).
SELECT t.code, p.provider_ref, p.amount, CONVERT(varchar(34), SWITCHOFFSET(p.paid_at, '+07:00'), 121) AS paid_at_vn
FROM toinoi.payments AS p
JOIN toinoi.trips AS t ON t.id = p.trip_id
WHERE p.status = 'pending'
  AND t.completed_at < DATEADD(hour, -1, CAST('2026-10-31 01:00:00 +07:00' AS datetimeoffset(3)))
ORDER BY t.code;
DECLARE @pending nvarchar(100) =
    (SELECT STRING_AGG(CONCAT(t.code, ':', p.provider_ref), ', ') WITHIN GROUP (ORDER BY t.code)
     FROM toinoi.payments AS p JOIN toinoi.trips AS t ON t.id = p.trip_id
     WHERE p.status = 'pending'
       AND t.completed_at < DATEADD(hour, -1, CAST('2026-10-31 01:00:00 +07:00' AS datetimeoffset(3))));
EXEC lab.expect_value N'6 pending cần theo dõi', @pending, N'103162:TH-103162-P';

-- Nếu cổng chuyển giao dịch đó sang succeeded, 103162 trở thành THU TRÙNG.
BEGIN TRANSACTION;
UPDATE toinoi.payments SET status = 'succeeded' WHERE provider_ref = 'TH-103162-P';
DECLARE @neu nvarchar(100) = (SELECT CONCAT(code, ':', loai_lech, ':', chenh) FROM lab.so_doi_soat WHERE code = 103162);
ROLLBACK TRANSACTION;
EXEC lab.expect_value N'6 giả định pending → succeeded', @neu, N'103162:4 THU TRÙNG:57000';
DECLARE @sau_rb nvarchar(20) = (SELECT status FROM toinoi.payments WHERE provider_ref = 'TH-103162-P');
EXEC lab.expect_value N'6 sau ROLLBACK vẫn', @sau_rb, N'pending';
GO

-- ===========================================================================
-- 7. BÀI TẬP MỞ RỘNG (lời giải trong bài viết)
-- ===========================================================================

-- Bài 1. Đối soát theo thành phố, có dòng tổng.
SELECT IIF(GROUPING(c.code) = 1, 'TONG', c.code) AS tp, COUNT(*) AS so_chuyen,
       SUM(s.phai_thu) AS phai_thu, SUM(s.thuc_thu) AS thuc_thu, SUM(s.chenh) AS chenh,
       SUM(IIF(s.loai_lech IS NOT NULL, 1, 0)) AS so_lech
FROM lab.so_doi_soat AS s
JOIN toinoi.cities AS c ON c.id = s.city_id
GROUP BY GROUPING SETS ((c.code), ())
ORDER BY GROUPING(c.code), c.code;
DECLARE @tp nvarchar(200) =
    (SELECT STRING_AGG(CONCAT(tp, ':', chenh, ':', so_lech), ', ') WITHIN GROUP (ORDER BY g, tp)
     FROM (SELECT IIF(GROUPING(c.code) = 1, 'TONG', c.code) AS tp, GROUPING(c.code) AS g,
                  SUM(s.chenh) AS chenh, SUM(IIF(s.loai_lech IS NOT NULL, 1, 0)) AS so_lech
           FROM lab.so_doi_soat AS s JOIN toinoi.cities AS c ON c.id = s.city_id
           GROUP BY GROUPING SETS ((c.code), ())) AS x);
EXEC lab.expect_value N'BT1 thành phố:chênh:số lệch', @tp, N'DN:-98000:1, HN:-85000:2, SG:451000:4, TONG:268000:7';

-- Muốn thêm cột liệt kê mã lệch ngay trong câu GROUPING SETS? STRING_AGG không
-- gộp được từ các nhóm con ⇒ lỗi 8710. Liệt kê mã ở một câu GROUP BY thường.
EXEC lab.expect_error N'BT1 STRING_AGG trong GROUPING SETS',
     N'SELECT city_id, STRING_AGG(CAST(code AS varchar(12)), '','') FROM lab.so_doi_soat WHERE loai_lech IS NOT NULL GROUP BY GROUPING SETS ((city_id), ());',
     8710;
SELECT c.code AS tp, STRING_AGG(CAST(s.code AS varchar(12)), ', ') WITHIN GROUP (ORDER BY s.code) AS ma_lech
FROM lab.so_doi_soat AS s
JOIN toinoi.cities AS c ON c.id = s.city_id
WHERE s.loai_lech IS NOT NULL
GROUP BY c.code
ORDER BY c.code;
GO

-- Bài 2. Giảm 10.000 đ cho 100048 (cước 119.000) theo ba cách, mỗi cách trong
--        một transaction rồi ROLLBACK.
DECLARE @a nvarchar(200), @b nvarchar(200), @c nvarchar(200);

BEGIN TRANSACTION;   -- (a) chỉ thêm refund 10.000 vào ví
INSERT INTO toinoi.payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
VALUES ('70000000-0000-4000-8000-000000100048', 'refund', 'wallet', 'succeeded', 10000, 'VX-100048-BT2', '2026-10-31 09:00:00 +07:00');
SELECT @a = CONCAT((SELECT CONCAT(phai_thu, ':', thuc_thu, ':', chenh, ':', ISNULL(loai_lech, N'khớp')) FROM lab.so_doi_soat WHERE code = 100048),
                   ' | ', (SELECT CONCAT(COUNT(*), ':', SUM(chenh)) FROM lab.so_doi_soat WHERE loai_lech IS NOT NULL));
ROLLBACK TRANSACTION;

BEGIN TRANSACTION;   -- (b) chỉ thêm adjustment −10.000
INSERT INTO toinoi.payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
VALUES ('70000000-0000-4000-8000-000000100048', 'adjustment', 'wallet', 'succeeded', -10000, 'VX-100048-BT2', '2026-10-31 09:00:00 +07:00');
SELECT @b = CONCAT((SELECT CONCAT(phai_thu, ':', thuc_thu, ':', chenh, ':', ISNULL(loai_lech, N'khớp')) FROM lab.so_doi_soat WHERE code = 100048),
                   ' | ', (SELECT CONCAT(COUNT(*), ':', SUM(chenh)) FROM lab.so_doi_soat WHERE loai_lech IS NOT NULL));
ROLLBACK TRANSACTION;

BEGIN TRANSACTION;   -- (c) sửa cước còn 109.000 VÀ thêm adjustment −10.000
UPDATE toinoi.trips SET fare = 109000 WHERE code = 100048;
INSERT INTO toinoi.payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
VALUES ('70000000-0000-4000-8000-000000100048', 'adjustment', 'wallet', 'succeeded', -10000, 'VX-100048-BT2', '2026-10-31 09:00:00 +07:00');
SELECT @c = CONCAT((SELECT CONCAT(phai_thu, ':', thuc_thu, ':', chenh, ':', ISNULL(loai_lech, N'khớp')) FROM lab.so_doi_soat WHERE code = 100048),
                   ' | ', (SELECT CONCAT(COUNT(*), ':', SUM(chenh)) FROM lab.so_doi_soat WHERE loai_lech IS NOT NULL));
ROLLBACK TRANSACTION;

SELECT N'a' AS cach, @a AS ket_qua UNION ALL SELECT N'b', @b UNION ALL SELECT N'c', @c;
EXEC lab.expect_value N'BT2 (a) chỉ refund', @a, N'119000:109000:-10000:5 LỆCH SỐ TIỀN | 8:258000';
EXEC lab.expect_value N'BT2 (b) chỉ adjustment', @b, N'119000:109000:-10000:5 LỆCH SỐ TIỀN | 8:258000';
EXEC lab.expect_value N'BT2 (c) sửa cước + adjustment', @c, N'109000:109000:0:khớp | 7:268000';
DECLARE @ve nvarchar(100) = (SELECT CONCAT(t.fare, ':', (SELECT COUNT(*) FROM toinoi.payments AS p WHERE p.trip_id = t.id))
                             FROM toinoi.trips AS t WHERE t.code = 100048);
EXEC lab.expect_value N'BT2 sau ROLLBACK cước:số giao dịch của 100048', @ve, N'119000:2';
GO

-- Bài 3. Tín hiệu thu trùng chặt hơn: cùng số tiền, cùng phương thức, cách
--        nhau ≤ 60 giây. Mỗi cặp xuất hiện một lần nhờ (paid_at, id) tăng dần.
SELECT t.code, a.method, a.amount, DATEDIFF(second, a.paid_at, b.paid_at) AS cach_giay
FROM toinoi.payments AS a
JOIN toinoi.payments AS b
  ON  b.trip_id = a.trip_id AND b.amount = a.amount AND b.method = a.method
  AND (b.paid_at > a.paid_at OR (b.paid_at = a.paid_at AND b.id > a.id))
JOIN toinoi.trips AS t ON t.id = a.trip_id
WHERE a.kind = 'charge' AND a.status = 'succeeded'
  AND b.kind = 'charge' AND b.status = 'succeeded'
  AND DATEDIFF(second, a.paid_at, b.paid_at) <= 60
ORDER BY t.code;
DECLARE @cap nvarchar(100) =
    (SELECT STRING_AGG(CONCAT(t.code, ':', a.method, ':', DATEDIFF(second, a.paid_at, b.paid_at)), ', ')
     FROM toinoi.payments AS a
     JOIN toinoi.payments AS b
       ON  b.trip_id = a.trip_id AND b.amount = a.amount AND b.method = a.method
       AND (b.paid_at > a.paid_at OR (b.paid_at = a.paid_at AND b.id > a.id))
     JOIN toinoi.trips AS t ON t.id = a.trip_id
     WHERE a.kind = 'charge' AND a.status = 'succeeded' AND b.kind = 'charge' AND b.status = 'succeeded'
       AND DATEDIFF(second, a.paid_at, b.paid_at) <= 60);
EXEC lab.expect_value N'BT3 cặp thu trùng chặt', @cap, N'101310:wallet:3';
GO

-- Dữ liệu sau file không đổi.
DECLARE @cuoi nvarchar(100) = CONCAT((SELECT COUNT(*) FROM toinoi.trips), '|', (SELECT COUNT(*) FROM toinoi.payments),
                                     '|', (SELECT SUM(CAST(fare AS bigint)) FROM toinoi.trips WHERE status = 'completed'));
EXEC lab.expect_value N'cuối file trips | payments | doanh thu', @cuoi, N'3174|2826|182287000';
GO
