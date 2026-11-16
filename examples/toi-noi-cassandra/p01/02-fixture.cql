-- Fixture P01: 12 chuyến của 3 khách, 3 tài xế, 2 thành phố (1 = TP.HCM, 2 = Hà Nội), ngày 02–03/11/2026.
-- Mỗi chuyến được ghi vào BỐN bảng trong cùng một logged batch (CS-008). Sự kiện ghi riêng vào trip_events_by_trip.
-- Mọi tên địa điểm là hư cấu hoặc địa danh công cộng; định danh là UUID cố định để kết quả tái lập được.
USE toinoi;

-- ## Chuyến 01: ride, 2026-11-02 07:42 (+07:00), completed
BEGIN BATCH
  INSERT INTO trips (trip_id, service, city_id, customer_id, driver_id, status, requested_at, pickup, dropoff, fare_vnd) VALUES (07000000-0000-4000-8000-000000000001, 'ride', 1, 0c000000-0000-4000-8000-000000000001, 0d000000-0000-4000-8000-000000000001, 'completed', '2026-11-02 07:42:00+0700', 'Chung cư Sky Garden, Q7', 'Toà nhà Bitexco, Q1', 86000);
  INSERT INTO trips_by_customer (customer_id, requested_at, trip_id, service, status, pickup, dropoff, fare_vnd, driver_id) VALUES (0c000000-0000-4000-8000-000000000001, '2026-11-02 07:42:00+0700', 07000000-0000-4000-8000-000000000001, 'ride', 'completed', 'Chung cư Sky Garden, Q7', 'Toà nhà Bitexco, Q1', 86000, 0d000000-0000-4000-8000-000000000001);
  INSERT INTO trips_by_driver_day (driver_id, day, requested_at, trip_id, customer_id, service, status, fare_vnd) VALUES (0d000000-0000-4000-8000-000000000001, '2026-11-02', '2026-11-02 07:42:00+0700', 07000000-0000-4000-8000-000000000001, 0c000000-0000-4000-8000-000000000001, 'ride', 'completed', 86000);
  INSERT INTO trips_by_city_hour (city_id, hour_bucket, requested_at, trip_id, service, status, fare_vnd) VALUES (1, '2026-11-02 07:00:00+0700', '2026-11-02 07:42:00+0700', 07000000-0000-4000-8000-000000000001, 'ride', 'completed', 86000);
APPLY BATCH;

-- ## Chuyến 02: food, 2026-11-02 11:05 (+07:00), completed
BEGIN BATCH
  INSERT INTO trips (trip_id, service, city_id, customer_id, driver_id, status, requested_at, pickup, dropoff, fare_vnd) VALUES (07000000-0000-4000-8000-000000000002, 'food', 1, 0c000000-0000-4000-8000-000000000002, 0d000000-0000-4000-8000-000000000002, 'completed', '2026-11-02 11:05:00+0700', 'Quán Cơm Tấm Cô Ba, Q3', 'Văn phòng Etown, Tân Bình', 42000);
  INSERT INTO trips_by_customer (customer_id, requested_at, trip_id, service, status, pickup, dropoff, fare_vnd, driver_id) VALUES (0c000000-0000-4000-8000-000000000002, '2026-11-02 11:05:00+0700', 07000000-0000-4000-8000-000000000002, 'food', 'completed', 'Quán Cơm Tấm Cô Ba, Q3', 'Văn phòng Etown, Tân Bình', 42000, 0d000000-0000-4000-8000-000000000002);
  INSERT INTO trips_by_driver_day (driver_id, day, requested_at, trip_id, customer_id, service, status, fare_vnd) VALUES (0d000000-0000-4000-8000-000000000002, '2026-11-02', '2026-11-02 11:05:00+0700', 07000000-0000-4000-8000-000000000002, 0c000000-0000-4000-8000-000000000002, 'food', 'completed', 42000);
  INSERT INTO trips_by_city_hour (city_id, hour_bucket, requested_at, trip_id, service, status, fare_vnd) VALUES (1, '2026-11-02 11:00:00+0700', '2026-11-02 11:05:00+0700', 07000000-0000-4000-8000-000000000002, 'food', 'completed', 42000);
APPLY BATCH;

-- ## Chuyến 03: food, 2026-11-02 11:48 (+07:00), completed
BEGIN BATCH
  INSERT INTO trips (trip_id, service, city_id, customer_id, driver_id, status, requested_at, pickup, dropoff, fare_vnd) VALUES (07000000-0000-4000-8000-000000000003, 'food', 1, 0c000000-0000-4000-8000-000000000001, 0d000000-0000-4000-8000-000000000001, 'completed', '2026-11-02 11:48:00+0700', 'Bún Bò Huế O Xuân, Q1', 'Toà nhà Bitexco, Q1', 35000);
  INSERT INTO trips_by_customer (customer_id, requested_at, trip_id, service, status, pickup, dropoff, fare_vnd, driver_id) VALUES (0c000000-0000-4000-8000-000000000001, '2026-11-02 11:48:00+0700', 07000000-0000-4000-8000-000000000003, 'food', 'completed', 'Bún Bò Huế O Xuân, Q1', 'Toà nhà Bitexco, Q1', 35000, 0d000000-0000-4000-8000-000000000001);
  INSERT INTO trips_by_driver_day (driver_id, day, requested_at, trip_id, customer_id, service, status, fare_vnd) VALUES (0d000000-0000-4000-8000-000000000001, '2026-11-02', '2026-11-02 11:48:00+0700', 07000000-0000-4000-8000-000000000003, 0c000000-0000-4000-8000-000000000001, 'food', 'completed', 35000);
  INSERT INTO trips_by_city_hour (city_id, hour_bucket, requested_at, trip_id, service, status, fare_vnd) VALUES (1, '2026-11-02 11:00:00+0700', '2026-11-02 11:48:00+0700', 07000000-0000-4000-8000-000000000003, 'food', 'completed', 35000);
APPLY BATCH;

-- ## Chuyến 04: ride, 2026-11-02 12:10 (+07:00), completed
BEGIN BATCH
  INSERT INTO trips (trip_id, service, city_id, customer_id, driver_id, status, requested_at, pickup, dropoff, fare_vnd) VALUES (07000000-0000-4000-8000-000000000004, 'ride', 2, 0c000000-0000-4000-8000-000000000003, 0d000000-0000-4000-8000-000000000003, 'completed', '2026-11-02 12:10:00+0700', 'Hồ Hoàn Kiếm', 'Đại học Bách khoa Hà Nội', 64000);
  INSERT INTO trips_by_customer (customer_id, requested_at, trip_id, service, status, pickup, dropoff, fare_vnd, driver_id) VALUES (0c000000-0000-4000-8000-000000000003, '2026-11-02 12:10:00+0700', 07000000-0000-4000-8000-000000000004, 'ride', 'completed', 'Hồ Hoàn Kiếm', 'Đại học Bách khoa Hà Nội', 64000, 0d000000-0000-4000-8000-000000000003);
  INSERT INTO trips_by_driver_day (driver_id, day, requested_at, trip_id, customer_id, service, status, fare_vnd) VALUES (0d000000-0000-4000-8000-000000000003, '2026-11-02', '2026-11-02 12:10:00+0700', 07000000-0000-4000-8000-000000000004, 0c000000-0000-4000-8000-000000000003, 'ride', 'completed', 64000);
  INSERT INTO trips_by_city_hour (city_id, hour_bucket, requested_at, trip_id, service, status, fare_vnd) VALUES (2, '2026-11-02 12:00:00+0700', '2026-11-02 12:10:00+0700', 07000000-0000-4000-8000-000000000004, 'ride', 'completed', 64000);
APPLY BATCH;

-- ## Chuyến 05: ride, 2026-11-02 17:31 (+07:00), cancelled
BEGIN BATCH
  INSERT INTO trips (trip_id, service, city_id, customer_id, driver_id, status, requested_at, pickup, dropoff, fare_vnd) VALUES (07000000-0000-4000-8000-000000000005, 'ride', 1, 0c000000-0000-4000-8000-000000000002, 0d000000-0000-4000-8000-000000000002, 'cancelled', '2026-11-02 17:31:00+0700', 'Văn phòng Etown, Tân Bình', 'Chợ Bến Thành, Q1', 0);
  INSERT INTO trips_by_customer (customer_id, requested_at, trip_id, service, status, pickup, dropoff, fare_vnd, driver_id) VALUES (0c000000-0000-4000-8000-000000000002, '2026-11-02 17:31:00+0700', 07000000-0000-4000-8000-000000000005, 'ride', 'cancelled', 'Văn phòng Etown, Tân Bình', 'Chợ Bến Thành, Q1', 0, 0d000000-0000-4000-8000-000000000002);
  INSERT INTO trips_by_driver_day (driver_id, day, requested_at, trip_id, customer_id, service, status, fare_vnd) VALUES (0d000000-0000-4000-8000-000000000002, '2026-11-02', '2026-11-02 17:31:00+0700', 07000000-0000-4000-8000-000000000005, 0c000000-0000-4000-8000-000000000002, 'ride', 'cancelled', 0);
  INSERT INTO trips_by_city_hour (city_id, hour_bucket, requested_at, trip_id, service, status, fare_vnd) VALUES (1, '2026-11-02 17:00:00+0700', '2026-11-02 17:31:00+0700', 07000000-0000-4000-8000-000000000005, 'ride', 'cancelled', 0);
APPLY BATCH;

-- ## Chuyến 06: ride, 2026-11-02 18:02 (+07:00), completed
BEGIN BATCH
  INSERT INTO trips (trip_id, service, city_id, customer_id, driver_id, status, requested_at, pickup, dropoff, fare_vnd) VALUES (07000000-0000-4000-8000-000000000006, 'ride', 1, 0c000000-0000-4000-8000-000000000001, 0d000000-0000-4000-8000-000000000002, 'completed', '2026-11-02 18:02:00+0700', 'Toà nhà Bitexco, Q1', 'Chung cư Sky Garden, Q7', 91000);
  INSERT INTO trips_by_customer (customer_id, requested_at, trip_id, service, status, pickup, dropoff, fare_vnd, driver_id) VALUES (0c000000-0000-4000-8000-000000000001, '2026-11-02 18:02:00+0700', 07000000-0000-4000-8000-000000000006, 'ride', 'completed', 'Toà nhà Bitexco, Q1', 'Chung cư Sky Garden, Q7', 91000, 0d000000-0000-4000-8000-000000000002);
  INSERT INTO trips_by_driver_day (driver_id, day, requested_at, trip_id, customer_id, service, status, fare_vnd) VALUES (0d000000-0000-4000-8000-000000000002, '2026-11-02', '2026-11-02 18:02:00+0700', 07000000-0000-4000-8000-000000000006, 0c000000-0000-4000-8000-000000000001, 'ride', 'completed', 91000);
  INSERT INTO trips_by_city_hour (city_id, hour_bucket, requested_at, trip_id, service, status, fare_vnd) VALUES (1, '2026-11-02 18:00:00+0700', '2026-11-02 18:02:00+0700', 07000000-0000-4000-8000-000000000006, 'ride', 'completed', 91000);
APPLY BATCH;

-- ## Chuyến 07: parcel, 2026-11-02 18:40 (+07:00), completed
BEGIN BATCH
  INSERT INTO trips (trip_id, service, city_id, customer_id, driver_id, status, requested_at, pickup, dropoff, fare_vnd) VALUES (07000000-0000-4000-8000-000000000007, 'parcel', 2, 0c000000-0000-4000-8000-000000000003, 0d000000-0000-4000-8000-000000000003, 'completed', '2026-11-02 18:40:00+0700', 'Bưu cục Cầu Giấy', 'Khu đô thị Times City', 28000);
  INSERT INTO trips_by_customer (customer_id, requested_at, trip_id, service, status, pickup, dropoff, fare_vnd, driver_id) VALUES (0c000000-0000-4000-8000-000000000003, '2026-11-02 18:40:00+0700', 07000000-0000-4000-8000-000000000007, 'parcel', 'completed', 'Bưu cục Cầu Giấy', 'Khu đô thị Times City', 28000, 0d000000-0000-4000-8000-000000000003);
  INSERT INTO trips_by_driver_day (driver_id, day, requested_at, trip_id, customer_id, service, status, fare_vnd) VALUES (0d000000-0000-4000-8000-000000000003, '2026-11-02', '2026-11-02 18:40:00+0700', 07000000-0000-4000-8000-000000000007, 0c000000-0000-4000-8000-000000000003, 'parcel', 'completed', 28000);
  INSERT INTO trips_by_city_hour (city_id, hour_bucket, requested_at, trip_id, service, status, fare_vnd) VALUES (2, '2026-11-02 18:00:00+0700', '2026-11-02 18:40:00+0700', 07000000-0000-4000-8000-000000000007, 'parcel', 'completed', 28000);
APPLY BATCH;

-- ## Chuyến 08: ride, 2026-11-03 07:55 (+07:00), completed
BEGIN BATCH
  INSERT INTO trips (trip_id, service, city_id, customer_id, driver_id, status, requested_at, pickup, dropoff, fare_vnd) VALUES (07000000-0000-4000-8000-000000000008, 'ride', 1, 0c000000-0000-4000-8000-000000000002, 0d000000-0000-4000-8000-000000000001, 'completed', '2026-11-03 07:55:00+0700', 'Chung cư Sunrise City, Q7', 'Sân bay Tân Sơn Nhất', 145000);
  INSERT INTO trips_by_customer (customer_id, requested_at, trip_id, service, status, pickup, dropoff, fare_vnd, driver_id) VALUES (0c000000-0000-4000-8000-000000000002, '2026-11-03 07:55:00+0700', 07000000-0000-4000-8000-000000000008, 'ride', 'completed', 'Chung cư Sunrise City, Q7', 'Sân bay Tân Sơn Nhất', 145000, 0d000000-0000-4000-8000-000000000001);
  INSERT INTO trips_by_driver_day (driver_id, day, requested_at, trip_id, customer_id, service, status, fare_vnd) VALUES (0d000000-0000-4000-8000-000000000001, '2026-11-03', '2026-11-03 07:55:00+0700', 07000000-0000-4000-8000-000000000008, 0c000000-0000-4000-8000-000000000002, 'ride', 'completed', 145000);
  INSERT INTO trips_by_city_hour (city_id, hour_bucket, requested_at, trip_id, service, status, fare_vnd) VALUES (1, '2026-11-03 07:00:00+0700', '2026-11-03 07:55:00+0700', 07000000-0000-4000-8000-000000000008, 'ride', 'completed', 145000);
APPLY BATCH;

-- ## Chuyến 09: food, 2026-11-03 11:20 (+07:00), completed
BEGIN BATCH
  INSERT INTO trips (trip_id, service, city_id, customer_id, driver_id, status, requested_at, pickup, dropoff, fare_vnd) VALUES (07000000-0000-4000-8000-000000000009, 'food', 1, 0c000000-0000-4000-8000-000000000001, 0d000000-0000-4000-8000-000000000001, 'completed', '2026-11-03 11:20:00+0700', 'Phở Thìn Lò Đúc, Q1', 'Toà nhà Bitexco, Q1', 52000);
  INSERT INTO trips_by_customer (customer_id, requested_at, trip_id, service, status, pickup, dropoff, fare_vnd, driver_id) VALUES (0c000000-0000-4000-8000-000000000001, '2026-11-03 11:20:00+0700', 07000000-0000-4000-8000-000000000009, 'food', 'completed', 'Phở Thìn Lò Đúc, Q1', 'Toà nhà Bitexco, Q1', 52000, 0d000000-0000-4000-8000-000000000001);
  INSERT INTO trips_by_driver_day (driver_id, day, requested_at, trip_id, customer_id, service, status, fare_vnd) VALUES (0d000000-0000-4000-8000-000000000001, '2026-11-03', '2026-11-03 11:20:00+0700', 07000000-0000-4000-8000-000000000009, 0c000000-0000-4000-8000-000000000001, 'food', 'completed', 52000);
  INSERT INTO trips_by_city_hour (city_id, hour_bucket, requested_at, trip_id, service, status, fare_vnd) VALUES (1, '2026-11-03 11:00:00+0700', '2026-11-03 11:20:00+0700', 07000000-0000-4000-8000-000000000009, 'food', 'completed', 52000);
APPLY BATCH;

-- ## Chuyến 10: ride, 2026-11-03 11:35 (+07:00), completed
BEGIN BATCH
  INSERT INTO trips (trip_id, service, city_id, customer_id, driver_id, status, requested_at, pickup, dropoff, fare_vnd) VALUES (07000000-0000-4000-8000-000000000010, 'ride', 2, 0c000000-0000-4000-8000-000000000003, 0d000000-0000-4000-8000-000000000003, 'completed', '2026-11-03 11:35:00+0700', 'Khu đô thị Times City', 'Phố cổ Hàng Bạc', 57000);
  INSERT INTO trips_by_customer (customer_id, requested_at, trip_id, service, status, pickup, dropoff, fare_vnd, driver_id) VALUES (0c000000-0000-4000-8000-000000000003, '2026-11-03 11:35:00+0700', 07000000-0000-4000-8000-000000000010, 'ride', 'completed', 'Khu đô thị Times City', 'Phố cổ Hàng Bạc', 57000, 0d000000-0000-4000-8000-000000000003);
  INSERT INTO trips_by_driver_day (driver_id, day, requested_at, trip_id, customer_id, service, status, fare_vnd) VALUES (0d000000-0000-4000-8000-000000000003, '2026-11-03', '2026-11-03 11:35:00+0700', 07000000-0000-4000-8000-000000000010, 0c000000-0000-4000-8000-000000000003, 'ride', 'completed', 57000);
  INSERT INTO trips_by_city_hour (city_id, hour_bucket, requested_at, trip_id, service, status, fare_vnd) VALUES (2, '2026-11-03 11:00:00+0700', '2026-11-03 11:35:00+0700', 07000000-0000-4000-8000-000000000010, 'ride', 'completed', 57000);
APPLY BATCH;

-- ## Chuyến 11: ride, 2026-11-03 17:45 (+07:00), completed
BEGIN BATCH
  INSERT INTO trips (trip_id, service, city_id, customer_id, driver_id, status, requested_at, pickup, dropoff, fare_vnd) VALUES (07000000-0000-4000-8000-000000000011, 'ride', 1, 0c000000-0000-4000-8000-000000000001, 0d000000-0000-4000-8000-000000000002, 'completed', '2026-11-03 17:45:00+0700', 'Toà nhà Bitexco, Q1', 'Chung cư Sky Garden, Q7', 89000);
  INSERT INTO trips_by_customer (customer_id, requested_at, trip_id, service, status, pickup, dropoff, fare_vnd, driver_id) VALUES (0c000000-0000-4000-8000-000000000001, '2026-11-03 17:45:00+0700', 07000000-0000-4000-8000-000000000011, 'ride', 'completed', 'Toà nhà Bitexco, Q1', 'Chung cư Sky Garden, Q7', 89000, 0d000000-0000-4000-8000-000000000002);
  INSERT INTO trips_by_driver_day (driver_id, day, requested_at, trip_id, customer_id, service, status, fare_vnd) VALUES (0d000000-0000-4000-8000-000000000002, '2026-11-03', '2026-11-03 17:45:00+0700', 07000000-0000-4000-8000-000000000011, 0c000000-0000-4000-8000-000000000001, 'ride', 'completed', 89000);
  INSERT INTO trips_by_city_hour (city_id, hour_bucket, requested_at, trip_id, service, status, fare_vnd) VALUES (1, '2026-11-03 17:00:00+0700', '2026-11-03 17:45:00+0700', 07000000-0000-4000-8000-000000000011, 'ride', 'completed', 89000);
APPLY BATCH;

-- ## Chuyến 12: food, 2026-11-03 18:20 (+07:00), in_transit
BEGIN BATCH
  INSERT INTO trips (trip_id, service, city_id, customer_id, driver_id, status, requested_at, pickup, dropoff, fare_vnd) VALUES (07000000-0000-4000-8000-000000000012, 'food', 1, 0c000000-0000-4000-8000-000000000002, 0d000000-0000-4000-8000-000000000002, 'in_transit', '2026-11-03 18:20:00+0700', 'Lẩu Dê 404, Q3', 'Chung cư Sunrise City, Q7', 118000);
  INSERT INTO trips_by_customer (customer_id, requested_at, trip_id, service, status, pickup, dropoff, fare_vnd, driver_id) VALUES (0c000000-0000-4000-8000-000000000002, '2026-11-03 18:20:00+0700', 07000000-0000-4000-8000-000000000012, 'food', 'in_transit', 'Lẩu Dê 404, Q3', 'Chung cư Sunrise City, Q7', 118000, 0d000000-0000-4000-8000-000000000002);
  INSERT INTO trips_by_driver_day (driver_id, day, requested_at, trip_id, customer_id, service, status, fare_vnd) VALUES (0d000000-0000-4000-8000-000000000002, '2026-11-03', '2026-11-03 18:20:00+0700', 07000000-0000-4000-8000-000000000012, 0c000000-0000-4000-8000-000000000002, 'food', 'in_transit', 118000);
  INSERT INTO trips_by_city_hour (city_id, hour_bucket, requested_at, trip_id, service, status, fare_vnd) VALUES (1, '2026-11-03 18:00:00+0700', '2026-11-03 18:20:00+0700', 07000000-0000-4000-8000-000000000012, 'food', 'in_transit', 118000);
APPLY BATCH;

-- ## Dòng thời gian sự kiện của 12 chuyến
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000001, '2026-11-02 07:42:00+0700', 1, 'requested', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000001, '2026-11-02 07:42:17+0700', 2, 'searching', 'system');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000001, '2026-11-02 07:43:51+0700', 3, 'accepted', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000001, '2026-11-02 07:46:42+0700', 4, 'arriving', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000001, '2026-11-02 07:53:50+0700', 5, 'picked_up', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000001, '2026-11-02 07:58:15+0700', 6, 'in_transit', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000001, '2026-11-02 08:11:57+0700', 7, 'completed', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000002, '2026-11-02 11:05:00+0700', 1, 'requested', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000002, '2026-11-02 11:05:17+0700', 2, 'searching', 'system');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000002, '2026-11-02 11:06:51+0700', 3, 'accepted', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000002, '2026-11-02 11:09:42+0700', 4, 'arriving', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000002, '2026-11-02 11:16:50+0700', 5, 'picked_up', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000002, '2026-11-02 11:21:15+0700', 6, 'in_transit', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000002, '2026-11-02 11:34:57+0700', 7, 'completed', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000003, '2026-11-02 11:48:00+0700', 1, 'requested', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000003, '2026-11-02 11:48:17+0700', 2, 'searching', 'system');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000003, '2026-11-02 11:49:51+0700', 3, 'accepted', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000003, '2026-11-02 11:52:42+0700', 4, 'arriving', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000003, '2026-11-02 11:59:50+0700', 5, 'picked_up', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000003, '2026-11-02 12:04:15+0700', 6, 'in_transit', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000003, '2026-11-02 12:17:57+0700', 7, 'completed', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000004, '2026-11-02 12:10:00+0700', 1, 'requested', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000004, '2026-11-02 12:10:17+0700', 2, 'searching', 'system');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000004, '2026-11-02 12:11:51+0700', 3, 'accepted', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000004, '2026-11-02 12:14:42+0700', 4, 'arriving', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000004, '2026-11-02 12:21:50+0700', 5, 'picked_up', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000004, '2026-11-02 12:26:15+0700', 6, 'in_transit', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000004, '2026-11-02 12:39:57+0700', 7, 'completed', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000005, '2026-11-02 17:31:00+0700', 1, 'requested', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000005, '2026-11-02 17:31:17+0700', 2, 'searching', 'system');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000005, '2026-11-02 17:32:51+0700', 3, 'cancelled', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000006, '2026-11-02 18:02:00+0700', 1, 'requested', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000006, '2026-11-02 18:02:17+0700', 2, 'searching', 'system');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000006, '2026-11-02 18:03:51+0700', 3, 'accepted', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000006, '2026-11-02 18:06:42+0700', 4, 'arriving', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000006, '2026-11-02 18:13:50+0700', 5, 'picked_up', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000006, '2026-11-02 18:18:15+0700', 6, 'in_transit', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000006, '2026-11-02 18:31:57+0700', 7, 'completed', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000007, '2026-11-02 18:40:00+0700', 1, 'requested', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000007, '2026-11-02 18:40:17+0700', 2, 'searching', 'system');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000007, '2026-11-02 18:41:51+0700', 3, 'accepted', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000007, '2026-11-02 18:44:42+0700', 4, 'arriving', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000007, '2026-11-02 18:51:50+0700', 5, 'picked_up', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000007, '2026-11-02 18:56:15+0700', 6, 'in_transit', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000007, '2026-11-02 19:09:57+0700', 7, 'completed', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000008, '2026-11-03 07:55:00+0700', 1, 'requested', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000008, '2026-11-03 07:55:17+0700', 2, 'searching', 'system');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000008, '2026-11-03 07:56:51+0700', 3, 'accepted', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000008, '2026-11-03 07:59:42+0700', 4, 'arriving', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000008, '2026-11-03 08:06:50+0700', 5, 'picked_up', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000008, '2026-11-03 08:11:15+0700', 6, 'in_transit', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000008, '2026-11-03 08:24:57+0700', 7, 'completed', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000009, '2026-11-03 11:20:00+0700', 1, 'requested', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000009, '2026-11-03 11:20:17+0700', 2, 'searching', 'system');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000009, '2026-11-03 11:21:51+0700', 3, 'accepted', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000009, '2026-11-03 11:24:42+0700', 4, 'arriving', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000009, '2026-11-03 11:31:50+0700', 5, 'picked_up', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000009, '2026-11-03 11:36:15+0700', 6, 'in_transit', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000009, '2026-11-03 11:49:57+0700', 7, 'completed', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000010, '2026-11-03 11:35:00+0700', 1, 'requested', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000010, '2026-11-03 11:35:17+0700', 2, 'searching', 'system');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000010, '2026-11-03 11:36:51+0700', 3, 'accepted', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000010, '2026-11-03 11:39:42+0700', 4, 'arriving', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000010, '2026-11-03 11:46:50+0700', 5, 'picked_up', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000010, '2026-11-03 11:51:15+0700', 6, 'in_transit', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000010, '2026-11-03 12:04:57+0700', 7, 'completed', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000011, '2026-11-03 17:45:00+0700', 1, 'requested', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000011, '2026-11-03 17:45:17+0700', 2, 'searching', 'system');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000011, '2026-11-03 17:46:51+0700', 3, 'accepted', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000011, '2026-11-03 17:49:42+0700', 4, 'arriving', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000011, '2026-11-03 17:56:50+0700', 5, 'picked_up', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000011, '2026-11-03 18:01:15+0700', 6, 'in_transit', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000011, '2026-11-03 18:14:57+0700', 7, 'completed', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000012, '2026-11-03 18:20:00+0700', 1, 'requested', 'customer');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000012, '2026-11-03 18:20:17+0700', 2, 'searching', 'system');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000012, '2026-11-03 18:21:51+0700', 3, 'accepted', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000012, '2026-11-03 18:24:42+0700', 4, 'arriving', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000012, '2026-11-03 18:31:50+0700', 5, 'picked_up', 'driver');
INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor) VALUES (07000000-0000-4000-8000-000000000012, '2026-11-03 18:36:15+0700', 6, 'in_transit', 'driver');

-- ## Đếm số dòng mỗi bảng (bảng nhỏ nên COUNT(*) chấp nhận được; với bảng thật thì không — CS-006)
SELECT (text)'trips' AS bang, COUNT(*) AS so_dong FROM trips;
SELECT (text)'trips_by_customer' AS bang, COUNT(*) AS so_dong FROM trips_by_customer;
SELECT (text)'trips_by_driver_day' AS bang, COUNT(*) AS so_dong FROM trips_by_driver_day;
SELECT (text)'trips_by_city_hour' AS bang, COUNT(*) AS so_dong FROM trips_by_city_hour;
SELECT (text)'trip_events_by_trip' AS bang, COUNT(*) AS so_dong FROM trip_events_by_trip;
