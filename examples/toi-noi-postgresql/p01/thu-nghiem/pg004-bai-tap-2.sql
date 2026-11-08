SET TIME ZONE 'UTC';
CREATE TEMP TABLE nap_thu AS SELECT '2026-11-03 08:00:00'::timestamptz AS requested_at;
SET TIME ZONE 'Asia/Ho_Chi_Minh';
SELECT requested_at FROM nap_thu;
