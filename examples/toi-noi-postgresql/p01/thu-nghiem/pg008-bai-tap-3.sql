-- PG-008 · bài tập 3: 07-B1 sau khi tài xế chuyển thành phố (chạy sau 00 → 02). Không thuộc lab.
SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- expect-rows: 1
UPDATE drivers SET city_id = 2 WHERE code = 8001;

-- expect-rows: 3
SELECT t.code, t.city_id AS thanh_pho_chuyen, d.code AS tai_xe, d.city_id AS thanh_pho_tai_xe
FROM trips AS t
JOIN drivers AS d ON d.id = t.driver_id
WHERE d.city_id <> t.city_id
ORDER BY t.code;

-- expect-rows: 1
SELECT t.code, t.status, t.city_id AS thanh_pho_chuyen, d.code AS tai_xe, d.city_id AS thanh_pho_tai_xe
FROM trips AS t
JOIN drivers AS d ON d.id = t.driver_id
WHERE d.city_id <> t.city_id
  AND t.status IN ('accepted', 'arriving', 'picked_up', 'in_transit')
ORDER BY t.code;
