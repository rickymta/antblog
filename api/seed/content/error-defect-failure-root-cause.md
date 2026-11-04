---
slug: error-defect-failure-root-cause
title: "Error, Defect, Failure và Root Cause: phân biệt lỗi code và lỗi nghiệp vụ"
summary: "Bốn khái niệm mà tiếng Việt hay gọi chung là «lỗi»: error, defect, failure và root cause. Tái hiện chuỗi nhân quả trên phí huỷ chuyến Tới Nơi bằng một bản cài đặt lỗi chạy thật với vitest."
category: testing
level: beginner
tags: [testing, istqb, ctfl, test-004, error, defect, failure, root-cause, vitest]
language: vi
---

Tài xế Tới Nơi phàn nàn: khách huỷ chuyến sau khi họ đã chạy được hai phút mà họ không nhận được đồng phí bù nào. Đó là «lỗi» gì? Một dòng code sai, một yêu cầu viết mơ hồ, hay một người đọc nhầm đặc tả? Tiếng Việt gọi chung tất cả là «lỗi», còn ISTQB tách thành bốn khái niệm: **error**, **defect**, **failure** và **root cause**. Bài này tách bốn khái niệm đó trên một ví dụ chạy được và phân biệt hai loại hay bị lẫn: lỗi trong code và lỗi trong nghiệp vụ (đặc tả).

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-004</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S01 · Fundamentals of Software Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L1 · Beginner</td></tr>
<tr><th scope="row">Test Level</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Type</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Test Technique</th><td>Không áp dụng</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/software-testing-la-gi-testing-khac-debugging">TEST-001 · Software Testing là gì? Testing khác Debugging</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): tái hiện error → defect → failure trên phí huỷ (SPEC R-CANCEL-02) — <code>examples/toi-noi-testing/articles/error-defect-failure-root-cause/</code>, chạy bằng vitest</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 1 · mục 1.2.3 · FL-1.2.3 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

Khi nghe «hệ thống bị lỗi», mỗi người trong đội hiểu một kiểu:

- Bộ phận chăm sóc khách hàng nghĩ tới **điều khách thấy**: «khách bị thu sai tiền».
- Lập trình viên nghĩ tới **dòng code**: «chỗ so sánh viết sai dấu».
- Trưởng nhóm nghĩ tới **người và quy trình**: «ai hiểu nhầm yêu cầu, vì sao không ai phát hiện».

Cả ba đều đúng, nhưng họ đang nói về ba thứ khác nhau. Nếu không tách ra, đội sẽ sửa nhầm tầng: sửa xong dòng code mà không ai sửa câu đặc tả mơ hồ, tháng sau một lập trình viên khác lại hiểu nhầm đúng câu đó ở một chỗ khác.

Bốn khái niệm của ISTQB cho mỗi tầng một cái tên. Có tên rồi thì báo cáo defect rõ hơn, đo lường đúng hơn (đếm failure khác với đếm defect), và biết nên chặn vấn đề ở đâu: trong code, trong đặc tả, trong môi trường hay trong cách làm việc.

Dùng khi nào: mỗi lần viết hay đọc defect report, phân loại sự cố trên production, họp rút kinh nghiệm sau sự cố, và khi trả lời câu hỏi «test đã đủ chưa?».

## 2. Lý thuyết theo ISTQB

Syllabus CTFL v4.0.1 mục 1.2.3 có learning objective **FL-1.2.3 (K2) — phân biệt root cause, error, defect và failure**. Nội dung chính, diễn đạt lại:

**Chuỗi nhân quả.** Con người mắc **error** (sai lầm, mistake); error tạo ra **defect** (khiếm khuyết — còn gọi là fault hay bug) trong một sản phẩm công việc; defect, khi được thực thi, **có thể** gây ra **failure** (thất bại — hệ thống không làm điều phải làm, hoặc làm điều không được làm).

**Vì sao con người mắc error.** Syllabus kể: áp lực thời gian; độ phức tạp của sản phẩm công việc, quy trình, hạ tầng hay các tương tác; mệt mỏi; thiếu đào tạo phù hợp.

**Defect nằm ở đâu.** Không chỉ trong code. Defect có thể nằm trong **tài liệu** (đặc tả yêu cầu, test script), trong **mã nguồn**, hoặc trong **sản phẩm hỗ trợ** như file build. Defect ở sản phẩm làm sớm (yêu cầu) mà không bị phát hiện thường kéo theo defect ở sản phẩm làm sau (thiết kế, code, test).

**Không phải defect nào cũng gây failure.** Có defect hễ chạy tới là failure; có defect chỉ gây failure trong điều kiện đặc biệt; có defect không bao giờ gây failure.

**Không phải failure nào cũng do defect.** Failure còn có thể do **điều kiện môi trường** — syllabus lấy ví dụ bức xạ hay trường điện từ làm hỏng firmware.

**Root cause** là lý do căn bản khiến vấn đề xảy ra, ví dụ một hoàn cảnh dẫn tới error. Root cause được tìm bằng **root cause analysis** (phân tích nguyên nhân gốc), thường làm khi có failure hoặc khi phát hiện defect. Gỡ bỏ root cause được kỳ vọng sẽ ngăn hoặc giảm các defect/failure **cùng loại** về sau — không chỉ một defect cụ thể.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 310" role="img" aria-label="Chuỗi bốn ô từ trái sang phải. Root cause: yêu cầu chỉ ghi trong vòng 2 phút, không có ví dụ mốc biên, gấp hạn không review. Mũi tên dẫn tới Error: lập trình viên hiểu trong vòng 2 phút là tính cả mốc 2:00. Mũi tên tạo ra Defect: cancel.ts viết nhỏ hơn hoặc bằng thay vì nhỏ hơn. Mũi tên khi được thực thi tới Failure: khách huỷ đúng 2:00 bị tính 0 đồng thay vì 10.000 đồng. Hàng dưới: dưới Root cause là ô phân tích nguyên nhân gốc, gỡ root cause để giảm defect cùng loại. Dưới Defect là ô không chạy tới nhánh lỗi thì không có failure. Dưới Failure là ô môi trường như đồng hồ lệch, bức xạ, cũng có thể gây failure dù không có defect.">
<defs>
<marker id="t004-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="t004-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
<marker id="t004-muted" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<text x="460" y="24" text-anchor="middle" class="t-b">Chuỗi nhân quả trên phí huỷ chuyến của Tới Nơi (câu chuyện giả định)</text>
<rect x="16" y="44" width="196" height="118" rx="8" class="f-phys-soft s-phys"/>
<text x="114" y="66" text-anchor="middle" class="t-b t-phys">Root cause</text>
<text x="114" y="84" text-anchor="middle" class="t-mono">nguyên nhân gốc</text>
<text x="114" y="108" text-anchor="middle" class="t-small">Yêu cầu chỉ ghi «trong vòng</text>
<text x="114" y="124" text-anchor="middle" class="t-small">2 phút», không có ví dụ mốc;</text>
<text x="114" y="140" text-anchor="middle" class="t-small">gấp hạn, không ai review</text>
<rect x="244" y="44" width="196" height="118" rx="8" class="f-pk-soft s-pk"/>
<text x="342" y="66" text-anchor="middle" class="t-b">Error</text>
<text x="342" y="84" text-anchor="middle" class="t-mono">sai lầm (mistake)</text>
<text x="342" y="108" text-anchor="middle" class="t-small">Lập trình viên hiểu «trong</text>
<text x="342" y="124" text-anchor="middle" class="t-small">vòng 2 phút» là tính cả 2:00</text>
<text x="342" y="140" text-anchor="middle" class="t-small">— hành động của con người</text>
<rect x="472" y="44" width="196" height="118" rx="8" class="f-pk-soft s-pk"/>
<text x="570" y="66" text-anchor="middle" class="t-b">Defect</text>
<text x="570" y="84" text-anchor="middle" class="t-mono">khiếm khuyết (bug, fault)</text>
<text x="570" y="108" text-anchor="middle" class="t-small">cancel.ts viết &lt;= thay vì &lt;</text>
<text x="570" y="124" text-anchor="middle" class="t-small">nằm im trong mã cho tới</text>
<text x="570" y="140" text-anchor="middle" class="t-small">khi được thực thi</text>
<rect x="700" y="44" width="196" height="118" rx="8" class="f-warn-soft s-warn"/>
<text x="798" y="66" text-anchor="middle" class="t-b t-warn">Failure</text>
<text x="798" y="84" text-anchor="middle" class="t-mono">thất bại quan sát được</text>
<text x="798" y="108" text-anchor="middle" class="t-small">Huỷ đúng 2:00 ⇒ phí 0 đ</text>
<text x="798" y="124" text-anchor="middle" class="t-small">thay vì 10.000 đ (R-CANCEL-02)</text>
<text x="798" y="140" text-anchor="middle" class="t-small">tài xế không được bù phí</text>
<line x1="212" y1="103" x2="242" y2="103" class="s-pk" stroke-width="1.6" marker-end="url(#t004-arrow)"/>
<line x1="440" y1="103" x2="470" y2="103" class="s-pk" stroke-width="1.6" marker-end="url(#t004-arrow)"/>
<line x1="668" y1="103" x2="698" y2="103" class="s-warn" stroke-width="1.6" marker-end="url(#t004-warn)"/>
<text x="683" y="186" text-anchor="middle" class="t-10">khi thực thi</text>
<rect x="16" y="214" width="196" height="76" rx="8" class="f-surface s-line"/>
<text x="114" y="236" text-anchor="middle" class="t-b">Root cause analysis</text>
<text x="114" y="256" text-anchor="middle" class="t-small">gỡ root cause ⇒ ít defect</text>
<text x="114" y="272" text-anchor="middle" class="t-small">CÙNG LOẠI về sau</text>
<line x1="114" y1="212" x2="114" y2="164" class="s-muted dash" stroke-width="1.4" marker-end="url(#t004-muted)"/>
<rect x="472" y="214" width="196" height="76" rx="8" class="f-surface s-line dash"/>
<text x="570" y="236" text-anchor="middle" class="t-b">Không chạy tới nhánh lỗi</text>
<text x="570" y="256" text-anchor="middle" class="t-small">(chuyến ở arriving, huỷ 1:59…)</text>
<text x="570" y="272" text-anchor="middle" class="t-small">⇒ defect có, failure không</text>
<line x1="570" y1="164" x2="570" y2="212" class="s-muted dash" stroke-width="1.4" marker-end="url(#t004-muted)"/>
<rect x="700" y="214" width="196" height="76" rx="8" class="f-warn-soft s-warn dash"/>
<text x="798" y="236" text-anchor="middle" class="t-b t-warn">Môi trường</text>
<text x="798" y="256" text-anchor="middle" class="t-small">đồng hồ lệch, bức xạ, phần cứng</text>
<text x="798" y="272" text-anchor="middle" class="t-small">⇒ failure dù code đúng</text>
<line x1="798" y1="212" x2="798" y2="164" class="s-warn dash" stroke-width="1.4" marker-end="url(#t004-warn)"/>
</svg>
</div>
<figcaption><b>Hình 1 · Root cause → error → defect → failure.</b> Hàng trên là chuỗi chính. Hàng dưới là ba ngoại lệ syllabus nhấn mạnh: phân tích nguyên nhân gốc để chặn cả loại lỗi; defect có thể không bao giờ gây failure; failure có thể không do defect.</figcaption>
</figure>

### Thuật ngữ Anh – Việt

| Thuật ngữ (ISTQB) | Đồng nghĩa | Nghĩa tiếng Việt dùng trong blog | Nằm ở đâu |
|---|---|---|---|
| error | mistake | sai lầm của con người | Trong đầu người: hiểu nhầm, quên, gõ nhầm |
| defect | fault, bug | khiếm khuyết trong sản phẩm công việc | Đặc tả, code, test script, file build, cấu hình |
| failure | — | thất bại quan sát được khi hệ thống chạy | Hành vi hệ thống: kết quả sai, treo, làm điều không được làm |
| root cause | — | nguyên nhân gốc | Hoàn cảnh, quy trình, kỹ năng, công cụ |
| root cause analysis (RCA) | — | phân tích nguyên nhân gốc | Hoạt động sau khi có failure/defect |
| anomaly | — | điều bất thường (chưa chắc là defect) | Thứ được báo cáo trước khi phân tích — CTFL mục 5.5 |
| false-positive result | — | báo lỗi giả | Test báo hỏng nhưng sản phẩm đúng |
| debugging | — | gỡ lỗi | Tìm, phân tích, sửa defect — [TEST-001](/articles/software-testing-la-gi-testing-khac-debugging) |

**Phạm vi áp dụng.** Bốn khái niệm dùng cho mọi sản phẩm công việc và mọi test level. Static testing (review đặc tả, review code) tìm **defect** trực tiếp mà không cần failure; dynamic testing (chạy phần mềm) làm lộ **failure**, rồi debugging lần ngược về defect.

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>Cách chia <b>«lỗi code» và «lỗi nghiệp vụ»</b> trong bài là cách nói của ngành, không phải thuật ngữ ISTQB. Trong ngôn ngữ ISTQB, cả hai đều là <b>defect</b>, chỉ khác <b>sản phẩm công việc chứa defect</b>: mã nguồn hay đặc tả yêu cầu. Các hệ phân loại defect chi tiết (ví dụ IEEE Std 1044 về phân loại bất thường phần mềm) và kỹ thuật RCA như 5 Whys hay sơ đồ xương cá thuộc Series 12, không thuộc CTFL chương 1.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### Yêu cầu nghiệp vụ

Phí khi **khách** huỷ chuyến (SPEC lab, mục R-CANCEL):

| Mã | Quy tắc |
|---|---|
| R-CANCEL-01 | Huỷ khi `searching`: 0 đ. Khi `accepted`, chưa đủ 2 phút từ `acceptedAt`: 0 đ. Khi `accepted` từ 2 phút trở lên, hoặc khi `arriving`: có phí. Trạng thái khác: không được huỷ (`INVALID_TRANSITION`). |
| R-CANCEL-02 | «Chưa đủ 2 phút» nghĩa là `now − acceptedAt < 2 phút`. **Đúng 2 phút 0 giây là có phí.** |
| R-CANCEL-03 | Phí 10.000 đ với xe máy (gồm mọi đơn đồ ăn), 20.000 đ với ô tô. |
| R-CANCEL-04 | Nếu `now > acceptedAt + etaMinutes + 10 phút` thì miễn phí vì tài xế trễ. |
| R-CANCEL-06 | Kết quả kèm lý do: `searching`, `grace_period`, `driver_late`, `charged`. |
| R-PAY-05 | Phí > 0 thu qua VíXanh (khoá `trip:<id>:cancel-fee`) hoặc ghi nhận tiền mặt. |

Business rule đằng sau: khi tài xế đã nhận và bắt đầu chạy tới, khách huỷ muộn làm tài xế mất thời gian và xăng; phí huỷ bù cho tài xế. Hai phút đầu là thời gian «đổi ý» miễn phí.

### System context

Trong kiến trúc đích, lệnh huỷ đi từ ứng dụng khách qua API Gateway tới **Booking/Order Service**; thời điểm `acceptedAt` do **Matching & Dispatch**/**Trip Lifecycle** ghi khi tài xế nhận; phí được thu qua **Payment & Wallet** gọi cổng VíXanh; sự kiện huỷ đi qua Kafka để **Notification** báo cho tài xế. Mỗi service chạy nhiều bản trên nhiều máy, mỗi máy có đồng hồ riêng.

Phòng lab gộp tất cả vào một tiến trình: hàm thuần `customerCancellationFee` trong `src/domain/cancel.ts`, được `cancelByCustomer` trong `src/app/trips.ts` gọi với thời điểm lấy từ **một** đồng hồ tiêm vào (`Clock`). Lab không có nhiều máy chủ, nên lệch đồng hồ giữa các máy chỉ được **mô phỏng** trong test.

### Rủi ro có thể xảy ra

| Rủi ro | Ai thấy failure | Loại nguồn gốc |
|---|---|---|
| Sai mốc 2 phút (`<=` thay vì `<`) | Tài xế (không được bù), bộ phận đối soát | Defect trong code |
| Đặc tả bỏ sót một quy tắc khách cần (ví dụ trả lại lượt voucher khi chuyến bị huỷ) | Khách | Defect trong đặc tả |
| Đồng hồ giữa các máy lệch vài giây | Khách bị thu oan, hoặc tài xế mất phí | Môi trường |
| Ca test viết sai kỳ vọng ở mốc 2:00 | Đội phát triển (báo lỗi giả, hoặc bỏ lọt) | Defect trong test script |
| Thu phí huỷ hai lần khi thử lại cổng thanh toán | Khách | Defect trong code tích hợp — R-PAY-04 |

## 4. Phân tích kỹ thuật

### 4.1 Tách bốn tầng của một sự cố

Câu chuyện giả định: bản phát hành đầu của tính năng phí huỷ có một dòng so sánh sai. Một tuần sau, nhóm tài xế xe máy ở TP. Hồ Chí Minh báo không nhận được phí bù dù khách huỷ «sau đúng hai phút». Tách sự cố theo bốn tầng:

| Tầng | Câu hỏi | Trả lời trong sự cố | Ai xử lý | Hành động |
|---|---|---|---|---|
| Failure | Hệ thống đã làm sai điều gì, ai thấy? | Chuyến `accepted`, khách huỷ ở `acceptedAt + 120 giây` ⇒ phí 0 đ, lý do `grace_period`; đặc tả đòi 10.000 đ, `charged` | Tester, vận hành | Tái hiện, ghi defect report, đánh giá ảnh hưởng (bao nhiêu chuyến bị tính sai) |
| Defect | Khiếm khuyết nằm ở sản phẩm nào, chỗ nào? | `cancel.ts`, điều kiện miễn phí viết `<=` | Lập trình viên (debugging) | Sửa thành `<`, thêm ca test ở mốc 2:00 |
| Error | Ai đã làm sai điều gì? | Hiểu «trong vòng 2 phút» là tính cả mốc 2:00 | Nhóm (không đổ lỗi cá nhân) | Thống nhất cách đọc mốc thời gian |
| Root cause | Hoàn cảnh nào khiến error xảy ra và lọt qua? | Đặc tả cũ chỉ viết «trong vòng 2 phút», không có ví dụ mốc; không review đặc tả; không có quy ước test giá trị biên | Trưởng nhóm, BA, QA | Viết lại đặc tả có ví dụ mốc (như R-CANCEL-02 hiện tại), thêm bước review, đưa «test biên» vào checklist |

Điểm quan trọng: sửa dòng code chỉ xử lý **một** defect. Nếu root cause là «đặc tả không có ví dụ mốc», cùng lỗi sẽ quay lại ở R-CANCEL-04 (`>` hay `≥` ở mốc tài xế trễ), R-QUOTE-01 (báo giá hết hạn đúng lúc `expiresAt`), R-STATE-05 (huỷ tìm tài xế đúng mốc 5 phút)… SPEC hiện tại của lab ghi rõ mốc cho từng chỗ đó — đó là kiểu biện pháp chặn ở tầng root cause.

### 4.2 Lỗi code và lỗi nghiệp vụ

Đề bài của bài này là «phân biệt lỗi code và lỗi nghiệp vụ». Dùng thuật ngữ ISTQB, câu hỏi thật là: **defect nằm trong sản phẩm công việc nào?**

| | A. Lỗi code | B. Lỗi nghiệp vụ (defect trong đặc tả) | C. Failure không do defect |
|---|---|---|---|
| Ví dụ Tới Nơi | Đặc tả R-CANCEL-02 đúng; code viết `<=` | Giả sử đặc tả **quên** câu «chuyến bị huỷ trả lại lượt voucher» (R-VOUCHER-07). Code làm đúng bản đặc tả thiếu đó | Code đúng; máy tính phí chạy nhanh 5 giây so với máy ghi `acceptedAt` |
| Failure | Huỷ đúng 2:00 không bị tính phí | Khách bị hệ thống huỷ vì không có tài xế (`no_driver`), đặt lại thì `GIAM20` báo `VOUCHER_USAGE_LIMIT` | Khách huỷ ở 1:57 thực tế bị thu 10.000 đ |
| Test theo đặc tả có bắt được? | **Có** — ca biên 2:00 đỏ | **Không** — mọi ca theo đặc tả đều xanh vì đặc tả sai từ gốc | Thường **không** trên một máy; cần môi trường giống production |
| Cách phát hiện hiệu quả | Dynamic testing với giá trị biên; review code | Review đặc tả với người dùng/nghiệp vụ (static testing), validation, phản hồi của khách | Kiểm thử trên môi trường nhiều máy, giám sát vận hành |
| Sửa ở đâu | Code + ca test | Đặc tả trước, rồi code và test suy ra từ nó | Môi trường (đồng bộ thời gian) hoặc thiết kế (một nguồn thời gian duy nhất) |

Nhận xét:

- **Lỗi nghiệp vụ nguy hiểm hơn với tester**: test viết từ đặc tả sai sẽ «xác nhận» luôn cái sai. Đây là một lý do syllabus nói defect ở sản phẩm làm sớm thường dẫn tới defect ở sản phẩm làm sau — kể cả bộ test. Cặp verification/validation ở [TEST-005](/articles/verification-va-validation) giải thích vì sao phải có góc nhìn thứ hai.
- **Cùng một failure có thể đến từ A, B hoặc C.** Khi tài xế báo «không được bù phí», chưa thể kết luận ngay đó là lỗi code. Phải đọc lại đặc tả, đọc code, kiểm cả môi trường.
- Ranh giới A/B không phải lúc nào cũng rõ. Nếu đặc tả chỉ ghi «trong vòng 2 phút», việc lập trình viên chọn `<=` là error do đặc tả mơ hồ — defect có mặt ở **cả hai** sản phẩm: một câu đặc tả không kiểm chứng được và một dòng code sai so với ý định nghiệp vụ.

### 4.3 Khi nào defect không gây failure

Defect `<=` của ví dụ chỉ gây failure khi **đồng thời**: chuyến ở `accepted` (không phải `arriving`), và khách huỷ **đúng** tại mốc 120.000 mili-giây. Với đồng hồ tính tới mili-giây, xác suất một lần huỷ ngẫu nhiên rơi đúng mốc đó rất nhỏ. Defect có thể «ngủ» hàng tháng trên production.

Ngược lại, có những defect luôn gây failure khi chạy tới (ví dụ viết nhầm mức phí ô tô 2.000 đ thay vì 20.000 đ), và có defect không bao giờ gây failure (ví dụ một nhánh code chết không ai gọi tới). Mức độ nghiêm trọng của defect vì thế không chỉ phụ thuộc «sai bao nhiêu» mà còn «bao lâu mới lộ ra một lần».

Hệ quả cho tester: test với dữ liệu «bình thường» hay ngẫu nhiên gần như không bao giờ chạm đúng mốc 120.000 ms. Muốn defect kiểu này lộ ra phải **chủ động** chọn giá trị biên (Series 04) và kiểm soát được thời gian (đồng hồ tiêm vào, SPEC R-ARCH-03).

### 4.4 Failure do môi trường

Trong hệ thống đích, `acceptedAt` được ghi trên máy A (Dispatch/Trip), còn phí huỷ tính trên máy B (Booking). Nếu đồng hồ máy B nhanh hơn 5 giây, khách bấm huỷ lúc 1:57 thực tế nhưng máy B tính ra 2:02 ⇒ thu phí. Code **đúng** từng ký tự; failure đến từ điều kiện môi trường.

Câu hỏi hay để thảo luận: đây có thật sự «không có defect»? Một kiến trúc sư có thể nói thiết kế có defect vì để hai máy so thời gian của nhau thay vì dùng một nguồn thời gian. Cả hai cách nhìn đều hợp lệ — điều quan trọng là failure này **không thể** tìm thấy bằng cách đọc `cancel.ts`. Lab dùng một đồng hồ tiêm vào (`Clock`, SPEC R-ARCH-03) nên tránh được loại failure này, nhưng hệ thống nhiều service thì phải tính tới.

## 5. Thực hành

### 5.1 Bản cài đặt có lỗi

Thư mục `examples/toi-noi-testing/articles/error-defect-failure-root-cause/` có một bản chép của `src/domain/cancel.ts`, đổi **đúng một ký tự** (không sửa `src/`):

```ts
// buggy-cancel.ts — chép từ src/domain/cancel.ts
const sinceAcceptedMs = input.now.getTime() - input.acceptedAt.getTime();
// DEFECT: đặc tả R-CANCEL-02 là «now − acceptedAt < 2 phút»; ở đây viết `<=`.
if (input.status === 'accepted' && sinceAcceptedMs <= FREE_CANCEL_MINUTES * MINUTE_MS) {
  return { fee: 0, reason: 'grace_period' };
}
```

Bản đúng trong `src/domain/cancel.ts` dùng `sinceAcceptedMs < FREE_CANCEL_MINUTES * MINUTE_MS`.

### 5.2 Test data và kết quả mong đợi

Dữ liệu chung: chuyến xe máy (`bike`), tài xế nhận lúc 09:00:00 giờ Việt Nam ngày 04/11/2026, `etaMinutes = 5` (mốc tài xế trễ là 15:00 sau khi nhận — nằm ngoài vùng xét).

| Ca | Trạng thái | Huỷ sau khi nhận | Mong đợi (SPEC) | Bản đúng | Bản lỗi | Failure? |
|---|---|---|---|---|---|---|
| TC-01 | `accepted` | 1:59 | 0 đ, `grace_period` | 0 đ, `grace_period` | 0 đ, `grace_period` | Không |
| TC-02 | `accepted` | **2:00** | 10.000 đ, `charged` | 10.000 đ, `charged` | **0 đ, `grace_period`** | **Có** |
| TC-03 | `accepted` | 2:01 | 10.000 đ, `charged` | 10.000 đ, `charged` | 10.000 đ, `charged` | Không |
| TC-04 | `accepted` | 5:00 | 10.000 đ, `charged` | 10.000 đ, `charged` | 10.000 đ, `charged` | Không |
| TC-05 | `arriving` | 2:00 | 10.000 đ, `charged` | 10.000 đ, `charged` | 10.000 đ, `charged` | Không — nhánh lỗi không được thực thi |
| TC-06 | `accepted`, đồng hồ lệch +5 s | 1:57 thực tế | 0 đ, `grace_period` | **10.000 đ, `charged`** | — | **Có** — do môi trường, mã đúng |

Cột «Bản đúng» và «Bản lỗi» là kết quả chạy thật (mục 5.3). Chú ý TC-06: hàm đúng nhận thời điểm 2:02 (đồng hồ máy B) nên tính phí — failure xảy ra mà không có defect nào trong `cancel.ts`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 240" role="img" aria-label="Trục thời gian từ 0:00 tới 5:00 sau khi tài xế nhận. Dải trên là đặc tả và bản đúng: miễn phí từ 0:00 tới trước 2:00, từ đúng 2:00 trở đi thu 10.000 đồng. Dải dưới là bản lỗi: miễn phí từ 0:00 tới hết 2:00, sau 2:00 mới thu. Hai dải chỉ khác nhau tại đúng mốc 2:00; quét 601 giây thì chỉ lệch ở giây thứ 120.">
<text x="460" y="22" text-anchor="middle" class="t-b">Phí huỷ theo thời gian kể từ acceptedAt — chuyến xe máy ở trạng thái accepted</text>
<text x="80" y="54" class="t-small">Đặc tả R-CANCEL-02 = bản đúng (src/domain/cancel.ts): now − acceptedAt &lt; 2 phút</text>
<rect x="80" y="62" width="312" height="26" rx="4" class="f-phys-soft s-phys"/>
<text x="236" y="80" text-anchor="middle" class="t-mono">0 đ · grace_period</text>
<rect x="392" y="62" width="468" height="26" rx="4" class="f-warn-soft s-warn"/>
<text x="626" y="80" text-anchor="middle" class="t-mono">10.000 đ · charged</text>
<text x="80" y="116" class="t-small">Bản lỗi (buggy-cancel.ts): now − acceptedAt &lt;= 2 phút</text>
<rect x="80" y="124" width="316" height="26" rx="4" class="f-phys-soft s-phys"/>
<text x="236" y="142" text-anchor="middle" class="t-mono">0 đ · grace_period</text>
<rect x="396" y="124" width="464" height="26" rx="4" class="f-warn-soft s-warn"/>
<text x="626" y="142" text-anchor="middle" class="t-mono">10.000 đ · charged</text>
<line x1="392" y1="44" x2="392" y2="176" class="s-warn dash" stroke-width="1.6"/>
<text x="400" y="170" class="t-10 t-warn">chỉ khác nhau tại đúng mốc 2:00 (120.000 ms)</text>
<line x1="80" y1="186" x2="860" y2="186" class="s-muted"/>
<line x1="80" y1="182" x2="80" y2="190" class="s-muted"/><text x="80" y="204" text-anchor="middle" class="t-mono">0:00</text>
<line x1="236" y1="182" x2="236" y2="190" class="s-muted"/><text x="236" y="204" text-anchor="middle" class="t-mono">1:00</text>
<line x1="392" y1="182" x2="392" y2="190" class="s-warn"/><text x="392" y="204" text-anchor="middle" class="t-mono t-b t-warn">2:00</text>
<line x1="548" y1="182" x2="548" y2="190" class="s-muted"/><text x="548" y="204" text-anchor="middle" class="t-mono">3:00</text>
<line x1="704" y1="182" x2="704" y2="190" class="s-muted"/><text x="704" y="204" text-anchor="middle" class="t-mono">4:00</text>
<line x1="860" y1="182" x2="860" y2="190" class="s-muted"/><text x="860" y="204" text-anchor="middle" class="t-mono">5:00</text>
<text x="460" y="228" text-anchor="middle" class="t-small">Quét 601 giây (0:00 → 10:00): hai bản chỉ lệch ở giây thứ 120 — test ngẫu nhiên gần như không bao giờ chạm tới</text>
</svg>
</div>
<figcaption><b>Hình 2 · Một ký tự, một điểm thời gian.</b> Defect <code>&lt;=</code> chỉ lộ thành failure tại đúng mốc biên. Dải dưới được vẽ lệch vài pixel để thấy mốc 2:00 thuộc về vùng miễn phí.</figcaption>
</figure>

### 5.3 Chạy test

Tệp `chuoi-loi.test.ts` có 7 ca. Ca quan trọng nhất: **cùng một ca kiểm thử theo đặc tả** chạy trên hai bản. Trên bản đúng nó đạt; trên bản lỗi nó phải đỏ — được viết bằng `it.fails` của vitest (ca được tính là đạt khi và chỉ khi phần thân thất bại), để bộ test của lab vẫn xanh:

```ts
it('ca kiểm thử theo đặc tả: huỷ đúng 2:00 ⇒ thu 10.000 đ (bản đúng của src/ đạt)', () => {
  expect(customerCancellationFee(trip(120))).toEqual({ fee: 10_000, reason: 'charged' });
});

it.fails('CÙNG ca kiểm thử chạy trên bản lỗi ⇒ đỏ (it.fails: kỳ vọng ca này thất bại)', () => {
  expect(buggyCustomerCancellationFee(trip(120))).toEqual({ fee: 10_000, reason: 'charged' });
});

it('quét từng giây 0:00 → 10:00: hai bản chỉ lệch nhau tại đúng 1 điểm (giây thứ 120)', () => {
  const lech: number[] = [];
  for (let s = 0; s <= 600; s++) {
    const dung = customerCancellationFee(trip(s));
    const loi = buggyCustomerCancellationFee(trip(s));
    if (dung.fee !== loi.fee || dung.reason !== loi.reason) lech.push(s);
  }
  expect(lech).toEqual([120]);
});
```

Lệnh và kết quả chạy thật (Node 22, vitest 3.2.7, rút gọn tiền tố đường dẫn mỗi dòng):

```bash
cd examples/toi-noi-testing
npx vitest run articles/error-defect-failure-root-cause --reporter=verbose
```

```text
 ✓ … > ca kiểm thử theo đặc tả: huỷ đúng 2:00 ⇒ thu 10.000 đ (bản đúng của src/ đạt) 2ms
 ✓ … > CÙNG ca kiểm thử chạy trên bản lỗi ⇒ đỏ (it.fails: kỳ vọng ca này thất bại) 9ms
 ✓ … > failure quan sát được: bản lỗi trả 0 đ, lý do grace_period ở mốc 2:00 0ms
 ✓ … > defect «ngủ»: ở 1:59, 2:01 và 5:00 hai bản cho cùng kết quả 0ms
 ✓ … > quét từng giây 0:00 → 10:00: hai bản chỉ lệch nhau tại đúng 1 điểm (giây thứ 120) 1ms
 ✓ … > defect không được thực thi: chuyến đã ở arriving thì nhánh lỗi bị bỏ qua, không failure 0ms
 ✓ … > failure KHÔNG do defect: mã đúng, nhưng đồng hồ máy tính phí chạy nhanh 5 giây (giả định môi trường) 0ms

 Test Files  1 passed (1)
      Tests  7 passed (7)
```

Đọc kết quả theo bốn khái niệm:

- Ca 1 và 2 cho thấy **test làm lộ failure**, không chỉ ra defect ở dòng nào. Tìm ra `<=` là việc của debugging ([TEST-001](/articles/software-testing-la-gi-testing-khac-debugging)).
- Ca 4, 5, 6 cho thấy **defect có mặt nhưng không gây failure** ở hầu hết đầu vào — kể cả quét 601 giây cũng chỉ lệch ở một điểm.
- Ca 7 cho thấy **failure không cần defect**.
- Không ca nào cho thấy **error** hay **root cause** — hai thứ đó không nằm trong mã, phải tìm bằng trao đổi và phân tích.

### 5.4 Ghi nhận sự cố tách bốn tầng

Một defect report đầy đủ (mẫu chi tiết ở [TEST-010](/articles/testware-san-pham-kiem-thu)) nên tách rõ phần quan sát và phần phân tích. Phần dưới chỉ minh hoạ cách đặt bốn khái niệm vào đúng ô:

```text
Tiêu đề        : Khách huỷ đúng 2:00 sau khi tài xế nhận không bị tính phí huỷ (R-CANCEL-02)
FAILURE        : Chuyến bike, accepted lúc 09:00:00, khách huỷ 09:02:00
                 Thực tế : cancellationFee = 0, feeReason = grace_period
                 Mong đợi: cancellationFee = 10.000, feeReason = charged
                 Tái hiện: TC-02; luôn tái hiện được với đồng hồ cố định
DEFECT         : (điền sau debugging) cancel.ts — điều kiện miễn phí dùng <= thay vì <
ERROR          : (điền khi phân tích) hiểu «trong vòng 2 phút» là gồm cả mốc 2:00
ROOT CAUSE     : (điền sau RCA) đặc tả không có ví dụ mốc; không có quy ước test biên
Ảnh hưởng      : chuyến bị huỷ đúng mốc 2:00 từ ngày phát hành — truy vấn đối soát để bù cho tài xế
```

Người viết báo cáo ban đầu (tester) thường chỉ điền chắc chắn phần **failure**. Phần defect, error, root cause do người khác bổ sung sau — và có thể sai lúc đầu. Ghi lẫn giả thuyết «chắc do `<=`» vào ô failure là một lỗi trình bày phổ biến.

## 6. Góc nhìn nâng cao

**Defect trong chính bộ test.** Syllabus nói defect có thể nằm trong test script. Nếu ca test TC-02 viết kỳ vọng `0 đ` (người viết test cũng hiểu nhầm «trong vòng 2 phút»), bản lỗi sẽ **đạt** và bản đúng sẽ **đỏ**. Test đỏ trên bản đúng là một **false-positive result** (báo lỗi giả); test xanh trên bản lỗi là bỏ lọt (thường gọi false negative). Cả hai đều do defect trong testware. CTFL mục 5.5 nhắc rằng một anomaly được báo cáo, sau khi phân tích, có thể hoá ra là báo lỗi giả hoặc một yêu cầu thay đổi chứ không phải defect.

**Một-nhiều trong cả hai chiều.** Một defect có thể gây nhiều failure khác nhau (`<=` vừa làm sai phí, vừa làm sai thống kê «tỉ lệ huỷ có phí»). Một failure có thể cần nhiều defect cùng lúc mới xảy ra, hoặc defect này che defect kia (sửa defect A xong mới lộ failure của defect B).

**Defect trong cấu hình và dữ liệu.** Voucher `HN10` khai `cityCodes` sai trong dữ liệu production là defect nằm trong một sản phẩm hỗ trợ (dữ liệu cấu hình), không phải code. Failure giống hệt lỗi code (khách ở `SG` dùng được `HN10`), nhưng sửa ở chỗ khác hoàn toàn.

**Root cause không phải «người gây lỗi».** RCA tìm hoàn cảnh (đặc tả mơ hồ, áp lực thời gian, thiếu đào tạo), không tìm người để phạt. Đổ lỗi cá nhân làm mọi người giấu error, và root cause thật không bao giờ được gỡ. Kỹ thuật RCA chi tiết thuộc Series 12.

**Môi trường trong hệ phân tán.** Ngoài lệch đồng hồ, hệ thống đích của Tới Nơi có những failure kiểu «môi trường» khác: Kafka giao sự kiện trễ hoặc trùng, VíXanh trả lời chậm hơn 3 giây, mạng điện thoại tài xế chập chờn. Nhiều trường hợp ranh giới «môi trường hay defect thiết kế» không rõ: code không chịu được sự kiện trùng là defect, vì kiến trúc đã nói sự kiện trùng là điều chắc chắn xảy ra (anti-pattern «exactly-once assumption»). Lab chưa có Kafka nên chỉ thảo luận ở mức thiết kế.

**Sai lầm thường gặp:**

| Sai lầm | Đúng là |
|---|---|
| «Test tìm ra defect» | Dynamic testing làm lộ **failure**; debugging mới định vị **defect**. Static testing thì tìm thẳng defect |
| «Không failure ⇒ không defect» | Defect có thể ngủ (mục 4.3) |
| «Failure ⇒ chắc chắn có bug trong code» | Có thể do đặc tả, dữ liệu, cấu hình, test sai hoặc môi trường |
| Dùng «error» để chỉ thông báo lỗi trên màn hình | Theo ISTQB, error là sai lầm của **con người**; thông báo `INVALID_TRANSITION` là hành vi của hệ thống |
| «Root cause = dòng code sai» | Dòng code sai là defect; root cause là lý do dòng đó được viết ra và lọt qua |
| Sửa defect xong là đóng | Hỏi thêm: còn chỗ nào cùng root cause? (R-CANCEL-04, R-QUOTE-01, R-STATE-05) |

## 7. Bài tập và câu hỏi ôn tập

**Bài tập 1 — đừng vội gán defect cho failure.** Khách huỷ chuyến ô tô ở trạng thái `arriving`, 25 phút sau khi tài xế nhận, `etaMinutes = 10`, và bị thu 20.000 đ. Khách khiếu nại vì tài xế tới muộn. Khi đọc code, một lập trình viên thấy nhánh `driver_late` viết `sinceAcceptedMs >= lateAfterMs` thay vì `>` và kết luận «đã tìm ra nguyên nhân». (a) Theo SPEC, phí đúng là bao nhiêu? (b) Defect `>=` có giải thích được failure này không; nếu không, nó gây failure ở thời điểm nào? (c) Nêu một error và một root cause hợp lý cho defect `>=`.

<details>
<summary><b>Lời giải</b></summary>
<p>Mốc tài xế trễ = 10 + 10 = 20 phút sau khi nhận. Ở phút 25, <code>now &gt; acceptedAt + 20 phút</code> ⇒ theo R-CANCEL-04 phí phải là <b>0 đ, <code>driver_late</code></b>. Vậy việc khách bị thu 20.000 đ ở phút 25 là <b>failure</b>, nhưng nó <b>không</b> do <code>&gt;=</code> gây ra: với 25 &gt; 20 thì cả <code>&gt;</code> lẫn <code>&gt;=</code> đều cho miễn phí. Phải tìm defect khác (ví dụ code không xét nhánh <code>driver_late</code> cho trạng thái <code>arriving</code>, hoặc dùng sai <code>etaMinutes</code>), hoặc kiểm môi trường (đồng hồ, dữ liệu <code>acceptedAt</code> bị ghi đè).</p>
<p>Nếu code chỉ có defect <code>&gt;=</code>, failure duy nhất là ở <b>đúng</b> phút 20:00: SPEC nói «đúng bằng mốc vẫn thu phí», code lại miễn phí.</p>
<p>Bài học: đừng nhận ngay defect đầu tiên nhìn thấy làm nguyên nhân của failure đang điều tra. Error hợp lý cho defect <code>&gt;=</code>: hiểu «sau 10 phút» là gồm cả mốc. Root cause hợp lý: đặc tả không có ví dụ mốc (lab đã sửa bằng câu «Đúng bằng mốc đó vẫn thu phí»), chưa có quy ước test biên.</p>
</details>

**Bài tập 2 — lỗi code hay lỗi nghiệp vụ?** Phân loại mỗi trường hợp: (A) lỗi code, (B) defect trong đặc tả, (C) môi trường/dữ liệu, (D) defect trong test.

1. SPEC đúng; code tính phụ phí mưa trước hệ số cao điểm nên phụ phí bị nhân 1,25.
2. Đặc tả bản nháp quên nói đơn đồ ăn luôn đi xe máy; code cho chọn ô tô; khách chọn ô tô cho đơn trà sữa và bị tính cước ô tô.
3. Máy chủ production đặt múi giờ UTC, code dùng giờ của máy để xét cao điểm.
4. Ca test kỳ vọng cước 7,4 km cao điểm mưa `GIAM20` là 51.700 đ; hệ thống trả 52.000 đ.

<details>
<summary><b>Đáp án</b></summary>
<p>1 — <b>A</b>: R-FARE-03/05 nói rõ mưa cộng sau cao điểm. 2 — <b>B</b>: đặc tả thiếu quy tắc (R-FOOD-06 trong SPEC hiện tại); code đúng với bản nháp. 3 — <b>A</b> là chính: R-GEN-02 yêu cầu giờ Việt Nam bất kể múi giờ máy chủ, nên code dựa vào múi giờ máy là defect; cấu hình múi giờ chỉ là điều kiện làm defect lộ ra (nếu đặc tả không có R-GEN-02 thì đó là B). 4 — <b>D</b>: 51.700 đ là số trước bước làm tròn lên (R-FARE-08, R-FARE-09); test đỏ trên hệ thống đúng là báo lỗi giả.</p>
</details>

**Câu hỏi trắc nghiệm** (tự soạn theo phong cách ISTQB, mỗi câu một đáp án đúng).

**Câu 1 (K1).** Theo ISTQB, thuật ngữ nào chỉ hành động của con người tạo ra kết quả sai?

- a) Failure
- b) Defect
- c) Error
- d) Anomaly

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — error (mistake) là sai lầm của con người. a) failure là hành vi sai khi hệ thống chạy. b) defect là khiếm khuyết trong sản phẩm công việc do error tạo ra. d) anomaly là điều bất thường được ghi nhận, chưa chắc là defect.</p>
</details>

**Câu 2 (K2).** Lập trình viên hiểu nhầm mốc 2 phút và viết <code>&lt;=</code> thay vì <code>&lt;</code>. Khách huỷ đúng 2:00 không bị tính phí. Ghép nào đúng?

- a) Hiểu nhầm = defect; dòng <code>&lt;=</code> = error; không tính phí = failure
- b) Hiểu nhầm = error; dòng <code>&lt;=</code> = defect; không tính phí = failure
- c) Hiểu nhầm = root cause; dòng <code>&lt;=</code> = failure; không tính phí = defect
- d) Hiểu nhầm = error; dòng <code>&lt;=</code> = failure; không tính phí = defect

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — người làm sai (error) tạo ra khiếm khuyết trong code (defect), khi chạy thì hệ thống cho kết quả sai (failure). a) đảo error và defect. c) hiểu nhầm là error; root cause là hoàn cảnh dẫn tới hiểu nhầm (đặc tả mơ hồ…); và dòng code không phải failure. d) đảo defect và failure.</p>
</details>

**Câu 3 (K2).** Phát biểu nào về quan hệ giữa defect và failure là ĐÚNG?

- a) Mọi defect đều gây failure ngay lần chạy đầu tiên.
- b) Mọi failure đều do một defect trong mã nguồn gây ra.
- c) Một defect có thể chỉ gây failure trong điều kiện đặc biệt, hoặc không bao giờ gây failure.
- d) Defect chỉ tồn tại trong mã nguồn, không tồn tại trong tài liệu.

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> đúng — như defect <code>&lt;=</code> chỉ lộ ở đúng mốc 2:00. a) sai: nhiều defect «ngủ». b) sai: failure còn có thể do điều kiện môi trường. d) sai: defect có thể nằm trong đặc tả, test script, file build.</p>
</details>

**Câu 4 (K2).** Sau sự cố phí huỷ, đội Tới Nơi muốn ngăn các lỗi mốc thời gian **tương tự** ở báo giá hết hạn và tài xế trễ. Hoạt động nào phù hợp nhất?

- a) Debugging dòng <code>&lt;=</code> trong <code>cancel.ts</code>.
- b) Root cause analysis để tìm và gỡ hoàn cảnh dẫn tới error, ví dụ bổ sung ví dụ mốc vào đặc tả và quy ước test biên.
- c) Chạy lại bộ test hồi quy hiện có nhiều lần hơn.
- d) Ghi nhận failure vào defect report và đóng lại sau khi sửa.

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — gỡ root cause nhằm ngăn hoặc giảm defect cùng loại. a) chỉ sửa một defect. c) bộ test hiện có không có ca biên nên chạy nhiều hơn không giúp (xem nguyên lý «tests wear out» ở <a href="/articles/bay-nguyen-ly-kiem-thu">TEST-003</a>). d) cần thiết nhưng không ngăn lặp lại.</p>
</details>

**Câu 5 (K2).** Chuyến của khách bị thu phí huỷ dù khách huỷ trước mốc 2 phút. Điều tra cho thấy mã tính phí đúng, nhưng đồng hồ máy chủ tính phí nhanh hơn đồng hồ máy ghi thời điểm nhận chuyến. Phân loại nào phù hợp nhất với syllabus?

- a) Failure do điều kiện môi trường, không phải do defect trong mã tính phí.
- b) Defect trong mã tính phí.
- c) Error của khách hàng.
- d) Không có failure vì mã đúng.

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>a)</b> đúng — syllabus nêu failure có thể do điều kiện môi trường. b) mã tính phí đúng với đầu vào nó nhận. c) khách không làm sai gì. d) failure là hành vi sai quan sát được; khách bị thu oan là failure, bất kể nguyên nhân.</p>
</details>

**Câu 6 (K2).** Ca test của Tới Nơi kỳ vọng phí 0 đ khi khách huỷ đúng 2:00. Hệ thống trả 10.000 đ và ca test đỏ. Đặc tả nói đúng 2:00 là có phí. Nhận định nào đúng?

- a) Hệ thống có defect, vì test đỏ.
- b) Đây là báo lỗi giả (false-positive result): defect nằm trong ca test, hệ thống đúng.
- c) Đây là failure do môi trường.
- d) Đây là root cause của sự cố.

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b> đúng — test script cũng có thể chứa defect; đối chiếu với đặc tả cho thấy hệ thống đúng. a) test đỏ chưa chắc hệ thống sai — anomaly cần được phân tích (CTFL mục 5.5). c) không có dấu hiệu môi trường. d) root cause là hoàn cảnh dẫn tới error (ví dụ người viết test hiểu nhầm), không phải bản thân kết quả đỏ.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* **v4.0.1** (2024), mục 1.2.3 «Errors, Defects, Failures, and Root Causes», LO FL-1.2.3 (K2); liên quan mục 1.1.2 (testing và debugging) và mục 5.5 (defect management): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary: https://glossary.istqb.org/ — mục *error*, *defect*, *failure*, *root cause*, *root cause analysis*, *false-positive result*.
- Trang chủ ISTQB: https://www.istqb.org/
- IEEE Std 1044-2009, *IEEE Standard Classification for Software Anomalies* — tham khảo cách phân loại bất thường, ngoài phạm vi CTFL.
- ISO/IEC/IEEE 29119-1, *Software and systems engineering — Software testing — Part 1: General concepts* — khái niệm chung về kiểm thử.
- Phòng lab Tới Nơi: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing — SPEC R-CANCEL, R-GEN-02, R-ARCH-03; mã bài tại `articles/error-defect-failure-root-cause/`.
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-001 · Testing khác Debugging](/articles/software-testing-la-gi-testing-khac-debugging) · [TEST-002 · Vì sao cần kiểm thử](/articles/vi-sao-can-kiem-thu) · [TEST-003 · Bảy nguyên lý kiểm thử](/articles/bay-nguyen-ly-kiem-thu) · [TEST-005 · Verification và Validation](/articles/verification-va-validation) · [TEST-010 · Testware](/articles/testware-san-pham-kiem-thu).

Bài tiếp theo: [Verification và Validation — xác minh thuật toán tính giá, xác nhận trải nghiệm người dùng](/articles/verification-va-validation).
