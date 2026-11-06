# Hồ sơ thực hành BA · Tới Nơi

Đọc cùng [khung chương trình](../../docs/ba-master-curriculum.md). Đây là tình huống hư cấu phục vụ học BA, không thay thế hồ sơ nghiệp vụ chung hoặc SPEC của lab Testing.

## Ca P01: khách và tài xế khó tìm nhau

**Dữ kiện giả lập:** trong một tuần quan sát ở khu vực A có 1.000 lượt đặt xe đã được tài xế nhận. Có 120 lượt kết thúc bằng huỷ trước khi đón; 50 trong 120 lượt mang lý do do người dùng chọn “không tìm thấy nhau”. Có 80 cuộc gọi CSKH được gắn nhãn liên quan tới điểm đón; một lượt có thể sinh nhiều cuộc gọi. Bộ số liệu không khẳng định chất lượng gắn nhãn hoặc quan hệ nhân quả.

- Tỷ lệ huỷ trước đón: 120 / 1.000 = 12%.
- Tỷ lệ lượt có nhãn huỷ “không tìm thấy nhau” trên lượt đã được nhận: 50 / 1.000 = 5%.
- Tỷ trọng nhãn này trong các lượt huỷ trước đón: 50 / 120 ≈ 41,7%.
- Không suy ra 80 khách hoặc 80 chuyến từ 80 cuộc gọi. Chưa có dữ liệu nối cuộc gọi với lượt đặt xe.

**Lời đề nghị giả lập từ vận hành:** “Thêm chat vào app để tài xế và khách tìm nhau nhanh hơn.”

**Giả thuyết chưa xác minh:** ghim bản đồ sai; địa điểm có nhiều cổng; khách thiếu chỉ dẫn; tài xế không xem ghi chú; dữ liệu lý do huỷ không phản ánh nguyên nhân chính.

**Mục tiêu đề xuất để thảo luận:** trong pilot bốn tuần ở A, giảm tỷ lệ huỷ trước đón từ baseline minh hoạ 12% xuống không quá 9%. Baseline một tuần cần được kiểm tra tính đại diện trước khi dùng ra quyết định. Tử số và mẫu số của pilot dùng cùng định nghĩa với baseline, theo cohort lượt được nhận trong kỳ; chờ đủ kết quả trước khi chốt số. Chỉ số bảo vệ đề xuất: khiếu nại trên 1.000 lượt đã được nhận không tăng so với baseline tương đương. Chưa có số baseline khiếu nại; phải đo trước pilot.

Mức 9% là giả định để học viết mục tiêu, không phải dự báo tác động của chat hay cam kết triển khai. Giảm 12% xuống 9% là giảm 3 điểm phần trăm, tương đương 25% tương đối; phép tính không chứng minh nguyên nhân.

## Mẫu problem brief

| Trường | Nội dung cần điền |
|---|---|
| Vấn đề và nhóm chịu tác động | Điều đang xảy ra, ở đâu, với ai |
| Bằng chứng và giới hạn | Nguồn, thời gian, mẫu số, chất lượng dữ liệu |
| Tác động | Hậu quả với khách, tài xế, CSKH và tổ chức |
| Mục tiêu đề xuất | Chỉ số, baseline, đích, thời gian và chỉ số bảo vệ |
| Phạm vi in/out | Bao gồm và chưa bao gồm gì |
| Giả thuyết / câu hỏi mở | Điều chưa biết và cách kiểm chứng |
| Phương án | Giữ nguyên, thay đổi quy trình, thay đổi sản phẩm |
| Quyết định cần xin | Người quyết định, hạn và bằng chứng cần có |

## Decision log khởi tạo

| ID | Nội dung | Trạng thái | Người quyết định trong bài tập | Bước tiếp |
|---|---|---|---|---|
| D-01 | Có dành một tuần discovery cho điểm đón? | Đề xuất | PO trong phạm vi ngân sách được giao | Xác nhận nguồn dữ liệu và người tham gia |
| D-02 | Có chọn chat làm giải pháp? | Chưa quyết định | PO, với sponsor nếu vượt ngân sách | So sánh với cải thiện ghi chú/điểm đón |
| D-03 | Có duyệt mục tiêu pilot 9%? | Đề xuất | Sponsor/đại diện nghiệp vụ được uỷ quyền | Kiểm tra baseline và khả năng đo guardrail |

Các vai trò trên là phân công giả lập của Tới Nơi cho bài tập, không phải quy tắc bắt buộc ở mọi tổ chức.

## Tự kiểm tra trước khi nộp

1. Người khác có phân biệt được dữ kiện và giả thuyết không?
2. Mọi tỷ lệ có đúng mẫu số, cohort và thời gian không?
3. Có phương án không xây tính năng và lý do so sánh không?
4. Quyết định có người sở hữu, trạng thái và bước tiếp không?
5. Yêu cầu mới có được ghi là đề xuất, tránh bị hiểu là SPEC đã duyệt không?
