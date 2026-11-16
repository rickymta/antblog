---
slug: test-case-design-thuc-hanh-dat-chuyen
title: "Capstone Thiết kế Test Case: trọn bộ test case cho «đặt chuyến từ báo giá» của Tới Nơi"
summary: "Bài tổng hợp nhánh TCD: từ SPEC đến một bộ 31 test case logic, 47 test case cụ thể cho POST /api/trips — phân tích và test condition, chọn kỹ thuật, test data, oracle kể cả tác dụng phụ, RTM sinh tự động, lịch chạy risk-based, review, chạy qua Express thật bằng supertest với API kiểm thử, và báo cáo kết thúc kiểm thử có 3 defect."
category: testing
level: advanced
tags: [testing, istqb, ctfl, test-case-design, tcd-10, capstone, test-analysis, traceability, test-case-prioritization, idempotency, rate-limit, supertest, vitest, toi-noi]
language: vi
---

«Đặt chuyến» là một nút bấm trên app, nhưng phía sau nó là `POST /api/trips` với sáu lớp kiểm tra chồng lên nhau. Lớp đầu là xác thực, rồi tới vai trò, giới hạn tần suất, idempotency, định dạng, và cuối cùng là bốn lỗi của báo giá. Lượt dùng voucher được kiểm lại lúc tạo chuyến, còn giá phải được chụp nguyên từ báo giá. Một bộ test viết theo cảm hứng sẽ có vài ca «đặt được», «báo giá hết hạn», «gửi lại không trùng», và bỏ sót những chỗ dễ gây sự cố: biên nửa đêm của lượt voucher, yêu cầu thứ sáu ở giây 59,999, hay lỗi 4xx được lưu lại khiến khách bấm lại bao nhiêu lần cũng nhận cùng một lỗi. Bài capstone này đi trọn quy trình của nhánh TCD trên đúng một tính năng: phân tích → test condition → kỹ thuật → test case logic → test case cụ thể → test data → oracle → RTM → ưu tiên → review → chạy → báo cáo. Mỗi con số trong bài được lab chạy thật xác nhận.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TCD-10</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>TCD · Thiết kế Test Case — bài tổng hợp</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Áp dụng (tổng hợp)</td></tr>
<tr><th scope="row">Test Level</th><td>System, qua API HTTP của ứng dụng đã lắp ráp</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; một phần bảo mật (giới hạn tần suất, chống IDOR)</td></tr>
<tr><th scope="row">Test Technique</th><td>Equivalence partitioning, boundary value analysis, decision table, state transition, error guessing; cặp vi phạm cho thứ tự lỗi</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td>TCD-01…09, đặc biệt <a href="/articles/test-case-design-traceability-matrix">TCD-08</a> và <a href="/articles/test-case-design-uu-tien-review-bao-tri">TCD-09</a></td></tr>
<tr><th scope="row">Thực hành</th><td>31 test case logic, 47 test case cụ thể chạy qua Express bằng supertest, dùng <code>/api/test/reset</code>, <code>/clock</code>, <code>/weather</code>; RTM và coverage sinh tự động: <code>examples/toi-noi-testing/articles/test-case-design-thuc-hanh-dat-chuyen/</code> (48 test)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · mục 1.4.1 · FL-1.4.1 (K2) · mục 1.4.3 · FL-1.4.3 (K2) · mục 1.4.4 · FL-1.4.4 (K2) · mục 3.2 (review) · mục 4.2 (kỹ thuật hộp đen) · mục 5.1.5 · FL-5.1.5 (K3)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Chín bài trước của nhánh TCD mỗi bài dạy một khâu. Bài này ghép các khâu lại trên một đề bài thật, theo đúng trình tự một tester làm ngoài dự án:

| Bước | Việc | Sản phẩm | Học ở |
|---:|---|---|---|
| 1 | Phân tích test basis, review đặc tả | Câu hỏi gửi BA, 26 test condition | [TCD-01](/articles/test-case-design-test-analysis-test-condition) |
| 2 | Chọn kỹ thuật cho từng condition | Bảng condition → kỹ thuật | [TCD-06](/articles/test-case-design-phoi-hop-ky-thuat) |
| 3 | Test case logic → cụ thể | 31 ca logic, 47 ca cụ thể | [TCD-02](/articles/test-case-design-giai-phau-test-case), [TCD-03](/articles/test-case-design-logic-va-cu-the) |
| 4 | Test data | TD-01…TD-04 và dòng dữ liệu biên | [TCD-05](/articles/test-case-design-test-data) |
| 5 | Kết quả mong đợi, oracle, tác dụng phụ | Oracle tính tay, quan sát trạng thái lưu | [TCD-04](/articles/test-case-design-expected-result-test-oracle) |
| 6 | Ma trận truy vết | RTM xuôi/ngược, độ phủ | [TCD-08](/articles/test-case-design-traceability-matrix) |
| 7 | Ưu tiên, lịch chạy | Test execution schedule risk-based | [TCD-09](/articles/test-case-design-uu-tien-review-bao-tri) |
| 8 | Review bộ test case | 8 phát hiện, bản sửa | [TCD-09](/articles/test-case-design-uu-tien-review-bao-tri) |
| 9 | Chạy qua API | 48 test vitest, RTM sinh tự động | Mục 5 |
| 10 | Báo cáo kết thúc | Test completion summary, 3 defect report | Mục 5.5 |

Domain được chọn khác với hai bài thực hành có sẵn. [Thực hành Series 01](/articles/thuc-hanh-series-01-phan-tich-tinh-nang-dat-xe) dừng ở test condition cho cả luồng đặt xe. [SYS-03](/articles/system-testing-functional-test) thiết kế ca chức năng cho **cước**. Bài này lấy riêng **bước tạo chuyến**: nơi idempotency, giới hạn tần suất, lượt voucher và báo giá gặp nhau, và cũng là nơi R-VALID-02 quy định thứ tự sáu lớp lỗi.

Đọc theo cách của bài Series 01: tự làm mỗi bước trên giấy trước khi xem lời giải. Lời giải mẫu ở đây không phải đáp án duy nhất. Thứ cần đối chiếu là **lý do** và **truy vết**.

## 2. Lý thuyết theo ISTQB — các mảnh được ghép lại

Không có khái niệm mới. Bảng dưới chỉ nói mỗi mục CTFL đóng vai gì trong bài.

| Mục CTFL v4.0.1 | Nội dung dùng trong bài |
|---|---|
| 1.4.1 · FL-1.4.1 (K2) | Ba hoạt động nối tiếp: **test analysis** (test cái gì: condition, ưu tiên sơ bộ, defect của test basis), **test design** (test thế nào: test case, coverage item, test data requirement), **test implementation** (chuẩn bị chạy: test procedure, test suite, test execution schedule, dữ liệu thật). Cuối cùng là test execution và test completion |
| 1.4.3 · FL-1.4.3 (K2) | Testware của từng hoạt động: test condition đã ưu tiên và defect report về test basis; test case, coverage item; test procedure, test suite, test execution schedule; log, defect report; test completion report |
| 1.4.4 · FL-1.4.4 (K2) | Truy vết test basis ↔ condition ↔ test case ↔ kết quả ↔ defect, để đo coverage, đánh giá rủi ro còn lại, phân tích ảnh hưởng |
| 3.2 | Quy trình review năm hoạt động; ở đây review bộ test case bằng checklist CL-TC của TCD-09 |
| 4.2 | Kỹ thuật hộp đen: EP, BVA, decision table, state transition. Coverage của từng kỹ thuật |
| 5.1.5 · FL-5.1.5 (K3) | Ưu tiên test case theo rủi ro, có tôn trọng phụ thuộc |

Mục 1.4.1 nhấn mạnh một điều: test analysis không chỉ **đọc** test basis mà còn **đánh giá** nó, và defect tìm thấy lúc này là rẻ nhất. Bước 1 của bài tìm được năm câu hỏi cho BA trước khi viết test case nào. Một trong số đó (Q-2) cho thấy hành vi đúng đặc tả nhưng gây khó cho khách.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Các quy ước sau là của bài. Mã <code>TCOND-BOOK-nn</code>, <code>TC-BOOK-nnn</code>, <code>TD-nn</code>, <code>RB-nn</code>. Thang rủi ro 1–3 với mức = khả năng × tác động. Kỹ thuật «cặp vi phạm liền kề» để kiểm thứ tự lỗi. Viết thiết kế thành dữ liệu TypeScript để sinh RTM. Cách dùng <code>it.fails</code> để giữ ca có defect đã biết trong bộ hồi quy.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Khách đã có một báo giá còn hạn (R-QUOTE-01: 5 phút) và bấm «Đặt chuyến». App gửi `POST /api/trips` với `quoteId`, `paymentMethod` tuỳ chọn và header `Idempotency-Key`. Theo R-UI-01, mỗi báo giá dùng **một** khoá để bấm lại không tạo chuyến trùng.

| Mã | Quy tắc (tóm tắt SPEC v1.1) |
|---|---|
| R-API-08 | Khách, cần `Idempotency-Key`; thân `{ quoteId, paymentMethod? }`, mặc định `cash`; giá lấy từ báo giá, client không gửi giá; tạo chuyến ở `searching`, trả **201** |
| R-QUOTE-03 | Theo thứ tự: không có báo giá hoặc của khách khác ⇒ 422 `QUOTE_NOT_FOUND` → hết hạn ⇒ 422 `QUOTE_EXPIRED` → đã dùng ⇒ 409 `QUOTE_ALREADY_USED` → hết lượt voucher ⇒ 422 `VOUCHER_USAGE_LIMIT`. Mỗi báo giá tối đa một chuyến, kể cả khi hai yêu cầu tới cùng lúc |
| R-QUOTE-04 | Chuyến lưu ảnh chụp giá; cao điểm, thời tiết, voucher đổi sau khi báo giá không làm đổi giá chuyến |
| R-VOUCHER-07 | Lượt dùng = số chuyến chưa huỷ của khách với mã đó trong ngày giờ VN, phải &lt; `perCustomerPerDay`; tạo chuyến tiêu lượt; huỷ trả lượt; kiểm lại khi tạo chuyến, ngày tính theo thời điểm của lần kiểm |
| R-IDEM-01…05 | Khoá 8–128 ký tự `A–Z a–z 0–9 _ . : -`; gửi lại cùng nội dung ⇒ trả lại phản hồi đầu; khác nội dung ⇒ 422; đang xử lý ⇒ 409; 2xx/4xx của bước xử lý được lưu, 5xx không; 401/403/429 không gắn với khoá |
| R-VALID-02 | Thứ tự: 401 → 403 → 429 → idempotency → định dạng (400) → lỗi báo giá |
| R-API-13 | Tối đa **5** `POST /api/trips` mỗi khách trong **60 giây trượt**; vượt ⇒ 429 `RATE_LIMITED` + `Retry-After`; đếm mọi yêu cầu đã qua xác thực và vai trò, kể cả lỗi dữ liệu và gửi lại, trừ chính yêu cầu bị 429 |

Yêu cầu phụ có ca truy vết tới: R-STATE-06 (tạo chuyến ghi 2 dòng nhật ký), R-API-05 (báo giá khách khác trả giống không tồn tại), R-API-14 (trường lạ bị bỏ qua), R-QUOTE-01 (mốc hết hạn), R-API-01 (JSON hỏng).

### 3.2. System context

Ở kiến trúc đích, app khách gọi API Gateway, Gateway lo xác thực và giới hạn tần suất, rồi chuyển tới Booking/Order Service. Booking giữ idempotency key, gọi Pricing & Quote để lấy báo giá, gọi Promotion để kiểm lượt voucher, ghi đơn kèm outbox. Trong lab, mọi thứ chạy trong một tiến trình Express theo chuỗi `express.json` → `authenticate` → `only('customer')` → `rateLimitTrips` → `runIdempotent` → `parse` → `createTrip` ([TEST-050](/articles/api-testing-tu-goc-nhin-code) đã vẽ chuỗi này). Bộ đếm tần suất và khoá idempotency nằm trong bộ nhớ của một tiến trình (giới hạn của lab).

Bài kiểm **từ ngoài vào** như trên một môi trường test: chỉ gọi HTTP, điều khiển đồng hồ và thời tiết bằng API kiểm thử `/api/test/*` (R-TEST-02). Chỉ có ba chỗ cần thay một phụ thuộc trong tiến trình để tạo tình huống không gây ra được từ ngoài: hai yêu cầu đồng thời, yêu cầu đang xử lý, và kho dữ liệu lỗi.

### 3.3. Rủi ro

| Mã | Rủi ro | K | T | Mức |
|---|---|---:|---:|---:|
| RB-01 | Gửi lại hoặc gửi đồng thời tạo chuyến trùng; đặt hai chuyến từ một báo giá | 3 | 3 | 9 |
| RB-02 | Giá chuyến khác giá đã báo, hoặc client chỉnh được giá | 2 | 3 | 6 |
| RB-03 | Khách dùng voucher vượt số lượt | 2 | 2 | 4 |
| RB-04 | Khách hợp lệ bị chặn sai (khoá, tần suất, lỗi đã lưu, lượt chưa trả) | 2 | 2 | 4 |
| RB-06 | Đặt được từ báo giá hết hạn | 2 | 2 | 4 |
| RB-07 | Tạo chuyến hàng loạt không bị giới hạn | 2 | 2 | 4 |
| RB-05 | Lộ sự tồn tại báo giá/chuyến của khách khác | 1 | 3 | 3 |
| RB-08 | Mã lỗi sai thứ tự khiến client xử lý sai | 2 | 1 | 2 |
| RB-09 | Trạng thái thanh toán ban đầu sai | 1 | 2 | 2 |

RB-01 có khả năng 3 vì app di động gửi lại yêu cầu rất thường xuyên: mạng chập chờn, người dùng bấm hai lần, client tự thử lại khi hết giờ. RB-04 hay bị đánh giá thấp. Chặn nhầm khách hợp lệ không ồn ào như thu trùng tiền, nhưng mỗi lần chặn nhầm là một chuyến mất.

## 4. Phân tích kỹ thuật

### 4.1. Bước 1 — test analysis: review test basis và test condition

Đọc chéo R-API-08, R-QUOTE-03, R-IDEM, R-VALID-02, R-API-13 với R-UI-01 cho ra năm câu hỏi gửi BA **trước khi** viết test case:

| Mã | SPEC | Câu hỏi | Vì sao quan trọng |
|---|---|---|---|
| Q-1 | R-API-13 | «60 giây trượt»: yêu cầu ở đúng giây thứ 60 kể từ yêu cầu cũ nhất còn trong cửa sổ không? | Quyết định kết quả mong đợi của ca biên TD-51b. Lab đang coi là **ra khỏi** cửa sổ |
| Q-2 | R-UI-01, R-IDEM-05, R-VOUCHER-07 | UI dùng một khoá cho mỗi báo giá; 4xx được lưu. Đặt bị `VOUCHER_USAGE_LIMIT`, khách huỷ chuyến kia để lấy lại lượt, bấm lại «Đặt chuyến» ⇒ vẫn nhận 422 đã lưu, mãi mãi | Đúng đặc tả nhưng khách bị kẹt; phải báo giá lại mà không ai nói cho họ |
| Q-3 | R-API-01, R-VALID-02 | JSON hỏng (400) và thân quá lớn (413) đứng ở đâu trong thứ tự của R-VALID-02? | Hiện lab trả 400 trước cả 401 và 429 |
| Q-4 | R-IDEM-05, R-QUOTE-03 | 5xx «client được thử lại», nhưng nếu báo giá đã bị đánh dấu dùng trước khi lỗi thì thử lại sẽ ra 409. SPEC có đòi đánh dấu báo giá và tạo chuyến là một giao dịch không? | Gắn với defect DR-050-01 |
| Q-5 | R-IDEM-02, R-API-14 | «Cùng nội dung» là cùng JSON. Client thử lại mà thêm một trường lạ (bị R-API-14 bỏ qua) thì nhận 422 `IDEMPOTENCY_KEY_REUSED` | Thư viện client hay tự thêm trường; cần ghi rõ cho đội app |

Từ test basis, ta có **26 test condition**: `TCOND-BOOK-01…26`. Mỗi condition là một khía cạnh kiểm được, chưa có dữ liệu:

| Nhóm | Condition | Yêu cầu |
|---|---|---|
| Thành công và giá | 01 đặt hợp lệ, giá và bảng chi tiết từ báo giá, mặc định `cash` · 02 `vixanh` ⇒ `pending`, không gọi ví · 03 ảnh chụp giá · 04 giá do client gửi bị bỏ qua · 26 hai dòng nhật ký | R-API-08, R-QUOTE-04, R-STATE-06 |
| Lỗi báo giá | 05 không có / của khách khác ⇒ cùng 422 · 06 mốc hết hạn · 07 đã dùng, kể cả đồng thời · 08 thứ tự bốn lỗi | R-QUOTE-03 |
| Voucher | 09 hết lượt ⇒ 422, báo giá không bị tiêu · 10 huỷ trả lượt · 11 ngày theo thời điểm tạo chuyến, giờ VN · 12 biên `perCustomerPerDay` = 2 | R-VOUCHER-07 |
| Idempotency | 13 định dạng khoá · 14 gửi lại cùng nội dung · 15 phạm vi theo người gọi · 16 khác nội dung · 17 đang xử lý · 18 4xx được lưu · 19 5xx không lưu · 20 429 không gắn khoá | R-IDEM-01…05 |
| Tần suất | 21 tối đa 5, thứ 6 ⇒ 429 · 22 biên cửa sổ, `Retry-After` · 23 đếm lỗi và gửi lại, không đếm 429 · 24 theo từng khách | R-API-13 |
| Thứ tự lỗi | 25 401 → 403 → 429 → idempotency → định dạng → báo giá | R-VALID-02 |

### 4.2. Bước 2 — chọn kỹ thuật cho từng condition

Không có kỹ thuật nào phủ hết. Mỗi condition gọi một kỹ thuật theo **hình dạng** của quy tắc ([TCD-06](/articles/test-case-design-phoi-hop-ky-thuat)):

| Hình dạng quy tắc | Condition | Kỹ thuật | Coverage item |
|---|---|---|---|
| Nhiều lớp đầu vào được xử lý như nhau | 05, 13 (bộ ký tự), 15, 16, 24 | **EP** — một đại diện mỗi phân vùng; ca âm mỗi ca một lỗi | Phân vùng |
| Ngưỡng số hay ngưỡng thời gian | 06 (5 phút), 11 (nửa đêm), 12 (2 lượt), 13 (8 và 128 ký tự), 21 (5 yêu cầu), 22 (60 giây) | **BVA 2 giá trị** — giá trị trên biên và ngay sát nó | Cặp giá trị biên |
| Kết quả phụ thuộc tổ hợp | 03 (cao điểm × mưa × voucher lúc báo giá vs lúc đặt) | **Decision table** rút gọn còn một cột «mọi điều kiện đổi» | Cột |
| Kết quả phụ thuộc lịch sử | 10, 14, 18, 19, 20 | **State transition** trên bản ghi khoá idempotency và lượt voucher | Chuyển trạng thái |
| Chuỗi kiểm tra có thứ tự | 08, 25 | **Cặp vi phạm liền kề**: n bước ⇒ n − 1 cặp khả thi | Cặp |
| Tình huống dễ sai mà đặc tả nói gọn | 04, 07 (đồng thời), 17, 19 (5xx) | **Error guessing** có định hướng | Danh sách lỗi đoán |

Hai lựa chọn đáng giải thích kỹ hơn.

**Bản ghi khoá idempotency là một máy trạng thái.** R-IDEM nghe giống một danh sách luật, nhưng thật ra mô tả các trạng thái của cặp (người gọi, khoá): *chưa có* → *đang xử lý* → *đã lưu phản hồi*. Có một cạnh đặc biệt: *đang xử lý* → *chưa có* khi gặp 5xx. Lỗi 401/403/429 xảy ra trước khi bản ghi được tạo nên không gắn với khoá. Nhìn theo trạng thái thì thấy ngay những chuyển cần ca: gửi lại khi *đã lưu* (14, 18), gửi khi *đang xử lý* (17), gửi lại sau 5xx (19), gửi lại sau 429 (20). Một danh sách luật dễ làm sót cạnh 429.

**Kiểm thứ tự bằng cặp liền kề, không bằng tổ hợp.** R-VALID-02 có sáu lớp. Kiểm mọi tổ hợp vi phạm thì tới 2⁶ − 1 = 63 tổ hợp, phần lớn không khả thi: không thể vừa 401 vừa 403, vì phải xác thực rồi mới xét vai trò. Cách gọn là cho mỗi cặp lớp liền kề một ca chứa **cả hai** vi phạm và kiểm rằng lớp đứng trước thắng. Thứ tự là quan hệ bắc cầu, nên đủ các cặp liền kề là đủ thứ tự. Lớp 401/403 không ghép được với nhau, nên cặp a và b dùng ca «vi phạm mọi thứ phía sau». Đây là đúng tinh thần single fault của TCD-06: **trước hết** mỗi lỗi có ca đơn của nó (ở các nhóm khác), **sau đó** mới tới ca nhiều lỗi với mục đích duy nhất là kiểm thứ tự.

### 4.3. Bước 3 — từ test case logic tới test case cụ thể

Test case logic nói *cái gì* với giá trị trừu tượng. Test case cụ thể gắn giá trị thật ([TCD-03](/articles/test-case-design-logic-va-cu-the)). Một ví dụ đi trọn từ condition tới dòng dữ liệu:

| Mức | Nội dung |
|---|---|
| Condition | TCOND-BOOK-22 · Biên của cửa sổ trượt 60 giây và giá trị `Retry-After` |
| Test case logic | TC-BOOK-051 · Năm yêu cầu cùng lúc tại t; yêu cầu thứ sáu ngay trước t + 60 giây ⇒ 429 với `Retry-After` là số giây còn lại làm tròn lên, tối thiểu 1; tại đúng t + 60 giây ⇒ được xử lý |
| Cụ thể TD-51a | t = 04/11/2026 09:00:00,000 giờ VN; yêu cầu thứ sáu lúc 09:00:59,999 ⇒ 429, `Retry-After: 1` (0,001 giây làm tròn lên 1) |
| Cụ thể TD-51b | Yêu cầu thứ sáu lúc 09:01:00,000 ⇒ 201 (theo cách hiểu hiện tại của Q-1) |

Toàn bộ 31 test case logic, nhóm theo condition, kèm số test case cụ thể:

| Nhóm | Test case logic (số ca cụ thể) | Kỹ thuật |
|---|---|---|
| Thành công, giá | 001 đặt hợp lệ mặc định `cash` (1) · 002 `vixanh` (1) · 003 ảnh chụp giá 12:58 → 13:01 (1) · 004 client gửi `fare` (1) | EP, decision table, error guessing |
| Lỗi báo giá | 010 không tồn tại vs của C002 (1) · 011 hạn 4:59,999 / 5:00,000 (2) · 012 đã dùng, khoá khác (1) · 013 hai yêu cầu đồng thời (1) · 014 ba cặp thứ tự (3) | EP, BVA, error guessing, cặp liền kề |
| Voucher | 020 hết lượt (1) · 021 huỷ trả lượt (1) · 022 nửa đêm 23:59:59,999 / 00:00:00,000 (2) · 023 FOOD15 lượt 1, 2, 3 (1) | EP, state, BVA |
| Idempotency | 030 thiếu khoá (1) · 031 độ dài 7 / 8 / 128 / 129 (4) · 032 bộ ký tự (4) · 033 gửi lại đảo khoá (1) · 034 phạm vi theo khách (1) · 035 khác nội dung (1) · 036 đang xử lý (1) · 037 4xx được lưu sau khi nguyên nhân hết (1) · 038 5xx không lưu (1) · 039 429 không gắn khoá (1) | EP, BVA, state, error guessing |
| Tần suất | 050 thứ 6 ⇒ 429 (1) · 051 biên 59,999 / 60,000 giây (2) · 052 đếm gửi lại (1) · 053 không đếm 429 (1) · 054 theo từng khách (1) | BVA, EP |
| Thứ tự lỗi | 060 sáu cặp (6) · 061 JSON hỏng không `x-user` (1) · 062 hết lượt tần suất + JSON hỏng (1) | Cặp liền kề |

Cộng lại là 31 test case logic, 47 test case cụ thể.

TC-BOOK-053 cho thấy vì sao phải nghĩ tới **dữ liệu phân biệt được** đúng sai. Ý tưởng đầu tiên cho «không đếm yêu cầu bị 429» là: 5 yêu cầu ở giây 0, ba yêu cầu bị 429 ở giây 30, đặt ở giây 60. Nhưng nếu hệ thống lỡ đếm cả 429, cửa sổ tại giây 60 chỉ chứa 3 yêu cầu (những yêu cầu ở giây 0 đã ra khỏi cửa sổ), dưới ngưỡng 5. Thế là ca vẫn xanh dù có lỗi. Phải dùng **năm** yêu cầu bị 429 ở giây 30. Khi đó hệ thống đếm sai sẽ thấy 5 trong cửa sổ và chặn, còn hệ thống đúng sẽ cho đặt.

### 4.4. Bước 4 — test data

Toàn bộ dữ liệu là dữ liệu mẫu của lab (SPEC Phụ lục A). Thời điểm là giờ VN trên đồng hồ giả, đặt qua `POST /api/test/clock`. Giá tính tay theo R-FARE, **không** gọi mã tính cước của sản phẩm.

| Mã | Dữ liệu | Tính tay | Dùng ở |
|---|---|---|---|
| TD-01 | C001, `ride`, SG, `bike`, 7,4 km, 04/11/2026 09:00, không mưa | 12.000 + 4.500 × 7,4 = 45.300 ⇒ **46.000 đ** | Phần lớn các ca |
| TD-02 | TD-01 + `GIAM20` (20%, trần 30.000 đ, 1 lần/ngày) | Giảm 9.060 ⇒ 36.240 ⇒ **37.000 đ** | 014, 020, 021, 022, 037 |
| TD-03 | TD-02 lúc 12:58, SG đang mưa | 45.300 × 1,25 = 56.625 + 8.000 = 64.625 − 12.925 = 51.700 ⇒ **52.000 đ** (đúng ví dụ R-FARE-09) | 003 |
| TD-04 | `food`, SG, 3 km, một món 60.000 đ, `FOOD15` (15%, tối thiểu 50.000 đ, 2 lần/ngày) | Phí giao 12.000 + 13.500 = 25.500 ⇒ 26.000; giảm 9.000; tổng 60.000 − 9.000 + 26.000 = **77.000 đ** | 023 |
| Khoá | `dat-chuyen-00nn` (15 ký tự); biên `k` × 7, 8, 128, 129; `Ab9_.:-x` đủ mọi lớp ký tự hợp lệ | — | 030–039 |

Mỗi ca bắt đầu từ `POST /api/test/reset`, đưa dữ liệu, khoá idempotency và bộ đếm tần suất về ban đầu, rồi đặt đồng hồ về 09:00. Đây là **hậu điều kiện được khôi phục** cho ca kế tiếp. Không có nó, năm ca đầu tiên đã tiêu hết 5 lượt tần suất của C001 trong cùng một phút đồng hồ giả, và ca thứ sáu sẽ đỏ vì một lý do chẳng liên quan gì tới nó.

### 4.5. Bước 5 — kết quả mong đợi và oracle, kể cả tác dụng phụ

Kết quả mong đợi phải gồm **mọi đầu ra quan sát được** ([TCD-04](/articles/test-case-design-expected-result-test-oracle)), không chỉ mã HTTP. Với tạo chuyến, đó là:

| Đầu ra | Quan sát bằng | Ví dụ ca |
|---|---|---|
| Mã HTTP, `error.code`, thân lỗi | Phản hồi | Mọi ca âm; 010 so **nguyên thân** của hai lỗi |
| Header `Idempotent-Replayed`, `Retry-After` | Phản hồi | 033, 039, 050, 051 |
| Nội dung chuyến: trạng thái, giá, bảng chi tiết, `paymentStatus` | Phản hồi 201 | 001, 003, 004 |
| Số chuyến được tạo | `GET /api/driver/available-trips` của D101 | 011, 012, 013, 020, 030, 033, 035 |
| Nhật ký trạng thái | Đọc kho (`listEvents`) — trạng thái lưu | 001, 033 |
| Báo giá có bị tiêu không | Đặt lại bằng khoá khác | 030, 021 |
| Lượt voucher | Báo giá mới với cùng mã | 014, 037 |
| Lời gọi ví | Sổ ghi của ví giả | 001, 002 |

Ba nguồn oracle được dùng. **Đặc tả**: mã lỗi, thứ tự, số lượt. **Tính tay**: giá TD-01…04. **So sánh hai lần chạy**: thân replay phải giống hệt thân đầu, thân lỗi «không tồn tại» phải giống hệt thân lỗi «của khách khác». Loại thứ ba rất mạnh cho R-API-05. Nó không cần biết thông điệp lỗi là gì, chỉ cần hai phản hồi không phân biệt được với nhau.

Bài học «kiểm chứng điều kiện thí nghiệm» của SYS-03 được áp ở ba chỗ:

- TC-BOOK-003 báo giá lại lúc 13:01 với cùng thân. Kết quả phải là 37.000 đ, `peak` và `raining` đều false. Có vậy mới chắc «hết mưa, hết cao điểm» thật sự xảy ra, trước khi khẳng định chuyến vẫn giữ 52.000 đ.
- TC-BOOK-014 cặp 3 xin một báo giá `GIAM20` mới và kiểm rằng nó bị `VOUCHER_USAGE_LIMIT`. Như vậy vi phạm thứ hai thật sự có mặt, rồi mới khẳng định `QUOTE_ALREADY_USED` thắng.
- TC-BOOK-037 xin một báo giá `GIAM20` mới **sau khi huỷ** và kiểm rằng nó được 200. Lượt đã thật sự được trả, nên 422 khi gửi lại là do idempotency, không phải do voucher.

### 4.6. Bước 6 — RTM và độ phủ dự kiến

RTM được **sinh** từ dữ liệu thiết kế chứ không gõ tay. Mỗi test case khai báo `reqs` và `covers`; `rtm.ts` lật ngược lại thành yêu cầu → condition → test case, rồi cộng kết quả chạy. Trước khi chạy, RTM đã trả lời được ba câu hỏi của TCD-08:

- **Có yêu cầu trong phạm vi nào không có ca?** Không: 11/11 yêu cầu đều có ít nhất một ca.
- **Có condition nào không có ca?** Không: 26/26.
- **Có ca mồ côi không?** Không: mọi ca trỏ tới R-… và condition có thật.

Độ phủ theo kỹ thuật cũng đếm được. Ở BVA 2 giá trị, bảy biên đều có đủ cặp: 5 phút, nửa đêm, 60 giây, độ dài khoá tối thiểu 8 và tối đa 128 đều có cả hai phía. «5 yêu cầu» có 5 → 201, 6 → 429. «2 lượt» có lượt 2 → 201, lượt 3 → 422. Ở cặp thứ tự, R-QUOTE-03 có 3/3 cặp liền kề. R-VALID-02 có 6 cặp, cộng hai ca cho vị trí của JSON hỏng: một ca so với 401, một ca so với 429.

### 4.7. Bước 7 — ưu tiên và test execution schedule

Dùng đúng hàm của TCD-09: risk-based theo mức rủi ro cao nhất mà ca giảm, hoà thì theo mã, rồi sửa theo phụ thuộc. Phụ thuộc khai báo là **mọi ca phụ thuộc TC-BOOK-001**. Lý do: nếu luồng đặt chuyến cơ bản hỏng, các ca khác đều dùng nó làm bước chuẩn bị và sẽ đỏ hàng loạt vì cùng một nguyên nhân, kết quả không còn mang thông tin. TC-BOOK-001 chỉ ở mức rủi ro 6 nên thứ tự thuần đặt nó sau sáu ca mức 9. Phụ thuộc kéo nó lên đầu:

```text
001 → 012 → 013 → 030 → 033 → 035 → 036              (RB-01, mức 9)
→ 003 → 004                                          (RB-02, mức 6)
→ 011 → 020 → 021 → 022 → 023 → 031 → 032 → 037 → 038 → 039 → 050 → 051 → 052 → 053 → 054   (mức 4)
→ 010 → 034                                          (RB-05, mức 3)
→ 002 → 014 → 060 → 061 → 062                        (mức 2)
```

Nếu phải dừng sau 7 ca đầu, toàn bộ RB-01 (đặt trùng, mức 9) đã có kết quả. Nếu phải dừng sau 9 ca, cả RB-02 (giá) cũng đã có. Các ca về thứ tự lỗi (RB-08) xếp cuối: chúng quan trọng với đội app, nhưng sai thì không mất tiền.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Quy trình capstone gồm mười bước từ SPEC tới báo cáo: phân tích ra 26 test condition và 5 câu hỏi; chọn kỹ thuật; 31 test case logic thành 47 cụ thể; test data; oracle; RTM 11 trên 11 yêu cầu và 26 trên 26 condition; lịch risk-based có phụ thuộc; review 8 phát hiện; chạy 47 ca qua API với 44 đạt và 3 không đạt; báo cáo kết thúc với 3 defect.">
<defs>
<marker id="tcd10-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="30" width="160" height="70" rx="8" class="f-pk-soft s-pk"/><text x="100" y="54" text-anchor="middle" class="t-b t-pk">1 · Phân tích</text><text x="100" y="74" text-anchor="middle" class="t-small">26 condition</text><text x="100" y="90" text-anchor="middle" class="t-small">5 câu hỏi BA</text>
<rect x="200" y="30" width="160" height="70" rx="8" class="f-surface s-line"/><text x="280" y="54" text-anchor="middle" class="t-b">2 · Kỹ thuật</text><text x="280" y="74" text-anchor="middle" class="t-small">EP · BVA · DT</text><text x="280" y="90" text-anchor="middle" class="t-small">state · cặp lỗi</text>
<rect x="380" y="30" width="160" height="70" rx="8" class="f-surface s-line"/><text x="460" y="54" text-anchor="middle" class="t-b">3 · Logic → cụ thể</text><text x="460" y="74" text-anchor="middle" class="t-small">31 → 47 ca</text><text x="460" y="90" text-anchor="middle" class="t-small">4 · test data TD</text>
<rect x="560" y="30" width="160" height="70" rx="8" class="f-surface s-line"/><text x="640" y="54" text-anchor="middle" class="t-b">5 · Oracle</text><text x="640" y="74" text-anchor="middle" class="t-small">tính tay, so hai lần</text><text x="640" y="90" text-anchor="middle" class="t-small">tác dụng phụ</text>
<rect x="740" y="30" width="160" height="70" rx="8" class="f-phys-soft s-phys"/><text x="820" y="54" text-anchor="middle" class="t-b t-phys">6 · RTM</text><text x="820" y="74" text-anchor="middle" class="t-small">11/11 yêu cầu</text><text x="820" y="90" text-anchor="middle" class="t-small">26/26 condition</text>
<line x1="180" y1="65" x2="198" y2="65" class="s-muted" stroke-width="1.6" marker-end="url(#tcd10-a)"/>
<line x1="360" y1="65" x2="378" y2="65" class="s-muted" stroke-width="1.6" marker-end="url(#tcd10-a)"/>
<line x1="540" y1="65" x2="558" y2="65" class="s-muted" stroke-width="1.6" marker-end="url(#tcd10-a)"/>
<line x1="720" y1="65" x2="738" y2="65" class="s-muted" stroke-width="1.6" marker-end="url(#tcd10-a)"/>
<path d="M820 100 L820 140" class="f-none s-muted" stroke-width="1.6" marker-end="url(#tcd10-a)"/>
<rect x="740" y="142" width="160" height="70" rx="8" class="f-surface s-line"/><text x="820" y="166" text-anchor="middle" class="t-b">7 · Ưu tiên</text><text x="820" y="186" text-anchor="middle" class="t-small">risk-based</text><text x="820" y="202" text-anchor="middle" class="t-small">001 kéo lên đầu</text>
<rect x="560" y="142" width="160" height="70" rx="8" class="f-warn-soft s-warn"/><text x="640" y="166" text-anchor="middle" class="t-b t-warn">8 · Review</text><text x="640" y="186" text-anchor="middle" class="t-small">CL-TC</text><text x="640" y="202" text-anchor="middle" class="t-small">8 phát hiện</text>
<rect x="380" y="142" width="160" height="70" rx="8" class="f-surface s-line"/><text x="460" y="166" text-anchor="middle" class="t-b">9 · Chạy qua API</text><text x="460" y="186" text-anchor="middle" class="t-small">supertest + /api/test</text><text x="460" y="202" text-anchor="middle" class="t-small">44 đạt · 3 không đạt</text>
<rect x="200" y="142" width="160" height="70" rx="8" class="f-pk-soft s-pk"/><text x="280" y="166" text-anchor="middle" class="t-b t-pk">10 · Báo cáo</text><text x="280" y="186" text-anchor="middle" class="t-small">completion summary</text><text x="280" y="202" text-anchor="middle" class="t-small">3 defect · rủi ro còn lại</text>
<line x1="740" y1="177" x2="722" y2="177" class="s-muted" stroke-width="1.6" marker-end="url(#tcd10-a)"/>
<line x1="560" y1="177" x2="542" y2="177" class="s-muted" stroke-width="1.6" marker-end="url(#tcd10-a)"/>
<line x1="380" y1="177" x2="362" y2="177" class="s-muted" stroke-width="1.6" marker-end="url(#tcd10-a)"/>
<path d="M640 212 L640 250 L100 250 L100 103" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#tcd10-a)"/>
<text x="370" y="270" text-anchor="middle" class="t-small t-warn">review và chạy thử trả ngược về phân tích: Q-2 từ TC-BOOK-037, DR-TCD10-01 từ TC-BOOK-062</text>
<text x="20" y="20" class="t-small">test analysis</text><text x="380" y="20" class="t-small">test design</text><text x="740" y="232" class="t-small">test implementation</text><text x="200" y="232" class="t-small">execution · completion</text>
</svg>
</div>
<figcaption><b>Hình 1 · Mười bước của capstone, mỗi bước một sản phẩm.</b> Quy trình không thẳng một chiều: review và kết quả chạy đẩy câu hỏi ngược về test analysis, đúng như CTFL 1.4.1 nói các hoạt động có thể lặp và đan xen.</figcaption>
</figure>

### 4.8. Bước 8 — review bộ test case trước khi chạy

Bản nháp đầu tiên được review bằng checklist CL-TC của TCD-09, rồi một đồng nghiệp đọc kèm SPEC. Tám phát hiện:

| # | Ca nháp | Vấn đề | Mục CL-TC | Sửa |
|---|---|---|---|---|
| R-1 | 001 | Kết quả mong đợi «Tạo chuyến thành công» | 02 | Liệt kê trạng thái, giá = `quote.total`, `fareBreakdown` = `quote.fare`, `paymentStatus`, 2 dòng nhật ký, 0 lời gọi ví |
| R-2 | 011 | «Đợi 5 phút rồi đặt» | 04 | Dời đồng hồ giả tới 09:04:59,999 và 09:05:00,000 giờ VN |
| R-3 | cả bộ | Các ca dùng chung một app; ca thứ 6 của C001 trong cùng phút bị 429 | 03, 09 | `POST /api/test/reset` + đặt lại đồng hồ trong `beforeEach` |
| R-4 | 050 | Gộp «thứ 6 bị chặn» với «hết cửa sổ thì mở lại» | 01 | Tách thành 050 và 051 (hai dòng biên) |
| R-5 | 037 | Mong đợi «gửi lại sau khi huỷ chuyến kia ⇒ 201 vì đã có lượt» | 02 (sai đặc tả) | R-IDEM-05 lưu 4xx ⇒ mong đợi 422 replayed; đưa hành vi lên thành câu hỏi Q-2 |
| R-6 | (ca 070) | «p95 tạo chuyến &lt; 500 ms» trong bộ chức năng | 07 | Loại: R-NFR-02 thuộc bộ hiệu năng, không đo được có ý nghĩa ở lab một máy |
| R-7 | 003 | Không có bước kiểm điều kiện đã đổi | 08 | Thêm báo giá đối chứng lúc 13:01 = 37.000 đ |
| R-8 | 053 | Ba yêu cầu bị 429 không phân biệt được đếm hay không đếm | 08 | Năm yêu cầu (mục 4.3) |

R-5 là phát hiện giá trị nhất. Người viết nháp suy luận theo **ý định nghiệp vụ**: lượt đã trả thì khách phải đặt được. Reviewer đọc **câu chữ** R-IDEM-05: 4xx của bước xử lý được lưu và trả lại. Hai cách đọc cho hai kết quả mong đợi khác nhau. Test case không được tự chọn cách nào thuận tiện. Nó phải theo đặc tả hiện hành, và đưa mâu thuẫn lên thành câu hỏi Q-2 cho BA. R-6 và R-8 là hai kiểu lỗi máy không bắt được: một ca nằm sai bộ test, và một ca có dữ liệu không phân biệt được đúng với sai.

## 5. Thực hành

### 5.1. Tổ chức lab

Mã ở `examples/toi-noi-testing/articles/test-case-design-thuc-hanh-dat-chuyen/` ([lab trên GitHub](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing)):

| Tệp | Vai trò |
|---|---|
| `thiet-ke.ts` | Phạm vi 11 yêu cầu, 26 condition, 9 rủi ro, 31 test case logic (kỹ thuật, `reqs`, `covers`, `risks`, số ca cụ thể, phụ thuộc, defect đã biết), test data TD |
| `dat-chuyen.test.ts` | Cài đặt 47 test case cụ thể; đăng ký theo lịch risk-based; ghi kết quả từng ca; ca cuối sinh RTM |
| `rtm.ts` | RTM xuôi/ngược, độ phủ yêu cầu và condition, ca mồ côi; xuất Markdown và JSON |

Cài đặt của một ca đọc gần như nguyên văn test case cụ thể. Đây là TC-BOOK-022, dòng TD-22a:

```ts
reg('TC-BOOK-022', `${row.td} · đặt B lúc ${row.now.slice(0, 23)} giờ VN ⇒ ${row.status}`, async () => {
  await at('2026-11-04T23:57:00.000+07:00');
  const a = await quote(TD.RIDE_GIAM20);                    // báo giá A
  await at('2026-11-04T23:58:00.000+07:00');
  const b = await quote(TD.RIDE_GIAM20);                    // báo giá B, hết hạn 00:03
  await at('2026-11-04T23:58:30.000+07:00');
  await book('dat-chuyen-0026', { quoteId: a.quoteId }).expect(201);   // tiêu lượt ngày 04/11
  await at(row.now);                                        // 23:59:59,999 hoặc 00:00:00,000 hôm sau
  expect(err(await book('dat-chuyen-0027', { quoteId: b.quoteId }))).toEqual([row.status, row.code]);
});
```

`at()` gọi `POST /api/test/clock` với chuỗi ISO **có múi giờ** `+07:00`. Kết quả vì thế không phụ thuộc múi giờ của máy chạy test (R-GEN-02, CL-TC-04). Hàm `reg` bọc thân test để ghi `pass` hoặc `fail` cho RTM. Ca có defect đã biết được đăng ký bằng `it.fails`:

```ts
function reg(tc: string, label: string, fn: () => Promise<void>) {
  const defect = TEST_CASES.find((t) => t.id === tc)?.knownDefect;
  const body = async () => {
    try { await fn(); results.push({ tc, name: label, outcome: 'pass' }); }
    catch (error) { results.push({ tc, name: label, outcome: 'fail' }); throw error; }
  };
  if (defect && !process.env.SHOW_RED) it.fails(`${label} [${defect}]`, body);
  else it(label, body);
}
```

Lịch chạy là đầu ra của hàm TCD-09, dùng trực tiếp để đăng ký các khối `describe`:

```ts
export const SCHEDULE = respectDependencies(riskBased(TEST_CASES, RISKS), TEST_CASES);
SCHEDULE.forEach((id, i) => {
  const tc = TEST_CASES.find((t) => t.id === id)!;
  describe(`#${String(i + 1).padStart(2, '0')} ${id} · ${tc.reqs.join(', ')} · ${tc.title}`, IMPL[id]!);
});
```

### 5.2. Chạy

```bash
cd examples/toi-noi-testing
npx vitest run articles/test-case-design-thuc-hanh-dat-chuyen --reporter=verbose
```

Kết quả thật (Node 22, Vitest 3.2.7; đã lược tên file):

```text
✓ #01 TC-BOOK-001 · R-API-08, R-STATE-06 · Đặt từ báo giá hợp lệ, mặc định cash … > TD-01 · không gửi paymentMethod
✓ #02 TC-BOOK-012 · R-QUOTE-03 · Báo giá đã dùng, khoá khác ⇒ 409 QUOTE_ALREADY_USED, vẫn một chuyến
✓ #03 TC-BOOK-013 · R-QUOTE-03 · Hai yêu cầu đồng thời, cùng báo giá, khác khoá ⇒ một 201, một 409
…
✓ #13 TC-BOOK-022 · R-VOUCHER-07 · Nửa đêm giờ VN … > TD-22a · đặt B lúc 2026-11-04T23:59:59.999 giờ VN ⇒ 422
✓ #13 TC-BOOK-022 · R-VOUCHER-07 · Nửa đêm giờ VN … > TD-22b · đặt B lúc 2026-11-05T00:00:00.000 giờ VN ⇒ 201
…
✓ #18 TC-BOOK-038 · R-IDEM-05 · 500 không lưu: gửi lại cùng khoá ⇒ 201 > … [DR-050-01]
✓ #21 TC-BOOK-051 · R-API-13 · Cửa sổ trượt … > TD-51a · yêu cầu thứ 6 lúc 09:00:59.999 ⇒ 429
✓ #21 TC-BOOK-051 · R-API-13 · Cửa sổ trượt … > TD-51b · yêu cầu thứ 6 lúc 09:01:00.000 ⇒ 201
…
✓ #30 TC-BOOK-061 · R-VALID-02, R-API-01 · JSON hỏng, không có x-user ⇒ 401 > … [DR-050-02]
✓ #31 TC-BOOK-062 · R-VALID-02, R-API-13 · Khách đã hết lượt tần suất gửi JSON hỏng ⇒ 429 > … [DR-TCD10-01]
✓ Báo cáo > RTM và test completion summary (ghi reports/tcd-10/)

 Test Files  1 passed (1)
      Tests  48 passed (48)
```

48 test = 47 test case cụ thể + 1 test sinh báo cáo. Ba ca gắn nhãn defect chạy bằng `it.fails`: chúng **xanh vì hệ thống đang sai đặc tả**. `SHOW_RED=1` cho thấy kết quả thật theo SPEC: `3 failed | 45 passed`, với ba thông điệp:

```text
→ expected [ 409, 'QUOTE_ALREADY_USED' ] to deeply equal [ 201, undefined ]      TC-BOOK-038
→ expected [ 400, 'INVALID_JSON' ] to deeply equal [ 401, 'UNAUTHENTICATED' ]    TC-BOOK-061
→ expected [ 400, 'INVALID_JSON' ] to deeply equal [ 429, 'RATE_LIMITED' ]       TC-BOOK-062
```

### 5.3. RTM sinh tự động

Test cuối ghi `reports/tcd-10/rtm.md` và `rtm.json`. Nội dung thật của lần chạy:

| Yêu cầu | Condition | Test case | Chạy | Đạt | Không đạt | Defect | Trạng thái |
|---|---|---|---:|---:|---:|---|---|
| R-API-08 | 01, 02, 04 | 001, 002, 004 | 3 | 3 | 0 | — | Đạt |
| R-QUOTE-03 | 05, 06, 07, 08 | 010, 011, 012, 013, 014, 020 | 9 | 9 | 0 | — | Đạt |
| R-QUOTE-04 | 03 | 003 | 1 | 1 | 0 | — | Đạt |
| R-VOUCHER-07 | 09, 10, 11, 12 | 020, 021, 022, 023, 037 | 6 | 6 | 0 | — | Đạt |
| R-IDEM-01 | 13 | 030, 031, 032 | 9 | 9 | 0 | — | Đạt |
| R-IDEM-02 | 14, 15 | 033, 034 | 2 | 2 | 0 | — | Đạt |
| R-IDEM-03 | 16 | 035 | 1 | 1 | 0 | — | Đạt |
| R-IDEM-04 | 17 | 036 | 1 | 1 | 0 | — | Đạt |
| R-IDEM-05 | 18, 19, 20 | 037, 038, 039 | 3 | 2 | 1 | DR-050-01 | Có defect mở |
| R-VALID-02 | 25 | 060, 061, 062 | 8 | 6 | 2 | DR-050-02, DR-TCD10-01 | Có defect mở |
| R-API-13 | 21, 22, 23, 24 | 039, 050–054, 062 | 8 | 7 | 1 | DR-TCD10-01 | Có defect mở |

Kèm theo là các yêu cầu phụ: R-STATE-06 (1/1), R-API-14 (1/1), R-API-05 (1/1), R-QUOTE-01 (2/2) đều đạt, còn R-API-01 không đạt 0/1 (DR-050-02). Tổng: yêu cầu trong phạm vi **11/11 (100,0%)**, condition **26/26 (100,0%)**, 47/47 ca cụ thể đã chạy, **44 đạt, 3 không đạt**, không có ca mồ côi.

Cách đọc cho đúng: «100% yêu cầu có ca» là độ phủ **truy vết**, không phải bằng chứng yêu cầu đúng. R-IDEM-02 chỉ có 2 lần chạy cho hai condition. Đủ để phủ, nhưng chưa có ca cho `/cancel` (khoá có phạm vi theo `:id` của chuyến). Phần đó TCD-09 đã chạy (TC-IDEM-007) và nằm ngoài phạm vi «đặt chuyến». Con số phủ chỉ có nghĩa khi đi cùng phạm vi đã khai báo.

### 5.4. Ba defect report

**DR-050-01 (đã biết, mức cao).** TC-BOOK-038: kho chuyến lỗi đúng một lần ⇒ 500. Gửi lại cùng khoá thì R-IDEM-05 cho phép xử lý lại, mong đợi 201; thực tế 409 `QUOTE_ALREADY_USED`, khách mất báo giá và phải báo giá lại. Nguyên nhân đã được [TEST-050](/articles/api-testing-tu-goc-nhin-code) chỉ ra: `markUsed` chạy ngoài giao dịch với `trips.create`. Capstone tái hiện được defect qua một đường khác (bộ test chức năng thay vì phủ nhánh), cho thấy defect vẫn còn mở.

**DR-050-02 (đã biết, mức thấp).** TC-BOOK-061: JSON hỏng, không có `x-user` ⇒ 400 `INVALID_JSON` thay vì 401. `express.json()` chạy trước xác thực.

**DR-TCD10-01 (mới, mức thấp).** TC-BOOK-062. Tóm tắt: khách đã dùng hết 5 lượt trong 60 giây, gửi thêm một yêu cầu có thân JSON hỏng thì nhận 400 `INVALID_JSON`, trong khi R-VALID-02 đặt 429 trước mọi lỗi định dạng. Tái hiện: `POST /api/test/reset`; C001 gửi 5 `POST /api/trips` thân `{}` với 5 khoá khác nhau (cả 5 đều 400 `VALIDATION_ERROR`, được đếm); gửi thân `{"quoteId":`. Mong đợi 429 `RATE_LIMITED` kèm `Retry-After`; thực tế 400 `INVALID_JSON`. Nguyên nhân nghi ngờ trùng DR-050-02. Hệ quả suy ra từ cùng nguyên nhân, **chưa có ca riêng**: yêu cầu JSON hỏng không đi qua bộ đếm, nên cũng không bị đếm, trái với câu «kể cả yêu cầu bị từ chối vì dữ liệu sai» của R-API-13 (bài tập 4). Đề xuất: gộp với DR-050-02 khi BA trả lời Q-3, vì có thể sửa đặc tả thay vì sửa mã.

### 5.5. Test completion summary

| Mục | Nội dung |
|---|---|
| Đối tượng và phạm vi | `POST /api/trips` của lab Tới Nơi, SPEC v1.1: R-API-08, R-QUOTE-03/04, R-VOUCHER-07, R-IDEM-01…05, R-VALID-02, R-API-13. Ngoài phạm vi: UI (R-UI-01), `/cancel`, hiệu năng (R-NFR-02), nhiều tiến trình |
| Môi trường | Express trong tiến trình qua supertest, kho bộ nhớ, đồng hồ giả qua `/api/test/clock`, thời tiết giả; Node 22, Vitest 3.2.7 |
| Thực hiện | 31 test case logic, 47 cụ thể, chạy theo lịch risk-based; 44 đạt, 3 không đạt; 0 bị chặn, 0 bỏ qua |
| Độ phủ | Yêu cầu 11/11, condition 26/26; BVA đủ hai phía ở 7 biên; thứ tự lỗi 3/3 cặp (R-QUOTE-03) và 6 cặp + 2 ca JSON hỏng (R-VALID-02) |
| Defect | 3 mở: DR-050-01 (cao), DR-050-02 (thấp), DR-TCD10-01 (thấp, mới) |
| Câu hỏi test basis | Q-1…Q-5 chờ BA; Q-2 ảnh hưởng trải nghiệm đặt chuyến khi dùng voucher |
| Rủi ro còn lại | RB-01 với giao dịch thật song song và nhiều bản máy chủ: lab chỉ xen kẽ ở mức `await` trong một tiến trình. Bộ đếm tần suất và khoá idempotency ở bộ nhớ, không chia sẻ giữa các bản. RB-04 qua Q-2 trên UI thật chưa kiểm |
| Đánh giá | Hành vi đặt chuyến khớp đặc tả ở mọi nhóm trừ ba điểm trên. DR-050-01 nên sửa trước khi phát hành vì làm khách mất báo giá sau một lỗi hạ tầng tạm thời. Hai defect còn lại chờ quyết định về đặc tả |

Báo cáo nói rõ cả những gì **không** biết. «Đã chạy 47 ca, 44 đạt» mà không có dòng rủi ro còn lại thì người đọc sẽ hiểu thành «đặt chuyến an toàn khi chạy nhiều bản máy chủ», một điều bộ test này không hề kiểm.

## 6. Góc nhìn nâng cao

**Viết thiết kế thành dữ liệu có giá phải trả.** Lợi ích đã thấy rõ: RTM, độ phủ và lịch chạy được **tính**, không gõ tay, nên không lệch khỏi mã test. Cái giá là phải có kỷ luật. Thêm một `it` mà quên khai báo vào `thiet-ke.ts` thì RTM không biết. Lab chặn trường hợp này bằng hai kiểm tra ở test cuối: mọi ca trong thiết kế phải có cài đặt, và số ca chạy phải bằng số ca dự kiến (47). Nếu bộ test do nhiều người viết, hai kiểm tra đó nên chạy trong CI.

**Ưu tiên khi mọi thứ chạy dưới 3 giây.** Cả bộ chạy chừng 2,5 giây, vậy ưu tiên để làm gì? Thứ nhất, cùng thiết kế này khi chạy tay trên staging thật sẽ tốn nhiều giờ. Thứ hai, khi một ca đầu lịch đỏ, ta biết ngay rủi ro cao nhất có vấn đề, không phải đọc 47 dòng. Thứ ba, khi bộ hồi quy lớn lên và CI bị giới hạn thời gian, lịch có sẵn cho biết cắt ở đâu là ít mất thông tin nhất. Ưu tiên là thuộc tính của **thiết kế**, không phụ thuộc vào việc hôm nay chạy nhanh hay chậm.

**Ba chỗ phải thay phụ thuộc trong tiến trình.** TC-BOOK-013, 036 và 038 không gây ra được từ ngoài bằng HTTP một cách tất định. Hai yêu cầu «đồng thời» qua mạng có khi xen nhau, có khi không. Lab dùng cổng chờ và một proxy làm kho lỗi, như TEST-050. Cái giá: ba ca này kiểm **logic** xử lý xen kẽ trong một tiến trình, chưa kiểm khoá dòng của PostgreSQL thật dưới nhiều kết nối. Vì thế báo cáo vẫn giữ RB-01 trong rủi ro còn lại.

**Đúng đặc tả mà vẫn sai nhu cầu.** TC-BOOK-037 xanh: hệ thống làm đúng R-IDEM-05. Nhưng ghép với R-UI-01 thì khách bị kẹt (Q-2). Đây là khác biệt giữa verification và validation ([TEST-005](/articles/verification-va-validation)). Test case chỉ kiểm được cái thứ nhất. Cái thứ hai cần người đặt các yêu cầu cạnh nhau và hỏi «khách thấy gì?». Có ba hướng sửa để BA chọn: UI sinh khoá mới khi người dùng chủ động bấm lại sau một lỗi 4xx; không lưu `VOUCHER_USAGE_LIMIT` vì đó là lỗi có thể hết; hoặc thông báo rõ «hãy báo giá lại».

**Cặp liền kề không phải lúc nào cũng đủ.** Lập luận bắc cầu chỉ đúng khi các lớp kiểm tra thật sự là một chuỗi tuần tự. Trong mã, một lớp có thể có nhánh rẽ khác: `express.json` không nằm trong chuỗi R-VALID-02 mà chạy **trước** cả chuỗi. Hai ca 061 và 062 tồn tại vì error guessing nghi ngờ đúng chỗ đó. Bài học: kỹ thuật cho ra các ca **tối thiểu** theo mô hình; kinh nghiệm thêm ca ở chỗ mô hình có thể sai.

| Sai lầm thường gặp ở bài tổng hợp | Cách tránh |
|---|---|
| Nhảy từ SPEC thẳng vào code test | Viết condition và câu hỏi BA trước; Q-2 tìm được trước dòng code nào |
| Mọi ca dùng một app, chạy riêng xanh, chạy chung đỏ | Khôi phục hậu điều kiện (`/api/test/reset`) mỗi ca |
| Chỉ kiểm mã HTTP | Liệt kê mọi đầu ra quan sát được (mục 4.5) |
| Đọc «48 passed» thành «không có defect» | Báo cáo theo SPEC: 44 đạt, 3 không đạt; `it.fails` phải có nhãn defect |
| «100% yêu cầu được phủ» thay cho đánh giá | Kèm phạm vi, ca mỏng, rủi ro còn lại |

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 (K3).** Thêm biên cho `GIAM20` ở phía báo giá: khách đã có một chuyến `GIAM20` hôm nay, báo giá mới với `GIAM20` lúc 23:59:59,999 và lúc 00:00:00,000 hôm sau. Viết hai test case cụ thể với kết quả mong đợi, và nói chúng truy vết tới condition nào.

<details><summary><b>Lời giải</b></summary>
<p>R-VOUCHER-07 kiểm lượt cả khi báo giá. 23:59:59,999 ngày 04/11: đã có 1 chuyến trong ngày ⇒ <code>POST /api/quotes</code> 422 <code>VOUCHER_USAGE_LIMIT</code>. 00:00:00,000 ngày 05/11: 0 chuyến trong ngày mới ⇒ 200, <code>discount</code> 9.060, <code>total</code> 37.000 đ. Truy vết TCOND-BOOK-11 về mặt ý nghĩa, nhưng condition đó viết cho «thời điểm tạo chuyến». Cách đúng là thêm condition mới «ngày của lượt ở bước báo giá» thay vì nhét vào condition cũ. Nhét vào thì RTM sẽ nói sai condition nào đã được phủ.</p>
</details>

**Bài tập 2 (K3).** Thêm rủi ro RB-10 «khách mất báo giá sau lỗi hạ tầng tạm thời», K2 × T3 = 6, và gắn TC-BOOK-038 vào RB-10 thay cho RB-04. TC-BOOK-038 sẽ chạy ở vị trí nào?

<details><summary><b>Lời giải</b></summary>
<p>Mức 6 ⇒ vào nhóm cùng 003, 004 (RB-02). Hoà thì theo mã: 003, 004, 038. Lịch mới: 001, 012, 013, 030, 033, 035, 036 (mức 9), rồi 003, 004, <b>038</b> ⇒ vị trí <b>10</b>, thay vì 18. Kiểm bằng cách sửa <code>risks</code> trong bản sao của <code>thiet-ke.ts</code>: tiêu đề khối <code>describe</code> in số thứ tự.</p>
</details>

**Bài tập 3 (K2).** Q-5 nói client thêm trường lạ khi thử lại thì nhận 422. Thiết kế một ca chứng minh, chọn kỹ thuật, và nói vì sao ca này **không** mâu thuẫn với TC-BOOK-004 (trường lạ bị bỏ qua).

<details><summary><b>Lời giải</b></summary>
<p>EP (phân vùng «thân khác nội dung»): lần 1 <code>{ quoteId }</code> khoá K ⇒ 201; lần 2 <code>{ quoteId, clientVersion: "2.3" }</code> cùng khoá ⇒ 422 <code>IDEMPOTENCY_KEY_REUSED</code>. Không mâu thuẫn, vì hai quy tắc tác động ở hai lớp khác nhau. Idempotency (R-IDEM-02) so <b>toàn bộ thân JSON</b> trước khi xử lý. Bước định dạng (R-API-14) bỏ trường lạ <b>sau</b> đó. TC-BOOK-004 dùng khoá mới nên không gặp lớp idempotency. Tương tự, <code>{ quoteId }</code> và <code>{ quoteId, paymentMethod: "cash" }</code> cùng nghĩa nhưng khác JSON ⇒ 422 khi dùng cùng khoá.</p>
</details>

**Bài tập 4 (K3).** Viết test case chứng minh hệ quả nêu ở DR-TCD10-01: yêu cầu JSON hỏng không được đếm vào giới hạn tần suất. Kết quả mong đợi theo SPEC là gì? Bạn dự đoán lab cho kết quả gì?

<details><summary><b>Lời giải</b></summary>
<p>C001 gửi 5 <code>POST /api/trips</code> thân <code>{"quoteId":</code> (JSON hỏng), có <code>x-user</code> và khoá; rồi gửi yêu cầu thứ 6 hợp lệ với báo giá còn hạn. Theo R-API-13, mọi yêu cầu đã qua xác thực và vai trò đều được đếm, kể cả yêu cầu lỗi dữ liệu. Theo R-VALID-02, xác thực đứng trước định dạng. Vậy cả 5 yêu cầu phải được đếm, và yêu cầu thứ 6 ⇒ <b>429</b>. Dự đoán từ chuỗi middleware: 5 yêu cầu bị <code>express.json</code> từ chối trước <code>authenticate</code>, không đi tới <code>rateLimitTrips</code> ⇒ yêu cầu thứ 6 được <b>201</b>. Đây là dự đoán; hãy thêm ca vào bản sao và chạy để xác nhận trước khi ghi vào defect report.</p>
</details>

**Bài tập 5 (K2).** RTM báo «R-IDEM-02: Đạt, 2 lần chạy». Một đồng nghiệp kết luận «idempotency đã được kiểm đầy đủ». Bạn phản hồi thế nào?

<details><summary><b>Lời giải</b></summary>
<p>RTM nói mọi condition <b>đã khai báo</b> của R-IDEM-02 trong phạm vi bài có ca đạt. Nó không nói condition đã đủ. Còn thiếu: endpoint <code>/cancel</code> (phạm vi gồm <code>:id</code>), khoá giữ được bao lâu (SPEC đích ≥ 24 giờ, lab giữ suốt vòng đời tiến trình), nhiều bản máy chủ dùng chung kho khoá. Độ phủ luôn tương đối với tập coverage item (CTFL 1.4.4) — chất lượng của tập đó được kiểm ở bước review test analysis, không phải bằng con số phủ.</p>
</details>

**Câu 1 (K2).** Theo CTFL v4.0.1 mục 1.4.1, hoạt động nào tạo ra test execution schedule?

a) Test analysis
b) Test design
c) Test implementation
d) Test completion

<details><summary><b>Đáp án: c</b></summary>
<p>Test implementation chuẩn bị mọi thứ để chạy: test procedure, test suite, test execution schedule. Test analysis cho test condition (a), test design cho test case và coverage item (b), test completion cho báo cáo kết thúc (d).</p>
</details>

**Câu 2 (K2).** Trong bài, defect «khách bị kẹt với 422 đã lưu sau khi lượt voucher được trả» (Q-2) được phát hiện chủ yếu nhờ hoạt động nào?

a) Chạy test tự động
b) Đánh giá test basis trong test analysis và review test case
c) Đo code coverage
d) Ưu tiên test case

<details><summary><b>Đáp án: b</b></summary>
<p>TC-BOOK-037 <b>xanh</b> vì hệ thống đúng đặc tả; không lần chạy nào báo đỏ (a). Vấn đề lộ ra khi đặt R-UI-01, R-IDEM-05, R-VOUCHER-07 cạnh nhau (test analysis, mục 1.4.1) và khi reviewer sửa kết quả mong đợi của bản nháp (R-5). c, d không liên quan.</p>
</details>

**Câu 3 (K3).** Năm test case có mức rủi ro: A = 9, B = 6, C = 9, D = 4, E = 6. Mọi ca phụ thuộc D. Lịch risk-based (hoà theo chữ cái) sau khi sửa phụ thuộc là gì?

a) A, C, B, E, D
b) D, A, C, B, E
c) A, D, C, B, E
d) D, A, B, C, E

<details><summary><b>Đáp án: b</b></summary>
<p>Thứ tự thuần A, C, B, E, D. Xếp A cần D chạy trước ⇒ D, A; các ca sau đã thoả. a vi phạm phụ thuộc. c để C, B, E sau D nhưng A trước D. d xếp B (6) trước C (9).</p>
</details>

**Câu 4 (K2).** RTM sinh ra cho thấy mọi test case trỏ tới ít nhất một R-… và một condition có thật. Theo CTFL 1.4.4, điều này giúp chủ yếu việc gì?

a) Chứng minh hệ thống không còn defect
b) Phát hiện test case mồ côi và đánh giá độ phủ, ảnh hưởng của thay đổi
c) Thay cho review test case
d) Giảm thời gian chạy test

<details><summary><b>Đáp án: b</b></summary>
<p>Truy vết hai chiều phục vụ đo độ phủ, phân tích ảnh hưởng, kiểm toán. a sai: không bộ test nào chứng minh hết defect. c sai: review bắt được R-5, R-6, R-8 mà truy vết không thấy. d không liên quan.</p>
</details>

**Câu 5 (K3).** Để kiểm thứ tự bốn lỗi R-QUOTE-03 (NOT_FOUND → EXPIRED → ALREADY_USED → VOUCHER_USAGE_LIMIT) bằng cặp vi phạm liền kề, cần tối thiểu bao nhiêu ca nhiều lỗi?

a) 2
b) 3
c) 6
d) 15

<details><summary><b>Đáp án: b</b></summary>
<p>Chuỗi n = 4 bước ⇒ n − 1 = 3 cặp liền kề, như TC-BOOK-014. c là số mọi cặp C(4, 2), thừa vì thứ tự có tính bắc cầu. d là mọi tổ hợp khác rỗng 2⁴ − 1. a thiếu một cặp.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 1.4.1 Test Activities and Tasks (FL-1.4.1, K2); 1.4.3 Testware (FL-1.4.3, K2); 1.4.4 Traceability (FL-1.4.4, K2); 3.2 Feedback and Review Process; 4.2 Black-Box Test Techniques; 5.1.5 Test Case Prioritization (FL-5.1.5, K3) — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/
- ISO/IEC/IEEE 29119-3, *Software testing — Part 3: Test documentation* (mẫu test completion report, tham khảo).
- IETF, *The Idempotency-Key HTTP Header Field* (Internet-Draft, httpapi WG) — bối cảnh cho R-IDEM; SPEC lab là nguồn chính.
- Vitest — `test.fails`: https://vitest.dev/api/#test-fails · supertest: https://github.com/ladjs/supertest
- Lab Tới Nơi: `SPEC.md` (R-API-08, R-API-13, R-QUOTE-03/04, R-VOUCHER-07, R-IDEM, R-VALID-02, R-TEST-02, R-UI-01) và `articles/test-case-design-thuc-hanh-dat-chuyen/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TCD-01](/articles/test-case-design-test-analysis-test-condition) · [TCD-02](/articles/test-case-design-giai-phau-test-case) · [TCD-03](/articles/test-case-design-logic-va-cu-the) · [TCD-04](/articles/test-case-design-expected-result-test-oracle) · [TCD-05](/articles/test-case-design-test-data) · [TCD-06](/articles/test-case-design-phoi-hop-ky-thuat) · [TCD-07](/articles/test-case-design-user-story-acceptance-criteria) · [TCD-08](/articles/test-case-design-traceability-matrix) · [TCD-09](/articles/test-case-design-uu-tien-review-bao-tri) · [TEST-050 · API Testing từ góc nhìn code](/articles/api-testing-tu-goc-nhin-code) · [SYS-03 · Functional Test](/articles/system-testing-functional-test) · [Thực hành Series 01](/articles/thuc-hanh-series-01-phan-tich-tinh-nang-dat-xe)

Đây là bài cuối của nhánh TCD. Kỹ thuật riêng lẻ dùng trong bài có ở Series 04: [Equivalence Partitioning](/articles/equivalence-partitioning), [Boundary Value Analysis](/articles/boundary-value-analysis), [Decision Table Testing](/articles/decision-table-testing), [State Transition Testing](/articles/state-transition-testing-co-ban).
