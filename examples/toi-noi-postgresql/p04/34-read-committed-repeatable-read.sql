-- PG-034 · Read Committed và Repeatable Read — `postgresql-isolation-read-committed-repeatable-read`
--
-- Bài toán Tới Nơi: báo cáo đọc cước một chuyến hai lần trong cùng giao dịch ra hai số khác
-- nhau; hai nhân viên cùng nạp tiền vào một ví thì mất một khoản. Mức cô lập (isolation
-- level) quyết định giao dịch thấy thay đổi của giao dịch khác ra sao:
--   * READ COMMITTED (mặc định): MỖI CÂU chụp snapshot mới — thấy mọi thứ đã commit trước câu.
--   * REPEATABLE READ: chụp snapshot MỘT lần ở câu đầu tiên của giao dịch, giữ tới hết.
-- Bài chạy CÙNG bốn lịch S1–S4 dưới hai mức, cùng dữ liệu (phiên [C] đặt lại trước mỗi lịch),
-- rồi so kết quả. [A], [B] là hai giao dịch tranh chấp; câu lẻ của [B] (không BEGIN) tự commit.
--
-- Bảng so sánh (đúng như đầu ra bên dưới):
--   lịch                               READ COMMITTED            REPEATABLE READ
--   S1 đọc lại một dòng                 37000 → 40000 (đổi)       37000 → 37000
--   S2 đếm lại theo điều kiện           3 → 4 (phantom)           3 → 3 (PostgreSQL: không phantom)
--   S3 đọc–tính–ghi (app cộng tiền)     550000: MẤT +20000 của A  B lỗi 40001 ⇒ thử lại ⇒ 570000
--   S4 ghi tương đối balance + x        570000 đúng (B chờ rồi    B lỗi 40001 ⇒ phải thử lại
--                                       đọc lại bản mới)

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- ###########################################################################
-- PHẦN 1 · READ COMMITTED
-- ###########################################################################

-- S1 · Đọc lại một dòng (non-repeatable read) ------------------------------
-- @C
UPDATE trips SET fare = 37000 WHERE code = 400107;

-- @A
BEGIN ISOLATION LEVEL READ COMMITTED;

-- @A
-- expect-value: 37000
SELECT fare FROM trips WHERE code = 400107;

-- @B
UPDATE trips SET fare = 40000, version = version + 1 WHERE code = 400107;

-- @A
-- Câu mới, snapshot mới ⇒ thấy 40000: cùng giao dịch, cùng câu, khác kết quả.
-- expect-value: 40000
SELECT fare FROM trips WHERE code = 400107;

-- @A
COMMIT;

-- S2 · Đếm lại (phantom) ----------------------------------------------------
-- Khu 113 (SG-TT-03) có 3 chuyến đang chờ: 400102, 400105, 400108.
-- @A
BEGIN ISOLATION LEVEL READ COMMITTED;

-- @A
-- expect-value: 3
SELECT count(*) FROM trips WHERE pickup_area_id = 113 AND status = 'requested';

-- @B
INSERT INTO trips (id, kind, city_id, customer_id, fare, pickup_area_id)
VALUES ('a4000000-0000-4000-8000-000000400211', 'ride', 1, 'c4000000-0000-4000-8000-000000000011', 44000, 113);

-- @A
-- Dòng "ma" (phantom) xuất hiện: cùng điều kiện, thêm một dòng.
-- expect-value: 4
SELECT count(*) FROM trips WHERE pickup_area_id = 113 AND status = 'requested';

-- @A
COMMIT;

-- @C
DELETE FROM trips WHERE id = 'a4000000-0000-4000-8000-000000400211';

-- S3 · Đọc – tính trong ứng dụng – ghi (lost update) ---------------------------
-- Hai nhân viên cùng nạp tiền vào ví 1001: A +20.000, B +50.000. Ứng dụng đọc số dư, cộng
-- trong code, rồi GHI ĐÈ con số đã tính (\gset lưu kết quả vào biến psql :so_du của phiên).
-- @C
UPDATE wallets SET balance = 500000 WHERE id = 1001;

-- @A
BEGIN ISOLATION LEVEL READ COMMITTED;

-- @A
-- Đọc số dư vào biến psql :so_du của phiên (\gset không in kết quả).
SELECT balance AS so_du FROM wallets WHERE id = 1001 \gset

-- @A
-- expect-value: 500000
SELECT :so_du AS so_du_da_doc;

-- @B
BEGIN ISOLATION LEVEL READ COMMITTED;

-- @B
-- Đọc số dư vào biến psql :so_du của phiên (\gset không in kết quả).
SELECT balance AS so_du FROM wallets WHERE id = 1001 \gset

-- @B
-- expect-value: 500000
SELECT :so_du AS so_du_da_doc;

-- @A
-- expect-rows: 1
UPDATE wallets SET balance = :so_du + 20000 WHERE id = 1001;

-- @B
-- Ghi cùng dòng ⇒ chờ A.
-- expect-blocked
-- expect-rows: 1
UPDATE wallets SET balance = :so_du + 50000 WHERE id = 1001;

-- @A
COMMIT;

-- (B được chạy tiếp và GHI ĐÈ 550000 — con số tính từ 500000 nó đã đọc trước khi A commit.)

-- @B
COMMIT;

-- @C
-- Đúng ra phải là 570000: khoản +20000 của A đã MẤT (lost update) — không lỗi nào báo.
-- expect-value: 550000
SELECT balance FROM wallets WHERE id = 1001;

-- S4 · Ghi tương đối (balance = balance + x) --------------------------------
-- @C
UPDATE wallets SET balance = 500000 WHERE id = 1001;

-- @A
BEGIN ISOLATION LEVEL READ COMMITTED;

-- @A
UPDATE wallets SET balance = balance + 20000 WHERE id = 1001;

-- @B
BEGIN ISOLATION LEVEL READ COMMITTED;

-- @B
-- expect-blocked
-- expect-rows: 1
UPDATE wallets SET balance = balance + 50000 WHERE id = 1001 RETURNING balance;

-- @A
COMMIT;

-- (B được khoá dòng, ĐỌC LẠI bản mới 520000 rồi mới cộng — "EvalPlanQual" của Read Committed.)

-- @B
COMMIT;

-- @C
-- expect-value: 570000
SELECT balance FROM wallets WHERE id = 1001;

-- ###########################################################################
-- PHẦN 2 · REPEATABLE READ — y nguyên bốn lịch, chỉ đổi mức cô lập
-- ###########################################################################

-- S1 ---------------------------------------------------------------------------
-- @C
UPDATE trips SET fare = 37000 WHERE code = 400107;

-- @A
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @A
-- expect-value: 37000
SELECT fare FROM trips WHERE code = 400107;

-- @B
UPDATE trips SET fare = 40000, version = version + 1 WHERE code = 400107;

-- @A
-- Snapshot của giao dịch chụp ở câu đầu ⇒ vẫn 37000.
-- expect-value: 37000
SELECT fare FROM trips WHERE code = 400107;

-- @A
COMMIT;

-- @A
-- Giao dịch mới ⇒ thấy 40000.
-- expect-value: 40000
SELECT fare FROM trips WHERE code = 400107;

-- S2 ---------------------------------------------------------------------------
-- @A
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @A
-- expect-value: 3
SELECT count(*) FROM trips WHERE pickup_area_id = 113 AND status = 'requested';

-- @B
INSERT INTO trips (id, kind, city_id, customer_id, fare, pickup_area_id)
VALUES ('a4000000-0000-4000-8000-000000400212', 'ride', 1, 'c4000000-0000-4000-8000-000000000012', 44000, 113);

-- @A
-- Chuẩn SQL CHO PHÉP phantom ở Repeatable Read; PostgreSQL thì không (snapshot cả giao dịch).
-- expect-value: 3
SELECT count(*) FROM trips WHERE pickup_area_id = 113 AND status = 'requested';

-- @A
COMMIT;

-- @C
DELETE FROM trips WHERE id = 'a4000000-0000-4000-8000-000000400212';

-- S3 ---------------------------------------------------------------------------
-- @C
UPDATE wallets SET balance = 500000 WHERE id = 1001;

-- @A
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @A
-- Đọc số dư vào biến psql :so_du của phiên (\gset không in kết quả).
SELECT balance AS so_du FROM wallets WHERE id = 1001 \gset

-- @A
-- expect-value: 500000
SELECT :so_du AS so_du_da_doc;

-- @B
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @B
-- Đọc số dư vào biến psql :so_du của phiên (\gset không in kết quả).
SELECT balance AS so_du FROM wallets WHERE id = 1001 \gset

-- @B
-- expect-value: 500000
SELECT :so_du AS so_du_da_doc;

-- @A
-- expect-rows: 1
UPDATE wallets SET balance = :so_du + 20000 WHERE id = 1001;

-- @B
-- expect-blocked
-- expect-error: 40001
UPDATE wallets SET balance = :so_du + 50000 WHERE id = 1001;

-- @A
COMMIT;

-- (Khi A commit, B nhận lỗi 40001: dòng B định sửa đã bị một giao dịch commit SAU snapshot
--  của B sửa trước — B không được ghi đè lên thứ nó chưa từng thấy.)

-- @B
-- Giao dịch đã hỏng: COMMIT trả ROLLBACK.
-- expect-match: ^ROLLBACK$
COMMIT;

-- @C
-- expect-value: 520000
SELECT balance FROM wallets WHERE id = 1001;

-- Ứng dụng THỬ LẠI TOÀN BỘ giao dịch (đọc lại từ đầu) — đúng cách xử lý 40001 (PG-038).
-- @B
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @B
-- Đọc số dư vào biến psql :so_du của phiên (\gset không in kết quả).
SELECT balance AS so_du FROM wallets WHERE id = 1001 \gset

-- @B
-- expect-value: 520000
SELECT :so_du AS so_du_da_doc;

-- @B
-- expect-rows: 1
UPDATE wallets SET balance = :so_du + 50000 WHERE id = 1001;

-- @B
COMMIT;

-- @C
-- expect-value: 570000
SELECT balance FROM wallets WHERE id = 1001;

-- S4 ---------------------------------------------------------------------------
-- @C
UPDATE wallets SET balance = 500000 WHERE id = 1001;

-- @A
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @A
UPDATE wallets SET balance = balance + 20000 WHERE id = 1001;

-- @B
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @B
-- Kể cả ghi tương đối, Repeatable Read cũng không "đọc lại bản mới" như Read Committed: lỗi.
-- expect-blocked
-- expect-error: 40001
UPDATE wallets SET balance = balance + 50000 WHERE id = 1001;

-- @A
COMMIT;

-- @B
-- expect-match: ^ROLLBACK$
COMMIT;

-- @C
-- Chỉ có +20000 của A; B phải thử lại (chưa thử ở đây).
-- expect-value: 520000
SELECT balance FROM wallets WHERE id = 1001;

-- ###########################################################################
-- Đặt mức cô lập: BEGIN ISOLATION LEVEL …, SET TRANSACTION ISOLATION LEVEL … (câu đầu
-- giao dịch), hoặc mặc định của phiên/database: default_transaction_isolation.
-- ###########################################################################

-- @A
SET default_transaction_isolation = 'repeatable read';

-- @A
BEGIN;

-- @A
-- expect-value: repeatable read
SHOW transaction_isolation;

-- @A
COMMIT;

-- @A
-- READ UNCOMMITTED được chấp nhận về cú pháp nhưng PostgreSQL chạy như READ COMMITTED:
-- không bao giờ đọc được dữ liệu chưa commit (không có "dirty read").
BEGIN ISOLATION LEVEL READ UNCOMMITTED;

-- @A
-- expect-value: read uncommitted
SHOW transaction_isolation;

-- @A
COMMIT;
