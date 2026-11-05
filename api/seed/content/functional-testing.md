---
slug: functional-testing
title: "Functional Testing: hệ thống có làm đúng, làm đủ, làm hợp không?"
summary: "Functional testing là một test type, không phải test level. Bài tách ba câu hỏi correctness, completeness, appropriateness theo CTFL v4.0.1 và chạy cùng một bộ test chức năng ở mức component lẫn API trên lab Tới Nơi."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-017, functional-testing, test-type, iso-25010, toi-noi]
language: vi
---

«Functional test là việc của tester, làm ở giai đoạn system test, bấm trên giao diện» — câu này sai ở cả ba vế. Functional testing là một **test type** (loại kiểm thử): nó được định nghĩa bằng *câu hỏi về chất lượng* mà nó trả lời, không phải bằng người làm, thời điểm hay công cụ. Bài này trả lời hai câu: functional testing hỏi chính xác những gì, và vì sao cùng một câu hỏi chức năng có thể được kiểm ở hàm thuần lẫn qua API của Tới Nơi.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-017</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S02 · Test Levels &amp; Test Types</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1 (đọc thêm CTAL-TA v4.0)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Component; System</td></tr>
<tr><th scope="row">Test Type</th><td>Functional</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/tong-quan-5-test-level-istqb">TEST-011 · Tổng quan 5 test level</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab <code>articles/functional-testing</code> (20 ca vitest): functional correctness của phụ phí bưu kiện R-PARCEL-03 (5 · 5,1 · 6,0 · 6,1 · 30 kg) ở mức component và qua API + kiểm tra completeness bảng chi tiết R-FARE-10</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 2 · mục 2.2 và 2.2.2 · FL-2.2.2 (K2). Đọc thêm: CTAL-TA v4.0 · Chương 4 · mục 4.1 · TA-4.1.1 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Bài [TEST-011](/articles/tong-quan-5-test-level-istqb) chia hoạt động kiểm thử theo **test level** — kiểm *cái gì* (hàm, giao tiếp, cả hệ thống, hệ thống với bên ngoài) và *ai, khi nào*. Bài này mở trục thứ hai: **test type** — kiểm *khía cạnh chất lượng nào*. Hai trục vuông góc nhau. Một ca test luôn nằm ở một ô của lưới: «functional × component», «functional × system», «non-functional × system»…

Functional testing trả lời câu hỏi **«hệ thống làm gì?»**: báo giá bưu kiện 6,1 kg có cộng 4.000 đ phụ phí không, bảng chi tiết cước có đủ từng bước không, tính năng có thật sự giúp khách đặt được đơn không. Nó **không** hỏi «nhanh không», «có an toàn không» — đó là non-functional testing ([TEST-018](/articles/non-functional-testing-tong-quan)).

Vấn đề nó giải quyết: không gọi tên được câu hỏi thì đội có hàng trăm ca test «chức năng» mà không biết chúng phủ **đúng**, **đủ** hay **hợp** — và thường chỉ phủ «đúng». Khi nào dùng: ở mọi test level, cho mọi chức năng trong test basis; câu hỏi chỉ là làm ở level nào cho rẻ nhất.

## 2. Lý thuyết theo ISTQB

### 2.1. Test type trong CTFL v4.0.1

Mục 2.2 định nghĩa test type là các nhóm hoạt động kiểm thử gắn với **những đặc tính chất lượng cụ thể**, và phần lớn các hoạt động đó làm được ở **mọi test level**. Mục 2.2.2 (FL-2.2.2, K2) nêu bốn test type:

| Test type | Câu hỏi | Tests suy ra từ đâu |
|---|---|---|
| Functional testing | Hệ thống làm **gì**? Làm có đúng, đủ, hợp không? | Chức năng được mô tả (yêu cầu, user story, use case…) |
| Non-functional testing | Hệ thống **hoạt động tốt tới mức nào**? | Đặc tính chất lượng ngoài chức năng (ISO/IEC 25010) |
| Black-box testing | Hành vi có khớp **đặc tả** không? | Tài liệu không liên quan tới cấu trúc bên trong |
| White-box testing | Cấu trúc bên trong đã được **phủ** đủ chưa? | Code, kiến trúc, luồng công việc, luồng dữ liệu |

Cả bốn áp được cho mọi test level, chỉ khác trọng tâm; mọi test technique đều có thể dùng để suy ra test condition cho cả bốn. Lưu ý bốn test type này **không loại trừ nhau**: hai dòng đầu phân theo *đặc tính chất lượng*, hai dòng sau phân theo *nguồn suy ra test*. Một ca test vừa functional vừa white-box là chuyện bình thường (mục 2.3).

### 2.2. Ba câu hỏi của functional testing

Syllabus nói mục tiêu chính của functional testing là kiểm **functional completeness**, **functional correctness** và **functional appropriateness**. Đây là ba đặc tính con của *functional suitability* trong mô hình chất lượng sản phẩm ISO/IEC 25010. CTFL chỉ liệt kê; CTAL-TA v4.0 mục 4.1 (TA-4.1.1, K2) giải thích sự khác nhau — bảng dưới diễn đạt lại:

| Đặc tính con | Câu hỏi chính | Cần gì để trả lời | Bắt đầu sớm nhất | Kỹ thuật hợp (theo CTAL-TA) |
|---|---|---|---|---|
| **Completeness** (đầy đủ) | Mọi thứ được yêu cầu đã được làm chưa? | Truy vết test basis ↔ test condition ↔ test case | Review đặc tả, thảo luận user story và acceptance criteria | Kỹ thuật theo hành vi (scenario-based), các kỹ thuật black-box khác |
| **Correctness** (đúng) | Kết quả thực tế có đúng — chính xác, đủ độ chính xác, nhất quán — với cả đầu vào hợp lệ lẫn không hợp lệ? | Một **test oracle** cho kết quả mong đợi chi tiết | Component testing, component integration testing | Mọi kỹ thuật black-box, experience-based, collaboration-based |
| **Appropriateness** (phù hợp) | Những gì đã làm có thật sự giúp người dùng hoàn thành nhiệm vụ? | Người dùng (hoặc đại diện), nhiệm vụ thật | Review thiết kế giao diện; demo; system và acceptance testing | Exploratory, collaboration-based, kỹ thuật theo hành vi |

Ba câu hỏi bổ sung cho nhau và không thay được nhau: một bảng chi tiết cước có thể **đúng** từng con số nhưng **thiếu** một bước; có thể đủ và đúng nhưng khách vẫn **không dùng được** để hiểu vì sao bị tính tiền.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Một chức năng phụ phí bưu kiện được kiểm bằng functional testing ở bốn test level: component, component integration, system và acceptance. Mỗi level có một ví dụ ca test và đặc tính con trọng tâm.">
<defs>
<marker id="t017-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="460" y="24" text-anchor="middle" class="t-b">Functional testing = một test type, áp ở mọi test level (CTFL v4.0.1 mục 2.2.2)</text>
<text x="115" y="50" text-anchor="middle" class="t-small t-b">Test level</text>
<text x="450" y="50" text-anchor="middle" class="t-small t-b">Ví dụ ca test chức năng — phụ phí bưu kiện</text>
<text x="795" y="50" text-anchor="middle" class="t-small t-b">Đặc tính con trọng tâm</text>
<rect x="20" y="62" width="190" height="46" rx="8" class="f-pk-soft s-pk"/>
<text x="115" y="90" text-anchor="middle" class="t-b t-pk">Component</text>
<rect x="230" y="62" width="440" height="46" rx="8" class="f-surface s-line"/>
<text x="450" y="90" text-anchor="middle" class="t-small t-mono">parcelSurcharge(6.1) === 4000</text>
<rect x="690" y="62" width="210" height="46" rx="8" class="f-surface s-line"/>
<text x="795" y="90" text-anchor="middle" class="t-small">correctness</text>
<rect x="20" y="120" width="190" height="46" rx="8" class="f-pk-soft s-pk"/>
<text x="115" y="148" text-anchor="middle" class="t-b t-pk">Component integration</text>
<rect x="230" y="120" width="440" height="46" rx="8" class="f-surface s-line"/>
<text x="450" y="148" text-anchor="middle" class="t-small">pricing.ts cộng phụ phí vào cước cơ bản TRƯỚC bước cao điểm</text>
<rect x="690" y="120" width="210" height="46" rx="8" class="f-surface s-line"/>
<text x="795" y="148" text-anchor="middle" class="t-small">correctness</text>
<rect x="20" y="178" width="190" height="46" rx="8" class="f-phys-soft s-phys"/>
<text x="115" y="206" text-anchor="middle" class="t-b t-phys">System (qua API)</text>
<rect x="230" y="178" width="440" height="46" rx="8" class="f-surface s-line"/>
<text x="450" y="206" text-anchor="middle" class="t-small">POST /api/quotes ⇒ fare đủ 11 bước, total = 86.000 đ</text>
<rect x="690" y="178" width="210" height="46" rx="8" class="f-surface s-line"/>
<text x="795" y="206" text-anchor="middle" class="t-small">correctness + completeness</text>
<rect x="20" y="236" width="190" height="46" rx="8" class="f-warn-soft s-warn"/>
<text x="115" y="264" text-anchor="middle" class="t-b t-warn">Acceptance</text>
<rect x="230" y="236" width="440" height="46" rx="8" class="f-surface s-line"/>
<text x="450" y="264" text-anchor="middle" class="t-small">Khách tự giải thích được vì sao phải trả 86.000 đ trước khi đặt?</text>
<rect x="690" y="236" width="210" height="46" rx="8" class="f-surface s-line"/>
<text x="795" y="264" text-anchor="middle" class="t-small">appropriateness</text>
<line x1="210" y1="85" x2="228" y2="85" class="s-muted" stroke-width="1.4" marker-end="url(#t017-a)"/>
<line x1="210" y1="143" x2="228" y2="143" class="s-muted" stroke-width="1.4" marker-end="url(#t017-a)"/>
<line x1="210" y1="201" x2="228" y2="201" class="s-muted" stroke-width="1.4" marker-end="url(#t017-a)"/>
<line x1="210" y1="259" x2="228" y2="259" class="s-muted" stroke-width="1.4" marker-end="url(#t017-a)"/>
</svg>
</div>
<figcaption><b>Hình 1 · Cùng một test type, bốn test level.</b> Câu hỏi «hệ thống làm gì» không đổi; thứ thay đổi là test object, test basis và đặc tính con dễ kiểm nhất ở level đó. Ví dụ 86.000 đ: ô tô 3 km, 6,1 kg, cao điểm, mưa (mục 5.3).</figcaption>
</figure>

### 2.3. Functional không đồng nghĩa với black-box

Người mới hay ghép «functional = black-box» và «non-functional = white-box». Bảng 2×2 dưới cho thấy cả bốn ô đều có thật ở Tới Nơi:

| | Black-box (từ đặc tả) | White-box (từ cấu trúc) |
|---|---|---|
| **Functional** | Chọn 5,0 · 5,1 · 6,0 · 6,1 · 30 kg từ ví dụ của R-PARCEL-03 | Đọc `parcelSurcharge`, thấy nhánh `extraTenths <= 0`; thêm ca 5,0 kg để phủ nhánh trả 0 |
| **Non-functional** | p95 của `POST /api/quotes` dưới 500 ms theo R-NFR-03 | Đọc code thấy báo giá không bao giờ bị dọn khỏi kho ⇒ thiết kế test chạy dài để đo bộ nhớ |

### 2.4. Bảng thuật ngữ Anh – Việt

| Thuật ngữ (ISTQB) | Tiếng Việt dùng trong bài | Ghi chú |
|---|---|---|
| test type | loại kiểm thử | nhóm hoạt động gắn với một đặc tính chất lượng |
| functional testing | kiểm thử chức năng | đánh giá chức năng test object phải thực hiện |
| functional suitability | sự phù hợp về chức năng | đặc tính chất lượng ISO/IEC 25010 |
| functional completeness | tính đầy đủ chức năng | làm đủ mọi thứ được yêu cầu |
| functional correctness | tính đúng chức năng | kết quả đúng, chính xác, nhất quán |
| functional appropriateness | tính phù hợp chức năng | giúp người dùng hoàn thành nhiệm vụ |
| test oracle | nguồn kết quả mong đợi | ở đây: ví dụ trong SPEC và phép tính tay |
| traceability | truy vết | cần để đo completeness |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Trong tin tuyển dụng và công cụ quản lý test, «functional testing» hay được dùng theo nghĩa hẹp: kiểm thử thủ công trên giao diện do QA làm. Đó là một cách tổ chức công việc, không phải định nghĩa của ISTQB. Theo CTFL, unit test do developer viết để kiểm <code>parcelSurcharge</code> cũng là functional testing — ở test level component.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Giao bưu kiện chiếm khoảng 0,3 trên 2 triệu chuyến mỗi ngày ([hồ sơ dự án](/articles/du-an-mau-toi-noi)). Trước khi đặt, khách nhận **báo giá** có thời hạn: tổng tiền kèm bảng chi tiết từng bước — CSKH dùng nó để giải thích giá, ứng dụng dùng nó để hiển thị.

### 3.2. Business rules dùng trong bài

- **R-PARCEL-01** — khối lượng làm tròn 0,1 kg half-up rồi phải thoả 0 < w ≤ 30 kg; sai ⇒ `WEIGHT_OUT_OF_RANGE`.
- **R-PARCEL-02** — w > 20 kg bắt buộc ô tô; chọn xe máy ⇒ `VEHICLE_TOO_SMALL`.
- **R-PARCEL-03** — phần vượt 5 kg tính 2.000 đ cho mỗi kg bắt đầu: 5,1 ⇒ 2.000 đ; 6,0 ⇒ 2.000 đ; 6,1 ⇒ 4.000 đ; 30 ⇒ 50.000 đ.
- **R-PARCEL-04** — phụ phí cộng vào cước cơ bản nên bị nhân hệ số cao điểm.
- **R-FARE-01, -03, -08** — bảng giá ô tô 25.000 đ + 11.000 đ/km, tối thiểu 30.000 đ; thứ tự 6 bước; làm tròn lên 1.000 đ ở bước cuối.
- **R-FARE-10** — báo giá trả kèm **11 bước**: phí mở cửa, tiền theo km, phụ phí bưu kiện, cước cơ bản, sau cao điểm, phụ phí mưa, trước giảm, tiền giảm, sau giảm, cước tối thiểu, tổng.
- **R-API-07** — `POST /api/quotes` trả `quote.fare` là bảng R-FARE-10, kể cả với đơn đồ ăn (bảng của phí giao).

### 3.3. System context

Trong kiến trúc đích, **Pricing & Quote Service** (kiến trúc §4.3) tạo báo giá có `quote_id`, `expires_at`, `pricing_version` và *breakdown*; ứng dụng khách gọi qua API Gateway bằng REST; Booking giữ ảnh chụp báo giá để giá không đổi hồi tố. Bản lab là modular monolith giai đoạn MVP (§18), cùng chuỗi đó nằm trong một tiến trình:

```text
src/domain/parcel.ts  (parcelSurcharge)          ← component
        │
src/domain/pricing.ts (priceRequest → calculateFare)  ← component integration
        │
src/app/quote.ts → src/api/app.ts  POST /api/quotes  ← system (qua API)
        │
src/web/index.html  «Báo giá»                    ← giao diện cho khách
```

### 3.4. Rủi ro có thể xảy ra

| Rủi ro | Hậu quả | Đặc tính con |
|---|---|---|
| Kg chưa trọn bị bỏ qua (6,1 kg tính như 6 kg) | Thu thiếu 2.000 đ mỗi đơn; với số lớn là doanh thu thất thoát | correctness |
| 6,0 kg bị tính thừa thành 2 kg (cài đặt «phần nguyên + 1» thay cho ⌈w − 5⌉) | Khách bị thu thừa ở mọi khối lượng nguyên trên 5 kg, khiếu nại | correctness |
| Bảng chi tiết của đơn đồ ăn thiếu bước «phụ phí mưa» | CSKH không giải thích được phí giao ngày mưa | completeness |
| Mọi bước đúng và đủ trong API nhưng ứng dụng chỉ hiện tổng tiền | Khách nghĩ bị «ăn gian» khi trời mưa | appropriateness |
| Đơn 30,05 kg lọt qua kiểm tra | Xe quá tải, giá sai | correctness (đầu vào sai) |

## 4. Phân tích kỹ thuật

### 4.1. Từ yêu cầu tới test condition theo ba câu hỏi

| Mã | Test condition | Đặc tính con | Level rẻ nhất để kiểm |
|---|---|---|---|
| TCO-01 | Phần vượt 5 kg chưa trọn kg được tính trọn một kg | correctness | component |
| TCO-02 | Phần vượt đúng số kg nguyên không bị tính thừa | correctness | component |
| TCO-03 | Phụ phí đi vào cước cơ bản, trước hệ số cao điểm | correctness | component integration |
| TCO-04 | Khối lượng ngoài khoảng bị từ chối đúng mã lỗi, đúng HTTP 422 | correctness | system (qua API) |
| TCO-05 | Các bước trong bảng chi tiết khớp nhau và khớp tổng | correctness (nhất quán) | system (qua API) |
| TCO-06 | Bảng chi tiết có đủ 11 bước cho cả `ride`, `parcel`, `food` | completeness | system (qua API) |
| TCO-07 | Khách nhìn báo giá hiểu được vì sao giá tăng khi có phụ phí | appropriateness | acceptance / phiên dùng thử |

Cách chọn giá trị biên (5,0 / 5,1 / 6,0 / 6,1 / 30) là việc của kỹ thuật thiết kế test — Equivalence Partitioning và Boundary Value Analysis ở Series 04. Ở đây ta lấy đúng các ví dụ SPEC cho sẵn để tập trung vào *loại* câu hỏi.

### 4.2. Cùng một câu hỏi, khác test level thì bắt được gì?

| | Component (`parcelSurcharge`) | System qua API (`POST /api/quotes`) |
|---|---|---|
| Bắt được | Sai công thức, sai cách đếm «kg bắt đầu», sai số dấu phẩy động | Như cột trái **cộng** sai lắp ráp: phụ phí không được truyền vào `calculateFare`, cộng sau bước cao điểm, kiểm tra đầu vào sai thứ tự, JSON đổi tên trường |
| Không bắt được | Hàm đúng nhưng không ai gọi nó | Lỗi nằm sâu khó định vị — test đỏ không chỉ ra dòng nào sai |
| Chi phí | Mili giây, không cần dựng hệ thống | Cần dựng ứng dụng (lab: trong bộ nhớ, vài mili giây mỗi ca) |

Vì thế lab chạy **cùng năm ca** ở cả hai level: nếu component xanh mà API đỏ, lỗi nằm ở phần lắp ráp chứ không ở công thức.

### 4.3. Oracle: tính tay trước khi chạy

Ô tô 3 km, SG, 09:00 (không cao điểm), không mưa, không voucher: cước cơ bản = 25.000 + 11.000 × 3 = **58.000 đ**. Cộng phụ phí, giữ nguyên qua bước 2–5 (đã ≥ 30.000 đ), làm tròn lên 1.000 đ:

| Khối lượng | Phụ phí (R-PARCEL-03) | Tổng mong đợi |
|---|---|---|
| 5,0 kg | 0 đ | 58.000 đ |
| 5,1 kg | 2.000 đ | 60.000 đ |
| 6,0 kg | 2.000 đ | 60.000 đ |
| 6,1 kg | 4.000 đ | 62.000 đ |
| 30 kg | 50.000 đ | 108.000 đ |

Kết quả mong đợi phải có **trước** khi nhìn kết quả thực tế. Chép đầu ra của hệ thống vào làm «mong đợi» là biến test thành bản ghi lại hành vi hiện tại — đúng hay sai đều xanh.

«Nhất quán» (consistent) cũng thuộc correctness: dù không có oracle cho từng ô, các bước phải khớp nhau — cước cơ bản = phí mở cửa + tiền km + phụ phí; trước giảm = sau cao điểm + phụ phí mưa; sau giảm = trước giảm − tiền giảm; tổng = làm tròn lên của max(sau giảm, cước tối thiểu).

### 4.4. Completeness cần truy vết, không cần oracle số

Kiểm completeness không hỏi «số có đúng không» mà hỏi «có **mặt** không». Muốn vậy phải có bảng nối từng bước trong đặc tả với một trường trong kết quả. Lab đọc thẳng danh sách 11 bước từ dòng R-FARE-10 của `SPEC.md`: nếu đặc tả thêm bước mới mà test chưa ánh xạ, test đỏ ngay — completeness của chính bộ test cũng được canh.

## 5. Thực hành

**Đề bài (curriculum TEST-017):** viết test case kiểm functional correctness cho phụ phí khối lượng R-PARCEL-03 (5 kg, 5,1 kg, 6,0 kg, 6,1 kg, 30 kg) và một kiểm tra completeness: mọi trường của bảng chi tiết R-FARE-10 đều có mặt. Tự làm trước, rồi so với lời giải dưới.

### 5.1. Test case functional correctness

| Mã | Level | Đầu vào | Kết quả mong đợi | Truy vết |
|---|---|---|---|---|
| FC-01…05 | component | `parcelSurcharge(w)`, w = 5 · 5,1 · 6,0 · 6,1 · 30 | 0 · 2.000 · 2.000 · 4.000 · 50.000 | R-PARCEL-03 · TCO-01, 02 |
| FC-01…05 | system (API) | `POST /api/quotes` parcel, ô tô, 3 km, SG, 09:00, cùng 5 khối lượng | `fare.parcelSurcharge` như trên; `total` 58.000 · 60.000 · 60.000 · 62.000 · 108.000 | R-PARCEL-03, R-FARE-01/08 · TCO-01, 02 |
| FC-06 | system (API) | 30,05 kg (làm tròn thành 30,1) | 422, `WEIGHT_OUT_OF_RANGE` | R-PARCEL-01 · TCO-04 |
| FC-07 | system (API) | 20,1 kg, xe máy | 422, `VEHICLE_TOO_SMALL` | R-PARCEL-02 · TCO-04 |
| FC-08…10 | system (API) | 11:30, SG mưa: chuyến R-FARE-09 · bưu kiện ô tô 3 km 6,1 kg · đồ ăn 2,5 km tiền món 50.000 | 4 đẳng thức mục 4.3 đúng; tổng 52.000 · 86.000 · 88.000 đ | R-FARE-03/08/10, R-PARCEL-04, R-FOOD-04 · TCO-03, 05 |

Phép tính của FC-09: (58.000 + 4.000) × 1,25 = 77.500 → + 8.000 = 85.500 → làm tròn lên **86.000 đ** — phụ phí bị nhân 1,25 đúng R-PARCEL-04. FC-10: phí giao xe máy (12.000 + 4.500 × 2,5) × 1,25 + 8.000 = 37.062,5 → 38.000 đ; cộng 50.000 đ tiền món = **88.000 đ**.

### 5.2. Kiểm tra functional completeness

| Mã | Kiểm tra | Mong đợi |
|---|---|---|
| FCP-01 | Bảng ánh xạ của test phủ đúng danh sách bước đọc từ SPEC R-FARE-10 | Đúng 11 tên, không thiếu, không thừa |
| FCP-02…04 | `quote.fare` của `ride`, `parcel`, `food` có đủ 11 khoá, mỗi khoá là số hữu hạn | Danh sách bước thiếu = rỗng |
| FCP-05 | Chạy hàm kiểm trên một bảng chi tiết **giả định** thiếu bước (`fare-ban-cu.json`) | Báo thiếu «phụ phí mưa», «cước tối thiểu» |

FCP-05 không kiểm sản phẩm mà kiểm **chính bộ kiểm tra**: một hàm completeness luôn trả rỗng thì vô dụng; cần thấy nó đỏ được.

### 5.3. Code và lần chạy thật

Lõi của kiểm tra completeness (`articles/functional-testing/fare-fields.ts`); `FARE_FIELD_BY_LABEL` nối 11 tên bước của SPEC với khoá JSON (`'phí mở cửa' → 'openingFee'`, …, `'tổng' → 'total'`):

```ts

export function missingFareFields(fare: Record<string, unknown>): string[] {
  return Object.entries(FARE_FIELD_BY_LABEL)
    .filter(([, key]) => typeof fare[key] !== 'number' || !Number.isFinite(fare[key] as number))
    .map(([label]) => label);
}
```

Cùng bảng `ORACLE` (5 dòng của mục 4.3) chạy ở hai level; ở mức component chỉ là `expect(parcelSurcharge(weightKg)).toBe(surcharge)`, qua API thì (`functional-correctness.test.ts`, rút gọn):

```ts
describe('Mức system qua API · POST /api/quotes, ô tô 3 km, SG, 09:00, không mưa', () => {
  it.each(ORACLE)('$id: $weightKg kg ⇒ fare.parcelSurcharge $surcharge, total $total', async (row) => {
    const res = await request(app).post('/api/quotes').set('x-user', 'customer:C001')
      .send({ kind: 'parcel', cityCode: 'SG', vehicle: 'car', distanceKm: 3, parcelWeightKg: row.weightKg })
      .expect(200);
    expect(res.body.quote.fare.parcelSurcharge).toBe(row.surcharge);
    expect(res.body.quote.total).toBe(row.total);
  });
});
```

Chạy trong thư mục lab:

```bash
npx vitest run articles/functional-testing
```

Kết quả thực tế (vitest 3.2.7, Node 22, lần chạy khi viết bài):

```text
 ✓ articles/functional-testing/functional-completeness.test.ts (5 tests)
 ✓ articles/functional-testing/functional-correctness.test.ts (15 tests)

 Test Files  2 passed (2)
      Tests  20 passed (20)
```

### 5.4. Đọc kết quả cho đúng

- 20/20 xanh nói rằng, **với các ví dụ đã chọn**, phụ phí bưu kiện đúng ở cả hai level và bảng chi tiết đủ 11 bước. Nó không chứng minh mọi khối lượng đều đúng (nguyên lý «exhaustive testing is impossible», [TEST-003](/articles/bay-nguyen-ly-kiem-thu)).
- Không có ca nào trả lời TCO-07 (appropriateness). Câu đó cần người dùng thật — xem phiếu phát hiện validation ở [TEST-005](/articles/verification-va-validation).
- Muốn thấy test **đỏ** trên bản cài đặt sai (6,0 kg bị tính 4.000 đ), xem bản lỗi và test log ở [TEST-010](/articles/testware-san-pham-kiem-thu).

## 6. Góc nhìn nâng cao

**Completeness luôn tương đối với một test basis.** API trả đủ 11 bước, nên «đủ» so với R-FARE-10. Nhưng giao diện `/` chỉ hiện tổng tiền, cờ cao điểm/mưa, tiền giảm — vẫn «đủ» so với R-UI-01, vì R-UI-01 không đòi hiện bảng chi tiết. Không test nào đỏ, nhưng khách không xem được breakdown. Đây là lỗ hổng của **test basis**, phát hiện tốt nhất bằng review đặc tả (Series 03), không phải bằng thêm test.

**Đầu vào không hợp lệ là một nửa của correctness.** CTAL-TA nói rõ «cho cả đầu vào hợp lệ và không hợp lệ». Bộ test chỉ có đường vui (happy path) bỏ qua FC-06, FC-07 — đúng loại lỗi làm xe quá tải.

**Đúng tới mức nào?** Correctness gồm cả *precision*. Bảng chi tiết có giá trị lẻ như 37.062,5 đ (R-GEN-01 không làm tròn giữa chừng). So sánh bằng `toBe` trên số thực dễ đỏ giả vì sai số dấu phẩy động; lab dùng `toBeCloseTo(…, 6)` cho đẳng thức trung gian và `toBe` cho tổng (luôn là số nguyên đồng).

**Đếm ca test không đo được completeness.** «Mỗi yêu cầu có ít nhất 3 test case» có thể là 3 ca cùng kiểm một trường. Completeness đo bằng truy vết: bước nào chưa có test condition.

**Functional là nền của non-functional.** Chạy FC-02 hàng nghìn lần và đo thời gian là đã bắt đầu kiểm time behaviour — bài tiếp theo đi theo hướng đó.

**Sai lầm thường gặp:**

| Sai lầm | Vì sao sai |
|---|---|
| «Functional testing = system testing» | Test type ≠ test level; functional testing có ở cả bốn level trong Hình 1 |
| «Functional testing = black-box» | Có thể suy ra test chức năng từ cấu trúc code (mục 2.3) |
| Chỉ kiểm correctness | Thiếu completeness thì không biết còn chức năng nào chưa làm; thiếu appropriateness thì làm đủ, đúng mà không ai dùng được |
| Lấy đầu ra hiện tại làm kết quả mong đợi | Không còn oracle; test xanh cả khi hệ thống sai |
| Dồn mọi test chức năng lên giao diện | Chậm, dễ vỡ; correctness nên dời trái về component (CTAL-TA 4.1) |

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Ba câu hỏi cho phí giao đồ ăn

Với R-FOOD-04 (phí giao theo công thức xe máy, có cao điểm và mưa, tối thiểu 15.000 đ, voucher không trừ vào phí giao), viết **một** test condition cho mỗi đặc tính con: correctness, completeness, appropriateness. Với mỗi condition, ghi level rẻ nhất để kiểm.

<details><summary><b>Lời giải mẫu</b></summary><ul><li><b>Correctness</b> — đơn 1,0 km ngoài cao điểm, không mưa: 12.000 + 4.500 = 16.500 ⇒ 17.000 đ; đơn 0,5 km: 14.250 ⇒ nâng lên tối thiểu 15.000 đ; đơn có voucher FOOD15: tiền giảm chỉ trừ vào tiền món, <code>fare.discount</code> = 0. Level: component (<code>priceFood</code>), một ca qua API để kiểm lắp ráp.</li><li><b>Completeness</b> — báo giá <code>food</code> trả <code>fare</code> đủ 11 bước và đủ ba trường <code>itemsSubtotal</code>, <code>itemsDiscount</code>, <code>deliveryFee</code> (R-API-07). Level: system qua API.</li><li><b>Appropriateness</b> — khách có phân biệt được «tiền món được giảm» với «phí giao không được giảm» khi xem báo giá không? Level: acceptance / phiên dùng thử với khách thật.</li></ul></details>

### Bài tập 2 — Phân loại

Xếp 6 kiểm tra vào correctness (Co), completeness (Cm) hay appropriateness (Ap), kèm test level: (1) `normalizeWeightKg(0.04)` ném `WEIGHT_OUT_OF_RANGE`; (2) đối chiếu danh sách mã lỗi R-API-06 với mã lỗi API thật trả về để xem mã nào chưa bao giờ xuất hiện; (3) cho 5 tài xế dùng `/driver.html` và đếm số người tự tìm được nút «Đang tới đón»; (4) `POST /api/quotes` với mã thành phố `NT` trả 200; (5) review user story «xem lịch sử chuyến» cùng PO để xem acceptance criteria có thiếu trường hợp chuyến bị huỷ không; (6) chuyến R-FARE-09 qua API ra 52.000 đ.

<details><summary><b>Lời giải</b></summary><ul><li>(1) Co — đầu vào không hợp lệ; component.</li><li>(2) Cm — có mã lỗi được đặc tả mà không có đường nào sinh ra; system qua API (kết hợp truy vết).</li><li>(3) Ap — nhiệm vụ thật, người dùng thật; acceptance.</li><li>(4) Co — Nha Trang thuộc 6 thành phố R-GEN-03; system qua API.</li><li>(5) Cm, làm bằng static testing (review) — completeness nên xử lý sớm nhất có thể.</li><li>(6) Co — oracle là ví dụ R-FARE-09; system qua API.</li></ul></details>

### Câu hỏi trắc nghiệm

**Câu 1 (K1).** Theo CTFL v4.0.1, mục tiêu chính của functional testing là kiểm tra điều gì?

- a) Thời gian phản hồi và mức dùng tài nguyên
- b) Functional completeness, functional correctness và functional appropriateness
- c) Mức độ phủ cấu trúc code
- d) Khả năng chịu tải và phục hồi sau sự cố

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — đó là performance efficiency, thuộc non-functional testing.</li><li>b) Đúng — mục 2.2.2.</li><li>c) Sai — đó là mục tiêu của white-box testing.</li><li>d) Sai — đó là non-functional (performance efficiency, reliability).</li></ul></details>

**Câu 2 (K2).** Phát biểu nào đúng về quan hệ giữa test type và test level?

- a) Functional testing chỉ thực hiện ở system testing
- b) Mỗi test level chỉ được dùng một test type
- c) Phần lớn hoạt động của một test type có thể làm ở mọi test level, với trọng tâm khác nhau
- d) Test type là tên khác của test level trong mô hình Agile

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Sai — functional testing có ở mọi level, kể cả component.</li><li>b) Sai — một level thường kết hợp nhiều test type.</li><li>c) Đúng — mục 2.2 và 2.2.2.</li><li>d) Sai — hai khái niệm độc lập, không phụ thuộc mô hình phát triển.</li></ul></details>

**Câu 3 (K2).** Báo giá trả đủ 11 bước của R-FARE-10, nhưng bước «sau cao điểm» của chuyến cao điểm bằng đúng cước cơ bản (không nhân 1,25). Đây là vấn đề của đặc tính con nào?

- a) Functional completeness
- b) Functional correctness
- c) Functional appropriateness
- d) Không phải vấn đề chức năng

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — đủ 11 bước, không thiếu gì.</li><li>b) Đúng — kết quả có mặt nhưng sai giá trị.</li><li>c) Sai — vấn đề không nằm ở việc người dùng có hoàn thành nhiệm vụ hay không.</li><li>d) Sai — hệ số cao điểm là quy tắc chức năng R-FARE-04.</li></ul></details>

**Câu 4 (K2).** Ca test nào kiểm functional appropriateness?

- a) Unit test khẳng định 6,1 kg có phụ phí 4.000 đ
- b) Đối chiếu từng bước trong R-FARE-10 với trường JSON trả về
- c) Quan sát khách thật xem họ có hiểu vì sao giá ngày mưa cao hơn trước khi bấm «Đặt chuyến»
- d) Gửi 30,05 kg và chờ lỗi `WEIGHT_OUT_OF_RANGE`

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Sai — correctness, có oracle từ SPEC.</li><li>b) Sai — completeness.</li><li>c) Đúng — chức năng có giúp người dùng hoàn thành nhiệm vụ không.</li><li>d) Sai — correctness với đầu vào không hợp lệ.</li></ul></details>

**Câu 5 (K2).** Tester đọc mã `parcelSurcharge`, thấy nhánh trả 0 khi không vượt 5 kg chưa có test, và thêm ca 5,0 kg. Ca test này thuộc loại nào?

- a) Chỉ functional
- b) Chỉ white-box
- c) Vừa functional vừa white-box
- d) Non-functional, vì xuất phát từ code

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Sai — bỏ qua việc ca được suy ra từ cấu trúc code.</li><li>b) Sai — ca vẫn kiểm một chức năng (phụ phí) so với kết quả mong đợi.</li><li>c) Đúng — hai cách phân loại là hai trục khác nhau, không loại trừ nhau.</li><li>d) Sai — xuất phát từ code không làm ca test thành non-functional.</li></ul></details>

**Câu 6 (K2).** Theo CTAL-TA v4.0, để xác định mức functional completeness đã đạt, điều gì là thiết yếu?

- a) Môi trường kiểm thử giống production
- b) Truy vết giữa test basis, test condition và test case
- c) Công cụ tự động hoá giao diện
- d) Đo độ phủ câu lệnh

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — hữu ích cho system testing nhưng không đo được «đã làm đủ chưa».</li><li>b) Đúng — CTAL-TA mục 4.1; lab dùng bảng ánh xạ SPEC ↔ JSON.</li><li>c) Sai — công cụ không trả lời câu hỏi thiếu hay đủ.</li><li>d) Sai — statement coverage đo phủ code, không đo phủ yêu cầu.</li></ul></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 2.2 và 2.2.2 (FL-2.2.2) — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level Test Analyst Syllabus* v4.0 (2025): chương 4, mục 4.1 (TA-4.1.1) và Phụ lục F — https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (mục *functional testing*, *functional suitability*, *test type*).
- ISO/IEC 25010:2023, *Systems and software engineering — SQuaRE — Product quality model* (đặc tính functional suitability).
- Lab Tới Nơi: `SPEC.md` (R-PARCEL, R-FARE, R-FOOD-04, R-API-07) và `articles/functional-testing/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-011 Tổng quan 5 test level](/articles/tong-quan-5-test-level-istqb) · [TEST-012 Component Testing](/articles/component-testing-unit-testing-tong-quan) · [TEST-014 System Testing](/articles/system-testing) · [TEST-005 Verification và Validation](/articles/verification-va-validation) · [TEST-010 Testware](/articles/testware-san-pham-kiem-thu) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)

Bài tiếp theo: [TEST-018 · Non-functional Testing: tổng quan theo ISO/IEC 25010](/articles/non-functional-testing-tong-quan)
