-- PG-017 · Lỗi thường gặp: ON CONFLICT DO NOTHING KHÔNG ghi conflict_target
-- nuốt lặng lẽ cả những xung đột ta không định bỏ qua (ở đây: trùng tên món).
-- Chạy sau toàn bộ 17-upsert-on-conflict.sql (bảng menu_items có 7 món).

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Dòng 7 của lô 2: sku mới BB-05 nhưng trùng TÊN với NC-01 "Nước cam".
-- Có conflict_target (merchant_id, sku): xung đột tên KHÔNG phải trọng tài
-- (arbiter) ⇒ lỗi nhìn thấy được, cả câu bị huỷ.
-- expect-error: 23505 uq_menu_items_name
INSERT INTO menu_items (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT merchant_id, sku, name, price, is_available, source_updated_at, batch_id
FROM menu_item_imports
WHERE batch_id = 2 AND line_no = 7
ON CONFLICT (merchant_id, sku) DO NOTHING;

-- Bỏ conflict_target: mọi ràng buộc UNIQUE/khoá chính đều thành trọng tài ⇒
-- dòng trùng tên bị bỏ qua LẶNG LẼ, thẻ lệnh INSERT 0 0, không ai biết.
-- expect-rows: 0
INSERT INTO menu_items (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT merchant_id, sku, name, price, is_available, source_updated_at, batch_id
FROM menu_item_imports
WHERE batch_id = 2 AND line_no = 7
ON CONFLICT DO NOTHING;

-- expect-value: 0
SELECT count(*) FROM menu_items WHERE sku = 'BB-05';

-- Khoá ngoại không bao giờ là trọng tài: dòng 8 (quán không tồn tại) vẫn làm
-- hỏng cả câu, kể cả với DO NOTHING không target.
-- expect-error: 23503 fk_menu_items_merchant
INSERT INTO menu_items (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT merchant_id, sku, name, price, is_available, source_updated_at, batch_id
FROM menu_item_imports
WHERE batch_id = 2 AND line_no = 8
ON CONFLICT DO NOTHING;

-- DO UPDATE thì BẮT BUỘC có conflict_target (hoặc ON CONSTRAINT).
-- expect-error: 42601
INSERT INTO menu_items (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT merchant_id, sku, name, price, is_available, source_updated_at, batch_id
FROM menu_item_imports
WHERE batch_id = 2 AND line_no = 7
ON CONFLICT DO UPDATE SET price = EXCLUDED.price;

-- Bảng đích không đổi: vẫn 7 món.
-- expect-value: 7
SELECT count(*) FROM menu_items;
