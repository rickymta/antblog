---
slug: boundary-value-analysis
title: "Boundary Value Analysis: biên 2-value, 3-value và biên sau khi làm tròn"
summary: "BVA chọn giá trị ở biên phân vùng — nơi developer hay sai nhất. Bài so BVA 2-value với 3-value theo CTFL v4.0.1 trên quãng đường, khối lượng, giờ cao điểm của Tới Nơi, và vì sao phải tìm biên cả trước lẫn sau làm tròn."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-032, boundary-value-analysis, black-box, test-design, toi-noi]
language: vi
---

Ở [TEST-031](/articles/equivalence-partitioning), một bản cài đặt «kiểm khoảng rồi mới làm tròn quãng đường» qua mặt cả bộ ca phân vùng tương đương: nó chỉ sai trong dải 0,45–0,49 km và 100,01–100,04 km. Boundary Value Analysis (BVA — phân tích giá trị biên) sinh ra cho đúng loại lỗi đó. Bài trả lời ba câu: **biên nằm ở đâu, chọn bao nhiêu giá trị quanh mỗi biên, và khi đầu vào bị làm tròn thì biên nằm trên giá trị nào?**

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-032</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S04 · Black-box Test Design Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1 (đọc thêm Advanced — CTAL-TA v4.0)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Component; System</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Boundary Value Analysis</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/equivalence-partitioning">TEST-031 · Equivalence Partitioning</a></td></tr>
<tr><th scope="row">Thực hành</th><td>BVA cho quãng đường (0,44/0,45; 100,04/100,05 — R-FARE-02), khối lượng (0,04/0,05; 30,04/30,05 — R-PARCEL-01), ngưỡng xe máy 20,0/20,1 kg (R-PARCEL-02), giờ 10:59:59/11:00:00/12:59:59/13:00:00 (R-FARE-04); so số ca 2-value với 3-value — lab <code>articles/boundary-value-analysis</code> (76 ca vitest)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 4 · mục 4.2.2 · FL-4.2.2 (K3). Đọc thêm: CTAL-TA v4.0 · Chương 3 · mục 3.1.1 · TA-3.1.1 (K3) — domain testing tổng quát hoá BVA, đi sâu ở TEST-040</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

EP chọn **một đại diện ở giữa** mỗi phân vùng. Cách đó bắt được lỗi xử lý sai *cả phân vùng* (quên cận trên — 150 km được chấp nhận) nhưng mù với lỗi chỉ xảy ra *sát đường ranh*: `<` viết thành `<=`, so sánh trước khi làm tròn, hằng số lệch một đơn vị. Đó lại là những lỗi developer hay mắc nhất — syllabus nói thẳng lý do BVA tập trung vào biên.

BVA giải quyết câu hỏi: **giá trị nào quanh đường ranh giữa hai phân vùng cần thử để chắc ranh nằm đúng chỗ?** Khi nào dùng: mọi phân vùng **có thứ tự** — số, ngày giờ, độ dài chuỗi, số lượng, tiền. Ở Tới Nơi gần như mọi quy tắc giá đều có ranh: quãng đường, khối lượng, khung giờ cao điểm, mốc 2 phút miễn phí huỷ, tiền món tối thiểu 30.000 đ.

## 2. Lý thuyết theo ISTQB

### 2.1. Định nghĩa

Theo CTFL v4.0.1 mục 4.2.2, BVA là kỹ thuật dựa trên việc **thử các biên của phân vùng tương đương**. Hệ quả trực tiếp:

- BVA **chỉ dùng được cho phân vùng có thứ tự**. Mã thành phố `SG`, `HN`… không có «sát biên» — BVA không áp được.
- **Giá trị biên** (boundary value) là giá trị **nhỏ nhất và lớn nhất** của một phân vùng.
- Phân vùng phải «liền»: hai phần tử cùng phân vùng thì mọi phần tử nằm giữa chúng cũng thuộc phân vùng đó.

Lỗi điển hình BVA tìm thấy: biên được cài đặt **lệch lên hoặc lệch xuống** so với vị trí dự định, hoặc **bị bỏ quên** hoàn toàn.

### 2.2. Hai phiên bản: 2-value và 3-value

| | 2-value BVA | 3-value BVA |
|---|---|---|
| Coverage item cho mỗi giá trị biên | giá trị biên + **hàng xóm gần nhất thuộc phân vùng liền kề** | giá trị biên + **cả hai hàng xóm** |
| Nguồn syllabus dẫn | Craig 2002, Myers 2011 | Koomen 2006, O'Regan 2019 |
| Coverage | số giá trị biên đã thử / tổng số giá trị biên × 100% | số giá trị biên và hàng xóm đã thử / tổng số đó × 100% |
| Đặc điểm | ít ca hơn | chặt hơn; một số coverage item **không phải** giá trị biên |

Với 2-value, «hàng xóm ở phân vùng kề» của một biên chính là giá trị biên của phân vùng bên kia — nên tập coverage item thực chất là tập các giá trị biên.

Syllabus minh hoạ vì sao 3-value chặt hơn: quyết định `x ≤ 10` bị cài đặt nhầm thành `x = 10`. Hai giá trị của 2-value (10 và 11) cho kết quả giống hệt bản đúng; chỉ hàng xóm phía trong — 9 — của 3-value mới lộ lỗi. Mục 5.4 có bản Tới Nơi của đúng lỗi này.

### 2.3. «Hàng xóm gần nhất» phụ thuộc độ chính xác

CTFL không nói về độ chính xác, nhưng không có nó thì không xác định được hàng xóm: hàng xóm của 0,45 km là 0,44 (độ chính xác 0,01) hay 0,449 (0,001)? CTAL-TA v4.0 mục 3.1.1, khi nói về domain testing, chọn điểm sát biên «theo độ chính xác đã cho» (*according to the given precision*). Trong bài: quãng đường và khối lượng **thô** dùng 0,01; giá trị **sau làm tròn** dùng 0,1; thời gian dùng 1 giây. Độ chính xác là **giả định của người test**, phải ghi rõ trong test design.

### 2.4. Bảng thuật ngữ Anh – Việt

| Thuật ngữ (ISTQB) | Tiếng Việt dùng trong bài | Ghi chú |
|---|---|---|
| boundary value analysis (BVA) | phân tích giá trị biên | kỹ thuật black-box |
| boundary value | giá trị biên | min hoặc max của một phân vùng |
| 2-value / 3-value BVA | BVA hai giá trị / ba giá trị | khác số coverage item mỗi biên |
| neighbor | hàng xóm | giá trị liền kề theo độ chính xác |
| ordered partition | phân vùng có thứ tự | điều kiện để dùng BVA |
| domain testing | kiểm thử miền | CTAL-TA; điểm ON/OFF/IN/OUT |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Nhiều giáo trình dùng bộ 5–7 điểm «min−, min, min+, nom, max−, max, max+» (robust boundary value testing, Jorgensen). Bỏ «nom», bộ đó gần trùng 3-value BVA cho một phân vùng hợp lệ. CTFL v4.0.1 chỉ định nghĩa 2-value và 3-value; bài dùng đúng hai tên này.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ và business rules

Hai khung cao điểm chiếm 50% số chuyến trong ngày ([hồ sơ dự án](/articles/du-an-mau-toi-noi)); lệch biên cao điểm một giây là lệch giá của hàng chục chuyến tạo ra trong giây đó (≈ 70 chuyến/giây). Các quy tắc có biên trong bài:

- **R-FARE-02** — quãng đường làm tròn **0,1 km half-up trước** mọi bước (0,45 ⇒ 0,5; 0,44 ⇒ 0,4), rồi phải trong **[0,5; 100]**, tính cả hai đầu; ví dụ của SPEC: 100,04 hợp lệ, 100,05 không.
- **R-PARCEL-01** — khối lượng làm tròn 0,1 kg half-up, rồi phải thoả **0 &lt; w ≤ 30**: 0,04 ⇒ 0,0 (sai); 0,05 ⇒ 0,1 (đúng); 30,04 đúng; 30,05 sai.
- **R-PARCEL-02** — w &gt; 20 kg bắt buộc ô tô; **20,0 kg vẫn đi xe máy được**.
- **R-FARE-04** — cao điểm khi giờ Việt Nam thuộc **[11:00; 13:00)** hoặc **[17:00; 19:00)**: 11:00:00 và 12:59:59 là cao điểm; 13:00:00 thì không.
- **R-GEN-02** — giờ nghiệp vụ là giờ Việt Nam UTC+7, bất kể múi giờ máy chủ.

### 3.2. System context

Trong kiến trúc đích, quãng đường do **Pricing & Quote Service** (kiến trúc §4.3) lấy từ bản đồ MapViet — một số thực có nhiều chữ số lẻ, không phải số «đẹp» người test hay gõ tay. Giờ cao điểm đọc từ đồng hồ hệ thống; kiến trúc §17.2 yêu cầu *clock abstraction* để test được hết hạn và mốc thời gian. Lab có đúng các móc đó: `FixedClock` + `vnTime(…)` tạo thời điểm giờ VN chính xác tới giây; quãng đường do client gửi thẳng (R-GEN-06).

Biên ở Tới Nơi vì thế nằm trên **hai lớp giá trị**: giá trị **thô** (client hoặc MapViet gửi) và giá trị **sau làm tròn** (thứ quy tắc thật sự so sánh).

### 3.3. Rủi ro có thể xảy ra

| Rủi ro | Hậu quả | Biên lộ lỗi |
|---|---|---|
| So sánh trước khi làm tròn | Khách 0,45 km bị từ chối dù SPEC cho phép | 0,45 (thô) |
| Làm tròn bằng `toFixed(1)` | 100,05 km thành 100,0 và được nhận — chuyến vượt giới hạn | 100,05 (thô) |
| `kg === 0` thay vì `kg <= 0` | Khối lượng âm lọt qua, phụ phí sai | −0,1 (sau làm tròn) |
| `hour <= 13` thay vì `< 13` | Cả giờ 13:00–13:59 tính giá cao điểm, khách bị thu thừa 25% | 13:00:00 |
| Dùng giờ máy chủ (UTC) | Cao điểm lệch 7 tiếng: 04:00 VN tính giá cao điểm | mọi biên giờ |
| `w >= 20` thay vì `w > 20` | Bưu kiện đúng 20,0 kg bị bắt đi ô tô | 20,0 |

## 4. Phân tích kỹ thuật

### 4.1. Bước 1 — từ phân vùng có thứ tự tới giá trị biên

Lấy phân vùng của [TEST-031](/articles/equivalence-partitioning) (D1–D3 cho quãng đường), viết lại ở **hai** lớp giá trị:

| Miền | Độ chính xác | D1 (không hợp lệ) | D2 (hợp lệ) | D3 (không hợp lệ) | Giá trị biên |
|---|---|---|---|---|---|
| Quãng đường thô | 0,01 km | ≤ 0,44 | 0,45 … 100,04 | ≥ 100,05 | 0,44 · 0,45 · 100,04 · 100,05 |
| Quãng đường sau làm tròn | 0,1 km | ≤ 0,4 | 0,5 … 100,0 | ≥ 100,1 | 0,4 · 0,5 · 100,0 · 100,1 |

D1 không có biên dưới, D3 không có biên trên (không chặn) — nên mỗi miền có 4 giá trị biên chứ không phải 6.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 270" role="img" aria-label="Hai trục số: trục trên là quãng đường thô độ chính xác 0,01 km, trục dưới là quãng đường sau làm tròn 0,1 km. Mũi tên cho thấy 0,43 và 0,44 cùng thành 0,4; 0,45 và 0,46 thành 0,5; 100,03 và 100,04 thành 100,0; 100,05 và 100,06 thành 100,1. Chấm đặc là coverage item của 2-value, chấm rỗng là phần 3-value thêm vào.">
<defs>
<marker id="t032-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-b">Biên quãng đường ở hai lớp giá trị (R-FARE-02)</text>
<text x="16" y="86" class="t-small t-b">thô</text>
<text x="16" y="190" class="t-small t-b">làm tròn</text>
<line x1="80" y1="82" x2="380" y2="82" class="s-line" stroke-width="2"/>
<line x1="540" y1="82" x2="840" y2="82" class="s-line" stroke-width="2"/>
<line x1="80" y1="186" x2="380" y2="186" class="s-line" stroke-width="2"/>
<line x1="540" y1="186" x2="840" y2="186" class="s-line" stroke-width="2"/>
<text x="460" y="86" text-anchor="middle" class="t-small">…</text>
<text x="460" y="190" text-anchor="middle" class="t-small">…</text>
<line x1="230" y1="48" x2="230" y2="220" class="s-dead dash" stroke-width="1.5"/>
<line x1="690" y1="48" x2="690" y2="220" class="s-dead dash" stroke-width="1.5"/>
<text x="150" y="44" text-anchor="middle" class="t-10 t-warn">D1</text>
<text x="310" y="44" text-anchor="middle" class="t-10 t-phys">D2</text>
<text x="620" y="44" text-anchor="middle" class="t-10 t-phys">D2</text>
<text x="770" y="44" text-anchor="middle" class="t-10 t-warn">D3</text>
<text x="110" y="66" text-anchor="middle" class="t-small t-mono">0,43</text>
<text x="190" y="66" text-anchor="middle" class="t-small t-mono">0,44</text>
<text x="270" y="66" text-anchor="middle" class="t-small t-mono">0,45</text>
<text x="350" y="66" text-anchor="middle" class="t-small t-mono">0,46</text>
<text x="570" y="66" text-anchor="middle" class="t-small t-mono">100,03</text>
<text x="650" y="66" text-anchor="middle" class="t-small t-mono">100,04</text>
<text x="730" y="66" text-anchor="middle" class="t-small t-mono">100,05</text>
<text x="810" y="66" text-anchor="middle" class="t-small t-mono">100,06</text>
<circle cx="110" cy="82" r="6" class="f-bg s-warn" stroke-width="2"/>
<circle cx="190" cy="82" r="6" class="f-warn s-warn"/>
<circle cx="270" cy="82" r="6" class="f-phys s-phys"/>
<circle cx="350" cy="82" r="6" class="f-bg s-phys" stroke-width="2"/>
<circle cx="570" cy="82" r="6" class="f-bg s-phys" stroke-width="2"/>
<circle cx="650" cy="82" r="6" class="f-phys s-phys"/>
<circle cx="730" cy="82" r="6" class="f-warn s-warn"/>
<circle cx="810" cy="82" r="6" class="f-bg s-warn" stroke-width="2"/>
<line x1="110" y1="90" x2="186" y2="176" class="s-muted" stroke-width="1.2" marker-end="url(#t032-a)"/>
<line x1="190" y1="90" x2="190" y2="176" class="s-muted" stroke-width="1.2" marker-end="url(#t032-a)"/>
<line x1="270" y1="90" x2="270" y2="176" class="s-muted" stroke-width="1.2" marker-end="url(#t032-a)"/>
<line x1="350" y1="90" x2="274" y2="176" class="s-muted" stroke-width="1.2" marker-end="url(#t032-a)"/>
<line x1="570" y1="90" x2="646" y2="176" class="s-muted" stroke-width="1.2" marker-end="url(#t032-a)"/>
<line x1="650" y1="90" x2="650" y2="176" class="s-muted" stroke-width="1.2" marker-end="url(#t032-a)"/>
<line x1="730" y1="90" x2="730" y2="176" class="s-muted" stroke-width="1.2" marker-end="url(#t032-a)"/>
<line x1="810" y1="90" x2="734" y2="176" class="s-muted" stroke-width="1.2" marker-end="url(#t032-a)"/>
<circle cx="110" cy="186" r="6" class="f-bg s-warn" stroke-width="2"/>
<circle cx="190" cy="186" r="6" class="f-warn s-warn"/>
<circle cx="270" cy="186" r="6" class="f-phys s-phys"/>
<circle cx="350" cy="186" r="6" class="f-bg s-phys" stroke-width="2"/>
<circle cx="570" cy="186" r="6" class="f-bg s-phys" stroke-width="2"/>
<circle cx="650" cy="186" r="6" class="f-phys s-phys"/>
<circle cx="730" cy="186" r="6" class="f-warn s-warn"/>
<circle cx="810" cy="186" r="6" class="f-bg s-warn" stroke-width="2"/>
<text x="110" y="210" text-anchor="middle" class="t-small t-mono">0,3</text>
<text x="190" y="210" text-anchor="middle" class="t-small t-mono">0,4</text>
<text x="270" y="210" text-anchor="middle" class="t-small t-mono">0,5</text>
<text x="350" y="210" text-anchor="middle" class="t-small t-mono">0,6</text>
<text x="570" y="210" text-anchor="middle" class="t-small t-mono">99,9</text>
<text x="650" y="210" text-anchor="middle" class="t-small t-mono">100,0</text>
<text x="730" y="210" text-anchor="middle" class="t-small t-mono">100,1</text>
<text x="810" y="210" text-anchor="middle" class="t-small t-mono">100,2</text>
<text x="460" y="252" text-anchor="middle" class="t-small">● 2-value · ○ phần 3-value thêm · xanh = D2 hợp lệ · cam = không hợp lệ · vạch đứt = biên</text>
</svg>
</div>
<figcaption><b>Hình 1 · Một biên, hai lớp giá trị.</b> Hàng trên là cái client gửi; hàng dưới là cái quy tắc so sánh. Bốn giá trị thô của 3-value chỉ rơi vào hai giá trị sau làm tròn — và ngược lại, −0,1 hay 99,9 «sau làm tròn» chỉ thử được bằng cách gửi đúng số đó.</figcaption>
</figure>

### 4.2. Bước 2 — coverage item 2-value và 3-value

Lab sinh coverage item bằng hàm, không gõ tay: mỗi miền là danh sách phân vùng liền nhau theo **số nguyên bước** độ chính xác (0,45 km = 45 bước 0,01), tránh sai số dấu phẩy động khi tính «±1 bước». Kết quả (ca `bva-items.test.ts`):

| Miền | 2-value | 3-value | Số item 2v → 3v |
|---|---|---|---|
| Quãng đường thô (0,01) | 0,44 · 0,45 · 100,04 · 100,05 | + 0,43 · 0,46 · 100,03 · 100,06 | 4 → 8 |
| Quãng đường sau làm tròn (0,1) | 0,4 · 0,5 · 100,0 · 100,1 | + 0,3 · 0,6 · 99,9 · 100,2 | 4 → 8 |
| Khối lượng thô (0,01) | 0,04 · 0,05 · 30,04 · 30,05 | + 0,03 · 0,06 · 30,03 · 30,06 | 4 → 8 |
| Khối lượng sau làm tròn (0,1) | 0,0 · 0,1 · 30,0 · 30,1 | + **−0,1** · 0,2 · 29,9 · 30,2 | 4 → 8 |
| Ngưỡng xe máy (0,1) | 20,0 · 20,1 | + 19,9 · 20,2 | 2 → 4 |
| Giờ đặt (1 giây) | 10:59:59 · 11:00:00 · 12:59:59 · 13:00:00 · 16:59:59 · 17:00:00 · 18:59:59 · 19:00:00 | + 10:59:58 · 11:00:01 · 12:59:58 · 13:00:01 · 16:59:58 · 17:00:01 · 18:59:58 · 19:00:01 | 8 → 16 |
| **Tổng** | | | **26 → 52** |

3-value đúng gấp đôi ở mọi miền vì các biên đứng thành cặp sát nhau. Đổi lại, nó thêm giá trị mà 2-value không bao giờ sinh: **−0,1 kg** là hàng xóm phía trong phân vùng không hợp lệ của biên 0,0.

### 4.3. Biên thời gian: khung giờ, nửa đêm và múi giờ

Hai khung cao điểm chia ngày thành 5 phân vùng có thứ tự (sáng · trưa cao điểm · chiều · tối cao điểm · tối). Ba điểm dễ sai:

- **Nửa đêm không phải biên của quy tắc này.** Đêm 19:00 → 10:59:59 hôm sau là **một** phân vùng «giờ thường» vòng qua 00:00. Nửa đêm *là* biên của quy tắc khác — ngày hiệu lực voucher (R-VOUCHER-04) và lượt dùng trong ngày (R-VOUCHER-07).
- **Biên nằm ở giờ Việt Nam.** 11:00:00 VN = 04:00:00 UTC. Ca test phải dựng thời điểm bằng `vnTime(…)`; dùng `new Date(2026, 10, 4, 11)` thì kết quả phụ thuộc múi giờ máy chạy test, và ca có thể xanh trên máy dev ở Việt Nam, đỏ trên CI chạy UTC.
- **Độ chính xác 1 giây là giả định.** Nếu yêu cầu được nhận với mili giây, hàng xóm của 11:00:00.000 là 10:59:59.999. Lab và SPEC dùng giây; đáng hỏi lại khi đặc tả nói «tính theo thời điểm hệ thống nhận yêu cầu».

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 150" role="img" aria-label="Trục 24 giờ theo giờ Việt Nam với hai khung cao điểm 11 giờ tới trước 13 giờ và 17 giờ tới trước 19 giờ. Các giá trị biên 2-value ghi ở mép mỗi khung.">
<text x="460" y="20" text-anchor="middle" class="t-b">Giờ đặt theo giờ Việt Nam (UTC+7) — R-FARE-04</text>
<rect x="425" y="44" width="70" height="40" class="f-pk-soft s-pk"/>
<rect x="635" y="44" width="70" height="40" class="f-pk-soft s-pk"/>
<text x="460" y="68" text-anchor="middle" class="t-small t-pk t-b">×1,25</text>
<text x="670" y="68" text-anchor="middle" class="t-small t-pk t-b">×1,25</text>
<line x1="40" y1="84" x2="880" y2="84" class="s-line" stroke-width="2"/>
<text x="40" y="102" text-anchor="middle" class="t-10 t-mono">00:00</text>
<text x="880" y="102" text-anchor="middle" class="t-10 t-mono">24:00</text>
<text x="425" y="38" text-anchor="end" class="t-10 t-mono">10:59:59 |</text>
<text x="428" y="38" text-anchor="start" class="t-10 t-mono t-pk">11:00:00</text>
<text x="495" y="102" text-anchor="end" class="t-10 t-mono t-pk">12:59:59 |</text>
<text x="498" y="102" text-anchor="start" class="t-10 t-mono">13:00:00</text>
<text x="635" y="38" text-anchor="end" class="t-10 t-mono">16:59:59 |</text>
<text x="638" y="38" text-anchor="start" class="t-10 t-mono t-pk">17:00:00</text>
<text x="705" y="102" text-anchor="end" class="t-10 t-mono t-pk">18:59:59 |</text>
<text x="708" y="102" text-anchor="start" class="t-10 t-mono">19:00:00</text>
<text x="200" y="132" text-anchor="middle" class="t-small">đêm + sáng: một phân vùng vòng qua 00:00</text>
<text x="565" y="132" text-anchor="middle" class="t-small">chiều</text>
<text x="800" y="132" text-anchor="middle" class="t-small">tối</text>
</svg>
</div>
<figcaption><b>Hình 2 · 8 giá trị biên 2-value của giờ cao điểm.</b> 3-value thêm 8 giá trị nữa (10:59:58, 11:00:01, …). Trục theo tỉ lệ: mỗi giờ 35 px.</figcaption>
</figure>

### 4.4. Kết quả mong đợi suy từ phân vùng, không từ code

Với mỗi coverage item, kết quả mong đợi là kết quả của **phân vùng chứa nó**: 0,46 km thuộc D2 ⇒ hợp lệ; 100,05 thuộc D3 ⇒ `DISTANCE_OUT_OF_RANGE`; 12:59:59 thuộc «cao điểm trưa» ⇒ `peak = true`. Lab tính mong đợi bằng cùng mô hình phân vùng (`inValidPartition`) — không gọi hàm của sản phẩm để biết «đáng lẽ ra gì».

## 5. Thực hành

**Đề bài (curriculum TEST-032):** áp BVA cho quãng đường (0,44/0,45; 100,04/100,05), khối lượng (0,04/0,05; 30,04/30,05), ngưỡng xe máy 20,0/20,1 kg và giờ 10:59:59/11:00:00/12:59:59/13:00:00; so số test 2-value với 3-value. Tự làm trước, rồi so với lời giải.

### 5.1. Test case mức component (3-value, 52 ca)

| Nhóm | Test item | Giá trị | Mong đợi |
|---|---|---|---|
| Quãng đường | `normalizeDistanceKm` | 8 thô + 8 sau làm tròn (mục 4.2) | hợp lệ ⇔ thuộc D2 |
| Khối lượng | `normalizeWeightKg` | 8 thô + 8 sau làm tròn | hợp lệ ⇔ 0 &lt; w ≤ 30 sau làm tròn; −0,1 và 0,0 bị từ chối |
| Ngưỡng xe máy | `assertVehicleFits(normalizeWeightKg(kg), …)` | 19,9 · 20,0 · 20,1 · 20,2 | xe máy được với 19,9 và 20,0; ô tô được với cả bốn |
| Giờ cao điểm | `isPeakTime(vnTime(2026, 11, 4, h, m, s))` | 16 thời điểm | `true` ⇔ trong [11:00; 13:00) hoặc [17:00; 19:00) |

### 5.2. Test case mức system qua API (14 ca)

Biên phải còn đúng **sau khi** đi qua lớp định dạng, quy tắc đầu vào và tính cước. Lab chọn các ca 2-value trên giá trị thô (giá trị client thật sự gửi):

| Mã | Yêu cầu `POST /api/quotes` | Mong đợi | Oracle |
|---|---|---|---|
| BVA-API-01, 02 | ride, xe máy, SG, 09:00 · 0,44 · 0,45 km | 422 · 200, tổng 15.000 đ | 0,5 km: 12.000 + 2.250 = 14.250 ⇒ tối thiểu 15.000 |
| BVA-API-03, 04 | như trên · 100,04 · 100,05 km | 200, tổng 462.000 đ · 422 | 100,0 km: 12.000 + 450.000 |
| BVA-API-05, 06 | parcel, ô tô, 3 km · 0,04 · 0,05 kg | 422 `WEIGHT_OUT_OF_RANGE` · 200 | R-PARCEL-01 |
| BVA-API-07, 08 | parcel, **xe máy** · 20,04 · 20,05 kg | 200 · 422 `VEHICLE_TOO_SMALL` | 20,0 vẫn xe máy; 20,1 phải ô tô |
| BVA-API-09, 10 | parcel, ô tô · 30,04 · 30,05 kg | 200 · 422 `WEIGHT_OUT_OF_RANGE` | R-PARCEL-01 |
| BVA-API-11…14 | ride, xe máy, 7,4 km, đồng hồ ở 10:59:59 · 11:00:00 · 12:59:59 · 13:00:00 | `peak` false · true · true · false; tổng 46.000 · 57.000 · 57.000 · 46.000 | 45.300 × 1,25 = 56.625 ⇒ 57.000 |

### 5.3. Code và lần chạy thật

Sinh coverage item (`articles/boundary-value-analysis/bva-model.ts`, rút gọn):

```ts
/** 2-value: mỗi giá trị biên + hàng xóm gần nhất thuộc phân vùng LIỀN KỀ. */
export function twoValueItems(domain: OrderedDomain): number[] {
  const items = new Set<number>();
  for (const b of boundaryValues(domain)) {
    items.add(b);
    for (const n of [b - 1, b + 1]) if (partitionOf(domain, n) !== partitionOf(domain, b)) items.add(n);
  }
  return [...items].sort((a, b) => a - b);
}

/** 3-value: mỗi giá trị biên + CẢ HAI hàng xóm. */
export function threeValueItems(domain: OrderedDomain): number[] {
  const items = new Set<number>();
  for (const b of boundaryValues(domain)) [b - 1, b, b + 1].forEach((v) => items.add(v));
  return [...items].sort((a, b) => a - b);
}
```

Ca biên giờ qua API — đồng hồ giả đặt đúng giây (`bva-api.test.ts`, rút gọn):

```ts
it.each([
  { time: [10, 59, 59], peak: false, total: 46_000 },
  { time: [11, 0, 0], peak: true, total: 57_000 },
  { time: [12, 59, 59], peak: true, total: 57_000 },
  { time: [13, 0, 0], peak: false, total: 46_000 },
] as const)('BVA-API $time ⇒ peak $peak, total $total', async ({ time, peak, total }) => {
  const res = await quote(appAt(...time), { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 }).expect(200);
  expect(res.body.quote.peak).toBe(peak);
  expect(res.body.quote.total).toBe(total);
});
```

```bash
npx vitest run articles/boundary-value-analysis
```

Kết quả thực tế (vitest 3.2.7, Node 22, lần chạy khi viết bài):

```text
 ✓ articles/boundary-value-analysis/bva-items.test.ts (6 tests)
 ✓ articles/boundary-value-analysis/bva-component.test.ts (52 tests)
 ✓ articles/boundary-value-analysis/bva-so-sanh.test.ts (4 tests)
 ✓ articles/boundary-value-analysis/bva-api.test.ts (14 tests)

 Test Files  4 passed (4)
      Tests  76 passed (76)
```

Bản cài đặt của lab qua hết 66 ca biên (52 component + 14 API). Để biết bộ ca **có khả năng** bắt lỗi biên hay không, phải cho nó chạy trên bản sai.

### 5.4. Bốn bộ ca trên ba lỗi cố ý

`bien-loi.ts` chứa ba bản cài đặt sai, mỗi bản là một lỗi biên thật người ta hay viết. `bva-so-sanh.test.ts` chạy bốn bộ ca — thô/sau làm tròn × 2-value/3-value — và ghi lại giá trị nào cho kết quả khác bản đúng (ca đỏ):

| Lỗi cố ý | Thô 2-value | Thô 3-value | Sau làm tròn 2-value | Sau làm tròn 3-value |
|---|---|---|---|---|
| **B** — quãng đường: kiểm `raw >= 0.5 && raw <= 100` rồi mới làm tròn | **0,45 · 100,04** | **0,45 · 0,46 · 100,03 · 100,04** | — | — |
| **T** — quãng đường: làm tròn bằng `Number(raw.toFixed(1))` | **100,05** | **100,05** | — | — |
| **K** — khối lượng: từ chối khi `kg === 0` thay vì `kg <= 0` | — | — | — | **−0,1** |

(— = không ca nào đỏ: lỗi lọt.)

Đọc ma trận:

- **Lỗi T là lỗi dấu phẩy động thật.** Số `100.05` trong máy là 100,04999…; `toFixed(1)` làm tròn theo giá trị nhị phân đó nên ra `"100.0"`, và chuyến 100,05 km được nhận. Giá trị «sau làm tròn» gõ tay như 100,1 không bao giờ đi qua đường lỗi này. Cùng cách làm tròn còn biến 7,35 thành 7,3 — trái ví dụ R-FARE-02 — nhưng đó không phải biên, nên không bộ BVA nào ở đây bắt (cần ví dụ oracle hoặc random testing — [TEST-041](/articles/random-testing)).
- **Lỗi K là bản Tới Nơi của ví dụ `x ≤ 10` → `x = 10` trong syllabus.** 2-value thử 0,0 (bị từ chối — đúng) và 0,1 (được nhận — đúng): xanh. Chỉ hàng xóm phía trong phân vùng không hợp lệ, −0,1, mới lộ lỗi. Trên giá trị thô thì 0,03–0,06 đều không âm nên cũng bỏ lọt.
- **Không có bộ nào bắt cả ba.** Bộ tối thiểu bắt hết là **thô 2-value + sau làm tròn 3-value** (4 + 8 = 12 giá trị cho quãng đường hoặc khối lượng).

## 6. Góc nhìn nâng cao

**Đầu vào bị biến đổi trước khi so sánh thì có hai tầng biên.** Làm tròn, cắt khoảng trắng, đổi đơn vị (m → km), đổi múi giờ… đều tạo ra một lớp giá trị trung gian. Biên của quy tắc nằm trên lớp trung gian; lỗi của phép biến đổi nằm trên lớp đầu vào. Ma trận 5.4 cho thấy cần thử cả hai.

**Biên của đầu ra cũng là biên.** Phụ phí bưu kiện R-PARCEL-03 nhảy bậc ở 5,0|5,1, 6,0|6,1, …, 29,0|29,1 — 25 biên nội bộ, 50 coverage item 2-value chỉ cho một tham số. Cước tối thiểu xe máy đổi phân vùng đầu ra giữa 0,6 và 0,7 km. Khi số biên bùng nổ, chọn theo rủi ro: bậc đầu (5,0|5,1; 6,0|6,1 — phân biệt «⌈w − 5⌉» với «phần nguyên + 1») và bậc cuối, ghi rõ phần không phủ.

**BVA một chiều không phủ tương tác.** «20,1 kg xe máy» là biên của *tổ hợp* khối lượng × loại xe. Khi biên là một biểu thức nhiều biến (vd quãng đường × khối lượng quyết định giá), cần domain testing với điểm ON/OFF — [TEST-040](/articles/data-based-test-techniques-domain-testing).

**Biên thay đổi theo phiên bản giá.** `pricingVersion` (R-QUOTE-02) cho phép đổi bảng giá. Khung cao điểm thành [10:30; 13:30) thì toàn bộ ca biên giờ phải sinh lại — lý do lab sinh coverage item từ mô hình thay vì gõ tay.

**Sai lầm thường gặp:**

| Sai lầm | Vì sao sai |
|---|---|
| Chỉ thử giá trị biên của phân vùng **hợp lệ** (0,5 và 100) | Thiếu hàng xóm ở phân vùng kề (0,4; 100,1) — không phân biệt được biên đặt đúng hay lệch |
| Chọn biên theo «số tròn» (1 km, 100 km) thay vì theo đặc tả | Ví dụ 1–100 km trong nhiều tài liệu khác với SPEC [0,5; 100] |
| Áp BVA cho dữ liệu không thứ tự (mã thành phố) | Không có «hàng xóm»; dùng EP |
| Không ghi độ chính xác | Người khác không tái tạo được «hàng xóm»; cãi nhau 0,44 hay 0,449 |
| Dựng thời điểm test bằng giờ máy | Ca biên giờ xanh/đỏ tuỳ múi giờ máy chạy |
| Tin 100% BVA coverage là «biên đúng» | Coverage đo item đã thử, không đo việc chọn đúng lớp giá trị (lỗi T, K) |

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Mốc 2 phút và tài xế trễ

R-CANCEL-02: khách huỷ chuyến `accepted` khi `now − acceptedAt < 2 phút` thì miễn phí; đúng 2:00 thì có phí. R-CANCEL-04: nếu `now > acceptedAt + etaMinutes + 10 phút` thì miễn phí do tài xế trễ; đúng bằng mốc vẫn thu. Với `etaMinutes = 5`, độ chính xác 1 giây, liệt kê coverage item 2-value và 3-value cho **thời gian từ lúc nhận chuyến tới lúc huỷ**, kèm `feeReason` mong đợi.

<details><summary><b>Lời giải mẫu</b></summary><ul><li>Phân vùng (theo thời gian trôi): [0; 1:59] <code>grace_period</code> · [2:00; 15:00] <code>charged</code> · ≥ 15:01 <code>driver_late</code>.</li><li><b>2-value</b> (4 item): 1:59 → grace_period; 2:00 → charged; 15:00 → charged; 15:01 → driver_late.</li><li><b>3-value</b> (8 item): thêm 1:58 (grace_period), 2:01 (charged), 14:59 (charged), 15:02 (driver_late).</li><li>Biên dưới 0:00 (huỷ ngay khi nhận) là biên của phân vùng đầu tiên nhưng không có phân vùng kề phía dưới (thời gian âm không xảy ra) — ghi nhận và thử 0:00 như một ca EP.</li><li>Bẫy hay gặp: dùng <code>&lt;=</code> ở R-CANCEL-04 làm 15:00 thành miễn phí — chỉ ca 15:00 lộ ra.</li></ul></details>

### Bài tập 2 — Tiền món tối thiểu

R-FOOD-02, R-FOOD-03: đơn giá là bội số 1.000 đ; tiền món &lt; 30.000 đ ⇒ `SUBTOTAL_TOO_LOW`; đúng 30.000 đ hợp lệ. Độ chính xác của tiền món là bao nhiêu? Liệt kê coverage item 2-value và 3-value quanh biên này, và một đơn món cho mỗi giá trị.

<details><summary><b>Lời giải mẫu</b></summary><ul><li>Vì đơn giá là bội số 1.000 đ và số lượng nguyên, tiền món luôn là bội số 1.000 đ ⇒ độ chính xác <b>1.000 đ</b>, không phải 1 đ. Chọn 29.999 đ là chọn giá trị không thể tạo ra qua API hợp lệ.</li><li><b>2-value:</b> 29.000 (lỗi) · 30.000 (hợp lệ). <b>3-value:</b> thêm 28.000 (lỗi) · 31.000 (hợp lệ).</li><li>Đơn mẫu: 1 × 29.000; 1 × 30.000; 1 × 28.000; 1 × 31.000 — hoặc 2 × 15.000 cho 30.000 để thử phép nhân số lượng.</li></ul></details>

### Câu hỏi trắc nghiệm

**Câu 1 (K1).** Theo CTFL v4.0.1, BVA chỉ dùng được với loại phân vùng nào?

- a) Phân vùng hợp lệ
- b) Phân vùng có thứ tự
- c) Phân vùng hữu hạn
- d) Phân vùng của đầu vào giao diện

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — BVA thử cả biên của phân vùng không hợp lệ.</li><li>b) Đúng — mục 4.2.2: cần thứ tự để có min, max và hàng xóm.</li><li>c) Sai — phân vùng có thể không chặn một đầu (D3: ≥ 100,05).</li><li>d) Sai — áp được cho đầu ra, giá trị thời gian, tham số giao diện…</li></ul></details>

**Câu 2 (K3).** Quãng đường thô làm tròn 0,1 km half-up rồi phải trong [0,5; 100]. Độ chính xác giả định 0,01 km. Bộ nào là đủ và đúng coverage item 2-value?

- a) 0,5 · 100
- b) 0,4 · 0,5 · 100 · 100,1
- c) 0,44 · 0,45 · 100,04 · 100,05
- d) 0,44 · 0,45 · 0,46 · 100,04 · 100,05 · 100,06

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Sai — thiếu hàng xóm ở phân vùng kề.</li><li>b) Sai — đó là biên của giá trị <i>sau làm tròn</i>, không phải của đầu vào thô ở độ chính xác 0,01.</li><li>c) Đúng — min/max của D2 thô và hàng xóm ở D1, D3.</li><li>d) Sai — trộn thêm một phần của 3-value, vẫn thiếu 0,43 và 100,03.</li></ul></details>

**Câu 3 (K3).** Cao điểm là [11:00; 13:00) và [17:00; 19:00), độ chính xác 1 giây; nửa đêm không phải biên. Áp 3-value BVA thì có bao nhiêu coverage item?

- a) 8
- b) 12
- c) 16
- d) 24

<details><summary><b>Đáp án: c</b></summary><ul><li>8 giá trị biên (10:59:59, 11:00:00, 12:59:59, 13:00:00 và bốn giá trị của khung tối); mỗi giá trị biên thêm hàng xóm phía trong phân vùng của nó: 10:59:58, 11:00:01, 12:59:58, 13:00:01, … ⇒ 16.</li><li>a) Sai — đó là 2-value.</li><li>b) Sai — chỉ thêm hàng xóm cho một nửa số biên.</li><li>d) Sai — đếm trùng: hàng xóm phía ngoài của mỗi biên đã là giá trị biên của phân vùng kề.</li></ul></details>

**Câu 4 (K2).** Kiểm khối lượng được cài đặt nhầm `kg === 0` thay vì `kg <= 0` (trên giá trị sau làm tròn 0,1). Vì sao 3-value BVA tìm được lỗi còn 2-value thì không?

- a) 3-value thử nhiều phân vùng hơn
- b) 3-value thêm hàng xóm −0,1 phía trong phân vùng không hợp lệ; với 0,0 và 0,1 bản sai cho kết quả giống bản đúng
- c) 2-value không thử phân vùng không hợp lệ
- d) 3-value dùng độ chính xác nhỏ hơn

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — số phân vùng như nhau.</li><li>b) Đúng — giống ví dụ <code>x ≤ 10</code> / <code>x = 10</code> của mục 4.2.2.</li><li>c) Sai — 2-value thử 0,0, thuộc phân vùng không hợp lệ.</li><li>d) Sai — cùng độ chính xác 0,1.</li></ul></details>

**Câu 5 (K3).** R-PARCEL-02: w &gt; 20 kg bắt buộc ô tô (w sau làm tròn 0,1). Với xe máy, cặp giá trị nào là coverage item 2-value của biên này?

- a) 20,0 và 20,1
- b) 19,9 và 20,0
- c) 20,0 và 21,0
- d) 20 và 30

<details><summary><b>Đáp án: a</b></summary><ul><li>a) Đúng — max của «xe máy được» và min của «bắt buộc ô tô».</li><li>b) Sai — cả hai cùng một phân vùng; không phân biệt được <code>&gt;</code> với <code>&gt;=</code>.</li><li>c) Sai — 21,0 không phải hàng xóm gần nhất ở độ chính xác 0,1.</li><li>d) Sai — 30 là biên của quy tắc khác (R-PARCEL-01).</li></ul></details>

**Câu 6 (K2).** Đội gõ tay giá trị biên «sau làm tròn» (0,4; 0,5; 100,0; 100,1) và gửi thẳng qua API. Lỗi nào có thể lọt?

- a) Quên cận trên 100 km
- b) Cài đặt so sánh `km < 0.5` thành `km <= 0.5`
- c) Hàm làm tròn sai với một số giá trị thô như 100,05
- d) Mã lỗi trả về sai

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Sai — 100,1 bị nhận sẽ lộ lỗi.</li><li>b) Sai — 0,5 bị từ chối sẽ lộ lỗi.</li><li>c) Đúng — giá trị đã «tròn» không đi qua đường sai của phép làm tròn (lỗi T ở mục 5.4).</li><li>d) Sai — mỗi ca đều kiểm mã lỗi.</li></ul></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 4.2.2 (FL-4.2.2) — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level Test Analyst Syllabus* v4.0: mục 3.1.1 (TA-3.1.1, domain testing, điểm ON/OFF «theo độ chính xác đã cho») — https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (mục *boundary value analysis*, *boundary value*).
- ISO/IEC/IEEE 29119-4, *Software and systems engineering — Software testing — Part 4: Test techniques* (boundary value analysis).
- G. J. Myers, C. Sandler, T. Badgett, *The Art of Software Testing*, 3rd ed., 2011 — một trong các nguồn CTFL dẫn cho 2-value BVA.
- Lab Tới Nơi: `SPEC.md` (R-FARE-02, R-FARE-04, R-PARCEL-01, R-PARCEL-02, R-GEN-02) và `articles/boundary-value-analysis/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-031 Equivalence Partitioning](/articles/equivalence-partitioning) · [TEST-019 Confirmation và Regression Testing](/articles/confirmation-testing-va-regression-testing) · [TEST-040 Domain testing](/articles/data-based-test-techniques-domain-testing) · [TEST-041 Random Testing](/articles/random-testing) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)

Bài tiếp theo: [TEST-033 · Decision Table Testing](/articles/decision-table-testing)
