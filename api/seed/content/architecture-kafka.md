---
slug: architecture-kafka
title: "Message Broker với Kafka qua sự kiện nhận chuyến"
summary: "Theo chân một sự kiện từ outbox đến hai consumer group, rồi thử dừng consumer giữa lúc ghi dữ liệu và commit offset."
category: architecture
level: intermediate
tags: [architecture, kafka, message-broker, outbox]
language: vi
---

Sau khi tài xế nhận chuyến T42, màn hình điều phối cần bỏ chuyến khỏi danh sách chờ và hệ thống phân tích cần đếm một lượt nhận chuyến. Nếu phân tích dừng bảo trì nửa giờ, điều phối vẫn cần cập nhật ngay. Kafka làm trung gian giữa bên phát và các bên đọc bằng cách lưu các sự kiện vào một nhật ký có thể đọc lại.

## Theo dấu một bản ghi

Trong backend .NET 10 của Tới Nơi, dịch vụ chuyến xe cập nhật T42 và thêm sự kiện vào bảng outbox trong cùng giao dịch. Một tiến trình đọc các hàng đã commit rồi gửi đến topic `trip-events`. Nhờ vậy, nếu tiến trình dừng ngay sau khi giao dịch hoàn tất, sự kiện vẫn còn trong outbox để gửi tiếp. Ví dụ dưới đây là giá trị của bản ghi, còn khoá Kafka được đặt là `T42`.

```json
{
  "eventId": "evt-817",
  "type": "TripAccepted",
  "tripId": "T42",
  "driverId": "D9",
  "tripVersion": 8
}
```

Topic gồm nhiều partition. Khi dùng cùng cách phân vùng theo khoá và giữ nguyên số partition, các bản ghi của T42 đi vào cùng một partition. Broker gán cho mỗi bản ghi một vị trí gọi là *offset*. Thứ tự offset phản ánh thứ tự ghi vào partition. Nếu nhiều tiến trình cùng xuất outbox, phía phát vẫn phải phối hợp để gửi các phiên bản của T42 đúng thứ tự nghiệp vụ. [Các khái niệm của Apache Kafka](https://kafka.apache.org/intro/)

## Hai nhóm đọc cùng một sự kiện

Đặt consumer điều phối vào group `dispatch`, consumer phân tích vào group `analytics`. Mỗi group theo dõi vị trí đọc riêng trên từng partition. Khi analytics chạy lại, nó tiếp tục từ offset đã commit của chính nó, miễn là dữ liệu còn trong thời hạn lưu giữ. Nếu đặt hai consumer vào cùng một group, Kafka sẽ chia partition cho chúng và một trong hai chức năng có thể bỏ lỡ sự kiện cần xử lý.

## Nếu consumer dừng ngay sau khi cập nhật PostgreSQL

Giả sử analytics đã tăng bộ đếm cho `evt-817` và commit giao dịch PostgreSQL nhưng chưa kịp commit offset Kafka. Sau khi khởi động lại, nó có thể đọc lại bản ghi đó. Để bộ đếm vẫn đúng, consumer có thể xử lý tuần tự từng partition theo các bước sau.

1. Mở giao dịch PostgreSQL và ghi `eventId` vào bảng sự kiện đã xử lý của analytics với ràng buộc duy nhất. Dùng `ON CONFLICT DO NOTHING RETURNING event_id` để biết lần ghi này có thêm hàng hay không.
2. Chỉ tăng bộ đếm khi vừa thêm được hàng. Commit cả dấu đã xử lý và thay đổi bộ đếm trong cùng giao dịch.
3. Commit vị trí kế tiếp cho partition sau khi giao dịch thành công. Nếu lần commit offset thất bại, lần đọc lại sẽ gặp dấu đã xử lý.

Nếu triển khai consumer bằng một `BackgroundService` của .NET 10 với Confluent.Kafka, cấu hình `EnableAutoCommit = false` và chỉ gọi `Commit(consumeResult)` sau khi giao dịch PostgreSQL đã commit. Khi worker nhận tín hiệu dừng giữa chừng, bản ghi chưa commit offset có thể được giao lại cho worker khác trong group.

Có thể kiểm chứng bằng cách chủ động dừng consumer giữa bước 2 và bước 3. Khi chạy lại, kiểm tra bộ đếm chỉ tăng một lần và offset cuối cùng tiến lên. Nếu cần dựng lại báo cáo từ đầu, hãy dùng bảng kết quả và tập dấu đã xử lý mới, đồng thời kiểm tra mốc dữ liệu sớm nhất Kafka còn giữ. Đặt lại offset chỉ cho phép đọc lại những bản ghi vẫn còn trong topic.
