-- P01 · 00-reset.sql — dựng lại database lab ToiNoi từ đầu (MS-001, MS-002).
--
-- Chạy được nhiều lần: nếu database đã có thì đóng mọi phiên đang dùng nó
-- (kể cả tab DBStudio đang mở) rồi xoá. CHỈ dùng trên lab — câu
-- `ROLLBACK IMMEDIATE` huỷ ngang giao dịch của người khác.
--
-- Quy ước của lab:
--   * database `ToiNoi`, collation `Vietnamese_100_CI_AS` (so sánh và sắp xếp
--     tiếng Việt, không phân biệt hoa thường, phân biệt dấu) — MS-004 giải thích;
--   * schema `toinoi` chứa bảng nghiệp vụ, schema `lab` chứa thủ tục kiểm tra
--     của lab (không phải một phần của Tới Nơi);
--   * mọi kiểm tra in một dòng bắt đầu bằng `PASS` hoặc `FAIL`; runner đếm
--     các dòng đó.

SET NOCOUNT ON;
USE master;
GO

IF DB_ID(N'ToiNoi') IS NOT NULL
BEGIN
    ALTER DATABASE ToiNoi SET SINGLE_USER WITH ROLLBACK IMMEDIATE;
    DROP DATABASE ToiNoi;
    PRINT N'Đã xoá database ToiNoi cũ';
END;
GO

CREATE DATABASE ToiNoi COLLATE Vietnamese_100_CI_AS;
GO

USE ToiNoi;
GO

CREATE SCHEMA toinoi AUTHORIZATION dbo;
GO
CREATE SCHEMA lab AUTHORIZATION dbo;
GO

-- ---------------------------------------------------------------------------
-- lab.expect_error: chạy một câu lệnh mà ta MONG nó lỗi.
--   PASS khi lỗi đúng số hiệu (và, nếu có, thông báo nhắc đúng tên ràng buộc);
--   FAIL khi câu lệnh chạy thành công hoặc lỗi khác.
-- Câu lệnh chạy trong một transaction riêng và luôn được ROLLBACK, nên dữ liệu
-- không đổi dù câu lệnh có lỡ thành công.
CREATE PROCEDURE lab.expect_error
    @label    nvarchar(200),
    @sql      nvarchar(max),
    @number   int,
    @contains nvarchar(200) = NULL
AS
BEGIN
    SET NOCOUNT ON;
    DECLARE @err int, @msg nvarchar(2048);
    BEGIN TRY
        BEGIN TRANSACTION;
        EXEC sys.sp_executesql @sql;
        ROLLBACK TRANSACTION;
        PRINT CONCAT(N'FAIL ', @label, N' — câu lệnh KHÔNG lỗi, mong đợi lỗi ', @number);
        RETURN;
    END TRY
    BEGIN CATCH
        SELECT @err = ERROR_NUMBER(), @msg = ERROR_MESSAGE();
        IF XACT_STATE() <> 0 ROLLBACK TRANSACTION;
    END CATCH;

    IF @err = @number AND (@contains IS NULL OR CHARINDEX(@contains, @msg) > 0)
        PRINT CONCAT(N'PASS ', @label, N' — lỗi ', @err, N': ', @msg);
    ELSE
        PRINT CONCAT(N'FAIL ', @label, N' — lỗi ', @err, N' (mong đợi ', @number,
                     COALESCE(N' + "' + @contains + N'"', N''), N'): ', @msg);
END;
GO

-- lab.expect_value: so một giá trị thực tế với giá trị mong đợi (dạng chuỗi).
CREATE PROCEDURE lab.expect_value
    @label    nvarchar(200),
    @actual   nvarchar(4000),
    @expected nvarchar(4000)
AS
BEGIN
    SET NOCOUNT ON;
    IF (@actual = @expected) OR (@actual IS NULL AND @expected IS NULL)
        PRINT CONCAT(N'PASS ', @label, N' = ', COALESCE(@actual, N'NULL'));
    ELSE
        PRINT CONCAT(N'FAIL ', @label, N' = ', COALESCE(@actual, N'NULL'),
                     N' (mong đợi ', COALESCE(@expected, N'NULL'), N')');
END;
GO

-- Bốn lớp: instance → database → schema → đối tượng. Phiên này đang ở đâu?
SELECT DB_NAME()                                   AS database_hien_tai,
       SUSER_SNAME()                               AS login,
       USER_NAME()                                 AS user_trong_db,
       SCHEMA_NAME()                               AS schema_mac_dinh,
       CONVERT(nvarchar(128), DATABASEPROPERTYEX(N'ToiNoi', 'Collation')) AS collation_db;

SELECT CONVERT(nvarchar(20), SERVERPROPERTY('ProductVersion'))     AS phien_ban,
       CONVERT(nvarchar(20), SERVERPROPERTY('ProductUpdateLevel')) AS cu,
       CONVERT(nvarchar(60), SERVERPROPERTY('Edition'))            AS edition;
GO
