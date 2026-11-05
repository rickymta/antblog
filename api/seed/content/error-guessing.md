---
slug: error-guessing
title: "Error Guessing: lập danh sách fault attack cho thanh toán VíXanh và huỷ chuyến"
summary: "Error guessing là đoán trước lỗi từ kinh nghiệm rồi thiết kế test để lộ chúng. Bài theo CTFL v4.0.1 mục 4.4.1, lập 12 fault attack cho thanh toán và huỷ chuyến của Tới Nơi, chạy trên API lab và tìm ra ba anomaly mới."
category: testing
level: advanced
tags: [testing, istqb, ctfl, test-051, error-guessing, fault-attack, experience-based, payment, toi-noi]
language: vi
---

Các kỹ thuật hộp đen ở Series 04 cho ta test **từ những gì đặc tả nói**. Nhưng nhiều failure tốn tiền nhất của một hệ thống thanh toán lại nằm ở chỗ đặc tả không nói tới: tài xế bấm «Hoàn thành» hai lần, ví trả lời chậm đúng lúc, khách bấm huỷ đúng giây thứ 120. Bài này trả lời: **error guessing là gì theo ISTQB, fault attack biến «linh cảm» thành test thế nào**, và một danh sách 12 fault attack chạy thật trên lab Tới Nơi tìm ra gì.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-051</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S05 · White-box &amp; Experience-based Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">Test Level</th><td>System</td></tr>
<tr><th scope="row">Test Type</th><td>Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Error Guessing (fault attack)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/functional-testing">TEST-017 · Functional Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab <code>articles/error-guessing</code>: danh sách 12 fault attack cho thanh toán VíXanh và huỷ chuyến (bấm hoàn thành hai lần, cổng hết giờ, ví từ chối, huỷ đúng giây thứ 120, lý do huỷ 201 ký tự…) và chạy trên API lab bằng vitest + supertest</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 4 · mục 4.4 và 4.4.1 · FL-4.4.1 (K2). Liên quan: mục 4.1 · FL-4.1.1 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

**Error guessing** (đoán lỗi) là kỹ thuật thiết kế test dựa trên kinh nghiệm: tester **đoán trước** những sai lầm dễ xảy ra, những defect chúng để lại và những failure chúng gây ra, rồi viết test nhắm thẳng vào đó. Câu hỏi của nó không phải «đặc tả yêu cầu gì?» mà là **«chỗ này người ta thường làm sai thế nào?»**.

Vấn đề nó giải quyết: kỹ thuật hộp đen chỉ mạnh bằng đặc tả. Phân vùng cho phí huỷ sinh ra ca «chưa đủ 2 phút / từ 2 phút trở lên», nhưng không tự gợi ra ca «khách và tài xế bấm cùng lúc» — đặc tả không nói tới. Kinh nghiệm thì có: dev rất hay viết «đọc → kiểm → ghi» mà quên rằng giữa đọc và ghi có người khác chen vào.

Khi nào dùng:

- Sau khi đã có bộ test hệ thống theo kỹ thuật hộp đen, để **bù chỗ trống** — CTFL coi kỹ thuật dựa trên kinh nghiệm là phần bổ sung, không thay thế.
- Ở vùng **rủi ro cao**, lịch sử đã có lỗi: thanh toán, huỷ có phí.
- Khi đặc tả sơ sài hoặc ít thời gian.

## 2. Lý thuyết theo ISTQB

### 2.1. Vị trí: nhóm kỹ thuật dựa trên kinh nghiệm

CTFL v4.0.1 mục 4.1 (FL-4.1.1, K2) chia test technique thành ba nhóm: **black-box** (dựa trên hành vi được đặc tả), **white-box** (dựa trên cấu trúc bên trong) và **experience-based** (dựa trên hiểu biết và kinh nghiệm của tester). Syllabus nhấn hai ý về nhóm thứ ba: hiệu quả của nó **phụ thuộc rất nhiều vào kỹ năng của tester**, và nó **tìm được defect mà hai nhóm kia bỏ sót** — nên nó bổ sung cho hai nhóm kia. Mục 4.4 nêu ba kỹ thuật thường dùng: error guessing (bài này), exploratory testing ([TEST-052](/articles/exploratory-testing)) và checklist-based testing ([TEST-053](/articles/checklist-based-testing)).

### 2.2. Định nghĩa và ba nguồn kinh nghiệm

Theo mục 4.4.1 (FL-4.4.1, K2), error guessing là kỹ thuật dùng để **dự đoán sự xuất hiện của error, defect và failure** dựa trên hiểu biết của tester, gồm ba nguồn (diễn đạt lại):

1. Ứng dụng **đã từng chạy thế nào** trong quá khứ (lỗi cũ, hành vi lạ đã gặp).
2. Những **loại sai lầm lập trình viên hay mắc** và loại defect chúng sinh ra.
3. Những **loại failure đã xảy ra ở các ứng dụng tương tự**.

Ba khái niệm error → defect → failure ở đây đúng nghĩa của [TEST-004](/articles/error-defect-failure-root-cause): đoán *sai lầm* (dev dùng `<=` thay `<`), suy ra *defect* (điều kiện sai trong hàm phí huỷ), rồi thiết kế input để *failure* lộ ra (huỷ đúng giây 120 mà không mất phí).

Syllabus gợi sáu nhóm nơi error/defect/failure hay nằm — input (input đúng bị từ chối, tham số thiếu), output (sai định dạng, sai kết quả), logic (thiếu trường hợp, sai toán tử), computation (sai phép tính), interfaces (lệch tham số, kiểu), data (khởi tạo sai, sai kiểu). Áp vào Tới Nơi:

| Nhóm (CTFL 4.4.1) | Đoán lỗi ở Tới Nơi |
|---|---|
| Input | Lý do huỷ 200 ký tự có dấu bị từ chối; `etaMinutes` gửi dạng chuỗi |
| Output | Thông báo lỗi nói sai luật; tiền hiện thiếu « đ» |
| Logic | `<=` thay `<` ở mốc 2 phút; quên ca hai người cùng thao tác |
| Computation | Phí huỷ ô tô lấy mức xe máy; thu phí trên `fare` thay vì phí huỷ |
| Interfaces | Thử lại cổng thanh toán với khoá idempotency **mới** ⇒ trừ tiền hai lần |
| Data | `paymentStatus` không được cập nhật khi thu thất bại; tài xế kẹt `busy` |

### 2.3. Fault attack — cách thực hiện error guessing có kỷ luật

Syllabus nêu **fault attack** là một cách triển khai error guessing: tester **tạo ra hoặc thu thập** một danh sách các error, defect và failure có thể xảy ra, rồi thiết kế test để (a) nhận diện defect gắn với các error đó, (b) làm lộ defect, hoặc (c) gây ra failure. Danh sách có thể xây từ kinh nghiệm, từ **dữ liệu defect và failure**, hoặc từ hiểu biết chung về lý do phần mềm hỏng. CTFL dẫn Whittaker (2002, 2003) và Andrews & Whittaker (2006) cho phần này.

Khác biệt giữa «đoán mò» và fault attack nằm ở **danh sách**: có danh sách thì lặp lại, review, truyền cho người mới và cập nhật được.

### 2.4. Thuật ngữ Anh – Việt

| Thuật ngữ | Tiếng Việt | Nghĩa trong bài |
|---|---|---|
| experience-based test technique | kỹ thuật kiểm thử dựa trên kinh nghiệm | Nhóm kỹ thuật dùng hiểu biết, kinh nghiệm của tester làm nguồn thiết kế test |
| error guessing | đoán lỗi | Dự đoán error, defect, failure để thiết kế test nhắm vào chúng |
| fault attack | tấn công lỗi | Cách triển khai error guessing bằng một danh sách lỗi có thể xảy ra và test để lộ từng lỗi |
| fault list / defect list | danh sách lỗi dự đoán | Tài liệu đầu vào của fault attack — testware cần lưu và cập nhật |
| anomaly | điều bất thường | Kết quả lệch kỳ vọng, chưa kết luận là defect |

### 2.5. Phạm vi áp dụng và giới hạn

- Áp được ở **mọi test level**; bài này dùng ở system level (qua API).
- **Không đo được coverage** như kỹ thuật hộp đen/hộp trắng — nhiều nhất là «bao nhiêu mục trong danh sách đã có test».
- **Phụ thuộc người làm**: ai chưa từng gặp lỗi «thử lại với khoá mới» sẽ không nghĩ tới nó; danh sách viết ra giảm phụ thuộc này.

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Các «mẫu lỗi hay gặp» dùng ở mục 4 — bấm lặp, kết quả không rõ khi hết giờ, đọc-kiểm-ghi bị chen ngang, mốc thời gian lệch một đơn vị, đếm độ dài chuỗi Unicode — là kiến thức thực hành của ngành, không phải danh sách do ISTQB quy định. Syllabus chỉ yêu cầu hiểu error guessing và fault attack ở mức K2.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ và business rules

Đối tượng là hai luồng có tiền của lab: **thu cước qua VíXanh khi hoàn thành chuyến** và **khách huỷ có phí**.

| Mã | Nội dung (tóm tắt SPEC) |
|---|---|
| R-PAY-03 | Chuyến `vixanh` hoàn thành ⇒ thu đúng `fare`, khoá `trip:<id>:fare` |
| R-PAY-04 | Mỗi lần gọi hết giờ sau 3 giây; hết giờ/lỗi tạm thời ⇒ thử lại tối đa 2 lần **cùng khoá**; bị từ chối ⇒ không thử lại |
| R-PAY-05 | Phí huỷ > 0 với chuyến `vixanh` ⇒ thu với khoá `trip:<id>:cancel-fee` |
| R-PAY-06 | Thu thất bại **không chặn** hoàn thành/huỷ: chuyến vẫn đổi trạng thái, `paymentStatus = failed`, API 200 |
| R-CANCEL-01…04 | `searching` miễn phí; `accepted` chưa đủ 2 phút miễn phí; đúng 2 phút có phí; `arriving` có phí; trễ **quá** `acceptedAt + ETA + 10 phút` thì miễn |
| R-STATE-02/03 | Khách huỷ được từ `searching`, `accepted`, `arriving`; cặp không có trong bảng ⇒ `INVALID_TRANSITION` |
| R-IDEM-01…04 | Huỷ cần `Idempotency-Key`; cùng khoá cùng thân ⇒ trả lại kết quả cũ |
| R-API-11 | Lý do huỷ tối đa **200 ký tự** |

### 3.2. System context

Kiến trúc đích: Trip gọi Payment qua gRPC (§7.1); Payment gọi cổng thanh toán, nhận **webhook có chữ ký, có thể trễ hoặc trùng**, cho phép trạng thái `PENDING` và đối soát (§8.3). Lab là modular monolith (§18): `src/app/trips.ts` gọi thẳng `chargeWithRetry` (`src/app/payment.ts`), cổng là `FakePaymentGateway` có năm chế độ lỗi (R-TEST-03). Lab **không có** webhook, ledger, outbox (R-ARCH-02).

### 3.3. Rủi ro

Bảng chống lỗi ở §14 của kiến trúc gọi tên đúng những gì cần đoán: duplicate request, cổng thanh toán sập, webhook trùng, race condition. Hậu quả tính tiền (số **giả định**, chỉ để minh hoạ cách ước lượng):

```text
2.000.000 chuyến/ngày (hồ sơ dự án) × 30% trả bằng VíXanh (giả định) × 0,01% bị trừ hai lần (giả định)
= 60 khoản trùng/ngày × 46.000 đ (cước mẫu) ≈ 2,76 triệu đ/ngày phải hoàn — chưa tính phiếu hỗ trợ và mất lòng tin
```

Tỉ lệ lỗi rất nhỏ nhân khối lượng lớn vẫn ra tiền thật — vì vậy thanh toán là vùng đầu tiên nên có danh sách fault attack.

## 4. Phân tích kỹ thuật

### 4.1. Ba nguồn ⇒ danh sách lỗi dự đoán

| Nguồn (CTFL 4.4.1) | Ở Tới Nơi lấy từ đâu | Ý tưởng lỗi rút ra |
|---|---|---|
| Ứng dụng đã từng chạy thế nào | Phát hiện cũ của chính loạt bài: `lost_response` ghi `failed` (TEST-028), thông báo lộ mã nội bộ ở `/driver.html` (TEST-014) | Kết quả «không rõ»; thông báo lỗi sai/khó hiểu |
| Sai lầm dev hay mắc | Mẫu lỗi quen: `<` ↔ `<=`, thử lại tạo khoá mới, thử lại cả khi bị từ chối, đọc-kiểm-ghi không khoá, cắt chuỗi rồi mới/không kiểm độ dài | Mốc 120 giây, mốc tài xế trễ, retry, race |
| Failure ở ứng dụng tương tự | Các lớp sự cố quen thuộc của ứng dụng gọi xe/ví: trừ tiền hai lần khi mạng chập chờn, nút bấm đúp, người dùng gõ emoji | Bấm đúp, hai thiết bị, Unicode |

Lỗi **đã biết** (TEST-024, 027, 028) không đưa vào 12 mục dưới; trong dự án thật chúng vẫn nằm trong danh sách dưới dạng test hồi quy.

### 4.2. Mỗi mục là một giả thuyết có oracle

Một mục fault attack tốt có bốn phần: **đoán gì** (error → defect), **tấn công thế nào** (input, thao tác, môi trường), **mong đợi gì** (mã R-… làm oracle), **quan sát gì**. Thiếu oracle thì không biết đúng sai; thiếu chỗ quan sát thì failure xảy ra mà không ai thấy — «trừ tiền hai lần» chỉ thấy khi đếm lần gọi ví, không thấy trong thân HTTP.

Nhiều attack cần **móc kiểm thử** (§17.2 kiến trúc): ví giả có chế độ lỗi, đồng hồ dời được — thiếu chúng thì «cổng hết giờ» hay «huỷ đúng giây 120» gần như không làm được.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 230" role="img" aria-label="Vòng error guessing có kỷ luật: ba nguồn kinh nghiệm tạo danh sách lỗi dự đoán, mỗi mục thành một fault attack có oracle, chạy và đối chiếu, kết quả mới quay lại cập nhật danh sách.">
<defs>
<marker id="t051-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="20" width="200" height="120" rx="8" class="f-surface s-line"/>
<text x="120" y="44" text-anchor="middle" class="t-b">Ba nguồn kinh nghiệm</text>
<text x="120" y="70" text-anchor="middle" class="t-small">ứng dụng từng chạy thế nào</text>
<text x="120" y="92" text-anchor="middle" class="t-small">sai lầm dev hay mắc</text>
<text x="120" y="114" text-anchor="middle" class="t-small">failure ở hệ tương tự</text>
<rect x="250" y="20" width="190" height="120" rx="8" class="f-pk-soft s-pk"/>
<text x="345" y="44" text-anchor="middle" class="t-b t-pk">Danh sách lỗi dự đoán</text>
<text x="345" y="70" text-anchor="middle" class="t-small">error ⇒ defect ⇒ failure</text>
<text x="345" y="92" text-anchor="middle" class="t-small">xếp theo rủi ro</text>
<text x="345" y="114" text-anchor="middle" class="t-small">là testware, có phiên bản</text>
<rect x="470" y="20" width="200" height="120" rx="8" class="f-surface s-line"/>
<text x="570" y="44" text-anchor="middle" class="t-b">Fault attack</text>
<text x="570" y="70" text-anchor="middle" class="t-small">thao tác / input / môi trường</text>
<text x="570" y="92" text-anchor="middle" class="t-small">oracle: mã R-… của SPEC</text>
<text x="570" y="114" text-anchor="middle" class="t-small">dấu hiệu cần quan sát</text>
<rect x="700" y="20" width="200" height="120" rx="8" class="f-phys-soft s-phys"/>
<text x="800" y="44" text-anchor="middle" class="t-b t-phys">Chạy và đối chiếu</text>
<text x="800" y="70" text-anchor="middle" class="t-small">đạt ⇒ giữ làm hồi quy</text>
<text x="800" y="92" text-anchor="middle" class="t-small">lệch ⇒ anomaly</text>
<text x="800" y="114" text-anchor="middle" class="t-small">⇒ defect hoặc câu hỏi</text>
<line x1="220" y1="80" x2="248" y2="80" class="s-muted" stroke-width="1.6" marker-end="url(#t051-a)"/>
<line x1="440" y1="80" x2="468" y2="80" class="s-muted" stroke-width="1.6" marker-end="url(#t051-a)"/>
<line x1="670" y1="80" x2="698" y2="80" class="s-muted" stroke-width="1.6" marker-end="url(#t051-a)"/>
<path d="M800 140 L800 190 L345 190 L345 143" class="f-none s-pk" stroke-width="1.6" marker-end="url(#t051-a)"/>
<text x="572" y="182" text-anchor="middle" class="t-small t-pk">kết quả mới (kể cả defect lọt ra production) bổ sung danh sách</text>
<text x="460" y="216" text-anchor="middle" class="t-small">Không có danh sách: chỉ còn «đoán mò» — không lặp lại, không review, không truyền được</text>
</svg>
</div>
<figcaption><b>Hình 1 · Error guessing có kỷ luật.</b> Ba nguồn kinh nghiệm theo CTFL v4.0.1 mục 4.4.1; danh sách lỗi và fault attack là cách syllabus gợi để triển khai. Vòng phản hồi giống cách checklist được cập nhật ở <a href="/articles/checklist-based-review">TEST-028</a>.</figcaption>
</figure>

### 4.3. Chọn 12 mục theo rủi ro

Giữ 12 mục theo hai câu hỏi: hậu quả có dính **tiền hoặc trạng thái chuyến** không, và có **thử được trên lab** không. Ý loại ra — webhook trùng, ledger lệch, hai bản backend cùng thu — chuyển sang Series 07 và 11.

## 5. Thực hành

### 5.1. Đề bài

Lập danh sách 12 fault attack cho thanh toán VíXanh và huỷ chuyến (gồm: bấm hoàn thành hai lần, cổng hết giờ, ví từ chối, huỷ đúng giây thứ 120, lý do huỷ 201 ký tự) và thử trên API lab.

### 5.2. Danh sách fault attack FA-01…FA-12

Dữ liệu chung: khách `C001`, tài xế `D101` (SG, xe máy), 09:00 giờ VN 04/11/2026 (ngoài cao điểm), chuyến xe máy 7,4 km ⇒ cước 12.000 + 4.500 × 7,4 = 45.300 ⇒ **46.000 đ**.

| ID | Nhóm | Đoán lỗi (error ⇒ defect) | Tấn công | Mong đợi (oracle) |
|---|---|---|---|---|
| FA-01 | Logic | Không chặn thao tác lặp ⇒ thu cước lần hai | Bấm «Hoàn thành» hai lần liên tiếp | Lần 2: 409 `INVALID_TRANSITION`; ví gọi 1 lần (R-STATE-03, R-PAY-03) |
| FA-02 | Logic | Đổi trạng thái **sau** khi thu tiền ⇒ lần bấm thứ hai lọt vào lúc ví đang chậm | Hai lệnh «Hoàn thành» song song, ví chậm 300 ms | Một 200, một 409; ví gọi 1 lần |
| FA-03 | Interfaces | Không có hết giờ hoặc nuốt lỗi ⇒ treo / mất trạng thái | Ví không bao giờ trả lời (`timeout`) | 200, `completed`, `failed`, 3 lần gọi **cùng khoá** (R-PAY-04, 06) |
| FA-04 | Interfaces | Thử lại với khoá mới ⇒ trừ tiền hai lần | Hết giờ 2 lần rồi ví trả lời bình thường | `paid`, 3 lần gọi, ví trừ đúng 46.000 đ |
| FA-05 | Logic | Thử lại mọi lỗi kể cả bị từ chối | Ví `decline` | 1 lần gọi, `failed`, chuyến vẫn `completed` |
| FA-06 | Data | Lỗi thu phí huỷ làm hỏng huỷ hoặc kẹt tài xế `busy` | Khách huỷ ở `arriving`, ví `error` liên tục | 200, `cancelled`/`failed`, khoá `cancel-fee`, tài xế `online` (R-PAY-05, 06, R-ACCEPT-05) |
| FA-07 | Logic | `<=` thay `<` ở mốc 2 phút | Huỷ sau 119,999 giây và đúng 120 giây | 0 đ `grace_period`; 10.000 đ `charged` (R-CANCEL-02) |
| FA-08 | Computation | `>=` thay `>` ở mốc tài xế trễ | Huỷ đúng `acceptedAt + ETA + 10 phút` và thêm 1 ms | Đúng mốc: có phí; +1 ms: 0 đ `driver_late` (R-CANCEL-04) |
| FA-09 | Input | Kiểm độ dài lệch một / cắt khoảng trắng sai chỗ | Lý do 200, 201, 200 + một dấu cách, chỉ dấu cách | 200 OK; 201 ⇒ 400; còn lại theo SPEC (R-API-11) |
| FA-10 | Input | «200 ký tự» bị đếm khác ý người dùng | Lý do chữ tổ hợp (NFD), emoji, emoji ghép | Lý do ít hơn 200 ký tự nhìn thấy phải được nhận |
| FA-11 | Interfaces | Phạm vi khoá idempotency sai ⇒ huỷ/thu phí hai lần | Huỷ hai lần với hai khoá (hai thiết bị), rồi lặp lại khoá đầu | Khoá khác: 409; cùng khoá: trả lại kết quả cũ; ví 1 lần (R-IDEM-02) |
| FA-12 | Logic | Đọc-kiểm-ghi không chịu được thao tác chen ngang | Khách huỷ (30 giây sau khi nhận) đúng lúc tài xế bấm «Đang tới đón» | Một trong hai thao tác thắng, thông báo đúng luật R-STATE-02 |

### 5.3. Chạy trên API lab

Test ở `examples/toi-noi-testing/articles/error-guessing/fault-attacks.test.ts`: supertest gọi `createApp` với kho trong bộ nhớ, `FixedClock` và ví giả. Hết giờ được rút còn 50 ms cho nhanh; ca `FA-03R` đo với đúng 3 giây. FA-12 cần thao tác chen **đúng** giữa lúc đọc và ghi: test bọc `TripRepository` bằng một `Proxy` để chạy lệnh «arrive» của tài xế ngay sau khi use case huỷ đọc chuyến — một test double, vì ở system test thật race chỉ lộ ra khi lặp hàng nghìn lần ([TEST-086](/articles/race-condition-testing)).

```bash
cd examples/toi-noi-testing
npx vitest run articles/error-guessing
FA_REAL_TIMEOUT=1 npx vitest run articles/error-guessing -t "FA-03R"
```

Kết quả thật (cắt gọn các dòng log):

```text
[FA-01] lần 1: 200 paid · lần 2: 409 INVALID_TRANSITION · gọi ví 1 lần, trừ 46000 đ
[FA-02] hai lệnh đồng thời: 200 / 409 · gọi ví 1 lần, trừ 46000 đ
[FA-03] hết giờ 50 ms × 3: HTTP 200, completed/failed, 3 lần gọi, phản hồi sau 186 ms
[FA-04] timeout, timeout, ok: paid, 3 lần gọi, trừ 46000 đ
[FA-05] decline: completed/failed, 1 lần gọi
[FA-06] error × 3 khi thu phí huỷ: cancelled/failed, 3 lần gọi, tài xế online
[FA-07] 119,999 s ⇒ 0 đ (grace_period) · 120,000 s ⇒ 10000 đ (charged)
[FA-08] ETA 5 phút: đúng mốc ⇒ 10000 đ (charged) · mốc + 1 ms ⇒ 0 đ (driver_late)
[FA-09] 200 ký tự ⇒ 200 "aaaaaaaaaaa · 201 ký tự ⇒ 400 VALIDATION_ERROR · 200 + 1 khoảng trắng ⇒ 200 "aaaaaaaaaaa · 3 khoảng trắng ⇒ 200 ""
[FA-10] 70 × «ế» NFC: nhìn thấy 70, code point 70, UTF-16 70 ⇒ 200 · 70 × «ế» NFD: nhìn thấy 70, code point 210, UTF-16 210 ⇒ 400 · 101 × 🚗: nhìn thấy 101, code point 101, UTF-16 202 ⇒ 200 · 41 × 👨‍👩‍👧: nhìn thấy 41, code point 205, UTF-16 328 ⇒ 400
[FA-11] lần 1: 200 phí 10000 · khoá khác: 409 INVALID_TRANSITION · cùng khoá: 200 replayed=true · ví 1 lần, 10000 đ
[FA-12] lần 1: INVALID_TRANSITION — «Không thể thực hiện "customer_cancel" khi chuyến đang ở trạng thái "arriving".» · thử lại: 200, phí 10000 đ (charged)
 Tests  12 passed | 1 skipped (13)
[FA-03R] hết giờ 3 000 ms × 3: tài xế chờ phản hồi «Hoàn thành» 9033 ms
```

Test khẳng định **hành vi hiện tại** nên đều xanh; phần kết luận nằm ở bảng:

| ID | Kết luận |
|---|---|
| FA-01, 02, 04, 05, 07, 08, 11 | **Đạt** — lab chịu được: trạng thái đổi trước khi thu tiền (nên lần bấm thứ hai bị chặn ngay), thử lại cùng khoá, mốc thời gian đúng toán tử |
| FA-03, 06 | **Đạt** theo SPEC. Nhưng FA-03R lộ một điều SPEC không nói: xem A-3 |
| FA-09 | Đạt về độ dài. Quan sát nhỏ: lý do chỉ toàn dấu cách được lưu thành `""` thay vì giá trị mặc định `customer_request` — ghi nhận, mức thấp |
| FA-10 | **Anomaly A-2** |
| FA-12 | **Anomaly A-1** |

### 5.4. Ba anomaly

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Diễn biến FA-12: use case huỷ của khách đọc chuyến ở trạng thái accepted và tính phí 0 đồng; tài xế chen vào ghi accepted sang arriving; lệnh ghi có điều kiện của khách thất bại và trả INVALID_TRANSITION nói không thể huỷ ở arriving, trái với bảng chuyển trạng thái; khách thử lại thì bị thu 10.000 đồng.">
<defs>
<marker id="t051-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
<marker id="t051-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<rect x="80" y="14" width="160" height="34" rx="6" class="f-pk-soft s-pk"/>
<text x="160" y="36" text-anchor="middle" class="t-b t-pk">Khách — huỷ</text>
<rect x="380" y="14" width="160" height="34" rx="6" class="f-surface s-line"/>
<text x="460" y="36" text-anchor="middle" class="t-b">Kho chuyến</text>
<rect x="680" y="14" width="160" height="34" rx="6" class="f-phys-soft s-phys"/>
<text x="760" y="36" text-anchor="middle" class="t-b t-phys">Tài xế — tới đón</text>
<line x1="160" y1="48" x2="160" y2="290" class="s-muted dash"/>
<line x1="460" y1="48" x2="460" y2="290" class="s-muted dash"/>
<line x1="760" y1="48" x2="760" y2="290" class="s-muted dash"/>
<line x1="160" y1="74" x2="456" y2="74" class="s-muted" stroke-width="1.4" marker-end="url(#t051-b)"/>
<text x="310" y="68" text-anchor="middle" class="t-small">① đọc: accepted, 30 giây sau khi nhận</text>
<text x="170" y="96" class="t-small">② tính phí: 0 đ (grace_period)</text>
<line x1="760" y1="124" x2="464" y2="124" class="s-muted" stroke-width="1.4" marker-end="url(#t051-b)"/>
<text x="610" y="118" text-anchor="middle" class="t-small">③ ghi: accepted ⇒ arriving (thành công)</text>
<line x1="160" y1="160" x2="456" y2="160" class="s-muted" stroke-width="1.4" marker-end="url(#t051-b)"/>
<text x="310" y="154" text-anchor="middle" class="t-small">④ ghi «nếu vẫn accepted» ⇒ cancelled</text>
<line x1="460" y1="188" x2="164" y2="188" class="s-warn" stroke-width="1.4" marker-end="url(#t051-w)"/>
<text x="310" y="182" text-anchor="middle" class="t-small t-warn">⑤ thất bại: trạng thái đã là arriving</text>
<rect x="20" y="204" width="560" height="40" rx="6" class="f-warn-soft s-warn"/>
<text x="300" y="222" text-anchor="middle" class="t-small t-warn">409 «Không thể thực hiện customer_cancel khi … "arriving"»</text>
<text x="300" y="238" text-anchor="middle" class="t-small t-warn">— nhưng R-STATE-02 CHO PHÉP khách huỷ ở arriving</text>
<text x="170" y="272" class="t-small">⑥ khách bấm lại ⇒ 200, phí 10.000 đ (charged)</text>
</svg>
</div>
<figcaption><b>Hình 2 · FA-12: khoá lạc quan làm đúng việc, thông báo thì sai.</b> Không có chuyện huỷ hai lần hay mất dữ liệu — ghi có điều kiện ở bước ④ chặn đúng. Vấn đề nằm ở câu thông báo và ở chỗ khách bấm trong thời gian miễn phí nhưng cuối cùng bị thu phí.</figcaption>
</figure>

**A-1 (FA-12) — thông báo `INVALID_TRANSITION` nói sai luật khi thua race.** Hàm lưu chuyển trạng thái (`saveTransition` trong `src/app/trips.ts`) khi ghi có điều kiện thất bại thì dựng lỗi từ **trạng thái mới nhất** và sự kiện cũ: «không thể `customer_cancel` khi `arriving`». Câu này trái với R-STATE-02, và R-STATE-03 chỉ dành `INVALID_TRANSITION` cho cặp *không có trong bảng*. Đề xuất ghi thành defect mức thấp–trung bình, loại «thông báo sai» (mẫu báo cáo ở [TEST-010](/articles/testware-san-pham-kiem-thu)):

```text
Tiêu đề : Khách huỷ thua race với «Đang tới đón» nhận 409 INVALID_TRANSITION nói không được huỷ ở arriving
Bước    : nhận chuyến; 30 giây sau, khách huỷ đúng lúc tài xế bấm «Đang tới đón» (FA-12)
Mong đợi: hoặc huỷ thành công, hoặc lỗi cho biết «chuyến vừa đổi trạng thái, vui lòng thử lại» (mã riêng hoặc 409 kèm trạng thái mới)
Thực tế : 409 «Không thể thực hiện "customer_cancel" khi chuyến đang ở trạng thái "arriving".»
Ảnh hưởng: khách tin rằng không huỷ được; lần bấm lại bị thu 10.000 đ dù lúc bấm đầu còn trong 2 phút miễn phí
```

Phần thứ hai — **bấm lần đầu còn miễn phí, bấm lại mất phí** — là **câu hỏi cho PO**, chưa phải defect: phí tính theo lúc khách bấm hay lúc hệ thống ghi?

**A-2 (FA-10) — «200 ký tự» chưa được định nghĩa.** Lab đếm theo **code point** (thư viện kiểm đầu vào zod 4): 70 chữ «ế» dạng tổ hợp NFD — người dùng thấy 70 ký tự — bị từ chối vì là 210 code point; 41 emoji gia đình bị từ chối; 101 emoji ô tô lại được nhận dù chiếm 202 đơn vị UTF-16. Nếu ứng dụng khách dùng `maxlength` của HTML (đếm theo đơn vị UTF-16) thì máy khách và máy chủ sẽ không đồng ý với nhau. Đây là anomaly ở **đặc tả**: R-API-11 cần nói rõ đơn vị đếm (đề xuất: chuẩn hoá NFC rồi đếm code point, ghi rõ trong SPEC), và test ở cả hai phía phải dùng cùng định nghĩa.

**A-3 (FA-03R) — tài xế chờ ≈ 9 giây cho nút «Hoàn thành» khi VíXanh không trả lời.** Đúng R-PAY-04 (3 lần × 3 giây) và R-PAY-06 (vẫn 200), nhưng toàn bộ thời gian thu tiền nằm trong request HTTP của tài xế. SPEC có độ trễ cho `POST /api/trips` (R-NFR-02) mà không có cho lệnh của tài xế; kiến trúc §8.3 thì cho phép thanh toán ở `PENDING` và xử lý bất đồng bộ. Câu hỏi cho PO và kiến trúc sư: «Hoàn thành» có nên trả ngay với `paymentStatus = pending` rồi thu ở nền? Trước khi có câu trả lời, ghi là anomaly, chưa phải defect.

### 5.5. Cập nhật danh sách sau đợt chạy

Theo vòng ở Hình 1, danh sách v1.1 thêm ba mục rút từ kết quả: «thông báo khi thua race có đúng luật?» (mọi thao tác ghi có điều kiện), «độ dài chuỗi đếm cùng một cách ở mọi lớp?» (tên món, mã voucher), «thao tác nào chờ bên thứ ba trong request của người dùng?». Danh sách là đầu vào cho [TEST-053](/articles/checklist-based-testing) (checklist khi thực thi) và [TEST-054](/articles/defect-based-testing) (phân loại defect).

## 6. Góc nhìn nâng cao

**Dễ bỏ sót.** Chỉ nhìn thân HTTP sẽ bỏ lỡ FA-04 (phải đếm tiền thực trừ ở ví) và FA-06 (phải xem trạng thái tài xế). Mỗi mục cần ghi **nơi quan sát**.

**Giới hạn.** «12/12 mục đã chạy» không nói gì về số lỗi còn lại. Danh sách thiên về quá khứ — lỗi của công nghệ mới chưa có trong kinh nghiệm của ai. Nhiều attack cần điều khiển môi trường (ví giả, đồng hồ, chen thao tác): thiếu testability hooks thì đội bỏ đúng những ca nguy hiểm nhất.

**So sánh với kỹ thuật khác.**

| | Error guessing | Exploratory ([TEST-052](/articles/exploratory-testing)) | Checklist-based ([TEST-053](/articles/checklist-based-testing)) | Defect-based ([TEST-054](/articles/defect-based-testing)) | BVA ([TEST-032](/articles/boundary-value-analysis)) |
|---|---|---|---|---|---|
| Đầu vào | Danh sách lỗi dự đoán | Test charter | Checklist điều kiện | Taxonomy, lịch sử defect | Đặc tả có biên |
| Thiết kế test | Trước khi chạy, nhắm vào từng lỗi | Đồng thời với chạy | Trước, theo mục | Trước, theo nhóm lỗi | Trước, có hệ thống |
| Đo coverage | Không (chỉ % mục) | Không (theo charter) | % mục checklist | Theo nhóm taxonomy | Có (giá trị biên) |

FA-07 và FA-08 trông như BVA — đúng vậy: error guessing quyết định **chỗ nào** đáng dồn sức (mốc có tiền), BVA cho cách chọn giá trị một cách hệ thống. Hai kỹ thuật bổ sung, không cạnh tranh.

**Sai lầm thường gặp.**

- Gọi «bấm lung tung» là error guessing — không danh sách, không oracle.
- Mục quá chung («kiểm tra lỗi thanh toán») — phải nói rõ thao tác và cái cần quan sát.
- Coi mọi kết quả lạ là defect: A-2, A-3 là câu hỏi về đặc tả, phải qua PO.
- Nhầm «fault attack» với tấn công bảo mật: CTAL-TTA dùng «attack» theo nghĩa bảo mật; ở CTFL 4.4.1 đó là thao tác nhằm làm lộ lỗi chức năng.

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Bốn fault attack cho voucher

Từ R-VOUCHER-01…08, viết bốn mục fault attack (đoán lỗi, tấn công, oracle).

<details><summary><b>Lời giải mẫu</b></summary><ul><li><b>Ngày hiệu lực theo UTC thay vì giờ VN</b> — báo giá lúc 23:30 ngày 31/12/2030 giờ VN (= 16:30 UTC) và 00:30 ngày 01/01/2031 giờ VN (= 17:30 UTC ngày 31/12) với <code>GIAM20</code> — oracle R-VOUCHER-04, R-GEN-02: lần đầu hợp lệ, lần sau <code>VOUCHER_EXPIRED</code>.</li><li><b>Không chuẩn hoá mã</b> — gửi « giam20 » — oracle R-VOUCHER-02: như <code>GIAM20</code>.</li><li><b>Không trả lượt khi huỷ</b> — dùng <code>GIAM20</code>, huỷ, đặt lại trong cùng ngày — oracle R-VOUCHER-07: lần hai được nhận.</li><li><b>Thứ tự lỗi sai</b> — <code>HN10</code> cho chuyến xe máy 2 km ở SG (21.000 đ: vừa sai thành phố vừa dưới mức tối thiểu 40.000 đ) — oracle R-VOUCHER-08: <code>VOUCHER_NOT_APPLICABLE</code>, không phải <code>VOUCHER_MIN_AMOUNT</code>.</li></ul></details>

### Bài tập 2 — Sửa mục yếu

Viết lại thành fault attack chạy được: (a) «Thử thanh toán lỗi»; (b) «Thử huỷ nhiều lần»; (c) «Kiểm tra giờ».

<details><summary><b>Lời giải mẫu</b></summary><ul><li>(a) FA-04: ví <code>timeout</code> hai lần rồi bình thường ⇒ <code>paid</code>, 3 lần gọi cùng khoá <code>trip:&lt;id&gt;:fare</code>, ví trừ đúng một lần — quan sát ở ví, không chỉ ở HTTP.</li><li>(b) FA-11: hai khoá khác nhau ⇒ lần hai 409; khoá cũ ⇒ <code>Idempotent-Replayed: true</code>; phí thu một lần.</li><li>(c) FA-07: huỷ ở <code>accepted</code> sau 119,999 giây và đúng 120 giây ⇒ 0 đ / 10.000 đ (R-CANCEL-02).</li></ul></details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Phát biểu nào mô tả đúng error guessing theo CTFL v4.0.1?

- a) Thiết kế test bằng cách chọn ngẫu nhiên input
- b) Dự đoán error, defect và failure dựa trên hiểu biết của tester để thiết kế test
- c) Đo tỉ lệ nhánh được thực thi trong code
- d) Học đối tượng test trong khi đồng thời thiết kế và chạy test

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — đó là random testing, không dựa trên kinh nghiệm.</li><li>b) Đúng — mục 4.4.1.</li><li>c) Sai — đó là branch coverage (4.3.2).</li><li>d) Sai — đó là exploratory testing (4.4.2).</li></ul></details>

**Câu 2 (K1).** Nguồn nào **không** nằm trong ba nguồn hiểu biết mà mục 4.4.1 nêu cho error guessing?

- a) Ứng dụng đã từng hoạt động thế nào
- b) Loại sai lầm lập trình viên hay mắc
- c) Loại failure đã xảy ra ở ứng dụng tương tự
- d) Báo cáo code coverage của bộ test hiện có

<details><summary><b>Đáp án: d</b></summary><ul><li>a, b, c) Là ba nguồn được nêu.</li><li>d) Đúng là ngoại lệ — coverage là thước đo của kỹ thuật hộp trắng, không phải nguồn kinh nghiệm.</li></ul></details>

**Câu 3 (K2).** Điều nào đúng về fault attack?

- a) Là một loại kiểm thử bảo mật bắt buộc trước khi phát hành
- b) Tester tạo hoặc thu thập danh sách lỗi có thể xảy ra rồi thiết kế test để lộ hoặc gây ra chúng
- c) Chỉ dùng được khi có mã nguồn
- d) Thay thế được kỹ thuật hộp đen nếu danh sách đủ dài

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — «attack» ở 4.4.1 nhằm lộ lỗi nói chung, không phải tấn công bảo mật.</li><li>b) Đúng — mục 4.4.1.</li><li>c) Sai — fault attack ở bài này chạy qua API, không đọc code.</li><li>d) Sai — kỹ thuật dựa trên kinh nghiệm là bổ sung (4.1).</li></ul></details>

**Câu 4 (K2).** Dev viết `if (sinceAcceptedMs <= 120_000) phí = 0`, trong khi đặc tả nói đúng 2 phút là có phí. Theo nhóm của mục 4.4.1, defect này thuộc nhóm nào?

- a) Input
- b) Logic (sai toán tử)
- c) Interfaces
- d) Data (khởi tạo sai)

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — input vẫn đúng, xử lý sai.</li><li>b) Đúng — syllabus lấy «wrong operator» làm ví dụ của nhóm logic; FA-07 nhắm vào đúng lỗi này.</li><li>c) Sai — không có giao tiếp giữa hai thành phần.</li><li>d) Sai — không liên quan khởi tạo dữ liệu.</li></ul></details>

**Câu 5 (K2).** Giới hạn chính của error guessing theo CTFL v4.0.1 là gì?

- a) Không áp được ở system level
- b) Hiệu quả phụ thuộc rất nhiều vào kỹ năng của tester
- c) Luôn đòi hỏi công cụ tự động
- d) Chỉ tìm được defect mà kỹ thuật hộp đen đã tìm

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — áp ở mọi level.</li><li>b) Đúng — mục 4.1 về nhóm experience-based.</li><li>c) Sai — làm tay được; bài này tự động hoá cho lặp lại.</li><li>d) Sai — ngược lại, nó tìm được defect mà hai nhóm kia bỏ sót.</li></ul></details>

**Câu 6 (K2).** Ví thanh toán có thể hết giờ. Fault attack nào nhắm đúng sai lầm «thử lại với khoá idempotency mới»?

- a) Ví từ chối giao dịch, kiểm chuyến vẫn hoàn thành
- b) Ví hết giờ hai lần rồi trả lời bình thường, đếm số tiền thực bị trừ
- c) Gửi lý do huỷ 201 ký tự
- d) Huỷ chuyến khi đang `searching`

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — bị từ chối thì không thử lại.</li><li>b) Đúng — FA-04: có thử lại thật, và quan sát số tiền ở ví mới thấy trừ hai lần.</li><li>c) Sai — nhắm vào kiểm độ dài input.</li><li>d) Sai — không có thu tiền.</li></ul></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 4.1 (FL-4.1.1), 4.4, 4.4.1 (FL-4.4.1) — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (mục *error guessing*, *experience-based test technique*)
- Whittaker, J. (2002), *How to Break Software: A Practical Guide to Testing*, Pearson; Whittaker, J. & Thompson, H. (2003), *How to Break Software Security*, Addison-Wesley; Andrews, M. & Whittaker, J. (2006), *How to Break Web Software*, Addison-Wesley — các nguồn CTFL v4.0.1 dẫn cho error guessing và fault attack.
- ISO/IEC/IEEE 29119-4, *Software testing — Test techniques* — CTFL dẫn làm nguồn chung về kỹ thuật kiểm thử.
- Unicode Standard Annex #15 (*Normalization Forms*) và #29 (*Text Segmentation*) — nền cho anomaly A-2.
- Lab Tới Nơi: `SPEC.md` (R-PAY, R-CANCEL, R-IDEM, R-STATE, R-API-11, R-TEST-03), `src/app/payment.ts`, `src/app/trips.ts`, `articles/error-guessing/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-004 Error, Defect, Failure](/articles/error-defect-failure-root-cause) · [TEST-017 Functional Testing](/articles/functional-testing) · [TEST-028 Checklist-based Review](/articles/checklist-based-review) · [TEST-032 Boundary Value Analysis](/articles/boundary-value-analysis) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)

Bài tiếp theo: [TEST-052 · Exploratory Testing](/articles/exploratory-testing)
