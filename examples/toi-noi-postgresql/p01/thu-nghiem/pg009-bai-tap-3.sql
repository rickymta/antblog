SET search_path TO toinoi;
ALTER TABLE drivers ADD COLUMN vehicle_plate text;
UPDATE drivers AS d
SET vehicle_plate = b.vehicle_plate
FROM drivers_vehicle_plate_backup AS b
WHERE b.driver_id = d.id;
SELECT code, vehicle_plate FROM drivers ORDER BY code;
