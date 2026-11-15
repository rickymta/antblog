---
slug: architecture-hop-dong-su-kien-kafka
title: "Đổi hợp đồng sự kiện Kafka khi các consumer nâng cấp khác ngày"
summary: "Bổ sung mã tiền tệ vào sự kiện báo giá, kiểm tra hai chiều tương thích và giữ khả năng đọc lại dữ liệu lịch sử."
category: architecture
level: advanced
tags: [architecture, dotnet, kafka, schema-evolution, contract-testing]
language: vi
---

Báo cáo doanh thu của Tới Nơi đang cộng trường `amountVnd` trong sự kiện báo giá. Nhóm Pricing muốn hỗ trợ loại tiền thứ hai. Nếu đổi trường này thành một con số mang nghĩa “số tiền theo tiền tệ của chuyến”, consumer cũ vẫn có thể đọc thành công rồi cộng đô la vào cột đồng Việt Nam. Một bản tin hợp lệ về cú pháp vẫn có thể làm sai báo cáo.

## Giữ nghĩa của dữ liệu đã phát

Giả sử hợp đồng cũ quy định mọi báo giá đều bằng VND. Trong giai đoạn chuyển đổi, producer chỉ phát báo giá VND và điền đồng thời trường cũ lẫn biểu diễn mới. Bản ghi minh hoạ dưới đây chỉ trích phần dữ liệu nghiệp vụ.

```json
{
  "quoteId": "Q42",
  "amountVnd": 85000,
  "price": { "minorUnits": 85000, "currency": "VND" }
}
```

Hợp đồng cần ghi rõ `minorUnits` là số nguyên tính theo đơn vị nhỏ nhất của từng tiền tệ và xác định bộ mã tiền tệ được hỗ trợ. Với VND, giá trị 85000 biểu thị 85.000 đồng. Consumer .NET 10 mới ưu tiên đọc `price`. Khi đọc dữ liệu lịch sử chỉ có `amountVnd`, nó có thể suy ra VND từ nghĩa đã công bố của trường cũ. Thiếu cả hai trường là lỗi dữ liệu cần ghi nhận.

## Triển khai qua những ngày khác nhau

| Dữ liệu | Consumer được thử | Điều phải xác minh |
|---|---|---|
| Bản ghi cũ chỉ có `amountVnd` | Consumer mới dựng lại báo cáo | Khôi phục đúng 85.000 VND |
| Bản ghi chuyển tiếp có cả hai trường | Phiên bản consumer đang chạy | Đọc đúng trường cũ dù payload có thuộc tính mới |
| Bản ghi có `price` bằng ngoại tệ | Consumer mới | Ghi đúng loại tiền và tổng hợp riêng từng loại |

Trước khi producer phát hai trường, chạy hàng thứ hai với đúng cấu hình giải tuần tự của consumer cũ. Một reader từ chối thuộc tính lạ sẽ cần được nâng cấp trước. Sau khi triển khai reader mới, theo dõi các bản ghi có hai biểu diễn VND bất nhất để phát hiện lỗi chuyển đổi tại nguồn.

Chỉ bật ngoại tệ sau khi mọi hệ thống còn nhận luồng đó đều hiểu nghĩa mới. Nếu một hệ thống kế toán cũ bắt buộc phải tiếp tục chạy, có thể mở topic phiên bản mới và duy trì topic cũ với phạm vi VND đã thoả thuận. Hai topic cần dùng chung mã sự kiện cho cùng một báo giá để đối chiếu và tránh đếm hai lần trong thời gian chuyển giao.

## Thử dựng lại báo cáo tháng trước

Lưu các payload đại diện vào bộ kiểm thử hợp đồng. Chạy consumer mới trên tập bản ghi trộn giữa dữ liệu cũ, dữ liệu chuyển tiếp và ngoại tệ. So sánh tổng theo từng mã tiền, kiểm tra số bản ghi lỗi và chạy lại cùng tập dữ liệu với kho kết quả trống. Chỉ xoá nhánh đọc `amountVnd` khi đã xét cả dữ liệu lưu trữ dùng để phục hồi. Việc mọi producer đã nâng cấp chỉ cho biết định dạng của bản ghi sắp tới. [Cách Kafka lưu bản ghi để consumer đọc lại](/articles/architecture-kafka)

Đọc tiếp bài [Realtime cho tài xế khi kết nối SignalR bị gián đoạn](/articles/architecture-signalr-khoi-phuc-trang-thai).
