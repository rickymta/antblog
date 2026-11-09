-- PG-020 · cầu nối từ tổng charge thành công tới thực thu (không có trong lab).
-- Mọi giao dịch đều thuộc chuyến trong phạm vi (lab mục 1), nên cộng thẳng trên payments.
-- Nền: p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql
SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- expect-line: charge     | 2714 | 183640000
-- expect-line: refund     |   13 |  -1080000
-- expect-line: adjustment |    2 |     -5000
-- expect-line: THỰC THU   | 2729 | 182555000
SELECT coalesce(kind, 'THỰC THU') AS khoan, count(*) AS so_gd,
       sum(CASE kind WHEN 'refund' THEN -amount ELSE amount END) AS tien
FROM payments
WHERE status = 'succeeded'
GROUP BY ROLLUP (kind)
ORDER BY grouping(kind), CASE kind WHEN 'charge' THEN 1 WHEN 'refund' THEN 2 ELSE 3 END;

-- Giao dịch không thành công: không đổi tiền nhưng phải đếm được.
-- expect-line: failed  | 96
-- expect-line: pending |  1
SELECT status, count(*) FROM payments WHERE status <> 'succeeded' GROUP BY status ORDER BY status COLLATE "C";
