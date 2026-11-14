---
slug: checklist-based-review
title: "Checklist-based Review: xây và dùng checklist review chức năng thanh toán"
summary: "Checklist-based reviewing là review technique dùng ở bước review cá nhân. Bài xây checklist 15 mục cho chức năng thanh toán của Tới Nơi, áp lên đặc tả R-PAY và kiến trúc, rồi cập nhật checklist từ chính các phát hiện."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, ctal-ta, ctal-tta, test-028, static-testing, review, checklist, toi-noi]
language: vi
---

Đặc tả thanh toán của Tới Nơi đã được review một lần bằng checklist 8 câu ([TEST-027](/articles/review-process-va-cac-vai-tro)). Vẫn còn lọt một kiểu lỗi nguy hiểm: khách bị trừ tiền mà hệ thống ghi «thanh toán thất bại». Không phải người review kém — checklist không có câu nào hỏi về nó. Bài này trả lời: checklist review là gì theo ISTQB, xây một checklist tốt thế nào, áp nó ra sao, và giữ nó không lỗi thời bằng cách nào.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-028</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S03 · Static Testing &amp; Review Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1; Advanced — CTAL-TA v4.0, CTAL-TTA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Checklist-based reviewing (review technique)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/review-process-va-cac-vai-tro">TEST-027 Review Process và các vai trò</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Checklist 15 mục cho chức năng thanh toán, áp lên SPEC R-PAY và kiến trúc §4.9 — lab <code>articles/checklist-based-review</code> (tài liệu review + 2 test vitest làm bằng chứng)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 3 · mục 3.2.2 · FL-3.2.2 (K2); mục 3.2.4 · FL-3.2.4 (K2). CTAL-TA v4.0 · Chương 5 · mục 5.2.2 · TA-5.2.2 (K3). CTAL-TTA v4.0 · Chương 5 · mục 5.1, 5.2 · TTA 5.2.1 (K4), TTA 5.2.2 (K4)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

**Checklist-based reviewing** (review theo checklist) là cách đọc một work product đối chiếu với danh sách câu hỏi định sẵn: «số tiền có luôn dương không?», «gửi lại có bị thu hai lần không?». Checklist giải quyết ba vấn đề của review tuỳ hứng: người đọc quên đúng những điểm hay sai; chất lượng review phụ thuộc hoàn toàn vào kinh nghiệm từng người; nhiều người cùng báo một lỗi hiển nhiên mà bỏ qua lỗi khó.

Nên dùng khi một loại work product lặp lại (đặc tả thanh toán, API contract, migration), miền có rủi ro cao với kiểu lỗi đã biết, hoặc reviewer mới cần «mượn» kinh nghiệm người cũ. Checklist **không** thay việc suy nghĩ: nó nhắc, không giới hạn.

## 2. Lý thuyết theo ISTQB

### 2.1. Một review technique, không phải một loại review

CTFL v4.0.1 mục 3.2.2 đặt checklist-based reviewing ở hoạt động **individual review**: mỗi reviewer đánh giá work product bằng một hay nhiều **review technique** (ví dụ checklist-based, scenario-based) và ghi lại anomaly, đề xuất, câu hỏi. Vì vậy checklist dùng được trong **mọi** loại review của mục 3.2.4: một informal review cũng có thể dùng checklist, còn [inspection](/articles/inspection-review-hinh-thuc) thường dùng nó cùng góc nhìn của từng reviewer.

Đừng nhầm với **checklist-based testing** (CTFL 4.4.3, bài TEST-053, Series 05): đó là kỹ thuật kiểm thử dựa trên kinh nghiệm, dùng checklist **khi chạy phần mềm**. Bài này là kiểm thử **tĩnh**: không có gì được thực thi.

### 2.2. CTAL-TA v4.0 mục 5.2.2 — checklist trong năm review technique

TA-5.2.2 (K3) yêu cầu áp một review technique lên test basis để tìm defect. Về checklist, syllabus nêu (diễn đạt lại):

- Checklist **nhắc** reviewer kiểm những điểm cụ thể và giúp **phi cá nhân hoá** buổi review.
- Có thể **chung** hoặc **chuyên** theo đặc tính chất lượng, mục tiêu kiểm thử, loại test basis. Test analyst **may đo** checklist theo loại test basis, mức rủi ro, test condition.
- Phải **cập nhật thường xuyên** bằng các defect từng bị bỏ sót.
- **Không bao trùm mọi thứ**: reviewer không bị giới hạn trong danh sách, thấy gì ngoài danh sách vẫn ghi.

Bốn kỹ thuật còn lại của mục 5.2.2: **ad hoc** (tự do, phụ thuộc kỹ năng, dễ trùng lặp), **scenario-based** (chạy thử «trên giấy» theo use case), **role-based** (mỗi người đóng một persona), **perspective-based reading** (mỗi người một góc nhìn, thử dựng sản phẩm từ tài liệu — xem [TEST-026](/articles/inspection-review-hinh-thuc)). Chúng ghép được với checklist.

### 2.3. CTAL-TTA v4.0 chương 5 — checklist cho kiến trúc và code

Mục 5.2 của CTAL-TTA bổ sung: checklist có thể chung (ví dụ dùng đúng «shall»/«should», định dạng) hoặc tập trung vào một vùng (bảo mật, hiệu năng). Checklist **hữu ích nhất** là checklist do chính tổ chức **tự xây dần**, vì checklist đó phản ánh bản chất sản phẩm, môi trường phát triển (con người, công cụ, ưu tiên), lịch sử thành công và defect, và các vấn đề riêng. Checklist trong syllabus chỉ là ví dụ — phải may đo theo tổ chức, có khi theo từng dự án.

- **TTA 5.2.1 (K4)** — review **kiến trúc** theo checklist. Ví dụ của syllabus cho hiệu năng website: connection pooling, cân bằng tải, xử lý phân tán, caching, lazy instantiation, transaction concurrency, tách OLTP/OLAP, nhân bản dữ liệu.
- **TTA 5.2.2 (K4)** — review **code** theo checklist. Checklist code nên cụ thể theo ngôn ngữ, chia nhóm: cấu trúc, tài liệu, biến, phép tính số học, vòng lặp và rẽ nhánh, lập trình phòng vệ (kiểm biên, kiểm đầu vào, có hết giờ khi gọi thiết bị ngoài…).
- Mục **5.1**: technical test analyst cần **đủ thời gian chuẩn bị**, gồm cả đối chiếu tài liệu liên quan; thiếu chuẩn bị thì buổi review chỉ còn là sửa câu chữ. Họ cũng góp phần định nghĩa, áp dụng và **bảo trì** checklist.

### 2.4. Thuật ngữ Anh – Việt

| Thuật ngữ | Tiếng Việt trong bài | Ghi chú |
|---|---|---|
| checklist-based reviewing | review theo checklist | review technique, CTFL 3.2.2, TA 5.2.2 |
| checklist-based testing | kiểm thử theo checklist | kỹ thuật kinh nghiệm khi chạy test, CTFL 4.4.3 |
| review technique | kỹ thuật review | dùng ở individual review |
| de-personalize | phi cá nhân hoá | «checklist chung, không nhắm vào bạn» |
| architectural review / code review | review kiến trúc / review code | TTA 5.2.1 / 5.2.2 |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB — cách viết một mục checklist tốt</span>
<ul>
<li><b>Câu hỏi đóng</b>, trả lời được Đạt / Phát hiện / Không áp dụng — không phải chủ đề («Bảo mật?»).</li>
<li><b>Một ý mỗi mục</b>, đủ cụ thể để hai người đọc cùng trả lời giống nhau.</li>
<li><b>Có cột «vì sao»</b>: rủi ro, sự cố hay mục kiến trúc sinh ra mục đó — để biết khi nào được bỏ.</li>
<li><b>Bắt ghi bằng chứng</b> (mã yêu cầu, mục tài liệu) cho mọi «Đạt».</li>
<li><b>Có chủ, có phiên bản</b>, đủ ngắn để đọc hết trong một phiên (thường vài chục mục là tối đa).</li>
</ul>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ và business rules

Chuyến trả bằng ví VíXanh được thu tiền khi hoàn thành; khách huỷ có phí cũng bị thu qua ví. Quy tắc trong SPEC lab:

| Mã | Nội dung (tóm tắt) |
|---|---|
| R-PAY-01, 02 | `cash` hoặc `vixanh`; `paymentStatus` ∈ `cash`, `pending`, `paid`, `failed`, `none` |
| R-PAY-03 | Hoàn thành chuyến `vixanh` ⇒ thu số tiền `fare`, khoá `trip:<id>:fare` |
| R-PAY-04 | Mỗi lần gọi hết giờ sau 3 giây; hết giờ/lỗi tạm thời ⇒ thử lại tối đa 2 lần cùng khoá; bị từ chối ⇒ dừng |
| R-PAY-05 | Phí huỷ > 0 qua ví: khoá `trip:<id>:cancel-fee`; phí 0 ⇒ `none` |
| R-PAY-06 | Thu thất bại không chặn hoàn thành/huỷ; `paymentStatus = failed`, API 200 |
| R-TEST-03 | Ví giả có chế độ `lost_response`: **đã trừ tiền nhưng không trả lời** |

### 3.2. System context

```text
Kiến trúc đích:  Trip/Booking ──gRPC──► Payment & Wallet ──► cổng thanh toán (PSP)
                                            │  ledger bất biến, idempotency key, external reference
                                            ◄── webhook có chữ ký (có thể trễ/trùng) · đối soát định kỳ (§8.3)
Lab (MVP):       driverStep/cancel ──► chargeWithRetry ──► ví VíXanh giả (decline/error/timeout/lost_response)
```

Lab chỉ có «payment adapter» (R-ARCH-01). Webhook, ledger và đối soát nằm ngoài phạm vi lab (R-ARCH-02) — checklist vẫn phải hỏi về chúng, câu trả lời là «không áp dụng ở lab, xem kiến trúc».

### 3.3. Rủi ro

Trừ tiền hai lần; trừ tiền nhưng ghi thất bại; thu sai số tiền ở một loại dịch vụ; tin thông báo từ app; webhook giả hoặc lặp; không đối soát được vì thiếu mã giao dịch (kiến trúc §14).

## 4. Phân tích kỹ thuật

### 4.1. Năm bước xây checklist

1. **Chốt phạm vi**: loại work product (đặc tả + kiến trúc thanh toán), mức rủi ro (cao — tiền).
2. **Gom nguồn**: kiến trúc §4.9, §8.3, §14; các chế độ lỗi đã biết của cổng (R-TEST-03); phát hiện của lần review trước (TEST-027: A-01…A-07).
3. **Viết mục**: câu hỏi đóng + lý do + bằng chứng cần có.
4. **Nhóm và sắp** theo rủi ro: số tiền → idempotency → hết giờ/thử lại → kết quả và trạng thái → nguồn tin cậy, dữ liệu.
5. **Gắn chủ, phiên bản, lịch xem lại**; sau mỗi lần dùng, đưa phát hiện ngoài checklist và defect lọt vào bản sau.

### 4.2. Mục yếu và mục tốt

| Mục yếu | Vấn đề | Viết lại |
|---|---|---|
| «Thanh toán có an toàn không?» | Chủ đề, ai cũng trả lời «có» | PAY-14: webhook có xác minh chữ ký, chống replay, xử lý trùng/trễ? |
| «Có xử lý lỗi chưa?» | Không nói lỗi nào | PAY-08: lỗi nào thử lại, lỗi nào không, tối đa mấy lần? |
| «Idempotency OK?» | Không kiểm chứng được | PAY-04/06: khoá xác định, ổn định qua lần thử, mỗi khoản một khoá? |
| «Kiểm tra trạng thái» | Thiếu tình huống nguy hiểm nhất | PAY-09: «không rõ kết quả» có tách khỏi «thất bại»? |

PAY-09 không phải tự nghĩ ra mà đến từ việc đọc R-TEST-03 — cổng có thể **trừ tiền rồi mất phản hồi**. Checklist 8 câu của TEST-027 hỏi «trạng thái cuối là gì?», không hỏi «trạng thái cuối có **đúng sự thật** không?».

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 220" role="img" aria-label="Vòng đời checklist: xây từ nguồn rủi ro, áp trong review cá nhân, phân tích phát hiện, rồi cập nhật phiên bản mới từ phát hiện ngoài checklist và defect lọt ở test hoặc production.">
<defs>
<marker id="t028-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
<marker id="t028-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<rect x="20" y="30" width="190" height="80" rx="8" class="f-pk-soft s-pk"/>
<text x="115" y="56" text-anchor="middle" class="t-b t-pk">1 · Xây (v1.0)</text>
<text x="115" y="76" text-anchor="middle" class="t-small">kiến trúc, chế độ lỗi,</text>
<text x="115" y="94" text-anchor="middle" class="t-small">phát hiện cũ</text>
<rect x="250" y="30" width="190" height="80" rx="8" class="f-surface s-line"/>
<text x="345" y="56" text-anchor="middle" class="t-b">2 · Áp</text>
<text x="345" y="76" text-anchor="middle" class="t-small">individual review,</text>
<text x="345" y="94" text-anchor="middle" class="t-small">mỗi mục kèm bằng chứng</text>
<rect x="480" y="30" width="190" height="80" rx="8" class="f-surface s-line"/>
<text x="575" y="56" text-anchor="middle" class="t-b">3 · Phân tích</text>
<text x="575" y="76" text-anchor="middle" class="t-small">anomaly ⇒ defect,</text>
<text x="575" y="94" text-anchor="middle" class="t-small">câu hỏi, đề xuất</text>
<rect x="710" y="30" width="190" height="80" rx="8" class="f-pk-soft s-pk"/>
<text x="805" y="56" text-anchor="middle" class="t-b t-pk">4 · Cập nhật (v1.1)</text>
<text x="805" y="76" text-anchor="middle" class="t-small">thêm mục từ chỗ sót,</text>
<text x="805" y="94" text-anchor="middle" class="t-small">sửa mục mơ hồ</text>
<line x1="210" y1="70" x2="248" y2="70" class="s-muted" stroke-width="1.6" marker-end="url(#t028-a)"/>
<line x1="440" y1="70" x2="478" y2="70" class="s-muted" stroke-width="1.6" marker-end="url(#t028-a)"/>
<line x1="670" y1="70" x2="708" y2="70" class="s-muted" stroke-width="1.6" marker-end="url(#t028-a)"/>
<path d="M805 110 L805 180 L115 180 L115 113" class="f-none s-pk" stroke-width="1.6" marker-end="url(#t028-a)"/>
<text x="460" y="172" text-anchor="middle" class="t-small t-pk">lần review sau dùng bản mới</text>
<rect x="260" y="128" width="400" height="30" rx="6" class="f-warn-soft s-warn"/>
<text x="460" y="148" text-anchor="middle" class="t-small t-warn">phát hiện ngoài checklist · defect lọt ở test/production</text>
<path d="M660 143 L760 143 L760 113" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#t028-w)"/>
<text x="460" y="205" text-anchor="middle" class="t-small">Không cập nhật ⇒ checklist lỗi thời: «Đạt hết» nhưng không còn hỏi đúng rủi ro hiện tại</text>
</svg>
</div>
<figcaption><b>Hình 1 · Checklist là tài liệu sống.</b> CTAL-TA 5.2.2 yêu cầu cập nhật checklist bằng defect từng bị bỏ sót; CTAL-TTA 5.2 chỉ ra checklist tốt nhất là checklist tổ chức tự xây dần từ lịch sử của mình.</figcaption>
</figure>

## 5. Thực hành

Tài liệu ở `examples/toi-noi-testing/articles/checklist-based-review/` ([lab trên GitHub](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing)): checklist, kết quả áp (lời giải) và một test làm bằng chứng.

### 5.1. Checklist CL-PAY v1.0 — 15 mục

| Mã | Câu hỏi (rút gọn) | Nguồn |
|---|---|---|
| PAY-01 | Số tiền thu được định nghĩa cho **từng** dịch vụ và từng khoản, truy được về báo giá? | R-FOOD-01 cấu trúc giá khác |
| PAY-02 | Số tiền gửi cổng luôn > 0; tình huống 0 đ và số âm thì sao? | Số âm = hoàn tiền ngoài ý muốn |
| PAY-03 | Số nguyên đồng, không số thực? | R-GEN-01 |
| PAY-04 | Mỗi khoản có idempotency key xác định, ổn định qua mọi lần thử? | Kiến trúc §4.9, §14 |
| PAY-05 | Client gửi lại không tạo khoản thu thứ hai? | R-IDEM |
| PAY-06 | Hai khoản khác nhau không dùng chung khoá? | Khoá chung ⇒ khoản sau không thu |
| PAY-07 | Hết giờ mỗi lần gọi (3 giây) và thời gian tối đa của cả thao tác? | R-PAY-04, R-NFR-05 |
| PAY-08 | Phân biệt lỗi thử lại / không thử lại; giới hạn số lần? | Thử lại khi bị từ chối |
| PAY-09 | Kết quả **không rõ** tách khỏi «thất bại»? | R-TEST-03 `lost_response` |
| PAY-10 | Có bảng chuyển trạng thái thanh toán, biết trạng thái kết thúc? | Đối soát khép được |
| PAY-11 | Ai, khi nào, bằng cách nào xử lý thất bại / chưa rõ? | Kiến trúc §8.3 |
| PAY-12 | Thu thất bại có chặn nghiệp vụ không — ghi rõ? | Tài xế kẹt chuyến |
| PAY-13 | Không coi callback từ client là bằng chứng đã thanh toán? | Kiến trúc §4.9 |
| PAY-14 | Webhook xác minh chữ ký, chống replay, xử lý trùng/trễ? | Kiến trúc §4.9, §14 |
| PAY-15 | Không lưu PAN/CVV; lưu mã giao dịch phía cổng để đối soát? | Kiến trúc §4.9 |

### 5.2. Nhiệm vụ

Áp CL-PAY lên SPEC mục 11 (R-PAY) cùng R-IDEM, R-NFR-05, R-ARCH-02, R-TEST-03, rồi lên kiến trúc §4.9, §8.3. Mỗi mục ghi kết quả + bằng chứng; ghi thêm mọi phát hiện ngoài checklist.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Tự làm trước</span>
<p>Phần dưới là kết quả mẫu. Mức dùng chung thang với TEST-027: Major = có thể sai tiền hoặc sai trạng thái nghiệp vụ; Minor = câu chữ hoặc thiếu sót không đổi hành vi đã cài đặt.</p>
</div>

### 5.3. Kết quả trên SPEC

| Mục | Kết quả | Bằng chứng / phát hiện |
|---|---|---|
| PAY-01 | Phát hiện (đã biết) | Trùng **A-01** của [TEST-027](/articles/review-process-va-cac-vai-tro): `fare` của chuyến chưa được định nghĩa là `quote.total` (R-API-07 dùng `quote.fare` cho bảng chi tiết). Major |
| PAY-02, 03 | Đạt | Cước ≥ tối thiểu (R-FARE-07), tiền món ≥ 30.000 đ (R-FOOD-03), phí 0 ⇒ không gọi ví; số nguyên đồng (R-GEN-01) |
| PAY-04, 05, 06 | Đạt | Hai khoá `:fare`, `:cancel-fee` dùng lại khi thử (R-PAY-04); gửi lại có R-IDEM-02; `/complete` lần hai ⇒ `INVALID_TRANSITION` |
| PAY-07 | Câu hỏi (đã biết) | Thời gian tối đa của `/complete` chưa nêu — trùng A-04 |
| PAY-08 | Đạt | `declined` không thử lại; tối đa 2 lần thử lại |
| PAY-09 | **Phát hiện C-1 · Major** | Hết lượt thử ⇒ `failed` (R-PAY-06), dù R-TEST-03 thừa nhận cổng có thể đã trừ tiền |
| PAY-10 | **Phát hiện C-2 · Minor** | 5 giá trị `paymentStatus` nhưng không có bảng chuyển: `failed` có thành `paid` được không? |
| PAY-11 | Phát hiện (đã biết) | «`failed` cần xử lý sau» — trùng A-07, đã hoãn sang hệ đích |
| PAY-12 | Đạt | R-PAY-06 ghi rõ không chặn |
| PAY-13 | Đạt (ngầm định) | Không endpoint nào (R-API-07…15) cho client đặt `paymentStatus`; đề xuất viết thành quy tắc |
| PAY-14 | Không áp dụng ở lab | Webhook ngoài phạm vi (R-ARCH-02) |
| PAY-15 | **Phát hiện C-3 · Minor ở lab** | SPEC không yêu cầu lưu mã giao dịch phía cổng; kiến trúc §4.9 yêu cầu |
| *ngoài checklist* | **Phát hiện O-1** | Khoá `trip:<id>:fare` chỉ duy nhất khi id duy nhất; id tuần tự của lab (`id-0001`…) bắt đầu lại mỗi lần khởi động máy chủ |

Tổng: 8 Đạt · 5 Phát hiện (3 mới: C-1, C-2, C-3) · 1 Câu hỏi · 1 Không áp dụng · 1 phát hiện ngoài checklist. Ba mục cho kết quả trùng TEST-027 là điều tốt: hai cách review độc lập xác nhận cùng một chỗ yếu.

### 5.4. C-1 — anomaly có phải defect không?

Phát hiện C-1 có được **chỉ nhờ đọc**: R-PAY-04 cho tối đa 3 lần gọi, R-PAY-06 nói hết lượt ⇒ `failed`, R-TEST-03 nói cổng có chế độ «đã trừ tiền nhưng không trả lời». Ghép ba câu: khách có thể bị trừ tiền trong khi chuyến ghi thất bại.

Ở bước communication and analysis, Developer Payment phản biện: «cổng chống trùng theo khoá, thử lại sẽ nhận kết quả thành công». Đúng — **nếu cổng trả lời**. Để khép tranh luận, nhóm dùng một test nhỏ làm bằng chứng (không phải cách tìm ra lỗi):

```ts
const gateway = new FakePaymentGateway();
gateway.setMode('lost_response');                          // R-TEST-03: đã trừ tiền nhưng không trả lời
const outcome = await chargeWithRetry(gateway, request, { timeoutMs: 20, maxRetries: 2 });
expect(outcome).toMatchObject({ status: 'failed', failure: 'timeout', attempts: 3 });
expect(gateway.totalCharged()).toBe(52_000);               // tiền đã bị trừ, đúng một lần
```

```bash
npx vitest run articles/checklist-based-review --reporter=verbose
```

```text
[PAY-09] lost_response: status=failed, 3 lần gọi, ví đã trừ 52000 đ
[PAY-09] timeout:       status=failed, 3 lần gọi, ví đã trừ 0 đ

 Test Files  1 passed (1)
      Tests  2 passed (2)
```

(Node 22.18, vitest 3.2.7; hết giờ rút còn 20 ms để test nhanh, SPEC là 3.000 ms.) Hai tình huống trái ngược — đã trừ và chưa trừ — cho **cùng một** kết quả `failed`. Kết luận: **defect của đặc tả**, Major. Đề xuất: thêm trạng thái «chưa rõ» (tương ứng `PENDING` của kiến trúc §8.3) và quy tắc đối soát; mã lab làm đúng SPEC hiện tại nên chưa phải sửa code.

Hai phát hiện mới còn lại: **C-2** — thiếu bảng chuyển `paymentStatus`, nên không biết `failed` là kết thúc hay còn đổi được. **C-3** — mã lab chỉ lưu `paymentStatus` và bỏ `chargeId` (`collectPayment` trong `src/app/trips.ts`); chấp nhận ở lab, nhưng thiếu mã giao dịch thì đối soát C-1 không làm nổi.

### 5.5. Áp lên kiến trúc (TTA 5.2.1)

| Mục | Kiến trúc §4.9, §8.3, §14 | Nhận xét |
|---|---|---|
| PAY-04, 05 | Đạt — lưu idempotency key, unique constraint | |
| PAY-09 | Đạt — giữ `PENDING` tới khi chắc chắn | Lab lệch với đích; R-PAY phải nói rõ là giản lược |
| PAY-11 | Đạt — đối soát định kỳ | Chưa nêu chu kỳ, người chịu trách nhiệm |
| PAY-14, 15 | Đạt — verify chữ ký, chống replay; không lưu PAN/CVV, lưu external reference | |
| **K-1** (ngoài checklist) | §8.3 vẽ «Confirm booking / complete trip» rồi tạo payment intent; §9.2 chỉ nói «đúng thời điểm nghiệp vụ» | Câu hỏi cho kiến trúc sư: authorize lúc đặt hay chỉ capture lúc hoàn thành? Ảnh hưởng phí huỷ và tiền bị giữ |

Mức K4 của TTA: không chỉ tick, mà phân tích thiết kế có trả lời được câu hỏi của checklist không, còn gì bỏ ngỏ.

### 5.6. Một lượt review code nhanh (TTA 5.2.2)

Bốn mục trích từ nhóm «lập trình phòng vệ», «cấu trúc» của checklist code TTA, áp lên `src/app/payment.ts`:

| Mục (TTA 5.2.2) | `chargeWithRetry` | Kết quả |
|---|---|---|
| Có hết giờ khi gọi thiết bị/dịch vụ ngoài? | `withTimeout` bọc mỗi lần gọi | Đạt |
| Dùng hằng có tên thay magic number? | `timeoutMs`, `maxRetries` lấy từ cấu hình (3.000 ms, 2) | Đạt |
| Mọi nhánh đều được xử lý, có nhánh mặc định? | Lỗi không phải hết giờ/tạm thời ⇒ ném lại, không nuốt | Đạt |
| Đầu vào được kiểm hợp lệ? | Không kiểm `amount > 0` — dựa vào nơi gọi | Ghi nhận Minor (đặc tả đã bảo đảm số tiền dương) |

### 5.7. Bảo trì: CL-PAY v1.1

O-1 thành mục mới **PAY-16**: «Khoá idempotency có duy nhất theo thời gian — không lặp lại sau khởi động lại, reset dữ liệu, hay giữa các môi trường dùng chung sandbox cổng?». Ở lab, ví giả cũng mất dữ liệu khi khởi động lại nên không có hậu quả; nếu nhiều lần chạy cùng gọi một sandbox thật, chuyến mới mang khoá cũ sẽ nhận kết quả cũ và **không bị thu tiền**. Không mục nào bị xoá: PAY-02 «Đạt» lần này không có nghĩa nó vô dụng.

## 6. Góc nhìn nâng cao

**Rủi ro khi checklist lỗi thời.** Hôm nay PAY-14 ghi «không áp dụng ở lab». Khi lab thêm webhook (hiện thuộc R-ARCH-02), nếu không ai xem lại checklist, thói quen tick «không áp dụng» vẫn còn — đúng lúc rủi ro trở thành thật. Tương tự, khi thêm hoàn tiền (`REFUND_*` của kiến trúc §9.1), checklist v1.1 không có câu nào hỏi «hoàn tiền có idempotency không?», buổi review sẽ «Đạt hết». Biện pháp: gắn checklist với phiên bản phạm vi; mỗi thay đổi kiến trúc hay sự cố production là một sự kiện kích hoạt xem lại.

**Sai lầm thường gặp:**

- *Tick không bằng chứng.* «Đạt» mà không có mã R-… là ý kiến, không phải kết quả review.
- *Checklist quá dài.* 80 mục thì nửa sau bị lướt; tách theo vai (dev, tester, vận hành) — kết hợp với role-based reviewing.
- *Mục chung chung* («đã rõ ràng chưa?») — không bao giờ ra «Phát hiện».
- *Chỉ đọc trong danh sách.* O-1 và K-1 nằm ngoài checklist; CTAL-TA nói rõ checklist không bao trùm mọi thứ.
- *Dùng kết quả checklist để chấm người.* Checklist để phi cá nhân hoá review, không phải để xếp hạng ai tick đúng.

**Tự động hoá các mục máy kiểm được.** «Không dùng số thực cho tiền», «mọi `catch` phải ném lại hoặc ghi log» có thể thành luật static analysis ([TEST-029](/articles/static-analysis-phan-tich-code-tinh)). Checklist review khi đó chỉ giữ những câu cần con người hiểu nghiệp vụ, như PAY-09.

**So với checklist-based testing.** Cùng một ý («trừ tiền nhưng ghi thất bại»), bản review hỏi «đặc tả có tách trạng thái không rõ không?»; bản kiểm thử ở TEST-053 (Series 05) sẽ là «đặt ví giả `lost_response`, hoàn thành chuyến, kiểm số dư và trạng thái». Phát hiện ở review rẻ hơn vì chưa có dòng code nào phải sửa.

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Checklist cho voucher

Viết 5 mục checklist review cho đặc tả voucher (R-VOUCHER-01…08) và kiến trúc §4.10 (Promotion Service), rồi áp lên SPEC, ghi ít nhất một phát hiện.

<details><summary><b>Lời giải mẫu</b></summary><ul><li><b>V-1</b> Mọi mốc ngày có ghi múi giờ và tính đầu mút? — Đạt: R-VOUCHER-01 (giờ Việt Nam, tính cả hai đầu).</li><li><b>V-2</b> Khi nhiều lỗi cùng lúc, thứ tự báo lỗi có xác định? — Đạt: R-VOUCHER-08.</li><li><b>V-3</b> Tiền giảm có trần và không kéo giá dưới mức tối thiểu? — Đạt: R-VOUCHER-06, R-FARE-07.</li><li><b>V-4</b> Lượt dùng được trả lại khi huỷ? — Đạt: R-VOUCHER-07.</li><li><b>V-5</b> Giới hạn lượt dùng còn đúng khi <b>hai yêu cầu tạo chuyến đồng thời</b> từ hai báo giá khác nhau? — <b>Phát hiện</b>: R-QUOTE-03 chỉ bảo đảm một báo giá ra tối đa một chuyến; R-VOUCHER-07 không nói gì về hai báo giá cùng mã gửi cùng lúc. Kiến trúc §4.10 yêu cầu chống vượt quota đồng thời bằng thao tác nguyên tử ⇒ đặc tả lab thiếu. Câu hỏi cho PO, và một test case đồng thời cho Series 08.</li></ul></details>

### Bài tập 2 — Sửa mục yếu

Viết lại thành mục kiểm chứng được: (a) «API có ổn không?»; (b) «Đã nghĩ tới bảo mật chưa?»; (c) «Kiểm tra số tiền».

<details><summary><b>Lời giải mẫu</b></summary><ul><li>(a) «Mỗi mã lỗi trong đặc tả có HTTP status và thân lỗi theo mẫu chung (R-API-02, R-API-06)?»</li><li>(b) «Endpoint xem chuyến có trả 404 giống nhau cho chuyến không tồn tại và chuyến không có quyền xem (chống IDOR, R-API-05)?»</li><li>(c) PAY-01 và PAY-02: số tiền định nghĩa cho từng dịch vụ, luôn dương, số nguyên đồng.</li></ul></details>

### Bài tập 3 — Review code theo checklist (K4)

Dùng bốn mục TTA ở mục 5.6, tìm vấn đề trong đoạn mã giả sau:

```ts
async function charge(tripId: string, amount: number) {
  for (let i = 0; i <= 3; i++) {
    try {
      return await gateway.charge({ amount, idempotencyKey: `trip:${tripId}:${Date.now()}` });
    } catch {}
  }
}
```

<details><summary><b>Lời giải mẫu</b></summary><ul><li><b>Khoá idempotency đổi mỗi lần thử</b> (<code>Date.now()</code>) ⇒ cổng coi mỗi lần là một giao dịch mới: trừ tiền nhiều lần khi hết giờ — vi phạm R-PAY-04 và PAY-04.</li><li><b>Magic number, sai giới hạn</b>: <code>i &lt;= 3</code> là 4 lần gọi, SPEC cho tối đa 3.</li><li><b>Không có hết giờ</b> cho lời gọi ra ngoài: một lần treo là treo cả hàm.</li><li><b>Nuốt mọi lỗi</b>: <code>catch {}</code> gộp cả bị từ chối (không được thử lại) lẫn lỗi lập trình; hết vòng thì trả <code>undefined</code> — nơi gọi không biết thất bại (biến đầu ra không được gán).</li><li><b>Không kiểm đầu vào</b> <code>amount</code>.</li></ul></details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Theo CTFL v4.0.1, checklist-based reviewing được dùng ở hoạt động nào của quy trình review?

- a) Planning
- b) Individual review
- c) Fixing and reporting
- d) Chỉ trong buổi họp communication and analysis

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — planning có thể chọn kỹ thuật, nhưng việc áp kỹ thuật diễn ra khi review cá nhân.</li><li>b) Đúng — mục 3.2.2: reviewer áp một hay nhiều review technique, ví dụ checklist-based.</li><li>c) Sai — đó là bước sửa và báo cáo.</li><li>d) Sai — buổi họp phân tích anomaly đã ghi.</li></ul></details>

**Câu 2 (K2).** Phát biểu nào đúng?

- a) Checklist-based review là loại review hình thức nhất
- b) Checklist-based reviewing là review technique, dùng được trong nhiều loại review
- c) Checklist-based reviewing và checklist-based testing là một kỹ thuật
- d) Chỉ inspection mới được dùng checklist

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — loại hình thức nhất là inspection; checklist không phải loại review.</li><li>b) Đúng — kỹ thuật của individual review (3.2.2), không gắn với một loại review (3.2.4).</li><li>c) Sai — checklist-based testing (4.4.3) chạy phần mềm; review thì không.</li><li>d) Sai — informal review hay technical review đều dùng được.</li></ul></details>

**Câu 3 (K2).** Theo CTAL-TA v4.0, điều nào đúng về checklist trong review?

- a) Reviewer chỉ được ghi anomaly có trong checklist
- b) Checklist nên được cập nhật bằng các defect từng bị bỏ sót
- c) Checklist tốt là checklist dùng nguyên văn từ syllabus
- d) Checklist làm tăng tính cá nhân của buổi review

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — checklist không bao trùm mọi thứ; reviewer vẫn ghi phát hiện ngoài danh sách.</li><li>b) Đúng — mục 5.2.2.</li><li>c) Sai — checklist phải được may đo; CTAL-TTA 5.2 nói checklist trong syllabus chỉ là ví dụ.</li><li>d) Sai — ngược lại, checklist giúp phi cá nhân hoá.</li></ul></details>

**Câu 4 (K3).** Áp mục «Kết quả không rõ có được tách khỏi thất bại?» lên quy tắc: «Cổng hết giờ sau 3 lần gọi ⇒ ghi `failed`, không thử lại nữa». Phát hiện nào đúng?

- a) Không có vấn đề: thất bại là thất bại
- b) Nếu cổng đã trừ tiền nhưng mất phản hồi, chuyến bị ghi `failed` dù khách đã trả — cần trạng thái «chưa rõ» và đối soát
- c) Nên tăng lên 10 lần gọi để chắc chắn
- d) Cần đổi sang thanh toán tiền mặt

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — hết giờ không chứng minh được là chưa trừ tiền.</li><li>b) Đúng — đúng phát hiện C-1 (TA-5.2.2: áp review technique lên test basis).</li><li>c) Sai — thêm lần gọi không biến kết quả không rõ thành rõ.</li><li>d) Sai — đổi nghiệp vụ, không trả lời câu hỏi của checklist.</li></ul></details>

**Câu 5 (K4).** Cho mục checklist code «Có hết giờ hoặc bẫy lỗi khi truy cập thiết bị/dịch vụ bên ngoài?». Đoạn mã nào vi phạm?

- a) `await withTimeout(gateway.charge(req), 3000)`
- b) `const r = await gateway.charge(req)` không có hết giờ, không `try/catch`
- c) `if (amount <= 0) throw new Error('amount')`
- d) `const TIMEOUT_MS = 3000`

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — có hết giờ.</li><li>b) Đúng — gọi ra ngoài không hết giờ, không bẫy lỗi (TTA 5.2.2, nhóm lập trình phòng vệ).</li><li>c) Sai — là kiểm đầu vào, đáp ứng mục khác.</li><li>d) Sai — hằng có tên, đáp ứng mục «không dùng magic number».</li></ul></details>

**Câu 6 (K2).** Theo CTAL-TTA v4.0, vì sao checklist do tổ chức tự xây dần thường hữu ích nhất?

- a) Vì nó dài hơn checklist mẫu
- b) Vì nó phản ánh bản chất sản phẩm, môi trường phát triển và lịch sử defect của chính tổ chức
- c) Vì nó không cần cập nhật
- d) Vì nó thay được buổi họp review

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — độ dài không phải tiêu chí.</li><li>b) Đúng — mục 5.2.</li><li>c) Sai — checklist phải được cập nhật.</li><li>d) Sai — checklist hỗ trợ review cá nhân, không thay bước phân tích.</li></ul></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 3.2.2 (FL-3.2.2), 3.2.4 (FL-3.2.4), 4.4.3 — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level Test Analyst Syllabus* v4.0: mục 5.2.2 (TA-5.2.2) — https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB, *Certified Tester Advanced Level Technical Test Analyst Syllabus* v4.0: mục 5.1, 5.2, 5.2.1 (TTA 5.2.1), 5.2.2 (TTA 5.2.2) — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CTAL-TTA_Syllabus_v4.0.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (mục *checklist-based reviewing*, *review technique*)
- ISO/IEC 20246:2017, *Software and systems engineering — Work product reviews* (CTFL dẫn cho các review technique).
- Wiegers, K. (2002), *Peer Reviews in Software: A Practical Guide*, Addison-Wesley — CTAL-TTA dẫn làm nguồn checklist review.
- Brykczynski, B. (1999), «A survey of software inspection checklists», *ACM SIGSOFT Software Engineering Notes* — CTFL v4.0.1 dẫn trong tài liệu tham khảo.
- Lab Tới Nơi: `SPEC.md` (R-PAY, R-IDEM, R-TEST-03, R-ARCH-02), `src/app/payment.ts` và `articles/checklist-based-review/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-027 Review Process và các vai trò](/articles/review-process-va-cac-vai-tro) · [TEST-026 Inspection](/articles/inspection-review-hinh-thuc) · [TEST-021 Static Testing là gì?](/articles/static-testing-la-gi) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)

Bài tiếp theo: [TEST-029 · Static Analysis: phân tích code tĩnh](/articles/static-analysis-phan-tich-code-tinh)
