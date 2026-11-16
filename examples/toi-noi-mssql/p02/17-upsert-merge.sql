-- P02 · 17-upsert-merge.sql — MS-017 `mssql-upsert-merge`
-- Upsert: MERGE và các mẫu an toàn hơn — NHẬP DỮ LIỆU có quy tắc cập nhật,
-- với fixture CỐ Ý trùng khoá (cùng dữ liệu với 17-upsert-on-conflict.sql
-- của lab PostgreSQL).
--
-- Bài toán: quán gửi file thực đơn theo lô (batch) qua cổng đối tác. Tới Nơi
-- nạp vào bảng nhập `menu_item_imports` (nhận mọi thứ, không ràng buộc nghiệp
-- vụ), rồi hợp nhất sang `menu_items` theo quy tắc:
--   R1  khoá món = (quán, sku): chưa có thì THÊM, có rồi thì CẬP NHẬT;
--   R2  KHÔNG ghi đè dữ liệu MỚI hơn: chỉ cập nhật khi source_updated_at của
--       dòng nhập > của dòng đang có (lô đến trễ, gửi lại lô cũ…);
--   R3  không đổi gì thì KHÔNG ghi (tránh UPDATE thừa: vẫn ghi log, vẫn khoá
--       dòng, vẫn kích hoạt trigger nếu có);
--   R4  trong CÙNG một lô, một sku xuất hiện nhiều lần ⇒ lấy dòng mới nhất;
--   R5  dòng vi phạm ràng buộc khác (quán không tồn tại, trùng tên món khác
--       sku) ⇒ TỪ CHỐI dòng đó, không làm hỏng cả lô.
--
-- File tự tạo hai bảng của mình trong schema toinoi (DROP … IF EXISTS trước
-- nên chạy lại được), không đụng dữ liệu của 01-fixture. Đồng thời (hai phiên
-- cùng upsert một khoá) KHÔNG minh hoạ ở đây — Phần 4 (MS-036, MS-038).

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

-- ---------------------------------------------------------------------------
-- 0. Bảng đích và bảng nhập.
DROP TABLE IF EXISTS toinoi.menu_item_imports, toinoi.menu_items;

CREATE TABLE toinoi.menu_items (
    merchant_id       uniqueidentifier  NOT NULL,
    sku               varchar(20)       NOT NULL,
    name              nvarchar(100)     NOT NULL,
    price             int               NOT NULL,           -- đồng
    is_available      bit               NOT NULL,
    source_updated_at datetimeoffset(3) NOT NULL,           -- giờ sửa ở hệ thống của quán
    updated_by_batch  int               NOT NULL,           -- lô nào ghi dòng này lần cuối
    CONSTRAINT pk_menu_items PRIMARY KEY CLUSTERED (merchant_id, sku),
    CONSTRAINT uq_menu_items_name UNIQUE (merchant_id, name),
    CONSTRAINT fk_menu_items_merchant FOREIGN KEY (merchant_id) REFERENCES toinoi.merchants (id),
    CONSTRAINT ck_menu_items_price_positive CHECK (price > 0)
);

-- Bảng nhập: cố ý KHÔNG có FK/UNIQUE theo nghiệp vụ — nhận nguyên văn file.
CREATE TABLE toinoi.menu_item_imports (
    batch_id          int               NOT NULL,
    line_no           int               NOT NULL,
    merchant_id       uniqueidentifier  NOT NULL,
    sku               varchar(20)       NOT NULL,
    name              nvarchar(100)     NOT NULL,
    price             int               NOT NULL,
    is_available      bit               NOT NULL,
    source_updated_at datetimeoffset(3) NOT NULL,
    CONSTRAINT pk_menu_item_imports PRIMARY KEY CLUSTERED (batch_id, line_no)
);

-- Lô 1: thực đơn ban đầu của quán "Bún Bò Lab 101" (e1000000-…-000000000101).
INSERT INTO toinoi.menu_item_imports VALUES
    (1, 1, 'e1000000-0000-4000-8000-000000000101', 'BB-01', N'Bún bò đặc biệt', 55000, 1, '2026-10-20 08:00 +07:00'),
    (1, 2, 'e1000000-0000-4000-8000-000000000101', 'BB-02', N'Bún bò tái',      45000, 1, '2026-10-20 08:00 +07:00'),
    (1, 3, 'e1000000-0000-4000-8000-000000000101', 'BB-03', N'Bún bò chả',      45000, 1, '2026-10-20 08:00 +07:00'),
    (1, 4, 'e1000000-0000-4000-8000-000000000101', 'TR-01', N'Trà đá',           5000, 1, '2026-10-20 08:00 +07:00'),
    (1, 5, 'e1000000-0000-4000-8000-000000000101', 'NC-01', N'Nước cam',        25000, 1, '2026-10-20 08:00 +07:00');

-- Lô 2: mỗi dòng cài một tình huống (cột ket_qua của bảng xem trước ở mục 4).
INSERT INTO toinoi.menu_item_imports VALUES
    (2, 1, 'e1000000-0000-4000-8000-000000000101', 'BB-01', N'Bún bò đặc biệt', 59000, 1, '2026-10-28 09:00 +07:00'), -- tăng giá
    (2, 2, 'e1000000-0000-4000-8000-000000000101', 'BB-02', N'Bún bò tái',      45000, 1, '2026-10-28 09:00 +07:00'), -- y hệt, chỉ giờ mới
    (2, 3, 'e1000000-0000-4000-8000-000000000101', 'BB-03', N'Bún bò chả',      42000, 1, '2026-10-15 07:00 +07:00'), -- bản CŨ gửi lại
    (2, 4, 'e1000000-0000-4000-8000-000000000101', 'TR-01', N'Trà đá',           5000, 0, '2026-10-28 09:00 +07:00'), -- hết hàng
    (2, 5, 'e1000000-0000-4000-8000-000000000101', 'BB-04', N'Bún bò giò',      50000, 1, '2026-10-28 09:05 +07:00'), -- món mới…
    (2, 6, 'e1000000-0000-4000-8000-000000000101', 'BB-04', N'Bún bò giò',      52000, 1, '2026-10-28 09:30 +07:00'), -- …sửa giá trong cùng lô
    (2, 7, 'e1000000-0000-4000-8000-000000000101', 'BB-05', N'Nước cam',        25000, 1, '2026-10-28 09:00 +07:00'), -- sku mới, trùng TÊN NC-01
    (2, 8, 'e1000000-0000-4000-8000-000000000999', 'XX-01', N'Món lạ',          30000, 1, '2026-10-28 09:00 +07:00'); -- quán không tồn tại
GO

-- ---------------------------------------------------------------------------
-- 1. Lô 1 bằng MERGE: một câu, nhiều nhánh. Nguồn (USING) nối với đích theo
--    ON; mỗi dòng nguồn rơi vào đúng một nhánh WHEN. OUTPUT $action cho biết
--    nhánh nào đã chạy trên từng dòng. Thứ tự dòng OUTPUT không được bảo đảm
--    ⇒ ghi vào biến bảng rồi SELECT … ORDER BY.
DECLARE @kq TABLE (hanh_dong nvarchar(10), sku varchar(20));

MERGE toinoi.menu_items AS m
USING (SELECT * FROM toinoi.menu_item_imports WHERE batch_id = 1) AS s
   ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN MATCHED THEN
    UPDATE SET name = s.name, price = s.price, is_available = s.is_available,
               source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id
WHEN NOT MATCHED BY TARGET THEN
    INSERT (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
    VALUES (s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id)
OUTPUT $action, inserted.sku INTO @kq;          -- MERGE BẮT BUỘC kết thúc bằng ";"

DECLARE @rc int = @@ROWCOUNT;
SELECT hanh_dong, COUNT(*) AS so_dong FROM @kq GROUP BY hanh_dong ORDER BY hanh_dong;
EXEC lab.expect_value N'MERGE lô 1: @@ROWCOUNT', @rc, N'5';
DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CONCAT(hanh_dong, N' ', sku), N', ') WITHIN GROUP (ORDER BY sku) FROM @kq);
EXEC lab.expect_value N'MERGE lô 1: hành động', @v, N'INSERT BB-01, INSERT BB-02, INSERT BB-03, INSERT NC-01, INSERT TR-01';
GO

-- ---------------------------------------------------------------------------
-- 2. Lô 2, cách ngây thơ: cùng câu MERGE. BB-04 xuất hiện HAI lần trong lô và
--    chưa có trong đích ⇒ cả hai dòng rơi vào nhánh NOT MATCHED ⇒ chèn hai
--    lần cùng khoá ⇒ lỗi 2627 trên khoá chính. Cả câu bị huỷ.
DECLARE @err int, @msg nvarchar(2048);
BEGIN TRY
    MERGE toinoi.menu_items AS m
    USING (SELECT * FROM toinoi.menu_item_imports WHERE batch_id = 2) AS s
       ON m.merchant_id = s.merchant_id AND m.sku = s.sku
    WHEN MATCHED THEN
        UPDATE SET name = s.name, price = s.price, is_available = s.is_available,
                   source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id
    WHEN NOT MATCHED BY TARGET THEN
        INSERT (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
        VALUES (s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id);
END TRY
BEGIN CATCH
    SELECT @err = ERROR_NUMBER(), @msg = ERROR_MESSAGE();
END CATCH;
PRINT CONCAT(N'Msg ', @err, N': ', @msg);
EXEC lab.expect_value N'MERGE lô 2 ngây thơ: số lỗi', @err, N'2627';
DECLARE @co_ten nvarchar(10) = IIF(CHARINDEX(N'pk_menu_items', @msg) > 0, N'có', N'không');
EXEC lab.expect_value N'MERGE lô 2 ngây thơ: thông báo nhắc pk_menu_items', @co_ten, N'có';
GO

-- 3. Khử trùng lặp trong lô (R4) bằng ROW_NUMBER: giữ dòng mới nhất của mỗi
--    (quán, sku). Vẫn LỖI: ON chỉ so khớp theo (merchant_id, sku); dòng 7
--    (BB-05) vi phạm MỘT ràng buộc UNIQUE khác (tên món) — MERGE không "bắt"
--    xung đột đó, nó là lỗi 2627 bình thường.
DECLARE @err int, @msg nvarchar(2048);
BEGIN TRY
    MERGE toinoi.menu_items AS m
    USING (SELECT *
           FROM (SELECT i.*, ROW_NUMBER() OVER (PARTITION BY i.merchant_id, i.sku
                                                ORDER BY i.source_updated_at DESC, i.line_no DESC) AS rn
                 FROM toinoi.menu_item_imports AS i
                 WHERE i.batch_id = 2) AS x
           WHERE x.rn = 1) AS s
       ON m.merchant_id = s.merchant_id AND m.sku = s.sku
    WHEN MATCHED THEN
        UPDATE SET name = s.name, price = s.price, is_available = s.is_available,
                   source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id
    WHEN NOT MATCHED BY TARGET THEN
        INSERT (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
        VALUES (s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id);
END TRY
BEGIN CATCH
    SELECT @err = ERROR_NUMBER(), @msg = ERROR_MESSAGE();
END CATCH;
PRINT CONCAT(N'Msg ', @err, N': ', @msg);
EXEC lab.expect_value N'MERGE lô 2 đã khử trùng lặp: số lỗi', @err, N'2627';
DECLARE @co_ten nvarchar(10) = IIF(CHARINDEX(N'uq_menu_items_name', @msg) > 0, N'có', N'không');
EXEC lab.expect_value N'MERGE lô 2 đã khử trùng lặp: thông báo nhắc uq_menu_items_name', @co_ten, N'có';

-- Bỏ riêng BB-05 ra thì tới lượt dòng 8 (quán không tồn tại): lỗi 547.
SELECT @err = NULL, @msg = NULL;
BEGIN TRY
    MERGE toinoi.menu_items AS m
    USING (SELECT *
           FROM (SELECT i.*, ROW_NUMBER() OVER (PARTITION BY i.merchant_id, i.sku
                                                ORDER BY i.source_updated_at DESC, i.line_no DESC) AS rn
                 FROM toinoi.menu_item_imports AS i
                 WHERE i.batch_id = 2 AND i.sku <> 'BB-05') AS x
           WHERE x.rn = 1) AS s
       ON m.merchant_id = s.merchant_id AND m.sku = s.sku
    WHEN MATCHED THEN
        UPDATE SET name = s.name, price = s.price, is_available = s.is_available,
                   source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id
    WHEN NOT MATCHED BY TARGET THEN
        INSERT (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
        VALUES (s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id);
END TRY
BEGIN CATCH
    SELECT @err = ERROR_NUMBER(), @msg = ERROR_MESSAGE();
END CATCH;
PRINT CONCAT(N'Msg ', @err, N': ', @msg);
EXEC lab.expect_value N'MERGE lô 2 bỏ BB-05: số lỗi', @err, N'547';

-- Ba câu lỗi không để lại gì: mỗi câu là một đơn vị, lỗi thì huỷ cả câu.
DECLARE @n nvarchar(10) = (SELECT COUNT(*) FROM toinoi.menu_items);
EXEC lab.expect_value N'menu_items sau ba câu lỗi', @n, N'5';
GO

-- ---------------------------------------------------------------------------
-- 4. XEM TRƯỚC (dry run): phân loại TỪNG dòng của lô theo R1–R5 trước khi ghi.
--    Đây cũng là "biên bản nhập" gửi lại cho quán. IS DISTINCT FROM có từ
--    SQL Server 2022 (so sánh coi NULL = NULL); ở đây các cột NOT NULL nên
--    `<>` cũng đúng, nhưng thói quen này an toàn khi bảng có cột NULL được.
SELECT i.line_no, i.sku,
       CASE
         WHEN NOT EXISTS (SELECT 1 FROM toinoi.merchants AS q WHERE q.id = i.merchant_id)
           THEN N'TỪ CHỐI — quán không tồn tại (R5)'
         WHEN EXISTS (SELECT 1 FROM toinoi.menu_items AS x
                      WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)
           THEN N'TỪ CHỐI — trùng tên món khác (R5)'
         WHEN ROW_NUMBER() OVER (PARTITION BY i.merchant_id, i.sku
                                 ORDER BY i.source_updated_at DESC, i.line_no DESC) > 1
           THEN N'BỎ QUA — có dòng mới hơn trong cùng lô (R4)'
         WHEN m.sku IS NULL
           THEN N'THÊM'
         WHEN i.source_updated_at <= m.source_updated_at
           THEN N'BỎ QUA — cũ hơn dữ liệu đang có (R2)'
         WHEN i.name IS NOT DISTINCT FROM m.name AND i.price IS NOT DISTINCT FROM m.price
              AND i.is_available IS NOT DISTINCT FROM m.is_available
           THEN N'BỎ QUA — không có gì thay đổi (R3)'
         ELSE N'CẬP NHẬT'
       END AS ket_qua
INTO #xem_truoc
FROM toinoi.menu_item_imports AS i
LEFT JOIN toinoi.menu_items AS m ON m.merchant_id = i.merchant_id AND m.sku = i.sku
WHERE i.batch_id = 2;

SELECT line_no, sku, ket_qua FROM #xem_truoc ORDER BY line_no;

DECLARE @v nvarchar(1000) = (SELECT STRING_AGG(CONCAT(line_no, N' ', ket_qua), N' | ') WITHIN GROUP (ORDER BY line_no) FROM #xem_truoc);
EXEC lab.expect_value N'xem trước lô 2', @v, N'1 CẬP NHẬT | 2 BỎ QUA — không có gì thay đổi (R3) | 3 BỎ QUA — cũ hơn dữ liệu đang có (R2) | 4 CẬP NHẬT | 5 BỎ QUA — có dòng mới hơn trong cùng lô (R4) | 6 THÊM | 7 TỪ CHỐI — trùng tên món khác (R5) | 8 TỪ CHỐI — quán không tồn tại (R5)';
DROP TABLE #xem_truoc;
GO

-- ---------------------------------------------------------------------------
-- 5. MERGE ĐÚNG QUY TẮC (xem trong giao dịch rồi ROLLBACK):
--    * nguồn đã lọc R5 và khử trùng lặp R4;
--    * R2 + R3 nằm ở điều kiện "WHEN MATCHED AND …": dòng không thoả điều
--      kiện KHÔNG bị cập nhật, không có trong OUTPUT, không vào @@ROWCOUNT;
--    * WITH (HOLDLOCK) = khoá SERIALIZABLE trên đích: không có nó, hai phiên
--      cùng MERGE một khoá CHƯA có có thể cùng rơi vào nhánh INSERT và một
--      phiên lỗi 2627 (Phần 4 giải thích).
BEGIN TRANSACTION;

DECLARE @kq TABLE (hanh_dong nvarchar(10), sku varchar(20), gia_cu int, gia_moi int, con_ban bit);

MERGE toinoi.menu_items WITH (HOLDLOCK) AS m
USING (SELECT *
       FROM (SELECT i.*, ROW_NUMBER() OVER (PARTITION BY i.merchant_id, i.sku
                                            ORDER BY i.source_updated_at DESC, i.line_no DESC) AS rn
             FROM toinoi.menu_item_imports AS i
             WHERE i.batch_id = 2
               AND EXISTS (SELECT 1 FROM toinoi.merchants AS q WHERE q.id = i.merchant_id)
               AND NOT EXISTS (SELECT 1 FROM toinoi.menu_items AS x
                               WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)
            ) AS x
       WHERE x.rn = 1) AS s
   ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN MATCHED AND s.source_updated_at > m.source_updated_at                        -- R2
             AND (s.name IS DISTINCT FROM m.name OR s.price IS DISTINCT FROM m.price
                  OR s.is_available IS DISTINCT FROM m.is_available) THEN          -- R3
    UPDATE SET name = s.name, price = s.price, is_available = s.is_available,
               source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id
WHEN NOT MATCHED BY TARGET THEN
    INSERT (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
    VALUES (s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id)
OUTPUT $action, inserted.sku, deleted.price, inserted.price, inserted.is_available INTO @kq;

DECLARE @rc int = @@ROWCOUNT;
SELECT hanh_dong, sku, gia_cu, gia_moi, con_ban FROM @kq ORDER BY sku;
EXEC lab.expect_value N'MERGE đúng quy tắc: @@ROWCOUNT', @rc, N'3';
DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CONCAT(hanh_dong, N' ', sku, N' ', gia_cu, N'→', gia_moi, N' ', con_ban), N', ') WITHIN GROUP (ORDER BY sku) FROM @kq);
EXEC lab.expect_value N'MERGE đúng quy tắc: OUTPUT', @v, N'UPDATE BB-01 55000→59000 1, INSERT BB-04 →52000 1, UPDATE TR-01 5000→5000 0';

ROLLBACK TRANSACTION;

DECLARE @n nvarchar(10) = (SELECT COUNT(*) FROM toinoi.menu_items);
EXEC lab.expect_value N'sau ROLLBACK: menu_items', @n, N'5';
GO

-- ---------------------------------------------------------------------------
-- 6. MẪU AN TOÀN HƠN: UPDATE trước, rồi INSERT … WHERE NOT EXISTS, trong MỘT
--    giao dịch. Mỗi câu làm đúng một việc, đọc và kiểm thử riêng được.
--    * UPDLOCK, SERIALIZABLE (= HOLDLOCK) giữ khoá trên cả khoảng khoá chưa
--      có dòng, nên phiên khác không chèn chen vào giữa UPDATE và INSERT.
--    * XACT_ABORT ON: lỗi giữa chừng ⇒ huỷ cả giao dịch (MS-006).
--    Nguồn chung đặt vào bảng tạm #nguon (R4 + R5 đã lọc) để hai câu dùng lại.
SET XACT_ABORT ON;

SELECT x.merchant_id, x.sku, x.name, x.price, x.is_available, x.source_updated_at, x.batch_id
INTO #nguon
FROM (SELECT i.*, ROW_NUMBER() OVER (PARTITION BY i.merchant_id, i.sku
                                     ORDER BY i.source_updated_at DESC, i.line_no DESC) AS rn
      FROM toinoi.menu_item_imports AS i
      WHERE i.batch_id = 2
        AND EXISTS (SELECT 1 FROM toinoi.merchants AS q WHERE q.id = i.merchant_id)
        AND NOT EXISTS (SELECT 1 FROM toinoi.menu_items AS x
                        WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)
     ) AS x
WHERE x.rn = 1;

DECLARE @so_nguon nvarchar(10) = (SELECT COUNT(*) FROM #nguon);
EXEC lab.expect_value N'nguồn sau R4 + R5', @so_nguon, N'5';

BEGIN TRANSACTION;

UPDATE m WITH (UPDLOCK, SERIALIZABLE)
SET name = s.name, price = s.price, is_available = s.is_available,
    source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id
OUTPUT N'UPDATE' AS hanh_dong, inserted.sku, deleted.price AS gia_cu, inserted.price AS gia_moi
FROM toinoi.menu_items AS m
JOIN #nguon AS s ON s.merchant_id = m.merchant_id AND s.sku = m.sku
WHERE s.source_updated_at > m.source_updated_at                                    -- R2
  AND (s.name IS DISTINCT FROM m.name OR s.price IS DISTINCT FROM m.price
       OR s.is_available IS DISTINCT FROM m.is_available);                         -- R3
DECLARE @so_sua int = @@ROWCOUNT;

INSERT INTO toinoi.menu_items (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
OUTPUT N'INSERT' AS hanh_dong, inserted.sku, inserted.price AS gia_moi
SELECT s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id
FROM #nguon AS s
WHERE NOT EXISTS (SELECT 1 FROM toinoi.menu_items AS m WITH (UPDLOCK, SERIALIZABLE)
                  WHERE m.merchant_id = s.merchant_id AND m.sku = s.sku);
DECLARE @so_them int = @@ROWCOUNT;

COMMIT TRANSACTION;

EXEC lab.expect_value N'mẫu UPDATE rồi INSERT: số dòng cập nhật', @so_sua, N'2';
EXEC lab.expect_value N'mẫu UPDATE rồi INSERT: số dòng thêm', @so_them, N'1';

-- Trạng thái sau lô 2: BB-02 và BB-03 giữ nguyên dữ liệu và số lô 1.
SELECT sku, name, price, is_available, CAST(source_updated_at AS varchar(34)) AS source_updated_at, updated_by_batch
FROM toinoi.menu_items
ORDER BY sku;

DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CONCAT(sku, N' ', price, N' ', is_available, N' lô', updated_by_batch), N', ') WITHIN GROUP (ORDER BY sku) FROM toinoi.menu_items);
EXEC lab.expect_value N'menu_items sau lô 2', @v, N'BB-01 59000 1 lô2, BB-02 45000 1 lô1, BB-03 45000 1 lô1, BB-04 52000 1 lô2, NC-01 25000 1 lô1, TR-01 5000 0 lô2';
GO

-- ---------------------------------------------------------------------------
-- 7. IDEMPOTENT: quán gửi lại lô 2 lần nữa. Cả hai cách đều không đổi gì.
--    a) MERGE đúng quy tắc: 0 dòng.
DECLARE @kq TABLE (hanh_dong nvarchar(10), sku varchar(20));

MERGE toinoi.menu_items WITH (HOLDLOCK) AS m
USING (SELECT *
       FROM (SELECT i.*, ROW_NUMBER() OVER (PARTITION BY i.merchant_id, i.sku
                                            ORDER BY i.source_updated_at DESC, i.line_no DESC) AS rn
             FROM toinoi.menu_item_imports AS i
             WHERE i.batch_id = 2
               AND EXISTS (SELECT 1 FROM toinoi.merchants AS q WHERE q.id = i.merchant_id)
               AND NOT EXISTS (SELECT 1 FROM toinoi.menu_items AS x
                               WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)
            ) AS x
       WHERE x.rn = 1) AS s
   ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN MATCHED AND s.source_updated_at > m.source_updated_at
             AND (s.name IS DISTINCT FROM m.name OR s.price IS DISTINCT FROM m.price
                  OR s.is_available IS DISTINCT FROM m.is_available) THEN
    UPDATE SET name = s.name, price = s.price, is_available = s.is_available,
               source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id
WHEN NOT MATCHED BY TARGET THEN
    INSERT (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
    VALUES (s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id)
OUTPUT $action, inserted.sku INTO @kq;

DECLARE @rc int = @@ROWCOUNT;
EXEC lab.expect_value N'gửi lại lô 2 bằng MERGE: @@ROWCOUNT', @rc, N'0';
DECLARE @n nvarchar(10) = (SELECT COUNT(*) FROM @kq);
EXEC lab.expect_value N'gửi lại lô 2 bằng MERGE: số dòng OUTPUT', @n, N'0';

--    b) UPDATE rồi INSERT: 0 + 0 dòng.
BEGIN TRANSACTION;
UPDATE m WITH (UPDLOCK, SERIALIZABLE)
SET name = s.name, price = s.price, is_available = s.is_available,
    source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id
FROM toinoi.menu_items AS m
JOIN #nguon AS s ON s.merchant_id = m.merchant_id AND s.sku = m.sku
WHERE s.source_updated_at > m.source_updated_at
  AND (s.name IS DISTINCT FROM m.name OR s.price IS DISTINCT FROM m.price
       OR s.is_available IS DISTINCT FROM m.is_available);
DECLARE @so_sua int = @@ROWCOUNT;
INSERT INTO toinoi.menu_items (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id
FROM #nguon AS s
WHERE NOT EXISTS (SELECT 1 FROM toinoi.menu_items AS m WITH (UPDLOCK, SERIALIZABLE)
                  WHERE m.merchant_id = s.merchant_id AND m.sku = s.sku);
DECLARE @so_them int = @@ROWCOUNT;
COMMIT TRANSACTION;
DECLARE @hai nvarchar(20) = CONCAT(@so_sua, N' + ', @so_them);
EXEC lab.expect_value N'gửi lại lô 2 bằng UPDATE + INSERT', @hai, N'0 + 0';

DROP TABLE #nguon;
GO

-- ---------------------------------------------------------------------------
-- 8. Lỗi 8672: gửi lại lô 2 KHÔNG khử trùng lặp, nhánh MATCHED cập nhật vô
--    điều kiện. Lần này BB-04 đã có trong đích ⇒ HAI dòng nguồn khớp CÙNG một
--    dòng đích ⇒ MERGE từ chối cập nhật một dòng hai lần. (Ở mục 2, cùng lỗi
--    dữ liệu đó hiện ra thành 2627 vì BB-04 chưa có.)
DECLARE @err int, @msg nvarchar(2048);
BEGIN TRY
    MERGE toinoi.menu_items AS m
    USING (SELECT i.*
           FROM toinoi.menu_item_imports AS i
           WHERE i.batch_id = 2
             AND EXISTS (SELECT 1 FROM toinoi.merchants AS q WHERE q.id = i.merchant_id)
             AND NOT EXISTS (SELECT 1 FROM toinoi.menu_items AS x
                             WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)) AS s
       ON m.merchant_id = s.merchant_id AND m.sku = s.sku
    WHEN MATCHED THEN
        UPDATE SET name = s.name, price = s.price, is_available = s.is_available,
                   source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id;
END TRY
BEGIN CATCH
    SELECT @err = ERROR_NUMBER(), @msg = ERROR_MESSAGE();
END CATCH;
PRINT CONCAT(N'Msg ', @err, N': ', @msg);
EXEC lab.expect_value N'MERGE nguồn trùng khoá đã có: số lỗi', @err, N'8672';
GO

-- ---------------------------------------------------------------------------
-- 9. WHEN NOT MATCHED BY SOURCE: "đồng bộ trọn thực đơn" — món có trong đích
--    mà không có trong file thì đánh dấu hết hàng. Bẫy: nhánh này xét MỌI dòng
--    của bảng đích, kể cả món của QUÁN KHÁC. Chạy trong giao dịch rồi ROLLBACK.
--    Thêm tạm hai món của quán "Phở Lab 102" để thấy bẫy.
BEGIN TRANSACTION;

INSERT INTO toinoi.menu_items VALUES
    ('e1000000-0000-4000-8000-000000000102', 'PH-01', N'Phở bò tái', 50000, 1, '2026-10-20 08:00 +07:00', 9),
    ('e1000000-0000-4000-8000-000000000102', 'PH-02', N'Phở gà',     45000, 1, '2026-10-20 08:00 +07:00', 9);

-- File thực đơn trọn vẹn của quán 101 = 5 món của lô 2 đã lọc (thiếu NC-01).
SELECT x.merchant_id, x.sku
INTO #thuc_don_101
FROM toinoi.menu_items AS x
WHERE x.merchant_id = 'e1000000-0000-4000-8000-000000000101' AND x.sku <> 'NC-01';

-- a) SAI: không giới hạn quán ⇒ cả hai món của quán 102 bị tắt.
DECLARE @kq TABLE (hanh_dong nvarchar(10), merchant_id uniqueidentifier, sku varchar(20));
MERGE toinoi.menu_items AS m
USING #thuc_don_101 AS s
   ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN NOT MATCHED BY SOURCE THEN
    UPDATE SET is_available = 0
OUTPUT $action, inserted.merchant_id, inserted.sku INTO @kq;

SELECT hanh_dong, RIGHT(CAST(merchant_id AS char(36)), 3) AS quan, sku FROM @kq ORDER BY quan, sku;
DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CONCAT(RIGHT(CAST(merchant_id AS char(36)), 3), N'/', sku), N', ') WITHIN GROUP (ORDER BY merchant_id, sku) FROM @kq);
EXEC lab.expect_value N'BY SOURCE không lọc quán: dòng bị tắt', @v, N'101/NC-01, 102/PH-01, 102/PH-02';

ROLLBACK TRANSACTION;
GO

-- b) ĐÚNG: thêm điều kiện vào chính nhánh WHEN (không đặt vào ON — tài liệu
--    MERGE cảnh báo lọc đích ở ON cho kết quả sai).
BEGIN TRANSACTION;

INSERT INTO toinoi.menu_items VALUES
    ('e1000000-0000-4000-8000-000000000102', 'PH-01', N'Phở bò tái', 50000, 1, '2026-10-20 08:00 +07:00', 9),
    ('e1000000-0000-4000-8000-000000000102', 'PH-02', N'Phở gà',     45000, 1, '2026-10-20 08:00 +07:00', 9);

SELECT x.merchant_id, x.sku
INTO #thuc_don_101
FROM toinoi.menu_items AS x
WHERE x.merchant_id = 'e1000000-0000-4000-8000-000000000101' AND x.sku <> 'NC-01';

DECLARE @kq TABLE (hanh_dong nvarchar(10), merchant_id uniqueidentifier, sku varchar(20));
MERGE toinoi.menu_items AS m
USING #thuc_don_101 AS s
   ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN NOT MATCHED BY SOURCE AND m.merchant_id = 'e1000000-0000-4000-8000-000000000101' THEN
    UPDATE SET is_available = 0
OUTPUT $action, inserted.merchant_id, inserted.sku INTO @kq;

DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CONCAT(RIGHT(CAST(merchant_id AS char(36)), 3), N'/', sku), N', ') WITHIN GROUP (ORDER BY merchant_id, sku) FROM @kq);
EXEC lab.expect_value N'BY SOURCE có điều kiện quán: dòng bị tắt', @v, N'101/NC-01';

ROLLBACK TRANSACTION;
GO

-- ---------------------------------------------------------------------------
-- 10. Cú pháp: MERGE thiếu dấu ";" cuối câu ⇒ lỗi biên dịch 10713 (cả batch
--     không chạy). Câu chạy qua sp_executesql để bắt lỗi.
EXEC lab.expect_error N'MERGE thiếu dấu chấm phẩy',
    N'MERGE toinoi.menu_items AS m
      USING (SELECT CAST(''e1000000-0000-4000-8000-000000000101'' AS uniqueidentifier) AS merchant_id, ''BB-01'' AS sku) AS s
         ON m.merchant_id = s.merchant_id AND m.sku = s.sku
      WHEN MATCHED THEN UPDATE SET price = price',
    10713, N'semi-colon';
GO

-- Đích không đổi sau các mục 8–10.
DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CONCAT(sku, N' ', price, N' ', is_available), N', ') WITHIN GROUP (ORDER BY sku) FROM toinoi.menu_items);
EXEC lab.expect_value N'menu_items không đổi sau mục 8–10', @v, N'BB-01 59000 1, BB-02 45000 1, BB-03 45000 1, BB-04 52000 1, NC-01 25000 1, TR-01 5000 0';
GO

-- ===========================================================================
-- Lời giải bài tập
-- ===========================================================================

-- Bài 1. "Chỉ thêm món mới, món đã có thì để nguyên" (lô 3): MERGE chỉ có
-- nhánh NOT MATCHED BY TARGET. BB-01 (giá 99.000 trong lô) bị bỏ qua.
INSERT INTO toinoi.menu_item_imports VALUES
    (3, 1, 'e1000000-0000-4000-8000-000000000101', 'BB-01', N'Bún bò đặc biệt', 99000, 1, '2026-10-30 08:00 +07:00'),
    (3, 2, 'e1000000-0000-4000-8000-000000000101', 'CF-01', N'Cà phê sữa',      22000, 1, '2026-10-30 08:00 +07:00');

DECLARE @kq TABLE (hanh_dong nvarchar(10), sku varchar(20), price int);
MERGE toinoi.menu_items WITH (HOLDLOCK) AS m
USING (SELECT * FROM toinoi.menu_item_imports WHERE batch_id = 3) AS s
   ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN NOT MATCHED BY TARGET THEN
    INSERT (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
    VALUES (s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id)
OUTPUT $action, inserted.sku, inserted.price INTO @kq;

DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CONCAT(hanh_dong, N' ', sku, N' ', price), N', ') WITHIN GROUP (ORDER BY sku) FROM @kq);
EXEC lab.expect_value N'Bài 1: chỉ thêm món mới', @v, N'INSERT CF-01 22000';
DECLARE @p nvarchar(10) = (SELECT price FROM toinoi.menu_items WHERE sku = 'BB-01');
EXEC lab.expect_value N'Bài 1: BB-01 giữ giá', @p, N'59000';
GO

-- Bài 2. Giới hạn đích bằng CTE thay cho điều kiện ở nhánh WHEN: MERGE vào
-- một CTE lọc theo quán; NOT MATCHED BY SOURCE chỉ còn thấy món của quán 101.
BEGIN TRANSACTION;

INSERT INTO toinoi.menu_items VALUES
    ('e1000000-0000-4000-8000-000000000102', 'PH-01', N'Phở bò tái', 50000, 1, '2026-10-20 08:00 +07:00', 9),
    ('e1000000-0000-4000-8000-000000000102', 'PH-02', N'Phở gà',     45000, 1, '2026-10-20 08:00 +07:00', 9);

DECLARE @kq TABLE (hanh_dong nvarchar(10), merchant_id uniqueidentifier, sku varchar(20));
WITH dich AS (
    SELECT * FROM toinoi.menu_items
    WHERE merchant_id = 'e1000000-0000-4000-8000-000000000101'
)
MERGE dich AS m
USING (SELECT merchant_id, sku FROM toinoi.menu_items
       WHERE merchant_id = 'e1000000-0000-4000-8000-000000000101' AND sku NOT IN ('NC-01', 'CF-01')) AS s
   ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN NOT MATCHED BY SOURCE THEN
    UPDATE SET is_available = 0
OUTPUT $action, inserted.merchant_id, inserted.sku INTO @kq;

DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CONCAT(RIGHT(CAST(merchant_id AS char(36)), 3), N'/', sku), N', ') WITHIN GROUP (ORDER BY merchant_id, sku) FROM @kq);
EXEC lab.expect_value N'Bài 2: MERGE vào CTE lọc quán', @v, N'101/CF-01, 101/NC-01';

ROLLBACK TRANSACTION;
GO

-- Bài 3. Gửi lại lô 2 KHÔNG khử trùng lặp nhưng CÓ điều kiện R2 + R3 ở nhánh
-- MATCHED: có lỗi 8672 không? Không — hai dòng BB-04 (09:05 và 09:30) đều
-- không mới hơn dòng đích (09:30) nên không dòng nào thật sự cập nhật.
DECLARE @err int = 0, @rc int;
BEGIN TRY
    MERGE toinoi.menu_items AS m
    USING (SELECT i.*
           FROM toinoi.menu_item_imports AS i
           WHERE i.batch_id = 2
             AND EXISTS (SELECT 1 FROM toinoi.merchants AS q WHERE q.id = i.merchant_id)
             AND NOT EXISTS (SELECT 1 FROM toinoi.menu_items AS x
                             WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)) AS s
       ON m.merchant_id = s.merchant_id AND m.sku = s.sku
    WHEN MATCHED AND s.source_updated_at > m.source_updated_at
                 AND (s.name IS DISTINCT FROM m.name OR s.price IS DISTINCT FROM m.price
                      OR s.is_available IS DISTINCT FROM m.is_available) THEN
        UPDATE SET name = s.name, price = s.price, is_available = s.is_available,
                   source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id;
    SET @rc = @@ROWCOUNT;
END TRY
BEGIN CATCH
    SET @err = ERROR_NUMBER();
END CATCH;
EXEC lab.expect_value N'Bài 3: lỗi khi có điều kiện', @err, N'0';
EXEC lab.expect_value N'Bài 3: @@ROWCOUNT', @rc, N'0';
GO
