---
slug: verification-va-validation
title: "Verification và Validation: tính cước đúng đặc tả mà khách vẫn thiệt"
summary: "Verification hỏi «làm đúng đặc tả chưa», validation hỏi «có đúng thứ người dùng cần không». Bài phân biệt hai câu hỏi theo CTFL v4.0.1 qua báo giá cước của Tới Nơi, kèm test chạy thật trên lab."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-005, verification, validation, absence-of-defects, toi-noi]
language: vi
---

Đội Pricing của Tới Nơi báo cáo: mọi test của công thức cước đều xanh, ví dụ kiểm trong đặc tả ra đúng 52.000 đ. Cùng tuần đó, bộ phận chăm sóc khách hàng nhận phản ánh «áp mã GIAM20 mà không rẻ hơn đồng nào». Cả hai cùng đúng — vì họ đang trả lời hai câu hỏi khác nhau. Bài này tách hai câu hỏi đó: **verification** (xác minh) và **validation** (xác nhận), theo đúng cách CTFL v4.0.1 dùng chúng.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-005</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S01 · Fundamentals of Software Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng (ví dụ chạm Component và System)</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/software-testing-la-gi-testing-khac-debugging">TEST-001</a>, <a href="/articles/bay-nguyen-ly-kiem-thu">TEST-003</a>, <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab <code>articles/verification-va-validation</code> (6 ca vitest) · bài tập 2 kiểm tra verification + 2 câu hỏi validation cho báo giá cước</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.1 và 1.1.1 · FL-1.1.1 (K1); mục 1.3 nguyên lý 7 · FL-1.3.1 (K2). Liên quan: 2.2.1 (acceptance testing), 3.1 (static testing). V&amp;V không có LO riêng.</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Mọi sản phẩm phần mềm đều đi qua một chuỗi chuyển đổi: **nhu cầu** của người dùng được viết thành **đặc tả**, đặc tả được xây thành **sản phẩm chạy được**. Mỗi bước chuyển có thể sai theo hai cách khác nhau:

- Sản phẩm **không làm đúng** điều đặc tả viết. Ví dụ: code nhân phụ phí trời mưa với hệ số cao điểm, trong khi đặc tả nói cộng sau.
- Sản phẩm làm đúng đặc tả, nhưng **đặc tả không phản ánh đúng nhu cầu**. Ví dụ: đặc tả cho phép voucher «giảm» trên chuyến mà khách không được rẻ hơn.

Verification bắt loại sai thứ nhất. Validation bắt loại sai thứ hai. Một đội chỉ làm verification sẽ giao một sản phẩm «đúng đặc tả» mà người dùng vẫn không hài lòng. Một đội chỉ làm validation sẽ có sản phẩm được khách thích trong buổi demo nhưng tính sai tiền ở tình huống biên.

| | Verification | Validation |
|---|---|---|
| Câu hỏi | Có làm đúng điều đã đặc tả không? | Có đúng thứ người dùng và các bên liên quan cần không? |
| So với cái gì | Yêu cầu đã đặc tả (test basis) | Nhu cầu, mục tiêu, môi trường vận hành thật |
| Ai trả lời được | Người đọc kỹ đặc tả: developer, tester | Người dùng, Product Owner, bên vận hành, CSKH |
| Ví dụ Tới Nơi | Chuyến R-FARE-09 phải ra 52.000 đ | Khách có hiểu vì sao giá là 52.000 đ không? |

Khi nào cần? Ở **mọi** test level và cho **mọi** sản phẩm công việc (work product), không chỉ code. Đặc tả cũng được verify (có mâu thuẫn nội tại không) và validate (có đúng ý người dùng không).

## 2. Lý thuyết theo ISTQB

### 2.1. Verification và validation trong CTFL v4.0.1

Mục 1.1 «What is Testing?» nêu một hiểu lầm phổ biến: testing chỉ là kiểm xem test object có khớp đặc tả hay không. Syllabus nói rõ testing **gồm cả hai**:

- **Verification**: kiểm tra hệ thống có đáp ứng các **yêu cầu đã được đặc tả** hay không.
- **Validation**: kiểm tra hệ thống có đáp ứng **nhu cầu của người dùng và các bên liên quan khác, trong môi trường vận hành của nó** hay không.

Cụm «trong môi trường vận hành» quan trọng. Một màn hình báo giá có thể đẹp trên máy tester nhưng khó đọc trên điện thoại cũ dưới nắng, khi khách đang vội.

Danh sách mục tiêu kiểm thử điển hình ở mục 1.1.1 (FL-1.1.1, K1) chứa cả hai loại:

| Mục tiêu trong 1.1.1 (diễn đạt lại) | Thuộc |
|---|---|
| Xác minh các yêu cầu đã đặc tả được đáp ứng | Verification |
| Xác minh test object tuân thủ yêu cầu hợp đồng, pháp lý, quy định | Verification (so với một «đặc tả» bên ngoài) |
| Xác nhận test object đầy đủ và hoạt động như các bên liên quan mong đợi | Validation |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">V&amp;V không có LO riêng</span>
<p>CTFL v4.0.1 không có learning objective riêng cho V&amp;V. Hai khái niệm được hỏi qua FL-1.1.1 (K1, «đâu là một mục tiêu kiểm thử») và FL-1.3.1 (K2, «nguyên lý nào bị vi phạm»).</p>
</div>

### 2.2. Nguyên lý 7 — Absence-of-defects fallacy

Nguyên lý 7 (mục 1.3) gọi tên sai lầm khi tin rằng **verification sẽ đảm bảo hệ thống thành công**. Test kỹ mọi yêu cầu đã đặc tả và sửa hết defect tìm được vẫn có thể cho ra một hệ thống không đáp ứng nhu cầu người dùng, không giúp khách hàng đạt mục tiêu kinh doanh, hoặc kém hơn đối thủ. Kết luận của syllabus: ngoài verification, **phải làm thêm validation** ([bảy nguyên lý](/articles/bay-nguyen-ly-kiem-thu)).

### 2.3. Hai khái niệm xuất hiện ở đâu khác trong syllabus

| Mục | Nội dung liên quan |
|---|---|
| 2.2.1 | Acceptance testing tập trung vào validation và sự sẵn sàng triển khai |
| 3.1, 3.1.2 | Static testing dùng được cho **cả** verification lẫn validation; review yêu cầu giúp các bên liên quan kiểm yêu cầu có đúng nhu cầu thật không |
| 5.1.7 | Quadrant Q3 (exploratory, usability, UAT) thiên về validation |

Glossary của ISTQB định nghĩa hai thuật ngữ theo tinh thần ISO 9000: cả hai là sự xác nhận **bằng chứng khách quan** — cho yêu cầu đã đặc tả (verification) hoặc cho một mục đích sử dụng cụ thể (validation). «Tôi thấy ổn» chưa phải validation; cần quan sát, số đo hoặc ý kiến ghi lại của đúng người.

### 2.4. Bảng thuật ngữ Anh – Việt

| Thuật ngữ (ISTQB) | Tiếng Việt dùng trong bài | Ghi chú |
|---|---|---|
| verification | xác minh | so với yêu cầu đã đặc tả |
| validation | xác nhận | so với nhu cầu trong môi trường vận hành |
| specified requirement | yêu cầu đã đặc tả | ở lab: các mã `R-…` của SPEC |
| stakeholder | bên liên quan | khách, tài xế, Admin/Operator, PO, CSKH |
| test basis | cơ sở kiểm thử | tài liệu dùng để suy ra test (SPEC.md) |
| test object | đối tượng kiểm thử | thứ được kiểm (hàm `calculateFare`, API báo giá) |
| absence-of-defects fallacy | ngộ nhận «không có lỗi» | nguyên lý 7 |
| acceptance testing | kiểm thử chấp nhận | test level thiên về validation |
| operational environment | môi trường vận hành | điện thoại thật, MapViet thật |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<ul>
<li><b>Cặp câu của Boehm</b> (tác giả nguyên lý 7 trích dẫn): «building the product right» (verification) và «building the right product» (validation). Dễ nhớ, nhưng khi thi bám định nghĩa ở mục 2.1.</li>
<li><b>V-model</b> hay được vẽ với verification ở nhánh trái, validation ở đỉnh nhánh phải. CTFL v4 chỉ nhắc V-model như ví dụ mô hình tuần tự (2.1.1), không dùng nó để định nghĩa V&amp;V.</li>
<li><b>IEEE 1012</b> là chuẩn riêng về quy trình V&amp;V, hay gặp ở hệ an toàn cao (kèm IV&amp;V — V&amp;V độc lập).</li>
</ul>
</div>

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 230" role="img" aria-label="Nhu cầu được viết thành đặc tả, đặc tả được xây thành sản phẩm. Verification so sản phẩm với đặc tả; validation so sản phẩm và cả đặc tả với nhu cầu.">
<defs>
<marker id="t005-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="t005-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-phys"/></marker>
<marker id="t005-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="30" width="250" height="80" rx="8" class="f-phys-soft s-phys"/>
<text x="145" y="55" text-anchor="middle" class="t-b t-phys">Nhu cầu, mục tiêu</text>
<text x="145" y="75" text-anchor="middle" class="t-small">khách · tài xế · vận hành</text>
<text x="145" y="95" text-anchor="middle" class="t-small">«giá rõ ràng, không bị thiệt»</text>
<rect x="335" y="30" width="250" height="80" rx="8" class="f-pk-soft s-pk"/>
<text x="460" y="55" text-anchor="middle" class="t-b t-pk">Đặc tả = test basis</text>
<text x="460" y="75" text-anchor="middle" class="t-small">SPEC R-FARE, R-VOUCHER, R-UI</text>
<text x="460" y="95" text-anchor="middle" class="t-small">«tối thiểu áp sau voucher»</text>
<rect x="650" y="30" width="250" height="80" rx="8" class="f-surface s-line"/>
<text x="775" y="55" text-anchor="middle" class="t-b">Sản phẩm chạy được</text>
<text x="775" y="75" text-anchor="middle" class="t-small t-mono">calculateFare · /api/quotes</text>
<text x="775" y="95" text-anchor="middle" class="t-small">màn hình báo giá</text>
<line x1="270" y1="70" x2="333" y2="70" class="s-muted" stroke-width="1.6" marker-end="url(#t005-m)"/>
<text x="302" y="62" text-anchor="middle" class="t-small">viết</text>
<line x1="585" y1="70" x2="648" y2="70" class="s-muted" stroke-width="1.6" marker-end="url(#t005-m)"/>
<text x="617" y="62" text-anchor="middle" class="t-small">xây</text>
<path d="M760 110 L760 150 L480 150 L480 113" class="f-none s-pk" stroke-width="1.6" marker-end="url(#t005-a)"/>
<text x="620" y="143" text-anchor="middle" class="t-small t-pk">Verification: làm đúng đặc tả chưa?</text>
<path d="M440 110 L440 172 L170 172 L170 113" class="f-none s-phys dash" stroke-width="1.4" marker-end="url(#t005-p)"/>
<text x="305" y="166" text-anchor="middle" class="t-small t-phys">review đặc tả cùng người dùng</text>
<path d="M800 110 L800 205 L125 205 L125 113" class="f-none s-phys" stroke-width="1.6" marker-end="url(#t005-p)"/>
<text x="462" y="198" text-anchor="middle" class="t-small t-phys">Validation: có đúng thứ người dùng cần, trong môi trường thật?</text>
</svg>
</div>
<figcaption><b>Hình 1 · Hai câu hỏi, hai nguồn sự thật.</b> Verification lấy đặc tả làm chuẩn. Validation lấy nhu cầu làm chuẩn — áp cho cả sản phẩm lẫn chính đặc tả (đường nét đứt: static testing dùng cho validation).</figcaption>
</figure>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Trước khi đặt chuyến, khách bấm «Báo giá». Hệ thống trả một báo giá có hạn 5 phút (R-QUOTE-01) kèm bảng chi tiết từng bước (R-FARE-10). Màn hình khách hiện tổng tiền, cờ cao điểm/mưa, tiền giảm và hạn báo giá (R-UI-01). Khách thấy ổn thì đặt chuyến; giá của chuyến là ảnh chụp giá của báo giá (R-QUOTE-04).

### 3.2. Business rules dùng trong bài

| Mã | Nội dung (tóm tắt SPEC) |
|---|---|
| R-FARE-01 | Xe máy: mở cửa 12.000 đ, 4.500 đ/km, tối thiểu 15.000 đ |
| R-FARE-03 | Thứ tự cố định 6 bước: cơ bản → ×1,25 cao điểm → +mưa → −voucher → nâng lên tối thiểu → làm tròn lên |
| R-FARE-05 | Mưa +8.000 đ, cộng sau hệ số cao điểm (không bị nhân) |
| R-FARE-07 | Cước tối thiểu áp **sau** voucher; voucher không kéo cước xuống dưới mức tối thiểu |
| R-FARE-08 | Làm tròn **lên** bội số 1.000 đ |
| R-FARE-09 | Ví dụ kiểm: 7,4 km xe máy, cao điểm, mưa, giảm 20% ⇒ 52.000 đ |
| R-VOUCHER-06 | Tiền giảm = min(percent% × A, trần), A là số tiền sau bước 3 |
| R-VOUCHER-07 | Mỗi chuyến chưa huỷ dùng mã tiêu một lượt; `GIAM20` được 1 lượt/ngày (Phụ lục A) |
| R-UI-01 | Màn hình báo giá hiện tổng tiền, cờ cao điểm/mưa, tiền giảm |

### 3.3. System context

Trong kiến trúc đích, báo giá do **Pricing & Quote Service** tạo: có `quote_id`, `expires_at`, `pricing_version` và breakdown; quãng đường lấy từ bản đồ MapViet. Mã giảm giá thuộc **Promotion Service**; lượt dùng được kiểm lại khi **Booking Service** tạo booking. Ứng dụng khách đi qua API Gateway (REST).

```text
App khách ──REST──► API Gateway ──► Pricing & Quote ──► MapViet (quãng đường)
                                        │  └──gRPC──► Promotion (voucher, lượt dùng)
                                        ▼
                                  Booking (lưu ảnh chụp giá, kiểm lại lượt voucher)
```

Lab là bản MVP modular monolith: module Pricing & Quote, Promotion và Booking chạy trong một tiến trình (R-ARCH-01), quãng đường do client gửi thẳng (R-GEN-06). Ví dụ của bài chạy được hoàn toàn trên lab.

### 3.4. Rủi ro có thể xảy ra

| Rủi ro | Loại câu hỏi phát hiện được |
|---|---|
| Sai thứ tự bước (nhân phụ phí trời mưa với 1,25) | Verification |
| Làm tròn sai (tròn xuống, hoặc làm tròn giữa chừng) | Verification |
| Mốc 13:00:00 bị tính là cao điểm | Verification |
| Khách không hiểu vì sao giá cao hơn hôm qua (không thấy rõ phụ phí trời mưa) | Validation |
| Báo giá ghi «giảm» nhưng khách không rẻ hơn; khách phàn nàn, gọi tổng đài | Validation |
| Hạn 5 phút quá ngắn với khách đang so giá | Validation |

## 4. Phân tích kỹ thuật

### 4.1. Cùng một tính năng, hai bộ câu hỏi

| Mã | Verification — test condition | Truy vết |
|---|---|---|
| VC-1 | Ví dụ kiểm ra đúng từng bước và 52.000 đ | R-FARE-09 |
| VC-2 | Phụ phí trời mưa không bị nhân hệ số cao điểm | R-FARE-03, R-FARE-05 |
| VC-3 | Tổng làm tròn lên bội 1.000 đ; số tròn nghìn giữ nguyên | R-FARE-08 |
| VC-4 | Voucher không kéo cước dưới 15.000 đ | R-FARE-07 |

| Mã | Validation — câu hỏi | Hỏi ai / bằng gì |
|---|---|---|
| VQ-1 | Khách có hiểu bảng chi tiết cước (R-FARE-10) không? | Phiên quan sát 5 khách, ghi lại câu họ hỏi |
| VQ-2 | Khách có thấy phụ phí trời mưa **trước khi** đặt không? | Quan sát trên điện thoại thật; số ticket «sao đắt hơn» |
| VQ-3 | Dòng «Giảm giá» có đúng với số tiền khách thực tiết kiệm không? | PO + Marketing (chủ chiến dịch voucher) |
| VQ-4 | Khách có muốn giữ voucher cho chuyến dài hơn không? | Phỏng vấn CSKH, dữ liệu lượt dùng voucher |

Khác biệt nằm ở **cột cuối**. Verification truy vết về mã `R-…`: đặc tả là test oracle. Validation truy vết về **một người**: không đọc SPEC mà trả lời được.

### 4.2. Static và dynamic, verification và validation

V&amp;V không đồng nghĩa với static/dynamic. Bốn ô đều có việc:

| | Verification | Validation |
|---|---|---|
| **Static** (không chạy phần mềm) | Review code `fare.ts` đối chiếu R-FARE-03; static analysis | Review SPEC cùng PO, Marketing: luật R-FARE-07 + R-VOUCHER-07 có hợp ý chiến dịch không? |
| **Dynamic** (chạy phần mềm) | Unit test ví dụ kiểm R-FARE-09; API test lượt voucher | Phiên usability màn báo giá; beta ở một thành phố; UAT với Operator |

Ô trên bên phải hay bị quên nhất, dù rẻ nhất: phát hiện đặc tả lệch nhu cầu ngay khi mới là văn bản.

### 4.3. Ghép các luật đúng thành một kết quả đáng hỏi

Từng luật dưới đây, verify riêng, đều đúng. Ghép lại cho chuyến xe máy 0,5 km lúc 09:00, không mưa, áp `GIAM20`:

```text
Cước cơ bản   12.000 + 4.500 × 0,5   = 14.250 đ     (R-FARE-01)
Tiền giảm     20% × 14.250            =  2.850 đ     (R-VOUCHER-06)
Sau giảm      14.250 − 2.850          = 11.400 đ
Tối thiểu     max(11.400; 15.000)     = 15.000 đ     (R-FARE-07)
Tổng                                    15.000 đ     (R-FARE-08)

Không voucher: max(14.250; 15.000) = 15.000 đ  ⇒  khách được lợi 0 đ
Màn hình (R-UI-01) vẫn hiện «Giảm giá: 2.850 đ», lượt GIAM20 hôm nay bị tiêu (R-VOUCHER-07)
```

Verification trả lời: **đúng đặc tả**. Validation hỏi tiếp: khách có muốn điều này không? Đây không phải defect của code — code làm đúng. Nếu có defect, nó nằm ở **test basis** (đặc tả) và người quyết định là Product Owner, không phải developer.

## 5. Thực hành

### 5.1. Chạy test trên lab

Mã nằm ở `examples/toi-noi-testing/articles/verification-va-validation/` ([lab trên GitHub](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing)). Không sửa `src/` — chỉ đọc hành vi của lab.

Sáu ca: V1–V4 là VC-1…VC-4 (gọi thẳng `calculateFare`, mức Component); Q1–Q2 gọi API `POST /api/quotes` và `POST /api/trips` để thu dữ liệu cho VQ-3, VQ-4. Ca Q1, rút gọn:

```ts
it('Q1 · chuyến 0,5 km: có GIAM20 hay không, khách vẫn trả 15.000 đ — nhưng báo giá ghi «giảm 2.850 đ»', async () => {
  const without = (await quote(shortRide).expect(200)).body.quote;
  const withVoucher = (await quote({ ...shortRide, voucherCode: 'GIAM20' }).expect(200)).body.quote;
  expect(without).toMatchObject({ total: 15_000, discount: 0 });
  expect(withVoucher).toMatchObject({ total: 15_000, discount: 2_850, voucherCode: 'GIAM20' });
});
```

Lệnh và kết quả chạy thật (Node 22, vitest 3.2.7; bỏ 6 dòng ✓ V1…Q2):

```bash
npx vitest run articles/verification-va-validation --reporter=verbose
```

```text
[Q1] không voucher: 15000 đ · có GIAM20: 15000 đ, «giảm» 2850 đ ⇒ khách thực được 0 đ
[Q2] chuyến 7,4 km lúc 10:00: không voucher 46000 đ · nếu còn GIAM20 37000 đ ⇒ khách mất cơ hội giảm 9000 đ


 Test Files  1 passed (1)
      Tests  6 passed (6)
```

Sáu ca **đều xanh**. Q1 và Q2 xanh vì hệ thống làm đúng SPEC — chúng không «bắt lỗi» mà **thu bằng chứng** để mang đi hỏi người dùng. Một bộ test tự động không trả lời được câu hỏi validation; nó chỉ chuẩn bị dữ liệu cho câu hỏi đó.

### 5.2. Test case verification mẫu

| ID | Truy vết | Đầu vào | Kết quả mong đợi | Thực tế |
|---|---|---|---|---|
| TC-005-V01 | R-FARE-09 | bike, 7,4 km, cao điểm, mưa, GIAM20 | 45.300 → 56.625 → 64.625 → −12.925 → 51.700 → **52.000 đ** | Đạt (V1) |
| TC-005-V02 | R-FARE-03, R-FARE-05 | bike, 7,4 km, cao điểm; có/không mưa | Chênh `beforeDiscount` đúng 8.000 đ (không phải 10.000) | Đạt (V2) |
| TC-005-V03 | R-FARE-08 | bike 7,4 km và 2,0 km, thấp điểm | 45.300 → 46.000 đ; 21.000 → 21.000 đ | Đạt (V3) |
| TC-005-V04 | R-FARE-07 | bike 0,5 km, GIAM20 | Tiền giảm 2.850 đ, tổng 15.000 đ | Đạt (V4) |

### 5.3. Phiếu phát hiện validation (không phải defect report)

Phát hiện này **không** được ghi như một defect của code: code làm đúng R-FARE-07. Nó đi vào backlog dưới dạng câu hỏi/đề xuất thay đổi yêu cầu cho Product Owner.

| Trường | Nội dung |
|---|---|
| Mã | VAL-005-01 |
| Tiêu đề | Voucher «giảm» trên chuyến chạm cước tối thiểu: khách không rẻ hơn nhưng mất lượt trong ngày |
| Phạm vi đặc tả | R-FARE-07 + R-VOUCHER-06 + R-VOUCHER-07 + R-UI-01 |
| Bằng chứng | Lab Q1, Q2 (mục 5.1); phép tính mục 4.3 |
| Ảnh hưởng (giả định) | Nếu 2% chuyến dùng GIAM20 rơi vào tình huống này và 30% số đó gọi tổng đài: với 10.000 chuyến/ngày dùng mã ⇒ 10.000 × 2% × 30% = 60 cuộc gọi/ngày |
| Phương án để PO chọn | (a) Không áp voucher và không tiêu lượt khi tổng không đổi; (b) hiện «Tiết kiệm thực tế: 0 đ» kèm cảnh báo; (c) giữ nguyên, ghi rõ trong điều khoản voucher |
| Cần ai trả lời | PO Pricing, Marketing (chủ chiến dịch), CSKH |
| Sau khi chọn | Sửa SPEC trước, rồi verify lại bằng test mới truy vết tới phiên bản SPEC mới |

## 6. Góc nhìn nâng cao

**Sai lầm thường gặp**

| Sai lầm | Vì sao sai |
|---|---|
| «Validation là việc của UAT ở cuối dự án» | Static testing dùng được cho validation (mục 3.1): review SPEC cùng PO rẻ hơn nhiều so với phát hiện ở UAT. |
| «Tester tự đoán ý khách là đủ validation» | Tester đại diện gián tiếp cho người dùng (mục 1.2.1), nhưng validation cần bằng chứng từ đúng người. Tester chuẩn bị dữ liệu và câu hỏi. |
| «Hết test case đỏ là sẵn sàng phát hành» | Đúng nguyên lý 7: verification đạt không đảm bảo thành công. |

**Trường hợp dễ bỏ sót**

- **Môi trường vận hành khác môi trường test.** Lab nhận quãng đường do client gửi (R-GEN-06). Hệ thống đích lấy từ MapViet: cùng hai điểm, khách đi đường tắt quen thuộc, bản đồ chọn đường dài hơn ⇒ giá đúng công thức nhưng khách thấy «bị tính dư». Đây là câu hỏi validation chỉ trả lời được trong môi trường thật.
- **SLO cũng có hai mặt.** Kiến trúc Tới Nơi đặt mục tiêu lan truyền trạng thái p95 dưới 2–3 giây (§14.1, R-NFR-04). Verification đo xem có đạt con số đó không. Validation hỏi: khách đứng chờ tài xế có thấy 3 giây là ổn, hay đã bấm huỷ vì tưởng app treo?
- **Đặc tả mơ hồ thì không verify được.** Nếu SPEC chỉ viết «giá hợp lý», không có test oracle. Lúc đó việc đầu tiên là sửa test basis — đó vừa là verification (đặc tả có kiểm được không) vừa là validation (đặc tả có nói đúng ý không).

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Báo giá cước

Cho báo giá cước (R-FARE), viết **2 kiểm tra verification** (một cho thứ tự 6 bước R-FARE-03, một cho làm tròn lên R-FARE-08) và **2 câu hỏi validation** (khách có hiểu bảng chi tiết R-FARE-10 không; khách có thấy phụ phí trời mưa trước khi đặt không). Với mỗi câu hỏi validation, ghi: hỏi ai, bằng cách nào, dấu hiệu nào cho thấy câu trả lời là «không».

<details><summary><b>Lời giải mẫu</b></summary><p><b>Verification 1 — R-FARE-03.</b> Ô tô 10 km, cao điểm, mưa, không voucher. Mong đợi: 25.000 + 110.000 = 135.000 → ×1,25 = 168.750 → +8.000 = 176.750 → tổng <b>177.000 đ</b>. Bản cài sai (cộng mưa trước rồi mới nhân) sẽ ra 178.750 → 179.000 đ, nên ca này phân biệt được hai thứ tự.</p><p><b>Verification 2 — R-FARE-08.</b> Xe máy 2,0 km thấp điểm: 21.000 đ phải giữ nguyên 21.000 đ (không thành 22.000); xe máy 7,4 km thấp điểm: 45.300 đ phải thành 46.000 đ (không thành 45.000).</p><p><b>Validation 1 — R-FARE-10.</b> Hỏi: 5 khách thật, phiên quan sát 15 phút trên điện thoại của họ. Cách: đưa báo giá 52.000 đ (ví dụ R-FARE-09), nhờ họ giải thích vì sao ra số đó. Dấu hiệu «không»: quá nửa không chỉ ra được phụ phí trời mưa hoặc hệ số cao điểm.</p><p><b>Validation 2 — phụ phí trời mưa.</b> Hỏi: CSKH và dữ liệu ticket. Cách: đếm ticket «sao đắt hơn» trong ngày mưa so với ngày khô; quan sát khách có nhìn thấy cờ «Đang mưa» trước khi bấm «Đặt chuyến». Dấu hiệu «không»: ticket tăng rõ vào ngày mưa, hoặc khách chỉ nhìn tổng tiền.</p></details>

### Bài tập 2 — Phân loại hoạt động

Xếp 6 hoạt động vào V (verification) hoặc Val (validation), kèm static (S) hay dynamic (D): (1) chạy test R-ACCEPT-03 hai tài xế cùng nhận; (2) cho 3 tài xế thật dùng `/driver.html` một buổi sáng; (3) review SPEC R-CANCEL cùng đội vận hành; (4) static analysis mã `cancel.ts`; (5) beta tính năng giao bưu kiện ở Cần Thơ; (6) review code `fare.ts` đối chiếu R-FARE-03.

<details><summary><b>Lời giải</b></summary><ul><li>(1) V, D — so với R-ACCEPT-03.</li><li>(2) Val, D — tài xế thật, môi trường thật.</li><li>(3) Val, S — đặc tả có hợp với cách đội vận hành xử lý huỷ chuyến không. (Nếu buổi review chỉ tìm mâu thuẫn giữa R-CANCEL-01 và R-CANCEL-04 thì là V, S — một buổi review có thể làm cả hai.)</li><li>(4) V, S — so với quy tắc code và đặc tả, không chạy.</li><li>(5) Val, D — beta là một dạng acceptance testing.</li><li>(6) V, S.</li></ul></details>

### Câu hỏi trắc nghiệm

**Câu 1 (K1).** Theo CTFL v4.0.1, validation là kiểm tra điều gì?

- a) Hệ thống có đáp ứng các yêu cầu đã đặc tả không
- b) Hệ thống có đáp ứng nhu cầu của người dùng và các bên liên quan trong môi trường vận hành không
- c) Mã nguồn có tuân thủ quy ước code của nhóm không
- d) Defect đã được sửa có còn gây failure không

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — đó là verification.</li><li>b) Đúng — định nghĩa ở mục 1.1.</li><li>c) Sai — đó là một kiểm tra so với quy tắc đã viết (verification), thường bằng static analysis.</li><li>d) Sai — đó là confirmation testing (mục 2.2.3).</li></ul></details>

**Câu 2 (K2).** Hoạt động nào là verification?

- a) Quan sát 5 khách đặt chuyến để xem họ có hiểu phụ phí trời mưa không
- b) Unit test khẳng định chuyến ví dụ R-FARE-09 có tổng 52.000 đ
- c) Chạy beta tính năng giao đồ ăn ở Đà Nẵng
- d) Phỏng vấn CSKH về các khiếu nại tiền giảm voucher

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — so với nhu cầu hiểu giá của khách: validation.</li><li>b) Đúng — oracle là đặc tả R-FARE-09.</li><li>c) Sai — beta là acceptance testing, thiên về validation.</li><li>d) Sai — thu ý kiến bên liên quan về nhu cầu: validation.</li></ul></details>

**Câu 3 (K2).** Bộ test của Tới Nơi đạt 100% yêu cầu trong SPEC, không còn defect mở. Sau phát hành, tỉ lệ khách bỏ đặt sau khi xem báo giá tăng mạnh. Nguyên lý nào mô tả đúng nhất tình huống này?

- a) Exhaustive testing is impossible
- b) Defects cluster together
- c) Absence-of-defects fallacy
- d) Tests wear out

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Sai — vấn đề không nằm ở việc thiếu tổ hợp test.</li><li>b) Sai — không có dữ kiện về defect tập trung.</li><li>c) Đúng — verification đạt nhưng hệ thống không đáp ứng nhu cầu; cần validation.</li><li>d) Sai — không có dấu hiệu bộ test lặp lại mất hiệu quả.</li></ul></details>

**Câu 4 (K2).** Phát biểu nào về static testing đúng với CTFL v4.0.1?

- a) Static testing chỉ dùng cho verification vì không chạy phần mềm
- b) Static testing dùng được cho cả verification lẫn validation
- c) Static testing chỉ tìm được defect trong mã nguồn
- d) Static testing chỉ do tester độc lập thực hiện

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — review yêu cầu cùng các bên liên quan là validation.</li><li>b) Đúng — mục 3.1.</li><li>c) Sai — static testing áp cho hầu hết work product: đặc tả, test plan, user story…</li><li>d) Sai — syllabus khuyến nghị nhiều bên liên quan cùng tham gia.</li></ul></details>

**Câu 5 (K1).** Theo CTFL v4.0.1, test level nào tập trung vào validation và chứng minh hệ thống sẵn sàng triển khai?

- a) Component testing
- b) Component integration testing
- c) System integration testing
- d) Acceptance testing

<details><summary><b>Đáp án: d</b></summary><ul><li>a) Sai — tập trung vào component riêng lẻ.</li><li>b) Sai — tập trung vào giao tiếp giữa các component.</li><li>c) Sai — tập trung vào giao tiếp với hệ thống và dịch vụ bên ngoài.</li><li>d) Đúng — mục 2.2.1.</li></ul></details>

**Câu 6 (K2).** Màn hình báo giá ghi «Giảm giá: 2.850 đ» nhưng tổng tiền giống hệt khi không dùng voucher, đúng như SPEC quy định. Cách xử lý phù hợp nhất?

- a) Ghi defect cho developer sửa `calculateFare`
- b) Ghi nhận là phát hiện validation, chuyển Product Owner quyết định có sửa đặc tả không
- c) Bỏ qua, vì mọi test đều xanh
- d) Chuyển sang debugging để tìm defect trong code

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — code làm đúng đặc tả; sửa code mà không sửa SPEC tạo lệch giữa test basis và sản phẩm.</li><li>b) Đúng — vấn đề (nếu có) nằm ở test basis; quyết định thuộc người sở hữu yêu cầu.</li><li>c) Sai — đúng là ngộ nhận «không có lỗi» (nguyên lý 7).</li><li>d) Sai — không có failure so với đặc tả để tái hiện và chẩn đoán.</li></ul></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 1.1 và 1.1.1 (FL-1.1.1), mục 1.3 nguyên lý 7 (FL-1.3.1), mục 2.2.1, 3.1, 3.1.2, 5.1.7 — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — mục *verification*, *validation*; tra tại trang chủ ISTQB: https://www.istqb.org/
- ISO/IEC/IEEE 29119-1, *Software testing — Part 1: General concepts* (syllabus dẫn ở mục 1.1).
- IEEE 1012, *Standard for System, Software, and Hardware Verification and Validation* (ngoài syllabus).
- Boehm, B. (1981), *Software Engineering Economics* — nguồn syllabus trích cho nguyên lý 7.
- Lab Tới Nơi: `SPEC.md` (R-FARE, R-VOUCHER, R-UI, R-NFR) và `articles/verification-va-validation/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-001 Testing là gì](/articles/software-testing-la-gi-testing-khac-debugging) · [TEST-003 Bảy nguyên lý](/articles/bay-nguyen-ly-kiem-thu) · [TEST-004 Error, Defect, Failure](/articles/error-defect-failure-root-cause)

Bài tiếp theo: [TEST-006 · Mục tiêu kiểm thử và Quality Assurance](/articles/muc-tieu-kiem-thu-va-quality-assurance)
