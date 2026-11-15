---
slug: architecture-quyen-so-huu-trang-thai
title: "Ai sở hữu trạng thái chuyến? Ranh giới giữa Booking, Dispatch và Trip"
summary: "Phân xử xung đột giữa thao tác huỷ và nhận chuyến bằng quyền sở hữu dữ liệu, phiên bản và một quyết định có thẩm quyền."
category: architecture
level: advanced
tags: [architecture, dotnet, bounded-context, concurrency, dispatch]
language: vi
---

Lúc 08:15:02, khách huỷ đơn B42. Cũng trong giây đó, tài xế nhận đề nghị O17. Sắp xếp hai log theo đồng hồ máy chủ sẽ cho một thứ tự, nhưng độ lệch đồng hồ và thời gian truyền mạng khiến thứ tự ấy thiếu căn cứ để phân xử. Nhóm cần thống nhất thao tác nào quyết định kết quả và dữ liệu nào chứng minh quyết định đã được ghi nhận.

## Ba câu hỏi, ba chủ sở hữu

| Miền | Câu hỏi miền này trả lời | Dữ liệu có thẩm quyền |
|---|---|---|
| Booking | Đơn còn cho phép phân công hoặc huỷ theo chính sách nào? | Vòng đời đơn, phiên bản đơn |
| Dispatch | Đề nghị nào còn hạn và đang dành cho tài xế nào? | Đề nghị, lần tìm, phân công |
| Trip | Tài xế đã đến điểm đón hay bắt đầu chở khách? | Các mốc thực hiện chuyến |

Màn hình điều phối ghép cả ba nguồn thành một mô hình đọc. Nút *Nhận chuyến* có thể còn sáng trên điện thoại dù đề nghị vừa hết hạn. Mỗi command vì thế cần được kiểm tra lại tại miền sở hữu quyết định. [Mô hình đọc và ghi qua thao tác nhận chuyến](/articles/architecture-cqrs)

## Chọn một nơi phân xử huỷ và nhận

Giả sử chính sách cho phép huỷ tự do cho tới khi Booking chấp thuận phân công. Khi tài xế bấm nhận, Dispatch kiểm tra tài xế được mời, hạn và lượt tìm, rồi giữ chỗ cho O17. Dispatch gửi yêu cầu xác nhận phân công về Booking với mã đơn, mã lượt tìm và mã đề nghị. Trong thiết kế này, thao tác huỷ và thao tác xác nhận đều phải chuyển cùng một hàng Booking từ trạng thái chờ bằng cập nhật có điều kiện.

Nếu huỷ thắng, Booking lưu trạng thái huỷ và từ chối xác nhận O17. Dispatch giải phóng chỗ đã giữ. Nếu xác nhận thắng, Booking lưu mã phân công và phát sự kiện qua outbox. Yêu cầu huỷ đến sau phải đi theo chính sách huỷ sau phân công. Chỉ khi có quyết định chấp thuận, ứng dụng mới báo tài xế đã nhận thành công.

## Thử đúng lúc phản hồi bị mất

Cho Booking chấp thuận O17 rồi ngắt kết nối trước khi Dispatch nhận phản hồi. Hết thời gian chờ chỉ cho biết Dispatch chưa thấy kết quả. Dispatch giữ trạng thái chờ xác minh và truy vấn hoặc gửi lại cùng mã O17. Booking trả quyết định đã lưu cho O17. Nếu một lượt tìm khác đã bắt đầu, mã lượt tìm giúp Booking loại đề nghị cũ.

Trong bài kiểm thử tích hợp, dùng hai tác vụ .NET chờ cùng một tín hiệu rồi đồng thời gửi lệnh huỷ và nhận. Kiểm tra trạng thái cuối ở Booking, việc thu hồi hoặc xác nhận đề nghị ở Dispatch, và số chuyến thực sự được tạo ở Trip. Chạy thêm trường hợp sự kiện xác nhận được giao hai lần. Một ràng buộc duy nhất theo mã phân công tại Trip giúp cả hai lần xử lý cùng quy về một chuyến. [Bounded context và aggregate trong backend .NET 10](/articles/dotnet10-ddd-bounded-context-aggregate)

Đọc tiếp bài [Saga đặt chuyến khi thanh toán và điều phối hoàn tất lệch nhịp](/articles/architecture-saga-giu-tien-tim-xe).
