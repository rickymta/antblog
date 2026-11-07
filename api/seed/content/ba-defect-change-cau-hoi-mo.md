---
slug: ba-defect-change-cau-hoi-mo
title: "Defect, change request hay câu hỏi chưa được chốt?"
summary: "Phân loại phản hồi bằng baseline và bằng chứng, tránh dùng nhãn bug hoặc change để đẩy trách nhiệm hoặc bỏ sót vấn đề nghiệp vụ."
category: business-analysis
level: beginner
tags: [business-analysis, ba-051, defect, change-request, toi-noi]
language: vi
---

Người nghiệm thu nói “sai rồi”, developer nói “đúng tài liệu”. Trước khi tranh luận ai đúng, BA cần xác định tài liệu nào, phiên bản nào và điều gì thực sự xảy ra.

<div class="tkh-callout"><span class="tkh-callout-title">BA-051 · Phần 7/8 · Cơ bản</span><p>Đầu ra: triage log với căn cứ và bước tiếp. Ví dụ dựa trên P04 và baseline phí huỷ P03; không phải lỗi đã phát hiện trong sản phẩm thật.</p></div>

## 1. Thu bằng chứng trước nhãn

Ghi actor, trạng thái, thao tác, expected, actual, build và nguồn expected. Kiểm môi trường/dữ liệu có đúng không. Nếu người dùng nhầm chuyến hoặc bản UAT chưa được cập nhật, cần sửa điều kiện kiểm trước khi kết luận sản phẩm sai.

## 2. Ba hướng xử lý

Defect là sai lệch so với yêu cầu hoặc kỳ vọng hợp lệ được dùng làm căn cứ kiểm tra. Change request đề xuất hành vi mới hoặc thay baseline. Câu hỏi mở xuất hiện khi căn cứ chưa rõ hoặc mâu thuẫn; cần quyết định trước khi biết nên sửa gì.

Không phải mọi defect đều nằm ở code. Yêu cầu có thể thiếu hoặc mâu thuẫn với nhu cầu đã được chấp nhận; quy trình triage của nhóm cần cho phép ghi loại vấn đề đó.

## 3. Phân loại ví dụ

| Phản hồi giả lập | Căn cứ | Xử lý ban đầu |
|---|---|---|
| Người khác xác nhận được | RULE-P04-01 chỉ chủ chuyến | Defect cần điều tra |
| Muốn sửa điểm sau accepted | Ngoài phạm vi P04 | Change request |
| Không rõ timeout nên báo gì | Nếu AC-P04-05 có outcome thì đối chiếu | Defect hoặc câu hỏi tùy baseline thực |
| Muốn miễn phí tại accepted 150 giây | Baseline dưới 120 giây | CR-BA-01, không tự đổi rule |

Nhãn có thể cập nhật khi có bằng chứng mới. Giữ lịch sử lý do thay đổi, tránh sửa ticket làm mất dấu cuộc trao đổi.

## 4. Severity và priority khác nhau

Severity mô tả mức tác động của vấn đề; priority là thứ tự xử lý trong bối cảnh hiện tại. Lỗi quyền có thể ảnh hưởng nghiêm trọng dù ca khó gặp. Một lỗi câu chữ dễ thấy có thể ưu tiên sửa nhanh nhưng không vì vậy có tác động cao hơn lỗi tiền.

Định nghĩa thang dùng chung trong dự án; đừng áp nhãn P1/P2 khi mỗi đội hiểu khác.

## 5. Tranh chấp không giải bằng tên gọi

Đánh dấu change không làm nhu cầu biến mất. Đánh dấu bug cũng không miễn impact analysis. Khi lỗi yêu cầu gây hành vi không phù hợp, cần xem rủi ro, phương án và người quyết định, thay vì dùng nhãn để quyết ai trả tiền hoặc ai “thua”.

## 6. Triage log tối thiểu

Mỗi mục có ID, bằng chứng, baseline, phân loại tạm, tác động, owner và bước tiếp. Nếu còn tranh chấp, ghi chính xác câu hỏi và hạn trả lời. Một ticket “needs clarification” không owner có thể chặn UAT lâu hơn việc sửa code.

## 7. Thực hành

Tài liệu nói đúng revision mới được xác nhận, prototype lại không có thông báo stale. Người dùng gặp stale và không biết làm gì. Hãy tách lỗi chức năng khỏi khoảng trống trải nghiệm.

<details><summary>Gợi ý</summary><p>Nếu hệ thống ghi nhận revision cũ thì sai rule. Nếu đã từ chối đúng nhưng hướng dẫn chưa rõ, review yêu cầu phản hồi và prototype để xác định defect hay yêu cầu cần bổ sung. Hai vấn đề có thể cùng tồn tại.</p></details>

## 8. Đọc tiếp

Ôn [baseline và approval](/articles/ba-baseline-version-approval). Tiếp: [nghiệm thu có điều kiện và go/no-go](/articles/ba-nghiem-thu-co-dieu-kien-go-no-go).
