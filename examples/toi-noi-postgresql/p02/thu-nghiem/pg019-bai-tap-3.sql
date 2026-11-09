-- PG-019 · Bài tập 3: với cây CỐ ĐỊNH 3 tầng, chặn chu trình ngay ở schema bằng
-- FK hai cột (parent_id, parent_level) + CHECK thứ tự tầng. Đổi schema thật trong database của lần chạy (mỗi lần chạy runner là database mới).
-- Nền: p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql
SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

ALTER TABLE areas ADD COLUMN parent_level text;
-- expect-rows: 31
UPDATE areas AS a SET parent_level = p.level FROM areas AS p WHERE p.id = a.parent_id;
ALTER TABLE areas ADD CONSTRAINT uq_areas_id_level UNIQUE (id, level);
ALTER TABLE areas ADD CONSTRAINT fk_areas_parent_level
  FOREIGN KEY (parent_id, parent_level) REFERENCES areas (id, level);
ALTER TABLE areas ADD CONSTRAINT ck_areas_level_order CHECK (
     (level = 'city'     AND parent_level IS NULL)
  OR (level = 'district' AND parent_level = 'city')
  OR (level = 'zone'     AND parent_level = 'district'));

-- Bản cập nhật đảo cha như 19-chu-trinh: chỉ đổi parent_id ⇒ (221, 'city') không tồn tại.
-- expect-error: 23503 fk_areas_parent_level
UPDATE areas SET parent_id = 221 WHERE id = 21;

-- Sửa cả parent_level cho "khớp" ⇒ vùng có cha là khu, trái thứ tự tầng.
-- expect-error: 23514 ck_areas_level_order
UPDATE areas SET parent_id = 221, parent_level = 'zone' WHERE id = 21;

-- Chuyển một khu sang vùng khác (cùng tầng) vẫn được.
-- expect-rows: 1
UPDATE areas SET parent_id = 22 WHERE code = 'HN-TT-03';
