---
slug: equivalence-partitioning
title: "Equivalence Partitioning: phân vùng tương đương cho quãng đường chuyến xe"
summary: "Phân vùng tương đương chia miền dữ liệu thành các nhóm được xử lý giống nhau để chọn ít test case mà đủ. Bài áp EP theo CTFL v4.0.1 lên quãng đường, thành phố, loại xe của API báo giá Tới Nơi, đo EP và Each Choice coverage."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-031, equivalence-partitioning, black-box, test-design, toi-noi]
language: vi
---

Quãng đường trong yêu cầu báo giá là một số thực: có vô số giá trị, và không ai thử hết được. Equivalence Partitioning (EP — phân vùng tương đương) trả lời câu hỏi: **chọn những giá trị nào, bao nhiêu là đủ, và làm sao chứng minh là đủ?** Bài áp kỹ thuật này lên ba tham số của `POST /api/quotes` trong lab Tới Nơi, viết test thật ở hai test level và đo độ phủ bằng con số.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-031</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S04 · Black-box Test Design Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1 (đọc thêm Advanced — CTAL-TA v4.0)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Component; System</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Equivalence Partitioning</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/testware-san-pham-kiem-thu">TEST-010 · Testware</a>, <a href="/articles/functional-testing">TEST-017 · Functional Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Phân vùng quãng đường R-FARE-02 (&lt; 0,5 km, [0,5; 100], &gt; 100 km, không phải số), mã thành phố R-GEN-03 và loại xe; <code>test.each</code> cho <code>calculateFare</code> và <code>POST /api/quotes</code> — lab <code>articles/equivalence-partitioning</code> (34 test case vitest)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 4 · mục 4.1 và 4.2.1 · FL-4.2.1 (K3). Đọc thêm: CTAL-TA v4.0 · Chương 3 · mục 3.1 và 3.1.1 · TA-3.1.1 (K3) — domain testing, đi sâu ở TEST-040</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Ở [TEST-017](/articles/functional-testing) ta lấy sẵn các ví dụ trong SPEC (5,0 · 5,1 · 6,0 · 6,1 · 30 kg) để kiểm phụ phí bưu kiện và hứa rằng *cách chọn* giá trị là việc của Series 04. Bài này bắt đầu thực hiện lời hứa đó bằng kỹ thuật nền tảng nhất.

Vấn đề EP giải quyết là **bùng nổ đầu vào**. `distanceKm` nhận mọi số thực; `cityCode` nhận mọi chuỗi; tổ hợp của chúng là vô hạn ([TEST-003](/articles/bay-nguyen-ly-kiem-thu), nguyên lý «exhaustive testing is impossible»). EP thay câu hỏi «thử giá trị nào?» bằng câu hỏi dễ hơn: **«hệ thống xử lý các giá trị này theo bao nhiêu cách khác nhau?»** — mỗi cách là một phân vùng, mỗi phân vùng thử một lần.

Khi nào dùng: gần như mọi lúc có dữ liệu — tham số hàm, trường JSON, cấu hình, khoảng thời gian, cả **đầu ra**. EP là bước đầu tiên trước BVA ([TEST-032](/articles/boundary-value-analysis)), bảng quyết định ([TEST-033](/articles/decision-table-testing)) và các kỹ thuật tổ hợp ([TEST-036](/articles/classification-tree-method)…[TEST-038](/articles/combinatorial-testing)).

## 2. Lý thuyết theo ISTQB

### 2.1. EP trong nhóm kỹ thuật black-box

CTFL v4.0.1 mục 4.1 chia test technique thành **black-box** (dựa trên hành vi được đặc tả), **white-box** (dựa trên cấu trúc bên trong) và **experience-based** (dựa trên kinh nghiệm). Kỹ thuật hỗ trợ cả *test analysis* (kiểm cái gì) lẫn *test design* (kiểm thế nào). Test case black-box không phụ thuộc cách cài đặt: đổi code mà hành vi giữ nguyên thì bộ test vẫn dùng được.

### 2.2. Định nghĩa và giả định cốt lõi

Theo mục 4.2.1, EP chia dữ liệu thành các **equivalence partition** (phân vùng tương đương) dựa trên **kỳ vọng** rằng mọi phần tử trong cùng phân vùng được test item xử lý *theo cùng một cách*. Hệ quả: nếu một giá trị trong phân vùng làm lộ defect thì giá trị khác trong cùng phân vùng cũng làm lộ defect đó ⇒ **một test case cho mỗi phân vùng là đủ**.

Chữ quan trọng là «kỳ vọng». EP là một **giả thuyết** về cách hệ thống xử lý dữ liệu. Giả thuyết sai — tức trong một «phân vùng» thật ra có hai cách xử lý — thì một test case không còn đủ.

Các tính chất syllabus nêu:

| Tính chất | Ý nghĩa | Ví dụ Tới Nơi |
|---|---|---|
| Phân vùng cho **mọi phần tử dữ liệu** của test item | Đầu vào, đầu ra, cấu hình, giá trị nội bộ, giá trị thời gian, tham số giao diện | đầu vào `distanceKm`; đầu ra «có bị nâng lên cước tối thiểu»; thời gian «trong/ngoài cao điểm» |
| Liên tục/rời rạc, có/không thứ tự, hữu hạn/vô hạn | Ảnh hưởng tới việc có dùng được BVA không | quãng đường: liên tục, có thứ tự; mã thành phố: rời rạc, không thứ tự |
| **Không chồng lấn, không rỗng** | Mỗi giá trị thuộc đúng một phân vùng; không có phân vùng không chứa giá trị nào | xem phân vùng đầu ra của ô tô ở mục 4.3 |

### 2.3. Phân vùng hợp lệ và không hợp lệ

Phân vùng chứa giá trị hợp lệ là **valid partition**; chứa giá trị không hợp lệ là **invalid partition**. Syllabus lưu ý định nghĩa «hợp lệ» **có thể khác nhau** giữa các đội: có đội hiểu hợp lệ là «được hệ thống xử lý», có đội hiểu là «được đặc tả mô tả cách xử lý»; tương tự, không hợp lệ là «bị bỏ qua/từ chối» hoặc «không có cách xử lý nào trong đặc tả». Trong bài này: hợp lệ = báo giá thành công; không hợp lệ = bị từ chối với một mã lỗi của R-API-06.

### 2.4. Coverage

Trong EP, **coverage item** (hạng mục phủ) chính là các phân vùng. Muốn đạt 100%, các test case phải chạm **mọi** phân vùng đã xác định — **kể cả phân vùng không hợp lệ** — ít nhất một lần:

```text
EP coverage = số phân vùng được ít nhất một ca chạm / tổng số phân vùng đã xác định × 100%
```

Test item có nhiều tham số thì có nhiều *tập* phân vùng, và một test case chạm phân vùng của nhiều tập cùng lúc. Tiêu chí đơn giản nhất cho trường hợp này là **Each Choice coverage** (syllabus dẫn Ammann 2016): mỗi phân vùng của mỗi tập được chạm ít nhất một lần; tiêu chí này **không** xét tổ hợp giữa các phân vùng (tổ hợp là việc của [TEST-036](/articles/classification-tree-method)…[TEST-038](/articles/combinatorial-testing)).

Ghi chú phát hành của CTFL v4.0.1 nói bản này bổ sung vào mục 4.2.1 ý: **phân vùng không hợp lệ nên được kiểm riêng từng cái** để tránh **defect masking** (che lỗi) — hai giá trị sai trong cùng một test case thì lỗi thứ nhất có thể che mất cách xử lý giá trị thứ hai.

### 2.5. EP trong CTAL-TA v4.0

CTAL-TA v4.0 mục 3.1 gọi EP và BVA là **data-based test techniques** (kỹ thuật dựa trên dữ liệu) và giới thiệu thêm ba kỹ thuật cùng nhóm: **domain testing** (mở rộng EP, BVA sang miền nhiều tham số, phân vùng phức tạp — TA-3.1.1, K3), combinatorial testing và random testing. Bản v4.0 đã thay các LO riêng về EP, BVA của bản cũ bằng TA-3.1.1. Bài này dừng ở mức CTFL; miền hai biến quãng đường × khối lượng với điểm ON/OFF/IN/OUT ở [TEST-040](/articles/data-based-test-techniques-domain-testing).

### 2.6. Bảng thuật ngữ Anh – Việt

| Thuật ngữ (ISTQB) | Tiếng Việt dùng trong bài | Ghi chú |
|---|---|---|
| equivalence partitioning (EP) | phân vùng tương đương | kỹ thuật black-box |
| equivalence partition | phân vùng | tập giá trị kỳ vọng được xử lý như nhau |
| valid / invalid partition | phân vùng hợp lệ / không hợp lệ | định nghĩa tuỳ đội — ghi rõ |
| coverage item | hạng mục phủ | với EP: chính là phân vùng |
| Each Choice coverage | phủ mỗi lựa chọn | mỗi phân vùng của mỗi tham số ≥ 1 lần |
| defect masking | che lỗi | lỗi này làm không quan sát được lỗi kia |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Sách giáo khoa (vd Jorgensen) phân biệt <i>weak</i> (≈ Each Choice) và <i>strong</i> (mọi tổ hợp phân vùng) equivalence class testing, biến thể <i>robust</i> thêm phân vùng không hợp lệ. CTFL v4.0.1 không dùng các tên này.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Trước khi đặt chuyến, ứng dụng khách gửi yêu cầu báo giá. Với 2 triệu chuyến mỗi ngày ([hồ sơ dự án](/articles/du-an-mau-toi-noi)), 1% yêu cầu bị từ chối sai là hàng chục nghìn lượt đặt hỏng mỗi ngày; một yêu cầu sai lọt qua là một báo giá không ai chở nổi.

### 3.2. Business rules dùng trong bài

- **R-FARE-02** — quãng đường làm tròn 0,1 km half-up **trước** mọi bước, rồi phải nằm trong [0,5; 100] km tính cả hai đầu; ngoài khoảng ⇒ `DISTANCE_OUT_OF_RANGE`.
- **R-GEN-03** — 6 thành phố `SG`, `HN`, `DN`, `HP`, `CT`, `NT`; mã khác ⇒ `CITY_NOT_SUPPORTED`.
- **R-GEN-04** — loại xe `bike` hoặc `car`.
- **R-FARE-01, R-FARE-07** — bảng giá và cước tối thiểu (xe máy 15.000 đ, ô tô 30.000 đ).
- **R-API-07** — `POST /api/quotes` thân `{ kind, cityCode, vehicle, distanceKm, voucherCode? }`; **R-API-02** — lỗi định dạng trả `VALIDATION_ERROR` kèm `error.details` (danh sách trường sai).
- **R-VALID-01** — nhiều lỗi cùng lúc chỉ báo lỗi đầu: … định dạng (400) → `CITY_NOT_SUPPORTED` → `DISTANCE_OUT_OF_RANGE` → …

### 3.3. System context

Trong kiến trúc đích, ứng dụng khách gọi **Pricing & Quote Service** (kiến trúc §4.3) qua API Gateway/BFF; quãng đường lấy từ bản đồ MapViet. Lab giản lược: client gửi thẳng `distanceKm` (R-GEN-06), mọi thứ trong một tiến trình. Yêu cầu đi qua ba lớp, và **mỗi lớp từ chối một nhóm phân vùng khác nhau**:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 250" role="img" aria-label="Yêu cầu báo giá đi qua ba lớp của lab: kiểm định dạng bằng zod trả 400 cho phân vùng D4, C3, V3; quy tắc nghiệp vụ trả 422 cho C2, D1, D3; tính cước trả 200 cho D2, C1, V1, V2.">
<defs>
<marker id="t031-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="460" y="24" text-anchor="middle" class="t-b">POST /api/quotes — mỗi lớp «tiêu thụ» một nhóm phân vùng</text>
<rect x="20" y="50" width="150" height="60" rx="8" class="f-surface s-line"/>
<text x="95" y="72" text-anchor="middle" class="t-small t-b">Yêu cầu JSON</text>
<text x="95" y="89" text-anchor="middle" class="t-10 t-mono">cityCode, vehicle,</text>
<text x="95" y="102" text-anchor="middle" class="t-10 t-mono">distanceKm</text>
<rect x="215" y="50" width="200" height="60" rx="8" class="f-pk-soft s-pk"/>
<text x="315" y="76" text-anchor="middle" class="t-b t-pk">1 · Định dạng (zod)</text>
<rect x="460" y="50" width="200" height="60" rx="8" class="f-warn-soft s-warn"/>
<text x="560" y="76" text-anchor="middle" class="t-b t-warn">2 · Quy tắc đầu vào</text>
<text x="560" y="96" text-anchor="middle" class="t-small t-mono">normalizePriceRequest</text>
<rect x="705" y="50" width="195" height="60" rx="8" class="f-phys-soft s-phys"/>
<text x="802" y="76" text-anchor="middle" class="t-b t-phys">3 · Tính cước</text>
<text x="802" y="96" text-anchor="middle" class="t-small t-mono">calculateFare</text>
<line x1="170" y1="80" x2="212" y2="80" class="s-muted" stroke-width="1.5" marker-end="url(#t031-a)"/>
<line x1="415" y1="80" x2="457" y2="80" class="s-muted" stroke-width="1.5" marker-end="url(#t031-a)"/>
<line x1="660" y1="80" x2="702" y2="80" class="s-muted" stroke-width="1.5" marker-end="url(#t031-a)"/>
<line x1="315" y1="110" x2="315" y2="140" class="s-pk" stroke-width="1.5" marker-end="url(#t031-a)"/>
<line x1="560" y1="110" x2="560" y2="140" class="s-warn" stroke-width="1.5" marker-end="url(#t031-a)"/>
<line x1="802" y1="110" x2="802" y2="140" class="s-phys" stroke-width="1.5" marker-end="url(#t031-a)"/>
<rect x="215" y="144" width="200" height="86" rx="8" class="f-bg s-pk dash"/>
<text x="315" y="168" text-anchor="middle" class="t-small t-b">400 VALIDATION_ERROR</text>
<text x="315" y="190" text-anchor="middle" class="t-small">D4 không phải số</text>
<text x="315" y="206" text-anchor="middle" class="t-small">C3 sai định dạng</text>
<text x="315" y="222" text-anchor="middle" class="t-small">V3 khác bike/car</text>
<rect x="460" y="144" width="200" height="86" rx="8" class="f-bg s-warn dash"/>
<text x="560" y="168" text-anchor="middle" class="t-small t-b">422 theo R-VALID-01</text>
<text x="560" y="190" text-anchor="middle" class="t-small">C2 → CITY_NOT_SUPPORTED</text>
<text x="560" y="206" text-anchor="middle" class="t-small">D1, D3 →</text>
<text x="560" y="222" text-anchor="middle" class="t-small">DISTANCE_OUT_OF_RANGE</text>
<rect x="705" y="144" width="195" height="86" rx="8" class="f-bg s-phys dash"/>
<text x="802" y="168" text-anchor="middle" class="t-small t-b">200 báo giá</text>
<text x="802" y="190" text-anchor="middle" class="t-small">D2 · C1 · V1 hoặc V2</text>
<text x="802" y="206" text-anchor="middle" class="t-small">+ phân vùng đầu ra:</text>
<text x="802" y="222" text-anchor="middle" class="t-small">có/không cước tối thiểu</text>
</svg>
</div>
<figcaption><b>Hình 1 · Ba lớp, ba nhóm phân vùng.</b> Ký hiệu D/C/V là phân vùng của quãng đường, thành phố, loại xe (mục 4.2). Ở mức component (<code>calculateFare</code>), không còn lớp 1 nên phân vùng «không phải số» biến thành «số không hữu hạn» (NaN, Infinity).</figcaption>
</figure>

### 3.4. Rủi ro có thể xảy ra

| Rủi ro | Hậu quả | Phân vùng sẽ lộ ra |
|---|---|---|
| Quên kiểm cận trên 100 km | Báo giá chuyến 150 km, không tài xế nhận, khách chờ tới khi hệ thống huỷ (R-STATE-05) | D3 |
| Chuỗi `"7.4"` được ép kiểu ngầm thành số | Ứng dụng cũ gửi sai kiểu vẫn «chạy», tới ngày đổi thư viện thì vỡ hàng loạt | D4 |
| Thành phố chưa mở (`DL`) được báo giá | Khách đặt được chuyến ở nơi không có tài xế | C2 |
| Xe máy quãng ngắn không được nâng lên cước tối thiểu | Thu thiếu ở mọi chuyến dưới 0,7 km | đầu ra «có cước tối thiểu» |

## 4. Phân tích kỹ thuật

### 4.1. Bước 1 — xác định test item và phần tử dữ liệu

Hai test item, cùng một test basis (SPEC):

| Test item | Test level | Phần tử dữ liệu |
|---|---|---|
| `calculateFare({ vehicle, distanceKm, peak, raining, … })` | Component | đầu vào `vehicle`, `distanceKm`; đầu ra `total`, `afterDiscount` so với `minimumFare` |
| `POST /api/quotes` dịch vụ `ride` | System (qua API) | trường JSON `distanceKm`, `cityCode`, `vehicle`; đầu ra HTTP status, `error.code`, `quote.total` |

Thành phố **không có** trong chữ ký `calculateFare` — phân vùng C1–C3 chỉ tồn tại ở mức API. Phân vùng gắn với test item, không gắn với «tính năng» chung chung.

### 4.2. Bước 2 — phân vùng đầu vào

| Mã | Tham số | Phân vùng | Hợp lệ? | Kết quả mong đợi | Đại diện |
|---|---|---|---|---|---|
| D1 | `distanceKm` | số, sau làm tròn &lt; 0,5 (gồm 0, số âm) | Không | 422 `DISTANCE_OUT_OF_RANGE` | 0,2 |
| D2 | `distanceKm` | số, sau làm tròn trong [0,5; 100] | Có | 200 | 7,4 · 50 |
| D3 | `distanceKm` | số, sau làm tròn &gt; 100 | Không | 422 `DISTANCE_OUT_OF_RANGE` | 150 |
| D4 | `distanceKm` | không phải số JSON: chuỗi, `null`, thiếu trường | Không | 400 `VALIDATION_ERROR`, `details` chỉ `distanceKm` | `"7.4"` |
| C1 | `cityCode` | một trong 6 mã R-GEN-03 | Có | 200 | `SG`, `HN` |
| C2 | `cityCode` | chuỗi 1–10 ký tự, không phải mã được phục vụ | Không | 422 `CITY_NOT_SUPPORTED` | `DL` |
| C3 | `cityCode` | sai định dạng: rỗng, &gt; 10 ký tự, không phải chuỗi, thiếu | Không | 400 `VALIDATION_ERROR` | `""` |
| V1 | `vehicle` | `bike` | Có | 200 | `bike` |
| V2 | `vehicle` | `car` | Có | 200 | `car` |
| V3 | `vehicle` | giá trị khác | Không | 400 `VALIDATION_ERROR` | `"truck"` |

Ba quyết định đáng giải thích:

- **D1 gộp 0 và số âm** vì SPEC xử lý chúng như nhau; có mã lỗi riêng cho số âm thì phải tách.
- **C2 và C3 tách nhau** dù đều «thành phố sai»: C3 bị chặn ở lớp định dạng (400), C2 ở lớp nghiệp vụ (422).
- **V1 và V2 là hai phân vùng hợp lệ** vì hai bảng giá khác nhau (R-FARE-01).

Biên giữa D1 và D2 nằm ở đâu trên giá trị **thô** client gửi lên? Vì làm tròn trước, D2 là [0,45; 100,05) theo giá trị thô. Đặt biên đúng chỗ và chọn giá trị sát biên là việc của BVA — EP chỉ cần mỗi phân vùng một đại diện, nên ta chọn giá trị **ở giữa** (0,2; 7,4; 150) để không phụ thuộc vào biên.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 220" role="img" aria-label="Trục số quãng đường thô chia ba phân vùng: D1 nhỏ hơn 0,45, D2 từ 0,45 tới dưới 100,05, D3 từ 100,05 trở lên. Đại diện được chọn ở giữa mỗi phân vùng: 0,2; 7,4 và 50; 150. Phân vùng D4 không phải số nằm ngoài trục.">
<text x="460" y="24" text-anchor="middle" class="t-b">Phân vùng quãng đường (giá trị thô client gửi) — trục không theo tỉ lệ</text>
<rect x="40" y="50" width="170" height="44" rx="6" class="f-warn-soft s-warn"/>
<text x="125" y="70" text-anchor="middle" class="t-b t-warn">D1 · không hợp lệ</text>
<text x="125" y="86" text-anchor="middle" class="t-small">thô &lt; 0,45 ⇒ &lt; 0,5 km</text>
<rect x="210" y="50" width="480" height="44" rx="6" class="f-phys-soft s-phys"/>
<text x="450" y="70" text-anchor="middle" class="t-b t-phys">D2 · hợp lệ</text>
<text x="450" y="86" text-anchor="middle" class="t-small">0,45 ≤ thô &lt; 100,05 ⇒ [0,5; 100] km</text>
<rect x="690" y="50" width="190" height="44" rx="6" class="f-warn-soft s-warn"/>
<text x="785" y="70" text-anchor="middle" class="t-b t-warn">D3 · không hợp lệ</text>
<text x="785" y="86" text-anchor="middle" class="t-small">thô ≥ 100,05 ⇒ &gt; 100 km</text>
<line x1="40" y1="122" x2="880" y2="122" class="s-line" stroke-width="2"/>
<line x1="210" y1="112" x2="210" y2="132" class="s-dead" stroke-width="2"/>
<line x1="690" y1="112" x2="690" y2="132" class="s-dead" stroke-width="2"/>
<text x="210" y="150" text-anchor="middle" class="t-small t-mono">0,45</text>
<text x="690" y="150" text-anchor="middle" class="t-small t-mono">100,05</text>
<circle cx="125" cy="122" r="7" class="f-warn s-warn"/>
<text x="125" y="150" text-anchor="middle" class="t-small t-b">0,2</text>
<circle cx="300" cy="122" r="7" class="f-phys s-phys"/>
<text x="300" y="150" text-anchor="middle" class="t-small t-b">7,4</text>
<circle cx="520" cy="122" r="7" class="f-phys s-phys"/>
<text x="520" y="150" text-anchor="middle" class="t-small t-b">50</text>
<circle cx="785" cy="122" r="7" class="f-warn s-warn"/>
<text x="785" y="150" text-anchor="middle" class="t-small t-b">150</text>
<rect x="210" y="170" width="480" height="38" rx="6" class="f-bg s-warn dash"/>
<text x="450" y="194" text-anchor="middle" class="t-small">D4 · ngoài trục số: "7.4" (chuỗi), null, thiếu trường ⇒ 400</text>
</svg>
</div>
<figcaption><b>Hình 2 · Một đại diện ở giữa mỗi phân vùng.</b> Vạch đỏ là biên — EP không cần biết chính xác, BVA ở TEST-032 mới cần. Chấm tròn là giá trị dùng trong test case EP-01…EP-04.</figcaption>
</figure>

### 4.3. Bước 3 — phân vùng đầu ra

Syllabus cho phép phân vùng cả đầu ra. Với `calculateFare`, R-FARE-07 tạo ra hai cách xử lý: **bị nâng lên cước tối thiểu** hay **giữ cước tính được**. Không voucher, không cao điểm:

| Loại xe | Điều kiện nâng lên tối thiểu | Quãng đường hợp lệ rơi vào | Phân vùng «bị nâng» |
|---|---|---|---|
| `bike` | 12.000 + 4.500 × d &lt; 15.000 ⇔ d &lt; 0,667 | d = 0,5 (14.250 đ) và 0,6 (14.700 đ) | **có** — đại diện 0,5 km ⇒ 15.000 đ |
| `car` | 25.000 + 11.000 × d &lt; 30.000 ⇔ d &lt; 0,455 | không có (0,5 km đã là 30.500 đ) | **rỗng** |

Phân vùng phải không rỗng, nên với ô tô (không voucher) chỉ có một phân vùng đầu ra. Test case EPO-04 không «tin» phép tính tay mà quét cả 996 quãng đường hợp lệ (bước 0,1 km) để khẳng định điều đó. Có voucher thì vùng này có giá trị cho cả ô tô — việc của bảng quyết định ([TEST-033](/articles/decision-table-testing)).

### 4.4. EP làm lộ câu hỏi về đặc tả

Khi xếp đại diện vào phân vùng, có giá trị **không biết xếp đâu**: `"sg"` (chữ thường), `" SG"` (khoảng trắng). R-VOUCHER-02 nói rõ mã voucher được bỏ khoảng trắng và viết hoa, nhưng R-GEN-03 **không nói gì** về mã thành phố. Cài đặt hiện tại xếp `"sg"` vào C2 (`CITY_NOT_SUPPORTED`). Đó là *hành vi*, chưa phải *yêu cầu* — tester hỏi BA thay vì tự quyết. Phân vùng sớm cũng là một cách review đặc tả ([TEST-030](/articles/shift-left-testing)).

### 4.5. Bước 4 — từ phân vùng tới test case theo Each Choice

Ba tập phân vùng: quãng đường (1 hợp lệ + 3 không hợp lệ), thành phố (1 + 2), loại xe (2 + 1).

- **Test case hợp lệ:** phủ mọi phân vùng hợp lệ, có thể ghép nhiều phân vùng vào một test case. Tập lớn nhất có 2 phân vùng hợp lệ (V1, V2) ⇒ **2 test case**.
- **Test case không hợp lệ:** mỗi phân vùng không hợp lệ **một test case riêng**, các tham số còn lại hợp lệ ⇒ 3 + 2 + 1 = **6 test case**.

Tổng **8 test case** cho Each Choice 100%, so với 4 × 3 × 3 = 36 nếu nhân mọi phân vùng.

Vì sao không gộp các test case sai cho đỡ tốn? Gửi `{ cityCode: "DL", distanceKm: 0.2 }`: theo R-VALID-01 hệ thống chỉ báo `CITY_NOT_SUPPORTED`, **không bao giờ** chạy tới kiểm quãng đường. Nếu kiểm D1 bị hỏng (vd quên cận dưới) thì test case này vẫn xanh — defect masking. Test case đó chỉ được tính là phủ C2.

## 5. Thực hành

**Đề bài (curriculum TEST-031):** phân vùng quãng đường R-FARE-02 (&lt; 0,5 km, [0,5; 100], &gt; 100 km, không phải số) và mã thành phố R-GEN-03, rồi viết `test.each` cho `calculateFare` và `POST /api/quotes`. Tự làm trên giấy trước, rồi so với lời giải.

### 5.1. Test case mức system (qua API)

Điều kiện chung: 09:00 giờ VN (ngoài cao điểm), không mưa, không voucher, khách `C001`, dịch vụ `ride`.

| Mã | Phủ | Đầu vào khác mặc định (`SG`, `bike`, 7,4 km) | Mong đợi | Oracle |
|---|---|---|---|---|
| EP-01 | D2, C1, V1 | — | 200, `total` = 46.000 | 12.000 + 4.500 × 7,4 = 45.300 ⇒ 46.000 |
| EP-02 | D2, C1, V2 | `HN`, `car`, 50 km | 200, `total` = 575.000 | 25.000 + 11.000 × 50 |
| EP-03 | D1 | 0,2 km | 422 `DISTANCE_OUT_OF_RANGE` | R-FARE-02 |
| EP-04 | D3 | 150 km | 422 `DISTANCE_OUT_OF_RANGE` | R-FARE-02 |
| EP-05 | D4 | `"7.4"` (chuỗi) | 400 `VALIDATION_ERROR`, `details` = [`distanceKm`] | R-API-02 |
| EP-06 | C2 | `DL` | 422 `CITY_NOT_SUPPORTED` | R-GEN-03 |
| EP-07 | C3 | `""` | 400 `VALIDATION_ERROR`, `details` = [`cityCode`] | R-API-02 |
| EP-08 | V3 | `truck` | 400 `VALIDATION_ERROR`, `details` = [`vehicle`] | R-GEN-04 |

Với lỗi định dạng, test case kiểm `details` **chỉ chứa đúng trường đang kiểm** — nếu không, một lỗi ở trường khác (vd test case vô tình gửi sai `kind`) cũng cho ra `VALIDATION_ERROR` và test vẫn xanh.

Ngoài 8 test case bắt buộc, lab chạy thêm 9 đại diện khác của cùng phân vùng (0; −5; 1.000.000; `null`; thiếu trường; `"sg"`; `"HCM"`; chuỗi 11 ký tự; số `79`) để kiểm chính **giả định** EP: cùng phân vùng thì cùng mã lỗi.

### 5.2. Test case mức component

| Mã | Đầu vào `calculateFare` | Mong đợi |
|---|---|---|
| EPC-01…05 | `bike` với 0,2 · −3 · 150 · `NaN` · `Infinity` km | ném `DISTANCE_OUT_OF_RANGE` |
| EPC-06 | `bike`, 7,4 km | `total` = 46.000 |
| EPO-01…03 | `bike` 0,5 km · `bike` 7,4 km · `car` 0,5 km | nâng tối thiểu: có (15.000) · không (46.000) · không (31.000) |
| EPO-04 | `car`, mọi d từ 0,5 tới 100,0 bước 0,1 | không giá trị nào bị nâng lên tối thiểu |

### 5.3. Code

Bảng test case và phân vùng đặt trong **một** tệp (`articles/equivalence-partitioning/ep-model.ts`) để vừa chạy test vừa đo coverage: hàm `epCoverage` đếm phân vùng có trong `covers` của ít nhất một ca — thêm test case mà quên khai `covers` thì số coverage lộ ngay. Trích:

```ts
export const API_CASES: ApiCase[] = [
  { id: 'EP-01', covers: ['D2', 'C1', 'V1'], body: { ...ride }, status: 200, total: 46_000 },
  { id: 'EP-03', covers: ['D1'], body: { ...ride, distanceKm: 0.2 }, status: 422, code: 'DISTANCE_OUT_OF_RANGE' },
  { id: 'EP-05', covers: ['D4'], body: { ...ride, distanceKm: '7.4' }, status: 400, code: 'VALIDATION_ERROR', field: 'distanceKm' },
  // … EP-02, EP-04, EP-06…08
];
```

Test qua API (`ep-api.test.ts`, rút gọn):

```ts
it.each(API_CASES)('$id phủ $covers ⇒ HTTP $status', async (c) => {
  const res = await quote(c.body).expect(c.status);
  if (c.total !== undefined) expect(res.body.quote.total).toBe(c.total);
  if (c.code !== undefined) expect(res.body.error.code).toBe(c.code);
  if (c.field !== undefined) {
    const paths = (res.body.error.details as { path: string }[]).map((d) => d.path);
    expect(paths).toEqual([c.field]); // đúng trường đang kiểm, không trường nào khác
  }
});
```

Chạy trong thư mục lab:

```bash
npx vitest run articles/equivalence-partitioning
```

Kết quả thực tế (vitest 3.2.7, Node 22, lần chạy khi viết bài; rút gọn):

```text
 ✓ ep-api.test.ts > Cùng phân vùng ⇒ cùng cách xử lý … > 'C2': 'cityCode = "sg"' ⇒ 'CITY_NOT_SUPPORTED'
 ✓ ep-api.test.ts > Defect masking … > EP-X thành phố DL + quãng đường 0,2 km ⇒ chỉ thấy CITY_NOT_SUPPORTED (R-VALID-01), D1 KHÔNG được kiểm
 ✓ ep-component.test.ts > Phân vùng đầu ra … > EPO-04 với ô tô, phân vùng «bị nâng lên tối thiểu» là RỖNG trên toàn miền hợp lệ
 ✓ ep-gioi-han.test.ts > … > Lỗi B «kiểm trước khi làm tròn»: EP BỎ LỌT — lỗi nằm sát biên

 Test Files  4 passed (4)
      Tests  34 passed (34)
```

### 5.4. Đo EP coverage

| Bộ ca | Phân vùng được chạm | EP coverage | Thiếu |
|---|---|---|---|
| Each Choice EP-01…08 | 10/10 | **100%** | — |
| «Thường gặp»: một ca đúng + hai ca sai quãng đường (EP-01, 03, 04) | 5/10 | **50%** | D4, C2, C3, V2, V3 |
| Một ca gộp `DL` + 0,2 km (khai trung thực) | 1/10 | 10% | khai «phủ cả C2 và D1» là thổi phồng thành 20% |

Bộ «thường gặp» — một test case happy path và vài giá trị sai của trường «quan trọng nhất» — không có test case nào cho ô tô, thành phố chưa mở hay kiểu sai.

### 5.5. EP bắt được gì, bỏ lọt gì

Lab chạy cùng bảng đại diện EP (0,2 · 7,4 · 150 · NaN) trên bản đúng và hai bản `normalizeDistanceKm` **có lỗi cố ý** (`quang-duong-loi.ts`):

| Bản cài đặt | Lỗi | Số ca EP đỏ | Kết luận |
|---|---|---|---|
| `src/domain/fare.ts` | — | 0 | |
| Lỗi A «quên cận trên» | bỏ điều kiện `km > 100` | 1 (D3 · 150 km được chấp nhận) | EP **bắt** được — cả một phân vùng bị xử lý sai |
| Lỗi B «kiểm trước khi làm tròn» | so `raw < 0.5 \|\| raw > 100` rồi mới làm tròn | 0 | EP **bỏ lọt** — chỉ 0,45…0,49 và 100,01…100,04 bị xử lý sai |

Lỗi B từ chối 0,45 km dù R-FARE-02 ghi rõ «0,45 hợp lệ», và không đại diện «ở giữa» nào chạm được dải hẹp đó. Đây là **giới hạn của kỹ thuật**: EP chọn *phân vùng*, không chọn *biên* — việc của BVA ở bài sau.

## 6. Góc nhìn nâng cao

**Phân vùng phụ thuộc test condition, không chỉ phụ thuộc dữ liệu.** Với phép tính cước, 6 thành phố là một phân vùng. Nhưng khi kèm voucher `HN10` (chỉ áp cho `HN`, R-VOUCHER-05), `HN` tách khỏi 5 mã còn lại; khi trời mưa ở `SG` thì `SG` khác `HN`. Một mô hình phân vùng đúng cho câu hỏi này có thể sai cho câu hỏi khác — đừng tái dùng bảng phân vùng mà không xem lại.

**Phân vùng ẩn trong cài đặt.** Code có thể xử lý một dải giá trị khác đi mà đặc tả không nói (vd cache riêng cho quãng đường dưới 1 km). EP không tự phát hiện được; cần white-box (Series 05) hoặc kinh nghiệm ([TEST-051](/articles/error-guessing)).

**Sai lầm thường gặp:**

| Sai lầm | Vì sao sai |
|---|---|
| Chỉ phân vùng hợp lệ | 100% coverage của EP tính cả phân vùng không hợp lệ (mục 4.2.1) |
| Gộp nhiều giá trị sai vào một ca | Defect masking; ca chỉ được tính phủ phân vùng gây lỗi đầu tiên |
| Tách phân vùng theo «cảm giác» (1–10 km, 10–50 km, 50–100 km) | Không có quy tắc nào xử lý chúng khác nhau ⇒ ba ca làm việc của một |
| Lấy biên làm đại diện EP | Ca đỏ thì không biết lỗi do cả phân vùng hay do biên — trộn hai kỹ thuật |
| Chỉ kiểm mã HTTP của lỗi định dạng | Lỗi ở trường khác cũng ra 400; phải kiểm `details` |

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Khối lượng bưu kiện

Với `POST /api/quotes` dịch vụ `parcel`, phân vùng tham số `parcelWeightKg` theo R-PARCEL-01, R-PARCEL-02 và R-PARCEL-03 (đầu vào **và** đầu ra phụ phí), rồi chọn bộ test case Each Choice nhỏ nhất kết hợp với loại xe (`bike`, `car`). Giữ thành phố `SG`, 3 km, 09:00.

<details><summary><b>Lời giải mẫu</b></summary><ul><li><b>Phân vùng đầu vào</b> (theo giá trị sau làm tròn 0,1 kg): W1 ≤ 0 — không hợp lệ, <code>WEIGHT_OUT_OF_RANGE</code>; W2 (0; 20] — hợp lệ với cả hai loại xe; W3 (20; 30] — hợp lệ chỉ với ô tô; W4 &gt; 30 — không hợp lệ; W5 không phải số — 400 <code>VALIDATION_ERROR</code>.</li><li><b>Phân vùng đầu ra phụ phí:</b> S1 w ≤ 5 ⇒ 0 đ; S2 w &gt; 5 ⇒ 2.000 × ⌈w − 5⌉. S1 nằm trong W2, S2 cắt cả W2 và W3.</li><li><b>Tổ hợp bị cấm:</b> W3 × <code>bike</code> ⇒ <code>VEHICLE_TOO_SMALL</code> — một phân vùng không hợp lệ <i>của tổ hợp</i>, cần một ca riêng.</li><li><b>Bộ ca</b> (6): <code>bike</code> 3 kg (W2, S1); <code>car</code> 25 kg (W3, S2 ⇒ 40.000 đ phụ phí); <code>bike</code> 25 kg (W3 × bike ⇒ 422); <code>car</code> −1 kg (W1); <code>car</code> 40 kg (W4); <code>car</code> <code>"5"</code> (W5).</li></ul></details>

### Bài tập 2 — Xếp vào phân vùng nào?

Xếp các giá trị `cityCode` sau vào C1, C2, C3 theo **cài đặt hiện tại**, rồi đánh dấu giá trị nào cần hỏi BA: `"NT"`, `"sg"`, `" SG"`, `"Sài Gòn"`, `"SAIGONCITY1"`, `null`.

<details><summary><b>Lời giải</b></summary><ul><li><code>"NT"</code> — C1 (Nha Trang, R-GEN-03).</li><li><code>"sg"</code>, <code>" SG"</code> — hiện là C2 (so khớp chính xác). <b>Cần hỏi BA</b>: voucher được chuẩn hoá (R-VOUCHER-02), thành phố thì SPEC im lặng — ứng dụng cũ gửi chữ thường sẽ bị từ chối.</li><li><code>"Sài Gòn"</code> — 7 ký tự ⇒ C2 (định dạng đúng, mã không được phục vụ).</li><li><code>"SAIGONCITY1"</code> — 11 ký tự ⇒ C3 (vượt 10 ký tự).</li><li><code>null</code> — C3 (không phải chuỗi).</li></ul></details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Giả định nền tảng của Equivalence Partitioning là gì?

- a) Mọi giá trị đầu vào đều có xác suất gây lỗi như nhau
- b) Các phần tử trong cùng phân vùng được kỳ vọng xử lý như nhau, nên một ca cho mỗi phân vùng là đủ
- c) Lỗi thường nằm ở biên của khoảng giá trị
- d) Phải thử mọi tổ hợp phân vùng của các tham số

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — EP không giả định gì về xác suất lỗi.</li><li>b) Đúng — mục 4.2.1.</li><li>c) Sai — đó là lập luận của BVA.</li><li>d) Sai — Each Choice không xét tổ hợp; tổ hợp là việc của combinatorial testing.</li></ul></details>

**Câu 2 (K3).** Mô hình có 10 phân vùng như mục 4.2. Bộ test case gồm: `SG`/`bike`/7,4 km (hợp lệ); `SG`/`bike`/0,2 km; `SG`/`bike`/`"abc"`. Với test case lỗi chỉ tính phân vùng không hợp lệ của nó. EP coverage là bao nhiêu?

- a) 30%
- b) 50%
- c) 60%
- d) 100%

<details><summary><b>Đáp án: b</b></summary><ul><li>Ca 1 chạm D2, C1, V1; ca 2 chạm D1; ca 3 chạm D4 ⇒ 5/10 = 50%.</li><li>a) Sai — bỏ sót C1, V1 của ca hợp lệ. c) Sai — tính thêm cho ca lỗi. d) Sai — thiếu D3, C2, C3, V2, V3.</li></ul></details>

**Câu 3 (K3).** Test item có ba tham số: A có 3 phân vùng hợp lệ và 1 không hợp lệ; B có 2 hợp lệ và 2 không hợp lệ; C có 2 hợp lệ, không có phân vùng không hợp lệ. Mỗi phân vùng không hợp lệ được kiểm riêng. Số test case tối thiểu để đạt 100% Each Choice?

- a) 4
- b) 6
- c) 9
- d) 12

<details><summary><b>Đáp án: b</b></summary><ul><li>Ca hợp lệ: bằng số phân vùng hợp lệ của tập lớn nhất = 3. Ca không hợp lệ: 1 + 2 = 3. Tổng 6.</li><li>a) Sai — gộp phân vùng không hợp lệ (masking). c) Sai — không ghép phân vùng hợp lệ. d) Sai — 3 × 2 × 2 là mọi tổ hợp.</li></ul></details>

**Câu 4 (K2).** Test case gửi `cityCode = "DL"` và `distanceKm = 0.2` cùng lúc, nhận `CITY_NOT_SUPPORTED`. Nhận định nào đúng?

- a) Ca phủ cả phân vùng thành phố không hỗ trợ lẫn quãng đường quá ngắn
- b) Ca chỉ thật sự kiểm phân vùng thành phố; việc xử lý quãng đường 0,2 km bị che (defect masking)
- c) Ca không hợp lệ vì một ca chỉ được có một tham số
- d) Ca chứng minh thứ tự kiểm tra sai

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — theo R-VALID-01, hệ thống dừng ở lỗi đầu, không chạy tới kiểm quãng đường.</li><li>b) Đúng — vì thế phân vùng không hợp lệ được kiểm riêng.</li><li>c) Sai — một ca có nhiều tham số; chỉ là không nên có hai giá trị sai.</li><li>d) Sai — kết quả đúng R-VALID-01.</li></ul></details>

**Câu 5 (K2).** Phát biểu nào **đúng** theo CTFL v4.0.1 mục 4.2.1?

- a) Phân vùng có thể chồng lấn nếu dùng cho đầu ra
- b) Chỉ phân vùng được đầu vào của giao diện người dùng
- c) Định nghĩa giá trị hợp lệ và không hợp lệ có thể khác nhau giữa các đội và tổ chức
- d) Phân vùng rỗng được tính vào mẫu số của EP coverage

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Sai — phân vùng không được chồng lấn, ở mọi loại phần tử dữ liệu.</li><li>b) Sai — áp cho đầu vào, đầu ra, cấu hình, giá trị nội bộ, thời gian, tham số giao diện.</li><li>c) Đúng — syllabus nêu rõ.</li><li>d) Sai — phân vùng phải không rỗng; phân vùng rỗng không tồn tại để tính.</li></ul></details>

**Câu 6 (K3).** Với `calculateFare` của xe máy, không cao điểm, không mưa, không voucher, đại diện nào **phủ phân vùng đầu ra «bị nâng lên cước tối thiểu»**?

- a) 0,4 km
- b) 0,6 km
- c) 0,7 km
- d) 7,4 km

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — 0,4 km thuộc phân vùng đầu vào không hợp lệ, bị từ chối trước khi tính cước.</li><li>b) Đúng — 12.000 + 4.500 × 0,6 = 14.700 &lt; 15.000 ⇒ 15.000 đ.</li><li>c) Sai — 15.150 đ ≥ 15.000, không bị nâng.</li><li>d) Sai — 45.300 đ.</li></ul></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 4.1, 4.2.1 (FL-4.2.1) và ghi chú phát hành v4.0.1 — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level Test Analyst Syllabus* v4.0: mục 3.1, 3.1.1 (TA-3.1.1) — https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (mục *equivalence partitioning*, *equivalence partition*, *coverage item*, *defect masking*).
- ISO/IEC/IEEE 29119-4, *Software and systems engineering — Software testing — Part 4: Test techniques* (equivalence partitioning).
- P. Ammann, J. Offutt, *Introduction to Software Testing*, 2nd ed., 2016 — nguồn của tiêu chí Each Choice mà CTFL dẫn.
- Lab Tới Nơi: `SPEC.md` (R-FARE-02, R-GEN-03, R-GEN-04, R-API-02, R-VALID-01) và `articles/equivalence-partitioning/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-003 Bảy nguyên lý kiểm thử](/articles/bay-nguyen-ly-kiem-thu) · [TEST-010 Testware](/articles/testware-san-pham-kiem-thu) · [TEST-017 Functional Testing](/articles/functional-testing) · [TEST-030 Shift-left Testing](/articles/shift-left-testing) · [TEST-040 Domain testing](/articles/data-based-test-techniques-domain-testing) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)

Bài tiếp theo: [TEST-032 · Boundary Value Analysis](/articles/boundary-value-analysis)
