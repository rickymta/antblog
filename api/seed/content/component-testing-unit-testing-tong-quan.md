---
slug: component-testing-unit-testing-tong-quan
title: "Component Testing (Unit Testing): góc nhìn tổng quan"
summary: "Component testing nhìn như một test level: kiểm cái gì, căn cứ vào đâu, ai làm, ở môi trường nào, cô lập phụ thuộc ra sao — và vì sao không nên đồng nhất hoàn toàn với «unit test». Ví dụ: hàm tính cước của Tới Nơi."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-012, component-testing, unit-testing, test-level, test-harness]
language: vi
---

«Test hàm tính giá chuyến xe» nghe đơn giản: gọi hàm, so kết quả. Nhưng trước khi viết dòng test nào, cần trả lời mấy câu mà nhiều đội bỏ qua: **component** ở đây là hàm nào, hay cả module? Căn cứ vào tài liệu nào để biết kết quả đúng? Đồng hồ và thời tiết — thứ làm giá thay đổi — thì xử lý ra sao? Bài này nhìn component testing ở góc **test level**: mục tiêu, test object, test basis, người làm, môi trường và cách cô lập. Kỹ thuật viết unit test chi tiết thuộc Series 06.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-012</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S02 · Test Levels &amp; Test Types</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Component</td></tr>
<tr><th scope="row">Test Type</th><td>Functional · White-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/tong-quan-5-test-level-istqb">TEST-011 Tổng quan 5 test level của ISTQB</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (đọc code, chưa viết test): đọc <code>src/domain/fare.ts</code> và <code>src/app/ports.ts</code>, xác định test object, test basis (R-FARE-01…08), dữ liệu cần và cách cô lập đồng hồ, thời tiết (mục 5, có lời giải)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 2 · mục 2.2.1 · FL-2.2.1 (K2); liên quan mục 2.2.2 (functional, white-box) và 2.1.5 (non-functional từ component level)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Giả sử system test báo: chuyến xe máy 7,4 km lúc 11:30 trời mưa có voucher 20% ra **53.000 đ**, đặc tả nói **52.000 đ** (R-FARE-09). Lỗi nằm ở đâu? Có sáu bước tính, một hàm làm tròn quãng đường, một hàm giảm giá, chưa kể API, kho dữ liệu, đồng hồ máy chủ. Tìm từ ngoài vào mất cả buổi.

Component testing đặt câu hỏi ở tầng thấp nhất: **từng phần nhỏ có tự làm đúng việc của nó không**, khi được kiểm riêng, không qua API, không database, không mạng. Lợi ích:

- **Khoanh lỗi ngay:** test đỏ chỉ thẳng vào bước «cộng phụ phí trời mưa trước khi nhân 1,25».
- **Rẻ và nhanh:** hàng trăm test chạy trong vài mili giây, chạy được sau mỗi lần lưu file và ở bước 2 của pipeline (xem [TEST-008](/articles/kiem-thu-trong-sdlc-agile-waterfall)).
- **Kiểm được các biên khó chạm từ ngoài:** 0,45 km, 100,05 km, đúng 13:00:00.

Dùng khi component có **logic**: công thức cước, máy trạng thái chuyến, phí huỷ, điều kiện voucher. Code chỉ chuyển dữ liệu qua lại (gọi kho rồi trả về) thường kiểm hiệu quả hơn ở component integration ([TEST-013](/articles/component-integration-testing)).

## 2. Lý thuyết theo ISTQB

### 2.1. Định nghĩa

CTFL v4.0.1 (mục 2.2.1) mô tả component testing — **còn gọi là unit testing** — là level tập trung **kiểm các component một cách tách riêng**. Nó thường cần hỗ trợ đặc thù như **test harness** hoặc **unit test framework**, và **thường do developer thực hiện trong môi trường phát triển của họ**.

Ba chữ cần đọc kỹ:

- **«Tách riêng» (in isolation):** component được kiểm mà không cần phần còn lại của hệ thống. Không có nghĩa là thay mọi thứ bằng đồ giả — xem 2.3.
- **«Thường» do developer:** không phải luật. Tester vẫn góp test condition, dữ liệu biên, review bộ test.
- **«Component»:** theo ISTQB Glossary, là một phần của hệ thống có thể kiểm riêng. Kích cỡ do ngữ cảnh quyết định: một hàm, một class, một module.

### 2.2. Năm thuộc tính của level component

[TEST-011](/articles/tong-quan-5-test-level-istqb) đã có bảng cho cả 5 level. Đây là bản chi tiết cho component, ví dụ ở Tới Nơi:

| Thuộc tính (2.2.1) | Ở level component | Ví dụ Tới Nơi |
|---|---|---|
| Test object | Hàm, class, module; code của nó | `calculateFare`, `normalizeDistanceKm` trong `fare.ts` |
| Test objectives | Tìm defect trong component; xác minh nó đúng đặc tả và thiết kế; tạo niềm tin để tích hợp tiếp | Mỗi bước của R-FARE-03 đúng, đúng thứ tự, đúng làm tròn |
| Test basis | Đặc tả component, thiết kế chi tiết, code, mô hình dữ liệu | R-FARE-01…10, R-GEN-01, kiểu `FareInput`/`FareBreakdown` |
| Defects / failures | Sai công thức, sai biên, sai làm tròn, sai luồng điều khiển, ném lỗi sai mã | Dùng `<` thay `<=` ở biên 100 km; làm tròn tiền ở bước 2 |
| Approach, responsibility | Framework + harness; chạy rất thường xuyên, tự động trong CI; developer chính, tester góp ý | vitest, Node, không DB; developer của Pricing |

Cột giữa do blog điền theo các thuộc tính syllabus liệt kê; CTFL v4.0.1 không đưa bảng chi tiết cho từng level.

### 2.3. Test harness, driver, stub

Để kiểm một component tách riêng, cần hai loại «đồ phụ trợ»:

- **Driver** — thứ *gọi* component thay cho phần gọi thật. Với unit test framework, chính test case đóng vai driver.
- **Stub** (và các test double khác) — thứ *thay* phần mà component gọi tới, khi phần đó chậm, không tất định hoặc chưa có (đồng hồ, thời tiết, database, ví).

Test harness là tập hợp driver, stub và môi trường chạy. Nguyên tắc thực dụng: chỉ thay những phụ thuộc **không tất định, chậm hoặc có tác dụng phụ**. Hàm thuần như `roundToTenth` thì cứ dùng thật — thay nó bằng stub chỉ làm test xa thực tế hơn. Các loại test double (dummy, stub, fake, mock, spy) đi sâu ở Series 06.

### 2.4. Test type ở level component

Bốn test type của mục 2.2.2 đều làm được ở level này, với trọng tâm khác:

| Test type | Ở level component | Ví dụ |
|---|---|---|
| Functional / black-box | Kiểm hàm theo đặc tả, không nhìn code | Oracle R-FARE-09 ⇒ 52.000 đ |
| White-box | Phủ cấu trúc code: mọi nhánh `peak`, `raining`, có/không voucher, nhánh cước tối thiểu | Đo coverage — Series 05 |
| Non-functional | Thời gian chạy của một hàm nóng; khả năng bảo trì | Mục 2.1.5 xem non-functional từ component level là một dạng shift left |
| Confirmation, regression | Chạy lại sau mỗi thay đổi | Bộ test component chạy ở mỗi commit |

### 2.5. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa | Ghi chú |
|---|---|---|
| component | thành phần | phần của hệ thống kiểm riêng được |
| component testing | kiểm thử thành phần | ISTQB coi unit testing là tên khác |
| in isolation | tách riêng, cô lập | không cần phần còn lại của hệ thống |
| test harness | bộ khung kiểm thử | driver + stub + môi trường chạy |
| driver / stub | bộ gọi / bộ thay thế | thay phần gọi tới / phần được gọi |
| test double | đồ thế thân trong test | dummy, stub, fake, mock, spy — Series 06 |
| unit test framework | khung unit test | vitest, JUnit, pytest |
| testability | khả năng kiểm thử | thiết kế sao cho dễ cô lập |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB — «unit» và «component» trong thực hành</span>
<p>CTFL coi hai tên là một. Ngoài đời, bạn sẽ gặp ít nhất ba cách dùng:</p>
<table>
<thead><tr><th>Cách dùng</th><th>«Unit» là</th><th>«Component» là</th></tr></thead>
<tbody>
<tr><td>Theo ISTQB</td><td>tên khác của component</td><td>phần kiểm riêng được</td></tr>
<tr><td>Nhiều đội developer</td><td>một hàm/class, thay hết phụ thuộc</td><td>cụm nhiều class phối hợp</td></tr>
<tr><td>Cộng đồng microservices (vd tài liệu kiến trúc Tới Nơi §17.1)</td><td>quy tắc bên trong service: «Pricing rules, fee calculation»</td><td>cả một service chạy riêng với database test: «Booking service + PostgreSQL/Redis test instance»</td></tr>
</tbody>
</table>
<p>Vì vậy, khi đọc test strategy, đừng đoán: hỏi «component» của đội này to cỡ nào và cô lập những gì. Một số tài liệu ngành còn tách unit test «solitary» (thay mọi cộng tác viên) với «sociable» (dùng cộng tác viên thật) — cũng là thuật ngữ ngoài ISTQB.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

**Yêu cầu nghiệp vụ.** Pricing & Quote tính giá theo bảng giá từng loại xe (R-FARE-01), làm tròn quãng đường 0,1 km half-up và kiểm khoảng [0,5; 100] km (R-FARE-02), tính theo thứ tự cố định sáu bước: cước cơ bản → cao điểm ×1,25 → phụ phí trời mưa 8.000 đ → voucher → cước tối thiểu → làm tròn lên 1.000 đ (R-FARE-03…08). Mọi bước trung gian không làm tròn (R-GEN-01). Kết quả trả kèm bảng chi tiết từng bước (R-FARE-10).

**System context.** Trong kiến trúc đích, Pricing & Quote là một service riêng (§4.3): Booking gọi nó qua gRPC, nó lấy quãng đường từ MapViet, lưu quote có `pricing_version` và breakdown. Trong lab, Pricing là module `src/domain/fare.ts`, `pricing.ts` và `src/app/quote.ts` (SPEC R-ARCH-01). Lõi `calculateFare` là **hàm thuần**: không I/O, không đọc giờ hệ thống.

**Rủi ro — và rủi ro nào thuộc level component.**

| Rủi ro | Thuộc level |
|---|---|
| Cộng phụ phí trời mưa trước khi nhân 1,25 (sai R-FARE-03/05) | Component |
| Làm tròn tiền ở giữa công thức ⇒ lệch 1.000 đ (R-GEN-01) | Component |
| 0,45 km bị từ chối, 100,05 km được chấp nhận (R-FARE-02) | Component |
| Voucher kéo giá xuống dưới cước tối thiểu (R-FARE-07) | Component |
| Sai số dấu phẩy động: 0,18 × 10 = 1,7999999999999998 | Component |
| Giờ cao điểm tính theo múi giờ máy chủ thay vì UTC+7 (R-GEN-02) | Component — nhưng của `time.ts` (`isPeakTime`), không phải `fare.ts` |
| `quote.ts` truyền cờ `peak` sai cho `calculateFare` | Component integration |
| Dịch vụ thời tiết trả «đang mưa» cho sai thành phố | Component integration / system |
| MapViet trả quãng đường sai đơn vị (mét) | System integration |

Bảng này là kết quả quan trọng của tư duy test level: **không phải rủi ro nào của «tính giá» cũng kiểm được bằng component test của `calculateFare`.**

## 4. Phân tích kỹ thuật

### 4.1. Chọn ranh giới component

| Lựa chọn test object | Phải cô lập gì | Nhận xét |
|---|---|---|
| `normalizeDistanceKm` riêng | Không gì | Rất hẹp; hợp để khoá biên quãng đường |
| `calculateFare` (+ `money.ts`, `voucherDiscountMilli` dùng thật) | Không gì — đầu vào đã là giá trị thuần | **Chọn cho bài này**: đúng phạm vi R-FARE-01…08 |
| `priceRequest` trong `pricing.ts` | Không gì | Thêm bưu kiện, đồ ăn; rộng hơn đề bài |
| `priceQuote` trong `quote.ts` | Đồng hồ, thời tiết, kho voucher, kho chuyến (`ports.ts`) | Ranh giới với component integration: ghép domain với các cổng |

Không có đáp án «đúng duy nhất». Điều cần là **ghi rõ** test object và lý do, để không ai tưởng test `calculateFare` đã kiểm luôn giờ cao điểm.

### 4.2. Phụ thuộc và cách cô lập

Điểm then chốt nằm ở kiểu đầu vào của `calculateFare` (trích `src/domain/fare.ts`):

```ts
export interface FareInput {
  vehicle: Vehicle;
  distanceKm: number;        // chưa làm tròn — hàm tự chuẩn hoá
  parcelSurcharge?: number;  // đồng, cộng vào cước cơ bản
  peak: boolean;
  raining: boolean;
  voucher?: VoucherTerms | null;
}
```

Giá phụ thuộc giờ đặt và thời tiết, nhưng hàm **không** hỏi đồng hồ hay dịch vụ thời tiết: nó nhận sẵn hai giá trị `peak`, `raining`. Việc hỏi đồng hồ (`isPeakTime(deps.clock.now())`) và hỏi thời tiết (`deps.weather.isRaining(...)`) nằm ở tầng gọi `priceQuote`, qua các cổng `Clock`, `WeatherService` của `ports.ts`. Đây là thiết kế cho **testability** — đúng «clock abstraction» và «fake adapters» trong testability hooks của kiến trúc (§17.2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 360" role="img" aria-label="Ranh giới component khi kiểm calculateFare. Bên trái: priceQuote trong quote.ts dùng các cổng Clock, WeatherService, VoucherRepository, TripRepository; khi test thay bằng FixedClock, thời tiết giả, kho trong bộ nhớ. Mũi tên sang phải mang peak, raining và voucher terms. Bên phải: khung test object fare.ts gồm calculateFare, money.ts và voucherDiscountMilli dùng thật vì thuần và tất định; test basis R-FARE-01 đến 08 và R-GEN-01; chạy trong mili giây, không database, không mạng.">
<defs>
<marker id="t012-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="t012-dep" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="135" y="28" text-anchor="middle" class="t-small">tầng gọi (component integration)</text>
<rect x="20" y="100" width="230" height="70" rx="8" class="f-bg s-line"/>
<text x="135" y="128" text-anchor="middle" class="t-mono t-b">priceQuote</text>
<text x="135" y="148" text-anchor="middle" class="t-mono t-small">src/app/quote.ts</text>
<rect x="20" y="210" width="230" height="96" rx="8" class="f-warn-soft s-warn"/>
<text x="135" y="232" text-anchor="middle" class="t-b t-warn">cổng trong ports.ts</text>
<text x="135" y="252" text-anchor="middle" class="t-mono t-small">Clock · WeatherService</text>
<text x="135" y="270" text-anchor="middle" class="t-mono t-small">VoucherRepository</text>
<text x="135" y="288" text-anchor="middle" class="t-mono t-small">TripRepository</text>
<line x1="135" y1="170" x2="135" y2="206" class="s-muted dash" stroke-width="1.4" marker-end="url(#t012-dep)"/>
<text x="135" y="330" text-anchor="middle" class="t-small">khi test: FixedClock, thời tiết giả,</text>
<text x="135" y="348" text-anchor="middle" class="t-small">kho trong bộ nhớ</text>
<line x1="250" y1="140" x2="368" y2="140" class="s-pk" stroke-width="1.8" marker-end="url(#t012-arrow)"/>
<text x="298" y="128" text-anchor="middle" class="t-small">peak, raining</text>
<text x="298" y="160" text-anchor="middle" class="t-small">voucher terms</text>
<rect x="350" y="50" width="550" height="290" rx="12" class="f-surface s-phys dash"/>
<text x="625" y="74" text-anchor="middle" class="t-b t-phys">Test object — src/domain/fare.ts</text>
<rect x="370" y="105" width="200" height="70" rx="8" class="f-phys-soft s-phys"/>
<text x="470" y="133" text-anchor="middle" class="t-mono t-b">calculateFare</text>
<text x="470" y="153" text-anchor="middle" class="t-mono t-small">normalizeDistanceKm</text>
<rect x="650" y="95" width="230" height="44" rx="8" class="f-bg s-line"/>
<text x="765" y="115" text-anchor="middle" class="t-mono">money.ts</text>
<text x="765" y="131" text-anchor="middle" class="t-small">toMilli · percentOf · làm tròn</text>
<rect x="650" y="150" width="230" height="44" rx="8" class="f-bg s-line"/>
<text x="765" y="170" text-anchor="middle" class="t-mono">voucher.ts</text>
<text x="765" y="186" text-anchor="middle" class="t-small">voucherDiscountMilli</text>
<line x1="570" y1="128" x2="646" y2="117" class="s-muted" stroke-width="1.4" marker-end="url(#t012-dep)"/>
<line x1="570" y1="152" x2="646" y2="170" class="s-muted" stroke-width="1.4" marker-end="url(#t012-dep)"/>
<text x="765" y="214" text-anchor="middle" class="t-small t-phys">thuần, tất định ⇒ dùng thật</text>
<line x1="370" y1="236" x2="880" y2="236" class="s-line"/>
<text x="625" y="262" text-anchor="middle" class="t-small">test basis: R-FARE-01…08, R-GEN-01, R-VOUCHER-06 · oracle R-FARE-09</text>
<text x="625" y="286" text-anchor="middle" class="t-small">driver = test case của vitest · không cần stub</text>
<text x="625" y="310" text-anchor="middle" class="t-small">chạy trong mili giây · không database · không mạng</text>
</svg>
</div>
<figcaption><b>Hình 1 · Ranh giới component khi kiểm <code>calculateFare</code>.</b> Đồng hồ và thời tiết đã bị đẩy ra tầng gọi, nên component nhận giá trị thuần và không cần stub. Cái giá phải trả: việc «tính đúng cờ <code>peak</code>» phải được kiểm ở chỗ khác — <code>isPeakTime</code> (component của <code>time.ts</code>) và <code>priceQuote</code> (component integration).</figcaption>
</figure>

### 4.3. Test condition ở level component

Từ test basis, các test condition (chưa phải test case) cho `calculateFare`:

1. Cước cơ bản đúng bảng giá cho `bike` và `car` (R-FARE-01).
2. Quãng đường làm tròn half-up trước mọi bước; biên 0,44 / 0,45 và 100,04 / 100,05 (R-FARE-02).
3. Ngoài khoảng ⇒ lỗi `DISTANCE_OUT_OF_RANGE`, không trả giá (R-FARE-02).
4. Hệ số cao điểm áp lên cước cơ bản kể cả phụ phí bưu kiện (R-FARE-04, R-PARCEL-04).
5. Phụ phí trời mưa cộng sau hệ số, không bị nhân (R-FARE-05).
6. Voucher tính trên số sau bước 3, theo min(percent × A, trần); dưới `minAmount` ⇒ `VOUCHER_MIN_AMOUNT` (R-FARE-06, R-VOUCHER-06).
7. Cước tối thiểu áp sau voucher (R-FARE-07).
8. Làm tròn lên 1.000 đ chỉ ở bước cuối; giá trị trung gian giữ phần lẻ (R-FARE-08, R-GEN-01, R-FARE-10).

Kỹ thuật biến các condition này thành test case (EP, BVA, decision table) là nội dung Series 04; viết thành code là Series 06.

## 5. Thực hành

**Đề bài (bài thực hành của curriculum).** Mở `src/domain/fare.ts` và `src/app/ports.ts` của lab. Không viết code, hãy điền phiếu: (1) test object, (2) test basis, (3) dữ liệu cần, (4) cách cô lập phụ thuộc — đặc biệt đồng hồ và thời tiết, (5) môi trường và người thực hiện.

<details>
<summary><b>Lời giải mẫu — phiếu phân tích</b></summary>
<table>
<thead><tr><th>Mục</th><th>Trả lời</th></tr></thead>
<tbody>
<tr><td>1. Test object</td><td><code>calculateFare</code> và <code>normalizeDistanceKm</code> (export của <code>fare.ts</code>). Hai helper <code>money.ts</code> và <code>voucherDiscountMilli</code> nằm <i>trong</i> phạm vi: chúng là một phần cách hàm tính, không phải phụ thuộc cần thay. Hằng <code>FARE_TABLE</code>, <code>PEAK_PERCENT</code>, <code>RAIN_SURCHARGE</code> cũng thuộc đối tượng.</td></tr>
<tr><td>2. Test basis</td><td>R-FARE-01…08 (quy tắc), R-FARE-09 (oracle), R-FARE-10 (các trường của bảng chi tiết), R-GEN-01 (không làm tròn giữa chừng), R-VOUCHER-06 (tiền giảm, <code>minAmount</code>), R-PARCEL-04 (phụ phí bưu kiện bị nhân hệ số). Thêm kiểu <code>FareInput</code>/<code>FareBreakdown</code> (thiết kế chi tiết) và code (căn cứ cho white-box).</td></tr>
<tr><td>3. Dữ liệu cần</td><td>Hai loại xe; quãng đường biên 0,44 / 0,45 / 7,35 / 100,04 / 100,05; tổ hợp <code>peak</code> × <code>raining</code>; voucher: không có, 20% trần lớn, 50% (đụng cước tối thiểu), <code>minAmount</code> lớn hơn giá; <code>parcelSurcharge</code> 0 và 2.000 đ. Kết quả mong đợi tính tay từ R-FARE (bảng dưới).</td></tr>
<tr><td>4. Cô lập phụ thuộc</td><td>Không cần stub nào: hàm không gọi <code>Date.now</code>, không gọi thời tiết, không đọc kho. Đồng hồ và thời tiết đã thành tham số <code>peak</code>, <code>raining</code>; voucher đã thành <code>VoucherTerms</code>. Các cổng <code>Clock</code>, <code>WeatherService</code>, <code>VoucherRepository</code>, <code>TripRepository</code> trong <code>ports.ts</code> chỉ cần thay (bằng <code>FixedClock</code>, thời tiết giả, kho trong bộ nhớ) khi kiểm tầng <code>priceQuote</code>. Mốc 11:00/13:00 kiểm ở component <code>isPeakTime</code>.</td></tr>
<tr><td>5. Môi trường, người làm</td><td>Node 22 + vitest trên máy developer và ở bước unit của CI; không server, không database. Developer của Pricing viết và duy trì; tester review danh sách condition và dữ liệu biên.</td></tr>
</tbody>
</table>
</details>

**Dữ liệu mong đợi (oracle tính tay theo R-FARE).** Các số dưới đây được tính tay rồi đối chiếu bằng một lần gọi `calculateFare` thật của lab — khớp toàn bộ.

| # | Đầu vào | Tính tay | Tổng mong đợi |
|---|---|---|---|
| D1 | bike, 0,44 km | làm tròn 0,4 < 0,5 | lỗi `DISTANCE_OUT_OF_RANGE` |
| D2 | bike, 0,45 km | ⇒ 0,5 km: 12.000 + 2.250 = 14.250 < 15.000 | 15.000 đ (cước tối thiểu) |
| D3 | bike, 7,35 km | ⇒ 7,4 km: 45.300 | 46.000 đ |
| D4 | car, 10 km | 25.000 + 110.000 | 135.000 đ |
| D5 | bike, 3 km, cao điểm | 25.500 × 1,25 = 31.875 | 32.000 đ |
| D6 | bike, 3 km, cao điểm, mưa | 31.875 + 8.000 = 39.875 | 40.000 đ (mưa không bị nhân) |
| D7 | bike, 7,4 km, cao điểm, mưa, GIAM20 | 64.625 − 12.925 = 51.700 | 52.000 đ (R-FARE-09) |
| D8 | bike, 1 km, voucher 50% | 16.500 − 8.250 = 8.250 < 15.000 | 15.000 đ (R-FARE-07) |
| D9 | bike, 2 km, cao điểm, phụ phí bưu kiện 2.000 | (12.000 + 9.000 + 2.000) × 1,25 = 28.750 | 29.000 đ |
| D10 | car, 100,04 / 100,05 km | ⇒ 100,0 km: 1.125.000 / ⇒ 100,1 km | 1.125.000 đ / lỗi |

**Quan sát: vì sao component test rẻ.** Lab có sẵn ba tệp test khói ở `tests/smoke/` (không phải code của bài này), mỗi tệp ở một tầng. Chạy từng tệp:

```bash
cd examples/toi-noi-testing
npx vitest run tests/smoke/domain.test.ts    # hàm thuần của src/domain
npx vitest run tests/smoke/api.test.ts       # qua API bằng supertest
npx vitest run tests/smoke/pglite.test.ts    # kho PostgreSQL (PGlite)
```

| Tệp | Tầng | Số test | Thời gian chạy test (vitest báo «tests») |
|---|---|---:|---:|
| `domain.test.ts` | Component (hàm thuần) | 7 | 7 ms |
| `api.test.ts` | Hệ thống qua API | 11 | 468 ms |
| `pglite.test.ts` | Kho dữ liệu PostgreSQL | 4 | 1,10 s |

Số đo của một lần chạy trên máy viết bài (Windows 11, Node 22.18, vitest 3.2.7) — chỉ để so tương đối. Mỗi test component nhanh hơn hàng chục đến hàng trăm lần, nên có thể có hàng trăm test biên như bảng D1–D10 mà vẫn chạy sau mỗi lần lưu file.

## 6. Góc nhìn nâng cao

- **Unit ≠ component ở mọi nơi.** ISTQB coi là một, nhưng bước «3 · Component tests» trong pipeline kiến trúc (§17) là cả service Booking với PostgreSQL test container — theo ngôn ngữ ISTQB, đó gần với component integration hơn. Đọc định nghĩa của đội trước khi so số lượng test.
- **Cô lập quá tay.** Thay cả `money.ts` bằng mock thì test không còn kiểm cách tính thật; tệ hơn, test vẫn xanh khi `toMilli` sai. Thay những gì không tất định hoặc chậm; giữ thật những gì thuần.
- **Component đúng chưa chắc hệ đúng.** `calculateFare` xanh 100% nhưng `priceQuote` truyền `peak` của giờ máy chủ thay vì giờ VN ⇒ khách vẫn trả sai. Lỗi ở **chỗ nối** — việc của component integration ([TEST-013](/articles/component-integration-testing)).
- **Coverage cao không có nghĩa là test tốt.** Bộ test có thể đi qua mọi nhánh mà không kiểm biên 0,45 km. White-box và giới hạn của coverage ở Series 05 (TEST-043 trở đi).
- **Thiết kế quyết định mức dễ test.** Nếu `calculateFare` tự gọi `new Date()` bên trong, test buộc phải chạy đúng 11:30 hoặc giả lập đồng hồ toàn cục — chậm và chập chờn. Đẩy phụ thuộc ra ngoài là quyết định thiết kế; tester nên nêu điểm này khi review (Series 03).
- **Component testing không chỉ của developer.** Tester đóng góp danh sách biên, oracle tính tay, và review xem test có kiểm đúng đặc tả không. Ở đội test-first, test component còn được viết trước code (TDD — [TEST-008](/articles/kiem-thu-trong-sdlc-agile-waterfall)).

Đi tiếp: cách viết unit test (Arrange–Act–Assert, FIRST) ở TEST-055, thiết kế bộ test Pricing Engine ở TEST-056, test double ở Series 06.

## 7. Bài tập và câu hỏi ôn tập

**Bài 1.** Làm phiếu phân tích như mục 5 cho `src/domain/parcel.ts` (`normalizeWeightKg`, `assertVehicleFits`, `parcelSurcharge`).

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><b>Test object:</b> ba hàm export và các hằng <code>MAX_PARCEL_KG</code>, <code>BIKE_MAX_KG</code>, <code>FREE_KG</code>. <b>Test basis:</b> R-PARCEL-01 (làm tròn 0,1 kg, 0 &lt; w ≤ 30), R-PARCEL-02 (&gt; 20 kg phải ô tô), R-PARCEL-03 (2.000 đ mỗi kg bắt đầu vượt 5 kg). R-PARCEL-04 (phụ phí bị nhân hệ số) thuộc <code>calculateFare</code>, không thuộc tệp này. <b>Dữ liệu:</b> 0,04 / 0,05 / 30,04 / 30,05 kg; 20,0 kg với <code>bike</code> (được) và 20,1 kg với <code>bike</code> (lỗi <code>VEHICLE_TOO_SMALL</code>); 5 / 5,1 / 6,0 / 6,1 / 30 kg ⇒ 0 / 2.000 / 2.000 / 4.000 / 50.000 đ. <b>Cô lập:</b> không phụ thuộc nào cần thay. <b>Rủi ro đáng chú ý:</b> 6,1 − 5 trong dấu phẩy động — code tính trên số phần mười kg để tránh.</p>
</details>

**Bài 2.** `customerCancellationFee` trong `src/domain/cancel.ts` phụ thuộc thời gian («đúng 2 phút sau khi nhận»). Nó cô lập phụ thuộc này thế nào? Nếu hàm tự gọi `Date.now()`, việc kiểm ở level component khó hơn ra sao?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Hàm nhận <code>now</code> là một trường của <code>CancellationInput</code> (cùng <code>acceptedAt</code>, <code>etaMinutes</code>), nên test truyền thẳng hai mốc 119 giây và 120 giây (R-CANCEL-02) mà không cần đồng hồ giả. Nếu hàm tự gọi <code>Date.now()</code>: test phải giả lập đồng hồ toàn cục hoặc chờ thời gian thật — chậm, dễ chập chờn quanh mốc giây; biên «đúng 2 phút 0 giây» gần như không kiểm ổn định được. Đồng hồ thật chỉ cần ở tầng use case (<code>cancelByCustomer</code> dùng <code>deps.clock</code>), nơi test dùng <code>FixedClock</code>.</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K1).** Theo CTFL v4.0.1, component testing còn được gọi là gì?

- a) Unit testing
- b) Integration testing
- c) Smoke testing
- d) Module integration testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a) đúng</b> — syllabus ghi «also known as unit testing». b) và d) là tên gần với component integration testing (syllabus gọi là unit integration testing). c) smoke testing không phải test level.</p>
</details>

**Câu 2 (K2).** Phát biểu nào mô tả đúng component testing theo CTFL v4.0.1?

- a) Kiểm interface giữa Tới Nơi và VíXanh trong môi trường gần vận hành
- b) Kiểm các component tách riêng, thường cần test harness hoặc unit test framework, thường do developer làm trong môi trường phát triển
- c) Luôn do đội test độc lập làm sau khi system testing xong
- d) Chỉ kiểm các đặc tính phi chức năng của component

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b>, khớp mô tả 2.2.1. a) là system integration testing. c) sai về người làm và về thứ tự. d) sai: functional là trọng tâm thường gặp, non-functional chỉ là một khả năng.</p>
</details>

**Câu 3 (K2).** Đội Tới Nơi kiểm `calculateFare`. Đâu là test basis phù hợp nhất ở level component?

- a) Tiêu chí chấp nhận do vận hành ký
- b) Đặc tả quy tắc cước R-FARE-01…08 và thiết kế kiểu `FareInput`/`FareBreakdown`
- c) Tài liệu API của MapViet
- d) Runbook khôi phục database

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng:</b> đặc tả và thiết kế chi tiết của component. a) là căn cứ của acceptance testing. c) là căn cứ của system integration testing. d) là căn cứ của operational acceptance testing.</p>
</details>

**Câu 4 (K2).** Khi kiểm `calculateFare`, cách xử lý phụ thuộc nào hợp lý nhất?

- a) Thay `money.ts` bằng mock để test «thật sự cô lập»
- b) Dùng thật các helper thuần như `money.ts`; chỉ thay phụ thuộc không tất định hoặc chậm — ở đây không có, vì giờ và thời tiết đã là tham số
- c) Khởi động máy chủ lab rồi gọi `POST /api/quotes`
- d) Chạy test đúng lúc 11:30 để có giờ cao điểm thật

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng.</b> a) làm test xa thực tế, có thể xanh khi helper sai. c) đó là test qua API của cả hệ, không còn là component testing. d) phụ thuộc giờ thật làm test chập chờn; <code>peak</code> là tham số nên chỉ cần truyền <code>true</code>.</p>
</details>

**Câu 5 (K2).** Lỗi nào ít khả năng được phát hiện nhất bởi component test của `calculateFare`?

- a) Phụ phí trời mưa bị nhân 1,25
- b) 100,05 km được chấp nhận
- c) Giá bị làm tròn ở bước cao điểm
- d) MapViet trả quãng đường bằng mét thay vì km

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>d) đúng:</b> đó là lỗi ở interface với hệ bên ngoài — system integration testing mới bắt được. a), b), c) đều là lỗi logic bên trong component, đúng thứ component testing nhắm tới.</p>
</details>

**Câu 6 (K2).** Tài liệu kiến trúc Tới Nơi gọi «Component test» là kiểm service Booking cùng một PostgreSQL test instance. So với ISTQB, nhận xét nào đúng nhất?

- a) Hoàn toàn trùng định nghĩa component testing của ISTQB, không cần làm rõ
- b) Là acceptance testing vì có database thật
- c) Là cách dùng thuật ngữ của ngành; theo ISTQB, phạm vi đó (code service tương tác với database) gần component integration hơn — test strategy nên ghi rõ định nghĩa đang dùng
- d) Là system integration testing vì PostgreSQL là hệ bên ngoài

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng.</b> «Component» trong cộng đồng microservices thường là cả service; ISTQB nhấn vào kiểm tách riêng. a) bỏ qua khác biệt. b) acceptance nhằm validation nhu cầu người dùng. d) database riêng của service là một phần của hệ thống Tới Nơi, không phải hệ bên ngoài.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — mục 2.2.1 Test Levels (component testing, LO FL-2.2.1), 2.2.2 Test Types, 2.1.5 Shift Left. PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (component, component testing, test harness, driver, stub).
- Tài liệu kiến trúc Tới Nơi (nội bộ blog): §4.3 Pricing & Quote, §17 pipeline, §17.1 chiến lược test, §17.2 testability hooks.
- Phòng lab: `src/domain/fare.ts`, `src/app/quote.ts`, `src/app/ports.ts`, `tests/smoke/`; `SPEC.md` R-FARE, R-GEN, R-PARCEL, R-VOUCHER, R-ARCH — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-011 Tổng quan 5 test level](/articles/tong-quan-5-test-level-istqb) · [TEST-001 Software Testing là gì?](/articles/software-testing-la-gi-testing-khac-debugging) (test tìm failure tính cước vs debugging) · [TEST-008 Kiểm thử trong SDLC](/articles/kiem-thu-trong-sdlc-agile-waterfall) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)

Bài tiếp theo: [TEST-013 · Component Integration Testing](/articles/component-integration-testing).
