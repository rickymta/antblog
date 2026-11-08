SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- E1. UPDATE 0: vì sao? Đọc lại dòng theo id để phân biệt "không có chuyến" với "sai trạng thái".
-- expect-rows: 0
UPDATE trips
SET status = 'arriving', updated_at = '2026-11-03 09:40:00+07'
WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'accepted'
RETURNING code, status;

-- expect-rows: 1
SELECT code, status, driver_id
FROM trips
WHERE id = 'b0000000-0000-4000-8000-000000000001';

-- expect-rows: 0
SELECT code, status
FROM trips
WHERE id = 'b0000000-0000-4000-8000-000000009999';

-- E2. Lost update: cùng câu nhận chuyến nhưng BỎ điều kiện status, chạy trên
-- chuyến 100012 (đã do tài xế 8001 nhận). Bọc BEGIN/ROLLBACK để không để lại dấu vết.
BEGIN;

-- expect-rows: 1
UPDATE trips
SET status      = 'accepted',
    driver_id   = 'd0000000-0000-4000-8000-000000008003',
    accepted_at = '2026-11-03 09:45:00+07',
    updated_at  = '2026-11-03 09:45:00+07'
WHERE code = 100012
RETURNING code, status, driver_id, accepted_at;

ROLLBACK;

-- expect-rows: 1
SELECT code, status, driver_id, accepted_at FROM trips WHERE code = 100012;

-- E3. UPDATE không có WHERE thì sửa mọi dòng (thử trong giao dịch rồi huỷ).
BEGIN;
-- expect-rows: 16
UPDATE trips SET fare = 0;
ROLLBACK;

-- expect-rows: 1
SELECT code, fare FROM trips WHERE fare = 0;
