# Checklist review chức năng thanh toán — CL-PAY

- Chủ checklist: QA Lead nhóm Payment · Phạm vi: đặc tả, thiết kế, kiến trúc có thu/trả tiền qua cổng thanh toán
- Cách dùng: mỗi mục trả lời **Đạt / Phát hiện / Câu hỏi / Không áp dụng**, luôn kèm **bằng chứng** (mã R-…, mục kiến trúc).
  Checklist không bao trùm mọi thứ — thấy vấn đề ngoài danh sách vẫn ghi, rồi đề xuất thêm mục.
- Lịch sử: v1.0 (05/11/2026, 15 mục) · v1.1 (sau lần áp đầu tiên: thêm PAY-16)

| Mã | Nhóm | Câu hỏi | Vì sao có mục này (nguồn) |
|---|---|---|---|
| PAY-01 | Số tiền | Số tiền thu được định nghĩa rõ cho **từng** dịch vụ và từng khoản (cước, phí huỷ), truy được về nguồn (báo giá)? | Thu sai số tiền là lỗi tiền trực tiếp; đồ ăn có cấu trúc giá khác (R-FOOD-01) |
| PAY-02 | Số tiền | Số tiền gửi cổng luôn > 0? Ca 0 đ và số âm được xử lý thế nào? | Số âm có thể thành «hoàn tiền» ngoài ý muốn |
| PAY-03 | Số tiền | Đơn vị và làm tròn rõ ràng (số nguyên đồng, không số thực)? | R-GEN-01; lỗi làm tròn tích luỹ |
| PAY-04 | Idempotency | Mỗi khoản thu có idempotency key xác định, ổn định qua mọi lần thử lại? | Kiến trúc §4.9, §14: trùng giao dịch |
| PAY-05 | Idempotency | Client gửi lại (bấm hai lần, mạng chập chờn) không tạo khoản thu thứ hai? | R-IDEM; rủi ro hàng đầu của kiến trúc |
| PAY-06 | Idempotency | Hai khoản khác nhau của cùng một chuyến không dùng chung khoá? | Khoá chung ⇒ khoản sau bị coi là trùng, không thu |
| PAY-07 | Hết giờ | Có giá trị hết giờ cho mỗi lần gọi (3 giây) và thời gian xử lý tối đa của cả thao tác? | R-PAY-04, R-NFR-05 |
| PAY-08 | Thử lại | Phân biệt lỗi được thử lại (hết giờ, tạm thời) với lỗi không thử lại (bị từ chối); có giới hạn số lần? | Thử lại khi bị từ chối = spam cổng, có thể bị khoá |
| PAY-09 | Kết quả | Kết quả **không rõ** (hết giờ nhưng có thể đã trừ tiền) có trạng thái riêng, không gộp vào «thất bại»? | Cổng có thể trừ tiền rồi mất phản hồi (R-TEST-03 `lost_response`) |
| PAY-10 | Trạng thái | Mọi trạng thái thanh toán có định nghĩa, có bảng chuyển hợp lệ, biết trạng thái nào là kết thúc? | Trạng thái «trôi» khiến đối soát không khép được |
| PAY-11 | Đối soát | Có quy trình xử lý sau cho thất bại / chưa rõ: ai, khi nào, bằng cách nào? | Kiến trúc §8.3: đối soát định kỳ |
| PAY-12 | Nghiệp vụ | Thu tiền thất bại có chặn nghiệp vụ hay không — được ghi rõ, kèm hệ quả với khách/tài xế? | Chặn sai ⇒ tài xế không hoàn thành được chuyến |
| PAY-13 | Nguồn tin cậy | Không coi callback / thông báo từ client là bằng chứng đã thanh toán; trạng thái chỉ đổi theo kết quả phía server? | Kiến trúc §4.9 |
| PAY-14 | Nguồn tin cậy | Webhook của cổng được xác minh chữ ký, chống replay, xử lý trùng/trễ? | Kiến trúc §4.9, §14 |
| PAY-15 | Dữ liệu | Không lưu PAN/CVV; có lưu mã giao dịch phía cổng (external reference) để đối soát? | Kiến trúc §4.9 |
| PAY-16 *(v1.1)* | Idempotency | Khoá idempotency **duy nhất theo thời gian**: không lặp lại sau khi khởi động lại, reset dữ liệu, hay giữa các môi trường dùng chung sandbox cổng? | Phát hiện O-1 lần áp đầu tiên (id chuyến tuần tự của lab bắt đầu lại từ `id-0001`) |
