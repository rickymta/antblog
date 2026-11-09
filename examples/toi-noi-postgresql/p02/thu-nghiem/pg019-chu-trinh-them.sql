-- PG-019 · thí nghiệm ngoài lab trên dữ liệu có vòng (BEGIN … ROLLBACK).
-- Nền: p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql
SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

BEGIN;
-- expect-rows: 2
UPDATE areas SET parent_id = CASE id WHEN 21 THEN 221 WHEN 22 THEN 211 END WHERE id IN (21, 22);

-- 1. Đối chiếu "qua cây" với "trực tiếp" của Hà Nội: lệch ngay (đi xuống từ gốc vẫn dừng).
-- expect-line: 340 | 1094
WITH RECURSIVE closure (to_tien_id, hau_due_id) AS (
  SELECT id, id FROM areas WHERE code = 'HN'
  UNION ALL
  SELECT c.to_tien_id, a.id FROM closure AS c JOIN areas AS a ON a.parent_id = c.hau_due_id
)
SELECT (SELECT count(*) FROM closure AS c JOIN trips AS t ON t.pickup_area_id = c.hau_due_id) AS qua_cay,
       (SELECT count(*) FROM trips WHERE city_id = 2 AND pickup_area_id IS NOT NULL)          AS truc_tiep;

-- 2. Kiểm tầng (K9 của 02-kiem-tra-fixture) bắt được vòng này vì vùng nhận cha là khu.
-- expect-line: HN-TA | district | zone
-- expect-line: HN-TT | district | zone
SELECT a.code, a.level, p.level AS cap_cha
FROM areas AS a JOIN areas AS p ON p.id = a.parent_id
WHERE (a.level, p.level) NOT IN (('district', 'city'), ('zone', 'district'))
ORDER BY a.code COLLATE "C";

-- 3. UNION chỉ tự dừng khi CẢ DÒNG lặp lại. Mang thêm cột bậc thì không dòng nào
--    trùng: chỉ chốt bac < 12 làm nó dừng (13 dòng, 4 nút).
-- expect-line: 13 | 4 | 12
WITH RECURSIVE len (id, bac) AS (
  SELECT id, 0 FROM areas WHERE code = 'HN-TT-01'
  UNION
  SELECT a.parent_id, l.bac + 1 FROM len AS l JOIN areas AS a ON a.id = l.id
  WHERE a.parent_id IS NOT NULL AND l.bac < 12
)
SELECT count(*) AS so_dong, count(DISTINCT id) AS so_nut, max(bac) AS bac_lon_nhat FROM len;

-- Chỉ mang id: dòng lặp bị khử, truy vấn tự dừng sau 4 dòng.
-- expect-value: 4
WITH RECURSIVE len (id) AS (
  SELECT id FROM areas WHERE code = 'HN-TT-01'
  UNION
  SELECT a.parent_id FROM len AS l JOIN areas AS a ON a.id = l.id
  WHERE a.parent_id IS NOT NULL
)
SELECT count(*) FROM len;

-- 4. Bảng closure của 19-cay-khu-vuc (neo = MỌI nút) sẽ lặp vô hạn trên dữ liệu này.
--    Thêm CYCLE thì nó dừng và đánh dấu các đường quay vòng.
-- expect-line: 98 | 4
WITH RECURSIVE closure (to_tien_id, hau_due_id) AS (
  SELECT id, id FROM areas
  UNION ALL
  SELECT c.to_tien_id, a.id FROM closure AS c JOIN areas AS a ON a.parent_id = c.hau_due_id
) CYCLE hau_due_id SET vong USING duong_di
SELECT count(*) FILTER (WHERE NOT vong)                AS cap_to_tien_hau_due,
       count(DISTINCT to_tien_id) FILTER (WHERE vong)  AS so_nut_xuat_phat_gap_vong
FROM closure;

ROLLBACK;

-- Cây lành: closure có 3×1 + 10×2 + 21×3 = 86 cặp (tổ tiên, hậu duệ), không đường nào gặp vòng.
-- expect-line: 86 | 0
WITH RECURSIVE closure (to_tien_id, hau_due_id) AS (
  SELECT id, id FROM areas
  UNION ALL
  SELECT c.to_tien_id, a.id FROM closure AS c JOIN areas AS a ON a.parent_id = c.hau_due_id
) CYCLE hau_due_id SET vong USING duong_di
SELECT count(*) FILTER (WHERE NOT vong), count(DISTINCT to_tien_id) FILTER (WHERE vong) FROM closure;
