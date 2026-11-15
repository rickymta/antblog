---
slug: architecture-vong-doi-bao-gia
title: "Báo giá còn hiệu lực khi khách bấm đặt chuyến?"
summary: "Thiết kế hợp đồng giữa Pricing và Booking để xử lý giá hết hạn, lần đặt lại và yêu cầu giải thích giá sau chuyến."
category: architecture
level: advanced
tags: [architecture, dotnet, pricing, booking, idempotency]
language: vi
---

Lúc 08:00:00, khách thấy giá 85.000 đồng cho tuyến A–B. Lúc 08:01:00, Pricing cập nhật phụ phí giờ cao điểm. Khách bấm đặt lúc 08:01:59, nhưng request tới Booking lúc 08:02:01. Có ba thời điểm hợp lý để so với hạn báo giá: lúc bấm, lúc server nhận và lúc server chấp thuận. Chọn mốc nào sẽ thay đổi số tiền khách phải xác nhận.

## Báo giá là một cam kết có điều kiện

Giả sử sản phẩm cam kết giá cố định trong hai phút, áp dụng cho đúng khách, hành trình và loại xe. Pricing phát `quoteId`, số tiền, mã tiền tệ, hạn hiệu lực theo UTC, phiên bản biểu giá và chi tiết từng khoản. Booking nhận mã báo giá cùng thông tin chuyến. Ứng dụng chỉ hiển thị số đếm ngược để khách tham khảo, vì đồng hồ điện thoại có thể lệch.

Trong thiết kế này, Pricing quyết định tại thời điểm **chấp thuận sử dụng báo giá** theo đồng hồ server. Với hạn 08:02:00, lần sử dụng mới phải đến trước mốc đó. Request tới lúc 08:02:01 nhận kết quả hết hạn. Ứng dụng lấy báo giá mới, hiển thị phần chênh và chờ khách đồng ý. Nếu đã được chấp thuận lúc 08:01:59, Booking vẫn có thể lưu đơn sau 08:02:00 theo quyết định đã ghi.

## Khi Pricing và Booking ghi vào hai cơ sở dữ liệu

1. Booking tạo mã đơn ổn định cho lần bấm, lưu trạng thái chờ xác nhận giá và khoá idempotency của khách. Cùng khoá nhưng khác hành trình là xung đột.
2. Pricing kiểm tra chủ báo giá, điều kiện chuyến và hạn. Trong một giao dịch, nó ràng buộc báo giá dùng một lần với mã đơn và lưu số tiền đã chấp thuận. Khi cùng mã đơn gửi lại, Pricing trả quyết định cũ trước khi xét hạn cho một lần sử dụng mới.
3. Booking lưu bản chụp báo giá đã chấp thuận, chuyển đơn sang chờ điều phối và ghi outbox. Chỉ lúc đó ứng dụng mới hiển thị đơn đã được nhận. Nếu Booking dừng giữa bước 2 và 3, worker tiếp tục bằng mã đơn cũ. Một đơn bị bỏ dở cần chính sách đóng rõ ràng để khoản giữ khuyến mãi, nếu có, được giải phóng.

## Giải thích được con số trên biên nhận

Với Q42, giả sử giá cơ bản là 65.000 đồng, phụ phí 25.000 đồng và ưu đãi 5.000 đồng. Snapshot của đơn phải cho phép cộng lại đúng 85.000 đồng sau khi biểu giá thay đổi. Với sản phẩm còn thu phí chờ hay phí cầu đường sau chuyến, lưu riêng các khoản phát sinh cùng căn cứ tính. Nhân viên hỗ trợ có thể chỉ rõ khoản nào khách đã xác nhận và khoản nào được ghi thêm.

Trong bộ kiểm thử .NET 10, truyền đồng hồ qua `TimeProvider` để đặt thời điểm ngay trước, đúng bằng và ngay sau hạn. Thử thêm hai đơn dùng chung Q42, và trường hợp Pricing đã chấp thuận nhưng Booking mất phản hồi. Kết quả mong đợi là một mã đơn giữ quyền dùng Q42, một số tiền nhất quán và không có lượt tìm nào được mở cho báo giá hết hạn.

Đọc tiếp bài [Hai chuyến cùng chọn một tài xế](/articles/architecture-giu-cho-tai-xe)
