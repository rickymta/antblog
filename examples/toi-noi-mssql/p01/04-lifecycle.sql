-- P01 · 04-lifecycle.sql — vòng đời một chuyến bằng INSERT/UPDATE/DELETE và
-- mệnh đề OUTPUT (MS-006).
--
-- Chuyến mới: khách 0900000004 (Phạm Thu Dung) đặt xe máy ở Hà Nội sáng 03/11.
-- Tài xế 8003 (Ngô Quang Minh, đang online) nhận. Mỗi lần đổi trạng thái:
--   1. UPDATE CÓ ĐIỀU KIỆN trạng thái cũ (chỉ đi tiếp từ đúng trạng thái trước);
--   2. OUTPUT … INTO một biến bảng, rồi INSERT vào trip_events — cả hai trong
--      MỘT transaction. Không ghi thẳng OUTPUT INTO toinoi.trip_events được:
--      SQL Server cấm bảng đích của OUTPUT INTO có CHECK đang bật hoặc nằm ở
--      một đầu của khoá ngoại (lỗi 333 / 332 — xem bước 2a).
--
-- Chuyến mới dùng GUID mẫu 71000000-…; mã `code` do IDENTITY cấp. Sau 03 mã
-- tiếp theo KHÔNG phải 100015: mỗi INSERT thất bại trong 03 vẫn tiêu một giá
-- trị IDENTITY dù đã ROLLBACK (bước 1 kiểm điều này).
-- Thời điểm dùng hằng số để kết quả lặp lại được; hệ thật dùng SYSDATETIMEOFFSET().

SET NOCOUNT ON;
SET XACT_ABORT ON;   -- lỗi giữa transaction ⇒ huỷ cả transaction
USE ToiNoi;
GO

-- ------------------------------------------------------------ 1. Khách đặt
-- OUTPUT inserted.* trả lại dòng vừa ghi, kể cả cột do database sinh (code,
-- status mặc định) — không cần SELECT lại hay đoán SCOPE_IDENTITY().
DECLARE @moi TABLE (code bigint);
BEGIN TRANSACTION;
INSERT INTO toinoi.trips (id, kind, city_id, customer_id, fare, requested_at, updated_at)
OUTPUT inserted.code INTO @moi (code)
OUTPUT inserted.code, inserted.status, inserted.fare, inserted.requested_at
VALUES ('71000000-0000-4000-8000-000000000001', 'ride', 2,
        (SELECT id FROM toinoi.customers WHERE phone = '0900000004'),
        42000, '2026-11-03 09:00:00 +07:00', '2026-11-03 09:00:00 +07:00');
INSERT INTO toinoi.trip_events (trip_id, status, occurred_at)
VALUES ('71000000-0000-4000-8000-000000000001', 'requested', '2026-11-03 09:00:00 +07:00');
COMMIT TRANSACTION;

DECLARE @code nvarchar(20) = (SELECT CONVERT(nvarchar(20), code) FROM @moi);
EXEC lab.expect_value N'mã chuyến mới (100014 + 17 giá trị bị tiêu ở 03 + 1)', @code, N'100032';
GO

-- --------------------------------- 2a. OUTPUT INTO thẳng bảng lịch sử: bị cấm
EXEC lab.expect_error N'O1 OUTPUT INTO bảng có CHECK/khoá ngoại',
     N'UPDATE toinoi.trips SET status = ''searching''
       OUTPUT inserted.id, inserted.status, inserted.updated_at INTO toinoi.trip_events (trip_id, status, occurred_at)
       WHERE id = ''71000000-0000-4000-8000-000000000001''', 333, N'ck_trip_events_status';
GO

-- --------------------------------------- 2b. Hệ thống tìm tài xế (đúng cách)
DECLARE @ev TABLE (trip_id uniqueidentifier, status varchar(12), occurred_at datetimeoffset(3));
BEGIN TRANSACTION;
UPDATE toinoi.trips
SET status = 'searching', updated_at = '2026-11-03 09:00:05 +07:00'
OUTPUT inserted.id, inserted.status, inserted.updated_at INTO @ev
WHERE id = '71000000-0000-4000-8000-000000000001' AND status = 'requested';
INSERT INTO toinoi.trip_events (trip_id, status, occurred_at) SELECT trip_id, status, occurred_at FROM @ev;
COMMIT TRANSACTION;
DECLARE @rc int = (SELECT COUNT(*) FROM @ev);
EXEC lab.expect_value N'requested → searching', @rc, N'1';
GO

-- ------------------------------------------------------- 3. Tài xế nhận
-- OUTPUT deleted.* (giá trị TRƯỚC) và inserted.* (giá trị SAU) trong cùng dòng.
DECLARE @ev TABLE (trip_id uniqueidentifier, status varchar(12), occurred_at datetimeoffset(3));
BEGIN TRANSACTION;
UPDATE toinoi.trips
SET status = 'accepted',
    driver_id = (SELECT id FROM toinoi.drivers WHERE code = 8003),
    accepted_at = '2026-11-03 09:01:00 +07:00',
    updated_at  = '2026-11-03 09:01:00 +07:00'
OUTPUT inserted.id, inserted.status, inserted.updated_at INTO @ev
OUTPUT deleted.status AS truoc, inserted.status AS sau, inserted.accepted_at
WHERE id = '71000000-0000-4000-8000-000000000001' AND status = 'searching' AND driver_id IS NULL;
INSERT INTO toinoi.trip_events (trip_id, status, occurred_at) SELECT trip_id, status, occurred_at FROM @ev;
UPDATE toinoi.drivers SET status = 'busy' WHERE code = 8003 AND status = 'online';
DECLARE @busy int = @@ROWCOUNT;
COMMIT TRANSACTION;
DECLARE @rc int = (SELECT COUNT(*) FROM @ev);
EXEC lab.expect_value N'searching → accepted', @rc, N'1';
EXEC lab.expect_value N'tài xế 8003 online → busy', @busy, N'1';
GO

-- --------------------------------------- 4. Tài xế khác bấm nhận CHẬM HƠN
-- Điều kiện status = 'searching' không còn đúng ⇒ 0 dòng bị đổi, KHÔNG lỗi.
-- Ứng dụng phải đọc @@ROWCOUNT (hoặc số dòng OUTPUT) để biết mình đã thua.
UPDATE toinoi.trips
SET status = 'accepted',
    driver_id = (SELECT id FROM toinoi.drivers WHERE code = 8001),
    accepted_at = '2026-11-03 09:01:02 +07:00',
    updated_at  = '2026-11-03 09:01:02 +07:00'
WHERE id = '71000000-0000-4000-8000-000000000001' AND status = 'searching' AND driver_id IS NULL;
EXEC lab.expect_value N'tài xế 8001 nhận trễ: số dòng đổi', @@ROWCOUNT, N'0';
GO

-- ------------------------------------- 5. Bốn bước còn lại tới completed
DECLARE @steps TABLE (n int, tu varchar(12), toi varchar(12), luc datetimeoffset(3));
INSERT INTO @steps VALUES
    (1, 'accepted',   'arriving',   '2026-11-03 09:01:30 +07:00'),
    (2, 'arriving',   'picked_up',  '2026-11-03 09:07:00 +07:00'),
    (3, 'picked_up',  'in_transit', '2026-11-03 09:07:20 +07:00'),
    (4, 'in_transit', 'completed',  '2026-11-03 09:31:00 +07:00');

DECLARE @ev TABLE (trip_id uniqueidentifier, status varchar(12), occurred_at datetimeoffset(3));
DECLARE @n int = 1, @tu varchar(12), @toi varchar(12), @luc datetimeoffset(3), @rc int, @lbl nvarchar(60);
WHILE @n <= 4
BEGIN
    SELECT @tu = tu, @toi = toi, @luc = luc FROM @steps WHERE n = @n;
    DELETE FROM @ev;
    BEGIN TRANSACTION;
    UPDATE toinoi.trips
    SET status = @toi,
        completed_at = IIF(@toi = 'completed', @luc, completed_at),
        updated_at = @luc
    OUTPUT inserted.id, inserted.status, inserted.updated_at INTO @ev
    WHERE id = '71000000-0000-4000-8000-000000000001' AND status = @tu;
    INSERT INTO toinoi.trip_events (trip_id, status, occurred_at) SELECT trip_id, status, occurred_at FROM @ev;
    COMMIT TRANSACTION;
    SET @rc = (SELECT COUNT(*) FROM @ev);
    SET @lbl = CONCAT(@tu, N' → ', @toi);
    EXEC lab.expect_value @lbl, @rc, N'1';
    SET @n += 1;
END;

UPDATE toinoi.drivers SET status = 'online' WHERE code = 8003 AND status = 'busy';
GO

-- Lịch sử đủ 7 trạng thái, thời lượng chuyến tính từ hai mốc.
SELECT e.id, e.status, CAST(e.occurred_at AS varchar(34)) AS occurred_at
FROM toinoi.trip_events AS e
WHERE e.trip_id = '71000000-0000-4000-8000-000000000001'
ORDER BY e.occurred_at, e.id;

SELECT code, status, fare, DATEDIFF(minute, requested_at, completed_at) AS so_phut
FROM toinoi.trips WHERE id = '71000000-0000-4000-8000-000000000001';

DECLARE @ev int = (SELECT COUNT(*) FROM toinoi.trip_events);
EXEC lab.expect_value N'số dòng trip_events', @ev, N'7';
GO

-- ------------------------------------------- 6. Bản nháp đặt nhầm rồi xoá
INSERT INTO toinoi.trips (id, kind, city_id, customer_id, fare, requested_at, updated_at)
VALUES ('71000000-0000-4000-8000-000000000002', 'parcel', 2,
        'c0000000-0000-4000-8000-000000000004', 25000,
        '2026-11-03 09:40:00 +07:00', '2026-11-03 09:40:00 +07:00');
INSERT INTO toinoi.trip_events (trip_id, status, occurred_at)
VALUES ('71000000-0000-4000-8000-000000000002', 'requested', '2026-11-03 09:40:00 +07:00');
GO

-- Xoá chuyến khi còn lịch sử trỏ tới: khoá ngoại chặn (không có ON DELETE CASCADE).
EXEC lab.expect_error N'D1 xoá chuyến còn lịch sử',
     N'DELETE FROM toinoi.trips WHERE id = ''71000000-0000-4000-8000-000000000002''', 547, N'fk_trip_events_trip';
GO

-- Xoá lịch sử trước, rồi chuyến, trong MỘT transaction; OUTPUT deleted.* cho
-- biết chính xác dòng nào đã đi. Điều kiện status = 'requested' giữ an toàn:
-- chuyến đã có tài xế nhận thì không xoá, transaction bị huỷ.
BEGIN TRANSACTION;
DELETE FROM toinoi.trip_events
OUTPUT deleted.trip_id, deleted.status
WHERE trip_id = '71000000-0000-4000-8000-000000000002';

DELETE FROM toinoi.trips
OUTPUT deleted.code, deleted.kind, deleted.status, deleted.fare
WHERE id = '71000000-0000-4000-8000-000000000002' AND status = 'requested';
IF @@ROWCOUNT = 1 COMMIT TRANSACTION; ELSE ROLLBACK TRANSACTION;
GO

-- Mã 100033 đã bị "tiêu": chuyến tiếp theo sẽ là 100034, không lấp lỗ.
DECLARE @cur nvarchar(20) = CONVERT(nvarchar(20), IDENT_CURRENT(N'toinoi.trips'));
EXEC lab.expect_value N'IDENT_CURRENT sau khi xoá bản nháp', @cur, N'100033';
DECLARE @n nvarchar(40) = (SELECT CONCAT((SELECT COUNT(*) FROM toinoi.trips), '/',
                                         (SELECT COUNT(*) FROM toinoi.trip_events)));
EXEC lab.expect_value N'số dòng trips/trip_events', @n, N'16/7';
GO
