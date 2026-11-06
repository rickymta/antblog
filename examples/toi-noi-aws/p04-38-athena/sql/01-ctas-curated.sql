-- CTAS: đọc JSON raw của một ngày, ghi ra Parquet nén ZSTD, partition theo dt và city.
-- Cột partition phải nằm CUỐI danh sách SELECT. external_location phải trống.
-- Giới hạn: một câu CTAS hoặc INSERT INTO của Athena chỉ ghi tối đa 100 partition; một ngày x 6 thành phố = 6 partition là an toàn.
-- Với dữ liệu nhiều ngày, chạy theo từng ngày (hoặc dùng Glue ETL) thay vì một câu lệnh khổng lồ.
CREATE TABLE trips_curated_ctas
WITH (
  format            = 'PARQUET',
  write_compression = 'ZSTD',
  external_location = 's3://toinoi-lab-lake-123456789012/curated/trips_ctas/',
  partitioned_by    = ARRAY['dt', 'city']
) AS
SELECT
  trip_id,
  code,
  kind,
  customer_id,
  customer_phone,
  driver_id,
  merchant_id,
  status,
  fare,
  CAST(from_iso8601_timestamp(requested_at) AS timestamp) AS requested_at,
  CAST(from_iso8601_timestamp(accepted_at)  AS timestamp) AS accepted_at,
  CAST(from_iso8601_timestamp(completed_at) AS timestamp) AS completed_at,
  substr(requested_at, 1, 10) AS dt,
  city
FROM trips_raw
WHERE substr(requested_at, 1, 10) = '2026-03-02';
