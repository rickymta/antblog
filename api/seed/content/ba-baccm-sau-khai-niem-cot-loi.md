---
slug: ba-baccm-sau-khai-niem-cot-loi
title: "BACCM: nhìn bài toán qua sáu khái niệm cốt lõi"
summary: "Dùng sáu khái niệm Change, Need, Solution, Value, Stakeholder và Context để kiểm tra đề xuất cải thiện điểm đón của Tới Nơi, tìm giả định còn thiếu và theo dõi tác động khi bối cảnh thay đổi."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-004, baccm, discovery, toi-noi]
language: vi
---

Nhóm Tới Nơi đã có problem brief, danh sách người tham gia và vài yêu cầu đề xuất. Một người hỏi: “Nếu cải thiện ghi chú rẻ hơn chat thì chọn ghi chú luôn được không?” Câu trả lời còn phụ thuộc vào việc tài xế có thể đọc ghi chú đúng lúc, khách có biết ghi gì và điểm đón có đủ rõ để mô tả hay không.

Để xem xét các mối liên hệ đó, BA cần một khung câu hỏi gọn. BACCM giúp nhóm nhìn cùng một thay đổi qua nhiều góc, thay vì chỉ tranh luận tính năng nào có nhiều ưu điểm hơn.

<div class="tkh-callout">
<span class="tkh-callout-title">BA-004 · Phần 1/8 · Trung bình</span>
<p>Đọc trước: <a href="/articles/ba-phan-loai-yeu-cau-nghiep-vu">Phân loại yêu cầu</a>. Đầu ra: bảng phân tích sáu khái niệm và một impact note. Mục tiêu: đặt câu hỏi về quan hệ giữa nhu cầu, giải pháp và giá trị; nhận biết điều phải xem lại khi bối cảnh thay đổi.</p>
</div>

## 1. BACCM là mô hình khái niệm, không phải sáu bước dự án

Theo [IIBA](https://www.iiba.org/knowledgehub/the-business-analysis-standard/2-understanding-business-analysis/2-2-a-model-for-effective-analysis-baccm/), Business Analysis Core Concept Model — BACCM — gồm sáu khái niệm liên hệ với nhau: **Change, Need, Solution, Value, Stakeholder và Context**. Chúng cung cấp ngôn ngữ chung để phân tích nghiệp vụ.

Không có chuyện phải hoàn thành ô Need rồi mới được nghĩ tới Stakeholder. Một quan sát mới từ tài xế có thể khiến nhóm sửa cách hiểu nhu cầu; một giới hạn bối cảnh có thể làm phương án đang được ưa chuộng không còn phù hợp. Các bảng dưới đây là cách áp dụng riêng cho bài tập Tới Nơi, không phải biểu mẫu bắt buộc của IIBA.

## 2. Điền sáu ô bằng một tình huống cụ thể

Toàn bộ dữ kiện và đề xuất trong bài thuộc hồ sơ giả lập của series. Chưa có quyết định xây chat hoặc thay đổi SPEC của hệ thống.

| Khái niệm | Cách hiểu để đặt câu hỏi | Áp dụng vào tình huống điểm đón |
|---|---|---|
| Need — nhu cầu | Vấn đề hoặc cơ hội nào cần được xử lý? | Hai bên cần xác định cùng điểm đón; chưa rõ nguyên nhân nào làm việc này thất bại |
| Change — thay đổi | Điều gì trong cách vận hành sẽ khác đi? | Có thể thay cách khách mô tả điểm chờ và cách tài xế nhận thông tin |
| Solution — giải pháp | Cách nào đáp ứng nhu cầu trong tình huống này? | Hướng dẫn chọn cổng, làm rõ ghi chú, chat hoặc kết hợp quy trình hỗ trợ |
| Value — giá trị | Ai nhận được lợi ích gì, phải chịu chi phí gì? | Khách bớt chờ; tài xế bớt vòng lại; nhóm phải trả chi phí phát triển và duy trì |
| Stakeholder — bên liên quan | Ai có quan hệ với nhu cầu, thay đổi hoặc giải pháp? | Khách, tài xế, CSKH, vận hành, PO, nhóm kỹ thuật và người chịu trách nhiệm dữ liệu |
| Context — bối cảnh | Điều kiện nào làm thay đổi có ý nghĩa hoặc bị giới hạn? | Địa điểm nhiều cổng, chất lượng mạng, thời điểm sử dụng, phiên bản app, khu vực pilot |

Viết “Value = tăng doanh thu” chưa đủ cho tình huống này. Nếu không biết lượt đón được cải thiện thế nào hoặc chi phí phát sinh là gì, giá trị đó mới là mong muốn. Viết “Stakeholder = người dùng” cũng che mất sự khác nhau giữa khách, tài xế và người hỗ trợ.

## 3. Giá trị của bảng nằm ở các mối nối

### Need ↔ Solution: giải pháp xử lý đúng nhu cầu nào?

Nếu vấn đề chủ yếu là địa điểm nhiều cổng, chat có thể giúp trao đổi thêm nhưng vẫn yêu cầu khách mô tả đúng cổng. Một danh sách điểm đón có hướng dẫn có thể phù hợp hơn ở địa điểm được quản lý tốt. Ngược lại, danh sách cứng dễ lỗi thời ở nơi đang sửa đường. Cả hai nhận định đều cần kiểm chứng trong bối cảnh cụ thể.

BA nên viết quan hệ dưới dạng giả thuyết: “Nếu khách chọn được cổng nhận biết rõ và tài xế nhận cùng thông tin trước khi tới, số trường hợp đứng khác cổng có thể giảm.” Câu này cho thấy điều kiện cần quan sát, thay vì khẳng định “thêm tính năng sẽ giảm huỷ”.

### Stakeholder ↔ Value: lợi ích có đang chuyển thành gánh nặng cho bên khác?

Yêu cầu khách nhập một đoạn mô tả dài có thể cung cấp thêm dữ liệu cho tài xế, nhưng làm quá trình đặt xe khó hơn. Hướng dẫn điểm đón cố định có thể giúp khách mới, nhưng tạo việc duy trì thông tin cho vận hành. BA phải tìm người chịu phần việc đó và xác nhận khả năng thực hiện.

### Context ↔ Solution: có điều kiện nào khiến phương án không dùng được?

“Tài xế có thể đọc thông tin” khác với “tài xế có thể đọc đúng lúc và an toàn”. Cần quan sát thời điểm sử dụng, tránh thiết kế luồng buộc tương tác khi đang điều khiển xe. Đây là câu hỏi thiết kế và vận hành cần review, không phải giả định đã được giải quyết vì màn hình có chữ.

### Change ↔ Value: thay đổi hành vi nào phải xảy ra trước khi có lợi ích?

Ra mắt giao diện chưa đồng nghĩa khách sẽ nhập thông tin hữu ích. Chuỗi kỳ vọng có thể là: khách hiểu cần mô tả gì → dữ liệu có ích được tạo → tài xế nhận biết dữ liệu → hai bên cùng xác định địa điểm → bớt lượt đón thất bại. Mỗi mối nối có thể đứt. Chỉ đo số lượt mở màn hình không cho biết toàn bộ chuỗi hoạt động.

## 4. Khi bối cảnh đổi, cập nhật cả phân tích

Giả sử sau discovery tại A, PO muốn mở pilot sang khu vực B có nhiều điểm đón tạm thời. **Đây là biến thể giả định để luyện phân tích**, chưa bổ sung dữ kiện vào baseline P01.

| Nội dung phải xem lại | Câu hỏi cần trả lời | Đầu ra cập nhật |
|---|---|---|
| Context | Điểm đón thay đổi theo ngày hay theo giờ? | Danh sách điều kiện mới |
| Stakeholder | Ai cập nhật và xác nhận điểm đón tạm? | Người phụ trách và quy trình xác nhận |
| Solution | Hướng dẫn cố định còn phù hợp? | So sánh phương án theo bối cảnh B |
| Change | Vận hành cần làm thêm việc gì? | Quy trình mới và nhu cầu hướng dẫn |
| Value | Lợi ích có bù chi phí duy trì không? | Giả thuyết giá trị và cách đo |
| Need | Vẫn là khó nhận diện cổng hay vấn đề đã khác? | Problem statement được kiểm tra lại |

Không nên lấy tỷ lệ 12% của A làm baseline B. Thay phạm vi không chỉ là sửa tên khu vực trong tài liệu; việc đó có thể đổi cả nhu cầu và cách đo.

## 5. Từ mô hình sang việc cần làm

Một bảng sáu ô đầy chữ nhưng không dẫn tới hành động sẽ nhanh chóng bị bỏ quên. Hãy gắn mỗi khoảng trống quan trọng với người xác minh và đầu ra.

| Khoảng trống | Người phối hợp đề xuất | Bằng chứng cần mang về |
|---|---|---|
| Khách có biết mô tả cổng bằng từ ngữ nào? | BA và UX | Ghi chép quan sát, cách gọi địa điểm thực tế |
| Ai duy trì hướng dẫn điểm đón? | Vận hành | Người phụ trách, tần suất kiểm tra và cách báo thông tin sai |
| Tài xế có nhận được ghi chú đúng lúc? | UX, developer, tài xế tham gia nghiên cứu | Quan sát thao tác và kiểm tra luồng dữ liệu |
| Chi phí vận hành có thể vượt lợi ích? | PO và vận hành | Ước lượng có giả định; chưa điền ROI khi thiếu dữ liệu |

Những người phụ trách ở đây là đề xuất phân công. BA cần xác nhận với người tham gia trước khi ghi thành cam kết.

## 6. Lỗi thường gặp

- **Sáu ô thành sáu silo:** chỉ điền định nghĩa. Hãy bổ sung ít nhất một quan hệ có thể kiểm chứng giữa hai ô.
- **Need được viết bằng tên giải pháp:** “cần chat”. Hỏi người đó muốn thực hiện việc gì tốt hơn nhờ chat.
- **Value chỉ có lợi ích của tổ chức:** bổ sung chi phí và ảnh hưởng với khách, tài xế, vận hành.
- **Context là đoạn giới thiệu không cập nhật:** khi đổi khu vực hoặc nhóm người dùng, xem lại giả thuyết và phương án.

## 7. Bài tập và câu hỏi ôn tập

Chọn phương án hướng dẫn cổng đón. Điền sáu ô BACCM, viết ba quan hệ dưới dạng “nếu… thì… với điều kiện…”, rồi phân tích biến thể khu vực B ở trên. Nộp một impact note tối đa nửa trang, nêu điều giữ nguyên, điều phải xác minh và quyết định cần hoãn.

Tự đánh giá: có ít nhất ba nhóm stakeholder; nêu lợi ích và chi phí; không dùng baseline A cho B; mỗi giả định quan trọng có cách kiểm tra; không coi phương án đã được duyệt.

**Câu 1.** Điền đủ sáu ô có chứng minh giải pháp đúng không?

<details><summary>Gợi ý trả lời</summary><p>Không. Mô hình giúp phát hiện câu hỏi và mối liên hệ cần phân tích; tính đúng của giả thuyết vẫn cần bằng chứng.</p></details>

**Câu 2.** Khách được lợi nhưng CSKH phải làm thêm việc thì ghi vào đâu?

<details><summary>Gợi ý trả lời</summary><p>Ghi ảnh hưởng ở Value theo từng stakeholder, đồng thời xem lại Change và khả năng vận hành trong Context. Một thông tin có thể liên quan nhiều khái niệm.</p></details>

## 8. Tham khảo và bài tiếp theo

- [IIBA — A Model for Effective Analysis: BACCM](https://www.iiba.org/knowledgehub/the-business-analysis-standard/2-understanding-business-analysis/2-2-a-model-for-effective-analysis-baccm/).
- Hồ sơ tình huống trong repository: `examples/toi-noi-ba/README.md`.

Bài tiếp theo: [Problem statement, mục tiêu và chỉ số thành công](/articles/ba-problem-statement-muc-tieu-chi-so).
