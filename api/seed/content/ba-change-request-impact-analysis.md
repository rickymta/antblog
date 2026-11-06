---
slug: ba-change-request-impact-analysis
title: "Change request và impact analysis: thay một phút, ảnh hưởng những đâu?"
summary: "Lập hồ sơ thay đổi có delta, tác động và phương án; dùng CR-BA-01 để kiểm rule, dữ liệu, giao diện, vận hành và bằng chứng kiểm thử."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-045, change-request, impact-analysis, toi-noi]
language: vi
---

Yêu cầu “đổi hai thành ba” chỉ mô tả một phần delta. Nó chưa nói chuyến nào được hưởng, khách đã nhận thông báo cũ thì sao, hoặc gửi lại yêu cầu huỷ có tính lại phí không.

<div class="tkh-callout"><span class="tkh-callout-title">BA-045 · Phần 6/8 · Trung bình</span><p>Đầu ra: impact map và lựa chọn cho CR-BA-01. Đọc baseline P03; mọi thay đổi vẫn là đề xuất chưa được duyệt.</p></div>

## 1. Viết delta có giới hạn

Baseline miễn phí khách huỷ ở accepted khi elapsed < 120 giây. CR đề xuất < 180 giây. Không tự mở rộng sang arriving, không đổi ngưỡng miễn do tài xế trễ, mức phí hoặc luồng driver_cancel. Ghi cả phần giữ nguyên giúp reviewer phát hiện cách hiểu rộng hơn yêu cầu.

## 2. Tìm lý do trước giải pháp

Người đề xuất cần nêu vấn đề và kỳ vọng đo được. Ví dụ “khách cần thêm thời gian đọc thông tin” là giả thuyết cần kiểm; chưa đủ để kết luận thêm một phút làm giảm khiếu nại. Hỏi có phương án thông báo rõ hơn hoặc giữ nguyên kèm discovery không.

Mục tiêu giảm huỷ 12% xuống 9% của P01 không tự trở thành business case cho miễn phí lâu hơn. Chính sách có thể khiến khách huỷ nhiều hơn nhưng ít tranh chấp hơn; cần xác định outcome muốn ưu tiên.

## 3. Impact map

| Miền | Câu hỏi cần review | Sản phẩm bị ảnh hưởng |
|---|---|---|
| Rule | Chính xác 180 giây tính phí? | Decision table, AC |
| Dữ liệu | Xác định phiên bản có hiệu lực ra sao? | Thiết kế lưu/đọc policy |
| UI | Nội dung hiển thị theo phiên bản nào? | Màn hình và thông báo |
| API | Replay cũ có được giữ? | Contract, case idempotency |
| CS | Giải thích chuyến qua mốc thế nào? | Hướng dẫn và công cụ tra cứu |
| Đo lường | So cohort theo policy nào? | Metric dictionary |

Mỗi hàng cần owner trả lời. “Không ảnh hưởng” là kết luận sau review, không phải mặc định khi chưa tìm thấy file.

## 4. Case biên cụ thể

Ở accepted, không thuộc miễn do trễ: 119 giây miễn ở cả hai rule; 120 và 179 giây thay đổi kết quả; 180 giây vẫn có phí. Ở arriving, CR không tạo khoảng miễn ba phút. Thêm ca đúng ngưỡng trễ và vượt ngưỡng vì điều kiện baseline dùng dấu lớn hơn.

Khi tài xế rời rồi nhận lại, mốc acceptedAt thay đổi theo baseline. Cách chọn phiên bản chính sách ở lần nhận mới phải được chốt riêng.

## 5. So phương án thay đổi

Có thể hoãn CR, áp cho cohort mới có phiên bản rõ, hoặc chia discovery và triển khai thành hai đợt. Mỗi phương án có lợi ích, chi phí, rủi ro và điều kiện tiên quyết. Không coi feature flag là lời giải đầy đủ nếu chưa biết giao dịch qua mốc chọn policy thế nào.

## 6. Kết luận có người quyết định

BA tổng hợp tác động, nhóm kỹ thuật và QA cung cấp estimate/bằng chứng, nghiệp vụ chấp nhận thay đổi trong thẩm quyền. Người chịu trách nhiệm phát hành cân nhắc readiness. Những trách nhiệm này có thể thuộc các cá nhân khác nhau; không mặc định một chữ ký thay mọi quyết định.

## 7. Thực hành

Viết một trang CR gồm lý do, delta, sáu miền tác động, hai phương án và ba câu hỏi còn mở. Với mỗi câu hỏi ghi ai trả lời và việc nào bị chặn.

<details><summary>Lỗi dễ gặp</summary><p>Chỉ sửa case 120 giây mà quên arriving, replay và mốc hiệu lực. Impact analysis cần tìm cả chỗ phải đổi lẫn điều kiện không được vô tình đổi.</p></details>

## 8. Đọc tiếp

[Decision table phí huỷ](/articles/ba-decision-table-phi-huy) cung cấp baseline. Tiếp: [ưu tiên theo giá trị và độ tin cậy](/articles/ba-uu-tien-gia-tri-chi-phi-do-tin-cay).
