---
slug: ba-po-pm-ux-dev-tester-phoi-hop
title: "BA, PO, PM, UX, Developer và Tester phối hợp thế nào?"
summary: "Một yêu cầu thêm chat đi qua những ai, ai phân tích và ai quyết định? Thiết kế cách phối hợp, bảng trách nhiệm và decision log cho tình huống cải thiện điểm đón của Tới Nơi."
category: business-analysis
level: beginner
tags: [business-analysis, ba-002, stakeholder, collaboration, decision-log, toi-noi]
language: vi
---

Cuộc họp điểm đón vừa kết thúc. Bộ phận vận hành nghĩ nhóm đã đồng ý làm chat. Developer nghĩ mới đang khảo sát. Tester hỏi yêu cầu được duyệt ở đâu. BA có biên bản dài ba trang, nhưng chưa có câu nào nói rõ **đã quyết định điều gì**.

Vấn đề này không giải quyết được chỉ bằng việc ghi chép nhiều hơn. Nhóm cần phân biệt ai đóng góp thông tin, ai thực hiện công việc và ai có quyền chốt từng quyết định.

<div class="tkh-callout">
<span class="tkh-callout-title">BA-002 · Phần 1/8 · Cơ bản</span>
<p>Bài trước: <a href="/articles/ba-la-gi-tu-tinh-nang-den-bai-toan">BA là gì?</a> Đầu ra: bảng trách nhiệm và decision log. Sau bài này, bạn có thể xác định các góc nhìn cần tham gia, tách người phân tích khỏi người quyết định và ghi lại kết luận có thể hành động.</p>
</div>

## 1. Thống nhất ý nghĩa vai trò trước khi phân việc

Trong bài này, **PM là Project Manager — quản lý dự án**. Một số tổ chức dùng PM cho Product Manager; nếu chỉ ghi “PM duyệt” thì hai người có thể hiểu khác nhau. Đây là quy ước của tình huống Tới Nơi, không phải sơ đồ tổ chức chuẩn cho mọi công ty hay mô tả bắt buộc của Scrum.

| Vai trò trong bài tập | Góc nhìn chính | Câu hỏi họ đem tới tình huống điểm đón |
|---|---|---|
| BA — Business Analyst | Nhu cầu, bằng chứng, quy tắc và tác động | Chúng ta đang giải quyết vấn đề nào? |
| PO — Product Owner | Giá trị và thứ tự ưu tiên sản phẩm | Việc này đáng ưu tiên hơn việc nào khác? |
| PM — Project Manager | Kế hoạch, nguồn lực và phụ thuộc dự án | Với phạm vi này, nhóm có thể phối hợp thực hiện ra sao? |
| UX — người nghiên cứu/thiết kế trải nghiệm | Hành vi, khả năng hiểu và sử dụng | Khách và tài xế có hiểu thông tin điểm đón? |
| Developer / Tech Lead | Tính khả thi, thiết kế kỹ thuật và triển khai | Dữ liệu này đến từ đâu, điều gì xảy ra khi mất mạng? |
| Tester | Rủi ro chất lượng và bằng chứng kiểm tra | Trường hợp nào khiến hành vi khác điều mong đợi? |
| Vận hành / CSKH | Quy trình thực tế và xử lý ngoại lệ | Ai hỗ trợ khi hai bên vẫn không gặp được nhau? |
| Sponsor / đại diện nghiệp vụ | Đầu tư, mục tiêu và rủi ro được chấp nhận | Giá trị kỳ vọng có đáng nguồn lực và rủi ro không? |

Một người có thể kiêm nhiều vai. Khi đó vẫn nên nêu rõ họ đang quyết định với thẩm quyền nào. Việc kiêm BA và PO không làm nhu cầu kiểm chứng giả thuyết biến mất.

## 2. BA tạo hiểu biết chung bằng cách nối các góc nhìn

Bộ phận vận hành nói “nhiều cuộc gọi”. UX quan sát thấy khách chọn nhầm cổng. Developer biết địa chỉ hiển thị có thể khác toạ độ ghim. Tester đặt tình huống khách sửa điểm đón sau khi tài xế đã nhận chuyến.

BA liên kết các thông tin này thành câu hỏi nghiệp vụ: **điểm đón nào là điểm hai bên đang cùng xác nhận, ở thời điểm nào và thay đổi có được thông báo không?** Câu hỏi đó cần cả nghiệp vụ, trải nghiệm và kỹ thuật để trả lời.

Nếu BA chỉ nhận lời vận hành rồi chuyển nguyên văn cho developer, sự mơ hồ vẫn còn. Nếu BA tự thiết kế hết để “tiết kiệm thời gian cho nhóm”, các giới hạn kỹ thuật hoặc rủi ro sử dụng có thể bị phát hiện quá muộn. Cách phối hợp phù hợp là cùng làm rõ sớm các quyết định có chi phí sửa cao.

## 3. Lập bảng trách nhiệm theo quyết định cụ thể

“PO phụ trách sản phẩm” quá rộng để xử lý tranh luận trong cuộc họp. Hãy viết xuống các quyết định nhóm sắp phải đưa ra. Bảng sau là **đề xuất phân công cho bài tập**, cần được người tham gia xác nhận.

| Việc / quyết định | Người chuẩn bị hoặc thực hiện | Người chốt trong phạm vi được giao | Người cần tham vấn |
|---|---|---|---|
| Xác nhận cách phân loại cuộc gọi điểm đón | BA và CSKH | Trưởng CSKH | Người phụ trách dữ liệu |
| Lập problem brief | BA | PO xác nhận phạm vi vấn đề ưu tiên | UX, vận hành, dev, tester |
| Dành một tuần discovery | BA và PM đề xuất kế hoạch | PO nếu trong ngân sách đã giao | UX, vận hành, Tech Lead |
| Chọn phương án sản phẩm | BA tổng hợp so sánh | PO; vượt ngân sách thì chuyển sponsor | UX, dev, tester, vận hành |
| Chọn cách triển khai kỹ thuật | Developer / Tech Lead | Tech Lead trong quyền hạn của nhóm | BA, UX, tester |
| Duyệt thay đổi mục tiêu kinh doanh | BA chuẩn bị tác động | Sponsor hoặc đại diện được uỷ quyền | PO, vận hành, PM |

“Chốt” không có nghĩa người đó tự nghĩ ra toàn bộ nội dung. “Chốt” có nghĩa là sau khi đã xem xét ý kiến và bằng chứng, nhóm biết ai chịu trách nhiệm cho quyết định cuối. Nếu tổ chức dùng RACI, có thể chuyển bảng này sang Responsible, Accountable, Consulted, Informed; cần thống nhất cách dùng để tránh biến bảng thành nơi gán mọi người vào mọi ô.

## 4. Đi qua một cuộc trao đổi thay vì bàn giao tuần tự

**Bước 1 — BA cùng bộ phận vận hành xác nhận bằng chứng.** Trong dữ liệu giả lập của bài trước, 80 là số cuộc gọi, chưa phải số chuyến hay số người. Kết luận này được ghi vào brief để mọi người không dùng nhầm số.

**Bước 2 — UX và BA chuẩn bị quan sát.** Nhóm chọn các trường hợp có nhiều cổng và trường hợp thông thường để so sánh. Chưa mặc định khách là bên làm sai. Kết quả quan sát cần có bối cảnh và giới hạn của mẫu.

**Bước 3 — Developer và tester tham gia làm rõ ngoại lệ.** Nếu hướng tới cải thiện thông tin điểm đón, nhóm cần hỏi: dữ liệu thiếu thì sao, hai bên thấy khác nhau thì sao, mạng yếu thì sao? Các câu hỏi này có thể làm thay đổi phạm vi trước khi có bản đặc tả dài.

**Bước 4 — BA tổng hợp lựa chọn; PO chốt bước tiếp.** Có thể chốt thử prototype ghi chú trước, hoặc tiếp tục xác minh dữ liệu. “Chưa đủ bằng chứng để chọn chat” là trạng thái quyết định hợp lệ nếu có người theo dõi và hạn quay lại.

**Bước 5 — PM làm rõ tác động kế hoạch.** Nếu cần thêm người nghiên cứu hoặc lùi công việc khác, quyết định ưu tiên được phản ánh vào kế hoạch. PM không tự biến một mong muốn trong cuộc họp thành cam kết thời hạn.

## 5. Decision log: ghi kết luận và cả điều chưa chốt

| ID | Câu hỏi | Trạng thái | Nội dung và lý do | Người quyết định / bước tiếp |
|---|---|---|---|---|
| D-01 | Dành một tuần discovery? | Đề xuất | Cần xác minh nguyên nhân trước khi chọn phương án | PO xem kế hoạch của BA và PM |
| D-02 | Có xây chat? | Chưa quyết định | Chưa so sánh với hướng dẫn điểm đón và cải thiện ghi chú | BA bổ sung bằng chứng, PO xem lại |
| D-03 | Mục tiêu pilot 9% đã được duyệt? | Đề xuất | Cần kiểm tra baseline và khả năng đo khiếu nại | Sponsor/đại diện nghiệp vụ xem xét |

Bảng trên cố ý chưa ghi “đã duyệt”, vì tình huống chưa có quyết định thực tế. Khi chốt, thêm ngày, người chốt, bằng chứng tham chiếu và điều kiện xem lại. Nếu thay đổi quyết định, giữ lịch sử hoặc tạo dòng mới liên kết tới dòng cũ.

Một biên bản tốt cũng ghi ý kiến chưa đồng thuận. Chẳng hạn bộ phận vận hành muốn ra mắt nhanh, còn Tech Lead cho rằng phạm vi chat có phụ thuộc lớn. BA cần ghi tác động và đề nghị giải quyết, không rút gọn thành “cả nhóm đồng ý triển khai”.

## 6. Những cách phối hợp thường gây tắc

**BA trở thành cổng truyền tin duy nhất.** Developer không được hỏi bộ phận vận hành, tester chỉ đọc bản cuối. Cách sửa: tổ chức buổi làm rõ có người liên quan và cập nhật kết luận vào nguồn dùng chung.

**Mọi quyết định đều chờ một cuộc họp đông người.** Cách sửa: phân loại quyết định và thẩm quyền; tham vấn đúng người, chỉ chuyển cấp khi vượt quyền hạn hoặc có xung đột chưa xử lý được.

**Không có phản hồi bị hiểu là đồng ý.** Cách sửa: ghi rõ “chờ xác nhận”, người cần phản hồi và hạn; khi quá hạn, báo ảnh hưởng hoặc chuyển cấp theo cơ chế đã thống nhất. Im lặng không tự biến đề xuất thành baseline.

**BA duyệt thay nghiệp vụ.** Cách sửa: phân biệt người soạn, người review và người phê duyệt. BA có thể có quyền chốt một số nội dung, nhưng quyền đó phải được giao rõ.

## 7. Bài tập và câu hỏi ôn tập

Giả sử PO yêu cầu phát hành chat trong hai tuần; Tech Lead cho biết cần đánh giá nhà cung cấp; CSKH muốn xem lịch sử trao đổi để xử lý khiếu nại. Hãy lập ba dòng decision log và bảng người cần tham gia. Đừng tự biến hai tuần thành cam kết đã duyệt.

Một bài làm đạt yêu cầu phải tách được quyết định ưu tiên, quyết định giải pháp kỹ thuật và quyết định quyền truy cập lịch sử. Với mỗi quyết định, nêu người chốt, bằng chứng còn thiếu, bước tiếp và hạn xác nhận do nhóm thoả thuận. Quyền xem lịch sử cần review về phạm vi sử dụng, dữ liệu và kiểm soát truy cập trước khi trở thành yêu cầu được duyệt.

**Câu 1.** BA có phải người quyết định ưu tiên trong mọi nhóm không?

<details><summary>Gợi ý trả lời</summary><p>Không. BA cung cấp phân tích; thẩm quyền ưu tiên phụ thuộc phân công. Trong bài tập, PO chốt trong phạm vi được giao, sponsor xem phần vượt thẩm quyền.</p></details>

**Câu 2.** Tester nên chờ yêu cầu hoàn chỉnh rồi mới tham gia?

<details><summary>Gợi ý trả lời</summary><p>Không cần chờ. Tester có thể giúp phát hiện ngoại lệ, điểm mơ hồ và điều khó kiểm chứng ngay khi phân tích phương án.</p></details>

**Câu 3.** Decision log khác danh sách công việc thế nào?

<details><summary>Gợi ý trả lời</summary><p>Decision log lưu câu hỏi quyết định, kết luận hoặc trạng thái chưa chốt, lý do và thẩm quyền. Danh sách công việc theo dõi hành động thực hiện; hai tài liệu có thể liên kết với nhau.</p></details>

## 8. Tham khảo và bài tiếp theo

Các bảng phân công trong bài là mô hình thực hành riêng của Tới Nơi. Khái niệm BA tham khảo [IIBA — Defining Business Analysis](https://www.iiba.org/knowledgehub/the-business-analysis-standard/2-understanding-business-analysis/2-1-defining-business-analysis/); bài không dùng bảng này để quy định trách nhiệm theo Scrum hay một chuẩn quản lý dự án.

- Bài trước: [BA là gì?](/articles/ba-la-gi-tu-tinh-nang-den-bai-toan).
- Hồ sơ thực hành: `examples/toi-noi-ba/README.md` trong repository.
- Bài tiếp theo: [Phân loại yêu cầu nghiệp vụ](/articles/ba-phan-loai-yeu-cau-nghiep-vu).
