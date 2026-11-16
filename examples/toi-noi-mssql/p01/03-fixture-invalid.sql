-- P01 · 03-fixture-invalid.sql — mỗi câu vi phạm ĐÚNG MỘT quy tắc; SQL Server
-- phải từ chối với đúng số hiệu lỗi (MS-004, MS-005, MS-010).
--
-- Số hiệu lỗi hay gặp trong phần này:
--   515  Cannot insert the value NULL …            (NOT NULL)
--   2627 Violation of PRIMARY KEY / UNIQUE KEY …   (PK, UNIQUE constraint)
--   547  … conflicted with the FOREIGN KEY / CHECK constraint …
--   544  Cannot insert explicit value for identity column … IDENTITY_INSERT OFF
--   2628 String or binary data would be truncated in table …, column … (SQL Server 2019+)
--   220  Arithmetic overflow error for data type smallint …
--   8115 Arithmetic overflow error converting expression to data type int
--   245  Conversion failed when converting the varchar value … to data type int
--   241  Conversion failed when converting date and/or time from character string
--   8169 Conversion failed when converting from a character string to uniqueidentifier
--
-- lab.expect_error chạy câu lệnh trong transaction riêng rồi ROLLBACK, nên dù
-- câu lệnh có lỡ thành công, dữ liệu vẫn giữ nguyên 3/6/6/3/15.

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

-- ---------------------------------------------------------------- NOT NULL (3)
EXEC lab.expect_error N'N1 khách thiếu số điện thoại',
     N'INSERT INTO toinoi.customers (full_name) VALUES (N''Khách Thiếu Số'')', 515, N'phone';
EXEC lab.expect_error N'N2 chuyến thiếu cước',
     N'INSERT INTO toinoi.trips (kind, city_id, customer_id) VALUES (''ride'', 1, ''c0000000-0000-4000-8000-000000000001'')', 515, N'fare';
EXEC lab.expect_error N'N3 đặt NULL tường minh cho status (DEFAULT không cứu)',
     N'INSERT INTO toinoi.trips (kind, city_id, customer_id, status, fare) VALUES (''ride'', 1, ''c0000000-0000-4000-8000-000000000001'', NULL, 1000)', 515, N'status';
GO

-- ------------------------------------------------------- PRIMARY KEY / UNIQUE (4)
EXEC lab.expect_error N'U1 trùng id thành phố',
     N'INSERT INTO toinoi.cities (id, code, name) VALUES (1, ''CT'', N''Cần Thơ'')', 2627, N'pk_cities';
EXEC lab.expect_error N'U2 trùng số điện thoại',
     N'INSERT INTO toinoi.customers (phone, full_name) VALUES (''0900000001'', N''Người Khác'')', 2627, N'uq_customers_phone';
EXEC lab.expect_error N'U3 trùng mã tài xế',
     N'INSERT INTO toinoi.drivers (code, full_name, city_id, vehicle_type) VALUES (8001, N''Tài Xế Trùng Mã'', 1, ''bike'')', 2627, N'uq_drivers_code';
-- Dùng mã in hoa trùng để câu này chỉ vi phạm UNIQUE ('hn' còn vi phạm thêm
-- CHECK định dạng; collation CI vốn coi 'hn' = 'HN').
EXEC lab.expect_error N'U4 trùng mã thành phố',
     N'INSERT INTO toinoi.cities (id, code, name) VALUES (9, ''HN'', N''Hà Nội 2'')', 2627, N'uq_cities_code';
GO

-- ----------------------------------------------------------- FOREIGN KEY (3)
EXEC lab.expect_error N'F1 chuyến của khách không tồn tại',
     N'INSERT INTO toinoi.trips (kind, city_id, customer_id, fare) VALUES (''ride'', 1, ''c0000000-0000-4000-8000-0000000000ff'', 1000)', 547, N'fk_trips_customer';
EXEC lab.expect_error N'F2 tài xế ở thành phố không tồn tại',
     N'INSERT INTO toinoi.drivers (code, full_name, city_id, vehicle_type) VALUES (8099, N''Tài Xế Lạc'', 99, ''bike'')', 547, N'fk_drivers_city';
EXEC lab.expect_error N'F3 xoá khách đang có chuyến',
     N'DELETE FROM toinoi.customers WHERE phone = ''0900000001''', 547, N'fk_trips_customer';
GO

-- ----------------------------------------------------------------- CHECK (17)
DECLARE @c nvarchar(60) = N'''c0000000-0000-4000-8000-000000000001''';
DECLARE @d nvarchar(60) = N'''d0000000-0000-4000-8000-000000008001''';
DECLARE @e nvarchar(60) = N'''e0000000-0000-4000-8000-000000000001''';
DECLARE @ins nvarchar(400) = N'INSERT INTO toinoi.trips (kind, city_id, customer_id, driver_id, merchant_id, status, fare, requested_at, accepted_at, completed_at) VALUES ';

EXEC lab.expect_error N'C1 mã thành phố chữ thường',
     N'INSERT INTO toinoi.cities (id, code, name) VALUES (4, ''ct'', N''Cần Thơ'')', 547, N'ck_cities_code_format';
EXEC lab.expect_error N'C2 tên thành phố toàn khoảng trắng',
     N'INSERT INTO toinoi.cities (id, code, name) VALUES (4, ''CT'', N''   '')', 547, N'ck_cities_name_not_blank';
EXEC lab.expect_error N'C3 số điện thoại dạng +84',
     N'INSERT INTO toinoi.customers (phone, full_name) VALUES (''+849000001'', N''Khách Quốc Tế'')', 547, N'ck_customers_phone_format';
EXEC lab.expect_error N'C4 số điện thoại 9 chữ số',
     N'INSERT INTO toinoi.customers (phone, full_name) VALUES (''090000009'', N''Khách Thiếu Số'')', 547, N'ck_customers_phone_format';
EXEC lab.expect_error N'C5 mã tài xế âm',
     N'INSERT INTO toinoi.drivers (code, full_name, city_id, vehicle_type) VALUES (-1, N''Tài Xế Âm'', 1, ''bike'')', 547, N'ck_drivers_code_positive';
EXEC lab.expect_error N'C6 loại xe lạ',
     N'INSERT INTO toinoi.drivers (code, full_name, city_id, vehicle_type) VALUES (8098, N''Tài Xế Xe Tải'', 1, ''truck'')', 547, N'ck_drivers_vehicle_type';
EXEC lab.expect_error N'C7 trạng thái tài xế lạ',
     N'UPDATE toinoi.drivers SET status = ''sleeping'' WHERE code = 8003', 547, N'ck_drivers_status';

DECLARE @s nvarchar(max);
SET @s = @ins + N'(''bus'', 1, ' + @c + N', NULL, NULL, ''requested'', 1000, ''2026-11-03 09:00 +07:00'', NULL, NULL)';
EXEC lab.expect_error N'C8 loại dịch vụ lạ', @s, 547, N'ck_trips_kind';
SET @s = @ins + N'(''ride'', 1, ' + @c + N', NULL, NULL, ''done'', 1000, ''2026-11-03 09:00 +07:00'', NULL, NULL)';
EXEC lab.expect_error N'C9 trạng thái chuyến lạ', @s, 547, N'ck_trips_status';
SET @s = @ins + N'(''ride'', 1, ' + @c + N', NULL, NULL, ''requested'', -5000, ''2026-11-03 09:00 +07:00'', NULL, NULL)';
EXEC lab.expect_error N'C10 cước âm', @s, 547, N'ck_trips_fare_non_negative';
SET @s = @ins + N'(''food'', 1, ' + @c + N', NULL, NULL, ''requested'', 50000, ''2026-11-03 09:00 +07:00'', NULL, NULL)';
EXEC lab.expect_error N'C11 đồ ăn không có quán', @s, 547, N'ck_trips_merchant_only_food';
SET @s = @ins + N'(''ride'', 1, ' + @c + N', NULL, ' + @e + N', ''requested'', 50000, ''2026-11-03 09:00 +07:00'', NULL, NULL)';
EXEC lab.expect_error N'C12 đặt xe gắn quán', @s, 547, N'ck_trips_merchant_only_food';
SET @s = @ins + N'(''ride'', 1, ' + @c + N', ' + @d + N', NULL, ''cancelled'', 50000, ''2026-11-03 09:00 +07:00'', NULL, NULL)';
EXEC lab.expect_error N'C13 có tài xế nhưng thiếu accepted_at', @s, 547, N'ck_trips_driver_accepted_together';
SET @s = @ins + N'(''ride'', 1, ' + @c + N', ' + @d + N', NULL, ''searching'', 50000, ''2026-11-03 09:00 +07:00'', ''2026-11-03 09:01 +07:00'', NULL)';
EXEC lab.expect_error N'C14 đang tìm tài xế mà đã có tài xế', @s, 547, N'ck_trips_unassigned_no_driver';
SET @s = @ins + N'(''ride'', 1, ' + @c + N', NULL, NULL, ''arriving'', 50000, ''2026-11-03 09:00 +07:00'', NULL, NULL)';
EXEC lab.expect_error N'C15 đang tới đón mà không có tài xế', @s, 547, N'ck_trips_assigned_has_driver';
SET @s = @ins + N'(''ride'', 1, ' + @c + N', NULL, NULL, ''cancelled'', 50000, ''2026-11-03 09:00 +07:00'', NULL, ''2026-11-03 09:30 +07:00'')';
EXEC lab.expect_error N'C16 huỷ mà có mốc hoàn thành', @s, 547, N'ck_trips_completed_at_by_status';
-- Cùng một khoảnh khắc viết theo hai múi giờ: 08:59 +07:00 = 01:59 UTC, sớm hơn
-- 09:00 +07:00 ⇒ nhận trước khi đặt. datetimeoffset so sánh theo UTC.
SET @s = @ins + N'(''ride'', 1, ' + @c + N', ' + @d + N', NULL, ''accepted'', 50000, ''2026-11-03 09:00 +07:00'', ''2026-11-03 01:59 +00:00'', NULL)';
EXEC lab.expect_error N'C17 nhận chuyến trước giờ đặt (khác múi giờ)', @s, 547, N'ck_trips_accepted_after_requested';
GO

-- -------------------------------------------------------------- IDENTITY (1)
EXEC lab.expect_error N'I1 tự ghi mã chuyến',
     N'INSERT INTO toinoi.trips (code, kind, city_id, customer_id, fare) VALUES (999, ''ride'', 1, ''c0000000-0000-4000-8000-000000000001'', 1000)', 544, N'IDENTITY_INSERT';
GO

-- ------------------------------------------------ Kiểu, độ dài, chuyển đổi (6)
EXEC lab.expect_error N'T1 tên dài quá 100 ký tự',
     N'INSERT INTO toinoi.customers (phone, full_name) VALUES (''0900000099'', REPLICATE(N''A'', 101))', 2628, N'full_name';
EXEC lab.expect_error N'T2 city_id vượt smallint',
     N'INSERT INTO toinoi.cities (id, code, name) VALUES (40000, ''XX'', N''Quá Lớn'')', 220, NULL;
EXEC lab.expect_error N'T3 cước vượt int (2.147.483.648 đ)',
     N'INSERT INTO toinoi.trips (kind, city_id, customer_id, fare) VALUES (''ride'', 1, ''c0000000-0000-4000-8000-000000000001'', 2147483648)', 8115, NULL;
EXEC lab.expect_error N'T4 cước viết kiểu "45.000đ"',
     N'INSERT INTO toinoi.trips (kind, city_id, customer_id, fare) VALUES (''ride'', 1, ''c0000000-0000-4000-8000-000000000001'', ''45.000đ'')', 245, NULL;
EXEC lab.expect_error N'T5 ngày 31/11 không tồn tại',
     N'INSERT INTO toinoi.trips (kind, city_id, customer_id, fare, requested_at) VALUES (''ride'', 1, ''c0000000-0000-4000-8000-000000000001'', 1000, ''2026-11-31 09:00 +07:00'')', 241, NULL;
EXEC lab.expect_error N'T6 GUID sai định dạng',
     N'INSERT INTO toinoi.trips (kind, city_id, customer_id, fare) VALUES (''ride'', 1, ''khach-001'', 1000)', 8169, NULL;
GO

-- -------------------------------------------- Bẫy KHÔNG báo lỗi (cần biết)
-- DATEFORMAT dmy của phiên làm '03/11/2026' thành ngày 3 tháng 11; dmy đổi
-- thành mdy thì cùng chuỗi đó là ngày 11 tháng 3. Không lỗi nào cả ⇒ luôn
-- dùng dạng ISO 'YYYY-MM-DD hh:mm:ss +07:00' trong script và tham số kiểu
-- ngày giờ từ ứng dụng.
SET DATEFORMAT dmy;
DECLARE @a nvarchar(30) = CONVERT(nvarchar(30), CAST('03/11/2026' AS date), 23);
SET DATEFORMAT mdy;
DECLARE @b nvarchar(30) = CONVERT(nvarchar(30), CAST('03/11/2026' AS date), 23);
EXEC lab.expect_value N'B1 ''03/11/2026'' với DATEFORMAT dmy', @a, N'2026-11-03';
EXEC lab.expect_value N'B2 ''03/11/2026'' với DATEFORMAT mdy', @b, N'2026-03-11';
GO

-- Dữ liệu vẫn nguyên sau 34 câu sai.
DECLARE @n nvarchar(40) = (SELECT CONCAT((SELECT COUNT(*) FROM toinoi.cities), '/',
    (SELECT COUNT(*) FROM toinoi.customers), '/', (SELECT COUNT(*) FROM toinoi.drivers), '/',
    (SELECT COUNT(*) FROM toinoi.merchants), '/', (SELECT COUNT(*) FROM toinoi.trips)));
EXEC lab.expect_value N'số dòng cities/customers/drivers/merchants/trips', @n, N'3/6/6/3/15';
GO
