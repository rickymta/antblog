---
slug: testware-san-pham-kiem-thu
title: "Testware và các sản phẩm kiểm thử: test plan, test case, test data, defect report"
summary: "Mỗi hoạt động kiểm thử theo CTFL v4 sinh ra testware gì, và vì sao phải truy vết chúng về yêu cầu. Bộ testware mẫu cho phụ phí bưu kiện Tới Nơi: test case, test data, test script chạy thật, defect report."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-010, testware, traceability, test-case, defect-report]
language: vi
---

Kiểm thử không chỉ là «bấm thử xem có lỗi không». Mỗi hoạt động kiểm thử để lại sản phẩm: kế hoạch, điều kiện test, test case, dữ liệu, script, nhật ký chạy, báo cáo lỗi. ISTQB gọi chung là **testware**. Bài này trả lời hai câu: hoạt động nào sinh ra testware nào, và vì sao phải nối chúng thành một chuỗi **truy vết** (traceability) về tận yêu cầu — minh hoạ bằng một bộ testware hoàn chỉnh, chạy được, cho phụ phí khối lượng bưu kiện của Tới Nơi.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-010</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S01 · Fundamentals of Software Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/quy-trinh-kiem-thu-test-process">TEST-007 · Quy trình kiểm thử</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Bộ testware tối thiểu cho giao bưu kiện (SPEC R-PARCEL) + test vitest trong lab: <code>articles/testware-san-pham-kiem-thu</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.4.3, 1.4.4 · FL-1.4.3 (K2), FL-1.4.4 (K2). Liên quan: mục 5.1.1 (test plan), 5.4 (configuration management), 5.5 · FL-5.5.1 (K3) (defect report)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Hãy hỏi đội Tới Nơi ba câu ngay trước giờ phát hành tính năng giao bưu kiện:

1. «Quy tắc phụ phí khối lượng R-PARCEL-03 đã được test chưa, bằng dữ liệu nào?»
2. «Test đỏ hôm qua ảnh hưởng yêu cầu nào, rủi ro nào?»
3. «Nếu đổi phụ phí từ 2.000 đ lên 2.500 đ mỗi kg, phải sửa những test nào?»

Nếu test chỉ nằm trong đầu tester, không câu nào trả lời được. Testware **biến công việc kiểm thử thành sản phẩm có tên, có phiên bản, có liên kết** để người khác đọc lại, chạy lại, kiểm toán và đo độ phủ. Mức chi tiết tuỳ ngữ cảnh, nhưng chuỗi «yêu cầu → test → kết quả → defect» thì dự án nào cũng cần.

## 2. Lý thuyết theo ISTQB

### 2.1. Testware là gì (mục 1.4.3)

Testware là các **work product đầu ra của các hoạt động kiểm thử**. Syllabus nói rõ hai điều: mỗi tổ chức tạo, đặt tên, sắp xếp và quản lý chúng rất khác nhau; và **configuration management** (quản lý cấu hình, mục 5.4) giữ cho chúng nhất quán và toàn vẹn — mỗi mục có định danh duy nhất, có phiên bản, theo dõi thay đổi. Danh sách dưới đây không đầy đủ, chỉ là các loại thường gặp:

| Hoạt động (xem [TEST-007](/articles/quy-trinh-kiem-thu-test-process)) | Testware tiêu biểu | Ví dụ trong bài |
|---|---|---|
| Test planning | test plan, test schedule, risk register, entry/exit criteria (ba thứ sau thường nằm trong test plan) | trích test plan mục 5.1 |
| Test monitoring & control | test progress report, chỉ thị điều khiển đã ghi lại, thông tin rủi ro | dòng tiến độ mục 5.6 |
| Test analysis | test condition (đã ưu tiên, vd acceptance criteria), defect report về defect **trong test basis** (nếu không sửa ngay) | TCOND-PARCEL-03a…04a |
| Test design | test case (đã ưu tiên), test charter, coverage item, yêu cầu về test data, yêu cầu về môi trường | TC-PARCEL-002 |
| Test implementation | test procedure, test script thủ công và tự động, test suite, test data, test execution schedule, thành phần môi trường (stub, driver, simulator, service virtualization) | TD-PARCEL-01, `parcel-surcharge.test.ts` |
| Test execution | test log, defect report | log vitest, DEF-0042 |
| Test completion | test completion report, việc cần cải tiến, lessons learned, change request (vd product backlog item) | dòng kết thúc mục 5.6 |

Risk register, theo syllabus, là danh sách rủi ro kèm khả năng xảy ra, mức tác động và cách giảm thiểu.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Bảy hoạt động kiểm thử và testware tiêu biểu của mỗi hoạt động theo CTFL 1.4.3.">
<rect x="10" y="20" width="120" height="44" rx="8" class="f-pk-soft s-pk"/><text x="70" y="47" text-anchor="middle" class="t-b t-pk">Planning</text>
<rect x="140" y="20" width="120" height="44" rx="8" class="f-pk-soft s-pk"/><text x="200" y="39" text-anchor="middle" class="t-b t-pk">Monitoring</text><text x="200" y="55" text-anchor="middle" class="t-b t-pk">&amp; control</text>
<rect x="270" y="20" width="120" height="44" rx="8" class="f-phys-soft s-phys"/><text x="330" y="47" text-anchor="middle" class="t-b t-phys">Analysis</text>
<rect x="400" y="20" width="120" height="44" rx="8" class="f-phys-soft s-phys"/><text x="460" y="47" text-anchor="middle" class="t-b t-phys">Design</text>
<rect x="530" y="20" width="120" height="44" rx="8" class="f-phys-soft s-phys"/><text x="590" y="47" text-anchor="middle" class="t-b t-phys">Implementation</text>
<rect x="660" y="20" width="120" height="44" rx="8" class="f-phys-soft s-phys"/><text x="720" y="47" text-anchor="middle" class="t-b t-phys">Execution</text>
<rect x="790" y="20" width="120" height="44" rx="8" class="f-pk-soft s-pk"/><text x="850" y="47" text-anchor="middle" class="t-b t-pk">Completion</text>
<rect x="10" y="74" width="120" height="176" rx="6" class="f-surface s-line"/>
<rect x="140" y="74" width="120" height="176" rx="6" class="f-surface s-line"/>
<rect x="270" y="74" width="120" height="176" rx="6" class="f-surface s-line"/>
<rect x="400" y="74" width="120" height="176" rx="6" class="f-surface s-line"/>
<rect x="530" y="74" width="120" height="176" rx="6" class="f-surface s-line"/>
<rect x="660" y="74" width="120" height="176" rx="6" class="f-surface s-line"/>
<rect x="790" y="74" width="120" height="176" rx="6" class="f-surface s-line"/>
<text x="70" y="96" text-anchor="middle" class="t-10">test plan</text><text x="70" y="116" text-anchor="middle" class="t-10">test schedule</text><text x="70" y="136" text-anchor="middle" class="t-10">risk register</text><text x="70" y="156" text-anchor="middle" class="t-10">entry/exit criteria</text>
<text x="200" y="96" text-anchor="middle" class="t-10">progress report</text><text x="200" y="116" text-anchor="middle" class="t-10">control directives</text><text x="200" y="136" text-anchor="middle" class="t-10">thông tin rủi ro</text>
<text x="330" y="96" text-anchor="middle" class="t-10">test conditions</text><text x="330" y="116" text-anchor="middle" class="t-10">acceptance criteria</text><text x="330" y="136" text-anchor="middle" class="t-10">defect report</text><text x="330" y="152" text-anchor="middle" class="t-10">của test basis</text>
<text x="460" y="96" text-anchor="middle" class="t-10">test cases</text><text x="460" y="116" text-anchor="middle" class="t-10">test charters</text><text x="460" y="136" text-anchor="middle" class="t-10">coverage items</text><text x="460" y="156" text-anchor="middle" class="t-10">yêu cầu test data</text><text x="460" y="176" text-anchor="middle" class="t-10">yêu cầu môi trường</text>
<text x="590" y="96" text-anchor="middle" class="t-10">test procedures</text><text x="590" y="116" text-anchor="middle" class="t-10">test scripts</text><text x="590" y="136" text-anchor="middle" class="t-10">test suites</text><text x="590" y="156" text-anchor="middle" class="t-10">test data</text><text x="590" y="176" text-anchor="middle" class="t-10">execution schedule</text><text x="590" y="196" text-anchor="middle" class="t-10">stub, driver,</text><text x="590" y="212" text-anchor="middle" class="t-10">simulator…</text>
<text x="720" y="96" text-anchor="middle" class="t-10">test logs</text><text x="720" y="116" text-anchor="middle" class="t-10">defect reports</text>
<text x="850" y="96" text-anchor="middle" class="t-10">completion report</text><text x="850" y="116" text-anchor="middle" class="t-10">lessons learned</text><text x="850" y="136" text-anchor="middle" class="t-10">action items</text><text x="850" y="156" text-anchor="middle" class="t-10">change requests</text>
<text x="460" y="280" text-anchor="middle" class="t-small">hồng: hoạt động trọng tâm của test management role · xanh: của testing role (xem TEST-009)</text>
</svg>
</div>
<figcaption><b>Hình 1 · Mỗi hoạt động một nhóm testware (CTFL 1.4.3).</b> Danh sách không đầy đủ; tên gọi và cách tổ chức khác nhau giữa các công ty. Các hoạt động thường lặp hoặc chạy song song, nên testware cũng được cập nhật nhiều vòng.</figcaption>
</figure>

Ba cặp hay nhầm:

- **Test condition** (điều cần kiểm — «cái gì») ≠ **test case** (đầu vào, tiền điều kiện, kết quả mong đợi cụ thể — «kiểm thế nào»).
- **Test case** ≠ **test procedure** (chuỗi bước thực hiện một hoặc nhiều test case) ≠ **test script** (procedure viết dưới dạng chạy được, thủ công hoặc tự động). Nhiều procedure gom thành **test suite**.
- **Test log** (ghi lại chuyện gì đã xảy ra khi chạy) ≠ **defect report** (một bất thường cần xử lý, kèm đủ thông tin để tái hiện).

### 2.2. Truy vết giữa test basis và testware (mục 1.4.4)

Để theo dõi và điều khiển kiểm thử hiệu quả, cần lập và **duy trì** truy vết suốt quy trình giữa: các phần tử của test basis (yêu cầu), testware gắn với chúng (test condition, rủi ro, test case), kết quả test và defect. Syllabus nêu các giá trị:

| Truy vết | Trả lời câu hỏi |
|---|---|
| test case ↔ yêu cầu | yêu cầu nào đã được test case phủ — đo **coverage**; tiêu chí phủ đo được có thể làm KPI cho mức đạt test objective |
| kết quả test ↔ rủi ro | còn bao nhiêu **rủi ro tồn đọng** (residual risk) trong test object |
| toàn chuỗi | đổi một yêu cầu thì **ảnh hưởng** những test nào; phục vụ **audit** và tiêu chí quản trị CNTT |
| trạng thái theo yêu cầu | báo cáo tiến độ và báo cáo kết thúc **dễ hiểu** với stakeholder không chuyên kỹ thuật |

Tóm lại, truy vết cho thông tin để đánh giá chất lượng sản phẩm, năng lực quy trình và tiến độ dự án so với mục tiêu kinh doanh. Truy vết phải **hai chiều**: xuôi (yêu cầu → test, «đã phủ chưa?») và ngược (test/defect → yêu cầu, «cái đỏ này thuộc về đâu?»).

### 2.3. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa trong bài |
|---|---|
| Testware | sản phẩm đầu ra của các hoạt động kiểm thử |
| Test basis | nguồn dùng làm cơ sở cho phân tích và thiết kế test (ở đây: SPEC.md) |
| Test condition | khía cạnh kiểm thử được của một thành phần/hệ thống |
| Test case | bộ tiền điều kiện, đầu vào, hành động, kết quả mong đợi và hậu điều kiện |
| Test procedure / test script | chuỗi bước thực hiện test / chuỗi bước đó ở dạng chạy được |
| Test data | dữ liệu cần để thực thi test |
| Test log | nhật ký thực thi test |
| Defect report | báo cáo về một defect (ISO 29119-3 gọi là incident report) |
| Traceability | truy vết: quan hệ giữa hai hay nhiều work product |
| Configuration management | quản lý cấu hình: định danh, phiên bản, kiểm soát thay đổi |
| Residual risk | rủi ro tồn đọng sau kiểm thử |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>«Ma trận truy vết yêu cầu» (RTM — Requirements Traceability Matrix), mẫu test case trong công cụ quản lý test, quy ước mã kiểu <code>TC-PARCEL-002</code> và việc kiểm truy vết bằng một test tự động (mục 5.5) là thực hành ngành. CTFL chỉ yêu cầu truy vết được duy trì; hình thức là lựa chọn của đội. Mẫu tài liệu chi tiết có trong ISO/IEC/IEEE 29119-3.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu và business rules

Tính năng **giao bưu kiện** (`parcel`) chiếm khoảng 0,3 triệu chuyến mỗi ngày theo [hồ sơ dự án](/articles/du-an-mau-toi-noi). Test basis là nhóm R-PARCEL của SPEC lab:

- **R-PARCEL-01** — khối lượng làm tròn 0,1 kg half-up rồi phải thoả 0 < w ≤ 30 kg, sai ⇒ `WEIGHT_OUT_OF_RANGE`.
- **R-PARCEL-02** — w > 20 kg bắt buộc ô tô; chọn xe máy ⇒ `VEHICLE_TOO_SMALL`; 20,0 kg vẫn đi xe máy.
- **R-PARCEL-03** — phần vượt 5 kg tính **2.000 đ cho mỗi kg bắt đầu**: 5,1 ⇒ 2.000 đ; 6,0 ⇒ 2.000 đ; 6,1 ⇒ 4.000 đ; 30 ⇒ 50.000 đ.
- **R-PARCEL-04** — phụ phí cộng vào cước cơ bản (bước 1 của R-FARE-03) nên **bị nhân** hệ số cao điểm 1,25.

### 3.2. System context

Kiến trúc đích: Customer App → API Gateway/Customer BFF → **Pricing & Quote** (quy tắc khối lượng từ **Service Catalog**; quote có `quote_id`, `expires_at`, `pricing_version`) → **Booking** lưu ảnh chụp giá (R-QUOTE-04) → Payment thu đúng `fare`. Lab gộp thành một modular monolith; endpoint kiểm được là `POST /api/quotes`. Giá phụ thuộc phiên bản quy tắc (`fare-2026.1`) nên testware phải ghi rõ **phiên bản test basis** — chỗ configuration management chạm tới kiểm thử.

### 3.3. Rủi ro

| Rủi ro sản phẩm | Hậu quả | Testware giảm rủi ro |
|---|---|---|
| tính sai «kg bắt đầu» ở biên (6,0 kg) | thu thừa tiền khách, khiếu nại, hoàn tiền | test case biên + test data có khối lượng nguyên |
| quên nhân phụ phí với cao điểm, hoặc nhân hai lần | lệch giá giữa báo giá và thu tiền | test case R-PARCEL-04 |
| làm tròn khối lượng sau khi tính phí | 5,05 kg không bị tính phụ phí | test data 5,04 / 5,05 |
| đổi phụ phí mà quên sửa test | test xanh nhưng sai quy tắc mới | truy vết xuôi/ngược + phiên bản test basis |

Ước tính tác động của rủi ro đầu tiên (**giả định**): nếu 20% đơn bưu kiện có khối lượng là số kg nguyên trên 5 kg, mỗi đơn bị thu thừa 2.000 đ ⇒ 300.000 × 20% × 2.000 đ = **120 triệu đ mỗi ngày** — chưa tính hệ số cao điểm và chi phí xử lý khiếu nại.

## 4. Phân tích kỹ thuật

### 4.1. Từ yêu cầu tới test condition (test analysis)

| Mã | Test condition | Nguồn | Ưu tiên |
|---|---|---|---|
| TCOND-PARCEL-03a | không vượt 5 kg thì không có phụ phí | R-PARCEL-03 | Trung bình |
| TCOND-PARCEL-03b | phần vượt chưa trọn kg được tính trọn một kg | R-PARCEL-03 | Cao |
| TCOND-PARCEL-03c | phần vượt đúng số kg nguyên **không** bị tính thừa kg | R-PARCEL-03 | Cao |
| TCOND-PARCEL-03d | khối lượng làm tròn 0,1 kg **trước** khi tính phụ phí | R-PARCEL-01, -03 | Trung bình |
| TCOND-PARCEL-02a | biên 20 kg của xe máy | R-PARCEL-02 | Cao |
| TCOND-PARCEL-01a | biên 30 kg tối đa | R-PARCEL-01 | Trung bình |
| TCOND-PARCEL-04a | phụ phí bị nhân hệ số cao điểm | R-PARCEL-04 | Cao |

Ưu tiên «Cao» gắn với rủi ro tiền ở mục 3.3. Chọn giá trị cụ thể ở biên là việc của test design — kỹ thuật phân tích giá trị biên sẽ học ở Series 04; ở đây chỉ dùng kết quả.

### 4.2. Từ test condition tới test case và test data (test design, implementation)

| Test case | Phủ test condition | Dòng test data |
|---|---|---|
| TC-PARCEL-001 · không phụ phí tới 5 kg | 03a, 03d | TD-01, TD-02 |
| TC-PARCEL-002 · phụ phí theo kg bắt đầu | 03b, 03c, 03d | TD-03, TD-04, TD-05 |
| TC-PARCEL-003 · biên 20 kg xe máy | 02a, 03c | TD-06, TD-07 |
| TC-PARCEL-004 · biên 30 kg | 01a, 03c | TD-08, TD-09 |
| TC-PARCEL-005 · phụ phí và cao điểm | 04a | TD-10 |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 190" role="img" aria-label="Chuỗi truy vết từ yêu cầu R-PARCEL-03 qua test condition, test case, test data, test log tới defect DEF-0042, có chiều xuôi và chiều ngược.">
<defs>
<marker id="t010-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="60" width="130" height="60" rx="8" class="f-pk-soft s-pk"/><text x="85" y="84" text-anchor="middle" class="t-mono t-b">R-PARCEL-03</text><text x="85" y="102" text-anchor="middle" class="t-small">test basis</text>
<rect x="170" y="60" width="130" height="60" rx="8" class="f-phys-soft s-phys"/><text x="235" y="84" text-anchor="middle" class="t-mono t-b">TCOND-03c</text><text x="235" y="102" text-anchor="middle" class="t-small">test condition</text>
<rect x="320" y="60" width="130" height="60" rx="8" class="f-phys-soft s-phys"/><text x="385" y="84" text-anchor="middle" class="t-mono t-b">TC-PARCEL-002</text><text x="385" y="102" text-anchor="middle" class="t-small">test case</text>
<rect x="470" y="60" width="130" height="60" rx="8" class="f-phys-soft s-phys"/><text x="535" y="84" text-anchor="middle" class="t-mono t-b">TD-04 · 6,0 kg</text><text x="535" y="102" text-anchor="middle" class="t-small">test data</text>
<rect x="620" y="60" width="130" height="60" rx="8" class="f-surface s-line"/><text x="685" y="84" text-anchor="middle" class="t-mono t-b">FAIL</text><text x="685" y="102" text-anchor="middle" class="t-small">test log</text>
<rect x="770" y="60" width="130" height="60" rx="8" class="f-warn-soft s-warn"/><text x="835" y="84" text-anchor="middle" class="t-mono t-b t-warn">DEF-0042</text><text x="835" y="102" text-anchor="middle" class="t-small">defect report</text>
<line x1="150" y1="90" x2="168" y2="90" class="s-muted" stroke-width="1.4" marker-start="url(#t010-arrow)" marker-end="url(#t010-arrow)"/>
<line x1="300" y1="90" x2="318" y2="90" class="s-muted" stroke-width="1.4" marker-start="url(#t010-arrow)" marker-end="url(#t010-arrow)"/>
<line x1="450" y1="90" x2="468" y2="90" class="s-muted" stroke-width="1.4" marker-start="url(#t010-arrow)" marker-end="url(#t010-arrow)"/>
<line x1="600" y1="90" x2="618" y2="90" class="s-muted" stroke-width="1.4" marker-start="url(#t010-arrow)" marker-end="url(#t010-arrow)"/>
<line x1="750" y1="90" x2="768" y2="90" class="s-muted" stroke-width="1.4" marker-start="url(#t010-arrow)" marker-end="url(#t010-arrow)"/>
<line x1="40" y1="30" x2="880" y2="30" class="s-phys" stroke-width="1.4" marker-end="url(#t010-arrow)"/>
<text x="460" y="22" text-anchor="middle" class="t-small">xuôi: yêu cầu này đã được phủ chưa, kết quả ra sao?</text>
<line x1="880" y1="150" x2="40" y2="150" class="s-warn" stroke-width="1.4" marker-end="url(#t010-arrow)"/>
<text x="460" y="172" text-anchor="middle" class="t-small">ngược: defect này vi phạm yêu cầu nào, rủi ro nào còn tồn đọng?</text>
</svg>
</div>
<figcaption><b>Hình 2 · Một chuỗi truy vết hoàn chỉnh.</b> Mỗi mắt xích có mã riêng và trỏ được sang mắt xích kề bên theo cả hai chiều. Thiếu một mắt xích, câu hỏi tương ứng ở mục 1 không trả lời được.</figcaption>
</figure>

## 5. Thực hành

Mọi tệp chạy được nằm trong lab: [`articles/testware-san-pham-kiem-thu`](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/testware-san-pham-kiem-thu). Để có một defect thật mà **không sửa `src/`**, thư mục bài có `buggy-parcel.ts` — một bản cài đặt hàm phụ phí cố ý sai, đóng vai «bản dựng rc» mà developer gửi sang test.

### 5.1. Trích test plan (test planning)

| Mục | Nội dung |
|---|---|
| Phạm vi | trong: R-PARCEL-01…04 qua `POST /api/quotes` và hàm miền; ngoài: đặt chuyến, thanh toán, giao diện, voucher cho bưu kiện |
| Mục tiêu | phát hiện defect tính phụ phí trước khi phát hành; xác nhận báo giá bưu kiện đúng SPEC |
| Test basis | SPEC v1.1, nhóm R-PARCEL, R-FARE-03; `pricingVersion = fare-2026.1` |
| Risk register | RR-1 thu thừa ở biên kg — khả năng TB, tác động Cao, giảm bằng TC-PARCEL-002…004 · RR-2 sai cao điểm — khả năng Thấp, tác động Cao, giảm bằng TC-PARCEL-005 |
| Cách tiếp cận | component test tự động trên hàm miền (developer + QA), một test qua API (QA Engineer); dữ liệu biên tách khỏi script |
| Entry criteria | bản dựng chạy được, `npm run typecheck` sạch, SPEC đã chốt |
| Exit criteria | 100% test condition ưu tiên Cao có test case đạt; không còn defect mức nghiêm trọng Cao đang mở |

### 5.2. Test case mẫu (test design)

| Trường | TC-PARCEL-002 |
|---|---|
| Tiêu đề | Phụ phí bưu kiện tính theo **mỗi kg bắt đầu** vượt 5 kg |
| Truy vết | R-PARCEL-03, R-PARCEL-01 · TCOND-PARCEL-03b, 03c, 03d |
| Ưu tiên | Cao (RR-1) |
| Tiền điều kiện | máy chủ lab chạy với `NODE_ENV=test`; đã gọi `POST /api/test/reset`; đồng hồ `2026-11-04T09:00:00+07:00` (ngoài cao điểm); SG không mưa |
| Dữ liệu | TD-03, TD-04, TD-05 của bộ TD-PARCEL-01 |
| Bước | 1) `POST /api/quotes`, header `x-user: customer:C001`, thân `{"kind":"parcel","cityCode":"SG","vehicle":"bike","distanceKm":3,"parcelWeightKg":<w>}` với w lấy từ dòng dữ liệu. 2) Đọc `quote.parcelWeightKg`, `quote.fare.parcelSurcharge`, `quote.total`. |
| Kết quả mong đợi | HTTP 200; `parcelWeightKg` = khối lượng đã làm tròn (5,1 / 6 / 6,1); `fare.parcelSurcharge` và `total` đúng cột mong đợi của dòng dữ liệu |
| Hậu điều kiện | báo giá được lưu, không tiêu lượt voucher (R-API-07) — không cần dọn |

Kết quả mong đợi lấy từ **test basis** (SPEC), không lấy từ việc chạy thử hệ thống rồi chép lại.

### 5.3. Test data (test implementation)

Bộ `TD-PARCEL-01` (tệp `test-data/td-parcel-01.json`). Chung cho mọi dòng: SG, 3 km, không mưa, không voucher; cước cơ bản xe máy = 12.000 + 4.500 × 3 = 25.500 đ, ô tô = 25.000 + 11.000 × 3 = 58.000 đ.

| Dòng | Test case | Yêu cầu | w (kg) | Xe | Cao điểm | Phụ phí mong đợi | Tổng mong đợi |
|---|---|---|---:|---|---|---:|---:|
| TD-01 | TC-001 | R-PARCEL-03 | 5,0 | bike | không | 0 | 26.000 |
| TD-02 | TC-001 | -01, -03 | 5,04 (⇒ 5,0) | bike | không | 0 | 26.000 |
| TD-03 | TC-002 | -01, -03 | 5,05 (⇒ 5,1) | bike | không | 2.000 | 28.000 |
| TD-04 | TC-002 | -03 | 6,0 | bike | không | 2.000 | 28.000 |
| TD-05 | TC-002 | -03 | 6,1 | bike | không | 4.000 | 30.000 |
| TD-06 | TC-003 | -02, -03 | 20,0 | bike | không | 30.000 | 56.000 |
| TD-07 | TC-003 | -02 | 20,1 | bike | không | — | `VEHICLE_TOO_SMALL` |
| TD-08 | TC-004 | -01, -03 | 30,0 | car | không | 50.000 | 108.000 |
| TD-09 | TC-004 | -01 | 30,05 (⇒ 30,1) | car | không | — | `WEIGHT_OUT_OF_RANGE` |
| TD-10 | TC-005 | -04 | 6,1 | bike | có | 4.000 | 37.000 |

Kiểm một dòng bằng tay: TD-10 = (25.500 + 4.000) × 1,25 = 36.875 ⇒ làm tròn lên 1.000 đ ⇒ **37.000 đ**. Dữ liệu là **giả lập hoàn toàn** — không dùng thông tin khách thật (kiến trúc §16.2).

### 5.4. Test script tự động và test log (implementation, execution)

Script đọc dữ liệu từ JSON nên thêm một dòng dữ liệu không phải sửa code:

```ts
import data from './test-data/td-parcel-01.json';

const useBuggy = process.env.TOINOI_PARCEL_IMPL === 'buggy';
const surchargeOf = useBuggy ? buggyParcelSurcharge : parcelSurcharge;

it.each(okRows)('$testCase / $id: $weightKg kg, $vehicle, cao điểm=$peak', (row) => {
  expect(quoteParcel(row)).toEqual(row.expected); // { surcharge, total } từ test data
});
```

Chạy trên mã thật của lab:

```bash
npx vitest run articles/testware-san-pham-kiem-thu
```

```text
 ✓ articles/testware-san-pham-kiem-thu/traceability.test.ts (2 tests)
 ✓ articles/testware-san-pham-kiem-thu/parcel-surcharge.test.ts (11 tests)
 Test Files  2 passed (2)
      Tests  13 passed (13)
```

Chạy cùng script trên «bản dựng rc» (bản lỗi):

```bash
TOINOI_PARCEL_IMPL=buggy npx vitest run articles/testware-san-pham-kiem-thu/parcel-surcharge.test.ts
```

```text
 × 'TC-PARCEL-002' / 'TD-04': 6 kg, 'bike', cao điểm=false
   → expected { surcharge: 4000, total: 30000 } to deeply equal { surcharge: 2000, total: 28000 }
 × 'TC-PARCEL-003' / 'TD-06': 20 kg, 'bike', cao điểm=false
   → expected { surcharge: 32000, total: 58000 } to deeply equal { surcharge: 30000, total: 56000 }
 × 'TC-PARCEL-004' / 'TD-08': 30 kg, 'car', cao điểm=false
   → expected { surcharge: 52000, total: 110000 } to deeply equal { surcharge: 50000, total: 108000 }
      Tests  3 failed | 7 passed | 1 skipped (11)
```

(Trích từ lần chạy thật, đã lược tiền tố đường dẫn. Test qua API bị bỏ qua ở chế độ này vì nó luôn dùng mã thật.) Đây chính là **test log**: bằng chứng gắn vào defect report bên dưới.

### 5.5. Defect report (test execution)

Đủ các trường mà CTFL mục 5.5 liệt kê cho một defect report trong dynamic testing:

| Trường | DEF-0042 |
|---|---|
| Mã | DEF-0042 |
| Tiêu đề | Phụ phí bưu kiện thừa 2.000 đ khi khối lượng là số kg nguyên trên 5 kg (6,0 kg ⇒ 4.000 đ) |
| Ngày · tổ chức · người báo | 05/11/2026 · squad Pricing & Promotion · Lan (QA Engineer, giả định) |
| Test object · môi trường | hàm phụ phí bưu kiện, bản dựng rc (`buggy-parcel.ts`); lab toi-noi-testing, Node 22, vitest 3.2.7, kho bộ nhớ, `pricingVersion = fare-2026.1`, đồng hồ cố định 09:00 giờ VN |
| Ngữ cảnh | test execution chu kỳ 1; TC-PARCEL-002 với TD-04 (cũng đỏ: TC-PARCEL-003/TD-06, TC-PARCEL-004/TD-08); dữ liệu giá trị biên |
| Bước tái hiện | 1) Ở thư mục lab, đặt `TOINOI_PARCEL_IMPL=buggy`. 2) Chạy `npx vitest run articles/testware-san-pham-kiem-thu/parcel-surcharge.test.ts`. 3) Xem kết quả dòng TD-04. |
| Kết quả mong đợi | 6,0 kg ⇒ phụ phí 2.000 đ, tổng 28.000 đ (R-PARCEL-03: «6,0 ⇒ 2.000 đ») |
| Kết quả thực tế | 6,0 kg ⇒ phụ phí 4.000 đ, tổng 30.000 đ. Tương tự 20,0 kg ⇒ 32.000 đ (mong đợi 30.000 đ); 30,0 kg ⇒ 52.000 đ (mong đợi 50.000 đ). 5,1 kg và 6,1 kg đúng |
| Mức nghiêm trọng | Cao — khách bị thu thừa tiền ở mọi khối lượng nguyên trên 5 kg, không có cách né |
| Ưu tiên sửa | P1 — chặn phát hành (exit criteria) |
| Trạng thái | Open |
| Tham chiếu | R-PARCEL-03 · TCOND-PARCEL-03c · TC-PARCEL-002/003/004 · TD-PARCEL-01 · test log mục 5.4 |

Tiêu đề nói **triệu chứng và điều kiện**, không đoán nguyên nhân. Phần «chỉ sai ở khối lượng nguyên» là quan sát giúp developer khoanh vùng nhanh — chuỗi error → defect → failure của chính lỗi này (đếm «kg bắt đầu» bằng `floor(w − 5) + 1`) là đề tài của [TEST-004](/articles/error-defect-failure-root-cause). Sau khi sửa, trạng thái chuyển sang chờ **confirmation testing**: chạy lại đúng TD-04, TD-06, TD-08, rồi chạy lại cả bộ để kiểm hồi quy.

### 5.6. Ma trận truy vết, tiến độ và kết thúc

Tệp `traceability.test.ts` đọc `SPEC.md`, lấy mọi mã `R-PARCEL-xx` và kiểm hai chiều: mọi yêu cầu có test case (xuôi), mọi mã testware trích dẫn đều có trong SPEC (ngược). Ma trận mà test khẳng định:

| Yêu cầu | Test case | Trạng thái (bản rc) |
|---|---|---|
| R-PARCEL-01 | TC-PARCEL-001, 002, 004 | TC-004 đỏ (DEF-0042) |
| R-PARCEL-02 | TC-PARCEL-003 | TC-003 đỏ ở TD-06 (DEF-0042) |
| R-PARCEL-03 | TC-PARCEL-001, 002, 003, 004 | đỏ — DEF-0042 |
| R-PARCEL-04 | TC-PARCEL-005 | đạt |

Đọc ma trận theo chiều rủi ro: RR-1 (thu thừa ở biên kg) **chưa giảm** vì DEF-0042 còn mở; RR-2 (cao điểm) đã có bằng chứng đạt. Đó là thông tin cho **test progress report**: «4/4 yêu cầu có test, 7/10 dòng dữ liệu đạt, 1 defect Cao chặn phát hành». Khi DEF-0042 đóng và cả bộ xanh, **test completion report** ghi kết quả cuối, lưu bộ TD-PARCEL-01 và script để dùng lại ở đợt sau, và một lessons learned: «thêm khối lượng nguyên vào mọi bộ dữ liệu phụ phí theo đơn vị».

## 6. Góc nhìn nâng cao

| Sai lầm thường gặp | Cách làm đúng |
|---|---|
| test case không ghi phiên bản test basis ⇒ đổi `pricingVersion` không biết test nào lỗi thời | ghi phiên bản SPEC; đưa testware vào configuration management |
| kết quả mong đợi chép từ hệ thống đang chạy ⇒ lỗi hiện có thành «đúng» | suy kết quả mong đợi từ test basis, có công thức |
| truy vết một chiều | duy trì cả xuôi và ngược, cập nhật mỗi khi đổi |
| dữ liệu nhúng cứng trong script | tách test data thành tệp riêng có mã dòng |
| coi testware là tài liệu viết một lần | bảo trì script và dữ liệu như code; lưu trữ lúc test completion |
| tiêu đề defect report đoán nguyên nhân | tiêu đề = triệu chứng + điều kiện |

Hai điểm dễ bỏ sót:

- **Defect report không chỉ đến từ execution.** Đọc SPEC trong test analysis thấy mâu thuẫn cũng là một defect report — về test basis.
- **Bất thường chưa chắc là defect.** Test đỏ có thể là false positive (dữ liệu, môi trường sai) hoặc change request; quy trình defect management mới quyết định. Vì thế test data cũng phải được review. Mức chi tiết của testware thì tuỳ ngữ cảnh: đội Agile có thể dùng acceptance criteria và test charter, nhưng quy tắc tiền vẫn nên có dữ liệu biên tường minh.

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1.** Viết test case TC-PARCEL-006 cho R-PARCEL-02 với ba dòng dữ liệu: 20,0 kg xe máy, 20,1 kg xe máy, 20,1 kg ô tô (SG, 3 km, 09:00, không mưa). Ghi truy vết và kết quả mong đợi có công thức.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Truy vết: R-PARCEL-02, R-PARCEL-03 · TCOND-PARCEL-02a. Dữ liệu và mong đợi: (1) 20,0 kg bike ⇒ hợp lệ, phụ phí 2.000 × ⌈15⌉ = 30.000 đ, tổng 25.500 + 30.000 = 55.500 ⇒ <b>56.000 đ</b>. (2) 20,1 kg bike ⇒ HTTP 422 <code>VEHICLE_TOO_SMALL</code>. (3) 20,1 kg car ⇒ phụ phí 2.000 × ⌈15,1⌉ = 32.000 đ, tổng 58.000 + 32.000 = <b>90.000 đ</b>. Tiền điều kiện và bước như TC-PARCEL-002. Dòng (3) đáng có vì nó chứng minh 20,1 kg không bị chặn khi chọn đúng xe.</p>
</details>

**Bài tập 2.** Đội có ma trận cho nhóm phí huỷ: R-CANCEL-01 → TC-CAN-01, 02; R-CANCEL-02 → TC-CAN-03; R-CANCEL-03 → TC-CAN-04; TC-CAN-09 không trỏ tới yêu cầu nào. R-CANCEL-04…07 chưa có dòng. Nêu các khoảng trống và việc cần làm.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Truy vết xuôi: R-CANCEL-04 (miễn phí khi tài xế trễ), -05, -06, -07 chưa có test ⇒ độ phủ yêu cầu 3/7, không thể khẳng định rủi ro «thu phí sai khi tài xế trễ» đã giảm. Truy vết ngược: TC-CAN-09 «mồ côi» ⇒ hoặc nó kiểm một yêu cầu chưa được viết vào SPEC (báo cho người viết đặc tả — có thể là defect của test basis), hoặc nó thừa và nên xoá. Việc cần làm: phân tích R-CANCEL-04…07 thành test condition, ưu tiên theo rủi ro tiền, và đưa ma trận vào báo cáo tiến độ.</p>
</details>

**Câu 1 (K2).** Testware nào là sản phẩm của **test design**?

- a) Risk register.
- b) Test charter.
- c) Test log.
- d) Test execution schedule.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — thuộc test planning. <b>b đúng</b> — test design tạo test case, test charter, coverage item, yêu cầu về dữ liệu và môi trường. <b>c sai</b> — test execution. <b>d sai</b> — test implementation.</p>
</details>

**Câu 2 (K2).** Cặp hoạt động – testware nào ghép **đúng**?

- a) Test analysis – test procedure.
- b) Test implementation – test data và test suite.
- c) Test completion – test condition.
- d) Test monitoring – test script tự động.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — procedure thuộc implementation. <b>b đúng</b> — implementation tạo procedure, script, suite, test data. <b>c sai</b> — test condition thuộc analysis. <b>d sai</b> — script thuộc implementation; monitoring tạo progress report.</p>
</details>

**Câu 3 (K2).** Truy vết giữa **kết quả test và rủi ro** chủ yếu giúp gì?

- a) Tự động sinh test case từ rủi ro.
- b) Đánh giá mức rủi ro tồn đọng trong test object.
- c) Loại bỏ nhu cầu review yêu cầu.
- d) Tính số lượng defect developer gây ra.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — truy vết không sinh test. <b>b đúng</b> — đúng ví dụ syllabus nêu ở 1.4.4. <b>c sai</b> — không liên quan. <b>d sai</b> — truy vết không nhằm đếm lỗi theo người.</p>
</details>

**Câu 4 (K2).** Tới Nơi đổi phụ phí bưu kiện từ 2.000 đ lên 2.500 đ mỗi kg. Giá trị nào của truy vết thể hiện rõ nhất trong tình huống này?

- a) Xác định những test case và dữ liệu bị ảnh hưởng bởi thay đổi.
- b) Bảo đảm developer không cần chạy lại test.
- c) Thay thế configuration management.
- d) Làm cho test case không cần kết quả mong đợi.

<details>
<summary><b>Đáp án: a</b></summary>
<p><b>a đúng</b> — từ R-PARCEL-03 lần ra test condition, test case, dòng dữ liệu phải cập nhật. <b>b sai</b> — ngược lại, phải chạy lại. <b>c sai</b> — CM là nền để duy trì truy vết. <b>d sai</b> — không liên quan.</p>
</details>

**Câu 5 (K2).** Thông tin nào **không** thuộc nội dung điển hình của một defect report theo CTFL?

- a) Kết quả mong đợi và kết quả thực tế.
- b) Mức nghiêm trọng và ưu tiên sửa.
- c) Tên developer đã viết dòng code sai, để đánh giá hiệu suất.
- d) Định danh test object và test environment.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a, b, d sai</b> — đều có trong danh sách mục 5.5. <b>c đúng</b> — defect report nhằm xử lý lỗi, theo dõi chất lượng, cải tiến quy trình; quy lỗi cho cá nhân không phải mục tiêu.</p>
</details>

**Câu 6 (K2).** Trong **test analysis**, tester tìm thấy mâu thuẫn trong đặc tả và người viết chưa sửa ngay. Testware phù hợp là gì?

- a) Test log.
- b) Defect report về defect trong test basis.
- c) Test completion report.
- d) Không cần ghi lại vì chưa chạy test.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — test log ghi lại việc thực thi. <b>b đúng</b> — syllabus liệt kê defect report về test basis (nếu không sửa trực tiếp) là sản phẩm của test analysis. <b>c sai</b> — thuộc test completion. <b>d sai</b> — defect tìm bằng static testing cũng phải được ghi và xử lý.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB® Certified Tester Foundation Level Syllabus **v4.0.1** (2024): mục 1.4.3 Testware; 1.4.4 Traceability between the Test Basis and Testware; 5.1.1 (nội dung test plan); 5.4 Configuration Management; 5.5 Defect Management (FL-5.5.1). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB® Glossary (tra *testware*, *test condition*, *test case*, *test procedure*, *traceability*): https://glossary.istqb.org/
- Trang chủ ISTQB: https://www.istqb.org/
- ISO/IEC/IEEE 29119-3 — Test documentation (mẫu test plan, test case, incident report); ISO/IEC/IEEE 29119-2 — Test processes.
- Lab Tới Nơi: `SPEC.md` (R-PARCEL, R-FARE-03) và thư mục bài: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-004 · Error, Defect, Failure, Root Cause](/articles/error-defect-failure-root-cause) · [TEST-007 · Quy trình kiểm thử](/articles/quy-trinh-kiem-thu-test-process) · [TEST-009 · Vai trò trong kiểm thử](/articles/vai-tro-trong-kiem-thu)

Bài tiếp theo: [Thực hành Series 01 — phân tích tính năng đặt xe](/articles/thuc-hanh-series-01-phan-tich-tinh-nang-dat-xe).
