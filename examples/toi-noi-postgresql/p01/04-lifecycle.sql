-- P01 · 04-lifecycle.sql — vòng đời một chuyến bằng INSERT / UPDATE / DELETE
-- … RETURNING (PG-006).
--
-- Kịch bản: 09:00 ngày 03/11, khách Phạm Thu Dung (Hà Nội) đặt xe máy. Tài xế
-- Ngô Quang Minh (mã 8003, đang online) nhận chuyến và chở khách tới nơi.
-- Mỗi lần đổi trạng thái = một UPDATE trips + một dòng trip_events.
--
-- Chuyến mới mang UUID do "ứng dụng" sinh sẵn (b…0001) — cách làm phổ biến khi
-- app cần biết id trước khi gửi xuống database; nhờ đó các câu sau tìm đúng
-- chuyến mà không cần đoán mã. Mã chuyến (code) vẫn do IDENTITY cấp.
--
-- Mốc thời gian ghi tay để kết quả ổn định; ứng dụng thật dùng now().
-- Mỗi câu ở đây tự commit riêng. Ghi trips và trip_events trong CÙNG một giao
-- dịch là chủ đề của P04 (PG-031); câu cuối của phần 1 cho thấy cách gộp
-- bằng một câu lệnh duy nhất.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ===========================================================================
-- Phần 1 · Chuyến đi trọn vòng đời
-- ===========================================================================

-- 1. Khách đặt: chỉ ghi những gì đã biết; status, accepted_at… dùng DEFAULT/NULL.
--    RETURNING trả lại các cột database tự điền (code, status) ngay trong câu
--    INSERT — không cần SELECT thêm lần nữa.
--    Mã chuyến KHÔNG phải 100015: 03-fixture-invalid.sql đã gọi IDENTITY nhiều
--    lần trong các câu thất bại, mà số đã cấp thì không được trả lại.
-- expect-rows: 1
INSERT INTO trips (id, kind, city_id, customer_id, fare, requested_at, updated_at)
VALUES ('b0000000-0000-4000-8000-000000000001', 'ride', 2, 'c0000000-0000-4000-8000-000000000004',
        54000, '2026-11-03 09:00:00+07', '2026-11-03 09:00:00+07')
RETURNING code, kind, status, driver_id, fare, requested_at;

-- expect-rows: 1
INSERT INTO trip_events (trip_id, status, occurred_at)
VALUES ('b0000000-0000-4000-8000-000000000001', 'requested', '2026-11-03 09:00:00+07')
RETURNING id, status, occurred_at;

-- 2. Hệ thống bắt đầu tìm tài xế. Điều kiện `AND status = 'requested'` bảo đảm
--    chỉ chuyển từ đúng trạng thái trước đó.
-- expect-rows: 1
UPDATE trips
SET status = 'searching', updated_at = '2026-11-03 09:00:05+07'
WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'requested'
RETURNING code, status, updated_at;

INSERT INTO trip_events (trip_id, status, occurred_at)
VALUES ('b0000000-0000-4000-8000-000000000001', 'searching', '2026-11-03 09:00:05+07');

-- 3. Tài xế 8003 bấm nhận. Chỉ thành công khi chuyến vẫn đang `searching`.
-- expect-rows: 1
UPDATE trips
SET status      = 'accepted',
    driver_id   = 'd0000000-0000-4000-8000-000000008003',
    accepted_at = '2026-11-03 09:01:00+07',
    updated_at  = '2026-11-03 09:01:00+07'
WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'searching'
RETURNING code, status, driver_id, accepted_at;

INSERT INTO trip_events (trip_id, status, occurred_at)
VALUES ('b0000000-0000-4000-8000-000000000001', 'accepted', '2026-11-03 09:01:00+07');

-- Tài xế chuyển sang bận — cũng có điều kiện: chỉ từ online.
-- expect-rows: 1
UPDATE drivers SET status = 'busy'
WHERE code = 8003 AND status = 'online'
RETURNING code, full_name, status;

-- 4. Một giây sau, tài xế 8001 cũng bấm nhận CÙNG chuyến. Chuyến đã `accepted`
--    nên điều kiện `status = 'searching'` không còn đúng: UPDATE 0, không dòng
--    nào bị sửa, tài xế 8003 giữ nguyên chuyến. Ứng dụng đọc "0 dòng" để báo
--    "chuyến đã có người nhận".
--    Nếu bỏ điều kiện status, câu này sẽ GHI ĐÈ tài xế 8003 — lỗi "lost update".
--    (Hai tài xế bấm trong cùng một khoảnh khắc thật sự: xem P04, PG-035/PG-040.)
-- expect-rows: 0
UPDATE trips
SET status      = 'accepted',
    driver_id   = 'd0000000-0000-4000-8000-000000008001',
    accepted_at = '2026-11-03 09:01:01+07',
    updated_at  = '2026-11-03 09:01:01+07'
WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'searching'
RETURNING code, status, driver_id;

-- 5. Tài xế đang tới đón → đã đón → đang chở.
-- expect-rows: 1
UPDATE trips SET status = 'arriving', updated_at = '2026-11-03 09:01:30+07'
WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'accepted'
RETURNING code, status, updated_at;

INSERT INTO trip_events (trip_id, status, occurred_at)
VALUES ('b0000000-0000-4000-8000-000000000001', 'arriving', '2026-11-03 09:01:30+07');

-- expect-rows: 1
UPDATE trips SET status = 'picked_up', updated_at = '2026-11-03 09:07:00+07'
WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'arriving'
RETURNING code, status, updated_at;

INSERT INTO trip_events (trip_id, status, occurred_at)
VALUES ('b0000000-0000-4000-8000-000000000001', 'picked_up', '2026-11-03 09:07:00+07');

-- expect-rows: 1
UPDATE trips SET status = 'in_transit', updated_at = '2026-11-03 09:07:20+07'
WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'picked_up'
RETURNING code, status, updated_at;

INSERT INTO trip_events (trip_id, status, occurred_at)
VALUES ('b0000000-0000-4000-8000-000000000001', 'in_transit', '2026-11-03 09:07:20+07');

-- 6. Hoàn thành: thử quên ghi completed_at ⇒ CHECK chặn lại.
-- expect-error: 23514 ck_trips_completed_at_by_status
UPDATE trips SET status = 'completed', updated_at = '2026-11-03 09:31:00+07'
WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'in_transit';

-- 7. Hoàn thành đúng cách, GỘP hai việc trong MỘT câu lệnh: UPDATE … RETURNING
--    nằm trong WITH, kết quả của nó đi thẳng vào INSERT trip_events. Một câu
--    lệnh luôn thành công hoặc thất bại trọn vẹn, nên không thể có chuyến
--    `completed` mà thiếu dòng lịch sử. Thẻ lệnh in ra là của câu INSERT ngoài.
-- expect-rows: 1
WITH done AS (
  UPDATE trips
  SET status       = 'completed',
      completed_at = '2026-11-03 09:31:00+07',
      updated_at   = '2026-11-03 09:31:00+07'
  WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'in_transit'
  RETURNING id, status, completed_at
)
INSERT INTO trip_events (trip_id, status, occurred_at)
SELECT id, status, completed_at FROM done
RETURNING id, status, occurred_at;

-- expect-rows: 1
UPDATE drivers SET status = 'online'
WHERE code = 8003 AND status = 'busy'
RETURNING code, status;

-- 8. Đọc lại: trạng thái cuối và lịch sử (sắp theo thời điểm, hoà thì theo id).
-- expect-rows: 1
SELECT code, status, fare, requested_at, accepted_at, completed_at,
       completed_at - requested_at AS tong_thoi_gian
FROM trips
WHERE id = 'b0000000-0000-4000-8000-000000000001';

-- expect-rows: 7
SELECT id, status, occurred_at
FROM trip_events
WHERE trip_id = 'b0000000-0000-4000-8000-000000000001'
ORDER BY occurred_at, id;

-- ===========================================================================
-- Phần 2 · Bản nháp đặt nhầm: huỷ rồi xoá
-- ===========================================================================
-- Tới Nơi KHÔNG xoá chuyến thật (cần cho đối soát, khiếu nại, thống kê):
-- chuyến bị huỷ giữ trạng thái `cancelled`. DELETE chỉ dành cho dữ liệu tạo
-- nhầm/thử nghiệm — và ngay cả khi đó, khoá ngoại vẫn bảo vệ lịch sử.

-- expect-rows: 1
INSERT INTO trips (id, kind, city_id, customer_id, fare, requested_at, updated_at)
VALUES ('b0000000-0000-4000-8000-000000000002', 'parcel', 1, 'c0000000-0000-4000-8000-000000000006',
        25000, '2026-11-03 09:10:00+07', '2026-11-03 09:10:00+07')
RETURNING code, kind, status;

INSERT INTO trip_events (trip_id, status, occurred_at)
VALUES ('b0000000-0000-4000-8000-000000000002', 'requested', '2026-11-03 09:10:00+07');

-- Huỷ được khi chưa ai nhận (requested hoặc searching).
-- expect-rows: 1
UPDATE trips SET status = 'cancelled', updated_at = '2026-11-03 09:10:40+07'
WHERE id = 'b0000000-0000-4000-8000-000000000002' AND status IN ('requested', 'searching')
RETURNING code, status, updated_at;

INSERT INTO trip_events (trip_id, status, occurred_at)
VALUES ('b0000000-0000-4000-8000-000000000002', 'cancelled', '2026-11-03 09:10:40+07');

-- Xoá chuyến khi còn dòng lịch sử trỏ tới nó ⇒ khoá ngoại chặn (NO ACTION).
-- expect-error: 23503 fk_trip_events_trip
DELETE FROM trips
WHERE id = 'b0000000-0000-4000-8000-000000000002' AND status = 'cancelled';

-- Xoá lịch sử trước (RETURNING cho biết đã xoá những gì), rồi mới xoá chuyến.
-- expect-rows: 2
DELETE FROM trip_events
WHERE trip_id = 'b0000000-0000-4000-8000-000000000002'
RETURNING status, occurred_at;

-- expect-rows: 1
DELETE FROM trips
WHERE id = 'b0000000-0000-4000-8000-000000000002' AND status = 'cancelled'
RETURNING code, kind, status, fare;

-- Mã của chuyến vừa xoá không được dùng lại: sequence `trips_code_seq` (do
-- IDENTITY tự tạo) đã cấp tới số đó, chuyến tiếp theo nhận số kế tiếp. Dãy mã
-- có "lỗ" (từ các câu lỗi ở 03 và chuyến vừa xoá) — mã là để tra cứu, không
-- phải để đếm số chuyến.
SELECT max(code) AS ma_lon_nhat,
       count(*) AS so_chuyen,
       (SELECT last_value FROM trips_code_seq) AS so_cap_gan_nhat
FROM trips;
