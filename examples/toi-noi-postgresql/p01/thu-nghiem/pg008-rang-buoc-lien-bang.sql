SET search_path TO toinoi;
ALTER TABLE merchants ADD CONSTRAINT uq_merchants_id_city UNIQUE (id, city_id);
ALTER TABLE trips ADD CONSTRAINT fk_trips_merchant_city
  FOREIGN KEY (merchant_id, city_id) REFERENCES merchants (id, city_id);

-- Đơn đồ ăn ở SG (city 1) đặt quán Bún Chả Ngõ Mẫu ở Hà Nội (city 2)
-- expect-error: 23503 fk_trips_merchant_city
INSERT INTO trips (kind, city_id, customer_id, merchant_id, fare, requested_at, updated_at)
VALUES ('food', 1, 'c0000000-0000-4000-8000-000000000001',
        'e0000000-0000-4000-8000-000000000002', 50000,
        '2026-11-03 10:00:00+07', '2026-11-03 10:00:00+07');

UPDATE drivers SET city_id = 2 WHERE code = 8001;

SELECT code, status, city_id FROM trips
WHERE driver_id = 'd0000000-0000-4000-8000-000000008001'
ORDER BY code;

SELECT code, fare, requested_at FROM trips WHERE code IN (100000, 100008) ORDER BY code;

CREATE TABLE fare_rule_versions (
  id         integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  city_id    smallint NOT NULL REFERENCES cities (id),
  kind       text NOT NULL CHECK (kind IN ('ride', 'food', 'parcel')),
  base_fare  integer NOT NULL CHECK (base_fare >= 0),
  per_km     integer NOT NULL CHECK (per_km >= 0),
  valid_from timestamptz NOT NULL,
  UNIQUE (city_id, kind, valid_from)
);

INSERT INTO fare_rule_versions (city_id, kind, base_fare, per_km, valid_from) VALUES
  (1, 'ride', 12000, 4500, '2026-10-01 00:00+07'),
  (1, 'ride', 13000, 5000, '2026-11-03 00:00+07');

-- Phiên bản áp dụng cho chuyến đặt 07:58 ngày 02/11 ở SG
SELECT id, base_fare, per_km, valid_from
FROM fare_rule_versions
WHERE city_id = 1 AND kind = 'ride' AND valid_from <= '2026-11-02 07:58+07'
ORDER BY valid_from DESC
LIMIT 1;

