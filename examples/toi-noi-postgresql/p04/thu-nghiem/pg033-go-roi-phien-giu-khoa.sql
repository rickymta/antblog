-- Thí nghiệm PG-033 (bốn phiên): CHẨN ĐOÁN và GỠ một hàng chờ khoá, không dùng hàm lab.
--
-- Tình huống "màn sửa cước bị treo": [A] mở màn sửa cước chuyến 400108 (UPDATE trong giao
-- dịch rồi ngồi yên — idle in transaction); [B] và [C] lần lượt bấm sửa cùng chuyến ⇒ xếp
-- hàng. [D] là người trực: đọc pg_stat_activity + pg_blocking_pids() bằng câu SQL THƯỜNG
-- (không qua lab_cho()), rồi gỡ theo hai mức:
--   pg_cancel_backend(pid)    — huỷ CÂU LỆNH đang chạy của một phiên (phiên còn sống, 57014)
--   pg_terminate_backend(pid) — cắt KẾT NỐI (giao dịch dở bị ROLLBACK, nhả mọi khoá)
-- Huỷ người CHỜ (B) không giải quyết gì: người GIỮ khoá (A) mới là gốc.
-- Phần cuối: advisory lock mức PHIÊN không theo giao dịch — ROLLBACK không nhả nó.
-- Không có trong lab chính (p04/33-…sql §1 dùng lab_cho()/lab_khoa()).

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- @D
SET search_path TO toinoi, public;

-- ===========================================================================
-- §1. Tạo hàng chờ: A giữ, B chờ A, C chờ sau B.

-- @A
BEGIN;

-- @A
-- expect-value: 39000
UPDATE trips SET fare = fare + 1000 WHERE code = 400108 RETURNING fare;

-- @B
-- B chạy ở chế độ tự commit (không BEGIN). Bị D huỷ ở §2 ⇒ 57014.
-- expect-blocked
-- expect-error: 57014
UPDATE trips SET fare = fare + 200 WHERE code = 400108;

-- @C
-- expect-blocked
-- expect-rows: 1
UPDATE trips SET fare = fare + 500 WHERE code = 400108 RETURNING fare;

-- @D
-- Câu chẩn đoán dùng được trên mọi server (cột pid thật thay bằng application_name cho
-- đầu ra tất định). "chặn bởi" = application_name của các pid trong pg_blocking_pids().
-- A: không chờ gì, đang ngồi yên trong giao dịch ⇒ GỐC của hàng chờ.
-- B: chờ GIAO DỊCH của A (transactionid). C: chờ khoá 'tuple' mà B đang giữ ⇒ chặn bởi B.
-- expect-line: lab-A | idle in transaction | Client | ClientRead |
-- expect-line: lab-B | active | Lock | transactionid | lab-A
-- expect-line: lab-C | active | Lock | tuple | lab-B
SELECT a.application_name, a.state, a.wait_event_type, a.wait_event,
       coalesce((SELECT string_agg(b.application_name, ',' ORDER BY b.application_name)
                 FROM pg_stat_activity AS b
                 WHERE b.pid = ANY (pg_blocking_pids(a.pid))), '') AS chan_boi
FROM pg_stat_activity AS a
WHERE a.application_name IN ('lab-A', 'lab-B', 'lab-C')
ORDER BY a.application_name;

-- @D
-- Tìm gốc: phiên ĐANG CHẶN người khác mà bản thân KHÔNG bị ai chặn.
-- expect-line: lab-A | idle in transaction
SELECT a.application_name, a.state
FROM pg_stat_activity AS a
WHERE cardinality(pg_blocking_pids(a.pid)) = 0
  AND EXISTS (SELECT 1 FROM pg_stat_activity AS w WHERE a.pid = ANY (pg_blocking_pids(w.pid)));

-- ===========================================================================
-- §2. Gỡ SAI chỗ: huỷ câu lệnh của người chờ B.

-- @D
-- expect-value: t
SELECT pg_cancel_backend(pid) FROM pg_stat_activity WHERE application_name = 'lab-B';

-- @D
-- B đã rời hàng (lỗi 57014 in ở dòng ✔ của B phía trên), C tiến lên một bậc: giờ C chờ thẳng
-- giao dịch của A. Hàng chờ vẫn còn — gốc A chưa được xử lý.
-- expect-line: lab-B | idle | Client | ClientRead |
-- expect-line: lab-C | active | Lock | transactionid | lab-A
SELECT a.application_name, a.state, coalesce(a.wait_event_type, '') AS loai_cho,
       coalesce(a.wait_event, '') AS cho,
       coalesce((SELECT string_agg(b.application_name, ',')
                 FROM pg_stat_activity AS b
                 WHERE b.pid = ANY (pg_blocking_pids(a.pid))), '') AS chan_boi
FROM pg_stat_activity AS a
WHERE a.application_name IN ('lab-B', 'lab-C')
ORDER BY a.application_name;

-- ===========================================================================
-- §3. Gỡ ĐÚNG chỗ: cắt kết nối của phiên giữ khoá A (phần sửa của A bị huỷ).

-- @D
-- expect-value: t
SELECT pg_terminate_backend(pid, 5000) FROM pg_stat_activity WHERE application_name = 'lab-A';

-- @A
-- A quay lại bấm Lưu: kết nối đã mất.
-- expect-mat-ket-noi
COMMIT;

-- @D
-- +1000 của A không còn (A bị ROLLBACK), +200 của B bị huỷ, chỉ có +500 của C: 38000 + 500.
-- expect-value: 38500
SELECT fare FROM trips WHERE code = 400108;

-- ===========================================================================
-- §4. Advisory lock mức PHIÊN không theo giao dịch: ROLLBACK KHÔNG nhả khoá.

-- @A
SET search_path TO toinoi, public;

-- @A
BEGIN;

-- @A
SELECT pg_advisory_lock(400109);

-- @A
ROLLBACK;

-- @B
-- Giao dịch của A đã kết thúc nhưng khoá mức phiên vẫn còn.
-- expect-value: f
SELECT pg_try_advisory_lock(400109);

-- @D
-- expect-line: lab-A | ExclusiveLock | t
SELECT a.application_name, l.mode, l.granted
FROM pg_locks AS l JOIN pg_stat_activity AS a ON a.pid = l.pid
WHERE l.locktype = 'advisory' AND l.objid = 400109;

-- @A
-- expect-value: t
SELECT pg_advisory_unlock(400109);

-- @B
-- expect-value: t
SELECT pg_try_advisory_lock(400109);

-- @B
-- expect-value: t
SELECT pg_advisory_unlock(400109);
