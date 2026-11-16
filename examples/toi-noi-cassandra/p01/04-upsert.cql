-- P01 · 04-upsert (CS-005): INSERT, UPDATE, DELETE đều là "ghi một phiên bản mới của ô dữ liệu".
-- Dùng chuyến nháp 07000000-…-000000000099 trong bảng trips; xoá ở cuối file.
-- Mọi câu ghi đặt USING TIMESTAMP cố định (micro giây) để WRITETIME tái lập được giữa các lần chạy.
USE toinoi;

-- ## INSERT hai lần cùng khoá chính: không lỗi trùng khoá, lần sau ghi đè
INSERT INTO trips (trip_id, service, city_id, status, fare_vnd)
VALUES (07000000-0000-4000-8000-000000000099, 'ride', 1, 'requested', 50000)
USING TIMESTAMP 1793580000000000;
INSERT INTO trips (trip_id, service, city_id, status, fare_vnd)
VALUES (07000000-0000-4000-8000-000000000099, 'ride', 1, 'searching', 52000)
USING TIMESTAMP 1793580001000000;
SELECT trip_id, status, fare_vnd, WRITETIME(status) AS wt_status FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000099;

-- ## INSERT chỉ ghi các cột được nêu; cột không nêu giữ nguyên giá trị cũ
INSERT INTO trips (trip_id, status)
VALUES (07000000-0000-4000-8000-000000000099, 'accepted')
USING TIMESTAMP 1793580002000000;
SELECT trip_id, service, status, fare_vnd FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000099;

-- ## Mỗi ô có WRITETIME riêng: dòng là tập các ô, không phải một bản ghi nguyên khối
SELECT WRITETIME(service) AS wt_service, WRITETIME(fare_vnd) AS wt_fare, WRITETIME(status) AS wt_status
FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000099;

-- ## UPDATE một dòng chưa tồn tại cũng tạo ra dữ liệu (upsert)
UPDATE trips USING TIMESTAMP 1793580003000000
SET status = 'requested', fare_vnd = 40000
WHERE trip_id = 07000000-0000-4000-8000-000000000098;
SELECT trip_id, status, fare_vnd FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000098;

-- ## Last-write-wins: câu ghi đến SAU nhưng mang timestamp CŨ hơn bị bỏ qua
UPDATE trips USING TIMESTAMP 1793579999000000
SET status = 'cancelled'
WHERE trip_id = 07000000-0000-4000-8000-000000000099;
SELECT trip_id, status, WRITETIME(status) AS wt_status FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000099;

-- ## Ghi NULL không phải "để trống": nó là một tombstone cho ô đó
INSERT INTO trips (trip_id, pickup, dropoff)
VALUES (07000000-0000-4000-8000-000000000099, 'Chợ Bến Thành, Q1', null)
USING TIMESTAMP 1793580004000000;
SELECT trip_id, pickup, dropoff, WRITETIME(dropoff) AS wt_dropoff FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000099;

-- ## Không sửa được cột thuộc khoá chính: muốn "đổi khoá" thì xoá dòng cũ và ghi dòng mới
-- EXPECT: PRIMARY KEY part trip_id found in SET part
UPDATE trips SET trip_id = 07000000-0000-4000-8000-000000000097 WHERE trip_id = 07000000-0000-4000-8000-000000000099;

-- ## Ghi có điều kiện (lightweight transaction): IF NOT EXISTS trả về [applied]
INSERT INTO trips (trip_id, status) VALUES (07000000-0000-4000-8000-000000000099, 'requested') IF NOT EXISTS;
UPDATE trips SET status = 'arriving' WHERE trip_id = 07000000-0000-4000-8000-000000000099 IF status = 'accepted';
UPDATE trips SET status = 'arriving' WHERE trip_id = 07000000-0000-4000-8000-000000000099 IF status = 'searching';

-- ## Không trộn USING TIMESTAMP với ghi có điều kiện
-- EXPECT: Cannot provide custom timestamp for conditional updates
UPDATE trips USING TIMESTAMP 1793580009000000 SET status = 'x' WHERE trip_id = 07000000-0000-4000-8000-000000000099 IF EXISTS;

-- ## DELETE một cột, rồi xoá cả dòng
DELETE fare_vnd FROM trips USING TIMESTAMP 1793580005000000 WHERE trip_id = 07000000-0000-4000-8000-000000000098;
SELECT trip_id, status, fare_vnd FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000098;
DELETE FROM trips USING TIMESTAMP 1793580006000000 WHERE trip_id = 07000000-0000-4000-8000-000000000098;
SELECT COUNT(*) AS con_lai FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000098;

-- ## Ghi lại với timestamp CŨ hơn lần xoá: dữ liệu không "sống lại" vì tombstone thắng
INSERT INTO trips (trip_id, status) VALUES (07000000-0000-4000-8000-000000000098, 'requested') USING TIMESTAMP 1793580005500000;
SELECT COUNT(*) AS con_lai FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000098;

-- ## DELETE theo dải clustering: xoá các sự kiện sau một thời điểm của một chuyến nháp
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000099, '2026-11-02 09:00:00+0700', 1, 'requested', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000099, '2026-11-02 09:01:00+0700', 2, 'searching', 'system');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000099, '2026-11-02 09:03:00+0700', 3, 'accepted', 'driver');
DELETE FROM trip_events_by_trip WHERE trip_id = 07000000-0000-4000-8000-000000000099 AND event_at > '2026-11-02 09:00:30+0700';
SELECT event_at, event_seq, status FROM trip_events_by_trip WHERE trip_id = 07000000-0000-4000-8000-000000000099;

-- ## DELETE phải chỉ rõ partition: không có "DELETE … WHERE status = …"
-- EXPECT: Some partition key parts are missing: trip_id
DELETE FROM trips WHERE status = 'cancelled';

-- ## Dọn chuyến nháp (không đặt timestamp: dùng giờ hiện tại nên chắc chắn mới hơn mọi lần ghi ở trên)
DELETE FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000099;
DELETE FROM trip_events_by_trip WHERE trip_id = 07000000-0000-4000-8000-000000000099;
SELECT COUNT(*) AS so_chuyen FROM trips;
