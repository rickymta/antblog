---
slug: non-functional-testing-tong-quan
title: "Non-functional Testing: tổng quan theo ISO/IEC 25010"
summary: "Non-functional testing hỏi «hệ thống hoạt động tốt tới mức nào». Bài đi qua 9 đặc tính chất lượng của ISO/IEC 25010:2023 theo CTFL v4.0.1, gán 7 yêu cầu R-NFR của Tới Nơi và giải thích vì sao phải kiểm thử từ sớm."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-018, non-functional-testing, iso-25010, quality-characteristics, toi-noi]
language: vi
---

Báo giá của Tới Nơi ra đúng 62.000 đ cho bưu kiện 6,1 kg — [TEST-017](/articles/functional-testing) đã kiểm thử điều đó. Nhưng nếu báo giá mất 4 giây mới hiện, hoặc một khách khác xem được chuyến của bạn, sản phẩm vẫn hỏng dù mọi test chức năng xanh. Bài này trả lời ba câu: non-functional testing kiểm những đặc tính nào theo ISO/IEC 25010, gán một yêu cầu cụ thể vào đặc tính nào, và vì sao không nên đợi tới cuối dự án mới kiểm thử.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-018</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S02 · Test Levels &amp; Test Types</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1 (đọc thêm CTAL-TTA v4.0, CTAL-TA v4.0)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Component; System</td></tr>
<tr><th scope="row">Test Type</th><td>Non-functional</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/functional-testing">TEST-017 · Functional Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Đọc SPEC: gán R-NFR-01…07 (SLO khởi tạo của kiến trúc §14.1) vào đặc tính ISO/IEC 25010 — có lời giải. Kèm lab <code>articles/non-functional-testing-tong-quan</code> (5 ca vitest)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 2 · mục 2.2.2 · FL-2.2.2 (K2). Đọc thêm: CTAL-TTA v4.0 · Chương 4 · mục 4.1, 4.2 · TTA-4.2.3 (K2); CTAL-TA v4.0 · Phụ lục F (mô hình ISO/IEC 25010:2023)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Functional testing hỏi «hệ thống làm **gì**». Non-functional testing hỏi «hệ thống làm điều đó **tốt tới mức nào**»: đủ nhanh không, có sẵn sàng khi khách cần không, có giữ kín dữ liệu không, có chạy trên điện thoại của tài xế không, có sửa được mà không vỡ chỗ khác không.

Vấn đề mà non-functional testing giải quyết có hai mặt. Thứ nhất, yêu cầu phi chức năng hay bị viết mơ hồ («phải nhanh», «phải an toàn») hoặc không được viết ra — CTAL-TTA mục 4.2.1 nói thẳng điều này. Không có chuẩn phân loại, đội sẽ chỉ nghĩ tới performance và bỏ quên cả nhóm đặc tính còn lại. Thứ hai, CTFL cảnh báo: phát hiện defect phi chức năng muộn có thể đe doạ cả dự án — vì sửa thường là sửa kiến trúc, không phải sửa một dòng code.

Khi nào dùng: ở mọi test level, theo mức ưu tiên rủi ro. Mỗi đặc tính có cách kiểm, môi trường và công cụ riêng — phần đi sâu nằm ở Series 09.

## 2. Lý thuyết theo ISTQB

### 2.1. Định nghĩa trong CTFL v4.0.1

Mục 2.2.2 (FL-2.2.2, K2) mô tả non-functional testing là đánh giá các thuộc tính **ngoài** đặc tính chức năng của một component hay hệ thống — kiểm thử «hệ thống hoạt động tốt tới mức nào». Mục tiêu chính: kiểm các **đặc tính chất lượng phi chức năng**. Syllabus dùng phân loại của chuẩn ISO/IEC 25010 với tám đặc tính:

1. Performance efficiency — hiệu năng
2. Compatibility — tương thích
3. Usability (còn gọi là interaction capability) — khả dụng
4. Reliability — tin cậy
5. Security — bảo mật
6. Maintainability — khả năng bảo trì
7. Portability (còn gọi là flexibility) — linh hoạt
8. Safety — an toàn

Đặc tính thứ chín — functional suitability — thuộc functional testing (TEST-017). Ghi chú phát hành của CTFL v4.0.1 giải thích: ISO/IEC 25010 bản **2023** đổi usability thành *interaction capability*, portability thành *flexibility*, thêm *safety*; syllabus giữ tên cũ, ghi kèm tên mới.

### 2.2. Mô hình chất lượng sản phẩm ISO/IEC 25010:2023

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 470" role="img" aria-label="Chín đặc tính chất lượng sản phẩm của ISO/IEC 25010:2023 cùng các đặc tính con. Functional suitability thuộc functional testing; tám đặc tính còn lại thuộc non-functional testing. Interaction capability, Flexibility và Safety là tên mới hoặc đặc tính mới ở bản 2023.">
<rect x="20" y="20" width="280" height="130" rx="8" class="f-phys-soft s-phys"/>
<text x="160" y="44" text-anchor="middle" class="t-b t-phys">Functional suitability</text>
<text x="160" y="66" text-anchor="middle" class="t-small">completeness · correctness</text>
<text x="160" y="84" text-anchor="middle" class="t-small">appropriateness</text>
<text x="160" y="130" text-anchor="middle" class="t-small t-phys">→ functional testing (TEST-017)</text>
<rect x="320" y="20" width="280" height="130" rx="8" class="f-surface s-line"/>
<text x="460" y="44" text-anchor="middle" class="t-b">Performance efficiency</text>
<text x="460" y="66" text-anchor="middle" class="t-small">time behaviour</text>
<text x="460" y="84" text-anchor="middle" class="t-small">resource utilization · capacity</text>
<rect x="620" y="20" width="280" height="130" rx="8" class="f-surface s-line"/>
<text x="760" y="44" text-anchor="middle" class="t-b">Compatibility</text>
<text x="760" y="66" text-anchor="middle" class="t-small">co-existence · interoperability</text>
<rect x="20" y="165" width="280" height="130" rx="8" class="f-warn-soft s-warn"/>
<text x="160" y="189" text-anchor="middle" class="t-b t-warn">Interaction capability</text>
<text x="160" y="209" text-anchor="middle" class="t-small">appropriateness recognizability</text>
<text x="160" y="226" text-anchor="middle" class="t-small">learnability · operability</text>
<text x="160" y="243" text-anchor="middle" class="t-small">user error protection · user engagement</text>
<text x="160" y="260" text-anchor="middle" class="t-small">inclusivity · user assistance</text>
<text x="160" y="277" text-anchor="middle" class="t-small">self-descriptiveness</text>
<rect x="320" y="165" width="280" height="130" rx="8" class="f-surface s-line"/>
<text x="460" y="189" text-anchor="middle" class="t-b">Reliability</text>
<text x="460" y="211" text-anchor="middle" class="t-small">faultlessness · availability</text>
<text x="460" y="229" text-anchor="middle" class="t-small">fault tolerance · recoverability</text>
<rect x="620" y="165" width="280" height="130" rx="8" class="f-surface s-line"/>
<text x="760" y="189" text-anchor="middle" class="t-b">Security</text>
<text x="760" y="211" text-anchor="middle" class="t-small">confidentiality · integrity</text>
<text x="760" y="229" text-anchor="middle" class="t-small">non-repudiation · accountability</text>
<text x="760" y="247" text-anchor="middle" class="t-small">authenticity · resistance</text>
<rect x="20" y="310" width="280" height="130" rx="8" class="f-surface s-line"/>
<text x="160" y="334" text-anchor="middle" class="t-b">Maintainability</text>
<text x="160" y="356" text-anchor="middle" class="t-small">modularity · reusability</text>
<text x="160" y="374" text-anchor="middle" class="t-small">analysability · modifiability</text>
<text x="160" y="392" text-anchor="middle" class="t-small">testability</text>
<rect x="320" y="310" width="280" height="130" rx="8" class="f-warn-soft s-warn"/>
<text x="460" y="334" text-anchor="middle" class="t-b t-warn">Flexibility</text>
<text x="460" y="356" text-anchor="middle" class="t-small">adaptability · scalability</text>
<text x="460" y="374" text-anchor="middle" class="t-small">installability · replaceability</text>
<rect x="620" y="310" width="280" height="130" rx="8" class="f-warn-soft s-warn"/>
<text x="760" y="334" text-anchor="middle" class="t-b t-warn">Safety</text>
<text x="760" y="356" text-anchor="middle" class="t-small">operational constraint</text>
<text x="760" y="374" text-anchor="middle" class="t-small">risk identification · fail safe</text>
<text x="760" y="392" text-anchor="middle" class="t-small">hazard warning · safe integration</text>
<rect x="20" y="452" width="14" height="10" class="f-warn-soft s-warn"/>
<text x="42" y="461" class="t-small">tên mới hoặc đặc tính mới ở bản 2023 (trước: usability, portability; safety mới thêm)</text>
</svg>
</div>
<figcaption><b>Hình 1 · Chín đặc tính, một mô hình.</b> Ô xanh là functional suitability; tám ô còn lại là đối tượng của non-functional testing. Đối chiếu theo Phụ lục F của CTAL-TA v4.0.</figcaption>
</figure>

Ngoài ba tên đặc tính, bản 2023 còn đổi một số đặc tính con (theo Phụ lục F của CTAL-TA v4.0): *maturity* thành *faultlessness*; *user interface aesthetics* thành *user engagement*; *accessibility* tách thành *inclusivity* và *user assistance*; thêm *self-descriptiveness*, *resistance* (security) và *scalability* (flexibility). CTAL-TTA v4.0 (2021) vẫn dùng tên bản 2011 — đọc tài liệu cũ thì đối chiếu.

| Đặc tính | Hỏi gì (diễn đạt lại) | Ví dụ ở Tới Nơi | Đi sâu |
|---|---|---|---|
| Performance efficiency | Nhanh, tốn tài nguyên, chịu được bao nhiêu so với điều kiện đã nêu? | p95 báo giá dưới 500 ms; 70 chuyến tạo mới/giây lúc cao điểm | S09: TEST-094…100 |
| Compatibility | Chạy chung, trao đổi dữ liệu được với hệ khác? | Gọi MapViet, VíXanh; app tài xế chạy cùng app dẫn đường | TEST-109 |
| Interaction capability | Người dùng dùng được hiệu quả, dễ học, không bấm nhầm, kể cả người khuyết tật? | Màn đặt chuyến có nhãn, `role="alert"` (R-UI-03) | TEST-090, 091, 111 |
| Reliability | Ít hỏng, sẵn sàng, chịu lỗi, phục hồi được? | Sẵn sàng 99,9%/tháng; VíXanh hết giờ vẫn không trừ tiền hai lần | TEST-105…108 |
| Security | Giữ kín, toàn vẹn, truy được trách nhiệm, chống tấn công? | Không xem được chuyến của khách khác; giới hạn tần suất | TEST-101…104 |
| Maintainability | Sửa, mở rộng, kiểm thử có dễ không? | Domain thuần không I/O, đồng hồ tiêm vào | TEST-110, static analysis S03 |
| Flexibility | Thích nghi môi trường mới, mở rộng quy mô, cài đặt, thay thế? | Thêm thành phố mới; mở rộng ngang Dispatch giờ cao điểm | TEST-099 |
| Safety | Tránh gây nguy hại cho người, tài sản, môi trường? | App tài xế không bắt thao tác khi xe đang chạy | (chưa có bài riêng) |

### 2.3. Vì sao nên kiểm từ sớm

CTFL v4.0.1 mục 2.2.2 nêu bốn ý, CTAL-TTA mục 4.1–4.2 (TTA-4.2.3, K2) mở rộng ra theo vòng đời:

- Đôi khi nên bắt đầu non-functional testing **sớm** — trong review hoặc component testing.
- Nhiều test phi chức năng được **dựng từ test chức năng**: chạy đúng test chức năng đó, thêm kiểm tra ràng buộc phi chức năng (ví dụ chức năng chạy xong trong thời gian quy định).
- Phát hiện defect phi chức năng **muộn** có thể đe doạ thành công của dự án.
- Một số test cần **môi trường rất riêng** (phòng thử usability, môi trường đo tải) — phải lên kế hoạch từ đầu.

| Giai đoạn / test level | Hoạt động phi chức năng | Ví dụ Tới Nơi |
|---|---|---|
| Phân tích yêu cầu | Review NFR: đo được không, thiếu đặc tính nào | R-NFR-03 ghi «phụ thuộc MapViet» — phần loại trừ phải định nghĩa |
| Thiết kế kiến trúc | Review kiến trúc theo từng đặc tính | Timeout, retry, circuit breaker cho bên thứ ba (kiến trúc §14) |
| Component | Benchmark, static analysis, kiểm tra sớm dựng từ unit test | `calculateFare` chạy hàng nghìn lần; độ phức tạp của `voucher.ts` |
| Component integration / API | Kiểm tra bảo mật cơ bản, timeout với adapter giả | IDOR, thân yêu cầu > 10 KB, VíXanh giả chế độ `timeout` |
| System | Tải, usability, reliability trên môi trường đại diện | p95 Booking dưới tải tham chiếu R-NFR-07 |
| System integration | Interoperability, chịu lỗi của bên thứ ba | MapViet chậm; webhook PSP trùng |
| Acceptance (vận hành) | Recoverability, sao lưu/khôi phục, giám sát | Khôi phục backup theo runbook (TEST-016) |
| Production | Theo dõi SLO; có khi test tiếp | Availability 99,9% đo theo tháng |

### 2.4. Bảng thuật ngữ Anh – Việt

| Thuật ngữ | Tiếng Việt dùng trong bài | Ghi chú |
|---|---|---|
| non-functional testing | kiểm thử phi chức năng | test type, CTFL 2.2.2 |
| quality characteristic | đặc tính chất lượng | 9 đặc tính trong ISO/IEC 25010:2023 |
| sub-characteristic | đặc tính con | ví dụ time behaviour thuộc performance efficiency |
| performance efficiency | hiệu năng | time behaviour, resource utilization, capacity |
| fault tolerance / recoverability | khả năng chịu lỗi / khả năng phục hồi | |
| confidentiality / resistance | tính bảo mật thông tin / khả năng chống chịu tấn công | resistance mới ở bản 2023 |
| interaction capability | khả năng tương tác | tên mới của usability |
| flexibility | tính linh hoạt | tên mới của portability |
| operational profile | hồ sơ vận hành | mô hình tải thực tế, CTAL-TTA 4.9 |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p><b>SLO, SLI, error budget</b> là thuật ngữ của Site Reliability Engineering, không phải thuật ngữ ISTQB. Tài liệu kiến trúc Tới Nơi (§14.1) dùng chúng cho mục tiêu khởi tạo; SPEC lab chép lại thành R-NFR-01…04. Gần nhất trong ISTQB: CTAL-TTA nói số đo đặc tính chất lượng trước phát hành có thể làm cơ sở cho SLA với các bên liên quan. Viết tắt «NFR» (non-functional requirement) cũng là cách gọi ngành.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Theo [hồ sơ dự án](/articles/du-an-mau-toi-noi), lúc cao điểm (11:00–13:00, 17:00–19:00) Tới Nơi có khoảng **70 chuyến tạo mới/giây** và **30.000 truy vấn đọc/giây**. Khách đang vội, tài xế dùng app ngoài đường, tiền đi qua VíXanh — mỗi đặc tính ở mục 2.2 đều có bên liên quan quan tâm.

### 3.2. Business rules: nhóm R-NFR của SPEC

Bảy yêu cầu sau là **mục tiêu khởi tạo để thảo luận, chưa phải cam kết** (SPEC mục 16):

| Mã | Nội dung (rút gọn) |
|---|---|
| R-NFR-01 | Public API sẵn sàng 99,9% theo tháng (≈ 43 phút gián đoạn/tháng 30 ngày) |
| R-NFR-02 | p95 `POST /api/trips` dưới 500 ms, không tính thời gian tìm tài xế |
| R-NFR-03 | p95 `POST /api/quotes` dưới 500 ms (hệ thống đích phụ thuộc MapViet) |
| R-NFR-04 | Thay đổi trạng thái chuyến tới app khách trong p95 dưới 2–3 giây; lab: trang khách hỏi lại mỗi 2 giây |
| R-NFR-05 | Gọi VíXanh: hết giờ 3 giây, tối đa 2 lần thử lại cùng khoá idempotency; không bao giờ trừ tiền hai lần |
| R-NFR-06 | Không lộ dữ liệu khách khác (R-API-05); lỗi 500 không lộ stack trace; giới hạn tần suất; giới hạn thân yêu cầu; API kiểm thử tắt mặc định |
| R-NFR-07 | Mức tải tham chiếu (không phải mục tiêu): ≈ 70 chuyến tạo mới/giây, ≈ 30.000 đọc/giây |

### 3.3. System context

Kiến trúc đích dàn trải các đặc tính lên nhiều thành phần: API Gateway giới hạn tần suất và xác thực token (§3.2, §13); Pricing gọi MapViet có timeout/fallback (§4.3); Payment xử lý webhook trùng và đối soát (§4.9); realtime gateway đẩy trạng thái (§10); Kafka với outbox/inbox, Redis cho lease và cache (§6, §14). Bản lab là modular monolith **một tiến trình**: không gateway, không Kafka, không Redis, không realtime gateway (trang khách hỏi lại mỗi 2 giây), kho trong bộ nhớ hoặc PGlite một kết nối. Vì vậy mọi số đo trên lab chỉ minh hoạ phương pháp — không kết luận được hệ thống đích đạt hay không.

### 3.4. Rủi ro có thể xảy ra

| Rủi ro (kiến trúc §13, §14, §21; giới hạn lab) | Đặc tính bị ảnh hưởng |
|---|---|
| MapViet chậm làm báo giá vượt 500 ms giờ cao điểm | Performance efficiency › time behaviour; Compatibility › interoperability |
| VíXanh hết giờ, Booking thử lại không cùng khoá ⇒ trừ tiền hai lần | Reliability › fault tolerance (và functional correctness) |
| Mã theo dõi công khai bị dò tuần tự, lộ vị trí chuyến | Security › confidentiality, resistance |
| Báo giá không bao giờ bị dọn khỏi kho (giới hạn đã biết của lab) | Performance efficiency › resource utilization, capacity |
| «Distributed monolith»: một service sập kéo theo cả luồng đặt chuyến | Reliability › fault tolerance, availability |
| Đổi nhà cung cấp SMS TinNhanh phải sửa nhiều service | Maintainability › modularity; Flexibility › replaceability |

## 4. Phân tích kỹ thuật

### 4.1. Ba câu hỏi để gán một yêu cầu vào đặc tính

1. **Yêu cầu nói về «làm gì» hay «làm tốt tới mức nào»?** «Không lộ chuyến của khách khác» có thể viết như một quy tắc chức năng (R-API-05 trả 404) nhưng thuộc đặc tính security. Một yêu cầu có thể có mặt ở cả hai.
2. **Đo bằng gì?** Phân vị độ trễ ⇒ time behaviour. Tỉ lệ thời gian phục vụ được ⇒ availability. Số lần thử lại khi bên thứ ba lỗi ⇒ fault tolerance.
3. **Đặc tính con nào khớp nhất?** Chọn **một đặc tính chính** để biết dùng test nào, môi trường nào và ai phụ trách; ghi **đặc tính phụ** khi yêu cầu chạm nhiều nơi. Đừng nhét một yêu cầu vào năm đặc tính — mất tác dụng phân loại.

### 4.2. Một yêu cầu phi chức năng tốt có gì?

Mổ xẻ R-NFR-03:

| Thành phần | Trong R-NFR-03 | Còn thiếu (nêu khi review) |
|---|---|---|
| Đại lượng | thời gian phản hồi `POST /api/quotes` | đo ở đâu: gateway hay service? |
| Thống kê | p95 | p99? tỉ lệ lỗi tối đa? |
| Ngưỡng | dưới 500 ms | |
| Điều kiện | — | ở mức tải nào (R-NFR-07?), cửa sổ đo bao lâu |
| Phần loại trừ | «phụ thuộc MapViet» | MapViet chậm thì tính vào hay loại ra? |

So với «báo giá phải nhanh», R-NFR-03 đã kiểm thử được — nhưng chưa đủ để đặt tiêu chí đạt/không đạt. Viết lại NFR cho đo được là nội dung của TEST-093 (Series 09).

### 4.3. Phân loại xong, nhìn chỗ trống

Gán xong bảy yêu cầu, xét ngược lại: **đặc tính nào chưa có yêu cầu nào?** Đây là bài toán completeness của chính bộ NFR (giống completeness chức năng ở TEST-017). Chỗ trống không mặc nhiên là lỗi — nó là **câu hỏi** đưa vào buổi review với Product Owner:

| Đặc tính chưa có R-NFR | Câu hỏi cho buổi review |
|---|---|
| Compatibility | Hỗ trợ Android/iOS từ phiên bản nào? Đổi phiên bản API MapViet thì sao? |
| Interaction capability | R-UI-03 có nhãn ô nhập, `role="alert"`, `aria-live` — có mục tiêu đo được (WCAG mức nào, tỉ lệ đặt thành công) không? |
| Maintainability | Ngưỡng độ phức tạp, độ phủ tối thiểu trong pipeline (kiến trúc §17)? |
| Flexibility | Mở rộng ngang Dispatch tới bao nhiêu lần tải tham chiếu? Thêm thành phố mới mất bao lâu? |
| Safety | App tài xế có được bắt thao tác khi xe đang chạy không? Có nút SOS không? |

## 5. Thực hành

**Đề bài (curriculum TEST-018):** gán 7 yêu cầu R-NFR-01…07 của lab (R-NFR-01…04 chính là SLO khởi tạo của kiến trúc §14.1) vào đặc tính ISO/IEC 25010 tương ứng. Tự làm trước — chỉ cần SPEC mục 16 và Hình 1 — rồi so với lời giải.

### 5.1. Lời giải

| Mã | Đặc tính chính › đặc tính con | Đặc tính phụ / ghi chú | Kiểm sớm nhất ở đâu |
|---|---|---|---|
| R-NFR-01 | Reliability › availability | Đo theo tháng, ở production hoặc chạy dài; lab một tiến trình không đo được | Review kiến trúc (điểm hỏng đơn lẻ); OAT |
| R-NFR-02 | Performance efficiency › time behaviour | Phải loại thời gian tìm tài xế — API trả ngay ở `searching` | Component/API (cảnh báo sớm), system (tải) |
| R-NFR-03 | Performance efficiency › time behaviour | Phụ: Compatibility › interoperability (MapViet); cần định nghĩa phần loại trừ | Như R-NFR-02 + system integration với MapViet |
| R-NFR-04 | Performance efficiency › time behaviour (đầu-cuối) | Lab hỏi lại mỗi 2 giây ⇒ riêng chu kỳ hỏi đã chiếm gần hết ngưỡng; hệ thống đích dùng realtime gateway (chưa có trong lab) | Review kiến trúc; system |
| R-NFR-05 | Reliability › fault tolerance | Phụ: Compatibility › interoperability (VíXanh). Vế «không trừ tiền hai lần» thực chất là **functional correctness** của R-PAY-04/R-IDEM, kiểm bằng ví giả chế độ `timeout`, `lost_response` | Component integration với adapter giả |
| R-NFR-06 | Security › confidentiality (IDOR, stack trace), resistance (giới hạn tần suất, giới hạn thân, API kiểm thử tắt) | Gom 5 kiểm tra; nên tách thành 5 yêu cầu để truy vết | API (component integration), review code |
| R-NFR-07 | **Không phải yêu cầu đạt/không đạt** | Dữ liệu đầu vào cho mô hình tải (operational profile) khi kiểm Performance efficiency › capacity và time behaviour dưới tải | Lập kế hoạch test hiệu năng |

Ba điểm hay sai: (1) xếp R-NFR-07 như một mục tiêu — SPEC ghi rõ «không phải mục tiêu»; (2) xếp R-NFR-05 vào Security vì «tiền» — cơ chế được mô tả là chịu lỗi khi bên thứ ba hết giờ; (3) bỏ qua phần loại trừ MapViet của R-NFR-03 — không có nó, một sự cố của MapViet sẽ bị tính là Tới Nơi vi phạm.

Lời giải cũng nằm trong lab dạng dữ liệu (`phan-loai-r-nfr.ts`), kèm một test kiểm tra lời giải có phủ **đủ** mọi mã `R-NFR-xx` trong `SPEC.md` và chỉ dùng tên 9 đặc tính bản 2023 — nếu SPEC thêm R-NFR-08 thì test sẽ đỏ.

### 5.2. Dựng kiểm tra phi chức năng từ một test chức năng

Minh hoạ ý «nhiều test phi chức năng dựng từ test chức năng» của CTFL: lấy test case FC-04 của TEST-017 (bưu kiện ô tô 3 km, 6,1 kg ⇒ 62.000 đ) và thêm ràng buộc:

| Mã | Từ test chức năng | Ràng buộc thêm | Đặc tính |
|---|---|---|---|
| NF-01 | 200 lần FC-04, mọi lần vẫn phải ra 62.000 đ | p95 thời gian phản hồi < 500 ms | Performance efficiency › time behaviour |
| NF-02 | Đặt chuyến từ báo giá FC-04 rồi khách C002 xem chuyến đó | Phản hồi giống hệt chuyến không tồn tại (cùng 404, cùng thân) | Security › confidentiality |
| NF-03 | Gửi FC-04 kèm mã voucher 11 KB | 413 `PAYLOAD_TOO_LARGE`, không xử lý | Security › resistance |

Lõi của NF-01 (`articles/non-functional-testing-tong-quan/nfr-tu-test-chuc-nang.test.ts`, rút gọn):

```ts
for (let i = 0; i < 200; i++) {
  const t0 = performance.now();
  const res = await request(server).post('/api/quotes').set('x-user', 'customer:C001').send(QUOTE);
  durations.push(performance.now() - t0);
  expect(res.status).toBe(200);              // ràng buộc phi chức năng không thay cho kết quả chức năng
  expect(res.body.quote.total).toBe(62_000);
}
durations.sort((a, b) => a - b);
expect(percentile(durations, 95)).toBeLessThan(500);
```

Chạy trong thư mục lab:

```bash
npx vitest run articles/non-functional-testing-tong-quan
```

Kết quả thực tế khi viết bài (vitest 3.2.7, Node 22.18, Intel Core i7-10700, 64 GB RAM, Windows 11, kho trong bộ nhớ; các số đo thay đổi nhẹ giữa các lần chạy):

```text
 ✓ articles/non-functional-testing-tong-quan/phan-loai-r-nfr.test.ts (2 tests)
NF-01: n=200 p50=1.1 ms p95=1.9 ms max=26.4 ms
 ✓ articles/non-functional-testing-tong-quan/nfr-tu-test-chuc-nang.test.ts (3 tests)

 Test Files  2 passed (2)
      Tests  5 passed (5)
```

### 5.3. Đọc kết quả cho đúng

- p95 = 1,9 ms **không** chứng minh R-NFR-03 đạt. Lab chạy một tiến trình, tuần tự, không mạng, không MapViet, không tải song song. NF-01 chỉ là **cảnh báo sớm**: nếu một thay đổi làm p95 của chính phép đo này nhảy từ 2 ms lên 200 ms, nhóm biết ngay ở pipeline, trước khi tới môi trường đo tải.
- `max = 26,4 ms` gấp 14 lần p95 vì lần gọi đầu phải khởi động kết nối. Ví dụ nhỏ cho thấy vì sao các phép đo hiệu năng dùng phân vị và bỏ giai đoạn khởi động; chi tiết ở TEST-094.
- NF-02, NF-03 chỉ là kiểm tra bảo mật **cơ bản** ở mức API; security testing thật ở Series 09 (TEST-101…104).

## 6. Góc nhìn nâng cao

**Non-functional ≠ performance.** Performance chỉ là một trong tám đặc tính. Đội chỉ đo tải sẽ bỏ sót yêu cầu bảo mật, khôi phục, khả năng bảo trì — đúng loại lỗi phát hiện muộn đắt nhất.

**Ranh giới chức năng / phi chức năng không rạch ròi.** Kiểm soát truy cập vừa là quy tắc chức năng (R-API-05: trả 404) vừa là đặc tính security. «Không trừ tiền hai lần» nằm trong R-NFR-05 nhưng kiểm như một quy tắc chức năng. Quan trọng không phải xếp nhãn đúng tuyệt đối, mà là không để yêu cầu nào rơi khỏi cả hai bộ test.

**Đo trên lab khác đo trên production.** CTAL-TTA lưu ý performance và reliability có thể cho kết quả khác ở production. Availability 99,9%/tháng chỉ đo được bằng giám sát thật; test trước phát hành chỉ giảm rủi ro (ví dụ kiểm thử chịu lỗi, failover).

**Static testing cũng kiểm đặc tính phi chức năng** (CTAL-TTA mục 4.1) — maintainability chủ yếu kiểm bằng phân tích tĩnh (TEST-029).

**Sai lầm thường gặp:**

| Sai lầm | Vì sao sai |
|---|---|
| Để non-functional testing tới cuối dự án | Defect phi chức năng thường là defect kiến trúc; phát hiện muộn đe doạ dự án (CTFL 2.2.2) |
| Coi mức tải tham chiếu là tiêu chí đạt | R-NFR-07 là đầu vào mô hình tải, không phải mục tiêu |
| Báo «p95 = 1,9 ms, đạt» từ máy dev | Sai môi trường, sai tải; chỉ là kiểm tra sớm |
| Một yêu cầu gán vào mọi đặc tính | Mất tác dụng phân loại; chọn một chính, vài phụ |

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Gán đặc tính

Gán đặc tính chính › đặc tính con cho 6 yêu cầu (a–d lấy ý từ tài liệu kiến trúc): (a) vị trí tài xế trên bản đồ khách không cũ quá 10 giây (p95); (b) che dữ liệu cá nhân trong log và dashboard; (c) mỗi thao tác của Admin có nhật ký không sửa/xoá tuỳ tiện; (d) Kafka sập thì đặt chuyến vẫn ghi được, sự kiện phát lại sau (outbox); (e) đổi nhà cung cấp SMS TinNhanh mà không sửa Booking; (f) app tài xế nhận diện được chữ cỡ lớn của hệ điều hành.

<details><summary><b>Lời giải mẫu</b></summary><ul><li>(a) Performance efficiency › time behaviour (độ tươi dữ liệu đầu-cuối).</li><li>(b) Security › confidentiality.</li><li>(c) Security › accountability (truy được hành động về người làm); phụ: non-repudiation.</li><li>(d) Reliability › fault tolerance; phụ: recoverability.</li><li>(e) Maintainability › modularity; phụ: Flexibility › replaceability.</li><li>(f) Interaction capability › inclusivity (trước đây: accessibility).</li></ul></details>

### Bài tập 2 — Kiểm sớm nhất ở đâu?

Với mỗi rủi ro, chọn hoạt động/test level sớm nhất phát hiện được: (1) R-NFR-03 không nói MapViet chậm có bị tính không; (2) `calculateFare` chậm hẳn sau khi thêm luật giá mới; (3) Booking thử lại thanh toán với khoá idempotency mới mỗi lần.

<details><summary><b>Lời giải</b></summary><ul><li>(1) Review yêu cầu (static testing) — lỗ hổng nằm trong test basis.</li><li>(2) Component testing: kiểm tra thời gian dựng từ unit test của <code>calculateFare</code>, chạy trong pipeline.</li><li>(3) Review code hoặc component integration testing với VíXanh giả chế độ <code>timeout</code>/<code>lost_response</code>, đếm số lần trừ tiền.</li></ul></details>

### Câu hỏi trắc nghiệm

**Câu 1 (K1).** Đặc tính nào **không** nằm trong danh sách đặc tính phi chức năng mà CTFL v4.0.1 lấy từ ISO/IEC 25010?

- a) Safety
- b) Functional suitability
- c) Compatibility
- d) Maintainability

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — safety có trong danh sách (thêm ở bản 2023).</li><li>b) Đúng — functional suitability là đối tượng của functional testing.</li><li>c) Sai — có trong danh sách.</li><li>d) Sai — có trong danh sách.</li></ul></details>

**Câu 2 (K2).** «p95 của `POST /api/quotes` dưới 500 ms» kiểm đặc tính nào?

- a) Reliability › availability
- b) Functional suitability › functional correctness
- c) Performance efficiency › time behaviour
- d) Flexibility › scalability

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Sai — availability là tỉ lệ thời gian phục vụ được.</li><li>b) Sai — yêu cầu không nói kết quả có đúng không.</li><li>c) Đúng — thời gian phản hồi.</li><li>d) Sai — scalability hỏi khả năng mở rộng khi tải tăng.</li></ul></details>

**Câu 3 (K2).** Theo CTFL v4.0.1, phát biểu nào đúng?

- a) Non-functional testing chỉ làm được ở system testing
- b) Nhiều test phi chức năng được dựng từ test chức năng, thêm kiểm tra một ràng buộc phi chức năng
- c) Non-functional testing không dùng được kỹ thuật thiết kế test
- d) Non-functional testing luôn cần phòng thử usability

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — có thể bắt đầu sớm, trong review hoặc component testing.</li><li>b) Đúng — mục 2.2.2.</li><li>c) Sai — mọi test technique có thể dùng cho mọi test type.</li><li>d) Sai — chỉ một số test (usability) cần môi trường riêng đó.</li></ul></details>

**Câu 4 (K2).** Vì sao syllabus khuyến nghị không để non-functional testing tới cuối dự án?

- a) Vì test phi chức năng rẻ hơn test chức năng
- b) Vì phát hiện muộn defect phi chức năng có thể đe doạ thành công của dự án
- c) Vì ISO/IEC 25010 yêu cầu kiểm theo thứ tự đặc tính
- d) Vì test phi chức năng không cần test basis

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — thường đắt hơn (môi trường, công cụ, kỹ năng riêng).</li><li>b) Đúng — mục 2.2.2.</li><li>c) Sai — ISO/IEC 25010 là mô hình phân loại, không quy định thứ tự kiểm.</li><li>d) Sai — vẫn cần NFR, SLO, chuẩn làm test basis.</li></ul></details>

**Câu 5 (K2).** R-NFR-07 ghi «≈ 70 chuyến tạo mới/giây, không phải mục tiêu». Cách dùng đúng nhất trong kiểm thử?

- a) Tiêu chí đạt: hệ thống phải xử lý đúng 70 chuyến/giây
- b) Đầu vào để dựng mô hình tải khi lập kế hoạch test hiệu năng
- c) Bỏ qua vì không phải yêu cầu
- d) Xếp vào Security vì liên quan giới hạn tần suất

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — SPEC nói rõ không phải mục tiêu.</li><li>b) Đúng — dữ liệu cho operational profile/mô hình tải.</li><li>c) Sai — vẫn là thông tin quan trọng để test có ý nghĩa.</li><li>d) Sai — đây là tải toàn hệ thống, không phải giới hạn mỗi khách.</li></ul></details>

**Câu 6 (K1).** Trong ISO/IEC 25010:2023, «interaction capability» là tên mới của đặc tính nào?

- a) Compatibility
- b) Usability
- c) Portability
- d) Reliability

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — compatibility giữ tên.</li><li>b) Đúng — CTFL v4.0.1 ghi «usability (also known as interaction capability)».</li><li>c) Sai — portability đổi thành flexibility.</li><li>d) Sai — reliability giữ tên.</li></ul></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 2.2 và 2.2.2 (FL-2.2.2), ghi chú phát hành về ISO 25010 bản 2023 — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level Technical Test Analyst Syllabus* v4.0 (2021): chương 4, mục 4.1, 4.2 (TTA-4.2.3), 4.9 — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB, *Certified Tester Advanced Level Test Analyst Syllabus* v4.0 (2025): chương 4 và Phụ lục F (so sánh ISO/IEC 25010:2023 với 2011) — https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (mục *non-functional testing*, *quality characteristic*).
- ISO/IEC 25010:2023, *Systems and software engineering — SQuaRE — Product quality model*; bản trước ISO/IEC 25010:2011.
- Lab Tới Nơi: `SPEC.md` mục 16 (R-NFR), R-API-01/05 và `articles/non-functional-testing-tong-quan/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-017 Functional Testing](/articles/functional-testing) · [TEST-014 System Testing](/articles/system-testing) · [TEST-016 Acceptance Testing](/articles/acceptance-testing-tong-quan) · [TEST-003 Bảy nguyên lý](/articles/bay-nguyen-ly-kiem-thu) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)

Bài tiếp theo: [TEST-019 · Confirmation Testing và Regression Testing](/articles/confirmation-testing-va-regression-testing)
