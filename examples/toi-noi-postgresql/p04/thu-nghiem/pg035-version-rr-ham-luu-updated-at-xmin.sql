-- Thí nghiệm PG-035 (hai phiên): bốn điểm bổ sung cho 35-optimistic-version.sql.
--   §1. Cùng lịch "hai lần lưu chồng nhau" (35 §3) nhưng B chạy REPEATABLE READ ⇒ B KHÔNG nhận
--       0 dòng mà nhận lỗi 40001 — ứng dụng phải xử lý CẢ HAI tín hiệu xung đột.
--   §2. Hàm lưu có kiểm version, phân biệt "đã lưu" / "xung đột" / "không tồn tại" (lời giải bài tập).
--   §3. Bẫy dùng updated_at = now() làm version: now() là giờ BẮT ĐẦU giao dịch ⇒ giao dịch dài
--       ghi một mốc CŨ HƠN mốc đang có — updated_at đi lùi.
--   §4. Cột hệ thống xmin làm "version" có sẵn: đổi theo MỌI câu UPDATE; khoá dòng (FOR NO KEY UPDATE) không đổi nó.
-- Chuyến dùng: 400108 (§1), 400105 (§2), 400107 (§3), 400106 (§4) — đều version 1 lúc đầu.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- ===========================================================================
-- §1. Repeatable Read: người lưu sau chờ, rồi 40001 thay vì 0 dòng.

-- @A
BEGIN;

-- @A
-- expect-value: 2
UPDATE trips SET fare = 41000, version = version + 1 WHERE code = 400108 AND version = 1
RETURNING version;

-- @B
BEGIN ISOLATION LEVEL REPEATABLE READ;

-- @B
-- Snapshot của B chụp lúc câu này bắt đầu (A chưa commit) ⇒ B thấy version 1, dòng đang bị A
-- khoá ⇒ chờ. A commit đã SỬA dòng ⇒ Repeatable Read không được đọc lại ⇒ 40001.
-- expect-blocked
-- expect-error: 40001
UPDATE trips SET fare = 47000, version = version + 1 WHERE code = 400108 AND version = 1;

-- @A
COMMIT;

-- @B
ROLLBACK;

-- @B
-- Thử lại TOÀN BỘ: đọc lại bản mới (version 2) ⇒ form báo xung đột cho người dùng quyết.
-- expect-line: 41000 | 2
SELECT fare, version FROM trips WHERE code = 400108;

-- ===========================================================================
-- §2. Hàm lưu cước có kiểm version. 0 dòng có HAI nghĩa (sai version / không có chuyến) —
-- đọc lại để phân biệt, trả version mới khi thành công.

-- @A
CREATE FUNCTION luu_cuoc(p_code bigint, p_fare integer, p_version integer) RETURNS text
LANGUAGE plpgsql AS $$
DECLARE
  v_moi integer;
  v_hien_tai integer;
BEGIN
  UPDATE trips SET fare = p_fare, updated_at = now(), version = version + 1
  WHERE code = p_code AND version = p_version
  RETURNING version INTO v_moi;
  IF FOUND THEN
    RETURN 'da_luu version=' || v_moi;
  END IF;
  SELECT version INTO v_hien_tai FROM trips WHERE code = p_code;
  IF NOT FOUND THEN
    RETURN 'khong_ton_tai';
  END IF;
  RETURN 'xung_dot version_hien_tai=' || v_hien_tai;
END
$$;

-- @A
-- expect-value: da_luu version=2
SELECT luu_cuoc(400105, 40000, 1);

-- @B
-- B vẫn giữ version 1 đã đọc trước khi A lưu.
-- expect-value: xung_dot version_hien_tai=2
SELECT luu_cuoc(400105, 42000, 1);

-- @B
-- expect-value: khong_ton_tai
SELECT luu_cuoc(499999, 42000, 1);

-- @B
-- expect-line: 40000 | 2
SELECT fare, version FROM trips WHERE code = 400105;

-- ===========================================================================
-- §3. updated_at làm version: now() cố định ở đầu giao dịch.

-- @A
BEGIN;

-- @A
-- A bắt đầu giao dịch (now() của A chốt ở đây), rồi làm việc khác một lúc…
-- expect-value: 1
SELECT version FROM trips WHERE code = 400107;

-- @B
-- …trong lúc đó B sửa và commit (autocommit) — now() của B MUỘN hơn của A.
-- expect-rows: 1
UPDATE trips SET fare = fare + 500, updated_at = now() WHERE code = 400107;

-- @B
SELECT updated_at AS moc_b FROM trips WHERE code = 400107 \gset

-- @A
-- expect-rows: 1
UPDATE trips SET fare = fare + 1000, updated_at = now() WHERE code = 400107;

-- @A
COMMIT;

-- @B
-- Lần sửa SAU (của A) ghi mốc NHỎ HƠN lần sửa trước (của B): updated_at đi lùi.
-- expect-line: 38500 | t
SELECT fare, updated_at < :'moc_b'::timestamptz AS di_lui FROM trips WHERE code = 400107;

-- ===========================================================================
-- §4. xmin = xid của giao dịch đã ghi phiên bản dòng hiện tại.

-- @A
-- A mở form, giữ xmin như một "version".
SELECT xmin AS xm FROM trips WHERE code = 400106 \gset

-- @B
-- Khoá dòng (không sửa) không đổi xmin — chỉ ghi dấu khoá vào xmax. (Biến \gset thuộc
-- từng phiên ⇒ B tự đọc xmin của mình.)
SELECT xmin AS xm FROM trips WHERE code = 400106 \gset

-- @B
BEGIN;

-- @B
-- expect-value: 36000
SELECT fare FROM trips WHERE code = 400106 FOR NO KEY UPDATE;

-- @B
-- expect-value: t
SELECT xmin = :'xm'::xid FROM trips WHERE code = 400106;

-- @B
COMMIT;

-- @B
-- Câu UPDATE "không đổi gì" (fare = fare) vẫn tạo phiên bản dòng mới ⇒ xmin đổi.
-- expect-rows: 1
UPDATE trips SET fare = fare WHERE code = 400106;

-- @A
-- A lưu với xmin cũ ⇒ 0 dòng, dù dữ liệu nghiệp vụ không ai đổi.
-- expect-rows: 0
UPDATE trips SET fare = 50000 WHERE code = 400106 AND xmin = :'xm'::xid;

-- @A
-- expect-line: 36000 | 1
SELECT fare, version FROM trips WHERE code = 400106;
