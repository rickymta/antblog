# SYS-08 — Load Test: tải dự kiến có thực sự được tạo và xử lý đúng?

Trạng thái: bản nháp hoàn chỉnh để review, có lab và evidence. Bài trước: [Performance Test](07-performance-test.md). [Lộ trình](../system-testing-roadmap.md).

## 1. Bài toán và mục tiêu học tập

Ở bài trước, một worker chỉ gửi request mới khi request trước hoàn tất. Khi hệ thống chậm, tốc độ gửi cũng giảm. Nhưng trong giờ cao điểm, người dùng mới vẫn tiếp tục mở ứng dụng và yêu cầu báo giá. Muốn đánh giá hành vi dưới một mức tải dự kiến, cần biết mình đã định gửi bao nhiêu, thực gửi được bao nhiêu và hệ thống xử lý được bao nhiêu.

Bài này dùng lịch arrival độc lập với completion, qua endpoint báo giá có ghi dữ liệu PGlite. Sau bài, bạn có thể xây workload model, phân biệt RPS/VU/concurrency, đo drops và scheduler lag, đọc kết quả theo từng bước tải và nhận ra khi generator làm sai thí nghiệm. Cần kiến thức HTTP và metrics của SYS-07.

## 2. Khái niệm và phân biệt

Load testing đánh giá hệ thống ở tải được xác định là bình thường hoặc dự kiến, với các tiêu chí latency, lỗi, throughput và tài nguyên. Tải không chỉ là số user: gồm loại request, nhịp arrival, hành trình, dữ liệu, thời gian chạy và trạng thái cache/DB.

| Đại lượng | Ý nghĩa | Điều dễ nhầm |
|---|---|---|
| Arrival rate | Số iteration được lên lịch bắt đầu mỗi giây | Không mặc định bằng số request nếu iteration có nhiều HTTP calls |
| Request rate | Số HTTP requests theo thời gian | Không đồng nghĩa số người dùng |
| Concurrency | Số tác vụ đang thực hiện đồng thời | Không bằng số tài khoản trong dataset |
| VU | Worker/ngữ cảnh mô phỏng trong công cụ | Cần đủ VU để đạt arrival rate khi response chậm |
| Throughput | Số hoàn tất trong cửa sổ đo | Có thể gồm cả lỗi trả về rất nhanh |

Stress test cố ý đi vượt tải thường để quan sát suy giảm/giới hạn; load test cần tải mục tiêu có căn cứ. Lab này chọn 30 arrivals/s như **giả định học tập**, không lấy từ telemetry sản phẩm và không xác nhận toàn hệ thống Tới Nơi đạt một công suất nào. Không dùng con số tham chiếu của kiến trúc mục tiêu như thể đó là yêu cầu cho một tiến trình lab.

## 3. Xây workload từ nghiệp vụ

Trong dự án thật, bắt đầu từ số phiên hoạt động, tần suất báo giá/đặt xe, tỷ lệ đọc/ghi, phân bố thành phố/dịch vụ và biến thiên theo giờ. Một khách có thể báo giá nhiều lần nhưng không đặt; một chuyến có nhiều polling requests. Từ đó suy arrival cho từng endpoint, không chia đều tải tùy ý.

| Thành phần cần xác định | Thiết kế lab hiện tại | Giới hạn |
|---|---|---|
| Operation | Một iteration = một POST /api/quotes | Không mô phỏng booking/payment/polling |
| Dataset | C001, ride SG bike 5 km | Không có phân bố người dùng hoặc hot keys đa dạng |
| Oracle | 200, total 35.000, quoteId chuỗi | Chưa kiểm mọi trường response |
| Storage | PGlite, mỗi quote tạo bản ghi | Một kết nối, không server PostgreSQL nhiều connection |
| Clock/weather | 09:00 VN cố định, không mưa | Không có biến thiên nghiệp vụ theo giờ |
| Profile | 10/s trong 3s → 30/s trong 8s → 10/s trong 3s | Profile ngắn theo bước, không phải soak |

Dataset tiếp tục tăng từ 0 tới 300 quote trong phần đo; không reset giữa các bước để phản ánh ghi tích lũy trong lượt này. Warm-up 20 request rồi reset trước đo. Không dùng voucher để tránh giới hạn lượt làm thay đổi mix lỗi; không đặt chuyến nên rate limit POST trips không áp dụng. Đây không phải tắt rate limit để làm kết quả đẹp.

## 4. Scheduler: gửi theo lịch, không đợi response

Với rate r, slot i có thời điểm dự kiến `t0 + i×1000/r`. Tới slot, script gửi mà không await completion; chỉ giữ promise trong tập active. Như vậy response chậm không tự đổi lịch như closed loop. Tuy nhiên Node event loop có thể bận, nên phải đo `lag = actual start − due time`.

Lab đặt cap 20 request đang chạy. Khi không còn chỗ, slot được ghi dropped với lý do in-flight-cap; slot trễ quá 100 ms bị ghi scheduler-late và không bắn bù thành burst. Nếu đã có 10 failures, không gửi thêm và ghi failure-stop cho các slot sau. Lịch vẫn đi tới hết profile để bảo toàn accounting, không giả vờ các slot đó đã hoàn tất.

Đây là scheduler nhỏ tự viết cho bài học, **không phải k6 executor**. Công cụ k6 cũng có khái niệm dropped iterations nhưng semantics phải đọc theo executor; không đồng nhất tên counter của lab với mọi metric k6. Để đo quy mô lớn, dùng generator được kiểm chứng và máy riêng.

Cuối mỗi bước, chờ hết cửa sổ theo lịch rồi drain request đang chạy trước bước tiếp. Vì có drain, ba bước không phải một ramp liên tục theo thời gian tuyệt đối. Ghi riêng scheduled duration và elapsed including drain để không che phần thời gian đuôi. Sau profile giảm về 10/s, chỉ quan sát tải thấp trong vài giây, chưa đủ kết luận recovery sau sự cố.

## 5. Metrics và gate đặt trước phép đo

| Metric | Mẫu số/ranh giới |
|---|---|
| Planned | rate × scheduled seconds của bước |
| Sent | Số slot thực sự bắt đầu request |
| Dropped | Slot không gửi, có lý do; planned=sent+dropped |
| Error rate sent | Request sai HTTP/oracle/timeout chia sent |
| Drop rate planned | dropped/planned; tách khỏi HTTP error |
| Sent per scheduled second | sent/scheduled seconds; không gọi đây là completion RPS |
| Completed in window | Request có endOffset trước cuối cửa sổ lịch |
| Valid RPS including drain | Số response đúng / elapsed gồm drain |
| Latency p95/p99 | Nearest-rank trên các request đã gửi, gồm failed attempts |
| Scheduler lag | Độ trễ lúc đánh giá slot so với due time |

Dropped slot không có response latency, nên không nhét số 0 vào percentile. Nếu latency thấp nhưng nhiều drops, hệ thống chưa được kiểm ở tải yêu cầu; gate phải xem cả hai. Khi không gửi được request nào, latency/error rate là null, không phải 0 và không được pass.

Gate **giả định local**: từng bước zero drops, zero errors và p95<500 ms. Giới hạn bảo vệ: tối đa 300 measured requests, 20 in-flight, timeout 2 giây/request, ngừng gửi mới sau 10 failures. Không đổi ngưỡng sau khi xem số đo để đạt pass. Timeout là deadline phía client; event-loop nghẽn có thể làm thời gian quan sát vượt cấu hình.

## 6. Thực hành có thể chạy lại

Mã: `examples/toi-noi-testing/articles/system-testing-load-test/load.ts`. Tại thư mục lab, Node >=22.12:

```sh
npm ci
npx tsx articles/system-testing-load-test/load.ts
npm run typecheck
```

Script tự tạo PGlite và HTTP server loopback cổng ngẫu nhiên, không nhận URL bên ngoài. Không mở server khác trước; không dùng chung DB. Warm-up phải đúng oracle mới chạy lịch. Cuối cùng đóng server và DB. Không chạy đồng thời benchmark khác nếu muốn giảm nhiễu.

Expected: planned=300, sent=300, dropped=0, errors=0, ba gate đạt và exit 0. Kết quả thực tế có thể fail theo tài nguyên máy; không hardcode số latency. JSON tại `reports/load/sys08-load.json` chứa environment, thiết kế, summary, start/end offsets, lag, status và error của từng sample cùng danh sách dropped slots.

Source cốt lõi cần đọc là vòng slot: `await sleep` chỉ chờ tới due time, không chờ HTTP response. `send().then(...)` thêm mẫu khi request hoàn tất; `Promise.all(active)` dùng lúc drain. Không viết `await send()` ngay trong vòng slot nếu muốn giữ mô hình arrival độc lập.

Client, server và PGlite vẫn cùng process, CPU và event loop. Đây là hạn chế chủ ý để lab gọn, không đủ cô lập bottleneck. Chuyển từ bài memory sang PGlite cũng đổi storage và lịch tải; không lấy chênh lệch số đo giữa hai bài để quy riêng tác động của DB.

## 7. Đọc báo cáo và phân loại lỗi

Kết quả lượt thực chạy nằm trong [báo cáo SYS-08](08-test-report.md), raw JSON được giữ cùng lab. Đọc theo thứ tự: planned/sent/drops → lỗi HTTP/nghiệp vụ → latency từng bước → lag/in-flight → thời gian drain. Không bắt đầu bằng con số p95 đẹp nhất.

| Dấu hiệu | Khả năng cần kiểm | Bước tiếp theo |
|---|---|---|
| Drops ngay đầu bước | Worker/cap quá nhỏ hoặc generator bận | Kiểm active count, lag và môi trường generator |
| Drops tăng cùng latency | Hệ thống chậm giữ slot lâu | Xem DB queue và CPU, tách client/server |
| 200 nhưng sai total | Sai nghiệp vụ dưới tải | Dừng tối ưu, tái hiện bằng functional case |
| 429 nhiều | Rate limit hoặc workload không phù hợp | Đối chiếu hợp đồng và account mix; không tự tắt limiter |
| Sau giảm tải vẫn chậm | Backlog/GC/resource retention | Theo dõi lâu hơn, thu metrics để xác định nguyên nhân |

Entry: functional smoke đúng, dataset/endpoint/mục tiêu rõ, môi trường thuộc quyền test. Exit bài tập: accounting khớp, đủ profile, gate đạt, raw evidence lưu và hạn chế công bố. Nếu fail do generator, kết luận thí nghiệm chưa đạt tải yêu cầu, không vội khẳng định server không chịu nổi. Nếu server trả sai tiền dù tải thấp, đó là failure cần điều tra độc lập với latency.

## 8. Mở rộng nâng cao

### 8.1. Ước lượng worker và đo saturation

Trong điều kiện phù hợp, concurrency trung bình gần arrival rate × thời gian trung bình một iteration. Ví dụ giả định 30/s và 0,2 giây thì trung bình khoảng 6 tác vụ đang chạy; cần headroom cho tail/biến thiên. Không dùng mean để cam kết cap 6 là đủ. Với p99 cao, số slot bận có thể vượt ước lượng và tạo drops; tăng cap chỉ có ý nghĩa nếu generator và hệ thống còn tài nguyên.

### 8.2. Workload mix và dữ liệu thực tế

Thêm ride/parcel/food, quãng đường và thành phố theo phân bố có căn cứ. Với tạo chuyến, cần khách/tài xế đủ lớn, idempotency key riêng và policy cleanup. Không tạo hàng nghìn request với cùng key rồi tuyên bố kiểm throughput tạo mới: phần lớn có thể chỉ replay cached response. Tách transaction mới và replay để đọc đúng tải.

### 8.3. Kiểm tải đọc/ghi và DB lớn

300 quote mới trên DB nhỏ chưa mô phỏng index/buffer/cache của dữ liệu nhiều tháng. Dựng dataset đại diện, kiểm query plan và connection pool, theo dõi lock/wait/IO. Không chỉ tăng số VU mà bỏ qua kích thước dữ liệu. PGlite không thay kiểm nhiều transaction của PostgreSQL server thật.

### 8.4. Rate theo endpoint và tail latency

Một iteration nghiệp vụ có thể gồm quote, book và nhiều polling calls. 30 iterations/s không còn là 30 HTTP requests/s. Ghi metric theo operation; percentile gộp có thể che endpoint ít traffic nhưng chậm. Nếu tạo tải theo lịch, đo cả lag tại generator và latency sau khi request thật được gửi để thấy phần chờ trước send.

### 8.5. So sánh release và độ tin cậy

Chạy baseline/candidate với cùng machine, dữ liệu, profile, cache policy và đủ thời gian. Lặp nhiều lượt để quan sát biến thiên, không kết luận từ vài ms chênh lệch ở một lượt 14 giây. Đặt threshold theo SLO và mục tiêu thí nghiệm trước chạy; luôn giữ kiểm correctness khi tối ưu.

## 9. Anti-pattern và giới hạn

Gọi “100 users” mà không mô tả hành vi; đổi sleep để đạt RPS mong muốn rồi bỏ scheduler lag; báo p95 chỉ của response thành công; dùng mọi request cùng key; chạy chưa đủ tải nhưng vẫn pass; lấy loopback một process để dự báo cluster — đều tạo kết luận thiếu căn cứ.

Profile trong bài chỉ 14 giây theo lịch, có drain giữa bước, không phải steady-state dài hay soak. Không thu CPU/DB telemetry tách biệt; chưa có browser, mạng ngoài, payment provider, load generator riêng hoặc k6 thực thi. Bài không chứng minh giới hạn hệ thống, và việc giảm về tải thấp không thay Recovery Test.

## 10. Bài tập và đáp án định hướng

1. **Cơ bản:** planned=300, sent=270, errors=0 có đạt gate không? Không: drop rate 10%, chưa đạt tải; 0 lỗi trên sent không bù được slot bị bỏ.
2. **Trung bình:** 50 arrivals/s, iteration trung bình 0,4 giây cần concurrency trung bình khoảng bao nhiêu? Khoảng 20, cộng headroom; con số này không bảo đảm tail không làm cạn slot.
3. **Trung bình:** đổi iteration thành quote rồi book, rate=30/s thì request rate mục tiêu ít nhất bao nhiêu? Khoảng 60/s nếu mỗi iteration gửi đúng hai calls; retry/polling làm tăng thêm. Cần account mix và rate-limit policy phù hợp.
4. **Nâng cao:** thiết kế generator và server ở hai process/máy. Đáp án cần đồng bộ cấu hình, đo CPU/network hai phía, lịch arrival, giới hạn in-flight, raw samples và endpoint kiểm đúng; không chỉ tách process rồi coi mọi nhiễu đã hết.
5. **Tự kiểm:** drop do cap thiếu có luôn là server quá tải? Không. p95 thấp sau khi drop nhiều slot có đủ chứng minh tải mục tiêu đạt? Không.

## 11. Nguồn tham khảo

- [k6 — Arrival-rate VU allocation](https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/arrival-rate-vu-allocation/): cần đủ worker cho lịch arrival, quan hệ với iteration duration.
- [k6 — Dropped iterations](https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/dropped-iterations/): phân biệt nguyên nhân cấu hình và hệ thống, đưa drops vào tiêu chí đánh giá.
- SPEC lab R-API-07/R-FARE: hợp đồng báo giá và oracle 35.000. Script riêng trong bài không phải k6 và không tuyên bố chạy k6.

Nguồn đối chiếu ngày 13/11/2026. Bài tiếp theo: Stress Test — tăng tải có giới hạn, quan sát suy giảm và hành vi sau khi giảm tải.
