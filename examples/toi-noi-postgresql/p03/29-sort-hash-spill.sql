-- P03 · 29-sort-hash-spill.sql — PG-029 «Sort, hash và truy vấn tràn ra đĩa»
-- (`postgresql-sort-hash-memory-spill`).
--
-- Làm được sau bài: nhận ra truy vấn TRÀN ĐĨA trong kế hoạch (Sort Method: external
-- merge Disk, Hash Batches > 1, HashAggregate Disk Usage), đếm file tạm bằng
-- pg_stat_database / log_temp_files, hiểu work_mem × hash_mem_multiplier là giới hạn
-- MỖI NÚT MỖI PHIÊN (không phải của cả server) và đánh đổi khi nhiều phiên đồng thời.
-- Thời gian thật (và tải đồng thời bằng pgbench): p03/DO-DAC.md (PG-029).
--
-- Cluster lab: work_mem = 4MB, hash_mem_multiplier = 2 (đều là mặc định của PG 18).

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- 1. SORT. Sắp 1 triệu chuyến theo cước (không có index trên fare).
-- 1a. work_mem = 4MB (mặc định): dữ liệu cần sắp lớn hơn 4MB ⇒ sắp từng phần, ghi
--     ra file tạm rồi trộn ("external merge", Disk = dung lượng file tạm).
-- expect-match: Sort Method: external merge  Disk: \d+kB
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, fare FROM trips ORDER BY fare DESC, code;

-- 1b. work_mem = 64kB (mức tối thiểu): vẫn external merge, nhiều lượt trộn hơn.
SET work_mem = '64kB';
-- expect-match: Sort Method: external merge  Disk: \d+kB
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, fare FROM trips ORDER BY fare DESC, code;

-- 1c. work_mem = 256MB: cả 1 triệu dòng vừa bộ nhớ ⇒ quicksort, không file tạm.
SET work_mem = '256MB';
-- expect-match: Sort Method: quicksort  Memory: \d+kB
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, fare FROM trips ORDER BY fare DESC, code;

-- 1d. Chỉ cần 20 dòng đầu (LIMIT): "top-N heapsort" giữ đúng 20 dòng trong bộ nhớ —
--     không tràn đĩa kể cả với work_mem tối thiểu.
SET work_mem = '64kB';
-- expect-match: Sort Method: top-N heapsort  Memory: \d+kB
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, fare FROM trips ORDER BY fare DESC, code LIMIT 20;
RESET work_mem;

-- ---------------------------------------------------------------------------
-- 2. File tạm: đếm bằng thống kê database và ghi nhật ký từng file.
--    Thống kê tích luỹ được đẩy định kỳ ⇒ ép đẩy + xoá ảnh chụp trước khi đọc.
SELECT pg_stat_force_next_flush();
SELECT pg_stat_clear_snapshot();
SELECT temp_files AS tf0, temp_bytes AS tb0 FROM pg_stat_database WHERE datname = current_database() \gset

-- log_temp_files = 0: ghi MỌI file tạm (kèm kích thước) vào nhật ký server khi file bị
-- xoá. Runner in các dòng nhật ký mới sau câu lệnh (chỉ thị lab: nhat-ky).
SET log_temp_files = 0;
-- lab: nhat-ky
SELECT count(*) FROM (SELECT code, fare FROM trips ORDER BY fare DESC, code OFFSET 0) AS x;
RESET log_temp_files;

SELECT pg_stat_force_next_flush();
SELECT pg_stat_clear_snapshot();
-- Một file tạm, vài chục MB.
-- expect-line: 1 | t
SELECT temp_files - :tf0 AS file_tam_moi, temp_bytes - :tb0 > 10000000 AS hon_10mb
FROM pg_stat_database WHERE datname = current_database();

-- ---------------------------------------------------------------------------
-- 3. HASH JOIN. Doanh thu theo loại khách: bảng băm dựng từ customers (200.003 dòng).
--    Giới hạn bộ nhớ của bảng băm = work_mem × hash_mem_multiplier = 4MB × 2 = 8MB.
--    Không vừa ⇒ chia "Batches": chỉ giữ một phần trong bộ nhớ, phần còn lại (của CẢ
--    hai phía join) ghi ra file tạm và xử lý lần lượt.
-- expect-match: Hash Join
-- expect-match: Buckets: \d+  Batches: [2-9]\d*  Memory Usage: \d+kB
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT c.kind, count(*), sum(t.fare)
FROM trips AS t JOIN customers AS c ON c.id = t.customer_id
GROUP BY c.kind;

-- hash_mem_multiplier = 8 ⇒ bảng băm được 32MB ⇒ một batch, không file tạm (sort vẫn
-- dùng work_mem 4MB — hệ số chỉ áp cho các nút băm).
SET hash_mem_multiplier = 8;
-- expect-match: Buckets: \d+  Batches: 1  Memory Usage: \d+kB
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT c.kind, count(*), sum(t.fare)
FROM trips AS t JOIN customers AS c ON c.id = t.customer_id
GROUP BY c.kind;
RESET hash_mem_multiplier;

-- ---------------------------------------------------------------------------
-- 4. HASH AGGREGATE tràn đĩa (từ PG 13): đếm chuyến theo từng khách — 191.852 nhóm.
--    Bảng băm vượt 8MB ⇒ ghi các nhóm còn lại ra đĩa ("Batches", "Disk Usage").
-- expect-match: HashAggregate
-- expect-match: Batches: [2-9]\d*  Memory Usage: \d+kB  Disk Usage: \d+kB
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT customer_id, count(*) FROM trips GROUP BY customer_id;

-- Cùng câu với work_mem 64MB (chỉ cho câu này: SET LOCAL trong giao dịch) ⇒ một batch.
BEGIN;
SET LOCAL work_mem = '64MB';
-- expect-match: Batches: 1  Memory Usage: \d+kB$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT customer_id, count(*) FROM trips GROUP BY customer_id;
COMMIT;

-- ---------------------------------------------------------------------------
-- 5. Đánh đổi: work_mem là hạn mức cho MỖI nút sort/hash, trong MỖI phiên (và mỗi
--    worker song song). Một câu có 3 nút như dưới (Hash, HashAggregate, Sort) có thể
--    dùng tới 8MB + 8MB + 4MB; 40 phiên chạy cùng lúc (max_connections của lab) ⇒ tới
--    ~800MB — trong khi shared_buffers chỉ 128MB. Đặt work_mem = 256MB "cho nhanh"
--    trên cả server là mời hệ điều hành giết tiến trình vì hết RAM.
-- expect-match: Sort
-- expect-match: HashAggregate
-- expect-match: Hash Join
EXPLAIN (COSTS OFF)
SELECT c.kind, t.city_id, count(*) AS so_chuyen
FROM trips AS t JOIN customers AS c ON c.id = t.customer_id
GROUP BY c.kind, t.city_id
ORDER BY so_chuyen DESC;

-- Ước tính TRẦN bộ nhớ sort/hash nếu mọi phiên cùng chạy câu trên (MB). Đây là phép
-- nhân minh hoạ, không phải đo; đo thật bằng pgbench ở p03/DO-DAC.md.
-- expect-line: 40 | 4 | 2 | 800
SELECT current_setting('max_connections')::int AS so_phien,
       pg_size_bytes(current_setting('work_mem')) / 1048576 AS work_mem_mb,
       current_setting('hash_mem_multiplier')::numeric::int AS hash_mem_multiplier,
       (current_setting('max_connections')::int
         * (2 * pg_size_bytes(current_setting('work_mem')) * current_setting('hash_mem_multiplier')::numeric
            + pg_size_bytes(current_setting('work_mem'))) / 1048576)::int AS tran_mb;

-- Thực hành an toàn: tăng work_mem CHO RIÊNG câu báo cáo cần, trong giao dịch của nó
-- (SET LOCAL), giữ mặc định nhỏ cho phần còn lại; đặt temp_file_limit để một câu tồi
-- không ghi đầy đĩa.
BEGIN;
SET LOCAL work_mem = '128MB';
SET LOCAL temp_file_limit = '1GB';
-- expect-value: 128MB
SHOW work_mem;
COMMIT;
-- expect-value: 4MB
SHOW work_mem;
