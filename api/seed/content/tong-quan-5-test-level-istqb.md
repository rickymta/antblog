---
slug: tong-quan-5-test-level-istqb
title: "Tổng quan 5 test level của ISTQB: từ component tới acceptance"
summary: "Năm test level của CTFL v4.0.1 — component, component integration, system, system integration, acceptance — phân biệt bằng test object, test basis, defect và người thực hiện, qua 12 kiểm thử của Booking ở Tới Nơi."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-011, test-level, component-testing, system-testing, integration-testing, acceptance-testing]
language: vi
---

Đội Tới Nơi có hàng trăm kiểm thử cho Booking: test hàm làm tròn cước, test gọi Pricing qua gRPC, test đặt chuyến trên trang web, test thu tiền qua VíXanh, buổi tài xế dùng thử app… Khi họp, «test tích hợp» của người này lại là «test hệ thống» của người kia. Bài này trả lời câu hỏi: **năm test level của ISTQB là gì, phân biệt chúng bằng tiêu chí nào**, và vì sao test level không phải là một thứ tự thời gian cứng nhắc.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-011</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S02 · Test Levels &amp; Test Types</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Component · Component Integration · System · System Integration · Acceptance</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/quy-trinh-kiem-thu-test-process">TEST-007 Quy trình kiểm thử</a>, <a href="/articles/kiem-thu-trong-sdlc-agile-waterfall">TEST-008 Kiểm thử trong SDLC</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Giấy/bảng tính: phân loại 12 kiểm thử của Booking vào 5 test level và ghi test basis của từng kiểm thử (mục 5, có lời giải)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 2 · mục 2.2 (đoạn mở đầu), 2.2.1 · FL-2.2.1 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Một hệ thống như Tới Nơi có nhiều «tầng» để kiểm thử: một hàm tính cước, một service, cả nền tảng, chỗ nối với ví VíXanh, và cuối cùng là câu hỏi «khách và tài xế có dùng được không». Nếu không chia tầng, đội test gặp hai vấn đề:

- **Trùng lặp:** cùng một mốc «đúng 2 phút» được kiểm ở unit test, ở API test, ở test trên app — tốn công mà không thêm thông tin.
- **Bỏ sót:** ai cũng nghĩ phần nối với VíXanh «chắc đội khác test rồi».

Test level giải quyết việc này: mỗi level có **đối tượng, mục tiêu, tài liệu căn cứ, loại lỗi và người phụ trách riêng**. Khi lập test plan hay viết test strategy (Series 12), câu đầu tiên thường là «level nào, ai làm, ở môi trường nào». Bài này là bản đồ tổng quan; mỗi level có bài riêng ngay sau trong Series 02.

## 2. Lý thuyết theo ISTQB

### 2.1. Test level và test type

Theo CTFL v4.0.1 (mục 2.2), **test level** là một nhóm hoạt động test được tổ chức và quản lý cùng nhau. Mỗi test level là **một lần chạy của test process** ([TEST-007](/articles/quy-trinh-kiem-thu-test-process)) gắn với phần mềm ở một giai đoạn phát triển — từ từng component tới cả hệ thống, có khi tới hệ thống của các hệ thống (system of systems).

Syllabus nói thêm ba ý dễ bị bỏ qua:

- Trong mô hình tuần tự, exit criteria của level trước thường là một phần entry criteria của level sau. Trong một số mô hình lặp, điều này **có thể không áp dụng**.
- Một hoạt động phát triển có thể trải qua nhiều test level.
- **Các test level có thể chồng lấn về thời gian.**

**Test type** thì khác: đó là nhóm hoạt động test gắn với một đặc tính chất lượng (functional, non-functional, black-box, white-box), và hầu hết làm được ở **mọi** test level. Level trả lời «kiểm thử ở phạm vi nào»; type trả lời «kiểm thử khía cạnh nào». Test type có bài riêng từ [TEST-017](/articles/functional-testing).

### 2.2. Năm test level của CTFL v4.0.1

| Test level | Trọng tâm theo syllabus |
|---|---|
| **Component testing** (còn gọi unit testing) | Kiểm từng component **tách riêng**. Thường cần hỗ trợ riêng như test harness hoặc unit test framework; thường do developer làm trong môi trường phát triển của mình. |
| **Component integration testing** (còn gọi unit integration testing) | Kiểm **interface và tương tác giữa các component**. Phụ thuộc mạnh vào chiến lược tích hợp: bottom-up, top-down, big-bang. |
| **System testing** | Kiểm **hành vi và năng lực tổng thể** của cả hệ thống: chức năng các tác vụ đầu-cuối và đặc tính phi chức năng. Một số đặc tính (vd usability) nên kiểm trên hệ hoàn chỉnh ở môi trường đại diện; có thể giả lập các hệ con. Có thể do đội test độc lập làm, căn cứ đặc tả hệ thống. |
| **System integration testing** | Kiểm **interface giữa hệ thống đang test với hệ thống khác và dịch vụ bên ngoài**. Cần môi trường test phù hợp, tốt nhất gần môi trường vận hành. |
| **Acceptance testing** | Tập trung vào **validation** và chứng minh **sẵn sàng triển khai**: hệ đáp ứng nhu cầu nghiệp vụ của người dùng. Lý tưởng là do người dùng dự kiến thực hiện. Các dạng chính: UAT, operational, contractual, regulatory, alpha, beta. |

### 2.3. Năm thuộc tính để phân biệt

Syllabus liệt kê (không đầy đủ) các thuộc tính phân biệt test level, để tránh chồng chéo: **test object**, **test objectives**, **test basis**, **defects and failures**, **approach and responsibilities**. CTFL v4.0.1 chỉ nêu tên thuộc tính; bảng dưới do blog điền, tổng hợp từ mô tả ở 2.2.1 và cách làm phổ biến.

| | Component | Component integration | System | System integration | Acceptance |
|---|---|---|---|---|---|
| **Test object** | Hàm, class, module | Interface giữa component, giữa service nội bộ | Cả hệ thống | Interface tới hệ khác, API bên ngoài | Hệ thống (hoặc một phần) trong bối cảnh nghiệp vụ |
| **Mục tiêu** | Component làm đúng phần việc của nó | Dữ liệu và lời gọi qua interface đúng | Hành vi đầu-cuối đúng đặc tả, đủ đặc tính chất lượng | Trao đổi với hệ ngoài đúng, kể cả khi hệ ngoài lỗi | Đáp ứng nhu cầu, sẵn sàng đưa vào dùng |
| **Test basis** | Thiết kế chi tiết, code, đặc tả component | Thiết kế kiến trúc, đặc tả interface (API, `.proto`, schema sự kiện) | Đặc tả hệ thống, use case, user story, yêu cầu phi chức năng | Đặc tả interface của đối tác, hợp đồng tích hợp | Yêu cầu nghiệp vụ, quy trình, tiêu chí chấp nhận, quy định, runbook |
| **Defect / failure điển hình** | Sai công thức, sai biên, sai làm tròn | Sai đơn vị, sai định dạng, gọi sai thứ tự, thiếu xử lý lỗi | Luồng nghiệp vụ hỏng, sai trạng thái, chậm, lộ dữ liệu | Hiểu sai API đối tác, không xử lý timeout/trùng/trễ | Làm đúng đặc tả nhưng không hợp nhu cầu; vận hành không được |
| **Cách tiếp cận, trách nhiệm** | Developer, framework + harness, môi trường dev | Developer hoặc tester, theo chiến lược tích hợp | Tester (có thể đội độc lập), môi trường đại diện | Tester + bên đối tác, sandbox/môi trường gần vận hành | Người dùng, khách hàng, vận hành, cơ quan quản lý |

### 2.4. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa | Ghi chú |
|---|---|---|
| test level | cấp độ kiểm thử | một lần chạy test process |
| test type | loại kiểm thử | functional, non-functional, black-box, white-box |
| test object | đối tượng kiểm thử | thứ đang bị kiểm |
| test basis | căn cứ kiểm thử | tài liệu/hiểu biết làm cơ sở suy ra test |
| test objective | mục tiêu kiểm thử | lý do làm test ở level đó |
| test harness | bộ khung kiểm thử | driver, stub… để chạy component tách riêng |
| integration strategy | chiến lược tích hợp | bottom-up, top-down, big-bang |
| entry / exit criteria | tiêu chí vào / ra | điều kiện bắt đầu / kết thúc một level |
| operational environment | môi trường vận hành | nơi hệ thống chạy thật |
| validation | xác nhận | đúng nhu cầu — xem [TEST-005](/articles/verification-va-validation) |

## 3. Tình huống kiểm thử tại Tới Nơi

**Yêu cầu nghiệp vụ.** Khách đặt chuyến từ một báo giá còn hạn; mỗi báo giá đặt được tối đa một chuyến (R-QUOTE-03); bấm «Đặt chuyến» lại không tạo chuyến trùng (R-IDEM, R-UI-01); chuyến trả bằng VíXanh được thu khi hoàn thành, hết giờ 3 giây, thử lại tối đa 2 lần cùng khoá idempotency (R-PAY-03, R-PAY-04).

**System context.** Theo tài liệu kiến trúc, Booking/Order là nguồn dữ liệu gốc của booking (§4.4), gọi Pricing & Quote qua gRPC (§7.1), phát `BookingCreated` qua Kafka với transactional outbox (§6.3); Payment & Wallet giao tiếp với VíXanh; Notification gửi SMS qua TinNhanh và push qua APNs/FCM; Pricing lấy quãng đường từ MapViet. Ranh giới «hệ thống Tới Nơi» bao mọi service của Tới Nơi; VíXanh, MapViet, TinNhanh, APNs/FCM là **hệ bên ngoài**.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 420" role="img" aria-label="Năm test level trên kiến trúc Tới Nơi: 1 component là fare.ts trong Pricing; 2 component integration là lời gọi gRPC Booking sang Pricing; 3 system là khung hệ thống Tới Nơi gồm Booking, Pricing, Dispatch, Trip, Notification, Payment; 4 system integration là các nối tới MapViet, VíXanh, TinNhanh, APNs/FCM; 5 acceptance là khách hàng, tài xế, vận hành nghiệm thu.">
<defs>
<marker id="t011-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="t011-ext" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<circle cx="30" cy="40" r="10" class="f-surface s-pk"/><text x="30" y="44" text-anchor="middle" class="t-b t-pk">5</text>
<text x="46" y="44" class="t-b">Người dùng</text>
<rect x="16" y="60" width="140" height="250" rx="10" class="f-pk-soft s-pk"/>
<rect x="30" y="80" width="112" height="40" rx="6" class="f-surface s-line"/><text x="86" y="105" text-anchor="middle">Khách hàng</text>
<rect x="30" y="140" width="112" height="40" rx="6" class="f-surface s-line"/><text x="86" y="165" text-anchor="middle">Tài xế</text>
<rect x="30" y="200" width="112" height="40" rx="6" class="f-surface s-line"/><text x="86" y="225" text-anchor="middle">Vận hành</text>
<text x="86" y="270" text-anchor="middle" class="t-small">nghiệm thu:</text>
<text x="86" y="288" text-anchor="middle" class="t-small">đúng nhu cầu?</text>
<line x1="156" y1="185" x2="188" y2="185" class="s-pk" stroke-width="1.8" marker-end="url(#t011-arrow)"/>
<rect x="196" y="48" width="504" height="324" rx="12" class="f-surface s-pk dash"/>
<circle cx="214" cy="66" r="10" class="f-surface s-pk"/><text x="214" y="70" text-anchor="middle" class="t-b t-pk">3</text>
<text x="230" y="70" class="t-b">Hệ thống Tới Nơi (system)</text>
<rect x="210" y="90" width="120" height="60" rx="8" class="f-bg s-line"/><text x="270" y="125" text-anchor="middle" class="t-b">Booking</text>
<rect x="400" y="90" width="160" height="110" rx="8" class="f-bg s-line"/><text x="480" y="112" text-anchor="middle" class="t-b">Pricing &amp; Quote</text>
<rect x="420" y="128" width="120" height="58" rx="6" class="f-phys-soft s-phys"/><text x="480" y="152" text-anchor="middle" class="t-mono t-phys">fare.ts</text><text x="480" y="172" text-anchor="middle" class="t-mono t-small">calculateFare</text>
<circle cx="546" cy="132" r="10" class="f-surface s-phys"/><text x="546" y="136" text-anchor="middle" class="t-b t-phys">1</text>
<rect x="585" y="90" width="100" height="60" rx="8" class="f-bg s-line"/><text x="635" y="125" text-anchor="middle" class="t-b">Dispatch</text>
<line x1="330" y1="120" x2="398" y2="120" class="s-pk" stroke-width="1.8" marker-end="url(#t011-arrow)"/>
<text x="364" y="112" text-anchor="middle" class="t-small">gRPC</text>
<circle cx="364" cy="138" r="10" class="f-surface s-pk"/><text x="364" y="142" text-anchor="middle" class="t-b t-pk">2</text>
<rect x="210" y="230" width="120" height="50" rx="8" class="f-bg s-line"/><text x="270" y="260" text-anchor="middle" class="t-b">Trip</text>
<rect x="400" y="230" width="160" height="50" rx="8" class="f-bg s-line"/><text x="480" y="260" text-anchor="middle" class="t-b">Notification</text>
<rect x="585" y="230" width="100" height="50" rx="8" class="f-bg s-line"/><text x="635" y="260" text-anchor="middle" class="t-b">Payment</text>
<circle cx="766" cy="40" r="10" class="f-surface s-warn"/><text x="766" y="44" text-anchor="middle" class="t-b t-warn">4</text>
<text x="782" y="44" class="t-b">Hệ bên ngoài</text>
<rect x="740" y="56" width="160" height="40" rx="8" class="f-warn-soft s-warn"/><text x="820" y="81" text-anchor="middle">MapViet (bản đồ)</text>
<rect x="740" y="237" width="160" height="36" rx="8" class="f-warn-soft s-warn"/><text x="820" y="260" text-anchor="middle">VíXanh (ví)</text>
<rect x="740" y="290" width="160" height="36" rx="8" class="f-warn-soft s-warn"/><text x="820" y="313" text-anchor="middle">TinNhanh (SMS)</text>
<rect x="740" y="336" width="160" height="36" rx="8" class="f-warn-soft s-warn"/><text x="820" y="359" text-anchor="middle">APNs / FCM (push)</text>
<path d="M480 90 L480 76 L738 76" class="f-none s-warn" stroke-width="1.6" marker-end="url(#t011-ext)"/>
<line x1="685" y1="255" x2="738" y2="255" class="s-warn" stroke-width="1.6" marker-end="url(#t011-ext)"/>
<path d="M460 280 L460 308 L738 308" class="f-none s-warn" stroke-width="1.6" marker-end="url(#t011-ext)"/>
<path d="M500 280 L500 354 L738 354" class="f-none s-warn" stroke-width="1.6" marker-end="url(#t011-ext)"/>
<text x="460" y="406" text-anchor="middle" class="t-small">1 Component · 2 Component integration · 3 System · 4 System integration · 5 Acceptance</text>
</svg>
</div>
<figcaption><b>Hình 1 · Năm test level đặt lên kiến trúc Tới Nơi.</b> Level được xác định bởi <i>phạm vi đối tượng</i> và <i>mục tiêu</i>, không phải bởi công cụ hay thời điểm chạy. Mũi tên gRPC Booking → Pricing nằm trong hệ thống nên là component integration; mũi tên vượt khung sang VíXanh, MapViet… là system integration.</figcaption>
</figure>

**Lab.** Phòng lab `examples/toi-noi-testing` là modular monolith giai đoạn MVP (SPEC R-ARCH-01): `src/domain/` là các component thuần, `src/app/` là use case ghép domain với repository, `src/api/` + `src/web/` là bề mặt hệ thống, VíXanh là bản giả (`fake-payment-gateway.ts`). Lab **không có** gRPC, Kafka, MapViet thật hay app di động (R-ARCH-02) — các ví dụ đó là của hệ đích.

**Rủi ro theo tầng.**

| Tầng | Rủi ro điển hình (từ tài liệu kiến trúc và SPEC) |
|---|---|
| Component | Làm tròn giữa chừng trái R-GEN-01; mốc 2 phút lệch một giây |
| Component integration | Booking đọc giá từ Pricing sai đơn vị (đồng vs milli-đồng); hai yêu cầu cùng dùng một báo giá (R-QUOTE-03); event `BookingCreated` thiếu trường |
| System | Bấm đặt hai lần tạo hai chuyến; khách xem được chuyến của người khác (R-NFR-06); p95 vượt 500 ms (R-NFR-02) |
| System integration | Webhook VíXanh trùng hoặc trễ; ví timeout ⇒ trừ tiền hai lần (§14) |
| Acceptance | Tài xế không hiểu luồng các bước; vận hành không xử lý được chuyến `paymentStatus = failed` |

## 4. Phân tích kỹ thuật

### 4.1. Ba câu hỏi để xếp một kiểm thử vào level

1. **Test object là gì — phạm vi lớn đến đâu?** Một hàm? Chỗ nối giữa hai phần của Tới Nơi? Cả Tới Nơi? Chỗ nối Tới Nơi với hệ ngoài?
2. **Test basis là gì?** Đặc tả của hàm, đặc tả interface, đặc tả hệ thống, hợp đồng với đối tác, hay nhu cầu nghiệp vụ?
3. **Mục tiêu là gì?** Tìm defect trong một phần (verification) hay chứng minh hệ dùng được cho nghiệp vụ (validation)?

Công cụ, người chạy hay thời điểm chạy **không** quyết định level: một test vitest dùng supertest gọi API cũng có thể là system test.

### 4.2. Một yêu cầu, năm level

Lấy luồng «khách đặt chuyến xe máy, trả bằng VíXanh». Mỗi level kiểm một phần khác nhau, không lặp lại nhau:

| Level | Test object | Ví dụ kiểm thử | Test basis | Defect cần bắt |
|---|---|---|---|---|
| Component | Quy tắc hạn báo giá; `customerCancellationFee` | Báo giá đúng bằng `expiresAt` là hết hạn; huỷ lúc 1:59 miễn phí, 2:00 có phí | R-QUOTE-01, R-CANCEL-02 | Dùng `<=` thay `<` |
| Component integration | `createTrip` ↔ kho báo giá (`markUsed`) | Hai lần tạo chuyến cùng `quoteId` ⇒ một chuyến, lần sau `QUOTE_ALREADY_USED` | R-QUOTE-03, interface `QuoteRepository` trong `ports.ts` | Đánh dấu «đã dùng» không có điều kiện ⇒ hai chuyến |
| System | Tới Nơi qua API và trang `/` | Đặt chuyến từ trang web tới `searching`; gửi lại cùng `Idempotency-Key` không tạo chuyến mới | R-UI-01, R-IDEM, R-STATE-02 | Nút «Đặt chuyến» sinh khoá mới mỗi lần bấm |
| System integration | Payment ↔ VíXanh sandbox | Ví hết giờ 2 lần rồi trả lời ⇒ `paid`, chỉ trừ một lần | R-PAY-03, R-PAY-04, tài liệu API VíXanh | Lần thử lại dùng khoá idempotency mới |
| Acceptance | Luồng đặt và trả tiền trong bối cảnh thật | 30 khách nội bộ đặt chuyến thật bằng ví trong một tuần; vận hành xử lý chuyến `failed` theo runbook | Tiêu chí chấp nhận, quy trình vận hành | Hoá đơn khó hiểu; runbook thiếu bước hoàn tiền |

### 4.3. Đối chiếu bảng chiến lược test của kiến trúc (§17.1)

Tài liệu kiến trúc liệt kê «Test level/type» theo ngôn ngữ của đội kỹ thuật. Đối chiếu sang ISTQB giúp thấy chỗ trộn giữa level và type:

| Dòng §17.1 | Phạm vi ví dụ | Theo ISTQB |
|---|---|---|
| Unit | Pricing rules, state transition, fee calculation | Component testing |
| Component | Booking service + PostgreSQL/Redis test instance | Giáp ranh component / component integration — tuỳ đội coi «component» là cả service hay từng module (xem [TEST-012](/articles/component-testing-unit-testing-tong-quan)) |
| Contract | Booking ↔ Pricing gRPC; schema `BookingCreated` | Component integration (hai service nội bộ); contract testing là cách làm, không phải level |
| Integration | Outbox relay, Kafka consumer, payment webhook | Outbox, consumer: component integration · webhook PSP: system integration |
| API | REST validation, auth, idempotency | Không phải level — API là bề mặt; thường là system testing qua API |
| System | Customer đặt chuyến từ đầu đến cuối | System testing |
| E2E | Customer App ↔ Driver App ↔ Backend ↔ PSP sandbox | System + system integration; «E2E» là thuật ngữ ngành, không phải level ISTQB |
| Performance, Resilience, Security, Data consistency, Mobile | — | Test type (thường non-functional), làm ở một hoặc nhiều level |

## 5. Thực hành

**Đề bài (bài thực hành của curriculum).** Phân loại 12 kiểm thử của Booking vào 5 test level và ghi test basis của mỗi kiểm thử. Tự làm trước, rồi mở lời giải.

| # | Kiểm thử |
|---|---|
| T1 | Hàm `roundUpToThousandDong` với 51.700 / 52.000 / 52.001 đ |
| T2 | `customerCancellationFee` khi huỷ đúng 2 phút 0 giây sau khi tài xế nhận |
| T3 | Use case `createTrip` với kho báo giá trong bộ nhớ: báo giá đã dùng bị từ chối |
| T4 | Booking gọi `GetQuote` của Pricing qua gRPC, Pricing không trả lời trước deadline |
| T5 | Booking ghi chuyến vào PostgreSQL và bản ghi outbox `BookingCreated` trong cùng giao dịch |
| T6 | Khách đặt chuyến xe máy trên trang `/` từ báo giá tới trạng thái `searching` |
| T7 | Gửi lại `POST /api/trips` cùng `Idempotency-Key` sau khi mạng đứt |
| T8 | Đo p95 của `POST /api/trips` trên staging ở mức tải tham chiếu |
| T9 | Thu tiền chuyến `vixanh` qua sandbox VíXanh: hết giờ rồi thử lại |
| T10 | Push «tài xế đã nhận chuyến» tới điện thoại Android thật qua FCM |
| T11 | 20 tài xế ở Đà Nẵng dùng bản app mới trong một tuần, ghi góp ý |
| T12 | Đội vận hành diễn tập khôi phục backup database Booking theo runbook |

<details>
<summary><b>Lời giải mẫu</b></summary>
<table>
<thead><tr><th>#</th><th>Test level</th><th>Test basis</th><th>Vì sao</th></tr></thead>
<tbody>
<tr><td>T1</td><td>Component</td><td>R-FARE-08; code <code>money.ts</code></td><td>Một hàm thuần, kiểm tách riêng</td></tr>
<tr><td>T2</td><td>Component</td><td>R-CANCEL-01, R-CANCEL-02</td><td>Hàm nhận <code>now</code> qua tham số, không cần đồng hồ hay kho</td></tr>
<tr><td>T3</td><td>Component integration</td><td>R-QUOTE-03; interface <code>QuoteRepository.markUsed</code> trong <code>ports.ts</code></td><td>Kiểm tương tác use case ↔ repository</td></tr>
<tr><td>T4</td><td>Component integration</td><td>File <code>.proto</code> của GetQuote; quy ước deadline §7.2</td><td>Hai service đều thuộc Tới Nơi; không phải system integration</td></tr>
<tr><td>T5</td><td>Component integration</td><td>Thiết kế outbox §6.3; schema sự kiện</td><td>Tương tác code ↔ database của service (Kafka chưa có trong lab)</td></tr>
<tr><td>T6</td><td>System</td><td>R-UI-01, R-QUOTE-03, R-STATE-02</td><td>Tác vụ đầu-cuối trên hệ hoàn chỉnh</td></tr>
<tr><td>T7</td><td>System</td><td>R-IDEM, R-VALID-02</td><td>Kiểm hành vi quan sát được từ ngoài hệ qua API công khai</td></tr>
<tr><td>T8</td><td>System (test type non-functional)</td><td>R-NFR-02, SLO §14.1</td><td>Hiệu năng là test type; ở đây làm ở level system, môi trường đại diện</td></tr>
<tr><td>T9</td><td>System integration</td><td>R-PAY-03, R-PAY-04; tài liệu API VíXanh</td><td>Interface tới hệ bên ngoài, môi trường gần vận hành</td></tr>
<tr><td>T10</td><td>System integration</td><td>Tài liệu FCM; yêu cầu thông báo của Notification</td><td>Chỗ nối Tới Nơi ↔ dịch vụ push bên ngoài</td></tr>
<tr><td>T11</td><td>Acceptance (beta)</td><td>Quy trình nghiệp vụ tài xế; tiêu chí chấp nhận của bản phát hành</td><td>Người dùng dự kiến, môi trường của họ, mục tiêu validation</td></tr>
<tr><td>T12</td><td>Acceptance (operational)</td><td>Runbook; RTO/RPO; §16.4 kiến trúc</td><td>Chứng minh hệ vận hành được, do vận hành thực hiện</td></tr>
</tbody>
</table>
<p>Ba chỗ dễ sai: T4 (gRPC nội bộ ≠ system integration), T7 (idempotency có thể kiểm ở component của module <code>idempotency.ts</code>, nhưng kiểm qua API trên hệ hoàn chỉnh thì là system) và T8 (performance là test type, không phải level thứ sáu).</p>
</details>

**Cách dùng kết quả.** Đưa cột «Test level» vào bảng test của Booking giúp thấy ngay chỗ trống: trong 12 kiểm thử trên, chưa có kiểm thử system integration nào cho MapViet — một rủi ro vì Quote API phụ thuộc bản đồ (R-NFR-03).

## 6. Góc nhìn nâng cao

- **Test level ≠ thứ tự thời gian.** Ở Tới Nơi, acceptance test cho phí huỷ được viết trước khi có code (ATDD, xem [TEST-008](/articles/kiem-thu-trong-sdlc-agile-waterfall)); test system integration với sandbox VíXanh có thể chạy trước khi system test toàn luồng xong; beta với tài xế chạy song song với component test của bản kế tiếp. Syllabus nói rõ các level có thể chồng lấn về thời gian.
- **«Integration» có hai nghĩa.** Một câu «đã test tích hợp» chưa nói gì: tích hợp *trong* Tới Nơi (component integration) hay *ra ngoài* (system integration)? Viết rõ trong test plan.
- **Ranh giới «hệ thống» là một quyết định.** Với microservices, có đội coi mỗi service là một hệ riêng; khi đó Booking ↔ Pricing được xếp vào system integration. Series này coi cả nền tảng Tới Nơi là *một* hệ thống; điều quan trọng là chọn một cách và ghi vào test strategy.
- **Level không đồng nghĩa với người làm.** Component testing «thường» do developer làm, nhưng tester vẫn góp test condition và dữ liệu biên; acceptance «lý tưởng» do người dùng làm, nhưng tester thường chuẩn bị môi trường và kịch bản.
- **Mỗi level có mục tiêu riêng để tránh trùng lặp** (thực hành tốt của mục 2.1.2). Đừng kiểm thử lại mọi mốc thời gian ở system level; ở đó chỉ cần đại diện vài trường hợp và tập trung vào luồng, dữ liệu thật, đặc tính phi chức năng.
- **Sai lầm thường gặp:** coi «API test», «E2E test», «smoke test» là test level; coi system testing chỉ là functional (syllabus nêu rõ có non-functional); coi acceptance là «chạy lại system test với người dùng đứng xem».

Đi sâu từng level: component ở [TEST-012](/articles/component-testing-unit-testing-tong-quan) rồi Series 06; component integration ở [TEST-013](/articles/component-integration-testing) rồi Series 07; system ở [TEST-014](/articles/system-testing) rồi Series 08; system integration ở [TEST-015](/articles/system-integration-testing); acceptance ở [TEST-016](/articles/acceptance-testing-tong-quan) rồi TEST-092.

## 7. Bài tập và câu hỏi ôn tập

**Bài 1.** Quy tắc R-VOUCHER-07 giới hạn số lượt dùng voucher của mỗi khách trong một ngày giờ Việt Nam. Đề xuất một kiểm thử cho mỗi level có ý nghĩa, ghi test object và test basis.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><b>Component:</b> <code>assertUsageAvailable</code> với số lượt đã dùng = giới hạn − 1 và = giới hạn (R-VOUCHER-07). <b>Component integration:</b> <code>assertVoucherQuota</code> với kho chuyến thật — chuyến đã huỷ không tính lượt, khoảng ngày [00:00; 24:00) giờ VN (R-GEN-02, interface <code>countVoucherUses</code>). <b>System:</b> qua API, khách đã dùng đủ <code>perCustomerPerDay</code> lượt trong ngày tạo thêm chuyến bằng cùng mã ⇒ <code>VOUCHER_USAGE_LIMIT</code> (R-QUOTE-03). <b>System integration:</b> không có hệ ngoài nào tham gia — bỏ, và ghi lý do. <b>Acceptance:</b> bộ phận marketing xác nhận thông báo hết lượt dễ hiểu và đúng chính sách khuyến mãi. Không phải yêu cầu nào cũng cần cả 5 level.</p>
</details>

**Bài 2.** Dòng «E2E: Customer App ↔ Driver App ↔ Backend ↔ PSP sandbox» của §17.1 thuộc test level nào? Nếu muốn tách thành các kiểm thử ISTQB rõ ràng, bạn tách thế nào?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Dòng này trộn hai level: <b>system testing</b> (luồng khách đặt, tài xế nhận và hoàn thành trên hệ hoàn chỉnh) và <b>system integration testing</b> (thu tiền qua PSP sandbox). Tách: (1) system test luồng đầu-cuối với ví giả để luồng ổn định, căn cứ R-STATE-02, R-UI-01, R-UI-02; (2) system integration test riêng với sandbox VíXanh cho thu tiền, timeout, webhook, căn cứ R-PAY và tài liệu đối tác; (3) giữ một vài kiểm thử E2E đầy đủ làm smoke trước phát hành. Tách ra giúp biết lỗi nằm ở đâu khi test đỏ.</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K1).** Mục nào KHÔNG phải một trong năm test level của CTFL v4.0.1?

- a) Component integration testing
- b) System integration testing
- c) Regression testing
- d) Acceptance testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng.</b> Regression testing là hoạt động kiểm thử sau thay đổi (mục 2.2.3), làm được ở mọi level. a), b), d) đều nằm trong năm level của 2.2.1.</p>
</details>

**Câu 2 (K2).** Booking gọi Pricing qua gRPC, cả hai đều là service của Tới Nơi. Kiểm thử tập trung vào việc Booking xử lý đúng mã lỗi `DEADLINE_EXCEEDED` của Pricing thuộc level nào (theo cách Series này coi Tới Nơi là một hệ thống)?

- a) Component testing
- b) Component integration testing
- c) System integration testing
- d) Acceptance testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng:</b> trọng tâm là interface và tương tác giữa hai thành phần trong cùng hệ thống. a) sai: không kiểm một component tách riêng. c) sai: system integration là interface với hệ khác hoặc dịch vụ bên ngoài. d) sai: không nhằm validation nhu cầu nghiệp vụ.</p>
</details>

**Câu 3 (K2).** Phát biểu nào về system integration testing là đúng theo CTFL v4.0.1?

- a) Tập trung vào interface giữa hệ thống đang test với hệ thống khác và dịch vụ bên ngoài, cần môi trường test tốt nhất gần môi trường vận hành
- b) Chỉ làm sau khi acceptance testing xong
- c) Là tên khác của component integration testing
- d) Chỉ kiểm chức năng, không quan tâm lỗi của hệ bên ngoài

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a) đúng</b>, đúng mô tả 2.2.1. b) sai: level không gắn thứ tự cứng, và SIT thường làm trước acceptance. c) sai: component integration là giữa các component bên trong. d) sai: phản ứng khi VíXanh timeout chính là mục tiêu quan trọng của SIT.</p>
</details>

**Câu 4 (K2).** Phát biểu nào đúng?

- a) Component testing luôn do đội test độc lập thực hiện
- b) System testing chỉ gồm functional testing
- c) Acceptance testing tập trung vào validation và chứng minh hệ sẵn sàng triển khai
- d) Trong mọi SDLC, exit criteria của level trước luôn là entry criteria của level sau

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng.</b> a) sai: syllabus nói component testing thường do developer làm. b) sai: system testing gồm cả non-functional. d) sai: điều này thường thấy ở mô hình tuần tự, có thể không áp dụng ở mô hình lặp.</p>
</details>

**Câu 5 (K2).** Đội Tới Nơi đo p95 của `POST /api/trips` trên staging. Phát biểu nào đúng nhất?

- a) Đây là test level thứ sáu: performance level
- b) Đây là một test type (non-functional) và ở tình huống này được làm ở level system
- c) Đây là acceptance testing vì có con số mục tiêu
- d) Đây là component testing vì chỉ gọi một endpoint

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng.</b> Test type gắn với đặc tính chất lượng và làm được ở nhiều level. a) sai: CTFL chỉ có năm level. c) sai: có số mục tiêu không biến nó thành validation bởi người dùng. d) sai: một endpoint trên hệ hoàn chỉnh vẫn đi qua toàn bộ hệ thống.</p>
</details>

**Câu 6 (K2).** Đâu là tập thuộc tính CTFL v4.0.1 dùng để phân biệt các test level?

- a) Công cụ, ngôn ngữ lập trình, thời lượng chạy
- b) Test object, test objectives, test basis, defects and failures, approach and responsibilities
- c) Ngân sách, số tester, số test case
- d) Test technique, test coverage, test data

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b> — danh sách (không đầy đủ) ở cuối 2.2.1. a), c), d) có thể khác nhau giữa các level nhưng không phải tiêu chí syllabus nêu; dùng công cụ để xếp level là sai lầm phổ biến.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — mục 2.2 Test Levels and Test Types, 2.2.1 Test Levels (LO FL-2.2.1). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (test level, test type, test object, test basis, test harness).
- Tài liệu kiến trúc Tới Nơi (nội bộ blog): §4 phân rã service, §6.3 outbox, §7 gRPC, §14 resilience, §17.1 chiến lược test theo kiến trúc.
- Phòng lab: `SPEC.md` (R-QUOTE, R-IDEM, R-PAY, R-NFR, R-ARCH) — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-005 Verification và Validation](/articles/verification-va-validation) · [TEST-007 Quy trình kiểm thử](/articles/quy-trinh-kiem-thu-test-process) · [TEST-008 Kiểm thử trong SDLC](/articles/kiem-thu-trong-sdlc-agile-waterfall) · [TEST-010 Testware](/articles/testware-san-pham-kiem-thu)

Bài tiếp theo: [TEST-012 · Component Testing (Unit Testing): góc nhìn tổng quan](/articles/component-testing-unit-testing-tong-quan).
