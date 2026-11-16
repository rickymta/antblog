---
slug: test-case-design-phoi-hop-ky-thuat
title: "Phối hợp kỹ thuật thiết kế test case: EP, BVA và bảng quyết định cho kiểm tra yêu cầu báo giá Tới Nơi"
summary: "Chọn và phối hợp EP, BVA, bảng quyết định cho một tính năng: kiểm tra yêu cầu báo giá theo thứ tự lỗi R-VALID-01. Gộp ca dương, tách ca âm theo single fault, đo độ phủ riêng từng kỹ thuật, rút gọn tham lam 73 ca còn 18 ca giữ đủ 38/38 coverage item, và một ca âm gộp lỗi che mất hai defect. Lab 49 test vitest."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-case-design, tcd-06, equivalence-partitioning, boundary-value-analysis, decision-table-testing, defect-masking, test-suite-minimization, black-box, vitest, toi-noi]
language: vi
---

Bạn đã biết EP, BVA và bảng quyết định từ Series 04. Mỗi bài dạy một kỹ thuật trên một quy tắc được chọn cho vừa kỹ thuật đó. Công việc thật thì ngược lại: người ta giao cho bạn **một tính năng**, và tính năng đó cần cả ba kỹ thuật cùng lúc. Nếu áp riêng từng kỹ thuật rồi cộng kết quả, bộ test sẽ phình to và trùng lặp. Nếu gộp tuỳ tiện, các ca âm sẽ che lỗi của nhau. Bài này lấy tính năng kiểm tra yêu cầu báo giá của Tới Nơi, chọn kỹ thuật cho từng phần của nó, đo độ phủ **riêng theo từng kỹ thuật**, rồi rút gọn bộ ca mà không mất coverage item nào. Lab cũng cho thấy một ca âm gộp ba lỗi vẫn chạy xanh trên một bản cài đặt có hai defect.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TCD-06</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>TCD · Thiết kế Test Case</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1; bối cảnh Advanced — CTAL-TA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>System (qua API)</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Equivalence Partitioning, Boundary Value Analysis, Decision Table Testing — phối hợp</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/test-case-design-logic-va-cu-the">TCD-03</a>, <a href="/articles/equivalence-partitioning">TEST-031 · EP</a>, <a href="/articles/boundary-value-analysis">TEST-032 · BVA</a>, <a href="/articles/decision-table-testing">TEST-033 · Decision Table</a>, <a href="/articles/state-transition-testing-co-ban">TEST-034 · State Transition</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Sinh bộ ca từ mô hình EP (20 phân vùng), BVA (12 giá trị), bảng quyết định (6 cột) cho <code>POST /api/quotes</code> (R-VALID-01, R-GEN-03, R-FARE-02, R-PARCEL-01/02/03); đo độ phủ từng kỹ thuật; rút gọn tham lam; chạy qua supertest; ca âm gộp lỗi trên bản có defect — <code>examples/toi-noi-testing/articles/test-case-design-phoi-hop-ky-thuat/</code> (49 test vitest)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 4 · mục 4.1, 4.2.1, 4.2.2, 4.2.3 · FL-4.2.1 (K3), FL-4.2.2 (K3), FL-4.2.3 (K3). CTAL-TA v4.0: bối cảnh</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Hỏi một tester mới «kiểm tra yêu cầu báo giá cần bao nhiêu test case?», câu trả lời thường là một trong hai cực. Cực thứ nhất: «ba bốn ca, một ca đúng và vài ca sai». Cực thứ hai: chạy cả ba kỹ thuật vừa học, mỗi kỹ thuật một bộ, cộng lại 31 ca, rồi thêm vài ca «cho chắc». Cả hai đều không trả lời được câu hỏi quan trọng hơn: **bộ ca đó phủ được gì, và ca nào trong đó thừa?**

Bài này giải quyết bốn vấn đề của việc phối hợp kỹ thuật:

- **Chọn kỹ thuật theo cấu trúc của quy tắc**, không theo thói quen. Quy tắc «mã thành phố thuộc danh sách» khác quy tắc «quãng đường trong khoảng», và cả hai khác quy tắc «lỗi nào được báo trước».
- **Gộp ca dương, tách ca âm.** Một ca dương có thể mang nhiều phân vùng hợp lệ và nhiều giá trị biên hợp lệ cùng lúc. Một ca âm chỉ được mang **một** giá trị sai, vì R-VALID-01 chỉ báo lỗi đầu tiên.
- **Đo độ phủ theo từng kỹ thuật.** «100% coverage» không có nghĩa nếu không nói coverage của kỹ thuật nào. Bộ ca đạt 100% EP có thể chỉ đạt 0% BVA.
- **Loại ca thừa mà không mất độ phủ.** Khi coverage item của mọi kỹ thuật được viết thành dữ liệu, việc rút gọn bộ ca thành một bài toán tính được, kiểm được.

Khi nào dùng cách làm này: mọi tính năng có từ hai quy tắc trở lên tác động lên cùng đầu vào, nhất là lớp kiểm tra đầu vào của API, biểu mẫu, bộ quy tắc giá. Bài dừng ở mức kiểm thử hộp đen qua API; phần kiểm thử theo cấu trúc mã nằm ở Series 05.

## 2. Lý thuyết theo ISTQB

### 2.1. Kỹ thuật kiểm thử hỗ trợ cả test analysis lẫn test design — CTFL v4.0.1 mục 4.1

Mục 4.1 giới thiệu test technique như công cụ giúp tester trong **test analysis** (kiểm cái gì) và **test design** (kiểm thế nào), để xây dựng một bộ test case **tương đối nhỏ nhưng đủ** một cách có hệ thống. Kỹ thuật cũng giúp xác định test condition, coverage item và test data. Syllabus chia kỹ thuật thành ba nhóm: black-box, white-box và experience-based. Bài này chỉ dùng nhóm black-box.

Hai chữ cần giữ trong đầu là «nhỏ» và «đủ». Một kỹ thuật cho ta một định nghĩa về «đủ» (coverage item của nó) và một cách sinh ca. Phối hợp nhiều kỹ thuật nghĩa là có **nhiều định nghĩa về đủ** cùng lúc. Việc của test design là thoả mãn tất cả với số ca nhỏ nhất có thể. Syllabus không đưa ra thuật toán nào cho việc phối hợp. Phần còn lại của bài là một cách làm thực hành, có giải thích vì sao.

### 2.2. Coverage item của ba kỹ thuật — mục 4.2.1, 4.2.2, 4.2.3

| Kỹ thuật | Coverage item | Coverage | LO |
|---|---|---|---|
| Equivalence Partitioning (4.2.1) | phân vùng tương đương, gồm cả phân vùng không hợp lệ | số phân vùng đã chạm ÷ tổng số phân vùng | FL-4.2.1 (K3) |
| Boundary Value Analysis (4.2.2) | 2-value: mỗi giá trị biên và hàng xóm ở phân vùng kề; 3-value: giá trị biên và cả hai hàng xóm | số item đã thử ÷ tổng số item | FL-4.2.2 (K3) |
| Decision Table Testing (4.2.3) | cột chứa tổ hợp điều kiện khả thi | số cột đã chạy ÷ tổng số cột khả thi | FL-4.2.3 (K3) |

Ba định nghĩa này **không thay thế nhau**. Một ca chạm phân vùng D2 (quãng đường hợp lệ) bằng giá trị 7,4 km phủ một item EP nhưng không phủ item BVA nào. Một ca thử 0,45 km phủ cả phân vùng D2 lẫn giá trị biên 0,45. Đây chính là chỗ phối hợp tạo ra lợi ích: một ca có thể đồng thời là coverage item của nhiều kỹ thuật.

### 2.3. Ca âm kiểm riêng từng giá trị sai

Ghi chú phát hành của CTFL v4.0.1 nói bản này bổ sung vào mục 4.2.1: phân vùng không hợp lệ nên được kiểm **riêng** để tránh **defect masking** (che lỗi). Lý do: khi một ca có hai giá trị sai, cách hệ thống xử lý giá trị sai thứ nhất có thể khiến giá trị sai thứ hai không bao giờ được xử lý. [TEST-031](/articles/equivalence-partitioning) đã minh hoạ ý này bằng một ca. Bài này đi xa hơn: đếm chính xác ca gộp lỗi phủ được bao nhiêu coverage item, và cho ca đó chạy trên một bản cài đặt có lỗi.

Bài gọi nguyên tắc này là **single fault** (một lỗi mỗi ca âm). Tên gọi lấy từ giả định «single fault» trong sách của Jorgensen, không có trong CTFL. Ý nghĩa ở đây hẹp hơn: **mỗi ca âm chứa đúng một giá trị vi phạm, mọi tham số khác hợp lệ.**

### 2.4. Ca dương và ca âm

| | Ca dương (positive test case) | Ca âm (negative test case) |
|---|---|---|
| Đầu vào | mọi giá trị hợp lệ | đúng một giá trị không hợp lệ |
| Mong đợi | xử lý thành công, kết quả đúng (ở đây: 200, tổng tiền đúng) | từ chối đúng cách (status, mã lỗi, trường bị báo) |
| Gộp nhiều coverage item | **được** — mọi giá trị đều được xử lý | **không** — chỉ giá trị gây lỗi đầu tiên được xử lý |
| Rủi ro khi gộp | ca đỏ thì khó chẩn đoán item nào sai | che lỗi |

Ca dương được gộp nhưng không phải miễn phí: khi một ca dương mang 0,45 km **và** 0,05 kg bị đỏ, ta phải chẩn đoán thêm một bước. Đó là cái giá chấp nhận được, vì ca đỏ vẫn **báo** có lỗi. Ca âm gộp lỗi thì nguy hiểm hơn hẳn: nó **xanh** trong khi lỗi vẫn còn.

### 2.5. CTAL-TA v4.0 — bối cảnh

CTAL-TA v4.0 xếp EP, BVA vào nhóm kỹ thuật dựa trên dữ liệu và bảng quyết định vào nhóm kỹ thuật dựa trên quy tắc, đồng thời bàn cách Test Analyst chọn và kết hợp kỹ thuật theo rủi ro. Bài chỉ dùng tài liệu này làm bối cảnh, không gắn LO.

### 2.6. Thuật ngữ Anh – Việt

| Thuật ngữ | Dạng dùng trong bài | Ghi chú |
|---|---|---|
| positive / negative test case | ca dương / ca âm | |
| coverage item | coverage item | đơn vị đếm của độ phủ |
| defect masking | che lỗi | một lỗi làm lỗi khác không quan sát được |
| single fault | single fault | ngoài ISTQB; mỗi ca âm một giá trị sai |
| test suite minimization | rút gọn bộ test | ngoài ISTQB; loại ca thừa giữ nguyên độ phủ |
| greedy algorithm | thuật toán tham lam | mỗi bước chọn ca có lợi nhất |
| test oracle | test oracle | ở đây là hàm tính kết quả mong đợi từ SPEC |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Rút gọn bộ test bằng thuật toán tham lam là bài toán phủ tập hợp (set cover) — một bài toán NP-khó, nên thuật toán tham lam cho kết quả tốt nhưng không bảo đảm nhỏ nhất. Thuật ngữ «single fault assumption» lấy từ P. C. Jorgensen, <i>Software Testing: A Craftsman's Approach</i>. CTFL v4.0.1 không dùng hai khái niệm này.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Mọi chuyến xe và đơn bưu kiện của Tới Nơi bắt đầu bằng một yêu cầu báo giá. Lớp kiểm tra đầu vào của `POST /api/quotes` là cửa đầu tiên: nó phải nhận đúng mọi yêu cầu hợp lệ (nhận sai một yêu cầu hợp lệ là mất một chuyến), từ chối mọi yêu cầu sai với **đúng** mã lỗi (ứng dụng khách hiển thị thông báo theo mã lỗi), và tính đúng tiền cho yêu cầu được nhận. Bài giới hạn ở hai dịch vụ `ride` và `parcel`, không voucher, ngoài giờ cao điểm, không mưa. Đơn đồ ăn và voucher đi qua cùng chuỗi kiểm tra nhưng nằm ngoài phạm vi.

### 3.2. Business rules

| Mã | Quy tắc |
|---|---|
| R-VALID-01 | Nhiều vi phạm cùng lúc ⇒ chỉ báo lỗi **đầu tiên** theo thứ tự: xác thực (401) → vai trò (403) → định dạng `VALIDATION_ERROR` (400) → `CITY_NOT_SUPPORTED` → `DISTANCE_OUT_OF_RANGE` → `WEIGHT_OUT_OF_RANGE` → `VEHICLE_TOO_SMALL` → `SUBTOTAL_TOO_LOW` → lỗi voucher |
| R-API-02 | Lỗi kiểm tra đầu vào có thêm `error.details` (danh sách trường sai) |
| R-API-07 | `ride`: `{ kind, cityCode, vehicle, distanceKm }`; `parcel`: thêm `parcelWeightKg` |
| R-GEN-03 | 6 thành phố `SG`, `HN`, `DN`, `HP`, `CT`, `NT`; mã khác ⇒ `CITY_NOT_SUPPORTED` |
| R-GEN-04 | `vehicle`: `bike` hoặc `car` |
| R-FARE-02 | Quãng đường làm tròn 0,1 km half-up, rồi phải trong [0,5; 100] km |
| R-PARCEL-01 | Khối lượng làm tròn 0,1 kg half-up, rồi phải thoả 0 &lt; w ≤ 30; đơn bưu kiện bắt buộc có khối lượng |
| R-PARCEL-02 | w &gt; 20 kg bắt buộc ô tô; chọn xe máy ⇒ `VEHICLE_TOO_SMALL`; 20,0 kg vẫn đi xe máy |
| R-PARCEL-03 | Phụ phí 2.000 đ cho mỗi kg bắt đầu vượt 5 kg |
| R-FARE-01, 07, 08 | Bảng giá xe máy 12.000 + 4.500/km, tối thiểu 15.000 đ; ô tô 25.000 + 11.000/km, tối thiểu 30.000 đ; làm tròn lên 1.000 đ |

### 3.3. System context

Trong kiến trúc đích, yêu cầu đi qua API Gateway tới **Pricing & Quote Service**; quãng đường lấy từ bản đồ MapViet. Lab giản lược: client gửi thẳng `distanceKm` (R-GEN-06). Yêu cầu đi qua hai lớp kiểm tra trước khi tính cước: lớp định dạng (thư viện zod trong `src/api/schemas.ts`, trả 400 kèm `details`) và lớp quy tắc nghiệp vụ (`normalizePriceRequest` trong `src/domain/pricing.ts`, kiểm theo đúng thứ tự R-VALID-01, dừng ở lỗi đầu).

### 3.4. Rủi ro

| Rủi ro | Hậu quả | Kỹ thuật có khả năng phát hiện |
|---|---|---|
| Quên cả một phân vùng (thành phố chưa mở được báo giá) | Khách đặt được chuyến nơi không có tài xế | EP |
| Biên lệch hoặc so sánh trước khi làm tròn (0,45 km bị từ chối) | Khách hợp lệ bị từ chối | BVA |
| Quên một quy tắc tổ hợp (xe máy chở 25 kg) | Tài xế xe máy nhận đơn không chở nổi | Bảng quyết định |
| Đổi thứ tự kiểm (báo `DISTANCE_OUT_OF_RANGE` trước `CITY_NOT_SUPPORTED`) | Ứng dụng hiển thị sai thông báo | Bảng quyết định |
| Bộ test có ca âm gộp lỗi | Hai lỗi đầu trong bảng này lọt qua với bộ test «xanh» | Single fault |

## 4. Phân tích kỹ thuật

### 4.1. Chọn kỹ thuật cho từng phần của tính năng

Bước đầu tiên không phải là sinh ca, mà là đọc **cấu trúc** của từng quy tắc và hỏi kỹ thuật nào sinh ra để kiểm cấu trúc đó.

| Phần của tính năng | Cấu trúc | Kỹ thuật chính | Vì sao |
|---|---|---|---|
| `kind`, `cityCode`, `vehicle` | tập giá trị rời rạc, không thứ tự | EP | không có «sát biên» — BVA không áp được |
| `distanceKm`, `parcelWeightKg` | số có thứ tự, có khoảng hợp lệ, có làm tròn | EP rồi BVA | EP cho phân vùng; BVA đặt giá trị đúng sát ranh |
| Phụ phí khối lượng | đầu ra nhảy bậc theo khối lượng | EP đầu ra + BVA ở bậc đầu | 25 biên nội bộ — chỉ chọn bậc đầu 5,0/5,1 theo rủi ro |
| Xe máy và khối lượng | quy tắc **tổ hợp** hai tham số | Bảng quyết định | kết quả phụ thuộc tổ hợp, không phụ thuộc từng giá trị |
| Thứ tự lỗi R-VALID-01 | quy tắc «nếu điều kiện trước sai thì không xét điều kiện sau» | Bảng quyết định | mỗi cột là một «lỗi đầu tiên» |
| Vòng đời báo giá (còn hạn, hết hạn, đã dùng) | trạng thái và sự kiện | State transition | **không thuộc tính năng này** — xem dưới |

Dòng cuối là một quyết định **không dùng** kỹ thuật, và nó đáng được ghi lại như một quyết định. Kiểm tra một yêu cầu báo giá là xử lý không trạng thái: mỗi yêu cầu được xét độc lập, kết quả không phụ thuộc yêu cầu trước. State transition testing ([TEST-034](/articles/state-transition-testing-co-ban)) không có gì để mô hình hoá ở đây. Trạng thái xuất hiện **sau** khi báo giá được lưu: báo giá còn hạn hay đã hết hạn (R-QUOTE-01), đã dùng cho một chuyến chưa (R-QUOTE-03). Đó là tính năng «đặt chuyến từ báo giá», phần capstone [TCD-10](/articles/test-case-design-thuc-hanh-dat-chuyen) sẽ dùng state transition.

Thứ tự áp dụng là **EP → BVA → bảng quyết định**: EP cho phân vùng, BVA dùng các phân vùng có thứ tự của EP để tìm biên, bảng quyết định dùng kết quả hợp lệ/không hợp lệ của từng tham số làm điều kiện. Mỗi kỹ thuật dùng lại đầu ra của kỹ thuật trước.

### 4.2. Mô hình EP — 20 phân vùng

| Tham số | Hợp lệ | Không hợp lệ |
|---|---|---|
| `kind` | K1 `ride` · K2 `parcel` | K3 giá trị khác (400) |
| `cityCode` | C1 một trong 6 mã | C2 chuỗi 1–10 ký tự không được phục vụ (422) · C3 sai định dạng (400) |
| `vehicle` | V1 `bike` · V2 `car` | V3 giá trị khác (400) |
| `distanceKm` | D2 sau làm tròn trong [0,5; 100] | D1 &lt; 0,5 (422) · D3 &gt; 100 (422) · D4 không phải số (400) |
| `parcelWeightKg` | W2 (0; 20] · W3 (20; 30] | W1 ≤ 0 (422) · W4 &gt; 30 (422) · W5 không phải số hoặc thiếu (400) |
| Phụ phí (đầu ra) | S1 0 đ (w ≤ 5) · S2 &gt; 0 (w &gt; 5) | — |

10 phân vùng hợp lệ, 10 không hợp lệ. W2 và W3 tách nhau vì R-PARCEL-02 xử lý chúng khác nhau khi đi xe máy. Ranh giới này là đầu vào cho cả BVA (biên 20,0/20,1) lẫn bảng quyết định (điều kiện «xe đủ tải»).

### 4.3. Mô hình BVA — 12 giá trị

BVA 2-value trên **giá trị thô** client gửi, độ chính xác 0,01. [TEST-032](/articles/boundary-value-analysis) đã chỉ ra rằng biên trên giá trị thô bắt được lỗi của phép làm tròn, điều mà biên trên giá trị đã làm tròn không bắt được.

| Biên | Giá trị (thô) | Phía hợp lệ | Phía không hợp lệ |
|---|---|---|---|
| Quãng đường dưới (R-FARE-02) | 0,44 · 0,45 | 0,45 | 0,44 ⇒ `DISTANCE_OUT_OF_RANGE` |
| Quãng đường trên | 100,04 · 100,05 | 100,04 | 100,05 ⇒ `DISTANCE_OUT_OF_RANGE` |
| Khối lượng dưới (R-PARCEL-01) | 0,04 · 0,05 | 0,05 | 0,04 ⇒ `WEIGHT_OUT_OF_RANGE` |
| Khối lượng trên | 30,04 · 30,05 | 30,04 (ô tô) | 30,05 ⇒ `WEIGHT_OUT_OF_RANGE` |
| Ngưỡng xe máy (R-PARCEL-02) | 20,04 · 20,05, **với xe máy** | 20,04 | 20,05 ⇒ `VEHICLE_TOO_SMALL` |
| Bậc phụ phí đầu tiên (R-PARCEL-03) | 5,04 · 5,05 | 5,04 ⇒ 0 đ | 5,05 ⇒ 2.000 đ (vẫn hợp lệ) |

Hai giá trị biên ở dòng cuối đều hợp lệ: đó là biên giữa hai **phân vùng đầu ra** S1 và S2. Hai giá trị ngưỡng xe máy chỉ là coverage item khi `vehicle = bike`; với ô tô, 20,05 kg chỉ là một giá trị bất kỳ trong W3.

### 4.4. Bảng quyết định — thứ tự lỗi cho báo giá bưu kiện

Các điều kiện là kết quả hợp lệ/không hợp lệ của từng bước kiểm tra, xếp theo R-VALID-01. Bảng giới hạn cho `parcel`, vì chỉ dịch vụ này đi qua đủ năm bước.

| Điều kiện / hành động | R1 | R2 | R3 | R4 | R5 | R6 |
|---|---|---|---|---|---|---|
| F0 · định dạng hợp lệ | F | T | T | T | T | T |
| F1 · thành phố được phục vụ | – | F | T | T | T | T |
| F2 · quãng đường trong khoảng | – | – | F | T | T | T |
| F3 · khối lượng trong khoảng | – | – | – | F | T | T |
| F4 · xe đủ tải (không phải xe máy chở &gt; 20 kg) | – | – | – | – | F | T |
| Kết quả | 400 `VALIDATION_ERROR` | `CITY_NOT_SUPPORTED` | `DISTANCE_OUT_OF_RANGE` | `WEIGHT_OUT_OF_RANGE` | `VEHICLE_TOO_SMALL` | 200 |

Bảng đầy đủ có 2⁵ = 32 cột; bảng rút gọn có 6 cột. Checksum theo cách của [TEST-033](/articles/decision-table-testing): R1 có bốn `–` ⇒ 16; R2 ⇒ 8; R3 ⇒ 4; R4 ⇒ 2; R5, R6 ⇒ 1 mỗi cột. Tổng 32, bằng bảng đầy đủ: không thiếu, không chồng lấn.

Đây là dạng bảng «lỗi đầu tiên» điển hình, và nó nối thẳng tới nguyên tắc single fault. Mỗi `–` nghĩa là «kết quả không phụ thuộc điều kiện này **theo đặc tả**». Khi chọn giá trị cụ thể cho `–`, ta có hai lựa chọn:

- chọn **T** (hợp lệ) cho mọi `–` ⇒ ca âm chỉ có một vi phạm: single fault;
- chọn **F** cho một số `–` ⇒ ca âm có nhiều vi phạm, và mọi vi phạm sau vi phạm đầu bị che.

Cả hai lựa chọn đều đạt 100% decision table coverage, vì mỗi cột vẫn được chạy. Chỉ lựa chọn thứ nhất không biến coverage item của EP và BVA thành coverage item «ảo». Lab sinh mọi ca bảng quyết định với `–` = T.

### 4.5. Bộ ca của từng kỹ thuật, và độ phủ chéo

Sinh riêng từng bộ theo đúng kỹ thuật:

- **Bộ EP (13 ca):** Each Choice, ba ca dương ghép phân vùng hợp lệ (`ride`/xe máy/7,4 km; `parcel`/ô tô/25 kg; `parcel`/xe máy/3 kg) và mười ca âm, mỗi ca một phân vùng không hợp lệ ở giá trị giữa (0,2 km; 150 km; `DL`; …).
- **Bộ BVA (12 ca):** mỗi giá trị biên một ca. Biên quãng đường đặt trên nền `ride`/xe máy; biên khối lượng trên nền `parcel`/ô tô; ngưỡng xe máy trên nền `parcel`/xe máy.
- **Bộ bảng quyết định (6 ca):** mỗi cột một ca `parcel`, `–` = T.

Rồi đo độ phủ của **mỗi bộ theo cả ba kỹ thuật**. Đây là các con số lab tính bằng hàm phân loại, không đếm tay:

| Bộ | Số ca | EP (20) | BVA (12) | DT (6) | Tổng (38) |
|---|---|---|---|---|---|
| EP | 13 | **20/20** | 0/12 | 3/6 | 23/38 |
| BVA | 12 | 14/20 | **12/12** | 3/6 | 29/38 |
| DT | 6 | 10/20 | 0/12 | **6/6** | 16/38 |
| Gộp ba bộ | 31 | 20/20 | 12/12 | 6/6 | **38/38** |

Bảng nói hai điều. Mỗi kỹ thuật đạt 100% theo định nghĩa của chính nó và bỏ trống phần lớn định nghĩa của kỹ thuật khác: bộ EP không có ca biên nào, bộ bảng quyết định không chạm một nửa số phân vùng. Và bộ BVA phủ chéo nhiều nhất (14 phân vùng EP), vì mỗi giá trị biên nằm trong một phân vùng. Nhưng 14/20 vẫn thiếu đúng sáu phân vùng lỗi định dạng (K3, C2, C3, V3, D4, W5): BVA không áp được cho tham số rời rạc.

Cách đếm có một quy ước quan trọng: **ca âm chỉ phủ đúng lỗi nó gây ra**. Ca «thành phố `DL`» trên nền bưu kiện 7,4 km, 3 kg phủ C2 và cột R2, **không** phủ D2 hay W2. Đúng là quãng đường và khối lượng trong ca đó hợp lệ, nhưng hệ thống dừng ở thành phố và không xử lý chúng. Quy ước này là cách viết nguyên tắc single fault thành con số.

### 4.6. Gộp ca dương

Bộ gộp 31 ca phủ đủ 38/38 item, nhưng nhiều ca thừa. Ở ca âm, không gộp được gì: mỗi giá trị sai cần một ca. Ở ca dương, có thể gộp: một ca bưu kiện xe máy 0,45 km, 0,05 kg mang đồng thời K2, C1, V1, D2, W2, S1, biên 0,45, biên 0,05 và cột R6. Ca này thay được ba ca dương riêng lẻ.

Lab sinh một **kho ca dương ghép**: tích Đề-các của mọi giá trị hợp lệ mà các kỹ thuật đã chọn (2 loại xe × 3 quãng đường {7,4; 0,45; 100,04} × {không khối lượng cho `ride`, 7 khối lượng cho `parcel`}), bỏ các tổ hợp bị từ chối (xe máy với 25 kg hoặc 30,04 kg). Kết quả là 42 ca dương ghép. Không ca nào trong kho đó là ca âm: **chỉ ca dương được phép ghép**.

### 4.7. Rút gọn tham lam

Kho cuối có 31 + 42 = 73 ca. Thuật toán tham lam:

1. Mỗi bước, chọn ca phủ **nhiều item chưa được phủ nhất**; hoà thì chọn ca đứng trước trong kho (ca của kỹ thuật đứng trước ca ghép).
2. Lặp tới khi mọi item đã được phủ.
3. Quét ngược: bỏ ca nào mà các ca còn lại đã phủ hết item của nó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Quy trình phối hợp kỹ thuật: ba mô hình EP 20 phân vùng, BVA 12 giá trị, bảng quyết định 6 cột sinh ba bộ 13, 12 và 6 ca, gộp thành 31 ca. Thêm 42 ca dương ghép thành kho 73 ca. Rút gọn tham lam còn 18 ca, gồm 6 ca dương và 12 ca âm, vẫn phủ 38 trên 38 coverage item.">
<defs>
<marker id="tcd06-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-b">Từ ba mô hình kỹ thuật tới một bộ ca không thừa</text>
<rect x="20" y="44" width="170" height="54" rx="8" class="f-pk-soft s-pk"/>
<text x="105" y="66" text-anchor="middle" class="t-b t-pk">EP · 20 phân vùng</text>
<text x="105" y="86" text-anchor="middle" class="t-small">bộ EP: 13 ca</text>
<rect x="20" y="114" width="170" height="54" rx="8" class="f-warn-soft s-warn"/>
<text x="105" y="136" text-anchor="middle" class="t-b t-warn">BVA · 12 giá trị</text>
<text x="105" y="156" text-anchor="middle" class="t-small">bộ BVA: 12 ca</text>
<rect x="20" y="184" width="170" height="54" rx="8" class="f-phys-soft s-phys"/>
<text x="105" y="206" text-anchor="middle" class="t-b t-phys">DT · 6 cột</text>
<text x="105" y="226" text-anchor="middle" class="t-small">bộ DT: 6 ca</text>
<line x1="190" y1="71" x2="248" y2="130" class="s-muted" stroke-width="1.5" marker-end="url(#tcd06-a)"/>
<line x1="190" y1="141" x2="248" y2="141" class="s-muted" stroke-width="1.5" marker-end="url(#tcd06-a)"/>
<line x1="190" y1="211" x2="248" y2="152" class="s-muted" stroke-width="1.5" marker-end="url(#tcd06-a)"/>
<rect x="250" y="110" width="160" height="62" rx="8" class="f-surface s-line"/>
<text x="330" y="134" text-anchor="middle" class="t-b">Gộp ba bộ</text>
<text x="330" y="156" text-anchor="middle" class="t-small">31 ca · 38/38 item</text>
<rect x="250" y="200" width="160" height="62" rx="8" class="f-surface s-line dash"/>
<text x="330" y="224" text-anchor="middle" class="t-small t-b">Ca dương ghép</text>
<text x="330" y="244" text-anchor="middle" class="t-small">42 ca (chỉ ca dương)</text>
<line x1="410" y1="141" x2="468" y2="141" class="s-muted" stroke-width="1.5" marker-end="url(#tcd06-a)"/>
<line x1="410" y1="231" x2="468" y2="152" class="s-muted" stroke-width="1.5" marker-end="url(#tcd06-a)"/>
<rect x="470" y="110" width="160" height="62" rx="8" class="f-surface s-line"/>
<text x="550" y="134" text-anchor="middle" class="t-b">Kho</text>
<text x="550" y="156" text-anchor="middle" class="t-small">73 ca</text>
<line x1="630" y1="141" x2="688" y2="141" class="s-muted" stroke-width="1.5" marker-end="url(#tcd06-a)"/>
<text x="659" y="132" text-anchor="middle" class="t-10">tham lam</text>
<rect x="690" y="100" width="210" height="82" rx="8" class="f-phys-soft s-phys"/>
<text x="795" y="124" text-anchor="middle" class="t-b t-phys">Bộ cuối · 18 ca</text>
<text x="795" y="146" text-anchor="middle" class="t-small">6 ca dương · 12 ca âm</text>
<text x="795" y="166" text-anchor="middle" class="t-small">EP 20/20 · BVA 12/12 · DT 6/6</text>
<rect x="470" y="210" width="430" height="52" rx="8" class="f-bg s-warn dash"/>
<text x="685" y="232" text-anchor="middle" class="t-small">Ca âm không bao giờ được ghép: mỗi ca âm đúng một giá trị sai</text>
<text x="685" y="250" text-anchor="middle" class="t-small">vì R-VALID-01 chỉ báo lỗi đầu tiên</text>
<text x="460" y="288" text-anchor="middle" class="t-small">Mọi con số trong hình được kiểm bằng test trong articles/test-case-design-phoi-hop-ky-thuat</text>
</svg>
</div>
<figcaption><b>Hình 1 · Phối hợp ba kỹ thuật.</b> Độ phủ được đo theo từng kỹ thuật ở mọi bước; rút gọn chỉ hợp lệ khi cả ba con số giữ nguyên.</figcaption>
</figure>

Kết quả: **18 ca, vẫn 38/38 item**, gồm 6 ca dương và 12 ca âm. Lab kiểm thêm rằng bộ cuối **không có ca thừa**: bỏ bất kỳ ca nào cũng mất ít nhất một item.

| Ca | Loại | Nội dung | Mong đợi | Phủ |
|---|---|---|---|---|
| GH-09 | dương | `parcel`, xe máy, 0,45 km, 0,05 kg | 200 · 15.000 đ | K2 C1 V1 D2 W2 S1 · biên 0,45 và 0,05 · R6 |
| GH-39 | dương | `parcel`, ô tô, 100,04 km, 30,04 kg | 200 · 1.175.000 đ | K2 C1 V2 D2 W3 S2 · biên 100,04 và 30,04 · R6 |
| BVA-09 | dương | `parcel`, xe máy, 7,4 km, 20,04 kg | 200 · 76.000 đ | V1 W2 S2 · ngưỡng 20,04 · R6 |
| BVA-11 | dương | `parcel`, ô tô, 7,4 km, 5,04 kg | 200 · 107.000 đ | biên phụ phí 5,04 · R6 |
| BVA-12 | dương | `parcel`, ô tô, 7,4 km, 5,05 kg | 200 · 109.000 đ | biên phụ phí 5,05 · R6 |
| EP-01 | dương | `ride`, xe máy, 7,4 km | 200 · 46.000 đ | K1 C1 V1 D2 |
| EP-04 | âm | `kind: "taxi"` | 400 · `details` = [`kind`] | K3 |
| EP-06 | âm | `cityCode: ""` (`ride`) | 400 · [`cityCode`] | C3 |
| EP-07 | âm | `vehicle: "truck"` (`ride`) | 400 · [`vehicle`] | V3 |
| EP-10 | âm | `distanceKm: "7.4"` (chuỗi) | 400 · [`distanceKm`] | D4 |
| EP-13 | âm | `parcelWeightKg: "5"` (chuỗi) | 400 · [`parcelWeightKg`] | W5 · R1 |
| DT-R2 | âm | `parcel`, `cityCode: "DL"` | 422 `CITY_NOT_SUPPORTED` | C2 · R2 |
| DT-R3 | âm | `parcel`, 150 km | 422 `DISTANCE_OUT_OF_RANGE` | D3 · R3 |
| BVA-01 | âm | `ride`, 0,44 km | 422 `DISTANCE_OUT_OF_RANGE` | D1 · biên 0,44 |
| BVA-04 | âm | `ride`, 100,05 km | 422 `DISTANCE_OUT_OF_RANGE` | D3 · biên 100,05 |
| BVA-05 | âm | `parcel`, ô tô, 0,04 kg | 422 `WEIGHT_OUT_OF_RANGE` | W1 · biên 0,04 · R4 |
| BVA-08 | âm | `parcel`, ô tô, 30,05 kg | 422 `WEIGHT_OUT_OF_RANGE` | W4 · biên 30,05 · R4 |
| BVA-10 | âm | `parcel`, xe máy, 20,05 kg | 422 `VEHICLE_TOO_SMALL` | ngưỡng 20,05 · R5 |

Oracle của các ca dương tính tay từ SPEC: GH-09 là 12.000 + 4.500 × 0,5 = 14.250 đ, nâng lên tối thiểu 15.000 đ. GH-39 là 25.000 + 11.000 × 100 + 2.000 × 25 = 1.175.000 đ. BVA-09 là 45.300 + 2.000 × ⌈20,0 − 5⌉ = 75.300 đ ⇒ 76.000 đ. BVA-11 là 25.000 + 11.000 × 7,4 = 106.400 đ ⇒ 107.000 đ; BVA-12 thêm 2.000 đ phụ phí ⇒ 109.000 đ.

Ba điều đáng chú ý ở bộ cuối:

- **Đại diện EP «ở giữa» biến mất** khi có giá trị biên phủ cùng phân vùng: không còn ca 0,2 km cho D1, vì 0,44 km cũng thuộc D1. [TEST-031](/articles/equivalence-partitioning) khuyên không lấy biên làm đại diện EP, vì khi ca đỏ thì khó biết lỗi do cả phân vùng hay do biên. Rút gọn đã đánh đổi khả năng chẩn đoán đó để lấy số ca nhỏ hơn. Với một bản cài đặt quên **cả** cận dưới, ca 0,44 vẫn đỏ, nên khả năng phát hiện không mất; chỉ phần chẩn đoán khó hơn một chút.
- **Ca âm của bảng quyết định chiếm chỗ ca âm của EP** khi chúng phủ cùng phân vùng: DT-R2 (`parcel`, `DL`) phủ C2 và R2, nên ca `ride`, `DL` của bộ EP thành thừa.
- **Thuật toán tham lam không bảo đảm nhỏ nhất.** BVA-01 và BVA-04 đặt biên quãng đường trên nền `ride`, nên không phủ cột R3, và thuật toán phải giữ thêm DT-R3. Lab thử đặt bốn ca biên quãng đường lên nền `parcel` thì rút gọn còn **17 ca**. Cách chọn **nền** của ca (các tham số «còn lại» đặt giá trị gì) cũng là một quyết định test design.

Nếu không có kho ca dương ghép, rút gọn chỉ xuống được **19 ca**. Ghép ca dương ở tính năng này chỉ tiết kiệm một ca. Lý do: mỗi ca bưu kiện chỉ mang được **một** khối lượng, nên năm giá trị biên khối lượng hợp lệ (0,05; 30,04; 20,04; 5,04; 5,05) luôn cần năm ca dương khác nhau. Gộp ca dương hiệu quả nhất khi các coverage item nằm trên **các tham số khác nhau**.

### 4.8. Một ca âm gộp lỗi

Ca MASK-01 được viết theo kiểu hay gặp: «gom các giá trị sai vào một ca cho đỡ tốn»: bưu kiện, thành phố `DL`, 0,44 km, xe máy, 25 kg. Ca này **vi phạm ba quy tắc**: R-GEN-03, R-FARE-02 và R-PARCEL-02.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 190" role="img" aria-label="Chuỗi kiểm tra R-VALID-01 cho ca MASK-01: bước định dạng đạt, bước thành phố thất bại với DL và trả CITY_NOT_SUPPORTED. Ba bước quãng đường 0,44 km, khối lượng, xe máy 25 kg không bao giờ được chạy.">
<defs>
<marker id="tcd06-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-b">MASK-01: R-VALID-01 dừng ở lỗi đầu, các giá trị sai phía sau không được xử lý</text>
<rect x="20" y="50" width="140" height="50" rx="8" class="f-phys-soft s-phys"/>
<text x="90" y="72" text-anchor="middle" class="t-small t-b">Định dạng</text>
<text x="90" y="90" text-anchor="middle" class="t-small">đạt</text>
<rect x="195" y="50" width="140" height="50" rx="8" class="f-warn-soft s-warn"/>
<text x="265" y="72" text-anchor="middle" class="t-small t-b">Thành phố DL</text>
<text x="265" y="90" text-anchor="middle" class="t-small t-warn">dừng ở đây</text>
<rect x="370" y="50" width="160" height="50" rx="8" class="f-bg s-line dash"/>
<text x="450" y="72" text-anchor="middle" class="t-small">Quãng đường 0,44</text>
<text x="450" y="90" text-anchor="middle" class="t-small">không chạy</text>
<rect x="565" y="50" width="140" height="50" rx="8" class="f-bg s-line dash"/>
<text x="635" y="72" text-anchor="middle" class="t-small">Khối lượng 25</text>
<text x="635" y="90" text-anchor="middle" class="t-small">không chạy</text>
<rect x="740" y="50" width="160" height="50" rx="8" class="f-bg s-line dash"/>
<text x="820" y="72" text-anchor="middle" class="t-small">Xe máy &gt; 20 kg</text>
<text x="820" y="90" text-anchor="middle" class="t-small">không chạy</text>
<line x1="160" y1="75" x2="193" y2="75" class="s-muted" stroke-width="1.5" marker-end="url(#tcd06-b)"/>
<line x1="335" y1="75" x2="368" y2="75" class="s-muted dash" stroke-width="1.2"/>
<line x1="530" y1="75" x2="563" y2="75" class="s-muted dash" stroke-width="1.2"/>
<line x1="705" y1="75" x2="738" y2="75" class="s-muted dash" stroke-width="1.2"/>
<line x1="265" y1="100" x2="265" y2="130" class="s-warn" stroke-width="1.5" marker-end="url(#tcd06-b)"/>
<text x="265" y="150" text-anchor="middle" class="t-small t-b">422 CITY_NOT_SUPPORTED</text>
<text x="640" y="150" text-anchor="middle" class="t-small">Phủ thật: C2 + cột R2. Khai «phủ»: thêm D1, biên 0,44, R3, R5</text>
<text x="460" y="178" text-anchor="middle" class="t-small">Cùng kết quả trên bản đúng và trên bản có lỗi Q, X ⇒ ca xanh ở cả hai</text>
</svg>
</div>
<figcaption><b>Hình 2 · Che lỗi.</b> Ca chạy xanh, đúng R-VALID-01, nhưng chỉ kiểm được một trong ba quy tắc nó chứa.</figcaption>
</figure>

Qua API, MASK-01 trả 422 `CITY_NOT_SUPPORTED`, đúng với SPEC, nên ca xanh. Ca này không sai. Vấn đề là nó **phủ ít hơn nhiều so với điều người viết tưởng**: hàm phân loại của lab tính được nó chỉ phủ C2 và cột R2.

Để thấy hậu quả, lab có một bản chép lớp kiểm tra (`ban-loi.ts`) với hai lỗi cố ý:

- **Lỗi Q:** quên cận dưới quãng đường — chỉ kiểm &gt; 100 km.
- **Lỗi X:** quên R-PARCEL-02 — xe máy chở được trên 20 kg.

Chạy từng bộ ca trên bản lỗi, ca nào cho kết quả khác oracle là ca đỏ:

| Bộ ca | Ca đỏ trên bản lỗi | Bắt lỗi Q? | Bắt lỗi X? |
|---|---|---|---|
| MASK-01 (một ca gộp ba lỗi) | không ca nào | không | không |
| Bộ EP (13 ca) | EP-08 · 0,2 km | **có** | không — EP không có ca tổ hợp xe máy × khối lượng |
| Bộ DT (6 ca) | DT-R5 · xe máy 25 kg | không — cột R3 chọn 150 km, phía trên | **có** |
| Bộ BVA (12 ca) | BVA-01 · 0,44 km; BVA-10 · xe máy 20,05 kg | **có** | **có** |
| Bộ cuối (18 ca) | BVA-01; BVA-10 | **có** | **có** |

Bảng này tóm tắt cả bài. MASK-01 chứa đúng hai giá trị gây lỗi cho bản lỗi (0,44 km và xe máy 25 kg) mà vẫn không bắt được lỗi nào. Mỗi kỹ thuật đơn lẻ bỏ lọt ít nhất một lỗi, trừ BVA ở ví dụ này. Bộ cuối 18 ca, nhỏ hơn bộ gộp 31 ca, bắt cả hai, vì rút gọn **giữ nguyên coverage item của mọi kỹ thuật**.

### 4.9. Một chi tiết: lỗi định dạng không che nhau, trừ một trường hợp

Lớp định dạng (400) hoạt động khác lớp nghiệp vụ (422). Gửi `parcel` với `cityCode: ""` **và** `vehicle: "truck"`: `details` liệt kê **cả hai** trường. Lớp định dạng kiểm mọi trường rồi báo tất cả, nên hai lỗi định dạng trong một ca không che nhau. Ngoại lệ là `kind`: `kind` sai thì hệ thống không biết dùng lược đồ của dịch vụ nào, nên `details` chỉ có `kind`. Lab kiểm cả hai hành vi.

Đây là hành vi của cài đặt (zod với `discriminatedUnion`), không phải yêu cầu tường minh của SPEC. R-API-02 chỉ nói `details` là «danh sách trường sai». Hệ quả cho test design: dù lỗi định dạng không che nhau trong lab, bộ ca vẫn giữ single fault, vì một bản cài đặt khác hoàn toàn được phép dừng ở trường sai đầu tiên mà vẫn đúng SPEC. Câu hỏi «`details` phải liệt kê tất cả hay chỉ trường đầu?» nên gửi BA.

## 5. Thực hành

### 5.1. Tổ chức mã

| Tệp | Vai trò | Số test |
|---|---|---|
| `mo-hinh.ts` | Mô hình EP, BVA, DT viết thành dữ liệu; oracle từ SPEC; hàm phân loại `coverageItems`; các bộ ca; kho ghép; `greedyMinimize` | — |
| `ban-loi.ts` | Bản chép lớp kiểm tra có lỗi Q và X | — |
| `mo-hinh.test.ts` | Oracle tính tay, độ phủ từng bộ, rút gọn | 21 |
| `api.test.ts` | Bộ cuối qua `POST /api/quotes`; đối chiếu oracle trên cả 73 ca; MASK-01; lỗi định dạng | 22 |
| `che-loi.test.ts` | MASK-01 và các bộ ca trên bản lỗi | 6 |

### 5.2. Oracle không gọi code sản phẩm

Giá trị số trong mô hình được viết bằng **chuỗi thập phân** (`d('100.05')`). Oracle làm tròn half-up bằng số học nguyên trên chuỗi đó, không qua phép nhân số thực. Mục đích là để oracle độc lập với `roundToTenth` của sản phẩm. Nếu oracle gọi chính hàm làm tròn của sản phẩm, một lỗi làm tròn sẽ xuất hiện ở cả hai phía và test vẫn xanh ([TCD-04](/articles/test-case-design-expected-result-test-oracle)).

```ts
/** Chuỗi thập phân ⇒ số phần mười, làm tròn half-up (R-FARE-02, R-PARCEL-01). */
export function tenthsHalfUp(dec: string): number {
  const neg = dec.startsWith('-');
  const [ip, fp = ''] = (neg ? dec.slice(1) : dec).split('.');
  const digits = Number(ip + fp);
  let tenths: number;
  if (fp.length <= 1) tenths = digits * 10 ** (1 - fp.length);
  else {
    const scale = 10 ** (fp.length - 1);
    tenths = Math.floor(digits / scale) + ((digits % scale) * 2 >= scale ? 1 : 0);
  }
  return neg ? -tenths : tenths;
}
```

Lab đối chiếu oracle với hệ thống trên **cả 73 ca** của kho, không chỉ 18 ca của bộ cuối: mọi status, mã lỗi, tổng tiền và `details` đều khớp. Đây là bước kiểm chính oracle trước khi tin nó.

### 5.3. Phân loại: ca âm chỉ phủ lỗi của nó

```ts
export function coverageItems(c: QuoteCase): string[] {
  const o = expectedOutcome(c);
  if (o.status === 200) return [/* mọi phân vùng hợp lệ, giá trị biên, cột R6 */];
  if (o.status === 400) return [/* phân vùng của các trường trong details */, ...dt('R1')];
  switch (o.code) {
    case 'CITY_NOT_SUPPORTED': return ['EP:C2', ...dt('R2')];
    case 'DISTANCE_OUT_OF_RANGE': return [/* D1 hoặc D3 */, ...bvaHits(c, 'distanceKm'), ...dt('R3')];
    // … WEIGHT_OUT_OF_RANGE, VEHICLE_TOO_SMALL
  }
}
```

Coverage item được **tính từ đầu vào**, không do người viết ca tự khai. Một ca khai «phủ D1» mà thực ra dừng ở thành phố sẽ không được tính D1. Đó là cách MASK-01 bị lộ.

### 5.4. Rút gọn

```ts
export function greedyMinimize(pool: QuoteCase[]): QuoteCase[] {
  // … uncovered = mọi item mà kho phủ được
  while (uncovered.size > 0) {
    // chọn ca có nhiều item chưa phủ nhất; hoà thì ca đứng trước
  }
  // quét ngược: bỏ ca mà các ca còn lại đã phủ hết item của nó
}
```

### 5.5. Chạy qua API

```ts
const app = createApp(createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 16, 9, 0)) }).deps, { logErrors: false });

it.each(greedyMinimize(POOL))('$id · $note', async (c) => {
  const res = await quote(toBody(c));
  const exp = expectedOutcome(c);
  expect(res.status).toBe(exp.status);
  if (exp.status === 200) expect(res.body.quote.total).toBe(exp.total);
  else expect(res.body.error.code).toBe(exp.code);
  if (exp.status === 400) expect(paths(res.body)).toEqual([...exp.fields].sort());
});
```

Ca lỗi định dạng kiểm `details` đúng bằng tập trường đang kiểm, như [TEST-031](/articles/equivalence-partitioning) khuyên: nếu không, một ca vô tình gửi sai trường khác cũng ra 400 và vẫn xanh.

```bash
cd examples/toi-noi-testing
npx vitest run articles/test-case-design-phoi-hop-ky-thuat
```

Kết quả thật (vitest 3.2.7, Node 22, khi viết bài):

```text
 ✓ articles/test-case-design-phoi-hop-ky-thuat/mo-hinh.test.ts (21 tests)
 ✓ articles/test-case-design-phoi-hop-ky-thuat/che-loi.test.ts (6 tests)
 ✓ articles/test-case-design-phoi-hop-ky-thuat/api.test.ts (22 tests)

 Test Files  3 passed (3)
      Tests  49 passed (49)
```

## 6. Góc nhìn nâng cao

**Coverage của từng kỹ thuật phải được báo cáo riêng.** «Bộ ca đạt 100%» chỉ có nghĩa khi đi kèm tên kỹ thuật và mô hình. Báo cáo của bài là «EP 20/20, BVA 12/12 (2-value, giá trị thô, độ chính xác 0,01), DT 6/6 (bảng rút gọn)». Mỗi cụm trong ngoặc là một giả định có thể bị chất vấn, và người đọc báo cáo cần thấy giả định đó.

**Rút gọn đúng với mô hình, không đúng với sản phẩm.** Bộ cuối không mất item nào **của mô hình**. Nếu mô hình thiếu một phân vùng (ví dụ mã thành phố chữ thường `sg`, mà [TEST-031](/articles/equivalence-partitioning) đã chỉ ra SPEC không nói gì), rút gọn không tự nhận ra. Trước khi rút gọn, mô hình cần được review như một sản phẩm của test analysis ([TCD-01](/articles/test-case-design-test-analysis-test-condition)).

**Bộ cuối ít dư thừa hơn, nên nhạy hơn với một ca bị hỏng.** 31 ca có dư thừa; tắt một ca, item vẫn còn ca khác phủ. 18 ca không dư thừa; tắt một ca (`it.skip` khi gỡ lỗi rồi quên bật lại) là mất item. Lab kiểm «không ca thừa» như một test, và chính test đó báo khi có ca bị tắt nhầm. Ở dự án thật, có thể giữ chủ ý một ít dư thừa cho các item rủi ro cao.

**Khi có nhiều tham số tự do, chuyển sang kỹ thuật tổ hợp.** Bài chỉ ghép ca dương để phủ **mỗi** giá trị ít nhất một lần, tương tự Each Choice. Nếu rủi ro nằm ở **tương tác** giữa hai tham số (ô tô × quãng đường dài × bưu kiện nặng), cần [pairwise](/articles/pairwise-testing) hoặc [combinatorial testing](/articles/combinatorial-testing). Kho ca ghép của lab có thể dùng làm điểm khởi đầu cho các kỹ thuật đó.

**Thứ tự lỗi là một yêu cầu, không phải chi tiết cài đặt.** Bảng quyết định ở mục 4.4 mô tả **thứ tự**, nhưng các ca single fault của nó không kiểm được thứ tự. Giả sử một developer chuyển kiểm quãng đường lên trước kiểm thành phố. Ca DT-R2 (`parcel`, `DL`, 7,4 km) vẫn nhận `CITY_NOT_SUPPORTED`, vì quãng đường của nó hợp lệ, nên ca vẫn xanh. Muốn bắt kiểu đổi thứ tự này, cần một ca cố ý có **hai** lỗi đứng ở hai bước liền nhau. Đây là trường hợp hiếm mà ca có hai lỗi là **có chủ đích**: nó kiểm thứ tự, không kiểm từng quy tắc, và phải được ghi rõ như vậy (Bài tập 2).

**Sai lầm thường gặp:**

| Sai lầm | Hậu quả | Cách tránh |
|---|---|---|
| Áp từng kỹ thuật rồi cộng số ca | Bộ test phình to, nhiều ca thừa | Đo coverage chéo, rút gọn |
| Gộp nhiều giá trị sai vào một ca âm | Che lỗi; báo cáo coverage thổi phồng | Single fault; tính coverage từ kết quả thật |
| Chọn `–` của bảng quyết định là giá trị sai | Cột vẫn «đạt» nhưng biến thành ca gộp lỗi | `–` = hợp lệ cho ca âm |
| Báo «100% coverage» không kèm kỹ thuật | Không ai biết đã đủ cái gì | Báo cáo theo từng kỹ thuật và mô hình |
| Áp state transition cho xử lý không trạng thái | Mô hình rỗng, tốn công | Chọn kỹ thuật theo cấu trúc quy tắc |
| Oracle gọi lại hàm của sản phẩm | Lỗi xuất hiện ở cả hai phía, test vẫn xanh | Oracle độc lập, kiểm oracle trên toàn kho |

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Chọn nền cho ca biên

Mục 4.7 nói đặt bốn ca biên quãng đường lên nền `parcel` (ô tô, 3 kg) thay vì `ride` làm bộ cuối giảm từ 18 xuống 17 ca. Giải thích vì sao, và chỉ ra ca nào trong bộ 18 ca trở nên thừa.

<details><summary><b>Lời giải</b></summary>
<p>Trên nền <code>ride</code>, ca 0,44 km và 100,05 km phủ D1/D3 và biên, nhưng <b>không</b> phủ cột R3 của bảng quyết định (bảng chỉ dành cho <code>parcel</code>). Thuật toán phải giữ thêm DT-R3 (150 km, <code>parcel</code>) chỉ để phủ R3. Khi ca biên quãng đường ở nền <code>parcel</code>, chúng phủ luôn R3, nên DT-R3 thừa. Bài học: các tham số «còn lại» của một ca (nền) không trung lập; chọn nền sao cho ca rơi vào nhiều mô hình nhất. Lab kiểm con số 17 trong <code>mo-hinh.test.ts</code>.</p>
</details>

### Bài tập 2 — Kiểm thứ tự lỗi

Bộ cuối có DT-R2 (`parcel`, `DL`, 7,4 km, 3 kg ⇒ `CITY_NOT_SUPPORTED`). Một developer đổi thứ tự, kiểm quãng đường trước thành phố. Ca DT-R2 có bắt được thay đổi này không? Nếu không, thiết kế một ca bắt được nó, và giải thích vì sao ca đó không vi phạm nguyên tắc single fault theo đúng tinh thần của nguyên tắc.

<details><summary><b>Lời giải</b></summary>
<ul>
<li>Không. Quãng đường 7,4 km hợp lệ, nên dù kiểm trước hay sau, lỗi đầu tiên vẫn là thành phố.</li>
<li>Ca bắt được: <code>parcel</code>, <code>DL</code>, <b>0,44 km</b>, ô tô, 3 kg. Mong đợi theo R-VALID-01: <code>CITY_NOT_SUPPORTED</code>. Bản đổi thứ tự trả <code>DISTANCE_OUT_OF_RANGE</code>.</li>
<li>Ca này có hai giá trị sai, nhưng mục tiêu của nó là kiểm <b>thứ tự</b> — quy tắc R-VALID-01 — chứ không phải kiểm R-FARE-02. Single fault nhằm tránh việc tính ca này là phủ D1 hay biên 0,44. Khi ghi ca, chỉ khai nó phủ «thứ tự thành phố trước quãng đường»; D1 và biên 0,44 vẫn cần ca single fault riêng (BVA-01).</li>
</ul>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K3).** Test item có ba tham số: A có 2 phân vùng hợp lệ và 2 không hợp lệ; B có 3 hợp lệ và 1 không hợp lệ; C có 2 hợp lệ và 0 không hợp lệ. Ca dương được ghép tự do, ca âm theo single fault. Số ca tối thiểu để đạt 100% EP coverage?

- a) 5
- b) 6
- c) 9
- d) 12

<details><summary><b>Đáp án: b</b></summary>
<ul><li>Ca dương: bằng số phân vùng hợp lệ của tham số nhiều nhất = 3. Ca âm: 2 + 1 = 3. Tổng 6.</li><li>a) Sai — gộp hai phân vùng không hợp lệ vào một ca (che lỗi).</li><li>c) Sai — cộng thêm ca dương không cần thiết thay vì ghép phân vùng hợp lệ của các tham số vào cùng ca.</li><li>d) Sai — 2 × 3 × 2 tổ hợp.</li></ul>
</details>

**Câu 2 (K3).** Ca `parcel`, `cityCode: "DL"`, 0,44 km, xe máy, 25 kg nhận `CITY_NOT_SUPPORTED`. Theo cách đếm single fault, ca này phủ những coverage item nào trong mô hình của bài?

- a) C2, D1, biên 0,44, R5
- b) C2 và cột R2
- c) C2, D1 và cột R3
- d) Không item nào, vì ca có nhiều giá trị sai

<details><summary><b>Đáp án: b</b></summary>
<ul><li>a) Sai — R-VALID-01 dừng ở thành phố; quãng đường và khối lượng không được xử lý.</li><li>b) Đúng — đúng con số lab tính cho MASK-01.</li><li>c) Sai — cột R3 là «thành phố đúng, quãng đường sai».</li><li>d) Sai — lỗi đầu tiên vẫn được kiểm thật.</li></ul>
</details>

**Câu 3 (K3).** Quãng đường được làm tròn 0,1 km half-up rồi phải trong [0,5; 100]. Với 2-value BVA trên giá trị thô (độ chính xác 0,01), ca nào là **ca dương** phủ hai giá trị biên cùng lúc?

- a) `ride`, 0,44 km
- b) `parcel`, ô tô, 100,04 km, 30,04 kg
- c) `parcel`, xe máy, 100,04 km, 30,04 kg
- d) `parcel`, ô tô, 100,05 km, 30,04 kg

<details><summary><b>Đáp án: b</b></summary>
<ul><li>a) Sai — 0,44 km là ca âm.</li><li>b) Đúng — 100,04 và 30,04 đều hợp lệ với ô tô (GH-39, 1.175.000 đ).</li><li>c) Sai — 30,0 kg &gt; 20 kg với xe máy ⇒ <code>VEHICLE_TOO_SMALL</code>.</li><li>d) Sai — 100,05 km thành 100,1 km ⇒ ca âm.</li></ul>
</details>

**Câu 4 (K3).** Bảng quyết định rút gọn ở mục 4.4 có cột R3 (F0 = T, F1 = T, F2 = F, F3 = –, F4 = –). Cách chọn giá trị nào cho ca âm của cột R3 đúng single fault?

- a) Quãng đường 150 km, khối lượng 40 kg
- b) Quãng đường 150 km, khối lượng 3 kg, ô tô
- c) Quãng đường 150 km, xe máy, 25 kg
- d) Cả ba đều đúng vì «–» nghĩa là không ảnh hưởng

<details><summary><b>Đáp án: b</b></summary>
<ul><li>a), c) Sai — kết quả vẫn đúng <code>DISTANCE_OUT_OF_RANGE</code>, nhưng ca chứa thêm vi phạm bị che.</li><li>b) Đúng — mọi «–» chọn giá trị hợp lệ.</li><li>d) Sai — «–» không ảnh hưởng <b>kết quả</b> theo đặc tả, nhưng ảnh hưởng việc ca có che lỗi hay không.</li></ul>
</details>

**Câu 5 (K2).** Một bộ test đạt 100% decision table coverage cho bảng «lỗi đầu tiên» của mục 4.4. Phát biểu nào đúng?

- a) Bộ test chắc chắn đạt 100% EP coverage
- b) Bộ test chắc chắn bắt được lỗi quên cận dưới quãng đường
- c) Bộ test có thể bỏ lọt lỗi quên cận dưới, tuỳ giá trị chọn cho cột R3
- d) Bộ test chắc chắn đạt 100% BVA coverage

<details><summary><b>Đáp án: c</b></summary>
<ul><li>a) Sai — bộ DT của lab chỉ đạt 10/20 phân vùng.</li><li>b), d) Sai — bộ DT của lab chọn 150 km cho R3, đạt 0/12 BVA và bỏ lọt lỗi Q.</li><li>c) Đúng — mỗi kỹ thuật chỉ bảo đảm coverage item của chính nó.</li></ul>
</details>

**Câu 6 (K2).** Vì sao bài không dùng state transition testing cho kiểm tra yêu cầu báo giá?

- a) CTFL v4.0.1 không có state transition testing
- b) Kiểm tra một yêu cầu báo giá không phụ thuộc yêu cầu trước — không có trạng thái để mô hình hoá
- c) State transition testing chỉ dùng cho giao diện người dùng
- d) Báo giá không có thời hạn

<details><summary><b>Đáp án: b</b></summary>
<ul><li>a) Sai — CTFL mục 4.2.4.</li><li>b) Đúng — trạng thái chỉ xuất hiện sau khi báo giá được lưu (còn hạn, hết hạn, đã dùng).</li><li>c) Sai — dùng cho mọi hệ có trạng thái.</li><li>d) Sai — R-QUOTE-01 có thời hạn; nhưng đó là tính năng đặt chuyến từ báo giá, không phải kiểm tra yêu cầu.</li></ul>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 4.1 Test Techniques Overview; 4.2.1 Equivalence Partitioning (FL-4.2.1, K3); 4.2.2 Boundary Value Analysis (FL-4.2.2, K3); 4.2.3 Decision Table Testing (FL-4.2.3, K3); ghi chú phát hành v4.0.1 về kiểm riêng phân vùng không hợp lệ — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level Test Analyst Syllabus* v4.0 — bối cảnh về kỹ thuật dựa trên dữ liệu và dựa trên quy tắc: https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (mục *coverage item*, *defect masking*, *negative testing*).
- ISO/IEC/IEEE 29119-4, *Software and systems engineering — Software testing — Part 4: Test techniques*.
- P. C. Jorgensen, B. DeVries, *Software Testing: A Craftsman's Approach*, 5th ed., CRC Press, 2021 — giả định single fault.
- P. Ammann, J. Offutt, *Introduction to Software Testing*, 2nd ed., 2016 — tiêu chí Each Choice.
- Lab Tới Nơi: `SPEC.md` (R-VALID-01, R-API-02, R-API-07, R-GEN-03/04, R-FARE-01/02/07/08, R-PARCEL-01/02/03) và `articles/test-case-design-phoi-hop-ky-thuat/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-031 · EP](/articles/equivalence-partitioning) · [TEST-032 · BVA](/articles/boundary-value-analysis) · [TEST-033 · Decision Table](/articles/decision-table-testing) · [TEST-034 · State Transition](/articles/state-transition-testing-co-ban) · [TCD-04 · Kết quả mong đợi và test oracle](/articles/test-case-design-expected-result-test-oracle) · [TCD-05 · Test data](/articles/test-case-design-test-data)

Bài tiếp theo: [TCD-07 · Test case từ user story và acceptance criteria](/articles/test-case-design-user-story-acceptance-criteria).
