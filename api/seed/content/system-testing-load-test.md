---
slug: system-testing-load-test
title: "Load Test: tải dự kiến có thực sự được tạo ra và xử lý đúng?"
summary: "Load testing qua API báo giá của Tới Nơi: dựng workload từ nghiệp vụ, phân biệt arrival rate, request rate, concurrency và VU, viết scheduler open-model không chờ response, kế toán planned = sent + dropped, gate đặt trước phép đo và đọc một lượt 300 request thực đo trên PGlite mà không suy ra capacity production."
category: testing
level: advanced
tags: [testing, sys-08, load-testing, performance-testing, arrival-rate, workload-model, open-model, k6, pglite, test-oracle]
language: vi
---

Báo cáo tải ghi «p95 = 6 ms, 0 lỗi, PASS». Nghe rất đẹp — cho tới khi ai đó hỏi: kế hoạch là 30 request mỗi giây, vậy thực tế đã *gửi* được bao nhiêu? Nếu generator chỉ gửi được một nửa vì nó tự chờ response, thì 0 lỗi kia là 0 lỗi của một thí nghiệm khác, nhẹ hơn thí nghiệm ta định làm. Bài này trả lời: **làm sao chắc rằng tải dự kiến đã thực sự được tạo ra, được xử lý đúng nghiệp vụ, và con số cuối báo cáo được phép nói điều gì.** Ví dụ là endpoint báo giá `POST /api/quotes` của lab Tới Nơi, chạy trên PGlite với một scheduler gửi theo lịch.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-08</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 1</td></tr>
<tr><th scope="row">Cấp độ</th><td>Nâng cao — đã nắm latency, percentile, throughput và baseline ở SYS-07; biết HTTP và TypeScript</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>System · Non-functional (hiệu năng — load testing), giữ oracle nghiệp vụ trên từng response</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-performance-test">SYS-07 Performance Test</a>; nền tảng phi chức năng ở <a href="/articles/non-functional-testing-tong-quan">Non-functional testing tổng quan</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (script): 300 slot theo lịch 10/s × 3 s → 30/s × 8 s → 10/s × 3 s vào <code>POST /api/quotes</code> (R-API-07, oracle 35.000 theo R-FARE-01…08), PGlite, HTTP loopback — <code>examples/toi-noi-testing/articles/system-testing-load-test/load.ts</code>, evidence <code>load-observed.json</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>Grafana k6 — Load test types, Arrival-rate VU allocation, Dropped iterations; <code>SPEC.md</code> của lab (R-API-07, R-FARE, R-NFR-03, R-NFR-07)</td></tr>
</tbody>
</table>
</div>

## 1. Tình huống và mục tiêu

### 1.1. Vì sao baseline của SYS-07 chưa đủ

Ở [SYS-07](/articles/system-testing-performance-test), một worker gửi request mới **chỉ khi request trước đã xong**. Đó là mô hình *closed loop*: tốc độ gửi tự điều chỉnh theo tốc độ trả lời. Khi server chậm đi, generator cũng chậm theo — và báo cáo vẫn có thể ghi «0 lỗi».

Người dùng Tới Nơi không cư xử như vậy. Lúc 17:00, khách mới vẫn tiếp tục mở ứng dụng và bấm báo giá, bất kể những khách trước đang chờ lâu hay nhanh. Tải đến theo *nhịp đến* (arrival), không theo nhịp hoàn tất. Muốn đánh giá hệ thống ở một mức tải dự kiến, ta cần trả lời tách bạch ba câu:

1. Ta **định** gửi bao nhiêu (planned)?
2. Ta **thực gửi** được bao nhiêu (sent), số còn lại bị bỏ vì lý do gì (dropped)?
3. Hệ thống **xử lý đúng** được bao nhiêu, nhanh tới đâu?

### 1.2. Đối tượng và điều kiện tiên quyết

Bài dành cho QA/SDET và developer đã đọc SYS-07: biết p95/p99 theo nearest-rank, phân biệt throughput với goodput, hiểu vì sao phải lưu mẫu gốc. Cần đọc được TypeScript async (`Promise`, `Set`, `await`) và biết `fetch`. Không cần cài k6: lab dùng một scheduler nhỏ tự viết để mọi dòng logic đều đọc được.

### 1.3. Mục tiêu đo được

Sau bài, bạn có thể:

- Viết workload model cho một endpoint từ dữ liệu nghiệp vụ, ghi rõ phần nào là giả định.
- Phân biệt arrival rate, request rate, concurrency, VU, throughput; ước lượng concurrency từ arrival rate và thời gian một iteration.
- Đọc và sửa một scheduler open-model: gửi theo lịch, không `await` response trong vòng slot, đo scheduler lag, ghi drop có lý do.
- Kiểm bất biến kế toán `planned = sent + dropped` và đặt gate gồm cả drop, lỗi nghiệp vụ lẫn latency trước khi chạy.
- Đọc báo cáo theo thứ tự đúng và phân loại «generator không đạt tải» khác với «hệ thống không chịu nổi».

## 2. Khái niệm và những điều dễ nhầm

**Load testing** đánh giá hệ thống ở mức tải được xác định là bình thường hoặc dự kiến, với tiêu chí về latency, lỗi, throughput và tài nguyên. Trong phân loại của Grafana k6, load test ở mức trung bình (average-load) khác smoke tải thấp, stress, spike, soak và breakpoint ở *mục tiêu* chứ không ở công cụ. Load test cần một **mức tải mục tiêu có căn cứ**; stress test ([SYS-09](/articles/system-testing-stress-test)) mới cố ý đi vượt mức đó.

«Tải» không chỉ là số người dùng. Nó gồm loại request, nhịp đến, hành trình nhiều bước, dữ liệu, thời gian chạy và trạng thái cache/DB. Năm đại lượng sau thường bị dùng lẫn:

| Đại lượng | Ý nghĩa | Điều dễ nhầm |
|---|---|---|
| Arrival rate | Số iteration được lên lịch **bắt đầu** mỗi giây | Không bằng request rate nếu một iteration có nhiều HTTP call |
| Request rate | Số HTTP request theo thời gian | Không đồng nghĩa số người dùng |
| Concurrency | Số tác vụ **đang** thực hiện cùng lúc | Không bằng số tài khoản trong dataset |
| VU | Worker/ngữ cảnh mô phỏng của công cụ | Cần đủ VU để đạt arrival rate khi response chậm |
| Throughput | Số hoàn tất trong cửa sổ đo | Có thể gồm cả lỗi trả về rất nhanh |

Hai mô hình sinh tải:

- **Closed model** — N worker, mỗi worker gửi, chờ, rồi gửi tiếp. Tốc độ gửi ≈ N / thời gian phản hồi. Server chậm ⇒ gửi ít đi. Hợp với baseline và với hệ thống có số client cố định (ví dụ một pool worker nội bộ).
- **Open model** — request đến theo lịch, độc lập với việc request trước đã xong chưa. Server chậm ⇒ request chồng lên nhau, concurrency tăng. Hợp với traffic người dùng công khai như báo giá.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Mức tải trong lab là giả định học tập</span>
<p>Lab chọn 30 arrival/giây làm «tải mục tiêu». Con số này <b>không</b> lấy từ telemetry sản phẩm. SPEC R-NFR-07 ghi mức tham chiếu toàn hệ thống lúc cao điểm khoảng 70 chuyến tạo mới/giây và 30.000 truy vấn đọc/giây, nhưng nói rõ đó là số «để dựng mô hình tải», không phải mục tiêu — và càng không phải yêu cầu cho một tiến trình lab trên một máy. Gate p95 &lt; 500 ms cùng ngưỡng với R-NFR-03, nhưng đo trong một process thì không chứng minh R-NFR-03 đạt hay không.</p>
</div>

## 3. Workload: từ nghiệp vụ tới thiết kế lab

### 3.1. Trong dự án thật

Workload model bắt đầu từ dữ liệu nghiệp vụ, không từ «cho 100 user chạy thử». Với báo giá của Tới Nơi, các câu hỏi cần có số liệu:

- Số phiên hoạt động theo giờ trong ngày; tỷ lệ cao điểm 11–13 h và 17–19 h (khớp khung giờ R-FARE-04).
- Một khách báo giá bao nhiêu lần trước khi đặt, hoặc bỏ đi không đặt.
- Phân bố theo dịch vụ (ride/parcel/food), thành phố, loại xe, quãng đường.
- Tỷ lệ đọc/ghi: một chuyến kéo theo nhiều lần hỏi lại trạng thái (trang khách hỏi mỗi 2 giây theo R-NFR-04).

Từ đó suy ra arrival rate **cho từng endpoint**, không chia đều tải tuỳ ý. Ví dụ giả định: 3 lần báo giá cho mỗi chuyến đặt ⇒ báo giá chiếm phần lớn request ghi, còn polling chiếm phần lớn request đọc.

### 3.2. Thiết kế lab và giới hạn của nó

| Thành phần | Thiết kế lab | Giới hạn |
|---|---|---|
| Operation | Một iteration = một `POST /api/quotes` | Không mô phỏng đặt chuyến, thanh toán, polling |
| Dataset | Khách C001, ride SG xe máy 5 km | Không có phân bố người dùng hay hot key đa dạng |
| Oracle | HTTP 200, `total` = 35.000, `quoteId` là chuỗi | Chưa kiểm mọi trường response |
| Storage | PGlite; mỗi quote tạo một bản ghi (R-QUOTE-01) | Một kết nối, không phải PostgreSQL nhiều connection |
| Clock/weather | 09:00 giờ VN cố định, không mưa | Không có biến thiên theo giờ |
| Profile | 10/s × 3 s → 30/s × 8 s → 10/s × 3 s | Ngắn, theo bước, không phải soak |

Oracle 35.000 tính tay từ SPEC, không gọi lại calculator: xe máy 12.000 + 4.500 × 5 = 34.500 (R-FARE-01), 09:00 không phải cao điểm (R-FARE-04), không mưa, không voucher, trên mức tối thiểu 15.000 (R-FARE-07), làm tròn lên bội 1.000 ⇒ 35.000 (R-FARE-08). Kiểm tiền trên **mọi** response dưới tải để bắt loại lỗi «nhanh nhưng sai».

Ba lựa chọn có chủ đích:

- **Không reset giữa các bước**: sau warm-up 20 request, dữ liệu được reset một lần; sau đó số quote tăng từ 0 lên 300 qua ba bước, phản ánh ghi tích luỹ trong lượt này.
- **Không dùng voucher**: voucher có giới hạn lượt, sẽ làm đổi mix lỗi giữa chừng.
- **Không đặt chuyến**: R-API-13 giới hạn 5 lần `POST /api/trips`/60 giây/khách; R-API-07 ghi rõ endpoint báo giá «không bị giới hạn tần suất». Chọn endpoint không có limiter là chọn đúng đối tượng cho câu hỏi này, **không phải** tắt limiter để kết quả đẹp.

## 4. Scheduler open-model: gửi theo lịch, không đợi response

### 4.1. Nguyên lý

Với tốc độ r slot/giây, slot thứ i có thời điểm dự kiến `due = t0 + i × 1000 / r`. Tới slot, script **khởi động** request rồi đi tiếp tới slot sau; nó chỉ giữ promise trong tập `active`. Response chậm không làm lịch dịch đi như ở closed loop.

Nhưng generator cũng là một chương trình: event loop của Node có thể bận, timer có thể thức dậy muộn. Vì vậy phải đo **scheduler lag** = thời điểm thực sự xét slot − `due`. Không đo lag thì không phân biệt được «hệ thống chậm» với «máy sinh tải chậm».

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 290" role="img" aria-label="So sánh hai mô hình sinh tải. Hàng trên, closed loop của SYS-07: mỗi request bắt đầu khi request trước kết thúc; khi request thứ hai chậm, các request sau bị đẩy lùi và tốc độ gửi giảm. Hàng dưới, open schedule của SYS-08: các slot đặt đều mỗi 33 ms trên trục thời gian; request chậm chồng lên request sau thay vì đẩy lịch; một slot không gửi được vì đầy giới hạn in-flight được ghi là dropped kèm lý do; scheduler lag là khoảng giữa due time và lúc thực gửi.">
<defs>
<marker id="sys08-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="16" y="30" class="t-b">Closed loop (SYS-07): gửi khi request trước xong</text>
<line x1="16" y1="100" x2="900" y2="100" class="s-phys" stroke-width="1.2"/>
<rect x="40" y="56" width="90" height="30" rx="6" class="f-phys-soft s-phys"/><text x="85" y="76" text-anchor="middle" class="t-small">req 1</text>
<rect x="134" y="56" width="260" height="30" rx="6" class="f-warn-soft s-warn"/><text x="264" y="76" text-anchor="middle" class="t-small">req 2 — chậm</text>
<rect x="398" y="56" width="90" height="30" rx="6" class="f-phys-soft s-phys"/><text x="443" y="76" text-anchor="middle" class="t-small">req 3</text>
<rect x="492" y="56" width="90" height="30" rx="6" class="f-phys-soft s-phys"/><text x="537" y="76" text-anchor="middle" class="t-small">req 4</text>
<text x="600" y="76" class="t-small t-warn">server chậm ⇒ generator gửi ít đi, lịch tự co lại</text>
<text x="16" y="146" class="t-b">Open schedule (SYS-08): slot cách đều 1000/r ms, không chờ response</text>
<line x1="16" y1="250" x2="900" y2="250" class="s-pk" stroke-width="1.2" marker-end="url(#sys08-arrow)"/>
<line x1="40" y1="242" x2="40" y2="258" class="s-pk" stroke-width="1.6"/><text x="40" y="274" text-anchor="middle" class="t-small">slot 0</text>
<line x1="160" y1="242" x2="160" y2="258" class="s-pk" stroke-width="1.6"/><text x="160" y="274" text-anchor="middle" class="t-small">slot 1</text>
<line x1="280" y1="242" x2="280" y2="258" class="s-pk" stroke-width="1.6"/><text x="280" y="274" text-anchor="middle" class="t-small">slot 2</text>
<line x1="400" y1="242" x2="400" y2="258" class="s-pk" stroke-width="1.6"/><text x="400" y="274" text-anchor="middle" class="t-small">slot 3</text>
<line x1="520" y1="242" x2="520" y2="258" class="s-pk" stroke-width="1.6"/><text x="520" y="274" text-anchor="middle" class="t-small">slot 4</text>
<line x1="640" y1="242" x2="640" y2="258" class="s-pk" stroke-width="1.6"/><text x="640" y="274" text-anchor="middle" class="t-small">slot 5</text>
<rect x="40" y="206" width="90" height="26" rx="6" class="f-pk-soft s-pk"/><text x="85" y="224" text-anchor="middle" class="t-small">req 0</text>
<rect x="160" y="166" width="380" height="26" rx="6" class="f-warn-soft s-warn"/><text x="350" y="184" text-anchor="middle" class="t-small">req 1 — chậm, vẫn đang chạy</text>
<rect x="292" y="206" width="90" height="26" rx="6" class="f-pk-soft s-pk"/><text x="337" y="224" text-anchor="middle" class="t-small">req 2</text>
<line x1="280" y1="236" x2="292" y2="236" class="s-warn" stroke-width="2"/><text x="286" y="203" text-anchor="middle" class="t-small t-warn">lag</text>
<rect x="400" y="206" width="90" height="26" rx="6" class="f-pk-soft s-pk"/><text x="445" y="224" text-anchor="middle" class="t-small">req 3</text>
<rect x="520" y="206" width="104" height="26" rx="6" class="f-bg s-warn dash"/><text x="572" y="224" text-anchor="middle" class="t-small t-warn">dropped</text>
<rect x="640" y="206" width="90" height="26" rx="6" class="f-pk-soft s-pk"/><text x="685" y="224" text-anchor="middle" class="t-small">req 5</text>
<text x="745" y="190" class="t-small">request chồng nhau</text>
<text x="745" y="206" class="t-small">⇒ concurrency tăng;</text>
<text x="745" y="222" class="t-small t-warn">không gửi được ⇒ ghi</text>
<text x="745" y="238" class="t-small t-warn">drop kèm lý do</text>
</svg>
</div>
<figcaption><b>Hình 1 · Closed loop và open schedule.</b> Ở hàng trên, request chậm kéo lùi cả lịch nên «0 lỗi» có thể là 0 lỗi của một tải thấp hơn dự định. Ở hàng dưới, lịch giữ nguyên; độ trễ của generator hiện ra thành lag, còn slot không gửi được hiện ra thành drop — cả hai đều phải vào báo cáo.</figcaption>
</figure>

### 4.2. Ba lý do drop và các rào bảo vệ

Lab đặt các giới hạn sau, tất cả cố định **trước** khi chạy:

| Rào | Giá trị | Khi chạm |
|---|---|---|
| In-flight cap | 20 request đang chạy | Slot ghi `in-flight-cap`, không gửi |
| Lag tối đa | 100 ms | Slot ghi `scheduler-late`, **không bắn bù** thành burst |
| Dừng khi lỗi | Đã có 10 failure | Slot ghi `failure-stop` cho phần còn lại |
| Timeout/request | 2 giây (deadline phía client) | Request tính là failure, lỗi `TimeoutError` |

Thứ tự xét trong code là `failure-stop` → `scheduler-late` → `in-flight-cap`. Lịch vẫn đi tới hết profile kể cả khi đã dừng gửi, để bất biến `planned = sent + dropped` luôn đúng — không giả vờ các slot đó đã hoàn tất, cũng không lặng lẽ biến mất khỏi mẫu số.

Không bắn bù slot trễ là một quyết định quan trọng: nếu generator khựng 300 ms rồi gửi dồn 9 request cùng lúc, nó tạo ra một spike nhân tạo mà workload model không hề mô tả.

### 4.3. Scheduler này không phải k6

Đây là scheduler nhỏ viết cho bài học, **không phải** executor của k6. k6 cũng có khái niệm *dropped iterations* và hướng dẫn cấp đủ VU cho executor arrival-rate, nhưng ngữ nghĩa counter phải đọc theo từng executor; đừng đồng nhất tên counter của lab với metric của mọi công cụ. Để đo quy mô lớn, dùng generator đã được kiểm chứng, chạy trên máy riêng.

### 4.4. Bước tải và drain

Cuối mỗi bước, script chờ hết cửa sổ theo lịch, rồi **drain** — chờ mọi request còn chạy kết thúc — trước khi sang bước sau. Vì có drain, ba bước không phải một ramp liên tục theo thời gian tuyệt đối. Báo cáo ghi riêng thời lượng theo lịch và thời gian gồm drain để không che phần đuôi. Bước `return-low` chỉ quan sát tải thấp vài giây sau bước mục tiêu; nó chưa đủ để kết luận hồi phục sau sự cố — đó là việc của [SYS-14](/articles/system-testing-recovery-test).

## 5. Metrics và gate đặt trước phép đo

### 5.1. Mỗi metric có mẫu số riêng

| Metric | Mẫu số / ranh giới |
|---|---|
| Planned | `rps × seconds` của bước |
| Sent | Số slot thực sự bắt đầu request |
| Dropped | Slot không gửi, có lý do; `planned = sent + dropped` |
| Error rate (sent) | Request sai HTTP/oracle/timeout chia **sent** |
| Drop rate (planned) | `dropped / planned`; tách khỏi lỗi HTTP |
| Sent per scheduled second | `sent / seconds`; **không** gọi đây là completion RPS |
| Completed in window | Request có `endOffsetMs` trước cuối cửa sổ lịch |
| Valid RPS including drain | Số response đúng / thời gian gồm drain |
| p50/p95/p99 | Nearest-rank trên mọi request đã gửi, **gồm** failed attempts |
| Scheduler lag | Lag tối đa trên cả slot đã gửi lẫn slot bị drop |

Slot bị drop không có latency, nên **không** được nhét số 0 vào percentile — làm thế sẽ kéo p95 xuống đúng lúc hệ thống tệ nhất. Khi một bước không gửi được request nào, latency và error rate là `null`, không phải 0, và gate phải fail.

Một chi tiết nearest-rank: p99 của bước chỉ có 30 mẫu là mẫu thứ ⌈0,99 × 30⌉ = 30, tức **chính là max**. Với 240 mẫu, p99 là mẫu thứ 238. Con số p99 trên 30 mẫu không phải ước lượng tail ổn định.

### 5.2. Gate

Gate **giả định cho lab local**, khai báo trong code trước khi chạy: mỗi bước **zero drops, zero errors và p95 &lt; 500 ms**. Ba điều kiện đi cùng nhau vì mỗi điều kiện bịt một lỗ hổng của hai điều kiện kia:

- p95 thấp nhưng drop nhiều ⇒ hệ thống chưa được kiểm ở tải yêu cầu.
- 0 drop, p95 thấp nhưng sai tiền ⇒ nhanh nhưng sai.
- 0 lỗi, 0 drop nhưng p95 vượt ⇒ đúng nhưng chậm.

Không đổi ngưỡng sau khi xem số đo. Nếu ngưỡng sai, sửa nó trong một thay đổi riêng có lý do, rồi chạy lại.

### 5.3. Entry và exit

**Entry**: functional smoke của endpoint đúng ([SYS-01](/articles/system-testing-smoke-test)); dataset, endpoint và mức tải mục tiêu đã ghi; môi trường thuộc quyền kiểm thử; không có benchmark khác chạy cùng máy nếu muốn giảm nhiễu.

**Exit của bài tập**: kế toán khớp, chạy đủ profile, gate đạt, raw evidence được lưu, giới hạn được công bố. Nếu fail do generator, kết luận là *thí nghiệm chưa đạt tải yêu cầu* — chưa được nói server không chịu nổi.

## 6. Thực hành có thể chạy lại

### 6.1. Chạy

Từ `examples/toi-noi-testing`, Node ≥ 22.12:

```sh
npm ci
npx tsx articles/system-testing-load-test/load.ts
npm run typecheck
```

Script tự tạo PGlite và HTTP server trên cổng loopback ngẫu nhiên, không nhận URL bên ngoài và không dùng DB của người dùng. Mỗi lượt mới ghi `reports/load/sys08-load.json`; evidence đã commit nằm riêng ở `load-observed.json` và không bị ghi đè.

### 6.2. Dựng hệ thống và request có oracle

```ts
const system = await createPgliteSystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
const server = createApp(system.deps, { logErrors: false }).listen(0, '127.0.0.1');
```

Đồng hồ cố định 09:00 để giá không trượt sang giờ cao điểm giữa chừng. Mỗi request mang oracle của chính nó:

```ts
const res = await fetch(url, {
  method: 'POST', headers: { 'content-type': 'application/json', 'x-user': 'customer:C001' },
  body: JSON.stringify({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 }),
  signal: AbortSignal.timeout(2_000),
});
status = res.status;
const body = await res.json() as { quote?: { total?: number; quoteId?: string } };
valid = status === 200 && body.quote?.total === 35_000 && typeof body.quote.quoteId === 'string';
```

Latency đo từ trước `fetch` tới sau khi parse JSON và kiểm oracle — tức gồm client, loopback, server, PGlite và xử lý response. Đây là ranh giới đo của lab, không giống hệt `http_req_duration` của k6.

### 6.3. Vòng slot — phần cốt lõi

```ts
for (let slot = 0; slot < planned; slot++) {
  const due = start + slot * 1_000 / stage.rps;
  while (performance.now() < due) await sleep(Math.max(1, due - performance.now()));
  const lagMs = performance.now() - due;
  const reason = failureCount >= 10 ? 'failure-stop' : lagMs > maxLagMs ? 'scheduler-late' : active.size >= maxInFlight ? 'in-flight-cap' : null;
  if (reason) { dropped.push({ slot, lagMs, reason }); continue; }
  const startOffsetMs = performance.now() - start;
  const task = send().then(result => { samples.push({ slot, lagMs, startOffsetMs, endOffsetMs: performance.now() - start, ...result }); });
  active.add(task);
  peakInFlight = Math.max(peakInFlight, active.size);
  void task.then(() => active.delete(task));
}
```

Đọc kỹ ba dòng:

- `await sleep(...)` chỉ chờ **tới due time**, không chờ HTTP response.
- `send().then(...)` ghi mẫu khi request xong; vòng lặp không đứng đợi.
- Không có `await send()` trong vòng slot. Viết `await send()` ở đây là quay lại closed loop mà không ai nhận ra — lỗi thiết kế phổ biến nhất khi tự viết generator.

Sau vòng slot, chờ hết cửa sổ rồi drain:

```ts
const end = start + stage.seconds * 1_000;
while (performance.now() < end) await sleep(Math.max(1, end - performance.now()));
await Promise.all(active); // drain before the next step; report includes drain time
const elapsedMsIncludingDrain = performance.now() - start;
```

### 6.4. Percentile và gate trong code

```ts
const sorted = samples.map(s => s.latencyMs).sort((a, b) => a - b);
const p = (q: number) => sorted.length ? sorted[Math.ceil(q * sorted.length) - 1]! : null;
```

```ts
illustrativeGatePassed: dropped.length === 0 && errors === 0 && p95 !== null && p95 < 500,
```

`p95 !== null` là điều kiện nhỏ nhưng thiết yếu: một bước không gửi được gì phải fail, không được lọt qua vì «không có latency xấu».

### 6.5. Expected và observed

**Expected** (đặt trước khi chạy): planned = 300, sent = 300, dropped = 0, errors = 0, cả ba bước đạt gate, exit 0. Số latency không được hardcode vì phụ thuộc máy.

**Observed** — lượt ghi trong `load-observed.json`, đo lúc 14:09:09 UTC ngày 13/11/2026 (21:09:09 giờ Việt Nam) trên Windows 10.0.26300, Node v24.15.0, Intel Core i7-10700K, 16 logical CPU, RAM 34.245.918.720 byte; exit 0; typecheck lab đạt:

| Bước | Lịch | Planned / sent / drop / lỗi | p50 ms | p95 ms | p99 ms | Valid RPS gồm drain | Lag tối đa ms | Peak in-flight |
|---|---|---|---:|---:|---:|---:|---:|---:|
| low | 10/s × 3 s | 30 / 30 / 0 / 0 | 3,04 | 11,23 | 13,23 | 9,99 | 15,52 | 1 |
| target | 30/s × 8 s | 240 / 240 / 0 / 0 | 2,96 | 6,31 | 10,37 | 29,95 | 16,48 | 1 |
| return-low | 10/s × 3 s | 30 / 30 / 0 / 0 | 2,63 | 10,09 | 10,21 | 9,97 | 15,24 | 1 |

Mọi request hoàn tất trong cửa sổ của bước (`completedInWindow` = sent ở cả ba bước). Thời gian gồm drain là 3.002,5 ms, 8.012,7 ms và 3.009,0 ms — gần như trùng cửa sổ lịch vì không có request nào treo tới cuối. Báo cáo của lượt đó còn kiểm JSON độc lập: `planned = sent + dropped`, số mẫu, số drop slot, failure count và p95 khớp summary.

Bài viết **không** chạy lại script khi biên tập: máy đang chạy các tác vụ khác, số đo mới sẽ bị nhiễu và không so được với lượt trên. Mọi số liệu ở đây là của lượt 13/11/2026 kèm cấu hình máy đã ghi.

### 6.6. Đọc kỹ kết quả «đẹp»

Ba quan sát làm câu kết luận nhỏ lại:

1. **Peak in-flight = 1 ở cả ba bước.** Latency khoảng 3 ms, còn khoảng cách giữa hai slot ở 30/s là 33 ms; request luôn xong trước khi slot sau tới. Open model đúng về thiết kế, nhưng lượt này **chưa tạo ra overlap** — nên chưa có gì để phân biệt nó với closed loop. Ước lượng theo quan hệ concurrency ≈ arrival rate × thời gian một iteration: 30/s × 0,003 s ≈ 0,09, khớp với peak 1.
2. **Các nhánh drop chưa được kích hoạt.** Không slot nào chạm cap 20, lag 100 ms hay 10 failure. Vậy lượt này chưa kiểm nghiệm hành vi khi quá tải; đó là thí nghiệm riêng của SYS-09.
3. **Lag tối đa khoảng 15–16 ms ở mọi bước, kể cả bước 10/s.** Lag không tăng theo tải, gợi ý nguồn gốc nằm ở độ phân giải timer của hệ điều hành hơn là ở server. Đây là *giả thuyết* chưa được kiểm; nó dưới ngưỡng 100 ms nên không gây drop, nhưng là lý do phải đo lag thay vì giả định timer chính xác.

Kết luận được phép: *đạt tải giả định 30 arrival/s trong lab local với oracle nghiệp vụ đúng trên 300/300 response*. Chưa xác định capacity hay bottleneck.

## 7. Đọc báo cáo và phân loại lỗi

Thứ tự đọc: **planned/sent/drop → lỗi HTTP và nghiệp vụ → latency từng bước → lag và in-flight → thời gian drain.** Không bắt đầu bằng con số p95 đẹp nhất.

| Dấu hiệu | Khả năng cần kiểm | Bước tiếp theo |
|---|---|---|
| Drop ngay đầu bước | Cap quá nhỏ hoặc generator bận | Xem `active` count, lag, tài nguyên máy sinh tải |
| Drop tăng cùng latency | Hệ thống chậm giữ slot lâu | Xem hàng đợi DB, CPU; tách client/server |
| 200 nhưng sai `total` | Sai nghiệp vụ dưới tải | Dừng tối ưu, tái hiện bằng functional case ([SYS-03](/articles/system-testing-functional-test)) |
| Nhiều 429 | Đụng rate limit hoặc workload không hợp | Đối chiếu R-API-13 và account mix; không tự tắt limiter |
| Lag lớn, latency nhỏ | Generator là nút cổ chai | Chuyển generator sang process/máy khác trước khi kết luận |
| Sau giảm tải vẫn chậm | Backlog, GC, tài nguyên giữ lại | Theo dõi lâu hơn, thu metrics để xác định nguyên nhân |

Phân loại kết luận theo bằng chứng:

- **Thí nghiệm không hợp lệ** — drop do generator hoặc lag lớn: chưa đạt tải yêu cầu, chạy lại sau khi sửa generator.
- **Failure hiệu năng** — gửi đủ, đúng nghiệp vụ, nhưng p95 vượt gate.
- **Failure chức năng dưới tải** — response sai tiền dù tải thấp hay cao: điều tra độc lập với latency, mức ưu tiên cao hơn tối ưu tốc độ.

## 8. Mở rộng nâng cao

### 8.1. Ước lượng worker và nhận biết saturation

Trong điều kiện ổn định, concurrency trung bình ≈ arrival rate × thời gian trung bình một iteration. Ví dụ giả định: 30/s và 0,2 s ⇒ khoảng 6 tác vụ đang chạy. Nhưng mean không phải cam kết: khi p99 cao, số slot bận vượt xa ước lượng và tạo drop. Cần headroom cho tail. Tăng cap chỉ có ý nghĩa khi generator và hệ thống còn tài nguyên — nếu không, ta chỉ chuyển drop thành timeout.

Dấu hiệu saturation khi tăng dần tải theo bước: throughput hoàn tất ngừng tăng theo arrival, latency và in-flight tăng nhanh, rồi drop xuất hiện. Điểm uốn đó là thông tin cho capacity planning — nhưng tìm nó là mục tiêu của stress/breakpoint, không phải load test.

### 8.2. Workload mix và dữ liệu thực tế

Mở rộng sang ride/parcel/food, quãng đường và thành phố theo phân bố có căn cứ. Với tạo chuyến, cần đủ khách và tài xế, idempotency key riêng cho mỗi iteration và chính sách cleanup. Đừng tạo hàng nghìn request với cùng `Idempotency-Key` rồi tuyên bố đã kiểm throughput tạo chuyến: theo R-IDEM, phần lớn chỉ là replay kết quả đã lưu. Tách giao dịch mới và replay thành hai metric.

### 8.3. Tải đọc/ghi và DB lớn

300 quote mới trên DB nhỏ chưa mô phỏng index, buffer, cache của dữ liệu nhiều tháng. Dựng dataset đại diện, xem query plan, connection pool, lock/wait/IO. Không chỉ tăng VU mà bỏ qua kích thước dữ liệu. PGlite một kết nối không thay được thí nghiệm nhiều transaction trên PostgreSQL server thật ([SYS-13](/articles/system-testing-data-integrity-test) gặp đúng giới hạn này).

### 8.4. Rate theo endpoint và tail latency

Một iteration nghiệp vụ «khách đặt xe» có thể gồm báo giá, đặt chuyến và nhiều lần hỏi trạng thái. 30 iteration/s lúc đó không còn là 30 request/s. Ghi metric theo operation; percentile gộp có thể che một endpoint ít traffic nhưng chậm. Khi sinh tải theo lịch, đo cả lag tại generator lẫn latency sau khi request thực gửi để thấy phần thời gian chờ trước khi gửi — nếu bỏ phần này, ta mắc *coordinated omission* đã nói ở SYS-07.

### 8.5. So sánh release và độ tin cậy của kết luận

So baseline với candidate trên cùng máy, dữ liệu, profile, chính sách cache và đủ thời gian. Lặp nhiều lượt để thấy biến thiên; không kết luận từ vài ms chênh lệch ở một lượt 14 giây. Threshold theo SLO và mục tiêu thí nghiệm, đặt trước khi chạy; luôn giữ oracle nghiệp vụ khi tối ưu. Trong pipeline, load test ngắn kiểu lab hợp với gate «không thoái lui rõ rệt», còn load test dài trên môi trường riêng hợp với quyết định release.

### 8.6. Tách generator khỏi hệ thống

Trong lab, client, server và PGlite cùng một process, cùng CPU và event loop. Request của client và xử lý của server tranh cùng tài nguyên; lag của generator và latency của server không tách được hoàn toàn. Bước tiếp theo hợp lý: server một process, generator process/máy khác, đồng bộ cấu hình, đo CPU/mạng ở cả hai phía. Tách process không tự động xoá nhiễu — chỉ làm nhiễu đo được.

## 9. Anti-pattern, giới hạn và đánh đổi

Những cách làm tạo kết luận thiếu căn cứ:

- Ghi «100 users» mà không mô tả hành vi, nhịp đến, dữ liệu.
- Chỉnh `sleep` cho tới khi ra RPS mong muốn rồi bỏ đo scheduler lag.
- Báo p95 chỉ trên response thành công, hoặc nhét slot drop vào percentile bằng số 0.
- `await send()` trong vòng slot — vô tình quay về closed loop.
- Mọi request dùng chung một idempotency key.
- Chạy chưa đủ tải nhưng vẫn ghi PASS.
- Lấy số loopback một process để dự báo cluster production.
- Không kiểm nội dung response, coi mọi 200 là thành công.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn bằng chứng của SYS-08</span>
<p>Profile chỉ 14 giây theo lịch, có drain giữa các bước — không phải steady-state dài hay soak. Client/server/PGlite cùng process, một DB connection, dữ liệu nhỏ, một khách. Không thu CPU/DB telemetry tách biệt. Không có browser, mạng ngoài, provider thanh toán, generator riêng hay k6 thực thi. Các nhánh drop/stop không được kích hoạt ở lượt đo. Bài không chứng minh giới hạn hệ thống; bước giảm về tải thấp không thay Recovery Test. Không so chênh lệch với SYS-07 như phép đo tác động của DB, vì cả workload lẫn storage đều đã đổi.</p>
</div>

Đánh đổi chi phí/phạm vi: profile ngắn trên một máy rẻ, chạy được trong pipeline, phát hiện sớm thoái lui lớn và lỗi «nhanh nhưng sai». Đổi lại, nó không trả lời câu hỏi capacity. Load test có ý nghĩa cho release cần môi trường gần production, dữ liệu đại diện, generator riêng và thời gian chạy dài — tốn hơn nhiều, nên chỉ chạy ở mốc quyết định.

## 10. Bài tập và câu hỏi tự kiểm

### Bài tập

**Bài 1 (cơ bản).** Một lượt ghi planned = 300, sent = 270, errors = 0, p95 = 8 ms. Có đạt gate không?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Không. Drop rate = 30/300 = 10%: hệ thống chưa được kiểm ở tải mục tiêu. 0 lỗi trên 270 request đã gửi không bù được 30 slot bị bỏ. Bước tiếp theo là đọc lý do drop: <code>in-flight-cap</code> hay <code>scheduler-late</code> chỉ về hai hướng điều tra khác nhau.</p>
</details>

**Bài 2 (trung bình).** Arrival 50/s, một iteration trung bình 0,4 s. Concurrency trung bình khoảng bao nhiêu, và cap 20 có đủ không?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Khoảng 50 × 0,4 = 20. Cap 20 bằng đúng mức trung bình, không có headroom: chỉ cần tail dài hơn mean là slot bị drop vì <code>in-flight-cap</code>. Cần cap lớn hơn, có căn cứ từ phân bố latency (không chỉ mean), và generator đủ tài nguyên.</p>
</details>

**Bài 3 (trung bình).** Đổi iteration thành «báo giá rồi đặt chuyến», giữ 30 iteration/s. Request rate mục tiêu là bao nhiêu và vấn đề gì xuất hiện ngay?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Ít nhất 60 request/s nếu mỗi iteration gửi đúng hai call; retry/polling làm tăng thêm. Vấn đề ngay lập tức: R-API-13 giới hạn 5 lần <code>POST /api/trips</code>/60 giây/khách, nên một khách C001 sẽ nhận 429 gần như toàn bộ. Cần account mix đủ lớn, idempotency key riêng mỗi iteration, và metric tách quote/book.</p>
</details>

**Bài 4 (nâng cao).** Một đồng nghiệp sửa vòng slot thành <code>await send()</code> «cho dễ đọc». Lượt chạy vẫn xanh. Giải thích vì sao lượt đó xanh và vì sao thay đổi vẫn phải bị từ chối.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Với latency ~3 ms và khoảng slot 33 ms, request luôn xong trước slot sau, nên closed loop và open model cho kết quả gần như giống nhau ở lab này (peak in-flight = 1). Nhưng khi server chậm hơn khoảng slot, <code>await send()</code> sẽ đẩy lùi lịch: slot đến muộn, lag vượt 100 ms và bị drop là <code>scheduler-late</code> — hoặc tệ hơn, nếu bỏ cả rào lag, tải thực tế giảm âm thầm. Thay đổi phá mô hình workload; test xanh chỉ vì điều kiện hiện tại không phơi lỗi.</p>
</details>

**Bài 5 (nâng cao).** Thiết kế lại thí nghiệm để generator và server ở hai process (hoặc hai máy).

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Cần: server khởi động riêng với cấu hình và dữ liệu được ghi; generator nhận base URL của môi trường kiểm thử; đồng bộ đồng hồ/ghi timezone; đo CPU, bộ nhớ, mạng ở cả hai phía; giữ lịch arrival, cap in-flight, timeout, raw samples và oracle 35.000; ghi rõ phiên bản build. Kết luận vẫn phải nói những gì chưa tách được (mạng thật, DB nhiều kết nối).</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** Drop do <code>in-flight-cap</code> có luôn nghĩa là server quá tải không?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. Cap có thể đặt quá nhỏ so với arrival × latency, hoặc generator bận làm request kết thúc muộn. Cần xem lag, latency và tài nguyên generator trước khi quy cho server.</p>
</details>

**Câu 2.** p95 thấp sau khi bỏ nhiều slot có chứng minh tải mục tiêu đạt không?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. Percentile chỉ tính trên request đã gửi; slot bị bỏ không có latency. Gate phải gồm cả drop rate.</p>
</details>

**Câu 3.** Vì sao p99 của bước <code>low</code> (30 mẫu) không nên dùng làm ước lượng tail?

<details>
<summary><b>Đáp án</b></summary>
<p>Nearest-rank cho ⌈0,99 × 30⌉ = 30, tức p99 chính là giá trị lớn nhất. Một mẫu đơn lẻ quyết định con số; cần nhiều mẫu hơn hoặc nhiều lượt để nói về tail.</p>
</details>

**Câu 4.** Lượt đo có peak in-flight = 1. Điều đó nói gì về những gì bài lab đã và chưa kiểm?

<details>
<summary><b>Đáp án</b></summary>
<p>Đã kiểm: hệ thống xử lý đúng 30 arrival/s khi request không chồng nhau. Chưa kiểm: hành vi khi có overlap thật, tranh chấp DB, cap và các nhánh drop. Open model đúng về thiết kế nhưng điều kiện chưa phơi ra khác biệt với closed loop.</p>
</details>

**Câu 5.** Vì sao script không bắn bù slot bị trễ quá 100 ms?

<details>
<summary><b>Đáp án</b></summary>
<p>Bắn bù dồn nhiều request vào một thời điểm, tạo spike nhân tạo không có trong workload model. Ghi drop kèm lý do giữ thí nghiệm trung thực: báo cáo cho thấy generator đã không theo kịp lịch.</p>
</details>

## 11. Tài liệu tham khảo

Các nguồn ngoài được tra cứu 13/11/2026.

- Grafana k6 — Load test types: https://grafana.com/docs/k6/latest/testing-guides/test-types/ (phân biệt smoke, average-load, stress, soak, spike, breakpoint theo mục tiêu).
- Grafana k6 — Arrival-rate VU allocation: https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/arrival-rate-vu-allocation/ (cần đủ VU cho lịch arrival; quan hệ với thời gian iteration).
- Grafana k6 — Dropped iterations: https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/dropped-iterations/ (nguyên nhân cấu hình và hệ thống; đưa drop vào tiêu chí đánh giá). Script trong bài **không** phải k6 và bài không tuyên bố đã chạy k6.
- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf: thuật ngữ test type phi chức năng. Cách dựng scheduler và gate là thực hành của series.
- Lab Tới Nơi: `SPEC.md` (R-API-07, R-API-13, R-FARE-01…08, R-QUOTE-01, R-IDEM, R-NFR-03, R-NFR-07), script `load.ts`, evidence `load-observed.json` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-load-test
- Bài liên quan: [SYS-07 Performance Test](/articles/system-testing-performance-test) · [SYS-09 Stress Test](/articles/system-testing-stress-test) · [SYS-03 Functional Test](/articles/system-testing-functional-test) · [Non-functional testing tổng quan](/articles/non-functional-testing-tong-quan)

Bài tiếp theo: [SYS-09 · Stress Test](/articles/system-testing-stress-test).
