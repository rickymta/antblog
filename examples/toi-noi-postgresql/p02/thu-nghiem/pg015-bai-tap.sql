-- PG-015 · lời giải bài tập.
-- Chạy sau nền P02 (p01/00 + p01/01 + p02/00 → 02):
--   node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
--     p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02-extra/pg015-bai-tap.sql

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Bài 1 · Khách 0910000016 (36 chuyến): ba khoảng nghỉ dài nhất giữa hai chuyến
--         liên tiếp. lag() tính ở truy vấn con, sắp xếp/giới hạn ở truy vấn ngoài.
-- expect-line: 102664 | 102372 | 2 days 09:57:00
-- expect-line: 101792 | 101634 | 1 day 07:15:00
-- expect-line: 103123 | 102984 | 1 day 06:40:00
SELECT code, chuyen_truoc, khoang_nghi
FROM (
  SELECT t.code,
         lag(t.code) OVER w                         AS chuyen_truoc,
         t.requested_at - lag(t.requested_at) OVER w AS khoang_nghi
  FROM trips AS t
  JOIN customers AS c ON c.id = t.customer_id
  WHERE c.phone = '0910000016'
  WINDOW w AS (ORDER BY t.requested_at, t.code)
) AS x
ORDER BY khoang_nghi DESC NULLS LAST, code
LIMIT 3;

-- expect-value: 36
SELECT count(*) FROM trips AS t JOIN customers AS c ON c.id = t.customer_id WHERE c.phone = '0910000016';

-- Bài 2 · Ngày đầu tiên doanh thu luỹ kế tháng 10 của mỗi thành phố đạt nửa
--         doanh thu tháng. Hai cửa sổ: luỹ kế (ROWS) và tổng cả nhóm (không
--         ORDER BY ⇒ cả partition).
-- expect-line: DN | 2026-10-14 | 26416000
-- expect-line: HN | 2026-10-15 | 62991000
-- expect-line: SG | 2026-10-16 | 92880000
SELECT tp, min(ngay) AS ngay_vuot_nua, max(tong_thang) AS tong_thang
FROM (
  SELECT c.code AS tp, s.ngay,
         sum(s.doanh_thu) OVER (PARTITION BY c.code ORDER BY s.ngay ROWS UNBOUNDED PRECEDING) AS luy_ke,
         sum(s.doanh_thu) OVER (PARTITION BY c.code)                                         AS tong_thang
  FROM (SELECT city_id, (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS ngay, sum(fare) AS doanh_thu
        FROM trips WHERE status = 'completed' GROUP BY 1, 2) AS s
  JOIN cities AS c ON c.id = s.city_id
) AS x
WHERE luy_ke * 2 >= tong_thang
GROUP BY tp
ORDER BY tp COLLATE "C";

-- Bài 3 · Tài xế 8101: tổng cước 3 chuyến hoàn thành NGAY TRƯỚC mỗi chuyến
--         (không gồm chuyến hiện tại) — frame ROWS 3 PRECEDING AND 1 PRECEDING.
--         Khung rỗng ở chuyến đầu ⇒ sum = NULL, count = 0.
-- expect-line: 100275 |  81000 |   NULL | 0
-- expect-line: 100307 |  97000 |  81000 | 1
-- expect-line: 100467 | 117000 | 178000 | 2
-- expect-line: 100486 |  37000 | 295000 | 3
-- expect-line: 100501 |  63000 | 251000 | 3
SELECT t.code, t.fare,
       sum(t.fare) OVER w AS ba_chuyen_truoc,
       count(*)    OVER w AS so_chuyen_trong_khung
FROM trips AS t
JOIN drivers AS d ON d.id = t.driver_id
WHERE d.code = 8101 AND t.status = 'completed'
WINDOW w AS (ORDER BY t.completed_at, t.code ROWS BETWEEN 3 PRECEDING AND 1 PRECEDING)
ORDER BY t.completed_at, t.code
LIMIT 5;

-- Bài 3 · cách viết khác: khung "3 dòng trước + dòng hiện tại" rồi EXCLUDE
--         CURRENT ROW. Thứ tự (completed_at, code) không có dòng ngang hàng nên
--         hai cách cho cùng kết quả trên MỌI chuyến hoàn thành của 8101.
-- expect-line: 68 | 0
SELECT count(*) AS so_chuyen,
       count(*) FILTER (WHERE a IS DISTINCT FROM b) AS so_dong_khac
FROM (
  SELECT sum(t.fare) OVER (ORDER BY t.completed_at, t.code ROWS BETWEEN 3 PRECEDING AND 1 PRECEDING) AS a,
         sum(t.fare) OVER (ORDER BY t.completed_at, t.code
                           ROWS BETWEEN 3 PRECEDING AND CURRENT ROW EXCLUDE CURRENT ROW)       AS b
  FROM trips AS t
  JOIN drivers AS d ON d.id = t.driver_id
  WHERE d.code = 8101 AND t.status = 'completed'
) AS x;
