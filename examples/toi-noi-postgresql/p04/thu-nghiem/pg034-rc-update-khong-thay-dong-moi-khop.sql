-- Thí nghiệm PG-034 (hai phiên): bất thường của Read Committed khi CÂU GHI gặp dòng đang bị khoá.
--
-- B chạy "tăng cước 1.000 đ cho mọi chuyến đang chờ ở khu 113". Câu UPDATE chụp snapshot lúc
-- bắt đầu; gặp dòng đang bị A khoá thì CHỜ, rồi chỉ ĐỌC LẠI ĐÚNG DÒNG ĐÓ (EvalPlanQual) —
-- không quét lại cả bảng. Hệ quả:
--   * dòng A vừa đổi để KHÔNG còn khớp (400102 bị huỷ) ⇒ B bỏ qua — đúng;
--   * dòng A vừa đổi để TRỞ NÊN khớp (400101 chuyển sang khu 113) ⇒ B KHÔNG thấy — dù A commit
--     trước khi B kết thúc. Kết quả của B không ứng với BẤT KỲ snapshot nào. Repeatable Read
--     không có hiện tượng này: nó báo 40001 thay vì đọc lại (xem 34-…sql S3/S4).
-- Khu 113 lúc đầu có 3 chuyến chờ: 400102, 400105, 400108; 400101 ở khu 112.

-- @A
SET search_path TO toinoi, public;

-- @B
SET search_path TO toinoi, public;

-- @A
BEGIN;

-- @A
-- expect-rows: 1
UPDATE trips SET status = 'cancelled', updated_at = now() WHERE code = 400102;

-- @A
-- expect-rows: 1
UPDATE trips SET pickup_area_id = 113, updated_at = now() WHERE code = 400101;

-- @B
-- B gặp 400102 (đang bị A khoá) ⇒ chờ.
-- expect-blocked
-- expect-line: 400105
-- expect-line: 400108
UPDATE trips SET fare = fare + 1000 WHERE pickup_area_id = 113 AND status = 'requested'
RETURNING code;

-- @A
COMMIT;

-- (B chạy tiếp: 400102 đọc lại ⇒ đã huỷ ⇒ bỏ; 400101 không có trong lần quét của B ⇒ bỏ sót.)

-- @A
-- Sau khi cả hai commit: 400101 đang chờ ở khu 113 mà KHÔNG được tăng cước.
-- expect-line: 400101 | 31000
-- expect-line: 400105 | 36000
-- expect-line: 400108 | 39000
SELECT code, fare FROM trips WHERE pickup_area_id = 113 AND status = 'requested' ORDER BY code;
