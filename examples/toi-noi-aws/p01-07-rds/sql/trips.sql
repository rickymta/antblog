-- Bản rút gọn của bảng trips (đầy đủ nằm trong hồ sơ Tới Nơi). Chạy lại được: bảng bị tạo lại từ đầu.
DROP TABLE IF EXISTS trips;
CREATE TABLE trips (
  id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  code         bigint GENERATED ALWAYS AS IDENTITY (START WITH 100000) UNIQUE,
  kind         text NOT NULL CHECK (kind IN ('ride', 'food', 'parcel')),
  city_code    text NOT NULL,
  status       text NOT NULL DEFAULT 'requested',
  fare         integer NOT NULL,
  requested_at timestamptz NOT NULL DEFAULT now(),
  updated_at   timestamptz NOT NULL DEFAULT now()
);

INSERT INTO trips (kind, city_code, status, fare) VALUES
  ('ride',   'HCM', 'completed', 25500),
  ('food',   'HCM', 'in_transit', 42000),
  ('parcel', 'HN',  'searching', 31000),
  ('ride',   'DN',  'accepted',  18000),
  ('food',   'HN',  'cancelled', 56000);

SELECT kind, status, count(*) AS trips, sum(fare) AS fare_total
FROM trips GROUP BY 1, 2 ORDER BY 1, 2;

-- Giá trị tham số đang hiệu lực từ parameter group:
SHOW log_min_duration_statement;
SHOW max_connections;
SHOW ssl;
