-- P02 · 13-join-cardinality.sql — PG-013 `postgresql-join-quan-he-cardinality`
-- JOIN và số dòng sau khi nối bảng: kiểm quan hệ một–nhiều, KHÔNG nhân đôi
-- tổng tiền.
--
-- Quan hệ trong dữ liệu P02 (đếm ở 02-kiem-tra-fixture.sql, H9):
--   trips 1 — 0..n payments      (485 chuyến 0 dòng, 2.552 chuyến 1 dòng, 137 chuyến 2 dòng)
--   trips 1 — 1..8 trip_events   (mỗi chuyến 3–7 dòng lịch sử)
--   trips 1 — 0..1 trip_ratings  (khoá chính trip_id ⇒ không bao giờ quá 1)
--   trips n — 1 cities / drivers / merchants  (FK trỏ tới khoá chính)
-- Phạm vi: 2.707 chuyến hoàn thành; doanh thu đúng = 182.287.000 đ (PG-012).

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- 1. Đếm trước khi nối: số dòng mỗi phía, và số dòng SAU khi nối.
--    Nối một–nhiều cho ra MỖI DÒNG BÊN "NHIỀU" một dòng, không phải mỗi chuyến.
-- expect-line: 2707 | 2800 | 2800 | 2832
SELECT (SELECT count(*) FROM trips WHERE status = 'completed')                    AS chuyen_hoan_thanh,
       (SELECT count(*) FROM payments AS p JOIN trips AS t ON t.id = p.trip_id
         WHERE t.status = 'completed')                                             AS gd_cua_chung,
       (SELECT count(*) FROM trips AS t JOIN payments AS p ON p.trip_id = t.id
         WHERE t.status = 'completed')                                             AS dong_inner_join,
       (SELECT count(*) FROM trips AS t LEFT JOIN payments AS p ON p.trip_id = t.id
         WHERE t.status = 'completed')                                             AS dong_left_join;

-- ---------------------------------------------------------------------------
-- 2. LỖI KINH ĐIỂN: cộng cước SAU khi nối với payments. Chuyến có 2 dòng
--    thanh toán (thu lỗi rồi thu lại, trả một phần ví + thẻ, thu trùng…) bị
--    cộng cước 2 lần; chuyến không có dòng nào (cước 0 đ, thiếu thanh toán)
--    biến mất khỏi INNER JOIN (2.675 chuyến còn lại thay vì 2.707).
-- expect-line: 2800 | 2675 | 191644000 | 189686000
SELECT count(*)                AS so_dong,
       count(DISTINCT t.id)    AS so_chuyen,
       sum(t.fare)             AS tong_cuoc_SAI,
       sum(p.amount)           AS tong_gd_moi_trang_thai
FROM trips AS t
JOIN payments AS p ON p.trip_id = t.id
WHERE t.status = 'completed';

-- "Chữa" bằng sum(DISTINCT t.fare) còn SAI hơn: nó cộng các MỨC cước khác
-- nhau (mỗi mức một lần), không phải cước của từng chuyến.
-- expect-line: 182287000 | 11259000
SELECT (SELECT sum(fare) FROM trips WHERE status = 'completed') AS tong_cuoc_dung,
       sum(DISTINCT t.fare)                                    AS sum_distinct_SAI
FROM trips AS t
JOIN payments AS p ON p.trip_id = t.id
WHERE t.status = 'completed';

-- ---------------------------------------------------------------------------
-- 3. Tìm chỗ "nở" dòng: khoá nối không duy nhất ở phía payments.
--    Luôn hỏi trước khi nối: "với mỗi dòng bên trái, bên phải có tối đa mấy dòng?"
-- expect-line: 1 | 2550
-- expect-line: 2 | 125
SELECT so_dong_gd, count(*) AS so_chuyen
FROM (SELECT p.trip_id, count(*) AS so_dong_gd
      FROM payments AS p JOIN trips AS t ON t.id = p.trip_id
      WHERE t.status = 'completed'
      GROUP BY p.trip_id) AS x
GROUP BY so_dong_gd
ORDER BY so_dong_gd;

-- trip_ratings: khoá chính trip_id ⇒ không chuyến nào có quá 1 dòng. Đây là
-- bằng chứng an toàn cho LEFT JOIN trips → trip_ratings (số dòng giữ nguyên).
-- expect-rows: 0
SELECT trip_id FROM trip_ratings GROUP BY trip_id HAVING count(*) > 1;

-- expect-value: 2707
SELECT count(*) FROM trips AS t
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed';

-- ---------------------------------------------------------------------------
-- 4. CÁCH ĐÚNG: gom phía "nhiều" về MỘT dòng mỗi chuyến TRƯỚC, rồi mới nối.
--    Sau khi gom, quan hệ là một–một (hoặc một–không) ⇒ cộng cước an toàn.
--    LEFT JOIN để giữ cả chuyến không có giao dịch nào. Thực thu ≠ cước:
--    chênh +244.000 đ là các sai lệch cài sẵn của chuyến hoàn thành (PG-020).
-- expect-line: 2707 | 2707 | 182287000 | 182531000
SELECT count(*)                       AS so_dong,
       count(DISTINCT t.id)           AS so_chuyen,
       sum(t.fare)                    AS tong_cuoc,
       sum(coalesce(g.thuc_thu, 0))   AS tong_thuc_thu
FROM trips AS t
LEFT JOIN (
  SELECT trip_id,
         sum(CASE kind WHEN 'refund' THEN -amount ELSE amount END) AS thuc_thu
  FROM payments
  WHERE status = 'succeeded'
  GROUP BY trip_id
) AS g ON g.trip_id = t.id
WHERE t.status = 'completed';

-- ---------------------------------------------------------------------------
-- 5. INNER JOIN lặng lẽ bỏ chuyến không có dòng bên phải. Chuyến hoàn thành
--    KHÔNG có giao dịch thành công nào — tìm bằng LEFT JOIN … IS NULL
--    (anti-join). Phần lớn là cước 0 đ (đúng), còn lại là sai lệch (PG-020).
-- expect-line: cước 0 đ — không cần thu | 31
-- expect-line: CÓ cước nhưng chưa thu    | 2
SELECT CASE WHEN t.fare = 0 THEN 'cước 0 đ — không cần thu' ELSE 'CÓ cước nhưng chưa thu' END AS loai,
       count(*) AS so_chuyen
FROM trips AS t
LEFT JOIN payments AS p ON p.trip_id = t.id AND p.status = 'succeeded'
WHERE t.status = 'completed' AND p.id IS NULL
GROUP BY 1
ORDER BY 2 DESC;

-- ---------------------------------------------------------------------------
-- 6. Điều kiện của bảng bên phải: đặt ở ON hay ở WHERE cho kết quả khác nhau.
--    Ở WHERE, dòng NULL do LEFT JOIN sinh ra bị loại (NULL = 'succeeded' không
--    TRUE) ⇒ LEFT JOIN thoái hoá thành INNER JOIN.
-- expect-line: 2674 | 2707
SELECT (SELECT count(DISTINCT t.id) FROM trips AS t
          LEFT JOIN payments AS p ON p.trip_id = t.id
         WHERE t.status = 'completed' AND p.status = 'succeeded')       AS dieu_kien_o_WHERE,
       (SELECT count(DISTINCT t.id) FROM trips AS t
          LEFT JOIN payments AS p ON p.trip_id = t.id AND p.status = 'succeeded'
         WHERE t.status = 'completed')                                  AS dieu_kien_o_ON;

-- ---------------------------------------------------------------------------
-- 7. Hai quan hệ một–nhiều trong CÙNG một câu nhân với nhau: mỗi giao dịch ×
--    mỗi dòng lịch sử. Chuyến 100042 (thẻ lỗi rồi thu lại): 2 × 7 = 14 dòng.
-- expect-line: 100042 | 2 | 7 | 14 | 82000 | 1148000
SELECT t.code,
       count(DISTINCT p.id)   AS so_gd,
       count(DISTINCT e.id)   AS so_su_kien,
       count(*)               AS so_dong_sau_noi,
       max(t.fare)            AS cuoc,
       sum(p.amount)          AS sum_amount_SAI
FROM trips AS t
JOIN payments    AS p ON p.trip_id = t.id
JOIN trip_events AS e ON e.trip_id = t.id
WHERE t.code = 100042
GROUP BY t.code;

-- Trên cả tháng: tổng tiền giao dịch thành công bị nhân lên ~7 lần.
-- expect-line: 183640000 | 1283272000
SELECT (SELECT sum(p.amount) FROM payments AS p WHERE p.status = 'succeeded' AND p.kind = 'charge') AS tong_thu_dung,
       sum(p.amount)                                                                        AS tong_thu_SAI_sau_2_join
FROM payments AS p
JOIN trip_events AS e ON e.trip_id = p.trip_id
WHERE p.status = 'succeeded' AND p.kind = 'charge';

-- ---------------------------------------------------------------------------
-- 8. Nối NHIỀU–MỘT (chuyến → tài xế, thành phố, quán) không làm nở dòng vì
--    phía "một" là khoá chính. Số dòng trước và sau phải bằng nhau.
-- expect-line: 2707 | 2707
SELECT (SELECT count(*) FROM trips WHERE status = 'completed') AS truoc,
       count(*)                                                AS sau
FROM trips AS t
JOIN cities         AS c ON c.id = t.city_id
JOIN drivers        AS d ON d.id = t.driver_id
LEFT JOIN merchants AS m ON m.id = t.merchant_id
WHERE t.status = 'completed';

-- ---------------------------------------------------------------------------
-- 9. Câu hỏi "có hay không" thì đừng JOIN rồi đếm: "bao nhiêu chuyến hoàn
--    thành từng dùng ví VíXanh?" — JOIN đếm DÒNG giao dịch; EXISTS (PG-014)
--    đếm CHUYẾN, không cần DISTINCT.
-- expect-line: 1050 | 961 | 961
SELECT (SELECT count(*) FROM trips AS t JOIN payments AS p ON p.trip_id = t.id
         WHERE t.status = 'completed' AND p.method = 'wallet')              AS join_dem_dong,
       (SELECT count(DISTINCT t.id) FROM trips AS t JOIN payments AS p ON p.trip_id = t.id
         WHERE t.status = 'completed' AND p.method = 'wallet')              AS join_distinct,
       (SELECT count(*) FROM trips AS t
         WHERE t.status = 'completed'
           AND EXISTS (SELECT 1 FROM payments AS p WHERE p.trip_id = t.id AND p.method = 'wallet')) AS exists_dem_chuyen;
