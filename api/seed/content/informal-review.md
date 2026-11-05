---
slug: informal-review
title: "Informal Review: review nhanh, rẻ và biết lúc nào là chưa đủ"
summary: "Informal review theo CTFL v4.0.1: không quy trình xác định, không bắt buộc đầu ra ghi chép, mục tiêu chính là phát hiện anomaly. Thực hành 15 phút trên user story huỷ chuyến của Tới Nơi và chỉ ra giới hạn."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-023, static-testing, review, informal-review, user-story]
language: vi
---

«Xem giúp mình cái story này 15 phút được không?» là kiểu review phổ biến nhất ở mọi đội phần mềm, kể cả ở Tới Nơi. Nó không có tên trên lịch họp, không có biên bản, nhưng vẫn là một review đúng nghĩa ISTQB: **informal review** (review không chính thức). Bài này trả lời ba câu: informal review là gì theo CTFL v4.0.1, làm một buổi 15 phút sao cho ra được phát hiện có giá trị, và khi nào nó **không đủ** — phải chuyển sang review hình thức hơn.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-023</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S03 · Static Testing &amp; Review Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Informal review</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/static-testing-la-gi">TEST-021 · Static Testing là gì?</a> (nên đọc thêm <a href="/articles/review-process-va-cac-vai-tro">TEST-027 · Review Process và các vai trò</a>)</td></tr>
<tr><th scope="row">Thực hành</th><td>Cùng một đồng nghiệp review nhanh 15 phút user story «Khách huỷ chuyến và xem phí» dựa trên R-CANCEL-01…06, ghi lại câu hỏi và phát hiện — giấy/bảng tính</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 3 · mục 3.2.4 · FL-3.2.4 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Review hình thức tốn kém: phải lập kế hoạch, mời người, chờ mọi người chuẩn bị, họp, ghi nhật ký. Với một user story vừa viết xong trong buổi refinement, chờ ba ngày để có review hình thức nghĩa là dev bắt đầu code theo bản nháp còn lỗi. Informal review lấp khoảng trống đó: **một hai người, vài phút, ngay khi có bản nháp**.

Vấn đề nó giải quyết là **phát hiện sớm** những chỗ mơ hồ, thiếu, mâu thuẫn rõ ràng — loại lỗi mà chỉ cần một người thứ hai đọc là thấy. Dùng khi:

- Work product **nhỏ** (một user story, một đoạn đặc tả, một hàm), vừa viết xong.
- Mục tiêu chỉ là **tìm anomaly**, không cần đồng thuận cả đội hay đào tạo người mới.
- **Rủi ro thấp**, hoặc đây là **lượt đầu** trước một review hình thức hơn.

## 2. Lý thuyết theo ISTQB

### 2.1. Định nghĩa (mục 3.2.4)

Theo CTFL v4.0.1, informal review có ba đặc điểm:

1. **Không theo một quy trình xác định** (no defined process).
2. **Không đòi hỏi đầu ra có tài liệu chính thức** (no formal documented output).
3. **Mục tiêu chính là phát hiện anomaly.**

ISTQB Glossary định nghĩa tương tự: một loại review không theo quy trình xác định và không có đầu ra được ghi lại chính thức. Chú ý chữ «không đòi hỏi»: ghi chép **được phép** và thường **nên có** — chỉ là không bắt buộc theo mẫu nào.

### 2.2. Informal review trong bức tranh các loại review

Mục 3.2.4 nói các loại review trải từ không chính thức tới hình thức. **Mức hình thức cần thiết** phụ thuộc vào:

- SDLC đang theo (Agile có nhiều review ngắn trong refinement);
- độ trưởng thành của quy trình phát triển;
- mức nghiêm trọng (criticality) và độ phức tạp của work product;
- yêu cầu pháp lý hoặc quy định;
- nhu cầu có **audit trail** (dấu vết kiểm tra lại được).

**Chọn loại review** còn tuỳ mục tiêu, nhu cầu dự án, nguồn lực, loại work product và rủi ro, lĩnh vực nghiệp vụ, văn hoá công ty. Và một work product có thể được review **nhiều lần bằng nhiều loại**, ví dụ informal trước, formal sau.

| Loại review | Ai dẫn | Mục tiêu chính | Review cá nhân trước | Đầu ra | Đọc tiếp |
|---|---|---|---|---|---|
| **Informal review** | Không quy định | Phát hiện anomaly | Không bắt buộc | Không bắt buộc có tài liệu chính thức | Bài này |
| Walkthrough | Author | Nhiều: đánh giá chất lượng, đào tạo, đồng thuận, ý tưởng mới, phát hiện anomaly | Có thể, không bắt buộc | Tuỳ | [TEST-024](/articles/walkthrough) |
| Technical review | Moderator | Đồng thuận và quyết định về vấn đề kỹ thuật, phát hiện anomaly… | Thường có | Có | [TEST-025](/articles/technical-review-api-contract) |
| Inspection | Review leader (không phải author) | Tìm **nhiều anomaly nhất**; thu metric để cải tiến | Có — theo đủ quy trình chung | Có, kèm metric | [TEST-026](/articles/inspection-review-hinh-thuc) |

Cột «Review cá nhân trước» và «Đầu ra» của technical review là cách làm phổ biến, CTFL không viết thành quy định; các cột còn lại theo đúng mục 3.2.4.

### 2.3. Informal review đi qua quy trình chung thế nào

Quy trình chung có năm hoạt động ([TEST-027](/articles/review-process-va-cac-vai-tro)). Informal review không *theo* quy trình đó, nhưng một buổi làm tốt vẫn giữ **dáng** của vài hoạt động:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Năm hoạt động của review process khi làm informal review: planning và initiation rút gọn thành một câu nhờ và một đường dẫn; individual review thường bỏ, đọc cùng nhau; communication and analysis là trọng tâm, hỏi đáp tại chỗ; fixing and reporting do tác giả tự sửa, ghi chép không bắt buộc. Bên dưới là dòng thời gian buổi 15 phút: 2 phút bối cảnh, 9 phút đọc và hỏi, 3 phút rà ghi chú, 1 phút chốt người xử lý.">
<text x="460" y="26" text-anchor="middle" class="t-b">Informal review «ép» năm hoạt động của review process vào vài phút</text>
<rect x="20" y="44" width="160" height="64" rx="8" class="f-surface s-pk"/>
<text x="100" y="70" text-anchor="middle" class="t-b">Planning</text>
<text x="100" y="90" text-anchor="middle" class="t-10">«xem giúp 15 phút?»</text>
<rect x="200" y="44" width="160" height="64" rx="8" class="f-surface s-pk"/>
<text x="280" y="70" text-anchor="middle" class="t-b">Review initiation</text>
<text x="280" y="90" text-anchor="middle" class="t-10">gửi link bản nháp</text>
<rect x="380" y="44" width="160" height="64" rx="8" class="f-surface s-muted dash"/>
<text x="460" y="70" text-anchor="middle" class="t-b">Individual review</text>
<text x="460" y="90" text-anchor="middle" class="t-10">thường bỏ, đọc cùng nhau</text>
<rect x="560" y="44" width="160" height="64" rx="8" class="f-pk-soft s-pk"/>
<text x="640" y="64" text-anchor="middle" class="t-b">Communication</text>
<text x="640" y="80" text-anchor="middle" class="t-b">and analysis</text>
<text x="640" y="98" text-anchor="middle" class="t-10">hỏi – đáp tại chỗ</text>
<rect x="740" y="44" width="160" height="64" rx="8" class="f-surface s-pk"/>
<text x="820" y="64" text-anchor="middle" class="t-b">Fixing</text>
<text x="820" y="80" text-anchor="middle" class="t-b">and reporting</text>
<text x="820" y="98" text-anchor="middle" class="t-10">tác giả tự sửa</text>
<rect x="150" y="128" width="16" height="12" rx="2" class="f-pk-soft s-pk"/>
<text x="174" y="139" class="t-small">trọng tâm</text>
<rect x="320" y="128" width="16" height="12" rx="2" class="f-surface s-pk"/>
<text x="344" y="139" class="t-small">rút gọn tối đa</text>
<rect x="510" y="128" width="16" height="12" rx="2" class="f-surface s-muted dash"/>
<text x="534" y="139" class="t-small">tuỳ chọn, thường bỏ</text>
<text x="460" y="176" text-anchor="middle" class="t-b">Buổi 15 phút ở mục 5</text>
<rect x="60" y="192" width="107" height="40" class="f-surface s-pk"/>
<text x="113" y="216" text-anchor="middle" class="t-10">0–2′ bối cảnh</text>
<rect x="167" y="192" width="480" height="40" class="f-pk-soft s-pk"/>
<text x="407" y="216" text-anchor="middle" class="t-10">2–11′ đọc từng tiêu chí chấp nhận, so với SPEC, hỏi và ghi chú</text>
<rect x="647" y="192" width="160" height="40" class="f-surface s-pk"/>
<text x="727" y="216" text-anchor="middle" class="t-10">11–14′ rà lại ghi chú</text>
<rect x="807" y="192" width="53" height="40" class="f-warn-soft s-warn"/>
<text x="833" y="216" text-anchor="middle" class="t-10">14–15′</text>
<text x="833" y="252" text-anchor="middle" class="t-10">ai xử lý gì</text>
<text x="60" y="252" class="t-small">Không quy trình bắt buộc, không biên bản bắt buộc — nhưng vẫn để lại một danh sách ghi chú.</text>
<text x="460" y="284" text-anchor="middle" class="t-small">Ô nét đứt: CTFL không đòi; người review tự chọn có làm hay không.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Informal review so với quy trình review chung.</b> Phần trọng tâm là trao đổi trực tiếp giữa người đọc và (nếu có mặt) tác giả. Việc giữ một danh sách ghi chú ngắn không phải yêu cầu của CTFL mà là thói quen giúp phát hiện không bị rơi mất.</figcaption>
</figure>

### 2.4. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa tiếng Việt | Ghi chú |
|---|---|---|
| informal review | review không chính thức | Không quy trình xác định, không bắt buộc đầu ra chính thức |
| formal review | review hình thức | Có quy trình, vai trò, đầu ra được ghi lại |
| review type | loại review | Informal, walkthrough, technical review, inspection |
| anomaly | điều bất thường | Mục tiêu chính của informal review là tìm chúng |
| audit trail | dấu vết kiểm tra | Một lý do phải chọn review hình thức hơn |
| acceptance criteria | tiêu chí chấp nhận | Phần dễ có lỗi nhất của user story |
| user story | câu chuyện người dùng | Work product thường gặp của informal review trong Agile |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB v4.0.1 — tên gọi trong ngành</span>
<p>Các đội thường gọi informal review bằng nhiều tên: <b>buddy check</b> (nhờ bạn xem giúp), <b>pair review</b> (hai người đọc cùng nhau), <b>desk check</b> (tự đọc lại kỹ), <b>over-the-shoulder review</b> (đứng sau lưng xem màn hình), góp ý nhanh trên pull request. CTFL v4.0.1 không liệt kê các tên này; khi thi chỉ cần nhận ra đặc điểm: không quy trình xác định, không bắt buộc đầu ra chính thức, mục tiêu phát hiện anomaly. Lưu ý: một review pull request có quy tắc bắt buộc (số người duyệt, checklist, chặn merge) đã mang nhiều yếu tố hình thức hơn.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ

Trong buổi refinement, PO viết nhanh user story **US-CANCEL-07 «Khách huỷ chuyến và xem phí»** cho ứng dụng khách. Nguồn đã chốt là `SPEC.md` mục 10 (R-CANCEL). Tester nhờ một developer squad Trip lifecycle cùng đọc 15 phút trước khi story được kéo vào sprint.

Bản nháp (tệp `articles/informal-review/us-cancel-07-ban-nhap.md` trong lab):

```text
US-CANCEL-07 — Khách huỷ chuyến và xem phí  (bản nháp 0.1, PO)
Là khách hàng, tôi muốn huỷ chuyến và biết mình mất bao nhiêu tiền,
để quyết định có huỷ hay không.

AC1. Khách huỷ khi đang tìm tài xế thì không mất phí.
AC2. Khách huỷ trong vòng 2 phút sau khi tài xế nhận thì miễn phí.
AC3. Sau 2 phút, phí huỷ là 10.000 đ.
AC4. Nếu tài xế đến trễ quá 10 phút thì khách được miễn phí.
AC5. Trước khi xác nhận huỷ, app hiện số phí khách sẽ mất.
AC6. Sau khi huỷ, app hiện phí và lý do.
AC7. Khách có thể huỷ chuyến bất cứ lúc nào trước khi hoàn thành.
```

### 3.2. Business rules làm chuẩn đối chiếu

| Mã | Nội dung tóm tắt |
|---|---|
| R-CANCEL-01 | `searching` ⇒ 0 đ; `accepted` chưa đủ 2 phút ⇒ 0 đ; `accepted` từ 2 phút trở lên hoặc `arriving` ⇒ có phí; trạng thái khác ⇒ không được huỷ (`INVALID_TRANSITION`) |
| R-CANCEL-02 | «Chưa đủ 2 phút» = `now − acceptedAt < 2 phút`; đúng 2:00 ⇒ có phí |
| R-CANCEL-03 | 10.000 đ xe máy (gồm mọi đơn đồ ăn), 20.000 đ ô tô |
| R-CANCEL-04 | `now > acceptedAt + etaMinutes + 10 phút` (lớn hơn hẳn) ⇒ 0 đ; đúng bằng mốc vẫn thu |
| R-CANCEL-05 | Tài xế huỷ hoặc hệ thống huỷ ⇒ không thu phí khách |
| R-CANCEL-06 | Lý do: `searching`, `grace_period`, `driver_late`, `charged`; ưu tiên `searching` → `grace_period` → `driver_late` |

### 3.3. System context

Ứng dụng khách gọi `POST /api/trips/:id/cancel` (cần `Idempotency-Key`); phản hồi trả `trip`, `cancellationFee`, `feeReason` (R-API-11). Phí > 0 được thu theo R-PAY-05. Trong kiến trúc đích, lệnh huỷ đi qua Booking/Order Service và Trip Lifecycle Service, phí huỷ qua Payment & Wallet Service (kiến trúc §4.4, §4.7, §4.9).

### 3.4. Rủi ro nếu story sai mà không ai thấy

- Khách ô tô bị thu 10.000 đ thay vì 20.000 đ ⇒ mất doanh thu có hệ thống.
- Màn hình hứa «huỷ bất cứ lúc nào» ⇒ khách đã lên xe bấm huỷ và nhận lỗi, gọi tổng đài.
- Hiểu «trễ 10 phút» theo cách khác SPEC ⇒ app hiện «miễn phí» nhưng hệ thống thu tiền ⇒ khiếu nại.

## 4. Phân tích kỹ thuật

### 4.1. Informal review có đủ cho story này không?

Áp các yếu tố của mục 3.2.4:

| Yếu tố | Với US-CANCEL-07 | Kết luận |
|---|---|---|
| SDLC | Agile, story đang refinement | Hợp với review ngắn, sớm |
| Mức nghiêm trọng | Dính tới **tiền khách trả** | Chưa đủ nếu chỉ informal |
| Độ phức tạp | 6 quy tắc, có mốc thời gian, nhiều trạng thái | Trung bình |
| Pháp lý / audit trail | Phí huỷ có thể bị khiếu nại, cần giải thích được | Cần dấu vết ở bước sau |
| Kích thước | 7 tiêu chí, đọc trong 5 phút | Hợp với informal |

Kết luận: informal review **làm lượt đầu ngay hôm nay** để gỡ lỗi rõ ràng; đặc tả phí huỷ (R-CANCEL) vẫn sẽ qua **inspection** ở [TEST-026](/articles/inspection-review-hinh-thuc). Đây đúng là ví dụ «cùng một work product, informal trước, formal sau» của syllabus.

### 4.2. Biến 15 phút thành phát hiện có giá trị

Không có quy trình bắt buộc không có nghĩa là không có cách làm. Ba quy ước nhỏ cho buổi này:

- **Có chuẩn đối chiếu**: mở SPEC R-CANCEL bên cạnh story. Không có chuẩn thì chỉ góp ý được câu chữ.
- **Đọc theo từng tiêu chí**, với mỗi AC hỏi ba câu: *đúng với SPEC không? có biên nào mơ hồ không? kiểm thử được không?*
- **Ghi ngắn bằng ký hiệu**: `?` câu hỏi, `!` nghi là defect, `+` đề xuất. Không tranh luận cách sửa — đó là việc của tác giả.

## 5. Thực hành

### 5.1. Ghi chép sau 15 phút (kết quả mô phỏng)

Người đọc: Tester + Dev squad Trip lifecycle. Ghi vào bảng tính `ghi-chep-informal-review.csv` trong lab rồi gửi PO qua kênh chat của squad.

| # | AC | Ký hiệu | Ghi chú | Đối chiếu |
|---|---|---|---|---|
| N1 | AC2 | `!` | «Trong vòng 2 phút» — đúng 2:00 thì sao? | R-CANCEL-02: đúng 2:00 **có** phí |
| N2 | AC3 | `!` | Chỉ nêu 10.000 đ; chuyến ô tô 20.000 đ | R-CANCEL-03 |
| N3 | AC4 | `?` | «Trễ quá 10 phút» — trễ so với cái gì? | R-CANCEL-04: so với `acceptedAt + ETA + 10′`, lớn hơn hẳn |
| N4 | AC5 | `?` | API chỉ trả phí **sau khi** huỷ; muốn hiện trước thì cần API mới. Phí còn có thể đổi giữa lúc xem và lúc bấm (vượt mốc 2 phút) | R-API-11 |
| N5 | AC7 | `!` | «Bất cứ lúc nào trước khi hoàn thành» mâu thuẫn SPEC: từ `picked_up` trở đi không huỷ được | R-CANCEL-01, R-STATE-02 |
| N6 | AC6 | `?` | Hiện lý do nào khi nhiều lý do miễn cùng đúng? Dev: `grace_period` (< 2′) và `driver_late` (> ETA + 10′ ≥ 11′) không thể cùng đúng; chỉ cần ánh xạ 4 mã lý do ra câu chữ | R-CANCEL-06 |
| N7 | — | `+` | Thêm ví dụ số vào AC: huỷ ở 1:59 và 2:00; ETA 5′, huỷ ở 15:00 (thu phí) và 15:01 (miễn) | R-CANCEL-02, -04 |
| N8 | — | `?` | Thu phí qua ví thuộc story này hay story thanh toán? | R-CANCEL-07, R-PAY-05 |

Kiểm lại ví dụ N7: tài xế nhận lúc 0:00 với ETA 5 phút ⇒ mốc trễ = 0 + 5 + 10 = 15:00. Huỷ đúng 15:00 không «lớn hơn hẳn» ⇒ vẫn thu; 15:01 ⇒ miễn phí.

### 5.2. Tác giả xử lý (cũng không theo mẫu nào)

PO trả lời trong chat sau buổi refinement:

- N1, N2, N3, N5: **sửa** AC2, AC3, AC4, AC7 theo SPEC; thêm ví dụ của N7.
- N4: **tách** AC5 thành story mới «Xem trước phí huỷ» (cần API mới, cần quyết định phí theo thời điểm xem hay thời điểm bấm) — đưa vào refinement tuần sau.
- N6: AC6 đổi thành «hiện phí và câu chữ của 4 mã lý do».
- N8: **ngoài** story này; thuộc story thanh toán phí huỷ.

Kết quả: 8 ghi chú ⇒ 4 lỗi rõ ràng được sửa trước khi vào sprint, 1 story mới được tách, 2 câu hỏi được trả lời, 1 đề xuất được nhận. Tổng chi phí: 2 người × 15 phút + 10 phút của PO.

### 5.3. Những gì 15 phút này **bỏ sót**

Đối chiếu lại toàn bộ SPEC sau buổi, ba điểm sau không ai hỏi tới — loại điểm mà một review có checklist và có chuẩn bị (như inspection ở [TEST-026](/articles/inspection-review-hinh-thuc)) dễ bắt hơn:

| Bỏ sót | Vì sao informal dễ bỏ sót |
|---|---|
| Đơn **đồ ăn** luôn tính phí xe máy (R-CANCEL-03) — AC3 vô tình đúng nhưng không nói | Không có checklist «đủ cả ba dịch vụ `ride`/`parcel`/`food`» |
| Huỷ chuyến **trả lại lượt voucher** (R-VOUCHER-07) | Ảnh hưởng chéo sang miền khác; hai người đọc không phụ trách voucher |
| Bấm huỷ hai lần / mạng chập chờn — không được thu phí hai lần (R-IDEM-01, R-IDEM-02) | Không có góc nhìn kỹ thuật API, không chuẩn bị trước |

Đây là giới hạn điển hình: informal review **tìm tốt** lỗi nằm ngay trên trang đang đọc, **tìm kém** lỗi nằm ở chỗ giao nhau giữa các tài liệu.

## 6. Góc nhìn nâng cao

**Giới hạn của informal review:**

- **Phụ thuộc người đọc.** Không có checklist hay góc nhìn được giao, độ phủ bằng đúng kinh nghiệm của người được nhờ.
- **Không chuẩn bị ⇒ nông.** Đọc lần đầu trong lúc nói chuyện, người ta thấy chỗ mơ hồ câu chữ hơn là chỗ thiếu.
- **Dễ mất dấu.** Không bắt buộc ghi chép nên phát hiện nói miệng hay bị quên; không có audit trail để giải thích khi khách khiếu nại phí.
- **Áp lực xã giao.** Nhờ bạn thân xem giúp thì ngại chê; ngược lại, người được nhờ có thể «LGTM» cho xong.
- **Không đo được.** Không có metric nên không biết review của đội có hiệu quả hay không — khác với inspection.

**Khi nào phải nâng mức hình thức:** work product dính tới tiền, an toàn, dữ liệu cá nhân, hợp đồng với bên thứ ba (VíXanh, MapViet); nhiều đội cùng phụ thuộc; cần audit trail; hoặc informal review đã lộ ra mâu thuẫn với nguồn chuẩn (như N5) — dấu hiệu tài liệu cần một lượt kỹ hơn.

**Informal ≠ tuỳ tiện.** Không có quy trình xác định là đặc điểm của loại review, không phải giấy phép bỏ chuẩn đối chiếu. Một buổi informal có chuẩn đối chiếu và danh sách ghi chú vẫn rẻ, nhưng giá trị hơn nhiều so với «nhìn qua thấy ổn».

**Sai lầm thường gặp:**

- Coi informal review là «không phải review», nên không tính vào static testing — thực ra nó là loại review được dùng nhiều nhất.
- Dùng informal review làm **lượt duy nhất** cho đặc tả tiền hoặc hợp đồng API.
- Nhờ đúng tác giả tự đọc lại và gọi đó là review độc lập (xem mức độc lập ở [TEST-009](/articles/vai-tro-trong-kiem-thu)).
- Tranh luận cách sửa ngay trong buổi — 15 phút trôi qua với hai ghi chú.
- Nhầm với walkthrough: walkthrough do **author dẫn** và có thêm mục tiêu đào tạo, đồng thuận ([TEST-024](/articles/walkthrough)).

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1.** Tự làm lại buổi 15 phút với một đồng nghiệp trên bản nháp US-CANCEL-07, nhưng thêm một ràng buộc: người đọc thứ hai đóng vai **nhân viên chăm sóc khách hàng**. Ghi ít nhất 3 ghi chú mà góc nhìn đó tìm ra còn bảng ở 5.1 không có.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Ví dụ: (1) <code>?</code> Khách thấy «miễn phí do tài xế trễ» thì tổng đài tra mốc trễ ở đâu? Cần hiện <code>acceptedAt</code> và ETA trong công cụ hỗ trợ. (2) <code>+</code> AC6 nên hiện cả giờ huỷ để khách đối chiếu khi khiếu nại. (3) <code>?</code> Tài xế huỷ (R-CANCEL-05) thì khách được báo gì — story này có cần một câu cho trường hợp đó không? (4) <code>!</code> Câu «bạn sẽ mất 10.000 đ» sai với khách ô tô (cùng N2 nhưng nhìn từ phía khiếu nại). Điểm chính: đổi người đọc là đổi độ phủ — đúng giới hạn «phụ thuộc người đọc».</p>
</details>

**Bài tập 2.** Với mỗi work product sau, informal review có đủ không, hay nên chọn loại hình thức hơn? (a) Đổi câu chữ nút «Huỷ chuyến» thành «Huỷ»; (b) Đặc tả thu phí huỷ qua VíXanh với khoá idempotency (R-PAY-05); (c) Một hàm tiện ích định dạng tiền `52.000 đ` (R-UI-04) đã có test; (d) Hợp đồng gRPC `GetQuote` giữa Booking và Pricing.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(a) Informal đủ — rủi ro thấp, nhỏ. (b) Không đủ — tiền, bên thứ ba, cần audit trail ⇒ ít nhất review có cấu trúc, có thể inspection; informal làm lượt đầu được. (c) Informal đủ — nhỏ, đã có test động bù lại. (d) Không đủ — nhiều đội phụ thuộc, cần quyết định kỹ thuật ⇒ technical review (<a href="/articles/technical-review-api-contract">TEST-025</a>).</p>
</details>

**Câu 1 (K2).** Đặc điểm nào mô tả đúng informal review theo CTFL v4.0.1?

- a) Do author dẫn dắt, mục tiêu gồm đào tạo người review.
- b) Không theo quy trình xác định, không đòi hỏi đầu ra có tài liệu chính thức, mục tiêu chính là phát hiện anomaly.
- c) Theo đủ quy trình chung và thu metric để cải tiến quy trình.
- d) Do người có chuyên môn kỹ thuật thực hiện, moderator dẫn, để ra quyết định kỹ thuật.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — đó là walkthrough. <b>b đúng</b> — ba đặc điểm của informal review ở mục 3.2.4. <b>c sai</b> — đó là inspection. <b>d sai</b> — đó là technical review.</p>
</details>

**Câu 2 (K2).** Yếu tố nào là lý do mạnh nhất để chọn loại review hình thức hơn thay vì informal review?

- a) Work product chỉ có 5 dòng.
- b) Đội đang làm Agile.
- c) Cần có audit trail vì quy định pháp lý.
- d) Tác giả muốn có góp ý trong ngày.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a sai</b> — work product nhỏ hợp với informal. <b>b sai</b> — Agile không tự động đòi review hình thức; ngược lại thường có nhiều review ngắn. <b>c đúng</b> — yêu cầu pháp lý và nhu cầu audit trail là yếu tố syllabus nêu khi quyết định mức hình thức. <b>d sai</b> — cần góp ý nhanh là lý do chọn informal.</p>
</details>

**Câu 3 (K2).** PO gửi user story cho một tester qua chat; tester tự đọc trong 10 phút và trả lời ba câu hỏi trong cùng luồng chat, không có cuộc họp, không có mẫu ghi nhận. Đây là loại review nào?

- a) Informal review
- b) Walkthrough
- c) Technical review
- d) Inspection

<details>
<summary><b>Đáp án: a</b></summary>
<p><b>a đúng</b> — không quy trình xác định, không đầu ra chính thức, mục tiêu tìm anomaly. <b>b sai</b> — walkthrough do author dẫn người khác đi qua work product. <b>c sai</b> — technical review cần reviewer có chuyên môn kỹ thuật, moderator dẫn, nhằm quyết định kỹ thuật. <b>d sai</b> — inspection theo đủ quy trình, có vai trò cố định và metric.</p>
</details>

**Câu 4 (K2).** Phát biểu nào đúng về việc chọn loại review?

- a) Một work product chỉ được review bằng một loại review duy nhất.
- b) Một work product có thể được review bằng nhiều loại, ví dụ informal trước, hình thức hơn sau.
- c) Informal review luôn kém hiệu quả nên chỉ dùng khi thiếu người.
- d) Loại review được chọn chỉ dựa vào kích thước work product.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — syllabus nói ngược lại. <b>b đúng</b> — đúng ví dụ ở mục 3.2.4 (và tình huống US-CANCEL-07). <b>c sai</b> — informal review nhanh, rẻ, vẫn tìm được anomaly; vấn đề là chọn đúng chỗ. <b>d sai</b> — còn mục tiêu, rủi ro, nguồn lực, lĩnh vực, văn hoá công ty…</p>
</details>

**Câu 5 (K2).** Điều nào **không** phải là đặc điểm của informal review?

- a) Có thể diễn ra mà không có cuộc họp.
- b) Người tham gia có thể ghi chú nếu muốn.
- c) Author không được giữ vai trò review leader hoặc scribe.
- d) Mục tiêu chính là phát hiện anomaly.

<details>
<summary><b>Đáp án: c</b></summary>
<p><b>a</b> và <b>b</b> đều đúng với informal review — không có quy trình hay đầu ra bắt buộc, nhưng không cấm ghi chú. <b>c</b> là ràng buộc của <b>inspection</b> (mục 3.2.4), không áp cho informal review ⇒ đây là đáp án. <b>d</b> đúng là mục tiêu chính của informal review.</p>
</details>

**Câu 6 (K2).** Đội Tới Nơi chỉ dùng informal review cho mọi đặc tả, kể cả đặc tả thu tiền qua VíXanh. Rủi ro nào lớn nhất của cách làm này?

- a) Tốn quá nhiều thời gian chuẩn bị.
- b) Độ phủ phụ thuộc người đọc và không có dấu vết để giải thích quyết định về tiền sau này.
- c) Author không được tham gia.
- d) Không thể tìm được bất kỳ anomaly nào.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — informal review gần như không có chuẩn bị. <b>b đúng</b> — hai giới hạn chính: phụ thuộc người đọc và không có đầu ra chính thức (audit trail), nặng nhất với work product về tiền. <b>c sai</b> — author có thể tham gia. <b>d sai</b> — informal review vẫn tìm được anomaly; vấn đề là độ sâu và dấu vết.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB® Certified Tester Foundation Level Syllabus **v4.0.1** (2024): chương 3, mục 3.2.4 Review Types (đối chiếu 3.2.2 cho quy trình chung). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (tra *informal review*, *formal review*, *review*, *anomaly*).
- Trang chủ ISTQB: https://www.istqb.org/
- ISO/IEC 20246:2017 — *Software and systems engineering — Work product reviews* (quy trình review chung mà các loại review được may đo từ đó).
- Đặc tả lab Tới Nơi (`SPEC.md` mục 10 R-CANCEL, R-API-11) và tài liệu của bài trong `articles/informal-review/`: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-021 · Static Testing là gì?](/articles/static-testing-la-gi) · [TEST-027 · Review Process và các vai trò](/articles/review-process-va-cac-vai-tro) · [TEST-024 · Walkthrough](/articles/walkthrough) · [TEST-026 · Inspection](/articles/inspection-review-hinh-thuc) · [TEST-009 · Vai trò trong kiểm thử](/articles/vai-tro-trong-kiem-thu) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)

Bài tiếp theo: [Walkthrough](/articles/walkthrough).
