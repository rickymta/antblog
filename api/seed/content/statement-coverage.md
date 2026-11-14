---
slug: statement-coverage
title: "Statement Coverage: đo, thiết kế test tới 100% câu lệnh và vì sao vẫn chưa đủ"
summary: "Statement testing theo CTFL v4.0.1 và CTAL-TTA v4.0: đếm câu lệnh thực thi được, thiết kế test để phủ 100% pricing.ts của Tới Nơi bằng vitest + coverage v8, rồi chỉ ra một defect vẫn lọt dù đã đạt 100%."
category: testing
level: advanced
tags: [testing, istqb, ctfl, ctal-tta, test-043, white-box, statement-coverage, code-coverage, vitest]
language: vi
---

Bộ test báo giá của Tới Nơi xanh hết, nhưng có ai biết những dòng code nào **chưa từng chạy** trong lúc test? Statement testing trả lời câu hỏi đó một cách đo được: liệt kê câu lệnh thực thi được, thiết kế test để chạy qua chúng, rồi tính tỉ lệ. Bài này đo `pricing.ts` của lab từ 50% lên 100% câu lệnh, rồi cố tình gài một defect để thấy rõ **100% statement coverage bảo đảm gì và không bảo đảm gì**.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-043</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S05 · White-box &amp; Experience-based Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1; Advanced — CTAL-TTA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">Test Level</th><td>Component</td></tr>
<tr><th scope="row">Test Type</th><td>White-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Statement Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/component-testing-unit-testing-tong-quan">TEST-012 · Component Testing (Unit Testing)</a> · <a href="/articles/equivalence-partitioning">TEST-031 · Equivalence Partitioning</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): đo statement coverage của <code>src/domain/pricing.ts</code> bằng vitest + <code>@vitest/coverage-v8</code>, thêm test tới 100% câu lệnh, chỉ ra một defect vẫn lọt — <code>examples/toi-noi-testing/articles/statement-coverage/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 4 · mục 4.3, 4.3.1, 4.3.3 · FL-4.3.1 (K2), liên quan FL-4.3.3 (K2) · CTAL-TTA v4.0 · Chương 2 · mục 2.1, 2.2 · TTA-2.2.1 (K3)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

**Statement testing** (kiểm thử câu lệnh) là kỹ thuật **white-box** (hộp trắng): thiết kế test dựa trên **cấu trúc code**, không chỉ dựa trên đặc tả. Đơn vị cần phủ là **câu lệnh thực thi được**; thước đo là **statement coverage** — bao nhiêu phần trăm câu lệnh đã chạy ít nhất một lần.

Kỹ thuật này giải quyết một điểm mù của kỹ thuật hộp đen (Series 04): bạn chia lớp tương đương, lấy giá trị biên, dựng bảng quyết định — nhưng không biết bộ test đó **đã chạm vào phần nào của code**. Một nhánh xử lý đồ ăn chưa từng chạy thì mọi defect trong đó chắc chắn chưa có cơ hội lộ ra.

Khi nào dùng: ở **component level** (bài 012) khi có mã nguồn và công cụ đo; làm **mức sàn** cho mọi code được test (CTAL-TTA 2.2), thường kèm ngưỡng trong CI; và làm **bước bổ sung** — chạy bộ test hộp đen trước, đọc báo cáo coverage để tìm vùng chưa phủ. Code quan trọng hơn cần tiêu chí mạnh hơn: branch/decision coverage (bài 044), MC/DC (bài 047).

## 2. Lý thuyết theo ISTQB

### 2.1. Statement testing trong CTFL v4.0.1 (mục 4.3.1)

CTFL chọn hai kỹ thuật white-box gắn với code để giới thiệu: statement testing và branch testing (mục 4.3). Với statement testing:

- **Coverage item** là các **câu lệnh thực thi được** (executable statement).
- Mục tiêu: thiết kế test chạy qua các câu lệnh cho tới khi đạt **mức coverage chấp nhận được**.
- Công thức: **statement coverage = số câu lệnh đã chạy ÷ tổng số câu lệnh thực thi được × 100%**.

100% statement coverage bảo đảm **mọi câu lệnh chạy ít nhất một lần** — nên câu lệnh nào chứa defect cũng **có cơ hội** gây failure. Nhưng chạy qua chưa chắc lộ lỗi. Syllabus nêu hai giới hạn:

1. **Defect phụ thuộc dữ liệu** — ví dụ phép chia cho 0 chỉ hỏng khi mẫu số bằng 0. Câu lệnh đã chạy, nhưng với giá trị «vô hại».
2. **Logic quyết định chưa được kiểm hết** — 100% câu lệnh không bảo đảm mọi nhánh đã đi (bài 044).

### 2.2. Góc nhìn CTAL-TTA v4.0 (mục 2.1, 2.2)

CTAL-TTA nâng lên mức **K3**: TTA-2.2.1 yêu cầu *áp dụng* statement testing để thiết kế test đạt một mức coverage cho trước. Ba ý cần nhớ:

| Ý | Nội dung (diễn đạt lại) |
|---|---|
| Kết quả mong đợi | Lấy từ nguồn **ngoài cấu trúc code** (đặc tả, thiết kế) — mục 2.1. Không chép «code đang trả gì» vào `expect` |
| 100% nghĩa là gì | Kỹ thuật **không còn gợi ý test hữu ích nào nữa**, không phải bộ test đã đầy đủ — mục 2.1 |
| Phạm vi | **Mức tối thiểu** cho mọi code được test, dù thực tế không phải lúc nào cũng đạt — mục 2.2 |
| Giới hạn | Tỉ lệ cao vẫn sót defect logic; **mã không tới được** làm 100% bất khả — ví dụ `default` bắt buộc của `switch` đã xử lý đủ mọi trường hợp |

### 2.3. Giá trị và giới hạn của white-box (CTFL 4.3.3)

| Điểm mạnh | Điểm yếu |
|---|---|
| Xét **toàn bộ phần cài đặt**, tìm được defect cả khi đặc tả mơ hồ, cũ, thiếu | Không thấy **defect thiếu sót** (omission): yêu cầu không được cài đặt thì không có câu lệnh nào để phủ |
| Cho **số đo khách quan** về phần code đã test, chỉ ra chỗ cần thêm test | Coverage chỉ nói code **đã chạy**, không nói kết quả **đã được kiểm** |
| Dùng được trong **static testing**: chạy khô (dry run) code chưa chạy được, pseudocode | Cần mã nguồn, công cụ đo, người đọc được code |

### 2.4. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa tiếng Việt | Ghi chú |
|---|---|---|
| white-box test technique | kỹ thuật kiểm thử hộp trắng | dựa trên cấu trúc bên trong |
| statement testing | kiểm thử câu lệnh | kỹ thuật |
| statement coverage | độ phủ câu lệnh | thước đo, % |
| executable statement | câu lệnh thực thi được | khai báo kiểu, `import type`, chú thích không tính |
| control flow graph | đồ thị luồng điều khiển | nút = khối lệnh, cạnh = chuyển điều khiển |
| unreachable code | mã không tới được | làm 100% không đạt được |
| data-dependent defect | defect phụ thuộc dữ liệu | chỉ lộ với một số giá trị |
| defect of omission | defect thiếu sót | yêu cầu không được cài đặt |
| test oracle | nguồn kết quả mong đợi | SPEC, không phải code |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB — công cụ đo coverage hoạt động thế nào</span>
<p>Hai cách phổ biến với JavaScript/TypeScript: <b>chèn bộ đếm vào mã</b> trước khi chạy (Istanbul) hoặc <b>hỏi engine V8</b> khối mã nào đã chạy (<code>@vitest/coverage-v8</code>). V8 nhanh, không sửa mã, nhưng đếm theo <i>khối</i> và khoảng byte; kết quả được chuyển về tệp TypeScript qua source map, và cách chuyển quyết định «một câu lệnh» là gì (mục 6.1).</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Khách gửi yêu cầu báo giá (`POST /api/quotes`) cho một trong ba dịch vụ: đặt xe (`ride`), giao bưu kiện (`parcel`), giao đồ ăn (`food`). Trong lab, `src/domain/pricing.ts` là **lớp điều phối** của báo giá: hàm `normalizePriceRequest` kiểm và chuẩn hoá đầu vào theo đúng thứ tự R-VALID-01, hàm `priceRequest` chuyển yêu cầu đã chuẩn hoá sang công thức của từng dịch vụ.

### Business rules (SPEC lab)

| Mã | Quy tắc | Phần code liên quan |
|---|---|---|
| R-VALID-01 | Báo lỗi đầu tiên theo thứ tự: thành phố → quãng đường → khối lượng → loại xe theo khối lượng → tiền món | `normalizePriceRequest` |
| R-FARE-02, R-FARE-03 | Làm tròn quãng đường 0,1 km; 6 bước tính cước cố định | gọi `normalizeDistanceKm`, `calculateFare` |
| R-FARE-09 | Oracle: xe máy 7,4 km, cao điểm, mưa, giảm 20% ⇒ **52.000 đ** | TC-01 |
| R-PARCEL-01, 02 | Khối lượng làm tròn 0,1 kg rồi phải 0 &lt; w ≤ 30; **w &gt; 20 kg bắt buộc ô tô, 20,0 kg vẫn đi xe máy** | `case 'parcel'` |
| R-PARCEL-03, 04 | 2.000 đ mỗi kg bắt đầu vượt 5 kg; phụ phí cộng vào cước cơ bản nên **bị nhân** hệ số cao điểm | `parcelSurcharge` |
| R-FOOD-01, 03, 04, 05 | Tổng = tiền món − giảm + phí giao; tiền món ≥ 30.000 đ; phí giao theo công thức xe máy, voucher không trừ vào phí giao | `case 'food'`, nhánh `priceFood` |

### System context

Trong kiến trúc mục tiêu đây là **Pricing &amp; Quote Service** (§4.3: quote có `pricing_version`, breakdown, thời hạn); test quy tắc giá thuộc tầng **Unit** (§17.1), chạy ở bước 2 pipeline (§17) — đúng chỗ đặt ngưỡng coverage. Lab là bản MVP modular monolith: `quote.ts` → `pricing.ts` → `fare.ts`, `parcel.ts`, `food.ts`. Bài chỉ đo **một tệp** `pricing.ts`.

### Rủi ro có thể xảy ra

| Rủi ro | Hậu quả | Statement testing giúp gì |
|---|---|---|
| Nhánh `parcel` hoặc `food` chưa từng chạy trong test | Lỗi cấu trúc (sai trường, sai hàm) ra tới production | Báo cáo chỉ đúng dòng chưa chạy |
| Lớp điều phối dùng nhầm biến (thô thay vì đã chuẩn hoá) | Từ chối nhầm đơn hợp lệ ở biên | **Không đủ** — câu lệnh chạy nhưng với giá trị vô hại (mục 5.4) |
| Thiếu hẳn một quy tắc (ví dụ quên kiểm loại xe của đơn đồ ăn) | Chấp nhận đơn sai | **Không** — không có câu lệnh nào để phủ |

Ước lượng (giả định) cho rủi ro thứ hai: hồ sơ dự án có 0,3 triệu đơn bưu kiện/ngày. Nếu 0,1% đơn có khối lượng khách nhập nằm ngay trên 20 kg (ví dụ cân điện tử ra 20,03), số đơn bị từ chối sai mỗi ngày ≈ 300.000 × 0,001 = **300 đơn** — mỗi đơn là một khách phải đổi sang ô tô đắt hơn hoặc bỏ đi.

## 4. Phân tích kỹ thuật

### 4.1. Liệt kê coverage item

Câu lệnh thực thi được của `pricing.ts` (đếm theo cây cú pháp; `import`, `type`, `interface` và chú thích không tính). Cột «Dòng» là số dòng trong tệp của lab.

| Mã | Dòng | Câu lệnh | Thuộc hàm |
|---|---|---|---|
| S1 | 9 | `export const PRICING_VERSION = 'fare-2026.1'` | cấp module (chạy khi nạp tệp) |
| S2 | 41 | `if (!isCityCode(request.cityCode))` | `normalizePriceRequest` |
| S3 | 42 | `throw new DomainError('CITY_NOT_SUPPORTED', …)` | |
| S4 | 44 | `const distanceKm = normalizeDistanceKm(…)` | |
| S5 | 46 | `switch (request.kind)` | |
| S6 | 48 | `return { kind: 'ride', … }` | |
| S7 | 50 | `const weight = normalizeWeightKg(…)` | |
| S8 | 51 | `assertVehicleFits(weight, request.vehicle)` | |
| S9 | 52 | `return { kind: 'parcel', … }` | |
| S10 | 55 | `assertSubtotalEnough(itemsSubtotal(request.items))` | |
| S11 | 56 | `return { kind: 'food', … vehicle: 'bike', … }` | |
| S12 | 65 | `if (request.kind === 'food')` | `priceRequest` |
| S13 | 66 | `const food = priceFood({ …, items: request.items ?? [], … })` | |
| S14 | 67–74 | `return { fare: food.deliveryFee, … }` | |
| S15 | 76–81 | `const fare = calculateFare({ …, parcelSurcharge: w === null ? 0 : … })` | |
| S16 | 82 | `return { fare, itemsSubtotal: null, … }` | |

Tổng **16** câu lệnh. Câu S13 và S15 mỗi câu chứa một biểu thức có hai khả năng (`??`, `? :`) nhưng vẫn chỉ là **một** câu lệnh — statement coverage coi chúng là «đã chạy» ngay khi một khả năng được đi.

### 4.2. Bao nhiêu test là đủ?

Mỗi lần gọi `normalizePriceRequest` chỉ đi ra đúng một trong bốn lối: S3 (ném lỗi thành phố), S6 (ride), S9 (parcel), S11 (food). Vậy cần **ít nhất bốn lần gọi** — với quy ước một test case một lần gọi, tối thiểu là **4 test case**. Ở `priceRequest`, S13–S14 cần một yêu cầu `food`, S15–S16 cần một yêu cầu không phải `food`; hai yêu cầu đó đã có trong bốn test case trên, nên không cần thêm.

Test condition rút ra (mỗi condition dẫn vào một lối ra):

| Test condition | Câu lệnh mục tiêu | Quy tắc |
|---|---|---|
| TCo-1 · Yêu cầu đặt xe hợp lệ | S6, S15, S16 | R-FARE-03, R-FARE-09 |
| TCo-2 · Thành phố chưa phục vụ | S3 | R-VALID-01 |
| TCo-3 · Bưu kiện hợp lệ có phụ phí khối lượng | S7, S8, S9 | R-PARCEL-01…04 |
| TCo-4 · Đơn đồ ăn hợp lệ có voucher | S10, S11, S13, S14 | R-FOOD-01, 04, 05 |

### 4.3. Test case — kết quả mong đợi lấy từ SPEC

| Ca | Đầu vào | Kết quả mong đợi (tính từ SPEC) | Câu lệnh mới phủ |
|---|---|---|---|
| TC-01 | ride, SG, bike, 7,4 km; cao điểm, mưa, GIAM20 (20%, trần 30.000) | giảm 12.925; tổng **52.000 đ** (R-FARE-09) | S1, S2, S4, S5, S6, S12, S15, S16 |
| TC-02 | ride, `DL`, bike, 5 km | `CITY_NOT_SUPPORTED` | S3 |
| TC-03 | parcel, HN, car, 10 km, 6,1 kg; cao điểm | phụ phí 4.000; cơ bản 25.000 + 110.000 + 4.000 = 139.000; × 1,25 = 173.750; tổng **174.000 đ** | S7, S8, S9 |
| TC-04 | food, DN, 3 km, 2 × 50.000; FOOD15 (15%, trần 25.000, tối thiểu 50.000) | tiền món 100.000; giảm 15.000; phí giao 12.000 + 13.500 = 25.500 ⇒ **26.000**; tổng **111.000 đ**; xe `bike` | S10, S11, S13, S14 |

TC-03 chọn khối lượng 6,1 kg (không phải 6,0) vì 6,1 kg cho phụ phí hai «kg bắt đầu» — giá trị này còn kiểm luôn R-PARCEL-03; và chọn cao điểm để kiểm R-PARCEL-04 (phụ phí bị nhân 1,25). Statement testing quyết định *đi vào nhánh nào*; **chọn giá trị nào trong nhánh** vẫn là việc của kỹ thuật hộp đen.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 270" role="img" aria-label="Ma trận phủ câu lệnh của pricing.ts: 16 cột câu lệnh S1 đến S16, bốn hàng TC-01 đến TC-04. TC-01 phủ mới 8 câu, TC-02 thêm S3, TC-03 thêm S7 S8 S9, TC-04 thêm S10 S11 S13 S14. Độ phủ cộng dồn 8, 9, 12, 16 trên 16.">
<text x="12" y="30" class="t-small t-b">Ca</text>
<text x="804" y="30" class="t-small t-b">Cộng dồn</text>
<g class="t-10 t-mono">
<text x="133" y="30" text-anchor="middle">S1</text><text x="175" y="30" text-anchor="middle">S2</text><text x="217" y="30" text-anchor="middle">S3</text><text x="259" y="30" text-anchor="middle">S4</text><text x="301" y="30" text-anchor="middle">S5</text><text x="343" y="30" text-anchor="middle">S6</text><text x="385" y="30" text-anchor="middle">S7</text><text x="427" y="30" text-anchor="middle">S8</text><text x="469" y="30" text-anchor="middle">S9</text><text x="511" y="30" text-anchor="middle">S10</text><text x="553" y="30" text-anchor="middle">S11</text><text x="595" y="30" text-anchor="middle">S12</text><text x="637" y="30" text-anchor="middle">S13</text><text x="679" y="30" text-anchor="middle">S14</text><text x="721" y="30" text-anchor="middle">S15</text><text x="763" y="30" text-anchor="middle">S16</text>
</g>
<text x="12" y="66" class="t-small t-mono">TC-01 ride</text>
<text x="12" y="108" class="t-small t-mono">TC-02 thành phố</text>
<text x="12" y="150" class="t-small t-mono">TC-03 parcel</text>
<text x="12" y="192" class="t-small t-mono">TC-04 food</text>
<g class="f-phys s-phys">
<rect x="116" y="46" width="34" height="30" rx="4"/><rect x="158" y="46" width="34" height="30" rx="4"/><rect x="242" y="46" width="34" height="30" rx="4"/><rect x="284" y="46" width="34" height="30" rx="4"/><rect x="326" y="46" width="34" height="30" rx="4"/><rect x="578" y="46" width="34" height="30" rx="4"/><rect x="704" y="46" width="34" height="30" rx="4"/><rect x="746" y="46" width="34" height="30" rx="4"/>
<rect x="200" y="88" width="34" height="30" rx="4"/>
<rect x="368" y="130" width="34" height="30" rx="4"/><rect x="410" y="130" width="34" height="30" rx="4"/><rect x="452" y="130" width="34" height="30" rx="4"/>
<rect x="494" y="172" width="34" height="30" rx="4"/><rect x="536" y="172" width="34" height="30" rx="4"/><rect x="620" y="172" width="34" height="30" rx="4"/><rect x="662" y="172" width="34" height="30" rx="4"/>
</g>
<g class="f-phys-soft s-line">
<rect x="116" y="88" width="34" height="30" rx="4"/><rect x="158" y="88" width="34" height="30" rx="4"/>
<rect x="116" y="130" width="34" height="30" rx="4"/><rect x="158" y="130" width="34" height="30" rx="4"/><rect x="242" y="130" width="34" height="30" rx="4"/><rect x="284" y="130" width="34" height="30" rx="4"/><rect x="578" y="130" width="34" height="30" rx="4"/><rect x="704" y="130" width="34" height="30" rx="4"/><rect x="746" y="130" width="34" height="30" rx="4"/>
<rect x="116" y="172" width="34" height="30" rx="4"/><rect x="158" y="172" width="34" height="30" rx="4"/><rect x="242" y="172" width="34" height="30" rx="4"/><rect x="284" y="172" width="34" height="30" rx="4"/><rect x="578" y="172" width="34" height="30" rx="4"/>
</g>
<text x="804" y="66" class="t-small t-b">8/16 · 50%</text>
<text x="804" y="108" class="t-small t-b">9/16 · 56,25%</text>
<text x="804" y="150" class="t-small t-b">12/16 · 75%</text>
<text x="804" y="192" class="t-small t-b t-phys">16/16 · 100%</text>
<rect x="116" y="226" width="20" height="16" rx="3" class="f-phys s-phys"/><text x="144" y="239" class="t-small">ca này phủ câu lệnh lần đầu</text>
<rect x="356" y="226" width="20" height="16" rx="3" class="f-phys-soft s-line"/><text x="384" y="239" class="t-small">ca này chạy lại câu lệnh đã phủ</text>
<text x="604" y="239" class="t-small">ô trống: không chạy</text>
</svg>
</div>
<figcaption><b>Hình 1 · Ma trận phủ câu lệnh của <code>pricing.ts</code>.</b> Đọc theo cột: mỗi câu lệnh có ít nhất một ô tô màu thì đạt 100%. Số cộng dồn bên phải khớp với báo cáo coverage v8 khi lọc lần lượt <code>-t "TC-01"</code>, <code>"TC-0[12]"</code>, <code>"TC-0[1-3]"</code>, <code>"TC-0[1-4]"</code>.</figcaption>
</figure>

### 4.4. Những gì ma trận không cho thấy

Vế `[]` của `?? []` (S13) không bao giờ chạy, và S15 «đã phủ» dù mới một vế của `? :` được đi. Quan trọng hơn: S8 (`assertVehicleFits`) chỉ chạy với 6,1 kg, xa biên 20 kg — câu lệnh đã phủ, **giá trị nguy hiểm thì chưa**. Mục 5.4 khai thác đúng chỗ này.

## 5. Thực hành

Mọi lệnh chạy từ gốc lab `examples/toi-noi-testing` (Node 22.18, vitest 3.2.7, `@vitest/coverage-v8` 3.2). Mã bài trong `articles/statement-coverage/`; kết quả chép từ lần chạy thật. Cờ `--coverage.experimentalAstAwareRemapping` cho vitest đếm câu lệnh theo cây cú pháp, khớp định nghĩa ISTQB (lý do ở mục 6.1). Báo cáo ghi vào thư mục riêng để không đụng `reports/coverage` của lệnh khác.

```bash
COV="--coverage.enabled --coverage.reporter=text --coverage.reportsDirectory=reports/coverage-statement-coverage"
INC="--coverage.include=src/domain/pricing.ts --coverage.experimentalAstAwareRemapping"
```

### 5.1. Điểm xuất phát: bộ smoke test của lab

```bash
npx vitest run tests/smoke $COV $INC
```

```text
      Tests  22 passed (22)
 pricing.ts |      50 |    36.36 |     100 |      50 | 42,50-56,66-67
```

22 test xanh, nhưng chỉ **8/16** câu lệnh của `pricing.ts` từng chạy: smoke test API chỉ báo giá đặt xe. Nhánh bưu kiện (50–52), đồ ăn (55–56, 66–67) và lỗi thành phố (42) chưa ai đi qua.

### 5.2. Bước 1 — một test case theo oracle R-FARE-09

```bash
npx vitest run articles/statement-coverage/phu-cau-lenh.test.ts -t "TC-01" $COV $INC
```

```text
      Tests  1 passed | 4 skipped (5)
File        | % Stmts | % Branch | % Funcs | % Lines | Uncovered Line #s
All files   |      50 |    36.36 |     100 |      50 |
 pricing.ts |      50 |    36.36 |     100 |      50 | 42,50-56,66-67
```

Một test case «chuẩn» cho đúng cùng mức với 22 smoke test — vì cả hai chỉ đi lối `ride`. Cột «Uncovered Line #s» là danh sách việc cần làm: mỗi dòng ứng với một câu lệnh ở bảng 4.1.

### 5.3. Bước 2 — thêm test case theo từng câu lệnh chưa phủ

Mã của bốn test case (rút gọn từ `bo-phu-cau-lenh.ts`; bộ test case viết thành hàm để chạy được trên hai bản cài đặt ở bước 3):

```ts
it('TC-03 · bưu kiện 6,1 kg ô tô 10 km cao điểm ⇒ 174.000 đ (R-PARCEL-03/04)', () => {
  const req = p.normalizePriceRequest({ kind: 'parcel', cityCode: 'HN', vehicle: 'car', distanceKm: 10, parcelWeightKg: 6.1 });
  const price = p.priceRequest(req, { peak: true, raining: false });
  expect(price.fare).toMatchObject({ parcelSurcharge: 4_000, base: 139_000, afterPeak: 173_750 });
  expect(price.total).toBe(174_000);
});
```

```bash
npx vitest run articles/statement-coverage/phu-cau-lenh.test.ts -t "TC-0[1-4]" $COV $INC
```

```text
      Tests  4 passed | 1 skipped (5)
File        | % Stmts | % Branch | % Funcs | % Lines | Uncovered Line #s
All files   |     100 |     90.9 |     100 |     100 |
 pricing.ts |     100 |     90.9 |     100 |     100 | 66
```

**100% câu lệnh (16/16)** với đúng bốn test case như phân tích ở 4.2. Cột «Uncovered» vẫn còn dòng 66: đó là một **nhánh** chưa đi (vế `[]` của `??`), không phải câu lệnh. Nhánh này không thể đi qua đường công khai của module — đúng loại «không tới được» mà CTAL-TTA nhắc. Bài 044 bàn tiếp về nhánh.

### 5.4. Bước 3 — một defect vẫn lọt dù 100%

Bản `pricing-loi.ts` chép từ mã lab, khác **một dòng** trong `case 'parcel'`:

```ts
// src/domain/pricing.ts (đúng)
assertVehicleFits(weight, request.vehicle);
// articles/statement-coverage/pricing-loi.ts (lỗi): dùng khối lượng khách gửi, CHƯA làm tròn 0,1 kg
assertVehicleFits(request.parcelWeightKg, request.vehicle);
```

Lỗi rất đời thường: hai biến cùng nghĩa «khối lượng», chọn nhầm. Chạy **cùng bốn test case** trên bản lỗi, đo coverage của chính bản lỗi:

```bash
npx vitest run articles/statement-coverage/ban-loi.test.ts -t "TC-0[1-4]" $COV \
  --coverage.include=articles/statement-coverage/pricing-loi.ts --coverage.experimentalAstAwareRemapping
```

```text
      Tests  4 passed | 1 skipped (5)
File            | % Stmts | % Branch | % Funcs | % Lines | Uncovered Line #s
All files       |     100 |     90.9 |     100 |     100 |
 pricing-loi.ts |     100 |     90.9 |     100 |     100 | 72
```

Bốn test case xanh, **100% câu lệnh của chính bản lỗi** — dòng có defect đã chạy (TC-03), nhưng với 6,1 kg thì khối lượng thô và khối lượng đã làm tròn bằng nhau, nên không có failure. Defect chỉ lộ khi khối lượng thô nằm trong khoảng (20; 20,05) kg: làm tròn thành 20,0 kg (xe máy hợp lệ theo R-PARCEL-02) nhưng giá trị thô lại &gt; 20.

Test case TC-05 lấy từ phân tích giá trị biên (bài 032), không phải từ coverage:

```ts
// 20,04 kg ⇒ làm tròn 20,0 kg ⇒ xe máy vẫn hợp lệ (R-PARCEL-01, R-PARCEL-02)
const req = p.normalizePriceRequest({ kind: 'parcel', cityCode: 'SG', vehicle: 'bike', distanceKm: 5, parcelWeightKg: 20.04 });
expect(req).toMatchObject({ vehicle: 'bike', parcelWeightKg: 20 });
```

```bash
SHOW_RED=1 npx vitest run articles/statement-coverage/ban-loi.test.ts
```

```text
   × pricing-loi.ts (bản có lỗi) > TC-05 · bưu kiện 20,04 kg đi xe máy ⇒ phải hợp lệ (R-PARCEL-01/02) 8ms
     → Bưu kiện trên 20 kg phải giao bằng ô tô.
 FAIL  articles/statement-coverage/ban-loi.test.ts > pricing-loi.ts (bản có lỗi) > TC-05 …
DomainError: Bưu kiện trên 20 kg phải giao bằng ô tô.
 ❯ assertVehicleFits src/domain/parcel.ts:22:11
 ❯ Object.normalizePriceRequest articles/statement-coverage/pricing-loi.ts:57:7
      Tests  1 failed | 4 passed (5)
```

Trong `npm test` của lab, TC-05 trên bản lỗi khai `it.fails` (ghi nhận defect mà không làm đỏ cả lab); trên mã thật TC-05 là test case thường và xanh.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 210" role="img" aria-label="Trục khối lượng thô từ 19,9 đến 20,1 kg. Khoảng từ 20 đến 20,05 kg tô cảnh báo: mã thật làm tròn thành 20,0 kg và cho xe máy, bản lỗi so giá trị thô lớn hơn 20 nên từ chối. Ca TC-03 ở 6,1 kg nằm xa vùng này nên không lộ lỗi; TC-05 ở 20,04 kg nằm trong vùng và lộ lỗi.">
<line x1="80" y1="110" x2="860" y2="110" class="s-line"/>
<rect x="470" y="86" width="195" height="48" class="f-warn-soft s-warn"/>
<line x1="80" y1="102" x2="80" y2="118" class="s-line"/><text x="80" y="150" text-anchor="middle" class="t-small t-mono">19,90</text>
<line x1="275" y1="102" x2="275" y2="118" class="s-line"/><text x="275" y="150" text-anchor="middle" class="t-small t-mono">19,95</text>
<line x1="470" y1="96" x2="470" y2="124" class="s-line"/><text x="470" y="150" text-anchor="middle" class="t-small t-mono t-b">20,00</text>
<line x1="665" y1="96" x2="665" y2="124" class="s-line"/><text x="665" y="150" text-anchor="middle" class="t-small t-mono t-b">20,05</text>
<line x1="860" y1="102" x2="860" y2="118" class="s-line"/><text x="860" y="150" text-anchor="middle" class="t-small t-mono">20,10</text>
<text x="567" y="76" text-anchor="middle" class="t-small t-b t-warn">vùng defect: (20; 20,05)</text>
<circle cx="626" cy="110" r="7" class="f-warn s-warn"/><text x="626" y="176" text-anchor="middle" class="t-small t-mono">TC-05 · 20,04 kg</text>
<text x="40" y="40" class="t-small">Mã thật: làm tròn 0,1 kg trước ⇒ mọi giá trị &lt; 20,05 thành ≤ 20,0 ⇒ xe máy hợp lệ.</text>
<text x="40" y="58" class="t-small">Bản lỗi: so giá trị thô &gt; 20 ⇒ từ chối nhầm trong vùng tô màu.</text>
<text x="80" y="198" class="t-small">TC-03 dùng 6,1 kg — xa biên: câu lệnh chạy nhưng không lộ lỗi.</text>
</svg>
</div>
<figcaption><b>Hình 2 · Defect phụ thuộc dữ liệu.</b> Câu lệnh chứa defect được mọi test case bưu kiện chạy qua; chỉ giá trị trong một khoảng rất hẹp mới làm hai biến «khối lượng» khác nhau.</figcaption>
</figure>

### 5.5. Tổng hợp kết quả

| Bước | Bộ test | Statement coverage | Branch (v8) | Kết quả test | Defect của bản lỗi |
|---|---|---|---|---|---|
| 0 | 22 smoke test | 8/16 · 50% | 4/11 · 36,36% | 22 passed | — |
| 1 | TC-01 | 8/16 · 50% | 4/11 · 36,36% | 1 passed | — |
| 2 | TC-01…04 | 16/16 · 100% | 10/11 · 90,9% | 4 passed | — |
| 3 | TC-01…04 trên bản lỗi | 16/16 · 100% | 10/11 · 90,9% | 4 passed | **lọt** |
| 3' | + TC-05 (biên) | 100% | 90,9% | TC-05 failure | **bị bắt** |

## 6. Góc nhìn nâng cao

### 6.1. «Câu lệnh» của công cụ chưa chắc là câu lệnh của ISTQB

Chạy lại bước 1 và bước 2 **không** có cờ `experimentalAstAwareRemapping` (mặc định của coverage-v8 trong vitest 3.2):

| Bộ test | Mặc định (đếm theo dòng) | Theo cây cú pháp |
|---|---|---|
| TC-01 | Stmts 61,7% (29/47) · Branch 28,57% (2/7) | Stmts 50% (8/16) · Branch 36,36% (4/11) |
| TC-01…04 | Stmts 100% (47/47) · Branch 91,66% (11/12) | Stmts 100% (16/16) · Branch 90,9% (10/11) |

Chế độ mặc định coi **mỗi dòng mã là một «statement»** (47 dòng, gồm cả dòng chỉ có `{` hay tham số), nên con số đẹp hơn thực tế ở bước 1. Mẫu số nhánh còn **đổi giữa hai lần chạy** (7 rồi 12) vì V8 chỉ báo nhánh của những khối mã đã được biên dịch. Bài học: khi báo cáo «statement coverage = X%», ghi kèm **công cụ và cấu hình**; đặt ngưỡng CI trên một cách đếm cố định.

Thêm một sai lệch đo thật trong lab: chạy riêng TC-02 + TC-04 (`-t "TC-0[24]"`), công cụ báo dòng 82 (S16) **đã chạy** dù không test case nào tới đó — TC-02 ném lỗi trước khi gọi `priceRequest`, TC-04 trả về sớm ở nhánh `food`. V8 đo theo khối mã chứ không đặt bộ đếm cho từng câu lệnh, nên phần mã ngay sau một khối `if` có `return` có thể bị suy đoán sai. Con số coverage là **ước lượng của công cụ**; đếm tay một vài hàm quan trọng là cách kiểm chéo rẻ (bài tập 2).

### 6.2. Phủ không phải kiểm

Test gọi `priceRequest` không có `expect` vẫn cho 100%: coverage chứng minh **code đã chạy**, oracle mới chứng minh **code đúng**. Bài 062 dựng ví dụ này; mutation testing (bài 063) đo chất lượng assert.

### 6.3. Những chỗ statement coverage «mù»

| Cấu trúc | Vì sao mù | Kỹ thuật bù |
|---|---|---|
| `if` không có `else` | Đi nhánh đúng là đủ phủ, nhánh sai không bao giờ bị ép | Branch/decision testing (044) |
| `a ? b : c`, `a ?? b`, `a && b` trên một dòng | Một vế chạy là câu lệnh «đã phủ» | Branch (044), condition/MC/DC (045–047) |
| Giá trị nguy hiểm trong câu lệnh đã phủ | Defect phụ thuộc dữ liệu (mục 5.4) | BVA (032), EP (031), property-based (041) |
| Yêu cầu không được cài đặt | Không có câu lệnh để phủ (defect of omission) | Kỹ thuật dựa trên đặc tả, review |
| Vòng lặp chạy 1 lần | Thân vòng đã phủ, 0 lần và nhiều lần chưa thử | Loop testing (049) |

Ví dụ defect thiếu sót ở lab: R-FOOD-06 (đơn đồ ăn gửi `car` ⇒ `VALIDATION_ERROR`) được kiểm ở lớp schema API, `pricing.ts` không có dòng nào cho nó. Nếu lớp schema quên quy tắc, white-box trên `pricing.ts` không thể phát hiện — chỉ test theo đặc tả làm được.

### 6.4. Sai lầm thường gặp

- **Coi 100% là đích.** Viết test «cho đủ dòng», bỏ qua giá trị biên — bước 3 cho thấy cái giá. Coverage là **đèn báo** chỗ thiếu, không phải mục tiêu.
- **Lấy kết quả mong đợi từ code.** Chép số hàm đang trả vào `expect` ⇒ test luôn xanh kể cả khi code sai (TTA 2.1).
- **Gộp coverage nhiều tệp.** «Lab đạt 85%» có thể che một tệp rủi ro cao chỉ 40% — đo theo module.

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 (K3).** Hàm `parcelSurcharge` trong `src/domain/parcel.ts`:

```ts
export function parcelSurcharge(weightKg: number): number {
  const extraTenths = Math.round(weightKg * 10) - FREE_KG * 10;
  if (extraTenths <= 0) return 0;
  return Math.ceil(extraTenths / 10) * SURCHARGE_PER_STARTED_KG;
}
```

(a) Bao nhiêu test tối thiểu cho 100% câu lệnh? (b) Một đồng nghiệp chọn 3 kg và 6,0 kg. Nếu ai đó «tối ưu» `Math.ceil` thành `Math.round`, bộ hai ca này có bắt được không? (c) Sửa bộ test thế nào?

<details>
<summary><b>Lời giải</b></summary>
<p>(a) Bốn câu lệnh (gán, <code>if</code>, hai <code>return</code>); hai <code>return</code> loại trừ nhau ⇒ tối thiểu <b>2 ca</b>: một ca w ≤ 5 kg, một ca w &gt; 5 kg.</p>
<p>(b) Không. 3 kg ⇒ 0 đ (đúng ở cả hai bản). 6,0 kg ⇒ extraTenths = 10 ⇒ <code>ceil(1) = round(1) = 1</code> ⇒ 2.000 đ ở cả hai bản. 100% câu lệnh, defect lọt — lại là defect phụ thuộc dữ liệu.</p>
<p>(c) Dùng giá trị SPEC R-PARCEL-03 đưa ra: 5,1 kg (mong đợi 2.000; bản <code>round</code> cho <code>round(0,1)=0</code> ⇒ 0) hoặc 6,1 kg (mong đợi 4.000; bản <code>round</code> cho 2.000). Kiểm tay trên lab: <code>[5.1, 6, 6.1]</code> ⇒ bản đúng <code>[2000, 2000, 4000]</code>, bản <code>round</code> <code>[0, 2000, 2000]</code>.</p>
</details>

**Bài tập 2 (K3).** Chỉ chạy TC-02 và TC-04. (a) Đếm tay statement coverage của `pricing.ts` theo bảng 4.1. (b) Chạy `npx vitest run articles/statement-coverage/phu-cau-lenh.test.ts -t "TC-0[24]" $COV $INC` và so sánh. Giải thích chênh lệch.

<details>
<summary><b>Lời giải</b></summary>
<p>(a) TC-02 chạy S1, S2, S3. TC-04 chạy S1, S2, S4, S5, S10, S11, S12, S13, S14. Hợp lại 10 câu ⇒ <b>10/16 = 62,5%</b>. Chưa chạy: S6, S7, S8, S9, S15, S16.</p>
<p>(b) Công cụ báo <b>68,75%</b> (11/16), dòng chưa phủ <code>48-52,76</code>: nó coi S16 (dòng 82) đã chạy. Thực tế không ca nào tới dòng 82 (TC-02 ném lỗi ở <code>normalizePriceRequest</code>, TC-04 trả về ở nhánh <code>food</code>). Đây là sai lệch của cách đo theo khối của V8 (mục 6.1) — con số công cụ cao hơn sự thật 6,25 điểm phần trăm.</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Nhận định nào **đúng** về 100% statement coverage?

- a) Bảo đảm mọi defect trong code đã được phát hiện
- b) Bảo đảm mọi nhánh của mọi quyết định đã được đi qua
- c) Bảo đảm mỗi câu lệnh chứa defect đã được chạy ít nhất một lần, nhưng chưa chắc gây ra failure
- d) Bảo đảm mọi yêu cầu trong đặc tả đã được cài đặt

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — đúng ý CTFL 4.3.1 (ví dụ chia cho 0). a) sai: defect phụ thuộc dữ liệu vẫn lọt (mục 5.4). b) sai: đó là branch coverage, mạnh hơn. d) sai: white-box không thấy defect thiếu sót (CTFL 4.3.3).</p>
</details>

**Câu 2 (K3).** Cho hàm (số dòng ở đầu):

```ts
1 function fee(status: string, minutes: number): number {
2   let fee = 0;
3   if (status === 'accepted') {
4     if (minutes >= 2) fee = 10_000;
5   }
6   return fee;
7 }
```

Bộ test chỉ có một ca `fee('accepted', 1)`. Statement coverage (đếm 5 câu lệnh: dòng 2, 3, 4 phần `if`, 4 phần gán, 6) là bao nhiêu?

- a) 60%
- b) 80%
- c) 100%
- d) 40%

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> 4/5 = 80%: chạy <code>let</code>, <code>if</code> ngoài, <code>if</code> trong, <code>return</code>; phép gán <code>fee = 10_000</code> không chạy vì 1 &lt; 2. a) và d) đếm thiếu. c) sai: phép gán chưa chạy.</p>
</details>

**Câu 3 (K3).** Với hàm ở câu 2, số ca tối thiểu để đạt 100% statement coverage là?

- a) 1
- b) 2
- c) 3
- d) 4

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a)</b> Một ca <code>fee('accepted', 5)</code> chạy cả 5 câu lệnh. Đó cũng là minh hoạ vì sao statement coverage yếu: không ca nào kiểm trạng thái khác <code>accepted</code> hay <code>minutes &lt; 2</code>. b), c), d) là số ca cần cho tiêu chí mạnh hơn (branch cần 3 ca), không phải cho statement.</p>
</details>

**Câu 4 (K2).** Theo CTFL v4.0.1, điểm yếu cơ bản mà mọi kỹ thuật white-box cùng có là gì?

- a) Không dùng được khi đặc tả mơ hồ
- b) Không phát hiện được defect do một yêu cầu không được cài đặt
- c) Không cho số đo coverage khách quan
- d) Không áp dụng được trong static testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng (4.3.3, defect of omission). a) ngược lại: white-box mạnh chính khi đặc tả mơ hồ. c) ngược lại: đó là điểm mạnh. d) sai: syllabus nói white-box dùng được khi chạy khô code, pseudocode.</p>
</details>

**Câu 5 (K2).** Đội Pricing đặt ngưỡng 100% statement coverage cho `pricing.ts` nhưng mãi kẹt ở một nhánh `default` của `switch` đã xử lý đủ mọi giá trị. Cách nhìn nào phù hợp với CTAL-TTA?

- a) Viết test gọi hàm với dữ liệu sai kiểu bằng `as any` để ép chạy nhánh đó
- b) Hạ ngưỡng toàn dự án xuống 80%
- c) Đây là mã không tới được; 100% không phải lúc nào cũng đạt được — ghi lý do ngoại lệ hoặc xem lại thiết kế code
- d) Bỏ đo statement coverage vì không đáng tin

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng (TTA 2.2 nêu đúng ví dụ <code>default</code> của <code>switch</code>). a) có thể chạy nhưng chỉ «mua» con số, test đó không kiểm hành vi nào có thật. b) quá tay: đánh mất ngưỡng cho mọi tệp khác. d) sai: thước đo vẫn có giá trị là mức tối thiểu.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024), mục 4.3, 4.3.1, 4.3.3; LO FL-4.3.1 (K2), FL-4.3.3 (K2): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level — Technical Test Analyst Syllabus* **v4.0** (2021), mục 2.1, 2.2; LO TTA-2.2.1 (K3): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (statement testing, statement coverage, white-box test technique).
- ISO/IEC/IEEE 29119-4, *Software and systems engineering — Software testing — Part 4: Test techniques* — đặc tả statement testing (CTAL-TTA dẫn chiếu).
- Vitest — Coverage: https://vitest.dev/guide/coverage
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-VALID-01, R-FARE-02…09, R-PARCEL-01…04, R-FOOD-01…06; mã bài tại `articles/statement-coverage/`.
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-003 · Bảy nguyên lý kiểm thử](/articles/bay-nguyen-ly-kiem-thu) · [TEST-012 · Component Testing](/articles/component-testing-unit-testing-tong-quan) · [TEST-022 · Static và Dynamic Testing](/articles/static-testing-va-dynamic-testing) · [TEST-031 · Equivalence Partitioning](/articles/equivalence-partitioning) · [TEST-032 · Boundary Value Analysis](/articles/boundary-value-analysis) · [TEST-062 · Code Coverage trong thực hành](/articles/code-coverage-trong-thuc-hanh).

Bài tiếp theo: [Branch Coverage / Decision Coverage — phủ mọi nhánh của phí huỷ chuyến](/articles/branch-coverage-decision-coverage).
