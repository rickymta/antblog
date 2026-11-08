CREATE SCHEMA phang;
SET search_path TO phang;

CREATE TABLE trip_flat (
  trip_code      bigint PRIMARY KEY,
  kind           text NOT NULL,
  status         text NOT NULL,
  fare           integer NOT NULL,
  requested_at   timestamptz NOT NULL,
  customer_phone text NOT NULL,
  customer_name  text NOT NULL,
  driver_code    integer,
  driver_name    text,
  city_code      text NOT NULL,
  city_name      text NOT NULL,
  merchant_name  text,
  status_history text
);

INSERT INTO trip_flat VALUES
  (100000, 'ride', 'completed', 45000, '2026-11-02 07:58+07', '0900000001', 'Nguyễn Văn An', 8001, 'Bùi Văn Khoa', 'SG', 'TP. Hồ Chí Minh', NULL,
   'requested,searching,accepted,arriving,picked_up,in_transit,completed'),
  (100002, 'food', 'completed', 62000, '2026-11-02 11:05+07', '0900000002', 'Trần Thị Bình', 8002, 'Hồ Thị Lan', 'SG', 'TP. Hồ Chí Minh', 'Cơm Tấm Phố Lab',
   'requested,searching,accepted,arriving,picked_up,in_transit,completed'),
  (100003, 'food', 'cancelled', 71000, '2026-11-02 12:00+07', '0900000003', 'Lê Hoàng Cường', 8003, 'Ngô Quang Minh', 'HN', 'Hà Nội', 'Bún Chả Ngõ Mẫu',
   'requested,searching,accepted,cancelled'),
  (100006, 'ride', 'cancelled', 52000, '2026-11-02 18:00+07', '0900000001', 'Nguyễn Văn An', NULL, NULL, 'SG', 'TP. Hồ Chí Minh', NULL,
   'requested,searching,cancelled'),
  (100012, 'ride', 'accepted', 39000, '2026-11-03 08:20+07', '0900000001', 'Nguyễn Văn An', 8001, 'Bùi Văn Khoa', 'SG', 'TP. Hồ Chí Minh', NULL,
   'requested,searching,accepted');

SELECT trip_code, customer_name, customer_phone, driver_name, city_name, merchant_name
FROM trip_flat
ORDER BY trip_code;

UPDATE trip_flat SET customer_phone = '0900000011' WHERE trip_code = 100012;

SELECT trip_code, customer_name, customer_phone
FROM trip_flat
WHERE customer_name = 'Nguyễn Văn An'
ORDER BY trip_code;

SET search_path TO toinoi;
UPDATE customers SET phone = '0900000011'
WHERE id = 'c0000000-0000-4000-8000-000000000001'
RETURNING full_name, phone;

SET search_path TO phang;
-- Lỗi cố ý. Dòng dưới là chỉ thị cho runner của lab; psql coi là chú thích.
-- expect-error: 23502 trip_code
INSERT INTO trip_flat (driver_code, driver_name, kind, status, fare, requested_at,
                       customer_phone, customer_name, city_code, city_name)
VALUES (8006, 'Trịnh Văn Phúc', 'ride', 'requested', 0, '2026-11-03 10:00+07', '?', '?', 'DN', 'Đà Nẵng');

DELETE FROM trip_flat WHERE status = 'cancelled'
RETURNING trip_code, customer_name, merchant_name;

SELECT trip_code, status_history
FROM trip_flat
WHERE status_history LIKE '%accepted%'
ORDER BY trip_code;

