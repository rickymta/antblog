-- PG-019 · thí nghiệm ngoài lab: từng vòng của phần đệ quy, SEARCH BREADTH FIRST,
-- và năm lỗi thường gặp khi viết WITH RECURSIVE.
-- Nền: p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql
SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- 1. Mỗi vòng chỉ xử lý các dòng MỚI của vòng trước: đánh số vòng rồi gom lại.
-- expect-line: 0 |  1 | SG
-- expect-line: 1 |  4 | SG-DO SG-NA SG-TA SG-TT
-- expect-line: 2 | 10 | SG-DO-01 SG-DO-02 SG-DO-03 SG-NA-01 SG-NA-02 SG-TA-01 SG-TA-02 SG-TT-01 SG-TT-02 SG-TT-03
WITH RECURSIVE cay (id, code, vong) AS (
  SELECT id, code, 0 FROM areas WHERE code = 'SG'
  UNION ALL
  SELECT a.id, a.code, c.vong + 1 FROM areas AS a JOIN cay AS c ON a.parent_id = c.id
)
SELECT vong, count(*) AS so_nut, string_agg(code, ' ' ORDER BY code COLLATE "C") AS cac_nut
FROM cay GROUP BY vong ORDER BY vong;

-- 2. SEARCH BREADTH FIRST: cột thứ tự là bản ghi (độ sâu, khoá).
-- expect-line: 0 | SG       | (0,SG)
-- expect-line: 1 | SG-DO    | (1,SG-DO)
-- expect-line: 1 | SG-NA    | (1,SG-NA)
-- expect-line: 1 | SG-TA    | (1,SG-TA)
-- expect-line: 1 | SG-TT    | (1,SG-TT)
-- expect-line: 2 | SG-DO-01 | (2,SG-DO-01)
-- expect-line: 2 | SG-DO-02 | (2,SG-DO-02)
-- expect-line: 2 | SG-DO-03 | (2,SG-DO-03)
-- expect-line: 2 | SG-NA-01 | (2,SG-NA-01)
-- expect-line: 2 | SG-NA-02 | (2,SG-NA-02)
-- expect-line: 2 | SG-TA-01 | (2,SG-TA-01)
-- expect-line: 2 | SG-TA-02 | (2,SG-TA-02)
-- expect-line: 2 | SG-TT-01 | (2,SG-TT-01)
-- expect-line: 2 | SG-TT-02 | (2,SG-TT-02)
-- expect-line: 2 | SG-TT-03 | (2,SG-TT-03)
WITH RECURSIVE cay (id, code, do_sau) AS (
  SELECT id, code, 0 FROM areas WHERE code = 'SG'
  UNION ALL
  SELECT a.id, a.code, c.do_sau + 1 FROM areas AS a JOIN cay AS c ON a.parent_id = c.id
) SEARCH BREADTH FIRST BY code SET thu_tu
SELECT do_sau, code, thu_tu FROM cay ORDER BY thu_tu;

-- SEARCH DEPTH FIRST: cột thứ tự là mảng đường đi.
-- expect-line: 1 | SG-DO    | {(SG),(SG-DO)}
-- expect-line: 2 | SG-DO-01 | {(SG),(SG-DO),(SG-DO-01)}
-- expect-line: 2 | SG-DO-02 | {(SG),(SG-DO),(SG-DO-02)}
-- expect-line: 2 | SG-DO-03 | {(SG),(SG-DO),(SG-DO-03)}
WITH RECURSIVE cay (id, code, do_sau) AS (
  SELECT id, code, 0 FROM areas WHERE code = 'SG'
  UNION ALL
  SELECT a.id, a.code, c.do_sau + 1 FROM areas AS a JOIN cay AS c ON a.parent_id = c.id
) SEARCH DEPTH FIRST BY code SET thu_tu
SELECT do_sau, code, thu_tu FROM cay WHERE code LIKE 'SG-DO%' ORDER BY thu_tu;

-- 3. Lỗi thường gặp.
-- a) Quên chữ RECURSIVE.
-- expect-error: 42P01
WITH cay (id, code) AS (
  SELECT id, code FROM areas WHERE code = 'SG'
  UNION ALL
  SELECT a.id, a.code FROM areas AS a JOIN cay AS c ON a.parent_id = c.id
)
SELECT count(*) FROM cay;

-- b) Tham chiếu đệ quy nằm ở phía "có thể NULL" của outer join.
-- expect-error: 42P19
WITH RECURSIVE cay (id, code) AS (
  SELECT id, code FROM areas WHERE code = 'SG'
  UNION ALL
  SELECT a.id, a.code FROM areas AS a LEFT JOIN cay AS c ON a.parent_id = c.id
)
SELECT count(*) FROM cay;

-- c) Hàm gộp trong phần đệ quy.
-- expect-error: 42P19
WITH RECURSIVE cay (id, n) AS (
  SELECT id, 1 FROM areas WHERE code = 'SG'
  UNION ALL
  SELECT max(a.id), count(*)::int FROM areas AS a JOIN cay AS c ON a.parent_id = c.id
)
SELECT count(*) FROM cay;

-- d) Tham chiếu đệ quy hai lần.
-- expect-error: 42P19
WITH RECURSIVE cay (id) AS (
  SELECT id FROM areas WHERE code = 'SG'
  UNION ALL
  SELECT a.id FROM areas AS a JOIN cay AS c ON a.parent_id = c.id JOIN cay AS c2 ON c2.id = c.id
)
SELECT count(*) FROM cay;

-- e) Kiểu cột ở phần neo khác kiểu phần đệ quy.
-- expect-error: 42804
WITH RECURSIVE cay (id, duong_di) AS (
  SELECT id, code::varchar(40) FROM areas WHERE code = 'SG'
  UNION ALL
  SELECT a.id, c.duong_di || ' > ' || a.code FROM areas AS a JOIN cay AS c ON a.parent_id = c.id
)
SELECT count(*) FROM cay;

-- Sửa e): ép phần neo về đúng kiểu của cả cột (text).
-- expect-value: 15
WITH RECURSIVE cay (id, duong_di) AS (
  SELECT id, code::text FROM areas WHERE code = 'SG'
  UNION ALL
  SELECT a.id, c.duong_di || ' > ' || a.code FROM areas AS a JOIN cay AS c ON a.parent_id = c.id
)
SELECT count(*) FROM cay;
