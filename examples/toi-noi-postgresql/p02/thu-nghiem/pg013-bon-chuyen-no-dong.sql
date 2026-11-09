-- PG-013 · thí nghiệm của bài: nối một–nhiều trên 4 chuyến, rồi tách 191.644.000.
-- Nền: p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- 1. Bốn chuyến hoàn thành, mỗi chuyến một kiểu: 1 giao dịch, thẻ lỗi rồi thu lại,
--    trả một phần ví + thẻ, không có giao dịch nào.
-- expect-line: 100041 | 127000 | 1
-- expect-line: 100042 |  82000 | 2
-- expect-line: 100048 | 119000 | 2
-- expect-line: 100724 |  76000 | 0
SELECT t.code, t.fare,
       (SELECT count(*) FROM payments AS p WHERE p.trip_id = t.id) AS so_gd
FROM trips AS t
WHERE t.code IN (100041, 100042, 100048, 100724)
ORDER BY t.code;

-- 2. INNER JOIN: mỗi CẶP (chuyến, giao dịch) khớp là một dòng ⇒ 5 dòng, 100724 biến mất.
-- expect-line: 100041 | 127000 | card   | succeeded | 127000
-- expect-line: 100042 |  82000 | card   | failed    |  82000
-- expect-line: 100042 |  82000 | card   | succeeded |  82000
-- expect-line: 100048 | 119000 | wallet | succeeded | 104000
-- expect-line: 100048 | 119000 | card   | succeeded |  15000
SELECT t.code, t.fare, p.method, p.status, p.amount
FROM trips AS t
JOIN payments AS p ON p.trip_id = t.id
WHERE t.code IN (100041, 100042, 100048, 100724)
ORDER BY t.code, p.id;

-- 3. LEFT JOIN giữ 100724 với các cột bên phải là NULL ⇒ 6 dòng.
-- expect-line: 100041 | 127000 | card   | succeeded | 127000
-- expect-line: 100042 |  82000 | card   | failed    |  82000
-- expect-line: 100042 |  82000 | card   | succeeded |  82000
-- expect-line: 100048 | 119000 | wallet | succeeded | 104000
-- expect-line: 100048 | 119000 | card   | succeeded |  15000
-- expect-line: 100724 |  76000 | NULL   | NULL      | NULL
SELECT t.code, t.fare, p.method, p.status, p.amount
FROM trips AS t
LEFT JOIN payments AS p ON p.trip_id = t.id
WHERE t.code IN (100041, 100042, 100048, 100724)
ORDER BY t.code, p.id;

-- 4. Cộng cước của 4 chuyến theo ba cách.
-- expect-line: 404000 | 529000 | 605000
SELECT (SELECT sum(fare) FROM trips WHERE code IN (100041, 100042, 100048, 100724)) AS dung,
       (SELECT sum(t.fare) FROM trips AS t JOIN payments AS p ON p.trip_id = t.id
         WHERE t.code IN (100041, 100042, 100048, 100724))                         AS sau_inner_join,
       (SELECT sum(t.fare) FROM trips AS t LEFT JOIN payments AS p ON p.trip_id = t.id
         WHERE t.code IN (100041, 100042, 100048, 100724))                         AS sau_left_join;

-- 5. Tách khoản chênh 191.644.000 − 182.287.000 = 9.357.000 trên cả tháng:
--    + cước của 125 chuyến có 2 dòng giao dịch (bị cộng thêm một lần)
--    − cước của 32 chuyến không có dòng nào (rơi khỏi INNER JOIN; 31 chuyến 0 đ + 100724)
-- expect-line: 125 | 9433000 | 32 | 76000 | 191644000
SELECT (SELECT count(*) FROM trips AS t WHERE t.status = 'completed'
          AND (SELECT count(*) FROM payments AS p WHERE p.trip_id = t.id) = 2)    AS chuyen_2_dong,
       (SELECT sum(t.fare) FROM trips AS t WHERE t.status = 'completed'
          AND (SELECT count(*) FROM payments AS p WHERE p.trip_id = t.id) = 2)    AS cuoc_bi_cong_them,
       (SELECT count(*) FROM trips AS t WHERE t.status = 'completed'
          AND NOT EXISTS (SELECT 1 FROM payments AS p WHERE p.trip_id = t.id))    AS chuyen_0_dong,
       (SELECT sum(t.fare) FROM trips AS t WHERE t.status = 'completed'
          AND NOT EXISTS (SELECT 1 FROM payments AS p WHERE p.trip_id = t.id))    AS cuoc_bi_mat,
       182287000 + 9433000 - 76000                                                AS tai_lap;

-- 6. Khoá phía "nhiều" có duy nhất không? Đếm nhóm trùng trước khi nối.
-- expect-line: payments.trip_id      | 137
-- expect-line: trip_ratings.trip_id  |   0
-- expect-line: trip_events.trip_id   | 3173
SELECT 'payments.trip_id' AS khoa, count(*) AS so_gia_tri_lap
FROM (SELECT trip_id FROM payments GROUP BY trip_id HAVING count(*) > 1) AS x
UNION ALL
SELECT 'trip_ratings.trip_id', count(*)
FROM (SELECT trip_id FROM trip_ratings GROUP BY trip_id HAVING count(*) > 1) AS x
UNION ALL
SELECT 'trip_events.trip_id', count(*)
FROM (SELECT trip_id FROM trip_events GROUP BY trip_id HAVING count(*) > 1) AS x;
