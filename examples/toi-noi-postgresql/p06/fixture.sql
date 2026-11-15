-- P06 · fixture tất định cho phần mở rộng và tích hợp.
-- gps: điểm GPS theo giờ, 01/10–30/11/2026 UTC (1.464 điểm, 20 tài xế), phân vùng theo tháng.
-- gps_plain: cùng dữ liệu, bảng thường, để so sánh với bản phân vùng.
-- fares: sổ cước rút gọn, xuyên suốt replication, failover và capstone.
CREATE TABLE gps (
  driver_id int NOT NULL,
  seen_at timestamptz NOT NULL,
  attrs jsonb NOT NULL CHECK (jsonb_typeof(attrs) = 'object'),
  PRIMARY KEY (driver_id, seen_at)
) PARTITION BY RANGE (seen_at);
CREATE TABLE gps_oct PARTITION OF gps FOR VALUES FROM ('2026-10-01 UTC') TO ('2026-11-01 UTC');
CREATE TABLE gps_nov PARTITION OF gps FOR VALUES FROM ('2026-11-01 UTC') TO ('2026-12-01 UTC');
INSERT INTO gps
SELECT 1 + n % 20, '2026-10-01 UTC'::timestamptz + n * interval '1 hour',
       jsonb_build_object('source', 'phone', 'accuracy_m', 5 + n % 10)
FROM generate_series(0, 1463) n;
CREATE TABLE gps_plain (LIKE gps INCLUDING ALL);
INSERT INTO gps_plain SELECT * FROM gps;
ANALYZE gps;
ANALYZE gps_plain;

CREATE TABLE fares (
  id int PRIMARY KEY,
  city text NOT NULL,
  fare int NOT NULL CHECK (fare >= 0)  -- VND
);
INSERT INTO fares VALUES (1, 'HCM', 85000), (2, 'HN', 65000);
CREATE MATERIALIZED VIEW fare_report AS
  SELECT city, count(*) AS trips, sum(fare) AS total, now() AS refreshed_at
  FROM fares GROUP BY city;
