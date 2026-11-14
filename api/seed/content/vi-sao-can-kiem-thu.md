---
slug: vi-sao-can-kiem-thu
title: "Vì sao cần kiểm thử? (Why is Testing Necessary?)"
summary: "Bốn đóng góp của kiểm thử vào thành công dự án theo CTFL v4.0.1, minh hoạ bằng sự cố giả định Tới Nơi giao nhầm bưu kiện: chuỗi hậu quả, mô hình chi phí có công thức và hoạt động kiểm thử chặn từng khâu."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-002, why-testing, cost-of-failure, quality-control, toi-noi]
language: vi
---

Kiểm thử tốn tiền: người, thời gian, môi trường, công cụ. Vậy đổi lại được gì, và vì sao một nền tảng giao hàng không thể «cứ phát hành rồi sửa sau»? Bài này trả lời theo mục 1.2 của syllabus CTFL v4.0.1, qua một sự cố giả định của Tới Nơi: **bưu kiện được giao tới địa chỉ cũ** sau khi khách đã sửa địa chỉ — và tính thử sự cố đó tốn bao nhiêu, kiểm thử chặn được ở những khâu nào.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-002</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S01 · Fundamentals of Software Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/software-testing-la-gi-testing-khac-debugging">TEST-001 · Software Testing là gì? Testing khác Debugging</a> · <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Giấy/bảng tính: bảng thiệt hại của một sự cố giao nhầm bưu kiện (có công thức) và bảng «khâu → hoạt động kiểm thử lẽ ra chặn»</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.2, 1.2.1 · FL-1.2.1 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

[Bài trước](/articles/software-testing-la-gi-testing-khac-debugging) trả lời kiểm thử *là gì*. Bài này trả lời câu khó thuyết phục hơn: *vì sao phải bỏ tiền cho nó*.

Câu hỏi này xuất hiện ở mọi dự án, thường dưới dạng: «Sprint này gấp, bỏ bớt test được không?», «Developer tự test là đủ, cần tester làm gì?», «Lỗi thì khách báo, mình sửa sau». Muốn trả lời có sức nặng, cần hai thứ:

- **Lập luận chuẩn** — kiểm thử đóng góp gì vào thành công của dự án, theo cách ISTQB mô tả.
- **Con số cụ thể của chính hệ thống** — một failure lọt ra production thì ai trả giá, trả bao nhiêu, và kiểm thử chặn được ở khâu nào.

Ở Tới Nơi, quy mô biến lỗi nhỏ thành thiệt hại lớn: 2 triệu chuyến mỗi ngày, trong đó 0,3 triệu đơn bưu kiện và 0,9 triệu đơn đồ ăn ([hồ sơ dự án](/articles/du-an-mau-toi-noi)). Một defect chỉ xảy ra với một phần nghìn đơn vẫn là hàng trăm đơn mỗi ngày.

Dùng bài này khi bạn cần giải thích giá trị của kiểm thử cho product owner, quản lý dự án hay chính đội phát triển — và khi ôn LO FL-1.2.1 của kỳ thi CTFL.

## 2. Lý thuyết theo ISTQB

### 2.1. Phần mềm không chạy đúng thì sao

Mục 1.1 của syllabus mở đầu bằng một nhận xét: hầu như ai cũng từng gặp phần mềm chạy không như mong đợi, và phần mềm sai có thể gây **mất tiền, mất thời gian, mất uy tín kinh doanh**, trường hợp nghiêm trọng là **thương tích hoặc tử vong**. Kiểm thử giúp đánh giá chất lượng và **giảm rủi ro phần mềm thất bại khi vận hành**.

Với Tới Nơi, cả bốn loại hậu quả đều có thật:

| Hậu quả (theo 1.1) | Ví dụ ở Tới Nơi |
|---|---|
| Mất tiền | Hoàn cước, bồi thường hàng thất lạc, thu thiếu cước |
| Mất thời gian | Tài xế chạy lại, tổng đài xử lý khiếu nại, đội kỹ thuật trực sự cố |
| Mất uy tín | Đánh giá 1 sao, khách và quán chuyển sang nền tảng khác |
| Thương tích | Món có đậu phộng giao nhầm cho khách bị dị ứng; tài xế bị điều tới điểm đón sai giữa đêm |

### 2.2. Kiểm thử là một hình thức kiểm soát chất lượng

Mục 1.2 định vị kiểm thử là **một hình thức quality control** (kiểm soát chất lượng): nó giúp dự án đạt các mục tiêu kiểm thử đã thống nhất **trong giới hạn** phạm vi, thời gian, chất lượng và ngân sách. Hai ý đi kèm:

- Đóng góp của kiểm thử **không chỉ đến từ đội kiểm thử**. Bất kỳ bên liên quan nào — developer, BA, product owner, nhân viên vận hành — cũng có thể dùng kỹ năng kiểm thử để đưa dự án gần thành công hơn.
- Kiểm thử component, hệ thống **và các work product đi kèm** (ví dụ tài liệu) đều giúp tìm defect.

Kiểm thử khác quality assurance thế nào là nội dung của mục 1.2.2 — xem [Mục tiêu kiểm thử và Quality Assurance](/articles/muc-tieu-kiem-thu-va-quality-assurance).

### 2.3. Bốn đóng góp vào thành công (FL-1.2.1, K2)

Mục 1.2.1 nêu bốn cách kiểm thử đóng góp vào thành công. Diễn đạt lại:

| # | Đóng góp | Ý chính | Lưu ý hay bị hiểu sai |
|---:|---|---|---|
| 1 | **Phát hiện defect một cách hiệu quả về chi phí** | Kiểm thử tìm defect; debugging (một hoạt động *không phải* kiểm thử) gỡ chúng | Kiểm thử chỉ **gián tiếp** nâng chất lượng test object — chất lượng tăng khi defect được gỡ |
| 2 | **Đánh giá trực tiếp chất lượng** ở nhiều giai đoạn của SDLC | Kết quả đo được là đầu vào cho quản lý dự án | Dùng cho quyết định chuyển giai đoạn, trong đó có **quyết định phát hành** |
| 3 | **Đại diện gián tiếp cho người dùng** trong dự án | Tester bảo đảm hiểu biết về nhu cầu người dùng được tính tới suốt vòng đời | Cách thay thế là đưa một nhóm người dùng đại diện vào dự án — thường không khả thi vì tốn kém và khó có đúng người |
| 4 | **Đáp ứng yêu cầu hợp đồng, pháp lý, quy định** | Có trường hợp kiểm thử là bắt buộc | Không phải lý do «phụ»: thiếu nó có thể không được phép vận hành |

Đây là LO mức **K2 — exemplify**: bạn cần **lấy được ví dụ** cho từng đóng góp, không chỉ đọc thuộc. Mục 3–5 của bài làm đúng việc đó.

### 2.4. Bảng thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa tiếng Việt dùng trong blog | Ghi chú |
|---|---|---|
| quality control (QC) | kiểm soát chất lượng | kiểm thử là một hình thức QC, hướng vào sản phẩm |
| quality assurance (QA) | đảm bảo chất lượng | hướng vào quy trình — [TEST-006](/articles/muc-tieu-kiem-thu-va-quality-assurance) |
| stakeholder | bên liên quan | khách, tài xế, quán, vận hành, tài chính, pháp chế… |
| test object | đối tượng kiểm thử | xem [TEST-001](/articles/software-testing-la-gi-testing-khac-debugging) |
| defect / failure | khiếm khuyết / thất bại | phân biệt kỹ ở [TEST-004](/articles/error-defect-failure-root-cause) |
| debugging | gỡ lỗi | hoạt động không phải kiểm thử |
| release decision | quyết định phát hành | một quyết định chuyển giai đoạn |
| contractual / legal / regulatory requirement | yêu cầu hợp đồng / pháp lý / quy định | lý do thứ tư của 1.2.1 |
| risk | rủi ro | khái niệm đầy đủ ở chương 5 CTFL (Series 12) |

### 2.5. Phạm vi áp dụng

Lập luận của 1.2.1 áp dụng cho mọi loại dự án, nhưng **trọng số khác nhau theo ngữ cảnh**. Trang giới thiệu dịch vụ của Tới Nơi chủ yếu cần đóng góp 3 (người dùng hiểu được bảng giá). Payment & Wallet cần cả bốn, trong đó đóng góp 4 nặng nhất. Chọn mức đầu tư kiểm thử theo rủi ro là chủ đề của Series 12; ở Series 01 chỉ cần thấy rằng «vì sao cần kiểm thử» luôn có câu trả lời cụ thể cho từng phần hệ thống.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Ngoài syllabus ISTQB: «chi phí sửa lỗi tăng 10 lần mỗi giai đoạn»</span>
<p>Nhiều tài liệu ngành trích một tỉ lệ cố định kiểu «lỗi phát hiện ở production đắt gấp 100 lần lúc viết yêu cầu». CTFL v4.0.1 <b>không</b> đưa ra tỉ lệ nào; syllabus chỉ nói kiểm thử phát hiện defect một cách hiệu quả về chi phí, và nguyên lý «early testing saves time and money» (xem <a href="/articles/bay-nguyen-ly-kiem-thu">Bảy nguyên lý kiểm thử</a>). Các tỉ lệ cố định đó phụ thuộc mạnh vào từng dự án. Blog không dùng chúng; mọi con số chi phí trong bài là <b>giả định có công thức</b> để bạn thay bằng số của mình.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Dịch vụ giao bưu kiện (`parcel`) cho khách sửa địa chỉ người nhận **trước khi tài xế lấy hàng** — ví dụ người nhận báo đã chuyển văn phòng. Tài xế nhìn địa chỉ trên Driver App để giao.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Phạm vi của tình huống</span>
<p>Lab <code>toi-noi-testing</code> không có điểm đón/điểm giao — client gửi thẳng quãng đường (R-GEN-06) — và không có Kafka hay sự kiện (R-ARCH-02). Tình huống của bài thuộc <b>hệ thống đích</b> theo tài liệu kiến trúc; phần thực hành là bài phân tích trên giấy/bảng tính, không chạy code. Các quy tắc <code>BR-DLV-…</code> dưới đây là <b>giả định của bài</b>, không có trong SPEC lab.</p>
</div>

### 3.2. Business rules

| Mã | Quy tắc | Nguồn |
|---|---|---|
| BR-DLV-01 | Địa chỉ người nhận được sửa khi đơn chưa ở `picked_up`; sửa sau đó bị từ chối | giả định của bài |
| BR-DLV-02 | Driver App luôn hiển thị **bản địa chỉ mới nhất** mà Booking/Order đã chấp nhận | giả định của bài |
| BR-DLV-03 | Thay đổi địa chỉ tới được app tài xế trong p95 dưới 2–3 giây | SLO «Realtime status propagation», kiến trúc §14.1 |
| R-STATE-02 | Vòng đời 8 trạng thái; `picked_up` đứng sau `arriving` | SPEC lab |
| R-ACCEPT-03 | Nhiều tài xế cùng nhận một chuyến ⇒ đúng một người thắng | SPEC lab |
| R-API-05 | Không lộ chuyến của khách khác (chống IDOR) | SPEC lab |

### 3.3. System context

Theo tài liệu kiến trúc Tới Nơi:

- **Booking/Order Service** là nguồn dữ liệu gốc của đơn: lưu điểm lấy/điểm giao và **quản lý version để cập nhật cũ không ghi đè cập nhật mới** (§4.4).
- **Trip/Delivery Lifecycle Service** quản lý vòng đời sau khi có tài xế (§4.7) và cung cấp dữ liệu cho Driver App qua Driver BFF.
- Hai service nối nhau bằng **Kafka** theo kiểu at-least-once + consumer idempotent + transactional outbox/inbox (§1.1, §6.3). Sự kiện của đơn giao hàng đi trên topic `delivery.events.v1`, key `order_id` (§6.4). Tên sự kiện «đổi địa chỉ», ví dụ `DeliveryAddressChanged`, là giả định của bài.
- Kafka chỉ giữ thứ tự **trong một partition** (§6.5). Bảng rủi ro §14 ghi đúng hai mối nguy của tình huống: **sự kiện trùng** (xử lý bằng dedup/inbox) và **sự kiện đến sai thứ tự** (xử lý bằng aggregate version và kiểm tra chuyển trạng thái).

### 3.4. Sự cố giả định SC-01

1. 10:00 — khách tạo đơn bưu kiện, địa chỉ người nhận A (version 1).
2. 10:02 — khách sửa thành địa chỉ B (version 2). Booking chấp nhận, phát sự kiện v2.
3. Bản tin mang địa chỉ A (v1) **đến Trip service sau** bản tin v2 — do được gửi lại (at-least-once) hoặc do một đường cập nhật thứ hai chậm hơn.
4. Trip service ghi đè **không so version** ⇒ Driver App hiện địa chỉ A.
5. Tài xế giao bưu kiện tới A. Người ở A ký nhận hộ hoặc từ chối; người nhận thật ở B không nhận được hàng.

### 3.5. Rủi ro

| Nhóm | Rủi ro cụ thể |
|---|---|
| Tài chính | Hoàn cước, bồi thường hàng, chi phí giao lại, chi phí tổng đài |
| Vận hành | Tài xế mất thêm một chuyến; đơn khác của tài xế đó bị trễ |
| Uy tín | Người gửi (thường là shop bán online) mất khách, chuyển nền tảng |
| Pháp lý / dữ liệu | Tên, số điện thoại, nội dung hàng của người nhận lộ cho người lạ ở địa chỉ A |
| An toàn | Biến thể đồ ăn: món chứa chất gây dị ứng tới nhầm người |

## 4. Phân tích kỹ thuật

### 4.1. Chuỗi hậu quả và các điểm chặn

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 290" role="img" aria-label="Chuỗi hậu quả của sự cố SC-01 gồm năm ô nối tiếp: yêu cầu mơ hồ, defect ở consumer, failure giao nhầm bưu kiện, hậu quả trực tiếp, hậu quả gián tiếp. Bên dưới bốn ô đầu là các điểm chặn: review yêu cầu, test version check ở mức component và integration, test luồng đầy đủ ở mức system và acceptance, giám sát vận hành (ngoài kiểm thử). Ô cuối không còn điểm chặn. Mũi tên dưới cùng: chặn càng sớm thì càng ít khâu phải trả giá.">
<defs>
<marker id="t002-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
<marker id="t002-stop" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-phys"/></marker>
</defs>
<text x="20" y="30" class="t-b">Chuỗi hậu quả của SC-01 và các điểm kiểm thử có thể chặn</text>
<rect x="20" y="52" width="156" height="72" rx="8" class="f-pk-soft s-pk"/>
<text x="98" y="74" text-anchor="middle" class="t-b">Yêu cầu mơ hồ</text>
<text x="98" y="92" text-anchor="middle" class="t-small">BR-DLV-02 thiếu luật</text>
<text x="98" y="108" text-anchor="middle" class="t-small">thứ tự cập nhật</text>
<rect x="201" y="52" width="156" height="72" rx="8" class="f-pk-soft s-pk"/>
<text x="279" y="74" text-anchor="middle" class="t-b">Defect ở consumer</text>
<text x="279" y="92" text-anchor="middle" class="t-small">không so version,</text>
<text x="279" y="108" text-anchor="middle" class="t-small">bản tin cũ đè bản mới</text>
<rect x="382" y="52" width="156" height="72" rx="8" class="f-warn-soft s-warn"/>
<text x="460" y="74" text-anchor="middle" class="t-b t-warn">Failure</text>
<text x="460" y="92" text-anchor="middle" class="t-small">tài xế thấy địa chỉ cũ,</text>
<text x="460" y="108" text-anchor="middle" class="t-small">giao nhầm bưu kiện</text>
<rect x="563" y="52" width="156" height="72" rx="8" class="f-warn-soft s-warn"/>
<text x="641" y="74" text-anchor="middle" class="t-b">Hậu quả trực tiếp</text>
<text x="641" y="92" text-anchor="middle" class="t-small">hoàn cước, bồi thường,</text>
<text x="641" y="108" text-anchor="middle" class="t-small">giao lại, tổng đài</text>
<rect x="744" y="52" width="156" height="72" rx="8" class="f-warn-soft s-warn"/>
<text x="822" y="74" text-anchor="middle" class="t-b">Hậu quả gián tiếp</text>
<text x="822" y="92" text-anchor="middle" class="t-small">uy tín, khách rời bỏ,</text>
<text x="822" y="108" text-anchor="middle" class="t-small">dữ liệu cá nhân bị lộ</text>
<line x1="176" y1="88" x2="199" y2="88" class="s-muted" stroke-width="1.6" marker-end="url(#t002-arrow)"/>
<line x1="357" y1="88" x2="380" y2="88" class="s-muted" stroke-width="1.6" marker-end="url(#t002-arrow)"/>
<line x1="538" y1="88" x2="561" y2="88" class="s-muted" stroke-width="1.6" marker-end="url(#t002-arrow)"/>
<line x1="719" y1="88" x2="742" y2="88" class="s-muted" stroke-width="1.6" marker-end="url(#t002-arrow)"/>
<rect x="20" y="176" width="156" height="56" rx="8" class="f-phys-soft s-phys"/>
<text x="98" y="198" text-anchor="middle" class="t-b">Review yêu cầu</text>
<text x="98" y="216" text-anchor="middle" class="t-small">kiểm thử tĩnh</text>
<rect x="201" y="176" width="156" height="56" rx="8" class="f-phys-soft s-phys"/>
<text x="279" y="198" text-anchor="middle" class="t-b">Test version check</text>
<text x="279" y="216" text-anchor="middle" class="t-small">component, integration</text>
<rect x="382" y="176" width="156" height="56" rx="8" class="f-phys-soft s-phys"/>
<text x="460" y="198" text-anchor="middle" class="t-b">Test luồng đầy đủ</text>
<text x="460" y="216" text-anchor="middle" class="t-small">system, acceptance</text>
<rect x="563" y="176" width="156" height="56" rx="8" class="f-bg s-muted dash"/>
<text x="641" y="198" text-anchor="middle" class="t-b">Giám sát vận hành</text>
<text x="641" y="216" text-anchor="middle" class="t-small">ngoài kiểm thử</text>
<text x="822" y="208" text-anchor="middle" class="t-small">không còn điểm chặn</text>
<line x1="98" y1="176" x2="98" y2="127" class="s-phys dash" stroke-width="1.4" marker-end="url(#t002-stop)"/>
<line x1="279" y1="176" x2="279" y2="127" class="s-phys dash" stroke-width="1.4" marker-end="url(#t002-stop)"/>
<line x1="460" y1="176" x2="460" y2="127" class="s-phys dash" stroke-width="1.4" marker-end="url(#t002-stop)"/>
<line x1="641" y1="176" x2="641" y2="127" class="s-muted dash" stroke-width="1.4" marker-end="url(#t002-arrow)"/>
<text x="106" y="156" class="t-small t-phys">chặn</text>
<text x="287" y="156" class="t-small t-phys">chặn</text>
<text x="468" y="156" class="t-small t-phys">chặn</text>
<text x="649" y="156" class="t-small">phát hiện sớm</text>
<line x1="20" y1="268" x2="898" y2="268" class="s-muted" stroke-width="1.4" marker-end="url(#t002-arrow)"/>
<text x="460" y="258" text-anchor="middle" class="t-small">chặn càng ở bên trái, càng ít khâu phía sau phải trả giá</text>
</svg>
</div>
<figcaption><b>Hình 1 · Từ một yêu cầu mơ hồ tới thiệt hại.</b> Ô hồng là nơi defect sinh ra và tồn tại, ô cam là failure và hậu quả. Ô xanh là hoạt động kiểm thử có thể chặn chuỗi ở khâu tương ứng; giám sát vận hành không phải kiểm thử, chỉ rút ngắn thời gian failure tồn tại.</figcaption>
</figure>

Đọc hình từ trái sang phải:

- Nếu **review yêu cầu** hỏi được câu «hai lần sửa địa chỉ tới lệch thứ tự thì sao?», defect không bao giờ được viết ra. Đây là đóng góp 1 (phát hiện defect hiệu quả về chi phí) ở dạng rẻ nhất: kiểm thử tĩnh, không có failure nào.
- Nếu yêu cầu đã rõ mà code vẫn sai, **test ở mức component/integration** (áp v2 rồi gửi v1) bắt được defect trước khi ghép cả hệ thống. Các test level được giới thiệu ở Series 02.
- Nếu lọt tới **test luồng đầy đủ**, failure vẫn xảy ra — nhưng trong môi trường test, không phải trên bưu kiện thật.
- Qua khỏi đó, chỉ còn **giám sát vận hành** (tỉ lệ khiếu nại «không nhận được hàng» tăng bất thường) giúp dừng sớm. Giám sát rút ngắn thời gian thiệt hại, không ngăn được đơn đầu tiên bị giao nhầm.

### 4.2. Mô hình chi phí của SC-01

Mọi tham số đánh dấu *(gs)* là **giả định của bài**; số đơn lấy từ hồ sơ dự án.

| Ký hiệu | Ý nghĩa | Giá trị |
|---|---|---:|
| N | Đơn bưu kiện mỗi ngày | 300.000 (hồ sơ dự án) |
| p₁ | Tỉ lệ đơn bị sửa địa chỉ trước khi lấy hàng | 2% *(gs)* |
| p₂ | Tỉ lệ lần sửa gặp bản tin cũ đến sau | 0,5% *(gs)* |
| C | Chi phí trực tiếp một đơn giao nhầm | 265.000 đ *(gs, chi tiết ở 5.1)* |
| D | Số ngày từ lúc phát hành tới lúc sửa xong | 10 ngày *(gs)* |

Số đơn giao nhầm mỗi ngày = N × p₁ × p₂ = 300.000 × 0,02 × 0,005 = **30 đơn/ngày**.

Chi phí trực tiếp = 30 × 265.000 × D = 7.950.000 đ/ngày × 10 ngày = **79,5 triệu đồng**.

Ba điều rút ra, không phụ thuộc vào con số chính xác:

1. **Xác suất nhỏ × quy mô lớn = thiệt hại đều đặn.** Một defect chỉ lộ ra ở 0,01% đơn vẫn tạo 30 sự cố mỗi ngày.
2. **D là biến số mà kiểm thử và giám sát kiểm soát được.** Chặn trước phát hành ⇒ D = 0. Giám sát tốt rút D từ 10 ngày xuống 1 ngày ⇒ chi phí trực tiếp còn 7,95 triệu.
3. **C chưa gồm hậu quả gián tiếp** — khách rời bỏ, dữ liệu cá nhân bị lộ, và với biến thể đồ ăn, nguy cơ sức khoẻ không quy được ra tiền.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Nhắc lại lỗi cước của TEST-001 ở quy mô thật</span>
<p>Defect làm tròn xuống ở <a href="/articles/software-testing-la-gi-testing-khac-debugging">TEST-001</a> thu thiếu từ 1 đến 999 đ mỗi chuyến có tổng lẻ. Giả định <i>(gs)</i> 90% chuyến đặt xe có tổng lẻ và thiếu trung bình 500 đ: 800.000 chuyến × 0,9 × 500 đ ≈ <b>360 triệu đồng mỗi ngày</b>. Không khách nào khiếu nại vì họ được lợi — failure loại này chỉ kiểm thử (hoặc đối soát tài chính) mới thấy.</p>
</div>

### 4.3. Bốn đóng góp của 1.2.1 áp vào SC-01

| Đóng góp | Ở SC-01 trông như thế nào |
|---|---|
| 1. Phát hiện defect hiệu quả về chi phí | Một câu hỏi trong buổi review BR-DLV-02, hoặc một test áp v2 rồi v1, rẻ hơn nhiều so với 79,5 triệu đồng hậu quả. Developer sửa (debugging) — kiểm thử chỉ phát hiện |
| 2. Đánh giá chất lượng cho quyết định | Báo cáo kiểm thử trước khi bật tính năng «sửa địa chỉ»: «test sai thứ tự sự kiện chưa chạy» ⇒ product owner quyết định bật thử ở một thành phố thay vì cả 6 |
| 3. Đại diện gián tiếp cho người dùng | Tester đóng vai tài xế đang trên đường tới lấy hàng thì địa chỉ đổi: app có báo rõ không, có rung/chuông không? 25.000 tài xế trực tuyến lúc cao điểm không thể ngồi họp sprint |
| 4. Hợp đồng, pháp lý, quy định | Cam kết bồi thường với shop gửi hàng; quy định bảo vệ dữ liệu cá nhân của người nhận — kiểm thử cung cấp bằng chứng đã kiểm tra các yêu cầu này |

## 5. Thực hành

Bài thực hành không cần code: hai bảng, làm trên giấy hoặc bảng tính.

### 5.1. Bảng thiệt hại một đơn giao nhầm

| Khoản | Công thức | Giả định | Giá trị |
|---|---|---|---:|
| Hoàn cước giao | cước trung bình đơn bưu kiện | 35.000 đ *(gs)* | 35.000 đ |
| Bồi thường hàng | tỉ lệ không thu hồi được × mức bồi thường trung bình | 30% × 500.000 đ *(gs)* | 150.000 đ |
| Giao lại | một chuyến tài xế lấy lại và giao đúng | 30.000 đ *(gs)* | 30.000 đ |
| Tổng đài | phút xử lý × chi phí giờ công ÷ 60 | 20 phút × 90.000 đ/giờ *(gs)* | 30.000 đ |
| Voucher xin lỗi | cố định | 20.000 đ *(gs)* | 20.000 đ |
| **C — trực tiếp** | tổng các dòng trên | | **265.000 đ** |
| Khách rời bỏ *(gián tiếp)* | tỉ lệ rời bỏ × giá trị vòng đời khách | 30% × 1.200.000 đ *(gs)* | 360.000 đ |

Công thức để dán vào bảng tính (cột B là giá trị, hàng 2–6 là năm khoản trực tiếp, hàng 9–12 là N, p₁, p₂, D):

```text
B7  = SUM(B2:B6)                  ' C, chi phí trực tiếp một đơn
B13 = B9 * B10 * B11              ' đơn giao nhầm mỗi ngày
B14 = B13 * B7 * B12              ' chi phí trực tiếp cả đợt
B15 = B13 * B12 * B8              ' chi phí gián tiếp (B8 = khách rời bỏ / đơn)
B16 = B14 + B15                   ' tổng
```

Kết quả mong đợi với giả định của bài: B13 = 30 đơn/ngày; B14 = 79.500.000 đ; B15 = 30 × 10 × 360.000 = 108.000.000 đ; B16 = **187.500.000 đ**.

Phân tích độ nhạy: đổi một tham số, giữ nguyên phần còn lại:

| Thay đổi | Đơn/ngày | Tổng (trực tiếp + gián tiếp) |
|---|---:|---:|
| Như trên | 30 | 187,5 triệu đ |
| p₂ nhỏ hơn 10 lần (0,05%) | 3 | 18,75 triệu đ |
| Giám sát phát hiện sau 1 ngày (D = 1) | 30 | 18,75 triệu đ |
| Bắt được trước phát hành (D = 0) | 0 | 0 đ |

Hai dòng giữa cho cùng một con số: **giảm xác suất xảy ra 10 lần** và **phát hiện nhanh gấp 10 lần** có giá trị ngang nhau về tiền. Nhưng chỉ dòng cuối giữ được uy tín với người nhận đầu tiên.

### 5.2. Bảng «khâu → hoạt động kiểm thử lẽ ra chặn»

| Khâu | Defect/failure ở khâu này | Hoạt động kiểm thử | Test condition ví dụ | Kết quả mong đợi |
|---|---|---|---|---|
| Yêu cầu | BR-DLV-02 không nói «mới nhất» nghĩa là gì khi cập nhật đến lệch thứ tự | Review yêu cầu (tĩnh) | «Bản nào thắng nếu v1 đến sau v2?» | Yêu cầu bổ sung: so theo version, bỏ bản cũ hơn |
| Thiết kế sự kiện | Sự kiện đổi địa chỉ không mang version của đơn | Review hợp đồng sự kiện (tĩnh) — kiến trúc §6.6 yêu cầu mã định danh aggregate và quản lý schema | Sự kiện có `order_id` và version không? | Schema có trường version, có quy tắc tương thích |
| Code consumer | Trip service ghi đè không so version | Component test | Áp v2 rồi nhận v1 | Địa chỉ giữ v2; bản tin v1 được ghi log là cũ |
| Tích hợp qua broker | Sự kiện trùng, sự kiện sai thứ tự | Integration test có phát lại sự kiện | Gửi v1, v2, rồi gửi lại v1 | Trạng thái cuối = v2; không tạo thay đổi thừa |
| Luồng hoàn chỉnh | App tài xế hiển thị bản cũ | System test | Sửa địa chỉ khi tài xế đang `arriving` | App tài xế thấy địa chỉ B trong p95 dưới 2–3 giây (§14.1) |
| Nghiệm thu | Quy trình vận hành không có bước xác nhận | Acceptance test cùng đội vận hành | Tài xế giao tới nơi, người nhận không đúng tên | Có quy trình liên hệ người gửi trước khi giao hộ |

Kết quả thực tế: **chưa chạy được** — hệ thống đích chưa có, lab không có địa chỉ và sự kiện. Bảng này là đầu vào cho các bài thực hành ở Series 07 và 11.

### 5.3. Phần làm được ngay trên lab

Ba quy tắc của SPEC lab bảo vệ đúng những hậu quả giống SC-01, và có thể kiểm tra ngay ở các series sau:

| Quy tắc lab | Nếu sai thì hậu quả gần với… |
|---|---|
| R-ACCEPT-03 — một tài xế thắng | Hai tài xế cùng tới lấy một bưu kiện; một người chạy không |
| R-API-05 — khách chỉ xem chuyến của mình | Lộ thông tin chuyến của người khác, như lộ địa chỉ người nhận |
| R-QUOTE-04 — chuyến giữ ảnh chụp giá | Giá đổi hồi tố — cùng kiểu lỗi «bản cũ, bản mới» của SC-01 nhưng với tiền |

## 6. Góc nhìn nâng cao

**Kiểm thử không tự làm sản phẩm tốt lên.** Syllabus nói rõ: kiểm thử góp vào chất lượng **gián tiếp**. Một defect report bị xếp vào backlog vô thời hạn không giảm được đồng thiệt hại nào. Giá trị của kiểm thử chỉ thành hiện thực khi thông tin do kiểm thử tạo ra được sử dụng — để sửa, hoặc để quyết định không phát hành.

**«Cần kiểm thử» không có nghĩa «kiểm thử mọi thứ như nhau».** Không thể test hết mọi tổ hợp ([Bảy nguyên lý kiểm thử](/articles/bay-nguyen-ly-kiem-thu)), và ngân sách kiểm thử có giới hạn. Lập luận ở bài này dùng để quyết định **đầu tư bao nhiêu cho phần nào**: SC-01 đáng một test integration riêng; màu nút «Đặt chuyến» thì không cần cùng mức.

**Con số giả định là công cụ, không phải bằng chứng.** Mô hình 4.2 hữu ích vì nó cho thấy **biến nào chi phối** thiệt hại (N, p, D) và để người đọc thay số thật. Đừng mang con số 187,5 triệu vào cuộc họp như một sự thật; mang công thức và hỏi số thật từ vận hành.

**Chi phí của việc kiểm thử thừa cũng cần tính.** Kiểm thử quá mức một tính năng ít rủi ro làm chậm phát hành và có chi phí cơ hội. Đóng góp 2 (thông tin cho quyết định) bao gồm cả câu trả lời «đủ rồi, có thể phát hành».

**Failure «có lợi cho khách» vẫn là failure.** Ví dụ thu thiếu 360 triệu/ngày ở mục 4.2 không sinh khiếu nại nào. Nếu chỉ dựa vào phản hồi người dùng để tìm lỗi, loại này không bao giờ được phát hiện — một lý do mạnh để có kiểm thử độc lập với khách hàng.

**Kiểm thử là việc của cả đội.** Câu hỏi quyết định ở khâu yêu cầu của SC-01 có thể đến từ BA, developer hay nhân viên vận hành từng gặp chuyện giao nhầm. Syllabus nhấn mạnh bất kỳ bên liên quan nào cũng có thể dùng kỹ năng kiểm thử — xem [Vai trò trong kiểm thử](/articles/vai-tro-trong-kiem-thu).

**Đừng nhầm cơ chế phòng vệ với kiểm thử.** Version check, inbox dedup hay OTP người nhận là **thiết kế** giúp hệ thống chịu được lỗi. Kiểm thử là hoạt động **chứng minh** các cơ chế đó thật sự hoạt động. Cả hai đều cần; cái này không thay cái kia.

## 7. Bài tập và câu hỏi ôn tập

### Bài tập

**Bài 1.** Biến thể đồ ăn SC-02 *(giả định)*: một tài xế gom hai đơn của cùng một quán; app đảo nhãn hai đơn, khách X nhận món của khách Y và ngược lại. Liệt kê các thiệt hại (ít nhất 5, chia trực tiếp/gián tiếp) và với mỗi khâu, chỉ ra hoạt động kiểm thử nào lẽ ra chặn được.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><b>Trực tiếp:</b> hoàn tiền hai đơn (tiền món + phí giao); quán phải làm lại món hoặc Tới Nơi chịu chi phí; tài xế chạy lại; tổng đài xử lý hai khiếu nại; voucher xin lỗi. <b>Gián tiếp:</b> quán mất uy tín dù không có lỗi; khách rời bỏ; <b>nguy cơ sức khoẻ</b> nếu một trong hai khách dị ứng thành phần món của người kia — loại hậu quả syllabus gọi là thương tích.</p>
<p><b>Điểm chặn:</b> review yêu cầu gom đơn (mỗi túi phải mang mã đơn khớp với app?); component test ánh xạ đơn → nhãn khi có 2 đơn cùng quán; system test kịch bản gom đơn cùng quán, cùng giờ; acceptance test với tài xế và quán thật (quy trình dán nhãn). Lưu ý: gom đơn là tính năng của hệ thống đích, chưa có trong lab (R-ACCEPT-06: lab không chủ động điều phối).</p>
</details>

**Bài 2.** Với mô hình 5.1, đội vận hành đề xuất: không cần test sai thứ tự sự kiện vì «có giám sát, lỗi là thấy sau 1 ngày». Tính tổng thiệt hại theo đề xuất này và nêu hai lý do nó vẫn chưa đủ.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>D = 1: 30 đơn × (265.000 + 360.000) đ = <b>18,75 triệu đồng</b> cho một đợt. Chưa đủ vì: (1) 30 người nhận đầu tiên vẫn bị ảnh hưởng, kể cả lộ dữ liệu cá nhân — không đảo ngược được; (2) giám sát chỉ thấy failure có triệu chứng rõ (khiếu nại), còn defect có thể quay lại mỗi lần sửa code consumer — không có regression test thì mỗi lần phát hành lại trả thêm một đợt.</p>
</details>

**Bài 3.** Xếp mỗi việc vào một trong bốn đóng góp của 1.2.1: (a) báo cáo «2 defect cao chưa đóng» gửi buổi họp go/no-go; (b) tester thử đặt bưu kiện bằng điện thoại cũ màn hình nhỏ như nhiều tài xế đang dùng; (c) giữ bằng chứng kiểm thử theo yêu cầu của đối tác cổng thanh toán VíXanh; (d) review SPEC phát hiện R-CANCEL-04 và R-CANCEL-06 cần thống nhất thứ tự ưu tiên lý do miễn phí.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(a) đóng góp 2 — đánh giá chất lượng cho quyết định phát hành; (b) đóng góp 3 — đại diện gián tiếp cho người dùng; (c) đóng góp 4 — yêu cầu hợp đồng; (d) đóng góp 1 — phát hiện defect hiệu quả về chi phí, ở dạng kiểm thử tĩnh (thực tế R-CANCEL-06 đã ghi thứ tự ưu tiên, nên review này xác nhận đặc tả nhất quán).</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Ví dụ nào thể hiện rõ nhất việc kiểm thử đóng góp vào **quyết định phát hành**?

- a) Developer sửa lỗi làm tròn cước sau khi tester báo
- b) Báo cáo kiểm thử cho thấy tính năng sửa địa chỉ chưa được test với sự kiện sai thứ tự, và product owner quyết định chỉ bật ở một thành phố
- c) Tester đóng vai tài xế để đánh giá cách app thông báo đổi địa chỉ
- d) Đội kiểm thử lưu hồ sơ test theo yêu cầu của đối tác thanh toán

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b.</b> Kết quả đánh giá chất lượng được dùng trực tiếp cho quyết định chuyển giai đoạn/phát hành. a là debugging. c là đại diện gián tiếp cho người dùng. d là đáp ứng yêu cầu hợp đồng.</p>
</details>

**Câu 2 (K2).** Phát biểu nào mô tả đúng cách kiểm thử góp phần nâng chất lượng test object theo CTFL v4.0.1?

- a) Kiểm thử trực tiếp loại bỏ defect khỏi test object
- b) Kiểm thử phát hiện defect; defect được gỡ bằng debugging, nhờ đó chất lượng tăng gián tiếp
- c) Kiểm thử chứng minh test object không còn defect
- d) Kiểm thử chỉ nâng chất lượng khi do đội kiểm thử độc lập thực hiện

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b.</b> Syllabus nói kiểm thử phát hiện defect, còn việc gỡ bỏ là debugging — một hoạt động không phải kiểm thử — nên đóng góp là gián tiếp. a nhầm kiểm thử với debugging. c sai: kiểm thử không chứng minh được không còn defect. d sai: mọi bên liên quan đều có thể đóng góp bằng kỹ năng kiểm thử.</p>
</details>

**Câu 3 (K1).** Theo CTFL v4.0.1, phần mềm không hoạt động đúng có thể dẫn tới những hậu quả nào?

- a) Chỉ mất tiền và mất thời gian
- b) Mất tiền, mất thời gian, mất uy tín kinh doanh, và trường hợp nghiêm trọng là thương tích hoặc tử vong
- c) Chỉ ảnh hưởng uy tín vì lỗi luôn sửa được
- d) Không gây hậu quả đáng kể nếu có giám sát vận hành

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b.</b> Mục 1.1 liệt kê đủ các loại hậu quả này. a và c thiếu. d sai: giám sát rút ngắn thời gian failure tồn tại, không ngăn hậu quả của những lần failure đầu.</p>
</details>

**Câu 4 (K2).** Vì sao syllabus coi tester là **đại diện gián tiếp** cho người dùng?

- a) Vì tester luôn hiểu người dùng hơn product owner
- b) Vì đưa một nhóm người dùng đại diện vào suốt dự án thường quá tốn kém và khó tìm đúng người
- c) Vì người dùng không được phép tham gia kiểm thử
- d) Vì acceptance testing do tester thực hiện thay người dùng

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b.</b> Phương án thay thế — đưa người dùng đại diện vào dự án — thường không khả thi vì chi phí cao và thiếu người phù hợp. a là khẳng định quá mức. c sai: người dùng có thể tham gia, ví dụ acceptance testing. d sai: acceptance testing thường có người dùng hoặc đại diện nghiệp vụ tham gia.</p>
</details>

**Câu 5 (K2).** Tình huống nào là ví dụ cho đóng góp «đáp ứng yêu cầu hợp đồng, pháp lý hoặc quy định»?

- a) Đo thời gian phản hồi API báo giá để chọn cấu hình máy chủ
- b) Kiểm thử để chứng minh dữ liệu cá nhân của người nhận bưu kiện không hiển thị trên trang tra cứu công khai, theo cam kết bảo vệ dữ liệu của công ty
- c) Tìm defect trong công thức phụ phí bưu kiện
- d) Báo cáo số test đạt cho buổi họp sprint review

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b.</b> Kiểm thử nhằm đáp ứng nghĩa vụ bảo vệ dữ liệu cá nhân là ví dụ của đóng góp thứ tư. a là đánh giá chất lượng (phi chức năng) phục vụ quyết định kỹ thuật. c là phát hiện defect. d là cung cấp thông tin đánh giá chất lượng.</p>
</details>

**Câu 6 (K2).** Phát biểu nào **không** phải là đóng góp của kiểm thử theo mục 1.2.1?

- a) Phát hiện defect một cách hiệu quả về chi phí
- b) Cung cấp thước đo chất lượng cho quyết định chuyển giai đoạn
- c) Bảo đảm phần mềm phát hành không còn defect
- d) Giúp đáp ứng yêu cầu pháp lý

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c.</b> Kiểm thử không bảo đảm được phần mềm hết defect — nó chỉ giảm khả năng defect còn sót (nguyên lý 1, xem TEST-003). a, b, d đều nằm trong bốn đóng góp của 1.2.1.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus*, v4.0.1 (2024), Chương 1 — mục 1.1 (đoạn mở đầu về hậu quả của phần mềm sai), 1.2 «Why is Testing Necessary?», 1.2.1 «Testing's Contributions to Success» (FL-1.2.1, K2): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — tra quality control, quality assurance, stakeholder, defect, failure; truy cập từ trang chủ https://www.istqb.org/
- Tài liệu kiến trúc tổng thể Tới Nơi: §1.1 nguyên tắc, §4.4 Booking/Order (version), §4.7 Trip/Delivery, §6.3 outbox/inbox, §6.4–6.6 topic, thứ tự, schema, §14 bảng rủi ro, §14.1 SLO.
- Đặc tả lab Tới Nơi (SPEC v1.1): R-ARCH-02, R-GEN-06, R-STATE-02, R-ACCEPT-03, R-ACCEPT-06, R-API-05, R-QUOTE-04 — `examples/toi-noi-testing/SPEC.md`.
- Bài liên quan: [Software Testing là gì? Testing khác Debugging](/articles/software-testing-la-gi-testing-khac-debugging) · [Bảy nguyên lý kiểm thử](/articles/bay-nguyen-ly-kiem-thu) · [Error, Defect, Failure và Root Cause](/articles/error-defect-failure-root-cause) · [Mục tiêu kiểm thử và Quality Assurance](/articles/muc-tieu-kiem-thu-va-quality-assurance) · [Vai trò trong kiểm thử](/articles/vai-tro-trong-kiem-thu) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi).

Bài tiếp theo: [Bảy nguyên lý kiểm thử (Seven Testing Principles)](/articles/bay-nguyen-ly-kiem-thu)
