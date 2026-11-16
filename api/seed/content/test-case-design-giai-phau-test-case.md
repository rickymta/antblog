---
slug: test-case-design-giai-phau-test-case
title: "Giải phẫu một test case: từng trường và các lỗi viết hay gặp, qua lệnh nhận chuyến của Tới Nơi"
summary: "Mổ xẻ một test case trường theo trường: ID, tiêu đề, mục tiêu, tiền điều kiện, test data, bước, kết quả mong đợi, hậu điều kiện, ưu tiên, truy vết, môi trường. Bốn quy tắc viết, bảy cặp sửa từ dở sang tốt cho R-ACCEPT, schema zod, bộ lint và 25 test vitest."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-case-design, tcd-02, test-case, iso-29119, precondition, expected-result, zod, vitest, toi-noi]
language: vi
---

«Kiểm tra tài xế nhận chuyến. Bước: tài xế bấm nhận. Kết quả: hệ thống hoạt động đúng.» Đưa test case này cho ba tester, bạn sẽ nhận ba kết quả. Người thứ nhất dùng tài xế D101 và thấy chuyến chuyển sang `accepted`, ghi «đạt». Người thứ hai dùng D104 (đang offline), nhận lỗi 409 và cũng ghi «đạt», vì «báo lỗi là đúng». Người thứ ba hỏi lại: «chuyến nào? lúc mấy giờ?». Test case không sai về nội dung; nó chỉ **không đủ thông tin để hai người chạy ra cùng một kết luận**. Bài này mổ xẻ một test case theo từng trường. Ở mỗi trường, ta xem trường đó trả lời câu hỏi gì, viết nó thế nào cho đúng, và lỗi hay gặp là gì. Ví dụ là lệnh nhận chuyến của tài xế Tới Nơi (SPEC R-ACCEPT). Lab biến các quy tắc viết thành một schema zod và một bộ lint, rồi chạy năm test case viết tốt trên ứng dụng thật.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TCD-02</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>TCD · Thiết kế Test Case</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1; tham khảo ISO/IEC/IEEE 29119-3</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>System (qua API)</td></tr>
<tr><th scope="row">Test Type</th><td>Functional</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng (bài về cấu trúc và cách viết test case)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/test-case-design-test-analysis-test-condition">TCD-01 · Test analysis và test condition</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Schema zod cho tài liệu test case, bộ lint 7 luật, 5 test case R-ACCEPT viết tốt chạy qua HTTP, 7 bản viết dở bị lint bắt: <code>examples/toi-noi-testing/articles/test-case-design-giai-phau-test-case/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.4.1, 1.4.3, 1.4.4 · FL-1.4.1 (K2), FL-1.4.3 (K2), FL-1.4.4 (K2). Tham khảo: ISO/IEC/IEEE 29119-3 (test case specification)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

[TCD-01](/articles/test-case-design-test-analysis-test-condition) dừng ở danh sách test condition: **cái gì** cần kiểm. Test design biến mỗi condition thành một hay nhiều test case: **kiểm thế nào**, với dữ liệu nào, mong đợi điều gì. Ở bước này, chất lượng của **văn bản** test case quyết định chất lượng của cả bộ test. Một test case viết dở gây ra bốn loại chi phí, dù sản phẩm có đúng hay không:

- **Kết luận không lặp lại được.** Hai lần chạy ra hai kết quả mà sản phẩm không đổi. Đội mất niềm tin vào bộ test và bắt đầu bỏ qua test đỏ.
- **Bỏ lọt defect.** Kết quả mong đợi mơ hồ nên test ghi «đạt» cho một hành vi sai. Ví dụ «báo lỗi» đạt cả khi lỗi trả về sai mã.
- **Tốn công điều tra.** Test đỏ nhưng không rõ test đang chứng minh điều gì, nên phải đọc lại đặc tả, đọc code, hỏi người viết.
- **Không bảo trì được.** Đặc tả đổi nhưng không biết test case nào bị ảnh hưởng, vì test case không ghi nguồn.

Không phải test case nào cũng cần đủ mười mấy trường. Một test case tự động trong vitest có thể gói tiền điều kiện vào `beforeEach` và kết quả mong đợi vào `expect`. Một buổi exploratory testing dùng test charter, không dùng test case. Nhưng ngay cả khi rút gọn, **các câu hỏi mà từng trường trả lời vẫn phải có câu trả lời ở đâu đó**. Hiểu đủ bộ trường giúp bạn rút gọn có chủ đích, thay vì rút gọn vì quên.

## 2. Lý thuyết theo ISTQB

### 2.1. Test case trong CTFL v4.0.1

ISTQB Glossary định nghĩa test case là **một bộ gồm tiền điều kiện, đầu vào, hành động (khi cần), kết quả mong đợi và hậu điều kiện, được xây dựng dựa trên test condition**. Định nghĩa này có ba ý đáng chú ý:

1. Test case **đến từ test condition**, không đến thẳng từ yêu cầu. Lớp condition ở giữa cho phép đo độ phủ và truy vết (TCD-01).
2. **Kết quả mong đợi** là thành phần bắt buộc. Không có nó, ta chỉ có một chuỗi thao tác, chưa có test.
3. **Tiền điều kiện và hậu điều kiện** là một phần của test case, không phải ghi chú phụ. Hai test case cùng đầu vào nhưng khác tiền điều kiện là hai test case khác nhau.

Trong các hoạt động kiểm thử của CTFL mục 1.4.1, test case là sản phẩm của **test design**. Test design còn nhận diện coverage item, xác định yêu cầu về test data và môi trường. Theo mục 1.4.3, testware của test design gồm **test case đã được ưu tiên**, test charter, coverage item, yêu cầu về test data và yêu cầu về test environment. Hoạt động kế tiếp, test implementation, mới gom test case thành test procedure, test suite và test execution schedule ([TCD-03](/articles/test-case-design-logic-va-cu-the)).

### 2.2. ISO/IEC/IEEE 29119-3 làm khung tham khảo

CTFL không quy định mẫu test case. Bộ chuẩn ISO/IEC/IEEE 29119 thì có: phần 3 (Test documentation) mô tả tài liệu **test case specification**, trong đó mỗi test case có định danh duy nhất, mục tiêu, mức ưu tiên, truy vết, tiền điều kiện, đầu vào và kết quả mong đợi. Chuẩn tách riêng tài liệu **test procedure specification** cho trình tự thực hiện. Bài này dùng chuẩn làm **khung đối chiếu**, không làm mẫu bắt buộc. Danh sách trường ở mục 4 bám sát tinh thần của chuẩn và thêm vài trường hữu ích trong thực tế (phiên bản test basis, môi trường), nhưng tên trường và cách trình bày là của bài.

Có hai lý do để biết chuẩn này dù đội bạn không theo nó. Một là khi làm việc với đối tác hoặc ngành có kiểm toán (ngân hàng, y tế), họ thường hỏi «tài liệu test của các anh tương ứng mục nào của 29119-3». Hai là chuẩn tách test case và test procedure. Nhiều mẫu test case trong công cụ quản lý test gộp hai thứ này, và sự gộp đó là nguồn của nhiều lỗi viết ở mục 4.13.

### 2.3. Truy vết ở mức test case (mục 1.4.4)

CTFL mục 1.4.4 yêu cầu truy vết giữa phần tử của test basis, testware (test condition, rủi ro, test case), kết quả test và defect. Ở mức test case, điều đó có nghĩa là mỗi test case **ghi rõ nó phủ yêu cầu nào và test condition nào**. Khi test case đỏ, truy vết ngược cho biết yêu cầu nào bị vi phạm. Khi yêu cầu đổi, truy vết xuôi cho biết test case nào cần sửa. Truy vết chỉ có giá trị khi được duy trì. Vì vậy lab kiểm mã yêu cầu trong mỗi test case với `SPEC.md` mỗi lần chạy: một test case trỏ tới «R-ACCEPT-09» (không tồn tại) bị lint bắt ngay.

### 2.4. Thuật ngữ Anh – Việt

| Thuật ngữ | Cách dùng trong bài |
|---|---|
| test case | giữ nguyên tiếng Anh |
| tiền điều kiện (precondition) | trạng thái hệ thống và môi trường **trước** bước đầu tiên |
| hậu điều kiện (postcondition) | trạng thái **sau** khi test case chạy xong |
| kết quả mong đợi (expected result) | hành vi quan sát được mà test object phải tạo ra, suy từ test basis |
| test data | dữ liệu cần để chạy test case; mỗi dòng có mã `TD-nn` |
| ca dương / ca âm (positive / negative test case) | ca dương kiểm hành vi hợp lệ; ca âm kiểm hệ thống từ chối đúng cách |
| test case specification | tài liệu đặc tả test case (ISO/IEC/IEEE 29119-3) |
| test procedure | trình tự bước thực hiện một hoặc nhiều test case (TCD-03) |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Danh sách 12 trường ở mục 4, quy ước mã <code>TC-ACCEPT-nnn</code>, bốn quy tắc viết, bộ lint <code>TC-L01…07</code> và việc biểu diễn test case bằng schema zod là thực hành ngành và lựa chọn của bài. CTFL chỉ định nghĩa test case, đặt nó vào test design và yêu cầu truy vết. Mẫu tài liệu đầy đủ có trong ISO/IEC/IEEE 29119-3; bài không chép mẫu đó.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu: tài xế nhận chuyến

Tài xế Tới Nơi xem danh sách chuyến đang tìm tài xế cùng thành phố và đúng loại xe, rồi bấm nhận (SPEC R-ACCEPT-06: điều phối kiểu kéo, không gửi offer). Test basis:

| Mã | Quy tắc (tóm tắt SPEC v1.1) |
|---|---|
| R-ACCEPT-01 | nhận được khi đồng thời: tài xế `online`, cùng thành phố, đúng loại xe, chuyến đang `searching` |
| R-ACCEPT-02 | thứ tự kiểm và mã lỗi: `TRIP_NOT_FOUND` (404) → `DRIVER_OFFLINE` (409) → `DRIVER_BUSY` (409) → `CITY_MISMATCH` (422) → `VEHICLE_MISMATCH` (422) → chuyến đã huỷ `INVALID_TRANSITION` (409) → đã có tài xế `TRIP_ALREADY_TAKEN` (409) |
| R-ACCEPT-03 | nhận đồng thời: đúng một người thắng, người thua nhận `TRIP_ALREADY_TAKEN` và vẫn `online` |
| R-ACCEPT-04 | lệnh nhận gửi `etaMinutes` (nguyên 1–60), lưu cùng `acceptedAt` |
| R-ACCEPT-05 | nhận thành công ⇒ tài xế `busy` |
| R-ACCEPT-06 | danh sách chuyến chờ: `requestedAt` tăng dần, rồi `id` tăng dần |
| R-ACCEPT-07 | gửi lại lệnh nhận cho chuyến mình đang giữ ⇒ 200, không đổi gì (giữ `acceptedAt` và `etaMinutes` cũ) |

API: `POST /api/trips/:id/accept`, header `x-user: driver:<id>`, thân `{ "etaMinutes": n }` (SPEC R-API-10). Dữ liệu mẫu (Phụ lục A): D101, D102 là xe máy SG `online`; D103 ô tô SG `online`; D104 xe máy SG `offline`; D201/D202 xe máy/ô tô HN `online`.

### 3.2. System context

Trong kiến trúc đích, lệnh nhận chạm hai service: **Matching & Dispatch** giành chuyến (`searching` → `accepted`) và **Driver Availability** chuyển tài xế `online` → `busy`. Hai thay đổi phải nhất quán: không được có chuyến `accepted` mà tài xế vẫn `online`, hay tài xế `busy` mà không giữ chuyến nào (R-ACCEPT-05). Lab gộp cả hai vào `acceptTrip` trong `src/app/trips.ts`, dùng hai cập nhật có điều kiện (`tryMarkBusy`, `acceptIfSearching`). Hệ quả cho test case: kết quả mong đợi phải kiểm **cả chuyến lẫn tài xế**. Một test case chỉ đọc response HTTP sẽ bỏ sót nửa còn lại.

### 3.3. Rủi ro

| Rủi ro | Hậu quả | Test case phải quan sát |
|---|---|---|
| hai tài xế cùng giữ một chuyến | hai xe tới đón một khách; một tài xế chạy không công | `driverId` sau lệnh thua; trạng thái tài xế thua |
| tài xế kẹt ở `busy` sau khi thua | không nhận được chuyến nào nữa cho tới khi hỗ trợ can thiệp | trạng thái tài xế sau lệnh bị từ chối |
| sai thứ tự lỗi | app tài xế hiển thị lý do sai («sai loại xe» thay vì «sai thành phố») | mã lỗi khi vi phạm nhiều điều kiện |
| bấm nhận hai lần ghi đè ETA | khách thấy ETA nhảy; mốc phí huỷ R-CANCEL-04 bị dời | `etaMinutes`, `acceptedAt` sau lần nhận thứ hai |

### 3.4. Test condition (đầu ra của test analysis)

Theo cách làm của TCD-01, phân tích R-ACCEPT cho 13 condition. Bài chỉ cần chúng làm điểm truy vết:

| Mã | Test condition | Nguồn |
|---|---|---|
| TCOND-ACCEPT-01 | Tài xế online, cùng thành phố, đúng loại xe nhận được chuyến đang `searching` | R-ACCEPT-01, -05 |
| TCOND-ACCEPT-02 | Tài xế `offline` bị từ chối với `DRIVER_OFFLINE` | R-ACCEPT-02 |
| TCOND-ACCEPT-03 | Tài xế `busy` bị từ chối với `DRIVER_BUSY` | R-ACCEPT-02 |
| TCOND-ACCEPT-04 | Khác thành phố bị từ chối với `CITY_MISMATCH` | R-ACCEPT-02 |
| TCOND-ACCEPT-05 | Khác loại xe bị từ chối với `VEHICLE_MISMATCH` | R-ACCEPT-02 |
| TCOND-ACCEPT-06 | Chuyến đã có tài xế bị từ chối với `TRIP_ALREADY_TAKEN`; tài xế bị từ chối vẫn `online` | R-ACCEPT-02, -03 |
| TCOND-ACCEPT-07 | Chuyến đã huỷ bị từ chối với `INVALID_TRANSITION` | R-ACCEPT-02 |
| TCOND-ACCEPT-08 | Vi phạm nhiều điều kiện thì chỉ báo lỗi đứng trước theo thứ tự R-ACCEPT-02 | R-ACCEPT-02 |
| TCOND-ACCEPT-09 | `etaMinutes` và `acceptedAt` được lưu khi nhận | R-ACCEPT-04, R-STATE-07 |
| TCOND-ACCEPT-10 | Nhận lại chuyến mình đang giữ trả lại phân công cũ, không đổi gì | R-ACCEPT-07 |
| TCOND-ACCEPT-11 | Nhiều tài xế nhận đồng thời: đúng một người thắng | R-ACCEPT-03 |
| TCOND-ACCEPT-12 | Chuyến không tồn tại bị từ chối với `TRIP_NOT_FOUND` | R-ACCEPT-02 |
| TCOND-ACCEPT-13 | `etaMinutes` ngoài 1–60 hoặc không nguyên bị từ chối với `VALIDATION_ERROR` | R-ACCEPT-04, R-API-06 |

## 4. Phân tích kỹ thuật: giải phẫu từng trường

Hình 1 chia một test case thành bốn khối. Mỗi khối phục vụ một người đọc khác nhau: người quản lý đọc **đầu**, người chạy đọc **chuẩn bị** và **thân**, người chạy test case kế tiếp đọc **kết thúc**.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 360" role="img" aria-label="Giải phẫu một test case gồm bốn khối: đầu (ID, tiêu đề, mục tiêu, ưu tiên, truy vết, phiên bản test basis) trả lời test case này là gì và vì sao tồn tại; chuẩn bị (môi trường, tiền điều kiện, test data) trả lời phải dựng gì trước; thân (bước và kết quả mong đợi, ghép từng cặp) trả lời làm gì và quan sát gì; kết thúc (hậu điều kiện) trả lời để lại gì cho test case sau.">
<defs>
<marker id="tcd02-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="20" width="560" height="80" rx="8" class="f-pk-soft s-pk"/>
<text x="36" y="44" class="t-b t-pk">ĐẦU</text>
<text x="36" y="66" class="t-10 t-mono">ID · tiêu đề · mục tiêu · ưu tiên</text>
<text x="36" y="86" class="t-10 t-mono">truy vết (R-…, TCOND-…) · phiên bản test basis</text>
<rect x="20" y="112" width="560" height="80" rx="8" class="f-surface s-line"/>
<text x="36" y="136" class="t-b">CHUẨN BỊ</text>
<text x="36" y="158" class="t-10 t-mono">môi trường · tiền điều kiện (trạng thái, không phải lịch sử)</text>
<text x="36" y="178" class="t-10 t-mono">test data (TD-nn)</text>
<rect x="20" y="204" width="560" height="92" rx="8" class="f-phys-soft s-phys"/>
<text x="36" y="228" class="t-b t-phys">THÂN</text>
<rect x="36" y="238" width="250" height="46" rx="6" class="f-surface s-phys"/>
<text x="161" y="258" text-anchor="middle" class="t-10">bước: một hành động</text>
<text x="161" y="274" text-anchor="middle" class="t-10">dùng TD-nn</text>
<rect x="314" y="238" width="250" height="46" rx="6" class="f-surface s-phys"/>
<text x="439" y="258" text-anchor="middle" class="t-10">kết quả mong đợi: quan sát được,</text>
<text x="439" y="274" text-anchor="middle" class="t-10">suy từ test basis, tất định</text>
<line x1="286" y1="261" x2="312" y2="261" class="s-muted" stroke-width="1.4" marker-end="url(#tcd02-a)"/>
<rect x="20" y="308" width="560" height="40" rx="8" class="f-surface s-line dash"/>
<text x="36" y="333" class="t-b">KẾT THÚC</text>
<text x="150" y="333" class="t-10 t-mono">hậu điều kiện · cách dọn</text>
<text x="610" y="56" class="t-small t-b">Quản lý, reviewer, kiểm toán</text>
<text x="610" y="74" class="t-small">«test case này là gì, vì sao tồn tại,</text>
<text x="610" y="90" class="t-small">nó che rủi ro nào?»</text>
<text x="610" y="148" class="t-small t-b">Người chạy, script</text>
<text x="610" y="166" class="t-small">«phải dựng gì trước bước 1?»</text>
<text x="610" y="244" class="t-small t-b">Người chạy, script</text>
<text x="610" y="262" class="t-small">«làm gì, nhìn vào đâu,</text>
<text x="610" y="278" class="t-small">thế nào là đạt?»</text>
<text x="610" y="328" class="t-small t-b">Test case kế tiếp, CI</text>
<text x="610" y="346" class="t-small">«hệ thống còn lại trạng thái gì?»</text>
<line x1="582" y1="60" x2="604" y2="60" class="s-muted" stroke-width="1.2" marker-end="url(#tcd02-a)"/>
<line x1="582" y1="152" x2="604" y2="152" class="s-muted" stroke-width="1.2" marker-end="url(#tcd02-a)"/>
<line x1="582" y1="250" x2="604" y2="250" class="s-muted" stroke-width="1.2" marker-end="url(#tcd02-a)"/>
<line x1="582" y1="330" x2="604" y2="330" class="s-muted" stroke-width="1.2" marker-end="url(#tcd02-a)"/>
</svg>
</div>
<figcaption><b>Hình 1 · Bốn khối của một test case và người đọc của từng khối.</b> Thiếu một trường nghĩa là một câu hỏi của một người đọc không có câu trả lời, và người đó sẽ tự đoán.</figcaption>
</figure>

### 4.1. ID: định danh không mang nghĩa nghiệp vụ

`TC-ACCEPT-001`. Ba phần: tiền tố loại testware (`TC`), nhóm (`ACCEPT`, trùng nhóm R-… cho dễ lần), số thứ tự ba chữ số. Quy tắc:

- **Duy nhất và bất biến.** Đổi tiêu đề, đổi bước, đổi dữ liệu thì ID giữ nguyên. Defect report, test log và RTM trỏ vào ID; đổi ID là cắt đứt truy vết.
- **Không tái sử dụng.** Test case bị xoá thì số của nó bị bỏ trống, không gán cho test case mới. Nếu gán lại, một defect cũ trỏ tới `TC-ACCEPT-004` sẽ trỏ nhầm sang một test case khác hẳn.
- **Không nhồi ý nghĩa.** Đừng mã hoá ưu tiên hay test level vào ID (`TC-ACCEPT-H-SYS-001`). Những thuộc tính đó thay đổi; ID thì không được thay đổi.

Lab dùng dải `9xx` cho bộ sưu tập test case viết dở, để không lẫn với test case thật.

### 4.2. Tiêu đề: tình huống và kết quả trong một dòng

Tiêu đề là thứ người ta đọc trong danh sách 300 test case và trong thông báo CI. Nó nên nói **tình huống** và **kết quả** đủ để đoán được test đang kiểm gì mà không cần mở ra:

| Dở | Tốt |
|---|---|
| Test nhận chuyến 2 | Tài xế offline nhận chuyến bị từ chối với DRIVER_OFFLINE |
| Kiểm tra lỗi nhận chuyến | Tài xế khác cả thành phố lẫn loại xe nhận được lỗi CITY_MISMATCH |
| Nhận chuyến (happy case) | Tài xế online cùng thành phố, đúng loại xe nhận được chuyến đang tìm tài xế |

Tiêu đề không thay cho mục tiêu và kết quả mong đợi. Nó chỉ là bản tóm tắt đủ để lọc và đọc lướt. Schema của lab giới hạn 10–120 ký tự: ngắn hơn thì gần như chắc chắn thiếu thông tin, dài hơn thì nên chuyển bớt sang mục tiêu.

### 4.3. Mục tiêu: một test case, một điều cần chứng minh

Mục tiêu trả lời «nếu test case này đạt, ta biết thêm điều gì về sản phẩm?». Nó gần với test condition, nhưng nói ở ngôi của test case: «Chứng minh tài xế đang offline không nhận được chuyến và chuyến không bị thay đổi.»

**Một mục tiêu không có nghĩa là một assertion.** TC-ACCEPT-001 có ba dòng kết quả mong đợi (response, ETA và `acceptedAt`, trạng thái tài xế). Cả ba cùng phục vụ một mục tiêu: chứng minh lệnh nhận hợp lệ được ghi nhận đầy đủ. Ngược lại, «kiểm tra tài xế offline bị từ chối **và** kiểm tra tài xế busy bị từ chối» là hai mục tiêu, vì hai vế có thể đỏ độc lập. Khi test case đó đỏ, người đọc không biết vế nào sai. Tệ hơn, vế thứ hai có thể không bao giờ chạy nếu vế đầu dừng test.

### 4.4. Ưu tiên

Ưu tiên của test case **kế thừa từ rủi ro** của test condition mà nó phủ (TCD-01 mục 4.7), có điều chỉnh theo phụ thuộc và chi phí chạy. TC-ACCEPT-001 (đường đi chính), -002 và -004 (chặn tài xế không hợp lệ, chặn giành chuyến) là cao. -003 (thứ tự lỗi) và -005 (idempotent) là trung bình. Thứ tự chạy cụ thể và các chiến lược ưu tiên test case là đề tài của [TCD-09](/articles/test-case-design-uu-tien-review-bao-tri). Ở đây chỉ cần trường này **có giá trị và có lý do**.

### 4.5. Truy vết và phiên bản test basis

```text
Truy vết:  R-ACCEPT-01, R-ACCEPT-04, R-ACCEPT-05, R-STATE-07 · TCOND-ACCEPT-01, TCOND-ACCEPT-09
Test basis: SPEC v1.1 · fare-2026.1
```

Trường truy vết có hai tầng: **yêu cầu** (để báo cáo theo yêu cầu, phân tích ảnh hưởng) và **test condition** (để đo độ phủ condition). Phiên bản test basis cho biết kết quả mong đợi được suy từ bản đặc tả nào. Khi SPEC lên v1.2, test case ghi v1.1 là ứng viên cần review. Không có trường này, ta không phân biệt được «test đỏ vì sản phẩm sai» với «test đỏ vì test case lỗi thời». Cách dựng RTM đầy đủ từ các trường này ở [TCD-08](/articles/test-case-design-traceability-matrix).

### 4.6. Môi trường

Môi trường trả lời «chạy ở đâu thì kết quả mong đợi này đúng?». Với lab: Node 22, `createApp` với kho trong bộ nhớ, đồng hồ cố định, id tuần tự, gọi qua supertest. Môi trường ảnh hưởng kết quả nhiều hơn ta nghĩ. R-ACCEPT-03 (đồng thời) cho kết quả khác trên kho trong bộ nhớ, trên PGlite một kết nối, và trên PostgreSQL thật nhiều kết nối (README của lab nói rõ giới hạn này). Test case nào phụ thuộc môi trường thì phải ghi môi trường, nếu không «đạt trên máy tôi» sẽ thành câu trả lời mặc định.

### 4.7. Tiền điều kiện: trạng thái, không phải lịch sử

Tiền điều kiện mô tả **trạng thái** hệ thống phải có ngay trước bước 1:

```text
- Hệ thống vừa reset về dữ liệu mẫu (SPEC Phụ lục A)
- Đồng hồ hệ thống cố định ở 09:00:00 giờ VN ngày 04/11/2026 (2026-11-04T02:00:00.000Z)
- Khách C001 đã tạo chuyến T1 từ báo giá ride, SG, bike, 5 km; T1 đang ở searching, chưa có tài xế
- Tài xế D104 (SG, bike) đang offline
```

Bốn điểm hay sai:

1. **Viết lịch sử thay vì trạng thái.** «Đã chạy TC-ACCEPT-001» buộc test case này phụ thuộc test case khác: chạy riêng thì thiếu dữ liệu, chạy sau một test case thất bại thì sai dữ liệu. Viết trạng thái («D101 đã nhận T1 với ETA 7 phút») rồi dựng trạng thái đó trực tiếp.
2. **Quên đồng hồ.** Lệnh nhận ghi `acceptedAt`. Không cố định đồng hồ thì không viết được kết quả mong đợi cho `acceptedAt`, và test case sẽ «lặp lại được» chỉ trong cùng một phút. Đồng hồ giả (`FixedClock`) là một phần của tiền điều kiện, không phải chi tiết cài đặt.
3. **Quên trạng thái của thực thể phụ.** Test TC-ACCEPT-002 cần D104 đang `offline`. Dữ liệu mẫu đúng là như vậy, nhưng ghi ra thì người chạy kiểm lại được trước khi bắt đầu, và khi dữ liệu mẫu đổi, test case báo ngay chỗ lệch thay vì sai mà không ai hay.
4. **Tiền điều kiện không thoả thì không phải «không đạt».** Nếu không dựng được trạng thái (ví dụ API báo giá đang lỗi), kết quả là **blocked**, không phải **fail**. Phân biệt hai trạng thái này giúp báo cáo đúng: fail là thông tin về test object, blocked là thông tin về môi trường hoặc phụ thuộc.

### 4.8. Test data

Test data tách khỏi bước. Mỗi dòng có mã, mô tả **vì sao** chọn giá trị đó, và giá trị:

| Dòng | Vì sao chọn | Giá trị |
|---|---|---|
| TD-01 | ETA hợp lệ ở giữa khoảng 1–60 | `driver: D101`, `etaMinutes: 7` |
| TD-05 | ETA **khác** lần nhận đầu, để thấy giá trị cũ được giữ | `driver: D101`, `etaMinutes: 15` |

Cột «vì sao» quan trọng hơn vẻ ngoài của nó. TD-05 chọn 15 thay vì 7 không phải ngẫu nhiên: nếu lần nhận thứ hai cũng gửi 7, một bản cài đặt ghi đè ETA vẫn trả 7 và test xanh dù sai. Dữ liệu phải có khả năng **phân biệt** hành vi đúng với hành vi sai. Ở lab, test data nằm ngay trong tài liệu test case và script đọc từ đó (mục 5.5), nên sửa dữ liệu ở một chỗ. Cách dựng test data cho từng test case, builder, fixture và đồng hồ là đề tài của [TCD-05](/articles/test-case-design-test-data).

### 4.9. Bước

Mỗi bước là **một hành động** mà người chạy hoặc script thực hiện được, ở thể mệnh lệnh, đủ chi tiết để không phải đoán:

```text
1. Gửi POST /api/trips/T1/accept với header x-user: driver:D101, thân {"etaMinutes":7}  (TD-01)
2. Đọc trạng thái tài xế D101 trong kho dữ liệu
```

Ba lỗi hay gặp ở bước:

- **Bước chứa phán đoán.** «Kiểm tra xem chuyến đã được nhận đúng chưa» không phải hành động; nó là kết quả mong đợi viết lẫn vào bước. Tách ra: bước «đọc chuyến T1», kết quả mong đợi «T1 ở `accepted`».
- **Bước quá thô.** «Tài xế nhận chuyến» không cho biết qua kênh nào (app, API), tài xế nào, ETA bao nhiêu.
- **Bước quá mịn cho mục đích.** Với test case thủ công trên app, «bấm nút Nhận» là đủ; liệt kê toạ độ nút là việc của script. Mức chi tiết của bước phụ thuộc người chạy: người mới cần nhiều hơn người quen hệ thống. Bước của test case logic và test case cụ thể khác nhau như thế nào là đề tài của TCD-03.

Bước 2 của TC-ACCEPT-001 đáng chú ý: nó là bước **quan sát**, không gây thay đổi. Kết quả mong đợi «D101 ở `busy`» cần một điểm quan sát. Nếu test object không cho đọc trạng thái tài xế qua API, đó là vấn đề testability nên nêu ra từ test analysis. Lab đọc trực tiếp kho dữ liệu, tương đương một truy vấn kiểm tra database trong system test.

### 4.10. Kết quả mong đợi

Đây là trường quyết định test case có giá trị hay không. Bốn tính chất:

1. **Quan sát được và cụ thể.** «HTTP 409; `error.code` = `DRIVER_OFFLINE`» thay vì «báo lỗi phù hợp». Mỗi dòng nên neo vào ít nhất một giá trị so được: một con số, một mã, một trạng thái.
2. **Suy từ test basis.** 409 và `DRIVER_OFFLINE` đến từ R-ACCEPT-02 và R-API-06, không đến từ việc chạy thử rồi chép lại. Chép kết quả từ hệ thống đang chạy biến mọi lỗi hiện có thành «đúng».
3. **Đầy đủ các đầu ra liên quan.** Lệnh bị từ chối phải kiểm cả **những gì không được thay đổi**: T1 vẫn `searching`, `driverId` vẫn null, D104 vẫn `offline`. Một bản cài đặt có thể trả đúng 409 nhưng đã lỡ đánh dấu tài xế `busy`; chỉ kiểm response sẽ bỏ lọt đúng rủi ro «tài xế kẹt busy» ở mục 3.3.
4. **Tất định.** Không có «có thể», «tuỳ», «khoảng». Nếu đặc tả cho phép nhiều kết quả, test case phải cố định đủ điều kiện để chỉ còn một kết quả, hoặc tách thành nhiều test case.

Tính chất thứ ba thường bị bỏ qua nhất. [TCD-04](/articles/test-case-design-expected-result-test-oracle) đi sâu vào việc liệt kê mọi đầu ra quan sát được (response, trạng thái lưu, nhật ký, lời gọi ra ngoài) và các nguồn test oracle.

### 4.11. Hậu điều kiện

Hậu điều kiện mô tả hệ thống **sau** test case: «T1 ở `accepted` với D101; D101 `busy`, dọn bằng reset». Trường này có hai công dụng. Với người chạy thủ công, nó cho biết phải dọn gì trước test case kế tiếp. Nếu không dọn, D101 sẽ `busy` và mọi test case sau dùng D101 đều đỏ vì lý do không liên quan. Với người xếp test procedure (TCD-03), hậu điều kiện của test case này có thể **là** tiền điều kiện của test case khác. Dùng chung trạng thái giúp tiết kiệm thời gian chạy, nhưng đổi lại mất tính độc lập, nên phải là quyết định có ý thức, không phải tai nạn.

Ca âm thường có hậu điều kiện «không có thay đổi dữ liệu». Viết ra vẫn có ích: nó nhắc rằng kết quả mong đợi phải kiểm điều đó.

### 4.12. Bốn quy tắc viết

| Quy tắc | Câu hỏi kiểm tra | Vi phạm điển hình |
|---|---|---|
| **Một mục tiêu** | Nếu test case đỏ, có biết ngay điều gì sai không? | «kiểm offline và kiểm busy» |
| **Độc lập** | Chạy riêng test case này, theo bất kỳ thứ tự nào, có ra cùng kết quả không? | «tiền điều kiện: đã chạy TC-001» |
| **Lặp lại được** | Chạy lại ngày mai, trên máy khác, có ra cùng kết quả không? | không cố định đồng hồ; dùng «một tài xế đang rảnh bất kỳ» |
| **Kết quả mong đợi không mơ hồ** | Hai người đọc có kết luận giống nhau không? | «hệ thống hoạt động đúng»; «409 hoặc 422 tuỳ trường hợp» |

Cộng thêm một quy tắc nền: **truy vết được**. Test case không trỏ tới yêu cầu nào thì hoặc nó kiểm một điều chưa có trong đặc tả (cần báo lên), hoặc nó thừa.

### 4.13. Bộ sưu tập: từ dở sang tốt

Bảy bản dở dưới đây đều lấy TC-ACCEPT-002 (tài xế offline) làm khuôn, mỗi bản hỏng **đúng một chỗ**. Cột cuối là luật lint của lab bắt được nó (mục 5.3).

| # | Bản dở | Vấn đề | Bản tốt | Lint |
|---|---|---|---|---|
| 901 | Kết quả: «Hệ thống hoạt động đúng» | mơ hồ, không quan sát được | «HTTP 409; `error.code` = `DRIVER_OFFLINE`» | L01, L02 |
| 902 | Kết quả: «Tài xế nhận được thông báo lỗi phù hợp» | không neo vào giá trị nào; «phù hợp» do người chạy tự hiểu | như trên, cộng «T1 vẫn `searching`, `driverId` = null» | L02 |
| 903 | Tiền điều kiện: (trống) | không biết dữ liệu, đồng hồ, trạng thái tài xế | bốn dòng tiền điều kiện ở mục 4.7 | L03 |
| 904 | Mục tiêu: «Kiểm tra tài xế offline bị từ chối và kiểm tra tài xế busy bị từ chối» | hai mục tiêu | tách thành TC-ACCEPT-002 (offline) và một test case riêng cho `DRIVER_BUSY` | L04 |
| 905 | Tiền điều kiện: «Đã chạy xong TC-ACCEPT-001 (T1 có tài xế)» | phụ thuộc test case khác | mô tả trạng thái và dựng trực tiếp | L05 |
| 906 | Kết quả: «HTTP 409 hoặc 422 tuỳ trạng thái tài xế, có thể kèm `DRIVER_OFFLINE`» | không tất định, che chỗ người viết chưa đọc kỹ R-ACCEPT-02 | cố định tài xế offline ⇒ 409 `DRIVER_OFFLINE` | L06 |
| 907 | Truy vết: R-ACCEPT-09 | mã không tồn tại trong SPEC | R-ACCEPT-01, R-ACCEPT-02 | L07 |

Bản 906 là ví dụ đáng suy nghĩ. Người viết không chắc tài xế offline nhận 409 hay 422, nên viết cả hai. Kết quả mong đợi mơ hồ thường là **triệu chứng của test basis chưa được đọc kỹ**, hoặc của test basis mơ hồ. Ở trường hợp này, R-ACCEPT-02 và R-API-06 trả lời rõ (409). Nếu đặc tả thật sự không trả lời, đó là câu hỏi gửi BA theo cách của TCD-01 mục 4.6, không phải lý do để viết «hoặc».

## 5. Thực hành

### 5.1. Tổ chức lab

Lab: [`articles/test-case-design-giai-phau-test-case`](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/test-case-design-giai-phau-test-case). Không sửa `src/`.

| Tệp | Vai trò |
|---|---|
| `test-case-schema.ts` | schema zod: test case **đúng dạng** |
| `lint-test-case.ts` | 7 luật lint: test case **viết tốt** |
| `tc-accept.ts` | 5 test case tốt `TC-ACCEPT-001…005`, 7 bản dở `TC-ACCEPT-901…907` |
| `test-case-lint.test.ts` | 20 test: bản tốt khớp schema và sạch lint; bản dở khớp schema nhưng bị lint bắt đúng luật |
| `tc-accept-execution.test.ts` | 5 test: chạy 5 test case tốt qua HTTP |

### 5.2. Schema: đúng dạng

```ts
// test-case-schema.ts (trích)
export const testCaseSchema = z.object({
  id: z.string().regex(/^TC-[A-Z]+-\d{3}$/, 'mã test case dạng TC-NHOM-nnn'),
  title: z.string().min(10).max(120),
  objective: z.string().min(10),
  priority: z.enum(['cao', 'trung-binh', 'thap']),
  traceability: z.object({
    requirements: z.array(z.string().regex(/^R-[A-Z]+-\d{2}$/)).min(1),
    conditions: z.array(z.string().regex(/^TCOND-[A-Z]+-\d{2}$/)),
  }),
  basisVersion: z.string().min(1),
  environment: z.string().min(1),
  preconditions: z.array(z.string().min(1)),
  testData: z.array(testDataRowSchema),
  steps: z.array(stepSchema).min(1),
  expectedResults: z.array(z.string().min(1)).min(1),
  postconditions: z.array(z.string().min(1)),
});
```

Schema bắt lỗi **cấu trúc**: thiếu trường, sai kiểu, sai quy ước mã, không có bước, không truy vết yêu cầu nào. Test trong lab đưa vào một bản có ID `TC1`, mảng bước rỗng và không truy vết, rồi kiểm schema báo đúng ba đường dẫn trường: `id`, `steps`, `traceability.requirements`.

Schema **cố ý không** bắt mọi thứ. `preconditions` được phép rỗng, vì có test case thật sự không cần tiền điều kiện (ví dụ gọi `GET /api/health`). Kết quả mong đợi «hệ thống hoạt động đúng» là chuỗi hợp lệ. Muốn bắt những lỗi này cần kiểm **nội dung**, và đó là việc của lint.

### 5.3. Lint: viết tốt

| Luật | Bắt gì | Cách bắt (heuristic) |
|---|---|---|
| TC-L01 | kết quả mong đợi mơ hồ | cụm «hoạt động đúng/bình thường», «như mong đợi», «đúng yêu cầu», «không lỗi», «hiển thị đúng», hoặc chỉ một chữ «thành công» |
| TC-L02 | kết quả mong đợi không quan sát được | dòng không có chữ số, không có `mã trong backtick`, không có MÃ_LỖI |
| TC-L03 | thiếu tiền điều kiện | mảng tiền điều kiện rỗng |
| TC-L04 | nhiều mục tiêu | từ hai động từ mục tiêu («kiểm tra», «xác nhận», «đảm bảo», «chứng minh») hoặc có dấu «;» |
| TC-L05 | phụ thuộc test case khác | tiền điều kiện hoặc bước nhắc tới mã TC khác |
| TC-L06 | kết quả không tất định | «có thể», «tuỳ», «khoảng», «ngẫu nhiên», «thường thì» |
| TC-L07 | truy vết hỏng | mã R-… không có trong `SPEC.md` |

```ts
// lint-test-case.ts (trích)
const OBSERVABLE = /\d|`[^`]+`|\b[A-Z][A-Z0-9]*_[A-Z0-9_]+\b/;

for (const e of tc.expectedResults) {
  if (VAGUE.some((re) => re.test(e))) out.push({ rule: 'TC-L01', field: 'expectedResults', message: `mơ hồ: «${e}»` });
  if (!OBSERVABLE.test(e))            out.push({ rule: 'TC-L02', field: 'expectedResults', message: `không quan sát được: «${e}»` });
  if (NONDETERMINISTIC.test(e))       out.push({ rule: 'TC-L06', field: 'expectedResults', message: `không tất định: «${e}»` });
}
```

Các luật là **heuristic** và có chủ đích đơn giản. Chúng không hiểu ngữ nghĩa: một kết quả mong đợi có số vẫn có thể sai hoàn toàn, và chữ «hoặc» đôi khi hợp lệ («`driverId` = null hoặc không có trường» khi đặc tả cho phép cả hai). Lint không thay review của con người ([TCD-09](/articles/test-case-design-uu-tien-review-bao-tri)). Nó lọc trước các lỗi máy móc, để người review dành thời gian cho câu hỏi khó: kết quả mong đợi này có **đúng** với đặc tả không.

Test trong lab khẳng định hai chiều: năm test case tốt **không có** cảnh báo nào, và mỗi bản dở bị bắt **đúng** các luật dự kiến, không thừa không thiếu:

```ts
it.each(BAD.map((b) => [b.doc.id, b.label, b] as const))('%s (%s) vẫn đúng dạng nhưng bị lint bắt đúng luật', (_id, _label, b) => {
  expect(testCaseSchema.safeParse(b.doc).success).toBe(true); // schema không đủ để bắt lỗi viết
  expect(lintTestCase(b.doc, specCodes).map((f) => f.rule).sort()).toEqual([...b.violates].sort());
});
```

### 5.4. Một test case hoàn chỉnh

TC-ACCEPT-001 ở dạng tài liệu (bản dữ liệu nằm trong `tc-accept.ts`):

| Trường | TC-ACCEPT-001 |
|---|---|
| Tiêu đề | Tài xế online cùng thành phố, đúng loại xe nhận được chuyến đang tìm tài xế |
| Mục tiêu | Chứng minh tài xế thoả mọi điều kiện của R-ACCEPT-01 nhận được chuyến và các trường của phân công được ghi đúng |
| Ưu tiên | Cao |
| Truy vết | R-ACCEPT-01, R-ACCEPT-04, R-ACCEPT-05, R-STATE-07 · TCOND-ACCEPT-01, TCOND-ACCEPT-09 |
| Test basis | SPEC v1.1 · fare-2026.1 |
| Môi trường | lab toi-noi-testing, Node 22, `createApp` + kho trong bộ nhớ, `FixedClock`, id tuần tự; gọi qua supertest |
| Tiền điều kiện | hệ thống vừa reset về dữ liệu mẫu; đồng hồ cố định 09:00:00 giờ VN 04/11/2026; khách C001 đã tạo chuyến T1 (`ride`, SG, `bike`, 5 km), T1 ở `searching`, chưa có tài xế; D101 (SG, `bike`) đang `online` |
| Test data | TD-01: `driver` = D101, `etaMinutes` = 7 (ETA hợp lệ ở giữa khoảng 1–60) |
| Bước | 1) gửi `POST /api/trips/T1/accept`, `x-user: driver:D101`, thân `{"etaMinutes":7}` · 2) đọc trạng thái D101 trong kho dữ liệu |
| Kết quả mong đợi | HTTP 200; `trip.status` = `accepted`; `trip.driverId` = `D101` · `trip.etaMinutes` = 7; `trip.acceptedAt` = `2026-11-04T02:00:00.000Z` · D101 ở `busy` |
| Hậu điều kiện | T1 ở `accepted` với D101; D101 `busy`, dọn bằng reset |

Bốn test case còn lại, rút gọn:

| ID | Ưu tiên | Tiền điều kiện riêng | Bước (test data) | Kết quả mong đợi |
|---|---|---|---|---|
| TC-ACCEPT-002 | Cao | D104 `offline` | D104 nhận T1, ETA 5 (TD-02) | 409 `DRIVER_OFFLINE`; T1 `searching`, `driverId` null, `acceptedAt` null; D104 `offline` |
| TC-ACCEPT-003 | Trung bình | D202 (HN, `car`) `online` | D202 nhận T1 (SG, `bike`), ETA 5 (TD-03) | 422 `CITY_MISMATCH` (không phải `VEHICLE_MISMATCH`); T1 `searching`, `driverId` null |
| TC-ACCEPT-004 | Cao | D101 đã nhận T1 với ETA 7; D102 `online` | D102 nhận T1, ETA 4 (TD-04) | 409 `TRIP_ALREADY_TAKEN`; T1 `accepted`, `driverId` D101, ETA 7; D102 `online` |
| TC-ACCEPT-005 | Trung bình | D101 đã nhận T1 lúc 09:00:00 với ETA 7; đồng hồ ở 09:01:00 | D101 nhận lại T1, ETA 15 (TD-05) | 200; `accepted`, `driverId` D101; ETA **7**, `acceptedAt` **09:00** (`2026-11-04T02:00:00.000Z`) |

TC-ACCEPT-003 cho thấy một test case **ca âm có chủ đích**: dữ liệu vi phạm hai điều kiện cùng lúc, và mục tiêu chính là kiểm thứ tự. Thông thường, ca âm nên vi phạm đúng **một** điều kiện, để khi test đỏ biết ngay điều kiện nào bị xử lý sai (nguyên tắc single fault mà [TCD-06](/articles/test-case-design-phoi-hop-ky-thuat) đi sâu). Khi mục tiêu chính là thứ tự, vi phạm nhiều điều kiện là cần thiết. Mục tiêu nói rõ điều đó, và kết quả mong đợi ghi cả mã **không** được trả về.

TC-ACCEPT-004 và -005 có tiền điều kiện «D101 đã nhận T1». Tiền điều kiện ghi trạng thái, và lab dựng trạng thái đó bằng một lời gọi API trong phần chuẩn bị. Lab không gọi TC-ACCEPT-001, nên hai test case vẫn độc lập.

### 5.5. Chạy test case trên ứng dụng

```ts
// tc-accept-execution.test.ts (trích)
const td = (id: string) => tc(id).testData[0]!.values as { driver: string; etaMinutes: number };

beforeEach(async () => {                // tiền điều kiện chung của tài liệu
  clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
  system = createMemorySystem({ clock });
  app = createApp(system.deps, { logErrors: false });
  // báo giá ride SG bike 5 km rồi tạo chuyến T1 bằng API (có Idempotency-Key)…
});

it('TC-ACCEPT-002 · tài xế offline bị từ chối, chuyến không đổi', async () => {
  const d = td('TC-ACCEPT-002');
  expect(await driverStatus(d.driver)).toBe('offline');            // kiểm lại tiền điều kiện
  const res = await accept(T1, d.driver, d.etaMinutes);            // bước 1
  expect(res.status).toBe(409);                                    // kết quả mong đợi 1
  expect(res.body.error.code).toBe('DRIVER_OFFLINE');
  expect(await tripNow()).toMatchObject({ status: 'searching', driverId: null, acceptedAt: null }); // 2
  expect(await driverStatus('D104')).toBe('offline');              // 3
});
```

Ba chi tiết đáng học:

- **Test data đọc từ tài liệu test case.** Sửa `etaMinutes` trong `tc-accept.ts` là sửa luôn script. Tài liệu và script không trôi lệch nhau.
- **Tiền điều kiện được kiểm lại**, không chỉ được giả định. Nếu ai đó đổi dữ liệu mẫu cho D104 thành `online`, test dừng ngay ở dòng kiểm tiền điều kiện với thông báo rõ, thay vì đỏ ở kết quả mong đợi với thông báo khó hiểu.
- **Mỗi dòng kết quả mong đợi của tài liệu là một hoặc vài `expect`.** Đọc test và đọc tài liệu thấy cùng một cấu trúc.

```bash
cd examples/toi-noi-testing
npx vitest run articles/test-case-design-giai-phau-test-case
```

Kết quả thật:

```text
 ✓ articles/test-case-design-giai-phau-test-case/test-case-lint.test.ts (20 tests)
 ✓ articles/test-case-design-giai-phau-test-case/tc-accept-execution.test.ts (5 tests)

 Test Files  2 passed (2)
      Tests  25 passed (25)
```

20 test schema và lint = 5 test case tốt khớp schema + 3 test cấu trúc (bản hỏng bị từ chối đúng trường, dòng dữ liệu được bước trỏ tới có thật, ID duy nhất) + 5 test case tốt sạch lint + 7 bản dở bị bắt đúng luật. Năm test case R-ACCEPT đều đạt trên `src/` hiện tại.

## 6. Góc nhìn nâng cao

**Test case thủ công và test case tự động dùng cùng giải phẫu.** Trong vitest, `describe`/`it` là ID và tiêu đề, `beforeEach` là tiền điều kiện, biến hằng là test data, lời gọi là bước, `expect` là kết quả mong đợi, `afterEach` hoặc `reset()` là dọn hậu điều kiện. Chỗ test tự động hay thiếu là **mục tiêu**, **truy vết** và **phiên bản test basis**. Ba thứ đó không ảnh hưởng việc test chạy, nên dễ bị bỏ. Cách rẻ nhất để giữ chúng là đặt mã TC và mã R-… vào tên `it`, như lab làm. Một script đọc tên test có thể dựng lại RTM ([TCD-08](/articles/test-case-design-traceability-matrix)).

**Bao nhiêu chi tiết là đủ?** Mức chi tiết phụ thuộc người chạy, mức rủi ro và yêu cầu kiểm toán. Đội nhỏ, test tự động, người viết test cũng là người chạy: có thể rút gọn mục tiêu vào tiêu đề và bỏ hậu điều kiện khi `beforeEach` đã reset. Bộ test regression thủ công do đội khác chạy, hoặc ngành cần lưu bằng chứng: cần đủ trường. Ranh giới ngược lại cũng có. Test case viết tới mức «bấm vào nút xanh ở góc phải, toạ độ 320,48» thì vỡ mỗi lần đổi giao diện và không thêm thông tin. Đó là chi tiết của script, không phải của test case. Nhiều đội giải quyết bằng **test case logic** (không có giá trị cụ thể) cộng **test case cụ thể** (có giá trị), là chủ đề của [TCD-03](/articles/test-case-design-logic-va-cu-the).

**Một mục tiêu cho test case đồng thời.** TCOND-ACCEPT-11 (nhiều tài xế nhận cùng lúc) không có trong năm test case của lab, vì một test case tốt phải **lặp lại được**. Trên kho trong bộ nhớ hoặc PGlite một kết nối, hai lệnh «đồng thời» thực chất xen kẽ ở mức `await`. Ta kiểm được logic cập nhật có điều kiện, nhưng không kiểm được khoá dòng và giao dịch song song thật. Test case cho R-ACCEPT-03 phải ghi rõ môi trường (cần PostgreSQL nhiều kết nối) và kết quả mong đợi dạng bất biến («đúng một lệnh 200, các lệnh còn lại 409 `TRIP_ALREADY_TAKEN`, mọi tài xế thua vẫn `online`»), không phải «D101 thắng». Ai thắng là không tất định; **số người thắng** thì tất định. Đây là cách viết kết quả mong đợi tất định cho một hành vi có yếu tố ngẫu nhiên, và là chỗ race condition được kiểm.

**Lint có thể sai theo cả hai chiều.** TC-L04 đếm động từ. Một mục tiêu như «Chứng minh rằng lỗi được báo **và** chuyến không đổi» có một động từ nên qua lint, dù có thể lập luận đó là hai mục tiêu. Ngược lại, «Xác nhận rằng thứ tự kiểm tra đúng R-ACCEPT-02» có thể bị bắt nhầm nếu ai đó thêm chữ «kiểm tra» vào giữa câu. Khi lint báo sai, sửa luật hoặc chấp nhận ngoại lệ có ghi lý do; đừng viết vòng để lách lint.

| Sai lầm thường gặp | Cách làm đúng |
|---|---|
| kết quả mong đợi «hoạt động đúng», «thành công» | neo vào giá trị: mã HTTP, mã lỗi, trạng thái, số tiền |
| ca âm chỉ kiểm response | kiểm cả những gì **không** được thay đổi |
| tiền điều kiện «đã chạy TC trước» | mô tả trạng thái, dựng trực tiếp |
| quên đồng hồ trong tiền điều kiện | cố định thời điểm; ghi cả giờ VN và UTC nếu kết quả có thời điểm |
| test data chọn tuỳ tiện | ghi «vì sao» cho mỗi dòng; chọn giá trị phân biệt được đúng/sai |
| bước chứa phán đoán («kiểm tra xem…») | bước là hành động, phán đoán đưa vào kết quả mong đợi |
| ID mang nghĩa, đổi ID khi sửa | ID bất biến, không tái sử dụng |
| kết quả mong đợi chép từ hệ thống | suy từ test basis, ghi mã R-… làm căn cứ |

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1.** Viết TC-ACCEPT-006 cho TCOND-ACCEPT-03 (tài xế `busy` bị từ chối) theo đủ 12 trường của mục 4. Dùng dữ liệu mẫu của lab.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><b>Tiêu đề:</b> Tài xế đang giữ chuyến khác nhận thêm chuyến bị từ chối với DRIVER_BUSY. <b>Mục tiêu:</b> Chứng minh tài xế <code>busy</code> không nhận được chuyến thứ hai và cả hai chuyến không bị thay đổi. <b>Ưu tiên:</b> cao (tài xế giữ hai chuyến). <b>Truy vết:</b> R-ACCEPT-01, R-ACCEPT-02, R-ACCEPT-05 · TCOND-ACCEPT-03. <b>Tiền điều kiện:</b> reset; đồng hồ 09:00 giờ VN 04/11/2026; khách C001 đã tạo T1 và khách C002 đã tạo T2, cả hai <code>ride</code>, SG, <code>bike</code>, 5 km; D101 đã nhận T1 với ETA 7 (D101 <code>busy</code>); T2 ở <code>searching</code>. <b>Test data:</b> TD-06 <code>driver</code> = D101, <code>etaMinutes</code> = 5. <b>Bước:</b> gửi <code>POST /api/trips/T2/accept</code>, <code>x-user: driver:D101</code>, thân <code>{"etaMinutes":5}</code>; đọc T1, T2 và trạng thái D101. <b>Kết quả mong đợi:</b> HTTP 409, <code>error.code</code> = <code>DRIVER_BUSY</code>; T2 vẫn <code>searching</code>, <code>driverId</code> null; T1 vẫn <code>accepted</code> với D101, ETA 7; D101 vẫn <code>busy</code>. <b>Hậu điều kiện:</b> T1 <code>accepted</code>, D101 <code>busy</code>, dọn bằng reset. Chú ý tiền điều kiện ghi trạng thái («D101 đã nhận T1»), không ghi «đã chạy TC-ACCEPT-001».</p>
</details>

**Bài tập 2.** Viết lại test case sau cho đạt bốn quy tắc ở mục 4.12: «TC-07 · Nhận chuyến lỗi. Tiền điều kiện: chạy TC-05 trước. Bước: tài xế khác nhận chuyến đó; kiểm tra xem có lỗi không. Kết quả: có thể báo lỗi 409, hệ thống hoạt động bình thường.»

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Năm lỗi: ID không theo quy ước; tiêu đề không nói tình huống; phụ thuộc TC-05 (L05); bước chứa phán đoán; kết quả mong đợi không tất định và mơ hồ (L06, L01). Viết lại thành TC-ACCEPT-004 của bài: tiêu đề «Tài xế thứ hai nhận chuyến đã có người nhận bị từ chối với TRIP_ALREADY_TAKEN»; tiền điều kiện ghi trạng thái «D101 đã nhận T1 với ETA 7; D102 <code>online</code>»; bước «D102 gửi <code>POST /api/trips/T1/accept</code> với ETA 4» và «đọc T1, D102»; kết quả mong đợi «409 <code>TRIP_ALREADY_TAKEN</code>; T1 <code>accepted</code>, <code>driverId</code> D101, ETA 7; D102 <code>online</code>»; truy vết R-ACCEPT-02, R-ACCEPT-03.</p>
</details>

**Câu 1 (K2 · FL-1.4.3).** Theo CTFL v4.0.1, test case đã được ưu tiên là testware của hoạt động nào?

- a) Test analysis
- b) Test design
- c) Test implementation
- d) Test execution

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — test analysis tạo test condition đã ưu tiên. <b>b đúng</b> — test design tạo test case đã ưu tiên, test charter, coverage item, yêu cầu về test data và môi trường. <b>c sai</b> — implementation tạo test procedure, test script, test suite, test data, test execution schedule. <b>d sai</b> — execution tạo test log và defect report.</p>
</details>

**Câu 2 (K2).** Thành phần nào **không** thuộc định nghĩa test case của ISTQB Glossary?

- a) Tiền điều kiện
- b) Kết quả mong đợi
- c) Kết quả thực tế
- d) Hậu điều kiện

<details>
<summary><b>Đáp án: c</b></summary>
<p>Test case gồm tiền điều kiện, đầu vào, hành động (khi cần), kết quả mong đợi và hậu điều kiện. <b>c đúng</b> — kết quả thực tế chỉ có khi chạy và được ghi vào test log hoặc defect report, không thuộc test case.</p>
</details>

**Câu 3 (K2).** Tiền điều kiện «Đã chạy TC-ACCEPT-001 thành công» vi phạm quy tắc nào nhiều nhất?

- a) Một mục tiêu
- b) Độc lập
- c) Truy vết được
- d) Kết quả mong đợi không mơ hồ

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>b đúng</b> — test case không chạy riêng được, kết quả phụ thuộc thứ tự và kết quả của test case khác. Cách sửa: mô tả trạng thái cần có và dựng trực tiếp. Các phương án còn lại không phải vấn đề chính của tiền điều kiện này.</p>
</details>

**Câu 4 (K2).** Kết quả mong đợi nào **tốt nhất** cho test case «tài xế offline nhận chuyến»?

- a) «Hệ thống từ chối hợp lý.»
- b) «HTTP 409 với mã `DRIVER_OFFLINE`.»
- c) «HTTP 409 với mã `DRIVER_OFFLINE`; chuyến vẫn `searching`, chưa có tài xế; tài xế vẫn `offline`.»
- d) «Như kết quả khi chạy trên môi trường staging hôm qua.»

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a</b> mơ hồ. <b>b</b> đúng nhưng chưa đủ: bỏ qua những gì không được thay đổi, nên không bắt được lỗi «trả 409 nhưng vẫn gán tài xế». <b>c đúng</b> — quan sát được, suy từ R-ACCEPT-02 và R-ACCEPT-05, đầy đủ đầu ra liên quan. <b>d</b> chép từ hệ thống, không suy từ test basis.</p>
</details>

**Câu 5 (K2 · FL-1.4.4).** Mỗi test case ghi mã yêu cầu (R-…) và phiên bản test basis. Khi SPEC đổi R-ACCEPT-04 (ETA cho phép tới 90 phút), lợi ích trực tiếp nhất là gì?

- a) Không cần chạy lại test case nào.
- b) Lọc ra các test case truy vết tới R-ACCEPT-04 và ghi phiên bản cũ để review và cập nhật.
- c) Tự động sinh test case mới cho ETA 61–90.
- d) Xoá các test case có ETA trong test data.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>b đúng</b> — phân tích ảnh hưởng là giá trị của truy vết (CTFL 1.4.4); trường phiên bản cho biết test case nào được viết theo bản cũ. <b>a sai</b> — ngược lại. <b>c sai</b> — truy vết không sinh test. <b>d sai</b> — xoá mất bằng chứng; chỉ cần sửa các test case bị ảnh hưởng.</p>
</details>

**Câu 6 (K2).** Tiền điều kiện của một test case không dựng được vì API báo giá đang lỗi. Nên ghi kết quả test case đó thế nào?

- a) Fail, vì test không chạy được.
- b) Pass, vì không phát hiện lỗi ở chức năng đang kiểm.
- c) Blocked, kèm lý do; lỗi của API báo giá được báo riêng.
- d) Bỏ qua, không ghi gì.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>c đúng</b> — fail là thông tin về test object ở đúng khía cạnh test case kiểm; tiền điều kiện không thoả nghĩa là chưa kiểm được. <b>a</b> làm sai báo cáo chất lượng của lệnh nhận chuyến. <b>b</b> sai hiển nhiên. <b>d</b> làm mất thông tin tiến độ.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB® Certified Tester Foundation Level Syllabus **v4.0.1** (2024): mục 1.4.1 Test Activities and Tasks (test design), 1.4.3 Testware, 1.4.4 Traceability between the Test Basis and Testware · FL-1.4.1 (K2), FL-1.4.3 (K2), FL-1.4.4 (K2). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB® Glossary (tra *test case*, *precondition*, *postcondition*, *expected result*, *test procedure*): https://glossary.istqb.org/
- ISO/IEC/IEEE 29119-3 — Software and systems engineering — Software testing — Part 3: Test documentation (test case specification, test procedure specification). Bài chỉ dùng làm khung tham khảo.
- zod — thư viện khai báo và kiểm schema cho TypeScript: https://zod.dev/
- Lab Tới Nơi: `SPEC.md` (R-ACCEPT, R-API-06, R-API-10, R-STATE-07, Phụ lục A) và thư mục bài: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/test-case-design-giai-phau-test-case
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-010 · Testware](/articles/testware-san-pham-kiem-thu) · [TCD-01 · Test analysis](/articles/test-case-design-test-analysis-test-condition) · [TCD-04 · Kết quả mong đợi và test oracle](/articles/test-case-design-expected-result-test-oracle) · [TCD-05 · Test data](/articles/test-case-design-test-data) · [TCD-08 · Ma trận truy vết](/articles/test-case-design-traceability-matrix)

Bài tiếp theo: [TCD-03 · Test case logic và test case cụ thể](/articles/test-case-design-logic-va-cu-the).
