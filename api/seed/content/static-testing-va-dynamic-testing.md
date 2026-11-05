---
slug: static-testing-va-dynamic-testing
title: "Static Testing và Dynamic Testing: đọc code hay chạy test để tìm lỗi tính cước"
summary: "So sánh static testing và dynamic testing theo CTFL v4.0.1: loại defect mỗi cách tìm được, cách phát hiện, thời điểm. Tìm cùng một lỗi tính cước Tới Nơi bằng đọc code và bằng test vitest đỏ, chạy thật trên lab."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-022, static-testing, dynamic-testing, code-review, vitest, toi-noi]
language: vi
---

Ở [TEST-021](/articles/static-testing-la-gi), bản nháp đặc tả cước bỏ ngỏ một câu hỏi: phụ phí mưa có bị nhân hệ số cao điểm không? Giả sử không ai hỏi, và developer viết code nhân luôn cả phụ phí mưa. Bây giờ có hai cách tìm lỗi đó: **đọc code** (static testing) hoặc **chạy test** (dynamic testing). Bài này so sánh hai cách theo CTFL v4.0.1, rồi làm cả hai trên cùng một bản code lỗi của lab Tới Nơi để thấy mỗi cách tìm được gì, mất công ra sao và vì sao không cách nào thay được cách kia.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-022</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S03 · Static Testing &amp; Review Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Component</td></tr>
<tr><th scope="row">Test Type</th><td>White-box; Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Static testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/static-testing-la-gi">TEST-021 · Static Testing là gì?</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): cùng lỗi «phụ phí mưa bị nhân 1,25» (trái R-FARE-05) — tìm bằng đọc code, bằng một static check tự viết và bằng test vitest đỏ — <code>examples/toi-noi-testing/articles/static-testing-va-dynamic-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 3 · mục 3.1.3 · FL-3.1.3 (K2). Liên quan: CTAL-TTA v4.0 · mục 3.1 (static analysis và dynamic analysis — phần giới thiệu, không có LO riêng)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Hai câu nói hay nghe trong đội phát triển:

- «Có unit test phủ 100% rồi, review code làm gì cho tốn thời gian.»
- «Code đã được review kỹ, không cần viết test.»

Cả hai đều coi static và dynamic testing là hai cách làm **cùng một việc**, chọn một là đủ. Syllabus nói ngược lại: hai cách **bổ trợ nhau**. Chúng cùng mục tiêu phát hiện defect, nhưng mỗi cách nhìn thấy những loại defect khác nhau, theo những đường khác nhau, ở những thời điểm khác nhau.

Bài này trả lời ba câu hỏi thực tế: (1) loại defect nào nên tìm bằng cách nào; (2) khi tìm ra, mỗi cách cho ta thông tin gì; (3) trong một pipeline như của Tới Nơi, đặt hai cách ở đâu.

## 2. Lý thuyết theo ISTQB

### 2.1. Điểm chung và năm điểm khác (FL-3.1.3)

Static và dynamic testing có mục tiêu tương tự — hỗ trợ phát hiện defect trong work product. Điểm khác, theo mục 3.1.3:

| # | Khác biệt theo syllabus | Ở Tới Nơi |
|---|---|---|
| 1 | Cả hai đều dẫn tới phát hiện defect, nhưng có **loại defect chỉ một trong hai cách** tìm được | Chú thích sai công thức trong `fare.ts`: không test nào đỏ vì chú thích. Độ trễ p95 của Quote API: không ai đọc code mà đo ra được |
| 2 | Static testing tìm **thẳng defect**; dynamic testing gây ra **failure**, rồi defect được xác định qua **phân tích sau đó** | Đọc code thấy ngay dòng 29; chạy test chỉ thấy «54.000 ≠ 52.000», còn phải phân tích |
| 3 | Static testing dễ phát hiện defect nằm trên **đường đi ít khi chạy** hoặc khó chạm tới bằng dynamic testing | Nhánh xử lý cước âm không bao giờ chạy tới — không dữ liệu test nào vào được |
| 4 | Static testing áp được cho work product **không thực thi được**; dynamic testing chỉ cho thứ **thực thi được** | Review được đặc tả, `.proto`, test plan; chạy test chỉ được trên mã |
| 5 | Static testing đo được đặc tính chất lượng **không phụ thuộc thực thi** (vd maintainability); dynamic testing đo đặc tính **phụ thuộc thực thi** (vd performance efficiency) | Số `1.25` viết cứng khó bảo trì; p95 `POST /api/quotes` dưới 500 ms (R-NFR-03) |

### 2.2. Defect dễ hoặc rẻ hơn khi tìm bằng static testing

| Loại theo syllabus | Ví dụ ở Tới Nơi |
|---|---|
| Defect trong yêu cầu: không nhất quán, mơ hồ, mâu thuẫn, thiếu, không chính xác, trùng lặp | Bản nháp đặc tả cước v0.3 ở TEST-021 (12 defect) |
| Defect thiết kế: cấu trúc database kém hiệu quả, chia module kém | Pricing ghi thẳng vào bảng của Booking — anti-pattern «shared database» (kiến trúc §21) |
| Một số defect mã: biến chưa có giá trị, biến chưa khai báo, mã không chạy tới được hoặc lặp, độ phức tạp quá mức | Nhánh `atLeastMinimum <= 0` không bao giờ đúng |
| Lệch chuẩn: vd không theo quy ước đặt tên | Tên biến lẫn tiếng Việt không dấu và tiếng Anh |
| Đặc tả giao diện sai: lệch số lượng, kiểu, thứ tự tham số | `.proto` GetQuote khai `distance_km` kiểu `int32` trong khi Booking gửi số thực |
| Một số loại lỗ hổng bảo mật | Syllabus lấy ví dụ tràn bộ đệm; ở Node/TypeScript, ví dụ tương tự là ghép chuỗi SQL từ đầu vào |
| Lỗ hổng hoặc sai lệch trong độ phủ test basis | Không có test nào cho mốc 13:00 của R-FARE-04 |

Chiều ngược lại, dynamic testing cần cho mọi thứ **chỉ lộ ra khi chạy**: hiệu năng, hành vi khi các thành phần ghép lại, khác biệt của môi trường, và những lỗi tính toán mà không dòng code nào trông sai — sẽ thấy ở phần thực hành.

<div class="tkh-callout">
<span class="tkh-callout-title">Static/dynamic analysis ở CTAL-TTA — đừng nhầm với dynamic testing</span>
<p>CTAL-TTA v4.0 mục 3.1 tách <b>static analysis</b> (đánh giá mã theo hình thức, cấu trúc, nội dung mà không chạy) và <b>dynamic analysis</b> (theo dõi mã đang chạy để tìm lỗi như rò rỉ bộ nhớ, bằng công cụ hoặc người quan sát). Dynamic analysis là một dạng dynamic testing có mục tiêu hẹp; còn dynamic testing ở CTFL bao gồm mọi việc thực thi phần mềm với test case. Bài này dùng nghĩa CTFL.</p>
</div>

### 2.3. Thuật ngữ Anh – Việt

| Tiếng Anh | Tiếng Việt | Ghi chú |
|---|---|---|
| static testing | kiểm thử tĩnh | không thực thi |
| dynamic testing | kiểm thử động | thực thi phần mềm |
| failure | thất bại | hành vi sai quan sát được khi chạy |
| defect | khiếm khuyết | chỗ sai trong work product |
| static analysis | phân tích tĩnh | static testing bằng công cụ |
| dynamic analysis | phân tích động | theo dõi mã đang chạy (CTAL-TTA) |
| unreachable code | mã không chạy tới được | không đầu vào nào đi vào |
| maintainability | khả năng bảo trì | đo được bằng static testing |
| performance efficiency | hiệu quả hiệu năng | đo bằng dynamic testing |
| white-box / black-box | hộp trắng / hộp đen | dựa trên cấu trúc / dựa trên đặc tả |

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ và business rules

Hàm tính cước của Pricing & Quote Service phải theo đúng thứ tự R-FARE-03: cước cơ bản → **× 1,25 nếu cao điểm** (R-FARE-04) → **+ 8.000 đ nếu mưa, không bị nhân** (R-FARE-05) → − voucher → nâng lên cước tối thiểu → làm tròn lên 1.000 đ (R-FARE-08). Các bước trung gian tính chính xác, không làm tròn (R-GEN-01). Ví dụ kiểm R-FARE-09: xe máy 7,4 km, cao điểm, mưa, voucher 20% ⇒ **52.000 đ**.

### System context

Đối tượng kiểm là **một component**: hàm `calculateFare` trong `src/domain/fare.ts` của lab, tương ứng phần lõi của Pricing & Quote Service (kiến trúc §4.3). Pipeline CI của mỗi service (kiến trúc §17) đặt sẵn hai lớp: bước 1 **lint/static analysis**, bước 2 **unit test**. Đọc code là góc nhìn **white-box**; test suy từ SPEC là góc nhìn **black-box** — vì thế metadata ghi cả hai.

Để không sửa `src/`, bài dùng một bản chép có lỗi: `articles/static-testing-va-dynamic-testing/buggy-fare.ts`, gài 4 defect:

| Mã | Defect gài | Trái với |
|---|---|---|
| D1 | Phụ phí mưa bị nhân hệ số cao điểm | R-FARE-03, R-FARE-05 |
| D2 | Tính tiền bằng số thực thay vì milli-đồng như `money.ts` | R-GEN-01 (chính xác ở bước trung gian) |
| D3 | Chú thích nói voucher tính trên cước cơ bản; hệ số `1.25` viết cứng thay vì `PEAK_PERCENT` | R-FARE-06; quy ước bảo trì |
| D4 | Nhánh «cước âm» không bao giờ chạy tới | — (mã thừa) |

### Rủi ro có thể xảy ra

- D1 trốn sau bộ test «mỗi ca một điều kiện»: test riêng cao điểm, riêng mưa đều xanh.
- D2 chỉ lộ ở vài quãng đường, sai đúng 1.000 đ — khách khó nhận ra, đối soát mới thấy.
- D3 chưa gây failure hôm nay, nhưng khi bộ giá `fare-2026.2` đổi `PEAK_PERCENT`, dòng viết cứng vẫn nhân 1,25.
- Giá sai được chụp vào báo giá và chuyến (R-QUOTE-04), sửa code không sửa được dữ liệu đã lưu.

## 4. Phân tích kỹ thuật

### 4.1. Đoạn mã cần kiểm

Trích `buggy-fare.ts`, dòng 14–40 (số dòng ghi ở chú thích cuối dòng):

```ts
/**
 * Tính cước chuyến xe / bưu kiện.
 * Voucher được tính trên cước cơ bản, trước cao điểm và mưa.          // dòng 16
 */
export function calculateFareBuggy(input: FareInput): FareBreakdown {
  const distanceKm = normalizeDistanceKm(input.distanceKm);
  const prices = FARE_TABLE[input.vehicle];
  // 1. Cước cơ bản (+ phụ phí bưu kiện)
  const openingFee = prices.openingFee;
  const distanceCharge = prices.perKm * distanceKm;                    // dòng 24
  const parcelSurcharge = input.parcelSurcharge ?? 0;
  const base = openingFee + distanceCharge + parcelSurcharge;
  // 2–3. Cao điểm và mưa
  const rainSurcharge = input.raining ? RAIN_SURCHARGE : 0;
  const afterPeak = input.peak ? (base + rainSurcharge) * 1.25 : base + rainSurcharge; // dòng 29
  const beforeDiscount = afterPeak;
  // 4. Voucher
  const discount = input.voucher ? voucherDiscountMilli(input.voucher, toMilli(beforeDiscount)) / 1000 : 0;
  const afterDiscount = beforeDiscount - discount;
  // 5. Cước tối thiểu
  const atLeastMinimum = Math.max(afterDiscount, prices.minimumFare);
  if (atLeastMinimum <= 0) {                                           // dòng 36
    throw new DomainError('VALIDATION_ERROR', 'Cước không được âm.');
  }
  // 6. Làm tròn lên 1.000 đ
  const total = Math.ceil(atLeastMinimum / 1000) * 1000;              // dòng 40
```

### 4.2. Dự đoán trước khi làm

Trước khi chạy gì, dự đoán mỗi cách sẽ bắt được defect nào — đây chính là năng lực FL-3.1.3 đòi hỏi:

| Defect | Review code | Static check tự viết | Test «mỗi ca một điều kiện» | Thêm ca tổ hợp + ca chọn giá trị |
|---|---|---|---|---|
| D1 mưa bị nhân | Có — so dòng 29 với R-FARE-03 | Có, nếu có quy tắc cho nó | **Không** | Có |
| D2 số thực | Có thể — nếu người đọc biết bẫy số thực | Khó | Không | Có, nếu chọn trúng giá trị |
| D3 chú thích, số viết cứng | Có | Có (số viết cứng) | Không bao giờ | Không bao giờ |
| D4 nhánh thừa | Có | Cần công cụ hiểu miền giá trị | Không bao giờ | Không bao giờ |

### 4.3. Hai đường tới cùng một defect

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 320" role="img" aria-label="Hai đường tới defect D1. Hàng trên là static testing: mã buggy-fare.ts chưa chạy, đọc và đối chiếu SPEC R-FARE-03 thứ tự cao điểm rồi mưa, mũi tên tới thẳng ô defect. Hàng dưới là dynamic testing: chạy TC-05 theo R-FARE-09, thấy failure 54.000 khác 52.000 đồng, phân tích bảng chi tiết thấy cước cơ bản khớp nhưng sau cao điểm lệch 10.000, rồi mới tới ô defect. Ô defect bên phải: buggy-fare.ts dòng 29, biểu thức base cộng rainSurcharge nhân 1.25, phụ phí mưa bị nhân hệ số cao điểm. Dải dưới so sánh: tĩnh thấy defect trực tiếp có vị trí, không cần dữ liệu test; động thấy failure trước, chỉ lộ khi ca test có cả cao điểm và mưa.">
<defs>
<marker id="t022-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="t022-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-b">Cùng defect D1, hai đường tìm</text>
<text x="16" y="50" class="t-b t-phys">Static testing — đọc mã</text>
<rect x="16" y="60" width="200" height="64" rx="8" class="f-surface s-line"/>
<text x="116" y="86" text-anchor="middle" class="t-b t-mono">buggy-fare.ts</text>
<text x="116" y="106" text-anchor="middle" class="t-small">mã nguồn, chưa chạy</text>
<rect x="250" y="60" width="270" height="64" rx="8" class="f-phys-soft s-phys"/>
<text x="385" y="86" text-anchor="middle" class="t-b">Đọc, đối chiếu SPEC</text>
<text x="385" y="106" text-anchor="middle" class="t-small">R-FARE-03: × cao điểm rồi mới + mưa</text>
<line x1="216" y1="92" x2="248" y2="92" class="s-pk" stroke-width="1.6" marker-end="url(#t022-arrow)"/>
<line x1="520" y1="92" x2="638" y2="92" class="s-pk" stroke-width="1.6" marker-end="url(#t022-arrow)"/>
<text x="16" y="168" class="t-b t-warn">Dynamic testing — chạy test</text>
<rect x="16" y="178" width="150" height="64" rx="8" class="f-surface s-line"/>
<text x="91" y="204" text-anchor="middle" class="t-b">Chạy TC-05</text>
<text x="91" y="224" text-anchor="middle" class="t-small">theo R-FARE-09</text>
<rect x="186" y="178" width="170" height="64" rx="8" class="f-warn-soft s-warn"/>
<text x="271" y="204" text-anchor="middle" class="t-b t-warn">Failure</text>
<text x="271" y="224" text-anchor="middle" class="t-small">54.000 ≠ 52.000 đ</text>
<rect x="376" y="178" width="230" height="64" rx="8" class="f-warn-soft s-warn"/>
<text x="491" y="204" text-anchor="middle" class="t-b">Phân tích bảng chi tiết</text>
<text x="491" y="224" text-anchor="middle" class="t-small">base khớp, afterPeak lệch +10.000</text>
<line x1="166" y1="210" x2="184" y2="210" class="s-warn" stroke-width="1.6" marker-end="url(#t022-warn)"/>
<line x1="356" y1="210" x2="374" y2="210" class="s-warn" stroke-width="1.6" marker-end="url(#t022-warn)"/>
<line x1="606" y1="210" x2="638" y2="210" class="s-warn" stroke-width="1.6" marker-end="url(#t022-warn)"/>
<rect x="640" y="60" width="264" height="182" rx="8" class="f-pk-soft s-pk"/>
<text x="772" y="96" text-anchor="middle" class="t-b">Defect D1</text>
<text x="772" y="122" text-anchor="middle" class="t-small t-mono">buggy-fare.ts dòng 29</text>
<text x="772" y="150" text-anchor="middle" class="t-small t-mono">(base + rainSurcharge) * 1.25</text>
<text x="772" y="180" text-anchor="middle" class="t-small">phụ phí mưa bị nhân</text>
<text x="772" y="198" text-anchor="middle" class="t-small">hệ số cao điểm</text>
<rect x="16" y="262" width="436" height="46" rx="8" class="f-phys-soft s-phys"/>
<text x="234" y="281" text-anchor="middle" class="t-small">Tĩnh: thấy thẳng defect, có vị trí,</text>
<text x="234" y="298" text-anchor="middle" class="t-small">không cần dữ liệu test</text>
<rect x="468" y="262" width="436" height="46" rx="8" class="f-warn-soft s-warn"/>
<text x="686" y="281" text-anchor="middle" class="t-small">Động: thấy failure trước, phải phân tích;</text>
<text x="686" y="298" text-anchor="middle" class="t-small">chỉ lộ khi ca test có CẢ cao điểm và mưa</text>
</svg>
</div>
<figcaption><b>Hình 1 · Hai đường tới cùng một defect.</b> Đường tĩnh ngắn hơn nhưng phụ thuộc người đọc biết đối chiếu gì. Đường động dài hơn nhưng cho bằng chứng hành vi thật — và chỉ khi ca test chọn đúng tổ hợp điều kiện.</figcaption>
</figure>

## 5. Thực hành

Mọi lệnh chạy trong `examples/toi-noi-testing/`. Kết quả dưới đây chép từ lần chạy thật (vitest 3.2.7, Node 22).

### 5.1. Cách A — review code (static, white-box)

Đọc `buggy-fare.ts` cạnh SPEC, không chạy gì. Nhật ký review:

| # | Dòng | Phát hiện | Căn cứ | Defect |
|---|---|---|---|---|
| R1 | 29 | `(base + rainSurcharge) * 1.25` — phụ phí mưa nằm trong phép nhân | R-FARE-03 bước 2 → 3, R-FARE-05 | D1 |
| R2 | 24, 26, 40 | Tiền tính bằng số thực, `Math.ceil` trên số thực; `money.ts` của lab dùng milli-đồng chính để tránh việc này | R-GEN-01 | D2 |
| R3 | 16 | Chú thích nói voucher tính trên cước cơ bản; mã (đúng) tính trên số sau bước 3 | R-FARE-06 | D3 |
| R4 | 29 | `1.25` viết cứng; module đã có hằng `PEAK_PERCENT` | Bảo trì | D3 |
| R5 | 36–38 | `atLeastMinimum` ≥ cước tối thiểu (≥ 15.000 đ) nên điều kiện không bao giờ đúng | Mã không chạy tới được | D4 |

Nhật ký này giả định người review biết bẫy số thực (R2). Người review ít kinh nghiệm thường bỏ qua dòng 24 vì nó «trông đúng» — đó là giới hạn thật của review.

### 5.2. Cách A′ — static analysis bằng một script tự viết

`static-check.ts` đọc mã nguồn **như văn bản**, không import và không gọi `calculateFareBuggy`. Hai quy tắc: S1 cấm viết cứng `1.25`; S2 cấm phụ phí mưa nằm trong dòng nhân hệ số cao điểm. Đây là minh hoạ — quy tắc thô, công cụ thật (tsc, ESLint) ở [TEST-029](/articles/static-analysis-phan-tich-code-tinh).

```bash
npx vitest run articles/static-testing-va-dynamic-testing/static-check.test.ts
```

```text
buggy-fare.ts:29 [S1] Hệ số cao điểm viết cứng 1.25 — dùng PEAK_PERCENT.
    const afterPeak = input.peak ? (base + rainSurcharge) * 1.25 : base + rainSurcharge;
buggy-fare.ts:29 [S2] Phụ phí mưa nằm trong phép nhân hệ số cao điểm.
    const afterPeak = input.peak ? (base + rainSurcharge) * 1.25 : base + rainSurcharge;
```

Cùng script chạy trên `src/domain/fare.ts` (bản đúng) cho 0 phát hiện. Script bắt D1 và nửa D3, nhưng không thấy D2, D4 và chú thích sai — vì không ai viết quy tắc cho chúng.

### 5.3. Cách B — dynamic testing (black-box từ SPEC)

**Bước 1 — bộ test «mỗi ca một điều kiện».** Kiểu bộ test hay được viết đầu tiên: TC-01 không điều kiện (46.000 đ), TC-02 chỉ cao điểm (57.000 đ), TC-03 chỉ mưa (54.000 đ), TC-04 chỉ voucher GIAM20 (37.000 đ). Trên bản lỗi: **4/4 xanh**. Dòng 29 chạy ở cả bốn ca, nhưng chưa ca nào có cao điểm và mưa cùng lúc, nên `rainSurcharge` hoặc `peak` luôn mang giá trị «vô hại» — không có failure.

**Bước 2 — thêm ca tổ hợp và ca chọn giá trị.** TC-05 là chính ví dụ R-FARE-09; TC-06 là ô tô 18,6 km cao điểm, chọn vì 25.000 + 11.000 × 18,6 = 229.600, × 1,25 = **287.000** — rơi đúng bội số 1.000 đ, nơi sai số nhỏ nhất cũng bị làm tròn lên.

```bash
SHOW_RED=1 npx vitest run articles/static-testing-va-dynamic-testing
```

```text
 FAIL  … > TC-05 (R-FARE-09) cao điểm + mưa + GIAM20 ⇒ 52000 đ
AssertionError: expected 54000 to be 52000 // Object.is equality
 FAIL  … > TC-06 (R-FARE-03, R-FARE-08) ô tô 18,6 km, cao điểm ⇒ 287000 đ
AssertionError: expected 288000 to be 287000 // Object.is equality

 Test Files  1 failed | 2 passed (3)
      Tests  2 failed | 16 passed (18)
```

Không đặt `SHOW_RED`, hai ca này được đánh `it.fails` (failure đã biết) nên `npm test` của lab vẫn xanh: `18 passed` (gồm cả hai ca static check và tệp quét `d2-scan.test.ts` ở mục 6).

**Bước 3 — phân tích failure.** Test chỉ nói «sai bao nhiêu», chưa nói «sai ở đâu». So bảng chi tiết (R-FARE-10) của bản lỗi với SPEC:

| Bước | SPEC (TC-05) | Bản lỗi | Bản lỗi (TC-06) |
|---|---|---|---|
| Cước cơ bản `base` | 45.300 | 45.300 ✔ | 229.600,00000000003 |
| Sau cao điểm `afterPeak` | 56.625 | **66.625** ✘ (+10.000 = 8.000 × 1,25) | **287.000,00000000006** |
| Tổng | 52.000 | 54.000 | 288.000 |

TC-05: lệch bắt đầu ở bước 2–3 và đúng bằng phụ phí mưa × 1,25 ⇒ quay lại dòng 29. TC-06: `afterPeak` thừa 6 × 10⁻¹¹ đồng, `Math.ceil` đẩy lên 288.000 ⇒ nguyên nhân là biểu diễn số thực (D2). Sau khi định vị, ta tới đúng chỗ mà Cách A đã chỉ ra từ đầu.

### 5.4. So sánh kết quả

| | Cách A — review | Cách A′ — script tĩnh | Cách B — test động |
|---|---|---|---|
| D1 mưa bị nhân | ✔ trực tiếp, dòng 29 | ✔ dòng 29 | ✔ nhưng chỉ khi có ca tổ hợp; cần bước phân tích |
| D2 số thực | ✔ nếu người đọc biết bẫy | ✘ | ✔ với TC-06; cần phân tích |
| D3 chú thích, số viết cứng | ✔ | nửa (số viết cứng) | ✘ — không bao giờ đỏ |
| D4 nhánh thừa | ✔ | ✘ | ✘ — không đầu vào nào vào được |
| Thông tin mang lại | Vị trí defect, lý do | Vị trí, lặp lại được trong CI | Bằng chứng hành vi thật, giá trị sai cụ thể, ca hồi quy dùng lại mãi |
| Cần gì | Người đọc giỏi miền và ngôn ngữ | Người viết quy tắc trước | Mã chạy được, dữ liệu test chọn khéo |
| Công sức (ước lượng của người viết, không phải số đo) | ~10 phút đọc 50 dòng | Viết quy tắc ~30 phút, sau đó gần như 0 | Viết 6 ca ~20 phút; phân tích mỗi failure ~5 phút |

**Kết quả mong đợi** của bài: thấy được ít nhất một defect mà chỉ static testing tìm được (D3 hoặc D4) và một defect mà dynamic testing tìm dễ hơn hẳn (D2). **Kết quả thực tế**: đúng như bảng — không cách nào một mình bắt đủ cả bốn.

## 6. Góc nhìn nâng cao

**Phủ dòng ≠ tìm được defect.** Ở bước 1, dòng 29 đã được thực thi nhưng không có failure. Statement coverage 100% của hàm này vẫn để lọt D1. Đây là lý do kỹ thuật thiết kế test (tổ hợp điều kiện — Decision Table ở Series 04) quan trọng hơn con số coverage; chủ đề coverage có ở Series 05 (TEST-043 trở đi).

**Chọn giá trị là một nửa của dynamic testing.** `d2-scan.test.ts` quét cả 996 quãng đường hợp lệ (0,5–100 km, bước 0,1): với ô tô chỉ cao điểm, đúng **12 quãng** lộ D2 (18,6; 32,2; 36,2; … 91,4 km); ô tô ngoài cao điểm và xe máy: 0 quãng. Ca chọn ngẫu nhiên gần như chắc chắn bỏ lỡ; ca rơi đúng bội số 1.000 đ là ca có chủ đích — tư duy giá trị biên ở TEST-032.

**Có những thứ chỉ chạy mới biết.** Độ trễ p95 của `POST /api/quotes` (R-NFR-03), hành vi khi VíXanh hết giờ, rò rỉ bộ nhớ sau nhiều giờ chạy — không review nào thay được. Syllabus gọi đây là đặc tính chất lượng phụ thuộc thực thi.

**Static analysis có báo động giả.** Quy tắc S1 sẽ báo cả khi `1.25` là một hằng số khác (vd hệ số phí giờ đêm). Phân loại cảnh báo thật/giả là kỹ năng riêng ([TEST-029](/articles/static-analysis-phan-tich-code-tinh)).

**Test đúng đặc tả sai.** Nếu bản nháp đặc tả ở TEST-021 không được sửa, tester có thể viết TC-05 kỳ vọng 54.000 đ — test xanh, code sai. Dynamic testing chỉ tốt bằng test basis của nó; static testing trên test basis là lớp bảo vệ bên dưới.

**Thứ tự trong pipeline.** Kiến trúc §17 đặt static analysis trước unit test: rẻ, nhanh, chặn lỗi hiển nhiên trước khi tốn thời gian chạy. Nhưng đó là thứ tự **trong một lần build**, không phải «làm xong tĩnh rồi mới động».

**Sai lầm thường gặp:**

- Dùng một cách để «miễn» cách kia (hai câu ở mục 1).
- Bộ test chỉ có ca một điều kiện cho công thức nhiều bước.
- Coi chú thích, README là «không phải code» nên không review — chúng là work product, sai thì người sau làm sai theo.
- Nhầm dynamic analysis (CTAL-TTA) với dynamic testing nói chung.

## 7. Bài tập và câu hỏi ôn tập

**Bài 1.** Với mỗi defect, chọn cách tìm *dễ/rẻ hơn*: static, dynamic, hay cả hai tương đương. (a) `.proto` GetQuote khai `vehicle` là `int32`, Booking gửi chuỗi `"bike"`; (b) p95 `POST /api/quotes` là 900 ms khi 20 kết nối đồng thời; (c) test plan quên R-CANCEL; (d) `voucher.ts` có biến khai báo nhưng không dùng; (e) chuyến đồ ăn tính phí giao sai khi có mưa; (f) không có test nào cho mốc 13:00:00.

<details>
<summary><b>Lời giải</b></summary>
<p>(a) static — đặc tả giao diện sai kiểu tham số, review hợp đồng thấy ngay (chi tiết ở <a href="/articles/technical-review-api-contract">TEST-025</a>). (b) dynamic — hiệu năng phụ thuộc thực thi. (c) static — test plan không thực thi được. (d) static — loại defect mã syllabus nêu, static analysis bắt rẻ. (e) cả hai — review đối chiếu R-FOOD-04 hoặc test có mưa; dynamic cho bằng chứng hành vi. (f) static — lỗ hổng độ phủ test basis, review bộ test đối chiếu R-FARE-04.</p>
</details>

**Bài 2.** Thiết kế ca dynamic **ít điều kiện nhất** làm lộ D1. Kết quả mong đợi theo SPEC và kết quả bản lỗi là bao nhiêu?

<details>
<summary><b>Lời giải</b></summary>
<p>Cần đúng hai điều kiện: cao điểm và mưa, không voucher. Xe máy 7,4 km: SPEC 45.300 × 1,25 = 56.625 + 8.000 = 64.625 ⇒ <b>65.000 đ</b>. Bản lỗi: (45.300 + 8.000) × 1,25 = 66.625 ⇒ <b>67.000 đ</b>. Thêm voucher không cần thiết để lộ D1 — ca R-FARE-09 làm lộ được nhưng có nhiều điều kiện hơn mức cần.</p>
</details>

**Bài 3.** Dùng lab, tìm một quãng đường **khác 18,6 km** làm lộ D2 với ô tô cao điểm, không mưa, không voucher. Viết ca test (đầu vào, kết quả mong đợi).

<details>
<summary><b>Lời giải</b></summary>
<p>Ví dụ 32,2 km: 25.000 + 11.000 × 32,2 = 379.200; × 1,25 = <b>474.000 đ</b> (SPEC). Bản lỗi tính 474.000,00000000006 ⇒ làm tròn lên <b>475.000 đ</b>. Mẹo tìm: chọn quãng đường để cước sau cao điểm rơi đúng bội số 1.000 đ, rồi kiểm bằng <code>calculateFareBuggy</code>. Quãng 7,4 km xe máy không lộ vì phép nhân ra số đúng.</p>
</details>

**Câu 1 (K2).** Phát biểu nào **đúng** về khác biệt giữa static và dynamic testing?

- a) Static testing tìm thẳng defect; dynamic testing gây ra failure, rồi defect được xác định qua phân tích
- b) Dynamic testing tìm thẳng defect; static testing chỉ tìm failure
- c) Cả hai chỉ áp dụng được cho mã nguồn
- d) Static testing đo được performance efficiency tốt hơn dynamic testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a)</b> đúng — đúng ý mục 3.1.3. b) đảo ngược. c) static testing áp được cả cho work product không thực thi được. d) performance efficiency phụ thuộc thực thi, là việc của dynamic testing.</p>
</details>

**Câu 2 (K2).** Defect nào thường **dễ và rẻ hơn** khi tìm bằng static testing?

- a) Quote API chậm hơn 500 ms ở p95 khi tải cao điểm
- b) Bộ nhớ máy chủ tăng dần sau 24 giờ chạy
- c) Tệp `.proto` GetQuote khai sai kiểu và thứ tự tham số so với bên gọi
- d) Ứng dụng tài xế mất kết nối khi chuyển từ Wi-Fi sang 4G

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — «đặc tả giao diện sai» là một loại syllabus liệt kê. a), b), d) chỉ lộ khi chạy trong điều kiện cụ thể — cần dynamic testing (b còn là đối tượng của dynamic analysis ở CTAL-TTA).</p>
</details>

**Câu 3 (K2).** Hàm tính cước có bộ unit test xanh, phủ 100% câu lệnh. Sau đó một reviewer phát hiện phụ phí mưa bị nhân hệ số cao điểm. Giải thích nào hợp lý nhất?

- a) Bộ test sai vì không thể có defect khi phủ 100% câu lệnh
- b) Câu lệnh lỗi đã được thực thi, nhưng không ca nào kết hợp cao điểm với mưa nên không có failure quan sát được
- c) Reviewer nhầm, vì test đã xanh
- d) Defect nằm trên đường đi không bao giờ chạy tới

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — đúng tình huống bước 1 ở mục 5.3. a) coverage không chứng minh không còn defect. c) test xanh chỉ nói các ca đã chạy cho kết quả như kỳ vọng. d) dòng lỗi đã được chạy, chỉ là với giá trị không gây failure.</p>
</details>

**Câu 4 (K2).** Việc nào **chỉ** làm được bằng static testing?

- a) Đo thời gian phản hồi của `POST /api/trips`
- b) Tìm defect trong test plan của Sprint 12
- c) Kiểm luồng đặt chuyến trên trình duyệt
- d) Chạy lại bộ hồi quy sau khi sửa lỗi làm tròn

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — test plan không thực thi được, chỉ review được. a), c), d) đều đòi hỏi thực thi phần mềm.</p>
</details>

**Câu 5 (K2).** Nhánh xử lý khi MapViet trả quãng đường âm chỉ chạy khi nhà cung cấp gặp sự cố hiếm gặp, khó tái tạo trên môi trường test. Theo syllabus, cách nào **dễ** phát hiện defect trong nhánh này hơn?

- a) Dynamic testing, vì chỉ dynamic testing mới tìm được defect thật
- b) Static testing, vì nó dễ phát hiện defect trên đường đi ít khi chạy hoặc khó chạm tới
- c) Không cách nào, vì nhánh hiếm khi chạy thì không phải defect
- d) Acceptance testing với người dùng thật

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — điểm khác biệt thứ 3 của mục 3.1.3. a) static testing tìm defect thật, trực tiếp. c) defect vẫn là defect dù hiếm chạy; khi chạy tới sẽ gây failure. d) người dùng thật khó gặp đúng sự cố hiếm.</p>
</details>

**Câu 6 (K2).** Đội Pricing muốn biết hàm tính cước có dễ bảo trì không (độ phức tạp, số viết cứng, mã lặp). Cách phù hợp là gì?

- a) Static testing — maintainability là đặc tính không phụ thuộc thực thi
- b) Dynamic testing — cần chạy hàm với nhiều đầu vào
- c) Performance testing
- d) Không đánh giá được bằng kiểm thử

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a)</b> đúng — điểm khác biệt thứ 5: static testing đo được đặc tính không phụ thuộc thực thi như maintainability. b) chạy hàm không cho biết mã có dễ sửa không. c) đo hiệu năng, không phải bảo trì. d) sai — static analysis đánh giá được maintainability.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024), chương 3, mục 3.1.3 «Differences between Static Testing and Dynamic Testing», LO FL-3.1.3 (K2); liên quan mục 3.1, 3.1.2 và 1.1.2 (testing và debugging): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level — Technical Test Analyst Syllabus* **v4.0** (2021), chương 3 «Static and Dynamic Analysis», mục 3.1 và 3.3.1: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB Glossary — https://glossary.istqb.org/ — mục *static testing*, *dynamic testing*, *static analysis*, *dynamic analysis*, *failure*, *defect*.
- Trang chủ ISTQB: https://www.istqb.org/
- Tài liệu kiến trúc Tới Nơi: §4.3 Pricing & Quote, §17 CI/CD, §21 anti-pattern.
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-FARE, R-GEN-01, R-NFR-03; mã bài tại `articles/static-testing-va-dynamic-testing/`.
- Bài liên quan: [TEST-001 · Testing khác Debugging](/articles/software-testing-la-gi-testing-khac-debugging) · [TEST-004 · Error, Defect, Failure](/articles/error-defect-failure-root-cause) · [TEST-021 · Static Testing là gì?](/articles/static-testing-la-gi) · [TEST-029 · Static Analysis](/articles/static-analysis-phan-tich-code-tinh) · [TEST-019 · Confirmation và Regression Testing](/articles/confirmation-testing-va-regression-testing).

Bài tiếp theo: [Review Process và các vai trò — lập kế hoạch review đặc tả thanh toán VíXanh](/articles/review-process-va-cac-vai-tro).
