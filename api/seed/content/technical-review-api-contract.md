---
slug: technical-review-api-contract
title: "Technical Review: review hợp đồng API (OpenAPI, Protobuf) giữa các service Tới Nơi"
summary: "Technical review theo CTFL v4.0.1 và vai trò Technical Test Analyst theo CTAL-TTA v4.0, áp vào bản nháp OpenAPI POST /api/trips và Protobuf GetQuote Booking → Pricing: mã lỗi, tiền, deadline, idempotency, tương thích."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, ctal-tta, test-025, static-testing, technical-review, api-contract, openapi, protobuf]
language: vi
---

Một dòng sai trong hợp đồng API không chỉ là một defect: mọi đội tiêu thụ hợp đồng đó sẽ viết code theo dòng sai ấy. Bản nháp OpenAPI của `POST /api/trips` ghi thành công là `200` trong khi đặc tả nói `201`; bản Protobuf mới của Pricing tái dùng số field `5` cho một kiểu khác. Cả hai lỗi đều tìm được **trước khi có dòng code nào chạy** — bằng một buổi **technical review**. Bài này trả lời: technical review khác walkthrough ở đâu, Technical Test Analyst mang gì vào buổi review, và review một API contract theo tiêu chí kỹ thuật nào.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-025</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S03 · Static Testing &amp; Review Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1; Advanced — CTAL-TTA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Technical review (một review type của static testing)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/informal-review">TEST-023 · Informal Review</a> (nên đọc thêm <a href="/articles/review-process-va-cac-vai-tro">TEST-027 · Review Process và các vai trò</a>, <a href="/articles/walkthrough">TEST-024 · Walkthrough</a>)</td></tr>
<tr><th scope="row">Thực hành</th><td>Technical review bản nháp OpenAPI <code>POST /api/trips</code> (R-API-08, R-API-02, R-API-06) và Protobuf <code>GetQuote</code> Booking → Pricing (kiến trúc §7): mã lỗi, tiền nguyên đồng, deadline, idempotency key, tương thích phiên bản; bằng chứng chạy bằng vitest ở <code>articles/technical-review-api-contract/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 3 · mục 3.2.4 · FL-3.2.4 (K2) · CTAL-TTA v4.0 · Chương 5 · mục 5.1 · TTA-5.1.1 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Tới Nơi chia hệ thống thành nhiều service. Hai service nói chuyện với nhau qua **hợp đồng API** (API contract): OpenAPI cho REST mà app gọi, Protobuf cho gRPC nội bộ (kiến trúc §12.3). Hợp đồng là work product có **nhiều người tiêu thụ**: app iOS, Android, web, Booking… Lỗi trong hợp đồng vì thế được nhân bản theo số đội, và sửa sau khi các đội đã code xong tốn hơn nhiều so với sửa trên giấy.

**Technical review** là loại review do **người có chuyên môn kỹ thuật** thực hiện, **moderator** dẫn dắt, nhằm **đạt đồng thuận và ra quyết định** về một vấn đề kỹ thuật. Nên dùng khi work product là hợp đồng, thiết kế, lược đồ dữ liệu mà nhiều đội sẽ dựa vào; khi có câu hỏi cần **quyết định** (kiểu tiền, chính sách phiên bản, ai kiểm chủ báo giá) chứ không chỉ cần sửa; và khi cần ý kiến của cả bên cung cấp lẫn bên tiêu thụ.

Không phải lựa chọn tốt khi mục tiêu chính là đào tạo người mới (walkthrough — [TEST-024](/articles/walkthrough)) hay tìm tối đa defect kèm metric quy trình (inspection — TEST-026).

## 2. Lý thuyết theo ISTQB

### 2.1. Technical review trong CTFL v4.0.1 (mục 3.2.4)

Diễn đạt lại mô tả của syllabus:

- **Ai thực hiện:** reviewer có trình độ kỹ thuật phù hợp.
- **Ai dẫn:** một **moderator** — không phải tác giả (khác walkthrough).
- **Mục tiêu:** chính là **đạt đồng thuận và ra quyết định** về một vấn đề kỹ thuật; kèm theo là phát hiện anomaly, đánh giá chất lượng và tạo niềm tin vào work product, nảy ý tưởng mới, tạo động lực và giúp tác giả cải thiện.

Technical review vẫn đi qua năm hoạt động của quy trình review chung (3.2.2) ở mức hình thức cao hơn walkthrough; vai trò theo 3.2.3. Chi tiết: [TEST-027](/articles/review-process-va-cac-vai-tro). Áp vào buổi review hợp đồng:

| Vai trò (3.2.3) | Ở buổi review hợp đồng Booking/Pricing |
|---|---|
| Manager | Engineering Manager khối Booking–Pricing: quyết định review, dành thời gian |
| Author | developer Booking (OpenAPI), developer Pricing (Protobuf) |
| Moderator, review leader | Tech Lead nền tảng: chọn người, lịch, phạm vi; điều phối, giữ thời gian, giữ không khí an toàn |
| Reviewer | developer mobile (tiêu thụ REST), developer Booking (tiêu thụ gRPC), **Technical Test Analyst**, kỹ sư bảo mật |
| Scribe | một developer không phải tác giả: ghi anomaly và **quyết định** |

### 2.2. Technical Test Analyst trong review (CTAL-TTA v4.0, mục 5.1)

CTAL-TTA dành mục 5.1 cho nhiệm vụ của Technical Test Analyst (TTA) trong review. Các ý chính, diễn đạt lại:

1. TTA **tham gia chủ động** technical review và inspection, mang **góc nhìn vận hành, hành vi** mà developer dễ bỏ qua — ví dụ: «client gửi lại khi mạng chập chờn thì sao?».
2. TTA góp phần xây và duy trì **checklist review**, cung cấp thông tin **mức nghiêm trọng** của defect.
3. **Chuẩn bị là bắt buộc**, bất kể loại review. Chuẩn bị gồm: đọc work product, **đối chiếu tài liệu được tham chiếu** để kiểm tính nhất quán, và **xác định điều còn thiếu**. Thiếu thời gian chuẩn bị, buổi review dễ biến thành buổi **sửa câu chữ**.
4. Một review tốt: hiểu điều được viết, tìm điều **chưa được viết**, kiểm tính đúng về kỹ thuật, và kiểm sản phẩm được mô tả **nhất quán với các sản phẩm khác** đã có hoặc đang làm. Review không cô lập: phải xét tương tác của work product với phần còn lại của hệ thống.

Ý 3 chính là nội dung LO **TTA-5.1.1 (K2)**: giải thích vì sao chuẩn bị quan trọng với TTA. Ý 4 giải thích vì sao bài này đặt bản Protobuf v2 cạnh bản v1 **đang chạy**: tính tương thích chỉ thấy được khi so với sản phẩm đã có.

### 2.3. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa trong bài |
|---|---|
| Technical review | review do người có chuyên môn thực hiện, moderator dẫn, nhắm đồng thuận và quyết định kỹ thuật |
| Moderator (facilitator) | người điều phối buổi review: điều hoà, giữ thời gian, giữ môi trường an toàn |
| Technical Test Analyst | vai trò kiểm thử kỹ thuật của CTAL-TTA |
| API contract | hợp đồng API: mô tả request, response, lỗi, ràng buộc mà bên cung cấp và bên tiêu thụ cùng tuân theo |
| Breaking change | thay đổi phá vỡ tương thích: client cũ chạy sai hoặc lỗi |
| Idempotency key | khoá để gửi lại cùng một yêu cầu mà không tạo tác dụng hai lần |
| Deadline | hạn chót của một lời gọi gRPC; quá hạn thì bên gọi huỷ chờ |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB · kiến thức về hợp đồng API</span>
<p>Những quy tắc dưới đây là kiến thức kỹ thuật của OpenAPI/Protobuf, không thuộc ISTQB, nhưng là «chuyên môn» mà reviewer của technical review cần có:</p>
<p><b>Protobuf:</b> số field là danh tính trên đường truyền — không đổi kiểu, không tái dùng số của field đã xoá (khai <code>reserved</code>); enum nên có giá trị 0 mang nghĩa «chưa chỉ định» vì field vắng được đọc thành 0. <b>OpenAPI:</b> mã thành công, lược đồ lỗi, header bắt buộc là một phần của hợp đồng, không phải chú thích. <b>Công cụ:</b> có trình lint OpenAPI và trình kiểm breaking change cho Protobuf tự động bắt một phần lỗi dưới đây — đó là static analysis (TEST-029), không thay được phần quyết định của technical review. Kiểm hợp đồng khi chạy (contract testing) là dynamic testing, ở Series 07 và 11.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Hai work product đưa vào review

| Work product | Tác giả | Bên tiêu thụ | Lý do review |
|---|---|---|---|
| ① `openapi-trips.v0.yaml` — `POST /api/trips` | developer Booking | app iOS, Android, web | trước khi đội mobile sinh SDK |
| ② `pricing-quote.v2-draft.proto` — `GetQuote` | developer Pricing | Booking (đang dùng v1) | thêm dịch vụ giao đồ ăn vào báo giá |

Cả ba tệp (kèm v1 đang chạy) nằm ở [`articles/technical-review-api-contract/`](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/technical-review-api-contract). Bản nháp có lỗi gài sẵn, đánh dấu TR-xx và PQ-xx.

### 3.2. Test basis để đối chiếu

| Nguồn | Nội dung liên quan |
|---|---|
| R-API-08 | `POST /api/trips` (khách, cần `Idempotency-Key`), thân `{ quoteId, paymentMethod? }`, **client không gửi giá**, trả **201** `{ "trip": … }` |
| R-IDEM-01…05 | khoá 8–128 ký tự; gửi lại cùng nội dung ⇒ trả lại kết quả cũ kèm `Idempotent-Replayed: true`; khác nội dung ⇒ 422; đang xử lý ⇒ 409 |
| R-QUOTE-01…04 | báo giá có `expiresAt`, `pricingVersion`; không có hoặc của khách khác ⇒ 422 `QUOTE_NOT_FOUND`; hết hạn ⇒ 422; đã dùng ⇒ 409; giá chốt theo báo giá |
| R-API-02, -05, -06, -13, -14 | lỗi `{ "error": { code, message, details? } }`; không lộ báo giá của người khác; bảng mã → HTTP; 429 kèm `Retry-After`; chuỗi có giới hạn |
| R-GEN-01, R-FARE-02, R-PAY-01 | tiền là số nguyên đồng; quãng đường làm tròn 0,1 km half-up; phương thức `cash` hoặc `vixanh` |
| Kiến trúc §6.6, §7.2, §12.3, §14.1 | không đổi ý nghĩa field; deadline bắt buộc; status code gRPC; versioning; Booking/Quote API p95 < 500 ms (mục tiêu khởi tạo) |

### 3.3. System context

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 230" role="img" aria-label="Ngữ cảnh hai hợp đồng: Customer App gọi API Gateway và Booking qua REST theo hợp đồng OpenAPI POST /api/trips; Booking gọi Pricing qua gRPC theo hợp đồng Protobuf GetQuote; Booking phát sự kiện BookingCreated lên Kafka, ngoài phạm vi bài.">
<defs>
<marker id="t025-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="70" width="150" height="56" rx="8" class="f-surface s-line"/>
<text x="95" y="94" text-anchor="middle" class="t-b">Customer App</text>
<text x="95" y="112" text-anchor="middle" class="t-10">iOS · Android · web</text>
<rect x="250" y="70" width="160" height="56" rx="8" class="f-surface s-line"/>
<text x="330" y="94" text-anchor="middle" class="t-b">API Gateway</text>
<text x="330" y="112" text-anchor="middle" class="t-10">/ Customer BFF</text>
<rect x="490" y="70" width="150" height="56" rx="8" class="f-phys-soft s-phys"/>
<text x="565" y="102" text-anchor="middle" class="t-b t-phys">Booking</text>
<rect x="740" y="70" width="160" height="56" rx="8" class="f-pk-soft s-pk"/>
<text x="820" y="94" text-anchor="middle" class="t-b t-pk">Pricing &amp; Quote</text>
<text x="820" y="112" text-anchor="middle" class="t-10">MapViet, Promotion</text>
<line x1="170" y1="98" x2="248" y2="98" class="s-phys" stroke-width="2" marker-end="url(#t025-arrow)"/>
<line x1="410" y1="98" x2="488" y2="98" class="s-phys" stroke-width="2" marker-end="url(#t025-arrow)"/>
<line x1="640" y1="98" x2="738" y2="98" class="s-pk" stroke-width="2" marker-end="url(#t025-arrow)"/>
<rect x="150" y="18" width="360" height="34" rx="6" class="f-phys-soft s-phys"/>
<text x="330" y="40" text-anchor="middle" class="t-small t-phys">① REST/JSON · OpenAPI · POST /api/trips</text>
<rect x="600" y="18" width="300" height="34" rx="6" class="f-pk-soft s-pk"/>
<text x="750" y="40" text-anchor="middle" class="t-small t-pk">② gRPC · Protobuf · GetQuote</text>
<rect x="490" y="160" width="150" height="48" rx="8" class="f-bg s-muted dash"/>
<text x="565" y="182" text-anchor="middle" class="t-small">Kafka</text>
<text x="565" y="199" text-anchor="middle" class="t-10">BookingCreated (ngoài bài)</text>
<line x1="565" y1="126" x2="565" y2="158" class="s-muted dash" stroke-width="1.4" marker-end="url(#t025-arrow)"/>
</svg>
</div>
<figcaption><b>Hình 1 · Hai hợp đồng trong buổi review.</b> ① có bên tiêu thụ là các app; ② có bên tiêu thụ là Booking. Lab là một tiến trình, không có gRPC: endpoint ① có thật trong lab nên dùng để đối chiếu; ② chỉ có ở kiến trúc đích.</figcaption>
</figure>

### 3.4. Rủi ro nếu hợp đồng sai

| Lỗi hợp đồng có thể có | Hậu quả |
|---|---|
| Client gửi giá | khách sửa giá trong request ⇒ thất thu; vi phạm R-QUOTE-04 |
| Không bắt buộc `Idempotency-Key` | app gửi lại khi mạng chập chờn ⇒ hai chuyến, có thể thu tiền hai lần (kiến trúc §8.1) |
| Sai mã thành công / mã lỗi | SDK mobile sinh từ hợp đồng xử lý sai nhánh; chuyến tạo được mà app báo lỗi |
| Đổi kiểu hoặc tái dùng số field Protobuf | Booking v1 đọc nhầm dữ liệu của Pricing v2 mà không báo lỗi |
| Không deadline | Pricing chậm kéo Booking treo theo — anti-pattern «distributed monolith» (kiến trúc §21) |

## 4. Phân tích kỹ thuật

### 4.1. Chuẩn bị theo tinh thần TTA-5.1.1

Mỗi reviewer nhận hai bản nháp trước hai ngày làm việc kèm **danh sách tài liệu đối chiếu**. TTA dành 90 phút chuẩn bị (giả định) cho ba việc CTAL-TTA 5.1 nêu:

| Việc chuẩn bị | Làm gì với hợp đồng Tới Nơi | Tìm được gì |
|---|---|---|
| Đọc work product | đọc từng operation, từng field | TR-04, TR-05, TR-09 |
| Đối chiếu tài liệu tham chiếu | SPEC R-API/R-IDEM/R-QUOTE; kiến trúc §6.6, §7.2; **bản v1 đang chạy** | TR-06, PQ-01, PQ-02, PQ-08 |
| Tìm điều còn thiếu | hỏi «khi gửi lại, khi hết giờ, khi quá tải thì hợp đồng nói gì?» | TR-01, TR-07, TR-12, PQ-07 |

Hàng cuối là **góc nhìn hành vi** mà syllabus nói TTA mang vào: các phát hiện ở đó không nằm trên dòng nào của bản nháp — chúng là thứ **vắng mặt**.

### 4.2. Tiêu chí review kỹ thuật

| Mã | Tiêu chí | Câu hỏi reviewer đặt |
|---|---|---|
| C1 | Nhất quán với test basis | Field, giá trị enum, cấu trúc thân có khớp SPEC không? |
| C2 | Kiểu tiền và số đo | Tiền có là số nguyên đồng? Số đo có mất chính xác khi truyền? |
| C3 | Mã lỗi | Mỗi lỗi nghiệp vụ có mã HTTP/gRPC đúng và phân biệt được không? |
| C4 | Idempotency | Thao tác tạo có bắt buộc khoá? Hành vi gửi lại có được mô tả? |
| C5 | Deadline, timeout, retry | Lời gọi có hạn chót? Thao tác nào được thử lại? |
| C6 | Tương thích phiên bản | Bên tiêu thụ cũ còn đọc đúng không? Thay đổi phá vỡ có lên phiên bản mới? |
| C7 | Bảo mật đầu vào | Client có gửi được thứ không nên tin (giá)? Chuỗi có giới hạn? |
| C8 | Testability | Có kiểm được từng hành vi qua hợp đồng không (mã lỗi riêng, trường quan sát được)? |

### 4.3. So bản v2 với bản đang chạy

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="So sánh message Quote của Protobuf v1 đang chạy và v2 nháp theo số field: field 1, 2, 4, 6 giữ nguyên; field 3 đổi từ int64 total_dong sang double total là phá vỡ; field 5 đổi từ Timestamp expires_at sang bool raining là phá vỡ; field 7 đến 12 thêm mới là tương thích.">
<text x="45" y="30" text-anchor="middle" class="t-small t-b">số</text>
<text x="230" y="30" text-anchor="middle" class="t-small t-b">v1 — đang chạy (Booking đọc)</text>
<text x="570" y="30" text-anchor="middle" class="t-small t-b">v2 — bản nháp của Pricing</text>
<text x="820" y="30" text-anchor="middle" class="t-small t-b">kết luận</text>
<rect x="20" y="42" width="880" height="30" rx="4" class="f-surface s-line"/>
<text x="45" y="62" text-anchor="middle" class="t-mono">1, 2</text>
<text x="230" y="62" text-anchor="middle" class="t-mono">quote_id, customer_id</text>
<text x="570" y="62" text-anchor="middle" class="t-mono">quote_id, customer_id</text>
<text x="820" y="62" text-anchor="middle" class="t-small">giữ nguyên</text>
<rect x="20" y="80" width="880" height="30" rx="4" class="f-warn-soft s-warn"/>
<text x="45" y="100" text-anchor="middle" class="t-mono t-warn">3</text>
<text x="230" y="100" text-anchor="middle" class="t-mono">int64 total_dong</text>
<text x="570" y="100" text-anchor="middle" class="t-mono t-warn">double total</text>
<text x="820" y="100" text-anchor="middle" class="t-small t-warn t-b">PHÁ VỠ · PQ-01</text>
<rect x="20" y="118" width="880" height="30" rx="4" class="f-surface s-line"/>
<text x="45" y="138" text-anchor="middle" class="t-mono">4</text>
<text x="230" y="138" text-anchor="middle" class="t-mono">string pricing_version</text>
<text x="570" y="138" text-anchor="middle" class="t-mono">string pricing_version</text>
<text x="820" y="138" text-anchor="middle" class="t-small">giữ nguyên</text>
<rect x="20" y="156" width="880" height="30" rx="4" class="f-warn-soft s-warn"/>
<text x="45" y="176" text-anchor="middle" class="t-mono t-warn">5</text>
<text x="230" y="176" text-anchor="middle" class="t-mono">Timestamp expires_at</text>
<text x="570" y="176" text-anchor="middle" class="t-mono t-warn">bool raining</text>
<text x="820" y="176" text-anchor="middle" class="t-small t-warn t-b">PHÁ VỠ · PQ-02</text>
<rect x="20" y="194" width="880" height="30" rx="4" class="f-surface s-line"/>
<text x="45" y="214" text-anchor="middle" class="t-mono">6</text>
<text x="230" y="214" text-anchor="middle" class="t-mono">string voucher_code</text>
<text x="570" y="214" text-anchor="middle" class="t-mono">string voucher_code</text>
<text x="820" y="214" text-anchor="middle" class="t-small">giữ nguyên</text>
<rect x="20" y="232" width="880" height="30" rx="4" class="f-phys-soft s-phys"/>
<text x="45" y="252" text-anchor="middle" class="t-mono t-phys">7–12</text>
<text x="230" y="252" text-anchor="middle" class="t-small">—</text>
<text x="570" y="252" text-anchor="middle" class="t-mono">kind, distance_km, items, …</text>
<text x="820" y="252" text-anchor="middle" class="t-small t-phys">thêm mới · tương thích</text>
<text x="460" y="288" text-anchor="middle" class="t-10">Booking v1 vẫn đọc số 5 như Timestamp: byte của bool raining bị giải mã sai hoặc bị bỏ qua — không có lỗi nào báo ra.</text>
</svg>
</div>
<figcaption><b>Hình 2 · Đọc tương thích theo số field.</b> Số field là danh tính trên đường truyền; đổi kiểu ở cùng số hoặc tái dùng số cũ là thay đổi phá vỡ dù tên field trông «hợp lý».</figcaption>
</figure>

## 5. Thực hành

### 5.1. Trích bản nháp

Message `Quote` của bản Protobuf v2 đã tóm ở Hình 2 (tệp đầy đủ còn enum `ServiceKind` với `RIDE = 0` — PQ-05 — và hai dòng chú thích «không cần deadline», «hết hạn trả `total = 0`» — PQ-07, PQ-06). Dưới đây là phần chính của bản OpenAPI.

```yaml
# openapi-trips.v0.yaml (trích)
      parameters:
        - name: Idempotency-Key
          in: header
          required: false                      # TR-01
      requestBody:
        content:
          application/json:
            schema:
              required: [quoteId, fare]
              properties:
                fare: { type: number, format: double }   # TR-02
                paymentMethod: { type: string, enum: [cash, vixanh, card] }   # TR-03
      responses:
        '200': { $ref: '#/components/schemas/Trip' }    # TR-04, TR-05
        '404': { description: Không tìm thấy báo giá. } # TR-06
```

### 5.2. Nhật ký phát hiện

Mức nghiêm trọng: **Chặn** = không được phát hành hợp đồng; **Lớn** = phải sửa trước khi bên tiêu thụ code; **Nhỏ** = sửa trong lần cập nhật tới.

| Mã | Tiêu chí | Phát hiện | Căn cứ | Mức |
|---|---|---|---|---|
| TR-01 | C4 | `Idempotency-Key` không bắt buộc, không ràng buộc định dạng | R-IDEM-01 | Chặn |
| TR-02 | C7 | Client gửi `fare` (bắt buộc) | R-API-08, R-QUOTE-04 | Chặn |
| TR-03 | C1 | `paymentMethod` có `card` | R-PAY-01 | Lớn |
| TR-04 | C3 | Thành công là 200 | R-API-08: 201 | Lớn |
| TR-05 | C1 | Thân trả về là `Trip` trần, thiếu lớp bọc `{ trip }` | R-API-08 | Lớn |
| TR-06 | C3 | Báo giá không có ⇒ 404 | R-QUOTE-03, R-API-05: 422 `QUOTE_NOT_FOUND`, kể cả báo giá của khách khác | Lớn |
| TR-07 | C3, C4 | Thiếu 409 (`QUOTE_ALREADY_USED`, `IDEMPOTENCY_IN_PROGRESS`), 429 + `Retry-After`, 413 | R-API-06, R-API-13 | Lớn |
| TR-08 | C2 | Tiền `number/float` | R-GEN-01 | Lớn |
| TR-09 | C3 | Lược đồ lỗi phẳng `{ code, message }` | R-API-02 | Lớn |
| TR-10 | C1 | `status` thiếu `requested`, `cancelled` — SDK sinh từ enum lỗi khi gặp giá trị lạ | R-STATE-01 | Lớn |
| TR-11 | C7 | `quoteId` không giới hạn độ dài | R-API-14 | Nhỏ |
| TR-12 | C4, C8 | Không mô tả header `Idempotent-Replayed` và hành vi gửi lại | R-IDEM-02 | Nhỏ |
| PQ-01 | C2, C6 | `total` đổi `int64` → `double` ở cùng số 3 | R-GEN-01; §6.6 | Chặn |
| PQ-02 | C6 | Bỏ `expires_at`, tái dùng số 5 cho `bool raining`; Booking mất căn cứ báo `QUOTE_EXPIRED` | §6.6, §7.2; R-QUOTE-03 | Chặn |
| PQ-03 | C1 | Không ghi rõ voucher chỉ trừ tiền món | R-FOOD-04/05 | Nhỏ |
| PQ-04 | C2 | `float distance_km` mất chính xác (mục 5.4) | R-FARE-02 | Lớn |
| PQ-05 | C6 | Giá trị 0 của enum là `RIDE`: thiếu field ⇒ đơn đồ ăn thành đặt xe | quy ước Protobuf | Lớn |
| PQ-06 | C3, C8 | Hết hạn trả `total = 0` thay vì dữ liệu hoặc status code rõ ràng | §7.2; R-QUOTE-03 | Lớn |
| PQ-07 | C5 | Không deadline | §7.2; §14.1 | Chặn |
| PQ-08 | C6 | Thay đổi phá vỡ nhưng giữ package `v1` | §7.2, §12.3 | Lớn |

### 5.3. Quyết định đồng thuận

Mục tiêu chính của technical review là **ra quyết định**. Scribe ghi các quyết định — kèm phương án bị loại, để không ai phải tranh luận lại:

| Mã | Vấn đề | Quyết định | Phương án bị loại và lý do |
|---|---|---|---|
| QĐ-1 | Kiểu tiền (TR-08, PQ-01) | REST: `integer` (đồng); gRPC: `int64 *_dong` | số thực — lệch khi cộng dồn; chuỗi thập phân — không cần vì VND không có phần lẻ |
| QĐ-2 | Field 5 (PQ-02) | giữ `expires_at = 5`; `raining` thành field mới số 13 | xoá `expires_at` — Booking cần nó cho R-QUOTE-03 |
| QĐ-3 | Ai kiểm chủ báo giá | Booking so `customer_id` trả về với người gọi; không đổi Pricing ⇒ 422 `QUOTE_NOT_FOUND` như nhau cho hai trường hợp | thêm `customer_id` vào request — hai nơi cùng kiểm, dễ lệch |
| QĐ-4 | Hết hạn (PQ-06) | Pricing luôn trả `expires_at`; Booking quyết định `QUOTE_EXPIRED`; báo giá không tồn tại ⇒ gRPC `NOT_FOUND` | trả `total = 0` — không phân biệt được với đơn miễn phí |
| QĐ-5 | Deadline (PQ-07) | Booking đặt deadline cho `GetQuote`, nhỏ hơn hẳn ngân sách p95 500 ms của Booking API; con số cụ thể đo rồi mới chốt (giả định ban đầu 200 ms); `GetQuote` chỉ đọc nên được thử lại một lần khi `UNAVAILABLE` | không deadline — §7.2 cấm |
| QĐ-6 | Quãng đường (PQ-04) | `int32 distance_m` (mét); Pricing tự làm tròn 0,1 km | `double` — vẫn là số nhị phân, chỉ đỡ hơn `float` |
| QĐ-7 | Phiên bản (PQ-05, PQ-08) | thêm `SERVICE_KIND_UNSPECIFIED = 0`; thay đổi phá vỡ còn lại (nếu có) ⇒ package `v2` song song `v1` | sửa thẳng `v1` — Booking đang chạy bản cũ |

Phần OpenAPI sau review (trích):

```yaml
      parameters:
        - name: Idempotency-Key
          in: header
          required: true
          schema: { type: string, minLength: 8, maxLength: 128, pattern: '^[A-Za-z0-9_.:-]+$' }
      # requestBody: chỉ quoteId (maxLength 100) và paymentMethod [cash, vixanh], mặc định cash
      responses:
        '201': { description: Đã tạo chuyến, headers: { Idempotent-Replayed: … }, content: { trip: Trip } }
        '409': { $ref: '#/components/responses/Error' }   # QUOTE_ALREADY_USED, IDEMPOTENCY_IN_PROGRESS
        '422': { $ref: '#/components/responses/Error' }   # QUOTE_NOT_FOUND, QUOTE_EXPIRED, VOUCHER_USAGE_LIMIT…
        '429': { $ref: '#/components/responses/RateLimited' }   # kèm Retry-After
```

### 5.4. Bằng chứng trước khi chốt mức nghiêm trọng

Technical review tìm phát hiện bằng cách **đọc**. Nhưng trước khi gắn «Chặn» hay «Lớn», reviewer nên kiểm nhanh phát hiện có đúng không. Với hợp đồng ①, lab chính là bản cài đặt tham chiếu của SPEC, nên ta đối chiếu thẳng; với PQ-04, ta chứng minh lỗi `float` bằng chính hàm tính cước của lab. Lược đồ zod trong test là hợp đồng **đã sửa** của phản hồi 201; test PQ-04 đưa `Math.fround(7.35)` (giá trị `float` nhận được: 7,3499999…) vào hàm tính cước:

```bash
npx vitest run articles/technical-review-api-contract --reporter=verbose
```

```text
 ✓ TR-04/TR-05/TR-08 · R-API-08, R-GEN-01 · tạo chuyến trả 201, bọc trong { trip }, tiền là số nguyên đồng
 ✓ TR-01 · R-IDEM-01 · thiếu Idempotency-Key ⇒ 400 IDEMPOTENCY_KEY_REQUIRED
 ✓ TR-02 · R-QUOTE-04, R-API-14 · giá client gửi kèm bị bỏ qua, chuyến lấy giá của báo giá
 ✓ TR-03 · R-PAY-01 · paymentMethod "card" không được hỗ trợ ⇒ 400 VALIDATION_ERROR có details
 ✓ TR-06 · R-QUOTE-03, R-API-05 · báo giá của khách khác ⇒ 422 QUOTE_NOT_FOUND (không phải 404)
 ✓ TR-07 · R-QUOTE-03, R-IDEM-02 · gửi lại cùng khoá ⇒ 201 replay; cùng báo giá khoá khác ⇒ 409
 ✓ TR-07 · R-API-13 · yêu cầu thứ 6 trong 60 giây ⇒ 429 RATE_LIMITED kèm Retry-After
 ✓ PQ-04 · R-FARE-02 · quãng đường đi qua float 32 bit làm đổi cước và đổi kết quả kiểm biên
 Test Files  1 passed (1)
      Tests  8 passed (8)
```

Mọi test xanh nghĩa là **hệ thống đúng như SPEC, còn bản nháp sai** — đúng điều reviewer khẳng định. Lab bỏ qua `fare` client gửi (R-API-14), nhưng hợp đồng bắt buộc `fare` sẽ dụ bên cài đặt khác dùng nó — vì thế TR-02 là «Chặn». Lưu ý: test này **không phải** bước của technical review theo CTFL, và chỉ làm được khi đã có bản cài đặt tham chiếu; với hợp đồng mới hoàn toàn, chỉ còn cách đọc.

## 6. Góc nhìn nâng cao

### 6.1. Ba cách «kiểm» một hợp đồng API

| | Technical review | Static analysis trên hợp đồng (lint, kiểm breaking) | Contract testing |
|---|---|---|---|
| Loại kiểm thử | static, do người | static, do công cụ | dynamic |
| Bắt được | ngữ nghĩa sai so với nghiệp vụ, quyết định thiết kế, điều còn thiếu | sai cú pháp, vi phạm quy ước, đổi kiểu/tái dùng số field | provider thật trả khác điều consumer mong đợi |
| Không bắt được | lỗi chỉ lộ khi chạy, hiệu năng | «client gửi giá» — hợp lệ về cú pháp | hợp đồng sai từ đầu mà hai bên cùng làm theo |
| Bài | bài này | TEST-029 | Series 07, 11 |

Ba cách bổ sung cho nhau: công cụ bắt PQ-01, PQ-02 rẻ hơn người; chỉ người bắt được TR-02 và đưa ra QĐ-3.

### 6.2. Sai lầm thường gặp

| Sai lầm | Hệ quả | Cách chữa |
|---|---|---|
| Tác giả tự dẫn buổi | thành walkthrough; tác giả bảo vệ thiết kế | moderator không phải tác giả |
| Chỉ mời bên cung cấp | không ai hỏi «app xử lý 409 thế nào» | mời ít nhất một bên tiêu thụ cho mỗi hợp đồng |
| Không chuẩn bị | buổi họp thành sửa tên field, camelCase hay snake_case | gửi trước kèm danh sách tài liệu đối chiếu (TTA 5.1) |
| Review hợp đồng tách khỏi bản đang chạy | bỏ sót thay đổi phá vỡ | luôn đặt bản mới cạnh bản đang chạy |

### 6.3. Giới hạn

- Review không chứng minh được hành vi lúc chạy: p95 < 500 ms của §14.1 chỉ kiểm được bằng đo tải (Series 09). QĐ-5 vì vậy ghi rõ «đo rồi mới chốt».
- Đồng thuận không có nghĩa là đúng: nếu cả phòng cùng hiểu sai SPEC, technical review vẫn thông qua. Đó là lý do mời bên tiêu thụ và người có góc nhìn khác (TTA, bảo mật).

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1.** Review bản nháp sau cho lệnh huỷ chuyến qua gRPC (Booking → Trip). Tìm ít nhất bốn vấn đề, ghi tiêu chí C1–C8 và căn cứ.

```text
service TripService { rpc CancelTrip(CancelTripRequest) returns (CancelTripResponse); }
enum CancelledBy { CUSTOMER = 0; DRIVER = 1; SYSTEM = 2; }
message CancelTripRequest { string trip_id = 1; CancelledBy by = 2; string reason = 3; }
message CancelTripResponse { bool ok = 1; double cancellation_fee = 2; string error = 3; }
```

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(1) Không có khoá idempotency — gửi lại có thể huỷ hai lần hoặc thu phí hai lần (C4; R-IDEM-01 yêu cầu khoá cho lệnh huỷ, R-PAY-05 khoá <code>trip:&lt;id&gt;:cancel-fee</code>). (2) <code>double cancellation_fee</code> — phải là <code>int64</code> đồng (C2; R-GEN-01). (3) Enum giá trị 0 là <code>CUSTOMER</code>: field vắng ⇒ coi như khách huỷ và có thể bị thu phí (C6). (4) Lỗi trả trong <code>string error</code> kèm <code>ok</code> thay vì status code gRPC — không phân biệt <code>INVALID_TRANSITION</code> với không tìm thấy (C3, C8; kiến trúc §7.2). (5) <code>reason</code> không ghi giới hạn 200 ký tự (C7; R-API-11). (6) Thiếu lý do phí <code>searching/grace_period/driver_late/charged</code> (C1; R-CANCEL-06). (7) Không nói deadline (C5).</p>
</details>

**Câu 1 (K1).** Theo CTFL v4.0.1, ai dẫn dắt một technical review?

- a) Tác giả.
- b) Moderator.
- c) Manager.
- d) Bên tiêu thụ API.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — tác giả dẫn walkthrough. <b>b đúng</b> — technical review do moderator dẫn, reviewer có chuyên môn kỹ thuật. <b>c sai</b> — manager quyết định review và cấp nguồn lực. <b>d sai</b> — bên tiêu thụ là reviewer, không phải vai trò dẫn dắt theo syllabus.</p>
</details>

**Câu 2 (K2).** Mục tiêu nổi bật nhất của technical review là gì?

- a) Đào tạo thành viên mới về domain.
- b) Tìm số anomaly tối đa và thu metric cải tiến quy trình.
- c) Đạt đồng thuận và ra quyết định về một vấn đề kỹ thuật.
- d) Phát hiện anomaly mà không cần ghi lại gì.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a sai</b> — mục tiêu nổi bật của walkthrough. <b>b sai</b> — đặc trưng của inspection. <b>c đúng</b> — mục tiêu syllabus nêu đầu tiên cho technical review. <b>d sai</b> — gần với informal review.</p>
</details>

**Câu 3 (K2).** Theo CTAL-TTA v4.0, vì sao Technical Test Analyst cần đủ thời gian chuẩn bị trước review?

- a) Để viết lại work product thay tác giả.
- b) Để kịp đối chiếu tài liệu tham chiếu, xác định điều còn thiếu; thiếu chuẩn bị thì review dễ thành buổi sửa câu chữ.
- c) Vì syllabus chỉ cho phép TTA tham gia inspection.
- d) Để chạy toàn bộ test hồi quy trước buổi họp.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — sửa là việc của tác giả. <b>b đúng</b> — nội dung TTA-5.1.1: chuẩn bị gồm đọc work product, kiểm tài liệu được tham chiếu, tìm điều thiếu; không chuẩn bị thì thành editing exercise. <b>c sai</b> — TTA thường tham gia cả technical review và inspection. <b>d sai</b> — chạy test là dynamic testing, không phải chuẩn bị review.</p>
</details>

**Câu 4 (K2).** Khi review bản Protobuf v2 của Pricing, việc nào thể hiện đúng nhất ý «review không cô lập» của CTAL-TTA?

- a) Kiểm chính tả các comment trong tệp `.proto`.
- b) So bản v2 với bản v1 đang được Booking sử dụng để tìm thay đổi phá vỡ.
- c) Đếm số field trong message `Quote`.
- d) Chỉ mời developer Pricing tham dự.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — có thể hữu ích nhưng không liên quan tương tác với sản phẩm khác. <b>b đúng</b> — kiểm tính nhất quán với sản phẩm đã có, xét tương tác với phần còn lại của hệ thống. <b>c sai</b> — con số không nói gì về tương thích. <b>d sai</b> — thiếu bên tiêu thụ là thiếu góc nhìn tương tác.</p>
</details>

**Câu 5 (K2).** Buổi họp: developer Booking chiếu bản OpenAPI mình viết, tự dẫn mọi người đi từng endpoint, mục tiêu chính là để đội mobile mới hiểu cách gọi. Đây gần nhất với loại review nào?

- a) Technical review.
- b) Inspection.
- c) Walkthrough.
- d) Informal review.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a sai</b> — technical review do moderator dẫn, nhắm quyết định kỹ thuật. <b>b sai</b> — inspection không cho tác giả làm review leader. <b>c đúng</b> — tác giả dẫn, mục tiêu đào tạo và hiểu chung. <b>d sai</b> — có tổ chức buổi và người dẫn rõ ràng, không còn là review không quy trình.</p>
</details>

**Câu 6 (K2).** Phát hiện PQ-02 (tái dùng số field 5) được tìm ra trước khi có code. Điều này minh hoạ đúng nhất ý nào về static testing?

- a) Static testing chỉ áp dụng được cho source code.
- b) Static testing tìm defect trực tiếp trong work product, kể cả work product không chạy được, từ rất sớm.
- c) Static testing thay thế hoàn toàn contract testing.
- d) Static testing chỉ tìm được lỗi chính tả.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — hầu như mọi work product đều review được, gồm hợp đồng và đặc tả (CTFL 3.1.1). <b>b đúng</b> — static testing tìm defect trực tiếp, áp dụng cho work product không thực thi được (CTFL 3.1.3). <b>c sai</b> — hai cách bổ sung cho nhau. <b>d sai</b> — PQ-02 là lỗi tương thích nghiêm trọng.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB® Certified Tester Foundation Level Syllabus **v4.0.1** (2024): mục 3.1 Static Testing Basics, 3.2.2–3.2.5 (quy trình, vai trò, loại review, yếu tố thành công); LO FL-3.2.4 (K2). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB® Certified Tester Advanced Level — Technical Test Analyst Syllabus **v4.0**: chương 5 Reviews, mục 5.1 Technical Test Analyst Tasks in Reviews; LO TTA-5.1.1 (K2). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB Glossary (tra *technical review*, *moderator*, *review*, *anomaly*) — https://glossary.istqb.org/
- Trang chủ ISTQB: https://www.istqb.org/
- ISO/IEC 20246 — Software and systems engineering: Work product reviews.
- Karl Wiegers, *Peer Reviews in Software: A Practical Guide* (Addison-Wesley) — CTAL-TTA dẫn làm tài liệu về technical review và checklist.
- OpenAPI Specification 3.0.3: https://spec.openapis.org/oas/v3.0.3 · Protocol Buffers — Language Guide (proto3): https://protobuf.dev/programming-guides/proto3/
- Kiến trúc tổng thể Tới Nơi (§6.6, §7.1–7.2, §12.3, §14.1, §21); đặc tả lab (`SPEC.md` nhóm R-API, R-IDEM, R-QUOTE) và tệp của bài: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-022 · Static Testing và Dynamic Testing](/articles/static-testing-va-dynamic-testing) · [TEST-027 · Review Process và các vai trò](/articles/review-process-va-cac-vai-tro) · [TEST-024 · Walkthrough](/articles/walkthrough) · [TEST-029 · Static Analysis](/articles/static-analysis-phan-tich-code-tinh)

Bài tiếp theo: [Inspection: review hình thức nhất](/articles/inspection-review-hinh-thuc).
