# P04 · Gói yêu cầu điểm đón — bài tập thiết kế

Đây là **đề xuất giả lập**, không phải tính năng đã có hoặc yêu cầu đã duyệt. SPEC lab R-GEN-06 chỉ nhận distanceKm, không có điểm đón/đến. Không dùng API lab để tuyên bố các scenario dưới đã chạy được.

## Phạm vi thiết kế chung

- Khách chủ lượt xem và xác nhận mô tả điểm đón trước khi được tài xế nhận; bản đề xuất giới hạn thao tác ghi ở `searching`. Không đổi vòng đời 8 trạng thái.
- Đọc/ghi qua dữ liệu chuẩn phía hệ thống; mỗi lần sửa tăng `pickupRevision` (trường mới đề xuất). Xác nhận gắn với revision khách vừa xem. Nếu revision/state đã đổi, không xác nhận dữ liệu cũ; tải lại và yêu cầu xem lại.
- Xác nhận ở đây nghĩa là khách đã xác nhận thông tin hiển thị, **không** có nghĩa tài xế đã đọc, đã đồng ý hoặc đã đón khách. Không đổi trip state sang picked_up.
- Sau nhận chuyến, tài xế được gán có thể xem thông tin theo quyền đề xuất. Sửa sau nhận, chat, gửi ảnh và đổi tuyến nằm ngoài lát cắt đầu.
- Mất mạng/timeout: không hiển thị thành công khi chưa biết kết quả chuẩn. Retry cần định danh yêu cầu ổn định và đối soát; không tự bịa endpoint/cơ chế đã có trong lab.
- Lưu thành công trước khi state chuyển accepted có thể hợp lệ; accepted thắng trước thì lệnh ghi bị từ chối. Kiểm state và revision phải cùng một quyết định nhất quán. Đây là hợp đồng thiết kế cần review, không chọn cơ chế khoá cụ thể.

## ID truy vết

| ID | Nội dung | Trạng thái |
|---|---|---|
| NEED-P04-01 | Hai bên có tham chiếu điểm đón rõ ràng | Giả thuyết nhu cầu cần xác nhận từ P02 |
| US-P04-01 | Khách xem và xác nhận đúng thông tin mình cung cấp | Đề xuất |
| US-P04-02 | Tài xế được gán xem thông tin điểm đón | Đề xuất |
| RULE-P04-01 | Chỉ khách chủ lượt được ghi khi searching | Đề xuất |
| RULE-P04-02 | Xác nhận chỉ áp dụng revision đã xem; sửa tạo revision mới | Đề xuất |
| RULE-P04-03 | Thành công chỉ khi nguồn chuẩn xác nhận | Đề xuất |
| UC-P04-01 | Xác nhận thông tin điểm đón | Đề xuất |
| AC-P04-01 | Chủ lượt, searching, revision khớp → ghi nhận xác nhận đúng revision | Đề xuất |
| AC-P04-02 | Revision khác → không xác nhận bản cũ, yêu cầu xem lại | Đề xuất |
| AC-P04-03 | State đã accepted → từ chối ghi, đọc lại trạng thái | Đề xuất |
| AC-P04-04 | Không đúng quyền → không ghi/không lộ dữ liệu ngoài quyền | Đề xuất |
| AC-P04-05 | Timeout → chưa biết kết quả, kiểm tra lại theo định danh ổn định | Đề xuất |

## Case dùng chung

Khách K sở hữu lượt B đang searching; mô tả “Cổng số 2”; revision 4; chưa xác nhận. Các case độc lập: xác nhận r4; màn hình r4 nhưng server r5; server đã accepted; người X không sở hữu; request đã ghi nhưng response mất; ghi chú rỗng. Với case rỗng, cần quyết định nội dung nào bắt buộc — chưa tự cho phép hoặc từ chối.

## NFR-P04-01 — ngưỡng minh hoạ

Đề xuất thời gian từ thao tác mở chi tiết tới nội dung điểm đón dùng được: p95 ≤ 2 giây trên lần mở thành công; tỷ lệ lỗi ≤ 1% trên tất cả lần mở. Workload 100 phiên đồng thời, mỗi phiên mở một lượt mỗi 10 giây trong 15 phút; phân biệt 100 phiên với 100 request/s. Tối đa 5 giây mỗi lần mở rồi phân loại lỗi nếu không có kết quả dùng được. Cần chốt thiết bị, mạng, dữ liệu, warm-up, cache, công cụ, sample và cách thống kê trước nghiệm thu. Con số không thay SLO lab/production.

## Bản nháp có lỗi để review BA-032

“Mọi khách đều sửa điểm đón bất kỳ lúc nào. App lưu ngay và báo thành công; tài xế sẽ thấy tức thì. Xác nhận điểm đón chuyển chuyến sang picked_up. Tải phải nhanh, an toàn. Nếu lỗi thì retry tới khi được. MVP làm API trước, UI để sau. Phí huỷ mới ba phút áp dụng luôn.”

Tìm ít nhất tám vấn đề; mỗi finding có trigger, nguồn, ảnh hưởng, sửa đề xuất và cách kiểm. CR-BA-01 của P03 chưa được duyệt, không thuộc phạm vi đề xuất P04.

## Mẫu quyết định

| ID | Câu hỏi | Phương án/ảnh hưởng | Người có quyền chốt | Trạng thái/hiệu lực |
|---|---|---|---|---|
| OPEN-01 | Nội dung nào bắt buộc? | | | Chưa chốt |
| OPEN-02 | Chọn định danh, retry và retention ra sao? | | | Chưa chốt |
| OPEN-03 | NFR đo trên cấu hình nào? | | | Chưa chốt |
