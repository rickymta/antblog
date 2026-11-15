---
slug: architecture-thu-nghiem-thuat-toan-dieu-phoi
title: "Thay thuật toán điều phối mà vẫn đo được tác động"
summary: "Từ chạy song song đến thử nghiệm trên chuyến thật, xác định cách đo thời gian đón và bảo đảm chỉ một bên được gửi đề nghị."
category: architecture
level: advanced
tags: [architecture, dotnet, dispatch, experimentation, observability]
language: vi
---

Thuật toán A ưu tiên tài xế gần điểm đón theo đường chim bay. Thuật toán B xếp theo thời gian lái xe dự kiến. D9 ở gần khách hơn nhưng nằm bên kia dải phân cách. B chọn D12 và dự đoán đón sớm hơn hai phút. Làm sao biết khách thực sự được đón sớm hơn khi chỉ một trong hai tài xế có thể nhận đơn?

## Chạy hai bộ xếp hạng trên cùng đầu vào

Dispatch tạo bản chụp ứng viên ở thời điểm mở lượt tìm, gồm vị trí có dấu thời gian, loại xe, điều kiện đủ và dữ liệu tuyến đường cần thiết. Ghi phiên bản bộ xếp hạng, mã lượt tìm và phiên bản đầu vào. Nếu B được phép đọc vị trí mới hơn A, khác biệt đầu ra có thể chỉ phản ánh việc D9 vừa di chuyển.

Trong backend .NET 10, có thể tách hàm xếp hạng trả danh sách ứng viên khỏi thành phần có quyền giữ chỗ và gửi đề nghị. Ở giai đoạn chạy quan sát (*shadow*), chỉ kết quả A đi tới thành phần gửi. B ghi kết quả so sánh, dùng tài nguyên có giới hạn riêng và được phép bỏ mẫu khi quá tải. Theo dõi tỷ lệ mẫu bị bỏ theo khu vực và giờ cao điểm để thấy nhóm chuyến nào đang thiếu dữ liệu.

| Từ lần chạy song song có thể đo | Câu hỏi còn cần chuyến thật |
|---|---|
| Tỷ lệ hai thuật toán chọn cùng tài xế đầu tiên | Tài xế B chọn có bấm nhận hay bỏ qua? |
| p95 thời gian tính và tỷ lệ vượt ngân sách | Khách đợi bao lâu từ lúc đặt đến khi xe tới? |
| Tần suất B chọn ứng viên vi phạm điều kiện nhận chuyến | Chính sách mới ảnh hưởng thế nào tới tỷ lệ huỷ hoặc chuyến không tìm được xe? |

## Thiết kế phép thử trong một thị trường dùng chung tài xế

Nếu chia ngẫu nhiên theo từng đơn, B có thể giữ D12 khiến D12 biến mất khỏi danh sách của A ngay sau đó. Hai nhóm tác động lẫn nhau qua nguồn cung tài xế. Một phương án là luân phiên chính sách theo khu vực và khung giờ. Cân bằng các khung giờ tương ứng giữa ngày trong tuần, xét tài xế di chuyển qua biên khu vực và dành thời gian chuyển tiếp nếu tác động còn kéo dài. Ghi nhóm thử nghiệm khi mở lượt tìm và giữ nguyên lựa chọn ấy qua các lần thử lại của cùng lượt.

Trước khi phát đề nghị thật, thống nhất thước đo. Ví dụ, theo dõi tỷ lệ **toàn bộ đơn đủ điều kiện** được đón trong mười phút. Mẫu số gồm cả đơn thất bại và bị huỷ. Xem thêm p95 thời gian đón trong nhóm đã đón, tỷ lệ huỷ và số đề nghị gửi trên mỗi đơn. Nếu chỉ so thời gian của các chuyến thành công, B có thể trông nhanh hơn nhờ bỏ rơi những chuyến khó. Mốc mười phút là một giả định sản phẩm cho ví dụ này.

## Quay lui khi đã có đề nghị ngoài thực tế

Khi độ trễ xếp hạng hoặc tỷ lệ lỗi vượt ngưỡng đã đặt trước, dừng cấp lượt tìm mới cho B. Lượt đang chờ tài xế trả lời tiếp tục dùng mã đề nghị, thời hạn và chính sách đã ghi. Chỉ mở lượt kế tiếp sau khi lượt trước đã kết thúc hoặc được thu hồi. Dùng quy tắc [giữ chỗ theo thế hệ](/articles/architecture-giu-cho-tai-xe) để chặn phản hồi cũ.

Trước thử nghiệm, diễn tập bằng một đơn có đề nghị đang bay: chuyển chính sách về A, cho điện thoại gửi xác nhận muộn rồi gửi lại cùng xác nhận. Kiểm tra đơn chỉ sinh một phân công, log vẫn ghi được phiên bản đã chọn D12 và báo cáo vẫn tính đơn vào đúng nhóm ban đầu.

Đọc tiếp bài [Ring buffer trong Tới Nơi](/articles/architecture-ring-buffer-gps).
