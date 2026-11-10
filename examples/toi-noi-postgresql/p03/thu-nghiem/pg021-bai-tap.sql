-- Thí nghiệm PG-021 «Đọc EXPLAIN từ truy vấn tìm chuyến» — lời giải ba bài tập.
-- Không có trong lab chính (p03/21-explain-tim-chuyen.sql). Chạy trên bản sao nền p03:
--   node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg021-bai-tap.sql
SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- Bài 1. "Chuyến tài xế 20007 nhận trong ngày 15/10". Không có index (driver_id,
-- requested_at): planner phải ghép hai index riêng. Lần chạy khi viết bài ra BitmapAnd
-- của ix_trips_driver_id (139 chuyến của tài xế) và ix_trips_city_requested (5.896
-- chuyến trong ngày, skip scan) ⇒ chỉ 2 page heap. Kế hoạch dựa trên ước lượng ⇒ có
-- thể khác ở lần ANALYZE khác (vd. Index Scan + Filter) — chỉ kiểm phần chắc chắn.
SELECT id AS tx FROM drivers WHERE code = 20007 \gset

-- expect-value: 2
SELECT count(*) FROM trips
WHERE driver_id = :'tx'
  AND requested_at >= TIMESTAMPTZ '2026-10-15 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-16 00:00+07';

-- lab: bien-dong
-- expect-match: Index Cond: \(driver_id =
-- expect-match: on trips \(actual rows=2\.00 loops=1\)
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, fare FROM trips
WHERE driver_id = :'tx'
  AND requested_at >= TIMESTAMPTZ '2026-10-15 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-16 00:00+07';

-- Số chuyến của tài xế này trên toàn bảng.
-- expect-value: 139
SELECT count(*) AS tong_chuyen_tai_xe FROM trips WHERE driver_id = :'tx';

-- ---------------------------------------------------------------------------
-- Bài 2. Phân tích câu UPDATE "cộng 1.000 đ cho 300 chuyến đang chạy" mà không để
-- lại thay đổi: BEGIN … EXPLAIN ANALYZE … ROLLBACK.
-- expect-value: 300
SELECT count(*) FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-10-31 00:00+07';

SELECT sum(fare) AS tong_truoc FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-10-31 00:00+07' \gset

BEGIN;

-- Nút gốc ModifyTable in actual rows=0 (nó không trả dòng lên trên); số dòng đã sửa
-- nằm ở nút con (actual rows=300).
-- expect-match: Update on trips \(actual rows=0\.00 loops=1\)
-- expect-match: \(actual rows=300\.00 loops=1\)
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
UPDATE trips SET fare = fare + 1000 WHERE requested_at >= TIMESTAMPTZ '2026-10-31 00:00+07';

-- Trong giao dịch, thay đổi ĐÃ xảy ra thật.
-- expect-value: t
SELECT sum(fare) = :tong_truoc + 300 * 1000 FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-10-31 00:00+07';

ROLLBACK;

-- expect-value: t
SELECT sum(fare) = :tong_truoc FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-10-31 00:00+07';

-- EXPLAIN không ANALYZE của câu INSERT: chỉ in kế hoạch, không thêm dòng.
-- expect-match: Insert on trips
EXPLAIN (COSTS OFF)
INSERT INTO trips (kind, city_id, customer_id, status, fare)
SELECT 'ride', 1, id, 'requested', 30000 FROM customers WHERE phone = '0910000500';

-- expect-value: 0
SELECT count(*) FROM trips WHERE requested_at > TIMESTAMPTZ '2026-10-31 01:00+07';

-- ---------------------------------------------------------------------------
-- Bài 3. Giả định "độc lập" của planner lệch bao nhiêu khi khu đón thuộc hẳn một
-- thành phố? Sài Gòn chiếm ~55 % chuyến, Đà Nẵng ~13 % ⇒ ước lượng thấp hơn thực tế
-- khoảng 1/0,55 ≈ 1,8 lần với SG và 1/0,13 ≈ 7,7 lần với DN. Một cặp điều kiện gần
-- như độc lập (thành phố × loại dịch vụ) thì ước lượng sát.
-- expect-range: 1.4 2.4
SELECT lech FROM uoc_luong($$SELECT * FROM trips WHERE city_id = 1 AND pickup_area_id = 111$$);

-- expect-range: 5 10
SELECT lech FROM uoc_luong($$SELECT * FROM trips WHERE city_id = 3 AND pickup_area_id = 311$$);

-- expect-range: 1 1.3
SELECT lech FROM uoc_luong($$SELECT * FROM trips WHERE city_id = 3 AND kind = 'parcel'$$);

-- Số thực tế (tất định) của ba câu.
-- lab: bien-dong
SELECT 'SG khu 111' AS cau, uoc_tinh, thuc_te, lech FROM uoc_luong($$SELECT * FROM trips WHERE city_id = 1 AND pickup_area_id = 111$$)
UNION ALL
SELECT 'DN khu 311', uoc_tinh, thuc_te, lech FROM uoc_luong($$SELECT * FROM trips WHERE city_id = 3 AND pickup_area_id = 311$$)
UNION ALL
SELECT 'DN parcel', uoc_tinh, thuc_te, lech FROM uoc_luong($$SELECT * FROM trips WHERE city_id = 3 AND kind = 'parcel'$$);
