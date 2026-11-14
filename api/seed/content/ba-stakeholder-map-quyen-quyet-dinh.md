---
slug: ba-stakeholder-map-quyen-quyet-dinh
title: "Stakeholder map: ai chịu tác động, ai có quyền quyết định?"
summary: "Lập bản đồ stakeholder cho discovery điểm đón của Tới Nơi, phân biệt người chịu tác động với người có thẩm quyền và biến danh sách vai trò thành kế hoạch tham gia có mục đích."
category: business-analysis
level: beginner
tags: [business-analysis, ba-009, stakeholder, discovery, toi-noi]
language: vi
---

Nhóm bắt đầu discovery bằng cuộc họp với PO, bộ phận vận hành và Tech Lead. Sau một giờ, mọi người thống nhất khách cần nhập ghi chú rõ hơn. Nhưng chưa ai hỏi khách khó mô tả cổng ở đâu, hay tài xế nhìn thông tin vào thời điểm nào. Một phòng họp đủ người có chức danh chưa chắc đã đủ góc nhìn.

Stakeholder map giúp BA biết cần tìm hiểu từ ai, ai chịu ảnh hưởng và ai quyết định. Stakeholder map còn giúp phát hiện tiếng nói dễ bị bỏ qua trước khi giải pháp trở thành cam kết.

<div class="tkh-callout">
<span class="tkh-callout-title">BA-009 · Phần 2/8 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/ba-thuc-hanh-de-xuat-cai-thien-diem-don">Thực hành phần 1</a>. Đầu ra: stakeholder map và kế hoạch tham gia. Phần 2 giả định discovery một tuần đã được chấp thuận trong bài tập; chat hoặc pilot chưa được duyệt. Tất cả nhân vật và dữ liệu đều hư cấu.</p>
</div>

## 1. Bắt đầu từ thay đổi, không bắt đầu từ sơ đồ tổ chức

Với điểm đón, hãy lần theo hành trình: ai nhập thông tin, ai nhận, ai xử lý khi không gặp nhau, ai quản lý dữ liệu và ai cấp nguồn lực? Một số người không trực tiếp dùng app vẫn chịu tác động, như nhân viên hỗ trợ hoặc người duy trì hướng dẫn địa điểm.

Đừng xem stakeholder là danh sách người cần mời vào mọi cuộc họp. Với mỗi nhóm, phải biết họ giúp trả lời câu hỏi gì và cần được tham gia ở thời điểm nào.

| Nhóm | Điều họ biết hoặc chịu tác động | Cần tham gia để làm gì? |
|---|---|---|
| Khách mới/khách quen địa điểm | Cách gọi cổng, mức tự tin khi chọn điểm | Hiểu cách xác định và truyền đạt điểm chờ |
| Tài xế mới/quen khu vực | Thời điểm đọc thông tin, khả năng tiếp cận điểm đón | Quan sát hành vi khi xe đã dừng, tìm ngoại lệ |
| CSKH các ca trực | Cách nối cuộc gọi với lượt, xử lý thông tin thiếu | Làm rõ quy trình hỗ trợ và nhu cầu dữ liệu |
| Vận hành địa bàn | Điểm đón thay đổi, thông tin cổng | Xác định người duy trì hướng dẫn nếu chọn phương án đó |
| PO / sponsor | Ưu tiên và nguồn lực | Chốt trong phạm vi thẩm quyền đã giao |
| UX, dev, tester, dữ liệu | Nghiên cứu, khả thi, chất lượng và đo lường | Thiết kế hoạt động, review giới hạn và bằng chứng |

## 2. Quyền lực, mức chịu tác động và kiến thức là ba thứ khác nhau

PO có quyền ưu tiên nhưng không thể thay lời mọi tài xế. Tài xế có trải nghiệm trực tiếp nhưng không tự duyệt ngân sách. Nhóm dữ liệu biết cách truy vấn nhưng cần nghiệp vụ xác nhận ý nghĩa của sự kiện huỷ.

Có thể dùng ma trận ảnh hưởng–mức quan tâm để chuẩn bị trao đổi, nhưng không được dùng ma trận này để loại nhóm ít quyền lực mà chịu tác động lớn. Bản đồ cần thêm mức ảnh hưởng của thay đổi lên mỗi nhóm và kiến thức họ mang tới.

| Stakeholder | Quyền quyết định cần xác nhận | Tác động dự kiến | Kiến thức cần học |
|---|---|---|---|
| Khách ở địa điểm nhiều cổng | Không duyệt ngân sách; có quyền quyết định tham gia nghiên cứu | Cao trong tình huống điểm đón | Ngôn ngữ mô tả địa điểm |
| Trưởng CSKH | Xác nhận quy trình trong phạm vi được giao | Tải hỗ trợ và cách tra cứu có thể đổi | Luồng xử lý thực tế, trường hợp khó |
| PO | Ưu tiên sản phẩm trong uỷ quyền | Chịu trách nhiệm đánh đổi backlog | Bối cảnh ưu tiên và tiêu chí quyết định |

Các mức trên là giả định ban đầu để lập kế hoạch, phải cập nhật khi có thông tin mới. Không biến đánh giá “ít quan tâm” thành đặc điểm cố định của một người.

## 3. Tìm khoảng trống đại diện

Nếu chỉ mời khách đã gọi CSKH, nhóm dễ bỏ sót người tự xử lý hoặc bỏ cuộc không gọi. Nếu chỉ hỏi tài xế quen khu vực, nhóm có thể đánh giá thấp khó khăn với tên cổng. Nếu chỉ gặp trưởng ca, BA có thể nghe quy trình mong muốn thay vì cách nhân viên đang làm.

Chia nhóm theo khác biệt có liên quan tới câu hỏi nghiên cứu: quen/lạ địa điểm, nhiều/một cổng, ca đón thuận lợi/có khó khăn, đã/chưa dùng hỗ trợ. Không cần kết hợp mọi thuộc tính để tạo ra hàng chục ô tuyển người không khả thi. Ưu tiên các khác biệt có thể làm đổi kết luận, và ghi lại nhóm chưa tiếp cận được.

Trong hồ sơ P02, INT-01 là lời kể một khách và INT-02 là lời kể một tài xế **ở ca khác**. Hai nguồn này giúp mở câu hỏi, không đại diện hai phía của cùng một sự kiện.

## 4. Chuyển bản đồ thành kế hoạch tham gia

| Câu hỏi | Nhóm cần tiếp cận | Hình thức đề xuất | Đầu ra |
|---|---|---|---|
| Khách xác định cổng thế nào? | Khách quen và chưa quen địa điểm | Phỏng vấn tình huống gần đây, quan sát | Từ ngữ thực tế, bước gây lúng túng |
| Thông tin đến tài xế lúc nào? | Tài xế có trải nghiệm khác nhau | Quan sát khi đã dừng, hỏi lại sau thao tác | Dòng thời gian và điều kiện sử dụng |
| Có thể nối ticket với lượt không? | CSKH và phụ trách dữ liệu | Walkthrough quy trình, kiểm tra mẫu | Quy tắc liên kết và giới hạn |
| Ai chốt phương án? | PO và người được uỷ quyền liên quan | Review quyết định cuối tuần | Decision log có thẩm quyền |

Ghi thêm người tổ chức, thời gian thoả thuận và phương án khi nhóm quan trọng không tham gia. “Nhờ bộ phận vận hành mời giúp” cần kèm tiêu chí tuyển để tránh chỉ gặp người thuận tiện hoặc thân quen.

## 5. Xác nhận kết quả khác với phê duyệt giải pháp

Khách xác nhận “BA đã ghi đúng điều tôi kể” không đồng nghĩa khách phê duyệt yêu cầu hệ thống. Trưởng CSKH xác nhận luồng xử lý không tự cấp quyền xem mọi dữ liệu. Mỗi lần xin phản hồi cần nêu rõ đang xác nhận sự kiện, cách diễn giải hay quyết định.

[IIBA nhấn mạnh việc xác nhận kết quả sớm và dùng hình thức phù hợp với stakeholder](https://www.iiba.org/professional-development/knowledge-centre/articles/engaging-stakeholders-in-elicitation-and-collaboration/). Với P02, một đoạn tóm tắt ngắn kèm câu hỏi cụ thể hữu ích hơn việc gửi toàn bộ hồ sơ rồi hỏi “anh/chị duyệt nhé”. Không có phản hồi vẫn là chờ xác nhận.

## 6. Những lỗi cần tránh

- Chỉ vẽ người có quyền duyệt, bỏ người dùng và nhóm vận hành hậu trường.
- Coi một người nhiệt tình là đại diện toàn bộ nhóm.
- Mời tất cả vào một cuộc họp, khiến người ít quyền lực khó nói điều trái ý.
- Không cập nhật bản đồ khi phát hiện phụ thuộc hoặc nhóm chịu tác động mới.

Một bản đồ tốt kết thúc bằng kế hoạch tiếp cận và khoảng trống còn lại, không chỉ có tên người trong các ô.

## 7. Bài tập và tự kiểm tra

Lập stakeholder map cho P02, chọn ba khác biệt cần phản ánh trong việc tuyển người và viết kế hoạch cho hai hoạt động đầu tiên. Ghi một nhóm chưa tiếp cận được và kết luận nào phải giới hạn vì thiếu nhóm đó.

**Câu hỏi:** Trưởng CSKH nói tài xế luôn bỏ qua ghi chú. Có thể dùng lời này thay phỏng vấn tài xế không?

<details><summary>Gợi ý trả lời</summary><p>Không. Đây là góc nhìn cần ghi nhận và tìm nguồn sự kiện cụ thể. Cần kiểm tra hành vi, bối cảnh và thông tin mà tài xế thực sự nhận được; từ “luôn” vượt quá bằng chứng hiện có.</p></details>

Tự đánh giá theo bốn điểm: đủ người chịu tác động, rõ quyền quyết định, có mục đích cho từng tương tác và nêu giới hạn đại diện.

## 8. Tham khảo và bài tiếp theo

- [IIBA — Engaging Stakeholders in Elicitation and Collaboration](https://www.iiba.org/professional-development/knowledge-centre/articles/engaging-stakeholders-in-elicitation-and-collaboration/).
- Hồ sơ dùng chung: `examples/toi-noi-ba/p02-discovery-ho-so.md` trong repository.

Bài tiếp theo: [Phỏng vấn nghiệp vụ: hỏi để hiểu thay vì dẫn dắt](/articles/ba-phong-van-nghiep-vu-khong-dan-dat).
