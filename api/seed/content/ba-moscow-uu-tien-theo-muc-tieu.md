---
slug: ba-moscow-uu-tien-theo-muc-tieu
title: "MoSCoW: Must có thật sự bắt buộc trong đợt này?"
summary: "Dùng MoSCoW với mục tiêu và timebox cụ thể, phân biệt điều kiện không thể thiếu với mong muốn có giá trị nhưng có thể lùi."
category: business-analysis
level: beginner
tags: [business-analysis, ba-042, moscow, prioritization, toi-noi]
language: vi
---

Nếu tất cả đều Must, nhóm chưa có phương án khi thời gian không đủ. Câu hỏi quan trọng là thiếu điều này thì đợt triển khai có còn đạt mục tiêu và được phép vận hành không?

<div class="tkh-callout"><span class="tkh-callout-title">BA-042 · Phần 6/8 · Cơ bản</span><p>Đầu ra: bảng ưu tiên cho pilot điểm đón đề xuất. Mức ưu tiên chỉ có nghĩa trong mục tiêu và thời hạn đã ghi, không phải nhãn vĩnh viễn.</p></div>

## 1. Chốt khung trước khi xếp hạng

Giả định bài tập: pilot giới hạn một khu vực, khách xác nhận điểm đón trước accepted và tài xế được phân công có thể xem thông tin đó. Chưa chốt ngày chạy thật. Mục tiêu là kiểm tra giả thuyết giảm hiểu sai vị trí; không nhằm ra mắt mọi kênh liên lạc.

Nếu pilot chưa xác định đối tượng và điều kiện đo, cuộc tranh luận Must/Should thường chỉ phản ánh sở thích của từng người.

## 2. Bốn nhóm có ý nghĩa gì?

Must là phần không thể thiếu trong đợt đã xác định. Should quan trọng nhưng có thể dùng phương án thay thế chấp nhận được. Could mang giá trị nhưng tác động khi bỏ thấp hơn. Won't have this time ghi rõ là nằm ngoài đợt này, và không đồng nghĩa với cam kết chắc chắn làm ở đợt sau.

Các nhóm giúp thương lượng phạm vi, không thay phân tích giá trị hoặc quyền phê duyệt.

## 3. Bảng đề xuất cho pilot

| Hạng mục | Nhóm đề xuất | Lý do |
|---|---|---|
| Chỉ chủ chuyến xác nhận | Must | Sai quyền thì pilot không thể chấp nhận |
| Kiểm state và revision | Must | Tránh xác nhận thông tin cũ hoặc sai thời điểm |
| Tài xế được phân công đọc điểm đón | Must | Thiếu thì không kiểm được hành trình đã chọn |
| Mẫu hướng dẫn theo từng cổng | Should | Có thể dùng ghi chú có hướng dẫn tạm thời |
| Minh hoạ chuyển động | Could | Không quyết định tính đúng của xác nhận |
| Chat đầy đủ | Won't lần này | Chưa có bằng chứng và nằm ngoài lát cắt của đợt này |

Đây là đề xuất cần các bên đồng thuận, không tự động trở thành quyết định chỉ vì BA đặt trong bảng.

## 4. Kiểm tra phương án thay thế

“CSKH sẽ xử lý thủ công” chỉ là workaround khi CSKH thực sự có người, công cụ, thời gian và quyền xử lý. Nếu mỗi ngày có hàng trăm ca mà một người không đáp ứng được, không thể dùng câu đó để hạ một Must thành Should.

Ghi công suất giả định, chi phí và điều kiện dừng workaround. Phương án tạm thời cũng tạo yêu cầu vận hành.

## 5. Khi Must vượt khả năng

Không lặng lẽ đổi nhãn để bảng trông cân đối. Có thể thu hẹp cohort, đổi ngày, chọn giải pháp khác hoặc không chạy pilot. Quyết định cần người có quyền chấp nhận tác động. Ước lượng của nhóm thực hiện là đầu vào, không phải số BA tự đặt cho vừa timebox.

## 6. Ghi lại và xem xét lại

Mỗi hàng có lý do, người quyết định và đợt áp dụng. Khi mục tiêu thay đổi từ pilot sang toàn quốc, xếp hạng cũ phải được xem lại. Một mục Won't không được trở lại phạm vi qua một câu nói trong buổi demo mà không có đánh giá tác động.

## 7. Thực hành

Stakeholder nói mẫu cổng phải là Must vì “đẹp và chuyên nghiệp”. Viết câu hỏi để kiểm tra lý do ấy, và nêu bằng chứng nào sẽ khiến bạn đổi ưu tiên.

<details><summary>Gợi ý</summary><p>Hỏi xem nếu không có mẫu thì người dùng có xác nhận đúng được không, nhóm nào bị ảnh hưởng và workaround có dùng được không. Nếu bằng chứng cho thấy họ không thể hoàn thành hành trình, cần xem lại phạm vi hoặc mức ưu tiên.</p></details>

## 8. Nguồn và bài tiếp

[Agile Business Consortium — MoSCoW](https://www.agilebusiness.org/resource/what-is-moscow-prioritization/) giải thích kỹ thuật và các mức. Tiếp: [traceability](/articles/ba-traceability-muc-tieu-yeu-cau-test).
