-- P02 · 00-mo-rong-schema.sql — mở rộng lược đồ P01 cho phần "SQL cho nghiệp vụ".
--
-- Chạy SAU p01/00-reset.sql và p01/01-schema.sql (không chạy fixture P01):
--
--   node run.mjs p01/00-reset.sql p01/01-schema.sql p02      (= npm run p02)
--
-- P02 KHÔNG nạp 15 chuyến của p01/02-fixture-valid.sql mà sinh một bộ dữ liệu
-- 30 ngày riêng (01-fixture.sql). Lý do và hình dạng dữ liệu: p02/README.md.
-- Lược đồ P01 giữ nguyên; file này chỉ THÊM:
--   * areas            — cây khu vực điều phối: thành phố → vùng → khu (PG-019)
--   * trips.pickup_area_id — khu đón khách; NULL khi mất GPS (PG-011, PG-019)
--   * trip_ratings     — đánh giá sau chuyến, 0..1 dòng mỗi chuyến (PG-011, PG-012)
--   * payments         — giao dịch thanh toán, 0..n dòng mỗi chuyến (PG-013, PG-020)
-- Bảng nhập danh mục món cho PG-017 được tạo ngay trong 17-upsert.sql.
-- Không thêm drivers.vehicle_plate của P01 · 06 (P02 không cần; xem p01/README).

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Chặn chạy nhầm khi chưa có lược đồ P01: báo lỗi rõ ràng thay vì một loạt
-- lỗi "relation does not exist" ở các câu sau.
DO $$
BEGIN
  IF to_regclass('toinoi.trips') IS NULL THEN
    RAISE EXCEPTION 'Chưa có lược đồ P01 trong schema toinoi'
      USING HINT = 'Chạy p01/00-reset.sql và p01/01-schema.sql trước (npm run p02 làm sẵn việc này).';
  END IF;
END
$$;

-- ---------------------------------------------------------------------------
-- Cây khu vực điều phối của Tới Nơi (hư cấu, KHÔNG phải đơn vị hành chính):
-- thành phố (gốc, parent_id NULL) → vùng → khu. Chuyến được gắn vào KHU.
-- id đặt tay theo quy ước dễ đọc: thành phố 1..3, vùng 11..34, khu 111..322.
CREATE TABLE areas (
  id        integer  NOT NULL,
  parent_id integer,
  code      text     NOT NULL,
  name      text     NOT NULL,
  level     text     NOT NULL,
  city_id   smallint,
  CONSTRAINT pk_areas PRIMARY KEY (id),
  CONSTRAINT uq_areas_code UNIQUE (code),
  CONSTRAINT fk_areas_parent FOREIGN KEY (parent_id) REFERENCES areas (id),
  CONSTRAINT fk_areas_city FOREIGN KEY (city_id) REFERENCES cities (id),
  CONSTRAINT ck_areas_level CHECK (level IN ('city', 'district', 'zone')),
  -- Chỉ nút gốc (thành phố) không có cha, và chỉ nút gốc gắn với cities.
  CONSTRAINT ck_areas_root_no_parent CHECK ((level = 'city') = (parent_id IS NULL)),
  CONSTRAINT ck_areas_root_has_city CHECK ((level = 'city') = (city_id IS NOT NULL)),
  -- Chặn được vòng 1 nút (tự làm cha mình)… nhưng KHÔNG chặn được vòng 2+ nút:
  -- CHECK chỉ nhìn một dòng. 19-chu-trinh.sql cho thấy hệ quả.
  CONSTRAINT ck_areas_not_own_parent CHECK (parent_id <> id)
);
CREATE INDEX ix_areas_parent ON areas (parent_id);

COMMENT ON TABLE areas IS 'Cây khu vực điều phối: city → district (vùng) → zone (khu); dữ liệu hư cấu';

-- ---------------------------------------------------------------------------
-- Khu đón khách của chuyến. Cho phép NULL: app tài xế mất GPS lúc đón ⇒ không
-- biết khu (thiếu dữ liệu). FK chỉ bảo đảm khu tồn tại, KHÔNG bảo đảm đó là nút
-- cấp "zone" hay cùng thành phố với chuyến (19-cay-khu-vuc.sql kiểm).
ALTER TABLE trips ADD COLUMN pickup_area_id integer;
ALTER TABLE trips
  ADD CONSTRAINT fk_trips_pickup_area FOREIGN KEY (pickup_area_id) REFERENCES areas (id);

COMMENT ON COLUMN trips.pickup_area_id IS 'Khu đón (areas.level = zone); NULL = không xác định được (mất GPS)';

-- ---------------------------------------------------------------------------
-- Đánh giá sau chuyến: tối đa MỘT dòng mỗi chuyến (khoá chính là trip_id).
--   * không có dòng          = khách chưa mở màn đánh giá
--   * stars NULL             = khách mở màn đánh giá nhưng bỏ trống phần sao
--   * tip NULL / tip = 0     = bỏ qua bước tip  /  chủ động chọn "không tip"
-- Chuyến huỷ không được đánh giá (không áp dụng) — quy tắc nhiều bảng, CHECK
-- một dòng không diễn đạt được; 02-kiem-tra-fixture.sql kiểm.
CREATE TABLE trip_ratings (
  trip_id  uuid        NOT NULL,
  stars    smallint,
  tip      integer,                       -- đồng
  rated_at timestamptz NOT NULL,
  CONSTRAINT pk_trip_ratings PRIMARY KEY (trip_id),
  CONSTRAINT fk_trip_ratings_trip FOREIGN KEY (trip_id) REFERENCES trips (id),
  CONSTRAINT ck_trip_ratings_stars CHECK (stars BETWEEN 1 AND 5),
  CONSTRAINT ck_trip_ratings_tip_non_negative CHECK (tip >= 0)
);

COMMENT ON TABLE trip_ratings IS 'Đánh giá sau chuyến; 0..1 dòng mỗi chuyến';
COMMENT ON COLUMN trip_ratings.stars IS '1..5 sao; NULL = khách bỏ trống phần sao';
COMMENT ON COLUMN trip_ratings.tip IS 'Tiền tip (đồng); NULL = bỏ qua bước tip, 0 = chọn không tip';

-- ---------------------------------------------------------------------------
-- Giao dịch thanh toán: 0..n dòng mỗi chuyến.
--   kind   charge (thu) · refund (hoàn) · adjustment (điều chỉnh của CSKH, CÓ DẤU)
--   method cash (tiền mặt, tài xế ghi nhận) · wallet (ví VíXanh) · card (thẻ)
--   status pending · succeeded · failed — chỉ succeeded làm đổi tiền thật
-- Số tiền luôn là số nguyên đồng. charge/refund ghi số DƯƠNG, chiều tiền do
-- kind quyết định; adjustment ghi số có dấu (+ thu thêm, − giảm trừ).
-- Tiền thực thu của một chuyến = Σ charge − Σ refund + Σ adjustment (succeeded).
CREATE TABLE payments (
  id           bigint      GENERATED ALWAYS AS IDENTITY,
  trip_id      uuid        NOT NULL,
  kind         text        NOT NULL,
  method       text        NOT NULL,
  status       text        NOT NULL,
  amount       integer     NOT NULL,       -- đồng
  provider_ref text,                       -- mã giao dịch phía ví/cổng thẻ
  paid_at      timestamptz NOT NULL,       -- thời điểm phát sinh giao dịch
  CONSTRAINT pk_payments PRIMARY KEY (id),
  CONSTRAINT fk_payments_trip FOREIGN KEY (trip_id) REFERENCES trips (id),
  -- Cổng thanh toán không cấp trùng mã; NULL (tiền mặt) không bị UNIQUE chặn.
  CONSTRAINT uq_payments_provider_ref UNIQUE (provider_ref),
  CONSTRAINT ck_payments_kind CHECK (kind IN ('charge', 'refund', 'adjustment')),
  CONSTRAINT ck_payments_method CHECK (method IN ('cash', 'wallet', 'card')),
  CONSTRAINT ck_payments_status CHECK (status IN ('pending', 'succeeded', 'failed')),
  CONSTRAINT ck_payments_amount_sign CHECK (
    (kind = 'adjustment' AND amount <> 0) OR (kind <> 'adjustment' AND amount > 0)),
  -- Tiền mặt không qua cổng ⇒ không có mã cổng; ví/thẻ luôn có.
  CONSTRAINT ck_payments_ref_by_method CHECK ((method = 'cash') = (provider_ref IS NULL))
);
CREATE INDEX ix_payments_trip ON payments (trip_id);

COMMENT ON TABLE payments IS 'Giao dịch thanh toán của chuyến; 0..n dòng mỗi chuyến';
COMMENT ON COLUMN payments.amount IS 'Đồng; charge/refund > 0, adjustment có dấu';

-- Danh sách bảng sau khi mở rộng (thay cho \dt của psql).
-- expect-line: areas
-- expect-line: cities
-- expect-line: customers
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
