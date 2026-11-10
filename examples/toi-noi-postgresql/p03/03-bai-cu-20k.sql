-- P03 · 03-bai-cu-20k.sql — dựng lại bộ dữ liệu mà HAI BÀI ĐÃ CÓ dùng để đo:
--   * PG-022 `page-don-vi-doc-ghi`       ("bộ dữ liệu 20.000 chuyến của bài về heap và ctid")
--   * PG-024 `heap-ctid-va-primary-key`  ("bộ dữ liệu mẫu thu nhỏ còn 20.000 chuyến")
--
-- Hai bài đó chạy lược đồ LÕI của hồ sơ Tới Nơi (`du-an-mau-toi-noi`, mục "Dựng dữ
-- liệu mẫu trên máy": ràng buộc KHÔNG đặt tên ⇒ khoá chính tên `trips_pkey`, không
-- có pickup_area_id, không có CHECK nghiệp vụ) với bộ sinh dữ liệu của hồ sơ thu
-- từ 200.000 xuống 20.000 chuyến. File này làm đúng như vậy trong schema RIÊNG
-- `bai_cu` (không đụng schema toinoi), chỉ thay hai chỗ để TẤT ĐỊNH:
--   1. random()            → số đều từ md5 (toinoi.p03_u);
--   2. gen_random_uuid()   → uuid dạng v4 suy từ md5 (toinoi.p03_uuid) — vẫn "ngẫu nhiên"
--                            theo nghĩa không thẳng hàng với thứ tự chèn;
--   3. now()               → mốc chụp cố định 2026-10-31 01:00 +07.
-- Ngưỡng 'searching' của hồ sơ (200 chuyến cuối trên 200.000) thu theo tỉ lệ còn
-- 20 chuyến cuối. Bảng 20.000 dòng < 30.000 dòng mẫu của ANALYZE ⇒ ANALYZE đọc
-- MỌI dòng ⇒ thống kê (correlation, n_distinct…) TẤT ĐỊNH, so khớp được.
--
-- Bài 22 và 24 dùng schema này để KIỂM CHỨNG từng con số của hai bài cũ.

DROP SCHEMA IF EXISTS bai_cu CASCADE;
CREATE SCHEMA bai_cu;
SET search_path TO bai_cu, toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Lược đồ lõi của hồ sơ, chép nguyên văn (trừ DEFAULT gen_random_uuid()/now()
-- vẫn giữ — chỉ dữ liệu nạp dưới đây là tất định).
CREATE TABLE cities (
  id    smallint PRIMARY KEY,
  code  text NOT NULL UNIQUE,
  name  text NOT NULL
);

CREATE TABLE customers (
  id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  phone      text NOT NULL UNIQUE,
  full_name  text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE drivers (
  id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  code         integer NOT NULL UNIQUE,
  full_name    text NOT NULL,
  city_id      smallint NOT NULL REFERENCES cities (id),
  vehicle_type text NOT NULL CHECK (vehicle_type IN ('bike', 'car')),
  status       text NOT NULL DEFAULT 'offline' CHECK (status IN ('offline', 'online', 'busy')),
  created_at   timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE merchants (
  id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  city_id    smallint NOT NULL REFERENCES cities (id),
  name       text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE trips (
  id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  code         bigint GENERATED ALWAYS AS IDENTITY (START WITH 100000),
  kind         text NOT NULL CHECK (kind IN ('ride', 'food', 'parcel')),
  city_id      smallint NOT NULL REFERENCES cities (id),
  customer_id  uuid NOT NULL REFERENCES customers (id),
  driver_id    uuid REFERENCES drivers (id),
  merchant_id  uuid REFERENCES merchants (id),
  status       text NOT NULL DEFAULT 'requested' CHECK (status IN
                 ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed', 'cancelled')),
  fare         integer NOT NULL,              -- đồng
  requested_at timestamptz NOT NULL DEFAULT now(),
  accepted_at  timestamptz,
  completed_at timestamptz,
  updated_at   timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT uq_trips_code UNIQUE (code)
);
CREATE INDEX ix_trips_city_requested     ON trips (city_id, requested_at);
CREATE INDEX ix_trips_customer_requested ON trips (customer_id, requested_at DESC);
CREATE INDEX ix_trips_driver_id          ON trips (driver_id);

-- Dữ liệu của hồ sơ: 6 thành phố, 50.000 khách, 5.000 tài xế, 2.000 quán.
-- expect-rows: 6
INSERT INTO cities (id, code, name) VALUES
  (1, 'SG', 'TP. Hồ Chí Minh'), (2, 'HN', 'Hà Nội'), (3, 'DN', 'Đà Nẵng'),
  (4, 'HP', 'Hải Phòng'), (5, 'CT', 'Cần Thơ'), (6, 'NT', 'Nha Trang');

-- expect-rows: 50000
INSERT INTO customers (id, phone, full_name, created_at)
SELECT p03_uuid('bc-kh', g), '09' || lpad(g::text, 8, '0'), 'Khách ' || g, TIMESTAMPTZ '2026-10-01 00:00+07'
FROM generate_series(1, 50000) AS g;

-- expect-rows: 5000
INSERT INTO drivers (id, code, full_name, city_id, vehicle_type, status, created_at)
SELECT p03_uuid('bc-tx', g), 8000 + g, 'Tài xế ' || g, 1 + g % 6,
       CASE WHEN g % 4 = 0 THEN 'car' ELSE 'bike' END, 'online', TIMESTAMPTZ '2026-10-01 00:00+07'
FROM generate_series(1, 5000) AS g;

-- expect-rows: 2000
INSERT INTO merchants (id, city_id, name, created_at)
SELECT p03_uuid('bc-q', g), 1 + g % 6, 'Quán ' || g, TIMESTAMPTZ '2026-10-01 00:00+07'
FROM generate_series(1, 2000) AS g;

-- 20.000 chuyến trải đều 30 ngày trước mốc chụp, chèn đúng thứ tự thời gian đặt
-- (y như câu INSERT của hồ sơ, chỉ thay random() bằng p03_u(md5(...), i)).
-- expect-rows: 20000
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
WITH s AS (
  SELECT g,
         TIMESTAMPTZ '2026-10-31 01:00+07' - interval '30 days' + (g / 20000.0) * interval '30 days' AS t,
         md5('bc-t' || g::text) AS h
  FROM generate_series(1, 20000) AS g
)
SELECT p03_uuid('bc-t', s.g),
       k.kind,
       1 + s.g % 6,
       p03_uuid('bc-kh', 1 + (p03_u(s.h, 1) * 49999)::int),
       CASE WHEN st.status = 'searching' THEN NULL ELSE p03_uuid('bc-tx', 1 + (p03_u(s.h, 2) * 4999)::int) END,
       CASE WHEN k.kind = 'food' THEN p03_uuid('bc-q', 1 + (p03_u(s.h, 3) * 1999)::int) END,
       st.status,
       (15 + (p03_u(md5('bc-f' || s.g::text), 0) * 150)::int) * 1000,
       s.t,
       CASE WHEN st.status <> 'searching' THEN s.t + interval '2 minutes' END,
       CASE WHEN st.status = 'completed' THEN s.t + interval '25 minutes' END,
       s.t
FROM s
CROSS JOIN LATERAL (SELECT CASE WHEN p03_u(s.h, 0) < 0.40 THEN 'ride'
                                WHEN p03_u(s.h, 0) < 0.85 THEN 'food'
                                ELSE 'parcel' END AS kind) AS k
CROSS JOIN LATERAL (SELECT CASE WHEN s.g > 19980 THEN 'searching'
                                WHEN p03_u(md5('bc-f' || s.g::text), 1) < 0.06 THEN 'cancelled'
                                ELSE 'completed' END AS status) AS st
ORDER BY s.g;

VACUUM (ANALYZE) bai_cu.cities, bai_cu.customers, bai_cu.drivers, bai_cu.merchants, bai_cu.trips;

-- expect-line: 6 | 50000 | 5000 | 2000 | 20000 | 18785 | 1195 | 20
SELECT (SELECT count(*) FROM bai_cu.cities) AS cities, (SELECT count(*) FROM bai_cu.customers) AS customers,
       (SELECT count(*) FROM bai_cu.drivers) AS drivers, (SELECT count(*) FROM bai_cu.merchants) AS merchants,
       (SELECT count(*) FROM bai_cu.trips) AS trips,
       (SELECT count(*) FROM bai_cu.trips WHERE status = 'completed') AS completed,
       (SELECT count(*) FROM bai_cu.trips WHERE status = 'cancelled') AS cancelled,
       (SELECT count(*) FROM bai_cu.trips WHERE status = 'searching') AS searching;
