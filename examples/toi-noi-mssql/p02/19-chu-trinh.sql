-- P02 · 19-chu-trinh.sql — MS-019 `mssql-recursive-cte-cay-khu-vuc` (phần 2/2)
-- Fixture CHU TRÌNH: phát hiện dữ liệu cây bị vòng.
--
-- Tình huống: bản cập nhật cây khu vực từ bộ phận vận hành bị ĐẢO cha của
-- hai vùng Hà Nội — HN-TT (21) nhận cha là khu HN-TA-01 (221), còn HN-TA (22)
-- nhận cha là khu HN-TT-01 (211). Mọi ràng buộc đều qua: FK (cha tồn tại),
-- ck_areas_root_no_parent (vùng có cha), ck_areas_not_own_parent (không tự
-- làm cha mình). CHECK chỉ nhìn MỘT dòng nên không thấy vòng
--     21 → 221 → 22 → 211 → 21.
--
-- Toàn bộ file chạy trong MỘT giao dịch và ROLLBACK ở cuối: các file sau vẫn
-- thấy cây lành. SQL Server KHÔNG có mệnh đề CYCLE ⇒ tự mang chuỗi đường đi.
-- Truy vấn đệ quy "trần" (không chốt) trên dữ liệu này không chạy mãi: nó
-- dừng ở lỗi 530 sau 100 vòng (MAXRECURSION mặc định) — mục 5.

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

BEGIN TRANSACTION;

DECLARE @doi TABLE (id int, code varchar(12), cha_cu int, cha_moi int);

UPDATE toinoi.areas
SET parent_id = CASE id WHEN 21 THEN 221 WHEN 22 THEN 211 END
OUTPUT inserted.id, inserted.code, deleted.parent_id, inserted.parent_id INTO @doi
WHERE id IN (21, 22);

SELECT id, code, cha_cu, cha_moi FROM @doi ORDER BY id;

DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CONCAT(id, N':', cha_cu, N'→', cha_moi), N', ') WITHIN GROUP (ORDER BY id) FROM @doi);
EXEC lab.expect_value N'đảo cha hai vùng HN (mọi ràng buộc vẫn qua)', @v, N'21:2→221, 22:2→211';

-- Ràng buộc vẫn được tin cậy (không bị tắt) — chúng đã kiểm và cho qua.
SET @v = (SELECT COUNT(*) FROM sys.check_constraints
          WHERE parent_object_id = OBJECT_ID(N'toinoi.areas') AND (is_disabled = 1 OR is_not_trusted = 1));
EXEC lab.expect_value N'CHECK của areas bị tắt hoặc không tin cậy', @v, N'0';

-- ---------------------------------------------------------------------------
-- 1. Hệ quả IM LẶNG: đi xuống từ gốc HN không còn thấy hai vùng và năm khu
--    của chúng. Báo cáo cộng dồn của HN tụt từ 1.094 chuyến (19-cay-khu-vuc)
--    xuống 340 — không có lỗi nào được báo. Đi xuống vẫn DỪNG: các nút trên
--    vòng không còn đi tới được từ HN.
WITH to_tien_hau_due (to_tien_id, hau_due_id) AS (
    SELECT id, id FROM toinoi.areas WHERE code = 'HN'
    UNION ALL
    SELECT c.to_tien_id, a.id FROM to_tien_hau_due AS c JOIN toinoi.areas AS a ON a.parent_id = c.hau_due_id
)
SELECT @v = CONCAT((SELECT COUNT(*) FROM to_tien_hau_due), N' | ',
                   (SELECT COUNT(*) FROM to_tien_hau_due AS c JOIN toinoi.trips AS t ON t.pickup_area_id = c.hau_due_id));
PRINT CONCAT(N'số nút dưới HN | số chuyến HN: ', @v);
EXEC lab.expect_value N'cây HN sau khi đảo cha (số nút | số chuyến)', @v, N'5 | 340';

-- ---------------------------------------------------------------------------
-- 2. Đi LÊN từ khu HN-TT-01 thì không bao giờ tới gốc. Chốt dừng theo ĐỘ SÂU
--    (bac < 5) cho thấy vòng lặp lại — chốt này chỉ để an toàn, không phát
--    hiện được chu trình.
WITH to_tien (id, parent_id, code, bac) AS (
    SELECT id, parent_id, code, 0 FROM toinoi.areas WHERE code = 'HN-TT-01'
    UNION ALL
    SELECT a.id, a.parent_id, a.code, t.bac + 1
    FROM toinoi.areas AS a JOIN to_tien AS t ON a.id = t.parent_id
    WHERE t.bac < 5
)
SELECT bac, code FROM to_tien ORDER BY bac;

WITH to_tien (id, parent_id, code, bac) AS (
    SELECT id, parent_id, code, 0 FROM toinoi.areas WHERE code = 'HN-TT-01'
    UNION ALL
    SELECT a.id, a.parent_id, a.code, t.bac + 1 FROM toinoi.areas AS a JOIN to_tien AS t ON a.id = t.parent_id WHERE t.bac < 5
)
SELECT @v = STRING_AGG(code, ' > ') WITHIN GROUP (ORDER BY bac) FROM to_tien;
EXEC lab.expect_value N'đi lên từ HN-TT-01 với chốt độ sâu', @v, N'HN-TT-01 > HN-TT > HN-TA-01 > HN-TA > HN-TT-01 > HN-TT';

-- ---------------------------------------------------------------------------
-- 3. Phát hiện chu trình bằng chuỗi đường đi: mang theo '/id/id/…/' đã đi
--    qua; dòng nào có id đã nằm trong chuỗi thì đánh dấu la_chu_trinh = 1 và
--    KHÔNG đệ quy tiếp ⇒ truy vấn tự dừng, không cần MAXRECURSION.
WITH to_tien (id, parent_id, code, bac, da_qua, la_chu_trinh) AS (
    SELECT id, parent_id, code, 0,
           CAST('/' + CAST(id AS varchar(10)) + '/' AS varchar(400)), 0
    FROM toinoi.areas WHERE code = 'HN-TT-01'
    UNION ALL
    SELECT a.id, a.parent_id, a.code, t.bac + 1,
           CAST(t.da_qua + CAST(a.id AS varchar(10)) + '/' AS varchar(400)),
           CASE WHEN CHARINDEX('/' + CAST(a.id AS varchar(10)) + '/', t.da_qua) > 0 THEN 1 ELSE 0 END
    FROM toinoi.areas AS a JOIN to_tien AS t ON a.id = t.parent_id
    WHERE t.la_chu_trinh = 0                       -- dòng đã quay vòng thì dừng
)
SELECT bac, code, da_qua, la_chu_trinh FROM to_tien ORDER BY bac;

WITH to_tien (id, parent_id, bac, da_qua, la_chu_trinh) AS (
    SELECT id, parent_id, 0, CAST('/' + CAST(id AS varchar(10)) + '/' AS varchar(400)), 0
    FROM toinoi.areas WHERE code = 'HN-TT-01'
    UNION ALL
    SELECT a.id, a.parent_id, t.bac + 1, CAST(t.da_qua + CAST(a.id AS varchar(10)) + '/' AS varchar(400)),
           CASE WHEN CHARINDEX('/' + CAST(a.id AS varchar(10)) + '/', t.da_qua) > 0 THEN 1 ELSE 0 END
    FROM toinoi.areas AS a JOIN to_tien AS t ON a.id = t.parent_id
    WHERE t.la_chu_trinh = 0
)
SELECT @v = (SELECT CONCAT(da_qua, N' ', la_chu_trinh) FROM to_tien WHERE bac = (SELECT MAX(bac) FROM to_tien));
EXEC lab.expect_value N'dòng cuối khi phát hiện vòng', @v, N'/211/21/221/22/211/ 1';

-- ---------------------------------------------------------------------------
-- 4. Truy vấn KIỂM TRA cho cả bảng (chạy định kỳ / trước khi nhận bản cập nhật):
--    a) nút KHÔNG đi tới được từ gốc nào — gồm nút nằm trên vòng và mọi hậu
--       duệ. (PostgreSQL có thể dùng UNION để tự dừng; SQL Server bắt buộc
--       UNION ALL nên đi xuống từ gốc là đủ: nút trên vòng không bao giờ
--       được đi tới.)
WITH tu_goc (id) AS (
    SELECT id FROM toinoi.areas WHERE parent_id IS NULL
    UNION ALL
    SELECT a.id FROM toinoi.areas AS a JOIN tu_goc AS g ON a.parent_id = g.id
)
SELECT a.id, a.code
FROM toinoi.areas AS a
WHERE NOT EXISTS (SELECT 1 FROM tu_goc AS g WHERE g.id = a.id)
ORDER BY a.id;

WITH tu_goc (id) AS (
    SELECT id FROM toinoi.areas WHERE parent_id IS NULL
    UNION ALL
    SELECT a.id FROM toinoi.areas AS a JOIN tu_goc AS g ON a.parent_id = g.id
)
SELECT @v = STRING_AGG(code, ', ') WITHIN GROUP (ORDER BY a.id)
FROM toinoi.areas AS a WHERE NOT EXISTS (SELECT 1 FROM tu_goc AS g WHERE g.id = a.id);
EXEC lab.expect_value N'nút không tới được từ gốc', @v, N'HN-TT, HN-TA, HN-TT-01, HN-TT-02, HN-TT-03, HN-TA-01, HN-TA-02';

--    b) nút NẰM TRÊN vòng: đi lên từ chính nó và quay lại chính nó. Chuỗi
--       đường đi chặn việc đi lại một nút đã qua.
WITH len (xuat_phat, id, parent_id, da_qua) AS (
    SELECT id, id, parent_id, CAST('/' + CAST(id AS varchar(10)) + '/' AS varchar(400))
    FROM toinoi.areas
    UNION ALL
    SELECT l.xuat_phat, a.id, a.parent_id, CAST(l.da_qua + CAST(a.id AS varchar(10)) + '/' AS varchar(400))
    FROM len AS l JOIN toinoi.areas AS a ON a.id = l.parent_id
    WHERE CHARINDEX('/' + CAST(a.id AS varchar(10)) + '/', l.da_qua) = 0   -- dừng trước khi lặp
)
SELECT n.id, n.code, l.da_qua AS vong
FROM len AS l
JOIN toinoi.areas AS n ON n.id = l.xuat_phat
WHERE l.parent_id = l.xuat_phat                    -- bước kế tiếp sẽ quay về điểm xuất phát
ORDER BY n.id;

WITH len (xuat_phat, id, parent_id, da_qua) AS (
    SELECT id, id, parent_id, CAST('/' + CAST(id AS varchar(10)) + '/' AS varchar(400)) FROM toinoi.areas
    UNION ALL
    SELECT l.xuat_phat, a.id, a.parent_id, CAST(l.da_qua + CAST(a.id AS varchar(10)) + '/' AS varchar(400))
    FROM len AS l JOIN toinoi.areas AS a ON a.id = l.parent_id
    WHERE CHARINDEX('/' + CAST(a.id AS varchar(10)) + '/', l.da_qua) = 0
)
SELECT @v = STRING_AGG(CONCAT(l.xuat_phat, N' ', l.da_qua), N', ') WITHIN GROUP (ORDER BY l.xuat_phat)
FROM len AS l WHERE l.parent_id = l.xuat_phat;
EXEC lab.expect_value N'nút nằm trên vòng', @v, N'21 /21/221/22/211/, 22 /22/211/21/221/, 211 /211/21/221/22/, 221 /221/22/211/21/';

-- ---------------------------------------------------------------------------
-- 5. Truy vấn "trần": đi lên không chốt. SQL Server không lặp vô hạn — nó
--    dừng ở vòng 101 với lỗi 530. Trong giao dịch tường minh, lỗi này làm
--    giao dịch hỏng (XACT_STATE() = -1): chỉ còn ROLLBACK được. Vì vậy mục
--    này đặt CUỐI, ngay trước ROLLBACK.
DECLARE @loi int, @xact int, @so_dong int;
BEGIN TRY
    WITH to_tien (id, parent_id) AS (
        SELECT id, parent_id FROM toinoi.areas WHERE code = 'HN-TT-01'
        UNION ALL
        SELECT a.id, a.parent_id FROM toinoi.areas AS a JOIN to_tien AS t ON a.id = t.parent_id
    )
    SELECT @so_dong = COUNT(*) FROM to_tien;
END TRY
BEGIN CATCH
    SELECT @loi = ERROR_NUMBER(), @xact = XACT_STATE();
    PRINT CONCAT(N'Msg ', ERROR_NUMBER(), N': ', ERROR_MESSAGE());
END CATCH;
SET @v = CONCAT(@loi, N' | ', @xact);
EXEC lab.expect_value N'truy vấn trần trên vòng (lỗi | XACT_STATE)', @v, N'530 | -1';

ROLLBACK TRANSACTION;
GO

-- Sau ROLLBACK cây lành lại: mọi nút đi tới được từ gốc, cha cũ trở về.
DECLARE @v nvarchar(100);
WITH tu_goc (id) AS (
    SELECT id FROM toinoi.areas WHERE parent_id IS NULL
    UNION ALL
    SELECT a.id FROM toinoi.areas AS a JOIN tu_goc AS g ON a.parent_id = g.id
)
SELECT @v = CONCAT((SELECT COUNT(*) FROM toinoi.areas AS a WHERE NOT EXISTS (SELECT 1 FROM tu_goc AS g WHERE g.id = a.id)),
                   N' | ', (SELECT STRING_AGG(CONCAT(id, N':', parent_id), N',') WITHIN GROUP (ORDER BY id) FROM toinoi.areas WHERE id IN (21, 22)));
EXEC lab.expect_value N'sau ROLLBACK (nút không tới gốc | cha của 21, 22)', @v, N'0 | 21:2,22:2';
GO
