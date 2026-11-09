-- PG-016 · lời giải bài tập.
-- Chạy sau nền P02 (p01/00 + p01/01 + p02/00 → 02):
--   node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
--     p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02-extra/pg016-bai-tap.sql

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Bài 1 · Chuyến đặt trong tuần 12–18/10 giờ VN (thứ hai → chủ nhật). Nửa mở,
--         hai biên ghi +07; đối chiếu bằng date_trunc 3 tham số. Chạy lại ở
--         phiên UTC: không đổi.
-- expect-line: 748 | 748
SELECT count(*) FILTER (WHERE requested_at >= '2026-10-12 00:00:00+07'
                          AND requested_at <  '2026-10-19 00:00:00+07')                          AS nua_mo,
       count(*) FILTER (WHERE date_trunc('week', requested_at, 'Asia/Ho_Chi_Minh') = '2026-10-12 00:00:00+07') AS date_trunc_tuan
FROM trips;

SET TIME ZONE 'UTC';

-- expect-line: 748 | 748
SELECT count(*) FILTER (WHERE requested_at >= '2026-10-12 00:00:00+07'
                          AND requested_at <  '2026-10-19 00:00:00+07')                          AS nua_mo,
       count(*) FILTER (WHERE date_trunc('week', requested_at, 'Asia/Ho_Chi_Minh') = '2026-10-12 00:00:00+07') AS date_trunc_tuan
FROM trips;

SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Bài 2 · "Ngày vận hành" của ca đêm bắt đầu 04:00 giờ VN. Ngày vận hành 15/10
--         = [15/10 04:00, 16/10 04:00) +07. Cách gom: lùi 4 giờ trên giờ VN rồi
--         lấy ngày. Hai cách phải khớp.
-- expect-value: 107
SELECT count(*) FROM trips
WHERE requested_at >= '2026-10-15 04:00:00+07' AND requested_at < '2026-10-16 04:00:00+07';

-- expect-line: 2026-10-14 |   6
-- expect-line: 2026-10-15 | 107
-- expect-line: 2026-10-16 | 102
SELECT ((requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh') - interval '4 hours')::date AS ngay_van_hanh,
       count(*) AS so_chuyen
FROM trips
WHERE requested_at >= '2026-10-15 00:00:00+07' AND requested_at < '2026-10-17 04:00:00+07'
GROUP BY 1
ORDER BY 1;

-- Kiểm HAI BIÊN của công thức gom bằng giá trị dựng tay (không có chuyến nào
-- đặt đúng 04:00 trong dữ liệu): 03:59:59.5 còn thuộc ngày trước, 04:00:00
-- thuộc ngày mới.
-- expect-line: 2026-10-15 03:59:59.5+07 | 2026-10-14
-- expect-line: 2026-10-15 04:00:00+07   | 2026-10-15
-- expect-line: 2026-10-16 03:59:59.5+07 | 2026-10-15
-- expect-line: 2026-10-16 04:00:00+07   | 2026-10-16
SELECT v.thoi_diem,
       ((v.thoi_diem AT TIME ZONE 'Asia/Ho_Chi_Minh') - interval '4 hours')::date AS ngay_van_hanh
FROM (VALUES (TIMESTAMPTZ '2026-10-15 03:59:59.5+07'), (TIMESTAMPTZ '2026-10-15 04:00:00+07'),
             (TIMESTAMPTZ '2026-10-16 03:59:59.5+07'), (TIMESTAMPTZ '2026-10-16 04:00:00+07')) AS v (thoi_diem)
ORDER BY v.thoi_diem;

-- Bài 3 · Biên tháng từ tham số kiểu date (ngày đầu tháng). Tháng 10 giờ VN
--         gồm cả 6 chuyến đang chạy sáng 31/10. Tháng 10 "theo UTC" bắt đầu
--         07:00 sáng 01/10 giờ VN ⇒ thiếu các chuyến đặt 00:00–06:59 ngày 01/10.
-- expect-line: 2026-10-01 00:00:00+07 | 2026-11-01 00:00:00+07 | 3174 | 3162
WITH tham_so AS (SELECT DATE '2026-10-01' AS thang),
     bien AS (
       SELECT thang::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh'                         AS tu,
              (thang + interval '1 month')::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh' AS den_truoc
       FROM tham_so)
SELECT b.tu, b.den_truoc,
       (SELECT count(*) FROM trips WHERE requested_at >= b.tu AND requested_at < b.den_truoc) AS thang_vn,
       (SELECT count(*) FROM trips WHERE requested_at >= '2026-10-01 00:00:00+00'
                                     AND requested_at <  '2026-11-01 00:00:00+00')            AS thang_utc
FROM bien AS b;

-- 12 chuyến chênh lệch đúng là các chuyến đặt trước 07:00 sáng 01/10 giờ VN.
-- expect-value: 12
SELECT count(*) FROM trips
WHERE requested_at >= '2026-10-01 00:00:00+07' AND requested_at < '2026-10-01 07:00:00+07';
