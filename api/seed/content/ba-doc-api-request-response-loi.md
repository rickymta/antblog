---
slug: ba-doc-api-request-response-loi
title: "Đọc API request, response và mã lỗi bằng ngôn ngữ nghiệp vụ"
summary: "Lần theo báo giá và đặt chuyến để hiểu quyền, thời hạn, lỗi và kết quả API; phân biệt tiếp nhận yêu cầu với hoàn thành nghiệp vụ."
category: business-analysis
level: beginner
tags: [business-analysis, ba-035, api, contract, toi-noi]
language: vi
---

Response 200 không tự giải thích “thành công” là gì. Trong lab, huỷ chuyến có thể thành công nhưng thu ví failed. BA đọc API cần nối cấu trúc dữ liệu với outcome nghiệp vụ.

<div class="tkh-callout"><span class="tkh-callout-title">BA-035 · Phần 5/8 · Cơ bản</span><p>Đầu ra: bản đọc hợp đồng báo giá và danh sách câu hỏi. Nguồn baseline: SPEC R-QUOTE/R-VALID/R-IDEM; không gọi API thật.</p></div>

## 1. Đọc theo năm câu hỏi

Ai gọi, muốn làm gì, cần gửi dữ kiện nào, kết quả nào thay đổi, lỗi nào có thể xảy ra? Sau đó mới xem tên trường/mã HTTP. Cùng method và URL vẫn có thể cho outcome khác theo actor/state.

## 2. Báo giá là đối tượng có hiệu lực

SPEC lưu quoteId, createdAt, expiresAt, pricingVersion và chi tiết giá. Hiệu lực là `now < expiresAt`; đúng mốc là hết hạn. Một số tiền nhìn thấy trên màn hình không phải cam kết vô thời hạn.

Khi tạo chuyến phải dùng báo giá của chính khách đó. Chuyến giữ ảnh chụp giá theo báo giá; không tự tính lại vì thời tiết hoặc giờ cao điểm đã đổi.

## 3. Đọc thứ tự lỗi

| Tình huống theo R-QUOTE-03 | Kết quả |
|---|---|
| Không có hoặc không thuộc khách | QUOTE_NOT_FOUND, 422 |
| Hết hạn | QUOTE_EXPIRED, 422 |
| Đã dùng | QUOTE_ALREADY_USED, 409 |
| Vượt lượt voucher | VOUCHER_USAGE_LIMIT, 422 |

Nếu nhiều điều kiện sai, thứ tự kiểm trong SPEC quyết định lỗi đầu tiên. UI cần cách phản hồi phù hợp; không chỉ hiện “có lỗi”.

## 4. Status code và nghiệp vụ

[RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html) định nghĩa 202 là tiếp nhận để xử lý, chưa cam kết hoàn tất. Với refund đề xuất, nếu chọn 202 phải có cách theo dõi kết quả; không hiển thị “đã hoàn tiền” ngay.

Đây là ví dụ thiết kế, không nói API refund 202 đã có trong lab. Mã lỗi chi tiết và response schema cần hợp đồng riêng.

## 5. Retry phải đọc cả hợp đồng

Tạo chuyến yêu cầu Idempotency-Key theo lab. Gửi lại cùng key/payload trả kết quả đã lưu trong điều kiện hợp đồng; không dùng key mới mỗi lần mạng lỗi. Các lỗi trước bước idempotency có xử lý khác, cần đọc R-IDEM/R-VALID thay vì suy từ một response.

## 6. Câu hỏi BA cần mang tới review

Thời hạn đo theo nguồn nào? Ai xem được dữ liệu? Có thể xử lý lặp không? Lỗi có làm thay đổi một phần không? Kết quả nào cần người dùng hành động, kết quả nào nên hệ thống tự phục hồi? Những câu hỏi này ảnh hưởng trực tiếp trải nghiệm và kiểm thử.

## 7. Thực hành

Viết luồng khi khách bấm đặt đúng expiresAt, và khi mất response sau tạo chuyến. Nêu điều UI nên thông báo và dữ kiện cần giữ để không tạo thêm chuyến.

<details><summary>Vì sao không suy “200 = đã thu tiền”?</summary><p>200 diễn đạt thành công theo ngữ nghĩa endpoint. Endpoint huỷ có thể hoàn tất huỷ nhưng paymentStatus failed; phải đọc outcome và rule.</p></details>

## 8. Nguồn và bài tiếp

SPEC lab R-QUOTE/R-IDEM; [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html). Tiếp theo: [SQL cho BA](/articles/ba-sql-kiem-chung-gia-thuyet).
