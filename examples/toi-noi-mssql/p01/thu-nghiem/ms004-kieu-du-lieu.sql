-- Thí nghiệm MS-004 · kiểu dữ liệu cho chuỗi tiếng Việt, tiền và thời gian.
-- Độc lập: chỉ cần database ToiNoi (sau 00-reset). Không tạo bảng nghiệp vụ.
--   node run.mjs p01/00-reset.sql p01/thu-nghiem/ms004-kieu-du-lieu.sql

SET NOCOUNT ON;
USE ToiNoi;
GO

-- ------------------------------------------------ 1. Thiếu tiền tố N'…'
-- Database dùng Vietnamese_100_CI_AS (code page 1258 cho varchar). Chuỗi
-- không có N là varchar: chữ có dấu nào code page 1258 không biểu diễn được
-- ở dạng dựng sẵn sẽ bị đổi — thường thành '?'.
DECLARE @t TABLE (nhan varchar(30), v_varchar varchar(40), v_nvarchar nvarchar(40));
INSERT INTO @t VALUES
    ('literal có N',   N'Nguyễn Văn An — Đà Nẵng',  N'Nguyễn Văn An — Đà Nẵng'),
    ('literal không N', 'Nguyễn Văn An — Đà Nẵng',   'Nguyễn Văn An — Đà Nẵng');
SELECT nhan, v_varchar, v_nvarchar,
       DATALENGTH(v_nvarchar) AS byte_nvarchar, LEN(v_nvarchar) AS ky_tu
FROM @t;
GO

-- ------------------------------------------ 2. UTF-8 collation (SQL 2019+)
-- Collation _UTF8 cho phép varchar lưu Unicode bằng UTF-8: chữ ASCII 1 byte,
-- chữ có dấu 2–3 byte. nvarchar (UTF-16) luôn 2 byte/ký tự cơ bản.
DECLARE @s nvarchar(40) = N'Nguyễn Văn An';
SELECT DATALENGTH(@s)                                                           AS byte_utf16,
       DATALENGTH(CAST(@s COLLATE Vietnamese_100_CI_AS_SC_UTF8 AS varchar(40))) AS byte_utf8,
       CAST(@s COLLATE Vietnamese_100_CI_AS_SC_UTF8 AS varchar(40))             AS gia_tri_utf8;
GO

-- -------------------------------------------------- 3. Tiền: int vs money vs float
-- Chia đều 100.000 đ cho 3 người: money giữ 4 chữ số lẻ, phép nhân lại không
-- về 100.000; float cộng 0.1 mười lần không ra 1.
DECLARE @m money = 100000, @f float = 0, @i int = 0;
WHILE @i < 10 BEGIN SET @f += 0.1; SET @i += 1; END;
SELECT @m / 3                        AS money_chia_3,
       (@m / 3) * 3                  AS money_nhan_lai,
       100000 / 3                    AS int_chia_3,
       100000 % 3                    AS int_du,
       IIF(@f = 1.0, N'bằng', N'KHÁC') AS float_10_lan_0_1_so_voi_1;
GO

-- SUM(int) trả về int ⇒ có thể tràn khi cộng nhiều; ép bigint trước khi cộng.
DECLARE @x TABLE (fare int);
INSERT INTO @x VALUES (2000000000), (2000000000);
BEGIN TRY
    SELECT SUM(fare) AS tong_int FROM @x;
END TRY
BEGIN CATCH
    PRINT CONCAT(N'PASS SUM(int) tràn — lỗi ', ERROR_NUMBER(), N': ', ERROR_MESSAGE());
END CATCH;
SELECT SUM(CAST(fare AS bigint)) AS tong_bigint FROM @x;
GO

-- ------------------------------------------- 4. Thời gian và múi giờ
-- datetime: làm tròn tới bội số 1/300 giây (.000/.003/.007).
-- datetime2(3)/datetimeoffset(3): chính xác tới mili giây.
SELECT CAST('2026-11-03 09:00:00.001' AS datetime)       AS datetime_lam_tron,
       CAST('2026-11-03 09:00:00.001' AS datetime2(3))   AS datetime2_3;

-- Cùng một khoảnh khắc ở hai độ lệch: bằng nhau khi so sánh.
DECLARE @vn datetimeoffset(3) = '2026-11-03 09:00:00 +07:00',
        @utc datetimeoffset(3) = '2026-11-03 02:00:00 +00:00';
SELECT IIF(@vn = @utc, N'bằng', N'khác') AS so_sanh,
       CAST(SWITCHOFFSET(@vn, '+00:00') AS varchar(34)) AS doi_sang_utc,
       CAST(@utc AT TIME ZONE 'SE Asia Standard Time' AS varchar(34)) AS doi_sang_gio_vn;

-- datetime2 KHÔNG mang múi giờ: AT TIME ZONE chỉ "gắn" độ lệch, không đổi giờ.
DECLARE @khong_mui datetime2(0) = '2026-11-03 09:00:00';
SELECT CAST(@khong_mui AT TIME ZONE 'SE Asia Standard Time' AS varchar(34)) AS gan_gio_vn,
       CAST(@khong_mui AT TIME ZONE 'UTC' AS varchar(34))                   AS gan_utc;

-- Tên múi giờ là tên Windows, kể cả khi SQL Server chạy trên Linux.
SELECT name, current_utc_offset FROM sys.time_zone_info
WHERE name IN (N'SE Asia Standard Time', N'UTC') ORDER BY name;
GO

-- ------------------------------------------------- 5. GUID: NEWID vs NEWSEQUENTIALID
-- NEWSEQUENTIALID chỉ dùng được trong DEFAULT của cột.
DECLARE @g TABLE (n int IDENTITY, id uniqueidentifier DEFAULT NEWSEQUENTIALID());
INSERT INTO @g DEFAULT VALUES; INSERT INTO @g DEFAULT VALUES; INSERT INTO @g DEFAULT VALUES;
-- Thứ tự sắp theo uniqueidentifier trùng thứ tự chèn ⇒ không chèn vào giữa index.
SELECT IIF(COUNT(*) = 3, N'PASS', N'FAIL') + N' NEWSEQUENTIALID tăng dần theo thứ tự chèn' AS ket_qua
FROM (SELECT n, ROW_NUMBER() OVER (ORDER BY id) AS rn FROM @g) AS x WHERE n = rn;
BEGIN TRY
    EXEC (N'SELECT NEWSEQUENTIALID();');
END TRY
BEGIN CATCH
    PRINT CONCAT(N'PASS NEWSEQUENTIALID ngoài DEFAULT — lỗi ', ERROR_NUMBER(), N': ', ERROR_MESSAGE());
END CATCH;
GO
