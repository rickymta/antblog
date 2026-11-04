-- Lược đồ tối giản của lab, dựa trên các bảng của hồ sơ dự án Tới Nơi.
-- Khác hồ sơ: id là text (để test đọc dễ: 'C001', 'D101'), thành phố khoá bằng mã ('SG'),
-- thêm bảng quotes/vouchers/idempotency_keys mà bài hồ sơ chưa có.

CREATE TABLE cities (
  code text PRIMARY KEY,
  name text NOT NULL
);

CREATE TABLE customers (
  id         text PRIMARY KEY,
  phone      text NOT NULL UNIQUE,
  full_name  text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE drivers (
  id           text PRIMARY KEY,
  full_name    text NOT NULL,
  city_code    text NOT NULL REFERENCES cities (code),
  vehicle_type text NOT NULL CHECK (vehicle_type IN ('bike', 'car')),
  status       text NOT NULL DEFAULT 'offline' CHECK (status IN ('offline', 'online', 'busy'))
);

CREATE TABLE vouchers (
  code                 text PRIMARY KEY,
  percent              integer NOT NULL CHECK (percent BETWEEN 1 AND 50),
  max_discount         integer NOT NULL CHECK (max_discount >= 0),
  min_amount           integer NOT NULL CHECK (min_amount >= 0),
  kinds                text[] NOT NULL DEFAULT '{}',
  city_codes           text[] NOT NULL DEFAULT '{}',
  valid_from           date NOT NULL,
  valid_to             date NOT NULL,
  per_customer_per_day integer NOT NULL DEFAULT 1 CHECK (per_customer_per_day >= 1)
);

CREATE TABLE quotes (
  id              text PRIMARY KEY,
  customer_id     text NOT NULL REFERENCES customers (id),
  pricing_version text NOT NULL,
  details         jsonb NOT NULL,             -- ảnh chụp toàn bộ bảng giá
  created_at      timestamptz NOT NULL,
  expires_at      timestamptz NOT NULL,
  used_by_trip_id text
);

CREATE TABLE trips (
  id               text PRIMARY KEY,
  quote_id         text NOT NULL UNIQUE REFERENCES quotes (id),
  pricing_version  text NOT NULL,
  kind             text NOT NULL CHECK (kind IN ('ride', 'food', 'parcel')),
  city_code        text NOT NULL REFERENCES cities (code),
  customer_id      text NOT NULL REFERENCES customers (id),
  driver_id        text REFERENCES drivers (id),
  vehicle_type     text NOT NULL CHECK (vehicle_type IN ('bike', 'car')),
  status           text NOT NULL DEFAULT 'requested' CHECK (status IN
                     ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed', 'cancelled')),
  distance_km      numeric(5, 1) NOT NULL,
  parcel_weight_kg numeric(4, 1),
  items            jsonb,
  items_subtotal   integer,
  delivery_fee     integer,
  peak             boolean NOT NULL,
  raining          boolean NOT NULL,
  voucher_code     text,
  discount         numeric(12, 3) NOT NULL DEFAULT 0,
  fare             integer NOT NULL,             -- đồng
  fare_breakdown   jsonb NOT NULL,
  payment_method   text NOT NULL CHECK (payment_method IN ('cash', 'vixanh')),
  payment_status   text NOT NULL CHECK (payment_status IN ('cash', 'pending', 'paid', 'failed', 'none')),
  requested_at     timestamptz NOT NULL,
  searching_since  timestamptz,
  accepted_at      timestamptz,
  eta_minutes      integer,
  completed_at     timestamptz,
  cancelled_at     timestamptz,
  cancelled_by     text CHECK (cancelled_by IN ('customer', 'system')),
  cancel_reason    text,
  cancellation_fee integer NOT NULL DEFAULT 0,
  updated_at       timestamptz NOT NULL
);
CREATE INDEX ix_trips_city_status       ON trips (city_code, status, requested_at);
CREATE INDEX ix_trips_customer_requested ON trips (customer_id, requested_at DESC);
CREATE INDEX ix_trips_driver_id          ON trips (driver_id);

CREATE TABLE trip_events (
  id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  trip_id     text NOT NULL REFERENCES trips (id),
  status      text NOT NULL,
  occurred_at timestamptz NOT NULL
);
CREATE INDEX ix_trip_events_trip ON trip_events (trip_id, occurred_at);

CREATE TABLE idempotency_keys (
  scope       text NOT NULL,
  key         text NOT NULL,
  fingerprint text NOT NULL,
  status_code integer,                         -- NULL = đang xử lý
  body        jsonb,
  created_at  timestamptz NOT NULL,
  PRIMARY KEY (scope, key)
);

INSERT INTO cities (code, name) VALUES
  ('SG', 'TP. Hồ Chí Minh'), ('HN', 'Hà Nội'), ('DN', 'Đà Nẵng'),
  ('HP', 'Hải Phòng'), ('CT', 'Cần Thơ'), ('NT', 'Nha Trang');
