---
slug: architecture-gio-cao-diem-gps
title: "Giờ cao điểm: giữ luồng nhận chuyến hoạt động khi GPS tăng tải"
summary: "Tính tải từ tần suất gửi vị trí, tách tài nguyên cho điều phối và kiểm tra hệ thống bằng một đợt GPS dồn về sau mất mạng."
category: architecture
level: advanced
tags: [architecture, dotnet, backpressure, gps, load-testing]
language: vi
---

Giả sử 12.000 tài xế đang trực tuyến, mỗi máy gửi một vị trí sau mỗi 3 giây. Riêng luồng GPS tạo trung bình 4.000 điểm mỗi giây. Nếu 3.000 điện thoại đồng thời gửi bù 10 điểm sau khi mạng phục hồi, hệ thống còn phải tiếp nhận thêm 30.000 điểm. Đây là số liệu giả định để thiết kế phép thử. Câu hỏi của đội vận hành là liệu khách có đặt được xe và tài xế có nhận được đề nghị trong đợt dồn tải ấy hay không.

## Quyết định dữ liệu nào được phép gộp

| Nơi dùng dữ liệu | Khi năm điểm của một tài xế đến cùng lúc |
|---|---|
| Bản đồ hiển thị vị trí hiện tại | Có thể gộp thành vị trí hợp lệ mới nhất. So số thứ tự trong cùng phiên thiết bị để điểm gửi bù không ghi đè điểm mới. |
| Lịch sử lộ trình phục vụ đối soát | Lưu theo chính sách lấy mẫu và lưu giữ đã công bố. Nếu cần đủ năm điểm, ghi theo lô vào luồng lưu trữ bền vững. |
| Nhận chuyến và thanh toán | Mỗi thay đổi nghiệp vụ cần mã nhận dạng và kết quả xử lý. Gộp theo “giữ bản mới nhất” có thể làm mất một lần nhận chuyến hoặc một lần hoàn tiền. |

Vị trí mới nhất cũng có thể quá cũ để dùng. Dispatch cần xét tuổi của điểm và độ chính xác trước khi chọn tài xế. Ghi cả thời gian thiết bị đo lẫn thời gian server nhận để điều tra trường hợp hàng loạt điểm cũ vừa tới qua một kết nối mới.

## Tách chỗ xếp hàng và tài nguyên xử lý

Đưa GPS vào topic riêng, cấp consumer và giới hạn số tác vụ ghi vị trí đang chạy. Trong một worker .NET 10, hàng đợi có dung lượng cố định buộc nhóm chọn cách xử lý khi đầy. Với bản đồ trực tiếp, có thể thay điểm cũ bằng điểm mới của **cùng tài xế**. Nếu đã cam kết lưu lịch sử, chỉ xác nhận nhận lô sau khi lô đã được ghi bền vững. Bộ đệm trong bộ nhớ có thể mất khi tiến trình khởi động lại. [BackgroundService và Channel trong .NET 10](/articles/dotnet10-backgroundservice-channel)

Hai topic riêng vẫn có thể tranh I/O trên cùng cụm Kafka. Hai dịch vụ riêng vẫn có thể dùng hết kết nối của cùng máy chủ PostgreSQL. Hãy giới hạn tài nguyên của worker GPS tại nơi thực sự có tranh chấp, dành dung lượng đo được cho phân công và theo dõi cả độ trễ ghi broker lẫn thời gian chờ kết nối cơ sở dữ liệu.

## Một phép thử có thể bác bỏ thiết kế

Chạy đều lưu lượng đặt và nhận chuyến, ghi lại p95 từ khi tài xế bấm nhận tới khi Dispatch xác nhận. Giữ nguyên lưu lượng đó, bơm thêm 30.000 điểm gửi bù rồi cố ý làm chậm kho lịch sử. Theo dõi p95 nhận chuyến, tuổi vị trí đang hiển thị, số điểm được gộp và thời gian hàng đợi trở lại bình thường.

Nếu p95 nhận chuyến tăng trong khi consumer điều phối gần như rảnh, hãy xem tài nguyên dùng chung trước khi tăng số consumer. Nếu p95 ổn nhưng bản đồ lùi vị trí sau đợt gửi bù, hãy kiểm tra quy tắc chọn điểm mới nhất. Hai kết quả này đòi hỏi hai thay đổi thiết kế khác nhau dù chúng xuất hiện trong cùng một lần thử tải.

Đọc tiếp bài [Modular monolith hay microservices cho phiên bản đầu của Tới Nơi?](/articles/architecture-modular-monolith-hay-microservices)
