-- P02 · 18-phan-trang-keyset.sql — MS-018 `mssql-phan-trang-keyset`
-- Phân trang lịch sử chuyến: OFFSET/FETCH và keyset — xử lý KHOÁ SẮP XẾP
-- TRÙNG và chứng minh điều kiện trang kế tiếp.
--
-- Màn "Lịch sử chuyến" của khách doanh nghiệp "Cửa hàng Phụ Kiện Lab" (KHDN,
-- cb000000-…-000000000001): 18 chuyến bưu kiện, mới nhất trước, 5 chuyến một
-- trang. Hai lô đặt qua API có CÙNG giờ đặt đến từng giây:
--   10/10 09:00:00 — 6 chuyến (100971…100976) · 18/10 14:30:00 — 4 chuyến (101882…101885)
-- Cả hai lô đều nằm VẮT qua ranh giới trang.
--
-- Keyset (seek) pagination: thay vì "bỏ qua N dòng" (OFFSET), trang sau bắt
-- đầu NGAY SAU dòng cuối của trang trước, theo một khoá sắp xếp DUY NHẤT.
-- requested_at không duy nhất ⇒ khoá là (requested_at, code).
--
-- File chỉ đọc, trừ mục 7 (chèn một chuyến trong giao dịch rồi ROLLBACK).

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

-- ---------------------------------------------------------------------------
-- 1. Bảng đối chiếu: toàn bộ lịch sử theo thứ tự hiển thị, kèm vị trí và trang.
SELECT ROW_NUMBER() OVER (ORDER BY requested_at DESC, code DESC)              AS vi_tri,
       (ROW_NUMBER() OVER (ORDER BY requested_at DESC, code DESC) - 1) / 5 + 1 AS trang,
       code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY requested_at DESC, code DESC;

DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY requested_at DESC, code DESC)
                            FROM toinoi.trips WHERE customer_id = 'cb000000-0000-4000-8000-000000000001');
EXEC lab.expect_value N'thứ tự 18 chuyến', @v,
     N'102897,102611,102149,101885,101884,101883,101882,101555,101224,100976,100975,100974,100973,100972,100971,100669,100500,100130';
GO

-- ---------------------------------------------------------------------------
-- 2. OFFSET/FETCH: trang p = OFFSET (p-1)*5 ROWS FETCH NEXT 5 ROWS ONLY.
--    Đúng khi dữ liệu đứng yên VÀ ORDER BY có tie-breaker (MS-007).
SELECT code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY requested_at DESC, code DESC
OFFSET 5 ROWS FETCH NEXT 5 ROWS ONLY;

DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY requested_at DESC, code DESC)
    FROM (SELECT code, requested_at FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
          ORDER BY requested_at DESC, code DESC
          OFFSET 5 ROWS FETCH NEXT 5 ROWS ONLY) AS t);
EXEC lab.expect_value N'OFFSET 5: trang 2', @v, N'101883,101882,101555,101224,100976';
GO

-- ---------------------------------------------------------------------------
-- 3. Keyset, trang 1. Ứng dụng giữ lại "con trỏ" = khoá của dòng CUỐI trang:
--    (2026-10-18 14:30:00 +07:00, 101884).
SELECT TOP (5) code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY requested_at DESC, code DESC;
GO

-- 4. Trang 2 — SAI kiểu 1: con trỏ chỉ có requested_at, so sánh `<`.
--    Hai chuyến còn lại của lô 18/10 (101883, 101882) bị BỎ MẤT vĩnh viễn.
DECLARE @con_tro_at datetimeoffset(3) = '2026-10-18 14:30:00 +07:00';

SELECT TOP (5) code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND requested_at < @con_tro_at
ORDER BY requested_at DESC, code DESC;

DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY requested_at DESC, code DESC)
    FROM (SELECT TOP (5) code, requested_at FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001' AND requested_at < @con_tro_at
          ORDER BY requested_at DESC, code DESC) AS t);
EXEC lab.expect_value N'con trỏ một cột, <', @v, N'101555,101224,100976,100975,100974';

-- Trang 2 — SAI kiểu 2: `<=` thì HIỆN LẠI 101885, 101884 của trang 1. Nếu một
-- lô cùng giây dài hơn cỡ trang, `<=` trả mãi cùng một trang.
SET @v = (SELECT STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY requested_at DESC, code DESC)
    FROM (SELECT TOP (5) code, requested_at FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001' AND requested_at <= @con_tro_at
          ORDER BY requested_at DESC, code DESC) AS t);
EXEC lab.expect_value N'con trỏ một cột, <=', @v, N'101885,101884,101883,101882,101555';
GO

-- ---------------------------------------------------------------------------
--    comparison) `(a, b) < (x, y)` như PostgreSQL: câu đó bị từ chối với lỗi 4145.
--    comparison) `(a, b) < (x, y)` như PostgreSQL: câu đó là lỗi cú pháp 102.
EXEC lab.expect_error N'so sánh bộ (requested_at, code) < (…)',
    N'SELECT code FROM toinoi.trips
      WHERE (requested_at, code) < (CAST(''2026-10-18 14:30:00 +07:00'' AS datetimeoffset(3)), 101884)',
    4145, N'non-boolean type';

--    Viết tách theo đúng định nghĩa thứ tự từ điển:
--    (a, b) < (x, y)  ⇔  a < x  OR  (a = x AND b < y)
DECLARE @con_tro_at datetimeoffset(3) = '2026-10-18 14:30:00 +07:00',
        @con_tro_code bigint = 101884;

SELECT TOP (5) code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at < @con_tro_at
       OR (requested_at = @con_tro_at AND code < @con_tro_code))
ORDER BY requested_at DESC, code DESC;

DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY requested_at DESC, code DESC)
    FROM (SELECT TOP (5) code, requested_at FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
            AND (requested_at < @con_tro_at OR (requested_at = @con_tro_at AND code < @con_tro_code))
          ORDER BY requested_at DESC, code DESC) AS t);
EXEC lab.expect_value N'keyset trang 2', @v, N'101883,101882,101555,101224,100976';

-- Trang 3 từ con trỏ (2026-10-10 09:00:00 +07:00, 100976): bắt đầu đúng ở
-- giữa lô 10/10.
SELECT @con_tro_at = '2026-10-10 09:00:00 +07:00', @con_tro_code = 100976;
SET @v = (SELECT STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY requested_at DESC, code DESC)
    FROM (SELECT TOP (5) code, requested_at FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
            AND (requested_at < @con_tro_at OR (requested_at = @con_tro_at AND code < @con_tro_code))
          ORDER BY requested_at DESC, code DESC) AS t);
EXEC lab.expect_value N'keyset trang 3', @v, N'100975,100974,100973,100972,100971';
GO

-- ---------------------------------------------------------------------------
-- 6. Hai cột sắp NGƯỢC CHIỀU (giờ đặt mới trước, nhưng trong cùng giây mã
--    NHỎ trước): dấu so sánh của mỗi cột theo chiều của chính cột đó.
--    Con trỏ: dòng cuối trang 1 theo thứ tự này là (2026-10-18 14:30, 101883).
DECLARE @con_tro_at datetimeoffset(3) = '2026-10-18 14:30:00 +07:00',
        @con_tro_code bigint = 101883;

DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY requested_at DESC, code ASC)
    FROM (SELECT TOP (5) code, requested_at FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
          ORDER BY requested_at DESC, code ASC) AS t);
EXEC lab.expect_value N'ngược chiều: trang 1', @v, N'102897,102611,102149,101882,101883';

SELECT TOP (5) code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at < @con_tro_at                                  -- DESC ⇒ <
       OR (requested_at = @con_tro_at AND code > @con_tro_code))   -- ASC  ⇒ >
ORDER BY requested_at DESC, code ASC;

SET @v = (SELECT STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY requested_at DESC, code ASC)
    FROM (SELECT TOP (5) code, requested_at FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
            AND (requested_at < @con_tro_at OR (requested_at = @con_tro_at AND code > @con_tro_code))
          ORDER BY requested_at DESC, code ASC) AS t);
EXEC lab.expect_value N'ngược chiều: trang 2', @v, N'101884,101885,101555,101224,100971';
GO

-- ---------------------------------------------------------------------------
-- 7. CHỨNG MINH điều kiện trang kế tiếp: đi hết mọi trang bằng vòng lặp, mỗi
--    bước lấy 5 dòng sau con trỏ của bước trước. Đúng khi: hợp các trang =
--    toàn bộ lịch sử, KHÔNG dòng nào lặp, không dòng nào thiếu.
--    Không viết được bằng recursive CTE như lab PostgreSQL: phần đệ quy của
--    SQL Server cấm TOP/OFFSET (lỗi 461) — MS-019 giải thích recursive CTE.
EXEC lab.expect_error N'TOP trong phần đệ quy của CTE',
    N'WITH trang AS (
          SELECT 1 AS so_trang, CAST(0 AS bigint) AS code
          UNION ALL
          SELECT tr.so_trang + 1, nx.code
          FROM trang AS tr
          CROSS APPLY (SELECT TOP (5) t.code FROM toinoi.trips AS t
                       WHERE t.code > tr.code ORDER BY t.code) AS nx)
      SELECT * FROM trang',
    461, N'recursive part';

DECLARE @trang TABLE (so_trang int, vi_tri int, code bigint, requested_at datetimeoffset(3));
DECLARE @so_trang int = 0, @n int = 1,
        @con_tro_at datetimeoffset(3) = NULL, @con_tro_code bigint = NULL;

WHILE @n > 0
BEGIN
    SET @so_trang += 1;

    INSERT INTO @trang (so_trang, vi_tri, code, requested_at)
    SELECT TOP (5) @so_trang, ROW_NUMBER() OVER (ORDER BY requested_at DESC, code DESC), code, requested_at
    FROM toinoi.trips
    WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
      AND (@con_tro_at IS NULL                                     -- trang 1: chưa có con trỏ
           OR requested_at < @con_tro_at
           OR (requested_at = @con_tro_at AND code < @con_tro_code))
    ORDER BY requested_at DESC, code DESC;
    SET @n = @@ROWCOUNT;

    -- Con trỏ mới = dòng cuối của trang vừa lấy.
    SELECT TOP (1) @con_tro_at = requested_at, @con_tro_code = code
    FROM @trang WHERE so_trang = @so_trang
    ORDER BY vi_tri DESC;
END;

SELECT so_trang, STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY vi_tri) AS ma
FROM @trang
GROUP BY so_trang
ORDER BY so_trang;

DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CONCAT(so_trang, N':', so_dong), N' ') WITHIN GROUP (ORDER BY so_trang)
                            FROM (SELECT so_trang, COUNT(*) AS so_dong FROM @trang GROUP BY so_trang) AS g);
EXEC lab.expect_value N'số dòng mỗi trang', @v, N'1:5 2:5 3:5 4:3';

-- Bằng chứng: 18 mã qua các trang, 18 mã khác nhau, đúng bằng số chuyến, và
-- thứ tự ghép lại trùng thứ tự của bảng đối chiếu (mục 1).
SELECT (SELECT COUNT(*) FROM @trang)                                          AS tong_ma_qua_cac_trang,
       (SELECT COUNT(DISTINCT code) FROM @trang)                              AS ma_khac_nhau,
       (SELECT COUNT(*) FROM toinoi.trips
        WHERE customer_id = 'cb000000-0000-4000-8000-000000000001')           AS so_chuyen;

SET @v = (SELECT CONCAT(COUNT(*), N'/', COUNT(DISTINCT code)) FROM @trang);
EXEC lab.expect_value N'tổng mã / mã khác nhau', @v, N'18/18';

SET @v = (SELECT COUNT(*)
          FROM (SELECT code, ROW_NUMBER() OVER (ORDER BY so_trang, vi_tri) AS thu_tu FROM @trang) AS p
          FULL JOIN (SELECT code, ROW_NUMBER() OVER (ORDER BY requested_at DESC, code DESC) AS thu_tu
                     FROM toinoi.trips
                     WHERE customer_id = 'cb000000-0000-4000-8000-000000000001') AS d
            ON d.thu_tu = p.thu_tu AND d.code = p.code
          WHERE p.code IS NULL OR d.code IS NULL);
EXEC lab.expect_value N'dòng lệch giữa ghép trang và bảng đối chiếu', @v, N'0';
GO

-- Cùng vòng lặp với hai con trỏ SAI của mục 4 (chặn tối đa 8 trang):
--   `<` một cột: dừng sau 3 trang, chỉ gom được 13 mã (mất 5);
--   `<=` một cột: lô 10/10 (6 chuyến) dài hơn cỡ trang ⇒ từ trang 4 trả mãi
--   cùng 5 mã, vòng lặp chỉ dừng nhờ chốt 8 trang (14 mã, thiếu 4 chuyến cũ nhất).
DECLARE @trang TABLE (cach char(2), so_trang int, vi_tri int, code bigint, requested_at datetimeoffset(3));
DECLARE @cach char(2), @so_trang int, @n int, @con_tro_at datetimeoffset(3);
DECLARE c CURSOR LOCAL FAST_FORWARD FOR SELECT cach FROM (VALUES ('<'), ('<=')) AS v(cach);
OPEN c;
FETCH NEXT FROM c INTO @cach;
WHILE @@FETCH_STATUS = 0
BEGIN
    SELECT @so_trang = 0, @n = 1, @con_tro_at = NULL;
    WHILE @n > 0 AND @so_trang < 8
    BEGIN
        SET @so_trang += 1;
        INSERT INTO @trang (cach, so_trang, vi_tri, code, requested_at)
        SELECT TOP (5) @cach, @so_trang, ROW_NUMBER() OVER (ORDER BY requested_at DESC, code DESC), code, requested_at
        FROM toinoi.trips
        WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
          AND (@con_tro_at IS NULL
               OR (@cach = '<'  AND requested_at <  @con_tro_at)
               OR (@cach = '<=' AND requested_at <= @con_tro_at))
        ORDER BY requested_at DESC, code DESC;
        SET @n = @@ROWCOUNT;
        SELECT TOP (1) @con_tro_at = requested_at FROM @trang
        WHERE cach = @cach AND so_trang = @so_trang ORDER BY vi_tri DESC;
    END;
    FETCH NEXT FROM c INTO @cach;
END;
CLOSE c;
DEALLOCATE c;

DECLARE @v nvarchar(400) = (SELECT CONCAT(COUNT(DISTINCT so_trang), N' trang có dòng, ', COUNT(DISTINCT code), N' mã khác nhau')
                            FROM @trang WHERE cach = '<');
EXEC lab.expect_value N'con trỏ một cột < qua mọi trang', @v, N'3 trang có dòng, 13 mã khác nhau';
SET @v = (SELECT CONCAT(COUNT(DISTINCT so_trang), N' trang, ', COUNT(*), N' dòng, ', COUNT(DISTINCT code), N' mã khác nhau')
          FROM @trang WHERE cach = '<=');
EXEC lab.expect_value N'con trỏ một cột <= (chốt 8 trang)', @v, N'8 trang, 40 dòng, 14 mã khác nhau';
SET @v = (SELECT STRING_AGG(CONCAT(so_trang, N':', ma), N' ') WITHIN GROUP (ORDER BY so_trang)
          FROM (SELECT so_trang, STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY vi_tri) AS ma
                FROM @trang WHERE cach = '<=' AND so_trang >= 4 GROUP BY so_trang) AS g);
EXEC lab.expect_value N'con trỏ <=: trang 4 … 8', @v,
     N'4:100976,100975,100974,100973,100972 5:100976,100975,100974,100973,100972 6:100976,100975,100974,100973,100972 7:100976,100975,100974,100973,100972 8:100976,100975,100974,100973,100972';

-- Thiếu ngoặc quanh khối OR: nhánh thứ hai không còn điều kiện khách hàng.
-- Trên fixture này kết quả TÌNH CỜ vẫn đúng 13 dòng (không khách nào khác đặt
-- đúng 18/10 14:30:00) — test trên dữ liệu này không bắt được lỗi đó.
DECLARE @con_tro_code bigint = 101884;
SET @con_tro_at = '2026-10-18 14:30:00 +07:00';
SET @v = (SELECT COUNT(*) FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
            AND requested_at < @con_tro_at
             OR (requested_at = @con_tro_at AND code < @con_tro_code));
EXEC lab.expect_value N'thiếu ngoặc: số dòng sau con trỏ', @v, N'13';
SET @v = (SELECT COUNT(*) FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
            AND (requested_at < @con_tro_at OR (requested_at = @con_tro_at AND code < @con_tro_code)));
EXEC lab.expect_value N'đủ ngoặc: số dòng sau con trỏ', @v, N'13';
GO

-- ---------------------------------------------------------------------------
-- 8. OFFSET trôi khi dữ liệu thay đổi giữa hai lần tải trang. Khách xem trang
--    1, trong lúc đó đặt thêm một chuyến; trang 2 bằng OFFSET 5 lặp lại
--    101884 (vừa thấy ở trang 1). Keyset không bị ảnh hưởng.
BEGIN TRANSACTION;

DECLARE @moi TABLE (code bigint);
INSERT INTO toinoi.trips (kind, city_id, customer_id, status, fare, requested_at, updated_at)
OUTPUT inserted.code INTO @moi
VALUES ('parcel', 1, 'cb000000-0000-4000-8000-000000000001', 'requested', 30000,
        '2026-10-31 00:59:00 +07:00', '2026-10-31 00:59:00 +07:00');
SELECT code AS chuyen_moi FROM @moi;
DECLARE @v nvarchar(400) = (SELECT CAST(code AS varchar(10)) FROM @moi);
EXEC lab.expect_value N'mã chuyến mới (trong giao dịch)', @v, N'103174';

SELECT code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY requested_at DESC, code DESC
OFFSET 5 ROWS FETCH NEXT 5 ROWS ONLY;

SET @v = (SELECT STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY requested_at DESC, code DESC)
    FROM (SELECT code, requested_at FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
          ORDER BY requested_at DESC, code DESC
          OFFSET 5 ROWS FETCH NEXT 5 ROWS ONLY) AS t);
EXEC lab.expect_value N'OFFSET 5 sau khi có chuyến mới', @v, N'101884,101883,101882,101555,101224';

DECLARE @con_tro_at datetimeoffset(3) = '2026-10-18 14:30:00 +07:00', @con_tro_code bigint = 101884;
SET @v = (SELECT STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY requested_at DESC, code DESC)
    FROM (SELECT TOP (5) code, requested_at FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
            AND (requested_at < @con_tro_at OR (requested_at = @con_tro_at AND code < @con_tro_code))
          ORDER BY requested_at DESC, code DESC) AS t);
EXEC lab.expect_value N'keyset sau khi có chuyến mới', @v, N'101883,101882,101555,101224,100976';

ROLLBACK TRANSACTION;

-- ROLLBACK huỷ dòng nhưng KHÔNG trả lại giá trị IDENTITY đã cấp (MS-006).
-- Lab đặt lại để các file sau thấy đúng trạng thái của fixture; ĐỪNG làm vậy
-- trên hệ thống thật.
DECLARE @id nvarchar(20) = CAST(IDENT_CURRENT('toinoi.trips') AS nvarchar(20));
EXEC lab.expect_value N'IDENT_CURRENT sau ROLLBACK', @id, N'103174';
DBCC CHECKIDENT ('toinoi.trips', RESEED, 103173) WITH NO_INFOMSGS;
SET @id = CAST(IDENT_CURRENT('toinoi.trips') AS nvarchar(20));
EXEC lab.expect_value N'IDENT_CURRENT sau khi đặt lại', @id, N'103173';

SET @v = (SELECT COUNT(*) FROM toinoi.trips WHERE customer_id = 'cb000000-0000-4000-8000-000000000001');
EXEC lab.expect_value N'số chuyến KHDN sau ROLLBACK', @v, N'18';
GO

-- ===========================================================================
-- Lời giải bài tập
-- ===========================================================================

-- Bài 1. Nút "Trang trước": đang ở trang 3 (dòng đầu = 100975 lúc 10/10
-- 09:00). Lấy 5 dòng ĐỨNG TRƯỚC con trỏ: đảo dấu và đảo chiều sắp xếp, rồi
-- sắp lại theo thứ tự hiển thị.
DECLARE @dau_at datetimeoffset(3) = '2026-10-10 09:00:00 +07:00', @dau_code bigint = 100975;
DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY requested_at DESC, code DESC)
    FROM (SELECT TOP (5) code, requested_at FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
            AND (requested_at > @dau_at OR (requested_at = @dau_at AND code > @dau_code))
          ORDER BY requested_at ASC, code ASC) AS t);
EXEC lab.expect_value N'Bài 1: trang trước của trang 3', @v, N'101883,101882,101555,101224,100976';
GO

-- Bài 2. "Còn trang sau không?" — lấy TOP (6): có dòng thứ 6 thì còn, hiển
-- thị 5 dòng đầu. Trang 3 (con trỏ 100976) còn; trang 4 (con trỏ 100971) hết.
DECLARE @con_tro_at datetimeoffset(3) = '2026-10-10 09:00:00 +07:00', @con_tro_code bigint = 100976;
DECLARE @n3 int = (SELECT COUNT(*) FROM (SELECT TOP (6) code FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
            AND (requested_at < @con_tro_at OR (requested_at = @con_tro_at AND code < @con_tro_code))
          ORDER BY requested_at DESC, code DESC) AS t);
SELECT @con_tro_code = 100971;
DECLARE @n4 int = (SELECT COUNT(*) FROM (SELECT TOP (6) code FROM toinoi.trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
            AND (requested_at < @con_tro_at OR (requested_at = @con_tro_at AND code < @con_tro_code))
          ORDER BY requested_at DESC, code DESC) AS t);
DECLARE @v nvarchar(20) = CONCAT(@n3, N'/', @n4);
EXEC lab.expect_value N'Bài 2: TOP (6) ở trang 3 / trang 4', @v, N'6/3';
GO

-- Bài 3. Chứng minh cho thứ tự NGƯỢC CHIỀU (requested_at DESC, code ASC):
-- cùng vòng lặp, đổi dấu so sánh của code.
DECLARE @trang TABLE (so_trang int, vi_tri int, code bigint, requested_at datetimeoffset(3));
DECLARE @so_trang int = 0, @n int = 1,
        @con_tro_at datetimeoffset(3) = NULL, @con_tro_code bigint = NULL;
WHILE @n > 0
BEGIN
    SET @so_trang += 1;
    INSERT INTO @trang (so_trang, vi_tri, code, requested_at)
    SELECT TOP (5) @so_trang, ROW_NUMBER() OVER (ORDER BY requested_at DESC, code ASC), code, requested_at
    FROM toinoi.trips
    WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
      AND (@con_tro_at IS NULL
           OR requested_at < @con_tro_at
           OR (requested_at = @con_tro_at AND code > @con_tro_code))
    ORDER BY requested_at DESC, code ASC;
    SET @n = @@ROWCOUNT;
    SELECT TOP (1) @con_tro_at = requested_at, @con_tro_code = code
    FROM @trang WHERE so_trang = @so_trang ORDER BY vi_tri DESC;
END;
DECLARE @v nvarchar(400) = (SELECT STRING_AGG(CAST(code AS varchar(10)), ',') WITHIN GROUP (ORDER BY so_trang, vi_tri) FROM @trang);
EXEC lab.expect_value N'Bài 3: ghép trang ngược chiều', @v,
     N'102897,102611,102149,101882,101883,101884,101885,101555,101224,100971,100972,100973,100974,100975,100976,100669,100500,100130';
SET @v = (SELECT CONCAT(COUNT(*), N'/', COUNT(DISTINCT code)) FROM @trang);
EXEC lab.expect_value N'Bài 3: tổng mã / mã khác nhau', @v, N'18/18';
GO
