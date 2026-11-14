---
slug: bay-nguyen-ly-kiem-thu
title: "Bảy nguyên lý kiểm thử (Seven Testing Principles): vì sao không thể test hết mọi tổ hợp đặt xe"
summary: "Bảy nguyên lý kiểm thử của CTFL v4.0.1, mỗi nguyên lý gắn với một tình huống khác nhau ở Tới Nơi; đếm thử không gian đầu vào của API báo giá để thấy vì sao kiểm thử toàn bộ là bất khả thi."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-003, nguyen-ly-kiem-thu, exhaustive-testing, defect-clustering, tests-wear-out]
language: vi
---

Một khách mở ứng dụng Tới Nơi, chọn dịch vụ, thành phố, loại xe, nhập quãng đường, gõ mã giảm giá rồi bấm «Báo giá». Có bao nhiêu cách khác nhau để làm việc đó, và đội kiểm thử phải thử bao nhiêu cách thì mới yên tâm? Bài này trả lời bằng **bảy nguyên lý kiểm thử** của ISTQB — bảy «luật chơi» quyết định ta nên kiểm thử cái gì, khi nào, nhiều tới đâu và tin kết quả tới mức nào. Mỗi nguyên lý được đặt vào một tình huống khác nhau của Tới Nơi, và nguyên lý thứ hai được chứng minh bằng một phép đếm cụ thể trên đặc tả của phòng lab.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-003</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S01 · Fundamentals of Software Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/software-testing-la-gi-testing-khac-debugging">TEST-001 · Software Testing là gì?</a>, <a href="/articles/vi-sao-can-kiem-thu">TEST-002 · Vì sao cần kiểm thử?</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (đọc SPEC): đếm tổ hợp đầu vào của <code>POST /api/quotes</code> (R-API-07) và chọn 10 tổ hợp đáng test nhất — mục 5 và bài tập 1</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.3 · FL-1.3.1 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

[TEST-001](/articles/software-testing-la-gi-testing-khac-debugging) đã nói kiểm thử là gì, [TEST-002](/articles/vi-sao-can-kiem-thu) nói vì sao cần nó. Câu hỏi tiếp theo rất thực tế: **làm kiểm thử thế nào cho khôn ngoan?** Thời gian và người luôn có hạn, còn số cách dùng một hệ thống như Tới Nơi thì gần như vô hạn.

Bảy nguyên lý kiểm thử không phải kỹ thuật thiết kế test. Chúng là những nhận định chung, đúng với hầu hết dự án, được rút ra qua nhiều thập kỷ làm phần mềm. Chúng giúp tránh các ảo tưởng phổ biến:

- «Chạy hết test xanh rồi, chắc không còn lỗi.»
- «Cứ thử đủ mọi trường hợp là xong.»
- «Code xong rồi tester mới vào.»
- «Bộ test hồi quy năm ngoái vẫn chạy tốt, không cần đụng.»
- «Đúng đặc tả 100% thì khách chắc chắn hài lòng.»

Khi nào dùng: mỗi lần lập kế hoạch test, ước lượng, chọn phạm vi, báo cáo kết quả hay quyết định release. Một kế hoạch test vi phạm một trong bảy nguyên lý thường có lỗ hổng mà nhìn vào số ca test không thấy được.

## 2. Lý thuyết theo ISTQB

Syllabus CTFL v4.0.1 mục 1.3 liệt kê bảy nguyên lý, kèm learning objective **FL-1.3.1 (K2) — giải thích bảy nguyên lý kiểm thử**. Mức K2 nghĩa là người học phải hiểu và giải thích được bằng ví dụ, không chỉ thuộc tên. Bảng dưới diễn đạt lại từng nguyên lý (không phải bản dịch nguyên văn).

| # | Tên trong CTFL v4.0.1 | Ý chính (diễn đạt lại) | Hệ quả khi làm việc |
|---|---|---|---|
| 1 | Testing shows the presence, not the absence of defects | Test tìm được defect thì chứng minh có defect; không tìm được thì chỉ làm giảm khả năng còn defect, không chứng minh phần mềm đúng | Báo cáo «chưa phát hiện lỗi trong phạm vi đã test», không báo «không có lỗi» |
| 2 | Exhaustive testing is impossible | Thử mọi tổ hợp đầu vào và điều kiện chỉ khả thi với đối tượng tầm thường | Dùng test technique, ưu tiên test case (test case prioritization) và kiểm thử dựa trên rủi ro (risk-based testing) để dồn sức |
| 3 | Early testing saves time and money | Defect gỡ sớm không sinh thêm defect ở các sản phẩm làm sau; ít failure muộn hơn nên chi phí chất lượng giảm | Bắt đầu cả static testing (review đặc tả, review code) lẫn dynamic testing càng sớm càng tốt |
| 4 | Defects cluster together | Phần lớn defect thường tập trung ở một số ít thành phần — một biểu hiện của nguyên lý Pareto | Dùng cụm defect dự đoán và cụm defect thực tế làm đầu vào cho risk-based testing |
| 5 | Tests wear out | Lặp lại mãi cùng một bộ test thì nó ngày càng kém trong việc tìm defect **mới** | Sửa test và dữ liệu cũ, viết test mới; nhưng hồi quy tự động lặp lại vẫn có ích cho mục đích của nó |
| 6 | Testing is context dependent | Không có một cách kiểm thử dùng cho mọi nơi | Chọn cách test theo rủi ro, ngành, quy định, công nghệ, mô hình phát triển |
| 7 | Absence-of-defects fallacy | Ngộ nhận rằng chỉ cần verification kỹ là hệ thống thành công | Ngoài verification phải có validation: hệ thống có đáp ứng nhu cầu người dùng và mục tiêu kinh doanh không |

Syllabus dẫn nguồn cho từng nguyên lý (Buxton 1970, Manna 1978, Boehm 1981, Enders 1975, Beizer 1990, Kaner 2011). Nguyên lý 1 thường được gắn với câu nói nổi tiếng của Dijkstra, được ghi lại trong kỷ yếu hội nghị kỹ thuật phần mềm NATO mà Buxton là đồng biên tập.

<div class="tkh-callout">
<span class="tkh-callout-title">Tên cũ của nguyên lý 5 và 7</span>
<p>Syllabus CTFL 2018 (v3.1) gọi nguyên lý 5 là <b>pesticide paradox</b> (nghịch lý thuốc trừ sâu: sâu «nhờn thuốc», bộ test «nhờn lỗi») và nguyên lý 7 là <b>absence-of-errors is a fallacy</b>. Bản v4.0.1 đổi thành <b>tests wear out</b> và <b>absence-of-defects fallacy</b>. Nhiều tài liệu và đề luyện cũ vẫn dùng tên cũ — nghĩa không đổi.</p>
</div>

### Thuật ngữ Anh – Việt

| Thuật ngữ (ISTQB) | Nghĩa tiếng Việt dùng trong blog | Ghi chú |
|---|---|---|
| testing principle | nguyên lý kiểm thử | Hướng dẫn chung, không phải quy trình bắt buộc |
| defect (bug, fault) | khiếm khuyết, lỗi trong sản phẩm | Chi tiết ở [TEST-004](/articles/error-defect-failure-root-cause) |
| exhaustive testing | kiểm thử toàn bộ / vét cạn | Thử mọi tổ hợp đầu vào và điều kiện tiên quyết |
| test technique | kỹ thuật kiểm thử | Phân vùng tương đương, giá trị biên… (CTFL chương 4) |
| test case prioritization | ưu tiên ca kiểm thử | CTFL mục 5.1.5 |
| risk-based testing | kiểm thử dựa trên rủi ro | CTFL mục 5.2 |
| static testing / dynamic testing | kiểm thử tĩnh / kiểm thử động | Không chạy code / có chạy code |
| defect clustering | sự tập trung defect | Hiện tượng Pareto |
| tests wear out (pesticide paradox) | test bị mòn (nghịch lý thuốc trừ sâu) | Tên cũ trong CTFL 2018 |
| regression testing | kiểm thử hồi quy | Lặp lại để phát hiện tác dụng phụ của thay đổi |
| verification / validation | xác minh / xác nhận | Đúng đặc tả / đúng nhu cầu — [TEST-005](/articles/verification-va-validation) |
| absence-of-defects fallacy | ngộ nhận «không có defect là thành công» | |

**Phạm vi áp dụng.** Bảy nguyên lý áp cho mọi test level, mọi test type, mọi mô hình phát triển. Chúng không nói phải dùng kỹ thuật nào; chúng nói vì sao phải chọn và chọn theo tiêu chí gì.

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Phép đếm tổ hợp, tốc độ «1.000 ca mỗi giây», bảng phân bố defect theo module và chi phí sửa lỗi trong bài là <b>số giả định</b> do blog tự đặt để minh hoạ. Tỉ lệ «80/20» chỉ là cách nói quen thuộc về Pareto; syllabus không quy định con số nào.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Tính năng đang xét là **luồng đặt chuyến**: khách xin báo giá, đặt chuyến từ báo giá, tài xế nhận, khách có thể huỷ, hệ thống thu tiền. Theo đặc tả phòng lab ([SPEC v1.1](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing)):

- **Báo giá** `POST /api/quotes` (R-API-07): ba dịch vụ `ride`, `parcel`, `food`; sáu thành phố (R-GEN-03); xe máy hoặc ô tô (R-GEN-04); quãng đường làm tròn 0,1 km, hợp lệ trong [0,5; 100] km (R-FARE-02).
- **Thứ tự tính cước cố định 6 bước** (R-FARE-03): cước cơ bản → hệ số cao điểm 1,25 trong [11:00; 13:00) và [17:00; 19:00) (R-FARE-04) → phụ phí trời mưa 8.000 đ (R-FARE-05) → trừ voucher → nâng lên cước tối thiểu → làm tròn lên 1.000 đ.
- **Voucher** (R-VOUCHER): thời hạn, phạm vi dịch vụ/thành phố, số tiền tối thiểu, lượt dùng mỗi ngày, thứ tự báo lỗi cố định.
- **Báo giá có hạn 5 phút** và mỗi báo giá đặt tối đa một chuyến (R-QUOTE-01, R-QUOTE-03).
- **Tài xế nhận**: đúng một người thắng khi nhiều người cùng bấm (R-ACCEPT-03).
- **Thu tiền qua ví VíXanh**: hết giờ 3 giây, thử lại tối đa 2 lần với cùng khoá idempotency (R-PAY-04).

### System context

Trong kiến trúc đích của Tới Nơi, một lần đặt chuyến đi qua: ứng dụng khách → API Gateway → **Booking/Order Service** → **Pricing & Quote Service** (gọi bản đồ MapViet để lấy quãng đường) và **Promotion Service** (voucher) → **Matching & Dispatch** → **Trip Lifecycle** → **Payment & Wallet** (gọi VíXanh, ThẻNội Địa). Các service nói chuyện bằng REST/gRPC; sự kiện như `BookingCreated`, `DriverAssigned` đi qua Kafka với transactional outbox; Redis giữ lease khi ghép tài xế.

Phòng lab là bản **modular monolith** của giai đoạn MVP: cùng ranh giới module Pricing, Promotion, Booking, Dispatch, Trip, Payment adapter nhưng chạy một tiến trình, **không có Kafka, Redis hay realtime gateway** (SPEC R-ARCH-02). Khi bài nhắc tới sự kiện hay cache, đó là hệ thống đích, không phải lab.

### Rủi ro có thể xảy ra

| Rủi ro | Hậu quả | Nguồn |
|---|---|---|
| Tính sai cước ở mốc giờ cao điểm, mốc quãng đường, thứ tự voucher/cước tối thiểu | Khách trả sai tiền, khiếu nại | R-FARE-02…08 |
| Voucher dùng vượt lượt hoặc dùng sai thành phố | Thất thoát ngân sách khuyến mãi | R-VOUCHER-05, R-VOUCHER-07 |
| Hai tài xế cùng nhận một chuyến | Hai người tới đón, một người bị huỷ oan | R-ACCEPT-03, kiến trúc §4.5 |
| Thử lại khi ví hết giờ mà trừ tiền hai lần | Khách mất tiền, rủi ro pháp lý | R-PAY-04, kiến trúc §4.9 |
| Gửi lại yêu cầu đặt chuyến tạo chuyến trùng | Hai chuyến cho một nhu cầu | R-IDEM-02 |
| Sự kiện tới sai thứ tự làm trạng thái lùi (hệ thống đích) | Khách thấy chuyến «quay lại» trạng thái cũ | Kiến trúc §14, chưa có trong lab |

Bảy nguyên lý giúp quyết định nên đổ công sức vào hàng nào của bảng này và theo cách nào. Mỗi nguyên lý dưới đây gắn với **một tình huống khác nhau**:

| Nguyên lý | Tình huống tại Tới Nơi | Domain |
|---|---|---|
| 1 | 150 test cước đều xanh trên máy dev, lên máy chủ thì sai giờ cao điểm | Pricing |
| 2 | Đếm tổ hợp đầu vào của API báo giá | Pricing + Promotion |
| 3 | Đặc tả chưa nói voucher trừ trước hay sau cước tối thiểu | Pricing (đặc tả) |
| 4 | Nhật ký defect của bản MVP dồn vào hai module | Toàn hệ thống |
| 5 | Bộ hồi quy voucher chỉ dùng mã `GIAM20` ở TP. Hồ Chí Minh | Promotion |
| 6 | Kiểm thử thanh toán VíXanh khác kiểm thử trang tra cứu đơn | Payment vs Public Tracking |
| 7 | Báo giá đúng đặc tả nhưng khách bỏ đặt vì không thấy phụ phí trời mưa | Booking UI |

## 4. Phân tích kỹ thuật

### 4.1 Nguyên lý 1 — test chỉ ra có defect, không chứng minh hết defect

Đội Pricing viết 150 ca test cho hàm tính cước. Tất cả chạy trên máy lập trình viên đặt múi giờ UTC+7 và đều xanh. Lên máy chủ đặt múi giờ UTC, chuyến đặt lúc 11:30 giờ Việt Nam bị coi là 04:30 — không cao điểm — nên tính thiếu 25%.

150 ca xanh không sai. Chúng chỉ không bao giờ chạy trong điều kiện «máy chạy ở múi giờ khác». Đây là lý do SPEC viết hẳn R-GEN-02 («giờ nghiệp vụ là giờ Việt Nam, bất kể múi giờ của máy chủ») và lab tính giờ tường minh từ mốc UTC trong `src/domain/time.ts`.

**Quyết định:** báo cáo test ghi rõ phạm vi đã thử («cước đúng với 150 ca, chạy ở UTC+7») thay vì «module cước không còn lỗi». Thêm ít nhất một ca chạy với đồng hồ tiêm vào ở thời điểm UTC khác ngày Việt Nam.

### 4.2 Nguyên lý 2 — kiểm thử toàn bộ là bất khả thi

Đầu vào của `POST /api/quotes` nhìn thì ít trường, nhưng mỗi trường có rất nhiều giá trị và hệ thống còn phụ thuộc thời điểm đặt, thời tiết, lượt voucher đã dùng. Mục 5.1 đếm cụ thể: riêng `ride` đã hơn **10 tỉ** tổ hợp; cả ba dịch vụ khoảng **1,29 × 10<sup>15</sup>** — mà vẫn chưa tính ngày trong năm, đơn nhiều dòng món hay dữ liệu sai định dạng.

**Quyết định:** không đặt mục tiêu «thử hết». Dùng phân vùng tương đương và giá trị biên (Series 04) để chọn giá trị đại diện, rồi ưu tiên tổ hợp theo rủi ro ở bảng mục 3.

### 4.3 Nguyên lý 3 — kiểm thử sớm tiết kiệm thời gian và tiền

Giả sử bản nháp đặc tả chỉ ghi «áp voucher và cước tối thiểu». Không nói cái nào trước. Hai lập trình viên có thể hiểu khác nhau:

| Cách hiểu | Ô tô 0,5 km, voucher `GIAM20` | Kết quả |
|---|---|---|
| Voucher trước, cước tối thiểu sau (SPEC R-FARE-03 bước 4 → 5) | 30.500 − 6.100 = 24.400 ⇒ nâng lên 30.000 | **30.000 đ** |
| Cước tối thiểu trước, voucher sau | 30.500 ≥ 30.000 ⇒ 30.500 − 6.100 = 24.400 | **25.000 đ** |

Một câu hỏi trong buổi review đặc tả («voucher có được kéo cước xuống dưới mức tối thiểu không?») là đủ để sửa. Phát hiện sau khi phát hành thì phải sửa code, sửa test, đối soát các chuyến đã thu sai và trả lời khiếu nại. [TEST-002](/articles/vi-sao-can-kiem-thu) bàn chi phí của lỗi; ở đây chỉ cần nhớ: review đặc tả **là kiểm thử** (static testing), không phải việc làm trước kiểm thử.

**Quyết định:** tester đọc và hỏi về đặc tả ngay khi bản nháp có, không đợi bản build đầu tiên. SPEC hiện tại ghi rõ R-FARE-07 «voucher không bao giờ kéo cước xuống dưới mức tối thiểu» — chính là loại câu cần có từ đầu.

### 4.4 Nguyên lý 4 — defect tập trung thành cụm

Sau hai tháng chạy thử bản MVP, nhật ký có 40 defect (số giả định). Phân theo module:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 290" role="img" aria-label="Biểu đồ cột ngang, số giả định: 40 defect của bản MVP. Pricing và Voucher 16, Payment adapter 10, Trip lifecycle 6, Dispatch 4, Booking và idempotency 2, Giao diện web 2. Hai module đầu chiếm 26 trên 40, tức 65 phần trăm.">
<text x="460" y="26" text-anchor="middle" class="t-b">Số defect theo module — 40 defect sau 2 tháng chạy thử (giả định)</text>
<text x="250" y="66" text-anchor="end">Pricing &amp; Voucher</text>
<rect x="270" y="50" width="480" height="24" rx="4" class="f-warn-soft s-warn"/>
<text x="760" y="67" class="t-mono t-b t-warn">16</text>
<text x="250" y="100" text-anchor="end">Payment adapter</text>
<rect x="270" y="84" width="300" height="24" rx="4" class="f-warn-soft s-warn"/>
<text x="580" y="101" class="t-mono t-b t-warn">10</text>
<text x="250" y="134" text-anchor="end">Trip lifecycle</text>
<rect x="270" y="118" width="180" height="24" rx="4" class="f-pk-soft s-pk"/>
<text x="460" y="135" class="t-mono">6</text>
<text x="250" y="168" text-anchor="end">Dispatch</text>
<rect x="270" y="152" width="120" height="24" rx="4" class="f-pk-soft s-pk"/>
<text x="400" y="169" class="t-mono">4</text>
<text x="250" y="202" text-anchor="end">Booking &amp; idempotency</text>
<rect x="270" y="186" width="60" height="24" rx="4" class="f-pk-soft s-pk"/>
<text x="340" y="203" class="t-mono">2</text>
<text x="250" y="236" text-anchor="end">Giao diện web</text>
<rect x="270" y="220" width="60" height="24" rx="4" class="f-pk-soft s-pk"/>
<text x="340" y="237" class="t-mono">2</text>
<line x1="270" y1="44" x2="270" y2="250" class="s-muted"/>
<text x="460" y="276" text-anchor="middle" class="t-small">2 trong 6 module chứa 26/40 defect (65%) — ứng viên hàng đầu cho risk-based testing ở vòng sau</text>
</svg>
</div>
<figcaption><b>Hình 1 · Defect tập trung thành cụm.</b> Số liệu giả định. Pricing &amp; Voucher dày đặc quy tắc biên, làm tròn và giờ; Payment adapter có hết giờ, thử lại và idempotency. Cả hai đều là nơi kiến trúc Tới Nơi cảnh báo rủi ro cao.</figcaption>
</figure>

Cụm này không ngẫu nhiên: hai module đứng đầu là nơi có nhiều quy tắc biên (R-FARE, R-VOUCHER) và nhiều tình huống lỗi mạng (R-PAY-04). Ngay cả trước khi có số liệu, người hiểu kiến trúc đã có thể **dự đoán** cụm này — syllabus gọi đó là cụm dự đoán (predicted clusters), dùng cùng cụm thực tế làm đầu vào cho risk-based testing.

**Quyết định:** vòng test kế tiếp dành phần lớn thời gian cho Pricing/Voucher và Payment; không chia đều công sức cho sáu module. Nhưng vẫn giữ một phần cho module «ít lỗi» — xem mục 6 vì sao.

### 4.5 Nguyên lý 5 — test bị mòn

Bộ hồi quy tự động của Promotion có 30 ca, chạy mỗi lần merge. Mọi ca đều dùng `GIAM20` (mọi dịch vụ, mọi thành phố) ở `SG`. Một thay đổi làm sai kiểm tra phạm vi thành phố: `HN10` (chỉ `ride`, chỉ `HN`) nay dùng được ở `SG`. Bộ 30 ca vẫn xanh suốt ba tháng, vì không ca nào chạm vào nhánh «sai thành phố» của R-VOUCHER-05.

Bộ test không hỏng — nó đã «nhờn». Nó vẫn làm tốt việc của nó (bắt lỗi hồi quy trên `GIAM20`) nhưng không còn khả năng tìm defect mới ở chỗ khác.

**Quyết định:** định kỳ rà bộ hồi quy theo mã yêu cầu: R-VOUCHER-03…08 mỗi mã có ít nhất một ca; xoay dữ liệu (thêm `HN10`, `FOOD15`, `XMAS50` đã hết hạn, mã chữ thường có khoảng trắng hai đầu «␣giam20␣»). Giữ các ca cũ vì chúng vẫn bảo vệ hồi quy.

### 4.6 Nguyên lý 6 — kiểm thử phụ thuộc ngữ cảnh

Cùng một công ty, hai tính năng cần hai cách test rất khác nhau:

| Yếu tố | Thanh toán qua VíXanh (Payment & Wallet) | Trang tra cứu mã đơn công khai (Public Tracking) |
|---|---|---|
| Hậu quả khi sai | Trừ tiền hai lần, sai đối soát, vi phạm quy định | Lộ dữ liệu cá nhân nếu mã dễ đoán; sai thì khách tra lại |
| Trọng tâm | Idempotency, hết giờ/thử lại (R-PAY-04), không trừ hai lần | Phân quyền, chống dò mã, chỉ trả thông tin tối thiểu |
| Môi trường test | Ví giả có chế độ lỗi (`decline`, `timeout`, `lost_response`), sandbox của cổng | Trình duyệt, dữ liệu mã đơn giả |
| Mức độ hình thức | Tài liệu hoá ca test, truy vết yêu cầu, review độc lập | Gọn hơn, có thể dựa nhiều vào exploratory testing |
| Tiêu chí dừng | Mọi nhánh lỗi của cổng đã thử, đối soát khớp | Không lộ thông tin ngoài phạm vi, chịu được tải đọc |

Ngữ cảnh còn gồm giai đoạn: bản lab MVP một tiến trình không cần test sự kiện trùng qua Kafka; hệ thống đích nhiều service thì bắt buộc (kiến trúc §17.1). Cũng là luồng đặt chuyến, nhưng chiến lược test khác nhau.

### 4.7 Nguyên lý 7 — ngộ nhận «không có defect là thành công»

Màn hình báo giá qua hết các ca test R-FARE và R-UI. Không còn defect mở. Sau một tuần, tỉ lệ khách bấm «Báo giá» rồi bỏ không đặt tăng lên trong những ngày mưa. Phỏng vấn nhanh cho thấy khách chỉ thấy **tổng tiền**, còn phụ phí trời mưa 8.000 đ nằm trong bảng chi tiết phải mở ra mới thấy (R-FARE-10 có trả bảng chi tiết, nhưng đặc tả không bắt hiển thị nổi bật). Khách thấy giá «tự nhiên tăng» và nghĩ ứng dụng ăn gian.

Phần mềm đúng đặc tả — verification đạt. Nhưng nó không đáp ứng kỳ vọng của người dùng — validation chưa từng được làm. [TEST-005](/articles/verification-va-validation) đi sâu vào cặp khái niệm này.

**Quyết định:** kế hoạch test có ít nhất một hoạt động validation: cho vài khách thật dùng thử, đo tỉ lệ bỏ đặt, hỏi «bạn có hiểu vì sao giá là 52.000 đ không?».

## 5. Thực hành

Bài thực hành dùng SPEC của lab, không cần chạy code. Mục tiêu: thấy bằng số vì sao nguyên lý 2 đúng, rồi biến «không thể test hết» thành một tập ca test có chủ đích.

### 5.1 Đếm không gian đầu vào của `POST /api/quotes`

Chỉ đếm **đầu vào hợp lệ về định dạng** và các điều kiện hệ thống đơn giản nhất. Thời điểm đặt tính theo giây trong một ngày (cao điểm phụ thuộc giờ, R-FARE-04).

| Yếu tố | Miền giá trị (theo SPEC) | Số giá trị |
|---|---|---:|
| Thành phố | `SG`, `HN`, `DN`, `HP`, `CT`, `NT` (R-GEN-03) | 6 |
| Loại xe | `bike`, `car` (R-GEN-04) | 2 |
| Quãng đường | 0,5 → 100,0 km, bước 0,1 (R-FARE-02) | 996 |
| Thời điểm đặt | giây trong ngày, giờ Việt Nam (R-GEN-02) | 86.400 |
| Đang mưa | có / không (R-GEN-05) | 2 |
| Voucher | không mã + 4 mã mẫu `GIAM20`, `FOOD15`, `HN10`, `XMAS50` (Phụ lục A) | 5 |
| Khối lượng (chỉ `parcel`) | 0,1 → 30,0 kg, bước 0,1 (R-PARCEL-01) | 300 |
| Một dòng món (chỉ `food`) | đơn giá 1.000 → 5.000.000 đ bước 1.000 × số lượng 1 → 50 (R-FOOD-02) | 250.000 |

Phép nhân (đã chạy bằng script, số chính xác):

```text
ride   = 6 × 2 × 996 × 86.400 × 2 × 5                     =        10.326.528.000  ≈ 1,03 × 10^10
parcel = 6 × 2 × 996 × 300 × 86.400 × 2 × 5               =     3.097.958.400.000  ≈ 3,10 × 10^12
food   = 6 × 1 × 996 × 250.000 × 86.400 × 2 × 5 (1 dòng)  = 1.290.816.000.000.000  ≈ 1,29 × 10^15
tổng                                                      ≈ 1,29 × 10^15

Giả định chạy 1.000 ca mỗi giây:
  riêng ride ≈ 10.326.528 giây ≈ 119,5 ngày
  cả ba      ≈ 1,29 × 10^12 giây ≈ 41.000 năm
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Sáu yếu tố nhân với nhau: thành phố 6, loại xe 2, quãng đường 996, thời điểm đặt 86.400, mưa 2, voucher 5. Kết quả riêng ride khoảng 1,03 nhân 10 mũ 10. Nhân thêm 300 khối lượng thì parcel khoảng 3,1 nhân 10 mũ 12. Food với một dòng món nhân 250.000 và chỉ xe máy, khoảng 1,29 nhân 10 mũ 15. Với 1.000 ca mỗi giây, riêng ride mất khoảng 119,5 ngày, cả ba khoảng 41.000 năm.">
<defs>
<marker id="t003-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="460" y="24" text-anchor="middle" class="t-b">Không gian đầu vào hợp lệ của POST /api/quotes (chỉ một ngày, chưa tính trạng thái hệ thống)</text>
<rect x="20" y="44" width="128" height="56" rx="8" class="f-pk-soft s-pk"/><text x="84" y="66" text-anchor="middle" class="t-small">thành phố</text><text x="84" y="88" text-anchor="middle" class="t-mono t-b">6</text>
<text x="159" y="78" text-anchor="middle" class="t-b">×</text>
<rect x="170" y="44" width="128" height="56" rx="8" class="f-pk-soft s-pk"/><text x="234" y="66" text-anchor="middle" class="t-small">loại xe</text><text x="234" y="88" text-anchor="middle" class="t-mono t-b">2</text>
<text x="309" y="78" text-anchor="middle" class="t-b">×</text>
<rect x="320" y="44" width="128" height="56" rx="8" class="f-pk-soft s-pk"/><text x="384" y="66" text-anchor="middle" class="t-small">quãng đường</text><text x="384" y="88" text-anchor="middle" class="t-mono t-b">996</text>
<text x="459" y="78" text-anchor="middle" class="t-b">×</text>
<rect x="470" y="44" width="128" height="56" rx="8" class="f-pk-soft s-pk"/><text x="534" y="66" text-anchor="middle" class="t-small">giây trong ngày</text><text x="534" y="88" text-anchor="middle" class="t-mono t-b">86.400</text>
<text x="609" y="78" text-anchor="middle" class="t-b">×</text>
<rect x="620" y="44" width="128" height="56" rx="8" class="f-pk-soft s-pk"/><text x="684" y="66" text-anchor="middle" class="t-small">đang mưa</text><text x="684" y="88" text-anchor="middle" class="t-mono t-b">2</text>
<text x="759" y="78" text-anchor="middle" class="t-b">×</text>
<rect x="770" y="44" width="128" height="56" rx="8" class="f-pk-soft s-pk"/><text x="834" y="66" text-anchor="middle" class="t-small">voucher</text><text x="834" y="88" text-anchor="middle" class="t-mono t-b">5</text>
<line x1="459" y1="104" x2="160" y2="136" class="s-muted" stroke-width="1.4" marker-end="url(#t003-arrow)"/>
<line x1="459" y1="104" x2="459" y2="136" class="s-muted" stroke-width="1.4" marker-end="url(#t003-arrow)"/>
<line x1="459" y1="104" x2="758" y2="136" class="s-muted" stroke-width="1.4" marker-end="url(#t003-arrow)"/>
<rect x="40" y="140" width="240" height="66" rx="8" class="f-surface s-line"/><text x="160" y="162" text-anchor="middle" class="t-mono t-b">ride</text><text x="160" y="182" text-anchor="middle" class="t-small">không thêm yếu tố</text><text x="160" y="200" text-anchor="middle" class="t-mono t-b">≈ 1,03 × 10^10</text>
<rect x="340" y="140" width="240" height="66" rx="8" class="f-surface s-line"/><text x="460" y="162" text-anchor="middle" class="t-mono t-b">parcel</text><text x="460" y="182" text-anchor="middle" class="t-small">× 300 khối lượng</text><text x="460" y="200" text-anchor="middle" class="t-mono t-b">≈ 3,10 × 10^12</text>
<rect x="640" y="140" width="240" height="66" rx="8" class="f-warn-soft s-warn"/><text x="760" y="162" text-anchor="middle" class="t-mono t-b t-warn">food (1 dòng món)</text><text x="760" y="182" text-anchor="middle" class="t-small">× 250.000, chỉ xe máy</text><text x="760" y="200" text-anchor="middle" class="t-mono t-b t-warn">≈ 1,29 × 10^15</text>
<rect x="40" y="228" width="840" height="54" rx="8" class="f-warn-soft s-warn"/>
<text x="460" y="250" text-anchor="middle" class="t-b">1.000 ca/giây (giả định): riêng ride ≈ 119,5 ngày · cả ba ≈ 41.000 năm</text>
<text x="460" y="270" text-anchor="middle" class="t-small">Chưa tính: ngày trong năm (hạn voucher), lượt voucher đã dùng, đơn 2–20 dòng món, đầu vào sai, đồng thời</text>
</svg>
</div>
<figcaption><b>Hình 2 · Bùng nổ tổ hợp ở API báo giá.</b> Số tổ hợp chính xác từ SPEC; tốc độ 1.000 ca/giây là giả định. Đơn đồ ăn có tới 20 dòng món nên số thật còn lớn hơn rất nhiều bậc.</figcaption>
</figure>

Và đó mới là phần dễ. Kết quả báo giá còn phụ thuộc **ngày** (hạn voucher R-VOUCHER-04), **lịch sử** của khách (lượt voucher trong ngày R-VOUCHER-07), và các yêu cầu sai định dạng (thành phố `DL`, quãng đường âm, chuỗi thay số) — những thứ không đếm được thành con số hữu hạn.

### 5.2 Từ «không thể thử hết» tới một tập ca có chủ đích

Bước 1 — **chọn giá trị đại diện** cho từng yếu tố bằng phân vùng tương đương và giá trị biên (kỹ thuật của Series 04, ở đây chỉ dùng kết quả):

| Yếu tố | Giá trị đại diện | Lý do | Số giá trị |
|---|---|---|---:|
| Thành phố | `SG`, `HN`, `DL` | một thành phố thường, thành phố của `HN10`, một mã không hỗ trợ | 3 |
| Loại xe | `bike`, `car` | hai bảng giá khác nhau | 2 |
| Quãng đường | 0,44 · 0,45 · 7,4 · 100,04 · 100,05 | hai biên làm tròn của [0,5; 100] + ví dụ oracle | 5 |
| Thời điểm | 09:00 · 10:59:59 · 11:00:00 · 12:59:59 · 13:00:00 · 16:59:59 · 17:00:00 · 18:59:59 · 19:00:00 | ngoài cao điểm + 8 điểm biên của hai khung | 9 |
| Đang mưa | có, không | | 2 |
| Voucher | không mã · `GIAM20` · `HN10` · `FOOD15` · `XMAS50` · `ABC` · «␣giam20␣» | đủ mỗi mã lỗi của R-VOUCHER-08 + chuẩn hoá mã | 7 |

Bước 2 — **đừng nhân hết**. Nhân các giá trị đại diện cho riêng `ride` vẫn ra 3 × 2 × 5 × 9 × 2 × 7 = **3.780** ca. Thay vào đó:

- Mỗi giá trị đại diện xuất hiện ít nhất một lần: tối thiểu **9 ca** (bằng yếu tố nhiều giá trị nhất).
- Thêm các tổ hợp mà quy tắc nghiệp vụ cho thấy **có tương tác**: cao điểm × phụ phí bưu kiện (R-PARCEL-04 — phụ phí bị nhân 1,25), voucher × cước tối thiểu (R-FARE-07), voucher × thành phố (R-VOUCHER-05), nhiều lỗi cùng lúc × thứ tự báo lỗi (R-VALID-01).
- Thêm tổ hợp theo rủi ro ở bảng mục 3 và theo cụm defect ở Hình 1.

Kết quả là vài chục ca thay cho 10<sup>15</sup> — và mỗi ca có lý do để tồn tại. Đây chính là điều nguyên lý 2 yêu cầu: dùng kỹ thuật, ưu tiên và rủi ro để dồn sức, không «thử thật nhiều cho chắc».

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài CTFL</span>
<p>Cách «mỗi giá trị xuất hiện ít nhất một lần» (each choice) và «mọi cặp giá trị» (pairwise) là kỹ thuật tổ hợp; CTFL v4.0.1 không có LO cho chúng, CTAL-TA v4.0 thì có. Series 04 sẽ trình bày đầy đủ.</p>
</div>

### 5.3 Phiếu soát nguyên lý cho một kế hoạch test nháp

Đội nhận tính năng «đặt chuyến» cho bản phát hành tới. Bản nháp kế hoạch test có bảy câu dưới đây. Mỗi câu vi phạm một nguyên lý — cột bên phải là cách sửa.

| Câu trong kế hoạch nháp | Vi phạm | Viết lại |
|---|---|---|
| «Khi 150 ca test cước đều đạt, xác nhận với ban giám đốc module cước không còn lỗi.» | NL1 | «… báo cáo: 150 ca đạt, phạm vi đã thử là …, rủi ro còn lại là …» |
| «Test mọi quãng đường từ 0,5 tới 100 km ở mọi giờ trong ngày.» | NL2 | Chọn giá trị đại diện và biên như mục 5.2 |
| «Tester bắt đầu khi bản build đầu tiên lên môi trường test.» | NL3 | Tester review SPEC R-FARE, R-VOUCHER ngay khi có bản nháp |
| «Chia đều thời gian test cho sáu module.» | NL4 | Ưu tiên Pricing/Voucher và Payment theo cụm dự đoán và nhật ký defect |
| «Bộ hồi quy voucher giữ nguyên từ năm ngoái.» | NL5 | Rà theo mã R-VOUCHER, thêm mã và dữ liệu mới |
| «Áp nguyên checklist test của trang tin tức nội bộ cho thanh toán VíXanh.» | NL6 | Thêm ca hết giờ, thử lại, `lost_response`, đối soát |
| «Đủ điều kiện phát hành khi không còn defect mở.» | NL7 | Thêm tiêu chí validation: khách dùng thử hiểu được giá |

## 6. Góc nhìn nâng cao

**Nguyên lý là hướng dẫn, không phải định luật.** Syllabus dùng chữ «thường» (usually) ở nguyên lý 4 và «trừ trường hợp tầm thường» ở nguyên lý 2. Áp máy móc dễ phản tác dụng.

**Kiểm thử toàn bộ khả thi với đối tượng tầm thường.** Hàm `isPeakTime` của lab chỉ phụ thuộc giờ trong ngày: 86.400 giây là đủ để thử hết, chạy vài chục mili-giây. [TEST-004](/articles/error-defect-failure-root-cause) quét từng giây trong 10 phút của hàm phí huỷ — cũng là một dạng vét cạn trên miền nhỏ. Nguyên lý 2 nói về **hệ thống**, không cấm vét cạn một hàm nhỏ khi có thể.

**Cụm defect có thể là ảo ảnh.** Module được test nhiều sẽ có nhiều defect được ghi nhận; module ít ai test trông «sạch» chỉ vì chưa ai nhìn. Hình 1 có thể phản ánh nơi đội đã test, không chỉ nơi defect nằm. Luôn giữ một phần công sức cho vùng «ít lỗi», và xem cả failure ở vận hành chứ không chỉ defect lúc test.

**Nguyên lý 5 không mâu thuẫn với hồi quy.** Bộ hồi quy lặp lại để bắt **tác dụng phụ** của thay đổi — việc đó vẫn hiệu quả. Cái bị mòn là khả năng tìm defect **mới**. Hai mục đích, hai cách đối xử: giữ hồi quy ổn định, còn test khám phá và dữ liệu thì xoay vòng.

**Nguyên lý 6 không có nghĩa «tuỳ hứng».** Phụ thuộc ngữ cảnh là lý do để **chọn có căn cứ** (rủi ro, quy định, công nghệ), không phải lý do bỏ quy trình. Phần lớn hoạt động test vẫn giống nhau ở mọi dự án — xem [TEST-007](/articles/quy-trinh-kiem-thu-test-process).

**Nguyên lý 1 và «coverage 100%».** Đạt 100% statement coverage cho `fare.ts` vẫn không chứng minh hết defect: coverage đo code đã chạy, không đo mọi giá trị dữ liệu hay mọi điều kiện môi trường (như ví dụ múi giờ ở mục 4.1).

**Sai lầm thường gặp khi học nguyên lý:**

| Sai lầm | Đúng là |
|---|---|
| «Kiểm thử sớm = tester vào dự án sớm» | Là **hoạt động test** bắt đầu sớm, gồm static testing trên đặc tả |
| «Không tìm thấy defect ⇒ test kém» | Có thể sản phẩm tốt; đánh giá test bằng độ phủ rủi ro và yêu cầu, không chỉ số defect |
| «Pareto ⇒ chỉ test 20% module» | Dồn trọng tâm, không bỏ phần còn lại |
| Nhầm nguyên lý 7 với nguyên lý 1 | NL1 nói về **giới hạn của test**; NL7 nói về **đúng đặc tả chưa chắc đúng nhu cầu** |

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — chọn 10 tổ hợp đáng test nhất.** Từ không gian đầu vào ở mục 5.1, chọn đúng 10 tổ hợp cho `POST /api/quotes` và ghi lý do (mã yêu cầu, rủi ro). Được dùng dữ liệu mẫu Phụ lục A của SPEC.

<details>
<summary><b>Lời giải mẫu</b></summary>
<p>Một lời giải hợp lý (có nhiều lời giải đúng — quan trọng là lý do):</p>
<table>
<thead><tr><th>#</th><th>Tổ hợp</th><th>Kỳ vọng</th><th>Lý do</th></tr></thead>
<tbody>
<tr><td>1</td><td>ride, SG, bike, 7,4 km, 11:30, mưa, <code>GIAM20</code></td><td>52.000 đ</td><td>Oracle R-FARE-09 đi qua cả 6 bước</td></tr>
<tr><td>2</td><td>ride, HN, car, 10 km, 13:00:00, không mưa</td><td>Không cao điểm</td><td>Biên trên khung cao điểm R-FARE-04</td></tr>
<tr><td>3</td><td>ride, SG, bike, 0,45 km, 09:00</td><td>15.000 đ</td><td>Biên làm tròn R-FARE-02 + cước tối thiểu R-FARE-07</td></tr>
<tr><td>4</td><td>ride, SG, bike, 100,05 km</td><td><code>DISTANCE_OUT_OF_RANGE</code></td><td>Biên ngoài sau làm tròn</td></tr>
<tr><td>5</td><td>ride, SG, car, 0,5 km, 09:00, <code>GIAM20</code></td><td>30.000 đ</td><td>Voucher không kéo dưới cước tối thiểu R-FARE-07</td></tr>
<tr><td>6</td><td>parcel, SG, bike, 5 km, 6,1 kg, 17:00</td><td>49.000 đ</td><td>Phụ phí 4.000 đ bị nhân 1,25 (R-PARCEL-03/04)</td></tr>
<tr><td>7</td><td>parcel, SG, bike, 20,1 kg</td><td><code>VEHICLE_TOO_SMALL</code></td><td>Biên 20 kg R-PARCEL-02</td></tr>
<tr><td>8</td><td>food, SG, tiền món 30.000 đ, <code>FOOD15</code></td><td><code>VOUCHER_MIN_AMOUNT</code></td><td>Biên R-FOOD-03 hợp lệ nhưng chưa đủ 50.000 đ của voucher</td></tr>
<tr><td>9</td><td>ride, SG, bike, <code>HN10</code></td><td><code>VOUCHER_NOT_APPLICABLE</code></td><td>Phạm vi thành phố R-VOUCHER-05 — vùng bộ hồi quy cũ bỏ sót (mục 4.5)</td></tr>
<tr><td>10</td><td>ride, <code>DL</code>, bike, 200 km, <code>XMAS50</code></td><td><code>CITY_NOT_SUPPORTED</code></td><td>Nhiều lỗi cùng lúc, chỉ báo lỗi đầu tiên R-VALID-01</td></tr>
</tbody>
</table>
<p>Số tiền ở ca 1, 3, 5, 6 đã kiểm bằng hàm <code>calculateFare</code> của lab. Tổ hợp ngoài danh sách (ví dụ <code>XMAS50</code> đơn lẻ, chuẩn hoá mã «␣giam20␣») vẫn cần test — chúng chỉ không lọt vào top 10 theo rủi ro.</p>
</details>

**Bài tập 2 — nhận diện nguyên lý.** Mỗi câu nói của thành viên đội Tới Nơi dưới đây liên quan nguyên lý nào?

1. «Tháng này 70% khiếu nại là về phí huỷ, sprint sau dồn test vào `cancel.ts`.»
2. «App tài xế cần test pin, GPS nền, mất sóng; app khách thì không cần nhiều như vậy.»
3. «Bộ E2E đặt chuyến chạy xanh 200 lần liên tiếp, vậy mà lỗi chuyến trùng vẫn lọt ra.»
4. «Đặc tả hoàn tiền chưa có thì đừng code, để tester đọc trước đã.»

<details>
<summary><b>Đáp án</b></summary>
<p>1 — NL4 (cụm defect thực tế dùng cho risk-based testing). 2 — NL6 (ngữ cảnh: ứng dụng tài xế chạy nền, phụ thuộc thiết bị). 3 — NL5 (test bị mòn: cùng một kịch bản lặp lại không chạm vào gửi lại yêu cầu hay hai yêu cầu đồng thời; cũng liên quan NL1 — 200 lần xanh không chứng minh không có defect). 4 — NL3 (kiểm thử sớm bằng static testing).</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K1).** Tên nào sau đây là một trong bảy nguyên lý kiểm thử của CTFL v4.0.1?

- a) Testing proves the absence of defects
- b) Tests wear out
- c) Testing is independent of context
- d) Exhaustive testing is required for safety-critical systems

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — tên nguyên lý 5 trong v4.0.1 (bản cũ gọi là pesticide paradox). a) ngược với nguyên lý 1. c) ngược với nguyên lý 6. d) sai: nguyên lý 2 nói kiểm thử toàn bộ không khả thi trừ trường hợp tầm thường, kể cả với hệ thống quan trọng về an toàn.</p>
</details>

**Câu 2 (K2).** Bộ test cước của Tới Nơi có 150 ca, tất cả đều đạt. Câu nào trong báo cáo test phù hợp nhất với các nguyên lý kiểm thử?

- a) Module cước không còn defect.
- b) Không phát hiện defect trong 150 ca đã chạy; khả năng còn defect chưa phát hiện đã giảm nhưng không loại trừ được.
- c) Vì không có defect nên không cần kiểm thử hồi quy cho module cước nữa.
- d) Vì không có defect nên module cước chắc chắn đáp ứng nhu cầu khách hàng.

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — nguyên lý 1. a) khẳng định điều test không chứng minh được. c) sai: hồi quy cần mỗi khi có thay đổi, không phụ thuộc kết quả vòng trước. d) là absence-of-defects fallacy (nguyên lý 7).</p>
</details>

**Câu 3 (K2).** Đội test Tới Nơi nhận thấy 26/40 defect nằm ở Pricing &amp; Voucher và Payment adapter. Cách dùng thông tin này phù hợp nhất là gì?

- a) Ngừng test bốn module còn lại.
- b) Dùng nó làm đầu vào cho kiểm thử dựa trên rủi ro, tăng trọng tâm vào hai module đó.
- c) Kết luận hai module đó do lập trình viên kém viết, đề nghị thay người.
- d) Chia đều thời gian cho cả sáu module để công bằng.

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — nguyên lý 4: cụm defect là đầu vào cho risk-based testing. a) bỏ hẳn phần còn lại là cực đoan, và cụm có thể chỉ phản ánh nơi đã test nhiều. c) nguyên lý không nói gì về đánh giá cá nhân; cụm thường do độ phức tạp. d) bỏ qua thông tin rủi ro đã có.</p>
</details>

**Câu 4 (K2).** Bộ hồi quy voucher chạy xanh ba tháng liền trong khi một defect về phạm vi thành phố vẫn tồn tại. Hành động nào xử lý đúng gốc vấn đề theo nguyên lý «tests wear out»?

- a) Chạy bộ hồi quy nhiều lần hơn mỗi ngày.
- b) Xoá toàn bộ bộ hồi quy và viết lại từ đầu.
- c) Rà và bổ sung ca test, đổi dữ liệu (thêm mã chỉ áp ở một thành phố), giữ các ca cũ còn giá trị hồi quy.
- d) Chuyển bộ hồi quy từ tự động sang chạy tay.

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — sửa test và dữ liệu, viết test mới; hồi quy lặp lại vẫn có ích. a) lặp nhiều hơn cùng một bộ không tìm được defect mới. b) bỏ luôn phần hồi quy đang có giá trị. d) đổi cách chạy không đổi nội dung test.</p>
</details>

**Câu 5 (K2).** Ví dụ nào minh hoạ rõ nhất nguyên lý «early testing saves time and money»?

- a) Tester chạy bộ E2E mỗi đêm sau khi phát hành.
- b) Trong buổi review đặc tả, tester hỏi voucher áp trước hay sau cước tối thiểu, và đặc tả được sửa trước khi viết code.
- c) Đội vận hành theo dõi tỉ lệ lỗi trên production.
- d) Tester tăng số ca test hồi quy trước ngày phát hành.

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — defect trong đặc tả được gỡ bằng static testing trước khi lan sang code và test. a), c), d) đều là hoạt động ở giai đoạn muộn; có ích nhưng không minh hoạ nguyên lý 3.</p>
</details>

**Câu 6 (K2).** Màn hình báo giá của Tới Nơi qua mọi ca test theo đặc tả, nhưng khách bỏ đặt vì không thấy phụ phí trời mưa. Nguyên lý nào mô tả đúng nhất tình huống này?

- a) Exhaustive testing is impossible
- b) Defects cluster together
- c) Absence-of-defects fallacy
- d) Testing is context dependent

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — verification đạt nhưng hệ thống không đáp ứng nhu cầu người dùng; cần validation. a) không liên quan tới số tổ hợp. b) không nói về phân bố defect. d) ngữ cảnh có ảnh hưởng nhưng không giải thích vì sao «không có defect» vẫn thất bại.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024), mục 1.3 «Testing Principles», LO FL-1.3.1 (K2): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf — các mục 4 (test techniques), 5.1.5 (ưu tiên test case), 5.2 (risk-based testing) được nguyên lý 2 và 4 dẫn tới.
- ISTQB, *Certified Tester Foundation Level Syllabus* 2018 (v3.1) — tên cũ «pesticide paradox», «absence-of-errors is a fallacy».
- ISTQB Glossary: https://glossary.istqb.org/ — tra các mục *exhaustive testing*, *defect*, *risk-based testing*.
- Trang chủ ISTQB: https://www.istqb.org/
- Nguồn gốc các nguyên lý theo syllabus: Buxton &amp; Randell (eds.) 1970, *Software Engineering Techniques* (kỷ yếu hội nghị NATO); Manna 1978; Boehm 1981, *Software Engineering Economics*; Enders 1975; Beizer 1990, *Software Testing Techniques*; Kaner 2011.
- Đặc tả và mã nguồn phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing (SPEC R-API-07, R-FARE, R-VOUCHER, R-PARCEL, R-FOOD, Phụ lục A).
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-001](/articles/software-testing-la-gi-testing-khac-debugging) · [TEST-002](/articles/vi-sao-can-kiem-thu) · [TEST-004](/articles/error-defect-failure-root-cause) · [TEST-005](/articles/verification-va-validation) · [TEST-007](/articles/quy-trinh-kiem-thu-test-process).

Bài tiếp theo: [Error, Defect, Failure và Root Cause — phân biệt lỗi code và lỗi nghiệp vụ](/articles/error-defect-failure-root-cause).
