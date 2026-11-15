---
slug: architecture-grpc-kafka-dat-chuyen
title: "Đặt chuyến gọi gRPC ở đâu, phát Kafka ở đâu?"
summary: "Theo dấu lần bấm Đặt chuyến để chọn điểm trả HTTP, đặt deadline cho Pricing và xử lý khi Dispatch tạm dừng."
category: architecture
level: advanced
tags: [architecture, dotnet, grpc, kafka, outbox]
language: vi
---

Khách đã thấy giá 85.000 đồng và bấm *Đặt chuyến*. Một giây sau, ứng dụng mất mạng. Khi mở lại, khách cần biết mình đã có đơn chưa. Backend .NET 10 của Tới Nơi phải trả lời được câu hỏi này kể cả khi dịch vụ tìm tài xế đang bảo trì.

## Chọn thời điểm trả lời cho ứng dụng

Trong thiết kế minh hoạ, Booking chỉ nhận đơn khi báo giá còn hiệu lực và thuộc đúng khách. Pricing sở hữu báo giá, nên Booking hỏi Pricing qua gRPC để lấy quyết định cùng số tiền và hạn hiệu lực. Nếu báo giá hết hạn trước lúc chấp nhận đơn, Booking yêu cầu khách xác nhận giá mới. Ở bước sau, Dispatch có thể bắt đầu tìm tài xế từ một đơn đã được lưu.

| Bước | Khi nào bước này hoàn tất? | Khách có phải chờ? |
|---|---|---|
| Xác nhận báo giá | Pricing trả về báo giá hợp lệ | Có |
| Nhận đơn | Booking commit đơn ở trạng thái `PENDING_DISPATCH` cùng bản ghi outbox `BookingCreated` | Có |
| Tìm tài xế | Dispatch đọc sự kiện và mở lượt tìm | Khách đã có thể xem mã đơn |
| Hiển thị người nhận | Dispatch xác nhận phân công, ứng dụng nhận trạng thái mới | Ứng dụng cập nhật sau |

Có thể trả HTTP 201 ngay sau commit, kèm mã đơn và đường dẫn đọc trạng thái. Nếu API trả “đã có tài xế” ở thời điểm này, ứng dụng đang hứa một kết quả mà Dispatch còn chưa xử lý.

## Một deadline cho phần khách phải chờ

Giả sử endpoint có ngân sách 500 ms. Sau 180 ms xác thực và xếp hàng, nó chỉ còn 320 ms cho cả Pricing lẫn giao dịch lưu đơn. Client gRPC cần dành lại thời gian cho giao dịch và truyền tín hiệu huỷ của request. Cấp mới trọn 500 ms cho mỗi lần thử sẽ khiến thời gian thực tế vượt ngân sách. [Deadline và cancellation trong gRPC trên ASP.NET Core](https://learn.microsoft.com/en-us/aspnet/core/grpc/deadlines-cancellation?view=aspnetcore-10.0)

## Mất phản hồi sau commit

Ứng dụng sinh một khoá idempotency cho lần bấm và dùng lại khoá đó khi thử lại. Booking đặt ràng buộc duy nhất trên cặp khách hàng và khoá, đồng thời lưu dấu nhận dạng nội dung yêu cầu. Nếu đơn đã tồn tại và nội dung khớp, trả lại mã đơn cũ trước khi gọi Pricing lần nữa. Cùng khoá nhưng khác hành trình là xung đột. Ràng buộc trong cơ sở dữ liệu cũng bảo vệ trường hợp hai request đến đồng thời.

Thử dừng consumer Dispatch, đặt đơn rồi ngắt kết nối ngay sau commit. Gửi lại request với cùng khoá và kiểm tra hai lần đều dẫn tới một mã đơn. Khi bật Dispatch, một lượt tìm hợp lệ phải được mở dù sự kiện được giao lại. Hãy đo riêng thời gian nhận đơn và thời gian từ lúc nhận đơn tới lúc mở lượt tìm. HTTP vẫn nhanh trong lúc chỉ số thứ hai tăng là dấu hiệu cần kiểm tra luồng điều phối. [Cơ chế outbox và inbox trong .NET 10](/articles/dotnet10-outbox-inbox-idempotent-consumer)

Đọc tiếp bài [Ai sở hữu trạng thái chuyến? Ranh giới giữa Booking, Dispatch và Trip](/articles/architecture-quyen-so-huu-trang-thai).
