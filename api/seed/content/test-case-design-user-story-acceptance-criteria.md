---
slug: test-case-design-user-story-acceptance-criteria
title: "Test case từ user story và acceptance criteria: Given/When/Then, dạng rule và ATDD cho huỷ chuyến Tới Nơi"
summary: "Từ user story «huỷ chuyến và biết trước phí» tới acceptance criteria dạng scenario và dạng rule, phát hiện chỗ mơ hồ (mốc 2 phút, «now» là lúc nào, tài xế trễ đúng mốc), thêm kịch bản biên, ánh xạ AC → test case và chạy ATDD bằng step runner tí hon trong vitest: 32 ca đạt, 1 ca chờ BA."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-case-design, tcd-07, user-story, acceptance-criteria, given-when-then, atdd, example-mapping, vitest, toi-noi]
language: vi
---

«Là khách, tôi muốn huỷ chuyến và biết trước mình mất bao nhiêu phí.» Câu này đủ ngắn để viết lên một tấm thẻ, nhưng chưa đủ để kiểm thử: «biết trước» là trước khi bấm hay sau khi bấm? «Trước 2 phút» tính từ lúc nào, đến lúc nào? Tài xế «trễ» thì trễ so với cái gì? Bài này đi từ user story tới **acceptance criteria** ở hai dạng mà CTFL v4.0.1 nêu — dạng kịch bản Given/When/Then và dạng quy tắc — rồi dùng **ATDD** để biến chúng thành test case viết trước code và chạy được. Trên đường đi, ta bắt được sáu chỗ mơ hồ của chính yêu cầu, thêm năm kịch bản nằm trên biên, và chứng minh bằng lab vì sao một bộ ví dụ «điển hình» để lọt defect so sánh biên.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TCD-07</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>TCD · Thiết kế Test Case</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Acceptance (user story); System</td></tr>
<tr><th scope="row">Test Type</th><td>Functional</td></tr>
<tr><th scope="row">Test Technique</th><td>ATDD; acceptance criteria dạng scenario và dạng rule; kết hợp Boundary Value Analysis</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/test-case-design-giai-phau-test-case">TCD-02 · Giải phẫu test case</a> · <a href="/articles/acceptance-testing-tong-quan">TEST-016 · Acceptance Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): 15 kịch bản Given/When/Then viết thành dữ liệu có kiểu, chạy qua API bằng step runner tí hon + bảng rule 10 dòng — <code>examples/toi-noi-testing/articles/test-case-design-user-story-acceptance-criteria/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 4 · mục 4.5.1; mục 4.5.2 · FL-4.5.2 (K2); mục 4.5.3 · FL-4.5.3 (K3)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Ở các bài trước của nhánh TCD, test basis là một đặc tả có mã yêu cầu rõ ràng. Trong đội Agile, test basis thường đến dưới dạng khác: một **user story** kèm vài dòng **acceptance criteria** (AC — tiêu chí chấp nhận), viết trong buổi refinement, đôi khi chỉ vài phút trước khi developer bắt tay vào làm. Hai vấn đề nảy ra ngay:

1. **AC viết bằng ngôn ngữ tự nhiên nên mơ hồ.** «Huỷ trong 2 phút đầu được miễn phí» — đúng 2 phút thì sao? Khách bấm lúc 1 phút 59 giây nhưng mạng chậm, yêu cầu tới máy chủ lúc 2 phút 0,2 giây thì sao?
2. **AC chưa phải test case.** Một AC có thể sinh ra nhiều test case (giá trị biên, loại xe, phương thức thanh toán), và một test case có thể chứng minh nhiều AC. Nếu không ánh xạ rõ, ta không biết AC nào đã có bằng chứng, AC nào chưa.

Bài này trả lời hai vấn đề đó theo đúng ba mục của CTFL v4.0.1 phần 4.5 (collaboration-based test approaches): viết user story cùng nhau (4.5.1), viết acceptance criteria (4.5.2), và dùng ATDD để suy ra test case (4.5.3). Dùng khi:

- yêu cầu đến dưới dạng user story, test case cần sẵn sàng **trước** khi code xong;
- đội muốn acceptance test chạy tự động và đọc được bởi người không lập trình;
- quy tắc nghiệp vụ có mốc thời gian, mốc tiền — nơi ví dụ cụ thể làm lộ hiểu lầm nhanh hơn văn xuôi.

Không thay thế: kỹ thuật thiết kế test có hệ thống ([TEST-033 · Decision Table](/articles/decision-table-testing) cho đủ tổ hợp, [TEST-034 · State Transition](/articles/state-transition-testing-co-ban) cho chuỗi sự kiện) — ATDD **dùng** các kỹ thuật đó khi viết test case, CTFL nói rõ điều này. Cũng không thay thế UAT do người dùng thật thực hiện ([SYS-15 · UAT](/articles/system-testing-uat)).

## 2. Lý thuyết theo ISTQB

### 2.1. User story viết cùng nhau — mục 4.5.1

CTFL mô tả user story là một tính năng có giá trị với người dùng hoặc người mua hệ thống, gồm ba khía cạnh thường gọi là **3C**:

| C | Nghĩa | Ở Tới Nơi |
|---|---|---|
| **Card** | Phương tiện ghi story (thẻ giấy, mục trong backlog) | «Là khách, tôi muốn huỷ chuyến và biết trước mình mất bao nhiêu phí, để tự quyết định huỷ ngay hay chờ tài xế» |
| **Conversation** | Cuộc trao đổi giải thích phần mềm sẽ được dùng thế nào; có thể ghi lại hoặc chỉ nói miệng | Buổi example mapping giữa PO, BA, developer, tester ở mục 4.2 |
| **Confirmation** | Acceptance criteria | AC-01…AC-08 ở mục 4.3 |

Khuôn phổ biến nhất: «Là [vai trò], tôi muốn [mục tiêu], để [giá trị kinh doanh]», theo sau là acceptance criteria. Story được viết **cùng nhau** — business, developer, tester — bằng các kỹ thuật như brainstorming hay mind map, để mỗi người góp một góc nhìn. Story tốt thoả **INVEST**: Independent, Negotiable, Valuable, Estimable, Small, Testable. CTFL lưu ý một tín hiệu đáng giá cho tester: nếu stakeholder **không biết kiểm story thế nào**, có thể story chưa đủ rõ, chưa phản ánh giá trị thật, hoặc stakeholder cần giúp đỡ để kiểm. Cách viết story từ phía BA có ở [User story: vai trò, nhu cầu và giá trị](/articles/ba-user-story-vai-tro-nhu-cau-gia-tri); bài này chỉ dùng góc nhìn kiểm thử.

### 2.2. Acceptance criteria — mục 4.5.2 (FL-4.5.2, K2)

Acceptance criteria là **các điều kiện mà phần cài đặt của user story phải thoả để được stakeholder chấp nhận**. CTFL đưa ra một góc nhìn quan trọng với người thiết kế test: **AC chính là các test condition** cần được test thực thi. Chúng thường là kết quả của phần Conversation. AC được dùng để:

- xác định **phạm vi** của story;
- đạt **đồng thuận** giữa các stakeholder;
- mô tả cả kịch bản **dương** lẫn **âm**;
- làm **cơ sở cho acceptance testing** của story (mục 4.5.3);
- giúp **lập kế hoạch và ước lượng** chính xác.

Hai dạng phổ biến nhất:

| Dạng | Hình thức | Mạnh ở | Yếu ở |
|---|---|---|---|
| **Scenario-oriented** (hướng kịch bản) | Given/When/Then, dạng dùng trong BDD | Kể một ví dụ cụ thể có bối cảnh, hành động, kết quả; dễ đọc với business; chuyển thành test chạy được gần như 1–1 | Nhiều kịch bản cho một quy tắc dễ trùng lặp; khó thấy «còn thiếu tổ hợp nào» |
| **Rule-oriented** (hướng quy tắc) | Danh sách kiểm (bullet), hoặc **bảng ánh xạ vào–ra** | Gọn, phủ cả quy tắc trong một chỗ; bảng làm lộ ô trống | Dễ viết chung chung («phí hợp lý»); thiếu bối cảnh thời gian, thứ tự |

CTFL nói phần lớn AC rơi vào một trong hai dạng, nhưng dạng khác vẫn được chấp nhận **miễn là AC được định nghĩa tốt và không mơ hồ**. Ý cuối này là trọng tâm của mục 4.5 bài này.

### 2.3. ATDD — mục 4.5.3 (FL-4.5.3, K3)

Acceptance test-driven development là cách tiếp cận **test-first** (CTFL mục 2.1.3): test case được tạo **trước** khi cài đặt story, bởi các thành viên có góc nhìn khác nhau — khách hàng, developer, tester — và có thể chạy thủ công hoặc tự động. Trình tự CTFL mô tả:

1. **Specification workshop.** Story và AC (nếu chưa có) được phân tích, thảo luận, viết lại. Chỗ **chưa đầy đủ, mơ hồ hoặc sai** của story được giải quyết ngay trong bước này.
2. **Tạo test case** từ AC — cả đội cùng làm hoặc một tester làm. Test case là **ví dụ** về cách phần mềm hoạt động; vì ví dụ và test là một, hai từ thường dùng lẫn cho nhau. Khi thiết kế, áp dụng được các kỹ thuật ở mục 4.2–4.4 (EP, BVA, decision table, state transition, kỹ thuật dựa trên kinh nghiệm).
3. **Thứ tự**: trước hết là **ca dương** — chuỗi hoạt động khi mọi thứ diễn ra như mong đợi, không lỗi, không ngoại lệ; tiếp theo là **ca âm**; cuối cùng là đặc tính **phi chức năng** (hiệu năng, khả dụng…).
4. **Cách diễn đạt**: test case viết sao cho stakeholder hiểu được, thường bằng câu tự nhiên gồm tiền điều kiện (nếu có), đầu vào và hậu điều kiện.

Ba ràng buộc cho bộ test case, rất đáng thuộc:

- phải **phủ mọi đặc tính** của user story;
- **không vượt quá** story (nhưng AC có thể chi tiết hoá vấn đề mà story mô tả);
- **không có hai test case mô tả cùng một đặc tính** của story.

Khi test case được viết theo định dạng mà framework tự động hoá hỗ trợ, developer viết **mã hỗ trợ** (supporting code) trong lúc cài đặt tính năng, và acceptance test trở thành **yêu cầu chạy được** (executable requirements). Lab của bài làm đúng điều này mà không cần Cucumber: kịch bản là dữ liệu, mã hỗ trợ là một step runner vài chục dòng.

### 2.4. ATDD, BDD và TDD — đặt cạnh nhau

CTFL mục 2.1.3 xếp ba cách tiếp cận test-first cạnh nhau: TDD dẫn dắt code bằng test case (thường ở mức component), ATDD suy test từ acceptance criteria như một phần của thiết kế hệ thống, BDD diễn đạt hành vi mong muốn bằng ngôn ngữ tự nhiên đơn giản — thường là Given/When/Then — rồi chuyển tự động thành test chạy được. Trong thực tế ATDD và BDD chồng lấn nhiều; khác biệt nằm ở trọng tâm (tiêu chí chấp nhận của story so với mô tả hành vi) hơn là ở cú pháp. Vị trí của test-first trong vòng đời có ở [TEST-030 · Shift-left testing](/articles/shift-left-testing).

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p><b>Example mapping</b> (Matt Wynne, 2015) là kỹ thuật tổ chức buổi Conversation bằng bốn màu thẻ: vàng = story, xanh dương = quy tắc (rule), xanh lá = ví dụ (example), đỏ = câu hỏi chưa ai trả lời được. Buổi họp ngắn, kết thúc khi các thẻ xanh dương đều có ví dụ và thẻ đỏ đã có người nhận trả lời. <b>Specification by Example</b> (Gojko Adzic, 2011) là cách gọi rộng hơn cho việc dùng ví dụ cụ thể làm đặc tả. Cả hai không có trong CTFL; bài dùng example mapping như một cách thực hiện bước specification workshop của mục 4.5.3. Cú pháp Gherkin và công cụ Cucumber cũng là thực hành ngành, không phải yêu cầu của ISTQB.</p>
</div>

### 2.5. Thuật ngữ Anh – Việt

| Thuật ngữ | Tiếng Việt | Ghi chú |
|---|---|---|
| user story | câu chuyện người dùng | Card + Conversation + Confirmation |
| acceptance criteria | tiêu chí chấp nhận | CTFL: điều kiện để stakeholder chấp nhận; xem như test condition |
| scenario-oriented / rule-oriented | hướng kịch bản / hướng quy tắc | hai dạng AC phổ biến (CTFL 4.5.2) |
| Given/When/Then | Cho trước / Khi / Thì | bối cảnh, hành động, kết quả quan sát được |
| ATDD (acceptance test-driven development) | phát triển hướng kiểm thử chấp nhận | test-first, CTFL 4.5.3 |
| specification workshop | buổi làm rõ đặc tả | bước đầu của ATDD |
| executable requirement | yêu cầu chạy được | acceptance test tự động hoá |
| supporting code / step definition | mã hỗ trợ / định nghĩa bước | nối câu tự nhiên với hệ thống |
| example mapping | bản đồ ví dụ | ngoài ISTQB |
| INVEST | — | tiêu chí story tốt (CTFL 4.5.1) |

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. User story và cuộc trao đổi

PO đưa vào sprint một story từ phản hồi của khách: nhiều người huỷ chuyến rồi mới biết mình mất 20.000 đ, gọi tổng đài khiếu nại. Tấm thẻ:

> **US-CANCEL-01.** Là **khách**, tôi muốn **huỷ chuyến và biết trước mình mất bao nhiêu phí**, để **tự quyết định huỷ ngay hay chờ tài xế**.

Test basis đi kèm là SPEC lab của Tới Nơi (đã review ở [TEST-023](/articles/informal-review)). Tester mang theo bảng quyết định đã lập ở [TEST-033](/articles/decision-table-testing) — không phải để thay AC, mà để biết **những quy tắc nào chắc chắn phải có ví dụ**.

### 3.2. Business rules liên quan

| Mã | Quy tắc |
|---|---|
| R-CANCEL-01 | Khách huỷ: `searching` ⇒ 0 đ; `accepted` chưa đủ 2 phút kể từ `acceptedAt` ⇒ 0 đ; `accepted` từ 2 phút trở lên ⇒ có phí; `arriving` ⇒ có phí; trạng thái khác ⇒ không được huỷ (`INVALID_TRANSITION`) |
| R-CANCEL-02 | «Chưa đủ 2 phút» = `now − acceptedAt < 2 phút`; đúng 2 phút 0 giây tính là có phí |
| R-CANCEL-03 | 10.000 đ xe máy (gồm mọi đơn đồ ăn), 20.000 đ ô tô |
| R-CANCEL-04 | `now > acceptedAt + etaMinutes + 10 phút` (lớn hơn hẳn) ⇒ 0 đ; đúng bằng mốc vẫn thu |
| R-CANCEL-05 | Tài xế huỷ hoặc hệ thống huỷ không thu phí khách |
| R-CANCEL-06 | Lý do: `searching`, `grace_period`, `driver_late`, `charged`; ưu tiên `searching` → `grace_period` → `driver_late` |
| R-CANCEL-07 | Phí > 0 thu theo R-PAY-05 (VíXanh khoá `trip:<id>:cancel-fee`, tiền mặt ghi nhận) và lưu vào chuyến (`cancellationFee`) |
| R-STATE-03/04 | Chuyển không hợp lệ bị từ chối, chuyến giữ nguyên; tài xế huỷ ⇒ chuyến về `searching`, xoá `acceptedAt` |
| R-UI-01 | Trang khách: nút «Huỷ chuyến» **hiện phí huỷ** |
| R-IDEM-02 | Gửi lại cùng `Idempotency-Key`, cùng nội dung ⇒ trả lại đúng phản hồi lần đầu |

### 3.3. System context

Kiến trúc đích: app khách gọi API Gateway → **Booking/Order Service** (huỷ) → **Trip Lifecycle** (kiểm chuyển trạng thái) → **Payment & Wallet** (thu phí VíXanh). Trong lab: `POST /api/trips/:id/cancel` (R-API-11), use case `cancelByCustomer` ở `src/app/trips.ts`, hàm phí thuần `customerCancellationFee` ở `src/domain/cancel.ts`. Trang khách `src/web/customer.js` gọi API khi bấm nút và **chỉ hiện phí sau khi huỷ xong** — chi tiết này sẽ quan trọng ở mục 4.4.

### 3.4. Rủi ro

| Rủi ro | Hậu quả | AC liên quan |
|---|---|---|
| Thu phí khi đáng lẽ miễn (đúng biên 2 phút, tài xế trễ) | Khiếu nại, hoàn tiền thủ công | AC-02, AC-04 |
| Màn hình nói «miễn phí» nhưng hệ thống thu tiền | Mất niềm tin — đúng vấn đề story muốn giải | AC-07, AC-08 |
| Đúng tiền nhưng sai lý do | Khách không hiểu vì sao bị thu | AC-07 |
| Bấm «Huỷ» hai lần bị thu hai lần | Thu trùng | AC-07 (R-IDEM) |
| Hiểu khác nhau giữa PO, developer, tester | Làm lại sau sprint review | Toàn bộ — giải bằng specification workshop |

## 4. Phân tích kỹ thuật

### 4.1. Soát story theo INVEST

| Tiêu chí | Đánh giá | Ghi chú |
|---|---|---|
| Independent | Đạt | Không phụ thuộc story khác trong sprint |
| Negotiable | Đạt | Cách hiển thị phí còn thương lượng được |
| Valuable | Đạt | Giảm khiếu nại về phí huỷ |
| Estimable | **Chưa** | «Biết trước» chưa rõ là cần màn hình xác nhận hay chỉ hiện phí sau khi huỷ |
| Small | **Chưa chắc** | Nếu «biết trước» cần API xem trước phí mới thì story to hơn một sprint |
| Testable | **Chưa** | Chưa ai trả lời được «kiểm "biết trước" thế nào» — đúng tín hiệu CTFL 4.5.1 nhắc |

Ba chữ «chưa» cùng chỉ về một cụm từ: **«biết trước»**. SPEC hiện có (R-UI-01, R-API-11) chỉ cho khách thấy phí **sau** khi lệnh huỷ đã chạy. Đó là câu hỏi đầu tiên mang vào buổi trao đổi.

### 4.2. Buổi example mapping

Bốn người, 25 phút. Kết quả ghi lại thành sơ đồ dưới đây: năm quy tắc (thẻ xanh dương), mỗi quy tắc có ví dụ (thẻ xanh lá), và các câu hỏi chưa có lời đáp (thẻ đỏ).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 940 430" role="img" aria-label="Bản đồ ví dụ cho user story huỷ chuyến: thẻ story ở trên cùng, năm thẻ quy tắc thành năm cột, dưới mỗi quy tắc là các thẻ ví dụ, bên phải là các thẻ câu hỏi chưa có lời đáp.">
<rect x="20" y="16" width="740" height="44" rx="6" class="f-warn-soft s-warn"/>
<text x="390" y="44" text-anchor="middle" class="t-b">US-CANCEL-01 · Khách huỷ chuyến và biết trước mình mất bao nhiêu phí</text>
<rect x="20" y="78" width="140" height="56" rx="6" class="f-phys-soft s-phys"/>
<text x="90" y="100" text-anchor="middle" class="t-small t-b t-phys">Chưa có tài xế</text>
<text x="90" y="120" text-anchor="middle" class="t-small">⇒ miễn phí</text>
<rect x="170" y="78" width="140" height="56" rx="6" class="f-phys-soft s-phys"/>
<text x="240" y="100" text-anchor="middle" class="t-small t-b t-phys">Trong 2 phút đầu</text>
<text x="240" y="120" text-anchor="middle" class="t-small">⇒ miễn phí</text>
<rect x="320" y="78" width="140" height="56" rx="6" class="f-phys-soft s-phys"/>
<text x="390" y="100" text-anchor="middle" class="t-small t-b t-phys">Sau đó / đã tới</text>
<text x="390" y="120" text-anchor="middle" class="t-small">⇒ 10.000 / 20.000 đ</text>
<rect x="470" y="78" width="140" height="56" rx="6" class="f-phys-soft s-phys"/>
<text x="540" y="100" text-anchor="middle" class="t-small t-b t-phys">Tài xế trễ</text>
<text x="540" y="120" text-anchor="middle" class="t-small">⇒ miễn phí</text>
<rect x="620" y="78" width="140" height="56" rx="6" class="f-phys-soft s-phys"/>
<text x="690" y="100" text-anchor="middle" class="t-small t-b t-phys">Đã đón khách</text>
<text x="690" y="120" text-anchor="middle" class="t-small">⇒ không huỷ được</text>
<g class="t-small">
<rect x="20" y="150" width="140" height="40" rx="6" class="f-pk-soft s-pk"/>
<text x="90" y="174" text-anchor="middle">huỷ 09:00:30 ⇒ 0 đ</text>
<rect x="170" y="150" width="140" height="40" rx="6" class="f-pk-soft s-pk"/>
<text x="240" y="174" text-anchor="middle">nhận +1′ ⇒ 0 đ</text>
<rect x="170" y="198" width="140" height="40" rx="6" class="f-pk-soft s-pk"/>
<text x="240" y="222" text-anchor="middle">+1′59,999 ⇒ 0 đ</text>
<rect x="170" y="246" width="140" height="40" rx="6" class="f-pk-soft s-pk"/>
<text x="240" y="270" text-anchor="middle">TX huỷ, TX khác +1′</text>
<rect x="320" y="150" width="140" height="40" rx="6" class="f-pk-soft s-pk"/>
<text x="390" y="174" text-anchor="middle">đúng +2′00 ⇒ 10.000</text>
<rect x="320" y="198" width="140" height="40" rx="6" class="f-pk-soft s-pk"/>
<text x="390" y="222" text-anchor="middle">ô tô +5′ ⇒ 20.000</text>
<rect x="320" y="246" width="140" height="40" rx="6" class="f-pk-soft s-pk"/>
<text x="390" y="270" text-anchor="middle">đã tới +1′ ⇒ 10.000</text>
<rect x="320" y="294" width="140" height="40" rx="6" class="f-pk-soft s-pk"/>
<text x="390" y="318" text-anchor="middle">đồ ăn ⇒ 10.000</text>
<rect x="470" y="150" width="140" height="40" rx="6" class="f-pk-soft s-pk"/>
<text x="540" y="174" text-anchor="middle">đúng mốc ⇒ vẫn thu</text>
<rect x="470" y="198" width="140" height="40" rx="6" class="f-pk-soft s-pk"/>
<text x="540" y="222" text-anchor="middle">mốc + 1 ms ⇒ 0 đ</text>
<rect x="470" y="246" width="140" height="40" rx="6" class="f-pk-soft s-pk"/>
<text x="540" y="270" text-anchor="middle">tới muộn rồi huỷ</text>
<rect x="620" y="150" width="140" height="40" rx="6" class="f-pk-soft s-pk"/>
<text x="690" y="174" text-anchor="middle">picked_up ⇒ 409</text>
</g>
<g class="t-small">
<rect x="790" y="78" width="134" height="62" rx="6" class="f-surface s-dead"/>
<text x="857" y="100" text-anchor="middle" class="t-b">Q-01</text>
<text x="857" y="120" text-anchor="middle">«biết trước» là</text>
<text x="857" y="134" text-anchor="middle">trước khi bấm?</text>
<rect x="790" y="150" width="134" height="62" rx="6" class="f-surface s-dead"/>
<text x="857" y="172" text-anchor="middle" class="t-b">Q-02</text>
<text x="857" y="192" text-anchor="middle">«now» là giờ</text>
<text x="857" y="206" text-anchor="middle">máy khách hay chủ?</text>
<rect x="790" y="222" width="134" height="62" rx="6" class="f-surface s-dead"/>
<text x="857" y="244" text-anchor="middle" class="t-b">Q-04</text>
<text x="857" y="264" text-anchor="middle">tới muộn rồi huỷ</text>
<text x="857" y="278" text-anchor="middle">có miễn phí?</text>
</g>
<line x1="20" y1="356" x2="924" y2="356" class="s-muted dash" stroke-width="1"/>
<rect x="20" y="372" width="18" height="14" class="f-warn-soft s-warn"/><text x="46" y="384" class="t-small">story</text>
<rect x="110" y="372" width="18" height="14" class="f-phys-soft s-phys"/><text x="136" y="384" class="t-small">quy tắc → acceptance criteria</text>
<rect x="340" y="372" width="18" height="14" class="f-pk-soft s-pk"/><text x="366" y="384" class="t-small">ví dụ → kịch bản Given/When/Then</text>
<rect x="600" y="372" width="18" height="14" class="f-surface s-dead"/><text x="626" y="384" class="t-small">câu hỏi → gửi PO/BA, chặn «sẵn sàng»</text>
<text x="470" y="414" text-anchor="middle" class="t-small">Mỗi ví dụ xanh lá là một kịch bản SC-… chạy được trong lab của bài</text>
</svg>
</div>
<figcaption><b>Hình 1 · Example mapping cho US-CANCEL-01.</b> Cột nhiều thẻ xanh lá là quy tắc phức tạp; thẻ đỏ còn lại nghĩa là story chưa sẵn sàng đưa vào làm. Màu theo quy ước của Matt Wynne (ngoài ISTQB).</figcaption>
</figure>

Đọc sơ đồ như một công cụ kiểm thử tĩnh: cột «Sau đó / đã tới» có bốn ví dụ — quy tắc phức tạp nhất, đáng ưu tiên khi viết test. Cột «Đã đón khách» chỉ có một ví dụ — đủ, vì đó là phân vùng tương đương các trạng thái không cho huỷ ([TEST-031](/articles/equivalence-partitioning)). Thẻ đỏ không được bỏ qua: chúng là **defect tiềm ẩn của test basis**, phát hiện trước khi viết dòng code nào.

### 4.3. Acceptance criteria dạng rule

Mỗi thẻ xanh dương thành một AC có thể kiểm chứng. Cột «SPEC» giữ truy vết; AC-08 không có căn cứ trong SPEC nên ghi rõ là **giả định bài tập**.

| Mã | Acceptance criterion | SPEC |
|---|---|---|
| AC-01 | Chưa có tài xế nhận: khách huỷ miễn phí, lý do `searching` | R-CANCEL-01, R-CANCEL-06 |
| AC-02 | Tài xế đã nhận nhưng chưa báo tới, chưa đủ 2 phút kể từ lúc nhận: miễn phí, lý do `grace_period` | R-CANCEL-01, R-CANCEL-02 |
| AC-03 | Từ đúng 2 phút trở đi, hoặc tài xế đã báo tới: 10.000 đ xe máy (mọi đơn đồ ăn), 20.000 đ ô tô, lý do `charged` | R-CANCEL-01…03 |
| AC-04 | Quá hẳn mốc «lúc nhận + ETA + 10 phút»: miễn phí, lý do `driver_late`; đúng bằng mốc vẫn thu | R-CANCEL-04, R-CANCEL-06 |
| AC-05 | Đã đón khách trở đi: không huỷ được (409 `INVALID_TRANSITION`), chuyến và nhật ký giữ nguyên | R-CANCEL-01, R-STATE-03 |
| AC-06 | Tài xế huỷ: khách không mất phí, chuyến quay lại tìm tài xế; mốc 2 phút tính lại từ lần nhận mới | R-CANCEL-05, R-STATE-04, R-CANCEL-02 |
| AC-07 | Kết quả huỷ báo số phí và lý do; phí > 0 thu qua VíXanh (khoá `trip:<id>:cancel-fee`) hoặc ghi nhận nếu tiền mặt; phí lưu vào chuyến | R-CANCEL-06, R-CANCEL-07, R-PAY-05, R-UI-01 |
| AC-08 | *(giả định — chưa có trong SPEC)* Trước khi xác nhận huỷ, khách thấy phí dự kiến và mốc giờ phí thay đổi | — |

Dạng rule còn có biến thể thứ hai mà CTFL nêu: **bảng ánh xạ vào–ra**. Với quy tắc phí, bảng gọn hơn văn xuôi rất nhiều (ETA 5 phút, mốc trễ = 15 phút sau khi nhận):

| Dòng | Trạng thái | Xe | Thời gian kể từ lúc nhận | Phí | Lý do |
|---|---|---|---|---|---|
| TD-01 | `searching` | xe máy | — | 0 đ | `searching` |
| TD-02 | `accepted` | xe máy | 1′59,999″ | 0 đ | `grace_period` |
| TD-03 | `accepted` | xe máy | 2′00″ | 10.000 đ | `charged` |
| TD-04 | `accepted` | ô tô | 2′00″ | 20.000 đ | `charged` |
| TD-05 | `arriving` | xe máy | 0′30″ | 10.000 đ | `charged` |
| TD-06 | `accepted` | ô tô | 15′00″ | 20.000 đ | `charged` |
| TD-07 | `accepted` | ô tô | 15′00,001″ | 0 đ | `driver_late` |
| TD-08 | `arriving` | xe máy | 16′00″ | 0 đ | `driver_late` |
| TD-09 | `picked_up` | xe máy | 6′00″ | — | `INVALID_TRANSITION` |
| TD-10 | `completed` | ô tô | 30′00″ | — | `INVALID_TRANSITION` |

Bảng này **không** phải bảng quyết định đầy đủ của TEST-033 (40 cột khả thi). Nó là lát cắt mà PO đọc trong một phút và gật đầu được — đúng mục đích «đạt đồng thuận» của AC. Muốn đủ tổ hợp thì dùng bảng của TEST-033 làm test bổ sung.

### 4.4. Phát hiện chỗ mơ hồ

Viết ví dụ cụ thể buộc ta trả lời những câu mà văn xuôi né được. Sáu câu hỏi dưới đây ghi vào phiếu gửi PO/BA. Với từng câu, cột «SPEC nói gì» cho biết câu hỏi đã có lời đáp trong test basis hay là lỗ hổng thật.

| Mã | Câu hỏi | SPEC nói gì | Xử lý trong AC |
|---|---|---|---|
| Q-01 | «Biết trước» là trước khi bấm xác nhận, hay chỉ cần thấy phí sau khi huỷ? | R-UI-01 chỉ nói nút «Huỷ chuyến» **hiện phí huỷ**; trang khách hiện phí **sau** khi API trả về. Không có API xem trước | **Lỗ hổng.** Tách thành AC-08 (giả định), kịch bản SC-16 để ở trạng thái chờ (todo). Đề xuất tách story: phần «huỷ và thấy phí» làm ngay; phần «xem trước» là story mới |
| Q-02 | «Now» là giờ trên điện thoại khách hay giờ máy chủ? Khách nhìn lúc 1′59″ nhưng yêu cầu tới máy chủ lúc 2′00,2″ thì sao? | R-GEN-02 nói «thời điểm đặt» theo đồng hồ hệ thống; R-CANCEL-02 không nói rõ cho lệnh huỷ. Lab dùng giờ máy chủ lúc xử lý | **Mơ hồ.** Ghi vào AC-02: «tính theo giờ máy chủ lúc nhận lệnh». Nếu làm AC-08, màn hình xem trước phải cảnh báo khi còn ít giây |
| Q-03 | «Tài xế trễ» là trễ so với ETA hiển thị hay so với lúc nhận + ETA? Đúng mốc thì sao? | R-CANCEL-04 rõ: so với `acceptedAt + etaMinutes + 10′`, **lớn hơn hẳn**; đúng mốc vẫn thu | **Đã rõ trong SPEC**, nhưng câu tự nhiên «trễ hơn 10 phút» dễ bị hiểu thành `≥`. Thêm hai kịch bản biên SC-08, SC-09 |
| Q-04 | Tài xế tới muộn (đã ở `arriving`) rồi khách mới huỷ — vẫn miễn phí? | R-CANCEL-04 nói «kể cả khi R-CANCEL-01 có phí» ⇒ áp cả cho `arriving`. SPEC không lưu thời điểm tới | **Đúng theo chữ, nhưng có thể không đúng ý.** Kịch bản SC-10 khẳng định hành vi hiện tại; PO xác nhận đây có phải chủ đích |
| Q-05 | Tài xế huỷ, tài xế khác nhận lại — 2 phút miễn phí tính từ lần nhận nào? | R-STATE-04 xoá `acceptedAt`; lần nhận mới ghi `acceptedAt` mới | **Đã rõ khi đọc kết hợp hai quy tắc.** Thêm SC-13 vì rất dễ cài sai (giữ `acceptedAt` cũ) |
| Q-06 | Đã báo tới (`arriving`) trong 2 phút đầu có được miễn phí? | R-CANCEL-01: `arriving` ⇒ có phí, không có điều kiện thời gian | **Đã rõ**, nhưng thẻ story không gợi ý điều này. Thêm SC-06 |

Bài học: chỉ Q-01 là lỗ hổng thật của SPEC, Q-02 là mơ hồ, bốn câu còn lại SPEC đã trả lời — nhưng **AC viết bằng lời của story thì không**. Nếu tester chỉ đọc thẻ story mà không đối chiếu SPEC, cả sáu đều thành hiểu lầm tiềm tàng. Đây chính là «incompleteness, ambiguities, or defects» mà CTFL 4.5.3 muốn giải quyết ở specification workshop.

### 4.5. Acceptance criteria dạng scenario

Mỗi thẻ xanh lá thành một kịch bản Given/When/Then. Năm nguyên tắc viết cho bài này:

1. **Dữ liệu cụ thể.** «09:03:00.000», «10.000 đ», không «sau một lúc», «một khoản phí».
2. **Khai báo, không mệnh lệnh** (declarative, not imperative). «Tài xế nhận lúc 09:01:00 với ETA 5 phút», không «mở app tài xế, chọn chuyến, gõ 5, bấm Nhận».
3. **Một hành vi một kịch bản.** When chỉ có một hành động nghiệp vụ (trừ kịch bản kiểm gửi lại, cần hai lần bấm).
4. **Then quan sát được** từ bên ngoài: phí, lý do, trạng thái, lời gọi ví — không «biến nội bộ được đặt đúng».
5. **Ca dương trước, ca âm sau** đúng thứ tự CTFL 4.5.3.

Kịch bản biên đặt hai giá trị sát mốc, theo BVA hai giá trị ([TEST-032](/articles/boundary-value-analysis)):

```gherkin
Scenario: SC-04 — huỷ đúng 2 phút sau khi nhận thì mất 10.000 đ (xe máy, tiền mặt)
  Given khách C001 có chuyến xe máy ở SG tạo lúc 09:00:00, trả tiền mặt
  And   tài xế mặc định nhận lúc 09:01:00 với ETA 5 phút
  When  khách bấm «Huỷ chuyến» lúc 09:03:00.000
  Then  chuyến ở trạng thái «cancelled»
  And   phí huỷ là 10.000 đ
  And   lý do phí là «charged»
  And   paymentStatus là «cash»
  And   ví VíXanh không bị gọi

Scenario: SC-09 — quá mốc 1 ms thì miễn phí vì tài xế trễ
  Given khách C001 có chuyến xe máy ở SG tạo lúc 09:00:00, trả tiền mặt
  And   tài xế mặc định nhận lúc 09:01:00 với ETA 5 phút
  When  khách bấm «Huỷ chuyến» lúc 09:16:00.001
  Then  phí huỷ là 0 đ
  And   lý do phí là «driver_late»
```

Văn bản trên **không viết tay**: lab in nó ra từ chính dữ liệu kịch bản (hàm `vietKichBan`), và một test khẳng định văn bản in ra khớp từng dòng. Nhờ vậy bản BA đọc và bản máy chạy không thể lệch nhau.

Kịch bản SC-15 trả lời Q-02 theo một hướng ít ai nghĩ tới: khách bấm «Huỷ» lúc 09:02:59 (còn miễn phí), mạng lỗi, app tự gửi lại lúc 09:03:30 với **cùng `Idempotency-Key`**. Theo R-IDEM-02, lần gửi lại nhận đúng phản hồi lần đầu — 0 đ, `grace_period` — dù lúc đó đã quá 2 phút. «Now» của phép tính phí là thời điểm **lần xử lý đầu tiên**, không phải lúc gửi lại.

### 4.6. Ánh xạ AC → test case

Bảng dưới là phần Confirmation đã thành bằng chứng. Mỗi kịch bản trỏ về một hoặc nhiều AC; một AC có thể cần nhiều kịch bản.

| AC | Kịch bản (lab) | Ghi chú |
|---|---|---|
| AC-01 | SC-01 | ca dương đơn giản nhất |
| AC-02 | SC-02, SC-03 (biên), SC-13, SC-15 (biên + gửi lại) | |
| AC-03 | SC-04 (biên), SC-05, SC-06, SC-07 | đủ hai mức phí, `arriving`, đồ ăn |
| AC-04 | SC-08 (biên), SC-09 (biên), SC-10 | SC-10 chờ PO xác nhận Q-04 |
| AC-05 | SC-11 | ca âm, kiểm cả nhật ký không đổi |
| AC-06 | SC-12, SC-13 | |
| AC-07 | SC-01, SC-04, SC-14, SC-15 | cách thu: không thu, tiền mặt, VíXanh, không thu trùng |
| AC-08 | SC-16 (*todo*) | chờ trả lời Q-01 |

Soát ba ràng buộc CTFL:

- **Phủ mọi đặc tính của story**: 7 AC có căn cứ đều có ít nhất một kịch bản (lab tự kiểm). AC-08 cố ý chưa có — và hiện rõ ra là chưa có, thay vì bị quên.
- **Không vượt quá story**: không kịch bản nào kiểm cước chuyến, voucher hay quy tắc nhận chuyến. SC-07 chạm R-GEN-04 (đồ ăn luôn là xe máy) chỉ vì AC-03 nói «mọi đơn đồ ăn».
- **Không hai test case cùng một đặc tính**: SC-02 và SC-03 cùng AC-02 nhưng khác đặc tính (giữa phân vùng / sát biên); SC-04 và SC-05 khác biên và khác loại xe. Nếu có thêm «huỷ 1 phút 30 giây» thì đó là kịch bản thừa — trùng đặc tính của SC-02.

## 5. Thực hành

### 5.1. Tổ chức mã trong lab

| Tệp | Vai trò |
|---|---|
| `cau-chuyen.ts` | User story, AC-01…AC-08 dạng rule, bảng vào–ra TD-01…TD-10, 5 «ví dụ điển hình» VD-1…VD-5 |
| `kich-ban.ts` | 15 kịch bản SC-01…SC-15 là **dữ liệu có kiểu**; `vietKichBan` in thành văn bản Given/When/Then |
| `chay-kich-ban.ts` | Step runner: mỗi loại bước có đúng một hàm thực thi; gọi API thật bằng supertest, `FixedClock` đặt giờ từng bước |
| `buggy-cancel-fee.ts` | Bản chép hàm phí có defect cố ý ở hai phép so sánh biên |
| `acceptance-criteria.test.ts` | 32 ca vitest + 1 todo |

### 5.2. Kịch bản là dữ liệu, không phải chuỗi tự do

Cucumber nối câu tự nhiên với code bằng biểu thức chính quy. Lab không thêm thư viện, nên đi đường ngược lại: **bước là union type của TypeScript**, câu tự nhiên được sinh ra từ dữ liệu. Sai chính tả một bước là lỗi biên dịch, không phải «step undefined» lúc chạy.

```ts
// kich-ban.ts (trích)
export type Then =
  | { buoc: 'trang_thai'; la: TripStatus }
  | { buoc: 'phi'; dong: number }
  | { buoc: 'ly_do'; la: FeeReason }
  | { buoc: 'loi'; http: number; ma: string }
  | { buoc: 'thu_vi'; dong: number }
  | { buoc: 'phat_lai_y_nguyen' } /* … */;

{ id: 'SC-08', ten: 'đúng mốc nhận + ETA + 10 phút vẫn thu phí', ac: ['AC-04'], spec: ['R-CANCEL-04'], bien: true,
  given: [{ buoc: 'co_chuyen', xe: 'bike' }, { buoc: 'tai_xe_nhan', eta: 5, luc: '09:01:00' }],
  when:  [{ buoc: 'khach_huy', luc: '09:16:00.000' }],
  then:  [{ buoc: 'phi', dong: 10_000 }, { buoc: 'ly_do', la: 'charged' }] },
```

Đổi lại, BA không tự sửa được tệp `.ts` như sửa tệp `.feature`. Mục 6 bàn khi nào nên chọn cách nào.

### 5.3. Step runner: mã hỗ trợ của ATDD

Mỗi kịch bản chạy trên một hệ thống mới (kho bộ nhớ, `FixedClock` bắt đầu 09:00:00 ngày 04/11/2026, ngoài cao điểm), nên các kịch bản độc lập nhau. Bước Given dựng chuyến qua chính API (`POST /api/quotes`, `/api/trips`, `/accept`, `/arrive`…), không chèn dữ liệu thẳng vào kho — tiền điều kiện được tạo bằng đường đi thật.

```ts
// chay-kich-ban.ts (trích)
case 'phi':
  // Kết quả trả về (nếu có) và giá trị lưu trong chuyến phải cùng khớp (R-CANCEL-07).
  if (cuoi.status === 200 && 'cancellationFee' in cuoi.body) expect(cuoi.body.cancellationFee).toBe(t.dong);
  expect(trip.cancellationFee).toBe(t.dong);
  return;
case 'thu_vi':
  expect(c.system.payments.calls).toEqual([
    expect.objectContaining({ idempotencyKey: `trip:${c.tripId}:cancel-fee`, amount: t.dong, customerId: 'C001' }),
  ]);
  return;
```

Bước `phi` kiểm **hai** nơi: phản hồi API và chuyến đã lưu. Một bản cài đặt trả đúng số trong phản hồi nhưng quên lưu `cancellationFee` sẽ trượt ở đây — kết quả mong đợi phủ mọi đầu ra quan sát được, đúng tinh thần [TCD-04](/articles/test-case-design-expected-result-test-oracle).

### 5.4. Chạy

```bash
cd examples/toi-noi-testing
npx vitest run articles/test-case-design-user-story-acceptance-criteria --reporter=verbose
```

Kết quả thật (rút gọn):

```text
✓ … 1. kiểm chính bộ acceptance criteria > ma trận AC → kịch bản: 7 AC có căn cứ, 15 kịch bản, 5 kịch bản nằm trên biên; …
✓ … 2. acceptance test dạng scenario (Given/When/Then) qua API > SC-04 · AC-03 AC-07 · huỷ đúng 2 phút sau khi nhận thì mất 10.000 đ (xe máy, tiền mặt)
✓ … 2. … > SC-09 · AC-04 · quá mốc 1 ms thì miễn phí vì tài xế trễ
✓ … 2. … > SC-15 · AC-02 AC-07 · bấm lại sau mốc 2 phút với cùng khoá: nhận lại kết quả lần đầu
↓ … 2. … > SC-16 · AC-08 · trước khi xác nhận, khách thấy phí dự kiến — chờ BA quyết (SPEC chưa có API xem trước phí)
✓ … 4. ví dụ trên biên trong AC bắt defect so sánh biên > bảng 10 dòng có ví dụ trên biên: bản lỗi trượt đúng 3 dòng TD-03, TD-04, TD-06
✓ … 5. mơ hồ trong AC: «now» … > khách nhìn lúc 09:02:59,500 (còn miễn phí) nhưng yêu cầu tới máy chủ lúc 09:03:00,200 ⇒ bị thu 10.000 đ

 Test Files  1 passed (1)
      Tests  32 passed | 1 todo (33)
```

32 ca đạt = 4 ca kiểm chính bộ AC (mã R-… có trong SPEC, mỗi AC có kịch bản, ma trận AC → kịch bản, văn bản Given/When/Then) + 15 kịch bản qua API + 10 dòng bảng rule + 2 ca về defect + 1 ca minh hoạ mơ hồ. Ca todo là SC-16 của AC-08.

### 5.5. Vì sao AC cần ví dụ trên biên: một defect thật sự dễ mắc

`buggy-cancel-fee.ts` chép hàm phí và đổi đúng hai ký tự — lỗi kinh điển khi dịch câu «trong 2 phút» và «trễ quá 10 phút» thành code:

```ts
// đúng:  since <  2 phút ⇒ grace_period;   since >  ETA + 10 phút ⇒ driver_late
// lỗi:   since <= 2 phút ⇒ grace_period;   since >= ETA + 10 phút ⇒ driver_late
```

| Bộ ví dụ trong AC | Số dòng | Bắt được bản lỗi? |
|---|---|---|
| «Điển hình»: đang tìm, 1′, 5′, 20′, đã đón | 5 | **Không** — cả 5 dòng xanh |
| Bảng rule TD-01…TD-10 có giá trị biên | 10 | Có — trượt TD-03, TD-04 (đúng 2′00″) và TD-06 (đúng 15′00″) |

Bộ «điển hình» là thứ người ta hay viết khi AC chỉ có văn xuôi. Nó đạt 100% «AC có ví dụ» mà vẫn để lọt defect gây khiếu nại nhiều nhất. Điều này không mâu thuẫn với nguyên tắc «không hai test case cùng một đặc tính»: biên chính là **một đặc tính riêng** của quy tắc.

### 5.6. Nhịp ATDD trong sprint

Trong dự án thật, thứ tự là: buổi example mapping → AC + kịch bản được PO duyệt → kịch bản vào repo, **đỏ** (chưa có code) → developer viết code và mã hỗ trợ → kịch bản **xanh** → story được coi là «done» khi mọi kịch bản xanh và mọi câu hỏi đỏ đã đóng. Lab bắt đầu ở bước đã xanh vì `src/` đã có; SC-16 cho thấy trạng thái «đã viết trước, chưa có code» trông thế nào trong báo cáo: một dòng `todo`, nhìn thấy được trong mỗi lần chạy, thay vì một ghi chú bị lãng quên trong biên bản họp.

## 6. Góc nhìn nâng cao

**Typed data hay Gherkin + Cucumber?** Tệp `.feature` cho phép PO tự sửa và đọc trên Git; đổi lại cần thư viện, biểu thức chính quy cho từng bước, và lỗi chỉ lộ khi chạy. Dữ liệu có kiểu được trình biên dịch kiểm, refactor an toàn, không thêm phụ thuộc; đổi lại người không lập trình phải đọc qua bản in. Quy tắc thực hành: nếu business **thật sự** đọc và sửa kịch bản hằng tuần, chọn Gherkin; nếu chỉ developer và tester đụng tới, dữ liệu có kiểu cộng bản in tự động thường đủ.

**AC không phải toàn bộ bộ test.** AC chứng minh story được chấp nhận; chúng không thay bảng quyết định đầy đủ, kiểm thử tranh chấp khi hai lệnh cùng tới một chuyến, hay kiểm thử bảo mật. Bài BA [Acceptance criteria Given–When–Then](/articles/ba-acceptance-criteria-given-when-then) cũng nhấn mạnh điểm này từ phía nghiệp vụ.

**Kịch bản mệnh lệnh làm hỏng ATDD.** «Given tôi mở trang `/`, nhập C001, chọn SG, bấm Báo giá…» biến AC thành kịch bản UI giòn: đổi một nút là hỏng mười kịch bản, mà quy tắc phí không đổi gì. Giữ Given/When/Then ở mức nghiệp vụ; chi tiết UI để cho test E2E riêng.

**Thời gian là dữ liệu.** Mọi kịch bản ở đây phụ thuộc giờ. Không có `FixedClock`, SC-03 (1′59,999″) và SC-09 (+1 ms) không thể viết thành test tất định. Cách giữ thời gian ổn định cho từng test case có ở [TCD-05 · Test data](/articles/test-case-design-test-data).

**AC tốt vẫn có thể sai ý.** SC-10 xanh vì hệ thống làm đúng chữ của R-CANCEL-04. Nếu PO trả lời Q-04 rằng «tài xế đã tới thì không miễn phí nữa», SC-10 phải đổi kết quả mong đợi, SPEC phải đổi, code phải đổi — và đó là **thay đổi yêu cầu**, không phải defect. Phân biệt hai thứ này cần truy vết từ kịch bản về AC về SPEC; [TCD-08](/articles/test-case-design-traceability-matrix) dựng công cụ cho việc đó.

| Sai lầm thường gặp | Cách tránh |
|---|---|
| AC chép nguyên câu story («khách biết trước phí») | Hỏi «kiểm câu này thế nào?»; không trả lời được ⇒ chưa phải AC |
| Chỉ có ví dụ «điển hình», không có ví dụ trên biên | Mỗi mốc số trong AC có hai ví dụ sát mốc (mục 5.5) |
| Then kiểm một đầu ra (số tiền) | Kiểm mọi đầu ra quan sát được: lý do, trạng thái, lời gọi ví, giá trị lưu |
| Câu hỏi mở bị ghi vào biên bản rồi quên | Biến thành kịch bản `todo` hoặc AC «giả định», hiện trong mỗi lần chạy |
| Một kịch bản kiểm nhiều hành vi | Một When nghiệp vụ mỗi kịch bản |
| Kịch bản phụ thuộc nhau (SC sau dùng chuyến của SC trước) | Mỗi kịch bản dựng hệ thống và dữ liệu riêng |

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 (K3).** PO trả lời Q-01: làm màn hình xác nhận huỷ, hiện «Phí dự kiến: 10.000 đ» hoặc «Miễn phí nếu huỷ trước 09:03:00». Viết AC-08 dạng rule và ba kịch bản Given/When/Then cho nó, trong đó có ít nhất một kịch bản xử lý Q-02 (phí trên màn hình khác phí thật).

<details><summary><b>Lời giải</b></summary>
<p>AC-08 (rule): «Khi khách bấm "Huỷ chuyến", hệ thống hiện phí dự kiến tính theo giờ máy chủ tại thời điểm hiện màn hình, và — nếu đang trong thời gian miễn phí — mốc giờ hết miễn phí. Nếu phí thật khi xác nhận khác phí dự kiến, hệ thống không huỷ mà hiện lại màn hình với phí mới.» Ba kịch bản: (1) <i>Given</i> tài xế nhận lúc 09:01:00, <i>When</i> khách bấm Huỷ lúc 09:01:30, <i>Then</i> màn hình hiện «Miễn phí nếu huỷ trước 09:03:00». (2) <i>Given</i> như trên, <i>When</i> khách bấm Huỷ lúc 09:04:00, <i>Then</i> màn hình hiện «Phí dự kiến: 10.000 đ». (3) <i>Given</i> màn hình hiện «Miễn phí» lúc 09:02:59, <i>When</i> khách xác nhận và máy chủ nhận lúc 09:03:00,200, <i>Then</i> chuyến <b>không</b> bị huỷ, màn hình hiện lại «Phí dự kiến: 10.000 đ». Kịch bản (3) là quyết định nghiệp vụ mới — phải được PO duyệt, và kéo theo thay đổi API (SPEC hiện không có), nên story xem trước phí nên tách riêng (tiêu chí Small của INVEST).</p>
</details>

**Bài tập 2 (K3).** Thêm kịch bản cho ô tô trả VíXanh nhưng ví từ chối (`decline`). AC nào chi phối? Viết Then đầy đủ. Bước nào của step runner cần thêm?

<details><summary><b>Lời giải</b></summary>
<p>AC-07 cùng R-PAY-04 (bị từ chối thì không thử lại) và R-PAY-06 (thanh toán không chặn nghiệp vụ). <i>Given</i> chuyến ô tô VíXanh, tài xế nhận 09:01:00, ví giả ở chế độ từ chối; <i>When</i> khách huỷ lúc 09:06:00; <i>Then</i> chuyến <code>cancelled</code>, phí 20.000 đ, lý do <code>charged</code>, ví được gọi <b>đúng một lần</b> với khoá <code>trip:&lt;id&gt;:cancel-fee</code>, <code>paymentStatus</code> = <code>failed</code>, API trả 200. Cần thêm một bước Given (ví dụ <code>{ buoc: 'vi_tu_choi' }</code> gọi <code>system.payments.failNext('decline')</code>) và mở rộng kiểu của bước <code>payment_status</code> để nhận <code>'failed'</code>. Chú ý: kịch bản này kiểm một đặc tính mới (thất bại thu tiền), không trùng SC-14.</p>
</details>

**Câu 1 (K2 — FL-4.5.2).** Acceptance criterion nào dưới đây viết theo dạng **rule-oriented**?

a) Given chuyến đã được nhận lúc 09:01, When khách huỷ lúc 09:03, Then phí là 10.000 đ
b) Bảng: trạng thái × thời gian kể từ lúc nhận × loại xe → phí, lý do
c) Một đoạn video quay màn hình huỷ chuyến
d) «Khách hài lòng với cách tính phí huỷ»

<details><summary><b>Đáp án: b</b></summary>
<p>CTFL nêu dạng rule-oriented gồm danh sách kiểm (bullet) hoặc bảng ánh xạ vào–ra — b. a là dạng scenario-oriented (Given/When/Then). c không phải AC. d không kiểm chứng được, nên không phải AC định nghĩa tốt dù viết ở dạng nào.</p>
</details>

**Câu 2 (K2 — FL-4.5.2).** Theo CTFL v4.0.1, mục đích nào **không** nằm trong danh sách công dụng của acceptance criteria?

a) Xác định phạm vi của user story
b) Mô tả cả kịch bản dương và âm
c) Thay thế việc review user story
d) Làm cơ sở cho acceptance testing của user story

<details><summary><b>Đáp án: c</b></summary>
<p>CTFL liệt kê: xác định phạm vi, đạt đồng thuận, mô tả kịch bản dương và âm, làm cơ sở cho acceptance testing, giúp lập kế hoạch và ước lượng. AC không thay review — chính việc viết AC trong specification workshop là một dạng review story.</p>
</details>

**Câu 3 (K3 — FL-4.5.3).** Đội dùng ATDD cho AC-04 («quá hẳn mốc lúc nhận + ETA + 10 phút thì miễn phí; đúng mốc vẫn thu»). ETA 5 phút, nhận lúc 09:01:00. Bộ test case nào là tốt nhất?

a) Huỷ lúc 09:20:00
b) Huỷ lúc 09:16:00,000 và 09:16:00,001
c) Huỷ lúc 09:10:00, 09:15:00, 09:20:00, 09:25:00
d) Huỷ lúc 09:16:00,000, 09:16:00,001 và 09:30:00

<details><summary><b>Đáp án: b</b></summary>
<p>Đặc tính của AC-04 là mốc: thu ở đúng mốc, miễn khi vượt mốc. b phủ đủ cả hai phía bằng hai giá trị sát mốc (BVA hai giá trị). a chỉ một phía, không chạm mốc. c không chạm mốc 09:16:00 nên lọt lỗi <code>&gt;=</code>. d thêm 09:30:00 trùng đặc tính với 09:16:00,001 — vi phạm «không có hai test case mô tả cùng một đặc tính».</p>
</details>

**Câu 4 (K2 — 4.5.3).** Theo CTFL, trong ATDD thứ tự thường gặp khi tạo test case là gì?

a) Ca âm → ca dương → phi chức năng
b) Ca dương → ca âm → phi chức năng
c) Phi chức năng → ca dương → ca âm
d) Không có thứ tự; viết theo thứ tự AC

<details><summary><b>Đáp án: b</b></summary>
<p>CTFL: thường viết ca dương trước (chuỗi hoạt động khi mọi thứ như mong đợi), rồi ca âm, cuối cùng là đặc tính phi chức năng như hiệu năng, khả dụng. Ở bài: SC-01…SC-10 dương, SC-11 âm; phi chức năng (ví dụ thời gian phản hồi lệnh huỷ) chưa có trong story.</p>
</details>

**Câu 5 (K2 — 4.5.1).** Trong buổi refinement, không ai trả lời được «kiểm "biết trước phí" thế nào». Theo CTFL, điều đó gợi ý gì?

a) Story phải bỏ khỏi backlog
b) Story có thể chưa đủ rõ, chưa phản ánh giá trị thật, hoặc stakeholder cần giúp đỡ để kiểm
c) Tester chưa đủ năng lực
d) Story đạt tiêu chí Testable của INVEST

<details><summary><b>Đáp án: b</b></summary>
<p>Đó gần như nguyên văn nhận xét của CTFL 4.5.1. a quá mức — làm rõ hoặc tách story là đủ. c đổ lỗi sai chỗ. d ngược lại: không biết kiểm thế nào nghĩa là <b>chưa</b> Testable.</p>
</details>

**Câu 6 (K3).** Kịch bản nào là **thừa** nếu bộ đã có SC-02 (huỷ 1′ sau khi nhận ⇒ 0 đ) và SC-03 (1′59,999″ ⇒ 0 đ)?

a) Huỷ 1′30″ sau khi nhận ⇒ 0 đ
b) Huỷ đúng 2′00″ ⇒ 10.000 đ
c) Tài xế khác nhận lại, huỷ 1′ sau lần nhận mới ⇒ 0 đ
d) Đã báo tới, huỷ 1′ sau khi nhận ⇒ 10.000 đ

<details><summary><b>Đáp án: a</b></summary>
<p>a nằm giữa phân vùng «chưa đủ 2 phút» giống SC-02, không thêm đặc tính nào. b là biên phía có phí (SC-04). c kiểm việc đặt lại <code>acceptedAt</code> (SC-13). d kiểm <code>arriving</code> không có thời gian miễn phí (SC-06). Ba kịch bản b, c, d đều là đặc tính khác.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB — Certified Tester Foundation Level Syllabus v4.0.1, mục 4.5 Collaboration-based Test Approaches: 4.5.1 Collaborative User Story Writing; 4.5.2 Acceptance Criteria, LO FL-4.5.2 (K2); 4.5.3 Acceptance Test-driven Development (ATDD), LO FL-4.5.3 (K3); mục 2.1.3 Testing as a Driver for Software Development: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary (tra *acceptance criteria*, *user story*, *acceptance test-driven development*): https://glossary.istqb.org/
- R. Jeffries, A. Anderson, C. Hendrickson — *Extreme Programming Installed*, Addison-Wesley, 2000 (nguồn của Card – Conversation – Confirmation).
- B. Wake — «INVEST in Good Stories, and SMART Tasks», 2003.
- M. Wynne — «Introducing Example Mapping», Cucumber blog, 2015 (ngoài ISTQB).
- G. Adzic — *Specification by Example*, Manning, 2011 (ngoài ISTQB).
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-CANCEL-01…07, R-STATE-03/04, R-UI-01, R-IDEM-02, R-PAY-05; mã bài tại `articles/test-case-design-user-story-acceptance-criteria/`.
- Bài liên quan: [TCD-02 · Giải phẫu test case](/articles/test-case-design-giai-phau-test-case) · [TCD-04 · Expected result và test oracle](/articles/test-case-design-expected-result-test-oracle) · [TCD-06 · Phối hợp kỹ thuật](/articles/test-case-design-phoi-hop-ky-thuat) · [TEST-016 · Acceptance Testing](/articles/acceptance-testing-tong-quan) · [TEST-033 · Decision Table](/articles/decision-table-testing) · [SYS-15 · UAT](/articles/system-testing-uat) · [BA · Acceptance criteria Given–When–Then](/articles/ba-acceptance-criteria-given-when-then) · [BA · User story](/articles/ba-user-story-vai-tro-nhu-cau-gia-tri).

Bài tiếp theo: [TCD-08 · Ma trận truy vết](/articles/test-case-design-traceability-matrix).
