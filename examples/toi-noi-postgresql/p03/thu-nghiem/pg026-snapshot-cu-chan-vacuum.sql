-- Thí nghiệm PG-026 (hai phiên): một giao dịch MỞ LÂU giữ snapshot cũ ⇒ VACUUM không
-- dọn được phiên bản chết và không đặt lại dấu all-visible ⇒ Index Only Scan tiếp tục
-- phải ghé heap ("Heap Fetches" > 0) cho tới khi giao dịch đó kết thúc.
-- (Cũng là lý do "idle in transaction" lâu trên production làm bảng phình — P04/P05.)

-- @A
BEGIN ISOLATION LEVEL REPEATABLE READ;
-- Câu đầu tiên trong REPEATABLE READ chụp snapshot của cả giao dịch.
-- expect-value: 1000
SELECT count(*) FROM toinoi.trips WHERE code BETWEEN 500000 AND 500999;

-- @A
SELECT sum(fare) AS tong_luc_dau FROM toinoi.trips WHERE code BETWEEN 500000 AND 500999 \gset

-- @B
-- expect-rows: 1000
UPDATE toinoi.trips SET fare = fare + 1000 WHERE code BETWEEN 500000 AND 500999;

-- @B
VACUUM toinoi.trips;

-- Phiên bản cũ vẫn có thể được A nhìn thấy ⇒ VACUUM phải giữ lại; page chứa chúng
-- không all-visible ⇒ Index Only Scan ghé heap.
-- @B
-- expect-match: Index Only Scan using uq_trips_code on trips
-- expect-match: Heap Fetches: [1-9]\d*$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code FROM toinoi.trips WHERE code BETWEEN 500000 AND 500999;

-- A vẫn thấy dữ liệu NHƯ LÚC CHỤP snapshot (cước cũ), dù B đã COMMIT.
-- @A
-- expect-value: t
SELECT sum(fare) = :tong_luc_dau FROM toinoi.trips WHERE code BETWEEN 500000 AND 500999;

-- @A
COMMIT;

-- VACUUM thường giờ dọn được phiên bản chết… nhưng với số dòng chết ÍT (dưới ~2 % số page
-- của bảng), PostgreSQL (từ bản 14) BỎ QUA bước dọn index để tiết kiệm ("index scan
-- bypassed" trong VACUUM VERBOSE) ⇒ các ô chết (LP_DEAD) còn trong page ⇒ page chưa thể
-- all-visible ⇒ Index Only Scan vẫn ghé heap ~1.000 lần.
-- @B
VACUUM toinoi.trips;

-- @B
-- expect-match: Heap Fetches: 1000$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code FROM toinoi.trips WHERE code BETWEEN 500000 AND 500999;

-- Ép dọn cả index: INDEX_CLEANUP ON ⇒ page sạch ô chết ⇒ đặt lại all-visible.
-- @B
VACUUM (INDEX_CLEANUP ON) toinoi.trips;

-- @B
-- expect-match: Heap Fetches: 0$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code FROM toinoi.trips WHERE code BETWEEN 500000 AND 500999;
