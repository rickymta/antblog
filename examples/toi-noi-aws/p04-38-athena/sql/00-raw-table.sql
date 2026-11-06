-- Bảng raw: JSON lines do Firehose (bài 35) hoặc DMS ghi vào s3://<lake>/raw/trips/.
-- Chạy trong workgroup toinoi-lab-analytics, database toinoi_lab_analytics.
-- Thay <account-id> bằng ID tài khoản lab của bạn (ở các bài khác là 123456789012).
CREATE EXTERNAL TABLE IF NOT EXISTS trips_raw (
  trip_id        string,
  code           bigint,
  kind           string,
  city           string,
  customer_id    string,
  customer_phone string,
  driver_id      string,
  merchant_id    string,
  status         string,
  fare           int,
  requested_at   string,
  accepted_at    string,
  completed_at   string
)
ROW FORMAT SERDE 'org.openx.data.jsonserde.JsonSerDe'
LOCATION 's3://toinoi-lab-lake-123456789012/raw/trips/';
