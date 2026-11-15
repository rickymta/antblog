---
slug: architecture-doi-soat-sau-chuyen
title: "Chuyến đã hoàn tất, tiền vẫn đang chờ: đối soát sau chuyến"
summary: "Theo dấu một lần thu tiền mất webhook, từ phát hiện khoản lệch đến cập nhật sổ giao dịch và trả lời khách."
category: architecture
level: advanced
tags: [architecture, dotnet, payment, reconciliation, idempotency]
language: vi
---

Chuyến T42 đã kết thúc. Payment gửi lệnh thu 85.000 đồng, sau đó kết nối tới nhà cung cấp thanh toán bị ngắt. Khách nhìn thấy giao dịch trong ứng dụng ngân hàng nhưng biên nhận của Tới Nơi vẫn báo đang xử lý. Nếu nhân viên hỗ trợ bấm thu lại với một mã mới, khách có thể bị tính tiền hai lần.

## Giữ riêng trạng thái chuyến và trạng thái tiền

Trip có đủ căn cứ để ghi nhận hành trình đã kết thúc. Payment vẫn cần bằng chứng về kết quả của lần thu tiền. Trước khi gọi nhà cung cấp, Payment lưu mã thao tác ổn định cho lần thu, mã đơn, số tiền và tiền tệ dự kiến cùng mã tham chiếu của nhà cung cấp nếu đã có. Khi mất phản hồi, đánh dấu lần thu đang chờ xác minh. Việc khôi phục bắt đầu bằng tra cứu cùng thao tác đó. Đây là bước tiếp theo của tình huống mất phản hồi trong [bài Saga đặt chuyến](/articles/architecture-saga-giu-tien-tim-xe).

## Một lượt đối soát đi đến quyết định nào?

Một `BackgroundService` trong .NET 10 có thể quét các lần thu còn chờ quá ngưỡng vận hành đã thoả thuận với nhà cung cấp. Mỗi lượt tra ghi lại thời điểm, nguồn bằng chứng và kết quả. Việc đối chiếu phải dùng tài khoản người bán, mã giao dịch, số tiền và tiền tệ. Hai giao dịch cùng 85.000 đồng trong cùng phút vẫn có thể thuộc hai khách khác nhau.

| Bằng chứng nhận được | Hành động của Payment |
|---|---|
| Nhà cung cấp xác nhận đã thu đúng số tiền và loại tiền | Ghi bút toán thu và chuyển lần thu sang thành công trong cùng giao dịch |
| Nhà cung cấp vẫn đang xử lý hoặc tra cứu tạm lỗi | Giữ trạng thái chờ, hẹn lần tra sau với khoảng cách tăng dần |
| Đã thu nhưng số tiền hoặc tiền tệ khác dự kiến | Lưu bằng chứng, mở hồ sơ chênh lệch để người có thẩm quyền xử lý |
| Nhà cung cấp xác nhận lần thu thất bại dứt điểm | Kết thúc lần thu hiện tại. Việc yêu cầu thanh toán lại tuân theo chính sách của sản phẩm |

Nếu nhà cung cấp cho tải báo cáo giao dịch, đối chiếu định kỳ theo cả chiều ngược lại. Một dòng trong báo cáo không ghép được với bất kỳ lần thu nội bộ nào sẽ bị bỏ sót nếu worker chỉ quét các hàng ở trạng thái chờ. Cần phân biệt thời điểm khách bị thu tiền với thời điểm nhà cung cấp chuyển doanh thu cho người bán.

## Webhook về đúng lúc worker đang ghi

Cả webhook đã xác thực và worker tra cứu đều đi qua cùng hàm áp dụng kết quả. Trong một giao dịch, kiểm tra chuyển trạng thái hợp lệ và dùng ràng buộc duy nhất theo nhà cung cấp, mã lần thu và loại bút toán. Một lần thu thành công sau đó nhận thêm bằng chứng trùng có thể bổ sung lịch sử đối soát mà không cộng tiền lần hai. Một webhook cũ mang trạng thái đang xử lý cũng không được kéo kết quả thành công lùi lại.

Hãy thử cho nhà cung cấp giả lập thu tiền rồi ngắt kết nối, tiếp theo trả kết quả tra cứu thành công đồng thời gửi hai webhook. Kiểm tra có đúng một bút toán 85.000 đồng và cả ba lần nhận bằng chứng đều truy vết được. Màn hình hỗ trợ cần trả lời được ba câu: lần thu nào đang được hỏi, bằng chứng gần nhất đến từ đâu và bước xử lý kế tiếp là gì.

Đọc tiếp bài [Thay thuật toán điều phối mà vẫn đo được tác động](/articles/architecture-thu-nghiem-thuat-toan-dieu-phoi)
