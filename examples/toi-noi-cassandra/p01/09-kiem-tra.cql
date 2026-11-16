-- P01 · 09-kiem-tra (CS-010): bộ kiểm tra bàn giao mô hình lịch sử chuyến.
-- Mỗi truy vấn của ứng dụng đọc đúng một partition; mỗi truy vấn "không được phép" bị từ chối;
-- bốn bảng phi chuẩn hoá khớp nhau. So kết quả với OUTPUT.txt.
USE toinoi;

-- ## K1 · Q1 trả đúng một chuyến theo mã
SELECT trip_id, status, fare_vnd FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000005;

-- ## K2 · Q2 trả chuyến của khách 3 theo thứ tự mới nhất trước
SELECT requested_at, service, fare_vnd FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000003;

-- ## K3 · Q3 trả chuyến của tài xế 1 trong ngày 03/11 và tổng cước
SELECT COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips_by_driver_day
WHERE driver_id = 0d000000-0000-4000-8000-000000000001 AND day = '2026-11-03';

-- ## K4 · Q4 trả chuyến của Hà Nội trong giờ 11:00 ngày 03/11
SELECT requested_at, service, fare_vnd FROM trips_by_city_hour
WHERE city_id = 2 AND hour_bucket = '2026-11-03 11:00:00+0700';

-- ## K5 · Q5: chuyến huỷ có đúng ba sự kiện, kết thúc bằng cancelled
SELECT event_seq, status, actor FROM trip_events_by_trip
WHERE trip_id = 07000000-0000-4000-8000-000000000005;

-- ## K6 · Chuyến 12 đã hoàn thành ở cả bốn bảng (sau CS-008)
SELECT status FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000012;
SELECT status FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000002 AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
SELECT status FROM trips_by_driver_day
WHERE driver_id = 0d000000-0000-4000-8000-000000000002 AND day = '2026-11-03' AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
SELECT status FROM trips_by_city_hour
WHERE city_id = 1 AND hour_bucket = '2026-11-03 18:00:00+0700' AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;

-- ## K7 · Đối soát: số chuyến và tổng cước của bốn bảng bằng nhau
SELECT COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips;
SELECT COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips_by_customer;
SELECT COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips_by_driver_day;
SELECT COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips_by_city_hour WHERE city_id IN (1, 2) ALLOW FILTERING;

-- ## K8 · Truy vấn không có partition key bị từ chối
-- EXPECT: Cannot execute this query as it might involve data filtering
SELECT * FROM trips_by_customer WHERE status = 'completed';
-- EXPECT: Cannot execute this query as it might involve data filtering
SELECT * FROM trips_by_driver_day WHERE driver_id = 0d000000-0000-4000-8000-000000000001;
-- EXPECT: Cannot execute this query as it might involve data filtering
SELECT * FROM trips WHERE customer_id = 0c000000-0000-4000-8000-000000000001;

-- ## K9 · Thứ tự chỉ đảo được theo clustering đã khai báo
-- EXPECT: Order by is currently only supported on the clustered columns of the PRIMARY KEY
SELECT * FROM trips_by_customer WHERE customer_id = 0c000000-0000-4000-8000-000000000001 ORDER BY fare_vnd;

-- ## K10 · Khoá chính không đổi được bằng UPDATE
-- EXPECT: PRIMARY KEY part requested_at found in SET part
UPDATE trips_by_customer SET requested_at = '2026-11-03 18:21:00+0700'
WHERE customer_id = 0c000000-0000-4000-8000-000000000002 AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
