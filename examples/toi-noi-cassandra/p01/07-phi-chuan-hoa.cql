-- P01 · 07-phi-chuan-hoa (CS-008): một chuyến nằm ở bốn bảng; đổi trạng thái phải đổi cả bốn.
USE toinoi;

-- ## Trước: chuyến 12 đang in_transit ở cả bốn bảng
SELECT status AS o_trips FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000012;
SELECT status AS o_trips_by_customer FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000002 AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
SELECT status AS o_trips_by_driver_day FROM trips_by_driver_day
WHERE driver_id = 0d000000-0000-4000-8000-000000000002 AND day = '2026-11-03' AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
SELECT status AS o_trips_by_city_hour FROM trips_by_city_hour
WHERE city_id = 1 AND hour_bucket = '2026-11-03 18:00:00+0700' AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;

-- ## Giao xong: một logged batch cập nhật cả bốn bảng và ghi sự kiện
-- Logged batch bảo đảm: một khi batch đã vào batchlog thì MỌI câu bên trong rồi sẽ được áp dụng
-- (kể cả khi node điều phối chết giữa chừng). Nó KHÔNG cô lập: người đọc có thể thấy bảng này đã đổi, bảng kia chưa.
BEGIN BATCH
  UPDATE trips SET status = 'completed' WHERE trip_id = 07000000-0000-4000-8000-000000000012;
  UPDATE trips_by_customer SET status = 'completed'
    WHERE customer_id = 0c000000-0000-4000-8000-000000000002 AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
  UPDATE trips_by_driver_day SET status = 'completed'
    WHERE driver_id = 0d000000-0000-4000-8000-000000000002 AND day = '2026-11-03' AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
  UPDATE trips_by_city_hour SET status = 'completed'
    WHERE city_id = 1 AND hour_bucket = '2026-11-03 18:00:00+0700' AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
  INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor)
    VALUES (07000000-0000-4000-8000-000000000012, '2026-11-03 18:58:41+0700', 7, 'completed', 'driver');
APPLY BATCH;

-- ## Sau: cả bốn bảng cùng completed, dòng thời gian có thêm sự kiện thứ 7
SELECT status AS o_trips FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000012;
SELECT status AS o_trips_by_customer FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000002 AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
SELECT status AS o_trips_by_driver_day FROM trips_by_driver_day
WHERE driver_id = 0d000000-0000-4000-8000-000000000002 AND day = '2026-11-03' AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
SELECT status AS o_trips_by_city_hour FROM trips_by_city_hour
WHERE city_id = 1 AND hour_bucket = '2026-11-03 18:00:00+0700' AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
SELECT event_seq, status FROM trip_events_by_trip WHERE trip_id = 07000000-0000-4000-8000-000000000012;

-- ## Thiếu khoá đầy đủ thì không UPDATE được bản sao: phải biết requested_at của chuyến
-- EXPECT: Some clustering keys are missing: requested_at
UPDATE trips_by_customer SET status = 'completed'
WHERE customer_id = 0c000000-0000-4000-8000-000000000002 AND trip_id = 07000000-0000-4000-8000-000000000012;

-- ## Lỗi thường gặp: chỉ sửa bảng trips → các bản sao lệch nhau
UPDATE trips SET fare_vnd = 95000 WHERE trip_id = 07000000-0000-4000-8000-000000000011;
SELECT fare_vnd AS fare_o_trips FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000011;
SELECT fare_vnd AS fare_o_trips_by_customer FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001 AND requested_at = '2026-11-03 17:45:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000011;

-- ## Sửa lệch: ghi lại giá trị đúng vào mọi bản sao trong một batch
BEGIN BATCH
  UPDATE trips_by_customer SET fare_vnd = 95000
    WHERE customer_id = 0c000000-0000-4000-8000-000000000001 AND requested_at = '2026-11-03 17:45:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000011;
  UPDATE trips_by_driver_day SET fare_vnd = 95000
    WHERE driver_id = 0d000000-0000-4000-8000-000000000002 AND day = '2026-11-03' AND requested_at = '2026-11-03 17:45:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000011;
  UPDATE trips_by_city_hour SET fare_vnd = 95000
    WHERE city_id = 1 AND hour_bucket = '2026-11-03 17:00:00+0700' AND requested_at = '2026-11-03 17:45:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000011;
APPLY BATCH;
SELECT fare_vnd AS fare_o_trips_by_customer FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001 AND requested_at = '2026-11-03 17:45:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000011;

-- ## Batch có điều kiện (IF) chỉ được phép trong MỘT partition của MỘT bảng
-- EXPECT: Batch with conditions cannot span multiple tables
BEGIN BATCH
  UPDATE trips SET status = 'completed' WHERE trip_id = 07000000-0000-4000-8000-000000000012 IF status = 'in_transit';
  UPDATE trips_by_customer SET status = 'completed'
    WHERE customer_id = 0c000000-0000-4000-8000-000000000002 AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
APPLY BATCH;

-- ## Đối soát: số chuyến và tổng cước theo bốn bảng phải bằng nhau
-- (bảng nhỏ nên quét toàn bảng được; với dữ liệu thật làm bằng job quét theo token range — CS-036)
SELECT (text)'trips' AS bang, COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips;
SELECT (text)'trips_by_customer' AS bang, COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips_by_customer;
SELECT (text)'trips_by_driver_day' AS bang, COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips_by_driver_day;
SELECT (text)'trips_by_city_hour' AS bang, COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips_by_city_hour WHERE city_id IN (1, 2) ALLOW FILTERING;
