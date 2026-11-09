-- PG-012 · lời giải bài tập.
-- Chạy sau nền P02 (p01/00 + p01/01 + p02/00 → 02):
--   node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
--     p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02-extra/pg012-bai-tap.sql

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Bài 1 · Bảng tháng 10 theo dịch vụ (cả nước). Cột da_ket_thuc cộng lại = 3.168,
--         doanh_thu cộng lại = 182.287.000.
-- expect-line: food   | 1430 | 1201 | 229 | 16.0 | 102825000 | 85616
-- expect-line: parcel |  488 |  409 |  79 | 16.2 |  19711000 | 48193
-- expect-line: ride   | 1250 | 1097 | 153 | 12.2 |  59751000 | 54468
SELECT t.kind,
       count(*)                                                                     AS da_ket_thuc,
       count(*) FILTER (WHERE t.status = 'completed')                               AS hoan_thanh,
       count(*) FILTER (WHERE t.status = 'cancelled')                               AS huy,
       round(100.0 * count(*) FILTER (WHERE t.status = 'cancelled') / count(*), 1)  AS ty_le_huy_pct,
       sum(t.fare) FILTER (WHERE t.status = 'completed')                            AS doanh_thu,
       round(avg(t.fare) FILTER (WHERE t.status = 'completed'))                     AS cuoc_tb
FROM trips AS t
WHERE t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY t.kind
ORDER BY t.kind COLLATE "C";

-- Bài 2 · Quán có ít nhất 50 đơn trong tháng VÀ tỉ lệ huỷ từ 20 % trở lên.
--         Nhóm theo m.id (khoá chính) nên được chọn m.name.
-- expect-line: Bún Bò Lab 201  | HN | 63 | 17 | 27.0
-- expect-line: Bún Bò Lab 101  | SG | 70 | 17 | 24.3
-- expect-line: Bánh Mì Lab 203 | HN | 58 | 13 | 22.4
-- expect-line: Gà Rán Lab 105  | SG | 54 | 11 | 20.4
SELECT m.name, c.code AS tp,
       count(*)                                                                     AS don,
       count(*) FILTER (WHERE t.status = 'cancelled')                               AS huy,
       round(100.0 * count(*) FILTER (WHERE t.status = 'cancelled') / count(*), 1)  AS ty_le_huy_pct
FROM trips AS t
JOIN merchants AS m ON m.id = t.merchant_id
JOIN cities AS c ON c.id = m.city_id
WHERE t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY m.id, c.code
HAVING count(*) >= 50
   AND count(*) FILTER (WHERE t.status = 'cancelled') * 100.0 / count(*) >= 20
ORDER BY ty_le_huy_pct DESC, m.name COLLATE "C";

-- Bài 3 · Tài xế có điểm sao trung bình dưới 4,0 tính trên ÍT NHẤT 35 lượt có sao.
--         Ngưỡng đặt trên count(r.stars) — mẫu số của avg — chứ không phải count(*).
-- expect-line: 8202 | 49 | 36 | 3.94
-- expect-line: 8211 | 55 | 40 | 3.95
-- expect-line: 8215 | 55 | 39 | 3.95
-- expect-line: 8209 | 58 | 38 | 3.97
SELECT d.code AS tai_xe,
       count(*)               AS hoan_thanh,
       count(r.stars)         AS co_sao,
       round(avg(r.stars), 2) AS diem_tb
FROM trips AS t
JOIN drivers AS d ON d.id = t.driver_id
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
  AND t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY d.code
HAVING count(r.stars) >= 35 AND avg(r.stars) < 4
ORDER BY diem_tb, tai_xe;

-- Cùng câu nhưng ngưỡng nhầm trên count(*) (số chuyến hoàn thành): lọt thêm 8105
-- (31 lượt có sao) và 8110 (33 lượt có sao).
-- expect-line: 8105 | 51 | 31 | 3.74
-- expect-line: 8110 | 57 | 33 | 3.91
-- expect-line: 8202 | 49 | 36 | 3.94
-- expect-line: 8211 | 55 | 40 | 3.95
-- expect-line: 8215 | 55 | 39 | 3.95
-- expect-line: 8209 | 58 | 38 | 3.97
SELECT d.code AS tai_xe,
       count(*)               AS hoan_thanh,
       count(r.stars)         AS co_sao,
       round(avg(r.stars), 2) AS diem_tb
FROM trips AS t
JOIN drivers AS d ON d.id = t.driver_id
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
  AND t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY d.code
HAVING count(*) >= 35 AND avg(r.stars) < 4
ORDER BY diem_tb, tai_xe;
