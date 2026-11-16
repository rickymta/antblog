-- P01 · 05-select (CS-006): SELECT theo khoá. Khách 1 = 0c…01, tài xế 1 = 0d…01, chuyến 1 = 07…01.
USE toinoi;

-- ## Q2 · Lịch sử chuyến của khách 1: một partition, đã sắp sẵn mới nhất trước
SELECT requested_at, service, status, fare_vnd, trip_id
FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001;

-- ## LIMIT: 2 chuyến gần nhất, không cần ORDER BY vì thứ tự đã nằm trong bảng
SELECT requested_at, service, fare_vnd
FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001
LIMIT 2;

-- ## Dải trên cột clustering đầu tiên: chuyến của khách 1 trong ngày 02/11 (giờ VN)
SELECT requested_at, service, fare_vnd
FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001
  AND requested_at >= '2026-11-02 00:00:00+0700'
  AND requested_at <  '2026-11-03 00:00:00+0700';

-- ## ORDER BY chỉ đảo chiều thứ tự clustering đã khai báo
SELECT requested_at, service
FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001
ORDER BY requested_at ASC, trip_id DESC;

-- EXPECT: Order by is currently only supported on the clustered columns of the PRIMARY KEY
SELECT requested_at, service FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001
ORDER BY fare_vnd DESC;

-- ## Lọc theo cột thường: bị từ chối nếu không có ALLOW FILTERING
-- EXPECT: Cannot execute this query as it might involve data filtering
SELECT requested_at, status FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001 AND status = 'completed';

-- ## ALLOW FILTERING trong MỘT partition nhỏ: chấp nhận được
SELECT requested_at, status, fare_vnd FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001 AND status = 'completed'
ALLOW FILTERING;

-- ## ALLOW FILTERING không có partition key: quét mọi partition trên mọi node
SELECT customer_id, requested_at, status FROM trips_by_customer
WHERE status = 'cancelled'
ALLOW FILTERING;

-- ## Q3 · Partition key gồm hai cột: phải có đủ cả driver_id và day
SELECT requested_at, service, status, fare_vnd
FROM trips_by_driver_day
WHERE driver_id = 0d000000-0000-4000-8000-000000000001 AND day = '2026-11-02';

-- EXPECT: Cannot execute this query as it might involve data filtering
SELECT requested_at, status FROM trips_by_driver_day
WHERE driver_id = 0d000000-0000-4000-8000-000000000001;

-- ## IN trên partition key: hai ngày = hai partition, có thể nằm ở hai node khác nhau
SELECT day, requested_at, service, fare_vnd
FROM trips_by_driver_day
WHERE driver_id = 0d000000-0000-4000-8000-000000000001 AND day IN ('2026-11-02', '2026-11-03');

-- ## Không được bỏ qua cột clustering đứng trước
-- EXPECT: PRIMARY KEY column "trip_id" cannot be restricted as preceding column "requested_at" is not restricted
SELECT * FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001
  AND trip_id = 07000000-0000-4000-8000-000000000001;

-- ## Dải trên partition key: partition rải theo token chứ không theo giá trị, nên đây là quét toàn bảng
-- EXPECT: Cannot execute this query as it might involve data filtering
SELECT * FROM trips_by_city_hour
WHERE city_id = 1 AND hour_bucket >= '2026-11-02 11:00:00+0700' AND hour_bucket < '2026-11-02 13:00:00+0700';

-- ## Thứ tự giữa các partition là thứ tự token, không phải thứ tự khách
SELECT DISTINCT customer_id, token(customer_id) AS token_cua_partition FROM trips_by_customer;

-- ## PER PARTITION LIMIT: chuyến gần nhất của từng khách
SELECT customer_id, requested_at, service, fare_vnd
FROM trips_by_customer
WHERE customer_id IN (0c000000-0000-4000-8000-000000000001, 0c000000-0000-4000-8000-000000000002, 0c000000-0000-4000-8000-000000000003)
PER PARTITION LIMIT 1;

-- ## Q5 · Dòng thời gian sự kiện của chuyến 1, cũ nhất trước
SELECT event_at, event_seq, status, actor
FROM trip_events_by_trip
WHERE trip_id = 07000000-0000-4000-8000-000000000001;

-- ## Q4 · Chuyến tạo trong giờ cao điểm trưa 11:00–12:00 ngày 02/11 ở TP.HCM
SELECT requested_at, service, status, fare_vnd
FROM trips_by_city_hour
WHERE city_id = 1 AND hour_bucket = '2026-11-02 11:00:00+0700';

-- ## Hai giờ liên tiếp = hai partition: liệt kê bằng IN
SELECT hour_bucket, requested_at, service
FROM trips_by_city_hour
WHERE city_id = 1 AND hour_bucket IN ('2026-11-02 17:00:00+0700', '2026-11-02 18:00:00+0700');
