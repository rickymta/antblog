---
slug: architecture-modular-monolith-hay-microservices
title: "Modular monolith hay microservices cho phiên bản đầu của Tới Nơi?"
summary: "Theo dấu một lần đổi thuật toán tìm xe để quyết định khi nào Booking, Dispatch và Trip nên chạy chung, khi nào đáng tách Dispatch."
category: architecture
level: advanced
tags: [architecture, dotnet, modular-monolith, microservices, dispatch]
language: vi
---

Sáng thứ Hai, đội phát triển muốn ưu tiên tài xế có thời gian tới điểm đón ngắn nhất. Bản sửa chỉ nằm ở Dispatch. Nếu Booking, Dispatch và Trip chạy trong cùng ứng dụng ASP.NET Core trên .NET 10, đội sẽ phát hành lại cả ứng dụng. Nếu Dispatch chạy riêng, đội có thể phát hành riêng nhưng phải trả lời câu hỏi khác: điều gì xảy ra với đơn vừa tạo trong lúc Dispatch khởi động lại?

## Vẽ ranh giới trước khi chọn số tiến trình

Ở phiên bản đầu, có thể đặt ba module trong một ứng dụng. Booking sở hữu đơn, Dispatch sở hữu đề nghị và phân công, Trip sở hữu hành trình đã bắt đầu. Cả ba dùng được cùng máy chủ PostgreSQL. Mỗi module quản lý bảng, migration và mã ghi dữ liệu của mình. Booking chuyển mã đơn cùng thông tin điểm đón qua hợp đồng đã định nghĩa. Nếu bộ xếp hạng cần thêm thông tin, nhóm bổ sung dữ liệu vào hợp đồng. Việc cho Dispatch truy vấn thẳng bảng nội bộ của Booking sẽ khiến lần đổi schema sau buộc hai module nâng cấp cùng lúc.

Một ranh giới đáng giữ ngay từ đầu là thời điểm Booking xác nhận đã nhận đơn. Lưu đơn và công việc cần chuyển cho Dispatch trong cùng giao dịch. Worker có thể xử lý công việc đó sau commit, kể cả khi hai module cùng chạy trong một tiến trình. Nếu chỉ đưa công việc vào một hàng đợi trong bộ nhớ, ứng dụng có thể khởi động lại sau khi trả mã đơn và trước khi mở lượt tìm. Khách sẽ có đơn nhưng không ai tìm xe. [Luồng nhận đơn và mở lượt tìm](/articles/architecture-grpc-kafka-dat-chuyen)

## Một lần thay thuật toán tác động đến đâu?

| Câu hỏi | Chạy chung ứng dụng | Dispatch chạy riêng |
|---|---|---|
| Phát hành và quay lui | Quay lui ứng dụng có thể kéo theo bản Booking vừa phát hành | Có thể quay lui Dispatch nếu phiên bản cũ còn đọc được các sự kiện đang chờ |
| Khi xếp hạng dùng hết CPU | Request tạo đơn cùng tranh CPU với bộ xếp hạng | Có thể cấp tài nguyên và tăng số worker riêng cho Dispatch |
| Khi một lượt tìm bị bỏ dở | Kiểm tra việc lưu công việc qua lần khởi động lại | Kiểm tra thêm giao lại bản tin, độ trễ consumer và tương thích giữa hai phiên bản |

Với đội nhỏ đang đổi cả quy tắc đặt và nhận chuyến mỗi tuần, bài này chọn chạy chung để sửa và kiểm thử hai module trong cùng một lần phát hành. Hãy ghi lại số lần phải trì hoãn một bản Booking vì Dispatch, thời gian phục hồi khi quay lui, và p95 tạo đơn lúc xếp hạng tăng tải. Nếu một module liên tục gây chậm phát hành hoặc chiếm hết tài nguyên, những số liệu đó cho biết việc tách nó sẽ giải quyết vấn đề nào.

## Thử tách Dispatch mà vẫn chỉ có một bên gửi đề nghị

Cho worker mới đọc bản sao đầu vào và ghi kết quả xếp hạng để so sánh. Sau khi kiểm tra kết quả, chuyển quyền mở lượt tìm cho worker mới theo từng nhóm đơn. Cả đường cũ lẫn đường mới cần nhận diện cùng mã lượt tìm và dùng chung quy tắc chống gửi trùng. Thử dừng worker giữa lúc mở lượt và gửi đề nghị, rồi quay lui. Một đơn đã nhận phải tiếp tục được tìm xe và mỗi đề nghị chỉ có một chủ sở hữu. Đây cũng là phép thử xem ranh giới module đã đủ rõ để vận hành độc lập hay chưa.

Đọc tiếp bài [Báo giá còn hiệu lực khi khách bấm đặt chuyến?](/articles/architecture-vong-doi-bao-gia)
