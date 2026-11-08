SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';
BEGIN;
ALTER TABLE customers ADD COLUMN email text;
ALTER TABLE customers ADD CONSTRAINT uq_customers_email UNIQUE (email);
ALTER TABLE customers ADD CONSTRAINT ck_customers_email_lower CHECK (email = lower(email));
-- expect-rows: 2
UPDATE customers SET email = NULL WHERE phone IN ('0900000001', '0900000002');
-- expect-rows: 1
UPDATE customers SET email = 'an@toinoi.test' WHERE phone = '0900000001';
-- expect-error: 23514 ck_customers_email_lower
UPDATE customers SET email = 'Binh@toinoi.test' WHERE phone = '0900000002';
ROLLBACK;
-- Kiểm: cancelled + completed_at
-- expect-error: 23514 ck_trips_completed_at_by_status
UPDATE trips SET completed_at = '2026-11-02 18:05:00+07' WHERE code = 100006;
