---
slug: test-case-design-test-data
title: "Test data cho từng test case: lượt voucher qua nửa đêm, hạn báo giá và thời tiết ở Tới Nơi"
summary: "Thiết kế test data theo từng test case: dữ liệu tiền điều kiện, đầu vào, kết quả mong đợi; giữ các ca độc lập bằng builder thay cho fixture dùng chung; đặt biên ngày theo giờ Việt Nam cho lượt dùng voucher, hạn báo giá đúng bằng expiresAt và thời tiết giả lập. Lab 39 test vitest."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-case-design, tcd-05, test-data, test-data-builder, test-fixture, timezone, voucher, vitest, toi-noi]
language: vi
---

Một test case viết đúng vẫn có thể cho kết quả sai nếu test data của nó sai. Bộ test voucher của Tới Nơi chạy xanh trên máy dev suốt ba tuần. Đến ngày CI đổi sang chạy song song và xáo thứ tự, ca «báo giá với voucher được giảm 20%» bắt đầu đỏ ngẫu nhiên. Không có dòng code sản phẩm nào thay đổi. Ca đó đỏ vì nó chưa từng tự khai báo dữ liệu mình cần: ca chạy đúng nhờ chưa có ca nào khác tiêu lượt voucher trước nó. Bài này dạy cách thiết kế test data **cho từng test case**: tách dữ liệu tiền điều kiện, dữ liệu đầu vào và kết quả mong đợi; giữ các ca độc lập với nhau; chọn giữa builder, fixture và dữ liệu mẫu; và xử lý thời gian sao cho «hôm nay» của test data trùng với «hôm nay» của đặc tả.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TCD-05</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>TCD · Thiết kế Test Case</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1; bối cảnh Advanced — CTAL-TA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Component; System</td></tr>
<tr><th scope="row">Test Type</th><td>Functional</td></tr>
<tr><th scope="row">Test Technique</th><td>Không phải một kỹ thuật riêng; dùng EP và BVA (<a href="/articles/equivalence-partitioning">TEST-031</a>, <a href="/articles/boundary-value-analysis">TEST-032</a>) để chọn giá trị dữ liệu</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/test-case-design-logic-va-cu-the">TCD-03 · Test case logic và test case cụ thể</a>, <a href="/articles/dependency-isolation">TEST-058 · Dependency Isolation</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Test data builder; cùng ba test case chạy theo 6 thứ tự với fixture dùng chung và với builder; bảng TD quanh nửa đêm giờ VN và nửa đêm UTC cho lượt voucher (R-VOUCHER-07, R-GEN-02); hạn báo giá (R-QUOTE-01); thời tiết — <code>examples/toi-noi-testing/articles/test-case-design-test-data/</code> (39 test vitest)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.4.1, 1.4.3 · FL-1.4.1 (K2). CTAL-TA v4.0: bối cảnh</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Ở [TCD-03](/articles/test-case-design-logic-va-cu-the), một test case logic trở thành test case cụ thể khi mỗi «giá trị hợp lệ», «một chuyến đã dùng voucher» được thay bằng một giá trị thật. Bước thay thế đó sinh ra **test data**, và chất lượng của bước này quyết định ba điều:

- **Test case có kiểm đúng điều nó định kiểm không.** Ca «khách đã hết lượt voucher» mà không có chuyến nào dùng voucher trong kho thì không kiểm gì về lượt dùng. Nó chỉ tình cờ đỏ hoặc xanh.
- **Test case có tái hiện được không.** Kết quả phụ thuộc giờ máy, múi giờ của CI, thứ tự chạy hay dữ liệu ca khác để lại thì không ai tái hiện được lần đỏ hôm qua.
- **Test case có an toàn không.** Dữ liệu lấy từ môi trường thật (tên, số điện thoại, lịch sử chuyến của khách thật) đưa vào bộ test là rủi ro rò rỉ dữ liệu cá nhân, dù bộ test chỉ chạy nội bộ.

Bài này không bàn chuyện quản lý test data cấp tổ chức (kho dữ liệu dùng chung cho nhiều đội, che dữ liệu production, làm mới môi trường staging). Phạm vi là **một test case cần dữ liệu gì và lấy dữ liệu đó từ đâu**. Domain là ba quy tắc của Tới Nơi mà dữ liệu và thời gian đan vào nhau chặt nhất: lượt dùng voucher trong ngày (R-VOUCHER-07), giờ nghiệp vụ theo giờ Việt Nam (R-GEN-02) và hạn báo giá (R-QUOTE-01), cộng thời tiết như một nguồn dữ liệu môi trường.

## 2. Lý thuyết theo ISTQB

### 2.1. Test data trong các hoạt động kiểm thử — CTFL v4.0.1 mục 1.4.1

CTFL v4.0.1 mô tả quy trình kiểm thử bằng các nhóm hoạt động. Test data xuất hiện ở hai nhóm liền nhau:

- **Test design** biến test condition thành test case và các testware khác. Theo syllabus, hoạt động này gồm cả việc **xác định yêu cầu về test data** cần cho test case, cùng với thiết kế môi trường test. Ở bước này ta trả lời câu «ca TC-VU-003 cần một khách đã có một chuyến chưa huỷ dùng voucher trong ngày» — chưa cần biết khách là ai, chuyến lúc mấy giờ.
- **Test implementation** tạo hoặc lấy testware cần cho việc chạy test, trong đó có **test data**, và kiểm rằng môi trường cùng dữ liệu đã sẵn sàng. Ở bước này yêu cầu ở trên thành dữ liệu cụ thể: khách `C901`, chuyến dùng `THU20` lúc 08:30 ngày 16/11/2026 giờ Việt Nam.

LO liên quan là **FL-1.4.1 (K2)**: tóm tắt các hoạt động và nhiệm vụ kiểm thử. Bài không đòi hỏi bạn thuộc danh sách nhiệm vụ. Điều cần nắm là test data được **thiết kế** trước, rồi mới **dựng**. Hai bước này nằm ở hai hoạt động khác nhau và sinh ra hai sản phẩm khác nhau.

### 2.2. Test data là testware — mục 1.4.3

Mục 1.4.3 liệt kê testware theo hoạt động sinh ra nó. Sản phẩm của test design có **yêu cầu về test data** (test data requirements) và yêu cầu về môi trường. Sản phẩm của test implementation có **test data**, test procedure, test suite, test execution schedule và các thành phần môi trường. Gọi test data là testware có hệ quả thực tế: test data phải được **quản lý cấu hình, review và bảo trì** như test case. Một tệp seed sửa tay không ai review, một bản ghi trong staging «đừng xoá, ca 47 cần» đều là testware không được quản lý.

### 2.3. Ba lớp dữ liệu của một test case

ISTQB không chia test data thành các lớp có tên riêng. Bài dùng cách chia sau vì nó khớp với các trường của test case ở [TCD-02](/articles/test-case-design-giai-phau-test-case):

| Lớp | Trả lời câu hỏi | Ví dụ với TC-VU-003 |
|---|---|---|
| Dữ liệu tiền điều kiện (precondition data) | Hệ thống phải ở trạng thái nào **trước** khi chạy bước? | voucher `THU20` 1 lượt/ngày; khách `C901`; một chuyến của `C901` dùng `THU20` lúc 08:30 ngày 16/11, chưa huỷ |
| Dữ liệu môi trường (environment data) | Các nguồn bên ngoài trả gì tại thời điểm chạy? | đồng hồ ở 09:00:00 giờ VN ngày 16/11; TP. Hồ Chí Minh không mưa |
| Dữ liệu đầu vào (input data) | Bước test gửi gì? | `{ kind: "ride", cityCode: "SG", vehicle: "bike", distanceKm: 7.4, voucherCode: "THU20" }` |
| Dữ liệu mong đợi (expected data) | So kết quả với gì? | lỗi `VOUCHER_USAGE_LIMIT` (422 qua API) |

Dữ liệu môi trường thật ra là một phần của tiền điều kiện. Bài tách riêng vì ở Tới Nơi, đồng hồ và thời tiết không nằm trong database mà đến từ cổng (`Clock`, `WeatherService`) — cách dựng khác hẳn. Kết quả mong đợi, theo [TCD-04](/articles/test-case-design-expected-result-test-oracle), phải suy từ test oracle, không chép từ lần chạy trước. Ở bài này, «dữ liệu mong đợi» là phần kết quả mong đợi có dạng dữ liệu: số tiền, mã lỗi, chuỗi thời điểm ISO.

### 2.4. CTAL-TA v4.0 — bối cảnh

CTAL-TA v4.0 bàn kỹ hơn phần việc của Test Analyst trong test design và test implementation, trong đó có chuẩn bị test data và kiểm tra rằng dữ liệu sẵn sàng trước khi chạy. Bài chỉ dùng tài liệu này làm bối cảnh: nội dung kỹ thuật (builder, đồng hồ giả, biên ngày) là thực hành kỹ nghệ, không gắn với LO nào của CTAL-TA.

### 2.5. Thuật ngữ Anh – Việt

| Thuật ngữ | Dạng dùng trong bài | Ghi chú |
|---|---|---|
| test data | test data | không dịch; «dữ liệu kiểm thử» cũng chấp nhận |
| test data requirements | yêu cầu về test data | sản phẩm của test design |
| precondition | tiền điều kiện | dữ liệu tiền điều kiện = dữ liệu dựng trạng thái đó |
| expected result | kết quả mong đợi | |
| test fixture | fixture | ngoài ISTQB; trạng thái cố định dựng sẵn cho test |
| test data builder | builder | ngoài ISTQB; Freeman & Pryce, 2009 |
| seed data | dữ liệu mẫu | Phụ lục A của SPEC |
| synthetic data | dữ liệu giả lập | không lấy từ người thật |
| order-dependent test | test phụ thuộc thứ tự | kết quả đổi khi đổi thứ tự chạy |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Các mẫu «test data builder» (Nat Pryce; Freeman & Pryce, <i>Growing Object-Oriented Software, Guided by Tests</i>, 2009) và «object mother» (Martin Fowler) là thực hành kỹ nghệ phần mềm, không có trong CTFL. Bài dùng tên tiếng Anh vì chưa có tên Việt quen dùng.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Voucher là công cụ khuyến mãi chính của Tới Nơi, và cũng là nơi bị lạm dụng nhiều nhất: một khách tạo liên tục chuyến rồi huỷ để giữ lượt, hoặc canh nửa đêm để dùng hai lượt trong vài giây. Quy tắc lượt dùng vì vậy phụ thuộc cả **lịch sử chuyến** của khách lẫn **ngày nào** theo giờ Việt Nam. Một test case cho quy tắc này không thể chỉ có «đầu vào»: phần lớn công sức nằm ở việc dựng lịch sử chuyến đúng như ca cần.

### 3.2. Business rules dùng trong bài

| Mã | Quy tắc |
|---|---|
| R-GEN-02 | Mọi quy tắc theo giờ hoặc theo ngày dùng **giờ Việt Nam (UTC+7, không có giờ mùa hè)**, bất kể múi giờ máy chủ. «Thời điểm đặt» là thời điểm hệ thống nhận yêu cầu, theo đồng hồ của hệ thống. |
| R-VOUCHER-01 | `perCustomerPerDay`: số lần mỗi khách được dùng trong một ngày, mặc định 1. |
| R-VOUCHER-04 | Ngày đặt (giờ VN) ngoài [`validFrom`; `validTo`], tính cả hai đầu ⇒ `VOUCHER_EXPIRED`. |
| R-VOUCHER-07 | Số chuyến **chưa bị huỷ** khách đã tạo với mã đó **trong cùng ngày (giờ VN)** phải nhỏ hơn `perCustomerPerDay`; ngược lại ⇒ `VOUCHER_USAGE_LIMIT`. Báo giá không tiêu lượt, tạo chuyến tiêu lượt, chuyến bị huỷ trả lại lượt. Lượt được kiểm **hai lần**: khi báo giá và khi tạo chuyến, ngày tính theo thời điểm của từng lần kiểm. |
| R-QUOTE-01 | `expiresAt = createdAt + 5 phút`; báo giá còn hiệu lực khi **now < `expiresAt`**; đúng bằng `expiresAt` là đã hết hạn. |
| R-QUOTE-04 | Chuyến giữ ảnh chụp giá của báo giá; thời tiết đổi sau khi báo giá không làm đổi giá. |
| R-GEN-05, R-FARE-05 | «Đang mưa» do dịch vụ thời tiết cung cấp tại thời điểm đặt; mưa ⇒ cộng 8.000 đ. |
| Phụ lục A | Dữ liệu mẫu của máy chủ lab: khách `C001`–`C003`, voucher `GIAM20`, `FOOD15`, `HN10`, `XMAS50`. |

Giá dùng xuyên bài: đặt xe máy 7,4 km ở TP. Hồ Chí Minh ngoài giờ cao điểm, không mưa. Theo R-FARE-01: 12.000 + 4.500 × 7,4 = 45.300 đ, làm tròn lên **46.000 đ**. Với voucher 20% trần 30.000 đ: tiền giảm 9.060 đ, còn 36.240 đ, làm tròn lên **37.000 đ**. Trời mưa: 45.300 + 8.000 = 53.300 đ ⇒ **54.000 đ**.

### 3.3. System context

Trong kiến trúc đích, lượt dùng voucher do **Promotion Service** quản lý, lịch sử chuyến nằm ở **Booking Service**, đồng hồ là đồng hồ của từng service (đồng bộ NTP), thời tiết từ một nhà cung cấp bên ngoài. Lab gom tất cả vào một tiến trình: `assertVoucherQuota` ở `src/app/quote.ts` đếm chuyến qua `trips.countVoucherUses` trong khoảng ngày do `vnDayRange` ở `src/domain/time.ts` tính; đồng hồ và thời tiết là hai cổng được inject (`Clock`, `WeatherService`) với bản giả `FixedClock` và `FakeWeather` (R-ARCH-03). Hai cổng đó chính là chỗ test data «môi trường» đi vào.

### 3.4. Rủi ro

| Rủi ro về test data | Hậu quả | Cách bài xử lý |
|---|---|---|
| Ca dựa vào dữ liệu ca khác để lại | Ca đỏ hoặc xanh tuỳ thứ tự chạy; không tái hiện được | Mỗi ca tự dựng tiền điều kiện (mục 4.6, 5.3) |
| Dựng thời điểm test bằng giờ máy | Ca xanh trên máy dev ở Việt Nam, đỏ trên CI chạy UTC | Thời điểm là dữ liệu tường minh, có offset (mục 4.3) |
| Chỉ thử dữ liệu lúc 09:00 | Không bao giờ chạm biên ngày; lỗi «ngày UTC» lọt | Bảng TD quanh hai mốc nửa đêm (mục 4.3) |
| Tiền điều kiện dựng thẳng vào database, sai với nghiệp vụ | Ca kiểm một trạng thái hệ thống không bao giờ đạt được | Dựng tiền điều kiện qua use case (mục 4.7) |
| Dùng dữ liệu cá nhân thật làm test data | Rò rỉ dữ liệu, vi phạm quy định bảo vệ dữ liệu | Dữ liệu giả lập có dấu hiệu nhận biết (mục 4.8) |

## 4. Phân tích kỹ thuật

### 4.1. Từ test condition tới yêu cầu về test data

Lấy các test condition của R-VOUCHER-07 (cách đặt tên theo [TCD-01](/articles/test-case-design-test-analysis-test-condition)) và viết yêu cầu về test data cho từng condition, **chưa chọn giá trị cụ thể**:

| Test condition | Yêu cầu về test data (sản phẩm của test design) |
|---|---|
| TCOND-VU-01 · khách chưa dùng lượt nào trong ngày được giảm | voucher 1 lượt/ngày; khách không có chuyến nào dùng mã đó trong ngày |
| TCOND-VU-02 · khách đã dùng hết lượt trong ngày bị từ chối | voucher k lượt/ngày; khách có đúng k chuyến chưa huỷ dùng mã đó trong ngày |
| TCOND-VU-03 · chuyến bị huỷ trả lại lượt | như trên nhưng một trong các chuyến đã bị huỷ |
| TCOND-VU-04 · lượt tính theo từng khách | một khách **khác** đã dùng hết lượt; khách của ca chưa dùng |
| TCOND-VU-05 · «ngày» là ngày giờ Việt Nam | chuyến và báo giá ở hai phía của nửa đêm giờ VN; và ở hai phía nửa đêm UTC |
| TCOND-VU-06 · lượt được kiểm lại khi tạo chuyến | hai báo giá cùng dùng mã; chuyến thứ nhất tạo trước, chuyến thứ hai tạo sau |

Viết yêu cầu trước khi chọn giá trị giúp thấy ngay ba điều. Thứ nhất, **tiền điều kiện chiếm phần lớn**: đầu vào của sáu condition gần như giống nhau (một yêu cầu báo giá có voucher), chỗ khác nhau nằm ở lịch sử chuyến và đồng hồ. Thứ hai, có **phân vùng trên chính dữ liệu tiền điều kiện**: chuyến của chính khách hay của khách khác, chưa huỷ hay đã huỷ, trong ngày hay ngoài ngày. Đó là EP ([TEST-031](/articles/equivalence-partitioning)) áp lên lịch sử, không phải lên trường JSON. Thứ ba, có **biên**: số lượt đã dùng so với `perCustomerPerDay` (k−1 và k), và mốc nửa đêm. Đó là BVA ([TEST-032](/articles/boundary-value-analysis)) áp lên số đếm và thời gian.

### 4.2. Ba lớp dữ liệu trong một test case cụ thể

Viết TC-VU-003 thành test case cụ thể theo các trường của [TCD-02](/articles/test-case-design-giai-phau-test-case), chỉ giữ phần dữ liệu:

| Trường | Giá trị |
|---|---|
| Tiền điều kiện | Voucher `THU20`: 20%, trần 30.000 đ, tối thiểu 0 đ, mọi dịch vụ, mọi thành phố, 01/01/2026–31/12/2030, 1 lượt/ngày. Khách `C901` «Khách Thử 01», số 0900000901. Chuyến của `C901` tạo lúc 16/11/2026 08:30:00 giờ VN bằng `THU20`, trạng thái `searching`. |
| Môi trường | Đồng hồ: 16/11/2026 09:00:00 giờ VN (02:00:00Z). Không thành phố nào mưa. |
| Đầu vào | Báo giá `ride`, `SG`, `bike`, 7,4 km, `voucherCode: "THU20"` |
| Kết quả mong đợi | `VOUCHER_USAGE_LIMIT` |

Chú ý hai điều về cách ghi. Thời điểm luôn ghi **kèm múi giờ** (giờ VN và UTC); «08:30» không kèm múi giờ là dữ liệu mơ hồ. Và dữ liệu tiền điều kiện được ghi **đủ để dựng lại**, không ghi «một chuyến đã dùng voucher» chung chung. Nếu thiếu thông tin chuyến lúc 08:30, người khác dựng lại có thể đặt chuyến lúc 23:00 hôm trước và ca sẽ đổi ý nghĩa.

### 4.3. Biên ngày theo giờ Việt Nam

«Trong cùng ngày giờ Việt Nam» là một phân vùng có thứ tự trên trục thời gian: ngày 16/11 giờ VN là khoảng nửa hở [16/11 00:00:00 +07; 17/11 00:00:00 +07), tức [15/11 17:00:00Z; 16/11 17:00:00Z). Có **hai** mốc nửa đêm đáng thử, và chỉ một mốc là biên của đặc tả:

- **Nửa đêm giờ VN** (17:00:00Z) là biên thật của R-VOUCHER-07. Dữ liệu ở hai phía mốc này phải cho hai kết quả khác nhau.
- **Nửa đêm UTC** (07:00:00 giờ VN) **không** phải biên của đặc tả. Nó là biên của một **lỗi dễ mắc**: cài đặt dùng ngày UTC (ví dụ `toISOString().slice(0, 10)`) thay vì ngày giờ VN. Dữ liệu ở hai phía mốc này phải cho **cùng** kết quả. Nếu kết quả khác nhau thì hệ thống đang tính ngày theo UTC.

Mốc thứ hai là một ví dụ BVA dựa trên rủi ro: ta thêm giá trị biên không phải vì đặc tả có biên ở đó, mà vì một cách cài đặt sai có biên ở đó. Đây cũng là tinh thần của error guessing ([TEST-051](/articles/error-guessing)).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Hai trục thời gian song song từ 22 giờ ngày 15/11 tới 8 giờ ngày 16/11 giờ Việt Nam. Trục trên là ngày theo giờ Việt Nam, đổi ngày lúc 00:00 giờ Việt Nam bằng 17:00Z. Trục dưới là ngày UTC, đổi ngày lúc 07:00 giờ Việt Nam bằng 00:00Z. Các điểm dữ liệu TD-01 và TD-02 nằm hai bên nửa đêm giờ Việt Nam, TD-03 nằm đúng nửa đêm giờ Việt Nam, TD-04 nằm hai bên nửa đêm UTC.">
<defs>
<marker id="tcd05-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-b">«Hôm nay» của R-VOUCHER-07 là ngày giờ Việt Nam, không phải ngày UTC</text>
<text x="20" y="66" class="t-small t-b">Giờ VN</text>
<rect x="100" y="48" width="140" height="30" rx="4" class="f-warn-soft s-warn"/>
<text x="170" y="68" text-anchor="middle" class="t-small t-warn">ngày 15/11</text>
<rect x="240" y="48" width="640" height="30" rx="4" class="f-phys-soft s-phys"/>
<text x="560" y="68" text-anchor="middle" class="t-small t-phys">ngày 16/11 (giờ VN)</text>
<text x="20" y="226" class="t-small t-b">UTC</text>
<rect x="100" y="208" width="630" height="30" rx="4" class="f-warn-soft s-warn"/>
<text x="415" y="228" text-anchor="middle" class="t-small t-warn">ngày 15/11 (UTC)</text>
<rect x="730" y="208" width="150" height="30" rx="4" class="f-pk-soft s-pk"/>
<text x="805" y="228" text-anchor="middle" class="t-small t-pk">16/11 (UTC)</text>
<line x1="240" y1="40" x2="240" y2="246" class="s-dead dash" stroke-width="1.6"/>
<line x1="730" y1="40" x2="730" y2="246" class="s-pk dash" stroke-width="1.6"/>
<text x="240" y="264" text-anchor="middle" class="t-10 t-mono">00:00 VN = 17:00Z</text>
<text x="730" y="264" text-anchor="middle" class="t-10 t-mono">07:00 VN = 00:00Z</text>
<text x="100" y="100" text-anchor="middle" class="t-10 t-mono">22:00</text>
<text x="880" y="100" text-anchor="middle" class="t-10 t-mono">08:00</text>
<circle cx="232" cy="130" r="6" class="f-warn s-warn"/>
<text x="226" y="122" text-anchor="end" class="t-small t-b">TD-01 · 23:59:59 ⇒ hết lượt</text>
<circle cx="248" cy="150" r="6" class="f-phys s-phys"/>
<text x="256" y="154" class="t-small t-b">TD-02 · 00:00:00 ⇒ được</text>
<circle cx="240" cy="178" r="6" class="f-bg s-dead" stroke-width="2"/>
<text x="250" y="182" class="t-small">TD-03 · chuyến đúng 00:00:00 vẫn tính cho 16/11</text>
<circle cx="722" cy="130" r="6" class="f-warn s-warn"/>
<circle cx="738" cy="130" r="6" class="f-warn s-warn"/>
<text x="716" y="122" text-anchor="end" class="t-small t-b">TD-04 · chuyến 06:59:59, báo giá 07:00:00 ⇒ vẫn hết lượt</text>
<text x="460" y="290" text-anchor="middle" class="t-small">Vạch đỏ: biên của đặc tả. Vạch tím: biên của lỗi «tính ngày theo UTC» — hai phía phải cho cùng kết quả.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Hai mốc nửa đêm, một biên thật.</b> Trục không theo tỉ lệ ở hai đầu. TD-02 và TD-04 là hai dòng duy nhất phân biệt được bản cài đặt đúng với bản tính ngày theo UTC (mục 5.4).</figcaption>
</figure>

Độ chính xác của thời gian là **1 giây**: SPEC không nói tới mili giây, và các ví dụ biên giờ của SPEC (R-FARE-04) đều ở mức giây. Đây là giả định của người test, ghi rõ trong test design như [TEST-032](/articles/boundary-value-analysis) khuyên.

### 4.4. Bảng TD

Gộp phân vùng trên lịch sử chuyến (mục 4.1) với biên ngày (mục 4.3) thành bảng dữ liệu. Mọi dòng dùng cùng đầu vào (báo giá `THU20`, 7,4 km, xe máy, `SG`), chỉ khác tiền điều kiện và đồng hồ. Kết quả «được» nghĩa là báo giá thành công với tiền giảm 9.060 đ, tổng 37.000 đ.

| Dòng | Test condition | Tiền điều kiện (giờ VN) | Đồng hồ lúc báo giá (giờ VN) | Mong đợi |
|---|---|---|---|---|
| TD-01 | VU-05 | chuyến của khách lúc 15/11 23:59:58 | 15/11 23:59:59 | `VOUCHER_USAGE_LIMIT` |
| TD-02 | VU-05 | chuyến của khách lúc 15/11 23:59:58 | 16/11 00:00:00 | được |
| TD-03 | VU-05 | chuyến của khách lúc 16/11 00:00:00 | 16/11 00:00:01 | `VOUCHER_USAGE_LIMIT` |
| TD-04 | VU-05 | chuyến lúc 16/11 06:59:59 (= 15/11 23:59:59Z) | 16/11 07:00:00 (= 16/11 00:00:00Z) | `VOUCHER_USAGE_LIMIT` |
| TD-05 | VU-03 | chuyến lúc 15/11 23:59:58, **đã bị khách huỷ** | 15/11 23:59:59 | được |
| TD-06 | VU-04 | **khách khác** (`C902`) có chuyến lúc 16/11 08:30 | 16/11 09:00:00 | được |
| TD-07 | VU-01 | voucher **2 lượt/ngày**, đã dùng 1 lượt lúc 08:00 | 16/11 09:00:00 | được |
| TD-08 | VU-02 | voucher 2 lượt/ngày, đã dùng 2 lượt lúc 08:00 và 08:30 | 16/11 09:00:00 | `VOUCHER_USAGE_LIMIT` |

TD-03 kiểm đầu dưới của khoảng nửa hở: chuyến tạo đúng 00:00:00 thuộc ngày mới. TD-07 và TD-08 là cặp biên trên số đếm (k−1 = 1, k = 2). Một bộ test chỉ có «1 lượt/ngày» sẽ không phân biệt được `used < perDay` với `used < 1`.

### 4.5. Kiểm lần hai khi tạo chuyến qua nửa đêm

R-VOUCHER-07 nói lượt được kiểm **hai lần**, ngày tính theo thời điểm của từng lần kiểm. Câu này sinh ra một tình huống mà test data phải dựng theo **trình tự thời gian**, không chỉ là một ảnh chụp trạng thái:

| Bước | Giờ VN | Hành động | Kết quả |
|---|---|---|---|
| 1 | 15/11 23:57:00 | báo giá Q1 với `THU20` | được (chưa dùng lượt) |
| 2 | 15/11 23:57:30 | báo giá Q2 với `THU20` | được (báo giá không tiêu lượt) |
| 3 | 15/11 23:58:00 | tạo chuyến từ Q1 | được, tiêu lượt ngày 15/11 |
| 4a · TD-09 | 15/11 23:59:00 | tạo chuyến từ Q2 | `VOUCHER_USAGE_LIMIT` |
| 4b · TD-10 | 16/11 00:00:00 | tạo chuyến từ Q2 (còn hạn tới 00:02:30) | được; chuyến có `requestedAt` 16/11 nên tiêu lượt ngày 16/11 |
| 5 (sau 4b) | 16/11 00:01:00 | báo giá thêm với `THU20` | `VOUCHER_USAGE_LIMIT` |

TD-10 cho thấy một hệ quả của đặc tả mà người đọc lướt dễ bỏ qua: báo giá lập ngày 15/11 nhưng chuyến tạo ngày 16/11 tiêu lượt của **ngày 16/11**. Hệ quả này hợp lý với câu chữ của SPEC, nhưng vẫn đáng xác nhận với BA, vì khách sẽ thấy «hôm nay» mình đã hết lượt ngay từ phút đầu tiên.

### 4.6. Độc lập giữa các test case

Một test case **độc lập** nếu kết quả của nó chỉ phụ thuộc dữ liệu chính nó khai báo. Có hai kiểu phụ thuộc phá vỡ điều đó:

- **Rò dữ liệu (leak).** Ca A ghi dữ liệu và ca B vô tình thấy dữ liệu đó. Ví dụ: TC-VU-002 tạo chuyến dùng `THU20`, sau đó TC-VU-001 («khách chưa dùng lượt nào») chạy trên cùng kho và nhận `VOUCHER_USAGE_LIMIT`.
- **Thiếu tiền điều kiện (hidden precondition).** Ca B **cần** dữ liệu mà chỉ ca A tạo ra. Ví dụ: TC-VU-003 («khách đã hết lượt») chỉ đỏ đúng khi TC-VU-002 chạy trước nó.

Hai kiểu này cần hai cách sửa khác nhau, và lab cho thấy rõ điều đó (mục 5.3): thêm bước dọn dữ liệu (`reset()` trước mỗi ca) chữa được rò dữ liệu nhưng **không** chữa được thiếu tiền điều kiện. Ca thiếu tiền điều kiện chỉ chữa được bằng cách để chính ca đó khai báo dữ liệu nó cần.

Phụ thuộc giữa test case không phải lúc nào cũng sai. [TCD-03](/articles/test-case-design-logic-va-cu-the) bàn các test procedure có chủ đích nối nhiều bước (báo giá → đặt chuyến → nhận → huỷ). Điều cần tránh là phụ thuộc **ngầm**: không ghi trong test case, không thể hiện trong test execution schedule, chỉ đúng nhờ thứ tự mặc định của công cụ chạy test.

### 4.7. Builder, fixture và dữ liệu mẫu

Có ba cách phổ biến để có dữ liệu tiền điều kiện. Không cách nào luôn đúng.

| Cách | Mô tả | Hợp với | Rủi ro |
|---|---|---|---|
| Dữ liệu mẫu (seed) | Một bộ dữ liệu cố định nạp vào hệ thống — Phụ lục A: `C001`, `GIAM20`… | smoke test, demo, E2E đọc dữ liệu tham chiếu, trải nghiệm thủ công | mọi ca dùng chung; sửa seed làm đổi nghĩa nhiều ca; ca không nói rõ nó cần gì từ seed |
| Fixture dùng chung | Dựng một lần cho cả bộ (`beforeAll`) rồi các ca dùng chung | dữ liệu **chỉ đọc**, đắt để dựng (PGlite, container) | rò dữ liệu khi ca ghi; thiếu tiền điều kiện khi ca dựa vào ca khác |
| Builder | Mỗi ca gọi `aWorld().withVoucherTrip(...).at(...).build()` khai báo đúng thứ nó cần, mặc định hợp lý cho phần còn lại | dữ liệu **có ghi**, nhiều biến thể nhỏ quanh một mặc định | builder tự nó có thể sai; dựng mỗi ca tốn thời gian hơn |

Builder có hai đặc điểm làm nó hợp với test data «cho từng test case». Một, **mặc định hợp lý**: ca chỉ ghi phần khác mặc định, nên đọc test là thấy ngay dữ liệu nào quan trọng với ca đó (TD-05 chỉ khác TD-01 ở `cancelled: true`). Hai, **mỗi lần gọi trả object mới**: không có chỗ cho rò dữ liệu qua tham chiếu dùng chung.

Một quyết định thiết kế quan trọng của builder trong lab: chuyến tiền điều kiện được dựng **qua đúng use case nghiệp vụ** (`createQuote` → `createTrip` → `cancelByCustomer`), không chèn bản ghi thẳng vào kho. Dựng qua use case chậm hơn, nhưng bảo đảm trạng thái tiền điều kiện là trạng thái **hệ thống thật sự đạt được**. Lab có một test case cho điều này: khai báo hai chuyến cùng ngày với voucher 1 lượt/ngày thì `build()` thất bại với `VOUCHER_USAGE_LIMIT`, vì nghiệp vụ không cho phép trạng thái đó. Chèn thẳng vào database sẽ «thành công» và cho ra một ca kiểm một trạng thái không tồn tại.

Ngược lại, có lúc **cần** chèn thẳng: kiểm dữ liệu cũ sinh ra từ một phiên bản trước (`pricingVersion` khác), hoặc kiểm hệ thống xử lý dữ liệu hỏng ra sao ([SYS-12 · Data Integrity Test](/articles/system-testing-data-integrity-test)). Khi đó test case phải ghi rõ đây là dữ liệu trái nghiệp vụ có chủ đích.

### 4.8. Dữ liệu giả lập, không dữ liệu thật

Mọi test data trong bài là **dữ liệu giả lập**: khách «Khách Thử 01», số điện thoại 0900000901, mã voucher `THU20`. Ba quy tắc thực hành:

- **Không sao chép dữ liệu cá nhân thật** (họ tên, số điện thoại, địa chỉ, lịch sử chuyến) vào tệp test, seed hay ảnh chụp màn hình, kể cả khi «chỉ chạy nội bộ». Test data được commit vào repository, chép sang máy CI, đính vào báo cáo lỗi — mỗi bản sao là một chỗ có thể rò.
- **Dữ liệu giả lập nên nhận ra được.** Tên có chữ «Thử», mã khách có tiền tố riêng (`C9nn`), số điện thoại trong một dải cố định. Lab có một test case kiểm builder chỉ sinh dữ liệu đúng các mẫu đó. Dải số `0900000xxx` là quy ước của lab, không phải dải số được cơ quan nào dành riêng cho kiểm thử.
- **Dữ liệu giả lập vẫn phải giống thật ở những chỗ quy tắc quan tâm.** Tên khách có dấu tiếng Việt (để lộ lỗi mã hoá ký tự), độ dài nằm trong giới hạn, số tiền đúng đơn vị đồng. Giả lập không có nghĩa là `"test"`, `"abc"` ở mọi trường.

Khi buộc phải dùng dữ liệu bắt nguồn từ production (ví dụ tái hiện một sự cố), cần các kỹ thuật che hoặc ẩn danh hoá dữ liệu và quy trình phê duyệt riêng. Đó là quản lý test data cấp tổ chức, ngoài phạm vi bài này.

### 4.9. Hạn báo giá và thời tiết

**Hạn báo giá (R-QUOTE-01)** là biên thời gian thứ hai trong bài. Hai ca TC-QE-001 và TC-QE-002 dùng hai báo giá **riêng**: nếu dùng chung một báo giá, ca thứ hai sẽ gặp `QUOTE_ALREADY_USED` trước khi kịp kiểm hết hạn — một dạng phụ thuộc ngầm như mục 4.6. Kết quả mong đợi viết bằng chuỗi ISO UTC tường minh (`createdAt` 2026-11-16T02:00:00.000Z, `expiresAt` 2026-11-16T02:05:00.000Z), không tính lại bằng `addMinutes` của sản phẩm.

**Thời tiết** là dữ liệu môi trường: khách không gửi, hệ thống hỏi dịch vụ thời tiết tại thời điểm đặt (R-GEN-05). Vì vậy test data về thời tiết thuộc phần tiền điều kiện của ca, và nên khai báo **theo thành phố**. TC-WX-003 (mưa ở `HN`, báo giá ở `SG` ⇒ 46.000 đ) kiểm đúng điều này: một cài đặt «có thành phố nào mưa là cộng phụ phí trời mưa» sẽ cho 54.000 đ. Ca TC-WX-004 kiểm R-QUOTE-04: trời đổ mưa **sau** khi báo giá thì chuyến vẫn 46.000 đ, `raining: false`.

## 5. Thực hành

### 5.1. Tổ chức mã trong lab

| Tệp | Vai trò | Số test |
|---|---|---|
| `test-data-builder.ts` | `aCustomer`, `aVoucher`, `aRideRequest`, `WorldBuilder` | — |
| `doc-lap-thu-tu.test.ts` | Ba ca lượt dùng chạy theo 6 thứ tự, ba cách dựng dữ liệu | 14 |
| `luot-dung-nua-dem.test.ts` | Bảng TD-01…08, bản lỗi «ngày UTC», TD-09/10 | 11 |
| `het-han-thoi-tiet.test.ts` | Hạn báo giá, thời tiết | 6 |
| `builder.test.ts` | Tính chất của builder | 4 |
| `bai-tap.test.ts` | Kiểm lời giải Bài tập 1 | 4 |

### 5.2. Builder

Trích `test-data-builder.ts` (rút gọn):

```ts
export function aVoucher(overrides: Partial<Voucher> = {}): Voucher {
  return voucher({ code: 'THU20', percent: 20, maxDiscount: 30_000, minAmount: 0, ...overrides });
}

export class WorldBuilder {
  private now = vnTime(2026, 11, 16, 9, 0, 0); // 09:00 giờ VN — ngoài cao điểm
  private readonly customers: Customer[] = [aCustomer(1)];
  private readonly vouchers: Voucher[] = [aVoucher()];
  private readonly priorTrips: PriorTrip[] = [];
  // at(), withCustomer(), withVoucher(), withRain(), withVoucherTrip() …

  async build(): Promise<World> {
    const ordered = [...this.priorTrips].sort((a, b) => a.at.getTime() - b.at.getTime());
    const clock = new FixedClock(ordered[0]?.at ?? this.now);
    const system = createMemorySystem({ clock, seed: { customers: [...this.customers], drivers: labSeed().drivers, vouchers: [...this.vouchers] }, rainingCities: this.raining });
    for (const p of ordered) {                       // dựng lịch sử qua use case thật
      clock.set(p.at);
      const quote = await createQuote(system.deps, actor, aRideRequest({ voucherCode: p.voucherCode }));
      const trip = await createTrip(system.deps, actor, { quoteId: quote.id });
      if (p.cancelled) await cancelByCustomer(system.deps, actor, trip.id);
    }
    clock.set(this.now);                             // rồi mới đặt «bây giờ» của test case
    return { system, deps: system.deps, clock, customer: { role: 'customer', id: this.customers[0]!.id }, priorTrips: trips };
  }
}
```

Một dòng TD đọc gần như câu tiếng Việt:

```ts
const w = await aWorld()
  .withVoucherTrip(vnTime(2026, 11, 15, 23, 59, 58), { cancelled: true })  // TD-05
  .at(vnTime(2026, 11, 15, 23, 59, 59))
  .build();
await expect(createQuote(w.deps, w.customer, aRideRequest({ voucherCode: 'THU20' })))
  .resolves.toMatchObject({ details: { discount: 9_060, total: 37_000 } });
```

Mọi thời điểm dựng bằng `vnTime(...)`, hàm tạo thời điểm từ giờ VN bằng phép tính trên UTC, nên kết quả không phụ thuộc múi giờ máy chạy test.

### 5.3. Fixture dùng chung và thứ tự chạy

`doc-lap-thu-tu.test.ts` viết ba test case **một lần**, rồi chạy chúng theo cả 6 thứ tự với ba cách dựng dữ liệu:

- **Fixture dùng chung:** một hệ thống cho cả bộ, như `beforeAll`.
- **Fixture dùng chung + `reset()`:** như trên, nhưng đưa kho về dữ liệu gốc trước mỗi ca, như `beforeEach`.
- **Builder:** mỗi ca tự dựng; riêng TC-VU-003 khai báo thêm một chuyến lúc 08:30.

| Thứ tự | Fixture dùng chung | + `reset()` trước mỗi ca | Builder |
|---|---|---|---|
| 1-2-3 | 001 ✓ · 002 ✓ · 003 ✓ | 003 ✗ | ✓ ✓ ✓ |
| 1-3-2 | 003 ✗ | 003 ✗ | ✓ ✓ ✓ |
| 2-1-3 | 001 ✗ | 003 ✗ | ✓ ✓ ✓ |
| 2-3-1 | 001 ✗ | 003 ✗ | ✓ ✓ ✓ |
| 3-1-2 | 003 ✗ | 003 ✗ | ✓ ✓ ✓ |
| 3-2-1 | 001 ✗ · 003 ✗ | 003 ✗ | ✓ ✓ ✓ |
| **Thứ tự xanh trọn vẹn** | **1/6** | **0/6** | **6/6** |

Đọc bảng:

- Với fixture dùng chung, ca 001 chỉ đạt khi 002 chạy **sau** nó (rò dữ liệu), ca 003 chỉ đạt khi 002 chạy **trước** nó (thiếu tiền điều kiện). Chỉ thứ tự viết trong tệp, 1-2-3, xanh trọn vẹn. Đó là lý do bộ test «xanh suốt ba tuần» trên máy dev: công cụ chạy test giữ thứ tự khai báo. 6 trong 18 lượt chạy ca bị đỏ.
- Thêm `reset()` chữa được ca 001 nhưng làm ca 003 đỏ **ở mọi thứ tự**. Kết quả này tốt hơn tình trạng trước: lỗi thiết kế test data lộ ra ổn định thay vì đỏ ngẫu nhiên. Nhưng chỉ dọn dữ liệu thì chưa đủ.
- Với builder, cả 6 thứ tự đều xanh. Thân ba test case không đổi một dòng, chỉ đổi nơi lấy dữ liệu.

Vitest có tuỳ chọn `--sequence.shuffle` để xáo thứ tự chạy test case. Lab không dùng tuỳ chọn ngẫu nhiên mà chạy **có hệ thống** cả 6 hoán vị, để kết quả tất định và đếm được.

### 5.4. Bản lỗi «ngày UTC»

`luot-dung-nua-dem.test.ts` chạy bảng TD-01…08 trên cài đặt của lab, rồi chạy cùng bảng trên một hàm đếm lượt **có lỗi cố ý**: thay `vnDayRange` bằng khoảng ngày UTC. Hàm lỗi nằm trong thư mục bài, `src/` không bị sửa.

```ts
async function buggyUtcDayQuotaOk(deps: AppDeps, customerId: string, at: Date, perDay: number) {
  const start = new Date(Date.UTC(at.getUTCFullYear(), at.getUTCMonth(), at.getUTCDate()));
  const end = new Date(start.getTime() + 24 * 3_600_000);
  const used = await deps.trips.countVoucherUses({ customerId, voucherCode: 'THU20', from: start, to: end });
  return used < perDay;
}
```

| Dòng | Mong đợi | Bản lỗi | Bắt được lỗi? |
|---|---|---|---|
| TD-01 | hết lượt | hết lượt (cùng ngày UTC 15/11) | không |
| TD-02 | được | **hết lượt** (17:00:00Z vẫn là 15/11 UTC) | **có** |
| TD-03 | hết lượt | hết lượt | không |
| TD-04 | hết lượt | **được** (00:00:00Z là ngày UTC mới) | **có** |
| TD-05…08 | | giống mong đợi (09:00 VN = 02:00Z, cùng ngày ở cả hai cách tính) | không |

Chỉ TD-02 và TD-04 bắt được lỗi, đúng như phân tích ở mục 4.3. Một bộ test mà mọi dữ liệu đều đặt lúc 09:00 sáng, như phần lớn test case viết tay, không có dòng nào bắt được lỗi này.

### 5.5. Chạy

```bash
cd examples/toi-noi-testing
npx vitest run articles/test-case-design-test-data
```

Kết quả thật (vitest 3.2.7, Node 22, khi viết bài):

```text
 ✓ articles/test-case-design-test-data/bai-tap.test.ts (4 tests)
 ✓ articles/test-case-design-test-data/builder.test.ts (4 tests)
 ✓ articles/test-case-design-test-data/het-han-thoi-tiet.test.ts (6 tests)
 ✓ articles/test-case-design-test-data/luot-dung-nua-dem.test.ts (11 tests)
 ✓ articles/test-case-design-test-data/doc-lap-thu-tu.test.ts (14 tests)

 Test Files  5 passed (5)
      Tests  39 passed (39)
```

Chạy lại với `TZ=UTC` và `TZ=America/Los_Angeles` cho cùng kết quả 39/39: test data thời gian không phụ thuộc múi giờ máy. Lệnh đặt múi giờ trên PowerShell xem README của [TEST-058](/articles/dependency-isolation).

## 6. Góc nhìn nâng cao

**Id sinh tuần tự cũng là dữ liệu dùng chung.** Lab dùng `SequentialIdGenerator`: báo giá và chuyến lần lượt có id `id-0001`, `id-0002`… trong một hệ thống. Với fixture dùng chung, id của báo giá trong TC-VU-001 đổi theo số đối tượng các ca trước đã tạo. Một assertion `expect(quote.id).toBe('id-0001')` là phụ thuộc ngầm thứ ba, khó thấy hơn hai kiểu ở mục 4.6. Kết quả mong đợi nên so id với giá trị đọc được từ chính bước trước, không so với hằng số.

**Dựng qua use case làm ca chậm và kéo theo phụ thuộc.** Builder dựng chuyến tiền điều kiện bằng `createQuote` → `createTrip`. Nếu `createTrip` có lỗi, mọi ca dùng `withVoucherTrip` cùng đỏ, kể cả ca chỉ kiểm báo giá. Đổi lại, ta không bao giờ kiểm một trạng thái không tồn tại. Một cân bằng thường gặp: dựng qua use case ở mức component (rẻ, trong bộ nhớ) và dùng API dữ liệu kiểm thử (`/api/test/*`, R-TEST) ở mức system.

**Fixture dùng chung vẫn có chỗ đứng.** PGlite mất khoảng 0,5–1 giây để khởi tạo (README của lab). Khi dữ liệu dùng chung **chỉ được đọc** (bảng giá, danh mục thành phố, tài xế mẫu), dựng một lần cho cả tệp là hợp lý. Quy tắc: fixture dùng chung cho dữ liệu chỉ đọc; builder cho dữ liệu mà ca ghi hoặc cần biến thể riêng.

**«Bây giờ» phải là dữ liệu, kể cả khi ca không quan tâm giờ.** Ca TC-WX-001 không nói gì về giờ, nhưng nếu đồng hồ là đồng hồ thật thì ca chạy lúc 11:30 sẽ thấy giá cao điểm 57.000 đ thay vì 46.000 đ. Builder đặt mặc định 09:00 giờ VN, ngoài cao điểm, cho đúng lý do đó. Mặc định của builder là một quyết định test design, nên được ghi trong tài liệu test, không chỉ trong code.

**Môi trường test dùng chung là fixture dùng chung cỡ lớn.** Staging mà nhiều đội cùng ghi dữ liệu gặp đúng hai kiểu phụ thuộc ở mục 4.6, ở quy mô lớn hơn và khó dọn hơn. Cách giảm thiểu: mỗi lần chạy tạo dữ liệu riêng có tiền tố của lần chạy (ví dụ mã khách theo id lần chạy), và không bao giờ viết ca dựa vào bản ghi «có sẵn» mà ca không tự tạo.

**Sai lầm thường gặp:**

| Sai lầm | Hậu quả | Cách tránh |
|---|---|---|
| Ghi «một chuyến đã dùng voucher» mà không ghi lúc nào | Người dựng lại chọn giờ khác, ca đổi nghĩa | Ghi thời điểm kèm múi giờ |
| Dùng `new Date()` hoặc `new Date(2026, 10, 16, 9)` trong test | Kết quả phụ thuộc giờ và múi giờ máy chạy | Đồng hồ được inject, thời điểm dựng từ UTC hoặc có offset |
| Chỉ thử dữ liệu «giữa ngày» | Lỗi ở biên ngày lọt | Thêm cặp biên quanh nửa đêm VN và nửa đêm UTC |
| `reset()` rồi tin rằng các ca đã độc lập | Ca thiếu tiền điều kiện vẫn sai | Mỗi ca tự khai báo tiền điều kiện |
| Object test data khai báo `const` ở đầu tệp rồi sửa trong ca | Rò sang ca sau (lab: `builder.test.ts`) | Hàm trả object mới mỗi lần gọi |
| Chép dữ liệu khách thật để «giống thật» | Rủi ro rò rỉ dữ liệu cá nhân | Dữ liệu giả lập có dấu hiệu nhận biết |

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Biên ngày hiệu lực voucher

Voucher `XMAS50` (Phụ lục A) hiệu lực 24/12/2025–26/12/2025, tính cả hai đầu, theo giờ Việt Nam (R-VOUCHER-04). Thiết kế bốn dòng test data 2-value BVA (độ chính xác 1 giây) cho hai biên của khoảng hiệu lực, ghi thời điểm báo giá theo giờ VN **và** UTC. Với báo giá xe máy 7,4 km ngoài cao điểm, không mưa, kết quả mong đợi của các dòng hợp lệ là bao nhiêu?

<details><summary><b>Lời giải</b></summary>
<ul>
<li>23/12/2025 23:59:59 giờ VN (= 16:59:59Z) ⇒ <code>VOUCHER_EXPIRED</code>.</li>
<li>24/12/2025 00:00:00 giờ VN (= 23/12 17:00:00Z) ⇒ hợp lệ.</li>
<li>26/12/2025 23:59:59 giờ VN (= 16:59:59Z) ⇒ hợp lệ.</li>
<li>27/12/2025 00:00:00 giờ VN (= 26/12 17:00:00Z) ⇒ <code>VOUCHER_EXPIRED</code>.</li>
<li>Dòng hợp lệ: 45.300 × 50% = 22.650 đ, dưới trần 50.000 đ ⇒ tiền giảm 22.650 đ, còn 22.650 đ, trên cước tối thiểu 15.000 đ ⇒ làm tròn lên <b>23.000 đ</b>.</li>
<li>Bẫy hay gặp: viết «26/12 23:59:59» mà không ghi múi giờ. Trên máy chạy UTC, chuỗi đó là 27/12 06:59:59 giờ VN và dòng «hợp lệ» thành hết hạn. Lab kiểm bốn dòng này trong <code>bai-tap.test.ts</code>, gửi mã dạng <code>"xmas50 "</code> để kiểm luôn R-VOUCHER-02.</li>
</ul>
</details>

### Bài tập 2 — Sửa bộ test dùng fixture chung

Một đồng nghiệp thấy bảng ở mục 5.3 và đề xuất: «giữ fixture dùng chung, nhưng thêm `beforeEach(reset)` và đặt TC-VU-002 chạy ngay trước TC-VU-003 bằng cách gộp hai ca vào cùng một `describe`». Đề xuất này chữa được những gì, còn để lại vấn đề gì? Đề xuất cách sửa của bạn.

<details><summary><b>Lời giải</b></summary>
<ul>
<li><code>reset()</code> chữa được rò dữ liệu: TC-VU-001 đạt ở mọi thứ tự. Nhưng <code>reset()</code> chạy trước <b>mỗi</b> ca, nên dữ liệu TC-VU-002 tạo ra bị xoá trước khi TC-VU-003 chạy. Lab cho thấy với reset, TC-VU-003 trượt ở cả 6 thứ tự. Đặt hai ca cạnh nhau không giúp gì.</li>
<li>Nếu bỏ reset giữa 002 và 003 để «giữ» dữ liệu, ta quay lại phụ thuộc ngầm: chạy riêng TC-VU-003 (ví dụ <code>-t "TC-VU-003"</code> khi gỡ lỗi) sẽ đỏ.</li>
<li>Cách sửa: TC-VU-003 tự khai báo tiền điều kiện «đã có một chuyến dùng THU20 lúc 08:30». Nếu thật sự muốn kiểm chuỗi «đặt chuyến rồi hết lượt», viết thành <b>một</b> test procedure có hai bước và ghi rõ trong test case, như TD-09/TD-10.</li>
</ul>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Theo CTFL v4.0.1 mục 1.4.1 và 1.4.3, phát biểu nào đúng về test data?

- a) Test data chỉ xuất hiện ở test execution
- b) Test design xác định yêu cầu về test data; test implementation tạo hoặc lấy test data
- c) Test data không phải testware vì nó thay đổi theo từng lần chạy
- d) Test analysis là hoạt động tạo test data

<details><summary><b>Đáp án: b</b></summary>
<ul><li>a) Sai — test data được thiết kế và dựng trước khi chạy.</li><li>b) Đúng — yêu cầu về test data là sản phẩm của test design; test data là sản phẩm của test implementation.</li><li>c) Sai — mục 1.4.3 liệt kê test data trong testware.</li><li>d) Sai — test analysis xác định test condition («kiểm cái gì»), chưa tới dữ liệu.</li></ul>
</details>

**Câu 2 (K2).** Bộ ba test case xanh khi chạy theo thứ tự trong tệp nhưng đỏ khi xáo thứ tự. Thêm bước đưa dữ liệu về trạng thái gốc trước mỗi ca thì một ca đỏ ở mọi thứ tự. Kết luận hợp lý nhất?

- a) Bước dọn dữ liệu có lỗi, nên bỏ đi
- b) Ca đó thiếu tiền điều kiện: nó dựa vào dữ liệu ca khác tạo ra
- c) Công cụ chạy test có lỗi khi xáo thứ tự
- d) Sản phẩm có lỗi race condition

<details><summary><b>Đáp án: b</b></summary>
<ul><li>a) Sai — bước dọn dữ liệu làm lộ lỗi một cách ổn định; bỏ đi chỉ giấu lỗi.</li><li>b) Đúng — mục 4.6 và bảng 5.3.</li><li>c) Sai — kết quả đổi theo thứ tự là dấu hiệu của phụ thuộc trong test data.</li><li>d) Sai — các ca chạy tuần tự; không có truy cập đồng thời.</li></ul>
</details>

**Câu 3 (K3).** Theo R-GEN-02, thời điểm nào là **đầu** ngày 16/11/2026 theo giờ nghiệp vụ của Tới Nơi?

- a) 2026-11-16T00:00:00Z
- b) 2026-11-15T17:00:00Z
- c) 2026-11-16T07:00:00Z
- d) Phụ thuộc múi giờ máy chủ

<details><summary><b>Đáp án: b</b></summary>
<ul><li>a) Sai — đó là 07:00 sáng 16/11 giờ VN, tức mốc nửa đêm UTC.</li><li>b) Đúng — 00:00 +07 = 17:00Z ngày hôm trước.</li><li>c) Sai — đó là 14:00 giờ VN.</li><li>d) Sai — R-GEN-02 nói rõ «bất kể múi giờ của máy chủ».</li></ul>
</details>

**Câu 4 (K3).** Báo giá tạo lúc 2026-11-16T02:00:00.000Z. Khách tạo chuyến từ báo giá này lúc 2026-11-16T02:05:00.000Z. Kết quả mong đợi?

- a) Tạo chuyến thành công vì còn trong 5 phút
- b) `QUOTE_EXPIRED`
- c) `QUOTE_NOT_FOUND`
- d) Tuỳ độ chính xác mili giây của máy chủ

<details><summary><b>Đáp án: b</b></summary>
<ul><li>a) Sai — R-QUOTE-01: còn hiệu lực khi now &lt; <code>expiresAt</code>; đúng bằng <code>expiresAt</code> là hết hạn.</li><li>b) Đúng — <code>expiresAt</code> = 02:05:00.000Z (lab: TC-QE-002).</li><li>c) Sai — báo giá tồn tại và của chính khách.</li><li>d) Sai — dữ liệu đã cho chính xác tới mili giây, kết quả xác định.</li></ul>
</details>

**Câu 5 (K3).** Voucher 2 lượt/ngày. Hôm nay khách đã tạo ba chuyến với mã đó: chuyến 1 đang `searching`, chuyến 2 đã bị khách huỷ, chuyến 3 `completed`. Khách báo giá thêm với mã đó. Kết quả mong đợi?

- a) Được, vì chuyến 3 đã hoàn thành nên không tính
- b) Được, vì chỉ đếm chuyến đang chạy
- c) `VOUCHER_USAGE_LIMIT`, vì có 2 chuyến chưa bị huỷ
- d) `VOUCHER_USAGE_LIMIT`, vì có 3 chuyến

<details><summary><b>Đáp án: c</b></summary>
<ul><li>a), b) Sai — R-VOUCHER-07 đếm chuyến «chưa bị huỷ», gồm cả chuyến đã hoàn thành.</li><li>c) Đúng — chuyến 1 và 3 được đếm: 2, không nhỏ hơn 2.</li><li>d) Sai — chuyến bị huỷ trả lại lượt.</li></ul>
</details>

**Câu 6 (K2).** Cách nào phù hợp nhất để có test data về khách hàng cho bộ test tự động của Tới Nơi?

- a) Xuất 100 khách ngẫu nhiên từ production để dữ liệu «giống thật»
- b) Xuất từ production rồi đổi họ tên, giữ nguyên số điện thoại
- c) Sinh khách giả lập có dấu hiệu nhận biết, giữ những đặc điểm quy tắc quan tâm (tên có dấu, độ dài hợp lệ)
- d) Dùng một khách tên «test» cho mọi test case

<details><summary><b>Đáp án: c</b></summary>
<ul><li>a) Sai — đưa dữ liệu cá nhân thật vào testware.</li><li>b) Sai — số điện thoại vẫn là dữ liệu cá nhân; che một phần không đủ.</li><li>c) Đúng — mục 4.8.</li><li>d) Sai — một khách dùng chung tạo phụ thuộc giữa ca, và «test» không giống dữ liệu thật ở chỗ quy tắc quan tâm.</li></ul>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 1.4.1 Test Activities and Tasks (FL-1.4.1, K2), mục 1.4.3 Testware — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level Test Analyst Syllabus* v4.0 — bối cảnh về vai trò của Test Analyst trong test design và test implementation: https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (mục *test data*, *precondition*, *testware*).
- S. Freeman, N. Pryce, *Growing Object-Oriented Software, Guided by Tests*, Addison-Wesley, 2009 — mẫu test data builder.
- M. Fowler, «ObjectMother» — https://martinfowler.com/bliki/ObjectMother.html
- Vitest — tuỳ chọn `sequence.shuffle`: https://vitest.dev/config/#sequence-shuffle
- Lab Tới Nơi: `SPEC.md` (R-GEN-02, R-GEN-05, R-VOUCHER-01/04/07, R-QUOTE-01/04, R-FARE-05, Phụ lục A) và `articles/test-case-design-test-data/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TCD-01 · Test analysis](/articles/test-case-design-test-analysis-test-condition) · [TCD-02 · Giải phẫu test case](/articles/test-case-design-giai-phau-test-case) · [TCD-03 · Test case logic và cụ thể](/articles/test-case-design-logic-va-cu-the) · [TCD-04 · Kết quả mong đợi và test oracle](/articles/test-case-design-expected-result-test-oracle) · [TEST-010 · Testware](/articles/testware-san-pham-kiem-thu) · [TEST-058 · Dependency Isolation](/articles/dependency-isolation)

Bài tiếp theo: [TCD-06 · Chọn và phối hợp kỹ thuật thiết kế test case](/articles/test-case-design-phoi-hop-ky-thuat).
