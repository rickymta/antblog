-- P01 · 01-schema: các bảng của mô hình lịch sử chuyến (CS-003, CS-007, CS-008, CS-009).
-- Mỗi bảng phục vụ MỘT truy vấn. Tên bảng nói luôn truy vấn đó: <thứ cần lấy>_by_<khoá tra cứu>.
USE toinoi;

-- ## Q1 · Tra một chuyến theo mã chuyến
CREATE TABLE trips (
  trip_id      uuid,
  service      text,        -- 'ride' | 'food' | 'parcel'
  city_id      int,
  customer_id  uuid,
  driver_id    uuid,
  status       text,
  requested_at timestamp,
  pickup       text,
  dropoff      text,
  fare_vnd     bigint,      -- tiền là số nguyên đồng, không dùng float (CS-004)
  PRIMARY KEY (trip_id)
);

-- ## Q2 · Lịch sử chuyến của một khách, mới nhất trước
CREATE TABLE trips_by_customer (
  customer_id  uuid,
  requested_at timestamp,
  trip_id      uuid,
  service      text,
  status       text,
  pickup       text,
  dropoff      text,
  fare_vnd     bigint,
  driver_id    uuid,
  PRIMARY KEY ((customer_id), requested_at, trip_id)
) WITH CLUSTERING ORDER BY (requested_at DESC, trip_id ASC)
  AND comment = 'Q2: lịch sử chuyến theo khách, mới nhất trước';

-- ## Q3 · Chuyến của một tài xế trong một ngày (giờ Việt Nam)
CREATE TABLE trips_by_driver_day (
  driver_id    uuid,
  day          date,        -- ngày theo giờ Việt Nam, do ứng dụng tính
  requested_at timestamp,
  trip_id      uuid,
  customer_id  uuid,
  service      text,
  status       text,
  fare_vnd     bigint,
  PRIMARY KEY ((driver_id, day), requested_at, trip_id)
) WITH CLUSTERING ORDER BY (requested_at DESC, trip_id ASC)
  AND comment = 'Q3: chuyến của tài xế theo ngày';

-- ## Q4 · Chuyến mới tạo trong một thành phố theo từng giờ (màn hình điều phối)
CREATE TABLE trips_by_city_hour (
  city_id      int,
  hour_bucket  timestamp,   -- requested_at làm tròn xuống đầu giờ (giờ Việt Nam)
  requested_at timestamp,
  trip_id      uuid,
  service      text,
  status       text,
  fare_vnd     bigint,
  PRIMARY KEY ((city_id, hour_bucket), requested_at, trip_id)
) WITH CLUSTERING ORDER BY (requested_at ASC, trip_id ASC)
  AND comment = 'Q4: chuyến theo thành phố và giờ';

-- ## Q5 · Dòng thời gian sự kiện của một chuyến, cũ nhất trước
CREATE TABLE trip_events_by_trip (
  trip_id   uuid,
  event_at  timestamp,
  event_seq int,           -- thứ tự trong chuyến; phá hoà khi hai sự kiện cùng mili giây
  status    text,
  actor     text,          -- 'customer' | 'driver' | 'system'
  note      text,
  PRIMARY KEY ((trip_id), event_at, event_seq)
) WITH comment = 'Q5: dòng thời gian sự kiện của chuyến';

-- ## Các bảng đã tạo và khoá của chúng
SELECT table_name, column_name, kind, position, clustering_order, type
FROM system_schema.columns
WHERE keyspace_name = 'toinoi'
  AND table_name IN ('trips', 'trips_by_customer', 'trips_by_driver_day', 'trips_by_city_hour', 'trip_events_by_trip');
