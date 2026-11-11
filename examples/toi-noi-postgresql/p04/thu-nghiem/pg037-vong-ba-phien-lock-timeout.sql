-- Thí nghiệm PG-037 (bốn phiên): hai điều bài nói tới nhưng tệp lab chính không chạy.
--   §1. Vòng chờ BA phiên (A → B → C → A) trên ba ví khách 1001, 1002, 1003: bộ dò vòng chờ
--       tìm cả vòng dài, không chỉ cặp hai phiên. C đặt deadlock_timeout 200 ms, A và B 10 s
--       ⇒ C là phiên chạy bộ dò và bị huỷ (lab ÉP, không phải quy tắc).
--   §2. Sau 55P03 (lock_timeout) giao dịch đã HỎNG: câu kế tiếp 25P02, COMMIT trả ROLLBACK,
--       cả phần đã ghi trước đó cũng mất — ứng dụng phải ROLLBACK rồi thử lại TOÀN BỘ.
--   §3. Giá trị mặc định và ai được đổi ba tham số dùng trong bài (pg_settings).
-- Bất biến: tổng ba ví 1001–1003 = 1.500.000 đ trước và sau.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- @D
SET search_path TO toinoi, public;

-- @A
SET deadlock_timeout = '10s';

-- @B
SET deadlock_timeout = '10s';

-- @C
SET deadlock_timeout = '200ms';

-- ===========================================================================
-- §1. Vòng chờ ba phiên. Mỗi phiên chuyển 1.000 đ từ "ví của mình" sang ví kế tiếp.

-- @A
BEGIN;

-- @A
-- expect-value: 499000
UPDATE wallets SET balance = balance - 1000 WHERE id = 1001 RETURNING balance;

-- @B
BEGIN;

-- @B
-- expect-value: 499000
UPDATE wallets SET balance = balance - 1000 WHERE id = 1002 RETURNING balance;

-- @C
BEGIN;

-- @C
-- expect-value: 499000
UPDATE wallets SET balance = balance - 1000 WHERE id = 1003 RETURNING balance;

-- @A
-- A cần 1002 (B giữ) ⇒ chờ B. Được chạy tiếp khi B COMMIT ⇒ thấy 499000 + 1000.
-- expect-blocked
-- expect-value: 500000
UPDATE wallets SET balance = balance + 1000 WHERE id = 1002 RETURNING balance;

-- @B
-- B cần 1003 (C giữ) ⇒ chờ C. Chưa có vòng: A → B → C, C vẫn rảnh.
-- (Được chạy tiếp khi C bị huỷ ⇒ khoản trừ của C mất ⇒ 500000 + 1000.)
-- expect-blocked
-- expect-value: 501000
UPDATE wallets SET balance = balance + 1000 WHERE id = 1003 RETURNING balance;

-- @D
-- expect-line: A | active | Lock: transactionid | B | UPDATE wallets SET balance = balance + 1000 WHERE id = 1002
-- expect-line: B | active | Lock: transactionid | C | UPDATE wallets SET balance = balance + 1000 WHERE id = 1003
-- expect-line: C | idle in transaction | Client: ClientRead |  | UPDATE wallets SET balance = balance - 1000 WHERE id = 1003
SELECT * FROM lab_cho();

-- @C
-- C cần 1001 (A giữ) ⇒ khép vòng A → B → C → A. Sau 200 ms C chạy bộ dò ⇒ 40P01.
-- DETAIL liệt kê đủ BA tiến trình trong vòng.
-- lab: cho-xong
-- lab: nhat-ky
-- expect-blocked
-- expect-error: 40P01
UPDATE wallets SET balance = balance + 1000 WHERE id = 1001 RETURNING balance;

-- (C bị huỷ ⇒ nhả 1003 ⇒ B chạy tiếp — dòng "✔" của B ở trên. A vẫn chờ B.)

-- @C
-- expect-match: ^ROLLBACK$
COMMIT;

-- @B
COMMIT;

-- (B commit ⇒ nhả 1002 ⇒ A chạy tiếp — dòng "✔" của A ở trên.)

-- @A
COMMIT;

-- @D
-- A: 1001 −1000, 1002 +1000; B: 1002 −1000, 1003 +1000; C bị huỷ toàn bộ.
-- expect-line: 1001 | 499000
-- expect-line: 1002 | 500000
-- expect-line: 1003 | 501000
SELECT id, balance FROM wallets WHERE id IN (1001, 1002, 1003) ORDER BY id;

-- @D
-- expect-value: 1500000
SELECT sum(balance) FROM wallets WHERE id IN (1001, 1002, 1003);

-- ===========================================================================
-- §2. 55P03 làm hỏng cả giao dịch, kể cả phần đã ghi trước câu bị huỷ.

-- @A
BEGIN;

-- @A
UPDATE wallets SET balance = balance - 5000 WHERE id = 1002;

-- @B
BEGIN;

-- @B
SET LOCAL lock_timeout = '300ms';

-- @B
-- B ghi ví 1004 trước (không ai giữ) — phần này sẽ mất theo.
-- expect-value: 495000
UPDATE wallets SET balance = balance - 5000 WHERE id = 1004 RETURNING balance;

-- @B
-- lab: cho-xong
-- expect-blocked
-- expect-error: 55P03
UPDATE wallets SET balance = balance + 5000 WHERE id = 1002;

-- @B
-- Giao dịch đã hỏng: mọi câu bị từ chối tới khi kết thúc.
-- expect-error: 25P02
SELECT balance FROM wallets WHERE id = 1004;

-- @B
-- expect-match: ^ROLLBACK$
COMMIT;

-- @A
ROLLBACK;

-- @D
-- Khoản trừ ở 1004 không còn.
-- expect-line: 1002 | 500000
-- expect-line: 1004 | 500000
SELECT id, balance FROM wallets WHERE id IN (1002, 1004) ORDER BY id;

-- ===========================================================================
-- §3. Mặc định và quyền đổi (context = superuser ⇒ chỉ superuser hoặc người được GRANT SET).

-- @D
-- expect-line: deadlock_timeout | 1000 | ms | superuser
-- expect-line: lock_timeout | 0 | ms | user
-- expect-line: log_lock_waits | off | NULL | superuser
SELECT name, setting, unit, context FROM pg_settings
WHERE name IN ('deadlock_timeout', 'lock_timeout', 'log_lock_waits') ORDER BY name;

-- ===========================================================================
-- §4. Vì sao "SELECT … ORDER BY id FOR NO KEY UPDATE" khoá đúng thứ tự id: nút LockRows nằm
-- TRÊN nút Sort ⇒ dòng được khoá lần lượt theo thứ tự đã sắp.

-- @D
-- expect-match: ^ LockRows$
-- expect-match: ^\s+->  Sort$
-- expect-match: Sort Key: id
EXPLAIN (COSTS OFF)
SELECT id FROM wallets WHERE id IN (1001, 2001) ORDER BY id FOR NO KEY UPDATE;
