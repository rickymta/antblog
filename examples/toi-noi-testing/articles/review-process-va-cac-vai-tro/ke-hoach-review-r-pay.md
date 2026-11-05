# Kế hoạch review — Đặc tả thanh toán VíXanh (R-PAY)

| Mục | Nội dung |
|---|---|
| Mã review | RV-PAY-01 |
| Loại review | Review có cấu trúc (individual review + họp + nhật ký + exit criteria); chưa thu metric như inspection |
| Mục đích | Tìm anomaly trong R-PAY trước khi xây Payment & Wallet Service; tạo hiểu chung giữa Payment, Booking, Vận hành, Tài chính |
| Work product | `SPEC.md` v1.1 mục 11 (R-PAY-01…06) |
| Đoạn liên quan (đọc kèm) | R-CANCEL-07; R-API-07, R-API-08; R-QUOTE-04; R-STATE-05, R-STATE-08 (cột Payment); R-NFR-05; R-TEST-03 |
| Đặc tính chất lượng | Đúng · đủ · nhất quán với R-API/R-CANCEL · kiểm thử được · không mơ hồ |
| Vùng tập trung | Số tiền thu · khoá idempotency · hết giờ/thử lại · nghĩa từng `paymentStatus` · chuyến bị huỷ |
| Ngoài phạm vi | Webhook, ledger, đối soát, hoàn tiền (R-ARCH-02) — gặp thì đánh dấu «hoãn», không tính là defect |
| Thông tin hỗ trợ | Kiến trúc §4.9, §8.3; checklist và kịch bản bên dưới |
| Tiêu chí vào | Đặc tả có số phiên bản; tác giả đã tự soát; mọi mã R-… dẫn chiếu tồn tại |
| Exit criteria | (1) 100% mục trong nhật ký có trạng thái + người chịu trách nhiệm; (2) 0 defect Major còn mở; (3) mọi câu hỏi đã trả lời hoặc thành việc có hạn chót; (4) có Major ⇒ review leader + 1 reviewer kiểm bản sửa (follow-up) |
| Nỗ lực (giả định) | Individual review 45 phút × 4 · họp 45 phút × 7 · sửa ≈ 2 giờ ⇒ ≈ 10 giờ công |

## Phân vai

| Vai trò | Người (giả định) | Góc nhìn / việc |
|---|---|---|
| Manager | Engineering Manager squad Payment | Chọn work product, cấp ≈ 10 giờ công |
| Author | BA viết R-PAY | Trả lời câu hỏi, sửa đặc tả |
| Review leader | QA Lead | Chọn người, xếp lịch, kiểm exit criteria, ký báo cáo |
| Moderator | Scrum Master squad Booking | Điều phối họp, giữ giờ, giữ không khí an toàn |
| Scribe | Tester squad Payment | Gộp nhật ký trước họp, ghi quyết định + anomaly mới |
| Reviewer 1 | Dev Payment | Bên cài đặt: có cài đặt đúng một cách không? lỗi nào «tạm thời»? |
| Reviewer 2 | Dev Booking | Bên gọi: trường nào truyền/đọc? khớp R-API không? |
| Reviewer 3 | SRE | Vận hành: hết giờ, thử lại ảnh hưởng thời gian phản hồi, giám sát |
| Reviewer 4 | Chuyên viên đối soát Tài chính (SME) | Từ `paymentStatus` có biết khoản nào đã thu, chưa thu? |

## Lịch 5 hoạt động

| # | Hoạt động | Khi | Ai | Đầu ra |
|---|---|---|---|---|
| 1 | Planning | Thứ Hai 9:00–9:30 | Manager, Review leader | Tài liệu này |
| 2 | Review initiation | Thứ Hai 14:00–14:15 | Review leader, Author, mọi người | Gói review đã gửi; mỗi người xác nhận mở được tài liệu, hiểu vai trò/góc nhìn |
| 3 | Individual review | Thứ Hai chiều → Thứ Ba 12:00 | 4 reviewer | Nhật ký chung (`nhat-ky-review-r-pay.md`) |
| 4 | Communication and analysis | Thứ Ba 15:00–15:45 | Moderator, Scribe, Author, Reviewer, Review leader | Quyết định từng dòng; kết luận về chất lượng work product |
| 5 | Fixing and reporting | Thứ Tư → Thứ Năm | Author, Review leader | Defect report (Major), đặc tả bản sửa, follow-up, báo cáo |

## Checklist 8 mục (individual review)

1. Số tiền thu lấy từ trường nào, có định nghĩa ở đặc tả không, có đúng cho cả `ride`, `parcel`, `food`?
2. Đơn vị tiền và kiểu số (nguyên đồng) có rõ không?
3. Mọi lần gọi ví có khoá idempotency, và lần thử lại có dùng lại đúng khoá?
4. Hết giờ là bao lâu, tính cho mỗi lần hay cả chuỗi?
5. Lỗi nào được thử lại, lỗi nào không — danh sách có đủ và không chồng nhau?
6. Mỗi giá trị `paymentStatus` có đúng một nghĩa? Trạng thái cuối của mọi nhánh có được nêu?
7. Ca số tiền bằng 0 xử lý ra sao (hoặc có bảo đảm không xảy ra)?
8. Mọi kiểu huỷ (khách, tài xế, hệ thống) ảnh hưởng thanh toán thế nào?

## Kịch bản (scenario-based)

- **S1** — Chuyến `vixanh` hoàn thành, ví đã trừ tiền nhưng mất phản hồi (`lost_response`, R-TEST-03). Đi theo R-PAY-04: kết quả cuối, số lần gọi, số lần trừ tiền?
- **S2** — Khách huỷ chuyến xe máy `vixanh` ở `arriving` (phí 10.000 đ, R-CANCEL-03). Khoá nào, trạng thái nào; nếu ví từ chối thì sao?
- **S3** — Đơn đồ ăn trả qua ví: tiền món 120.000 đ, voucher giảm 24.000 đ, phí giao 18.000 đ (số giả định). Ví bị gọi thu bao nhiêu?
