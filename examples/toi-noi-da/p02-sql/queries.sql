-- Q11: grains
SELECT (SELECT COUNT(*) FROM da_trips) AS trips,
       (SELECT COUNT(*) FROM da_events) AS events,
       (SELECT COUNT(*) FROM da_payments) AS payment_attempts;

-- Q12: explicit scope and deterministic order
SELECT trip_id, status, accepted_at
FROM da_trips
WHERE kind='ride' AND city_id=1
  AND accepted_at >= TIMESTAMPTZ '2026-10-05 00:00+07'
  AND accepted_at < TIMESTAMPTZ '2026-10-12 00:00+07'
ORDER BY accepted_at, trip_id;

-- Q13: sums do not imply cash or revenue
SELECT status, COUNT(*) AS trips, SUM(fare_vnd) AS quoted_vnd
FROM da_cohort GROUP BY status ORDER BY status;

-- Q14: intentionally wrong grain; T01 yields four joined rows
SELECT c.trip_id, COUNT(*) AS joined_rows, SUM(p.amount_vnd) AS repeated_amount
FROM da_cohort c JOIN da_payments p USING (trip_id)
JOIN da_tickets t USING (trip_id)
GROUP BY c.trip_id ORDER BY c.trip_id;
-- Correct: one row per trip, existence rather than a multiplying join
SELECT c.trip_id, EXISTS (
  SELECT 1 FROM da_tickets t WHERE t.trip_id=c.trip_id
) AS has_ticket FROM da_cohort c ORDER BY c.trip_id;

-- Q15: unknown outcomes stay in the cohort
SELECT COUNT(*) AS accepted,
       COUNT(*) FILTER (WHERE outcome='cancelled_before_pickup') AS cancelled,
       COUNT(*) FILTER (WHERE outcome='unknown') AS unknown,
       ROUND(100.0*COUNT(*) FILTER (WHERE outcome='cancelled_before_pickup')
             /NULLIF(COUNT(*),0),2) AS confirmed_cancel_pct
FROM da_classified;

-- Q16: calendar date is explicit, independent of session timezone
SELECT trip_id, (accepted_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS accepted_day,
       (cancelled_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS cancelled_day
FROM da_cohort WHERE trip_id IN ('T07','T08') ORDER BY trip_id;

-- Q17: isolate each step and preserve the denominator
WITH scoped AS (SELECT * FROM da_classified), counts AS (
  SELECT COUNT(*) AS n,
    COUNT(*) FILTER (WHERE outcome='cancelled_before_pickup') AS cancelled,
    COUNT(*) FILTER (WHERE outcome='unknown') AS unresolved
  FROM scoped
)
SELECT n, cancelled, unresolved,
       ROUND(100.0*cancelled/NULLIF(n,0),2) AS lower_pct,
       ROUND(100.0*(cancelled+unresolved)/NULLIF(n,0),2) AS upper_pct
FROM counts;

-- Q18: all seven days; the date spine also supports days with zero accepted trips
WITH days AS (
  SELECT DATE '2026-10-05'+i AS day FROM generate_series(0,6) AS g(i)
), daily AS (
  SELECT (accepted_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS day,
         COUNT(*) AS n FROM da_cohort GROUP BY 1
), filled AS (
  SELECT d.day, COALESCE(x.n,0) AS n FROM days d LEFT JOIN daily x USING(day)
)
SELECT day,n,
       SUM(n) OVER (ORDER BY day ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS cumulative,
       LAG(n) OVER (ORDER BY day) AS previous_day
FROM filled ORDER BY day;

-- Q19: cohort cash through snapshot, not accounting revenue
SELECT SUM(fare_vnd) AS quoted_vnd,
       SUM(fare_vnd) FILTER (WHERE status='completed') AS completed_quoted_vnd,
       SUM(captured_vnd) AS captured_vnd, SUM(refunded_vnd) AS refunded_vnd,
       SUM(net_cash_vnd) AS net_cash_vnd FROM da_cash;

-- Q20: daily operational report; left-side date spine preserves zero days
WITH days AS (
  SELECT DATE '2026-10-05'+i AS day FROM generate_series(0,6) AS g(i)
), daily AS (
  SELECT (c.accepted_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS day,
    COUNT(*) AS accepted,
    COUNT(*) FILTER (WHERE c.outcome='cancelled_before_pickup') AS cancelled,
    COUNT(*) FILTER (WHERE c.outcome='unknown') AS unknown,
    SUM(m.net_cash_vnd) AS net_cash_vnd
  FROM da_classified c JOIN da_cash m USING(trip_id) GROUP BY 1
)
SELECT d.day,COALESCE(x.accepted,0) AS accepted,
       COALESCE(x.cancelled,0) AS cancelled,COALESCE(x.unknown,0) AS unknown,
       ROUND(100.0*x.cancelled/NULLIF(x.accepted,0),2) AS confirmed_cancel_pct,
       COALESCE(x.net_cash_vnd,0) AS cohort_net_cash_vnd
FROM days d LEFT JOIN daily x USING(day) ORDER BY d.day;
