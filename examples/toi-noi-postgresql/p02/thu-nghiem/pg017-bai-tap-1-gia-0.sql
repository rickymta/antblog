-- PG-017 · Bài tập 1: một dòng giá 0 đ làm hỏng cả lô 4. ON CONFLICT chỉ xử
-- lý xung đột của trọng tài; vi phạm CHECK vẫn là lỗi của CẢ câu lệnh. Sửa:
-- đưa luật CHECK vào bộ lọc R5 (và vào biên bản nhập) để chỉ từ chối dòng đó.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- expect-rows: 3
INSERT INTO menu_item_imports VALUES
  (4, 1, 'e1000000-0000-4000-8000-000000000101', 'BB-02', 'Bún bò tái', 45000, false, '2026-10-30 10:00+07'), -- hết hàng
  (4, 2, 'e1000000-0000-4000-8000-000000000101', 'BB-03', 'Bún bò chả', 42000, true,  '2026-10-15 07:00+07'), -- bản cũ, gửi lại lần nữa
  (4, 3, 'e1000000-0000-4000-8000-000000000101', 'NC-01', 'Nước cam',       0, true,  '2026-10-30 10:00+07'); -- lỗi file: giá 0

-- Câu UPSERT đúng quy tắc của mục 5 (R2–R5), chỉ đổi số lô: LỖI cả câu.
-- expect-error: 23514 ck_menu_items_price_positive
INSERT INTO menu_items AS m (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT DISTINCT ON (i.merchant_id, i.sku)
       i.merchant_id, i.sku, i.name, i.price, i.is_available, i.source_updated_at, i.batch_id
FROM menu_item_imports AS i
WHERE i.batch_id = 4
  AND EXISTS (SELECT 1 FROM merchants AS q WHERE q.id = i.merchant_id)
  AND NOT EXISTS (SELECT 1 FROM menu_items AS x
                  WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)
ORDER BY i.merchant_id, i.sku, i.source_updated_at DESC, i.line_no DESC
ON CONFLICT (merchant_id, sku) DO UPDATE
SET name = EXCLUDED.name, price = EXCLUDED.price, is_available = EXCLUDED.is_available,
    source_updated_at = EXCLUDED.source_updated_at, updated_by_batch = EXCLUDED.updated_by_batch
WHERE EXCLUDED.source_updated_at > m.source_updated_at
  AND (m.name, m.price, m.is_available)
      IS DISTINCT FROM (EXCLUDED.name, EXCLUDED.price, EXCLUDED.is_available);

-- Sửa: thêm luật giá > 0 vào bộ lọc. Chỉ BB-02 được cập nhật; BB-03 bị R2
-- loại, NC-01 bị bộ lọc từ chối (biên bản nhập phải báo dòng này cho quán).
-- expect-line: BB-02 | 45000 | f | 4
INSERT INTO menu_items AS m (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT DISTINCT ON (i.merchant_id, i.sku)
       i.merchant_id, i.sku, i.name, i.price, i.is_available, i.source_updated_at, i.batch_id
FROM menu_item_imports AS i
WHERE i.batch_id = 4
  AND i.price > 0                                                   -- luật của ck_menu_items_price_positive
  AND EXISTS (SELECT 1 FROM merchants AS q WHERE q.id = i.merchant_id)
  AND NOT EXISTS (SELECT 1 FROM menu_items AS x
                  WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)
ORDER BY i.merchant_id, i.sku, i.source_updated_at DESC, i.line_no DESC
ON CONFLICT (merchant_id, sku) DO UPDATE
SET name = EXCLUDED.name, price = EXCLUDED.price, is_available = EXCLUDED.is_available,
    source_updated_at = EXCLUDED.source_updated_at, updated_by_batch = EXCLUDED.updated_by_batch
WHERE EXCLUDED.source_updated_at > m.source_updated_at
  AND (m.name, m.price, m.is_available)
      IS DISTINCT FROM (EXCLUDED.name, EXCLUDED.price, EXCLUDED.is_available)
RETURNING m.sku, m.price, m.is_available, m.updated_by_batch;

-- NC-01 giữ giá cũ 25.000 đ.
-- expect-value: 25000
SELECT price FROM menu_items WHERE sku = 'NC-01';
