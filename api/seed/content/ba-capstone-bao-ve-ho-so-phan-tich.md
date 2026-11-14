---
slug: ba-capstone-bao-ve-ho-so-phan-tich
title: "Capstone: bảo vệ hồ sơ BA từ discovery đến giá trị"
summary: "Tổng hợp tám phần thành portfolio có nguồn, quyết định, traceability và giới hạn; thực hành bảo vệ khuyến nghị khi bằng chứng thay đổi."
category: business-analysis
level: advanced
tags: [business-analysis, ba-064, capstone, portfolio, toi-noi]
language: vi
---

Một portfolio nhiều tài liệu chưa chắc cho thấy năng lực phân tích. Người review cần lần theo được từ vấn đề tới bằng chứng, lựa chọn, yêu cầu, nghiệm thu và quyết định sau khi đo kết quả.

<div class="tkh-callout"><span class="tkh-callout-title">BA-064 · Phần 8/8 · Nâng cao · Capstone</span><p>Đầu ra: hồ sơ BA liên kết và buổi bảo vệ 20 phút. Đây là bài tập cá nhân/nhóm trên Tới Nơi hư cấu, không phải chứng nhận nghề nghiệp hoặc dự án khách hàng thật.</p></div>

## 1. Chọn trục chính cho hồ sơ

Dùng vấn đề khó gặp nhau ở điểm đón từ P01 tới P04 và dữ liệu đánh giá giả lập P07. P05 hoàn tiền, P06 CR phí và P08 multi-stop là bài tập mở rộng kỹ năng; không gộp thành một dự án đã triển khai tất cả tính năng.

Ghi rõ vai trò của bạn trong bài tập, phần tự làm, công cụ hỗ trợ và giới hạn. Không trình bày số liệu giả lập như thành tích production.

## 2. Danh mục sản phẩm có liên kết

| Chặng | Artifact cần chọn | Câu hỏi reviewer phải trả lời được |
|---|---|---|
| Vấn đề/discovery | Problem brief, evidence log | Vì sao đáng tìm hiểu? |
| Phân tích | Journey, rule/state model | Hiểu hiện trạng và ngoại lệ thế nào? |
| Yêu cầu | Story, AC, NFR, prototype | Giải pháp đề xuất cần làm gì? |
| Quản trị | Traceability, decision/change log | Ai chốt, thay đổi ảnh hưởng đâu? |
| Chấp nhận | UAT, transition, readiness | Bằng chứng nào cho phép sử dụng? |
| Giá trị | Metric/evaluation/business case | Đã biết gì, nên làm gì tiếp? |

Chọn artifact giúp giải thích quyết định. Không cần đưa mọi trang nháp vào phần trình bày chính; giữ phụ lục để truy xuất khi được hỏi.

## 3. Làm một đường truy vết hoàn chỉnh

Ví dụ: giả thuyết giảm hiểu sai điểm đón → US-P04-01 → RULE-P04-02 đúng revision → UAT-P07-03 → dữ liệu adoption P07 → câu hỏi vì sao thấy mà không xác nhận. Ghi loại quan hệ và trạng thái từng nút: đề xuất, thiết kế test case, giả định nghiệm thu hay dữ liệu giả lập.

Đường nối cuối không chứng minh nhân quả. Đường nối này chỉ cho thấy cách nhóm học tiếp sau khi giải pháp được xây trong giả định bài tập.

## 4. Bảo vệ một lựa chọn bị phản biện

Reviewer hỏi “tại sao không xây chat ngay?”. Trả lời bằng nguồn discovery, phương án khác, điều chưa biết và cách kiểm chứng, không bằng sở thích. Reviewer hỏi “7,5% so với 13,16% có nghĩa thành công chưa?”. Giải thích về thiên lệch tự chọn và thiết kế đánh giá còn thiếu.

Khi không có bằng chứng, nói rõ chưa biết và bước tìm hiểu. Khả năng phân định rõ điều đã biết và chưa biết là một phần chất lượng hồ sơ.

## 5. Bài kiểm tra thay đổi bất ngờ

Giả sử stakeholder muốn cho phép sửa điểm sau khi chuyến đã accepted. Trong mười phút, chỉ ra yêu cầu, state/rule, quyền, revision, tài xế đọc, UAT và hỗ trợ nào bị ảnh hưởng. Không cần tự viết thiết kế hoàn chỉnh; cần xác định quyết định nào phải mở lại và người nào cần tham gia.

Sau đó đổi giả định: dữ liệu telemetry về việc tài xế đọc bị thiếu trên một phiên bản app. Sửa evaluation brief và khuyến nghị tương ứng, không giữ kết luận cũ chỉ vì đã đưa vào slide.

## 6. Rubric tự đánh giá

Chấm từng tiêu chí 0–2: nguồn và giới hạn; tính nhất quán mô hình; so sánh phương án; yêu cầu kiểm chứng được; quản lý thay đổi; đánh giá giá trị. Mức 0 là thiếu, 1 là có nhưng còn đứt liên kết, 2 là rõ và giải thích được. Tổng tối đa 12 chỉ là công cụ phản hồi, không phải điểm chứng chỉ.

Nếu coi dữ liệu giả lập là dữ liệu thật, tự ghi nhận phê duyệt hoặc khẳng định nhân quả không có căn cứ, phải sửa trước khi coi hồ sơ hoàn tất dù tổng điểm cao.

## 7. Bài nộp cuối

Nộp README định hướng người đọc, một trang khuyến nghị, danh mục artifact có phiên bản, decision log và reflection. Buổi bảo vệ gồm năm phút vấn đề/bằng chứng, năm phút lựa chọn/yêu cầu, năm phút nghiệm thu/giá trị và năm phút phản biện. Dành riêng thời gian làm bài; đọc xong series không thể thay cho việc thực hành.

<details><summary>Một reflection hữu ích</summary><p>Nêu giả định bạn từng tin, bằng chứng khiến bạn đổi ý, tài liệu đã cập nhật và điều sẽ làm khác ở vòng sau. Tránh chỉ ghi “học được nhiều kỹ năng BA” mà không có ví dụ.</p></details>

## 8. Hoàn thành chương trình, mở vòng học tiếp

Bạn đã đi qua 64 bài trong tám phần, mỗi phần có cơ bản, trung bình và nâng cao. Có thể quay lại [bài mở đầu](/articles/ba-la-gi-tu-tinh-nang-den-bai-toan) và so sánh cách đặt câu hỏi của mình trước và sau khi học. Hồ sơ tốt tiếp tục được sửa khi có bằng chứng mới; hoàn thành tài liệu là một mốc học tập, không chứng minh mọi giả thuyết đã đúng.
