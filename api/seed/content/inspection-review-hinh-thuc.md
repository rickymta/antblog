---
slug: inspection-review-hinh-thuc
title: "Inspection: review hình thức nhất — soát đặc tả phí huỷ chuyến"
summary: "Inspection là loại review hình thức nhất của ISTQB: vai trò cố định, tiêu chí vào/ra, checklist và số đo để cải tiến quy trình. Bài inspect bản nháp đặc tả phí huỷ chuyến của Tới Nơi và tìm ra 12 defect."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, ctal-ta, test-026, static-testing, review, inspection, toi-noi]
language: vi
---

Bản nháp đặc tả phí huỷ chuyến của Tới Nơi chỉ dài một trang, nhưng mỗi dòng trong đó quyết định tiền của khách và thu nhập của tài xế. Một buổi đọc lướt sẽ thấy vài lỗi; một **inspection** tìm được nhiều hơn hẳn, đồng thời để lại số đo cho biết lần sau nên review khác đi thế nào. Bài này trả lời: inspection khác các loại review khác ở đâu, chạy inspection ra sao, và số đo của inspection dùng vào việc gì.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-026</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S03 · Static Testing &amp; Review Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1; Advanced — CTAL-TA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Inspection (kết hợp checklist-based và perspective-based reviewing)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/walkthrough">TEST-024 Walkthrough</a>, <a href="/articles/technical-review-api-contract">TEST-025 Technical Review</a>, <a href="/articles/review-process-va-cac-vai-tro">TEST-027 Review Process và các vai trò</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Inspection bản nháp đặc tả phí huỷ (R-CANCEL) với checklist cho sẵn: ghi defect theo loại, thời gian chuẩn bị, mật độ defect/trang — lab <code>articles/inspection-review-hinh-thuc</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 3 · mục 3.2.4 · FL-3.2.4 (K2); liên quan 3.2.2 · FL-3.2.2 (K2), 3.2.3 · FL-3.2.3 (K1), 3.2.5 · FL-3.2.5 (K1). CTAL-TA v4.0 · Chương 5 · mục 5.2.2 · TA-5.2.2 (K3)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

**Inspection** (soát xét hình thức) là loại review chặt chẽ nhất: có kế hoạch, vai trò cố định, mỗi người chuẩn bị riêng, buổi họp có điều phối để phân loại từng phát hiện, tiêu chí để biết khi nào được dừng, và số đo được ghi lại.

Vấn đề mà inspection giải quyết: defect trong đặc tả lan sang code, test, hai ứng dụng và kịch bản tổng đài. [Informal review](/articles/informal-review) hay [walkthrough](/articles/walkthrough) hợp để thống nhất ý tưởng, nhưng không nhằm **vét hết** anomaly. Nên dùng inspection khi work product **rủi ro cao** (tiền, an toàn, pháp lý), **cần audit trail** (khách khiếu nại phí thì phải chỉ ra quy tắc do ai duyệt), đã **tương đối ổn định**, và tổ chức muốn **cải tiến quy trình bằng số đo**.

## 2. Lý thuyết theo ISTQB

### 2.1. Inspection trong CTFL v4.0.1

Mục 3.2.4 mô tả inspection bằng bốn đặc điểm (diễn đạt lại):

1. Là loại review **hình thức nhất**, nên đi qua **đầy đủ** quy trình review chung của mục 3.2.2.
2. Mục tiêu chính: **tìm được nhiều anomaly nhất**. Mục tiêu phụ: đánh giá chất lượng, tạo niềm tin vào work product, giúp tác giả cải thiện.
3. **Số đo (metrics) được thu thập** và dùng để cải tiến SDLC, kể cả chính quy trình inspection.
4. **Author không được làm review leader hay scribe.**

Căn cứ chọn mức hình thức (cũng mục 3.2.4): mô hình SDLC, độ trưởng thành của quy trình, mức quan trọng và phức tạp của work product, yêu cầu pháp lý, nhu cầu audit trail. Cùng một work product có thể review nhẹ trước, inspect sau.

| | Informal | Walkthrough | Technical review | Inspection |
|---|---|---|---|---|
| Ai dẫn | Không cố định | Author | Moderator | Review leader; author không được dẫn |
| Quy trình | Không định nghĩa | Tuỳ | Có | Đủ 5 hoạt động của 3.2.2 |
| Chuẩn bị cá nhân | Tuỳ | Không bắt buộc | Có | Bắt buộc |
| Mục tiêu chính | Phát hiện anomaly | Hiểu, đồng thuận, học… | Đồng thuận, quyết định kỹ thuật | Vét tối đa anomaly |
| Số đo | Không | Không bắt buộc | Không bắt buộc | Bắt buộc |

### 2.2. Năm hoạt động, làm đủ mọi tác vụ

Quy trình review chung (CTFL 3.2.2, dựa trên ISO/IEC 20246; chi tiết ở [TEST-027](/articles/review-process-va-cac-vai-tro)) càng hình thức càng làm nhiều tác vụ trong mỗi hoạt động. Inspection làm hết:

| Hoạt động | Inspection làm gì |
|---|---|
| Planning | Phạm vi, vùng tập trung, **exit criteria**, nỗ lực; thêm vai trò, entry criteria, checklist, góc nhìn, số đo sẽ thu |
| Review initiation | Kiểm tra entry criteria; ai cũng có tài liệu và hiểu vai trò |
| Individual review | Bắt buộc, bấm giờ, dùng **review technique**; ghi anomaly, đề xuất, câu hỏi |
| Communication and analysis | Moderator điều phối, scribe ghi; mỗi anomaly có trạng thái, người chịu trách nhiệm, hành động; không bàn cách sửa |
| Fixing and reporting | Defect report; leader kiểm tra từng sửa đổi; đạt exit criteria mới chấp nhận; tổng hợp số đo |

Điểm hay bị hỏi: anomaly **chưa chắc** là defect. Qua bước communication and analysis mới biết đó là defect, câu hỏi đã có lời đáp, hay một đề xuất.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 285" role="img" aria-label="Năm hoạt động của inspection từ Planning tới Fixing and reporting, có cổng tiêu chí vào sau Planning và cổng tiêu chí ra sau Fixing; số đo thu ở mọi bước đi vào cải tiến quy trình rồi quay lại Planning của lần sau.">
<defs>
<marker id="t026-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
<marker id="t026-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
<marker id="t026-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-phys"/></marker>
</defs>
<rect x="20" y="40" width="156" height="115" rx="8" class="f-pk-soft s-pk"/>
<text x="98" y="68" text-anchor="middle" class="t-b t-pk">Planning</text>
<text x="98" y="100" text-anchor="middle" class="t-small">phạm vi, mục tiêu</text>
<text x="98" y="118" text-anchor="middle" class="t-small">vai trò cố định</text>
<text x="98" y="136" text-anchor="middle" class="t-small">tiêu chí vào / ra</text>
<rect x="201" y="40" width="156" height="115" rx="8" class="f-surface s-line"/>
<text x="279" y="68" text-anchor="middle" class="t-b">Initiation</text>
<text x="279" y="100" text-anchor="middle" class="t-small">kiểm tra tiêu chí vào</text>
<text x="279" y="118" text-anchor="middle" class="t-small">phát work product,</text>
<text x="279" y="136" text-anchor="middle" class="t-small">tài liệu, checklist</text>
<rect x="382" y="40" width="156" height="115" rx="8" class="f-surface s-line"/>
<text x="460" y="68" text-anchor="middle" class="t-b">Individual review</text>
<text x="460" y="100" text-anchor="middle" class="t-small">bắt buộc, bấm giờ</text>
<text x="460" y="118" text-anchor="middle" class="t-small">checklist + góc nhìn</text>
<text x="460" y="136" text-anchor="middle" class="t-small">ghi mọi anomaly</text>
<rect x="563" y="40" width="156" height="115" rx="8" class="f-surface s-line"/>
<text x="641" y="62" text-anchor="middle" class="t-b">Communication</text>
<text x="641" y="78" text-anchor="middle" class="t-b">&amp; analysis</text>
<text x="641" y="100" text-anchor="middle" class="t-small">moderator điều phối</text>
<text x="641" y="118" text-anchor="middle" class="t-small">scribe ghi quyết định</text>
<text x="641" y="136" text-anchor="middle" class="t-small">phân loại anomaly</text>
<rect x="744" y="40" width="156" height="115" rx="8" class="f-surface s-line"/>
<text x="822" y="68" text-anchor="middle" class="t-b">Fixing &amp; reporting</text>
<text x="822" y="100" text-anchor="middle" class="t-small">author sửa</text>
<text x="822" y="118" text-anchor="middle" class="t-small">defect report</text>
<text x="822" y="136" text-anchor="middle" class="t-small">leader kiểm từng sửa</text>
<line x1="176" y1="105" x2="199" y2="105" class="s-warn" stroke-width="2" marker-end="url(#t026-w)"/>
<text x="188" y="30" text-anchor="middle" class="t-small t-warn">cổng vào</text>
<line x1="357" y1="105" x2="380" y2="105" class="s-muted" stroke-width="1.6" marker-end="url(#t026-a)"/>
<line x1="538" y1="105" x2="561" y2="105" class="s-muted" stroke-width="1.6" marker-end="url(#t026-a)"/>
<line x1="719" y1="105" x2="742" y2="105" class="s-muted" stroke-width="1.6" marker-end="url(#t026-a)"/>
<line x1="822" y1="155" x2="822" y2="198" class="s-warn" stroke-width="2" marker-end="url(#t026-w)"/>
<text x="834" y="182" class="t-small t-warn">cổng ra</text>
<rect x="210" y="200" width="690" height="64" rx="8" class="f-phys-soft s-phys"/>
<text x="555" y="225" text-anchor="middle" class="t-small">Số đo thu ở mọi bước: thời gian chuẩn bị, anomaly thô / duy nhất, defect theo loại và mức,</text>
<text x="555" y="245" text-anchor="middle" class="t-small">mật độ defect/trang, nỗ lực mỗi defect — dùng cải tiến quy trình, KHÔNG đánh giá người.</text>
<rect x="20" y="200" width="160" height="64" rx="8" class="f-phys-soft s-phys"/>
<text x="100" y="227" text-anchor="middle" class="t-small t-b t-phys">Cải tiến SDLC</text>
<text x="100" y="245" text-anchor="middle" class="t-small t-phys">và quy trình inspection</text>
<line x1="210" y1="232" x2="182" y2="232" class="s-phys" stroke-width="1.6" marker-end="url(#t026-p)"/>
<line x1="100" y1="200" x2="100" y2="157" class="s-phys dash" stroke-width="1.6" marker-end="url(#t026-p)"/>
</svg>
</div>
<figcaption><b>Hình 1 · Inspection = quy trình review chung làm đủ mọi tác vụ.</b> Cổng vào chặn work product chưa sẵn sàng; cổng ra chặn việc chấp nhận khi defect nghiêm trọng còn mở. Số đo khép vòng lặp cải tiến (nét đứt: áp cho lần inspection sau).</figcaption>
</figure>

### 2.3. Vai trò và ràng buộc

Sáu vai chính của CTFL 3.2.3 (manager, author, moderator, scribe, reviewer, review leader — chi tiết ở [TEST-027](/articles/review-process-va-cac-vai-tro)) đều có mặt. Inspection thêm một ràng buộc cứng: **author không làm review leader hay scribe** — người viết ghi biên bản về lỗi của mình thì dễ ghi nhẹ đi. Một người vẫn có thể giữ nhiều vai khác (thường gặp: leader kiêm moderator). Thực hành hay gặp nhưng không phải quy định: manager không dự họp, để buổi họp không thành buổi chấm điểm; scribe ghi theo mẫu để ra số đo.

### 2.4. Tiêu chí ra, số đo, yếu tố thành công

CTFL đặt **exit criteria** ngay trong planning; mục 3.2.5 coi «mục tiêu rõ ràng, exit criteria đo được» là yếu tố thành công đầu tiên, kèm câu chốt: đánh giá người tham gia **không bao giờ** được là mục tiêu. Các yếu tố khác: chọn đúng loại review, review từng phần nhỏ, đủ thời gian chuẩn bị, phản hồi kết quả tới author, quản lý ủng hộ, đào tạo vai trò, điều phối tốt. CTFL không liệt kê số đo cụ thể; bộ số đo trong bài là thực hành phổ biến của ngành.

### 2.5. Review technique ở bước chuẩn bị

CTAL-TA v4.0 mục 5.2.2 (TA-5.2.2, K3) nêu năm review technique: **ad hoc**, **checklist-based**, **scenario-based**, **role-based**, **perspective-based reading**. Inspection hay ghép hai thứ:

- **Checklist** để mọi reviewer cùng soát những điểm hay sai (đi sâu ở [TEST-028](/articles/checklist-based-review)).
- **Perspective-based reading**: mỗi reviewer đọc từ một góc nhìn và thử **dựng sản phẩm** góc nhìn đó cần — tester thử viết test, developer thử phác hàm. Chỗ không dựng được là defect; trùng lặp giữa reviewer giảm.

CTAL-TA 5.2 gọi mục đích chung là **phase containment**: bắt defect ngay ở giai đoạn nó sinh ra.

### 2.6. Thuật ngữ Anh – Việt

| Thuật ngữ | Tiếng Việt trong bài | Ghi chú |
|---|---|---|
| inspection | inspection (soát xét hình thức) | loại review hình thức nhất |
| anomaly | điểm bất thường | chưa chắc là defect |
| entry / exit criteria | tiêu chí vào / ra | điều kiện bắt đầu / kết thúc |
| review leader / moderator / scribe | trưởng review / người điều phối / thư ký | 3.2.3 |
| perspective-based reading | đọc theo góc nhìn | TA 5.2.2 |
| phase containment | chặn defect trong giai đoạn | CTAL-TA 5.2 |
| defect density | mật độ defect | defect trên một trang |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<ul>
<li><b>Fagan inspection</b> (IBM, 1976) là gốc của inspection hiện đại: chuẩn bị bắt buộc, họp có điều phối, ghi số đo. Một số biến thể có thêm vai <b>reader</b> diễn giải tài liệu trong họp thay author.</li>
<li><b>Gilb &amp; Graham</b>, <i>Software Inspection</i> (1993) — có trong tài liệu tham khảo của CTFL v4.0.1 — phổ biến tiêu chí vào/ra và đo tốc độ chuẩn bị. CTFL chỉ nói rõ exit criteria; cổng vào là thực hành của các phương pháp này.</li>
</ul>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Khách được huỷ chuyến trước khi được đón. Phí huỷ bù cho thời gian tài xế đã bỏ ra để chạy tới, nhưng phải công bằng: không thu khi khách vừa đổi ý, không thu khi tài xế trễ. Đội Booking viết lại mục phí huỷ thành **bản nháp v0.3** (7 mục, khoảng 1 trang) và muốn thay mục R-CANCEL của SPEC.

### 3.2. Business rules liên quan cần đối chiếu

Bản nháp không đứng một mình. Bản nháp phải khớp với:

| Mã SPEC | Nội dung (tóm tắt) |
|---|---|
| R-GEN-02 | Mọi quy tắc theo giờ dùng giờ Việt Nam theo **đồng hồ của hệ thống** |
| R-STATE-02/03 | Khách chỉ huỷ được từ `searching`, `accepted`, `arriving`; cặp khác ⇒ `INVALID_TRANSITION` |
| R-STATE-05, 07 | Hệ thống huỷ chuyến tìm quá 5 phút; `acceptedAt` ghi khi nhận |
| R-PAY-01, 05, 06 | Phương thức `cash` hoặc `vixanh`; phí qua ví dùng khoá `trip:<id>:cancel-fee`; thu thất bại không chặn huỷ |
| R-IDEM-02 | Gửi lại cùng yêu cầu huỷ với cùng khoá ⇒ trả kết quả cũ, không thu lại |

### 3.3. System context

```text
App khách ──REST──► API Gateway ──► Booking (huỷ, Idempotency-Key) ──gRPC──► Trip Lifecycle (trạng thái, acceptedAt)
                                         │
                                         └──► Payment & Wallet ──► VíXanh (khoá trip:<id>:cancel-fee)
Khiếu nại phí ─────────────────────────────► Support, Refund & Dispute (cần audit trail)
```

Lab là bản MVP modular monolith của các service trên (R-ARCH-01): quy tắc phí là hàm thuần trong `src/domain/cancel.ts`, thu phí ở `src/app/payment.ts`. Inspection không chạy code; lab chỉ cung cấp SPEC để đối chiếu.

### 3.4. Rủi ro nếu defect trong đặc tả lọt qua

Thu phí sai mốc ⇒ khiếu nại và hoàn tiền; thu hai lần khi app gửi lại yêu cầu ⇒ trừ tiền trùng (rủi ro hàng đầu ở kiến trúc §14); tính giờ theo dữ liệu client ⇒ gian lận bằng cách chỉnh đồng hồ máy; hai ứng dụng và tổng đài hiểu quy tắc khác nhau ⇒ tranh cãi. Vì đặc tả liên quan **tiền** và cần **audit trail**, đội chọn inspection thay vì walkthrough — đúng căn cứ chọn loại review của CTFL 3.2.4.

## 4. Phân tích kỹ thuật

### 4.1. Planning: quyết định trước khi đọc dòng nào

| Quyết định | Lựa chọn | Lý do |
|---|---|---|
| Kích thước | 1 trang, một vòng | Review từng phần nhỏ (3.2.5) |
| Vai trò | QA Lead: leader + moderator; BA: author; Tester 2: scribe; Trưởng nhóm Booking: manager | Author không làm leader, scribe |
| Reviewer | Tester (kiểm thử), Developer Booking (cài đặt), Operator (vận hành/CSKH) | Ba góc nhìn ít trùng nhau |
| Tiêu chí vào | Có phiên bản, đã tự soát, ≤ 2 trang, SPEC + checklist đã gửi, đủ 3 góc nhìn | Không inspect bản chưa sẵn sàng |
| Tiêu chí ra | Mọi anomaly có trạng thái; 0 critical/major còn mở; critical + major > 5/trang ⇒ thêm một vòng kiểm tra lại | Đo được, không «cảm thấy ổn» |

### 4.2. Checklist cho sẵn

Checklist CL-SPEC v1.0 có 8 câu (tệp `checklist-inspection.md`): **CL-1** mốc số có ghi rõ đầu mút? · **CL-2** mọi trạng thái/tổ hợp có kết quả? · **CL-3** một khái niệm một tên? · **CL-4** không mâu thuẫn với R-GEN, R-STATE, R-PAY, R-IDEM? · **CL-5** kiểm chứng được (có số, điều kiện, kết quả quan sát được)? · **CL-6** tài liệu được tham chiếu có tồn tại? · **CL-7** có nêu hành vi khi phụ thuộc lỗi (ví, đồng hồ, gửi lại)? · **CL-8** dữ liệu tính tiền lấy từ nguồn tin cậy, không do client tự khai?

### 4.3. Perspective-based reading: dựng sản phẩm để lộ chỗ hổng

Mỗi góc nhìn có một «sản phẩm thử» phải dựng được từ bản nháp:

| Góc nhìn | Sản phẩm thử | Dựng không được ở đâu |
|---|---|---|
| Kiểm thử | Bảng quyết định trạng thái × thời gian × tài xế trễ | Ô đúng 2:00; ô «trễ đáng kể»; ô `in_transit`; ô hệ thống huỷ |
| Cài đặt | Chữ ký hàm `cancellationFee(status, vehicle, acceptedAt, eta, now)` | Lấy `now` từ đâu khi đặc tả nói đồng hồ điện thoại tài xế? |
| Vận hành | Câu trả lời mẫu cho khách hỏi «sao tôi bị trừ 10.000 đ?» | Khách trả tiền mặt thì trừ vào ví nào? Bảng phí thành phố ở đâu? |

Đây là cách áp TA-5.2.2 vào test basis: reviewer không chỉ đọc, mà **dùng** tài liệu. Bảng quyết định đầy ô «?» là bằng chứng cụ thể hơn nhiều so với nhận xét «đặc tả chưa rõ».

## 5. Thực hành

Tài liệu nằm ở `examples/toi-noi-testing/articles/inspection-review-hinh-thuc/` ([lab trên GitHub](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing)): bản nháp, checklist và hồ sơ inspection đầy đủ (lời giải).

### 5.1. Work product: bản nháp v0.3

```text
N-CANCEL-01  Phí theo trạng thái lúc khách huỷ:
             searching ................................. 0 đ
             accepted, trong vòng 2 phút sau khi tài xế nhận .. 0 đ (lý do grace_period)
             accepted, sau 2 phút ....................... phí huỷ (N-CANCEL-03)
             arriving ................................... phí huỷ (N-CANCEL-03)
             picked_up .................................. gấp đôi phí huỷ
N-CANCEL-02  Thời gian chờ tính từ lúc tài xế bấm «Nhận», theo đồng hồ điện thoại tài xế (gửi kèm lệnh nhận).
N-CANCEL-03  10.000 đ với xe máy, 20.000 đ với ô tô. Phí có thể khác nhau giữa các thành phố,
             xem bảng phí theo thành phố.
N-CANCEL-04  Nếu tài xế đến trễ đáng kể so với thời gian dự kiến đã báo, khách được miễn phí huỷ.
N-CANCEL-05  Tài xế huỷ chuyến thì khách không mất phí.
N-CANCEL-06  Kết quả huỷ trả kèm một lý do: searching, grace, late hoặc charged.
N-CANCEL-07  Phí được trừ vào ví VíXanh của khách ngay khi huỷ. Mỗi yêu cầu huỷ tạo một khoản phí
             và được ghi vào chuyến.
```

### 5.2. Nhiệm vụ

1. Bấm giờ, đọc bản nháp với checklist CL-1…CL-8 và SPEC (mục 3.2 của bài).
2. Ghi mỗi anomaly: mục, mô tả, loại (mơ hồ / thiếu / mâu thuẫn / thuật ngữ), mức (critical / major / minor).
3. Phân loại anomaly thành defect, câu hỏi hay đề xuất.
4. Tính: số defect theo loại, thời gian chuẩn bị, mật độ defect/trang.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Tự làm trước</span>
<p>Phần dưới là kết quả inspection mẫu. Làm xong nhiệm vụ rồi mới đọc để so.</p>
</div>

### 5.3. Kết quả: nhật ký sau buổi họp

Ba reviewer chuẩn bị 40, 35 và 30 phút, ghi tổng 20 anomaly thô. Buổi họp 45 phút gộp còn **14 mục duy nhất**:

| Mã | Mục | Phát hiện | Loại | Mức | CL |
|---|---|---|---|---|---|
| D-01 | 01 | «trong vòng / sau 2 phút»: đúng 2:00 thuộc dòng nào? | Mơ hồ | Major | 1 |
| D-02 | 01 | Dòng `picked_up` trái R-STATE-02/03: khách không được huỷ sau khi đã đón | Mâu thuẫn | Major | 4 |
| D-03 | 01 | Không nêu kết quả cho `requested`, `in_transit`, `completed`, `cancelled` | Thiếu | Major | 2 |
| D-04 | 02 | Giờ theo điện thoại tài xế: trái R-GEN-02, client tự khai ⇒ chỉnh giờ để thu phí | Mâu thuẫn | Critical | 4, 8 |
| D-05 | 02 | «lúc bấm Nhận» khác thời điểm hệ thống ghi `acceptedAt` (R-STATE-07) | Thuật ngữ | Minor | 3 |
| D-06 | 03 | «bảng phí theo thành phố» không tồn tại | Thiếu | Major | 6 |
| D-07 | 04 | «trễ đáng kể» không kiểm chứng được; trễ so với mốc nào? | Mơ hồ | Major | 5 |
| D-08 | 05 | Không nêu phí khi hệ thống huỷ vì hết thời gian tìm (R-STATE-05) | Thiếu | Major | 2 |
| D-09 | 06 | Mã `grace`, `late` khác `grace_period` ở mục 01 | Thuật ngữ | Minor | 3 |
| D-10 | 07 | Chuyến tiền mặt không có ví để trừ (R-PAY-01, 05) | Mâu thuẫn | Major | 4 |
| D-11 | 07 | «Mỗi yêu cầu huỷ tạo một khoản phí»: gửi lại ⇒ thu hai lần, trái R-IDEM-02 | Mâu thuẫn | Critical | 4, 7 |
| D-12 | 07 | Thu phí thất bại thì huỷ có bị chặn không? | Thiếu | Major | 7 |
| Q-01 | 03 | Đơn đồ ăn tính phí xe máy hay ô tô? | Câu hỏi | — | — |
| R-01 | — | Hiện phí dự kiến trước khi khách bấm «Huỷ» | Đề xuất | — | — |

Q-01 **không phải defect**: R-GEN-04 đã nói đơn đồ ăn luôn do xe máy giao ⇒ 10.000 đ. R-01 là đề xuất giao diện, chuyển Product Owner. Đây chính là việc của bước communication and analysis: 14 anomaly, nhưng chỉ 12 defect.

### 5.4. Số đo

| Số đo | Giá trị | Cách tính |
|---|---|---|
| Thời gian chuẩn bị | 105 phút-người | 40 + 35 + 30 |
| Anomaly thô / duy nhất / trùng | 20 / 14 / 6 (30%) | |
| Defect theo mức | 2 critical · 8 major · 2 minor | |
| Defect theo loại | Mâu thuẫn 4 · Thiếu 4 · Mơ hồ 2 · Thuật ngữ 2 | |
| Mật độ defect | **12 defect/trang** (10 critical + major/trang) | 12 ÷ 1 trang |
| Hiệu suất chuẩn bị | ≈ 6,9 defect/giờ | 12 ÷ (105 ÷ 60) |
| Defect chỉ một góc nhìn thấy | 6/12 (kiểm thử 3, cài đặt 1, vận hành 2) | |
| Tổng nỗ lực | 620 phút-người ≈ 10,3 giờ-người | planning 40 + khởi động 90 + chuẩn bị 105 + họp 270 + sửa 60 + kiểm lại 40 + báo cáo 15 |
| Nỗ lực mỗi defect | ≈ 52 phút-người | 620 ÷ 12 |

Kết luận theo tiêu chí ra: 10 critical + major trên 1 trang > 5 ⇒ bản sửa v0.4 phải qua thêm một vòng kiểm tra lại (leader + Tester 1). Sau khi sửa, v0.4 trùng nội dung với **R-CANCEL-01…07** hiện có trong SPEC — ví dụ D-01 thành «`now − acceptedAt < 2 phút`», D-07 thành «`now > acceptedAt + etaMinutes + 10 phút`», D-11 thành khoá `trip:<id>:cancel-fee`.

### 5.5. Dùng số đo để cải tiến quy trình

| Quan sát từ số đo | Thay đổi cho lần inspection sau |
|---|---|
| 4/12 defect là mâu thuẫn với mục khác của SPEC | Gói khởi động đính kèm sẵn các đoạn SPEC liên quan; checklist ghi tên từng mã R-… phải đọc chéo |
| Góc nhìn vận hành tìm 2 defect không ai khác thấy | Đặc tả có tiền luôn mời Operator |
| 10 defect critical/major/trang ở bản nộp | BA tự soát bằng CL-SPEC trước khi nộp (thêm vào tiêu chí vào) |

Không dòng nào trong bảng nói «ai tìm được nhiều nhất»: số đo theo **góc nhìn** giúp chọn người cho lần sau; số đo theo **người** biến review thành buổi chấm điểm — điều CTFL 3.2.5 cấm.

## 6. Góc nhìn nâng cao

**Ước lượng defect còn sót (ngoài syllabus).** Capture–recapture: reviewer A tìm n₁, B tìm n₂, cùng tìm m ⇒ tổng ước lượng ≈ n₁ × n₂ ÷ m. Tester 8, Developer 6, trùng 4 ⇒ 8 × 6 ÷ 4 = **12**, bằng số đã tìm. Đừng vội mừng: phương pháp giả định reviewer **độc lập, xác suất tìm như nhau**, còn perspective-based reading cố tình làm họ khác nhau. Operator tìm D-06, D-10 mà hai người kia không thấy — đúng phần ước lượng hai người bỏ sót.

**Sai lầm thường gặp:**

- *Biến số đo thành KPI cá nhân* ⇒ người ta ghi vụn để tăng số, hoặc giấu lỗi của đồng nghiệp.
- *Bỏ chuẩn bị, đọc tại chỗ trong họp* ⇒ CTAL-TTA 5.1: review thành buổi sửa câu chữ.
- *Họp thành buổi thiết kế lại* ⇒ moderator ghi «D-11: thu hai lần», không để cả phòng bàn cách làm idempotency.
- *Inspect quá nhiều một lần* ⇒ 30 trang một buổi thì nửa sau gần như không ai đọc kỹ.
- *Coi inspection là ký duyệt* ⇒ inspection tìm anomaly; phát hành đặc tả vẫn do người sở hữu yêu cầu quyết.
- *Chỉ đọc một tài liệu* ⇒ 4/12 defect ở bài này chỉ lộ ra khi đọc chéo SPEC.

**Chi phí có đáng không? (giả định, có công thức).** Nếu D-11 lọt ra production: 2 triệu chuyến/ngày (hồ sơ dự án) × 6% huỷ (tỷ lệ trong dữ liệu mẫu của hồ sơ) × giả định 20% có phí × 30% trả VíXanh × 1% bị app gửi lại ⇒ 72 khoản thu trùng/ngày, mỗi khoản kéo theo một ticket hỗ trợ và một lần hoàn tiền. Một tuần là 504 ticket. Inspection tốn 10,3 giờ-người cho cả 12 defect.

**Inspection và kỹ thuật khác.** Static analysis ([TEST-029](/articles/static-analysis-phan-tich-code-tinh)) bắt lỗi máy đọc được trong code, không đọc được bản nháp tiếng Việt. Mô hình hoá (TA-5.2.1) — ở đây là bảng quyết định — làm ô trống của đặc tả hiện ra, và bổ sung cho inspection. Pull request review thường là informal hoặc technical review; muốn gọi là inspection phải có đủ vai trò, tiêu chí vào/ra và số đo.

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Inspection mini

Bản nháp (≈ 0,25 trang): «**N-SEARCH-01.** Chuyến chờ tài xế quá 5 phút sẽ bị huỷ. Hệ thống kiểm tra mỗi phút. Khách bị tính phí theo N-CANCEL-03 nếu đã có tài xế nhận rồi huỷ.» Dùng CL-1…CL-8 và SPEC (R-STATE-04, 05; R-CANCEL-05; R-VOUCHER-07). Liệt kê defect theo loại; chuẩn bị mất 15 phút — tính mật độ defect/trang.

<details><summary><b>Lời giải mẫu</b></summary><ul><li><b>D-a (Mơ hồ, CL-1):</b> «quá 5 phút» tính từ đâu? Sau khi tài xế huỷ, chuyến tìm lại và mốc phải đặt lại (R-STATE-04, <code>searchingSince</code>).</li><li><b>D-b (Mơ hồ, CL-1):</b> đúng 5:00 có bị huỷ không? SPEC: «≥ 5 phút».</li><li><b>D-c (Mâu thuẫn, CL-4):</b> thu phí khi hệ thống huỷ, trái R-CANCEL-05.</li><li><b>D-d (Mơ hồ, CL-3):</b> «tài xế nhận rồi huỷ» — ai huỷ? Tài xế huỷ thì chuyến về <code>searching</code>, khách không mất phí.</li><li><b>D-e (Thiếu, CL-2):</b> không nêu lý do <code>no_driver</code> và việc trả lượt voucher (R-VOUCHER-07).</li><li><b>Câu hỏi:</b> quét mỗi phút ⇒ huỷ trễ tới 1 phút, có chấp nhận không? Hỏi PO.</li><li>5 ÷ 0,25 trang = <b>20 defect/trang</b>; 5 ÷ 0,25 giờ = 20 defect/giờ chuẩn bị. Chỉ so mật độ giữa tài liệu cùng loại.</li></ul></details>

### Bài tập 2 — Phân vai

Nhóm có: BA (tác giả đặc tả), QA Lead, hai tester, một developer, trưởng nhóm kỹ thuật. Phân vai cho một inspection, nêu một cách phân vai **sai** quy định và vì sao.

<details><summary><b>Lời giải mẫu</b></summary><p><b>Hợp lệ:</b> trưởng nhóm kỹ thuật = manager; QA Lead = review leader + moderator; BA = author; tester 2 = scribe; tester 1 và developer = reviewer (thêm Operator nếu mời được). <b>Sai:</b> «BA làm scribe vì nắm tài liệu nhất» hoặc «BA dẫn buổi họp» — CTFL 3.2.4 cấm author làm review leader hay scribe trong inspection. Ghép leader với moderator thì được.</p></details>

### Bài tập 3 — Đọc số đo

Hai inspection cùng loại tài liệu: A — 3 trang, chuẩn bị 30 phút/người, 4 defect; B — 1 trang, chuẩn bị 40 phút/người, 12 defect. Có kết luận «tài liệu A tốt hơn B» được không?

<details><summary><b>Lời giải mẫu</b></summary><p>Chưa. A có tốc độ chuẩn bị 6 trang/giờ, B 1,5 trang/giờ: A được đọc nhanh gấp bốn lần, mật độ thấp (1,3 defect/trang) có thể do <b>đọc lướt</b> chứ không do tài liệu tốt. Cần inspect lại một phần A với tốc độ như B rồi mới so. Đây là lý do inspection đo cả thời gian chuẩn bị, không chỉ đếm defect.</p></details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Đặc điểm nào chỉ đúng với inspection, theo CTFL v4.0.1?

- a) Do author dẫn dắt để giới thiệu work product
- b) Số đo được thu thập và dùng để cải tiến SDLC, kể cả quy trình inspection
- c) Không có quy trình định nghĩa và không cần đầu ra văn bản
- d) Mục tiêu chính là đạt đồng thuận về một vấn đề kỹ thuật

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — đó là walkthrough; trong inspection author không được làm leader.</li><li>b) Đúng — mục 3.2.4.</li><li>c) Sai — đó là informal review.</li><li>d) Sai — đó là mục tiêu chính của technical review.</li></ul></details>

**Câu 2 (K1).** Trong inspection, author **không** được giữ vai trò nào?

- a) Người trả lời câu hỏi của reviewer trong họp
- b) Người sửa work product sau họp
- c) Scribe
- d) Người nhận báo cáo kết quả

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Sai — author có mặt và trả lời câu hỏi.</li><li>b) Sai — sửa work product chính là trách nhiệm của author (3.2.3).</li><li>c) Đúng — author không làm review leader hay scribe (3.2.4).</li><li>d) Sai — kết quả phải phản hồi tới author (3.2.5).</li></ul></details>

**Câu 3 (K2).** Reviewer ghi «đơn đồ ăn tính phí xe máy hay ô tô?» trong nhật ký cá nhân. Theo quy trình review của CTFL, việc tiếp theo là gì?

- a) Tạo ngay defect report gửi author
- b) Phân tích ở bước communication and analysis để quyết định trạng thái, người chịu trách nhiệm và hành động
- c) Xoá vì câu hỏi không được ghi trong review
- d) Chuyển thẳng cho developer sửa code

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — anomaly chưa chắc là defect; ở bài, đây là câu hỏi đã có lời đáp (R-GEN-04).</li><li>b) Đúng — mục 3.2.2: mọi anomaly được phân tích và quyết định trạng thái.</li><li>c) Sai — reviewer ghi cả anomaly, đề xuất và câu hỏi.</li><li>d) Sai — chưa có defect nào được xác nhận, và work product là đặc tả.</li></ul></details>

**Câu 4 (K1).** Điều nào **không** phải yếu tố giúp review thành công theo CTFL v4.0.1?

- a) Exit criteria đo được
- b) Review từng phần nhỏ
- c) Dùng số anomaly mỗi người tìm được để xếp hạng nhân viên
- d) Đào tạo người tham gia về vai trò của họ

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Sai — là yếu tố thành công.</li><li>b) Sai — là yếu tố thành công.</li><li>c) Đúng — mục 3.2.5: đánh giá người tham gia không bao giờ được là mục tiêu.</li><li>d) Sai — là yếu tố thành công.</li></ul></details>

**Câu 5 (K3).** Tester dùng perspective-based reading cho quy tắc «Khách được miễn phí nếu huỷ trong vòng 2 phút sau khi tài xế nhận», thử viết test case từ đó. Defect nào lộ ra **trực tiếp** từ việc này?

- a) Không thể xác định kết quả mong đợi khi khách huỷ đúng 2 phút 0 giây
- b) Quy tắc viết bằng tiếng Việt thay vì tiếng Anh
- c) Mức phí 10.000 đ quá cao so với thị trường
- d) Hàm tính phí chạy chậm

<details><summary><b>Đáp án: a</b></summary><ul><li>a) Đúng — viết test cho giá trị biên thì cần kết quả mong đợi ở 2:00; quy tắc không cho (TA-5.2.2: dùng test basis để dựng testware).</li><li>b) Sai — không phải defect.</li><li>c) Sai — ý kiến kinh doanh, không lộ ra từ việc viết test.</li><li>d) Sai — hiệu năng không quan sát được khi review đặc tả.</li></ul></details>

**Câu 6 (K2).** Tình huống nào hợp với inspection nhất?

- a) Hai developer trao đổi nhanh ý tưởng cho một màn hình mới
- b) BA giới thiệu luồng giao hàng mới cho cả đội để mọi người cùng hiểu
- c) Đặc tả phí huỷ đã ổn định, liên quan tiền khách, cần lưu vết khi có khiếu nại
- d) Nhóm kiến trúc cần chọn giữa REST và gRPC cho một API nội bộ

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Sai — informal review là đủ.</li><li>b) Sai — mục tiêu giáo dục, thống nhất: walkthrough.</li><li>c) Đúng — rủi ro cao, cần audit trail, work product ổn định: căn cứ chọn mức hình thức cao (3.2.4).</li><li>d) Sai — ra quyết định kỹ thuật: technical review.</li></ul></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 3.2.2 (FL-3.2.2), 3.2.3 (FL-3.2.3), 3.2.4 (FL-3.2.4), 3.2.5 (FL-3.2.5) — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level Test Analyst Syllabus* v4.0: mục 5.2 phase containment, 5.2.1 (TA-5.2.1), 5.2.2 (TA-5.2.2) — https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB, *Certified Tester Advanced Level Technical Test Analyst Syllabus* v4.0: mục 5.1 (chuẩn bị review) — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (mục *inspection*, *anomaly*, *formal review*)
- ISO/IEC 20246:2017, *Software and systems engineering — Work product reviews* (nguồn quy trình review chung của CTFL).
- Fagan, M. E. (1976), «Design and code inspections to reduce errors in program development», *IBM Systems Journal* 15(3) — ngoài syllabus.
- Gilb, T., Graham, D. (1993), *Software Inspection*, Addison-Wesley — CTFL v4.0.1 dẫn trong tài liệu tham khảo.
- Lab Tới Nơi: `SPEC.md` (R-CANCEL, R-STATE, R-GEN, R-PAY, R-IDEM) và `articles/inspection-review-hinh-thuc/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-004 Error, Defect, Failure](/articles/error-defect-failure-root-cause) · [TEST-021 Static Testing là gì?](/articles/static-testing-la-gi) · [TEST-027 Review Process và các vai trò](/articles/review-process-va-cac-vai-tro) · [TEST-024 Walkthrough](/articles/walkthrough) · [TEST-025 Technical Review](/articles/technical-review-api-contract) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)

Bài tiếp theo: [TEST-028 · Checklist-based Review](/articles/checklist-based-review)
