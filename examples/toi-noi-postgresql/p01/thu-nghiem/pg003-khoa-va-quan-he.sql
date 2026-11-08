SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Hai chuyến cùng giờ đặt: thời điểm không nhận diện được một chuyến.
SELECT code, kind, city_id, status, requested_at
FROM trips
WHERE requested_at = '2026-11-02 12:00:00+07'
ORDER BY code;

-- Bước 1: tìm chuyến bằng mã chuyến (khoá UNIQUE).
SELECT id, code, customer_id, driver_id, merchant_id
FROM trips
WHERE code = 100004;

-- Bước 2: đi theo customer_id tới đúng một khách.
SELECT id, phone, full_name
FROM customers
WHERE id = 'c0000000-0000-4000-8000-000000000005';

-- Bước 3: đi theo driver_id tới đúng một tài xế.
SELECT id, code, full_name, city_id
FROM drivers
WHERE id = 'd0000000-0000-4000-8000-000000008005';

-- Ràng buộc khoá của bảng trips, đọc từ catalog.
SELECT conname, contype, pg_get_constraintdef(oid) AS dinh_nghia
FROM pg_constraint
WHERE conrelid = 'toinoi.trips'::regclass AND contype IN ('p', 'u', 'f')
ORDER BY array_position(ARRAY['p', 'u', 'f'], contype::text), conname;

-- Chuyến chưa có tài xế: driver_id NULL, FK không kiểm.
SELECT code, status, driver_id
FROM trips
WHERE driver_id IS NULL
ORDER BY code;

-- Tài xế không tồn tại.
-- expect-error: 23503 fk_trips_driver
UPDATE trips
SET driver_id = 'd0000000-0000-4000-8000-000000008099'
WHERE code = 100012;

-- FK chỉ kiểm "có tồn tại": gán tài xế Hà Nội (8003) cho chuyến ở TP.HCM vẫn được.
BEGIN;
-- expect-rows: 1
UPDATE trips
SET driver_id = 'd0000000-0000-4000-8000-000000008003'
WHERE code = 100012
RETURNING code, city_id, driver_id;
ROLLBACK;

-- Một khách, nhiều chuyến.
SELECT code, kind, status
FROM trips
WHERE customer_id = 'c0000000-0000-4000-8000-000000000001'
ORDER BY code;
