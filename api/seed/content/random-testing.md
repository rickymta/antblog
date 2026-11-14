---
slug: random-testing
title: "Random Testing: sinh 10.000 yêu cầu báo giá ngẫu nhiên và bài toán oracle"
summary: "Random testing theo CTAL-TA v4.0: chọn phân phối, viết oracle tự động, giới hạn của kỹ thuật. Thực hành sinh 10.000 yêu cầu báo giá Tới Nơi bằng fast-check, tìm và thu nhỏ một lỗi cước tối thiểu."
category: testing
level: advanced
tags: [testing, istqb, ctal-ta, test-041, random-testing, property-based-testing, test-oracle, fast-check, toi-noi]
language: vi
---

Lõi định giá của Tới Nơi nhận ba dịch vụ, hai loại xe, sáu thành phố, quãng đường và khối lượng là số thực, giờ cao điểm, trời mưa, voucher. Không ai liệt kê hết tổ hợp được. Bài này dùng **random testing**: để máy chọn ngẫu nhiên hàng nghìn đầu vào theo một phân phối có chủ đích, rồi để một **oracle tự động** phán đúng/sai. Câu hỏi khó không nằm ở bước «sinh», mà ở hai chỗ: sinh theo phân phối nào, và lấy gì làm kết quả mong đợi cho một đầu vào chưa ai tính tay.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-041</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S04 · Black-box Test Design Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">Test Level</th><td>Component</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Random Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/equivalence-partitioning">TEST-031 · Equivalence Partitioning</a>; <a href="/articles/data-based-test-techniques-domain-testing">TEST-040 · Data-based test techniques: domain testing nhiều biến</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): sinh 10.000 yêu cầu báo giá ngẫu nhiên bằng fast-check, oracle «bội số 1.000 đ, không dưới cước tối thiểu, không NaN» (R-FARE-07, R-FARE-08) trên <code>src/domain/pricing.ts</code>; thu nhỏ phản ví dụ của một bản có lỗi — <code>examples/toi-noi-testing/articles/random-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TA v4.0 · Chương 3 · mục 3.1.3 · TA-3.1.3 (K2); liên quan mục 1.3.4 · TA-1.3.4 (K2) — test oracle problem</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

[Equivalence Partitioning](/articles/equivalence-partitioning), [BVA](/articles/boundary-value-analysis) và [domain testing](/articles/data-based-test-techniques-domain-testing) chọn **ít** giá trị bằng **lập luận**: chia miền, lấy đại diện, lấy biên. Chúng mạnh khi đặc tả rõ, nhưng chỉ tìm được lỗi ở chỗ người thiết kế nghĩ tới.

Random testing chọn **nhiều** giá trị bằng **xác suất**, không cần biết lỗi ở đâu nên tránh được thiên kiến «chỗ này chắc đúng rồi». Đổi lại, nó cần hai thứ:

- một **phân phối xác suất** mô tả «ngẫu nhiên» nghĩa là gì;
- một **oracle tự động**, vì không ai tính tay kết quả mong đợi cho 10.000 đầu vào.

Nên dùng khi miền đầu vào lớn hoặc liên tục, cần nhiều dữ liệu test, hiểu biết về miền còn hạn chế, hoặc làm lưới an toàn **bổ sung** — không thay — các kỹ thuật lập luận.

## 2. Lý thuyết theo ISTQB

### 2.1 Vị trí trong CTAL-TA v4.0

CTFL v4.0.1 có hai kỹ thuật thuộc nhóm dữ liệu: EP và BVA. CTAL-TA v4.0 (mục 3.1) gọi cả nhóm là **data-based test techniques** và thêm ba kỹ thuật: domain testing, combinatorial testing và **random testing**. Syllabus giới hạn phạm vi: chỉ bàn **dữ liệu đầu vào ngẫu nhiên**, không bàn chuỗi sự kiện ngẫu nhiên.

Diễn đạt lại định nghĩa của mục 3.1.3: random testing chọn dữ liệu test **ngẫu nhiên từ miền đầu vào** của test item, theo **một phân phối xác suất được đặc tả** trước. LO tương ứng là TA-3.1.3 (K2): tóm tắt được lợi ích và giới hạn của random testing.

### 2.2 Bốn quyết định khi làm random testing

| Quyết định | Syllabus nói gì (diễn đạt lại) | Hệ quả thực hành |
|---|---|---|
| Phân phối | Mục đích **validation** ⇒ phân phối theo **operational profile** (tỉ lệ dùng thật). Mục đích **verification** ⇒ phân phối **không theo thói quen dùng** (usage-agnostic) để tránh thiên kiến | Hai bộ sinh khác nhau cho hai câu hỏi khác nhau |
| Oracle | Kết quả mong đợi gắn vào test bằng test oracle, thường phải là **oracle tự động** | Viết oracle trước khi viết bộ sinh |
| Guided / unguided | **Unguided**: phân phối cố định suốt quá trình. **Guided**: phân phối thay đổi theo các giá trị đã chọn, vd **adaptive random testing** — trải đầu vào đều trên miền vì defect hay tụ thành cụm | Công cụ phổ biến chỉ làm unguided; guided cần công cụ hoặc code riêng |
| Tiêu chí dừng | Random testing **không có tiêu chí coverage được công nhận** ⇒ exit criteria chỉ dựa vào số test đã chạy, thời gian, hoặc thước đo tương tự | Ghi rõ «10.000 ca/seed, ≤ 1 giây» thay vì «đạt 100%» |

### 2.3 Lợi ích và giới hạn (TA-3.1.3)

| Lợi ích | Giới hạn |
|---|---|
| Đặc biệt có giá trị khi hiểu biết về miền còn hạn chế hoặc cần rất nhiều dữ liệu test | Bỏ qua **ngữ nghĩa** của dữ liệu ⇒ có thể bỏ lót defect gắn với ý nghĩa của dữ liệu |
| Rẻ: sinh và chạy tự động | Có thể bỏ sót một số defect (vùng lỗi quá nhỏ so với miền) |
| Cho cái nhìn **theo xác suất** về độ tin cậy của test object | Sinh nhiều test **thừa** (trùng hoặc cùng phân vùng) |
| Tránh thiên kiến, vd bỏ qua defect vì quá tin một đoạn code | **Phụ thuộc vào oracle tự động** |
| | Đầu ra ngẫu nhiên ⇒ kết quả test **không nhất quán** giữa các lần chạy |

Syllabus còn ghi nhận: random testing từng bị coi là kém hiệu quả, nhưng nghiên cứu thực nghiệm cho thấy trong các điều kiện trên nó có thể hiệu quả hơn các kỹ thuật data-based khác. Nó cũng được dùng trong **fuzz testing** và **chaos engineering**.

### 2.4 Test oracle problem (TA-1.3.4)

Mục 1.3.4 của CTAL-TA gọi tình huống «không có oracle đủ rẻ» là **test oracle problem**. Nguyên nhân điển hình: dữ liệu phức tạp, hệ không tất định, hành vi xác suất, yêu cầu thiếu hoặc mơ hồ. Các lời giải được nêu:

| Lời giải | Ý chính | Trong bài |
|---|---|---|
| Pseudo-oracle | Hệ độc lập cùng đặc tả (hệ cũ, bản giản lược) | oracle mã lỗi tính bằng mét/gam |
| Model-based testing | Oracle nằm trong mô hình test | — |
| **Property-based testing** | Kiểm **quan hệ** đầu vào – kết quả; hợp tự động hoá, hiệu quả phụ thuộc quan hệ chọn được | O1–O6 |
| Metamorphic testing | Quan hệ giữa **nhiều lần chạy** | [TEST-042](/articles/metamorphic-testing) |
| Human oracle | Người phán đoán; đắt, khan hiếm | ca R-FARE-09 |

### 2.5 Thuật ngữ

| Tiếng Anh | Tiếng Việt | Ghi chú |
|---|---|---|
| random testing | kiểm thử ngẫu nhiên | CTAL-TA 3.1.3 |
| probability distribution | phân phối xác suất | phải đặc tả trước |
| operational profile | hồ sơ vận hành (tỉ lệ dùng thật) | cho validation |
| guided / unguided random testing | ngẫu nhiên có / không định hướng | ART là một dạng guided |
| test oracle / automated test oracle | nguồn kết quả mong đợi / oracle tự động | |
| test oracle problem | bài toán oracle | CTAL-TA 1.3.4 |
| property-based testing | kiểm thử dựa trên tính chất | lời giải oracle |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB — property-based testing trong thực hành</span>
<p>Syllabus nêu property-based testing như một lời giải oracle. Trong ngành, từ này còn chỉ một <b>họ công cụ</b>, bắt đầu từ QuickCheck (Haskell, Claessen &amp; Hughes, 2000); ở TypeScript là <b>fast-check</b>. Ba khái niệm của công cụ, không có trong syllabus:</p>
<ul>
<li><b>Arbitrary</b> (bộ sinh): mô tả cách sinh một kiểu giá trị, ghép được với nhau.</li>
<li><b>Shrinking</b> (thu nhỏ): khi một ca thất bại, công cụ thử các giá trị «nhỏ hơn» để tìm phản ví dụ đơn giản nhất vẫn thất bại.</li>
<li><b>Seed</b>: giá trị khởi đầu của bộ sinh số giả ngẫu nhiên; cùng seed ⇒ cùng chuỗi đầu vào. Đây là cách khử giới hạn «kết quả không nhất quán».</li>
</ul>
<p>fast-check <b>không sinh đều tuyệt đối</b>: nó cố ý thiên về giá trị nhỏ và mép khoảng — định hướng bằng kinh nghiệm, khác adaptive random testing.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

**Yêu cầu nghiệp vụ.** Khách gửi yêu cầu báo giá; Pricing & Quote trả tổng tiền kèm bảng chi tiết (R-FARE-10). Giá sai theo cả hai chiều đều đắt: thu thừa sinh khiếu nại và hoàn tiền; thu thiếu là mất doanh thu trên mỗi chuyến, nhân lên 2 triệu chuyến mỗi ngày (hồ sơ dự án).

**Business rules** được dùng làm oracle (trích SPEC lab):

| Mã | Quy tắc | Thành tính chất kiểm được |
|---|---|---|
| R-GEN-01 | Tiền khách trả luôn là số nguyên đồng | O1 |
| R-FARE-03 | Thứ tự cố định: cơ bản → cao điểm → mưa → voucher → tối thiểu → làm tròn | O3, O4 |
| R-FARE-07 | Voucher không bao giờ kéo cước xuống dưới mức tối thiểu (xe máy 15.000 đ, ô tô 30.000 đ) | O3 |
| R-FARE-08 | Làm tròn **lên** bội số 1.000 đ | O2, O4 |
| R-VOUCHER-06 | Tiền giảm = min(percent% × A, trần) | O5 |
| R-FOOD-01, R-FOOD-04 | Đồ ăn: tổng = tiền món − giảm + phí giao; chỉ phí giao được làm tròn | O6 |
| R-VALID-01 | Nhiều lỗi ⇒ báo lỗi đầu tiên theo thứ tự cố định | oracle mã lỗi |

**System context.** Trong kiến trúc đích, Booking gọi Pricing qua gRPC, API Gateway đưa `POST /api/quotes` từ app khách tới Pricing (kiến trúc §4.3, §7.1); kiểm thử quy tắc giá là việc của tầng unit (§17.1). Lab (MVP modular monolith): lõi định giá là hai hàm thuần `normalizePriceRequest`, `priceRequest` trong `src/domain/pricing.ts` — không I/O, 10.000 lần chạy dưới một giây.

**Rủi ro:** tổng `NaN` hiện «NaN đ» trên app; bước tính bị đảo thứ tự khi refactor; voucher «xuyên» cước tối thiểu; làm tròn sai một bậc 1.000 đ; mã lỗi sai thứ tự.

## 4. Phân tích kỹ thuật

### 4.1 Chọn test item: ranh giới hợp đồng

Câu hỏi đầu tiên không phải «sinh gì» mà «sinh vào **đâu**». Nếu sinh JSON thô vào `POST /api/quotes`, gần như mọi ca bị zod chặn ở `VALIDATION_ERROR` (R-API-07) — đúng giới hạn «test thừa, bỏ qua ngữ nghĩa». Bài chọn test item là **lõi định giá sau lớp kiểm định dạng**: kiểu dữ liệu đã đúng (số là số, món ăn có đơn giá bội số 1.000 đ như R-FOOD-02), còn **giá trị** thì ngẫu nhiên trên toàn miền, kể cả vùng không hợp lệ về nghiệp vụ.

Sinh ngoài hợp đồng cho ra «failure giả»: đơn giá `NaN` truyền thẳng vào `priceFood` ra tổng `NaN`, nhưng hàm chỉ nhận dữ liệu đã qua zod — test sai, không phải lỗi sản phẩm.

### 4.2 Chọn phân phối

| Thành phần | Bộ sinh `unguidedCase` (verification) | Bộ sinh `profileCase` (validation) |
|---|---|---|
| Dịch vụ | đều 1/3 | ride 40% · food 45% · parcel 15% — từ hồ sơ dự án (0,8 + 0,9 + 0,3 triệu chuyến/ngày) |
| Giờ cao điểm | 50% | 50% — hồ sơ dự án: hai khung cao điểm chiếm 50% chuyến |
| Thành phố | đều trên 6 mã + `DL` (không phục vụ) | đều trên 6 mã |
| Quãng đường | số nguyên mét 0–120.000, chia 1.000 | giả định: 70% 1–8 km, 25% 8–25 km, 5% 25–100 km |
| Khối lượng | 0–35.000 g | giả định: 80% ≤ 5 kg |
| Mưa | 50% | giả định 20% |
| Voucher | 50% có, percent 1–50, trần 1.000–100.000 đ, tối thiểu 0–200.000 đ | giả định 30% dùng `GIAM20` (đồ ăn dùng `FOOD15`) |

Quãng đường và khối lượng được sinh bằng **số nguyên mét/gam**. Lý do: oracle mã lỗi so biên bằng số nguyên (0,45 km = 450 m) mà không gọi hàm làm tròn của chính sản phẩm — nếu gọi, oracle sẽ «đồng ý» với mọi lỗi làm tròn.

### 4.3 Thiết kế oracle: kiểm tính chất, không tính lại

Cám dỗ lớn nhất là **tính lại cước** trong oracle — tức chép lại code sản phẩm. Oracle của bài chỉ kiểm điều SPEC bắt buộc với **mọi** đầu vào — một partial oracle:

| Mã | Tính chất | Bắt được loại lỗi |
|---|---|---|
| O1 | Không NaN/Infinity, tổng nguyên | phép chia cho 0, `undefined` lọt vào công thức |
| O2 | Tổng cước chia hết 1.000 | quên làm tròn, làm tròn sai đơn vị |
| O3 | Tổng ≥ cước tối thiểu | đảo thứ tự voucher/tối thiểu |
| O4 | Tổng − max(sau giảm, tối thiểu) ∈ [0; 1.000) | làm tròn xuống, làm tròn hai lần |
| O5 | 0 ≤ giảm ≤ trần | bỏ trần voucher |
| O6 | Đồ ăn: tổng = món − giảm + phí giao | voucher trừ nhầm vào phí giao |
| Mã lỗi | Lỗi đầu tiên đúng R-VALID-01, tính lại bằng mét/gam | sai biên, sai thứ tự kiểm |

Một chi tiết: tính chất «tổng chia hết 1.000 đ» **không** áp cho tổng đơn đồ ăn. SPEC chỉ làm tròn phí giao; tiền giảm 15% của 35.000 đ là 5.250 đ nên tổng có thể lẻ. Đây là lỗ hổng đặc tả đã ghi nhận ở [TEST-024](/articles/walkthrough), không phải phát hiện mới. Oracle phải theo đặc tả, không theo cảm giác «tiền thì phải tròn».

### 4.4 Tiêu chí dừng

Không có coverage để đo ⇒ chốt **10.000 ca mỗi seed, dưới 1 giây**; seed cố định (41) trong CI để tái lập; đổi seed ở lần chạy ban đêm và **ghi seed vào log**.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 250" role="img" aria-label="Vòng lặp random testing: bộ sinh theo phân phối và seed tạo một ca, chạy qua lõi định giá, oracle kiểm O1 đến O6 và mã lỗi. Đạt thì quay lại sinh ca tiếp cho tới đủ 10.000 ca. Không đạt thì thu nhỏ phản ví dụ, in seed và path, rồi chép phản ví dụ thành test hồi quy cố định.">
<defs>
<marker id="t041-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-b">Random testing với oracle tự động và thu nhỏ phản ví dụ</text>
<rect x="20" y="60" width="180" height="76" rx="6" class="f-pk-soft s-pk"/>
<text x="110" y="82" text-anchor="middle" class="t-b">Bộ sinh</text>
<text x="110" y="102" text-anchor="middle" class="t-10">phân phối + seed 41</text>
<text x="110" y="120" text-anchor="middle" class="t-mono t-10">unguidedCase</text>
<rect x="250" y="60" width="190" height="76" rx="6" class="f-surface s-pk"/>
<text x="345" y="82" text-anchor="middle" class="t-b">Test item</text>
<text x="345" y="102" text-anchor="middle" class="t-mono t-10">normalizePriceRequest</text>
<text x="345" y="120" text-anchor="middle" class="t-mono t-10">priceRequest</text>
<rect x="490" y="60" width="170" height="76" rx="6" class="f-phys-soft s-phys"/>
<text x="575" y="82" text-anchor="middle" class="t-b">Oracle tự động</text>
<text x="575" y="102" text-anchor="middle" class="t-10">O1–O6 (tính chất)</text>
<text x="575" y="120" text-anchor="middle" class="t-10">mã lỗi R-VALID-01</text>
<rect x="720" y="60" width="180" height="76" rx="6" class="f-surface s-muted"/>
<text x="810" y="88" text-anchor="middle" class="t-b">Đủ 10.000 ca?</text>
<text x="810" y="110" text-anchor="middle" class="t-10">exit criteria = số ca</text>
<line x1="200" y1="98" x2="248" y2="98" class="s-pk" stroke-width="1.5" marker-end="url(#t041-arrow)"/>
<line x1="440" y1="98" x2="488" y2="98" class="s-pk" stroke-width="1.5" marker-end="url(#t041-arrow)"/>
<line x1="660" y1="98" x2="718" y2="98" class="s-pk" stroke-width="1.5" marker-end="url(#t041-arrow)"/>
<text x="690" y="90" text-anchor="middle" class="t-10">đạt</text>
<path d="M810,60 L810,40 L110,40 L110,58" fill="none" class="s-pk" stroke-width="1.5" marker-end="url(#t041-arrow)"/>
<text x="460" y="35" text-anchor="middle" class="t-10">chưa đủ ⇒ sinh ca tiếp</text>
<rect x="430" y="180" width="200" height="56" rx="6" class="f-warn-soft s-warn"/>
<text x="530" y="202" text-anchor="middle" class="t-b t-warn">Shrinking</text>
<text x="530" y="222" text-anchor="middle" class="t-10">thử giá trị nhỏ hơn, giữ ca còn đỏ</text>
<rect x="680" y="180" width="220" height="56" rx="6" class="f-warn-soft s-warn"/>
<text x="790" y="202" text-anchor="middle" class="t-b">Phản ví dụ + seed + path</text>
<text x="790" y="222" text-anchor="middle" class="t-10">⇒ chép thành test hồi quy cố định</text>
<line x1="575" y1="136" x2="545" y2="178" class="s-warn" stroke-width="1.5" marker-end="url(#t041-arrow)"/>
<text x="600" y="164" class="t-10 t-warn">vi phạm</text>
<line x1="630" y1="208" x2="678" y2="208" class="s-warn" stroke-width="1.5" marker-end="url(#t041-arrow)"/>
</svg>
</div>
<figcaption><b>Hình 1 · Một vòng random testing.</b> «Random» chỉ là ô đầu; oracle, tiêu chí dừng và lưu phản ví dụ mới quyết định giá trị của bộ test.</figcaption>
</figure>

## 5. Thực hành

Mã nguồn: `examples/toi-noi-testing/articles/random-testing/` (fast-check 4.10, vitest 3.2). Không sửa `src/`.

| Tệp | Nội dung |
|---|---|
| `generators.ts` | `unguidedCase`, `uniformCase(maxMeters)`, `profileCase` |
| `oracle.ts` | `judge(priceFn, ca)` ⇒ danh sách vi phạm; `srcPrice` = lõi định giá thật |
| `buggy-pricing.ts` | hai bản lỗi: `minimum-before-voucher` (đảo bước 4 và 5 của R-FARE-03), `rain-times-peak` (cộng mưa trước khi nhân 1,25) |
| `random-pricing.test.ts` | RT-01…03 trên sản phẩm thật |
| `buggy-pricing.test.ts` | BG-01…04 trên hai bản lỗi |

Lõi của một property chỉ vài dòng — phần lớn công sức nằm ở `generators.ts` và `oracle.ts`:

```ts
fc.assert(
  fc.property(unguidedCase, (c: QuoteCase) => {
    const { outcome, violations } = judge(srcPrice, c);   // chạy test item + oracle
    tally(outcomes, outcome.kind === 'ok' ? `OK ${c.req.kind}` : outcome.code);
    return violations.length === 0;                       // false ⇒ fast-check thu nhỏ rồi báo đỏ
  }),
  { seed: 41, numRuns: 10_000 },
);
```

```bash
cd examples/toi-noi-testing
npx vitest run articles/random-testing                        # 7 passed, 1 skipped
SHOW_RED=1 npx vitest run articles/random-testing -t SHOW_RED # báo cáo gốc của fast-check (đỏ cố ý)
```

### 5.1 RT-01 — phân phối cố định trên sản phẩm thật

```text
RT-01 · 10000 ca, seed 41, 396 ms — kết cục:
  DISTANCE_OUT_OF_RANGE        2912   29.1%
  OK food                      1890   18.9%
  OK ride                      1644   16.4%
  CITY_NOT_SUPPORTED           1422   14.2%
  OK parcel                    1002   10.0%
  WEIGHT_OUT_OF_RANGE           581    5.8%
  VOUCHER_MIN_AMOUNT            300    3.0%
  VEHICLE_TOO_SMALL             212    2.1%
  SUBTOTAL_TOO_LOW               37    0.4%
  đơn đồ ăn hợp lệ có tổng KHÔNG chia hết 1.000 đ: 331 (đúng SPEC: chỉ phí giao được làm tròn)
```

**Mong đợi:** không vi phạm. **Thực tế:** 10.000/10.000 đạt. Bảng cho thấy giới hạn «test thừa»: 29,1% ca rơi vào quãng đường ngoài khoảng (miền sinh 0–120 km, hợp lệ tới 100 km), lặp lại cùng một phân vùng; chỉ 45,3% ca tới công thức cước — cái giá của phân phối «không thiên vị».

### 5.2 RT-02 — operational profile

```text
RT-02 · 10000 ca theo profile — phân bố:
  food cao điểm                2318   23.2%
  food                         2192   21.9%
  ride cao điểm                2033   20.3%
  ride                         1960   19.6%
  parcel cao điểm               756    7.6%
  parcel                        741    7.4%
  lỗi SUBTOTAL_TOO_LOW          109    1.1%
  lỗi VOUCHER_MIN_AMOUNT         17    0.2%
  failure: 0 ⇒ cận trên xác suất failure mỗi yêu cầu ≈ 0.03% (theo profile này)
```

Phân bố khớp tỉ lệ dự kiến (food 45,1%, ride 39,9%, parcel 15,0%). 0 failure trong 10.000 ca độc lập ⇒ theo «quy tắc ba» của thống kê (ngoài syllabus), xác suất failure mỗi yêu cầu **theo profile này** dưới ~3/10.000 (độ tin cậy ~95%) — đó là «cái nhìn theo xác suất về độ tin cậy». Kết luận chỉ đúng khi profile giống thật; profile có giả định ⇒ chỉ là minh hoạ phương pháp.

### 5.3 RT-03 — tái lập bằng seed

```text
RT-03 · 3 ca đầu với seed 41:
  {"kind":"ride","cityCode":"SG","vehicle":"car","distanceKm":45.782,"peak":true,"raining":false,"voucher":15}
  {"kind":"parcel","cityCode":"SG","vehicle":"bike","distanceKm":119.991,"parcelWeightKg":0.004,"peak":false,"raining":false,"voucher":4}
  {"kind":"ride","cityCode":"CT","vehicle":"bike","distanceKm":119.991,"peak":false,"raining":true,"voucher":48}
```

Cùng seed ⇒ cùng chuỗi. 119,991 km xuất hiện hai lần trong ba ca, 0,004 kg sát biên dưới: fast-check cố ý thiên về mép khoảng.

### 5.4 BG-01 — tìm và thu nhỏ một lỗi thật sự

Bản `minimum-before-voucher` nâng lên cước tối thiểu **trước** khi trừ voucher. Báo cáo gốc của fast-check (`SHOW_RED=1`):

```text
Property failed after 557 tests
{ seed: 41, path: "556:0:4:1:1:1:1:1:3:3:3:3:5:6:5:4:4:5:4:5:0:0:0:0:0:1:2:3:0:0:1:2:2:1:2:2:2", endOnFailure: true }
Counterexample: [{"meters":450,"peak":false,"raining":false,"voucher":{"code":"RND","percent":8,"maxDiscount":1000,"minAmount":0},"grams":null,"req":{"kind":"ride","cityCode":"SG","vehicle":"bike","distanceKm":0.45}}]
Shrunk 36 time(s)
```

và phần chẩn đoán của test BG-01:

```text
  vi phạm   : O3 cước: tổng 14000 < tối thiểu 15000; O4 cước: tổng − max(sau giảm, tối thiểu) = -1000
  bản lỗi ra 14000 đ; bản đúng ra 15000 đ
```

Sau 36 bước thu nhỏ, phản ví dụ như một ca do người viết: xe máy, quãng đường **nhỏ nhất hợp lệ** (0,45 ⇒ 0,5 km), không cao điểm, không mưa, voucher yếu nhất. Kiểm tay: cước cơ bản 12.000 + 4.500 × 0,5 = 14.250 đ; tiền giảm min(8% × 14.250; 1.000) = 1.000 đ. Bản lỗi: max(14.250; 15.000) − 1.000 = 14.000 đ. Đúng SPEC: max(14.250 − 1.000; 15.000) = 15.000 đ. `seed + path` cho phép chạy lại đúng ca này; sau đó chép nó thành test ví dụ cố định (Bài tập 3).

### 5.5 BG-02 và BG-04 — phát hiện là chuyện xác suất

```text
BG-02 · số seed (trên 20) tìm ra lỗi: numRuns=100 ⇒ 0; numRuns=1.000 ⇒ 3
BG-04 · 0–3 km: 20/20 seed tìm ra lỗi trong 1.000 lần; số lần chạy tới failure: trung vị 170, nhỏ nhất 41, lớn nhất 496
```

Lỗi chỉ lộ khi đồng thời: xe máy (đặt xe hoặc bưu kiện), quãng đường làm tròn 0,5 hoặc 0,6 km, không cao điểm, không mưa, có voucher với mức tối thiểu đủ thấp — một lát cắt rất mỏng của miền (Hình 2). Mặc định 100 lần chạy: 0/20 seed tìm ra; 1.000 lần: 3/20. Seed 41 trúng ở lần 557 phần lớn nhờ fast-check thiên về giá trị biên — sinh đều thật sự thì xác suất còn nhỏ hơn nhiều.

BG-04 thu miền quãng đường về 0–3 km bằng hiểu biết nghiệp vụ («cước tối thiểu chỉ có hiệu lực với chuyến rất ngắn»): cả 20 seed đều tìm ra, trung vị 170 lần. Đó không phải adaptive random testing của syllabus, nhưng cho thấy cùng ý: **phân phối quyết định hiệu quả**.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 140" role="img" aria-label="Trục quãng đường 0 đến 120 km. Dưới 0,45 km và từ 100,05 km trở lên là vùng không hợp lệ. Vùng lỗi tối thiểu trước voucher chỉ là đoạn 0,45 đến 0,65 km sát biên dưới, rất hẹp so với cả trục.">
<defs>
<marker id="t041-arrow2" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-b">Vùng lỗi so với miền sinh quãng đường của unguidedCase</text>
<rect x="40" y="70" width="5" height="34" class="f-dead"/>
<rect x="45" y="70" width="3" height="34" class="f-warn"/>
<rect x="48" y="70" width="665" height="34" class="f-phys-soft s-phys"/>
<rect x="713" y="70" width="167" height="34" class="f-dead"/>
<line x1="40" y1="112" x2="880" y2="112" class="s-muted"/>
<text x="40" y="128" text-anchor="middle" class="t-10">0</text>
<text x="390" y="128" text-anchor="middle" class="t-10">50 km</text>
<text x="713" y="128" text-anchor="middle" class="t-10">100,05</text>
<text x="880" y="128" text-anchor="middle" class="t-10">120 km</text>
<text x="380" y="92" text-anchor="middle" class="t-small t-phys">hợp lệ: cước tối thiểu không còn tác dụng ⇒ bản lỗi cho kết quả đúng</text>
<text x="796" y="92" text-anchor="middle" class="t-small">không hợp lệ</text>
<path d="M46,68 L46,50 L120,50" fill="none" class="s-warn" stroke-width="1.5"/>
<text x="126" y="54" class="t-small t-warn">vùng lỗi: 0,45–0,65 km, xe máy, không cao điểm, không mưa, có voucher</text>
</svg>
</div>
<figcaption><b>Hình 2 · Vì sao random testing «bỏ sót một số defect».</b> Vùng lỗi chiếm ~0,2% trục quãng đường, trước khi nhân các điều kiện khác. Nó nằm sát biên dưới — chỗ <a href="/articles/boundary-value-analysis">BVA</a> nhìn vào đầu tiên.</figcaption>
</figure>

### 5.6 BG-03 — 10.000 ca xanh, sản phẩm vẫn sai

```text
BG-03 · 10.000 ca: failed=false. R-FARE-09: bản lỗi 54000 đ, bản đúng 52000 đ (oracle 52.000 đ)
```

Bản `rain-times-peak` cộng 8.000 đ phụ phí trời mưa **trước** khi nhân 1,25 (vi phạm R-FARE-05). Kết quả vẫn là bội số 1.000 đ, vẫn không dưới tối thiểu, vẫn làm tròn đúng một bậc — O1–O6 không có lý do gì để đỏ. Chỉ một oracle **cụ thể** (ví dụ R-FARE-09 do người tính: 52.000 đ) mới lộ ra: bản lỗi thu 54.000 đ. Đây chính là giới hạn «phụ thuộc oracle» và «bỏ qua ngữ nghĩa»: tính chất chung không biết rằng mưa phải cộng sau cao điểm. [TEST-042](/articles/metamorphic-testing) sẽ bắt lỗi này bằng một quan hệ giữa hai lần chạy — có mưa và không mưa.

## 6. Góc nhìn nâng cao

**So sánh với các kỹ thuật cùng nhóm.**

| | EP / BVA / domain | Combinatorial | Random |
|---|---|---|---|
| Chọn giá trị bằng | lập luận trên phân vùng và biên | lập luận trên tổ hợp tham số | xác suất |
| Coverage | có (EP, BVA, ON/OFF) | có (each-choice, pairwise, t-wise) | **không** |
| Oracle | người tính cho từng ca | người tính cho từng ca | **phải tự động** |
| Mạnh khi | đặc tả rõ, biên rõ | lỗi do tương tác tham số | miền lớn, hiểu biết hạn chế, cần nhiều dữ liệu |

Cách kết hợp hay dùng: **random trong từng phân vùng** — dùng EP/BVA để chọn vùng và biên, rồi sinh ngẫu nhiên bên trong mỗi vùng để có thêm giá trị không ai nghĩ tới.

**Trường hợp dễ bỏ sót.**

- Sinh ngoài hợp đồng của test item ⇒ failure giả, đội mất niềm tin vào bộ test.
- Oracle quá yếu (chỉ «không crash») ⇒ bộ test xanh vĩnh viễn. Đếm xem mỗi tính chất **đã từng đỏ** với bản lỗi nào chưa — giống ý tưởng [mutation testing](/articles/mutation-testing).
- Phân phối dồn ca vào một phân vùng (29,1% ca thừa ở RT-01) — luôn in thống kê kết cục.
- Không ghi seed ⇒ test «chập chờn» trong CI; không chép phản ví dụ thành test cố định ⇒ seed khác có thể không gặp lại.

**Sai lầm thường gặp.**

- Viết oracle bằng cách gọi lại hàm của sản phẩm (vd `roundToTenth`) hoặc chép công thức từ code: oracle đồng ý với chính lỗi cần tìm.
- Đồng nhất random testing với fuzz testing. Fuzz testing là một **ứng dụng** của random testing, thường nhắm vào độ bền và bảo mật (đầu vào hỏng cấu trúc); bài này nhắm vào đúng sai chức năng.
- Coi «10.000 ca xanh» là bằng chứng không có lỗi — trái [nguyên lý 1](/articles/bay-nguyen-ly-kiem-thu): testing chỉ cho thấy sự có mặt của defect.

**Trong hệ thống đích.** Ở tầng API, Quote API phụ thuộc MapViet (kiến trúc §14.1) ⇒ 10.000 lời gọi tốn thời gian và quota; giữ random testing ở tầng unit, tầng API chỉ chạy vài trăm ca với bản đồ giả (§17.2). Random **sự kiện** (thứ tự event, lỗi mạng) thuộc Series 11 (TEST-137).

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — bộ sinh tôn trọng hợp đồng.** Viết arbitrary cho danh sách món của đơn đồ ăn sao cho mọi giá trị đều thoả R-FOOD-02, và giải thích vì sao không nên sinh `unitPrice` bằng `fc.double()`.

<details>
<summary><b>Lời giải</b></summary>
<p><code>fc.array(fc.record({ name: fc.constantFrom('Phở bò', 'Cơm tấm'), unitPrice: fc.integer({ min: 1, max: 5_000 }).map(k =&gt; k * 1_000), quantity: fc.integer({ min: 1, max: 50 }) }), { minLength: 1, maxLength: 20 })</code>. Sinh «số nghìn» rồi nhân 1.000 bảo đảm bội số 1.000 đ mà không cần <code>filter</code> (lọc bỏ gần hết ca). <code>fc.double()</code> cho số lẻ, âm, NaN — tầng API (zod) chặn hết; đưa thẳng vào <code>priceFood</code> là sinh failure giả. Muốn kiểm việc chặn đầu vào xấu thì lấy <code>quoteSchema</code> làm test item riêng.</p>
</details>

**Bài tập 2 — cần chạy bao nhiêu ca?** Một lỗi chỉ xảy ra khi đầu vào rơi vào vùng chiếm p = 0,1% miền sinh. Cần bao nhiêu ca độc lập để xác suất phát hiện ít nhất một lần đạt 95%? Với 100 ca thì xác suất là bao nhiêu?

<details>
<summary><b>Lời giải</b></summary>
<p>Xác suất trượt cả n ca là (1 − p)<sup>n</sup>. Cần (1 − 0,001)<sup>n</sup> ≤ 0,05 ⇒ n ≥ ln 0,05 / ln 0,999 ≈ 2.995 ca. Với 100 ca: 1 − 0,999<sup>100</sup> ≈ 9,5%. Đây là lý do mặc định 100 lần chạy của công cụ không phù hợp làm exit criteria cho lõi giá. (Giả định sinh đều, các ca độc lập.)</p>
</details>

**Bài tập 3 — từ phản ví dụ thành test hồi quy.** Chép phản ví dụ BG-01 thành một test ví dụ cố định (không ngẫu nhiên) cho lõi giá thật, kèm kết quả mong đợi tính tay.

<details>
<summary><b>Lời giải</b></summary>
<p><code>it('R-FARE-07: voucher không kéo cước xe máy 0,45 km dưới 15.000 đ', () =&gt; { const r = priceRequest(normalizePriceRequest({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 0.45 }), { peak: false, raining: false, voucher: { code: 'T', percent: 8, maxDiscount: 1_000, minAmount: 0 } }); expect(r.fare.afterDiscount).toBe(13_250); expect(r.total).toBe(15_000); });</code> — kết quả mong đợi: 14.250 − 1.000 = 13.250 đ, nâng lên 15.000 đ, làm tròn giữ 15.000 đ. Đặt tên theo mã yêu cầu, ghi seed 41 trong comment.</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K2).** Đội Tới Nơi muốn ước lượng độ tin cậy của lõi định giá **như khách thật sử dụng**. Theo CTAL-TA, phân phối nào phù hợp nhất?

- a) Phân phối đều trên toàn miền đầu vào, kể cả giá trị không hợp lệ
- b) Phân phối theo operational profile
- c) Phân phối chỉ gồm giá trị biên
- d) Phân phối thay đổi theo các ca đã thất bại

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — mục đích validation ⇒ operational profile. a) là phân phối không theo thói quen dùng, hợp với verification. c) là BVA, không phải random testing. d) mô tả gần với guided random testing nhưng không phản ánh cách khách dùng thật.</p>
</details>

**Câu 2 (K2).** Tiêu chí dừng nào hợp lý cho random testing theo CTAL-TA?

- a) Đạt 100% random coverage
- b) Phủ mọi phân vùng tương đương ít nhất một lần
- c) Đã chạy 10.000 ca hoặc hết 2 phút, tuỳ điều kiện nào tới trước
- d) Không còn ca nào thất bại sau khi shrinking

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — random testing không có tiêu chí coverage được công nhận, nên exit criteria dựa vào số ca, thời gian hoặc thước đo tương tự. a) không tồn tại. b) là coverage của EP. d) shrinking là kỹ thuật của công cụ, không phải tiêu chí dừng.</p>
</details>

**Câu 3 (K2).** Phát biểu nào là một **giới hạn** của random testing được CTAL-TA nêu?

- a) Không thể tự động hoá
- b) Chỉ dùng được khi có đặc tả hình thức
- c) Phụ thuộc vào một oracle tự động
- d) Luôn kém hiệu quả hơn equivalence partitioning

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng. a) sai — random testing gần như luôn tự động. b) sai — nó có giá trị cả khi hiểu biết về miền hạn chế. d) sai — syllabus ghi nhận nghiên cứu thực nghiệm cho thấy trong một số điều kiện random testing hiệu quả hơn các kỹ thuật data-based khác.</p>
</details>

**Câu 4 (K2).** Điểm khác nhau giữa guided và unguided random testing là gì?

- a) Guided dùng seed cố định, unguided dùng seed ngẫu nhiên
- b) Guided điều chỉnh phân phối dựa trên các giá trị đã chọn trước đó; unguided giữ phân phối cố định
- c) Guided do người chọn đầu vào, unguided do máy chọn
- d) Guided chỉ sinh giá trị hợp lệ, unguided sinh cả giá trị không hợp lệ

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — vd adaptive random testing trải đầu vào đều trên miền vì defect hay tụ thành cụm. a) seed liên quan tới tái lập, không phải định hướng. c) cả hai đều do máy chọn. d) độ hợp lệ là do miền sinh, không phải do định hướng.</p>
</details>

**Câu 5 (K2).** Tester không có kết quả mong đợi chính xác cho 10.000 báo giá ngẫu nhiên, nên kiểm «tổng là bội số 1.000 đ và không nhỏ hơn cước tối thiểu». Đây là lời giải nào cho test oracle problem?

- a) Pseudo-oracle
- b) Human oracle
- c) Property-based testing
- d) Model-based testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — kiểm tính chất (quan hệ giữa đầu vào và kết quả) của từng ca. a) cần một hệ độc lập cùng đặc tả để so kết quả. b) cần người phán đoán từng ca. d) cần một mô hình test hình thức sinh ra kết quả mong đợi.</p>
</details>

**Câu 6 (K2).** Bộ random test 10.000 ca của lõi giá xanh, nhưng một ca do người tính (R-FARE-09) cho thấy cước sai 2.000 đ. Giải thích hợp lý nhất?

- a) Số ca chưa đủ; tăng lên 100.000 ca sẽ bắt được
- b) Oracle tính chất quá yếu để phân biệt kết quả đúng và sai với lỗi này
- c) Seed không tốt; đổi seed sẽ bắt được
- d) Ca R-FARE-09 sai vì không qua random testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — kết quả sai vẫn thoả mọi tính chất, nên thêm ca (a) hay đổi seed (c) cũng không đỏ. Cần oracle mạnh hơn: ca ví dụ, pseudo-oracle, metamorphic relation. d) sai — ca có oracle tính tay theo SPEC là bằng chứng mạnh hơn.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Advanced Level — Test Analyst (CTAL-TA) Syllabus*, v4.0 — mục 3.1 (data-based test techniques), **3.1.3 Random Testing**, 1.3.4 (test oracle). https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB, *Certified Tester Foundation Level Syllabus*, v4.0.1 — mục 4.2 (EP, BVA — nền của nhóm data-based). https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/
- ISO/IEC/IEEE 29119-4:2021, *Software testing — Part 4: Test techniques*.
- A. Arcuri, M. Z. Iqbal, L. Briand, «Random Testing: Theoretical Results and Practical Implications», IEEE TSE 38, 2012 — doi:10.1109/TSE.2011.121 (syllabus trích dẫn).
- R. Huang và cộng sự, «A Survey on Adaptive Random Testing», IEEE TSE 47(10), 2021 — doi:10.1109/TSE.2019.2942921 (syllabus trích dẫn).
- E. Barr và cộng sự, «The Oracle Problem in Software Testing: A Survey», IEEE TSE 41(5), 2015 — doi:10.1109/TSE.2014.2372785 (syllabus trích dẫn).
- K. Claessen, J. Hughes, «QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs», ICFP 2000 (nguồn gốc property-based testing — ngoài syllabus).
- fast-check — https://fast-check.dev/
- Trong blog: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-031 · Equivalence Partitioning](/articles/equivalence-partitioning) · [TEST-032 · Boundary Value Analysis](/articles/boundary-value-analysis) · [TEST-038 · Combinatorial Testing](/articles/combinatorial-testing) · [TEST-040 · Domain testing nhiều biến](/articles/data-based-test-techniques-domain-testing) · [TEST-063 · Mutation Testing](/articles/mutation-testing).

Bài tiếp theo: [TEST-042 · Metamorphic Testing — kiểm thuật toán cước khi không có oracle chính xác](/articles/metamorphic-testing).
