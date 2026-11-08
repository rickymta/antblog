SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- D1. Bảng nháp không đặt tên ràng buộc: PostgreSQL tự đặt.
CREATE TABLE nhap_khong_ten (
  id    integer PRIMARY KEY,
  code  bigint  UNIQUE,
  kind  text    NOT NULL CHECK (kind IN ('ride', 'food', 'parcel')),
  fare  integer NOT NULL CHECK (fare >= 0)
);

SELECT conname, contype
FROM pg_constraint
WHERE conrelid = 'nhap_khong_ten'::regclass
ORDER BY conname;

-- expect-error: 23514 nhap_khong_ten_fare_check
INSERT INTO nhap_khong_ten (id, code, kind, fare) VALUES (1, 100, 'ride', -5);

DROP TABLE nhap_khong_ten;

-- D2. Các ràng buộc của trips và định nghĩa của chúng.
SELECT conname, contype
FROM pg_constraint
WHERE conrelid = 'trips'::regclass
ORDER BY contype, conname;

-- D3. Một dòng vi phạm HAI CHECK (kind lạ và cước âm): chỉ một tên được báo.
-- expect-error: 23514
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000099', 'taxi', 1, 'c0000000-0000-4000-8000-000000000001', NULL, NULL,
        'requested', -1000, '2026-11-03 09:00:00+07', NULL, NULL, '2026-11-03 09:00:00+07');

-- D4. So sánh với NULL ra NULL, không ra false.
SELECT NULL::timestamptz >= TIMESTAMPTZ '2026-11-03 09:00:00+07' AS ket_qua_so_sanh,
       (NULL::timestamptz >= TIMESTAMPTZ '2026-11-03 09:00:00+07') IS NULL AS la_null;
