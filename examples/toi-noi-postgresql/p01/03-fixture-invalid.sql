-- P01 · 03-fixture-invalid.sql — dữ liệu SAI, mỗi câu vi phạm đúng MỘT quy tắc
-- (PG-004, PG-005, PG-010).
--
-- Mỗi câu INSERT/UPDATE có dòng `-- expect-error: <SQLSTATE> <tên>` ngay phía
-- trên: runner coi lỗi khớp là ĐẠT; nếu câu lệnh lại chạy được thì KHÔNG ĐẠT
-- (nghĩa là schema đang thiếu một quy tắc). psql thật bỏ qua các dòng này như
-- chú thích bình thường — khi chạy bằng psql, KHÔNG bật ON_ERROR_STOP cho file này.
--
-- SQLSTATE hay gặp:
--   23502 not_null_violation      23503 foreign_key_violation
--   23505 unique_violation        23514 check_violation
--   22P02 invalid_text_representation (sai định dạng số, uuid…)
--   22007 invalid_datetime_format 22008 datetime_field_overflow
--   22003 numeric_value_out_of_range
--   428C9 generated_always (ghi tay vào cột IDENTITY GENERATED ALWAYS)
--
-- Các câu lỗi ghi tay mọi cột (kể cả id, thời gian) để phần DETAIL của thông
-- báo lỗi giống nhau ở mọi lần chạy. UUID dạng f… = dòng dùng để thử lỗi.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- Đối chứng: dòng chuyến "gốc" dưới đây HỢP LỆ. Các câu thử lỗi về chuyến chỉ
-- đổi MỘT giá trị so với dòng gốc, nên lỗi chắc chắn do giá trị bị đổi.
-- Chạy trong giao dịch rồi ROLLBACK để không để lại dữ liệu (P04 nói kỹ về giao dịch).
BEGIN;

-- expect-rows: 1
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000000', 'ride', 1, 'c0000000-0000-4000-8000-000000000001', NULL, NULL,
        'requested', 40000, '2026-11-03 09:00:00+07', NULL, NULL, '2026-11-03 09:00:00+07')
RETURNING code, kind, status, fare;

ROLLBACK;

-- ---------------------------------------------------------------------------
-- NOT NULL: thiếu giá trị bắt buộc. PG 17 không đặt tên cho ràng buộc NOT NULL,
-- nên thông báo lỗi chỉ ra TÊN CỘT (và runner so khớp theo tên cột).

-- Khách phải có họ tên.
-- expect-error: 23502 full_name
INSERT INTO customers (id, phone, created_at)
VALUES ('f0000000-0000-4000-8000-000000000001', '0900000011', '2026-11-03 09:00:00+07');

-- Chuyến phải thuộc một thành phố.
-- expect-error: 23502 city_id
INSERT INTO trips (id, kind, customer_id, status, fare, requested_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000002', 'ride', 'c0000000-0000-4000-8000-000000000001',
        'requested', 40000, '2026-11-03 09:00:00+07', '2026-11-03 09:00:00+07');

-- Ghi NULL tường minh cũng bị chặn, kể cả khi cột có DEFAULT.
-- expect-error: 23502 status
INSERT INTO trips (id, kind, city_id, customer_id, status, fare, requested_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000003', 'ride', 1, 'c0000000-0000-4000-8000-000000000001',
        NULL, 40000, '2026-11-03 09:00:00+07', '2026-11-03 09:00:00+07');

-- ---------------------------------------------------------------------------
-- UNIQUE: hai dòng không được trùng khoá nghiệp vụ.

-- Số điện thoại 0900000001 đã thuộc khách Nguyễn Văn An.
-- expect-error: 23505 uq_customers_phone
INSERT INTO customers (id, phone, full_name, created_at)
VALUES ('f0000000-0000-4000-8000-000000000004', '0900000001', 'Khách Trùng Số', '2026-11-03 09:00:00+07');

-- Mã tài xế 8001 đã được cấp.
-- expect-error: 23505 uq_drivers_code
INSERT INTO drivers (id, code, full_name, city_id, vehicle_type, status, created_at)
VALUES ('f0000000-0000-4000-8000-000000000005', 8001, 'Tài Xế Trùng Mã', 1, 'bike', 'offline', '2026-11-03 09:00:00+07');

-- Mã thành phố là duy nhất.
-- expect-error: 23505 uq_cities_code
INSERT INTO cities (id, code, name) VALUES (9, 'SG', 'Sài Gòn (trùng mã)');

-- PRIMARY KEY = NOT NULL + UNIQUE: trùng id cũng bị chặn.
-- expect-error: 23505 pk_cities
INSERT INTO cities (id, code, name) VALUES (1, 'XX', 'Trùng id 1');

-- ---------------------------------------------------------------------------
-- FOREIGN KEY: tham chiếu tới dòng không tồn tại.

-- Không có thành phố id 9.
-- expect-error: 23503 fk_drivers_city
INSERT INTO drivers (id, code, full_name, city_id, vehicle_type, status, created_at)
VALUES ('f0000000-0000-4000-8000-000000000006', 8099, 'Tài Xế Lạc Đường', 9, 'bike', 'offline', '2026-11-03 09:00:00+07');

-- Không có khách …0099.
-- expect-error: 23503 fk_trips_customer
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000007', 'ride', 1, 'c0000000-0000-4000-8000-000000000099', NULL, NULL,
        'requested', 40000, '2026-11-03 09:00:00+07', NULL, NULL, '2026-11-03 09:00:00+07');

-- Không xoá được thành phố còn tài xế, quán, chuyến tham chiếu (mặc định NO ACTION).
-- expect-error: 23503 fk_drivers_city
DELETE FROM cities WHERE id = 3;

-- ---------------------------------------------------------------------------
-- CHECK: quy tắc nghiệp vụ trên một dòng.

-- Số điện thoại phải ở dạng chuẩn 0xxxxxxxxx; '+84…' bị từ chối để UNIQUE có nghĩa.
-- expect-error: 23514 ck_customers_phone_format
INSERT INTO customers (id, phone, full_name, created_at)
VALUES ('f0000000-0000-4000-8000-000000000008', '+84900000001', 'Nguyễn Văn An', '2026-11-03 09:00:00+07');

-- Họ tên toàn khoảng trắng: NOT NULL không chặn được, CHECK chặn được.
-- expect-error: 23514 ck_customers_full_name_not_blank
INSERT INTO customers (id, phone, full_name, created_at)
VALUES ('f0000000-0000-4000-8000-000000000009', '0900000019', '   ', '2026-11-03 09:00:00+07');

-- Mã thành phố phải là 2 chữ in hoa.
-- expect-error: 23514 ck_cities_code_format
INSERT INTO cities (id, code, name) VALUES (4, 'hp', 'Hải Phòng');

-- Loại xe chỉ có bike / car.
-- expect-error: 23514 ck_drivers_vehicle_type
INSERT INTO drivers (id, code, full_name, city_id, vehicle_type, status, created_at)
VALUES ('f0000000-0000-4000-8000-000000000010', 8010, 'Tài Xế Xe Tải', 1, 'truck', 'offline', '2026-11-03 09:00:00+07');

-- Trạng thái tài xế chỉ có offline / online / busy.
-- expect-error: 23514 ck_drivers_status
UPDATE drivers SET status = 'sleeping' WHERE code = 8006;

-- Loại chuyến chỉ có ride / food / parcel.
-- expect-error: 23514 ck_trips_kind
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000011', 'taxi', 1, 'c0000000-0000-4000-8000-000000000001', NULL, NULL,
        'requested', 40000, '2026-11-03 09:00:00+07', NULL, NULL, '2026-11-03 09:00:00+07');

-- Trạng thái chuyến phải thuộc 8 trạng thái của vòng đời.
-- expect-error: 23514 ck_trips_status
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000012', 'ride', 1, 'c0000000-0000-4000-8000-000000000001', NULL, NULL,
        'done', 40000, '2026-11-03 09:00:00+07', NULL, NULL, '2026-11-03 09:00:00+07');

-- Cước không được âm.
-- expect-error: 23514 ck_trips_fare_non_negative
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000013', 'ride', 1, 'c0000000-0000-4000-8000-000000000001', NULL, NULL,
        'requested', -1000, '2026-11-03 09:00:00+07', NULL, NULL, '2026-11-03 09:00:00+07');

-- Đơn đồ ăn phải có quán.
-- expect-error: 23514 ck_trips_merchant_only_food
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000014', 'food', 1, 'c0000000-0000-4000-8000-000000000001', NULL, NULL,
        'requested', 40000, '2026-11-03 09:00:00+07', NULL, NULL, '2026-11-03 09:00:00+07');

-- Chuyến đặt xe không gắn quán.
-- expect-error: 23514 ck_trips_merchant_only_food
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000015', 'ride', 1, 'c0000000-0000-4000-8000-000000000001', NULL,
        'e0000000-0000-4000-8000-000000000001',
        'requested', 40000, '2026-11-03 09:00:00+07', NULL, NULL, '2026-11-03 09:00:00+07');

-- Chuyến đang tìm tài xế không thể đã có tài xế.
-- expect-error: 23514 ck_trips_unassigned_no_driver
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000016', 'ride', 1, 'c0000000-0000-4000-8000-000000000001',
        'd0000000-0000-4000-8000-000000008003', NULL,
        'searching', 40000, '2026-11-03 09:00:00+07', '2026-11-03 09:01:00+07', NULL, '2026-11-03 09:01:00+07');

-- Chuyến hoàn thành phải có tài xế.
-- expect-error: 23514 ck_trips_assigned_has_driver
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000017', 'ride', 1, 'c0000000-0000-4000-8000-000000000001', NULL, NULL,
        'completed', 40000, '2026-11-03 09:00:00+07', NULL, '2026-11-03 09:30:00+07', '2026-11-03 09:30:00+07');

-- Có tài xế thì phải có mốc nhận chuyến.
-- expect-error: 23514 ck_trips_driver_accepted_together
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000018', 'ride', 1, 'c0000000-0000-4000-8000-000000000001',
        'd0000000-0000-4000-8000-000000008003', NULL,
        'accepted', 40000, '2026-11-03 09:00:00+07', NULL, NULL, '2026-11-03 09:01:00+07');

-- Trạng thái completed phải có completed_at (chuyến 100012 đang `accepted`,
-- đổi trạng thái mà quên ghi mốc hoàn thành).
-- expect-error: 23514 ck_trips_completed_at_by_status
UPDATE trips SET status = 'completed' WHERE code = 100012;

-- Không thể nhận chuyến trước khi khách đặt.
-- expect-error: 23514 ck_trips_accepted_after_requested
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000019', 'ride', 1, 'c0000000-0000-4000-8000-000000000001',
        'd0000000-0000-4000-8000-000000008003', NULL,
        'accepted', 40000, '2026-11-03 09:00:00+07', '2026-11-03 08:59:00+07', NULL, '2026-11-03 09:00:00+07');

-- Không thể hoàn thành trước khi được nhận.
-- expect-error: 23514 ck_trips_completed_after_accepted
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000020', 'ride', 1, 'c0000000-0000-4000-8000-000000000001',
        'd0000000-0000-4000-8000-000000008003', NULL,
        'completed', 40000, '2026-11-03 09:00:00+07', '2026-11-03 09:10:00+07', '2026-11-03 09:05:00+07',
        '2026-11-03 09:10:00+07');

-- Lịch sử chỉ nhận 8 trạng thái hợp lệ ('paid' là chuyện của thanh toán, không phải chuyến).
-- expect-error: 23514 ck_trip_events_status
INSERT INTO trip_events (trip_id, status, occurred_at)
VALUES ('70000000-0000-4000-8000-000000100000', 'paid', '2026-11-03 09:00:00+07');

-- ---------------------------------------------------------------------------
-- IDENTITY GENERATED ALWAYS: mã chuyến do database cấp, không ghi tay.
-- expect-error: 428C9
INSERT INTO trips (id, code, kind, city_id, customer_id, status, fare, requested_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000021', 1, 'ride', 1, 'c0000000-0000-4000-8000-000000000001',
        'requested', 40000, '2026-11-03 09:00:00+07', '2026-11-03 09:00:00+07');

-- ---------------------------------------------------------------------------
-- Sai kiểu / sai định dạng: lỗi xảy ra ngay khi đọc giá trị, trước mọi ràng buộc.

-- Cước phải là số nguyên đồng, không kèm dấu chấm nghìn hay ký hiệu tiền.
-- expect-error: 22P02
INSERT INTO trips (id, kind, city_id, customer_id, status, fare, requested_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000022', 'ride', 1, 'c0000000-0000-4000-8000-000000000001',
        'requested', '45.000đ', '2026-11-03 09:00:00+07', '2026-11-03 09:00:00+07');

-- `integer` tối đa 2.147.483.647; 3 tỷ đồng cho một chuyến là vượt miền giá trị.
-- expect-error: 22003
INSERT INTO trips (id, kind, city_id, customer_id, status, fare, requested_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000023', 'ride', 1, 'c0000000-0000-4000-8000-000000000001',
        'requested', 3000000000, '2026-11-03 09:00:00+07', '2026-11-03 09:00:00+07');

-- UUID sai định dạng.
-- expect-error: 22P02
INSERT INTO trips (id, kind, city_id, customer_id, status, fare, requested_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000024', 'ride', 1, 'khach-so-1',
        'requested', 40000, '2026-11-03 09:00:00+07', '2026-11-03 09:00:00+07');

-- Ngày không tồn tại (30/02).
-- expect-error: 22008
INSERT INTO trips (id, kind, city_id, customer_id, status, fare, requested_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000025', 'ride', 1, 'c0000000-0000-4000-8000-000000000001',
        'requested', 40000, '2026-02-30 09:00:00+07', '2026-11-03 09:00:00+07');

-- Chuỗi không phải thời điểm.
-- expect-error: 22007
INSERT INTO trips (id, kind, city_id, customer_id, status, fare, requested_at, updated_at)
VALUES ('f0000000-0000-4000-8000-000000000026', 'ride', 1, 'c0000000-0000-4000-8000-000000000001',
        'requested', 40000, 'hôm qua', '2026-11-03 09:00:00+07');

-- ---------------------------------------------------------------------------
-- Cái bẫy KHÔNG báo lỗi: với DateStyle mặc định 'ISO, MDY', chuỗi '03/11/2026'
-- được hiểu là THÁNG 3 ngày 11, không phải 3/11 như cách viết ở Việt Nam.
-- Luôn ghi ngày giờ theo ISO 8601: '2026-11-03 09:00:00+07'.
SELECT current_setting('DateStyle') AS datestyle,
       '03/11/2026'::date            AS chuoi_03_11_2026,
       DATE '2026-11-03'             AS iso_2026_11_03;

-- Sau toàn bộ các câu lỗi, dữ liệu hợp lệ không đổi.
-- expect-rows: 1
SELECT (SELECT count(*) FROM cities)    AS cities,
       (SELECT count(*) FROM customers) AS customers,
       (SELECT count(*) FROM drivers)   AS drivers,
       (SELECT count(*) FROM trips)     AS trips;
