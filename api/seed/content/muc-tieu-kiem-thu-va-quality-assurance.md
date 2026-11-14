---
slug: muc-tieu-kiem-thu-va-quality-assurance
title: "Mục tiêu kiểm thử và Quality Assurance: đặt mục tiêu chất lượng đo được cho Tới Nơi"
summary: "Chín mục tiêu kiểm thử điển hình của CTFL v4.0.1, vì sao chúng đổi theo ngữ cảnh, và khác biệt giữa testing (hướng sản phẩm) với QA (hướng quy trình) — áp vào mục tiêu chất lượng đo được cho Tới Nơi."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-006, test-objectives, quality-assurance, quality-control, toi-noi]
language: vi
---

«Test kỹ phần thanh toán nhé» là một yêu cầu không ai làm xong được: không biết kỹ tới đâu thì dừng, cũng không biết dừng rồi thì đã chứng minh được gì. Bài này trả lời hai câu hỏi: **test để đạt điều gì** (test objectives) và **testing khác Quality Assurance ở đâu** — rồi biến chúng thành bảng mục tiêu chất lượng đo được cho Tới Nơi.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-006</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S01 · Fundamentals of Software Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/vi-sao-can-kiem-thu">TEST-002</a>, <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Giấy/bảng tính: bảng mục tiêu chất lượng Tới Nơi; 5 mục tiêu kiểm thử đo được cho giao đồ ăn (R-FOOD); phân loại hoạt động QA/testing</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.1.1 · FL-1.1.1 (K1); mục 1.2 và 1.2.2 · FL-1.2.2 (K1). Liên quan: 5.1.3 (FL-5.1.3, K2), 5.3.1 (FL-5.3.1, K1), 2.1.6 (FL-2.1.6, K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

[TEST-001](/articles/software-testing-la-gi-testing-khac-debugging) đã giới thiệu testing là gì và nhắc qua vài mục tiêu. Bài này đi sâu hơn ở hai điểm:

- **Test objective** (mục tiêu kiểm thử) là lý do một hoạt động test tồn tại. Không có mục tiêu thì không có điểm dừng, không chọn được kỹ thuật, không viết được báo cáo có ý nghĩa.
- **Quality Assurance** (QA, đảm bảo chất lượng) thường bị dùng như tên khác của testing — trong tin tuyển dụng, «QA Engineer» hầu như là người làm test. CTFL tách rõ hai khái niệm, và sự tách đó quyết định ai chịu trách nhiệm cho chất lượng.

Dùng khi nào? Đầu mỗi test level, mỗi sprint, mỗi đợt phát hành: mục tiêu được ghi vào test plan (mục 5.1.1) và quyết định exit criteria (mục 5.1.3).

## 2. Lý thuyết theo ISTQB

### 2.1. Chín mục tiêu kiểm thử điển hình (1.1.1)

CTFL v4.0.1 liệt kê chín mục tiêu điển hình (FL-1.1.1, K1). Bảng dưới diễn đạt lại từng mục và đặt vào Tới Nơi:

| # | Mục tiêu (diễn đạt lại) | Ví dụ ở Tới Nơi |
|---|---|---|
| 1 | Đánh giá work product: yêu cầu, user story, thiết kế, code | Review SPEC R-CANCEL trước khi code phí huỷ |
| 2 | Gây ra failure và tìm defect | Chạy hai lệnh nhận đồng thời để tìm lỗi «hai tài xế cùng nhận» (R-ACCEPT-03) |
| 3 | Đảm bảo độ bao phủ cần thiết của test object | Mọi dòng của bảng chuyển trạng thái R-STATE-02 có ít nhất một test |
| 4 | Giảm mức rủi ro chất lượng phần mềm không đạt | Ưu tiên test thanh toán VíXanh trước giao diện tài xế |
| 5 | Xác minh yêu cầu đã đặc tả được đáp ứng | Ví dụ kiểm R-FARE-09 ra 52.000 đ |
| 6 | Xác minh tuân thủ yêu cầu hợp đồng, pháp lý, quy định | Hợp đồng với cổng VíXanh: một khoá idempotency chỉ trừ tiền một lần |
| 7 | Cung cấp thông tin để các bên liên quan ra quyết định | Báo cáo «có nên mở Nha Trang tuần sau không» |
| 8 | Xây dựng niềm tin vào chất lượng test object | Regression xanh trước đợt cao điểm cuối năm |
| 9 | Xác nhận test object đầy đủ, hoạt động như các bên liên quan mong đợi | Tài xế thật dùng thử trang `/driver.html` (validation — [TEST-005](/articles/verification-va-validation)) |

Mục 1 và 2 là lý do kiểm thử **tĩnh** và **động** đều có chỗ. Mục 7 hay bị quên nhất: kết quả test là **thông tin cho người ra quyết định**, không chỉ là danh sách bug.

### 2.2. Mục tiêu đổi theo ngữ cảnh

Syllabus nói mục tiêu kiểm thử thay đổi theo ngữ cảnh: work product được test, test level, rủi ro, SDLC đang theo, và yếu tố kinh doanh (cơ cấu tổ chức, cạnh tranh, thời gian ra thị trường). Cùng là Tới Nơi:

| Ngữ cảnh | Mục tiêu nổi bật |
|---|---|
| Component test hàm cước `fare.ts` | #2 tìm defect, #3 bao phủ, #5 xác minh R-FARE |
| Tích hợp cổng VíXanh | #4 giảm rủi ro trừ tiền hai lần, #6 tuân thủ hợp đồng cổng |
| Acceptance trước khi mở thành phố mới | #7 thông tin cho quyết định, #8 niềm tin, #9 validation |
| Bản vá gấp lỗi tính cước trong giờ cao điểm | #4 giảm rủi ro, #8 niềm tin — chấp nhận bao phủ hẹp vì thời gian ra thị trường |

### 2.3. Testing và Quality Assurance (1.2.2)

FL-1.2.2 (K1) yêu cầu nhớ quan hệ giữa testing và QA:

| | Testing | Quality Assurance |
|---|---|---|
| Hướng tới | **Sản phẩm** (product-oriented) | **Quy trình** (process-oriented) |
| Tính chất | **Khắc phục** (corrective) | **Phòng ngừa** (preventive) |
| Thuộc về | Một hình thức chính của **quality control** (QC) | Quản lý chất lượng — triển khai và cải tiến quy trình |
| Giả định nền | Chất lượng được đánh giá trên sản phẩm cụ thể | Quy trình tốt, làm đúng ⇒ sản phẩm tốt |
| Áp cho | Test object và work product | Cả quy trình phát triển **và** quy trình kiểm thử |
| Ai chịu trách nhiệm | Người làm test (bất kỳ vai trò nào) | **Mọi người** trong dự án |
| Dùng kết quả test để | Sửa defect | Phản hồi quy trình phát triển và kiểm thử đang chạy tốt tới đâu |

Ngoài testing, syllabus kể thêm các hình thức QC khác: **formal methods** (model checking, chứng minh tính đúng), **simulation** và **prototyping**. Nghĩa là testing ⊂ QC, nhưng QC không chỉ có testing.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chi tiết đổi ở bản 4.0.1</span>
<p>Release notes của v4.0.1 ghi: ở mục 1.2.2, chữ «QC» được thay bằng «testing», vì mục này so QA với testing chứ không so với QC. Đề thi theo v4.0.1 hỏi «testing và QA», còn «testing là một hình thức của QC» vẫn đúng (mục 1.2).</p>
</div>

### 2.4. Bảng thuật ngữ Anh – Việt

| Thuật ngữ | Tiếng Việt | Ghi chú |
|---|---|---|
| test objective | mục tiêu kiểm thử | lý do của một hoạt động test |
| quality | chất lượng | mức sản phẩm thoả nhu cầu đã nêu và ngầm định của các bên liên quan (glossary, diễn đạt lại) |
| quality assurance (QA) | đảm bảo chất lượng | hướng quy trình, phòng ngừa |
| quality control (QC) | kiểm soát chất lượng | hướng sản phẩm; testing là hình thức chính |
| exit criteria | tiêu chí kết thúc | điều phải đạt để coi một hoạt động là xong (5.1.3) |
| test metric | chỉ số kiểm thử | số đo tiến độ, chất lượng, bao phủ, defect (5.3.1) |
| retrospective | buổi nhìn lại | cơ chế cải tiến quy trình (2.1.6) |
| Definition of Done | định nghĩa hoàn thành | tên gọi exit criteria trong Agile |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<ul>
<li><b>Quality management ⊃ QA ⊃ QC.</b> Cách xếp lồng này đến từ quản lý chất lượng (họ ISO 9000), không phải định nghĩa của CTFL.</li>
<li><b>Chức danh «QA Engineer».</b> Trong ngành, đây thường là người thiết kế và chạy test. Gọi gì cũng được, miễn hiểu: hoạt động QA đúng nghĩa (cải tiến quy trình) là trách nhiệm chung, không riêng một người.</li>
<li><b>SMART, SLO, error budget.</b> Cách viết mục tiêu đo được (SMART) và SLO/error budget (từ SRE) là công cụ ngành, dùng ở mục 4–5 để biến mục tiêu thành số.</li>
</ul>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ: chất lượng nghĩa là gì với Tới Nơi

Tài liệu kiến trúc Tới Nơi (§1) đặt ra các mục tiêu: không mất đơn, **không tạo giao dịch trùng**, xử lý được retry/timeout, cập nhật trạng thái nhanh, bảo vệ dữ liệu cá nhân, có khả năng kiểm thử. §14.1 đưa ra các con số khởi tạo (để thảo luận, chưa phải cam kết):

| Capability | SLO khởi tạo | Mã trong lab |
|---|---|---|
| Public API availability | 99,9% theo tháng | R-NFR-01 |
| Booking API latency | p95 &lt; 500 ms, không tính thời gian tìm tài xế | R-NFR-02 |
| Quote API latency | p95 &lt; 500 ms | R-NFR-03 |
| Realtime status propagation | p95 &lt; 2–3 giây | R-NFR-04 |
| Payment state convergence | theo SLA của cổng thanh toán, có đối soát | — (lab không có ledger) |

### 3.2. Business rules dùng trong bài

| Mã | Quy tắc |
|---|---|
| R-PAY-04 | Gọi VíXanh hết giờ sau 3 giây, thử lại tối đa 2 lần với **cùng** khoá idempotency |
| R-IDEM-02 | Gửi lại cùng khoá, cùng thân ⇒ trả lại kết quả cũ, không tạo chuyến/thu tiền lại |
| R-QUOTE-03 | Mỗi báo giá đặt được tối đa một chuyến, kể cả hai yêu cầu cùng lúc |
| R-ACCEPT-03 | Nhiều tài xế cùng nhận ⇒ đúng một người thắng |
| R-API-05 | Khách không xem được chuyến của người khác (404 cho cả hai trường hợp) |
| R-FOOD-01…06 | Giao đồ ăn: tiền món, tối thiểu 30.000 đ, phí giao theo công thức xe máy |

### 3.3. System context

Kiến trúc đích: Booking, Matching & Dispatch, Payment & Wallet, Promotion… là microservice riêng; REST cho client qua API Gateway, gRPC nội bộ, Kafka với transactional outbox/inbox, Redis cho lease và cache. Lab là bản MVP modular monolith của lõi Pricing/Booking/Dispatch/Trip/Payment — **không có Kafka, Redis**; các mục tiêu về sự kiện trùng, sai thứ tự chỉ đặt được ở môi trường riêng (Series 11).

### 3.4. Rủi ro

Danh sách lấy từ bảng rủi ro kiến trúc §14: request trùng tạo giao dịch trùng; hai tài xế cùng nhận một chuyến; webhook thanh toán trùng hoặc trễ; event tới sai thứ tự; lộ dữ liệu theo dõi chuyến; cổng thanh toán hoặc bản đồ ngừng. Mỗi rủi ro cần **ít nhất một mục tiêu kiểm thử** trỏ tới nó — nếu không, nó chỉ là danh sách lo lắng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 210" role="img" aria-label="Chuỗi bốn tầng: mục tiêu kinh doanh, mục tiêu chất lượng, mục tiêu kiểm thử, tiêu chí đo được. QA lo quy trình để đạt cả chuỗi; testing đo và báo cáo ở hai tầng cuối.">
<defs>
<marker id="t006-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="10" y="20" width="195" height="90" rx="8" class="f-phys-soft s-phys"/>
<text x="107" y="44" text-anchor="middle" class="t-b t-phys">Mục tiêu kinh doanh</text>
<text x="107" y="68" text-anchor="middle" class="t-small">Khách tin Tới Nơi:</text>
<text x="107" y="88" text-anchor="middle" class="t-small">không bị trừ tiền oan</text>
<rect x="245" y="20" width="195" height="90" rx="8" class="f-phys-soft s-phys"/>
<text x="342" y="44" text-anchor="middle" class="t-b t-phys">Mục tiêu chất lượng</text>
<text x="342" y="68" text-anchor="middle" class="t-small">Không trừ tiền hai lần</text>
<text x="342" y="88" text-anchor="middle" class="t-small">(độ tin cậy)</text>
<rect x="480" y="20" width="195" height="90" rx="8" class="f-pk-soft s-pk"/>
<text x="577" y="44" text-anchor="middle" class="t-b t-pk">Mục tiêu kiểm thử</text>
<text x="577" y="68" text-anchor="middle" class="t-small">Xác minh R-PAY-04, R-IDEM</text>
<text x="577" y="88" text-anchor="middle" class="t-small">giảm rủi ro thanh toán</text>
<rect x="715" y="20" width="195" height="90" rx="8" class="f-pk-soft s-pk"/>
<text x="812" y="44" text-anchor="middle" class="t-b t-pk">Tiêu chí đo được</text>
<text x="812" y="68" text-anchor="middle" class="t-small">5 chế độ lỗi ví đều đạt</text>
<text x="812" y="88" text-anchor="middle" class="t-small">0 defect nặng còn mở</text>
<line x1="205" y1="65" x2="243" y2="65" class="s-muted" stroke-width="1.6" marker-end="url(#t006-a)"/>
<line x1="440" y1="65" x2="478" y2="65" class="s-muted" stroke-width="1.6" marker-end="url(#t006-a)"/>
<line x1="675" y1="65" x2="713" y2="65" class="s-muted" stroke-width="1.6" marker-end="url(#t006-a)"/>
<path d="M480 125 L480 135 L910 135 L910 125" class="f-none s-pk" stroke-width="1.4"/>
<text x="695" y="155" text-anchor="middle" class="t-small t-pk">Testing (QC, hướng sản phẩm): đo, tìm defect, báo cáo</text>
<path d="M10 172 L10 182 L910 182 L910 172" class="f-none s-phys" stroke-width="1.4"/>
<text x="460" y="202" text-anchor="middle" class="t-small t-phys">QA (hướng quy trình, phòng ngừa): quy trình của mọi người để cả chuỗi đạt được — cải tiến từ kết quả test</text>
</svg>
</div>
<figcaption><b>Hình 1 · Từ mục tiêu kinh doanh tới con số đo được.</b> Testing trả lời «sản phẩm này đã đạt chưa». QA trả lời «quy trình của chúng ta có làm ra sản phẩm đạt một cách ổn định không».</figcaption>
</figure>

## 4. Phân tích kỹ thuật

### 4.1. Một mục tiêu kiểm thử tốt trông thế nào

| Thuộc tính | Câu tự hỏi | Ví dụ thiếu | Sửa lại |
|---|---|---|---|
| Gắn rủi ro / bên liên quan | Ai cần kết quả này, để tránh điều gì? | «Test API» | «Giảm rủi ro thu tiền hai lần cho khách dùng VíXanh» |
| Có phạm vi | Test object nào, test level nào? | «Test thanh toán» | «`src/app/payment.ts` + API hoàn thành chuyến, mức component integration» |
| Đo được | Con số hay câu yes/no nào cho biết đạt? | «Test kỹ» | «5/5 chế độ lỗi của ví giả (R-TEST-03) có test đạt» |
| Có hạn | Đạt trước mốc nào? | (không có) | «Trước buổi quyết định phát hành thứ Năm» |
| Dẫn tới quyết định | Không đạt thì làm gì? | (không có) | «Không đạt ⇒ giữ VíXanh ở chế độ tắt, chỉ nhận tiền mặt» |

Ba thuộc tính cuối nối trực tiếp với **exit criteria** (5.1.3): đo độ kỹ lưỡng (bao phủ, số defect chưa xử lý…) hoặc tiêu chí yes/no (đã chạy hết test dự kiến…). Hết thời gian hoặc ngân sách cũng là exit criteria hợp lệ — nếu các bên liên quan đã xem xét và **chấp nhận rủi ro** phát hành mà không test thêm.

### 4.2. Đừng nhầm ba tầng mục tiêu

| Tầng | Ai sở hữu | Ví dụ | Sai lầm khi nhầm |
|---|---|---|---|
| Mục tiêu chất lượng sản phẩm | Product, kiến trúc, vận hành | Booking API p95 &lt; 500 ms (R-NFR-02) | Coi SLO là «việc của tester» |
| Mục tiêu kiểm thử | Người lập kế hoạch test | Cung cấp số đo p95 dưới tải tham chiếu để quyết định phát hành | Đặt «hệ thống đạt 500 ms» làm mục tiêu test — test không làm hệ thống nhanh lên |
| Chỉ số / exit criteria | Đội test cùng các bên liên quan | Có báo cáo đo ≥ 3 lần chạy, ghi cấu hình máy | Đo một lần trên máy dev rồi kết luận |

Testing **đo và báo cáo** chất lượng; bản thân testing không tạo ra chất lượng. Chất lượng do cách cả đội làm việc tạo ra — đó là phạm vi của QA.

### 4.3. Testing và QA trong một sự cố

Giả định: ba sprint liên tiếp, đội tìm 11 defect ở luồng voucher (R-VOUCHER), 7 trong số đó do thứ tự kiểm lỗi R-VOUCHER-08 bị hiểu khác nhau.

| Ai dùng kết quả test | Làm gì | Loại hoạt động |
|---|---|---|
| Developer | Tái hiện, sửa 11 defect | Debugging (không phải testing) |
| Tester | Confirmation test 11 bản sửa, thêm regression | Testing |
| Cả đội trong retrospective | Thấy 7/11 cùng nguồn: đặc tả thứ tự lỗi khó đọc ⇒ thêm bước example mapping cho mọi quy tắc «nhiều lỗi cùng lúc» vào Definition of Ready | QA |
| Test lead | Theo dõi số defect voucher mỗi sprint sau thay đổi | QA dùng metric của testing |

Đây đúng là ý của syllabus: cùng kết quả test, testing dùng để sửa defect, QA dùng để biết quy trình chạy tốt tới đâu.

## 5. Thực hành

### 5.1. Bảng mục tiêu chất lượng Tới Nơi — bản 1 (phạm vi lab)

Bảng tính một dòng một mục tiêu. Các ngưỡng của R-NFR lấy từ §14.1; các ngưỡng khác là **giả định** để minh hoạ, đội thật phải thống nhất với các bên liên quan.

| ID | Mục tiêu chất lượng (bên liên quan) | Mục tiêu kiểm thử (loại 1.1.1) | Đo bằng · ngưỡng | Không đạt thì |
|---|---|---|---|---|
| QO-01 | Không trừ tiền hai lần (khách, tài chính) | #4 giảm rủi ro, #5 xác minh R-PAY-04, R-IDEM-02 | 5/5 chế độ lỗi ví giả có test đạt; 0 defect nặng mở | Tắt VíXanh, chỉ nhận tiền mặt |
| QO-02 | Một báo giá một chuyến, một chuyến một tài xế (vận hành) | #2 tìm defect, #5 xác minh R-QUOTE-03, R-ACCEPT-03 | Test đồng thời đạt trên kho bộ nhớ và PGlite (giả định: 100 lần lặp, 0 lần lệch) | Chặn phát hành |
| QO-03 | Giá đúng như báo (khách) | #3 bao phủ, #5 xác minh R-FARE | 100% mã R-FARE-01…10 có ít nhất một test truy vết | Chặn phát hành |
| QO-04 | Khách không xem được chuyến người khác (khách, pháp chế) | #6 tuân thủ, #4 giảm rủi ro R-API-05 | 0 endpoint trả dữ liệu chuyến cho người không có quyền | Chặn phát hành |
| QO-05 | Đặt chuyến nhanh (khách) | #7 thông tin cho quyết định, R-NFR-02 | Báo cáo p95 `POST /api/trips` so với 500 ms, kèm cấu hình máy đo | PO quyết định, ghi nhận rủi ro |
| QO-06 | Đội vận hành tự tin mở thành phố mới | #8 niềm tin, #9 validation | UAT 1 ngày với Operator, không còn vấn đề mức chặn | Lùi ngày mở |

Kết quả thực tế: bài giấy — chưa đo. Cột «Không đạt thì» là thứ biến bảng thành công cụ ra quyết định; thiếu cột này, bảng chỉ là danh sách mong muốn.

### 5.2. Mục tiêu kiểm thử cho tính năng huỷ chuyến (R-CANCEL) — lời giải mẫu

| # | Mục tiêu đo được | Truy vết | Loại 1.1.1 |
|---|---|---|---|
| 1 | Mọi ô của bảng phí R-CANCEL-01 có test; riêng mốc 2 phút 0 giây và mốc trễ ETA + 10 phút có test hai phía | R-CANCEL-01, -02, -04 | #3, #5 |
| 2 | 4 lý do phí (`searching`, `grace_period`, `driver_late`, `charged`) đều xuất hiện ít nhất một lần, kể cả ca nhiều lý do cùng đúng | R-CANCEL-06 | #3, #5 |
| 3 | Huỷ chuyến VíXanh có phí khi ví hết giờ: thu đúng một lần, `paymentStatus` đúng | R-CANCEL-07, R-PAY-05 | #2, #4 |
| 4 | Review SPEC R-CANCEL cùng đội vận hành trước khi code, ghi lại mọi câu hỏi mở | R-CANCEL | #1, #9 |
| 5 | Báo cáo kết thúc ghi: số điều kiện đạt/tổng, defect còn mở theo mức, rủi ro còn lại — gửi PO trước buổi quyết định | — | #7 |

### 5.3. Phân loại hoạt động — bảng mẫu

| Hoạt động | Testing | QA | Ghi chú |
|---|---|---|---|
| Review SPEC R-FOOD tìm mâu thuẫn | ✔ | | Static testing — vẫn là testing (QC) |
| Chạy regression trước phát hành | ✔ | | |
| Retrospective sau sprint | | ✔ | Cải tiến quy trình (2.1.6) |
| Đặt quy tắc «PR phải kèm test truy vết mã R-…» | | ✔ | Phòng ngừa |
| Sửa defect làm tròn cước | | | Debugging — không phải testing, cũng không phải QA |
| Phân tích nguyên nhân gốc các defect voucher để đổi quy trình | | ✔ | Dùng kết quả test cho quy trình |

## 6. Góc nhìn nâng cao

**Sai lầm thường gặp**

| Sai lầm | Vì sao sai |
|---|---|
| Mục tiêu «tìm hết bug» | Nguyên lý 1: testing không chứng minh được hết defect ([bảy nguyên lý](/articles/bay-nguyen-ly-kiem-thu)). Mục tiêu phải có điểm dừng. |
| Đo tester bằng số bug tìm được | Khuyến khích bug vụn, chia nhỏ báo cáo, và làm hỏng hợp tác. Đo theo rủi ro đã giảm và thông tin đã cung cấp. |
| «Đội QA chịu trách nhiệm chất lượng» | CTFL: QA là trách nhiệm của mọi người. Đội test cung cấp thông tin; chất lượng do cả đội làm ra. |
| Lấy 100% code coverage làm mục tiêu | Coverage là chỉ số bao phủ (#3), không chứng minh đúng hay đủ — code chạy qua chưa chắc được kiểm. |
| Mục tiêu bất biến cả dự án | Mục tiêu đổi theo ngữ cảnh: test level, rủi ro, giai đoạn. Bản vá gấp và bản phát hành lớn có mục tiêu khác nhau. |

**Mục tiêu xung đột.** «Ra thị trường trước đối thủ» và «bao phủ đủ» kéo ngược nhau. Không có công thức; việc của người test là làm rõ đánh đổi bằng số — điều kiện nào chưa test, rủi ro nào còn lại — để người có quyền chấp nhận rủi ro ký tên (5.1.3).

**QC không chỉ có testing.** Ở Tới Nơi: máy trạng thái chuyến R-STATE-02 có thể kiểm bằng model checking (formal methods); tải giờ cao điểm 70 chuyến/giây ước lượng bằng mô phỏng trước khi dựng hạ tầng; màn hình báo giá thử bằng prototype giấy trước khi code. Cả ba đều là QC nhưng không phải testing theo nghĩa hẹp.

**Phạm vi lab.** Mục tiêu về event trùng/sai thứ tự (Kafka), mất cache Redis, webhook thanh toán không đặt được trên lab — lab không có các thành phần này (R-ARCH-02). Ghi chúng vào bảng với cột «Môi trường: riêng (Series 11)» thay vì bỏ đi.

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Mục tiêu kiểm thử cho giao đồ ăn

Viết **5 mục tiêu kiểm thử đo được** cho tính năng giao đồ ăn (R-FOOD-01…06), mỗi mục tiêu ghi truy vết và loại mục tiêu theo 1.1.1.

<details><summary><b>Lời giải mẫu</b></summary><ul><li><b>1.</b> Mọi biên của R-FOOD-02 (1 và 20 dòng; đơn giá 1.000 và 5.000.000 đ; số lượng 1 và 50) và giá trị ngay ngoài biên có test; 100% đạt. Truy vết R-FOOD-02. Loại #3, #5.</li><li><b>2.</b> Tiền món 29.000 đ ⇒ <code>SUBTOTAL_TOO_LOW</code>, 30.000 đ hợp lệ — test cả hai phía. R-FOOD-03. #5.</li><li><b>3.</b> Có ít nhất một test chứng minh voucher không trừ vào phí giao và tổng = tiền món − giảm + phí giao. R-FOOD-01, -04, -05. #2, #5.</li><li><b>4.</b> Gửi <code>vehicle: "car"</code> cho đơn đồ ăn ⇒ <code>VALIDATION_ERROR</code>; không có đường nào tạo đơn đồ ăn đi ô tô. R-FOOD-06. #4.</li><li><b>5.</b> Review R-FOOD cùng Marketing: điều kiện tối thiểu của <code>FOOD15</code> (50.000 đ) so với tiền món hay tổng đơn có đúng ý chiến dịch không — ghi kết quả trước khi viết test. R-FOOD-05. #1, #9.</li></ul></details>

### Bài tập 2 — QA hay testing?

Phân loại: (a) viết test Playwright cho trang khách; (b) thống nhất mẫu báo cáo defect cho cả công ty; (c) inspection tài liệu R-PAY; (d) đào tạo developer viết khoá idempotency đúng; (e) đo p95 Quote API; (f) đưa «review đặc tả có tester» thành bước bắt buộc trước sprint.

<details><summary><b>Lời giải</b></summary><p>Testing: (a), (c) — inspection là static testing, (e). QA: (b), (d), (f) — đều nhằm làm quy trình tạo ra sản phẩm tốt hơn, phòng ngừa thay vì phát hiện. Lưu ý (f): bản thân buổi review là testing; <i>quyết định đưa nó thành bước bắt buộc</i> là QA.</p></details>

### Bài tập 3 — Viết lại mục tiêu mơ hồ

Viết lại cho đo được: (1) «Đảm bảo tài xế nhận chuyến ổn định»; (2) «Test kỹ API».

<details><summary><b>Lời giải mẫu</b></summary><p>(1) «Trước mốc phát hành, chứng minh R-ACCEPT-03: 100 lần chạy hai tài xế nhận đồng thời, mỗi lần đúng một người thắng, người thua nhận <code>TRIP_ALREADY_TAKEN</code> và vẫn <code>online</code>; không đạt ⇒ chặn phát hành» (con số 100 là giả định). (2) «Mỗi mã lỗi trong bảng R-API-06 có ít nhất một test API gây ra đúng mã và đúng HTTP; báo cáo ghi mã nào chưa có test».</p></details>

### Câu hỏi trắc nghiệm

**Câu 1 (K1).** Điều nào **không** phải mục tiêu kiểm thử điển hình theo CTFL v4.0.1?

- a) Đánh giá work product như yêu cầu, user story, thiết kế
- b) Xây dựng niềm tin vào chất lượng test object
- c) Sửa defect trong mã nguồn
- d) Cung cấp thông tin để các bên liên quan ra quyết định

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Có trong danh sách 1.1.1.</li><li>b) Có trong danh sách 1.1.1.</li><li>c) Đúng — sửa defect là debugging, một hoạt động không phải testing (1.1.2, 1.2.1).</li><li>d) Có trong danh sách 1.1.1.</li></ul></details>

**Câu 2 (K1).** Phát biểu nào mô tả đúng Quality Assurance?

- a) Hướng sản phẩm, mang tính khắc phục
- b) Hướng quy trình, mang tính phòng ngừa
- c) Chỉ là trách nhiệm của đội kiểm thử
- d) Một loại dynamic testing

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — đó là testing.</li><li>b) Đúng — mục 1.2.2.</li><li>c) Sai — QA là trách nhiệm của mọi người trong dự án.</li><li>d) Sai — QA không phải hoạt động kiểm thử.</li></ul></details>

**Câu 3 (K1).** Trong QA, kết quả kiểm thử được dùng để làm gì?

- a) Sửa các defect đã tìm thấy
- b) Phản hồi về việc quy trình phát triển và kiểm thử đang vận hành tốt tới đâu
- c) Thay thế exit criteria
- d) Chứng minh phần mềm không còn defect

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — đó là cách testing dùng kết quả.</li><li>b) Đúng — mục 1.2.2.</li><li>c) Sai — không có quan hệ thay thế như vậy.</li><li>d) Sai — trái nguyên lý 1.</li></ul></details>

**Câu 4 (K1).** Theo CTFL v4.0.1, ngoài testing, đâu là một hình thức quality control?

- a) Retrospective
- b) Formal methods như model checking
- c) Đào tạo quy trình cho nhân viên mới
- d) Chuẩn hoá mẫu báo cáo

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — cơ chế cải tiến quy trình, gần QA.</li><li>b) Đúng — mục 1.2.2 kể formal methods, simulation, prototyping.</li><li>c) Sai — phòng ngừa, thuộc QA.</li><li>d) Sai — chuẩn hoá quy trình, thuộc QA.</li></ul></details>

**Câu 5 (K1).** Yếu tố nào thuộc ngữ cảnh làm mục tiêu kiểm thử thay đổi, theo CTFL v4.0.1?

- a) Số test case của dự án trước
- b) Test level, rủi ro, SDLC và yếu tố kinh doanh như thời gian ra thị trường
- c) Chỉ các yêu cầu chức năng
- d) Chức danh của người kiểm thử

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — dữ liệu lịch sử giúp ước lượng, không phải yếu tố ngữ cảnh được nêu.</li><li>b) Đúng — mục 1.1.1.</li><li>c) Sai — quá hẹp; còn rủi ro, test level, kinh doanh.</li><li>d) Sai — không được nêu.</li></ul></details>

**Câu 6 (K2).** Thanh toán của Tới Nơi liên tục có defect liên quan khoá idempotency. Đội quyết định thêm mục «khoá idempotency» vào checklist review code và Definition of Ready. Hoạt động này là gì?

- a) Testing
- b) Quality assurance
- c) Debugging
- d) Acceptance testing

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — không đánh giá một sản phẩm cụ thể; nó đổi cách làm việc.</li><li>b) Đúng — phòng ngừa, hướng quy trình, dùng kết quả test làm phản hồi.</li><li>c) Sai — không tìm hay sửa một defect cụ thể.</li><li>d) Sai — không phải một test level.</li></ul></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 1.1.1 (FL-1.1.1), 1.2 và 1.2.2 (FL-1.2.2), 2.1.6, 5.1.1, 5.1.3, 5.3.1; release notes v4.0.1 — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — mục *quality*, *quality assurance*, *quality control*, *test objective*; tra tại https://www.istqb.org/
- ISO/IEC/IEEE 29119-1 (khái niệm kiểm thử) và 29119-3 (mẫu test plan, báo cáo).
- ISO/IEC 25010 — mô hình đặc tính chất lượng sản phẩm (dùng khi đặt mục tiêu chất lượng).
- Tài liệu kiến trúc Tới Nơi §1, §14, §14.1 và lab `SPEC.md` (R-PAY, R-IDEM, R-QUOTE, R-ACCEPT, R-API-05, R-FOOD, R-NFR) — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-001](/articles/software-testing-la-gi-testing-khac-debugging) · [TEST-002 Vì sao cần kiểm thử](/articles/vi-sao-can-kiem-thu) · [TEST-005 Verification và Validation](/articles/verification-va-validation) · [TEST-009 Vai trò trong kiểm thử](/articles/vai-tro-trong-kiem-thu)

Bài tiếp theo: [TEST-007 · Quy trình kiểm thử (Test Process)](/articles/quy-trinh-kiem-thu-test-process)
