-- PG-014 · lời giải bài tập.
-- Nền: p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Bài 1. Khách chưa từng đặt chuyến nào. customer_id là NOT NULL nên NOT IN
-- cũng ra đúng — nhưng chỉ vì ràng buộc đó; NOT EXISTS đúng mà không cần nó.
-- expect-line: 25 | 25
SELECT (SELECT count(*) FROM customers AS c
         WHERE NOT EXISTS (SELECT 1 FROM trips AS t WHERE t.customer_id = c.id)) AS not_exists,
       (SELECT count(*) FROM customers AS c
         WHERE c.id NOT IN (SELECT t.customer_id FROM trips AS t))              AS not_in;

-- Bài 2. Chuyến hoàn thành chưa có đánh giá: NOT EXISTS và LEFT JOIN … IS NULL.
-- expect-line: 816 | 816
SELECT (SELECT count(*) FROM trips AS t
         WHERE t.status = 'completed'
           AND NOT EXISTS (SELECT 1 FROM trip_ratings AS r WHERE r.trip_id = t.id)) AS not_exists,
       (SELECT count(*) FROM trips AS t
         LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
         WHERE t.status = 'completed' AND r.trip_id IS NULL)                       AS left_join_is_null;

-- Bài 3. Thêm kiểm cho bước b2 của chuỗi đối soát:
--   k1  trip_id trong b2 duy nhất                       (kỳ vọng 0 nhóm trùng)
--   k2  mọi chuyến trong b2 thuộc phạm vi b1            (kỳ vọng 0)
--   k3  chuyến của b1 không có giao dịch thành công     (3.168 − 2.688 = 480)
-- expect-line: 0 | 0 | 480 | 447 | 33
WITH b1 AS (
  SELECT t.id, t.status
  FROM trips AS t
  WHERE t.status IN ('completed', 'cancelled')
    AND t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
),
b2 AS (
  SELECT p.trip_id,
         sum(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END) AS thuc_thu
  FROM payments AS p
  WHERE p.status = 'succeeded'
  GROUP BY p.trip_id
)
SELECT (SELECT count(*) FROM (SELECT trip_id FROM b2 GROUP BY trip_id HAVING count(*) > 1) AS x) AS k1_trung,
       (SELECT count(*) FROM b2 WHERE NOT EXISTS (SELECT 1 FROM b1 WHERE b1.id = b2.trip_id))     AS k2_ngoai_pham_vi,
       (SELECT count(*) FROM b1 WHERE NOT EXISTS (SELECT 1 FROM b2 WHERE b2.trip_id = b1.id))     AS k3_khong_gd,
       (SELECT count(*) FROM b1 WHERE b1.status = 'cancelled'
          AND NOT EXISTS (SELECT 1 FROM b2 WHERE b2.trip_id = b1.id))                             AS k3_huy,
       (SELECT count(*) FROM b1 WHERE b1.status = 'completed'
          AND NOT EXISTS (SELECT 1 FROM b2 WHERE b2.trip_id = b1.id))                             AS k3_hoan_thanh;
