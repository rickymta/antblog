-- Thí nghiệm PG-033 (hai phiên): bốn loại "hết giờ" khác nhau ở CÁI BỊ HUỶ.
--   lock_timeout                         — huỷ CÂU LỆNH đang chờ một khoá quá lâu (55P03)
--   statement_timeout                    — huỷ CÂU LỆNH chạy quá lâu (57014)
--   idle_in_transaction_session_timeout  — cắt KẾT NỐI ngồi yên trong giao dịch quá lâu (25P03, FATAL)
--   transaction_timeout (PG 17+)         — cắt KẾT NỐI có giao dịch kéo dài quá lâu, kể cả khi
--                                          nó vẫn đang làm việc (25P04, FATAL)
-- Hai loại đầu: phiên còn sống, giao dịch hỏng (phải ROLLBACK). Hai loại sau: mất kết nối.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @A
SET transaction_timeout = '300ms';

-- @A
BEGIN;

-- @A
-- expect-rows: 1
UPDATE trips SET fare = fare + 1000 WHERE code = 400101;

-- @A
-- Không ngồi yên (đang chạy câu lệnh) nhưng cả giao dịch đã quá 300 ms ⇒ server cắt kết nối.
-- lab: cho-xong
-- lab: nhat-ky
-- expect-mat-ket-noi
SELECT pg_sleep(1);

-- @B
-- Phần sửa của A bị huỷ.
-- expect-value: 31000
SELECT fare FROM trips WHERE code = 400101;
