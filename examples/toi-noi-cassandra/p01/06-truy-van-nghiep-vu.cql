-- P01 · 06-truy-van-nghiep-vu (CS-007): mỗi màn hình của Tới Nơi là một truy vấn, mỗi truy vấn đọc đúng một partition.
USE toinoi;

-- ## Màn hình "Chuyến của tôi" (app khách): Q2, trang đầu 3 chuyến
SELECT requested_at, service, status, pickup, dropoff, fare_vnd
FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000002
LIMIT 3;

-- ## Bấm vào một chuyến: Q1 lấy chi tiết, Q5 lấy dòng thời gian
SELECT trip_id, service, status, customer_id, driver_id, requested_at, fare_vnd FROM trips
WHERE trip_id = 07000000-0000-4000-8000-000000000008;
SELECT event_at, status, actor FROM trip_events_by_trip
WHERE trip_id = 07000000-0000-4000-8000-000000000008;

-- ## Màn hình "Thu nhập hôm nay" (app tài xế): Q3, rồi cộng tiền trong partition
SELECT requested_at, service, status, fare_vnd
FROM trips_by_driver_day
WHERE driver_id = 0d000000-0000-4000-8000-000000000002 AND day = '2026-11-03';
SELECT COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc_dong
FROM trips_by_driver_day
WHERE driver_id = 0d000000-0000-4000-8000-000000000002 AND day = '2026-11-03';

-- ## Màn hình điều phối TP.HCM, giờ 18:00–19:00 ngày 03/11: Q4
SELECT requested_at, service, status, fare_vnd
FROM trips_by_city_hour
WHERE city_id = 1 AND hour_bucket = '2026-11-03 18:00:00+0700';

-- ## Cùng câu hỏi "lịch sử chuyến của khách 2" nhưng hỏi bảng trips: chỉ chạy được khi quét toàn bảng
-- EXPECT: Cannot execute this query as it might involve data filtering
SELECT trip_id, requested_at, status FROM trips
WHERE customer_id = 0c000000-0000-4000-8000-000000000002;

-- Có ALLOW FILTERING thì chạy, nhưng kết quả KHÔNG theo thứ tự thời gian và chi phí tăng theo cả bảng.
SELECT trip_id, requested_at, status FROM trips
WHERE customer_id = 0c000000-0000-4000-8000-000000000002
ALLOW FILTERING;

-- ## Câu hỏi chưa có bảng nào trả lời: "các chuyến bị huỷ trong một giờ ở TP.HCM" → cần bảng mới (bài tập CS-007)
-- EXPECT: Cannot execute this query as it might involve data filtering
SELECT trip_id FROM trips_by_city_hour
WHERE city_id = 1 AND hour_bucket = '2026-11-02 17:00:00+0700' AND status = 'cancelled';
