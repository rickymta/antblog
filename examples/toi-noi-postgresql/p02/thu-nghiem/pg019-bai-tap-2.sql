-- PG-019 · Bài tập 2: cộng dồn cấp vùng và thành phố KHÔNG dùng đệ quy (cây cố định 3 tầng)
-- rồi đối chiếu với kết quả closure của 19-cay-khu-vuc.sql.
-- Nền: p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql
SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Mỗi chuyến có khu đón ⇒ một dòng (khu, vùng, thành phố) nhờ hai JOIN lên cha.
-- expect-line: DN    |  456 | 26124000
-- expect-line: DN-BI |  231 | 13618000
-- expect-line: DN-TT |  225 | 12506000
-- expect-line: HN    | 1094 | 62246000
-- expect-line: HN-BA |  340 | 19456000
-- expect-line: HN-NA |    0 |        0
-- expect-line: HN-TA |  295 | 16496000
-- expect-line: HN-TT |  459 | 26294000
-- expect-line: SG    | 1588 | 92143000
-- expect-line: SG-DO |  481 | 28484000
-- expect-line: SG-NA |  315 | 17353000
-- expect-line: SG-TA |  315 | 18523000
-- expect-line: SG-TT |  477 | 27783000
WITH chuyen_vung AS (
  SELECT t.id, t.status, t.fare, v.id AS vung_id, v.parent_id AS tp_id
  FROM trips AS t
  JOIN areas AS z ON z.id = t.pickup_area_id
  JOIN areas AS v ON v.id = z.parent_id
)
SELECT n.code,
       count(cv.id) AS so_chuyen,
       coalesce(sum(cv.fare) FILTER (WHERE cv.status = 'completed'), 0) AS doanh_thu
FROM areas AS n
LEFT JOIN chuyen_vung AS cv ON cv.vung_id = n.id OR cv.tp_id = n.id
WHERE n.level <> 'zone'
GROUP BY n.id, n.code
ORDER BY n.code COLLATE "C";
