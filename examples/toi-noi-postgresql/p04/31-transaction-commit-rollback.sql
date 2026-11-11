-- PG-031 · Transaction, COMMIT và ROLLBACK — `postgresql-transaction-commit-rollback`
--
-- Bài toán Tới Nơi: khi khách đặt chuyến, server ghi HAI thứ — dòng `trips` và sự kiện
-- 'requested' trong `trip_events`. Không bao giờ được có chuyến mà không có sự kiện (màn
-- lịch sử của CSKH trống) hay sự kiện trỏ tới chuyến không tồn tại. Giao dịch (transaction)
-- gom nhiều câu lệnh thành MỘT đơn vị: cùng thành công (COMMIT) hoặc cùng bị huỷ (ROLLBACK).
--
-- Hai phiên: [A] = server API đặt chuyến, [B] = người quan sát (CSKH / một request khác).
-- Mỗi phiên là một kết nối riêng (một tiến trình psql). Runner gửi từng bước theo thứ tự
-- trong tệp và in nhãn phiên trước mỗi bước — đó là LỊCH CHẠY của bài.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- ===========================================================================
-- §1. Chuyến + sự kiện trong MỘT giao dịch: B không thấy gì cho tới khi A COMMIT.

-- @A
BEGIN;

-- @A
-- Mã chuyến (code) do IDENTITY cấp: lấy giá trị kế tiếp của sequence (đang ở 400200).
-- expect-value: 400200
INSERT INTO trips (id, kind, city_id, customer_id, fare, pickup_area_id)
VALUES ('a4000000-0000-4000-8000-000000400201', 'ride', 1, 'c4000000-0000-4000-8000-000000000011', 45000, 111)
RETURNING code;

-- @A
-- expect-rows: 1
INSERT INTO trip_events (trip_id, status)
VALUES ('a4000000-0000-4000-8000-000000400201', 'requested');

-- B (một kết nối khác) chưa thấy dòng nào: dữ liệu chưa COMMIT chỉ giao dịch A thấy.
-- @B
-- expect-line: 0 | 0
SELECT (SELECT count(*) FROM trips WHERE id = 'a4000000-0000-4000-8000-000000400201') AS chuyen,
       (SELECT count(*) FROM trip_events WHERE trip_id = 'a4000000-0000-4000-8000-000000400201') AS su_kien;

-- Trong giao dịch, now() là thời điểm BẮT ĐẦU giao dịch — giống nhau ở mọi câu, nên
-- requested_at của chuyến = occurred_at của sự kiện (cả hai lấy mặc định now()).
-- @A
-- expect-value: t
SELECT t.requested_at = e.occurred_at AS cung_moc_now
FROM trips AS t JOIN trip_events AS e ON e.trip_id = t.id
WHERE t.id = 'a4000000-0000-4000-8000-000000400201';

-- @A
COMMIT;

-- @B
-- expect-line: 1 | 1
SELECT (SELECT count(*) FROM trips WHERE id = 'a4000000-0000-4000-8000-000000400201') AS chuyen,
       (SELECT count(*) FROM trip_events WHERE trip_id = 'a4000000-0000-4000-8000-000000400201') AS su_kien;

-- ===========================================================================
-- §2. Lỗi giữa giao dịch: mọi câu sau bị từ chối (25P02) và COMMIT thực chất là ROLLBACK.

-- @A
BEGIN;

-- @A
-- Câu này thành công và lấy mã 400201 từ sequence…
-- expect-value: 400201
INSERT INTO trips (id, kind, city_id, customer_id, fare, pickup_area_id)
VALUES ('a4000000-0000-4000-8000-000000400202', 'ride', 1, 'c4000000-0000-4000-8000-000000000012', 52000, 112)
RETURNING code;

-- @A
-- …câu này vi phạm CHECK (trạng thái không có trong danh sách) ⇒ lỗi ⇒ giao dịch HỎNG.
-- expect-error: 23514 ck_trip_events_status
INSERT INTO trip_events (trip_id, status, occurred_at)
VALUES ('a4000000-0000-4000-8000-000000400202', 'dang_tim', TIMESTAMPTZ '2026-10-31 01:00:00+07');

-- @A
-- Giao dịch hỏng từ chối MỌI câu (kể cả SELECT) cho tới khi kết thúc giao dịch.
-- expect-error: 25P02
SELECT 1;

-- @A
-- COMMIT trên giao dịch hỏng không báo lỗi — server trả thẻ ROLLBACK. Ứng dụng phải đọc
-- kết quả của COMMIT, đừng mặc định "đã lưu".
-- expect-match: ^ROLLBACK$
COMMIT;

-- @B
-- expect-value: 0
SELECT count(*) FROM trips WHERE id = 'a4000000-0000-4000-8000-000000400202';

-- ===========================================================================
-- §3. SAVEPOINT: huỷ MỘT PHẦN giao dịch, giữ phần trước điểm lưu.

-- @A
BEGIN;

-- @A
-- Mã 400201 đã bị giao dịch hỏng ở §2 lấy mất: sequence KHÔNG quay lui khi ROLLBACK
-- (để các giao dịch song song không phải chờ nhau) ⇒ mã chuyến có thể nhảy cóc.
-- expect-value: 400202
INSERT INTO trips (id, kind, city_id, customer_id, fare, pickup_area_id)
VALUES ('a4000000-0000-4000-8000-000000400203', 'parcel', 1, 'c4000000-0000-4000-8000-000000000013', 38000, 113)
RETURNING code;

-- @A
SAVEPOINT truoc_su_kien;

-- @A
-- expect-error: 23514 ck_trip_events_status
INSERT INTO trip_events (trip_id, status, occurred_at)
VALUES ('a4000000-0000-4000-8000-000000400203', 'dang_tim', TIMESTAMPTZ '2026-10-31 01:00:00+07');

-- @A
ROLLBACK TO SAVEPOINT truoc_su_kien;

-- @A
-- Sau ROLLBACK TO, giao dịch dùng tiếp được; chuyến chèn trước điểm lưu vẫn còn.
-- expect-rows: 1
INSERT INTO trip_events (trip_id, status)
VALUES ('a4000000-0000-4000-8000-000000400203', 'requested');

-- @A
-- expect-match: ^COMMIT$
COMMIT;

-- @B
-- expect-line: 400202 | 1
SELECT t.code, count(e.id) AS su_kien
FROM trips AS t LEFT JOIN trip_events AS e ON e.trip_id = t.id
WHERE t.id = 'a4000000-0000-4000-8000-000000400203'
GROUP BY t.code;

-- psql có biến ON_ERROR_ROLLBACK: bật lên thì psql TỰ đặt SAVEPOINT trước mỗi câu trong
-- giao dịch và tự ROLLBACK TO khi câu đó lỗi — tiện khi gõ tay, nhưng ứng dụng thật (driver
-- JDBC/Npgsql/psycopg…) không làm vậy; mỗi savepoint là một giao dịch con có chi phí.
-- @A
\set ON_ERROR_ROLLBACK on

-- @A
BEGIN;

-- @A
-- expect-value: 400203
INSERT INTO trips (id, kind, city_id, customer_id, fare, pickup_area_id)
VALUES ('a4000000-0000-4000-8000-000000400204', 'ride', 1, 'c4000000-0000-4000-8000-000000000014', 41000, 111)
RETURNING code;

-- @A
-- expect-error: 23514 ck_trip_events_status
INSERT INTO trip_events (trip_id, status, occurred_at)
VALUES ('a4000000-0000-4000-8000-000000400204', 'dang_tim', TIMESTAMPTZ '2026-10-31 01:00:00+07');

-- @A
-- Không còn 25P02: psql đã lùi về savepoint ngầm của câu lỗi.
-- expect-rows: 1
INSERT INTO trip_events (trip_id, status)
VALUES ('a4000000-0000-4000-8000-000000400204', 'requested');

-- @A
-- expect-match: ^COMMIT$
COMMIT;

-- @A
\set ON_ERROR_ROLLBACK off

-- ===========================================================================
-- §4. Autocommit của psql: mặc định BẬT — mỗi câu ngoài BEGIN là một giao dịch riêng.
-- Tắt đi (\set AUTOCOMMIT off) thì psql tự gửi BEGIN trước câu đầu tiên và CHỜ bạn COMMIT.

-- @A
\set AUTOCOMMIT off

-- @A
-- expect-value: 400204
INSERT INTO trips (id, kind, city_id, customer_id, fare, pickup_area_id)
VALUES ('a4000000-0000-4000-8000-000000400205', 'ride', 1, 'c4000000-0000-4000-8000-000000000015', 47000, 112)
RETURNING code;

-- @A
-- expect-rows: 1
INSERT INTO trip_events (trip_id, status)
VALUES ('a4000000-0000-4000-8000-000000400205', 'requested');

-- @A
-- Đang ở trong giao dịch (đã được cấp xid vì vừa ghi) dù không gõ BEGIN.
-- expect-value: t
SELECT pg_current_xact_id_if_assigned() IS NOT NULL AS dang_trong_giao_dich;

-- @B
-- expect-value: 0
SELECT count(*) FROM trips WHERE id = 'a4000000-0000-4000-8000-000000400205';

-- @A
COMMIT;

-- @A
\set AUTOCOMMIT on

-- @B
-- expect-value: 1
SELECT count(*) FROM trips WHERE id = 'a4000000-0000-4000-8000-000000400205';

-- Ngược lại, với autocommit BẬT, câu lẻ tự COMMIT: không còn giao dịch nào mở sau câu.
-- @A
-- expect-value: t
SELECT pg_current_xact_id_if_assigned() IS NULL AS khong_con_giao_dich_mo;

-- ===========================================================================
-- §5. Mất kết nối GIỮA giao dịch: server huỷ giao dịch, không để lại nửa vời.

-- @A
BEGIN;

-- @A
-- expect-value: 400205
INSERT INTO trips (id, kind, city_id, customer_id, fare, pickup_area_id)
VALUES ('a4000000-0000-4000-8000-000000400206', 'ride', 1, 'c4000000-0000-4000-8000-000000000016', 39000, 113)
RETURNING code;

-- @A
-- expect-rows: 1
INSERT INTO trip_events (trip_id, status)
VALUES ('a4000000-0000-4000-8000-000000400206', 'requested');

-- B "rút dây" của A: pg_terminate_backend(pid, 5000) gửi tín hiệu huỷ backend của A và CHỜ
-- tối đa 5 giây tới khi backend đó thoát hẳn (trả true). Cùng hậu quả với mất mạng, app chết,
-- server restart: kết nối mất trước COMMIT.
-- @B
-- lab: nhat-ky
-- expect-value: t
SELECT pg_terminate_backend(pid, 5000) FROM pg_stat_activity WHERE application_name = 'lab-A';

-- @A
-- A không hề biết, gửi COMMIT ⇒ psql phát hiện kết nối đã mất và thoát.
-- expect-mat-ket-noi
COMMIT;

-- @B
-- Không chuyến, không sự kiện: phần đã ghi của giao dịch dở bị huỷ.
-- expect-line: 0 | 0
SELECT (SELECT count(*) FROM trips WHERE id = 'a4000000-0000-4000-8000-000000400206') AS chuyen,
       (SELECT count(*) FROM trip_events WHERE trip_id = 'a4000000-0000-4000-8000-000000400206') AS su_kien;

-- Ứng dụng KHÔNG biết COMMIT đã tới server hay chưa ("lỗi kết nối" sau khi gửi COMMIT có
-- thể là commit xong rồi mới mất mạng). Cách an toàn: kết nối lại và ĐỌC trạng thái — hoặc
-- dùng idempotency key để gửi lại không sợ trùng (PG-039).
-- @A
SET search_path TO toinoi, public;

-- @A
-- expect-value: f
SELECT EXISTS (SELECT 1 FROM trips WHERE id = 'a4000000-0000-4000-8000-000000400206') AS da_luu;

-- Phiên "quên COMMIT": idle_in_transaction_session_timeout tự cắt kết nối giữ giao dịch mở
-- mà không làm gì quá lâu (giao dịch mở lâu giữ khoá và chặn VACUUM — P03 PG-026).
-- @A
SET idle_in_transaction_session_timeout = '200ms';

-- @A
BEGIN;

-- @A
-- expect-value: 400206
INSERT INTO trips (id, kind, city_id, customer_id, fare, pickup_area_id)
VALUES ('a4000000-0000-4000-8000-000000400207', 'ride', 1, 'c4000000-0000-4000-8000-000000000017', 43000, 111)
RETURNING code;

-- @B
-- B chờ 0,5 giây; trong lúc đó A ngồi yên trong giao dịch quá 200 ms ⇒ server cắt A.
-- lab: nhat-ky
SELECT pg_sleep(0.5);

-- @A
-- expect-mat-ket-noi
COMMIT;

-- @B
-- expect-value: 0
SELECT count(*) FROM trips WHERE id = 'a4000000-0000-4000-8000-000000400207';

-- ===========================================================================
-- §6. Bất biến cuối bài: mọi chuyến đều có sự kiện 'requested', không chuyến nửa vời.
-- Chuyến mới của bài: 400200, 400202, 400203, 400204 (400201, 400205, 400206 bị "đốt" bởi
-- các giao dịch đã huỷ).
-- @B
-- expect-line: 400200 | 1
-- expect-line: 400202 | 1
-- expect-line: 400203 | 1
-- expect-line: 400204 | 1
SELECT t.code, count(e.id) FILTER (WHERE e.status = 'requested') AS su_kien_requested
FROM trips AS t LEFT JOIN trip_events AS e ON e.trip_id = t.id
WHERE t.code >= 400200
GROUP BY t.code ORDER BY t.code;

-- @B
-- expect-value: 0
SELECT count(*) FROM trips AS t
WHERE NOT EXISTS (SELECT 1 FROM trip_events AS e WHERE e.trip_id = t.id AND e.status = 'requested');
