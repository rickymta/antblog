---
slug: component-integration-testing
title: "Component Integration Testing: kiểm chỗ các component ghép với nhau"
summary: "Test level thứ hai của CTFL v4.0.1: kiểm interface và tương tác giữa các component trong cùng hệ thống. Năm defect giao tiếp thật giữa quote.ts, pricing.ts và voucher.ts của lab Tới Nơi, kèm test chạy được."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-013, test-level, integration-testing, interface, component-integration]
language: vi
---

Hàm tính cước đúng, hàm voucher đúng, hàm giờ cao điểm đúng — vậy giá báo cho khách có đúng không? Chưa chắc. Bài này trả lời: **component integration testing kiểm cái gì mà component testing không kiểm được**, defect giao tiếp trông ra sao, xác định interface cần kiểm thế nào — qua năm cách ghép sai chuỗi `quote.ts` → `pricing.ts` → `voucher.ts` của lab Tới Nơi mà không component nào sai.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-013</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S02 · Test Levels &amp; Test Types</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Component Integration</td></tr>
<tr><th scope="row">Test Type</th><td>Functional</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/tong-quan-5-test-level-istqb">TEST-011 Tổng quan 5 test level</a>, <a href="/articles/component-testing-unit-testing-tong-quan">TEST-012 Component Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): liệt kê interface giữa <code>src/app/quote.ts</code> và <code>src/domain/pricing.ts</code>/<code>voucher.ts</code>, viết 5 defect giao tiếp và test bắt từng defect — <code>examples/toi-noi-testing/articles/component-integration-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 2 · mục 2.2 (2.2.1) · FL-2.2.1 (K2); liên quan 1.5.3 (độc lập kiểm thử), 5.1.7 (testing quadrants)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Ở [TEST-012](/articles/component-testing-unit-testing-tong-quan), mỗi component được kiểm **cô lập**: `calculateFare` nhận số liệu cho sẵn, phụ thuộc như đồng hồ, thời tiết được thay bằng bản giả. Cô lập giúp test nhanh và chỉ rõ chỗ hỏng — nhưng cũng che mất một loại lỗi: lỗi nằm **giữa** hai component. Bên gọi tưởng tiền tính bằng đồng, bên nhận tính bằng milli-đồng; bên gọi tưởng bên nhận tự chuẩn hoá mã voucher, bên nhận lại tưởng ngược lại. Mỗi bên đều «đúng» theo cách hiểu của mình.

**Component integration testing** sinh ra để bắt loại lỗi đó — khi các component bắt đầu được ghép, khi một interface vừa đổi (thêm tham số, đổi đơn vị, đổi thứ tự gọi), hay khi tầng ứng dụng điều phối nhiều quy tắc domain như `priceQuote`.

Bài dừng ở mức **test level**: mục tiêu, đối tượng, defect điển hình, ai làm. Chiến lược tích hợp (top-down, bottom-up, big-bang, incremental) và kỹ thuật viết integration test đi sâu ở Series 07 (TEST-065 … TEST-069).

## 2. Lý thuyết theo ISTQB

### 2.1. Định nghĩa

CTFL v4.0.1 (mục 2.2.1) mô tả component integration testing — còn gọi là *unit integration testing* — là test level **tập trung vào interface và tương tác giữa các component**, và nhấn mạnh nó **phụ thuộc nhiều vào chiến lược tích hợp** như bottom-up, top-down hay big-bang. ISTQB Glossary định nghĩa tương tự: test item là các interface và tương tác giữa những component đã được tích hợp.

Hai ý cần giữ:

1. **Đối tượng là chỗ nối, không phải từng component.** Nếu một ca test chỉ chạy đúng một hàm với đầu vào giả, đó là component test, dù nó nằm trong thư mục tên `integration/`.
2. **«Component» là khái niệm tương đối.** Trong lab, component là một module TypeScript. Trong kiến trúc đích của Tới Nơi, Booking Service và Pricing Service là hai component của **cùng một hệ thống** — gọi gRPC giữa chúng vẫn là component integration. Ranh giới chuyển sang system integration khi interface đi ra **ngoài** hệ thống: VíXanh, MapViet, TinNhanh ([TEST-015](/articles/system-integration-testing)).

### 2.2. Năm thuộc tính phân biệt test level, áp vào component integration

Syllabus phân biệt test level bằng năm thuộc tính (danh sách không đầy đủ): test object, test objective, test basis, defect/failure, approach và responsibility. Bảng dưới là cách bài này áp chúng vào Tới Nơi — syllabus v4.0.1 chỉ nêu tên thuộc tính, không cho ví dụ từng level.

| Thuộc tính | Component integration testing ở Tới Nơi |
|---|---|
| Test object | Interface giữa các component: lời gọi hàm `quote.ts` → `pricing.ts`; lời gọi gRPC `GetQuote` Booking → Pricing; schema sự kiện `BookingCreated` giữa Booking và Dispatch |
| Test objective | Xác nhận dữ liệu đi qua interface đúng nghĩa, đúng đơn vị, đúng thứ tự gọi; lỗi được truyền và xử lý đúng; phát hiện defect giao tiếp trước khi lên system test |
| Test basis | Thiết kế kiến trúc, hợp đồng interface (chữ ký hàm, `.proto`, OpenAPI, schema event), sơ đồ tuần tự, các quy tắc chung xuyên module như R-GEN-01, R-GEN-02, R-VALID-01 |
| Defect / failure điển hình | Sai đơn vị, dữ liệu bị biến đổi giữa đường, hai bên giả định khác nhau về ai chuẩn hoá, sai thứ tự gọi, lỗi của bên được gọi không được xử lý |
| Approach, responsibility | Thường do developer làm (FL-1.5.3: developer làm component và component integration testing), tự động hoá trong CI; chọn chiến lược tích hợp và bản giả cho phần chưa ghép |

### 2.3. Vị trí trong mô hình của syllabus

Trong **testing quadrants** (mục 5.1.7), component test và component integration test nằm ở **Q1** — hướng công nghệ, hỗ trợ đội, nên tự động hoá trong CI. Trong **test pyramid** (mục 5.1.6), một mô hình phổ biến gọi tầng giữa là *integration (component integration) tests*. Test type chủ yếu là functional, nhưng vẫn kiểm được phi chức năng ở chỗ nối (thời gian chờ của lời gọi gRPC, kích thước payload).

### 2.4. Bảng thuật ngữ

| Thuật ngữ | Nghĩa tiếng Việt | Ghi chú |
|---|---|---|
| component integration testing | kiểm thử tích hợp component | còn gọi unit integration testing |
| interface | giao diện (điểm nối) | chữ ký hàm, API, message |
| integration strategy | chiến lược tích hợp | top-down, bottom-up, big-bang |
| stub / driver | phần giả bên được gọi / bên gọi | thay component chưa có |
| system integration testing | kiểm thử tích hợp hệ thống | interface với hệ thống ngoài |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Ngoài syllabus v4.0.1</span>
<p>Danh sách «defect tích hợp điển hình» dùng ở mục 4 tổng hợp từ thực hành ngành (tài liệu kiểm thử thường liệt kê tương tự cho integration testing): dữ liệu sai hoặc thiếu, mã hoá sai; gọi interface sai thứ tự hoặc sai thời điểm; interface không khớp; lỗi giao tiếp giữa component không được xử lý; giả định sai về ý nghĩa, đơn vị, biên của dữ liệu truyền qua. CTFL v4.0.1 chỉ nêu «defect và failure» là một thuộc tính phân biệt test level, không đưa danh sách — dùng như khung tư duy, không phải nội dung thi.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu và business rules

Tính năng: **báo giá chuyến đặt xe có voucher**. Các quy tắc chạy xuyên nhiều module — đó chính là nơi interface dễ vỡ:

| Mã | Quy tắc (tóm tắt) | Module chạm tới |
|---|---|---|
| R-GEN-01 | Bước trung gian của công thức cước tính chính xác, không làm tròn; chỉ bước cuối làm tròn | pricing, fare, voucher, money |
| R-GEN-02 | Quy tắc theo giờ/ngày dùng giờ Việt Nam UTC+7, bất kể múi giờ máy chủ | quote, time, kho chuyến |
| R-FARE-03, R-FARE-06 | Thứ tự 6 bước; voucher tính trên số tiền sau cao điểm và mưa | pricing, fare, voucher |
| R-VOUCHER-02 | Mã bỏ khoảng trắng, viết hoa; chuỗi rỗng = không có voucher | API, quote, voucher |
| R-VOUCHER-06 | A &lt; `minAmount` ⇒ `VOUCHER_MIN_AMOUNT`; giảm = min(percent% × A, trần) | fare, voucher |
| R-VOUCHER-07, R-VOUCHER-08 | Lượt dùng trong ngày; nhiều lỗi thì báo theo thứ tự cố định, `VOUCHER_MIN_AMOUNT` trước `VOUCHER_USAGE_LIMIT` | quote, pricing, kho chuyến |

### 3.2. System context

- **Hệ thống đích** (kiến trúc §3.3, §7): Booking gọi Pricing qua **gRPC/Protobuf** vì cần báo giá phản hồi ngay; `.proto` được quản lý như hợp đồng, có deadline bắt buộc và quy ước mã lỗi (`INVALID_ARGUMENT`, `FAILED_PRECONDITION`, `DEADLINE_EXCEEDED`…). Promotion Service sở hữu voucher. Pipeline §17 có hẳn bước **contract test** «Booking ↔ Pricing gRPC».
- **Lab** (SPEC R-ARCH-01): modular monolith — Pricing &amp; Quote, Promotion là các module trong cùng tiến trình; «interface» là lời gọi hàm TypeScript. Không có gRPC, nên lab chỉ minh hoạ được defect về dữ liệu và thứ tự gọi, không minh hoạ được deadline hay mất kết nối.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 214" role="img" aria-label="Trong khung hệ thống Tới Nơi: Booking gọi Pricing qua gRPC, Pricing hỏi Promotion, Booking phát BookingCreated cho Dispatch — component integration. Ngoài khung: MapViet, VíXanh, TinNhanh — system integration, TEST-015.">
<defs>
<marker id="t013-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="t013-out" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<rect x="12" y="14" width="640" height="186" rx="10" class="f-bg s-muted dash"/>
<text x="28" y="34" class="t-b">Hệ thống Tới Nơi</text>
<text x="636" y="34" text-anchor="end" class="t-small t-pk">interface bên trong ⇒ component integration</text>
<rect x="36" y="58" width="150" height="48" rx="8" class="f-pk-soft s-pk"/><text x="111" y="87" text-anchor="middle" class="t-b">Booking</text>
<rect x="276" y="58" width="150" height="48" rx="8" class="f-pk-soft s-pk"/><text x="351" y="87" text-anchor="middle" class="t-b">Pricing</text>
<rect x="490" y="58" width="140" height="48" rx="8" class="f-pk-soft s-pk"/><text x="560" y="87" text-anchor="middle" class="t-b">Promotion</text>
<rect x="36" y="140" width="150" height="44" rx="8" class="f-pk-soft s-pk"/><text x="111" y="167" text-anchor="middle" class="t-b">Dispatch</text>
<rect x="276" y="140" width="150" height="44" rx="8" class="f-pk-soft s-pk"/><text x="351" y="167" text-anchor="middle" class="t-b">Payment</text>
<rect x="490" y="140" width="140" height="44" rx="8" class="f-pk-soft s-pk"/><text x="560" y="167" text-anchor="middle" class="t-b">Notification</text>
<line x1="186" y1="82" x2="272" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#t013-arrow)"/><text x="229" y="74" text-anchor="middle" class="t-small">gRPC GetQuote</text>
<line x1="426" y1="82" x2="486" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#t013-arrow)"/><text x="456" y="74" text-anchor="middle" class="t-small">voucher</text>
<line x1="111" y1="106" x2="111" y2="136" class="s-pk dash" stroke-width="1.6" marker-end="url(#t013-arrow)"/><text x="120" y="126" class="t-small">Kafka BookingCreated</text>
<rect x="716" y="40" width="190" height="40" rx="8" class="f-warn-soft s-warn"/><text x="811" y="65" text-anchor="middle" class="t-b">MapViet</text>
<rect x="716" y="100" width="190" height="40" rx="8" class="f-warn-soft s-warn"/><text x="811" y="125" text-anchor="middle" class="t-b">VíXanh</text>
<rect x="716" y="160" width="190" height="40" rx="8" class="f-warn-soft s-warn"/><text x="811" y="185" text-anchor="middle" class="t-b">TinNhanh</text>
<path d="M426 66 L560 44 L712 58" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#t013-out)"/>
<path d="M426 150 L470 124 L712 120" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#t013-out)"/>
<line x1="630" y1="180" x2="712" y2="180" class="s-warn dash" stroke-width="1.4" marker-end="url(#t013-out)"/>
<text x="811" y="24" text-anchor="middle" class="t-small t-warn">hệ thống ngoài ⇒ TEST-015</text>
</svg>
</div>
<figcaption><b>Hình 1 · Bên trong và bên ngoài ranh giới hệ thống.</b> Mũi tên liền và nét đứt màu chính (gRPC, Kafka) là interface giữa các component của Tới Nơi — đối tượng của bài này. Mũi tên ra khung phải là interface với hệ thống ngoài — system integration testing. Lab gộp Pricing, Promotion, Booking vào một tiến trình nên chỉ có các mũi tên bên trong, dạng lời gọi hàm.</figcaption>
</figure>

### 3.3. Rủi ro

Tiền đổi đơn vị giữa đường (đồng ↔ milli-đồng); làm tròn trước bước voucher; hai bên không thống nhất ai chuẩn hoá đầu vào (app di động gửi `""`); một module tự tính giờ thay vì dùng `time.ts`; gọi sai thứ tự làm thông báo lỗi sai. Hậu quả từng rủi ro được đo cụ thể ở bảng cuối mục 5.

## 4. Phân tích kỹ thuật

### 4.1. Xác định interface cần kiểm

Đọc tầng điều phối (`priceQuote`), liệt kê **mỗi lần dữ liệu đi qua ranh giới module** — tham số, giá trị trả về, lỗi ném ra, **thứ tự** lời gọi — và ghi đơn vị/định dạng, ai chịu trách nhiệm kiểm.

| Mã | Interface (bên gọi → bên được gọi) | Dữ liệu và điểm cần chú ý |
|---|---|---|
| GD-1 | `quote.ts` → `pricing.normalizePriceRequest` | Quãng đường làm tròn 0,1 km; ném 5 mã lỗi theo R-VALID-01 |
| GD-2 | `quote.ts` → `voucher.normalizeVoucherCode` | Chuỗi khách gõ (`""`, `null`, chữ thường) ⇒ mã chuẩn hoặc `null` |
| GD-3 | `quote.ts` → cổng `VoucherRepository.findByCode` | Tra theo mã **đã chuẩn hoá**; `null` ⇒ `VOUCHER_NOT_FOUND` |
| GD-4 | `quote.ts` → `voucher.assertVoucherEligible` | Ngày theo giờ VN, `kind`, `cityCode` |
| GD-5 | `quote.ts` → `time.isPeakTime`, cổng `WeatherService.isRaining` | Thời điểm UTC tuyệt đối; giờ VN tính trong `time.ts` |
| GD-6 | `quote.ts` → `pricing.priceRequest` → `fare.calculateFare` → `voucher.voucherDiscountMilli` | Bên trong tính bằng **milli-đồng**; `FareBreakdown` trả ra **đồng** (có thể lẻ); `VoucherTerms` ghi bằng đồng |
| GD-7 | `quote.ts` → `assertVoucherQuota` → `time.vnDayRange` + cổng `TripRepository.countVoucherUses` | Gọi **sau** `priceRequest` để giữ thứ tự lỗi R-VOUCHER-08 |

GD-6 đáng chú ý nhất: `src/domain/money.ts` cho thấy công thức cước chạy bằng số nguyên milli-đồng (để ×1,25 không sinh sai số), còn bảng chi tiết trả ra đồng. Một interface, hai đơn vị.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="API gọi quote.ts; quote.ts gọi pricing.ts (GD-1, GD-6; D1, D2), voucher.ts (GD-2, GD-4; D3), time.ts (GD-5; D4) và các cổng kho, thời tiết (GD-3, GD-7; D5); pricing.ts gọi fare.ts (milli-đồng), fare.ts gọi voucherDiscountMilli.">
<defs>
<marker id="t013-a2" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<rect x="16" y="122" width="160" height="56" rx="8" class="f-surface s-line"/><text x="96" y="146" text-anchor="middle" class="t-b">API /api/quotes</text><text x="96" y="165" text-anchor="middle" class="t-small">zod: voucherCode chuỗi</text>
<rect x="232" y="114" width="190" height="72" rx="8" class="f-pk-soft s-pk"/><text x="327" y="142" text-anchor="middle" class="t-b">quote.ts</text><text x="327" y="162" text-anchor="middle" class="t-mono t-small">priceQuote()</text>
<line x1="176" y1="150" x2="228" y2="150" class="s-pk" stroke-width="1.6" marker-end="url(#t013-a2)"/>
<rect x="540" y="24" width="200" height="54" rx="8" class="f-surface s-line"/><text x="640" y="46" text-anchor="middle" class="t-b">pricing.ts</text><text x="640" y="65" text-anchor="middle" class="t-small">normalize · priceRequest</text>
<rect x="780" y="24" width="124" height="54" rx="8" class="f-surface s-line"/><text x="842" y="46" text-anchor="middle" class="t-b">fare.ts</text><text x="842" y="65" text-anchor="middle" class="t-small t-warn">milli-đồng</text>
<rect x="540" y="96" width="200" height="54" rx="8" class="f-surface s-line"/><text x="640" y="118" text-anchor="middle" class="t-b">voucher.ts</text><text x="640" y="137" text-anchor="middle" class="t-small">chuẩn hoá · eligible · lượt</text>
<rect x="540" y="168" width="200" height="54" rx="8" class="f-surface s-line"/><text x="640" y="190" text-anchor="middle" class="t-b">time.ts</text><text x="640" y="209" text-anchor="middle" class="t-small">isPeakTime · vnDayRange</text>
<rect x="540" y="240" width="364" height="44" rx="8" class="f-bg s-muted dash"/><text x="722" y="267" text-anchor="middle" class="t-small">cổng: kho voucher · kho chuyến · thời tiết (bộ nhớ / giả)</text>
<line x1="422" y1="134" x2="536" y2="56" class="s-pk" stroke-width="1.6" marker-end="url(#t013-a2)"/>
<line x1="422" y1="146" x2="536" y2="123" class="s-pk" stroke-width="1.6" marker-end="url(#t013-a2)"/>
<line x1="422" y1="160" x2="536" y2="192" class="s-pk" stroke-width="1.6" marker-end="url(#t013-a2)"/>
<line x1="422" y1="174" x2="536" y2="258" class="s-pk" stroke-width="1.6" marker-end="url(#t013-a2)"/>
<line x1="740" y1="51" x2="776" y2="51" class="s-pk" stroke-width="1.6" marker-end="url(#t013-a2)"/>
<path d="M842 78 L842 123 L744 123" class="f-none s-pk" stroke-width="1.6" marker-end="url(#t013-a2)"/>
<text x="530" y="46" text-anchor="end" class="t-small t-warn">GD-1,6 · D1 D2</text>
<text x="530" y="150" text-anchor="end" class="t-small t-warn">GD-2,4 · D3</text>
<text x="530" y="210" text-anchor="end" class="t-small t-warn">GD-5 · D4</text>
<text x="530" y="280" text-anchor="end" class="t-small t-warn">GD-3,7 · D5</text>
<text x="750" y="142" class="t-small t-mono">voucherDiscountMilli</text>
</svg>
</div>
<figcaption><b>Hình 2 · Bản đồ interface của chuỗi báo giá trong lab.</b> Mỗi mũi tên là một chỗ nối cần kiểm; nhãn cam ghi mã interface (GD-n) và defect giao tiếp minh hoạ ở mục 5 (D1–D5). Cổng kho và thời tiết dùng bản trong bộ nhớ/bản giả, nên test ở đây tích hợp các module nghiệp vụ thật, chưa tích hợp database.</figcaption>
</figure>

### 4.2. Từ loại defect tới defect cụ thể

Ghép khung «defect tích hợp điển hình» (callout mục 2.4) với bảng interface:

| Loại defect giao tiếp | Interface | Defect cụ thể có thể xảy ra |
|---|---|---|
| Giả định sai về **đơn vị** dữ liệu | GD-6 | D1: lấy `fare.beforeDiscount` (đồng) đưa cho `voucherDiscountMilli` (cần milli-đồng) |
| Dữ liệu bị **biến đổi** giữa đường | GD-6 | D2: đưa cho voucher tổng đã làm tròn lên 1.000 đ (trái R-GEN-01, R-FARE-06) |
| Interface không khớp — **ai chuẩn hoá** | GD-2, GD-3 | D3: tầng ứng dụng chuyển thẳng `""` hay `" giam20 "` cho kho voucher |
| Giả định sai về **ý nghĩa** dữ liệu | GD-5 | D4: tự tính cao điểm bằng giờ máy chủ thay vì giờ VN (R-GEN-02) |
| Gọi interface **sai thứ tự** | GD-6, GD-7 | D5: hỏi lượt dùng trước khi tính giá ⇒ báo sai mã lỗi (R-VOUCHER-08) |

### 4.3. Test condition và dữ liệu chọn để lộ defect

Ca integration tốt chọn dữ liệu sao cho **kết quả đúng và sai khác nhau**. Với GIAM20 ở 7,4 km, làm tròn sớm hay muộn đều ra 52.000 đ vì bước làm tròn cuối «nuốt» chênh lệch — không bắt được D2. Phải chọn ca ngay biên `minAmount` của HN10: 6,1 km xe máy ở Hà Nội ngoài cao điểm = 12.000 + 4.500 × 6,1 = **39.450 đ** &lt; 40.000 đ; làm tròn sớm thành 40.000 đ là vượt biên.

| Ca | Interface | Test condition | Dữ liệu | Kết quả mong đợi |
|---|---|---|---|---|
| IF-1 | GD-6 | Tiền đi qua pricing → fare → voucher giữ đúng đơn vị | Oracle R-FARE-09: SG, xe máy, 7,4 km, 11:30, mưa, GIAM20 | Giảm 12.925 đ, tổng 52.000 đ |
| IF-2 | GD-6 | Voucher nhận số tiền **chưa** làm tròn | HN, xe máy, 6,1 km, 09:00, HN10 | `VOUCHER_MIN_AMOUNT` |
| IF-3 | GD-2, GD-3 | Mã từ API được chuẩn hoá trước khi tra kho | `voucherCode` = `""` và `" giam20 "` | Không voucher, 46.000 đ; GIAM20, 37.000 đ |
| IF-4 | GD-5 | Giờ cao điểm theo giờ VN | 11:30 giờ VN (04:30 UTC), 7,4 km | `peak = true`, 57.000 đ |
| IF-5 | GD-6, GD-7 | Thứ tự lỗi khi vi phạm cùng lúc | C001 đã dùng HN10 hôm nay, báo giá lại 6,1 km | `VOUCHER_MIN_AMOUNT`, không phải `VOUCHER_USAGE_LIMIT` |

## 5. Thực hành

### 5.1. Đề bài

*Liệt kê interface giữa `quote.ts` và `pricing.ts`/`voucher.ts`, rồi viết 5 defect giao tiếp có thể xảy ra* (đơn vị tiền, làm tròn giữa chừng trái R-GEN-01, voucher rỗng, sai giờ VN R-GEN-02, một defect tự chọn). Lời giải: bảng GD-1…GD-7 (mục 4.1) và D1–D5 (mục 4.2). Phần dưới đi thêm một bước: cài đặt từng defect và chứng minh ca integration bắt được.

### 5.2. Năm bản «ghép lỗi»

Tệp `buggy-quote.ts` trong thư mục bài chứa năm phiên bản của `priceQuote`. **Không component nào của `src/domain` bị sửa** — mỗi bản chỉ ghép sai một mắt xích. Ví dụ D1, một lần «tái cấu trúc» chuyển việc trừ voucher lên tầng ứng dụng:

```ts
const { fare } = priceRequest(request, { peak, raining, voucher: null });
// FareBreakdown trả ĐỒNG, còn voucherDiscountMilli nhận MILLI-ĐỒNG
const discountMilli = voucher ? voucherDiscountMilli(voucher, fare.beforeDiscount) : 0;
```

D2 chỉ khác một dòng: `voucherDiscountMilli(voucher, toMilli(fare.total))` — đơn vị đã đúng, nhưng `fare.total` là số đã nâng mức tối thiểu và làm tròn lên 1.000 đ. D3 bỏ `normalizeVoucherCode`, D4 thay `isPeakTime` bằng một hàm dùng `getUTCHours()` (mô phỏng container chạy múi giờ UTC), D5 đưa `assertVoucherQuota` lên trước `priceRequest`.

### 5.3. Bộ test và kết quả

`quote-integration.test.ts` có ba phần: (0) bốn ca component chứng minh từng component đúng khi kiểm riêng; (1) năm ca IF-1…IF-5 chạy trên `src/` thật — `createMemorySystem` với kho trong bộ nhớ, thời tiết giả, đồng hồ cố định; (2) **cùng** năm ca đó chạy trên năm bản ghép lỗi bằng `it.fails` (ca phải thất bại thì mới tính là đạt), kèm ca ghi lại failure quan sát được.

Chạy trong `examples/toi-noi-testing` (Node 22, vitest 3.2.7):

```bash
npx vitest run articles/component-integration-testing --reporter=verbose
```

```text
✓ TEST-013 · 0 · từng component đúng khi kiểm riêng (component testing) > fare.ts: oracle R-FARE-09 ⇒ 52.000 đ
  … (3 ca component còn lại đều ✓)
✓ TEST-013 · 1 · ca integration trên src/ thật > IF-1 · quote → pricing → voucher: oracle R-FARE-09 đi qua cả chuỗi ⇒ 52.000 đ, giảm 12.925 đ
  … (IF-2 … IF-5 đều ✓)
✓ TEST-013 · 2 · cùng ca trên bản ghép lỗi > D1-don-vi-tien ⇒ IF-1 đỏ
✓ … > D2-lam-tron-giua-chung ⇒ IF-2 đỏ
✓ … > D3-ma-chua-chuan-hoa ⇒ IF-3 đỏ
✓ … > D4-gio-may-chu ⇒ IF-4 đỏ
✓ … > D5-thu-tu-kiem-tra ⇒ IF-5 đỏ
✓ … > failure D1: giảm 12,925 đ thay vì 12.925 đ ⇒ khách trả 65.000 đ
  … (4 ca failure D2–D5 đều ✓)

 Test Files  1 passed (1)
      Tests  19 passed (19)
```

(Lược tên tệp, thời gian và các dòng lặp.) Tổng hợp kết quả mong đợi so với thực tế trên bản lỗi:

| Defect | Ca bắt được | Mong đợi (src/ thật) | Thực tế trên bản lỗi | Hậu quả nghiệp vụ |
|---|---|---|---|---|
| D1 đơn vị tiền | IF-1 | Giảm 12.925 đ, tổng 52.000 đ | Giảm 12,925 đ, tổng 65.000 đ | Voucher gần như vô tác dụng, khách trả thêm 13.000 đ |
| D2 làm tròn giữa chừng | IF-2 | `VOUCHER_MIN_AMOUNT` | Nhận HN10, tổng 36.000 đ | Voucher dùng được dưới mức tối thiểu |
| D3 mã chưa chuẩn hoá | IF-3 | Không voucher, 46.000 đ | `VOUCHER_NOT_FOUND` | Khách không nhập gì vẫn bị báo lỗi |
| D4 giờ máy chủ | IF-4 | Cao điểm, 57.000 đ | Ngoài cao điểm, 46.000 đ | Mất phụ phí cao điểm mỗi trưa |
| D5 thứ tự gọi | IF-5 | `VOUCHER_MIN_AMOUNT` | `VOUCHER_USAGE_LIMIT` | Thông báo sai: khách tưởng ngày mai dùng được |

Điểm mấu chốt nằm ở phần 0: **bốn ca component đều xanh trên cả năm bản lỗi**, vì các bản lỗi dùng chính các component đó. Chỉ ca đi xuyên qua chỗ nối mới đỏ.

## 6. Góc nhìn nâng cao

- **«Integration test» trong mã nguồn không đồng nghĩa với test level này.** Kiến trúc §17.1 dùng chữ «Component» cho «Booking service + PostgreSQL/Redis test instance» — đó là cách gọi ngành (kiểm một service cùng hạ tầng của nó), khác component testing của ISTQB. Khi trao đổi, hỏi lại: test object là gì, interface nào được kiểm?
- **Mock quá nhiều thì không còn tích hợp.** Thay `priceRequest` bằng mock trả sẵn 52.000 đ là D1, D2 biến mất. Bản giả chỉ dùng cho phần **ngoài** phạm vi tích hợp đang kiểm (ở đây: kho, thời tiết).
- **Chiến lược tích hợp quyết định thứ tự và bản giả.** Big-bang ghép tất cả rồi mới test — khó định vị lỗi. Top-down bắt đầu từ `quote.ts`, thay domain chưa xong bằng stub; bottom-up bắt đầu từ `fare.ts`/`voucher.ts`, cần driver gọi chúng. Syllabus chỉ nêu tên; cách chọn và ưu nhược ở Series 07 (TEST-066 … TEST-069).
- **Microservices làm interface thành hợp đồng.** Trong lab, đặt kiểu tiền riêng (`Milli`, `Dong`) sẽ biến D1 thành lỗi biên dịch. Qua gRPC thì không: đổi nghĩa trường `fare` từ đồng sang nghìn đồng trong `.proto` vẫn biên dịch được — lý do pipeline §17 có contract test (TEST-074, Series 11).
- **Lỗi giao tiếp không chỉ là dữ liệu sai.** Ở hệ thống đích còn có Pricing quá deadline, hay Booking chỉ xử lý `INVALID_ARGUMENT` mà quên `FAILED_PRECONDITION`. Lab không có mạng nên không minh hoạ được — đây là giới hạn của lab.
- **Sai lầm thường gặp:** chỉ test «đường vui» qua interface (IF-1) mà quên đường lỗi (IF-5); chọn dữ liệu mà bước làm tròn cuối che mất chênh lệch; coi kiểu dữ liệu TypeScript là đủ bằng chứng hai bên hiểu cùng một nghĩa.

## 7. Bài tập và câu hỏi ôn tập

### Bài tập

**Bài 1.** Liệt kê interface của `createTrip` (`src/app/trips.ts`) với kho báo giá, voucher và kho chuyến, rồi nêu ba defect giao tiếp có thể xảy ra (gợi ý: R-QUOTE-03, R-QUOTE-04, R-VOUCHER-07).

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Interface: <code>quotes.findById</code>; so <code>now</code> với <code>expiresAt</code>; <code>vouchers.findByCode</code> + <code>assertVoucherQuota</code> (kiểm lượt lần hai); <code>quotes.markUsed</code> có điều kiện; <code>trips.create</code>. Defect: (1) lấy <code>fare</code> từ một lần tính lại giá thay vì <code>quote.details</code> ⇒ đổi giá hồi tố (R-QUOTE-04); (2) kiểm lượt theo ngày của <code>quote.createdAt</code> thay vì thời điểm tạo chuyến ⇒ báo giá 23:58, đặt 00:01 tính sai ngày (R-VOUCHER-07); (3) gọi <code>trips.create</code> trước <code>markUsed</code> ⇒ hai yêu cầu đồng thời tạo hai chuyến từ một báo giá (R-QUOTE-03).</p>
</details>

**Bài 2.** Phân loại mỗi kiểm thử sau vào component, component integration hay system integration: (a) `parcelSurcharge(6.1)` = 4.000 đ; (b) `priceQuote` với kho voucher trong bộ nhớ, kiểm HN10 bị từ chối ở TP.HCM; (c) Booking gọi Pricing qua gRPC trên môi trường test, kiểm deadline 300 ms; (d) Payment gọi sandbox VíXanh, kiểm trừ tiền một lần khi thử lại.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(a) component — một hàm, đầu vào cho sẵn. (b) component integration — đi qua GD-2, GD-3, GD-4. (c) component integration — hai service của cùng hệ thống Tới Nơi; kiểm deadline là khía cạnh phi chức năng của interface. (d) system integration — VíXanh là hệ thống ngoài (TEST-015).</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Theo CTFL v4.0.1, component integration testing tập trung vào điều gì?

- a) Hành vi đầu-cuối của toàn bộ hệ thống so với đặc tả hệ thống
- b) Interface và tương tác giữa các component
- c) Từng component được kiểm cô lập trong môi trường phát triển
- d) Interface giữa hệ thống đang kiểm và hệ thống bên ngoài

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b> (mục 2.2.1). a) là system testing. c) là component testing. d) là system integration testing.</p>
</details>

**Câu 2 (K2).** Booking Service của Tới Nơi gọi Pricing Service qua gRPC để lấy báo giá. Kiểm thử interface này thuộc test level nào?

- a) Component testing, vì mỗi service là một component
- b) Component integration testing, vì đây là interface giữa hai component của cùng hệ thống
- c) System integration testing, vì hai service chạy trên hai tiến trình khác nhau
- d) Acceptance testing, vì giá ảnh hưởng trực tiếp tới khách

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng.</b> a) sai: đối tượng là chỗ nối, không phải một service riêng lẻ. c) sai: ranh giới system integration là hệ thống ngoài (VíXanh, MapViet), không phải ranh giới tiến trình. d) sai: acceptance tập trung vào xác nhận nhu cầu nghiệp vụ, không phải interface kỹ thuật.</p>
</details>

**Câu 3 (K2).** `fare.ts` tính bằng milli-đồng, còn bảng chi tiết cước trả ra đồng. Một developer đưa giá trị đồng vào hàm cần milli-đồng. Mọi unit test của hai component vẫn đạt. Phát biểu nào đúng nhất?

- a) Đây là failure do môi trường, không phải defect
- b) Đây là defect giao tiếp; component integration testing là test level phù hợp để phát hiện sớm
- c) Chỉ acceptance testing mới phát hiện được vì cần khách hàng xác nhận giá
- d) Unit test đạt chứng tỏ không có defect

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b> — giả định sai về đơn vị dữ liệu qua interface. a) sai: lỗi nằm trong mã ghép nối. c) sai: system hay acceptance testing có thể thấy failure, nhưng muộn và khó định vị hơn. d) sai: unit test kiểm từng component cô lập, không kiểm chỗ nối.</p>
</details>

**Câu 4 (K2).** Ca integration nào phù hợp NHẤT để phát hiện defect «đưa số tiền đã làm tròn lên 1.000 đ cho bước voucher»?

- a) GIAM20, 7,4 km, cao điểm, mưa — kỳ vọng 52.000 đ
- b) Không voucher, 7,4 km — kỳ vọng 46.000 đ
- c) HN10, 6,1 km ở Hà Nội ngoài cao điểm (39.450 đ) — kỳ vọng `VOUCHER_MIN_AMOUNT`
- d) Quãng đường 0,44 km — kỳ vọng `DISTANCE_OUT_OF_RANGE`

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng</b>: dữ liệu nằm ngay dưới biên <code>minAmount</code>, làm tròn sớm đẩy nó vượt biên nên kết quả đúng và sai khác nhau. a) sai: bước làm tròn cuối che chênh lệch, cả hai bản đều ra 52.000 đ. b) không đi qua voucher. d) dừng ở <code>normalizePriceRequest</code>, chưa tới bước voucher.</p>
</details>

**Câu 5 (K2).** Theo CTFL v4.0.1, phát biểu nào về trách nhiệm và vị trí của component integration testing là đúng?

- a) Thường do đội acceptance thực hiện trên môi trường production
- b) Thường do developer thực hiện; trong testing quadrants nằm ở Q1 cùng component test, nên được tự động hoá trong CI
- c) Bắt buộc do đội test độc lập thực hiện sau system testing
- d) Nằm ở Q3 vì hướng nghiệp vụ và phê bình sản phẩm

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b> — mục 1.5.3 nêu ví dụ developer làm component và component integration testing; mục 5.1.7 xếp hai loại này vào Q1. a) và c) sai về người làm và thời điểm. d) sai: Q3 chứa exploratory, usability, UAT.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — mục 2.2 Test Levels and Test Types, 2.2.1 Test Levels (FL-2.2.1); mục 1.5.3 Independence of Testing; mục 5.1.6 Test Pyramid, 5.1.7 Testing Quadrants. PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (component integration testing, interface, integration strategy, system integration testing).
- ISO/IEC/IEEE 29119-1, *Software and systems engineering — Software testing — Part 1: General concepts* — khái niệm test level.
- Tài liệu kiến trúc Tới Nơi (nội bộ blog): §3.3 giao tiếp đồng bộ/bất đồng bộ, §7 gRPC, §17 pipeline, §17.1 chiến lược test theo kiến trúc.
- Phòng lab: `SPEC.md` (R-GEN, R-FARE, R-VOUCHER, R-ARCH) và bộ test của bài — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-004 Error, Defect, Failure](/articles/error-defect-failure-root-cause) · [TEST-009 Vai trò trong kiểm thử](/articles/vai-tro-trong-kiem-thu) (độc lập kiểm thử) · [TEST-011 Tổng quan 5 test level](/articles/tong-quan-5-test-level-istqb) · [TEST-015 System Integration Testing](/articles/system-integration-testing)

Bài tiếp theo: [TEST-014 · System Testing](/articles/system-testing).
