---
slug: unit-test-design-pricing-engine
title: "Unit Test Design cho Pricing Engine: từ biên quãng đường đến oracle tiền cước"
summary: "Kết hợp phân vùng tương đương, giá trị biên và bảng quyết định để thiết kế 36 ca kiểm thử pricing Tới Nơi: chuẩn hoá quãng đường, cao điểm, mưa, voucher, cước sàn và tiền lẻ."
category: testing
level: advanced
tags: [testing, istqb, ctfl, test-056, unit-testing, pricing, boundary-value-analysis, decision-table, vitest]
language: vi
---

Năm test ở bài trước giúp bắt đầu, nhưng vẫn để hở những câu hỏi đáng tiền: 0,45 km có hợp lệ không? Voucher có làm mất cước tối thiểu không? Hai cách tính khác nhau có thể cùng trả 18.000 đ sau làm tròn không? Bài này biến các quy tắc của Pricing Engine thành một bộ test có lý do chọn dữ liệu, thay vì tiếp tục thêm ca theo cảm giác.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-056</span>
<table><tbody>
<tr><th scope="row">Series</th><td>S06 · Unit Testing &amp; Component Testing</td></tr>
<tr><th scope="row">ISTQB Level / Knowledge Level</th><td>CTFL v4.0.1 / L3 · Advanced thực hành</td></tr>
<tr><th scope="row">Test Level / Type</th><td>Component / Functional, Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Equivalence Partitioning, Boundary Value Analysis, Decision Table Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/unit-testing-fundamentals">TEST-055</a> · <a href="/articles/boundary-value-analysis">TEST-032 · BVA</a> · <a href="/articles/decision-table-testing">TEST-033 · Bảng quyết định</a></td></tr>
<tr><th scope="row">Tham chiếu</th><td>CTFL 4.2.1–4.2.3; FL-4.2.1, FL-4.2.2, FL-4.2.3 (K3). Tổ chức mã TypeScript và oracle tiền là thực hành mở rộng.</td></tr>
<tr><th scope="row">Lab</th><td><code>examples/toi-noi-testing/articles/unit-test-design-pricing-engine/</code></td></tr>
</tbody></table>
</div>

## 1. Giới thiệu

Một bộ test tốt phải giải thích được vì sao chọn 0,44 và 0,45 thay cho hai giá trị ngẫu nhiên. Với pricing, sai một ngưỡng hoặc sai thứ tự phép toán có thể ảnh hưởng mọi chuyến trong một nhóm khách.

Ta kiểm ba ranh giới: đầu vào JSON qua schema; chuẩn hoá nghiệp vụ trong `normalizePriceRequest`; tính tiền trong `priceRequest` và `calculateFare`. Không cần HTTP hoặc DB để kiểm các phần này. Tuy vậy, việc gọi helper thật bên trong ranh giới tính giá phải được ghi rõ; đây không phải bộ test cô lập từng hàm bằng mock.

Phạm vi chính là cước đặt xe. Bưu kiện, đồ ăn và hiệu lực voucher cần các nhóm test riêng, không suy ra đã được phủ chỉ vì cùng đi qua file `pricing.ts`.

## 2. Lý thuyết và nền tảng ISTQB

CTFL 4.2 cung cấp ba kỹ thuật dùng ở đây: phân vùng tương đương chọn đại diện cho những nhóm đầu vào dự kiến được xử lý giống nhau; phân tích giá trị biên tập trung vào ranh giới của các phân vùng có thứ tự; bảng quyết định nối tổ hợp điều kiện với kết quả. Các LO tương ứng yêu cầu áp dụng kỹ thuật để thiết kế test.

Trong bài, **test condition** là điều cần kiểm, chẳng hạn “quãng đường sau chuẩn hoá vượt 100 km”. **Test case** cụ thể hoá nó thành input 100,05 và lỗi mong đợi. **Oracle** là kết quả được tính từ SPEC. **Coverage item** phụ thuộc kỹ thuật: phân vùng, giá trị biên hay cột quyết định; không được gộp chúng thành một tỷ lệ “coverage” thiếu định nghĩa.

Ba kỹ thuật bổ sung nhau. Một ca 2 km đại diện cho khoảng hợp lệ không thay thế ca sát giới hạn; phủ các biên cũng chưa chứng minh thứ tự cao điểm → mưa → voucher đúng.

## 3. Tình huống kiểm thử tại Tới Nơi

Pricing Engine trả tổng phải thu và bảng chi tiết. Nhóm cần bảo vệ cả hai: tổng đúng nhưng bảng chi tiết làm tròn sai vẫn vi phạm hợp đồng hiển thị giá.

| Yêu cầu | Điều cần giữ |
|---|---|
| R-FARE-02 | Làm tròn quãng đường half-up đến 0,1 km, rồi kiểm khoảng [0,5; 100] |
| R-FARE-03 | Cước cơ bản → cao điểm → mưa → voucher → cước tối thiểu → làm tròn |
| R-FARE-06 / R-VOUCHER-06 | Giảm theo phần trăm trên tiền đã gồm mưa, có trần giảm |
| R-FARE-07 | Voucher không kéo số tiền cuối xuống dưới cước sàn |
| R-FARE-08 | Làm tròn lên bội số 1.000 đ |
| R-GEN-01 / R-FARE-10 | Giữ phần lẻ ở bước trung gian và trả bảng chi tiết |
| R-VALID-01 | Sau định dạng, lỗi thành phố được ưu tiên trước lỗi quãng đường |

Có một chỗ trong đề cương cần đọc cùng SPEC: “voucher vượt tổng tiền” không có nghĩa là tự tạo voucher giảm 200%. Voucher của lab có `percent` từ 1 đến 50, nên không thể giảm quá toàn bộ số tiền hợp lệ chỉ bằng phần trăm. Rủi ro hợp lệ cần kiểm là **tiền sau giảm thấp hơn cước sàn**. Đầu vào sai hợp đồng phải được đặt vào một bài kiểm validation riêng, không dùng làm ca nghiệp vụ bình thường.

## 4. Phân tích kỹ thuật và thiết kế test

### 4.1. Phân vùng và biên của quãng đường

Phân vùng nghiệp vụ xét **giá trị đã làm tròn**, không chỉ con số khách gửi:

| Input thô (km) | Sau chuẩn hoá | Kết quả | Lý do chọn |
|---:|---:|---|---|
| 0,44 | 0,4 | `DISTANCE_OUT_OF_RANGE` | Ngay dưới ngưỡng nhận khi xét độ phân giải 0,01 km |
| 0,45 | 0,5 | Hợp lệ | Ngưỡng làm tròn lên mức nhỏ nhất |
| 0,50 | 0,5 | Hợp lệ | Đúng mức nhỏ nhất sau chuẩn hoá |
| 0,55 | 0,6 | Hợp lệ | Nhảy sang mức 0,1 km tiếp theo |
| 99,94 | 99,9 | Hợp lệ | Phía dưới bước làm tròn về 100 |
| 99,95 | 100 | Hợp lệ | Ngưỡng làm tròn về giới hạn trên |
| 100,00 | 100 | Hợp lệ | Đúng giới hạn trên |
| 100,04 | 100 | Hợp lệ | Input thô vượt 100 nhưng sau làm tròn vẫn hợp lệ |
| 100,05 | 100,1 | Lỗi | Bắt đầu vượt giới hạn sau làm tròn |

Đây là tập biên thực dụng có nêu độ phân giải, không tuyên bố phủ mọi số thực lân cận. Bổ sung 7,35 → 7,4 để kiểm half-up trong vùng giữa; `-1`, `NaN`, `Infinity`, `-Infinity` để kiểm đầu vào số không dùng được. `NaN` và vô cực không phải số JSON hợp lệ; chúng chỉ có ý nghĩa ở lời gọi domain trực tiếp.

### 4.2. Bảng quyết định cho cao điểm, mưa và voucher

Cố định xe máy 2 km, cước cơ bản 21.000 đ. Voucher có mức giảm 20%, trần 100.000 đ, ngưỡng tối thiểu 0. Mọi cột dưới đều khả thi:

| Cao điểm | Mưa | Voucher | Tiền trước giảm | Sau giảm | Tổng sau làm tròn |
|---|---|---|---:|---:|---:|
| Không | Không | Không | 21.000 | 21.000 | 21.000 |
| Không | Không | Có | 21.000 | 16.800 | 17.000 |
| Không | Có | Không | 29.000 | 29.000 | 29.000 |
| Không | Có | Có | 29.000 | 23.200 | 24.000 |
| Có | Không | Không | 26.250 | 26.250 | 27.000 |
| Có | Không | Có | 26.250 | 21.000 | 21.000 |
| Có | Có | Không | 34.250 | 34.250 | 35.000 |
| Có | Có | Có | 34.250 | 27.400 | 28.000 |

Tám ca phủ tám cột của **bảng đã giới hạn này**. Chúng chưa phủ phân vùng chạm trần giảm, cước tối thiểu, loại xe hoặc điều kiện voucher hết hạn. Không nhân mọi chiều đầu vào với nhau khi chưa xác định rủi ro; thêm ca có mục đích cho từng quy tắc còn hở.

### 4.3. Oracle nhiều bước và hiện tượng lỗi bị che

R-FARE-09 cho sẵn một ví dụ độc lập: xe máy 7,4 km, cao điểm, mưa, giảm 20% không chạm trần. Chuỗi tiền là:

```text
45.300 → 56.625 → 64.625 → giảm 12.925 → 51.700 → 52.000 đ
```

Ta kiểm từng mốc bằng hằng số trong assertion. Không gọi `percentOf` của sản phẩm để tính expected; nếu helper có lỗi, oracle dùng chung helper sẽ yếu đi.

R-FARE-10 cần một ca khác: xe máy 0,5 km có base 14.250; cao điểm cho 17.812,5; tổng làm tròn là 18.000. Nếu mã làm tròn sớm thành 17.813 thì tổng vẫn 18.000. Vì vậy chỉ assert `total` sẽ bỏ lọt lỗi bảng chi tiết; phải assert `afterPeak` chính xác.

### 4.4. Kiểm đúng tầng

| Đầu vào / hành vi | Nơi kiểm trong bài | Không suy ra từ kết quả |
|---|---|---|
| `distanceKm: null`, chuỗi, boolean hoặc thiếu | `quoteSchema.safeParse` | HTTP status và nội dung response |
| Thành phố sai, quãng đường cũng sai | `normalizePriceRequest` | Thứ tự middleware xác thực |
| Cờ peak/raining đã xác định | `priceRequest` | Độ đúng của đồng hồ và dịch vụ thời tiết |
| Điều khoản voucher đã hợp lệ | `calculateFare` | Ngày hết hạn, lượt dùng, phạm vi thành phố |

Ép `null as unknown as number` rồi yêu cầu domain trả lỗi validation của HTTP là trộn hai hợp đồng. TypeScript chỉ giúp người gọi trong mã đã kiểm kiểu; dữ liệu ngoài tiến trình vẫn cần schema lúc chạy.

## 5. Thực hành

Từ thư mục lab, chạy:

```bash
npx vitest run articles/unit-test-design-pricing-engine
```

File `pricing.test.ts` chứa 36 ca: 14 ca quãng đường; 8 cột quyết định; 8 ca oracle, cước sàn, trần giảm và làm tròn; 6 ca schema cùng điều phối. Đây là số lượng ca, không phải tỷ lệ yêu cầu đã phủ. Kiểm chứng ngày 07/11/2026 với Vitest 3.2.7: cả 36 ca đạt trên mã lab hiện tại.

### Test tham số hoá với expected tường minh

Đoạn dưới rút hai ca từ nhóm biên; bản đầy đủ nằm trong thư mục bài:

```ts
import { expect, it } from 'vitest';
import { normalizeDistanceKm } from '../../src/domain/fare';

it.each([[0.45, 0.5], [100.04, 100]])(
  '%s km được chuẩn hoá thành %s km',
  (raw, expected) => {
    expect(normalizeDistanceKm(raw)).toBe(expected);
  },
);

it('từ chối quãng đường sau làm tròn vượt 100 km', () => {
  expect(() => normalizeDistanceKm(100.05)).toThrowError(
    expect.objectContaining({ code: 'DISTANCE_OUT_OF_RANGE' }),
  );
});
```

Truyền callback cho assertion lỗi để runner có thể quan sát exception. Kiểm mã lỗi ổn định giúp tránh failure chỉ vì sửa câu thông báo tiếng Việt; đồng thời không chấp nhận nhầm một `TypeError` bất kỳ.

### Gọi qua lớp điều phối pricing

Tám ca bảng quyết định gọi `normalizePriceRequest` rồi `priceRequest`. Như vậy test cũng quan sát việc điều phối từ yêu cầu đặt xe sang tính cước. Nó vẫn không chạm API mạng:

```ts
const request = normalizePriceRequest({
  kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 2,
});
const result = priceRequest(request, {
  peak: true,
  raining: true,
  voucher: { code: 'TEST20', percent: 20, maxDiscount: 100_000, minAmount: 0 },
});
expect(result.total).toBe(28_000);
```

Nếu nhóm chọn ranh giới unit là một hàm duy nhất, hãy gọi nhóm này là test cộng tác giữa các thành phần thuần. Tên gọi cần nhất quán, nhưng điều quan trọng là nói rõ có những thành phần nào chạy thật và bằng chứng dừng ở đâu.

### Đơn vị tiền phải hiện rõ

`roundUpToThousandDong` nhận **milli-đồng**, trả đồng. Ba ca 51.700 → 52.000, 52.000 → 52.000 và 52.001 → 53.000 dùng input lần lượt 51.700.000, 52.000.000 và 52.001.000 milli-đồng. Truyền 52.001 trực tiếp sẽ kiểm sai đơn vị chứ không kiểm đúng biên mong muốn.

Với số 17.812,5 trong ví dụ, `toBe` phù hợp: phần lẻ 0,5 biểu diễn chính xác và hợp đồng đòi giữ nguyên mốc này. Không dùng dung sai rộng chỉ để bỏ qua sai số tiền; nếu một hợp đồng cho phép dung sai thì phải nêu nguồn và đơn vị của dung sai đó.

## 6. Những điểm nâng cao và giới hạn

**Phủ bảng quyết định không đồng nghĩa phủ mọi nhánh code.** Bài này thiết kế từ yêu cầu; sau khi chạy có thể dùng coverage để tìm phần cài đặt chưa được chạm đến. Không đổi bộ dữ liệu chỉ để nâng tỷ lệ nếu chưa có oracle hữu ích.

**Giảm dưới sàn và chạm trần là hai rủi ro khác nhau.** Với bike 0,5 km, giảm 50% khiến 14.250 còn 7.125 nhưng tổng phải là 15.000. Với bike 2 km, giảm 20% nhưng trần 3.000 thì chỉ giảm 3.000 và tổng là 18.000. Ca đầu không thay ca sau.

**Ngày giờ và thời tiết đang được cố định bằng dữ liệu đầu vào.** Một lỗi xác định `peak` ở 13:00 sẽ không bị bộ test này phát hiện. Cần test `time.ts` và một số test luồng báo giá với đồng hồ tiêm vào; không cần biến tất cả ca số học thành test hệ thống.

**Không suy rộng sang toàn bộ sản phẩm.** Bộ này chưa kiểm phí khối lượng, giới hạn giỏ đồ ăn, vòng đời voucher hay ghi báo giá vào DB. Schema test không thay thế test API. Đây là lớp hồi quy cho ranh giới giá đặt xe đã nêu, không phải chứng nhận Pricing Engine hoàn chỉnh.

**Làm tròn có thể che defect.** Hãy ưu tiên thêm assertion ở mốc có ý nghĩa nghiệp vụ, như `afterPeak`, thay vì snapshot toàn bộ object. Snapshot dài dễ được chấp nhận lại mà không kiểm phép tính.

## 7. Bài tập và tự đánh giá

### Bài tập

1. Thêm ca cước ô tô 0,5 km, không cao điểm, không mưa, không voucher; ghi base và total.
2. Voucher giảm 20%, trần 3.000 đ: chọn một ca chưa chạm trần và một ca vừa chạm trần theo số tiền áp voucher. Nêu rõ bạn kiểm helper voucher hay toàn luồng cước, vì không phải mọi số tiền đều sinh ra từ quãng đường hợp lệ.
3. Tạo bản thử nghiệm riêng làm tròn `afterPeak` thành số nguyên đồng. Test nào trong bài bắt được lỗi dù tổng không đổi?
4. Bổ sung test ở tầng API cho `distanceKm: null`. Xác định assertion nào còn thiếu trong test schema hiện tại.

<details><summary>Gợi ý đáp án</summary>
<p>1. Base = 25.000 + 11.000 × 0,5 = 30.500; total = 31.000. 2. Ở helper voucher, tiền áp voucher 14.000 cho giảm 2.800; 15.000 cho giảm 3.000, với minAmount = 0. 3. Ca R-FARE-10 phải giữ afterPeak = 17.812,5. 4. Cần response HTTP 400 và mã VALIDATION_ERROR theo hợp đồng API, cùng cách bố trí user hợp lệ để không bị lỗi xác thực che mất lỗi định dạng.</p>
</details>

### Câu hỏi tự luyện

**Câu 1.** Vì sao 100,04 km được chấp nhận?

- A. Domain bỏ qua giới hạn trên.
- B. SPEC yêu cầu làm tròn thành 100,0 trước khi kiểm khoảng.
- C. Mọi số nhỏ hơn 101 đều hợp lệ.
- D. TypeScript tự làm tròn dữ liệu.

<details><summary>Đáp án: B</summary><p>Thứ tự chuẩn hoá rồi validation là một phần của yêu cầu. Ca 100,05 giúp phân biệt với giả thuyết sai ở phương án C.</p></details>

**Câu 2.** Tám ca bảng quyết định đều đạt. Kết luận nào phù hợp?

- A. Mọi quy tắc voucher đều đúng.
- B. Toàn bộ `pricing.ts` đạt 100% branch coverage.
- C. Tám tổ hợp đã chọn cho bike 2 km cho tổng đúng theo oracle trong bảng.
- D. Không cần kiểm cước sàn nữa.

<details><summary>Đáp án: C</summary><p>Bảng cố định loại xe, quãng đường và điều khoản voucher. Kết luận phải giữ nguyên phạm vi đó; branch coverage cần một phép đo riêng.</p></details>

**Câu 3.** Tổng vẫn là 18.000 nhưng `afterPeak` trả 17.813 thay vì 17.812,5. Có phải defect không?

- A. Không, khách trả đúng tổng.
- B. Có, R-GEN-01 và R-FARE-10 yêu cầu giữ phần lẻ trung gian.
- C. Chỉ khi giao diện dùng tiếng Anh.
- D. Chỉ khi coverage dưới 100%.

<details><summary>Đáp án: B</summary><p>Bảng chi tiết là một đầu ra được đặc tả; tổng cuối đúng không xoá vi phạm ở đầu ra khác.</p></details>

Các câu hỏi là bài tự luyện do tác giả biên soạn, không phải câu hỏi thi ISTQB chính thức.

## 8. Tài liệu tham khảo

- [ISTQB CTFL Syllabus v4.0.1](https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf): mục 4.2.1–4.2.3. Quyền tác giả thuộc ISTQB và các tác giả trong syllabus.
- [Vitest — Test API](https://vitest.dev/api/test): `it.each` và tổ chức các ca dữ liệu. Dùng API tương thích phiên bản khoá trong lab.
- [Lab Tới Nơi](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing): SPEC R-FARE, R-GEN-01, R-VOUCHER-06, R-VALID-01; `fare.ts`, `pricing.ts`, `money.ts`, `api/schemas.ts`.

Bài trước: [TEST-055 · Unit Testing Fundamentals](/articles/unit-testing-fundamentals). Bài tiếp theo: [TEST-057 · Test Doubles — Dummy, Stub, Fake, Mock, Spy](/articles/test-doubles-dummy-stub-fake-mock-spy).
