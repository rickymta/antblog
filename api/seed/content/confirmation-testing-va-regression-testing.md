---
slug: confirmation-testing-va-regression-testing
title: "Confirmation Testing và Regression Testing: kiểm lại bản sửa lỗi làm tròn quãng đường"
summary: "Phân biệt confirmation testing và regression testing theo CTFL v4.0.1, chọn phạm vi regression bằng impact analysis và chạy thật trên ba bản build sửa lỗi làm tròn quãng đường của Tới Nơi."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-019, confirmation-testing, regression-testing, impact-analysis, vitest, toi-noi]
language: vi
---

Developer báo «đã sửa lỗi làm tròn quãng đường». Tester chạy lại đúng test case từng đỏ: xanh. Có nên đóng phiếu lỗi và phát hành bảng giá mới không? Chưa. Còn câu hỏi thứ hai: bản sửa có làm hỏng chỗ nào khác không. Bài này tách hai câu hỏi đó thành **confirmation testing** và **regression testing**, chỉ cách dùng **impact analysis** để biết phải chạy lại những gì, rồi chạy thật trên ba bản build — trong đó có một bản «sửa xong» vượt qua mọi confirmation test nhưng vẫn làm sai cước.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-019</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S02 · Test Levels &amp; Test Types</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1 (đọc thêm CTAL-TA v4.0)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Component; System</td></tr>
<tr><th scope="row">Test Type</th><td>Confirmation &amp; Regression</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/error-defect-failure-root-cause">TEST-004 · Error, Defect, Failure và Root Cause</a>; <a href="/articles/functional-testing">TEST-017 · Functional Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): sửa lỗi làm tròn half-up R-FARE-02, chọn confirmation test và bộ regression bị ảnh hưởng (R-PARCEL-01, R-FOOD-04), chạy trên ba bản build bằng vitest — <code>examples/toi-noi-testing/articles/confirmation-testing-va-regression-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 2 · mục 2.2.3 · FL-2.2.3 (K2); liên quan mục 1.1.2 (FL-1.1.2), 2.1.4 · Đọc thêm CTAL-TA v4.0 · mục 2.2 · TA-2.2.1 (K4)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Mọi thay đổi mang hai rủi ro: thay đổi **không làm được** điều nó hứa (lỗi vẫn còn, hoặc chỉ hết ở đúng test case đã báo), hoặc **làm được nhưng phá thứ khác** (chức năng từng đúng nay sai). Hai rủi ro cần hai loại test khác nhau.

[TEST-001](/articles/software-testing-la-gi-testing-khac-debugging) đã giới thiệu hai khái niệm ở mức định nghĩa, trên một lỗi làm tròn tiền. Bài này đi tiếp ba bước:

- Confirmation testing có **mấy mức**, chọn mức nào theo rủi ro.
- Regression testing chạy **những gì**, chọn bằng impact analysis.
- Vì sao regression gần như luôn là ứng viên đầu tiên của tự động hoá.

Dùng bài này khi nhận một bản sửa lỗi, một tính năng chạm vào code dùng chung, hoặc khi đội tranh luận «lần này có cần chạy hết bộ regression không».

## 2. Lý thuyết theo ISTQB

CTFL v4.0.1 (mục 2.2.3) mở đầu bằng một quan sát: thay đổi thường là **thêm tính năng** hoặc **sửa defect**. Với cả hai, kiểm thử nên gồm confirmation testing và regression testing.

**Confirmation testing** (kiểm thử xác nhận) — xác nhận defect ban đầu đã được sửa thành công. Tuỳ mức rủi ro, có thể làm theo nhiều cách:

1. chạy lại **mọi test từng thất bại** vì defect đó;
2. **thêm test mới** để phủ những thay đổi mà việc sửa defect cần tới;
3. khi thiếu thời gian hoặc ngân sách: chỉ chạy lại **các bước tái hiện failure** và kiểm failure không còn xảy ra.

Mục 1.1.2 bổ sung: confirmation testing nên do **chính người đã chạy test ban đầu** thực hiện — họ biết rõ điều kiện tái hiện.

**Regression testing** (kiểm thử hồi quy) — xác nhận thay đổi không gây hậu quả bất lợi, **kể cả một bản sửa đã qua confirmation testing**. Hậu quả có thể nằm ở chính component bị sửa, ở component khác trong cùng hệ thống, hoặc ở **hệ thống khác có kết nối**. Regression cũng có thể liên quan tới **môi trường**, không chỉ test object. Syllabus khuyên làm **impact analysis** (phân tích tác động) trước để biết phạm vi regression: phân tích này chỉ ra phần nào của phần mềm có thể bị ảnh hưởng.

Hai ý cuối của mục 2.2.3 dẫn thẳng tới tự động hoá:

- Bộ regression chạy **nhiều lần** và thường **lớn dần** qua mỗi vòng lặp, mỗi bản phát hành ⇒ ứng viên mạnh cho tự động hoá; nên bắt đầu tự động hoá sớm. Với CI/DevOps (mục 2.1.4), nên đưa regression tự động vào pipeline, có thể ở nhiều test level.
- Confirmation và regression cần ở **mọi test level** nơi có defect được sửa hoặc có thay đổi.

| Tiêu chí | Confirmation testing | Regression testing |
|---|---|---|
| Câu hỏi | Defect này đã hết chưa? | Thay đổi có phá chỗ nào khác không? |
| Test lấy từ đâu | Test từng đỏ vì defect + test mới cho phần code bị sửa | Bộ test cũ đang xanh, chọn theo impact analysis |
| Phạm vi | Hẹp, quanh defect | Rộng: component, hệ thống, hệ kết nối, môi trường |
| Kết quả «đạt» nghĩa là | Failure cũ không còn | Không xuất hiện failure mới ở phần không định đổi |
| Số lần chạy | Mỗi bản sửa vài lần | Mỗi thay đổi, mỗi build, mỗi đêm — rất nhiều lần |
| Tự động hoá | Có ích | Gần như bắt buộc khi giao hàng thường xuyên |

<div class="tkh-callout">
<span class="tkh-callout-title">Ghi chú phiên bản</span>
<p>CTFL 2018 gộp hai loại này thành test type «change-related testing». CTFL v4.0.1 bỏ tên đó: mục 2.2.2 nêu bốn test type (functional, non-functional, black-box, white-box), confirmation và regression được trình bày riêng ở 2.2.3.</p>
</div>

### Thuật ngữ Anh – Việt

| Thuật ngữ | Tiếng Việt | Ghi chú |
|---|---|---|
| confirmation testing | kiểm thử xác nhận | tên cũ hay gặp: re-testing |
| regression testing | kiểm thử hồi quy | không đồng nghĩa với «chạy lại tất cả» |
| regression | sự hồi quy | chức năng từng đúng nay sai do thay đổi |
| impact analysis | phân tích tác động | xác định phần có thể bị ảnh hưởng |
| regression test suite | bộ test hồi quy | lớn dần theo thời gian |
| adverse consequence / side effect | hậu quả bất lợi / tác dụng phụ | |
| test automation | tự động hoá kiểm thử | chi tiết ở Series 10 |
| continuous integration (CI) | tích hợp liên tục | nơi regression tự động thường chạy |

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Đọc thêm — CTAL-TA v4.0, mục 2.2 (TA-2.2.1, K4)</span>
<p>Syllabus Test Analyst đi sâu vào việc <b>chọn</b> test hồi quy khi không chạy hết được. Với test tự động, tin cậy nhất là impact analysis có công cụ: ghi lại test nào chạy qua configuration item nào, item đổi thì chọn đúng các test đó. Với test thủ công, các cách hay dùng là chọn theo rủi ro, theo lịch sử lỗi, theo độ phủ, theo ma trận truy vết yêu cầu, theo operational profile — thường kết hợp và xét lại sau mỗi chu kỳ. Bài này dùng truy vết yêu cầu và impact analysis bằng công cụ.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Các quy tắc trong SPEC lab dính tới thay đổi:

- **R-FARE-02** — quãng đường làm tròn tới 0,1 km **half-up** trước mọi bước khác (0,45 ⇒ 0,5; 0,44 ⇒ 0,4; 7,35 ⇒ 7,4), rồi phải nằm trong [0,5; 100] km (100,04 hợp lệ; 100,05 không hợp lệ).
- **R-PARCEL-01** — khối lượng bưu kiện cũng làm tròn 0,1 kg half-up, rồi phải thoả 0 &lt; w ≤ 30 kg (0,04 ⇒ 0,0 không hợp lệ; 30,04 ⇒ 30,0 hợp lệ). **R-PARCEL-03** — phụ phí 2.000 đ cho mỗi kg bắt đầu vượt 5 kg.
- **R-FOOD-04** — phí giao đồ ăn tính theo công thức xe máy của R-FARE, **có áp R-FARE-02**.
- **R-FARE-09** — ví dụ kiểm: xe máy 7,4 km, cao điểm, mưa, `GIAM20` ⇒ 52.000 đ.

### Sự cố (giả định)

Trước đợt phát hành bảng giá `fare-2026.1`, bản build **rc1** có hàm làm tròn viết bằng `Number(v.toFixed(1))`. Tester chạy bộ test cước: test case xe máy 7,35 km lúc 09:00 ra **45.000 đ** thay vì **46.000 đ**. Defect report (cách viết xem [TEST-010](/articles/testware-san-pham-kiem-thu)) ghi triệu chứng: «quãng đường dạng x,x5 km có lúc bị làm tròn xuống». Developer sửa thành rc2, rồi rc3. Câu chuyện của bài là việc kiểm thử hai bản sửa đó.

### System context

Ở kiến trúc đích, làm tròn quãng đường nằm trong **Pricing &amp; Quote Service** (§4.3): mọi báo giá đặt xe, bưu kiện, phí giao đồ ăn đi qua đây; Booking gọi Pricing bằng gRPC và lưu ảnh chụp báo giá kèm `pricing_version`. Regression tự động nằm rải ở các tầng của pipeline §17 (§17.1: unit cho pricing rules, contract Booking ↔ Pricing, system cho luồng đặt chuyến).

Lab là modular monolith: hàm `roundToTenth` ở `src/domain/money.ts` được **dùng chung** bởi `fare.ts` (quãng đường) và `parcel.ts` (khối lượng). Chi tiết «dùng chung» này là mấu chốt của impact analysis bên dưới. Lab chưa có gRPC hay contract test.

### Rủi ro có thể xảy ra

| # | Rủi ro | Hậu quả |
|---|---|---|
| 1 | Bản sửa chỉ hết lỗi ở ca 7,35 km đã báo | Hàng trăm giá trị x,x5 khác vẫn sai — khách bị tính thiếu hoặc thừa |
| 2 | Bản sửa đổi hành vi ở biên [0,5; 100] km | Chuyến 100,04 km bị từ chối, chuyến 0,44 km được nhận |
| 3 | Bản sửa đụng hàm dùng chung | Phụ phí bưu kiện và biên khối lượng đổi theo mà không ai để ý |
| 4 | Phí giao đồ ăn thay đổi mà không ai hay | Đối soát tài chính lệch giữa báo giá và thu tiền |
| 5 | Môi trường build khác (phiên bản Node, ICU) | Hành vi số/định dạng khác giữa máy dev và production |

## 4. Phân tích kỹ thuật

### 4.1 Chọn mức confirmation theo rủi ro

Lỗi chạm vào **tiền của mọi chuyến**, nên chọn mức đầy đủ nhất của syllabus:

| Mức (mục 2.2.3) | Test cụ thể | Lý do |
|---|---|---|
| Tối thiểu: chạy lại bước tái hiện | CF-04 — gọi `POST /api/quotes` xe máy 7,35 km | Đúng thao tác của tester lúc phát hiện failure |
| Chạy lại mọi test từng đỏ | CF-01 (7,35 ⇒ 7,4), CF-02 (46.000 đ), CF-03 (100,05 bị từ chối), CF-04 | Cả bốn đã đỏ trên rc1 |
| Thêm test cho phần code bị sửa | CF-05 (5,05 kg ⇒ 5,1 kg, phụ phí 2.000 đ), CF-06 (quét 996 giá trị 0,45…99,95 dạng x,x5) | Bản sửa đụng hàm dùng chung cho khối lượng; defect thuộc **cả một lớp giá trị**, không chỉ 7,35 |

CF-06 đáng chú ý: thay vì chọn vài giá trị «nghi ngờ», nó so mọi giá trị x,x5 trong miền hợp lệ với kết quả tính bằng số nguyên. Defect làm tròn kiểu này hiếm khi chỉ ở một điểm.

### 4.2 Impact analysis: thay đổi lan tới đâu

Bốn bước, mỗi bước một câu hỏi:

1. **Đổi cái gì?** Thân hàm `roundToTenth` trong `src/domain/money.ts`.
2. **Ai gọi trực tiếp?**

```bash
grep -rn "roundToTenth(" src
```

```text
src/domain/fare.ts:19:  const km = roundToTenth(raw);
src/domain/money.ts:35:export function roundToTenth(value: number): number {
src/domain/parcel.ts:12:  const kg = roundToTenth(raw);
```

3. **Lan tiếp tới đâu?** `normalizeDistanceKm` được `calculateFare`, `food.ts` và `pricing.ts` dùng; `normalizeWeightKg` đi vào phụ phí bưu kiện; cả hai cuối cùng đi qua `quote.ts` ra `POST /api/quotes`.
4. **Ứng với yêu cầu và test nào?** Tra bảng truy vết SPEC Phụ lục B (yêu cầu → mã nguồn) theo chiều ngược.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 330" role="img" aria-label="roundToTenth trong money.ts lan tới normalizeDistanceKm và normalizeWeightKg; từ đó tới calculateFare, priceFood, parcelSurcharge; cả ba tới POST /api/quotes ở cấp hệ thống. Mỗi ô ghi yêu cầu và test được chọn. Dải dưới: phí huỷ, trạng thái, nhận chuyến, thanh toán, idempotency không bị ảnh hưởng.">
<defs>
<marker id="t019-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-b">Impact analysis của bản sửa roundToTenth — từ hàm bị đổi tới test được chọn</text>
<rect x="20" y="120" width="170" height="64" rx="6" class="f-warn-soft s-warn"/>
<text x="105" y="140" text-anchor="middle" class="t-small t-warn">THAY ĐỔI</text>
<text x="105" y="158" text-anchor="middle" class="t-mono">money.ts</text>
<text x="105" y="174" text-anchor="middle" class="t-mono t-b">roundToTenth</text>
<rect x="240" y="60" width="190" height="64" rx="6" class="f-pk-soft s-pk"/>
<text x="335" y="80" text-anchor="middle" class="t-mono">fare.ts</text>
<text x="335" y="96" text-anchor="middle" class="t-mono t-b">normalizeDistanceKm</text>
<text x="335" y="114" text-anchor="middle" class="t-10">R-FARE-02 · RG-01…03</text>
<rect x="240" y="180" width="190" height="64" rx="6" class="f-pk-soft s-pk"/>
<text x="335" y="200" text-anchor="middle" class="t-mono">parcel.ts</text>
<text x="335" y="216" text-anchor="middle" class="t-mono t-b">normalizeWeightKg</text>
<text x="335" y="234" text-anchor="middle" class="t-10">R-PARCEL-01 · RG-05, 06</text>
<rect x="480" y="40" width="200" height="56" rx="6" class="f-surface s-pk"/>
<text x="580" y="62" text-anchor="middle" class="t-mono">fare.ts · calculateFare</text>
<text x="580" y="82" text-anchor="middle" class="t-10">R-FARE-01…09 · CF-02 · RG-04</text>
<rect x="480" y="110" width="200" height="56" rx="6" class="f-surface s-pk"/>
<text x="580" y="132" text-anchor="middle" class="t-mono">food.ts · priceFood</text>
<text x="580" y="152" text-anchor="middle" class="t-10">R-FOOD-04 phí giao · RG-08</text>
<rect x="480" y="200" width="200" height="56" rx="6" class="f-surface s-pk"/>
<text x="580" y="222" text-anchor="middle" class="t-mono">parcel.ts · parcelSurcharge</text>
<text x="580" y="242" text-anchor="middle" class="t-10">R-PARCEL-03 · CF-05 · RG-07</text>
<rect x="730" y="95" width="170" height="96" rx="6" class="f-phys-soft s-phys"/>
<text x="815" y="117" text-anchor="middle" class="t-small t-phys">CẤP HỆ THỐNG</text>
<text x="815" y="137" text-anchor="middle" class="t-mono">pricing.ts → quote.ts</text>
<text x="815" y="155" text-anchor="middle" class="t-mono t-b">POST /api/quotes</text>
<text x="815" y="177" text-anchor="middle" class="t-10">CF-04 · RG-09</text>
<line x1="190" y1="140" x2="238" y2="94" class="s-pk" stroke-width="1.5" marker-end="url(#t019-arrow)"/>
<line x1="190" y1="164" x2="238" y2="210" class="s-pk" stroke-width="1.5" marker-end="url(#t019-arrow)"/>
<line x1="430" y1="84" x2="478" y2="70" class="s-pk" stroke-width="1.5" marker-end="url(#t019-arrow)"/>
<line x1="430" y1="102" x2="478" y2="134" class="s-pk" stroke-width="1.5" marker-end="url(#t019-arrow)"/>
<line x1="430" y1="212" x2="478" y2="226" class="s-pk" stroke-width="1.5" marker-end="url(#t019-arrow)"/>
<line x1="680" y1="68" x2="728" y2="115" class="s-pk" stroke-width="1.5" marker-end="url(#t019-arrow)"/>
<line x1="680" y1="138" x2="728" y2="143" class="s-pk" stroke-width="1.5" marker-end="url(#t019-arrow)"/>
<line x1="680" y1="228" x2="728" y2="175" class="s-pk" stroke-width="1.5" marker-end="url(#t019-arrow)"/>
<rect x="20" y="270" width="880" height="48" rx="6" class="f-surface s-muted dash"/>
<text x="460" y="290" text-anchor="middle" class="t-small">Không có đường tới roundToTenth ⇒ không chọn cho lần chạy nhắm đích (vẫn chạy trong bộ đầy đủ ban đêm)</text>
<text x="460" y="308" text-anchor="middle" class="t-mono t-dead">cancel.ts (R-CANCEL) · trip-state.ts (R-STATE) · trips.ts nhận chuyến (R-ACCEPT) · payment.ts (R-PAY) · idempotency.ts (R-IDEM)</text>
</svg>
</div>
<figcaption><b>Hình 1 · Impact analysis theo đường gọi hàm.</b> Bắt đầu từ hàm bị sửa, đi ngược lên những nơi dùng nó, gắn mỗi nơi với yêu cầu và test. Nhánh khối lượng (parcel.ts) là nhánh dễ bị quên nhất: defect được báo là «quãng đường», nhưng code sửa lại là hàm dùng chung.</figcaption>
</figure>

Mỗi ô trong Hình 1 cho ra một nhóm regression test: biên quãng đường (RG-01…03), cước đầy đủ (RG-04), biên khối lượng (RG-05, 06), phụ phí bưu kiện (RG-07), phí giao đồ ăn (RG-08, 09). Ưu tiên **ví dụ biên có sẵn trong SPEC**: bản sửa hàm làm tròn dễ làm lệch biên nhất.

### 4.3 Không chọn gì, và vì sao

Phí huỷ, máy trạng thái, nhận chuyến, thanh toán, idempotency không có đường tới `roundToTenth` ⇒ không cần cho **lần chạy nhắm đích** trước khi duyệt bản sửa. «Không chọn» không có nghĩa «không bao giờ chạy»: chúng vẫn nằm trong bộ đầy đủ chạy ban đêm, phòng khi impact analysis bỏ sót.

Công cụ cũng làm được impact analysis, ở độ mịn khác. `vitest related` chọn test theo **đồ thị import**:

```bash
npx vitest related src/domain/money.ts --run
```

Lúc viết bài, lệnh chọn **17/20** tệp test của lab — chỉ bỏ ba tệp về phí huỷ và truy vết. Lý do: `money.ts` là module nền, gần như mọi thứ import nó (kể cả `voucher.ts` vì hàm `percentOf`). Phân tích theo module thì an toàn nhưng thô; phân tích theo hàm (Hình 1) mịn hơn nhưng dựa vào người đọc code. Ở hệ thống lớn, công cụ ghi lại test nào chạy qua dòng code nào (CTAL-TA gọi là impact analysis có công cụ hỗ trợ) cho kết quả giữa hai mức này.

### 4.4 Ở test level nào

Confirmation và regression cần ở **mọi test level** có thay đổi. Bộ test của bài có hai: **component** (gọi thẳng hàm domain — nhanh, chỉ đúng chỗ) và **system** trong phạm vi lab (gọi `POST /api/quotes` qua supertest — đi qua kiểm tra đầu vào, use case báo giá, JSON). CF-04, RG-09 bắt được trường hợp hàm đúng mà lớp trên làm sai.

## 5. Thực hành

Mã nằm ở `examples/toi-noi-testing/articles/confirmation-testing-va-regression-testing/`. Không sửa `src/`: mỗi bản build thay `roundToTenth` bằng `vi.mock`, mọi module khác dùng code thật của lab.

### 5.1 Ba bản build

```ts
// rounding-variants.ts
/** rc1 — CÓ DEFECT: toFixed làm tròn theo giá trị nhị phân thật (7,35 lưu là 7,3499…). */
export function roundTenthRc1(value: number): number {
  return Number(value.toFixed(1));
}
/** rc2 — «sửa» lần 1, SAI: hiểu «half-up» thành «làm tròn lên». */
export function roundTenthRc2(value: number): number {
  return Math.ceil(value * 10) / 10;
}
// rc3 = src/domain/money.ts: Math.round(Number((value * 10).toPrecision(12))) / 10
```

```ts
// build-rc2.test.ts — thay hàm của bản build, giữ nguyên mọi thứ khác
vi.mock('../../src/domain/money', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../src/domain/money')>();
  const { roundTenthRc2 } = await import('./rounding-variants');
  return { ...original, roundToTenth: roundTenthRc2 };
});
defineSuite('rc2 (Math.ceil — sửa lần 1)', new Set(['RG-01', 'RG-03', 'RG-05', 'RG-06', 'RG-08', 'RG-09']));
```

Tham số thứ hai là danh sách failure đã biết, được đánh dấu `it.fails` để `npm test` của lab luôn xanh. Đặt `SHOW_RED=1` để thấy màu đỏ thật.

### 5.2 Bộ test và kết quả mong đợi

Kết quả mong đợi tính tay từ SPEC, không chép từ lần chạy code (lý do ở [TEST-001](/articles/software-testing-la-gi-testing-khac-debugging) — oracle phải độc lập với code).

| ID | Loại | Level | Đầu vào | Mong đợi (SPEC) |
|---|---|---|---|---|
| CF-01 | Chạy lại | Component | 7,35 km | 7,4 km |
| CF-02 | Chạy lại | Component | Xe máy 7,35 km, 09:00, không mưa | 12.000 + 4.500 × 7,4 = 45.300 ⇒ **46.000 đ** |
| CF-03 | Chạy lại | Component | 100,05 km | `DISTANCE_OUT_OF_RANGE` |
| CF-04 | Chạy lại | System | `POST /api/quotes` như CF-02 | 200, `distanceKm` 7,4, `total` 46.000 |
| CF-05 | Mới | Component | 5,05 kg | 5,1 kg, phụ phí 2.000 × ⌈0,1⌉ = 2.000 đ |
| CF-06 | Mới | Component | 996 giá trị 0,45 … 99,95 dạng x,x5 | Luôn làm tròn lên: 0 giá trị lệch |
| RG-01…03 | Regression | Component | 0,44 · 0,45 · 100,04 km | `DISTANCE_OUT_OF_RANGE` · 0,5 · 100,0 km |
| RG-04 | Regression | Component | R-FARE-09 | 52.000 đ |
| RG-05, 06 | Regression | Component | 0,04 · 30,04 kg | `WEIGHT_OUT_OF_RANGE` · 30,0 kg |
| RG-07 | Regression | Component | 6,0 kg; 6,1 kg | 2.000 đ; 4.000 đ |
| RG-08 | Regression | Component | Đồ ăn 2,01 km, một món 50.000 đ | 2,0 km ⇒ phí giao 12.000 + 9.000 = **21.000 đ**, tổng 71.000 đ |
| RG-09 | Regression | System | `POST /api/quotes` như RG-08 | 200, `deliveryFee` 21.000, `total` 71.000 |

### 5.3 Chạy thật

```bash
cd examples/toi-noi-testing
SHOW_RED=1 npx vitest run articles/confirmation-testing-va-regression-testing --reporter=verbose
```

Kết quả (rút gọn, chép từ lần chạy):

```text
 × rc1 … CF-01  expected 7.3 to be 7.4
 × rc1 … CF-02  expected 45000 to be 46000
 × rc1 … CF-03  expected 100 to deeply equal { error: 'DISTANCE_OUT_OF_RANGE' }
 × rc1 … CF-04  expected [ 200, 7.3, 45000 ] to deeply equal [ 200, 7.4, 46000 ]
 × rc1 … CF-05  expected [ 5, +0 ] to deeply equal [ 5.1, 2000 ]
 × rc1 … CF-06  398 giá trị lệch: expected [ '0.85⇒0.8', '0.95⇒0.9', '1.15⇒1.1' ] to deeply equal []
 ✓ rc1 … RG-01 … RG-09   (9/9)
 ✓ rc2 … CF-01 … CF-06   (6/6)
 × rc2 … RG-01  expected 0.5 to deeply equal { error: 'DISTANCE_OUT_OF_RANGE' }
 × rc2 … RG-03  expected { error: 'DISTANCE_OUT_OF_RANGE' } to be 100
 × rc2 … RG-05  expected 0.1 to deeply equal { error: 'WEIGHT_OUT_OF_RANGE' }
 × rc2 … RG-06  expected { error: 'WEIGHT_OUT_OF_RANGE' } to be 30
 × rc2 … RG-08  expected [ 22000, 72000 ] to deeply equal [ 21000, 71000 ]
 × rc2 … RG-09  expected [ 200, 22000, 72000 ] to deeply equal [ 200, 21000, 71000 ]
 ✓ rc3 … CF-01 … RG-09   (15/15)
      Tests  12 failed | 33 passed (45)
```

Chạy không có `SHOW_RED` (như CI của lab): `Tests 45 passed (45)`, khoảng 1 giây.

### 5.4 Đọc kết quả

| Bản build | Confirmation (6) | Regression (9) | Quyết định |
|---|---|---|---|
| rc1 — có defect | 0/6 đạt | 9/9 đạt | Mở defect (đã làm) |
| rc2 — sửa lần 1 | **6/6 đạt** | **3/9 đạt** | Đóng defect cũ được, nhưng **mở defect hồi quy mới**, không phát hành |
| rc3 — sửa lần 2 | 6/6 đạt | 9/9 đạt | Đủ điều kiện phát hành (kèm bộ đầy đủ ban đêm xanh) |

Ba điều đáng rút ra:

- **rc2 là cái bẫy kinh điển.** Mọi confirmation test, kể cả CF-06 quét 996 giá trị, đều xanh: `Math.ceil` đúng là đẩy lên mọi giá trị x,x5. Nhưng hàm này cũng đẩy 0,44 và 100,04 — hai ví dụ biên SPEC viết sẵn. Dừng ở confirmation thì Tới Nơi từ chối chuyến 100,04 km và tính phí giao 22.000 đ cho đơn 2,01 km.
- **Regression trên rc1 xanh hết, và đó là thông tin:** defect không lan ra biên. Bộ regression không có nhiệm vụ bắt defect cũ.
- **Khối lượng chỉ lộ ra nhờ test mới.** Bốn ví dụ khối lượng của SPEC (0,04; 0,05; 30,04; 30,05) đều đúng trên rc1; defect chỉ hiện ở 5,05 kg — giá trị không có trong bộ test cũ. Không có impact analysis thì không ai thêm CF-05.

### 5.5 Vì sao bộ này nên tự động

Một vòng sửa lỗi này cần 15 test × 3 bản build = 45 lượt. Giả định mỗi lượt thủ công mất 2 phút: 45 × 2 = **90 phút** người làm, so với khoảng 1 giây tự động. Sau vòng này CF-05, CF-06 **gia nhập bộ regression** — bộ test lớn dần đúng như syllabus mô tả, và chạy lại miễn phí trong CI mỗi khi ai đó chạm vào `money.ts`. Chi phí và rủi ro của tự động hoá bàn ở Series 10.

## 6. Góc nhìn nâng cao

**Confirmation đạt chưa phải bản sửa đúng.** Kết quả này chỉ chứng minh failure cũ không còn. rc2 là ví dụ: bản sửa đúng cho lớp giá trị bị báo, sai cho lớp giá trị khác.

**Bộ regression chỉ có giá trị khi nhạy với thay đổi.** Một bộ toàn ví dụ «đẹp» có thể xanh trên bản lỗi (bốn ví dụ khối lượng trên rc1). Khi impact analysis chỉ ra một vùng bị ảnh hưởng mà chưa test nào phân biệt được bản đúng và bản sai, đó là tín hiệu thêm test — không phải tín hiệu vùng đó an toàn.

**Test mới của confirmation đổi vai sau khi đóng defect.** CF-05, CF-06 là confirmation test của vòng này, nhưng từ vòng sau chúng là regression test. Đặt tên theo yêu cầu (R-PARCEL-03, R-FARE-02) thay vì theo số phiếu lỗi giúp bộ test sống lâu hơn phiếu lỗi.

**Regression có thể đến từ môi trường.** Image production thiếu dữ liệu ICU đầy đủ thì `toLocaleString('vi-VN')` có thể in «52,000 đ» thay vì «52.000 đ». Đổi môi trường cũng cần regression — [TEST-020](/articles/maintenance-testing) bàn tiếp.

**«Chạy lại tất cả» không phải lúc nào cũng sai.** Bộ test của lab chạy hết trong vài giây — chạy toàn bộ là rẻ nhất. Chọn lọc chỉ đáng công khi bộ đầy đủ đắt (E2E trên thiết bị thật, hàng chục service): chọn lọc mỗi lần merge, bộ đầy đủ mỗi đêm.

**Hồi quy trong hệ phân tán.** Ở kiến trúc đích, đổi Pricing có thể làm hỏng Booking (đọc trường `distanceKm` theo nghĩa cũ) hay báo cáo đối soát Payment — hệ «có kết nối» mà syllabus nhắc. Impact analysis khi đó phải đi qua hợp đồng gRPC và schema sự kiện, và regression cần contract test. Lab chưa có phần này; xem Series 07 (TEST-074) và Series 11 (TEST-129).

**Báo giá đã lưu không phải hồi quy.** Báo giá tạo trên rc1 còn hạn vẫn giữ giá cũ khi đặt chuyến trên rc3 — đúng R-QUOTE-04 (không đổi giá hồi tố).

**Sai lầm thường gặp:** gọi «chạy lại test case vừa sửa» là regression; nghĩ unit test của developer thay được confirmation ở system level; chép kết quả của rc3 làm kết quả mong đợi; bỏ qua test regression đỏ vì «không liên quan» mà chưa điều tra (có thể là hồi quy thật, có thể là defect trong test).

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — bản sửa thứ tư.** Một developer khác đề xuất bản rc4: `Math.round(value * 10) / 10`. (a) Bạn chạy những test nào trước khi đồng ý? (b) Dự đoán kết quả với 15 test của bài. (c) Bộ test xanh có chứng minh rc4 đúng không?

<details>
<summary><b>Lời giải</b></summary>
<p>(a) Cả 6 confirmation test (vì đây là bản sửa mới cho cùng defect) và 9 regression test (vì cùng hàm dùng chung), cộng bộ đầy đủ ban đêm. (b) Với mọi đầu vào có tối đa hai chữ số thập phân trong miền của lab, phép nhân 10 rơi đúng vào x,5 hoặc lệch đủ xa nên <code>Math.round</code> cho kết quả half-up đúng — cả 15 test dự kiến xanh (kiểm bằng cách thêm một tệp <code>build-rc4.test.ts</code> theo mẫu rc2). (c) Không. Theo <a href="/articles/bay-nguyen-ly-kiem-thu">nguyên lý 1</a>, test cho thấy có defect chứ không chứng minh không có. Đầu vào nhiều chữ số thập phân hơn vẫn có thể lệch; muốn tự tin hơn cần test sinh dữ liệu ngẫu nhiên so với oracle số nguyên (property-based testing, Series 04/06) và review code.</p>
</details>

**Bài tập 2 — impact analysis cho thay đổi khác.** Developer đổi cách tính của hàm `percentOf` trong `src/domain/money.ts` (dùng cho ×1,25 cao điểm và percent% của voucher). Liệt kê yêu cầu bị ảnh hưởng và chọn tối thiểu một regression test cho mỗi yêu cầu, có kết quả mong đợi.

<details>
<summary><b>Lời giải</b></summary>
<p><code>grep -rn "percentOf" src</code> cho hai nơi gọi: <code>fare.ts</code> (cao điểm) và <code>voucher.ts</code> (tiền giảm). Yêu cầu: R-FARE-04, R-PARCEL-04 (phụ phí bưu kiện bị nhân cao điểm), R-FARE-06/R-VOUCHER-06, R-FOOD-05. Test gợi ý: R-FARE-09 ⇒ 52.000 đ (phủ cả cao điểm lẫn voucher); xe máy 7,4 km cao điểm, không mưa ⇒ 45.300 × 1,25 = 56.625 ⇒ 57.000 đ; <code>GIAM20</code> trên A = 200.000 đ ⇒ giảm 30.000 đ (trần), không phải 40.000 đ; đồ ăn tiền món 100.000 đ với <code>FOOD15</code> ⇒ giảm 15.000 đ. R-FARE-02, R-PARCEL-01 không đi qua <code>percentOf</code>.</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K2).** Tester báo defect: đơn bưu kiện 5,05 kg không bị tính phụ phí. Sau khi developer sửa, tester chạy lại đúng test case 5,05 kg và kiểm phụ phí là 2.000 đ. Hoạt động này là gì?

- a) Regression testing
- b) Confirmation testing
- c) Debugging
- d) Maintenance testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — chạy lại test từng thất bại để xác nhận defect đã hết. a) regression kiểm phần <i>khác</i> có bị ảnh hưởng không. c) debugging là tái hiện, chẩn đoán, sửa — do developer làm trước bước này. d) maintenance testing là kiểm thử thay đổi trên hệ thống đang vận hành; có thể bao gồm confirmation, nhưng không phải tên đúng nhất cho hoạt động được mô tả.</p>
</details>

**Câu 2 (K2).** Sau bản sửa lỗi làm tròn quãng đường, mọi test từng đỏ đều xanh, nhưng chuyến 100,04 km giờ bị từ chối dù trước đó vẫn đặt được. Phát biểu nào đúng?

- a) Confirmation testing thất bại vì defect cũ chưa được sửa
- b) Đây là failure do hồi quy, loại failure mà regression testing nhắm tới
- c) Không cần ghi nhận vì nằm ngoài phạm vi defect đã báo
- d) Đây là failure do môi trường nên không phải trách nhiệm của bản sửa

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — chức năng từng đúng nay sai do thay đổi: hậu quả bất lợi mà regression testing tìm, kể cả với bản sửa đã qua confirmation. a) sai: các test từng đỏ đã xanh. c) sai: hồi quy phải được ghi nhận thành defect mới. d) sai: nguyên nhân là code thay đổi, không phải môi trường.</p>
</details>

**Câu 3 (K1).** Theo CTFL v4.0.1, vì sao regression testing là ứng viên mạnh cho tự động hoá?

- a) Vì regression test luôn tìm được nhiều defect hơn test mới
- b) Vì bộ regression được chạy nhiều lần và thường lớn dần qua mỗi vòng lặp hoặc bản phát hành
- c) Vì regression test không cần kết quả mong đợi
- d) Vì chỉ có thể làm regression ở mức component

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — đúng lý do syllabus nêu ở mục 2.2.3. a) không có khẳng định như vậy. c) sai: mọi test cần kết quả mong đợi. d) sai: regression cần ở mọi test level có thay đổi.</p>
</details>

**Câu 4 (K2).** Đội Tới Nơi nâng phiên bản Node.js của image Pricing, không đổi dòng code nào. Phát biểu nào phù hợp với CTFL v4.0.1?

- a) Không cần regression testing vì code không đổi
- b) Chỉ cần confirmation testing cho các defect đang mở
- c) Regression testing có thể cần vì hồi quy có thể liên quan tới môi trường, không chỉ test object
- d) Chỉ cần kiểm thử hiệu năng vì phiên bản runtime chỉ ảnh hưởng tốc độ

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — syllabus nói regression testing không chỉ giới hạn ở test object mà có thể liên quan tới môi trường. a) bỏ qua rủi ro môi trường (vd định dạng số «52.000 đ» phụ thuộc dữ liệu locale của runtime). b) không có defect nào được sửa. d) đổi runtime có thể làm đổi cả hành vi chức năng.</p>
</details>

**Câu 5 (K2).** Bản sửa defect làm tròn phải lên production trong ngày, đội chỉ còn 15 phút để kiểm. Theo syllabus, cách làm confirmation testing tối thiểu chấp nhận được là gì?

- a) Bỏ qua confirmation testing, chỉ chạy regression
- b) Chạy các bước tái hiện failure do defect gây ra và kiểm failure không còn xảy ra
- c) Thêm test mới cho mọi module dùng hàm làm tròn
- d) Để developer tự xác nhận vì họ hiểu bản sửa nhất

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — khi thiếu thời gian hoặc tiền, syllabus cho phép thu hẹp confirmation về chạy lại các bước tái hiện failure. a) bỏ mất câu hỏi «đã hết lỗi chưa». c) là mức mở rộng, không phải tối thiểu. d) syllabus ưu tiên người đã chạy test ban đầu thực hiện confirmation.</p>
</details>

**Câu 6 (K2).** Trước khi chọn bộ regression cho bản sửa hàm <code>roundToTenth</code>, tester liệt kê mọi nơi gọi hàm này và các yêu cầu liên quan. Mục đích chính của việc đó là gì?

- a) Tìm nguyên nhân gốc của defect
- b) Xác định phạm vi regression testing thông qua impact analysis
- c) Đo độ phủ câu lệnh của bộ test
- d) Ước lượng thời gian phát triển bản sửa

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — impact analysis chỉ ra phần có thể bị ảnh hưởng để quyết định phạm vi regression. a) là root cause analysis, phục vụ phòng ngừa. c) độ phủ là việc khác (Series 05). d) không liên quan tới chọn test.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024), mục 2.2.3 «Confirmation Testing and Regression Testing» (FL-2.2.3, K2); liên quan mục 1.1.2 «Testing and Debugging», 2.1.4 «DevOps and Testing»: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level — Test Analyst (CTAL-TA)* **v4.0** (2025), mục 2.2 «Risk Control», TA-2.2.1 (K4) «Analyze the impact of changes to determine the scope of regression testing»: https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/ — mục *confirmation testing*, *regression testing*, *impact analysis*.
- Trang chủ ISTQB: https://www.istqb.org/
- Vitest — CLI (`vitest related`): https://vitest.dev/guide/cli · API `vi.mock`: https://vitest.dev/api/vi — công cụ của lab, không thuộc ISTQB.
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-FARE-02, R-FARE-09, R-PARCEL-01/03, R-FOOD-04, R-QUOTE-04, Phụ lục B; mã bài tại `articles/confirmation-testing-va-regression-testing/`.
- Tài liệu kiến trúc tổng thể Tới Nơi: §4.3 Pricing &amp; Quote, §17 CI/CD, §17.1 testing strategy.
- Bài liên quan: [TEST-001 · Testing khác Debugging](/articles/software-testing-la-gi-testing-khac-debugging) · [TEST-003 · Bảy nguyên lý kiểm thử](/articles/bay-nguyen-ly-kiem-thu) · [TEST-004 · Error, Defect, Failure](/articles/error-defect-failure-root-cause) · [TEST-008 · Kiểm thử trong SDLC](/articles/kiem-thu-trong-sdlc-agile-waterfall) · [TEST-010 · Testware](/articles/testware-san-pham-kiem-thu) · [TEST-012 · Component Testing](/articles/component-testing-unit-testing-tong-quan) · [TEST-014 · System Testing](/articles/system-testing) · [TEST-017 · Functional Testing](/articles/functional-testing).

Bài tiếp theo: [TEST-020 · Maintenance Testing — kiểm thử khi Tới Nơi đang chạy mà phải thay đổi](/articles/maintenance-testing).
