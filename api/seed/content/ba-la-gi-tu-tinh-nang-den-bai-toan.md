---
slug: ba-la-gi-tu-tinh-nang-den-bai-toan
title: "BA là gì? Từ lời đề nghị tính năng đến bài toán nghiệp vụ"
summary: "Vận hành đề nghị thêm chat để khách và tài xế tìm nhau dễ hơn. BA cần làm gì trước khi viết yêu cầu? Cùng phân biệt vấn đề, nhu cầu và giải pháp, đọc số liệu đúng mẫu số và lập problem brief cho Tới Nơi."
category: business-analysis
level: beginner
tags: [business-analysis, ba-001, discovery, problem-statement, toi-noi]
language: vi
---

“Khách gọi lên quá nhiều vì không tìm thấy tài xế. Em viết yêu cầu thêm chat vào app nhé.” Nếu nhận việc này, bạn sẽ mở mẫu đặc tả ngay, hay hỏi thêm về những lần khách và tài xế không gặp được nhau?

Ở Tới Nơi, cả hai việc đều có thể cần làm. Nhưng thứ tự rất quan trọng: một tính năng chat được xây đúng yêu cầu vẫn có thể không giải quyết được chuyện ghim điểm đón nằm nhầm cổng. Công việc BA bắt đầu từ việc làm rõ thay đổi nào đáng thực hiện và vì sao.

<div class="tkh-callout">
<span class="tkh-callout-title">BA-001 · Phần 1/8 · Cơ bản</span>
<p>Bài cần đọc trước: <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a>. Đầu ra: một problem brief. Sau bài này, bạn có thể phân biệt nhu cầu với giải pháp, đặt câu hỏi làm rõ và viết đề xuất discovery có bằng chứng.</p>
</div>

## 1. Business Analysis là gì?

Theo cách diễn giải từ [định nghĩa của IIBA](https://www.iiba.org/knowledgehub/the-business-analysis-standard/2-understanding-business-analysis/2-1-defining-business-analysis/), business analysis là hoạt động hỗ trợ thay đổi trong tổ chức bằng cách xác định nhu cầu và đề xuất giải pháp tạo giá trị cho các bên liên quan trong một bối cảnh cụ thể.

**Business Analysis** là hoạt động phân tích nghiệp vụ; **Business Analyst** là một vai trò thường thực hiện hoạt động đó. Một PO, developer hay người vận hành cũng có thể thực hiện công việc phân tích nghiệp vụ. Có chức danh BA không đồng nghĩa một người là nơi duy nhất được quyền phân tích.

Với tình huống điểm đón, BA cần giúp nhóm trả lời: khó khăn xảy ra ở bước nào, ai chịu tác động, dữ kiện nào đáng tin, có những cách xử lý nào và làm sao biết cách đã chọn tạo ra giá trị. Tài liệu là phương tiện lưu lại hiểu biết và quyết định để mọi người phối hợp.

## 2. Tách lời đề nghị thành bốn lớp

| Lớp | Ví dụ trong Tới Nơi | Cần kiểm chứng gì? |
|---|---|---|
| Hiện tượng | Có khách và tài xế không gặp được nhau | Tần suất, địa điểm, hoàn cảnh |
| Vấn đề | Một số lượt đón thất bại gây mất thời gian và phát sinh hỗ trợ | Mức tác động và nhóm bị ảnh hưởng |
| Nhu cầu | Hai bên xác định được cùng một điểm đón và cách tiếp cận | Ai cần thông tin nào, tại thời điểm nào |
| Giải pháp đề xuất | Chat trong app | Chat có xử lý đúng nguyên nhân với chi phí chấp nhận được? |

“Cần có chat” đã chứa lựa chọn giải pháp. Nó có thể hợp lý, nhưng chưa giải thích vì sao chat tốt hơn hướng dẫn chọn cổng, cải thiện ghi chú hay điều chỉnh quy trình hỗ trợ. BA giữ lại đề nghị này trong danh sách phương án và tìm bằng chứng để so sánh.

Một câu hỏi mở đầu hữu ích là: **“Anh/chị kể lại một trường hợp gần đây, từ lúc khách đặt xe đến khi hai bên không gặp được nhau, được không?”** Câu trả lời thường giúp xác định bước vướng cụ thể hơn câu hỏi “Anh/chị muốn nút chat màu gì?”.

## 3. Đọc bằng chứng trước khi kết luận

Giả sử nhóm được cung cấp bộ số liệu sau. **Toàn bộ số liệu trong bài là giả lập cho việc học BA, không phải dữ liệu vận hành thật hoặc yêu cầu đã duyệt của Tới Nơi.**

| Quan sát tại khu vực A, trong một tuần | Số lượng |
|---|---:|
| Lượt đặt xe đã được tài xế nhận | 1.000 |
| Lượt huỷ trước khi đón | 120 |
| Trong số lượt huỷ, có nhãn “không tìm thấy nhau” | 50 |
| Cuộc gọi CSKH được gắn nhãn liên quan điểm đón | 80 |

Ta tính được tỷ lệ huỷ trước đón là **120 / 1.000 = 12%**. Các lượt mang nhãn “không tìm thấy nhau” chiếm **50 / 1.000 = 5%** số lượt đã được nhận, hoặc khoảng **41,7%** số lượt huỷ trước đón. Hai tỷ lệ trả lời hai câu hỏi khác nhau.

Chưa thể nói “80 khách bị lỗi điểm đón”: một người có thể gọi nhiều lần, và chưa có dữ liệu nối cuộc gọi với từng lượt đặt xe. Cũng chưa thể nói “chat sẽ giảm 50 lượt huỷ”: lý do người dùng chọn có thể thiếu chính xác; một số trường hợp có thể do ghim sai, không có mạng hoặc địa điểm cấm dừng.

BA nên ghi rõ ranh giới giữa điều đã quan sát và điều đang đoán:

| Giả thuyết | Cách tìm bằng chứng tiếp theo |
|---|---|
| Địa điểm có nhiều cổng khiến hai bên đứng khác nhau | Phân nhóm lượt huỷ theo địa điểm; quan sát ca đón tại nơi nhiều cổng |
| Ghi chú của khách khó thấy | Quan sát tài xế dùng màn hình; kiểm tra khả năng đo việc hiển thị/đọc ghi chú |
| Nhãn huỷ đang bị dùng như lựa chọn mặc định | Đối chiếu một mẫu ca có trao đổi với khách và tài xế |

Đây là kế hoạch tìm hiểu, chưa phải kết luận nguyên nhân gốc.

## 4. Tạo problem brief thay vì để hiểu biết nằm trong cuộc họp

Problem brief là bản mô tả ngắn để nhóm thống nhất vấn đề và bước tiếp. Không có một mẫu bắt buộc cho mọi tổ chức; với ca này, một trang là đủ để bắt đầu.

| Mục | Bản nháp cho Tới Nơi |
|---|---|
| Vấn đề | Một nhóm khách và tài xế ở A gặp khó khăn khi xác định cùng điểm đón; một số lượt kết thúc bằng huỷ |
| Bằng chứng | 120/1.000 lượt đã được nhận bị huỷ trước đón trong tuần giả lập; 50 lượt có nhãn liên quan việc tìm nhau |
| Giới hạn | Chưa xác nhận nguyên nhân, tính đại diện của tuần quan sát hoặc độ chính xác của nhãn |
| Tác động cần đo thêm | Thời gian tìm nhau, lượt đón thất bại, tải hỗ trợ, ảnh hưởng thu nhập tài xế |
| Mục tiêu đề xuất | Thảo luận pilot bốn tuần tại A với tỷ lệ huỷ trước đón không quá 9%, sau khi kiểm tra baseline |
| Chỉ số bảo vệ đề xuất | Khiếu nại trên 1.000 lượt được nhận không tăng; cần đo baseline trước pilot |
| Phạm vi discovery | Đón khách đặt xe tại A; chưa mở rộng sang giao hàng hoặc thay chính sách phí huỷ |
| Bước cần quyết định | Có dành một tuần để kiểm tra dữ liệu, quan sát và so sánh phương án không? |

Mốc 9% chỉ là **mục tiêu đề xuất của bài tập**, không phải dự báo chat sẽ làm được. Từ 12% xuống 9% là giảm 3 điểm phần trăm, tương đương giảm 25% tương đối. Trước pilot phải thống nhất cohort, thời gian đo và cách xử lý các lượt chưa có kết quả; không đổi mẫu số giữa baseline và pilot.

## 5. So sánh phương án, kể cả phương án chưa xây gì

| Phương án | Giá trị có thể có | Điều phải trả giá / cần xác minh |
|---|---|---|
| Giữ nguyên, đo và quan sát thêm | Tránh quyết định khi chưa hiểu nguyên nhân | Vấn đề hiện tại tiếp diễn; cần giới hạn thời gian discovery |
| Bổ sung hướng dẫn cổng/điểm đón | Có thể xử lý nhầm địa điểm | Phải duy trì hướng dẫn; không phù hợp mọi nơi |
| Làm ghi chú điểm đón dễ nhận biết hơn | Tận dụng thông tin đã có | Phải biết thông tin có đủ đúng và tài xế có thể xem an toàn |
| Xây chat trong app | Có kênh trao đổi bổ sung | Chi phí phát triển, bảo vệ thông tin, vận hành và xử lý tin nhắn không được đọc |

Ở bước này chưa đủ dữ liệu để chấm điểm chính xác. Gán ROI hoặc số ngày công tuỳ ý chỉ khiến bảng trông chắc chắn hơn thực tế. BA cần lấy ước lượng từ nhóm thực hiện và ghi giả định đi kèm.

Một kết quả tốt của buổi làm việc đầu có thể là **đồng ý discovery**, chưa phải đồng ý xây chat. Quyết định nhỏ này vẫn hữu ích vì nó có phạm vi, người chịu trách nhiệm và bằng chứng cần mang về.

## 6. Những lỗi dễ gặp

- **Viết mục tiêu bằng tên tính năng:** “Ra mắt chat” đo việc giao sản phẩm. Bổ sung kết quả mong muốn với khách/tài xế và cách đo.
- **Đổi lời kể thành sự thật:** “Tài xế không đọc ghi chú” cần nguồn hoặc nhãn giả thuyết.
- **Dùng sai mẫu số:** 50/120 mô tả cơ cấu các lượt huỷ; 50/1.000 mô tả tỷ lệ trên lượt được nhận.
- **BA tự duyệt giải pháp:** BA làm rõ lựa chọn và đề xuất; người có thẩm quyền chốt theo cơ chế của nhóm.

## 7. Bài tập và tự kiểm tra

Viết problem brief tối đa một trang từ bộ số liệu trên. Thêm ba câu hỏi cho vận hành, hai câu hỏi cho tài xế và hai phương án ngoài chat. Gắn nhãn rõ dữ kiện, giả thuyết và đề xuất.

Tự chấm theo năm tiêu chí: nêu đúng nhóm bị ảnh hưởng; dùng đúng mẫu số; không mặc định nguyên nhân; có phương án so sánh; ghi quyết định cần xin cùng người sở hữu. Nếu còn thiếu một mục, brief chưa đủ rõ để dùng cho cuộc họp tiếp theo.

**Câu 1.** Có 50 lượt huỷ mang nhãn “không tìm thấy nhau”. Có thể kết luận chat sẽ cứu được 50 lượt không?

<details><summary>Gợi ý trả lời</summary><p>Không. Nhãn không chứng minh nguyên nhân, và hiệu quả của chat chưa được kiểm tra. Cần xác minh ca thực tế và đánh giá phương án.</p></details>

**Câu 2.** Đầu ra BA có nhất thiết là một đặc tả tính năng không?

<details><summary>Gợi ý trả lời</summary><p>Không. Ở bước này có thể là problem brief, kế hoạch discovery hoặc khuyến nghị thay đổi quy trình. Đầu ra cần hỗ trợ quyết định đang đặt ra.</p></details>

**Câu 3.** Vì sao cần đo khiếu nại khi mục tiêu chính là giảm huỷ?

<details><summary>Gợi ý trả lời</summary><p>Giảm huỷ bằng cách gây khó cho khách khi huỷ có thể làm chỉ số chính đẹp hơn nhưng trải nghiệm tệ hơn. Chỉ số bảo vệ giúp phát hiện đánh đổi này.</p></details>

## 8. Tài liệu tham khảo và bài tiếp theo

- [IIBA — Defining Business Analysis](https://www.iiba.org/knowledgehub/the-business-analysis-standard/2-understanding-business-analysis/2-1-defining-business-analysis/).
- [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi).
- Hồ sơ và mẫu bài tập trong repository: `examples/toi-noi-ba/README.md`; khung chương trình: `docs/ba-master-curriculum.md`.

Bài tiếp theo: [BA, PO, PM, UX, Developer và Tester phối hợp thế nào?](/articles/ba-po-pm-ux-dev-tester-phoi-hop).
