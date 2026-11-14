---
slug: use-case-testing-scenario-based-testing
title: "Use Case Testing và Scenario-based Testing: kiểm thử trọn luồng khách đặt xe máy"
summary: "Scenario-based testing theo CTAL-TA v4.0: rút luồng chính, extension, exception từ use case «Khách đặt xe máy» của Tới Nơi, đo scenario coverage, phủ vòng lặp tài xế huỷ và chạy 11 test case qua API lab."
category: testing
level: intermediate
tags: [testing, istqb, ctal-ta, test-035, black-box, scenario-based-testing, use-case, dat-xe]
language: vi
---

Bảng quyết định kiểm thử từng quy tắc, state table kiểm thử từng cặp trạng thái – sự kiện. Nhưng khách không dùng Tới Nơi theo từng quy tắc: họ báo giá, đặt, chờ tài xế, bị tài xế bỏ, chờ tiếp, rồi mới tới nơi. Nhiều lỗi chỉ lộ ra khi trạng thái **được mang qua nhiều bước**. Bài này trả lời: thiết kế test từ use case như thế nào, phân biệt luồng chính, extension và exception ra sao, và «đủ» là bao nhiêu kịch bản.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-035</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S04 · Black-box Test Design Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TA v4.0 (CTFL v4.0.1 đã bỏ use case testing)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>System; Acceptance</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Scenario-based Testing (use case testing)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/state-transition-testing-co-ban">TEST-034 · State Transition Testing (cơ bản)</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Từ use case «Khách đặt xe máy»: luồng chính và 4 luồng thay thế/ngoại lệ (voucher hết hạn, tài xế huỷ về <code>searching</code>, không có tài xế 5 phút — R-STATE-05, khách huỷ có phí), test case cho từng luồng; 11 test chạy thật ở <code>articles/use-case-testing-scenario-based-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TA v4.0 · Chương 3 · mục 3.2.3 · TA-3.2.3 (K3); liên quan 3.5.1 (kết hợp kỹ thuật). CTFL v4.0.1 · Phụ lục C (use case testing đã bỏ khỏi Foundation)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

**Scenario-based testing** (kiểm thử dựa trên kịch bản) thiết kế test case từ một **mô hình kịch bản**: chuỗi hành động người dùng và hệ thống làm để đạt một mục tiêu. Mỗi đường đi qua mô hình là một kịch bản; mỗi kịch bản là một coverage item.

Kỹ thuật này giải quyết một lỗ hổng của các kỹ thuật «một bước»: EP, BVA, bảng quyết định kiểm một quy tắc tại một thời điểm; state transition kiểm từng chuyển trạng thái. Không kỹ thuật nào trong số đó buộc ta đi **cả hành trình** và kiểm tra những gì hệ thống nhớ giữa các bước — mốc thời gian, lượt voucher, báo giá đã dùng, tài xế đã rảnh chưa.

Nên dùng khi:

- test basis có dạng use case, user story kèm luồng, sơ đồ quy trình nghiệp vụ;
- cần kiểm thử từ góc người dùng ở system testing hoặc acceptance testing;
- hợp đồng hay khách hàng đưa sẵn danh sách kịch bản nghiệm thu (CTAL-TA 3.5.1 nêu đúng tình huống này);
- muốn sắp xếp test theo rủi ro nghiệp vụ («khách bị tài xế bỏ» quan trọng hơn «khách sửa ô nhập»).

## 2. Lý thuyết theo ISTQB

### 2.1. Từ use case testing tới scenario-based testing

| Syllabus | Kỹ thuật | Mức |
|---|---|---|
| CTFL v4.0.1 | Không còn use case testing (phụ lục C ghi rõ: đã bỏ, vẫn còn ở Advanced Test Analyst) | — |
| CTAL-TA v4.0 | **Scenario-based testing**, mục 3.2.3, nhóm **behavior-based test techniques** cùng CRUD testing và state transition testing | TA-3.2.3 (K3) |

Phụ lục thay đổi của CTAL-TA v4.0 nói LO cũ «use case testing» được thay bằng LO tổng quát hơn về scenario-based testing. Use case giờ chỉ là **một** trong các mô hình kịch bản. Bài giữ tên ghép vì thực tế nhiều đội vẫn gọi «test theo use case».

### 2.2. Mô hình kịch bản

CTAL-TA mô tả hai mô hình (diễn đạt lại):

- **Activity diagram** — sơ đồ luồng công việc: nút bắt đầu/kết thúc, hành động, chuyển tiếp, nút quyết định và nút gộp, fork/join cho việc chạy song song, swim lane cho từng tác nhân. Hợp để mô hình hoá quy trình nghiệp vụ.
- **Use case** — mô tả bằng chữ hoặc hình về tương tác giữa người dùng và hệ thống (hoặc giữa các hệ thống). Use case phân biệt ba loại kịch bản:

| Loại | Ý nghĩa (diễn đạt lại CTAL-TA 3.2.3) | Đạt mục tiêu? |
|---|---|---|
| **Main scenario** (luồng chính, «happy path») | Chuỗi hành động điển hình, được mong đợi, dẫn tới mục tiêu của người dùng. **Mỗi use case chỉ có một.** | Có |
| **Extension** (alternative scenario, luồng thay thế) | Chuỗi khác luồng chính nhưng **cuối cùng vẫn đạt** mục tiêu của luồng chính | Có |
| **Exception** (ngoại lệ) | Chuỗi mà một hành động không mong đợi (dùng sai, đầu vào sai…) khiến **không đạt** mục tiêu | Không |

Syllabus nhắc tới các mô hình khác (flowchart, BPMN, sequence diagram, collaboration diagram) nhưng không đi sâu và trỏ sang syllabus Acceptance Testing (CT-AcT).

### 2.3. Coverage

- **Coverage item** là kịch bản. Thứ tự thiết kế thường theo **risk-based prioritization**.
- **Scenario coverage** = số kịch bản đã thực thi ÷ số kịch bản đã xác định.
- Mô hình **không có vòng lặp** ⇒ số kịch bản hữu hạn, mỗi kịch bản một test case là đủ phủ hết.
- Mô hình **có vòng lặp** ⇒ số đường đi có thể vô hạn. Khi đó áp **simple loop coverage**: mỗi vòng lặp được kiểm khi chạy 0 lần (bỏ qua), đúng 1 lần, nhiều lần (số điển hình) và số lần tối đa (nếu xác định được).
- **Một kịch bản có thể cần nhiều test case**: biến xuất hiện trong kịch bản có thể cần thêm EP hoặc BVA. Coverage criterion cũng có thể đòi chạy một kịch bản nhiều lần.

### 2.4. Phạm vi áp dụng

Syllabus nói scenario-based testing **thường** dùng ở system testing và acceptance testing, dưới dạng end-to-end testing tập trung vào functional suitability từ góc người dùng. Nhưng kỹ thuật này dùng được ở level khác — component integration (kịch bản theo giao thức tương tác), component (lớp có trạng thái, gọi nhiều method theo chuỗi) — và cả non-functional testing (kịch bản làm phần tử của operational profile trong reliability, flexibility, compatibility testing).

Mục 3.5.1 còn gợi ý **kết hợp**: domain testing để chọn giá trị biến trong kịch bản; decision coverage (white-box) để phủ chặt các quyết định trong quy trình; round-trip coverage cho quy trình có chu kỳ.

### 2.5. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa tiếng Việt dùng trong bài |
|---|---|
| scenario-based testing | kiểm thử dựa trên kịch bản |
| use case | ca sử dụng |
| main scenario / happy path | luồng chính |
| extension / alternative scenario | luồng mở rộng / luồng thay thế (vẫn đạt mục tiêu) |
| exception | luồng ngoại lệ (không đạt mục tiêu) |
| scenario coverage | độ phủ kịch bản |
| simple loop coverage | độ phủ vòng lặp đơn |
| operational profile | hồ sơ vận hành (tần suất các kịch bản khi dùng thật) |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Cách viết use case trong bài (bảng bước đánh số; nhánh rẽ đánh «3a», «5b» theo số bước nơi rẽ; tiền điều kiện, hậu điều kiện khi thành công và tối thiểu) theo phong cách phổ biến của Alistair Cockburn (<i>Writing Effective Use Cases</i>). Trong sách đó «extension» gồm cả nhánh đạt lẫn không đạt mục tiêu; bài theo cách chia ba loại của CTAL-TA để đo coverage.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Use case UC-01 «Khách đặt xe máy»

| Mục | Nội dung |
|---|---|
| Tác nhân chính | Khách (C001) |
| Tác nhân phụ | Tài xế xe máy SG (D101, D102), hệ thống (bước quét hết giờ), ví VíXanh |
| Mục tiêu | Khách được chở tới nơi bằng xe máy với giá đã báo |
| Tiền điều kiện | Khách đã đăng nhập; thành phố SG; có tài xế xe máy `online` |
| Hậu điều kiện khi thành công | Chuyến `completed`, tài xế về `online`, tiền thu đúng một lần (R-PAY-03, R-PAY-04) |
| Hậu điều kiện tối thiểu | Không thu tiền sai, không có chuyến «treo», tài xế không bị kẹt `busy` |

**Luồng chính** (09:00 thứ Tư 04/11/2026, ngoài cao điểm, không mưa):

| Bước | Ai | Hành động | Quy tắc |
|---|---|---|---|
| 1 | Khách | Gửi yêu cầu báo giá: `ride`, SG, `bike`, 7,4 km, voucher `GIAM20` | R-API-07, R-VOUCHER |
| 2 | Hệ thống | Trả báo giá: 45.300 − 9.060 (20%) = 36.240 ⇒ **37.000 đ**, hạn 5 phút | R-FARE-03, R-FARE-08, R-QUOTE-01 |
| 3 | Khách | Đặt chuyến từ báo giá, trả bằng VíXanh ⇒ `searching` | R-QUOTE-03, R-API-08 |
| 4 | Tài xế | D101 xem danh sách chuyến chờ, thấy chuyến | R-ACCEPT-06 |
| 5 | Tài xế | D101 nhận, ETA 5 phút ⇒ `accepted`, D101 `busy` | R-ACCEPT-01, R-ACCEPT-05 |
| 6–8 | Tài xế | Tới đón, đón, bắt đầu chở ⇒ `arriving` → `picked_up` → `in_transit` | R-STATE-02 |
| 9 | Tài xế / hệ thống | Hoàn thành ⇒ `completed`; thu 37.000 đ qua VíXanh; D101 về `online` | R-PAY-03, R-ACCEPT-05 |

**Nhánh rẽ** (số = bước nơi rẽ):

- **1a** Voucher hết hạn (`XMAS50`) ⇒ `VOUCHER_EXPIRED`; khách bỏ voucher, báo giá lại, tiếp bước 2.
- **2a** Khách sửa một ô sau khi đã báo giá ⇒ giao diện bắt báo giá lại (R-UI-01).
- **3a** Báo giá đã hết hạn khi bấm đặt ⇒ `QUOTE_EXPIRED`; khách báo giá lại.
- **4b** Không ai nhận trong 5 phút ⇒ hệ thống huỷ, lý do `no_driver`, phí 0 đ (R-STATE-05). *Kết thúc.*
- **5a** Tài xế huỷ (ở `accepted`/`arriving`) ⇒ chuyến về `searching`, mốc tìm tài xế đặt lại, quay về bước 4 (R-STATE-04).
- **5b** Khách huỷ sau khi có tài xế ⇒ phí 0 đ hoặc 10.000 đ tuỳ thời điểm (R-CANCEL-01…04). *Kết thúc.*

### 3.2. System context

Trong kiến trúc đích, một kịch bản UC-01 đi qua Customer App → API Gateway/BFF → **Pricing & Quote** (`quote_id`, `expires_at`) → **Booking** (idempotency key, `BookingCreated` qua outbox) → **Matching & Dispatch** (offer có TTL, atomic assignment) → **Trip Lifecycle** → **Payment & Wallet** (PSP VíXanh) → Notification. Kịch bản cắt ngang các state machine riêng của Booking, Dispatch, Trip, Payment (kiến trúc §9.1) — chỗ test từng service bỏ sót.

Bản lab (SPEC R-ARCH) là modular monolith một tiến trình: không Kafka, không offer (tài xế tự kéo danh sách — R-ACCEPT-06), bước quét hết giờ chạy định kỳ mà test gọi tay qua `POST /api/test/expire-searches`. Kịch bản trong bài chạy qua API HTTP thật của lab, không qua giao diện (giao diện đã được kiểm thử đầu-cuối bằng Playwright ở [TEST-014](/articles/system-testing)).

### 3.3. Rủi ro chỉ lộ ra khi đi trọn kịch bản

| Rủi ro | Kịch bản bắt được | Vì sao test một bước bỏ sót |
|---|---|---|
| Tài xế huỷ nhưng mốc `searchingSince` không đặt lại ⇒ chuyến bị huỷ «không có tài xế» quá sớm | 5a → 4b | Test `cancelByDriver` riêng chỉ kiểm trạng thái về `searching` |
| `acceptedAt` của tài xế cũ còn sót ⇒ khách bị thu phí huỷ dù tài xế mới vừa nhận | 5a → 5b | Test phí huỷ riêng nhận `acceptedAt` do test tự đưa vào |
| Tài xế kẹt `busy` sau khi huỷ/hoàn thành ⇒ không nhận được chuyến sau | 5a, 9 | Phải có **chuyến sau** mới thấy |
| Báo giá cũ vẫn dùng được sau khi hết hạn ⇒ giá sai | 3a | Cần đồng hồ trôi giữa hai bước |
| Thu tiền hai lần ở bước 9 | luồng chính | Phải đi đủ 9 bước mới tới lệnh thu |

## 4. Phân tích kỹ thuật

### 4.1. Dựng mô hình kịch bản

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 365" role="img" aria-label="Sơ đồ hoạt động rút gọn của use case Khách đặt xe máy: luồng chính gồm báo giá, đặt chuyến, tìm tài xế, tài xế nhận, đón và chở, hoàn thành; các nhánh 1a, 2a, 3a quay về báo giá; nhánh 5a từ tài xế nhận quay về tìm tài xế tạo vòng lặp; hai ngoại lệ 4b hết 5 phút và 5b khách huỷ kết thúc không đạt mục tiêu.">
<defs>
<marker id="t035-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
<marker id="t035-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<text x="20" y="22" class="t-small">1a voucher hết hạn ⇒ bỏ voucher, báo giá lại · 2a sửa ô sau khi báo giá ⇒ báo giá lại (chỉ ở UI) · 3a báo giá hết hạn ⇒ báo giá lại</text>
<text x="20" y="42" class="t-small">5a tài xế huỷ ⇒ chuyến về tìm tài xế (vòng lặp) · 4b, 5b là exception: kết thúc mà khách không tới nơi</text>
<circle cx="22" cy="168" r="9" class="f-muted"/>
<line x1="31" y1="168" x2="43" y2="168" class="s-muted" stroke-width="1.4" marker-end="url(#t035-arrow)"/>
<rect x="44" y="140" width="108" height="56" rx="8" class="f-pk-soft s-pk" stroke-width="2"/>
<text x="98" y="163" text-anchor="middle" class="t-b">1–2 · Báo giá</text>
<text x="98" y="183" text-anchor="middle" class="t-small">hạn 5 phút</text>
<line x1="152" y1="168" x2="169" y2="168" class="s-muted" stroke-width="1.4" marker-end="url(#t035-arrow)"/>
<rect x="170" y="140" width="108" height="56" rx="8" class="f-pk-soft s-pk" stroke-width="2"/>
<text x="224" y="163" text-anchor="middle" class="t-b">3 · Đặt chuyến</text>
<text x="224" y="183" text-anchor="middle" class="t-small">Idempotency-Key</text>
<line x1="278" y1="168" x2="295" y2="168" class="s-muted" stroke-width="1.4" marker-end="url(#t035-arrow)"/>
<rect x="296" y="140" width="108" height="56" rx="8" class="f-pk-soft s-pk" stroke-width="2"/>
<text x="350" y="163" text-anchor="middle" class="t-b">4 · Tìm tài xế</text>
<text x="350" y="183" text-anchor="middle" class="t-small t-mono">searching</text>
<line x1="404" y1="168" x2="421" y2="168" class="s-muted" stroke-width="1.4" marker-end="url(#t035-arrow)"/>
<rect x="422" y="140" width="108" height="56" rx="8" class="f-pk-soft s-pk" stroke-width="2"/>
<text x="476" y="163" text-anchor="middle" class="t-b">5 · Tài xế nhận</text>
<text x="476" y="183" text-anchor="middle" class="t-small t-mono">accepted</text>
<line x1="530" y1="168" x2="547" y2="168" class="s-muted" stroke-width="1.4" marker-end="url(#t035-arrow)"/>
<rect x="548" y="140" width="108" height="56" rx="8" class="f-pk-soft s-pk" stroke-width="2"/>
<text x="602" y="163" text-anchor="middle" class="t-b">6–8 · Đón, chở</text>
<text x="602" y="183" text-anchor="middle" class="t-small">3 lệnh tài xế</text>
<line x1="656" y1="168" x2="673" y2="168" class="s-muted" stroke-width="1.4" marker-end="url(#t035-arrow)"/>
<rect x="674" y="140" width="108" height="56" rx="8" class="f-pk-soft s-pk" stroke-width="2"/>
<text x="728" y="163" text-anchor="middle" class="t-b">9 · Hoàn thành</text>
<text x="728" y="183" text-anchor="middle" class="t-small">thu tiền VíXanh</text>
<line x1="782" y1="168" x2="799" y2="168" class="s-muted" stroke-width="1.4" marker-end="url(#t035-arrow)"/>
<rect x="800" y="140" width="108" height="56" rx="28" class="f-phys-soft s-phys" stroke-width="2"/>
<text x="854" y="163" text-anchor="middle" class="t-b t-phys">Đạt mục tiêu</text>
<text x="854" y="183" text-anchor="middle" class="t-small">khách tới nơi</text>
<path d="M 70 140 C 70 100, 126 100, 126 140" class="f-none s-muted dash" stroke-width="1.4" marker-start="url(#t035-arrow)"/>
<text x="98" y="102" text-anchor="middle" class="t-small">1a · 2a</text>
<path d="M 224 140 C 224 64, 140 64, 140 140" class="f-none s-muted" stroke-width="1.4" marker-end="url(#t035-arrow)"/>
<text x="182" y="76" text-anchor="middle" class="t-small">3a</text>
<path d="M 470 196 C 470 262, 330 262, 330 196" class="f-none s-pk" stroke-width="1.6" marker-end="url(#t035-arrow)"/>
<text x="400" y="266" text-anchor="middle" class="t-small t-pk">5a · vòng lặp</text>
<line x1="310" y1="196" x2="310" y2="298" class="s-warn" stroke-width="1.4" marker-end="url(#t035-warn)"/>
<line x1="500" y1="196" x2="500" y2="298" class="s-warn" stroke-width="1.4" marker-end="url(#t035-warn)"/>
<rect x="200" y="300" width="200" height="50" rx="8" class="f-warn-soft s-warn"/>
<text x="300" y="320" text-anchor="middle" class="t-b t-warn">4b · Hết 5 phút</text>
<text x="300" y="339" text-anchor="middle" class="t-small">hệ thống huỷ, phí 0 đ</text>
<rect x="420" y="300" width="200" height="50" rx="8" class="f-warn-soft s-warn"/>
<text x="520" y="320" text-anchor="middle" class="t-b t-warn">5b · Khách huỷ</text>
<text x="520" y="339" text-anchor="middle" class="t-small">phí 0 đ hoặc 10.000 đ</text>
<text x="640" y="329" class="t-small t-warn">exception: kết thúc, không đạt mục tiêu</text>
</svg>
</div>
<figcaption><b>Hình 1 · Mô hình kịch bản của UC-01.</b> Ô tím là luồng chính. Ba nhánh phía trên là extension quay về báo giá (2a vẽ nét đứt vì chỉ tồn tại ở giao diện). Nhánh 5a tạo <b>vòng lặp</b> 4 → 5 → 4, nên số đường đi về lý thuyết là vô hạn. Sơ đồ rút gọn: bỏ nút quyết định và swim lane để dễ đọc.</figcaption>
</figure>

Dựng mô hình trước khi viết test giúp thấy ngay chỗ thiếu (nhánh không có đích, vòng lặp không có điều kiện thoát) và biến mỗi đường đi thành một coverage item đếm được.

### 4.2. Liệt kê kịch bản và ưu tiên theo rủi ro

| Mã | Loại | Đường đi | Ưu tiên | Lý do ưu tiên |
|---|---|---|---|---|
| KB-01 | main | 1→…→9 | Cao | Doanh thu và trải nghiệm cốt lõi; có thu tiền |
| KB-04 | extension | 1→…→5→**5a**→4→5→…→9 | Cao | Trạng thái phải «tháo» sạch: tài xế, `acceptedAt`, mốc tìm |
| KB-05 | exception | 1→2→3→**4b** | Cao | Khách bị bỏ rơi; chuyến không được treo mãi |
| KB-06 | exception | 1→…→5→**5b** | Cao | Liên quan tiền: phí huỷ sai là khiếu nại |
| KB-02 | extension | 1→**1a**→1→2→…→9 | Trung bình | Lỗi voucher thường gặp, nhưng không mất tiền |
| KB-03 | extension | 1→2→**3a**→1→2→3→…→9 | Trung bình | Giá sai nếu báo giá cũ còn dùng được |
| KB-07 | extension | 1→2→**2a**→1→…→9 | Thấp | Chỉ ở giao diện; API vẫn chặn bằng `quoteId` |

Bài thực hành của curriculum yêu cầu luồng chính và 4 nhánh (KB-02, KB-04, KB-05, KB-06). KB-03 và KB-07 được **xác định** thêm khi dựng mô hình — chúng vào mẫu số của coverage dù có chạy hay không.

### 4.3. Vòng lặp 5a và simple loop coverage

| Coverage item | Test case | Kiểm gì |
|---|---|---|
| 0 lần | TC-01 (luồng chính) | Không ai huỷ |
| 1 lần | TC-04 | D101 huỷ ⇒ D102 nhận, hoàn thành; D101 nhận được chuyến khác |
| Nhiều lần (2) | TC-05 | D101 huỷ, D102 huỷ, D101 nhận lại, hoàn thành |
| Tối đa | **Không thiết kế được** | SPEC không quy định số lần tài xế được huỷ một chuyến |

Ô cuối không phải thiếu sót của tester mà là **phát hiện về test basis** (xem 5.4).

### 4.4. Một kịch bản, nhiều test case

KB-05 và KB-06 chứa biến thời gian — đúng trường hợp syllabus nói cần thêm BVA trong kịch bản. Ngoài ra ghép hai nhánh lại để bắt hai rủi ro ở mục 3.3:

| Test case | Kịch bản | Giá trị chọn | Mong đợi |
|---|---|---|---|
| TC-06a | KB-05 | 4:59 sau khi đặt | Bước quét chưa huỷ (`expired = 0`) |
| TC-06b | KB-05 | 5:00 sau khi đặt (R-STATE-05: ≥ 5 phút) | Huỷ, `no_driver`, phí 0, `paymentStatus = none` |
| TC-07a | KB-06 | Khách huỷ 1:59 sau khi nhận | 0 đ, `grace_period` |
| TC-07b | KB-06 | Khách huỷ đúng 2:00 (R-CANCEL-02) | 10.000 đ, `charged`; D101 nhận được chuyến khác |
| TC-08 | 5a → 4b | D101 nhận lúc 4:00, huỷ lúc 4:30 | 9:29 chưa huỷ; 9:30 mới huỷ (mốc tìm đặt lại) |
| TC-09 | 5a → 5b | D101 nhận 0:00, huỷ 3:00; D102 nhận 3:00; khách huỷ 4:00 | 0 đ, `grace_period` — tính theo lần nhận của D102 |

### 4.5. Test case cho các kịch bản còn lại

TC-04, TC-05 ở 4.3; TC-06…TC-09 ở 4.4. Ba kịch bản còn lại:

| TC | Kịch bản | Bước chính | Kết quả mong đợi |
|---|---|---|---|
| TC-01 | KB-01 | Báo giá GIAM20; đặt VíXanh lúc 1:00; D101 thấy chuyến, nhận lúc 2:00; đi hết bước lúc 20:00 | Báo giá 37.000 đ, giảm 9.060 đ, hạn đúng 5 phút; chuyến `searching`/`pending`; cuối cùng `completed`, `paid`, khách thấy tài xế D101; ví thu tổng **37.000 đ** |
| TC-02 | KB-02 | Báo giá với `XMAS50`; bỏ voucher, báo giá lại; đặt; D101 hoàn thành | 422 `VOUCHER_EXPIRED`; báo giá mới 46.000 đ, `voucherCode = null`; `completed` |
| TC-03 | KB-03 | Báo giá lúc 0:00; đặt lúc 5:00; báo giá lại, đặt; hoàn thành | 422 `QUOTE_EXPIRED`; lần sau 201; `completed` |

## 5. Thực hành

### 5.1. Đề bài và lời giải

Đề bài: *từ use case «Khách đặt xe máy», viết luồng chính và 4 luồng thay thế/ngoại lệ (voucher hết hạn, tài xế huỷ về `searching`, không có tài xế 5 phút, khách huỷ có phí) rồi test case cho từng luồng.* Lời giải: use case ở 3.1, mô hình ở hình 1, kịch bản ở 4.2, test case ở 4.3–4.5. Phần dưới chạy chúng trên lab.

### 5.2. Code

Thư mục `articles/use-case-testing-scenario-based-testing/` có hai tệp. `kich-ban.ts` khai báo 7 kịch bản đã xác định (mã, loại, đường đi, test basis, ưu tiên) và hàm `scenarioCoverage` trả số đã chạy, số đã xác định, phần trăm và danh sách kịch bản chưa chạy.

`uc01-dat-xe-may.test.ts` chạy mỗi kịch bản qua API HTTP (supertest), đồng hồ giả `FixedClock`, ví VíXanh giả. Một test case chỉ ghi kịch bản vào tập «đã thực thi» **sau** khi mọi khẳng định đạt. Trích TC-09 — kịch bản ghép bắt lỗi «sót `acceptedAt` của tài xế cũ»:

```ts
it('TC-09 · ghép KB-04 → KB-06: phí huỷ tính theo lần nhận MỚI — 4:00 sau D101 nhưng 1:00 sau D102 ⇒ 0 đ', async () => {
  const tripId = await bookTrip();
  await driver('D101').accept(tripId).expect(200);
  at(3);
  await driver('D101').cancel(tripId).expect(200);
  await driver('D102').accept(tripId).expect(200);
  at(4);
  expect((await customerCancel(tripId).expect(200)).body).toMatchObject({ cancellationFee: 0, feeReason: 'grace_period' });
});
```

Nếu `cancelByDriver` quên xoá `acceptedAt`, hàm phí sẽ đo 4 phút từ lần nhận của D101 và trả `charged` 10.000 đ — test đỏ. Test phí huỷ đơn lẻ không bao giờ thấy lỗi này vì chính test tự đưa `acceptedAt` vào.

### 5.3. Lần chạy thật

Chạy trong `examples/toi-noi-testing` (Node 22.18, vitest 3.2.7):

```bash
npx vitest run articles/use-case-testing-scenario-based-testing
```

```text
stdout | articles/use-case-testing-scenario-based-testing/uc01-dat-xe-may.test.ts
Scenario coverage UC-01: 6/7 = 85.7% · chưa chạy: KB-07
Simple loop coverage (tài xế huỷ ⇒ tìm lại): 3/4 · chưa phủ: tối đa

 ✓ articles/use-case-testing-scenario-based-testing/uc01-dat-xe-may.test.ts (11 tests) 251ms

 Test Files  1 passed (1)
      Tests  11 passed (11)
```

Cả 11 test case đạt (thêm `--reporter=verbose` để xem tên từng test case — trùng các mã TC ở mục 4): lab xử lý đúng mọi kịch bản đã thiết kế, kể cả hai kịch bản ghép.

### 5.4. Đọc coverage cho đúng

**Scenario coverage 6/7 = 85,7%.** KB-07 không chạy vì nó chỉ tồn tại ở giao diện (R-UI-01). Báo cáo phải ghi rõ lý do và nơi phủ thay thế (một test Playwright ở trang `/`), thay vì bỏ KB-07 khỏi mẫu số mà không ghi chú để báo 100%. Lưu ý: 11 test case nhưng chỉ 6 kịch bản — đếm test case không phải đếm coverage.

**Simple loop coverage 3/4.** Mục «tối đa» không thiết kế được, nên ghi thành một anomaly gửi PO:

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Anomaly AN-035-01 · Vòng «tài xế huỷ ⇒ tìm lại» không có giới hạn</span>
<p><b>Test basis:</b> SPEC R-STATE-02, R-STATE-04, R-STATE-05. <b>Mô tả:</b> tài xế huỷ được không giới hạn số lần, và mỗi lần huỷ đặt lại <code>searchingSince</code>. TC-08 xác nhận hành vi: chuyến đặt lúc 0:00 vẫn <code>searching</code> lúc 9:29. Nếu các tài xế liên tục nhận rồi huỷ, R-STATE-05 không bao giờ kích hoạt và khách chờ không có điểm dừng. <b>Câu hỏi cho PO:</b> có giới hạn số lần tìm lại hoặc tổng thời gian chờ không? Khi vượt thì huỷ với lý do gì, có thông báo khách không? <b>Ghi chú:</b> lab đúng với SPEC; đây là chỗ trống của đặc tả, không phải lỗi code. Hệ thống đích có offer TTL và chính sách re-dispatch (kiến trúc §4.5) — chưa có trong lab.</p>
</div>

## 6. Góc nhìn nâng cao

**Extension hay exception?** Phân loại dựa trên **mục tiêu của use case**, nên đổi mục tiêu là đổi phân loại. KB-06 (khách tự huỷ) là exception với mục tiêu «được chở tới nơi», dù hành động huỷ hoàn toàn hợp lệ. Bài tập 1 có một trường hợp khó hơn: tiền thu thất bại nhưng chuyến vẫn hoàn thành (R-PAY-06). Viết mục tiêu và hậu điều kiện rõ trước khi phân loại.

**Bùng nổ kịch bản.** Ghép nhánh tự do (5a → 5a → 5b, 1a → 3a → 5a…) cho số kịch bản tăng nhanh. Đừng liệt kê mọi tổ hợp: dùng ưu tiên theo rủi ro, simple loop coverage cho vòng lặp, và chỉ ghép những nhánh **chia sẻ trạng thái** (5a đổi `acceptedAt` và `searchingSince` ⇒ đáng ghép với 5b, 4b; 1a không đổi gì mà 5b đọc ⇒ không cần ghép).

**Thiên vị happy path.** Đội thường viết luồng chính kỹ, nhánh rẽ sơ sài. Ở Tới Nơi, tiền và khiếu nại nằm ở nhánh rẽ (phí huỷ, hết giờ). Thứ tự ưu tiên ở 4.2 cố ý đặt ba nhánh ngang luồng chính.

**So với kỹ thuật khác.**

| Kỹ thuật | Coverage item | Bắt tốt | Bỏ sót |
|---|---|---|---|
| State transition ([TEST-034](/articles/state-transition-testing-co-ban)) | Trạng thái, chuyển trạng thái | Chuyển sai, chuyển không hợp lệ | Dữ liệu mang qua nhiều chuyển; nhiều tác nhân |
| Decision table ([TEST-033](/articles/decision-table-testing)) | Cột của bảng (quy tắc) | Tổ hợp điều kiện tại một thời điểm | Thứ tự các bước |
| Scenario-based | Kịch bản (đường đi qua mô hình) | Hành trình người dùng, trạng thái mang qua bước, bàn giao giữa tác nhân | Chi tiết từng quy tắc (dùng kỹ thuật khác bên trong kịch bản) |

Scenario-based testing **không thay** các kỹ thuật kia — TC-06 và TC-07 dùng BVA ngay bên trong kịch bản. Kiểm thử luồng nhiều vai trò của giao đồ ăn ở [TEST-081 · Business Flow Testing](/articles/business-flow-testing); end-to-end qua nhiều ứng dụng ở [TEST-080](/articles/end-to-end-testing); nhánh huỷ ở mức hệ thống ở [TEST-087](/articles/negative-flow-testing).

**Kịch bản dài thì chẩn đoán khó.** TC-01 có 9 bước; đỏ ở cuối không nói lỗi ở bước nào. Khẳng định ở từng bước (TC-01 kiểm cả báo giá, trạng thái sau đặt, danh sách tài xế) và giữ test component cho từng quy tắc.

**Dùng mô hình kịch bản cho static testing.** CTAL-TA mục 5.2.1 (TA-5.2.1, K3) dùng activity diagram để tìm defect của đặc tả (hành động thiếu, không tới được, ngõ cụt, sai thứ tự) và mục 5.2.2 có kỹ thuật **scenario-based reviewing** — chạy «khô» theo kịch bản khi review. AN-035-01 chính là loại phát hiện đó: vòng lặp không có điều kiện thoát bị lộ ngay khi vẽ hình 1, trước cả khi chạy test.

**Sai lầm thường gặp:** viết use case theo màn hình thay vì theo mục tiêu; một «kịch bản» gồm cả chục mục tiêu; quên tác nhân phụ (hệ thống quét, ví); bỏ kịch bản khó tự động khỏi mẫu số coverage; khẳng định chỉ ở bước cuối.

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Use case «Khách gửi bưu kiện»

Phân loại các nhánh sau của UC-02 (mục tiêu: «bưu kiện của khách được giao tới nơi»): (a) khách nhập 25 kg, chọn xe máy ⇒ `VEHICLE_TOO_SMALL`, đổi sang ô tô; (b) khách nhập 31 kg ⇒ `WEIGHT_OUT_OF_RANGE`; (c) chuyến `vixanh` hoàn thành nhưng ví từ chối thu tiền; (d) tài xế ô tô D103 huỷ, D202 ở Hà Nội cố nhận.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(a) <b>extension</b> — đổi xe rồi vẫn gửi được (R-PARCEL-02). (b) <b>exception</b> — 31 kg ngoài (0; 30] (R-PARCEL-01), không có cách nào trong hệ thống để gửi. (c) Theo mục tiêu của khách: <b>đạt</b> (bưu kiện tới nơi; R-PAY-06: chuyến vẫn <code>completed</code>, <code>paymentStatus = failed</code>) ⇒ extension. Nếu use case viết từ góc Tới Nơi với mục tiêu «giao và thu tiền» thì là exception — đây là lý do phải viết mục tiêu rõ. (d) Không phải nhánh của UC-02: D202 nhận chuyến SG là kịch bản lỗi của use case tài xế (<code>CITY_MISMATCH</code>, R-ACCEPT-02); ở UC-02 chỉ còn nhánh «tài xế huỷ ⇒ tìm lại» (vòng lặp như 5a).</p>
</details>

### Bài tập 2 — EP/BVA bên trong KB-06

Tài xế nhận lúc 0:00 với ETA 5 phút. Liệt kê test case của KB-06 phủ các phân vùng của R-CANCEL-01…04 (trạng thái lúc huỷ × thời điểm).

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><code>accepted</code> 1:59 ⇒ 0 đ <code>grace_period</code>; <code>accepted</code> 2:00 ⇒ 10.000 đ <code>charged</code>; <code>arriving</code> 1:00 ⇒ 10.000 đ (ân hạn 2 phút chỉ áp cho <code>accepted</code>); <code>arriving</code> 15:00 ⇒ 10.000 đ (đúng mốc ETA + 10 phút vẫn thu — R-CANCEL-04 dùng «lớn hơn hẳn»); <code>arriving</code> 15:01 ⇒ 0 đ <code>driver_late</code>; <code>accepted</code> 15:01 ⇒ 0 đ <code>driver_late</code>. Huỷ ở <code>searching</code> (0 đ) thuộc nhánh khác (huỷ trước khi có tài xế). Ở <code>picked_up</code> khách không huỷ được ⇒ <code>INVALID_TRANSITION</code> — đó là kiểm thử chuyển trạng thái không hợp lệ của TEST-034, không phải kịch bản của UC-01.</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Khách nhập voucher đã hết hạn, nhận thông báo lỗi, bỏ voucher và đặt chuyến thành công. Theo CTAL-TA v4.0, đây là:

- a) Main scenario.
- b) Extension.
- c) Exception.
- d) Không phải kịch bản vì có lỗi.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — use case chỉ có một luồng chính, và luồng này rẽ khỏi nó. <b>b đúng</b> — chuỗi khác luồng chính nhưng cuối cùng vẫn đạt mục tiêu. <b>c sai</b> — exception là khi không đạt mục tiêu. <b>d sai</b> — thông báo lỗi giữa chừng không làm kịch bản mất giá trị; nó là phần của extension.</p>
</details>

**Câu 2 (K1).** Phát biểu nào đúng về use case testing trong các syllabus ISTQB hiện hành?

- a) Là kỹ thuật K3 của CTFL v4.0.1, chương 4.
- b) Đã bị bỏ hoàn toàn khỏi mọi syllabus ISTQB.
- c) Đã bỏ khỏi CTFL v4.0.1; CTAL-TA v4.0 thay bằng scenario-based testing, trong đó use case là một mô hình kịch bản.
- d) Chỉ còn trong syllabus Performance Testing.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a sai</b> — CTFL v4.0.1 phụ lục C ghi đã bỏ use case testing. <b>b sai</b> — nó tồn tại dưới dạng tổng quát hơn ở CTAL-TA v4.0 mục 3.2.3. <b>c đúng</b>. <b>d sai</b> — không có cơ sở.</p>
</details>

**Câu 3 (K3).** Đội xác định 9 kịch bản cho use case «Khách huỷ chuyến». Bộ test có 12 test case, phủ 6 kịch bản khác nhau (có kịch bản được chạy 3 lần với giá trị biên khác nhau). Scenario coverage là bao nhiêu?

- a) 133%.
- b) 100%.
- c) 66,7%.
- d) 50%.

<details>
<summary><b>Đáp án: c</b></summary>
<p>Coverage = số kịch bản đã thực thi ÷ số kịch bản đã xác định = 6 ÷ 9 ≈ 66,7%. <b>a sai</b> — lấy 12 test case chia 9. <b>b sai</b> — coi mỗi test case là một kịch bản. <b>c đúng</b>. <b>d sai</b> — lấy 6 ÷ 12.</p>
</details>

**Câu 4 (K3).** Giả sử Tới Nơi cho khách đổi điểm đến **tối đa 3 lần** trong một chuyến; mô hình kịch bản có vòng lặp «đổi điểm đến». Bộ số lần lặp nào đạt simple loop coverage với ít test case nhất?

- a) 1, 3.
- b) 0, 1, 2, 3.
- c) 0, 3.
- d) 1, 2, 3, 4.

<details>
<summary><b>Đáp án: b</b></summary>
<p>Simple loop coverage đòi: bỏ qua vòng (0), đúng một lần (1), nhiều lần — số điển hình (2) — và số tối đa (3). <b>a sai</b> — thiếu 0 và «nhiều lần». <b>b đúng</b>. <b>c sai</b> — thiếu 1 và số điển hình. <b>d sai</b> — thiếu 0; còn 4 là lần lặp không hợp lệ, thuộc kiểm thử biên/negative, không phải coverage item của vòng lặp.</p>
</details>

**Câu 5 (K2).** Phát biểu nào về phạm vi áp dụng của scenario-based testing là **đúng**?

- a) Chỉ dùng được ở acceptance testing.
- b) Thường dùng ở system và acceptance testing, nhưng cũng dùng ở component integration testing và làm phần tử của operational profile trong reliability testing.
- c) Là kỹ thuật white-box vì cần biết luồng điều khiển của code.
- d) Không dùng được khi mô hình có vòng lặp.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — syllabus nêu cả system testing và các level khác. <b>b đúng</b> — đúng các ví dụ CTAL-TA 3.2.3 đưa ra. <b>c sai</b> — mô hình dựng từ use case/quy trình nghiệp vụ, là black-box (behavior-based). <b>d sai</b> — có vòng lặp thì dùng simple loop coverage.</p>
</details>

**Câu 6 (K3).** Test case nào **có giá trị nhất** để bắt lỗi «khi tài xế huỷ, hệ thống không đặt lại mốc bắt đầu tìm tài xế» (R-STATE-04, R-STATE-05)?

- a) Đặt chuyến, chờ đúng 5:00, kiểm chuyến bị huỷ `no_driver`.
- b) Đặt chuyến, tài xế nhận lúc 4:00 và huỷ lúc 4:30, kiểm bước quét lúc 9:29 **chưa** huỷ chuyến.
- c) Gọi hàm chuyển trạng thái với (`accepted`, `driver_cancel`) và kiểm kết quả `searching`.
- d) Tài xế nhận rồi hoàn thành chuyến.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — không có tài xế huỷ nên không chạm vào việc đặt lại mốc. <b>b đúng</b> — kịch bản ghép 5a → 4b (TC-08): nếu mốc không đặt lại, 9:29 đã quá 5 phút kể từ lúc đặt và chuyến bị huỷ sai. <b>c sai</b> — chỉ kiểm trạng thái, không kiểm dữ liệu mang sang bước sau. <b>d sai</b> — không đi qua nhánh huỷ.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB® Certified Tester Advanced Level — Test Analyst Syllabus **v4.0** (2025): mục 3.2 Behavior-Based Test Techniques, 3.2.3 Scenario-Based Testing, 3.5.1 (kết hợp kỹ thuật), 5.2.1–5.2.2 (dùng mô hình và review technique để tìm defect của test basis); LO TA-3.2.3 (K3). PDF: https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB® Certified Tester Foundation Level Syllabus **v4.0.1** (2024): phụ lục C (use case testing đã bỏ khỏi Foundation). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary (tra *scenario-based testing*, *use case*, *coverage item*) — https://glossary.istqb.org/
- ISO/IEC/IEEE 29119-4 — Software testing, Part 4: Test techniques (CTAL-TA dẫn khi nói mô hình kịch bản).
- Alistair Cockburn, *Writing Effective Use Cases* — cách viết use case có bước đánh số và nhánh rẽ (ngoài ISTQB).
- Đặc tả và mã của lab Tới Nơi (`SPEC.md` nhóm R-STATE, R-CANCEL, R-QUOTE, R-VOUCHER; test của bài): https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-034 · State Transition Testing (cơ bản)](/articles/state-transition-testing-co-ban) · [TEST-014 · System Testing](/articles/system-testing) · [TEST-016 · Acceptance Testing](/articles/acceptance-testing-tong-quan) · [TEST-081 · Business Flow Testing](/articles/business-flow-testing)

Bài tiếp theo: [Classification Tree Method](/articles/classification-tree-method).
