---
slug: test-case-design-traceability-matrix
title: "Ma trận truy vết (RTM): xuôi, ngược, nhiều-nhiều và phân tích ảnh hưởng cho vòng đời chuyến Tới Nơi"
summary: "Dựng RTM cho R-STATE và R-CANCEL: yêu cầu ↔ test condition ↔ test case ↔ kết quả ↔ defect, phân biệt độ phủ yêu cầu (92,9%) với độ phủ test condition (80%), tìm lỗ hổng, test case mồ côi và tag lệch, phân tích ảnh hưởng khi mốc 2 phút đổi thành 3 phút — RTM sinh tự động từ tag trong tiêu đề test, 30 ca vitest."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-case-design, tcd-08, traceability, rtm, test-coverage, impact-analysis, testware, vitest, toi-noi]
language: vi
---

Hỏi đội kiểm thử «yêu cầu nào đã được kiểm?», câu trả lời thường là một bảng tính: cột trái là mã yêu cầu, cột phải là danh sách test case, ô nào có chữ thì tô xanh. Bảng ấy cho ra con số đẹp — 13 trên 14 yêu cầu có test, 92,9% — nhưng không cho biết **mỗi yêu cầu được kiểm thử kỹ tới mức nào**, không cho biết test nào đã không còn trỏ về đâu, và mất giá trị ngay hôm có người sửa test mà quên sửa bảng. Bài này dựng **ma trận truy vết** (traceability matrix, RTM) cho hai nhóm yêu cầu dày logic nhất của Tới Nơi — vòng đời chuyến (R-STATE) và phí huỷ (R-CANCEL) — ở mức nhiều-nhiều thật: yêu cầu ↔ test condition ↔ test case ↔ kết quả ↔ defect. Rồi ta dùng nó cho đúng việc CTFL nói truy vết phục vụ: đo độ phủ, tìm lỗ hổng, và trả lời «SPEC đổi thì phải sửa những test nào». RTM trong lab không viết tay; nó được **sinh ra** từ tag trong tiêu đề test mỗi lần chạy.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TCD-08</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>TCD · Thiết kế Test Case</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Component; System</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; hỗ trợ regression và maintenance testing</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng — bài về testware và truy vết; dùng kết quả của state transition và decision table</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/test-case-design-test-analysis-test-condition">TCD-01 · Test analysis và test condition</a> · <a href="/articles/test-case-design-giai-phau-test-case">TCD-02 · Giải phẫu test case</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): 17 test thật có tag <code>@R-…</code>/<code>@TCOND-…</code>, bộ sinh RTM đọc SPEC và tiêu đề test, reporter ghép kết quả chạy, truy vấn ảnh hưởng và truy vết ngược — <code>examples/toi-noi-testing/articles/test-case-design-traceability-matrix/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.4.3 · FL-1.4.3 (K2); mục 1.4.4 · FL-1.4.4 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

[TEST-010 · Testware](/articles/testware-san-pham-kiem-thu) đã giới thiệu truy vết bằng một bảng bốn dòng cho phụ phí bưu kiện: mỗi yêu cầu một vài test case, một defect. Bảng ấy đủ để hiểu khái niệm, nhưng ẩn đi ba điều mà dự án thật gặp ngay:

1. **Quan hệ là nhiều-nhiều.** Một test case «tài xế huỷ ở `accepted`» chứng minh cùng lúc R-STATE-02 (chuyển hợp lệ), R-STATE-04 (hậu quả của tài xế huỷ) và R-CANCEL-05 (khách không mất phí). Một yêu cầu như R-CANCEL-01 lại cần năm test condition khác nhau. Bảng hai cột không biểu diễn được điều đó mà không lặp dòng.
2. **«Có test» không có nghĩa là «kiểm đủ».** R-CANCEL-04 (miễn phí khi tài xế trễ) có một test case ở 20 phút — yêu cầu được tô xanh. Biên đúng mốc, nơi lỗi `>` thành `>=` sống, thì chưa ai kiểm.
3. **Ma trận viết tay mục nát.** Test bị đổi tên, yêu cầu bị đánh số lại, test mới được thêm mà không ai cập nhật bảng. Sau vài sprint, RTM nói điều không còn đúng.

Bài này dùng RTM khi: cần báo cáo tiến độ theo yêu cầu cho stakeholder không chuyên kỹ thuật; dự án có audit hoặc quy định buộc chứng minh yêu cầu đã được kiểm; SPEC thay đổi thường xuyên và cần biết test nào phải sửa. Không dùng RTM để thay thế kỹ thuật thiết kế test — RTM chỉ cho biết **có liên kết**, không cho biết liên kết ấy **đủ mạnh**.

## 2. Lý thuyết theo ISTQB

### 2.1. Truy vết giữa test basis và testware — mục 1.4.4 (FL-1.4.4, K2)

CTFL v4.0.1 nói: để giám sát và điều khiển kiểm thử hiệu quả, cần **thiết lập và duy trì** truy vết xuyên suốt quy trình kiểm thử giữa **các phần tử của test basis**, **testware gắn với chúng** (ví dụ test condition, rủi ro, test case), **kết quả test** và **defect** phát hiện được. Syllabus nêu các giá trị:

| Giá trị | Nội dung theo CTFL | Ở bài này |
|---|---|---|
| Đánh giá độ phủ | Truy vết chính xác hỗ trợ đánh giá coverage; rất hữu ích nếu test basis có **tiêu chí phủ đo được**, có thể làm **KPI** cho mức đạt test objective | 92,9% yêu cầu, 80% test condition (mục 4.5) |
| Test case ↔ yêu cầu | Kiểm chứng yêu cầu đã được test case phủ | Cột «Test case» của RTM |
| Kết quả ↔ rủi ro | Đánh giá **rủi ro tồn đọng** trong test object | Cột «Kết quả» + defect |
| Ảnh hưởng của thay đổi | Xác định **impact** khi có thay đổi | Mốc 2 phút → 3 phút (mục 4.7) |
| Audit và quản trị CNTT | Hỗ trợ **test audit**, đáp ứng tiêu chí IT governance | RTM sinh từ mã, có lịch sử Git |
| Báo cáo dễ hiểu | Báo cáo tiến độ và kết thúc **dễ hiểu hơn** khi kèm trạng thái của từng phần tử test basis | Mỗi dòng RTM là một yêu cầu có trạng thái |

Câu kết của mục 1.4.4 tóm lại: truy vết cung cấp thông tin để đánh giá **chất lượng sản phẩm, năng lực quy trình và tiến độ dự án** so với mục tiêu kinh doanh.

### 2.2. Testware nào được nối — mục 1.4.3 (FL-1.4.3, K2)

Mục 1.4.3 liệt kê testware theo từng hoạt động. Phần nằm trong RTM của bài:

| Hoạt động (1.4.1) | Testware | Trong RTM |
|---|---|---|
| Test analysis | test condition (đã ưu tiên), defect report về test basis | Cột «Test condition»; câu hỏi Q-01 về TCOND-CANCEL-09 |
| Test design | test case, coverage item, yêu cầu test data | Cột «Test case» |
| Test implementation | test procedure, test script, test suite | Tệp test (`trang-thai.test.ts`, `phi-huy.test.ts`) |
| Test execution | test log, defect report | Cột «Kết quả», cột «Defect» |
| Test monitoring and control | test progress report | Bản in RTM có độ phủ |

Phân biệt bằng mức K2 của FL-1.4.3 là chỗ RTM hay làm sai: **test condition** («cái gì cần kiểm» — biên 2 phút), **test case** (tiền điều kiện, đầu vào, kết quả mong đợi cụ thể — huỷ lúc 2′00″ thì 10.000 đ), **test script** (mã chạy được), **test log** (lần chạy hôm nay đạt). Một RTM nối thẳng yêu cầu với tên tệp script đã bỏ qua tầng test condition — và mất luôn khả năng đo «kiểm thử kỹ tới mức nào».

### 2.3. Chiều, mức hạt và quan hệ nhiều-nhiều

**Truy vết xuôi** (forward): từ yêu cầu đi tới test — «R-CANCEL-02 đã có test chưa, kết quả thế nào?». Dùng khi lập kế hoạch, đo độ phủ, phân tích ảnh hưởng.

**Truy vết ngược** (backward): từ test, kết quả hoặc defect về yêu cầu — «test đỏ này thuộc yêu cầu nào, có còn yêu cầu nào không?». Dùng khi phân loại defect, khi xoá test, và khi tìm **test case mồ côi** (orphan) — test không truy về đâu hoặc truy về thứ không còn tồn tại.

**Nhiều-nhiều**: mỗi mắt xích nối được với nhiều mắt xích ở tầng kề bên. Hình 1 vẽ một lát cắt thật của RTM trong lab.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 940 360" role="img" aria-label="Lát cắt ma trận truy vết nhiều-nhiều: ba yêu cầu R-STATE-02, R-STATE-04, R-CANCEL-05 nối với các test condition STATE-03, STATE-06, STATE-07, CANCEL-11, rồi tới test case TC-STATE-003, TC-STATE-004; TC-STATE-004 đạt. TCOND-STATE-07 không có test case nào. Mũi tên trên là chiều xuôi, mũi tên dưới là chiều ngược.">
<defs>
<marker id="t08-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="100" y="26" text-anchor="middle" class="t-small t-b">Yêu cầu (SPEC)</text>
<text x="330" y="26" text-anchor="middle" class="t-small t-b">Test condition</text>
<text x="570" y="26" text-anchor="middle" class="t-small t-b">Test case</text>
<text x="800" y="26" text-anchor="middle" class="t-small t-b">Kết quả · defect</text>
<rect x="30" y="50" width="140" height="40" rx="6" class="f-warn-soft s-warn"/><text x="100" y="75" text-anchor="middle" class="t-small t-mono">R-STATE-02</text>
<rect x="30" y="130" width="140" height="40" rx="6" class="f-warn-soft s-warn"/><text x="100" y="155" text-anchor="middle" class="t-small t-mono">R-STATE-04</text>
<rect x="30" y="210" width="140" height="40" rx="6" class="f-warn-soft s-warn"/><text x="100" y="235" text-anchor="middle" class="t-small t-mono">R-CANCEL-05</text>
<rect x="250" y="50" width="160" height="40" rx="6" class="f-phys-soft s-phys"/><text x="330" y="75" text-anchor="middle" class="t-small t-mono">TCOND-STATE-03</text>
<rect x="250" y="130" width="160" height="40" rx="6" class="f-phys-soft s-phys"/><text x="330" y="155" text-anchor="middle" class="t-small t-mono">TCOND-STATE-06</text>
<rect x="250" y="210" width="160" height="40" rx="6" class="f-phys-soft s-phys"/><text x="330" y="235" text-anchor="middle" class="t-small t-mono">TCOND-CANCEL-11</text>
<rect x="250" y="280" width="160" height="40" rx="6" class="f-surface s-dead dash"/><text x="330" y="305" text-anchor="middle" class="t-small t-mono">TCOND-STATE-07</text>
<rect x="490" y="50" width="160" height="40" rx="6" class="f-pk-soft s-pk"/><text x="570" y="75" text-anchor="middle" class="t-small t-mono">TC-STATE-003</text>
<rect x="490" y="150" width="160" height="40" rx="6" class="f-pk-soft s-pk"/><text x="570" y="175" text-anchor="middle" class="t-small t-mono">TC-STATE-004</text>
<rect x="730" y="50" width="150" height="40" rx="6" class="f-surface s-line"/><text x="805" y="75" text-anchor="middle" class="t-small">đạt</text>
<rect x="730" y="150" width="150" height="40" rx="6" class="f-surface s-line"/><text x="805" y="175" text-anchor="middle" class="t-small">đạt</text>
<g class="s-muted" stroke-width="1.3">
<line x1="170" y1="70" x2="250" y2="70"/>
<line x1="170" y1="150" x2="250" y2="150"/>
<line x1="170" y1="230" x2="250" y2="150"/>
<line x1="170" y1="230" x2="250" y2="230"/>
<line x1="170" y1="230" x2="250" y2="300" class="dash"/>
<line x1="410" y1="70" x2="490" y2="70"/>
<line x1="410" y1="70" x2="490" y2="170"/>
<line x1="410" y1="150" x2="490" y2="170"/>
<line x1="410" y1="230" x2="490" y2="170"/>
<line x1="650" y1="70" x2="730" y2="70"/>
<line x1="650" y1="170" x2="730" y2="170"/>
</g>
<text x="440" y="305" class="t-small t-dead">← lỗ hổng: không test case nào (R-STATE-05 cũng trỏ vào đây)</text>
<line x1="60" y1="340" x2="880" y2="340" class="s-muted" stroke-width="1.4" marker-end="url(#t08-a)"/>
<text x="470" y="333" text-anchor="middle" class="t-small">xuôi: đã phủ chưa? · ngược (đọc từ phải sang): cái đỏ này thuộc về đâu?</text>
</svg>
</div>
<figcaption><b>Hình 1 · Một lát cắt nhiều-nhiều của RTM trong lab.</b> TC-STATE-004 một mình chứng minh cho ba test condition và ba yêu cầu; R-CANCEL-05 cần ba test condition, một trong số đó chưa có test case.</figcaption>
</figure>

**Mức hạt** (granularity) quyết định RTM trả lời được câu hỏi nào. Nối yêu cầu → test case cho biết «có test». Chèn tầng test condition ở giữa cho biết «kiểm khía cạnh nào». Nối thêm tầng coverage item (ví dụ 12 chuyển hợp lệ của R-STATE-02, đã đếm ở [TEST-034](/articles/state-transition-testing-co-ban)) thì RTM thành công cụ đo coverage của kỹ thuật — chi tiết hơn, nhưng đắt để duy trì. Bài này dừng ở tầng test condition; mục 6 bàn khi nào nên đi sâu hơn.

### 2.4. Độ phủ yêu cầu và độ phủ test condition

Hai chỉ số dễ nhầm:

- **Độ phủ yêu cầu** = số yêu cầu trong phạm vi có ít nhất một test case ÷ số yêu cầu trong phạm vi.
- **Độ phủ test condition** = số test condition có ít nhất một test case ÷ số test condition đã xác định trong test analysis.

Số thứ nhất thô: một test case bất kỳ đã đủ tô xanh một yêu cầu. Số thứ hai phản ánh **chiều sâu phân tích**, nhưng chỉ tốt bằng chính bộ test condition — nếu test analysis bỏ sót biên thì độ phủ test condition 100% vẫn sót biên. Không số nào đo chất lượng của test case; cả hai chỉ đo **có liên kết**. CTFL gọi tiêu chí phủ là KPI cho mức đạt test objective — KPI tốt khi đi đôi với tiêu chí đúng.

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Tên «Requirements Traceability Matrix», các khái niệm «test case mồ côi», «tag lệch», quy ước gắn tag <code>@R-…</code> vào tiêu đề test và việc sinh RTM tự động từ mã là thực hành ngành. CTFL chỉ yêu cầu truy vết được thiết lập và duy trì; hình thức là lựa chọn của đội. Quy ước tag trong tiêu đề giống cách Playwright cho phép lọc test theo <code>@tag</code>; lab của bài chỉ dùng vitest và tự đọc tiêu đề. ISO/IEC/IEEE 29119-3 có mẫu tài liệu cho test design specification và truy vết.</p>
</div>

### 2.5. Thuật ngữ Anh – Việt

| Thuật ngữ | Tiếng Việt | Ghi chú |
|---|---|---|
| traceability | truy vết | quan hệ giữa các work product (CTFL 1.4.4) |
| traceability matrix (RTM) | ma trận truy vết | bảng biểu diễn truy vết |
| forward / backward traceability | truy vết xuôi / ngược | yêu cầu → test / test, defect → yêu cầu |
| bidirectional traceability | truy vết hai chiều | duy trì cả xuôi lẫn ngược |
| requirement coverage | độ phủ yêu cầu | yêu cầu có test ÷ yêu cầu trong phạm vi |
| test condition coverage | độ phủ test condition | test condition có test ÷ tổng test condition |
| coverage gap | lỗ hổng phủ | yêu cầu hoặc test condition chưa có test |
| orphan test case | test case mồ côi | không truy về phần tử test basis nào còn tồn tại |
| impact analysis | phân tích ảnh hưởng | đổi gì thì phải xem lại gì |
| residual risk | rủi ro tồn đọng | CTFL 1.4.4 |

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu trong phạm vi

SPEC lab có 8 yêu cầu R-STATE và 7 yêu cầu R-CANCEL. Tóm tắt phần dùng cho RTM:

| Mã | Nội dung |
|---|---|
| R-STATE-01 | 8 trạng thái; `completed`, `cancelled` là trạng thái kết thúc |
| R-STATE-02 | Bảng 12 chuyển hợp lệ (luồng chính, khách huỷ ×3, tài xế huỷ ×2, hệ thống huỷ ×1) |
| R-STATE-03 | Cặp (trạng thái, sự kiện) ngoài bảng ⇒ `INVALID_TRANSITION` (409), chuyến giữ nguyên |
| R-STATE-04 | Tài xế huỷ ⇒ về `searching`, bỏ gán, xoá `acceptedAt` và ETA, tài xế `online`, đặt lại `searchingSince` |
| R-STATE-05 | `searching` từ 5 phút ⇒ hệ thống huỷ, lý do `no_driver`, phí 0 đ |
| R-STATE-06 | Mỗi lần đổi trạng thái ghi một dòng `trip_events`; tạo chuyến ghi hai dòng |
| R-STATE-07 | Ghi `acceptedAt`, `completedAt`, `cancelledAt` |
| R-STATE-08 | Bảng đối chiếu với kiến trúc đích — SPEC ghi rõ «tham khảo, không phải yêu cầu kiểm thử» |
| R-CANCEL-01…07 | Phí theo trạng thái, mốc 2 phút, mức 10.000/20.000 đ, miễn phí khi tài xế trễ, tài xế/hệ thống huỷ không thu phí, lý do phí, thu và lưu phí (chi tiết ở [TEST-033](/articles/decision-table-testing)) |

### 3.2. System context

Kiến trúc đích tách vòng đời chuyến thành trạng thái riêng ở Booking, Dispatch, Trip và Payment (R-STATE-08). Lab giữ một vòng đời tám trạng thái ở `src/domain/trip-state.ts`, các use case ở `src/app/trips.ts`, hàm phí ở `src/domain/cancel.ts` — đúng bảng truy vết yêu cầu → mã nguồn ở Phụ lục B của SPEC. RTM của bài là nửa còn lại của bức tranh: yêu cầu → **testware**.

### 3.3. Rủi ro

| Rủi ro | Hậu quả | Truy vết giúp gì |
|---|---|---|
| Báo «đã kiểm 100% yêu cầu» trong khi biên chưa kiểm | Phát hành với lỗi thu phí sai | Đo thêm độ phủ test condition |
| SPEC đổi mốc 2 phút, quên sửa test | Test xanh nhưng sai quy tắc mới | Phân tích ảnh hưởng xuôi |
| Test đỏ mà không ai biết nó thuộc yêu cầu nào | Phân loại defect chậm, ưu tiên sai | Truy vết ngược |
| Test cũ trỏ vào yêu cầu đã bị xoá | Bảo trì test vô ích, báo cáo sai | Phát hiện mồ côi |
| Auditor hỏi bằng chứng cho R-CANCEL-07 | Không chứng minh được | RTM có kết quả chạy kèm |

## 4. Phân tích kỹ thuật

### 4.1. Xác định phạm vi và mẫu số

Bộ sinh RTM đọc SPEC.md, lấy mọi mã có dạng `**R-STATE-nn —` hoặc `**R-CANCEL-nn —`: **15 yêu cầu**. R-STATE-08 bị loại khỏi mẫu số, **kèm lý do** ghi trong registry. Phạm vi còn **14**. Hai nguyên tắc:

- Mẫu số đến từ test basis, không từ danh sách test. Nếu đếm «yêu cầu được nhắc trong test» thì độ phủ luôn 100%.
- Loại trừ phải có lý do viết ra. Một yêu cầu biến khỏi mẫu số mà không ai giải thích là cách nhanh nhất để làm đẹp số liệu.

### 4.2. Test condition — tầng giữa của RTM

Test analysis ([TCD-01](/articles/test-case-design-test-analysis-test-condition)) cho ra 20 test condition. Cột cuối là quan hệ nhiều-nhiều với yêu cầu.

| Test condition | Nội dung | Truy về |
|---|---|---|
| TCOND-STATE-01 | Đúng 8 trạng thái, 2 trạng thái kết thúc | R-STATE-01 |
| TCOND-STATE-02 | Luồng chính tới `completed` (6 chuyển) | R-STATE-02 |
| TCOND-STATE-03 | Các chuyển huỷ hợp lệ | R-STATE-02 |
| TCOND-STATE-04 | Mọi sự kiện ở trạng thái kết thúc bị từ chối | R-STATE-03 |
| TCOND-STATE-05 | Sự kiện sai thứ tự / khách huỷ sau khi đón bị từ chối | R-STATE-03, R-CANCEL-01 |
| TCOND-STATE-06 | Hậu quả của tài xế huỷ | R-STATE-04, R-CANCEL-05 |
| TCOND-STATE-07 | Hết 5 phút tìm ⇒ `no_driver`, phí 0; đúng 5 phút tính là hết | R-STATE-05, R-CANCEL-05 |
| TCOND-STATE-08 | Nhật ký: 2 dòng khi tạo, 1 dòng mỗi chuyển, không ghi khi bị từ chối | R-STATE-06 |
| TCOND-STATE-09 | Mốc `acceptedAt`, `completedAt`, `cancelledAt` | R-STATE-07 |
| TCOND-CANCEL-01 | `searching` ⇒ 0 đ, `searching` | R-CANCEL-01, R-CANCEL-06 |
| TCOND-CANCEL-02 | `accepted` chưa đủ 2 phút ⇒ 0 đ, `grace_period` | R-CANCEL-01, 02, 06 |
| TCOND-CANCEL-03 | Biên 2 phút: 1′59,999″ miễn; 2′00″ thu | R-CANCEL-02 |
| TCOND-CANCEL-04 | `accepted` từ 2 phút ⇒ phí theo loại xe | R-CANCEL-01, 03 |
| TCOND-CANCEL-05 | `arriving` ⇒ có phí, kể cả trong 2 phút đầu | R-CANCEL-01 |
| TCOND-CANCEL-06 | Mức phí: xe máy 10.000 đ, ô tô 20.000 đ, đồ ăn 10.000 đ | R-CANCEL-03 |
| TCOND-CANCEL-07 | Quá mốc trễ ⇒ 0 đ, `driver_late` | R-CANCEL-04, 06 |
| TCOND-CANCEL-08 | Biên trễ: đúng mốc thu, quá 1 ms miễn | R-CANCEL-04 |
| TCOND-CANCEL-09 | Thứ tự ưu tiên lý do khi nhiều lý do miễn cùng đúng | R-CANCEL-06 |
| TCOND-CANCEL-10 | Thu VíXanh khoá `cancel-fee` / ghi nhận tiền mặt; lưu phí | R-CANCEL-07 |
| TCOND-CANCEL-11 | Tài xế huỷ không thu phí khách | R-CANCEL-05 |

TCOND-CANCEL-09 có ghi chú riêng. Với ETA tối thiểu 1 phút (R-ACCEPT-04), mốc trễ sớm nhất là 11 phút sau khi nhận, nên «chưa đủ 2 phút» và «tài xế trễ» không bao giờ cùng đúng — thứ tự ưu tiên của R-CANCEL-06 không kiểm được qua hệ thống. Đây là **defect report về test basis** (câu hỏi Q-01 gửi BA), một testware của test analysis theo CTFL 1.4.3. RTM giữ test condition này ở trạng thái chưa phủ, có lý do, thay vì xoá đi.

### 4.3. Quy ước tag: test case tự khai truy vết

Mỗi tiêu đề test bắt đầu bằng mã test case, theo sau là tag:

```ts
it('TC-STATE-004 @R-STATE-02 @R-STATE-04 @R-CANCEL-05 @TCOND-STATE-03 @TCOND-STATE-06 @TCOND-CANCEL-11 · tài xế huỷ ở accepted: chuyến về searching, khách không mất phí', async () => { … });
```

Tag `@TCOND-…` là liên kết chính; registry đã biết test condition truy về yêu cầu nào. Tag `@R-…` trực tiếp là thông tin dư — có chủ đích, để **kiểm chéo**: nếu test case gắn `@R-X` mà không test condition nào của nó truy về R-X thì hoặc registry thiếu một liên kết, hoặc tag sai. Bộ sinh RTM gọi đó là **tag lệch**.

Lý do đặt truy vết trong tiêu đề thay vì một bảng tính riêng: tiêu đề nằm cạnh mã test, đổi cùng một commit, được review cùng pull request, và xuất hiện trong mọi báo cáo chạy. Bảng tính riêng là bản sao thứ hai — và bản sao thì lệch.

### 4.4. RTM sinh ra

Bản in RTM của lab sau khi chạy (cột «Test case» bỏ tiền tố `TC-`, cột «Test condition» bỏ tiền tố `TCOND-`):

| Yêu cầu | Test condition | Test case | Kết quả | Defect |
|---|---|---|---|---|
| R-STATE-01 | STATE-01 | STATE-001 | 1/1 đạt | — |
| R-STATE-02 | STATE-02, STATE-03 | STATE-002, STATE-003, STATE-004 | 3/3 đạt | — |
| R-STATE-03 | STATE-04, STATE-05 | STATE-005, STATE-006 | 2/2 đạt | — |
| R-STATE-04 | STATE-06 | STATE-004 | 1/1 đạt | — |
| R-STATE-05 | STATE-07 | **chưa có** | — | — |
| R-STATE-06 | STATE-08 | STATE-002, STATE-005 | 2/2 đạt | — |
| R-STATE-07 | STATE-09 | STATE-002, STATE-003 | 2/2 đạt | — |
| R-CANCEL-01 | STATE-05, CANCEL-01, 02, 04, 05 | STATE-005, STATE-006, CANCEL-001, 002, 005, 006 | 6/6 đạt | DEF-SIM-001 |
| R-CANCEL-02 | CANCEL-02, CANCEL-03 | CANCEL-002, 003, 004, 006 | 4/4 đạt | DEF-SIM-001 |
| R-CANCEL-03 | CANCEL-04, CANCEL-06 | CANCEL-004, 005, 007 | 3/3 đạt | — |
| R-CANCEL-04 | CANCEL-07, CANCEL-08 | CANCEL-008 | 1/1 đạt | — |
| R-CANCEL-05 | STATE-06, STATE-07, CANCEL-11 | STATE-004 | 1/1 đạt | — |
| R-CANCEL-06 | CANCEL-01, 02, 07, 09 | CANCEL-001, 002, 008 | 3/3 đạt | — |
| R-CANCEL-07 | CANCEL-10 | CANCEL-009 | 1/1 đạt | — |

DEF-SIM-001 là defect **giả lập**, không phải lỗi của `src/`: mô phỏng lỗi «khách huỷ ở `arriving` trong 2 phút đầu không bị thu phí» mà TEST-033 cài trong bản lỗi của nó, gắn vào TC-CANCEL-006 để minh hoạ truy vết defect.

Đọc RTM theo dòng là truy vết xuôi. Đọc theo một test case qua mọi dòng là truy vết ngược: TC-STATE-004 xuất hiện ở R-STATE-02, R-STATE-04 và R-CANCEL-05 — một test, ba yêu cầu. Nếu test này đỏ, ba dòng cùng đỏ; báo cáo theo yêu cầu phải nói rõ đó là **một** lỗi, không phải ba.

### 4.5. 92,9% và 80% — vì sao con số thứ nhất nông

| Chỉ số | Kết quả | Thiếu gì |
|---|---|---|
| Độ phủ yêu cầu | 13/14 = **92,9%** | R-STATE-05 |
| Độ phủ test condition | 16/20 = **80%** | TCOND-STATE-04, TCOND-STATE-07, TCOND-CANCEL-08, TCOND-CANCEL-09 |

Ba trong bốn test condition chưa phủ thuộc về yêu cầu **đã được tô xanh**:

- **R-CANCEL-04** có TC-CANCEL-008 (huỷ ở 20 phút) nên «đã phủ». Nhưng TCOND-CANCEL-08 — đúng mốc vẫn thu, quá 1 ms thì miễn — chưa có test. [TCD-07](/articles/test-case-design-user-story-acceptance-criteria) đã chứng minh một bản lỗi `>=` vượt qua mọi ví dụ không nằm trên biên. RTM ở mức yêu cầu không thấy được lỗ này.
- **R-STATE-03** có hai test case cho chuyển sai thứ tự, nhưng chưa test nào kiểm trạng thái kết thúc (TCOND-STATE-04). Đúng loại lỗi mà [TEST-034](/articles/state-transition-testing-co-ban) cảnh báo: chuyến đã huỷ bị «hồi sinh».
- **R-CANCEL-06** có ba test case cho ba lý do, nhưng thứ tự ưu tiên (TCOND-CANCEL-09) không kiểm được — và đó là câu hỏi cho BA, không phải việc của tester.

Bài học đúng theo CTFL: tiêu chí phủ là KPI tốt khi **tiêu chí** đủ mịn. Báo cáo cho stakeholder nên kèm cả hai số, và nói rõ số nào đo cái gì.

### 4.6. Lỗ hổng, mồ côi, tag lệch

Bộ sinh RTM báo bốn loại bất thường:

| Loại | Phát hiện | Hành động |
|---|---|---|
| **Yêu cầu chưa có test** | R-STATE-05 | Viết test cho TCOND-STATE-07 (đúng 5 phút ⇒ hết hạn; 4′59,999″ ⇒ chưa) |
| **Test condition chưa có test** | 4 mục ở mục 4.5 | Viết test, hoặc ghi lý do (TCOND-CANCEL-09 chờ BA) |
| **Test case mồ côi — không tag** | «khách huỷ lần thứ hai một chuyến đã huỷ thì bị từ chối» | Gắn `@R-STATE-03 @TCOND-STATE-04` |
| **Test case mồ côi — trỏ vào mã không có** | TC-CANCEL-090 `@R-CANCEL-08` | R-CANCEL-08 không có trong SPEC; nội dung test (lưu lý do khách nhập) thuộc R-API-11 — đổi tag hoặc chuyển test sang bộ R-API |
| **Tag lệch** | TC-CANCEL-006 gắn `@R-CANCEL-02` nhưng TCOND-CANCEL-05 chỉ truy về R-CANCEL-01 | Quyết định: thêm R-CANCEL-02 vào TCOND-CANCEL-05 (vì «không có 2 phút miễn phí ở `arriving`» đúng là một khía cạnh của mốc 2 phút) hoặc bỏ tag |

Trường hợp mồ côi không tag đáng dừng lại: test ấy kiểm **đúng** TCOND-STATE-04 — chính test condition mà RTM báo là lỗ hổng. Hệ thống có test, chạy xanh, nhưng RTM không biết. Hệ quả: báo cáo tiến độ thấp hơn thực tế, và — nguy hiểm hơn — phân tích ảnh hưởng sẽ **bỏ sót** test này (mục 4.7). Truy vết chỉ đúng bằng kỷ luật gắn tag.

Tag lệch cũng có hậu quả lan: vì TC-CANCEL-006 gắn `@R-CANCEL-02`, defect giả lập DEF-SIM-001 hiện ở **cả** dòng R-CANCEL-02 lẫn R-CANCEL-01. Nếu tag ấy sai, báo cáo defect theo yêu cầu đang phóng đại.

### 4.7. Phân tích ảnh hưởng khi SPEC thay đổi

**Thay đổi 1 — mốc miễn phí từ 2 phút thành 3 phút** (giả định bài tập; R-CANCEL-02 hiện là 2 phút). Truy vấn ảnh hưởng xuôi trên R-CANCEL-02:

```text
Ảnh hưởng khi đổi R-CANCEL-02:
  test condition: TCOND-CANCEL-02, TCOND-CANCEL-03
  test case:      TC-CANCEL-002, TC-CANCEL-003, TC-CANCEL-004, TC-CANCEL-006
```

Danh sách là điểm khởi đầu, không phải kết luận. Xét từng test case:

| Test case | Dữ liệu hiện tại | Sau thay đổi | Việc phải làm |
|---|---|---|---|
| TC-CANCEL-002 | 1′ ⇒ 0 đ | vẫn 0 đ | Không đổi kết quả; giữ |
| TC-CANCEL-003 | 1′59,999″ ⇒ 0 đ | vẫn 0 đ nhưng **không còn là biên** | Đổi dữ liệu thành 2′59,999″ |
| TC-CANCEL-004 | 2′00″ ⇒ 10.000 đ | **0 đ, `grace_period`** | Đổi dữ liệu thành 3′00″ ⇒ 10.000 đ |
| TC-CANCEL-006 | `arriving` 1′ ⇒ 10.000 đ | vẫn 10.000 đ | Không đổi; lọt vào danh sách nhờ tag lệch — xem lại tag |

TC-CANCEL-003 là bài học tinh tế: nó **vẫn xanh** sau thay đổi, nhưng không còn kiểm thứ nó được đặt ra để kiểm. Chỉ đọc kết quả chạy thì không bao giờ phát hiện; phải đọc test condition (biên) cùng dữ liệu.

Những gì RTM này **không thấy**: hằng `FREE_CANCEL_MINUTES` trong `src/domain/cancel.ts` (truy vết yêu cầu → mã nguồn là Phụ lục B của SPEC, không phải RTM này); các kịch bản SC-03, SC-04, SC-15 của TCD-07; cột DT-02 của TEST-033; tiêu chí UA-02 của UAT. RTM chỉ biết những liên kết được khai cho nó. Phân tích ảnh hưởng thật cần một RTM bao trùm mọi bộ test của dự án, hoặc ít nhất một quy ước tag chung cho mọi bộ.

Cũng có ảnh hưởng **gián tiếp** mà không công cụ đọc liên kết nào thấy: nới mốc miễn phí làm thay đổi lập luận «không khả thi» của TEST-033 và của TCOND-CANCEL-09. Với 3 phút, mốc trễ sớm nhất (11 phút) vẫn lớn hơn, nên hai lý do vẫn không chồng lên nhau — nhưng điều đó phải được **suy luận** lại, không tự suy ra từ ma trận.

**Thay đổi 2 — thêm một trạng thái** (giả định bài tập: `waiting` — tài xế đã tới và đang chờ, giữa `arriving` và `picked_up`). Yêu cầu bị chạm: R-STATE-01 (danh sách trạng thái), R-STATE-02 (bảng chuyển), R-STATE-03 (mọi cặp mới không hợp lệ), R-CANCEL-01 (khách huỷ ở `waiting` có phí không?). Truy vấn trên bốn yêu cầu: **9 test condition, 10 test case**. Hai điều đáng chú ý:

- Test mồ côi «khách huỷ lần thứ hai…» **không** có trong danh sách, dù nó kiểm R-STATE-03. Sau thay đổi, nó vẫn chạy, vẫn xanh, và không ai xem lại.
- R-ACCEPT-05 (tài xế `busy` khi chuyến ở `accepted`/`arriving`/`picked_up`/`in_transit`) cũng phải đổi để gồm `waiting` — nhưng R-ACCEPT nằm ngoài phạm vi RTM này. Phạm vi hẹp làm phân tích ảnh hưởng hẹp theo.

## 5. Thực hành

### 5.1. Tổ chức mã trong lab

| Tệp | Vai trò |
|---|---|
| `dieu-kien.ts` | Registry: 20 test condition và liên kết tới yêu cầu, lý do loại R-STATE-08, defect giả lập DEF-SIM-001 |
| `trang-thai.test.ts` | 7 test thật cho R-STATE (6 có tag, 1 cố ý không tag) |
| `phi-huy.test.ts` | 10 test thật cho R-CANCEL (9 tag đúng, 1 trỏ vào R-CANCEL-08) |
| `dung-chuyen.ts` | Dựng chuyến tới trạng thái cần bằng use case thật, `FixedClock` |
| `rtm.ts` | Đọc SPEC và tiêu đề test, dựng RTM, tính độ phủ, tìm lỗ hổng/mồ côi/tag lệch, truy vấn ảnh hưởng và truy vết ngược |
| `rtm.test.ts` | 13 test kiểm chính RTM |
| `rtm-reporter.ts` | Reporter vitest: ghép kết quả chạy vào RTM, ghi `reports/rtm-tcd08.md` |
| `in-rtm.ts` | Script in RTM tĩnh và trả lời truy vấn |

Test «sản phẩm» là test thật: dựng chuyến bằng `createQuote`, `createTrip`, `acceptTrip`, `driverStep`, rồi gọi `cancelByCustomer`/`cancelByDriver` với đồng hồ cố định (tạo chuyến 09:00:00, nhận 09:01:00, ETA 5 phút). Ví dụ TC-STATE-005 kiểm cả ba đầu ra của một chuyển bị từ chối: mã lỗi, chuyến không đổi, nhật ký không thêm dòng.

```ts
it('TC-STATE-005 @R-STATE-03 @R-STATE-06 @TCOND-STATE-05 @TCOND-STATE-08 · khách huỷ ở picked_up bị từ chối, chuyến và nhật ký giữ nguyên', async () => {
  const { deps, tripId } = await dungChuyen({ toi: 'picked_up' });
  const truoc = (await deps.trips.findById(tripId))!;
  const soDong = (await deps.trips.listEvents(tripId)).length;
  expect(await maLoi(cancelByCustomer(deps, KHACH, tripId))).toBe('INVALID_TRANSITION');
  expect(await deps.trips.findById(tripId)).toEqual(truoc);
  expect(await deps.trips.listEvents(tripId)).toHaveLength(soDong);
});
```

### 5.2. Bộ sinh RTM

Phần lõi là ba phép đọc và một phép ghép:

```ts
// rtm.ts (trích)
export function docYeuCau(spec: string, nhom = NHOM_TRONG_PHAM_VI): string[] {
  const tatCa = [...spec.matchAll(/\*\*(R-[A-Z]+-\d{2}) —/g)].map((m) => m[1]!);
  return [...new Set(tatCa)].filter((m) => nhom.some((n) => m.startsWith(`${n}-`)));
}

/** Yêu cầu mà một test case truy về: tag @R-… trực tiếp ∪ yêu cầu của các @TCOND-…. */
export function yeuCauCuaTestCase(t: TestCaseTag, dk: TestCondition[]): string[] {
  const quaDk = t.tcond.flatMap((id) => dk.find((d) => d.id === id)?.spec ?? []);
  return [...new Set([...t.spec, ...quaDk])].sort();
}
```

Tiêu đề test được đọc **tĩnh** từ mã nguồn bằng biểu thức chính quy, không cần chạy test. Vì vậy quy ước bắt buộc: tiêu đề là chuỗi hằng, không dùng `it.each` với tiêu đề sinh động cho test cần truy vết. Kết quả chạy được ghép vào sau bằng reporter — hai nguồn tách nhau, nên RTM vẫn in được khi test chưa chạy (giai đoạn thiết kế) và có thêm cột kết quả khi đã chạy.

### 5.3. Chạy

```bash
cd examples/toi-noi-testing
npx vitest run articles/test-case-design-traceability-matrix \
  --reporter=default --reporter=./articles/test-case-design-traceability-matrix/rtm-reporter.ts
```

Kết quả thật (rút gọn; bảng RTM đầy đủ như mục 4.4):

```text
 ✓ articles/test-case-design-traceability-matrix/rtm.test.ts (13 tests)
 ✓ articles/test-case-design-traceability-matrix/trang-thai.test.ts (7 tests)
 ✓ articles/test-case-design-traceability-matrix/phi-huy.test.ts (10 tests)

Độ phủ yêu cầu: 13/14 = 92,9%
Độ phủ test condition: 16/20 = 80%
Yêu cầu chưa có test case: R-STATE-05
Test condition chưa có test case: TCOND-STATE-04, TCOND-STATE-07, TCOND-CANCEL-08, TCOND-CANCEL-09
Test case mồ côi: «khách huỷ lần thứ hai một chuyến đã huỷ …» (không có tag @R-… hay @TCOND-…); «TC-CANCEL-090 @R-CANCEL-08 · lý do khách…» (R-CANCEL-08 không có trong test basis)
Tag lệch registry: TC-CANCEL-006 → R-CANCEL-02

 Test Files  3 passed (3)
      Tests  30 passed (30)
```

30 ca = 13 ca kiểm RTM + 7 ca R-STATE + 10 ca R-CANCEL. Mọi test sản phẩm đều xanh: bản `src/` hiện tại đúng SPEC trên những gì đã kiểm. Đó chính là lý do cần RTM — **xanh hết không có nghĩa là kiểm hết**.

### 5.4. Truy vấn

```bash
npx tsx articles/test-case-design-traceability-matrix/in-rtm.ts --impact R-CANCEL-02
npx tsx articles/test-case-design-traceability-matrix/in-rtm.ts --back DEF-SIM-001
```

```text
Truy vết ngược DEF-SIM-001: {
  "testCase": "TC-CANCEL-006",
  "dieuKien": [ "TCOND-CANCEL-05" ],
  "yeuCau": [ "R-CANCEL-01", "R-CANCEL-02" ]
}
```

Chuỗi ngược defect → test case → test condition → yêu cầu là thứ người phân loại defect cần: biết defect thuộc quy tắc nào để ưu tiên, biết test condition nào để viết confirmation test ([TEST-019](/articles/confirmation-testing-va-regression-testing)).

### 5.5. Đưa vào CI

`rtm.test.ts` hiện khẳng định **đúng trạng thái có chủ đích** của lab (có lỗ hổng để phân tích). Trong dự án thật, đổi các khẳng định thành cổng chất lượng:

```ts
expect(rtm.moCoi).toEqual([]);                        // không test mồ côi
expect(rtm.lechTag).toEqual([]);                      // tag khớp registry
expect(rtm.phuYeuCau.phanTram).toBeGreaterThanOrEqual(NGUONG_DA_THONG_NHAT);
expect(rtm.yeuCauChuaCoDieuKien).toEqual([]);         // test analysis không bỏ sót yêu cầu
```

Ngưỡng phủ là quyết định của test manager cùng stakeholder (thường ghi trong test plan, [TEST-007](/articles/quy-trinh-kiem-thu-test-process)), không phải con số tester tự đặt. Hai cổng đầu thì nên tuyệt đối: một test mồ côi là một test không ai biết đang bảo vệ gì.

## 6. Góc nhìn nâng cao

**Chọn mức hạt theo rủi ro.** Logic tiền (R-CANCEL) đáng đi xuống tầng coverage item — mỗi cột khả thi của bảng quyết định là một dòng RTM. Quy tắc hiển thị (R-UI) thường chỉ cần tầng yêu cầu. RTM đồng nhất mức hạt cho mọi yêu cầu vừa tốn công ở chỗ không cần, vừa nông ở chỗ cần.

**RTM trong công cụ quản lý test.** Nhiều đội giữ RTM trong công cụ quản lý test, liên kết với hệ thống quản lý yêu cầu. Cách này mạnh về giao diện và phân quyền, nhưng có cùng điểm yếu với bảng tính: liên kết nằm xa mã test. Một hướng dung hoà là giữ tag trong mã làm nguồn chân lý, đồng bộ một chiều sang công cụ khi chạy CI.

**Truy vết rủi ro.** CTFL nêu kết quả test ↔ rủi ro để đánh giá rủi ro tồn đọng. Thêm cột rủi ro vào registry test condition (ví dụ RR-1 «thu phí sai ở biên») là đủ để RTM trả lời «rủi ro nào còn mở» — nền cho ưu tiên test case ở [TCD-09](/articles/test-case-design-uu-tien-review-bao-tri).

**Phiên bản test basis.** RTM chỉ đúng với một phiên bản SPEC. Khi SPEC lên phiên bản mới, so hai danh sách mã: mã mới xuất hiện ⇒ lỗ hổng; mã biến mất ⇒ test mồ côi (đúng loại TC-CANCEL-090). Đặt RTM dưới configuration management cùng SPEC — xem [TEST-010](/articles/testware-san-pham-kiem-thu) và [maintenance testing](/articles/maintenance-testing).

| Sai lầm thường gặp | Cách tránh |
|---|---|
| Mẫu số lấy từ danh sách test thay vì từ test basis | Đọc yêu cầu từ SPEC; loại trừ phải có lý do |
| Chỉ báo độ phủ yêu cầu | Báo kèm độ phủ test condition, nói rõ mỗi chỉ số phản ánh gì |
| RTM viết tay ở bảng tính riêng | Sinh từ tag cạnh mã test, kiểm trong CI |
| Bỏ qua tầng test condition | Nối yêu cầu → test condition → test case |
| Coi danh sách ảnh hưởng là kết luận | Đọc từng test case: kết quả mong đợi đổi? dữ liệu còn là biên? |
| Xoá test condition không kiểm được | Giữ, ghi lý do, mở câu hỏi cho BA |
| Đếm một lỗi thành nhiều lỗi khi một test truy về nhiều yêu cầu | Báo cáo defect theo mã defect, RTM chỉ hiện ảnh hưởng |

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 (K3).** Gắn tag `@R-STATE-03 @TCOND-STATE-04` cho test không tag, và thêm một test `TC-CANCEL-010 @R-CANCEL-04 @TCOND-CANCEL-08` kiểm đúng mốc trễ. Không làm gì khác. Độ phủ yêu cầu, độ phủ test condition và số test mồ côi mới là bao nhiêu?

<details><summary><b>Lời giải</b></summary>
<p>Độ phủ test condition: thêm TCOND-STATE-04 và TCOND-CANCEL-08 ⇒ 18/20 = <b>90%</b> (còn TCOND-STATE-07 và TCOND-CANCEL-09). Độ phủ yêu cầu <b>không đổi</b>: 13/14 = 92,9%, vì R-STATE-03 và R-CANCEL-04 vốn đã «có test»; R-STATE-05 vẫn là lỗ hổng. Mồ côi còn <b>1</b> (TC-CANCEL-090). Bài tập cho thấy hai chỉ số phản ứng khác nhau: thêm test vào chỗ nông làm tăng số thứ hai mà không chạm số thứ nhất. Dữ liệu cho TC-CANCEL-010: nhận 09:01:00, ETA 5, huỷ 09:16:00,000 ⇒ 10.000 đ <code>charged</code>, và 09:16:00,001 ⇒ 0 đ <code>driver_late</code> (hai giá trị biên — có thể tách thành hai test case cùng test condition).</p>
</details>

**Bài tập 2 (K4).** SPEC đổi phí huỷ ô tô từ 20.000 đ lên 25.000 đ (giả định; R-CANCEL-03). Chạy truy vấn ảnh hưởng trên R-CANCEL-03. Test case nào có kết quả mong đợi phải đổi? RTM có bỏ sót test nào không?

<details><summary><b>Lời giải</b></summary>
<p>Truy vấn trả TCOND-CANCEL-04, TCOND-CANCEL-06 và TC-CANCEL-004, 005, 007. Xét từng ca: TC-CANCEL-004 là xe máy (10.000 đ) — không đổi; TC-CANCEL-007 là đồ ăn (10.000 đ) — không đổi; TC-CANCEL-005 là ô tô ⇒ đổi thành 25.000 đ. RTM <b>bỏ sót TC-CANCEL-009</b>: test VíXanh dùng ô tô và khẳng định số tiền 20.000 đ gửi tới ví, nhưng chỉ gắn <code>@R-CANCEL-07 @TCOND-CANCEL-10</code>. Sau thay đổi nó sẽ đỏ — tốt hơn là xanh sai, nhưng phân tích ảnh hưởng đã không báo trước. Sửa: gắn thêm <code>@TCOND-CANCEL-06</code> (test này cũng dựa vào mức phí), hoặc chấp nhận rằng test của R-CANCEL-07 phụ thuộc mức phí và ghi điều đó vào registry. Bài học: một test dùng giá trị của quy tắc khác là một liên kết ngầm; tag phải khai cả liên kết đó.</p>
</details>

**Câu 1 (K2 — FL-1.4.4).** Theo CTFL v4.0.1, giá trị nào **không** phải là giá trị của việc duy trì truy vết?

a) Đánh giá mức rủi ro tồn đọng qua truy vết kết quả test với rủi ro
b) Xác định ảnh hưởng của thay đổi
c) Tự động sinh test case từ yêu cầu
d) Làm báo cáo tiến độ dễ hiểu hơn nhờ kèm trạng thái của phần tử test basis

<details><summary><b>Đáp án: c</b></summary>
<p>a, b, d đều được CTFL nêu ở mục 1.4.4 (cùng với đánh giá coverage, hỗ trợ audit và IT governance). Truy vết không sinh test case — test case đến từ test design.</p>
</details>

**Câu 2 (K2 — FL-1.4.4).** RTM báo 100% yêu cầu có ít nhất một test case, tất cả đều đạt. Kết luận nào hợp lý nhất?

a) Sản phẩm không còn defect liên quan các yêu cầu này
b) Mọi yêu cầu đã có bằng chứng kiểm thử ở mức tối thiểu; chưa biết được kiểm thử kỹ tới mức nào
c) Độ phủ test condition cũng là 100%
d) Không cần phân tích ảnh hưởng khi yêu cầu đổi

<details><summary><b>Đáp án: b</b></summary>
<p>Độ phủ yêu cầu chỉ đo «có liên kết». a vi phạm nguyên tắc «kiểm thử cho thấy sự có mặt của defect, không chứng minh sự vắng mặt». c sai — ví dụ của bài: 92,9% so với 80%, và một yêu cầu xanh vẫn có test condition biên chưa kiểm. d không liên quan.</p>
</details>

**Câu 3 (K2 — FL-1.4.3).** Phần tử nào là **test condition** chứ không phải test case?

a) Huỷ chuyến xe máy lúc 09:03:00 sau khi nhận lúc 09:01:00 ⇒ 10.000 đ, lý do <code>charged</code>
b) Biên 2 phút của mốc miễn phí huỷ
c) Tệp <code>phi-huy.test.ts</code>
d) Dòng log «TC-CANCEL-004 passed»

<details><summary><b>Đáp án: b</b></summary>
<p>b nói «cái gì cần kiểm» — test condition. a có tiền điều kiện, đầu vào, kết quả mong đợi cụ thể — test case. c là test script/test suite (test implementation). d là một mục của test log (test execution).</p>
</details>

**Câu 4 (K2).** Một test case trỏ tới mã yêu cầu đã bị xoá khỏi SPEC phiên bản mới. Cách xử lý đúng nhất?

a) Xoá test ngay vì nó mồ côi
b) Giữ nguyên vì test vẫn xanh
c) Truy vết ngược nội dung test: nếu hành vi vẫn được yêu cầu khác quy định thì gắn lại tag; nếu hành vi không còn được yêu cầu thì xoá hoặc lưu trữ test
d) Thêm lại mã yêu cầu cũ vào SPEC

<details><summary><b>Đáp án: c</b></summary>
<p>Ví dụ TC-CANCEL-090: nội dung (lưu lý do khách nhập) thuộc R-API-11, nên gắn lại tag. a có thể xoá mất một test còn giá trị. b để lại test không ai biết bảo vệ gì. d sửa test basis để hợp với testware — ngược chiều.</p>
</details>

**Câu 5 (K2).** Phạm vi có 14 yêu cầu và 20 test condition. Bộ test phủ 12 yêu cầu và 15 test condition. Hai độ phủ là bao nhiêu?

a) 85,7% và 75%
b) 60% và 107%
c) 85,7% và 60%
d) 75% và 85,7%

<details><summary><b>Đáp án: a</b></summary>
<p>Độ phủ yêu cầu = 12 ÷ 14 ≈ 85,7%. Độ phủ test condition = 15 ÷ 20 = 75%. b chia nhầm chéo (12 ÷ 20 và 15 ÷ 14). c lấy 12 ÷ 20 cho số thứ hai. d đảo hai số.</p>
</details>

**Câu 6 (K2).** Vì sao bài khuyên giữ truy vết trong tiêu đề test thay vì bảng tính riêng?

a) Vì CTFL quy định truy vết phải nằm trong mã
b) Vì liên kết nằm cạnh mã test, đổi cùng commit, được review cùng thay đổi, nên khó lệch hơn bản sao ở nơi khác
c) Vì bảng tính không biểu diễn được quan hệ nhiều-nhiều
d) Vì như vậy không cần test condition nữa

<details><summary><b>Đáp án: b</b></summary>
<p>CTFL chỉ yêu cầu truy vết được duy trì, không quy định hình thức — a sai. c sai: bảng tính biểu diễn được, chỉ khó duy trì. d sai: tag <code>@TCOND-…</code> chính là cách giữ tầng test condition.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB — Certified Tester Foundation Level Syllabus v4.0.1, mục 1.4.3 Testware, LO FL-1.4.3 (K2); mục 1.4.4 Traceability between the Test Basis and Testware, LO FL-1.4.4 (K2); mục 1.4.1 Test Activities and Tasks (bối cảnh): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary (tra *traceability*, *test condition*, *coverage*, *impact analysis*): https://glossary.istqb.org/
- ISO/IEC/IEEE 29119-3 — Software and systems engineering — Software testing — Part 3: Test documentation.
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-STATE-01…08, R-CANCEL-01…07, Phụ lục B; mã bài tại `articles/test-case-design-traceability-matrix/`.
- Bài liên quan: [TEST-010 · Testware](/articles/testware-san-pham-kiem-thu) · [TCD-01 · Test analysis và test condition](/articles/test-case-design-test-analysis-test-condition) · [TCD-02 · Giải phẫu test case](/articles/test-case-design-giai-phau-test-case) · [TCD-07 · User story và acceptance criteria](/articles/test-case-design-user-story-acceptance-criteria) · [TEST-033 · Decision Table](/articles/decision-table-testing) · [TEST-034 · State Transition](/articles/state-transition-testing-co-ban) · [Maintenance testing](/articles/maintenance-testing).

Bài tiếp theo: [TCD-09 · Ưu tiên, review và bảo trì test case](/articles/test-case-design-uu-tien-review-bao-tri).
