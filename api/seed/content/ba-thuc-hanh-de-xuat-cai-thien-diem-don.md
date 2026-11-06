---
slug: ba-thuc-hanh-de-xuat-cai-thien-diem-don
title: "Thực hành: bảo vệ đề xuất cải thiện điểm đón"
summary: "Tổng hợp phần 1 thành hồ sơ BA: problem brief, BACCM, metric card, phạm vi, giả định, phương án và decision log. Tự bảo vệ khuyến nghị trước các câu hỏi về dữ liệu, chi phí, thẩm quyền và điều kiện dừng."
category: business-analysis
level: advanced
tags: [business-analysis, ba-008, workshop, case-study, decision-log, toi-noi]
language: vi
---

Bạn có mười phút trước PO, vận hành và Tech Lead. Họ không cần nghe lại toàn bộ định nghĩa BA. Họ cần biết có nên dành nguồn lực để giải quyết vấn đề điểm đón, bước tiếp theo là gì và dựa vào đâu để chốt.

Bài thực hành này gom sản phẩm của bảy bài trước thành một hồ sơ có thể review. Mục tiêu là bảo vệ lập luận và chỉ rõ giới hạn, không phải thuyết phục mọi người chọn một tính năng bằng mọi giá.

<div class="tkh-callout">
<span class="tkh-callout-title">BA-008 · Phần 1/8 · Nâng cao · Bài thực hành tổng hợp</span>
<p>Đọc BA-001 đến BA-007 trước khi làm. Đầu ra: hồ sơ đề xuất 3–5 trang hoặc bộ bảng tương đương; phần trình bày khoảng 10 phút. Dành thêm 60–90 phút làm bài. Toàn bộ dữ liệu, vai trò và quyết định là giả lập; không thay đổi hệ thống thật.</p>
</div>

## 1. Đề bài: xin quyết định nào?

Vận hành đề nghị thêm chat để khách và tài xế tìm nhau dễ hơn. PO muốn quyết định bước tiếp, còn Tech Lead cần hiểu nhu cầu và giới hạn trước khi ước lượng giải pháp. Bạn là BA chuẩn bị buổi review.

Dữ kiện ban đầu, chỉ áp dụng cho khu vực A trong một tuần giả lập:

| Dữ kiện | Giá trị |
|---|---:|
| Lượt đặt xe đã được tài xế nhận | 1.000 |
| Lượt huỷ trước đón | 120 |
| Lượt trong nhóm huỷ có nhãn “không tìm thấy nhau” | 50 |
| Cuộc gọi CSKH gắn nhãn liên quan điểm đón | 80 |

Chưa xác nhận nguyên nhân, chất lượng nhãn, khả năng nối cuộc gọi với lượt đặt xe và baseline khiếu nại. Mục tiêu 9% trong pilot bốn tuần mới là đề xuất. Chưa có phương án nào được duyệt để phát triển.

**Quyết định cần xin:** có dành một tuần discovery tại A hay không; nếu đồng ý, phạm vi, nguồn lực, đầu ra và mốc xem lại là gì? Bạn có thể đề xuất khác, nhưng phải giải thích bằng chứng và đánh đổi. Bài không yêu cầu mặc định duyệt chat hoặc tự bịa một kết quả pilot thành công.

## 2. Hồ sơ cần nộp

| Sản phẩm | Nội dung tối thiểu | Dựa vào bài |
|---|---|---|
| Problem brief | Vấn đề, bằng chứng, tác động cần đo, điều chưa biết, quyết định cần xin | BA-001 |
| Bảng vai trò | Người chuẩn bị, người chốt, người tham vấn; phân biệt PM là Project Manager | BA-002 |
| Chuỗi yêu cầu | Một BR, hai SHR và yêu cầu giải pháp minh hoạ có trạng thái | BA-003 |
| BACCM | Sáu khái niệm và ít nhất ba mối liên hệ quan trọng | BA-004 |
| Metric card | Chỉ số chính, guardrail, cohort, mẫu số và cách xử lý kết quả chưa đủ | BA-005 |
| Scope và sổ giả định | In/out, giả định, ràng buộc, phụ thuộc; có owner đề xuất | BA-006 |
| So sánh phương án | Giữ nguyên/đo thêm, thay quy trình, thay sản phẩm; điều kiện đổi lựa chọn | BA-007 |
| Recommendation và decision log | Khuyến nghị, điều kiện bắt đầu/dừng, người quyết định và mốc xem lại | BA-002, BA-007 |

Không cần viết nhiều trang cho mỗi sản phẩm. Một bảng có ID tham chiếu rõ giúp người review lần theo lập luận tốt hơn các đoạn văn lặp lại. Mẫu điền trong repository nằm ở `examples/toi-noi-ba/p01-ho-so-de-xuat.md`.

## 3. Làm bài theo bốn vòng

### Vòng 1 — Kiểm tra điều đã biết

Tính 120/1.000, 50/1.000 và 50/120; ghi bên cạnh câu hỏi mà mỗi tỷ lệ trả lời. Đánh dấu 80 là cuộc gọi, chưa phải số khách hoặc số lượt duy nhất. Viết problem statement không dùng cụm “do thiếu chat”.

Sau đó tách ghi chép thành dữ kiện giả lập, giả thuyết và đề xuất. Các mức mục tiêu, owner và lịch chưa được xác nhận phải giữ nhãn đề xuất.

### Vòng 2 — Nối nhu cầu với lựa chọn

Lập BACCM, ghi khách/tài xế/CSKH có thể được lợi và chịu chi phí gì. Viết ít nhất hai nhu cầu stakeholder trước khi ghi giải pháp. Với mỗi phương án, chỉ ra giả thuyết nó xử lý và bằng chứng nào còn thiếu.

Ví dụ, cải thiện ghi chú có thể phục vụ nhu cầu tài xế nhận biết đúng cổng. Nó chưa chắc giải quyết tình huống cổng vừa đóng và khách phải di chuyển. Cần thể hiện giới hạn đó trong bảng so sánh.

### Vòng 3 — Thiết kế cách học và cách đo

Chọn hoạt động discovery theo câu hỏi có thể thay đổi quyết định. Đừng biến một tuần thành lịch kín các cuộc họp không có đầu ra. Nếu muốn quan sát người dùng, ghi cách chọn ca và giới hạn suy rộng. Nếu cần dữ liệu, ghi điều kiện chất lượng phải kiểm tra.

Metric card phải phân biệt số liệu tạm thời với kết quả đã đủ điều kiện đánh giá. Chưa có baseline guardrail thì ghi công việc phải làm trước pilot, không đánh dấu “đạt”.

### Vòng 4 — Chuẩn bị quyết định và phản biện

Viết khuyến nghị tối đa một trang. Nêu việc cần duyệt ngay, việc chưa cần chốt và phương án nếu điều kiện không đạt. Chuẩn bị trả lời vì sao không xây ngay, vì sao không nghiên cứu vô hạn và ai có quyền đổi phạm vi.

## 4. Gợi ý một hồ sơ đạt yêu cầu

Đây là một lời giải có thể chấp nhận, không phải quyết định đã diễn ra của Tới Nơi.

**Khuyến nghị:** dành một tuần discovery tại A; chưa chốt triển khai chat. Cơ sở là có tín hiệu khó khăn ở điểm đón nhưng chưa phân biệt được nguyên nhân và chưa đủ dữ liệu so sánh giải pháp.

**Phạm vi:** kiểm tra dữ liệu lượt huỷ, quan sát một số kiểu điểm đón, xác nhận cách vận hành hỗ trợ và lấy đánh giá kỹ thuật sơ bộ. Chưa bao gồm triển khai chat, giao hàng nhiều điểm hoặc thay chính sách phí huỷ.

**Đầu ra:** brief được cập nhật; bảng bằng chứng theo giả thuyết; metric card được review; phương án và khoảng ước lượng có giả định; đề xuất bước tiếp. Không cam kết discovery tự làm tỷ lệ huỷ giảm.

**Điều kiện:** PO xác nhận ưu tiên trong thẩm quyền; người tham gia xác nhận khả năng hỗ trợ; dữ liệu chia sẻ được chuẩn bị phù hợp. Nếu không đủ điều kiện, thu hẹp discovery hoặc chuyển quyết định lên người có thẩm quyền, ghi ảnh hưởng rõ ràng.

**Mốc xem lại:** cuối tuần discovery, trình kết quả và một trong ba lựa chọn: thử nhỏ một phương án, xác minh thêm một câu hỏi quan trọng với thời hạn mới, hoặc dừng/đổi ưu tiên. Gia hạn cần quyết định mới, không tự động.

### Bảng truy vết minh hoạ

| Mục | Liên kết | Điều cần kiểm tra |
|---|---|---|
| BR-01: mục tiêu huỷ trước đón không quá 9% | Problem brief tại A | Baseline, khả năng đo, tính khả thi và duyệt mục tiêu |
| SHR-01: tài xế cần xác định cổng khách đang chờ | BR-01 | Nhu cầu thực tế qua quan sát |
| SHR-02: khách cần biết thông tin nào đã gửi | BR-01 | Khách có hiểu dữ liệu đang được dùng không? |
| FR-01: hiển thị ghi chú điểm đón | SHR-01, SHR-02 | Chỉ là yêu cầu minh hoạ nếu chọn phương án ghi chú |
| M-01: tỷ lệ huỷ trước đón | BR-01 | Định nghĩa cohort, ID duy nhất và kết quả cuối |

Chuỗi này không biến FR-01 thành giải pháp đã duyệt. Nếu chọn phương án khác, cập nhật yêu cầu giải pháp nhưng vẫn giữ nhu cầu và lịch sử quyết định để giải thích thay đổi.

## 5. Phản biện trực tiếp trong buổi review

| Câu hỏi | Cách trả lời có căn cứ |
|---|---|
| “50 lượt không tìm thấy nhau, chat cứu được bao nhiêu?” | Chưa có tỷ lệ hiệu quả; nhãn cũng cần kiểm tra. Đề xuất hoạt động phân biệt nguyên nhân và đánh giá phương án |
| “Có 80 cuộc gọi, tức 80 khách?” | Chưa biết; cần gộp theo người/lượt và phân biệt cuộc gọi với khiếu nại |
| “Nếu giao diện ghi chú rẻ hơn, chọn luôn?” | Cần xem có xử lý đúng nhu cầu và có dùng được trong bối cảnh thật không |
| “Prototype mọi người đều dùng được thì rollout?” | Prototype hỗ trợ kết luận về khả năng sử dụng trong mẫu; chưa chứng minh hiệu quả nghiệp vụ hoặc vận hành |
| “Ai chấp nhận mục tiêu 9%?” | Sponsor/đại diện nghiệp vụ theo uỷ quyền sau khi review baseline, guardrail và nguồn lực |

Trong vai BA, bạn có thể nói “chưa đủ bằng chứng” nhưng phải nối với bước xử lý. Nếu người quyết định vẫn chọn xây ngay, ghi rõ căn cứ, bất định đã nêu, thẩm quyền, điều kiện theo dõi và quyết định thực tế; không sửa lịch sử để làm như mọi giả định đã được xác minh.

## 6. Tiêu chí dừng và điều kiện chuyển bước

Với discovery, nên xem lại hoặc dừng hướng hiện tại khi không tiếp cận được nguồn bằng chứng quan trọng, vấn đề ưu tiên đã đổi, hoặc kết quả bác bỏ giả thuyết cốt lõi. “Dừng hướng chat” không nhất thiết “dừng giải quyết điểm đón”.

Trước pilot cần có định nghĩa metric, baseline có thể sử dụng, owner hỗ trợ, phạm vi, cơ chế ngừng và điều kiện review đã thống nhất. Không có một ngưỡng số lượng chung cho mọi tình huống. Nếu phát hiện thông tin hiển thị sai người, cần dừng luồng bị ảnh hưởng và điều tra theo kế hoạch đã chốt; không chờ hết bốn tuần để xem tỷ lệ huỷ.

Sau pilot, chỉ xem xét mở rộng khi dữ liệu đủ, guardrail được đánh giá và giới hạn của kết luận được nêu rõ. Đạt target trong một khu vực không tự chứng minh hiệu quả tại mọi khu vực.

## 7. Rubric tự đánh giá và biến thể nâng cao

Chấm mỗi tiêu chí 0–2: **0** thiếu/sai; **1** có nhưng còn mơ hồ; **2** rõ và nhất quán với hồ sơ. Tổng tối đa 10 điểm.

| Tiêu chí | Điều cần thấy để đạt 2 điểm |
|---|---|
| Bằng chứng | Đúng mẫu số, phân biệt dữ kiện và giả thuyết, nêu giới hạn |
| Nhu cầu và phương án | Không chốt chat từ đầu; có ít nhất ba lựa chọn và đánh đổi |
| Đo lường | Cohort, pending, guardrail và baseline thiếu được xử lý rõ |
| Phạm vi và trách nhiệm | Có in/out, owner đề xuất, quyền chốt và phụ thuộc |
| Khuyến nghị | Quyết định cần xin rõ, có điều kiện, mốc review và cách đổi hướng |

Mốc tự học đề xuất: đạt từ 8/10, đồng thời không có tiêu chí nào bằng 0. Nếu dùng 80 cuộc gọi thành 80 khách hoặc biến target thành kết quả đạt được, sửa bằng chứng trước khi trình bày dù tổng điểm có vẻ cao. Đây là rubric của bài tập, không phải đánh giá chứng chỉ BA.

**Biến thể:** giữa buổi review, PO yêu cầu thêm khu vực B. Viết impact note thay vì sửa tên khu vực ngay. Cần kiểm tra tính đại diện, khác biệt điểm đón, owner vận hành, dữ liệu đo và nguồn lực. Bạn có thể đề xuất tách B thành discovery tiếp theo với lý do cụ thể.

**Câu hỏi kết bài:** Đầu ra có giá trị nhất của P01 là gì?

<details><summary>Gợi ý trả lời</summary><p>Một quyết định được hỗ trợ bởi lập luận, bằng chứng và các giới hạn rõ ràng. Tài liệu giúp nhóm phối hợp và xem lại quyết định khi thông tin đổi; số lượng trang hoặc tên kỹ thuật không tự tạo giá trị.</p></details>

## 8. Tài liệu và bước tiếp theo

- Hồ sơ và mẫu điền: `examples/toi-noi-ba/README.md`, `examples/toi-noi-ba/p01-ho-so-de-xuat.md` trong repository.
- [BA là gì?](/articles/ba-la-gi-tu-tinh-nang-den-bai-toan) · [BACCM](/articles/ba-baccm-sau-khai-niem-cot-loi) · [Metric card](/articles/ba-problem-statement-muc-tieu-chi-so) · [Quyết định khi thiếu bằng chứng](/articles/ba-quyet-dinh-khi-thieu-bang-chung).

Phần 1 hoàn thành ở đây. Tiếp tục phần 2 với [Stakeholder map: ai chịu tác động, ai có quyền quyết định?](/articles/ba-stakeholder-map-quyen-quyet-dinh), rồi đi sâu vào phỏng vấn, quan sát, workshop và tổng hợp bằng chứng.
