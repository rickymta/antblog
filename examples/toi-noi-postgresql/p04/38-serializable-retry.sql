-- PG-038 · Serializable và retry toàn giao dịch — `postgresql-serializable-retry`
--
-- Bài toán Tới Nơi: quy tắc điều phối "mỗi khu luôn còn ÍT NHẤT MỘT tài xế trực". Ứng dụng
-- cho tài xế xin nghỉ nếu khu còn ≥ 2 người trực (đếm, rồi mới tắt trực). Khu SG-TT-01 (111)
-- có hai người trực: 9101 [A] và 9102 [B]. Hai người cùng bấm "Xin nghỉ" cùng lúc:
-- mỗi giao dịch ĐỌC thấy 2 người, rồi GHI dòng CỦA MÌNH — hai dòng khác nhau, không ai khoá
-- ai. Kết quả: khu không còn ai trực. Đây là WRITE SKEW: mỗi giao dịch đúng khi chạy một mình,
-- chạy xen kẽ thì phá bất biến. Repeatable Read không chặn được; SERIALIZABLE thì có — bằng
-- cách huỷ một giao dịch (40001), và ỨNG DỤNG phải chạy lại TOÀN BỘ giao dịch đó.
--
-- Phép thử ĐỒNG THỜI có retry tự động phía client (giới hạn số lần + backoff), đếm số lần
-- thử lại và kiểm bất biến sau nhiều lượt: p04/dong-thoi.mjs, kịch bản K038.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @C
SET search_path TO toinoi, public;

-- @C
-- Hàm đếm số khu VI PHẠM bất biến (khu có tài xế trực mà không ai đang trực).
CREATE FUNCTION khu_vi_pham() RETURNS bigint LANGUAGE sql STABLE AS $$
  SELECT count(*) FROM (SELECT area_id FROM driver_duty GROUP BY area_id
                        HAVING count(*) FILTER (WHERE on_duty) = 0) AS k
$$;

-- ###########################################################################
-- §1. REPEATABLE READ: write skew lọt qua.
-- ###########################################################################

-- @A
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @A
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @B
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @B
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @A
-- 2 ≥ 2 ⇒ được nghỉ.
-- expect-rows: 1
UPDATE driver_duty SET on_duty = false, changed_at = now()
WHERE driver_id = 'd4000000-0000-4000-8000-000000009101';

-- @B
-- Dòng của B khác dòng của A ⇒ không chờ, không xung đột cập nhật.
-- expect-rows: 1
UPDATE driver_duty SET on_duty = false, changed_at = now()
WHERE driver_id = 'd4000000-0000-4000-8000-000000009102';

-- @A
COMMIT;

-- @B
-- expect-match: ^COMMIT$
COMMIT;

-- @C
-- Khu 111 không còn ai trực: bất biến bị phá, không lỗi nào báo.
-- expect-line: 0 | 1
SELECT (SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty) AS dang_truc_khu_111,
       khu_vi_pham() AS so_khu_vi_pham;

-- @C
UPDATE driver_duty SET on_duty = true WHERE area_id = 111;

-- ###########################################################################
-- §2. SERIALIZABLE: cùng lịch ⇒ giao dịch commit sau bị huỷ (40001).
-- ###########################################################################

-- @A
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @A
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @B
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @B
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @C
-- SSI (Serializable Snapshot Isolation) ghi lại MỌI THỨ mỗi giao dịch đã ĐỌC bằng khoá vị
-- từ SIReadLock — không chặn ai, chỉ để phát hiện xung đột đọc–ghi. Bảng nhỏ ⇒ planner chọn
-- Seq Scan ⇒ khoá cả BẢNG (locktype relation).
-- expect-line: A | relation | driver_duty | SIReadLock | t
-- expect-line: B | relation | driver_duty | SIReadLock | t
SELECT * FROM lab_khoa() WHERE che_do = 'SIReadLock';

-- @A
-- expect-rows: 1
UPDATE driver_duty SET on_duty = false, changed_at = now()
WHERE driver_id = 'd4000000-0000-4000-8000-000000009101';

-- @B
-- SERIALIZABLE không CHỜ ai: B ghi xong ngay…
-- expect-rows: 1
UPDATE driver_duty SET on_duty = false, changed_at = now()
WHERE driver_id = 'd4000000-0000-4000-8000-000000009102';

-- @A
-- expect-match: ^COMMIT$
COMMIT;

-- @B
-- …nhưng lúc COMMIT thì bị từ chối: A đã đọc thứ B ghi, B đã đọc thứ A ghi, và A commit
-- trước ⇒ không có thứ tự chạy LẦN LƯỢT nào cho ra kết quả này ⇒ huỷ B.
-- expect-error: 40001
COMMIT;

-- @C
-- expect-line: 1 | 0
SELECT (SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty) AS dang_truc_khu_111,
       khu_vi_pham() AS so_khu_vi_pham;

-- Retry: ứng dụng chạy lại TOÀN BỘ giao dịch của B từ đầu — đọc lại, quyết định lại. Không
-- được chỉ gửi lại câu UPDATE (quyết định "được nghỉ" dựa trên lần đọc cũ, giờ đã sai).
-- @B
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @B
-- Lần thử 2: chỉ còn 1 người trực ⇒ ứng dụng TỪ CHỐI cho nghỉ ⇒ ROLLBACK (không ghi gì).
-- expect-value: 1
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @B
ROLLBACK;

-- @C
UPDATE driver_duty SET on_duty = true WHERE area_id = 111;

-- ###########################################################################
-- §3. Báo động GIẢ: hai tài xế ở HAI KHU KHÁC NHAU (112: 9103, 113: 9105) — không hề
-- xung đột về nghiệp vụ — nhưng khoá vị từ mức cả bảng (Seq Scan) vẫn làm B bị huỷ.
-- ###########################################################################

-- @A
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @A
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 112 AND on_duty;

-- @A
UPDATE driver_duty SET on_duty = false WHERE driver_id = 'd4000000-0000-4000-8000-000000009103';

-- @B
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @B
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 113 AND on_duty;

-- @B
UPDATE driver_duty SET on_duty = false WHERE driver_id = 'd4000000-0000-4000-8000-000000009105';

-- @A
COMMIT;

-- @B
-- expect-error: 40001
COMMIT;

-- @C
UPDATE driver_duty SET on_duty = true WHERE area_id IN (112, 113);

-- Cùng lịch, nhưng truy vấn đi qua index ix_driver_duty_area ⇒ khoá vị từ chỉ phủ page
-- index + các dòng đã đọc ⇒ hai khu không còn "chạm" nhau ⇒ cả hai commit. (Trên bảng lớn
-- planner tự chọn index; ở đây ép bằng enable_seqscan = off cho bảng 24 dòng.)

-- @A
SET enable_seqscan = off;

-- @B
SET enable_seqscan = off;

-- @A
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @A
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 112 AND on_duty;

-- @A
UPDATE driver_duty SET on_duty = false WHERE driver_id = 'd4000000-0000-4000-8000-000000009103';

-- @B
BEGIN ISOLATION LEVEL SERIALIZABLE;

-- @B
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 113 AND on_duty;

-- @B
UPDATE driver_duty SET on_duty = false WHERE driver_id = 'd4000000-0000-4000-8000-000000009105';

-- @C
-- Không còn khoá cả bảng: khoá trên page index và trên từng dòng đã đọc.
-- expect-no-match: \|\s+relation\s+\|\s+driver_duty\s+\|
SELECT * FROM lab_khoa() WHERE che_do = 'SIReadLock';

-- @A
-- expect-match: ^COMMIT$
COMMIT;

-- @B
-- expect-match: ^COMMIT$
COMMIT;

-- @A
RESET enable_seqscan;

-- @B
RESET enable_seqscan;

-- @C
-- expect-line: 112 | 1
-- expect-line: 113 | 1
SELECT area_id, count(*) FILTER (WHERE on_duty) FROM driver_duty WHERE area_id IN (112, 113)
GROUP BY area_id ORDER BY area_id;

-- @C
UPDATE driver_duty SET on_duty = true WHERE area_id IN (112, 113);

-- ###########################################################################
-- §4. Cách khác không cần SERIALIZABLE: "vật chất hoá xung đột" — mọi giao dịch đổi người
-- trực của một khu đều KHOÁ DÒNG KHU (areas) trước khi đếm ⇒ hai yêu cầu cùng khu xếp hàng.
-- Đúng ở Read Committed, không cần retry; đổi lại phải nhớ khoá ở MỌI đường ghi.
-- ###########################################################################

-- @A
BEGIN;

-- @A
-- expect-value: 111
SELECT id FROM areas WHERE id = 111 FOR NO KEY UPDATE;

-- @A
-- expect-value: 2
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @B
BEGIN;

-- @B
-- expect-blocked
-- expect-value: 111
SELECT id FROM areas WHERE id = 111 FOR NO KEY UPDATE;

-- @A
UPDATE driver_duty SET on_duty = false WHERE driver_id = 'd4000000-0000-4000-8000-000000009101';

-- @A
COMMIT;

-- @B
-- B đếm SAU khi A commit (câu mới ở Read Committed ⇒ snapshot mới) ⇒ 1 ⇒ từ chối.
-- expect-value: 1
SELECT count(*) FROM driver_duty WHERE area_id = 111 AND on_duty;

-- @B
ROLLBACK;

-- @C
-- Bất biến cuối bài.
-- expect-value: 0
SELECT khu_vi_pham();
