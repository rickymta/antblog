---
slug: ba-data-dictionary-don-vi-null
title: "Data dictionary: định nghĩa trường, đơn vị và giá trị rỗng"
summary: "Định nghĩa tiền, thời gian, toạ độ, null và phiên bản đủ để hai hệ thống hiểu cùng dữ liệu; tránh suy ra ý nghĩa từ tên trường."
category: business-analysis
level: beginner
tags: [business-analysis, ba-034, data-dictionary, toi-noi]
language: vi
---

Payload có amount bằng 10000. Đó là đồng hay đơn vị nhỏ nhất của loại tiền tệ nào, là số đã thu hay số muốn hoàn? Kiểu integer không trả lời những câu hỏi nghiệp vụ này.

<div class="tkh-callout"><span class="tkh-callout-title">BA-034 · Phần 5/8 · Cơ bản</span><p>Đầu ra: dictionary cho đề xuất refund P05. Các trường mới chưa có trong lab. Đọc trước <a href="/articles/ba-domain-glossary-mo-hinh-du-lieu">glossary</a>.</p></div>

## 1. Một trường cần nhiều hơn tên và kiểu

Ghi nghĩa, đơn vị, miền giá trị, bắt buộc khi nào, nguồn, người phụ trách, quy tắc cập nhật và ví dụ. Nếu cùng tên nhưng nghĩa khác ở hai hệ thống, mapping phải chỉ ra khác biệt đó.

## 2. Dictionary minh hoạ

| Trường đề xuất | Nghĩa / điều kiện |
|---|---|
| amountVnd | Số đồng nguyên dương muốn hoàn, không vượt hạn mức còn khả dụng |
| paymentId | Định danh khoản thu gốc, không thay bằng tripId |
| requestedAt | Thời điểm ghi nhận yêu cầu theo nguồn chuẩn; định dạng/múi giờ cần chốt |
| providerRef | Mã phía nhà cung cấp; có thể chưa có trước khi họ tiếp nhận |
| status | Trạng thái refund theo enum/version hợp đồng |

## 3. Null, thiếu và rỗng không đồng nghĩa

providerRef chưa được cấp khác với chuỗi rỗng do mapping lỗi. Trường không xuất hiện trong cập nhật có thể nghĩa không thay đổi, trong khi null có thể nghĩa xoá; phải chốt theo endpoint, không suy ra một quy tắc chung cho tất cả.

Với tiền, không đổi null thành 0 để qua validation: chưa biết số đã thu khác đã thu 0. Khi mất dữ liệu, phải giữ lại dấu hiệu để xử lý.

## 4. Thời gian cần mốc nghiệp vụ

Phân biệt thời điểm sự kiện xảy ra, thời điểm hệ thống nhận và thời điểm dữ liệu được ghi. Callback đến muộn có thể mang occurredAt cũ. Không dùng receivedAt thay occurredAt để sắp xếp nghiệp vụ mà chưa đánh giá.

Định dạng phải có offset hoặc quy ước múi giờ rõ; thống nhất độ chính xác và ranh giới kỳ báo cáo. Ngày hiển thị theo giờ địa phương khác với giá trị chuẩn dùng để so sánh.

## 5. Toạ độ và phiên bản là ví dụ tương tự

P04 chưa chốt dữ liệu bản đồ. Nếu thêm latitude/longitude cần hệ quy chiếu, thứ tự, độ chính xác và nguồn. Không coi mọi cặp số là cùng một vị trí.

pickupRevision được đề xuất để phát hiện thay đổi, không phải số chuyến hay timestamp. Ghi rõ revision tăng lúc nào và đối tượng nào sở hữu revision.

## 6. Review bằng dữ liệu xấu

Thử amount âm, số thập phân, currency khác, providerRef thiếu, timestamp không offset, status mới. Với từng trường hợp, nêu hệ thống sẽ từ chối, chờ bổ sung hay xử lý theo hướng tương thích; không tự bỏ trường chưa hiểu rồi tiếp tục giao dịch tiền.

## 7. Thực hành

Viết dictionary cho ba trường, thêm ví dụ hợp lệ và không hợp lệ cùng lý do. Đánh dấu những quyết định chưa có nguồn.

<details><summary>Kiểu string có đủ định nghĩa timestamp không?</summary><p>Không. Cần định dạng, múi giờ, độ chính xác, mốc nghiệp vụ và cách xử lý giá trị không hợp lệ.</p></details>

## 8. Nguồn và bài tiếp

Hồ sơ: `examples/toi-noi-ba/p05-tich-hop.md`. Tiếp theo: [Đọc API bằng ngôn ngữ nghiệp vụ](/articles/ba-doc-api-request-response-loi).
