-- Thí nghiệm PG-027 «Statistics, ANALYZE và ước lượng cardinality» — không có trong
-- lab chính (p03/27-statistics.sql). Lời giải bài tập + ba phép thử phụ:
--   1. Tự tính ước lượng của một giá trị NGOÀI danh sách MCV bằng công thức trong tài
--      liệu PostgreSQL (row-estimation-examples) rồi so với rows= của planner.
--   2. Điều kiện trên BIỂU THỨC (giờ đặt chuyến) không có thống kê ⇒ planner dùng độ
--      chọn lọc mặc định; CREATE STATISTICS trên biểu thức (PostgreSQL 14+) sửa được.
--   3. Ghi đè n_distinct: sửa ước lượng cho khách "trung bình", KHÔNG sửa khách đi nhiều.
--   4. Thống kê cũ: planner tự co giãn TỔNG số dòng theo số page hiện tại, nhưng không
--      biết dòng mới nằm ở đâu; ngưỡng autovacuum ANALYZE mặc định với bảng 1 triệu dòng.
-- Số thống kê đến từ mẫu 30.000 dòng ⇒ chỉ kiểm bằng khoảng (expect-range).

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

SELECT id AS kh_nhieu FROM customers WHERE phone = '0910000066' \gset
SELECT id AS kh_khong FROM customers AS c
WHERE NOT EXISTS (SELECT 1 FROM trips AS t WHERE t.customer_id = c.id) ORDER BY phone LIMIT 1 \gset

-- ---------------------------------------------------------------------------
-- 1. Công thức cho giá trị ngoài MCV:
--    selectivity = (1 − null_frac − Σ most_common_freqs) / (n_distinct − số MCV)
--    rows        = selectivity × số dòng của bảng (reltuples, co giãn theo số page hiện tại)
-- lab: bien-dong
SELECT s.null_frac, s.n_distinct,
       cardinality(s.most_common_vals::text::text[]) AS so_mcv,
       round((SELECT sum(f) FROM unnest(s.most_common_freqs) AS f)::numeric, 4) AS tong_mcv,
       c.reltuples::bigint AS reltuples, c.relpages, pg_relation_size('trips') / 8192 AS page_hien_tai
FROM pg_stats AS s JOIN pg_class AS c ON c.oid = 'trips'::regclass
WHERE s.schemaname = 'toinoi' AND s.tablename = 'trips' AND s.attname = 'customer_id';

-- Tự tính (n_distinct âm = tỉ lệ so với số dòng ⇒ quy về số tuyệt đối trước).
SELECT round(((1 - s.null_frac - (SELECT sum(f) FROM unnest(s.most_common_freqs) AS f))
             / ((CASE WHEN s.n_distinct < 0 THEN -s.n_distinct * c.reltuples ELSE s.n_distinct END)
                - cardinality(s.most_common_vals::text::text[]))
             * c.reltuples)::numeric, 1) AS uoc_tu_tinh
FROM pg_stats AS s JOIN pg_class AS c ON c.oid = 'trips'::regclass
WHERE s.schemaname = 'toinoi' AND s.tablename = 'trips' AND s.attname = 'customer_id' \gset

-- lab: bien-dong
SELECT :uoc_tu_tinh AS uoc_tu_tinh,
       (SELECT uoc_tinh FROM uoc_luong(format('SELECT * FROM trips WHERE customer_id = %L', :'kh_khong'))) AS planner_khach_0_chuyen,
       (SELECT uoc_tinh FROM uoc_luong(format('SELECT * FROM trips WHERE customer_id = %L', :'kh_nhieu'))) AS planner_khach_76_chuyen;

-- Công thức khớp planner (sai số làm tròn): mọi khách ngoài MCV — 0 chuyến hay 76
-- chuyến — đều nhận CÙNG một ước lượng.
-- expect-range: 0.9 1.1
SELECT round(:uoc_tu_tinh / greatest(uoc_tinh, 1), 3)
FROM uoc_luong(format('SELECT * FROM trips WHERE customer_id = %L', :'kh_khong'));

-- expect-value: t
SELECT (SELECT uoc_tinh FROM uoc_luong(format('SELECT * FROM trips WHERE customer_id = %L', :'kh_khong')))
     = (SELECT uoc_tinh FROM uoc_luong(format('SELECT * FROM trips WHERE customer_id = %L', :'kh_nhieu')));

-- ---------------------------------------------------------------------------
-- 2. Điều kiện trên biểu thức: "chuyến đặt trong khung 12 giờ (giờ Việt Nam)".
-- Thực tế (tất định):
-- expect-value: 89965
SELECT count(*) FROM trips WHERE extract(hour FROM requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh') = 12;

-- Không có thống kê cho biểu thức ⇒ ước lượng = 0,5 % số dòng (độ chọn lọc mặc định
-- của phép "=" khi planner không biết gì), bất kể giờ nào.
-- expect-range: 4900 5100
SELECT uoc_tinh FROM uoc_luong($$SELECT * FROM trips WHERE extract(hour FROM requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh') = 12$$);
-- expect-range: 4900 5100
SELECT uoc_tinh FROM uoc_luong($$SELECT * FROM trips WHERE extract(hour FROM requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh') = 3$$);

-- Thống kê trên biểu thức, không cần index (không tốn chi phí ghi).
CREATE STATISTICS st_trips_gio_dat ON (extract(hour FROM requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')) FROM trips;
ANALYZE trips;

-- expect-range: 1 1.2
SELECT lech FROM uoc_luong($$SELECT * FROM trips WHERE extract(hour FROM requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh') = 12$$);

-- Thống kê biểu thức đọc ở pg_stats_ext_exprs (24 giá trị khác nhau = 24 giờ).
-- expect-value: 24
SELECT n_distinct FROM pg_stats_ext_exprs WHERE statistics_name = 'st_trips_gio_dat';

DROP STATISTICS st_trips_gio_dat;

-- ---------------------------------------------------------------------------
-- 3. Ghi đè n_distinct bằng số đếm thật (191.852 khách có chuyến). Chỉ có hiệu lực
--    từ lần ANALYZE sau.
ALTER TABLE trips ALTER COLUMN customer_id SET (n_distinct = 191852);
ANALYZE trips (customer_id);

-- expect-value: 191852
SELECT n_distinct FROM pg_stats WHERE schemaname = 'toinoi' AND tablename = 'trips' AND attname = 'customer_id';

-- Khách "trung bình" ngoài MCV giờ được ước ~5 chuyến (≈ 1 triệu × 0,92 / 191.800)…
-- expect-range: 3 7
SELECT uoc_tinh FROM uoc_luong(format('SELECT * FROM trips WHERE customer_id = %L', :'kh_khong'));

-- …nhưng khách 76 chuyến vẫn ngoài MCV ⇒ lệch còn TỆ hơn trước (~7 lần ⇒ ~16 lần).
-- expect-range: 10 25
SELECT lech FROM uoc_luong(format('SELECT * FROM trips WHERE customer_id = %L', :'kh_nhieu'));

ALTER TABLE trips ALTER COLUMN customer_id RESET (n_distinct);
ANALYZE trips (customer_id);

-- ---------------------------------------------------------------------------
-- 4. Thống kê cũ. Nạp 30.000 chuyến ngày 01/11 (như bài lab) mà không ANALYZE.
-- expect-rows: 30000
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at, pickup_area_id)
SELECT p03_uuid('km', t.code * 10 + g), t.kind, t.city_id, t.customer_id, t.driver_id, t.merchant_id, t.status, t.fare,
       t.requested_at + interval '8 days' + g * interval '7 seconds',
       t.accepted_at + interval '8 days' + g * interval '7 seconds',
       t.completed_at + interval '8 days' + g * interval '7 seconds',
       t.updated_at + interval '8 days' + g * interval '7 seconds',
       t.pickup_area_id
FROM trips AS t CROSS JOIN generate_series(1, 5) AS g
WHERE t.requested_at >= TIMESTAMPTZ '2026-10-24 00:00+07' AND t.requested_at < TIMESTAMPTZ '2026-10-25 00:00+07'
  AND t.status IN ('completed', 'cancelled')
ORDER BY t.code, g
LIMIT 30000;

-- 4a. Tổng số dòng: reltuples chưa đổi, nhưng planner nhân mật độ cũ với số page HIỆN
--     TẠI ⇒ vẫn ước được ~1.030.000 dòng (lệch ≈ 1,0).
-- lab: bien-dong
SELECT c.reltuples::bigint AS reltuples_cu, c.relpages AS relpages_cu, pg_relation_size('trips') / 8192 AS page_hien_tai
FROM pg_class AS c WHERE c.oid = 'trips'::regclass;

-- expect-value: 1030000
SELECT thuc_te FROM uoc_luong($$SELECT id FROM trips$$);
-- expect-range: 1 1.02
SELECT lech FROM uoc_luong($$SELECT id FROM trips$$);

-- 4b. Nhưng PHÂN BỐ thì cũ: histogram requested_at dừng ở 31/10 ⇒ ngày 01/11 ước ~1 dòng.
-- expect-range: 1000 30000
SELECT lech FROM uoc_luong($$SELECT * FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-11-01 00:00+07' AND requested_at < TIMESTAMPTZ '2026-11-02 00:00+07'$$);

-- 4c. Server thật có bật autovacuum có tự ANALYZE không? Ngưỡng = base + scale × reltuples
--     (mặc định 50 + 0,1 × số dòng; reltuples sau ANALYZE là số ƯỚC nên làm tròn nghìn).
--     30.000 dòng mới < ngưỡng ≈ 100.000 ⇒ KHÔNG.
SELECT pg_stat_force_next_flush();
-- expect-line: 50 | 0.1 | 100000 | 30000 | f
SELECT current_setting('autovacuum_analyze_threshold') AS base,
       current_setting('autovacuum_analyze_scale_factor') AS scale,
       round(current_setting('autovacuum_analyze_threshold')::int
             + current_setting('autovacuum_analyze_scale_factor')::numeric * c.reltuples::numeric, -3) AS nguong_lam_tron,
       s.n_mod_since_analyze,
       s.n_mod_since_analyze > current_setting('autovacuum_analyze_threshold')::int
             + current_setting('autovacuum_analyze_scale_factor')::numeric * c.reltuples AS se_tu_analyze
FROM pg_class AS c JOIN pg_stat_user_tables AS s ON s.relid = c.oid
WHERE c.oid = 'trips'::regclass;

-- Hạ ngưỡng RIÊNG cho bảng lớn (storage parameter của bảng): 50 + 0,01 × 1.000.000 ≈ 10.050.
ALTER TABLE trips SET (autovacuum_analyze_scale_factor = 0.01);
-- expect-value: {autovacuum_analyze_scale_factor=0.01}
SELECT reloptions FROM pg_class WHERE oid = 'trips'::regclass;
