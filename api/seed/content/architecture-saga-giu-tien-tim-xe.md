---
slug: architecture-saga-giu-tien-tim-xe
title: "Saga đặt chuyến khi thanh toán và điều phối hoàn tất lệch nhịp"
summary: "Thiết kế saga cho phương án giữ tiền trước khi tìm xe, xử lý phản hồi muộn và đối soát với nhà cung cấp thanh toán."
category: architecture
level: advanced
tags: [architecture, dotnet, saga, payment, dispatch]
language: vi
---

Giả sử Tới Nơi thử một chính sách thanh toán thẻ cho phép giữ trước 85.000 đồng rồi tìm tài xế trong 60 giây. Đến giây 61, ứng dụng báo hết xe. Khách mở ứng dụng ngân hàng và vẫn thấy số tiền bị giữ. Booking có thể đã kết thúc tìm kiếm trong khi Payment còn một nghĩa vụ chưa hoàn thành.

## Ghi tiến độ của từng bước

Một saga phối hợp nhiều giao dịch cục bộ. Trong ví dụ này, một bộ điều phối lưu tiến độ theo mã đơn trong PostgreSQL. Mỗi lần chuyển bước, nó ghi cả trạng thái mới lẫn lệnh cần gửi vào outbox trong một giao dịch. Worker .NET 10 có thể tiếp tục từ bước đã lưu sau khi khởi động lại.

| Tín hiệu nhận được | Quyết định cần lưu | Lệnh tiếp theo |
|---|---|---|
| Đơn bắt đầu | Chờ giữ tiền | Yêu cầu Payment giữ tiền theo mã thao tác duy nhất |
| Payment xác nhận đã giữ | Lưu mã authorization, mở lượt tìm và hạn chót | Yêu cầu Dispatch tìm tài xế |
| Dispatch đề xuất phân công khi lượt tìm còn hiệu lực | Chốt phân công cho lượt hiện tại | Xác nhận với Dispatch |
| Hết hạn khi vẫn đang tìm | Đóng lượt tìm, đánh dấu chờ giải toả | Thu hồi đề nghị và yêu cầu Payment giải toả |
| Nhà cung cấp xác nhận giải toả | Ghi nhận nghĩa vụ thanh toán đã kết thúc | Cập nhật trạng thái cho khách |

Giữ tiền (*authorize*) và thu tiền (*capture*) là hai bước riêng. Bảng trên giả định khoản tiền mới được giữ. Khi đã thu tiền, Payment cần xử lý hoàn tiền theo chính sách riêng và lưu mã giao dịch hoàn để đối soát.

## Xử lý hai phản hồi đến muộn

Tín hiệu hết hạn và đề xuất của tài xế có thể cùng tới bộ điều phối. Cả hai phải kiểm tra mã lượt tìm, trạng thái, hạn chót và phiên bản trước khi cập nhật. Một khi lượt tìm đã đóng, đề xuất đến muộn chỉ dẫn tới giải phóng chỗ giữ của tài xế. Thiết bị tài xế chỉ hiển thị thành công sau khi nhận xác nhận cuối.

Một trường hợp khó hơn xảy ra khi Payment đã gửi yêu cầu giữ tiền nhưng mất phản hồi của nhà cung cấp. Hãy lưu trạng thái *chưa rõ kết quả* và mã thao tác ban đầu. Worker Payment dùng mã đó để tra cứu hoặc đối chiếu webhook. Nếu đơn đã đóng mà kết quả sau cùng là giữ tiền thành công, saga vẫn phải phát lệnh giải toả. Người dùng thấy “Đã dừng tìm xe, đang chờ ngân hàng giải toả” cho đến khi có kết quả chắc chắn.

## Kiểm tra khả năng phục hồi

Dùng nhà cung cấp thanh toán giả lập có thể ghi nhận lệnh rồi chủ động ngắt kết nối. Khởi động lại worker giữa lúc gửi yêu cầu giải toả và nhận xác nhận. Gửi lại webhook hai lần. Kỳ vọng là một lần giải toả có hiệu lực, lịch sử vẫn giữ cả mã giữ tiền và mã giải toả, và không có chuyến nào được mở từ lượt tìm đã đóng. Trong vận hành, tuổi của yêu cầu giải toả lâu nhất giúp phát hiện một khoản giữ bị bỏ quên dù phần lớn đơn vẫn kết thúc bình thường.

Đọc tiếp bài [Đổi hợp đồng sự kiện Kafka khi các consumer nâng cấp khác ngày](/articles/architecture-hop-dong-su-kien-kafka).
