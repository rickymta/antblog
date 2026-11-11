-- PG-037 · Deadlock, thứ tự lấy khoá và timeout — `postgresql-deadlock-lock-order-timeout`
--
-- Bài toán Tới Nơi: hai nghiệp vụ chuyển tiền giữa ví VíXanh chạy cùng lúc trên CÙNG một cặp
-- ví nhưng theo HAI CHIỀU:
--   [A] thanh toán chuyến : trừ ví khách 1001 → cộng ví tài xế 2001 (30.000 đ)
--   [B] hoàn tiền         : trừ ví tài xế 2001 → cộng ví khách 1001 (10.000 đ)
-- Mỗi bên khoá ví thứ nhất rồi chờ ví thứ hai đang bị bên kia khoá ⇒ VÒNG CHỜ (deadlock):
-- không bên nào tự thoát được. PostgreSQL phát hiện vòng chờ và huỷ MỘT giao dịch (40P01).
--
-- Ai bị huỷ? Phiên nào chờ đủ deadlock_timeout (mặc định 1 s) sẽ chạy bộ dò vòng chờ; thấy
-- vòng thì nó TỰ huỷ chính mình. Để lịch chạy tất định, bài đặt deadlock_timeout của A = 10 s,
-- của B = 200 ms ⇒ B luôn là bên dò ra và bị huỷ (đặt được vì lab chạy bằng superuser).
-- Bất biến: tổng tiền trong mọi ví không đổi (p04/02: 14.800.000 đ).

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- @A
SET deadlock_timeout = '10s';

-- @B
SET deadlock_timeout = '200ms';

-- @B
-- log_lock_waits: ghi nhật ký server khi một lần chờ khoá kéo dài quá deadlock_timeout.
SET log_lock_waits = on;

-- ===========================================================================
-- §1. Tái hiện vòng chờ.

-- @A
BEGIN;

-- @A
-- expect-value: 470000
UPDATE wallets SET balance = balance - 30000, updated_at = now() WHERE id = 1001 RETURNING balance;

-- @B
BEGIN;

-- @B
-- expect-value: 190000
UPDATE wallets SET balance = balance - 10000, updated_at = now() WHERE id = 2001 RETURNING balance;

-- @A
-- A cần ví 2001 — B đang giữ ⇒ A chờ B. (Khi được chạy tiếp, A thấy 230000: khoản trừ của B đã bị huỷ.)
-- expect-blocked
-- expect-value: 230000
UPDATE wallets SET balance = balance + 30000, updated_at = now() WHERE id = 2001 RETURNING balance;

-- @C
-- expect-line: A | active | Lock: transactionid | B | UPDATE wallets SET balance = balance + 30000, updated_at = n
-- expect-line: B | idle in transaction | Client: ClientRead |  | UPDATE wallets SET balance = balance - 10000, updated_at = n
SELECT * FROM lab_cho();

-- @B
-- B cần ví 1001 — A đang giữ ⇒ B chờ A ⇒ vòng A → B → A. Sau 200 ms B chạy bộ dò, thấy vòng
-- và bị huỷ với 40P01. Thông báo lỗi chỉ nói "process … waits for … blocked by process …";
-- câu lệnh của hai phiên chỉ có trong nhật ký server.
-- lab: cho-xong
-- lab: nhat-ky
-- expect-blocked
-- expect-error: 40P01
UPDATE wallets SET balance = balance + 10000, updated_at = now() WHERE id = 1001 RETURNING balance;

-- (B bị huỷ ⇒ khoá của B nhả ngay ⇒ A — đang chờ ví 2001 — chạy tiếp, xem dòng "✔" ở trên.)

-- @A
COMMIT;

-- @B
-- Giao dịch của B đã hỏng: COMMIT trả ROLLBACK, khoản trừ 10.000 ở ví 2001 cũng bị huỷ.
-- expect-match: ^ROLLBACK$
COMMIT;

-- @C
-- Chỉ thanh toán của A có hiệu lực; tổng tiền không đổi.
-- expect-line: 1001 | 470000
-- expect-line: 2001 | 230000
SELECT id, balance FROM wallets WHERE id IN (1001, 2001) ORDER BY id;

-- @C
-- expect-value: 14800000
SELECT sum(balance) FROM wallets;

-- Ứng dụng nhận 40P01 ⇒ THỬ LẠI cả giao dịch hoàn tiền (giống 40001 — PG-038).
-- @B
BEGIN;

-- @B
UPDATE wallets SET balance = balance - 10000, updated_at = now() WHERE id = 2001;

-- @B
UPDATE wallets SET balance = balance + 10000, updated_at = now() WHERE id = 1001;

-- @B
COMMIT;

-- @C
-- expect-line: 1001 | 480000
-- expect-line: 2001 | 220000
SELECT id, balance FROM wallets WHERE id IN (1001, 2001) ORDER BY id;

-- ===========================================================================
-- §2. Sửa tận gốc: MỌI giao dịch lấy khoá theo CÙNG một thứ tự (ở đây: id ví tăng dần),
-- bằng một câu khoá trước tất cả ví sẽ đụng tới. Bên đến sau chờ ngay ở ví ĐẦU TIÊN, chưa
-- giữ gì ⇒ không thể thành vòng. (Một câu UPDATE … WHERE id IN (…) KHÔNG đủ: nó khoá dòng
-- theo thứ tự quét — không bảo đảm.) FOR NO KEY UPDATE: đủ để sửa số dư, không chặn khoá ngoại.

-- @A
BEGIN;

-- @A
-- Thanh toán: khoá 1001 rồi 2001 (thứ tự id), rồi mới trừ/cộng.
-- expect-line: 1001
-- expect-line: 2001
SELECT id FROM wallets WHERE id IN (1001, 2001) ORDER BY id FOR NO KEY UPDATE;

-- @B
BEGIN;

-- @B
-- Hoàn tiền: CŨNG khoá 1001 trước (dù nghiệp vụ trừ 2001 trước) ⇒ chờ A ở 1001, chưa giữ 2001.
-- expect-blocked
-- expect-line: 1001
-- expect-line: 2001
SELECT id FROM wallets WHERE id IN (1001, 2001) ORDER BY id FOR NO KEY UPDATE;

-- @A
UPDATE wallets SET balance = balance - 30000, updated_at = now() WHERE id = 1001;

-- @A
-- A đi tiếp được vì đã giữ sẵn 2001.
UPDATE wallets SET balance = balance + 30000, updated_at = now() WHERE id = 2001;

-- @A
COMMIT;

-- (B nhận khoá cả hai ví — dòng "✔" ở trên.)

-- @B
UPDATE wallets SET balance = balance - 10000, updated_at = now() WHERE id = 2001;

-- @B
UPDATE wallets SET balance = balance + 10000, updated_at = now() WHERE id = 1001;

-- @B
COMMIT;

-- @C
-- expect-line: 1001 | 460000
-- expect-line: 2001 | 240000
SELECT id, balance FROM wallets WHERE id IN (1001, 2001) ORDER BY id;

-- ===========================================================================
-- §3. lock_timeout làm lưới an toàn: thay vì đứng chờ lâu (hoặc chờ deadlock_timeout mới bị
-- huỷ), câu lệnh bỏ cuộc sau một khoảng ⇒ 55P03 ⇒ ứng dụng rollback và thử lại / báo lỗi.
-- Nó KHÔNG thay cho thứ tự khoá: deadlock chỉ biến thành lỗi timeout sớm hơn.

-- @A
BEGIN;

-- @A
UPDATE wallets SET balance = balance - 5000, updated_at = now() WHERE id = 1002;

-- @B
BEGIN;

-- @B
SET LOCAL lock_timeout = '300ms';

-- @B
-- lab: cho-xong
-- expect-blocked
-- expect-error: 55P03
UPDATE wallets SET balance = balance + 5000, updated_at = now() WHERE id = 1002;

-- @B
ROLLBACK;

-- @A
ROLLBACK;

-- @C
-- Bất biến cuối bài: tổng tiền không đổi qua mọi lịch.
-- expect-value: 14800000
SELECT sum(balance) FROM wallets;
