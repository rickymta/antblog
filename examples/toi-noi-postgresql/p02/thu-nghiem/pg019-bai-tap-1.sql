-- PG-019 · Bài tập 1: nút lá kèm đường đi; lá nào không phải khu?
-- Nền: p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql
SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- expect-line: 22 | 1
WITH RECURSIVE cay (id, code, level, duong_di) AS (
  SELECT id, code, level, ARRAY[code] FROM areas WHERE parent_id IS NULL
  UNION ALL
  SELECT a.id, a.code, a.level, c.duong_di || a.code
  FROM areas AS a JOIN cay AS c ON a.parent_id = c.id
  WHERE cardinality(c.duong_di) < 10
),
la AS (
  SELECT * FROM cay AS c
  WHERE NOT EXISTS (SELECT 1 FROM areas AS con WHERE con.parent_id = c.id)
)
SELECT count(*) AS so_la, count(*) FILTER (WHERE level <> 'zone') AS la_khong_phai_khu FROM la;

-- expect-line: HN-NA | district | HN > HN-NA
WITH RECURSIVE cay (id, code, level, duong_di) AS (
  SELECT id, code, level, ARRAY[code] FROM areas WHERE parent_id IS NULL
  UNION ALL
  SELECT a.id, a.code, a.level, c.duong_di || a.code
  FROM areas AS a JOIN cay AS c ON a.parent_id = c.id
  WHERE cardinality(c.duong_di) < 10
)
SELECT code, level, array_to_string(duong_di, ' > ') AS duong_di
FROM cay AS c
WHERE level <> 'zone'
  AND NOT EXISTS (SELECT 1 FROM areas AS con WHERE con.parent_id = c.id)
ORDER BY code COLLATE "C";
