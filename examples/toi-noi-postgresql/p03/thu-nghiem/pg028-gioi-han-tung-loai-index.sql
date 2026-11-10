-- Thí nghiệm PG-028 «Khi nào cân nhắc GIN, GiST và BRIN?» — không có trong lab chính
-- (p03/28-gin-gist-brin.sql). Bổ sung cột "GIỚI HẠN" của ma trận workload × operator:
--   A. GIN: hàng chờ (pending list, fastupdate) — chèn nhanh nhưng tìm phải quét thêm.
--   B. GIN jsonb: ->> = '…' KHÔNG dùng index GIN; viết lại bằng @> thì dùng được.
--   C. Khoảng thời gian: B-tree trên tstzrange không phục vụ @>/&&; PostgreSQL 18 có
--      PRIMARY KEY … WITHOUT OVERLAPS (bên dưới là index GiST, cần btree_gist cho uuid).
--   D. BRIN: dải page mới CHƯA tóm tắt luôn bị đọc; pages_per_range 128 vs 32.
--   E. Locale: database LC_CTYPE=C ⇒ pg_trgm coi chữ có dấu là dấu cách.

CREATE EXTENSION pg_trgm WITH SCHEMA public;
CREATE EXTENSION btree_gist WITH SCHEMA public;
CREATE EXTENSION pgstattuple WITH SCHEMA public;
SET search_path TO toinoi, public;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ===========================================================================
-- A. GIN fastupdate: chèn 2.000 quán mới vào bảng có index trigram tên quán.
CREATE INDEX ix_merchants_name_trgm ON merchants USING gin (name gin_trgm_ops);
CREATE INDEX ix_merchants_name_trgm_off ON merchants USING gin (name gin_trgm_ops) WITH (fastupdate = off);

-- expect-rows: 2000
INSERT INTO merchants (id, city_id, name, created_at, address, attrs)
SELECT p03_uuid('q-moi', n), 1, 'Cơm Tấm Quán Mới ' || n, TIMESTAMPTZ '2026-10-31 00:00+07',
       n || ' Đường Hoa Sứ, Phường 1, Quận 3, TP. Hồ Chí Minh', '{"tags": ["com"]}'
FROM generate_series(1, 2000) AS n;

-- Index mặc định (fastupdate = on): mục mới nằm trong HÀNG CHỜ, chưa vào cây chính.
-- Index fastupdate = off: đã ghi thẳng vào cây (hàng chờ trống).
-- expect-line: ix_merchants_name_trgm     | t | t
-- expect-line: ix_merchants_name_trgm_off | f | f
SELECT i AS index_name, (pgstatginindex(i)).pending_pages > 0 AS co_trang_cho,
       (pgstatginindex(i)).pending_tuples > 0 AS co_muc_cho
FROM unnest(ARRAY['ix_merchants_name_trgm', 'ix_merchants_name_trgm_off']::regclass[]) AS i
ORDER BY 1::text;

-- lab: bien-dong
SELECT pending_pages, pending_tuples FROM pgstatginindex('ix_merchants_name_trgm');

-- Tìm vẫn đúng (planner quét cây chính + hàng chờ): 2.000 quán mới + quán cũ khớp.
DROP INDEX ix_merchants_name_trgm_off;
-- expect-match: Bitmap Index Scan on ix_merchants_name_trgm
-- expect-match: actual rows=2000\.00
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM merchants WHERE name ILIKE '%quán mới%';

-- Dọn hàng chờ vào cây chính (VACUUM/autovacuum cũng làm việc này).
-- expect-range: 1 1000
SELECT gin_clean_pending_list('ix_merchants_name_trgm');
-- expect-line: 0 | 0
SELECT pending_pages, pending_tuples FROM pgstatginindex('ix_merchants_name_trgm');

-- Giới hạn khác của trigram: index không phục vụ phép so sánh thứ tự (<, >) hay sắp xếp.
-- expect-no-match: ix_merchants_name_trgm
EXPLAIN (COSTS OFF) SELECT name FROM merchants WHERE name > 'Cơm Tấm' ORDER BY name LIMIT 5;

-- ===========================================================================
-- B. GIN jsonb: quán mở cửa lúc 07:00.
CREATE INDEX ix_merchants_attrs ON merchants USING gin (attrs);

-- expect-value: 2467
SELECT count(*) FROM merchants WHERE attrs ->> 'gio_mo' = '07:00';
-- ->> trả text; "text = text" không phải toán tử của jsonb_ops ⇒ Seq Scan.
-- expect-match: Seq Scan on merchants
-- expect-no-match: ix_merchants_attrs
EXPLAIN (COSTS OFF) SELECT count(*) FROM merchants WHERE attrs ->> 'gio_mo' = '07:00';

-- Cùng ý nghĩa viết bằng @> (chứa) ⇒ GIN phục vụ.
-- expect-value: 2467
SELECT count(*) FROM merchants WHERE attrs @> '{"gio_mo": "07:00"}';
-- expect-match: Bitmap Index Scan on ix_merchants_attrs
EXPLAIN (COSTS OFF) SELECT count(*) FROM merchants WHERE attrs @> '{"gio_mo": "07:00"}';

-- Muốn so sánh khoảng (mở trước 07:00) thì GIN jsonb không giúp: cần B-tree trên biểu thức.
CREATE INDEX ix_merchants_gio_mo ON merchants ((attrs ->> 'gio_mo'));
-- expect-match: ix_merchants_gio_mo
EXPLAIN (COSTS OFF) SELECT count(*) FROM merchants WHERE attrs ->> 'gio_mo' < '06:00';

-- ===========================================================================
-- C. Ca làm việc (tstzrange). B-tree trên kiểu range tồn tại (sắp theo cận dưới rồi cận
--    trên) nhưng chỉ phục vụ = < > — không phục vụ "chứa thời điểm" (@>) hay "giao nhau" (&&).
CREATE INDEX ix_driver_shifts_btree ON driver_shifts (shift);
-- expect-no-match: ix_driver_shifts_btree
EXPLAIN (COSTS OFF) SELECT count(*) FROM driver_shifts WHERE shift @> TIMESTAMPTZ '2026-10-15 09:00+07';
-- expect-no-match: ix_driver_shifts_btree
EXPLAIN (COSTS OFF) SELECT count(*) FROM driver_shifts
WHERE shift && tstzrange('2026-10-15 09:00+07', '2026-10-15 10:00+07');
DROP INDEX ix_driver_shifts_btree;

-- GiST trên range (không cần extension) phục vụ cả hai.
CREATE INDEX ix_driver_shifts_gist ON driver_shifts USING gist (shift);
-- expect-match: ix_driver_shifts_gist
EXPLAIN (COSTS OFF) SELECT count(*) FROM driver_shifts
WHERE shift && tstzrange('2026-10-15 09:00+07', '2026-10-15 10:00+07');
-- expect-value: 1713
SELECT count(*) FROM driver_shifts WHERE shift && tstzrange('2026-10-15 09:00+07', '2026-10-15 10:00+07');

-- PostgreSQL 18: khoá chính "theo thời gian" — WITHOUT OVERLAPS trên cột cuối.
CREATE TABLE driver_shifts_v18 (
  driver_id uuid      NOT NULL,
  shift     tstzrange NOT NULL,
  CONSTRAINT pk_driver_shifts_v18 PRIMARY KEY (driver_id, shift WITHOUT OVERLAPS)
);
-- expect-line: pk_driver_shifts_v18 | gist
SELECT c.relname, am.amname FROM pg_class AS c JOIN pg_am AS am ON am.oid = c.relam
WHERE c.relname = 'pk_driver_shifts_v18';

-- expect-rows: 2
INSERT INTO driver_shifts_v18 VALUES
  (p03_uuid('tx', 1), tstzrange('2026-11-01 07:00+07', '2026-11-01 15:00+07')),
  (p03_uuid('tx', 1), tstzrange('2026-11-01 15:00+07', '2026-11-01 19:00+07'));

-- Ca chồng giờ bị chặn như EXCLUDE.
-- expect-error: 23P01 pk_driver_shifts_v18
INSERT INTO driver_shifts_v18 VALUES (p03_uuid('tx', 1), tstzrange('2026-11-01 14:00+07', '2026-11-01 16:00+07'));

-- Khoảng rỗng không được phép trong cột WITHOUT OVERLAPS (lỗi CHECK 23514, không phải 23P01).
-- expect-error: 23514
INSERT INTO driver_shifts_v18 VALUES (p03_uuid('tx', 2), 'empty'::tstzrange);

-- ===========================================================================
-- D. BRIN: 500.000 điểm GPS (1.000 tài xế × 500 thời điểm, 5 giây/lần), chèn THEO THỨ TỰ.
CREATE TABLE gps_thu (driver_no int NOT NULL, recorded_at timestamptz NOT NULL, lat float8, lng float8);
-- expect-rows: 500000
INSERT INTO gps_thu
SELECT d, TIMESTAMPTZ '2026-10-31 00:00+07' + i * interval '5 seconds', 10.7 + d * 0.0001, 106.6 + i * 0.00001
FROM generate_series(0, 499) AS i CROSS JOIN generate_series(1, 1000) AS d
ORDER BY i, d;

CREATE INDEX ix_gps_brin128 ON gps_thu USING brin (recorded_at);
CREATE INDEX ix_gps_brin32 ON gps_thu USING brin (recorded_at) WITH (pages_per_range = 32);
VACUUM ANALYZE gps_thu;

-- expect-line: gps_thu        | 3677
-- expect-line: ix_gps_brin128 | 3
-- expect-line: ix_gps_brin32  | 3
SELECT relname, pg_relation_size(oid) / 8192 AS so_page FROM pg_class
WHERE relname IN ('gps_thu', 'ix_gps_brin128', 'ix_gps_brin32') ORDER BY relname;

-- "5 phút ĐẦU" (60 thời điểm = 60.000 điểm). Ép BRIN để so hai cỡ dải.
BEGIN;
SET LOCAL enable_seqscan = off;
DROP INDEX ix_gps_brin32;
-- expect-match: actual rows=60000\.00
-- expect-match: Heap Blocks: lossy=512$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM gps_thu WHERE recorded_at < TIMESTAMPTZ '2026-10-31 00:05:00+07';
ROLLBACK;

BEGIN;
SET LOCAL enable_seqscan = off;
DROP INDEX ix_gps_brin128;
-- expect-match: actual rows=60000\.00
-- expect-match: Heap Blocks: lossy=448$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM gps_thu WHERE recorded_at < TIMESTAMPTZ '2026-10-31 00:05:00+07';
ROLLBACK;
DROP INDEX ix_gps_brin32;

-- Nạp thêm 100.000 điểm MỚI (100 thời điểm sau). Dải page mới chưa có bản tóm tắt.
-- expect-rows: 100000
INSERT INTO gps_thu
SELECT d, TIMESTAMPTZ '2026-10-31 00:00+07' + i * interval '5 seconds', 10.7 + d * 0.0001, 106.6 + i * 0.00001
FROM generate_series(500, 599) AS i CROSS JOIN generate_series(1, 1000) AS d
ORDER BY i, d;

-- 3677 page đã tóm tắt (29 dải 128 page, dải cuối dở dang) → bảng giờ 4412 page.
-- expect-value: 4412
SELECT pg_relation_size('gps_thu') / 8192;

-- Câu hỏi về 5 phút ĐẦU giờ phải đọc thêm MỌI dải chưa tóm tắt (BRIN không biết trong đó
-- có gì) ⇒ số page lossy tăng dù kết quả không đổi.
BEGIN;
SET LOCAL enable_seqscan = off;
-- expect-match: actual rows=60000\.00
-- expect-match: Heap Blocks: lossy=1212$
-- expect-match: Rows Removed by Index Recheck: 104800$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM gps_thu WHERE recorded_at < TIMESTAMPTZ '2026-10-31 00:05:00+07';
ROLLBACK;

-- Tóm tắt các dải mới (VACUUM, autosummarize hoặc gọi tay) ⇒ trở lại như trước.
-- expect-value: 6
SELECT brin_summarize_new_values('ix_gps_brin128');
BEGIN;
SET LOCAL enable_seqscan = off;
-- expect-match: Heap Blocks: lossy=512$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM gps_thu WHERE recorded_at < TIMESTAMPTZ '2026-10-31 00:05:00+07';
ROLLBACK;

-- ===========================================================================
-- E. Locale: tạo database có LC_CTYPE=C (provider builtin như cluster lab) rồi so show_trgm.
-- lab: bien-dong
SELECT datlocprovider, datlocale, datctype FROM pg_database WHERE datname = current_database();
-- expect-value: 8
SELECT cardinality(show_trgm('Cơm Tấm'));
-- Hai món khác nhau: trên database này độ giống thấp (chỉ chung trigram "  c", "  t").
-- expect-value: 0.14
SELECT round(similarity('Cơm Tấm', 'Cám Tôm')::numeric, 2);

SELECT current_database() AS db_goc \gset
CREATE DATABASE thu_ctype_c TEMPLATE template0 ENCODING 'UTF8'
  LOCALE_PROVIDER builtin BUILTIN_LOCALE 'C.UTF-8' LC_COLLATE 'C' LC_CTYPE 'C';
\c thu_ctype_c
CREATE EXTENSION pg_trgm;
-- expect-line: C
SELECT datctype FROM pg_database WHERE datname = current_database();
-- Chữ có dấu thành dấu cách: "Cơm Tấm" chỉ còn các "từ" một chữ c, m, t.
-- expect-value: {"  c","  m","  t"," c "," m "," t "}
SELECT show_trgm('Cơm Tấm');
-- ⇒ "Cơm Tấm" và "Cám Tôm" giống nhau HOÀN TOÀN theo pg_trgm.
-- expect-value: 1.00
SELECT round(similarity('Cơm Tấm', 'Cám Tôm')::numeric, 2);
\c :db_goc
DROP DATABASE thu_ctype_c;
