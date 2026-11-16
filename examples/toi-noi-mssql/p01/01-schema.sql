-- P01 · 01-schema.sql — lược đồ tối thiểu cho thành phố, khách, tài xế, quán,
-- chuyến và lịch sử trạng thái chuyến (MS-003, MS-004, MS-005).
--
-- Cùng nghiệp vụ với hồ sơ `du-an-mau-toi-noi` (cùng tên bảng, tên cột, quy
-- tắc), nhưng THIẾT KẾ THEO SQL SERVER chứ không dịch cú pháp từ PostgreSQL:
--   1. Bảng `trips` có PRIMARY KEY NONCLUSTERED trên `id` (uniqueidentifier) và
--      clustered index UNIQUE trên `code` (bigint IDENTITY tăng dần). Trong SQL
--      Server, clustered index QUYẾT ĐỊNH thứ tự lưu của cả bảng (MS-003); khoá
--      GUID ngẫu nhiên làm clustered key thì mỗi chuyến mới chèn vào giữa bảng.
--   2. Mọi ràng buộc đều ĐẶT TÊN (pk_, fk_, uq_, ck_, df_) — kể cả DEFAULT, vì
--      SQL Server tự sinh tên ngẫu nhiên kiểu DF__trips__statu__3B75D760 nếu
--      không đặt, và script migration sau này không gọi được tên đó.
--   3. CHECK của T-SQL không có kiểu boolean: quy tắc "A tương đương B" phải viết
--      thành (A AND B) OR (NOT A AND NOT B) (MS-005).
--
-- Quy ước kiểu dữ liệu (MS-004):
--   * Tiền: số nguyên ĐỒNG, `int`. Không dùng `money` (4 chữ số lẻ, phép chia
--     làm tròn bất ngờ) hay `float` (sai số nhị phân). SUM(int) trả int và CÓ
--     THỂ TRÀN (lỗi 8115) — khi cộng doanh thu nhiều tháng dùng SUM(CAST(fare AS bigint)).
--   * Thời điểm: `datetimeoffset(3)` — lưu khoảnh khắc kèm độ lệch giờ, so sánh
--     theo UTC. Không dùng `datetime` (độ chính xác 1/300 giây, không múi giờ).
--   * Chuỗi: `nvarchar` cho chữ người đọc có dấu tiếng Việt; `varchar` cho mã
--     ASCII máy đọc (trạng thái, loại dịch vụ, số điện thoại).
--   * Định danh: `uniqueidentifier` + mã số dễ đọc `code` cho người vận hành.

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

-- ---------------------------------------------------------------------------
-- Thành phố: danh mục nhỏ, id do người quản trị đặt (1 = SG, 2 = HN…).
CREATE TABLE toinoi.cities (
    id    smallint      NOT NULL,
    code  char(2)       NOT NULL,
    name  nvarchar(100) NOT NULL,
    CONSTRAINT pk_cities PRIMARY KEY CLUSTERED (id),
    CONSTRAINT uq_cities_code UNIQUE (code),
    -- Collation của database không phân biệt hoa thường (CI): '[A-Z]' sẽ khớp
    -- cả 'sg'. Ép collation nhị phân ngay trong CHECK để chỉ nhận chữ IN HOA.
    CONSTRAINT ck_cities_code_format CHECK (code COLLATE Latin1_General_BIN2 LIKE '[A-Z][A-Z]'),
    CONSTRAINT ck_cities_name_not_blank CHECK (TRIM(name) <> N'')
);

-- ---------------------------------------------------------------------------
-- Khách hàng: đăng nhập bằng số điện thoại ⇒ số điện thoại là duy nhất.
CREATE TABLE toinoi.customers (
    id         uniqueidentifier  NOT NULL CONSTRAINT df_customers_id DEFAULT NEWSEQUENTIALID(),
    phone      varchar(10)       NOT NULL,
    full_name  nvarchar(100)     NOT NULL,
    created_at datetimeoffset(3) NOT NULL CONSTRAINT df_customers_created_at DEFAULT SYSDATETIMEOFFSET(),
    CONSTRAINT pk_customers PRIMARY KEY CLUSTERED (id),
    CONSTRAINT uq_customers_phone UNIQUE (phone),
    -- Một dạng chuẩn: 10 chữ số bắt đầu bằng 0. T-SQL không có regex; LIKE với
    -- lớp ký tự [0-9] đủ cho quy tắc này. varchar(10) đã chặn chuỗi dài hơn.
    CONSTRAINT ck_customers_phone_format CHECK (phone LIKE '0[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]'),
    CONSTRAINT ck_customers_full_name_not_blank CHECK (TRIM(full_name) <> N'')
);

-- ---------------------------------------------------------------------------
-- Tài xế: `code` là mã nội bộ do bộ phận tuyển dụng cấp (8001, 8002…).
CREATE TABLE toinoi.drivers (
    id           uniqueidentifier  NOT NULL CONSTRAINT df_drivers_id DEFAULT NEWSEQUENTIALID(),
    code         int               NOT NULL,
    full_name    nvarchar(100)     NOT NULL,
    city_id      smallint          NOT NULL,
    vehicle_type varchar(10)       NOT NULL,
    status       varchar(10)       NOT NULL CONSTRAINT df_drivers_status DEFAULT 'offline',
    created_at   datetimeoffset(3) NOT NULL CONSTRAINT df_drivers_created_at DEFAULT SYSDATETIMEOFFSET(),
    CONSTRAINT pk_drivers PRIMARY KEY CLUSTERED (id),
    CONSTRAINT uq_drivers_code UNIQUE (code),
    CONSTRAINT fk_drivers_city FOREIGN KEY (city_id) REFERENCES toinoi.cities (id),
    CONSTRAINT ck_drivers_code_positive CHECK (code > 0),
    CONSTRAINT ck_drivers_full_name_not_blank CHECK (TRIM(full_name) <> N''),
    CONSTRAINT ck_drivers_vehicle_type CHECK (vehicle_type IN ('bike', 'car')),
    CONSTRAINT ck_drivers_status CHECK (status IN ('offline', 'online', 'busy'))
);

-- ---------------------------------------------------------------------------
-- Quán: cần cho quy tắc "đơn đồ ăn phải có quán". P01 chỉ giữ các cột lõi.
CREATE TABLE toinoi.merchants (
    id         uniqueidentifier  NOT NULL CONSTRAINT df_merchants_id DEFAULT NEWSEQUENTIALID(),
    city_id    smallint          NOT NULL,
    name       nvarchar(200)     NOT NULL,
    created_at datetimeoffset(3) NOT NULL CONSTRAINT df_merchants_created_at DEFAULT SYSDATETIMEOFFSET(),
    CONSTRAINT pk_merchants PRIMARY KEY CLUSTERED (id),
    CONSTRAINT fk_merchants_city FOREIGN KEY (city_id) REFERENCES toinoi.cities (id),
    CONSTRAINT ck_merchants_name_not_blank CHECK (TRIM(name) <> N'')
);

-- ---------------------------------------------------------------------------
-- Chuyến: một yêu cầu đặt xe / giao đồ ăn / giao bưu kiện.
--
-- CHECK chỉ TỪ CHỐI khi biểu thức ra FALSE. Biểu thức ra UNKNOWN (so sánh với
-- NULL) thì dòng được CHẤP NHẬN: `accepted_at >= requested_at` tự bỏ qua chuyến
-- chưa có người nhận. Các quy tắc "phải có" viết tường minh bằng IS NULL.
CREATE TABLE toinoi.trips (
    id           uniqueidentifier  NOT NULL CONSTRAINT df_trips_id DEFAULT NEWID(),
    code         bigint            IDENTITY(100000, 1) NOT NULL,
    kind         varchar(10)       NOT NULL,
    city_id      smallint          NOT NULL,
    customer_id  uniqueidentifier  NOT NULL,
    driver_id    uniqueidentifier  NULL,
    merchant_id  uniqueidentifier  NULL,
    status       varchar(12)       NOT NULL CONSTRAINT df_trips_status DEFAULT 'requested',
    fare         int               NOT NULL,              -- đồng
    requested_at datetimeoffset(3) NOT NULL CONSTRAINT df_trips_requested_at DEFAULT SYSDATETIMEOFFSET(),
    accepted_at  datetimeoffset(3) NULL,
    completed_at datetimeoffset(3) NULL,
    updated_at   datetimeoffset(3) NOT NULL CONSTRAINT df_trips_updated_at DEFAULT SYSDATETIMEOFFSET(),
    -- Khoá chính là id (ứng dụng tham chiếu, khoá ngoại trỏ tới) nhưng KHÔNG
    -- clustered; bảng được sắp xếp vật lý theo code tăng dần.
    CONSTRAINT pk_trips PRIMARY KEY NONCLUSTERED (id),
    CONSTRAINT uq_trips_code UNIQUE CLUSTERED (code),
    CONSTRAINT fk_trips_city     FOREIGN KEY (city_id)     REFERENCES toinoi.cities (id),
    CONSTRAINT fk_trips_customer FOREIGN KEY (customer_id) REFERENCES toinoi.customers (id),
    CONSTRAINT fk_trips_driver   FOREIGN KEY (driver_id)   REFERENCES toinoi.drivers (id),
    CONSTRAINT fk_trips_merchant FOREIGN KEY (merchant_id) REFERENCES toinoi.merchants (id),
    CONSTRAINT ck_trips_kind CHECK (kind IN ('ride', 'food', 'parcel')),
    CONSTRAINT ck_trips_status CHECK (status IN
        ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed', 'cancelled')),
    -- 0 đ hợp lệ (chuyến miễn phí do khuyến mãi); số âm thì không.
    CONSTRAINT ck_trips_fare_non_negative CHECK (fare >= 0),
    -- Đơn đồ ăn phải có quán; đặt xe và bưu kiện không gắn quán.
    CONSTRAINT ck_trips_merchant_only_food CHECK (
        (kind = 'food' AND merchant_id IS NOT NULL) OR (kind <> 'food' AND merchant_id IS NULL)),
    -- Có tài xế ⇔ có mốc nhận chuyến (kể cả chuyến đã nhận rồi bị huỷ).
    CONSTRAINT ck_trips_driver_accepted_together CHECK (
        (driver_id IS NULL AND accepted_at IS NULL) OR (driver_id IS NOT NULL AND accepted_at IS NOT NULL)),
    -- "Nếu A thì B" viết là NOT A OR B. Mỗi CHECK giữ MỘT quy tắc để thông báo
    -- lỗi chỉ đúng quy tắc bị vi phạm.
    CONSTRAINT ck_trips_unassigned_no_driver CHECK (
        status NOT IN ('requested', 'searching') OR driver_id IS NULL),
    CONSTRAINT ck_trips_assigned_has_driver CHECK (
        status NOT IN ('accepted', 'arriving', 'picked_up', 'in_transit', 'completed') OR driver_id IS NOT NULL),
    -- Có mốc hoàn thành ⇔ trạng thái hoàn thành.
    CONSTRAINT ck_trips_completed_at_by_status CHECK (
        (status = 'completed' AND completed_at IS NOT NULL) OR (status <> 'completed' AND completed_at IS NULL)),
    CONSTRAINT ck_trips_accepted_after_requested CHECK (accepted_at >= requested_at),
    CONSTRAINT ck_trips_completed_after_accepted CHECK (completed_at >= accepted_at)
);

-- Ba index giống hồ sơ chung. P01 chưa cần chúng; P03 sẽ đo lợi ích và cái giá.
-- SQL Server KHÔNG tự tạo index cho khoá ngoại — ix_trips_driver_id là index
-- ta chủ động tạo.
CREATE INDEX ix_trips_city_requested     ON toinoi.trips (city_id, requested_at);
CREATE INDEX ix_trips_customer_requested ON toinoi.trips (customer_id, requested_at DESC);
CREATE INDEX ix_trips_driver_id          ON toinoi.trips (driver_id);

-- ---------------------------------------------------------------------------
-- Lịch sử trạng thái: mỗi lần chuyến đổi trạng thái thêm một dòng, không sửa.
CREATE TABLE toinoi.trip_events (
    id          bigint            IDENTITY(1, 1) NOT NULL,
    trip_id     uniqueidentifier  NOT NULL,
    status      varchar(12)       NOT NULL,
    occurred_at datetimeoffset(3) NOT NULL CONSTRAINT df_trip_events_occurred_at DEFAULT SYSDATETIMEOFFSET(),
    CONSTRAINT pk_trip_events PRIMARY KEY CLUSTERED (id),
    CONSTRAINT fk_trip_events_trip FOREIGN KEY (trip_id) REFERENCES toinoi.trips (id),
    CONSTRAINT ck_trip_events_status CHECK (status IN
        ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed', 'cancelled'))
);
CREATE INDEX ix_trip_events_trip ON toinoi.trip_events (trip_id, occurred_at);
GO

-- ---------------------------------------------------------------------------
-- Chú thích lưu trong catalog dưới dạng extended property MS_Description;
-- DBStudio, SSMS và công cụ sinh tài liệu đọc được.
DECLARE @t TABLE (tbl sysname, col sysname NULL, txt nvarchar(400));
INSERT INTO @t VALUES
    (N'cities',      NULL,           N'Thành phố Tới Nơi hoạt động; id do quản trị đặt'),
    (N'customers',   NULL,           N'Khách hàng; đăng nhập bằng số điện thoại'),
    (N'customers',   N'phone',       N'Số điện thoại dạng chuẩn 0xxxxxxxxx (10 chữ số), duy nhất'),
    (N'drivers',     NULL,           N'Tài xế dùng chung cho ba dịch vụ'),
    (N'drivers',     N'code',        N'Mã tài xế nội bộ, dễ đọc cho người vận hành'),
    (N'drivers',     N'status',      N'offline / online (rảnh) / busy (đang chạy chuyến)'),
    (N'merchants',   NULL,           N'Quán, cửa hàng nhận đơn đồ ăn'),
    (N'trips',       NULL,           N'Một chuyến: đặt xe (ride), giao đồ ăn (food) hoặc giao bưu kiện (parcel)'),
    (N'trips',       N'id',          N'Định danh kỹ thuật uniqueidentifier; khoá chính NONCLUSTERED'),
    (N'trips',       N'code',        N'Mã chuyến IDENTITY tăng dần, clustered key; có thể nhảy cóc'),
    (N'trips',       N'fare',        N'Cước tính bằng đồng (VND), số nguyên không âm'),
    (N'trips',       N'requested_at',N'Thời điểm khách đặt (datetimeoffset)'),
    (N'trips',       N'accepted_at', N'Thời điểm tài xế nhận; NULL khi chưa ai nhận'),
    (N'trips',       N'completed_at',N'Thời điểm hoàn thành; chỉ có khi status = completed'),
    (N'trip_events', NULL,           N'Lịch sử đổi trạng thái chuyến; chỉ thêm, không sửa');

DECLARE @tbl sysname, @col sysname, @txt nvarchar(400);
DECLARE c CURSOR LOCAL FAST_FORWARD FOR SELECT tbl, col, txt FROM @t;
OPEN c;
FETCH NEXT FROM c INTO @tbl, @col, @txt;
WHILE @@FETCH_STATUS = 0
BEGIN
    IF @col IS NULL
        EXEC sys.sp_addextendedproperty @name = N'MS_Description', @value = @txt,
             @level0type = N'SCHEMA', @level0name = N'toinoi', @level1type = N'TABLE', @level1name = @tbl;
    ELSE
        EXEC sys.sp_addextendedproperty @name = N'MS_Description', @value = @txt,
             @level0type = N'SCHEMA', @level0name = N'toinoi', @level1type = N'TABLE', @level1name = @tbl,
             @level2type = N'COLUMN', @level2name = @col;
    FETCH NEXT FROM c INTO @tbl, @col, @txt;
END;
CLOSE c;
DEALLOCATE c;
GO

-- ---------------------------------------------------------------------------
-- Xem lại cấu trúc bảng trips bằng catalog (tương đương sp_help, nhưng chọn
-- đúng cột cần đọc).
SELECT c.column_id                                   AS stt,
       c.name                                        AS cot,
       TYPE_NAME(c.user_type_id)
         + CASE WHEN TYPE_NAME(c.user_type_id) IN (N'varchar', N'char')    THEN CONCAT(N'(', c.max_length, N')')
                WHEN TYPE_NAME(c.user_type_id) IN (N'nvarchar', N'nchar') THEN CONCAT(N'(', c.max_length / 2, N')')
                WHEN TYPE_NAME(c.user_type_id) = N'datetimeoffset'        THEN CONCAT(N'(', c.scale, N')')
                ELSE N'' END                         AS kieu,
       IIF(c.is_nullable = 1, N'NULL', N'NOT NULL')  AS null_,
       IIF(c.is_identity = 1, N'IDENTITY', COALESCE(OBJECT_NAME(c.default_object_id), N'')) AS mac_dinh
FROM sys.columns AS c
WHERE c.object_id = OBJECT_ID(N'toinoi.trips')
ORDER BY c.column_id;

-- Index của trips: một clustered (code), còn lại nonclustered.
SELECT i.name AS index_name, i.type_desc, i.is_primary_key AS la_pk, i.is_unique AS duy_nhat,
       STRING_AGG(c.name, N', ') WITHIN GROUP (ORDER BY ic.key_ordinal) AS cot_khoa
FROM sys.indexes AS i
JOIN sys.index_columns AS ic ON ic.object_id = i.object_id AND ic.index_id = i.index_id
JOIN sys.columns AS c ON c.object_id = ic.object_id AND c.column_id = ic.column_id
WHERE i.object_id = OBJECT_ID(N'toinoi.trips')
GROUP BY i.name, i.type_desc, i.is_primary_key, i.is_unique, i.index_id
ORDER BY i.index_id;
GO
