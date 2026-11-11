-- Thí nghiệm PG-040 (lời giải bài tập) · `postgresql-thuc-hanh-tranh-chap-nhan-chuyen`
--
-- Bài 1: code SAI của cách (a) chạy ở REPEATABLE READ (bản tất định của K040-e): người ghi
--        sau chờ rồi nhận 40001; thử lại toàn giao dịch ⇒ đọc thấy 'accepted' ⇒ thôi.
-- Bài 2: tài xế bấm "Nhận" HAI lần (mạng chập chờn, app gửi lại) với cách (b): lần hai 0 dòng —
--        phải phân biệt "chính tôi đã nhận" với "người khác đã nhận".
-- Bài 3: thêm UNIQUE (trip_id) cho trip_claims rồi chạy lại đúng lịch (a): ràng buộc chặn người
--        thứ hai, cả giao dịch của B (kể cả UPDATE ghi đè trips) bị huỷ ⇒ bất biến giữ.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- ===========================================================================
-- Bài 1. Cách (a) + REPEATABLE READ, chuyến 400101.

-- @A
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @A
-- expect-value: requested
SELECT status FROM trips WHERE code = 400101;

-- @B
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @B
-- expect-value: requested
SELECT status FROM trips WHERE code = 400101;

-- @A
-- expect-rows: 1
UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
                 accepted_at = now(), updated_at = now(), version = version + 1
WHERE code = 400101;

-- @B
-- Cùng dòng A đang giữ ⇒ chờ; A commit ⇒ dòng đã đổi sau snapshot của B ⇒ 40001 (không ghi đè).
-- expect-blocked
-- expect-error: 40001
UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009102',
                 accepted_at = now(), updated_at = now(), version = version + 1
WHERE code = 400101;

-- @A
INSERT INTO trip_claims (trip_id, driver_id, method)
SELECT id, 'd4000000-0000-4000-8000-000000009101', 'e_rr' FROM trips WHERE code = 400101;

-- @A
COMMIT;

-- @B
-- expect-match: ^ROLLBACK$
COMMIT;

-- @B
-- Thử lại TOÀN BỘ giao dịch (snapshot mới): đọc thấy 'accepted' ⇒ ứng dụng thôi, không ghi.
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @B
-- expect-value: accepted
SELECT status FROM trips WHERE code = 400101;

-- @B
COMMIT;

-- @C
-- expect-line: 400101 | 1 | 9101 | 9101
SELECT t.code, count(c.id) AS so_claim, max(dc.code) AS duoc_bao_nhan, dt.code AS trips_driver
FROM trips AS t LEFT JOIN trip_claims AS c ON c.trip_id = t.id
LEFT JOIN drivers AS dc ON dc.id = c.driver_id JOIN drivers AS dt ON dt.id = t.driver_id
WHERE t.code = 400101 GROUP BY t.code, dt.code;

-- ===========================================================================
-- Bài 2. Bấm hai lần với cách (b), chuyến 400105.

-- @A
-- expect-value: 9101
WITH nhan AS (
  UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
                   accepted_at = now(), updated_at = now(), version = version + 1
  WHERE code = 400105 AND status = 'requested'
  RETURNING id, driver_id)
INSERT INTO trip_claims (trip_id, driver_id, method)
SELECT id, driver_id, 'b_co_dieu_kien' FROM nhan
RETURNING (SELECT code FROM drivers WHERE id = driver_id);

-- @A
-- Lần bấm thứ hai của CHÍNH 9101: 0 dòng — nếu app chỉ nhìn số dòng sẽ báo nhầm "đã có người nhận".
-- expect-rows: 0
WITH nhan AS (
  UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
                   accepted_at = now(), updated_at = now(), version = version + 1
  WHERE code = 400105 AND status = 'requested'
  RETURNING id, driver_id)
INSERT INTO trip_claims (trip_id, driver_id, method)
SELECT id, driver_id, 'b_co_dieu_kien' FROM nhan
RETURNING (SELECT code FROM drivers WHERE id = driver_id);

-- @A
-- 0 dòng ⇒ đọc lại người giữ chuyến: là mình ⇒ trả lại "bạn đã nhận" (idempotent), không claim thêm.
-- expect-line: accepted | t
SELECT status, driver_id = 'd4000000-0000-4000-8000-000000009101' AS la_toi FROM trips WHERE code = 400105;

-- @B
-- expect-line: accepted | f
SELECT status, driver_id = 'd4000000-0000-4000-8000-000000009102' AS la_toi FROM trips WHERE code = 400105;

-- @C
-- expect-value: 1
SELECT count(*) FROM trip_claims AS c JOIN trips AS t ON t.id = c.trip_id WHERE t.code = 400105;

-- ===========================================================================
-- Bài 3. UNIQUE trên trip_claims rồi lặp lại đúng lịch (a), chuyến 400106.

-- @C
CREATE UNIQUE INDEX ux_trip_claims_trip ON trip_claims (trip_id);

-- @A
BEGIN;

-- @A
-- expect-value: requested
SELECT status FROM trips WHERE code = 400106;

-- @B
BEGIN;

-- @B
-- expect-value: requested
SELECT status FROM trips WHERE code = 400106;

-- @A
-- expect-rows: 1
UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
                 accepted_at = now(), updated_at = now(), version = version + 1
WHERE code = 400106;

-- @A
INSERT INTO trip_claims (trip_id, driver_id, method)
SELECT id, 'd4000000-0000-4000-8000-000000009101', 'a_khong_khoa' FROM trips WHERE code = 400106;

-- @A
COMMIT;

-- @B
-- Vẫn ghi đè được trips (Read Committed, không điều kiện) …
-- expect-rows: 1
UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009102',
                 accepted_at = now(), updated_at = now(), version = version + 1
WHERE code = 400106;

-- @B
-- … nhưng không ghi được claim thứ hai.
-- expect-error: 23505 ux_trip_claims_trip
INSERT INTO trip_claims (trip_id, driver_id, method)
SELECT id, 'd4000000-0000-4000-8000-000000009102', 'a_khong_khoa' FROM trips WHERE code = 400106;

-- @B
-- Giao dịch hỏng ⇒ COMMIT trả thẻ ROLLBACK ⇒ cả UPDATE ghi đè của B cũng huỷ.
-- expect-match: ^ROLLBACK$
COMMIT;

-- @C
-- expect-line: 400106 | 1 | 9101 | 9101
SELECT t.code, count(c.id) AS so_claim, max(dc.code) AS duoc_bao_nhan, dt.code AS trips_driver
FROM trips AS t LEFT JOIN trip_claims AS c ON c.trip_id = t.id
LEFT JOIN drivers AS dc ON dc.id = c.driver_id JOIN drivers AS dt ON dt.id = t.driver_id
WHERE t.code = 400106 GROUP BY t.code, dt.code;
