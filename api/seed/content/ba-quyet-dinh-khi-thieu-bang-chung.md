---
slug: ba-quyet-dinh-khi-thieu-bang-chung
title: "Khi stakeholder đòi tính năng nhưng bằng chứng chưa đủ"
summary: "PO muốn chốt chat ngay, trong khi dữ liệu chưa giải thích được nguyên nhân huỷ. BA so sánh giả thuyết, chi phí trì hoãn và lựa chọn thử nghiệm có thể đảo ngược để đưa ra khuyến nghị có điều kiện."
category: business-analysis
level: advanced
tags: [business-analysis, ba-007, decision-analysis, uncertainty, discovery, toi-noi]
language: vi
---

“Đừng phân tích thêm nữa, cứ làm chat đi. Chậm một tuần là thêm một tuần khách gặp khó.” Lập luận này có phần đúng: tìm hiểu cũng tiêu tốn thời gian, trong khi vấn đề tiếp tục xảy ra. Nhưng chọn sai giải pháp có thể tiêu tốn nhiều tuần hơn và tạo thêm nghĩa vụ vận hành.

BA cần đưa ra một lựa chọn có thể hành động trong điều kiện chưa biết hết. Câu trả lời không nên chỉ là “cần thêm dữ liệu”, cũng không nên biến áp lực thời gian thành bằng chứng rằng chat là phương án tốt nhất.

<div class="tkh-callout">
<span class="tkh-callout-title">BA-007 · Phần 1/8 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/ba-scope-assumption-constraint-dependency">Scope, assumption, constraint và dependency</a>. Đầu ra: recommendation memo có phương án, bằng chứng, điều kiện và người quyết định. Mục tiêu: chọn bước tiếp theo tương xứng với mức bất định và chi phí thay đổi.</p>
</div>

## 1. Tách điều stakeholder muốn khỏi lý do họ muốn sớm

Trong tình huống giả lập P01, bộ phận vận hành muốn giảm khó khăn ở điểm đón, còn “làm chat” là phương án đề xuất. Khi có áp lực thời gian, BA cần hỏi điều gì xảy ra nếu chưa triển khai ngay: tải hỗ trợ tăng, có thời điểm cao điểm sắp tới, hay một cam kết nội bộ đang đến hạn?

Lý do trì hoãn gây thiệt hại cần được ghi riêng, tách biệt với bằng chứng về hiệu quả giải pháp. Có deadline thật không chứng minh chat xử lý đúng nguyên nhân. Ngược lại, thiếu chắc chắn về nguyên nhân không có nghĩa là nhóm được phép kéo dài discovery vô hạn.

Một lời đáp có ích là: “Mình có thể đưa ra quyết định về bước tiếp theo trong một tuần discovery có giới hạn. Trong thời gian đó, bộ phận vận hành xem xét biện pháp hỗ trợ tạm thời. Cuối tuần, nhóm sẽ có lựa chọn tiếp tục, thử nhỏ hoặc dừng với tiêu chí rõ.” Đây vẫn là **đề xuất**, cần người có thẩm quyền chấp thuận nguồn lực.

## 2. Lập bảng giả thuyết cạnh tranh

Bộ số liệu 1.000 lượt được nhận, 120 lượt huỷ, 50 nhãn “không tìm thấy nhau” và 80 cuộc gọi chưa phân biệt được các nguyên nhân dưới đây.

| Giả thuyết | Nếu đúng, có thể quan sát gì? | Bằng chứng phân biệt | Phương án có thể phù hợp |
|---|---|---|---|
| H-01: hai bên chọn khác cổng | Lượt thất bại tập trung ở địa điểm nhiều cổng; lời mô tả khác nhau | Xem lại một số lượt có bối cảnh, quan sát điểm đón | Hướng dẫn cổng, điểm đón nhận diện rõ |
| H-02: thông tin có nhưng không được thấy đúng lúc | Khách đã nhập ghi chú, tài xế không nhận biết | Quan sát màn hình và luồng thông tin | Cải thiện cách hiển thị/thông báo |
| H-03: cần trao đổi tình huống phát sinh | Ghi chú ban đầu đúng nhưng có thay đổi khó mô tả trước | Phỏng vấn hai bên về diễn biến cụ thể | Kênh trao đổi phù hợp, có thể là chat |
| H-04: nhãn huỷ không phản ánh đúng nguyên nhân | Đối chiếu từng lượt cho thấy lý do khác với nhãn | Review chất lượng gắn nhãn | Sửa đo lường và cách phân loại trước |

Các giả thuyết có thể cùng đúng ở các nhóm lượt khác nhau. Không cần tìm một nguyên nhân duy nhất cho mọi lượt huỷ. Khi chọn mẫu quan sát, đừng chỉ tuyển người đã từng yêu cầu chat; kết quả sẽ nghiêng về phương án đó ngay từ đầu.

## 3. Chọn việc tìm hiểu có khả năng đổi quyết định

Trước mỗi hoạt động discovery, hỏi: **kết quả nào sẽ khiến nhóm chọn khác đi?**

Nếu quan sát cho thấy thông tin đúng đã được nhập nhưng khó tìm, nhóm có lý do thử cách hiển thị trước khi xây kênh liên lạc mới. Nếu nhiều lượt cần trao đổi phát sinh và phương án hiện tại không đáp ứng, chat đáng được đánh giá sâu hơn. Nếu nhãn dữ liệu quá yếu, cần chỉnh cách đo trước khi dùng tỷ lệ đó làm căn cứ đầu tư.

Không phải mọi câu hỏi đều phải được trả lời trước quyết định. Giá trị của việc tìm hiểu phụ thuộc vào việc nó có giảm được rủi ro chọn sai đáng kể so với chi phí hay không. Đây là lập luận quyết định của bài tập, không phải phép tính ROI đã được xác thực.

| Hoạt động đề xuất | Quyết định có thể thay đổi | Giới hạn |
|---|---|---|
| Đối chiếu dữ liệu huỷ và cuộc gọi | Có dùng nhãn hiện tại để chọn lượt và đo mục tiêu? | Không tự xác định được hành vi thực tế |
| Quan sát khách/tài xế tại vài kiểu điểm đón | Ưu tiên thông tin có cấu trúc hay trao đổi tự do? | Mẫu nhỏ không ước lượng tác động toàn khu vực |
| Thử prototype cách hiển thị ghi chú | Người dùng có hiểu và tìm được thông tin? | Không chứng minh giảm huỷ ngoài vận hành |
| Đánh giá kỹ thuật sơ bộ | Phương án nào khả thi trong giới hạn nguồn lực? | Ước lượng sớm còn khoảng bất định |

## 4. Tính chi phí trì hoãn mà không tạo ra con số giả chắc chắn

Baseline minh hoạ có 120 lượt huỷ trước đón trong một tuần. Không được coi cả 120 là lượt mất có thể cứu bằng chat; cũng không được mặc định 50 lượt mang nhãn tìm nhau đều cứu được.

Nếu cần mô hình thảo luận, viết:

```text
Số lượt có thể tránh huỷ trong một tuần
= số lượt huỷ thuộc nhóm thực sự có thể xử lý
  × tỷ lệ giải pháp xử lý thành công trong nhóm đó
```

Giả sử chỉ để phân tích độ nhạy, nhóm chọn các kịch bản 20–50 lượt thuộc nhóm có thể xử lý và hiệu quả 20–60%. Phép nhân cho khoảng 4–30 lượt/tuần. **Đây không phải khoảng tin cậy hoặc dự báo**, vì cả hai đầu vào đều tự giả định. Mục đích là chỉ ra kết luận phụ thuộc mạnh vào thông tin nào.

Muốn quy đổi tiền, còn cần lợi ích ròng trên một lượt được phục hồi, chi phí phát triển và vận hành, tác động tới hành vi khác. Chưa có dữ liệu thì để trống hoặc ghi kịch bản; không lấy giá trị toàn bộ cuốc xe làm lợi nhuận của nền tảng.

## 5. So sánh bước tiếp theo theo mức cam kết

| Lựa chọn | Điều học được / giá trị kỳ vọng | Chi phí hoặc rủi ro | Khả năng đảo ngược |
|---|---|---|---|
| Xây chat đầy đủ ngay | Có thể đáp ứng trao đổi phát sinh | Chọn sai nhu cầu, chi phí tích hợp và vận hành | Thấp hơn vì đã đầu tư và người dùng hình thành kỳ vọng |
| Discovery có giới hạn thời gian | Phân biệt giả thuyết và làm rõ phương án | Vấn đề vẫn tồn tại trong thời gian tìm hiểu | Cao; có thể điều chỉnh hướng |
| Prototype thông tin điểm đón | Kiểm tra khả năng hiểu, dùng và nhận biết | Không chứng minh hiệu quả ngoài thực địa | Cao, nếu chưa đưa vào vận hành |
| Hỗ trợ tạm thời theo ca | Có thể giảm khó khăn trước mắt | Tăng tải nhân sự, chưa mở rộng được | Phụ thuộc cách tổ chức và khả năng ngừng |

Không dùng điểm số 1–5 để che dữ liệu chưa biết. Nếu chấm điểm, nêu người chấm, tiêu chí, trọng số và xem thứ hạng có đảo khi giả định đổi không. Với tình huống P01, bảng định tính có giới hạn rõ đã đủ để đề nghị bước discovery.

## 6. Viết khuyến nghị có điều kiện

Một memo ngắn có thể ghi:

> **Khuyến nghị đề xuất:** chưa cam kết xây chat đầy đủ; dành một tuần discovery tại A để xác minh nhóm nguyên nhân, chất lượng dữ liệu và phương án khả thi. BA tổng hợp bằng chứng; UX phối hợp quan sát; nhóm dữ liệu kiểm tra cohort; Tech Lead đưa đánh giá sơ bộ. Các phân công cần được xác nhận.
>
> **Cuối tuần:** chọn thử nghiệm nhỏ nếu xác định được nhu cầu có bằng chứng và phương án có thể kiểm tra; chưa mở pilot nếu không có cách đo chỉ số chính/guardrail hoặc chưa có người phụ trách vận hành. Nếu thông tin còn thiếu, người quyết định chọn dừng hoặc gia hạn với câu hỏi cụ thể, không tự động kéo dài.

Phân biệt **điều kiện chưa được bắt đầu**, **tín hiệu cần tạm dừng** và **tiêu chí xem xét mở rộng**. Ví dụ chưa xác nhận quyền truy cập dữ liệu thì chưa mở pilot; phát hiện sai quyền trong thử nghiệm thì dừng luồng bị ảnh hưởng để điều tra; prototype dùng được chỉ cho phép xem xét bước tiếp, chưa đủ mở rộng toàn khu vực.

## 7. Bài tập và câu hỏi ôn tập

Stakeholder phản biện: “Discovery cũng tốn tiền; cứ chọn phương án rẻ nhất đi.” Viết câu trả lời tối đa 150 từ, kèm một hoạt động xác minh và kết quả nào sẽ làm bạn đổi khuyến nghị.

Một câu trả lời đạt yêu cầu thừa nhận chi phí tìm hiểu, nêu rủi ro của việc tối ưu giá thành khi chưa biết nhu cầu, giới hạn thời gian và chỉ chọn câu hỏi có thể đổi quyết định. Không cần bảo vệ discovery bằng mọi giá: nếu đã có bằng chứng đủ tốt, việc triển khai thử nhỏ có thể hợp lý hơn.

**Câu 1.** Thử prototype thành công có chứng minh giảm huỷ không?

<details><summary>Gợi ý trả lời</summary><p>Không. Prototype có thể hỗ trợ giả thuyết về hiểu và sử dụng; hiệu quả nghiệp vụ cần quan sát trong điều kiện phù hợp và thiết kế đánh giá riêng.</p></details>

**Câu 2.** Phương án có thể đảo ngược có phải không có rủi ro?

<details><summary>Gợi ý trả lời</summary><p>Không. Thử nghiệm nhỏ vẫn có thể gây phiền, tạo dữ liệu hoặc ảnh hưởng vận hành. Cần giới hạn phạm vi, người chịu trách nhiệm và cách dừng phù hợp.</p></details>

## 8. Tham khảo và bài tiếp theo

- [IIBA — Needs and Solutions, Requirements and Designs](https://www.iiba.org/professional-development/knowledge-centre/articles/needs-and-solutions/): nền khái niệm để phân biệt nhu cầu và phương án. Bảng giả thuyết, kịch bản độ nhạy và memo là ví dụ riêng của bài.
- [Mục tiêu và chỉ số thành công](/articles/ba-problem-statement-muc-tieu-chi-so).

Bài tiếp theo: [Thực hành: bảo vệ đề xuất cải thiện điểm đón](/articles/ba-thuc-hanh-de-xuat-cai-thien-diem-don).
