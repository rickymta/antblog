-- P01 · 02-fixture-valid.sql — bộ dữ liệu nhỏ, cố định, hợp lệ (MS-005, MS-006,
-- MS-007, MS-010). Cùng dữ liệu với lab PostgreSQL để đối chiếu hai engine.
--
-- Mọi tên, số điện thoại và biển hiệu đều HƯ CẤU (090000000x là số giả).
--
-- Để kết quả in ra giống nhau ở mọi lần chạy:
--   * khách, tài xế, quán, chuyến dùng GUID ghi sẵn (c… = customer, d…800x =
--     tài xế mã 800x, e… = quán, 7…100000 ↔ chuyến mã 100000). Đuôi trùng mã
--     CHỈ để dễ đọc; trong hệ thật id không liên quan gì tới code;
--   * mọi mốc thời gian ghi rõ độ lệch +07:00;
--   * chuyến được thêm theo thứ tự giờ đặt ⇒ IDENTITY cấp 100000, 100001…
--
-- Ảnh chụp dữ liệu lúc 2026-11-03 08:30 (giờ VN): ngày 02/11 đã xong, sáng
-- 03/11 còn 6 chuyến đang chạy ở 6 trạng thái khác nhau. Lịch sử trip_events
-- của 15 chuyến này KHÔNG được nạp (dữ liệu chuyển từ hệ cũ chỉ có trạng thái
-- cuối); 04-lifecycle.sql ghi lịch sử đầy đủ cho chuyến mới.
--
-- Chuỗi tiếng Việt luôn có tiền tố N'…' (Unicode). Thiếu N, chuỗi bị đổi sang
-- code page của collation trước khi lưu — MS-004 cho thấy chữ nào bị mất.

SET NOCOUNT ON;
USE ToiNoi;
GO

INSERT INTO toinoi.cities (id, code, name) VALUES
    (1, 'SG', N'TP. Hồ Chí Minh'),
    (2, 'HN', N'Hà Nội'),
    (3, 'DN', N'Đà Nẵng');
EXEC lab.expect_value N'cities sau khi nạp', @@ROWCOUNT, N'3';
GO

INSERT INTO toinoi.customers (id, phone, full_name, created_at) VALUES
    ('c0000000-0000-4000-8000-000000000001', '0900000001', N'Nguyễn Văn An',  '2026-09-01 08:00:00 +07:00'),
    ('c0000000-0000-4000-8000-000000000002', '0900000002', N'Trần Thị Bình',  '2026-09-03 19:15:00 +07:00'),
    ('c0000000-0000-4000-8000-000000000003', '0900000003', N'Lê Hoàng Cường', '2026-09-10 12:00:00 +07:00'),
    ('c0000000-0000-4000-8000-000000000004', '0900000004', N'Phạm Thu Dung',  '2026-10-01 07:30:00 +07:00'),
    ('c0000000-0000-4000-8000-000000000005', '0900000005', N'Võ Minh Đức',    '2026-10-15 21:45:00 +07:00'),
    ('c0000000-0000-4000-8000-000000000006', '0900000006', N'Đặng Ngọc Hà',   '2026-11-01 10:00:00 +07:00');
EXEC lab.expect_value N'customers sau khi nạp', @@ROWCOUNT, N'6';
GO

-- Trạng thái tài xế khớp ảnh chụp: ai đang có chuyến chạy thì `busy`.
INSERT INTO toinoi.drivers (id, code, full_name, city_id, vehicle_type, status, created_at) VALUES
    ('d0000000-0000-4000-8000-000000008001', 8001, N'Bùi Văn Khoa',   1, 'bike', 'busy',    '2026-08-01 09:00:00 +07:00'),
    ('d0000000-0000-4000-8000-000000008002', 8002, N'Hồ Thị Lan',     1, 'bike', 'busy',    '2026-08-02 09:00:00 +07:00'),
    ('d0000000-0000-4000-8000-000000008003', 8003, N'Ngô Quang Minh', 2, 'bike', 'online',  '2026-08-05 09:00:00 +07:00'),
    ('d0000000-0000-4000-8000-000000008004', 8004, N'Đỗ Thanh Nam',   2, 'car',  'busy',    '2026-08-20 09:00:00 +07:00'),
    ('d0000000-0000-4000-8000-000000008005', 8005, N'Lý Thị Oanh',    3, 'bike', 'busy',    '2026-09-01 09:00:00 +07:00'),
    ('d0000000-0000-4000-8000-000000008006', 8006, N'Trịnh Văn Phúc', 3, 'car',  'offline', '2026-09-15 09:00:00 +07:00');
EXEC lab.expect_value N'drivers sau khi nạp', @@ROWCOUNT, N'6';
GO

INSERT INTO toinoi.merchants (id, city_id, name, created_at) VALUES
    ('e0000000-0000-4000-8000-000000000001', 1, N'Cơm Tấm Phố Lab',     '2026-07-01 08:00:00 +07:00'),
    ('e0000000-0000-4000-8000-000000000002', 2, N'Bún Chả Ngõ Mẫu',     '2026-07-01 08:00:00 +07:00'),
    ('e0000000-0000-4000-8000-000000000003', 3, N'Mì Quảng Thử Nghiệm', '2026-07-01 08:00:00 +07:00');
EXEC lab.expect_value N'merchants sau khi nạp', @@ROWCOUNT, N'3';
GO

-- 15 chuyến, đủ 8 trạng thái và 3 loại dịch vụ. Cột `code` không có trong
-- danh sách cột: IDENTITY tự cấp theo thứ tự các dòng của câu INSERT … VALUES.
-- Hai chuyến 100003 và 100004 CÙNG giờ đặt 12:00 (dùng ở 05-select.sql).
-- 100007 đặt 23:50 ngày 02/11, 100008 đặt 00:10 ngày 03/11 giờ VN (vẫn là
-- ngày 02/11 theo UTC) — dùng để thử lọc theo ngày.
INSERT INTO toinoi.trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                          requested_at, accepted_at, completed_at, updated_at) VALUES
    -- 100000: đặt xe hoàn thành
    ('70000000-0000-4000-8000-000000100000', 'ride',   1, 'c0000000-0000-4000-8000-000000000001', 'd0000000-0000-4000-8000-000000008001', NULL,
     'completed',  45000, '2026-11-02 07:58:00 +07:00', '2026-11-02 08:00:30 +07:00', '2026-11-02 08:24:10 +07:00', '2026-11-02 08:24:10 +07:00'),
    -- 100001: bưu kiện hoàn thành
    ('70000000-0000-4000-8000-000000100001', 'parcel', 2, 'c0000000-0000-4000-8000-000000000003', 'd0000000-0000-4000-8000-000000008003', NULL,
     'completed',  38000, '2026-11-02 09:15:00 +07:00', '2026-11-02 09:17:00 +07:00', '2026-11-02 10:02:00 +07:00', '2026-11-02 10:02:00 +07:00'),
    -- 100002: đồ ăn hoàn thành
    ('70000000-0000-4000-8000-000000100002', 'food',   1, 'c0000000-0000-4000-8000-000000000002', 'd0000000-0000-4000-8000-000000008002', 'e0000000-0000-4000-8000-000000000001',
     'completed',  62000, '2026-11-02 11:05:00 +07:00', '2026-11-02 11:06:00 +07:00', '2026-11-02 11:41:00 +07:00', '2026-11-02 11:41:00 +07:00'),
    -- 100003: đồ ăn bị huỷ SAU khi tài xế đã nhận (vẫn giữ tài xế và accepted_at)
    ('70000000-0000-4000-8000-000000100003', 'food',   2, 'c0000000-0000-4000-8000-000000000003', 'd0000000-0000-4000-8000-000000008003', 'e0000000-0000-4000-8000-000000000002',
     'cancelled',  71000, '2026-11-02 12:00:00 +07:00', '2026-11-02 12:01:00 +07:00', NULL,                         '2026-11-02 12:04:00 +07:00'),
    -- 100004: đồ ăn hoàn thành, CÙNG giờ đặt với 100003
    ('70000000-0000-4000-8000-000000100004', 'food',   3, 'c0000000-0000-4000-8000-000000000005', 'd0000000-0000-4000-8000-000000008005', 'e0000000-0000-4000-8000-000000000003',
     'completed',  55000, '2026-11-02 12:00:00 +07:00', '2026-11-02 12:02:00 +07:00', '2026-11-02 12:35:00 +07:00', '2026-11-02 12:35:00 +07:00'),
    -- 100005: đặt xe ô tô hoàn thành
    ('70000000-0000-4000-8000-000000100005', 'ride',   2, 'c0000000-0000-4000-8000-000000000004', 'd0000000-0000-4000-8000-000000008004', NULL,
     'completed', 126000, '2026-11-02 17:30:00 +07:00', '2026-11-02 17:31:00 +07:00', '2026-11-02 18:05:00 +07:00', '2026-11-02 18:05:00 +07:00'),
    -- 100006: đặt xe bị huỷ TRƯỚC khi có người nhận (không tài xế)
    ('70000000-0000-4000-8000-000000100006', 'ride',   1, 'c0000000-0000-4000-8000-000000000001', NULL, NULL,
     'cancelled',  52000, '2026-11-02 18:00:00 +07:00', NULL,                         NULL,                         '2026-11-02 18:03:00 +07:00'),
    -- 100007: bưu kiện đặt 23:50, hoàn thành sau nửa đêm
    ('70000000-0000-4000-8000-000000100007', 'parcel', 1, 'c0000000-0000-4000-8000-000000000006', 'd0000000-0000-4000-8000-000000008002', NULL,
     'completed',  30000, '2026-11-02 23:50:00 +07:00', '2026-11-02 23:52:00 +07:00', '2026-11-03 00:20:00 +07:00', '2026-11-03 00:20:00 +07:00'),
    -- 100008: đặt xe miễn phí (khuyến mãi) — cước 0 đ là hợp lệ
    ('70000000-0000-4000-8000-000000100008', 'ride',   1, 'c0000000-0000-4000-8000-000000000002', 'd0000000-0000-4000-8000-000000008001', NULL,
     'completed',      0, '2026-11-03 00:10:00 +07:00', '2026-11-03 00:11:00 +07:00', '2026-11-03 00:30:00 +07:00', '2026-11-03 00:30:00 +07:00'),
    -- 100009..100014: sáu chuyến đang chạy lúc 08:30 ngày 03/11
    ('70000000-0000-4000-8000-000000100009', 'food',   1, 'c0000000-0000-4000-8000-000000000002', 'd0000000-0000-4000-8000-000000008002', 'e0000000-0000-4000-8000-000000000001',
     'picked_up',  64000, '2026-11-03 08:00:00 +07:00', '2026-11-03 08:02:00 +07:00', NULL,                         '2026-11-03 08:15:00 +07:00'),
    ('70000000-0000-4000-8000-000000100010', 'ride',   2, 'c0000000-0000-4000-8000-000000000003', 'd0000000-0000-4000-8000-000000008004', NULL,
     'in_transit', 98000, '2026-11-03 08:05:00 +07:00', '2026-11-03 08:06:00 +07:00', NULL,                         '2026-11-03 08:13:00 +07:00'),
    ('70000000-0000-4000-8000-000000100011', 'parcel', 3, 'c0000000-0000-4000-8000-000000000005', 'd0000000-0000-4000-8000-000000008005', NULL,
     'arriving',   27000, '2026-11-03 08:10:00 +07:00', '2026-11-03 08:12:00 +07:00', NULL,                         '2026-11-03 08:12:30 +07:00'),
    ('70000000-0000-4000-8000-000000100012', 'ride',   1, 'c0000000-0000-4000-8000-000000000001', 'd0000000-0000-4000-8000-000000008001', NULL,
     'accepted',   39000, '2026-11-03 08:20:00 +07:00', '2026-11-03 08:21:00 +07:00', NULL,                         '2026-11-03 08:21:00 +07:00'),
    ('70000000-0000-4000-8000-000000100013', 'food',   2, 'c0000000-0000-4000-8000-000000000004', NULL, 'e0000000-0000-4000-8000-000000000002',
     'searching',  85000, '2026-11-03 08:25:00 +07:00', NULL,                         NULL,                         '2026-11-03 08:25:10 +07:00'),
    ('70000000-0000-4000-8000-000000100014', 'ride',   1, 'c0000000-0000-4000-8000-000000000006', NULL, NULL,
     'requested',  48000, '2026-11-03 08:29:30 +07:00', NULL,                         NULL,                         '2026-11-03 08:29:30 +07:00');
EXEC lab.expect_value N'trips sau khi nạp', @@ROWCOUNT, N'15';
GO

-- IDENTITY cấp đúng 100000 … 100014 theo thứ tự dòng, và id ghi sẵn khớp mã.
DECLARE @lech int = (SELECT COUNT(*) FROM toinoi.trips
                     WHERE RIGHT(CONVERT(char(36), id), 6) <> CONVERT(varchar(6), code));
EXEC lab.expect_value N'chuyến có id lệch mã', @lech, N'0';
DECLARE @cur nvarchar(20) = CONVERT(nvarchar(20), IDENT_CURRENT(N'toinoi.trips'));
EXEC lab.expect_value N'IDENT_CURRENT(trips)', @cur, N'100014';
GO

-- Kiểm nhanh: số chuyến theo trạng thái và loại (8 trạng thái đều có mặt).
-- T-SQL không có FILTER (WHERE …): đếm có điều kiện bằng SUM(CASE …).
SELECT status,
       SUM(CASE WHEN kind = 'ride'   THEN 1 ELSE 0 END) AS ride,
       SUM(CASE WHEN kind = 'food'   THEN 1 ELSE 0 END) AS food,
       SUM(CASE WHEN kind = 'parcel' THEN 1 ELSE 0 END) AS parcel,
       COUNT(*)                                         AS tong
FROM toinoi.trips
GROUP BY status
ORDER BY CASE status WHEN 'requested' THEN 1 WHEN 'searching' THEN 2 WHEN 'accepted' THEN 3
                     WHEN 'arriving' THEN 4 WHEN 'picked_up' THEN 5 WHEN 'in_transit' THEN 6
                     WHEN 'completed' THEN 7 ELSE 8 END;
GO
