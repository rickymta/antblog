-- Bảng Iceberg cho dữ liệu có upsert: trạng thái chuyến đổi nhiều lần trong vài chục phút.
-- Athena hỗ trợ MERGE INTO và time travel trên Iceberg; kiểm phiên bản engine của workgroup.
CREATE TABLE trips_iceberg (
  trip_id      string,
  city         string,
  status       string,
  fare         int,
  requested_at timestamp,
  updated_at   timestamp
)
PARTITIONED BY (day(requested_at), city)
LOCATION 's3://toinoi-lab-lake-123456789012/iceberg/trips/'
TBLPROPERTIES (
  'table_type'        = 'ICEBERG',
  'format'            = 'parquet',
  'write_compression' = 'zstd'
);

-- Upsert từ bảng thay đổi (CDC) trips_changes: có thì cập nhật, chưa có thì thêm.
MERGE INTO trips_iceberg AS t
USING trips_changes AS s
ON t.trip_id = s.trip_id
WHEN MATCHED THEN
  UPDATE SET status = s.status, fare = s.fare, updated_at = s.updated_at
WHEN NOT MATCHED THEN
  INSERT (trip_id, city, status, fare, requested_at, updated_at)
  VALUES (s.trip_id, s.city, s.status, s.fare, s.requested_at, s.updated_at);

-- Time travel: trạng thái bảng cách đây một giờ.
SELECT trip_id, status
FROM trips_iceberg FOR TIMESTAMP AS OF (current_timestamp - interval '1' hour)
WHERE trip_id = '0b6f0000-0000-4000-8000-000000000001';

-- Gộp file nhỏ và dọn snapshot, file mồ côi (Iceberg trên Athena engine v3).
OPTIMIZE trips_iceberg REWRITE DATA USING BIN_PACK;
VACUUM trips_iceberg;
