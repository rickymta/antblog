-- Thí nghiệm PG-029 «Sort, hash và truy vấn tràn ra đĩa» — lỗi thường gặp và lời
-- giải bài tập (không có trong lab chính 29-sort-hash-spill.sql).
--   A. EXPLAIN không ANALYZE KHÔNG cho biết có tràn đĩa hay không.
--   B. temp_file_limit chặn câu tràn quá trần (SQLSTATE 53400) — bài tập 2.
--   C. Tìm câu tràn đĩa bằng pg_stat_statements (temp_blks_written) — bài tập 1.
--   D. Song song: work_mem áp cho TỪNG worker — bài tập 3.
--   E. Index đúng thứ tự bỏ hẳn nút Sort (top-N và cả sắp toàn bộ hai cột trong index).
-- Cluster lab: work_mem 4MB, hash_mem_multiplier 2, song song TẮT (D bật lại trong phiên).

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- A. EXPLAIN (không ANALYZE) chỉ là KẾ HOẠCH: có nút Sort nhưng không có dòng
--    "Sort Method" — tràn hay không chỉ biết khi câu thật sự chạy.
-- expect-match: ^ Sort$
-- expect-no-match: Sort Method
-- expect-no-match: Disk
EXPLAIN (COSTS OFF)
SELECT code, fare FROM trips ORDER BY fare DESC, code;

-- ---------------------------------------------------------------------------
-- B. temp_file_limit = 10MB cho riêng giao dịch này. Câu sắp 1 triệu dòng cần một
--    file tạm ~21 MB (lab chính: Disk 21616kB) ⇒ bị huỷ giữa chừng.
BEGIN;
SET LOCAL temp_file_limit = '10MB';
-- expect-error: 53400
SELECT count(*) FROM (SELECT code, fare FROM trips ORDER BY fare DESC, code OFFSET 0) AS x;
ROLLBACK;

-- Cùng trần nhưng câu top-N (LIMIT 20) không cần file tạm ⇒ chạy bình thường.
BEGIN;
SET LOCAL temp_file_limit = '10MB';
-- expect-rows: 20
SELECT code, fare FROM trips ORDER BY fare DESC, code LIMIT 20;
ROLLBACK;

-- Bảng TẠM tường minh (CREATE TEMP TABLE) KHÔNG tính vào temp_file_limit.
BEGIN;
SET LOCAL temp_file_limit = '1MB';
-- expect-rows: 1000000
CREATE TEMP TABLE ban_sao_tam AS SELECT code, fare FROM trips;
ROLLBACK;

-- ---------------------------------------------------------------------------
-- C. Câu nào đang tràn đĩa? pg_stat_statements (đã nạp sẵn qua
--    shared_preload_libraries của cluster lab) cộng dồn temp_blks_written theo câu.
CREATE EXTENSION pg_stat_statements;
-- expect-value: t
SELECT pg_stat_statements_reset() IS NOT NULL;

-- Ba câu báo cáo: (1) đếm theo khách — HashAggregate 191.852 nhóm, tràn;
-- (2) top-20 cước — top-N heapsort, không tràn; (3) đếm chuyến cước cao — không sort/hash.
-- expect-value: 191852
SELECT count(*) FROM (SELECT customer_id, count(*) FROM trips GROUP BY customer_id) AS x;
-- expect-rows: 20
SELECT code, fare FROM trips ORDER BY fare DESC, code LIMIT 20;
-- expect-value: 3541
SELECT count(*) FROM trips WHERE fare >= 150000;

-- Xếp theo khối tạm đã ghi (1 khối = 8 kB). Chỉ câu (1) có temp_blks_written > 0.
-- expect-match: GROUP BY +\| +1 \| t$
-- expect-match: LIMIT \$1 +\| +1 \| f$
-- expect-match: fare >= \$1 +\| +1 \| f$
SELECT left(regexp_replace(query, '\s+', ' ', 'g'), 70) AS cau, calls,
       temp_blks_written > 0 AS co_tran
FROM pg_stat_statements
WHERE dbid = (SELECT oid FROM pg_database WHERE datname = current_database())
  AND query ~ 'FROM trips' AND query !~ 'pg_stat_statements'
ORDER BY temp_blks_written DESC, query;

-- Số khối tạm của câu (1): vài nghìn khối (lab chính: temp written=3138).
-- expect-range: 1000 10000
SELECT temp_blks_written FROM pg_stat_statements
WHERE dbid = (SELECT oid FROM pg_database WHERE datname = current_database())
  AND query ~ 'GROUP BY customer_id';

-- ---------------------------------------------------------------------------
-- D. Bật song song trong phiên: mỗi worker có hạn mức work_mem RIÊNG và sắp phần
--    dữ liệu của nó; Gather Merge trộn lại. Dùng SELECT * (dòng rộng): với câu chỉ
--    lấy (code, fare), planner thấy chuyển 1 triệu dòng qua Gather đắt hơn lợi ích nên
--    không chọn song song. Số kB mỗi tiến trình phụ thuộc cách chia page ⇒ biến động.
SET max_parallel_workers_per_gather = 2;
-- lab: bien-dong
-- expect-match: Gather Merge
-- expect-match: Workers Launched: 2
-- expect-match: ^         Sort Method: external merge  Disk: \d+kB
-- expect-match: Worker 0:  Sort Method: external merge  Disk: \d+kB
-- expect-match: Worker 1:  Sort Method: external merge  Disk: \d+kB
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT * FROM trips ORDER BY fare DESC, code;
RESET max_parallel_workers_per_gather;

-- Cùng câu, không song song: MỘT tiến trình sắp cả 1 triệu dòng rộng.
-- expect-match: ^ Sort \(actual rows=1000000\.00 loops=1\)
-- expect-match: Sort Method: external merge  Disk: \d+kB
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT * FROM trips ORDER BY fare DESC, code;

-- ---------------------------------------------------------------------------
-- E. Index theo đúng thứ tự sắp (fare DESC, code): câu top-N đọc thẳng 20 mục đầu
--    của index — không nút Sort, 4 page thay vì 19.441.
CREATE INDEX ix_trips_fare_code ON trips (fare DESC, code);
-- expect-match: Index Only Scan using ix_trips_fare_code on trips
-- expect-match: Buffers: shared read=4$
-- expect-no-match: Sort
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, fare FROM trips ORDER BY fare DESC, code LIMIT 20;

-- Sắp TOÀN BỘ (code, fare): index chứa đủ hai cột, bảng vừa VACUUM (all-visible) ⇒
-- Index Only Scan đọc index theo thứ tự — không Sort, không file tạm.
-- expect-match: Index Only Scan using ix_trips_fare_code on trips
-- expect-match: Heap Fetches: 0$
-- expect-no-match: Sort
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT code, fare FROM trips ORDER BY fare DESC, code;

-- SELECT * thì index không đủ cột. Planner vẫn chọn đi index để KHỎI sắp: Index Scan
-- đọc heap theo thứ tự fare (ngẫu nhiên với vị trí vật lý) — không Sort, không file tạm,
-- nhưng số lượt chạm page lên tới cỡ một triệu thay vì 19.441. Bỏ được tràn đĩa
-- KHÔNG có nghĩa là nhanh hơn (lab không đo phép này).
-- expect-match: Index Scan using ix_trips_fare_code on trips
-- expect-no-match: Sort
-- expect-match: Buffers: shared hit=\d+ read=\d+
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips ORDER BY fare DESC, code;

-- Số page chạm của kế hoạch đó (hit + read ở nút gốc).
CREATE FUNCTION so_page_cham(sql text) RETURNS bigint LANGUAGE plpgsql AS $$
DECLARE j jsonb;
BEGIN
  EXECUTE 'EXPLAIN (ANALYZE, BUFFERS, TIMING OFF, FORMAT JSON) ' || sql INTO j;
  RETURN (j -> 0 -> 'Plan' ->> 'Shared Hit Blocks')::bigint + (j -> 0 -> 'Plan' ->> 'Shared Read Blocks')::bigint;
END $$;
-- expect-range: 500000 1100000
SELECT so_page_cham('SELECT * FROM trips ORDER BY fare DESC, code') AS page_cham;
