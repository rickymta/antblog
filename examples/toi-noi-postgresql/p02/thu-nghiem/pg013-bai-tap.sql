-- PG-013 · lời giải bài tập.
-- Nền: p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Bài 1. Điểm sao trung bình theo thành phố: trips → cities (nhiều–một) và
-- trips → trip_ratings (0..1, khoá chính trip_id) ⇒ không nối nào làm nở dòng.
-- expect-line: DN |  389 | 266 | 4.30
-- expect-line: HN |  941 | 670 | 4.17
-- expect-line: SG | 1377 | 955 | 4.23
SELECT c.code AS tp,
       count(*)                 AS chuyen_hoan_thanh,
       count(r.trip_id)         AS co_danh_gia,
       round(avg(r.stars), 2)   AS sao_tb
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
GROUP BY c.code
ORDER BY c.code COLLATE "C";

-- Bài 2. Tiền theo phương thức thanh toán (chuyến hoàn thành, giao dịch thành công).
-- Cách SAI: gán cả cước chuyến cho MỖI dòng giao dịch ⇒ 100048 (ví 104.000 + thẻ
-- 15.000) được tính 119.000 ở ví VÀ 119.000 ở thẻ.
-- expect-line: card   |  686 |  686 | 48371000
-- expect-line: cash   | 1054 | 1054 | 69538000
-- expect-line: wallet |  963 |  961 | 66580000
SELECT p.method, count(*) AS so_gd, count(DISTINCT t.id) AS so_chuyen, sum(t.fare) AS tien_SAI
FROM trips AS t
JOIN payments AS p ON p.trip_id = t.id
WHERE t.status = 'completed' AND p.status = 'succeeded'
GROUP BY p.method
ORDER BY p.method COLLATE "C";

-- Tổng của cách sai lớn hơn cả cước lẫn thực thu.
-- expect-value: 184489000
SELECT sum(t.fare)
FROM trips AS t
JOIN payments AS p ON p.trip_id = t.id
WHERE t.status = 'completed' AND p.status = 'succeeded';

-- Cách ĐÚNG: câu hỏi nói về tiền đi qua từng phương thức ⇒ đơn vị là GIAO DỊCH,
-- cộng p.amount (có dấu) chứ không cộng t.fare. Tổng = thực thu 182.531.000.
-- expect-line: card   |  686 | 46932000
-- expect-line: cash   | 1054 | 69524000
-- expect-line: wallet |  963 | 66075000
SELECT p.method, count(*) AS so_gd,
       sum(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END) AS tien
FROM payments AS p
JOIN trips AS t ON t.id = p.trip_id
WHERE t.status = 'completed' AND p.status = 'succeeded'
GROUP BY p.method
ORDER BY p.method COLLATE "C";

-- expect-value: 182531000
SELECT sum(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END)
FROM payments AS p
JOIN trips AS t ON t.id = p.trip_id
WHERE t.status = 'completed' AND p.status = 'succeeded';

-- Bài 3. Dự đoán số dòng: trips (hoàn thành) LEFT JOIN payments LEFT JOIN trip_ratings.
-- payments làm nở 2.707 → 2.832; trip_ratings (0..1) không làm nở thêm.
-- Sao TB tính trên kết quả đó bị "trọng số" theo số giao dịch: 4,24 thay vì 4,22.
-- expect-line: 2832 | 4.24
SELECT count(*), round(avg(r.stars), 2)
FROM trips AS t
LEFT JOIN payments     AS p ON p.trip_id = t.id
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed';

-- Sửa: gom payments về một dòng mỗi chuyến trước ⇒ 2.707 dòng, sao TB 4,22 như PG-011.
-- expect-line: 2707 | 4.22 | 2800
SELECT count(*), round(avg(r.stars), 2), sum(g.so_gd)
FROM trips AS t
LEFT JOIN (SELECT trip_id, count(*) AS so_gd FROM payments GROUP BY trip_id) AS g ON g.trip_id = t.id
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed';
