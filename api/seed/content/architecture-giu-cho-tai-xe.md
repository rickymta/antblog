---
slug: architecture-giu-cho-tai-xe
title: "Hai chuyến cùng chọn một tài xế: giữ chỗ và xác nhận phân công"
summary: "Dùng lease có thế hệ và giao dịch PostgreSQL để ngăn worker cũ xác nhận trễ hoặc phân một tài xế cho hai chuyến."
category: architecture
level: advanced
tags: [architecture, dotnet, dispatch, concurrency, postgresql]
language: vi
---

Hai đơn B42 và B43 cách nhau một con phố. Hai worker đọc cùng bản đồ và đều chọn tài xế D9. Việc cả hai xếp D9 đầu danh sách là hợp lý. Sai sót xảy ra nếu cả hai cùng ghi được phân công có hiệu lực. Bài CQRS trước đã bảo vệ một chuyến khỏi hai lượt nhận. Lần này, quy tắc cần giữ nằm trên một tài xế và trải qua nhiều đơn. [Tình huống hai tài xế cùng nhận một chuyến](/articles/architecture-cqrs)

## Giữ chỗ có hạn và có thế hệ

Một cách thiết kế trong Dispatch là có một hàng `driver_slot` cho mỗi tài xế đủ điều kiện nhận chuyến. Hàng này có các trường trạng thái, mã đơn, hạn giữ và số thế hệ tăng dần. Worker chỉ giành được chỗ khi cập nhật thành công một hàng đang rảnh hoặc đã hết hạn giữ. Khoá hàng trong giao dịch ngắn, lấy thời điểm hiện tại từ cơ sở dữ liệu sau khi có khoá, rồi tăng thế hệ. Giao dịch kết thúc trước khi gửi đề nghị tới điện thoại.

Ví dụ, lượt tìm B42 giữ D9 ở thế hệ 71 tới 08:15:10. Worker đầu tiên bị dừng. Lúc 08:15:11, worker thứ hai chiếm lại chỗ cho B43 và nhận thế hệ 72. Khi tài xế bấm vào thông báo cũ, yêu cầu nhận mang theo mã đề nghị gắn với B42 và thế hệ 71. API lấy danh tính D9 từ phiên đã xác thực. Dispatch chỉ xác nhận nếu hàng hiện tại còn ở trạng thái giữ chỗ, cùng đơn, cùng thế hệ và còn hạn. Vì hàng đã ở thế hệ 72, yêu cầu cũ kết thúc mà không chạm vào B43. Lệnh thu hồi cũng phải kiểm tra thế hệ, nếu không worker cũ có thể giải phóng nhầm chỗ mới.

## Chốt phân công ở nơi có thể cưỡng chế quy tắc

Giả sử bảng dưới chỉ lưu phân công có hiệu lực hoặc đã kết thúc. Trong ví dụ này, mỗi tài xế chỉ phục vụ một đơn tại một thời điểm. Hai chỉ mục duy nhất có điều kiện bảo vệ cả hai phía ở mức PostgreSQL.

```sql
CREATE UNIQUE INDEX one_active_assignment_per_driver
    ON assignments (driver_id)
    WHERE ended_at IS NULL;

CREATE UNIQUE INDEX one_active_assignment_per_booking
    ON assignments (booking_id)
    WHERE ended_at IS NULL;
```

Hai cột mã cần có ràng buộc `NOT NULL`. Khi chốt, Dispatch kiểm tra lại chỗ giữ dưới khoá, ghi phân công, chuyển chỗ sang trạng thái bận và ghi sự kiện outbox trong cùng giao dịch. Chỉ trả thành công sau commit. Nếu Booking còn quyền từ chối vì khách đã huỷ, giữ trạng thái chờ xác nhận theo giao thức giữa hai miền trước khi báo thành công cho tài xế. [Chỉ mục có điều kiện trong PostgreSQL](https://www.postgresql.org/docs/current/indexes-partial.html) · [Quyền phân xử của Booking](/articles/architecture-quyen-so-huu-trang-thai)

## Kiểm thử worker quay lại muộn

Trong kiểm thử tích hợp .NET 10 với PostgreSQL thật, cho worker A dừng ngay sau khi giữ chỗ. Đặt hạn của lượt giữ đó về quá khứ trong dữ liệu thử, cho worker B giữ lại rồi mới cho A gửi cả lệnh xác nhận lẫn lệnh giải phóng. Kiểm tra hàng của D9 vẫn thuộc B43 và chỉ có một phân công hiệu lực sau khi B xác nhận. Cuối cùng gửi lại yêu cầu B sau khi mô phỏng mất phản hồi. Dựa trên mã đề nghị đã xử lý, API phải trả lại cùng mã phân công.

Đọc tiếp bài [Đối soát khi chuyến hoàn tất nhưng tiền vẫn đang chờ](/articles/architecture-doi-soat-sau-chuyen)
