---
slug: test-case-design-uu-tien-review-bao-tri
title: "Ưu tiên, review và bảo trì bộ test case: thanh toán VíXanh và idempotency của Tới Nơi"
summary: "Áp ba chiến lược ưu tiên test case của CTFL 5.1.5 (risk-based, coverage-based kiểu additional, requirements-based) lên 15 test case R-PAY và R-IDEM, xử lý phụ thuộc làm đảo thứ tự, review bộ test case bằng checklist 10 mục và bảy anti-pattern, rồi bảo trì khi SPEC đổi số lần thử lại từ 2 lên 3. Lab 40 test vitest."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-case-design, tcd-09, test-case-prioritization, risk-based-testing, review, checklist, traceability, maintenance, idempotency, vitest, toi-noi]
language: vi
---

Đội có 15 test case cho phần thanh toán VíXanh và idempotency của Tới Nơi, nhưng bản build mới tới lúc 16 giờ và buổi quyết định phát hành là 17 giờ. Chạy hết thì không kịp. Câu hỏi thực tế là: **chạy ca nào trước**, để nếu phải dừng giữa chừng thì phần đã chạy cho nhiều thông tin nhất? Bài này trả lời câu hỏi đó theo mục 5.1.5 của CTFL v4.0.1, với ba chiến lược ưu tiên và một quy tắc về phụ thuộc. Có thứ tự rồi thì còn hai việc thường bị bỏ quên. Việc thứ nhất là **review chính bộ test case** trước khi tin nó: một kết quả mong đợi mơ hồ thì chạy sớm cũng vô ích. Việc thứ hai là **bảo trì** bộ test case khi SPEC đổi, sao cho chỉ sửa đúng những ca bị ảnh hưởng.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TCD-09</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>TCD · Thiết kế Test Case</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Áp dụng</td></tr>
<tr><th scope="row">Test Level</th><td>System (qua API), component cho phần tính thứ tự</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Regression (phần bảo trì)</td></tr>
<tr><th scope="row">Test Technique</th><td>Test case prioritization; checklist-based reviewing (review technique)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/test-case-design-traceability-matrix">TCD-08 · Ma trận truy vết</a>; nên đọc <a href="/articles/checklist-based-review">TEST-028 · Checklist-based Review</a></td></tr>
<tr><th scope="row">Thực hành</th><td>15 test case R-PAY + R-IDEM viết thành dữ liệu, bốn chiến lược ưu tiên có xử lý phụ thuộc, review tự động phần máy kiểm được, chạy thật theo lịch risk-based, kịch bản bảo trì 2 → 3 lần thử lại: <code>examples/toi-noi-testing/articles/test-case-design-uu-tien-review-bao-tri/</code> (40 test)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 5 · mục 5.1.5 · FL-5.1.5 (K3) · Chương 3 · mục 3.2 (review process) · Chương 1 · mục 1.4.3, 1.4.4 · FL-1.4.3 (K2), FL-1.4.4 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Đến bài này của nhánh TCD, bạn đã có đủ nguyên liệu để viết một bộ test case tốt. [TCD-01](/articles/test-case-design-test-analysis-test-condition) cho test condition, [TCD-02](/articles/test-case-design-giai-phau-test-case) cho cách viết từng trường, [TCD-03](/articles/test-case-design-logic-va-cu-the) cho test procedure và test execution schedule. Sau đó là kết quả mong đợi ([TCD-04](/articles/test-case-design-expected-result-test-oracle)), test data ([TCD-05](/articles/test-case-design-test-data)), phối hợp kỹ thuật ([TCD-06](/articles/test-case-design-phoi-hop-ky-thuat)) và ma trận truy vết ([TCD-08](/articles/test-case-design-traceability-matrix)). Một bộ test case viết tốt vẫn còn ba câu hỏi cần trả lời trước khi dùng được lâu dài:

| Câu hỏi | Hoạt động | Mục trong bài |
|---|---|---|
| Thời gian không đủ, chạy ca nào trước? | Ưu tiên test case (test case prioritization) | 2.1, 4.2 |
| Bộ test case có đáng tin không? | Review bộ test case theo checklist | 2.3, 4.4, 4.5 |
| SPEC đổi thì sửa ca nào? | Phân tích ảnh hưởng qua RTM, bảo trì | 2.4, 4.6 |

Ưu tiên test case cần khi thời gian chạy bị giới hạn. Đó là lúc chạy tay trên staging, khi có hàng nghìn ca hồi quy, khi pipeline phải trả lời nhanh, hoặc khi lịch phát hành bị ép. Review bộ test case nên làm mỗi lần bộ test case được viết mới hay sửa lớn, trước khi đưa vào hồi quy. Lúc đó sửa một kết quả mong đợi chỉ mất vài phút. Nếu để lọt, kết quả mong đợi sai sẽ cho ra hàng chục lần chạy «đạt» sai. Bảo trì là việc cần làm mỗi khi test basis đổi.

Domain của bài là thanh toán VíXanh (R-PAY-03…06) và idempotency (R-IDEM-01…05). Hai nhóm yêu cầu này có rủi ro tiền bạc cao, có nhiều phụ thuộc giữa các bước, và ví giả của lab có đủ chế độ lỗi để chạy thật.

## 2. Lý thuyết theo ISTQB

### 2.1. CTFL v4.0.1 mục 5.1.5 — ưu tiên test case

Theo CTFL, sau khi test case và test procedure đã được đặc tả và gom thành test suite, có thể sắp các test suite đó vào một **test execution schedule**, tức thứ tự chạy. Có nhiều yếu tố để cân nhắc khi ưu tiên. Syllabus nêu ba chiến lược phổ biến nhất (diễn đạt lại):

- **Risk-based prioritization** (ưu tiên theo rủi ro). Thứ tự chạy dựa trên kết quả phân tích rủi ro (mục 5.2.3). Test case phủ rủi ro quan trọng nhất chạy trước.
- **Coverage-based prioritization** (ưu tiên theo độ phủ). Thứ tự chạy dựa trên độ phủ, ví dụ statement coverage. Test case đạt độ phủ cao nhất chạy trước. Biến thể **additional coverage prioritization** chọn ca đầu tiên là ca phủ nhiều nhất. Mỗi ca tiếp theo là ca **phủ thêm** được nhiều nhất những gì chưa được phủ.
- **Requirements-based prioritization** (ưu tiên theo yêu cầu). Thứ tự chạy dựa trên mức ưu tiên của các yêu cầu mà test case truy vết tới. Mức ưu tiên yêu cầu do stakeholder đặt. Test case gắn với yêu cầu quan trọng nhất chạy trước.

FL-5.1.5 ở mức **K3**: bạn phải **áp dụng** được các chiến lược trên một bộ dữ liệu cụ thể và ra đúng thứ tự, không chỉ kể tên. Câu thi K3 điển hình cho một bảng test case kèm rủi ro, độ phủ hoặc phụ thuộc, rồi hỏi «ca nào chạy thứ ba?». Mục 4.2 giải đúng dạng câu hỏi đó.

### 2.2. Phụ thuộc và nguồn lực

CTFL nói thêm hai điều làm thứ tự lý tưởng bị lệch.

**Phụ thuộc.** Lý tưởng là chạy theo đúng mức ưu tiên, nhưng cách đó có thể không làm được khi test case hoặc tính năng phụ thuộc lẫn nhau. Quy tắc của syllabus: nếu một ca ưu tiên **cao** phụ thuộc một ca ưu tiên **thấp** hơn, ca thấp hơn **phải chạy trước**. Phụ thuộc thắng ưu tiên, không có ngoại lệ.

**Nguồn lực.** Thứ tự chạy còn phải tính tới việc công cụ, môi trường test hay con người chỉ có mặt trong một khung giờ nhất định. Ví dụ ở Tới Nơi: sandbox của VíXanh thật chỉ mở cho đội QA từ 14 tới 16 giờ (giả định), nên mọi ca cần sandbox phải dồn vào khung đó, dù mức rủi ro của chúng thế nào.

### 2.3. CTFL mục 3.2 — review, áp lên test case

Test case là một work product, nên review được như đặc tả hay code. Mục 3.2 mô tả quy trình review gồm năm hoạt động. **Planning** xác định phạm vi, mục tiêu, tiêu chí kết thúc. **Review initiation** bảo đảm mọi người và tài liệu đã sẵn sàng. **Individual review**: từng reviewer đọc và ghi anomaly, dùng một review technique như checklist-based. **Communication and analysis**: phân tích anomaly, quyết định cái nào là defect. **Fixing and reporting**: sửa, báo cáo, kiểm tiêu chí kết thúc. Cũng trong mục 3.2 có các vai trò (manager, author, moderator, scribe, reviewer, review leader) và bốn loại review: informal review, walkthrough, technical review, inspection.

Bài này dùng checklist ở bước individual review. Cách xây và giữ checklist sống đã có ở [TEST-028](/articles/checklist-based-review); khác ở đây là **đối tượng review**. TEST-028 review **đặc tả** (R-PAY có tách «chưa rõ» khỏi «thất bại» không?), còn bài này review **test case** (kết quả mong đợi có kiểm chứng được không, có phụ thuộc ẩn không?).

### 2.4. Truy vết và bảo trì (mục 1.4.3, 1.4.4)

Mục 1.4.3 xếp test case, test procedure và test execution schedule vào testware của test design và test implementation. Testware cũng phải được quản lý cấu hình và bảo trì như mã nguồn. Mục 1.4.4 nêu lợi ích của truy vết: đánh giá độ phủ, và **xác định ảnh hưởng của thay đổi**. Khi một yêu cầu đổi, RTM cho biết ngay những test condition, test case nào truy vết tới nó. Đó là danh sách phải xem lại, thay vì đọc lại cả bộ.

### 2.5. Thuật ngữ

| Thuật ngữ | Tiếng Việt trong bài | Ghi chú |
|---|---|---|
| test case prioritization | ưu tiên test case | CTFL 5.1.5 |
| test execution schedule | không dịch | thứ tự chạy các test suite |
| risk-based / coverage-based / requirements-based prioritization | ưu tiên theo rủi ro / độ phủ / yêu cầu | ba chiến lược của 5.1.5 |
| additional coverage prioritization | ưu tiên theo độ phủ thêm | thuật toán tham lam |
| coverage item | coverage item | ở đây là test condition |
| dependency | phụ thuộc | ca thấp hơn chạy trước |
| individual review | review cá nhân | bước dùng checklist |
| anomaly | anomaly | ghi khi review, chưa chắc là defect |
| impact analysis | phân tích ảnh hưởng | dựa trên RTM |

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Các thứ sau là <b>quy ước của bài</b>, CTFL không quy định. Thang 1–3 cho khả năng và tác động, mức rủi ro bằng tích. Mức rủi ro của một test case lấy bằng mức cao nhất trong các rủi ro nó giảm. Ưu tiên yêu cầu 1–3 là <b>giả định bài tập</b> vì SPEC không có. Khi hoà điểm thì xếp theo mã tăng dần. Cách sửa thứ tự theo phụ thuộc là «kéo ca tiền điều kiện lên ngay trước». Checklist CL-TC 10 mục do bài tự xây. Thay một quy ước là thứ tự sẽ đổi; điều cần giữ là quy ước phải được ghi rõ trước khi tính.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

| Mã | Quy tắc (tóm tắt SPEC v1.1) |
|---|---|
| R-PAY-03 | Chuyến `vixanh` hoàn thành ⇒ gọi VíXanh thu đúng `fare`, khoá `trip:<id>:fare` |
| R-PAY-04 | Mỗi lần gọi hết giờ sau 3 giây; hết giờ hoặc lỗi tạm thời ⇒ thử lại **tối đa 2 lần** (tổng tối đa 3 lần gọi), **cùng khoá**; bị từ chối ⇒ không thử lại |
| R-PAY-05 | Phí huỷ > 0: `vixanh` ⇒ thu với khoá `trip:<id>:cancel-fee`; `cash` ⇒ `paymentStatus = cash`; phí 0 ⇒ `none` |
| R-PAY-06 | Thu thất bại không chặn hoàn thành hay huỷ: `paymentStatus = failed`, API 200 |
| R-IDEM-01 | `POST /api/trips` và `/cancel` bắt buộc `Idempotency-Key` 8–128 ký tự `A–Z a–z 0–9 _ . : -`; sai ⇒ 400 |
| R-IDEM-02 | Cùng phạm vi (người gọi, endpoint), cùng khoá, cùng nội dung ⇒ trả lại đúng phản hồi đầu, header `Idempotent-Replayed: true` |
| R-IDEM-03 | Cùng khoá, khác nội dung ⇒ 422 `IDEMPOTENCY_KEY_REUSED` |
| R-IDEM-04 | Yêu cầu đầu chưa xong ⇒ 409 `IDEMPOTENCY_IN_PROGRESS` |
| R-IDEM-05 | 2xx và 4xx của bước xử lý được lưu; 5xx không lưu; 401/403/429 không gắn với khoá |

### 3.2. System context

Ở kiến trúc đích, Trip/Booking gọi Payment & Wallet, Payment & Wallet gọi cổng thanh toán. Idempotency nằm ở API Gateway/Booking. Trong lab, `driverStep` và `cancelByCustomer` gọi `chargeWithRetry` tới ví VíXanh giả. Ví giả có năm chế độ (R-TEST-03): `none`, `decline`, `error`, `timeout`, `lost_response`. Idempotency do `runIdempotent` lo, khoá lưu trong bộ nhớ tiến trình. Webhook, ledger, đối soát nằm ngoài lab (R-ARCH-02).

### 3.3. Sổ rủi ro

Khả năng (K) và tác động (T) chấm theo thang 1–3. Đây là đầu vào của chiến lược risk-based, được làm ở bước test analysis ([TCD-01](/articles/test-case-design-test-analysis-test-condition)) chứ không phải lúc xếp lịch.

| Mã | Rủi ro (failure ta sợ) | K | T | Mức |
|---|---|---:|---:|---:|
| RR-01 | Trừ tiền khách hai lần khi thử lại hoặc gửi lại | 2 | 3 | 6 |
| RR-02 | Gửi lại yêu cầu tạo ra chuyến trùng | 3 | 2 | 6 |
| RR-03 | Ví lỗi làm kẹt chuyến hoặc thử lại sai luật | 2 | 2 | 4 |
| RR-04 | Khoá dùng lại hoặc gửi đồng thời bị xử lý như yêu cầu mới | 2 | 2 | 4 |
| RR-05 | Thu sai số tiền hoặc sai khoá idempotency | 1 | 3 | 3 |
| RR-06 | Lỗi tạm thời bị lưu thành kết quả vĩnh viễn | 1 | 3 | 3 |
| RR-07 | Trạng thái thanh toán của phí huỷ sai (`cash`/`none`) | 2 | 1 | 2 |
| RR-08 | Thiếu khoá hoặc lỗi đã lưu được xử lý khác đặc tả | 2 | 1 | 2 |

Tổng mức rủi ro của sổ là 30. Con số này dùng ở mục 4.3 để đo «5 ca đầu chạm được bao nhiêu phần rủi ro».

## 4. Phân tích kỹ thuật

### 4.1. Bộ 15 test case và 19 test condition

Phân tích R-PAY-03…06 và R-IDEM-01…05 cho 19 test condition: `TCOND-PAY-01…10` và `TCOND-IDEM-01…09`. Đây là **coverage item** của chiến lược coverage-based: mức phủ được đo trên các condition, không đo trên dòng code. Bộ test case hiện có:

| Test case | Nội dung (rút gọn) | Yêu cầu | Rủi ro | Mức RR | Ưu tiên YC | Condition phủ |
|---|---|---|---|---:|---:|---|
| TC-PAY-001 | Hoàn thành `vixanh`, ví bình thường ⇒ `paid`, 1 lần gọi | R-PAY-03 | RR-05 | 3 | 2 | PAY-01 |
| TC-PAY-002 | Hết giờ, hết giờ, rồi trả lời ⇒ `paid` ở lần 3, cùng khoá | R-PAY-04 | RR-01 | 6 | 1 | PAY-02 |
| TC-PAY-003 | Lỗi tạm thời liên tục ⇒ `failed` sau đúng 3 lần, chuyến vẫn `completed` | R-PAY-04, 06 | RR-03 | 4 | 1 | PAY-03, 04, 10 |
| TC-PAY-004 | Bị từ chối ⇒ `failed` sau 1 lần, chuyến vẫn `completed` | R-PAY-04, 06 | RR-03 | 4 | 1 | PAY-05, 10 |
| TC-PAY-005 | Đã trừ nhưng mất phản hồi, lần sau trả lời ⇒ `paid`, trừ một lần | R-PAY-04 | RR-01 | 6 | 1 | PAY-02, 06 |
| TC-PAY-006 | Khách huỷ `vixanh` có phí ⇒ thu 10.000 đ khoá `cancel-fee` | R-PAY-05 | RR-05 | 3 | 3 | PAY-07 |
| TC-PAY-007 | Khách huỷ `cash` có phí ⇒ `cash`, không gọi ví | R-PAY-05 | RR-07 | 2 | 3 | PAY-08 |
| TC-PAY-008 | Khách huỷ `vixanh` khi `searching` ⇒ 0 đ, `none` | R-PAY-05 | RR-07 | 2 | 3 | PAY-09 |
| TC-IDEM-001 | Thiếu khoá, khoá 7 ký tự, ký tự lạ ⇒ 400 | R-IDEM-01 | RR-08 | 2 | 3 | IDEM-01 |
| TC-IDEM-002 | Gửi lại cùng thân, đảo thứ tự khoá ⇒ 201 replayed, một chuyến | R-IDEM-02 | RR-02 | 6 | 1 | IDEM-02, 03 |
| TC-IDEM-003 | Cùng khoá, khác thân ⇒ 422 | R-IDEM-03 | RR-04 | 4 | 2 | IDEM-04 |
| TC-IDEM-004 | Yêu cầu hai khi yêu cầu đầu chưa xong ⇒ 409 | R-IDEM-04 | RR-04 | 4 | 2 | IDEM-05 |
| TC-IDEM-005 | `QUOTE_EXPIRED` được lưu ⇒ gửi lại 422 replayed | R-IDEM-05 | RR-08 | 2 | 3 | IDEM-06 |
| TC-IDEM-006 | 500 không lưu ⇒ gửi lại cùng khoá được xử lý như mới (201) | R-IDEM-05 | RR-06 | 3 | 3 | IDEM-07 |
| TC-IDEM-007 | Gửi lại lệnh huỷ của TC-PAY-006 ⇒ 200 replayed, ví không gọi thêm | R-IDEM-02, PAY-05 | RR-01 | 6 | 1 | IDEM-02, 09 |

Ưu tiên yêu cầu do PO đặt (giả định bài tập): **1** cho R-PAY-04 và R-IDEM-02, vì sai là mất tiền hoặc đặt trùng. **2** cho R-PAY-03, R-PAY-06, R-IDEM-03, R-IDEM-04. **3** cho R-PAY-05, R-IDEM-01, R-IDEM-05. Một test case truy vết tới nhiều yêu cầu thì lấy mức **cao nhất**: TC-IDEM-007 gắn R-IDEM-02 (1) và R-PAY-05 (3) nên được 1.

**Phụ thuộc đã khai báo.** Trong test procedure chạy tay trên staging, TC-IDEM-007 gửi lại **chính** yêu cầu huỷ mà TC-PAY-006 vừa gửi, cùng chuyến, cùng khoá `huy-0001`. Tạo một chuyến `vixanh` có tài xế nhận rồi chờ đủ thời gian để có phí tốn vài phút mỗi lần, nên đội chọn dùng lại hậu điều kiện của TC-PAY-006. Như vậy **TC-IDEM-007 phụ thuộc TC-PAY-006**, và quan hệ này được ghi trong trường tiền điều kiện. Mục 6 bàn về chuyện bản tự động hoá nên bỏ phụ thuộc này.

Ngay bảng trên đã lộ một lỗ hổng: TCOND-IDEM-08 (phạm vi khoá theo người gọi: khách khác dùng cùng khoá thì không bị ảnh hưởng) **không có test case nào**. Độ phủ test condition tối đa của bộ là 18/19. Lỗ hổng này sẽ được bắt lại ở bước review (mục 4.5).

### 4.2. Bài tập K3: ba chiến lược cho ra ba thứ tự

**Risk-based.** Sắp theo mức rủi ro giảm dần, hoà thì theo mã. Mức 6: IDEM-002, IDEM-007, PAY-002, PAY-005. Mức 4: IDEM-003, IDEM-004, PAY-003, PAY-004. Mức 3: IDEM-006, PAY-001, PAY-006. Mức 2: IDEM-001, IDEM-005, PAY-007, PAY-008. Thứ tự thuần là:

```text
IDEM-002 → IDEM-007 → PAY-002 → PAY-005 → IDEM-003 → IDEM-004 → PAY-003 → PAY-004
→ IDEM-006 → PAY-001 → PAY-006 → IDEM-001 → IDEM-005 → PAY-007 → PAY-008
```

Thứ tự này **vi phạm phụ thuộc**: IDEM-007 (vị trí 2) cần hậu điều kiện của PAY-006 (vị trí 11). Theo CTFL, ca thấp hơn phải chạy trước. Cách sửa: duyệt theo thứ tự ưu tiên, trước khi xếp một ca thì xếp các ca nó phụ thuộc mà chưa được xếp. PAY-006 (mức 3) được kéo lên ngay trước IDEM-007 (mức 6):

```text
IDEM-002 → PAY-006 → IDEM-007 → PAY-002 → PAY-005 → IDEM-003 → IDEM-004 → PAY-003
→ PAY-004 → IDEM-006 → PAY-001 → IDEM-001 → IDEM-005 → PAY-007 → PAY-008
```

Để ý rằng ta kéo ca tiền điều kiện **lên**, chứ không đẩy IDEM-007 **xuống**. Đẩy xuống cũng tôn trọng phụ thuộc, nhưng làm một ca mức 6 phải đợi tới vị trí 11. Cách kéo lên giữ cho ca rủi ro cao chạy sớm nhất có thể.

**Requirements-based.** Ưu tiên YC 1: IDEM-002, IDEM-007, PAY-002, PAY-003, PAY-004, PAY-005. Ưu tiên 2: IDEM-003, IDEM-004, PAY-001. Ưu tiên 3 là sáu ca còn lại. Sau khi sửa theo phụ thuộc:

```text
IDEM-002 → PAY-006 → IDEM-007 → PAY-002 → PAY-003 → PAY-004 → PAY-005 → IDEM-003
→ IDEM-004 → PAY-001 → IDEM-001 → IDEM-005 → IDEM-006 → PAY-007 → PAY-008
```

**Coverage-based, dạng tổng.** Đếm số condition mỗi ca phủ. PAY-003 phủ 3 condition nên đứng đầu. IDEM-002, IDEM-007, PAY-004, PAY-005 mỗi ca phủ 2. Các ca còn lại mỗi ca phủ 1. Năm vị trí đầu sau khi sửa phụ thuộc là PAY-003, IDEM-002, PAY-006, IDEM-007, PAY-004. Dạng tổng có một điểm yếu: nó không biết hai ca phủ **trùng** nhau. Trên đường phủ tích luỹ `3, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 16, 17, 18` có một bậc phẳng ở PAY-002, vì condition duy nhất PAY-002 phủ (TCOND-PAY-02) đã được PAY-005 phủ trước đó.

**Coverage-based, dạng additional.** Đây là thuật toán tham lam (greedy), tính từng bước:

| Bước | Phủ thêm của các ca còn lại (chỉ ghi ca lớn nhất) | Chọn | Phủ thêm | Đã phủ |
|---:|---|---|---:|---:|
| 1 | PAY-003: 3 | PAY-003 | 3 | 3 |
| 2 | IDEM-002, IDEM-007, PAY-005: 2 (PAY-004 còn 1 vì PAY-10 đã phủ) | IDEM-002 (mã nhỏ nhất) | 2 | 5 |
| 3 | PAY-005: 2; IDEM-007 còn 1 vì IDEM-02 đã phủ | PAY-005 | 2 | 7 |
| 4…14 | mỗi ca còn lại phủ thêm 1, riêng PAY-002 phủ thêm 0 | theo mã | 1 | 8…18 |
| 15 | PAY-002: 0 | PAY-002 | 0 | 18 |

Phần phủ thêm qua 15 bước là `3, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0`. Khi sửa phụ thuộc, PAY-006 được chèn ngay trước IDEM-007 ở vị trí 9. Lịch cuối cùng phủ đủ 18 condition sau 14 ca. Ca thứ 15 là **PAY-002**, ca mà risk-based xếp thứ tư.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 330" role="img" aria-label="Năm ca chạy đầu tiên của bốn chiến lược ưu tiên sau khi sửa phụ thuộc. Risk-based: IDEM-002, PAY-006, IDEM-007, PAY-002, PAY-005, phủ 6 trên 19 condition, chạm 15 trên 30 mức rủi ro. Requirements-based: IDEM-002, PAY-006, IDEM-007, PAY-002, PAY-003, phủ 8, chạm 19. Tổng coverage: PAY-003, IDEM-002, PAY-006, IDEM-007, PAY-004, phủ 8, chạm 19. Additional coverage: PAY-003, IDEM-002, PAY-005, IDEM-001, IDEM-003, phủ 9, chạm 22. PAY-006 là ca được kéo lên do phụ thuộc.">
<text x="20" y="26" class="t-b">5 ca chạy đầu tiên (đã sửa phụ thuộc)</text>
<text x="700" y="26" class="t-b">condition phủ</text>
<text x="815" y="26" class="t-b">mức RR chạm</text>
<text x="20" y="72" class="t-b">Risk-based</text>
<rect x="190" y="50" width="96" height="36" rx="6" class="f-surface s-line"/><text x="238" y="73" text-anchor="middle" class="t-small t-mono">IDEM-002</text>
<rect x="292" y="50" width="96" height="36" rx="6" class="f-warn-soft s-warn"/><text x="340" y="73" text-anchor="middle" class="t-small t-mono t-warn">PAY-006</text>
<rect x="394" y="50" width="96" height="36" rx="6" class="f-surface s-line"/><text x="442" y="73" text-anchor="middle" class="t-small t-mono">IDEM-007</text>
<rect x="496" y="50" width="96" height="36" rx="6" class="f-surface s-line"/><text x="544" y="73" text-anchor="middle" class="t-small t-mono">PAY-002</text>
<rect x="598" y="50" width="96" height="36" rx="6" class="f-surface s-line"/><text x="646" y="73" text-anchor="middle" class="t-small t-mono">PAY-005</text>
<text x="740" y="73" text-anchor="middle" class="t-b">6/19</text><text x="855" y="73" text-anchor="middle" class="t-b">15/30</text>
<text x="20" y="132" class="t-b">Requirements-based</text>
<rect x="190" y="110" width="96" height="36" rx="6" class="f-surface s-line"/><text x="238" y="133" text-anchor="middle" class="t-small t-mono">IDEM-002</text>
<rect x="292" y="110" width="96" height="36" rx="6" class="f-warn-soft s-warn"/><text x="340" y="133" text-anchor="middle" class="t-small t-mono t-warn">PAY-006</text>
<rect x="394" y="110" width="96" height="36" rx="6" class="f-surface s-line"/><text x="442" y="133" text-anchor="middle" class="t-small t-mono">IDEM-007</text>
<rect x="496" y="110" width="96" height="36" rx="6" class="f-surface s-line"/><text x="544" y="133" text-anchor="middle" class="t-small t-mono">PAY-002</text>
<rect x="598" y="110" width="96" height="36" rx="6" class="f-surface s-line"/><text x="646" y="133" text-anchor="middle" class="t-small t-mono">PAY-003</text>
<text x="740" y="133" text-anchor="middle" class="t-b">8/19</text><text x="855" y="133" text-anchor="middle" class="t-b">19/30</text>
<text x="20" y="192" class="t-b">Coverage (tổng)</text>
<rect x="190" y="170" width="96" height="36" rx="6" class="f-pk-soft s-pk"/><text x="238" y="193" text-anchor="middle" class="t-small t-mono">PAY-003</text>
<rect x="292" y="170" width="96" height="36" rx="6" class="f-pk-soft s-pk"/><text x="340" y="193" text-anchor="middle" class="t-small t-mono">IDEM-002</text>
<rect x="394" y="170" width="96" height="36" rx="6" class="f-warn-soft s-warn"/><text x="442" y="193" text-anchor="middle" class="t-small t-mono t-warn">PAY-006</text>
<rect x="496" y="170" width="96" height="36" rx="6" class="f-pk-soft s-pk"/><text x="544" y="193" text-anchor="middle" class="t-small t-mono">IDEM-007</text>
<rect x="598" y="170" width="96" height="36" rx="6" class="f-pk-soft s-pk"/><text x="646" y="193" text-anchor="middle" class="t-small t-mono">PAY-004</text>
<text x="740" y="193" text-anchor="middle" class="t-b">8/19</text><text x="855" y="193" text-anchor="middle" class="t-b">19/30</text>
<text x="20" y="252" class="t-b">Coverage (additional)</text>
<rect x="190" y="230" width="96" height="36" rx="6" class="f-phys-soft s-phys"/><text x="238" y="253" text-anchor="middle" class="t-small t-mono">PAY-003</text>
<rect x="292" y="230" width="96" height="36" rx="6" class="f-phys-soft s-phys"/><text x="340" y="253" text-anchor="middle" class="t-small t-mono">IDEM-002</text>
<rect x="394" y="230" width="96" height="36" rx="6" class="f-phys-soft s-phys"/><text x="442" y="253" text-anchor="middle" class="t-small t-mono">PAY-005</text>
<rect x="496" y="230" width="96" height="36" rx="6" class="f-phys-soft s-phys"/><text x="544" y="253" text-anchor="middle" class="t-small t-mono">IDEM-001</text>
<rect x="598" y="230" width="96" height="36" rx="6" class="f-phys-soft s-phys"/><text x="646" y="253" text-anchor="middle" class="t-small t-mono">IDEM-003</text>
<text x="740" y="253" text-anchor="middle" class="t-b t-phys">9/19</text><text x="855" y="253" text-anchor="middle" class="t-b t-phys">22/30</text>
<rect x="190" y="290" width="22" height="14" rx="3" class="f-warn-soft s-warn"/><text x="220" y="301" class="t-small">ca được kéo lên vì TC-IDEM-007 phụ thuộc nó — mức rủi ro 3, ưu tiên YC 3</text>
</svg>
</div>
<figcaption><b>Hình 1 · Bốn lịch chạy, cùng 15 test case.</b> «Mức RR chạm» là tổng mức của các rủi ro có ít nhất một ca đã chạy, trên tổng 30 của sổ rủi ro. Mọi con số được khẳng định trong <code>uu-tien.test.ts</code> của lab.</figcaption>
</figure>

### 4.3. Đọc kết quả: chiến lược nào «đúng»?

Không chiến lược nào đúng tuyệt đối. Mỗi chiến lược tối ưu cho một câu hỏi khác nhau, và bảng số liệu cho thấy rõ điều đó.

**Risk-based dồn đúng vào rủi ro nặng nhất, nhưng không đa dạng.** Ba trong năm ca đầu (IDEM-007, PAY-002, PAY-005) cùng giảm RR-01, nên sau năm ca chỉ có ba rủi ro được chạm, tổng mức 15/30. Nếu ca đầu tiên đã cho thấy RR-01 ổn, hai ca sau mang ít thông tin mới hơn một ca về RR-03 hay RR-04. Đây không phải lỗi của chiến lược. Đó là hệ quả của cách quy đổi «mức rủi ro của test case = mức của rủi ro cao nhất nó giảm». Một biến thể hay dùng là áp tinh thần «additional» cho rủi ro: mỗi bước chọn ca chạm được nhiều mức rủi ro **chưa chạm** nhất (bài tập 2).

**Additional coverage đa dạng nhất, nhưng mù rủi ro.** Năm ca đầu phủ 9/19 condition và chạm 22/30 mức rủi ro, cao nhất trong bốn lịch. Nhưng chiến lược này đẩy PAY-002 (hết giờ rồi thử lại, mức rủi ro 6) xuống **cuối cùng**, vì theo mô hình coverage, PAY-002 không phủ thêm gì. Mô hình đó sai ở chỗ quan trọng. «Hết giờ» (cổng không trừ tiền) và «mất phản hồi» (cổng đã trừ tiền) cùng thuộc TCOND-PAY-02 «hết giờ thì thử lại cùng khoá», nhưng là hai tình huống tiền khác hẳn nhau. Vậy coverage-based chỉ tốt bằng **độ mịn của coverage item**. Condition viết quá thô thì ca quan trọng bị coi là thừa.

**Requirements-based phản ánh ý kiến stakeholder.** Lịch này gần với risk-based vì PO đặt ưu tiên theo cùng nỗi lo về tiền và đặt trùng. Khác biệt nằm ở PAY-003 và PAY-004 (R-PAY-04 ưu tiên 1) được lên trước IDEM-003 và IDEM-004. Chiến lược này mạnh khi stakeholder hiểu nghiệp vụ hơn đội test. Nó yếu khi ưu tiên yêu cầu được đặt một lần rồi bỏ đó, trong lúc rủi ro đã đổi.

**Phụ thuộc thắng mọi chiến lược.** Ở cả bốn lịch, PAY-006 đều phải đứng trước IDEM-007. Muốn PAY-006 không chiếm chỗ sớm, cách duy nhất là **gỡ phụ thuộc**: cho IDEM-007 tự dựng tiền điều kiện của mình. Bản tự động hoá trong lab làm đúng như vậy (mục 6).

Trong thực tế, đội thường **ghép** chiến lược. Lấy risk-based làm khung, trong cùng mức rủi ro thì xếp theo additional coverage, rồi sửa theo phụ thuộc và nguồn lực. CTFL không cấm cách ghép này. Điều syllabus yêu cầu là bạn áp được từng chiến lược và tôn trọng phụ thuộc.

### 4.4. Checklist CL-TC để review bộ test case

Mười mục dưới đây là câu hỏi đóng, mỗi mục bắt một loại anti-pattern. Cột «máy kiểm được» cho biết phần nào lab tự kiểm bằng heuristic. Những phần còn lại cần người đọc hiểu SPEC.

| Mã | Câu hỏi | Anti-pattern bắt được | Máy kiểm được? |
|---|---|---|---|
| CL-TC-01 | Mỗi ca có **một** mục tiêu, nói được trong một câu? | Nhiều mục tiêu trong một ca | Một phần (đếm mục tiêu) |
| CL-TC-02 | Kết quả mong đợi có giá trị cụ thể cho mọi đầu ra quan sát được (HTTP, thân, trạng thái lưu, lời gọi ví, số tiền)? | Kết quả mong đợi mơ hồ | Một phần (không có số, mã hay tên trường ⇒ nghi) |
| CL-TC-03 | Ca chạy độc lập được; mọi phụ thuộc vào ca khác khai báo bằng mã? | Phụ thuộc thứ tự ẩn | Một phần (nhắc «ca trước» mà không khai báo) |
| CL-TC-04 | Mốc thời gian kèm múi giờ hoặc tương đối với đồng hồ giả; không chờ thời gian thật? | Ngày giờ cứng, múi giờ ngầm | Một phần |
| CL-TC-05 | Không trùng ca khác ở cả tiền điều kiện, dữ liệu, bước và kết quả? | Chép–dán | Có (so chữ ký) |
| CL-TC-06 | Chỉ kiểm qua giao diện của đối tượng test và bản giả ở biên, không kiểm cấu trúc bên trong? | Kiểm chi tiết cài đặt | Một phần (danh sách tên nội bộ) |
| CL-TC-07 | Truy vết tới ít nhất một R-… **có trong SPEC** và một test condition? | Ca không truy vết được | Có |
| CL-TC-08 | Test data có thật sự chạm điều kiện mà ca nhắm tới? | Gắn nhãn nhưng không chạm | Không |
| CL-TC-09 | Có hậu điều kiện hoặc dọn dẹp, không để lại trạng thái cho ca sau? | Rò trạng thái | Không |
| CL-TC-10 | Ưu tiên có ghi và có lý do (rủi ro, yêu cầu)? | Ưu tiên tuỳ hứng | Có (trường bắt buộc) |

CL-TC-08 lấy từ bài học «gắn nhãn requirement nhưng dữ liệu không chạm điều kiện» ở [SYS-03](/articles/system-testing-functional-test). CL-TC-02 và CL-TC-06 nối với [TCD-04](/articles/test-case-design-expected-result-test-oracle): kết quả mong đợi phải là mọi đầu ra quan sát được, và lấy từ test oracle độc lập.

### 4.5. Review bản nháp: chín phát hiện

Bản nháp gồm 9 ca do một thành viên mới viết. Áp CL-TC ở bước individual review cho ra:

| # | Ca (bản nháp) | Trích | Mục | Mức | Cách sửa |
|---|---|---|---|---|---|
| F-1 | TC-PAY-003 | Kết quả mong đợi: «Hệ thống xử lý lỗi đúng và thử lại theo quy định» | CL-TC-02 | Major | `completed`, `failed`, HTTP 200; ví nhận **đúng 3** lần gọi cùng khoá, trừ 0 đ |
| F-2 | TC-PAY-004 | Ba mục tiêu: từ chối, lỗi tạm thời, hết giờ trong một ca, ba chuyến nối nhau | CL-TC-01 | Major | Tách: PAY-004 chỉ còn `decline`; hai mục tiêu kia đã có PAY-003, PAY-002 |
| F-3 | TC-IDEM-007 | Tiền điều kiện «Chuyến đã huỷ ở ca trước», không khai báo phụ thuộc | CL-TC-03 | Major | Khai báo `dependsOn: TC-PAY-006`, ghi rõ khoá `huy-0001` và hậu điều kiện cần |
| F-4 | TC-PAY-006 | «Nhận lúc 09:00 ngày 04/11/2026», «Chờ 5 phút» | CL-TC-04 | Minor | «T0 = 04/11/2026 09:00 giờ VN (+07:00)», dời đồng hồ giả tới T0 + 5 phút |
| F-5 | TC-PAY-005 | Dữ liệu, bước, kết quả y hệt PAY-002 («timeout, timeout»), chỉ tiêu đề nói mất phản hồi | CL-TC-05 | Major | Dữ liệu `lost_response` rồi bình thường; mong đợi 2 lần gọi, trừ 46.000 đ **một lần** |
| F-6 | TC-IDEM-002 | «`MemoryIdempotencyStore` có 1 bản ghi; fingerprint là SHA-256 của thân» | CL-TC-06 | Major | Hai phản hồi 201 cùng `trip.id`, lần hai có `Idempotent-Replayed`, danh sách chuyến chờ có 1 chuyến |
| F-7 | TC-PAY-009 | «Thanh toán nhanh, người dùng hài lòng», không mã R-…, không condition | CL-TC-07, 02 | Major | Loại khỏi bộ functional; nếu cần, viết lại ở bộ hiệu năng theo R-NFR-05 với ngưỡng đo được |
| F-8 | TC-IDEM-006 | Kết quả mong đợi «Lần gửi lại không phải replay» | CL-TC-02 | Major | R-IDEM-05 nói client **được thử lại**: lần gửi lại phải trả 201 với chuyến mới |
| F-9 | *(cả bộ)* | TCOND-IDEM-08 không có ca nào (phát hiện ở RTM, ngoài 9 ca) | CL-TC-07 (chiều ngược) | Minor | Thêm ca: C002 dùng cùng khoá với C001 ⇒ 201 riêng, không trả chuyến của C001 |

F-1 đến F-8 ứng với tám trong chín ca nháp (TC-PAY-002 không có phát hiện). Chúng cho chín dòng phát hiện vì F-7 dính hai mục, và lab kiểm được cả chín dòng. F-9 phải nhìn RTM theo chiều ngược ([TCD-08](/articles/test-case-design-traceability-matrix)), không thấy được khi đọc từng ca.

Hai phát hiện đáng bàn ở bước **communication and analysis**:

**F-5 nguy hiểm hơn vẻ ngoài.** Bản chép–dán vẫn chạy «đạt», vì «timeout, timeout» đúng là cho kết quả như mong đợi. Thế là bộ test **trông như** có ca cho tình huống «cổng đã trừ tiền nhưng mất phản hồi», trong khi thật ra không có. Ma trận truy vết vẫn xanh ở TCOND-PAY-06. Chỉ người đọc kỹ dữ liệu so với tiêu đề mới thấy.

**F-8 dẫn tới một defect thật.** Sửa kết quả mong đợi thành «gửi lại ⇒ 201» xong, chạy thì ca **đỏ**: lab trả 409 `QUOTE_ALREADY_USED`. Đây là defect DR-050-01 đã ghi ở [TEST-050](/articles/api-testing-tu-goc-nhin-code): báo giá bị đánh dấu đã dùng trước khi tạo chuyến lỗi. Kết quả mong đợi mơ hồ của bản nháp đã che defect này, vì «không phải replay» thì đúng là đạt. Đây là lý do mạnh nhất để review test case: **một oracle yếu làm defect đi qua mà không ai biết**.

Một giới hạn cần nói rõ: máy chỉ bắt được dạng. Lab có một ca «viết đẹp nhưng sai SPEC» (mong đợi 2 lần gọi thay vì 3), và bộ kiểm tự động không phát hiện gì. CL-TC-02 hỏi «có giá trị cụ thể không». Câu hỏi «giá trị đó có **đúng** không» thì chỉ reviewer đọc R-PAY-04 trả lời được.

### 4.6. Bảo trì khi SPEC đổi: thử lại 2 → 3 lần

Giả định bài tập: SPEC v1.2 đổi R-PAY-04 thành «thử lại tối đa **3** lần (tổng tối đa 4 lần gọi)». Phân tích ảnh hưởng theo mục 1.4.4 đi qua bốn bước.

1. **Tra RTM theo chiều xuôi.** R-PAY-04 → TCOND-PAY-02…06 → TC-PAY-002, 003, 004, 005. Ngoài ra còn TC-IDEM-007 qua khoá idempotency, nhưng ca này không phụ thuộc số lần thử. Bốn ca cần xem lại, mười một ca còn lại không phải đụng tới.
2. **Tính lại kết quả mong đợi từ câu chữ SPEC mới, cho từng ca.**

| Ca | Dữ liệu ví | Mong đợi v1.1 | Mong đợi v1.2 | Đổi? |
|---|---|---|---|---|
| TC-PAY-002 | timeout, timeout, rồi bình thường | `paid`, 3 lần gọi | `paid`, 3 lần gọi | Không |
| TC-PAY-003 | error liên tục | `failed`, **3** lần gọi | `failed`, **4** lần gọi | **Có** |
| TC-PAY-004 | decline | `failed`, 1 lần gọi | `failed`, 1 lần gọi | Không |
| TC-PAY-005 | lost_response, rồi bình thường | `paid`, 2 lần gọi | `paid`, 2 lần gọi | Không |
| **TC-PAY-010** (mới) | timeout × 3, rồi bình thường | `failed`, 3 lần gọi | `paid`, **4** lần gọi | Ca biên mới |

3. **Tìm khoảng trống.** Ba ca «không đổi» vẫn xanh trên hệ thống mới, nhưng **không ca nào chạm lần gọi thứ 4**. Đây là biên mới của yêu cầu, nên phải thêm TC-PAY-010, theo đúng tinh thần BVA ([TEST-032](/articles/boundary-value-analysis)): biên dịch chuyển thì ca biên cũng dịch theo.
4. **Kiểm cả các tài liệu khác nói cùng một điều.** R-NFR-05 cũng ghi «tối đa 2 lần thử lại». Một thay đổi chỉ sửa R-PAY-04 sẽ để SPEC tự mâu thuẫn. Đây là phát hiện cho người viết change request, và lại cũng do RTM chỉ ra, vì R-NFR-05 truy vết tới cùng các ca đó. Thời gian chờ tối đa của `/complete` cũng tăng từ khoảng 9 lên khoảng 12 giây (4 × 3 giây). Đây là câu hỏi cho đội vận hành.

Lab kiểm chứng phân tích này. Chạy bộ v1.1 trên hệ thống cấu hình 3 lần thử lại thì **chỉ TC-PAY-003 đỏ**, đúng như dự đoán. Bộ v1.2 (đã cập nhật PAY-003, thêm PAY-010) xanh cả năm ca.

Bài học bảo trì quan trọng nhất nằm trong cách viết test: số «3 lần gọi» **không được rải** trong năm ca. Lab viết kết quả mong đợi theo một hằng tham số của SPEC (`retries`) qua một oracle nhỏ đọc từ câu chữ R-PAY-04. Đổi SPEC là đổi một hằng, rồi để bộ test tự chỉ ra ca nào đổi kết quả. Nhưng oracle đó **không** được import từ mã sản phẩm (`maxRetries` trong `DEFAULT_CONFIG`). Nếu import, khi ai đó cấu hình sai thành 5, test sẽ «đồng ý» theo.

## 5. Thực hành

### 5.1. Tổ chức lab

Mã ở `examples/toi-noi-testing/articles/test-case-design-uu-tien-review-bao-tri/`:

| Tệp | Vai trò |
|---|---|
| `bo-test-case.ts` | 19 condition, 8 rủi ro, ưu tiên yêu cầu, 15 test case viết thành **dữ liệu** |
| `uu-tien.ts` | `riskBased`, `requirementsBased`, `totalCoverage`, `additionalCoverage`, `respectDependencies`, `violations` |
| `uu-tien.test.ts` | Khẳng định mọi thứ tự và số liệu ở mục 4.2, 4.3 (9 test) |
| `ban-nhap.ts`, `review-tu-dong.ts`, `review.test.ts` | Bản nháp 9 ca, bản sửa 8 ca, phần máy kiểm được của CL-TC (4 test) |
| `thuc-thi.test.ts` | 15 ca chạy thật qua API theo lịch risk-based (16 test) |
| `bao-tri.test.ts` | Kịch bản 2 → 3 lần thử lại (11 test) |

Viết bộ test case thành dữ liệu cho phép **tính** lịch chạy, thay vì sắp tay. Hàm sửa phụ thuộc chỉ vài dòng:

```ts
// uu-tien.ts (trích): trước khi xếp một ca, xếp các ca nó phụ thuộc (theo thứ tự ưu tiên của chúng)
const visit = (id: string) => {
  if (done.has(id)) return;
  if (visiting.has(id)) throw new Error(`Vòng phụ thuộc tại ${id}`);
  visiting.add(id);
  [...deps.get(id)!].sort((a, b) => order.indexOf(a) - order.indexOf(b)).forEach(visit);
  visiting.delete(id);
  done.add(id);
  scheduled.push(id);
};
order.forEach(visit);
```

Lịch tính ra được dùng **trực tiếp** để đăng ký test. `thuc-thi.test.ts` duyệt `SCHEDULE` và tạo `it` theo đúng thứ tự đó. Vitest chạy các test trong một file tuần tự theo thứ tự đăng ký, nên thứ tự chạy thật bằng thứ tự đã tính:

```ts
export const SCHEDULE = respectDependencies(riskBased(TEST_CASES, RISKS), TEST_CASES);
SCHEDULE.forEach((id, i) => {
  const tc = TEST_CASES.find((t) => t.id === id)!;
  const name = `#${String(i + 1).padStart(2, '0')} ${id} · ${tc.reqs.join(', ')} · ${tc.title}`;
  if (KNOWN_DEFECT[id] && !process.env.SHOW_RED) it.fails(`${name} [defect ${KNOWN_DEFECT[id]}]`, IMPL[id]!);
  else it(name, IMPL[id]!);
});
```

### 5.2. Test data và các ca thật

Dữ liệu chung: T0 = 04/11/2026 09:00 giờ VN (ngoài cao điểm), khách C001, tài xế D101, SG, xe máy, 7,4 km. Cước tính tay là 12.000 + 4.500 × 7,4 = 45.300, làm tròn lên thành **46.000 đ**. Phí huỷ xe máy là **10.000 đ** (R-CANCEL-03), chuyến được nhận rồi huỷ sau 5 phút (ETA 5, chưa trễ). Ví giả được điều khiển bằng `failNext(...)` cho vài lần gọi kế tiếp và `setMode(...)` cho chế độ lâu dài. Hết giờ mỗi lần gọi rút còn 20 ms để test nhanh, trong khi SPEC là 3 giây. Ca TC-PAY-005 kiểm cả lời gọi lẫn tiền thật sự bị trừ:

```ts
async 'TC-PAY-005'() {
  const id = await inTransitTrip();
  system.payments.failNext('lost_response');                 // lần 1: ví đã trừ nhưng không trả lời
  const res = await complete(id).expect(200);
  expect(res.body.trip.paymentStatus).toBe('paid');
  expect(system.payments.calls.map((c) => c.idempotencyKey)).toEqual([`trip:${id}:fare`, `trip:${id}:fare`]);
  expect(system.payments.totalCharged()).toBe(FARE);         // 46.000 đ, đúng một lần
},
```

### 5.3. Chạy

```bash
cd examples/toi-noi-testing
npx vitest run articles/test-case-design-uu-tien-review-bao-tri --reporter=verbose
```

Kết quả thật (Node 22, Vitest 3.2.7, rút gọn tên khối):

```text
✓ #01 TC-IDEM-002 · R-IDEM-02 · Gửi lại cùng thân (đảo thứ tự khoá) ⇒ 201 replayed, một chuyến
✓ #02 TC-PAY-006 · R-PAY-05 · Khách huỷ chuyến vixanh có phí ⇒ thu 10.000 đ khoá cancel-fee
✓ #03 TC-IDEM-007 · R-IDEM-02, R-PAY-05 · Gửi lại lệnh huỷ của TC-PAY-006 ⇒ 200 replayed, ví không bị gọi thêm
✓ #04 TC-PAY-002 · R-PAY-04 · Ví hết giờ, hết giờ, rồi trả lời ⇒ paid ở lần gọi thứ 3, cùng khoá
…
✓ #10 TC-IDEM-006 · R-IDEM-05 · 500 không được lưu: gửi lại cùng khoá ⇒ được xử lý như mới (201) [defect DR-050-01]
…
✓ #15 TC-PAY-008 · R-PAY-05 · Khách huỷ chuyến vixanh khi searching ⇒ 0 đ, none, không gọi ví
✓ Bảo trì … > bộ v1.1 chạy trên hệ thống đã đổi sang 3 lần: đỏ đúng một ca — TC-PAY-003; …

 Test Files  4 passed (4)
      Tests  40 passed (40)
```

40 test gồm: 9 test kiểm thứ tự, 4 test review, 16 test thực thi (15 ca cộng 1 test kiểm đủ cài đặt) và 11 test bảo trì. Ca #10 chạy bằng `it.fails`: nó **xanh vì defect còn**. Chạy `SHOW_RED=1 npx vitest run articles/test-case-design-uu-tien-review-bao-tri/thuc-thi` thì thấy đỏ thật: `expected 409 to be 201`, 1 failed, 15 passed. Nói theo SPEC, 15 ca chạy thật có **14 đạt, 1 không đạt (DR-050-01)**. Đừng đọc «40 passed» thành «không có defect».

## 6. Góc nhìn nâng cao

**Phụ thuộc trong bản tự động hoá.** Trong `thuc-thi.test.ts`, TC-IDEM-007 **tự dựng** chuyến của mình (tạo, nhận, chờ 5 phút trên đồng hồ giả, huỷ), rồi mới gửi lại. Phụ thuộc vào TC-PAY-006 chỉ còn trong dữ liệu, để minh hoạ thuật toán. Lý do: chạy tự động thì dựng tiền điều kiện chỉ tốn vài chục mili giây, và một ca độc lập có thể chạy riêng, chạy song song, chạy theo bất kỳ lịch nào. Còn chạy tay trên staging thì dựng một chuyến có phí tốn vài phút người làm, nên phụ thuộc có khai báo là đánh đổi hợp lý. Quy tắc thực hành: **phụ thuộc được phép khi có khai báo và có lý do chi phí, còn phụ thuộc ẩn thì không bao giờ được phép**.

**Phụ thuộc ẩn hay gặp nhất không nằm ở văn bản mà ở trạng thái dùng chung.** Ví dụ: bộ đếm giới hạn tần suất 5 yêu cầu/60 giây (R-API-13) dùng chung giữa các ca, khoá idempotency lặp lại giữa các ca, voucher «1 lần/ngày» bị ca trước tiêu mất. Chạy theo thứ tự A thì xanh, đổi lịch ưu tiên sang thứ tự B thì đỏ. Cách phát hiện: chạy bộ test theo **nhiều** thứ tự (ngẫu nhiên có seed, hoặc đảo ngược). Lab của bài này tạo hệ thống mới cho mỗi ca nên không bị.

**«Kéo lên» và «hàng đợi ưu tiên» cho lịch khác nhau.** Cách sửa phụ thuộc thứ hai là Kahn có hàng đợi ưu tiên: mỗi bước chọn ca ưu tiên cao nhất trong số các ca **đã sẵn sàng**. Với dữ liệu ở đây, cách này chạy IDEM-002, PAY-002, PAY-005, … rồi mới tới PAY-006, IDEM-007. Kết quả là ca mức 6 IDEM-007 bị đẩy xuống sau nhiều ca mức 4. Cả hai cách đều tôn trọng phụ thuộc. CTFL chỉ đòi tôn trọng phụ thuộc, không quy định thuật toán, nên hãy ghi rõ mình dùng cách nào.

**Ưu tiên khi chạy hồi quy liên tục.** Coverage-based theo statement coverage, như ví dụ của syllabus, hữu ích khi có số liệu coverage đo thật cho từng ca, chẳng hạn từ công cụ đo của lab (`npm run test:coverage`). Nhưng thứ tự tối ưu hôm nay sẽ cũ đi khi mã đổi. Bộ hồi quy lớn thường tính lại thứ tự mỗi đêm, và thêm tín hiệu «ca này đỏ gần đây» hay «ca này chạm file vừa sửa». Đó là kỹ thuật mở rộng ngoài CTFL, chi tiết ở [SYS-06](/articles/system-testing-regression-test).

**Review test case không thay review đặc tả.** Nếu R-PAY-06 sai (TEST-028 đã chỉ ra: «hết lượt thử ⇒ `failed`» trong khi cổng có thể đã trừ tiền), thì một bộ test case hoàn hảo theo CL-TC vẫn kiểm đúng một đặc tả sai. Review test case hỏi «test case có diễn đạt đúng đặc tả không». Review đặc tả hỏi «đặc tả có đúng nhu cầu không».

| Sai lầm thường gặp | Cách tránh |
|---|---|
| Xếp lịch xong mới phát hiện phụ thuộc | Khai báo `dependsOn` lúc viết ca; để công cụ kiểm vi phạm |
| Gán mọi ca mức rủi ro «cao» | Dùng sổ rủi ro có K × T; nếu mọi thứ đều 9 thì chưa phân tích |
| Coverage item quá thô ⇒ ca quan trọng bị coi là thừa | Tách condition theo khác biệt có hậu quả (hết giờ khác mất phản hồi) |
| Review chỉ đọc từng ca | Đọc thêm RTM theo chiều ngược để thấy condition không có ca (F-9) |
| Sửa SPEC nhưng chỉ sửa ca đỏ | Ca xanh có thể đã mất ý nghĩa biên; tìm biên mới (TC-PAY-010) |
| Hằng số của SPEC rải khắp bộ test | Một chỗ khai báo, oracle tham số hoá, không import cấu hình sản phẩm |

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 (K3).** Thêm phụ thuộc: TC-PAY-002 (mức 6) phụ thuộc TC-PAY-001 (mức 3), vì cả hai dùng chung một phiên sandbox mà PAY-001 khởi tạo. Viết lại 6 vị trí đầu của lịch risk-based.

<details><summary><b>Lời giải</b></summary>
<p>Thứ tự thuần không đổi: IDEM-002, IDEM-007, PAY-002, PAY-005, … Duyệt lần lượt: IDEM-002 xếp vị trí 1. IDEM-007 cần PAY-006 ⇒ xếp PAY-006 rồi IDEM-007 (vị trí 2, 3). PAY-002 cần PAY-001 ⇒ xếp PAY-001 rồi PAY-002 (vị trí 4, 5). Vị trí 6 là PAY-005. Kết quả: <b>IDEM-002, PAY-006, IDEM-007, PAY-001, PAY-002, PAY-005</b>. Hai ca mức 3 lên trước ca mức 6 — đúng quy tắc CTFL. Muốn kiểm, sửa <code>dependsOn</code> trong <code>bo-test-case.ts</code> của bản sao và chạy <code>respectDependencies</code>.</p>
</details>

**Bài tập 2 (K3).** Áp tinh thần «additional» cho rủi ro: mỗi bước chọn ca chạm được tổng mức rủi ro **chưa chạm** lớn nhất, hoà thì theo mã. Năm ca đầu là gì, và chạm bao nhiêu trên 30?

<details><summary><b>Lời giải</b></summary>
<p>Bước 1: các ca mức 6 hoà (RR-01 hoặc RR-02 đều 6) ⇒ theo mã: IDEM-002 (RR-02). Bước 2: ca chạm RR-01 được 6 ⇒ IDEM-007 (mã nhỏ nhất trong IDEM-007, PAY-002, PAY-005), nhưng phụ thuộc kéo PAY-006 (RR-05, 3) lên trước. Bước tiếp theo: RR-03 và RR-04 cùng 4 ⇒ IDEM-003 (RR-04, mã nhỏ hơn IDEM-004, PAY-003). Rồi RR-03 ⇒ PAY-003. Năm ca đầu: <b>IDEM-002, PAY-006, IDEM-007, IDEM-003, PAY-003</b>. Rủi ro chạm: RR-01, 02, 03, 04, 05 = 6 + 6 + 4 + 4 + 3 = <b>23/30</b>, cao hơn cả 4 lịch ở Hình 1. Vẫn tôn trọng rủi ro (các ca mức 6 lên đầu) mà đa dạng hơn risk-based thuần.</p>
</details>

**Bài tập 3 (K3).** Viết test case cho phát hiện F-9 (TCOND-IDEM-08) theo đúng CL-TC: mục tiêu, tiền điều kiện, dữ liệu, bước, kết quả mong đợi, truy vết, ưu tiên.

<details><summary><b>Lời giải mẫu</b></summary>
<p><b>TC-IDEM-008</b> · Mục tiêu: khoá idempotency có phạm vi theo người gọi. Truy vết: R-IDEM-02, TCOND-IDEM-08, RR-04 (mức 4). Tiền điều kiện: dữ liệu mẫu, T0 = 04/11/2026 09:00 giờ VN trên đồng hồ giả; C001 và C002 mỗi người một báo giá TD-01 còn hạn. Bước: (1) C001 <code>POST /api/trips</code> khoá <code>dat-chuyen-0034</code>; (2) C002 <code>POST /api/trips</code> cùng khoá với báo giá của mình. Kết quả mong đợi: cả hai 201; phản hồi của C002 <b>không</b> có <code>Idempotent-Replayed</code>, <code>trip.customerId</code> = C002, <code>trip.id</code> khác chuyến của C001; danh sách chuyến chờ của D101 có 2 chuyến. Nếu C002 nhận lại chuyến của C001 thì đó là lỗi lộ dữ liệu (R-API-05) chứ không chỉ sai idempotency. Ca này chạy thật ở capstone <a href="/articles/test-case-design-thuc-hanh-dat-chuyen">TCD-10</a> (TC-BOOK-034).</p>
</details>

**Bài tập 4 (K2).** SPEC đổi R-PAY-05: phí huỷ xe máy từ 10.000 đ lên 12.000 đ (thực chất là đổi R-CANCEL-03). Dùng RTM, những ca nào của bộ này cần xem lại? Ca nào chỉ cần chạy lại?

<details><summary><b>Lời giải</b></summary>
<p>Ca có số tiền 10.000 đ trong kết quả mong đợi phải <b>sửa</b>: TC-PAY-006, TC-PAY-007, TC-IDEM-007. TC-PAY-008 (phí 0) chỉ cần <b>chạy lại</b>, vì không có số tiền phí trong kết quả mong đợi. Mười một ca còn lại không truy vết tới R-CANCEL-03. Điểm tinh: RTM của bộ này truy vết tới R-PAY-05 chứ không tới R-CANCEL-03, nên tra theo R-CANCEL-03 sẽ ra <b>rỗng</b>. Đó là lỗ hổng truy vết: test data có giá trị phụ thuộc yêu cầu khác thì phải truy vết cả yêu cầu đó (<a href="/articles/test-case-design-traceability-matrix">TCD-08</a>).</p>
</details>

**Câu 1 (K2).** Theo CTFL v4.0.1, trong additional coverage prioritization, ca thứ hai được chọn thế nào?

a) Ca có độ phủ tổng cao thứ hai
b) Ca phủ thêm được nhiều nhất những gì ca thứ nhất chưa phủ
c) Ca gắn với yêu cầu ưu tiên cao thứ hai
d) Ca có rủi ro cao nhất trong các ca còn lại

<details><summary><b>Đáp án: b</b></summary>
<p>a là coverage-based dạng tổng. c là requirements-based. d là risk-based. b đúng định nghĩa của syllabus: mỗi ca tiếp theo đạt độ phủ <b>thêm</b> cao nhất.</p>
</details>

**Câu 2 (K3).** Bốn test case: T1 rủi ro cao, T2 rủi ro trung bình, T3 rủi ro cao, T4 rủi ro thấp. T1 phụ thuộc T4. Theo risk-based và quy tắc phụ thuộc của CTFL, ca nào chạy **thứ hai**? (Hoà thì theo số thứ tự.)

a) T3
b) T1
c) T4
d) T2

<details><summary><b>Đáp án: b</b></summary>
<p>Thứ tự thuần: T1, T3, T2, T4. T1 phụ thuộc T4 nên T4 phải chạy trước T1: <b>T4, T1, T3, T2</b>. Ca thứ hai là T1. a quên phụ thuộc hoặc đẩy T1 xuống thay vì kéo T4 lên; c là ca thứ nhất; d nhầm thứ tự rủi ro.</p>
</details>

**Câu 3 (K3).** Ba ca phủ condition: A = {1, 2, 3}, B = {3, 4}, C = {4, 5, 6}. Thứ tự theo additional coverage (hoà thì theo chữ cái)?

a) A, B, C
b) A, C, B
c) C, A, B
d) B, A, C

<details><summary><b>Đáp án: b</b></summary>
<p>Bước 1: A và C cùng phủ 3, hoà ⇒ A. Bước 2: B phủ thêm 1 (chỉ 4), C phủ thêm 3 (4, 5, 6) ⇒ C. Bước 3: B phủ thêm 0. Kết quả A, C, B. a là thứ tự chữ cái; c chọn sai ca đầu khi hoà; d bắt đầu bằng ca phủ ít nhất.</p>
</details>

**Câu 4 (K2).** Theo CTFL mục 3.2, checklist được dùng ở hoạt động nào của quy trình review?

a) Planning
b) Review initiation
c) Individual review
d) Fixing and reporting

<details><summary><b>Đáp án: c</b></summary>
<p>Reviewer áp review technique, ví dụ checklist-based, khi review cá nhân. Planning có thể <b>chọn</b> checklist, nhưng không áp nó. b là bước chuẩn bị người và tài liệu. d là bước sửa và báo cáo.</p>
</details>

**Câu 5 (K2).** Yêu cầu R-PAY-04 đổi. Theo CTFL mục 1.4.4, truy vết giúp gì?

a) Tự động sửa kết quả mong đợi
b) Xác định test condition và test case bị ảnh hưởng để xem lại
c) Bảo đảm test case mới sẽ đạt
d) Thay thế cho phân tích rủi ro

<details><summary><b>Đáp án: b</b></summary>
<p>Truy vết hỗ trợ đánh giá ảnh hưởng của thay đổi. Nó không sửa test (a), không bảo đảm kết quả (c), không thay phân tích rủi ro (d). Ví dụ mục 4.6 vẫn cần người tìm biên mới.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 5.1.5 Test Case Prioritization, LO FL-5.1.5 (K3); mục 3.2 Feedback and Review Process; mục 1.4.3 Testware, FL-1.4.3 (K2); mục 1.4.4 Traceability, FL-1.4.4 (K2) — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (mục *test case prioritization*, *test execution schedule*, *traceability*)
- ISO/IEC 20246:2017, *Software and systems engineering — Work product reviews* (CTFL dẫn cho quy trình review).
- Rothermel, G., Untch, R., Chu, C., Harrold, M. J. (2001), «Prioritizing test cases for regression testing», *IEEE Transactions on Software Engineering* 27(10) — nguồn kinh điển của total và additional coverage prioritization.
- Lab Tới Nơi: `SPEC.md` (R-PAY-03…06, R-IDEM-01…05, R-NFR-05, R-TEST-03) và `articles/test-case-design-uu-tien-review-bao-tri/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TCD-08 · Ma trận truy vết](/articles/test-case-design-traceability-matrix) · [TEST-028 · Checklist-based Review](/articles/checklist-based-review) · [TEST-027 · Review Process](/articles/review-process-va-cac-vai-tro) · [TEST-050 · API Testing từ góc nhìn code](/articles/api-testing-tu-goc-nhin-code) · [Maintenance Testing](/articles/maintenance-testing) · [SYS-06 · Regression Test](/articles/system-testing-regression-test)

Bài tiếp theo: [TCD-10 · Capstone: thiết kế trọn bộ test case đặt chuyến](/articles/test-case-design-thuc-hanh-dat-chuyen).
