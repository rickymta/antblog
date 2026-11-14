---
slug: software-testing-la-gi-testing-khac-debugging
title: "Software Testing là gì? Testing khác Debugging như thế nào"
summary: "Kiểm thử phần mềm theo CTFL v4.0.1: định nghĩa, test tĩnh và test động, chín mục tiêu kiểm thử điển hình và ranh giới giữa testing với debugging, minh hoạ bằng một lỗi làm tròn cước chuyến xe Tới Nơi."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-001, debugging, test-objectives, static-testing, dynamic-testing, toi-noi]
language: vi
---

Một khách đặt xe máy 7,4 km lúc 11:30, trời mưa, nhập mã GIAM20. Đặc tả nói khách phải trả **52.000 đ**, ứng dụng báo **51.000 đ**. Ai phát hiện ra chênh lệch đó, ai tìm dòng code sai, ai xác nhận đã sửa xong — và mỗi việc gọi là gì? Bài này trả lời hai câu hỏi nền móng của cả chương trình: **kiểm thử phần mềm (software testing) thực chất là gì**, và **vì sao testing với debugging là hai hoạt động khác nhau** dù thường do cùng một nhóm người làm.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-001</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S01 · Fundamentals of Software Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): test phát hiện failure tính cước R-FARE-09 trên bản có lỗi, rồi debugging, confirmation test và regression test — <a href="https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing">examples/toi-noi-testing</a>, thư mục <code>articles/software-testing-la-gi-testing-khac-debugging</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.1 · FL-1.1.1 (K1), FL-1.1.2 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Hỏi mười người mới vào nghề «kiểm thử là gì», phần lớn sẽ trả lời: *mở ứng dụng ra, bấm thử, thấy lỗi thì báo*. Câu trả lời đó không sai, nhưng chỉ đúng một phần nhỏ. Nó bỏ sót ba điều:

- Kiểm thử bắt đầu **trước khi có code chạy được** — đọc đặc tả cước và thấy công thức mâu thuẫn cũng là kiểm thử.
- Kiểm thử không chỉ để **tìm lỗi**; nó còn cung cấp thông tin để người khác ra quyết định (có phát hành bản cập nhật bảng giá hay chưa).
- Kiểm thử **không bao gồm việc sửa lỗi**. Tìm ra dòng code sai và sửa nó là *debugging*, một hoạt động khác.

Phân biệt rõ ngay từ đầu giúp tránh hai kiểu hỗn loạn hay gặp ở đội phát triển: tester tự mở code «sửa giúp» rồi không ai kiểm lại, hoặc developer sửa xong tự đóng phiếu lỗi mà không ai chạy lại test. Ở Tới Nơi, nơi một đồng chênh lệch cước nhân với hàng trăm nghìn chuyến mỗi ngày, cả hai kiểu đều đắt.

Bài này dùng khi bạn cần: hiểu kiểm thử ở mức định nghĩa chuẩn ISTQB, viết được câu trả lời phỏng vấn «testing khác debugging thế nào», hoặc thống nhất cách gọi tên công việc trong nhóm.

## 2. Lý thuyết theo ISTQB

### 2.1. Kiểm thử là gì

Syllabus CTFL v4.0.1 (mục 1.1) mô tả kiểm thử phần mềm là **một tập hợp hoạt động** nhằm hai việc: **phát hiện defect** và **đánh giá chất lượng** của các sản phẩm công việc phần mềm (work product). Sản phẩm đang được kiểm thử gọi là **test object** (đối tượng kiểm thử): có thể là code, nhưng cũng có thể là tài liệu yêu cầu, user story, thiết kế.

Syllabus chỉ ra hai quan niệm sai phổ biến:

| Quan niệm sai | Thực tế theo CTFL v4.0.1 |
|---|---|
| Kiểm thử chỉ là **chạy** phần mềm rồi xem kết quả | Chạy test (test execution) chỉ là một hoạt động. Kiểm thử còn gồm lập kế hoạch, phân tích, thiết kế, chuẩn bị, báo cáo… và phải gắn với vòng đời phát triển (SDLC) |
| Kiểm thử chỉ là **xác minh** đúng đặc tả | Kiểm thử gồm cả **verification** (có đúng yêu cầu đã đặc tả không) lẫn **validation** (có đáp ứng nhu cầu thật của người dùng và các bên liên quan trong môi trường vận hành không) — xem [Verification và Validation](/articles/verification-va-validation) |

Kiểm thử có hai dạng:

- **Dynamic testing** (kiểm thử động): có **thực thi** phần mềm. Ví dụ: gọi `calculateFare` với 7,4 km rồi so kết quả.
- **Static testing** (kiểm thử tĩnh): **không thực thi**. Gồm review (đọc, rà soát tài liệu và code) và static analysis (công cụ phân tích code). Ví dụ: đọc đoạn code làm tròn và thấy nó ngược với R-FARE-08. Series 03 đi sâu phần này.

Hai ý nữa của mục 1.1 hay bị quên: kiểm thử **không chỉ là việc kỹ thuật** — nó cần được lập kế hoạch, ước lượng, theo dõi và điều khiển; và kiểm thử **chủ yếu là hoạt động trí tuệ** — công cụ giúp chạy nhanh hơn, còn quyết định test cái gì, kết quả nào là đúng vẫn cần kiến thức chuyên môn, tư duy phân tích, phản biện và tư duy hệ thống.

### 2.2. Mục tiêu kiểm thử điển hình (FL-1.1.1, K1)

Syllabus liệt kê chín mục tiêu kiểm thử (test objectives) điển hình. Diễn đạt lại, kèm ví dụ ở Tới Nơi:

| # | Mục tiêu (diễn đạt lại) | Ví dụ ở Tới Nơi |
|---:|---|---|
| 1 | Đánh giá các work product như yêu cầu, user story, thiết kế, code | Review SPEC R-FARE-03 xem thứ tự sáu bước có mâu thuẫn không |
| 2 | Gây ra failure và tìm defect | Chạy oracle R-FARE-09, thấy 51.000 đ thay vì 52.000 đ |
| 3 | Bảo đảm độ bao phủ (coverage) cần thiết của test object | Mỗi bước của R-FARE-03 có ít nhất một test chạm tới |
| 4 | Giảm mức rủi ro do chất lượng phần mềm không đạt | Ưu tiên test cước và thu tiền trước giao diện |
| 5 | Xác minh các yêu cầu đã đặc tả được đáp ứng | Mọi mã R-FARE-xx đều có test đạt |
| 6 | Xác minh test object tuân thủ yêu cầu hợp đồng, pháp lý, quy định | Hoá đơn hiện đúng số tiền khách đã trả |
| 7 | Cung cấp thông tin để các bên liên quan ra quyết định có căn cứ | Báo cáo «còn 1 defect cước mở» trước buổi quyết định phát hành |
| 8 | Xây dựng niềm tin vào chất lượng của test object | Bộ regression cước xanh qua 20 lần build liên tiếp |
| 9 | Xác nhận (validate) test object hoàn chỉnh và chạy đúng như các bên liên quan mong đợi | Tài xế thử trang `/driver.html` và hoàn thành được chuyến |

Mục tiêu **thay đổi theo ngữ cảnh**: work product đang test, test level, rủi ro, mô hình SDLC, và các yếu tố kinh doanh như cơ cấu tổ chức, cạnh tranh, thời gian ra thị trường. Kiểm thử trang giới thiệu dịch vụ và kiểm thử Payment & Wallet không thể chung một bộ mục tiêu. Bài [Mục tiêu kiểm thử và Quality Assurance](/articles/muc-tieu-kiem-thu-va-quality-assurance) đi sâu cách chọn mục tiêu theo ngữ cảnh; ở đây chỉ cần nhớ danh sách.

### 2.3. Testing và debugging (FL-1.1.2, K2)

Syllabus nói ngắn gọn: «Testing and debugging are separate activities» (CTFL v4.0.1, mục 1.1.2). Cụ thể:

- **Testing** *gây ra* failure (khi test động) hoặc *tìm thẳng* defect trong test object (khi test tĩnh).
- **Debugging** là việc *tìm nguyên nhân* của failure (tức defect), *phân tích* và *loại bỏ* nó.

Khi kiểm thử động gây ra một failure, quá trình debugging điển hình gồm ba bước:

1. **Reproduction** — tái hiện failure.
2. **Diagnosis** — chẩn đoán, tìm ra defect.
3. **Fixing** — sửa defect.

Sau khi sửa là hai hoạt động **kiểm thử** (không phải debugging):

- **Confirmation testing** (kiểm thử xác nhận): kiểm tra bản sửa đã giải quyết được vấn đề chưa. Syllabus khuyến nghị **nên do chính người đã chạy test ban đầu** thực hiện.
- **Regression testing** (kiểm thử hồi quy): kiểm tra bản sửa có làm hỏng phần khác của test object không. Hai loại này được đào sâu ở Series 02.

Khi **kiểm thử tĩnh** tìm ra defect, debugging chỉ còn việc **gỡ bỏ** nó: không cần tái hiện hay chẩn đoán, vì defect đã nằm ngay trước mắt người review, và kiểm thử tĩnh không gây ra failure.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 330" role="img" aria-label="Luồng xử lý lỗi làm tròn cước. Cột trái là kiểm thử động: 1 tester chạy TC-FARE-01, 2 quan sát failure 51.000 khác 52.000, 3 báo cáo defect; chuyển cho developer: 4 tái hiện, 5 chẩn đoán thấy bước 6 sai, 6 sửa Math.floor thành Math.ceil; trả lại tester: 7 confirmation test, 8 regression test. Cột phải là kiểm thử tĩnh: review code so với R-FARE-08, thấy defect trực tiếp, gỡ defect mà không cần tái hiện hay chẩn đoán.">
<defs>
<marker id="t001-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="20" y="24" class="t-b">Kiểm thử động: testing gây ra failure, debugging tìm và sửa defect</text>
<text x="815" y="24" text-anchor="middle" class="t-b">Kiểm thử tĩnh</text>
<line x1="705" y1="36" x2="705" y2="286" class="s-line dash" stroke-width="1.2"/>
<rect x="20" y="44" width="200" height="52" rx="8" class="f-pk-soft s-pk"/>
<text x="120" y="66" text-anchor="middle" class="t-b">1. Chạy TC-FARE-01</text>
<text x="120" y="84" text-anchor="middle" class="t-small">tester · R-FARE-09</text>
<rect x="250" y="44" width="200" height="52" rx="8" class="f-pk-soft s-pk"/>
<text x="350" y="66" text-anchor="middle" class="t-b">2. Quan sát failure</text>
<text x="350" y="84" text-anchor="middle" class="t-small">51.000 đ ≠ 52.000 đ</text>
<rect x="480" y="44" width="200" height="52" rx="8" class="f-pk-soft s-pk"/>
<text x="580" y="66" text-anchor="middle" class="t-b">3. Báo cáo defect</text>
<text x="580" y="84" text-anchor="middle" class="t-small t-mono">DEF-FARE-001</text>
<rect x="480" y="136" width="200" height="52" rx="8" class="f-warn-soft s-warn"/>
<text x="580" y="158" text-anchor="middle" class="t-b">4. Tái hiện</text>
<text x="580" y="176" text-anchor="middle" class="t-small">cùng đầu vào ⇒ luôn 51.000</text>
<rect x="250" y="136" width="200" height="52" rx="8" class="f-warn-soft s-warn"/>
<text x="350" y="158" text-anchor="middle" class="t-b">5. Chẩn đoán</text>
<text x="350" y="176" text-anchor="middle" class="t-small">bước 1–5 đúng, bước 6 sai</text>
<rect x="20" y="136" width="200" height="52" rx="8" class="f-warn-soft s-warn"/>
<text x="120" y="158" text-anchor="middle" class="t-b">6. Sửa defect</text>
<text x="120" y="176" text-anchor="middle" class="t-small t-mono">Math.floor → Math.ceil</text>
<rect x="20" y="228" width="200" height="52" rx="8" class="f-phys-soft s-phys"/>
<text x="120" y="250" text-anchor="middle" class="t-b">7. Confirmation test</text>
<text x="120" y="268" text-anchor="middle" class="t-small">chạy lại TC-FARE-01</text>
<rect x="250" y="228" width="200" height="52" rx="8" class="f-phys-soft s-phys"/>
<text x="350" y="250" text-anchor="middle" class="t-b">8. Regression test</text>
<text x="350" y="268" text-anchor="middle" class="t-small">TC-FARE-02…04 và bộ cũ</text>
<line x1="220" y1="70" x2="248" y2="70" class="s-muted" stroke-width="1.6" marker-end="url(#t001-arrow)"/>
<line x1="450" y1="70" x2="478" y2="70" class="s-muted" stroke-width="1.6" marker-end="url(#t001-arrow)"/>
<line x1="580" y1="96" x2="580" y2="134" class="s-muted" stroke-width="1.6" marker-end="url(#t001-arrow)"/>
<text x="590" y="120" class="t-small">giao cho developer</text>
<line x1="480" y1="162" x2="452" y2="162" class="s-muted" stroke-width="1.6" marker-end="url(#t001-arrow)"/>
<line x1="250" y1="162" x2="222" y2="162" class="s-muted" stroke-width="1.6" marker-end="url(#t001-arrow)"/>
<line x1="120" y1="188" x2="120" y2="226" class="s-muted" stroke-width="1.6" marker-end="url(#t001-arrow)"/>
<text x="130" y="212" class="t-small">trả lại tester</text>
<line x1="220" y1="254" x2="248" y2="254" class="s-muted" stroke-width="1.6" marker-end="url(#t001-arrow)"/>
<rect x="730" y="44" width="170" height="52" rx="8" class="f-pk-soft s-pk"/>
<text x="815" y="66" text-anchor="middle" class="t-b">Review code</text>
<text x="815" y="84" text-anchor="middle" class="t-small">so với R-FARE-08</text>
<rect x="730" y="136" width="170" height="52" rx="8" class="f-pk-soft s-pk"/>
<text x="815" y="158" text-anchor="middle" class="t-b">Thấy defect trực tiếp</text>
<text x="815" y="176" text-anchor="middle" class="t-small">không có failure nào</text>
<rect x="730" y="228" width="170" height="52" rx="8" class="f-warn-soft s-warn"/>
<text x="815" y="250" text-anchor="middle" class="t-b">Gỡ defect</text>
<text x="815" y="268" text-anchor="middle" class="t-small">bỏ qua tái hiện, chẩn đoán</text>
<line x1="815" y1="96" x2="815" y2="134" class="s-muted" stroke-width="1.6" marker-end="url(#t001-arrow)"/>
<line x1="815" y1="188" x2="815" y2="226" class="s-muted" stroke-width="1.6" marker-end="url(#t001-arrow)"/>
<rect x="20" y="302" width="14" height="14" rx="3" class="f-pk-soft s-pk"/>
<text x="42" y="313" class="t-small">testing (trước khi sửa)</text>
<rect x="220" y="302" width="14" height="14" rx="3" class="f-warn-soft s-warn"/>
<text x="242" y="313" class="t-small">debugging</text>
<rect x="340" y="302" width="14" height="14" rx="3" class="f-phys-soft s-phys"/>
<text x="362" y="313" class="t-small">testing (sau khi sửa)</text>
</svg>
</div>
<figcaption><b>Hình 1 · Testing và debugging trong một vòng xử lý lỗi.</b> Ô hồng và ô xanh là kiểm thử, ô cam là debugging. Kiểm thử tĩnh (cột phải) tìm thẳng defect nên debugging chỉ còn bước gỡ bỏ.</figcaption>
</figure>

### 2.4. Bảng thuật ngữ Anh – Việt

| Thuật ngữ ISTQB | Nghĩa tiếng Việt dùng trong blog | Ghi chú |
|---|---|---|
| testing | kiểm thử | tập hợp hoạt động, không chỉ chạy test |
| test object | đối tượng kiểm thử | work product đang được test |
| work product | sản phẩm công việc | yêu cầu, thiết kế, code, tài liệu… |
| test objective | mục tiêu kiểm thử | lý do thực hiện kiểm thử |
| dynamic testing | kiểm thử động | có thực thi phần mềm |
| static testing | kiểm thử tĩnh | review, static analysis — không thực thi |
| debugging | gỡ lỗi | tìm, phân tích, loại bỏ nguyên nhân failure |
| failure | thất bại (hành vi sai quan sát được) | chi tiết ở [TEST-004](/articles/error-defect-failure-root-cause) |
| defect | khiếm khuyết (lỗi trong work product) | đồng nghĩa fault, bug |
| confirmation testing | kiểm thử xác nhận | kiểm bản sửa đã khắc phục vấn đề |
| regression testing | kiểm thử hồi quy | kiểm bản sửa không làm hỏng chỗ khác |
| verification / validation | xác minh / xác nhận | xem [TEST-005](/articles/verification-va-validation) |

### 2.5. Phạm vi áp dụng

Định nghĩa và ranh giới testing/debugging áp dụng cho **mọi** test level và mọi mô hình phát triển. Chúng là định nghĩa **hoạt động**, không phải chức danh: một developer viết unit test là đang làm testing; một tester đọc log để khoanh vùng lỗi là đang góp phần vào debugging. Ai làm việc gì do tổ chức quyết định — bài [Vai trò trong kiểm thử](/articles/vai-tro-trong-kiem-thu) bàn chuyện đó.

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Trong giao tiếp hằng ngày, «bug» được dùng cho cả defect lẫn failure, và «QA» hay bị dùng như tên gọi khác của testing. Đó là cách nói của ngành, không phải định nghĩa ISTQB. Blog dùng đúng thuật ngữ ISTQB trong phần lý thuyết và chỉ dùng «bug» khi trích lời người dùng. Khác biệt testing và QA được giải thích ở <a href="/articles/muc-tieu-kiem-thu-va-quality-assurance">TEST-006</a>.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Khách xin báo giá trước khi đặt chuyến. Báo giá ghi rõ tổng tiền và có hiệu lực 5 phút (R-QUOTE-01); chuyến tạo từ báo giá giữ nguyên giá đó (R-QUOTE-04). Vì vậy, nếu công thức cước sai, **mọi chuyến đặt từ báo giá đó đều sai theo**.

### 3.2. Business rules liên quan (trích SPEC lab)

| Mã | Quy tắc |
|---|---|
| R-FARE-01 | Xe máy: mở cửa 12.000 đ, 4.500 đ/km, tối thiểu 15.000 đ. Ô tô: 25.000 đ, 11.000 đ/km, tối thiểu 30.000 đ |
| R-FARE-03 | Sáu bước cố định: cước cơ bản → × cao điểm → + mưa → − voucher → nâng lên cước tối thiểu → làm tròn |
| R-FARE-07 | Cước tối thiểu áp **sau** voucher |
| R-FARE-08 | Làm tròn **lên** bội số 1.000 đ: 51.700 ⇒ 52.000; 52.000 giữ nguyên; 52.001 ⇒ 53.000 |
| R-FARE-09 | Oracle: xe máy, 7,4 km, cao điểm, mưa, giảm 20% ⇒ 45.300 ⇒ 56.625 ⇒ 64.625 ⇒ 51.700 ⇒ **52.000 đ** |

R-FARE-09 là một **test oracle** sẵn có: kết quả đúng đã được tính tay trong đặc tả, độc lập với code.

### 3.3. System context

Trong kiến trúc đích, công thức cước thuộc **Pricing & Quote Service** (tài liệu kiến trúc §4.3): tính giá theo policy, tạo quote có `quote_id`, `expires_at`, `pricing_version` và bảng chi tiết, lưu snapshot để không đổi giá hồi tố. Client gọi qua API Gateway/BFF bằng REST; Booking gọi Pricing nội bộ bằng gRPC (§7.1). SLO khởi tạo của Quote API là p95 dưới 500 ms (§14.1).

Bản lab là **modular monolith** kiểu MVP (§18): module Pricing & Quote nằm ở `src/domain/fare.ts`, `pricing.ts` và `src/app/quote.ts`. Lab không có MapViet thật — client gửi thẳng quãng đường (R-GEN-06) — và không có gRPC hay Kafka. Toàn bộ ví dụ của bài chạy trong lab.

### 3.4. Rủi ro nếu công thức sai

- **Thu thiếu**: làm tròn xuống ⇒ mỗi chuyến có thể hụt tới gần 1.000 đ. Bài [Vì sao cần kiểm thử](/articles/vi-sao-can-kiem-thu) tính thử thiệt hại khi nhân lên quy mô thật.
- **Thu thừa**: sai theo chiều ngược lại ⇒ khiếu nại, hoàn tiền, ảnh hưởng uy tín.
- **Lệch giữa các kênh**: nếu app và hoá đơn dùng hai công thức khác nhau, khách thấy hai con số cho cùng một chuyến.
- **Sai hồi tố**: sửa công thức mà không đổi `pricingVersion` thì không biết chuyến cũ được tính theo bản nào (R-QUOTE-02).

## 4. Phân tích kỹ thuật

### 4.1. Ai làm gì trong vụ «51.000 đ»

Áp định nghĩa ở mục 2.3 vào tình huống, ta tách được ranh giới công việc:

| # | Hoạt động | Thuộc | Ai thường làm | Sản phẩm |
|---:|---|---|---|---|
| 1 | Đọc R-FARE-08/09, chọn đầu vào, tính kết quả mong đợi | Testing (phân tích, thiết kế) | Tester | Test case TC-FARE-01…04 |
| 2 | Chạy test trên bản build | Testing (thực thi, động) | Tester | Kết quả thực tế |
| 3 | So sánh mong đợi và thực tế, ghi nhận failure | Testing | Tester | Defect report |
| 4 | Chạy lại đúng đầu vào, thấy lỗi lặp lại | Debugging (tái hiện) | Developer | — |
| 5 | So từng bước trung gian, khoanh vùng tới hàm làm tròn | Debugging (chẩn đoán) | Developer | Vị trí defect |
| 6 | Đổi `Math.floor` thành `Math.ceil` | Debugging (sửa) | Developer | Bản build mới |
| 7 | Chạy lại TC-FARE-01 trên bản mới | Testing (confirmation) | Tester đã chạy lần đầu | Đóng hoặc mở lại defect |
| 8 | Chạy lại các test cước khác và bộ cũ | Testing (regression) | Tester hoặc pipeline | Bằng chứng không hỏng chỗ khác |

Điểm mấu chốt: **tester không cần biết dòng code sai** để làm tốt bước 1–3. Ngược lại, developer không nên tự kết luận ở bước 7 — đó là lý do syllabus khuyến nghị người chạy test ban đầu làm confirmation testing.

### 4.2. Test condition cho bước làm tròn

Từ R-FARE-08, các test condition (điều kiện cần kiểm) cho bước 6:

- TCond-1: số tiền **có phần lẻ** dưới 1.000 đ ⇒ phải tăng lên nghìn kế tiếp.
- TCond-2: số tiền **đã tròn nghìn** ⇒ giữ nguyên.
- TCond-3: số tiền đi qua **cước tối thiểu** (R-FARE-07) rồi mới làm tròn.

Bốn test case của bài phủ cả ba: TC-FARE-01 và TC-FARE-02 rơi vào TCond-1; TC-FARE-04 vào TCond-2; TC-FARE-03 vào TCond-3. Chọn test condition có hệ thống là chủ đề của Series 04; ở đây chỉ cần thấy rằng test case xuất phát từ đặc tả, **không** từ code.

### 4.3. Con đường tĩnh

Giả sử trước khi chạy test, một đồng nghiệp review pull request và đọc dòng `Math.floor(milli / (1000 * MILLI_PER_DONG))` bên cạnh R-FARE-08 «làm tròn **lên**». Defect bị phát hiện **ngay**, không có failure nào xảy ra, không ai cần tái hiện hay chẩn đoán — debugging chỉ còn việc sửa. Đây đúng là trường hợp cuối của mục 1.1.2 và là lý do kiểm thử tĩnh thường rẻ hơn khi làm sớm.

## 5. Thực hành

Mã nguồn: thư mục `examples/toi-noi-testing/articles/software-testing-la-gi-testing-khac-debugging/` của [lab trên GitHub](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing). Bài **không sửa** `src/` của lab; bản có lỗi là một bản chép riêng.

### 5.1. Bản build có lỗi

`buggy-fare.ts` chép nguyên sáu bước của `calculateFare`, chỉ khác hàm làm tròn — developer giả định hiểu «làm tròn tới bội số 1.000 đ» là bỏ phần lẻ:

```ts
/** Defect nằm ở đây: Math.floor thay vì Math.ceil. */
export function roundToThousandDongBuggy(milli: number): number {
  const thousandsOfDong = Math.floor(milli / (1000 * MILLI_PER_DONG));
  return thousandsOfDong * 1000;
}
```

(Lab tính tiền bằng «milli-đồng» — 1/1000 đồng — để các bước trung gian chính xác tuyệt đối, theo R-GEN-01.)

### 5.2. Test case của tester

| ID | Căn cứ | Đầu vào | Mong đợi | Thực tế (bản có lỗi) | Kết quả |
|---|---|---|---:|---:|---|
| TC-FARE-01 | R-FARE-09 | xe máy, 7,4 km, cao điểm, mưa, GIAM20 | 52.000 đ | 51.000 đ | **Fail** |
| TC-FARE-02 | R-FARE-01, R-FARE-08 | xe máy, 3 km, giờ thường, trời khô | 26.000 đ | 25.000 đ | **Fail** |
| TC-FARE-03 | R-FARE-07 | xe máy, 0,5 km, giờ thường, trời khô | 15.000 đ | 15.000 đ | Pass |
| TC-FARE-04 | R-FARE-01 | ô tô, 10 km, giờ thường, trời khô | 135.000 đ | 135.000 đ | Pass |

Kết quả mong đợi được tính tay: TC-FARE-02 là 12.000 + 4.500 × 3 = 25.500 ⇒ làm tròn lên 26.000; TC-FARE-03 là 14.250 < 15.000 ⇒ 15.000; TC-FARE-04 là 25.000 + 110.000 = 135.000.

```ts
// testing.test.ts — việc của tester: so mong đợi (từ SPEC) với thực tế
for (const tc of FARE_TEST_CASES) {
  run(`${tc.id} (${tc.basis}) ${tc.title} ⇒ ${tc.expectedTotal} đ`, () => {
    expect(calculateFareBuggy(tc.input).total).toBe(tc.expectedTotal);
  });
}
```

Chạy lần đầu, đúng như tester thấy (biến `SHOW_RED=1` tắt cơ chế đánh dấu failure đã biết, giải thích ở 5.5):

```bash
cd examples/toi-noi-testing
SHOW_RED=1 npx vitest run articles/software-testing-la-gi-testing-khac-debugging
```

```text
 × Testing · tính cước trên bản build có lỗi > TC-FARE-01 (R-FARE-09) Xe máy 7,4 km, cao điểm, mưa, GIAM20 ⇒ 52000 đ
   → expected 51000 to be 52000 // Object.is equality
 × Testing · tính cước trên bản build có lỗi > TC-FARE-02 (R-FARE-01, R-FARE-08) Xe máy 3 km, giờ thường, trời khô ⇒ 26000 đ
   → expected 25000 to be 26000 // Object.is equality
 ✓ Testing · tính cước trên bản build có lỗi > TC-FARE-03 (R-FARE-07) Xe máy 0,5 km, chạm cước tối thiểu ⇒ 15000 đ
 ✓ Testing · tính cước trên bản build có lỗi > TC-FARE-04 (R-FARE-01) Ô tô 10 km, giờ thường, trời khô ⇒ 135000 đ

 Test Files  1 failed | 1 passed (2)
      Tests  2 failed | 9 passed (11)
```

(Rút gọn từ lần chạy thật trên vitest 3.2.7, Node 22; 9 ca đạt gồm 2 ca của `testing.test.ts` và 7 ca của `debugging.test.ts`.)

Đến đây việc của testing **đã xong**: có failure, có bằng chứng, có căn cứ. Tester ghi báo cáo:

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Defect report rút gọn · DEF-FARE-001</span>
<table>
<tbody>
<tr><th scope="row">Tiêu đề</th><td>Tổng cước bị làm tròn xuống thay vì lên bội số 1.000 đ</td></tr>
<tr><th scope="row">Căn cứ</th><td>R-FARE-08, R-FARE-09</td></tr>
<tr><th scope="row">Bước tái hiện</th><td>Báo giá xe máy, SG, 7,4 km, lúc 11:30 giờ VN, SG đang mưa, voucher GIAM20</td></tr>
<tr><th scope="row">Mong đợi</th><td>52.000 đ</td></tr>
<tr><th scope="row">Thực tế</th><td>51.000 đ. Cùng hiện tượng: xe máy 3 km giờ thường ra 25.000 đ (mong đợi 26.000 đ)</td></tr>
<tr><th scope="row">Không lặp lại khi</th><td>Tổng đã tròn nghìn (ô tô 10 km) hoặc chạm cước tối thiểu (xe máy 0,5 km)</td></tr>
<tr><th scope="row">Mức nghiêm trọng</th><td>Cao — sai tiền trên mọi chuyến có tổng lẻ</td></tr>
</tbody>
</table>
</div>

Báo cáo không chứa câu nào kiểu «sửa dòng 23 thành `Math.ceil`». Tester mô tả **hành vi**, không chỉ định **cách sửa**. Mẫu báo cáo đầy đủ ở bài [Testware](/articles/testware-san-pham-kiem-thu).

### 5.3. Debugging của developer

```ts
// debugging.test.ts — bước 2: so từng bước trung gian với oracle R-FARE-09
it('bước 1–5 khớp SPEC, chỉ total lệch ⇒ defect nằm ở bước 6 (làm tròn)', () => {
  const b = calculateFareBuggy(tc01.input);
  expect(b).toMatchObject({ base: 45_300, afterPeak: 56_625, beforeDiscount: 64_625, discount: 12_925, afterDiscount: 51_700 });
  expect(b.total).toBe(51_000); // SPEC: 52.000
});

it('cô lập hàm làm tròn bằng ví dụ của R-FARE-08', () => {
  const inputs = [51_700, 52_000, 52_001].map(toMilli);
  expect(inputs.map(roundToThousandDongBuggy)).toEqual([51_000, 52_000, 52_000]); // bản có lỗi
  expect(inputs.map(roundUpToThousandDong)).toEqual([52_000, 52_000, 53_000]);      // đúng SPEC
});
```

Ba bước debugging tương ứng:

1. **Tái hiện**: gọi ba lần với cùng đầu vào, luôn ra 51.000 ⇒ lỗi tất định, không phụ thuộc thời gian hay dữ liệu ngoài.
2. **Chẩn đoán**: bảng chi tiết cước (R-FARE-10) cho thấy bước 1–5 khớp oracle tới từng đồng; lệch xuất hiện ở bước 6. Cô lập hàm làm tròn bằng ba ví dụ của R-FARE-08 ⇒ defect là `Math.floor`.
3. **Sửa**: đổi thành `Math.ceil`. Trong lab, bản đã sửa chính là `roundUpToThousandDong` ở `src/domain/money.ts`.

Bảng chi tiết cước là một **testability hook**: không có nó, developer chỉ thấy tổng sai và phải đoán sai ở bước nào trong sáu bước.

### 5.4. Confirmation test và regression test

```ts
it.each(FARE_TEST_CASES)('$id chạy lại trên bản đã sửa ⇒ $expectedTotal đ', (tc) => {
  expect(calculateFare(tc.input).total).toBe(tc.expectedTotal);
});
```

TC-FARE-01 chạy lại là **confirmation test**; TC-FARE-02…04 cùng toàn bộ test cũ của lab (`npm test`) là **regression test**. Kết quả chạy thật:

```bash
npx vitest run articles/software-testing-la-gi-testing-khac-debugging --reporter=verbose
```

```text
 ✓ Debugging · bước 1 — tái hiện failure > cùng đầu vào TC-FARE-01, bản có lỗi luôn trả 51.000 đ …
 ✓ Debugging · bước 2 — chẩn đoán … > bước 1–5 khớp SPEC, chỉ total lệch ⇒ defect nằm ở bước 6 (làm tròn)
 ✓ Debugging · bước 2 — chẩn đoán … > cô lập hàm làm tròn bằng ví dụ của R-FARE-08 …
 ✓ Sau khi sửa · confirmation test + regression test (tester) > 'TC-FARE-01' … ⇒ 52000 đ
 ✓ Sau khi sửa · confirmation test + regression test (tester) > 'TC-FARE-02' … ⇒ 26000 đ
 ✓ Sau khi sửa · confirmation test + regression test (tester) > 'TC-FARE-03' … ⇒ 15000 đ
 ✓ Sau khi sửa · confirmation test + regression test (tester) > 'TC-FARE-04' … ⇒ 135000 đ
 ✓ Testing · tính cước trên bản build có lỗi > TC-FARE-01 … TC-FARE-04 (4 ca, 2 ca it.fails)

 Test Files  2 passed (2)
      Tests  11 passed (11)
```

(Rút gọn tên test từ lần chạy thật.)

### 5.5. Vì sao lần chạy mặc định xanh

Hai ca TC-FARE-01, TC-FARE-02 trên bản có lỗi được đánh dấu `it.fails` — vitest coi chúng **đạt khi test bên trong thất bại**. Đó là cách ghi lại «failure đã biết, đã báo cáo» để `npm test` của cả lab không đỏ vĩnh viễn vì một bản chép cố ý có lỗi. Biến `SHOW_RED=1` bỏ cơ chế này để bạn nhìn thấy failure gốc. Trong dự án thật, đừng lạm dụng `it.fails`: một failure đã biết nên có defect report mở, không nên bị «chôn» thành màu xanh.

## 6. Góc nhìn nâng cao

**Test đạt không có nghĩa là không có defect.** TC-FARE-03 và TC-FARE-04 đạt **ngay trên bản có lỗi**: defect chỉ lộ ra khi tổng có phần lẻ. Nếu bộ test chỉ có hai ca này, bản lỗi đã lên production với bộ test xanh. Đây là nguyên lý đầu tiên trong [Bảy nguyên lý kiểm thử](/articles/bay-nguyen-ly-kiem-thu); vì sao một defect có lúc gây failure, có lúc không, xem [Error, Defect, Failure và Root Cause](/articles/error-defect-failure-root-cause).

**Oracle phải độc lập với code.** Sai lầm hay gặp: chạy code một lần, chép kết quả làm «mong đợi». Làm vậy thì bản có lỗi sẽ cho ra 51.000 và test khẳng định 51.000 là đúng — bộ test trở thành ảnh chụp của defect. Kết quả mong đợi trong bài đều tính tay từ SPEC.

**Testing và debugging tách hoạt động, không nhất thiết tách người.** Developer chạy unit test của chính mình rồi sửa là làm cả hai, lần lượt. Điều quan trọng là biết mình đang ở hoạt động nào: khi đang test thì ghi nhận trung thực; khi đang debug thì được phép thêm log, đặt breakpoint, sửa thử — những việc không được để lẫn vào bản build đang test.

**Debugging có thể dùng test làm công cụ.** Hai test «tái hiện» và «chẩn đoán» trong `debugging.test.ts` là code test, nhưng **mục đích** của chúng là debugging: tìm defect, không phải đánh giá chất lượng. Thước đo để phân loại là mục đích, không phải công cụ.

**Confirmation test không thay regression test.** Một bản sửa có thể đúng cho TC-FARE-01 nhưng làm hỏng chỗ khác — ví dụ developer «sửa» bằng cách cộng 999 rồi làm tròn xuống: TC-FARE-01 đạt, nhưng tổng 52.000,5 đ (giá trị trung gian lẻ dưới 1 đồng) bị đẩy sai. Chỉ chạy lại đúng ca vừa lỗi là không đủ.

**Chỉ dẫn sửa lỗi trong defect report là con dao hai lưỡi.** Gợi ý vị trí khả nghi giúp developer nhanh hơn, nhưng nếu tester ghi «sửa thành `Math.round`» thì 51.700 sẽ đúng mà 51.200 vẫn sai. Mô tả hành vi mong đợi theo đặc tả bền hơn mô tả cách sửa.

**Khi không tái hiện được.** Lỗi trong lab là tất định. Ở hệ thống đích, failure liên quan giờ cao điểm (R-FARE-04 tính theo giờ Việt Nam) có thể chỉ lộ ra khi máy chủ chạy ở múi giờ khác hoặc ngay mốc 13:00:00. Khi đó defect report cần ghi thêm thời điểm, `pricingVersion`, `quoteId` — và kiến trúc cần đồng hồ tiêm được (testability hook §17.2) để tái hiện trong môi trường test.

## 7. Bài tập và câu hỏi ôn tập

### Bài tập

**Bài 1.** Hệ thống trả 51.000 đ cho báo giá ở TC-FARE-01, oracle R-FARE-09 là 52.000 đ. Liệt kê các bước tester làm và các bước developer làm, tới lúc phiếu lỗi được đóng. Đánh dấu bước nào là testing, bước nào là debugging.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><b>Tester (testing):</b> (1) xác định oracle từ R-FARE-09; (2) chạy báo giá với đúng đầu vào, ghi lại 51.000 đ; (3) thử thêm đầu vào để mô tả phạm vi lỗi (3 km ra 25.000 đ, ô tô 10 km vẫn đúng); (4) viết defect report với căn cứ, bước tái hiện, mong đợi, thực tế.</p>
<p><b>Developer (debugging):</b> (5) tái hiện bằng đúng đầu vào; (6) chẩn đoán bằng bảng chi tiết cước, khoanh vùng hàm làm tròn; (7) sửa <code>Math.floor</code> thành <code>Math.ceil</code>.</p>
<p><b>Tester (testing):</b> (8) confirmation test — chạy lại TC-FARE-01 trên bản mới; (9) regression test — các ca cước khác và bộ test hiện có; (10) đóng phiếu nếu đạt, mở lại nếu không.</p>
</details>

**Bài 2.** Xếp mỗi việc sau vào *kiểm thử động*, *kiểm thử tĩnh* hoặc *debugging*: (a) đọc SPEC và phát hiện R-FARE-05 không nói phụ phí trời mưa có bị nhân hệ số cao điểm không; (b) đặt breakpoint trong `calculateFare`; (c) gọi `POST /api/quotes` với 100,05 km và kiểm lỗi `DISTANCE_OUT_OF_RANGE`; (d) chạy công cụ phân tích code báo một biến không dùng; (e) thêm `console.log` in từng bước trung gian; (f) chạy lại toàn bộ test cước sau khi sửa.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(a) kiểm thử tĩnh — review đặc tả (thực tế R-FARE-05 đã ghi rõ «không bị nhân 1,25», nên review này sẽ đóng câu hỏi ngay); (b) debugging; (c) kiểm thử động; (d) kiểm thử tĩnh — static analysis; (e) debugging; (f) kiểm thử động — regression testing.</p>
</details>

**Bài 3.** Tìm một đầu vào **ô tô** làm lộ defect làm tròn xuống, và một đầu vào ô tô **không** làm lộ. Giải thích.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Lộ: ô tô 2,3 km, giờ thường, khô ⇒ 25.000 + 11.000 × 2,3 = 50.300 đ; đúng là 51.000 đ, bản lỗi ra 50.000 đ. Không lộ: ô tô 2 km ⇒ 47.000 đ, đã tròn nghìn nên hai bản cho cùng kết quả. Quy luật: defect chỉ gây failure khi tổng sau bước 5 có phần lẻ dưới 1.000 đ.</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K1).** Điều nào sau đây là một mục tiêu kiểm thử điển hình theo CTFL v4.0.1?

- a) Sửa các defect tìm thấy trong code tính cước
- b) Đánh giá các work product như yêu cầu, thiết kế và code
- c) Chứng minh phần mềm không còn defect nào
- d) Viết lại đặc tả khi phát hiện mâu thuẫn

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b.</b> Đánh giá work product là mục tiêu đầu tiên trong danh sách của mục 1.1.1. a là debugging, không phải kiểm thử. c sai: kiểm thử chỉ giảm khả năng còn defect, không chứng minh được không còn. d là việc của người sở hữu đặc tả sau khi kiểm thử báo mâu thuẫn, không phải mục tiêu kiểm thử.</p>
</details>

**Câu 2 (K1).** Phát biểu nào đúng về kiểm thử phần mềm?

- a) Kiểm thử bắt đầu khi đã có bản build chạy được
- b) Kiểm thử chỉ gồm thực thi test và kiểm kết quả
- c) Kiểm thử gồm cả hoạt động không thực thi phần mềm, như review đặc tả
- d) Kiểm thử chỉ kiểm phần mềm có đúng đặc tả hay không

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c.</b> Static testing (review, static analysis) là kiểm thử. a và b là hai quan niệm sai syllabus nêu rõ. d sai vì kiểm thử gồm cả validation — đáp ứng nhu cầu thật của người dùng — chứ không chỉ verification.</p>
</details>

**Câu 3 (K2).** Tester chạy báo giá ô tô 2,3 km và nhận 50.000 đ thay vì 51.000 đ. Việc nào sau đây là **debugging**?

- a) Tester ghi lại đầu vào, kết quả mong đợi và thực tế vào defect report
- b) Tester chạy thêm ô tô 2 km để xem lỗi có lặp lại không
- c) Developer in giá trị từng bước trung gian để tìm bước tính sai
- d) Tester chạy lại ca 2,3 km sau khi có bản build mới

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c.</b> Tìm nguyên nhân của failure là chẩn đoán — một bước của debugging. a là báo cáo kết quả kiểm thử. b là kiểm thử thêm để mô tả phạm vi failure. d là confirmation testing.</p>
</details>

**Câu 4 (K2).** Trong buổi review pull request, một developer thấy hàm làm tròn dùng <code>Math.floor</code> trong khi đặc tả yêu cầu làm tròn lên. Phát biểu nào đúng?

- a) Phải tái hiện failure trên môi trường test trước khi được sửa
- b) Debugging ở đây chỉ còn việc gỡ bỏ defect, vì kiểm thử tĩnh đã tìm thẳng ra nó
- c) Đây chưa phải kiểm thử vì phần mềm chưa được chạy
- d) Phải chẩn đoán nguyên nhân bằng debugger trước khi sửa

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b.</b> Theo mục 1.1.2, khi kiểm thử tĩnh tìm ra defect thì không cần tái hiện hay chẩn đoán. a và d áp dụng cho failure từ kiểm thử động. c sai vì review là kiểm thử tĩnh.</p>
</details>

**Câu 5 (K2).** Sau khi developer sửa lỗi làm tròn, tester (A) chạy lại đúng ca 7,4 km từng thất bại, rồi (B) chạy toàn bộ bộ test báo giá. A và B lần lượt là gì?

- a) A: regression testing; B: confirmation testing
- b) A: debugging; B: regression testing
- c) A: confirmation testing; B: regression testing
- d) A: confirmation testing; B: debugging

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c.</b> Chạy lại ca từng thất bại để xem bản sửa có giải quyết vấn đề là confirmation testing; chạy các phần khác để xem bản sửa có làm hỏng gì là regression testing. Cả hai là kiểm thử, không phải debugging (loại b, d). a đảo ngược hai khái niệm.</p>
</details>

**Câu 6 (K1).** Hai ngày trước khi áp bảng giá mới, trưởng sản phẩm Tới Nơi hỏi đội kiểm thử: «Còn rủi ro gì nếu phát hành thứ Hai?». Đội kiểm thử đang phục vụ mục tiêu nào rõ nhất?

- a) Gây ra failure và tìm defect
- b) Bảo đảm độ bao phủ cần thiết của test object
- c) Cung cấp thông tin để các bên liên quan ra quyết định có căn cứ
- d) Đánh giá work product như user story

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c.</b> Câu hỏi là đầu vào cho quyết định phát hành — đúng mục tiêu cung cấp thông tin cho các bên liên quan. a và b có thể là hoạt động dẫn tới câu trả lời nhưng không phải điều người hỏi cần. d không liên quan tới quyết định phát hành.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus*, v4.0.1 (2024), Chương 1 — mục 1.1 «What is Testing?», 1.1.1 «Test Objectives» (FL-1.1.1, K1), 1.1.2 «Testing and Debugging» (FL-1.1.2, K2): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — tra các thuật ngữ testing, debugging, test object, test objective, confirmation testing, regression testing; truy cập từ trang chủ https://www.istqb.org/
- ISO/IEC/IEEE 29119-1, *Software and systems engineering — Software testing — Part 1: General concepts* — syllabus dẫn chiếu tiêu chuẩn này cho các khái niệm kiểm thử.
- Đặc tả lab Tới Nơi (SPEC v1.1): R-FARE-01…10, R-QUOTE-01…04, R-GEN-01, R-GEN-06 — `examples/toi-noi-testing/SPEC.md`.
- Tài liệu kiến trúc tổng thể Tới Nơi: §4.3 Pricing & Quote, §7.1 gRPC, §14.1 SLO, §17.2 testability hooks, §18 MVP.
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [Error, Defect, Failure và Root Cause](/articles/error-defect-failure-root-cause) · [Verification và Validation](/articles/verification-va-validation) · [Mục tiêu kiểm thử và Quality Assurance](/articles/muc-tieu-kiem-thu-va-quality-assurance) · [Testware và các sản phẩm kiểm thử](/articles/testware-san-pham-kiem-thu).

Bài tiếp theo: [Vì sao cần kiểm thử? (Why is Testing Necessary?)](/articles/vi-sao-can-kiem-thu)
