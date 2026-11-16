-- P02 · 19-cay-khu-vuc.sql — MS-019 `mssql-recursive-cte-cay-khu-vuc` (phần 1/2)
-- Recursive CTE với cây khu vực: duyệt cây có ĐIỀU KIỆN DỪNG và MAXRECURSION.
-- Phần 2 — dữ liệu có chu trình — ở 19-chu-trinh.sql.
--
-- Cây điều phối (bảng toinoi.areas, 34 nút): 3 thành phố → 10 vùng → 21 khu.
-- Vùng HN-NA mới mở, chưa có khu nào. Mỗi chuyến gắn với một KHU
-- (pickup_area_id); 36 chuyến mất GPS có pickup_area_id NULL.
--
-- T-SQL viết recursive CTE bằng WITH thường (KHÔNG có từ khoá RECURSIVE):
--   * phần neo (anchor member): chạy MỘT lần, cho các dòng xuất phát;
--   * UNION ALL (bắt buộc ALL giữa neo cuối và phần đệ quy);
--   * phần đệ quy (recursive member): tham chiếu chính CTE, chạy lặp trên các
--     dòng của vòng TRƯỚC — dừng khi một vòng không sinh thêm dòng nào;
--   * OPTION (MAXRECURSION n): chốt số vòng, mặc định 100 — quá thì lỗi 530.
--
-- File chỉ đọc.

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

-- ---------------------------------------------------------------------------
-- 1. Giải phẫu đệ quy: đếm 1 → 5. Điều kiện dừng nằm ở WHERE của phần đệ quy.
WITH dem (n) AS (
    SELECT 1                       -- neo
    UNION ALL
    SELECT n + 1 FROM dem          -- đệ quy: chỉ thấy các dòng của vòng TRƯỚC
    WHERE n < 5                    -- điều kiện dừng
)
SELECT n FROM dem;

DECLARE @v nvarchar(400);
WITH dem (n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM dem WHERE n < 5)
SELECT @v = STRING_AGG(CAST(n AS varchar(3)), ',') WITHIN GROUP (ORDER BY n) FROM dem;
EXEC lab.expect_value N'đếm 1 → 5', @v, N'1,2,3,4,5';
GO

-- Ba lỗi hay gặp khi chép từ PostgreSQL hoặc viết vội:
EXEC lab.expect_error N'WITH RECURSIVE (cú pháp PostgreSQL)',
    N'WITH RECURSIVE dem (n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM dem WHERE n < 5) SELECT n FROM dem',
    102, N'Incorrect syntax';
EXEC lab.expect_error N'UNION thay cho UNION ALL',
    N'WITH dem (n) AS (SELECT 1 UNION SELECT n + 1 FROM dem WHERE n < 5) SELECT n FROM dem',
    252, N'UNION ALL';
-- Kiểu cột của phần đệ quy phải TRÙNG KHÍT phần neo: varchar(400) + '/x'
-- cho ra varchar(402), nên phần đệ quy cũng phải CAST lại.
EXEC lab.expect_error N'kiểu cột lệch giữa neo và đệ quy',
    N'WITH cay (id, duong_di) AS (
          SELECT id, CAST(code AS varchar(400)) FROM toinoi.areas WHERE code = ''SG''
          UNION ALL
          SELECT a.id, c.duong_di + ''/'' + a.code
          FROM toinoi.areas AS a JOIN cay AS c ON a.parent_id = c.id)
      SELECT * FROM cay',
    240, N'duong_di';
GO

-- ---------------------------------------------------------------------------
-- 2. Đi XUỐNG: mọi nút thuộc cây TP. Hồ Chí Minh, kèm độ sâu và đường đi.
--    SQL Server không có mệnh đề SEARCH DEPTH FIRST ⇒ tự dựng chuỗi đường đi
--    và sắp theo nó (collation nhị phân: so từng byte, ký tự '/' đứng trước
--    chữ cái và chữ số).
WITH cay (id, code, name, do_sau, duong_di) AS (
    SELECT id, code, name, 0, CAST(code AS varchar(400))
    FROM toinoi.areas
    WHERE code = 'SG'
    UNION ALL
    SELECT a.id, a.code, a.name, c.do_sau + 1, CAST(c.duong_di + '/' + a.code AS varchar(400))
    FROM toinoi.areas AS a
    JOIN cay AS c ON a.parent_id = c.id
)
SELECT do_sau, code, name, duong_di
FROM cay
ORDER BY duong_di COLLATE Latin1_General_BIN2;

DECLARE @v nvarchar(1000);
WITH cay (id, code, do_sau, duong_di) AS (
    SELECT id, code, 0, CAST(code AS varchar(400)) FROM toinoi.areas WHERE code = 'SG'
    UNION ALL
    SELECT a.id, a.code, c.do_sau + 1, CAST(c.duong_di + '/' + a.code AS varchar(400))
    FROM toinoi.areas AS a JOIN cay AS c ON a.parent_id = c.id
)
SELECT @v = STRING_AGG(CONCAT(do_sau, N':', code), N' ') WITHIN GROUP (ORDER BY duong_di COLLATE Latin1_General_BIN2) FROM cay;
EXEC lab.expect_value N'cây SG theo chiều sâu', @v,
     N'0:SG 1:SG-DO 2:SG-DO-01 2:SG-DO-02 2:SG-DO-03 1:SG-NA 2:SG-NA-01 2:SG-NA-02 1:SG-TA 2:SG-TA-01 2:SG-TA-02 1:SG-TT 2:SG-TT-01 2:SG-TT-02 2:SG-TT-03';
GO

-- ---------------------------------------------------------------------------
-- 3. Đi LÊN: tổ tiên của khu HN-TA-02 (khu → vùng → thành phố). Phần đệ quy
--    nối ngược: dòng mới là CHA của dòng vòng trước.
WITH to_tien (id, parent_id, code, level, bac) AS (
    SELECT id, parent_id, code, level, 0 FROM toinoi.areas WHERE code = 'HN-TA-02'
    UNION ALL
    SELECT a.id, a.parent_id, a.code, a.level, t.bac + 1
    FROM toinoi.areas AS a
    JOIN to_tien AS t ON a.id = t.parent_id
)
SELECT bac, code, level FROM to_tien ORDER BY bac;

DECLARE @v nvarchar(400);
WITH to_tien (id, parent_id, code, bac) AS (
    SELECT id, parent_id, code, 0 FROM toinoi.areas WHERE code = 'HN-TA-02'
    UNION ALL
    SELECT a.id, a.parent_id, a.code, t.bac + 1 FROM toinoi.areas AS a JOIN to_tien AS t ON a.id = t.parent_id
)
SELECT @v = STRING_AGG(code, ' > ') WITHIN GROUP (ORDER BY bac) FROM to_tien;
EXEC lab.expect_value N'tổ tiên của HN-TA-02', @v, N'HN-TA-02 > HN-TA > HN';
GO

-- ---------------------------------------------------------------------------
-- 4. MAXRECURSION. Mặc định SQL Server cho tối đa 100 vòng đệ quy; vòng 101
--    ⇒ lỗi 530 và câu lệnh dừng. Đếm tới 200 cần OPTION (MAXRECURSION n),
--    n từ 0 (không giới hạn) tới 32767.
EXEC lab.expect_error N'đếm tới 200 với MAXRECURSION mặc định',
    N'WITH dem (n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM dem WHERE n < 200) SELECT COUNT(*) FROM dem',
    530, N'maximum recursion 100';

DECLARE @n nvarchar(10);
WITH dem (n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM dem WHERE n < 200)
SELECT @n = COUNT(*) FROM dem
OPTION (MAXRECURSION 200);
EXEC lab.expect_value N'đếm tới 200 với MAXRECURSION 200', @n, N'200';

EXEC lab.expect_error N'MAXRECURSION vượt 32767',
    N'WITH dem (n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM dem WHERE n < 5) SELECT n FROM dem OPTION (MAXRECURSION 32768)',
    310, N'32767';
GO

-- Lỗi 530 KHÔNG thu hồi các dòng đã gửi: client nhận dòng của các vòng đã
-- chạy (1 … 4), rồi mới nhận lỗi.
BEGIN TRY
    WITH dem (n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM dem WHERE n < 10)
    SELECT n FROM dem
    OPTION (MAXRECURSION 3);
END TRY
BEGIN CATCH
    PRINT CONCAT(N'Msg ', ERROR_NUMBER(), N': ', ERROR_MESSAGE());
    DECLARE @e nvarchar(10) = CAST(ERROR_NUMBER() AS nvarchar(10));
    EXEC lab.expect_value N'MAXRECURSION 3 khi cần 9 vòng', @e, N'530';
END CATCH;
GO

-- ---------------------------------------------------------------------------
-- 5. Cộng dồn theo cây: số chuyến và doanh thu (chuyến hoàn thành) của MỌI
--    khu nằm dưới mỗi nút. Bước 1 dựng bảng "tổ tiên – hậu duệ" (mỗi cặp
--    (tổ tiên, hậu duệ) một dòng, kể cả cặp (nút, chính nó)); bước 2 nối với
--    chuyến. LEFT JOIN để vùng chưa có khu (HN-NA) vẫn hiện 0. In các nút
--    thành phố và vùng (bỏ 21 khu cho gọn).
WITH to_tien_hau_due (to_tien_id, hau_due_id) AS (
    SELECT id, id FROM toinoi.areas              -- mỗi nút là hậu duệ của chính nó
    UNION ALL
    SELECT c.to_tien_id, a.id
    FROM to_tien_hau_due AS c
    JOIN toinoi.areas AS a ON a.parent_id = c.hau_due_id
)
SELECT n.code, n.level,
       COUNT(t.id)                                                          AS so_chuyen,
       SUM(CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END)  AS doanh_thu
FROM toinoi.areas AS n
JOIN to_tien_hau_due AS c ON c.to_tien_id = n.id
LEFT JOIN toinoi.trips AS t ON t.pickup_area_id = c.hau_due_id
WHERE n.level <> 'zone'
GROUP BY n.id, n.code, n.level
ORDER BY n.code;

DECLARE @v nvarchar(1000);
WITH to_tien_hau_due (to_tien_id, hau_due_id) AS (
    SELECT id, id FROM toinoi.areas
    UNION ALL
    SELECT c.to_tien_id, a.id FROM to_tien_hau_due AS c JOIN toinoi.areas AS a ON a.parent_id = c.hau_due_id
),
cong_don AS (
    SELECT n.code, COUNT(t.id) AS so_chuyen,
           SUM(CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END) AS doanh_thu
    FROM toinoi.areas AS n
    JOIN to_tien_hau_due AS c ON c.to_tien_id = n.id
    LEFT JOIN toinoi.trips AS t ON t.pickup_area_id = c.hau_due_id
    WHERE n.level <> 'zone'
    GROUP BY n.id, n.code
)
SELECT @v = STRING_AGG(CONCAT(code, N' ', so_chuyen, N' ', doanh_thu), N', ') WITHIN GROUP (ORDER BY code) FROM cong_don;
EXEC lab.expect_value N'cộng dồn theo cây', @v,
     N'DN 456 26124000, DN-BI 231 13618000, DN-TT 225 12506000, HN 1094 62246000, HN-BA 340 19456000, HN-NA 0 0, HN-TA 295 16496000, HN-TT 459 26294000, SG 1588 92143000, SG-DO 481 28484000, SG-NA 315 17353000, SG-TA 315 18523000, SG-TT 477 27783000';
GO

-- Đối chiếu: tổng của các thành phố qua cây PHẢI bằng đếm trực tiếp chuyến
-- có khu đón (36 chuyến mất GPS không thuộc nút nào).
WITH to_tien_hau_due (to_tien_id, hau_due_id) AS (
    SELECT id, id FROM toinoi.areas
    UNION ALL
    SELECT c.to_tien_id, a.id FROM to_tien_hau_due AS c JOIN toinoi.areas AS a ON a.parent_id = c.hau_due_id
)
SELECT (SELECT COUNT(*) FROM toinoi.areas AS n
          JOIN to_tien_hau_due AS c ON c.to_tien_id = n.id
          JOIN toinoi.trips AS t ON t.pickup_area_id = c.hau_due_id
         WHERE n.level = 'city')                                            AS qua_cay,
       (SELECT COUNT(*) FROM toinoi.trips WHERE pickup_area_id IS NOT NULL) AS truc_tiep,
       (SELECT COUNT(*) FROM toinoi.trips WHERE pickup_area_id IS NULL)     AS mat_gps;

DECLARE @v nvarchar(100);
WITH to_tien_hau_due (to_tien_id, hau_due_id) AS (
    SELECT id, id FROM toinoi.areas
    UNION ALL
    SELECT c.to_tien_id, a.id FROM to_tien_hau_due AS c JOIN toinoi.areas AS a ON a.parent_id = c.hau_due_id
)
SELECT @v = CONCAT(
       (SELECT COUNT(*) FROM toinoi.areas AS n JOIN to_tien_hau_due AS c ON c.to_tien_id = n.id
          JOIN toinoi.trips AS t ON t.pickup_area_id = c.hau_due_id WHERE n.level = 'city'), N' | ',
       (SELECT COUNT(*) FROM toinoi.trips WHERE pickup_area_id IS NOT NULL), N' | ',
       (SELECT COUNT(*) FROM toinoi.trips WHERE pickup_area_id IS NULL));
EXEC lab.expect_value N'qua cây | trực tiếp | mất GPS', @v, N'3138 | 3138 | 36';
GO

-- ---------------------------------------------------------------------------
-- 6. Kiểm tra cây LÀNH trước khi tin các con số trên: mọi nút phải đi tới
--    được từ một gốc, và độ sâu có giới hạn. `WHERE c.do_sau < 10` là chốt
--    an toàn của chính truy vấn: dù dữ liệu có chu trình, truy vấn vẫn dừng
--    sau 10 vòng thay vì chạy tới lỗi 530 (19-chu-trinh.sql).
WITH tu_goc (id, do_sau) AS (
    SELECT id, 0 FROM toinoi.areas WHERE parent_id IS NULL
    UNION ALL
    SELECT a.id, c.do_sau + 1
    FROM toinoi.areas AS a JOIN tu_goc AS c ON a.parent_id = c.id
    WHERE c.do_sau < 10
)
SELECT (SELECT COUNT(*) FROM toinoi.areas)                       AS tong_nut,
       (SELECT COUNT(DISTINCT id) FROM tu_goc)                   AS di_toi_tu_goc,
       (SELECT MAX(do_sau) FROM tu_goc)                          AS sau_nhat,
       (SELECT COUNT(*) FROM toinoi.areas AS a
         WHERE NOT EXISTS (SELECT 1 FROM toinoi.areas AS con WHERE con.parent_id = a.id)) AS so_la;

DECLARE @v nvarchar(100);
WITH tu_goc (id, do_sau) AS (
    SELECT id, 0 FROM toinoi.areas WHERE parent_id IS NULL
    UNION ALL
    SELECT a.id, c.do_sau + 1 FROM toinoi.areas AS a JOIN tu_goc AS c ON a.parent_id = c.id WHERE c.do_sau < 10
)
SELECT @v = CONCAT((SELECT COUNT(*) FROM toinoi.areas), N' | ', (SELECT COUNT(DISTINCT id) FROM tu_goc), N' | ',
                   (SELECT MAX(do_sau) FROM tu_goc), N' | ',
                   (SELECT COUNT(*) FROM toinoi.areas AS a WHERE NOT EXISTS (SELECT 1 FROM toinoi.areas AS con WHERE con.parent_id = a.id)));
EXEC lab.expect_value N'tổng nút | đi tới từ gốc | sâu nhất | số lá', @v, N'34 | 34 | 2 | 22';
GO

-- ===========================================================================
-- Lời giải bài tập
-- ===========================================================================

-- Bài 1. Mọi nút LÁ của cây Hà Nội kèm đường đi đầy đủ (HN-NA là lá cấp vùng).
DECLARE @v nvarchar(1000);
WITH cay (id, code, duong_di) AS (
    SELECT id, code, CAST(code AS varchar(400)) FROM toinoi.areas WHERE code = 'HN'
    UNION ALL
    SELECT a.id, a.code, CAST(c.duong_di + '/' + a.code AS varchar(400))
    FROM toinoi.areas AS a JOIN cay AS c ON a.parent_id = c.id
)
SELECT @v = STRING_AGG(duong_di, ', ') WITHIN GROUP (ORDER BY duong_di COLLATE Latin1_General_BIN2)
FROM cay AS c
WHERE NOT EXISTS (SELECT 1 FROM toinoi.areas AS con WHERE con.parent_id = c.id);
EXEC lab.expect_value N'Bài 1: lá của HN', @v,
     N'HN/HN-BA/HN-BA-01, HN/HN-BA/HN-BA-02, HN/HN-NA, HN/HN-TA/HN-TA-01, HN/HN-TA/HN-TA-02, HN/HN-TT/HN-TT-01, HN/HN-TT/HN-TT-02, HN/HN-TT/HN-TT-03';
GO

-- Bài 2. Vùng có doanh thu cao nhất của mỗi thành phố (tie-breaker: code).
DECLARE @v nvarchar(1000);
WITH to_tien_hau_due (to_tien_id, hau_due_id) AS (
    SELECT id, id FROM toinoi.areas
    UNION ALL
    SELECT c.to_tien_id, a.id FROM to_tien_hau_due AS c JOIN toinoi.areas AS a ON a.parent_id = c.hau_due_id
),
vung AS (
    SELECT n.parent_id AS tp_id, n.code,
           SUM(CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END) AS doanh_thu
    FROM toinoi.areas AS n
    JOIN to_tien_hau_due AS c ON c.to_tien_id = n.id
    LEFT JOIN toinoi.trips AS t ON t.pickup_area_id = c.hau_due_id
    WHERE n.level = 'district'
    GROUP BY n.id, n.parent_id, n.code
),
xep_hang AS (
    SELECT tp_id, code, doanh_thu,
           ROW_NUMBER() OVER (PARTITION BY tp_id ORDER BY doanh_thu DESC, code) AS hang
    FROM vung
)
SELECT @v = STRING_AGG(CONCAT(code, N' ', doanh_thu), N', ') WITHIN GROUP (ORDER BY code)
FROM xep_hang WHERE hang = 1;
EXEC lab.expect_value N'Bài 2: vùng doanh thu cao nhất mỗi thành phố', @v, N'DN-BI 13618000, HN-TT 26294000, SG-DO 28484000';
GO

-- Bài 3. hierarchyid: dựng giá trị '/2/21/211/' từ đường đi bằng id, rồi đếm
-- chuyến của cây HN bằng IsDescendantOf thay cho bảng tổ tiên – hậu duệ.
DECLARE @v nvarchar(100);
WITH cay (id, duong_di) AS (
    SELECT id, CAST('/' + CAST(id AS varchar(10)) + '/' AS varchar(400))
    FROM toinoi.areas WHERE parent_id IS NULL
    UNION ALL
    SELECT a.id, CAST(c.duong_di + CAST(a.id AS varchar(10)) + '/' AS varchar(400))
    FROM toinoi.areas AS a JOIN cay AS c ON a.parent_id = c.id
),
nut AS (
    SELECT id, CAST(duong_di AS hierarchyid) AS h FROM cay
)
SELECT @v = CONCAT(
       (SELECT h.ToString() FROM nut WHERE id = 211), N' | ',
       (SELECT h.GetLevel() FROM nut WHERE id = 211), N' | ',
       (SELECT COUNT(*) FROM toinoi.trips AS t
          JOIN nut AS k ON k.id = t.pickup_area_id
          JOIN nut AS hn ON hn.id = 2
         WHERE k.h.IsDescendantOf(hn.h) = 1));
EXEC lab.expect_value N'Bài 3: hierarchyid của 211 | cấp | số chuyến HN', @v, N'/2/21/211/ | 3 | 1094';
GO
