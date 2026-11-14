---
slug: acceptance-testing-tong-quan
title: "Acceptance Testing: UAT, OAT, nghiệm thu hợp đồng, quy định và alpha/beta"
summary: "Sáu dạng acceptance testing của CTFL v4.0.1 — UAT, vận hành, hợp đồng, quy định, alpha, beta: mỗi dạng trả lời câu hỏi gì, ai làm, ai quyết, qua đợt ra mắt Tới Nơi ở Đà Nẵng và bài phân loại 8 hoạt động."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-016, test-level, acceptance-testing, uat, oat, alpha-beta]
language: vi
---

Tới Nơi chuẩn bị mở dịch vụ ở Đà Nẵng. System test đã xong, các defect nghiêm trọng đã sửa. Nhưng câu hỏi lúc này không còn là «hệ thống có đúng đặc tả không», mà là: **khách và tài xế có dùng được không, đội vận hành có chạy được không, có đúng hợp đồng và đúng luật không — và ai có quyền nói "được, đưa lên"?** Bài này phân loại các dạng acceptance testing theo CTFL v4.0.1, chỉ ra ai chịu trách nhiệm từng dạng, rồi luyện phân loại trên tám hoạt động cụ thể của Tới Nơi.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-016</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S02 · Test Levels &amp; Test Types</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1; tham khảo CT-AcT (Acceptance Testing)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Acceptance</td></tr>
<tr><th scope="row">Test Type</th><td>Functional</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/tong-quan-5-test-level-istqb">TEST-011 Tổng quan 5 test level của ISTQB</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Giấy/bảng tính: phân loại 8 hoạt động cho sẵn (khách thử đặt xe, đội vận hành thử khôi phục backup, tài xế dùng bản beta app, kiểm tuân thủ dữ liệu cá nhân…) vào UAT / OAT / hợp đồng–quy định / alpha–beta, có lời giải</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 2 · mục 2.2.1 · FL-2.2.1 (K2); liên quan mục 1.1.1 (FL-1.1.1, K1) và mục 4.5.2–4.5.3 (đi sâu ở TEST-092)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Acceptance testing (kiểm thử chấp nhận, hay «nghiệm thu») là test level cuối trong danh sách năm level của CTFL. Level này giải quyết một vấn đề mà các level trước không giải quyết được: **một hệ thống đúng đặc tả vẫn có thể không dùng được, không vận hành được, hoặc không được phép đưa vào sử dụng**. Vài ví dụ ở Tới Nơi:

- Phí huỷ tính đúng R-CANCEL từng đồng, nhưng tài xế không hiểu vì sao khách huỷ mà mình không được gì — và bỏ app.
- Mọi API đạt p95 dưới 500 ms, nhưng chưa ai từng khôi phục database từ backup; ngày có sự cố mới biết bản backup không dùng được.
- Admin Portal chạy tốt, nhưng thiếu luồng duyệt hai người cho hoàn tiền mà hợp đồng với nhà thầu đã ghi.

Acceptance testing được dùng trước quyết định triển khai — một bản phát hành, một thành phố mới, một ứng dụng giao từ nhà thầu. Trong Agile, acceptance testing còn xuất hiện ở mức từng user story (qua acceptance criteria). Vì trả lời câu hỏi «có đúng thứ người dùng cần không», acceptance testing gắn chặt với **validation** ở [TEST-005](/articles/verification-va-validation).

## 2. Lý thuyết theo ISTQB

### 2.1. Định nghĩa (CTFL v4.0.1, mục 2.2.1)

Theo CTFL v4.0.1, acceptance testing **tập trung vào validation và vào việc chứng minh hệ thống sẵn sàng để triển khai** — tức là hệ thống đáp ứng nhu cầu nghiệp vụ của người dùng. Syllabus nói thêm: lý tưởng nhất, acceptance testing **do chính những người dùng dự kiến thực hiện**. Syllabus liệt kê các dạng chính: user acceptance testing (UAT), operational acceptance testing, contractual acceptance testing, regulatory acceptance testing, alpha testing và beta testing.

Mục 1.1.1 của CTFL cũng có một mục tiêu test liên quan trực tiếp: kiểm tra xem đối tượng test có tuân thủ yêu cầu hợp đồng, pháp lý và quy định hay không — đó chính là phạm vi của hai dạng nghiệm thu hợp đồng và quy định.

### 2.2. Acceptance testing qua năm thuộc tính phân biệt test level

| Thuộc tính | Với acceptance testing | Ví dụ Tới Nơi |
|---|---|---|
| Test object | Hệ thống hoàn chỉnh (hoặc phần sắp phát hành) trong môi trường gần vận hành; cả quy trình nghiệp vụ, quy trình vận hành, tài liệu hướng dẫn | Bản MVP cho Đà Nẵng: app khách, app tài xế, Admin Portal, runbook |
| Test objective | Validation; tạo tin cậy; chứng minh sẵn sàng triển khai | «Tài xế Đà Nẵng tự chạy được một ca làm việc không cần hỏi tổng đài» |
| Test basis | Nhu cầu và quy trình nghiệp vụ, user story + acceptance criteria, quy trình vận hành, hợp đồng, luật và quy định | R-CANCEL, R-UI; kiến trúc §16.4; phụ lục hợp đồng; quy định bảo vệ dữ liệu cá nhân |
| Defect, failure điển hình | Quy trình không khớp cách người dùng làm việc thật; thiếu chức năng nghiệp vụ; không khôi phục được; vi phạm hợp đồng hoặc quy định | Thông báo phí huỷ gây hiểu nhầm; restore quá RTO; log lộ số điện thoại |
| Cách tiếp cận, trách nhiệm | Người dùng, đội vận hành, pháp chế, khách hàng của hợp đồng thực hiện hoặc quyết định; tester hỗ trợ chuẩn bị, dữ liệu, môi trường | Product Owner ký quyết định ra mắt; SRE ký phần vận hành |

### 2.3. Sáu dạng acceptance testing

CTFL v4.0.1 chỉ nêu tên các dạng; phần mô tả dưới đây diễn đạt lại theo ISTQB Glossary và thực hành phổ biến.

| Dạng | Câu hỏi trả lời | Ai thực hiện | Đối chiếu với (test basis) | Ở đâu |
|---|---|---|---|---|
| User acceptance testing (UAT) | Người dùng làm được công việc của họ không? | Người dùng dự kiến: khách, tài xế, nhân viên vận hành nghiệp vụ | Nhu cầu, quy trình nghiệp vụ, user story | Môi trường vận hành thật hoặc mô phỏng |
| Operational acceptance testing (OAT) | Đội vận hành có chạy, giám sát, khôi phục được không? | Đội vận hành / SRE / quản trị hệ thống | Quy trình vận hành, runbook, yêu cầu khôi phục | Môi trường vận hành hoặc staging |
| Contractual acceptance testing | Hệ thống có đạt tiêu chí nghiệm thu ghi trong hợp đồng không? | Bên đặt hàng (thường kèm bên giao) | Điều khoản, phụ lục tiêu chí nghiệm thu | Theo hợp đồng |
| Regulatory acceptance testing | Hệ thống có tuân thủ luật, quy định, chuẩn bắt buộc không? | Pháp chế / tuân thủ, có khi kiểm toán độc lập hoặc cơ quan quản lý | Luật, quy định, chuẩn ngành | Thường là môi trường vận hành |
| Alpha testing | Người ngoài đội phát triển dùng thử thấy gì? | Người dùng tiềm năng, nhân viên ngoài đội dự án | Nhu cầu người dùng, phản hồi | Tại môi trường của bên phát triển |
| Beta testing | Người dùng thật, trong điều kiện thật của họ, thấy gì? | Người dùng thật bên ngoài tổ chức | Nhu cầu người dùng, phản hồi, số đo sử dụng | Tại nơi và trên thiết bị của người dùng |

Một chi tiết của bản v4.0.1: phần ghi chú thay đổi của syllabus đổi cách gọi «contractual and regulatory acceptance testing» thành **hai** tên riêng — contractual acceptance testing và regulatory acceptance testing. Hai dạng có test basis và người quyết định khác nhau, nên đừng gộp làm một.

### 2.4. Acceptance criteria và ATDD — chỉ giới thiệu

CTFL v4.0.1 mục 4.5.2 (FL-4.5.2, K2) bàn cách viết **acceptance criteria** cho user story (dạng kịch bản Given/When/Then hoặc dạng quy tắc), mục 4.5.3 (FL-4.5.3, K3) dùng **ATDD** để sinh test case từ đó. Đây là phần «thực thi» của nghiệm thu; bài này chỉ phân loại. Cách viết tiêu chí, tổ chức UAT và ký nghiệm thu đi sâu ở bài TEST-092 (Series 08 · System Testing & End-to-end Testing).

### 2.5. Bảng thuật ngữ

| Thuật ngữ | Nghĩa tiếng Việt | Ghi chú |
|---|---|---|
| acceptance testing | kiểm thử chấp nhận, nghiệm thu | Test level thứ năm của CTFL v4.0.1 |
| user acceptance testing (UAT) | kiểm thử chấp nhận của người dùng | |
| operational acceptance testing (OAT) | kiểm thử chấp nhận vận hành | Còn gọi production acceptance testing ở một số tổ chức |
| contractual acceptance testing | nghiệm thu theo hợp đồng | |
| regulatory acceptance testing | nghiệm thu theo quy định | |
| alpha testing / beta testing | thử nghiệm alpha / beta | Phân biệt theo nơi thử và người thử |
| acceptance criteria | tiêu chí chấp nhận | CTFL mục 4.5.2 |
| intended users | người dùng dự kiến | Người sẽ dùng hệ thống thật |
| readiness for deployment | sẵn sàng triển khai | Mục tiêu của acceptance testing |

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Bối cảnh: ra mắt MVP tại Đà Nẵng

Theo lộ trình kiến trúc (§18, Phase 1), MVP gồm Identity/Profile, Driver/Partner, Catalog + Pricing, Booking, Dispatch + Availability, Trip/Delivery, Payment, Notification, Location/Tracking và Admin/Public BFF. Giả định của blog: Admin & Operations Portal do một công ty phần mềm bên ngoài phát triển theo hợp đồng; phần còn lại do đội Tới Nơi tự làm.

### 3.2. Stakeholder và thứ họ cần chấp nhận

| Stakeholder | Thứ họ cần chấp nhận | Nguồn |
|---|---|---|
| Khách hàng | Báo giá dễ hiểu, đặt/huỷ chuyến, phí huỷ công bằng | R-UI-01, R-CANCEL-01…06 |
| Tài xế, đối tác giao hàng | Nhận chuyến, các bước tới hoàn thành, huỷ khi cần | R-UI-02, R-ACCEPT, R-STATE |
| Operator (Dispatcher, Support, Finance) | Điều phối lại, tra cứu chuyến, hoàn tiền có duyệt hai người, đối soát | Kiến trúc §11.2 |
| Đội vận hành / SRE | Backup và khôi phục, giám sát, cảnh báo, runbook sự cố, mục tiêu sẵn sàng | §14.1, §15, §16.4; R-NFR-01 |
| Pháp chế / tuân thủ | Bảo vệ dữ liệu cá nhân, không lưu dữ liệu thẻ, nhật ký kiểm toán | §4.9, §4.14, §13.2 |
| Product Owner / ban điều hành | Quyết định ra mắt dựa trên kết quả các dạng nghiệm thu | — |
| Nhà thầu Admin Portal | Được nghiệm thu và thanh toán khi đạt phụ lục tiêu chí | Hợp đồng (giả định) |

### 3.3. Rủi ro nếu bỏ qua từng dạng

| Bỏ qua | Rủi ro (giả định) |
|---|---|
| UAT | Ứng dụng đúng đặc tả nhưng tài xế bỏ app vì quy trình lạ; tổng đài quá tải câu hỏi «sao tôi bị trừ phí» |
| OAT | Sự cố đầu tiên mới biết backup không khôi phục được; cảnh báo không tới ai; API kiểm thử `/api/test/*` vô tình bật trên production (trái R-TEST-01) |
| Nghiệm thu hợp đồng | Thanh toán đợt cuối cho nhà thầu khi còn thiếu maker-checker; tranh chấp vì không có biên bản đối chiếu tiêu chí |
| Nghiệm thu quy định | Lộ số điện thoại qua trang tra cứu công khai; bị xử phạt về dữ liệu cá nhân |
| Alpha / beta | Lỗi chỉ xuất hiện trên máy cũ, mạng yếu, pin yếu của tài xế thật; phát hiện khi đã phát hành cho cả thành phố |

Một ràng buộc quan trọng của kiến trúc (§16.2): **không dùng dữ liệu PII production ở môi trường test**. UAT với người dùng thật vì thế vẫn phải dùng dữ liệu tổng hợp hoặc đã che — trừ beta, nơi người dùng dùng chính tài khoản thật của họ trên hệ thống thật.

## 4. Phân tích kỹ thuật

### 4.1. Hai câu hỏi để phân loại

Sáu dạng không nằm trên một trục. Bốn dạng đầu khác nhau ở **đối chiếu với cái gì**; alpha và beta khác nhau ở **ai thử và thử ở đâu**. Vì thế một hoạt động có thể thuộc hai dạng cùng lúc.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 330" role="img" aria-label="Phân loại acceptance testing bằng hai câu hỏi độc lập: đối chiếu với cái gì (UAT, OAT, hợp đồng, quy định) và ai thử, ở đâu (alpha, beta).">
<defs>
<marker id="t016-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<rect x="20" y="130" width="170" height="70" rx="8" class="f-surface s-line"/>
<text x="105" y="160" text-anchor="middle" class="t-b">Hoạt động</text>
<text x="105" y="180" text-anchor="middle" class="t-small">cần phân loại</text>
<rect x="240" y="40" width="200" height="60" rx="8" class="f-pk-soft s-pk"/>
<text x="340" y="64" text-anchor="middle" class="t-small t-b">Câu hỏi 1</text>
<text x="340" y="84" text-anchor="middle" class="t-small">Đối chiếu với cái gì?</text>
<rect x="240" y="230" width="200" height="60" rx="8" class="f-pk-soft s-pk"/>
<text x="340" y="254" text-anchor="middle" class="t-small t-b">Câu hỏi 2</text>
<text x="340" y="274" text-anchor="middle" class="t-small">Người ngoài đội thử ở đâu?</text>
<line x1="190" y1="155" x2="238" y2="80" class="s-pk" stroke-width="1.6" marker-end="url(#t016-arrow)"/>
<line x1="190" y1="175" x2="238" y2="250" class="s-pk" stroke-width="1.6" marker-end="url(#t016-arrow)"/>
<rect x="500" y="14" width="400" height="34" rx="6" class="f-phys-soft s-phys"/><text x="514" y="36" class="t-small"><tspan class="t-b">UAT</tspan> · nhu cầu, quy trình nghiệp vụ của người dùng</text>
<rect x="500" y="56" width="400" height="34" rx="6" class="f-phys-soft s-phys"/><text x="514" y="78" class="t-small"><tspan class="t-b">OAT</tspan> · vận hành được: backup, khôi phục, giám sát, runbook</text>
<rect x="500" y="98" width="400" height="34" rx="6" class="f-phys-soft s-phys"/><text x="514" y="120" class="t-small"><tspan class="t-b">Hợp đồng</tspan> · tiêu chí nghiệm thu ghi trong hợp đồng</text>
<rect x="500" y="140" width="400" height="34" rx="6" class="f-phys-soft s-phys"/><text x="514" y="162" class="t-small"><tspan class="t-b">Quy định</tspan> · luật, quy định, chuẩn bắt buộc</text>
<line x1="440" y1="70" x2="498" y2="31" class="s-phys" stroke-width="1.3" marker-end="url(#t016-arrow)"/>
<line x1="440" y1="70" x2="498" y2="73" class="s-phys" stroke-width="1.3" marker-end="url(#t016-arrow)"/>
<line x1="440" y1="70" x2="498" y2="115" class="s-phys" stroke-width="1.3" marker-end="url(#t016-arrow)"/>
<line x1="440" y1="70" x2="498" y2="157" class="s-phys" stroke-width="1.3" marker-end="url(#t016-arrow)"/>
<rect x="500" y="212" width="400" height="34" rx="6" class="f-warn-soft s-warn"/><text x="514" y="234" class="t-small"><tspan class="t-b">Alpha</tspan> · tại môi trường do bên phát triển kiểm soát</text>
<rect x="500" y="254" width="400" height="34" rx="6" class="f-warn-soft s-warn"/><text x="514" y="276" class="t-small"><tspan class="t-b">Beta</tspan> · ở nơi và trên thiết bị của chính người dùng</text>
<line x1="440" y1="260" x2="498" y2="229" class="s-warn" stroke-width="1.3" marker-end="url(#t016-arrow)"/>
<line x1="440" y1="260" x2="498" y2="271" class="s-warn" stroke-width="1.3" marker-end="url(#t016-arrow)"/>
<text x="460" y="318" text-anchor="middle" class="t-small">Hai câu hỏi độc lập: tài xế dùng bản beta để chạy chuyến thật là vừa beta testing vừa UAT.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Hai câu hỏi phân loại acceptance testing.</b> Cách trình bày bằng hai câu hỏi là của blog để dễ phân loại, không phải mô hình của ISTQB.</figcaption>
</figure>

Thêm một câu hỏi phụ khi phân vân: **ai có quyền nói «đạt»?** Người dùng ⇒ UAT; trưởng đội vận hành ⇒ OAT; bên đặt hàng trong hợp đồng ⇒ nghiệm thu hợp đồng; pháp chế hoặc cơ quan quản lý ⇒ nghiệm thu quy định.

### 4.2. Tiêu chí mẫu cho từng dạng ở Tới Nơi

| Dạng | Tiêu chí chấp nhận mẫu (giả định của blog) | Nguồn |
|---|---|---|
| UAT | 10 tài xế mới tự hoàn thành chuyến từ nhận tới hoàn thành không cần hướng dẫn; 8/10 khách giải thích lại đúng lý do phí huỷ hiện trên màn hình | R-UI-02, R-CANCEL-06 |
| OAT | Khôi phục PostgreSQL từ backup PITR về thời điểm chọn trước trong thời gian RTO đã chốt; cảnh báo consumer lag tới người trực; `/api/test/*` trả 404 trên bản production | §16.4, §15.1, R-TEST-01 |
| Hợp đồng | Admin Portal đạt toàn bộ tiêu chí phụ lục, gồm hoàn tiền có maker-checker và audit log không sửa/xoá được | §11.2, §13.3 |
| Quy định | Không lưu PAN/CVV; log, trace, dashboard che PII; trang tra cứu công khai không trả số điện thoại hay địa chỉ đầy đủ | §4.9, §13.2, §4.14 |
| Alpha | 15 nhân viên vận hành và 10 tài xế mời tới văn phòng dùng app Driver bản mới trên thiết bị của công ty trong 1 tuần; không còn lỗi chặn | — |
| Beta | 200 tài xế Đà Nẵng dùng bản thử qua kênh phát hành thử của cửa hàng ứng dụng 2 tuần; phản hồi được phân loại, tỉ lệ app bị đóng đột ngột dưới ngưỡng đã thống nhất | — |

Hai lưu ý: các con số là giả định; SLO §14.1 (99,9%/tháng, p95 dưới 500 ms) là **mục tiêu khởi tạo để thảo luận**, nên nếu muốn dùng làm tiêu chí OAT thì phải chốt thành cam kết trước.

### 4.3. Điều kiện trước khi bắt đầu nghiệm thu

Acceptance testing không phải chỗ tìm defect hàng loạt. Trước khi mời người dùng, đội cần: system test (và SIT với VíXanh, MapViet — [TEST-015](/articles/system-integration-testing)) đã đạt exit criteria; không còn defect chặn đã biết; môi trường và dữ liệu tổng hợp sẵn sàng; tiêu chí chấp nhận đã được người quyết định đồng ý **trước** khi thử.

## 5. Thực hành

### 5.1. Bài thực hành: phân loại 8 hoạt động

Phân loại mỗi hoạt động vào UAT, OAT, hợp đồng, quy định, alpha hoặc beta (có thể nhiều hơn một), ghi ai thực hiện, ai quyết định và test basis. Làm trên bảng tính trước khi mở lời giải.

| # | Hoạt động |
|---|---|
| A1 | 20 khách quen được mời thử đặt xe, đặt giao bưu kiện và huỷ chuyến trên staging với tài khoản và dữ liệu tổng hợp |
| A2 | Đội vận hành khôi phục database từ backup vào một môi trường riêng và đo thời gian khôi phục |
| A3 | 200 tài xế Đà Nẵng cài bản beta của app Driver trên điện thoại của mình và dùng khi chạy chuyến thật |
| A4 | Pháp chế cùng tester kiểm tra tuân thủ dữ liệu cá nhân: log che số điện thoại, có thời hạn lưu dữ liệu GPS, trang tra cứu công khai không lộ PII |
| A5 | 10 tài xế được mời tới văn phòng Tới Nơi dùng app Driver bản mới trên máy của công ty, có QA ngồi cạnh ghi nhận |
| A6 | Đối chiếu Admin Portal nhà thầu bàn giao với phụ lục tiêu chí nghiệm thu trước khi thanh toán đợt cuối |
| A7 | Nhân viên Finance thử luồng hoàn tiền có duyệt hai người và đối soát cuối ngày trên staging |
| A8 | Diễn tập runbook «MapViet không trả lời»: bật chế độ suy giảm, kiểm cảnh báo tới người trực, ghi thời gian xử lý |

<details>
<summary><b>Lời giải</b></summary>
<table>
<thead><tr><th>#</th><th>Dạng</th><th>Thực hiện / quyết định</th><th>Test basis</th><th>Lý do</th></tr></thead>
<tbody>
<tr><td>A1</td><td>UAT</td><td>Khách / Product Owner</td><td>User story đặt và huỷ chuyến, R-UI-01, R-CANCEL</td><td>Người dùng dự kiến kiểm nhu cầu của chính họ trong môi trường mô phỏng</td></tr>
<tr><td>A2</td><td>OAT</td><td>SRE / trưởng vận hành</td><td>Runbook khôi phục, RTO/RPO, kiến trúc §16.4</td><td>Câu hỏi là «vận hành được không», không phải nghiệp vụ</td></tr>
<tr><td>A3</td><td>Beta (và UAT)</td><td>Tài xế / Product Owner</td><td>Nhu cầu tài xế, phản hồi, số đo sử dụng</td><td>Người dùng thật, thiết bị và công việc thật của họ; đồng thời họ kiểm nhu cầu nghiệp vụ</td></tr>
<tr><td>A4</td><td>Quy định</td><td>Pháp chế (tester hỗ trợ) / pháp chế</td><td>Quy định bảo vệ dữ liệu cá nhân, kiến trúc §13.2, §4.14</td><td>Đối chiếu với luật và quy định, không với mong muốn người dùng</td></tr>
<tr><td>A5</td><td>Alpha</td><td>Tài xế được mời / Product Owner</td><td>Nhu cầu tài xế, phản hồi</td><td>Người ngoài đội phát triển, nhưng tại môi trường bên phát triển kiểm soát</td></tr>
<tr><td>A6</td><td>Hợp đồng</td><td>Tới Nơi (bên đặt hàng), nhà thầu tham dự / Tới Nơi</td><td>Phụ lục tiêu chí nghiệm thu</td><td>Kết quả quyết định nghĩa vụ thanh toán theo hợp đồng</td></tr>
<tr><td>A7</td><td>UAT</td><td>Finance / trưởng Finance</td><td>Quy trình hoàn tiền và đối soát, kiến trúc §11.2</td><td>Finance là người dùng nghiệp vụ của Admin Portal; nếu luồng duyệt hai người cũng nằm trong phụ lục hợp đồng thì kết quả dùng lại cho A6</td></tr>
<tr><td>A8</td><td>OAT</td><td>SRE / trưởng vận hành</td><td>Runbook sự cố, cảnh báo, kiến trúc §14, §16.4</td><td>Kiểm khả năng xử lý sự cố của đội vận hành</td></tr>
</tbody>
</table>
<p>Tổng: UAT 2 (A1, A7) + A3 kép; OAT 2 (A2, A8); hợp đồng 1 (A6); quy định 1 (A4); alpha 1 (A5); beta 1 (A3). Chỗ hay nhầm: A7 không phải OAT dù làm trên Admin Portal — Finance làm nghiệp vụ, không vận hành hệ thống. A4 có thể kiêm nghiệm thu hợp đồng nếu hợp đồng với VíXanh ràng buộc chuẩn dữ liệu thẻ.</p>
</details>

### 5.2. Ma trận trách nhiệm

T = thực hiện, Q = quyết định «đạt», H = hỗ trợ (chuẩn bị môi trường, dữ liệu, ghi nhận kết quả).

| Stakeholder | UAT | OAT | Hợp đồng | Quy định | Alpha / beta |
|---|---|---|---|---|---|
| Khách, tài xế | T | — | — | — | T |
| Operator (Dispatcher, Support, Finance) | T | H | — | — | T (alpha nội bộ) |
| Đội vận hành / SRE | H | T, Q | — | H | H |
| Pháp chế / tuân thủ | — | — | H | T, Q | H (đồng ý thu thập dữ liệu thử) |
| Product Owner | Q | — | Q (cùng ban điều hành) | — | Q |
| Nhà thầu | — | — | T (trình diễn) | — | — |
| Tester / QA | H | H | H | H | H |

Tester gần như luôn ở cột H. Đó là điểm khác lớn nhất giữa acceptance testing và system testing: ở đây tester **không phải người quyết định**.

## 6. Góc nhìn nâng cao

- **Phân loại theo mục tiêu, không theo công cụ.** Cùng một bộ kịch bản đặt xe có thể dùng cho system test (tester, đối chiếu đặc tả) và UAT (khách, đối chiếu nhu cầu). Khác nhau là ai làm, đối chiếu với gì, ai quyết.
- **Acceptance không nhất thiết là pha cuối.** Trong Agile, mỗi user story có acceptance criteria kiểm trong sprint (ATDD), còn UAT/OAT cấp bản phát hành diễn ra trước khi ra mắt thành phố. Test level không phải thứ tự thời gian cứng.
- **Validation là trọng tâm, nhưng không duy nhất.** Nghiệm thu hợp đồng và quy định đối chiếu với tiêu chí đã viết sẵn — bản chất gần verification. Đừng dùng câu «acceptance là validation» để bỏ qua việc kiểm từng điều khoản.
- **UAT tìm ra nhiều defect là tín hiệu xấu cho các level trước**, không phải thành công của UAT. Nhận xét này là kinh nghiệm ngành, không phải câu chữ của CTFL v4.0.1.
- **Beta có thiên lệch.** Người tự nguyện tham gia thường rành công nghệ hơn số đông; số liệu đo từ beta cần sự đồng ý của người dùng — đây lại là chuyện quy định.
- **Sai lầm thường gặp:** để tester chạy lại kịch bản system test rồi gọi là UAT; ký nghiệm thu khi chưa có tiêu chí viết ra; quên OAT tới ngày ra mắt; gọi bài test nội bộ của đội QA là «alpha»; dùng dữ liệu khách thật để «UAT cho giống».

## 7. Bài tập và câu hỏi ôn tập

### Bài tập

**Bài 1.** Viết ba tiêu chí OAT cho luồng thu tiền VíXanh của Tới Nơi (gợi ý: kiến trúc §8.3 có đối soát định kỳ, §15.2 có chỉ số tỉ lệ thu tiền thất bại).

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(1) Job đối soát cuối ngày chạy được trên môi trường vận hành, phát hiện giao dịch lệch giữa sổ của Tới Nơi và báo cáo VíXanh, và runbook xử lý lệch được đội vận hành diễn tập thành công. (2) Khi tỉ lệ thu tiền thất bại vượt ngưỡng đã thống nhất trong 10 phút, cảnh báo tới người trực (§15.2 có chỉ số «payment authorization/capture/failure rate»). (3) Đổi khoá bí mật dùng để gọi VíXanh (lưu trong Vault/KMS, §13.2) mà không làm gián đoạn thu tiền. Cả ba do đội vận hành thực hiện và quyết định — nên là OAT, không phải UAT.</p>
</details>

**Bài 2.** Nhà thầu Admin Portal nói: «System test của chúng tôi đạt 100%, Tới Nơi phải ký nghiệm thu». Viết ba ý phản biện.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(1) System test của nhà thầu đối chiếu với đặc tả do chính họ diễn giải và do tester của họ thực hiện; nghiệm thu hợp đồng đối chiếu với phụ lục tiêu chí và do bên đặt hàng quyết định. (2) «Đạt 100%» chỉ nói về các test họ đã chọn; cần biết độ bao phủ so với từng tiêu chí của phụ lục (truy vết — <a href="/articles/testware-san-pham-kiem-thu">TEST-010</a>). (3) Một số tiêu chí chỉ người dùng thật kiểm được: Finance phải tự thử luồng hoàn tiền duyệt hai người (UAT, A7) và kết quả đó là đầu vào của quyết định nghiệm thu hợp đồng.</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Theo CTFL v4.0.1, acceptance testing tập trung chủ yếu vào điều gì?

- a) Tìm càng nhiều defect càng tốt trước khi phát hành
- b) Validation và chứng minh hệ thống sẵn sàng để triển khai
- c) Kiểm giao diện giữa hệ thống và các dịch vụ bên ngoài
- d) Kiểm từng component một cách cô lập

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b>, khớp mô tả ở mục 2.2.1. a) không phải trọng tâm syllabus nêu cho level này. c) là system integration testing. d) là component testing.</p>
</details>

**Câu 2 (K2).** Đội SRE của Tới Nơi khôi phục database từ backup và đo thời gian khôi phục so với RTO đã chốt. Đây là dạng nào?

- a) User acceptance testing
- b) Operational acceptance testing
- c) Beta testing
- d) Regulatory acceptance testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b>: đội vận hành kiểm khả năng vận hành, khôi phục. a) sai: không phải người dùng nghiệp vụ kiểm nhu cầu nghiệp vụ. c) sai: không có người dùng ngoài tổ chức. d) sai: đối chiếu với runbook và RTO, không với luật.</p>
</details>

**Câu 3 (K2).** Mười tài xế được mời tới văn phòng Tới Nơi dùng app Driver bản mới trên thiết bị của công ty. Đây là:

- a) Alpha testing
- b) Beta testing
- c) Contractual acceptance testing
- d) Component integration testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a) đúng</b>: người ngoài đội phát triển thử tại môi trường do bên phát triển kiểm soát. b) sai: beta diễn ra ở nơi và trên thiết bị của người dùng. c) sai: không đối chiếu hợp đồng. d) sai: không phải test level kỹ thuật bên trong hệ thống.</p>
</details>

**Câu 4 (K2).** Việc kiểm tra Tới Nơi không lưu số thẻ và mã CVV, đồng thời log che dữ liệu cá nhân, theo quy định hiện hành thuộc dạng nào là chính?

- a) Operational acceptance testing
- b) User acceptance testing
- c) Regulatory acceptance testing
- d) Alpha testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng</b>: đối chiếu với luật, quy định và chuẩn bắt buộc. a) sai: không phải khả năng vận hành. b) sai: người dùng không phải người quyết định ở đây. d) sai: alpha nói về nơi thử và người thử, không về tuân thủ.</p>
</details>

**Câu 5 (K1).** Theo CTFL v4.0.1, lý tưởng nhất acceptance testing nên do ai thực hiện?

- a) Developer viết ra chức năng
- b) Đội test độc lập của bên phát triển
- c) Những người dùng dự kiến của hệ thống
- d) Công cụ tự động trong pipeline CI

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng</b> theo mục 2.2.1. a), b), d) có thể hỗ trợ, nhưng syllabus nêu người dùng dự kiến là người thực hiện lý tưởng.</p>
</details>

**Câu 6 (K2).** Phát biểu nào đúng?

- a) UAT phải luôn thực hiện sau beta testing
- b) Operational acceptance testing do khách hàng cuối thực hiện
- c) Trong CTFL v4.0.1, contractual acceptance testing và regulatory acceptance testing là hai dạng riêng: một đối chiếu tiêu chí hợp đồng, một đối chiếu luật và quy định
- d) Alpha testing là tên gọi khác của component testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng</b>: syllabus liệt kê hai tên riêng (và ghi chú thay đổi của v4.0.1 tách cách gọi gộp cũ). a) sai: không có thứ tự bắt buộc. b) sai: OAT do đội vận hành thực hiện. d) sai: alpha là một dạng acceptance testing với người ngoài đội phát triển.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — mục 2.2 và 2.2.1 Test Levels (FL-2.2.1); mục 1.1.1 Test Objectives (FL-1.1.1); mục 4.5.2 Acceptance Criteria (FL-4.5.2) và 4.5.3 ATDD (FL-4.5.3); phần ghi chú thay đổi của v4.0.1. PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — tra user acceptance testing, operational acceptance testing, contractual/regulatory acceptance testing, alpha testing, beta testing: https://glossary.istqb.org/
- ISTQB, *Certified Tester Acceptance Testing (CT-AcT)* — module chuyên biệt về nghiệm thu; chưa đối chiếu phiên bản, tra tại https://www.istqb.org/
- Tài liệu kiến trúc Tới Nơi (nội bộ blog): §4.9, §4.14, §11.2, §13.2–13.3, §14.1, §15, §16.2, §16.4, §18.
- Phòng lab: `SPEC.md` (R-UI, R-CANCEL, R-NFR, R-TEST-01) — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-011 Tổng quan 5 test level](/articles/tong-quan-5-test-level-istqb) · [TEST-005 Verification và Validation](/articles/verification-va-validation) · [TEST-014 System Testing](/articles/system-testing) · [TEST-015 System Integration Testing](/articles/system-integration-testing) · [TEST-009 Vai trò trong kiểm thử](/articles/vai-tro-trong-kiem-thu) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi). Đi sâu sau này: TEST-092 (acceptance criteria, ATDD, tổ chức UAT, ký nghiệm thu) ở Series 08.

Bài tiếp theo: [TEST-017 · Functional Testing](/articles/functional-testing).
