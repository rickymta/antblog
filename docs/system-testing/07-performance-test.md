# SYS-07 — Performance Test: đo đúng câu hỏi trước khi tăng tải

Trạng thái: bản nháp hoàn chỉnh để review; có script và số đo. Bài trước: [Regression Test](06-regression-test.md). [Lộ trình](../system-testing-roadmap.md).

## 1. Bài toán và mục tiêu

API báo giá trả đúng tiền, nhưng khách chờ lâu rồi bấm lại. Nhóm nhận xét “API nhanh trên máy tôi” hoặc “test đạt 2.000 request/giây” mà chưa nói workload, số lỗi, dữ liệu hay môi trường. Hai kết luận đó chưa đủ để quyết định chất lượng hiệu năng.

Performance testing đánh giá đặc tính thời gian, thông lượng và sử dụng tài nguyên dưới điều kiện xác định. Sau bài, bạn có thể đặt câu hỏi đo được, chọn ranh giới thời gian, tính percentile/throughput/error rate, lưu mẫu gốc và giải thích giới hạn của một baseline local. Cần biết HTTP, TypeScript và khái niệm test oracle; không cần cài k6 cho lab này.

## 2. Khái niệm: performance bao trùm nhiều mục tiêu

| Chủ đề | Câu hỏi |
|---|---|
| Performance baseline | Hệ thống trong cấu hình và workload này đang có đặc tính gì? |
| Load testing | Ở tải dự kiến, có đạt mục tiêu không? |
| Stress testing | Khi vượt tải thường hoặc tiến gần giới hạn, hệ thống suy giảm thế nào? |
| Spike testing | Tải tăng đột ngột gây hậu quả gì? |
| Soak testing | Chạy dài có tích lũy leak, backlog hoặc mất ổn định không? |

Performance là nhóm mục tiêu rộng; Load và Stress sẽ được triển khai riêng ở bài 08/09. Một microbenchmark hàm tính cước không đại diện latency HTTP có DB; một bài HTTP không chứng minh thời gian render UI. Chọn đúng đối tượng trước khi chọn công cụ.

Functional correctness vẫn cần kiểm: trả 200 rất nhanh nhưng tổng tiền sai không phải thành công. Tách lỗi transport, HTTP và nghiệp vụ khi đọc kết quả. Với endpoint có failure hợp lệ theo nghiệp vụ, định nghĩa expected mix trước, không gộp mọi 4xx thành hỏng hoặc coi mọi 200 là đạt.

## 3. Định nghĩa phép đo và mẫu số

Latency là thời gian một lần thực hiện trong ranh giới đã chọn. Trong lab, bắt đầu ngay trước fetch và kết thúc sau đọc/parse JSON cùng kiểm oracle; gồm client, loopback, server và xử lý response. Nó không chỉ là thời gian hàm backend và không giống hoàn toàn `http_req_duration` của một công cụ khác.

Throughput hoàn tất = số attempts hoàn tất / thời gian cửa sổ đo. Goodput trong bài được ghi là validRps = số request đạt oracle / thời gian. Error rate = số attempts không đạt oracle / tổng attempts đo. Khi không lỗi, throughput và goodput bằng nhau; khi server trả lỗi nhanh, throughput cao vẫn có thể che chất lượng thấp.

Percentile p95 là giá trị mà ít nhất khoảng 95% mẫu không vượt quá theo phương pháp tính đã chọn. Script dùng nearest-rank: sort N mẫu tăng dần, lấy vị trí ceil(0,95×N), đánh số từ 1. Với N=300, p95 ở vị trí 285, p99 ở 297. p99 dựa vào rất ít mẫu phía đuôi nên không ổn định như một thí nghiệm dài có nhiều dữ liệu.

Không lấy trung bình ba p95 rồi gọi là p95 toàn bộ 900 request. Muốn percentile gộp phải gộp mẫu gốc hoặc histogram tương thích rồi tính lại. Mean thấp cũng không loại trừ vài request rất chậm. Script ghi cả p50/p95/p99/max, toàn bộ mẫu và trạng thái của từng attempt.

## 4. Thiết kế trước khi chạy

Câu hỏi của bài là: **một request báo giá nối tiếp trên HTTP loopback với kho memory có trả đúng kết quả, và phân bố thời gian quan sát được là gì?** Đây là bài baseline nhập môn, không phải capacity test.

| Thành phần thiết kế | Giá trị |
|---|---|
| Endpoint | POST /api/quotes |
| Input | C001, ride, SG, bike, 5 km, không voucher |
| Oracle | HTTP 200, total 35.000, quoteId là chuỗi |
| Giờ/thời tiết | 09:00 VN 04/11/2026 cố định, không mưa |
| Storage | Memory; mỗi request vẫn lưu quote |
| Mô hình | Closed, concurrency=1, không think time |
| Số vòng | 3, cùng process; mỗi vòng warm-up 50 và đo 300 |
| Dữ liệu đầu vòng | Reset trước warm-up, reset lần nữa trước đo |
| Deadline request | 2.000 ms |
| Gate minh họa | Mỗi vòng error rate=0 và p95<1.000 ms |

Ngưỡng 1.000 ms là **giả định bài tập đặt trước phép đo**, không phải SLO cam kết của Tới Nơi, không phải chuẩn ngành. Muốn đánh giá yêu cầu phi chức năng của sản phẩm phải dùng đúng endpoint, môi trường, workload và ngưỡng trong test basis. Baseline không tự tạo ra SLO chỉ bằng cách lấy số vừa đo cộng một tỷ lệ tùy ý.

Warm-up làm hoạt động đường code, connection và JIT trước khi đo. Reset dữ liệu loại quote đã tạo trong warm-up khỏi dataset đo, nhưng không reset JIT, heap hay cache của process. Mỗi vòng đo vẫn tích lũy 300 quote; ba vòng không phải ba process khởi động độc lập. Điều này cần ghi khi diễn giải xu hướng nhanh dần.

## 5. Closed model và coordinated omission

Một worker của lab chỉ gửi request tiếp khi request trước hoàn tất. Khi server chậm, tốc độ gửi tự giảm. Đó là closed model, phù hợp câu hỏi một luồng làm việc nối tiếp nhưng không mô phỏng dòng yêu cầu đến độc lập ngoài đời.

Open model lên lịch arrival độc lập với thời gian phản hồi. Khi service chậm, queue hoặc concurrency tăng, hoặc generator không tạo được tải dự kiến. Nếu mục tiêu là giữ một arrival rate mà lại dùng closed model, kết quả có thể bỏ qua phần chờ đáng lẽ tích lũy khi tải vẫn tiếp tục tới — thường được thảo luận dưới tên coordinated omission.

Concurrency=1 không phải “một người dùng thật” đầy đủ: không có think time, hành trình nhiều endpoint hay network latency ngoài loopback. Throughput thu được là output của thí nghiệm này, không phải offered RPS cố định và không phải công suất tối đa. Bài Load Test sẽ thiết kế mục tiêu tải cụ thể hơn.

## 6. Lab có thể chạy lại

Mã: `examples/toi-noi-testing/articles/system-testing-performance-test/baseline.ts`. Từ thư mục lab, Node >=22.12:

```sh
npm ci
npx tsx articles/system-testing-performance-test/baseline.ts
npm run typecheck
```

Không cần mở server trước. Script lắp app thật với memory system và mở cổng ngẫu nhiên trên 127.0.0.1, không nhận địa chỉ production. Sau cùng đóng server trong finally. Nếu warm-up sai oracle, dừng thay vì tiếp tục đo một hệ thống chưa hoạt động đúng.

Đầu ra `reports/performance/sys07-baseline.json` gồm cấu hình, máy, thời điểm UTC, 900 mẫu đo, summary và gate. Warm-up không nằm trong percentiles. Nếu measured attempt timeout/lỗi, nó vẫn có thời lượng, error và valid=false, được tính vào error rate và phân bố all-attempts. Không âm thầm bỏ request chậm hoặc thất bại để đồ thị đẹp hơn.

Đồng hồ dùng `performance.now()` cho khoảng thời gian, không lấy thời gian nghiệp vụ FixedClock vốn cố định. Deadline dùng AbortSignal.timeout; nếu event loop nghẽn, thời gian quan sát có thể vượt 2.000 ms, không được tự clamp số đo xuống timeout cấu hình.

CPU là delta user+system của **cả process client và server**, RSS là snapshot đầu/cuối từng vòng. Không gọi RSS cuối là peak memory và không suy ra memory leak chỉ từ tăng RSS. V8 có thể giữ bộ nhớ đã cấp phát; muốn chứng minh leak cần thí nghiệm dài hơn, GC/heap evidence và trạng thái dữ liệu tương đương.

## 7. Kết quả thực đo và cách đọc

Lượt ngày 13/11/2026 trên Windows, Node 24.15.0, i7-10700K, 16 logical CPUs, RAM khoảng 31,9 GiB:

| Vòng | Mẫu | Lỗi | p50 ms | p95 ms | p99 ms | Completed RPS |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 300 | 0 | 0,688 | 1,139 | 1,642 | 1.326,9 |
| 2 | 300 | 0 | 0,551 | 0,950 | 1,771 | 1.592,3 |
| 3 | 300 | 0 | 0,440 | 0,691 | 1,331 | 1.938,0 |

Ba vòng đạt gate minh họa, nhưng mỗi cửa sổ chỉ dài khoảng 0,15–0,23 giây. Các giá trị RPS là số chuẩn hóa từ cửa sổ ngắn, không chứng minh duy trì được tốc độ ấy trong một giây, một giờ hoặc với DB/network thật. Số p95 thấp không chứng minh hệ thống đáp ứng mục tiêu production.

Vòng sau nhanh hơn có thể liên quan JIT, connection reuse, cache hoặc nhiễu máy; dữ liệu này chưa cô lập nguyên nhân nên không kết luận đã tối ưu phần mềm. Max vẫn khoảng 5–6 ms. Không chọn riêng vòng tốt nhất làm đại diện rồi bỏ hai vòng còn lại.

Chi tiết máy, CPU/RSS và evidence xem [báo cáo](07-test-report.md) và JSON raw được lưu cùng lab. Lần chạy khác có thể cho số khác; script không assert đúng các con số này, chỉ áp gate đã nêu.

## 8. Điều tra và tiêu chí kết thúc

| Quan sát | Giả thuyết | Bằng chứng cần thêm |
|---|---|---|
| RPS tăng nhưng validRps giảm | Server trả lỗi nhanh | Status/error breakdown và oracle nghiệp vụ |
| p99 tăng, median ổn | GC, lock, scheduler, dependency chậm | Trace/profile và timeline tài nguyên |
| CPU cao | Client hoặc server có thể là nút thắt | Tách process/máy, đo từng phía |
| RSS tăng | Quote tích lũy hoặc heap giữ allocation | Heap/GC và thời gian dài, kiểm sau cleanup |
| Timeout tăng | Service hoặc generator nghẽn | Event loop, sockets, DB queue, tải thực đạt |

Entry: functional smoke đúng, môi trường/dữ liệu/mục tiêu rõ, không có tác vụ tải nặng khác chủ động chạy cùng lúc. Exit bài tập: script hoàn tất đủ mẫu, raw evidence được lưu, oracle không lỗi và gate minh họa đạt; mọi hạn chế công bố. Nếu không đạt, giữ số liệu và exit nonzero, không tự nới threshold để làm xanh.

## 9. Mở rộng nâng cao

### 9.1. Đo trên môi trường đại diện

Tách load generator khỏi server; dùng DB thật với kích thước dữ liệu, index và connection pool đại diện. Ghi version build, cấu hình, CPU/RAM, network, cache state và các dependency. Chạy đủ dài để quan sát steady state và kiểm lại generator không bão hòa trước server. Loopback memory của bài chỉ chuẩn bị phương pháp.

### 9.2. SLI/SLO và workload mix

Một mục tiêu hữu ích có đối tượng, thời gian, tải và tiêu chí lỗi: ví dụ giả định “báo giá hợp lệ p95<300 ms tại 50 arrivals/s trong 10 phút, lỗi <1%”. Các số này là ví dụ thiết kế, không phải cam kết sản phẩm. Cần phân nhóm ride/parcel/food và theo endpoint; percentile gộp có thể che nhóm ít request nhưng rất chậm.

### 9.3. Độ tin cậy thống kê

Ba lượt ngắn không đủ ước lượng tail latency ổn định. Tăng thời lượng/mẫu, lặp ở điều kiện có kiểm soát và công bố biến thiên. Không gộp số liệu của các cấu hình khác nhau rồi kết luận một SLO. So baseline trước/sau cần cùng workload và môi trường; nếu thay nhiều yếu tố, phải phân biệt correlation với nguyên nhân.

### 9.4. Little's Law và kiểm tra tính hợp lý

Trong điều kiện ổn định phù hợp, L≈λW liên hệ số request trung bình trong hệ thống, throughput và thời gian trung bình trong cùng ranh giới. Dùng nó để phát hiện số liệu không nhất quán, không thay phép đo. Với closed concurrency=1 và không think time, throughput gần nghịch đảo thời gian một chu kỳ; không thể tùy tiện nhân RPS đo được với 16 logical CPUs để dự đoán capacity.

### 9.5. Profiling và tối ưu có kiểm chứng

Khi xác định một nút thắt, thay đổi một yếu tố, chạy lại cùng bài đo và kiểm functional regression. Tối ưu bỏ lưu quote có thể làm latency đẹp nhưng phá nghiệp vụ đặt từ quote. Theo dõi cả độ đúng, tail latency, error và tài nguyên. Không lấy thời gian suite Vitest làm proxy cho latency sản phẩm.

## 10. Anti-pattern và giới hạn

Đo health endpoint rồi suy ra khả năng tạo chuyến; chỉ báo average; bỏ timeout khỏi percentile; quên validate body; chạy generator cùng process mà gọi CPU là server-only; dùng vài trăm request để tuyên bố công suất production — đều làm kết luận yếu.

Lab không có DB ngoài, TLS, mạng WAN, external payment, nhiều user hay tải mở; không đo startup trong latency, không đo peak RSS hoặc steady state dài. Nó cũng không thực hiện Load/Stress Test của bài tiếp theo. Không có lỗi sản phẩm được phát hiện ở lượt đo này, nhưng không đồng nghĩa không có bottleneck.

## 11. Bài tập và đáp án định hướng

1. **Cơ bản:** với 100 request, 4 sai oracle trong 2 giây, throughput=50 RPS, validRps=48, error rate=4%. Chỉ báo 50 RPS bỏ mất điều gì? Chất lượng của kết quả trả về.
2. **Trung bình:** hai vòng có p95 10 và 100 ms có p95 gộp=55 không? Không; cần mẫu gốc/histogram cùng định nghĩa để tính.
3. **Trung bình:** tăng mẫu lên 3.000 có tự biến thành load test tải dự kiến không? Không; phải xác định workload/arrival/concurrency và mục tiêu, không chỉ tổng số request.
4. **Nâng cao:** thiết kế so sánh memory với PostgreSQL server. Đáp án cần process riêng, schema/index/dataset, warm-up, connection pool, cùng input mix, đo server và generator, và không quy mọi chênh lệch cho DB nếu network cũng đổi.
5. **Tự kiểm:** p95<1 ms có chứng minh UI nhanh không? Không. Gate pass có phải đạt SLO production? Không, gate ở đây chỉ là điều kiện minh họa local.

## 12. Tài liệu tham khảo

- [Grafana k6 — Metrics](https://grafana.com/docs/k6/latest/using-k6/metrics/): các nhóm metric và cách dùng request/error/duration.
- [Grafana k6 — Open and closed models](https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/open-vs-closed/): quan hệ giữa iteration duration và tốc độ gửi.
- [Node.js Performance measurement APIs](https://nodejs.org/api/perf_hooks.html): performance.now và các công cụ đo; lab dùng Node 24.15.0, chỉ dùng API đã kiểm chạy trên phiên bản đó.
- SPEC lab, R-FARE và R-API-07: oracle nghiệp vụ của POST báo giá.

Nguồn đối chiếu ngày 13/11/2026. Bài tiếp theo: Load Test — xây mô hình tải dự kiến và đo hệ thống dưới tải đó.
