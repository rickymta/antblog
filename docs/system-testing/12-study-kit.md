# SYS-12 — Bộ tài liệu thực hành usability study

Trạng thái: sẵn sàng pilot nội bộ; **chưa tuyển người, chưa chạy phiên nghiên cứu**. Mẫu dưới để trống có chủ đích. Không dùng lượt Playwright thay participant.

## Kế hoạch phiên

- Câu hỏi: khách có phân biệt báo giá với đặt chuyến; có hiểu và sửa lỗi quãng đường; có xác nhận được đã huỷ và số phí?
- Đối tượng: người có khả năng dùng dịch vụ đặt xe, gồm người mới/đã quen ứng dụng tương tự. Ghi mức kinh nghiệm, thiết bị và nhu cầu hỗ trợ; tránh chỉ dùng thành viên làm sản phẩm.
- Vòng formative đề xuất: 6 người, phân bổ 3 mới/3 quen để học vấn đề ban đầu; không phải cỡ mẫu bảo đảm phát hiện mọi lỗi hay ước lượng tỷ lệ toàn dân. Pilot một đồng nghiệp, dữ liệu pilot tách riêng.
- Thời lượng dự kiến 30–40 phút: giới thiệu 5, ba nhiệm vụ 15–20, hỏi lại 10, dự phòng 5. Cho phép nghỉ/dừng; không thúc ép vì giới hạn thời gian.
- Vai trò: moderator trung lập, note-taker, observer không hướng dẫn trực tiếp. Dùng ID P01…; không ghi tên/số điện thoại vào worksheet.
- Môi trường: local lab dữ liệu giả, browser/version/viewport được ghi. Đặt lại seed và giờ 09:00 VN trước từng nhiệm vụ độc lập. Không nối thanh toán thật.

## Chuẩn bị và đồng ý tham gia

Giải thích mục đích, tự nguyện, quyền bỏ câu hỏi/dừng, dữ liệu sẽ lưu và ai xem. Xin đồng ý riêng nếu ghi âm/màn hình; không có đồng ý thì ghi chép theo phương án đã thống nhất. Nhóm nghiên cứu phải chốt nơi lưu và hạn xoá trước tuyển, không để recording vô thời hạn. Cho participant xem tài liệu đồng ý và hỏi lại; mẫu này không thay quy trình tổ chức của đơn vị.

Script mở đầu: “Hôm nay chúng tôi muốn tìm hiểu trang đặt xe có hỗ trợ bạn thực hiện công việc không. Chúng tôi đang đánh giá giao diện, không chấm điểm bạn. Bạn có thể nói điều mình đang nghĩ, bỏ qua câu hỏi hoặc dừng bất kỳ lúc nào. Dữ liệu và tiền trong bản thử này đều là giả.”

Giải thích riêng giới hạn lab: nhân viên đã chuẩn bị C001, SG, bike. Không yêu cầu participant hiểu mã nội bộ hay tự chọn tài khoản. Không hướng dẫn trước nút nào là đáp án của nhiệm vụ.

## Nhiệm vụ cho participant và scoring riêng cho moderator

| ID | Prompt đọc cho participant | Setup riêng | Success không trợ giúp | Giới hạn quan sát |
|---|---|---|---|---|
| U1 | “Bạn định đi xe máy 5 km tại TP.HCM. Hãy tìm số tiền cần trả; chưa yêu cầu tài xế.” | Trang đầu, dữ liệu giả đã nạp | Báo đúng 35.000 đ và dừng trước booking | Tối đa 4 phút trước hỏi có muốn tiếp tục/nhận gợi ý |
| U2 | “Bạn đang xem báo giá nhưng thông tin quãng đường là 0,1 km. Hãy tìm cách lấy báo giá cho chuyến 5 km.” | Distance 0.1, chưa submit | Gặp/nhận biết lỗi khi áp dụng; sửa 5, báo giá 35.000 và giải thích đã sửa gì | Tối đa 4 phút; nếu họ sửa trước submit thì không ép tạo lỗi, ghi đường đi |
| U3 | “Bạn đã yêu cầu chuyến nhưng đổi kế hoạch khi chưa có tài xế nhận. Hãy dừng chuyến và cho biết có khoản phí nào không.” | Seed sạch, moderator tạo chuyến searching riêng trước nhiệm vụ | Huỷ, xác nhận đã huỷ và phí 0 đ | Tối đa 4 phút; không nhắc tên nút huỷ |

Không đọc cột success cho participant. U2 người sửa trước submit vẫn có thể đạt mục tiêu sửa dữ liệu, nhưng **không có evidence họ hiểu thông báo lỗi**; ghi “error-message exposure=no”, không tính họ vào mẫu đánh giá thông báo lỗi. U3 có setup trước, nên không tính là bằng chứng tự đặt được chuyến.

Moderator chỉ nhắc trung lập: “Bạn đang nghĩ gì?”, “Điều gì khiến bạn chọn cách đó?”. Khi cần giúp, ghi thời điểm/nội dung; đánh dấu assisted, không independent success. Nếu lỗi môi trường chặn nhiệm vụ, ghi blocked và tách khỏi failure do UI. Không hứa trước cách sửa thiết kế trong lúc quan sát.

## Quy tắc ghi số liệu

- Bắt đầu timer khi participant đã nghe/hiểu prompt và bắt đầu thao tác; kết thúc khi họ tuyên bố xong và kiểm kết quả, hoặc dừng/nhận trợ giúp theo quy ước. Ghi cả thời điểm trợ giúp.
- Outcome: independent-success / assisted-success / failed / abandoned / blocked. Ghi lý do; không gộp blocked vào user failure.
- Denominator: số người thực sự được giao task và có cơ hội thực hiện; báo riêng blocked/excluded với lý do. Với prompt/subgroup thay đổi, không gộp thiếu giải thích.
- Time-on-task: báo riêng independent successes và phần còn lại; không lấy thời gian script automation. Think-aloud có thể làm chậm, phải giữ phương pháp nhất quán khi so phiên bản.
- Sau task hỏi: “Vừa rồi việc này dễ hay khó với bạn, từ 1 rất khó đến 7 rất dễ? Vì sao?” Đây là câu hỏi tự thiết kế cho study, không gọi là SUS hoặc một thang chuẩn đã được tính đúng.

## Worksheet cho từng participant

Participant ID: ___ · nhóm kinh nghiệm: ___ · browser/version/device: ___ · hỗ trợ cần thiết: ___
Consent tham gia: ___ · recording riêng: ___ · build: ___ · ngày/giờ: ___ · moderator/note-taker: ___

| Task | Start/end | Outcome | Assistance/time | Error-message exposure | Rating 1–7 | Evidence/timestamp |
|---|---|---|---|---|---|---|
| U1 | Chưa chạy | | | N/A | | |
| U2 | Chưa chạy | | | | | |
| U3 | Chưa chạy | | | N/A | | |

| Timestamp | Quan sát thực tế | Lời nói nguyên văn nếu được ghi | Diễn giải/giả thuyết (tách riêng) |
|---|---|---|---|
| Chưa có | | | |

## Mẫu finding và synthesis

Finding ID: ___ · nhiệm vụ: ___ · build: ___ · mô tả hành vi quan sát: ___
Evidence/participant IDs: ___ · số bị ảnh hưởng / số được quan sát: ___ · tác động: ___
Giả thuyết nguyên nhân: ___ · cách kiểm giả thuyết: ___ · đề xuất thay đổi: ___
Ưu tiên: blocker / nghiêm trọng / vừa / nhỏ (giải thích bằng tác động và khả năng tự hồi phục, không chỉ số lượt).
Owner: ___ · kế hoạch retest: ___ · trạng thái: chưa có finding thực tế.

Tổng hợp theo task và nhóm người; giữ cả trường hợp trái chiều. Báo raw counts trước phần trăm khi mẫu nhỏ. Không điền “6/6 thành công” nếu mới dự kiến tuyển sáu người. Kết quả hiện tại: participant=0, study chưa chạy, chưa có success rate/time-on-task/rating thực đo.
