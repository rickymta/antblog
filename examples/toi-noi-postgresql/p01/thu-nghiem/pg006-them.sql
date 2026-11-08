SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';
-- expect-error: 23514 ck_trips_assigned_has_driver
UPDATE trips SET status = 'accepted', updated_at = '2026-11-03 08:41:00+07'
WHERE code = 100013 AND status = 'searching';
-- expect-rows: 0
WITH done AS (
  UPDATE trips
  SET status       = 'completed',
      completed_at = '2026-11-03 09:31:00+07',
      updated_at   = '2026-11-03 09:31:00+07'
  WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'in_transit'
  RETURNING id, status, completed_at
)
INSERT INTO trip_events (trip_id, status, occurred_at)
SELECT id, status, completed_at FROM done
RETURNING id, status, occurred_at;
BEGIN;
-- expect-rows: 1
INSERT INTO trips (kind, city_id, customer_id, fare, requested_at, updated_at)
VALUES ('ride', 1, 'c0000000-0000-4000-8000-000000000001', 30000, '2026-11-03 10:00:00+07', '2026-11-03 10:00:00+07')
RETURNING code;
ROLLBACK;
