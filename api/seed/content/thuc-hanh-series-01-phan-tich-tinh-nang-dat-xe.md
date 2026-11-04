---
slug: thuc-hanh-series-01-phan-tich-tinh-nang-dat-xe
title: "Thực hành Series 01: phân tích tính năng đặt xe từ điểm đón đến khi hoàn thành chuyến"
summary: "Bài tập tổng hợp Series 01: tự phân tích luồng đặt xe của Tới Nơi — mục tiêu test, đối tượng test, phạm vi trong/ngoài, rủi ro và test condition truy vết tới SPEC — rồi đối chiếu lời giải mẫu và test chạy thật trên lab."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-s01-lab, test-analysis, test-scope, risk, traceability, toi-noi]
language: vi
---

Mười bài của Series 01 cho bạn từng mảnh: testing là gì, vì sao cần, bảy nguyên lý, error–defect–failure, quy trình kiểm thử, testware, vai trò. Bài này bắt bạn ghép chúng lại trên **một tính năng thật**: khách nhập điểm đón, đặt chuyến, tài xế nhận, đón khách và hoàn thành chuyến. Bạn nhận đề trước, tự làm, rồi mới mở lời giải mẫu — mọi kết luận trong lời giải trích mã yêu cầu `R-…` của SPEC lab và được kiểm bằng test chạy thật.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-S01-LAB</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S01 · Fundamentals of Software Testing — bài thực hành cuối series</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>System (phân tích ở mức hệ thống, gợi ý mức thấp hơn cho từng đối tượng)</td></tr>
<tr><th scope="row">Test Type</th><td>Functional (rủi ro phi chức năng chỉ nêu, chưa đo)</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng — mới tới test analysis; kỹ thuật thiết kế ở Series 04</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/bay-nguyen-ly-kiem-thu">TEST-003</a>, <a href="/articles/error-defect-failure-root-cause">TEST-004</a>, <a href="/articles/muc-tieu-kiem-thu-va-quality-assurance">TEST-006</a>, <a href="/articles/quy-trinh-kiem-thu-test-process">TEST-007</a>, <a href="/articles/testware-san-pham-kiem-thu">TEST-010</a>, <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab <code>toi-noi-testing</code>: <code>SPEC.md</code> + <code>articles/thuc-hanh-series-01-phan-tich-tinh-nang-dat-xe/</code> (vitest, 11 test)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.1.1, 1.4.1, 1.4.3, 1.4.4 · FL-1.1.1 (K1), FL-1.4.1 (K2), FL-1.4.3 (K2), FL-1.4.4 (K2) · Chương 2 · mục 2.1.5 · FL-2.1.5 (K2) · giới thiệu sớm Chương 5 · mục 5.2 · FL-5.2.1 (K1), FL-5.2.2 (K2), FL-5.2.3 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Bài này không dạy khái niệm mới. Nó là một **bài tập test analysis**: đọc test basis, trả lời «test cái gì, test tới đâu, cái gì đáng sợ nhất», và ghi kết quả thành testware có truy vết. Theo [quy trình kiểm thử của ISTQB](/articles/quy-trinh-kiem-thu-test-process), test analysis trả lời «test cái gì?»; test design (Series 04) mới trả lời «test thế nào?».

| Việc trong bài | Kiến thức dùng | Học ở |
|---|---|---|
| Viết mục tiêu test | Mục tiêu điển hình, phụ thuộc ngữ cảnh | [TEST-001](/articles/software-testing-la-gi-testing-khac-debugging), [TEST-006](/articles/muc-tieu-kiem-thu-va-quality-assurance) |
| Chọn đối tượng, phạm vi | Không thể test hết; test object khác test basis | [TEST-003](/articles/bay-nguyen-ly-kiem-thu), [TEST-007](/articles/quy-trinh-kiem-thu-test-process) |
| Viết rủi ro | Rủi ro = failure ta sợ, gây bởi defect | [TEST-002](/articles/vi-sao-can-kiem-thu), [TEST-004](/articles/error-defect-failure-root-cause) |
| Test condition, truy vết | Test analysis, traceability | [TEST-007](/articles/quy-trinh-kiem-thu-test-process), [TEST-010](/articles/testware-san-pham-kiem-thu) |
| Làm trước khi có code | Shift left, review đặc tả | [TEST-008](/articles/kiem-thu-trong-sdlc-agile-waterfall) |
| Ai làm phần nào | Testing role, test management role | [TEST-009](/articles/vai-tro-trong-kiem-thu) |

**Khi nào làm việc này ngoài đời?** Ngay khi có bản nháp đặc tả hoặc user story, trước khi có code. Phân tích sớm vừa tìm được lỗ hổng trong đặc tả (lúc rẻ nhất để sửa), vừa cho cả đội một danh sách rủi ro chung để ưu tiên.

**Cách dùng bài:** đọc đề ở mục 3, tự làm 60–90 phút trên giấy hoặc bảng tính, rồi mới mở các khối «Lời giải mẫu» ở mục 4. Mục 5 cho bạn chạy test trên lab để kiểm những gì lời giải mẫu khẳng định.

## 2. Lý thuyết theo ISTQB

Chỉ ôn lại; mỗi khái niệm đã có bài riêng.

- **Test objective** — lý do ta test. CTFL v4.0.1 mục 1.1.1 liệt kê mục tiêu điển hình: đánh giá work product; gây failure, tìm defect; đạt coverage cần thiết; giảm mức rủi ro chất lượng; kiểm yêu cầu đã đặc tả; kiểm tuân thủ hợp đồng/pháp lý; cung cấp thông tin để ra quyết định; tạo niềm tin; xác nhận hệ thống đúng mong đợi. Mục tiêu **đổi theo ngữ cảnh** ([TEST-006](/articles/muc-tieu-kiem-thu-va-quality-assurance)).
- **Test object** là thứ được test (hàm, module, API, cả hệ thống). **Test basis** là tài liệu/tri thức nói hệ thống *phải* làm gì — ở đây là `SPEC.md`.
- **Test analysis** phân tích test basis để tìm tính năng kiểm được, định nghĩa và **ưu tiên test condition theo rủi ro**, đồng thời đánh giá chính test basis để tìm defect (CTFL 1.4.1). **Test condition** là một khía cạnh kiểm được, ví dụ «báo giá hết hạn ở đúng mốc 5 phút» — chưa có dữ liệu cụ thể, chưa phải test case.
- **Testware** của test analysis là test condition đã ưu tiên và defect report về test basis; risk register thuộc test planning (CTFL 1.4.3, [TEST-010](/articles/testware-san-pham-kiem-thu)). **Traceability** nối yêu cầu ↔ rủi ro ↔ test condition ↔ test case ↔ kết quả, để đánh giá coverage, rủi ro còn lại và tác động khi yêu cầu đổi (CTFL 1.4.4).
- **Test scope** — cái gì test, cái gì không. CTFL không có LO riêng cho từ này nhưng coi nó là phần «ngữ cảnh kiểm thử» của test plan (5.1.1) và là thứ phân tích rủi ro sản phẩm tác động tới (FL-5.2.3). Phải chọn phạm vi vì không thể test toàn bộ ([TEST-003](/articles/bay-nguyen-ly-kiem-thu)).
- **Risk** — sự kiện có thể xảy ra và gây hậu quả xấu, đặc trưng bởi **risk likelihood** và **risk impact**; hai yếu tố cho **risk level**, theo cách định lượng là tích của chúng (CTFL 5.2.1, 5.2.3). **Product risk** gắn với chất lượng sản phẩm; **project risk** gắn với quản lý dự án: tổ chức, con người, kỹ thuật, nhà cung cấp (CTFL 5.2.2). Chương 5 học kỹ ở Series 12.

| Thuật ngữ | Tiếng Việt | Ghi chú |
|---|---|---|
| test objective / test object / test basis | mục tiêu / đối tượng / cơ sở kiểm thử | ba thứ hay bị lẫn |
| test analysis / test condition | phân tích kiểm thử / điều kiện kiểm thử | «test cái gì» |
| test scope | phạm vi kiểm thử | trong / ngoài |
| product risk / project risk | rủi ro sản phẩm / rủi ro dự án | CTFL 5.2.2 |
| risk likelihood / impact / level | khả năng / tác động / mức rủi ro | CTFL 5.2.1 |
| residual risk | rủi ro còn lại | chưa giảm được bằng test |
| traceability | truy vết | CTFL 1.4.4 |

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Ba thứ trong lời giải là <b>quy ước thực hành</b>, không phải nội dung CTFL: thang 1–3 cho khả năng và tác động (CTFL chỉ nói mức rủi ro tính bằng phép nhân hoặc tra ma trận); bảng phạm vi «trong / ngoài» kèm lý do (dạng phổ biến trong test plan; ISO/IEC/IEEE 29119-3 có phần phạm vi trong tài liệu test plan); và các mã <code>TCD-xx</code>, <code>PR-xx</code>, <code>PJ-xx</code> do bài tự đặt để truy vết.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Tính năng «Đặt xe» của [Tới Nơi](/articles/du-an-mau-toi-noi): (1) khách nhập điểm đón, điểm đến, chọn `bike`/`car`, có thể nhập voucher, chọn tiền mặt hoặc ví «VíXanh»; (2) hệ thống báo giá có hạn dùng; (3) khách đặt, chuyến chuyển sang tìm tài xế; (4) tài xế cùng thành phố, đúng loại xe nhận — chỉ một người được nhận; (5) tài xế báo đang tới, đã đón, bắt đầu chở, hoàn thành; chuyến trả bằng VíXanh thì thu tiền khi hoàn thành; (6) trong lúc chờ, khách có thể huỷ (có thể mất phí), tài xế có thể huỷ (chuyến quay lại tìm), hệ thống huỷ nếu tìm quá lâu.

### 3.2. Business rules — phần SPEC cần đọc

Test basis là `SPEC.md` v1.1 của lab ([GitHub](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing)). Mục bưu kiện và đồ ăn không cần cho đề chính.

| Mục SPEC | Mã | Cần nắm |
|---|---|---|
| 0. Vị trí của lab | R-ARCH-01…03 | Module ↔ service; những gì **không có** trong lab; móc kiểm thử |
| 1. Quy định chung | R-GEN-02, R-GEN-06 | Giờ Việt Nam; **lab không có điểm đón/điểm đến — client gửi thẳng quãng đường** |
| 2. Cước | R-FARE-03, R-FARE-09 | Thứ tự tính; ví dụ oracle 52.000 đ |
| 6. Báo giá | R-QUOTE-01…04 | Hạn 5 phút; một báo giá một chuyến; không đổi giá hồi tố |
| 8. Vòng đời | R-STATE-01…07 | 8 trạng thái, chuyển hợp lệ, hết 5 phút tìm tài xế, nhật ký |
| 9. Nhận chuyến | R-ACCEPT-01…07 | Điều kiện, một người thắng, nhận lại là idempotent |
| 10. Phí huỷ | R-CANCEL-01…06 | Mốc 2 phút, tài xế trễ, lý do phí |
| 11. Thanh toán | R-PAY-01…06 | Hết giờ 3 giây, thử lại cùng khoá, không chặn nghiệp vụ |
| 12–13. Idempotency, API | R-IDEM-01…05, R-API-05/10/13 | Gửi lại cùng khoá; chống IDOR; lệnh tài xế; giới hạn tần suất |
| 16. Phi chức năng | R-NFR-02/04/05 | p95 Booking API &lt; 500 ms; lan truyền trạng thái p95 &lt; 2–3 giây |

### 3.3. System context

Kiến trúc đích là microservices: client qua API Gateway/BFF; Pricing tạo báo giá có hạn; Booking ghi đơn cùng outbox rồi phát `BookingCreated` qua Kafka; Dispatch gán tài xế **nguyên tử** để chỉ một người thắng; Trip giữ vòng đời; Payment thu qua cổng thanh toán có idempotency; Realtime Gateway đẩy trạng thái về app (luồng §8.1 và §8.3 của tài liệu kiến trúc). Lab là bản MVP «modular monolith» của lõi đó: cùng ranh giới module, một tiến trình, không Kafka, Redis hay realtime.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 350" role="img" aria-label="Luồng đặt xe theo kiến trúc Tới Nơi qua 13 bước giữa Customer App, API Gateway, Pricing, Booking, Kafka, Dispatch, Driver App, Realtime Gateway, Trip, Payment và VíXanh; khối có trong lab tô màu, khối chưa có viền đứt.">
<defs>
<marker id="tlab-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="8" width="22" height="14" rx="3" class="f-pk-soft s-pk"/><text x="50" y="19" class="t-small">có trong lab (module)</text>
<rect x="200" y="8" width="22" height="14" rx="3" class="f-bg s-muted dash"/><text x="230" y="19" class="t-small">chưa có trong lab (R-ARCH-02) — số trên mũi tên = bước của luồng</text>
<rect x="20" y="40" width="190" height="54" rx="8" class="f-pk-soft s-pk"/><text x="115" y="62" text-anchor="middle" class="t-b">Customer App</text><text x="115" y="80" text-anchor="middle" class="t-small">lab: trang web /</text>
<rect x="250" y="40" width="190" height="54" rx="8" class="f-bg s-muted dash"/><text x="345" y="62" text-anchor="middle" class="t-b">API Gateway / BFF</text><text x="345" y="80" text-anchor="middle" class="t-small">lab: Express, x-user giản lược</text>
<rect x="480" y="40" width="190" height="54" rx="8" class="f-pk-soft s-pk"/><text x="575" y="62" text-anchor="middle" class="t-b">Pricing &amp; Quote</text><text x="575" y="80" text-anchor="middle" class="t-small t-mono">quote.ts, fare.ts</text>
<rect x="710" y="40" width="190" height="54" rx="8" class="f-pk-soft s-pk"/><text x="805" y="62" text-anchor="middle" class="t-b">Booking / Order</text><text x="805" y="80" text-anchor="middle" class="t-small t-mono">createTrip, idempotency</text>
<rect x="20" y="160" width="190" height="54" rx="8" class="f-bg s-muted dash"/><text x="115" y="182" text-anchor="middle" class="t-b">Realtime Gateway</text><text x="115" y="200" text-anchor="middle" class="t-small">lab: hỏi lại mỗi 2 giây</text>
<rect x="250" y="160" width="190" height="54" rx="8" class="f-pk-soft s-pk"/><text x="345" y="182" text-anchor="middle" class="t-b">Driver App</text><text x="345" y="200" text-anchor="middle" class="t-small">lab: /driver.html</text>
<rect x="480" y="160" width="190" height="54" rx="8" class="f-pk-soft s-pk"/><text x="575" y="182" text-anchor="middle" class="t-b">Matching &amp; Dispatch</text><text x="575" y="200" text-anchor="middle" class="t-small">acceptTrip — kéo, không offer</text>
<rect x="710" y="160" width="190" height="54" rx="8" class="f-bg s-muted dash"/><text x="805" y="182" text-anchor="middle" class="t-b">Kafka + outbox</text><text x="805" y="200" text-anchor="middle" class="t-small">BookingCreated…</text>
<rect x="20" y="280" width="190" height="54" rx="8" class="f-bg s-muted dash"/><text x="115" y="302" text-anchor="middle" class="t-b">Location · Notification</text><text x="115" y="320" text-anchor="middle" class="t-small">GPS, SMS TinNhanh, push</text>
<rect x="250" y="280" width="190" height="54" rx="8" class="f-pk-soft s-pk"/><text x="345" y="302" text-anchor="middle" class="t-b">Trip Lifecycle</text><text x="345" y="320" text-anchor="middle" class="t-small t-mono">trip-state.ts, driverStep</text>
<rect x="480" y="280" width="190" height="54" rx="8" class="f-pk-soft s-pk"/><text x="575" y="302" text-anchor="middle" class="t-b">Payment &amp; Wallet</text><text x="575" y="320" text-anchor="middle" class="t-small">payment.ts (chưa ledger)</text>
<rect x="710" y="280" width="190" height="54" rx="8" class="f-pk-soft s-pk"/><text x="805" y="302" text-anchor="middle" class="t-b">VíXanh (cổng ví)</text><text x="805" y="320" text-anchor="middle" class="t-small">bản giả có chế độ lỗi</text>
<line x1="210" y1="67" x2="248" y2="67" class="s-muted" stroke-width="1.5" marker-end="url(#tlab-arrow)"/><text x="229" y="60" text-anchor="middle" class="t-b">1</text>
<line x1="440" y1="67" x2="478" y2="67" class="s-muted" stroke-width="1.5" marker-end="url(#tlab-arrow)"/><text x="459" y="60" text-anchor="middle" class="t-b">2</text>
<line x1="670" y1="67" x2="708" y2="67" class="s-muted" stroke-width="1.5" marker-end="url(#tlab-arrow)"/><text x="689" y="60" text-anchor="middle" class="t-b">3</text>
<line x1="805" y1="94" x2="805" y2="158" class="s-muted" stroke-width="1.5" marker-end="url(#tlab-arrow)"/><text x="815" y="131" class="t-b">4</text>
<line x1="710" y1="187" x2="672" y2="187" class="s-muted" stroke-width="1.5" marker-end="url(#tlab-arrow)"/><text x="691" y="180" text-anchor="middle" class="t-b">5</text>
<line x1="480" y1="180" x2="442" y2="180" class="s-muted" stroke-width="1.5" marker-end="url(#tlab-arrow)"/><text x="461" y="174" text-anchor="middle" class="t-b">6</text>
<line x1="442" y1="196" x2="478" y2="196" class="s-muted" stroke-width="1.5" marker-end="url(#tlab-arrow)"/><text x="461" y="210" text-anchor="middle" class="t-b">7</text>
<line x1="540" y1="214" x2="402" y2="278" class="s-muted" stroke-width="1.5" marker-end="url(#tlab-arrow)"/><text x="486" y="256" text-anchor="middle" class="t-b">8</text>
<line x1="345" y1="214" x2="345" y2="278" class="s-muted" stroke-width="1.5" marker-end="url(#tlab-arrow)"/><text x="335" y="252" text-anchor="end" class="t-b">9</text>
<line x1="440" y1="307" x2="478" y2="307" class="s-muted" stroke-width="1.5" marker-end="url(#tlab-arrow)"/><text x="459" y="300" text-anchor="middle" class="t-b">10</text>
<line x1="670" y1="307" x2="708" y2="307" class="s-muted" stroke-width="1.5" marker-end="url(#tlab-arrow)"/><text x="689" y="300" text-anchor="middle" class="t-b">11</text>
<line x1="262" y1="280" x2="160" y2="216" class="s-muted dash" stroke-width="1.5" marker-end="url(#tlab-arrow)"/><text x="196" y="262" text-anchor="middle" class="t-b">12</text>
<line x1="115" y1="160" x2="115" y2="96" class="s-muted dash" stroke-width="1.5" marker-end="url(#tlab-arrow)"/><text x="125" y="132" class="t-b">13</text>
</svg>
</div>
<figcaption><b>Hình 1 · Luồng đặt xe theo kiến trúc §8.1 và §8.3, đặt lên lab.</b> Khối tô màu là module có trong lab (R-ARCH-01); khối viền đứt chỉ có ở hệ thống đích. Bảng dưới diễn giải từng bước.</figcaption>
</figure>

| Bước | Hệ thống đích | Lab làm thế nào | SPEC |
|---|---|---|---|
| 1–2 | Xin báo giá qua Gateway; Pricing trả quote + hạn | `POST /api/quotes`, quãng đường do client gửi | R-GEN-06, R-QUOTE-01 |
| 3 | Tạo booking kèm idempotency key, kiểm quote còn hạn | `POST /api/trips` + `Idempotency-Key` | R-IDEM, R-QUOTE-03/04 |
| 4–5 | Ghi DB + outbox, phát `BookingCreated`; Dispatch tiêu thụ | Không có — chuyến vào `searching` ngay | R-ARCH-02 |
| 6–7 | Offer có TTL; tài xế accept, gán nguyên tử | Tài xế xem danh sách rồi `POST /accept` | R-ACCEPT-01…07 |
| 8–9 | `DriverAssigned` tạo Trip; tài xế cập nhật theo state machine | Cùng bản ghi chuyến; `/arrive`…`/complete` | R-STATE-02/03, R-API-10 |
| 10–11 | Trip yêu cầu thu tiền; PSP trả kết quả hoặc webhook | VíXanh giả, hết giờ 3 giây, thử lại cùng khoá | R-PAY-03…06 |
| 12–13 | Trạng thái lan về app khách qua realtime | Trang khách hỏi lại mỗi 2 giây | R-NFR-04 |

### 3.4. Đề bài

Bạn là tester vào sprint «MVP đặt xe». Code chưa xong; bạn có SPEC, tài liệu kiến trúc và bản lab chạy được. Trưởng nhóm cần trước buổi lập kế hoạch:

| Nhiệm vụ | Yêu cầu tối thiểu | Kiến thức |
|---|---|---|
| **T0 · Review test basis** | ≥ 4 câu hỏi/khiếm khuyết của SPEC về luồng đặt xe | Shift left (FL-2.1.5) |
| **T1 · Mục tiêu test** | ≥ 6 mục tiêu, mỗi cái ứng một mục tiêu điển hình CTFL và có cách biết «đã đạt» | FL-1.1.1 |
| **T2 · Đối tượng test** | Bảng đối tượng ↔ service đích ↔ module lab ↔ mức test gợi ý | Test object vs test basis |
| **T3 · Phạm vi** | Bảng trong/ngoài, mỗi dòng có lý do và «lab làm được không» | Nguyên lý 2 |
| **T4 · Rủi ro** | ≥ 10 product risk + ≥ 3 project risk, khả năng và tác động 1–3, mức = tích | FL-5.2.1, FL-5.2.2 |
| **T5 · Test condition** | ≥ 12 test condition, mỗi cái trỏ tới R-… và tới rủi ro | FL-1.4.1, FL-1.4.4 |
| **T6 · (tuỳ chọn)** | Viết test chạy được trên lab cho 3 test condition rủi ro cao nhất | Series 06 dạy kỹ |

Gợi ý rủi ro (tài liệu kiến trúc §14): **tạo trùng giao dịch** khi client gửi lại, **hai tài xế cùng nhận**, **webhook thanh toán trùng hoặc trễ**, **event sai thứ tự**, **lộ dữ liệu theo dõi chuyến**. Một số không kiểm được trên lab — chính đó là câu trả lời cho T3.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Tự chấm</span>
<p>Đạt khi: mục tiêu đo được (không phải «test thật kỹ»); đối tượng và cơ sở không bị lẫn; mỗi dòng «ngoài phạm vi» có lý do và nơi nhận; mỗi rủi ro là một <b>failure cụ thể</b> người dùng hoặc doanh nghiệp thấy được; mỗi rủi ro mức cao có ít nhất một test condition, mỗi test condition trỏ tới một mã R-….</p>
</div>

## 4. Phân tích kỹ thuật

Đây là **lời giải mẫu**, chia theo nhiệm vụ — mở từng khối sau khi tự làm. Không có đáp án duy nhất: rủi ro và ưu tiên phụ thuộc ngữ cảnh (CTFL 1.4.2); điều quan trọng là lý do và truy vết.

### 4.1. T0 — Review test basis

<details>
<summary><b>Lời giải mẫu T0 — 5 câu hỏi gửi BA</b></summary>
<table>
<thead><tr><th>Mã</th><th>SPEC</th><th>Vấn đề</th><th>Vì sao quan trọng</th></tr></thead>
<tbody>
<tr><td>Q-1</td><td>R-GEN-06</td><td>Tính năng bắt đầu từ «nhập điểm đón» nhưng lab không có điểm đón/điểm đến: client gửi thẳng <code>distanceKm</code>.</td><td>Không kiểm được nhập địa chỉ, geocoding, sai lệch quãng đường ⇒ ngoài phạm vi lab, là rủi ro còn lại.</td></tr>
<tr><td>Q-2</td><td>R-STATE-05</td><td>Hết 5 phút thì huỷ, nhưng máy chủ quét mỗi 15 giây ⇒ huỷ thực tế có thể trễ tới ~15 giây.</td><td>Test qua giao diện cần ghi dung sai trong expected result.</td></tr>
<tr><td>Q-3</td><td>R-STATE-04/05</td><td>Tài xế huỷ thì mốc tìm đặt lại; nhận rồi huỷ liên tục ⇒ khách có thể chờ vô hạn.</td><td>Câu hỏi validation: có cần giới hạn tổng thời gian chờ?</td></tr>
<tr><td>Q-4</td><td>R-PAY-06</td><td><code>paymentStatus = failed</code> «cần xử lý sau» — không nói ai, khi nào, khách có bị chặn đặt tiếp không.</td><td>Thiếu expected result; rủi ro mất doanh thu không ai sở hữu.</td></tr>
<tr><td>Q-5</td><td>R-ARCH-02, R-UI-01</td><td>Không có yêu cầu báo khách khi tài xế huỷ hay khi báo giá hết hạn trên giao diện.</td><td>Chỗ trải nghiệm dễ hỏng, ảnh hưởng mục tiêu validation.</td></tr>
</tbody>
</table>
</details>

### 4.2. T1 — Mục tiêu test

<details>
<summary><b>Lời giải mẫu T1 — 7 mục tiêu cho sprint MVP đặt xe</b></summary>
<table>
<thead><tr><th>Mã</th><th>Mục tiêu</th><th>Mục tiêu điển hình (1.1.1)</th><th>Biết «đã đạt» bằng cách nào</th></tr></thead>
<tbody>
<tr><td>O-1</td><td>Review SPEC phần đặt xe, giải quyết Q-1…Q-5 trước khi viết test case</td><td>Đánh giá work product</td><td>Mỗi câu hỏi có quyết định ghi lại</td></tr>
<tr><td>O-2</td><td>Gây failure ở ba điểm nguy hiểm: đặt trùng, nhận trùng, thu trùng</td><td>Gây failure, tìm defect</td><td>Có test cho cả ba, cả khi gửi lại/đồng thời</td></tr>
<tr><td>O-3</td><td>Phủ 9 dòng chuyển hợp lệ của R-STATE-02 và mọi mã lỗi của R-QUOTE-03</td><td>Đạt coverage cần thiết</td><td>Ma trận truy vết không ô trống</td></tr>
<tr><td>O-4</td><td>Không chuyến nào có hai tài xế, không khoản nào bị trừ hai lần</td><td>Giảm mức rủi ro</td><td>PR-01, PR-03 có test đạt; phần không kiểm được ghi là rủi ro còn lại</td></tr>
<tr><td>O-5</td><td>Mọi mã R-… trong phạm vi có ít nhất một test condition đạt</td><td>Kiểm yêu cầu (verification)</td><td>Tỉ lệ yêu cầu có test đạt = 100%</td></tr>
<tr><td>O-6</td><td>Một khách và một tài xế mới dùng lần đầu đi hết luồng trên <code>/</code> và <code>/driver.html</code></td><td>Xác nhận đúng mong đợi (validation)</td><td>Phiên dùng thử ghi chỗ vướng (<a href="/articles/verification-va-validation">TEST-005</a>)</td></tr>
<tr><td>O-7</td><td>Báo cáo rủi ro còn lại cho buổi quyết định ra mắt</td><td>Cung cấp thông tin để quyết định</td><td>Báo cáo liệt kê rủi ro chưa kiểm được và nơi sẽ kiểm</td></tr>
</tbody>
</table>
<p>Không chọn «tuân thủ pháp lý»: lab không có dữ liệu cá nhân hay thẻ thật. Hệ thống đích thì phải có — mục tiêu đổi theo ngữ cảnh.</p>
</details>

### 4.3. T2 — Đối tượng test

<details>
<summary><b>Lời giải mẫu T2 — 9 đối tượng test</b></summary>
<p>Cơ sở kiểm thử của mọi dòng là <code>SPEC.md</code>; đối tượng là phần mềm được kiểm.</p>
<table>
<thead><tr><th>Đối tượng</th><th>Service đích</th><th>Module lab</th><th>Mức test gợi ý</th></tr></thead>
<tbody>
<tr><td>Báo giá có hạn</td><td>Pricing &amp; Quote (§4.3)</td><td><code>app/quote.ts</code>, <code>domain/fare.ts</code></td><td>Component + API</td></tr>
<tr><td>Tạo chuyến</td><td>Booking / Order (§4.4)</td><td><code>createTrip</code>, <code>api/idempotency.ts</code></td><td>API</td></tr>
<tr><td>Nhận chuyến</td><td>Dispatch + Availability (§4.5–4.6)</td><td><code>acceptTrip</code>, <code>pg-trip-repository.ts</code></td><td>Component integration (DB) + API</td></tr>
<tr><td>Vòng đời chuyến</td><td>Trip Lifecycle (§4.7)</td><td><code>domain/trip-state.ts</code>, <code>driverStep</code></td><td>Component</td></tr>
<tr><td>Huỷ và phí huỷ</td><td>Booking + Trip</td><td><code>domain/cancel.ts</code>, <code>cancelByCustomer/Driver</code></td><td>Component + API</td></tr>
<tr><td>Hết thời gian tìm</td><td>Dispatch / Saga timeout (§9.2)</td><td><code>expireStaleSearches</code></td><td>Component</td></tr>
<tr><td>Thu tiền</td><td>Payment &amp; Wallet (§4.9) + PSP</td><td><code>app/payment.ts</code>, ví giả</td><td>Integration với bản giả</td></tr>
<tr><td>API HTTP</td><td>API Gateway / BFF</td><td><code>api/app.ts</code>, <code>rate-limit.ts</code></td><td>API</td></tr>
<tr><td>Giao diện</td><td>Customer App, Driver App</td><td><code>src/web/</code></td><td>System (thủ công, E2E)</td></tr>
</tbody>
</table>
<p>Không phải đối tượng test của bài vì không có trong lab: Kafka/outbox, Realtime Gateway, Location, Notification, Rating. Chúng sang T3 thành «ngoài phạm vi» và sang T4 thành rủi ro còn lại.</p>
</details>

### 4.4. T3 — Phạm vi

<details>
<summary><b>Lời giải mẫu T3 — trong / ngoài phạm vi</b></summary>
<table>
<thead><tr><th>Hạng mục</th><th>Phạm vi</th><th>Lý do</th><th>Lab làm được?</th></tr></thead>
<tbody>
<tr><td>Đặt xe <code>ride</code>, xe máy và ô tô</td><td>Trong</td><td>Chính là tính năng</td><td>Có</td></tr>
<tr><td>Báo giá có hạn, giá chụp, voucher ở mức có/không áp</td><td>Trong</td><td>Sai là mất tiền hoặc niềm tin</td><td>Có — đồng hồ, thời tiết giả</td></tr>
<tr><td>Tạo chuyến idempotent; một báo giá một chuyến</td><td>Trong</td><td>Rủi ro đặt trùng</td><td>Có</td></tr>
<tr><td>Nhận chuyến, một người thắng</td><td>Trong</td><td>Rủi ro nhận trùng</td><td>Một phần — không có giao dịch song song thật</td></tr>
<tr><td>Bước tài xế, chuyển sai bị từ chối; khách/tài xế/hệ thống huỷ</td><td>Trong</td><td>Xương sống và nhánh phụ có tiền</td><td>Có</td></tr>
<tr><td>Thu VíXanh, hết giờ và thử lại, thất bại không chặn</td><td>Trong</td><td>Rủi ro thu trùng, mất doanh thu</td><td>Có — ví giả có chế độ lỗi</td></tr>
<tr><td>Quyền xem chuyến, xác thực, giới hạn tần suất</td><td>Trong (ưu tiên thấp hơn)</td><td>Rủi ro lộ dữ liệu</td><td>Có — xác thực giản lược</td></tr>
<tr><td>Độ trễ Booking/Quote API</td><td>Trong, chỉ đo minh hoạ</td><td>Có SLO khởi tạo</td><td>Không kết luận đạt/không trên một máy</td></tr>
<tr><td>Nhập điểm đón/điểm đến, geocoding MapViet</td><td>Ngoài lab, trong sản phẩm</td><td>R-GEN-06</td><td>Không ⇒ SIT với bản đồ (Series 07)</td></tr>
<tr><td>Offer có TTL, mất kết nối sau khi nhận</td><td>Ngoài lab</td><td>R-ACCEPT-06: mô hình kéo</td><td>Không ⇒ Series 08/11</td></tr>
<tr><td>Kafka, outbox/inbox, event trùng/sai thứ tự; realtime, vị trí, SMS/push</td><td>Ngoài lab</td><td>R-ARCH-02</td><td>Không ⇒ Series 11</td></tr>
<tr><td>Webhook, ledger, hoàn tiền, đối soát</td><td>Ngoài lab</td><td>R-ARCH-02, R-STATE-08</td><td>Không ⇒ Series 07/11</td></tr>
<tr><td>App Android/iOS; tải ~70 chuyến/giây, nhiều bản máy chủ</td><td>Ngoài lab</td><td>Lab chỉ có web, một tiến trình</td><td>Không ⇒ Series 08/09</td></tr>
<tr><td>Giao bưu kiện, giao đồ ăn</td><td>Ngoài bài này</td><td>Không thuộc «đặt xe»</td><td>Có — để cho bài tập 1</td></tr>
</tbody>
</table>
<p>Có ba loại «ngoài»: ngoài bài này (để sau), ngoài lab nhưng trong sản phẩm (phải thành rủi ro còn lại có nơi nhận), và ngoài sản phẩm. Gộp chung một cột «out of scope» là cách giấu rủi ro.</p>
</details>

### 4.5. T4 — Rủi ro

<details>
<summary><b>Lời giải mẫu T4 — 15 product risk, 5 project risk</b></summary>
<p>Khả năng (K): 1 thấp · 2 vừa · 3 cao. Tác động (T): 1 nhẹ · 2 đáng kể · 3 nghiêm trọng (tiền, an toàn, pháp lý, niềm tin diện rộng). Mức = K × T.</p>
<table>
<thead><tr><th>Mã</th><th>Rủi ro sản phẩm (failure ta sợ)</th><th>Yêu cầu</th><th>K</th><th>T</th><th>Mức</th><th>Lab?</th></tr></thead>
<tbody>
<tr><td>PR-01</td><td>Hai tài xế cùng được gán, cả hai tới đón</td><td>R-ACCEPT-03</td><td>2</td><td>3</td><td>6</td><td>Một phần</td></tr>
<tr><td>PR-02</td><td>Bấm hai lần hoặc mạng gửi lại ⇒ hai chuyến</td><td>R-IDEM-02, R-QUOTE-03</td><td>3</td><td>2</td><td>6</td><td>Có</td></tr>
<tr><td>PR-03</td><td>Ví hết giờ, hệ thống thử lại ⇒ trừ tiền hai lần</td><td>R-PAY-04</td><td>2</td><td>3</td><td>6</td><td>Có</td></tr>
<tr><td>PR-04</td><td>Giá chuyến khác giá đã báo khi cao điểm/mưa đổi</td><td>R-QUOTE-04</td><td>2</td><td>2</td><td>4</td><td>Có</td></tr>
<tr><td>PR-05</td><td>Đặt được từ báo giá hết hạn</td><td>R-QUOTE-01</td><td>2</td><td>2</td><td>4</td><td>Có</td></tr>
<tr><td>PR-06</td><td>Chuyến kẹt ở <code>searching</code>, khách chờ mãi</td><td>R-STATE-04/05</td><td>2</td><td>2</td><td>4</td><td>Có</td></tr>
<tr><td>PR-07</td><td>Chuyến nhảy trạng thái sai (hoàn thành khi chưa đón)</td><td>R-STATE-03</td><td>1</td><td>3</td><td>3</td><td>Có</td></tr>
<tr><td>PR-08</td><td>Ví lỗi làm chuyến không hoàn thành, tài xế kẹt <code>busy</code></td><td>R-PAY-06, R-ACCEPT-05</td><td>2</td><td>2</td><td>4</td><td>Có</td></tr>
<tr><td>PR-09</td><td>Phí huỷ sai ở mốc 2 phút hoặc khi tài xế trễ</td><td>R-CANCEL-02/04</td><td>3</td><td>1</td><td>3</td><td>Có</td></tr>
<tr><td>PR-10</td><td>Khách xem được chuyến của khách khác (IDOR)</td><td>R-API-05</td><td>2</td><td>3</td><td>6</td><td>Có</td></tr>
<tr><td>PR-11</td><td>Một tài xế giữ hai chuyến đang chạy</td><td>R-ACCEPT-03/05</td><td>2</td><td>2</td><td>4</td><td>Có</td></tr>
<tr><td>PR-12</td><td>Đặt chuyến hoặc cập nhật trạng thái chậm lúc cao điểm</td><td>R-NFR-02/04</td><td>2</td><td>2</td><td>4</td><td>Minh hoạ</td></tr>
<tr><td>PR-13</td><td><code>DriverAssigned</code> xử lý hai lần/sai thứ tự ⇒ hai Trip, trạng thái lùi</td><td>Kiến trúc §6, §14</td><td>2</td><td>3</td><td>6</td><td>Không</td></tr>
<tr><td>PR-14</td><td>Webhook thanh toán trùng/trễ ⇒ ghi sổ hai lần</td><td>Kiến trúc §4.9, §8.3</td><td>2</td><td>3</td><td>6</td><td>Không</td></tr>
<tr><td>PR-15</td><td>Lộ vị trí tài xế, thông tin chuyến qua kênh theo dõi</td><td>Kiến trúc §4.14, §10</td><td>2</td><td>3</td><td>6</td><td>Không</td></tr>
</tbody>
</table>
<table>
<thead><tr><th>Mã</th><th>Rủi ro dự án</th><th>Loại (5.2.2)</th><th>K</th><th>T</th><th>Mức</th><th>Ứng phó</th></tr></thead>
<tbody>
<tr><td>PJ-01</td><td>Lab dùng PGlite một kết nối, không mô phỏng giao dịch đồng thời thật</td><td>Kỹ thuật / công cụ</td><td>3</td><td>2</td><td>6</td><td>Ghi PR-01 là rủi ro còn lại; kiểm trên PostgreSQL nhiều kết nối (Series 11)</td></tr>
<tr><td>PJ-02</td><td>Đặc tả thiếu điểm đón/điểm đến (Q-1)</td><td>Kỹ thuật</td><td>3</td><td>2</td><td>6</td><td>Hỏi BA; tách «nhập địa chỉ» sang SIT với MapViet</td></tr>
<tr><td>PJ-03</td><td>Sandbox VíXanh/MapViet chưa sẵn hoặc giới hạn quota</td><td>Nhà cung cấp</td><td>2</td><td>2</td><td>4</td><td>Dùng bản giả trước, đặt lịch SIT sớm</td></tr>
<tr><td>PJ-04</td><td>Thiếu người quen test đồng thời và thanh toán</td><td>Con người</td><td>2</td><td>2</td><td>4</td><td>Ghép cặp với developer (<a href="/articles/vai-tro-trong-kiem-thu">TEST-009</a>)</td></tr>
<tr><td>PJ-05</td><td>Ra mắt gấp, phạm vi phình giữa sprint</td><td>Tổ chức</td><td>2</td><td>2</td><td>4</td><td>Khoá phạm vi sprint</td></tr>
</tbody>
</table>
<p>Xếp vào ma trận 3 × 3: ô mức 6 có PR-01, PR-02, PR-03, PR-10, PR-13, PR-14, PR-15 cùng PJ-01, PJ-02; không rủi ro nào mức 9 — chấm 3/3 cho mọi thứ là dấu hiệu chưa phân tích. Rủi ro mức 6 đi trước. PR-13, PR-14, PR-15 <b>không giảm được bằng lab</b> ⇒ vào thẳng báo cáo rủi ro còn lại (O-7), không lặng lẽ bỏ qua.</p>
</details>

### 4.6. T5 — Test condition có truy vết

<details>
<summary><b>Lời giải mẫu T5 — 18 test condition</b></summary>
<table>
<thead><tr><th>Mã</th><th>Test condition</th><th>Yêu cầu</th><th>Rủi ro</th><th>Lab</th></tr></thead>
<tbody>
<tr><td>TCD-01</td><td>Luồng chính đủ 7 trạng thái, mỗi lần đổi ghi nhật ký, tài xế về <code>online</code>, thu ví một lần</td><td>R-STATE-02/06, R-ACCEPT-05, R-PAY-03</td><td>PR-07, PR-08</td><td>Đã có</td></tr>
<tr><td>TCD-02</td><td>Báo giá còn hạn ngay trước 5 phút, hết hạn ở đúng 5 phút</td><td>R-QUOTE-01/03</td><td>PR-05</td><td>Đã có</td></tr>
<tr><td>TCD-03</td><td>Giá chuyến bằng ảnh chụp báo giá dù cao điểm/mưa đã đổi</td><td>R-QUOTE-04, R-FARE-09</td><td>PR-04</td><td>Đã có</td></tr>
<tr><td>TCD-04</td><td>Gửi lại cùng khoá không tạo chuyến mới; một báo giá một chuyến</td><td>R-IDEM-02/03, R-QUOTE-03</td><td>PR-02</td><td>Đã có</td></tr>
<tr><td>TCD-05</td><td>Hai tài xế cùng nhận ⇒ đúng một thắng, người thua vẫn <code>online</code></td><td>R-ACCEPT-03</td><td>PR-01</td><td>Đã có (giới hạn PJ-01)</td></tr>
<tr><td>TCD-06</td><td>Tìm đủ 5 phút ⇒ hệ thống huỷ <code>no_driver</code>, phí 0 đ</td><td>R-STATE-05</td><td>PR-06</td><td>Đã có</td></tr>
<tr><td>TCD-07</td><td>Ví từ chối ⇒ chuyến vẫn hoàn thành, <code>paymentStatus = failed</code>, không thử lại</td><td>R-PAY-04/06</td><td>PR-08</td><td>Đã có</td></tr>
<tr><td>TCD-08</td><td>Phí huỷ ở mốc 2 phút: 1:59 miễn, 2:00 thu</td><td>R-CANCEL-01/02</td><td>PR-09</td><td>Đã có</td></tr>
<tr><td>TCD-09</td><td>Ví hết giờ, thử lại cùng khoá ⇒ trừ tiền một lần</td><td>R-PAY-04, R-NFR-05</td><td>PR-03</td><td>Test khói của lab</td></tr>
<tr><td>TCD-10</td><td>Khách khác xem chuyến ⇒ 404 như chuyến không tồn tại</td><td>R-API-05</td><td>PR-10</td><td>Test khói của lab</td></tr>
<tr><td>TCD-11</td><td>Bỏ bước (hoàn thành khi mới <code>accepted</code>) ⇒ 409, chuyến giữ nguyên</td><td>R-STATE-03, R-API-10</td><td>PR-07</td><td>Bài tập 2</td></tr>
<tr><td>TCD-12</td><td>Tài xế huỷ ⇒ về <code>searching</code>, mốc tìm đặt lại, khách không mất phí</td><td>R-STATE-04, R-CANCEL-05</td><td>PR-06</td><td>Chưa</td></tr>
<tr><td>TCD-13</td><td>Một tài xế nhận đồng thời hai chuyến ⇒ một thành công, một <code>DRIVER_BUSY</code></td><td>R-ACCEPT-03/05</td><td>PR-11</td><td>Chưa</td></tr>
<tr><td>TCD-14</td><td>Tài xế gửi lại lệnh nhận chuyến mình đang giữ ⇒ 200, không đổi gì</td><td>R-ACCEPT-07</td><td>PR-01</td><td>Test khói của lab</td></tr>
<tr><td>TCD-15</td><td>p95 tạo chuyến dưới 500 ms ở mức tải đã thoả thuận</td><td>R-NFR-02</td><td>PR-12</td><td>Minh hoạ (Series 09)</td></tr>
<tr><td>TCD-16</td><td>Event gán tài xế bị gửi lại không tạo Trip thứ hai</td><td>Kiến trúc §6.3</td><td>PR-13</td><td>Ngoài lab (Series 11)</td></tr>
<tr><td>TCD-17</td><td>Webhook trùng hoặc trễ không ghi sổ hai lần</td><td>Kiến trúc §4.9</td><td>PR-14</td><td>Ngoài lab (Series 07/11)</td></tr>
<tr><td>TCD-18</td><td>Kênh theo dõi chỉ trả chuyến của người được phép</td><td>Kiến trúc §10</td><td>PR-15</td><td>Ngoài lab (Series 09)</td></tr>
</tbody>
</table>
<p>Kiểm chéo hai chiều: mỗi PR-01…PR-15 có ít nhất một TCD; mỗi TCD trỏ tới ít nhất một yêu cầu. Rủi ro dự án không có TCD — chúng được xử lý bằng hành động quản lý, không bằng test.</p>
</details>

## 5. Thực hành

### 5.1. Kiểm chứng lời giải mẫu trên lab

Lời giải mẫu khẳng định nhiều điều về hệ thống: báo giá hết hạn ở đúng 5 phút, gửi lại cùng khoá không tạo chuyến mới, chỉ một tài xế thắng… Tester không tin khẳng định mà kiểm. Thư mục `examples/toi-noi-testing/articles/thuc-hanh-series-01-phan-tich-tinh-nang-dat-xe/` có 11 test vitest, mỗi test mang mã TCD và R-…; test gọi API HTTP của lab (supertest) với kho trong bộ nhớ, đồng hồ giả và ví giả; riêng TCD-05 chạy thêm trên PGlite. Ví dụ TCD-02 kiểm cả hai phía của mốc hết hạn:

```ts
it('TCD-02 · R-QUOTE-01: còn hạn ở 4 phút 59,999 giây; đúng 5 phút ⇒ 422 QUOTE_EXPIRED', async () => {
  const q1 = (await quote().expect(200)).body.quote;
  clock.advanceMs(5 * 60_000 - 1);
  await book(q1.quoteId, 'lab-key-0002').expect(201);

  const q2 = (await quote().expect(200)).body.quote;
  clock.advanceMinutes(5);
  const res = await book(q2.quoteId, 'lab-key-0003');
  expect(res.status).toBe(422);
  expect(res.body.error.code).toBe('QUOTE_EXPIRED');
});
```

Chạy (Node 22.18, vitest 3.2.7):

```bash
cd examples/toi-noi-testing
npx vitest run articles/thuc-hanh-series-01-phan-tich-tinh-nang-dat-xe --reporter=verbose
```

```text
 ✓ TCD-01 · R-STATE-02/06, R-PAY-03: đi hết luồng chính, nhật ký đủ 7 trạng thái, thu VíXanh một lần
 ✓ TCD-02 · R-QUOTE-01: còn hạn ở 4 phút 59,999 giây; đúng 5 phút ⇒ 422 QUOTE_EXPIRED
 ✓ TCD-03 · R-QUOTE-04: giá của chuyến là ảnh chụp lúc báo giá, cao điểm và mưa đã hết vẫn không đổi
 ✓ TCD-04 · R-IDEM-02 + R-QUOTE-03: gửi lại cùng khoá trả đúng chuyến cũ; khoá mới cho cùng báo giá ⇒ 409
 ✓ TCD-05 · R-ACCEPT-03: D101 và D102 cùng bấm nhận ⇒ đúng một người thắng, người thua vẫn online
 ✓ TCD-06 · R-STATE-05: 4 phút 59 giây chưa huỷ; đủ 5 phút ⇒ hệ thống huỷ no_driver, phí 0 đ
 ✓ TCD-07 · R-PAY-06: VíXanh từ chối ⇒ chuyến vẫn completed, paymentStatus = failed, API 200
 ✓ TCD-08 · R-CANCEL-01/02: khách huỷ lúc 1 phút 59 giây ⇒ 0 đ; đúng 2 phút ⇒ 10.000 đ
 ✓ Bài tập 2 · TCD-11 · R-STATE-03, R-API-10: hoàn thành khi mới accepted ⇒ 409, chuyến giữ nguyên; tài xế lạ ⇒ 404
 ✓ Ngoài phạm vi lab · R-GEN-06/R-API-14: gửi «điểm đón» bị bỏ qua — lab chỉ nhận quãng đường
 ✓ TCD-05 trên PGlite (PostgreSQL WASM, một kết nối) > R-ACCEPT-03: hai lệnh nhận xen kẽ ở mức await ⇒ đúng một thành công

 Test Files  1 passed (1)
      Tests  11 passed (11)
```

(Đã lược tên file và tên khối `describe` ở đầu mỗi dòng.) TCD-09, TCD-10, TCD-14 không viết lại vì test khói của lab đã có (`tests/smoke/api.test.ts`; `npx vitest run tests/smoke/api.test.ts` ⇒ `11 passed (11)`). Tái dùng testware có sẵn cũng là truy vết.

### 5.2. Kết quả mong đợi và thực tế

| TCD | Dữ liệu | Kết quả mong đợi (theo SPEC) | Thực tế |
|---|---|---|---|
| TCD-01 | Xe máy SG, 7,4 km, 09:00, VíXanh | 12.000 + 4.500 × 7,4 = 45.300 ⇒ làm tròn lên **46.000 đ**; nhật ký 7 dòng `requested` … `completed`; khoá thu `trip:<id>:fare`, trừ 46.000 đ một lần | Đúng |
| TCD-02 | Báo giá 09:00 | Đặt lúc 09:04:59,999 ⇒ 201; báo giá khác, đặt sau đúng 5 phút ⇒ 422 `QUOTE_EXPIRED` | Đúng |
| TCD-03 | Báo giá 12:58, SG mưa, `GIAM20`; đặt 13:01 khi hết mưa | Chuyến giữ **52.000 đ**, `peak`, `raining` = true (oracle R-FARE-09); báo giá mới lúc 13:01: 45.300 − 9.060 = 36.240 ⇒ **37.000 đ** | Đúng |
| TCD-04 | Cùng khoá hai lần, rồi khoá mới | Lần hai 201, `Idempotent-Replayed: true`, cùng id; khoá mới ⇒ 409 `QUOTE_ALREADY_USED` | Đúng |
| TCD-05 | D101, D102 cùng nhận | Một 200, một 409 `TRIP_ALREADY_TAKEN`; thua `online`, thắng `busy` | Đúng (bộ nhớ và PGlite) |
| TCD-06 | +4:59 rồi +5:00 | Lần quét đầu huỷ 0, lần sau 1; `no_driver`, phí 0, `paymentStatus = none` | Đúng |
| TCD-07 | Ví chế độ `decline` | `completed`, `failed`, một lần gọi ví, không trừ tiền | Đúng |
| TCD-08 | Huỷ lúc 1:59 và 2:00 sau khi nhận | 0 đ `grace_period`; 10.000 đ `charged` | Đúng |
| Ngoài phạm vi | Gửi thêm trường `pickup` | Bị bỏ qua, báo giá không có `pickup` | Đúng — xác nhận Q-1 |

11 test xanh **không** chứng minh luồng đặt xe hết lỗi ([TEST-003](/articles/bay-nguyen-ly-kiem-thu), nguyên lý 1). Chúng chỉ xác nhận các giả định của lời giải mẫu đúng với lab hôm nay, và cho một bộ regression nhỏ cho rủi ro mức cao. Muốn thêm dữ liệu cho mục tiêu O-6, chạy `npm start`, mở `/` (khách `C001`) và `/driver.html` (tài xế `D101`), đi hết luồng rồi ghi lại chỗ khó hiểu — việc test tự động không trả lời được.

## 6. Góc nhìn nâng cao

**Dễ bỏ sót.** *Hai phía của một mốc*: «hết hạn sau 5 phút» là hai câu hỏi (4:59,999 và 5:00); TCD-02, TCD-06, TCD-08 đều kiểm cả hai (kỹ thuật biên ở Series 04). *Nhánh quay lại*: tài xế huỷ đưa chuyến ngược về `searching` — vòng đời có chu trình, và chu trình sinh câu hỏi Q-3. *Thanh toán thất bại không phải chuyến thất bại*: R-PAY-06 cố ý tách hai việc; test «ví lỗi ⇒ chuyến lỗi» mang expected result sai.

**Giới hạn của lab.** TCD-05 xanh nghĩa là logic «cập nhật nếu còn `searching`» đúng khi hai lệnh xen kẽ trong một tiến trình, **chưa** nghĩa là đúng khi hai giao dịch chạy song song trên hai máy chủ (PJ-01). Ở hệ thống đích còn tầng thứ hai: `DriverAssigned` có thể đến hai lần vì Kafka giao ít nhất một lần (kiến trúc §6.3) — PR-13 không giảm chút nào nhờ lab.

| Hay nhầm | Khác nhau ở đâu |
|---|---|
| Test condition vs test case | TCD-02 là «báo giá hết hạn ở đúng mốc 5 phút»; test case là «báo giá 09:00, đặt 09:05:00, mong đợi 422 `QUOTE_EXPIRED`» |
| Test object vs test basis | `createTrip` là đối tượng; R-QUOTE-03 là cơ sở. Cơ sở có thể sai — test đúng đặc tả vẫn có thể ra sản phẩm sai (Q-3) |
| Product risk vs project risk | PR-01 làm khách thấy hai tài xế; PJ-01 làm *đội* không kiểm được PR-01 |
| Ngoài phạm vi vs không quan trọng | PR-14 ngoài phạm vi lab nhưng mức 6; «ngoài phạm vi» phải có nơi nhận |

**Sai lầm thường gặp:** mục tiêu kiểu «đảm bảo không còn lỗi» (không đạt được, không đo được); rủi ro kiểu «lỗi thanh toán» (quá mơ hồ để ước lượng hay viết test — viết thành failure); chấm mọi rủi ro 3/3 (ma trận mất tác dụng); bỏ qua project risk; truy vết chỉ một chiều yêu cầu → test nên không trả lời được «còn rủi ro gì chưa kiểm?».

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — Giao đồ ăn.** Áp cùng cách làm cho `food` (SPEC mục 4): viết 3 rủi ro sản phẩm mới so với đặt xe, mỗi rủi ro một test condition có mã R-….

<details>
<summary><b>Lời giải gợi ý</b></summary>
<table>
<thead><tr><th>Rủi ro mới</th><th>Test condition</th><th>Yêu cầu</th></tr></thead>
<tbody>
<tr><td>Voucher trừ cả vào phí giao</td><td>Đơn có voucher: giảm chỉ trên tiền món, phí giao giữ nguyên</td><td>R-FOOD-04/05</td></tr>
<tr><td>Đơn dưới mức tối thiểu vẫn đặt được</td><td>29.000 đ ⇒ <code>SUBTOTAL_TOO_LOW</code>; 30.000 đ được nhận</td><td>R-FOOD-03</td></tr>
<tr><td>Đơn đồ ăn gán ô tô ⇒ sai cước, sai phí huỷ</td><td><code>vehicle: car</code> ⇒ <code>VALIDATION_ERROR</code>; huỷ có phí ⇒ 10.000 đ</td><td>R-FOOD-06, R-CANCEL-03</td></tr>
</tbody>
</table>
<p>Vòng đời, nhận chuyến, thanh toán dùng chung ⇒ TCD-01…TCD-14 tái dùng được; chỉ phần giá là mới.</p>
</details>

**Bài tập 2 — Test cho TCD-11.** Tài xế D101 nhận chuyến rồi gửi ngay `/complete`: kết quả mong đợi theo R-STATE-03, R-API-10? Tài xế D102 (không được gán) gửi `/arrive` thì sao?

<details>
<summary><b>Lời giải (đã chạy — test «Bài tập 2 · TCD-11»)</b></summary>
<pre><code class="language-ts">await driver('D101', trip.id, 'accept', { etaMinutes: 5 }).expect(200);
const skip = await driver('D101', trip.id, 'complete');
expect(skip.status).toBe(409);
expect(skip.body.error.code).toBe('INVALID_TRANSITION');
expect((await system.deps.trips.findById(trip.id))?.status).toBe('accepted');
expect(await system.deps.trips.listEvents(trip.id)).toHaveLength(3); // requested, searching, accepted
const stranger = await driver('D102', trip.id, 'arrive');
expect(stranger.status).toBe(404);
expect(stranger.body.error.code).toBe('TRIP_NOT_FOUND');
</code></pre>
<p>Hai điểm hay quên: kiểm cả «chuyến giữ nguyên» (nhật ký không thêm dòng), không chỉ mã lỗi; và tài xế lạ nhận 404 chứ không phải 403, để không lộ chuyến có tồn tại.</p>
</details>

**Bài tập 3 — Ngữ cảnh đổi.** Tới Nơi thí điểm ở Nha Trang (`NT`) **chỉ nhận tiền mặt**. Rủi ro nào trong T4 đổi mức, rủi ro nào xuất hiện?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>PR-03, PR-08, PR-14 gần như biến mất (không gọi ví) ⇒ rời phạm vi thí điểm nhưng vẫn ghi lại cho lúc bật ví. PR-09 giảm tác động, nhưng xuất hiện rủi ro mới: phí huỷ tiền mặt chỉ được ghi nhận, «thu ở chuyến sau» (R-PAY-05) — có thể không bao giờ thu được, và SPEC để nó ngoài phạm vi. PR-01, PR-02, PR-06 giữ nguyên. Cùng tính năng, khác ngữ cảnh thì khác ưu tiên (CTFL 1.1.1, 1.4.2).</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K1).** Mục nào là **mục tiêu kiểm thử điển hình** theo CTFL v4.0.1?

a) Chứng minh luồng đặt xe không còn defect  
b) Giảm mức rủi ro chất lượng phần mềm không đạt  
c) Tìm dòng code gây tính sai phí huỷ và sửa nó  
d) Viết đặc tả cho tính năng đặt xe

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b</b> — có trong danh sách mục 1.1.1. a sai: testing không chứng minh được không còn defect. c là debugging (<a href="/articles/software-testing-la-gi-testing-khac-debugging">TEST-001</a>). d là việc viết yêu cầu; tester review đặc tả nhưng «viết đặc tả» không phải mục tiêu test.</p>
</details>

**Câu 2 (K2).** Cặp nào ghép **đúng** test object và test basis?

a) Object: `SPEC.md` mục R-QUOTE; basis: hàm `createTrip`  
b) Object: hàm `createTrip` và lớp idempotency; basis: R-QUOTE-03 và R-IDEM-02  
c) Object: bảng rủi ro PR-01…PR-15; basis: TCD-01…TCD-18  
d) Object: kết quả chạy vitest; basis: README của lab

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b.</b> Đối tượng là phần mềm được kiểm, cơ sở là tài liệu nói nó phải làm gì. a đảo ngược. c ghép hai loại testware với nhau. d: kết quả chạy là test log; README không đặc tả hành vi.</p>
</details>

**Câu 3 (K1).** Mức rủi ro = khả năng × tác động (thang 1–3). Rủi ro nào có **mức cao nhất**?

a) Phí huỷ sai ở mốc 2 phút: khả năng 3, tác động 1  
b) Chuyến nhảy từ `accepted` sang `completed`: khả năng 1, tác động 3  
c) Trừ tiền hai lần khi ví hết giờ: khả năng 2, tác động 3  
d) Chuyến kẹt ở `searching`: khả năng 2, tác động 2

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c</b> — mức 6. a và b cùng mức 3 dù rất khác nhau (hay xảy ra nhưng nhẹ, hiếm nhưng nặng), nên khi trình bày vẫn giữ cả hai yếu tố. d mức 4.</p>
</details>

**Câu 4 (K2).** Mục nào là **project risk**?

a) Hai tài xế cùng được gán một chuyến  
b) Khách xem được chuyến của khách khác  
c) Môi trường test chỉ có PGlite một kết nối nên đội không mô phỏng được giao dịch đồng thời thật  
d) Giá chuyến khác giá đã báo khi trời đổi mưa

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c</b> — vấn đề kỹ thuật/công cụ của dự án (CTFL 5.2.2), ảnh hưởng khả năng của đội chứ không phải hành vi sản phẩm. a, b, d là failure người dùng thấy được, tức product risk. c làm rủi ro a khó giảm.</p>
</details>

**Câu 5 (K2).** BA đổi R-QUOTE-01: báo giá hết hạn sau **3** phút. Nhờ truy vết R-… → TCD → test, đội làm được gì **ngay**?

a) Chứng minh hệ thống không còn lỗi liên quan báo giá  
b) Tìm nhanh test condition và test case bị ảnh hưởng (TCD-02, TCD-03) để cập nhật và chạy lại  
c) Bỏ regression vì chỉ đổi một con số  
d) Tự động sửa code của `createQuote`

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b.</b> Truy vết giúp đánh giá tác động thay đổi và coverage (CTFL 1.4.4). a không thể. c ngược thực hành — đổi yêu cầu là lúc cần confirmation và regression. d: truy vết là thông tin, không sửa code.</p>
</details>

**Câu 6 (K2).** Sản phẩm nào là **đầu ra của test analysis**?

a) Nhật ký chạy vitest với 11 test đạt  
b) Test condition đã ưu tiên và câu hỏi/defect về SPEC như Q-1…Q-5  
c) Test suite đã sắp lịch chạy trong CI  
d) Báo cáo hoàn thành kiểm thử sau khi ra mắt

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b</b> (CTFL 1.4.3). a là test log — đầu ra test execution. c thuộc test implementation. d thuộc test completion. Bảng hoạt động → testware ở <a href="/articles/testware-san-pham-kiem-thu">TEST-010</a>.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — mục 1.1.1 (FL-1.1.1), 1.3, 1.4.1–1.4.4 (FL-1.4.1, FL-1.4.3, FL-1.4.4), 2.1.5 (FL-2.1.5), 5.1.1, 5.2.1–5.2.3 (FL-5.2.1, FL-5.2.2, FL-5.2.3). https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB — trang chủ, glossary và các syllabus: https://www.istqb.org/
- ISO/IEC/IEEE 29119-2 (quy trình kiểm thử), 29119-3 (tài liệu kiểm thử, gồm test plan); ISO/IEC 25010 (đặc tính chất lượng — để gọi tên product risk phi chức năng).
- Phòng thí nghiệm Tới Nơi (`SPEC.md`, `README.md`, thư mục bài): https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · Series 01: [TEST-001](/articles/software-testing-la-gi-testing-khac-debugging) · [TEST-002](/articles/vi-sao-can-kiem-thu) · [TEST-003](/articles/bay-nguyen-ly-kiem-thu) · [TEST-004](/articles/error-defect-failure-root-cause) · [TEST-005](/articles/verification-va-validation) · [TEST-006](/articles/muc-tieu-kiem-thu-va-quality-assurance) · [TEST-007](/articles/quy-trinh-kiem-thu-test-process) · [TEST-008](/articles/kiem-thu-trong-sdlc-agile-waterfall) · [TEST-009](/articles/vai-tro-trong-kiem-thu) · [TEST-010](/articles/testware-san-pham-kiem-thu)

Bài tiếp theo: Series 02 «Test Levels & Test Types» sắp ra mắt — trong lúc chờ, đọc lại [TEST-001: Software Testing là gì?](/articles/software-testing-la-gi-testing-khac-debugging) với con mắt người vừa tự tay phân tích một tính năng.
