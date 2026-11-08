-- P01 · 06-migration-rollback.sql — hoàn tác 06-migration.sql (PG-009).
--
-- KHÔNG nằm trong `node run.mjs p01` (runner bỏ qua *-rollback.sql khi chạy
-- cả thư mục). Chạy tay khi cần:
--   node run.mjs p01 p01/06-migration-rollback.sql
--
-- Hai mức, chọn mức nhẹ nhất đủ giải quyết sự cố:
--
--   Mức 1 · NỚI LỎNG (khuyến nghị): bỏ NOT NULL và CHECK định dạng, GIỮ cột và
--   dữ liệu. Dùng khi ứng dụng cũ bị lỗi vì chưa gửi biển số. Không mất dữ liệu,
--   sau này chỉ cần chạy lại B4–B6.
--
--   Mức 2 · GỠ HẲN CỘT: chỉ khi chắc chắn bỏ tính năng. DROP COLUMN làm MẤT dữ
--   liệu biển số ⇒ sao lưu trước (bảng drivers_vehicle_plate_backup bên dưới).
--   Phiên bản ứng dụng nào đã đọc/ghi vehicle_plate phải được gỡ TRƯỚC.
--
-- Mọi câu dùng IF EXISTS / IF NOT EXISTS nên chạy nhiều lần không lỗi, kể cả
-- khi migration mới chạy được một phần.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';
SET lock_timeout = '5s';

-- ---------------------------------------------------------------------------
-- Mức 1 · Nới lỏng.
ALTER TABLE drivers DROP CONSTRAINT IF EXISTS ck_drivers_vehicle_plate_not_null;
ALTER TABLE drivers DROP CONSTRAINT IF EXISTS ck_drivers_vehicle_plate_format;

-- DROP NOT NULL không có IF EXISTS cho cột; khối DO kiểm cột trước khi sửa.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM information_schema.columns
             WHERE table_schema = 'toinoi' AND table_name = 'drivers'
               AND column_name = 'vehicle_plate') THEN
    ALTER TABLE toinoi.drivers ALTER COLUMN vehicle_plate DROP NOT NULL;
  END IF;
END
$$;

-- ---------------------------------------------------------------------------
-- Mức 2 · Gỡ hẳn cột, sau khi sao lưu. (Muốn dừng ở mức 1: xoá phần dưới đây.)
CREATE TABLE IF NOT EXISTS drivers_vehicle_plate_backup (
  driver_id     uuid        NOT NULL,
  vehicle_plate text        NOT NULL,
  backed_up_at  timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_drivers_vehicle_plate_backup PRIMARY KEY (driver_id)
);

-- Sao lưu chỉ khi cột còn tồn tại (SQL tĩnh không nhắc tới cột có thể đã mất,
-- nên phải đặt trong khối DO). ON CONFLICT … DO UPDATE là UPSERT (PG-017):
-- chạy lại thì cập nhật bản sao lưu thay vì báo trùng khoá.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM information_schema.columns
             WHERE table_schema = 'toinoi' AND table_name = 'drivers'
               AND column_name = 'vehicle_plate') THEN
    INSERT INTO toinoi.drivers_vehicle_plate_backup (driver_id, vehicle_plate)
    SELECT id, vehicle_plate FROM toinoi.drivers
    WHERE vehicle_plate IS NOT NULL
    ON CONFLICT (driver_id) DO UPDATE SET vehicle_plate = EXCLUDED.vehicle_plate,
                                          backed_up_at  = EXCLUDED.backed_up_at;
  END IF;
END
$$;

ALTER TABLE drivers DROP COLUMN IF EXISTS vehicle_plate;

RESET lock_timeout;

-- ---------------------------------------------------------------------------
-- Kiểm chứng: cột đã gỡ, không còn ràng buộc biển số, bản sao lưu còn đủ.
-- expect-rows: 0
SELECT column_name
FROM information_schema.columns
WHERE table_schema = 'toinoi' AND table_name = 'drivers' AND column_name = 'vehicle_plate';

-- expect-rows: 0
SELECT conname
FROM pg_constraint
WHERE conrelid = 'drivers'::regclass AND conname LIKE 'ck_drivers_vehicle_plate%';

SELECT count(*) AS so_bien_so_da_sao_luu FROM drivers_vehicle_plate_backup;
