SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';
-- expect-rows: 1
UPDATE trips
SET status = 'cancelled', updated_at = '2026-11-03 08:40:00+07'
WHERE code = 100013 AND status IN ('requested', 'searching')
RETURNING code, kind, status, updated_at;
-- expect-rows: 0
UPDATE trips
SET status = 'cancelled', updated_at = '2026-11-03 08:40:05+07'
WHERE code = 100013 AND status IN ('requested', 'searching')
RETURNING code, kind, status, updated_at;
-- Huỷ chuyến 100012 (accepted) theo cùng điều kiện: không được.
-- expect-rows: 0
UPDATE trips
SET status = 'cancelled', updated_at = '2026-11-03 08:41:00+07'
WHERE code = 100012 AND status IN ('requested', 'searching')
RETURNING code, status;
