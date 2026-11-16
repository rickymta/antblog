-- P02 · 00-mo-rong-schema.sql — mở rộng lược đồ P01 cho phần "T-SQL cho nghiệp vụ".
--
-- Chạy SAU p01/00-reset.sql và p01/01-schema.sql (KHÔNG chạy fixture P01):
--
--   node run.mjs p01/00-reset.sql p01/01-schema.sql p02      (= npm run p02)
--
-- P02 không nạp 15 chuyến của p01/02 mà sinh bộ dữ liệu 30 ngày riêng
-- (01-fixture.sql) — CÙNG dữ liệu với lab PostgreSQL P02 để đối chiếu hai engine.
-- File này chỉ THÊM:
--   * toinoi.areas               — cây khu vực: thành phố → vùng → khu (MS-019)
--   * toinoi.trips.pickup_area_id — khu đón; NULL khi mất GPS (MS-011, MS-019)
--   * toinoi.trip_ratings        — đánh giá sau chuyến, 0..1 dòng mỗi chuyến
--   * toinoi.payments            — giao dịch thanh toán, 0..n dòng mỗi chuyến
-- Bảng danh mục món cho MS-017 tạo ngay trong 17-upsert-merge.sql.

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

-- Chặn chạy nhầm khi chưa có lược đồ P01: báo lỗi rõ ràng rồi dừng (sqlcmd -b).
IF OBJECT_ID(N'toinoi.trips') IS NULL
    THROW 50000, N'Chưa có lược đồ P01 trong schema toinoi. Chạy p01/00-reset.sql và p01/01-schema.sql trước (npm run p02 làm sẵn).', 1;
GO

-- ---------------------------------------------------------------------------
-- Cây khu vực điều phối của Tới Nơi (hư cấu, KHÔNG phải đơn vị hành chính):
-- thành phố (gốc, parent_id NULL) → vùng → khu. id đặt theo quy ước dễ đọc:
-- thành phố 1..3, vùng 11..32, khu 111..322.
CREATE TABLE toinoi.areas (
    id        int           NOT NULL,
    parent_id int           NULL,
    code      varchar(12)   NOT NULL,
    name      nvarchar(100) NOT NULL,
    level     varchar(10)   NOT NULL,
    city_id   smallint      NULL,
    CONSTRAINT pk_areas PRIMARY KEY CLUSTERED (id),
    CONSTRAINT uq_areas_code UNIQUE (code),
    CONSTRAINT fk_areas_parent FOREIGN KEY (parent_id) REFERENCES toinoi.areas (id),
    CONSTRAINT fk_areas_city FOREIGN KEY (city_id) REFERENCES toinoi.cities (id),
    CONSTRAINT ck_areas_level CHECK (level IN ('city', 'district', 'zone')),
    -- Chỉ nút gốc (thành phố) không có cha, và chỉ nút gốc gắn với cities.
    CONSTRAINT ck_areas_root_no_parent CHECK (
        (level = 'city' AND parent_id IS NULL) OR (level <> 'city' AND parent_id IS NOT NULL)),
    CONSTRAINT ck_areas_root_has_city CHECK (
        (level = 'city' AND city_id IS NOT NULL) OR (level <> 'city' AND city_id IS NULL)),
    -- Chặn vòng 1 nút (tự làm cha mình)… nhưng KHÔNG chặn được vòng 2+ nút:
    -- CHECK chỉ nhìn một dòng. 19-chu-trinh.sql cho thấy hệ quả.
    CONSTRAINT ck_areas_not_own_parent CHECK (parent_id <> id)
);
CREATE INDEX ix_areas_parent ON toinoi.areas (parent_id);
GO

-- ---------------------------------------------------------------------------
-- Khu đón khách. NULL = app tài xế mất GPS lúc đón (thiếu dữ liệu). FK chỉ bảo
-- đảm khu tồn tại, KHÔNG bảo đảm đó là nút cấp "zone" hay cùng thành phố.
ALTER TABLE toinoi.trips ADD pickup_area_id int NULL
    CONSTRAINT fk_trips_pickup_area FOREIGN KEY REFERENCES toinoi.areas (id);
GO

-- ---------------------------------------------------------------------------
-- Đánh giá sau chuyến: tối đa MỘT dòng mỗi chuyến (khoá chính là trip_id).
--   * không có dòng          = khách chưa mở màn đánh giá
--   * stars NULL             = mở màn đánh giá nhưng bỏ trống phần sao
--   * tip NULL / tip = 0     = bỏ qua bước tip  /  chủ động chọn "không tip"
CREATE TABLE toinoi.trip_ratings (
    trip_id  uniqueidentifier  NOT NULL,
    stars    tinyint           NULL,
    tip      int               NULL,               -- đồng
    rated_at datetimeoffset(3) NOT NULL,
    CONSTRAINT pk_trip_ratings PRIMARY KEY CLUSTERED (trip_id),
    CONSTRAINT fk_trip_ratings_trip FOREIGN KEY (trip_id) REFERENCES toinoi.trips (id),
    CONSTRAINT ck_trip_ratings_stars CHECK (stars BETWEEN 1 AND 5),
    CONSTRAINT ck_trip_ratings_tip_non_negative CHECK (tip >= 0)
);
GO

-- ---------------------------------------------------------------------------
-- Giao dịch thanh toán: 0..n dòng mỗi chuyến.
--   kind   charge (thu) · refund (hoàn) · adjustment (điều chỉnh của CSKH, CÓ DẤU)
--   method cash (tiền mặt) · wallet (ví VíXanh) · card (thẻ)
--   status pending · succeeded · failed — chỉ succeeded làm đổi tiền thật
-- Thực thu của một chuyến = Σ charge − Σ refund + Σ adjustment (succeeded).
CREATE TABLE toinoi.payments (
    id           bigint            IDENTITY(1, 1) NOT NULL,
    trip_id      uniqueidentifier  NOT NULL,
    kind         varchar(10)       NOT NULL,
    method       varchar(10)       NOT NULL,
    status       varchar(10)       NOT NULL,
    amount       int               NOT NULL,        -- đồng
    provider_ref varchar(30)       NULL,            -- mã giao dịch phía ví/cổng thẻ
    paid_at      datetimeoffset(3) NOT NULL,
    CONSTRAINT pk_payments PRIMARY KEY CLUSTERED (id),
    CONSTRAINT fk_payments_trip FOREIGN KEY (trip_id) REFERENCES toinoi.trips (id),
    CONSTRAINT ck_payments_kind CHECK (kind IN ('charge', 'refund', 'adjustment')),
    CONSTRAINT ck_payments_method CHECK (method IN ('cash', 'wallet', 'card')),
    CONSTRAINT ck_payments_status CHECK (status IN ('pending', 'succeeded', 'failed')),
    CONSTRAINT ck_payments_amount_sign CHECK (
        (kind = 'adjustment' AND amount <> 0) OR (kind <> 'adjustment' AND amount > 0)),
    -- Tiền mặt không qua cổng ⇒ không có mã cổng; ví/thẻ luôn có.
    CONSTRAINT ck_payments_ref_by_method CHECK (
        (method = 'cash' AND provider_ref IS NULL) OR (method <> 'cash' AND provider_ref IS NOT NULL))
);
-- Cổng thanh toán không cấp trùng mã. KHÁC PostgreSQL: ràng buộc UNIQUE của
-- SQL Server chỉ cho MỘT dòng NULL, mà mọi giao dịch tiền mặt đều có
-- provider_ref NULL ⇒ dùng unique index có lọc (filtered index).
CREATE UNIQUE INDEX ux_payments_provider_ref ON toinoi.payments (provider_ref)
    WHERE provider_ref IS NOT NULL;
CREATE INDEX ix_payments_trip ON toinoi.payments (trip_id);
GO

-- Danh sách bảng sau khi mở rộng.
SELECT name AS bang FROM sys.tables WHERE schema_id = SCHEMA_ID(N'toinoi') ORDER BY name;
DECLARE @n nvarchar(10) = CONVERT(nvarchar(10), (SELECT COUNT(*) FROM sys.tables WHERE schema_id = SCHEMA_ID(N'toinoi')));
EXEC lab.expect_value N'số bảng trong schema toinoi', @n, N'9';
GO
