-- P02 · 19-cay-khu-vuc.sql — PG-019 `postgresql-recursive-cte-cay-khu-vuc` (phần 1/2)
-- Recursive CTE với cây khu vực: duyệt cây có ĐIỀU KIỆN DỪNG. Phần 2 —
-- dữ liệu có chu trình — ở 19-chu-trinh.sql.
--
-- Cây điều phối (bảng areas, 34 nút): 3 thành phố → 10 vùng → 21 khu. Vùng
-- HN-NA mới mở, chưa có khu nào. Mỗi chuyến gắn với một KHU (pickup_area_id).
--
-- WITH RECURSIVE gồm hai phần nối bằng UNION ALL (hoặc UNION):
--   * phần neo (non-recursive term): chạy MỘT lần, cho các dòng xuất phát;
--   * phần đệ quy: tham chiếu chính CTE, chạy lặp trên các dòng MỚI của vòng
--     trước — dừng khi một vòng không sinh thêm dòng nào.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- 1. Giải phẫu đệ quy: đếm 1 → 5. Điều kiện dừng nằm ở WHERE của phần đệ quy.
-- expect-line: 1
-- expect-line: 2
-- expect-line: 3
-- expect-line: 4
-- expect-line: 5
WITH RECURSIVE dem (n) AS (
  SELECT 1                       -- neo
  UNION ALL
  SELECT n + 1 FROM dem          -- đệ quy: chỉ thấy các dòng của vòng TRƯỚC
  WHERE n < 5                    -- điều kiện dừng
)
SELECT n FROM dem;

-- ---------------------------------------------------------------------------
-- 2. Đi XUỐNG: mọi nút thuộc cây TP. Hồ Chí Minh, kèm độ sâu và đường đi.
--    SEARCH DEPTH FIRST BY (PostgreSQL 14+) sinh cột thứ tự duyệt theo chiều
--    sâu — in cây đúng thứ tự cha trước, con ngay dưới.
-- expect-line: 0 | SG       | TP. Hồ Chí Minh
-- expect-line: 1 | SG-DO    | Phía Đông
-- expect-line: 2 | SG-DO-01 | Khu 01
-- expect-line: 2 | SG-DO-02 | Khu 02
-- expect-line: 2 | SG-DO-03 | Khu 03
-- expect-line: 1 | SG-NA    | Phía Nam
-- expect-line: 2 | SG-NA-01 | Khu 01
-- expect-line: 2 | SG-NA-02 | Khu 02
-- expect-line: 1 | SG-TA    | Phía Tây
-- expect-line: 2 | SG-TA-01 | Khu 01
-- expect-line: 2 | SG-TA-02 | Khu 02
-- expect-line: 1 | SG-TT    | Trung tâm
-- expect-line: 2 | SG-TT-01 | Khu 01
-- expect-line: 2 | SG-TT-02 | Khu 02
-- expect-line: 2 | SG-TT-03 | Khu 03
WITH RECURSIVE cay (id, code, name, do_sau) AS (
  SELECT id, code, name, 0 FROM areas WHERE code = 'SG'
  UNION ALL
  SELECT a.id, a.code, a.name, c.do_sau + 1
  FROM areas AS a
  JOIN cay AS c ON a.parent_id = c.id
) SEARCH DEPTH FIRST BY code SET thu_tu_duyet
SELECT do_sau, code, name
FROM cay
ORDER BY thu_tu_duyet;

-- Cùng kết quả, tự dựng đường đi bằng mảng (cách viết chạy được cả trước 14).
-- expect-line: SG > SG-TT > SG-TT-01
-- expect-line: SG > SG-TT > SG-TT-02
-- expect-line: SG > SG-TT > SG-TT-03
WITH RECURSIVE cay (id, code, duong_di) AS (
  SELECT id, code, ARRAY[code] FROM areas WHERE code = 'SG'
  UNION ALL
  SELECT a.id, a.code, c.duong_di || a.code
  FROM areas AS a
  JOIN cay AS c ON a.parent_id = c.id
)
SELECT array_to_string(duong_di, ' > ') AS duong_di
FROM cay
WHERE code LIKE 'SG-TT-%'
ORDER BY duong_di;

-- ---------------------------------------------------------------------------
-- 3. Đi LÊN: tổ tiên của khu HN-TA-02 (khu → vùng → thành phố).
-- expect-line: 0 | HN-TA-02 | zone
-- expect-line: 1 | HN-TA    | district
-- expect-line: 2 | HN       | city
WITH RECURSIVE to_tien (id, parent_id, code, level, bac) AS (
  SELECT id, parent_id, code, level, 0 FROM areas WHERE code = 'HN-TA-02'
  UNION ALL
  SELECT a.id, a.parent_id, a.code, a.level, t.bac + 1
  FROM areas AS a
  JOIN to_tien AS t ON a.id = t.parent_id
)
SELECT bac, code, level FROM to_tien ORDER BY bac;

-- ---------------------------------------------------------------------------
-- 4. Cộng dồn theo cây: số chuyến và doanh thu (chuyến hoàn thành) của MỌI
--    khu nằm dưới mỗi nút. Bước 1 dựng bảng "tổ tiên – hậu duệ" (closure),
--    bước 2 nối với chuyến. LEFT JOIN để vùng chưa có khu (HN-NA) vẫn hiện 0.
--    In các nút thành phố và vùng (bỏ 21 khu cho gọn).
-- expect-line: DN    | city     |  456 | 26124000
-- expect-line: DN-BI | district |  231 | 13618000
-- expect-line: DN-TT | district |  225 | 12506000
-- expect-line: HN    | city     | 1094 | 62246000
-- expect-line: HN-BA | district |  340 | 19456000
-- expect-line: HN-NA | district |    0 |        0
-- expect-line: HN-TA | district |  295 | 16496000
-- expect-line: HN-TT | district |  459 | 26294000
-- expect-line: SG    | city     | 1588 | 92143000
-- expect-line: SG-DO | district |  481 | 28484000
-- expect-line: SG-NA | district |  315 | 17353000
-- expect-line: SG-TA | district |  315 | 18523000
-- expect-line: SG-TT | district |  477 | 27783000
WITH RECURSIVE closure (to_tien_id, hau_due_id) AS (
  SELECT id, id FROM areas                     -- mỗi nút là hậu duệ của chính nó
  UNION ALL
  SELECT c.to_tien_id, a.id
  FROM closure AS c
  JOIN areas AS a ON a.parent_id = c.hau_due_id
)
SELECT n.code, n.level,
       count(t.id)                                              AS so_chuyen,
       coalesce(sum(t.fare) FILTER (WHERE t.status = 'completed'), 0) AS doanh_thu
FROM areas AS n
JOIN closure AS c ON c.to_tien_id = n.id
LEFT JOIN trips AS t ON t.pickup_area_id = c.hau_due_id
WHERE n.level <> 'zone'
GROUP BY n.id, n.code, n.level
ORDER BY n.code COLLATE "C";

-- Đối chiếu: tổng của mỗi thành phố qua cây PHẢI bằng đếm trực tiếp chuyến
-- của thành phố có khu đón (36 chuyến mất GPS không thuộc nút nào).
-- expect-line: 3138 | 3138 | 36
WITH RECURSIVE closure (to_tien_id, hau_due_id) AS (
  SELECT id, id FROM areas
  UNION ALL
  SELECT c.to_tien_id, a.id FROM closure AS c JOIN areas AS a ON a.parent_id = c.hau_due_id
)
SELECT (SELECT count(*) FROM areas AS n JOIN closure AS c ON c.to_tien_id = n.id
          JOIN trips AS t ON t.pickup_area_id = c.hau_due_id
         WHERE n.level = 'city')                                       AS qua_cay,
       (SELECT count(*) FROM trips WHERE pickup_area_id IS NOT NULL)   AS truc_tiep,
       (SELECT count(*) FROM trips WHERE pickup_area_id IS NULL)       AS mat_gps;

-- ---------------------------------------------------------------------------
-- 5. Kiểm tra cây LÀNH trước khi tin các con số trên: mọi nút phải đi tới
--    được từ một gốc, và độ sâu có giới hạn. Đếm nút đi được từ gốc = 34.
--    `WHERE c.do_sau < 10` là chốt an toàn: dù dữ liệu có chu trình, truy vấn
--    vẫn dừng (19-chu-trinh.sql).
-- expect-line: 34 | 34 | 2 | 22
WITH RECURSIVE tu_goc (id, do_sau) AS (
  SELECT id, 0 FROM areas WHERE parent_id IS NULL
  UNION ALL
  SELECT a.id, c.do_sau + 1
  FROM areas AS a JOIN tu_goc AS c ON a.parent_id = c.id
  WHERE c.do_sau < 10
)
SELECT (SELECT count(*) FROM areas)                                   AS tong_nut,
       (SELECT count(DISTINCT id) FROM tu_goc)                        AS di_toi_tu_goc,
       (SELECT max(do_sau) FROM tu_goc)                               AS sau_nhat,
       (SELECT count(*) FROM areas AS a
         WHERE NOT EXISTS (SELECT 1 FROM areas AS con WHERE con.parent_id = a.id)) AS so_la;
