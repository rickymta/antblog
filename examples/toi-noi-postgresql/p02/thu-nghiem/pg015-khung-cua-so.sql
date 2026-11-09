-- PG-015 · thí nghiệm frame (khung cửa sổ) ngoài luồng chính 15-window-function.sql.
-- Chạy sau nền P02 (p01/00 + p01/01 + p02/00 → 02):
--   node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
--     p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02-extra/pg015-khung-cua-so.sql

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- A. Hàm cửa sổ chạy SAU WHERE của cùng cấp truy vấn. Lọc 17–24/10 ngay trong
--    câu có cửa sổ ⇒ "luỹ kế" bắt đầu lại từ 17/10 (718.000), không còn là luỹ
--    kế tháng (15.952.000 ở 15-window-function.sql). Ngày bão 20/10 cũng biến
--    mất vì không có dòng.
-- expect-line: 2026-10-17 | 718000 |  718000
-- expect-line: 2026-10-18 | 945000 | 1663000
-- expect-line: 2026-10-19 | 637000 | 2300000
-- expect-line: 2026-10-21 | 872000 | 3172000
-- expect-line: 2026-10-22 | 785000 | 3957000
-- expect-line: 2026-10-23 | 941000 | 4898000
-- expect-line: 2026-10-24 | 787000 | 5685000
SELECT ngay, doanh_thu,
       sum(doanh_thu) OVER (ORDER BY ngay ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS luy_ke_SAI
FROM (SELECT (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS ngay, sum(fare) AS doanh_thu
      FROM trips WHERE city_id = 3 AND status = 'completed' GROUP BY 1) AS s
WHERE ngay BETWEEN DATE '2026-10-17' AND DATE '2026-10-24'
ORDER BY ngay;

-- ---------------------------------------------------------------------------
-- B. RANGE interval '6 days' chọn ĐÚNG ranh giới 7 ngày lịch trên dữ liệu thưa,
--    nhưng avg() vẫn chia cho SỐ DÒNG có trong khung, không phải 7 ngày: từ
--    21/10 tới 26/10 khung chỉ có 6 dòng (thiếu 20/10) ⇒ avg cao hơn trung bình
--    7 ngày thật (= sum / 7, khớp cột tb7_lich_day của 15-window-function.sql).
-- expect-line: 2026-10-19 | 2026-10-13 | 7 | 828571 | 828571
-- expect-line: 2026-10-21 | 2026-10-15 | 6 | 790167 | 677286
-- expect-line: 2026-10-22 | 2026-10-16 | 6 | 792167 | 679000
-- expect-line: 2026-10-23 | 2026-10-17 | 6 | 816333 | 699714
-- expect-line: 2026-10-24 | 2026-10-18 | 6 | 827833 | 709571
-- expect-line: 2026-10-25 | 2026-10-19 | 6 | 860167 | 737286
-- expect-line: 2026-10-26 | 2026-10-21 | 6 | 925333 | 793143
-- expect-line: 2026-10-27 | 2026-10-21 | 7 | 918143 | 918143
SELECT ngay, ngay_dau_khung, so_dong_trong_khung, avg_range_SAI, tb7_dung
FROM (
  SELECT ngay,
         min(ngay)      OVER r                  AS ngay_dau_khung,
         count(*)       OVER r                  AS so_dong_trong_khung,
         round(avg(doanh_thu) OVER r)           AS avg_range_SAI,
         round(sum(doanh_thu) OVER r / 7.0)     AS tb7_dung
  FROM (SELECT (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS ngay, sum(fare) AS doanh_thu
        FROM trips WHERE city_id = 3 AND status = 'completed' GROUP BY 1) AS s
  WINDOW r AS (ORDER BY ngay RANGE BETWEEN interval '6 days' PRECEDING AND CURRENT ROW)
) AS x
WHERE ngay BETWEEN DATE '2026-10-19' AND DATE '2026-10-27'
ORDER BY ngay;

-- ---------------------------------------------------------------------------
-- C. "So với hôm qua" bằng lag(): trên dữ liệu thưa, dòng trước của 21/10 là
--    19/10 (637.000) chứ không phải hôm qua 20/10 (0 đ). Trên lịch đủ, hôm qua
--    của 21/10 là 0 ⇒ phần trăm thay đổi chia cho 0 — NULLIF trả NULL thay vì lỗi.
-- expect-line: 2026-10-19 | 637000 | 945000 | 945000 |  -32.6
-- expect-line: 2026-10-20 |      0 | 637000 |   NULL | -100.0
-- expect-line: 2026-10-21 | 872000 |      0 | 637000 |   NULL
-- expect-line: 2026-10-22 | 785000 | 872000 | 872000 |  -10.0
SELECT ngay, doanh_thu, hom_qua_lich_day, dong_truoc_du_lieu_thua,
       round(100.0 * (doanh_thu - hom_qua_lich_day) / NULLIF(hom_qua_lich_day, 0), 1) AS thay_doi_pct
FROM (
  SELECT l.ngay,
         coalesce(d.doanh_thu, 0)                              AS doanh_thu,
         lag(coalesce(d.doanh_thu, 0)) OVER (ORDER BY l.ngay)  AS hom_qua_lich_day,
         d.dong_truoc                                          AS dong_truoc_du_lieu_thua
  FROM (SELECT DATE '2026-10-01' + n AS ngay FROM generate_series(0, 29) AS n) AS l
  LEFT JOIN (
    SELECT ngay, doanh_thu, lag(doanh_thu) OVER (ORDER BY ngay) AS dong_truoc
    FROM (SELECT (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS ngay, sum(fare) AS doanh_thu
          FROM trips WHERE city_id = 3 AND status = 'completed' GROUP BY 1) AS s
  ) AS d ON d.ngay = l.ngay
) AS x
WHERE ngay BETWEEN DATE '2026-10-19' AND DATE '2026-10-22'
ORDER BY ngay;

-- ---------------------------------------------------------------------------
-- D. Ba chế độ frame, cùng "1 PRECEDING", trên 18 chuyến khách doanh nghiệp
--    (hai lô đặt cùng giây):
--    ROWS   1 PRECEDING          — dòng ngay trước (vật lý) + dòng hiện tại.
--                                  Khung ROWS dùng ORDER BY không xác định
--                                  (thiếu code) nên có khoá phụ code ở đây.
--    GROUPS 1 PRECEDING          — nhóm ngang hàng (peer group) trước + cả nhóm
--                                  hiện tại: lô 10/10 = 57.000 + 285.000.
--    RANGE  interval '1 hour'    — mọi dòng có giờ đặt trong 1 giờ trước: chỉ
--                                  chính lô (285.000 / 144.000).
-- expect-line: 100130 | 2026-10-02 10:15:00+07 | 45000 |  45000 |  45000 |  45000
-- expect-line: 100500 | 2026-10-05 15:40:00+07 | 75000 | 120000 | 120000 |  75000
-- expect-line: 100669 | 2026-10-07 09:20:00+07 | 57000 | 132000 | 132000 |  57000
-- expect-line: 100971 | 2026-10-10 09:00:00+07 | 24000 |  81000 | 342000 | 285000
-- expect-line: 100972 | 2026-10-10 09:00:00+07 | 53000 |  77000 | 342000 | 285000
-- expect-line: 100973 | 2026-10-10 09:00:00+07 | 20000 |  73000 | 342000 | 285000
-- expect-line: 100974 | 2026-10-10 09:00:00+07 | 49000 |  69000 | 342000 | 285000
-- expect-line: 100975 | 2026-10-10 09:00:00+07 | 78000 | 127000 | 342000 | 285000
-- expect-line: 100976 | 2026-10-10 09:00:00+07 | 61000 | 139000 | 342000 | 285000
-- expect-line: 101224 | 2026-10-12 11:05:00+07 | 27000 |  88000 | 312000 |  27000
-- expect-line: 101555 | 2026-10-15 16:45:00+07 | 57000 |  84000 |  84000 |  57000
-- expect-line: 101882 | 2026-10-18 14:30:00+07 | 23000 |  80000 | 201000 | 144000
-- expect-line: 101883 | 2026-10-18 14:30:00+07 | 53000 |  76000 | 201000 | 144000
-- expect-line: 101884 | 2026-10-18 14:30:00+07 | 19000 |  72000 | 201000 | 144000
-- expect-line: 101885 | 2026-10-18 14:30:00+07 | 49000 |  68000 | 201000 | 144000
-- expect-line: 102149 | 2026-10-21 08:50:00+07 | 31000 |  80000 | 175000 |  31000
-- expect-line: 102611 | 2026-10-25 14:10:00+07 | 61000 |  92000 |  92000 |  61000
-- expect-line: 102897 | 2026-10-28 10:30:00+07 | 27000 |  88000 |  88000 |  27000
SELECT code, requested_at, fare,
       sum(fare) OVER (ORDER BY requested_at, code ROWS BETWEEN 1 PRECEDING AND CURRENT ROW)   AS rows_1,
       sum(fare) OVER (ORDER BY requested_at GROUPS BETWEEN 1 PRECEDING AND CURRENT ROW)       AS groups_1,
       sum(fare) OVER (ORDER BY requested_at
                       RANGE BETWEEN interval '1 hour' PRECEDING AND CURRENT ROW)              AS range_1_gio
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY requested_at, code;

-- ---------------------------------------------------------------------------
-- E. Lỗi khi viết frame/cửa sổ sai.
-- RANGE có khoảng cách cần ĐÚNG MỘT cột ORDER BY (không thêm được khoá phụ).
-- expect-error: 42P20
SELECT sum(fare) OVER (ORDER BY requested_at, code RANGE BETWEEN interval '1 hour' PRECEDING AND CURRENT ROW)
FROM trips LIMIT 1;

-- GROUPS cần ORDER BY (không có thứ tự thì không có "nhóm trước").
-- expect-error: 42P20
SELECT sum(fare) OVER (GROUPS BETWEEN 1 PRECEDING AND CURRENT ROW) FROM trips LIMIT 1;

-- Điểm đầu khung không thể là UNBOUNDED FOLLOWING.
-- expect-error: 42P20
SELECT count(*) OVER (ROWS BETWEEN UNBOUNDED FOLLOWING AND CURRENT ROW) FROM trips LIMIT 1;

-- Cửa sổ đặt tên: được THÊM frame khi gọi, không được ĐÈ ORDER BY đã có…
-- expect-error: 42P20
SELECT code, sum(fare) OVER (w ORDER BY code) FROM trips WINDOW w AS (ORDER BY requested_at) LIMIT 1;

-- …và không chép được cửa sổ đã có frame.
-- expect-error: 42P20
SELECT code, sum(fare) OVER (w ORDER BY code)
FROM trips WINDOW w AS (PARTITION BY city_id ROWS UNBOUNDED PRECEDING) LIMIT 1;

-- Thêm frame vào cửa sổ đặt tên chưa có frame thì hợp lệ.
-- expect-line: 100130 |  45000
-- expect-line: 100500 | 120000
SELECT code, sum(fare) OVER (w ROWS UNBOUNDED PRECEDING) AS luy_ke
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
WINDOW w AS (ORDER BY requested_at, code)
ORDER BY requested_at, code
LIMIT 2;

-- Hàm gộp không bọc được hàm cửa sổ: max(… lag(…) OVER …) ⇒ 42803. Phải tính
-- cửa sổ trong truy vấn con rồi mới gộp ở truy vấn ngoài (bài tập 1).
-- expect-error: 42803
SELECT max(t.requested_at - lag(t.requested_at) OVER (ORDER BY t.requested_at, t.code))
FROM trips AS t JOIN customers AS c ON c.id = t.customer_id
WHERE c.phone = '0910000016';
