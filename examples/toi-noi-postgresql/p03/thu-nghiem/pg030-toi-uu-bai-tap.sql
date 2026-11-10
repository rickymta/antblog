-- Thí nghiệm PG-030 «Thực hành: tối ưu truy vấn bằng bằng chứng» — lời giải bài tập
-- (không có trong lab chính 30-toi-uu-truy-van.sql).
--   A. Trang 2 bằng keyset trên index mới: cùng kết quả với OFFSET, ít page hơn — bài tập 1.
--   B. HOT từng đợt: trước/sau index mới; đổi CHỈ cột trong WHERE của partial index
--      cũng mất HOT — bài tập 2.
--   C. plan_cache_mode = auto: đếm kế hoạch riêng/chung trong pg_prepared_statements;
--      kế hoạch đổi, KẾT QUẢ không đổi (md5) — bài tập 3.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

SELECT id AS kh_b1 FROM customers WHERE phone = '0919990001' \gset

CREATE INDEX ix_trips_khach_hoan_thanh ON trips (customer_id, completed_at DESC, code DESC)
WHERE status = 'completed';

-- Đếm page chạm ở nút gốc (như lab chính).
CREATE FUNCTION so_page_cham(sql text) RETURNS bigint LANGUAGE plpgsql AS $$
DECLARE j jsonb;
BEGIN
  EXECUTE 'EXPLAIN (ANALYZE, BUFFERS, TIMING OFF, FORMAT JSON) ' || sql INTO j;
  RETURN (j -> 0 -> 'Plan' ->> 'Shared Hit Blocks')::bigint + (j -> 0 -> 'Plan' ->> 'Shared Read Blocks')::bigint;
END $$;

-- ---------------------------------------------------------------------------
-- A. Trang 2 (chuyến 21–40) của B1.
-- Mốc keyset = dòng cuối trang 1.
SELECT completed_at AS moc_ht, code AS moc_ma
FROM trips WHERE customer_id = :'kh_b1' AND status = 'completed'
ORDER BY completed_at DESC, code DESC OFFSET 19 LIMIT 1 \gset
-- expect-line: 2026-10-30 21:33:18+07 | 1099214
SELECT :'moc_ht'::timestamptz AS moc_ht, :'moc_ma'::int AS moc_ma;

-- Tham chiếu: OFFSET 20 (md5 của 20 dòng, giữ vào biến).
SELECT md5(string_agg(code || '|' || completed_at, ',' ORDER BY completed_at DESC, code DESC)) AS md5_offset
FROM (SELECT code, completed_at FROM trips
      WHERE customer_id = :'kh_b1' AND status = 'completed'
      ORDER BY completed_at DESC, code DESC OFFSET 20 LIMIT 20) AS x \gset

-- Keyset: (completed_at, code) < mốc — hai cột cùng DESC nên so sánh hàng dùng được.
-- expect-value: t
SELECT md5(string_agg(code || '|' || completed_at, ',' ORDER BY completed_at DESC, code DESC)) = :'md5_offset' AS giong_offset
FROM (SELECT code, completed_at FROM trips
      WHERE customer_id = :'kh_b1' AND status = 'completed'
        AND (completed_at, code) < (:'moc_ht'::timestamptz, :'moc_ma'::int)
      ORDER BY completed_at DESC, code DESC LIMIT 20) AS x;

-- Kế hoạch keyset: điều kiện hàng nằm trong Index Cond, không Sort. Hai cột cần đọc
-- (code, completed_at) đều nằm trong index ⇒ Index Only Scan.
-- expect-match: Index Only Scan using ix_trips_khach_hoan_thanh on trips
-- expect-match: Index Cond: .*ROW\(completed_at, code\) <
-- expect-no-match: Sort
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT code, completed_at FROM trips
WHERE customer_id = :'kh_b1' AND status = 'completed'
  AND (completed_at, code) < (:'moc_ht'::timestamptz, :'moc_ma'::int)
ORDER BY completed_at DESC, code DESC LIMIT 20;

-- Page chạm ở trang 200 (OFFSET 3980): OFFSET phải đi qua 4.000 mục index rồi bỏ 3.980,
-- keyset nhảy thẳng tới mốc — chênh lệch lớn dần theo số trang.
SELECT so_page_cham(format(
  'SELECT code FROM trips WHERE customer_id = %L AND status = ''completed'' ORDER BY completed_at DESC, code DESC OFFSET 3980 LIMIT 20',
  :'kh_b1')) AS page_offset_trang_200 \gset
SELECT completed_at AS moc200_ht, code AS moc200_ma
FROM trips WHERE customer_id = :'kh_b1' AND status = 'completed'
ORDER BY completed_at DESC, code DESC OFFSET 3979 LIMIT 1 \gset
SELECT so_page_cham(format(
  'SELECT code FROM trips WHERE customer_id = %L AND status = ''completed'' AND (completed_at, code) < (%L::timestamptz, %s) ORDER BY completed_at DESC, code DESC LIMIT 20',
  :'kh_b1', :'moc200_ht', :'moc200_ma')) AS page_keyset_trang_200 \gset
-- expect-value: t
SELECT :page_offset_trang_200 > :page_keyset_trang_200 AS offset_ton_hon;
-- expect-line: 29 | 5
SELECT :page_offset_trang_200 AS page_offset, :page_keyset_trang_200 AS page_keyset;

-- ---------------------------------------------------------------------------
-- B. HOT và partial index, tách từng đợt (lab chính 7c chỉ kiểm tổng 400 | 200).
--    Bản sao 10.000 chuyến (CREATE TABLE AS: KHÔNG chép CHECK), fillfactor 50 để page
--    chắc chắn còn chỗ — loại điều kiện "hết chỗ", chỉ còn điều kiện "cột có index".
CREATE TABLE hot_thu WITH (fillfactor = 50) AS
SELECT * FROM trips WHERE code BETWEEN 600000 AND 609999;
CREATE INDEX ON hot_thu (customer_id, requested_at DESC);
CREATE TEMP TABLE lo AS
SELECT code, (row_number() OVER (ORDER BY code) - 1) / 200 AS dot
FROM hot_thu WHERE status = 'cancelled';
-- expect-value: t
SELECT count(*) >= 800 FROM lo;

-- Đợt 1 — CHƯA có index mới: "đánh dấu hoàn thành" (status + completed_at) ⇒ HOT.
-- expect-rows: 200
UPDATE hot_thu SET status = 'completed', completed_at = updated_at
WHERE code IN (SELECT code FROM lo WHERE dot = 0);
SELECT pg_stat_force_next_flush();
-- expect-line: 200 | 200
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_user_tables WHERE relid = 'hot_thu'::regclass;

CREATE INDEX ON hot_thu (customer_id, completed_at DESC, code DESC) WHERE status = 'completed';

-- Đợt 2 — có index mới, cùng thao tác ⇒ không HOT.
-- expect-rows: 200
UPDATE hot_thu SET status = 'completed', completed_at = updated_at
WHERE code IN (SELECT code FROM lo WHERE dot = 1);
SELECT pg_stat_force_next_flush();
-- expect-line: 400 | 200
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_user_tables WHERE relid = 'hot_thu'::regclass;

-- Đợt 3 — CHỈ đổi status (cột chỉ có trong mệnh đề WHERE của index partial, không
-- phải cột khoá; completed_at giữ nguyên) ⇒ vẫn không HOT.
-- expect-rows: 200
UPDATE hot_thu SET status = 'completed'
WHERE code IN (SELECT code FROM lo WHERE dot = 2);
SELECT pg_stat_force_next_flush();
-- expect-line: 600 | 200
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_user_tables WHERE relid = 'hot_thu'::regclass;

-- Đợt 4 — đối chứng: đổi fare (không nằm trong index nào) ⇒ HOT dù đã có index mới.
-- expect-rows: 200
UPDATE hot_thu SET fare = fare + 1000
WHERE code IN (SELECT code FROM lo WHERE dot = 3);
SELECT pg_stat_force_next_flush();
-- expect-line: 800 | 400
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_user_tables WHERE relid = 'hot_thu'::regclass;

-- ---------------------------------------------------------------------------
-- C. plan_cache_mode = auto (mặc định): 5 lần đầu lập kế hoạch RIÊNG, sau đó so chi
--    phí với kế hoạch CHUNG. Đếm bằng pg_prepared_statements.
SET plan_cache_mode = auto;
PREPARE lich_su_moi(uuid) AS
SELECT t.code, t.completed_at, t.kind, t.fare, tien.da_thu
FROM (
  SELECT id, code, completed_at, kind, fare
  FROM trips
  WHERE customer_id = $1 AND status = 'completed'
  ORDER BY completed_at DESC, code DESC
  LIMIT 20
) AS t
CROSS JOIN LATERAL (
  SELECT coalesce(sum(p.amount) FILTER (WHERE p.kind = 'charge' AND p.status = 'succeeded'), 0)
       - coalesce(sum(p.amount) FILTER (WHERE p.kind = 'refund' AND p.status = 'succeeded'), 0) AS da_thu
  FROM payments AS p WHERE p.trip_id = t.id
) AS tien
ORDER BY t.completed_at DESC, t.code DESC;

-- Bảy lần EXECUTE cùng tham số B1, lần nào cũng cùng md5 với lab chính.
-- expect-md5: c8046204e6fa07cc57d4196a9cf7cc0e
EXECUTE lich_su_moi(:'kh_b1');
-- expect-md5: c8046204e6fa07cc57d4196a9cf7cc0e
EXECUTE lich_su_moi(:'kh_b1');
-- expect-md5: c8046204e6fa07cc57d4196a9cf7cc0e
EXECUTE lich_su_moi(:'kh_b1');
-- expect-md5: c8046204e6fa07cc57d4196a9cf7cc0e
EXECUTE lich_su_moi(:'kh_b1');
-- expect-md5: c8046204e6fa07cc57d4196a9cf7cc0e
EXECUTE lich_su_moi(:'kh_b1');
-- expect-md5: c8046204e6fa07cc57d4196a9cf7cc0e
EXECUTE lich_su_moi(:'kh_b1');
-- expect-md5: c8046204e6fa07cc57d4196a9cf7cc0e
EXECUTE lich_su_moi(:'kh_b1');

-- 7 lần = custom_plans + generic_plans. Chia thế nào tuỳ chi phí ước lượng của kế
-- hoạch chung (phụ thuộc ANALYZE lấy mẫu) ⇒ biến động; tổng thì chắc chắn.
-- lab: bien-dong
SELECT name, custom_plans, generic_plans FROM pg_prepared_statements;
-- expect-value: 7
SELECT custom_plans + generic_plans FROM pg_prepared_statements WHERE name = 'lich_su_moi';
-- expect-range: 5 7
SELECT custom_plans FROM pg_prepared_statements WHERE name = 'lich_su_moi';

-- Ép kế hoạch chung: kế hoạch mang $1 (có thể là Index Scan hoặc Bitmap + Sort tuỳ
-- ANALYZE — xem lab chính 6b), kết quả vẫn y hệt.
SET plan_cache_mode = force_generic_plan;
-- lab: bien-dong
-- expect-match: customer_id = \$1
EXPLAIN (COSTS OFF) EXECUTE lich_su_moi(:'kh_b1');
-- expect-md5: c8046204e6fa07cc57d4196a9cf7cc0e
EXECUTE lich_su_moi(:'kh_b1');
RESET plan_cache_mode;
DEALLOCATE ALL;
