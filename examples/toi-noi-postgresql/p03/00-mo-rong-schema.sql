-- P03 · 00-mo-rong-schema.sql — mở rộng lược đồ cho phần "Lưu trữ, index và
-- kế hoạch thực thi" (PG-021 … PG-030). Chạy trên PostgreSQL 18 THẬT bằng
-- pg/run-pg.mjs, sau p01/00-reset.sql, p01/01-schema.sql, p02/00-mo-rong-schema.sql:
--
--   node pg/run-pg.mjs p03          (= npm run p03; thứ tự nền khai trong p03/lab.json)
--
-- P03 KHÔNG dùng dữ liệu 30 ngày của P02 mà sinh bộ dữ liệu lớn riêng
-- (01-fixture.sql: 1.000.000 chuyến) — index chỉ có ý nghĩa khi bảng đủ lớn.
-- Lược đồ P01/P02 giữ nguyên; file này chỉ THÊM:
--   * customers.kind        — 'personal' / 'business' (khách doanh nghiệp đặt rất nhiều: PG-027)
--   * merchants.address     — địa chỉ quán (tìm gần đúng bằng pg_trgm: PG-028)
--   * merchants.attrs       — thuộc tính quán dạng jsonb (GIN cho jsonb: PG-028)
--   * driver_shifts         — ca làm việc của tài xế, khoảng thời gian tstzrange (GiST + exclusion: PG-028)
--   * driver_locations      — điểm GPS, chỉ chèn theo thời gian (BRIN: PG-028); dữ liệu sinh trong bài 28
--   * uoc_luong(sql)        — hàm tiện ích: số dòng ƯỚC LƯỢNG và THỰC TẾ của một truy vấn (PG-021, PG-027)

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Chặn chạy nhầm khi thiếu nền P01 + P02 (báo một lỗi rõ ràng).
DO $$
BEGIN
  IF to_regclass('toinoi.trips') IS NULL OR to_regclass('toinoi.payments') IS NULL THEN
    RAISE EXCEPTION 'Chưa có lược đồ P01 + P02 trong schema toinoi'
      USING HINT = 'Chạy p01/00-reset.sql, p01/01-schema.sql, p02/00-mo-rong-schema.sql trước (npm run p03 làm sẵn).';
  END IF;
END
$$;

-- Lab P03+ chỉ kiểm trên PostgreSQL 18 (EXPLAIN ANALYZE tự kèm BUFFERS, B-tree
-- skip scan, pg_buffercache_evict_relation… đều là tính năng của bản 18).
-- expect-value: t
SELECT current_setting('server_version_num')::int >= 180000 AS la_postgresql_18;

-- ---------------------------------------------------------------------------
ALTER TABLE customers ADD COLUMN kind text NOT NULL DEFAULT 'personal';
ALTER TABLE customers ADD CONSTRAINT ck_customers_kind CHECK (kind IN ('personal', 'business'));
COMMENT ON COLUMN customers.kind IS 'personal = khách cá nhân; business = khách doanh nghiệp đặt qua API';

ALTER TABLE merchants ADD COLUMN address text;
ALTER TABLE merchants ADD COLUMN attrs jsonb NOT NULL DEFAULT '{}';
ALTER TABLE merchants ADD CONSTRAINT ck_merchants_attrs_object CHECK (jsonb_typeof(attrs) = 'object');
COMMENT ON COLUMN merchants.address IS 'Địa chỉ quán (hư cấu)';
COMMENT ON COLUMN merchants.attrs IS 'Thuộc tính quán: {"tags": [...], "giao_nhanh": bool, "gio_mo": "HH:MM"}';

-- ---------------------------------------------------------------------------
-- Ca làm việc: mỗi dòng một khoảng [bắt đầu, kết thúc) — dạng nửa mở, nên ca
-- 06:00–14:00 và ca 14:00–22:00 KHÔNG chồng nhau. Quy tắc "một tài xế không có
-- hai ca chồng giờ" là quy tắc giữa NHIỀU dòng: CHECK không diễn đạt được, UNIQUE
-- cũng không. Bài PG-028 thêm EXCLUDE USING gist (cần btree_gist) — chưa thêm
-- ở đây vì fixture CÀI SẴN vài ca chồng giờ để bài tìm và sửa.
CREATE TABLE driver_shifts (
  id         bigint      GENERATED ALWAYS AS IDENTITY,
  driver_id  uuid        NOT NULL,
  shift      tstzrange   NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_driver_shifts PRIMARY KEY (id),
  CONSTRAINT fk_driver_shifts_driver FOREIGN KEY (driver_id) REFERENCES drivers (id),
  CONSTRAINT ck_driver_shifts_bounded CHECK (NOT isempty(shift) AND lower(shift) IS NOT NULL AND upper(shift) IS NOT NULL),
  CONSTRAINT ck_driver_shifts_max_12h CHECK (upper(shift) - lower(shift) <= interval '12 hours')
);
COMMENT ON TABLE driver_shifts IS 'Ca làm việc của tài xế, [bắt đầu, kết thúc)';

-- ---------------------------------------------------------------------------
-- Điểm GPS: đúng như hồ sơ Tới Nơi (PK (driver_id, recorded_at)). Chỉ chèn, theo
-- thứ tự thời gian ⇒ thứ tự vật lý của heap thẳng hàng với recorded_at — điều
-- kiện để BRIN hiệu quả. Dữ liệu (2 triệu điểm) sinh trong 28-gin-gist-brin.sql,
-- không nằm trong nền để các bài khác chép database nhanh hơn.
CREATE TABLE driver_locations (
  driver_id   uuid             NOT NULL,
  recorded_at timestamptz      NOT NULL,
  lat         double precision NOT NULL,
  lng         double precision NOT NULL,
  CONSTRAINT pk_driver_locations PRIMARY KEY (driver_id, recorded_at),
  CONSTRAINT fk_driver_locations_driver FOREIGN KEY (driver_id) REFERENCES drivers (id),
  CONSTRAINT ck_driver_locations_lat CHECK (lat BETWEEN -90 AND 90),
  CONSTRAINT ck_driver_locations_lng CHECK (lng BETWEEN -180 AND 180)
);
COMMENT ON TABLE driver_locations IS 'Điểm GPS tài xế; chỉ chèn, đọc vài phút gần nhất';

-- ---------------------------------------------------------------------------
-- uoc_luong(sql): chạy EXPLAIN (ANALYZE, FORMAT JSON) cho MỘT câu SELECT và trả
-- số dòng ước lượng của nút gốc, số dòng thực tế, và hệ số lệch (lớn hơn / nhỏ
-- hơn, luôn ≥ 1). Dùng để KIỂM tự động "ước lượng lệch bao nhiêu lần" mà không
-- phụ thuộc con số ước lượng chính xác (ANALYZE lấy mẫu ngẫu nhiên trên bảng lớn
-- ⇒ ước lượng dao động nhẹ giữa các lần chạy).
-- CẢNH BÁO: EXPLAIN ANALYZE THỰC THI câu lệnh. Hàm chỉ nhận câu bắt đầu bằng SELECT/WITH.
CREATE FUNCTION uoc_luong(sql text, OUT uoc_tinh bigint, OUT thuc_te bigint, OUT lech numeric)
LANGUAGE plpgsql AS $$
DECLARE
  j jsonb;
BEGIN
  IF sql !~* '^\s*(SELECT|WITH)\M' THEN
    RAISE EXCEPTION 'uoc_luong() chỉ nhận câu SELECT/WITH (EXPLAIN ANALYZE thực thi câu lệnh)';
  END IF;
  EXECUTE 'EXPLAIN (ANALYZE, TIMING OFF, BUFFERS OFF, FORMAT JSON) ' || sql INTO j;
  uoc_tinh := (j -> 0 -> 'Plan' ->> 'Plan Rows')::numeric::bigint;
  thuc_te  := round((j -> 0 -> 'Plan' ->> 'Actual Rows')::numeric * (j -> 0 -> 'Plan' ->> 'Actual Loops')::numeric)::bigint;
  lech     := round(greatest(uoc_tinh, 1)::numeric / greatest(thuc_te, 1)
                    * CASE WHEN uoc_tinh >= thuc_te THEN 1 ELSE 0 END
                  + greatest(thuc_te, 1)::numeric / greatest(uoc_tinh, 1)
                    * CASE WHEN uoc_tinh < thuc_te THEN 1 ELSE 0 END, 1);
END
$$;
COMMENT ON FUNCTION uoc_luong(text) IS 'Lab P03: số dòng ước lượng/thực tế của nút gốc một câu SELECT (chạy EXPLAIN ANALYZE)';

-- ke_hoach(sql): tóm tắt kế hoạch (EXPLAIN COSTS OFF, KHÔNG chạy câu lệnh) thành một
-- dòng gồm các nút, từ gốc xuống, cách nhau " / " — để lập bảng "điều kiện → kiểu
-- scan" gọn trong một câu SELECT. Ví dụ: 'Aggregate / Seq Scan on trips'.
CREATE FUNCTION ke_hoach(sql text) RETURNS text
LANGUAGE plpgsql AS $$
DECLARE
  r record;
  nut text[] := '{}';
  dau boolean := true;
BEGIN
  FOR r IN EXECUTE 'EXPLAIN (COSTS OFF) ' || sql LOOP
    IF dau OR r."QUERY PLAN" ~ '^\s*->' THEN
      nut := nut || regexp_replace(r."QUERY PLAN", '^\s*(->\s*)?', '');
    END IF;
    dau := false;
  END LOOP;
  RETURN array_to_string(nut, ' / ');
END
$$;
COMMENT ON FUNCTION ke_hoach(text) IS 'Lab P03: các nút của kế hoạch (EXPLAIN COSTS OFF) trên một dòng';

-- Danh sách bảng sau khi mở rộng.
-- expect-line: areas
-- expect-line: cities
-- expect-line: customers
-- expect-line: driver_locations
-- expect-line: driver_shifts
-- expect-line: drivers
-- expect-line: merchants
-- expect-line: payments
-- expect-line: trip_events
-- expect-line: trip_ratings
-- expect-line: trips
SELECT table_name
FROM information_schema.tables
WHERE table_schema = 'toinoi' AND table_type = 'BASE TABLE'
ORDER BY table_name COLLATE "C";
