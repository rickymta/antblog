-- Thí nghiệm PG-036 (ba phiên): ba điểm bổ sung cho 36-for-update-skip-locked.sql.
--   §1. Thời hạn thuê (lease) + "vé" chống ghi nhầm (fencing token = attempts): worker A chậm,
--       hết hạn thuê, B nhận lại việc; A tỉnh dậy đánh dấu done KÈM vé cũ ⇒ 0 dòng (A biết mình
--       đã mất việc). Vé giữ đúng TRẠNG THÁI việc — nhưng tác dụng ngoài DB vẫn xảy ra hai lần.
--   §2. Lấy theo LÔ: LIMIT 3 … SKIP LOCKED — mỗi worker nhận ba việc khác nhau, không chờ.
--   §3. Việc ưu tiên 1 bị worker treo giữ: nhìn thấy khoá dòng bằng pgrowlocks (pg_locks không
--       liệt kê khoá dòng), và idle_in_transaction_session_timeout tự cắt worker treo ⇒ việc
--       được người khác nhận.
-- Hàng đợi đầu tệp: việc 1 (ưu tiên 1), 2…10 (ưu tiên 5), đều 'queued'.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- @C
CREATE EXTENSION pgrowlocks SCHEMA public;

-- ===========================================================================
-- §1. Lease + fencing token.

-- @A
-- A nhận việc bằng MỘT câu (commit ngay): running, ghi người giữ, giờ nhận, tăng attempts.
-- attempts trả về là "vé" của lượt thuê này.
-- expect-line: 1 | 1
UPDATE jobs SET status = 'running', locked_by = 'A', locked_at = clock_timestamp(), attempts = attempts + 1
WHERE id = (SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1
            FOR NO KEY UPDATE SKIP LOCKED)
RETURNING id, attempts;

-- @A
-- A gửi SMS (tác dụng ngoài DB, giả lập bằng job_runs) rồi bị treo lâu hơn thời hạn thuê.
INSERT INTO job_runs (job_id, worker) VALUES (1, 'A');

-- @C
SELECT pg_sleep(0.3);

-- @C
-- Dọn dẹp: running quá 200 ms ⇒ về hàng đợi (attempts giữ nguyên).
-- expect-line: 1 | 1
UPDATE jobs SET status = 'queued', locked_by = NULL, locked_at = NULL
WHERE status = 'running' AND locked_at < clock_timestamp() - interval '200 milliseconds'
RETURNING id, attempts;

-- @B
-- expect-line: 1 | 2
UPDATE jobs SET status = 'running', locked_by = 'B', locked_at = clock_timestamp(), attempts = attempts + 1
WHERE id = (SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1
            FOR NO KEY UPDATE SKIP LOCKED)
RETURNING id, attempts;

-- @A
-- A tỉnh dậy, đánh dấu xong KÈM vé attempts = 1 ⇒ 0 dòng: lượt thuê của A đã hết.
-- (Không kèm vé, câu này sẽ đánh dấu done một việc B đang làm.)
-- expect-rows: 0
UPDATE jobs SET status = 'done', done_at = now(), locked_by = NULL, locked_at = NULL
WHERE id = 1 AND status = 'running' AND attempts = 1;

-- @B
INSERT INTO job_runs (job_id, worker) VALUES (1, 'B');

-- @B
-- expect-rows: 1
UPDATE jobs SET status = 'done', done_at = now(), locked_by = NULL, locked_at = NULL
WHERE id = 1 AND status = 'running' AND attempts = 2;

-- @C
-- Trạng thái đúng (done, attempts 2) nhưng SMS vẫn đi hai lần.
-- expect-line: done | 2 | A,B
SELECT j.status, j.attempts, string_agg(r.worker, ',' ORDER BY r.id)
FROM jobs AS j JOIN job_runs AS r ON r.job_id = j.id
WHERE j.id = 1 GROUP BY j.status, j.attempts;

-- ===========================================================================
-- §2. Lấy theo lô.

-- @A
BEGIN;

-- @A
-- expect-line: 2
-- expect-line: 3
-- expect-line: 4
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 3 FOR NO KEY UPDATE SKIP LOCKED;

-- @B
BEGIN;

-- @B
-- expect-line: 5
-- expect-line: 6
-- expect-line: 7
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 3 FOR NO KEY UPDATE SKIP LOCKED;

-- @A
ROLLBACK;

-- @B
ROLLBACK;

-- ===========================================================================
-- §3. Worker treo giữ việc gấp: thấy bằng pgrowlocks, gỡ bằng idle_in_transaction_session_timeout.

-- @C
-- expect-value: 11
INSERT INTO jobs (kind, priority) VALUES ('doi_soat', 1) RETURNING id;

-- @A
-- Cấu hình phía worker: giao dịch ngồi yên quá 1 giây ⇒ server cắt kết nối.
SET idle_in_transaction_session_timeout = '1s';

-- @A
BEGIN;

-- @A
-- expect-value: 11
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR NO KEY UPDATE SKIP LOCKED;

-- @B
-- B bỏ qua việc gấp 11, lấy việc 2 (ưu tiên 5).
-- expect-value: 2
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR NO KEY UPDATE SKIP LOCKED;

-- @C
-- pg_locks không có khoá dòng; pgrowlocks đọc dấu khoá trong từng dòng của bảng.
-- expect-line: 11 | queued | {"For No Key Update"} | A
SELECT j.id, j.status, p.modes, lab_nhan(p.pids[1]) AS phien
FROM jobs AS j JOIN pgrowlocks('jobs') AS p ON p.locked_row = j.ctid;

-- @C
-- A ngồi yên trong giao dịch quá 1 giây ⇒ server cắt A, khoá trên việc 11 được nhả.
-- lab: nhat-ky
SELECT pg_sleep(1.5);

-- @B
-- expect-value: 11
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR NO KEY UPDATE SKIP LOCKED;

-- @A
-- expect-mat-ket-noi
SELECT 1;

-- @C
-- expect-value: 0
SELECT count(*) FROM pgrowlocks('jobs');
