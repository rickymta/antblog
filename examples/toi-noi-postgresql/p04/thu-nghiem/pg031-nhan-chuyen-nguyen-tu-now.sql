-- Thí nghiệm PG-031 (hai phiên) — lời giải ba bài tập của bài.
--   §A  Tài xế nhận chuyến = UPDATE trips + INSERT trip_events trong MỘT giao dịch. Câu thứ hai
--       lỗi (gõ sai trip_id ⇒ FK 23503) ⇒ COMMIT trả ROLLBACK ⇒ chuyến vẫn 'requested', không
--       tài xế. Làm lại đúng ⇒ cả hai cùng có.
--   §B  Một câu lệnh LẺ (autocommit) cũng nguyên tử: INSERT nhiều dòng, một dòng sai ⇒ không
--       dòng nào vào.
--   §C  now() đứng yên trong giao dịch; statement_timestamp() theo từng câu; clock_timestamp()
--       là đồng hồ thật.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- ===========================================================================
-- §A. Nhận chuyến 400101 cho tài xế 9101 — lần 1 có lỗi ở câu thứ hai.

-- @A
BEGIN;

-- @A
-- expect-line: accepted | 1
UPDATE trips
SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
    accepted_at = now(), version = version + 1, updated_at = now()
WHERE code = 400101 AND status = 'requested'
RETURNING status, (SELECT count(*) FROM trip_events AS e WHERE e.trip_id = trips.id) AS su_kien;

-- @A
-- Gõ nhầm trip_id (…400199 — không có chuyến nào) ⇒ vi phạm khoá ngoại.
-- expect-error: 23503 fk_trip_events_trip
INSERT INTO trip_events (trip_id, status)
VALUES ('a4000000-0000-4000-8000-000000400199', 'accepted');

-- @A
-- expect-match: ^ROLLBACK$
COMMIT;

-- @B
-- Không có "chuyến đã có tài xế mà lịch sử không ghi": UPDATE cũng bị huỷ.
-- expect-line: requested | t | 1
SELECT t.status, t.driver_id IS NULL AS chua_co_tai_xe, count(e.id) AS su_kien
FROM trips AS t JOIN trip_events AS e ON e.trip_id = t.id
WHERE t.code = 400101
GROUP BY t.status, t.driver_id;

-- @A
BEGIN;

-- @A
-- expect-rows: 1
UPDATE trips
SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
    accepted_at = now(), version = version + 1, updated_at = now()
WHERE code = 400101 AND status = 'requested';

-- @A
-- expect-rows: 1
INSERT INTO trip_events (trip_id, status)
VALUES ('a4000000-0000-4000-8000-000000400101', 'accepted');

-- @A
-- expect-match: ^COMMIT$
COMMIT;

-- @B
-- expect-line: accepted | f | 2 | t
SELECT t.status, t.driver_id IS NULL AS chua_co_tai_xe, count(e.id) AS su_kien,
       max(e.occurred_at) = t.accepted_at AS cung_moc
FROM trips AS t JOIN trip_events AS e ON e.trip_id = t.id
WHERE t.code = 400101
GROUP BY t.status, t.driver_id, t.accepted_at;

-- ===========================================================================
-- §B. Câu lệnh lẻ cũng nguyên tử (không cần BEGIN).

-- @B
-- expect-value: 1
SELECT count(*) FROM trip_events WHERE trip_id = 'a4000000-0000-4000-8000-000000400102';

-- @B
-- Dòng 1 hợp lệ, dòng 2 sai trạng thái ⇒ cả câu lỗi, dòng 1 cũng không vào.
-- expect-error: 23514 ck_trip_events_status
INSERT INTO trip_events (trip_id, status)
VALUES ('a4000000-0000-4000-8000-000000400102', 'searching'),
       ('a4000000-0000-4000-8000-000000400102', 'dang_tim');

-- @B
-- expect-value: 1
SELECT count(*) FROM trip_events WHERE trip_id = 'a4000000-0000-4000-8000-000000400102';

-- ===========================================================================
-- §C. Ba loại "bây giờ" trong một giao dịch.

-- @A
BEGIN;

-- @A
CREATE TEMP TABLE moc AS SELECT now() AS n, clock_timestamp() AS c;

-- @A
SELECT pg_sleep(0.3);

-- @A
-- now() vẫn là đầu giao dịch; statement_timestamp() là lúc câu này tới; clock_timestamp() đã
-- trôi thêm ít nhất 0,3 s.
-- expect-line: t | t | t
SELECT now() = n AS now_dung_yen,
       statement_timestamp() > n + interval '0.25 second' AS statement_moi,
       clock_timestamp() > c + interval '0.25 second' AS dong_ho_chay
FROM moc;

-- @A
ROLLBACK;
