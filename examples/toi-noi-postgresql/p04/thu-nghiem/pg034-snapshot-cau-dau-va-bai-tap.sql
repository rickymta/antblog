-- Thí nghiệm PG-034 (ba phiên): bốn chi tiết của mức cô lập mà p04/34-…sql chưa chạy, và lời
-- giải bài tập của bài.
--   §1 READ UNCOMMITTED không đọc bẩn: PostgreSQL chạy nó như READ COMMITTED.
--   §2 Snapshot của REPEATABLE READ chụp ở CÂU ĐẦU TIÊN, không phải lúc BEGIN.
--   §3 REPEATABLE READ: người ghi trước ROLLBACK ⇒ người chờ ghi tiếp, KHÔNG 40001.
--   §4 (bài tập 2) Sửa lost update S3 ở READ COMMITTED bằng khoá dòng khi đọc.
--   §5 (bài tập 3) S4 dưới REPEATABLE READ: thử lại toàn bộ giao dịch ⇒ 570000.
--   §6 SET TRANSACTION ISOLATION LEVEL chỉ hợp lệ trước câu truy vấn đầu tiên (25001).
-- Ví 1002, 1003, 1004 lúc đầu đều 500000; chuyến 400103 cước 33000.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- ===========================================================================
-- §1. READ UNCOMMITTED: cú pháp được nhận, hành vi là READ COMMITTED.

-- @A
BEGIN;

-- @A
-- expect-rows: 1
UPDATE trips SET fare = 99000 WHERE code = 400103;

-- @B
BEGIN ISOLATION LEVEL READ UNCOMMITTED;

-- @B
-- A chưa commit 99000 ⇒ B vẫn chỉ thấy bản đã commit.
-- expect-value: 33000
SELECT fare FROM trips WHERE code = 400103;

-- @A
ROLLBACK;

-- @B
COMMIT;

-- ===========================================================================
-- §2. REPEATABLE READ chụp snapshot ở câu đầu tiên (không phải câu BEGIN).

-- @B
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @A
-- A sửa và tự commit SAU khi B đã BEGIN, nhưng TRƯỚC câu đầu tiên của B.
-- expect-rows: 1
UPDATE trips SET fare = 34000 WHERE code = 400103;

-- @B
-- Câu đầu tiên của B ⇒ snapshot chụp lúc này ⇒ thấy 34000.
-- expect-value: 34000
SELECT fare FROM trips WHERE code = 400103;

-- @A
-- expect-rows: 1
UPDATE trips SET fare = 35000 WHERE code = 400103;

-- @B
-- Từ đây B đứng yên ở snapshot đã chụp.
-- expect-value: 34000
SELECT fare FROM trips WHERE code = 400103;

-- @B
COMMIT;

-- ===========================================================================
-- §3. REPEATABLE READ: 40001 chỉ xảy ra khi người ghi trước COMMIT.

-- @A
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @A
-- expect-rows: 1
UPDATE wallets SET balance = balance + 20000 WHERE id = 1002;

-- @B
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @B
-- expect-blocked
-- expect-value: 550000
UPDATE wallets SET balance = balance + 50000 WHERE id = 1002 RETURNING balance;

-- @A
ROLLBACK;

-- @B
COMMIT;

-- @C
-- expect-value: 550000
SELECT balance FROM wallets WHERE id = 1002;

-- ===========================================================================
-- §4. Bài tập 2 — S3 (đọc – tính trong app – ghi) ở READ COMMITTED, đọc kèm khoá dòng.

-- @A
BEGIN;

-- @A
SELECT balance AS so_du FROM wallets WHERE id = 1003 FOR NO KEY UPDATE \gset

-- @A
-- expect-value: 500000
SELECT :so_du AS so_du_da_doc;

-- @B
BEGIN;

-- @B
-- B muốn đọc kèm khoá ⇒ phải chờ A; câu đọc này chưa trả gì cho tới khi A kết thúc.
-- expect-blocked
SELECT balance AS so_du FROM wallets WHERE id = 1003 FOR NO KEY UPDATE \gset

-- @A
-- expect-rows: 1
UPDATE wallets SET balance = :so_du + 20000 WHERE id = 1003;

-- @A
COMMIT;

-- @B
-- B nhận khoá rồi đọc bản MỚI: 520000.
-- expect-value: 520000
SELECT :so_du AS so_du_da_doc;

-- @B
-- expect-rows: 1
UPDATE wallets SET balance = :so_du + 50000 WHERE id = 1003;

-- @B
COMMIT;

-- @C
-- expect-value: 570000
SELECT balance FROM wallets WHERE id = 1003;

-- ===========================================================================
-- §5. Bài tập 3 — S4 dưới REPEATABLE READ rồi thử lại.

-- @A
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @A
-- expect-rows: 1
UPDATE wallets SET balance = balance + 20000 WHERE id = 1004;

-- @B
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @B
-- expect-blocked
-- expect-error: 40001
UPDATE wallets SET balance = balance + 50000 WHERE id = 1004;

-- @A
COMMIT;

-- @B
-- expect-match: ^ROLLBACK$
COMMIT;

-- @B
-- Lần thử 2: giao dịch mới, snapshot mới (đã có +20000 của A).
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @B
-- expect-value: 570000
UPDATE wallets SET balance = balance + 50000 WHERE id = 1004 RETURNING balance;

-- @B
COMMIT;

-- @C
-- expect-value: 570000
SELECT balance FROM wallets WHERE id = 1004;

-- ===========================================================================
-- §6. SET TRANSACTION phải đứng TRƯỚC câu truy vấn đầu tiên của giao dịch.

-- @A
BEGIN;

-- @A
SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;

-- @A
-- expect-value: repeatable read
SHOW transaction_isolation;

-- @A
COMMIT;

-- @A
BEGIN;

-- @A
-- expect-value: 1
SELECT 1;

-- @A
-- Đã có câu truy vấn ⇒ quá muộn để đổi mức cô lập.
-- expect-error: 25001
SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;

-- @A
ROLLBACK;
