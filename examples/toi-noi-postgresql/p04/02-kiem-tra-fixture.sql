-- P04 · 02-kiem-tra-fixture.sql — kiểm số dòng và các BẤT BIẾN mà bài sẽ phá/giữ.
-- Mỗi bài chạy trên một bản sao của nền này, nên các số dưới đây là điểm xuất phát của MỌI bài.

SET search_path TO toinoi, public;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Số dòng từng bảng.
-- expect-line: areas | 34
-- expect-line: customers | 20
-- expect-line: driver_duty | 24
-- expect-line: drivers | 24
-- expect-line: jobs | 10
-- expect-line: trip_events | 100
-- expect-line: trips | 40
-- expect-line: wallets | 44
SELECT 'areas' AS bang, count(*) FROM areas
UNION ALL SELECT 'customers', count(*) FROM customers
UNION ALL SELECT 'driver_duty', count(*) FROM driver_duty
UNION ALL SELECT 'drivers', count(*) FROM drivers
UNION ALL SELECT 'jobs', count(*) FROM jobs
UNION ALL SELECT 'trip_events', count(*) FROM trip_events
UNION ALL SELECT 'trips', count(*) FROM trips
UNION ALL SELECT 'wallets', count(*) FROM wallets
ORDER BY 1;

-- Chuyến theo trạng thái.
-- expect-line: cancelled | 3
-- expect-line: completed | 27
-- expect-line: requested | 10
SELECT status, count(*) FROM trips GROUP BY status ORDER BY status;

-- 10 chuyến đang chờ nhận: mã, khu đón, cước, version.
-- expect-md5: 464a6b63a43f77922dcc1312aa281141
SELECT code, kind, pickup_area_id, fare, version, driver_id IS NULL AS chua_co_tai_xe
FROM trips WHERE status = 'requested' ORDER BY code;

-- Tổng tiền trong các ví (bất biến của PG-037: chuyển tiền giữa ví không làm đổi tổng).
-- expect-line: customer | 20 | 10000000
-- expect-line: driver | 24 | 4800000
SELECT owner_kind, count(*), sum(balance) FROM wallets GROUP BY owner_kind ORDER BY 1;

-- Bất biến PG-038: mỗi khu có tài xế trực còn ít nhất 1 người trực (ở đây: đúng 2).
-- expect-value: 12
SELECT count(*) FROM (
  SELECT area_id FROM driver_duty GROUP BY area_id HAVING count(*) FILTER (WHERE on_duty) = 2
) AS k;

-- Ví dụ khu SG-TT-01 (111): hai tài xế 9101, 9102.
-- expect-line: 111 | 9101 | t
-- expect-line: 111 | 9102 | t
SELECT dd.area_id, d.code, dd.on_duty
FROM driver_duty AS dd JOIN drivers AS d ON d.id = dd.driver_id
WHERE dd.area_id = 111 ORDER BY d.code;

-- Hàng đợi: 10 việc 'queued', việc 1 ưu tiên 1.
-- expect-line: 1 | 1 | queued | 400101
-- expect-line: 2 | 5 | queued | 400102
-- expect-line: 10 | 5 | queued | 400110
SELECT j.id, j.priority, j.status, t.code
FROM jobs AS j JOIN trips AS t ON t.id = j.trip_id
WHERE j.id IN (1, 2, 10) ORDER BY j.id;
