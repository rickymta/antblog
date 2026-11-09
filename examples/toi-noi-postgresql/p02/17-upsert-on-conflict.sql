-- P02 · 17-upsert-on-conflict.sql — PG-017 `postgresql-upsert-on-conflict`
-- UPSERT và xung đột khoá duy nhất: NHẬP DỮ LIỆU có quy tắc cập nhật, với
-- fixture CỐ Ý trùng khoá.
--
-- Bài toán: quán gửi file thực đơn theo lô (batch) qua cổng đối tác. Tới Nơi
-- nạp vào bảng tạm `menu_item_imports` (nhận mọi thứ, không ràng buộc), rồi
-- hợp nhất sang `menu_items` theo quy tắc:
--   R1  khoá món = (quán, sku): chưa có thì THÊM, có rồi thì CẬP NHẬT;
--   R2  KHÔNG ghi đè dữ liệu MỚI hơn: chỉ cập nhật khi source_updated_at của
--       dòng nhập > của dòng đang có (lô đến trễ, gửi lại lô cũ…);
--   R3  không đổi gì thì KHÔNG ghi (tránh UPDATE thừa — mỗi UPDATE là một
--       phiên bản dòng mới, P04 giải thích);
--   R4  trong CÙNG một lô, một sku xuất hiện nhiều lần ⇒ lấy dòng mới nhất;
--   R5  dòng vi phạm ràng buộc khác (quán không tồn tại, trùng tên món khác
--       sku) ⇒ TỪ CHỐI dòng đó, không làm hỏng cả lô.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- Bảng đích và bảng nhập.
CREATE TABLE menu_items (
  merchant_id       uuid        NOT NULL,
  sku               text        NOT NULL,
  name              text        NOT NULL,
  price             integer     NOT NULL,           -- đồng
  is_available      boolean     NOT NULL,
  source_updated_at timestamptz NOT NULL,           -- giờ sửa ở hệ thống của quán
  updated_by_batch  integer     NOT NULL,           -- lô nào ghi dòng này lần cuối
  CONSTRAINT pk_menu_items PRIMARY KEY (merchant_id, sku),
  CONSTRAINT uq_menu_items_name UNIQUE (merchant_id, name),
  CONSTRAINT fk_menu_items_merchant FOREIGN KEY (merchant_id) REFERENCES merchants (id),
  CONSTRAINT ck_menu_items_price_positive CHECK (price > 0)
);

-- Bảng nhập: cố ý KHÔNG có FK/UNIQUE theo nghiệp vụ — nhận nguyên văn file.
CREATE TABLE menu_item_imports (
  batch_id          integer     NOT NULL,
  line_no           integer     NOT NULL,
  merchant_id       uuid        NOT NULL,
  sku               text        NOT NULL,
  name              text        NOT NULL,
  price             integer     NOT NULL,
  is_available      boolean     NOT NULL,
  source_updated_at timestamptz NOT NULL,
  CONSTRAINT pk_menu_item_imports PRIMARY KEY (batch_id, line_no)
);

-- Lô 1: thực đơn ban đầu của quán "Bún Bò Lab 101" (e1000000-…-000000000101).
-- expect-rows: 5
INSERT INTO menu_item_imports VALUES
  (1, 1, 'e1000000-0000-4000-8000-000000000101', 'BB-01', 'Bún bò đặc biệt', 55000, true, '2026-10-20 08:00+07'),
  (1, 2, 'e1000000-0000-4000-8000-000000000101', 'BB-02', 'Bún bò tái',      45000, true, '2026-10-20 08:00+07'),
  (1, 3, 'e1000000-0000-4000-8000-000000000101', 'BB-03', 'Bún bò chả',      45000, true, '2026-10-20 08:00+07'),
  (1, 4, 'e1000000-0000-4000-8000-000000000101', 'TR-01', 'Trà đá',           5000, true, '2026-10-20 08:00+07'),
  (1, 5, 'e1000000-0000-4000-8000-000000000101', 'NC-01', 'Nước cam',        25000, true, '2026-10-20 08:00+07');

-- Lô 2: mỗi dòng cài một tình huống (cột cuối của bảng xem trước ở mục 3).
-- expect-rows: 8
INSERT INTO menu_item_imports VALUES
  (2, 1, 'e1000000-0000-4000-8000-000000000101', 'BB-01', 'Bún bò đặc biệt', 59000, true,  '2026-10-28 09:00+07'), -- tăng giá
  (2, 2, 'e1000000-0000-4000-8000-000000000101', 'BB-02', 'Bún bò tái',      45000, true,  '2026-10-28 09:00+07'), -- y hệt, chỉ giờ mới
  (2, 3, 'e1000000-0000-4000-8000-000000000101', 'BB-03', 'Bún bò chả',      42000, true,  '2026-10-15 07:00+07'), -- bản CŨ gửi lại
  (2, 4, 'e1000000-0000-4000-8000-000000000101', 'TR-01', 'Trà đá',           5000, false, '2026-10-28 09:00+07'), -- hết hàng
  (2, 5, 'e1000000-0000-4000-8000-000000000101', 'BB-04', 'Bún bò giò',      50000, true,  '2026-10-28 09:05+07'), -- món mới…
  (2, 6, 'e1000000-0000-4000-8000-000000000101', 'BB-04', 'Bún bò giò',      52000, true,  '2026-10-28 09:30+07'), -- …sửa giá trong cùng lô
  (2, 7, 'e1000000-0000-4000-8000-000000000101', 'BB-05', 'Nước cam',        25000, true,  '2026-10-28 09:00+07'), -- sku mới, trùng TÊN NC-01
  (2, 8, 'e1000000-0000-4000-8000-000000000999', 'XX-01', 'Món lạ',          30000, true,  '2026-10-28 09:00+07'); -- quán không tồn tại

-- ---------------------------------------------------------------------------
-- 1. Nạp lô 1 bằng UPSERT: INSERT … ON CONFLICT (cột khoá) DO UPDATE.
--    EXCLUDED là dòng ĐỊNH chèn (bị xung đột); bảng đích gọi qua bí danh m.
-- expect-rows: 5
INSERT INTO menu_items AS m (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT merchant_id, sku, name, price, is_available, source_updated_at, batch_id
FROM menu_item_imports
WHERE batch_id = 1
ORDER BY line_no
ON CONFLICT (merchant_id, sku) DO UPDATE
SET name = EXCLUDED.name, price = EXCLUDED.price, is_available = EXCLUDED.is_available,
    source_updated_at = EXCLUDED.source_updated_at, updated_by_batch = EXCLUDED.updated_by_batch;

-- ---------------------------------------------------------------------------
-- 2. Lô 2, cách ngây thơ: cùng câu lệnh. BB-04 xuất hiện HAI lần trong lô:
--    dòng 5 vừa được chèn thì dòng 6 xung đột với chính nó ⇒ PostgreSQL từ
--    chối cả câu (một lệnh không được sửa cùng một dòng hai lần).
-- expect-error: 21000
INSERT INTO menu_items AS m (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT merchant_id, sku, name, price, is_available, source_updated_at, batch_id
FROM menu_item_imports
WHERE batch_id = 2
ORDER BY line_no
ON CONFLICT (merchant_id, sku) DO UPDATE
SET name = EXCLUDED.name, price = EXCLUDED.price, is_available = EXCLUDED.is_available,
    source_updated_at = EXCLUDED.source_updated_at, updated_by_batch = EXCLUDED.updated_by_batch;

-- Khử trùng trong lô (R4) bằng DISTINCT ON: giữ dòng đầu tiên của mỗi
-- (quán, sku) theo ORDER BY — ở đây là dòng mới nhất. Vẫn LỖI: ON CONFLICT chỉ
-- "bắt" xung đột của khoá được chỉ định (merchant_id, sku); dòng 7 vi phạm
-- MỘT ràng buộc UNIQUE khác (tên món) nên vẫn là lỗi thường.
-- expect-error: 23505 uq_menu_items_name
INSERT INTO menu_items AS m (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT DISTINCT ON (merchant_id, sku)
       merchant_id, sku, name, price, is_available, source_updated_at, batch_id
FROM menu_item_imports
WHERE batch_id = 2
ORDER BY merchant_id, sku, source_updated_at DESC, line_no DESC
ON CONFLICT (merchant_id, sku) DO UPDATE
SET name = EXCLUDED.name, price = EXCLUDED.price, is_available = EXCLUDED.is_available,
    source_updated_at = EXCLUDED.source_updated_at, updated_by_batch = EXCLUDED.updated_by_batch;

-- Hai câu lỗi trên không để lại gì: mỗi câu là một giao dịch, lỗi thì huỷ hết.
-- expect-value: 5
SELECT count(*) FROM menu_items;

-- ---------------------------------------------------------------------------
-- 3. XEM TRƯỚC (dry run): phân loại TỪNG dòng của lô theo R1–R5 trước khi ghi.
--    Đây cũng là "biên bản nhập" gửi lại cho quán.
-- expect-line: 1 | BB-01 | CẬP NHẬT
-- expect-line: 2 | BB-02 | BỎ QUA — không có gì thay đổi (R3)
-- expect-line: 3 | BB-03 | BỎ QUA — cũ hơn dữ liệu đang có (R2)
-- expect-line: 4 | TR-01 | CẬP NHẬT
-- expect-line: 5 | BB-04 | BỎ QUA — có dòng mới hơn trong cùng lô (R4)
-- expect-line: 6 | BB-04 | THÊM
-- expect-line: 7 | BB-05 | TỪ CHỐI — trùng tên món khác (R5)
-- expect-line: 8 | XX-01 | TỪ CHỐI — quán không tồn tại (R5)
SELECT i.line_no, i.sku,
       CASE
         WHEN NOT EXISTS (SELECT 1 FROM merchants AS q WHERE q.id = i.merchant_id)
           THEN 'TỪ CHỐI — quán không tồn tại (R5)'
         WHEN EXISTS (SELECT 1 FROM menu_items AS x
                      WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)
           THEN 'TỪ CHỐI — trùng tên món khác (R5)'
         WHEN i.line_no <> first_value(i.line_no) OVER (PARTITION BY i.merchant_id, i.sku
                                                        ORDER BY i.source_updated_at DESC, i.line_no DESC)
           THEN 'BỎ QUA — có dòng mới hơn trong cùng lô (R4)'
         WHEN m.sku IS NULL
           THEN 'THÊM'
         WHEN i.source_updated_at <= m.source_updated_at
           THEN 'BỎ QUA — cũ hơn dữ liệu đang có (R2)'
         WHEN (i.name, i.price, i.is_available) IS NOT DISTINCT FROM (m.name, m.price, m.is_available)
           THEN 'BỎ QUA — không có gì thay đổi (R3)'
         ELSE 'CẬP NHẬT'
       END AS ket_qua
FROM menu_item_imports AS i
LEFT JOIN menu_items AS m ON m.merchant_id = i.merchant_id AND m.sku = i.sku
WHERE i.batch_id = 2
ORDER BY i.line_no;

-- ---------------------------------------------------------------------------
-- 4. MERGE (PostgreSQL 15+; RETURNING + merge_action() từ PostgreSQL 17) cho
--    thấy hành động trên từng dòng. Chạy trong giao dịch rồi ROLLBACK — chỉ để
--    xem. Lưu ý: MERGE KHÔNG có cơ chế "chèn hay cập nhật" an toàn trước
--    phiên khác chèn cùng khoá như ON CONFLICT (có thể lỗi trùng khoá khi
--    chạy đồng thời) — P04. MERGE đặt trong WITH (cũng từ PostgreSQL 17) để
--    sắp xếp kết quả RETURNING.
BEGIN;

-- expect-line: UPDATE | BB-01 | 59000 | t
-- expect-line: INSERT | BB-04 | 52000 | t
-- expect-line: UPDATE | TR-01 |  5000 | f
WITH ket_qua AS (
MERGE INTO menu_items AS m
USING (
  SELECT DISTINCT ON (merchant_id, sku) *
  FROM menu_item_imports AS i
  WHERE batch_id = 2
    AND EXISTS (SELECT 1 FROM merchants AS q WHERE q.id = i.merchant_id)
    AND NOT EXISTS (SELECT 1 FROM menu_items AS x
                    WHERE x.merchant_id = i.merchant_id AND x.name = i.name AND x.sku <> i.sku)
  ORDER BY merchant_id, sku, source_updated_at DESC, line_no DESC
) AS s
ON m.merchant_id = s.merchant_id AND m.sku = s.sku
WHEN MATCHED AND s.source_updated_at > m.source_updated_at
              AND (m.name, m.price, m.is_available) IS DISTINCT FROM (s.name, s.price, s.is_available) THEN
  UPDATE SET name = s.name, price = s.price, is_available = s.is_available,
             source_updated_at = s.source_updated_at, updated_by_batch = s.batch_id
WHEN NOT MATCHED THEN
  INSERT (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
  VALUES (s.merchant_id, s.sku, s.name, s.price, s.is_available, s.source_updated_at, s.batch_id)
RETURNING merge_action() AS hanh_dong, m.sku, m.price, m.is_available
)
SELECT * FROM ket_qua ORDER BY sku;   -- thứ tự RETURNING của MERGE tuỳ kế hoạch nối ⇒ sắp lại

ROLLBACK;

-- ---------------------------------------------------------------------------
-- 5. UPSERT đúng quy tắc: lọc R5 trước, khử trùng R4, và đặt R2 + R3 vào
--    WHERE của DO UPDATE. Dòng bị WHERE loại không được cập nhật, không được
--    đếm trong thẻ lệnh, không có trong RETURNING.
--    Thẻ INSERT 0 3 = 1 thêm + 2 cập nhật.
-- expect-line: BB-01 | 59000 | t | 2
-- expect-line: BB-04 | 52000 | t | 2
-- expect-line: TR-01 |  5000 | f | 2
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
WHERE EXCLUDED.source_updated_at > m.source_updated_at                                    -- R2
  AND (m.name, m.price, m.is_available)
      IS DISTINCT FROM (EXCLUDED.name, EXCLUDED.price, EXCLUDED.is_available)              -- R3
RETURNING m.sku, m.price, m.is_available, m.updated_by_batch;

-- Trạng thái sau lô 2: BB-02 và BB-03 giữ nguyên dữ liệu và số lô 1.
-- expect-line: BB-01 | Bún bò đặc biệt | 59000 | t | 2026-10-28 09:00:00+07 | 2
-- expect-line: BB-02 | Bún bò tái      | 45000 | t | 2026-10-20 08:00:00+07 | 1
-- expect-line: BB-03 | Bún bò chả      | 45000 | t | 2026-10-20 08:00:00+07 | 1
-- expect-line: BB-04 | Bún bò giò      | 52000 | t | 2026-10-28 09:30:00+07 | 2
-- expect-line: NC-01 | Nước cam        | 25000 | t | 2026-10-20 08:00:00+07 | 1
-- expect-line: TR-01 | Trà đá          |  5000 | f | 2026-10-28 09:00:00+07 | 2
SELECT sku, name, price, is_available, source_updated_at, updated_by_batch
FROM menu_items
ORDER BY sku COLLATE "C";

-- ---------------------------------------------------------------------------
-- 6. IDEMPOTENT: chạy lại đúng câu đó (lô gửi lại lần hai) không đổi gì.
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
WHERE EXCLUDED.source_updated_at > m.source_updated_at
  AND (m.name, m.price, m.is_available)
      IS DISTINCT FROM (EXCLUDED.name, EXCLUDED.price, EXCLUDED.is_available);

-- ---------------------------------------------------------------------------
-- 7. ON CONFLICT DO NOTHING: "chỉ thêm món mới, món đã có thì để nguyên".
--    Lô 3 có một món đã có (BB-01, giá khác) và một món mới. Dòng xung đột bị
--    bỏ qua LẶNG LẼ — RETURNING chỉ trả dòng thật sự được chèn.
-- expect-rows: 2
INSERT INTO menu_item_imports VALUES
  (3, 1, 'e1000000-0000-4000-8000-000000000101', 'BB-01', 'Bún bò đặc biệt', 99000, true, '2026-10-30 08:00+07'),
  (3, 2, 'e1000000-0000-4000-8000-000000000101', 'CF-01', 'Cà phê sữa',      22000, true, '2026-10-30 08:00+07');

-- expect-line: CF-01 | 22000 | 3
INSERT INTO menu_items (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
SELECT merchant_id, sku, name, price, is_available, source_updated_at, batch_id
FROM menu_item_imports
WHERE batch_id = 3
ORDER BY line_no
ON CONFLICT (merchant_id, sku) DO NOTHING
RETURNING sku, price, updated_by_batch;

-- BB-01 vẫn 59.000 đ.
-- expect-value: 59000
SELECT price FROM menu_items WHERE sku = 'BB-01';

-- ON CONFLICT cần một ràng buộc UNIQUE/khoá chính KHỚP ĐÚNG các cột ghi ra
-- (hoặc ghi tên ràng buộc). Cột không có UNIQUE ⇒ lỗi 42P10 ngay khi lập kế hoạch.
-- expect-error: 42P10
INSERT INTO menu_items (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
VALUES ('e1000000-0000-4000-8000-000000000101', 'CF-02', 'Cà phê đen', 18000, true, '2026-10-30 08:00+07', 3)
ON CONFLICT (sku) DO NOTHING;

-- Ghi tên ràng buộc thay cho danh sách cột: ON CONFLICT ON CONSTRAINT.
-- expect-rows: 0
INSERT INTO menu_items (merchant_id, sku, name, price, is_available, source_updated_at, updated_by_batch)
VALUES ('e1000000-0000-4000-8000-000000000101', 'CF-01', 'Cà phê sữa', 22000, true, '2026-10-30 08:00+07', 3)
ON CONFLICT ON CONSTRAINT pk_menu_items DO NOTHING;
