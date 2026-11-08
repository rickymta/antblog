SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- 1. Float không biểu diễn chính xác 0,1 (đơn vị nghìn đồng: 0,1 = 100 đ).
SELECT 0.1::double precision + 0.2::double precision AS float_cong,
       0.1::double precision + 0.2::double precision = 0.3 AS float_bang_0_3,
       0.1::numeric + 0.2::numeric = 0.3 AS numeric_bang_0_3;

-- 2. Cộng mười khoản 0,1.
SELECT sum(0.1::double precision)     AS tong_float,
       sum(0.1::numeric)              AS tong_numeric,
       sum(100)                       AS tong_dong_integer
FROM generate_series(1, 10);

-- 3. Chia tiền bằng integer: phần dư bị cắt.
SELECT 100000 / 3 AS chia_integer,
       100000 % 3 AS phan_du,
       100000::numeric / 3 AS chia_numeric;

-- 4. sum(integer) trả bigint.
SELECT pg_typeof(sum(fare)) AS kieu_tong, sum(fare) AS tong_cuoc
FROM trips;

-- 5. money phụ thuộc lc_monetary.
SELECT current_setting('lc_monetary') AS lc_monetary, 45000::money AS cuoc_money;

-- 6. timestamptz vs timestamp với cùng một chuỗi.
SELECT '2026-11-03 00:10:00+07'::timestamptz AS co_mui_gio,
       '2026-11-03 00:10:00+07'::timestamp   AS khong_mui_gio;

SET TIME ZONE 'UTC';
SELECT '2026-11-03 00:10:00+07'::timestamptz AS co_mui_gio,
       '2026-11-03 00:10:00+07'::timestamp   AS khong_mui_gio;

-- Chuỗi không ghi múi giờ được hiểu theo TimeZone của phiên.
SELECT '2026-11-03 00:10:00'::timestamptz AS hieu_theo_utc;

-- Đổi về giờ VN và lấy ngày VN, dù phiên đang UTC.
SELECT code,
       requested_at,
       requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh'         AS gio_vn,
       (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS ngay_vn,
       requested_at::date                                   AS ngay_theo_phien
FROM trips
WHERE code IN (100007, 100008)
ORDER BY code;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- 7. uuid
SELECT pg_typeof(gen_random_uuid()) AS kieu,
       pg_column_size(gen_random_uuid()) AS byte_uuid,
       length(gen_random_uuid()::text) AS ky_tu_dang_chu,
       pg_column_size(100004::bigint) AS byte_bigint;

-- 8. Sequence sau 03: số đã bị tiêu.
SELECT last_value FROM trips_code_seq;

-- 9. Data dictionary trong catalog.
SELECT a.attname AS cot,
       format_type(a.atttypid, a.atttypmod) AS kieu,
       NOT a.attnotnull AS cho_null,
       col_description(a.attrelid, a.attnum) AS chu_thich
FROM pg_attribute a
WHERE a.attrelid = 'toinoi.trips'::regclass AND a.attnum > 0 AND NOT a.attisdropped
ORDER BY a.attnum;
