-- P02-LAB v1: synthetic analytical snapshot, not production schema or P01 data.
-- TEMP objects exist only in this connection. No application tables are modified.
SET TIME ZONE 'Asia/Ho_Chi_Minh';
CREATE TEMP TABLE da_trips (
  trip_id text PRIMARY KEY,
  kind text NOT NULL CHECK (kind IN ('ride','food','parcel')),
  city_id integer NOT NULL,
  status text NOT NULL CHECK (status IN ('searching','accepted','completed','cancelled')),
  fare_vnd bigint NOT NULL CHECK (fare_vnd >= 0),
  accepted_at timestamptz,
  picked_up_at timestamptz,
  completed_at timestamptz,
  cancelled_at timestamptz,
  cancel_reason text,
  history_complete boolean NOT NULL
);
INSERT INTO da_trips VALUES
('T01','ride',1,'completed',100000,'2026-10-05 07:00+07','2026-10-05 07:05+07','2026-10-05 07:30+07',NULL,NULL,true),
('T02','ride',1,'cancelled',80000,'2026-10-05 08:00+07',NULL,NULL,'2026-10-05 08:10+07','changed_plan',true),
('T03','ride',1,'completed',120000,'2026-10-06 09:00+07','2026-10-06 09:10+07','2026-10-06 09:30+07',NULL,NULL,true),
('T04','ride',1,'cancelled',90000,'2026-10-07 18:00+07',NULL,NULL,'2026-10-07 18:15+07',NULL,true),
('T05','ride',1,'accepted',70000,'2026-10-08 10:00+07',NULL,NULL,NULL,NULL,true),
('T06','ride',1,'completed',100000,'2026-10-09 17:00+07','2026-10-09 17:20+07','2026-10-09 17:45+07',NULL,NULL,true),
('T07','ride',1,'completed',110000,'2026-10-11 23:59:59+07','2026-10-12 00:15+07','2026-10-12 00:30+07',NULL,NULL,true),
('T08','ride',1,'cancelled',60000,'2026-10-10 07:00+07',NULL,NULL,'2026-10-12 00:05+07','changed_plan',true),
('T09','ride',1,'completed',50000,'2026-10-12 00:00+07','2026-10-12 00:05+07','2026-10-12 00:25+07',NULL,NULL,true),
('T10','ride',1,'completed',40000,'2026-10-04 23:59:59+07','2026-10-05 00:05+07','2026-10-05 00:20+07',NULL,NULL,true),
('T11','food',1,'completed',150000,'2026-10-06 12:00+07','2026-10-06 12:10+07','2026-10-06 12:40+07',NULL,NULL,true),
('T12','ride',2,'completed',200000,'2026-10-06 12:00+07','2026-10-06 12:10+07','2026-10-06 12:40+07',NULL,NULL,true),
('T13','ride',1,'searching',30000,NULL,NULL,NULL,NULL,NULL,true),
('T14','ride',1,'cancelled',40000,NULL,NULL,NULL,'2026-10-07 11:00+07','changed_plan',true);

CREATE TEMP TABLE da_events (
  event_id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  trip_id text NOT NULL REFERENCES da_trips(trip_id),
  status text NOT NULL,
  occurred_at timestamptz NOT NULL
);
-- This teaching snapshot explicitly supplies complete histories for these milestones.
INSERT INTO da_events (trip_id,status,occurred_at)
SELECT trip_id,'accepted',accepted_at FROM da_trips WHERE accepted_at IS NOT NULL
UNION ALL SELECT trip_id,'picked_up',picked_up_at FROM da_trips WHERE picked_up_at IS NOT NULL
UNION ALL SELECT trip_id,'completed',completed_at FROM da_trips WHERE completed_at IS NOT NULL
UNION ALL SELECT trip_id,'cancelled',cancelled_at FROM da_trips WHERE cancelled_at IS NOT NULL;

CREATE TEMP TABLE da_payments (
  payment_id text PRIMARY KEY,
  trip_id text NOT NULL REFERENCES da_trips(trip_id),
  status text NOT NULL CHECK (status IN ('succeeded','failed','pending')),
  amount_vnd bigint NOT NULL CHECK (amount_vnd > 0),
  settled_at timestamptz,
  CHECK ((status = 'succeeded') = (settled_at IS NOT NULL))
);
INSERT INTO da_payments VALUES
('P01F','T01','failed',100000,NULL),
('P01','T01','succeeded',100000,'2026-10-05 07:31+07'),
('P02','T02','succeeded',20000,'2026-10-05 08:01+07'),
('P03','T03','succeeded',120000,'2026-10-06 09:31+07'),
('P04','T04','pending',90000,NULL),
('P06','T06','succeeded',100000,'2026-10-09 17:46+07'),
('P07','T07','succeeded',110000,'2026-10-12 00:35+07'),
('P08','T08','failed',60000,NULL),
('P09','T09','succeeded',50000,'2026-10-12 00:26+07'),
('P10','T10','succeeded',40000,'2026-10-05 00:21+07');

CREATE TEMP TABLE da_refunds (
  refund_id text PRIMARY KEY,
  payment_id text NOT NULL REFERENCES da_payments(payment_id),
  status text NOT NULL CHECK (status IN ('succeeded','pending')),
  amount_vnd bigint NOT NULL CHECK (amount_vnd > 0),
  settled_at timestamptz,
  CHECK ((status = 'succeeded') = (settled_at IS NOT NULL))
);
INSERT INTO da_refunds VALUES
('R02','P02','succeeded',20000,'2026-10-05 09:00+07'),
('R03','P03','succeeded',30000,'2026-10-12 09:00+07'),
('R07','P07','pending',10000,NULL);

CREATE TEMP TABLE da_tickets (
  ticket_id text PRIMARY KEY,
  trip_id text NOT NULL REFERENCES da_trips(trip_id)
);
INSERT INTO da_tickets VALUES ('S01','T01'),('S02','T01'),('S03','T02'),('S04','T04'),('S05','T04');

CREATE TEMP VIEW da_cohort AS
SELECT * FROM da_trips
WHERE kind = 'ride' AND city_id = 1
  AND accepted_at >= TIMESTAMPTZ '2026-10-05 00:00:00+07'
  AND accepted_at <  TIMESTAMPTZ '2026-10-12 00:00:00+07';

CREATE TEMP VIEW da_classified AS
SELECT *, CASE
  WHEN NOT history_complete THEN 'unknown'
  WHEN status = 'cancelled' AND cancelled_at IS NOT NULL AND picked_up_at IS NULL
    THEN 'cancelled_before_pickup'
  WHEN picked_up_at IS NOT NULL THEN 'picked_up'
  ELSE 'unknown'
END AS outcome
FROM da_cohort;

-- Pre-aggregate each many-side before joining. Zero means no recorded settled flow
-- only because the fixture explicitly guarantees complete payment/refund exports.
CREATE TEMP VIEW da_cash AS
WITH captures AS (
  SELECT trip_id, SUM(amount_vnd) AS captured_vnd
  FROM da_payments WHERE status = 'succeeded' GROUP BY trip_id
), returns AS (
  SELECT p.trip_id, SUM(r.amount_vnd) AS refunded_vnd
  FROM da_refunds r JOIN da_payments p USING (payment_id)
  WHERE r.status = 'succeeded' GROUP BY p.trip_id
)
SELECT c.trip_id, c.fare_vnd, c.status,
       COALESCE(p.captured_vnd,0) AS captured_vnd,
       COALESCE(r.refunded_vnd,0) AS refunded_vnd,
       COALESCE(p.captured_vnd,0)-COALESCE(r.refunded_vnd,0) AS net_cash_vnd
FROM da_cohort c LEFT JOIN captures p USING (trip_id)
LEFT JOIN returns r USING (trip_id);
