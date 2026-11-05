---
slug: system-testing
title: "System Testing: kiểm toàn bộ Tới Nơi như một hệ thống"
summary: "System testing theo CTFL v4.0.1: kiểm hành vi đầu-cuối và đặc tính chất lượng của cả hệ thống so với đặc tả, trên môi trường gần production. Sáu test condition cho luồng đặt ô tô của lab, tự động hoá bằng Playwright."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-014, test-level, system-testing, playwright, test-environment]
language: vi
---

Mọi component đã đúng, mọi chỗ nối đã kiểm — vậy khách mở ứng dụng, đặt một chiếc ô tô và tài xế chở tới nơi thì mọi thứ có chạy không? Bài này trả lời: **system testing kiểm cái gì, dựa trên test basis nào, cần môi trường ra sao**, và viết test condition cấp hệ thống khác viết test cho một hàm thế nào. Ví dụ là luồng khách đặt ô tô trên trang `/` và tài xế hoàn thành chuyến trên `/driver.html` của lab Tới Nơi.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-014</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S02 · Test Levels &amp; Test Types</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>System</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Non-functional</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/tong-quan-5-test-level-istqb">TEST-011 Tổng quan 5 test level</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): viết 6 test condition cấp hệ thống cho luồng khách đặt ô tô qua <code>/</code> và tài xế hoàn thành qua <code>/driver.html</code> (R-UI-01, R-UI-02, R-STATE-02), tự động hoá bằng Playwright — <code>examples/toi-noi-testing/articles/system-testing/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 2 · mục 2.2 (2.2.1, 2.2.2) · FL-2.2.1 (K2); liên quan 1.5.3 (độc lập kiểm thử)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

[TEST-012](/articles/component-testing-unit-testing-tong-quan) kiểm từng hàm cô lập; [TEST-013](/articles/component-integration-testing) kiểm chỗ các hàm ghép với nhau. Cả hai đều nhìn hệ thống **từ bên trong**, qua mã nguồn. Có những câu hỏi chúng không trả lời được:

- Trang khách có tự thấy «Tài xế đã nhận» mà không phải tải lại không?
- Tài xế xe máy có nhìn thấy nhầm chuyến ô tô không?
- Thông báo lỗi khi tài xế bấm sai bước có dễ hiểu không?
- Trạng thái mới tới tay khách mất bao lâu?

Đó là câu hỏi về **hành vi của cả hệ thống** như người dùng thấy. **System testing** là test level trả lời chúng. Dùng khi đã có một bản hệ thống chạy được đầy đủ (hoặc gần đầy đủ) và một môi trường đủ giống thật.

Bài dừng ở mức **test level**. Chiến lược system test cho Tới Nơi (phạm vi, tiêu chí vào/ra, dữ liệu) đi sâu ở TEST-079; end-to-end testing và các kịch bản nhiều người dùng, đồng thời ở Series 08.

## 2. Lý thuyết theo ISTQB

### 2.1. Định nghĩa

Theo CTFL v4.0.1 (mục 2.2.1), system testing **tập trung vào hành vi tổng thể và năng lực của toàn bộ hệ thống hoặc sản phẩm**. Syllabus nêu thêm năm ý:

1. Thường gồm **functional testing các tác vụ đầu-cuối** và **non-functional testing các đặc tính chất lượng**.
2. Với một số đặc tính phi chức năng, nên kiểm trên **hệ thống hoàn chỉnh trong môi trường đại diện** — syllabus lấy ví dụ usability.
3. Có thể dùng **mô phỏng các sub-system** (hệ thống con).
4. Có thể do **đội test độc lập** thực hiện.
5. Gắn với **đặc tả của hệ thống** (specifications for the system).

### 2.2. Năm thuộc tính phân biệt test level, áp vào system testing

Syllabus phân biệt test level bằng các thuộc tính test object, test objective, test basis, defect/failure, approach và responsibility. Bảng dưới là cách bài này áp chúng vào Tới Nơi.

| Thuộc tính | System testing ở Tới Nơi |
|---|---|
| Test object | Toàn bộ hệ thống Tới Nơi đã lắp ráp: ứng dụng khách, ứng dụng tài xế, backend; bên thứ ba có thể là bản giả hoặc sandbox |
| Test objective | Hệ thống thực hiện đúng các tác vụ nghiệp vụ đầu-cuối (đặt chuyến, nhận, chở, huỷ, thu tiền) và đạt các đặc tính chất lượng (thời gian phản hồi, độ trễ cập nhật trạng thái, bảo mật, khả năng tiếp cận) |
| Test basis | Đặc tả hệ thống: yêu cầu chức năng và phi chức năng (SPEC R-UI, R-STATE, R-NFR), use case/user story, mô hình trạng thái, tài liệu hướng dẫn sử dụng |
| Defect / failure điển hình | Luồng nghiệp vụ đứt giữa chừng; hai phía (khách, tài xế) thấy trạng thái khác nhau; hệ thống làm khác đặc tả hoặc khác hướng dẫn; đặc tính phi chức năng không đạt; lỗi chỉ lộ ra khi đủ mọi thành phần chạy cùng nhau |
| Approach, responsibility | Thường hộp đen, theo kịch bản người dùng; trên môi trường gần production; thường do tester hoặc đội test độc lập (FL-1.5.3 nêu ví dụ đội test làm system và system integration testing) |

### 2.3. System testing và bốn test type

Mục 2.2.2 của syllabus nói cả bốn test type áp dụng được ở mọi test level, chỉ khác trọng tâm. Ở cấp hệ thống:

| Test type | Ở system testing của Tới Nơi |
|---|---|
| Functional | Khách đặt ô tô và chuyến đi hết 6 trạng thái; tài xế không nhận được chuyến khác loại xe |
| Non-functional | Trạng thái tới trang khách dưới 2–3 giây (R-NFR-04); p95 tạo chuyến dưới 500 ms (R-NFR-02); thông báo lỗi dễ hiểu (usability) |
| Black-box | Thiết kế test từ SPEC, không nhìn mã — cách tiếp cận chính ở cấp này |
| White-box | «Cấu trúc» ở cấp hệ thống không phải dòng lệnh mà là **luồng công việc** (workflow): phủ mọi chuyển trạng thái của R-STATE-02 |

### 2.4. Bảng thuật ngữ

| Thuật ngữ | Nghĩa tiếng Việt | Ghi chú |
|---|---|---|
| system testing | kiểm thử hệ thống | test level thứ ba trong năm test level |
| system under test (SUT) | hệ thống đang được kiểm | |
| test environment | môi trường kiểm thử | phần cứng, phần mềm, dữ liệu, công cụ |
| representative test environment | môi trường kiểm thử đại diện | đủ giống môi trường vận hành |
| end-to-end task | tác vụ đầu-cuối | một việc người dùng làm trọn vẹn |
| simulation / simulator | mô phỏng | thay sub-system chưa có hoặc không dùng được |
| independent test team | đội test độc lập | |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p><b>End-to-end (E2E) testing</b> là thuật ngữ ngành, không phải một test level của CTFL v4.0.1 (syllabus chỉ dùng cụm «end-to-end tasks» và «end-to-end tests» khi nói về test pyramid). Tài liệu kiến trúc Tới Nơi §17.1 tách hai hàng: «System — customer đặt chuyến từ đầu đến cuối» và «E2E — Customer App ↔ Driver App ↔ Backend ↔ PSP sandbox». Bài này theo nghĩa ISTQB: system testing kiểm cả hệ thống so với đặc tả; E2E là một cách thực hiện, có thể vượt sang system integration khi đi qua hệ thống ngoài thật.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu và business rules

Luồng: **khách C001 đặt ô tô ở TP.HCM, trả tiền mặt; tài xế D103 nhận và chở tới nơi**. Trích SPEC lab:

| Mã | Quy tắc (tóm tắt) |
|---|---|
| R-UI-01 | Trang khách: báo giá hiện tổng tiền, cờ cao điểm/mưa; «Đặt chuyến» tạo chuyến, bấm lại không tạo trùng; trạng thái tự làm mới mỗi 2 giây |
| R-UI-02 | Trang tài xế: nhập mã tài xế, tải chuyến đang chờ, nhận với ETA, bấm lần lượt các bước tới hoàn thành |
| R-FARE-01 | Ô tô: mở cửa 25.000 đ, 11.000 đ/km, tối thiểu 30.000 đ |
| R-ACCEPT-01, R-ACCEPT-06 | Tài xế chỉ thấy và nhận chuyến `searching` cùng thành phố, đúng loại xe |
| R-STATE-02, R-STATE-03 | `searching` → `accepted` → `arriving` → `picked_up` → `in_transit` → `completed`; bước sai thứ tự ⇒ `INVALID_TRANSITION`, chuyến giữ nguyên |
| R-NFR-04 | Thay đổi trạng thái tới ứng dụng khách trong p95 dưới 2–3 giây (lab: trang khách hỏi lại mỗi 2 giây) |

### 3.2. System context

- **Hệ thống đích** (kiến trúc §8.1): ứng dụng khách → API Gateway → Pricing (báo giá) → Booking (tạo chuyến, ghi outbox) → Kafka `BookingCreated` → Dispatch gửi offer realtime → tài xế nhận → Trip phát sự kiện → Realtime Gateway đẩy trạng thái về khách. Một lần «đặt ô tô» đi qua ít nhất 6 service, Kafka, Redis và gateway.
- **Lab** (SPEC R-ARCH): một tiến trình gồm web, API, use case, domain và kho trong bộ nhớ; không Kafka, không realtime gateway (khách hỏi lại mỗi 2 giây), không offer (tài xế tự kéo danh sách). Ví VíXanh, thời tiết, đồng hồ là bản giả — đúng ý «dùng mô phỏng sub-system» của syllabus.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 262" role="img" aria-label="Test object của system testing trong lab: hai trình duyệt (khách ở trang /, tài xế ở /driver.html) thao tác với hệ thống đang kiểm là máy chủ lab một tiến trình gồm web, API, use case, domain và kho trong bộ nhớ. Bên phải là các sub-system được mô phỏng: VíXanh giả, thời tiết giả, đồng hồ dời được. Trang khách hỏi lại trạng thái mỗi 2 giây.">
<defs>
<marker id="t014-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<rect x="16" y="40" width="170" height="56" rx="8" class="f-phys-soft s-phys"/><text x="101" y="64" text-anchor="middle" class="t-b">Trình duyệt khách</text><text x="101" y="83" text-anchor="middle" class="t-small">trang / · C001</text>
<rect x="16" y="150" width="170" height="56" rx="8" class="f-phys-soft s-phys"/><text x="101" y="174" text-anchor="middle" class="t-b">Trình duyệt tài xế</text><text x="101" y="193" text-anchor="middle" class="t-small">/driver.html · D103</text>
<text x="101" y="124" text-anchor="middle" class="t-small t-phys">hỏi lại mỗi 2 giây</text>
<rect x="230" y="20" width="470" height="210" rx="10" class="f-bg s-pk dash"/>
<text x="246" y="42" class="t-b">Hệ thống đang kiểm — máy chủ lab (một tiến trình)</text>
<rect x="250" y="60" width="130" height="44" rx="8" class="f-pk-soft s-pk"/><text x="315" y="87" text-anchor="middle">web (HTML, JS)</text>
<rect x="395" y="60" width="130" height="44" rx="8" class="f-pk-soft s-pk"/><text x="460" y="87" text-anchor="middle">API express</text>
<rect x="540" y="60" width="140" height="44" rx="8" class="f-pk-soft s-pk"/><text x="610" y="87" text-anchor="middle">use case app/</text>
<rect x="250" y="140" width="200" height="44" rx="8" class="f-pk-soft s-pk"/><text x="350" y="167" text-anchor="middle">domain: cước, trạng thái</text>
<rect x="470" y="140" width="210" height="44" rx="8" class="f-pk-soft s-pk"/><text x="575" y="167" text-anchor="middle">kho trong bộ nhớ</text>
<text x="465" y="214" text-anchor="middle" class="t-small">test chỉ thao tác qua giao diện, không gọi thẳng hàm bên trong</text>
<line x1="186" y1="68" x2="246" y2="78" class="s-phys" stroke-width="1.6" marker-end="url(#t014-arrow)"/>
<line x1="186" y1="178" x2="246" y2="92" class="s-phys" stroke-width="1.6" marker-end="url(#t014-arrow)"/>
<line x1="380" y1="82" x2="391" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#t014-arrow)"/>
<line x1="525" y1="82" x2="536" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#t014-arrow)"/>
<line x1="575" y1="104" x2="400" y2="136" class="s-pk" stroke-width="1.6" marker-end="url(#t014-arrow)"/>
<line x1="620" y1="104" x2="620" y2="136" class="s-pk" stroke-width="1.6" marker-end="url(#t014-arrow)"/>
<text x="822" y="28" text-anchor="middle" class="t-small t-warn">sub-system được mô phỏng</text>
<rect x="740" y="40" width="164" height="40" rx="8" class="f-warn-soft s-warn"/><text x="822" y="65" text-anchor="middle">VíXanh giả</text>
<rect x="740" y="100" width="164" height="40" rx="8" class="f-warn-soft s-warn"/><text x="822" y="125" text-anchor="middle">thời tiết giả</text>
<rect x="740" y="160" width="164" height="40" rx="8" class="f-warn-soft s-warn"/><text x="822" y="185" text-anchor="middle">đồng hồ dời được</text>
<line x1="700" y1="60" x2="736" y2="60" class="s-warn dash" stroke-width="1.4"/>
<line x1="700" y1="120" x2="736" y2="120" class="s-warn dash" stroke-width="1.4"/>
<line x1="700" y1="180" x2="736" y2="180" class="s-warn dash" stroke-width="1.4"/>
<text x="460" y="252" text-anchor="middle" class="t-small">khởi động: NODE_ENV=test (bật /api/test/reset), 09:00 giờ VN, không mưa, dữ liệu mẫu Phụ lục A</text>
</svg>
</div>
<figcaption><b>Hình 1 · Test object của system testing trong lab.</b> Hai người dùng là hai phiên trình duyệt riêng; hệ thống được kiểm nguyên khối qua giao diện. Ví, thời tiết, đồng hồ là sub-system mô phỏng. Ở Tới Nơi thật, khung giữa là hàng chục service, Kafka, Redis và realtime gateway trên môi trường staging.</figcaption>
</figure>

### 3.3. Yêu cầu môi trường

Môi trường là phần hay bị coi nhẹ của system testing. So sánh môi trường đích (kiến trúc §16.2) với lab:

| Yêu cầu | Tới Nơi thật — `staging` | Lab |
|---|---|---|
| Giống production | Cùng cấu hình triển khai, cùng phiên bản service, Kafka, Redis | Một tiến trình Node 22, kho trong bộ nhớ (có thể đổi PGlite) |
| Dữ liệu | Tổng hợp hoặc đã che (masked); không dùng PII production | Dữ liệu mẫu Phụ lục A; nạp lại bằng `POST /api/test/reset` |
| Bên thứ ba | Fake adapter hoặc sandbox của VíXanh, MapViet | VíXanh giả, quãng đường do client gửi (R-GEN-06) |
| Điều khiển thời gian | Clock abstraction cho expiry/timeout (§17.2) | `TOINOI_START_AT`, `/api/test/clock` |
| Móc kiểm thử | Seed/test data API chỉ bật ngoài production | `/api/test/*` chỉ khi `NODE_ENV=test` (R-TEST-01) |
| Thiết bị | Ứng dụng iOS/Android thật hoặc giả lập, trình duyệt Admin | Google Chrome cài sẵn, chạy headless |

### 3.4. Rủi ro

| Rủi ro | Vì sao chỉ thấy rõ ở cấp hệ thống |
|---|---|
| Khách thấy trạng thái cũ, tưởng chưa có tài xế, huỷ chuyến | Cần trang khách, cơ chế làm mới và backend chạy cùng nhau |
| Tài xế xe máy nhận chuyến ô tô | Bộ lọc nằm ở backend, danh sách hiện ở ứng dụng tài xế |
| Bấm đúp «Đặt chuyến» tạo hai chuyến | Liên quan cả giao diện (khoá nút) lẫn API (idempotency) |
| Thông báo lỗi khó hiểu khi bấm sai bước | Chỉ thấy khi đọc màn hình như người dùng |
| Môi trường test khác production ⇒ test đạt nhưng production lỗi | Bản chất của system testing là đo trên môi trường đại diện |

## 4. Phân tích kỹ thuật

### 4.1. Từ đặc tả hệ thống tới test condition

Test condition cấp hệ thống mô tả **điều người dùng làm được và thấy được**, không mô tả hàm nào trả gì. Cùng quy tắc R-FARE-01, component test hỏi «`calculateFare` trả 107.000 không»; system test hỏi «khách chọn Ô tô, nhập 7,4 km, bấm Báo giá thì **màn hình** hiện 107.000 đ không». Cách viết: lấy một tác vụ đầu-cuối (đặt ô tô → hoàn thành), đi theo từng màn hình, ở mỗi chỗ hỏi «người dùng cần thấy gì, hệ thống phải ngăn điều gì, mất bao lâu».

| Mã | Test condition | Test basis | Test type |
|---|---|---|---|
| TC-SYS-01 | Báo giá ô tô 7,4 km ngoài cao điểm, không mưa hiện đúng tổng tiền và cờ cao điểm «Không» | R-UI-01, R-FARE-01, R-UI-04 | Functional |
| TC-SYS-02 | Bấm (kể cả bấm đúp) «Đặt chuyến» tạo đúng một chuyến ở «Đang tìm tài xế» | R-UI-01, R-QUOTE-03, R-IDEM-02 | Functional |
| TC-SYS-03 | Chuyến ô tô chỉ hiện với tài xế ô tô cùng thành phố (D103), không hiện với tài xế xe máy (D101) | R-ACCEPT-01, R-ACCEPT-06, R-UI-02 | Functional |
| TC-SYS-04 | Tài xế đi lần lượt nhận → tới đón → đã đón → bắt đầu chở → hoàn thành; mỗi bước trang tài xế hiện đúng trạng thái | R-STATE-02, R-UI-02 | Functional |
| TC-SYS-05 | Trang khách tự hiện trạng thái mới và mã tài xế trong ≤ 3 giây sau mỗi bước, không tải lại trang; chuyến xong thì nút «Huỷ chuyến» bị khoá | R-UI-01, R-NFR-04, R-CANCEL-01 | Non-functional; Functional |
| TC-SYS-06 | Tài xế bấm «Hoàn thành» khi mới nhận ⇒ bị từ chối có thông báo, trạng thái hai phía giữ nguyên | R-STATE-03, R-UI-02, R-UI-03 | Functional |

### 4.2. Vì sao những condition này thuộc cấp hệ thống

Mục 2.1.2 của syllabus nhắc: mỗi test level có mục tiêu riêng để test đủ rộng mà **không trùng lặp**. Không lặp lại ở cấp hệ thống mọi tổ hợp cước (xe máy/ô tô × cao điểm × mưa × voucher) — chúng đã được khoá ở component test. TC-SYS-01 chỉ cần **một** giá trị đại diện để chứng minh màn hình hiển thị đúng con số backend tính. Ngược lại, TC-SYS-05 không thể kiểm ở cấp thấp hơn: nó cần trình duyệt thật, cơ chế hỏi lại 2 giây và backend cùng chạy.

### 4.3. Thiết kế test case

Gộp TC-SYS-01…05 thành **một** kịch bản dài theo đúng luồng người dùng (chi phí dựng dữ liệu một lần, phản ánh hành trình thật), tách TC-SYS-06 thành kịch bản riêng vì đó là nhánh lỗi.

| Bước | Hành động | Kết quả mong đợi | Condition |
|---|---|---|---|
| 1 | Khách: chọn Ô tô, nhập 7,4 km, bấm «Báo giá» | Tổng 107.000 đ (25.000 + 11.000 × 7,4 = 106.400, làm tròn lên); cao điểm «Không» | 01 |
| 2 | Khách: bấm đúp «Đặt chuyến» | Trạng thái «Đang tìm tài xế»; nút bị khoá | 02 |
| 3 | Tài xế D101 (xe máy): tải chuyến đang chờ | «Chưa có chuyến nào.» | 03 |
| 4 | Tài xế D103 (ô tô): tải chuyến đang chờ | Đúng **1** chuyến, mã khớp mã trên trang khách, 107.000 đ | 02, 03 |
| 5 | D103: «Nhận chuyến» (ETA 5) | Tài xế: «Tài xế đã nhận»; khách trong ≤ 3 giây: «Tài xế đã nhận», tài xế D103 | 04, 05 |
| 6–9 | D103: «Đang tới đón», «Đã đón», «Bắt đầu chở», «Hoàn thành» | Mỗi bước hai trang hiện «Tài xế đang tới», «Đã đón / lấy hàng», «Đang chở / giao», «Hoàn thành»; khách cập nhật trong ≤ 3 giây | 04, 05 |
| 10 | Khách: xem nút «Huỷ chuyến» | Bị khoá | 05 |

## 5. Thực hành

### 5.1. Đề bài và lời giải

Bài thực hành của curriculum: *viết 6 test condition cấp hệ thống cho luồng khách đặt ô tô qua trang `/` và tài xế hoàn thành qua `/driver.html` (R-UI-01, R-UI-02, R-STATE-02)*. Lời giải là bảng TC-SYS-01…06 (mục 4.1) và test case ở mục 4.3. Phần dưới tự động hoá hai test case đó và chạy trên lab.

### 5.2. Tự động hoá bằng Playwright

Tệp `e2e/dat-o-to.spec.ts` mở **hai phiên trình duyệt riêng** — một cho khách, một cho tài xế — và chỉ thao tác qua giao diện: chọn theo nhãn ô nhập, tên nút, `data-testid`. Trích đoạn đo TC-SYS-05:

```ts
/** Thời gian (ms) từ lúc tài xế bấm tới lúc trang khách tự hiện trạng thái mới (R-NFR-04: p95 dưới 2–3 giây). */
async function propagation(customer: Page, label: string, act: () => Promise<void>): Promise<number> {
  const t0 = Date.now();
  await act();
  await expect(customer.getByTestId('trip-status')).toHaveText(label, { timeout: 3_000 });
  return Date.now() - t0;
}
```

Bài có cấu hình Playwright riêng (`articles/system-testing/playwright.config.ts`): khởi động máy chủ lab ở cổng 3114 với `NODE_ENV=test`, đồng hồ 09:00 giờ VN 04/11/2026, tắt giới hạn tần suất. Chạy trong `examples/toi-noi-testing`:

```bash
npx playwright test -c articles/system-testing/playwright.config.ts
```

```text
Running 2 tests using 1 worker

Lan truyền trạng thái tới trang khách (ms): {"Tài xế đã nhận":1907,"Tài xế đang tới":2411,"Đã đón / lấy hàng":1872,"Đang chở / giao":1891,"Hoàn thành":1911}
  ok 1 articles\system-testing\e2e\dat-o-to.spec.ts:44:1 › TC-SYS-01…05 · khách đặt ô tô, chỉ tài xế ô tô thấy chuyến, D103 đi hết các bước, khách tự thấy Hoàn thành (11.1s)
Thông báo lỗi trang tài xế: Không thể thực hiện "complete" khi chuyến đang ở trạng thái "accepted".
  ok 2 articles\system-testing\e2e\dat-o-to.spec.ts:76:1 › TC-SYS-06 · bấm «Hoàn thành» khi mới nhận chuyến ⇒ bị từ chối, trạng thái hai phía giữ nguyên (2.6s)

  2 passed (18.2s)
```

Máy đo: Intel Core i7-10700, 64 GB RAM, Windows 11, Node 22.18, Playwright 1.64.1, Chrome 156 headless. Lần chạy thứ hai cho 1.861–2.436 ms.

### 5.3. Mong đợi và thực tế

| Condition | Mong đợi | Thực tế | Kết luận |
|---|---|---|---|
| TC-SYS-01 | 107.000 đ, cao điểm «Không» | 107.000 đ, «Không» | Đạt |
| TC-SYS-02 | Một chuyến dù bấm đúp | D103 thấy đúng 1 chuyến | Đạt |
| TC-SYS-03 | D101 không thấy, D103 thấy | «Chưa có chuyến nào.» / 1 chuyến | Đạt |
| TC-SYS-04 | Đủ 5 bước theo R-STATE-02 | Đủ 5 bước | Đạt |
| TC-SYS-05 | ≤ 3.000 ms mỗi bước; nút huỷ bị khoá | 1.872–2.411 ms; nút bị khoá | Đạt — chỉ minh hoạ phương pháp (xem 5.4) |
| TC-SYS-06 | Bị từ chối, trạng thái giữ nguyên, có thông báo | Đúng, nhưng thông báo là `Không thể thực hiện "complete" khi chuyến đang ở trạng thái "accepted".` | Đạt về chức năng; **ghi nhận** về usability |

### 5.4. Hai điều test «xanh» chưa nói

- **Số đo lan truyền không chứng minh R-NFR-04.** R-NFR-04 là p95 trên tải thật; ở đây là 5 mẫu, một người dùng, một máy dev. Con số quanh 1,9 giây chủ yếu phản ánh chu kỳ hỏi lại 2 giây của trang khách. Test này chỉ bắt được hồi quy thô (vd ai đó đổi chu kỳ thành 10 giây). Đo đúng nghĩa thuộc Series 09.
- **Thông báo lộ mã nội bộ.** Tài xế đọc `"complete"` và `"accepted"` — mã kỹ thuật tiếng Anh, không phải nhãn trên nút («Hoàn thành») hay trạng thái («Tài xế đã nhận»). Đây là một phát hiện chỉ có ở cấp hệ thống: component test của `nextStatus` chỉ kiểm mã lỗi `INVALID_TRANSITION`, không ai đọc câu chữ. Ghi thành một báo cáo lỗi mức nhẹ, loại usability (mẫu báo cáo đầy đủ ở [TEST-010](/articles/testware-san-pham-kiem-thu)).

## 6. Góc nhìn nâng cao

- **System test qua giao diện hay qua API?** Cả hai đều là system testing nếu test object là cả hệ thống và test basis là đặc tả hệ thống. Qua API nhanh và ổn định hơn nhưng bỏ sót phần giao diện (khoá nút, tự làm mới, câu chữ). Thường kết hợp: phần lớn condition qua API, vài hành trình quan trọng qua giao diện.
- **«Gần production» là phổ, không phải có/không.** Lab dùng kho trong bộ nhớ; chạy lại với `TOINOI_DB=pglite` gần thật hơn một bước, staging thêm Kafka, Redis, nhiều bản service. Mỗi khác biệt là một rủi ro phải ghi vào báo cáo: lab không thể cho thấy hai tài xế ở hai bản service cùng nhận một chuyến (R-ACCEPT-03 cần nhiều kết nối thật — xem giới hạn PGlite trong README lab).
- **Mô phỏng sub-system có giá phải trả.** VíXanh giả trả lời tức thì và luôn đúng hợp đồng; sandbox thật có độ trễ, lỗi lạ và giới hạn. Kiểm hành vi thật với VíXanh là system integration testing ([TEST-015](/articles/system-integration-testing)).
- **Độc lập kiểm thử.** System test hay do đội test độc lập làm để tránh thiên kiến «tôi viết nên tôi hiểu đúng» (FL-1.5.3), nhưng đội đó cần hiểu nghiệp vụ — chi tiết ở [TEST-009](/articles/vai-tro-trong-kiem-thu).
- **Test thời gian dễ chập chờn.** `toHaveText(..., { timeout: 3_000 })` với chu kỳ 2 giây chỉ dư khoảng 0,6 giây; máy CI chậm có thể làm test đỏ giả. Cách giảm: dùng đồng hồ tiêm vào cho logic thời gian nghiệp vụ, và tách phép đo hiệu năng khỏi bộ test chức năng chặn merge.
- **Sai lầm thường gặp:** lặp lại ở cấp hệ thống mọi tổ hợp đã kiểm ở component; chỉ test đường vui; chạy «system test» trên môi trường mà mọi thứ là mock; quên kiểm phía người dùng thứ hai (khách thấy gì khi tài xế làm).

## 7. Bài tập và câu hỏi ôn tập

### Bài tập

**Bài 1.** Viết 3 test condition cấp hệ thống cho luồng khách gửi bưu kiện 25 kg ở Hà Nội (R-PARCEL-01, R-PARCEL-02, R-PARCEL-03, R-ACCEPT-01) và chỉ ra condition nào KHÔNG nên viết ở cấp này.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(1) Chọn xe máy với 25 kg ⇒ trang khách báo lỗi xe máy không chở được khối lượng này (<code>VEHICLE_TOO_SMALL</code>) bằng câu dễ hiểu, không cho đặt. (2) Đổi sang ô tô, báo giá ⇒ tổng gồm phụ phí 40.000 đ (2.000 × ⌈25 − 5⌉); đặt chuyến ⇒ chỉ tài xế ô tô Hà Nội D202 thấy chuyến, D201 (xe máy) không thấy. (3) D202 hoàn thành ⇒ trang khách hiện «Hoàn thành». Không nên viết ở cấp này: các biên 5,0/5,1/6,0/6,1 kg, 30,04/30,05 kg của phụ phí và khối lượng — đã thuộc component test của <code>parcel.ts</code>; ở cấp hệ thống chỉ cần một giá trị đại diện.</p>
</details>

**Bài 2.** Tester chạy toàn bộ bộ system test trên máy cá nhân, ví VíXanh giả, kho trong bộ nhớ, rồi báo «hệ thống sẵn sàng release». Nêu ba điểm còn thiếu so với yêu cầu môi trường của system testing.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(1) Môi trường không đại diện: một tiến trình, không Kafka, Redis, không nhiều bản service ⇒ không thấy lỗi phân tán (event trễ, hai tài xế cùng nhận). (2) Không có phép đo phi chức năng có ý nghĩa: p95 của R-NFR-02, R-NFR-03 cần tải và môi trường gần production. (3) Bên thứ ba toàn bản giả ⇒ hành vi thật của VíXanh, MapViet chưa được kiểm (cần system integration testing với sandbox). Thêm: thiếu ứng dụng iOS/Android thật cho hành vi nền, thông báo đẩy.</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Theo CTFL v4.0.1, system testing tập trung vào điều gì?

- a) Interface và tương tác giữa các component
- b) Hành vi tổng thể và năng lực của toàn bộ hệ thống, thường gồm tác vụ đầu-cuối và đặc tính chất lượng phi chức năng
- c) Xác nhận hệ thống sẵn sàng triển khai và đáp ứng nhu cầu nghiệp vụ của người dùng
- d) Interface với các hệ thống và dịch vụ bên ngoài

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b> (mục 2.2.1). a) là component integration testing. c) là acceptance testing. d) là system integration testing.</p>
</details>

**Câu 2 (K2).** Đặc tính chất lượng nào được CTFL v4.0.1 lấy làm ví dụ cho loại nên kiểm trên hệ thống hoàn chỉnh trong môi trường đại diện?

- a) Maintainability
- b) Usability
- c) Portability
- d) Functional correctness

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng.</b> Syllabus nêu usability làm ví dụ. a) và c) là đặc tính phi chức năng nhưng không phải ví dụ được nêu; maintainability thường đánh giá được sớm qua review, static analysis. d) là đặc tính chức năng.</p>
</details>

**Câu 3 (K2).** Đội Tới Nơi chạy system test với ví VíXanh giả thay cho VíXanh thật. Phát biểu nào đúng?

- a) Không được phép: system testing bắt buộc dùng mọi hệ thống thật
- b) Được phép: syllabus nói system testing có thể dùng mô phỏng các sub-system; hành vi thật của VíXanh được kiểm ở test level khác
- c) Dùng ví giả biến test thành component testing
- d) Dùng ví giả biến test thành acceptance testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng.</b> Mô phỏng sub-system là khả năng syllabus nêu rõ. Interface với VíXanh thật là đối tượng của system integration testing. c), d) sai: test level xác định bởi test object, test basis, mục tiêu — không phải bởi việc có dùng bản giả hay không.</p>
</details>

**Câu 4 (K2).** Ca nào phù hợp NHẤT để làm ở system testing thay vì component testing?

- a) `parcelSurcharge(6.1)` trả 4.000 đ
- b) Mọi tổ hợp loại xe × cao điểm × mưa × voucher của công thức cước
- c) Trang khách tự hiện «Tài xế đã nhận» trong ≤ 3 giây sau khi tài xế bấm «Nhận chuyến» trên ứng dụng tài xế
- d) Hàm chuẩn hoá mã voucher bỏ khoảng trắng hai đầu

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng</b>: cần hai giao diện, cơ chế làm mới và backend chạy cùng nhau — chỉ có ở hệ thống hoàn chỉnh. a), d) là quy tắc của một hàm. b) nên khoá ở component test; lặp lại ở cấp hệ thống là trùng lặp mà mục 2.1.2 khuyên tránh.</p>
</details>

**Câu 5 (K1).** Theo CTFL v4.0.1, system testing gắn với loại tài liệu nào làm test basis?

- a) Đặc tả của hệ thống
- b) Mã nguồn của từng component
- c) Hợp đồng interface giữa hai component
- d) Kịch bản vận hành của đối tác thanh toán

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a) đúng</b> — syllabus nói system testing «related to specifications for the system». b) thuộc component testing (white-box). c) thuộc component integration testing. d) liên quan system integration hoặc operational acceptance.</p>
</details>

**Câu 6 (K2).** Phát biểu nào về người thực hiện system testing là đúng theo CTFL v4.0.1?

- a) Chỉ developer viết component mới được làm system testing
- b) System testing có thể do đội test độc lập thực hiện
- c) System testing phải do người dùng cuối thực hiện
- d) System testing chỉ được thực hiện sau acceptance testing

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b> (mục 2.2.1; mục 1.5.3 lấy ví dụ đội test làm system và system integration testing). a) sai: developer thường làm component và component integration. c) là ý tưởng của acceptance testing. d) sai: không có thứ tự cứng như vậy, và test level có thể chồng lấn về thời gian.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — mục 2.2 Test Levels and Test Types: 2.2.1 Test Levels (FL-2.2.1), 2.2.2 Test Types; mục 1.5.3 Independence of Testing; mục 2.1.2 good testing practices. PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (system testing, system under test, test environment).
- ISO/IEC/IEEE 29119-1, *Software and systems engineering — Software testing — Part 1: General concepts* — khái niệm test level, test environment.
- Tài liệu kiến trúc Tới Nơi (nội bộ blog): §8.1 luồng đặt xe đầu-cuối, §14.1 SLO khởi tạo, §16.2 môi trường, §17.1 chiến lược test theo kiến trúc, §17.2 testability hooks.
- Phòng lab: `SPEC.md` (R-UI, R-STATE, R-ACCEPT, R-NFR, R-TEST) và bộ test của bài — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Playwright — https://playwright.dev/ (công cụ ví dụ, không phải yêu cầu của ISTQB).
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-010 Testware](/articles/testware-san-pham-kiem-thu) · [TEST-011 Tổng quan 5 test level](/articles/tong-quan-5-test-level-istqb) · [TEST-013 Component Integration Testing](/articles/component-integration-testing) · [TEST-015 System Integration Testing](/articles/system-integration-testing) · [TEST-016 Acceptance Testing](/articles/acceptance-testing-tong-quan)

Bài tiếp theo: [TEST-015 · System Integration Testing](/articles/system-integration-testing).
