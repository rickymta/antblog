-- P02 · 14-subquery-exists-cte.sql — PG-014 `postgresql-subquery-exists-cte`
-- Subquery, EXISTS và CTE: tách truy vấn đối soát thành các BƯỚC KIỂM CHỨNG
-- ĐƯỢC.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- 1. Subquery vô hướng (trả đúng 1 giá trị) dùng như một hằng số.
--    Subquery TƯƠNG QUAN (correlated) tham chiếu dòng bên ngoài: "chuyến hoàn
--    thành có cước cao hơn trung bình CỦA THÀNH PHỐ mình".
-- expect-line: 67339
SELECT round(avg(fare)) AS cuoc_tb_toan_he_thong FROM trips WHERE status = 'completed';

-- expect-line: DN |  389 | 190
-- expect-line: HN |  941 | 436
-- expect-line: SG | 1377 | 636
SELECT c.code AS tp,
       count(*) AS hoan_thanh,
       count(*) FILTER (WHERE t.fare > (SELECT avg(t2.fare) FROM trips AS t2
                                        WHERE t2.city_id = t.city_id AND t2.status = 'completed')) AS tren_tb_thanh_pho
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
WHERE t.status = 'completed'
GROUP BY c.code
ORDER BY c.code COLLATE "C";

-- Subquery vô hướng trả HƠN 1 dòng là lỗi lúc chạy, không phải lúc viết.
-- expect-error: 21000
SELECT code, (SELECT p.amount FROM payments AS p WHERE p.trip_id = t.id) AS so_tien
FROM trips AS t
WHERE t.code = 100042;

-- ---------------------------------------------------------------------------
-- 2. IN và EXISTS cho câu hỏi "có hay không": khách từng huỷ ít nhất 1 chuyến.
--    Hai cách cho cùng kết quả; EXISTS dừng ở dòng khớp đầu tiên và không bao
--    giờ nhân bản dòng bên ngoài.
-- expect-line: 266 | 266
SELECT (SELECT count(*) FROM customers AS c
         WHERE c.id IN (SELECT t.customer_id FROM trips AS t WHERE t.status = 'cancelled'))   AS dung_in,
       (SELECT count(*) FROM customers AS c
         WHERE EXISTS (SELECT 1 FROM trips AS t WHERE t.customer_id = c.id AND t.status = 'cancelled')) AS dung_exists;

-- ---------------------------------------------------------------------------
-- 3. BẪY NOT IN + NULL: "tài xế chưa chạy chuyến nào".
--    trips.driver_id có 297 giá trị NULL. `x NOT IN (…, NULL)` không bao giờ
--    TRUE (x <> NULL ra NULL) ⇒ truy vấn trả 0 dòng, KHÔNG báo lỗi.
-- expect-rows: 0
SELECT d.code FROM drivers AS d
WHERE d.id NOT IN (SELECT t.driver_id FROM trips AS t)
ORDER BY d.code;

-- NOT EXISTS đúng với mọi trường hợp: 8219, 8220 vừa đăng ký 29/10.
-- expect-line: 8219 | 2026-10-29 09:00:00+07
-- expect-line: 8220 | 2026-10-29 09:00:00+07
SELECT d.code, d.created_at FROM drivers AS d
WHERE NOT EXISTS (SELECT 1 FROM trips AS t WHERE t.driver_id = d.id)
ORDER BY d.code;

-- NOT IN chỉ an toàn khi subquery chắc chắn không có NULL.
-- expect-rows: 2
SELECT d.code FROM drivers AS d
WHERE d.id NOT IN (SELECT t.driver_id FROM trips AS t WHERE t.driver_id IS NOT NULL)
ORDER BY d.code;

-- ---------------------------------------------------------------------------
-- 4. LATERAL: subquery trong FROM được nhìn thấy dòng đứng trước nó — tiện cho
--    "N dòng đầu của MỖI nhóm": 2 chuyến cước cao nhất mỗi thành phố
--    (hoà cước thì mã nhỏ hơn đứng trước — thứ tự xác định).
-- expect-line: SG | 101601 | food | 150000
-- expect-line: SG | 101725 | food | 150000
-- expect-line: HN | 101699 | food | 149000
-- expect-line: HN | 101913 | food | 149000
-- expect-line: DN | 101355 | food | 150000
-- expect-line: DN | 101431 | food | 150000
SELECT c.code AS tp, top.code, top.kind, top.fare
FROM cities AS c
CROSS JOIN LATERAL (
  SELECT t.code, t.kind, t.fare
  FROM trips AS t
  WHERE t.city_id = c.id AND t.status = 'completed'
  ORDER BY t.fare DESC, t.code
  LIMIT 2
) AS top
ORDER BY c.id, top.fare DESC, top.code;

-- ---------------------------------------------------------------------------
-- 5. CTE (WITH): đặt tên cho từng bước để ĐỌC và KIỂM được. Bài toán: tháng
--    10, chuyến nào có tiền thực thu khác số phải thu?
--      b1  phạm vi      : chuyến đặt trong tháng 10 đã kết thúc
--      b2  giao dịch    : thực thu mỗi chuyến = Σ charge − Σ refund + Σ adjustment (succeeded)
--      b3  ghép         : b1 LEFT JOIN b2 — phải giữ ĐÚNG số dòng của b1
--      b4  lệch         : thực thu ≠ phải thu (completed: cước; cancelled: 0)
--    Câu kiểm kê in số dòng của TỪNG bước — mỗi số đối chiếu được với một truy
--    vấn độc lập (PG-012: 3.168 chuyến đã kết thúc trong kỳ).
-- expect-line: 3168 | 2688 | 3168 | 3168 | 7
WITH b1 AS (
  SELECT t.id, t.code, t.status, t.fare,
         CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END AS phai_thu
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
),
b3 AS (
  SELECT b1.*, coalesce(b2.thuc_thu, 0) AS thuc_thu
  FROM b1 LEFT JOIN b2 ON b2.trip_id = b1.id
),
b4 AS (
  SELECT * FROM b3 WHERE thuc_thu <> phai_thu
)
SELECT (SELECT count(*) FROM b1)                 AS b1_pham_vi,
       (SELECT count(*) FROM b2)                 AS b2_chuyen_co_gd,
       (SELECT count(*) FROM b3)                 AS b3_sau_ghep,
       (SELECT count(DISTINCT id) FROM b3)       AS b3_chuyen_khac_nhau,
       (SELECT count(*) FROM b4)                 AS b4_lech;

-- Cùng chuỗi CTE, bước cuối in danh sách lệch và tổng chênh. Tổng chênh của
-- b4 PHẢI bằng Σ thực thu − Σ phải thu của cả b3 (chênh chỉ đến từ dòng lệch).
-- expect-line: 100724 | completed |  76000 |      0 |  -76000
-- expect-line: 101084 | completed |  98000 |      0 |  -98000
-- expect-line: 101310 | completed |  85000 | 170000 |   85000
-- expect-line: 101987 | cancelled |      0 | -72000 |  -72000
-- expect-line: 102443 | completed |  54000 |  45000 |   -9000
-- expect-line: 102642 | completed |  38000 | 380000 |  342000
-- expect-line: 102739 | cancelled |      0 |  96000 |   96000
-- expect-line: TỔNG b4 | NULL      |    351000 |    619000 |  268000
-- expect-line: TỔNG b3 | NULL      | 182287000 | 182555000 |  268000
WITH b1 AS (
  SELECT t.id, t.code, t.status, t.fare,
         CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END AS phai_thu
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
),
b3 AS (
  SELECT b1.*, coalesce(b2.thuc_thu, 0) AS thuc_thu
  FROM b1 LEFT JOIN b2 ON b2.trip_id = b1.id
),
b4 AS (
  SELECT * FROM b3 WHERE thuc_thu <> phai_thu
)
SELECT ma, status, phai_thu, thuc_thu, chenh
FROM (SELECT 1 AS nhom, code::text AS ma, status, phai_thu, thuc_thu, thuc_thu - phai_thu AS chenh FROM b4
      UNION ALL
      SELECT 2, 'TỔNG b4', NULL, sum(phai_thu), sum(thuc_thu), sum(thuc_thu - phai_thu) FROM b4
      UNION ALL
      SELECT 3, 'TỔNG b3', NULL, sum(phai_thu), sum(thuc_thu), sum(thuc_thu - phai_thu) FROM b3) AS u
ORDER BY nhom, ma;

-- ---------------------------------------------------------------------------
-- 6. Từ PostgreSQL 12, CTE chỉ được tham chiếu một lần (và không có tác dụng
--    phụ) thường được "gộp" vào truy vấn chính như subquery; MATERIALIZED buộc
--    tính riêng một lần, NOT MATERIALIZED buộc gộp. Kết quả GIỐNG NHAU — khác
--    nhau ở kế hoạch thực thi (P03 đo bằng EXPLAIN).
-- expect-line: 7 | 7
SELECT
  (WITH b2 AS MATERIALIZED (
     SELECT trip_id, sum(CASE kind WHEN 'refund' THEN -amount ELSE amount END) AS thuc_thu
     FROM payments WHERE status = 'succeeded' GROUP BY trip_id)
   SELECT count(*) FROM trips AS t LEFT JOIN b2 ON b2.trip_id = t.id
   WHERE t.status IN ('completed', 'cancelled')
     AND coalesce(b2.thuc_thu, 0) <> CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END) AS materialized,
  (WITH b2 AS NOT MATERIALIZED (
     SELECT trip_id, sum(CASE kind WHEN 'refund' THEN -amount ELSE amount END) AS thuc_thu
     FROM payments WHERE status = 'succeeded' GROUP BY trip_id)
   SELECT count(*) FROM trips AS t LEFT JOIN b2 ON b2.trip_id = t.id
   WHERE t.status IN ('completed', 'cancelled')
     AND coalesce(b2.thuc_thu, 0) <> CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END) AS not_materialized;
