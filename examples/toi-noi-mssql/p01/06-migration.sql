-- P01 · 06-migration.sql — thêm biển số xe cho tài xế mà ứng dụng bản cũ vẫn
-- chạy được: expand → backfill → contract (MS-009).
--
-- Bối cảnh: bộ phận vận hành cần biển số trên màn hình điều phối. Bản ứng dụng
-- cũ (đang chạy) không biết cột mới; bản mới sẽ ghi biển số khi tạo tài xế.
-- Không thể thêm thẳng cột NOT NULL không có DEFAULT vào bảng đã có dữ liệu,
-- và nếu thêm kèm DEFAULT giả ('???') thì dữ liệu rác hợp lệ mãi mãi.
--
-- Biển số mẫu (hư cấu, định dạng đơn giản hoá):
--   xe máy  59X1-123.45   (2 số tỉnh, 1 chữ, 1 số, '-', 3 số '.' 2 số)
--   ô tô    30A-123.45    (2 số tỉnh, 1 chữ, '-', 3 số '.' 2 số)

SET NOCOUNT ON;
SET XACT_ABORT ON;
USE ToiNoi;
GO

-- ------------------------------------------------------- Bước 1. EXPAND
-- Cột cho phép NULL: chỉ đổi metadata, không viết lại dòng nào; câu INSERT của
-- ứng dụng cũ (không nhắc tới cột) vẫn chạy.
ALTER TABLE toinoi.drivers ADD vehicle_plate varchar(12) NULL;
GO

-- Ứng dụng cũ vẫn thêm tài xế được (cột mới = NULL).
INSERT INTO toinoi.drivers (id, code, full_name, city_id, vehicle_type, status, created_at)
VALUES ('d0000000-0000-4000-8000-000000008007', 8007, N'Tạ Văn Quý', 1, 'bike', 'offline', '2026-11-03 10:00:00 +07:00');
EXEC lab.expect_value N'E1 app cũ thêm tài xế 8007', @@ROWCOUNT, N'1';
GO

-- Ràng buộc định dạng WITH NOCHECK: áp cho dòng MỚI và dòng bị SỬA từ bây
-- giờ, không quét dữ liệu cũ. SQL Server đánh dấu ràng buộc là "không tin cậy"
-- (is_not_trusted = 1). LIKE chạy trên collation nhị phân để chỉ nhận chữ hoa.
ALTER TABLE toinoi.drivers WITH NOCHECK
ADD CONSTRAINT ck_drivers_vehicle_plate_format CHECK (
    vehicle_plate COLLATE Latin1_General_BIN2 LIKE '[1-9][0-9][A-Z][0-9]-[0-9][0-9][0-9].[0-9][0-9]'
 OR vehicle_plate COLLATE Latin1_General_BIN2 LIKE '[1-9][0-9][A-Z]-[0-9][0-9][0-9].[0-9][0-9]');
GO

DECLARE @t nvarchar(5) = (SELECT CONVERT(nvarchar(5), is_not_trusted) FROM sys.check_constraints
                          WHERE name = N'ck_drivers_vehicle_plate_format');
EXEC lab.expect_value N'E2 ck_drivers_vehicle_plate_format is_not_trusted', @t, N'1';

EXEC lab.expect_error N'E3 app mới ghi biển sai định dạng',
     N'UPDATE toinoi.drivers SET vehicle_plate = ''59x1-12345'' WHERE code = 8001', 547, N'ck_drivers_vehicle_plate_format';
GO

-- ------------------------------------------------------ Bước 2. BACKFILL
-- Biển số lấy từ hồ sơ giấy. Một biển của dữ liệu cũ bị nhập sai (chữ thường)
-- — giả lập bằng cách tạm tắt ràng buộc, như dữ liệu nhập từ trước khi có nó.
ALTER TABLE toinoi.drivers NOCHECK CONSTRAINT ck_drivers_vehicle_plate_format;
UPDATE d SET vehicle_plate = v.plate
FROM toinoi.drivers AS d
JOIN (VALUES (8001, '59X1-123.45'), (8002, '59X2-234.56'), (8003, '29B1-345.67'),
             (8004, '30A-456.78'),  (8005, '43c1-567.89'), (8006, '43A-678.90'),
             (8007, '59X3-789.01')) AS v(code, plate) ON v.code = d.code;
EXEC lab.expect_value N'B1 backfill biển số', @@ROWCOUNT, N'7';
-- Bật lại (CHECK CONSTRAINT không kèm WITH CHECK): chặn dữ liệu sai từ giờ,
-- nhưng KHÔNG quét dòng cũ ⇒ ràng buộc vẫn không tin cậy.
ALTER TABLE toinoi.drivers CHECK CONSTRAINT ck_drivers_vehicle_plate_format;
GO

-- Muốn ràng buộc thành "tin cậy" phải kiểm cả dữ liệu cũ: WITH CHECK CHECK.
-- Biển '43c1-567.89' của tài xế 8005 làm câu này thất bại — đúng như mong đợi.
EXEC lab.expect_error N'B2 validate khi còn dữ liệu sai',
     N'ALTER TABLE toinoi.drivers WITH CHECK CHECK CONSTRAINT ck_drivers_vehicle_plate_format', 547, N'ck_drivers_vehicle_plate_format';

-- Tìm chính xác dòng vi phạm trước khi sửa.
SELECT code, vehicle_plate FROM toinoi.drivers
WHERE NOT (vehicle_plate COLLATE Latin1_General_BIN2 LIKE '[1-9][0-9][A-Z][0-9]-[0-9][0-9][0-9].[0-9][0-9]'
        OR vehicle_plate COLLATE Latin1_General_BIN2 LIKE '[1-9][0-9][A-Z]-[0-9][0-9][0-9].[0-9][0-9]');
GO

UPDATE toinoi.drivers SET vehicle_plate = UPPER(vehicle_plate) WHERE code = 8005;
ALTER TABLE toinoi.drivers WITH CHECK CHECK CONSTRAINT ck_drivers_vehicle_plate_format;
DECLARE @t nvarchar(5) = (SELECT CONVERT(nvarchar(5), is_not_trusted) FROM sys.check_constraints
                          WHERE name = N'ck_drivers_vehicle_plate_format');
EXEC lab.expect_value N'B3 sau khi sửa và validate: is_not_trusted', @t, N'0';
GO

-- ------------------------------------------------------ Bước 3. CONTRACT
-- Khi mọi bản ứng dụng đều ghi biển số: không còn NULL ⇒ đổi cột thành NOT
-- NULL. ALTER COLUMN quét cả bảng dưới khoá Sch-M: bảng lớn cần cửa sổ bảo trì.
DECLARE @nulls int = (SELECT COUNT(*) FROM toinoi.drivers WHERE vehicle_plate IS NULL);
EXEC lab.expect_value N'C1 còn tài xế thiếu biển', @nulls, N'0';
ALTER TABLE toinoi.drivers ALTER COLUMN vehicle_plate varchar(12) NOT NULL;
GO

EXEC lab.expect_error N'C2 app cũ (không gửi biển số) bị chặn sau contract',
     N'INSERT INTO toinoi.drivers (code, full_name, city_id, vehicle_type) VALUES (8008, N''Mai Thị Sen'', 2, ''bike'')', 515, N'vehicle_plate';

SELECT code, vehicle_type, vehicle_plate FROM toinoi.drivers ORDER BY code;
GO
