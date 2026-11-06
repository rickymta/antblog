---
slug: ba-scope-assumption-constraint-dependency
title: "Scope, assumption, constraint và dependency"
summary: "Khoanh phạm vi discovery và pilot điểm đón, phân biệt giả định với ràng buộc và phụ thuộc, lập sổ theo dõi có owner, bằng chứng xác minh và cách xử lý khi điều kiện thay đổi."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-006, scope, assumptions, constraints, dependencies, toi-noi]
language: vi
---

Buổi đầu chỉ bàn việc khách và tài xế khó tìm nhau. Vài ngày sau, danh sách mong muốn đã có chat, gửi ảnh, lịch sử trao đổi, dịch tự động và giao hàng nhiều điểm. Mỗi ý đều có lý do nghe hợp lý, nhưng nhóm chưa thống nhất điều đang cam kết tìm hiểu hoặc triển khai.

Quản lý phạm vi bắt đầu bằng cách làm rõ ranh giới và điều kiện của quyết định. Nó không phải chỉ nói “ngoài scope” khi có yêu cầu mới.

<div class="tkh-callout">
<span class="tkh-callout-title">BA-006 · Phần 1/8 · Trung bình</span>
<p>Đọc trước: <a href="/articles/ba-problem-statement-muc-tieu-chi-so">Mục tiêu và chỉ số thành công</a>. Đầu ra: scope in/out cùng sổ giả định, ràng buộc và phụ thuộc. Mục tiêu: chỉ rõ điều chưa biết, ai xác minh và thay đổi nào cần xem lại quyết định.</p>
</div>

## 1. Bốn khái niệm giải quyết bốn câu hỏi

| Khái niệm | Câu hỏi thực hành | Ví dụ đề xuất của Tới Nơi |
|---|---|---|
| Scope — phạm vi | Việc đang xem xét bao gồm và chưa bao gồm gì? | Discovery điểm đón đặt xe tại A; chưa nghiên cứu giao hàng nhiều điểm |
| Assumption — giả định | Điều gì đang được tạm tin nhưng chưa xác nhận? | Có thể nối cuộc gọi CSKH với lượt đặt xe bằng ID |
| Constraint — ràng buộc | Giới hạn nào phải tuân theo trong quyết định hiện tại? | Nếu được duyệt, discovery chỉ dùng nguồn lực đã cấp trong một tuần |
| Dependency — phụ thuộc | Đầu ra hoặc điều kiện nào từ nơi khác cần có? | Nhóm dữ liệu cung cấp bản trích đã kiểm tra trước buổi tổng hợp |

Thuật ngữ có thể được tổ chức sử dụng hơi khác. Cần thống nhất ý nghĩa trong nhóm và ghi nguồn của từng mục. Một câu “phải xong trong hai tuần” mới được nói trong họp có thể là mong muốn, chưa phải ràng buộc đã được người có thẩm quyền xác nhận.

## 2. Tách phạm vi tìm hiểu khỏi phạm vi giải pháp

Discovery có thể xem xét chat mà chưa cam kết xây chat. Pilot có thể chỉ thử hướng dẫn điểm đón dù discovery đã so sánh nhiều lựa chọn. Nếu dùng chung một dòng scope cho cả hai giai đoạn, nhóm dễ hiểu sai mức cam kết.

| Chiều phạm vi | Discovery P01 đề xuất | Chưa nằm trong cam kết |
|---|---|---|
| Nghiệp vụ | Đón khách của dịch vụ đặt xe | Giao hàng nhiều điểm, tính phí mới |
| Địa bàn | Khu vực A trong dữ liệu giả lập | Suy rộng kết quả sang mọi khu vực |
| Người tham gia | Khách, tài xế, CSKH và nhóm liên quan tại A | Khẳng định đã đại diện mọi nhóm người dùng |
| Công việc | Kiểm tra dữ liệu, quan sát, so sánh phương án | Xây và phát hành chat đầy đủ |
| Đầu ra | Brief cập nhật, bằng chứng, lựa chọn, điều kiện pilot | Cam kết đạt 9% hoặc cam kết ngày phát hành |

Phạm vi pilot chỉ được chốt sau khi chọn phương án và có thông tin cần thiết. Khi đó cần thêm nhóm người dùng, phiên bản áp dụng, cách hỗ trợ, chỉ số đo và điều kiện dừng.

## 3. Giả định cần có hạn sử dụng

“Giả định: dữ liệu tốt” không giúp người khác biết phải làm gì. Hãy viết giả định theo cách có thể bị bác bỏ và ghi hậu quả nếu sai.

| ID | Giả định chưa xác nhận | Cách xác minh đề xuất | Owner đề xuất / hạn | Nếu sai |
|---|---|---|---|---|
| A-01 | Cuộc gọi CSKH có thể nối với lượt đặt xe | Kiểm tra schema và mẫu record, đo tỷ lệ nối được | Người phụ trách dữ liệu / trước phân tích cuộc gọi | Không suy ra tỷ lệ theo chuyến từ cuộc gọi; thiết kế thu thập khác |
| A-02 | Nhãn huỷ phản ánh khó khăn tìm điểm đón đủ để chọn mẫu | Đối chiếu một mẫu ca có nguồn bổ sung | BA và CSKH / trước kết luận nguyên nhân | Sửa cách chọn mẫu, ghi giới hạn của nhãn |
| A-03 | Có thể tuyển khách và tài xế cho quan sát trong tuần | Xác nhận khả năng tham gia và lịch | UX và vận hành / trước khởi động | Điều chỉnh kế hoạch và mức độ chắc chắn |

Các hạn ở đây gắn với mốc công việc; khi bắt đầu thật phải đổi thành ngày cụ thể và xác nhận người nhận việc. Trạng thái nên phân biệt **chưa kiểm tra**, **được hỗ trợ trong phạm vi đã kiểm**, **bị bác bỏ**, **không còn áp dụng**. Kiểm tra một mẫu nhỏ không làm giả định đúng vĩnh viễn.

## 4. Ràng buộc phải có nguồn và phạm vi hiệu lực

Giả sử sponsor chấp thuận một tuần discovery với nhóm hiện có. Khi được xác nhận trong bài tập, đây có thể là ràng buộc kế hoạch cho discovery. Nó không tự biến thành “phải xây xong sản phẩm trong một tuần”.

Một bản ghi ràng buộc cần: nội dung, lý do hoặc nguồn quyết định, người có quyền thay đổi, phạm vi áp dụng và ngày xem lại. Nếu chưa được chốt, ghi **đề xuất**, không dùng nó để từ chối mọi câu hỏi mới.

Ràng buộc về bảo vệ dữ liệu cũng cần cụ thể. “Không đưa thông tin nhận dạng vào tài liệu chia sẻ rộng” dẫn tới việc chuẩn bị dữ liệu tổng hợp/giả lập, quyền truy cập và cách kiểm tra đầu ra. Trong series này mọi dữ liệu đều hư cấu; không lấy ví dụ này làm kết luận về nghĩa vụ pháp lý của hệ thống thật.

## 5. Phụ thuộc cần đầu ra nghiệm thu được

“Chờ đội data” không đủ để quản lý phụ thuộc. Cần biết đang chờ cái gì, thế nào là dùng được và trễ thì ảnh hưởng ra sao.

| ID | Đầu ra cần có | Điều kiện dùng được | Người phối hợp | Tác động nếu chậm |
|---|---|---|---|---|
| DEP-01 | Bản trích dữ liệu cohort tại A | Định nghĩa ID, thời gian, sự kiện; báo cáo thiếu/trùng | Nhóm dữ liệu | Chưa xác nhận baseline và mục tiêu pilot |
| DEP-02 | Lịch quan sát đã được xác nhận | Có nhóm người phù hợp và cách ghi nhận đã thống nhất | UX, vận hành | Chưa kiểm tra hành vi thực tế |
| DEP-03 | Đánh giá sơ bộ phương án | Có giả định kỹ thuật, khoảng ước lượng và điểm chưa rõ | Tech Lead | Chưa so sánh chi phí đủ tin cậy |

Phụ thuộc không chỉ là task ở đội khác. Quyết định cấp quyền dữ liệu hoặc sự tham gia của một stakeholder cũng có thể là điều kiện bắt buộc trước một bước tiếp theo.

## 6. Thay đổi phạm vi: phân tích trước khi đồng ý

Vận hành đề nghị thêm gửi ảnh điểm đón. BA nên đi qua bốn câu hỏi:

1. Ảnh phục vụ nhu cầu nào đã quan sát? Trường hợp dùng cụ thể là gì?
2. Có nằm trong phạm vi discovery không? Nếu có, việc đánh giá ý tưởng không đồng nghĩa cam kết triển khai.
3. Nếu đưa vào pilot, cần thêm thay đổi nào về dữ liệu, quyền truy cập, giao diện, hỗ trợ và kiểm thử?
4. Ai quyết định đánh đổi thời gian, nguồn lực hoặc bỏ bớt phạm vi khác?

Một kết luận hợp lý có thể là “ghi nhận để so sánh trong discovery; chưa đưa vào pilot vì thiếu đánh giá ảnh hưởng”. Ghi decision log và điều kiện xem lại, thay vì để ý tưởng biến mất hoặc âm thầm tăng phạm vi.

## 7. Bài tập và câu hỏi ôn tập

Phân loại các câu sau, bổ sung trường còn thiếu và cho biết câu nào chưa thể coi là cam kết:

- “Chắc đội data có lịch sử đủ bốn tuần.”
- “PO muốn demo thứ Sáu.”
- “Nhóm cần bản trích đã loại record trùng trước khi tính baseline.”
- “Phần này chỉ nghiên cứu điểm đón đặt xe tại A.”

<details><summary>Gợi ý lời giải</summary><p>Câu đầu là giả định, cần kiểm nguồn và độ phủ. Câu thứ hai là mong muốn lịch, cần xác nhận nội dung demo và quyền chốt trước khi coi là ràng buộc. Câu thứ ba là phụ thuộc với điều kiện chất lượng, cần owner và hạn. Câu cuối là đề xuất phạm vi, cần xác nhận và lưu version.</p></details>

Bài làm tốt còn nêu hậu quả khi điều kiện không đúng. Chỉ đổi nhãn mà không xác định bước xử lý sẽ không giúp kế hoạch tiến lên.

**Câu hỏi ôn tập:** Giả định bị bác bỏ có luôn làm dự án thất bại không?

<details><summary>Gợi ý trả lời</summary><p>Không. Nó cung cấp thông tin để sửa phương án hoặc phạm vi. Vấn đề lớn hơn là tiếp tục ra quyết định như thể giả định vẫn đúng.</p></details>

## 8. Tham khảo và bài tiếp theo

- [IIBA — Glossary](https://production.iiba.org/career-resources/a-business-analysis-professionals-foundation-for-success/babok/glossary/): thuật ngữ assumption, constraint và scope. Bảng quản lý trong bài là mẫu thực hành của Tới Nơi.
- [Phối hợp và decision log](/articles/ba-po-pm-ux-dev-tester-phoi-hop).

Bài tiếp theo: [Khi stakeholder đòi tính năng nhưng bằng chứng chưa đủ](/articles/ba-quyet-dinh-khi-thieu-bang-chung).
