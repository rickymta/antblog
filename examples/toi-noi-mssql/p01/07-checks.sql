-- P01 · 07-checks.sql — bất biến dữ liệu phải đúng sau 00 → 06 (MS-010; tư liệu
-- cho MS-008). Mỗi kiểm tra đếm số dòng VI PHẠM và mong đợi 0.
--
-- Ràng buộc trong bảng (PK, FK, UNIQUE, CHECK) do SQL Server giữ. Phần lớn quy
-- tắc dưới đây nằm GIỮA nhiều bảng hoặc nhiều dòng — CHECK của một dòng không
-- diễn đạt được — nên phải kiểm định kỳ, hoặc giữ bằng code/transaction.

SET NOCOUNT ON;
USE ToiNoi;
GO

DECLARE @v int;

-- K1. Tài xế nhận chuyến ở thành phố khác.
SET @v = (SELECT COUNT(*) FROM toinoi.trips AS t
          JOIN toinoi.drivers AS d ON d.id = t.driver_id
          WHERE d.city_id <> t.city_id);
EXEC lab.expect_value N'K1 tài xế khác thành phố với chuyến', @v, N'0';

-- K2. Đơn đồ ăn của quán ở thành phố khác.
SET @v = (SELECT COUNT(*) FROM toinoi.trips AS t
          JOIN toinoi.merchants AS m ON m.id = t.merchant_id
          WHERE m.city_id <> t.city_id);
EXEC lab.expect_value N'K2 quán khác thành phố với chuyến', @v, N'0';

-- K3. Một tài xế đang chạy nhiều hơn một chuyến cùng lúc.
SET @v = (SELECT COUNT(*) FROM (
            SELECT driver_id FROM toinoi.trips
            WHERE status IN ('accepted', 'arriving', 'picked_up', 'in_transit')
            GROUP BY driver_id HAVING COUNT(*) > 1) AS x);
EXEC lab.expect_value N'K3 tài xế có hơn một chuyến đang chạy', @v, N'0';

-- K4. drivers.status lệch với chuyến đang chạy (busy ⇔ có chuyến đang chạy).
SET @v = (SELECT COUNT(*) FROM toinoi.drivers AS d
          WHERE (d.status = 'busy' AND NOT EXISTS (
                    SELECT 1 FROM toinoi.trips AS t WHERE t.driver_id = d.id
                      AND t.status IN ('accepted', 'arriving', 'picked_up', 'in_transit')))
             OR (d.status <> 'busy' AND EXISTS (
                    SELECT 1 FROM toinoi.trips AS t WHERE t.driver_id = d.id
                      AND t.status IN ('accepted', 'arriving', 'picked_up', 'in_transit'))));
EXEC lab.expect_value N'K4 drivers.status lệch chuyến đang chạy', @v, N'0';

-- K5. Chuyến có lịch sử: sự kiện cuối cùng phải trùng trạng thái hiện tại.
SET @v = (SELECT COUNT(*) FROM toinoi.trips AS t
          CROSS APPLY (SELECT TOP (1) e.status FROM toinoi.trip_events AS e
                       WHERE e.trip_id = t.id ORDER BY e.occurred_at DESC, e.id DESC) AS last_ev
          WHERE last_ev.status <> t.status);
EXEC lab.expect_value N'K5 sự kiện cuối lệch trạng thái chuyến', @v, N'0';

-- K6. Sự kiện xảy ra trước khi chuyến được đặt.
SET @v = (SELECT COUNT(*) FROM toinoi.trip_events AS e
          JOIN toinoi.trips AS t ON t.id = e.trip_id
          WHERE e.occurred_at < t.requested_at);
EXEC lab.expect_value N'K6 sự kiện trước giờ đặt', @v, N'0';

-- K7. Mốc updated_at lùi so với các mốc khác của chính chuyến đó.
SET @v = (SELECT COUNT(*) FROM toinoi.trips
          WHERE updated_at < requested_at OR updated_at < accepted_at OR updated_at < completed_at);
EXEC lab.expect_value N'K7 updated_at nhỏ hơn mốc khác', @v, N'0';
GO

-- ------------------------------------------------ Sức khoẻ của ràng buộc
DECLARE @v int;

-- K8. Ràng buộc bị tắt hoặc không tin cậy (WITH NOCHECK chưa validate).
SET @v = (SELECT COUNT(*) FROM (
            SELECT name FROM sys.check_constraints
            WHERE OBJECT_SCHEMA_NAME(parent_object_id) = N'toinoi' AND (is_disabled = 1 OR is_not_trusted = 1)
            UNION ALL
            SELECT name FROM sys.foreign_keys
            WHERE OBJECT_SCHEMA_NAME(parent_object_id) = N'toinoi' AND (is_disabled = 1 OR is_not_trusted = 1)) AS x);
EXEC lab.expect_value N'K8 ràng buộc tắt / không tin cậy', @v, N'0';

-- K9. Tên ràng buộc sai quy ước (pk_ / fk_ / uq_ / ck_ / df_) — tên do SQL
-- Server tự sinh như DF__trips__statu__3B75D760 sẽ lọt vào đây.
SET @v = (SELECT COUNT(*) FROM sys.objects
          WHERE schema_id = SCHEMA_ID(N'toinoi')
            AND type IN ('PK', 'F', 'UQ', 'C', 'D')
            AND name NOT LIKE CASE type WHEN 'PK' THEN 'pk[_]%' WHEN 'F' THEN 'fk[_]%'
                                        WHEN 'UQ' THEN 'uq[_]%' WHEN 'C' THEN 'ck[_]%'
                                        ELSE 'df[_]%' END);
EXEC lab.expect_value N'K9 tên ràng buộc sai quy ước', @v, N'0';

-- K10. Khoá ngoại không có index hỗ trợ ở cột con (SQL Server không tự tạo).
-- fk_drivers_city, fk_merchants_city, fk_trips_city (cột city_id đứng đầu
-- ix_trips_city_requested ⇒ có), fk_trips_merchant, … — liệt kê để quyết định.
SELECT fk.name AS khoa_ngoai, COL_NAME(fkc.parent_object_id, fkc.parent_column_id) AS cot,
       IIF(EXISTS (SELECT 1 FROM sys.index_columns AS ic
                   WHERE ic.object_id = fkc.parent_object_id AND ic.key_ordinal = 1
                     AND ic.column_id = fkc.parent_column_id), N'có', N'KHÔNG') AS co_index
FROM sys.foreign_keys AS fk
JOIN sys.foreign_key_columns AS fkc ON fkc.constraint_object_id = fk.object_id
WHERE OBJECT_SCHEMA_NAME(fk.parent_object_id) = N'toinoi'
ORDER BY fk.name;
GO

-- K11. DBCC CHECKCONSTRAINTS quét lại MỌI CHECK/FK đang bật trên dữ liệu
-- hiện có; không in gì nghĩa là không dòng nào vi phạm.
DBCC CHECKCONSTRAINTS WITH ALL_CONSTRAINTS, NO_INFOMSGS;
PRINT N'PASS K11 DBCC CHECKCONSTRAINTS không báo vi phạm';
GO

-- ------------------------------------------------------- Kiểm kê cuối
SELECT OBJECT_NAME(parent_object_id) AS bang,
       SUM(IIF(type = 'PK', 1, 0)) AS pk, SUM(IIF(type = 'F', 1, 0)) AS fk,
       SUM(IIF(type = 'UQ', 1, 0)) AS uq, SUM(IIF(type = 'C', 1, 0)) AS ck,
       SUM(IIF(type = 'D', 1, 0)) AS df
FROM sys.objects
WHERE schema_id = SCHEMA_ID(N'toinoi') AND type IN ('PK', 'F', 'UQ', 'C', 'D')
GROUP BY parent_object_id
ORDER BY bang;

DECLARE @n nvarchar(40) = (SELECT CONCAT((SELECT COUNT(*) FROM toinoi.cities), '/',
    (SELECT COUNT(*) FROM toinoi.customers), '/', (SELECT COUNT(*) FROM toinoi.drivers), '/',
    (SELECT COUNT(*) FROM toinoi.merchants), '/', (SELECT COUNT(*) FROM toinoi.trips), '/',
    (SELECT COUNT(*) FROM toinoi.trip_events)));
EXEC lab.expect_value N'số dòng cuối cities/customers/drivers/merchants/trips/trip_events', @n, N'3/6/7/3/16/7';
GO
