---
slug: vai-tro-trong-kiem-thu
title: "Vai trò trong kiểm thử: Developer, Tester, QA, Test Lead (Test Roles and Responsibilities)"
summary: "Hai vai trò kiểm thử của ISTQB CTFL v4 — test management và testing — đối chiếu với chức danh thật ở đội Tới Nơi; kỹ năng của tester, whole team approach và chọn mức độc lập kiểm thử theo rủi ro."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-009, test-roles, whole-team, test-independence, qa-lead]
language: vi
---

Trong đội Tới Nơi có người mang chức danh «QA Engineer», có «SDET», có «QA Lead», còn developer cũng viết test. Vậy ai chịu trách nhiệm kiểm thử? Bài này trả lời theo cách của ISTQB: kiểm thử không chia theo chức danh mà theo **vai trò** (role), một người có thể đóng nhiều vai, và mức **độc lập** (independence) của người kiểm thử là một lựa chọn có chủ đích chứ không phải sơ đồ tổ chức ngẫu nhiên.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-009</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S01 · Fundamentals of Software Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/quy-trinh-kiem-thu-test-process">TEST-007 · Quy trình kiểm thử</a>, <a href="/articles/kiem-thu-trong-sdlc-agile-waterfall">TEST-008 · Kiểm thử trong SDLC</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Ma trận RACI cho việc kiểm thử voucher (SPEC R-VOUCHER) và chọn mức độc lập kiểm thử — giấy/bảng tính</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.4.5, 1.5 (1.5.1–1.5.3) · FL-1.4.5 (K2), FL-1.5.1 (K2), FL-1.5.2 (K1), FL-1.5.3 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Một lỗi voucher lọt lên production: khách huỷ chuyến nhưng không được trả lại lượt dùng mã `GIAM20`. Họp sau sự cố, developer nói «QA chưa test test case huỷ chuyến», QA nói «test case này không có trong ticket», product owner nói «tưởng ai cũng biết». Cái thiếu không phải chuyên môn mà là **ai chịu trách nhiệm phần nào của việc kiểm thử**.

Chủ đề vai trò trả lời ba câu: **ai làm việc gì** trong 7 nhóm hoạt động kiểm thử ([TEST-007](/articles/quy-trinh-kiem-thu-test-process)); **người đó cần kỹ năng gì**; và **người kiểm thử nên «xa» tác giả đến mức nào**. Bạn dùng chủ đề này khi lập đội cho tính năng mới, khi viết mục stakeholder và độc lập kiểm thử trong test plan, hoặc khi đội tranh cãi «QA có phải chịu trách nhiệm chất lượng không».

## 2. Lý thuyết theo ISTQB

### 2.1. Hai vai trò chính (mục 1.4.5)

CTFL v4.0.1 chỉ nói tới **hai vai trò**, không nói tới chức danh:

| Vai trò | Chịu trách nhiệm chung về | Tập trung vào các hoạt động |
|---|---|---|
| **Test management role** (vai trò quản lý kiểm thử) | quy trình kiểm thử, đội kiểm thử, dẫn dắt các hoạt động test | test planning, test monitoring & test control, test completion |
| **Testing role** (vai trò kiểm thử) | khía cạnh kỹ thuật (engineering) của kiểm thử | test analysis, test design, test implementation, test execution |

Ba điểm syllabus nhấn mạnh:

- Việc giao nhiệm vụ cho mỗi vai trò **phụ thuộc ngữ cảnh**: dự án, sản phẩm, kỹ năng của người đảm nhận và tổ chức.
- Vai trò quản lý có thể do **nhiều kiểu người** đảm nhận: trưởng nhóm, test manager, development manager… Trong Agile, một phần việc quản lý test có thể do **chính đội Agile** làm; việc xuyên suốt nhiều đội thì do test manager bên ngoài đội.
- **Một người có thể giữ cả hai vai trò** cùng lúc, và người giữ vai trò có thể thay đổi theo thời gian.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Hai vai trò của CTFL: test management role phụ trách planning, monitoring và control, completion; testing role phụ trách analysis, design, implementation, execution. Bên dưới là các chức danh ở Tới Nơi có thể đảm nhận từng vai trò.">
<rect x="20" y="20" width="420" height="190" rx="10" class="f-pk-soft s-pk"/>
<text x="230" y="46" text-anchor="middle" class="t-b t-pk">Test management role</text>
<text x="230" y="64" text-anchor="middle" class="t-small">trách nhiệm: quy trình, đội, dẫn dắt</text>
<rect x="45" y="80" width="370" height="32" rx="6" class="f-surface s-pk"/><text x="230" y="101" text-anchor="middle" class="t-mono">Test planning</text>
<rect x="45" y="120" width="370" height="32" rx="6" class="f-surface s-pk"/><text x="230" y="141" text-anchor="middle" class="t-mono">Test monitoring &amp; test control</text>
<rect x="45" y="160" width="370" height="32" rx="6" class="f-surface s-pk"/><text x="230" y="181" text-anchor="middle" class="t-mono">Test completion</text>
<rect x="480" y="20" width="420" height="190" rx="10" class="f-phys-soft s-phys"/>
<text x="690" y="46" text-anchor="middle" class="t-b t-phys">Testing role</text>
<text x="690" y="64" text-anchor="middle" class="t-small">trách nhiệm: khía cạnh kỹ thuật</text>
<rect x="505" y="80" width="175" height="50" rx="6" class="f-surface s-phys"/><text x="592" y="110" text-anchor="middle" class="t-mono">Test analysis</text>
<rect x="700" y="80" width="175" height="50" rx="6" class="f-surface s-phys"/><text x="787" y="110" text-anchor="middle" class="t-mono">Test design</text>
<rect x="505" y="142" width="175" height="50" rx="6" class="f-surface s-phys"/><text x="592" y="165" text-anchor="middle" class="t-mono">Test</text><text x="592" y="181" text-anchor="middle" class="t-mono">implementation</text>
<rect x="700" y="142" width="175" height="50" rx="6" class="f-surface s-phys"/><text x="787" y="172" text-anchor="middle" class="t-mono">Test execution</text>
<rect x="20" y="230" width="420" height="54" rx="8" class="f-bg s-line dash"/>
<text x="230" y="252" text-anchor="middle" class="t-small">ở Tới Nơi có thể là: QA Lead, Engineering Manager,</text>
<text x="230" y="270" text-anchor="middle" class="t-small">hoặc cả squad Agile (phần việc trong một đội)</text>
<rect x="480" y="230" width="420" height="54" rx="8" class="f-bg s-line dash"/>
<text x="690" y="252" text-anchor="middle" class="t-small">ở Tới Nơi có thể là: QA Engineer, SDET, Developer,</text>
<text x="690" y="270" text-anchor="middle" class="t-small">đại diện nghiệp vụ khi chạy acceptance test</text>
</svg>
</div>
<figcaption><b>Hình 1 · Vai trò là tập trách nhiệm, không phải chức danh.</b> Khung nét đứt là ví dụ người có thể đảm nhận — ngoài syllabus, tuỳ tổ chức. Một QA Lead ở squad nhỏ thường giữ cả hai khung.</figcaption>
</figure>

### 2.2. Kỹ năng chung của người kiểm thử (mục 1.5.1)

Syllabus coi kỹ năng là khả năng làm tốt một việc, đến từ kiến thức, luyện tập và năng khiếu. Sáu nhóm kỹ năng chung, kèm **lý do** syllabus gắn cho từng nhóm:

| Kỹ năng | Để làm gì | Ví dụ ở Tới Nơi |
|---|---|---|
| Kiến thức kiểm thử | tăng hiệu quả, ví dụ dùng test technique | phân tích giá trị biên cho mốc 2 phút (R-CANCEL-02) |
| Cẩn thận, tỉ mỉ, tò mò, có phương pháp | tìm defect, nhất là defect khó thấy | để ý hạn voucher tính theo **giờ Việt Nam** (R-VOUCHER-04) |
| Giao tiếp, lắng nghe, làm việc nhóm | trao đổi với stakeholder, báo cáo và thảo luận defect | báo lỗi để developer đọc là hiểu, không thấy bị chỉ trích |
| Tư duy phân tích, phản biện, sáng tạo | tăng hiệu quả kiểm thử | «hệ thống huỷ (`no_driver`) có trả lượt voucher không?» — có (R-VOUCHER-07) |
| Kiến thức kỹ thuật | tăng năng suất, ví dụ dùng đúng công cụ | dời giờ bằng `POST /api/test/clock` thay vì chờ tới 23:59 |
| Kiến thức nghiệp vụ | hiểu và trao đổi với đại diện nghiệp vụ | đơn đồ ăn do xe máy giao nên phí huỷ là 10.000 đ (R-CANCEL-03) |

Syllabus nhắc riêng một điều: tester thường là **người mang tin xấu**. Kết quả test dễ bị nghe như lời chê sản phẩm và chê tác giả; **confirmation bias** (thiên kiến xác nhận) khiến người ta khó chấp nhận thông tin trái với điều mình tin. Vì vậy thông tin về defect và failure cần được truyền đạt **theo cách mang tính xây dựng**.

### 2.3. Whole team approach (mục 1.5.2)

Cách làm có gốc từ Extreme Programming: **bất kỳ thành viên nào có đủ kiến thức và kỹ năng đều làm được bất kỳ việc nào, và mọi người cùng chịu trách nhiệm về chất lượng**. Đội ngồi chung một không gian (thật hoặc ảo) để trao đổi dễ hơn. Lợi ích syllabus liệt kê: cải thiện động lực của đội, tăng giao tiếp và cộng tác, tạo hiệu ứng cộng hưởng khi tận dụng nhiều bộ kỹ năng khác nhau.

Tester trong cách làm này **không biến mất**: họ cùng đại diện nghiệp vụ viết acceptance test, cùng developer thống nhất test strategy và cách tự động hoá, và truyền kiến thức kiểm thử cho cả đội. Syllabus cũng nói rõ giới hạn: có ngữ cảnh (ví dụ hệ thống an toàn sống còn — safety-critical) cần mức độc lập cao, khi đó whole team approach có thể không phù hợp.

### 2.4. Độc lập kiểm thử (mục 1.5.3)

Một mức độc lập nhất định giúp tester tìm defect tốt hơn, vì **thiên kiến nhận thức** (cognitive bias) của tác giả và của tester khác nhau. Nhưng độc lập không thay được **sự am hiểu**: developer tìm được nhiều defect trong chính code của họ một cách hiệu quả. Bốn mức:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 270" role="img" aria-label="Bốn mức độc lập kiểm thử dạng bậc thang: không độc lập (tác giả), một phần (đồng nghiệp cùng đội), cao (tester ngoài đội trong tổ chức), rất cao (ngoài tổ chức), kèm ví dụ Tới Nơi.">
<defs>
<marker id="t009-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="190" width="210" height="64" rx="8" class="f-surface s-line"/>
<text x="125" y="212" text-anchor="middle" class="t-b">Không độc lập</text>
<text x="125" y="230" text-anchor="middle" class="t-small">tác giả tự test</text>
<text x="125" y="246" text-anchor="middle" class="t-10">dev viết unit test voucher</text>
<rect x="245" y="135" width="210" height="119" rx="8" class="f-phys-soft s-phys"/>
<text x="350" y="157" text-anchor="middle" class="t-b t-phys">Một phần</text>
<text x="350" y="175" text-anchor="middle" class="t-small">đồng nghiệp cùng đội</text>
<text x="350" y="191" text-anchor="middle" class="t-10">QA Engineer trong squad,</text>
<text x="350" y="207" text-anchor="middle" class="t-10">dev khác review/pair test</text>
<rect x="470" y="80" width="210" height="174" rx="8" class="f-pk-soft s-pk"/>
<text x="575" y="102" text-anchor="middle" class="t-b t-pk">Cao</text>
<text x="575" y="120" text-anchor="middle" class="t-small">tester ngoài đội, trong công ty</text>
<text x="575" y="136" text-anchor="middle" class="t-10">nhóm QA nền tảng chạy</text>
<text x="575" y="152" text-anchor="middle" class="t-10">system test đặt chuyến</text>
<rect x="695" y="25" width="205" height="229" rx="8" class="f-warn-soft s-warn"/>
<text x="797" y="47" text-anchor="middle" class="t-b t-warn">Rất cao</text>
<text x="797" y="65" text-anchor="middle" class="t-small">tester ngoài tổ chức</text>
<text x="797" y="81" text-anchor="middle" class="t-10">công ty kiểm thử bảo mật</text>
<text x="797" y="97" text-anchor="middle" class="t-10">cho Payment &amp; Wallet</text>
<line x1="40" y1="20" x2="660" y2="20" class="s-muted" stroke-width="1.4" marker-end="url(#t009-arrow)"/>
<text x="40" y="40" class="t-small">càng lên: góc nhìn càng khác tác giả, nhưng càng xa đội</text>
</svg>
</div>
<figcaption><b>Hình 2 · Bốn mức độc lập theo CTFL 1.5.3, ví dụ ở Tới Nơi.</b> QA Engineer ngồi trong squad chỉ đạt mức «một phần» — họ là đồng nghiệp cùng đội của tác giả, dù chức danh khác.</figcaption>
</figure>

Khuyến nghị của syllabus: với đa số dự án, nên kết hợp **nhiều mức độc lập** — developer làm component và component integration test, đội test làm system và system integration test, đại diện nghiệp vụ làm acceptance test.

| | Nội dung (theo 1.5.3) |
|---|---|
| **Lợi ích** | tester độc lập dễ nhận ra những kiểu failure/defect khác với developer (khác nền tảng, góc nhìn kỹ thuật, thiên kiến); có thể kiểm chứng, chất vấn hoặc bác bỏ các giả định mà stakeholder đặt ra khi đặc tả và cài đặt |
| **Hạn chế** | tester bị tách khỏi đội phát triển ⇒ thiếu cộng tác, khó giao tiếp, thậm chí đối đầu; developer mất ý thức trách nhiệm về chất lượng; tester bị coi là nút thắt cổ chai hoặc bị đổ lỗi khi chậm phát hành |

### 2.5. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa trong bài |
|---|---|
| Test management role | vai trò quản lý kiểm thử: lập kế hoạch, theo dõi, điều khiển, kết thúc kiểm thử |
| Testing role | vai trò kiểm thử: phân tích, thiết kế, cài đặt, thực thi kiểm thử |
| Test manager | người giữ vai trò quản lý kiểm thử (chức danh có thể khác) |
| Tester | người thực hiện kiểm thử (chức danh có thể khác) |
| Whole team approach | cách làm cả đội: ai đủ năng lực đều làm được mọi việc, cả đội cùng chịu trách nhiệm chất lượng |
| Independence of testing | độc lập kiểm thử: mức tách biệt giữa người kiểm thử và tác giả của sản phẩm |
| Cognitive bias / confirmation bias | thiên kiến nhận thức / thiên kiến xác nhận |
| Stakeholder | bên liên quan: người có quyền lợi với sản phẩm hoặc kết quả kiểm thử |
| Business representative | đại diện nghiệp vụ (ở Tới Nơi: Vận hành, Chăm sóc khách hàng) |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB · chức danh trong ngành</span>
<p>CTFL không định nghĩa các chức danh dưới đây. Cách dùng phổ biến (mỗi công ty một khác):</p>
<p><b>Developer</b> — giữ testing role ở component/component integration test. <b>Tester / QA Tester</b> — testing role, thiên về phân tích, thiết kế, thực thi. <b>QA Engineer</b> — testing role, thường kèm tự động hoá và tham gia quy trình chất lượng; chữ «QA» ở đây là tên gọi, không có nghĩa họ «đảm bảo» chất lượng một mình (QA vs testing: xem <a href="/articles/muc-tieu-kiem-thu-va-quality-assurance">TEST-006</a>). <b>SDET</b> (Software Development Engineer in Test) — testing role nặng phần implementation: framework, test script, môi trường. <b>Test Lead / QA Lead</b> — test management role cho một đội hoặc một khối, thường vẫn làm cả testing role. <b>Test Manager / Head of QA</b> — test management role xuyên nhiều đội.</p>
<p>RACI dùng ở mục 5 cũng là công cụ quản lý dự án chung, không thuộc syllabus.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu và business rules

Squad **Pricing & Promotion** phát hành tính năng mã giảm giá cho cả ba dịch vụ. Test basis là nhóm R-VOUCHER của [SPEC lab](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing):

- **R-VOUCHER-02** — mã được bỏ khoảng trắng và đổi chữ hoa (` giam20 ` ≡ `GIAM20`).
- **R-VOUCHER-04** — thời hạn tính theo ngày giờ Việt Nam, tính cả hai đầu.
- **R-VOUCHER-06** — tiền giảm = min(percent% × A, maxDiscount); A < minAmount ⇒ `VOUCHER_MIN_AMOUNT`.
- **R-VOUCHER-07** — lượt dùng trong ngày; chuyến bị huỷ trả lại lượt; kiểm tra hai lần (khi báo giá và khi tạo chuyến).
- **R-VOUCHER-08** — nhiều lỗi thì chỉ báo lỗi đầu tiên theo thứ tự cố định.
- **R-FARE-07** — voucher không bao giờ kéo cước xuống dưới cước tối thiểu.

### 3.2. System context và con người

Kiến trúc đích: Customer App → API Gateway/Customer BFF → **Pricing & Quote** → **Promotion** (reserve/redeem/release, chống vượt quota bằng thao tác nguyên tử); **Booking** tạo chuyến từ quote; sự kiện huỷ đi qua Kafka để Promotion trả lượt. Lab là MVP modular monolith: không Kafka, lượt dùng đếm thẳng từ chuyến chưa huỷ (R-ARCH-02).

Đội (giả định): 1 product owner, 4 developer, 1 **QA Engineer** trong squad, 1 **SDET** của nhóm Platform QA dùng chung 4 squad, 1 **QA Lead** phụ trách khối Booking–Pricing, nhóm **Vận hành** làm đại diện nghiệp vụ.

### 3.3. Rủi ro

| Loại | Rủi ro | Liên quan tới vai trò thế nào |
|---|---|---|
| Product | hai yêu cầu đồng thời dùng cùng mã ⇒ vượt quota (kiến trúc §4.10) | cần SDET và môi trường nhiều kết nối — chưa có trong lab |
| Product | sự kiện huỷ tới trễ/trùng ⇒ lượt không được trả hoặc trả hai lần | luồng chạm hai squad, dễ thành «vùng không ai nhận» |
| Product | tính ngày theo UTC thay vì giờ Việt Nam (R-GEN-02) | developer tự test dễ bỏ sót vì cùng thiên kiến lúc viết |
| Project | không ai giữ test management role ⇒ không có tiêu chí kết thúc | lỗi tổ chức, không phải lỗi kỹ thuật |
| Project | dev mặc định «QA sẽ test», QA thành nút thắt trước phát hành | đúng hạn chế của độc lập kiểm thử (1.5.3) |

## 4. Phân tích kỹ thuật

### 4.1. Gán 7 nhóm hoạt động cho hai vai trò, rồi cho người

Bước 1 luôn là **vai trò** (theo 1.4.5); bước 2 mới là **người** (theo ngữ cảnh đội):

| Hoạt động (TEST-007) | Vai trò CTFL | Người đảm nhận ở squad Promotion |
|---|---|---|
| Test planning | test management | QA Lead, có PO và QA Engineer góp ý |
| Test monitoring & test control | test management | QA Lead theo dõi; squad tự điều chỉnh hằng ngày (Agile) |
| Test analysis | testing | QA Engineer + PO (test condition từ R-VOUCHER) |
| Test design | testing | QA Engineer; developer cho test ở mức component |
| Test implementation | testing | developer (unit), SDET (API, test đồng thời), QA Engineer (dữ liệu, procedure) |
| Test execution | testing | CI chạy tự động; QA Engineer chạy exploratory; Vận hành chạy acceptance |
| Test completion | test management | QA Lead viết test completion report, cả squad góp lessons learned |

### 4.2. Chọn mức độc lập theo đối tượng test và rủi ro

| Đối tượng test | Ai test | Mức độc lập | Lý do |
|---|---|---|---|
| hàm tính tiền giảm (R-VOUCHER-06, R-FARE-07) | developer tác giả | không | hiểu sâu code, nhanh, rẻ |
| API báo giá có voucher (R-VOUCHER-08) | QA Engineer trong squad | một phần | hiểu ngữ cảnh, không phải người viết |
| đặt chuyến có voucher qua app | nhóm QA nền tảng | cao | nhìn cả luồng Pricing → Booking → Payment |
| khuyến mãi có đúng ý kinh doanh | Vận hành / Marketing | cao | chỉ họ biết mã «đúng như đã hứa» |
| chống lạm dụng mã (giả định) | công ty bảo mật bên ngoài | rất cao | tiền thật, cần góc nhìn kẻ tấn công |

Nhận xét: **không có mức độc lập «đúng» cho cả tính năng**. Rủi ro càng liên quan tới tiền và người ngoài, càng nên thêm một tầng độc lập cao — nhưng giữ tầng developer tự test vì đó là nơi tìm defect rẻ nhất.

## 5. Thực hành

### 5.1. Ma trận RACI cho việc kiểm thử voucher

R = Responsible (người làm) · A = Accountable (người chịu trách nhiệm cuối, mỗi dòng đúng một A) · C = Consulted (được hỏi ý kiến) · I = Informed (được thông báo).

| Công việc kiểm thử | PO | Developer | QA Engineer | SDET | QA Lead |
|---|:-:|:-:|:-:|:-:|:-:|
| Viết test plan đợt phát hành voucher (phạm vi, tiêu chí vào/ra) | C | C | C | C | **A/R** |
| Phân tích R-VOUCHER-02…08 thành test condition | C | C | **A/R** | I | I |
| Unit test `voucherDiscount` (R-VOUCHER-06, R-FARE-07) | I | **A/R** | C | I | I |
| Test API thứ tự lỗi voucher (R-VOUCHER-08) | I | C | **A** | **R** | I |
| Test lượt dùng khi huỷ chuyến (R-VOUCHER-07) | C | C | **A/R** | C | I |
| Test đồng thời vượt quota (Promotion, ngoài lab) | I | C | C | **A/R** | I |
| Acceptance test với Vận hành | **A** | I | R | I | C |
| Báo cáo tiến độ, đề xuất go/no-go | C | I | C | I | **A/R** |
| Test completion report + lessons learned | C | R | R | R | **A** |

Cách kiểm tra ma trận: mỗi dòng có **đúng một A**; mỗi hoạt động của test management role có A là người giữ vai trò đó (QA Lead); không ai bị A ở quá nhiều dòng đến mức thành nút thắt. Riêng dòng acceptance test, A là PO vì đó là quyết định nghiệp vụ.

### 5.2. Mức độc lập đã chọn cho đợt voucher

> Kiểm thử voucher dùng **ba mức độc lập**: developer tự test component (không độc lập), QA Engineer trong squad test API và luồng huỷ (một phần), nhóm QA nền tảng chạy system test đặt chuyến có voucher (cao). Không thuê đơn vị ngoài ở đợt này vì voucher chưa nối thanh toán thật; sẽ cân nhắc mức «rất cao» khi mở mã giảm giá cho ví VíXanh.

Đoạn này nên nằm trong test plan, mục «Test approach — independence of testing» (CTFL 5.1.1).

### 5.3. Kỹ năng giao tiếp: cùng một defect, hai cách báo

| Cách báo kém | Cách báo mang tính xây dựng |
|---|---|
| «Voucher lại sai nữa rồi, code huỷ chuyến không ai test à?» | «Huỷ chuyến không trả lượt `GIAM20`: C001 báo giá → đặt chuyến → huỷ khi đang `searching` → báo giá lại cùng mã trong ngày ⇒ nhận `VOUCHER_USAGE_LIMIT`. Mong đợi theo R-VOUCHER-07: chuyến bị huỷ trả lại lượt. Đính kèm test log. Mình chưa chắc lỗi ở bước đếm hay bước huỷ — cần bạn xem giúp.» |

Bản bên phải nói **sự việc**, trích **yêu cầu**, cho **bước tái hiện**, và tách «tôi thấy gì» khỏi «tôi đoán gì». Cách viết defect report đầy đủ: xem [TEST-010](/articles/testware-san-pham-kiem-thu).

## 6. Góc nhìn nâng cao

| Sai lầm thường gặp | Thực tế theo CTFL |
|---|---|
| «QA Engineer chịu trách nhiệm chất lượng» | whole team: **mọi người** cùng chịu trách nhiệm; QA Engineer giữ testing role |
| «Có QA trong squad là đã độc lập» | đồng nghiệp cùng đội chỉ là độc lập **một phần** |
| «Developer không nên test code của mình» | developer tìm được nhiều defect trong code của họ rất hiệu quả; độc lập không thay được sự am hiểu |
| «Độc lập càng cao càng tốt» | độc lập cao có giá: cô lập, đối đầu, nút thắt, developer bớt trách nhiệm |
| «Test Lead chỉ quản lý, không test» | một người có thể giữ **cả hai** vai trò; ở squad nhỏ đó là chuyện bình thường |
| «Agile thì không cần test management» | phần việc quản lý vẫn còn, chỉ là có thể do chính đội đảm nhận |

Ba trường hợp dễ bỏ sót:

- **Vùng giữa hai squad.** «Huỷ chuyến → trả lượt voucher» chạm Booking và Promotion. RACI của mỗi squad chỉ phủ service của mình thì luồng xuyên service không có A — cần người sở hữu ở cấp khối.
- **Đổi người, không đổi vai trò.** QA Lead nghỉ phép thì test management role vẫn phải có người giữ; ghi người thay trong test plan.
- **Tài chính / safety-critical.** Quy định hoặc kiểm toán có thể đòi mức độc lập tối thiểu; whole team approach khi đó phải kèm một tầng độc lập cao.

Tổ chức đội QA, tuyển dụng, đánh giá năng lực thuộc quản lý kiểm thử nâng cao (CTAL-TM) — Series 12.

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1.** Một startup nhỏ làm phí huỷ chuyến (R-CANCEL-01…07) chỉ có 2 developer và 1 người vừa làm PO vừa test. Ai giữ test management role, ai giữ testing role? Mức độc lập nào còn thiếu và bù bằng cách nào?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Người PO kiêm tester giữ <b>cả hai</b> vai trò: lập kế hoạch, theo dõi, kết thúc (management) và phân tích, thiết kế, thực thi (testing) — CTFL cho phép một người giữ cả hai. Hai developer giữ testing role ở mức component (không độc lập), có thể test chéo code của nhau (một phần). Người PO kiêm tester ở cùng đội nên chỉ đạt mức «một phần»; thiếu mức «cao». Cách bù: nhờ người ngoài đội (ví dụ nhân viên Vận hành) chạy một phiên acceptance test theo kịch bản huỷ ở mốc 2 phút và tài xế trễ (R-CANCEL-02, R-CANCEL-04); hoặc thuê kiểm thử độc lập ngắn hạn trước khi bật thu phí qua ví.</p>
</details>

**Bài tập 2.** Xếp mỗi tình huống vào một mức độc lập: (a) developer Pricing chạy unit test của chính mình; (b) một developer khác trong squad Promotion review code voucher của đồng đội; (c) QA Engineer squad Dispatch test tính năng voucher của squad Promotion; (d) công ty bên ngoài kiểm thử xâm nhập cổng VíXanh tích hợp với Tới Nơi.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(a) không độc lập; (b) một phần — đồng nghiệp cùng đội; (c) cao — tester ngoài đội của tác giả nhưng trong cùng tổ chức; (d) rất cao — ngoài tổ chức. Lưu ý (b) là review (static testing) nhưng phân loại độc lập vẫn áp dụng cho mọi work product, không chỉ cho chạy test.</p>
</details>

**Câu 1 (K1).** Điều nào là một lợi ích của whole team approach theo CTFL?

- a) Loại bỏ nhu cầu có người làm kiểm thử chuyên trách.
- b) Tăng giao tiếp và cộng tác trong đội, tận dụng nhiều bộ kỹ năng.
- c) Bảo đảm mức độc lập kiểm thử cao nhất.
- d) Chuyển toàn bộ trách nhiệm chất lượng cho product owner.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — tester vẫn cùng viết acceptance test, thống nhất test strategy, truyền kiến thức. <b>b đúng</b> — lợi ích syllabus nêu. <b>c sai</b> — ngữ cảnh cần độc lập cao có thể không hợp với cách này. <b>d sai</b> — mọi người cùng chịu trách nhiệm chất lượng.</p>
</details>

**Câu 2 (K2).** Hoạt động nào chủ yếu thuộc test management role?

- a) Viết test script tự động cho API báo giá.
- b) Xác định test condition từ R-VOUCHER-07.
- c) So sánh tiến độ thực tế với kế hoạch và đưa ra hành động điều chỉnh.
- d) Chạy lại bộ test hồi quy sau khi sửa lỗi.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a sai</b> — test implementation (testing role). <b>b sai</b> — test analysis (testing role). <b>c đúng</b> — test monitoring và test control, trọng tâm của test management role cùng planning và completion. <b>d sai</b> — test execution (testing role).</p>
</details>

**Câu 3 (K2).** QA Engineer ngồi trong squad Promotion kiểm thử tính năng voucher do developer cùng squad viết. Theo CTFL, đây là mức độc lập nào?

- a) Không độc lập.
- b) Một phần.
- c) Cao.
- d) Rất cao.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — đó là khi tác giả tự test. <b>b đúng</b> — đồng nghiệp cùng đội; chức danh khác không đổi việc họ cùng đội. <b>c sai</b> — mức cao đòi tester ngoài đội của tác giả. <b>d sai</b> — rất cao là ngoài tổ chức.</p>
</details>

**Câu 4 (K2).** Điều nào là một **hạn chế** của độc lập kiểm thử?

- a) Tester độc lập nhận ra những kiểu defect khác với developer.
- b) Tester độc lập có thể chất vấn giả định của stakeholder.
- c) Developer có thể mất ý thức trách nhiệm về chất lượng.
- d) Thiên kiến của tester và tác giả khác nhau.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a, b sai</b> — đó là lợi ích. <b>c đúng</b> — hạn chế, cùng với cô lập, đối đầu, bị coi là nút thắt. <b>d sai</b> — khác biệt thiên kiến là lý do độc lập giúp tìm defect.</p>
</details>

**Câu 5 (K2).** Vì sao kỹ năng giao tiếp đặc biệt quan trọng với tester?

- a) Vì tester phải thuyết trình test plan trước ban giám đốc.
- b) Vì kết quả test có thể bị nghe như lời chỉ trích sản phẩm và tác giả, nên cần truyền đạt mang tính xây dựng.
- c) Vì tester là người duy nhất được phép trao đổi với khách hàng.
- d) Vì giao tiếp tốt thay được kiến thức kỹ thuật.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — không phải lý do syllabus nêu. <b>b đúng</b> — tester thường mang tin xấu, confirmation bias làm người nghe khó chấp nhận. <b>c sai</b> — không có quy định đó. <b>d sai</b> — mỗi kỹ năng phục vụ mục đích riêng.</p>
</details>

**Câu 6 (K2).** Phát biểu nào về vai trò trong kiểm thử là **đúng** theo CTFL v4.0.1?

- a) Test management role chỉ do người có chức danh Test Manager đảm nhận.
- b) Một người có thể giữ cả test management role và testing role cùng lúc.
- c) Testing role chịu trách nhiệm test planning và test completion.
- d) Trong Agile, không còn nhiệm vụ quản lý kiểm thử.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — có thể là team leader, development manager… <b>b đúng</b> — syllabus nói rõ. <b>c sai</b> — đó là trọng tâm của test management role. <b>d sai</b> — một phần việc quản lý chuyển cho chính đội Agile, không biến mất.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB® Certified Tester Foundation Level Syllabus **v4.0.1** (2024): mục 1.4.5 Roles in Testing; mục 1.5 Essential Skills and Good Practices in Testing (1.5.1, 1.5.2, 1.5.3); mục 5.1.1 (nội dung test plan, gồm stakeholders và independence of testing). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB® Glossary (tra các mục *test management*, *tester*, *whole team approach*, *independence of testing*): https://glossary.istqb.org/
- Trang chủ ISTQB: https://www.istqb.org/
- ISO/IEC/IEEE 29119-1 (khái niệm kiểm thử) và 29119-3 (tài liệu kiểm thử) — tham khảo khi viết phần stakeholder và vai trò trong test plan.
- Đặc tả lab Tới Nơi (`SPEC.md`, nhóm R-VOUCHER, R-CANCEL): https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-006 · Mục tiêu kiểm thử và Quality Assurance](/articles/muc-tieu-kiem-thu-va-quality-assurance) · [TEST-007 · Quy trình kiểm thử](/articles/quy-trinh-kiem-thu-test-process) · [TEST-008 · Kiểm thử trong SDLC](/articles/kiem-thu-trong-sdlc-agile-waterfall)

Bài tiếp theo: [Testware và các sản phẩm kiểm thử](/articles/testware-san-pham-kiem-thu).
