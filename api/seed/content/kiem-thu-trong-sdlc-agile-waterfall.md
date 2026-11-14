---
slug: kiem-thu-trong-sdlc-agile-waterfall
title: "Kiểm thử trong SDLC: Waterfall, Agile và DevOps"
summary: "Mô hình phát triển quyết định test lúc nào, tài liệu dày đến đâu, tự động hoá bao nhiêu: Waterfall, Agile, DevOps, test-first, shift left và retrospective theo CTFL v4.0.1, qua tính năng phí huỷ chuyến."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-008, sdlc, agile, waterfall, devops, shift-left, tdd]
language: vi
---

Cùng một quy tắc «khách huỷ sau 2 phút thì mất 10.000 đ», nhưng một đội test quy tắc này ở tuần thứ 10 của dự án, đội khác test ngay ngày thứ ba của sprint — và cả hai đều có thể đúng. Bài này trả lời câu hỏi: **mô hình phát triển phần mềm (SDLC) ảnh hưởng tới kiểm thử thế nào**, những thực hành tốt nào dùng được cho mọi mô hình, và DevOps, test-first, shift left, retrospective thay đổi công việc của tester ra sao. Ví dụ xuyên suốt là tính năng phí huỷ chuyến (R-CANCEL) của Tới Nơi.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-008</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S01 · Fundamentals of Software Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/quy-trinh-kiem-thu-test-process">TEST-007 Quy trình kiểm thử</a>, <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Giấy/bảng tính: so sánh cách test phí huỷ chuyến (R-CANCEL) trong dự án Waterfall 3 tháng và sprint 2 tuần có CI; kèm bộ test chấp nhận chạy được trong <code>examples/toi-noi-testing/articles/kiem-thu-trong-sdlc-agile-waterfall/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 2 · mục 2.1 (2.1.1–2.1.6) · FL-2.1.1 (K2), FL-2.1.2 (K1), FL-2.1.3 (K1), FL-2.1.4 (K2), FL-2.1.5 (K2), FL-2.1.6 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

[TEST-007](/articles/quy-trinh-kiem-thu-test-process) cho ta bảy nhóm hoạt động test và nhấn mạnh: quy trình phải may đo theo ngữ cảnh. Trong các yếu tố ngữ cảnh, **SDLC** là yếu tố làm quy trình đổi hình dạng mạnh nhất. SDLC quyết định:

- tester bắt đầu **chạy** được phần mềm từ tuần nào;
- test plan là tài liệu 30 trang được ký duyệt hay vài dòng trong sprint backlog;
- regression được làm bằng tay mỗi đợt release hay chạy tự động sau mỗi commit.

Hiểu điều này giúp tránh hai sai lầm hay gặp: mang nguyên quy trình test của dự án Waterfall vào một đội Scrum (chậm, nặng giấy tờ), hoặc ngược lại tin rằng «Agile thì không cần test plan». Năm test level (component, integration, system…) cũng gắn với SDLC, nhưng chúng là chủ đề của Series 02 — bài này chỉ nhắc khi cần.

## 2. Lý thuyết theo ISTQB

### 2.1. Mô hình SDLC và phương pháp phát triển

Theo CTFL v4.0.1 (mục 2.1), **mô hình SDLC** là bức tranh trừu tượng, mức cao về quy trình phát triển: các pha và loại hoạt động liên hệ với nhau thế nào về logic và thời gian. Syllabus chia ví dụ thành ba nhóm:

| Nhóm mô hình | Ví dụ trong syllabus | Ý chính |
|---|---|---|
| Tuần tự (sequential) | Waterfall, V-model | Các pha nối tiếp; code chạy được xuất hiện ở các pha sau |
| Lặp (iterative) | Spiral, prototyping | Lặp lại chu kỳ để tinh chỉnh sản phẩm |
| Tăng dần (incremental) | Unified Process | Mỗi lần thêm một phần chức năng |

Bên dưới mô hình là các **phương pháp và thực hành** chi tiết hơn: ATDD, BDD, DDD, XP, FDD, Kanban, Lean IT, Scrum, TDD. Chú ý cách syllabus xếp: Scrum, Kanban là phương pháp; «Agile» là cách tiếp cận dùng các phương pháp đó, thường trên nền mô hình lặp và tăng dần.

### 2.2. SDLC ảnh hưởng tới kiểm thử ra sao (FL-2.1.1)

Syllabus nêu năm khía cạnh chịu ảnh hưởng của SDLC: **phạm vi và thời điểm** của hoạt động test (gồm test level, test type), **mức chi tiết** của tài liệu test, **lựa chọn kỹ thuật và cách tiếp cận**, **mức tự động hoá**, và **vai trò, trách nhiệm** của tester.

| Khía cạnh | Tuần tự (Waterfall, V-model) | Lặp / tăng dần | Agile |
|---|---|---|---|
| Khi nào test động | Muộn — code chạy được ở các pha sau; giai đoạn đầu tester review yêu cầu, làm test analysis và test design | Mỗi iteration có bản chạy được ⇒ static và dynamic testing ở mọi test level trong từng iteration | Như mô hình lặp, nhịp ngắn |
| Tài liệu | Thường chi tiết, có ký duyệt | Tuỳ dự án | Nhẹ — vì thay đổi xảy ra suốt dự án |
| Kỹ thuật | Thiết kế trước, có hệ thống | Kết hợp | Phần lớn test thủ công dùng kỹ thuật dựa trên kinh nghiệm (experience-based), ít phân tích trước |
| Tự động hoá | Có thể ít | Cần cho regression vì chuyển giao thường xuyên | Nhiều — để regression dễ làm |
| Phản hồi | Chậm | Cần nhanh | Cần rất nhanh |

Mấu chốt: giao từng phần thường xuyên kéo theo **phản hồi nhanh và regression rộng**; không tự động hoá thì regression sẽ ngốn hết thời gian test.

### 2.3. Thực hành tốt cho mọi SDLC (FL-2.1.2)

Dù mô hình nào, syllabus nêu bốn thực hành tốt:

1. **Mỗi hoạt động phát triển có một hoạt động test tương ứng**, để mọi hoạt động phát triển đều được kiểm soát chất lượng. *Tới Nơi:* viết đặc tả phí huỷ ⇒ có review đặc tả; viết `cancel.ts` ⇒ có unit test.
2. **Mỗi test level có mục tiêu riêng, khác nhau**, để test đủ rộng mà không trùng lặp. *Tới Nơi:* unit test kiểm mốc 2 phút; system test kiểm luồng huỷ + thu phí qua VíXanh, không lặp lại mọi mốc thời gian.
3. **Test analysis và test design của một test level bắt đầu ngay trong pha phát triển tương ứng**, theo nguyên lý test sớm. *Tới Nơi:* thiết kế test chấp nhận cho R-CANCEL ngay khi viết user story.
4. **Tester review sản phẩm công việc ngay khi có bản nháp**, hỗ trợ shift left. *Tới Nơi:* review bản nháp SPEC R-CANCEL trước khi dev bắt đầu code.

### 2.4. Test-first: TDD, ATDD, BDD (FL-2.1.3)

Ba cách tiếp cận dùng test để **dẫn dắt** phát triển. Cả ba thực hiện nguyên lý test sớm, đi theo shift left (test được định nghĩa trước khi viết code) và hỗ trợ mô hình lặp.

| Cách tiếp cận | Đặc điểm theo syllabus | Ví dụ ở Tới Nơi |
|---|---|---|
| **TDD** — Test-Driven Development | Test case dẫn dắt việc viết code (thay cho thiết kế dài); viết test trước, viết code cho test qua, rồi refactor cả hai | Dev viết test «đúng 2 phút ⇒ 10.000 đ» trước, rồi mới viết `customerCancellationFee` |
| **ATDD** — Acceptance Test-Driven Development | Suy test từ **tiêu chí chấp nhận** trong lúc thiết kế hệ thống; viết test trước khi phát triển phần đó | PO, dev, tester chốt AC-1…AC-7 cho story «Khách huỷ chuyến» |
| **BDD** — Behavior-Driven Development | Mô tả hành vi bằng ngôn ngữ tự nhiên đơn giản, dễ hiểu với stakeholder, thường dạng Given/When/Then; sau đó nên chuyển tự động thành test chạy được | Kịch bản Gherkin ở mục 5.3 |

Với cả ba, test có thể ở lại dưới dạng **test tự động** để giữ chất lượng code khi sửa đổi, refactor sau này.

### 2.5. DevOps và kiểm thử (FL-2.1.4)

**DevOps** là cách tiếp cận tổ chức đưa phát triển (gồm cả testing) và vận hành cùng làm việc vì mục tiêu chung. DevOps đòi hỏi thay đổi văn hoá, coi trọng ngang nhau hai phía; khuyến khích đội tự chủ, phản hồi nhanh, chuỗi công cụ tích hợp và các thực hành **CI** (continuous integration) và **CD** (continuous delivery), qua một **delivery pipeline**.

| Lợi ích cho testing | Rủi ro / thách thức |
|---|---|
| Phản hồi nhanh về chất lượng code và tác động của thay đổi lên code cũ | Phải định nghĩa và dựng delivery pipeline |
| CI thúc đẩy shift left: dev nộp code kèm component test và static analysis | Phải đưa vào và bảo trì công cụ CI/CD |
| Quy trình tự động giúp có môi trường test ổn định | Test automation tốn thêm nguồn lực, khó dựng và khó bảo trì |
| Tăng khả năng nhìn thấy đặc tính phi chức năng (hiệu năng, độ tin cậy) | |
| Bớt test thủ công lặp đi lặp lại | |
| Rủi ro regression giảm nhờ quy mô regression tự động | |

Syllabus nhấn mạnh: dù tự động hoá nhiều, **test thủ công — đặc biệt từ góc nhìn người dùng — vẫn cần**.

### 2.6. Shift left (FL-2.1.5)

Nguyên lý test sớm đôi khi được gọi là **shift left**: đưa test về sớm hơn trong SDLC (không chờ code xong, không chờ tích hợp xong) — nhưng **không có nghĩa bỏ qua test ở giai đoạn sau**. Các thực hành syllabus nêu:

- review đặc tả từ góc nhìn tester (thường tìm ra chỗ mơ hồ, thiếu, mâu thuẫn);
- viết test case trước code và cho code chạy trong test harness khi đang cài đặt;
- dùng CI, tốt hơn nữa là CD, để có phản hồi nhanh và component test tự động đi kèm mã nguồn;
- static analysis trước dynamic testing, hoặc trong quy trình tự động;
- làm non-functional testing từ test level component khi có thể.

Shift left có thể tốn thêm đào tạo, công sức, chi phí **lúc đầu** nhưng được kỳ vọng tiết kiệm **về sau**; và cần stakeholder đồng thuận. Shift left có bài riêng ở Series 03 (Static Testing).

### 2.7. Retrospective và cải tiến quy trình (FL-2.1.6)

**Retrospective** thường diễn ra cuối dự án hoặc iteration, ở mốc release, hoặc khi cần; thời điểm và cách tổ chức tuỳ SDLC. Người tham gia — không chỉ tester mà cả developer, kiến trúc sư, product owner, BA — bàn ba câu: *điều gì thành công, nên giữ? điều gì chưa thành công, cần cải tiến? làm sao đưa cải tiến vào và giữ thành công trong tương lai?* Kết quả nên được ghi lại, thường là một phần của test completion report, và **phải theo dõi việc thực hiện** cải tiến. Lợi ích cho testing: test hiệu quả hơn, testware tốt hơn, đội gắn kết và học hỏi, test basis tốt hơn, dev và test hợp tác tốt hơn.

### 2.8. Bảng thuật ngữ

| Thuật ngữ | Nghĩa tiếng Việt | Ghi chú |
|---|---|---|
| software development lifecycle (SDLC) | vòng đời phát triển phần mềm | |
| sequential / iterative / incremental development model | mô hình tuần tự / lặp / tăng dần | |
| test-driven development (TDD) | phát triển hướng kiểm thử | mức code |
| acceptance test-driven development (ATDD) | phát triển hướng kiểm thử chấp nhận | từ tiêu chí chấp nhận |
| behavior-driven development (BDD) | phát triển hướng hành vi | Given/When/Then |
| acceptance criteria | tiêu chí chấp nhận | |
| continuous integration / delivery (CI/CD) | tích hợp / chuyển giao liên tục | |
| delivery pipeline | chuỗi chuyển giao (pipeline) | |
| shift left | dịch trái — test sớm | keyword chương 2 |
| retrospective | buổi nhìn lại | |
| regression testing | kiểm thử hồi quy | chi tiết ở Series 02 |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p><b>Shift right</b> (test và quan sát trên production: canary, giám sát, feature flag), <b>continuous testing</b> như một chiến lược tổ chức, và các mô hình lai kiểu «Water-Scrum-Fall» là thuật ngữ ngành, không có trong CTFL v4.0.1 (syllabus chỉ nhắc continuous testing như một hình thức của test execution). Bài dùng chúng ở mục 6 để so sánh, không phải nội dung thi.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu và business rules

Tính năng: **khách huỷ chuyến và phí huỷ**. Trích `SPEC.md` của lab:

| Mã | Quy tắc (tóm tắt) |
|---|---|
| R-CANCEL-01 | Huỷ khi `searching` ⇒ 0 đ; `accepted` chưa đủ 2 phút ⇒ 0 đ; `accepted` từ 2 phút trở lên hoặc `arriving` ⇒ có phí; trạng thái khác ⇒ `INVALID_TRANSITION` |
| R-CANCEL-02 | «Chưa đủ 2 phút» là `now − acceptedAt < 2 phút`; đúng 2 phút 0 giây đã tính phí |
| R-CANCEL-03 | Xe máy (kể cả đơn đồ ăn) 10.000 đ, ô tô 20.000 đ |
| R-CANCEL-04 | Tài xế trễ: `now > acceptedAt + etaMinutes + 10 phút` (lớn hơn hẳn) ⇒ miễn phí |
| R-CANCEL-05 | Tài xế huỷ hoặc hệ thống huỷ không bao giờ thu phí khách |
| R-CANCEL-06 | Trả lý do: `searching`, `grace_period`, `driver_late`, `charged` |
| R-CANCEL-07 | Phí > 0 thu theo R-PAY-05 (VíXanh với khoá idempotency `trip:<id>:cancel-fee`) |

### 3.2. System context

- **Hệ thống đích** (tài liệu kiến trúc §4.4, §4.9, §9.2): Booking Service nhận lệnh huỷ (idempotent theo khách + khoá), phát `BookingCancelled` qua outbox/Kafka; Trip Service chuyển trạng thái; Payment & Wallet thu phí qua cổng VíXanh có idempotency; nếu đã giữ khuyến mãi hay uỷ quyền thanh toán thì Saga chạy bước bù (release/void). Mỗi service có pipeline CI/CD riêng (§17).
- **Lab:** hàm thuần `customerCancellationFee` trong `src/domain/cancel.ts`, use case `cancelByCustomer` trong `src/app/trips.ts`, ví VíXanh giả có chế độ `timeout`, `lost_response`. Chưa có Kafka, Saga hay pipeline CI/CD thật — lab tương ứng bước «unit/component test» của pipeline.

### 3.3. Hai bối cảnh chuyển giao (giả định)

- **Bối cảnh A — Waterfall 3 tháng.** Tới Nơi thuê một đối tác làm «Gói chính sách huỷ chuyến» theo hợp đồng trọn gói: phạm vi chốt từ đầu, nghiệm thu một lần ở tuần 12, đội test độc lập của Tới Nơi.
- **Bối cảnh B — Scrum, sprint 2 tuần + CI.** Đội sản phẩm Booking tự làm; story «Khách huỷ chuyến» vào sprint, pipeline theo §17 của tài liệu kiến trúc chạy sau mỗi lần merge.

### 3.4. Rủi ro

| Rủi ro | Vì sao liên quan tới SDLC |
|---|---|
| Sai mốc biên (đúng 2 phút, đúng ETA + 10 phút) | Phát hiện muộn ở Waterfall ⇒ sửa tốn kém; ở Agile có thể khoá bằng unit test ngay ngày đầu |
| Thu phí hai lần khi cổng VíXanh timeout rồi thử lại | Cần test tích hợp với ví giả; ở Waterfall chỉ thấy ở pha test hệ thống |
| Huỷ bị chặn khi thu phí lỗi (trái R-PAY-06) | Lỗi ở ranh giới hai service — dễ lọt nếu mỗi đội chỉ test phần mình |
| Luật đổi giữa chừng (vd vận hành muốn 3 phút thay vì 2) | Waterfall: change request, điều chỉnh hợp đồng; Agile: một item backlog, sửa test trong sprint |
| Regression: sửa phí huỷ làm hỏng tính cước | Chuyển giao thường xuyên cần regression tự động rộng |

## 4. Phân tích kỹ thuật

### 4.1. Áp năm tác động của SDLC lên R-CANCEL

| Tác động (FL-2.1.1) | Bối cảnh A — Waterfall | Bối cảnh B — Scrum + CI |
|---|---|---|
| Phạm vi, thời điểm | Tuần 1–3 review đặc tả, tuần 4–6 test analysis/design; test động tuần 9–12 khi đối tác giao bản chạy được | Ngày 1–2 làm rõ AC; test chạy từ lúc commit đầu tiên; demo cuối sprint |
| Mức tài liệu | Test plan riêng có ký duyệt, test case chi tiết từng bước, biên bản nghiệm thu | AC trong story + test tự động là tài liệu sống; test plan vài dòng trong sprint plan |
| Kỹ thuật, cách tiếp cận | Kỹ thuật hộp đen có hệ thống (phân vùng, giá trị biên) làm trước; test theo kịch bản | Biên được khoá bằng test tự động; phần thủ công chủ yếu exploratory dựa trên kinh nghiệm |
| Tự động hoá | Có thể chỉ một phần, nhiều test chạy tay ở pha cuối | Cao: unit + component trong CI, smoke sau deploy |
| Vai trò tester | Tester độc lập, nhận bản giao, báo lỗi về đối tác | Tester trong đội, cùng PO/dev viết AC, pair với dev, lo phần exploratory |

### 4.2. Review sớm tìm gì trong R-CANCEL

Cả hai bối cảnh đều làm được shift left bằng review đặc tả — Waterfall ở tuần 2, Scrum ở buổi làm rõ story. Ba điểm một tester có thể nêu khi review bản nháp R-CANCEL:

1. **Biên phải nói rõ «nhỏ hơn» hay «nhỏ hơn hoặc bằng».** SPEC đã làm đúng việc này (R-CANCEL-02 «đúng 2 phút 0 giây tính là từ 2 phút trở lên», R-CANCEL-04 «lớn hơn hẳn»). Một bản nháp chỉ ghi «trong 2 phút» sẽ sinh ra hai cách cài đặt.
2. **Thứ tự ưu tiên lý do (R-CANCEL-06) có thật sự xảy ra không?** `grace_period` đòi dưới 2 phút từ lúc nhận; `driver_late` đòi quá `etaMinutes + 10` phút, mà ETA ≥ 1 (R-ACCEPT-04) ⇒ quá 11 phút. Hai điều kiện không bao giờ cùng đúng; `searching` thì chưa có `acceptedAt`. Quy tắc ưu tiên vì vậy không bao giờ được kích hoạt — không sai, nhưng là điểm nên hỏi lại để biết người viết có lường trước một trường hợp khác không.
3. **Phí huỷ chuyến tiền mặt thu lúc nào?** R-PAY-05 ghi «thu ở chuyến sau, ngoài phạm vi lab» — tester đánh dấu là ngoài phạm vi test, không để lơ lửng.

### 4.3. Test ở đâu trong pipeline (bối cảnh B)

Tài liệu kiến trúc §17 đặt pipeline mười bước cho mỗi service. Đặt các test của R-CANCEL vào đúng bước giúp phản hồi nhanh mà không lặp lại test:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 262" role="img" aria-label="Pipeline CI/CD mười bước của mỗi service Tới Nơi theo tài liệu kiến trúc mục 17. Hàng trên: 1 static analysis, 2 unit tests, 3 component tests, 4 contract tests, 5 integration tests. Hàng dưới: 6 security scan, 7 build image, 8 deploy dev/test, 9 smoke test, 10 promote staging/production. Các bước kiểm thử tô màu. Ghi chú: test chấp nhận phí huỷ AC-1 đến AC-7 chạy ở bước 2; thu phí qua ví giả timeout chạy ở bước 5; mũi tên phản hồi về developer trong vài phút; lab hiện chỉ có tương đương bước 1 và 2.">
<defs>
<marker id="t008-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="t008-back" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-phys"/></marker>
</defs>
<text x="460" y="20" text-anchor="middle" class="t-small">mỗi lần merge chạy từ trái sang phải; bước tô màu là bước kiểm thử</text>
<rect x="16" y="34" width="168" height="54" rx="8" class="f-pk-soft s-pk"/><text x="100" y="56" text-anchor="middle" class="t-b">1 · Static analysis</text><text x="100" y="74" text-anchor="middle" class="t-small">lint, phân tích tĩnh</text>
<rect x="196" y="34" width="168" height="54" rx="8" class="f-pk-soft s-pk"/><text x="280" y="56" text-anchor="middle" class="t-b">2 · Unit tests</text><text x="280" y="74" text-anchor="middle" class="t-small">luật cước, phí huỷ…</text>
<rect x="376" y="34" width="168" height="54" rx="8" class="f-pk-soft s-pk"/><text x="460" y="56" text-anchor="middle" class="t-b">3 · Component tests</text><text x="460" y="74" text-anchor="middle" class="t-small">service + DB test container</text>
<rect x="556" y="34" width="168" height="54" rx="8" class="f-pk-soft s-pk"/><text x="640" y="56" text-anchor="middle" class="t-b">4 · Contract tests</text><text x="640" y="74" text-anchor="middle" class="t-small">REST/gRPC/Kafka schema</text>
<rect x="736" y="34" width="168" height="54" rx="8" class="f-pk-soft s-pk"/><text x="820" y="56" text-anchor="middle" class="t-b">5 · Integration tests</text><text x="820" y="74" text-anchor="middle" class="t-small">outbox, consumer, webhook</text>
<rect x="16" y="140" width="168" height="54" rx="8" class="f-pk-soft s-pk"/><text x="100" y="162" text-anchor="middle" class="t-b">6 · Security scan</text><text x="100" y="180" text-anchor="middle" class="t-small">SAST, dependency, secret</text>
<rect x="196" y="140" width="168" height="54" rx="8" class="f-surface s-line"/><text x="280" y="162" text-anchor="middle" class="t-b">7 · Build image</text><text x="280" y="180" text-anchor="middle" class="t-small">immutable, ký artifact</text>
<rect x="376" y="140" width="168" height="54" rx="8" class="f-surface s-line"/><text x="460" y="162" text-anchor="middle" class="t-b">8 · Deploy dev/test</text><text x="460" y="180" text-anchor="middle" class="t-small">môi trường test</text>
<rect x="556" y="140" width="168" height="54" rx="8" class="f-pk-soft s-pk"/><text x="640" y="162" text-anchor="middle" class="t-b">9 · Smoke test</text><text x="640" y="180" text-anchor="middle" class="t-small">luồng chính còn sống</text>
<rect x="736" y="140" width="168" height="54" rx="8" class="f-surface s-line"/><text x="820" y="162" text-anchor="middle" class="t-b">10 · Promote</text><text x="820" y="180" text-anchor="middle" class="t-small">staging → prod, duyệt</text>
<line x1="184" y1="61" x2="194" y2="61" class="s-pk" stroke-width="1.6" marker-end="url(#t008-arrow)"/>
<line x1="364" y1="61" x2="374" y2="61" class="s-pk" stroke-width="1.6" marker-end="url(#t008-arrow)"/>
<line x1="544" y1="61" x2="554" y2="61" class="s-pk" stroke-width="1.6" marker-end="url(#t008-arrow)"/>
<line x1="724" y1="61" x2="734" y2="61" class="s-pk" stroke-width="1.6" marker-end="url(#t008-arrow)"/>
<path d="M820 88 L820 114 L100 114 L100 136" class="f-none s-pk" stroke-width="1.6" marker-end="url(#t008-arrow)"/>
<line x1="184" y1="167" x2="194" y2="167" class="s-pk" stroke-width="1.6" marker-end="url(#t008-arrow)"/>
<line x1="364" y1="167" x2="374" y2="167" class="s-pk" stroke-width="1.6" marker-end="url(#t008-arrow)"/>
<line x1="544" y1="167" x2="554" y2="167" class="s-pk" stroke-width="1.6" marker-end="url(#t008-arrow)"/>
<line x1="724" y1="167" x2="734" y2="167" class="s-pk" stroke-width="1.6" marker-end="url(#t008-arrow)"/>
<path d="M280 194 L280 230 L60 230 L60 196" class="f-none s-phys dash" stroke-width="1.4" marker-end="url(#t008-back)"/>
<text x="170" y="248" text-anchor="middle" class="t-small t-phys">lỗi ở bước 1–2 ⇒ dev biết trong vài phút</text>
<text x="600" y="222" text-anchor="middle" class="t-small">R-CANCEL AC-1…AC-7 chạy ở bước 2 · ví giả timeout / lost_response ở bước 5</text>
<text x="600" y="240" text-anchor="middle" class="t-small">lab hiện chỉ có tương đương bước 1–2 (tsc strict, vitest)</text>
</svg>
</div>
<figcaption><b>Hình 1 · Pipeline của mỗi service theo tài liệu kiến trúc §17.</b> Phần lớn bước là kiểm thử — đó là ý «CI thúc đẩy shift left» của syllabus. Bước 7, 8, 10 là build/deploy. Pipeline chỉ giảm test thủ công lặp lại; phiên exploratory trên app vẫn nằm ngoài pipeline.</figcaption>
</figure>

| Test cho R-CANCEL | Bước pipeline | Lý do đặt ở đó |
|---|---|---|
| Mốc 2 phút, ETA + 10 phút, mức phí theo loại xe | 2 · Unit | Hàm thuần, chạy mili giây, khoá biên ngay |
| `cancelByCustomer` lưu phí, trả tài xế về `online` | 3 · Component | Cần kho dữ liệu (PostgreSQL test) |
| Schema sự kiện `BookingCancelled` | 4 · Contract | Booking và Payment deploy độc lập |
| Thu phí qua VíXanh: timeout rồi thử lại cùng khoá, không trừ hai lần | 5 · Integration | Cần ví giả / sandbox |
| Huỷ một chuyến trên môi trường test | 9 · Smoke | Luồng chính còn sống sau deploy |
| Thông báo phí có dễ hiểu với khách không | Ngoài pipeline | Test thủ công từ góc nhìn người dùng |

## 5. Thực hành

### 5.1. So sánh hai bối cảnh: khi nào, ai, tài liệu gì

Đây là phần thực hành của bài: cùng tính năng R-CANCEL, hai cách tổ chức test.

| Câu hỏi | A — Waterfall 3 tháng (13 tuần) | B — Scrum, sprint 2 tuần + CI |
|---|---|---|
| Tester tham gia từ khi nào | Tuần 1: review đặc tả hợp đồng | Buổi refinement trước sprint |
| Test analysis, test design | Tuần 4–6, cho cả gói, trước khi có code | Ngày 1–2 của sprint, cho từng story |
| Chạy test động lần đầu | Tuần 9 (bản giao đầu tiên) | Ngày 2–3 (commit đầu kèm unit test) |
| Ai test | Đội test độc lập của Tới Nơi + nghiệm thu của vận hành | Dev (unit, component), tester trong đội (AC, exploratory), PO (demo) |
| Tài liệu test | Test plan có ký duyệt; test case chi tiết; báo cáo nghiệm thu | AC Given/When/Then trong story; test tự động; ghi chú phiên exploratory; vài dòng trong sprint plan |
| Regression | Chạy tay một đợt trước nghiệm thu | Tự động mỗi lần merge |
| Phản hồi một lỗi biên | Ngày–tuần (qua báo lỗi tới đối tác) | Phút (CI đỏ) |
| Luật đổi 2 phút ⇒ 3 phút ở tuần 7 | Change request, đánh giá lại phạm vi và chi phí, sửa test case | Story mới vào sprint sau; sửa AC và test, CI xác nhận |
| Điểm mạnh | Phạm vi rõ, có bằng chứng nghiệm thu, hợp với hợp đồng trọn gói | Lỗi tìm sớm, thích nghi thay đổi |
| Điểm yếu | Lỗi phát hiện muộn, sửa đắt | Dễ coi nhẹ tài liệu; tốn công giữ bộ test tự động |

### 5.2. Dòng thời gian

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 236" role="img" aria-label="So sánh dòng thời gian 13 tuần. Hàng trên Waterfall: yêu cầu tuần 1 đến 3, thiết kế tuần 4 đến 6, code tuần 7 đến 9, test hệ thống tuần 9 đến 12, nghiệm thu và release tuần 13; tester review và thiết kế test từ tuần 1 đến 6 nhưng chạy test động từ tuần 9. Hàng dưới Scrum: sáu sprint hai tuần, mỗi sprint có phân tích, code, test, demo; test động bắt đầu ngay sprint 1; thanh regression tự động dài dần qua mỗi sprint.">
<text x="140" y="20" text-anchor="middle" class="t-small">tuần 1</text><text x="460" y="20" text-anchor="middle" class="t-small">tuần 7</text><text x="860" y="20" text-anchor="middle" class="t-small">tuần 13</text>
<text x="20" y="54" class="t-b">Waterfall</text>
<rect x="110" y="34" width="180" height="32" rx="6" class="f-surface s-line"/><text x="200" y="55" text-anchor="middle">Yêu cầu (T1–3)</text>
<rect x="290" y="34" width="180" height="32" rx="6" class="f-surface s-line"/><text x="380" y="55" text-anchor="middle">Thiết kế (T4–6)</text>
<rect x="470" y="34" width="150" height="32" rx="6" class="f-surface s-line"/><text x="545" y="55" text-anchor="middle">Code (T7–9)</text>
<rect x="620" y="34" width="210" height="32" rx="6" class="f-pk-soft s-pk"/><text x="725" y="55" text-anchor="middle">Test hệ thống (T9–12)</text>
<rect x="830" y="34" width="60" height="32" rx="6" class="f-phys-soft s-phys"/><text x="860" y="55" text-anchor="middle" class="t-small">release</text>
<rect x="110" y="72" width="360" height="22" rx="5" class="f-pk-soft s-pk dash"/><text x="290" y="87" text-anchor="middle" class="t-small">tester: review đặc tả, test analysis, test design (tĩnh)</text>
<line x1="620" y1="28" x2="620" y2="100" class="s-warn" stroke-width="1.6"/><text x="620" y="114" text-anchor="middle" class="t-small t-warn">test động đầu tiên</text>
<text x="20" y="158" class="t-b">Scrum + CI</text>
<rect x="110" y="138" width="130" height="32" rx="6" class="f-pk-soft s-pk"/><text x="175" y="159" text-anchor="middle" class="t-small">Sprint 1</text>
<rect x="240" y="138" width="130" height="32" rx="6" class="f-pk-soft s-pk"/><text x="305" y="159" text-anchor="middle" class="t-small">Sprint 2</text>
<rect x="370" y="138" width="130" height="32" rx="6" class="f-pk-soft s-pk"/><text x="435" y="159" text-anchor="middle" class="t-small">Sprint 3</text>
<rect x="500" y="138" width="130" height="32" rx="6" class="f-pk-soft s-pk"/><text x="565" y="159" text-anchor="middle" class="t-small">Sprint 4</text>
<rect x="630" y="138" width="130" height="32" rx="6" class="f-pk-soft s-pk"/><text x="695" y="159" text-anchor="middle" class="t-small">Sprint 5</text>
<rect x="760" y="138" width="130" height="32" rx="6" class="f-pk-soft s-pk"/><text x="825" y="159" text-anchor="middle" class="t-small">Sprint 6</text>
<text x="500" y="128" text-anchor="middle" class="t-small">mỗi sprint: làm rõ AC → code + test → demo → retrospective</text>
<line x1="125" y1="132" x2="125" y2="200" class="s-warn" stroke-width="1.6"/><text x="132" y="212" class="t-small t-warn">test động từ ngày 2–3</text>
<rect x="240" y="180" width="20" height="10" class="f-phys"/><rect x="370" y="180" width="40" height="10" class="f-phys"/><rect x="500" y="180" width="60" height="10" class="f-phys"/><rect x="630" y="180" width="80" height="10" class="f-phys"/><rect x="760" y="180" width="100" height="10" class="f-phys"/>
<text x="560" y="222" text-anchor="middle" class="t-small t-phys">bộ regression tự động lớn dần qua mỗi sprint</text>
</svg>
</div>
<figcaption><b>Hình 2 · Cùng 13 tuần, hai thời điểm bắt đầu test động.</b> Waterfall vẫn «test sớm» được nhờ review và thiết kế test từ tuần 1 — nhưng là test tĩnh. Ở Scrum, test động bắt đầu từ sprint đầu và để lại bộ regression ngày càng lớn (số tuần là giả định).</figcaption>
</figure>

### 5.3. Test-first: từ tiêu chí chấp nhận tới test chạy được

Bối cảnh B, buổi làm rõ story «Khách huỷ chuyến». PO, dev và tester viết tiêu chí chấp nhận dạng BDD (trích):

```gherkin
Tính năng: Phí huỷ chuyến của khách
  Kịch bản: Huỷ đúng 2 phút sau khi tài xế xe máy nhận
    Cho trước tài xế xe máy đã nhận chuyến lúc 09:00
    Khi khách huỷ lúc 09:02:00
    Thì phí huỷ là 10.000 đ với lý do "charged"

  Kịch bản: Tài xế trễ quá ETA cộng 10 phút
    Cho trước tài xế nhận lúc 09:00 với ETA 5 phút và chuyến đang "arriving"
    Khi khách huỷ lúc 09:15:01
    Thì phí huỷ là 0 đ với lý do "driver_late"
```

Dev chuyển bảy kịch bản AC-1…AC-7 thành test vitest, giữ nguyên chữ Given/When/Then trong tên test để stakeholder đọc được log. Trích (toàn bộ ở `examples/toi-noi-testing/articles/kiem-thu-trong-sdlc-agile-waterfall/phi-huy-acceptance.test.ts`):

```ts
it('AC-6 · R-CANCEL-04 · Given ETA 5 phút, When khách huỷ ở phút 15 giây 1, Then miễn phí (driver_late)', () => {
  expect(customerCancellationFee(chuyen({ status: 'arriving', now: sau(15, 1) }))).toEqual({ fee: 0, reason: 'driver_late' });
});
```

Ở TDD thật, test này chạy **đỏ** trước khi hàm tồn tại. Trong lab, hàm đã có sẵn nên ta chỉ thấy bước sau cùng: test xanh, và từ đây chúng ở lại làm regression ở bước 2 của pipeline. Chạy trong `examples/toi-noi-testing` (Node 22, vitest 3.2.7):

```bash
npx vitest run articles/kiem-thu-trong-sdlc-agile-waterfall --reporter=verbose
```

```text
✓ Tính năng: Phí huỷ chuyến của khách (R-CANCEL) > AC-1 · R-CANCEL-01 · Given chuyến đang tìm tài xế, When khách huỷ, Then phí 0 đ, lý do searching
✓ … > AC-2 · R-CANCEL-02 · Given tài xế nhận được 1 phút 59 giây, When khách huỷ, Then miễn phí (grace_period)
✓ … > AC-3 · R-CANCEL-02/03 · Given tài xế xe máy nhận được đúng 2 phút, When khách huỷ, Then phí 10.000 đ
✓ … > AC-4 · R-CANCEL-03 · Given chuyến ô tô đang arriving, When khách huỷ, Then phí 20.000 đ
✓ … > AC-5 · R-CANCEL-04 · Given ETA 5 phút, When khách huỷ đúng phút 15, Then vẫn thu phí
✓ … > AC-6 · R-CANCEL-04 · Given ETA 5 phút, When khách huỷ ở phút 15 giây 1, Then miễn phí (driver_late)
✓ … > AC-7 · R-CANCEL-01 · Given chuyến đang in_transit, When khách huỷ, Then bị từ chối INVALID_TRANSITION

 Test Files  1 passed (1)
      Tests  7 passed (7)
```

(Tên tệp và thời gian được lược bớt.) Kết quả thực tế khớp mong đợi ở cả bảy tiêu chí. Ghi chú: kịch bản Gherkin ở trên là tài liệu làm rõ yêu cầu; lab không cài Cucumber, nên «chuyển thành test chạy được» ở đây làm bằng tay sang vitest.

### 5.4. Retrospective cuối sprint (bối cảnh B, giả định)

| Câu hỏi | Ý kiến của đội |
|---|---|
| Điều gì tốt, nên giữ? | Viết AC Given/When/Then trước sprint ⇒ không ai tranh cãi mốc «đúng 2 phút» khi demo |
| Điều gì chưa tốt? | Test thu phí khi VíXanh `lost_response` chỉ được nghĩ tới ở ngày 9; phiên exploratory bị dồn cuối sprint |
| Làm gì tiếp? | (1) Thêm câu «lỗi cổng thanh toán?» vào checklist làm rõ story có tiền. (2) Đặt phiên exploratory ở giữa sprint. Người theo dõi: tester; kiểm lại ở retrospective sau |

Kết quả ghi vào phần bài học của test completion report của sprint ([TEST-007](/articles/quy-trinh-kiem-thu-test-process), mục 5.4).

## 6. Góc nhìn nâng cao

- **«Agile không cần tài liệu» là hiểu sai.** Syllabus nói tài liệu **nhẹ**, không nói không có. Ở Tới Nơi, phần chạm tới tiền (phí huỷ, VíXanh) vẫn cần bằng chứng truy vết được — ngữ cảnh domain (FL-1.4.2) có thể đòi nhiều tài liệu hơn ngay trong đội Scrum.
- **Waterfall vẫn shift left được.** V-model gắn mỗi pha phát triển với một test level và đẩy test analysis/design sớm. Cái Waterfall không làm được là **test động** sớm, vì chưa có code chạy.
- **DevOps không xoá test thủ công.** Pipeline giỏi lặp lại cái đã biết; pipeline không tự hỏi «thông báo phí huỷ có làm khách hiểu nhầm không». Ứng dụng di động còn có một ràng buộc khác: bản app qua cửa hàng ứng dụng không «deploy liên tục» như backend.
- **Rủi ro của automation trong DevOps:** test chập chờn (flaky) làm đội quen bỏ qua CI đỏ; pipeline chậm làm phản hồi hết «nhanh»; bộ test lớn cần người bảo trì — đúng như syllabus liệt kê ở mặt thách thức.
- **Microservices làm tăng giá trị của contract test.** Booking và Payment deploy độc lập; nếu không có bước 4 của pipeline, một thay đổi schema `BookingCancelled` chỉ lộ ra ở staging. Ngược lại, tách service quá sớm (anti-pattern 11 của tài liệu kiến trúc) nhân số pipeline phải duy trì.
- **Shift left không phải «chỉ test sớm».** Test non-functional từ component level là shift left, nhưng tải thật 70 chuyến/giây vẫn phải đo trên môi trường gần production. Các thực hành quan sát trên production («shift right») là kiến thức ngành, ngoài syllabus.
- **Retrospective vô ích nếu không theo dõi.** Syllabus nhấn mạnh việc theo dõi các cải tiến; một danh sách hành động không có người chịu trách nhiệm sẽ lặp lại ở retrospective sau.

## 7. Bài tập và câu hỏi ôn tập

### Bài tập

**Bài 1.** Đội Payment của Tới Nơi làm theo Kanban (luồng liên tục, không sprint). Cần sửa R-PAY-04: số lần thử lại từ 2 thành 3. Liệt kê test cần chạy/sửa và đặt chúng vào các bước pipeline ở Hình 1.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Bước 2 (unit): sửa test đếm số lần gọi cổng (tối đa 4 lần thay vì 3) và test «declined không thử lại». Bước 3 (component): use case thu tiền với ví giả <code>timeout</code> 3 lần rồi thành công ⇒ <code>paid</code>. Bước 5 (integration): <code>lost_response</code> ⇒ mọi lần thử dùng cùng khoá idempotency, ví chỉ trừ một lần (R-PAY-04, R-NFR-05). Regression: toàn bộ test R-CANCEL-07 và R-PAY-05 vì phí huỷ dùng cùng cơ chế. Ngoài pipeline: hỏi PO việc thời gian chờ tối đa khi cổng không trả lời tăng từ 9 giây (3 × 3) lên 12 giây (4 × 3) có chấp nhận được không — đây là câu hỏi review sớm, trước khi sửa code. Kanban không đổi nguyên tắc: test đi theo từng thay đổi, phản hồi nhanh.</p>
</details>

**Bài 2.** Viết hai kịch bản Given/When/Then cho R-CANCEL-05 (tài xế huỷ, hệ thống huỷ).

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(1) <i>Cho trước</i> tài xế D101 đã nhận chuyến được 5 phút; <i>khi</i> tài xế huỷ; <i>thì</i> chuyến về <code>searching</code>, khách không bị tính phí, D101 về <code>online</code> (R-STATE-04). (2) <i>Cho trước</i> chuyến ở <code>searching</code> từ 09:00; <i>khi</i> hệ thống quét lúc 09:05:00; <i>thì</i> chuyến <code>cancelled</code> với lý do <code>no_driver</code>, phí 0 đ (R-STATE-05: ≥ 5 phút). Thêm một kịch bản biên: quét lúc 09:04:59 ⇒ chuyến vẫn <code>searching</code>.</p>
</details>

**Bài 3.** Sau sprint, đội ghi nhận: «3 lỗi tìm được ở demo đều do AC hiểu khác nhau; CI đỏ 4 lần vì test chập chờn và mọi người bấm chạy lại». Viết ba hành động cải tiến theo tinh thần FL-2.1.6.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(1) Mỗi story có tiền hoặc thời gian phải có ví dụ biên cụ thể trong AC (đúng 2 phút, quá 1 giây) — tester chịu trách nhiệm. (2) Test chập chờn bị cách ly khỏi bước chặn merge trong 48 giờ và phải được sửa (dùng đồng hồ tiêm vào thay vì giờ thật) — dev phụ trách. (3) Ghi hai hành động vào test completion report và kiểm lại ở retrospective sau; đo số lần «chạy lại CI» để biết đã cải thiện.</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Trong một dự án theo mô hình tuần tự, điều nào thường đúng với tester ở các pha đầu?

- a) Tester chạy dynamic testing ở mọi test level ngay từ pha yêu cầu
- b) Tester tham gia review yêu cầu, test analysis và test design; dynamic testing thường chưa làm được vì chưa có code chạy
- c) Tester không có việc gì cho tới khi nhận bản build đầu tiên
- d) Tester chủ yếu viết test tự động cho regression

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b>, đúng như FL-2.1.1 mô tả mô hình tuần tự. a) sai: code chạy được xuất hiện ở pha sau. c) sai: review và thiết kế test bắt đầu sớm. d) sai: tự động hoá regression rộng là đặc trưng của mô hình lặp/Agile, và cần có code.</p>
</details>

**Câu 2 (K1).** Đâu là một thực hành kiểm thử tốt áp dụng cho MỌI SDLC theo CTFL v4.0.1?

- a) Mỗi hoạt động phát triển có một hoạt động test tương ứng
- b) Tài liệu test luôn phải được ký duyệt trước khi chạy test
- c) Mọi test phải tự động hoá
- d) Chỉ tester độc lập mới được thiết kế test

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a) đúng.</b> Đây là thực hành đầu tiên trong FL-2.1.2. b), c), d) phụ thuộc ngữ cảnh, không phải thực hành chung cho mọi SDLC.</p>
</details>

**Câu 3 (K1).** Cách tiếp cận nào mô tả hành vi mong muốn bằng test case viết ở ngôn ngữ tự nhiên dễ hiểu với stakeholder, thường theo dạng Given/When/Then?

- a) TDD
- b) ATDD
- c) BDD
- d) DDD

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng.</b> a) TDD dẫn dắt code bằng test case ở mức code. b) ATDD suy test từ tiêu chí chấp nhận — gần nhất, nhưng định dạng Given/When/Then là đặc điểm syllabus gán cho BDD. d) DDD (domain-driven design) là phương pháp thiết kế, không phải cách tiếp cận test-first.</p>
</details>

**Câu 4 (K2).** Điều nào là một THÁCH THỨC của DevOps đối với testing (không phải lợi ích)?

- a) Phản hồi nhanh về chất lượng code
- b) Rủi ro regression giảm nhờ regression tự động quy mô lớn
- c) Test automation cần thêm nguồn lực và có thể khó dựng, khó bảo trì
- d) Tăng khả năng nhìn thấy các đặc tính phi chức năng

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng</b> — nằm trong danh sách rủi ro/thách thức của FL-2.1.4 (cùng với phải dựng pipeline và duy trì công cụ CI/CD). a), b), d) là lợi ích.</p>
</details>

**Câu 5 (K2).** Phát biểu nào về shift left là đúng?

- a) Shift left nghĩa là bỏ system testing để dồn công sức vào unit test
- b) Shift left đưa test về sớm hơn trong SDLC nhưng không có nghĩa bỏ qua test ở giai đoạn sau
- c) Shift left luôn giảm chi phí ngay từ đầu dự án
- d) Shift left chỉ áp dụng được trong DevOps

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b> theo FL-2.1.5. a) sai: syllabus nói rõ không được bỏ test muộn. c) sai: có thể tốn thêm chi phí lúc đầu, kỳ vọng tiết kiệm về sau. d) sai: review đặc tả từ góc nhìn tester là shift left trong mọi mô hình; CI/CD chỉ là một trong các thực hành.</p>
</details>

**Câu 6 (K2).** Đội Tới Nơi họp retrospective cuối sprint. Cách làm nào phù hợp nhất với CTFL v4.0.1?

- a) Chỉ tester tham gia, vì đây là cuộc họp cải tiến quy trình test
- b) Ghi lại kết quả (thường đưa vào test completion report) và theo dõi việc thực hiện các cải tiến
- c) Chỉ họp khi dự án thất bại
- d) Chỉ bàn điều chưa tốt, bỏ qua điều đã tốt để tiết kiệm thời gian

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng</b> theo FL-2.1.6. a) sai: người tham gia gồm cả developer, kiến trúc sư, PO, BA. c) sai: retrospective thường diễn ra cuối iteration, cuối dự án, ở mốc release hoặc khi cần. d) sai: một trong ba câu hỏi là điều gì thành công và nên giữ.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — mục 2.1 (2.1.1 Impact of the SDLC on Testing, 2.1.2 SDLC and Good Testing Practices, 2.1.3 Testing as a Driver for Software Development, 2.1.4 DevOps and Testing, 2.1.5 Shift Left, 2.1.6 Retrospectives and Process Improvement). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — tra thuật ngữ SDLC, shift left, TDD/ATDD/BDD tại trang chủ https://www.istqb.org/
- Các sách syllabus dẫn trong mục 2.1: Beck, K. (2003) *Test Driven Development: By Example*; Gärtner, M. (2011) *ATDD by Example*; Chelimsky, D. et al. (2010) *The RSpec Book*; Kim, G. et al. (2016) *The DevOps Handbook*.
- Tài liệu kiến trúc Tới Nơi (nội bộ blog): §17 CI/CD và chất lượng phần mềm, §17.1 chiến lược test theo kiến trúc, §21 anti-pattern.
- Phòng lab: `SPEC.md` (R-CANCEL, R-PAY) và bộ test của bài — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-003 Bảy nguyên lý kiểm thử](/articles/bay-nguyen-ly-kiem-thu) (nguyên lý test sớm) · [TEST-007 Quy trình kiểm thử](/articles/quy-trinh-kiem-thu-test-process) · [TEST-009 Vai trò trong kiểm thử](/articles/vai-tro-trong-kiem-thu)

Bài tiếp theo: [TEST-009 · Vai trò trong kiểm thử: Developer, Tester, QA, Test Lead](/articles/vai-tro-trong-kiem-thu).
