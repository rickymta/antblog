-- PG-011 · lời giải bài tập.
-- Chạy sau nền P02 (p01/00 + p01/01 + p02/00 → 02):
--   node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
--     p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02-extra/pg011-bai-tap.sql

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Bài 1 · Nghĩa của merchant_id. Nhánh ELSE rỗng nhờ ck_trips_merchant_only_food.
-- expect-line: không áp dụng — không phải đơn đồ ăn | 1742
-- expect-line: có quán                              | 1432
SELECT CASE
         WHEN merchant_id IS NOT NULL THEN 'có quán'
         WHEN kind <> 'food'          THEN 'không áp dụng — không phải đơn đồ ăn'
         ELSE 'LỖI — đơn đồ ăn thiếu quán'
       END AS y_nghia_merchant_id,
       count(*) AS so_chuyen
FROM trips
GROUP BY 1
ORDER BY 2 DESC;

-- Bài 2 · Cước + tip của chuyến hoàn thành tháng 10 theo thành phố. sum(fare + tip)
--         mất cước của chuyến không có dòng đánh giá và chuyến bỏ qua tip.
-- expect-line: DN |  389 | 26416000 |  805000 | 13866000 | 27221000
-- expect-line: HN |  941 | 62991000 | 1990000 | 34957000 | 64981000
-- expect-line: SG | 1377 | 92880000 | 2770000 | 49340000 | 95650000
SELECT c.code AS tp,
       count(*)                                  AS hoan_thanh,
       sum(t.fare)                               AS tong_cuoc,
       sum(r.tip)                                AS tong_tip,
       sum(t.fare + r.tip)                       AS sai_cong_trong_sum,
       sum(t.fare) + coalesce(sum(r.tip), 0)     AS thu_nhap_cuoc_cong_tip
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
  AND t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY c.code
ORDER BY c.code COLLATE "C";

-- Bài 3 · Điểm sao ngày 20/10 theo thành phố: Đà Nẵng không có chuyến ⇒ avg NULL,
--         hiển thị bằng chữ thay vì 0.
-- expect-line: DN |  0 |  0 | NULL | không có chuyến
-- expect-line: HN | 32 | 22 | 4.23 | 4.23
-- expect-line: SG | 42 | 28 | 4.39 | 4.39
SELECT c.code AS tp,
       count(t.id)                       AS hoan_thanh,
       count(r.stars)                    AS co_sao,
       round(avg(r.stars), 2)            AS diem_tb,
       coalesce(round(avg(r.stars), 2)::text,
                CASE WHEN count(t.id) = 0 THEN 'không có chuyến'
                     ELSE 'chưa có lượt chấm sao' END) AS hien_thi
FROM cities AS c
LEFT JOIN trips AS t
  ON t.city_id = c.id
 AND t.status = 'completed'
 AND t.requested_at >= '2026-10-20 00:00+07' AND t.requested_at < '2026-10-21 00:00+07'
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
GROUP BY c.code
ORDER BY c.code COLLATE "C";
