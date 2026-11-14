---
slug: static-analysis-phan-tich-code-tinh
title: "Static Analysis: phân tích code tĩnh với tsc --strict và ESLint"
summary: "Phân tích tĩnh tìm defect trong code mà không chạy chương trình: control flow, data flow, cyclomatic complexity, chuẩn coding. Chạy thật tsc --strict và ESLint trên voucher Tới Nơi, phân loại cảnh báo thật và giả."
category: testing
level: intermediate
tags: [testing, istqb, ctal-tta, test-029, static-analysis, eslint, typescript, cyclomatic-complexity]
language: vi
---

Một hàm kiểm voucher dài 35 dòng, build xanh, chạy thử vài lần đều ra số. Không ai chạy nó lúc 06:30 sáng ngày đầu tháng, không ai thử voucher «mọi dịch vụ» với chuyến bưu kiện. Công cụ phân tích tĩnh thì không cần chạy: nó đọc cấu trúc code và chỉ ra những chỗ đáng ngờ trong vài giây. Bài này trả lời ba câu hỏi: **static analysis tìm được loại bất thường nào** (control flow, data flow, độ phức tạp, chuẩn coding), **dùng nó thế nào trên code TypeScript thật**, và **làm sao phân biệt cảnh báo thật với cảnh báo giả**.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-029</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S03 · Static Testing &amp; Review Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TTA v4.0 (kèm Foundation — CTFL v4.0.1)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Component</td></tr>
<tr><th scope="row">Test Type</th><td>White-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Static analysis</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/static-testing-la-gi">TEST-021 · Static Testing là gì?</a> · <a href="/articles/static-testing-va-dynamic-testing">TEST-022 · Static Testing và Dynamic Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): chạy <code>tsc --strict</code> và ESLint trên <code>src/domain/voucher.ts</code> và một bản có code smell, tính cyclomatic complexity hàm kiểm voucher (R-VOUCHER-08), phân loại từng cảnh báo — <code>examples/toi-noi-testing/articles/static-analysis-phan-tich-code-tinh/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TTA v4.0 · Chương 3 · mục 3.2 · TTA-3.2.1 (K3), TTA-3.2.2 (K3), TTA-3.2.3 (K3) · CTFL v4.0.1 · Chương 3 · mục 3.1 · FL-3.1.2 (K2), FL-3.1.3 (K2) · Chương 6 · mục 6.1 · FL-6.1.1 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

**Static analysis** (phân tích tĩnh) là đánh giá code hoặc kiến trúc **mà không thực thi** nó, thường bằng công cụ. Trong dự án TypeScript, bạn đã dùng nó hằng ngày dù có thể không gọi tên: trình biên dịch `tsc` báo lỗi kiểu, ESLint gạch chân biến thừa, IDE tô xám dòng không bao giờ chạy tới.

Static analysis giải quyết ba vấn đề: **đường đi hiếm** (test động chỉ chạm nhánh mà dữ liệu test dẫn vào, công cụ tĩnh đọc mọi nhánh như nhau); **chi phí và độ trễ** (không cần dữ liệu, môi trường, test case — chạy ngay khi code vừa gõ, ở bước 1 pipeline CI); và **thứ test động không đo được** (độ phức tạp, chuẩn coding, code trùng lặp).

Khi nào dùng: **luôn luôn**, như cổng tự động trước code review và dynamic testing — nhưng không thay được test động (bài 022). Bài này đi sâu phía công cụ: đọc kết quả, phân loại, biến kết quả thành đề xuất cải thiện.

## 2. Lý thuyết theo ISTQB

### 2.1. Static analysis trong CTFL và CTAL-TTA

CTFL v4.0.1 (mục 3.1) xếp static analysis cùng review vào **static testing**: tìm vấn đề trước dynamic testing với ít công sức hơn (không cần test case), thường nằm trong CI, dùng cho defect code cụ thể, maintainability và security. Sản phẩm được phân tích bằng công cụ phải **có cấu trúc để kiểm tra** (code, model, văn bản có cú pháp hình thức). Mục 6.1 gọi đây là **static testing tools**.

CTAL-TTA v4.0 (mục 3.2): mục tiêu là phát hiện defect **thật hoặc tiềm ẩn** trong code và kiến trúc, đồng thời **cải thiện maintainability**, qua ba kỹ thuật:

| LO | Kỹ thuật | Tìm gì |
|---|---|---|
| TTA-3.2.1 (K3) | **Control flow analysis** trên control flow graph | Vòng lặp sai (nhiều lối vào, không dừng), mã không tới được, hàm không ai gọi, thứ tự thao tác sai; đo **cyclomatic complexity** |
| TTA-3.2.2 (K3) | **Data flow analysis** theo vòng đời biến | Chuỗi định nghĩa / dùng / huỷ sai thứ tự |
| TTA-3.2.3 (K3) | **Static analysis để cải thiện maintainability** | Vi phạm chuẩn coding, code trùng lặp, coupling cao / cohesion thấp, vùng phức tạp cấu trúc |

### 2.2. Cyclomatic complexity

**Cyclomatic complexity** (độ phức tạp chu trình, McCabe) là số đường đi độc lập trong control flow graph. Với hàm một lối vào, một lối ra: **V(G) = E − N + 2** (E cạnh, N nút) = **số điểm quyết định nhị phân + 1**. Theo McCabe, thành phần càng phức tạp càng khó bảo trì và càng nhiều defect; syllabus khuyên **review để cân nhắc refactor** thành phần có độ phức tạp cao, ví dụ tách nhỏ. Thiết kế test theo đường đi thuộc Series 05.

### 2.3. Bất thường luồng dữ liệu

Mỗi lần biến xuất hiện là một hành động: **d** định nghĩa (`x = 3`), **u** dùng (`if (x > t)`), **k** huỷ / ra khỏi phạm vi. Chuỗi đáng ngờ:

| Chuỗi | Ví dụ TypeScript |
|---|---|
| d → d hoặc d → k, không có u ở giữa | `let x = 0; x = f();` · biến khai báo không dùng |
| d mà không bao giờ k (rò bộ nhớ) | listener, timer không gỡ |
| u hoặc k trước d | TS2454 *used before being assigned* |
| u hoặc k sau k | đọc stream sau `close()` |

Hai giới hạn syllabus nêu: một số bất thường trình biên dịch tự bắt, số khác cần công cụ riêng; và công cụ **không phải lúc nào cũng biết đường đi có khả thi** — có thể báo bất thường không bao giờ xảy ra. Đừng nhầm *data flow analysis* (tĩnh) với *data flow testing* (động, phủ cặp definition–use — Series 05).

### 2.4. Cảnh báo, không phải defect

Mục 3.2.3: công cụ **thường đưa cảnh báo chứ không khẳng định defect**; code đúng cú pháp vẫn bị cảnh báo (ví dụ độ phức tạp). Vì thế phải **phân loại**. Glossary ISTQB: **false positive** (báo có vấn đề khi không có), **false negative** (không báo khi có).

### 2.5. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa tiếng Việt |
|---|---|
| static analysis | phân tích tĩnh |
| control flow analysis | phân tích luồng điều khiển |
| cyclomatic complexity | độ phức tạp chu trình |
| data flow analysis | phân tích luồng dữ liệu |
| anomaly | bất thường |
| maintainability | khả năng bảo trì |
| coding standard | chuẩn lập trình |
| false positive / false negative | báo động giả / bỏ sót |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>TypeScript, <code>tsc --strict</code>, ESLint, typescript-eslint và tên luật là công cụ ngành, không phải nội dung thi. ESLint cộng thêm mỗi toán tử <code>&amp;&amp;</code>, <code>||</code>, <code>??</code> khi đếm độ phức tạp — gần biến thể «extended cyclomatic complexity» hơn McCabe gốc theo câu lệnh.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Khách nhập mã voucher khi báo giá. Thứ tự kiểm theo **R-VOUCHER-08**: `VOUCHER_NOT_FOUND` → `VOUCHER_EXPIRED` → `VOUCHER_NOT_APPLICABLE` → `VOUCHER_MIN_AMOUNT` → `VOUCHER_USAGE_LIMIT` — chỉ báo lỗi đầu tiên.

### Business rules (SPEC lab)

| Mã | Quy tắc tóm tắt |
|---|---|
| R-VOUCHER-01 | `kinds` hoặc `cityCodes` **rỗng** = áp dụng cho mọi dịch vụ / mọi thành phố |
| R-VOUCHER-04 | Ngày đặt **theo giờ Việt Nam** ngoài [validFrom; validTo] (tính cả hai đầu) ⇒ `VOUCHER_EXPIRED` |
| R-VOUCHER-05 | Dịch vụ hoặc thành phố ngoài phạm vi ⇒ `VOUCHER_NOT_APPLICABLE` |
| R-VOUCHER-06 | A < `minAmount` ⇒ `VOUCHER_MIN_AMOUNT`; tiền giảm = **min(percent% × A, maxDiscount)** |
| R-VOUCHER-07 | Số chuyến chưa huỷ đã dùng mã hôm nay phải **nhỏ hơn** `perCustomerPerDay` |
| R-GEN-02 | Mọi phép tính ngày giờ theo giờ Việt Nam, không phụ thuộc múi giờ máy chủ |

### System context

Theo kiến trúc tổng thể, voucher thuộc **Promotion Service** (§4.10), được Pricing & Quote gọi khi báo giá. Pipeline mỗi service (§17) có bước 1 là **lint/static analysis**, trước unit test. Trong lab (MVP modular monolith §18), logic nằm ở `src/domain/voucher.ts` (4 hàm nhỏ) và `src/app/quote.ts` (`loadEligibleVoucher`, `priceQuote` ghép thứ tự R-VOUCHER-08).

### Rủi ro có thể xảy ra

- Voucher «mọi dịch vụ» bị từ chối ⇒ khách bỏ đặt, chiến dịch thất bại.
- Bỏ trần `maxDiscount` ⇒ thất thoát ngân sách. Giả định 10.000 chuyến/ngày dùng GIAM20, A trung bình 100.000 đ: giảm 20.000 thay vì 15.000 đ ⇒ lệch 10.000 × 5.000 = **50 triệu đ/ngày**.
- So ngày theo UTC ⇒ từ 00:00 tới 06:59 giờ VN, voucher hết hạn hôm qua vẫn dùng được.
- Hàm phình to ⇒ lần sửa sau (thêm điều kiện «khách mới») dễ gây regression.

## 4. Phân tích kỹ thuật

### 4.1. Hai đối tượng phân tích

1. **Mã thật** `src/domain/voucher.ts` + `src/app/quote.ts` — đã tách nhỏ, có test.
2. **Bản smelly** `voucher-smelly.mts` trong thư mục bài — cùng logic R-VOUCHER-03…07 gộp vào **một hàm** `checkVoucher`, gài bốn defect thật và vài cảnh báo vô hại. Đuôi `.mts` để `npm run typecheck` của lab không quét tệp cố ý lỗi này.

Đoạn chính của bản smelly (giữ số dòng để đối chiếu với kết quả công cụ):

```ts
23 export function checkVoucher(input: VoucherCheckInput, verbose) {
24   const v = input.voucher;
25   if (!v) {
26     throw new DomainError('VOUCHER_NOT_FOUND', 'Không tìm thấy voucher.');
27   }
28   const day = vnDate(input.at);
29   const utcDay = input.at.toISOString().slice(0, 10);
30   if (utcDay < v.validFrom || utcDay > v.validTo) {
31     throw new DomainError('VOUCHER_EXPIRED', `Voucher ${v.code} đã hết hạn.`);
32   }
33   if (!v.kinds || v.kinds.includes(input.kind)) {
34     if (v.cityCodes.length === 0 || v.cityCodes.includes(input.cityCode)) {
35       // phạm vi hợp lệ, đi tiếp
36     } else {
37       throw new DomainError('VOUCHER_NOT_APPLICABLE', `…`);
38     }
39   } else {
40     throw new DomainError('VOUCHER_NOT_APPLICABLE', `…`);
41   }
42   let discount = 0;
43   if (input.amount < v.minAmount) {
44     throw new DomainError('VOUCHER_MIN_AMOUNT', `Voucher ${v.code} cần đơn tối thiểu ${v.minAmount} đ.`);
45     discount = 0;
46   }
47   discount = Math.min((input.amount * v.percent) / 100, v.maxDiscount);
48   discount = (input.amount * v.percent) / 100;
49   let errorCode: ErrorCode;
50   if (input.usedToday >= v.perCustomerPerDay) {
51     errorCode = 'VOUCHER_USAGE_LIMIT';
52   }
53   if (input.usedToday > v.perCustomerPerDay) {
54     throw new DomainError(errorCode, `Bạn đã dùng hết lượt voucher ${v.code} hôm nay.`);
55   }
56   if (verbose) console.log('[voucher]', v.code, discount);
57   return discount;
58 }
```

### 4.2. Chọn lớp kiểm tra cho từng loại bất thường

| Loại bất thường (TTA) | `tsc --strict` | ESLint (luật) |
|---|---|---|
| Mã không tới được (control flow) | TS7027 (`--allowUnreachableCode false`) | `no-unreachable` |
| Điều kiện luôn đúng / sai (control flow) | — | `no-unnecessary-condition` (cần thông tin kiểu) |
| Độ phức tạp (control flow) | — | `complexity` (ngưỡng 10) |
| d → d không có u | — | `no-useless-assignment` |
| d → k không có u (biến thừa) | TS6133 (`--noUnusedLocals`) | `no-unused-vars` |
| u trước d | TS2454 (`strictNullChecks`, trong `--strict`) | — |
| Tham số không kiểu | TS7006 (`noImplicitAny`, trong `--strict`) | — |
| Chuẩn coding | — | `no-console`, `restrict-template-expressions` |

Trình biên dịch giỏi kiểu và gán xác định; linter giỏi mẫu code và chuẩn của đội. `eslint.config.js` của bài (ESLint 9 flat config + typescript-eslint 8 có thông tin kiểu) bật đúng các luật trong bảng.

### 4.3. Cyclomatic complexity của hàm kiểm voucher

Trong mã thật, R-VOUCHER-04/05 nằm ở `assertVoucherEligible`. Dựng control flow graph, tách mỗi toán tử `||` thành một điểm quyết định (vì `||` ngắt mạch tạo hai đường đi):

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 340" role="img" aria-label="Control flow graph của assertVoucherEligible: sáu nút quyết định (day nhỏ hơn from, day lớn hơn to, kinds rỗng, cities rỗng, kindOk sai, cityOk sai), hai nút gán kindOk và cityOk, hai nút ném lỗi EXPIRED và NOT_APPLICABLE, nút vào và nút ra. 12 nút, 17 cạnh, V(G) bằng 7, khớp ESLint.">
<defs>
<marker id="t029-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="16" y="40" width="56" height="36" rx="18" class="f-surface s-line"/><text x="44" y="63" text-anchor="middle" class="t-small">vào</text>
<rect x="96" y="40" width="104" height="36" rx="6" class="f-phys-soft s-phys"/><text x="148" y="63" text-anchor="middle" class="t-small t-mono">day &lt; from?</text>
<rect x="224" y="40" width="104" height="36" rx="6" class="f-phys-soft s-phys"/><text x="276" y="63" text-anchor="middle" class="t-small t-mono">day &gt; to?</text>
<rect x="352" y="40" width="104" height="36" rx="6" class="f-phys-soft s-phys"/><text x="404" y="63" text-anchor="middle" class="t-small t-mono">kinds rỗng?</text>
<rect x="520" y="40" width="104" height="36" rx="6" class="f-phys-soft s-phys"/><text x="572" y="63" text-anchor="middle" class="t-small t-mono">cities rỗng?</text>
<rect x="676" y="40" width="96" height="36" rx="6" class="f-phys-soft s-phys"/><text x="724" y="63" text-anchor="middle" class="t-small t-mono">!kindOk?</text>
<rect x="796" y="40" width="96" height="36" rx="6" class="f-phys-soft s-phys"/><text x="844" y="63" text-anchor="middle" class="t-small t-mono">!cityOk?</text>
<rect x="400" y="120" width="112" height="34" rx="6" class="f-surface s-line"/><text x="456" y="142" text-anchor="middle" class="t-small t-mono">kindOk=includes</text>
<rect x="568" y="120" width="112" height="34" rx="6" class="f-surface s-line"/><text x="624" y="142" text-anchor="middle" class="t-small t-mono">cityOk=includes</text>
<rect x="132" y="200" width="176" height="36" rx="6" class="f-warn-soft s-warn"/><text x="220" y="223" text-anchor="middle" class="t-small t-mono">throw EXPIRED</text>
<rect x="676" y="200" width="200" height="36" rx="6" class="f-warn-soft s-warn"/><text x="776" y="223" text-anchor="middle" class="t-small t-mono">throw NOT_APPLICABLE</text>
<rect x="400" y="282" width="112" height="36" rx="18" class="f-surface s-line"/><text x="456" y="305" text-anchor="middle" class="t-small">ra</text>
<line x1="72" y1="58" x2="94" y2="58" class="s-muted" marker-end="url(#t029-arrow)"/>
<line x1="200" y1="58" x2="222" y2="58" class="s-muted" marker-end="url(#t029-arrow)"/><text x="211" y="34" text-anchor="middle" class="t-10">F</text>
<line x1="328" y1="58" x2="350" y2="58" class="s-muted" marker-end="url(#t029-arrow)"/><text x="339" y="34" text-anchor="middle" class="t-10">F</text>
<line x1="456" y1="58" x2="518" y2="58" class="s-muted" marker-end="url(#t029-arrow)"/><text x="487" y="34" text-anchor="middle" class="t-10">T</text>
<line x1="624" y1="58" x2="674" y2="58" class="s-muted" marker-end="url(#t029-arrow)"/><text x="649" y="34" text-anchor="middle" class="t-10">T</text>
<line x1="772" y1="58" x2="794" y2="58" class="s-muted" marker-end="url(#t029-arrow)"/><text x="783" y="34" text-anchor="middle" class="t-10">F</text>
<path d="M404 76 L404 118" class="f-none s-muted" marker-end="url(#t029-arrow)"/><text x="396" y="100" text-anchor="end" class="t-10">F</text>
<path d="M512 137 L540 137 L540 78" class="f-none s-muted" marker-end="url(#t029-arrow)"/>
<path d="M572 76 L572 96 L624 96 L624 118" class="f-none s-muted" marker-end="url(#t029-arrow)"/><text x="600" y="90" text-anchor="middle" class="t-10">F</text>
<path d="M680 137 L700 137 L700 78" class="f-none s-muted" marker-end="url(#t029-arrow)"/>
<path d="M148 76 L148 198" class="f-none s-warn" marker-end="url(#t029-arrow)"/><text x="156" y="140" class="t-10">T</text>
<path d="M276 76 L276 198" class="f-none s-warn" marker-end="url(#t029-arrow)"/><text x="284" y="140" class="t-10">T</text>
<path d="M740 76 L740 198" class="f-none s-warn" marker-end="url(#t029-arrow)"/><text x="748" y="140" class="t-10">T</text>
<path d="M844 76 L844 198" class="f-none s-warn" marker-end="url(#t029-arrow)"/><text x="852" y="140" class="t-10">T</text>
<path d="M892 58 L906 58 L906 300 L514 300" class="f-none s-muted" marker-end="url(#t029-arrow)"/><text x="898" y="180" text-anchor="end" class="t-10">F</text>
<path d="M220 236 L220 300 L398 300" class="f-none s-muted" marker-end="url(#t029-arrow)"/>
<path d="M776 236 L776 270 L512 290" class="f-none s-muted" marker-end="url(#t029-arrow)"/>
<text x="20" y="270" class="t-small t-b">N = 12 nút · E = 17 cạnh</text>
<text x="20" y="290" class="t-small t-b t-phys">V(G) = 17 − 12 + 2 = 7</text>
<text x="20" y="324" class="t-small">ESLint: complexity 7</text>
</svg>
</div>
<figcaption><b>Hình 1 · Control flow graph của <code>assertVoucherEligible</code>.</b> Nút xanh là điểm quyết định; mỗi vế trái của <code>||</code> là một điểm vì toán tử ngắt mạch. Coi mỗi <code>if</code> là một quyết định và bỏ qua <code>||</code> thì McCabe theo câu lệnh chỉ cho 3.</figcaption>
</figure>

Tính tương tự cho các hàm khác (đếm tay, rồi đối chiếu với ESLint ở mục 5.2):

| Hàm | Điểm quyết định | V(G) |
|---|---|---|
| `normalizeVoucherCode` | `??`, toán tử ba ngôi `? :` | 3 |
| `assertVoucherEligible` | 2 `||` trong `if` + 2 `||` trong gán + 2 vế `if (!kindOk \|\| !cityOk)` | 7 |
| `voucherDiscountMilli` | 1 `if` | 2 |
| `assertUsageAvailable` | 1 `if` | 2 |
| `loadEligibleVoucher` (quote.ts) | 2 `if` | 3 |
| `checkVoucher` (smelly) | `if(!v)`; 2 + 2 + 2 cho ba `if` có `||`; `if` minAmount; 2 `if` lượt dùng; `if (verbose)` | **12** |

Mã thật có cùng số đường đi, nhưng **chia** cho nhiều hàm có tên, mỗi hàm ≤ 7 — test riêng được, đọc một lần là hiểu.

## 5. Thực hành

Mọi lệnh chạy từ gốc lab `examples/toi-noi-testing` (Node 22.18, TypeScript 5.9.3, ESLint 9.39.5, typescript-eslint 8.72.1). Kết quả chép từ lần chạy thật.

### 5.1. tsc: không strict, strict, strict + cờ bổ sung

```bash
npx tsc -p articles/static-analysis-phan-tich-code-tinh --strict false
```
```text
(không có output — mã thoát 0)
```

```bash
npx tsc -p articles/static-analysis-phan-tich-code-tinh --strict
```
```text
articles/static-analysis-phan-tich-code-tinh/voucher-smelly.mts(23,56): error TS7006: Parameter 'verbose' implicitly has an 'any' type.
articles/static-analysis-phan-tich-code-tinh/voucher-smelly.mts(54,27): error TS2454: Variable 'errorCode' is used before being assigned.
```

```bash
npx tsc -p articles/static-analysis-phan-tich-code-tinh --strict --noUnusedLocals --noUnusedParameters --allowUnreachableCode false
```
```text
articles/static-analysis-phan-tich-code-tinh/voucher-smelly.mts(23,56): error TS7006: Parameter 'verbose' implicitly has an 'any' type.
articles/static-analysis-phan-tich-code-tinh/voucher-smelly.mts(28,9): error TS6133: 'day' is declared but its value is never read.
articles/static-analysis-phan-tich-code-tinh/voucher-smelly.mts(45,5): error TS7027: Unreachable code detected.
articles/static-analysis-phan-tich-code-tinh/voucher-smelly.mts(54,27): error TS2454: Variable 'errorCode' is used before being assigned.
```

Cùng tệp, cùng trình biên dịch: tắt `strict` **0 lỗi**, bật 2, thêm cờ 4. Kiểu tĩnh là một lớp kiểm tra có thể điều chỉnh độ chặt (lab bật `strict` sẵn). Mã thật trong cùng `tsconfig.json` của bài không có lỗi nào ở cả ba mức.

### 5.2. ESLint với cấu hình của bài

```bash
npx eslint -c articles/static-analysis-phan-tich-code-tinh/eslint.config.js src/domain/voucher.ts articles/static-analysis-phan-tich-code-tinh/voucher-smelly.mts
```
```text
...\articles\static-analysis-phan-tich-code-tinh\voucher-smelly.mts
  23:8   warning  Function 'checkVoucher' has a complexity of 12. Maximum allowed is 10  complexity
  28:9   error    'day' is assigned a value but never used                               @typescript-eslint/no-unused-vars
  33:7   error    Unnecessary conditional, value is always falsy                         @typescript-eslint/no-unnecessary-condition
  42:7   error    This assigned value is not used in subsequent statements               no-useless-assignment
  44:88  error    Invalid type "number" of template literal expression                   @typescript-eslint/restrict-template-expressions
  45:5   error    Unreachable code                                                       no-unreachable
  47:3   error    This assigned value is not used in subsequent statements               no-useless-assignment
  56:16  warning  Unexpected console statement                                           no-console

...\src\domain\voucher.ts
  66:26  error  Invalid type "number" of template literal expression  @typescript-eslint/restrict-template-expressions

✖ 9 problems (7 errors, 2 warnings)
```

In độ phức tạp **mọi** hàm bằng cách hạ ngưỡng xuống 1 trên dòng lệnh (cột «Maximum allowed is 1» lược bớt):

```bash
npx eslint -c articles/static-analysis-phan-tich-code-tinh/eslint.config.js --rule 'complexity: ["warn", 1]' src/domain/voucher.ts src/app/quote.ts
```
```text
...\src\app\quote.ts
  105:1  warning  Async function 'loadEligibleVoucher' has a complexity of 3
  (requireRole 3, priceQuote 4 — lược)
...\src\domain\voucher.ts
  23:8  warning  Function 'normalizeVoucherCode' has a complexity of 3
  29:8  warning  Function 'assertVoucherEligible' has a complexity of 7
  51:8  warning  Function 'voucherDiscountMilli' has a complexity of 2
  62:8  warning  Function 'assertUsageAvailable' has a complexity of 2
```

Số ESLint khớp bảng đếm tay ở mục 4.3 (7 cho `assertVoucherEligible`, 12 cho `checkVoucher`).

### 5.3. Phân loại từng cảnh báo

Gộp kết quả hai công cụ (chỗ trùng gộp lại), thêm một vùng bỏ sót:

| # | Dòng | Công cụ | Loại TTA | Phân loại | Lý do |
|---|---|---|---|---|---|
| F1 | 28 | TS6133, `no-unused-vars` | Data flow d → k | **Defect thật** | `day` (giờ VN) tính xong bỏ đó, dòng 30 so bằng `utcDay` ⇒ trái R-VOUCHER-04/R-GEN-02. Biến thừa là **triệu chứng** của dùng nhầm biến |
| F2 | 33 | `no-unnecessary-condition` | Control flow: điều kiện luôn sai | **Defect thật** | Mảng luôn truthy nên `!v.kinds` luôn `false` (ý định: `length === 0`) ⇒ `kinds = []` bị `NOT_APPLICABLE`, trái R-VOUCHER-01 |
| F3 | 47 | `no-useless-assignment` | Data flow d → d | **Defect thật** | Kết quả có trần bị gán đè ở dòng 48 ⇒ bỏ `maxDiscount`, trái R-VOUCHER-06 |
| F4 | 54 | TS2454 | Data flow u trước d | **Giả về bất thường, dẫn tới defect thật** | `usedToday > n` kéo theo `>= n` nên `errorCode` luôn đã gán — đường «chưa gán» không khả thi (giới hạn syllabus nêu). Nhưng hai điều kiện lệch: `>` cho qua khi đã dùng đủ lượt, trái R-VOUCHER-07 |
| F5 | 42 | `no-useless-assignment` | Data flow d → d | Vô hại | `= 0` bị gán đè trước khi đọc, không đổi kết quả |
| F6 | 45 | TS7027, `no-unreachable` | Control flow: mã không tới được | Maintainability, không gây failure | Lệnh sau `throw` không bao giờ chạy — dấu hiệu hiểu sai luồng |
| F7 | 23 | `complexity` 12 > 10 | Control flow: độ phức tạp | Chỉ báo, không phải defect | Lý do đề xuất refactor (mục 5.5) |
| F8 | 23 | TS7006 | Kiểu | Rủi ro, chưa gây failure | `verbose: any` nhận cả chuỗi `'false'` (truthy) ⇒ log ngoài ý muốn |
| F9 | 56 | `no-console` | Chuẩn coding | Vi phạm chuẩn đội | Log nên qua logger có correlation ID (kiến trúc §15.3) |
| F10 | 44 | `restrict-template-expressions` | Chuẩn coding | Giả | Cố ý in số tiền; luật đặt quá chặt |
| F11 | voucher.ts 66 | `restrict-template-expressions` | Chuẩn coding | **False positive** trên mã đúng | In số lượt là đúng ý ⇒ nới luật (`allowNumber: true`, mặc định của typescript-eslint), không sửa code |
| F12 | 29–30 | không công cụ nào báo | — | **False negative tiềm ẩn** | Xoá dòng 28 đi thì F1 vẫn còn mà không công cụ nào báo: `toISOString()` hợp lệ về kiểu. Tương tự tính tiền bằng số thực đồng thay vì milli-đồng (R-GEN-01) — công cụ không biết nghiệp vụ |

Tổng: 4 dẫn tới defect thật (F1–F4), 4 đáng sửa vì maintainability / chuẩn (F6–F9), 3 giả hoặc vô hại (F5, F10, F11), 1 bỏ sót (F12).

### 5.4. Xác nhận bằng test động

Phân loại «thật» chỉ là giả thuyết cho tới khi có bằng chứng. `xac-nhan-canh-bao.test.ts` chạy cùng test case trên mã thật (phải đạt) và bản smelly (`it.fails` — phải thất bại):

| Ca | Phát hiện | Dữ liệu | Mong đợi (SPEC) | Bản smelly cho |
|---|---|---|---|---|
| C1 | F2 | GIAM20, `kinds = []`, chuyến ride | áp dụng được | `VOUCHER_NOT_APPLICABLE` |
| C2 | F3 | A = 100.000 đ, 20%, trần 15.000 đ | giảm 15.000 đ | 20.000 đ |
| C3 | F1 | 06:30 giờ VN 01/12 (23:30 UTC 30/11), `validTo` 30/11 | `VOUCHER_EXPIRED` | cho dùng |
| C4 | F4 | đã dùng 1/1 lượt | `VOUCHER_USAGE_LIMIT` | cho qua |

```bash
npx vitest run articles/static-analysis-phan-tich-code-tinh
```
```text
 Test Files  1 passed (1)
      Tests  8 passed (8)
```

4 test case trên mã thật đạt, 4 test case `it.fails` trên bản smelly thất bại đúng dự đoán ⇒ F1–F4 là defect thật. Công cụ tĩnh chỉ ra **chỗ**, test động xác nhận **hậu quả**.

### 5.5. Đề xuất cải thiện maintainability (TTA-3.2.3)

1. **Tách hàm theo quy tắc** như mã thật: `assertVoucherEligible` (04/05), `voucherDiscountMilli` (06), `assertUsageAvailable` (07). Độ phức tạp lớn nhất 12 → 7, mỗi hàm test độc lập.
2. **Bỏ `if` lồng rỗng và `throw` trùng** (dòng 33–41): gộp thành `kindOk`, `cityOk` rồi một `throw` — code trùng lặp cũng là mục tiêu của static analysis.
3. **Bỏ biến lỗi tạm** `errorCode`: ném thẳng mã lỗi ở chỗ phát hiện.
4. **Đặt cổng ở bước 1 pipeline** (kiến trúc §17): `tsc --strict --noUnusedLocals` + ESLint `complexity: 10`; lỗi chặn merge, cảnh báo độ phức tạp buộc reviewer xem.
5. **Chỉnh luật theo kết quả phân loại**: nới `restrict-template-expressions` (F10, F11), giữ `no-useless-assignment` ở `error` vì nó bắt F3; mỗi lần tắt luật ghi `eslint-disable-next-line … -- lý do`.

## 6. Góc nhìn nâng cao

- **False positive làm mòn niềm tin.** Hàng trăm cảnh báo giả khiến đội bỏ qua cả cảnh báo thật. Bắt đầu với ít luật giá trị cao, đo tỉ lệ thật/giả rồi mới thêm. Nhưng cảnh báo giả vẫn đáng đọc — F4 dẫn tới defect ngay cạnh.
- **Giới hạn của công cụ.** Syllabus nêu: khó biết đường đi khả thi, khó theo dữ liệu trong mảng/bản ghi động và giữa luồng đồng thời. Lỗi «hai tài xế cùng nhận một chuyến» (kiến trúc §21) là vấn đề đồng thời — cần test hệ thống và tầng database (Series 08, 11).
- **Độ phức tạp thấp chưa chắc là code đúng.** Đó là chỉ báo bảo trì, không phải chất lượng chức năng (F12). McCabe gốc và ESLint còn đếm khác nhau (3 so với 7 cho cùng hàm) — ghi rõ công cụ khi đặt ngưỡng.
- **Không chỉ code.** OpenAPI, `.proto`, schema event (kiến trúc §6.6) cũng lint được, ví dụ chặn breaking change gRPC. TTA còn nói quét bảo mật (SQL injection, XSS) — R-API-14 «truy vấn dùng tham số» là thứ SAST kiểm được (Series 09).
- **Không thay review.** Công cụ không biết «rỗng nghĩa là mọi dịch vụ», nó chỉ thấy `!array` luôn sai. Ghép với checklist review (bài 028). Đo maintainability cho cả `src/domain` và chi phí một thay đổi thật là chủ đề TEST-110 (Series 09).

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — tính cyclomatic complexity.** Hàm sau tính phụ phí bưu kiện (giả định, không có trong lab):

```ts
function surcharge(kg: number, fragile: boolean, city: string): number {
  let fee = 0;
  if (kg > 5) fee += 5_000;
  if (kg > 20 && city !== 'SG') fee += 10_000;
  for (let i = 0; i < (fragile ? 2 : 1); i++) fee += 1_000;
  return fee;
}
```

(a) Tính V(G) theo cách ESLint đếm. (b) Tính theo McCabe coi mỗi `if`/`for` là một quyết định. (c) Có bất thường data flow nào không?

<details>
<summary><b>Lời giải</b></summary>
<p>(a) Điểm quyết định: <code>if (kg &gt; 5)</code> (1), <code>if (… &amp;&amp; …)</code> (1 + 1 cho <code>&amp;&amp;</code>), <code>for</code> (1), toán tử ba ngôi (1) ⇒ 5 quyết định, V(G) = <b>6</b>. (b) Ba quyết định cấp câu lệnh (2 <code>if</code>, 1 <code>for</code>) ⇒ <b>4</b> (nếu tách <code>&amp;&amp;</code> và <code>? :</code> thì quay về 6). (c) Không có: <code>fee</code> được định nghĩa rồi đọc (<code>+=</code> là u rồi d), cuối cùng được dùng ở <code>return</code>; <code>i</code> được d, u, rồi k khi rời vòng lặp — chuỗi hợp lệ.</p>
</details>

**Bài tập 2 — phân loại cảnh báo.** Chạy ESLint (cấu hình của bài) trên `src/domain/cancel.ts` và `src/domain/fare.ts` của lab bằng lệnh `npx eslint -c articles/static-analysis-phan-tich-code-tinh/eslint.config.js --rule 'complexity: ["warn", 1]' src/domain/cancel.ts src/domain/fare.ts` (hai tệp đã có trong `tsconfig.json` của bài). Với mỗi cảnh báo, ghi: dòng, luật, loại TTA, thật/giả, đề xuất.

<details>
<summary><b>Gợi ý lời giải</b></summary>
<p>Lần chạy khi viết bài: 5 problems — <code>customerCancellationFee</code> complexity 9, <code>normalizeDistanceKm</code> 4, <code>calculateFare</code> 5 (chỉ là số đo, dưới ngưỡng 10 của đội) và 2 lỗi <code>restrict-template-expressions</code> ở <code>fare.ts</code> dòng 23 (thông điệp in biên 0,5 và 100 km — giả, như F11). Cách làm chung: số đo độ phức tạp chỉ thành vấn đề khi vượt ngưỡng của đội; với <code>no-unnecessary-condition</code> phải đối chiếu SPEC xem kiểu đã loại trừ giá trị (giả) hay viết nhầm (thật); cảnh báo nào cho là thật thì viết một test case động như mục 5.4 để chứng minh.</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K2).** Phát biểu nào đúng về static analysis theo CTFL v4.0.1?

- a) Static analysis chỉ áp dụng được cho mã nguồn đã biên dịch thành công
- b) Static analysis cần sản phẩm công việc có cấu trúc để kiểm và thường được đưa vào CI
- c) Static analysis tìm failure bằng cách chạy chương trình với dữ liệu test
- d) Static analysis thay thế được code review vì công cụ hiểu quy tắc nghiệp vụ

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng (mục 3.1, 3.1.1). a) sai: model, văn bản có cú pháp hình thức cũng phân tích được. c) sai: đó là dynamic testing. d) sai: công cụ không biết nghiệp vụ (F12).</p>
</details>

**Câu 2 (K3).** Một hàm có control flow graph gồm 14 nút và 18 cạnh, một lối vào, một lối ra. Cyclomatic complexity là bao nhiêu?

- a) 4
- b) 5
- c) 6
- d) 32

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> V(G) = E − N + 2 = 18 − 14 + 2 = 6. a) quên cộng 2. b) cộng 1 thay vì 2. d) cộng E và N.</p>
</details>

**Câu 3 (K3).** Đoạn code: `let fee = base * 1.25; fee = base + rainSurcharge; return fee;`. Phân tích luồng dữ liệu cho `fee` phát hiện bất thường nào?

- a) Dùng trước khi định nghĩa
- b) Định nghĩa rồi định nghĩa lại mà không dùng ở giữa
- c) Dùng sau khi huỷ
- d) Không có bất thường vì `fee` cuối cùng được dùng

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> d → d không có u (giống F3). a), c) sai: không đọc trước khi gán, không có lệnh huỷ. d) sai: giá trị đầu bị bỏ phí vẫn là bất thường — có thể là defect (mất hệ số cao điểm).</p>
</details>

**Câu 4 (K2).** Công cụ báo «biến có thể được dùng trước khi gán» ở một đường đi mà khi phân tích kỹ thì không bao giờ xảy ra lúc chạy. Kết luận phù hợp nhất là gì?

- a) Công cụ bị lỗi, nên tắt toàn bộ luật data flow
- b) Đây là false positive: công cụ tĩnh không phải lúc nào cũng biết đường đi có khả thi hay không
- c) Đây là false negative vì defect không gây failure
- d) Đây chắc chắn là defect vì công cụ đã báo

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng (TTA 3.2.2). a) sai: một cảnh báo giả không làm luật mất giá trị (F4 còn dẫn tới defect thật). c) sai: false negative là bỏ sót vấn đề có thật. d) sai: công cụ đưa cảnh báo, không khẳng định defect (TTA 3.2.3).</p>
</details>

**Câu 5 (K2).** Theo CTAL-TTA, cách nào **không** phải là cách static analysis hỗ trợ maintainability?

- a) Kiểm tuân thủ chuẩn coding (đặt tên, chú thích, thụt lề)
- b) Tìm code trùng lặp có thể tách thành thành phần
- c) Đo coupling và cohesion
- d) Phát hiện rò bộ nhớ chỉ xuất hiện sau nhiều giờ chạy tải

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>d)</b> rò bộ nhớ lộ ra khi chạy là việc của dynamic analysis (TTA 3.3). a), b), c) đều có trong mục 3.2.3.</p>
</details>

**Câu 6 (K2).** Đội Pricing đặt `complexity: 10`. Hàm `checkVoucher` bị cảnh báo 12, toàn bộ test đều xanh. Hành động nào hợp lý nhất?

- a) Tăng ngưỡng lên 15 vì test xanh chứng tỏ hàm đúng
- b) Coi đây là chỉ báo maintainability: review hàm và cân nhắc tách nhỏ theo từng quy tắc
- c) Ghi defect mức nghiêm trọng cao vì hàm chắc chắn có lỗi
- d) Bỏ qua vì độ phức tạp chỉ liên quan hiệu năng

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng (TTA 3.2.1: review để cân nhắc refactor). a) sai: test xanh không chứng minh không có defect (bài 003). c) sai: độ phức tạp là chỉ báo, không phải defect. d) sai: nó liên quan bảo trì và mật độ defect, không phải hiệu năng.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Advanced Level — Technical Test Analyst Syllabus* **v4.0** (2021), chương 3, mục 3.1, 3.2.1–3.2.3; LO TTA-3.2.1, TTA-3.2.2, TTA-3.2.3: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024), mục 3.1 (3.1.1–3.1.3) và 6.1 Tool Support for Testing; LO FL-3.1.2, FL-3.1.3, FL-6.1.1: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (static analysis, cyclomatic complexity, data flow analysis, false-positive result).
- T. J. McCabe, «A Complexity Measure», *IEEE Transactions on Software Engineering*, SE-2(4), 1976 — nguồn của cyclomatic complexity (syllabus trích [McCabe76]).
- B. Beizer, *Software Testing Techniques*, 2nd ed., 1990 — phân loại bất thường luồng dữ liệu (syllabus trích [Beizer90]).
- Công cụ: ESLint rules — https://eslint.org/docs/latest/rules/ · typescript-eslint — https://typescript-eslint.io/rules/ · TSConfig `strict` — https://www.typescriptlang.org/tsconfig/
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-VOUCHER-01…08, R-GEN-02; mã bài tại `articles/static-analysis-phan-tich-code-tinh/`.
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-004 · Error, Defect, Failure](/articles/error-defect-failure-root-cause) · [TEST-008 · Kiểm thử trong SDLC](/articles/kiem-thu-trong-sdlc-agile-waterfall) · [TEST-021 · Static Testing là gì?](/articles/static-testing-la-gi) · [TEST-022 · Static Testing và Dynamic Testing](/articles/static-testing-va-dynamic-testing) · [TEST-028 · Checklist-based Review](/articles/checklist-based-review).

Bài tiếp theo: [Shift-left Testing — tester tham gia ngay từ giai đoạn phân tích yêu cầu](/articles/shift-left-testing).
