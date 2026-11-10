-- P03 · 27-statistics.sql — PG-027 «Statistics, ANALYZE và ước lượng cardinality»
-- (`postgresql-statistics-analyze-cardinality`).
--
-- Làm được sau bài: tìm chỗ ước lượng lệch thực tế trên dữ liệu phân bố KHÔNG đều;
-- đọc pg_stats (null_frac, n_distinct, most_common_vals/freqs = MCV, histogram_bounds,
-- correlation); dùng CREATE STATISTICS (dependencies, ndistinct, mcv) cho hai cột
-- tương quan (thành phố ↔ khu đón); thấy thống kê CŨ làm hỏng ước lượng.
--
-- ANALYZE trên bảng lớn LẤY MẪU 300 × default_statistics_target = 30.000 dòng NGẪU
-- NHIÊN ⇒ mọi con số thống kê (n_distinct, tần suất MCV, ước lượng rows=…) DAO ĐỘNG
-- giữa các lần chạy. Vì vậy bài in chúng trong khối "biến động" và chỉ KIỂM bằng
-- khoảng (expect-range) hoặc bằng hệ số lệch qua hàm uoc_luong() của nền P03.
-- Số THỰC TẾ (đếm thật) thì tất định.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

SELECT id AS kh_b1 FROM customers WHERE phone = '0919990001' \gset
SELECT id AS kh_nhieu FROM customers WHERE phone = '0910000066' \gset
SELECT id AS kh_khong FROM customers AS c
WHERE NOT EXISTS (SELECT 1 FROM trips AS t WHERE t.customer_id = c.id) ORDER BY phone LIMIT 1 \gset

-- ---------------------------------------------------------------------------
-- 1. Thống kê của cột customer_id (do VACUUM ANALYZE của nền tạo).
-- lab: bien-dong
SELECT null_frac, n_distinct, cardinality(most_common_vals::text::text[]) AS so_mcv,
       round(most_common_freqs[1]::numeric, 4) AS tan_suat_mcv_1,
       round((SELECT sum(f) FROM unnest(most_common_freqs) AS f)::numeric, 3) AS tong_tan_suat_mcv,
       cardinality(histogram_bounds::text::text[]) AS so_moc_histogram
FROM pg_stats WHERE schemaname = 'toinoi' AND tablename = 'trips' AND attname = 'customer_id';

-- Số khách KHÁC NHAU thật sự (tất định): 191.852. n_distinct ở trên là ƯỚC LƯỢNG từ
-- 30.000 dòng mẫu — với dữ liệu lệch như này nó thường THẤP hơn thật nhiều lần.
-- expect-value: 191852
SELECT count(DISTINCT customer_id) FROM trips;

-- expect-range: 40000 150000
SELECT n_distinct FROM pg_stats WHERE schemaname = 'toinoi' AND tablename = 'trips' AND attname = 'customer_id';

-- Khách B1 (5,1 % số chuyến) chắc chắn nằm đầu danh sách MCV.
-- expect-value: 1
SELECT array_position(most_common_vals::text::uuid[], :'kh_b1'::uuid)
FROM pg_stats WHERE schemaname = 'toinoi' AND tablename = 'trips' AND attname = 'customer_id';

-- ---------------------------------------------------------------------------
-- 2. Ước lượng theo từng khách (EXPLAIN ANALYZE in "rows=ước (actual rows=thật)").
-- 2a. B1 — giá trị trong MCV: ước lượng = tần suất × số dòng ⇒ sát thực tế.
-- lab: bien-dong
EXPLAIN (ANALYZE, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT * FROM trips WHERE customer_id = :'kh_b1';

-- expect-range: 1 1.25
SELECT lech FROM uoc_luong(format('SELECT * FROM trips WHERE customer_id = %L', :'kh_b1'));

-- 2b. Khách 0910000066 — đi 76 chuyến, NGOÀI danh sách MCV: planner chia đều phần
--     không thuộc MCV cho (n_distinct − số MCV) giá trị ⇒ ước ~vài chục, lệch nhiều lần.
-- expect-value: 76
SELECT thuc_te FROM uoc_luong(format('SELECT * FROM trips WHERE customer_id = %L', :'kh_nhieu'));
-- expect-range: 3 15
SELECT lech FROM uoc_luong(format('SELECT * FROM trips WHERE customer_id = %L', :'kh_nhieu'));

-- 2c. Khách CHƯA đặt chuyến nào: thực tế 0, planner vẫn ước như khách "trung bình".
-- expect-value: 0
SELECT thuc_te FROM uoc_luong(format('SELECT * FROM trips WHERE customer_id = %L', :'kh_khong'));
-- expect-range: 4 20
SELECT uoc_tinh FROM uoc_luong(format('SELECT * FROM trips WHERE customer_id = %L', :'kh_khong'));

-- 2d. Tăng mức chi tiết thống kê RIÊNG cho cột này: mẫu 300.000 dòng, MCV tới 1.000 giá
--     trị ⇒ khách 76 chuyến lọt vào MCV, n_distinct sát hơn. Đổi lại: ANALYZE lâu hơn,
--     pg_statistic to hơn, planner tốn thêm chút thời gian.
ALTER TABLE trips ALTER COLUMN customer_id SET STATISTICS 1000;
ANALYZE trips (customer_id);

-- lab: bien-dong
SELECT n_distinct, cardinality(most_common_vals::text::text[]) AS so_mcv
FROM pg_stats WHERE schemaname = 'toinoi' AND tablename = 'trips' AND attname = 'customer_id';

-- expect-range: 1 1.5
SELECT lech FROM uoc_luong(format('SELECT * FROM trips WHERE customer_id = %L', :'kh_nhieu'));

-- ---------------------------------------------------------------------------
-- 3. Hai cột TƯƠNG QUAN: khu đón (pickup_area_id) quyết định luôn thành phố. Planner
--    mặc định coi các điều kiện ĐỘC LẬP ⇒ nhân xác suất ⇒ ước lượng THẤP ~7 lần.
-- expect-value: 22222
SELECT count(*) FROM trips WHERE city_id = 3 AND pickup_area_id = 311;

-- lab: bien-dong
EXPLAIN (ANALYZE, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT * FROM trips WHERE city_id = 3 AND pickup_area_id = 311;

-- expect-range: 5 10
SELECT lech FROM uoc_luong($$SELECT * FROM trips WHERE city_id = 3 AND pickup_area_id = 311$$);

-- GROUP BY hai cột: planner nhân số giá trị khác nhau (3 × 73 = 219 nhóm) trong khi
-- thực tế chỉ có 75 nhóm (72 khu + 3 nhóm "mất GPS" theo từng thành phố).
-- expect-value: 75
SELECT thuc_te FROM uoc_luong($$SELECT city_id, pickup_area_id, count(*) FROM trips GROUP BY city_id, pickup_area_id$$);
-- expect-range: 2.5 3.1
SELECT lech FROM uoc_luong($$SELECT city_id, pickup_area_id, count(*) FROM trips GROUP BY city_id, pickup_area_id$$);

-- Thống kê MỞ RỘNG trên cặp cột: dependencies (phụ thuộc hàm), ndistinct (số tổ hợp),
-- mcv (các tổ hợp phổ biến). Chỉ có hiệu lực sau ANALYZE.
CREATE STATISTICS st_trips_tp_khu (dependencies, ndistinct, mcv) ON city_id, pickup_area_id FROM trips;
ANALYZE trips;

-- pg_stats_ext: cột được gọi bằng SỐ THỨ TỰ cột (attnum): 4 = city_id, 14 = pickup_area_id.
-- n_distinct {"4, 14": 75} = 75 tổ hợp; dependencies {"14 => 4": ~0,99} = biết khu là
-- biết thành phố (không đúng 1,0 vì 1 % chuyến mất GPS có khu NULL ở cả 3 thành phố).
-- Chiều "thành phố ⇒ khu" yếu tới mức không được ghi.
-- lab: bien-dong
SELECT statistics_name, n_distinct, dependencies FROM pg_stats_ext WHERE statistics_name = 'st_trips_tp_khu';

-- expect-line: 4 | city_id
-- expect-line: 14 | pickup_area_id
SELECT attnum, attname FROM pg_attribute WHERE attrelid = 'trips'::regclass AND attname IN ('city_id', 'pickup_area_id') ORDER BY attnum;

-- expect-range: 0.98 1
SELECT round((dependencies::text::jsonb ->> '14 => 4')::numeric, 3)
FROM pg_stats_ext WHERE statistics_name = 'st_trips_tp_khu';

-- Ước lượng giờ sát thực tế cho cả hai câu.
-- lab: bien-dong
EXPLAIN (ANALYZE, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT * FROM trips WHERE city_id = 3 AND pickup_area_id = 311;

-- expect-range: 1 1.3
SELECT lech FROM uoc_luong($$SELECT * FROM trips WHERE city_id = 3 AND pickup_area_id = 311$$);
-- expect-range: 1 1.3
SELECT lech FROM uoc_luong($$SELECT city_id, pickup_area_id, count(*) FROM trips GROUP BY city_id, pickup_area_id$$);

-- ---------------------------------------------------------------------------
-- 4. Histogram cho cột liên tục: requested_at có 101 mốc chia 1 triệu chuyến thành 100
--    "xô" ~ bằng nhau ⇒ khoảng thời gian bất kỳ được ước lượng khá sát.
-- expect-value: 101
SELECT cardinality(histogram_bounds::text::text[]) FROM pg_stats
WHERE schemaname = 'toinoi' AND tablename = 'trips' AND attname = 'requested_at';

-- expect-range: 1 1.15
SELECT lech FROM uoc_luong($$SELECT * FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-08-01 00:00+07' AND requested_at < TIMESTAMPTZ '2026-08-08 00:00+07'$$);

-- ---------------------------------------------------------------------------
-- 5. Thống kê CŨ. Một ngày khuyến mãi (01/11) nạp thêm 30.000 chuyến SAU lần ANALYZE
--    cuối. Histogram vẫn kết thúc ở 31/10 ⇒ planner tin ngày 01/11 gần như không có gì.
--    (Cluster lab tắt autovacuum; server thật sẽ tự ANALYZE khi số dòng đổi vượt ngưỡng
--    autovacuum_analyze_threshold + autovacuum_analyze_scale_factor × số dòng = 50 + 10 %.)
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

-- Thực tế 30.000; ước lượng chỉ vài dòng ⇒ lệch hàng nghìn lần.
-- expect-value: 30000
SELECT thuc_te FROM uoc_luong($$SELECT * FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-11-01 00:00+07' AND requested_at < TIMESTAMPTZ '2026-11-02 00:00+07'$$);
-- expect-range: 1000 30000
SELECT lech FROM uoc_luong($$SELECT * FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-11-01 00:00+07' AND requested_at < TIMESTAMPTZ '2026-11-02 00:00+07'$$);

-- Hệ quả lên KẾ HOẠCH: join ngày khuyến mãi với tài xế — tin là "vài dòng" nên chọn
-- Nested Loop + tra index cho từng dòng; thực tế 30.000 vòng lặp.
-- expect-match: Nested Loop
EXPLAIN (COSTS OFF)
SELECT d.vehicle_type, count(*) FROM trips AS t JOIN drivers AS d ON d.id = t.driver_id
WHERE t.requested_at >= TIMESTAMPTZ '2026-11-01 00:00+07' AND t.requested_at < TIMESTAMPTZ '2026-11-02 00:00+07'
GROUP BY d.vehicle_type;

ANALYZE trips;

-- Sau ANALYZE: ước lượng sát (lệch dưới 2 lần), kế hoạch đổi sang Hash Join.
-- expect-range: 1 2
SELECT lech FROM uoc_luong($$SELECT * FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-11-01 00:00+07' AND requested_at < TIMESTAMPTZ '2026-11-02 00:00+07'$$);

-- expect-match: Hash Join
EXPLAIN (COSTS OFF)
SELECT d.vehicle_type, count(*) FROM trips AS t JOIN drivers AS d ON d.id = t.driver_id
WHERE t.requested_at >= TIMESTAMPTZ '2026-11-01 00:00+07' AND t.requested_at < TIMESTAMPTZ '2026-11-02 00:00+07'
GROUP BY d.vehicle_type;

-- Lần ANALYZE gần nhất (pg_stat_user_tables) — kiểm tra đầu tiên khi nghi thống kê cũ.
SELECT pg_stat_force_next_flush();
-- n_mod_since_analyze = số dòng thay đổi kể từ lần ANALYZE gần nhất (vừa ANALYZE ⇒ 0;
-- ngay trước ANALYZE ở trên, con số này là 30.000).
-- expect-line: t | 0
SELECT last_analyze IS NOT NULL AS da_analyze, n_mod_since_analyze
FROM pg_stat_user_tables WHERE relid = 'trips'::regclass;
