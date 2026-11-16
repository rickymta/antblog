---
slug: system-testing-regression-test
title: "Regression Test: giữ hành vi đúng sau mỗi thay đổi của Tới Nơi"
summary: "Regression testing qua một refactor công thức cước giả lập: lập impact map từ hàm dùng chung tới đặt xe, bưu kiện và đồ ăn, chọn input lộ tương tác cao điểm × mưa, tính oracle tiền độc lập, chứng minh suite phân biệt được một fault và quản lý suite lâu dài (selection, flaky quarantine, mutation)."
category: testing
level: intermediate
tags: [testing, sys-06, regression-testing, impact-analysis, test-selection, test-oracle, mutation-testing, flaky-test, vitest, toi-noi]
language: vi
---

Đội Tới Nơi muốn dọn code tính cước: phụ phí trời mưa và phụ phí khối lượng bưu kiện «trông giống nhau», vậy gom về một chỗ. Reviewer chạy thử chuyến xe máy 5 km lúc 9 giờ sáng trời mưa: 43.000 đồng, đúng như trước. Refactor được duyệt. Đến trưa, khách đặt xe lúc 11:30 trời mưa bị tính 54.000 thay vì 52.000 — phụ phí trời mưa đã lọt vào phần bị nhân hệ số cao điểm. Test case dùng để review không sai; nó chỉ **không có khả năng** làm lộ lỗi này. Bài này trả lời: **sau một thay đổi, chọn test case nào để phát hiện hành vi cũ bị hỏng, oracle lấy từ đâu, và làm sao biết bộ regression thực sự bắt được lỗi.**

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-06</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 1</td></tr>
<tr><th scope="row">Cấp độ</th><td>Intermediate — cần biết test oracle, assertion, Vitest + supertest; phần 8–9 dành cho QA/SDET duy trì gate CI</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>Regression là mục đích đánh giá ảnh hưởng thay đổi, không phải test level: bài dùng component test cho hàm cước và API test cho ba consumer (ride, parcel, food)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-system-test">SYS-05 · System Test</a>; nền tảng khái niệm: <a href="/articles/confirmation-testing-va-regression-testing">Confirmation Testing và Regression Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): 8 test case RG-01…RG-08 (8 lượt chạy) cho refactor công thức cước giả lập, kiểm R-FARE-03/04/05/08/09, R-PARCEL-03/04, R-FOOD-04/05 — <code>examples/toi-noi-testing/articles/system-testing-regression-test/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>ISTQB CTFL v4.0.1 (mục 2.2.3 confirmation/regression); Stryker — mutant states and metrics; <code>SPEC.md</code> của lab</td></tr>
</tbody>
</table>
</div>

## 1. Tình huống, đối tượng và mục tiêu

### 1.1. Một refactor «không đổi hành vi»

Công thức cước của Tới Nơi có thứ tự cố định (R-FARE-03): cước cơ bản cộng phụ phí khối lượng → nhân hệ số cao điểm → cộng phụ phí trời mưa → trừ voucher → nâng lên cước tối thiểu → làm tròn lên 1.000 đ. Hai khoản phụ phí ở **hai bước khác nhau**: phí khối lượng bưu kiện bị nhân 1,25 lúc cao điểm (R-PARCEL-04), còn phí mưa 8.000 đ thì không (R-FARE-05).

Refactor giả định gom phí mưa vào chung chỗ với phí khối lượng — tức là đưa nó lên bước 1, không đổi đặc tả. Nhưng `calculateFare` không chỉ phục vụ đặt xe (xem mục 3.1): một dòng sửa trong hàm dùng chung có thể làm sai ở nơi không ai nhìn vào diff.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Lỗi giả lập phục vụ học tập</span>
<p>Hàm <code>calculateFare</code> trong lab hiện tính đúng thứ tự. Bài <b>không</b> khẳng định repository từng có refactor sai hay đã sửa một bug thật. Bản refactor lỗi là hàm <code>buggyRainRefactor</code> đặt ngay trong file test của bài; mã sản phẩm ở <code>src/</code> không bị thay đổi, không có build lỗi nào được deploy.</p>
</div>

### 1.2. Đối tượng và mục tiêu

Bài dành cho tester/QA đã đọc [SYS-03 · Functional Test](/articles/system-testing-functional-test) và [SYS-04 · Integration Test](/articles/system-testing-integration-test): biết tính oracle tiền bằng tay, đọc được `toMatchObject`, chạy được Vitest trong thư mục lab. Sau bài này, bạn có thể:

- lập impact map cho thay đổi ở hàm dùng chung, theo dữ liệu chứ không theo tên file;
- chọn input **kích hoạt** được fault và tính oracle tiền độc lập từ SPEC;
- phân biệt *expected change* (thay đổi được phê duyệt) với *regression*;
- đọc 8 lượt pass và nói đúng những gì chúng không chứng minh;
- đề xuất cách duy trì suite theo rủi ro.

## 2. Khái niệm và những điều dễ nhầm

### 2.1. Định nghĩa dùng trong series

**Regression testing** kiểm xem một thay đổi có làm hỏng hành vi vốn đang đúng hay không. Thay đổi không chỉ là code: cấu hình, schema, dependency, môi trường, dữ liệu tham chiếu (bảng giá, voucher seed) đều có thể gây regression. Khái niệm theo CTFL v4.0.1 đã có ở bài [Confirmation Testing và Regression Testing](/articles/confirmation-testing-va-regression-testing); ở đây ta tập trung vào thực hành: **chọn gì, kiểm bằng oracle nào, và tin kết quả tới đâu.**

Test case functional RG-04 viết cho R-FARE-03/05 trở thành regression khi được chạy để đánh giá ảnh hưởng của refactor: vai trò do **mục đích chạy** quyết định, không phải thư mục chứa test.

### 2.2. Bốn hoạt động dễ nhầm

| Hoạt động | Câu hỏi trả lời | Ví dụ Tới Nơi |
|---|---|---|
| Smoke ([SYS-01](/articles/system-testing-smoke-test)) | Build/môi trường có đủ hoạt động để kiểm tiếp? | Đặt → nhận → hoàn tất → thu tiền được |
| Sanity ([SYS-02](/articles/system-testing-sanity-test)) | Thay đổi cụ thể có hợp lý để đưa vào kiểm thử chuyên sâu? | Trim mã voucher, mã rỗng, consumer của mã chuẩn |
| Confirmation | Lỗi đã báo còn tái hiện không? | Mã có khoảng trắng dùng được sau sửa |
| Regression | Hành vi khác có bị hỏng bởi thay đổi? | Refactor cước làm sai phí giao đồ ăn lúc cao điểm |

Confirmation pass không chứng minh vùng khác an toàn. Regression cũng không bắt buộc chạy mọi test case — có thể chọn theo impact và rủi ro, **miễn là công bố phần bỏ lại**. «Full regression» nghĩa là chạy đủ **suite đã định nghĩa**, không phải đã kiểm mọi hành vi của sản phẩm.

### 2.3. Expected change hay regression?

Nếu yêu cầu là tăng phí mở cửa xe máy từ 12.000 lên 13.000, giá mới khác baseline **là đúng**. Output khác hôm qua chưa chắc là defect; phải đối chiếu với thay đổi đã phê duyệt: hành vi nào được đổi, hành vi nào phải giữ. Refactor trong bài không đổi SPEC, nên bất kỳ khác biệt nào cũng là regression.

Ngược lại, đừng lấy output của phiên bản cũ làm chuẩn vàng. Baseline cũ có thể chứa bug; «đóng băng» nó vào kết quả mong đợi là đóng băng luôn bug. Oracle phải đến từ đặc tả.

## 3. Impact analysis trước khi chọn test

### 3.1. Lần theo dữ liệu, không lần theo file

Diff giả định chỉ chạm `src/domain/fare.ts`. Đọc callers thật trong lab:

- `src/domain/pricing.ts` gọi `calculateFare` cho ride và parcel, truyền `parcelSurcharge` tính từ khối lượng;
- `src/domain/food.ts` gọi `calculateFare` với `vehicle: 'bike'` để ra **phí giao** (R-FOOD-04), sau đó cộng với tiền món đã trừ voucher;
- kết quả giá được chụp vào báo giá, chép sang chuyến (R-QUOTE-04) và là số tiền đi tới bước thu VíXanh.

```ts
// src/domain/pricing.ts — consumer ride/parcel
const fare = calculateFare({
  vehicle: request.vehicle,
  distanceKm: request.distanceKm,
  parcelSurcharge: request.parcelWeightKg === null ? 0 : parcelSurcharge(request.parcelWeightKg),
  ...conditions,
});
```

```ts
// src/domain/food.ts — consumer food: phí giao dùng công thức xe máy
const deliveryFee = calculateFare({
  vehicle: 'bike',
  distanceKm: input.distanceKm,
  peak: input.peak,
  raining: input.raining,
});
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 270" role="img" aria-label="Impact map của refactor công thức cước. Ở giữa trái là hàm calculateFare, nơi thay đổi. Ba mũi tên đi ra: pricing.ts cho ride, pricing.ts cho parcel có phụ phí khối lượng, và food.ts dùng để tính phí giao. Cả ba dẫn tới báo giá, chuyến và thanh toán. Bên dưới các ca kiểm được gắn: RG-01 đến RG-05 ở cấp hàm, RG-06 cho ride qua API, RG-07 cho parcel qua API, RG-08 cho food qua API. Các suite khác SYS-03, SYS-04, SYS-01, SYS-05 phủ phần báo giá, chuyến, lưu SQL, thu ví và giao diện.">
<defs>
<marker id="sys06-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<rect x="16" y="96" width="160" height="56" rx="8" class="f-warn-soft s-warn"/><text x="96" y="120" text-anchor="middle" class="t-b">calculateFare</text><text x="96" y="139" text-anchor="middle" class="t-small t-warn">fare.ts · thay đổi</text>
<rect x="240" y="22" width="210" height="48" rx="8" class="f-pk-soft s-pk"/><text x="345" y="43" text-anchor="middle">pricing.ts · ride</text><text x="345" y="60" text-anchor="middle" class="t-small">RG-06 qua API</text>
<rect x="240" y="100" width="210" height="48" rx="8" class="f-pk-soft s-pk"/><text x="345" y="121" text-anchor="middle">pricing.ts · parcel</text><text x="345" y="138" text-anchor="middle" class="t-small">+ phí khối lượng · RG-07</text>
<rect x="240" y="178" width="210" height="48" rx="8" class="f-pk-soft s-pk"/><text x="345" y="199" text-anchor="middle">food.ts · phí giao</text><text x="345" y="216" text-anchor="middle" class="t-small">voucher trên tiền món · RG-08</text>
<line x1="176" y1="112" x2="236" y2="50" class="s-pk" stroke-width="1.6" marker-end="url(#sys06-arrow)"/>
<line x1="176" y1="124" x2="236" y2="124" class="s-pk" stroke-width="1.6" marker-end="url(#sys06-arrow)"/>
<line x1="176" y1="136" x2="236" y2="198" class="s-pk" stroke-width="1.6" marker-end="url(#sys06-arrow)"/>
<rect x="520" y="60" width="170" height="48" rx="8" class="f-phys-soft s-phys"/><text x="605" y="81" text-anchor="middle">báo giá → chuyến</text><text x="605" y="98" text-anchor="middle" class="t-small">snapshot · SYS-03</text>
<rect x="520" y="140" width="170" height="48" rx="8" class="f-phys-soft s-phys"/><text x="605" y="161" text-anchor="middle">lưu SQL · thu ví</text><text x="605" y="178" text-anchor="middle" class="t-small">SYS-04 · SYS-01</text>
<rect x="740" y="100" width="164" height="48" rx="8" class="f-phys-soft s-phys"/><text x="822" y="121" text-anchor="middle">giao diện</text><text x="822" y="138" text-anchor="middle" class="t-small">SYS-05 · Playwright</text>
<line x1="450" y1="46" x2="516" y2="78" class="s-phys" stroke-width="1.4" marker-end="url(#sys06-arrow)"/>
<line x1="450" y1="124" x2="516" y2="92" class="s-phys" stroke-width="1.4" marker-end="url(#sys06-arrow)"/>
<line x1="450" y1="202" x2="516" y2="170" class="s-phys" stroke-width="1.4" marker-end="url(#sys06-arrow)"/>
<line x1="605" y1="108" x2="605" y2="136" class="s-phys" stroke-width="1.4" marker-end="url(#sys06-arrow)"/>
<line x1="690" y1="164" x2="736" y2="132" class="s-phys dash" stroke-width="1.4" marker-end="url(#sys06-arrow)"/>
<text x="96" y="182" text-anchor="middle" class="t-small">RG-01…05</text><text x="96" y="198" text-anchor="middle" class="t-small">cấp hàm</text>
<text x="460" y="258" text-anchor="middle" class="t-small t-warn">chưa chọn trong bài: phí huỷ, cước tối thiểu với voucher lớn, ô tô, mọi thành phố, hiệu năng</text>
</svg>
</div>
<figcaption><b>Hình 1 · Impact map của refactor công thức cước.</b> Hàm dùng chung có ba consumer; mỗi consumer có một ca API riêng. Phần sau báo giá được phủ bởi các bài trước trong series, chạy như suite riêng — bài này không lặp lại chúng.</figcaption>
</figure>

### 3.2. Từ vùng ảnh hưởng tới suite

| Vùng ảnh hưởng | Rủi ro | Suite nên chọn và lý do |
|---|---|---|
| Công thức base/peak/rain | Sai thứ tự cộng/nhân | RG-01…04: bốn tổ hợp để tách từng điều kiện |
| Voucher trên cước | Giảm trên sai số tiền | RG-06 (oracle chuẩn R-FARE-09); FN-cases của SYS-03 |
| Bưu kiện | Nhầm phí khối lượng với phí mưa | RG-07: có cả hai khoản cùng lúc |
| Đồ ăn | Voucher áp nhầm vào phí giao | RG-08: kiểm riêng tiền món và phí giao |
| Snapshot giá chuyến | Giá đặt khác giá báo | Ca snapshot của SYS-03 |
| Lưu SQL và thu ví | Mapping/amount sai qua adapter | SYS-04, smoke SYS-01 |
| Hiển thị/luồng người dùng | UI không hiện kết quả cuối | SYS-05, chạy riêng qua Playwright |

Đây là mapping **thiết kế thủ công**, không phải công cụ phân tích dependency đã triển khai trong repo. Nếu diff chạm schema, xác thực hoặc tiện ích tiền tệ dùng chung (`money.ts`), mở rộng suite tương ứng. Không chắc impact thì chạy rộng hơn. **«Không có test nào được chọn» không bao giờ được hiểu là pass.**

## 4. Test basis và oracle độc lập

### 4.1. Yêu cầu dùng làm test basis

| Mã | Nội dung liên quan |
|---|---|
| R-FARE-01 | Xe máy: mở cửa 12.000, 4.500/km, tối thiểu 15.000; ô tô: 25.000, 11.000/km, tối thiểu 30.000 |
| R-FARE-03 | Thứ tự tính cố định sáu bước |
| R-FARE-04 | Cao điểm [11:00; 13:00) và [17:00; 19:00) giờ VN ⇒ nhân 1,25 |
| R-FARE-05 | Mưa ⇒ cộng 8.000, **sau** hệ số cao điểm, không bị nhân |
| R-FARE-08 | Làm tròn **lên** bội 1.000 ở bước cuối |
| R-FARE-09 | Ví dụ oracle: xe máy 7,4 km, cao điểm, mưa, giảm 20% |
| R-PARCEL-03/04 | Phí khối lượng `2.000 × ⌈w − 5⌉`, cộng vào cước cơ bản nên **bị nhân** cao điểm |
| R-FOOD-04/05 | Phí giao theo công thức xe máy, không trừ voucher; voucher tính trên tiền món |

### 4.2. Oracle tính tay: vì sao một test case đúng chưa đủ

Xe máy 5 km: cước cơ bản = 12.000 + 4.500 × 5 = **34.500**.

| Điều kiện | Cách tính đúng (R-FARE-03) | Đúng | Candidate sai: (34.500 + mưa) × hệ số |
|---|---|---|---|
| Thường, khô | 34.500 → 35.000 | 35.000 | 35.000 |
| Thường, mưa | 34.500 + 8.000 = 42.500 → 43.000 | 43.000 | 43.000 |
| Cao điểm, khô | 34.500 × 1,25 = 43.125 → 44.000 | 44.000 | 44.000 |
| Cao điểm, mưa | 43.125 + 8.000 = 51.125 → 52.000 | **52.000** | 42.500 × 1,25 = 53.125 → **54.000** |

Ba dòng đầu **không phân biệt** được bản đúng và bản sai. Chỉ khi hai điều kiện cùng xảy ra, vị trí của phép cộng mới đổi kết quả. Thêm test case «gần giống nhau» không làm suite mạnh hơn; chọn đúng **input kích hoạt** fault mới làm được việc đó.

### 4.3. Oracle cho ba consumer

**Ride, R-FARE-09** (7,4 km, cao điểm, mưa, GIAM20): cơ bản 12.000 + 4.500 × 7,4 = 45.300; × 1,25 = 56.625; + 8.000 = **64.625**; giảm 20% = **12.925** (trần GIAM20 30.000 không chạm); còn 51.700 → **52.000**.

**Parcel 6 kg**, xe máy 5 km, cao điểm, mưa: phí khối lượng 2.000 × ⌈6 − 5⌉ = **2.000**; base 34.500 + 2.000 = **36.500**; × 1,25 = **45.625**; + 8.000 = 53.625 → **54.000**. 54.000 ở đây là giá **đúng** cho bưu kiện — trùng con số với output sai của ride. Đừng lấy kết quả mong đợi của ride áp cho parcel chỉ vì cùng 5 km; mỗi consumer cần oracle riêng.

**Food**, tiền món 60.000, 5 km, cao điểm, mưa, FOOD15 (15%, trần 25.000, tối thiểu 50.000, chỉ `food`): giảm món 15% × 60.000 = **9.000**; phí giao theo công thức xe máy cao điểm + mưa = **52.000**, không áp voucher; tổng (60.000 − 9.000) + 52.000 = **103.000**; và `fare.discount` của phần phí giao phải bằng **0**. Test assert từng thành phần, không chỉ tổng.

## 5. Ma trận test và fault fixture

### 5.1. Bảng test case

| Case | Requirement/rủi ro | Tiền điều kiện | Input/hành động | Mong đợi | Cleanup |
|---|---|---|---|---|---|
| RG-01 | R-FARE-03 · baseline | Hàm thuần | bike 5 km, thường, khô | total 35.000 | Không cần |
| RG-02 | R-FARE-05 · mưa độc lập | Hàm thuần | bike 5 km, thường, mưa | 43.000 | Không cần |
| RG-03 | R-FARE-04 · hệ số độc lập | Hàm thuần | bike 5 km, cao điểm, khô | 44.000 | Không cần |
| RG-04 | R-FARE-03/05 · tương tác | Hàm thuần | bike 5 km, cao điểm, mưa | 52.000 | Không cần |
| RG-05 | Khả năng phân biệt fault | Fixture trong file test | Chạy candidate sai ở thường/mưa và cao điểm/mưa; chạy hàm thật | 43.000; 54.000 ≠ 52.000; hàm thật 52.000 | Không cần |
| RG-06 | R-FARE-09 · ride qua API | App mới; 11:30 VN 04/11/2026; SG mưa; C001; GIAM20 chưa dùng | POST báo giá ride 7,4 km + GIAM20 | beforeDiscount 64.625; discount 12.925; total 52.000 | App chạy trong bộ nhớ, bỏ sau test case |
| RG-07 | R-PARCEL-03/04 · parcel | Như trên | POST báo giá parcel 6 kg, bike 5 km | parcelSurcharge 2.000; base 36.500; afterPeak 45.625; rain 8.000; total 54.000 | Như trên |
| RG-08 | R-FOOD-04/05 · food | Như trên; FOOD15 chưa dùng | POST báo giá food 60.000 đ, 5 km + FOOD15 | itemsDiscount 9.000; deliveryFee 52.000; total 103.000; fare.discount 0 | Như trên |

RG-01…04 chạy bằng `it.each` nên runner đếm 4 lượt; cộng RG-05…08 là **8 lượt**.

### 5.2. Fault fixture: đặt lỗi ở đâu

Candidate sai nằm trong file test, mô phỏng refactor «gom phí mưa vào phụ phí ở bước 1»:

```ts
// Candidate lỗi GIẢ LẬP: refactor nhầm phí mưa thành phụ phí trong base.
// Không sửa implementation dùng chung.
function buggyRainRefactor(input: FareInput) {
  return calculateFare({ ...input, raining: false,
    parcelSurcharge: (input.parcelSurcharge ?? 0) + (input.raining ? 8_000 : 0) });
}
```

Nó tắt cờ `raining` rồi nhét 8.000 vào `parcelSurcharge` — trường mà R-PARCEL-04 nói sẽ bị nhân hệ số cao điểm. Phần còn lại dùng hàm sản phẩm, nên hai bên chỉ khác ở vị trí của phí mưa.

### 5.3. Môi trường và độc lập giữa các test case

Test case hàm thuần không có state bên ngoài. Mỗi test case API dựng **một hệ thống mới chạy trong bộ nhớ** trong `beforeEach`: đồng hồ cố định 11:30 giờ VN ngày 04/11/2026 (trong khung cao điểm R-FARE-04), Sài Gòn đang mưa, seed gồm C001, GIAM20, FOOD15. Không có DB dùng chung, không chia sẻ instance, nên thứ tự chạy hay chạy lẻ đều cho cùng kết quả.

### 5.4. Entry criteria và exit criteria

**Entry criteria:** thay đổi và danh sách hành vi phải giữ được nêu rõ (ở đây: không đổi SPEC); test basis xác định; dependency lab cài theo lockfile; không dùng dữ liệu production.

**Exit criteria của bài:** 8 lượt pass, không skip; typecheck đạt; phạm vi chưa chạy được ghi trong báo cáo. Một suite xanh là đầu vào cho quyết định release, **không tự là** quyết định release.

## 6. Thực hành có thể chạy lại

### 6.1. Lệnh chạy

Mã: [`regression.test.ts`](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-regression-test). Node ≥ 22.12, từ `examples/toi-noi-testing`:

```sh
npm ci
npm test -- articles/system-testing-regression-test
npm run typecheck
```

Chỉ cần `npm ci` khi chưa có dependency. Không cần DB hay server ngoài.

### 6.2. Bước 1 — bốn tổ hợp tách từng điều kiện

```ts
it.each([
  { id: 'RG-01', peak: false, raining: false, total: 35_000 },
  { id: 'RG-02', peak: false, raining: true, total: 43_000 },
  { id: 'RG-03', peak: true, raining: false, total: 44_000 },
  { id: 'RG-04', peak: true, raining: true, total: 52_000 },
])('$id · R-FARE-03/05: peak=$peak rain=$raining', ({ peak, raining, total }) => {
  expect(calculateFare({ vehicle: 'bike', distanceKm: 5, peak, raining }).total).toBe(total);
});
```

Kết quả mong đợi là hằng số tính tay ở mục 4.2, không phải `calculateFare` của một phiên bản khác; bảng bốn dòng cho thấy ngay tổ hợp nào đã được phủ.

### 6.3. Bước 2 — chứng minh oracle phân biệt được fault

```ts
it('RG-05 · fixture: một ca thường không phát hiện lỗi, ca tương tác phát hiện được', () => {
  const input: FareInput = { vehicle: 'bike', distanceKm: 5, peak: false, raining: true };
  expect(buggyRainRefactor(input).total).toBe(43_000);
  expect(buggyRainRefactor({ ...input, peak: true }).total).toBe(54_000);
  expect(buggyRainRefactor({ ...input, peak: true }).total).not.toBe(52_000);
  expect(calculateFare({ ...input, peak: true }).total).toBe(52_000);
});
```

Ở tổ hợp thường/mưa, candidate sai và hàm thật **cùng** cho 43.000 — reviewer chỉ thử tổ hợp này sẽ duyệt refactor. Ở cao điểm/mưa, hai bên tách ra: 54.000 so với 52.000. RG-05 xanh nghĩa là «oracle của RG-04 **sẽ** bắt được fault này nếu nó đi vào sản phẩm», không phải «sản phẩm có lỗi».

### 6.4. Bước 3 — ba consumer qua API

```ts
it('RG-07 · R-PARCEL-04: phí khối lượng nhân cao điểm nhưng phí mưa không nhân', async () => {
  const res = await request(app).post('/api/quotes').set('x-user', 'customer:C001').send({
    kind: 'parcel', cityCode: 'SG', vehicle: 'bike', distanceKm: 5, parcelWeightKg: 6,
  }).expect(200);
  expect(res.body.quote.fare).toMatchObject({ parcelSurcharge: 2_000, base: 36_500, afterPeak: 45_625, rainSurcharge: 8_000, total: 54_000 });
});

it('RG-08 · R-FOOD-04/05: voucher chỉ giảm tiền món, giữ phí giao không giảm', async () => {
  const res = await request(app).post('/api/quotes').set('x-user', 'customer:C001').send({
    kind: 'food', cityCode: 'SG', distanceKm: 5,
    items: [{ name: 'Cơm', unitPrice: 60_000, quantity: 1 }], voucherCode: 'FOOD15',
  }).expect(200);
  expect(res.body.quote).toMatchObject({ itemsSubtotal: 60_000, itemsDiscount: 9_000, deliveryFee: 52_000, total: 103_000 });
  expect(res.body.quote.fare.discount).toBe(0);
});
```

RG-07 kiểm từng bước của bảng chi tiết cước (R-FARE-10): nếu phí mưa trôi vào `base`, trường `base` thành 44.500 thay vì 36.500 và test chỉ ra đúng bước lệch. RG-08 khẳng định voucher chỉ chạm tiền món: nếu voucher bị truyền vào lời gọi phí giao, `fare.discount` khác 0 dù `total` có thể vẫn trùng hợp.

Các test case API đi qua route Express thật, header `x-user` của lab, validation, use case báo giá và hai consumer `pricing.ts`/`food.ts` — tức là kiểm **cách hàm dùng chung được gọi**, điều test case hàm thuần không chạm tới.

### 6.5. Mong đợi và thực tế

**Mong đợi** (theo README lab): 8 tests pass, không skip.

**Thực tế — báo cáo gốc (13/11/2026)** (Windows, Node v24.15.0, npm 11.12.1, Vitest 3.2.7): chạy nhóm SYS-01…04 + SYS-06 được 5 file, **50 tests pass** trong 4,36 giây (3 + 8 + 24 + 7 + 8); riêng regression 8 tests pass trong 112 ms; `npm run typecheck` exit 0.

**Thực tế — chạy lại khi biên tập (13/11/2026)** (Windows, Node v22.18.0, Vitest 3.2.7), lệnh `npx vitest run articles/system-testing-regression-test`:

```text
 ✓ articles/system-testing-regression-test/regression.test.ts (8 tests) 150ms
 Test Files  1 passed (1)
      Tests  8 passed (8)
   Duration  2.08s (transform 334ms, setup 0ms, collect 1.22s, tests 150ms, environment 0ms, prepare 223ms)
```

Chênh lệch thời gian (112 ms so với 150 ms) **không phải** tiêu chí pass hay bằng chứng hiệu năng; tiêu chí là số lượt, không skip và assertion nguyên vẹn.

Để chạy rộng các bài API đã viết:

```sh
npm test -- articles/system-testing-smoke-test articles/system-testing-sanity-test articles/system-testing-functional-test articles/system-testing-integration-test articles/system-testing-regression-test
```

Mong đợi 50 tests — nhóm SYS-01…04 và SYS-06, **không phải** toàn bộ test của repository. Hành trình trình duyệt SYS-05 chạy bằng lệnh riêng `npx playwright test --config system-testing.config.ts` và cần Chrome; Vitest cố ý không chạy browser specs.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">8 lượt xanh chưa nói gì</span>
<p>Lab không sửa sản phẩm, không chạy một build lỗi đỏ rồi sửa xanh, không chạy mutation engine nên <b>không có mutation score</b>. RG-05 chỉ chứng minh <b>một</b> fault cụ thể bị oracle phân biệt. Các test case API dùng kho dữ liệu trong bộ nhớ và đồng hồ/thời tiết giả; bằng chứng SQL nằm ở SYS-04, bằng chứng trình duyệt ở SYS-05 (lần chạy SYS-06 không chạy lại browser vì không đổi UI hay cấu hình). Chưa kiểm phí huỷ, ô tô, cước tối thiểu với voucher lớn, hiệu năng, provider thanh toán thật hay lựa chọn test tự động theo diff.</p>
</div>

## 7. Gate, báo cáo và xử lý failure

### 7.1. Đọc failure

| Failure | Cần phân biệt | Hành động |
|---|---|---|
| Kết quả khác mong đợi sau một yêu cầu thay đổi | Thay đổi có chủ đích hay regression? | Đối chiếu yêu cầu đã phê duyệt; cập nhật oracle **kèm lý do** và người duyệt |
| Chỉ RG-04 fail, RG-01…03 pass | Lỗi tương tác/thứ tự tính | So từng bước bảng chi tiết cước với 34.500 → 43.125 → 51.125 |
| Chỉ consumer food fail | Hàm dùng chung hay cách truyền voucher | So `itemsDiscount`, `deliveryFee`, `fare.discount`, không chỉ `total` |
| Chạy lại mới pass | Flaky test, môi trường hay race sản phẩm? | Giữ failure đầu, log, seed, clock; điều tra trước khi rerun |
| Nhiều test case fail cùng lỗi setup | Môi trường hoặc fixture chung | Báo blocked; không mở hàng loạt defect trùng |
| Fault fixture không bị phân biệt | Oracle yếu hoặc input không kích hoạt | Thêm test case tương tác, assertion theo test basis |

### 7.2. Defect report và bisect

Một defect regression tốt ghi: commit tốt/xấu gần nhất nếu biết, case ID, input, mong đợi/thực tế (với tiền: cả bảng chi tiết cước), cấu hình clock/thời tiết, bước tái hiện tối thiểu. Khi dùng `git bisect`, mỗi bước phải chạy trên cùng môi trường và dữ liệu cố định; test flaky sẽ khiến bisect chỉ vào commit sai. Lab dùng clock và thời tiết giả để kết quả chỉ phụ thuộc code.

### 7.3. Báo cáo đúng phạm vi

Ví dụ kết luận đạt chuẩn: «Refactor fare.ts: 8/8 lượt regression công thức và ba consumer pass; nhóm SYS-01…04 pass (50/50); chưa chạy browser SYS-05, chưa kiểm phí huỷ và ô tô. Đề xuất: bổ sung test case ô tô cao điểm/mưa trước khi merge vì cùng nhánh code.» Đừng viết «regression pass» trơn — người đọc sẽ hiểu là mọi thứ đã được kiểm.

## 8. Quản lý suite lâu dài

### 8.1. Các tầng chạy trong CI

Thiết kế đề xuất (chưa có workflow tương ứng trong repo):

- **Mỗi PR:** component/API tests theo impact map + smoke nền — phản hồi trong vài phút;
- **Trước merge/release:** nhóm integration quan trọng (SQL, ví) theo vùng thay đổi;
- **Định kỳ và trước release:** regression rộng, browser journeys, các suite chậm.

Tần suất phụ thuộc chi phí chạy và hậu quả bỏ sót; không có quy tắc chung «nightly là đủ» — thay đổi công thức tiền xứng đáng chạy test case tương tác ngay ở PR.

### 8.2. Flaky quarantine có trách nhiệm

Quarantine tách test flaky khỏi gate chính để khỏi chặn sai, nhưng **mở một lỗ hổng coverage**. Mỗi test bị quarantine cần owner, issue, hạn xử lý, lý do và phương án kiểm bù. Retry không xoá failure đầu tiên: lần đầu đỏ, lần hai xanh là tín hiệu cần điều tra (race, state rò rỉ, phụ thuộc thời gian thực).

### 8.3. Bảo trì và xoá test

Chỉ gộp test case trùng khi chúng bảo vệ **cùng rủi ro**: RG-02 và RG-04 trông giống nhau, nhưng gộp là mất khả năng phân biệt. Đừng xoá test chỉ vì lâu không fail; khi tính năng bị bỏ theo yêu cầu, cập nhật traceability và ghi lý do xoá. Tránh snapshot lớn không chọn field: nó dễ dẫn tới «chấp nhận baseline mới» hàng loạt mà không ai đọc.

## 9. Mở rộng nâng cao

### 9.1. Selection, prioritization và minimization

**Selection** chọn test case cần chạy cho thay đổi này; **prioritization** đổi thứ tự để tín hiệu quan trọng đến sớm; **minimization** loại test case trùng. Selection theo coverage code bỏ sót tác động qua cấu hình, schema, seed hay dependency động — đổi phần trăm của GIAM20 trong seed không chạm dòng code nào. Nên kết hợp mapping yêu cầu–test case, lịch sử lỗi và một suite nền luôn chạy; đừng loại một test case chậm nhưng bảo vệ rủi ro cao về tiền.

### 9.2. Mutation testing

Mutation testing tạo nhiều thay đổi nhỏ (mutant) và đo xem test có phát hiện không. Stryker phân loại kết quả thành killed, survived, no coverage, timeout, lỗi compile/runtime; mỗi metric có mẫu số riêng. Mutant sống sót có thể do thiếu oracle, input không kích hoạt, hoặc là mutant **tương đương** (không đổi hành vi quan sát được). Fault fixture của bài chỉ là một «mutant» viết tay — **không phải** một lượt Stryker.

### 9.3. Differential testing khi thay thuật toán

Khi thay hẳn engine tính giá, chạy bản cũ và mới trên cùng bộ dữ liệu giúp thu hẹp vùng nghi ngờ, nhưng **không** cho biết bên nào đúng. Mỗi khác biệt cần phân loại: thay đổi có chủ đích theo SPEC, bug cũ được sửa (cần quyết định có chấp nhận không), hay regression mới. Với tiền, bộ dữ liệu nên gồm biên quãng đường (0,5 và 100 km), ngưỡng khối lượng (5,0 / 5,1 / 20,0 kg), các mốc giờ cao điểm và voucher chạm trần.

### 9.4. Regression phi chức năng

Sau đổi thuật toán, chức năng vẫn đúng nhưng latency hay bộ nhớ có thể xấu đi. Regression hiệu năng cần workload xác định, cấu hình máy, warm-up, số mẫu và ngưỡng biến động chấp nhận — chủ đề của [SYS-07 · Performance Test](/articles/system-testing-performance-test). Security và accessibility cũng cần baseline riêng.

### 9.5. Metrics không đánh lừa

Escaped defects (lỗi lọt tới người dùng), thời gian phát hiện, tỷ lệ flake, thời gian điều tra và coverage **yêu cầu** có giá trị hơn số test. Tỷ lệ pass phải đi kèm mẫu số, số skip và số quarantine: «100% pass» trên 8 lượt được chọn không tương đương 100% pass trên bộ release cả nghìn test case.

## 10. Anti-pattern, giới hạn và đánh đổi

- **Chỉ chạy test cạnh file sửa:** bỏ qua consumer dùng chung.
- **Cập nhật kết quả mong đợi theo output mới cho xanh:** biến regression thành «expected change» không ai phê duyệt.
- **Copy implementation cũ làm oracle:** đóng băng cả bug cũ.
- **Một test case đại diện cho mọi tổ hợp:** RG-02 đúng không nói gì về RG-04.
- **Gọi mọi failure là regression sản phẩm:** che lỗi môi trường, tạo defect trùng.
- **Đo chất lượng bằng số test:** cảm giác an toàn không có căn cứ.

Đánh đổi: thêm test case tương tác làm suite dài hơn chút nhưng rẻ hơn nhiều so với tính sai tiền hàng nghìn chuyến cao điểm; ngược lại, chạy mọi suite ở mọi PR làm chậm phản hồi và khiến đội phải tắt gate. Impact map công khai phần bỏ lại cho người duyệt release biết rủi ro nào còn mở.

## 11. Bài tập và câu hỏi tự kiểm

### Bài tập

1. **Cơ bản.** Vì sao RG-02 không bắt được candidate sai? Test case tối thiểu nào bắt được?
2. **Trung bình.** Refactor cũng ảnh hưởng xe ô tô. Viết thêm một test case với oracle tính tay.
3. **Trung bình.** Một test food bắt đầu flaky trong CI. Có nên xoá khỏi gate không?
4. **Nâng cao.** Thiết kế mapping regression cho một thay đổi trong `src/domain/money.ts` (chuyển đổi đồng ↔ milli-đồng).
5. **Nâng cao.** Yêu cầu mới: phụ phí trời mưa tăng từ 8.000 lên 10.000. Suite hiện tại phản ứng thế nào và cần làm gì?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><b>1.</b> Không cao điểm thì hệ số là 1, nên cộng mưa trước hay sau khi nhân cho cùng kết quả (43.000). Test case tối thiểu là cao điểm + mưa — RG-04 — nơi candidate sai ra 54.000, oracle 52.000.</p>
<p><b>2.</b> Ô tô 5 km cao điểm mưa: cơ bản 25.000 + 11.000 × 5 = 80.000; × 1,25 = 100.000; + 8.000 = <b>108.000</b>. Candidate sai: (80.000 + 8.000) × 1,25 = 110.000. Nên kiểm thêm voucher và cước tối thiểu theo rủi ro, với oracle tính riêng cho ô tô — đừng chỉ đổi <code>vehicle</code> mà giữ con số của xe máy.</p>
<p><b>3.</b> Không xoá khỏi gate mà không báo lại. Phân loại nguyên nhân (thời gian thực, state rò rỉ, race sản phẩm). Nếu buộc quarantine: owner, issue, hạn xử lý, kiểm bù (ví dụ chạy test case tương đương ở cấp hàm trong gate), và báo cáo ghi rõ coverage food giảm trong thời gian đó.</p>
<p><b>4.</b> Trong lab, <code>money.ts</code> được <code>fare.ts</code>, <code>parcel.ts</code>, <code>voucher.ts</code> và <code>food.ts</code> import — tức là nằm dưới mọi phép tính tiền. Mapping phải mở tới ride, parcel, food, voucher (đặc biệt trần giảm và phần lẻ dưới 1 đồng) và mọi caller khác — lần theo callers thực trước khi chốt; chỉ chạy RG-01…04 là chưa đủ. Ưu tiên theo hậu quả sai tiền và khả năng phát hiện sớm: test case voucher/food trước, rồi ride/parcel.</p>
<p><b>5.</b> Đây là <b>expected change</b>: RG-02, RG-04, RG-06, RG-07, RG-08 sẽ đỏ đúng như mong đợi; RG-05 cũng đỏ ở dòng assert hàm thật bằng 52.000. Cập nhật SPEC trước, tính lại oracle (ví dụ RG-04: 43.125 + 10.000 = 53.125 → 54.000 — lúc này trùng con số với candidate sai cũ, nên RG-05 cũng cần đổi input để vẫn phân biệt được), cập nhật kết quả mong đợi kèm lý do và người duyệt. Kiểm thêm chuyến đặt từ báo giá cũ vẫn giữ giá cũ (R-QUOTE-04).</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** RG-05 pass cho phép kết luận gì?

- a) Sản phẩm có lỗi ở công thức cước
- b) Suite có mutation score 100%
- c) Oracle của ca tương tác phân biệt được một fault cụ thể đã mô phỏng
- d) Mọi refactor của `calculateFare` đều sẽ bị phát hiện

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng</b>. a) sai: hàm sản phẩm trả 52.000 đúng. b), d) vượt bằng chứng: chỉ một candidate được thử, không có tập mutant hay công cụ đo.</p>
</details>

**Câu 2.** Sau một yêu cầu tăng phí mở cửa đã được duyệt, RG-01 đỏ. Đây là gì?

- a) Regression, phải revert
- b) Expected change: cập nhật oracle theo SPEC mới kèm lý do
- c) Flaky test, chạy lại
- d) Lỗi môi trường

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b>. Output khác baseline không tự động là defect. Nhưng chỉ cập nhật những test case mà SPEC mới thực sự đổi; các test case khác vẫn phải giữ.</p>
</details>

**Câu 3.** Vì sao RG-08 assert cả `fare.discount = 0`, không chỉ `total = 103.000`?

- a) Để test dài hơn
- b) Để phát hiện voucher bị áp nhầm vào phí giao ngay cả khi tổng tình cờ hợp lý
- c) Vì SPEC cấm trả `total`
- d) Vì `total` thay đổi theo giờ

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b>. R-FOOD-04 cấm voucher trừ vào phí giao. Assert từng thành phần giúp định vị lỗi và tránh để các sai lệch bù trừ nhau trong tổng.</p>
</details>

**Câu 4.** «Full regression pass» trong báo cáo nên được hiểu thế nào?

- a) Mọi hành vi có thể có của sản phẩm đã được kiểm
- b) Suite regression đã định nghĩa chạy đủ và pass; phạm vi suite cần được nêu
- c) Sản phẩm sẵn sàng release
- d) Không cần chạy smoke sau deploy

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b>. «Full» là đầy đủ <b>suite</b>, không phải đầy đủ hành vi. Quyết định release còn cần các đặc tính phi chức năng, acceptance và smoke sau deploy.</p>
</details>

**Câu 5.** Một test bị quarantine vì flaky. Điều nào là bắt buộc?

- a) Xoá test sau một tuần
- b) Owner, issue, hạn xử lý, kiểm bù và ghi giảm coverage trong báo cáo
- c) Bật retry 10 lần cho test đó trong gate
- d) Không cần làm gì nếu các test khác vẫn xanh

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b>. Quarantine là quyết định rủi ro có thời hạn, không phải chỗ chôn test. c) che flakiness và có thể giấu race sản phẩm.</p>
</details>

## 12. Tài liệu tham khảo

Nguồn ngoài tra cứu 13/11/2026.

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — mục 2.2.3 Confirmation Testing and Regression Testing. PDF: https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf. Chiến lược chọn suite, quarantine và tầng CI trong bài là thực hành của series, không phải yêu cầu của syllabus.
- Stryker Mutator — *Mutant states and metrics*: https://stryker-mutator.io/docs/mutation-testing-elements/mutant-states-and-metrics/ — cách đọc killed/survived/no coverage và mẫu số của metric.
- Vitest — `it.each`, `toMatchObject`: https://vitest.dev/api/expect.html
- Phòng lab Tới Nơi: `SPEC.md` (R-FARE-01…10, R-PARCEL-03/04, R-FOOD-04/05, R-VOUCHER-06), `src/domain/fare.ts`, `pricing.ts`, `food.ts` và bộ test của bài — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-regression-test
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [Confirmation Testing và Regression Testing](/articles/confirmation-testing-va-regression-testing) · [Maintenance Testing](/articles/maintenance-testing) · [SYS-02 · Sanity Test](/articles/system-testing-sanity-test) · [SYS-03 · Functional Test](/articles/system-testing-functional-test) · [SYS-05 · System Test](/articles/system-testing-system-test)

Bài tiếp theo: [SYS-07 · Performance Test](/articles/system-testing-performance-test).
