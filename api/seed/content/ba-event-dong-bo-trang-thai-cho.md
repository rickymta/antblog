---
slug: ba-event-dong-bo-trang-thai-cho
title: "Event, đồng bộ và trạng thái chờ: khi kết quả đến sau response"
summary: "Mô hình hóa hoàn tiền bất đồng bộ với callback trễ, lặp và sai thứ tự; thiết kế thông báo chờ cùng đường xử lý ngoại lệ."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-038, event, state-model, toi-noi]
language: vi
---

Khách thấy vòng quay kết thúc không có nghĩa ngân hàng đã hoàn tiền. Giữa thao tác của khách và kết quả cuối còn có nhiều lần giao tiếp, mỗi lần đều có thể chậm hoặc bị mất.

<div class="tkh-callout"><span class="tkh-callout-title">BA-038 · Phần 5/8 · Trung bình</span><p>Đầu ra: timeline và bảng xử lý sự kiện cho refund đề xuất. Không mặc định Tới Nơi đang dùng nhà cung cấp cụ thể.</p></div>

## 1. Tách tiếp nhận và hoàn tất

Luồng đề xuất: nội bộ tạo refund requested, gửi sang đối tác rồi theo dõi processing, sau đó xác nhận succeeded hoặc rejected bằng bằng chứng hợp lệ. Danh sách này chưa phải state machine đầy đủ: quyền chuyển, trạng thái khi gửi thất bại và nguồn có thẩm quyền vẫn cần review.

HTTP 202 diễn đạt yêu cầu đã được tiếp nhận, chưa hoàn tất. Nếu dùng mã này, response nên giúp khách tra cứu tiến độ thay vì chỉ trả một thông báo chung.

## 2. Timeline bộc lộ khoảng trống

| Thời điểm giả lập | Sự kiện | Điều có thể kết luận |
|---|---|---|
| 10:00:00 | Nội bộ lưu refundId | Có yêu cầu nội bộ |
| 10:00:01 | Gửi yêu cầu đối tác | Đã thử gửi |
| 10:00:06 | Hết thời gian đợi response | Chưa biết đối tác đã xử lý chưa |
| 10:00:20 | Callback succeeded hợp lệ | Có bằng chứng cần kiểm tra và ghi nhận |

Khoảng 14 giây cuối không phải một lần hoàn tiền thất bại. Nếu UI tạo yêu cầu mới trong khoảng này, có thể phát sinh hai ý định hoàn tiền.

## 3. Callback là đầu vào cần kiểm chứng

Đề xuất kiểm xác thực callback theo hợp đồng, liên kết refundId/providerRef, số tiền/currency và event ID. Callback hợp lệ về chữ ký vẫn có thể không khớp giao dịch nội bộ; khi đó không cập nhật bừa.

Tài liệu webhook của Stripe là ví dụ thực tế về giao nhận lặp và không bảo đảm thứ tự. Điều này gợi câu hỏi review; không có nghĩa hợp đồng của mọi đối tác giống Stripe.

## 4. Trễ và sai thứ tự

| Trạng thái đã biết | Sự kiện đến | Đề xuất |
|---|---|---|
| processing | succeeded hợp lệ | Ghi nhận hoàn tất một lần |
| succeeded | succeeded lặp | Không ghi thêm khoản hoàn |
| succeeded | processing cũ | Không lùi trạng thái |
| succeeded | rejected mâu thuẫn | Đối soát, không tự đảo kết quả |

Chỉ so timestamp có thể không đủ khi đồng hồ hai bên khác nhau. Cần biết version/sequence hoặc cơ chế tra cứu có thẩm quyền.

## 5. Thông báo cho khách

“Đang xác nhận kết quả hoàn tiền” phù hợp hơn “Thất bại, hãy thử lại” khi chỉ timeout. Cần hiển thị mã tham chiếu và đường hỗ trợ; thời gian dự kiến chỉ được đưa ra khi có căn cứ vận hành. Không tự hứa “tiền về ngay” từ trạng thái nội bộ succeeded nếu định nghĩa của đối tác khác.

## 6. Ai xử lý yêu cầu chờ lâu?

Đặt owner cho hàng đợi quá hạn và quy tắc tra cứu/đối soát. Ngưỡng chờ, chu kỳ kiểm tra và quyền can thiệp là quyết định còn mở. Nếu thiếu các quyết định này, hệ thống có thể “không mất dữ liệu” nhưng khách vẫn mắc kẹt vô thời hạn.

## 7. Thực hành

Vẽ lại timeline khi callback đến trước response, rồi thêm callback lặp sau một giờ. Với mỗi bước ghi trạng thái, thông báo UI và tác động tiền.

<details><summary>Điểm phải giữ</summary><p>Response đến muộn không ghi đè kết quả cuối đã được xác nhận. Callback lặp không tạo tác động tiền lần hai. Mâu thuẫn cần lưu dấu và đối soát.</p></details>

## 8. Nguồn và bài tiếp

[RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html) về HTTP 202; [Stripe webhooks](https://docs.stripe.com/webhooks) về ví dụ giao nhận sự kiện. Tiếp: [idempotency và retry](/articles/ba-idempotency-retry-giao-dich-loi).
