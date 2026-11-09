-- PG-011 · thí nghiệm ngoài luồng chính `p02/11-null-case-coalesce.sql`:
-- bảng chân trị khi có NULL, hai bẫy của CASE, tập rỗng, NULL lây trong sum(),
-- kiểu của COALESCE, DISTINCT với NULL.
-- Chạy sau nền P02 (p01/00 + p01/01 + p02/00 → 02), không phụ thuộc file 11:
--   node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
--     p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02-extra/pg011-logic-ba-gia-tri.sql

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- 1. Bảng chân trị AND / OR / NOT khi một vế là NULL ("không biết").
--    false AND NULL = false, true OR NULL = true: kết quả đã chắc chắn bất kể vế kia.
-- expect-line: t    | NULL | NULL | t    | f
-- expect-line: f    | NULL | f    | NULL | t
-- expect-line: NULL | NULL | NULL | NULL | NULL
SELECT a, b, a AND b AS a_and_b, a OR b AS a_or_b, NOT a AS not_a
FROM (VALUES (true, NULL::boolean), (false, NULL::boolean), (NULL::boolean, NULL::boolean)) AS v(a, b);

-- 2. CASE dạng đơn giản so sánh bằng `=` ⇒ nhánh WHEN NULL không bao giờ khớp:
--    153 dòng bỏ trống sao vẫn bị gắn "có sao".
-- expect-line: có sao | 1891
SELECT CASE stars WHEN NULL THEN 'không rõ' ELSE 'có sao' END AS case_don_gian,
       count(*) AS so_danh_gia
FROM trip_ratings
GROUP BY 1
ORDER BY 1;

-- Dạng tìm kiếm với IS NULL mới tách được.
-- expect-line: có sao   | 1738
-- expect-line: không rõ | 153
SELECT CASE WHEN stars IS NULL THEN 'không rõ' ELSE 'có sao' END AS case_tim_kiem,
       count(*) AS so_danh_gia
FROM trip_ratings
GROUP BY 1
ORDER BY 1;

-- 3. Thứ tự nhánh WHEN: đặt nhánh "huỷ" lên đầu ⇒ 166 chuyến huỷ SAU khi đã có
--    tài xế bị dán nhầm nhãn "huỷ trước khi có người nhận" (295 → 461).
-- expect-line: có tài xế                                   | 2711
-- expect-line: không áp dụng — huỷ trước khi có người nhận | 461
-- expect-line: chưa có — đang tìm tài xế                   | 2
SELECT CASE
         WHEN status = 'cancelled'                 THEN 'không áp dụng — huỷ trước khi có người nhận'
         WHEN driver_id IS NOT NULL                THEN 'có tài xế'
         WHEN status IN ('requested', 'searching') THEN 'chưa có — đang tìm tài xế'
         ELSE 'SAI QUY TẮC — lẽ ra phải có tài xế'
       END AS nhan_sai_thu_tu,
       count(*) AS so_chuyen
FROM trips
GROUP BY 1
ORDER BY 2 DESC;

-- expect-value: 166
SELECT count(*) FROM trips WHERE status = 'cancelled' AND driver_id IS NOT NULL;

-- 4. CASE không có ELSE trả NULL cho dòng không khớp nhánh nào — và count(cột)
--    bỏ qua NULL, nên đây là cách đếm có điều kiện "kiểu cũ" (PG-012: FILTER).
-- expect-line: 3174 | 461
SELECT count(*) AS tong,
       count(CASE WHEN status = 'cancelled' THEN 1 END) AS dem_case_khong_else
FROM trips;

-- 5. Hàm tổng hợp trên tập RỖNG: count ra 0, sum và avg ra NULL.
--    Đà Nẵng ngày 20/10 (bão) không có chuyến nào.
-- expect-line: 0 | NULL | NULL
SELECT count(*) AS so_chuyen, sum(fare) AS tong_cuoc, avg(fare) AS cuoc_tb
FROM trips
WHERE city_id = 3
  AND requested_at >= '2026-10-20 00:00+07' AND requested_at < '2026-10-21 00:00+07';

-- 6. NULL lây qua phép cộng NẰM TRONG sum(): sum(fare + tip) bỏ hẳn cước của
--    541 lượt bỏ qua tip. Cộng hai tổng, hoặc COALESCE trước khi cộng, mới đúng.
-- expect-line: 1891 | 128577000 | 5565000 | 134142000 | 98163000 | 134142000
SELECT count(*)                          AS so_danh_gia,
       sum(t.fare)                       AS tong_cuoc,
       sum(r.tip)                        AS tong_tip,
       sum(t.fare) + sum(r.tip)          AS cong_hai_tong,
       sum(t.fare + r.tip)               AS tong_cua_tong_dong,
       sum(t.fare + coalesce(r.tip, 0))  AS tong_dong_coalesce
FROM trips AS t
JOIN trip_ratings AS r ON r.trip_id = t.id;

-- 7. Các đối số của COALESCE phải quy về MỘT kiểu: cột integer gặp chữ ⇒ PostgreSQL
--    thử đọc chữ thành integer và lỗi.
-- expect-error: 22P02
SELECT coalesce(d.code, 'đang tìm…')
FROM trips AS t LEFT JOIN drivers AS d ON d.id = t.driver_id
WHERE t.status = 'searching';

-- expect-line: 103172 | đang tìm…
SELECT t.code, coalesce(d.code::text, 'đang tìm…') AS tai_xe
FROM trips AS t LEFT JOIN drivers AS d ON d.id = t.driver_id
WHERE t.status = 'searching'
ORDER BY t.code;

-- 8. DISTINCT (và GROUP BY) gom mọi NULL thành MỘT nhóm; count(DISTINCT) thì bỏ NULL.
-- expect-line: 6 | 5
SELECT (SELECT count(*) FROM (SELECT DISTINCT stars FROM trip_ratings) AS s) AS so_dong_distinct,
       (SELECT count(DISTINCT stars) FROM trip_ratings)                      AS count_distinct;
