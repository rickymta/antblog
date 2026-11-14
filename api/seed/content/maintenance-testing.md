---
slug: maintenance-testing
title: "Maintenance Testing: kiểm thử khi Tới Nơi tăng giá, ngừng một thành phố và đổi kho dữ liệu"
summary: "Maintenance testing theo CTFL v4.0.1: ba nhóm trigger (sửa đổi, nâng cấp/di trú, ngừng dịch vụ), impact analysis để quyết định phạm vi, minh hoạ bằng code chạy thật trên lab Tới Nơi."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-020, maintenance-testing, impact-analysis, regression-testing, data-archiving, toi-noi]
language: vi
---

Tới Nơi đang chạy hai triệu chuyến mỗi ngày. Bộ phận kinh doanh muốn tăng giá mỗi km xe máy, ban điều hành quyết định ngừng phục vụ Nha Trang, đội hạ tầng chuẩn bị chuyển kho dữ liệu sang nền tảng mới. Ba thay đổi, trên một hệ thống có khách thật, tiền thật và lịch sử thật. Bài này trả lời: **maintenance testing** khác kiểm thử lúc phát triển ở đâu, mỗi loại thay đổi cần kiểm những gì, và làm sao quyết định phạm vi kiểm thử bằng **impact analysis** — kể cả trước khi thay đổi được duyệt.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-020</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S02 · Test Levels &amp; Test Types</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1 (đọc thêm CTAL-TA v4.0)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>System; Acceptance</td></tr>
<tr><th scope="row">Test Type</th><td>Confirmation &amp; Regression</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/confirmation-testing-va-regression-testing">TEST-019 · Confirmation Testing và Regression Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (SPEC + code): phân tích tác động khi tăng giá km xe máy (R-FARE-01) và ngừng phục vụ Nha Trang (R-GEN-03) — trigger nào, test nào chạy lại, voucher nào bị ảnh hưởng; kèm kiểm thử lưu trữ dữ liệu và di trú môi trường — <code>examples/toi-noi-testing/articles/maintenance-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 2 · mục 2.3 · FL-2.3.1 (K2); liên quan mục 2.2.3 (FL-2.2.3) · Đọc thêm CTAL-TA v4.0 · mục 2.2 · TA-2.2.1 (K4)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Kiểm thử lúc phát triển làm việc với hệ thống **chưa có ai dùng**: sai thì sửa, dữ liệu xoá đi làm lại. Khi Tới Nơi đã vận hành thì khác:

- Có **dữ liệu thật** phải giữ đúng: báo giá còn hạn, chuyến đang chạy, lịch sử để đối soát và giải quyết khiếu nại.
- Có **người dùng thật** nhận hậu quả ngay: khoảng 70 chuyến tạo mới mỗi giây lúc cao điểm (hồ sơ dự án).
- Phần **không đổi** chiếm gần hết hệ thống, nhưng vẫn có thể bị thay đổi làm hỏng.

**Maintenance testing** (kiểm thử bảo trì) là kiểm thử các thay đổi trên một hệ thống **đang vận hành**. Nó không phải một kỹ thuật mới: nó dùng lại confirmation testing và regression testing ([TEST-019](/articles/confirmation-testing-va-regression-testing)), cộng thêm những việc chỉ xuất hiện khi hệ thống đã sống: di trú dữ liệu, lưu trữ dữ liệu khi ngừng dịch vụ, chuyển đổi tại một mốc giờ cụ thể.

Dùng bài này khi bạn nhận một phiếu thay đổi cho hệ thống đã lên production — đổi cấu hình giá, hotfix, nâng cấp nền tảng, hay tắt một tính năng.

## 2. Lý thuyết theo ISTQB

CTFL v4.0.1 mục 2.3 nêu các ý chính sau (diễn đạt lại):

- **Loại bảo trì**: sửa lỗi (corrective), thích nghi với thay đổi của môi trường (adaptive), cải thiện hiệu năng hoặc khả năng bảo trì; chi tiết theo ISO/IEC 14764. Bảo trì có thể đi theo **bản phát hành có kế hoạch** hoặc **không có kế hoạch** (hotfix).
- **Impact analysis có thể làm trước khi thay đổi**, để giúp quyết định có nên thay đổi hay không, dựa trên hậu quả có thể xảy ra ở phần khác của hệ thống. Đây là điểm khác với TEST-019, nơi impact analysis dùng để chọn test sau khi đã có bản sửa.
- Kiểm thử thay đổi trên hệ thống vận hành gồm **hai phần**: đánh giá việc triển khai thay đổi có thành công không, và kiểm hồi quy ở phần **không đổi** — thường là phần lớn hệ thống.
- **Phạm vi** maintenance testing thường phụ thuộc ba yếu tố: mức rủi ro của thay đổi, kích thước hệ thống hiện có, kích thước thay đổi.

**Ba nhóm trigger** (sự kiện kích hoạt) của bảo trì và maintenance testing:

| Trigger | Syllabus mô tả | Kiểm thử đặc thù |
|---|---|---|
| **Modification** (sửa đổi) | Cải tiến có kế hoạch theo bản phát hành, sửa lỗi, hotfix | Kiểm thay đổi đạt + regression |
| **Upgrade / migration** của môi trường vận hành | Chuyển nền tảng; di trú dữ liệu từ ứng dụng khác vào | Test cho môi trường mới và phần mềm đã đổi; **test chuyển đổi dữ liệu** |
| **Retirement** (ngừng sử dụng) | Ứng dụng hết vòng đời | **Test lưu trữ dữ liệu** khi phải giữ lâu; **test khôi phục và truy xuất** sau lưu trữ |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Ba cột trigger. Modification: tăng giá km xe máy, hotfix; kiểm thay đổi đạt, impact analysis 15 ca, regression phần không đổi. Upgrade hoặc migration: đổi kho dữ liệu, nền tảng; test môi trường mới, cùng bộ regression trên hai môi trường, test chuyển đổi dữ liệu. Retirement: ngừng phục vụ Nha Trang; lưu trữ dữ liệu, khôi phục và truy xuất, xử lý chuyến đang chạy. Dải dưới: phạm vi phụ thuộc rủi ro, kích thước hệ thống, kích thước thay đổi.">
<text x="460" y="22" text-anchor="middle" class="t-b">Ba nhóm trigger của maintenance testing (CTFL v4.0.1 mục 2.3) và việc cần kiểm ở Tới Nơi</text>
<rect x="20" y="40" width="280" height="62" rx="6" class="f-pk-soft s-pk"/>
<text x="160" y="62" text-anchor="middle" class="t-b t-pk">Modification</text>
<text x="160" y="82" text-anchor="middle" class="t-small">CR-01 tăng giá km xe máy · hotfix</text>
<rect x="20" y="110" width="280" height="122" rx="6" class="f-surface s-pk"/>
<text x="36" y="134" class="t-10">• kiểm thay đổi đạt: giá mới đúng</text>
<text x="36" y="156" class="t-10">• impact analysis 15 ca: 9 đổi, 6 giữ</text>
<text x="36" y="178" class="t-10">• regression phần không đổi</text>
<text x="36" y="214" class="t-mono t-pk">impact.test.ts</text>
<rect x="320" y="40" width="280" height="62" rx="6" class="f-phys-soft s-phys"/>
<text x="460" y="62" text-anchor="middle" class="t-b t-phys">Upgrade / migration</text>
<text x="460" y="82" text-anchor="middle" class="t-small">CR-03 đổi kho dữ liệu, nền tảng</text>
<rect x="320" y="110" width="280" height="122" rx="6" class="f-surface s-phys"/>
<text x="336" y="134" class="t-10">• test cho môi trường mới</text>
<text x="336" y="156" class="t-10">• cùng bộ regression trên hai môi trường</text>
<text x="336" y="178" class="t-10">• test chuyển đổi dữ liệu (nếu nhập dữ liệu)</text>
<text x="336" y="214" class="t-mono t-phys">migration-env.test.ts</text>
<rect x="620" y="40" width="280" height="62" rx="6" class="f-warn-soft s-warn"/>
<text x="760" y="62" text-anchor="middle" class="t-b t-warn">Retirement</text>
<text x="760" y="82" text-anchor="middle" class="t-small">CR-02 ngừng phục vụ Nha Trang</text>
<rect x="620" y="110" width="280" height="122" rx="6" class="f-surface s-warn"/>
<text x="636" y="134" class="t-10">• lưu trữ dữ liệu cần giữ lâu</text>
<text x="636" y="156" class="t-10">• khôi phục và truy xuất sau lưu trữ</text>
<text x="636" y="178" class="t-10">• chuyến đang chạy lúc cắt dịch vụ</text>
<text x="636" y="214" class="t-mono t-warn">retirement.test.ts</text>
<rect x="20" y="244" width="880" height="48" rx="6" class="f-bg s-muted dash"/>
<text x="460" y="264" text-anchor="middle" class="t-small">Phạm vi phụ thuộc: mức rủi ro của thay đổi · kích thước hệ thống hiện có · kích thước thay đổi</text>
<text x="460" y="282" text-anchor="middle" class="t-small">Phần không đổi — thường là phần lớn hệ thống — vẫn cần regression testing</text>
</svg>
</div>
<figcaption><b>Hình 1 · Ba nhóm trigger, ba kiểu việc.</b> Cả ba đều cần confirmation và regression; mỗi nhóm thêm một loại kiểm thử riêng. Tên tệp ở cuối mỗi cột là mã chạy được trong lab.</figcaption>
</figure>

### Thuật ngữ Anh – Việt

| Thuật ngữ | Tiếng Việt | Ghi chú |
|---|---|---|
| maintenance testing | kiểm thử bảo trì | kiểm thử thay đổi trên hệ thống đang vận hành |
| operational system | hệ thống đang vận hành | đã có người dùng, dữ liệu thật |
| trigger | sự kiện kích hoạt | modification, upgrade/migration, retirement |
| hot fix | bản sửa nóng | phát hành không theo kế hoạch |
| impact analysis | phân tích tác động | trước thay đổi: có nên làm; sau thay đổi: test gì |
| data conversion / migration | chuyển đổi / di trú dữ liệu | dữ liệu từ ứng dụng khác vào hệ thống |
| data archiving | lưu trữ dữ liệu | khi ngừng hệ thống mà phải giữ dữ liệu lâu |
| restore and retrieval | khôi phục và truy xuất | lấy lại dữ liệu đã lưu trữ khi cần |

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Ngoài syllabus CTFL — thực hành ngành</span>
<p>Không thuộc CTFL nhưng giúp bảo trì an toàn hơn: phiên bản hoá quy tắc giá (<code>pricing_version</code>, kiến trúc §4.3); diễn tập trên <code>staging</code> (§16.2); phát hành dần và cờ tính năng theo từng thành phố (pipeline §17); đổi cấu hình giá qua Admin Portal có maker-checker, audit log (§11.2). Chọn test hồi quy khi phạm vi lớn: CTAL-TA v4.0 mục 2.2 (TA-2.2.1, K4).</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### Ba phiếu thay đổi (giả định)

| Mã | Thay đổi | Quy tắc SPEC bị chạm |
|---|---|---|
| CR-01 | Giá mỗi km xe máy 4.500 đ ⇒ **5.000 đ** (con số giả định), phiên bản giá `fare-2026.2` | R-FARE-01, R-QUOTE-02 |
| CR-02 | Ngừng phục vụ Nha Trang (`NT`) từ một mốc giờ G; giữ lịch sử để đối soát, khiếu nại | R-GEN-03 |
| CR-03 | Nâng cấp kho dữ liệu chuyến sang nền tảng PostgreSQL mới (kiến trúc §16.4: HA, backup PITR, **kiểm thử restore**) | Mọi quy tắc có lưu trạng thái |

### Business rules liên quan

- **R-FOOD-04** — phí giao đồ ăn dùng **công thức xe máy**. CR-01 không nhắc tới đồ ăn, nhưng phí giao sẽ đổi theo.
- **R-PARCEL-04** — phụ phí bưu kiện nằm trong cước cơ bản; bưu kiện xe máy đổi theo.
- **R-VOUCHER-06** — tiền giảm tính trên số tiền đã gồm cước; ngưỡng tối thiểu (`minAmount`) và trần (`maxDiscount`) so với con số này.
- **R-QUOTE-04** — chuyến giữ ảnh chụp giá của báo giá; báo giá còn hạn (5 phút, R-QUOTE-01) trước mốc G phải giữ giá cũ.
- **R-STATE** — chuyến ở Nha Trang có thể đang `searching` hay `in_transit` đúng lúc G.

### System context

Ở kiến trúc đích, CR-01 là thay đổi cấu hình của **Pricing &amp; Quote Service** (§4.3), làm qua Admin Portal (§11.2); **Promotion Service** (§4.10) chịu ảnh hưởng gián tiếp qua voucher. CR-02 chạm Service Catalog (danh mục thành phố), Driver Availability (tài xế Nha Trang), Booking (chuyến đang chạy) và chính sách lưu giữ dữ liệu (§13.2 *data retention policy*). CR-03 là thay đổi hạ tầng (§16.4). Lab là modular monolith: CR-01, CR-02 được mô phỏng bằng một bản thay đổi đặt trong thư mục bài; CR-03 mô phỏng bằng cách chạy cùng bộ test trên kho bộ nhớ và trên PGlite (PostgreSQL chạy WASM, **một kết nối**).

### Rủi ro có thể xảy ra

| # | Rủi ro | Từ phiếu |
|---|---|---|
| 1 | Báo giá tạo trước G bị tính theo giá mới khi đặt chuyến | CR-01 |
| 2 | Phí giao đồ ăn tăng mà không ai thông báo cho khách và quán | CR-01 |
| 3 | Voucher bắt đầu áp được cho chuyến trước đây không đủ điều kiện, ngân sách khuyến mãi vượt dự kiến | CR-01 |
| 4 | Chuyến Nha Trang đang chạy lúc G bị kẹt, không hoàn thành hoặc không thu tiền được | CR-02 |
| 5 | Lịch sử chuyến Nha Trang bị xoá theo, không còn bằng chứng cho khiếu nại hay kế toán | CR-02 |
| 6 | Nền tảng mới xử lý khoá dòng, kiểu số hay múi giờ khác ⇒ hai tài xế cùng nhận, lệch tiền | CR-03 |

## 4. Phân tích kỹ thuật

### 4.1 Phân loại trigger và phạm vi

| Phiếu | Trigger (mục 2.3) | Kế hoạch? | Rủi ro | Kích thước thay đổi | Phạm vi đề xuất |
|---|---|---|---|---|---|
| CR-01 | Modification (cải tiến) | Có, theo bản phát hành | Cao — chạm tiền mọi chuyến xe máy, đồ ăn, bưu kiện xe máy | Rất nhỏ — một con số | Regression nhắm đích toàn bộ định giá + khuyến mãi, smoke test sau triển khai |
| CR-02 | Retirement (một phần dịch vụ) + Modification (danh mục thành phố) | Có | Trung bình — ít khách, nhưng dữ liệu phải giữ | Nhỏ về code, lớn về vận hành | Lưu trữ, khôi phục, truy xuất; chuyến đang chạy; regression danh mục thành phố |
| CR-03 | Upgrade / migration môi trường | Có | Cao — mọi chức năng có lưu trạng thái | Lớn về hạ tầng, không đổi code nghiệp vụ | Toàn bộ regression trên môi trường mới, test restore |

Retirement trong syllabus nói về **cả ứng dụng** hết vòng đời. Ngừng một thành phố là ngừng **một phần dịch vụ**, nhưng câu hỏi kiểm thử giống hệt: dữ liệu nào phải giữ, giữ ở đâu, lấy lại thế nào.

### 4.2 Impact analysis trước khi duyệt CR-01

Syllabus nói impact analysis có thể làm **trước** để quyết định có nên thay đổi không. Với CR-01, cách làm là lấy các test đang có trong bộ regression, chạy trên cả bản hiện hành và bản đề xuất, rồi xem **kết quả mong đợi nào đổi**. Người phân tích dự đoán trước, test xác nhận dự đoán:

| Ca | Quy tắc | Tình huống | Hiện hành | `fare-2026.2` | Đổi? |
|---|---|---|---|---|---|
| M-01 | R-FARE-01 | Xe máy 7,4 km, 09:00 | 46.000 | 49.000 | Đổi |
| M-02 | R-FARE-09 | Oracle 7,4 km, cao điểm, mưa, `GIAM20` | 52.000 | 56.000 | Đổi |
| M-03 | R-FARE-01 | Ô tô 7,4 km | 107.000 | 107.000 | Giữ |
| M-04 | R-FARE-07 | Xe máy 0,5 km (cước tối thiểu) | 15.000 | 15.000 | Giữ |
| M-05 | R-FARE-08 | Xe máy 1,0 km | 17.000 | 17.000 | Giữ |
| M-06 | R-PARCEL-03 | Bưu kiện xe máy 3 km, 6,1 kg | 30.000 | 31.000 | Đổi |
| M-07 | R-PARCEL-02 | Bưu kiện ô tô 3 km, 21 kg | 90.000 | 90.000 | Giữ |
| M-08 | R-FOOD-04 | Đồ ăn 2 km, món 50.000 đ (tổng) | 71.000 | 72.000 | Đổi |
| M-09 | R-FOOD-05 | Đồ ăn 2 km, món 100.000 đ, `FOOD15` (giảm / tổng) | 15.000 / 106.000 | 15.000 / 107.000 | Đổi (tổng) |
| M-10 | R-VOUCHER-06 | `HN10`, xe máy 6 km | `VOUCHER_MIN_AMOUNT` | 38.000 | Đổi |
| M-11 | R-VOUCHER-06 | `GIAM20`, xe máy 30 km | 118.000 | 132.000 | Đổi |
| M-12 | R-GEN-03 | Báo giá ở `NT` | hợp lệ | `CITY_NOT_SUPPORTED` | Đổi |
| M-13 | R-GEN-03 | Báo giá ở `DN` | hợp lệ | hợp lệ | Giữ |
| M-14 | R-CANCEL-03 | Phí huỷ xe máy ở `arriving` | 10.000 | 10.000 | Giữ |
| M-15 | R-QUOTE-02 | `pricingVersion` lưu vào báo giá | `fare-2026.1` | `fare-2026.2` | Đổi |

Đơn vị đồng. Năm phát hiện đáng mang tới cuộc họp duyệt thay đổi:

1. **Phí giao đồ ăn tăng** (M-08, M-09): 22.000 thay vì 21.000 cho 2 km. CR-01 không nhắc tới đồ ăn — đúng kiểu «hậu quả ở phần khác» syllabus nói. Kinh doanh phải chọn: chấp nhận, hay tách công thức phí giao khỏi bảng giá xe máy.
2. **`HN10` đổi hành vi** (M-10): xe máy 6 km ở Hà Nội trước bị từ chối voucher (A = 39.000 &lt; 40.000), nay được giảm (A = 42.000 ⇒ giảm 4.200 ⇒ 38.000 đ). Nhiều chuyến đủ điều kiện hơn ⇒ chi phí khuyến mãi tăng.
3. **`GIAM20` chạm trần sớm hơn** (M-11): 20% × 162.000 = 32.400 &gt; trần 30.000.
4. **Ca không nhạy** (M-04, M-05, mọi ca ô tô): cước tối thiểu và làm tròn lên nghìn «nuốt» chênh lệch. Smoke test sau triển khai mà dùng chuyến xe máy 1 km sẽ xanh **dù giá mới chưa được áp** — phải dùng ca có kết quả đổi, như M-01.
5. **Không bị ảnh hưởng**: `XMAS50` (đã hết hạn), phần tiền giảm của `FOOD15` (tính trên tiền món).

### 4.3 Test condition cho CR-02 và ngoài lab

Từ rủi ro 4–5 và R-STATE, R-QUOTE:

| ID | Test condition | Chạy ở |
|---|---|---|
| TC-NT-01 | Sau G, báo giá ở `NT` bị từ chối `CITY_NOT_SUPPORTED` (M-12) | Lab |
| TC-NT-02 | Chuyến `NT` đã kết thúc được lưu trữ đủ, khớp tổng kiểm với nguồn | Lab (MT-R1, R3) |
| TC-NT-03 | Chuyến `NT` chưa kết thúc lúc chạy script không bị lưu trữ âm thầm mà được liệt kê để xử lý | Lab (MT-R2) |
| TC-NT-04 | Sau lưu trữ, tra lại được một chuyến bất kỳ với giá, phiên bản giá, nhật ký trạng thái | Lab (MT-R4, R5) |
| TC-NT-05 | Báo giá `NT` tạo lúc G − 3 phút (còn hạn tới G + 2) — đặt được hay không? | **Cần nghiệp vụ quyết**, chạy ở staging |
| TC-NT-06 | Voucher chỉ áp cho `NT` (`cityCodes = ['NT']`) được vô hiệu hoá, không còn hiện trong ứng dụng | Staging |
| TC-NT-07 | Tài xế đăng ký ở Nha Trang nhận thông báo, không còn nhận được chuyến | Staging |
| TC-NT-08 | Báo cáo doanh thu theo thành phố của kỳ cũ vẫn ra số Nha Trang từ kho lưu trữ | Staging / UAT |

TC-NT-05 cho thấy giá trị phụ của việc viết test condition sớm: lộ **khoảng trống trong yêu cầu**. SPEC không nói gì về báo giá còn hạn của thành phố vừa ngừng — phải hỏi trước giờ G, không phải sau.

## 5. Thực hành

Mã nằm ở `examples/toi-noi-testing/articles/maintenance-testing/`. Không sửa `src/` của lab: `src/` là bản **trước thay đổi**, thư mục bài chứa bản thay đổi và script.

### 5.1 Bản thay đổi và bảng impact chạy được

```ts
// pricing-2026-2.ts — mô phỏng commit sẽ sửa src/domain/fare.ts, types.ts, pricing.ts
export const PRICING_VERSION_2026_2 = 'fare-2026.2';
export const FARE_TABLE_2026_2 = { bike: { ...FARE_TABLE.bike, perKm: 5_000 }, car: FARE_TABLE.car } as const;
export const CITY_CODES_2026_2 = CITY_CODES.filter((code) => code !== 'NT');
// calculateFare20262: đúng sáu bước R-FARE-03, chỉ đổi bảng giá
```

```ts
// impact.test.ts — mỗi ca: kết quả cũ (SPEC v1.1), kết quả mới (tính tay), dự đoán đổi/giữ
{ id: 'M-10', basis: 'R-VOUCHER-06', title: 'HN10, xe máy 6 km (A so với tối thiểu 40.000 đ)',
  doi: true, cu: 'VOUCHER_MIN_AMOUNT', moi: 38_000,
  run: (v) => outcome(() => fare(v, { vehicle: 'bike', distanceKm: 6, ...off, voucher: HN10 })) },

it.each(CASES)('$id ($basis) $title — dự đoán đổi: $doi', (c) => {
  const cu = c.run('cu');
  const moi = c.run('moi');
  expect(cu, 'bản hiện hành khớp SPEC v1.1').toEqual(c.cu);
  expect(moi, 'bản thay đổi khớp quy tắc mới').toEqual(c.moi);
  expect(JSON.stringify(cu) !== JSON.stringify(moi), 'dự đoán đổi/giữ').toBe(c.doi);
});
```

Ba khẳng định trong mỗi ca tách ba câu hỏi: bộ regression hiện tại có đúng không, kết quả mong đợi mới có đúng không, và người phân tích có đoán đúng phạm vi ảnh hưởng không. Kết quả mong đợi mới **tính tay** từ quy tắc mới, không chép từ lần chạy bản thay đổi.

### 5.2 Retirement: lưu trữ, khôi phục, truy xuất

Script `retire-city.ts` đọc chuyến của một thành phố từ database vận hành, **chỉ lấy chuyến đã kết thúc**, đóng gói kèm nhật ký trạng thái và tổng kiểm (số chuyến, tổng cước), rồi nạp vào một kho lưu trữ có lược đồ khác:

```ts
export async function exportCityArchive(db: PGlite, cityCode: string): Promise<CityArchive> {
  const { rows } = await db.query<TripRow>(
    `SELECT id, kind, status, customer_id, fare, cancellation_fee, pricing_version, requested_at
       FROM trips WHERE city_code = $1 ORDER BY id`, [cityCode]);
  // chuyến chưa ở completed/cancelled ⇒ đưa vào `blocked`, không lưu trữ
  …
}
```

Dữ liệu test dựng bằng chính use case của lab trên PGlite, lúc 09:00 ngày 04/11/2026: chuyến NT-A hoàn thành (5 km ⇒ 12.000 + 22.500 = 34.500 ⇒ **35.000 đ**), NT-B khách huỷ khi đang tìm tài xế (3 km ⇒ **26.000 đ**, phí huỷ 0), NT-C còn đang tìm tài xế, SG-D ở thành phố khác. Bản lưu trữ được `JSON.stringify` rồi đọc lại để mô phỏng việc ghi ra tệp.

| ID | Kiểm | Mong đợi |
|---|---|---|
| MT-R1 | Nội dung bản lưu trữ | Đúng NT-A (`completed`), NT-B (`cancelled`); không có SG-D |
| MT-R2 | Chuyến đang chạy | NT-C nằm trong `blocked` |
| MT-R3 | Tổng kiểm so với nguồn (SQL trên database vận hành) | 2 chuyến, 61.000 đ |
| MT-R4 | Khôi phục vào kho lưu trữ | 2 dòng, 61.000 đ |
| MT-R5 | Truy xuất NT-A | `completed`, 35.000 đ, `fare-2026.1`, đủ 7 trạng thái từ `requested` tới `completed` |

### 5.3 Migration: cùng bộ regression trên hai môi trường

```ts
const ENVIRONMENTS = [
  { name: 'môi trường cũ — kho bộ nhớ', make: async () => createMemorySystem(options()) },
  { name: 'môi trường mới — PostgreSQL (PGlite)', make: () => createPgliteSystem(options()) },
];
describe.each(ENVIRONMENTS)('TEST-020 · regression sau di trú: $name', ({ make }) => { … });
```

Bốn ca chạy ở 11:30 (cao điểm), Sài Gòn đang mưa: MT-E1 R-FARE-09 + R-QUOTE-04 (chuyến lưu 52.000 đ, đọc lại vẫn 52.000 đ), MT-E2 R-QUOTE-03 (một báo giá một chuyến), MT-E3 R-VOUCHER-07 (hết lượt `GIAM20`, huỷ thì trả lượt), MT-E4 R-ACCEPT-03 (tài xế thứ hai nhận ⇒ `TRIP_ALREADY_TAKEN`).

### 5.4 Chạy thật

```bash
cd examples/toi-noi-testing
npx vitest run articles/maintenance-testing --reporter=verbose
```

Kết quả (rút gọn, chép từ lần chạy):

```text
 ✓ impact.test.ts > 'M-01' ('R-FARE-01') 'xe máy 7,4 km, 09:00, không mưa' — dự đoán đổi: true
 ✓ impact.test.ts > 'M-05' ('R-FARE-08') 'xe máy 1,0 km — làm tròn lên nghìn' — dự đoán đổi: false
 …  (M-01 … M-15)
 ✓ impact.test.ts > tổng kết: 9 ca phải cập nhật kết quả mong đợi, 6 ca là regression thuần
 ✓ migration-env.test.ts > … 'môi trường cũ — kho bộ nhớ' > MT-E1 … MT-E4
 ✓ migration-env.test.ts > … 'môi trường mới — PostgreSQL (PGlite)' > MT-E1 … MT-E4
 ✓ retirement.test.ts > … MT-R1 … MT-R5
      Tests  29 passed (29)
```

### 5.5 Kế hoạch maintenance test cho đợt phát hành

| Thời điểm | Việc | Bằng chứng |
|---|---|---|
| Trước khi duyệt | Impact analysis (5.1), mang 5 phát hiện ở 4.2 vào họp; chốt TC-NT-05 | Biên bản quyết định |
| Trên staging | Cập nhật kết quả mong đợi của 9 ca «đổi»; chạy toàn bộ regression; diễn tập lưu trữ và khôi phục với bản sao dữ liệu đã che PII (§16.2) | Báo cáo chạy test, tổng kiểm |
| Tại mốc G | Lưu trữ chuyến `NT` đã kết thúc; xử lý danh sách `blocked`; bật cấu hình giá mới | Log script, danh sách đã xử lý |
| Sau G | Smoke test production bằng ca **có kết quả đổi** (M-01); theo dõi chỉ số kinh doanh (§15.2) — doanh thu mỗi chuyến, tỷ lệ áp voucher | Kết quả smoke, biểu đồ theo dõi |

## 6. Góc nhìn nâng cao

**Maintenance testing là một bối cảnh, không phải một test type.** Nó gồm confirmation (thay đổi có đạt không) và regression (phần không đổi có còn đúng không) trên một hệ thống đang vận hành. Câu hỏi «bài này thuộc test type nào» với maintenance testing thường có câu trả lời: confirmation &amp; regression, ở system và acceptance level.

**Acceptance trong bảo trì.** Lưu trữ, khôi phục, runbook xử lý chuyến đang chạy là nội dung của operational acceptance testing ([TEST-016](/articles/acceptance-testing-tong-quan)) — người vận hành phải ký nhận thủ tục chạy được.

**Test không nhạy là test nguy hiểm nhất của bảo trì.** M-04, M-05 cho cùng kết quả trước và sau thay đổi. Trong bộ regression, chúng vẫn có ích (bảo vệ cước tối thiểu, làm tròn). Làm smoke test để chứng minh «giá mới đã lên», chúng vô dụng — và sẽ cho cảm giác an toàn giả.

**Hotfix nén phạm vi, không nén tư duy.** Hotfix phụ phí trời mưa trong mùa bão có thể chỉ có hai giờ. Khi đó ưu tiên: confirmation trên ca mưa, regression nhắm đích những đường đi qua phụ phí trời mưa (cước, bưu kiện, phí giao đồ ăn), smoke production; bộ đầy đủ chạy ngay sau phát hành. Ghi rõ những gì chưa chạy vào báo cáo.

**Retirement hay bị quên vì không có ai «dùng» nó.** Lưu trữ chỉ có giá trị nếu khôi phục được. Kiến trúc §16.4 yêu cầu kiểm thử restore định kỳ cho cùng lý do. Ngược lại, có loại dữ liệu phải **xoá** chứ không được giữ (dữ liệu vị trí, thông tin cá nhân theo chính sách lưu giữ §13.2) — test cả chiều đó.

**Khối lượng lưu trữ.** Giả định Nha Trang chiếm 5% của 2 triệu chuyến/ngày: 100.000 chuyến/ngày ≈ 36,5 triệu dòng mỗi năm, chưa kể `trip_events`. Ở quy mô đó script phải chạy theo lô và tiếp tục được khi gián đoạn — lab không kiểm được điều này.

**Chuyển đổi dữ liệu khác di trú nền tảng.** CR-03 chỉ đổi nơi chứa. Nếu nhập dữ liệu từ một ứng dụng khác (giả định: mua lại một dịch vụ giao hàng), cần test chuyển đổi: ánh xạ trạng thái cũ sang 8 trạng thái của Tới Nơi, đối soát số lượng và tổng tiền, xử lý bản ghi không ánh xạ được.

**Giới hạn của lab.** PGlite chạy một kết nối nên MT-E4 chỉ kiểm logic «một người thắng» khi các lệnh xen kẽ, không chứng minh khoá dòng dưới tải song song thật (README lab). Lab không có Kafka: ở hệ thống đích, ngừng một thành phố còn chạm tới sự kiện đang nằm trong hàng đợi và consumer của Notification, Payment.

**Testware cũng cần bảo trì.** 9 ca «đổi» là 9 kết quả mong đợi phải sửa; kết quả mong đợi chép tay rải rác nhiều nơi làm mỗi lần đổi giá tốn kém — chủ đề TEST-125 (Series 10).

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — mở thêm thành phố.** Tới Nơi mở thêm Huế (mã giả định `HU`). (a) Trigger thuộc nhóm nào? (b) Những quy tắc và dữ liệu nào trong SPEC/lab bị ảnh hưởng? (c) Một test nào hiện đang xanh có thể chuyển đỏ dù hệ thống đúng?

<details>
<summary><b>Lời giải</b></summary>
<p>(a) Modification — cải tiến có kế hoạch. (b) R-GEN-03 (danh sách thành phố, <code>CITY_NOT_SUPPORTED</code>); voucher có <code>cityCodes</code> rỗng (<code>GIAM20</code>, <code>FOOD15</code>) tự áp ở Huế — marketing phải xác nhận; bảng <code>cities</code> trong <code>schema.sql</code>; dịch vụ thời tiết (R-GEN-05); tài xế (R-ACCEPT-01 cùng thành phố). (c) Test âm nào dùng <code>HU</code> làm ví dụ «thành phố không hỗ trợ» sẽ đỏ — cần cập nhật test, không phải hồi quy. Ví dụ không hợp lệ nên là giá trị không bao giờ hợp lệ (như <code>XX</code>).</p>
</details>

**Bài tập 2 — hotfix hai giờ.** Mùa bão, phụ phí trời mưa R-FARE-05 tăng khẩn từ 8.000 lên 10.000 đ. (a) Tính lại oracle R-FARE-09. (b) Chọn bộ test tối thiểu trước khi phát hành và bộ chạy sau.

<details>
<summary><b>Lời giải</b></summary>
<p>(a) 45.300 × 1,25 = 56.625 ⇒ + 10.000 = 66.625 ⇒ − 20% (13.325, dưới trần 30.000) = 53.300 ⇒ làm tròn lên <b>54.000 đ</b> (trước đây 52.000 đ). (b) Trước phát hành: confirmation trên ca mưa (R-FARE-09 mới), một ca bưu kiện có mưa, một ca phí giao đồ ăn có mưa (R-FOOD-04 có áp mưa), một ca không mưa để chắc chắn không đổi; smoke production bằng một báo giá ở thành phố đang mưa. Sau phát hành: toàn bộ regression định giá và khuyến mãi (voucher có <code>minAmount</code> như <code>HN10</code> có thể đổi hành vi khi trời mưa), ghi vào báo cáo những gì chưa chạy trước giờ phát hành.</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K2).** Đội hạ tầng chuyển cơ sở dữ liệu chuyến của Tới Nơi sang một cụm PostgreSQL phiên bản mới, không đổi code nghiệp vụ. Theo CTFL v4.0.1, đây là trigger nào của maintenance testing?

- a) Modification
- b) Upgrade hoặc migration của môi trường vận hành
- c) Retirement
- d) Không phải maintenance testing vì code không đổi

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — chuyển môi trường vận hành từ nền tảng này sang nền tảng khác là nhóm upgrade/migration; cần test cho môi trường mới. a) modification là thay đổi phần mềm (cải tiến, sửa lỗi). c) không có gì bị ngừng. d) sai: thay đổi môi trường của hệ thống đang vận hành vẫn cần maintenance testing.</p>
</details>

**Câu 2 (K2).** Tới Nơi ngừng phục vụ một thành phố nhưng phải giữ lịch sử chuyến 5 năm để đối soát. Kiểm thử nào được syllabus nhắc riêng cho tình huống này?

- a) Kiểm thử hiệu năng của API báo giá
- b) Kiểm thử lưu trữ dữ liệu, và có thể cả thủ tục khôi phục, truy xuất sau lưu trữ
- c) Kiểm thử khả năng sử dụng của màn hình đặt xe
- d) Chỉ cần confirmation testing cho các defect đang mở

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — với retirement, syllabus nêu test lưu trữ dữ liệu khi cần giữ lâu, và test khôi phục, truy xuất khi dữ liệu có thể được cần trong thời gian lưu trữ. a), c) không gắn với việc ngừng dịch vụ. d) không có defect nào liên quan.</p>
</details>

**Câu 3 (K2).** Yếu tố nào KHÔNG thuộc nhóm yếu tố mà CTFL v4.0.1 nêu là thường quyết định phạm vi maintenance testing?

- a) Mức rủi ro của thay đổi
- b) Kích thước hệ thống hiện có
- c) Kích thước thay đổi
- d) Ngôn ngữ lập trình của hệ thống

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>d)</b> đúng (là đáp án cần chọn) — syllabus nêu ba yếu tố a, b, c. Ngôn ngữ lập trình có thể ảnh hưởng tới công cụ, nhưng không phải yếu tố quyết định phạm vi được nêu.</p>
</details>

**Câu 4 (K2).** Trước khi duyệt việc tăng giá km xe máy, tester chạy bộ regression trên bản đề xuất và báo: phí giao đồ ăn sẽ tăng, voucher <code>HN10</code> áp được cho nhiều chuyến hơn. Hoạt động này phục vụ mục đích nào được syllabus nêu?

- a) Debugging bản thay đổi
- b) Impact analysis trước thay đổi, giúp quyết định có nên thực hiện thay đổi
- c) Confirmation testing cho một defect
- d) Lưu trữ dữ liệu trước khi ngừng dịch vụ

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — syllabus nói impact analysis có thể làm trước khi thay đổi để giúp quyết định, dựa trên hậu quả ở phần khác của hệ thống. a) không có failure nào đang được tìm nguyên nhân. c) không có defect. d) không liên quan.</p>
</details>

**Câu 5 (K2).** Phát biểu nào về kiểm thử thay đổi trên hệ thống đang vận hành là ĐÚNG theo CTFL v4.0.1?

- a) Chỉ cần kiểm phần vừa thay đổi vì phần còn lại đã chạy ổn trên production
- b) Gồm đánh giá việc triển khai thay đổi có thành công và kiểm hồi quy ở phần không đổi, thường là phần lớn hệ thống
- c) Chỉ thực hiện ở component level
- d) Chỉ áp dụng cho bản phát hành có kế hoạch, không áp dụng cho hotfix

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — đúng hai phần syllabus nêu. a) bỏ regression. c) maintenance testing có thể ở mọi level; ở Tới Nơi chủ yếu system và acceptance. d) syllabus nói rõ bảo trì gồm cả bản không có kế hoạch (hotfix).</p>
</details>

**Câu 6 (K1).** Theo CTFL v4.0.1, «sửa nóng» (hot fix) một lỗi tính phí huỷ trên production thuộc nhóm trigger nào?

- a) Modification
- b) Upgrade hoặc migration
- c) Retirement
- d) Data conversion

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a)</b> đúng — syllabus liệt kê corrective changes và hot fixes trong nhóm modification. b) và c) là hai nhóm khác. d) chuyển đổi dữ liệu là một loại test trong nhóm migration, không phải trigger.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024), mục 2.3 «Maintenance Testing» (FL-2.3.1, K2); liên quan mục 2.2.3 «Confirmation Testing and Regression Testing» (FL-2.2.3, K2): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level — Test Analyst (CTAL-TA)* **v4.0** (2025), mục 2.2 «Risk Control», TA-2.2.1 (K4): https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/ — mục *maintenance testing*, *impact analysis*, *regression testing*.
- Trang chủ ISTQB: https://www.istqb.org/
- ISO/IEC 14764, *Software Engineering — Software Life Cycle Processes — Maintenance* — syllabus dẫn chiếu cho các loại bảo trì.
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-FARE-01/07/08/09, R-FOOD-04/05, R-PARCEL-02/03, R-VOUCHER-06, R-QUOTE-01…04, R-GEN-03; mã bài tại `articles/maintenance-testing/`.
- Tài liệu kiến trúc tổng thể Tới Nơi: §4.3 Pricing &amp; Quote, §4.10 Promotion, §11.2 Admin Portal, §13.2 Data protection, §15.2 Business metrics, §16.2 Môi trường, §16.4 DR, §17 CI/CD.
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-008 · Kiểm thử trong SDLC](/articles/kiem-thu-trong-sdlc-agile-waterfall) · [TEST-014 · System Testing](/articles/system-testing) · [TEST-016 · Acceptance Testing](/articles/acceptance-testing-tong-quan) · [TEST-019 · Confirmation và Regression Testing](/articles/confirmation-testing-va-regression-testing).

Bài tiếp theo: Series 03 mở đầu với [TEST-021 · Static Testing là gì?](/articles/static-testing-la-gi) — tìm defect trong đặc tả trước khi có code chạy được.
