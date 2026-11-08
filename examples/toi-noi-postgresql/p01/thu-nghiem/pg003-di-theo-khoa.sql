SET search_path TO toinoi;
SELECT code, customer_id, driver_id FROM trips WHERE code = 100004;
SELECT phone, full_name FROM customers WHERE id = 'c0000000-0000-4000-8000-000000000005';
SELECT code, full_name, city_id FROM drivers WHERE id = 'd0000000-0000-4000-8000-000000008005';
