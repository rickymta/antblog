-- PG-033 · Khoá dòng, khoá bảng và thời gian chờ — `postgresql-lock-row-table`
--
-- Bài toán Tới Nơi: lúc 01:00 tổng đài báo "màn sửa cước bị treo". MVCC (PG-032) cho người
-- ĐỌC không phải chờ, nhưng hai người cùng GHI một dòng thì người sau PHẢI chờ người trước
-- kết thúc giao dịch. Bài này tạo đúng tình huống đó rồi đọc ba nguồn để biết ai chặn ai:
-- pg_stat_activity (phiên đang làm gì, chờ gì), pg_locks (khoá giữ/chờ), pg_blocking_pids().
-- Hai hàm lab_cho() và lab_khoa() (p04/00) đọc các nguồn này và in NHÃN PHIÊN thay cho PID.
--
-- Ba phiên: [A], [B] ghi; [C] chỉ quan sát. "⏳ đang chờ khoá" là runner thấy backend của
-- bước đó có wait_event_type = 'Lock' (xem README gốc, mục «Nhiều phiên»).

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- ===========================================================================
-- §1. Khoá DÒNG do UPDATE: người ghi thứ hai chờ, người đọc không chờ.

-- @A
BEGIN;

-- @A
-- expect-value: 35000
UPDATE trips SET fare = fare + 1000, version = version + 1, updated_at = now()
WHERE code = 400104 RETURNING fare;

-- @B
-- Đọc: không chờ, thấy bản đã commit (34000).
-- expect-value: 34000
SELECT fare FROM trips WHERE code = 400104;

-- @B
-- Ghi cùng dòng: phải chờ A kết thúc giao dịch.
-- expect-blocked
UPDATE trips SET fare = fare + 500, version = version + 1, updated_at = now()
WHERE code = 400104 RETURNING fare;

-- @C
-- A "idle in transaction" (giữ khoá mà không làm gì — chính là "màn sửa cước" chưa bấm Lưu;
-- Client: ClientRead = backend đang chờ câu lệnh tiếp theo từ ứng dụng); B "active", chờ
-- Lock: transactionid — chờ GIAO DỊCH của A kết thúc, bị chặn bởi A. Cột câu lệnh là câu
-- CUỐI CÙNG mỗi phiên gửi (cắt 60 ký tự).
-- expect-line: A | idle in transaction | Client: ClientRead |  | UPDATE trips SET fare = fare + 1000, version = version + 1,
-- expect-line: B | active | Lock: transactionid | A | UPDATE trips SET fare = fare + 500, version = version + 1, u
SELECT * FROM lab_cho();

-- @C
-- Khoá DÒNG không nằm trong pg_locks: A giữ dòng bằng cách ghi xid của mình vào xmax của
-- dòng; pg_locks chỉ có khoá trên XID của A. Người chờ (B) xếp hàng bằng khoá 'tuple' rồi
-- chờ ShareLock trên xid của A (granted = f) — "chờ A commit hoặc rollback".
-- expect-line: A | relation | trips | RowExclusiveLock | t
-- expect-line: A | transactionid | xid của A | ExclusiveLock | t
-- expect-line: B | relation | trips | RowExclusiveLock | t
-- expect-line: B | transactionid | xid của A | ShareLock | f
-- expect-line: B | transactionid | xid của B | ExclusiveLock | t
-- expect-line: B | tuple | trips (0,34) | ExclusiveLock | t
SELECT * FROM lab_khoa();

-- @A
COMMIT;

-- @C
-- B chạy tiếp ngay khi A commit (dòng "✔ … đã chạy xong" ở trên) và áp +500 lên bản MỚI
-- (35000): ở Read Committed, người chờ ĐỌC LẠI dòng sau khi được khoá — không mất +1000 của A.
-- expect-line: 35500 | 3
SELECT fare, version FROM trips WHERE code = 400104;

-- ===========================================================================
-- §2. SELECT … FOR UPDATE: khoá dòng ĐỂ ĐỌC RỒI GHI. Bốn mức khoá dòng, mạnh dần:
-- FOR KEY SHARE (kiểm khoá ngoại dùng) · FOR SHARE · FOR NO KEY UPDATE (UPDATE không đổi
-- cột khoá dùng) · FOR UPDATE (DELETE, UPDATE đổi khoá). FOR UPDATE xung đột với CẢ FOR KEY
-- SHARE ⇒ chặn việc chèn dòng con tham chiếu tới dòng đang khoá.

-- @A
BEGIN;

-- @A
-- expect-line: 400105 | requested
SELECT code, status FROM trips WHERE code = 400105 FOR UPDATE;

-- @C
-- Dấu khoá dòng nằm NGAY TRÊN dòng: xmax = xid của A dù dòng không bị sửa.
-- expect-value: t
SELECT xmax::text::bigint = (SELECT backend_xid::text::bigint FROM pg_stat_activity WHERE application_name = 'lab-A')
FROM trips WHERE code = 400105;

-- @B
-- Thêm sự kiện cho chuyến đang bị FOR UPDATE: kiểm khoá ngoại cần FOR KEY SHARE trên dòng
-- trips ⇒ xung đột ⇒ CHỜ.
-- expect-blocked
-- expect-rows: 1
INSERT INTO trip_events (trip_id, status)
SELECT id, 'searching' FROM trips WHERE code = 400105;

-- @A
COMMIT;

-- @A
BEGIN;

-- @A
-- FOR NO KEY UPDATE: đủ để "đọc rồi sửa" một dòng mà không đổi khoá chính.
-- expect-line: 400105 | requested
SELECT code, status FROM trips WHERE code = 400105 FOR NO KEY UPDATE;

-- @B
-- Không xung đột với FOR KEY SHARE ⇒ chèn sự kiện KHÔNG phải chờ.
-- expect-rows: 1
INSERT INTO trip_events (trip_id, status)
SELECT id, 'searching' FROM trips WHERE code = 400105;

-- @B
-- Nhưng một phiên khác cũng muốn khoá để sửa ⇒ chờ.
-- expect-blocked
-- expect-line: 400105 | cancelled
SELECT code, status FROM trips WHERE code = 400105 FOR NO KEY UPDATE;

-- @A
-- expect-rows: 1
UPDATE trips SET status = 'cancelled', updated_at = now(), version = version + 1 WHERE code = 400105;

-- @A
COMMIT;

-- (B nhận dòng sau khi A commit và đọc lại bản mới: status = cancelled.)

-- ===========================================================================
-- §3. Khoá BẢNG: DDL lấy ACCESS EXCLUSIVE ⇒ chặn cả SELECT. lock_timeout ⇒ 55P03.

-- @A
BEGIN;

-- @A
ALTER TABLE trips ADD COLUMN ghi_chu text;

-- @B
-- expect-blocked
SELECT count(*) FROM trips;

-- @C
-- expect-line: A | relation | trips | AccessExclusiveLock | t
-- expect-line: A | transactionid | xid của A | ExclusiveLock | t
-- expect-line: B | relation | trips | AccessShareLock | f
SELECT * FROM lab_khoa();

-- @A
ROLLBACK;

-- (B đếm xong: 40 chuyến — cột ghi_chu không còn vì A đã ROLLBACK: DDL của PostgreSQL nằm
--  trong giao dịch như dữ liệu.)

-- Chiều ngược lại — DDL phải chờ người đọc, và trong lúc DDL chờ, MỌI người đến sau (kể cả
-- SELECT) xếp hàng sau nó: thí nghiệm p03/thu-nghiem/pg024-cluster-hang-doi-khoa.sql.
-- Cách an toàn khi đổi schema trên bảng đang chạy: đặt lock_timeout NGẮN cho câu DDL — chờ
-- không được thì bỏ cuộc sớm (rồi thử lại), thay vì đứng chặn cả hàng đợi.

-- @A
BEGIN;

-- @A
-- Một báo cáo dài đang đọc bảng (giữ AccessShareLock tới hết giao dịch).
-- expect-value: 40
SELECT count(*) FROM trips;

-- @B
SET lock_timeout = '300ms';

-- @B
-- lab: cho-xong
-- expect-blocked
-- expect-error: 55P03
ALTER TABLE trips ADD COLUMN ghi_chu text;

-- @C
-- B đã bỏ cuộc ⇒ không còn ai xếp hàng ⇒ C đọc bình thường dù A vẫn mở giao dịch.
-- expect-value: 40
SELECT count(*) FROM trips;

-- @A
COMMIT;

-- @B
RESET lock_timeout;

-- ===========================================================================
-- §4. statement_timeout ⇒ 57014: giới hạn TỔNG thời gian một câu (chờ khoá + chạy).
-- lock_timeout chỉ giới hạn MỖI lần chờ một khoá. Cả hai huỷ CÂU LỆNH (giao dịch hỏng —
-- phải ROLLBACK), không cắt kết nối.

-- @A
BEGIN;

-- @A
-- expect-rows: 1
UPDATE trips SET fare = fare + 1000 WHERE code = 400106;

-- @B
SET statement_timeout = '300ms';

-- @B
BEGIN;

-- @B
-- lab: cho-xong
-- expect-blocked
-- expect-error: 57014
UPDATE trips SET fare = fare + 500 WHERE code = 400106;

-- @B
-- expect-error: 25P02
SELECT 1;

-- @B
ROLLBACK;

-- @B
-- Không cần khoá vẫn bị huỷ nếu câu chạy quá lâu.
-- lab: cho-xong
-- expect-error: 57014
SELECT pg_sleep(1);

-- @B
RESET statement_timeout;

-- @A
COMMIT;

-- @C
-- Chỉ phần +1000 của A được ghi: 36000 + 1000.
-- expect-value: 37000
SELECT fare FROM trips WHERE code = 400106;

-- ===========================================================================
-- §5. Advisory lock: khoá theo một CON SỐ do ứng dụng tự đặt nghĩa (ở đây: mã chuyến).
-- pg_locks ghi locktype 'advisory', người chờ cũng là wait_event_type 'Lock'.

-- @A
-- Khoá mức PHIÊN: giữ qua COMMIT, tới khi unlock hoặc phiên đóng (dễ quên nhả!).
SELECT pg_advisory_lock(400107);

-- @B
-- expect-value: f
SELECT pg_try_advisory_lock(400107);

-- @B
-- expect-blocked
SELECT pg_advisory_lock(400107);

-- @C
-- expect-line: A | advisory | advisory 400107 | ExclusiveLock | t
-- expect-line: B | advisory | advisory 400107 | ExclusiveLock | f
SELECT * FROM lab_khoa();

-- @A
-- expect-value: t
SELECT pg_advisory_unlock(400107);

-- @B
-- expect-value: t
SELECT pg_advisory_unlock(400107);

-- @B
-- pg_advisory_xact_lock: khoá mức GIAO DỊCH — tự nhả khi COMMIT/ROLLBACK (nên dùng loại này).
BEGIN;

-- @B
SELECT pg_advisory_xact_lock(400107);

-- @B
COMMIT;

-- @C
-- expect-value: 0
SELECT count(*) FROM pg_locks WHERE locktype = 'advisory';
