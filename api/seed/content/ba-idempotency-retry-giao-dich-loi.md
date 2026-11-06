---
slug: ba-idempotency-retry-giao-dich-loi
title: "Idempotency và retry: giữ một ý định nghiệp vụ qua nhiều lần gửi"
summary: "Phân tích key, payload, timeout và cạnh tranh đồng thời để tránh hoàn tiền lặp; tách chống gửi lại khỏi kiểm soát hạn mức."
category: business-analysis
level: advanced
tags: [business-analysis, ba-039, idempotency, retry, toi-noi]
language: vi
---

“API có idempotency rồi” chưa trả lời được liệu hai nhân viên có hoàn vượt tiền đã thu hay không. Chống lặp cùng yêu cầu và bảo vệ tổng tiền là hai điều kiện riêng.

<div class="tkh-callout"><span class="tkh-callout-title">BA-039 · Phần 5/8 · Nâng cao</span><p>Đầu ra: ma trận retry và bất biến tiền cho refund đề xuất. Không sao chép thời hạn key hay mã lỗi của lab đặt chuyến sang hợp đồng mới.</p></div>

## 1. Định nghĩa cùng một ý định

Khách hoặc nhân viên gửi hoàn 30.000 cho payment P, bị timeout và gửi lại. Đây là cùng ý định, cần giữ key và payload tương ứng. Một khoản hoàn bổ sung đã được duyệt là ý định mới, cần phân biệt bằng hồ sơ nghiệp vụ, không chỉ bằng lần bấm nút.

BA cần chốt key do ai tạo, lưu qua reload thế nào, scope theo tài khoản/endpoint nào và tồn tại bao lâu. Thiếu scope có thể tạo xung đột giữa hai người không liên quan.

## 2. Ma trận gửi lại

| Lần gửi sau | Câu hỏi hợp đồng phải trả lời |
|---|---|
| Cùng key, cùng payload, đã hoàn tất | Trả outcome nào, có giữ response gốc không? |
| Cùng key, payload khác | Từ chối rõ, không âm thầm đổi yêu cầu |
| Cùng key, lần đầu đang chạy | Đợi, trả processing hay conflict? |
| Key đã hết hạn lưu | Tra cứu trước khi gửi, tránh tạo mới ngoài ý muốn |
| Key mới, cùng payment và amount | Có thể là ý định mới; không tự coi là replay |

Trong đề xuất P05, cùng ý định không tạo tác động tiền lần hai. Chi tiết mã lỗi và thời hạn vẫn mở, phải chốt trước triển khai.

## 3. Không áp một hợp đồng cho mọi dịch vụ

Stripe API v1 mô tả việc lưu response sau khi endpoint bắt đầu thực thi, kể cả lỗi 500; API v2 có ngữ nghĩa retry khác. Đây là lý do phải ghi phiên bản hợp đồng đang tích hợp. “Idempotent” không đủ để suy mọi lần retry trả cùng body hoặc tự chữa mọi lỗi.

## 4. Retry có giới hạn và đường kết thúc

Phân biệt lỗi validation cần sửa dữ liệu, lỗi quyền cần xử lý quyền, và lỗi truyền thông chưa rõ outcome. Với trường hợp cuối, ưu tiên tra cứu và gửi lại theo hợp đồng. Số lần thử, khoảng cách và tổng thời gian là quyết định kỹ thuật cần gắn với trải nghiệm chờ và owner ngoại lệ.

Hết retry budget không chứng minh tiền chưa chuyển. Chuyển sang xử lý ngoại lệ có mã tham chiếu, thay vì tự phát một yêu cầu hoàn khác.

## 5. Hai key khác nhau vẫn có thể gây vượt tiền

Payment đã thu 100.000. Hai yêu cầu 60.000 và 50.000 cùng đọc số khả dụng 100.000 rồi cùng được chấp nhận. Mỗi key chỉ chạy một lần nhưng tổng là 110.000.

Bất biến đề xuất: tổng đã hoàn thành cộng tiền đang giữ cho refund processing không vượt capturedAmount. Kiểm và giữ hạn mức phải nhất quán khi cạnh tranh đồng thời; lựa chọn transaction/lock cụ thể thuộc thiết kế kỹ thuật.

## 6. Giải phóng tiền giữ cũng cần bằng chứng

Nếu timeout rồi giải phóng ngay khoản giữ 60.000, yêu cầu mới 50.000 có thể được nhận trước callback succeeded của khoản cũ. Chỉ giải phóng theo transition có bằng chứng và rule được duyệt. Cần định nghĩa cách xử lý yêu cầu mắc kẹt, không giữ mãi cũng không tự xóa.

## 7. Thực hành

Viết ba chuỗi sự kiện: replay cùng key, hai key cạnh tranh, timeout rồi callback muộn. Mỗi chuỗi phải có số tiền khả dụng ở từng bước và bằng chứng quyết định.

<details><summary>Tiêu chí đạt</summary><p>Không có tác động tiền lặp cho cùng ý định; tổng giữ và hoàn không vượt tiền thu; unknown không bị biến thành failed chỉ vì hết thời gian chờ. Mỗi ngoại lệ có owner và cách xác minh.</p></details>

## 8. Nguồn và bài tiếp

[Stripe idempotent requests](https://docs.stripe.com/api/idempotent_requests) là hợp đồng tham khảo có phiên bản, không phải yêu cầu Tới Nơi. Tiếp: [review hợp đồng hoàn tiền](/articles/ba-thuc-hanh-hop-dong-hoan-tien).
