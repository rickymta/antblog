-- Thí nghiệm PG-032 (hai phiên) — lời giải ba bài tập của bài.
--   §A  Danh sách "đang chạy" của snapshot KHÔNG rỗng: A (X+1) giữ giao dịch mở, B (X+2) ghi
--       và commit sau đó ⇒ snapshot của B = xmin=X+1 xmax=X+3 đang chạy={X+1}.
--   §B  REPEATABLE READ chụp snapshot ở câu lệnh ĐẦU TIÊN, không phải ở BEGIN.
--   §C  DELETE cũng chỉ đặt xmax: B vẫn thấy dòng tới khi A commit; sau đó còn 1 dòng chết.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @A
SELECT lab_dat_moc_xid();

-- ===========================================================================
-- §A. Snapshot có giao dịch đang chạy nằm GIỮA xmin và xmax.

-- @A
BEGIN;

-- @A
-- expect-line: 35000 | X+1
UPDATE trips SET fare = 35000 WHERE code = 400104 RETURNING fare, xid_td(xmin);

-- @B
-- Câu lẻ của B: giao dịch X+2, commit ngay (A vẫn mở).
-- expect-line: 40000 | X+2
UPDATE trips SET fare = 40000 WHERE code = 400105 RETURNING fare, xid_td(xmin);

-- @B
-- X+2 đã kết thúc ⇒ xmax = X+3; X+1 < xmax và chưa kết thúc ⇒ phải nằm trong danh sách.
-- expect-value: xmin=X+1 xmax=X+3 đang chạy={X+1}
SELECT snap_td(pg_current_snapshot());

-- @B
-- expect-line: 34000 | 40000
SELECT (SELECT fare FROM trips WHERE code = 400104) AS c400104,
       (SELECT fare FROM trips WHERE code = 400105) AS c400105;

-- @A
COMMIT;

-- @B
-- expect-value: xmin=X+3 xmax=X+3 đang chạy={}
SELECT snap_td(pg_current_snapshot());

-- ===========================================================================
-- §B. REPEATABLE READ: snapshot chụp ở câu đầu tiên của giao dịch, không phải ở BEGIN.

-- @B
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @A
-- A sửa và commit SAU khi B gõ BEGIN nhưng TRƯỚC câu đầu tiên của B.
-- expect-line: 40000 | X+3
UPDATE trips SET fare = 40000 WHERE code = 400106 RETURNING fare, xid_td(xmin);

-- @B
-- Snapshot chụp ở đây ⇒ thấy 40000 (bản của X+3).
-- expect-value: 40000
SELECT fare FROM trips WHERE code = 400106;

-- @A
-- expect-line: 41000 | X+4
UPDATE trips SET fare = 41000 WHERE code = 400106 RETURNING fare, xid_td(xmin);

-- @B
-- expect-value: 40000
SELECT fare FROM trips WHERE code = 400106;

-- @B
COMMIT;

-- ===========================================================================
-- §C. DELETE: phiên bản bị xoá chỉ được đánh xmax; còn nằm trong page tới khi VACUUM.

-- @B
VACUUM toinoi.jobs;

-- @B
-- expect-line: 10 | 0
SELECT tuple_count, dead_tuple_count FROM pgstattuple('toinoi.jobs');

-- @A
BEGIN;

-- @A
-- expect-line: 3 | X+5
DELETE FROM jobs WHERE id = 3 RETURNING id, xid_td(pg_current_xact_id());

-- @B
-- A chưa commit: B vẫn thấy việc 3, xmax = X+5.
-- expect-line: 3 | <X | X+5
SELECT id, xid_td(xmin) AS xmin, xid_td(xmax) AS xmax FROM jobs WHERE id = 3;

-- @A
COMMIT;

-- @B
-- expect-value: 0
SELECT count(*) FROM jobs WHERE id = 3;

-- @B
-- expect-line: 9 | 1
SELECT tuple_count, dead_tuple_count FROM pgstattuple('toinoi.jobs');

-- @B
VACUUM toinoi.jobs;

-- @B
-- expect-line: 9 | 0
SELECT tuple_count, dead_tuple_count FROM pgstattuple('toinoi.jobs');
