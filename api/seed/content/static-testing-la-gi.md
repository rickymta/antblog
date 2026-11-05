---
slug: static-testing-la-gi
title: "Static Testing là gì? Review đặc tả tính cước trước khi viết code"
summary: "Static testing kiểm tra work product mà không chạy phần mềm: review và static analysis. Bài nêu những gì kiểm tĩnh được, giá trị của việc tìm defect sớm, và luyện tìm 12 defect trong bản nháp đặc tả cước Tới Nơi."
category: testing
level: intermediate
tags: [testing, istqb, ctfl, test-021, static-testing, review, static-analysis, toi-noi]
language: vi
---

Nhóm Pricing của Tới Nơi vừa gửi bản nháp đặc tả cước. Chưa có dòng code nào, chưa chạy được test nào — vậy tester làm được gì? Câu trả lời của ISTQB là **static testing** (kiểm thử tĩnh): đánh giá sản phẩm công việc mà không thực thi phần mềm. Bài này giải thích static testing là gì, những thứ nào kiểm tĩnh được, vì sao nó đáng làm, rồi cho bạn tự tìm defect trong một bản nháp đặc tả cước có lỗi gài sẵn.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-021</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S03 · Static Testing &amp; Review Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Static testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/error-defect-failure-root-cause">TEST-004 · Error, Defect, Failure, Root Cause</a> · <a href="/articles/quy-trinh-kiem-thu-test-process">TEST-007 · Quy trình kiểm thử</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (đọc SPEC): review bản nháp đặc tả cước v0.3 có 12 defect gài sẵn, đối chiếu SPEC R-FARE-01…09 — <code>examples/toi-noi-testing/articles/static-testing-la-gi/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 3 · mục 3.1, 3.1.1, 3.1.2 · FL-3.1.1 (K1), FL-3.1.2 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Nhiều người mới vào nghề nghĩ kiểm thử là «chạy phần mềm rồi xem kết quả». Đó mới là một nửa — **dynamic testing** (kiểm thử động). Nửa còn lại diễn ra khi chưa có gì để chạy: đọc đặc tả, soi user story, rà code, dùng công cụ quét mã nguồn.

Vấn đề static testing giải quyết rất cụ thể. Câu đặc tả «trời mưa cộng phụ phí 8.000 đ» đi qua developer (viết code) và tester (viết test) — nếu cả hai cùng hiểu sai một kiểu thì test xanh, sản phẩm lên production vẫn sai. Chỉ người **đọc** đặc tả và hỏi «phụ phí mưa có bị nhân hệ số cao điểm không?» mới chặn được nó.

Khi nào dùng: ngay khi có một work product (sản phẩm công việc) đọc được — bản nháp yêu cầu, user story, thiết kế API, mã nguồn, cả bộ test case. Càng sớm càng rẻ.

## 2. Lý thuyết theo ISTQB

### 2.1. Định nghĩa

Theo CTFL v4.0.1 mục 3.1, static testing là kiểm thử mà **phần mềm được kiểm không cần được thực thi**. Work product được đánh giá theo hai cách:

- **thủ công** — điển hình là **review** (rà soát): con người đọc và đánh giá;
- **bằng công cụ** — điển hình là **static analysis** (phân tích tĩnh): công cụ phân tích cấu trúc mà không chạy.

Mục tiêu của static testing gồm cải thiện chất lượng, phát hiện defect, và đánh giá các đặc tính như **dễ đọc, đầy đủ, đúng, kiểm thử được, nhất quán**. Static testing dùng được cho cả verification lẫn validation (xem [TEST-005](/articles/verification-va-validation)).

| | Review | Static analysis |
|---|---|---|
| Ai làm | Con người (tester, BA, PO, developer…) | Công cụ, người đọc kết quả |
| Đầu vào cần gì | Đọc và hiểu được | Có cấu trúc hình thức để đối chiếu (mã, mô hình, văn bản có cú pháp) |
| Mạnh ở | Ý nghĩa, nghiệp vụ, chỗ thiếu, chỗ mâu thuẫn | Lỗi mã cụ thể, độ phức tạp, maintainability, một số lỗ hổng bảo mật |
| Ở Tới Nơi | Review đặc tả cước, user story, hợp đồng API | Bước 1 «Lint/static analysis» trong pipeline CI của mỗi service (kiến trúc §17) |

Syllabus nhấn mạnh hai điều về static analysis: nó tìm được vấn đề **trước** dynamic testing và thường tốn ít công hơn, vì không cần viết test case; và nó hay được đưa vào CI. Trình kiểm chính tả, công cụ đo độ dễ đọc cũng là static analysis.

Static testing còn diễn ra trong các buổi làm việc chung: tester, PO, BA và developer cùng làm example mapping, cùng viết user story, cùng refinement backlog để story đạt các tiêu chí như **Definition of Ready**. Tester đóng góp bằng cách đặt đúng câu hỏi để story đầy đủ, dễ hiểu và có acceptance criteria kiểm thử được. Phần này đi sâu ở [TEST-030 · Shift-left Testing](/articles/shift-left-testing).

### 2.2. Những work product kiểm tĩnh được (FL-3.1.1)

Gần như mọi work product đều kiểm tĩnh được. Syllabus liệt kê ví dụ: tài liệu đặc tả yêu cầu, mã nguồn, test plan, test case, product backlog item, test charter, tài liệu dự án, hợp đồng và mô hình.

Điều kiện khác nhau theo cách kiểm:

- **Review**: bất cứ thứ gì **đọc và hiểu được**.
- **Static analysis**: work product phải có **cấu trúc** để công cụ đối chiếu — mô hình, mã, văn bản có cú pháp hình thức.
- **Không phù hợp**: thứ con người khó diễn giải **và** không nên đưa cho công cụ phân tích — ví dụ mã thực thi của bên thứ ba, vì lý do pháp lý.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Ba cột work product của Tới Nơi. Cột 1, review được vì con người đọc hiểu: đặc tả SPEC, user story, acceptance criteria; test plan, test case, test charter; hợp đồng với VíXanh, MapViet; mô hình máy trạng thái chuyến; tài liệu dự án, runbook. Cột 2, review và static analysis vì có cấu trúc hình thức: mã TypeScript fare.ts; OpenAPI của POST /api/trips; tệp .proto GetQuote; schema sự kiện Kafka và schema.sql; văn bản qua công cụ kiểm chính tả. Cột 3, không phù hợp: SDK nhị phân VíXanh gửi kèm mà giấy phép cấm dịch ngược. Dải dưới: cả ba cột đều không chạy phần mềm; dynamic testing chỉ áp cho thứ chạy được.">
<text x="460" y="24" text-anchor="middle" class="t-b">Work product của Tới Nơi và cách kiểm tĩnh phù hợp</text>
<rect x="16" y="40" width="284" height="186" rx="8" class="f-pk-soft s-pk"/>
<text x="158" y="64" text-anchor="middle" class="t-b">Review được</text>
<text x="158" y="82" text-anchor="middle" class="t-small">con người đọc và hiểu được</text>
<text x="32" y="110" class="t-small">• Đặc tả SPEC, user story, AC</text>
<text x="32" y="132" class="t-small">• Test plan, test case, test charter</text>
<text x="32" y="154" class="t-small">• Hợp đồng với VíXanh, MapViet</text>
<text x="32" y="176" class="t-small">• Mô hình máy trạng thái chuyến</text>
<text x="32" y="198" class="t-small">• Tài liệu dự án, runbook</text>
<rect x="318" y="40" width="284" height="186" rx="8" class="f-phys-soft s-phys"/>
<text x="460" y="64" text-anchor="middle" class="t-b t-phys">Review + static analysis</text>
<text x="460" y="82" text-anchor="middle" class="t-small">có cấu trúc, cú pháp hình thức</text>
<text x="334" y="110" class="t-small t-mono">• src/domain/fare.ts</text>
<text x="334" y="132" class="t-small">• OpenAPI của POST /api/trips</text>
<text x="334" y="154" class="t-small">• .proto GetQuote (gRPC nội bộ)</text>
<text x="334" y="176" class="t-small">• Schema sự kiện Kafka, schema.sql</text>
<text x="334" y="198" class="t-small">• Văn bản qua công cụ kiểm chính tả</text>
<rect x="620" y="40" width="284" height="186" rx="8" class="f-warn-soft s-warn"/>
<text x="762" y="64" text-anchor="middle" class="t-b t-warn">Không phù hợp</text>
<text x="762" y="82" text-anchor="middle" class="t-small">khó diễn giải, không được phân tích</text>
<text x="636" y="110" class="t-small">• SDK nhị phân VíXanh gửi kèm</text>
<text x="636" y="130" class="t-small">  (giấy phép cấm dịch ngược)</text>
<text x="636" y="160" class="t-small">Kiểm gián tiếp: review hợp đồng</text>
<text x="636" y="180" class="t-small">tích hợp, chạy test qua sandbox</text>
<rect x="16" y="242" width="888" height="44" rx="8" class="f-surface s-line"/>
<text x="460" y="262" text-anchor="middle" class="t-small">Cả ba cột: KHÔNG chạy phần mềm. Dynamic testing chỉ áp được cho thứ chạy được —</text>
<text x="460" y="278" text-anchor="middle" class="t-small">mã đã build, dịch vụ, toàn hệ thống (xem TEST-022).</text>
</svg>
</div>
<figcaption><b>Hình 1 · Ba nhóm work product theo cách kiểm tĩnh.</b> Ranh giới giữa cột 1 và cột 2 là có cấu trúc hình thức hay không. Cột 3 không phải «không cần kiểm» mà là kiểm bằng đường khác.</figcaption>
</figure>

### 2.3. Giá trị của static testing (FL-3.1.2)

| Giá trị theo syllabus | Ở Tới Nơi |
|---|---|
| Phát hiện defect ở những pha sớm nhất của SDLC — đúng nguyên lý kiểm thử sớm | Sửa một câu trong bản nháp đặc tả thay vì sửa code, test, dữ liệu báo giá đã lưu |
| Tìm được defect mà dynamic testing không tìm được: mã không bao giờ chạy tới, mẫu thiết kế không được cài đúng ý, defect trong work product không thực thi được | Một ví dụ tính sai trong đặc tả — không test nào «chạy» được một đoạn văn |
| Đánh giá chất lượng và tạo niềm tin vào work product | Đặc tả cước được review xong mới được gắn `pricingVersion` mới |
| Qua việc kiểm yêu cầu, stakeholder chắc chắn yêu cầu mô tả đúng nhu cầu thật (validation) | Hỏi đội vận hành tài xế: phụ phí mưa có nên bị nhân cao điểm không? |
| Tạo hiểu biết chung, cải thiện giao tiếp giữa các bên — nên mời nhiều loại stakeholder | BA, developer Pricing, tester, CSKH cùng hiểu một công thức |
| Review tốn chi phí, nhưng tổng chi phí dự án thường thấp hơn vì sửa defect muộn tốn công hơn | Xem ước tính ở mục 3 |
| Static analysis tìm một số loại defect mã hiệu quả hơn dynamic testing ⇒ ít defect hơn, tổng công phát triển thấp hơn | ESLint chặn biến dùng trước khi gán ngay trong CI |

### 2.4. Thuật ngữ Anh – Việt

| Tiếng Anh | Tiếng Việt | Ghi chú |
|---|---|---|
| static testing | kiểm thử tĩnh | không thực thi phần mềm |
| dynamic testing | kiểm thử động | có thực thi phần mềm |
| review | rà soát | hình thức thủ công của static testing |
| static analysis | phân tích tĩnh | hình thức dùng công cụ |
| work product | sản phẩm công việc | đặc tả, code, test case, hợp đồng… |
| anomaly | bất thường | điều khác kỳ vọng — chưa chắc là defect |
| test basis | cơ sở kiểm thử | nguồn để suy ra test, như SPEC |
| Definition of Ready | tiêu chí sẵn sàng | story đủ điều kiện để đưa vào sprint |
| backlog refinement | tinh chỉnh backlog | buổi làm rõ story trước sprint |
| testability | khả năng kiểm thử được | có tiêu chí đạt/không đạt rõ |

**Phạm vi bài.** Bài này dừng ở «static testing là gì, kiểm được gì, vì sao đáng làm». So sánh chi tiết với dynamic testing ở [TEST-022](/articles/static-testing-va-dynamic-testing); quy trình review và vai trò ở [TEST-027](/articles/review-process-va-cac-vai-tro); các loại review ở [TEST-023](/articles/informal-review) → [TEST-026](/articles/inspection-review-hinh-thuc); static analysis bằng công cụ ở [TEST-029](/articles/static-analysis-phan-tich-code-tinh).

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Các công cụ nhắc trong bài — ESLint, trình biên dịch TypeScript (<code>tsc</code>), trình lint OpenAPI hay <code>.proto</code>, trình kiểm chính tả — là ví dụ thực tiễn ngành. CTFL chỉ nói tới static analysis tool nói chung (chương 6), không quy định công cụ nào.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Pricing & Quote Service (kiến trúc §4.3) tính giá, tạo báo giá có hạn với `quote_id`, `expires_at`, `pricing_version` và bảng chi tiết, rồi lưu ảnh chụp giá để không đổi giá hồi tố. Trước khi code bộ quy tắc mới, nhóm Pricing soạn **đặc tả cước bản nháp v0.3** và gửi cho tester, developer, PO.

### Business rules (nguồn đã chốt)

Biên bản chốt nghiệp vụ chính là SPEC của lab, nhóm R-FARE:

- **R-FARE-03** — thứ tự cố định: cước cơ bản → × cao điểm → + mưa → − voucher → nâng lên cước tối thiểu → làm tròn lên 1.000 đ.
- **R-FARE-02** — quãng đường làm tròn 0,1 km half-up **trước** mọi bước, hợp lệ trong [0,5; 100] km tính cả hai đầu.
- **R-FARE-04/05** — cao điểm [11:00; 13:00) và [17:00; 19:00) giờ Việt Nam ×1,25; mưa +8.000 đ **không bị nhân** 1,25.
- **R-FARE-07/08** — voucher không kéo cước dưới mức tối thiểu; chỉ làm tròn ở bước cuối (R-GEN-01).
- **R-FARE-09** — ví dụ kiểm: xe máy 7,4 km, cao điểm, mưa, voucher 20% ⇒ **52.000 đ**.

### System context

Booking gọi Pricing qua gRPC (`.proto` là hợp đồng, kiến trúc §7.2); client gọi REST qua API Gateway; báo giá được lưu làm ảnh chụp. Trong ngữ cảnh đó, những work product cần kiểm tĩnh trải từ đặc tả, OpenAPI, `.proto` tới mã `fare.ts` và bộ test (Hình 1). Bản lab là modular monolith nên không có gRPC hay API Gateway thật — ví dụ `.proto` chỉ để minh hoạ.

### Rủi ro có thể xảy ra

| Rủi ro | Vì sao test động không cứu |
|---|---|
| Đặc tả bỏ ngỏ thứ tự cao điểm – mưa, developer nhân cả phụ phí mưa | Tester viết test từ cùng đặc tả ⇒ test và code cùng sai, test xanh |
| Biên 100 km mâu thuẫn giữa hai câu | Mỗi bên chọn một câu, hai bản code khác nhau đều «đúng đặc tả» |
| Giá sai được lưu vào báo giá và chuyến (R-QUOTE-04) | Sửa code xong vẫn còn dữ liệu giá sai đã chụp, phải đối soát |

**Ước tính chi phí nếu lọt (giả định).** Defect «phụ phí mưa bị nhân 1,25» làm thu thừa khoảng 8.000 × 0,25 = **2.000 đ** mỗi chuyến cao điểm có mưa (đúng con số ở ví dụ R-FARE-09: 54.000 thay vì 52.000 đ). Hồ sơ dự án cho 1,1 triệu chuyến đặt xe + bưu kiện mỗi ngày, hai khung cao điểm chiếm 50% chuyến. **Giả định** trong một ngày mưa, 20% chuyến cao điểm gặp mưa:

```text
1.100.000 × 50% × 20% = 110.000 chuyến × 2.000 đ ≈ 220 triệu đ thu thừa mỗi ngày mưa
```

Chưa tính hoàn tiền, CSKH và uy tín. Còn chi phí chặn nó ở bản nháp: một câu hỏi trong buổi review.

## 4. Phân tích kỹ thuật

### 4.1. Biến mục tiêu thành câu hỏi review

Năm đặc tính syllabus nêu ở mục 3.1 là bộ câu hỏi tốt để đọc một đặc tả:

| Đặc tính | Câu hỏi khi đọc đặc tả cước |
|---|---|
| Dễ đọc (readability) | Một người mới đọc có tính được ví dụ R-FARE-09 mà không phải hỏi ai? |
| Đầy đủ (completeness) | Mọi bước có thứ tự chưa? Mỗi quy tắc có nói làm gì ở biên không? |
| Đúng (correctness) | Có khớp biên bản chốt nghiệp vụ không? Ví dụ có tính đúng không? |
| Kiểm thử được (testability) | Có viết được tiêu chí đạt/không đạt cho từng câu không? «Nhanh» là bao nhiêu? |
| Nhất quán (consistency) | Cùng một giá trị có bị ghi hai nơi, hai số khác nhau không? |

### 4.2. Những kiểu defect hay gặp trong đặc tả

Syllabus (mục 3.1.3) liệt kê các defect yêu cầu dễ tìm bằng static testing: **không nhất quán, mơ hồ, mâu thuẫn, thiếu, không chính xác, trùng lặp**. Dùng danh sách này làm «lưới» khi đọc, kèm thêm **không kiểm thử được** từ đặc tính testability. Bài sau ([TEST-022](/articles/static-testing-va-dynamic-testing)) so sánh vì sao các loại này rẻ hơn khi tìm bằng static testing.

### 4.3. Đối chiếu với cái gì

Một phát hiện chỉ thành defect khi có căn cứ. Bốn nguồn đối chiếu thường dùng:

1. **Chính tài liệu** — câu này có mâu thuẫn câu kia không (D-03 «dưới 100 km» vs D-13 «tối đa 100 km»).
2. **Nguồn cấp trên** — biên bản chốt nghiệp vụ, ở đây là SPEC R-FARE.
3. **Ví dụ kiểm (oracle)** — tự tính lại từng bước của ví dụ trong tài liệu.
4. **Stakeholder** — khi chính nguồn cấp trên chưa nói, hỏi người có nhu cầu (validation).

Phát hiện chưa có căn cứ thì ghi là **câu hỏi** (anomaly), không gán vội là defect.

## 5. Thực hành

### 5.1. Bản nháp cần review

Tệp đầy đủ: `examples/toi-noi-testing/articles/static-testing-la-gi/ban-nhap-dac-ta-cuoc-v0.3.md`.

| Dòng | Nội dung |
|---|---|
| D-01 | Cước gồm phí mở cửa và tiền theo km. Bảng giá: xe máy — mở cửa 12.000 đ, 4.500 đ/km, tối thiểu 15.000 đ; ô tô — mở cửa 25.000 đ, 11.000 đ/km, tối thiểu 30.000 đ. |
| D-02 | Quãng đường do khách gửi được làm tròn tới 0,1 km. |
| D-03 | Quãng đường hợp lệ từ 0,5 km đến dưới 100 km; ngoài khoảng báo lỗi `DISTANCE_OUT_OF_RANGE`. |
| D-04 | Giờ cao điểm 11h–13h và 17h–19h: cước nhân 1,25. |
| D-05 | Khi trời mưa cộng phụ phí 8.000 đ. |
| D-06 | Voucher giảm theo phần trăm của cước, không vượt trần của voucher. |
| D-07 | Cước tối thiểu được áp trước khi trừ voucher. |
| D-08 | Tiền được làm tròn tới 1.000 đ. |
| D-09 | Bưu kiện trên 5 kg cộng 2.000 đ cho mỗi kg vượt. |
| D-10 | Ô tô có cước tối thiểu 35.000 đ. |
| D-11 | Hệ thống phải trả báo giá nhanh. |
| D-12 | Ví dụ: xe máy 7,4 km, cao điểm, trời mưa, voucher 20%: 12.000 + 4.500 × 7,4 = 45.300 → × 1,25 = 56.625 → + 8.000 = 64.625 → − 20% = 51.900 → 52.000 đ. |
| D-13 | Một chuyến dài tối đa 100 km. |

### 5.2. Cách làm

1. Hẹn giờ 20 phút. Không chạy gì, không mở mã nguồn.
2. Đọc từng dòng với bảng câu hỏi ở mục 4.1. Ghi mỗi phát hiện theo mẫu: *dòng · loại · mô tả · đối chiếu SPEC · hậu quả nếu lọt*.
3. Tự tính lại ví dụ D-12 từng bước.
4. **Kết quả mong đợi**: ít nhất 5 defect, trong đó có ít nhất một mâu thuẫn và một chỗ thiếu thứ tự tính.

### 5.3. Lời giải

<details>
<summary><b>Nhật ký review mẫu — 12 defect (mở sau khi tự làm)</b></summary>
<table>
<thead><tr><th>#</th><th>Dòng</th><th>Loại</th><th>Phát hiện</th><th>SPEC</th><th>Hậu quả nếu lọt</th></tr></thead>
<tbody>
<tr><td>F1</td><td>D-02</td><td>Mơ hồ</td><td>Không nói kiểu làm tròn</td><td>R-FARE-02 half-up</td><td>0,45 km bị cắt thành 0,4 ⇒ báo lỗi</td></tr>
<tr><td>F2</td><td>D-02, D-03</td><td>Thiếu</td><td>Làm tròn trước hay sau khi kiểm khoảng?</td><td>R-FARE-02: trước</td><td>100,04 km hợp lệ hay lỗi tuỳ người code</td></tr>
<tr><td>F3</td><td>D-03 ↔ D-13</td><td>Mâu thuẫn</td><td>«dưới 100» vs «tối đa 100»</td><td>R-FARE-02 [0,5; 100]</td><td>Chuyến đúng 100,0 km bị từ chối</td></tr>
<tr><td>F4</td><td>D-04</td><td>Mơ hồ + thiếu</td><td>13:00 có là cao điểm? Giờ máy chủ hay giờ VN?</td><td>R-FARE-04, R-GEN-02</td><td>Máy chủ chạy UTC ⇒ cao điểm lệch 7 giờ</td></tr>
<tr><td>F5</td><td>D-04, D-05</td><td>Thiếu</td><td>Phụ phí mưa có bị nhân 1,25?</td><td>R-FARE-03, R-FARE-05</td><td>Ví dụ R-FARE-09: 54.000 đ thay vì 52.000 đ</td></tr>
<tr><td>F6</td><td>D-06</td><td>Mơ hồ</td><td>«phần trăm của cước» — cước ở bước nào?</td><td>R-FARE-06: sau bước 3</td><td>Giảm trên cước cơ bản ⇒ khách được giảm ít hơn</td></tr>
<tr><td>F7</td><td>D-07</td><td>Sai</td><td>Tối thiểu trước voucher ⇒ voucher kéo cước dưới mức tối thiểu</td><td>R-FARE-07</td><td>Xe máy 0,5 km, voucher 20%, ngoài cao điểm: 12.000 đ thay vì 15.000 đ</td></tr>
<tr><td>F8</td><td>D-08</td><td>Mơ hồ + thiếu</td><td>Làm tròn lên/xuống/gần nhất? Ở bước nào?</td><td>R-GEN-01, R-FARE-08</td><td>Làm tròn gần nhất ở mọi bước: ví dụ R-FARE-09 ra 51.000 đ</td></tr>
<tr><td>F9</td><td>D-09</td><td>Mơ hồ + thiếu</td><td>Kg chưa trọn tính sao? Phụ phí ở bước nào?</td><td>R-PARCEL-03/04</td><td>5,1 kg tính 0 đ thay vì 2.000 đ</td></tr>
<tr><td>F10</td><td>D-01 ↔ D-10</td><td>Trùng lặp + mâu thuẫn</td><td>Tối thiểu ô tô 30.000 hay 35.000?</td><td>R-FARE-01: 30.000</td><td>Hai developer chọn hai số</td></tr>
<tr><td>F11</td><td>D-11</td><td>Không kiểm thử được</td><td>«nhanh» không có số đo</td><td>R-NFR-03: p95 dưới 500 ms</td><td>Không có tiêu chí đạt/không đạt</td></tr>
<tr><td>F12</td><td>D-12</td><td>Sai</td><td>64.625 − 20% = 51.700, không phải 51.900</td><td>R-FARE-09</td><td>Người đọc nghi ngờ công thức đúng</td></tr>
</tbody>
</table>
<p>Cách tính các con số: F7 — SPEC: 12.000 + 4.500 × 0,5 = 14.250, giảm 2.850 còn 11.400, nâng lên 15.000 đ; bản nháp: nâng lên 15.000 trước, giảm 3.000 còn 12.000 đ. F8 — làm tròn gần nhất ở mỗi bước: 45.300 → 45.000; × 1,25 = 56.250 → 56.000; + 8.000 = 64.000; − 12.800 = 51.200 → 51.000 đ. F5 — (45.300 + 8.000) × 1,25 = 66.625; − 13.325 = 53.300 → 54.000 đ.</p>
</details>

### 5.4. Kết quả thực tế và nhận xét

Nhật ký mẫu ghi **12 defect** và một **câu hỏi đã đóng** («voucher có điều kiện đơn tối thiểu không?» — đã có ở đặc tả voucher riêng R-VOUCHER-06). Ba điều đáng để ý:

- **F12 bị che**: kết quả cuối 52.000 đ vẫn đúng vì làm tròn lên che mất lỗi số học.
- **F5 nguy hiểm nhất**: không câu nào sai, chỉ thiếu. Chỉ thấy khi tự hỏi «hai bước này theo thứ tự nào?».
- **F7 cần nguồn cấp trên**: đọc riêng bản nháp thì D-07 hợp lý; chỉ sai khi so với nghiệp vụ đã chốt — lý do mời PO vào review.

[TEST-022](/articles/static-testing-va-dynamic-testing) giả định F5 đã lọt vào mã, rồi so sánh tìm nó bằng đọc code với tìm bằng một test đỏ.

## 6. Góc nhìn nâng cao

**«Chưa chạy thì chưa phải test.»** Sai theo ISTQB. Review đặc tả là kiểm thử, có mục tiêu, có kết quả (defect, câu hỏi), có đầu ra (nhật ký phát hiện).

**Test xanh trên đặc tả sai.** Đây là điểm mù lớn nhất của dynamic testing: test case suy ra từ test basis, nên test basis sai thì test «đúng» theo cái sai. Static testing trên test basis là cách chính để chặn lớp lỗi này. Cùng lý do, bộ test case cũng phải được review — thiếu test cho một acceptance criterion là defect kiểm tĩnh được.

**Phát hiện ≠ defect.** Một dòng trong nhật ký là anomaly cho tới khi được phân tích. Gán vội mọi câu hỏi là defect làm tác giả phòng thủ và làm số liệu defect sai lệch.

**Static analysis cho tài liệu.** Văn tự do khó phân tích bằng công cụ. Khi đặc tả có cấu trúc — mỗi quy tắc một mã `R-…`, có bảng, có ví dụ — một script nhỏ đã kiểm được «mã nào bị trùng», «quy tắc nào chưa có ví dụ».

**Giới hạn.** Static testing không cho biết hệ thống chạy ra sao: không đo được độ trễ p95 của Quote API, không thấy được lỗi làm tròn số thực chỉ lộ ở một vài giá trị. Hai cách bổ trợ nhau — chủ đề của [TEST-022](/articles/static-testing-va-dynamic-testing).

**Sai lầm thường gặp:**

- Review chỉ soi chính tả, bỏ qua thứ tự và biên.
- Review khi đặc tả đã «đóng băng» và code đã viết xong — mất phần lớn giá trị phát hiện sớm.
- Chỉ tester review; không mời PO, BA, developer, vận hành — mất phần validation và hiểu biết chung.
- Review cả tài liệu 50 trang trong một lần. Syllabus lưu ý work product lớn nên chia nhiều lượt review (quy trình ở [TEST-027](/articles/review-process-va-cac-vai-tro)).
- Biến buổi review thành buổi phê bình tác giả.

## 7. Bài tập và câu hỏi ôn tập

**Bài 1.** Xếp mỗi work product vào *review được*, *review + static analysis* hoặc *không phù hợp cho static testing*: (a) acceptance criteria của story «khách huỷ chuyến»; (b) `src/domain/cancel.ts`; (c) tệp `.proto` của Pricing; (d) thư viện nhị phân bản đồ MapViet gửi kèm, giấy phép cấm dịch ngược; (e) test plan Sprint 12; (f) `schema.sql` của kho PGlite; (g) test charter khám phá luồng thanh toán.

<details>
<summary><b>Lời giải</b></summary>
<p>Review được: (a), (e), (g). Review + static analysis: (b), (c), (f) — có cú pháp hình thức nên công cụ đối chiếu được. Không phù hợp: (d) — kiểm gián tiếp bằng review hợp đồng tích hợp và dynamic testing qua sandbox. Lưu ý (a), (e), (g) vẫn có thể qua kiểm chính tả bằng công cụ, nhưng phần giá trị chính là review.</p>
</details>

**Bài 2.** Review nhanh story bưu kiện sau, đối chiếu R-PARCEL-01…04, tìm ít nhất 3 defect.

```text
Là khách gửi bưu kiện, tôi muốn biết phụ phí khối lượng.
AC1: Khối lượng từ 0 đến 30 kg.
AC2: Hàng nặng phải chọn ô tô.
AC3: Mỗi kg trên 5 kg cộng 2.000 đ; phụ phí cộng sau bước cao điểm.
AC4: Khối lượng làm tròn một chữ số thập phân.
```

<details>
<summary><b>Lời giải</b></summary>
<p>AC1 — sai biên dưới: SPEC là 0 &lt; w ≤ 30 (0 kg không hợp lệ); không nói làm tròn trước khi kiểm. AC2 — không kiểm thử được: «nặng» là bao nhiêu? SPEC: w &gt; 20 kg bắt buộc ô tô, 20,0 kg vẫn đi xe máy. AC3 — sai vị trí: R-PARCEL-04 cộng vào cước cơ bản nên <b>bị nhân</b> cao điểm; thiếu luật kg chưa trọn (R-PARCEL-03: mỗi kg bắt đầu). AC4 — thiếu kiểu làm tròn (R-PARCEL-01: half-up 0,1 kg).</p>
</details>

**Bài 3.** PM nói: «Bỏ review đặc tả đi, cuối sprint chạy test là bắt hết.» Viết ba lập luận phản biện dựa trên mục 3.1.2.

<details>
<summary><b>Lời giải</b></summary>
<p>(1) Test suy ra từ đặc tả: đặc tả sai thì test sai theo, như F5 — chạy bao nhiêu cũng xanh. (2) Có defect dynamic testing không tìm được: ví dụ tính sai F12, mâu thuẫn F3 trong tài liệu. (3) Sửa ở bản nháp chỉ là sửa một câu; sửa cuối sprint là sửa code, test, dữ liệu báo giá đã lưu — tổng chi phí cao hơn chi phí review (ước tính ở mục 3). Thêm: review tạo hiểu biết chung giữa Pricing, tester và CSKH.</p>
</details>

**Câu 1 (K1).** Work product nào **ít phù hợp nhất** cho static testing?

- a) Tệp `.proto` của dịch vụ GetQuote
- b) Test charter khám phá luồng huỷ chuyến
- c) Thư viện thực thi nhị phân của VíXanh, giấy phép cấm dịch ngược
- d) Hợp đồng tích hợp với MapViet

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — syllabus nêu mã thực thi của bên thứ ba là ví dụ không phù hợp: con người khó diễn giải và không nên đưa cho công cụ phân tích vì lý do pháp lý. a) có cú pháp hình thức, review và phân tích được. b) và d) đọc hiểu được nên review được — syllabus liệt kê cả test charter và hợp đồng.</p>
</details>

**Câu 2 (K1).** Work product nào dưới đây đáp ứng điều kiện để dùng **static analysis**, không chỉ review?

- a) Ghi chú tay sau buổi phỏng vấn tài xế
- b) Tệp OpenAPI mô tả `POST /api/trips`
- c) Ý tưởng tính năng vẽ trên bảng trắng
- d) Email PO mô tả mong muốn bằng lời

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — static analysis cần cấu trúc để công cụ đối chiếu, như văn bản có cú pháp hình thức. a), c), d) chỉ review được (nếu đọc hiểu được).</p>
</details>

**Câu 3 (K2).** Điều nào là **giá trị của static testing** theo CTFL v4.0.1?

- a) Đo được độ trễ p95 của Quote API
- b) Chứng minh mã không còn defect
- c) Thay thế được dynamic testing nếu review đủ kỹ
- d) Tìm được defect trong work product không thực thi được, như một ví dụ tính sai trong đặc tả

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>d)</b> đúng — mục 3.1.2 nêu static testing tìm được defect mà dynamic testing không tìm được, gồm defect trong work product không thực thi được. a) độ trễ phụ thuộc thực thi — việc của dynamic testing. b) kiểm thử không chứng minh được không còn defect (nguyên lý 1, <a href="/articles/bay-nguyen-ly-kiem-thu">TEST-003</a>). c) syllabus nói hai cách bổ trợ nhau.</p>
</details>

**Câu 4 (K2).** PO than review đặc tả tốn thời gian. Phát biểu nào phù hợp với syllabus?

- a) Review có chi phí, nhưng tổng chi phí dự án thường thấp hơn vì bớt công sửa defect về sau
- b) Review luôn rẻ hơn dynamic testing ở mọi dự án
- c) Chỉ nên review sau khi có sự cố production
- d) Review chỉ có ích cho tài liệu, không dùng cho mã nguồn

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a)</b> đúng — sát ý mục 3.1.2. b) «luôn» là quá đà; syllabus nói «thường». c) đi ngược nguyên lý kiểm thử sớm. d) sai — mã nguồn là work product điển hình của review.</p>
</details>

**Câu 5 (K2).** Trong buổi refinement story «phụ phí mưa», tester hỏi: «Phụ phí có bị nhân hệ số cao điểm không?» và nhóm bổ sung acceptance criterion. Nhận định nào đúng?

- a) Chưa phải kiểm thử vì chưa có code
- b) Đây là debugging vì đang tìm nguyên nhân lỗi
- c) Đây là static testing: tester giúp story đầy đủ và có acceptance criteria kiểm thử được
- d) Đây là dynamic testing vì câu hỏi nói về hành vi hệ thống

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — mục 3.1 mô tả tester đặt câu hỏi trong refinement để story đạt tiêu chí như Definition of Ready. a) static testing không cần code. b) debugging tìm và sửa nguyên nhân failure trong mã. d) không có gì được thực thi.</p>
</details>

**Câu 6 (K2).** Nhóm review đặc tả phí huỷ cùng đại diện tài xế để xem mức phí bù có đúng điều tài xế cần không. Nhận định nào đúng?

- a) Static testing chỉ dùng cho verification, nên đây không phải static testing
- b) Đây là validation bằng static testing — syllabus nói static testing áp dụng cho cả verification lẫn validation
- c) Validation chỉ làm được ở acceptance testing
- d) Không phải kiểm thử vì không có test case

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — mục 3.1 và 3.1.2: qua việc kiểm yêu cầu, stakeholder chắc chắn yêu cầu mô tả đúng nhu cầu thật. a) và c) sai theo chính câu đó. d) review không cần test case.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024), chương 3 «Static Testing», mục 3.1 «Static Testing Basics», 3.1.1 «Work Products Examinable by Static Testing», 3.1.2 «Value of Static Testing»; LO FL-3.1.1 (K1), FL-3.1.2 (K2): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ — mục *static testing*, *dynamic testing*, *review*, *static analysis*, *anomaly*, *work product*.
- Trang chủ ISTQB: https://www.istqb.org/
- ISO/IEC 20246, *Software and systems engineering — Work product reviews* — chuẩn quy trình review mà CTFL mục 3.2.2 dẫn chiếu.
- Tài liệu kiến trúc Tới Nơi: §4.3 Pricing & Quote, §7.2 quy ước gRPC, §17 CI/CD.
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-FARE, R-PARCEL, R-GEN, R-NFR; tài liệu bài tại `articles/static-testing-la-gi/`.
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-003 · Bảy nguyên lý kiểm thử](/articles/bay-nguyen-ly-kiem-thu) · [TEST-004 · Error, Defect, Failure](/articles/error-defect-failure-root-cause) · [TEST-005 · Verification và Validation](/articles/verification-va-validation) · [TEST-007 · Quy trình kiểm thử](/articles/quy-trinh-kiem-thu-test-process) · [TEST-008 · Kiểm thử trong SDLC](/articles/kiem-thu-trong-sdlc-agile-waterfall).

Bài tiếp theo: [Static Testing và Dynamic Testing — đọc code hay chạy test để tìm lỗi tính cước](/articles/static-testing-va-dynamic-testing).
