-- PG-014 · thí nghiệm của bài: logic ba giá trị của IN/NOT IN, JOIN vs IN, và
-- câu kiểm kê bắt (hoặc không bắt) ba lỗi cố ý trong chuỗi CTE đối soát.
-- Nền: p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- 1. IN / NOT IN khi danh sách có NULL: kết quả là NULL (không biết), không phải false/true.
-- expect-line: t | NULL | f | NULL | t
SELECT 1 IN (1, NULL)      AS in_co_khop,
       1 IN (2, NULL)      AS in_khong_khop,
       1 NOT IN (1, NULL)  AS not_in_co_khop,
       1 NOT IN (2, NULL)  AS not_in_khong_khop,
       1 NOT IN (2, 3)     AS not_in_khong_null;

-- trips.driver_id có 297 NULL ⇒ mọi phép `NOT IN (SELECT driver_id FROM trips)` đều ra NULL hoặc false.
-- expect-value: 297
SELECT count(*) FROM trips WHERE driver_id IS NULL;

-- 2. JOIN đếm DÒNG chuyến huỷ; IN/EXISTS đếm KHÁCH.
-- expect-line: 461 | 266 | 266
SELECT (SELECT count(*) FROM customers AS c JOIN trips AS t ON t.customer_id = c.id
         WHERE t.status = 'cancelled')                                         AS join_dem_dong,
       (SELECT count(DISTINCT c.id) FROM customers AS c JOIN trips AS t ON t.customer_id = c.id
         WHERE t.status = 'cancelled')                                         AS join_distinct,
       (SELECT count(*) FROM customers AS c
         WHERE c.id IN (SELECT t.customer_id FROM trips AS t WHERE t.status = 'cancelled')) AS dung_in;

-- 3. Subquery tương quan viết lại bằng "gom trước rồi nối" (PG-013): cùng kết quả.
-- expect-line: DN |  389 | 190
-- expect-line: HN |  941 | 436
-- expect-line: SG | 1377 | 636
SELECT c.code AS tp, count(*) AS hoan_thanh,
       count(*) FILTER (WHERE t.fare > a.cuoc_tb) AS tren_tb_thanh_pho
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
JOIN (SELECT city_id, avg(fare) AS cuoc_tb FROM trips WHERE status = 'completed' GROUP BY city_id) AS a
  ON a.city_id = t.city_id
WHERE t.status = 'completed'
GROUP BY c.code
ORDER BY c.code COLLATE "C";

-- 4. Lỗi A: b2 quên GROUP BY (mỗi giao dịch một dòng) ⇒ b3 nở dòng.
--    Kiểm kê bắt được: b3 (3.209) ≠ b1 (3.168), dù số chuyến khác nhau vẫn 3.168.
-- expect-line: 3168 | 2729 | 3209 | 3168 | 86 | 46
WITH b1 AS (
  SELECT t.id, t.status, CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END AS phai_thu
  FROM trips AS t
  WHERE t.status IN ('completed', 'cancelled')
    AND t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
),
b2 AS (
  SELECT p.trip_id, CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END AS thuc_thu
  FROM payments AS p
  WHERE p.status = 'succeeded'
),
b3 AS (SELECT b1.*, coalesce(b2.thuc_thu, 0) AS thuc_thu FROM b1 LEFT JOIN b2 ON b2.trip_id = b1.id),
b4 AS (SELECT * FROM b3 WHERE thuc_thu <> phai_thu)
SELECT (SELECT count(*) FROM b1), (SELECT count(*) FROM b2), (SELECT count(*) FROM b3),
       (SELECT count(DISTINCT id) FROM b3), (SELECT count(*) FROM b4), (SELECT count(DISTINCT id) FROM b4);

-- 5. Lỗi B: b3 dùng JOIN thay LEFT JOIN ⇒ mất 480 chuyến không có giao dịch thành công,
--    kéo theo mất 100724 và 101084 khỏi danh sách lệch (còn 5 dòng, +442.000).
--    Kiểm kê bắt được: b3 (2.688) ≠ b1 (3.168).
-- expect-line: 3168 | 2688 | 2688 | 2688 | 5 | 442000
WITH b1 AS (
  SELECT t.id, t.status, CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END AS phai_thu
  FROM trips AS t
  WHERE t.status IN ('completed', 'cancelled')
    AND t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
),
b2 AS (
  SELECT p.trip_id, sum(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END) AS thuc_thu
  FROM payments AS p
  WHERE p.status = 'succeeded'
  GROUP BY p.trip_id
),
b3 AS (SELECT b1.*, b2.thuc_thu FROM b1 JOIN b2 ON b2.trip_id = b1.id),
b4 AS (SELECT * FROM b3 WHERE thuc_thu <> phai_thu)
SELECT (SELECT count(*) FROM b1), (SELECT count(*) FROM b2), (SELECT count(*) FROM b3),
       (SELECT count(DISTINCT id) FROM b3), (SELECT count(*) FROM b4), (SELECT sum(thuc_thu - phai_thu) FROM b4);

-- 6. Lỗi C: b2 quên lọc status = 'succeeded' (cộng cả giao dịch failed/pending).
--    Mọi số dòng vẫn khớp (b3 = b1 = 3.168, không trùng) nhưng b4 nhảy lên 102 chuyến,
--    +7.423.000 đ. Kiểm kê số dòng KHÔNG bắt được lỗi định nghĩa.
-- expect-line: 3168 | 2689 | 3168 | 3168 | 102 | 7423000
WITH b1 AS (
  SELECT t.id, t.status, CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END AS phai_thu
  FROM trips AS t
  WHERE t.status IN ('completed', 'cancelled')
    AND t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
),
b2 AS (
  SELECT p.trip_id, sum(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END) AS thuc_thu
  FROM payments AS p
  GROUP BY p.trip_id
),
b3 AS (SELECT b1.*, coalesce(b2.thuc_thu, 0) AS thuc_thu FROM b1 LEFT JOIN b2 ON b2.trip_id = b1.id),
b4 AS (SELECT * FROM b3 WHERE thuc_thu <> phai_thu)
SELECT (SELECT count(*) FROM b1), (SELECT count(*) FROM b2), (SELECT count(*) FROM b3),
       (SELECT count(DISTINCT id) FROM b3), (SELECT count(*) FROM b4), (SELECT sum(thuc_thu - phai_thu) FROM b4);

-- Cái bắt được lỗi C là đối chiếu với một nguồn độc lập: số giao dịch không thành công.
-- expect-line: failed  | 96 | 96
-- expect-line: pending |  1 |  1
SELECT status, count(*) AS so_gd, count(DISTINCT trip_id) AS so_chuyen
FROM payments
WHERE status <> 'succeeded'
GROUP BY status
ORDER BY status COLLATE "C";
