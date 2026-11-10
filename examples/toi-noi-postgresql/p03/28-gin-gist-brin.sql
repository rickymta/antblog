-- P03 · 28-gin-gist-brin.sql — PG-028 «Khi nào cân nhắc GIN, GiST và BRIN?»
-- (`postgresql-gin-gist-brin-lua-chon`).
--
-- Làm được sau bài: lập ma trận WORKLOAD → OPERATOR → LOẠI INDEX → GIỚI HẠN:
--   A. GIN + pg_trgm  — tìm gần đúng tên/địa chỉ quán (LIKE '%…%', similarity %)
--   B. GIN cho jsonb  — thuộc tính quán (@>, ?), jsonb_ops vs jsonb_path_ops
--   C. GiST + btree_gist — ràng buộc "một tài xế không có hai ca chồng giờ" (EXCLUDE)
--   D. BRIN           — điểm GPS chèn theo thời gian: dữ liệu THEO THỨ TỰ vs XÁO TRỘN
-- Ma trận tổng hợp: p03/README.md (PG-028). Thời gian truy vấn: p03/DO-DAC.md.

CREATE EXTENSION pg_trgm WITH SCHEMA public;
CREATE EXTENSION unaccent WITH SCHEMA public;
CREATE EXTENSION btree_gist WITH SCHEMA public;
CREATE EXTENSION pageinspect WITH SCHEMA public;
SET search_path TO toinoi, public;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ===========================================================================
-- A. GIN + pg_trgm: tìm quán gần đúng.
-- Trigram = mọi cụm 3 ký tự liên tiếp của từng từ (thêm khoảng trắng đệm hai đầu).
-- Trigram có chữ có dấu được in dạng mã băm (0x…) — chúng vẫn là trigram bình thường.
-- (Cluster lab đặt LC_CTYPE để pg_trgm coi chữ tiếng Việt có dấu là CHỮ CÁI — xem README;
-- với LC_CTYPE=C, "Cơm Tấm" chỉ còn trigram của c, m, t.)
-- expect-value: 8
SELECT cardinality(show_trgm('Cơm Tấm'));

-- Chưa có index: Seq Scan qua 15.000 quán.
-- expect-match: Seq Scan on merchants
-- expect-match: actual rows=75\.00
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT name, address FROM merchants WHERE name ILIKE '%tấm bà năm%';

CREATE INDEX ix_merchants_name_trgm ON merchants USING gin (name gin_trgm_ops);

-- Có index GIN: Bitmap Index Scan lấy các quán chứa ĐỦ trigram của mẫu, rồi Recheck
-- lại biểu thức thật trên heap (trigram khớp chưa chắc chuỗi khớp).
-- expect-match: Bitmap Index Scan on ix_merchants_name_trgm
-- expect-match: actual rows=75\.00
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT name, address FROM merchants WHERE name ILIKE '%tấm bà năm%';

-- Giới hạn: mẫu dưới 3 ký tự không tạo được trigram ⇒ index không giúp gì.
-- expect-match: Seq Scan on merchants
EXPLAIN (COSTS OFF) SELECT count(*) FROM merchants WHERE name ILIKE '%gà%';

-- Gõ KHÔNG DẤU ("com tam") không khớp tên CÓ DẤU: trigram "tam" ≠ "tấm".
-- expect-value: 0
SELECT count(*) FROM merchants WHERE name ILIKE '%com tam ba nam%';

-- Cách làm thường gặp: index trên biểu thức bỏ dấu. unaccent() chỉ là STABLE (từ điển
-- có thể đổi) nên phải bọc trong hàm IMMUTABLE tự viết — chấp nhận rằng đổi từ điển
-- unaccent thì phải REINDEX.
CREATE FUNCTION khong_dau(text) RETURNS text
LANGUAGE sql IMMUTABLE PARALLEL SAFE STRICT
AS $$ SELECT lower(public.unaccent('public.unaccent'::regdictionary, $1)) $$;
CREATE INDEX ix_merchants_name_khong_dau ON merchants USING gin (khong_dau(name) gin_trgm_ops);

-- expect-match: Bitmap Index Scan on ix_merchants_name_khong_dau
-- expect-match: actual rows=75\.00
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT name FROM merchants WHERE khong_dau(name) LIKE '%com tam ba nam%';

-- Tìm theo độ GIỐNG (gõ sai chính tả): toán tử % (similarity ≥ pg_trgm.similarity_threshold
-- = 0,3) + sắp theo khoảng cách <->. Index GIN lọc ứng viên cho %.
-- expect-match: Bitmap Index Scan on ix_merchants_name_khong_dau
EXPLAIN (COSTS OFF)
SELECT name FROM merchants WHERE khong_dau(name) % 'com tam ba nan' ORDER BY khong_dau(name) <-> 'com tam ba nan' LIMIT 5;

-- expect-line: Cơm Tấm Bà Năm | 0.81
-- expect-line: Cơm Tấm Bà Năm | 0.81
-- expect-line: Cơm Tấm Bà Năm | 0.81
SELECT name, round(similarity(khong_dau(name), 'com tam ba nan')::numeric, 2) AS do_giong
FROM merchants WHERE khong_dau(name) % 'com tam ba nan'
ORDER BY khong_dau(name) <-> 'com tam ba nan', id LIMIT 3;

-- Địa chỉ: điều kiện trên nhiều cụm (đường + quận) — GIN gộp trigram của cả mẫu.
CREATE INDEX ix_merchants_address_trgm ON merchants USING gin (address gin_trgm_ops);
-- expect-match: Bitmap Index Scan on ix_merchants_address_trgm
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM merchants WHERE address ILIKE '%đường hoa sứ%quận 3,%';

-- Cái giá: index trigram LỚN (mỗi chuỗi sinh hàng chục trigram) — so với chính bảng.
-- expect-line: merchants                   | table
-- expect-line: ix_merchants_address_trgm   | index
-- expect-line: ix_merchants_name_khong_dau | index
-- expect-line: ix_merchants_name_trgm      | index
SELECT relname, CASE relkind WHEN 'r' THEN 'table' ELSE 'index' END AS loai
FROM pg_class WHERE relname IN ('merchants', 'ix_merchants_name_trgm', 'ix_merchants_name_khong_dau', 'ix_merchants_address_trgm')
  AND relnamespace = 'toinoi'::regnamespace
ORDER BY relkind DESC, relname;

SELECT relname, pg_size_pretty(pg_relation_size(oid)) AS kich_thuoc, pg_relation_size(oid) AS bytes
FROM pg_class WHERE relname IN ('merchants', 'ix_merchants_name_trgm', 'ix_merchants_name_khong_dau', 'ix_merchants_address_trgm')
  AND relnamespace = 'toinoi'::regnamespace
ORDER BY relkind DESC, relname;

-- ===========================================================================
-- B. GIN cho jsonb: quán có tag "chay".
-- expect-value: 1204
SELECT count(*) FROM merchants WHERE attrs @> '{"tags": ["chay"]}';

-- jsonb_path_ops: index nhỏ hơn, chỉ phục vụ @> (chứa) và jsonpath @? @@.
CREATE INDEX ix_merchants_attrs_path ON merchants USING gin (attrs jsonb_path_ops);
-- expect-match: Bitmap Index Scan on ix_merchants_attrs_path
EXPLAIN (COSTS OFF) SELECT count(*) FROM merchants WHERE attrs @> '{"tags": ["chay"]}';

-- Toán tử ? (có khoá) KHÔNG được jsonb_path_ops hỗ trợ ⇒ không dùng index đó.
-- (Khoá 'khuyen_mai' chưa quán nào có — câu hỏi rất chọn lọc, đáng lẽ index phải giúp.)
-- expect-no-match: ix_merchants_attrs_path
EXPLAIN (COSTS OFF) SELECT count(*) FROM merchants WHERE attrs ? 'khuyen_mai';

-- jsonb_ops (mặc định) hỗ trợ cả ? ?| ?& — đổi lại index to hơn.
CREATE INDEX ix_merchants_attrs ON merchants USING gin (attrs);
-- expect-match: Bitmap Index Scan on ix_merchants_attrs$
EXPLAIN (COSTS OFF) SELECT count(*) FROM merchants WHERE attrs ? 'khuyen_mai';

-- expect-line: ix_merchants_attrs      | t
-- expect-line: ix_merchants_attrs_path | f
SELECT relname, pg_relation_size(oid) > pg_relation_size('ix_merchants_attrs_path') AS lon_hon_path_ops
FROM pg_class WHERE relname IN ('ix_merchants_attrs', 'ix_merchants_attrs_path') AND relnamespace = 'toinoi'::regnamespace
ORDER BY relname;

-- ===========================================================================
-- C. GiST + btree_gist: "một tài xế không có hai ca chồng giờ" — quy tắc giữa NHIỀU
-- dòng mà CHECK/UNIQUE không diễn đạt được. EXCLUDE: không tồn tại hai dòng có
-- driver_id = nhau VÀ shift && nhau (&& = hai khoảng giao nhau). Cần btree_gist để
-- GiST hiểu toán tử = trên uuid.
-- Nền cài sẵn 3 ca chồng giờ ⇒ thêm ràng buộc thất bại ngay, kèm dòng vi phạm.
-- expect-error: 23P01 ex_driver_shifts_khong_chong
ALTER TABLE driver_shifts
  ADD CONSTRAINT ex_driver_shifts_khong_chong EXCLUDE USING gist (driver_id WITH =, shift WITH &&);

-- Tìm các cặp chồng nhau (self-join theo đúng toán tử của ràng buộc).
-- expect-line: 20007 | 2026-10-20 10:00:00+07
-- expect-line: 23605 | 2026-10-20 10:00:00+07
-- expect-line: 25401 | 2026-10-20 10:00:00+07
SELECT d.code, b.created_at AS ca_nhap_sau
FROM driver_shifts AS a
JOIN driver_shifts AS b ON b.driver_id = a.driver_id AND b.id > a.id AND b.shift && a.shift
JOIN drivers AS d ON d.id = a.driver_id
ORDER BY d.code;

-- Xoá ca nhập sau (lỗi nhập liệu) rồi thêm ràng buộc.
-- expect-rows: 3
DELETE FROM driver_shifts AS b
USING driver_shifts AS a
WHERE b.driver_id = a.driver_id AND b.id > a.id AND b.shift && a.shift;

ALTER TABLE driver_shifts
  ADD CONSTRAINT ex_driver_shifts_khong_chong EXCLUDE USING gist (driver_id WITH =, shift WITH &&);

-- Ràng buộc tự tạo một index GiST cùng tên.
-- expect-line: ex_driver_shifts_khong_chong | gist
SELECT c.relname, am.amname FROM pg_class AS c JOIN pg_am AS am ON am.oid = c.relam
WHERE c.relname = 'ex_driver_shifts_khong_chong';

-- Ca chồng giờ bị chặn, kèm DETAIL chỉ ra ca đang có.
SELECT lower(shift) AS bd, upper(shift) AS kt FROM driver_shifts
WHERE driver_id = p03_uuid('tx', 1) ORDER BY lower(shift) LIMIT 1 \gset
-- expect-error: 23P01 ex_driver_shifts_khong_chong
INSERT INTO driver_shifts (driver_id, shift)
VALUES (p03_uuid('tx', 1), tstzrange(:'bd'::timestamptz + interval '3 hours', :'kt'::timestamptz + interval '3 hours'));

-- Ca LIỀN SAU (bắt đầu đúng lúc ca trước kết thúc) hợp lệ: khoảng [bắt đầu, kết thúc) nửa mở.
-- expect-rows: 1
INSERT INTO driver_shifts (driver_id, shift)
VALUES (p03_uuid('tx', 1), tstzrange(:'kt'::timestamptz, :'kt'::timestamptz + interval '2 hours'));

-- Index GiST còn phục vụ câu hỏi "ai đang trong ca lúc t" (toán tử @> chứa điểm).
-- expect-match: ex_driver_shifts_khong_chong
EXPLAIN (COSTS OFF)
SELECT count(*) FROM driver_shifts WHERE shift @> TIMESTAMPTZ '2026-10-15 09:00+07';

-- expect-value: 1713
SELECT count(*) FROM driver_shifts WHERE shift @> TIMESTAMPTZ '2026-10-15 09:00+07';

-- ===========================================================================
-- D. BRIN: điểm GPS. 2.000.000 điểm của 1.000 tài xế, 5 giây một điểm, chèn THEO THỨ
-- TỰ THỜI GIAN (đúng như bảng chỉ-chèn ngoài đời) ⇒ page sau luôn chứa thời điểm muộn hơn.
-- Nạp lô: bỏ tạm khoá chính + khoá ngoại rồi tạo lại; giá trị theo tài xế tính MỘT lần
-- (CTE MATERIALIZED, 1.000 dòng) thay vì 2 triệu lần; work_mem lớn TẠM THỜI (SET LOCAL)
-- để sắp 2 triệu dòng trong bộ nhớ thay vì tràn ra đĩa (PG-029).
ALTER TABLE driver_locations DROP CONSTRAINT fk_driver_locations_driver, DROP CONSTRAINT pk_driver_locations;
BEGIN;
SET LOCAL work_mem = '256MB';
-- expect-rows: 2000000
INSERT INTO driver_locations (driver_id, recorded_at, lat, lng)
WITH tx AS MATERIALIZED (
  SELECT d, p03_uuid('tx', d) AS id,
         10.70 + p03_u(md5('gps' || d::text), 0) * 0.15 AS lat0,
         106.60 + p03_u(md5('gps' || d::text), 1) * 0.15 AS lng0
  FROM generate_series(1, 1000) AS d
)
SELECT tx.id,
       TIMESTAMPTZ '2026-10-31 01:00:00+07' - (2000 - i) * interval '5 seconds',
       tx.lat0 + i * 0.00001,
       tx.lng0 - i * 0.00001
FROM generate_series(1, 2000) AS i CROSS JOIN tx
ORDER BY i, tx.d;
-- Bản sao XÁO TRỘN thứ tự vật lý (cùng dữ liệu): như bảng nhập lô cũ không theo thời gian.
CREATE TABLE driver_locations_xao AS
SELECT * FROM driver_locations ORDER BY md5(driver_id::text || recorded_at::text);
COMMIT;
ALTER TABLE driver_locations ADD CONSTRAINT pk_driver_locations PRIMARY KEY (driver_id, recorded_at);
ALTER TABLE driver_locations ADD CONSTRAINT fk_driver_locations_driver FOREIGN KEY (driver_id) REFERENCES drivers (id);
ANALYZE driver_locations;
ANALYZE driver_locations_xao;

-- expect-line: driver_locations     | 1.000
-- expect-line: driver_locations_xao | 0.0
SELECT tablename, round(correlation::numeric, CASE WHEN tablename = 'driver_locations' THEN 3 ELSE 1 END) AS correlation
FROM pg_stats WHERE schemaname = 'toinoi' AND attname = 'recorded_at' AND tablename LIKE 'driver_locations%'
ORDER BY tablename;

CREATE INDEX ix_dl_brin     ON driver_locations     USING brin (recorded_at);
CREATE INDEX ix_dl_btree    ON driver_locations     (recorded_at);
CREATE INDEX ix_dl_xao_brin ON driver_locations_xao USING brin (recorded_at);

-- Kích thước: BRIN chỉ lưu (min, max) cho mỗi 128 page ⇒ 24 KB. B-tree trên cùng cột
-- 14 MB (đã nhỏ nhờ gộp khoá trùng: mỗi thời điểm có 1.000 điểm của 1.000 tài xế).
-- expect-line: driver_locations | 136536064
-- expect-line: ix_dl_btree      | 14655488
-- expect-line: ix_dl_brin       | 24576
SELECT relname, pg_relation_size(oid) AS bytes FROM pg_class
WHERE relname IN ('driver_locations', 'ix_dl_brin', 'ix_dl_btree') AND relnamespace = 'toinoi'::regnamespace
ORDER BY bytes DESC;

-- Mỗi mục BRIN: một dải 128 page và (min .. max) của recorded_at trong dải đó.
-- Bảng THEO THỨ TỰ: các dải nối tiếp nhau, không chồng.
-- expect-line: 0   | {2026-10-30 22:13:25+07 .. 2026-10-30 22:14:40+07}
-- expect-line: 128 | {2026-10-30 22:14:40+07 .. 2026-10-30 22:15:55+07}
-- expect-line: 256 | {2026-10-30 22:15:55+07 .. 2026-10-30 22:17:15+07}
SELECT blknum, value FROM brin_page_items(get_raw_page('ix_dl_brin', 2), 'ix_dl_brin') ORDER BY blknum LIMIT 3;

-- Bảng XÁO TRỘN: dải nào cũng phủ TRỌN 2 giờ 47 phút ⇒ không loại được dải nào.
-- expect-line: 0   | {2026-10-30 22:13:25+07 .. 2026-10-31 01:00:00+07}
-- expect-line: 128 | {2026-10-30 22:13:25+07 .. 2026-10-31 01:00:00+07}
-- expect-line: 256 | {2026-10-30 22:13:25+07 .. 2026-10-31 01:00:00+07}
SELECT blknum, value FROM brin_page_items(get_raw_page('ix_dl_xao_brin', 2), 'ix_dl_xao_brin') ORDER BY blknum LIMIT 3;

-- "5 phút gần nhất" (61.000 điểm ≈ 3 %) trên bảng THEO THỨ TỰ, chỉ cho phép BRIN:
-- Bitmap Heap Scan đọc 539 page (các dải 128 page khớp — "lossy" = cả dải), kiểm lại
-- và bỏ 3.640 dòng thừa ở dải biên.
BEGIN;
DROP INDEX ix_dl_btree;
SET LOCAL enable_indexscan = off;
-- expect-match: Bitmap Index Scan on ix_dl_brin
-- expect-match: actual rows=61000\.00
-- expect-match: Heap Blocks: lossy=539$
-- expect-match: Heap Blocks: lossy=\d+
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM driver_locations WHERE recorded_at >= TIMESTAMPTZ '2026-10-31 00:55:00+07';
ROLLBACK;

-- Cùng câu trên bảng XÁO TRỘN: BRIN không loại được dải nào. Planner biết (correlation
-- ≈ 0) nên chọn Seq Scan; ép dùng BRIN thì phải đọc LẠI CẢ BẢNG kèm kiểm lại 2 triệu dòng.
-- expect-match: Seq Scan on driver_locations_xao
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM driver_locations_xao WHERE recorded_at >= TIMESTAMPTZ '2026-10-31 00:55:00+07';

BEGIN;
SET LOCAL enable_seqscan = off;
-- expect-match: Bitmap Index Scan on ix_dl_xao_brin
-- expect-match: Rows Removed by Index Recheck: 1939000
-- expect-match: Heap Blocks: lossy=16704$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM driver_locations_xao WHERE recorded_at >= TIMESTAMPTZ '2026-10-31 00:55:00+07';
ROLLBACK;

-- So với B-tree trên bảng theo thứ tự: B-tree trỏ đúng từng dòng (không đọc dải thừa)
-- nhưng to gấp ~600 lần BRIN. Ở đây là Index Only Scan với "Heap Fetches: 61000" —
-- bảng vừa nạp chưa VACUUM nên visibility map trống (PG-026).
-- expect-match: Index Only Scan using ix_dl_btree on driver_locations|Index Scan using ix_dl_btree on driver_locations|Bitmap Index Scan on ix_dl_btree
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM driver_locations WHERE recorded_at >= TIMESTAMPTZ '2026-10-31 00:55:00+07';

-- Giới hạn của BRIN: dữ liệu MỚI chèn sau khi tạo index nằm ở dải chưa tóm tắt (luôn
-- phải đọc) cho tới khi VACUUM hoặc brin_summarize_new_values() tóm tắt
-- (hoặc tạo index với autosummarize = on).
-- expect-value: 0
SELECT brin_summarize_new_values('ix_dl_brin');
