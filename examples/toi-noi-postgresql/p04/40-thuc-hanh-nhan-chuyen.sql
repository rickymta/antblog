-- PG-040 · Thực hành: hai tài xế cùng nhận một chuyến — `postgresql-thuc-hanh-tranh-chap-nhan-chuyen`
--
-- Đề: chuyến mới được gửi cho vài tài xế gần nhất CÙNG LÚC; ai bấm "Nhận" trước thì được.
-- BẤT BIẾN cần bàn giao: mỗi chuyến có ĐÚNG MỘT người thắng — đúng một tài xế được ứng dụng
-- báo "bạn đã nhận chuyến" (một dòng trip_claims), và đó chính là trips.driver_id.
-- Bốn cách làm, mỗi cách trên một chuyến riêng, cùng các tài xế 9101 [A], 9102 [B], 9103 [C]:
--   (a) 400101 — đọc trạng thái rồi ghi, KHÔNG khoá, không điều kiện       ⇒ HAI người thắng
--   (b) 400102 — UPDATE có điều kiện WHERE status = 'requested'           ⇒ một người thắng
--   (c) 400103 — SELECT … FOR UPDATE rồi kiểm rồi ghi                     ⇒ một người thắng
--   (d) 400104 — ràng buộc duy nhất: trip_assignments PRIMARY KEY (trip_id) ⇒ một người thắng
-- Lịch chạy tất định ở đây chỉ ra CƠ CHẾ. Bằng chứng ở quy mô (1.000 chuyến × 8 phiên đồng
-- thời, lặp nhiều lần, đếm số người thắng mỗi chuyến): p04/dong-thoi.mjs, kịch bản K040-* —
-- `npm run p04:measure` ghi p04/DO-DAC.md; `npm test` chạy bản nhỏ và kiểm bất biến.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- ===========================================================================
-- (a) Đọc rồi ghi, không khoá. Lịch: A đọc · B đọc · A ghi + commit · B ghi + commit.

-- @A
BEGIN;

-- @A
-- expect-value: requested
SELECT status FROM trips WHERE code = 400101;

-- @B
BEGIN;

-- @B
-- expect-value: requested
SELECT status FROM trips WHERE code = 400101;

-- @A
-- "Còn trống" ⇒ A ghi mình là tài xế và báo A đã nhận.
-- expect-rows: 1
UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
                 accepted_at = now(), updated_at = now(), version = version + 1
WHERE code = 400101;

-- @A
INSERT INTO trip_claims (trip_id, driver_id, method)
SELECT id, 'd4000000-0000-4000-8000-000000009101', 'a_khong_khoa' FROM trips WHERE code = 400101;

-- @A
COMMIT;

-- @B
-- B cũng đã thấy "còn trống" ⇒ cũng ghi (không điều kiện) — ghi đè A, không lỗi, không chờ.
-- expect-rows: 1
UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009102',
                 accepted_at = now(), updated_at = now(), version = version + 1
WHERE code = 400101;

-- @B
INSERT INTO trip_claims (trip_id, driver_id, method)
SELECT id, 'd4000000-0000-4000-8000-000000009102', 'a_khong_khoa' FROM trips WHERE code = 400101;

-- @B
COMMIT;

-- @C
-- Hai tài xế cùng được báo "đã nhận"; trong DB chỉ còn B. A chạy tới điểm đón vô ích.
-- expect-line: 9101 | 9102
-- expect-line: 9102 | 9102
SELECT d.code AS duoc_bao_nhan, (SELECT dt.code FROM drivers AS dt WHERE dt.id = t.driver_id) AS driver_trong_db
FROM trip_claims AS c JOIN drivers AS d ON d.id = c.driver_id JOIN trips AS t ON t.id = c.trip_id
WHERE t.code = 400101 ORDER BY c.id;

-- ===========================================================================
-- (b) Một câu UPDATE có điều kiện + ghi claim trong CÙNG câu (CTE). Ba tài xế cùng lúc:
-- A ghi trước (chưa commit) ⇒ B, C chờ khoá dòng ⇒ A commit ⇒ B, C đọc lại dòng: status
-- không còn 'requested' ⇒ 0 dòng ⇒ không claim.

-- @A
BEGIN;

-- @A
-- expect-value: 9101
WITH nhan AS (
  UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
                   accepted_at = now(), updated_at = now(), version = version + 1
  WHERE code = 400102 AND status = 'requested'
  RETURNING id, driver_id)
INSERT INTO trip_claims (trip_id, driver_id, method)
SELECT id, driver_id, 'b_co_dieu_kien' FROM nhan
RETURNING (SELECT code FROM drivers WHERE id = driver_id);

-- @B
-- expect-blocked
-- expect-rows: 0
WITH nhan AS (
  UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009102',
                   accepted_at = now(), updated_at = now(), version = version + 1
  WHERE code = 400102 AND status = 'requested'
  RETURNING id, driver_id)
INSERT INTO trip_claims (trip_id, driver_id, method)
SELECT id, driver_id, 'b_co_dieu_kien' FROM nhan
RETURNING (SELECT code FROM drivers WHERE id = driver_id);

-- @C
-- C xếp hàng SAU B: B đang giữ khoá 'tuple' (chỗ đầu hàng chờ dòng này) ⇒ C chờ B, không chờ A.
-- expect-blocked
-- expect-rows: 0
WITH nhan AS (
  UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009103',
                   accepted_at = now(), updated_at = now(), version = version + 1
  WHERE code = 400102 AND status = 'requested'
  RETURNING id, driver_id)
INSERT INTO trip_claims (trip_id, driver_id, method)
SELECT id, driver_id, 'b_co_dieu_kien' FROM nhan
RETURNING (SELECT code FROM drivers WHERE id = driver_id);

-- @A
COMMIT;

-- (B và C chạy tiếp lần lượt — dòng "✔" ở trên — mỗi người 0 dòng ⇒ app báo "đã có người nhận".)

-- ===========================================================================
-- (c) Khoá dòng trước (FOR UPDATE), kiểm trạng thái trong ứng dụng, rồi ghi.

-- @A
BEGIN;

-- @A
-- expect-value: requested
SELECT status FROM trips WHERE code = 400103 FOR UPDATE;

-- @B
BEGIN;

-- @B
-- expect-blocked
-- expect-value: accepted
SELECT status FROM trips WHERE code = 400103 FOR UPDATE;

-- @A
UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
                 accepted_at = now(), updated_at = now(), version = version + 1
WHERE code = 400103;

-- @A
INSERT INTO trip_claims (trip_id, driver_id, method)
SELECT id, 'd4000000-0000-4000-8000-000000009101', 'c_for_update' FROM trips WHERE code = 400103;

-- @A
COMMIT;

-- (B nhận khoá và đọc ĐÚNG bản mới — 'accepted' — dòng "✔" ở trên ⇒ app không ghi.)

-- @B
ROLLBACK;

-- ===========================================================================
-- (d) Ràng buộc duy nhất làm trọng tài: mỗi người chèn một dòng phân công; khoá chính
-- trip_id chỉ cho MỘT dòng tồn tại. Đúng kể cả khi logic ứng dụng có lỗi (lưới an toàn cuối).

-- @A
BEGIN;

-- @A
-- expect-rows: 1
INSERT INTO trip_assignments (trip_id, driver_id)
SELECT id, 'd4000000-0000-4000-8000-000000009101' FROM trips WHERE code = 400104;

-- @B
BEGIN;

-- @B
-- Cùng giá trị khoá đang được A chèn (chưa commit) ⇒ chờ xem A commit hay huỷ.
-- expect-blocked
-- expect-error: 23505 pk_trip_assignments
INSERT INTO trip_assignments (trip_id, driver_id)
SELECT id, 'd4000000-0000-4000-8000-000000009102' FROM trips WHERE code = 400104;

-- @C
-- Biến thể không ném lỗi: ON CONFLICT DO NOTHING ⇒ 0 dòng.
-- expect-blocked
-- expect-rows: 0
INSERT INTO trip_assignments (trip_id, driver_id)
SELECT id, 'd4000000-0000-4000-8000-000000009103' FROM trips WHERE code = 400104
ON CONFLICT (trip_id) DO NOTHING;

-- @A
UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
                 accepted_at = now(), updated_at = now(), version = version + 1
WHERE code = 400104;

-- @A
INSERT INTO trip_claims (trip_id, driver_id, method)
SELECT id, 'd4000000-0000-4000-8000-000000009101', 'd_unique' FROM trips WHERE code = 400104;

-- @A
COMMIT;

-- @B
-- (B nhận 23505 — dòng "✔" ở trên; giao dịch hỏng ⇒ ROLLBACK.)
ROLLBACK;

-- ===========================================================================
-- Bàn giao: kiểm bất biến trên cả bốn chuyến.

-- @C
-- Số người được báo "đã nhận" mỗi chuyến, và claim có khớp trips.driver_id không.
-- expect-line: 400101 | a_khong_khoa | 2 | 1 | VI PHẠM
-- expect-line: 400102 | b_co_dieu_kien | 1 | 1 | đạt
-- expect-line: 400103 | c_for_update | 1 | 1 | đạt
-- expect-line: 400104 | d_unique | 1 | 1 | đạt
SELECT t.code, c.method, count(*) AS so_nguoi_thang,
       count(*) FILTER (WHERE c.driver_id = t.driver_id) AS khop_driver_id,
       CASE WHEN count(*) = 1 AND count(*) FILTER (WHERE c.driver_id = t.driver_id) = 1
            THEN 'đạt' ELSE 'VI PHẠM' END AS bat_bien
FROM trip_claims AS c JOIN trips AS t ON t.id = c.trip_id
GROUP BY t.code, c.method ORDER BY t.code;

-- @C
-- Phân công khớp trips cho cách (d).
-- expect-line: 400104 | 9101 | 9101
SELECT t.code, da.code AS phan_cong, dt.code AS trips_driver
FROM trip_assignments AS a JOIN trips AS t ON t.id = a.trip_id
JOIN drivers AS da ON da.id = a.driver_id JOIN drivers AS dt ON dt.id = t.driver_id;
