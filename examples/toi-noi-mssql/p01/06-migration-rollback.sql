-- P01 · 06-migration-rollback.sql — hoàn tác 06 theo hai mức (MS-009).
-- KHÔNG nằm trong `npm run p01`; chạy tay hoặc qua `npm run p01:rollback`.
-- Chạy lặp được: mỗi bước kiểm trạng thái trước khi làm.
--
-- Mức 1 · Nới lỏng (giữ dữ liệu): cột trở lại NULL ⇒ ứng dụng cũ thêm tài xế
--   được ngay. Đây là bước nên làm trước khi có sự cố sau contract.
-- Mức 2 · Gỡ hẳn: sao lưu biển số sang bảng riêng rồi xoá ràng buộc và cột.

SET NOCOUNT ON;
SET XACT_ABORT ON;
USE [$(LAB_DB)];
GO

-- Mức 1
IF COLUMNPROPERTY(OBJECT_ID(N'toinoi.drivers'), N'vehicle_plate', 'AllowsNull') = 0
    ALTER TABLE toinoi.drivers ALTER COLUMN vehicle_plate varchar(12) NULL;
GO

-- Mức 2: sao lưu trước khi xoá (SELECT … INTO tạo bảng mới).
IF COL_LENGTH(N'toinoi.drivers', N'vehicle_plate') IS NOT NULL
   AND OBJECT_ID(N'lab.backup_drivers_vehicle_plate') IS NULL
BEGIN
    EXEC (N'SELECT id, code, vehicle_plate, SYSDATETIMEOFFSET() AS backed_up_at
            INTO lab.backup_drivers_vehicle_plate
            FROM toinoi.drivers WHERE vehicle_plate IS NOT NULL;');
END;
GO

ALTER TABLE toinoi.drivers DROP CONSTRAINT IF EXISTS ck_drivers_vehicle_plate_format;
IF COL_LENGTH(N'toinoi.drivers', N'vehicle_plate') IS NOT NULL
    ALTER TABLE toinoi.drivers DROP COLUMN vehicle_plate;
GO

DECLARE @col nvarchar(10) = IIF(COL_LENGTH(N'toinoi.drivers', N'vehicle_plate') IS NULL, N'không', N'còn');
EXEC lab.expect_value N'R1 cột vehicle_plate', @col, N'không';
DECLARE @bk nvarchar(10) = CONVERT(nvarchar(10), (SELECT COUNT(*) FROM lab.backup_drivers_vehicle_plate));
EXEC lab.expect_value N'R2 số biển số đã sao lưu', @bk, N'7';
GO
