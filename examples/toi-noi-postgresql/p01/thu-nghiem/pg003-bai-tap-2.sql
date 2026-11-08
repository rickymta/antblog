SET search_path TO toinoi;
-- expect-error: 23505 uq_drivers_code
INSERT INTO drivers (code, full_name, city_id, vehicle_type) VALUES (8002, 'Tài Xế Mới', 1, 'bike');
-- expect-error: 23503 fk_trips_merchant
UPDATE trips SET merchant_id = 'e0000000-0000-4000-8000-000000000099' WHERE code = 100002;
-- expect-error: 23503 fk_trips_customer
DELETE FROM customers WHERE id = 'c0000000-0000-4000-8000-000000000001';
