# SYS-04 — Integration Test: khi các thành phần đúng riêng nhưng sai khi nối lại

Trạng thái: bản nháp hoàn chỉnh để review; có lab. Bài trước: [Functional Test](03-functional-test.md). [Lộ trình](../system-testing-roadmap.md).

## 1. Bài toán và mục tiêu

Hàm cước trả đúng 35.000, repository lưu được chuyến, cổng ví thu tiền thành công khi kiểm riêng. Khi khách hoàn tất chuyến, response của ví bị mất. Ứng dụng thử lại với khóa mới và khách bị thu hai lần. Mỗi thành phần có thể đã qua unit test, nhưng hợp đồng phối hợp vẫn sai.

Bài dành cho người đã biết API tests, SQL cơ bản và test doubles. Sau bài, bạn có thể xác định ranh giới tích hợp, chọn phần thật/phần giả có lý do, kiểm dữ liệu xuyên API–SQL, inject lỗi thanh toán và giới hạn đúng kết luận về retry, transaction và concurrency.

## 2. Khái niệm và phân biệt

Integration testing tập trung vào tương tác giữa các thành phần hoặc hệ thống: dữ liệu trao đổi, thứ tự gọi, mapping, xử lý lỗi và các tác dụng phụ liên quan. Component integration xét sự phối hợp giữa thành phần; system integration xét các giao tiếp giữa hệ thống với nhau hoặc dịch vụ ngoài. Cần ghi rõ ranh giới, vì tên “integration test” không tự cho biết đang chạy dependency thật nào.

| Phạm vi | Ví dụ Tới Nơi | Bằng chứng chính |
|---|---|---|
| Component | Hàm tính tiền giảm | Giá trị trả về theo oracle |
| Component integration | Use case + repository SQL | Mapping dữ liệu, ràng buộc và kết quả ghi/đọc |
| System integration | Ứng dụng + nhà cung cấp ví sandbox | HTTP contract, credentials, retry và semantics phía provider |
| System/E2E | Hành trình khách/tài xế qua hệ thống triển khai | Hành vi toàn luồng với cấu hình môi trường |

Lab này nối Express → use cases → PostgreSQL repository → PGlite và nối use case thanh toán tới fake PaymentGateway. Nó kiểm tích hợp nội bộ và cách consumer dùng payment port. Nó **chưa** kiểm adapter HTTP tới nhà cung cấp thanh toán thật. Một fake dùng đúng interface TypeScript vẫn có thể không giống semantics của nhà cung cấp.

## 3. Chiến lược ghép nối và chọn dependency

Ghép từng phần giúp khoanh vùng lỗi: kiểm repository trước, thêm use case, rồi API. Ghép tất cả cùng lúc có thể bắt vấn đề toàn luồng nhưng khó biết lỗi nằm ở serialization, schema hay nghiệp vụ. Top-down dùng thành phần tầng trên và thay tầng dưới bằng stub; bottom-up kiểm adapter/repository trước rồi ghép tầng trên. Đây là các lựa chọn chiến lược, không phải thứ tự bắt buộc cho mọi nhóm.

Trong bài, dùng SQL thật vì memory repository không thể chứng minh foreign key, kiểu numeric/jsonb hoặc mapping tên cột. Dùng ví giả vì cần chủ động tái hiện decline, timeout và mất phản hồi sau thu tiền mà không phụ thuộc mạng hoặc giao dịch ngoài. Fake clock loại bỏ giá cao điểm thay đổi theo giờ chạy.

| Thành phần | Cách dùng trong lab | Phạm vi chưa chứng minh |
|---|---|---|
| Express + use cases | Mã thật, gọi qua supertest | DNS, TLS, reverse proxy, deployment |
| Repository + schema | Mã thật, PGlite PostgreSQL WASM | Server PostgreSQL nhiều kết nối, network driver |
| PaymentGateway | Fake có ledger và fault modes | Provider HTTP schema, chữ ký, retry phía provider |
| Clock/weather | Fake cố định | Dịch vụ thời tiết, clock skew giữa máy |

Đánh đổi là độ trung thực, tốc độ, khả năng tái hiện và chi phí vận hành. Không lấy tỷ lệ dependency thật làm thước đo duy nhất: fake đúng ở một port có thể hữu ích hơn sandbox không tái hiện được lost response, miễn kết luận không vượt bằng chứng.

## 4. Hợp đồng và oracle xuyên ranh giới

Theo SPEC, một quote chỉ tạo một chuyến (R-QUOTE-03). Gửi lại POST tạo chuyến cùng key/body phải trả lại đúng status/body, có header replay và không thêm dữ liệu (R-IDEM-02). Chuyến ví hoàn thành thu đúng fare với khóa `trip:<id>:fare` (R-PAY-03). Timeout/lỗi tạm thời thử lại tối đa hai lần với cùng khóa; decline không retry (R-PAY-04). Thu tiền thất bại vẫn cho chuyến completed, paymentStatus failed và HTTP 200 (R-PAY-06).

Từ đó cần nhiều lớp oracle: response cho client, dữ liệu SQL, nhật ký trạng thái, trạng thái tài xế và ledger của fake ví. Chỉ kiểm HTTP 200 có thể bỏ sót payment failed; chỉ kiểm response paid có thể bỏ sót DB chưa cập nhật. Đọc trực tiếp SQL còn tránh việc adapter ghi sai và adapter đọc sai cùng một cách khiến round-trip qua repository vẫn xanh.

Foreign key `trip_events.trip_id → trips.id` là hợp đồng schema trong `src/infra/pglite/schema.sql`, không tự gán một mã yêu cầu nghiệp vụ mới. Khi insert event không có chuyến, SQLSTATE expected là 23503. Assert code và không có row, tránh phụ thuộc toàn bộ câu thông báo có thể đổi ngôn ngữ hoặc phiên bản.

## 5. Ma trận test và dữ liệu

Tiền điều kiện chung: seed C001/D101, xe máy SG 5 km, 09:00 VN 04/11/2026, không mưa, không voucher, fare=35.000. Mỗi execution reset DB/fake, dựng app mới; suite đóng PGlite khi kết thúc. Các case chạy tuần tự trong file để không reset DB khi case khác đang dùng.

| ID / dataset | Basis và hành động | Expected |
|---|---|---|
| IN-01 | R-QUOTE-03/R-IDEM-02: quote, tạo chuyến, gửi lại cùng key/body | 201/body giống nhau, replay=true; một trip; quote trỏ trip; chỉ requested/searching events |
| IN-02 | Schema FK: insert event cho missing-trip | Reject 23503; không có event |
| IN-03/none | R-PAY-03: hoàn tất bình thường | completed/paid; một call, thu 35.000 |
| IN-03/lost_response | R-PAY-04: đã thu nhưng mất phản hồi lần đầu | Hai call cùng key, paid; ledger chỉ thu 35.000 |
| IN-03/decline | R-PAY-04/06: ví từ chối | Một call, failed; không thu; vẫn completed |
| IN-03/timeout | R-PAY-04/06: mọi lần không phản hồi và chưa thu | Ba call cùng key, failed; thu 0; completed |
| IN-03/error | R-PAY-04/06: lỗi tạm thời mọi lần | Ba call cùng key, failed; thu 0; completed |

Mọi dòng thanh toán còn kiểm SQL payment_status, driver trở lại online và đúng một completed event. Hai lần gọi không đồng nghĩa hai lần thu: phải xem ledger. Ngược lại không thấy response không có nghĩa provider chưa thu tiền.

## 6. Thực hành từng bước

Mã đầy đủ: `examples/toi-noi-testing/articles/system-testing-integration-test/integration.test.ts`. Tại thư mục lab, Node >=22.12:

```sh
npm ci
npm test -- articles/system-testing-integration-test
npm run typecheck
```

`npm ci` chỉ cần khi chuẩn bị dependency theo lockfile. Expected là **7 tests pass**: hai ca riêng và năm dataset thanh toán. Kết quả thực tế được ghi trong [báo cáo](04-test-report.md).

### Bước 1 — Lắp ráp và kiểm soát vòng đời

`createPgliteSystem` tạo schema và seed; `beforeEach` gọi reset rồi dựng Express app mới để các trạng thái process-local không rò giữa test. `afterAll` đóng DB. Không dùng test.concurrent với setup này. Nếu muốn chạy song song, mỗi worker/case cần DB riêng hoặc namespace và cleanup thực sự cách ly.

### Bước 2 — Kích hoạt qua API, quan sát bằng SQL

IN-01 không insert trip trực tiếp để chuẩn bị: gọi API quote và booking thật, sau đó query các bảng. Dùng parameterized query cho ID thay vì ghép chuỗi. Event được sort theo identity id vì các timestamp cùng clock cố định có thể bằng nhau; ORDER BY thời gian đơn lẻ sẽ không đủ xác định thứ tự.

### Bước 3 — Tạo lỗi ở đúng ranh giới

Helper inTransit đưa chuyến qua accept/arrive/pickup/start bằng API. Sau đó mới cấu hình payment fault. `failNext('lost_response')` áp dụng cho lần gọi đầu, lần sau trở lại bình thường. `setMode('timeout')` áp dụng mọi lần nên chứng minh giới hạn retry; nếu chỉ failNext một timeout thì chưa kiểm đường hết lượt.

Lab giảm `timeoutMs` xuống **50 ms**, giữ `maxRetries=2`. Đây là cấu hình thử để kiểm logic deadline và số lần gọi nhanh; SPEC mặc định là 3 giây. Không dùng kết quả này để tuyên bố đã kiểm ngưỡng 3 giây hay SLA latency production. Fake timeout trả promise không hoàn tất; deadline trong ứng dụng quyết định tiếp tục, không cần sleep tùy ý trong test.

### Bước 4 — Kiểm kết quả ở nhiều phía

Mỗi payment call phải giữ customerId C001, amount 35.000 và cùng key theo trip. Sau complete, query DB phải khớp response; ledger phải có tối đa một khóa thu. Decline/timeout/error vẫn HTTP 200 theo SPEC vì hoàn tất chuyến và thu tiền là hai kết quả khác nhau. Test đòi 500 cho mọi lỗi ví sẽ áp đặt một hợp đồng sai.

## 7. Phân tích thất bại và quyết định

| Dấu hiệu | Khả năng cần kiểm | Bằng chứng tiếp theo |
|---|---|---|
| Response đúng, SQL sai | Mapping/update chưa lưu | Row theo tripId và câu SQL của adapter |
| Replay tạo thêm event | Idempotency không bao toàn bộ handler | Key scope, stored response, số row trước/sau |
| Lost response thu hai lần | Key đổi giữa attempts hoặc fake/provider không dedupe | Danh sách request và ledger |
| Decline có ba calls | Gộp lỗi nghiệp vụ với lỗi tạm thời | Branch xử lý ChargeResult |
| FK test không lỗi | Schema/constraint chưa được nạp | Kiểm schema thực của DB test |
| Test treo | Deadline/cleanup không hoạt động | Timer, promise và nơi chờ; không chỉ tăng timeout runner |

Entry: SPEC/schema xác định, app dựng được, test data độc lập. Exit: bảy executions pass, typecheck đạt, ranh giới thật/giả công bố rõ. Khi fail, ghi commit, case/dataset, cấu hình timeout, response, row SQL và calls đã quan sát. Không log token hoặc dữ liệu thanh toán thật khi mở rộng sang sandbox.

## 8. Mở rộng nâng cao

### 8.1. Timeout không hủy tác dụng phụ

`Promise.race` giới hạn thời gian chờ của caller, không tự hủy request đang xử lý ở provider. Lost response minh họa việc đã thu tiền dù caller timeout. Trong triển khai thực cần idempotency được provider bảo đảm, đối soát trạng thái giao dịch và cơ chế xử lý kết quả muộn. Không suy ra “failed ở DB” luôn có nghĩa “chưa thu tiền”. Nếu provider đã thu nhưng tất cả phản hồi mất, hệ thống có thể cần trạng thái không xác định và reconciliation; SPEC lab giản lược thành failed.

### 8.2. Transaction cục bộ và giao dịch ngoài hệ thống

API → SQL → ví không tự nằm trong một transaction ACID chung. Có cửa sổ ví đã thu nhưng cập nhật DB thất bại. Suite hiện tại chưa inject lỗi ở điểm đó. Bài mở rộng cần ép write paymentStatus thất bại sau charge thành công, quan sát ledger/DB lệch, rồi thiết kế reconciliation hoặc outbox workflow. Không “sửa” bằng rollback SQL và giả định tiền ngoài hệ thống cũng rollback.

### 8.3. Contract testing và provider verification

TypeScript interface kiểm hình dạng lời gọi nội bộ ở compile time, không kiểm payload HTTP, mã lỗi provider, đơn vị tiền hoặc chữ ký. Khi có adapter ngoài, lưu contract về request/response, kiểm serialization với stub HTTP và xác minh provider trên sandbox/version cụ thể. Fake cần được đối chiếu contract định kỳ để tránh drift. Adapter gửi 35.000 theo đồng nhưng provider hiểu đơn vị nhỏ hơn có thể khiến mọi test port xanh mà tích hợp thật sai.

### 8.4. Schema evolution và nhiều phiên bản ứng dụng

DB mới tạo sạch không chứng minh migration từ bản cũ đúng. Cần dựng schema cũ với dữ liệu đại diện, chạy migration rồi kiểm app cũ/mới trong khoảng rolling deploy; xem nullability, default, backfill và đọc/ghi hai chiều. Expand–migrate–contract là một hướng triển khai: thêm trường tương thích, chuyển dữ liệu/consumer, sau đó mới bỏ trường cũ. Các bước phải có tiêu chí và test tương ứng, không suy từ fresh schema pass.

### 8.5. Concurrency và ranh giới của PGlite

Lab dùng PGlite một kết nối; Promise.all không biến nó thành PostgreSQL server có nhiều connection. Kiểm tranh chấp hai tài xế nhận cùng chuyến cần hai transaction độc lập, điểm đồng bộ khởi chạy, isolation level rõ và invariant một người thắng. Test replay IN-01 là tuần tự, không chứng minh xử lý hai request đang in-progress. Phần concurrency cần môi trường riêng, chưa chạy ở bài này.

### 8.6. Broker, outbox và quan sát

Nếu sau này phát event TripCompleted, cần kiểm payload/version, correlation ID, retry, duplicate delivery và consumer idempotency. Oracle nên là trạng thái nghiệp vụ cuối cùng cộng số tác dụng phụ, không chỉ “message được publish”. Chờ có điều kiện với deadline thay cho sleep cố định. Lab chưa có broker/outbox nên đây là hướng thiết kế mở rộng, không phải chức năng đang được kiểm.

## 9. Anti-pattern và giới hạn

Mock repository rồi gọi suite là đã kiểm SQL sẽ bỏ qua schema. Assert exact toàn bộ lỗi DB dễ giòn. Reset dữ liệu chung khi test song song tạo flakiness. Retry cả decline có thể gây nhiễu và không đúng hợp đồng. Assert số calls mà không kiểm ledger sẽ bỏ sót thu trùng. Tăng timeout runner để che deadlock khiến phản hồi chậm hơn mà chưa sửa nguyên nhân.

Suite chưa kiểm toàn bộ ràng buộc schema, restart durability, rollback khi giữa luồng bị lỗi, provider thật, JWT, browser, network, concurrency hoặc migration. Khác biệt quan trọng với SYS-03 là oracle SQL và phối hợp qua các adapter thật, không phải chỉ thêm số lượng ca.

## 10. Bài tập và đáp án định hướng

1. **Cơ bản:** thêm một lần `error` rồi thành công. Expected hai calls, cùng key, completed/paid, ledger 35.000. Dùng failNext thay setMode để giới hạn fault.
2. **Trung bình:** gửi cùng key nhưng quoteId khác. Theo R-IDEM-03 expected 422 IDEMPOTENCY_KEY_REUSED; kiểm số trip không tăng và quote mới chưa bị dùng. Đây là case khác replay cùng body.
3. **Trung bình:** đặt chuyến cash rồi complete. Expected paymentStatus cash, không có call ví, DB completed và driver online; không áp oracle paid của vixanh.
4. **Nâng cao:** ép cập nhật SQL lỗi sau ví thu thành công. Đáp án cần nêu cửa sổ mất nhất quán, số tiền thực đã thu và cơ chế sửa/đối soát; không chỉ retry toàn luồng với key mới.
5. **Tự kiểm:** vì sao hai calls vẫn chỉ thu một lần? Provider fake dedupe cùng idempotency key. Có chứng minh provider thật làm vậy không? Không, cần provider verification. Bảy case pass có chứng minh transaction nguyên tử qua SQL và ví? Không.

## 11. Nguồn tham khảo

- [ISTQB CTFL v4.0.1](https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf), mục 2.2.1: phân biệt component integration và system integration.
- [PGlite Getting Started](https://pglite.dev/docs/): khởi tạo PostgreSQL WASM trong tiến trình, query có tham số; cần phân biệt cấu hình in-memory và lưu filesystem.
- [PostgreSQL 18 — Constraints](https://www.postgresql.org/docs/18/ddl-constraints.html): foreign key và các ràng buộc SQL làm nền cho oracle schema.
- `examples/toi-noi-testing/SPEC.md`, `src/infra/pglite/schema.sql`, `src/infra/fake-payment-gateway.ts`, `src/app/payment.ts`: basis và giới hạn implementation của lab.

Đối chiếu nguồn ngày 13/11/2026. Bài tiếp theo: System Test — hành trình và hành vi toàn hệ thống trong một môi trường được xác định.
