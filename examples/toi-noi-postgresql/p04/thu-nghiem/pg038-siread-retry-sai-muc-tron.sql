-- Thí nghiệm PG-038 (ba phiên): bổ sung cho 38-serializable-retry.sql.
--   §1. SIReadLock đọc THẲNG từ pg_locks (lab_khoa() lọc bỏ page index): khi ép đi index,
--       giao dịch giữ khoá vị từ trên page của ix_driver_duty_area + từng dòng đã đọc; sau khi
--       chính nó UPDATE một dòng đã đọc, khoá đọc trên dòng đó không còn hiện (thay vào đó có
--       page của pk_driver_duty mà câu UPDATE đã dò).
--   §2. 40001 KHÔNG chỉ rơi ở COMMIT: hai giao dịch SERIALIZABLE sửa CÙNG một dòng ⇒ bên sau
--       chờ, rồi lỗi ngay ở câu UPDATE ("concurrent update") — giống Repeatable Read.
--   §3. Trộn mức cô lập: A SERIALIZABLE, B READ COMMITTED (như một job/script quên đặt mức)
--       ⇒ write skew LỌT QUA, không lỗi nào. SSI chỉ bảo vệ giữa các giao dịch SERIALIZABLE.
--   §4. Retry SAI (lời giải bài tập): B nhận 40001 ở COMMIT rồi chỉ gửi lại câu UPDATE, không
--       đọc lại ⇒ khu 111 không còn ai trực.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- ###########################################################################
-- §1. SIReadLock thật trong pg_locks (ép index).
-- ###########################################################################

-- @A
SET enable_seqscan = off;

-- @A
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @A
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 112 AND on_duty;

-- @C
-- Đọc qua index ⇒ khoá vị từ trên page index + hai dòng của khu 112 (9103 ở (0,3), 9104 ở (0,4)).
-- expect-line: A | page | ix_driver_duty_area | 1 | NULL
-- expect-line: A | tuple | driver_duty | 0 | 3
-- expect-line: A | tuple | driver_duty | 0 | 4
SELECT lab_nhan(l.pid) AS phien, l.locktype, c.relname, l.page, l.tuple
FROM pg_locks AS l JOIN pg_class AS c ON c.oid = l.relation
WHERE l.mode = 'SIReadLock' AND lab_nhan(l.pid) IN ('A', 'B')
ORDER BY 1, 2, 3, 4, 5;

-- @A
UPDATE driver_duty SET on_duty = false WHERE driver_id = 'd4000000-0000-4000-8000-000000009103';

-- @C
-- Sau khi A tự sửa dòng 9103: khoá đọc trên (0,3) không còn; thêm page của pk_driver_duty
-- (câu UPDATE tìm dòng qua khoá chính); còn page ix_driver_duty_area và dòng 9104.
-- expect-line: A | page | ix_driver_duty_area | 1 | NULL
-- expect-line: A | page | pk_driver_duty | 1 | NULL
-- expect-line: A | tuple | driver_duty | 0 | 4
SELECT lab_nhan(l.pid) AS phien, l.locktype, c.relname, l.page, l.tuple
FROM pg_locks AS l JOIN pg_class AS c ON c.oid = l.relation
WHERE l.mode = 'SIReadLock' AND lab_nhan(l.pid) IN ('A', 'B')
ORDER BY 1, 2, 3, 4, 5;

-- @A
ROLLBACK;

-- @A
RESET enable_seqscan;

-- ###########################################################################
-- §2. Cùng sửa một dòng ⇒ 40001 ở câu UPDATE (sau khi chờ), không đợi tới COMMIT.
-- ###########################################################################

-- @A
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @A
-- expect-rows: 1
UPDATE driver_duty SET changed_at = now() WHERE driver_id = 'd4000000-0000-4000-8000-000000009101';

-- @B
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @B
-- Câu đầu của giao dịch chụp snapshot.
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @B
-- expect-blocked
-- expect-error: 40001
UPDATE driver_duty SET on_duty = false WHERE driver_id = 'd4000000-0000-4000-8000-000000009101';

-- @A
COMMIT;

-- @B
ROLLBACK;

-- ###########################################################################
-- §3. Trộn mức cô lập: SSI không thấy giao dịch READ COMMITTED.
-- ###########################################################################

-- @A
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @A
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @B
BEGIN ISOLATION LEVEL READ COMMITTED;

-- @B
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @B
-- expect-rows: 1
UPDATE driver_duty SET on_duty = false WHERE driver_id = 'd4000000-0000-4000-8000-000000009102';

-- @B
-- expect-match: ^COMMIT$
COMMIT;

-- @A
-- expect-rows: 1
UPDATE driver_duty SET on_duty = false WHERE driver_id = 'd4000000-0000-4000-8000-000000009101';

-- @A
-- expect-match: ^COMMIT$
COMMIT;

-- @C
-- Khu 111 không còn ai trực, không lỗi nào báo.
-- expect-value: 0
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @C
UPDATE driver_duty SET on_duty = true WHERE area_id = 111;

-- ###########################################################################
-- §4. Retry SAI: chỉ gửi lại câu UPDATE sau 40001.
-- ###########################################################################

-- @A
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @A
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @B
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @B
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @A
UPDATE driver_duty SET on_duty = false WHERE driver_id = 'd4000000-0000-4000-8000-000000009101';

-- @B
UPDATE driver_duty SET on_duty = false WHERE driver_id = 'd4000000-0000-4000-8000-000000009102';

-- @A
-- expect-match: ^COMMIT$
COMMIT;

-- @B
-- expect-error: 40001
COMMIT;

-- @B
-- "Thử lại" sai: mở giao dịch mới nhưng chỉ phát lại câu ghi đã có, KHÔNG đếm lại.
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @B
-- expect-rows: 1
UPDATE driver_duty SET on_duty = false WHERE driver_id = 'd4000000-0000-4000-8000-000000009102';

-- @B
-- Không ai chạy song song nên COMMIT thành công — và bất biến bị phá.
-- expect-match: ^COMMIT$
COMMIT;

-- @C
-- expect-value: 0
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @C
UPDATE driver_duty SET on_duty = true WHERE area_id = 111;
