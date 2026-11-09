-- P02 · 16-date-time-timezone.sql — PG-016 `postgresql-date-time-timezone`
-- Ngày giờ và múi giờ trong truy vấn: bộ lọc NGÀY NGHIỆP VỤ với kiểm tra HAI
-- BIÊN.
--
-- Nhắc lại PG-004: timestamptz lưu một KHOẢNH KHẮC tuyệt đối; phiên (TimeZone)
-- chỉ quyết định cách HIỂN THỊ và cách HIỂU chuỗi không ghi múi giờ. "Ngày
-- 15/10" của Tới Nơi là ngày theo giờ Việt Nam (+07, không có giờ mùa hè):
-- từ 2026-10-15 00:00 +07 (= 14/10 17:00 UTC) tới TRƯỚC 2026-10-16 00:00 +07.
--
-- Fixture có 5 chuyến cài sát biên (02-kiem-tra-fixture.sql, H5):
--   101489  15/10 00:00:00     101502  15/10 06:59:59     101503  15/10 07:00:00
--   101594  15/10 23:59:59.5   101595  16/10 00:00:00   (giờ VN)

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- 1. Cùng một khoảnh khắc, hai cách hiển thị.
-- expect-line: 101503 | 2026-10-15 07:00:00+07 | 2026-10-15 00:00:00 | 2026-10-15 07:00:00
SELECT code,
       requested_at,                                   -- theo TimeZone của phiên
       requested_at AT TIME ZONE 'UTC'              AS gio_utc,     -- timestamp (không múi giờ)
       requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh' AS gio_vn
FROM trips WHERE code = 101503;

-- ---------------------------------------------------------------------------
-- 2. Đếm chuyến "ngày 15/10" bằng 6 cách, phiên đang là giờ VN.
--    Đúng: khoảng NỬA MỞ [00:00 ngày 15, 00:00 ngày 16) ghi rõ +07.
-- expect-line: 106 | 106 | 106 | 106 | 105 | 107
SELECT
  count(*) FILTER (WHERE requested_at >= '2026-10-15 00:00:00+07'
                     AND requested_at <  '2026-10-16 00:00:00+07')              AS a_nua_mo,
  count(*) FILTER (WHERE (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date
                         = DATE '2026-10-15')                                   AS b_doi_gio_vn_roi_lay_ngay,
  count(*) FILTER (WHERE date_trunc('day', requested_at, 'Asia/Ho_Chi_Minh')
                         = '2026-10-15 00:00:00+07')                            AS c_date_trunc_3_tham_so,
  count(*) FILTER (WHERE requested_at::date = DATE '2026-10-15')                AS d_ep_date_theo_phien,
  -- SAI 1: BETWEEN tới 23:59:59 bỏ sót 23:59:59.5 (timestamptz có micro giây)
  count(*) FILTER (WHERE requested_at BETWEEN '2026-10-15 00:00:00' AND '2026-10-15 23:59:59') AS e_between_SAI,
  -- SAI 2: "<= ngày 16" lấy luôn chuyến đúng 00:00:00 ngày 16
  count(*) FILTER (WHERE requested_at >= '2026-10-15' AND requested_at <= '2026-10-16')    AS f_nho_hon_bang_SAI
FROM trips;

-- Cách d và e/f còn phụ thuộc PHIÊN: chuỗi không ghi múi giờ và phép ép
-- ::date đều dùng TimeZone hiện tại. Cùng câu đếm, phiên UTC:
SET TIME ZONE 'UTC';

-- expect-line: 106 | 106 | 106 | 105 | 105 | 105
SELECT
  count(*) FILTER (WHERE requested_at >= '2026-10-15 00:00:00+07'
                     AND requested_at <  '2026-10-16 00:00:00+07')              AS a_nua_mo,
  count(*) FILTER (WHERE (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date
                         = DATE '2026-10-15')                                   AS b_doi_gio_vn_roi_lay_ngay,
  count(*) FILTER (WHERE date_trunc('day', requested_at, 'Asia/Ho_Chi_Minh')
                         = '2026-10-15 00:00:00+07')                            AS c_date_trunc_3_tham_so,
  count(*) FILTER (WHERE requested_at::date = DATE '2026-10-15')                AS d_ep_date_theo_phien,
  count(*) FILTER (WHERE requested_at BETWEEN '2026-10-15 00:00:00' AND '2026-10-15 23:59:59') AS e_between_SAI,
  count(*) FILTER (WHERE requested_at >= '2026-10-15' AND requested_at <= '2026-10-16')    AS f_nho_hon_bang_SAI
FROM trips;

SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- 3. KIỂM TRA HAI BIÊN: bộ lọc đúng phải giữ chuyến đúng 00:00:00 ngày 15 và
--    23:59:59.5, loại chuyến đúng 00:00:00 ngày 16. In từng chuyến biên với
--    kết quả của bộ lọc nửa mở (a) và hai bộ lọc sai (e, f).
-- expect-line: 101489 | 2026-10-15 00:00:00+07   | t | t | t
-- expect-line: 101502 | 2026-10-15 06:59:59+07   | t | t | t
-- expect-line: 101503 | 2026-10-15 07:00:00+07   | t | t | t
-- expect-line: 101594 | 2026-10-15 23:59:59.5+07 | t | f | t
-- expect-line: 101595 | 2026-10-16 00:00:00+07   | f | f | t
SELECT code, requested_at,
       requested_at >= '2026-10-15 00:00:00+07' AND requested_at < '2026-10-16 00:00:00+07'  AS a_nua_mo,
       requested_at BETWEEN '2026-10-15 00:00:00+07' AND '2026-10-15 23:59:59+07'            AS e_between,
       requested_at >= '2026-10-15 00:00:00+07' AND requested_at <= '2026-10-16 00:00:00+07' AS f_nho_hon_bang
FROM trips
WHERE code IN (101489, 101502, 101503, 101594, 101595)
ORDER BY code;

-- Dựng biên từ một tham số NGÀY (thứ ứng dụng truyền vào) mà không phụ thuộc
-- phiên: make_timestamptz(…, 'Asia/Ho_Chi_Minh') hoặc (ngày::timestamp) AT
-- TIME ZONE 'Asia/Ho_Chi_Minh' — cả hai ra cùng khoảnh khắc.
-- expect-line: 2026-10-15 00:00:00+07 | 2026-10-15 00:00:00+07 | 2026-10-16 00:00:00+07 | 106
SELECT make_timestamptz(2026, 10, 15, 0, 0, 0, 'Asia/Ho_Chi_Minh')                AS tu,
       (DATE '2026-10-15')::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh'            AS tu_cach_2,
       (DATE '2026-10-15' + 1)::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh'        AS den_truoc,
       (SELECT count(*) FROM trips
        WHERE requested_at >= (DATE '2026-10-15')::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh'
          AND requested_at <  (DATE '2026-10-15' + 1)::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh') AS so_chuyen;

-- ---------------------------------------------------------------------------
-- 4. Gom theo ngày: ngày UTC và ngày VN lệch nhau ở MỌI chuyến đặt 00:00–06:59
--    giờ VN (383 chuyến trong dữ liệu). Ba ngày giữa tháng:
-- expect-line: 2026-10-14 | 100 | 102
-- expect-line: 2026-10-15 | 106 | 105
-- expect-line: 2026-10-16 | 103 | 104
SELECT ngay,
       count(*) FILTER (WHERE theo = 'vn')  AS theo_ngay_vn,
       count(*) FILTER (WHERE theo = 'utc') AS theo_ngay_utc
FROM trips AS t
CROSS JOIN LATERAL (VALUES ('vn',  (t.requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date),
                           ('utc', (t.requested_at AT TIME ZONE 'UTC')::date)) AS v (theo, ngay)
WHERE ngay BETWEEN DATE '2026-10-14' AND DATE '2026-10-16'
GROUP BY ngay
ORDER BY ngay;

-- Biên THÁNG cũng lệch: 6 chuyến đang chạy đặt 00:30–00:58 ngày 31/10 giờ VN
-- là 30/10 theo UTC ⇒ báo cáo tháng 10 gom theo ngày UTC đếm thừa 6 chuyến.
-- expect-line: 3168 | 3174
SELECT count(*) FILTER (WHERE requested_at >= '2026-10-01 00:00+07' AND requested_at < '2026-11-01 00:00+07'
                          AND (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date <= DATE '2026-10-30') AS ky_vn_01_30,
       count(*) FILTER (WHERE (requested_at AT TIME ZONE 'UTC')::date BETWEEN DATE '2026-09-30' AND DATE '2026-10-30') AS ky_utc_tuong_ung
FROM trips;

-- ---------------------------------------------------------------------------
-- 5. Giờ trong ngày (giờ VN): extract trên giá trị đã đổi sang giờ VN. Lấy
--    extract(hour FROM requested_at) trực tiếp thì theo TimeZone phiên.
--    Giờ 0–6 cộng lại = 383, đúng số chuyến lệch ngày UTC ở mục 4.
-- expect-line: 0 |  68
-- expect-line: 1 |  28
-- expect-line: 2 |  39
-- expect-line: 3 |  39
-- expect-line: 4 |  25
-- expect-line: 5 |  61
-- expect-line: 6 | 123
-- expect-line: 7 | 193
SELECT extract(hour FROM requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::int AS gio_vn,
       count(*) AS so_chuyen
FROM trips
WHERE extract(hour FROM requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh') < 8
GROUP BY 1
ORDER BY 1;

-- ---------------------------------------------------------------------------
-- 6. Lịch ngày cho báo cáo: generate_series(date, date, interval) trả về
--    timestamptz (nửa đêm THEO PHIÊN), không phải date. So với cột date thì
--    PostgreSQL ép kiểu ngầm — đúng ở phiên VN, lệch ở phiên khác. Dùng
--    DATE + số nguyên, hoặc ép ::date ngay khi sinh.
-- expect-line: timestamp with time zone | date | date
SELECT pg_typeof(g)                       AS kieu_generate_series,
       pg_typeof(g::date)                 AS sau_khi_ep,
       pg_typeof(DATE '2026-10-01' + 1)   AS date_cong_so_nguyen
FROM generate_series(DATE '2026-10-01', DATE '2026-10-02', interval '1 day') AS g
LIMIT 1;

-- Lịch đủ ngày + LEFT JOIN ⇒ ngày không có chuyến vẫn hiện (DN, bão 20/10).
-- expect-line: 2026-10-19 | 13
-- expect-line: 2026-10-20 |  0
-- expect-line: 2026-10-21 | 17
SELECT l.ngay, count(t.id) AS chuyen_dn
FROM (SELECT DATE '2026-10-19' + n AS ngay FROM generate_series(0, 2) AS n) AS l
LEFT JOIN trips AS t
  ON t.city_id = 3
 AND t.requested_at >= l.ngay::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh'
 AND t.requested_at <  (l.ngay + 1)::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh'
GROUP BY l.ngay
ORDER BY l.ngay;

-- ---------------------------------------------------------------------------
-- 7. Khoảng thời gian (interval): hiệu hai timestamptz. Thời gian từ lúc đặt
--    tới lúc hoàn thành, theo dịch vụ; đổi ra phút bằng extract(epoch …).
-- expect-line: food   | 00:29:00 | 29.0
-- expect-line: parcel | 00:28:00 | 28.0
-- expect-line: ride   | 00:28:24 | 28.4
SELECT kind,
       date_trunc('second', avg(completed_at - requested_at))                    AS tb_thoi_gian,
       round(extract(epoch FROM avg(completed_at - requested_at)) / 60, 1)       AS tb_phut
FROM trips
WHERE status = 'completed'
GROUP BY kind
ORDER BY kind COLLATE "C";

-- ---------------------------------------------------------------------------
-- 8. Vì sao lab ghi múi giờ bằng TÊN, không bằng độ lệch: ở nơi có giờ mùa hè,
--    "+1 ngày" và "+24 giờ" khác nhau. New York lùi đồng hồ ngày 01/11/2026
--    (ngày đó dài 25 giờ). Việt Nam không có giờ mùa hè nên hai cách trùng nhau.
SET TIME ZONE 'America/New_York';

-- expect-line: 2026-11-01 00:00:00-04 | 2026-11-02 00:00:00-05 | 2026-11-01 23:00:00-05
SELECT TIMESTAMPTZ '2026-11-01 00:00:00 America/New_York'                       AS bat_dau,
       TIMESTAMPTZ '2026-11-01 00:00:00 America/New_York' + interval '1 day'    AS cong_1_ngay,
       TIMESTAMPTZ '2026-11-01 00:00:00 America/New_York' + interval '24 hours' AS cong_24_gio;

SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- expect-line: 2026-10-16 00:00:00+07 | 2026-10-16 00:00:00+07
SELECT TIMESTAMPTZ '2026-10-15 00:00:00+07' + interval '1 day'    AS cong_1_ngay,
       TIMESTAMPTZ '2026-10-15 00:00:00+07' + interval '24 hours' AS cong_24_gio;
