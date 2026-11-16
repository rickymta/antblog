---
slug: test-case-design-logic-va-cu-the
title: "Test case logic và test case cụ thể: từ thiết kế tới test procedure, test suite và lịch chạy"
summary: "Viết 12 test case logic cho đơn đồ ăn Tới Nơi (R-FOOD), mở rộng bằng bảng test data thành 19 test case cụ thể có kết quả mong đợi tính từ SPEC, ghép thành test procedure có setup/teardown, test suite và test execution schedule; vì sao phải giảm phụ thuộc giữa test case. Lab vitest 34 ca."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-case-design, tcd-03, logical-test-case, concrete-test-case, test-procedure, test-suite, test-execution-schedule, test-data, vitest, toi-noi]
language: vi
---

Hai tester cùng nhận dòng «Kiểm tra phí giao giờ cao điểm cho đơn đồ ăn». Người thứ nhất đặt một đơn 3 km lúc 11:30, thấy phí giao 34.000 đ, «có vẻ hợp lý», đánh dấu đạt. Người thứ hai đặt đơn 3,3 km lúc 11:30, tính tay ra 33.562,5 đ, chờ hệ thống làm tròn lên 34.000 đ, so khớp, đánh dấu đạt. Cùng một test case, hai lần chạy, hai mức bằng chứng khác hẳn nhau. Dòng chữ kia là một **test case logic**: nó nói kiểm *cái gì* nhưng chưa nói kiểm *bằng giá trị nào* và *mong đợi con số nào*. Bài này dạy khi nào để test case ở mức logic là đủ, khi nào phải đi tới **test case cụ thể**, cách mở rộng có hệ thống bằng bảng test data, rồi ghép test case thành **test procedure**, **test suite** và **test execution schedule** — kèm bài học đắt giá nhất của phần test implementation: phụ thuộc giữa test case.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TCD-03</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>TCD · Thiết kế Test Case</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>System (qua API)</td></tr>
<tr><th scope="row">Test Type</th><td>Functional</td></tr>
<tr><th scope="row">Test Technique</th><td>Không phải một kỹ thuật riêng — dùng kết quả của EP, BVA (S04) để chọn dữ liệu</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/test-case-design-giai-phau-test-case">TCD-02 · Giải phẫu test case</a> · <a href="/articles/testware-san-pham-kiem-thu">TEST-010 · Testware</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): 12 test case logic cho đơn đồ ăn viết thành dữ liệu, hàm mở rộng thành 19 test case cụ thể chạy qua <code>POST /api/quotes</code>, runner test procedure có setup/teardown, test suite và lập lịch theo phụ thuộc — <code>examples/toi-noi-testing/articles/test-case-design-logic-va-cu-the/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.4.1 (test design, test implementation), 1.4.3 · FL-1.4.1 (K2), FL-1.4.3 (K2); ISTQB Glossary (high-level / low-level test case, test procedure, test suite, test execution schedule)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

[TCD-01](/articles/test-case-design-test-analysis-test-condition) dừng ở test condition — «cái cần kiểm». [TCD-02](/articles/test-case-design-giai-phau-test-case) mổ xẻ từng trường của một test case. Giữa hai bài đó và lúc bấm nút chạy còn một quãng mà nhiều đội đi tắt: **mức chi tiết** của test case, và cách **sắp xếp** test case để chạy.

Đi tắt theo hai hướng đều đắt. Một đội viết mọi thứ ở mức cụ thể ngay từ đầu: hàng trăm test case với giá trị nhập và con số mong đợi, để rồi khi bảng giá đổi từ 4.500 đ lên 4.800 đ mỗi km thì phải sửa tay từng dòng. Đội khác giữ mọi thứ ở mức logic vì «tester có kinh nghiệm sẽ tự biết chọn dữ liệu» — và rồi không ai trả lời được câu hỏi kiểm toán «lần chạy ngày 12 dùng dữ liệu gì, mong đợi bao nhiêu». Cách làm bền là tách hai lớp: test case logic giữ **ý đồ** (ổn định, ít đổi), bảng test data giữ **giá trị** (đổi theo dữ liệu, theo phiên bản giá), và một bước mở rộng có quy tắc nối hai lớp lại.

Bài này dùng tính năng **giao đồ ăn** của Tới Nơi (R-FOOD) vì nó gói đủ các loại quy tắc: kiểm định dạng (dòng món), ngưỡng (tiền món tối thiểu), công thức tiền có làm tròn (phí giao theo công thức xe máy), quy tắc theo thời điểm (cao điểm, mưa), và tương tác với voucher. Phạm vi của bài là **từ test case tới lịch chạy** — chọn giá trị biên bằng BVA, phân vùng bằng EP thuộc S04; viết test data cho từng ca một cách kỹ lưỡng là [TCD-05](/articles/test-case-design-test-data); viết kết quả mong đợi kiểm chứng được là [TCD-04](/articles/test-case-design-expected-result-test-oracle).

## 2. Lý thuyết theo ISTQB

### 2.1. Test design và test implementation trong CTFL v4.0.1 mục 1.4.1

CTFL chia quy trình kiểm thử thành các nhóm hoạt động; hai nhóm liên quan trực tiếp tới bài này là:

| Hoạt động | Câu hỏi | Công việc chính theo mục 1.4.1 (diễn đạt lại) |
|---|---|---|
| **Test design** | «kiểm như thế nào?» | triển khai test condition thành test case và testware khác (ví dụ test charter); xác định coverage item làm cơ sở cho đầu vào; xác định yêu cầu về test data; thiết kế môi trường test, xác định hạ tầng và công cụ cần thiết |
| **Test implementation** | «đã có đủ mọi thứ để chạy chưa?» | tạo hoặc thu thập testware cần cho test execution (ví dụ test data); sắp test case thành test procedure, thường gom thành test suite; viết test script thủ công và tự động; ưu tiên và sắp test procedure vào test execution schedule để chạy hiệu quả; dựng và kiểm tra môi trường test |

Đọc kỹ bảng sẽ thấy ranh giới tự nhiên của bài: **test design** sinh ra test case và *yêu cầu* về test data — nghĩa là thiết kế có thể dừng ở chỗ «cần một đơn có tiền món đúng bằng mức tối thiểu». **Test implementation** mới *tạo* dữ liệu thật («hai ổ bánh mì 15.000 đ»), viết bước chạy, gom thành procedure và suite, rồi lập lịch. Hai hoạt động này có thể diễn ra gần như cùng lúc (trong một test tự động chúng là vài dòng code liền nhau), nhưng tách chúng ra trong đầu giúp quyết định cái gì nên ổn định và cái gì được phép đổi.

### 2.2. Test case logic và test case cụ thể

ISTQB Glossary định nghĩa hai mức chi tiết của test case (diễn đạt lại):

- **Test case logic** (*logical test case*, còn gọi *high-level test case*): tiền điều kiện, dữ liệu đầu vào, hành động, kết quả mong đợi và hậu điều kiện được mô tả **trừu tượng** — bằng lớp giá trị, bằng quy tắc, không có giá trị cụ thể.
- **Test case cụ thể** (*concrete test case*, còn gọi *low-level test case*): các phần đó có **giá trị cụ thể**, chạy được mà không phải diễn giải thêm.

Ví dụ trên cùng một test condition «phí giao chịu hệ số cao điểm và phụ phí trời mưa»:

| Trường | Test case logic TC-FOOD-005 | Test case cụ thể TC-FOOD-005/TD-08 |
|---|---|---|
| Tiền điều kiện | thời điểm đặt thuộc giờ cao điểm; thành phố đang mưa | đồng hồ hệ thống 04/11/2026 11:30 giờ VN; thời tiết giả: `SG` đang mưa |
| Đầu vào | đơn hợp lệ, quãng đường cho cước cơ bản có phần lẻ sau khi nhân 1,25 | `{"kind":"food","cityCode":"SG","distanceKm":3.3,"items":[{"name":"Cơm tấm sườn","unitPrice":35000,"quantity":2},{"name":"Trà đá","unitPrice":5000,"quantity":1}]}` |
| Kết quả mong đợi | phí giao = (cơ bản × 1,25 + 8.000) làm tròn lên 1.000 đ; tổng = tiền món + phí giao | HTTP 200; `itemsSubtotal` 75.000; `deliveryFee` 42.000; `total` 117.000 |

Cột trái đúng với mọi quãng đường; cột phải chỉ đúng với 3,3 km. Đó vừa là điểm mạnh vừa là điểm yếu của mỗi bên — mục 2.4 bàn cách chọn.

Một test case logic thường sinh **nhiều** test case cụ thể (TC-FOOD-005 sinh ba: chỉ cao điểm, chỉ mưa, cả hai). Ngược lại, nếu thiết kế tốt, mỗi test case cụ thể truy vết về **đúng một** test case logic. Quan hệ một-nhiều này là lý do bảng test data cần cột «thuộc test case nào».

### 2.3. Test procedure, test suite, test execution schedule

Ba thuật ngữ này đều là testware của **test implementation** (CTFL mục 1.4.3) và hay bị dùng lẫn. Theo ISTQB Glossary, diễn đạt lại:

| Thuật ngữ | Là gì | Ví dụ trong bài |
|---|---|---|
| **Test procedure** | chuỗi test case theo **thứ tự thực hiện**, kèm các hành động cần để dựng tiền điều kiện ban đầu và dọn dẹp sau khi chạy | TP-FOOD-01: dựng môi trường → báo giá → đặt chuyến → tài xế nhận → khách huỷ → kiểm tài xế về `online` → dọn |
| **Test suite** | một tập test script hoặc test procedure được chạy trong một lần chạy | TS-FOOD-SMOKE, TS-FOOD-REGRESSION |
| **Test execution schedule** | lịch chạy các test suite trong một chu kỳ test (thứ tự, ai/khi nào chạy) | smoke trước, rồi ca dương, ca âm, procedure; 03b sau 03a |

Mối quan hệ bao nhau: test case → được sắp vào test procedure → test procedure (và test script — dạng chạy được của procedure) gom thành test suite → test suite xếp vào test execution schedule. Phần **setup** và **teardown** thuộc về procedure, không thuộc về test case: test case nói «tiền điều kiện: tài xế D101 đã nhận chuyến», còn procedure nói «bước 3: gọi `POST /api/trips/:id/accept` với `driver:D101`» để tạo ra tiền điều kiện đó.

CTFL nói rõ procedure được **ưu tiên và sắp xếp** trong schedule «để chạy hiệu quả». Hiệu quả ở đây có hai nghĩa: ca quan trọng chạy trước (ưu tiên — chủ đề của [TCD-09](/articles/test-case-design-uu-tien-review-bao-tri)), và thứ tự phải tôn trọng **phụ thuộc** — ca B cần trạng thái mà ca A để lại thì A phải chạy trước. Phụ thuộc là ràng buộc cứng, ưu tiên là sở thích: khi hai thứ xung đột, phụ thuộc thắng. Mục 4.6 cho thấy hệ quả.

### 2.4. Khi nào dừng ở mức logic, khi nào đi tới mức cụ thể

CTFL v4.0.1 không ra một quy tắc cố định; mức chi tiết của testware tuỳ ngữ cảnh. Bảng sau tổng hợp các yếu tố thực hành thường dùng để quyết định:

| Yếu tố | Nghiêng về **logic** | Nghiêng về **cụ thể** |
|---|---|---|
| Người chạy | tester giàu kinh nghiệm về domain, chạy kiểu exploratory hoặc theo test charter | người mới, đội thuê ngoài, người không biết domain |
| Cách chạy | thủ công, có quan sát và phán đoán | tự động — máy không «tự biết» chọn giá trị |
| Ràng buộc pháp lý, kiểm toán | không có | ngành có quy định (tài chính, y tế) — phải chứng minh đã chạy gì, mong đợi gì |
| Độ ổn định của dữ liệu | dữ liệu đổi thường xuyên (bảng giá, voucher theo mùa) — test case cụ thể sẽ lỗi thời nhanh | dữ liệu ổn định hoặc được quản lý cùng phiên bản (ví dụ `pricingVersion`) |
| Rủi ro của quy tắc | thấp, sai lệch nhỏ chấp nhận được | tiền, an toàn, pháp lý — một đồng sai cũng là defect |
| Giai đoạn | sớm, yêu cầu còn đổi | ổn định, sắp phát hành, cần regression lặp lại |

Hai điểm hay bị hiểu sai. Thứ nhất, «logic» không có nghĩa là «mơ hồ»: test case logic tốt vẫn phải nói rõ **lớp** giá trị (biên nào, phân vùng nào) và **quy tắc** của kết quả mong đợi — «phí giao đúng» là mơ hồ, «phí giao = max(cước cơ bản, 15.000) làm tròn lên 1.000 đ» là logic. Thứ hai, chọn mức logic không xoá bỏ mức cụ thể; nó chỉ **dời** việc cụ thể hoá sang lúc chạy, và đẩy trách nhiệm ghi lại giá trị đã dùng sang test log. Đội chạy logic mà không ghi giá trị vào log thì mất luôn khả năng tái hiện defect.

Với Tới Nơi, quy tắc tiền (phí giao, voucher) rơi vào cột phải ở gần như mọi dòng của bảng: tự động, rủi ro tiền, cần regression. Nhưng test case logic vẫn có giá trị — chúng là nơi review với BA (BA đọc được «tiền món đúng bằng mức tối thiểu» dễ hơn đọc một mảng JSON), và là thứ ổn định khi bảng giá đổi.

### 2.5. Thuật ngữ Anh – Việt

| Thuật ngữ | Cách dùng trong bài | Ghi chú |
|---|---|---|
| logical test case / high-level test case | test case logic | giá trị trừu tượng |
| concrete test case / low-level test case | test case cụ thể | giá trị cụ thể, chạy được |
| test procedure | test procedure | thứ tự + setup + teardown |
| test script | test script | procedure ở dạng chạy được (thủ công hoặc tự động) |
| test suite | test suite | tập procedure/script chạy cùng một lần |
| test execution schedule | test execution schedule | lịch chạy các suite trong một chu kỳ |
| precondition / postcondition | tiền điều kiện / hậu điều kiện | |
| setup / teardown | dựng / dọn | phần của procedure |
| test data | test data | bảng dữ liệu TD-nn |
| data-driven testing | kiểm thử hướng dữ liệu | một script, nhiều dòng dữ liệu |

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Giao đồ ăn chiếm phần lớn đơn buổi trưa của Tới Nơi ([hồ sơ dự án](/articles/du-an-mau-toi-noi)). Test basis là nhóm R-FOOD của SPEC lab cùng các quy tắc nó dẫn tới:

| Mã | Quy tắc (tóm tắt) |
|---|---|
| R-FOOD-01 | Khách trả = (tiền món − tiền giảm voucher) + phí giao |
| R-FOOD-02 | 1–20 dòng món; tên 1–100 ký tự; đơn giá nguyên, bội số 1.000 đ, trong [1.000; 5.000.000]; số lượng nguyên [1; 50]; sai ⇒ `VALIDATION_ERROR` |
| R-FOOD-03 | Tiền món < 30.000 đ ⇒ `SUBTOTAL_TOO_LOW`; đúng 30.000 đ hợp lệ |
| R-FOOD-04 | Phí giao theo công thức xe máy của R-FARE (làm tròn quãng đường, cao điểm, mưa, tối thiểu 15.000 đ, làm tròn lên 1.000 đ); voucher không trừ vào phí giao |
| R-FOOD-05 | Tiền giảm tính trên tiền món; điều kiện tối thiểu của voucher so với tiền món; tiền giảm là số nguyên đồng |
| R-FOOD-06 | `vehicle` bỏ trống hoặc `bike`; `car` ⇒ `VALIDATION_ERROR` |
| R-FARE-04, -05 | Cao điểm [11:00; 13:00) và [17:00; 19:00) ⇒ ×1,25; mưa ⇒ +8.000 đ sau hệ số |
| R-VOUCHER-06, -07 | Tiền giảm = min(percent% × A, trần); lượt dùng trong ngày theo `perCustomerPerDay` |
| R-CANCEL-03 | Phí huỷ xe máy 10.000 đ — gồm mọi đơn đồ ăn |

Dữ liệu mẫu dùng trong bài (SPEC Phụ lục A): khách `C001`, tài xế xe máy `D101` ở TP. Hồ Chí Minh, voucher `GIAM20` (20%, trần 30.000 đ, không tối thiểu) và `FOOD15` (15%, trần 25.000 đ, tiền món tối thiểu 50.000 đ, chỉ đồ ăn, 2 lần/ngày).

### 3.2. System context

Kiến trúc đích: ứng dụng khách → API Gateway → **Pricing & Quote** (tính tiền món, voucher qua **Promotion**, phí giao theo công thức xe máy) → báo giá có hạn 5 phút → **Booking** tạo chuyến → **Dispatch** cho tài xế xe máy nhận. Trong lab (modular monolith), mọi thứ ở `src/`: `priceFood` trong `src/domain/food.ts`, endpoint `POST /api/quotes` trong `src/api/app.ts`. Test của bài chạy qua HTTP bằng supertest với kho bộ nhớ, đồng hồ `FixedClock` và thời tiết giả — đủ để mọi tiền điều kiện về thời điểm và thời tiết đều **dựng được**, không phải chờ trời mưa thật.

### 3.3. Rủi ro

| Rủi ro sản phẩm | Hậu quả | Test case logic giảm rủi ro |
|---|---|---|
| voucher trừ nhầm vào phí giao, hoặc tính trên tổng thay vì tiền món | thất thu hoặc thu sai, quán và tài xế khiếu nại | TC-FOOD-006, 007 |
| biên 30.000 đ sai chiều (`<=` thay vì `<`) | khách đặt đúng 30.000 đ bị từ chối | TC-FOOD-002, 003 |
| phí giao không áp tối thiểu, hoặc làm tròn xuống | thất thu trên đơn ngắn | TC-FOOD-004 |
| mưa bị nhân hệ số cao điểm | thu thừa khi trời mưa giờ trưa | TC-FOOD-005 |
| nhận `car` cho đơn đồ ăn | điều phối sai loại xe | TC-FOOD-010 |

Rủi ro **của chính bộ test** cũng đáng ghi: test case cụ thể lỗi thời khi bảng giá đổi; test case phụ thuộc nhau làm một lỗi môi trường kéo đổ cả chục ca; dữ liệu của test case trượt ra khỏi lớp mà test case định kiểm (một ca «tiền món đúng tối thiểu» mà ai đó sửa dữ liệu thành 32.000 đ thì vẫn xanh nhưng không còn kiểm biên nữa).

## 4. Phân tích kỹ thuật

### 4.1. Từ test condition tới 12 test case logic

Từ R-FOOD, test analysis (theo cách của [TCD-01](/articles/test-case-design-test-analysis-test-condition)) cho các test condition; ở test design, mỗi test condition thành một hoặc vài test case logic. Bảng dưới là kết quả, viết đúng như trong lab (`test-case-logic.ts`):

| Test case logic | Tiêu đề | Truy vết | Kết quả mong đợi (mức logic) |
|---|---|---|---|
| TC-FOOD-001 | Đơn hợp lệ cơ bản | R-FOOD-01, 02, 04 | 200; tổng = tiền món + phí giao xe máy |
| TC-FOOD-002 | Tiền món đúng mức tối thiểu | R-FOOD-03 | 200 |
| TC-FOOD-003 | Tiền món dưới tối thiểu một bậc 1.000 đ | R-FOOD-03, R-VALID-01 | 422 `SUBTOTAL_TOO_LOW` |
| TC-FOOD-004 | Phí giao quanh cước tối thiểu | R-FOOD-04, R-FARE-07, 08 | phí giao = max(cơ bản, 15.000) làm tròn lên |
| TC-FOOD-005 | Phí giao chịu cao điểm và mưa | R-FOOD-04, R-FARE-04, 05 | cơ bản → ×1,25 → +8.000 → làm tròn lên |
| TC-FOOD-006 | Voucher % trên tiền món, không trừ phí giao | R-FOOD-05, R-VOUCHER-06 | giảm = percent × tiền món; phí giao giữ nguyên |
| TC-FOOD-007 | Tiền giảm bị chặn ở trần | R-FOOD-05, R-VOUCHER-06 | giảm = trần |
| TC-FOOD-008 | Dưới mức tối thiểu của voucher | R-FOOD-05, R-VOUCHER-06, 08 | 422 `VOUCHER_MIN_AMOUNT` |
| TC-FOOD-009 | Đúng mức tối thiểu của voucher | R-FOOD-05, R-VOUCHER-06 | 200; giảm có thể không tròn nghìn |
| TC-FOOD-010 | Gửi `car` cho đơn đồ ăn | R-FOOD-06, R-API-06 | 400 `VALIDATION_ERROR` |
| TC-FOOD-011 | Bỏ trống `vehicle` hay gửi `bike` cho cùng kết quả | R-FOOD-06, R-GEN-04 | 200; `vehicle` = `bike`; tổng bằng nhau |
| TC-FOOD-012 | Dòng món sai định dạng | R-FOOD-02, R-API-06 | 400 `VALIDATION_ERROR` |

Mỗi test case logic trong lab còn có một trường mà bảng trên không hiện: **ràng buộc lớp dữ liệu** (`rangBuoc`) — một hàm trả lời «dòng dữ liệu này có thuộc đúng lớp mà test case mô tả không». TC-FOOD-002 đòi tiền món bằng đúng 30.000 đ; TC-FOOD-006 đòi voucher `GIAM20` với 20% tiền món *nhỏ hơn* trần (nếu không thì nó đã là TC-FOOD-007). Ràng buộc biến câu chữ của test case logic thành thứ máy kiểm được, và chặn rủi ro «dữ liệu trượt lớp» ở mục 3.3.

Lưu ý ca âm: mã lỗi (`SUBTOTAL_TOO_LOW`, `VALIDATION_ERROR`) đã **cụ thể ngay ở mức logic**, vì nó không phụ thuộc giá trị nhập. Không phải phần nào của test case logic cũng trừu tượng — chỉ những phần thực sự thay đổi theo dữ liệu.

### 4.2. Mở rộng bằng bảng test data

Bảng test data gán giá trị cho từng test case logic. Chung cho mọi dòng: thành phố `SG`, khách `C001`; «ngoài cao điểm» = 04/11/2026 09:00 giờ VN, «cao điểm» = 11:30 cùng ngày (R-FARE-04, R-GEN-02).

| Dòng | Test case logic | Món | Km | Cao điểm | Mưa | Voucher | Mong đợi |
|---|---|---|---:|---|---|---|---|
| TD-01 | TC-FOOD-001 | Cơm tấm 35.000 × 2, trà đá 5.000 × 1 | 3,2 | không | không | — | 102.000 |
| TD-02 | TC-FOOD-002 | Bánh mì 15.000 × 2 | 2 | không | không | — | 51.000 |
| TD-03 | TC-FOOD-003 | Bánh mì 14.000, sữa đậu 15.000 | 2 | không | không | — | 422 `SUBTOTAL_TOO_LOW` |
| TD-04 | TC-FOOD-004 | Phở bò 40.000 | 0,6 | không | không | — | 55.000 |
| TD-05 | TC-FOOD-004 | Phở bò 40.000 | 0,7 | không | không | — | 56.000 |
| TD-06 | TC-FOOD-005 | như TD-01 | 3,3 | có | không | — | 109.000 |
| TD-07 | TC-FOOD-005 | như TD-01 | 3,3 | không | có | — | 110.000 |
| TD-08 | TC-FOOD-005 | như TD-01 | 3,3 | có | có | — | 117.000 |
| TD-09 | TC-FOOD-006 | như TD-01 | 3,2 | không | không | `GIAM20` | 87.000 |
| TD-10 | TC-FOOD-007 | Lẩu thái 150.000 | 3,2 | không | không | `GIAM20` | 147.000 |
| TD-11 | TC-FOOD-007 | Lẩu thái 200.000 | 3,2 | không | không | `GIAM20` | 197.000 |
| TD-12 | TC-FOOD-008 | Gà rán 49.000 | 3,2 | không | không | `FOOD15` | 422 `VOUCHER_MIN_AMOUNT` |
| TD-13 | TC-FOOD-009 | Gà rán 50.000 | 3,2 | không | không | `FOOD15` | 69.500 |
| TD-14 | TC-FOOD-010 | như TD-01, `vehicle: car` | 3,2 | không | không | — | 400 `VALIDATION_ERROR` |
| TD-15 | TC-FOOD-011 | Bún bò 45.000, không gửi `vehicle` | 4 | không | không | — | 75.000 |
| TD-16 | TC-FOOD-011 | Bún bò 45.000, `vehicle: bike` | 4 | không | không | — | 75.000 |
| TD-17 | TC-FOOD-012 | Chè 15.500 × 2 | 3,2 | không | không | — | 400 `VALIDATION_ERROR` |
| TD-18 | TC-FOOD-012 | Chè 15.000 × 0 | 3,2 | không | không | — | 400 `VALIDATION_ERROR` |
| TD-19 | TC-FOOD-012 | Chè 1.000 × 51 | 3,2 | không | không | — | 400 `VALIDATION_ERROR` |

12 test case logic × bảng 19 dòng ⇒ **19 test case cụ thể**, đặt tên `TC-FOOD-nnn/TD-nn`. Kiểm tay vài dòng để thấy con số đến từ đâu:

- **TD-01.** Tiền món 35.000 × 2 + 5.000 = 75.000. Phí giao: 12.000 + 4.500 × 3,2 = 26.400 ⇒ ≥ 15.000 ⇒ làm tròn lên **27.000**. Tổng 75.000 + 27.000 = **102.000 đ**.
- **TD-04 và TD-05** là một cặp biên của cước tối thiểu: 0,6 km ⇒ 12.000 + 2.700 = 14.700 < 15.000 ⇒ phí giao **15.000**; 0,7 km ⇒ 15.150 ⇒ làm tròn lên **16.000**. Nhưng một bản cài đặt *quên* cước tối thiểu cũng trả đúng 15.000 ở TD-04 (vì 14.700 làm tròn lên cũng là 15.000) và 16.000 ở TD-05 — cặp dòng này không phân biệt được bản đúng với bản lỗi đó. Đây là ví dụ nhỏ về việc test case cụ thể phải được **review**, không chỉ test case logic — Bài tập 2 phân tích tới cùng.
- **TD-08.** 3,3 km ⇒ cơ bản 12.000 + 14.850 = 26.850 ⇒ × 1,25 = 33.562,5 ⇒ + 8.000 = 41.562,5 ⇒ làm tròn lên **42.000**. Tổng 75.000 + 42.000 = **117.000 đ**. Chọn 3,3 km thay vì 3,2 km là có chủ ý: 26.400 × 1,25 = 33.000 tròn trịa, không lộ được lỗi làm tròn sai chiều; 33.562,5 thì lộ.
- **TD-09.** `GIAM20` trên tiền món: 20% × 75.000 = 15.000 < trần 30.000. Phí giao vẫn 27.000 (voucher không đụng phí giao). Tổng 75.000 − 15.000 + 27.000 = **87.000 đ**. Đồng thời bảng chi tiết của phí giao (`fare.discount`) phải bằng 0.
- **TD-13.** `FOOD15`: tiền món 50.000 = đúng mức tối thiểu ⇒ hợp lệ; 15% × 50.000 = **7.500**; tổng 50.000 − 7.500 + 27.000 = **69.500 đ** — không tròn nghìn. R-FOOD-01 không yêu cầu làm tròn tổng đơn đồ ăn, nên đây là kết quả đúng theo SPEC. Nhưng nó đáng một câu hỏi gửi BA (mục 6).

Kết quả mong đợi trong lab được tính **hai lần**: người viết tính tay vào cột `tinhTay`, và hàm mở rộng tính lại bằng công thức chép từ SPEC. Test đối chiếu hai nguồn trước khi chạy bất cứ thứ gì trên hệ thống — kỹ thuật «ghi hai lần» quen thuộc của kế toán, ở đây dùng để bắt lỗi của chính test data.

### 4.3. Hàm mở rộng làm những gì

Mở rộng một test case logic thành test case cụ thể không chỉ là thay chữ bằng số. Hàm `moRong` trong lab làm năm việc theo thứ tự:

1. **Lọc** các dòng dữ liệu thuộc test case logic; không có dòng nào ⇒ báo lỗi (test case logic chưa được hiện thực hoá — một lỗ hổng phủ).
2. **Kiểm ràng buộc lớp** của từng dòng; dòng trượt lớp ⇒ báo lỗi ngay, trước khi chạy.
3. **Cụ thể hoá tiền điều kiện**: «cao điểm» thành thời điểm ISO có múi giờ, «mưa» thành danh sách thành phố đang mưa cho thời tiết giả.
4. **Dựng đầu vào**: thân `POST /api/quotes`, chỉ gửi `vehicle`, `voucherCode` khi dòng dữ liệu có.
5. **Tính kết quả mong đợi từ SPEC**: tham số giá (12.000 đ mở cửa, 4.500 đ/km, tối thiểu 15.000 đ, 8.000 đ mưa, voucher theo Phụ lục A) chép vào lab, không import từ `src/`. Phí giao tính bằng số nguyên «phần tư đồng» để × 1,25 không sinh sai số dấu phẩy động.

Điểm 5 là ranh giới với [TCD-04](/articles/test-case-design-expected-result-test-oracle): ở đây hàm mở rộng là một **test oracle** đơn giản cho đúng phạm vi bảng dữ liệu; TCD-04 bàn kỹ cách viết oracle độc lập và những cái bẫy của nó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 340" role="img" aria-label="Từ 12 test case logic và bảng 19 dòng test data, hàm mở rộng sinh 19 test case cụ thể. Test case được sắp vào test procedure có setup, các bước và teardown; procedure gom vào test suite smoke và regression; các suite xếp vào test execution schedule theo phụ thuộc trước, ưu tiên sau.">
<defs>
<marker id="tcd03-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="230" y="22" text-anchor="middle" class="t-small t-b">TEST DESIGN</text>
<text x="690" y="22" text-anchor="middle" class="t-small t-b">TEST IMPLEMENTATION</text>
<line x1="460" y1="10" x2="460" y2="300" class="s-line dash" stroke-width="1"/>
<rect x="20" y="40" width="180" height="78" rx="8" class="f-pk-soft s-pk"/>
<text x="110" y="64" text-anchor="middle" class="t-b t-pk">12 test case logic</text>
<text x="110" y="84" text-anchor="middle" class="t-small">TC-FOOD-001…012</text>
<text x="110" y="102" text-anchor="middle" class="t-small">lớp giá trị + quy tắc</text>
<rect x="20" y="140" width="180" height="78" rx="8" class="f-surface s-line"/>
<text x="110" y="164" text-anchor="middle" class="t-b">Bảng test data</text>
<text x="110" y="184" text-anchor="middle" class="t-small">TD-01…TD-19</text>
<text x="110" y="202" text-anchor="middle" class="t-small">giá trị + cột tính tay</text>
<rect x="250" y="90" width="180" height="78" rx="8" class="f-phys-soft s-phys"/>
<text x="340" y="114" text-anchor="middle" class="t-b t-phys">moRong()</text>
<text x="340" y="134" text-anchor="middle" class="t-small">kiểm lớp · dựng tiền đk</text>
<text x="340" y="152" text-anchor="middle" class="t-small">mong đợi từ SPEC</text>
<line x1="200" y1="79" x2="248" y2="115" class="s-muted" stroke-width="1.5" marker-end="url(#tcd03-a)"/>
<line x1="200" y1="179" x2="248" y2="143" class="s-muted" stroke-width="1.5" marker-end="url(#tcd03-a)"/>
<rect x="250" y="210" width="180" height="70" rx="8" class="f-phys-soft s-phys"/>
<text x="340" y="236" text-anchor="middle" class="t-b t-phys">19 test case cụ thể</text>
<text x="340" y="256" text-anchor="middle" class="t-small t-mono">TC-FOOD-005/TD-08</text>
<line x1="340" y1="168" x2="340" y2="208" class="s-muted" stroke-width="1.5" marker-end="url(#tcd03-a)"/>
<rect x="490" y="40" width="200" height="110" rx="8" class="f-surface s-phys"/>
<text x="590" y="62" text-anchor="middle" class="t-b">Test procedure</text>
<text x="590" y="84" text-anchor="middle" class="t-small">setup: đồng hồ, thời tiết</text>
<text x="590" y="102" text-anchor="middle" class="t-small">bước: báo giá → đặt → nhận → huỷ</text>
<text x="590" y="120" text-anchor="middle" class="t-small">teardown: kiểm tài xế, dọn</text>
<text x="590" y="138" text-anchor="middle" class="t-small t-mono">TP-FOOD-01, 02, 03</text>
<line x1="430" y1="235" x2="488" y2="120" class="s-muted" stroke-width="1.5" marker-end="url(#tcd03-a)"/>
<rect x="490" y="180" width="200" height="90" rx="8" class="f-surface s-pk"/>
<text x="590" y="204" text-anchor="middle" class="t-b">Test suite</text>
<text x="590" y="226" text-anchor="middle" class="t-small">SMOKE: 1 ca + TP-FOOD-01</text>
<text x="590" y="246" text-anchor="middle" class="t-small">REGRESSION: 19 ca + 3 TP</text>
<line x1="590" y1="150" x2="590" y2="178" class="s-muted" stroke-width="1.5" marker-end="url(#tcd03-a)"/>
<line x1="430" y1="250" x2="488" y2="235" class="s-muted" stroke-width="1.5" marker-end="url(#tcd03-a)"/>
<rect x="730" y="100" width="170" height="130" rx="8" class="f-warn-soft s-warn"/>
<text x="815" y="124" text-anchor="middle" class="t-b t-warn">Execution schedule</text>
<text x="815" y="146" text-anchor="middle" class="t-small">1. SMOKE</text>
<text x="815" y="164" text-anchor="middle" class="t-small">2. ca dương · ca âm</text>
<text x="815" y="182" text-anchor="middle" class="t-small">3. TP-FOOD-02, 03a</text>
<text x="815" y="200" text-anchor="middle" class="t-small">4. 03b (chờ 03a)</text>
<text x="815" y="218" text-anchor="middle" class="t-small">phụ thuộc trước, ưu tiên sau</text>
<line x1="690" y1="225" x2="728" y2="190" class="s-muted" stroke-width="1.5" marker-end="url(#tcd03-a)"/>
<text x="460" y="326" text-anchor="middle" class="t-small">Mọi con số trong hình được kiểm bằng test trong examples/toi-noi-testing/articles/test-case-design-logic-va-cu-the</text>
</svg>
</div>
<figcaption><b>Hình 1 · Từ test case logic tới lịch chạy.</b> Bên trái là sản phẩm của test design (ý đồ, ổn định); bên phải là sản phẩm của test implementation (giá trị, thứ tự, môi trường). Bảng test data là chỗ duy nhất phải sửa khi bảng giá đổi.</figcaption>
</figure>

### 4.4. Test procedure: thứ tự, setup, teardown

Mỗi dòng ở mục 4.2 chỉ cần một lời gọi API, nên procedure của nó rất ngắn: dựng hệ thống mới, đặt đồng hồ và thời tiết, gọi `POST /api/quotes`, so kết quả. Phần thú vị là các test case cần **chuỗi** hành động. TP-FOOD-01 chạy hai test case — TC-FOOD-001 (báo giá đơn cơ bản) và TC-FOOD-013 (khách huỷ đơn đồ ăn trong 2 phút đầu thì miễn phí, R-CANCEL-01/02):

| # | Phần | Hành động | Kết quả mong đợi |
|---|---|---|---|
| 1 | setup | đặt đồng hồ 04/11/2026 09:00 giờ VN | — |
| 2 | setup | thời tiết giả: `SG` không mưa | — |
| 3 | bước | `POST /api/quotes`, `x-user: customer:C001`, thân của TD-01 | 200, `total` 102.000; lưu `quoteId` |
| 4 | bước | `POST /api/trips`, `Idempotency-Key: tp-food-01-dat`, `{ quoteId }` | 201, `status` = `searching`; lưu `tripId` |
| 5 | bước | `POST /api/trips/:id/accept`, `x-user: driver:D101`, `{ "etaMinutes": 5 }` | 200, `status` = `accepted` |
| 6 | bước | đồng hồ + 1 phút | — |
| 7 | bước | `POST /api/trips/:id/cancel`, khách `C001` | 200, `cancellationFee` 0, `feeReason` = `grace_period`, `status` = `cancelled` |
| 8 | teardown | kiểm hậu điều kiện: tài xế `D101` | `online` (R-ACCEPT-05) |
| 9 | teardown | dọn: reset kho, thời tiết, ví giả | — |

TP-FOOD-02 giống hệt nhưng bước 6 là + 5 phút và bước 7 mong đợi phí **10.000 đ**, `charged` — R-CANCEL-03 nói rõ mọi đơn đồ ăn tính theo xe máy.

Ba quy tắc viết procedure mà lab áp dụng:

- **Setup dựng tiền điều kiện bằng hành động, không giả định.** «Tài xế đã nhận» là kết quả của bước 5, không phải dữ liệu chèn thẳng vào kho. Đổi lại procedure dài hơn; được lại là mỗi bước đi đúng đường người dùng đi, và nếu bước 5 đỏ thì biết ngay vì sao bước 7 không chạy.
- **Teardown luôn chạy**, kể cả khi bước giữa chừng thất bại. Runner trong lab đặt teardown trong `finally`; test thứ ba của phần 3 cố ý bỏ bước đặt chuyến để chứng minh: bước nhận chuyến trượt, các bước sau không chạy, nhưng hai dòng teardown vẫn có trong nhật ký.
- **Hậu điều kiện là một phần của kết quả mong đợi**, không phải việc dọn dẹp. Bước 8 kiểm tài xế về `online` — một đầu ra không nằm trong phản hồi của lệnh huỷ nhưng là yêu cầu của SPEC. TCD-04 mở rộng ý này thành «mọi đầu ra quan sát được».

### 4.5. Test suite: gom theo mục đích chạy

Một test case có thể nằm trong nhiều suite; suite được định nghĩa bằng **mục đích của lần chạy**, không bằng tính năng. Lab có hai:

| Suite | Mục đích | Thành phần |
|---|---|---|
| TS-FOOD-SMOKE | chạy mỗi lần build, vài giây: «đặt đồ ăn còn sống không?» | TC-FOOD-001/TD-01, TP-FOOD-01 |
| TS-FOOD-REGRESSION | chạy trước mỗi bản phát hành | 19 test case cụ thể + TP-FOOD-01, 02, 03 = 22 thành phần |

Test của lab kiểm ba tính chất của suite: mọi thành phần tồn tại (không trỏ tới test case đã bị xoá), không trùng lặp, và smoke là tập con của regression. Đó là kiểm tra rẻ tiền nhưng hay bị bỏ qua: suite trong công cụ quản lý test hay mục tool CI cấu hình bằng tay rất dễ «mồ côi» một ca sau vài lần đổi tên.

### 4.6. Phụ thuộc giữa test case và test execution schedule

R-VOUCHER-07 cho `FOOD15` 2 lượt/ngày. Test condition «lượt thứ ba trong ngày bị chặn» cần hai chuyến đã đặt bằng `FOOD15` làm tiền điều kiện. Có hai cách viết:

- **Phụ thuộc:** TP-FOOD-03a đặt hai đơn `FOOD15` (75.000 − 11.250 + 27.000 = 90.750 đ mỗi đơn) và *để lại* chúng trong kho; TP-FOOD-03b chỉ có một bước: báo giá lần ba, mong đợi 422 `VOUCHER_USAGE_LIMIT`.
- **Độc lập:** TP-FOOD-03 tự đặt hai đơn trong **setup** của chính nó, chạy bước báo giá lần ba, và **teardown** dọn sạch.

Lab chạy cả hai và cho kết quả:

| Cách chạy | 03a → 03b (phụ thuộc) | TP-FOOD-03 (độc lập) |
|---|---|---|
| theo đúng thứ tự trên cùng một môi trường | đạt | đạt |
| chạy riêng ca kiểm lượt thứ ba | **trượt**: `expected 200 to be 422` | đạt |
| chạy hai lần liền trên cùng môi trường | — | đạt (teardown đã dọn) |
| chạy sau một procedure khác | phụ thuộc vào procedure đó | đạt |

Dòng thứ hai là vấn đề cốt lõi: 03b **trượt khi hệ thống hoàn toàn đúng**. Đó là false positive do thiết kế test, và mỗi lần như vậy tốn một người điều tra. Phụ thuộc còn gây hai hậu quả khác mà lab cũng đo:

- **Lịch bị ràng buộc.** Hàm `lapLich` sắp mục theo thứ tự topo của phụ thuộc, trong số mục sẵn sàng thì ưu tiên cao chạy trước. 03b có ưu tiên 1 (cao nhất) nhưng vẫn phải đứng cuối, sau 03a ưu tiên 3. Ưu tiên của 03b trên giấy là vô nghĩa — đúng điều [TCD-09](/articles/test-case-design-uu-tien-review-bao-tri) sẽ gọi là ưu tiên có phụ thuộc. Vòng phụ thuộc (A cần B, B cần A) thì không lập được lịch: hàm báo lỗi.
- **Lỗi lan.** Hàm `chayTheoLich` đánh dấu `blocked` mọi mục có phụ thuộc không đạt. Smoke trượt ⇒ **5 mục** còn lại bị blocked. Chỉ 03a trượt ⇒ chỉ 03b bị blocked. Một procedure bị blocked là một procedure không cho thông tin gì về chất lượng sản phẩm.

Vì vậy quy tắc thực hành là **giảm phụ thuộc tới mức tối thiểu**: mỗi test case (hoặc mỗi procedure) tự dựng tiền điều kiện của mình và tự dọn. Phụ thuộc chỉ nên giữ khi có lý do rõ — dựng tiền điều kiện quá đắt (ví dụ một bộ dữ liệu lớn cần nhiều phút), hoặc chính chuỗi là thứ cần kiểm (một kịch bản end-to-end; khi đó cả chuỗi là *một* procedure, không phải nhiều test case rời). Phụ thuộc vào **smoke** là phụ thuộc hợp lý: nếu hệ thống không sống thì chạy tiếp cũng vô ích; nó tiết kiệm thời gian thay vì gây false positive.

## 5. Thực hành

### 5.1. Tổ chức mã trong lab

Mọi tệp ở [`articles/test-case-design-logic-va-cu-the`](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/test-case-design-logic-va-cu-the); không sửa `src/`.

| Tệp | Vai trò |
|---|---|
| `test-case-logic.ts` | 12 test case logic (tiền điều kiện, đầu vào, kết quả mong đợi ở mức logic, ràng buộc lớp) và bảng TD-01…TD-19 kèm cột tính tay |
| `mo-rong.ts` | `moRong()`: logic × dòng dữ liệu ⇒ test case cụ thể; `phiGiaoTheoSpec()` tính bằng số nguyên |
| `thu-tuc.ts` | runner test procedure, TP-FOOD-01/02/03, 03a/03b, `lapLich()`, `chayTheoLich()` |
| `logic-va-cu-the.test.ts` | 34 ca vitest |

Test case logic là **dữ liệu**, không phải chú thích trong code:

```ts
// test-case-logic.ts (trích)
{
  id: 'TC-FOOD-005',
  tieuDe: 'Phí giao chịu hệ số cao điểm và phụ phí mưa',
  truyVet: ['R-FOOD-04', 'R-FARE-04', 'R-FARE-05'],
  tienDieuKien: 'cao điểm và/hoặc mưa (mỗi dòng một tổ hợp khác «không cả hai»)',
  dauVao: 'quãng đường cho cước cơ bản có phần lẻ sau khi nhân 1,25',
  ketQuaMongDoi: '200; phí giao theo thứ tự cơ bản → ×1,25 → +8.000 → làm tròn lên',
  rangBuoc: (d) => d.caoDiem || d.mua,
},
```

Phí giao theo SPEC, không dùng số thực cho bước × 1,25:

```ts
// mo-rong.ts (trích)
const coBan = XE_MAY.moCua + (XE_MAY.moiKm / 10) * phanMuoiKm; // đồng, nguyên
let phanTu = coBan * 4;                                        // đơn vị 1/4 đồng
if (caoDiem) phanTu = coBan * 5;                               // ×1,25 = ×5/4
if (mua) phanTu += PHU_PHI_MUA * 4;
phanTu = Math.max(phanTu, XE_MAY.toiThieu * 4);
return Math.ceil(phanTu / 4_000) * 1_000;                      // làm tròn lên 1.000 đ
```

Một test case cụ thể khi chạy chỉ còn là dựng tiền điều kiện, gửi đầu vào, so kết quả — test script hướng dữ liệu (data-driven) quen thuộc:

```ts
// logic-va-cu-the.test.ts (trích)
it.each(TAT_CA_CU_THE.map((c) => [c.id, c] as const))('%s', async (_id, c) => {
  const ctx = taoNguCanh();                                   // hệ thống mới cho mỗi ca
  ctx.clock.set(new Date(c.tienDieuKien.dongHo));
  ctx.system.weather.setRaining(c.tienDieuKien.thanhPhoMua as never);
  const res = await request(ctx.app).post('/api/quotes').set('x-user', 'customer:C001').send(c.body);
  expect(res.status).toBe(c.mongDoi.http);
  // … 200: so itemsSubtotal, itemsDiscount, deliveryFee, total, vehicle; fare.discount = 0
});
```

### 5.2. Chạy

```bash
cd examples/toi-noi-testing
npx vitest run articles/test-case-design-logic-va-cu-the --reporter=verbose
```

Kết quả thật (rút gọn):

```text
✓ TCD-03 · 1. kiểm chính bộ test case trước khi chạy > dòng dữ liệu trượt khỏi lớp của test case logic bị chặn khi mở rộng
✓ TCD-03 · 2. chạy 19 test case cụ thể qua POST /api/quotes > TC-FOOD-005/TD-08
✓ TCD-03 · 3. test procedure: setup → bước → teardown > bước thất bại giữa chừng: các bước sau không chạy nhưng teardown vẫn chạy
✓ TCD-03 · 4. phụ thuộc giữa test case > bản phụ thuộc: chạy riêng 03b (hoặc đảo thứ tự) ⇒ 03b trượt dù hệ thống không có lỗi
✓ TCD-03 · 5. test suite và test execution schedule > smoke trượt ⇒ 5 mục còn lại bị blocked; chỉ 03a trượt ⇒ chỉ 03b bị blocked

 Test Files  1 passed (1)
      Tests  34 passed (34)
```

34 ca = 5 ca kiểm chính bộ test case + 19 test case cụ thể + 3 ca test procedure + 3 ca phụ thuộc + 4 ca suite và lịch. Mọi test case cụ thể đều đạt trên `src/` hiện tại — bộ test được thiết kế để đạt trên bản đúng; giá trị của nó nằm ở chỗ mỗi con số mong đợi có nguồn gốc rõ và có thể tái hiện.

### 5.3. Đọc kết quả phần «kiểm chính bộ test case»

Năm ca đầu không đụng tới hệ thống. Chúng kiểm **testware**: 12 test case logic, 19 dòng, mỗi logic có ít nhất một dòng, mỗi dòng thuộc đúng một logic, mọi mã truy vết đúng dạng `R-…`; công thức khớp cột tính tay ở cả 19 dòng; ví dụ 3,3 km của mục 4.2; và một dòng cố ý sai (tiền món 32.000 đ gắn vào TC-FOOD-002) bị chặn với thông báo «TD-99 không thuộc lớp mà TC-FOOD-002 mô tả». Đây là static testing áp vào chính bộ test — rẻ, chạy trong vài mili giây, và chặn được lỗi thiết kế test trước khi nó biến thành false positive hay false negative.

### 5.4. Thay đổi thử: bảng giá đổi

Giả sử Tới Nơi tăng giá mỗi km xe máy từ 4.500 đ lên 4.800 đ (giả định bài tập, không có trong SPEC). Sửa đúng **một hằng số** trong `mo-rong.ts` thì 19 test case cụ thể nhận kết quả mong đợi mới; 12 test case logic không đổi một chữ. Cột `tinhTay` sẽ lệch ở các dòng có phí giao — test «ghi hai lần» đỏ, buộc người sửa tính lại bằng tay và nhìn lại từng dòng. Có người sẽ coi đó là phiền; thực ra đó là chỗ kiểm soát thay đổi: ai đổi giá thì phải tự tay xác nhận con số mới. Thử chạy với hằng số mới để thấy test nào đỏ là cách nhanh nhất hiểu ranh giới «ý đồ ổn định, giá trị đổi được».

## 6. Góc nhìn nâng cao

**Test case logic và exploratory testing.** Một test charter của exploratory testing gần với test case logic nhất: nêu mục tiêu và vùng cần khám phá, để tester chọn dữ liệu khi chạy. Khác biệt là test case logic có kết quả mong đợi dạng quy tắc, còn charter thường chỉ nêu câu hỏi. Đội có thể dùng test case logic làm «bản đồ» cho phiên exploratory: chọn giá trị khác bảng TD mỗi lần, và ghi giá trị đã dùng vào ghi chú phiên — vừa đa dạng dữ liệu, vừa giữ được khả năng tái hiện.

**Kiểm thử hướng dữ liệu là test case cụ thể đã tách đúng chỗ.** `it.each` trên bảng là cách viết phổ biến; điều quan trọng không phải cú pháp mà là bảng có cột truy vết về test case logic và có kết quả mong đợi tính từ test basis. Một bảng `it.each` chỉ có đầu vào và con số «chạy ra thấy đúng» là anti-pattern mà TCD-04 phân tích kỹ.

**Phụ thuộc ẩn khó thấy hơn phụ thuộc tường minh.** 03a/03b là phụ thuộc tường minh. Các phụ thuộc ẩn hay gặp: id sinh tuần tự (`id-0001`, `id-0002`…) mà test so cứng với id; đồng hồ hệ thống thật (test chạy đúng lúc 11:00 thì cao điểm, 13:00 thì không); dữ liệu seed bị test khác sửa; bộ đếm giới hạn tần suất (R-API-13, 5 lần/60 giây) cộng dồn qua nhiều test cùng khách. Cách lộ ra: chạy bộ test theo **thứ tự ngẫu nhiên** (vitest có `--sequence.shuffle`) và chạy từng test riêng lẻ. Test nào đổi kết quả theo thứ tự là có phụ thuộc ẩn.

**Thứ tự trong một procedure khác thứ tự giữa các procedure.** Trong TP-FOOD-01, thứ tự các bước là bắt buộc — đó chính là nội dung của procedure. Giữa các procedure, thứ tự lý tưởng là **không quan trọng**. Nhầm hai thứ này dẫn tới việc tách một kịch bản thành nhiều test case rời nhưng vẫn phụ thuộc nhau — tức là có một procedure trá hình mà không có teardown.

**Hai câu hỏi gửi BA từ bảng dữ liệu.** (1) TD-13 cho tổng 69.500 đ: R-FOOD-01 không yêu cầu làm tròn tổng đơn đồ ăn, trong khi cước đặt xe luôn tròn nghìn (R-FARE-08). Có chủ ý không, và màn hình thanh toán tiền mặt xử lý 500 đ lẻ thế nào? (2) Với đồ ăn, R-API-07 trả hai trường giảm giá khác nghĩa: `discount` = tiền giảm trên món, còn `fare.discount` (bảng chi tiết phí giao) luôn bằng 0. Đúng SPEC, nhưng client đọc nhầm trường là rất dễ — nên ghi chú trong hợp đồng API. Cả hai là defect tiềm năng **trong test basis**, phát hiện được nhờ phải viết con số cụ thể.

| Sai lầm thường gặp | Cách tránh |
|---|---|
| Test case logic mơ hồ: «kiểm phí giao đúng» | nêu lớp giá trị và quy tắc của kết quả mong đợi |
| Viết cụ thể ngay từ đầu, nhúng giá trị vào bước | tách bảng test data; một test case logic, nhiều dòng |
| Dữ liệu trượt khỏi lớp mà không ai biết | ràng buộc lớp chạy được, kiểm trước khi chạy |
| Chọn giá trị «tròn» che mất lỗi làm tròn | chọn giá trị cho phần lẻ (3,3 km ⇒ 33.562,5) |
| Tiền điều kiện dựa vào ca chạy trước | setup tự dựng, teardown tự dọn |
| Teardown đặt sau bước cuối, không chạy khi lỗi | teardown trong `finally` |
| Ưu tiên cao nhưng phụ thuộc ca ưu tiên thấp | lập lịch theo phụ thuộc trước, xem lại ưu tiên |

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 (K2).** Viết test case logic TC-FOOD-016 cho R-FOOD-02 «số dòng món ở biên trên» và mở rộng thành hai test case cụ thể (TD-20, TD-21). Ghi tiền điều kiện, đầu vào, kết quả mong đợi ở cả hai mức.

<details><summary><b>Lời giải gợi ý</b></summary>
<p><b>Logic:</b> tiền điều kiện «ngoài cao điểm, không mưa»; đầu vào «đơn có đúng số dòng món tối đa» và «vượt tối đa một dòng»; kết quả mong đợi «tối đa ⇒ 200, tổng = tiền món + phí giao; vượt ⇒ 400 <code>VALIDATION_ERROR</code>». Truy vết R-FOOD-02, R-API-06. <b>Cụ thể:</b> đồng hồ 04/11/2026 09:00 giờ VN, <code>SG</code> không mưa, khách <code>C001</code>, 2 km. TD-20: 20 dòng, mỗi dòng «Món n» 2.000 đ × 1 ⇒ tiền món 40.000 ≥ 30.000; phí giao 12.000 + 9.000 = 21.000 ⇒ tổng <b>61.000 đ</b>. TD-21: 21 dòng như trên ⇒ <b>400 <code>VALIDATION_ERROR</code></b>. Lưu ý chọn đơn giá sao cho tiền món của TD-20 vượt 30.000 đ — nếu chọn 1.000 đ × 20 = 20.000 đ thì TD-20 sẽ ra <code>SUBTOTAL_TOO_LOW</code> và không còn kiểm biên số dòng.</p>
</details>

**Bài tập 2 (K2).** Ở mục 4.2, một bản cài đặt «quên» cước tối thiểu (chỉ làm tròn lên 1.000 đ) có bị TD-04 và TD-05 bắt không? Đề xuất sửa bảng dữ liệu của TC-FOOD-004.

<details><summary><b>Lời giải gợi ý</b></summary>
<p>Không bị bắt. TD-04: 14.700 làm tròn lên = 15.000, trùng với kết quả đúng. TD-05: 15.150 ⇒ 16.000, cũng trùng. Cần một dòng có cước cơ bản ≤ 14.000 đ để «làm tròn lên» và «nâng lên tối thiểu» cho kết quả khác nhau: 0,5 km ⇒ 14.250 ⇒ bản lỗi 15.000, bản đúng 15.000 — vẫn trùng! Với xe máy, cước cơ bản nhỏ nhất hợp lệ là 14.250 đ (0,5 km), mọi giá trị từ 14.001 tới 14.999 đều làm tròn lên 15.000. Kết luận: với dịch vụ đồ ăn không có voucher trên phí giao, lỗi «quên tối thiểu» <b>không quan sát được</b> qua tổng. Phải kiểm qua trường <code>fare.minimumFare</code>/<code>fare.afterDiscount</code> của bảng chi tiết, hoặc kiểm ở R-FARE-07 với đặt xe có voucher (ví dụ 0,5 km + <code>GIAM20</code> ⇒ 11.400 ⇒ bản lỗi 12.000, bản đúng 15.000). Ghi vào ràng buộc của TC-FOOD-004: «tổng không phân biệt được; kiểm thêm bảng chi tiết».</p>
</details>

**Bài tập 3 (K2).** Đội đề xuất gộp TP-FOOD-01 và TP-FOOD-02 thành một procedure: huỷ chuyến thứ nhất sau 1 phút, rồi dùng lại đơn đó đặt chuyến thứ hai và huỷ sau 5 phút. Nêu một lợi và hai rủi ro.

<details><summary><b>Lời giải gợi ý</b></summary>
<p>Lợi: ít bước setup hơn. Rủi ro 1: báo giá đã dùng cho chuyến thứ nhất không đặt được chuyến thứ hai (R-QUOTE-03, <code>QUOTE_ALREADY_USED</code>) — procedure phải báo giá lại, và nếu đồng hồ đã trôi quá 5 phút thì báo giá cũ còn hết hạn. Rủi ro 2: nếu phần đầu trượt (ví dụ tài xế không về <code>online</code>), phần sau bị blocked — mất thông tin về TC-FOOD-014 dù có thể nó đúng. Hai test case độc lập về logic nên để hai procedure độc lập.</p>
</details>

**Câu 1 (K2).** Theo CTFL v4.0.1 mục 1.4.1, việc nào thuộc **test implementation**?

a) Xác định coverage item
b) Xác định yêu cầu về test data
c) Sắp test procedure vào test execution schedule
d) Triển khai test condition thành test case

<details><summary><b>Đáp án: c</b></summary>
<p>a, b, d là việc của test design. c đúng: test implementation tạo testware cần để chạy, sắp test case thành procedure, gom suite và lập lịch.</p>
</details>

**Câu 2 (K2).** Phát biểu nào đúng về test case logic?

a) Không có kết quả mong đợi
b) Mô tả tiền điều kiện, đầu vào và kết quả mong đợi ở mức trừu tượng, chưa có giá trị cụ thể
c) Chỉ dùng cho kiểm thử thủ công
d) Phải được thay hoàn toàn bằng test case cụ thể trước khi chạy và xoá đi

<details><summary><b>Đáp án: b</b></summary>
<p>a sai: test case logic có kết quả mong đợi, dạng quy tắc. c sai: test tự động hướng dữ liệu vẫn giữ test case logic làm lớp ý đồ. d sai: test case logic được giữ lại và tái sử dụng với dữ liệu mới.</p>
</details>

**Câu 3 (K2).** Tình huống nào **ít** cần đi tới test case cụ thể nhất?

a) Bộ regression tự động cho quy tắc tính phí
b) Đội thuê ngoài chạy thủ công, không biết domain
c) Một phiên khám phá do tester giàu kinh nghiệm thực hiện trên tính năng còn đang thay đổi
d) Kiểm thử phục vụ kiểm toán trong ngành tài chính

<details><summary><b>Đáp án: c</b></summary>
<p>a, b, d đều cần giá trị cụ thể: máy không tự chọn giá trị, người không biết domain không chọn đúng, kiểm toán cần bằng chứng. c là ngữ cảnh mà test case logic (hoặc test charter) đủ, miễn giá trị đã dùng được ghi vào log.</p>
</details>

**Câu 4 (K2).** Thứ gì thuộc **test procedure** nhưng không thuộc test case?

a) Kết quả mong đợi
b) Thứ tự thực hiện các test case và hành động setup/teardown
c) Truy vết về yêu cầu
d) Tiền điều kiện

<details><summary><b>Đáp án: b</b></summary>
<p>Test case có tiền điều kiện (d), kết quả mong đợi (a), truy vết (c). Procedure thêm thứ tự và các hành động dựng tiền điều kiện, dọn sau khi chạy.</p>
</details>

**Câu 5 (K2).** Lịch chạy có: A (ưu tiên 3), B (ưu tiên 1, phụ thuộc A), C (ưu tiên 2). Không có phụ thuộc nào khác, ưu tiên 1 là cao nhất, mỗi lúc chạy được một mục. Thứ tự hợp lý?

a) B, C, A
b) C, A, B
c) A, B, C
d) B, A, C

<details><summary><b>Đáp án: b</b></summary>
<p>B không thể chạy trước A (loại a, d). Lúc đầu sẵn sàng có A và C; C ưu tiên cao hơn nên chạy trước, rồi A, rồi B mới sẵn sàng: C, A, B. c chạy A trước C dù C ưu tiên cao hơn — không sai về phụ thuộc nhưng không tôn trọng ưu tiên. (Nếu muốn B chạy sớm vì ưu tiên cao, phải đưa A lên — đó là lý do phụ thuộc làm méo ưu tiên.)</p>
</details>

**Câu 6 (K2).** Test X luôn đạt khi chạy cả bộ, nhưng trượt khi chạy riêng. Giải thích hợp lý nhất?

a) Hệ thống có race condition
b) X phụ thuộc trạng thái do test khác để lại
c) X có kết quả mong đợi sai
d) Công cụ chạy test bị lỗi

<details><summary><b>Đáp án: b</b></summary>
<p>Đây đúng là trường hợp 03b ở mục 4.6. a, c, d có thể xảy ra nhưng không giải thích được việc kết quả đổi theo việc chạy cùng bộ hay chạy riêng. Cách sửa: chuyển phần dựng trạng thái vào setup của X.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB — Certified Tester Foundation Level Syllabus v4.0.1, mục 1.4.1 Test Activities and Tasks (test design, test implementation), 1.4.3 Testware; LO FL-1.4.1 (K2), FL-1.4.3 (K2): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — tra *high-level test case*, *low-level test case*, *test procedure*, *test suite*, *test execution schedule*, *precondition*, *postcondition*: https://glossary.istqb.org/
- ISO/IEC/IEEE 29119-3 — Test documentation (mẫu test case specification, test procedure specification); ISO/IEC/IEEE 29119-2 — Test processes.
- Vitest — tuỳ chọn `sequence.shuffle`, `test.each`: https://vitest.dev/
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-FOOD-01…06, R-FARE-04/05/07/08, R-VOUCHER-06/07, R-CANCEL-03; mã bài tại `articles/test-case-design-logic-va-cu-the/`.
- Bài liên quan: [TEST-007 · Quy trình kiểm thử](/articles/quy-trinh-kiem-thu-test-process) · [TEST-010 · Testware](/articles/testware-san-pham-kiem-thu) · [TEST-031 · Equivalence Partitioning](/articles/equivalence-partitioning) · [TEST-032 · Boundary Value Analysis](/articles/boundary-value-analysis) · [TCD-05 · Test data](/articles/test-case-design-test-data) · [TCD-08 · Ma trận truy vết](/articles/test-case-design-traceability-matrix).

Bài tiếp theo: [TCD-04 · Kết quả mong đợi và test oracle](/articles/test-case-design-expected-result-test-oracle).
