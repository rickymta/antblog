SET search_path TO toinoi;
ALTER TABLE merchants ADD CONSTRAINT uq_merchants_id_city UNIQUE (id, city_id);
ALTER TABLE trips
  ADD CONSTRAINT fk_trips_merchant_city FOREIGN KEY (merchant_id, city_id)
  REFERENCES merchants (id, city_id) NOT VALID;
ALTER TABLE trips VALIDATE CONSTRAINT fk_trips_merchant_city;
-- expect-error: 23503 fk_trips_merchant_city
INSERT INTO trips (id, kind, city_id, customer_id, merchant_id, status, fare, requested_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000040', 'food', 2, 'c0000000-0000-4000-8000-000000000001',
        'e0000000-0000-4000-8000-000000000001', 'requested', 40000,
        '2026-11-03 09:00:00+07', '2026-11-03 09:00:00+07');
