---
slug: architecture-signalr-khoi-phuc-trang-thai
title: "Realtime cho tài xế khi kết nối SignalR bị gián đoạn"
summary: "Phối hợp snapshot, phiên bản và thông báo SignalR để khôi phục đúng màn hình nhận chuyến sau khi điện thoại mất mạng."
category: architecture
level: advanced
tags: [architecture, dotnet, signalr, realtime, dispatch]
language: vi
---

Tài xế đang nhìn nút *Nhận chuyến* của đề nghị O17 thì xe đi vào hầm. Trong lúc mất mạng, khách huỷ đơn. Khi xe ra khỏi hầm, kết nối SignalR đã mở lại nhưng màn hình vẫn giữ dữ liệu cũ. Kết nối thành công xác nhận đường truyền hoạt động. Để bật lại nút, ứng dụng còn phải biết đề nghị nào đang có hiệu lực.

## Một phiên bản cho màn hình của từng tài xế

Trong thiết kế minh hoạ, Dispatch duy trì snapshot danh sách đề nghị cho mỗi tài xế cùng một số phiên bản tăng dần. Khi thêm hoặc thu hồi đề nghị, Dispatch cập nhật danh sách, tăng phiên bản và ghi sự kiện vào outbox trong cùng giao dịch. Backend .NET 10 gửi qua SignalR một thông báo gọn gồm phiên bản mới. Ứng dụng dùng thông báo để biết khi nào cần tải lại snapshot.

Phiên bản phải thuộc cùng phạm vi với snapshot. Ví dụ, O17 và O18 có thể cùng ở phiên bản 1 nếu đếm riêng theo đề nghị. Chỉ so hai số đó sẽ không cho biết danh sách nào mới hơn. API lấy danh tính tài xế từ phiên đã xác thực và trả cả danh sách lẫn phiên bản từ một ảnh dữ liệu nhất quán.

## Khép khoảng trống giữa đăng ký và tải dữ liệu

1. Khi mất kết nối, giữ thông tin để tài xế còn xem được nhưng tạm khoá thao tác nhận. Khi kết nối lại, đăng ký lại luồng thông báo của tài xế và đợi server xác nhận đăng ký.
2. Bắt đầu lấy snapshot qua HTTP. Trong khi chờ, ghi nhớ phiên bản lớn nhất vừa nhận qua SignalR. Giả sử snapshot trả phiên bản 31 còn thông báo mới nhất là 32, tiếp tục lấy snapshot cho tới khi theo kịp. Nếu mô hình đọc đang trễ, giãn khoảng cách giữa các lần thử.
3. Chỉ áp dụng snapshot có phiên bản bằng hoặc lớn hơn trạng thái đang hiển thị. Nếu phiên bản 33 chứa danh sách rỗng, xoá O17. Một phản hồi HTTP cũ mang phiên bản 31 về sau đó phải được bỏ qua.

Luồng này cần chạy cả khi ứng dụng trở lại từ nền. Thêm một lần đồng bộ định kỳ nếu sản phẩm cần giới hạn thời gian màn hình có thể cũ trong trường hợp bỏ lỡ thông báo cuối. Khi tài xế bấm nhận, Dispatch vẫn phải kiểm tra quyền sở hữu và hạn đề nghị. [Ranh giới quyết định giữa Booking và Dispatch](/articles/architecture-quyen-so-huu-trang-thai)

## Dựng lại lỗi trước khi cho xe ra đường

Giữ phản hồi HTTP phiên bản 31 ở một proxy thử nghiệm, thu hồi O17 rồi chuyển thông báo phiên bản 32 tới điện thoại. Cho phản hồi cũ đi tiếp và kiểm tra rằng nút nhận chỉ bật lại sau khi snapshot mới xác nhận đề nghị còn hiệu lực. Lặp lại với hai lần ngắt mạng liên tiếp và một thông báo trùng. Đo thời gian từ lúc kết nối lại đến lúc màn hình khớp với Dispatch, vì thời điểm socket mở có thể đến sớm hơn đáng kể.

Đọc tiếp bài [Giờ cao điểm: giữ luồng nhận chuyến hoạt động khi GPS tăng tải](/articles/architecture-gio-cao-diem-gps).
