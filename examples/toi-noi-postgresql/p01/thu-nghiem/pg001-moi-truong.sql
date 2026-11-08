SELECT current_database() AS database_hien_tai, current_user AS nguoi_dung, current_schema() AS schema_hien_tai;

SELECT datname FROM pg_database ORDER BY datname;

SELECT schema_name FROM information_schema.schemata ORDER BY schema_name;

SELECT table_name FROM information_schema.tables WHERE table_schema = 'toinoi' ORDER BY table_name;

SHOW search_path;

SELECT count(*) AS so_chuyen FROM toinoi.trips;

SELECT count(*) AS so_chuyen FROM trips;

SET search_path TO public;

-- expect-error: 42P01
SELECT count(*) FROM trips;

SELECT count(*) AS so_chuyen FROM toinoi.trips;

SET search_path TO toinoi;

SELECT code, kind, status, fare FROM trips WHERE code = 100004;

SELECT
  (SELECT count(*) FROM cities)      AS cities,
  (SELECT count(*) FROM customers)   AS customers,
  (SELECT count(*) FROM drivers)     AS drivers,
  (SELECT count(*) FROM merchants)   AS merchants,
  (SELECT count(*) FROM trips)       AS trips,
  (SELECT count(*) FROM trip_events) AS trip_events;
