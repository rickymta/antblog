---
slug: architecture-cqrs
title: "CQRS qua thao tác nhận chuyến và màn hình điều phối"
summary: "Thiết kế mô hình ghi bảo vệ quyền nhận chuyến và mô hình đọc phục vụ điều phối, từ một cơ sở dữ liệu đến projection bất đồng bộ."
category: architecture
level: intermediate
tags: [architecture, cqrs, read-model, concurrency]
language: vi
---

Hai tài xế cùng thấy chuyến T42 đang chờ. Cả hai chạm “Nhận chuyến” trước khi màn hình kịp làm mới. Màn hình điều phối cần trả lời “những chuyến nào có thể nhận?”, còn thao tác nhận chuyến phải quyết định ai giành được quyền đó ngay lúc ghi.

CQRS (Command Query Responsibility Segregation) tách mô hình thực hiện thay đổi khỏi mô hình trả lời truy vấn. Hai mô hình có thể cùng chạy trong một ứng dụng và dùng chung PostgreSQL. [Martin Fowler về CQRS](https://martinfowler.com/bliki/CQRS.html)

## Command quyết định ai nhận được chuyến

Trong backend .NET 10 của Tới Nơi, endpoint ASP.NET Core có thể tạo command `AcceptTrip` từ mã chuyến trên URL và danh tính tài xế trong `HttpContext.User`. Handler nhận command cùng `CancellationToken` của request. Sau khi kiểm tra quyền, bộ xử lý thực hiện câu lệnh có điều kiện dưới đây. Ví dụ giả định mã chuyến là khoá chính và trạng thái ban đầu là `waiting`.

```sql
UPDATE trips
SET status = 'accepted',
    driver_id = $2,
    version = version + 1
WHERE id = $1 AND status = 'waiting'
RETURNING id, driver_id, version;
```

Ở mức cô lập mặc định của PostgreSQL, hai lệnh cùng sửa T42 sẽ tranh khoá trên cùng hàng. Sau khi lệnh đầu commit, lệnh sau kiểm tra lại điều kiện và cập nhật được 0 hàng. Bộ xử lý có thể đọc trạng thái hiện tại để phân biệt chuyến không tồn tại, chuyến đã thuộc tài xế khác, hoặc chính tài xế vừa gửi lại yêu cầu vì mất phản hồi.

## Query phục vụ màn hình điều phối

Màn hình cần mã chuyến, khu vực đón và số phút đã chờ. Một câu SELECT hoặc SQL view có thể trả đúng các cột này từ dữ liệu đã commit. Mô hình đọc được phép đổi cách sắp xếp hay bổ sung tên khu vực mà không đụng đến quy tắc nhận chuyến. Với truy vấn chạy nhanh trên cơ sở dữ liệu chính, thiết kế có thể dừng ở đây.

Khi phép tổng hợp cho điều phối quá tốn kém, có thể duy trì riêng bảng `dispatch_board`. Trong cùng giao dịch nhận chuyến, ghi một sự kiện `TripAccepted` vào bảng outbox. Sau khi commit, một tiến trình chuyển sự kiện tới bộ xử lý để cập nhật bảng đọc. Bản dữ liệu được dựng cho mục đích đọc này gọi là *projection*.

## Khoảng trễ hiện ra trên màn hình như thế nào?

Giả sử command trả về phiên bản 8 nhưng bảng điều phối vẫn ở phiên bản 7. Ứng dụng của người vừa nhận chuyến có thể hiển thị ngay kết quả từ command. Một tài xế khác vẫn có thể nhìn thấy T42 trong danh sách cũ, nhưng điều kiện trong câu UPDATE tiếp tục bảo vệ quyền nhận chuyến. Với projection bất đồng bộ, cần theo dõi tuổi của sự kiện chưa xử lý lâu nhất và xử lý an toàn khi cùng sự kiện đến hai lần.

Hãy thử dừng bộ xử lý projection, gửi đồng thời hai command rồi khởi động lại nó. Kết quả cần quan sát là chỉ một tài xế nhận được chuyến và bảng điều phối cuối cùng phản ánh đúng tài xế đó. Bài [Message Broker với Kafka](/articles/architecture-kafka) đi tiếp vào tình huống bộ xử lý phải đọc lại một sự kiện.
