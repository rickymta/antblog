---
slug: quy-trinh-kiem-thu-test-process
title: "Quy trình kiểm thử (Test Process): từ STLC tới bảy nhóm hoạt động của CTFL v4"
summary: "Bảy nhóm hoạt động test theo ISTQB CTFL v4.0.1, vì sao «STLC» chỉ là thuật ngữ ngành, ngữ cảnh làm quy trình thay đổi ra sao và cách truy vết từ yêu cầu tới kết quả, qua tính năng tài xế nhận chuyến."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-007, test-process, stlc, traceability, test-planning]
language: vi
---

Nhận bản build, mở ứng dụng, bấm thử vài luồng rồi báo «ổn» — đó là kiểm thử không có quy trình. Nó không trả lời được ba câu mà người ra quyết định release cần: đã test những gì, test tới đâu thì đủ, còn rủi ro nào. Bài này giải thích **test process** (quy trình kiểm thử) theo ISTQB CTFL v4.0.1: bảy nhóm hoạt động, quy trình thay đổi theo ngữ cảnh thế nào, và **traceability** (truy vết) nối mọi thứ lại với nhau. Ta đi trọn một vòng trên tính năng «tài xế nhận chuyến» của Tới Nơi, có bộ test chạy thật.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-007</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S01 · Fundamentals of Software Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/software-testing-la-gi-testing-khac-debugging">TEST-001 Software Testing là gì?</a>, <a href="/articles/muc-tieu-kiem-thu-va-quality-assurance">TEST-006 Mục tiêu kiểm thử và QA</a>, <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (đọc SPEC): bảng bảy hoạt động test cho «tài xế nhận chuyến» (R-ACCEPT) + bộ test chạy được trong <code>examples/toi-noi-testing/articles/quy-trinh-kiem-thu-test-process/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.4.1, 1.4.2, 1.4.4 · FL-1.4.1 (K2), FL-1.4.2 (K2), FL-1.4.4 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Ở [TEST-001](/articles/software-testing-la-gi-testing-khac-debugging) ta đã thấy testing không chỉ là chạy chương trình, và ở [TEST-006](/articles/muc-tieu-kiem-thu-va-quality-assurance) ta đã đặt mục tiêu test cho Tới Nơi. Câu hỏi còn lại là **làm theo trình tự nào** để đạt các mục tiêu đó.

Không có quy trình, đội test hay gặp ba vấn đề:

- **Thiếu mà không biết thiếu.** Không ai liệt kê điều kiện cần test, nên một quy tắc như «hai tài xế cùng nhận thì chỉ một người thắng» có thể không có test nào.
- **Không biết khi nào dừng.** «Test xong chưa?» được trả lời bằng cảm giác thay vì bằng tiêu chí.
- **Không giải thích được kết quả.** Báo cáo «chạy 120 test, 3 lỗi» không cho biết yêu cầu nào đã an toàn, rủi ro nào còn treo.

Test process giải quyết cả ba bằng một bộ khung chung. Bộ khung này dùng cho **mọi** dự án — từ một sprint hai tuần tới một dự án kéo dài cả năm — nhưng luôn được **điều chỉnh** theo ngữ cảnh. Bài cũng giải thích từ «STLC» mà bạn sẽ gặp ở nhiều khoá học: nó là gì, và vì sao CTFL v4 không dùng nó.

## 2. Lý thuyết theo ISTQB

### 2.1. Test process là gì

Theo CTFL v4.0.1 (mục 1.4), testing phụ thuộc ngữ cảnh, nhưng ở mức tổng quát luôn có những **nhóm hoạt động chung** mà thiếu chúng thì khó đạt mục tiêu test. Tập các nhóm hoạt động đó tạo thành **test process**. Hoạt động nào được đưa vào, làm bằng cách nào và làm lúc nào được quyết định trong **test planning** cho từng tình huống cụ thể. Syllabus trỏ tới chuẩn ISO/IEC/IEEE 29119-2 nếu cần mô tả quy trình chi tiết hơn.

Hai ý cần nhớ ngay từ đầu:

1. Các hoạt động **có vẻ** nối tiếp nhau theo logic, nhưng trong thực tế thường **lặp lại** hoặc **chạy song song**.
2. Quy trình luôn phải **may đo** (tailor) cho hệ thống và dự án — không có bản «chuẩn» để áp nguyên.

### 2.2. Bảy nhóm hoạt động (FL-1.4.1)

| Nhóm hoạt động | Trả lời câu hỏi | Nhiệm vụ chính (diễn đạt lại từ syllabus) |
|---|---|---|
| **Test planning** — lập kế hoạch | Test để đạt gì, bằng cách nào, trong giới hạn nào? | Xác định test objective rồi chọn cách tiếp cận (approach) đạt được mục tiêu đó trong ràng buộc của ngữ cảnh |
| **Test monitoring & test control** — giám sát và điều khiển | Đang đi đúng kế hoạch không? Cần làm gì để về đích? | Giám sát: liên tục kiểm tra mọi hoạt động, so tiến độ thực tế với kế hoạch. Điều khiển: thực hiện hành động cần thiết để đạt mục tiêu |
| **Test analysis** — phân tích | Test **cái gì**? | Phân tích test basis để tìm tính năng test được; định nghĩa và xếp ưu tiên **test condition** theo rủi ro; tìm defect ngay trong test basis và đánh giá khả năng test (testability) |
| **Test design** — thiết kế | Test **thế nào**? | Biến test condition thành test case và testware khác (vd test charter); xác định coverage item; định ra yêu cầu test data, thiết kế môi trường, xác định hạ tầng và công cụ |
| **Test implementation** — cài đặt | Đã sẵn sàng chạy chưa? | Tạo/mua testware cần cho thực thi (vd test data); gom test case thành test procedure và test suite; viết test script thủ công/tự động; xếp lịch thực thi; dựng và kiểm môi trường |
| **Test execution** — thực thi | Hệ thống có hành xử như mong đợi? | Chạy test theo lịch (thủ công hoặc tự động, kể cả continuous testing, pair testing); so kết quả thực tế với kết quả mong đợi; ghi log; phân tích bất thường để báo cáo failure |
| **Test completion** — kết thúc | Học được gì, bàn giao gì? | Thường ở mốc dự án (release, cuối iteration, xong một test level): tạo change request/backlog item cho defect chưa xử lý; lưu trữ hoặc bàn giao testware còn dùng được; đưa môi trường về trạng thái đã thoả thuận; rút bài học; lập test completion report |

Hai cặp hay bị nhập làm một: **analysis** trả lời «test cái gì» bằng test condition đo được độ bao phủ; **design** trả lời «test thế nào» bằng test case cụ thể. **Monitoring** là quan sát; **control** là hành động. Sản phẩm sinh ra ở từng hoạt động (test plan, test case, test log…) có bài riêng: [TEST-010 Testware](/articles/testware-san-pham-kiem-thu).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 272" role="img" aria-label="Bảy nhóm hoạt động test. Dải trên cùng là test monitoring và test control, nối bằng nét đứt xuống cả sáu hoạt động bên dưới. Hàng giữa từ trái sang phải: test planning, test analysis, test design, test implementation, test execution, test completion, nối bằng mũi tên. Hai vòng phản hồi nét đứt: từ test execution quay về test analysis khi phát hiện lỗ hổng trong test basis; từ test completion quay về test planning cho iteration hoặc release sau.">
<defs>
<marker id="t007-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="t007-back" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<text x="460" y="22" text-anchor="middle" class="t-small">thứ tự logic từ trái sang phải — nhưng thực tế các hoạt động lặp lại và chồng lên nhau</text>
<rect x="26" y="34" width="867" height="44" rx="8" class="f-phys-soft s-phys"/>
<text x="460" y="53" text-anchor="middle" class="t-b t-phys">Test monitoring &amp; test control</text>
<text x="460" y="69" text-anchor="middle" class="t-small">chạy suốt quá trình: so tiến độ thực tế với kế hoạch, ra quyết định điều chỉnh để đạt mục tiêu test</text>
<line x1="92" y1="78" x2="92" y2="118" class="s-phys dash" stroke-width="1.2"/>
<line x1="239" y1="78" x2="239" y2="118" class="s-phys dash" stroke-width="1.2"/>
<line x1="386" y1="78" x2="386" y2="118" class="s-phys dash" stroke-width="1.2"/>
<line x1="533" y1="78" x2="533" y2="118" class="s-phys dash" stroke-width="1.2"/>
<line x1="680" y1="78" x2="680" y2="118" class="s-phys dash" stroke-width="1.2"/>
<line x1="827" y1="78" x2="827" y2="118" class="s-phys dash" stroke-width="1.2"/>
<rect x="26" y="120" width="132" height="62" rx="8" class="f-pk-soft s-pk"/><text x="92" y="146" text-anchor="middle" class="t-b">Test planning</text><text x="92" y="164" text-anchor="middle" class="t-small">mục tiêu, cách tiếp cận</text>
<rect x="173" y="120" width="132" height="62" rx="8" class="f-pk-soft s-pk"/><text x="239" y="146" text-anchor="middle" class="t-b">Test analysis</text><text x="239" y="164" text-anchor="middle" class="t-small">test cái gì?</text>
<rect x="320" y="120" width="132" height="62" rx="8" class="f-pk-soft s-pk"/><text x="386" y="146" text-anchor="middle" class="t-b">Test design</text><text x="386" y="164" text-anchor="middle" class="t-small">test thế nào?</text>
<rect x="467" y="120" width="132" height="62" rx="8" class="f-pk-soft s-pk"/><text x="533" y="146" text-anchor="middle" class="t-b">Test implementation</text><text x="533" y="164" text-anchor="middle" class="t-small">sẵn sàng để chạy</text>
<rect x="614" y="120" width="132" height="62" rx="8" class="f-pk-soft s-pk"/><text x="680" y="146" text-anchor="middle" class="t-b">Test execution</text><text x="680" y="164" text-anchor="middle" class="t-small">chạy, so sánh, ghi log</text>
<rect x="761" y="120" width="132" height="62" rx="8" class="f-pk-soft s-pk"/><text x="827" y="146" text-anchor="middle" class="t-b">Test completion</text><text x="827" y="164" text-anchor="middle" class="t-small">bàn giao, rút bài học</text>
<line x1="158" y1="151" x2="171" y2="151" class="s-pk" stroke-width="1.6" marker-end="url(#t007-arrow)"/>
<line x1="305" y1="151" x2="318" y2="151" class="s-pk" stroke-width="1.6" marker-end="url(#t007-arrow)"/>
<line x1="452" y1="151" x2="465" y2="151" class="s-pk" stroke-width="1.6" marker-end="url(#t007-arrow)"/>
<line x1="599" y1="151" x2="612" y2="151" class="s-pk" stroke-width="1.6" marker-end="url(#t007-arrow)"/>
<line x1="746" y1="151" x2="759" y2="151" class="s-pk" stroke-width="1.6" marker-end="url(#t007-arrow)"/>
<path d="M680 182 L680 214 L239 214 L239 186" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#t007-back)"/>
<text x="460" y="208" text-anchor="middle" class="t-small t-warn">chạy test thấy đặc tả mơ hồ ⇒ quay lại phân tích</text>
<path d="M827 182 L827 252 L92 252 L92 186" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#t007-back)"/>
<text x="460" y="246" text-anchor="middle" class="t-small t-warn">bài học (lessons learned) ⇒ kế hoạch của iteration / release sau</text>
</svg>
</div>
<figcaption><b>Hình 1 · Bảy nhóm hoạt động của test process.</b> Monitoring &amp; control không phải một «bước» nằm giữa hàng mà phủ lên tất cả. Hai vòng nét đứt là hai kiểu lặp thường gặp; còn nhiều vòng khác (thiết kế thêm test case khi chạy, cài lại môi trường…).</figcaption>
</figure>

### 2.3. Ngữ cảnh quyết định hình dạng quy trình (FL-1.4.2)

Testing không làm riêng lẻ: nó là một phần của quy trình phát triển, được các bên liên quan (stakeholder) cấp ngân sách, và đích cuối là giúp đáp ứng nhu cầu kinh doanh của họ. Vì vậy syllabus liệt kê các yếu tố ngữ cảnh sau:

| Yếu tố | Ví dụ ở Tới Nơi |
|---|---|
| Stakeholder (nhu cầu, kỳ vọng, mức sẵn sàng hợp tác) | Bộ phận vận hành cần biết trước rủi ro «một chuyến hai tài xế» |
| Thành viên đội (kỹ năng, kinh nghiệm, thời gian) | Đội có SDET viết được test đồng thời hay chỉ có tester thủ công |
| Domain nghiệp vụ (mức quan trọng, rủi ro, quy định pháp lý) | Thanh toán qua VíXanh nhạy cảm hơn hẳn đổi màu nút |
| Yếu tố kỹ thuật (loại phần mềm, kiến trúc, công nghệ) | Microservices + Kafka (hệ đích) khác modular monolith (lab) |
| Ràng buộc dự án (phạm vi, thời gian, ngân sách, nguồn lực) | Sprint hai tuần, một tester cho bốn developer (giả định) |
| Yếu tố tổ chức (cơ cấu, chính sách, thực hành sẵn có) | Có quy định bắt buộc test plan ký duyệt hay không |
| SDLC (thực hành kỹ thuật, phương pháp phát triển) | Scrum + CI hay Waterfall — xem [TEST-008](/articles/kiem-thu-trong-sdlc-agile-waterfall) |
| Công cụ (có sẵn, dễ dùng, tuân thủ) | vitest, Playwright, quản lý test case bằng công cụ hay bảng tính |

Các yếu tố này tác động tới **test strategy, kỹ thuật test, mức tự động hoá, mức coverage yêu cầu, độ chi tiết của testware và cách báo cáo**. Hai đội cùng test «tài xế nhận chuyến» có thể có hai quy trình trông rất khác nhau — và cả hai đều đúng.

### 2.4. Truy vết giữa test basis và testware (FL-1.4.4)

**Traceability** là mối nối hai chiều giữa phần tử của **test basis** (yêu cầu, rủi ro…), testware gắn với nó (test condition, test case), **test result** và defect. Syllabus nêu các giá trị của việc giữ truy vết:

- **Đánh giá coverage.** Truy vết test case → yêu cầu cho biết yêu cầu nào đã có test. Nếu test basis có tiêu chí coverage đo được, chúng dùng được như KPI cho mức đạt mục tiêu test.
- **Đánh giá rủi ro còn lại.** Truy vết kết quả test → rủi ro cho biết mức **residual risk** (rủi ro tồn dư) của đối tượng test.
- **Phân tích ảnh hưởng thay đổi.** Một yêu cầu đổi ⇒ biết ngay test case nào phải sửa, chạy lại.
- **Kiểm toán và quản trị CNTT.** Có bằng chứng nối từ yêu cầu tới kết quả.
- **Báo cáo dễ hiểu.** Test progress report và test completion report nói theo trạng thái từng yêu cầu thay vì chỉ đếm test.

### 2.5. Bảng thuật ngữ

| Thuật ngữ (ISTQB) | Nghĩa tiếng Việt dùng trong blog | Ghi chú |
|---|---|---|
| test process | quy trình kiểm thử | tập các nhóm hoạt động test; keyword chương 1 từ v4.0.1 |
| test activity | hoạt động test | bảy nhóm ở mục 2.2 |
| test basis | cơ sở kiểm thử | tài liệu/nguồn để suy ra test (ở lab: `SPEC.md`) |
| test object | đối tượng kiểm thử | thứ được test (vd use case `acceptTrip`) |
| test condition | điều kiện kiểm thử | khía cạnh test được của test basis |
| coverage item | hạng mục bao phủ | đơn vị đếm coverage (vd mỗi mã lỗi) |
| test case | ca kiểm thử | tiền điều kiện, đầu vào, kết quả mong đợi |
| test procedure / test suite | thủ tục / bộ kiểm thử | chuỗi test case có thứ tự / tập test procedure |
| test result | kết quả kiểm thử | passed, failed… sau khi chạy |
| entry / exit criteria | tiêu chí vào / ra | điều kiện bắt đầu / kết thúc một hoạt động hay test level |
| traceability | truy vết | nối test basis ↔ testware ↔ kết quả ↔ defect |
| residual risk | rủi ro tồn dư | rủi ro còn lại sau khi test |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Ngoài syllabus ISTQB: «STLC»</span>
<p><b>STLC (Software Testing Life Cycle)</b> là thuật ngữ phổ biến trong các khoá học và tin tuyển dụng, nhưng <b>không xuất hiện trong syllabus CTFL v4.0.1</b> (đã tra toàn văn). Cách trình bày hay gặp nhất chia thành sáu pha nối tiếp, mỗi pha có tiêu chí vào/ra riêng. Không có văn bản chuẩn nào cố định số pha, nên các nguồn khác nhau đôi chút. Đối chiếu gần đúng:</p>
<table>
<thead><tr><th>Pha STLC (cách gọi phổ biến)</th><th>Tương ứng trong CTFL v4.0.1</th></tr></thead>
<tbody>
<tr><td>Requirement Analysis</td><td>Test analysis (một phần: phân tích test basis, testability)</td></tr>
<tr><td>Test Planning</td><td>Test planning</td></tr>
<tr><td>Test Case Development</td><td>Test analysis (test condition) + test design + một phần test implementation (script, test data)</td></tr>
<tr><td>Test Environment Setup</td><td>Test design (thiết kế môi trường) + test implementation (dựng và kiểm môi trường)</td></tr>
<tr><td>Test Execution</td><td>Test execution</td></tr>
<tr><td>Test Cycle Closure</td><td>Test completion</td></tr>
<tr><td>(không có pha riêng)</td><td>Test monitoring &amp; test control — xuyên suốt</td></tr>
</tbody>
</table>
<p>Khác biệt đáng nhớ: STLC thường được vẽ như các pha <b>tuần tự</b>; ISTQB nói về <b>nhóm hoạt động</b> có thể lặp, chồng lên nhau và được may đo. Đi thi CTFL, hãy dùng tên bảy nhóm hoạt động.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Sau khi khách đặt chuyến từ một báo giá còn hạn, chuyến nằm ở `searching`. Tài xế mở danh sách chuyến chờ và bấm «Nhận». Đây là mắt xích nối Booking với Trip trong luồng đặt xe. Các quy tắc trích từ `SPEC.md` của lab:

| Mã | Quy tắc (tóm tắt) |
|---|---|
| R-ACCEPT-01 | Nhận được khi tài xế `online`, cùng thành phố, đúng loại xe, chuyến đang `searching` |
| R-ACCEPT-02 | Kiểm theo thứ tự, báo lỗi đầu tiên: `TRIP_NOT_FOUND` → `DRIVER_OFFLINE` → `DRIVER_BUSY` → `CITY_MISMATCH` → `VEHICLE_MISMATCH` → chuyến đã huỷ `INVALID_TRANSITION` → đã có người nhận `TRIP_ALREADY_TAKEN` |
| R-ACCEPT-03 | Nhiều tài xế nhận đồng thời ⇒ **đúng một** người thắng, người thua vẫn `online`; một tài xế không giữ được hai chuyến |
| R-ACCEPT-04 | Lệnh nhận gửi `etaMinutes` (1–60), lưu cùng `acceptedAt` |
| R-ACCEPT-05 | Nhận thành công ⇒ tài xế `busy` |
| R-ACCEPT-06 | Danh sách chuyến chờ sắp theo `requestedAt` tăng dần, rồi theo `id` |
| R-ACCEPT-07 | Gửi lại lệnh nhận cho chuyến mình đang giữ ⇒ trả chuyến hiện tại, không đổi gì |

Liên quan: R-STATE-02 (chuyển `searching → accepted`), R-API-10 (endpoint `POST /api/trips/:id/accept`).

### 3.2. System context

- **Hệ thống đích** (tài liệu kiến trúc §4.5, §8.1): Matching & Dispatch gửi offer có TTL tới Driver App qua realtime gateway; tài xế accept; Dispatch gán bằng **atomic claim/lease** hoặc optimistic concurrency để chỉ một người thắng, lưu vào datastore rồi phát `DriverAssigned` qua Kafka (outbox) cho Trip Service. Driver Availability giữ trạng thái `online/busy`.
- **Lab** (SPEC R-ARCH): modular monolith, mô hình «kéo» — tài xế tự xem danh sách và nhận (R-ACCEPT-06). Use case `acceptTrip` trong `src/app/trips.ts`; giữ tài xế (`online → busy` có điều kiện) rồi giành chuyến (`searching → accepted` có điều kiện). **Chưa có trong lab:** offer TTL, Kafka, nhiều instance chạy song song.
- **Tải tham chiếu** (hồ sơ dự án): ≈ 70 chuyến tạo mới mỗi giây lúc cao điểm, 25.000 tài xế trực tuyến — hai tài xế cùng bấm một chuyến trong cùng giây là chuyện xảy ra hằng ngày, không phải ca hiếm.

### 3.3. Rủi ro có thể xảy ra

| Mã | Rủi ro sản phẩm | Hậu quả | Mức (giả định) |
|---|---|---|---|
| RR-01 | Hai tài xế cùng được gán một chuyến | Hai xe tới đón, một tài xế mất công, khiếu nại | Cao |
| RR-02 | Một tài xế giữ hai chuyến cùng lúc | Một khách chờ vô ích tới khi huỷ | Cao |
| RR-03 | Nhận được chuyến khách đã huỷ | Tài xế chạy tới điểm đón không có ai | Trung bình |
| RR-04 | Tài xế sai thành phố / sai loại xe nhận được | Ô tô đi đón khách đặt xe máy, sai cước | Trung bình |
| RR-05 | Mạng chập chờn, app gửi lại lệnh nhận ⇒ ETA hoặc giờ nhận bị ghi đè | Mốc 2 phút miễn phí huỷ (R-CANCEL-02) tính sai | Trung bình |
| RR-06 | Danh sách chờ sắp sai thứ tự | Chuyến đặt trước bị bỏ quên, quá 5 phút bị huỷ `no_driver` | Thấp |

## 4. Phân tích kỹ thuật

Giả định ngữ cảnh: tính năng nằm trong một sprint hai tuần, đội gồm bốn developer và một tester, có pipeline CI chạy vitest. Phần này làm hai hoạt động «nặng chất xám» nhất — analysis và design; mục 5 làm phần còn lại.

### 4.1. Test analysis — test cái gì

Đọc R-ACCEPT, tester liệt kê **test condition** và xếp ưu tiên theo rủi ro ở mục 3.3:

| Test condition | Truy vết tới | Rủi ro | Ưu tiên |
|---|---|---|---|
| TCOND-ACC-01 · Tài xế đủ điều kiện nhận được, chuyển `busy`, có `acceptedAt` + ETA | R-ACCEPT-01, 04, 05 | — | Cao |
| TCOND-ACC-02 · Mỗi lý do từ chối trả đúng mã lỗi theo thứ tự | R-ACCEPT-02 | RR-03, RR-04 | Trung bình |
| TCOND-ACC-03 · Chuyến đã huỷ không nhận được | R-ACCEPT-02 | RR-03 | Trung bình |
| TCOND-ACC-04 · Hai tài xế nhận đồng thời: đúng một người thắng | R-ACCEPT-03 | RR-01 | Cao |
| TCOND-ACC-05 · Một tài xế nhận đồng thời hai chuyến: đúng một thành công | R-ACCEPT-03 | RR-02 | Cao |
| TCOND-ACC-06 · Gửi lại lệnh nhận không đổi ETA và `acceptedAt` | R-ACCEPT-07 | RR-05 | Trung bình |
| TCOND-ACC-07 · Thứ tự danh sách chuyến chờ | R-ACCEPT-06 | RR-06 | Thấp |

Phân tích còn **soi chính test basis** — một dạng static testing. Ba ghi chú tester gửi lại cho BA trong lúc phân tích:

1. *R-ACCEPT-07 đứng ngay sau «chuyến không tồn tại»*: tài xế đang `busy` gửi lại lệnh nhận chuyến của chính mình nhận 200, không phải `DRIVER_BUSY`. Đó là chủ đích — nhưng phải có test riêng, vì cài đặt «tự nhiên» sẽ kiểm `busy` trước.
2. *R-ACCEPT-02 nói lệnh nhận không áp quy tắc ẩn chuyến của R-API-05*: tài xế HN gọi nhận chuyến SG nhận `CITY_MISMATCH` (422), tức biết chuyến đó tồn tại. Hỏi lại: có chấp nhận rò rỉ này không? (Đặc tả trả lời: có, có chủ đích — ghi vào biên bản.)
3. *Testability của «đồng thời»* (R-ACCEPT-03): một tiến trình Node chạy hai lời gọi `Promise` xen kẽ chứ không song song thật. Kế hoạch phải nói rõ test ở lab chứng minh đến đâu (xem mục 6).

### 4.2. Test design — test thế nào

Mỗi mã lỗi của R-ACCEPT-02 là một **coverage item**; mỗi test condition sinh ít nhất một test case. Bảy test case đầu tiên:

| Test case | Test condition | Tiền điều kiện | Đầu vào | Kết quả mong đợi |
|---|---|---|---|---|
| TC-ACC-01 | 01 | Chuyến xe máy SG `searching`; D101 xe máy SG `online` | D101 nhận, ETA 5 | `accepted`, `driverId = D101`, ETA 5, `acceptedAt` = giờ hệ thống; D101 `busy` |
| TC-ACC-02 | 02 | Như trên; D104 `offline` | D104 nhận | `DRIVER_OFFLINE` |
| TC-ACC-03 | 02 | Như trên; D201 xe máy HN | D201 nhận | `CITY_MISMATCH` |
| TC-ACC-04 | 02 | Như trên; D103 ô tô SG | D103 nhận | `VEHICLE_MISMATCH` |
| TC-ACC-05 | 04 | Như trên; D101, D102 `online` | D101 và D102 nhận cùng lúc | Đúng 1 thành công, 1 `TRIP_ALREADY_TAKEN`; một `busy`, một `online` |
| TC-ACC-06 | 06 | D101 đã nhận với ETA 5 | 1 phút sau D101 nhận lại với ETA 9 | Trả chuyến cũ: ETA vẫn 5, `acceptedAt` không đổi |
| TC-ACC-07 | 03 | Khách C001 đã huỷ chuyến | D101 nhận | `INVALID_TRANSITION`; D101 vẫn `online` |

Design cũng chốt **yêu cầu test data** (khách C001; tài xế D101–D104, D201 của Phụ lục A SPEC) và **môi trường**: hệ thống trong bộ nhớ với đồng hồ cố định 09:00 giờ VN (ngoài cao điểm, để giá không nhiễu). TCOND-ACC-05 và 07 chưa có test case — ta sẽ thấy ma trận truy vết lộ ra điều đó.

## 5. Thực hành

### 5.1. Bảng bảy hoạt động cho «tài xế nhận chuyến»

Đây là bài thực hành của bài: mỗi hoạt động ghi **đầu vào, nhiệm vụ chính, đầu ra**.

| Hoạt động | Đầu vào | Nhiệm vụ chính | Đầu ra |
|---|---|---|---|
| Test planning | User story, `SPEC.md` R-ACCEPT, danh sách rủi ro, lịch sprint | Mục tiêu: xác minh R-ACCEPT, tìm failure đồng thời, cấp thông tin cho quyết định release. Phạm vi trong: R-ACCEPT-01…07 trên lab; ngoài: offer TTL, Kafka. Entry: SPEC đã review, build có API test. Exit: mọi test case đã chạy, 0 defect mức cao còn mở, mọi yêu cầu R-ACCEPT có ít nhất một test passed hoặc ngoại lệ được ghi | Phần test của sprint plan (mục tiêu, phạm vi, entry/exit criteria, rủi ro) |
| Test monitoring & control | Kế hoạch, log chạy test, danh sách defect | Theo dõi % test condition có test case, % test case đã chạy, defect mở. Ví dụ control: ngày 7 TC-ACC-05 failed ⇒ dời phiên exploratory sang ngày 9, chạy thêm trên PGlite | Cập nhật tiến độ ở daily, quyết định điều chỉnh |
| Test analysis | SPEC R-ACCEPT, rủi ro RR-01…06 | Lập test condition có ưu tiên; ghi câu hỏi về test basis | TCOND-ACC-01…07, ba ghi chú gửi BA (mục 4.1) |
| Test design | Test condition | Suy test case, coverage item, yêu cầu test data và môi trường | TC-ACC-01…07, yêu cầu dữ liệu và môi trường |
| Test implementation | Test case, lab | Viết test script tự động, chuẩn bị dữ liệu mẫu, sắp lịch chạy (ca rủi ro cao trước), kiểm môi trường chạy được | `nhan-chuyen.test.ts`, lệnh chạy, thứ tự ưu tiên |
| Test execution | Test suite, build | Chạy, so kết quả thực tế với mong đợi, ghi log, phân tích bất thường, báo defect | Log vitest, defect report (nếu có) |
| Test completion | Log, defect, ma trận truy vết | Ghi khoảng trống còn lại vào backlog, giữ test trong bộ regression, rút bài học cho retrospective | Tóm tắt test completion của sprint (mục 5.4) |

### 5.2. Implementation và execution: bộ test chạy thật

Bảy test case được viết thành test vitest. Tên mỗi test bắt đầu bằng **mã test case rồi mã yêu cầu** — một quy ước nhỏ khiến log chạy test cũng là bảng truy vết. Trích một test (toàn bộ tệp ở `examples/toi-noi-testing/articles/quy-trinh-kiem-thu-test-process/nhan-chuyen.test.ts`):

```ts
it('TC-ACC-05 · R-ACCEPT-03 · hai tài xế cùng nhận: đúng một người thắng, người thua vẫn online', async () => {
  const trip = await chuyenXeMaySG();
  const results = await Promise.allSettled([
    acceptTrip(system.deps, driver('D101'), trip.id, 5),
    acceptTrip(system.deps, driver('D102'), trip.id, 7),
  ]);
  const thang = results.filter((r) => r.status === 'fulfilled');
  const thua = results.filter((r) => r.status === 'rejected');
  expect(thang).toHaveLength(1);
  expect(thua).toHaveLength(1);
  expect((thua[0] as PromiseRejectedResult).reason).toMatchObject({ code: 'TRIP_ALREADY_TAKEN' });
  const statuses = [await trangThaiTaiXe('D101'), await trangThaiTaiXe('D102')].sort();
  expect(statuses).toEqual(['busy', 'online']);
});
```

Chạy (Node 22, vitest 3.2.7, trong thư mục `examples/toi-noi-testing`):

```bash
npx vitest run articles/quy-trinh-kiem-thu-test-process --reporter=verbose
```

```text
✓ TEST-007 · Tài xế nhận chuyến (R-ACCEPT) > TC-ACC-01 · R-ACCEPT-01/04/05 · tài xế đủ điều kiện nhận được chuyến, chuyển sang busy
✓ TEST-007 · Tài xế nhận chuyến (R-ACCEPT) > TC-ACC-02 · R-ACCEPT-02 · tài xế offline bị từ chối DRIVER_OFFLINE
✓ TEST-007 · Tài xế nhận chuyến (R-ACCEPT) > TC-ACC-03 · R-ACCEPT-02 · tài xế khác thành phố bị từ chối CITY_MISMATCH
✓ TEST-007 · Tài xế nhận chuyến (R-ACCEPT) > TC-ACC-04 · R-ACCEPT-02 · tài xế ô tô nhận chuyến xe máy bị từ chối VEHICLE_MISMATCH
✓ TEST-007 · Tài xế nhận chuyến (R-ACCEPT) > TC-ACC-05 · R-ACCEPT-03 · hai tài xế cùng nhận: đúng một người thắng, người thua vẫn online
✓ TEST-007 · Tài xế nhận chuyến (R-ACCEPT) > TC-ACC-06 · R-ACCEPT-07 · gửi lại lệnh nhận cho chuyến mình đang giữ: trả chuyến cũ, không đổi ETA
✓ TEST-007 · Tài xế nhận chuyến (R-ACCEPT) > TC-ACC-07 · R-ACCEPT-02 · chuyến khách đã huỷ bị từ chối INVALID_TRANSITION

 Test Files  1 passed (1)
      Tests  7 passed (7)
```

(Đường dẫn tệp và thời gian từng test được lược bớt cho dễ đọc.) Kết quả thực tế khớp kết quả mong đợi ở cả bảy test case. Lưu ý: «7 passed» chỉ nói **những gì đã test** đều đạt — nó không nói đã test đủ. Câu đó thuộc về truy vết.

### 5.3. Ma trận truy vết — thứ mà «7 passed» không nói

| Yêu cầu | Test condition | Test case | Kết quả | Rủi ro được giảm |
|---|---|---|---|---|
| R-ACCEPT-01 | TCOND-ACC-01 | TC-ACC-01 | Passed | — |
| R-ACCEPT-02 | TCOND-ACC-02, 03 | TC-ACC-02, 03, 04, 07 | Passed | RR-03, RR-04 |
| R-ACCEPT-03 | TCOND-ACC-04 | TC-ACC-05 | Passed | RR-01 |
| R-ACCEPT-03 | TCOND-ACC-05 | **chưa có** | — | RR-02 **còn nguyên** |
| R-ACCEPT-04, 05 | TCOND-ACC-01 | TC-ACC-01 | Passed | — |
| R-ACCEPT-06 | TCOND-ACC-07 | **chưa có** | — | RR-06 còn nguyên |
| R-ACCEPT-07 | TCOND-ACC-06 | TC-ACC-06 | Passed | RR-05 |

Đọc theo yêu cầu: 6/7 mã R-ACCEPT có ít nhất một test passed, nhưng R-ACCEPT-03 mới phủ **một nửa**. Đọc theo rủi ro: RR-02 — mức **cao** — chưa được giảm chút nào. Đó là thông tin quyết định release, và nó chỉ hiện ra nhờ ma trận. Coverage item của R-ACCEPT-02 cũng còn thiếu: `DRIVER_BUSY` và `TRIP_ALREADY_TAKEN` khi nhận tuần tự (không đồng thời) chưa có test riêng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 196" role="img" aria-label="Chuỗi truy vết hai chiều. Hàng trên: rủi ro RR-01 hai tài xế cùng nhận, nối với yêu cầu R-ACCEPT-03, nối với test condition TCOND-ACC-04, nối với test case TC-ACC-05, nối với kết quả Passed, nối với ô defect ghi không có. Hàng dưới: yêu cầu R-ACCEPT-06 thứ tự danh sách chờ, nối với một khung nét đứt ghi chưa có test condition nào, ma trận lộ khoảng trống và phải nêu trong báo cáo tiến độ.">
<defs>
<marker id="t007-tr" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="87" y="30" text-anchor="middle" class="t-small">rủi ro</text>
<text x="236" y="30" text-anchor="middle" class="t-small">test basis</text>
<text x="385" y="30" text-anchor="middle" class="t-small">test condition</text>
<text x="534" y="30" text-anchor="middle" class="t-small">test case</text>
<text x="683" y="30" text-anchor="middle" class="t-small">test result</text>
<text x="832" y="30" text-anchor="middle" class="t-small">defect</text>
<rect x="20" y="42" width="134" height="52" rx="8" class="f-warn-soft s-warn"/><text x="87" y="64" text-anchor="middle" class="t-mono t-b t-warn">RR-01</text><text x="87" y="81" text-anchor="middle" class="t-small">2 tài xế cùng nhận</text>
<rect x="169" y="42" width="134" height="52" rx="8" class="f-pk-soft s-pk"/><text x="236" y="64" text-anchor="middle" class="t-mono t-b">R-ACCEPT-03</text><text x="236" y="81" text-anchor="middle" class="t-small">đúng một người thắng</text>
<rect x="318" y="42" width="134" height="52" rx="8" class="f-pk-soft s-pk"/><text x="385" y="64" text-anchor="middle" class="t-mono t-b">TCOND-ACC-04</text><text x="385" y="81" text-anchor="middle" class="t-small">nhận đồng thời</text>
<rect x="467" y="42" width="134" height="52" rx="8" class="f-pk-soft s-pk"/><text x="534" y="64" text-anchor="middle" class="t-mono t-b">TC-ACC-05</text><text x="534" y="81" text-anchor="middle" class="t-small">2 accept song song</text>
<rect x="616" y="42" width="134" height="52" rx="8" class="f-phys-soft s-phys"/><text x="683" y="64" text-anchor="middle" class="t-mono t-b t-phys">Passed</text><text x="683" y="81" text-anchor="middle" class="t-small">vitest, lần chạy 1</text>
<rect x="765" y="42" width="134" height="52" rx="8" class="f-surface s-line"/><text x="832" y="64" text-anchor="middle" class="t-mono t-b">—</text><text x="832" y="81" text-anchor="middle" class="t-small">không có defect</text>
<line x1="156" y1="68" x2="167" y2="68" class="s-muted" stroke-width="1.4" marker-start="url(#t007-tr)" marker-end="url(#t007-tr)"/>
<line x1="305" y1="68" x2="316" y2="68" class="s-muted" stroke-width="1.4" marker-start="url(#t007-tr)" marker-end="url(#t007-tr)"/>
<line x1="454" y1="68" x2="465" y2="68" class="s-muted" stroke-width="1.4" marker-start="url(#t007-tr)" marker-end="url(#t007-tr)"/>
<line x1="603" y1="68" x2="614" y2="68" class="s-muted" stroke-width="1.4" marker-start="url(#t007-tr)" marker-end="url(#t007-tr)"/>
<line x1="752" y1="68" x2="763" y2="68" class="s-muted" stroke-width="1.4" marker-start="url(#t007-tr)" marker-end="url(#t007-tr)"/>
<rect x="169" y="122" width="134" height="52" rx="8" class="f-pk-soft s-pk"/><text x="236" y="144" text-anchor="middle" class="t-mono t-b">R-ACCEPT-06</text><text x="236" y="161" text-anchor="middle" class="t-small">thứ tự danh sách chờ</text>
<rect x="318" y="122" width="581" height="52" rx="8" class="f-none s-warn dash" stroke-width="1.4"/>
<text x="608" y="144" text-anchor="middle" class="t-b t-warn">chưa có test condition nào được thiết kế thành test case</text>
<text x="608" y="161" text-anchor="middle" class="t-small">ma trận lộ khoảng trống ⇒ ghi vào test progress report, quyết định test thêm hay chấp nhận rủi ro</text>
<line x1="305" y1="148" x2="316" y2="148" class="s-warn dash" stroke-width="1.4"/>
</svg>
</div>
<figcaption><b>Hình 2 · Truy vết hai chiều.</b> Đi từ trái sang phải: rủi ro này đã được test chưa, kết quả ra sao. Đi từ phải sang trái: một defect hay một kết quả failed ảnh hưởng tới yêu cầu và rủi ro nào. Hàng dưới là khoảng trống mà chỉ đếm số test không bao giờ thấy.</figcaption>
</figure>

### 5.4. Test completion của sprint

Tóm tắt mà tester đưa vào báo cáo cuối sprint (dạng đầy đủ của test completion report: xem [TEST-010](/articles/testware-san-pham-kiem-thu)):

```text
Tính năng: Tài xế nhận chuyến (R-ACCEPT) — sprint 21 (giả định)
Đã chạy: 7/7 test case, 7 passed, 0 failed. Defect mới: 0.
Coverage yêu cầu: 6/7 mã R-ACCEPT có test passed; R-ACCEPT-03 phủ một nửa.
Rủi ro còn lại: RR-02 (một tài xế nhận hai chuyến) — CAO, chưa test;
                RR-06 (thứ tự danh sách chờ) — thấp, chưa test.
Đề xuất: không chặn release nếu thêm TC cho RR-02 trước ngày 9; RR-06 đưa vào backlog.
Testware giữ lại: nhan-chuyen.test.ts vào bộ regression chạy trên CI.
Bài học cho retrospective: đặt tên test theo mã TC + mã R giúp làm ma trận trong 10 phút.
```

## 6. Góc nhìn nâng cao

- **Quy trình không đồng nghĩa với giấy tờ dày.** Ở ví dụ trên, cả bảy hoạt động diễn ra trong vài ngày của một sprint; test plan chỉ là vài dòng trong sprint plan. Mức tài liệu là một yếu tố ngữ cảnh (FL-1.4.2), không phải tiêu chí đúng/sai của quy trình.
- **Lặp là bình thường, không phải thất bại.** Execution phát hiện test basis mơ hồ ⇒ quay lại analysis (Hình 1). Một quy trình ép đi một chiều sẽ buộc người ta «nhắm mắt» cho qua.
- **Test pass trên lab chưa chứng minh hệ đích.** TC-ACC-05 chạy trong một tiến trình: hai lời gọi xen kẽ nhau, không song song thật. Hệ đích có nhiều instance Dispatch, lease trên Redis và datastore riêng (kiến trúc §4.5). Test plan phải ghi rõ: ở lab ta xác minh **logic điều kiện** (giành chuyến có điều kiện); tính đúng dưới song song thật cần môi trường riêng (vd test trên PostgreSQL thật, nhiều kết nối) — chủ đề của Series 08 và 11.
- **Testability hooks là đầu vào của test design.** Đồng hồ tiêm vào, dữ liệu mẫu, điều phối tất định (kiến trúc §17.2, SPEC R-ARCH-03) là lý do ta viết được TC-ACC-06 «1 phút sau» mà không phải chờ 60 giây thật. Thiếu chúng, nhiều test condition rơi vào «không test được».
- **Sai lầm thường gặp:**
  - Đồng nhất pha STLC với hoạt động ISTQB rồi coi test analysis là «đọc yêu cầu cho biết» — bỏ qua bước xếp ưu tiên theo rủi ro.
  - Bỏ test completion vì «release xong rồi»: mất bài học, testware tốt không được giữ lại.
  - Nghĩ monitoring là báo cáo cuối kỳ. Monitoring diễn ra liên tục; báo cáo cuối kỳ là đầu ra của test completion.
  - Truy vết chỉ tới yêu cầu, không tới rủi ro — nên «100% yêu cầu có test» vẫn che mất rủi ro cao như RR-02.
- **Giữ truy vết rẻ.** Không nhất thiết cần công cụ quản lý test đắt tiền: quy ước đặt tên (mã TC + mã R trong tên test) cộng một bảng tính đã đủ cho đội nhỏ. Khi số yêu cầu lên hàng nghìn, công cụ mới đáng tiền.

## 7. Bài tập và câu hỏi ôn tập

### Bài tập

**Bài 1.** Lập bảng bảy hoạt động (đầu vào, nhiệm vụ chính, đầu ra) cho tính năng «đặt chuyến từ báo giá» (R-QUOTE-03, R-API-08, R-IDEM). Nêu ít nhất ba test condition có ưu tiên.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><b>Planning:</b> mục tiêu là xác minh thứ tự lỗi <code>QUOTE_NOT_FOUND → QUOTE_EXPIRED → QUOTE_ALREADY_USED → VOUCHER_USAGE_LIMIT</code> và chống tạo chuyến trùng; ngoài phạm vi: giới hạn tần suất (để đợt khác). Exit: mọi mã lỗi có test passed, không còn defect cao. <b>Analysis:</b> test condition ưu tiên cao: (1) mỗi báo giá đặt tối đa một chuyến kể cả hai yêu cầu đồng thời (R-QUOTE-03); (2) gửi lại cùng <code>Idempotency-Key</code> và cùng thân trả đúng kết quả cũ, không tạo chuyến mới (R-IDEM-02); (3) mốc hết hạn: đúng bằng <code>expiresAt</code> là đã hết hạn (R-QUOTE-01). Ưu tiên trung bình: báo giá của khách khác trả <code>QUOTE_NOT_FOUND</code> (chống IDOR). <b>Design:</b> test case với đồng hồ cố định: tạo báo giá 09:00, đặt lúc 09:04:59 (được) và 09:05:00 (<code>QUOTE_EXPIRED</code>). <b>Implementation:</b> test supertest qua API vì idempotency nằm ở tầng HTTP; dữ liệu C001/C002. <b>Execution:</b> log kèm mã TC + mã R. <b>Completion:</b> ma trận cho R-QUOTE-03, R-IDEM-02…05; khoảng trống ghi backlog. <b>Monitoring &amp; control:</b> theo dõi số mã lỗi đã phủ / tổng số mã lỗi.</p>
</details>

**Bài 2.** Cùng tính năng «tài xế nhận chuyến», nêu ba điểm khác trong quy trình test khi chuyển từ lab sang hệ đích microservices. Mỗi điểm gắn với một yếu tố ngữ cảnh của FL-1.4.2.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(1) <b>Yếu tố kỹ thuật</b>: có Kafka và offer TTL ⇒ thêm test condition cho offer hết hạn, sự kiện <code>DriverAssigned</code> trùng/sai thứ tự; test design phải có môi trường có broker. (2) <b>Ràng buộc dự án / đội</b>: nhiều đội sở hữu Dispatch, Trip, Availability ⇒ planning phải phân rõ đội nào test hợp đồng nào (contract test), monitoring gom kết quả từ nhiều pipeline. (3) <b>Domain nghiệp vụ</b>: ở production có 25.000 tài xế trực tuyến ⇒ mức coverage yêu cầu cho R-ACCEPT-03 cao hơn, cần test tải/đồng thời trên môi trường gần production (staging) chứ không chỉ một tiến trình.</p>
</details>

**Bài 3.** Ma trận ở mục 5.3 cho thấy RR-02 chưa có test. Viết test condition và một test case cho nó.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><b>TCOND-ACC-05</b> (R-ACCEPT-03, RR-02): một tài xế gửi đồng thời hai lệnh nhận cho hai chuyến khác nhau ⇒ đúng một thành công. <b>TC-ACC-08</b>: tiền điều kiện — hai chuyến xe máy SG đang <code>searching</code> (C001 và C002 mỗi người một chuyến), D101 <code>online</code>; đầu vào — D101 nhận chuyến 1 và chuyến 2 cùng lúc (<code>Promise.allSettled</code>), ETA 5; mong đợi — đúng một thành công, lệnh kia <code>DRIVER_BUSY</code>; chuyến còn lại vẫn <code>searching</code>, <code>driverId</code> rỗng; D101 <code>busy</code>. Cập nhật ma trận: R-ACCEPT-03 phủ đủ hai test condition.</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Trong dự án Tới Nơi, tester đang xác định cần những tài xế mẫu nào (online, offline, khác thành phố) và môi trường phải có đồng hồ điều khiển được. Đây chủ yếu là nhiệm vụ của hoạt động nào?

- a) Test planning
- b) Test analysis
- c) Test design
- d) Test execution

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng.</b> Test design gồm cả xác định yêu cầu test data và thiết kế môi trường test. a) sai: planning chọn mục tiêu và cách tiếp cận, không đi tới dữ liệu cụ thể. b) sai: analysis xác định «test cái gì» (test condition), chưa tới dữ liệu và môi trường. d) sai: execution dùng dữ liệu và môi trường đã có để chạy test. Lưu ý: <i>tạo</i> dữ liệu và <i>dựng</i> môi trường thuộc test implementation; <i>xác định yêu cầu</i> và <i>thiết kế</i> thuộc test design.</p>
</details>

**Câu 2 (K2).** Câu nào mô tả đúng test analysis theo CTFL v4.0.1?

- a) Biến test condition thành test case có đầu vào và kết quả mong đợi cụ thể
- b) Phân tích test basis để xác định và xếp ưu tiên test condition, đồng thời tìm defect trong chính test basis
- c) So kết quả thực tế với kết quả mong đợi và phân tích các bất thường
- d) Rút bài học và lưu trữ testware cho dự án sau

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng.</b> Analysis trả lời «test cái gì», xếp ưu tiên theo rủi ro, và đánh giá test basis (defect, testability). a) là test design. c) là test execution. d) là test completion.</p>
</details>

**Câu 3 (K1).** Việc nào sau đây KHÔNG thuộc test completion?

- a) Tạo change request hoặc backlog item cho defect chưa xử lý
- b) Đưa môi trường test về trạng thái đã thoả thuận
- c) Sắp xếp test procedure vào lịch thực thi
- d) Lập test completion report gửi các bên liên quan

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng</b> (là việc KHÔNG thuộc completion): xếp test procedure vào test execution schedule là nhiệm vụ của test implementation. a), b), d) đều là nhiệm vụ test completion được syllabus nêu.</p>
</details>

**Câu 4 (K2).** Hai đội cùng test tính năng thanh toán phí huỷ: đội A ở công ty khởi nghiệp, đội B làm cho một ví điện tử chịu thanh tra định kỳ. Theo FL-1.4.2, khác biệt nào là hợp lý nhất?

- a) Hai đội phải dùng đúng cùng một quy trình vì quy trình test là chuẩn chung
- b) Đội B có thể cần testware chi tiết hơn và truy vết chặt hơn do yếu tố domain và quy định pháp lý
- c) Đội A không cần test planning vì dự án nhỏ
- d) Đội B phải bỏ tự động hoá vì cơ quan thanh tra chỉ chấp nhận test thủ công

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng.</b> Domain nghiệp vụ (quy định pháp lý, mức quan trọng) ảnh hưởng tới độ chi tiết testware và mức coverage yêu cầu. a) sai: quy trình được may đo theo ngữ cảnh. c) sai: planning vẫn có, chỉ gọn hơn. d) sai: không có căn cứ; ngữ cảnh ảnh hưởng <i>mức</i> tự động hoá chứ không cấm nó.</p>
</details>

**Câu 5 (K2).** Báo cáo cuối sprint ghi «7/7 test passed». Thông tin bổ sung nào giúp đánh giá rủi ro tồn dư tốt nhất?

- a) Tổng thời gian chạy bộ test
- b) Truy vết từ kết quả test tới các rủi ro đã nhận diện
- c) Số dòng code của tính năng
- d) Tên người viết từng test case

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng.</b> Syllabus nêu truy vết kết quả test → rủi ro dùng để đánh giá mức rủi ro tồn dư; ở mục 5.3, nhờ nó ta thấy RR-02 mức cao chưa được test dù 7/7 passed. a), c), d) không cho biết rủi ro nào đã được giảm.</p>
</details>

**Câu 6 (K2).** Phát biểu nào đúng về các hoạt động của test process?

- a) Phải thực hiện tuần tự, xong hoạt động trước mới bắt đầu hoạt động sau
- b) Thường lặp lại hoặc song song, và được điều chỉnh theo hệ thống và dự án
- c) Test monitoring chỉ diễn ra một lần ở cuối dự án
- d) «STLC» là tên chính thức của test process trong CTFL v4.0.1

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng.</b> Syllabus nói các hoạt động có vẻ theo trình tự logic nhưng thường được thực hiện lặp hoặc song song, và cần may đo. a) sai vì lý do trên. c) sai: monitoring là kiểm tra liên tục mọi hoạt động. d) sai: thuật ngữ STLC không có trong syllabus CTFL v4.0.1.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — mục 1.4 (1.4.1 Test Activities and Tasks, 1.4.2 Test Process in Context, 1.4.4 Traceability between the Test Basis and Testware); phụ lục C (từ khoá «test process», «traceability» bổ sung ở v4.0.1). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — tra các thuật ngữ test process, test condition, traceability tại trang chủ https://www.istqb.org/
- ISO/IEC/IEEE 29119-2:2021 *Software and systems engineering — Software testing — Part 2: Test processes* (syllabus dẫn chiếu cho mô tả quy trình chi tiết).
- ISO/IEC/IEEE 29119-3:2021 *Part 3: Test documentation* — mẫu testware và báo cáo.
- Phòng lab: `SPEC.md` (R-ACCEPT, R-ARCH) và bộ test của bài — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-001](/articles/software-testing-la-gi-testing-khac-debugging) · [TEST-006](/articles/muc-tieu-kiem-thu-va-quality-assurance) · [TEST-009 Vai trò trong kiểm thử](/articles/vai-tro-trong-kiem-thu) · [TEST-010 Testware](/articles/testware-san-pham-kiem-thu)

Bài tiếp theo: [TEST-008 · Kiểm thử trong SDLC: Waterfall, Agile và DevOps](/articles/kiem-thu-trong-sdlc-agile-waterfall).
