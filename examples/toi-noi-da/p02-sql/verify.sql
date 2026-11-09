-- Fail fast on regressions that would change the teaching conclusions.
DO $$
BEGIN
  IF (SELECT COUNT(*) FROM da_trips) <> 14 THEN RAISE EXCEPTION 'fixture trip count'; END IF;
  IF (SELECT COUNT(*) FROM da_events) <> 32 THEN RAISE EXCEPTION 'fixture event count'; END IF;
  IF (SELECT COUNT(*) FROM da_cohort) <> 8 THEN RAISE EXCEPTION 'cohort boundaries'; END IF;
  IF (SELECT string_agg(trip_id,',' ORDER BY trip_id) FROM da_cohort)
      <> 'T01,T02,T03,T04,T05,T06,T07,T08' THEN RAISE EXCEPTION 'wrong cohort'; END IF;
  IF (SELECT COUNT(*) FROM da_classified WHERE outcome='cancelled_before_pickup') <> 3
     OR (SELECT COUNT(*) FROM da_classified WHERE outcome='unknown') <> 1
     OR (SELECT COUNT(*) FROM da_classified WHERE outcome='picked_up') <> 4
    THEN RAISE EXCEPTION 'outcome classes'; END IF;
  IF (SELECT COUNT(cancel_reason) FROM da_cohort WHERE status='cancelled') <> 2
    THEN RAISE EXCEPTION 'null reason'; END IF;
  IF (SELECT COUNT(*) FROM da_cohort c JOIN da_payments p USING(trip_id)
       JOIN da_tickets t USING(trip_id) WHERE c.trip_id='T01') <> 4
    THEN RAISE EXCEPTION 'join fanout'; END IF;
  IF (SELECT COUNT(*) FROM da_cash) <> 8
     OR (SELECT SUM(fare_vnd) FROM da_cash) <> 730000
     OR (SELECT SUM(captured_vnd) FROM da_cash) <> 450000
     OR (SELECT SUM(refunded_vnd) FROM da_cash) <> 50000
     OR (SELECT SUM(net_cash_vnd) FROM da_cash) <> 400000
    THEN RAISE EXCEPTION 'cohort cash reconciliation'; END IF;
  IF (SELECT SUM(fare_vnd) FROM da_cohort WHERE status='completed') <> 430000
    THEN RAISE EXCEPTION 'completed fare'; END IF;
  IF EXISTS (SELECT 1 FROM da_events e JOIN da_trips t USING(trip_id)
             WHERE e.occurred_at > TIMESTAMPTZ '2026-10-13 00:00+07')
    THEN RAISE EXCEPTION 'snapshot cutoff'; END IF;
  IF EXISTS (SELECT p.payment_id FROM da_payments p JOIN da_refunds r USING(payment_id)
             WHERE r.status='succeeded' GROUP BY p.payment_id,p.amount_vnd,p.status
             HAVING SUM(r.amount_vnd)>p.amount_vnd OR p.status<>'succeeded')
    THEN RAISE EXCEPTION 'refund invariant'; END IF;
END $$;

-- Invariance to session timezone: timestamps and display dates are explicit.
SET TIME ZONE 'UTC';
DO $$ BEGIN
  IF (SELECT COUNT(*) FROM da_cohort) <> 8
    OR (SELECT (accepted_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date
        FROM da_trips WHERE trip_id='T07') <> DATE '2026-10-11'
    THEN RAISE EXCEPTION 'timezone dependency'; END IF;
END $$;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Meaningful perturbations: no unknown -> fabricated zero, no duplicated source key,
-- no classifying an incomplete history as a confirmed cancellation.
BEGIN;
UPDATE da_trips SET history_complete=false WHERE trip_id='T02';
DO $$ BEGIN
  IF (SELECT outcome FROM da_classified WHERE trip_id='T02') <> 'unknown'
    THEN RAISE EXCEPTION 'incomplete history treated as known'; END IF;
END $$;
ROLLBACK;
BEGIN;
DELETE FROM da_events;
DELETE FROM da_tickets;
DELETE FROM da_refunds;
DELETE FROM da_payments;
DELETE FROM da_trips;
DO $$ DECLARE n numeric; BEGIN
  SELECT 100.0*COUNT(*) FILTER (WHERE outcome='cancelled_before_pickup')
    /NULLIF(COUNT(*),0) INTO n FROM da_classified;
  IF n IS NOT NULL THEN RAISE EXCEPTION 'empty cohort must be undefined'; END IF;
END $$;
ROLLBACK;
SELECT 'P02 assertions passed' AS verification;
