# SYS-09 — Stress Test: quan sát suy giảm khi tải vượt mức dự kiến

Trạng thái: bản nháp hoàn chỉnh để review, có lab và evidence. Bài trước: [Load Test](08-load-test.md). [Lộ trình](../system-testing-roadmap.md).

## 1. Tình huống và mục tiêu

Khuyến mãi khiến lượng báo giá tăng nhiều lần. Tới Nơi có thể phản hồi chậm, bỏ yêu cầu, hoặc tiếp tục trả 200 nhưng sai dữ liệu. Khi chiến dịch kết thúc, traffic giảm nhưng backlog có thể còn. Câu hỏi không chỉ là “server có chết không” mà là hệ thống suy giảm ở đâu, giữ được điều gì và trở lại hoạt động thế nào.

Bài dành cho người đã biết latency, throughput, open model và dropped slots ở SYS-07/08. Sau bài, bạn có thể thiết kế mức tải cao có giới hạn, đặt điều kiện dừng, phân biệt lỗi generator với lỗi hệ thống, kiểm sau tải và báo cáo cả điều chưa xác định được. Không yêu cầu làm sập máy hoặc tìm breakpoint bằng mọi giá.

## 2. Stress, spike, breakpoint và recovery

Stress testing đánh giá hành vi dưới tải cao hơn mức thông thường đã xác định. Không có tỷ lệ nhân cố định phù hợp mọi hệ thống. Cần có baseline/load test trước để hiểu trạng thái bình thường, rồi chọn mức tăng theo rủi ro.

| Loại kiểm | Mục tiêu | Khác biệt |
|---|---|---|
| Load | Đạt tiêu chí ở tải dự kiến | Tập trung mức vận hành mục tiêu |
| Stress | Hành vi khi tải cao hơn bình thường | Chấp nhận quan sát suy giảm, vẫn giữ oracle dữ liệu |
| Spike | Phản ứng với thay đổi tải đột ngột | Tốc độ tăng/giảm là yếu tố chính |
| Breakpoint | Tìm giới hạn theo tiêu chí định trước | Cần mở rộng đủ mức tải và xác minh generator |
| Recovery | Khả năng phục hồi sau sự cố | Cần failure model, thời gian và tính toàn vẹn dữ liệu |

Lab dùng các bước tải tăng, không phải ramp tuyến tính hoặc spike production. Mức 30/s là normal giả định từ bài Load, không phải dữ liệu traffic thật. 150/s và 600/s được chọn để học cách quan sát mức cao hơn, không tuyên bố đó là capacity limit của Tới Nơi.

## 3. Những thuộc tính phải giữ khi quá tải

Có thể cho phép latency xấu đi hoặc từ chối có kiểm soát, nhưng không tự cho phép tính sai tiền, ghi trùng chuyến hay báo thanh toán sai. Định nghĩa degraded behavior cần dựa trên hợp đồng: 429 do quota khác 503 do service không nhận thêm, timeout phía client lại chưa chắc biết server đã thực hiện thao tác hay chưa.

Với endpoint báo giá của lab, mỗi response được kỳ vọng HTTP 200, total=35.000 và quoteId dạng chuỗi. Lỗi được ghi rõ, không loại khỏi percentile. Dropped slots ở generator không đi tới server nên không được gọi là server đã từ chối. Endpoint ghi quote nên dữ liệu tích lũy; không lấy health request thay workload nghiệp vụ.

## 4. Thiết kế thí nghiệm có ranh giới

| Mục | Thiết kế |
|---|---|
| SUT | Express + use cases + repository + PGlite local |
| Profile | Baseline 30/s×3s → elevated 150/s×4s → high 600/s×4s → return-low 30/s×3s |
| Quy mô | 3.180 slots, warm-up 20, một quote probe cuối và một health request |
| Request | C001, ride SG bike 5 km, không voucher, giờ thường, không mưa |
| Scheduling | Open slots; không await response để gửi slot tiếp |
| Trần | 40 in-flight; slot trễ >100ms bỏ, không bắn bù |
| Dừng gửi | Sau 10 request failures, các slots sau ghi failure-stop |
| Deadline | 2 giây/request; finite profile thay cho tăng vô hạn |
| Data | Reset sau warm-up, giữ dữ liệu xuyên bốn bước |

Mỗi bước drain trước bước sau, nên khoảng chuyển có thể dài hơn scheduled profile. Điều này làm giảm khả năng quan sát backlog liên tục; cần nêu thay vì gọi đây là ramp không gián đoạn. Nếu safety stop đã kích hoạt, return-low slots cũng bị bỏ; probe cuối vẫn chạy một lần nhưng không thay thế đầy đủ phase giảm tải.

Client, server, DB chung process/event loop. CPU máy và generator có thể trở thành giới hạn trước SUT riêng biệt. Vì vậy drops/lag chỉ là bằng chứng cần điều tra, không đủ xác định năng lực PostgreSQL hoặc backend production.

## 5. Tiêu chí: quan sát suy giảm khác với bỏ qua lỗi

Giữ reference envelope: zero errors/drops và p95<500ms cho từng bước, nhưng diễn giải theo vai trò bước. Baseline và return-low phải trong envelope. Ở elevated/high, p95 cao hoặc drops là kết quả quan sát suy giảm, không tự khiến script exit 1. Bất kỳ HTTP/oracle/transport error vẫn khiến exit 1; probe quote và health phải đạt.

Exit 0 vì thế chỉ nói các điều kiện bắt buộc của bài đạt, **không** nói mọi slot tải cao được gửi đủ. Báo cáo cần trình bày sent/dropped và envelope của từng bước. Không thay tiêu chí sau khi thấy hệ thống không chịu tải; nếu mục tiêu release yêu cầu zero drop ở 600/s thì phải gate theo mục tiêu đó trước khi chạy.

Điều kiện dừng và ngưỡng là giả định thử nghiệm local, không phải chuẩn ngành. Dừng theo lỗi giúp giới hạn tác động nhưng có độ trễ bởi các request đang chạy; tổng lỗi có thể vượt 10 trước khi tất cả in-flight kết thúc. Không gọi 10 là trần tuyệt đối số failure.

## 6. Lab từng bước

Mã: `examples/toi-noi-testing/articles/system-testing-stress-test/stress.ts`. Từ thư mục lab, Node >=22.12:

```sh
npm ci
npx tsx articles/system-testing-stress-test/stress.ts
npm run typecheck
```

Script kế thừa cách lập lịch của SYS-08 nhưng giữ file riêng để bài trước vẫn tái hiện đúng. Nó tự tạo server trên loopback cổng ngẫu nhiên, không nhận URL bên ngoài; đóng server và DB trong finally. File kết quả nằm dưới `reports/stress/sys09-stress.json` tương đối thư mục chạy.

**Bước 1:** 20 request warm-up phải đúng oracle; reset dữ liệu trước baseline. **Bước 2:** gửi theo slots, ghi start/end offsets, lag và kết quả từng request. **Bước 3:** drain và tính metrics từng mức, không gộp hết thành một p95 che bước tệ nhất. **Bước 4:** giảm tải, thử thêm một báo giá nghiệp vụ và health; lưu raw JSON trước khi quyết định exit code.

Quote probe kiểm service còn xử lý được nghiệp vụ, health chỉ kiểm phản hồi tiến trình. Một health 200 không thay quote đúng tiền. Probe đơn sau drain cũng chưa đo recovery time từ thời điểm quá tải: cần timeline liên tục và tiêu chí ổn định trong nhiều cửa sổ để tính đại lượng đó.

## 7. Đọc kết quả theo nguyên nhân có thể kiểm chứng

Kết quả thực tế và raw evidence xem [báo cáo SYS-09](09-test-report.md). Trước hết kiểm `planned=sent+dropped`, lỗi trên sent và lý do dropped; sau đó mới so latency/throughput.

| Mẫu quan sát | Điều có thể suy ra | Chưa thể kết luận |
|---|---|---|
| Latency tăng, in-flight tăng | Tác vụ giữ slot lâu hơn | Chính xác CPU/SQL/network nào nghẽn |
| Lag tăng, nhiều scheduler-late | Generator không giữ lịch | Server riêng biệt không chịu được target |
| Sent đủ, lỗi tăng | Requests đã tới nhưng không đạt oracle | Nguyên nhân chỉ từ tải, cần phân loại lỗi |
| Return-low đạt và probe đúng | Sau phase/drain, luồng chọn hoạt động | Khôi phục đầy đủ mọi tài nguyên hoặc RTO đạt |
| Tất cả mức vẫn đạt | Chưa quan sát failure trong giới hạn thử | Đã tìm capacity tối đa |

Không ép kết quả phải có lỗi để “đúng bài Stress”. Nếu 600/s vẫn đạt trong lab, ghi chưa tìm thấy điểm gãy; bước tiếp theo là thiết kế thí nghiệm mới có phạm vi/tài nguyên phù hợp, không tự tăng tải vô hạn. Nếu drops ở generator, ưu tiên tách process/máy trước khi nâng target.

## 8. Mở rộng nâng cao

### 8.1. Graceful degradation và backpressure

Hệ thống thật có thể dùng queue hữu hạn, admission control, rate limit hoặc circuit breaker. Test cần xác định khi nào từ chối, response có giúp client retry đúng không, và các giao dịch đã nhận có hoàn tất đúng không. Queue không giới hạn có thể làm latency tăng và cạn bộ nhớ; throughput cao ban đầu không chứng minh ổn định.

### 8.2. Retry amplification

Nếu nhiều client retry tức thì khi timeout, tải mới và tải retry cộng lại. Thiết kế workload cần tách attempts với logical transactions, ghi retry count và idempotency key. Kiểm jitter/backoff và retry budget khi đã có yêu cầu tương ứng; không chỉ tăng số requests rồi bỏ qua hành vi client. Lab hiện không retry request báo giá, nên chưa mô phỏng retry storm.

### 8.3. Không mất tính toàn vẹn dưới áp lực

Với booking/payment, kiểm invariant số chuyến, event, tiền thu và idempotency trong suốt hoặc sau tải. Timeout không đồng nghĩa chưa ghi dữ liệu. Test phải đối soát những logical transactions có kết quả không rõ, không gửi key mới rồi coi đó là retry an toàn. Lab chỉ báo giá, chưa kiểm cạnh tranh ví hoặc dispatch.

### 8.4. Đo recovery có thời gian

Định nghĩa trạng thái phục hồi trước: ví dụ giả định ba cửa sổ liên tiếp có zero errors và p95 dưới ngưỡng ở tải thường, backlog về mức chấp nhận. Ghi thời điểm giảm tải và thời điểm đạt liên tiếp để tính time-to-recover. Cần tách thời gian drain do test harness khỏi hành vi hệ thống. Một probe xanh không đủ chứng minh ổn định.

### 8.5. Autoscaling và capacity planning

Trên staging có autoscaler, ghi replica count, thời gian khởi động, readiness, queue và saturation từng instance. Tăng target nhanh hơn tốc độ scale có thể gây lỗi tạm thời dù capacity cuối đủ. Không lấy 600/s của một phiên PGlite để nhân số node hoặc dự báo chi phí cluster. Capacity planning cần workload đại diện và margin rủi ro.

## 9. Anti-pattern và giới hạn

Không dùng production hoặc endpoint bên ngoài khi chưa thuộc phạm vi thử. Không bỏ guard để cố tìm crash. Không gọi mọi drop là 503. Không báo “phục hồi hoàn toàn” chỉ vì health xanh. Không lấy metric gộp che latency bước high hoặc bỏ các request timeout khỏi phân bố.

Lab chỉ kéo dài 14 giây theo lịch cộng drain/warm-up; không phải soak, không đo memory leak, RTO/RPO hoặc crash recovery. Không có monitoring tách client/server/DB. Nếu không kích hoạt safety stop, chỉ ghi guard có trong code chứ chưa tuyên bố đã kiểm nhánh đó. Không sửa src sản phẩm để tạo ra lỗi mong muốn.

## 10. Bài tập và đáp án định hướng

1. **Cơ bản:** high có 10% drop nhưng return-low/probe đạt, exit 0 có nghĩa đạt tải high không? Không; đọc planned/sent và envelope riêng.
2. **Trung bình:** health 200 nhưng quote probe sai total. Kết luận: process sống, nghiệp vụ không đạt; script phải fail, không gọi recovery thành công.
3. **Trung bình:** 600/s không lỗi có nên viết capacity=600/s? Không; chỉ mức đã thử trong cấu hình/thời lượng đó, chưa tìm giới hạn hoặc độ bền dài hạn.
4. **Nâng cao:** thêm retry client. Đáp án cần budget, backoff/jitter, logical transaction ID và phân biệt offered mới với retry; kiểm idempotency nếu chuyển sang booking.
5. **Nâng cao:** thiết kế recovery observation không drain giữa high và low. Cần scheduler theo timeline chung, tracking request thuộc stage nào, đủ thời gian quan sát và tiêu chí ổn định; không trộn response đến muộn vào bucket sai.

## 11. Tài liệu tham khảo

- [Grafana k6 — Stress testing](https://grafana.com/docs/k6/latest/testing-guides/test-types/stress-testing/): mục tiêu tải cao hơn thông thường và cần baseline/load trước; mức tăng phụ thuộc rủi ro, không có phần trăm cố định.
- Bài SYS-07/08: metrics, arrival model, drops và giới hạn generator local.
- SPEC lab R-FARE/R-API-07: oracle của endpoint báo giá.

Nguồn đối chiếu ngày 13/11/2026. Script là Node lab tự viết, không phải k6 đã chạy. Bài tiếp theo: Security Test – Pen test, với threat model, phạm vi và bằng chứng kiểm bảo mật trên lab.
