---
slug: test-case-design-test-analysis-test-condition
title: "Test analysis: từ test basis tới test condition cho voucher Tới Nơi"
summary: "Phân tích tám quy tắc voucher của Tới Nơi thành 23 test condition: đọc test basis có phương pháp, đặt tên condition, chọn mức chi tiết, ghi câu hỏi về đặc tả, ưu tiên sơ bộ theo rủi ro và truy vết R-VOUCHER → TCOND, kèm 32 test vitest kiểm chính kết quả phân tích."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-case-design, tcd-01, test-analysis, test-condition, test-basis, traceability, voucher, vitest, toi-noi]
language: vi
---

Đội Tới Nơi nhận yêu cầu «kiểm thử voucher» vào sáng thứ Hai. Một tester mở trình soạn thảo, gõ ngay test case đầu tiên: «nhập GIAM20, báo giá 7,4 km, mong đợi 37.000 đ». Đến chiều, bộ test có 15 test case, đều xanh. Không ai trả lời được câu «đã kiểm hết những gì voucher phải làm chưa?», vì chưa ai liệt kê **những gì voucher phải làm**. Bài này dạy bước bị bỏ qua đó: **test analysis**. Đầu vào là tám quy tắc R-VOUCHER của SPEC. Đầu ra là 23 **test condition** có tên, có nguồn, có mức ưu tiên, cộng năm câu hỏi gửi người viết đặc tả. Lab kiểm chính kết quả phân tích bằng code.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TCD-01</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>TCD · Thiết kế Test Case</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Component; System</td></tr>
<tr><th scope="row">Test Type</th><td>Functional</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng (test analysis đứng trước việc chọn kỹ thuật)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/quy-trinh-kiem-thu-test-process">TEST-007 · Quy trình kiểm thử</a>, <a href="/articles/testware-san-pham-kiem-thu">TEST-010 · Testware</a></td></tr>
<tr><th scope="row">Thực hành</th><td>23 test condition cho voucher (SPEC R-VOUCHER-01…08, R-FARE-06, R-FARE-07, R-FOOD-05) viết thành dữ liệu có kiểu, test truy vết và 24 bằng chứng chạy được: <code>examples/toi-noi-testing/articles/test-case-design-test-analysis-test-condition/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.4.1, 1.4.3, 1.4.4 · FL-1.4.1 (K2), FL-1.4.3 (K2), FL-1.4.4 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Viết test case ngay khi đọc xong đặc tả là thói quen phổ biến, và nó có ba hệ quả. Thứ nhất, test case phản ánh **những gì người viết nghĩ tới trước**: thường là đường đi vui vẻ và một hai lỗi hiển nhiên. Thứ hai, không có gì để **đo độ phủ**: 15 test case xanh nghe có vẻ nhiều, nhưng không biết là 15 trên bao nhiêu. Thứ ba, các câu hỏi về đặc tả bị **chôn trong test case**: tester tự chọn một cách hiểu, viết kết quả mong đợi theo cách hiểu đó, và người viết đặc tả không bao giờ biết đã có chỗ mơ hồ.

Test analysis tách việc «kiểm **cái gì**» khỏi việc «kiểm **thế nào**». Kết quả của nó là danh sách test condition: mỗi condition là một khía cạnh kiểm thử được của test object, đủ cụ thể để biết khi nào đã kiểm xong, nhưng chưa gắn với giá trị đầu vào nào. Danh sách đó làm ba việc mà một chồng test case không làm được:

- là **mẫu số** cho độ phủ: 22 trên 23 condition đã có bằng chứng, condition thứ 23 đang chờ câu trả lời của BA;
- là **điểm neo truy vết**: đổi R-VOUCHER-06 thì biết ngay sáu condition và các test case bên dưới chúng bị ảnh hưởng;
- là **chỗ để đặt câu hỏi**: condition nào không viết được kết quả rõ ràng thì đó là chỗ đặc tả có vấn đề.

Khi nào cần làm test analysis tường minh? Gần như luôn luôn. Khác nhau chỉ là hình thức: dự án có kiểm toán sẽ có danh sách condition trong công cụ quản lý test; đội Agile có thể dùng acceptance criteria của user story làm test condition ([TCD-07](/articles/test-case-design-user-story-acceptance-criteria)); một developer viết unit test cũng nên liệt kê nhanh các condition trong phần mô tả pull request. Bài này dùng hình thức chặt nhất để thấy rõ từng bước. Khi đã quen, bạn sẽ biết bước nào rút gọn được.

## 2. Lý thuyết theo ISTQB

### 2.1. Test analysis trong CTFL v4.0.1 (mục 1.4.1)

CTFL chia quy trình kiểm thử thành các nhóm hoạt động: test planning, test monitoring và test control, test analysis, test design, test implementation, test execution, test completion ([TEST-007](/articles/quy-trinh-kiem-thu-test-process)). Với **test analysis**, syllabus nêu các việc chính:

| Việc trong test analysis | Ý nghĩa thực hành |
|---|---|
| Phân tích test basis để nhận diện **tính năng kiểm thử được** (testable feature) | chia voucher thành các mảng: nhận diện mã, thời hạn, phạm vi, tiền giảm, lượt dùng, thứ tự lỗi |
| Định nghĩa và **ưu tiên** các test condition liên quan, cùng rủi ro và mức rủi ro | 23 condition, mỗi condition một rủi ro, mức ưu tiên cao / trung bình / thấp |
| Đánh giá test basis và test object để **tìm defect** và đánh giá **khả năng kiểm thử** (testability) | năm câu hỏi Q-01…Q-05; một condition không kiểm được qua test object hiện có |
| Thường được hỗ trợ bởi **kỹ thuật kiểm thử** | biết trước condition nào sẽ cần BVA, decision table… nhưng chưa áp ở đây |

Syllabus tóm tắt: test analysis trả lời câu hỏi **«kiểm thử cái gì?»** dưới dạng các **tiêu chí độ phủ đo được**. Test design tiếp theo trả lời **«kiểm thử thế nào?»**: biến test condition thành test case và testware khác, nhận diện coverage item, xác định yêu cầu về test data và môi trường.

Hai chi tiết hay bị đọc lướt:

1. **Đánh giá cả test object, không chỉ test basis.** Nếu một condition không có cách nào quan sát được qua giao diện của test object (không có API, không có log, không có trạng thái đọc được), đó là vấn đề testability, cần nêu sớm. TCOND-VOUCHER-23 trong bài này là ví dụ.
2. **Phân tích là hoạt động lặp.** Các hoạt động trong CTFL không chạy theo thứ tự cứng. Khi thiết kế test case ở TCD-02 hoặc chạy test, ta thường phát hiện condition còn thiếu và quay lại bổ sung.

### 2.2. Test basis, test condition, coverage item

| Thuật ngữ | Định nghĩa (diễn đạt lại từ ISTQB Glossary) | Ví dụ trong bài |
|---|---|---|
| test basis | toàn bộ kiến thức dùng làm cơ sở cho test analysis và test design | SPEC v1.1, nhóm R-VOUCHER và các quy tắc nối với nó |
| test object | thành phần hoặc hệ thống được kiểm thử | bước voucher của báo giá và tạo chuyến (`voucher.ts`, `quote.ts`, `trips.ts`) |
| test condition | một khía cạnh kiểm thử được của thành phần hoặc hệ thống, được nhận diện làm cơ sở cho việc kiểm thử | TCOND-VOUCHER-12: «tiền giảm không vượt maxDiscount» |
| coverage item | thuộc tính hoặc tổ hợp thuộc tính suy ra từ một hoặc nhiều test condition bằng một kỹ thuật kiểm thử, dùng để đo độ phủ | hai giá trị biên «A = minAmount − 1 đ» và «A = minAmount» suy từ TCOND-VOUCHER-10 bằng BVA |
| test case | bộ tiền điều kiện, đầu vào, hành động, kết quả mong đợi và hậu điều kiện, xây dựng từ test condition | «HN10, HN, xe máy 6,3 km, 09:00 ⇒ giảm 4.035 đ, tổng 37.000 đ» |

Quan hệ giữa chúng là **nhiều-nhiều** nhưng có hướng: một yêu cầu sinh ra nhiều condition; một condition có thể đến từ nhiều yêu cầu (TCOND-VOUCHER-13 đến từ R-FARE-06 và R-VOUCHER-06); một test case có thể phủ nhiều condition; một condition cần nhiều test case khi có nhiều coverage item. Ma trận nhiều-nhiều đầy đủ là đề tài của [TCD-08](/articles/test-case-design-traceability-matrix). Bài này chỉ dựng mắt xích đầu tiên: test basis → test condition.

### 2.3. Testware của test analysis (mục 1.4.3)

Theo CTFL mục 1.4.3, sản phẩm của test analysis gồm **test condition đã được ưu tiên** (ví dụ acceptance criteria) và **defect report về defect trong test basis** (nếu defect không được sửa trực tiếp). Bài [TEST-010](/articles/testware-san-pham-kiem-thu) đã liệt kê testware của mọi hoạt động. Ở đây ta làm ra đúng hai loại đó cho voucher: bảng 23 condition (mục 4.3) và danh sách năm câu hỏi Q-01…Q-05 (mục 4.6). Câu hỏi nào được BA xác nhận là lỗi thì trở thành defect report về test basis.

### 2.4. Truy vết bắt đầu từ test analysis (mục 1.4.4)

CTFL mục 1.4.4 yêu cầu lập và duy trì truy vết giữa các phần tử của test basis, testware gắn với chúng (test condition, rủi ro, test case), kết quả test và defect. Truy vết giúp đánh giá độ phủ, tìm ảnh hưởng của thay đổi, phục vụ kiểm toán và làm báo cáo dễ hiểu với người không chuyên kỹ thuật. Nếu test condition không ghi nguồn ngay lúc tạo, sau này rất khó nối lại: không ai nhớ «condition về nửa đêm» đến từ R-VOUCHER-04 hay R-GEN-02. Vì vậy mỗi condition trong bài có trường `basis` bắt buộc, và lab kiểm trường đó theo hai chiều.

### 2.5. Thuật ngữ Anh – Việt

| Thuật ngữ | Cách dùng trong bài |
|---|---|
| test analysis | phân tích kiểm thử — hoạt động trả lời «kiểm thử cái gì» |
| test basis | giữ nguyên tiếng Anh |
| testable feature | tính năng kiểm thử được — một mảng của test object đủ độc lập để phân tích riêng |
| test condition | giữ nguyên; số nhiều vẫn viết «test condition» |
| coverage item | phần tử độ phủ, do kỹ thuật kiểm thử sinh ra từ test condition |
| testability | khả năng kiểm thử: quan sát và điều khiển được test object tới mức nào |
| product risk | rủi ro sản phẩm: khả năng sản phẩm không đáp ứng nhu cầu |
| traceability | truy vết |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Mã <code>TCOND-VOUCHER-nn</code>, câu khẳng định làm tên condition, các «lăng kính» đọc đặc tả ở mục 4.1, thang ưu tiên ba mức và kiểm kết quả phân tích bằng test tự động là thực hành của bài, không phải yêu cầu của CTFL. CTFL chỉ yêu cầu test condition được định nghĩa, ưu tiên và truy vết được. ISO/IEC/IEEE 29119-2 mô tả quy trình thiết kế test đi từ tập tính năng tới test condition, rồi test coverage item, rồi test case. Nếu tổ chức của bạn theo bộ chuẩn đó, có thể đối chiếu các bước trong bài với mô tả ở đó.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Test basis: voucher và các quy tắc nối với nó

Voucher của Tới Nơi là mã giảm phần trăm có trần, có điều kiện tối thiểu, giới hạn theo dịch vụ, thành phố, ngày và số lượt mỗi ngày. Test basis gồm tám quy tắc R-VOUCHER và các quy tắc ở nhóm khác quyết định voucher được trừ vào đâu:

| Mã | Nội dung (tóm tắt SPEC v1.1) |
|---|---|
| R-VOUCHER-01 | thuộc tính: `percent` 1–50, `maxDiscount`, `minAmount`, `kinds`, `cityCodes` (rỗng = tất cả), `validFrom`/`validTo` tính cả hai đầu theo giờ VN, `perCustomerPerDay` (mặc định 1) |
| R-VOUCHER-02 | bỏ khoảng trắng hai đầu, viết hoa; chuỗi rỗng coi như không nhập |
| R-VOUCHER-03 | không có mã ⇒ `VOUCHER_NOT_FOUND` |
| R-VOUCHER-04 | ngày đặt ngoài [validFrom; validTo] ⇒ `VOUCHER_EXPIRED` |
| R-VOUCHER-05 | dịch vụ hoặc thành phố ngoài phạm vi ⇒ `VOUCHER_NOT_APPLICABLE` |
| R-VOUCHER-06 | A < `minAmount` ⇒ `VOUCHER_MIN_AMOUNT`; tiền giảm = min(percent% × A, maxDiscount) |
| R-VOUCHER-07 | lượt dùng: báo giá không tiêu, tạo chuyến tiêu, huỷ trả lại; kiểm hai lần; thời hạn và phạm vi chỉ kiểm khi báo giá |
| R-VOUCHER-08 | thứ tự lỗi: NOT_FOUND → EXPIRED → NOT_APPLICABLE → MIN_AMOUNT → USAGE_LIMIT |
| R-FARE-06 | với đặt xe/bưu kiện, A là số tiền sau bước 3 (đã gồm cao điểm và mưa) |
| R-FARE-07 | voucher không kéo cước xuống dưới cước tối thiểu |
| R-FOOD-05 | với đồ ăn, A là tiền món; voucher không trừ phí giao (R-FOOD-04) |
| R-GEN-02 | mọi quy tắc theo ngày dùng giờ Việt Nam UTC+7 |

Dữ liệu mẫu (SPEC Phụ lục A) có bốn voucher: `GIAM20` (20%, trần 30.000 đ, mọi dịch vụ, mọi thành phố, 1 lần/ngày), `FOOD15` (15%, trần 25.000 đ, tối thiểu 50.000 đ, chỉ đồ ăn, 2 lần/ngày), `HN10` (10%, trần 15.000 đ, tối thiểu 40.000 đ, chỉ đặt xe ở Hà Nội), `XMAS50` (50%, trần 50.000 đ, chỉ 24–26/12/2025, đã hết hạn).

### 3.2. System context

Trong kiến trúc đích, voucher thuộc **Promotion Service**; Pricing & Quote gọi sang để kiểm và tính tiền giảm, Booking gọi lại để kiểm lượt khi tạo chuyến. Lab gộp thành modular monolith (SPEC R-ARCH-01): quy tắc thuần ở `src/domain/voucher.ts`, thứ tự kiểm ở `src/app/quote.ts`, kiểm lượt lần hai ở `createTrip` của `src/app/trips.ts`. Điều này quan trọng với test analysis: voucher **không phải một điểm kiểm tra** mà là ba thời điểm (báo giá, tạo chuyến, huỷ chuyến) và hai loại đơn (đặt xe/bưu kiện, đồ ăn). Một danh sách condition chỉ nghĩ tới «lúc nhập mã» sẽ bỏ sót một nửa.

### 3.3. Rủi ro

| Rủi ro sản phẩm | Hậu quả | Mức |
|---|---|---|
| tính sai tiền giảm (sai cơ sở A, quên trần, sai thứ tự với cước tối thiểu) | lệch giá trên mọi chuyến dùng voucher, thất thu hoặc khiếu nại | cao |
| voucher dùng được nhiều hơn số lượt cho phép | lạm dụng khuyến mãi, ngân sách khuyến mãi bị vượt | cao |
| sai ngày quanh nửa đêm (máy chủ chạy UTC) | voucher mở sớm hoặc đóng muộn 7 giờ; khách bị từ chối oan | cao |
| sai phạm vi dịch vụ / thành phố | chương trình khuyến mãi địa phương bị dùng toàn quốc | trung bình |
| sai thông báo lỗi | khách không hiểu vì sao bị từ chối, gọi tổng đài | thấp |

Ước tính quy mô (**giả định bài tập**): giả sử có 1 triệu chuyến đặt xe mỗi ngày và 10% trong số đó dùng voucher. Nếu tiền giảm bị tính trên cước cơ bản thay vì sau phụ phí trời mưa, mỗi chuyến mưa bị giảm thiếu 1.600 đ (20% × 8.000 đ), trước khi làm tròn. Một ngày mưa trên toàn hệ thống, 100.000 chuyến × 1.600 đ ≈ 160 triệu đồng thu thừa của khách. Lỗi này khó phát hiện bằng mắt vì con số trên màn hình vẫn «trông hợp lý».

## 4. Phân tích kỹ thuật

### 4.1. Đọc test basis có phương pháp

Đọc đặc tả một lượt rồi viết condition theo cảm hứng sẽ bỏ sót đúng những gì đặc tả nói ngầm. Cách đọc của bài là đi qua cùng một đoạn đặc tả bằng năm «lăng kính», mỗi lăng kính đặt một câu hỏi:

| Lăng kính | Câu hỏi | Áp vào voucher |
|---|---|---|
| Đầu vào, đầu ra | Cái gì đi vào, cái gì quan sát được ở đầu ra? | mã khách nhập; ngày đặt; A; đầu ra là `voucherCode`, `discount`, `total`, mã lỗi |
| Quy tắc và ngưỡng | Có phép so sánh, ngưỡng, công thức nào? | `minAmount` (≥), `maxDiscount` (min), `percent%`, [validFrom; validTo] tính cả hai đầu |
| Thời gian | Quy tắc nào phụ thuộc giờ, ngày, thứ tự thời điểm? | ngày theo giờ VN; lượt theo ngày; báo giá ở ngày này, tạo chuyến ở ngày khác |
| Trạng thái và bộ đếm | Có gì được tiêu, được trả, được nhớ giữa các lần gọi? | lượt dùng: báo giá không tiêu, tạo chuyến tiêu, huỷ trả |
| Tương tác giữa quy tắc | Quy tắc này nằm ở đâu so với quy tắc khác? | voucher sau cao điểm và mưa, trước cước tối thiểu; đồ ăn khác đặt xe; thứ tự lỗi |

Kết quả của lượt đọc đầu là sáu **tính năng kiểm thử được**: nhận diện mã (R-VOUCHER-02, 03), thời hạn (01, 04), phạm vi (01, 05), tiền giảm (06 cùng R-FARE-06, 07, R-FOOD-05), lượt dùng (07), thứ tự lỗi (08). Mỗi mảng được phân tích riêng. Chia như vậy giúp không lạc giữa tám quy tắc, và mỗi mảng sau này thường ứng với một nhóm test case dùng chung tiền điều kiện.

Lăng kính thứ năm là nơi tìm được nhiều condition nhất mà người đọc lướt bỏ qua. R-VOUCHER-06 tự nó chỉ nói «tiền giảm = min(percent% × A, maxDiscount)». Chỉ khi đặt nó cạnh R-FARE-03 mới thấy A **đã gồm** hệ số cao điểm và phụ phí trời mưa (TCOND-VOUCHER-13), và kết quả còn phải qua bước cước tối thiểu (TCOND-VOUCHER-15). Đặt cạnh R-FOOD-05 thì thấy đồ ăn dùng **cơ sở khác** (TCOND-VOUCHER-14).

### 4.2. Đặt tên test condition

Một test condition tốt là **một câu khẳng định** có bốn tính chất:

1. **Kiểm chứng được**: đọc xong biết phải quan sát gì để nói đạt hay không đạt. «Voucher được xử lý hợp lý» không kiểm chứng được. «Tiền giảm không vượt maxDiscount» thì kiểm chứng được.
2. **Một khía cạnh**: một condition, một lý do để đỏ. Nếu một condition đỏ mà phải đọc log mới biết vế nào sai, nên tách nó ra.
3. **Không chứa giá trị cụ thể**: không có `GIAM20`, không có `7,4 km`, không có `37.000 đ`. Giá trị thuộc về test case; đưa giá trị vào condition sẽ khoá sớm lựa chọn của test design và làm condition lỗi thời khi dữ liệu mẫu đổi.
4. **Nói cả kết quả**: «mã không tồn tại» chỉ là một tình huống. «Mã không tồn tại bị từ chối với `VOUCHER_NOT_FOUND`» mới là condition, vì nó nêu điều phải đúng.

Quy ước mã của nhánh TCD là `TCOND-<NHÓM>-nn`, đánh số liên tục trong nhóm. Bài [TEST-010](/articles/testware-san-pham-kiem-thu) dùng hậu tố chữ (`03a`, `03b`) để bám từng yêu cầu. Cách đó tiện khi mỗi condition chỉ có một nguồn, nhưng vỡ khi condition có nhiều nguồn như TCOND-VOUCHER-13. Số liên tục cộng trường `basis` riêng thì không vỡ: nguồn là dữ liệu, không nằm trong tên.

### 4.3. 23 test condition cho voucher

| Mã | Test condition | Nguồn | Ưu tiên |
|---|---|---|---|
| TCOND-VOUCHER-01 | Mã có khoảng trắng hai đầu hoặc chữ thường được chuẩn hoá rồi mới tra | R-VOUCHER-02 | Trung bình |
| TCOND-VOUCHER-02 | Mã rỗng hoặc chỉ gồm khoảng trắng được coi như không nhập voucher | R-VOUCHER-02 | Thấp |
| TCOND-VOUCHER-03 | Mã không có trong danh sách bị từ chối với `VOUCHER_NOT_FOUND` | R-VOUCHER-03 | Trung bình |
| TCOND-VOUCHER-04 | Ngày đặt trong [validFrom; validTo], tính cả ngày đầu và ngày cuối, được chấp nhận | R-VOUCHER-01, -04 | Trung bình |
| TCOND-VOUCHER-05 | Ngày đặt trước validFrom hoặc sau validTo bị từ chối với `VOUCHER_EXPIRED` | R-VOUCHER-04 | Trung bình |
| TCOND-VOUCHER-06 | Ngày đặt tính theo giờ Việt Nam, không theo múi giờ máy chủ | R-VOUCHER-04, R-GEN-02 | Cao |
| TCOND-VOUCHER-07 | Dịch vụ không thuộc `kinds` bị từ chối với `VOUCHER_NOT_APPLICABLE` | R-VOUCHER-05 | Trung bình |
| TCOND-VOUCHER-08 | Thành phố không thuộc `cityCodes` bị từ chối với `VOUCHER_NOT_APPLICABLE` | R-VOUCHER-05 | Trung bình |
| TCOND-VOUCHER-09 | `kinds` rỗng là mọi dịch vụ; `cityCodes` rỗng là mọi thành phố | R-VOUCHER-01, -05 | Trung bình |
| TCOND-VOUCHER-10 | A < minAmount bị từ chối với `VOUCHER_MIN_AMOUNT`; A = minAmount hợp lệ | R-VOUCHER-06 | Cao |
| TCOND-VOUCHER-11 | Khi chưa chạm trần, tiền giảm bằng percent% × A | R-VOUCHER-06 | Cao |
| TCOND-VOUCHER-12 | Tiền giảm không vượt maxDiscount | R-VOUCHER-06 | Cao |
| TCOND-VOUCHER-13 | Với đặt xe và bưu kiện, A là số tiền sau bước 3 (đã gồm cao điểm và mưa) | R-FARE-06, R-VOUCHER-06 | Cao |
| TCOND-VOUCHER-14 | Với đồ ăn, A là tiền món: minAmount so với tiền món và voucher không trừ phí giao | R-FOOD-05, R-VOUCHER-06 | Cao |
| TCOND-VOUCHER-15 | Voucher không kéo cước xuống dưới cước tối thiểu của loại xe | R-FARE-07, R-FARE-06 | Cao |
| TCOND-VOUCHER-16 | Báo giá không tiêu lượt; tạo chuyến từ báo giá có voucher tiêu một lượt | R-VOUCHER-07 | Cao |
| TCOND-VOUCHER-17 | Đã dùng đủ lượt trong ngày thì bị từ chối `VOUCHER_USAGE_LIMIT`, cả lúc báo giá lẫn lúc tạo chuyến | R-VOUCHER-07, R-QUOTE-03 | Cao |
| TCOND-VOUCHER-18 | Chuyến bị huỷ trả lại lượt dùng | R-VOUCHER-07 | Trung bình |
| TCOND-VOUCHER-19 | Lượt dùng đếm theo ngày giờ Việt Nam; sang ngày mới thì có lượt mới | R-VOUCHER-07, R-GEN-02 | Trung bình |
| TCOND-VOUCHER-20 | perCustomerPerDay lớn hơn 1 cho phép dùng đúng chừng đó lượt trong ngày | R-VOUCHER-01, -07 | Trung bình |
| TCOND-VOUCHER-21 | Thời hạn và phạm vi chỉ kiểm khi báo giá, không kiểm lại khi tạo chuyến từ báo giá còn hiệu lực | R-VOUCHER-07, R-QUOTE-04 | Trung bình |
| TCOND-VOUCHER-22 | Vi phạm nhiều quy tắc voucher thì chỉ báo lỗi đứng trước theo thứ tự R-VOUCHER-08 | R-VOUCHER-08 | Thấp |
| TCOND-VOUCHER-23 | Voucher có percent ngoài 1–50 không được áp dụng (bị chặn bởi Q-04) | R-VOUCHER-01 | Thấp |

Đọc bảng theo cột «Nguồn» sẽ thấy một điểm thú vị: R-VOUCHER-01 (thuộc tính) không có condition nào của riêng nó, trừ TCOND-VOUCHER-23. Thuộc tính tự nó không có hành vi; hành vi xuất hiện khi quy tắc khác dùng thuộc tính. Vì vậy R-VOUCHER-01 được phủ **gián tiếp** qua các condition về thời hạn, phạm vi và lượt. Đây là chuyện bình thường. Điều cần tránh là để một yêu cầu không có condition nào mà không ai nhận ra, và lab có test riêng cho việc đó (mục 5.3).

### 4.4. Chọn mức chi tiết: quá thô, vừa, quá mịn

Mức chi tiết của test condition là quyết định thiết kế, không phải sở thích. Cùng một mảng «tiền giảm» có thể viết ở ba mức:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Ba mức chi tiết của test condition cho tiền giảm voucher: quá thô là một condition voucher tính đúng tiền; vừa là năm condition về tỉ lệ, trần, cơ sở A, đồ ăn và cước tối thiểu; quá mịn là các dòng có giá trị cụ thể như GIAM20 7,4 km bằng 37.000 đồng, vốn là test case.">
<defs>
<marker id="tcd01-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="30" width="250" height="200" rx="8" class="f-warn-soft s-warn"/>
<text x="145" y="56" text-anchor="middle" class="t-b t-warn">Quá thô (1)</text>
<text x="145" y="92" text-anchor="middle" class="t-10">«Voucher tính đúng</text>
<text x="145" y="110" text-anchor="middle" class="t-10">tiền giảm»</text>
<text x="145" y="150" text-anchor="middle" class="t-small">đỏ hay xanh đều không nói</text>
<text x="145" y="168" text-anchor="middle" class="t-small">được quy tắc nào đã kiểm</text>
<text x="145" y="198" text-anchor="middle" class="t-small">độ phủ 1/1 vô nghĩa</text>
<rect x="335" y="30" width="250" height="200" rx="8" class="f-phys-soft s-phys"/>
<text x="460" y="56" text-anchor="middle" class="t-b t-phys">Vừa (5) — TCOND-10…15</text>
<text x="460" y="86" text-anchor="middle" class="t-10">ngưỡng minAmount (≥)</text>
<text x="460" y="106" text-anchor="middle" class="t-10">percent% × A khi dưới trần</text>
<text x="460" y="126" text-anchor="middle" class="t-10">không vượt maxDiscount</text>
<text x="460" y="146" text-anchor="middle" class="t-10">A sau cao điểm và mưa</text>
<text x="460" y="166" text-anchor="middle" class="t-10">đồ ăn: A = tiền món</text>
<text x="460" y="186" text-anchor="middle" class="t-10">không dưới cước tối thiểu</text>
<text x="460" y="212" text-anchor="middle" class="t-small">mỗi condition một lý do để đỏ</text>
<rect x="650" y="30" width="250" height="200" rx="8" class="f-surface s-line"/>
<text x="775" y="56" text-anchor="middle" class="t-b">Quá mịn (hàng chục)</text>
<text x="775" y="88" text-anchor="middle" class="t-10 t-mono">GIAM20 · 7,4 km ⇒ 37.000</text>
<text x="775" y="108" text-anchor="middle" class="t-10 t-mono">GIAM20 · mưa ⇒ 43.000</text>
<text x="775" y="128" text-anchor="middle" class="t-10 t-mono">GIAM20 · ô tô 12 km ⇒ 127.000</text>
<text x="775" y="148" text-anchor="middle" class="t-10 t-mono">HN10 · 6,2 km ⇒ MIN_AMOUNT</text>
<text x="775" y="168" text-anchor="middle" class="t-10">…</text>
<text x="775" y="198" text-anchor="middle" class="t-small">đây đã là test case cụ thể</text>
<line x1="270" y1="130" x2="333" y2="130" class="s-muted" stroke-width="1.6" marker-end="url(#tcd01-a)"/>
<line x1="585" y1="130" x2="648" y2="130" class="s-muted dash" stroke-width="1.6" marker-end="url(#tcd01-a)"/>
<text x="302" y="120" text-anchor="middle" class="t-small">tách</text>
<text x="617" y="120" text-anchor="middle" class="t-small">test design</text>
<text x="460" y="268" text-anchor="middle" class="t-small">Test analysis dừng ở cột giữa; cột phải là đầu ra của test design và test implementation (TCD-02, TCD-03)</text>
</svg>
</div>
<figcaption><b>Hình 1 · Ba mức chi tiết cho cùng một mảng «tiền giảm».</b> Mức vừa cho mỗi condition đúng một lý do để đỏ, đủ để đo độ phủ và truy vết, mà chưa khoá lựa chọn giá trị của test design.</figcaption>
</figure>

**Quá thô** có hai triệu chứng: không viết được kết quả mong đợi mà không nói «đúng», và một test case đỏ không cho biết quy tắc nào bị vi phạm. Độ phủ 100% của một condition quá thô gần như không mang thông tin.

**Quá mịn** cũng có hai triệu chứng: condition chứa giá trị cụ thể, và số condition tăng theo số dòng dữ liệu. Ở mức đó, condition trùng với test case, bảng truy vết phình to mà không thêm thông tin, và mỗi lần đổi dữ liệu mẫu (ví dụ GIAM20 đổi trần từ 30.000 đ lên 40.000 đ) lại phải sửa cả lớp phân tích.

Không có công thức cho mức «vừa». Bài dùng bốn câu hỏi để quyết định:

| Câu hỏi | Nếu «có» thì |
|---|---|
| Hai vế của condition có thể sai **độc lập** với nhau không? | tách. TCOND-VOUCHER-11 (tỉ lệ) và -12 (trần) tách vì một bản cài đặt có thể đúng tỉ lệ mà quên trần |
| Hai vế có **rủi ro khác mức** không? | tách, để ưu tiên riêng. Thời hạn (-05, trung bình) và múi giờ (-06, cao) tách vì lý do này |
| Condition sẽ cần **kỹ thuật kiểm thử** khác nhau không? | tách. -10 là ngưỡng (BVA), -22 là thứ tự lỗi (decision table) |
| Người đọc báo cáo (BA, PO) có cần thấy riêng nó không? | tách nếu họ hỏi riêng; gộp nếu với họ đó là một chuyện |

Mức chi tiết còn phụ thuộc bối cảnh. Ở test level component, developer có thể dùng condition mịn hơn, gần với từng nhánh code. Ở acceptance test, PO thường chỉ cần mức «lượt dùng hoạt động theo R-VOUCHER-07». Ngành có kiểm toán thường cần chi tiết hơn. Điều cần giữ là **nhất quán trong một bộ**: một bảng vừa có «voucher tính đúng» vừa có «GIAM20 7,4 km» thì không đo được gì.

### 4.5. Test condition như đơn vị đo độ phủ

Test analysis trả lời «cái gì» dưới dạng tiêu chí độ phủ đo được. Với bảng ở mục 4.3, tiêu chí đơn giản nhất là **độ phủ test condition**: số condition có ít nhất một test case đạt chia cho tổng số condition. Ở cuối bài, lab cho 22/23 ≈ 95,7%; condition còn lại bị chặn có lý do ghi rõ.

Độ phủ test condition là thước đo thô. Nó cho biết **không có khía cạnh nào bị bỏ quên**, nhưng không cho biết mỗi khía cạnh đã được kiểm kỹ tới đâu. Khi test design áp kỹ thuật, mỗi condition sinh ra **coverage item** chi tiết hơn:

| Test condition | Kỹ thuật dự kiến (test design) | Coverage item |
|---|---|---|
| TCOND-VOUCHER-10 (ngưỡng minAmount) | BVA hai giá trị ([TEST-032](/articles/boundary-value-analysis)) | A = minAmount − 1 đ; A = minAmount |
| TCOND-VOUCHER-11, -12 (tỉ lệ, trần) | EP ([TEST-031](/articles/equivalence-partitioning)) + BVA | phân vùng «dưới trần», «trên trần»; biên percent% × A = maxDiscount |
| TCOND-VOUCHER-16…18 (lượt dùng) | state transition ([TEST-034](/articles/state-transition-testing-co-ban)) | các chuyển «còn lượt → hết lượt → còn lượt» |
| TCOND-VOUCHER-22 (thứ tự lỗi) | decision table ([TEST-033](/articles/decision-table-testing)) | các cột có từ hai lỗi trở lên |

Bảng trên là **ghi chú cho test design**, không phải test design. Viết nó ngay trong test analysis có lợi: khi ước lượng công sức, ta biết TCOND-VOUCHER-22 sẽ nở thành một bảng quyết định, còn TCOND-VOUCHER-02 chỉ cần một test case. Cách chọn và phối hợp kỹ thuật cho cả một tính năng là đề tài của [TCD-06](/articles/test-case-design-phoi-hop-ky-thuat).

### 4.6. Defect trong test basis tìm thấy khi phân tích

Viết condition buộc ta trả lời «điều gì phải đúng». Chỗ nào không trả lời được từ đặc tả thì đó là ứng viên defect của test basis. Năm câu hỏi dưới đây phát sinh trong lúc viết bảng 4.3. Mỗi câu ghi loại vấn đề và condition liên quan:

| Mã | Câu hỏi gửi người viết đặc tả | Loại | Liên quan |
|---|---|---|---|
| Q-01 | Khi cước tối thiểu (R-FARE-07) nâng giá lên, «tiền giảm» trả cho khách (`discount`, R-FARE-10) là tiền giảm tính theo R-VOUCHER-06 hay phần thực sự được giảm? | mơ hồ | TCOND-VOUCHER-15 |
| Q-02 | Khoảng trắng **ở giữa** mã («GIAM 20») có được bỏ không, hay chỉ bỏ hai đầu? | cần xác nhận | TCOND-VOUCHER-01, -03 |
| Q-03 | Chuyến khách huỷ **có phí** vẫn trả lại lượt voucher? Có giới hạn số lần huỷ để lấy lại lượt không? | cần xác nhận | TCOND-VOUCHER-18 |
| Q-04 | Voucher có percent ngoài 1–50 hoặc maxDiscount = 0 thì xử lý thế nào, ai kiểm, mã lỗi gì? | thiếu | TCOND-VOUCHER-23 |
| Q-05 | «Cùng ngày» của chuyến đã tạo tính theo thời điểm tạo chuyến hay thời điểm báo giá của chuyến đó? | mơ hồ | TCOND-VOUCHER-19 |

Q-01 là ví dụ đáng xem kỹ. Chuyến xe máy 1 km với GIAM20: cước cơ bản 12.000 + 4.500 × 1 = 16.500 đ; giảm 20% = 3.300 đ; còn 13.200 đ, thấp hơn 15.000 đ nên nâng lên 15.000 đ. Bảng chi tiết ghi `discount` = 3.300 đ. Không có voucher, khách trả 16.500 đ làm tròn lên 17.000 đ. Vậy khách thực sự chỉ được lợi **2.000 đ**. Nếu màn hình hiện «Bạn được giảm 3.300 đ», khách cộng lại sẽ thấy không khớp. SPEC không sai, vì R-FARE-10 nói rõ bảng chi tiết trả «tiền giảm» của bước 4. Nhưng SPEC **không nói** con số nào được hiển thị cho khách và đối soát với ngân sách khuyến mãi. Đó là lỗ hổng, không phải lỗi code.

Q-03 khác loại: SPEC rõ ràng (mọi chuyến bị huỷ đều trả lượt), nhưng tester thấy rủi ro nghiệp vụ. Khách có thể đặt, huỷ miễn phí khi chuyến còn đang tìm tài xế, rồi đặt lại, lặp vô hạn mà không mất lượt. Hành vi đó không hại tiền trực tiếp, nhưng ảnh hưởng chỉ số dùng khuyến mãi. Câu hỏi kiểu «SPEC có thật sự muốn thế không?» vẫn thuộc trách nhiệm của tester trong test analysis. Đây là đóng góp của kiểm thử tĩnh ([static testing](/articles/static-testing-la-gi)) lên đặc tả.

Q-04 cho thấy vấn đề **testability**. R-VOUCHER-01 ràng buộc percent trong 1–50, nhưng test object (báo giá, tạo chuyến) không có đường nào tạo voucher. Voucher đến từ dữ liệu mẫu, công cụ quản trị nằm ngoài lab. Condition -23 vì thế không thiết kế test case được qua test object hiện có. Ta ghi lại, gắn `blockedBy: Q-04`, và **không giả vờ** nó đã được phủ.

Cách xử lý mỗi câu hỏi: gửi cho BA hoặc PO, chờ trả lời. Nếu câu trả lời là «đặc tả sai hoặc thiếu, sẽ sửa», nhưng chưa sửa ngay, câu hỏi trở thành **defect report về test basis**, đúng loại testware CTFL 1.4.3 liệt kê. Nếu câu trả lời là «đặc tả đúng, đây là ý đồ», cập nhật condition (thêm ví dụ vào ghi chú) và đóng câu hỏi. Trong thời gian chờ, **không** viết kết quả mong đợi theo phỏng đoán. Lab ghi lại hành vi hiện tại của Q-01 và Q-02 (mục 5.5) để làm bằng chứng, nhưng chú thích rõ đó là hành vi quan sát được, chưa phải kết quả mong đợi đã được xác nhận.

### 4.7. Ưu tiên sơ bộ theo rủi ro

CTFL đặt việc ưu tiên test condition ngay trong test analysis: condition cùng rủi ro và mức rủi ro liên quan. Ở giai đoạn này chưa có test case nên chưa xếp được thứ tự chạy ([TCD-09](/articles/test-case-design-uu-tien-review-bao-tri) dạy ưu tiên test case). Mục tiêu chỉ là phân **công sức thiết kế**: condition ưu tiên cao được thiết kế kỹ hơn, dùng nhiều coverage item hơn, và phải có bằng chứng trước khi phát hành.

Bài dùng ma trận hai trục quen thuộc: **khả năng xảy ra** (logic phức tạp, nhiều quy tắc chạm nhau, phụ thuộc thời gian) và **tác động** (tiền, lạm dụng, trải nghiệm khách).

| | Tác động thấp | Tác động trung bình | Tác động cao (tiền, lạm dụng) |
|---|---|---|---|
| **Khả năng cao** | — | — | TCOND-06, -13, -14, -15, -17 |
| **Khả năng trung bình** | -22 | -01, -04, -05, -18, -19, -21 | -10, -11, -12, -16 |
| **Khả năng thấp** | -02, -23 | -03, -07, -08, -09, -20 | — |

Ô trên-phải và ô giữa-phải cho mức **cao** (9 condition); ô chính giữa và ô dưới-giữa cho mức **trung bình** (11); ô giữa-trái và ô dưới-trái cho mức **thấp** (3). Lý do đặt TCOND-VOUCHER-06 (múi giờ) ở khả năng cao: lỗi «dùng giờ máy chủ» là lỗi kinh điển, và máy chủ sản xuất gần như luôn chạy UTC. Lý do đặt -13, -14, -15 ở khả năng cao: chúng nằm ở chỗ nhiều quy tắc chạm nhau, nơi developer phải đọc ba đoạn đặc tả cùng lúc.

Mức ưu tiên ở đây là **sơ bộ**. Khi có số liệu thật (tỉ lệ chuyến dùng voucher, tỉ lệ chuyến mưa, lịch sử defect), nên đánh giá lại. Ghi rõ lý do của từng ô để lần sau người khác sửa được, thay vì chỉ có một cột «Cao».

### 4.8. Truy vết test basis → test condition

Ma trận xuôi lấy từ trường `basis` của bảng 4.3:

| Yêu cầu | Số condition | Test condition |
|---|---:|---|
| R-VOUCHER-01 | 4 | -04, -09, -20, -23 |
| R-VOUCHER-02 | 2 | -01, -02 |
| R-VOUCHER-03 | 1 | -03 |
| R-VOUCHER-04 | 3 | -04, -05, -06 |
| R-VOUCHER-05 | 3 | -07, -08, -09 |
| R-VOUCHER-06 | 5 | -10, -11, -12, -13, -14 |
| R-VOUCHER-07 | 6 | -16, -17, -18, -19, -20, -21 |
| R-VOUCHER-08 | 1 | -22 |
| R-FARE-06 · R-FARE-07 · R-FOOD-05 | 2 · 1 · 1 | -13, -15 · -15 · -14 |
| R-GEN-02 · R-QUOTE-03 · R-QUOTE-04 | 2 · 1 · 1 | -06, -19 · -17 · -21 |

Đọc theo chiều xuôi: không yêu cầu R-VOUCHER nào trống. Đọc theo chiều ngược: mọi mã trong cột phải có thật trong SPEC; một condition trích «R-VOUCHER-09» sẽ là dấu hiệu condition bịa hoặc SPEC đã đổi mà bảng chưa cập nhật. Cả hai chiều đều được lab kiểm tự động. Khi SPEC đổi, việc đầu tiên là chạy lại test truy vết: nó cho biết ngay condition nào trỏ vào mã đã mất và yêu cầu mới nào chưa có condition. Đây là bước đầu của phân tích ảnh hưởng mà [TCD-08](/articles/test-case-design-traceability-matrix) đi sâu.

## 5. Thực hành

### 5.1. Tổ chức lab

Lab: [`articles/test-case-design-test-analysis-test-condition`](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/test-case-design-test-analysis-test-condition). Không sửa `src/`.

| Tệp | Vai trò |
|---|---|
| `test-conditions.ts` | Kết quả test analysis viết thành **dữ liệu có kiểu**: 23 condition và 5 câu hỏi |
| `traceability.test.ts` | Kiểm chính kết quả phân tích (8 test) |
| `voucher-conditions.test.ts` | Bằng chứng chạy được cho 22 condition qua use case thật (24 test) |

Viết test condition thành dữ liệu thay vì một bảng trong tài liệu có hai lợi ích. Kết quả phân tích được **review như code**: thay đổi nằm trong pull request, có lịch sử, có người duyệt. Quan trọng hơn, các tính chất của bảng (đủ, đúng nguồn, đúng quy ước) được **máy kiểm** mỗi lần chạy test, không phụ thuộc trí nhớ của người review.

### 5.2. Test condition là dữ liệu

```ts
// test-conditions.ts (trích)
export interface TestCondition {
  id: string;            // TCOND-VOUCHER-nn
  statement: string;     // một câu khẳng định, không chứa giá trị cụ thể
  basis: RCode[];        // ≥ 1 mã yêu cầu trong SPEC.md
  priority: 'cao' | 'trung-binh' | 'thap';
  risk: string;          // rủi ro sản phẩm mà condition giúp giảm
  blockedBy?: string;    // Q-xx nếu đang chờ người viết đặc tả trả lời
}

{ id: 'TCOND-VOUCHER-13',
  statement: 'Với đặt xe và bưu kiện, A là số tiền sau bước 3 (đã gồm hệ số cao điểm và phụ phí trời mưa)',
  basis: ['R-FARE-06', 'R-VOUCHER-06'], priority: 'cao',
  risk: 'tính giảm trên cước cơ bản ⇒ lệch giá giữa báo giá và đối soát' },
```

Kiểu `RCode` giới hạn tiền tố hợp lệ ngay lúc biên dịch. Trường `risk` bắt người phân tích viết ra **vì sao** condition này đáng tồn tại. Nếu không viết được rủi ro nào, condition có thể thừa.

### 5.3. Kiểm chính kết quả phân tích

Tám test trong `traceability.test.ts` không chạm vào sản phẩm. Chúng kiểm **testware**:

| Test | Bắt được lỗi gì của bảng |
|---|---|
| SPEC có đúng 8 R-VOUCHER, mỗi mã có ≥ 1 condition | yêu cầu bị bỏ quên; SPEC thêm R-VOUCHER-09 mà bảng chưa cập nhật |
| ma trận xuôi: số condition mỗi yêu cầu | thay đổi `basis` không chủ ý |
| ngược: mọi mã trích dẫn có trong SPEC | mã gõ sai, mã đã bị xoá khỏi SPEC |
| quy ước mã liên tục 01…23, duy nhất | trùng mã, nhảy số |
| phân bố ưu tiên 9 / 11 / 3 | đổi ưu tiên mà không ai để ý |
| condition ưu tiên cao có bằng chứng chạy được | condition «cao» chưa có test nào |
| `blockedBy` trỏ tới câu hỏi có thật | condition bị chặn bởi câu hỏi đã bị xoá |
| statement không chứa giá trị cụ thể | condition trượt xuống mức quá mịn |

Test cuối là cách máy hoá quy tắc ở mục 4.2: nếu ai đó viết «GIAM20 giảm đúng 9.060 đ» vào bảng condition, test đỏ.

```ts
it('mức chi tiết: statement không chứa giá trị cụ thể', () => {
  const tooFine = TEST_CONDITIONS
    .filter((c) => /\d{1,3}\.\d{3}|GIAM20|FOOD15|HN10|XMAS50/.test(c.statement))
    .map((c) => c.id);
  expect(tooFine).toEqual([]);
});
```

### 5.4. Bằng chứng chạy được

Lab chưa phải bộ test case đầy đủ; đó là việc của test design. Nhưng mỗi condition ưu tiên cao cần **ít nhất một bằng chứng** rằng nó kiểm chứng được trên test object và hành vi hiện tại khớp SPEC. `voucher-conditions.test.ts` dựng hệ thống bằng `createMemorySystem`, đồng hồ giả `FixedClock` ở 09:00 giờ VN ngày 04/11/2026 (ngoài cao điểm), khách `C001`, rồi gọi `createQuote`, `createTrip`, `cancelByCustomer`. Mọi số tiền mong đợi được tính tay từ SPEC trước khi chạy:

| Condition | Dữ liệu | Tính từ SPEC | Mong đợi |
|---|---|---|---|
| -11 | GIAM20, xe máy 7,4 km, không mưa | A = 12.000 + 4.500 × 7,4 = 45.300; giảm 20% = 9.060; 36.240 ⇒ làm tròn | giảm 9.060 đ, tổng **37.000 đ** |
| -12 | GIAM20, ô tô 12 km | A = 25.000 + 11.000 × 12 = 157.000; 20% = 31.400 > 30.000 | giảm 30.000 đ, tổng **127.000 đ** |
| -13 | GIAM20, 7,4 km, SG đang mưa | A = 45.300 + 8.000 = 53.300; giảm 10.660; 42.640 ⇒ | tổng **43.000 đ** (cài sai trên cước cơ bản sẽ ra 45.000 đ) |
| -13 | GIAM20, 7,4 km, 11:30, mưa (R-FARE-09) | 45.300 × 1,25 + 8.000 = 64.625; giảm 12.925; 51.700 ⇒ | tổng **52.000 đ** |
| -14 | FOOD15, SG 3 km, 2 × 30.000 đ | giảm 15% × 60.000 = 9.000; phí giao 12.000 + 4.500 × 3 = 25.500 ⇒ 26.000 | tổng 60.000 − 9.000 + 26.000 = **77.000 đ** |
| -14 | FOOD15, 1 × 40.000 đ | tiền món 40.000 < 50.000 (dù cộng phí giao là 66.000) | `VOUCHER_MIN_AMOUNT` |
| -10 | HN10, HN, xe máy 6,2 / 6,3 km | A = 39.900 < 40.000 / A = 40.350; giảm 4.035; 36.315 ⇒ | `VOUCHER_MIN_AMOUNT` / tổng **37.000 đ** |
| -15 | GIAM20, xe máy 1 km | 16.500 − 3.300 = 13.200 < 15.000 | tổng **15.000 đ**, `discount` = 3.300 (Q-01) |
| -06 | GIAM20 lúc `2025-12-31T17:00:00Z` / `16:59:59Z` | 00:00 / 23:59:59 giờ VN | hợp lệ / `VOUCHER_EXPIRED` |
| -16…-18 | GIAM20 hai báo giá, một chuyến, huỷ | 1 lượt/ngày | báo giá 2 được; báo giá 3 và đặt từ báo giá 2 ⇒ `VOUCHER_USAGE_LIMIT`; huỷ xong báo giá lại được |
| -21 | GIAM20 báo giá 23:58 ngày 31/12/2030, đặt 00:01 hôm sau | báo giá hết hạn lúc 00:03 | chuyến tạo được, giá 37.000 đ; báo giá mới ⇒ `VOUCHER_EXPIRED` |
| -22 | HN10 ở SG 2 km / HN10 ở SG năm 2031 | NOT_APPLICABLE trước MIN_AMOUNT / EXPIRED trước NOT_APPLICABLE | đúng lỗi đứng trước |

Dòng -13 đáng chú ý vì nó được chọn để **phân biệt** cách hiểu đúng với cách hiểu sai. Với dữ liệu không mưa, ngoài cao điểm, tính giảm trên cước cơ bản hay sau bước 3 đều ra cùng số; test xanh nhưng không chứng minh gì. Chọn dữ liệu biết phân biệt các cách cài đặt sai là kỹ năng của test design, và nó bắt đầu từ việc condition đã nói rõ «sau bước 3».

Mỗi `it` ghi mã condition trong tên. Test truy vết ở mục 5.3 đọc chính tệp này để kiểm mọi condition ưu tiên cao đều có mặt:

```ts
it('TCOND-VOUCHER-16 · TCOND-VOUCHER-17 · TCOND-VOUCHER-18 · báo giá không tiêu lượt, ...', async () => {
  const q1 = ok(await quote(ride74('GIAM20')));
  const q2 = ok(await quote(ride74('GIAM20')));               // báo giá không tiêu lượt
  const trip = await createTrip(deps, C001, { quoteId: q1.id }); // tiêu lượt duy nhất trong ngày
  expect(await quote(ride74('GIAM20'))).toBe('VOUCHER_USAGE_LIMIT');                       // kiểm lần 1
  expect(await errorOf(createTrip(deps, C001, { quoteId: q2.id }))).toBe('VOUCHER_USAGE_LIMIT'); // kiểm lần 2
  await cancelByCustomer(deps, C001, trip.id);
  expect(typeof (await quote(ride74('GIAM20')))).toBe('object'); // lượt đã được trả lại
});
```

Test này phủ ba condition trong một kịch bản, vì ba condition là ba bước liên tiếp của cùng một chuỗi trạng thái. Đó là quyết định ở mức bằng chứng. Khi thiết kế test case chính thức ([TCD-02](/articles/test-case-design-giai-phau-test-case)), mỗi condition nên có test case riêng với một mục tiêu riêng.

### 5.5. Chạy

```bash
cd examples/toi-noi-testing
npx vitest run articles/test-case-design-test-analysis-test-condition
```

Kết quả thật:

```text
 ✓ articles/test-case-design-test-analysis-test-condition/traceability.test.ts (8 tests)
 ✓ articles/test-case-design-test-analysis-test-condition/voucher-conditions.test.ts (24 tests)

 Test Files  2 passed (2)
      Tests  32 passed (32)
```

Độ phủ test condition: **22/23**. TCOND-VOUCHER-23 không có bằng chứng và cũng không được tính là đạt: nó bị chặn bởi Q-04. Báo cáo tiến độ nên ghi đúng như vậy: «22/23 condition có bằng chứng; 1 condition chờ BA trả lời về cấu hình voucher».

Hai test của lab ghi lại hành vi hiện tại liên quan tới câu hỏi đang mở: Q-01 (`discount` = 3.300 nhưng `total` = 15.000) và Q-02 («GIAM 20» ⇒ `VOUCHER_NOT_FOUND`). Chúng phát hiện được nếu hành vi đổi, nhưng chưa phải kết quả mong đợi đã được xác nhận.

Bản `src/` hiện tại đạt mọi bằng chứng, đúng như mong đợi với mã đã qua nhiều vòng test của các bài trước. Giá trị của bài không nằm ở chỗ tìm ra bug trong code. Nó nằm ở chỗ biến «voucher đã được test» thành một khẳng định đo được, và tìm ra năm câu hỏi về đặc tả mà trước đó không ai đặt.

## 6. Góc nhìn nâng cao

**Test condition không chỉ đến từ đặc tả.** CTFL nói test basis là «toàn bộ kiến thức» dùng cho phân tích, không chỉ tài liệu yêu cầu. Nguồn bổ sung cho voucher gồm: danh sách rủi ro (mục 3.3), lịch sử defect (lỗi múi giờ là loại lỗi kinh điển của mọi quy tắc theo giờ, như R-FARE-04), kiến trúc (voucher được kiểm ở hai service khác nhau trong hệ thống đích), và kinh nghiệm (Q-03 về lạm dụng huỷ). Condition sinh từ nguồn ngoài SPEC vẫn phải có `basis`. Nếu không trỏ được tới mã R-…, ghi nguồn khác (ví dụ «RR-03» trong risk register) và đánh dấu là giả định để người review biết.

**Test analysis cho yêu cầu phi chức năng.** Voucher có một khía cạnh hiệu năng ẩn: R-VOUCHER-07 đếm chuyến trong ngày, nên mỗi báo giá có voucher là một truy vấn đếm. Ở mức hệ thống đích, condition kiểu «báo giá có voucher vẫn đạt R-NFR-03» là hợp lệ, dù bài này không đưa vào bảng vì lab không đo được ý nghĩa (SPEC mục 16). Condition không kiểm được ở môi trường hiện tại vẫn nên được ghi lại, kèm lý do, thay vì biến mất khỏi phân tích.

**Khi một condition nở thành nhiều condition.** TCOND-VOUCHER-17 nói «cả lúc báo giá lẫn lúc tạo chuyến». Khi thiết kế, ta có thể thấy đây thực chất là hai condition với hai rủi ro khác nhau (lần kiểm thứ hai mới chặn được kiểu lạm dụng «giữ sẵn nhiều báo giá»). Tách nó ra là bình thường; cập nhật bảng và để test truy vết bắt các chỗ cần sửa theo.

**Đừng để công cụ quyết định mức chi tiết.** Nhiều công cụ quản lý test chỉ có «requirement» và «test case», không có chỗ cho test condition. Đội nhảy thẳng từ yêu cầu xuống test case, và mất lớp «cái gì» ở giữa. Nếu công cụ không có trường cho condition, vẫn có thể giữ nó: một tệp dữ liệu như lab, một cột trong bảng tính, hoặc acceptance criteria trong user story.

| Sai lầm thường gặp | Cách làm đúng |
|---|---|
| bỏ qua test analysis, viết test case ngay | liệt kê condition trước, dù chỉ 10 phút; đó là mẫu số của độ phủ |
| condition là tiêu đề chung chung («Voucher») | câu khẳng định kiểm chứng được, nói cả kết quả |
| condition chứa giá trị cụ thể | giá trị để cho test design; condition sống lâu hơn dữ liệu mẫu |
| đọc từng quy tắc riêng lẻ | đọc thêm lăng kính «tương tác giữa quy tắc»; nhiều rủi ro nằm ở chỗ nối |
| tự chọn một cách hiểu khi đặc tả mơ hồ | ghi câu hỏi Q-xx, chờ trả lời; không viết kết quả mong đợi theo phỏng đoán |
| condition không kiểm được thì xoá | giữ lại, đánh dấu bị chặn, không tính vào độ phủ |
| ưu tiên «Cao» cho mọi thứ | ma trận khả năng × tác động, ghi lý do từng ô |
| không ghi nguồn, «sẽ bổ sung sau» | `basis` bắt buộc ngay lúc tạo; kiểm hai chiều tự động |

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1.** Phân tích R-QUOTE-01 (báo giá có thời hạn 5 phút, «hiện tại < expiresAt» là còn hiệu lực) thành test condition theo quy tắc mục 4.2. Ghi nguồn, ưu tiên và ít nhất một câu hỏi gửi người viết đặc tả.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Một bộ hợp lý: <b>TCOND-QUOTE-01</b> «báo giá thành công được lưu kèm quoteId, createdAt, expiresAt = createdAt + 5 phút, pricingVersion» (R-QUOTE-01, R-QUOTE-02 · trung bình); <b>TCOND-QUOTE-02</b> «tạo chuyến khi thời điểm hiện tại nhỏ hơn expiresAt được chấp nhận» (R-QUOTE-01, R-QUOTE-03 · cao); <b>TCOND-QUOTE-03</b> «tạo chuyến khi thời điểm hiện tại bằng hoặc sau expiresAt bị từ chối với QUOTE_EXPIRED» (R-QUOTE-01, R-QUOTE-03 · cao, vì đúng biên là chỗ dễ sai &lt; và ≤). Không ghi «5 phút 0 giây» hay «09:05» vào condition; đó là coverage item của BVA khi thiết kế. Câu hỏi có thể đặt: «đồng hồ nào là chuẩn khi Booking và Pricing chạy trên hai máy trong hệ thống đích?». SPEC chỉ nói «đồng hồ của hệ thống» (R-GEN-02), và lab chạy một tiến trình nên không gặp vấn đề này.</p>
</details>

**Bài tập 2.** Một đồng nghiệp nộp ba condition: (a) «Voucher hoạt động theo SPEC»; (b) «GIAM20 cho xe máy 7,4 km lúc 09:00 giảm 9.060 đ»; (c) «Khi khách nhập voucher, hệ thống chuẩn hoá mã, kiểm tồn tại, kiểm hạn và tính tiền giảm». Nhận xét từng condition và viết lại.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(a) quá thô: không kiểm chứng được, không có lý do cụ thể để đỏ. Thay bằng bảng 23 condition, hoặc ít nhất sáu condition, mỗi mảng ở mục 4.1 một condition. (b) quá mịn: đó là test case cụ thể. Condition tương ứng là TCOND-VOUCHER-11 «khi chưa chạm trần, tiền giảm bằng percent% × A», còn bộ dữ liệu GIAM20 7,4 km để test design chọn. (c) nhiều khía cạnh trong một câu: bốn bước có thể sai độc lập và có rủi ro khác mức. Tách thành TCOND-VOUCHER-01, -03, -05 và -11; nếu muốn kiểm thứ tự giữa chúng thì thêm -22.</p>
</details>

**Câu 1 (K2 · FL-1.4.1).** Việc nào **thuộc test analysis** theo CTFL v4.0.1?

- a) Xác định coverage item và yêu cầu về test data.
- b) Đánh giá test basis để tìm defect và đánh giá khả năng kiểm thử.
- c) Gom test procedure thành test suite và lập test execution schedule.
- d) So sánh kết quả thực tế với kết quả mong đợi.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — coverage item và yêu cầu test data thuộc test design. <b>b đúng</b> — test analysis đánh giá test basis và test object để nhận diện defect và testability. <b>c sai</b> — thuộc test implementation. <b>d sai</b> — thuộc test execution.</p>
</details>

**Câu 2 (K2 · FL-1.4.1).** Câu nào mô tả đúng nhất sự khác nhau giữa test analysis và test design?

- a) Test analysis do developer làm, test design do tester làm.
- b) Test analysis trả lời «kiểm thử cái gì», test design trả lời «kiểm thử thế nào».
- c) Test analysis chỉ dùng cho dự án Waterfall.
- d) Test design xảy ra trước test analysis để biết cần phân tích gì.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — CTFL không gắn hoạt động với chức danh. <b>b đúng</b> — đúng cách syllabus tóm tắt. <b>c sai</b> — mọi SDLC đều có các hoạt động này, khác nhau ở hình thức. <b>d sai</b> — design dùng đầu ra của analysis, dù các hoạt động có thể lặp.</p>
</details>

**Câu 3 (K2 · FL-1.4.3).** Testware nào là **sản phẩm của test analysis**?

- a) Test condition đã được ưu tiên và defect report về defect trong test basis.
- b) Test case đã được ưu tiên và test charter.
- c) Test log và defect report từ lần chạy test.
- d) Test plan và risk register.

<details>
<summary><b>Đáp án: a</b></summary>
<p><b>a đúng</b> — CTFL 1.4.3 liệt kê đúng hai loại này cho test analysis. <b>b sai</b> — thuộc test design. <b>c sai</b> — thuộc test execution. <b>d sai</b> — thuộc test planning.</p>
</details>

**Câu 4 (K2 · FL-1.4.4).** Đội đổi R-VOUCHER-06 (thêm điều kiện: tiền giảm tối thiểu 1.000 đ). Nhờ truy vết R-code → test condition, đội làm được gì ngay?

- a) Biết ngay năm condition dựa trên R-VOUCHER-06 cần xem lại, rồi lần xuống các test case bên dưới.
- b) Không cần chạy lại test, vì truy vết đã bảo đảm test còn đúng.
- c) Xoá toàn bộ condition cũ và phân tích lại từ đầu.
- d) Truy vết chỉ có ích cho kiểm toán, không giúp gì trong tình huống này.

<details>
<summary><b>Đáp án: a</b></summary>
<p><b>a đúng</b> — phân tích ảnh hưởng là một giá trị của truy vết mà CTFL 1.4.4 nêu. <b>b sai</b> — truy vết chỉ ra chỗ cần sửa, không thay việc chạy lại. <b>c sai</b> — lãng phí; truy vết cho phép sửa đúng chỗ. <b>d sai</b> — kiểm toán chỉ là một trong các giá trị.</p>
</details>

**Câu 5 (K2).** Condition nào được viết ở **mức chi tiết phù hợp** cho test analysis?

- a) «Voucher hoạt động đúng.»
- b) «Tiền giảm không vượt maxDiscount.»
- c) «Ô tô 12 km với GIAM20 trả 127.000 đ.»
- d) «Kiểm voucher bằng BVA.»

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a</b> quá thô, không kiểm chứng được. <b>b đúng</b> — một khía cạnh, kiểm chứng được, không chứa giá trị cụ thể. <b>c</b> là test case cụ thể (quá mịn). <b>d</b> là quyết định kỹ thuật của test design, không phải điều phải đúng của test object.</p>
</details>

**Câu 6 (K2).** Trong lúc phân tích, tester thấy SPEC không nói «tiền giảm» hiển thị cho khách là số nào khi cước tối thiểu nâng giá. Tester nên làm gì?

- a) Chọn cách hiểu hợp lý nhất và viết kết quả mong đợi theo cách hiểu đó.
- b) Bỏ condition liên quan vì đặc tả không rõ.
- c) Ghi câu hỏi gửi người viết đặc tả; nếu xác nhận là thiếu sót chưa sửa ngay thì lập defect report về test basis.
- d) Chờ tới test execution, nếu test đỏ thì báo lỗi.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a sai</b> — phỏng đoán biến chỗ mơ hồ thành «đúng» một cách lặng lẽ. <b>b sai</b> — mất khía cạnh rủi ro cao. <b>c đúng</b> — đây là việc «đánh giá test basis để tìm defect» của test analysis; defect report về test basis là testware CTFL 1.4.3 liệt kê. <b>d sai</b> — phát hiện muộn và đắt hơn, và test có thể xanh với một cách hiểu sai.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB® Certified Tester Foundation Level Syllabus **v4.0.1** (2024): mục 1.4.1 Test Activities and Tasks, 1.4.3 Testware, 1.4.4 Traceability between the Test Basis and Testware · FL-1.4.1 (K2), FL-1.4.3 (K2), FL-1.4.4 (K2). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB® Glossary (tra *test analysis*, *test basis*, *test condition*, *coverage item*, *testability*): https://glossary.istqb.org/
- ISO/IEC/IEEE 29119-2 — Test processes (quy trình thiết kế test từ test condition tới test case); ISO/IEC/IEEE 29119-3 — Test documentation.
- Lab Tới Nơi: `SPEC.md` (R-VOUCHER, R-FARE-03/06/07/09, R-FOOD-04/05, R-GEN-02, Phụ lục A) và thư mục bài: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/test-case-design-test-analysis-test-condition
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-007 · Quy trình kiểm thử](/articles/quy-trinh-kiem-thu-test-process) · [TEST-010 · Testware](/articles/testware-san-pham-kiem-thu) · [Static testing là gì](/articles/static-testing-la-gi) · [TCD-06 · Phối hợp kỹ thuật](/articles/test-case-design-phoi-hop-ky-thuat) · [TCD-08 · Ma trận truy vết](/articles/test-case-design-traceability-matrix)

Bài tiếp theo: [TCD-02 · Giải phẫu một test case](/articles/test-case-design-giai-phau-test-case).
