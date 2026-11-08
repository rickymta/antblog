-- P01 · 06-migration.sql — thay đổi schema mà dữ liệu cũ và ứng dụng cũ vẫn chạy
-- (PG-009). Hoàn tác: 06-migration-rollback.sql.
--
-- Yêu cầu nghiệp vụ: lưu BIỂN SỐ XE của tài xế, định dạng chuẩn, cuối cùng
-- phải bắt buộc. Biển số của tài xế hiện có nằm ở một file do bộ phận tuyển
-- dụng gửi — giá trị phụ thuộc từng dòng, không phải một hằng số chung.
--
-- Không làm một phát `ADD COLUMN vehicle_plate text NOT NULL`: bảng đã có dữ
-- liệu nên câu đó lỗi ngay; và ứng dụng cũ (chưa biết cột mới) sẽ không thêm
-- được tài xế. Thay vào đó là chuỗi bước "mở rộng → điền dữ liệu → siết lại"
-- (expand → backfill → contract), mỗi bước giữ hệ thống chạy được:
--
--   B1  ADD COLUMN cho phép NULL           (ứng dụng cũ không bị ảnh hưởng)
--   B2  ứng dụng cũ vẫn ghi được            (kiểm chứng)
--   B3  backfill từ dữ liệu nguồn           (UPDATE … FROM)
--   B4  CHECK định dạng NOT VALID           (chặn dữ liệu MỚI sai ngay)
--   B5  VALIDATE CONSTRAINT                 (rà dữ liệu CŨ; phát hiện dòng sai)
--   B6  CHECK (… IS NOT NULL) NOT VALID → VALIDATE → SET NOT NULL → bỏ CHECK phụ
--       — chỉ làm khi MỌI phiên bản ứng dụng đã ghi biển số.
--
-- Trên server thật nhiều người dùng: đặt lock_timeout để câu ALTER không đứng
-- chờ khoá quá lâu và chặn các truy vấn xếp hàng sau nó; backfill bảng lớn chia
-- lô (vd. theo khoảng `code`). Lab chỉ có vài dòng nên chạy một lần.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';
SET lock_timeout = '5s';

-- ---------------------------------------------------------------------------
-- B0. Kiểm tra trước khi đổi: có bao nhiêu tài xế cần biển số.
SELECT count(*) AS so_tai_xe FROM drivers;

-- ---------------------------------------------------------------------------
-- B1. Thêm cột cho phép NULL, không DEFAULT: chỉ sửa catalog, không viết lại
--     bảng. Mọi dòng cũ đọc ra NULL.
ALTER TABLE drivers ADD COLUMN vehicle_plate text;

COMMENT ON COLUMN drivers.vehicle_plate IS 'Biển số xe dạng chuẩn, vd. 59X1-123.45 (xe máy) hoặc 30F-678.90 (ô tô)';

-- Truy vấn của ứng dụng cũ (liệt kê cột tường minh) vẫn chạy y như trước.
-- `SELECT *` thì nhận thêm một cột — lý do ứng dụng không nên phụ thuộc SELECT *.
SELECT code, full_name, vehicle_type, status
FROM drivers
ORDER BY code;

-- ---------------------------------------------------------------------------
-- B2. Ứng dụng cũ đăng ký tài xế mới mà không biết cột biển số: vẫn thành công.
--     (Viết dạng INSERT … SELECT … WHERE NOT EXISTS để chạy lại file không lỗi.)
INSERT INTO drivers (id, code, full_name, city_id, vehicle_type, status, created_at)
SELECT 'd0000000-0000-4000-8000-000000008007', 8007, 'Mai Xuân Quý', 1, 'bike', 'offline',
       '2026-11-03 10:00:00+07'
WHERE NOT EXISTS (SELECT 1 FROM drivers WHERE code = 8007);

-- ---------------------------------------------------------------------------
-- B3. Nạp file biển số vào bảng tạm (TEMP: chỉ sống trong phiên này), rồi điền
--     vào drivers. File nguồn có MỘT dòng sai định dạng (tài xế 8006, chữ
--     thường, thiếu gạch) — để thấy bước B5 bắt được nó.
CREATE TEMP TABLE driver_plate_imports (
  driver_code integer PRIMARY KEY,
  plate       text NOT NULL
);

-- expect-rows: 7
INSERT INTO driver_plate_imports (driver_code, plate) VALUES
  (8001, '59X1-123.45'),
  (8002, '59X2-234.56'),
  (8003, '29B1-345.67'),
  (8004, '30F-456.78'),
  (8005, '43C1-567.89'),
  (8006, '43a 67890'),
  (8007, '59X3-789.01');

-- Chỉ điền dòng còn NULL: chạy lại không ghi đè giá trị đã sửa tay.
-- expect-rows: 7
UPDATE drivers AS d
SET vehicle_plate = i.plate
FROM driver_plate_imports AS i
WHERE i.driver_code = d.code
  AND d.vehicle_plate IS NULL;

-- Kiểm chứng backfill: không còn tài xế thiếu biển số.
-- expect-rows: 0
SELECT code, full_name FROM drivers WHERE vehicle_plate IS NULL;

-- ---------------------------------------------------------------------------
-- B4. Thêm CHECK định dạng với NOT VALID: KHÔNG quét dữ liệu cũ (nhanh, khoá
--     ngắn), nhưng mọi INSERT/UPDATE từ giờ phải đúng định dạng.
--     NULL vẫn qua được CHECK, nên ứng dụng cũ (không ghi biển số) chưa bị chặn.
--     Định dạng lab: 2 số tỉnh + seri (1 chữ in hoa, có thể thêm 1 chữ/số)
--     + '-' + 3 số + '.' + 2 số. Không phải quy chuẩn biển số đầy đủ.
ALTER TABLE drivers
  ADD CONSTRAINT ck_drivers_vehicle_plate_format
  CHECK (vehicle_plate ~ '^[0-9]{2}[A-Z][A-Z0-9]?-[0-9]{3}\.[0-9]{2}$') NOT VALID;

-- Dữ liệu MỚI sai định dạng bị chặn ngay…
-- expect-error: 23514 ck_drivers_vehicle_plate_format
UPDATE drivers SET vehicle_plate = '59x1 12345' WHERE code = 8001;

-- …trong khi dòng CŨ sai (8006) vẫn nằm đó, chưa ai phát hiện.
-- expect-rows: 1
SELECT code, vehicle_plate
FROM drivers
WHERE vehicle_plate !~ '^[0-9]{2}[A-Z][A-Z0-9]?-[0-9]{3}\.[0-9]{2}$';

-- ---------------------------------------------------------------------------
-- B5. VALIDATE quét toàn bảng với khoá nhẹ hơn (SHARE UPDATE EXCLUSIVE: đọc
--     và ghi dữ liệu vẫn chạy song song). Dòng 8006 làm nó thất bại.
-- expect-error: 23514 ck_drivers_vehicle_plate_format
ALTER TABLE drivers VALIDATE CONSTRAINT ck_drivers_vehicle_plate_format;

-- Sửa dữ liệu gốc (sau khi hỏi lại bộ phận tuyển dụng), rồi VALIDATE lại.
-- expect-rows: 1
UPDATE drivers SET vehicle_plate = '43A-678.90'
WHERE code = 8006
RETURNING code, vehicle_plate;

ALTER TABLE drivers VALIDATE CONSTRAINT ck_drivers_vehicle_plate_format;

-- ---------------------------------------------------------------------------
-- B6. Siết NOT NULL — bước "contract", phá vỡ ứng dụng cũ không ghi biển số.
--     SET NOT NULL tự nó phải quét toàn bảng dưới khoá ACCESS EXCLUSIVE. Có
--     sẵn một CHECK (… IS NOT NULL) ĐÃ VALIDATE thì PostgreSQL (từ bản 12)
--     dùng nó làm bằng chứng và bỏ qua lần quét đó. Bật thông báo DEBUG1 để
--     thấy chính server xác nhận điều này.
ALTER TABLE drivers
  ADD CONSTRAINT ck_drivers_vehicle_plate_not_null
  CHECK (vehicle_plate IS NOT NULL) NOT VALID;

ALTER TABLE drivers VALIDATE CONSTRAINT ck_drivers_vehicle_plate_not_null;

SET client_min_messages = debug1;

ALTER TABLE drivers ALTER COLUMN vehicle_plate SET NOT NULL;

SET client_min_messages = notice;

-- CHECK phụ đã xong việc; NOT NULL thay thế nó.
ALTER TABLE drivers DROP CONSTRAINT ck_drivers_vehicle_plate_not_null;

-- Từ giờ ứng dụng cũ (không gửi biển số) bị từ chối — đúng như dự tính.
-- expect-error: 23502 vehicle_plate
INSERT INTO drivers (id, code, full_name, city_id, vehicle_type, status, created_at)
VALUES ('f0000000-0000-4000-8000-000000000030', 8030, 'Ứng Dụng Cũ', 1, 'bike', 'offline',
        '2026-11-03 11:00:00+07');

-- ---------------------------------------------------------------------------
-- Kiểm chứng sau migration.
-- expect-rows: 7
SELECT code, full_name, vehicle_type, vehicle_plate
FROM drivers
ORDER BY code;

-- Trạng thái cột và các ràng buộc liên quan (convalidated = đã rà dữ liệu cũ).
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_schema = 'toinoi' AND table_name = 'drivers' AND column_name = 'vehicle_plate';

-- expect-rows: 1
SELECT conname, convalidated, pg_get_constraintdef(oid) AS dinh_nghia
FROM pg_constraint
WHERE conrelid = 'drivers'::regclass AND conname LIKE 'ck_drivers_vehicle_plate%'
ORDER BY conname;

DROP TABLE driver_plate_imports;

RESET lock_timeout;
