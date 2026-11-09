-- PG-017 · Bài tập 2: bỏ R3 (chỉ giữ R2 trong WHERE của DO UPDATE) thì lô 2
-- ghi những dòng nào? Dựng lại trạng thái sau lô 1 trong một giao dịch rồi
-- ROLLBACK, để không ảnh hưởng tệp khác.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

BEGIN;

-- expect-rows: 7
DELETE FROM menu_items;

-- Trạng thái sau lô 1.
-- expect-rows: 5
INSERT INTO menu_items (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT merchant_id, sku, name, price, is_available, source_updated_at, batch_id
FROM menu_item_imports
WHERE batch_id = 1;

-- Lô 2, chỉ R2: BB-02 không đổi gì vẫn bị GHI (giờ nguồn mới hơn) ⇒ 4 dòng
-- được ghi (đặt trong WITH để sắp RETURNING; thẻ lệnh ngoài cùng là SELECT 4).
-- expect-line: BB-01 | 59000 | t | 2
-- expect-line: BB-02 | 45000 | t | 2
-- expect-line: BB-04 | 52000 | t | 2
-- expect-line: TR-01 |  5000 | f | 2
WITH ghi AS (
INSERT INTO menu_items AS m (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT DISTINCT ON (i.merchant_id, i.sku)
       i.merchant_id, i.sku, i.name, i.price, i.is_available, i.source_updated_at, i.batch_id
FROM menu_item_imports AS i
WHERE i.batch_id = 2
  AND EXISTS (SELECT 1 FROM merchants AS q WHERE q.id = i.merchant_id)
  AND NOT EXISTS (SELECT 1 FROM menu_items AS x
                  WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)
ORDER BY i.merchant_id, i.sku, i.source_updated_at DESC, i.line_no DESC
ON CONFLICT (merchant_id, sku) DO UPDATE
SET name = EXCLUDED.name, price = EXCLUDED.price, is_available = EXCLUDED.is_available,
    source_updated_at = EXCLUDED.source_updated_at, updated_by_batch = EXCLUDED.updated_by_batch
WHERE EXCLUDED.source_updated_at > m.source_updated_at                  -- chỉ R2
RETURNING m.sku, m.price, m.is_available, m.updated_by_batch
)
SELECT * FROM ghi ORDER BY sku COLLATE "C";

-- Gửi lại lô 2: R2 dùng `>` nên giờ nguồn bằng nhau không ghi ⇒ vẫn idempotent.
-- expect-rows: 0
INSERT INTO menu_items AS m (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT DISTINCT ON (i.merchant_id, i.sku)
       i.merchant_id, i.sku, i.name, i.price, i.is_available, i.source_updated_at, i.batch_id
FROM menu_item_imports AS i
WHERE i.batch_id = 2
  AND EXISTS (SELECT 1 FROM merchants AS q WHERE q.id = i.merchant_id)
  AND NOT EXISTS (SELECT 1 FROM menu_items AS x
                  WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)
ORDER BY i.merchant_id, i.sku, i.source_updated_at DESC, i.line_no DESC
ON CONFLICT (merchant_id, sku) DO UPDATE
SET name = EXCLUDED.name, price = EXCLUDED.price, is_available = EXCLUDED.is_available,
    source_updated_at = EXCLUDED.source_updated_at, updated_by_batch = EXCLUDED.updated_by_batch
WHERE EXCLUDED.source_updated_at > m.source_updated_at;

ROLLBACK;

-- Sau ROLLBACK: trạng thái của 17 còn nguyên (7 món, BB-02 vẫn thuộc lô 1).
-- expect-line: 7 | 1
SELECT count(*), (SELECT updated_by_batch FROM menu_items WHERE sku = 'BB-02') FROM menu_items;
