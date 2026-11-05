---
slug: mc-dc-testing
title: "MC/DC Testing: chứng minh từng điều kiện ảnh hưởng độc lập — bài thực hành voucher"
summary: "Modified condition/decision testing theo CTAL-TTA: cặp test độc lập, bộ N+1, điều kiện ghép, ngắt mạch. Bài thực hành nâng cao Series 05: voucher Tới Nơi đạt 100% statement, branch và MC/DC, chạy thật."
category: testing
level: advanced
tags: [testing, istqb, ctal-tta, test-047, mc-dc, white-box, code-coverage, voucher]
language: vi
---

Quyết định «voucher có được áp không» của Tới Nơi ghép chín điều kiện. Decision coverage chỉ cần hai ca (một được, một không) và để lọt lỗi gõ nhầm `||` thành `&&`; multiple condition coverage đòi 2⁹ = 512 tổ hợp. Bài này trả lời: **MC/DC đòi gì, vì sao N + 1 ca thường là đủ, thiết kế cặp test độc lập thế nào khi điều kiện bị ghép và ngôn ngữ luôn ngắt mạch** — rồi làm trọn bài thực hành nâng cao của Series 05: một bộ test cho thuật toán áp voucher đạt 100% statement, 100% branch và MC/DC, chạy thật, nộp báo cáo coverage.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-047</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S05 · White-box &amp; Experience-based Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TTA v4.0 (kèm Foundation — CTFL v4.0.1)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">Test Level</th><td>Component</td></tr>
<tr><th scope="row">Test Type</th><td>White-box</td></tr>
<tr><th scope="row">Test Technique</th><td>MC/DC (Modified Condition/Decision Testing)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/multiple-condition-testing">TEST-046 · Multiple Condition Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Bài thực hành nâng cao của Series 05 (lab, code): bộ test cho thuật toán áp voucher (R-VOUCHER-02…08) đạt 100% statement, 100% branch và MC/DC cho quyết định hợp lệ, kèm báo cáo coverage — <code>examples/toi-noi-testing/articles/mc-dc-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TTA v4.0 · Chương 2 · mục 2.4 · TTA-2.4.1 (K3); bài thực hành dùng thêm mục 2.2, 2.3 · TTA-2.2.1 (K3), TTA-2.3.1 (K3) và mục 2.8 (quan hệ bao hàm) · CTFL v4.0.1 · mục 4.3.1, 4.3.2 · FL-4.3.1 (K2), FL-4.3.2 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

**Modified condition/decision testing** (MC/DC — kiểm thử điều kiện/quyết định sửa đổi) là kỹ thuật white-box cho các **quyết định gồm nhiều điều kiện**. Nó không hỏi «quyết định đã đúng và sai chưa» (decision testing, [TEST-044](/articles/branch-coverage-decision-coverage)), cũng không đòi mọi tổ hợp (multiple condition testing, [TEST-046](/articles/multiple-condition-testing)). Nó đòi một bằng chứng cụ thể: **mỗi điều kiện, khi đổi giá trị một mình, làm đổi kết quả của quyết định**.

Nó nhắm loại defect «quyết định đi sai đường vì một điều kiện bị bỏ qua, đảo dấu hoặc nối sai toán tử» — loại mà hai ca decision coverage đi qua mà không điều kiện nào thật sự «được hỏi ý kiến».

Khi nào dùng: syllabus nêu ngành an toàn (hàng không, ô tô) — nơi failure có thể gây thảm hoạ. Ngoài đó, MC/DC đáng tiền ở quyết định **trực tiếp ra tiền hoặc ra quyền**: áp voucher, duyệt hoàn tiền, nhận chuyến.

## 2. Lý thuyết theo ISTQB

### 2.1. Định nghĩa và yêu cầu

CTAL-TTA v4.0 mục 2.4 mô tả MC/DC như sau (diễn đạt lại):

- Một **decision predicate** (vị từ quyết định) gồm một hoặc nhiều **atomic condition** (điều kiện nguyên tử), mỗi điều kiện cho TRUE/FALSE, được nối bằng toán tử logic để ra kết quả quyết định. Quyết định chỉ có một điều kiện thì MC/DC trùng decision testing.
- Kỹ thuật kiểm rằng **từng điều kiện nguyên tử ảnh hưởng độc lập và đúng đắn** tới kết quả. Cách chứng minh: **cặp test** chỉ khác nhau ở giá trị **một** điều kiện và cho **kết quả quyết định khác nhau**.
- Với N điều kiện **duy nhất, độc lập lẫn nhau**, thường đạt MC/DC với **N + 1 lần thực thi quyết định**. Một test case có thể đi qua nhiều quyết định, nên số test case có thể ít hơn N + 1 lần đếm riêng từng quyết định.

**MC/DC coverage** = số điều kiện đã chứng minh được ảnh hưởng độc lập ÷ tổng số điều kiện. LO **TTA-2.4.1 (K3)**: thiết kế test case cho một test object để đạt **MC/DC đầy đủ**.

Vì sao N + 1: với một quyết định toàn AND, lấy một ca «tất cả đúng» (kết quả TRUE) rồi lần lượt lật từng điều kiện thành sai (N ca, kết quả FALSE). Mỗi ca lật ghép cặp với ca gốc. Với OR thì đối xứng: ca gốc «tất cả sai». Quyết định trộn AND/OR như voucher cần chọn ca gốc khéo hơn (mục 4.3).

### 2.2. Giới hạn syllabus nêu

| Giới hạn | Nội dung | Ở Tới Nơi |
|---|---|---|
| **Coupled conditions** (điều kiện ghép) | Cùng một biến xuất hiện nhiều lần ⇒ có thể không lật được một điều kiện mà giữ nguyên điều kiện khác. Cách xử lý: chỉ áp MC/DC cho điều kiện không ghép, hoặc phân tích riêng quyết định có ghép | `kinds.length === 0` và `kinds.includes(...)` cùng đọc `kinds`; `day < validFrom` và `day > validTo` cùng đọc `day` |
| **Short-circuiting** (ngắt mạch) | Bỏ đánh giá vế còn lại khi đã biết kết quả ⇒ có ca không đạt được; ngành an toàn thường không được tắt vì code kiểm thử phải trùng code giao hàng | JavaScript/TypeScript **luôn** ngắt mạch `&&`, `||`, `??` — không có công tắc tắt |

### 2.3. Vị trí trong quan hệ bao hàm

Mục 2.8 của CTAL-TTA: khi đặt mức coverage ở 100%, **branch/decision bao hàm statement, MC/DC bao hàm decision và branch, multiple condition bao hàm MC/DC**. Syllabus cũng khuyên chỉ cần đặt **một** tiêu chí ở 100% (không cần đòi cả 100% statement lẫn 100% MC/DC) — nhưng điều đó đúng khi tiêu chí áp cho **mọi quyết định** của test object. Bài thực hành sẽ cho thấy MC/DC của **một** quyết định không kéo theo 100% branch của cả tệp.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 230" role="img" aria-label="Bốn tiêu chí coverage áp cho quyết định voucher V: statement; decision 2 ca; MC/DC 10 ca; multiple condition 512 tổ hợp, 288 khả thi. Tiêu chí bên phải bao hàm bên trái.">
<defs>
<marker id="t047-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="16" y="40" width="200" height="110" rx="8" class="f-surface s-line"/>
<text x="116" y="66" text-anchor="middle" class="t-b">Statement</text>
<text x="116" y="92" text-anchor="middle" class="t-small">mỗi câu lệnh chạy</text>
<text x="116" y="110" text-anchor="middle" class="t-small">ít nhất một lần</text>
<text x="116" y="136" text-anchor="middle" class="t-small t-mono">TTA-2.2.1</text>
<rect x="242" y="40" width="200" height="110" rx="8" class="f-surface s-line"/>
<text x="342" y="66" text-anchor="middle" class="t-b">Decision / Branch</text>
<text x="342" y="92" text-anchor="middle" class="t-small">V nhận TRUE và FALSE</text>
<text x="342" y="110" text-anchor="middle" class="t-small t-b">V: 2 ca</text>
<text x="342" y="136" text-anchor="middle" class="t-small t-mono">TTA-2.3.1</text>
<rect x="468" y="40" width="200" height="110" rx="8" class="f-phys-soft s-phys"/>
<text x="568" y="66" text-anchor="middle" class="t-b t-phys">MC/DC</text>
<text x="568" y="92" text-anchor="middle" class="t-small">mỗi điều kiện có một cặp</text>
<text x="568" y="110" text-anchor="middle" class="t-small t-b">V: 10 ca = N + 1</text>
<text x="568" y="136" text-anchor="middle" class="t-small t-mono">TTA-2.4.1</text>
<rect x="694" y="40" width="200" height="110" rx="8" class="f-surface s-line"/>
<text x="794" y="66" text-anchor="middle" class="t-b">Multiple condition</text>
<text x="794" y="92" text-anchor="middle" class="t-small">mọi tổ hợp điều kiện</text>
<text x="794" y="110" text-anchor="middle" class="t-small t-b">V: 512, khả thi 288</text>
<text x="794" y="136" text-anchor="middle" class="t-small t-mono">TTA-2.5.1</text>
<line x1="466" y1="95" x2="444" y2="95" class="s-muted" marker-end="url(#t047-arrow)"/>
<line x1="240" y1="95" x2="218" y2="95" class="s-muted" marker-end="url(#t047-arrow)"/>
<line x1="692" y1="95" x2="670" y2="95" class="s-muted" marker-end="url(#t047-arrow)"/>
<text x="460" y="186" text-anchor="middle" class="t-small">Mũi tên: tiêu chí bên phải bao hàm (subsumes) tiêu chí bên trái khi đạt 100% trên mọi quyết định.</text>
<text x="460" y="208" text-anchor="middle" class="t-small">V = quyết định «voucher được áp», 9 điều kiện; 224 tổ hợp bất khả thi do điều kiện ghép.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Bốn tiêu chí white-box của CTAL-TTA áp cho quyết định voucher.</b> MC/DC nằm giữa: chặt hơn decision (2 ca), rẻ hơn rất nhiều so với multiple condition. Số 288 = 512 trừ các tổ hợp «kinds rỗng mà vẫn chứa dịch vụ» và «trước validFrom mà sau validTo».</figcaption>
</figure>

### 2.4. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa tiếng Việt |
|---|---|
| decision / decision predicate | quyết định / vị từ quyết định |
| atomic condition | điều kiện nguyên tử |
| modified condition/decision coverage (MC/DC) | độ phủ điều kiện/quyết định sửa đổi |
| independence pair | cặp chứng minh độc lập |
| coupled conditions | điều kiện ghép |
| short-circuiting | đánh giá ngắt mạch |
| subsumes | bao hàm |
| infeasible | bất khả thi |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Syllabus không đặt tên các biến thể. Tài liệu ngành (Chilenski &amp; Miller 1994; hướng dẫn MC/DC của NASA 2001) phân biệt <b>unique-cause MC/DC</b> (mọi điều kiện khác giữ nguyên giá trị) và <b>masking MC/DC</b> (điều kiện khác được phép đổi nếu bị «che» — không ảnh hưởng kết quả). Bài dùng unique-cause, coi điều kiện <b>không được đánh giá</b> vì ngắt mạch là «không quan tâm» — cách thường dùng cho ngôn ngữ luôn ngắt mạch. Chuẩn phần mềm bay DO-178C đòi MC/DC cho mức cao nhất — chỉ để định vị, không phải nội dung thi.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Khách nhập mã voucher khi báo giá; nếu hợp lệ, tiền giảm được trừ vào cước (R-FARE-06). Mọi lỗi voucher trả về một mã lỗi duy nhất theo thứ tự cố định.

### Business rules (SPEC lab)

| Mã | Quy tắc tóm tắt |
|---|---|
| R-VOUCHER-02 | Bỏ khoảng trắng hai đầu, viết hoa; chuỗi rỗng = không nhập voucher |
| R-VOUCHER-03 | Không có mã ⇒ `VOUCHER_NOT_FOUND` |
| R-VOUCHER-04 | Ngày đặt (giờ VN) ngoài [validFrom; validTo], **tính cả hai đầu** ⇒ `VOUCHER_EXPIRED` |
| R-VOUCHER-05 | Dịch vụ ngoài `kinds` hoặc thành phố ngoài `cityCodes` ⇒ `VOUCHER_NOT_APPLICABLE`; danh sách **rỗng = áp cho mọi** (R-VOUCHER-01) |
| R-VOUCHER-06 | A < `minAmount` ⇒ `VOUCHER_MIN_AMOUNT` (A = minAmount hợp lệ); giảm = min(percent% × A, maxDiscount) |
| R-VOUCHER-07 | Số chuyến chưa huỷ đã dùng mã trong ngày phải **nhỏ hơn** `perCustomerPerDay` |
| R-VOUCHER-08 | Thứ tự báo lỗi: NOT_FOUND → EXPIRED → NOT_APPLICABLE → MIN_AMOUNT → USAGE_LIMIT |

### System context

Theo kiến trúc tổng thể, eligibility và quota thuộc **Promotion Service** (§4.10), được **Pricing & Quote** (§4.3) gọi khi tạo quote; kiến trúc còn yêu cầu chống vượt quota khi nhiều yêu cầu đồng thời — phần đó thuộc kiểm thử đồng thời (Series 08), lab không mô phỏng. Trong lab (MVP modular monolith §18): `src/domain/voucher.ts` (chuẩn hoá mã, thời hạn/phạm vi, tiền giảm, lượt) và `src/app/quote.ts` (`priceQuote` ghép thứ tự R-VOUCHER-08). Pipeline §17 đặt unit test ngay sau static analysis — nơi bộ test của bài này chạy.

### Rủi ro

- **Voucher lọt phạm vi** (`||` viết thành `&&`): thất thoát ngân sách — giả định 2.000 lượt sai/ngày × 4.530 đ ≈ 9,1 triệu đ/ngày.
- **Từ chối nhầm voucher hợp lệ** (hiểu sai «rỗng = mọi», biên ngày không tính hai đầu): khiếu nại, bỏ đặt chuyến.
- **Bỏ qua kiểm lượt**: C9 không bao giờ ảnh hưởng kết quả ⇒ một khách dùng mã nhiều lần.

## 4. Phân tích kỹ thuật

### 4.1. Tách quyết định V từ đặc tả

Gộp R-VOUCHER-03…07 thành một quyết định «voucher được áp»:

```text
V = C1 ∧ C2 ∧ C3 ∧ (C4 ∨ C5) ∧ (C6 ∨ C7) ∧ C8 ∧ C9
```

| Điều kiện | Ý nghĩa | Mã |
|---|---|---|
| C1 | tìm thấy mã | R-VOUCHER-03 |
| C2 / C3 | ngày ≥ validFrom / ngày ≤ validTo | R-VOUCHER-04 |
| C4 / C5 | `kinds` rỗng / `kinds` chứa dịch vụ | R-VOUCHER-05 |
| C6 / C7 | `cityCodes` rỗng / `cityCodes` chứa thành phố | R-VOUCHER-05 |
| C8 | A ≥ minAmount | R-VOUCHER-06 |
| C9 | lượt đã dùng < perCustomerPerDay | R-VOUCHER-07 |

N = 9. R-VOUCHER-02 (chuẩn hoá) là một quyết định **riêng** đứng trước V; R-VOUCHER-08 là thứ tự báo lỗi — oracle, không phải điều kiện.

### 4.2. Điều kiện ghép và tổ hợp bất khả thi

- **C4, C5 ghép** qua biến `kinds`: rỗng thì không chứa gì ⇒ (C4, C5) = (T, T) bất khả thi. C6, C7 tương tự.
- **C2, C3 ghép** qua biến `day`: với validFrom ≤ validTo, không thể vừa trước validFrom vừa sau validTo ⇒ (C2, C3) = (F, F) bất khả thi.

Multiple condition coverage vì thế không phải 512 mà 288 tổ hợp khả thi (mục 2.3, Hình 1), và còn ít hơn nữa nếu tính ngắt mạch. MC/DC chỉ cần **các cặp còn lật được**: với OR `C4 ∨ C5`, cặp cho C5 là (F, T) ↔ (F, F) và cặp cho C4 là (T, F) ↔ (F, F) — cả bốn vectơ đều khả thi, nên ghép ở đây **không** chặn MC/DC unique-cause.

### 4.3. Thiết kế bộ N + 1

Cách làm từng bước:

1. **Ca gốc M1** cho V = TRUE. Với nhóm OR, chọn vế «chứa» đúng và vế «rỗng» sai (C4 = F, C5 = T), để còn chỗ lật.
2. **Lật lần lượt từng điều kiện của phần AND** (C1, C2, C3, C8, C9) ⇒ 5 ca, mỗi ca ghép cặp với M1.
3. **Nhóm OR**: lật C5 (M5: `kinds = [food]` ⇒ (F, F), V = F) ghép với M1. Muốn chứng minh C4 cần thêm M6 (T, F) ⇒ V = T, ghép với **M5**, không phải M1. Nhóm C6/C7 làm y hệt (M7, M8).
4. Đếm: 1 + 5 + 2 + 2 = **10 = N + 1**.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 380" role="img" aria-label="Quyết định V: C4 và C5 vào một cổng OR, C6 và C7 vào cổng OR khác; hai cổng OR cùng C1, C2, C3, C8, C9 vào cổng AND cho ra V. Bên phải: cặp độc lập của từng điều kiện, C4 ghép M5 với M6, C6 ghép M7 với M8, các điều kiện còn lại ghép với ca gốc M1.">
<defs>
<marker id="t047-arr2" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="16" y="20" width="230" height="28" rx="6" class="f-surface s-line"/><text x="26" y="39" class="t-small">C1 tìm thấy mã · R-03</text>
<rect x="16" y="58" width="230" height="28" rx="6" class="f-surface s-line"/><text x="26" y="77" class="t-small">C2 ngày ≥ validFrom · R-04</text>
<rect x="16" y="96" width="230" height="28" rx="6" class="f-surface s-line"/><text x="26" y="115" class="t-small">C3 ngày ≤ validTo · R-04</text>
<rect x="16" y="134" width="230" height="28" rx="6" class="f-pk-soft s-pk"/><text x="26" y="153" class="t-small">C4 kinds rỗng · R-05</text>
<rect x="16" y="172" width="230" height="28" rx="6" class="f-pk-soft s-pk"/><text x="26" y="191" class="t-small">C5 kinds chứa dịch vụ · R-05</text>
<rect x="16" y="210" width="230" height="28" rx="6" class="f-pk-soft s-pk"/><text x="26" y="229" class="t-small">C6 cityCodes rỗng · R-05</text>
<rect x="16" y="248" width="230" height="28" rx="6" class="f-pk-soft s-pk"/><text x="26" y="267" class="t-small">C7 cityCodes chứa TP · R-05</text>
<rect x="16" y="286" width="230" height="28" rx="6" class="f-surface s-line"/><text x="26" y="305" class="t-small">C8 A ≥ minAmount · R-06</text>
<rect x="16" y="324" width="230" height="28" rx="6" class="f-surface s-line"/><text x="26" y="343" class="t-small">C9 còn lượt hôm nay · R-07</text>
<rect x="290" y="140" width="60" height="56" rx="8" class="f-pk-soft s-pk"/><text x="320" y="173" text-anchor="middle" class="t-small t-b">OR</text>
<rect x="290" y="216" width="60" height="56" rx="8" class="f-pk-soft s-pk"/><text x="320" y="249" text-anchor="middle" class="t-small t-b">OR</text>
<rect x="420" y="20" width="70" height="332" rx="8" class="f-phys-soft s-phys"/><text x="455" y="191" text-anchor="middle" class="t-b t-phys">AND</text>
<rect x="560" y="166" width="80" height="40" rx="8" class="f-warn-soft s-warn"/><text x="600" y="191" text-anchor="middle" class="t-b">V</text>
<line x1="246" y1="34" x2="418" y2="34" class="s-muted" marker-end="url(#t047-arr2)"/>
<line x1="246" y1="72" x2="418" y2="72" class="s-muted" marker-end="url(#t047-arr2)"/>
<line x1="246" y1="110" x2="418" y2="110" class="s-muted" marker-end="url(#t047-arr2)"/>
<line x1="246" y1="148" x2="288" y2="156" class="s-muted" marker-end="url(#t047-arr2)"/>
<line x1="246" y1="186" x2="288" y2="182" class="s-muted" marker-end="url(#t047-arr2)"/>
<line x1="246" y1="224" x2="288" y2="232" class="s-muted" marker-end="url(#t047-arr2)"/>
<line x1="246" y1="262" x2="288" y2="258" class="s-muted" marker-end="url(#t047-arr2)"/>
<line x1="350" y1="168" x2="418" y2="168" class="s-muted" marker-end="url(#t047-arr2)"/>
<line x1="350" y1="244" x2="418" y2="244" class="s-muted" marker-end="url(#t047-arr2)"/>
<line x1="246" y1="300" x2="418" y2="300" class="s-muted" marker-end="url(#t047-arr2)"/>
<line x1="246" y1="338" x2="418" y2="338" class="s-muted" marker-end="url(#t047-arr2)"/>
<line x1="490" y1="186" x2="558" y2="186" class="s-muted" marker-end="url(#t047-arr2)"/>
<text x="680" y="36" class="t-small t-b">Cặp độc lập (unique-cause)</text>
<text x="680" y="66" class="t-small t-mono">C1  M1 ↔ M2</text>
<text x="680" y="92" class="t-small t-mono">C2  M1 ↔ M3</text>
<text x="680" y="118" class="t-small t-mono">C3  M1 ↔ M4</text>
<text x="680" y="144" class="t-small t-mono t-pk">C4  M5 ↔ M6</text>
<text x="680" y="170" class="t-small t-mono t-pk">C5  M1 ↔ M5</text>
<text x="680" y="196" class="t-small t-mono t-pk">C6  M7 ↔ M8</text>
<text x="680" y="222" class="t-small t-mono t-pk">C7  M1 ↔ M7</text>
<text x="680" y="248" class="t-small t-mono">C8  M1 ↔ M9</text>
<text x="680" y="274" class="t-small t-mono">C9  M1 ↔ M10</text>
<text x="680" y="314" class="t-small">M1 = ca gốc, V = TRUE</text>
<text x="680" y="336" class="t-small">10 ca = N + 1, N = 9</text>
</svg>
</div>
<figcaption><b>Hình 2 · Cấu trúc V và cặp độc lập của từng điều kiện.</b> Điều kiện trong nhóm OR (màu hồng) không ghép cặp được hết với ca gốc: C4 phải ghép M5 ↔ M6.</figcaption>
</figure>

Bộ test (T đúng, F sai, «–» không đánh giá). Đặt lúc 09:00 04/11/2026 giờ VN (ngoài cao điểm, không mưa); xe máy SG 7,4 km ⇒ A = 12.000 + 4.500 × 7,4 = **45.300 đ**. Ca gốc: giảm 10%, trần 20.000 đ, `minAmount` 45.300 đ, `kinds = [ride, parcel]`, `cityCodes = [SG, HN]`, hạn 04/11–04/11, 1 lượt/ngày.

| Ca | Dữ liệu khác ca gốc | C1 | C2 | C3 | C4 | C5 | C6 | C7 | C8 | C9 | V | Kết quả mong đợi |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| M1 | — | T | T | T | F | T | F | T | T | T | **T** | tổng 41.000 đ, giảm 4.530 đ |
| M2 | khách gõ `KHONGCO` | F | – | – | – | – | – | – | – | – | F | `VOUCHER_NOT_FOUND` |
| M3 | hạn 05/11–30/11 | T | F | T | F | T | F | T | T | T | F | `VOUCHER_EXPIRED` |
| M4 | hạn 01/11–03/11 | T | T | F | F | T | F | T | T | T | F | `VOUCHER_EXPIRED` |
| M5 | `kinds = [food]` | T | T | T | F | F | F | T | T | T | F | `VOUCHER_NOT_APPLICABLE` |
| M6 | `kinds = []` | T | T | T | T | F | F | T | T | T | **T** | 41.000 đ, giảm 4.530 đ |
| M7 | `cityCodes = [HN]` | T | T | T | F | T | F | F | T | T | F | `VOUCHER_NOT_APPLICABLE` |
| M8 | `cityCodes = []` | T | T | T | F | T | T | F | T | T | **T** | 41.000 đ, giảm 4.530 đ |
| M9 | `minAmount` 45.301 đ | T | T | T | F | T | F | T | F | T | F | `VOUCHER_MIN_AMOUNT` |
| M10 | đã đặt 1 chuyến với mã hôm nay | T | T | T | F | T | F | T | T | F | F | `VOUCHER_USAGE_LIMIT` |

Kết quả mong đợi tính theo R-FARE-03: 45.300 − 4.530 = 40.770 ≥ 15.000 ⇒ làm tròn lên **41.000 đ**.

### 4.4. Chọn giá trị: ghép MC/DC với BVA

MC/DC chỉ đòi điều kiện nhận T/F, không đòi giá trị nào. Tận dụng tự do đó: ca gốc đặt **ngay biên** — `minAmount` = A, validFrom = validTo = ngày đặt — nên M1 đồng thời kiểm «tính cả hai đầu» của R-VOUCHER-04/06; M9 lấy A + 1 đồng. Một bộ, hai kỹ thuật ([TEST-032](/articles/boundary-value-analysis)).

### 4.5. Từ quyết định đặc tả xuống quyết định trong code

Code không có biểu thức V; nó là **chuỗi guard** ném lỗi. Mỗi guard là một quyết định nhỏ, chỉ được đánh giá khi các guard trước đã qua, và `||` ngắt mạch vế phải. Kiểm lại: 10 ca có đạt MC/DC trên **từng** quyết định của code không?

| Quyết định trong code | Số điều kiện | Ca đi tới | Cặp độc lập |
|---|---|---|---|
| `voucher.ts:34` `day < validFrom \|\| day > validTo` | 2 | M1, M3–M10 | M1 ↔ M3 (vế phải không đánh giá ở M3), M1 ↔ M4 |
| `voucher.ts:40` `kinds.length === 0 \|\| kinds.includes(…)` | 2 | M1, M5–M10 | M5 ↔ M6, M1 ↔ M5 |
| `voucher.ts:41` `cityCodes…` (như trên) | 2 | M1, M5–M10 | M7 ↔ M8, M1 ↔ M7 |
| `voucher.ts:42` `!kindOk \|\| !cityOk` | 2 | M1, M5–M10 | M1 ↔ M5, M1 ↔ M7 |
| `voucher.ts:52` `amountMilli < minAmount` | 1 | M1, M6, M8, M9, M10 | M1 ↔ M9 |
| `voucher.ts:63` `usedToday >= perCustomerPerDay` | 1 | M1, M6, M8, M10 | M1 ↔ M10 |

Đạt — vì đặc tả và code **cùng cấu trúc logic**; phần C của `mcdc-voucher.test.ts` kiểm lại bằng code.

## 5. Thực hành

### 5.1. Đề bài (bài thực hành nâng cao Series 05)

> Thiết kế bộ test cho thuật toán áp voucher (R-VOUCHER-02…08) đạt **100% statement**, **100% branch** trên `src/domain/voucher.ts` và `src/app/quote.ts`, và **MC/DC** cho quyết định hợp lệ V. Chạy, nộp báo cáo coverage.

Lời giải trong `examples/toi-noi-testing/articles/mc-dc-testing/`:

`mcdc.ts` (bộ kiểm MC/DC tối giản — V8 không đo được MC/DC) · `voucher-cases.ts` (10 ca, hàm tính vectơ điều kiện **từ dữ liệu theo SPEC**, bộ chạy `priceQuote` thật) · `mcdc-voucher.test.ts` (A kiểm thiết kế, B chạy 10 ca, C MC/DC trên từng quyết định code) · `bo-sung-statement-branch.test.ts` · `buggy-eligible.ts` + `bat-loi-and-or.test.ts` · `bao-cao-coverage.md` (báo cáo nộp bài).

### 5.2. Kiểm thiết kế bằng code, không bằng mắt

Vectơ điều kiện của mỗi ca **không gõ tay**: `conditionsOf` tính từ dữ liệu ca theo đúng câu chữ SPEC, độc lập với code. Bộ kiểm tìm cặp chỉ khác nhau ở một điều kiện (điều kiện không đánh giá là «không quan tâm») mà kết quả khác nhau:

```ts
// mcdc.ts — hai vectơ chỉ khác ở điều kiện i
export function differsOnlyAt(x: readonly CondValue[], y: readonly CondValue[], i: number): boolean {
  const xi = x[i];
  const yi = y[i];
  if (xi === null || yi === null || xi === undefined || yi === undefined || xi === yi) return false;
  return x.every((v, j) => j === i || v === null || y[j] === null || v === y[j]);
}
```


Ca gốc `BASE` nằm ngay biên; M10 tiêu lượt **thật** (`createQuote` + `createTrip` một chuyến với mã, rồi báo giá lại).

### 5.3. Chạy

```bash
npx vitest run articles/mc-dc-testing --reporter=verbose
```

Trích kết quả thật (05/11/2026, vitest 3.2.7, Node 22.18):

```text
✓ A · thiết kế bộ MC/DC > 10 ca = N + 1 và mọi điều kiện có cặp độc lập ⇒ MC/DC 100%
✓ A · thiết kế bộ MC/DC > bỏ M6 thì mất cặp của C4 ⇒ MC/DC còn 8/9 = 88,9%
✓ B · chạy 10 ca MC/DC trên src/app/quote.ts > M10 · C9 sai: đã dùng 1/1 lượt hôm nay
✓ C · cùng 10 ca đạt MC/DC trên TỪNG quyết định của code > voucher.ts:42 !kindOk || !cityOk
…
Test Files  3 passed (3)
     Tests  33 passed (33)
```

Mọi kết quả thực tế khớp kết quả mong đợi ở bảng 4.3; không có failure trên `src/`.

### 5.4. Đo statement và branch coverage

Lần 1 — **chỉ** tệp chứa 10 ca MC/DC:

```bash
npx vitest run articles/mc-dc-testing/mcdc-voucher.test.ts --coverage.enabled \
  --coverage.include=src/domain/voucher.ts --coverage.include=src/app/quote.ts \
  --coverage.reporter=text --coverage.reportsDirectory=reports/coverage-mc-dc-testing
```

```text
File         | % Stmts | % Branch | % Funcs | % Lines | Uncovered Line #s
All files    |   98.05 |    82.75 |     100 |   98.05 |
  quote.ts   |   96.77 |    76.92 |     100 |   96.77 | 49-50
  voucher.ts |     100 |     87.5 |     100 |     100 | 24-25
```

MC/DC 100% trên V mà branch của tệp chỉ 82,75%. Không mâu thuẫn với mục 2.3: các nhánh thiếu thuộc **quyết định khác** — `raw ?? ''` và `code === '' ? null : code` (chuẩn hoá), `if (code === null) return null` và `voucher?.code ?? null` (không nhập voucher), nhánh ném của `requireRole`. Thêm 6 ca bổ sung:

S1 không gửi `voucherCode` (tổng 46.000 đ; phủ `?? ''`, `return null`, `?? null`) · S2 mã `'   '` (phủ `? null`) · S3 `' mcdc10 '` ≡ `MCDC10` (R-VOUCHER-02) · S4 `createQuote` lưu báo giá · S5 tài xế báo giá ⇒ `FORBIDDEN` · S6 `requireRole(khách, 'driver')` (vế còn lại của `?:`).

Lần 2 — cả thư mục:

```text
File         | % Stmts | % Branch | % Funcs | % Lines | Uncovered Line #s
All files    |     100 |      100 |     100 |     100 |
  quote.ts   |     100 |      100 |     100 |     100 |
  voucher.ts |     100 |      100 |     100 |     100 |
```

Đ1, Đ2 **không** tăng coverage nhưng vẫn cần: Đ1 kiểm trần giảm (50% × 45.300 = 22.650 > trần 5.000 ⇒ giảm 5.000; `Math.min` không phải nhánh), Đ2 kiểm R-VOUCHER-08 khi ba lỗi cùng lúc (chỉ `VOUCHER_EXPIRED`).

### 5.5. Bằng chứng: decision coverage để lọt, MC/DC bắt được

`buggy-eligible.ts` chép `assertVoucherEligible` và đổi `!kindOk || !cityOk` thành `!kindOk && !cityOk` — chỉ chặn khi sai **cả** dịch vụ **lẫn** thành phố. Một bộ 3 ca «decision coverage» (đúng hết; sai cả hai; hết hạn) chạy trên bản lỗi:

```bash
npx vitest run articles/mc-dc-testing/bat-loi-and-or.test.ts -t "decision coverage" --coverage.enabled \
  --coverage.include=articles/mc-dc-testing/buggy-eligible.ts --coverage.reporter=text \
  --coverage.reportsDirectory=reports/coverage-mc-dc-testing
```

```text
✓ bat-loi-and-or.test.ts (5 tests | 2 skipped)
File               | % Stmts | % Branch | % Funcs | % Lines
 buggy-eligible.ts |     100 |      100 |     100 |     100
```

**100% statement, 100% branch — và lỗi vẫn nằm đó**: cả ba ca cho cùng kết quả trên bản đúng và bản lỗi. Thêm hai ca MC/DC M5 (chỉ sai dịch vụ) và M7 (chỉ sai thành phố): bản đúng ném `VOUCHER_NOT_APPLICABLE`, bản lỗi trả «OK» ⇒ failure lộ ra. Đây là đúng loại defect mục 2.1 nói tới: quyết định đi sai đường vì một điều kiện không còn ảnh hưởng độc lập.

### 5.6. Báo cáo nộp bài

`bao-cao-coverage.md`: đối tượng, công cụ, hai lần đo statement/branch (10 ca: 98,05% / 82,75%; 18 ca: 100% / 100%), bảng 9 cặp độc lập, kết luận về bản lỗi. Báo cáo phải nói rõ **đo bằng gì**: V8 đo statement/branch theo khối mã; MC/DC ở đây là kết quả phân tích bằng `mcdc.ts` — lab không có công cụ đo MC/DC trực tiếp.

## 6. Góc nhìn nâng cao

- **MC/DC của một quyết định ≠ MC/DC của test object.** Quan hệ bao hàm (mục 2.3) chỉ đúng khi tiêu chí áp cho **mọi** quyết định. Đặt mục tiêu «MC/DC cho V» là hợp lý theo rủi ro; nhưng đừng ghi «đã đạt MC/DC» cho cả module.
- **Ngắt mạch không chặn MC/DC.** Unique-cause với «không quan tâm» (hoặc masking) vẫn cho đủ cặp — như M3 ở mục 4.5; chỉ đừng thiết kế ca dựa vào giá trị vế phải của `||` khi vế trái đúng.
- **Cặp phải đổi ĐÚNG MỘT điều kiện.** Sai lầm hay gặp: lấy M1 ghép với ca «kinds = [food], cityCodes = [HN]» rồi ghi «đã chứng minh C5» — ca đó lật cả C5 lẫn C7. Đếm N + 1 ca mà không kiểm cặp là đếm test case, không phải đo coverage.
- **MC/DC không tìm điều kiện bị thiếu.** Nếu code quên hẳn kiểm lượt lần hai khi tạo chuyến (R-VOUCHER-07 «kiểm hai lần»), không tiêu chí cấu trúc nào trên `quote.ts` phát hiện — phải có test từ đặc tả. Tương tự trần tiền giảm (Đ1) và thứ tự lỗi (Đ2).
- **Coverage là tiêu chí cấu trúc, nhạy với refactor.** Gom R-VOUCHER-04…06 vào một biểu thức hay chuyển sang bảng luật là đổi coverage item; phải phân tích lại — xem [TEST-062](/articles/code-coverage-trong-thuc-hanh) về vận hành công cụ coverage và ngưỡng trong CI.
- **Chọn kỹ thuật theo rủi ro (TTA-2.8.1, K4).** Tới Nơi không phải hệ an toàn; theo mục 2.8.1 (hợp đồng, chiến lược, phong cách code, lịch sử defect, kỹ năng, công cụ), đề xuất: MC/DC cho quyết định ra tiền/ra quyền (voucher, hoàn tiền, nhận chuyến), branch coverage cho phần còn lại, black-box trước rồi white-box bù chỗ thiếu.

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1.** R-ACCEPT-01: tài xế nhận được chuyến khi `online` ∧ cùng thành phố ∧ đúng loại xe ∧ chuyến `searching` (4 điều kiện). [TEST-046](/articles/multiple-condition-testing) liệt kê 2⁴ = 16 tổ hợp. Thiết kế bộ MC/DC tối thiểu và nêu cặp độc lập.

<details><summary><b>Lời giải</b></summary><p>Quyết định toàn AND ⇒ N + 1 = 5 ca. A1: (T, T, T, T) ⇒ nhận được. A2: tài xế <code>offline</code> (F, T, T, T) ⇒ <code>DRIVER_OFFLINE</code>. A3: khác thành phố (T, F, T, T) ⇒ <code>CITY_MISMATCH</code>. A4: khác loại xe (T, T, F, T) ⇒ <code>VEHICLE_MISMATCH</code>. A5: chuyến đã <code>accepted</code> (T, T, T, F) ⇒ <code>TRIP_ALREADY_TAKEN</code>. Cặp: điều kiện i ↔ (A1, A(i+1)). Trong code «online» là hai so sánh (<code>offline</code>, <code>busy</code>, R-ACCEPT-02) ⇒ 5 điều kiện, 6 ca. So với 16 tổ hợp — chênh lệch tăng nhanh theo N (N = 9: 10 so với 512).</p></details>

**Bài tập 2** (tình huống của kế hoạch Series 05, **giả định — chưa có trong lab**). Support/Refund Service (kiến trúc §4.13) tự duyệt hoàn tiền khi: `số tiền ≤ 200.000 đ` ∧ (`tài xế huỷ` ∨ `tài xế trễ quá 10 phút`) ∧ ¬`tài khoản bị gắn cờ gian lận`. Thiết kế bộ MC/DC, chỉ rõ ca gốc.

<details><summary><b>Lời giải</b></summary><p>Điều kiện: R1 tiền ≤ 200.000, R2 tài xế huỷ, R3 tài xế trễ, R4 không gắn cờ. Quyết định = R1 ∧ (R2 ∨ R3) ∧ R4, N = 4 ⇒ 5 ca. H1 (T, T, F, T) ⇒ duyệt — ca gốc chọn R2 = T, R3 = F để còn lật được nhóm OR. H2 (F, T, F, T) ⇒ không duyệt (cặp R1: H1 ↔ H2). H3 (T, F, F, T) ⇒ không (cặp R2: H1 ↔ H3). H4 (T, F, T, T) ⇒ duyệt (cặp R3: H3 ↔ H4). H5 (T, T, F, F) ⇒ không (cặp R4: H1 ↔ H5). Ghép BVA: H1 dùng đúng 200.000 đ, H2 dùng 200.001 đ; R3 dùng «trễ 10 phút 1 giây» và (ca khác) đúng 10 phút.</p></details>

**Câu 1 (K2).** MC/DC yêu cầu điều gì?

- a) Mọi tổ hợp giá trị của các điều kiện nguyên tử được thực thi
- b) Mỗi điều kiện nguyên tử được chứng minh, bằng một cặp test, là đổi một mình nó thì đổi kết quả quyết định
- c) Mỗi điều kiện nguyên tử nhận cả TRUE và FALSE ít nhất một lần
- d) Mỗi quyết định nhận cả TRUE và FALSE ít nhất một lần

<details><summary><b>Đáp án</b></summary><p><b>b)</b> đúng (TTA mục 2.4). a) là multiple condition coverage. c) là condition coverage — chưa chắc đổi kết quả (bài <a href="/articles/condition-coverage">TEST-045</a>). d) là decision coverage.</p></details>

**Câu 2 (K3).** Quyết định `A && B && C && D`, bốn điều kiện độc lập. Số ca tối thiểu để đạt MC/DC là bao nhiêu?

- a) 2
- b) 4
- c) 5
- d) 16

<details><summary><b>Đáp án</b></summary><p><b>c)</b> 5 = N + 1: một ca tất cả đúng, bốn ca mỗi ca lật một điều kiện thành sai. a) đủ cho decision coverage. b) thiếu ca gốc nên không có cặp. d) là multiple condition coverage.</p></details>

**Câu 3 (K3).** Quyết định `(A || B) && C`. Bộ test có các vectơ (A, B, C) → kết quả: T1 (F, T, T) → T; T2 (F, F, T) → F; T3 (T, F, T) → T; T4 (F, T, F) → F. Cặp nào chứng minh ảnh hưởng độc lập của A?

- a) T1 và T3
- b) T2 và T3
- c) T1 và T4
- d) T1 và T2

<details><summary><b>Đáp án</b></summary><p><b>b)</b> T2, T3 chỉ khác A (B = F, C = T) và kết quả khác nhau. a) khác cả A lẫn B, kết quả giống nhau. c) là cặp của C. d) là cặp của B.</p></details>

**Câu 4 (K2).** Trong TypeScript, `if (kinds.length === 0 || kinds.includes(kind))`. Ảnh hưởng của đánh giá ngắt mạch tới MC/DC là gì?

- a) Không thể đạt MC/DC cho quyết định này
- b) Khi vế trái đúng, vế phải không được đánh giá, nên ca đó không cho biết giá trị vế phải; cặp test phải được chọn tính tới điều này
- c) Ngắt mạch làm tăng số ca cần thiết lên 2^N
- d) Có thể tắt ngắt mạch bằng cờ biên dịch của TypeScript

<details><summary><b>Đáp án</b></summary><p><b>b)</b> đúng; bài vẫn đạt MC/DC bằng M5 ↔ M6 và M1 ↔ M5. a) sai — vẫn đạt được. c) sai — đó là multiple condition. d) sai — JavaScript luôn ngắt mạch, không có cờ tắt.</p></details>

**Câu 5 (K4).** Đội Tới Nơi có ngân sách hạn chế cho white-box testing. Module nào nên ưu tiên đặt mục tiêu MC/DC?

- a) Module định dạng hiển thị số tiền («52.000 đ»), không có quyết định nhiều điều kiện
- b) Module quyết định áp voucher, mỗi quyết định có tới 9 điều kiện, sai là thất thoát tiền
- c) Trang giới thiệu tĩnh của website
- d) Script sinh dữ liệu demo chỉ chạy ở máy dev

<details><summary><b>Đáp án</b></summary><p><b>b)</b> đúng: rủi ro tài chính cao và có quyết định nhiều điều kiện (mục 2.8.1 nêu phong cách code — không có điều kiện ghép thì MC/DC là lãng phí). a) không có quyết định nhiều điều kiện ⇒ MC/DC trùng decision. c) và d) rủi ro thấp.</p></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Advanced Level — Technical Test Analyst (CTAL-TTA)*, v4.0, 2021 — mục 2.1–2.5, 2.8 (TTA-2.2.1, TTA-2.3.1, TTA-2.4.1, TTA-2.8.1). https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB, *Certified Tester Foundation Level Syllabus*, v4.0.1, 2024 — mục 4.3.1, 4.3.2 (FL-4.3.1, FL-4.3.2). https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (modified condition/decision coverage, atomic condition, decision).
- ISO/IEC/IEEE 29119-4, *Software and systems engineering — Software testing — Part 4: Test techniques* (syllabus dẫn chiếu cho đặc tả và ví dụ kỹ thuật).
- J. J. Chilenski, S. P. Miller, «Applicability of modified condition/decision coverage to software testing», *Software Engineering Journal*, 9(5), 1994.
- K. J. Hayhurst và cộng sự, *A Practical Tutorial on Modified Condition/Decision Coverage*, NASA/TM-2001-210876, 2001.
- Bài liên quan: [TEST-043 · Statement Coverage](/articles/statement-coverage), [TEST-044 · Branch/Decision Coverage](/articles/branch-coverage-decision-coverage), [TEST-045 · Condition Coverage](/articles/condition-coverage), [TEST-046 · Multiple Condition Testing](/articles/multiple-condition-testing), [TEST-029 · Static Analysis](/articles/static-analysis-phan-tich-code-tinh), [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi).
- Lab: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — thư mục `articles/mc-dc-testing/`.

Bài tiếp theo: [TEST-048 · Path Testing](/articles/path-testing) — đường đi độc lập, cyclomatic complexity và vì sao không thể phủ mọi đường.
