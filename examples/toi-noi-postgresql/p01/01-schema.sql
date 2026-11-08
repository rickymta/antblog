-- P01 · 01-schema.sql — lược đồ tối thiểu cho thành phố, khách, tài xế, quán,
-- chuyến và lịch sử trạng thái chuyến (PG-003, PG-004, PG-005).
--
-- Bám lược đồ lõi trong hồ sơ `du-an-mau-toi-noi`: cùng tên bảng, tên cột,
-- kiểu dữ liệu và giá trị mặc định. Khác biệt có chủ đích (xem p01/README.md):
--   1. Mọi ràng buộc đều được ĐẶT TÊN (pk_…, fk_…, uq_…, ck_…) để thông báo lỗi
--      chỉ thẳng vào quy tắc bị vi phạm.
--   2. Thêm các CHECK mã hoá quy tắc nghiệp vụ (định dạng số điện thoại, cước
--      không âm, thứ tự mốc thời gian, trạng thái ↔ tài xế ↔ mốc hoàn thành…).
--   3. `trip_events.status` có CHECK cùng danh sách 8 trạng thái với `trips`.
--   4. Chưa có `driver_locations` (thuộc các phần sau, không cần cho P01).
--
-- Quy ước kiểu dữ liệu (PG-004):
--   * Tiền: số nguyên ĐỒNG (`integer`). VND không dùng đơn vị lẻ, nên không cần
--     numeric/float. `integer` chứa tới 2.147.483.647 đ cho MỘT chuyến — dư sức;
--     `sum(integer)` trả về `bigint`, nên cộng hàng triệu chuyến vẫn không tràn.
--     Không dùng `real`/`double precision` (sai số nhị phân) hay kiểu `money`
--     (định dạng phụ thuộc lc_monetary của server).
--   * Thời điểm: `timestamptz` — lưu một khoảnh khắc tuyệt đối, hiển thị theo
--     TimeZone của phiên. Không dùng `timestamp` (không múi giờ) cho mốc sự kiện.
--   * Định danh: `uuid` sinh bằng gen_random_uuid() (UUID v4, có sẵn từ PG 13,
--     không cần extension) + mã số dễ đọc (`code`) cho người vận hành.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- Thành phố: danh mục nhỏ, id do người quản trị đặt (1 = SG, 2 = HN…).
CREATE TABLE cities (
  id    smallint NOT NULL,
  code  text     NOT NULL,
  name  text     NOT NULL,
  CONSTRAINT pk_cities PRIMARY KEY (id),
  CONSTRAINT uq_cities_code UNIQUE (code),
  CONSTRAINT ck_cities_code_format CHECK (code ~ '^[A-Z]{2}$'),
  CONSTRAINT ck_cities_name_not_blank CHECK (btrim(name) <> '')
);

-- ---------------------------------------------------------------------------
-- Khách hàng: đăng nhập bằng số điện thoại ⇒ số điện thoại là duy nhất.
CREATE TABLE customers (
  id         uuid        NOT NULL DEFAULT gen_random_uuid(),
  phone      text        NOT NULL,
  full_name  text        NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_customers PRIMARY KEY (id),
  CONSTRAINT uq_customers_phone UNIQUE (phone),
  -- Lưu MỘT dạng chuẩn: 10 chữ số bắt đầu bằng 0. Nếu cho phép cả '+84…' và
  -- '0…', UNIQUE không còn chặn được cùng một số viết theo hai cách.
  CONSTRAINT ck_customers_phone_format CHECK (phone ~ '^0[0-9]{9}$'),
  CONSTRAINT ck_customers_full_name_not_blank CHECK (btrim(full_name) <> '')
);

-- ---------------------------------------------------------------------------
-- Tài xế: `code` là mã nội bộ do bộ phận tuyển dụng cấp (8001, 8002…).
CREATE TABLE drivers (
  id           uuid        NOT NULL DEFAULT gen_random_uuid(),
  code         integer     NOT NULL,
  full_name    text        NOT NULL,
  city_id      smallint    NOT NULL,
  vehicle_type text        NOT NULL,
  status       text        NOT NULL DEFAULT 'offline',
  created_at   timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_drivers PRIMARY KEY (id),
  CONSTRAINT uq_drivers_code UNIQUE (code),
  CONSTRAINT fk_drivers_city FOREIGN KEY (city_id) REFERENCES cities (id),
  CONSTRAINT ck_drivers_code_positive CHECK (code > 0),
  CONSTRAINT ck_drivers_full_name_not_blank CHECK (btrim(full_name) <> ''),
  CONSTRAINT ck_drivers_vehicle_type CHECK (vehicle_type IN ('bike', 'car')),
  CONSTRAINT ck_drivers_status CHECK (status IN ('offline', 'online', 'busy'))
);

-- ---------------------------------------------------------------------------
-- Quán: cần cho quy tắc "đơn đồ ăn phải có quán". P01 chỉ giữ các cột lõi.
CREATE TABLE merchants (
  id         uuid        NOT NULL DEFAULT gen_random_uuid(),
  city_id    smallint    NOT NULL,
  name       text        NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_merchants PRIMARY KEY (id),
  CONSTRAINT fk_merchants_city FOREIGN KEY (city_id) REFERENCES cities (id),
  CONSTRAINT ck_merchants_name_not_blank CHECK (btrim(name) <> '')
);

-- ---------------------------------------------------------------------------
-- Chuyến: một yêu cầu đặt xe / giao đồ ăn / giao bưu kiện.
--
-- Lưu ý khi đọc các CHECK bên dưới: CHECK chỉ TỪ CHỐI khi biểu thức ra FALSE.
-- Nếu biểu thức ra NULL (vd. so sánh với accepted_at đang NULL) thì dòng được
-- CHẤP NHẬN. Vì vậy `accepted_at >= requested_at` tự bỏ qua chuyến chưa có
-- người nhận, còn các quy tắc "phải có" được viết tường minh bằng IS NULL /
-- IS NOT NULL.
CREATE TABLE trips (
  id           uuid        NOT NULL DEFAULT gen_random_uuid(),
  code         bigint      GENERATED ALWAYS AS IDENTITY (START WITH 100000),
  kind         text        NOT NULL,
  city_id      smallint    NOT NULL,
  customer_id  uuid        NOT NULL,
  driver_id    uuid,
  merchant_id  uuid,
  status       text        NOT NULL DEFAULT 'requested',
  fare         integer     NOT NULL,              -- đồng
  requested_at timestamptz NOT NULL DEFAULT now(),
  accepted_at  timestamptz,
  completed_at timestamptz,
  updated_at   timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_trips PRIMARY KEY (id),
  CONSTRAINT uq_trips_code UNIQUE (code),
  CONSTRAINT fk_trips_city     FOREIGN KEY (city_id)     REFERENCES cities (id),
  CONSTRAINT fk_trips_customer FOREIGN KEY (customer_id) REFERENCES customers (id),
  CONSTRAINT fk_trips_driver   FOREIGN KEY (driver_id)   REFERENCES drivers (id),
  CONSTRAINT fk_trips_merchant FOREIGN KEY (merchant_id) REFERENCES merchants (id),
  CONSTRAINT ck_trips_kind CHECK (kind IN ('ride', 'food', 'parcel')),
  CONSTRAINT ck_trips_status CHECK (status IN
    ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed', 'cancelled')),
  -- 0 đ hợp lệ (chuyến miễn phí do khuyến mãi); số âm thì không.
  CONSTRAINT ck_trips_fare_non_negative CHECK (fare >= 0),
  -- Đơn đồ ăn phải có quán; đặt xe và bưu kiện không gắn quán.
  CONSTRAINT ck_trips_merchant_only_food CHECK ((kind = 'food') = (merchant_id IS NOT NULL)),
  -- Có tài xế ⇔ có mốc nhận chuyến (kể cả chuyến đã nhận rồi bị huỷ).
  CONSTRAINT ck_trips_driver_accepted_together CHECK ((driver_id IS NULL) = (accepted_at IS NULL)),
  -- Hai quy tắc dạng "nếu A thì B", viết là `NOT A OR B`:
  --   chưa ai nhận (requested/searching) ⇒ chưa có tài xế;
  --   từ accepted tới completed ⇒ phải có tài xế.
  -- Chuyến huỷ có thể có hoặc không có tài xế (huỷ trước hay sau khi được nhận).
  -- Mỗi CHECK chỉ giữ MỘT quy tắc: trạng thái lạ như 'done' chỉ vi phạm
  -- ck_trips_status, thông báo lỗi không bị lẫn.
  CONSTRAINT ck_trips_unassigned_no_driver CHECK (
    status NOT IN ('requested', 'searching') OR driver_id IS NULL),
  CONSTRAINT ck_trips_assigned_has_driver CHECK (
    status NOT IN ('accepted', 'arriving', 'picked_up', 'in_transit', 'completed') OR driver_id IS NOT NULL),
  -- Có mốc hoàn thành ⇔ trạng thái hoàn thành.
  CONSTRAINT ck_trips_completed_at_by_status CHECK ((status = 'completed') = (completed_at IS NOT NULL)),
  CONSTRAINT ck_trips_accepted_after_requested CHECK (accepted_at >= requested_at),
  CONSTRAINT ck_trips_completed_after_accepted CHECK (completed_at >= accepted_at)
);

-- Ba index giống hồ sơ chung. P01 chưa cần chúng; P03 sẽ đo lợi ích và cái giá.
-- (PRIMARY KEY và UNIQUE tự tạo index riêng: pk_trips, uq_trips_code.)
CREATE INDEX ix_trips_city_requested     ON trips (city_id, requested_at);
CREATE INDEX ix_trips_customer_requested ON trips (customer_id, requested_at DESC);
CREATE INDEX ix_trips_driver_id          ON trips (driver_id);

-- ---------------------------------------------------------------------------
-- Lịch sử trạng thái: mỗi lần chuyến đổi trạng thái thêm một dòng, không sửa.
CREATE TABLE trip_events (
  id          bigint      GENERATED ALWAYS AS IDENTITY,
  trip_id     uuid        NOT NULL,
  status      text        NOT NULL,
  occurred_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_trip_events PRIMARY KEY (id),
  CONSTRAINT fk_trip_events_trip FOREIGN KEY (trip_id) REFERENCES trips (id),
  CONSTRAINT ck_trip_events_status CHECK (status IN
    ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed', 'cancelled'))
);
CREATE INDEX ix_trip_events_trip ON trip_events (trip_id, occurred_at);

-- ---------------------------------------------------------------------------
-- Chú thích: lưu ngay trong catalog, công cụ quản trị và psql (\d+) đọc được.
COMMENT ON TABLE cities IS 'Thành phố Tới Nơi hoạt động; id do quản trị đặt';
COMMENT ON TABLE customers IS 'Khách hàng; đăng nhập bằng số điện thoại';
COMMENT ON COLUMN customers.phone IS 'Số điện thoại dạng chuẩn 0xxxxxxxxx (10 chữ số), duy nhất';
COMMENT ON TABLE drivers IS 'Tài xế dùng chung cho ba dịch vụ';
COMMENT ON COLUMN drivers.code IS 'Mã tài xế nội bộ, dễ đọc cho người vận hành';
COMMENT ON COLUMN drivers.status IS 'offline / online (rảnh) / busy (đang chạy chuyến)';
COMMENT ON TABLE merchants IS 'Quán, cửa hàng nhận đơn đồ ăn';
COMMENT ON TABLE trips IS 'Một chuyến: đặt xe (ride), giao đồ ăn (food) hoặc giao bưu kiện (parcel)';
COMMENT ON COLUMN trips.id IS 'Định danh kỹ thuật UUID v4; ứng dụng hoặc database sinh';
COMMENT ON COLUMN trips.code IS 'Mã chuyến tăng dần cho người dùng và CSKH; có thể nhảy cóc';
COMMENT ON COLUMN trips.fare IS 'Cước tính bằng đồng (VND), số nguyên không âm';
COMMENT ON COLUMN trips.requested_at IS 'Thời điểm khách đặt (timestamptz)';
COMMENT ON COLUMN trips.accepted_at IS 'Thời điểm tài xế nhận; NULL khi chưa ai nhận';
COMMENT ON COLUMN trips.completed_at IS 'Thời điểm hoàn thành; chỉ có khi status = completed';
COMMENT ON COLUMN trips.updated_at IS 'Lần thay đổi gần nhất của dòng';
COMMENT ON TABLE trip_events IS 'Lịch sử đổi trạng thái chuyến; chỉ thêm, không sửa';

-- ---------------------------------------------------------------------------
-- Xem lại cấu trúc bảng trips bằng catalog chuẩn SQL (thay cho lệnh \d của psql,
-- vốn là lệnh của công cụ psql chứ không phải SQL).
SELECT column_name, data_type, is_nullable, column_default, is_identity
FROM information_schema.columns
WHERE table_schema = 'toinoi' AND table_name = 'trips'
ORDER BY ordinal_position;
