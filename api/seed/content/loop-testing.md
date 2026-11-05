---
slug: loop-testing
title: "Loop Testing: kiểm vòng cộng tiền món và vòng thử lại thanh toán của Tới Nơi"
summary: "Loop testing kiểu Beizer: chọn số lần lặp 0, 1, 2, điển hình, max−1, max, max+1; mở rộng cho vòng lồng nhau, kết hợp BVA và chạy thật trên vòng cộng tiền món, vòng thử lại VíXanh của lab Tới Nơi."
category: testing
level: advanced
tags: [testing, white-box, loop-testing, test-049, beizer, boundary-value-analysis, vitest, toi-noi]
language: vi
---

Đơn đồ ăn có từ 1 tới 20 dòng món. Hàm cộng tiền món chỉ là một vòng lặp ba dòng code. Đạt 100% branch coverage cho nó chỉ cần **một** test có một món. Vậy đơn 0 món, 20 món, 21 món có ai kiểm chưa? Bài này trình bày **loop testing** — kỹ thuật white-box chọn số lần lặp đặc biệt — rồi áp vào ba vòng lặp thật của lab Tới Nơi. Kết quả chạy có một defect mà mọi test «một món» đều bỏ lọt.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-049</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S05 · White-box &amp; Experience-based Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Mở rộng ngoài ISTQB (liên hệ CTFL v4.0.1, CTAL-TTA v4.0, CTAL-TA v4.0)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">Test Level</th><td>Component</td></tr>
<tr><th scope="row">Test Type</th><td>White-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Loop Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/branch-coverage-decision-coverage">TEST-044 · Branch Coverage / Decision Coverage</a>; nên đọc <a href="/articles/boundary-value-analysis">TEST-032 · Boundary Value Analysis</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): vòng cộng tiền món R-FOOD-02 với 0, 1, 2, 5, 19, 20, 21 dòng — kiểm tổng tiền và <code>VALIDATION_ERROR</code>; vòng thử lại R-PAY-04; vòng lồng nhau R-IDEM-02 — <code>examples/toi-noi-testing/articles/loop-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>Không có LO. Bối cảnh: CTFL v4.0.1 · mục 4.3.2 (quyết định thoát/tiếp tục vòng là một nhánh); CTAL-TTA v4.0 · mục 2.3 (outcome của vòng LOOP), 3.2.1 (vòng thiết kế tồi trong control flow analysis), 5.2.2 (checklist «Loops and Branches»); CTAL-TA v4.0 · mục 3.2.3 («simple loop coverage» trong scenario-based testing)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Vòng lặp là nơi defect thích trú ẩn: chỉ số bắt đầu sai một, biến cộng dồn khởi tạo nhầm chỗ, điều kiện dừng lệch một, giới hạn trên bị «phòng thủ» bằng cách cắt bớt dữ liệu. Các lỗi này có chung một đặc điểm: chúng **chỉ lộ ra ở một số lần lặp nhất định**. Test với «một giá trị điển hình» thường đi qua chúng mà không thấy gì.

**Loop testing** (kiểm thử vòng lặp) trả lời câu hỏi: *nên cho vòng chạy bao nhiêu lần trong test?* Câu trả lời là một tập số lần lặp cố định, suy ra từ cấu trúc vòng: bỏ qua, một lần, hai lần, số điển hình, và quanh giới hạn trên.

Dùng loop testing khi số lần lặp phụ thuộc dữ liệu (danh sách món, chuyến chờ, số lần thử lại), khi đặc tả có giới hạn số phần tử, hoặc khi đã đạt branch coverage mà vẫn lo trường hợp rỗng, đầy, tràn.

Bài này nằm sau [TEST-044](/articles/branch-coverage-decision-coverage) vì loop testing bắt đầu đúng ở chỗ branch coverage dừng lại.

## 2. Lý thuyết theo ISTQB

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Loop testing là kỹ thuật kinh điển của Boris Beizer (<i>Software Testing Techniques</i>, 1990), được nhiều giáo trình kỹ nghệ phần mềm phổ biến lại. Syllabus ISTQB hiện hành <b>không có</b> mục hay learning objective riêng cho kỹ thuật này. Mục 2.1 dưới đây là kiến thức ngành; mục 2.2 nêu những chỗ syllabus ISTQB chạm tới vòng lặp, để bạn biết ranh giới.</p>
</div>

### 2.1 Bốn loại vòng và cách chọn số lần lặp

Beizer chia vòng lặp thành bốn loại, mỗi loại một chiến lược.

**Vòng đơn (simple loop)** — một vòng, không chứa vòng khác. Gọi `n` là số lần lặp tối đa cho phép. Chọn các số lần lặp:

| Số lần lặp | Vì sao |
|---|---|
| 0 — bỏ qua vòng | Kiểm khởi tạo và giá trị trả về khi danh sách rỗng |
| 1 | Kiểm thân vòng chạy đúng một lần, biến cộng dồn được dùng đúng |
| 2 | Lần lặp thứ hai dùng kết quả lần thứ nhất ⇒ lộ lỗi khởi tạo trong vòng, lỗi «chỉ nhớ phần tử cuối» |
| m (điển hình, m &lt; n−1) | Hành vi bình thường |
| n − 1, n | Giới hạn trên: lỗi lệch một ở điều kiện dừng |
| n + 1 | Cố vượt giới hạn: bị từ chối đúng cách, hay bị cắt bớt lặng lẽ, hay tràn |

Bộ này thực chất là **boundary value analysis áp lên biến «số lần lặp»** (biên dưới 0/1, biên trên n), cộng thêm giá trị 2 vì lý do cấu trúc.

**Vòng lồng nhau (nested loops)** — số tổ hợp tăng theo cấp số nhân, nên Beizer đề xuất đi từ trong ra ngoài:

1. Bắt đầu ở vòng trong cùng; giữ mọi vòng ngoài ở số lần lặp **tối thiểu**.
2. Cho vòng trong chạy bộ giá trị của vòng đơn.
3. Chuyển ra vòng kế ngoài; giữ vòng trong ở giá trị **điển hình**, vòng ngoài hơn ở tối thiểu; lặp lại bước 2.
4. Tiếp tục tới vòng ngoài cùng.

**Vòng nối tiếp (concatenated loops)** — hai vòng đứng liền nhau. Nếu độc lập: kiểm như hai vòng đơn. Nếu vòng sau phụ thuộc vòng trước (dùng biến đếm hay kết quả của nó): kiểm như vòng lồng nhau.

**Vòng không cấu trúc (unstructured loops)** — nhảy vào giữa vòng, nhiều lối vào. Không thiết kế test cho chúng; đề nghị viết lại code.

### 2.2 Vòng lặp trong syllabus ISTQB

| Syllabus | Chỗ nói tới vòng lặp | Liên hệ |
|---|---|---|
| CTFL v4.0.1 · 4.3.2 | Nhánh có điều kiện gồm cả quyết định thoát/tiếp tục vòng | 100% branch coverage: một test có ≥ 1 lần lặp là đủ |
| CTAL-TTA v4.0 · 2.3 | Vòng LOOP có hai outcome đúng/sai | Như trên, ở mức decision coverage |
| CTAL-TTA v4.0 · 3.2.1 | Control flow analysis tìm vòng thiết kế tồi (nhiều lối vào, không dừng) | «Vòng không cấu trúc» của Beizer |
| CTAL-TTA v4.0 · 5.2.2 | Checklist code review «Loops and Branches» | Bản tĩnh của loop testing — xem [TEST-028](/articles/checklist-based-review) |
| CTAL-TA v4.0 · 3.2.3 | «Simple loop coverage»: 0 lần, 1 lần, nhiều lần, tối đa (nếu được) | Cùng ý tưởng, áp lên **mô hình hộp đen** (use case, activity diagram) |

Điểm cần nhớ: theo CTFL, branch coverage **không** bắt buộc chạy vòng 0 lần hay đủ n lần. Loop testing bổ sung đúng phần đó.

### 2.3 Thuật ngữ Anh – Việt

| Tiếng Anh | Tiếng Việt | Ghi chú |
|---|---|---|
| loop testing | kiểm thử vòng lặp | Beizer; ngoài ISTQB |
| simple / nested / concatenated / unstructured loop | vòng đơn / lồng nhau / nối tiếp / không cấu trúc | bốn loại của Beizer |
| loop bound | giới hạn (số lần) lặp | có thể nằm ở chỗ khác với vòng |
| accumulator | biến cộng dồn | `sum`, `total` |
| off-by-one error | lỗi lệch một | `<` với `<=`, bắt đầu từ 1 thay vì 0 |
| simple loop coverage | độ phủ vòng đơn | thuật ngữ của CTAL-TA mục 3.2.3 |

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1 Yêu cầu nghiệp vụ

Ba quy tắc của SPEC lab dẫn tới ba vòng lặp khác nhau:

- **R-FOOD-02** — Đơn đồ ăn có **1–20 dòng món**; mỗi dòng có đơn giá là bội số 1.000 đ trong [1.000; 5.000.000] và số lượng trong [1; 50]; sai ⇒ `VALIDATION_ERROR`. Tiền món = Σ đơn giá × số lượng. Kèm **R-FOOD-03** (tiền món ≥ 30.000 đ, ngược lại `SUBTOTAL_TOO_LOW`) và **R-FOOD-01/04** (tổng = tiền món − giảm + phí giao; phí giao theo công thức xe máy).
- **R-PAY-04** — Mỗi lần gọi VíXanh hết giờ sau 3 giây; hết giờ hoặc lỗi tạm thời ⇒ thử lại **tối đa 2 lần** (tổng tối đa 3 lần gọi) với **cùng khoá idempotency**; bị từ chối ⇒ dừng.
- **R-IDEM-02** — «Cùng nội dung» nghĩa là cùng JSON sau khi **sắp xếp khoá**; thứ tự khoá và khoảng trắng không quan trọng.

### 3.2 System context

Trong kiến trúc đích, tiền món được tính ở **Pricing &amp; Quote Service** (§4.3 tài liệu kiến trúc) khi Customer App gửi đơn qua API Gateway; vòng thử lại nằm ở **Payment &amp; Wallet Service** (§4.9), theo nguyên tắc «deadline bắt buộc, retry có giới hạn» (§7.2, §14); vân tay yêu cầu (request fingerprint) thuộc **Booking Service** (§4.4). Lab gộp cả ba vào một modular monolith (SPEC R-ARCH-01):

| Vòng | Mã nguồn lab | Giới hạn số lần lặp đặt ở đâu |
|---|---|---|
| Cộng tiền món | `itemsSubtotal` — `items.reduce(...)` trong `src/domain/food.ts` | `src/api/schemas.ts`: `z.array(foodItem).min(1).max(20)` — **không** nằm trong `food.ts` |
| Thử lại thanh toán | `for (attempt = 1; attempt <= 1 + maxRetries; …)` trong `src/app/payment.ts` | `config.payment.maxRetries` (mặc định 2) |
| Vân tay JSON | `canonicalJson` (đệ quy: mảng ⇒ `map`, object ⇒ `sort` + `map`) trong `src/api/idempotency.ts` | Không có — phụ thuộc kích thước thân (≤ 10 KB, R-API-01) |

### 3.3 Rủi ro có thể xảy ra

| Mã | Rủi ro | Hậu quả |
|---|---|---|
| RK-1 | Lệch một ở chỉ số đầu/cuối | Thu thiếu một món trên mọi đơn |
| RK-2 | Biến cộng dồn khởi tạo trong vòng | Đơn nhiều món chỉ tính món cuối |
| RK-3 | Chặn trên viết lệch (`< 19`) | Đơn đúng 20 món tính thiếu |
| RK-4 | Cắt bớt thay vì từ chối | Món thứ 21 được giao mà không tính tiền |
| RK-5 | Biến cộng dồn vượt kiểu dữ liệu nơi lưu | Báo giá 200 rồi đặt chuyến 500 |
| RK-6 | Thử lại quá 3 lần, hoặc đổi khoá giữa chừng | Gọi cổng thừa; đổi khoá ⇒ có thể trừ tiền hai lần |
| RK-7 | Không dừng khi bị từ chối | Spam cổng, chậm `/complete` |

## 4. Phân tích kỹ thuật

### 4.1 Vẽ vòng và tìm giới hạn

Bước đầu tiên là đọc code để biết **vòng nằm đâu, giới hạn nằm đâu**. Với tiền món, hai thứ ở hai tệp khác nhau (Hình 1):

- Ở **tầng domain** (`priceFood`), 0 dòng không bị chặn bởi giới hạn số dòng mà rơi vào quy tắc khác (`SUBTOTAL_TOO_LOW`), còn 21 dòng được tính bình thường.
- Ở **tầng API** (`POST /api/quotes`), 0 và 21 dòng bị zod chặn ⇒ 400 `VALIDATION_ERROR`.


<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Vòng cộng tiền món: sum bằng 0, còn dòng thì cộng giá nhân số lượng rồi lặp, hết dòng trả sum. Số lần lặp chọn: 0, 1, 2, 5, 19, 20, 21. Giới hạn 1–20 dòng nằm ở schemas.ts, không ở food.ts.">
<defs>
<marker id="t049-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-b">Vòng cộng tiền món — cấu trúc vòng và các số lần lặp được chọn</text>
<rect x="20" y="44" width="250" height="40" rx="6" class="f-warn-soft s-warn"/>
<text x="145" y="62" text-anchor="middle" class="t-mono t-small">api/schemas.ts</text>
<text x="145" y="77" text-anchor="middle" class="t-10">items: .min(1).max(20) ⇒ 400</text>
<line x1="145" y1="84" x2="145" y2="110" class="s-pk" stroke-width="1.5" marker-end="url(#t049-arrow)"/>
<rect x="45" y="112" width="200" height="34" rx="6" class="f-surface s-pk"/>
<text x="145" y="134" text-anchor="middle" class="t-mono">sum = 0</text>
<line x1="145" y1="146" x2="145" y2="166" class="s-pk" stroke-width="1.5" marker-end="url(#t049-arrow)"/>
<path d="M145 168 L215 198 L145 228 L75 198 Z" class="f-pk-soft s-pk"/>
<text x="145" y="202" text-anchor="middle" class="t-mono t-small">còn dòng?</text>
<line x1="215" y1="198" x2="296" y2="198" class="s-pk" stroke-width="1.5" marker-end="url(#t049-arrow)"/>
<text x="255" y="191" text-anchor="middle" class="t-10">đúng</text>
<rect x="298" y="180" width="200" height="36" rx="6" class="f-surface s-pk"/>
<text x="398" y="203" text-anchor="middle" class="t-mono t-small">sum += giá × SL</text>
<path d="M398 180 C 398 150, 230 150, 185 178" class="f-none s-pk" stroke-width="1.5" marker-end="url(#t049-arrow)"/>
<line x1="145" y1="228" x2="145" y2="256" class="s-pk" stroke-width="1.5" marker-end="url(#t049-arrow)"/>
<text x="160" y="246" class="t-10">sai</text>
<rect x="45" y="258" width="200" height="30" rx="6" class="f-phys-soft s-phys"/>
<text x="145" y="278" text-anchor="middle" class="t-mono t-small t-phys">return sum</text>
<text x="398" y="250" text-anchor="middle" class="t-10">domain/food.ts · itemsSubtotal</text>
<text x="398" y="266" text-anchor="middle" class="t-10">không biết gì về giới hạn 20</text>
<rect x="540" y="44" width="360" height="244" rx="8" class="f-bg s-muted"/>
<text x="720" y="66" text-anchor="middle" class="t-b t-small">Số lần lặp chọn (n = 20)</text>
<text x="560" y="94" class="t-mono t-small">0</text><text x="610" y="94" class="t-small">bỏ qua — API 400; domain SUBTOTAL_TOO_LOW</text>
<text x="560" y="122" class="t-mono t-small">1</text><text x="610" y="122" class="t-small">một lần</text>
<text x="560" y="150" class="t-mono t-small">2</text><text x="610" y="150" class="t-small">lần hai dùng kết quả lần một</text>
<text x="560" y="178" class="t-mono t-small">5</text><text x="610" y="178" class="t-small">điển hình</text>
<text x="560" y="206" class="t-mono t-small">19</text><text x="610" y="206" class="t-small">max − 1</text>
<text x="560" y="234" class="t-mono t-small">20</text><text x="610" y="234" class="t-small">max</text>
<text x="560" y="262" class="t-mono t-small t-warn">21</text><text x="610" y="262" class="t-small t-warn">max + 1 — API 400; domain vẫn cộng đủ</text>
</svg>
</div>
<figcaption><b>Hình 1 · Vòng và giới hạn của nó ở hai tệp khác nhau.</b> Vòng <code>reduce</code> trong <code>food.ts</code> chạy bao nhiêu lần cũng được; chỉ lớp API mới giữ quy tắc 1–20 dòng. Kiểm max+1 ở đâu quyết định bạn đang kiểm vòng hay kiểm bộ kiểm tra đầu vào.</figcaption>
</figure>

### 4.2 Dữ liệu test: mỗi lần lặp phải để lại dấu vết riêng

Nếu mọi dòng cùng giá, «bỏ dòng đầu» và «bỏ dòng cuối» cho cùng một tổng sai; một số lỗi (như «chỉ giữ dòng cuối» với 1 dòng) còn bị che hẳn. Quy tắc chọn dữ liệu:

- Dòng thứ i (tính từ 1) có đơn giá **30.000 + 1.000 × (i − 1)**, số lượng 1. Mỗi dòng góp một số tiền khác nhau; dòng 1 đã đủ tiền món tối thiểu (R-FOOD-03).
- **Oracle không dùng vòng lặp**: tiền món của n dòng = 30.000·n + 1.000·n(n − 1)/2. Nếu oracle cũng viết bằng `reduce`, nó có thể sai cùng kiểu với code được kiểm.

| n | Tiền món (oracle) | Tổng tại API (phí giao 2,0 km ngoài cao điểm = 21.000 đ) |
|---:|---:|---:|
| 0 | 0 | 400 `VALIDATION_ERROR` |
| 1 | 30.000 | 51.000 |
| 2 | 61.000 | 82.000 |
| 5 | 160.000 | 181.000 |
| 19 | 741.000 | 762.000 |
| 20 | 790.000 | 811.000 |
| 21 | 840.000 (domain) | 400 `VALIDATION_ERROR` |

### 4.3 Mỗi số lần lặp bắt được lỗi nào

Để thấy giá trị của từng số lần lặp, dựng bốn bản cài đặt sai (nằm trong thư mục bài, không sửa `src/`), mỗi bản ứng với một rủi ro ở mục 3.3. Dự đoán trước khi chạy (X = test đỏ, tức phát hiện được):

| Bản (rủi ro) / n | 0 | 1 | 2 | 5 | 19 | 20 | 21 |
|---|---|---|---|---|---|---|---|
| L1 `for (let i = 1; …)` — bỏ dòng đầu (RK-1) | · | X | X | X | X | X | X |
| L2 `let sum = 0` đặt trong thân vòng (RK-2) | · | · | X | X | X | X | X |
| L3 `i < min(len, MAX_ITEMS - 1)` (RK-3) | · | · | · | · | · | X | X |
| L4 `i < min(len, MAX_ITEMS)` — cắt bớt (RK-4) | · | · | · | · | · | · | X |

Đọc bảng theo cột: **n = 2** là giá trị nhỏ nhất bắt được L2; **n = 20** là giá trị duy nhất dưới max+1 bắt được L3; **chỉ n = 21** bắt được L4. Đọc theo hàng 0: không bản nào sai ở 0 lần lặp — 0 vẫn cần vì nó kiểm *khởi tạo* (ở đây cả bốn bản đều khởi tạo đúng). Đây là lý do bộ giá trị Beizer không rút gọn được thành «một giá trị điển hình».

### 4.4 Kết hợp với BVA: biên của biến cộng dồn

Loop testing chọn số lần lặp; BVA chọn giá trị **trong** mỗi lần lặp. Kết hợp hai thứ cho ra câu hỏi: *ở số lần lặp nào, với giá trị trần của mỗi dòng, biến cộng dồn vượt một giới hạn nào đó?*

- Giá trị trần mỗi dòng theo R-FOOD-02: 5.000.000 đ × 50 = **250.000.000 đ**.
- 20 dòng trần: tiền món **5.000.000.000 đ** — SPEC không đặt trần cho tổng.
- Lab có hai kho: bộ nhớ (số JavaScript, an toàn tới 2^53) và PGlite. Đọc `src/infra/pglite/schema.sql`: `trips.items_subtotal` và `trips.fare` là `integer` — tối đa **2.147.483.647**.

Biên của biến cộng dồn nằm giữa 8 dòng (2.000.000.000 đ) và 9 dòng (2.250.000.000 đ). Test cần đúng hai điểm đó trên kho PGlite, theo luồng báo giá rồi đặt chuyến. Kết quả chạy ở mục 5.

### 4.5 Vòng thử lại: số lần lặp do môi trường quyết định

Vòng trong `chargeWithRetry` khác vòng tiền món ở chỗ số lần lặp không do dữ liệu đầu vào mà do **phản hồi của cổng** quyết định, và có **lối thoát sớm** (thành công, bị từ chối). Bộ giá trị Beizer dịch sang:

| Lần lặp | Cách gây ra bằng ví giả (`failNext`) | Kỳ vọng |
|---|---|---|
| 0 (bỏ qua) | Không gây được theo SPEC; chỉ khi cấu hình `maxRetries = −1` | Ghi nhận hành vi: `failed`, `failure: null`, 0 lần gọi |
| 1 | — | `paid`, `attempts: 1` |
| 2 | `timeout` | `paid`, 2 lần gọi, **cùng một khoá** |
| 3 = max | `timeout`, `error` | `paid` ở lần 3 |
| max + 1 | 4 lần `timeout` xếp hàng | Dừng sau **đúng 3** lần gọi, `failed`/`timeout`, không trừ tiền |
| thoát giữa vòng | `error`, `decline` | Dừng ở lần 2, `declined` |

Max+1 ở đây là «môi trường đòi lần thứ 4, vòng phải từ chối». Ca 0 lần lặp lộ một cấu hình không được kiểm: `maxRetries` âm ⇒ vòng không chạy, trả `failure: null` — thất bại không lý do. SPEC không cho phép cấu hình đó nên đây là quan sát về độ bền, không phải defect theo đặc tả.

Một hệ quả khác đọc được ngay từ số lần lặp tối đa: thời gian xấu nhất của một lần thu tiền là 3 × 3 giây = **9 giây** (R-PAY-04). Endpoint `/complete` của tài xế chờ chừng ấy khi VíXanh treo — con số cần đưa vào thiết kế timeout của client.

### 4.6 Vòng lồng nhau: `canonicalJson`

`canonicalJson` gặp mảng thì duyệt từng phần tử (vòng ngoài); gặp object thì sắp xếp rồi duyệt từng khoá (vòng trong). Với thân `{ items: [ {name, unitPrice, quantity}, … ], paymentMethod }`, đặt vòng ngoài = số phần tử (0, 1, 2, 20) và vòng trong = số khoá mỗi phần tử (0, 1, 2, 3). Kiểm đủ tổ hợp là 4 × 4 = 16 ca. Chiến lược Beizer cho 7 ca (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 250" role="img" aria-label="Lưới 4 nhân 4: hàng là số phần tử của mảng (vòng ngoài) 0, 1, 2, 20; cột là số khoá mỗi phần tử (vòng trong) 0, 1, 2, 3. Bảy ô được chọn: cả hàng vòng ngoài bằng 1, và cả cột vòng trong bằng 3.">
<text x="460" y="22" text-anchor="middle" class="t-b">Vòng lồng nhau: chọn 7 trên 16 tổ hợp</text>
<text x="330" y="52" text-anchor="middle" class="t-small">vòng trong — số khoá mỗi phần tử</text>
<text x="220" y="76" text-anchor="middle" class="t-mono">0</text><text x="290" y="76" text-anchor="middle" class="t-mono">1</text><text x="360" y="76" text-anchor="middle" class="t-mono">2</text><text x="430" y="76" text-anchor="middle" class="t-mono">3</text>
<text x="110" y="160" text-anchor="middle" class="t-small">vòng ngoài</text>
<text x="110" y="176" text-anchor="middle" class="t-small">số phần tử</text>
<text x="168" y="108" text-anchor="end" class="t-mono">0</text><text x="168" y="148" text-anchor="end" class="t-mono">1</text><text x="168" y="188" text-anchor="end" class="t-mono">2</text><text x="168" y="228" text-anchor="end" class="t-mono">20</text>
<rect x="190" y="88" width="60" height="32" rx="4" class="f-surface s-muted"/><rect x="260" y="88" width="60" height="32" rx="4" class="f-surface s-muted"/><rect x="330" y="88" width="60" height="32" rx="4" class="f-surface s-muted"/><rect x="400" y="88" width="60" height="32" rx="4" class="f-phys-soft s-phys"/>
<rect x="190" y="128" width="60" height="32" rx="4" class="f-pk-soft s-pk"/><rect x="260" y="128" width="60" height="32" rx="4" class="f-pk-soft s-pk"/><rect x="330" y="128" width="60" height="32" rx="4" class="f-pk-soft s-pk"/><rect x="400" y="128" width="60" height="32" rx="4" class="f-pk-soft s-pk"/>
<rect x="190" y="168" width="60" height="32" rx="4" class="f-surface s-muted"/><rect x="260" y="168" width="60" height="32" rx="4" class="f-surface s-muted"/><rect x="330" y="168" width="60" height="32" rx="4" class="f-surface s-muted"/><rect x="400" y="168" width="60" height="32" rx="4" class="f-phys-soft s-phys"/>
<rect x="190" y="208" width="60" height="32" rx="4" class="f-surface s-muted"/><rect x="260" y="208" width="60" height="32" rx="4" class="f-surface s-muted"/><rect x="330" y="208" width="60" height="32" rx="4" class="f-surface s-muted"/><rect x="400" y="208" width="60" height="32" rx="4" class="f-phys-soft s-phys"/>
<rect x="510" y="96" width="22" height="16" rx="3" class="f-pk-soft s-pk"/><text x="542" y="109" class="t-small">Bước 1–2: ngoài = 1 (tối thiểu), trong chạy 0, 1, 2, 3</text>
<rect x="510" y="136" width="22" height="16" rx="3" class="f-phys-soft s-phys"/><text x="542" y="149" class="t-small">Bước 3: trong = 3 (điển hình), ngoài chạy 0, 2, 20</text>
<rect x="510" y="176" width="22" height="16" rx="3" class="f-surface s-muted"/><text x="542" y="189" class="t-small">Không chọn: 9 tổ hợp còn lại</text>
<text x="542" y="222" class="t-small">Oracle: đảo thứ tự khoá ⇒ vân tay không đổi (R-IDEM-02)</text>
</svg>
</div>
<figcaption><b>Hình 2 · Chiến lược vòng lồng nhau.</b> Vòng trong được kiểm đủ khi vòng ngoài ở mức tối thiểu; sau đó vòng ngoài được kiểm đủ khi vòng trong ở mức điển hình. 4 + 4 − 1 = 7 ca thay vì 16; với ba tầng lồng, khoản tiết kiệm lớn hơn nhiều.</figcaption>
</figure>

Oracle mỗi ô: đảo thứ tự khoá ⇒ vân tay **không đổi**. Thêm hai ca âm: đổi giá trị ở phần tử **cuối** (20 phần tử) ⇒ vân tay phải đổi (bắt vòng ngoài dừng sớm); đảo thứ tự **phần tử** ⇒ vân tay phải đổi (R-IDEM-02 chỉ bỏ qua thứ tự khoá).

## 5. Thực hành

### 5.1 Bộ test

Thư mục `examples/toi-noi-testing/articles/loop-testing/`:

- `vong-cong-tien-mon.test.ts` — **LT-A** `itemsSubtotal`/`priceFood` ở 7 số lần lặp (9 test); **LT-B** ma trận phát hiện L1…L4 + bản thật (1); **LT-C** `POST /api/quotes` với 0…21 dòng (7); **LT-D** 20, 8, 9 dòng giá trần, báo giá rồi đặt chuyến trên kho bộ nhớ và PGlite (3).
- `vong-thu-lai-va-long-nhau.test.ts` — **LR** vòng thử lại: 0, 1, 2, 3, max+1, thoát sớm (6); **LN** vòng lồng nhau: 7 ô + 2 ca âm (9).
- `loop-variants.ts` (L1…L4), `fixtures.ts` (dữ liệu + oracle dạng đóng).

Ca biên của biến cộng dồn (`DEFECT · PostgreSQL: 9 dòng giá trần`) khẳng định hành vi đúng theo SPEC — đặt chuyến 201 — và được đánh dấu `it.fails` vì defect còn đó; `SHOW_RED=1` chạy nó như test thường để thấy failure thật.

### 5.2 Chạy thật

```bash
cd examples/toi-noi-testing
npx vitest run articles/loop-testing
```

```text
stdout | … > LT-B · ma trận phát hiện … > bảng phát hiện khớp phân tích ở mục 4 của bài
n:                       0 1 2 5 19 20 21
L1 bỏ dòng đầu           · X X X X X X
L2 khởi tạo trong vòng   · · X X X X X
L3 chặn trên lệch một    · · · · · X X
L4 lặng lẽ cắt bớt       · · · · · · X
src (bản thật)           · · · · · · ·

 ✓ articles/loop-testing/vong-cong-tien-mon.test.ts (20 tests)
 Test Files  2 passed (2)
      Tests  35 passed (35)
```

Ma trận chạy thật khớp từng ô với dự đoán ở mục 4.3. Bản thật `itemsSubtotal` đúng ở cả bảy số lần lặp. Bật failure thật của ca biên:

```bash
SHOW_RED=1 npx vitest run articles/loop-testing
```

```text
× LT-D · … > DEFECT · PostgreSQL: 9 dòng giá trần ⇒ báo giá 200 nhưng đặt chuyến phải 201 (thực tế 500)
  → expected { status: 500, …(1) } to match object { status: 201 }
 Tests  1 failed | 34 passed (35)
```

Chạy lại cùng luồng với `createApp(deps)` để bật log lỗi 500, PGlite báo: `value "2250000000" is out of range for type integer` — chính là `items_subtotal` của 9 dòng.

### 5.3 Đọc kết quả

| Ca | Kết quả mong đợi | Thực tế |
|---|---|---|
| LT-A, LT-C (n = 0…21) | Tiền món = oracle; domain: 0 dòng ⇒ `SUBTOTAL_TOO_LOW`, 21 dòng ⇒ 840.000 đ; API: 0 và 21 dòng ⇒ 400 `VALIDATION_ERROR` (`details` chỉ ra `items`) | Đạt |
| LT-D, 20 dòng trần (bộ nhớ) / 8 dòng trần (PGlite) | 201 | Đạt |
| LT-D, 9 dòng trần (PGlite) | Báo giá 2.250.021.000 đ, đặt chuyến 201 | **500 `INTERNAL_ERROR`** |
| LR, LN | Như mục 4.5, 4.6 | Đạt; max+1 dừng đúng 3 lần gọi, cùng một khoá |

Defect report rút gọn (mẫu đầy đủ ở [TEST-010](/articles/testware-san-pham-kiem-thu)):

| Trường | Nội dung |
|---|---|
| Tiêu đề | Đơn đồ ăn từ 9 dòng ở giá trần: báo giá thành công nhưng đặt chuyến lỗi 500 (kho PostgreSQL) |
| Tái hiện | Kho PGlite; `POST /api/quotes` đồ ăn, SG, 2 km, 9 dòng × 5.000.000 đ × 50 ⇒ 200, `total` 2.250.021.000; `POST /api/trips` với `quoteId` đó ⇒ 500 |
| Mong đợi | 201 (R-FOOD-02 cho phép tiền món tới 5 tỉ đồng; SPEC không đặt trần tổng) — **hoặc** SPEC đặt trần tổng và báo giá bị từ chối với mã lỗi rõ ràng |
| Thực tế | 500 `INTERNAL_ERROR`; log `value "2250000000" is out of range for type integer` |
| Nơi nghi ngờ | `schema.sql`: `trips.items_subtotal`, `trips.fare` kiểu `integer`; báo giá lưu `jsonb` nên không lỗi — lệch kiểu giữa hai bảng |
| Ghi chú | Kho bộ nhớ không tái hiện ⇒ bộ test chỉ dùng kho bộ nhớ không bao giờ thấy lỗi này |

Defect này không nằm trong vòng lặp mà nằm ở **nơi kết quả của vòng được cất**. Loop testing tìm ra nó vì đã hỏi «max lần lặp × max giá trị mỗi lần thì sao».

## 6. Góc nhìn nâng cao

**Vòng ẩn trong hàm bậc cao.** `itemsSubtotal` dùng `reduce`, không có `for`: báo cáo branch coverage không có quyết định «còn phần tử?», ca 0 lần lặp chỉ lộ qua function coverage (callback không được gọi). Không thấy nhánh không có nghĩa là không có vòng — đọc code.

**Vòng không có giới hạn thiết kế.** `expireStaleSearches` (R-STATE-05) duyệt mọi chuyến `searching` quá 5 phút, không có n tối đa trong code. Khi đó «max» lấy từ **operational profile** (giả định: sự cố điều phối lúc cao điểm 70 chuyến/giây ⇒ hàng nghìn chuyến mỗi lượt quét) và trở thành câu hỏi hiệu năng — mỗi lần lặp một câu `UPDATE` — thuộc Series 09.

**Giới hạn không nằm cạnh vòng.** Giới hạn 20 dòng nằm ở lớp API. Một kênh gọi thẳng domain sau này (vd job import đơn đối tác) sẽ cho max+1 đi qua; test max+1 ở tầng domain ghi lại hành vi đó.

**Max của vòng × max của mỗi lần lặp.** Như mục 4.4: biến cộng dồn, kích thước bộ đệm, thời gian tổng (3 lần × 3 giây), kích thước thân (`canonicalJson` với 20 phần tử phải nằm trong 10 KB của R-API-01). Mỗi đại lượng tích luỹ theo vòng là một biến mới cần BVA.

**So với kỹ thuật khác.** Branch coverage ([TEST-044](/articles/branch-coverage-decision-coverage)) bảo đảm mọi nhánh được đi qua; path testing ([TEST-048](/articles/path-testing)) phủ tổ hợp nhánh trong thân vòng; BVA ([TEST-032](/articles/boundary-value-analysis)) lo biên đơn giá, số lượng của từng dòng; property-based testing (Series 06) sinh ngẫu nhiên số dòng × giá trị. Loop testing là phần duy nhất trong số đó nhắm thẳng vào **số lần lặp**.

**Sai lầm thường gặp.**

- Chỉ kiểm «vài phần tử» — bỏ 0, 2 và quanh max.
- Oracle cũng viết bằng vòng lặp giống code (cùng lỗi lệch một).
- Dữ liệu mỗi lần lặp giống nhau, làm các lỗi khác nhau cho cùng kết quả.
- Coi max+1 «không thể xảy ra vì UI đã chặn» — client khác, job khác, API đối tác không qua UI.
- Kiểm vòng thử lại với timeout thật 3 giây — chậm, chập chờn; lab tiêm `timeoutMs: 20`.

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — vòng huỷ chuyến hết hạn.** Thiết kế các ca loop testing cho `expireStaleSearches` (`src/app/trips.ts`, R-STATE-05). Vòng có một nhánh bên trong: `updateIfStatus` trả `false` khi chuyến đã đổi trạng thái giữa chừng (ví dụ tài xế vừa nhận).

<details>
<summary><b>Lời giải</b></summary>
<p>Số lần lặp = số chuyến <code>searching</code> quá hạn; không có max thiết kế ⇒ lấy từ operational profile. Ca: (1) 0 chuyến (có một chuyến mới 4 phút 59 giây — biên R-STATE-05) ⇒ <code>expired = 0</code>; (2) 1 chuyến đúng 5 phút ⇒ 1, chuyến <code>cancelled</code> lý do <code>no_driver</code>, phí 0, <code>paymentStatus = none</code>; (3) 2 chuyến ⇒ 2, mỗi chuyến một dòng <code>trip_events</code>; (4) điển hình 10; (5) lớn, vd 1.000 (giả định sự cố) ⇒ đúng 1.000 và đo thời gian; (6) 3 chuyến, chuyến thứ 2 bị tài xế nhận ngay trước <code>updateIfStatus</code> (repository bọc trì hoãn) ⇒ <code>expired = 2</code>, chuyến 2 vẫn <code>accepted</code> — nhánh bên trong thân vòng.</p>
</details>

**Bài tập 2 — vòng của giới hạn tần suất.** `createRateLimiter` (`src/api/rate-limit.ts`, R-API-13) lọc mảng mốc thời gian trong cửa sổ 60 giây, tối đa 5 yêu cầu. Chọn số lần lặp và kết quả mong đợi.

<details>
<summary><b>Lời giải</b></summary>
<p>Vòng <code>filter</code> chạy trên số mốc đã lưu: 0 mốc (yêu cầu đầu) ⇒ qua; 1; 2; 4 (max − 1) ⇒ yêu cầu thứ 5 qua; 5 (max) ⇒ yêu cầu thứ 6 bị 429, <code>Retry-After</code> = 60 nếu 5 yêu cầu cùng thời điểm; max + 1 — yêu cầu thứ 7 ngay sau ⇒ vẫn 429 và <b>không</b> được đếm (mảng vẫn 5 mốc, đúng R-API-13). Thêm biến thời gian: dời đồng hồ 60 giây ⇒ mốc cũ nhất rơi khỏi cửa sổ (<code>t &gt; now − windowMs</code>), yêu cầu kế tiếp qua.</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K2).** Một hàm có vòng `for` duyệt danh sách dòng món. Bộ test hiện có một ca với 3 dòng món và đạt 100% branch coverage. Phát biểu nào đúng?

- a) Vòng đã được kiểm đầy đủ vì cả hai outcome của điều kiện vòng đều được chạy
- b) Branch coverage 100% không đòi hỏi chạy vòng 0 lần hay ở số lần lặp tối đa
- c) Cần thêm test để đạt statement coverage
- d) Không thể đạt branch coverage cho vòng lặp

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — một ca có ≥ 1 lần lặp đã đi qua cả nhánh vào vòng và nhánh thoát vòng (CTFL mục 4.3.2). a) sai: «đủ nhánh» không có nghĩa là đủ số lần lặp đặc biệt. c) sai: branch coverage bao hàm statement coverage. d) sai: điều kiện vòng có hai outcome như mọi quyết định.</p>
</details>

**Câu 2 (K3).** Vòng cộng tiền món cho phép tối đa 20 dòng. Theo chiến lược vòng đơn của Beizer, bộ số lần lặp nào phù hợp nhất?

- a) 1, 10, 20
- b) 0, 1, 2, 10, 19, 20, 21
- c) 1, 20
- d) 0, 20, 40

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — bỏ qua, 1, 2, điển hình, max − 1, max, max + 1. a) thiếu 0, 2, 19, 21. c) chỉ hai điểm, bỏ lỡ lỗi lộ ở 2 lần lặp và lỗi lệch một ở max. d) 40 không có lý do; thiếu 1, 2, quanh max.</p>
</details>

**Câu 3 (K2).** Bản cài đặt đặt `let sum = 0` bên trong thân vòng nên chỉ trả tiền của dòng cuối. Số lần lặp **nhỏ nhất** nào làm test với oracle đúng phát hiện được lỗi này?

- a) 0
- b) 1
- c) 2
- d) 20

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — ở 0 và 1 lần lặp, «tiền dòng cuối» trùng với tổng đúng; lần lặp thứ hai là lần đầu tiên kết quả cũ bị ghi đè. Đây chính là lý do Beizer giữ giá trị 2. d) cũng phát hiện được nhưng không phải nhỏ nhất.</p>
</details>

**Câu 4 (K2).** Vòng thử lại thanh toán cho tối đa 3 lần gọi cổng. Ca test nào thể hiện đúng tinh thần «max + 1» của loop testing?

- a) Gọi hàm thu tiền 4 lần liên tiếp từ test
- b) Cho cổng giả hết giờ 4 lần liên tiếp và kiểm vòng dừng sau đúng 3 lần gọi, không lần thứ 4
- c) Đặt `maxRetries = 3` và kiểm có 4 lần gọi
- d) Cho cổng giả thành công ngay lần đầu

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — môi trường «đòi» lần lặp thứ 4, vòng phải từ chối. a) là 4 vòng riêng biệt. c) đổi cấu hình là kiểm max = 4, không phải max + 1 của R-PAY-04. d) là ca 1 lần lặp.</p>
</details>

**Câu 5 (K2).** Test đơn đồ ăn 20 dòng ở giá trần chạy trên kho bộ nhớ thì đạt, trên kho PostgreSQL thì lỗi 500 do cột `integer` tràn. Phát biểu nào mô tả đúng nhất?

- a) Loop testing chỉ kiểm số lần lặp nên đây không phải phát hiện của loop testing
- b) Max lần lặp × giá trị tối đa mỗi lần tạo ra một đại lượng tích luỹ có biên riêng; kết hợp loop testing với BVA trên đại lượng đó đã lộ defect
- c) Đây là lỗi môi trường, không cần ghi nhận
- d) Nên bỏ ca này vì khách thật không đặt đơn lớn như vậy

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — ý «kết hợp loop testing với BVA» của mục 4.4. a) quá hẹp: chọn max lần lặp dẫn tới ca này. c) sai: lệch giữa đặc tả (cho phép 5 tỉ) và lược đồ là defect của sản phẩm. d) sai: chấp nhận rủi ro là quyết định của chủ sản phẩm sau khi defect được ghi nhận (có thể sửa SPEC đặt trần tổng thay vì đổi kiểu cột).</p>
</details>

## 8. Tài liệu tham khảo

- Boris Beizer, *Software Testing Techniques*, 2nd ed., Van Nostrand Reinhold, 1990 — chương về control flow testing và loop testing (nguồn gốc của kỹ thuật, ngoài ISTQB).
- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024), mục 4.3.2 «Branch Testing and Branch Coverage»: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level — Technical Test Analyst (CTAL-TTA)* **v4.0**, mục 2.3 «Decision Testing», 3.2.1 «Control Flow Analysis», 5.2.2 «Code Reviews» (checklist «Loops and Branches»): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB, *Certified Tester Advanced Level — Test Analyst (CTAL-TA)* **v4.0**, mục 3.2.3 «Scenario-Based Testing» (simple loop coverage; TA-3.2.3, K3): https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/ — mục *branch coverage*, *decision testing*, *test oracle*.
- PostgreSQL Documentation — *Numeric Types* (giới hạn của `integer`): https://www.postgresql.org/docs/current/datatype-numeric.html
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-FOOD-01…04, R-PAY-04, R-IDEM-02, R-STATE-05, R-API-13; mã bài tại `articles/loop-testing/`.
- Tài liệu kiến trúc tổng thể Tới Nơi: §4.3 Pricing &amp; Quote, §4.4 Booking, §4.9 Payment, §7.2 quy ước deadline, §14 bounded retry.
- Bài liên quan: [TEST-032 · Boundary Value Analysis](/articles/boundary-value-analysis) · [TEST-044 · Branch Coverage](/articles/branch-coverage-decision-coverage) · [TEST-048 · Path Testing](/articles/path-testing) · [TEST-028 · Checklist-based Review](/articles/checklist-based-review) · [TEST-010 · Testware](/articles/testware-san-pham-kiem-thu) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi).

Bài tiếp theo: [TEST-050 · API Testing từ góc nhìn code — phủ từng nhánh của handler đặt chuyến](/articles/api-testing-tu-goc-nhin-code).
