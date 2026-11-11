-- PG-035 · Optimistic concurrency và cập nhật có điều kiện — `postgresql-optimistic-version-conditional-update`
--
-- Bài toán Tới Nơi: tổng đài viên [A] mở màn sửa chuyến 400108 (đọc), gõ cước mới trong một
-- phút rồi bấm Lưu. Trong phút đó khách [B] đã HUỶ chuyến. Màn sửa ghi lại MỌI trường của
-- form ⇒ chuyến "sống lại". Giữa lúc đọc và lúc ghi có khoảng hở rất dài (người gõ), không
-- thể giữ khoá dòng suốt khoảng đó. Cách lạc quan (optimistic): không khoá khi đọc, nhưng
-- lúc ghi chỉ ghi nếu dòng CHƯA ĐỔI kể từ khi đọc — kiểm bằng cột version (hoặc trạng thái),
-- và đọc SỐ DÒNG ĐÃ ĐỔI: 0 ⇒ có người sửa trước ⇒ báo xung đột, tải lại.
--
-- Biến psql: `SELECT … \gset` lưu kết quả vào biến của PHIÊN (:v, :'trang_thai'), giả lập
-- giá trị form giữ ở phía ứng dụng. `:ROW_COUNT` = số dòng câu trước đã đổi.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- ===========================================================================
-- §1. KHÔNG kiểm gì: ghi đè trạng thái (lost update).

-- @A
-- A mở form: đọc cước, trạng thái, version.
SELECT fare AS cuoc, status AS trang_thai, version AS v FROM trips WHERE code = 400108 \gset

-- @A
-- expect-line: 38000 | requested | 1
SELECT :cuoc AS cuoc, :'trang_thai' AS trang_thai, :v AS v;

-- @B
-- Khách huỷ chuyến.
-- expect-rows: 1
UPDATE trips SET status = 'cancelled', updated_at = now() WHERE code = 400108;

-- @A
-- A bấm Lưu: form gửi lại mọi trường, kể cả trạng thái đã đọc từ một phút trước.
-- expect-rows: 1
UPDATE trips SET fare = 45000, status = :'trang_thai', updated_at = now() WHERE code = 400108;

-- @B
-- Chuyến đã huỷ "sống lại" với trạng thái requested: lệnh huỷ của khách bị MẤT, không lỗi nào.
-- expect-line: 45000 | requested
SELECT fare, status FROM trips WHERE code = 400108;

-- Đặt lại để chạy lại cùng lịch với version.
-- @B
UPDATE trips SET fare = 38000, status = 'requested', version = 1 WHERE code = 400108;

-- ===========================================================================
-- §2. Optimistic: ghi kèm điều kiện version = <giá trị đã đọc>, rồi đọc số dòng đã đổi.

-- @A
SELECT fare AS cuoc, status AS trang_thai, version AS v FROM trips WHERE code = 400108 \gset

-- @B
-- Khách huỷ — cũng theo quy ước: tăng version mỗi lần sửa.
-- expect-line: cancelled | 2
UPDATE trips SET status = 'cancelled', updated_at = now(), version = version + 1
WHERE code = 400108 AND version = 1
RETURNING status, version;

-- @A
-- A lưu với version 1 đã đọc ⇒ không dòng nào khớp ⇒ 0 dòng đổi.
-- expect-rows: 0
UPDATE trips SET fare = 45000, updated_at = now(), version = version + 1
WHERE code = 400108 AND version = :v;

-- @A
-- Ứng dụng đọc số dòng đã đổi (JDBC: executeUpdate() trả số; PL/pgSQL: GET DIAGNOSTICS / FOUND).
-- 0 ⇒ báo "Chuyến đã được người khác sửa — tải lại" và đọc lại bản mới.
-- expect-value: 0
SELECT :ROW_COUNT AS so_dong_doi;

-- @A
-- expect-line: 38000 | cancelled | 2
SELECT fare, status, version FROM trips WHERE code = 400108;

-- ===========================================================================
-- §3. Hai lần lưu CHỒNG NHAU về thời gian (cùng đọc version 2): người sau phải chờ người
-- trước, rồi Read Committed ĐỌC LẠI dòng (version đã thành 3) ⇒ điều kiện sai ⇒ 0 dòng.
-- Kiểm version vẫn đúng khi hai câu UPDATE chạy đồng thời thật.

-- @A
BEGIN;

-- @A
-- expect-line: 3
UPDATE trips SET fare = 41000, version = version + 1 WHERE code = 400108 AND version = 2
RETURNING version;

-- @B
-- expect-blocked
-- expect-rows: 0
UPDATE trips SET fare = 47000, version = version + 1 WHERE code = 400108 AND version = 2;

-- @A
COMMIT;

-- @B
-- expect-line: 41000 | 3
SELECT fare, version FROM trips WHERE code = 400108;

-- ===========================================================================
-- §4. Cập nhật có điều kiện theo TRẠNG THÁI: chuyển trạng thái chỉ từ trạng thái hợp lệ.
-- Hai tài xế 9101 [A] và 9102 [B] cùng bấm "Nhận" chuyến 400109.

-- @A
-- expect-line: 400109 | accepted
UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
                 accepted_at = now(), updated_at = now(), version = version + 1
WHERE code = 400109 AND status = 'requested'
RETURNING code, status;

-- @B
-- expect-rows: 0
UPDATE trips SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009102',
                 accepted_at = now(), updated_at = now(), version = version + 1
WHERE code = 400109 AND status = 'requested'
RETURNING code, status;

-- @B
-- 0 dòng ⇒ ứng dụng báo B "chuyến đã có người nhận".
-- expect-value: 0
SELECT :ROW_COUNT AS so_dong_doi;

-- Khách chỉ được huỷ khi chưa đón: tập trạng thái hợp lệ nằm NGAY trong WHERE.
-- @B
-- expect-line: cancelled
UPDATE trips SET status = 'cancelled', updated_at = now(), version = version + 1
WHERE code = 400109 AND status IN ('requested', 'searching', 'accepted', 'arriving')
RETURNING status;

-- @A
-- Tài xế bấm "Đã đón" sau khi khách huỷ ⇒ 0 dòng (không chuyển từ cancelled sang picked_up).
-- expect-rows: 0
UPDATE trips SET status = 'picked_up', updated_at = now(), version = version + 1
WHERE code = 400109 AND status IN ('accepted', 'arriving');

-- ===========================================================================
-- §5. Quên tăng version ở MỘT đường ghi là đủ làm hỏng cơ chế. Trigger BEFORE UPDATE tự
-- tăng version cho MỌI câu UPDATE — kể cả câu không nhắc tới cột version.

-- @A
CREATE FUNCTION tang_version() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
  NEW.version := OLD.version + 1;
  RETURN NEW;
END
$$;

-- @A
CREATE TRIGGER trg_trips_tang_version BEFORE UPDATE ON trips
FOR EACH ROW EXECUTE FUNCTION tang_version();

-- @B
-- expect-value: 1
SELECT version FROM trips WHERE code = 400110;

-- @B
-- Câu ghi "quên" version vẫn làm version tăng (1 → 2).
-- expect-value: 2
UPDATE trips SET fare = fare + 1000 WHERE code = 400110 RETURNING version;

-- @B
-- Câu ghi đúng quy ước (version = version + 1) cũng chỉ tăng 1, không phải 2.
-- expect-value: 3
UPDATE trips SET fare = fare + 1000, version = version + 1 WHERE code = 400110 AND version = 2
RETURNING version;
