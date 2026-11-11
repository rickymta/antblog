-- PG-036 · FOR UPDATE, NOWAIT và SKIP LOCKED — `postgresql-for-update-skip-locked`
--
-- Bài toán Tới Nơi: bảng `jobs` là hàng đợi việc nền (gửi thông báo cho chuyến mới…). Nhiều
-- worker cùng lấy việc. Yêu cầu: (1) không việc nào bị HAI worker xử lý, (2) mọi việc đều
-- được xử lý, (3) worker không đứng chờ nhau vô ích. Câu "lấy việc kế tiếp":
--     SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE …
-- và ba biến thể của mệnh đề khoá: chờ (mặc định), NOWAIT (lỗi ngay), SKIP LOCKED (bỏ qua
-- dòng đang bị khoá). Hàng đợi ban đầu: việc 1 (ưu tiên 1) và 2…10 (ưu tiên 5), xem p04/01.
-- Mỗi lần xử lý THẬT ghi một dòng `job_runs` — bất biến: mỗi việc đúng một dòng.
--
-- Ba worker [A], [B], [C]. Phép thử ĐỒNG THỜI nhiều worker × nhiều việc (không tất định,
-- đếm và kiểm bất biến): p04/dong-thoi.mjs, kịch bản K036 (npm run p04:measure).

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- ===========================================================================
-- §1. FOR UPDATE (chờ): an toàn nhưng các worker XẾP HÀNG nối đuôi nhau.

-- @A
BEGIN;

-- @A
-- expect-value: 1
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE;

-- @B
BEGIN;

-- @B
-- B cũng muốn việc 1 (dòng đầu theo thứ tự) ⇒ chờ A.
-- expect-blocked
-- expect-value: 2
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE;

-- @A
-- Xử lý việc 1 (tác dụng + đánh dấu xong) TRONG CÙNG giao dịch rồi commit.
INSERT INTO job_runs (job_id, worker) VALUES (1, 'A');

-- @A
UPDATE jobs SET status = 'done', done_at = now(), attempts = attempts + 1 WHERE id = 1;

-- @A
COMMIT;

-- (B được khoá dòng 1, kiểm lại điều kiện: việc 1 đã 'done' ⇒ loại, lấy tiếp ⇒ việc 2. B
--  không nhận nhầm việc đã xong, nhưng đã phải đứng chờ suốt thời gian A xử lý.)

-- @B
INSERT INTO job_runs (job_id, worker) VALUES (2, 'B');

-- @B
UPDATE jobs SET status = 'done', done_at = now(), attempts = attempts + 1 WHERE id = 2;

-- @B
COMMIT;

-- ===========================================================================
-- §2. NOWAIT: không chờ — báo lỗi 55P03 ngay nếu dòng đang bị khoá.

-- @A
BEGIN;

-- @A
-- expect-value: 3
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE;

-- @B
BEGIN;

-- @B
-- expect-error: 55P03
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE NOWAIT;

-- @B
ROLLBACK;

-- @A
INSERT INTO job_runs (job_id, worker) VALUES (3, 'A');

-- @A
UPDATE jobs SET status = 'done', done_at = now(), attempts = attempts + 1 WHERE id = 3;

-- @A
COMMIT;

-- ===========================================================================
-- §3. SKIP LOCKED: mỗi worker lấy NGAY dòng chưa bị ai khoá — không chờ, không trùng.

-- @A
BEGIN;

-- @A
-- expect-value: 4
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED;

-- @B
BEGIN;

-- @B
-- expect-value: 5
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED;

-- @C
BEGIN;

-- @C
-- expect-value: 6
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED;

-- @C
INSERT INTO job_runs (job_id, worker) VALUES (6, 'C');

-- @C
UPDATE jobs SET status = 'done', done_at = now(), attempts = attempts + 1 WHERE id = 6;

-- @C
COMMIT;

-- @B
INSERT INTO job_runs (job_id, worker) VALUES (5, 'B');

-- @B
UPDATE jobs SET status = 'done', done_at = now(), attempts = attempts + 1 WHERE id = 5;

-- @B
COMMIT;

-- @A
INSERT INTO job_runs (job_id, worker) VALUES (4, 'A');

-- @A
UPDATE jobs SET status = 'done', done_at = now(), attempts = attempts + 1 WHERE id = 4;

-- @A
COMMIT;

-- ===========================================================================
-- §4. Việc có tác dụng NGOÀI database (gọi API gửi SMS, mất vài giây): không nên giữ giao
-- dịch + khoá trong lúc chờ mạng. Mẫu "nhận việc ngắn": MỘT câu đánh dấu running (commit
-- ngay) ⇒ xử lý ngoài giao dịch ⇒ câu thứ hai đánh dấu done. Worker chết giữa chừng để lại
-- việc 'running' mãi ⇒ cần "thời hạn thuê" (lease): việc running quá hạn được trả về hàng đợi.

-- @A
-- expect-line: 7 | running | 1
UPDATE jobs SET status = 'running', locked_by = 'A', locked_at = clock_timestamp(), attempts = attempts + 1
WHERE id = (SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED)
RETURNING id, status, attempts;

-- @A
-- A đã GỬI SMS (tác dụng bên ngoài — ở đây giả lập bằng một dòng job_runs)…
INSERT INTO job_runs (job_id, worker) VALUES (7, 'A');

-- (…rồi A "chết" trước khi kịp đánh dấu done.)

-- @C
-- Tiến trình dọn dẹp: việc running quá hạn thuê (ở đây 200 ms cho nhanh) về lại hàng đợi.
SELECT pg_sleep(0.3);

-- @C
-- expect-line: 7 | 1
UPDATE jobs SET status = 'queued', locked_by = NULL, locked_at = NULL
WHERE status = 'running' AND locked_at < clock_timestamp() - interval '200 milliseconds'
RETURNING id, attempts;

-- @B
-- expect-line: 7 | 2
UPDATE jobs SET status = 'running', locked_by = 'B', locked_at = clock_timestamp(), attempts = attempts + 1
WHERE id = (SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED)
RETURNING id, attempts;

-- @B
INSERT INTO job_runs (job_id, worker) VALUES (7, 'B');

-- @B
UPDATE jobs SET status = 'done', done_at = now(), locked_by = NULL, locked_at = NULL WHERE id = 7 AND locked_by = 'B';

-- @C
-- Việc 7 đã chạy HAI lần (khách nhận 2 SMS): tác dụng ngoài database chỉ đạt "ÍT NHẤT MỘT
-- lần" (at-least-once); bên nhận phải chống trùng (PG-039). Tác dụng TRONG database thì làm
-- cùng giao dịch với đánh dấu done (như §1–§3) ⇒ đúng một lần.
-- expect-line: 7 | A,B
SELECT job_id, string_agg(worker, ',' ORDER BY id) FROM job_runs WHERE job_id = 7 GROUP BY job_id;

-- ===========================================================================
-- §5. SKIP LOCKED KHÔNG công bằng: việc bị một worker "treo" giữ khoá thì những worker khác
-- cứ bỏ qua nó — kể cả việc ƯU TIÊN CAO NHẤT — và hàng đợi trông như rỗng.

-- @C
-- Việc gấp mới: id 11, ưu tiên 1. Còn trong hàng: 8, 9, 10 (ưu tiên 5).
-- expect-value: 11
INSERT INTO jobs (kind, priority) VALUES ('doi_soat', 1) RETURNING id;

-- @A
BEGIN;

-- @A
-- A nhận việc 11 rồi TREO (vd. chờ một API không trả lời), giao dịch vẫn mở.
-- expect-value: 11
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED;

-- @B
-- B và C làm hết việc ưu tiên 5 — việc gấp 11 bị vượt mặt.
-- expect-value: 8
WITH v AS (SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED),
     r AS (INSERT INTO job_runs (job_id, worker) SELECT id, 'B' FROM v)
UPDATE jobs SET status = 'done', done_at = now(), attempts = attempts + 1 FROM v WHERE jobs.id = v.id
RETURNING jobs.id;

-- @C
-- expect-value: 9
WITH v AS (SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED),
     r AS (INSERT INTO job_runs (job_id, worker) SELECT id, 'C' FROM v)
UPDATE jobs SET status = 'done', done_at = now(), attempts = attempts + 1 FROM v WHERE jobs.id = v.id
RETURNING jobs.id;

-- @B
-- expect-value: 10
WITH v AS (SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED),
     r AS (INSERT INTO job_runs (job_id, worker) SELECT id, 'B' FROM v)
UPDATE jobs SET status = 'done', done_at = now(), attempts = attempts + 1 FROM v WHERE jobs.id = v.id
RETURNING jobs.id;

-- @B
-- Với B, hàng đợi "rỗng" (0 dòng) — dù việc 11 vẫn 'queued'.
-- expect-rows: 0
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED;

-- @B
-- Đếm việc tồn bằng SKIP LOCKED cũng sai (0 thay vì 1): SKIP LOCKED cho một tập KHÔNG nhất
-- quán — chỉ dùng để lấy việc, đừng dùng để đếm/giám sát.
-- expect-line: 1 | 0
SELECT (SELECT count(*) FROM jobs WHERE status = 'queued') AS dem_thuong,
       (SELECT count(*) FROM (SELECT id FROM jobs WHERE status = 'queued' FOR UPDATE SKIP LOCKED) AS s) AS dem_skip_locked;

-- @C
-- Giám sát: việc queued đang bị khoá quá lâu = có worker treo. Huỷ phiên treo (pg_terminate)
-- ⇒ giao dịch của A rollback ⇒ khoá nhả.
-- expect-value: t
SELECT pg_terminate_backend(pid, 5000) FROM pg_stat_activity WHERE application_name = 'lab-A';

-- @B
-- expect-value: 11
WITH v AS (SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED),
     r AS (INSERT INTO job_runs (job_id, worker) SELECT id, 'B' FROM v)
UPDATE jobs SET status = 'done', done_at = now(), attempts = attempts + 1 FROM v WHERE jobs.id = v.id
RETURNING jobs.id;

-- ===========================================================================
-- §5b. Bẫy khoá ngoại: FOR UPDATE SKIP LOCKED bỏ qua cả dòng chỉ đang bị FOR KEY SHARE
-- (khoá do kiểm khoá ngoại khi bảng con chèn dòng trỏ tới nó). Dùng FOR NO KEY UPDATE.

-- @C
-- expect-value: 12
INSERT INTO jobs (kind, priority) VALUES ('tinh_cuoc', 5) RETURNING id;

-- @C
BEGIN;

-- @C
-- Một giao dịch khác chèn job_runs trỏ tới việc 12 (vd. ghi nhật ký) ⇒ FOR KEY SHARE trên 12.
INSERT INTO job_runs (job_id, worker) VALUES (12, 'kiem-tra');

-- @B
-- expect-rows: 0
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED;

-- @B
-- expect-value: 12
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR NO KEY UPDATE SKIP LOCKED;

-- @C
ROLLBACK;

-- @B
-- expect-value: 12
WITH v AS (SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR NO KEY UPDATE SKIP LOCKED),
     r AS (INSERT INTO job_runs (job_id, worker) SELECT id, 'B' FROM v)
UPDATE jobs SET status = 'done', done_at = now(), attempts = attempts + 1 FROM v WHERE jobs.id = v.id
RETURNING jobs.id;

-- ===========================================================================
-- §6. Bất biến: mọi việc xong; mỗi việc đúng MỘT lần xử lý — trừ việc 7 (tác dụng ngoài DB,
-- at-least-once, đã giải thích ở §4).

-- @C
-- expect-line: done | 12
SELECT status, count(*) FROM jobs GROUP BY status ORDER BY status;

-- @C
-- expect-line: 7 | 2
SELECT job_id, count(*) FROM job_runs GROUP BY job_id HAVING count(*) <> 1 ORDER BY job_id;

-- @C
-- expect-line: 1 | A
-- expect-line: 2 | B
-- expect-line: 3 | A
-- expect-line: 4 | A
-- expect-line: 5 | B
-- expect-line: 6 | C
-- expect-line: 7 | A,B
-- expect-line: 8 | B
-- expect-line: 9 | C
-- expect-line: 10 | B
-- expect-line: 11 | B
-- expect-line: 12 | B
SELECT job_id, string_agg(worker, ',' ORDER BY id) AS worker FROM job_runs GROUP BY job_id ORDER BY job_id;
