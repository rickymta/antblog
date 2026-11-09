-- PG-016 · thí nghiệm ngày giờ/múi giờ ngoài luồng chính 16-date-time-timezone.sql.
-- Chạy sau nền P02 (p01/00 + p01/01 + p02/00 → 02):
--   node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
--     p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02-extra/pg016-mui-gio.sql
-- Tệp đổi TimeZone giữa chừng và luôn đặt lại 'Asia/Ho_Chi_Minh' sau mỗi thí nghiệm.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- A. Ghi múi giờ bằng ĐỘ LỆCH dạng chuỗi là bẫy. Chuỗi 'UTC+7' hay '+07' trong
--    AT TIME ZONE được hiểu theo quy ước POSIX: dấu + là PHÍA TÂY Greenwich, tức
--    UTC−7. Chuyến 101503 (07:00 sáng 15/10 giờ VN) thành 17:00 chiều 14/10.
--    Viết bằng INTERVAL hoặc TÊN múi giờ thì đúng.
-- expect-line: 101503 | 2026-10-14 17:00:00 | 2026-10-14 17:00:00 | 2026-10-15 07:00:00 | 2026-10-15 07:00:00
SELECT code,
       requested_at AT TIME ZONE 'UTC+7'            AS chuoi_utc_cong_7,
       requested_at AT TIME ZONE '+07'              AS chuoi_cong_07,
       requested_at AT TIME ZONE INTERVAL '+07:00'  AS interval_cong_07,
       requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh' AS ten_mui_gio
FROM trips WHERE code = 101503;

-- Ở phiên: SET TIME ZONE 'UTC+7' cũng là UTC−7 (hiển thị -07)…
SET TIME ZONE 'UTC+7';

-- expect-line: 101503 | 2026-10-14 17:00:00-07
SELECT code, requested_at FROM trips WHERE code = 101503;

-- …nhưng SET TIME ZONE với một SỐ (kể cả viết trong nháy '+07') thì dấu + là
-- phía ĐÔNG — cùng ký tự, hai nghĩa ngược nhau giữa SET và AT TIME ZONE.
SET TIME ZONE '+07';

-- expect-line: 101503 | 2026-10-15 07:00:00+07
SELECT code, requested_at FROM trips WHERE code = 101503;

SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- B. Thêm ba cách lọc "ngày 15/10" phụ thuộc PHIÊN, ngoài d/e/f của tệp 16:
--    date_trunc 2 tham số, so với hằng DATE (ép thành nửa đêm THEO PHIÊN), ép
--    ::timestamp. Phiên VN đều ra 106; phiên UTC đều ra 105. date_trunc 3 tham
--    số (múi giờ ghi rõ) giữ 106 ở cả hai phiên.
-- expect-line: 106 | 106 | 106 | 106
SELECT count(*) FILTER (WHERE date_trunc('day', requested_at) = '2026-10-15')                    AS trunc_2_tham_so,
       count(*) FILTER (WHERE date_trunc('day', requested_at, 'Asia/Ho_Chi_Minh') = '2026-10-15 00:00+07') AS trunc_3_tham_so,
       count(*) FILTER (WHERE requested_at >= DATE '2026-10-15' AND requested_at < DATE '2026-10-16') AS so_voi_date,
       count(*) FILTER (WHERE requested_at::timestamp >= '2026-10-15'
                          AND requested_at::timestamp <  '2026-10-16')                           AS ep_timestamp
FROM trips;

SET TIME ZONE 'UTC';

-- expect-line: 105 | 106 | 105 | 105
SELECT count(*) FILTER (WHERE date_trunc('day', requested_at) = '2026-10-15')                    AS trunc_2_tham_so,
       count(*) FILTER (WHERE date_trunc('day', requested_at, 'Asia/Ho_Chi_Minh') = '2026-10-15 00:00+07') AS trunc_3_tham_so,
       count(*) FILTER (WHERE requested_at >= DATE '2026-10-15' AND requested_at < DATE '2026-10-16') AS so_voi_date,
       count(*) FILTER (WHERE requested_at::timestamp >= '2026-10-15'
                          AND requested_at::timestamp <  '2026-10-16')                           AS ep_timestamp
FROM trips;

SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- C. Tuần giờ VN: date_trunc('week', …, 'Asia/Ho_Chi_Minh') về 00:00 THỨ HAI.
--    01/10/2026 là thứ năm ⇒ tuần đầu chỉ có 4 ngày (01–04/10); tuần cuối gồm
--    cả 6 chuyến đang chạy sáng 31/10.
-- expect-line: 2026-09-28 00:00:00+07 | 440
-- expect-line: 2026-10-05 00:00:00+07 | 751
-- expect-line: 2026-10-12 00:00:00+07 | 748
-- expect-line: 2026-10-19 00:00:00+07 | 728
-- expect-line: 2026-10-26 00:00:00+07 | 507
SELECT date_trunc('week', requested_at, 'Asia/Ho_Chi_Minh') AS dau_tuan, count(*) AS so_chuyen
FROM trips
GROUP BY 1
ORDER BY 1;

-- isodow: thứ hai = 1 … chủ nhật = 7; dow: chủ nhật = 0. Tuần ISO của 12/10.
-- expect-line: 4 | 0 | 7 | 2026-W42
SELECT extract(isodow FROM DATE '2026-10-01') AS thu_01_10,
       extract(dow    FROM DATE '2026-10-04') AS dow_04_10,
       extract(isodow FROM DATE '2026-10-04') AS isodow_04_10,
       to_char(DATE '2026-10-12', 'IYYY-"W"IW') AS tuan_iso;

-- ---------------------------------------------------------------------------
-- D. Biên tháng: 00:30 ngày 01/11 giờ VN vẫn là 31/10 theo UTC.
-- expect-line: 2026-11-01 00:00:00+07 | 2026-10-01 07:00:00+07
SELECT date_trunc('month', TIMESTAMPTZ '2026-11-01 00:30:00+07', 'Asia/Ho_Chi_Minh') AS thang_vn,
       date_trunc('month', TIMESTAMPTZ '2026-11-01 00:30:00+07', 'UTC')              AS thang_utc;

-- ---------------------------------------------------------------------------
-- E. Giờ cao điểm: cùng dữ liệu, giờ VN và giờ UTC lệch nhau 7 — "18 giờ"
--    của VN là "11 giờ" UTC.
-- expect-line: 18 | 289
-- expect-line: 12 | 280
-- expect-line: 11 | 255
SELECT extract(hour FROM requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::int AS gio_vn, count(*) AS so_chuyen
FROM trips GROUP BY 1 ORDER BY 2 DESC, 1 LIMIT 3;

-- expect-line: 11 | 289
-- expect-line:  5 | 280
-- expect-line:  4 | 255
SELECT extract(hour FROM requested_at AT TIME ZONE 'UTC')::int AS gio_utc, count(*) AS so_chuyen
FROM trips GROUP BY 1 ORDER BY 2 DESC, 1 LIMIT 3;

-- ---------------------------------------------------------------------------
-- F. Cộng THÁNG: ngày cuối tháng bị kẹp về ngày cuối của tháng đích, và cộng
--    hai lần một tháng khác cộng một lần hai tháng.
-- expect-line: 2026-02-28 00:00:00 | 2026-03-28 00:00:00 | 2026-03-31 00:00:00
SELECT DATE '2026-01-31' + interval '1 month'                          AS cong_1_thang,
       (DATE '2026-01-31' + interval '1 month') + interval '1 month'   AS cong_1_thang_2_lan,
       DATE '2026-01-31' + interval '2 months'                         AS cong_2_thang;

-- ---------------------------------------------------------------------------
-- G. interval: '1 day 06:33:00' (khoảng giữa hai chuyến ở PG-015) = 30,55 giờ.
--    So sánh interval coi 1 ngày = 24 giờ; cộng vào timestamptz thì không
--    (xem New York ở tệp 16).
-- expect-line: t | 30.55 | 1 day 06:33:00
SELECT interval '1 day 06:33:00' = interval '30 hours 33 minutes'          AS bang_nhau,
       round(extract(epoch FROM interval '1 day 06:33:00') / 3600, 2)       AS so_gio,
       justify_hours(interval '30 hours 33 minutes')                       AS chuan_hoa;
