-- P02 · 19-chu-trinh.sql — PG-019 `postgresql-recursive-cte-cay-khu-vuc` (phần 2/2)
-- Fixture CHU TRÌNH: phát hiện dữ liệu cây bị vòng.
--
-- Tình huống: bản cập nhật cây khu vực từ bộ phận vận hành bị ĐẢO cha của
-- hai vùng Hà Nội — HN-TT (21) nhận cha là khu HN-TA-01 (221), còn HN-TA (22)
-- nhận cha là khu HN-TT-01 (211). Mọi ràng buộc đều qua: FK (cha tồn tại),
-- ck_areas_root_no_parent (vùng có cha), ck_areas_not_own_parent (không tự
-- làm cha mình). CHECK chỉ nhìn MỘT dòng nên không thấy vòng
--     21 → 221 → 22 → 211 → 21.
--
-- Toàn bộ file chạy trong MỘT giao dịch và ROLLBACK ở cuối: các file sau (20)
-- vẫn thấy cây lành. KHÔNG chạy truy vấn đệ quy "trần" (không chốt dừng) trên
-- dữ liệu này — nó lặp vô hạn tới khi hết bộ nhớ/hết thời gian.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

BEGIN;

-- expect-line: 21 | HN-TT | 221
-- expect-line: 22 | HN-TA | 211
WITH doi AS (
  UPDATE areas
  SET parent_id = CASE id WHEN 21 THEN 221 WHEN 22 THEN 211 END
  WHERE id IN (21, 22)
  RETURNING id, code, parent_id
)
SELECT * FROM doi ORDER BY id;

-- ---------------------------------------------------------------------------
-- 1. Hệ quả IM LẶNG: đi xuống từ gốc HN không còn thấy hai vùng và năm khu
--    của chúng. Báo cáo cộng dồn của HN tụt từ 1.094 chuyến (19-cay-khu-vuc)
--    xuống 340 — không có lỗi nào được báo.
-- expect-line: 5 | 340
WITH RECURSIVE closure (to_tien_id, hau_due_id) AS (
  SELECT id, id FROM areas WHERE code = 'HN'
  UNION ALL
  SELECT c.to_tien_id, a.id FROM closure AS c JOIN areas AS a ON a.parent_id = c.hau_due_id
)
SELECT (SELECT count(*) FROM closure)                                        AS so_nut_duoi_hn,
       (SELECT count(*) FROM closure AS c JOIN trips AS t ON t.pickup_area_id = c.hau_due_id) AS so_chuyen_hn;

-- ---------------------------------------------------------------------------
-- 2. Đi LÊN từ khu HN-TT-01 thì không bao giờ tới gốc. Chốt dừng theo ĐỘ SÂU
--    (bac < 6) cho thấy vòng lặp lại — chốt này chỉ để an toàn, không phát
--    hiện được chu trình.
-- expect-line: 0 | HN-TT-01
-- expect-line: 1 | HN-TT
-- expect-line: 2 | HN-TA-01
-- expect-line: 3 | HN-TA
-- expect-line: 4 | HN-TT-01
-- expect-line: 5 | HN-TT
WITH RECURSIVE to_tien (id, parent_id, code, bac) AS (
  SELECT id, parent_id, code, 0 FROM areas WHERE code = 'HN-TT-01'
  UNION ALL
  SELECT a.id, a.parent_id, a.code, t.bac + 1
  FROM areas AS a JOIN to_tien AS t ON a.id = t.parent_id
  WHERE t.bac < 5
)
SELECT bac, code FROM to_tien ORDER BY bac;

-- ---------------------------------------------------------------------------
-- 3. Phát hiện chu trình bằng mệnh đề CYCLE (PostgreSQL 14+): PostgreSQL tự
--    giữ đường đi theo cột `id` và đánh dấu dòng quay lại một id đã đi qua;
--    dòng đó không được đệ quy tiếp ⇒ truy vấn tự dừng.
-- expect-line: 0 | HN-TT-01 | f | {(211)}
-- expect-line: 1 | HN-TT    | f | {(211),(21)}
-- expect-line: 2 | HN-TA-01 | f | {(211),(21),(221)}
-- expect-line: 3 | HN-TA    | f | {(211),(21),(221),(22)}
-- expect-line: 4 | HN-TT-01 | t | {(211),(21),(221),(22),(211)}
WITH RECURSIVE to_tien (id, parent_id, code, bac) AS (
  SELECT id, parent_id, code, 0 FROM areas WHERE code = 'HN-TT-01'
  UNION ALL
  SELECT a.id, a.parent_id, a.code, t.bac + 1
  FROM areas AS a JOIN to_tien AS t ON a.id = t.parent_id
) CYCLE id SET la_chu_trinh USING duong_di
SELECT bac, code, la_chu_trinh, duong_di FROM to_tien ORDER BY bac;

-- Cách tự viết tương đương (chạy được trước 14): mang theo mảng id đã đi qua
-- và chỉ đi tiếp khi nút kế tiếp CHƯA có trong mảng.
-- expect-line: 0 | HN-TT-01 | {211}            | f
-- expect-line: 1 | HN-TT    | {211,21}         | f
-- expect-line: 2 | HN-TA-01 | {211,21,221}     | f
-- expect-line: 3 | HN-TA    | {211,21,221,22}  | f
-- expect-line: 4 | HN-TT-01 | {211,21,221,22,211} | t
WITH RECURSIVE to_tien (id, parent_id, code, bac, da_qua, lap) AS (
  SELECT id, parent_id, code, 0, ARRAY[id], false FROM areas WHERE code = 'HN-TT-01'
  UNION ALL
  SELECT a.id, a.parent_id, a.code, t.bac + 1, t.da_qua || a.id, a.id = ANY (t.da_qua)
  FROM areas AS a JOIN to_tien AS t ON a.id = t.parent_id
  WHERE NOT t.lap
)
SELECT bac, code, da_qua, lap FROM to_tien ORDER BY bac;

-- ---------------------------------------------------------------------------
-- 4. Truy vấn KIỂM TRA cho cả bảng (chạy định kỳ / trước khi nhận bản cập nhật):
--    a) nút KHÔNG đi tới được từ gốc nào — gồm nút nằm trên vòng và mọi hậu duệ;
-- expect-line: 21  | HN-TT
-- expect-line: 22  | HN-TA
-- expect-line: 211 | HN-TT-01
-- expect-line: 212 | HN-TT-02
-- expect-line: 213 | HN-TT-03
-- expect-line: 221 | HN-TA-01
-- expect-line: 222 | HN-TA-02
WITH RECURSIVE tu_goc (id) AS (
  SELECT id FROM areas WHERE parent_id IS NULL
  UNION                                             -- UNION (khử trùng) cũng tự dừng trên vòng
  SELECT a.id FROM areas AS a JOIN tu_goc AS g ON a.parent_id = g.id
)
SELECT a.id, a.code
FROM areas AS a
WHERE NOT EXISTS (SELECT 1 FROM tu_goc AS g WHERE g.id = a.id)
ORDER BY a.id;

--    b) nút NẰM TRÊN vòng: đi lên từ chính nó và quay lại chính nó.
-- expect-line: 21  | HN-TT    | {21,221,22,211}
-- expect-line: 22  | HN-TA    | {22,211,21,221}
-- expect-line: 211 | HN-TT-01 | {211,21,221,22}
-- expect-line: 221 | HN-TA-01 | {221,22,211,21}
WITH RECURSIVE len (xuat_phat, id, parent_id, da_qua) AS (
  SELECT id, id, parent_id, ARRAY[id] FROM areas
  UNION ALL
  SELECT l.xuat_phat, a.id, a.parent_id, l.da_qua || a.id
  FROM len AS l JOIN areas AS a ON a.id = l.parent_id
  WHERE NOT a.id = ANY (l.da_qua)                   -- dừng trước khi lặp
)
SELECT n.id, n.code, l.da_qua AS vong
FROM len AS l
JOIN areas AS n ON n.id = l.xuat_phat
WHERE l.parent_id = l.xuat_phat                     -- bước kế tiếp sẽ quay về điểm xuất phát
ORDER BY n.id;

ROLLBACK;

-- Sau ROLLBACK cây lành lại: mọi nút đi tới được từ gốc.
-- expect-value: 0
WITH RECURSIVE tu_goc (id) AS (
  SELECT id FROM areas WHERE parent_id IS NULL
  UNION
  SELECT a.id FROM areas AS a JOIN tu_goc AS g ON a.parent_id = g.id
)
SELECT count(*) FROM areas AS a WHERE NOT EXISTS (SELECT 1 FROM tu_goc AS g WHERE g.id = a.id);
