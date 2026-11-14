---
slug: system-testing-sanity-test
title: "Sanity Test: kiểm đúng vùng thay đổi trước khi mở rộng vòng kiểm thử"
summary: "Sanity testing qua một bản sửa chuẩn hoá voucher của Tới Nơi: phân biệt với smoke, confirmation và regression, dựng impact map từ input API tới lượt dùng voucher, chọn sáu test case có lý do và chạy lab Vitest + supertest với oracle tiền tính tay."
category: testing
level: beginner
tags: [testing, sys-02, sanity-testing, confirmation-testing, regression-testing, impact-analysis, voucher, vitest, toi-noi]
language: vi
---

Khách Tới Nơi dán mã ` giam20 ` từ tin nhắn vào ô voucher — có một dấu cách ở đầu và một ở cuối. Ứng dụng báo «không tìm thấy voucher», dù GIAM20 đang chạy khuyến mãi. Developer thêm `trim()` và báo đã sửa. Câu hỏi của tester lúc này không phải «chạy hết bao nhiêu test», mà là: **bản sửa này có đủ hợp lý để đưa vào vòng kiểm thử chuyên sâu chưa, và cần kiểm những gì để trả lời câu đó?** Bài này dùng tình huống ấy để dựng một sanity suite nhỏ nhưng có lý do cho từng test case: từ hàm chuẩn hoá, qua API báo giá, tới chuyến được tạo và lượt dùng voucher.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-02</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 1</td></tr>
<tr><th scope="row">Cấp độ</th><td>Beginner — cần biết HTTP status, cấu trúc Arrange–Act–Assert và cách chạy Vitest</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>Component + API (hệ thống lắp ráp trong tiến trình); chọn phạm vi theo thay đổi: confirmation cộng regression gần</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-smoke-test">SYS-01 · Smoke Test</a>; nên đọc thêm <a href="/articles/confirmation-testing-va-regression-testing">Confirmation Testing và Regression Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): 6 test case SN-01…SN-06 (8 lượt chạy) cho thay đổi chuẩn hoá voucher R-VOUCHER-02, kiểm tới R-VOUCHER-03/06/07 và R-QUOTE-04 — <code>examples/toi-noi-testing/articles/system-testing-sanity-test/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>ISTQB CTFL v4.0.1 (confirmation/regression, impact analysis); <code>SPEC.md</code> của lab; Vitest</td></tr>
</tbody>
</table>
</div>

## 1. Tình huống, điều kiện tiên quyết và mục tiêu

### 1.1. Một dòng sửa, nhiều câu hỏi

Đường xử lý voucher trong tình huống giả định chỉ viết hoa mã mà không bỏ khoảng trắng hai đầu. Bản sửa thêm `trim`. Nhìn diff thì nhỏ, nhưng mã voucher không dừng ở hàm chuẩn hoá: nó được dùng để tra voucher, tính tiền giảm, lưu vào báo giá, chép sang chuyến khi đặt, rồi được đếm khi kiểm lượt dùng trong ngày. Nếu chỉ chạy lại đúng test case ` giam20 ` ở cấp hàm, ta biết hàm đã đúng — nhưng chưa biết API có thực sự gọi hàm đó, chưa biết mã đã chuẩn hoá có đi đúng tới chuyến không, và chưa biết chuỗi rỗng có bị hiểu thành «mã sai» không.

Ngược lại, chạy toàn bộ bộ test đặt xe, giao hàng, đồ ăn, thanh toán và báo cáo chỉ để đánh giá một dòng `trim` là tốn kém và làm chậm phản hồi. Ta cần một vòng kiểm tập trung, trả lời bốn câu hỏi:

1. Lỗi gốc đã hết chưa — ở cấp hàm **và** ở cấp người dùng thấy (API)?
2. Mã vốn đã đúng (`GIAM20`) có còn dùng được không?
3. Mã rỗng và mã không tồn tại có còn được phân biệt đúng không?
4. Mã sau chuẩn hoá có được lưu và đếm lượt đúng ở bước tiếp theo không?

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Lỗi giả lập phục vụ học tập</span>
<p>Hàm <code>normalizeVoucherCode</code> trong lab hiện đã chuẩn hoá đúng. Bài <b>không</b> khẳng định repository từng có defect này hay đã sửa một bug thật. Phiên bản «trước khi sửa» là một <b>fault fixture</b> đặt trong chính file test của bài; mã sản phẩm dùng chung ở <code>src/</code> không bị thay đổi.</p>
</div>

### 1.2. Mục tiêu đo được

Sau bài này, bạn có thể:

- phân biệt sanity với smoke, confirmation và regression bằng câu hỏi mỗi hoạt động trả lời;
- vẽ impact map cho một thay đổi nhỏ và giải thích vì sao chọn từng ca;
- tính oracle tiền bằng tay từ đặc tả, không lấy kết quả của code đang kiểm;
- chạy lab, đọc 8 lượt pass và nói đúng những gì 8 lượt đó **không** chứng minh;
- viết kết luận sanity đúng phạm vi: «được kiểm tiếp», không phải «được release».

## 2. Khái niệm: sanity là lựa chọn phạm vi theo thay đổi

### 2.1. Định nghĩa dùng trong series

**Sanity testing** trong series này là vòng kiểm ngắn, tập trung vào **một thay đổi cụ thể và những hành vi liên quan trực tiếp**, nhằm đánh giá thay đổi đó có đủ hợp lý để tiếp tục kiểm thử chuyên sâu hay không. Đây là quy ước vận hành của nhóm, không phải một test level độc lập và cũng không phải tên suite bắt buộc ở mọi tổ chức. Syllabus ISTQB CTFL v4.0.1 không định nghĩa «sanity» như một test type riêng; syllabus này phân biệt **confirmation testing** và **regression testing**, và khuyên dùng **impact analysis** để quyết định phạm vi regression. Sanity ở đây là cách ghép hai thứ đó lại cho một thay đổi nhỏ.

Sanity có thể được làm bằng manual testing, component test hay API test — tuỳ thay đổi nằm ở đâu. Một test case có sẵn trong bộ regression có thể được chọn vào sanity của lần sửa này nhưng không cần cho lần sửa khác. Tính chất «sanity» nằm ở **lý do chọn**, không nằm ở bản thân test case.

### 2.2. Bốn hoạt động dễ nhầm

| Hoạt động | Căn cứ chọn test case | Kết luận hợp lệ khi pass | Ví dụ Tới Nơi |
|---|---|---|---|
| Smoke | Khả năng thiết yếu của build/môi trường | Có thể bắt đầu vòng kiểm tiếp theo | Health, chặn người vô danh, đặt → hoàn tất → thu tiền (SYS-01) |
| Confirmation | Điều kiện từng gây ra lỗi đã biết | Lỗi đó không còn tái hiện với ca đã chạy | ` giam20 ` không còn bị báo không tìm thấy |
| Sanity | Thay đổi và vùng ảnh hưởng gần | Thay đổi hoạt động hợp lý trong phạm vi đã chọn | SN-01…SN-06 của bài này |
| Regression | Rủi ro ảnh hưởng bất lợi do thay đổi, rộng hơn | Các hành vi được chọn vẫn đáp ứng yêu cầu | Voucher ở mọi dịch vụ, thành phố, ngày hết hạn, huỷ hoàn lượt |

Một sanity suite thường gồm **test case confirmation cộng một ít regression có chủ đích**. Nó không thay regression đầy đủ. Mẹo nhớ «smoke rộng và nông, sanity hẹp và sâu» có ích nhưng có giới hạn: phạm vi thực tế phải xuất phát từ mục tiêu, rủi ro và kiến trúc, không phải từ số lượng ca. Khi hai nhóm dùng tên khác nhau, hãy ghi mục tiêu và phạm vi vào test plan thay vì tranh luận về chữ.

### 2.3. Khi nào một vòng sanity nhỏ là không đủ?

Độ dài diff không phải thước đo rủi ro. Một dòng sửa trong hàm chuẩn hoá **dùng chung** có thể đổi hành vi của mọi dịch vụ nhận voucher. Thay đổi schema, cơ chế xác thực, kiểu dữ liệu tiền hay thư viện nền có thể cần regression rộng và môi trường riêng dù chỉ vài dòng. Nếu chưa biết ai gọi hàm và ai tiêu thụ đầu ra của nó, việc đầu tiên là **điều tra impact**, chưa phải chọn test case. Gọi suite nhỏ là «đủ» khi chưa làm bước đó là đoán.

## 3. Test basis, fault fixture và oracle độc lập

### 3.1. Yêu cầu liên quan trong SPEC

Đọc `examples/toi-noi-testing/SPEC.md` trước khi đọc implementation. Các yêu cầu chạm tới thay đổi:

| Mã | Nội dung tóm tắt | Vì sao liên quan |
|---|---|---|
| R-VOUCHER-02 | Bỏ khoảng trắng hai đầu, đổi chữ hoa trước khi tra; chuỗi rỗng sau khi bỏ khoảng trắng coi như không nhập voucher | Chính là hành vi được sửa |
| R-VOUCHER-03 | Không có voucher với mã đó ⇒ `VOUCHER_NOT_FOUND` | Không được «nuốt» mã sai thành không dùng mã |
| R-VOUCHER-06 | Tiền giảm = min(percent% × A, maxDiscount), A là số tiền sau bước 3 của R-FARE-03 | Mã tra đúng thì tiền giảm phải đúng |
| R-VOUCHER-07 | Chuyến chưa huỷ trong ngày tiêu lượt; báo giá không tiêu lượt; kiểm lượt cả khi báo giá và khi tạo chuyến | Mã lưu sai dạng có thể làm đếm sót lượt |
| R-QUOTE-04 | Chuyến giữ ảnh chụp giá của báo giá, không tính lại | Chuyến phải mang đúng giá và mã từ báo giá |

Seed của lab định nghĩa GIAM20: giảm 20%, trần 30.000 đ, đơn tối thiểu 0 đ, một lượt mỗi khách mỗi ngày.

### 3.2. Fault fixture: lỗi đặt ở đâu

Phiên bản «trước khi sửa» được biểu diễn bằng đúng một dòng trong file test:

```ts
// Fault fixture: phiên bản giả định trước sửa, cố ý thiếu trim.
// Chỉ dùng để minh họa khả năng phát hiện lỗi, không thay src/ dùng chung.
const beforeFix = (raw: string) => raw.toUpperCase();
```

Với input ` giam20 `, fixture trả ` GIAM20 ` — vẫn còn hai dấu cách, nên khác `GIAM20`. Hàm thật trong `src/domain/voucher.ts`:

```ts
export function normalizeVoucherCode(raw: string | null | undefined): string | null {
  const code = (raw ?? '').trim().toUpperCase();
  return code === '' ? null : code;
}
```

Điểm quan trọng: expected `GIAM20` đến từ **R-VOUCHER-02**, không đến từ việc gọi `normalizeVoucherCode`. Nếu test lấy kết quả của hàm thật làm expected để so với fixture, ta chỉ đang so hai implementation với nhau; khi cả hai cùng sai, test vẫn xanh.

### 3.3. Oracle tiền tính tay

Báo giá dùng xe máy (`bike`), 5 km, thành phố SG, 09:00 giờ Việt Nam ngày 04/11/2026 (không phải cao điểm), không mưa. Theo bảng cước R-FARE-01 và thứ tự R-FARE-03:

| Bước | Có GIAM20 | Không voucher |
|---|---|---|
| 1. Cước cơ bản: 12.000 + 4.500 × 5 | 34.500 | 34.500 |
| 2. Cao điểm (09:00 không thuộc khung) | 34.500 | 34.500 |
| 3. Phụ phí trời mưa (không mưa) | 34.500 | 34.500 |
| 4. Trừ voucher: min(20% × 34.500; 30.000) = 6.900 | 27.600 | — |
| 5. Cước tối thiểu xe máy 15.000 | 27.600 | 34.500 |
| Làm tròn lên bội 1.000 (R-FARE-08) | **28.000** | **35.000** |

Một implementation sai thứ tự — làm tròn trước rồi giảm 20% trên 35.000 — sẽ ra giảm 7.000 và 28.000 sau làm tròn: **tổng trùng nhau** dù cách tính sai. Vì vậy test kiểm cả `discount` (6.900) lẫn `total` (28.000). Nếu chỉ kiểm tổng thì phép làm tròn sẽ che mất lỗi.

## 4. Impact map và lựa chọn test case

### 4.1. Lần theo dữ liệu, không lần theo file

Thay vì hỏi «file nào bị sửa», hãy hỏi «giá trị bị đổi đi đâu tiếp». Mã voucher đi qua chuỗi: input API → chuẩn hoá → tra voucher → tính giá → lưu báo giá → tạo chuyến → đếm lượt dùng. Mỗi mắt xích là một chỗ thay đổi có thể đúng ở trước nhưng hỏng ở sau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 250" role="img" aria-label="Impact map của thay đổi chuẩn hoá voucher. Chuỗi bảy bước từ trái sang phải: input API, chuẩn hoá mã, tra voucher, tính giá, lưu báo giá, tạo chuyến, đếm lượt dùng. Bước chuẩn hoá được tô đậm là nơi thay đổi. Bên dưới, các ca SN-01 đến SN-06 được gắn vào đoạn chuỗi mà chúng phủ: SN-01 chỉ phủ chuẩn hoá; SN-02, SN-03, SN-04 phủ từ input API tới tính giá; SN-05 phủ tới tra voucher; SN-06 phủ toàn chuỗi tới đếm lượt dùng. Dòng cuối ghi các vùng chưa chọn.">
<defs>
<marker id="sys02-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<rect x="10" y="20" width="110" height="48" rx="8" class="f-phys-soft s-phys"/><text x="65" y="49" text-anchor="middle">input API</text>
<rect x="140" y="20" width="110" height="48" rx="8" class="f-warn-soft s-warn"/><text x="195" y="42" text-anchor="middle" class="t-b">chuẩn hoá</text><text x="195" y="59" text-anchor="middle" class="t-small t-warn">thay đổi</text>
<rect x="270" y="20" width="110" height="48" rx="8" class="f-pk-soft s-pk"/><text x="325" y="49" text-anchor="middle">tra voucher</text>
<rect x="400" y="20" width="110" height="48" rx="8" class="f-pk-soft s-pk"/><text x="455" y="49" text-anchor="middle">tính giá</text>
<rect x="530" y="20" width="110" height="48" rx="8" class="f-pk-soft s-pk"/><text x="585" y="49" text-anchor="middle">lưu báo giá</text>
<rect x="660" y="20" width="110" height="48" rx="8" class="f-pk-soft s-pk"/><text x="715" y="49" text-anchor="middle">tạo chuyến</text>
<rect x="790" y="20" width="120" height="48" rx="8" class="f-pk-soft s-pk"/><text x="850" y="49" text-anchor="middle">đếm lượt dùng</text>
<line x1="120" y1="44" x2="136" y2="44" class="s-pk" stroke-width="1.6" marker-end="url(#sys02-arrow)"/>
<line x1="250" y1="44" x2="266" y2="44" class="s-pk" stroke-width="1.6" marker-end="url(#sys02-arrow)"/>
<line x1="380" y1="44" x2="396" y2="44" class="s-pk" stroke-width="1.6" marker-end="url(#sys02-arrow)"/>
<line x1="510" y1="44" x2="526" y2="44" class="s-pk" stroke-width="1.6" marker-end="url(#sys02-arrow)"/>
<line x1="640" y1="44" x2="656" y2="44" class="s-pk" stroke-width="1.6" marker-end="url(#sys02-arrow)"/>
<line x1="770" y1="44" x2="786" y2="44" class="s-pk" stroke-width="1.6" marker-end="url(#sys02-arrow)"/>
<rect x="140" y="86" width="110" height="22" rx="5" class="f-warn-soft s-warn"/><text x="195" y="102" text-anchor="middle" class="t-small">SN-01 · cấp hàm</text>
<rect x="10" y="116" width="500" height="22" rx="5" class="f-pk-soft s-pk"/><text x="260" y="132" text-anchor="middle" class="t-small">SN-02 · SN-03 · SN-04 (3 dataset) · qua API báo giá, kiểm mã + discount + total</text>
<rect x="10" y="146" width="370" height="22" rx="5" class="f-pk-soft s-pk"/><text x="195" y="162" text-anchor="middle" class="t-small">SN-05 · mã không tồn tại ⇒ 422 VOUCHER_NOT_FOUND</text>
<rect x="10" y="176" width="900" height="22" rx="5" class="f-pk-soft s-pk dash"/><text x="460" y="192" text-anchor="middle" class="t-small">SN-06 · báo giá có khoảng trắng → tạo chuyến giữ GIAM20 + 28.000 → báo giá lại ⇒ 422 VOUCHER_USAGE_LIMIT</text>
<text x="460" y="234" text-anchor="middle" class="t-small t-warn">chưa chọn: dịch vụ/thành phố khác, hết hạn, trần giảm, huỷ hoàn lượt, concurrency, thanh toán, browser</text>
</svg>
</div>
<figcaption><b>Hình 1 · Impact map của thay đổi chuẩn hoá voucher.</b> Mỗi thanh ngang là đoạn chuỗi một ca đi qua. Chỉ SN-01 dừng ở cấp hàm; SN-06 là ca duy nhất phủ tới lượt dùng — lý do bài chọn một ca có trạng thái dù thay đổi gốc chỉ là xử lý chuỗi.</figcaption>
</figure>

### 4.2. Bảng test case

| ID | Basis / rủi ro | Precondition | Input và hành động | Expected | Vai trò | Cleanup |
|---|---|---|---|---|---|---|
| SN-01 | R-VOUCHER-02; thiếu trim | Không cần app | Gọi fixture và hàm thật với ` giam20 ` | Fixture ≠ `GIAM20`; hàm thật = `GIAM20` | Chứng minh oracle phân biệt được lỗi gốc | Không có |
| SN-02 | R-VOUCHER-02/06; chuẩn hoá không nối vào API | System mới, C001, 09:00 VN 04/11/2026 | POST `/api/quotes` với ` giam20 ` | 200; `voucherCode` GIAM20; discount 6.900; total 28.000 | Confirmation ở API | Bỏ instance |
| SN-03 | Bản sửa làm hỏng mã chuẩn | Như SN-02 | POST với `GIAM20` | Như SN-02 | Regression gần | Bỏ instance |
| SN-04 | R-VOUCHER-02; nhầm rỗng với mã sai | Như SN-02 | Ba dataset: `''`, `'   '`, không gửi field | 200; `voucherCode` null; discount 0; total 35.000 | Phân vùng đầu vào | Bỏ instance |
| SN-05 | R-VOUCHER-03; nuốt lỗi mã sai | Như SN-02 | POST với ` khongco ` | 422; `VOUCHER_NOT_FOUND` | Negative path | Bỏ instance |
| SN-06 | R-VOUCHER-07, R-QUOTE-04; mã chuẩn bị mất khi đặt | Như SN-02 | Báo giá ` giam20 ` → tạo chuyến bằng quoteId → báo giá lại `GIAM20` | Chuyến: fare 28.000, voucherCode GIAM20; báo giá sau: 422 `VOUCHER_USAGE_LIMIT` | Hợp đồng downstream | Bỏ instance |

SN-04 dùng ba giá trị đại diện cho một phân vùng «không dùng voucher» theo R-VOUCHER-02 — kỹ thuật [phân vùng tương đương](/articles/equivalence-partitioning). Ba giá trị khác nhau ở **cách** đến được phân vùng (field vắng, chuỗi rỗng, chuỗi chỉ có khoảng trắng); trong đó `'   '` là giá trị trực tiếp liên quan tới `trim`.

### 4.3. Những gì cố ý không chọn — và đi đâu

Không chọn: mọi tổ hợp thành phố/dịch vụ, ngày hết hạn (R-VOUCHER-04/05), biên trần giảm, huỷ chuyến hoàn lượt, concurrency, thanh toán và browser. Bỏ khỏi sanity **không** có nghĩa rủi ro được miễn: các vùng đó thuộc functional/regression (SYS-03, SYS-06) hoặc data integrity (SYS-13). Quan trọng hơn, bảng này chỉ hợp lệ với **giả định diff chỉ chạm normalization**. Nếu diff thật chạm `voucherDiscountMilli`, `assertVoucherEligible` hay repository, phải mở rộng phạm vi ngay — sanity được thiết kế theo thay đổi, nên thay đổi khác thì suite khác.

### 4.4. Entry và exit criteria

**Entry:** yêu cầu chuẩn hoá đã rõ (R-VOUCHER-02), biết diff giả định, seed và clock xác định, lab chạy được. Nếu smoke nền (SYS-01) đỏ ở phần sanity cần dùng, hãy sửa môi trường hoặc ghi blocked trước khi giải thích kết quả sanity.

**Exit:** đủ 8 lượt pass, không skip, assertion không bị nới lỏng, các vùng chưa kiểm được ghi rõ. Có failure chưa giải thích được ⇒ chưa đạt gate. Đạt gate ⇒ **được kiểm tiếp**, không tự động được release.

## 5. Thực hành: từ assertion nhỏ tới API có trạng thái

Mã đầy đủ: [`articles/system-testing-sanity-test/sanity.test.ts`](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-sanity-test). Từ thư mục `examples/toi-noi-testing`, Node ≥ 22.12:

```sh
npm ci
npm test -- articles/system-testing-sanity-test
npm run typecheck
```

`npm ci` chỉ cần khi chuẩn bị hoặc cập nhật dependency theo lockfile. Không cần DB hay server ngoài: mỗi ca dựng một system bộ nhớ mới với đồng hồ cố định, ví và thời tiết giả.

### 5.1. Dựng môi trường cho từng ca

```ts
let app: ReturnType<typeof createApp>;
const ride = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 };
beforeEach(() => {
  const system = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
  app = createApp(system.deps, { logErrors: false });
});
const quote = (voucherCode?: string) => request(app).post('/api/quotes')
  .set('x-user', 'customer:C001').send({ ...ride, ...(voucherCode === undefined ? {} : { voucherCode }) });
```

Ba chi tiết đáng chú ý. `beforeEach` tạo system mới, nên lượt voucher mà SN-06 sử dụng không rò sang test case khác — đó cũng là «cleanup» của bài. `FixedClock` giữ 09:00 để cao điểm không làm đổi oracle. Và helper `quote` gửi `voucherCode` **nguyên trạng**: nếu helper tự `trim()` cho «gọn», ca sẽ xanh ngay cả khi server còn lỗi — test đã sửa hộ sản phẩm. Khi `voucherCode` là `undefined`, helper không gửi field, để phủ đúng dataset «không nhập».

### 5.2. Bước 1 — Oracle có phân biệt được đúng và sai không?

```ts
it('SN-01 · R-VOUCHER-02: oracle bắt bản thiếu trim và chấp nhận hàm hiện tại', () => {
  const input = ' giam20 ';
  expect(beforeFix(input)).not.toBe('GIAM20');
  expect(normalizeVoucherCode(input)).toBe('GIAM20');
});
```

Test case này xanh có chủ đích. Assertion đầu đòi fixture **khác** oracle — tức oracle đủ nhạy để bắt lỗi thiếu trim. Assertion sau đòi hàm hiện tại **bằng** oracle. Đây không phải log của một build sản phẩm lỗi đã chạy đỏ rồi sửa xanh. Muốn tự thấy red/green, trên bản sao riêng của test hãy đổi `.not.toBe` thành `.toBe`, chạy riêng SN-01, quan sát failure rồi hoàn nguyên; không commit một assertion cố ý fail.

### 5.3. Bước 2 — API có thực sự dùng hành vi đúng không?

```ts
it('SN-02 · R-VOUCHER-02/06: đầu vào lỗi gốc qua API trả mã chuẩn và 28.000 đ', async () => {
  const res = await quote(' giam20 ').expect(200);
  expect(res.body.quote).toMatchObject({ voucherCode: 'GIAM20', discount: 6_900, total: 28_000 });
});
```

Hàm chuẩn hoá đúng chưa có nghĩa API gọi nó: endpoint có thể tra voucher bằng input thô, hoặc một bước validation phía trước có thể từ chối chuỗi có khoảng trắng. SN-02 kiểm ba thứ cùng lúc — mã chuẩn được trả về, tiền giảm đúng 6.900 và tổng đúng 28.000 — thay vì chỉ kiểm «không bị lỗi». SN-03 lặp lại assertion với `GIAM20` để chắc bản sửa không làm hỏng đường vốn đã đúng. SN-04 và SN-05 kiểm hai phía của ranh giới «không có mã» và «mã sai»:

```ts
it.each(['', '   ', undefined])('SN-04 · mã rỗng/không nhập (%s) vẫn báo giá không giảm', async (code) => {
  const res = await quote(code).expect(200);
  expect(res.body.quote).toMatchObject({ voucherCode: null, discount: 0, total: 35_000 });
});
it('SN-05 · R-VOUCHER-03: mã không tồn tại không biến thành không dùng mã', async () => {
  const res = await quote(' khongco ').expect(422);
  expect(res.body.error.code).toBe('VOUCHER_NOT_FOUND');
});
```

Một bản sửa vụng — ví dụ «nếu tra không thấy thì coi như không dùng voucher» — sẽ làm SN-05 trả 200 với giá 35.000. Khách không được báo mã sai, và nhóm mất tín hiệu về mã khuyến mãi gõ nhầm. Đó là lý do SN-05 thuộc sanity dù không chứa khoảng trắng bên trong mã.

### 5.4. Bước 3 — Dữ liệu sau thay đổi có được tiêu thụ đúng không?

```ts
it('SN-06 · R-VOUCHER-07/R-QUOTE-04: mã chuẩn được giữ khi đặt và tính lượt dùng', async () => {
  const first = await quote(' giam20 ').expect(200);
  const trip = await request(app).post('/api/trips').set('x-user', 'customer:C001')
    .set('Idempotency-Key', 'sys02-sanity-001').send({ quoteId: first.body.quote.quoteId }).expect(201);
  expect(trip.body.trip).toMatchObject({ fare: 28_000, voucherCode: 'GIAM20' });
  const next = await quote('GIAM20').expect(422);
  expect(next.body.error.code).toBe('VOUCHER_USAGE_LIMIT');
});
```

Test case này dùng `quoteId` thật do API trả về, không hardcode. Chuyến phải mang `fare` 28.000 và `voucherCode` GIAM20 (R-QUOTE-04: giữ ảnh chụp giá). Vì chuyến chưa huỷ đã tiêu lượt duy nhất trong ngày, báo giá tiếp theo với `GIAM20` phải bị từ chối `VOUCHER_USAGE_LIMIT` (R-VOUCHER-07 kiểm lượt ngay khi báo giá). Giả sử chuyến lưu mã ở dạng ` GIAM20 ` còn khoảng trắng: phần đếm theo mã chuẩn có thể không thấy chuyến đó và cho khách dùng thêm lượt. Thay đổi gốc chỉ là xử lý chuỗi, nhưng hậu quả nằm ở trạng thái — vì vậy sanity cần một ca có trạng thái.

### 5.5. Mong đợi và thực tế

**Expected** (theo README lab): 8 tests pass — sáu ID test case, trong đó SN-04 chạy ba dataset, nên runner báo 8 lượt chứ không phải 6. Không skip.

**Observed** — báo cáo SYS-02 ngày 13/11/2026 (Windows, Node v24.15.0, npm 11.12.1, Vitest 3.2.7): chạy chung với smoke SYS-01 được 2 file, 11 tests pass trong 4,01 giây; riêng sanity 8 tests pass trong 160 ms; `npm run typecheck` exit 0.

Chạy lại khi biên tập bài, cùng ngày 13/11/2026 (Windows, Node v22.18.0, Vitest 3.2.7), lệnh `npx vitest run articles/system-testing-sanity-test`:

```text
 ✓ articles/system-testing-sanity-test/sanity.test.ts (8 tests) 538ms
 Test Files  1 passed (1)
      Tests  8 passed (8)
   Duration  5.47s (transform 334ms, setup 0ms, collect 3.72s, tests 538ms, environment 0ms, prepare 436ms)
```

Thời gian khác nhau giữa hai lượt (160 ms so với 538 ms phần test; lượt sau có SN-02 chiếm 417 ms vì là request đầu tiên làm nóng app) — con số thời gian phụ thuộc máy và lượt chạy, không phải tiêu chí pass. Tiêu chí là số lượt pass, không skip và assertion nguyên vẹn.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">8 lượt xanh chưa nói gì</span>
<p>Lab dùng kho bộ nhớ, đồng hồ/thời tiết/ví giả và danh tính qua header <code>x-user</code> (giản lược R-API-03, không phải xác thực thật). Lab này <b>không</b> chứng minh: query, collation hay transaction của PostgreSQL; hai request đồng thời tranh một lượt voucher; trình duyệt; Unicode ngoài ASCII; lịch sử một bug thật. Suite cũng không chạy một build API lỗi rồi sửa — lỗi chỉ được so ở cấp hàm qua fixture.</p>
</div>

## 6. Đọc thất bại và ra quyết định

Khi sanity đỏ, việc đầu tiên là xác định **bước đầu tiên lệch** trên impact map, không phải sửa expected cho xanh.

| Thất bại | Cần phân biệt | Bước điều tra |
|---|---|---|
| SN-01 pass, SN-02 fail | Wiring, validation, lookup hay một rule khác | Xem error code và input thực gửi; lần theo use case báo giá |
| SN-04 fail với `'   '` | Quy ước null/rỗng hay validation từ chối | Đối chiếu R-VOUCHER-02 trước khi động vào expected |
| Total đúng, discount sai | Sai thứ tự tính bị làm tròn che | So từng bước với oracle 34.500 → 6.900 → 27.600 → 28.000 |
| SN-05 trả 200 | Mã sai bị coi như không dùng mã | Kiểm nhánh xử lý khi tra voucher không thấy |
| SN-06 không bị giới hạn | Chuyến lưu mã chưa chuẩn hoặc không đếm lượt | Kiểm `voucherCode` trên chuyến, phạm vi khách/ngày, rồi repository |
| Chạy riêng pass, chạy chung fail | Rò clock, seed hoặc state giữa ca | Kiểm `beforeEach` và việc tham chiếu instance cũ |

Defect report cần: build/commit, ID test case, input nguyên văn **với khoảng trắng hiển thị rõ** (ví dụ ghi `␠giam20␠` hoặc kèm độ dài chuỗi), clock, seed, expected/actual và bước đầu tiên lệch. Ảnh chụp dòng «test failed» không giúp ai tái hiện.

Ví dụ kết luận đúng phạm vi khi xanh: «Sanity cho thay đổi chuẩn hoá voucher đạt 8/8 lượt trong lab bộ nhớ; tiếp tục functional/regression cho voucher. Chưa kiểm PostgreSQL, concurrency lượt dùng, Unicode.» Khi SN-02 đỏ nhưng SN-01 xanh: «Không đóng lỗi — hàm đúng nhưng hành vi người dùng qua API chưa đáp ứng R-VOUCHER-02.»

## 7. Mở rộng nâng cao: mở phạm vi theo rủi ro thật

### 7.1. Thay đổi nhỏ ở hàm dùng chung

Đặt xe, bưu kiện và đồ ăn đều nhận voucher. Với đồ ăn, tiền giảm tính trên **tiền món** (R-FOOD-05), không phải trên cước. Nếu chuẩn hoá đổi cách hiểu mã cho mọi dịch vụ, hãy lập caller map từ code và hợp đồng API, rồi thêm ít nhất một đường đại diện cho mỗi **cấu trúc xử lý khác nhau** — không nhân bản mọi ca cho mọi dịch vụ. Tiêu chí chọn là khác biệt về cách dữ liệu được dùng, không phải khác biệt về tên endpoint.

### 7.2. Unicode và canonicalization

`trim` và `toUpperCase` không giải quyết ký tự vô hình (zero-width space), khoảng trắng bên trong hay các biểu diễn Unicode khác nhau của cùng một chữ. Đừng tự quyết «xoá mọi khoảng trắng»: `GIAM 20` có thể cần báo không tồn tại thay vì biến thành GIAM20 — đó là quyết định của đặc tả, không của tester. Nếu mã chỉ cho phép ASCII, cần hợp đồng validation rõ và ca cho ký tự ngoài tập. Nếu nhận Unicode, phải quyết định chuẩn lưu và đối chiếu trước khi viết được expected.

### 7.3. Giới hạn lượt và concurrency

SN-06 chạy tuần tự: lượt sau nhìn thấy lượt trước trong bộ nhớ. Hai request tạo chuyến đồng thời có thể cùng đọc «chưa dùng lượt nào» rồi cùng thành công. Kiểm yêu cầu «không vượt một lượt» cần DB có transaction thật, khởi chạy hai request qua barrier và một invariant như «số chuyến hợp lệ dùng GIAM20 của C001 trong ngày ≤ 1» kiểm bằng SQL. Chạy `Promise.all` trên kho bộ nhớ không chứng minh production an toàn. Đây là phần giao giữa sanity theo impact và data integrity (SYS-13).

### 7.4. Tự động chọn test và rủi ro bỏ sót

Có thể gắn mapping `normalizeVoucherCode → SN-01…SN-06` để CI chạy nhanh theo diff. Nhưng mapping theo file không thấy ảnh hưởng đi qua cấu hình, seed, dependency hay schema. Cần một suite nền luôn chạy, lịch regression rộng định kỳ, và theo dõi lỗi lọt qua để sửa mapping. Nếu công cụ báo «không có test nào bị ảnh hưởng», hãy coi đó là tín hiệu cần điều tra — không phải pass.

### 7.5. Làm mạnh bằng property testing (đề xuất, chưa có trong lab)

Một property hữu ích là tính idempotent: chuẩn hoá hai lần bằng chuẩn hoá một lần, N(N(x)) = N(x), với xử lý `null` đúng kiểu. Nhưng một hàm luôn trả `null` cũng thoả property đó. Vì vậy property chỉ **bổ trợ** các ví dụ GIAM20, chuỗi rỗng và mã sai, không thay oracle nghiệp vụ. Khi sinh chuỗi ngẫu nhiên, giới hạn miền dữ liệu theo hợp đồng API và ghi seed của generator để tái hiện khi fail. Phần này chưa được thực thi trong suite kèm bài.

## 8. Anti-pattern, giới hạn và đánh đổi

- **Chỉ chạy lại test case lỗi gốc rồi gọi là sanity.** Đó là confirmation; vùng lân cận (SN-03…SN-06) chưa được đánh giá.
- **Test helper tự sửa input.** `trim()` trong helper che lỗi server; test nói dối.
- **Chạy toàn bộ test rồi gọi là sanity.** Mất ý nghĩa lựa chọn phạm vi, mất tốc độ phản hồi, và không còn ai biết mỗi test case chạy vì lý do gì.
- **Gán sanity là «luôn manual» hoặc «luôn automated».** Hình thức tuỳ thay đổi; ở đây API test hợp lý vì thay đổi nằm sau API.
- **Suy rủi ro từ độ dài diff.** Một dòng trong code dùng chung có thể rộng hơn trăm dòng trong một màn hình riêng.
- **Sửa expected khi đỏ.** Expected đến từ SPEC; đổi expected cần đổi yêu cầu trước, có người chịu trách nhiệm.

Đánh đổi: sáu ca chạy dưới một giây cho phản hồi nhanh, đổi lại phải chấp nhận và **ghi ra** những vùng chưa phủ. Thêm ca có trạng thái như SN-06 tốn chút chi phí dựng dữ liệu nhưng bắt được loại lỗi mà kiểm cấp hàm không thể thấy. Mở rộng sang PostgreSQL và concurrency thì đắt hơn nhiều và thuộc suite riêng — quyết định dựa vào việc thay đổi có thực sự chạm tới chúng không.

## 9. Bài tập và câu hỏi ôn tập

### Bài tập

1. **Cơ bản.** Thêm input `gIaM20` vào sanity. Expected là gì, và ca này phủ thêm điều gì so với ` giam20 `?
2. **Trung bình.** Thêm input `GIAM 20` (khoảng trắng ở giữa). Expected theo SPEC hiện tại là gì? Có nên sửa helper để xoá khoảng trắng bên trong không?
3. **Trung bình.** Giả sử thay đổi tiếp theo là đổi GIAM20 từ 20% thành 25%. Tám lượt hiện tại có còn là sanity phù hợp không?
4. **Nâng cao.** Thiết kế ca kiểm hai request tạo chuyến đồng thời tranh một lượt GIAM20.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><b>1.</b> Expected giống SN-02/SN-03: 200, <code>voucherCode</code> GIAM20, discount 6.900, total 28.000. Ca phủ quy tắc <b>viết hoa</b> của R-VOUCHER-02 với chữ hoa/thường lẫn lộn, không chỉ quy tắc trim.</p>
<p><b>2.</b> R-VOUCHER-02 chỉ bỏ khoảng trắng <b>hai đầu</b>; <code>GIAM 20</code> không có trong seed nên expected là 422 <code>VOUCHER_NOT_FOUND</code>. Không sửa helper: muốn chấp nhận khoảng trắng bên trong là thay đổi đặc tả, cần quyết định nghiệp vụ trước.</p>
<p><b>3.</b> Không dùng nguyên được. Oracle đổi: 25% × 34.500 = 8.625 ⇒ 25.875 ⇒ làm tròn 26.000. Phạm vi cũng chuyển từ normalization sang pricing: cần thêm ca trần giảm, cước tối thiểu và ảnh chụp giá (chuyến đặt từ báo giá cũ giữ giá cũ theo R-QUOTE-04). Làm rõ thay đổi seed/requirement trước khi viết expected.</p>
<p><b>4.</b> Cần DB có transaction thật (PostgreSQL riêng, không phải kho bộ nhớ), hai báo giá hợp lệ của C001, barrier để hai request <code>POST /api/trips</code> khởi chạy cùng lúc với hai Idempotency-Key khác nhau, expected đúng một 201 và một 422 <code>VOUCHER_USAGE_LIMIT</code>, kèm SQL kiểm invariant số chuyến chưa huỷ dùng GIAM20 trong ngày ≤ 1, và cleanup theo run ID. Lặp nhiều lần để giảm khả năng pass do may mắn về thứ tự.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** Ca nào dưới đây là confirmation thuần, không phải regression gần?

- a) SN-02 — ` giam20 ` qua API
- b) SN-03 — `GIAM20` qua API
- c) SN-05 — ` khongco ` qua API
- d) SN-06 — đặt chuyến rồi báo giá lại

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a) đúng</b>: SN-02 tái hiện đúng điều kiện gây lỗi (mã có khoảng trắng hai đầu) ở cấp người dùng thấy. b), c), d) kiểm hành vi vốn đã đúng hoặc nằm sau thay đổi — vai trò regression gần và hợp đồng downstream.</p>
</details>

**Câu 2.** Unit test của <code>normalizeVoucherCode</code> pass nhưng SN-02 fail. Có được đóng phiếu lỗi không?

- a) Có, vì hàm đã đúng
- b) Không, vì hành vi qua API — điều khách gặp — chưa đáp ứng R-VOUCHER-02
- c) Có, nếu SN-03 pass
- d) Không, vì phải chạy toàn bộ regression trước

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b>. Lỗi được báo ở hành vi người dùng; hàm đúng mà không được gọi đúng thì lỗi vẫn còn. d) sai lý do: chưa cần regression toàn bộ để biết không được đóng — SN-02 đã đủ.</p>
</details>

**Câu 3.** Vì sao test kiểm cả <code>discount</code> lẫn <code>total</code>?

- a) Để test dài hơn
- b) Vì làm tròn lên bội 1.000 có thể làm hai cách tính sai/đúng ra cùng tổng
- c) Vì SPEC cấm trả tổng
- d) Vì <code>total</code> không ổn định giữa các lần chạy

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b>. Giảm 20% trên 35.000 (sai thứ tự) cho 7.000 và tổng 28.000 — trùng với cách đúng (6.900 → 27.600 → 28.000). Chỉ <code>discount</code> lộ ra lỗi.</p>
</details>

**Câu 4.** Tám lượt sanity xanh cho phép kết luận gì?

- a) Bản sửa đủ điều kiện release production
- b) Không còn lỗi voucher nào
- c) Thay đổi hoạt động hợp lý trong phạm vi đã chọn; được tiếp tục kiểm thử chuyên sâu
- d) Giới hạn một lượt mỗi ngày an toàn khi có request đồng thời

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng</b>. a), b) vượt phạm vi bằng chứng. d) không được kiểm: SN-06 chạy tuần tự trên kho bộ nhớ.</p>
</details>

## 10. Tài liệu tham khảo

Nguồn ngoài tra cứu 13/11/2026.

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — mục 2.2.3 Confirmation Testing and Regression Testing, impact analysis. PDF: https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf. Thuật ngữ «sanity suite» và cách ghép confirmation + regression gần trong bài là thực tiễn của series, không phải định nghĩa của syllabus.
- Vitest — Expect API (`toBe`, `toMatchObject`, `it.each`): https://vitest.dev/api/expect.html
- Phòng lab Tới Nơi: `SPEC.md` (R-VOUCHER-02/03/06/07, R-QUOTE-04, R-FARE-01…08), `src/domain/voucher.ts`, bộ test của bài — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-sanity-test
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [SYS-01 · Smoke Test](/articles/system-testing-smoke-test) · [Confirmation Testing và Regression Testing](/articles/confirmation-testing-va-regression-testing) · [Equivalence Partitioning](/articles/equivalence-partitioning) · [Phân tích tác động của change request](/articles/ba-change-request-impact-analysis)

Bài tiếp theo: [SYS-03 · Functional Test](/articles/system-testing-functional-test).
