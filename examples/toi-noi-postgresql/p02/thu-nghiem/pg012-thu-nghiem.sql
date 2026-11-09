-- PG-012 · thí nghiệm ngoài luồng chính `p02/12-aggregate-group-by.sql`:
-- ba lỗi vị trí điều kiện (WHERE / HAVING), FILTER so với CASE, số dòng của kỳ
-- báo cáo, NULL của ROLLUP trùng với NULL dữ liệu, và trung bình cộng các tỉ lệ.
-- Chạy sau nền P02 (p01/00 + p01/01 + p02/00 → 02), không phụ thuộc file 12:
--   node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
--     p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02-extra/pg012-thu-nghiem.sql

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- 1. Hàm tổng hợp trong WHERE: WHERE chạy trên từng dòng, lúc đó chưa có nhóm.
-- expect-error: 42803
SELECT c.code, count(*)
FROM trips AS t JOIN cities AS c ON c.id = t.city_id
WHERE count(*) > 1000
GROUP BY c.code;

-- 2. Tên đặt bằng AS chưa dùng được trong HAVING (select list tính SAU HAVING).
-- expect-error: 42703
SELECT c.code, count(*) AS so_chuyen
FROM trips AS t JOIN cities AS c ON c.id = t.city_id
GROUP BY c.code
HAVING so_chuyen > 1000;

-- 3. Điều kiện trên DÒNG đặt nhầm vào HAVING: mỗi nhóm có nhiều status khác nhau.
-- expect-error: 42803
SELECT c.code, count(*)
FROM trips AS t JOIN cities AS c ON c.id = t.city_id
GROUP BY c.code
HAVING t.status = 'completed';

-- 4. Kỳ báo cáo tháng 10: 3.168 dòng, tất cả đã kết thúc; dòng sớm nhất / muộn nhất.
-- expect-line: 3168 | 3168 | 2026-10-01 00:29:00+07 | 2026-10-30 23:29:00+07
SELECT count(*)                                                       AS so_dong,
       count(*) FILTER (WHERE status IN ('completed', 'cancelled'))   AS da_ket_thuc,
       min(requested_at)                                              AS dat_som_nhat,
       max(requested_at)                                              AS dat_muon_nhat
FROM trips
WHERE requested_at >= '2026-10-01 00:00+07' AND requested_at < '2026-10-31 00:00+07';

-- 5. FILTER và hai cách viết bằng CASE cho cùng con số.
-- expect-line: 461 | 461 | 461 | 182287000 | 182287000
SELECT count(*) FILTER (WHERE status = 'cancelled')            AS huy_filter,
       count(CASE WHEN status = 'cancelled' THEN 1 END)        AS huy_count_case,
       sum(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END)   AS huy_sum_case,
       sum(fare) FILTER (WHERE status = 'completed')           AS doanh_thu_filter,
       sum(CASE WHEN status = 'completed' THEN fare END)       AS doanh_thu_case
FROM trips
WHERE requested_at >= '2026-10-01 00:00+07' AND requested_at < '2026-10-31 00:00+07';

-- 6. ROLLUP trên cột CÓ THỂ NULL: chuyến Đà Nẵng theo khu đón. Hai dòng có khu
--    NULL — một là 4 chuyến mất GPS, một là tổng 460. COALESCE gắn nhãn cả hai là
--    'TỔNG'; grouping() mới tách được.
-- expect-line: DN-BI-01 | 112
-- expect-line: DN-BI-02 | 119
-- expect-line: DN-TT-01 | 113
-- expect-line: DN-TT-02 | 112
-- expect-line: TỔNG     |   4
-- expect-line: TỔNG     | 460
SELECT coalesce(z.code, 'TỔNG') AS khu_sai, count(*) AS so_chuyen
FROM trips AS t
LEFT JOIN areas AS z ON z.id = t.pickup_area_id
WHERE t.city_id = 3
GROUP BY ROLLUP (z.code)
ORDER BY grouping(z.code), z.code COLLATE "C" NULLS LAST;

-- expect-line: DN-BI-01           | 112
-- expect-line: DN-BI-02           | 119
-- expect-line: DN-TT-01           | 113
-- expect-line: DN-TT-02           | 112
-- expect-line: không rõ (mất GPS) |   4
-- expect-line: TỔNG               | 460
SELECT CASE WHEN grouping(z.code) = 1 THEN 'TỔNG'
            ELSE coalesce(z.code, 'không rõ (mất GPS)') END AS khu,
       count(*) AS so_chuyen
FROM trips AS t
LEFT JOIN areas AS z ON z.id = t.pickup_area_id
WHERE t.city_id = 3
GROUP BY ROLLUP (z.code)
ORDER BY grouping(z.code), z.code COLLATE "C" NULLS LAST;

-- 7. Tỉ lệ không lấy trung bình cộng được: trung bình 9 tỉ lệ nhóm (thành phố ×
--    dịch vụ) khác tỉ lệ huỷ thật của cả tháng (461 / 3.168).
-- expect-line: 14.55 | 14.35
SELECT round(100.0 * count(*) FILTER (WHERE status = 'cancelled') / count(*), 2) AS ty_le_ca_thang,
       (SELECT round(avg(ty_le), 2)
        FROM (SELECT 100.0 * count(*) FILTER (WHERE status = 'cancelled') / count(*) AS ty_le
              FROM trips
              WHERE requested_at >= '2026-10-01 00:00+07' AND requested_at < '2026-10-31 00:00+07'
              GROUP BY city_id, kind) AS nhom) AS tb_cong_9_ty_le
FROM trips
WHERE requested_at >= '2026-10-01 00:00+07' AND requested_at < '2026-10-31 00:00+07';
