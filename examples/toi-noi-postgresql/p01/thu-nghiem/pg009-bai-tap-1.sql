SET search_path TO toinoi;
ALTER TABLE drivers ADD COLUMN vehicle_plate text;
UPDATE drivers SET vehicle_plate = '59X1-000.0' || (code - 8000);
SET client_min_messages = debug1;
ALTER TABLE drivers ALTER COLUMN vehicle_plate SET NOT NULL;
SET client_min_messages = notice;
SELECT column_name, is_nullable FROM information_schema.columns
WHERE table_schema = 'toinoi' AND table_name = 'drivers' AND column_name = 'vehicle_plate';
