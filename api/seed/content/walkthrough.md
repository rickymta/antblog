---
slug: walkthrough
title: "Walkthrough: tác giả dẫn cả đội đi qua luồng giao đồ ăn"
summary: "Walkthrough theo CTFL v4.0.1: tác giả dẫn dắt, nhiều mục tiêu (hiểu chung, đào tạo, ý tưởng, tìm anomaly). Chuẩn bị kịch bản cho BA trình bày luồng giao đồ ăn của Tới Nơi với 8 câu hỏi cần đặt."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-024, static-testing, review, walkthrough, giao-do-an]
language: vi
---

BA vừa viết xong luồng giao đồ ăn. Developer Pricing hiểu «tiền món tối thiểu 30.000 đ» là sau voucher, Vận hành tưởng voucher giảm cả phí giao, còn hai bạn mới vào chưa biết đơn đồ ăn đi qua những trạng thái nào. Một buổi **walkthrough** — tác giả dẫn cả nhóm đi qua tài liệu từng bước — giải quyết cả ba chuyện trong 60 phút. Bài này trả lời: walkthrough khác các loại review khác ở đâu, khi nào nên chọn, và chuẩn bị một buổi như thế nào để nó không biến thành buổi thuyết trình một chiều.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-024</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S03 · Static Testing &amp; Review Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Walkthrough (một review type của static testing)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/informal-review">TEST-023 · Informal Review</a> (nên đọc thêm <a href="/articles/review-process-va-cac-vai-tro">TEST-027 · Review Process và các vai trò</a>)</td></tr>
<tr><th scope="row">Thực hành</th><td>Kịch bản walkthrough luồng giao đồ ăn (SPEC R-FOOD-01…06) do BA dẫn: mục tiêu buổi, người tham dự, 8 câu hỏi cần đặt; câu trả lời đã chốt được khoá bằng test vitest ở <code>articles/walkthrough/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 3 · mục 3.2.4 (liên quan 3.2.2, 3.2.5) · FL-3.2.4 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Một tài liệu yêu cầu được nhiều người đọc thì sinh ra nhiều cách hiểu. Với luồng giao đồ ăn của Tới Nơi, ít nhất sáu nhóm phải hiểu giống nhau: BA, developer Pricing (phí giao, voucher), developer Booking/Dispatch (đặt đơn, tài xế nhận), QA, Vận hành quán và Chăm sóc khách hàng. Gửi tài liệu qua chat rồi chờ góp ý (informal review — [TEST-023](/articles/informal-review)) nhanh và rẻ, nhưng không bảo đảm ai cũng đọc, càng không bảo đảm họ hiểu giống nhau.

**Walkthrough** giải quyết đúng chỗ đó: chính **tác giả** dẫn người tham dự đi qua work product theo thứ tự tác giả chọn, mọi người hỏi và nêu vấn đề ngay tại chỗ. Nên dùng khi work product chạm nhiều nhóm và cần **hiểu chung**, khi có **người mới** cần học domain, khi bản nháp còn sớm và tác giả muốn **ý tưởng**, và khi rủi ro ở mức trung bình, không có yêu cầu pháp lý hay audit trail.

Không nên dùng walkthrough làm cách duy nhất khi mục tiêu là tìm **tối đa** defect ở phần rủi ro cao (dùng inspection — TEST-026) hoặc khi cần **quyết định kỹ thuật** có đồng thuận của chuyên gia (technical review — [TEST-025](/articles/technical-review-api-contract)).

## 2. Lý thuyết theo ISTQB

### 2.1. Walkthrough trong CTFL v4.0.1 (mục 3.2.4)

Syllabus mô tả walkthrough bằng ba ý, diễn đạt lại:

1. **Do tác giả dẫn dắt.** Đây là đặc điểm phân biệt nó với technical review (moderator dẫn) và inspection (tác giả không được làm review leader hay scribe).
2. **Phục vụ nhiều mục tiêu**: đánh giá chất lượng và tạo niềm tin vào work product; đào tạo người tham dự; đạt đồng thuận; nảy ý tưởng mới; tạo động lực và giúp tác giả cải thiện; phát hiện anomaly.
3. **Review cá nhân trước buổi là tuỳ chọn.** Người tham dự có thể đọc trước, nhưng syllabus không bắt buộc.

Lưu ý chữ **anomaly** (điểm bất thường): thứ người review nêu ra chưa chắc là defect — có thể là câu hỏi, hiểu lầm, hay gợi ý. Việc phân loại anomaly thành defect diễn ra ở hoạt động communication and analysis (mục 3.2.2).

### 2.2. Walkthrough trên thang hình thức

CTFL nói mức hình thức cần thiết của review phụ thuộc vào SDLC đang theo, độ trưởng thành của quy trình, mức nghiêm trọng và độ phức tạp của work product, yêu cầu pháp lý và nhu cầu audit trail. Cùng một work product có thể qua nhiều loại review — ví dụ informal trước, formal sau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 225" role="img" aria-label="Bốn loại review của CTFL xếp theo mức hình thức tăng dần: informal review, walkthrough, technical review, inspection; mỗi ô ghi ai dẫn và mục tiêu chính, walkthrough được tô đậm.">
<defs>
<marker id="t024-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<line x1="20" y1="24" x2="895" y2="24" class="s-muted" stroke-width="1.4" marker-end="url(#t024-arrow)"/>
<text x="20" y="16" class="t-small">ít hình thức</text>
<text x="895" y="16" text-anchor="end" class="t-small">hình thức nhất</text>
<rect x="20" y="40" width="205" height="170" rx="10" class="f-surface s-line"/>
<text x="122" y="66" text-anchor="middle" class="t-b">Informal review</text>
<text x="122" y="92" text-anchor="middle" class="t-small">không quy trình định sẵn</text>
<text x="122" y="112" text-anchor="middle" class="t-small">không cần biên bản chính thức</text>
<text x="122" y="138" text-anchor="middle" class="t-small">mục tiêu chính:</text>
<text x="122" y="156" text-anchor="middle" class="t-small">phát hiện anomaly</text>
<rect x="245" y="40" width="205" height="170" rx="10" class="f-pk-soft s-pk" stroke-width="2"/>
<text x="347" y="66" text-anchor="middle" class="t-b t-pk">Walkthrough</text>
<text x="347" y="92" text-anchor="middle" class="t-small">TÁC GIẢ dẫn dắt</text>
<text x="347" y="112" text-anchor="middle" class="t-small">đọc trước: tuỳ chọn</text>
<text x="347" y="138" text-anchor="middle" class="t-small">nhiều mục tiêu: hiểu chung,</text>
<text x="347" y="156" text-anchor="middle" class="t-small">đào tạo, đồng thuận, ý tưởng,</text>
<text x="347" y="174" text-anchor="middle" class="t-small">tìm anomaly</text>
<rect x="470" y="40" width="205" height="170" rx="10" class="f-surface s-line"/>
<text x="572" y="66" text-anchor="middle" class="t-b">Technical review</text>
<text x="572" y="92" text-anchor="middle" class="t-small">MODERATOR dẫn dắt</text>
<text x="572" y="112" text-anchor="middle" class="t-small">reviewer có chuyên môn</text>
<text x="572" y="138" text-anchor="middle" class="t-small">mục tiêu chính: đồng thuận,</text>
<text x="572" y="156" text-anchor="middle" class="t-small">quyết định vấn đề kỹ thuật</text>
<rect x="695" y="40" width="205" height="170" rx="10" class="f-surface s-line"/>
<text x="797" y="66" text-anchor="middle" class="t-b">Inspection</text>
<text x="797" y="92" text-anchor="middle" class="t-small">đủ quy trình chung 3.2.2</text>
<text x="797" y="112" text-anchor="middle" class="t-small">tác giả ≠ leader, ≠ scribe</text>
<text x="797" y="138" text-anchor="middle" class="t-small">mục tiêu chính: tìm</text>
<text x="797" y="156" text-anchor="middle" class="t-small">tối đa anomaly, thu metric</text>
</svg>
</div>
<figcaption><b>Hình 1 · Bốn loại review phổ biến của CTFL 3.2.4.</b> Walkthrough đứng thứ hai về mức hình thức. Điểm nhận diện nhanh nhất: <b>ai dẫn buổi review</b>. Tổ chức có thể đặt tên và quy trình riêng cho từng loại.</figcaption>
</figure>

### 2.3. Walkthrough đi qua quy trình review chung như thế nào

Mục 3.2.2 (dựa trên ISO/IEC 20246) mô tả năm hoạt động; review càng hình thức càng làm đủ nhiều nhiệm vụ trong từng hoạt động. Chi tiết từng hoạt động và vai trò: [TEST-027](/articles/review-process-va-cac-vai-tro). Walkthrough thường làm «bản nhẹ»:

| Hoạt động (3.2.2) | Trong một walkthrough điển hình |
|---|---|
| Planning | tác giả chọn phạm vi (một luồng), mục tiêu, người mời, thời lượng |
| Review initiation | gửi tài liệu và kịch bản trước; ai đọc trước thì tốt, không bắt buộc |
| Individual review | tuỳ chọn — khác inspection, nơi đây là bước bắt buộc |
| Communication and analysis | chính là buổi họp: tác giả dẫn, mọi người hỏi; scribe ghi anomaly, câu hỏi, ý tưởng và người nhận |
| Fixing and reporting | tác giả sửa tài liệu; defect thật được ghi lại để theo dõi; gửi biên bản |

Về vai trò (3.2.3): ở walkthrough, **tác giả** thường kiêm luôn review leader và người điều phối buổi họp. Syllabus chỉ cấm tác giả làm review leader hoặc scribe ở **inspection**, nên walkthrough cho phép. Vẫn nên có một **scribe** khác tác giả — người vừa trình bày vừa ghi chép sẽ bỏ sót.

### 2.4. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa trong bài |
|---|---|
| Walkthrough | buổi review do tác giả dẫn người tham dự đi qua work product |
| Formal / informal review | review hình thức / không hình thức (có hay không quy trình và đầu ra được ghi lại) |
| Anomaly | điểm bất thường: điều người review nêu ra, chưa chắc là defect |
| Scribe (recorder) | người ghi: tập hợp anomaly, ghi quyết định trong buổi họp |
| Review leader | người chịu trách nhiệm chung của buổi review: mời ai, khi nào, ở đâu |
| Scenario-based reviewing | kỹ thuật review dựa trên kịch bản sử dụng (CTFL 3.2.2 nêu tên, ISO/IEC 20246 mô tả sâu) |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB · cách làm thường gặp</span>
<p>Không có trong CTFL nhưng phổ biến: <b>dẫn đọc bằng một ví dụ cụ thể</b> («đơn mẫu» đi qua từng bước — gần với scenario-based reviewing); <b>điểm dừng có câu hỏi soạn sẵn</b> do người khác tác giả (thường là QA) chuẩn bị; <b>time-box</b> 45–60 phút cho một luồng — khớp yếu tố «review từng phần nhỏ» của CTFL 3.2.5.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Work product được walkthrough

BA soạn tài liệu «Luồng giao đồ ăn v0.3» gồm bảy bước, dựa trên nhóm R-FOOD của [SPEC lab](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing):

1. Khách chọn quán và món. 2. Báo giá: tiền món, voucher, phí giao. 3. Đặt đơn. 4. Tài xế nhận đơn. 5. Tài xế tới quán, lấy món. 6. Giao cho khách. 7. Hoàn thành, thu tiền.

Business rules mà tài liệu phải phản ánh đúng:

| Mã | Nội dung |
|---|---|
| R-FOOD-01 | Khách trả = (tiền món − tiền giảm voucher) + phí giao |
| R-FOOD-02 | 1–20 dòng món; đơn giá bội số 1.000 đ trong [1.000; 5.000.000]; số lượng [1; 50] |
| R-FOOD-03 | Tiền món < 30.000 đ ⇒ `SUBTOTAL_TOO_LOW`; đúng 30.000 đ hợp lệ |
| R-FOOD-04 | Phí giao theo công thức xe máy của R-FARE trên quãng đường **quán → khách**, có cao điểm, mưa, tối thiểu 15.000 đ, làm tròn lên 1.000 đ; **voucher không trừ vào phí giao** |
| R-FOOD-05 | Voucher tính trên **tiền món**; điều kiện tối thiểu so với tiền món |
| R-FOOD-06 | Chỉ nhận `vehicle` bỏ trống hoặc `bike`; gửi `car` ⇒ `VALIDATION_ERROR` |
| liên quan | R-QUOTE-04 (giá chốt theo báo giá), R-CANCEL-01/03 (phí huỷ 10.000 đ cho mọi đơn đồ ăn), R-STATE-02 (vòng đời chuyến) |

### 3.2. System context

Trong kiến trúc đích, luồng này chạm nhiều service: Customer App → API Gateway/BFF → **Pricing & Quote** (phí giao; quãng đường quán → khách lấy từ MapViet) và **Promotion** (voucher) → **Booking/Order** (kiến trúc §4.4 gợi ý có thể tách Delivery Order khỏi Ride Booking) → sự kiện `BookingCreated` qua Kafka → **Matching & Dispatch** → **Trip/Delivery Lifecycle** → **Payment & Wallet** (VíXanh) và **Notification**.

Lab là bản MVP modular monolith: không Kafka, không có service quán; đơn đồ ăn đi chung vòng đời 8 trạng thái với chuyến xe — `arriving` là tài xế tới quán, `picked_up` là đã lấy món; quãng đường do client gửi (R-GEN-06). Phần ngoài lab (quán xác nhận món) vẫn được nêu trong buổi, ghi chú «chưa có trong lab».

### 3.3. Rủi ro nếu mỗi người hiểu một kiểu

| Hiểu sai có thể xảy ra | Hậu quả (giả định) |
|---|---|
| Ngưỡng 30.000 đ tính **sau** voucher | đơn hợp lệ bị chặn, khách bỏ giỏ hàng |
| Voucher trừ cả phí giao | khách được giảm thêm phần phí giao mà Tới Nơi vẫn phải trả tài xế. Giả định với đơn mẫu: 15% × (50.000 + 27.000) = 11.550 đ thay vì 7.500 đ ⇒ lệch 4.050 đ/đơn; 10.000 đơn ⇒ 40,5 triệu đ |
| Tổng tiền được làm tròn ở app nhưng không ở backend | khách thấy một số, bị thu số khác — khiếu nại |
| Không ai nghĩ tới khách huỷ khi món đã làm | quán chịu thiệt, tranh chấp với Vận hành |
| Hai bạn mới không hiểu vòng đời đơn | viết test sai trạng thái ở sprint sau |

Rủi ro cuối không phải defect của tài liệu — nó là lý do walkthrough có mục tiêu **đào tạo**, thứ informal review không nhắm tới.

## 4. Phân tích kỹ thuật

### 4.1. Vì sao chọn walkthrough cho tài liệu này

CTFL 3.2.4 nói chọn loại review dựa trên mục tiêu, cộng với nhu cầu dự án, nguồn lực, loại work product, rủi ro, domain và văn hoá công ty:

| Yếu tố | Tình trạng tài liệu giao đồ ăn | Gợi ý |
|---|---|---|
| Mục tiêu | hiểu chung giữa 6 nhóm + đào tạo người mới + tìm anomaly sớm | đúng thế mạnh của walkthrough |
| Độ chín của work product | bản nháp v0.3, tác giả còn muốn ý tưởng | chưa cần inspection |
| Mức nghiêm trọng | có tiền nhưng không trực tiếp chạm ví/thanh toán | trung bình |
| Pháp lý, audit trail | không | không cần hình thức cao |
| Phần thanh toán VíXanh | để riêng | technical review hoặc inspection sau ([TEST-025](/articles/technical-review-api-contract), TEST-026) |

### 4.2. Thiết kế buổi: dẫn đọc bằng một đơn mẫu

Thay vì đọc tài liệu từ trên xuống, BA đi **một đơn cụ thể** qua bảy bước: khách C001 ở SG đặt cơm tấm sườn 45.000 đ và trà đá 5.000 đ, quán cách nhà 3,2 km, 09:00 sáng (ngoài cao điểm), trời không mưa, nhập mã `FOOD15`. QA chuẩn bị sẵn tám câu hỏi, gắn vào đúng bước dễ hiểu sai.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 270" role="img" aria-label="Kịch bản walkthrough 60 phút: thanh thời gian gồm mở đầu, dẫn đọc theo đơn mẫu, chốt câu hỏi, hành động; bên dưới là bảy bước của luồng giao đồ ăn, các câu hỏi Q1 đến Q8 gắn vào bước 1, 2, 5 và 7.">
<rect x="20" y="20" width="73" height="30" rx="4" class="f-bg s-line"/>
<text x="56" y="40" text-anchor="middle" class="t-10">mở đầu 5'</text>
<rect x="93" y="20" width="583" height="30" rx="4" class="f-pk-soft s-pk"/>
<text x="384" y="40" text-anchor="middle" class="t-small">BA dẫn đọc theo đơn mẫu — 40'</text>
<rect x="676" y="20" width="146" height="30" rx="4" class="f-warn-soft s-warn"/>
<text x="749" y="40" text-anchor="middle" class="t-small">chốt câu hỏi 10'</text>
<rect x="822" y="20" width="73" height="30" rx="4" class="f-bg s-line"/>
<text x="858" y="40" text-anchor="middle" class="t-10">hành động 5'</text>
<rect x="20" y="80" width="118" height="54" rx="8" class="f-surface s-phys"/>
<text x="79" y="104" text-anchor="middle" class="t-small t-b">1 Chọn món</text>
<rect x="146" y="80" width="118" height="54" rx="8" class="f-surface s-phys"/>
<text x="205" y="104" text-anchor="middle" class="t-small t-b">2 Báo giá</text>
<rect x="272" y="80" width="118" height="54" rx="8" class="f-surface s-line"/>
<text x="331" y="104" text-anchor="middle" class="t-small t-b">3 Đặt đơn</text>
<rect x="398" y="80" width="118" height="54" rx="8" class="f-surface s-line"/>
<text x="457" y="104" text-anchor="middle" class="t-small t-b">4 Tài xế nhận</text>
<rect x="524" y="80" width="118" height="54" rx="8" class="f-surface s-phys"/>
<text x="583" y="100" text-anchor="middle" class="t-small t-b">5 Tới quán,</text>
<text x="583" y="117" text-anchor="middle" class="t-small t-b">lấy món</text>
<rect x="650" y="80" width="118" height="54" rx="8" class="f-surface s-line"/>
<text x="709" y="104" text-anchor="middle" class="t-small t-b">6 Giao</text>
<rect x="776" y="80" width="118" height="54" rx="8" class="f-surface s-phys"/>
<text x="835" y="100" text-anchor="middle" class="t-small t-b">7 Hoàn thành,</text>
<text x="835" y="117" text-anchor="middle" class="t-small t-b">thu tiền</text>
<text x="79" y="126" text-anchor="middle" class="t-10">R-FOOD-02/03</text>
<text x="205" y="126" text-anchor="middle" class="t-10">R-FOOD-04/05/06</text>
<text x="583" y="130" text-anchor="middle" class="t-10">R-CANCEL</text>
<text x="835" y="130" text-anchor="middle" class="t-10">R-FOOD-01</text>
<circle cx="62" cy="168" r="13" class="f-warn-soft s-warn"/><text x="62" y="172" text-anchor="middle" class="t-10 t-b">Q1</text>
<circle cx="96" cy="168" r="13" class="f-warn-soft s-warn"/><text x="96" y="172" text-anchor="middle" class="t-10 t-b">Q2</text>
<circle cx="163" cy="168" r="13" class="f-warn-soft s-warn"/><text x="163" y="172" text-anchor="middle" class="t-10 t-b">Q3</text>
<circle cx="191" cy="168" r="13" class="f-warn-soft s-warn"/><text x="191" y="172" text-anchor="middle" class="t-10 t-b">Q4</text>
<circle cx="219" cy="168" r="13" class="f-warn-soft s-warn"/><text x="219" y="172" text-anchor="middle" class="t-10 t-b">Q5</text>
<circle cx="247" cy="168" r="13" class="f-warn-soft s-warn"/><text x="247" y="172" text-anchor="middle" class="t-10 t-b">Q6</text>
<circle cx="583" cy="168" r="13" class="f-warn-soft s-warn"/><text x="583" y="172" text-anchor="middle" class="t-10 t-b">Q8</text>
<circle cx="835" cy="168" r="13" class="f-warn-soft s-warn"/><text x="835" y="172" text-anchor="middle" class="t-10 t-b">Q7</text>
<text x="331" y="172" text-anchor="middle" class="t-10">đi nhanh</text>
<text x="457" y="172" text-anchor="middle" class="t-10">đi nhanh</text>
<text x="709" y="172" text-anchor="middle" class="t-10">đi nhanh</text>
<rect x="20" y="200" width="874" height="52" rx="8" class="f-bg s-line dash"/>
<text x="457" y="222" text-anchor="middle" class="t-small">Đơn mẫu: C001 · SG · 3,2 km · 09:00 · không mưa · cơm tấm 45.000 đ + trà đá 5.000 đ · FOOD15</text>
<text x="457" y="241" text-anchor="middle" class="t-small">⇒ tiền món 50.000 · giảm 7.500 · phí giao 27.000 · tổng 69.500 đ</text>
</svg>
</div>
<figcaption><b>Hình 2 · Kịch bản walkthrough luồng giao đồ ăn.</b> Câu hỏi dồn vào bước có quy tắc tính tiền; các bước đã quen (đặt đơn, nhận, giao) đi nhanh để giữ thời lượng.</figcaption>
</figure>

## 5. Thực hành

### 5.1. Phiếu chuẩn bị walkthrough

| Mục | Nội dung |
|---|---|
| Work product | «Luồng giao đồ ăn v0.3», bước 1–7 (không gồm phần thanh toán VíXanh) |
| Mục tiêu buổi | (1) sáu nhóm hiểu giống nhau công thức tiền của đơn đồ ăn; (2) hai thành viên mới nắm vòng đời đơn; (3) ghi lại mọi anomaly và câu hỏi mở |
| Tiêu chí kết thúc | mỗi câu Q1–Q8 có câu trả lời hoặc có người nhận; ít nhất một người ngoài BA diễn đạt lại đúng công thức của đơn mẫu |
| Thời lượng | 60 phút, theo Hình 2 |
| Gửi trước | tài liệu + đơn mẫu, trước 1 ngày làm việc; khuyến khích đọc, không bắt buộc |
| Không làm trong buổi | sửa câu chữ, tranh luận cách cài đặt, đánh giá người viết (CTFL 3.2.5: không bao giờ lấy việc đánh giá người tham dự làm mục tiêu) |

Người tham dự (tên vai trò, không phải người thật):

| Người | Vai trò trong review | Lý do mời |
|---|---|---|
| BA phụ trách giao đồ ăn | **tác giả**, review leader, dẫn buổi | walkthrough do tác giả dẫn |
| QA Engineer squad Delivery | **scribe**, soạn câu hỏi | ghi chép độc lập với người trình bày |
| Developer Pricing & Promotion | reviewer | phí giao, voucher |
| Developer Booking/Dispatch | reviewer | đặt đơn, tài xế nhận |
| Đại diện Vận hành quán | reviewer | chuyện huỷ đơn khi món đã làm |
| Đại diện Chăm sóc khách hàng | reviewer | câu khách sẽ hỏi |
| Hai developer mới | reviewer (mục tiêu đào tạo) | học domain |

### 5.2. Tám câu hỏi cần đặt

Câu trả lời ở cột cuối lấy từ SPEC; số liệu là đơn mẫu ở mục 4.2.

| # | Bước | Câu hỏi | Vì sao hỏi | Câu trả lời theo SPEC |
|---|---|---|---|---|
| Q1 | 1 | Tiền món **đúng 30.000 đ** có đặt được không? 29.000 đ thì sao? | biên dễ viết nhầm `<=` | hợp lệ; 29.000 đ ⇒ `SUBTOTAL_TOO_LOW` (R-FOOD-03) |
| Q2 | 1 | Ngưỡng 30.000 đ so với tiền món trước hay sau voucher? | hai nhóm đang hiểu khác nhau | trước voucher: món 30.000 đ dùng `GIAM20` giảm 6.000 đ vẫn hợp lệ |
| Q3 | 2 | Voucher có trừ vào phí giao không? | ảnh hưởng trực tiếp tiền tài xế | không (R-FOOD-04); phí giao đơn mẫu 27.000 đ dù có `FOOD15` |
| Q4 | 2 | Điều kiện tối thiểu của voucher so với tiền món hay với tổng đơn? | dễ lấy «tổng» theo thói quen | tiền món (R-FOOD-05): món 49.000 đ + phí giao 27.000 đ ⇒ `VOUCHER_MIN_AMOUNT` với `FOOD15` (tối thiểu 50.000 đ) |
| Q5 | 2 | Phí giao tính trên quãng đường nào, có cao điểm và mưa không? | người mới hay nghĩ phí giao cố định | quán → khách, công thức xe máy: 12.000 + 4.500 × 3,2 = 26.400 ⇒ làm tròn 27.000 đ; có cao điểm, mưa (R-FOOD-04) |
| Q6 | 2 | Đơn đồ ăn lớn có chọn ô tô được không? | Vận hành hỏi cho tiệc công ty | không: `car` ⇒ `VALIDATION_ERROR` (R-FOOD-06) — ghi nhận thành ý tưởng cho bản sau |
| Q7 | 7 | Tổng 69.500 đ có làm tròn không? Khách trả tiền mặt thì thu bao nhiêu? | app có thể tự làm tròn | SPEC không làm tròn tổng (R-FOOD-01) ⇒ 50.000 − 7.500 + 27.000 = 69.500 đ; **cách thu 500 đ tiền mặt chưa có quy định** ⇒ anomaly |
| Q8 | 5 | Khách huỷ khi tài xế đã tới quán, món đã làm — ai chịu? | quán mất tiền món | SPEC chỉ nói phí huỷ 10.000 đ khi `arriving` (R-CANCEL-01/03); **quán có được bù không — SPEC im lặng** ⇒ anomaly |

Sáu câu đầu đã có câu trả lời trong SPEC — vẫn đáng hỏi vì mục tiêu là **hiểu chung**: nếu developer Pricing trả lời sai Q3 ngay trong buổi, ta vừa chặn được một defect trước khi nó vào code. Q7 và Q8 lộ ra **chỗ trống của chính đặc tả** — loại defect mà chạy test không tìm được, vì không có kết quả mong đợi để so.

### 5.3. Biên bản do scribe ghi

| Mã | Loại | Nội dung | Người nhận | Trạng thái sau buổi |
|---|---|---|---|---|
| WT-01 | anomaly | Bước 2 tài liệu viết «tối thiểu 30.000 đ sau ưu đãi» — mâu thuẫn R-FOOD-03 | BA | defect của tài liệu, sửa ở v0.4 |
| WT-02 | anomaly | Chưa quy định thu tiền mặt với tổng lẻ 500 đ (Q7) | PO + Vận hành | câu hỏi mở, hạn hết sprint |
| WT-03 | anomaly | Không có chính sách bù cho quán khi khách huỷ ở `arriving` (Q8) | PO | câu hỏi mở; ngoài phạm vi lab |
| WT-04 | ý tưởng | App ghi rõ «Mã chỉ giảm tiền món, không giảm phí giao» ở màn báo giá | Designer | đưa vào backlog |
| WT-05 | ý tưởng | Đơn đồ ăn trên 20 suất cho phép ô tô (Q6) | PO | đưa vào backlog, cần đổi R-FOOD-06 |
| WT-06 | đào tạo | Hai developer mới tự vẽ lại vòng đời đơn: `arriving` = tới quán, `picked_up` = đã lấy món | QA | đạt |

WT-01 là **defect** (tài liệu sai so với quy tắc đã chốt); WT-02, WT-03 là anomaly chưa chắc là defect — PO quyết định ở hoạt động communication and analysis, hoặc sau buổi.

### 5.4. Sau buổi: khoá câu trả lời đã chốt bằng test

Walkthrough là static testing, không chạy gì. Nhưng câu trả lời đã thống nhất rất dễ bị «hiểu lại» vài tuần sau. Cách rẻ để giữ chúng: viết mỗi câu đã có đáp án thành một test có tên mang mã câu hỏi và mã yêu cầu. Thư mục [`articles/walkthrough/`](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/walkthrough) có tám test như vậy (Q8 chỉ test phần SPEC đã nói):

```bash
npx vitest run articles/walkthrough --reporter=verbose
```

```text
 ✓ WT-Q1 · R-FOOD-03 · tiền món đúng 30.000 đ hợp lệ, 29.000 đ bị SUBTOTAL_TOO_LOW
 ✓ WT-Q2 · R-FOOD-03 · ngưỡng 30.000 đ so với tiền món TRƯỚC voucher
 ✓ WT-Q3 · R-FOOD-04 · voucher không trừ vào phí giao
 ✓ WT-Q4 · R-FOOD-05 · tối thiểu của voucher so với tiền món, không so với tổng
 ✓ WT-Q5 · R-FOOD-04 · phí giao = công thức xe máy trên quãng đường quán → khách, làm tròn lên 1.000 đ
 ✓ WT-Q6 · R-FOOD-06 · đơn đồ ăn chọn ô tô bị VALIDATION_ERROR
 ✓ WT-Q7 · R-FOOD-01 · tổng không làm tròn: 50.000 − 7.500 + 27.000 = 69.500 đ
 ✓ WT-Q8 (phần SPEC đã nói) · R-CANCEL-01/03 · khách huỷ khi tài xế đang tới quán ⇒ phí 10.000 đ
 Test Files  1 passed (1)
      Tests  8 passed (8)
```

Kết quả thực tế khớp kết quả mong đợi ở bảng 5.2. Tám test xanh không thay được buổi walkthrough: chúng chỉ kiểm điều SPEC đã viết, còn WT-02 và WT-03 vẫn nằm trên bàn PO.

## 6. Góc nhìn nâng cao

### 6.1. Sai lầm thường gặp

| Sai lầm | Vì sao hỏng | Cách chữa |
|---|---|---|
| Walkthrough thành buổi thuyết trình | tác giả nói 55 phút, không ai hỏi ⇒ chỉ còn mục tiêu «đào tạo» | câu hỏi soạn sẵn ở điểm dừng; người chuẩn bị không phải tác giả |
| Tác giả lái né chỗ yếu | tác giả dẫn nên chọn thứ tự; vô thức đi nhanh qua phần mình không chắc | cho scribe/QA chọn đơn mẫu, gồm cả đường lỗi |
| Phạm vi quá lớn | cả tài liệu giao hàng 40 trang trong một buổi | một luồng mỗi buổi (CTFL 3.2.5: review từng phần nhỏ) |
| Không ghi biên bản | câu hỏi mở biến mất sau buổi | scribe ghi anomaly, người nhận, trạng thái |
| Người mới ngại hỏi | «đồng thuận» giả | hỏi thẳng người mới diễn đạt lại công thức |

### 6.2. Giới hạn

- **Thiên kiến của tác giả** là cái giá của «tác giả dẫn». Trên cùng một tài liệu, walkthrough thường tìm được ít defect hơn inspection — nó không nhắm tối đa số defect.
- Walkthrough không thay **thiết kế test**: tám câu hỏi là điểm khởi đầu cho test condition, còn kỹ thuật thiết kế test (giá trị biên cho Q1, bảng quyết định cho Q2–Q4) thuộc Series 04.
- **Walkthrough từ xa, ghi hình rồi để người xem hỏi sau** (cách làm ngành, ngoài syllabus) giữ được mục tiêu đào tạo nhưng mất phần hỏi–đáp tại chỗ — chỗ walkthrough mạnh nhất.

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1.** Chuẩn bị walkthrough cho luồng **giao bưu kiện** (R-PARCEL-01…04). Nêu mục tiêu buổi, một đơn mẫu và năm câu hỏi cần đặt, mỗi câu gắn mã yêu cầu.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Mục tiêu: hiểu chung cách tính phụ phí khối lượng, đào tạo nhân viên Vận hành kho, tìm anomaly. Đơn mẫu: SG, xe máy, 6 km, bưu kiện 6,1 kg, 17:30 (cao điểm), không mưa. Câu hỏi: (1) 0,04 kg có gửi được không? — làm tròn thành 0,0 ⇒ <code>WEIGHT_OUT_OF_RANGE</code> (R-PARCEL-01); (2) đúng 20,0 kg đi xe máy được không? — được, trên 20 kg mới bắt buộc ô tô (R-PARCEL-02); (3) 6,1 kg phụ phí bao nhiêu? — 2.000 × ⌈1,1⌉ = 4.000 đ (R-PARCEL-03); (4) phụ phí có bị nhân hệ số cao điểm không? — có, vì cộng ở bước 1 (R-PARCEL-04); (5) 30,05 kg? — thành 30,1 ⇒ <code>WEIGHT_OUT_OF_RANGE</code>. Câu (4) là câu «hiểu chung» quan trọng nhất vì nó trái trực giác.</p>
</details>

**Bài tập 2.** Xếp mỗi buổi họp vào một loại review: (a) PO gửi user story vào nhóm chat, ai rảnh thì góp ý; (b) BA chiếu luồng huỷ đơn, dẫn cả đội đi từng bước và trả lời câu hỏi, có hai nhân viên CSKH mới; (c) kiến trúc sư làm moderator, ba developer cấp cao thống nhất cách lưu khoá idempotency; (d) review leader không phải tác giả, mọi reviewer đọc trước theo checklist, ghi số defect theo loại.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(a) informal review — không quy trình, không đầu ra bắt buộc. (b) walkthrough — tác giả dẫn, mục tiêu hiểu chung và đào tạo. (c) technical review — moderator dẫn, người có chuyên môn, mục tiêu quyết định kỹ thuật. (d) inspection — đủ quy trình, review cá nhân bắt buộc, thu metric, tác giả không làm leader.</p>
</details>

**Câu 1 (K1).** Theo CTFL v4.0.1, ai dẫn dắt một walkthrough?

- a) Moderator được đào tạo.
- b) Tác giả của work product.
- c) Manager của dự án.
- d) Scribe.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — moderator dẫn technical review. <b>b đúng</b> — walkthrough do tác giả dẫn. <b>c sai</b> — manager quyết định review gì và cấp nguồn lực, không dẫn buổi. <b>d sai</b> — scribe ghi anomaly và quyết định.</p>
</details>

**Câu 2 (K2).** Đặc điểm nào phân biệt walkthrough với inspection?

- a) Walkthrough không bao giờ tìm anomaly.
- b) Ở walkthrough, review cá nhân trước buổi là tuỳ chọn; ở inspection là bước của quy trình đầy đủ.
- c) Walkthrough bắt buộc thu metric để cải tiến quy trình.
- d) Ở walkthrough, tác giả không được làm review leader.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — phát hiện anomaly là một mục tiêu của walkthrough. <b>b đúng</b> — inspection theo đủ quy trình chung, gồm individual review; walkthrough thì tuỳ. <b>c sai</b> — thu metric là đặc điểm của inspection. <b>d sai</b> — đó là ràng buộc của inspection; walkthrough do chính tác giả dẫn.</p>
</details>

**Câu 3 (K2).** Đội Tới Nơi muốn hai developer mới hiểu luồng giao đồ ăn và đồng thời thu thập câu hỏi mở cho PO. Loại review nào phù hợp nhất?

- a) Inspection.
- b) Technical review.
- c) Walkthrough.
- d) Không cần review, chỉ cần chạy test.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a sai</b> — inspection nhắm tìm tối đa anomaly, tốn công, không nhắm đào tạo. <b>b sai</b> — technical review nhắm quyết định vấn đề kỹ thuật với người có chuyên môn. <b>c đúng</b> — đào tạo và đồng thuận là mục tiêu syllabus nêu cho walkthrough. <b>d sai</b> — test không phát hiện được chỗ trống của đặc tả như Q7, Q8.</p>
</details>

**Câu 4 (K2).** Trong walkthrough, người tham dự nêu: «Chưa có quy định quán có được bù khi khách huỷ lúc tài xế đã tới quán». Phát biểu nào đúng?

- a) Đây chắc chắn là defect của code.
- b) Đây là anomaly; cần phân tích để quyết định trạng thái, người nhận và hành động.
- c) Vì walkthrough không hình thức nên không cần ghi lại.
- d) Phải dừng buổi để tác giả sửa tài liệu ngay.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — chưa có code nào liên quan; điều nêu ra là chỗ trống của đặc tả và chưa chắc là defect. <b>b đúng</b> — CTFL 3.2.2: anomaly không nhất thiết là defect, cần phân tích và quyết định trạng thái, ownership, hành động. <b>c sai</b> — walkthrough vẫn có đầu ra; mất ghi chép thì mất câu hỏi mở. <b>d sai</b> — sửa thuộc hoạt động fixing and reporting, sau buổi.</p>
</details>

**Câu 5 (K1).** Điều nào là yếu tố thành công của review theo CTFL v4.0.1?

- a) Dùng kết quả review để đánh giá năng lực người viết.
- b) Review cả tài liệu lớn trong một buổi để tiết kiệm thời gian.
- c) Review từng phần nhỏ để người review không mất tập trung.
- d) Không cho người review thời gian chuẩn bị để thấy phản ứng tự nhiên.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a sai</b> — đánh giá người tham dự không bao giờ được là mục tiêu. <b>b sai</b> — ngược với yếu tố «review từng phần nhỏ». <b>c đúng</b> — yếu tố syllabus nêu ở 3.2.5. <b>d sai</b> — cần cho đủ thời gian chuẩn bị.</p>
</details>

**Câu 6 (K2).** Phát biểu nào về mức hình thức của review là **đúng**?

- a) Mỗi work product chỉ được review bằng đúng một loại review.
- b) Walkthrough luôn hình thức hơn technical review.
- c) Mức hình thức phụ thuộc các yếu tố như mức nghiêm trọng của work product, yêu cầu pháp lý và nhu cầu audit trail.
- d) Review không hình thức là review không có giá trị.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a sai</b> — cùng work product có thể qua informal trước, formal sau. <b>b sai</b> — thường technical review hình thức hơn; tổ chức có thể đặt khác. <b>c đúng</b> — các yếu tố syllabus liệt kê ở 3.2.4. <b>d sai</b> — informal review nhanh, rẻ và vẫn tìm được anomaly.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB® Certified Tester Foundation Level Syllabus **v4.0.1** (2024): mục 3.2.2 Review Process Activities, 3.2.3 Roles and Responsibilities in Reviews, 3.2.4 Review Types, 3.2.5 Success Factors for Reviews; LO FL-3.2.4 (K2). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary (tra *walkthrough*, *review*, *anomaly*, *formal review*, *informal review*) — https://glossary.istqb.org/
- Trang chủ ISTQB: https://www.istqb.org/
- ISO/IEC 20246 — Software and systems engineering: Work product reviews (quy trình review chung và các review technique mà CTFL dựa vào).
- IEEE Std 1028 — Standard for Software Reviews and Audits (mô tả walkthrough như một loại review có quy trình riêng).
- Đặc tả lab Tới Nơi (`SPEC.md`, nhóm R-FOOD, R-CANCEL) và test của bài: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-021 · Static Testing là gì?](/articles/static-testing-la-gi) · [TEST-027 · Review Process và các vai trò](/articles/review-process-va-cac-vai-tro) · [TEST-023 · Informal Review](/articles/informal-review) · [TEST-010 · Testware](/articles/testware-san-pham-kiem-thu)

Bài tiếp theo: [Technical Review: review API contract](/articles/technical-review-api-contract).
