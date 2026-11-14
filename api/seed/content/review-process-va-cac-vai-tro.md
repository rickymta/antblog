---
slug: review-process-va-cac-vai-tro
title: "Review Process và các vai trò: năm hoạt động, sáu vai trò, chín yếu tố thành công"
summary: "Năm hoạt động của review process theo CTFL v4.0.1 (ISO/IEC 20246), trách nhiệm của manager, author, moderator, scribe, reviewer, review leader và các yếu tố thành công, áp vào buổi review đặc tả thanh toán VíXanh."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-027, static-testing, review, review-process, review-roles, iso-20246]
language: vi
---

Đội Tới Nơi «review» rất nhiều: góp ý trong tài liệu, duyệt pull request, họp đọc đặc tả. Nhưng hỏi «buổi review đặc tả thanh toán hôm qua kết thúc khi nào, ai quyết định một góp ý là lỗi hay không, ai ghi lại» thì thường không ai trả lời được. Bài này trình bày khung chung mà CTFL v4.0.1 mượn từ ISO/IEC 20246: **năm hoạt động** của một review, **sáu vai trò** chính và **các yếu tố** làm review có ích thay vì tốn thời gian. Các loại review cụ thể (informal, walkthrough, technical review, inspection) là các bài tiếp theo; chúng đều là những cách «may đo» khung này.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-027</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S03 · Static Testing &amp; Review Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Review</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/static-testing-la-gi">TEST-021 · Static Testing là gì?</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lập kế hoạch review đặc tả thanh toán VíXanh (SPEC R-PAY-01…06): phân vai, tiêu chí vào/ra, lịch 5 hoạt động và mẫu ghi nhận phát hiện — lab (đọc SPEC)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 3 · mục 3.2.2, 3.2.3, 3.2.5 · FL-3.2.2 (K2), FL-3.2.3 (K1), FL-3.2.5 (K1)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Một review không có quy trình thường hỏng theo ba kiểu: **không ai chuẩn bị** (cả đội mở tài liệu lần đầu trong phòng họp, góp ý câu chữ, bỏ qua lỗi logic); **không ai quyết** (mười góp ý, ba cái mâu thuẫn, tác giả không biết sửa cái nào); **không biết khi nào xong** («xong» là hết giờ họp chứ không phải tài liệu đã đủ tốt).

**Review process** (quy trình review) giải ba vấn đề đó bằng cách tách việc thành các hoạt động có đầu ra rõ, gán trách nhiệm cho từng **vai trò** (role) và đặt **exit criteria** (tiêu chí ra) đo được. Dùng khung này khi: tổ chức review một tài liệu quan trọng (đặc tả tiền, hợp đồng API), khi chọn mức hình thức cho một loại review, và khi đội cần lý do vì sao review của mình không hiệu quả.

## 2. Lý thuyết theo ISTQB

### 2.1. Một quy trình chung, may đo theo tình huống

CTFL v4.0.1 (mục 3.2.2) lấy **quy trình review chung** của chuẩn ISO/IEC 20246. Đây là khung «có cấu trúc nhưng linh hoạt»: một review cụ thể được may đo (tailor) từ đó. Review càng hình thức thì càng dùng nhiều nhiệm vụ trong mỗi hoạt động; review không chính thức có thể chỉ giữ dáng của vài hoạt động.

Hai điểm syllabus nhấn mạnh:

- Một work product lớn thường **không review hết trong một lần**. Quy trình có thể được chạy **nhiều lần**, mỗi lần một phần.
- Kết quả của một lần review có thể cần một **follow-up review** (review tiếp theo) để hoàn tất các hành động đã quyết.

### 2.2. Năm hoạt động (FL-3.2.2)

| Hoạt động | Mục tiêu | Đầu ra chính |
|---|---|---|
| **Planning** (lập kế hoạch) | Xác định **phạm vi** của review: mục đích, work product được review, đặc tính chất lượng cần đánh giá, vùng cần tập trung, **exit criteria**, thông tin hỗ trợ (vd tiêu chuẩn), nỗ lực và khung thời gian | Kế hoạch review |
| **Review initiation** (khởi động review) | Bảo đảm mọi người, mọi thứ sẵn sàng: mỗi người tham gia **truy cập được** work product, **hiểu vai trò và trách nhiệm** của mình, nhận đủ thứ cần để review | Gói tài liệu review đã gửi, vai trò đã rõ |
| **Individual review** (review cá nhân) | Mỗi reviewer **tự** đánh giá chất lượng work product bằng một hoặc nhiều **review technique** (vd checklist-based, scenario-based) và **ghi lại** mọi anomaly, đề xuất, câu hỏi | Danh sách anomaly / đề xuất / câu hỏi của từng người |
| **Communication and analysis** (trao đổi và phân tích) | Phân tích, thảo luận mọi anomaly — vì **anomaly chưa chắc là defect**. Với từng cái: quyết định **trạng thái, người chịu trách nhiệm, hành động cần làm**. Thường diễn ra trong **review meeting**, nơi cũng quyết định mức chất lượng của work product và việc follow-up | Nhật ký đã có quyết định; đánh giá chất lượng work product |
| **Fixing and reporting** (sửa và báo cáo) | Mỗi defect có một **defect report** để theo dõi việc sửa. Khi đạt exit criteria, work product được **chấp nhận**. Kết quả review được báo cáo | Defect report, work product đã sửa, báo cáo review |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 306" role="img" aria-label="Năm hoạt động của review process nối tiếp nhau: Planning, Review initiation, Individual review, Communication and analysis, Fixing and reporting. Dưới mỗi hoạt động là đầu ra chính và vai trò chính. Một mũi tên nét đứt quay từ Fixing and reporting về Planning: follow-up review hoặc phần tiếp theo của work product lớn.">
<defs>
<marker id="t027-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="t027-back" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<text x="460" y="30" text-anchor="middle" class="t-b">Review process chung (ISO/IEC 20246 · CTFL v4.0.1 mục 3.2.2)</text>
<rect x="20" y="50" width="160" height="70" rx="8" class="f-pk-soft s-pk"/>
<text x="100" y="78" text-anchor="middle" class="t-b">1 · Planning</text>
<text x="100" y="98" text-anchor="middle" class="t-small">lập kế hoạch</text>
<rect x="200" y="50" width="160" height="70" rx="8" class="f-pk-soft s-pk"/>
<text x="280" y="78" text-anchor="middle" class="t-b">2 · Review initiation</text>
<text x="280" y="98" text-anchor="middle" class="t-small">khởi động</text>
<rect x="380" y="50" width="160" height="70" rx="8" class="f-pk-soft s-pk"/>
<text x="460" y="78" text-anchor="middle" class="t-b">3 · Individual review</text>
<text x="460" y="98" text-anchor="middle" class="t-small">review cá nhân</text>
<rect x="560" y="50" width="160" height="70" rx="8" class="f-pk-soft s-pk"/>
<text x="640" y="72" text-anchor="middle" class="t-b">4 · Communication</text>
<text x="640" y="88" text-anchor="middle" class="t-b">and analysis</text>
<text x="640" y="106" text-anchor="middle" class="t-small">trao đổi, phân tích</text>
<rect x="740" y="50" width="160" height="70" rx="8" class="f-pk-soft s-pk"/>
<text x="820" y="72" text-anchor="middle" class="t-b">5 · Fixing</text>
<text x="820" y="88" text-anchor="middle" class="t-b">and reporting</text>
<text x="820" y="106" text-anchor="middle" class="t-small">sửa, báo cáo</text>
<line x1="180" y1="85" x2="198" y2="85" class="s-pk" stroke-width="1.6" marker-end="url(#t027-arrow)"/>
<line x1="360" y1="85" x2="378" y2="85" class="s-pk" stroke-width="1.6" marker-end="url(#t027-arrow)"/>
<line x1="540" y1="85" x2="558" y2="85" class="s-pk" stroke-width="1.6" marker-end="url(#t027-arrow)"/>
<line x1="720" y1="85" x2="738" y2="85" class="s-pk" stroke-width="1.6" marker-end="url(#t027-arrow)"/>
<text x="100" y="146" text-anchor="middle" class="t-10">phạm vi, mục đích,</text>
<text x="100" y="162" text-anchor="middle" class="t-10">exit criteria, lịch</text>
<text x="280" y="146" text-anchor="middle" class="t-10">ai cũng có tài liệu,</text>
<text x="280" y="162" text-anchor="middle" class="t-10">hiểu vai trò của mình</text>
<text x="460" y="146" text-anchor="middle" class="t-10">mỗi người tự đọc, ghi</text>
<text x="460" y="162" text-anchor="middle" class="t-10">anomaly, câu hỏi, đề xuất</text>
<text x="640" y="146" text-anchor="middle" class="t-10">trạng thái, chủ sở hữu,</text>
<text x="640" y="162" text-anchor="middle" class="t-10">hành động cho từng mục</text>
<text x="820" y="146" text-anchor="middle" class="t-10">defect report, sửa,</text>
<text x="820" y="162" text-anchor="middle" class="t-10">đạt exit ⇒ chấp nhận</text>
<line x1="20" y1="182" x2="900" y2="182" class="s-line"/>
<text x="100" y="206" text-anchor="middle" class="t-small">Manager</text>
<text x="100" y="222" text-anchor="middle" class="t-small">Review leader</text>
<text x="280" y="206" text-anchor="middle" class="t-small">Review leader</text>
<text x="280" y="222" text-anchor="middle" class="t-small">Author</text>
<text x="460" y="206" text-anchor="middle" class="t-small">Reviewer</text>
<text x="640" y="206" text-anchor="middle" class="t-small">Moderator, Scribe</text>
<text x="640" y="222" text-anchor="middle" class="t-small">Author, Reviewer</text>
<text x="820" y="206" text-anchor="middle" class="t-small">Author</text>
<text x="820" y="222" text-anchor="middle" class="t-small">Review leader</text>
<path d="M820,240 L820,272 L100,272 L100,244" class="s-warn f-none dash" stroke-width="1.6" marker-end="url(#t027-back)"/>
<text x="460" y="294" text-anchor="middle" class="t-small">follow-up review, hoặc chạy lại cho phần tiếp theo của một work product lớn</text>
</svg>
</div>
<figcaption><b>Hình 1 · Năm hoạt động của review process.</b> Review càng hình thức, mỗi hoạt động càng nhiều nhiệm vụ bắt buộc; review không chính thức chỉ giữ dáng của vài hoạt động. Phần gán vai trò ở hàng dưới là cách làm phổ biến, CTFL không gán vai trò cho từng hoạt động.</figcaption>
</figure>

### 2.3. Sáu vai trò chính (FL-3.2.3)

| Vai trò | Còn gọi là | Trách nhiệm theo CTFL v4.0.1 |
|---|---|---|
| **Manager** (người quản lý) | — | Quyết định **cái gì** được review; cấp **nguồn lực** (người, thời gian) |
| **Author** (tác giả) | — | **Tạo ra** và **sửa** work product được review |
| **Moderator** (người điều phối) | facilitator | Bảo đảm **họp review chạy hiệu quả**: hoà giải, quản lý thời gian, giữ **môi trường an toàn** để ai cũng nói thẳng được |
| **Scribe** (thư ký) | recorder | **Gom anomaly** từ các reviewer, ghi thông tin review: **quyết định** và **anomaly mới** phát hiện trong họp |
| **Reviewer** (người review) | — | **Thực hiện review**. Có thể là người trong dự án, **chuyên gia nghiệp vụ** (subject matter expert) hoặc bất kỳ bên liên quan nào |
| **Review leader** (trưởng nhóm review) | — | Chịu **trách nhiệm chung** cho review: quyết định **ai tham gia**, tổ chức **khi nào, ở đâu** |

Ghi nhớ thêm:

- Một bên liên quan có thể giữ **nhiều vai trò**. Ở đội nhỏ, review leader thường kiêm moderator.
- ISO/IEC 20246 mô tả thêm các vai trò chi tiết hơn; CTFL chỉ yêu cầu nắm sáu vai trò chính.
- Ràng buộc nổi tiếng nhất nằm ở mục 3.2.4: trong **inspection**, author **không được** làm review leader hay scribe (bài [TEST-026](/articles/inspection-review-hinh-thuc)).
- **Moderator khác review leader**: moderator lo **cuộc họp** diễn ra tốt; review leader lo **cả cuộc review** (người, lịch, kết quả).

### 2.4. Yếu tố làm review thành công (FL-3.2.5)

CTFL v4.0.1 liệt kê chín yếu tố:

1. **Mục tiêu rõ và exit criteria đo được.** Đánh giá người tham gia **không bao giờ** được là mục tiêu.
2. **Chọn đúng loại review** cho mục tiêu, loại work product, người tham gia, nhu cầu và bối cảnh dự án.
3. **Review từng phần nhỏ**, để reviewer không mất tập trung khi review cá nhân và khi họp.
4. **Đưa phản hồi** từ review tới stakeholder và author để họ cải thiện sản phẩm và cách làm (liên hệ mục 3.2.1 — lợi ích của phản hồi sớm, sẽ bàn ở [TEST-030](/articles/shift-left-testing)).
5. **Đủ thời gian chuẩn bị** cho người tham gia.
6. **Ban quản lý ủng hộ** quy trình review.
7. **Review là một phần văn hoá** tổ chức, để thúc đẩy học hỏi và cải tiến quy trình.
8. **Đào tạo đủ** để mọi người biết làm tròn vai trò của mình.
9. **Điều phối cuộc họp** (facilitating meetings).

### 2.5. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa tiếng Việt | Ghi chú |
|---|---|---|
| review process | quy trình review | Khung chung theo ISO/IEC 20246 |
| anomaly | điều bất thường | Mọi chỗ lệch kỳ vọng; chưa chắc là defect |
| defect | khuyết tật | Anomaly đã được xác nhận là lỗi (xem [TEST-004](/articles/error-defect-failure-root-cause)) |
| exit criteria | tiêu chí ra / tiêu chí kết thúc | Điều kiện để chấp nhận work product |
| moderator (facilitator) · scribe (recorder) · review leader | người điều phối · thư ký · trưởng nhóm review | Xem bảng 2.3 |
| subject matter expert (SME) | chuyên gia nghiệp vụ | Một kiểu reviewer |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài chữ của CTFL v4.0.1</span>
<p><b>Entry criteria</b> (tiêu chí vào) của review — vd «tài liệu có số phiên bản, tác giả đã tự soát» — là thực hành phổ biến của các review hình thức (ngoài phạm vi chữ của v4.0.1); mục 3.2.2 của v4.0.1 chỉ nêu exit criteria trong planning và việc «mọi người sẵn sàng» ở initiation. Bài này dùng tiêu chí vào như một công cụ của initiation, không phải mục ghi nhớ cho kỳ thi.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Squad Payment chuẩn bị tách «Payment adapter» của bản MVP thành **Payment & Wallet Service** thật (kiến trúc §4.9, §18). Trước khi viết code mới, Engineering Manager muốn review đặc tả thanh toán hiện tại — mục 11 của `SPEC.md` (R-PAY-01…06) — vì mọi lỗi ở đây là lỗi tiền.

### 3.2. Business rules trong phạm vi

- **R-PAY-01/02** — phương thức `cash` (mặc định) hoặc `vixanh`; `paymentStatus` ∈ {`cash`, `pending`, `paid`, `failed`, `none`}.
- **R-PAY-03** — chuyến `vixanh` hoàn thành ⇒ thu đúng số tiền `fare`, khoá idempotency `trip:<id>:fare`.
- **R-PAY-04** — hết giờ 3 giây/lần; hết giờ hoặc lỗi tạm thời ⇒ thử lại tối đa 2 lần cùng khoá; bị từ chối ⇒ không thử lại.
- **R-PAY-05** — phí huỷ > 0: `vixanh` thu qua ví với khoá `trip:<id>:cancel-fee`, `cash` chỉ ghi nhận; phí 0 ⇒ `none`.
- **R-PAY-06** — thu tiền thất bại không chặn hoàn thành/huỷ: `paymentStatus = failed`, API trả 200.

Đặc tả này dính tới các mục khác: R-CANCEL-07 (thu phí huỷ), R-API-07/08 (trường `fare`, `total`), R-QUOTE-04 (ảnh chụp giá), R-STATE-08 (cột Payment), R-NFR-05, R-TEST-03 (ví giả có chế độ `lost_response`).

### 3.3. System context

Kiến trúc đích: Booking/Trip gọi Payment tạo payment intent có idempotency, Payment gọi VíXanh, kết quả chắc chắn đến bằng **webhook có chữ ký (có thể trễ, trùng)** và có đối soát định kỳ (§8.3). Lab chỉ có adapter gọi thẳng ví giả, **chưa có webhook, ledger, đối soát** (R-ARCH-02) — reviewer cần biết để không ghi «thiếu webhook» thành defect của đặc tả lab.

### 3.4. Rủi ro nếu đặc tả sai mà không ai thấy

- Hiểu sai **số tiền phải thu** ⇒ thu thiếu (mất doanh thu) hoặc thu thừa (khiếu nại, hoàn tiền).
- Thử lại **không cùng khoá idempotency** ⇒ trừ tiền hai lần — rủi ro kiến trúc nêu đích danh.
- Nghĩa của `paymentStatus` mơ hồ ⇒ đối soát và chăm sóc khách hàng xử lý sai chuyến `failed`.
- Không mời Booking và Tài chính ⇒ đặc tả đúng với Payment nhưng sai với bên gọi và bên đối soát.

## 4. Phân tích kỹ thuật

### 4.1. Chọn mức hình thức

Đặc tả dính tới tiền, nhiều bên dùng, cần dấu vết quyết định ⇒ review **có cấu trúc**: individual review, họp, nhật ký, exit criteria — dùng **gần đủ** nhiệm vụ của năm hoạt động. Chưa cần thu metric để cải tiến quy trình như inspection ([TEST-026](/articles/inspection-review-hinh-thuc)).

**Chia nhỏ.** `SPEC.md` dài khoảng 38 KB; review cả bản một buổi vi phạm yếu tố «review từng phần nhỏ». Lần này chỉ lấy mục 11 (≈ 1 trang) cộng các đoạn liên quan; các mục khác là những lần chạy sau của cùng quy trình.

### 4.2. Chọn reviewer theo góc nhìn

Mỗi reviewer được giao một **góc nhìn** để giảm trùng lặp và tăng độ phủ:

| Góc nhìn | Câu hỏi dẫn dắt |
|---|---|
| Bên cài đặt (Payment) | Có cài đặt được đúng một cách không? Lỗi nào là «tạm thời»? |
| Bên gọi (Booking) | Trường nào mình phải truyền, đọc trường nào? Có khớp R-API không? |
| Vận hành (SRE) | Hết giờ, thử lại ảnh hưởng thời gian phản hồi và giám sát thế nào? |
| Đối soát (Tài chính) | Từ `paymentStatus` có biết khoản nào đã thu, chưa thu không? |

### 4.3. Review technique cho individual review

- **Checklist-based**: 8 câu hỏi về tiền và gọi bên thứ ba (số tiền lấy từ đâu, đơn vị, khoá idempotency, hết giờ, thử lại, trạng thái cuối, trường hợp số 0, trường hợp huỷ). Cách xây checklist tốt: [TEST-028](/articles/checklist-based-review).
- **Scenario-based**: đi qua 3 kịch bản — (S1) ví trừ tiền nhưng mất phản hồi (`lost_response`); (S2) khách huỷ chuyến `vixanh` có phí; (S3) đơn đồ ăn trả qua ví.

## 5. Thực hành

Bộ tài liệu đầy đủ (kế hoạch + nhật ký mẫu) nằm trong lab: `examples/toi-noi-testing/articles/review-process-va-cac-vai-tro/` — `ke-hoach-review-r-pay.md` và `nhat-ky-review-r-pay.md`. Phần dưới là bản rút gọn kèm lời giải.

### 5.1. Kế hoạch review (planning)

| Mục | Nội dung |
|---|---|
| Mục đích | Tìm anomaly trong R-PAY trước khi xây Payment & Wallet Service; tạo sự hiểu biết chung giữa Payment, Booking, Vận hành, Tài chính |
| Work product | `SPEC.md` v1.1 mục 11 (R-PAY-01…06) + R-CANCEL-07, R-API-07/08, R-QUOTE-04, R-STATE-08 (cột Payment), R-NFR-05, R-TEST-03 |
| Đặc tính chất lượng | Đúng (correctness), đủ (completeness), nhất quán với R-API/R-CANCEL (consistency), kiểm thử được (testability), không mơ hồ |
| Vùng tập trung | Số tiền thu, khoá idempotency, hết giờ/thử lại, nghĩa của từng `paymentStatus`, chuyến bị huỷ |
| Thông tin hỗ trợ | Kiến trúc §4.9, §8.3; R-ARCH-02 (danh sách ngoài phạm vi lab); checklist 8 mục; 3 kịch bản S1–S3 |
| Tiêu chí vào (thực hành) | Đặc tả có số phiên bản; tác giả đã tự soát; mọi mã R-… dẫn chiếu tồn tại |
| **Exit criteria** | (1) 100% mục trong nhật ký có trạng thái và người chịu trách nhiệm; (2) 0 defect mức Major còn mở; (3) mọi câu hỏi đã trả lời hoặc thành việc có hạn chót; (4) có Major ⇒ review leader và 1 reviewer kiểm bản sửa (follow-up) |
| Nỗ lực, thời gian (giả định) | Individual review 45 phút × 4 người; họp 45 phút × 7 người; sửa ≈ 2 giờ ⇒ ≈ 10 giờ công; trong 4 ngày làm việc |

### 5.2. Phân vai

| Vai trò | Người ở Tới Nơi (giả định) | Việc cụ thể lần này |
|---|---|---|
| Manager | Engineering Manager squad Payment | Chọn R-PAY là work product ưu tiên; giữ ≈ 10 giờ công trong sprint |
| Author | BA viết mục R-PAY | Trả lời câu hỏi, sửa đặc tả thành bản mới |
| Review leader | QA Lead | Chọn reviewer theo 4 góc nhìn, xếp lịch, kiểm exit criteria, ký báo cáo |
| Moderator | Scrum Master squad Booking (người trung lập) | Điều phối họp 45 phút, giữ giờ, chặn tranh luận lan man, giữ không khí an toàn |
| Scribe | Tester squad Payment | Gộp nhật ký trước họp, ghi quyết định và anomaly mới trong họp |
| Reviewer | Dev Payment · Dev Booking · SRE · Chuyên viên đối soát (SME) | Mỗi người review theo góc nhìn được giao (mục 4.2) |

### 5.3. Lịch năm hoạt động

| # | Hoạt động | Khi | Đầu ra |
|---|---|---|---|
| 1 | Planning | Thứ Hai 9:00–9:30 | Bảng 5.1, 5.2 |
| 2 | Review initiation | Thứ Hai 14:00–14:15 | Gói review (SPEC v1.1, checklist, S1–S3, mẫu nhật ký); mỗi người xác nhận đã mở được và hiểu góc nhìn |
| 3 | Individual review | Thứ Hai chiều → Thứ Ba 12:00 | 4 reviewer ghi vào nhật ký chung |
| 4 | Communication and analysis | Thứ Ba 15:00–15:45 | Mỗi dòng nhật ký có quyết định; kết luận «chấp nhận sau khi sửa» |
| 5 | Fixing and reporting | Thứ Tư → Thứ Năm | Defect report cho mục Major, đặc tả bản sửa, follow-up, báo cáo 1 trang |

### 5.4. Nhật ký phát hiện mẫu (sau họp)

Các cột tối thiểu: ID · vị trí · loại reviewer ghi (defect nghi ngờ / câu hỏi / đề xuất) · mô tả · người nêu · **quyết định trong họp** · mức · người chịu trách nhiệm.

| ID | Vị trí | Mô tả (ghi lúc individual review) | Người nêu | Quyết định | Mức | Phụ trách |
|---|---|---|---|---|---|---|
| A-01 | R-PAY-03 | «Thu đúng số tiền `fare`». Nhưng R-API-07 định nghĩa `quote.fare` là **bảng chi tiết**, và với `food` thì `fare.total` chỉ là **phí giao**. Đọc theo R-API có thể chỉ thu phí giao. | Dev Booking | **Defect.** Sửa thành «số tiền khách phải trả = `quote.total`, lưu ở trường `fare` của chuyến (R-QUOTE-04)» và định nghĩa trường đó | Major | BA |
| A-02 | R-PAY-02 ↔ R-PAY-05 | `cash` vừa nghĩa «thu tiền mặt, hệ thống không gọi ví» vừa dùng cho phí huỷ tiền mặt **chưa thu**. Đối soát không biết khoản nào đã thu. | Đối soát | **Defect** về câu chữ: nói rõ `cash` = «khoản xử lý ngoài hệ thống, không có nghĩa đã thu». Tách trạng thái nợ ⇒ backlog hệ đích | Minor | BA |
| A-03 | R-PAY-04 | «Lỗi tạm thời (cổng không trả lời được)» trùng nghĩa với hết giờ. Cổng trả lỗi rõ ràng (vd 503) có thử lại không? R-TEST-03 coi `error` là lỗi tạm thời. | Dev Payment | **Defect**: liệt kê lỗi tạm thời gồm hết giờ, không kết nối được, lỗi tạm thời cổng trả về | Minor | BA |
| A-04 | R-PAY-04 | Không nói chờ bao lâu giữa các lần thử. Tệ nhất lệnh `/complete` chờ 3 × 3 = 9 giây — app tài xế có tự hết giờ trước đó không? | SRE | **Câu hỏi ⇒ việc mới**: bổ sung thời gian xử lý tối đa của `/complete` và lệnh huỷ; hạn trước sprint sau | — | PO, SRE |
| A-05 | R-PAY-03 | Không nói gì khi số tiền bằng 0. | Dev Payment | **Bác bỏ — không phải defect.** R-FARE-07 (cước tối thiểu) và R-FOOD-03/04 bảo đảm số tiền luôn > 0 | — | — |
| A-06 | R-PAY-05 ↔ R-STATE-05 | Câu «phí 0 ⇒ `none`» nằm trong đoạn khách huỷ. Chuyến `vixanh` bị **hệ thống** huỷ (`no_driver`) thì `paymentStatus` là gì? | Dev Booking | **Defect** (thiếu): nói rõ mọi lần huỷ có phí 0 ⇒ `none` | Minor | BA |
| A-07 | R-PAY-06 | `failed` «cần xử lý sau» — ai xử lý, bằng cách nào? | Đối soát | **Hoãn**: ngoài phạm vi lab (R-ARCH-02); đưa vào backlog Payment & Wallet Service (đối soát định kỳ, kiến trúc §8.3) | — | EM Payment |

Ba điều cần thấy trong bảng:

- **Anomaly ≠ defect.** 7 mục ⇒ 4 defect, 1 câu hỏi thành việc mới, 1 bác bỏ, 1 hoãn. Nếu chép thẳng 7 mục thành 7 bug, đội sẽ sửa cả thứ không sai (A-05).
- **A-01 là phát hiện thật** trên `SPEC.md` v1.1 của lab: mã nguồn đã làm đúng ý định (`src/app/trips.ts` gán `fare: q.total` cho chuyến), nhưng chữ của đặc tả thì không nói vậy. Người chỉ đọc SPEC để viết test có thể đặt oracle sai cho đơn đồ ăn.

### 5.5. Báo cáo review (fixing and reporting)

```text
REVIEW R-PAY · SPEC v1.1 mục 11 · kết thúc Thứ Năm (kết quả mô phỏng)
Người tham gia : 7 (4 reviewer) · nỗ lực thực tế ≈ 9,5 giờ công
Anomaly        : 7 → 4 defect (1 Major, 3 Minor) · 1 việc mới · 1 bác bỏ · 1 hoãn
Defect report  : 1 (A-01, Major) — đã sửa trong bản mới, follow-up đạt
Exit criteria  : (1) đạt  (2) đạt — 0 Major mở  (3) đạt — A-04 có hạn chót  (4) đạt
Kết luận       : chấp nhận đặc tả bản sửa; mở lại R-PAY khi thêm webhook/đối soát
Phản hồi       : gửi BA và squad Booking; đề xuất thêm mục «trường dữ liệu của chuyến» vào SPEC
```

## 6. Góc nhìn nâng cao

**Communication and analysis không bắt buộc là một cuộc họp.** Syllabus dùng chữ «thường» (typically). Đội phân tán có thể phân tích bất đồng bộ trong công cụ ghi chú, miễn mỗi anomaly vẫn có trạng thái, chủ sở hữu, hành động. Cái mất đi là lúc hai góc nhìn va nhau như A-01 — nên giữ một cuộc gọi ngắn cho các mục tranh cãi.

**Metric review là con dao hai lưỡi.** Đếm số defect mỗi người tìm được rồi đưa vào đánh giá năng lực vi phạm yếu tố số 1 («đánh giá người tham gia không bao giờ là mục tiêu»). Hậu quả quen thuộc: author giấu tài liệu tới phút chót, reviewer tách một lỗi thành năm. Metric chỉ dùng để cải tiến quy trình (TEST-026).

**Đừng để một người kiêm review leader, moderator và scribe.** Vừa tranh luận, vừa giữ giờ, vừa ghi chép thì thứ bị bỏ thường là ghi chép — và quyết định trong họp biến mất.

**Sai lầm thường gặp:**

- Không có exit criteria ⇒ «xong» khi hết giờ họp.
- Bỏ individual review cho «tiết kiệm thời gian» ⇒ cuộc họp thành buổi đọc to tài liệu.
- Mời toàn người cùng góc nhìn (bốn dev Payment) ⇒ bỏ sót lỗi phía bên gọi và phía đối soát.
- Sửa trực tiếp vào tài liệu trong lúc họp ⇒ không ai biết bản nào là bản đã thống nhất.

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1.** Squad Promotion có 4 người: PO (viết đặc tả voucher R-VOUCHER), 2 dev, 1 tester. Họ muốn làm một review **có tính hình thức gần inspection** cho R-VOUCHER-06…07. Hãy phân 6 vai trò cho 4 người, tôn trọng ràng buộc của inspection về author.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Author = PO. Author không được làm review leader hay scribe ⇒ Review leader = tester (kiêm moderator); Scribe = dev 1; Reviewer = dev 1, dev 2, tester (một người có thể giữ nhiều vai). Manager = người cấp thời gian (Engineering Manager), có thể ngoài 4 người. Nên mời thêm một reviewer ngoài squad, vd dev Booking — bên kiểm lượt voucher lúc tạo chuyến (R-VOUCHER-07).</p>
</details>

**Bài tập 2.** Mỗi việc sau thuộc hoạt động nào trong năm hoạt động? (a) gửi đường dẫn đặc tả và checklist, hỏi mọi người đã mở được chưa; (b) quyết định lần này chỉ review R-CANCEL, đặt tiêu chí «0 Major mở»; (c) reviewer ghi «R-CANCEL-04: "lớn hơn hẳn" có cần ví dụ biên không?»; (d) cả nhóm thống nhất mục đó là đề xuất, giao PO thêm ví dụ; (e) PO sửa đặc tả, review leader xác nhận đạt tiêu chí và gửi tóm tắt.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(a) Review initiation · (b) Planning · (c) Individual review · (d) Communication and analysis · (e) Fixing and reporting.</p>
</details>

**Bài tập 3.** Mỗi tình huống vi phạm yếu tố thành công nào? (a) Review cả `SPEC.md` 38 KB trong một buổi 2 giờ; (b) Trưởng phòng tuyên bố «ai tìm nhiều lỗi nhất được thưởng, tác giả nhiều lỗi nhất bị nhắc nhở»; (c) Gửi tài liệu lúc 8:55 cho cuộc họp 9:00; (d) Một reviewer mới được mời làm scribe nhưng không biết phải ghi gì.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(a) Review từng phần nhỏ. (b) Đánh giá người tham gia không bao giờ là mục tiêu (yếu tố 1); kéo theo văn hoá review (yếu tố 7). (c) Đủ thời gian chuẩn bị. (d) Đào tạo đủ để biết làm tròn vai trò.</p>
</details>

**Câu 1 (K2).** Trong review process của CTFL v4.0.1, exit criteria của review được xác định ở hoạt động nào?

- a) Review initiation
- b) Planning
- c) Communication and analysis
- d) Fixing and reporting

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — initiation lo cho mọi người sẵn sàng (có tài liệu, hiểu vai trò). <b>b đúng</b> — phạm vi của review, gồm exit criteria, được định ở planning. <b>c sai</b> — ở đây quyết định trạng thái từng anomaly và chất lượng work product. <b>d sai</b> — đây là lúc <i>kiểm tra</i> xem đã đạt exit criteria chưa, không phải lúc đặt ra.</p>
</details>

**Câu 2 (K1).** Vai trò nào chịu trách nhiệm giữ một môi trường review an toàn, nơi mọi người nói thẳng được?

- a) Review leader
- b) Manager
- c) Moderator
- d) Scribe

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a sai</b> — review leader chịu trách nhiệm chung: chọn người, tổ chức khi nào, ở đâu. <b>b sai</b> — manager quyết định cái gì được review và cấp nguồn lực. <b>c đúng</b> — moderator (facilitator) lo họp hiệu quả: hoà giải, giữ giờ, môi trường an toàn. <b>d sai</b> — scribe gom anomaly và ghi quyết định.</p>
</details>

**Câu 3 (K2).** Phát biểu nào đúng về hoạt động communication and analysis?

- a) Mọi anomaly reviewer ghi lại đều được chuyển thành defect report.
- b) Với từng anomaly, cần quyết định trạng thái, người chịu trách nhiệm và hành động cần làm.
- c) Đây là lúc reviewer lần đầu đọc work product.
- d) Author sửa work product ngay trong cuộc họp để tiết kiệm thời gian.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — anomaly chưa chắc là defect; chỉ defect mới cần defect report (ở fixing and reporting). <b>b đúng</b> — đúng nội dung mục 3.2.2. <b>c sai</b> — reviewer đọc ở individual review, trước đó. <b>d sai</b> — việc sửa thuộc fixing and reporting; sửa trong họp làm mất dấu quyết định.</p>
</details>

**Câu 4 (K1).** Điều nào là yếu tố giúp review thành công theo CTFL v4.0.1?

- a) Dùng số defect mỗi người tìm được để đánh giá năng lực.
- b) Review toàn bộ work product trong một buổi để giữ mạch.
- c) Review từng phần nhỏ để reviewer không mất tập trung.
- d) Bỏ thời gian chuẩn bị, tập trung vào cuộc họp.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a sai</b> — syllabus nói rõ đánh giá người tham gia không bao giờ là mục tiêu. <b>b sai</b> — ngược với yếu tố review từng phần nhỏ. <b>c đúng</b>. <b>d sai</b> — đủ thời gian chuẩn bị là một yếu tố thành công.</p>
</details>

**Câu 5 (K2).** Trong individual review, reviewer làm gì?

- a) Áp dụng một hay nhiều review technique, ghi lại anomaly, đề xuất và câu hỏi.
- b) Quyết định trạng thái của mọi anomaly đã ghi.
- c) Tạo defect report cho từng anomaly.
- d) Định nghĩa exit criteria cho lần review.

<details>
<summary><b>Đáp án: a</b></summary>
<p><b>a đúng</b> — đúng mô tả individual review. <b>b sai</b> — quyết định trạng thái thuộc communication and analysis. <b>c sai</b> — defect report thuộc fixing and reporting, và chỉ cho defect. <b>d sai</b> — exit criteria thuộc planning.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB® Certified Tester Foundation Level Syllabus **v4.0.1** (2024): chương 3, mục 3.2.2 Review Process Activities, 3.2.3 Roles and Responsibilities in Reviews, 3.2.5 Success Factors for Reviews (đối chiếu thêm 3.2.4 Review Types). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (tra *review*, *anomaly*, *moderator*, *scribe*, *review leader*, *exit criteria*).
- Trang chủ ISTQB: https://www.istqb.org/
- ISO/IEC 20246:2017 — *Software and systems engineering — Work product reviews*: quy trình review chung, vai trò chi tiết và review technique mà CTFL dẫn chiếu.
- Đặc tả lab Tới Nơi (`SPEC.md` mục 11 R-PAY, R-ARCH-02) và tài liệu review của bài trong `articles/review-process-va-cac-vai-tro/`: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-021 · Static Testing là gì?](/articles/static-testing-la-gi) · [TEST-004 · Error, Defect, Failure](/articles/error-defect-failure-root-cause) · [TEST-009 · Vai trò trong kiểm thử](/articles/vai-tro-trong-kiem-thu) · [TEST-026 · Inspection](/articles/inspection-review-hinh-thuc) · [TEST-028 · Checklist-based Review](/articles/checklist-based-review) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)

Bài tiếp theo: [Informal Review](/articles/informal-review).
