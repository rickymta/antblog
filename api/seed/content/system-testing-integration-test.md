---
slug: system-testing-integration-test
title: "Integration Test: khi các thành phần đúng riêng nhưng sai khi nối lại"
summary: "Kiểm chỗ nối API – use case – repository SQL – cổng ví của Tới Nơi: chọn phần thật và phần giả có lý do, quan sát bằng SQL và sổ ghi của ví giả, tiêm lỗi decline/timeout/mất phản hồi, rồi giới hạn đúng kết luận về retry, transaction, contract và concurrency. Lab PGlite 7 ca chạy lại được."
category: testing
level: intermediate
tags: [testing, sys-04, integration-testing, pglite, idempotency, retry, test-double, contract-testing, vitest, supertest]
language: vi
---

Hàm cước trả đúng 35.000 đồng, repository lưu được chuyến, cổng ví thu tiền thành công — mỗi thứ đều xanh khi kiểm riêng. Rồi một hôm phản hồi của ví bị mất giữa đường, ứng dụng thử lại với khoá mới, và khách bị trừ tiền hai lần. Không thành phần nào sai theo đặc tả của riêng nó; cái sai nằm ở **hợp đồng phối hợp**. Bài này trả lời: **integration test cần chạm vào ranh giới nào, phần nào nên thật và phần nào nên giả, oracle nào bắt được lỗi ở chỗ nối**, và bảy ca chạy được trên lab Tới Nơi chứng minh — cũng như *không* chứng minh — điều gì.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-04</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 1</td></tr>
<tr><th scope="row">Cấp độ</th><td>Intermediate — đã biết API test, SQL cơ bản và test double</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>Component integration (API → use case → repository → PostgreSQL WASM) và consumer side của cổng thanh toán; functional + reliability ở chỗ nối</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-functional-test">SYS-03 Functional Test</a>; nền tảng khái niệm: <a href="/articles/component-integration-testing">TEST-013 Component Integration Testing</a>, <a href="/articles/test-doubles-dummy-stub-fake-mock-spy">Test double</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): 7 execution — replay idempotency, foreign key, 5 dataset lỗi thanh toán (R-QUOTE-03, R-IDEM-02, R-PAY-03/04/06) — <code>examples/toi-noi-testing/articles/system-testing-integration-test/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>ISTQB CTFL v4.0.1 mục 2.2.1; PostgreSQL 18 Constraints; PGlite docs; <code>SPEC.md</code> của lab</td></tr>
</tbody>
</table>
</div>

## 1. Bài toán và mục tiêu

SYS-03 đã khoá các quy tắc chức năng — cước, voucher, huỷ chuyến — bằng oracle tiền số nguyên. Nhưng các ca đó chủ yếu nhìn **response HTTP**. Chúng chưa hỏi: dòng trong bảng `trips` có khớp response không? Gửi lại cùng yêu cầu có lén thêm một dòng nhật ký không? Khi ví im lặng, ứng dụng thử lại bao nhiêu lần, với khoá nào, và rốt cuộc trừ bao nhiêu tiền?

Đó là những câu hỏi chỉ trả lời được khi **nối thật** các thành phần và quan sát ở nhiều phía của ranh giới.

Bài dành cho người đã biết viết API test, đọc SQL cơ bản và phân biệt stub/fake/mock. Sau bài, bạn có thể:

1. Vẽ ranh giới tích hợp của một test và nói rõ phần nào chạy thật, phần nào được thay — kèm lý do.
2. Kích hoạt qua API rồi kiểm dữ liệu bằng **truy vấn SQL độc lập**, không chỉ round-trip qua repository.
3. Tiêm lỗi thanh toán ở đúng port (decline, timeout, lỗi tạm thời, mất phản hồi sau khi đã thu) và kiểm số lần gọi, khoá idempotency và **số tiền thực bị trừ**.
4. Viết kết luận đúng phạm vi: bảy ca xanh *không* chứng minh tính nguyên tử SQL–ví, tương thích provider thật hay xử lý tranh chấp nhiều kết nối.

## 2. Khái niệm và những chỗ dễ nhầm

**Integration testing** tập trung vào tương tác giữa các thành phần hoặc hệ thống: dữ liệu trao đổi, thứ tự gọi, mapping, xử lý lỗi và các tác dụng phụ đi kèm. CTFL v4.0.1 tách hai cấp: *component integration* (phối hợp giữa các thành phần trong một hệ thống) và *system integration* (giao tiếp giữa các hệ thống hoặc dịch vụ ngoài). Khái niệm nền đã có ở [TEST-013](/articles/component-integration-testing) và [TEST-015](/articles/system-integration-testing); bài này đi vào thực hành: port, contract, double, timeout/retry và schema.

Điều quan trọng nhất: cái tên «integration test» **không tự cho biết** dependency nào đang chạy thật. Một file tên `integration.test.ts` có thể mock toàn bộ DB. Vì vậy mỗi suite phải công bố ranh giới.

| Phạm vi | Ví dụ Tới Nơi | Bằng chứng chính |
|---|---|---|
| Component | Hàm tính tiền giảm của voucher | Giá trị trả về so với oracle |
| Component integration | Use case + repository SQL | Mapping dữ liệu, ràng buộc, kết quả ghi/đọc |
| System integration | Ứng dụng + nhà cung cấp ví sandbox | HTTP contract, credentials, retry và semantics phía provider |
| System / E2E | Hành trình khách/tài xế qua hệ thống đã triển khai | Hành vi toàn luồng với cấu hình môi trường |

Lab của bài nối **Express → use case → PostgreSQL repository → PGlite**, và nối use case thanh toán tới **fake PaymentGateway**. Nó kiểm tích hợp nội bộ và cách *consumer* dùng payment port. Nó **chưa** kiểm adapter HTTP tới nhà cung cấp thật — một fake đúng interface TypeScript vẫn có thể khác semantics của provider.

Ba cặp thuật ngữ hay bị trộn:

- **Interface vs contract.** Interface TypeScript là hình dạng lời gọi, kiểm ở compile time. Contract gồm cả ý nghĩa: đơn vị tiền, mã lỗi, khoá idempotency được tôn trọng ra sao, lỗi nào được thử lại.
- **Fake vs stub vs mock.** Stub trả lời đóng hộp; mock kiểm kỳ vọng tương tác; fake là cài đặt đơn giản *có hành vi*. `FakePaymentGateway` của lab là fake: có sổ ghi (ledger) các khoản đã trừ, dedupe theo khoá, và các chế độ lỗi.
- **Timeout vs huỷ.** Hết giờ chờ ở phía caller không có nghĩa thao tác phía provider đã dừng — đây là gốc của lỗi trừ tiền hai lần ở đầu bài.

## 3. Chiến lược ghép nối và chọn dependency

### 3.1. Thứ tự ghép

Ghép từng phần giúp khoanh vùng lỗi: kiểm repository trước, thêm use case, rồi API. Ghép tất cả cùng lúc (big-bang) bắt được vấn đề toàn luồng nhưng khó biết lỗi nằm ở serialization, schema hay nghiệp vụ. Top-down dùng tầng trên thật và thay tầng dưới bằng stub; bottom-up kiểm adapter/repository trước rồi ghép tầng trên. Đây là lựa chọn chiến lược theo rủi ro và lịch giao hàng, không phải thứ tự bắt buộc cho mọi nhóm.

Lab chọn một lát cắt dọc gần bottom-up: mọi tầng nội bộ thật, chỉ thay các hệ thống ngoài và nguồn bất định.

### 3.2. Thật hay giả — và vì sao

Dùng **SQL thật** vì repository trong bộ nhớ không thể chứng minh foreign key, kiểu `numeric`/`jsonb` hay mapping tên cột. Dùng **ví giả** vì cần chủ động tái hiện decline, timeout và «đã thu nhưng mất phản hồi» mà không phụ thuộc mạng hay giao dịch thật. Dùng **đồng hồ giả** để giá cao điểm không đổi theo giờ chạy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Ranh giới tích hợp của lab SYS-04. Bên trái là test Vitest gọi API qua supertest. Khung giữa là phần chạy thật: Express API, use case, repository PostgreSQL và PGlite. Bên phải là các port được thay bằng fake: cổng VíXanh giả có sổ ghi, đồng hồ cố định và thời tiết giả. Test quan sát ở ba nơi: response HTTP, truy vấn SQL trực tiếp và sổ ghi của ví giả.">
<defs>
<marker id="sys04-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<rect x="16" y="96" width="150" height="70" rx="8" class="f-phys-soft s-phys"/><text x="91" y="125" text-anchor="middle" class="t-b">Test Vitest</text><text x="91" y="146" text-anchor="middle" class="t-small">supertest · x-user</text>
<rect x="200" y="20" width="460" height="230" rx="10" class="f-bg s-pk dash"/>
<text x="216" y="42" class="t-b">Chạy thật trong tiến trình test</text>
<rect x="220" y="60" width="130" height="44" rx="8" class="f-pk-soft s-pk"/><text x="285" y="87" text-anchor="middle">Express API</text>
<rect x="380" y="60" width="120" height="44" rx="8" class="f-pk-soft s-pk"/><text x="440" y="87" text-anchor="middle">use case</text>
<rect x="220" y="150" width="170" height="44" rx="8" class="f-pk-soft s-pk"/><text x="305" y="177" text-anchor="middle">repository SQL</text>
<rect x="420" y="150" width="220" height="44" rx="8" class="f-pk-soft s-pk"/><text x="530" y="171" text-anchor="middle">PGlite (PostgreSQL WASM)</text><text x="530" y="187" text-anchor="middle" class="t-small">schema.sql · một kết nối</text>
<rect x="520" y="60" width="120" height="44" rx="8" class="f-pk-soft s-pk"/><text x="580" y="80" text-anchor="middle">chargeWithRetry</text><text x="580" y="96" text-anchor="middle" class="t-small">timeout · retry</text>
<line x1="166" y1="120" x2="216" y2="86" class="s-phys" stroke-width="1.6" marker-end="url(#sys04-arrow)"/>
<line x1="350" y1="82" x2="376" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#sys04-arrow)"/>
<line x1="500" y1="82" x2="516" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#sys04-arrow)"/>
<line x1="430" y1="104" x2="320" y2="146" class="s-pk" stroke-width="1.6" marker-end="url(#sys04-arrow)"/>
<line x1="390" y1="172" x2="416" y2="172" class="s-pk" stroke-width="1.6" marker-end="url(#sys04-arrow)"/>
<text x="790" y="28" text-anchor="middle" class="t-small t-warn">port được thay bằng fake</text>
<rect x="700" y="40" width="180" height="56" rx="8" class="f-warn-soft s-warn"/><text x="790" y="64" text-anchor="middle">VíXanh giả</text><text x="790" y="83" text-anchor="middle" class="t-small">ledger · chế độ lỗi</text>
<rect x="700" y="116" width="180" height="40" rx="8" class="f-warn-soft s-warn"/><text x="790" y="141" text-anchor="middle">FixedClock 09:00 VN</text>
<rect x="700" y="172" width="180" height="40" rx="8" class="f-warn-soft s-warn"/><text x="790" y="197" text-anchor="middle">thời tiết: không mưa</text>
<line x1="640" y1="72" x2="696" y2="68" class="s-warn dash" stroke-width="1.4" marker-end="url(#sys04-arrow)"/>
<text x="91" y="200" text-anchor="middle" class="t-small t-phys">quan sát ở 3 phía:</text>
<text x="91" y="218" text-anchor="middle" class="t-small">① response HTTP</text>
<text x="91" y="236" text-anchor="middle" class="t-small">② SELECT trực tiếp</text>
<text x="91" y="254" text-anchor="middle" class="t-small">③ calls + ledger ví</text>
<text x="460" y="285" text-anchor="middle" class="t-small">chưa nằm trong ranh giới: adapter HTTP tới provider thật, DNS/TLS, PostgreSQL server nhiều kết nối, broker</text>
</svg>
</div>
<figcaption><b>Hình 1 · Ranh giới tích hợp của SYS-04.</b> Mọi tầng nội bộ chạy thật trên PostgreSQL WASM; ví, đồng hồ, thời tiết là fake có kiểm soát. Test không tin một nguồn duy nhất: response, SQL và sổ ghi ví phải cùng kể một câu chuyện.</figcaption>
</figure>

| Thành phần | Cách dùng trong lab | Phạm vi chưa chứng minh |
|---|---|---|
| Express + use cases | Mã thật, gọi qua supertest | DNS, TLS, reverse proxy, deployment |
| Repository + schema | Mã thật, PGlite PostgreSQL WASM | Server PostgreSQL nhiều kết nối, network driver |
| PaymentGateway | Fake có ledger và fault modes | Provider HTTP schema, chữ ký, retry phía provider |
| Clock / weather | Fake cố định | Dịch vụ thời tiết, clock skew giữa máy |

Đánh đổi nằm giữa độ trung thực, tốc độ, khả năng tái hiện và chi phí vận hành. Đừng dùng «tỷ lệ dependency thật» làm thước đo duy nhất: một fake đúng ở port, tái hiện được *lost response* theo ý muốn, có ích hơn một sandbox không bao giờ cho bạn tạo ra lỗi đó — miễn kết luận không vượt quá bằng chứng.

## 4. Test basis, hợp đồng và oracle xuyên ranh giới

### 4.1. Yêu cầu dùng làm test basis

Các mã dưới đây có thật trong `examples/toi-noi-testing/SPEC.md`:

- **R-QUOTE-03** — mỗi báo giá đặt được tối đa một chuyến.
- **R-IDEM-02** — cùng phạm vi + cùng khoá + cùng nội dung ⇒ trả lại đúng mã HTTP và thân của lần đầu, kèm header `Idempotent-Replayed: true`; không tạo chuyến mới.
- **R-IDEM-03** — cùng khoá nhưng khác nội dung ⇒ 422 `IDEMPOTENCY_KEY_REUSED` (dùng trong bài tập).
- **R-PAY-03** — chuyến `vixanh` hoàn thành ⇒ thu đúng `fare` với khoá `trip:<id>:fare`.
- **R-PAY-04** — mỗi lần gọi hết giờ sau 3 giây; hết giờ/lỗi tạm thời ⇒ thử lại tối đa 2 lần (tổng 3 lần gọi) với **cùng khoá**; `declined` ⇒ không thử lại.
- **R-PAY-06** — thu tiền thất bại không chặn nghiệp vụ: chuyến vẫn `completed`, `paymentStatus = failed`, API trả 200.

Ngoài ra, foreign key `trip_events.trip_id → trips.id` trong `src/infra/pglite/schema.sql` là **hợp đồng schema**, không phải một yêu cầu nghiệp vụ; bài không tự gán cho nó mã R-… mới.

### 4.2. Nhiều lớp oracle

Một ranh giới có nhiều phía, nên cần nhiều oracle:

| Phía quan sát | Câu hỏi | Lỗi chỉ phía này bắt được |
|---|---|---|
| Response HTTP | Client thấy gì? | Mã lỗi sai, thiếu header replay |
| SQL trực tiếp | Dữ liệu bền thực sự là gì? | Response nói `paid` nhưng row chưa cập nhật |
| Nhật ký `trip_events` | Có tác dụng phụ thừa không? | Replay lén thêm event |
| Trạng thái tài xế | Tài nguyên đã được trả lại? | D101 kẹt ở `busy` sau lỗi ví (R-ACCEPT-05) |
| `payments.calls` | Gọi mấy lần, khoá nào, bao nhiêu tiền? | Retry đổi khoá, retry cả decline |
| Ledger của fake | Thực sự trừ bao nhiêu? | Hai lần gọi nhưng có trừ hai lần không |

Chỉ kiểm HTTP 200 sẽ bỏ sót `payment failed`. Chỉ kiểm response `paid` sẽ bỏ sót DB chưa cập nhật. Đọc qua chính repository để kiểm repository cũng không đủ: nếu adapter ghi sai và đọc sai *theo cùng một cách*, round-trip vẫn xanh. Vì vậy lab dùng `system.db.query(...)` — SQL độc lập với mapping của adapter.

### 4.3. Oracle cho lỗi schema

Insert một event trỏ tới chuyến không tồn tại phải bị PostgreSQL từ chối với SQLSTATE **23503** (`foreign_key_violation`). Test assert *mã* lỗi và việc không có row nào được ghi, không assert toàn văn thông báo — câu chữ có thể đổi theo ngôn ngữ hoặc phiên bản.

### 4.4. Tiêu chí bắt đầu và kết thúc

- **Entry:** SPEC và schema xác định; app dựng được; dữ liệu mỗi ca độc lập (reset trước từng execution); dependency cài theo lockfile.
- **Exit:** 7/7 execution pass, không skip; `npm run typecheck` exit 0; ranh giới thật/giả được công bố trong báo cáo.

## 5. Thiết kế ca kiểm

### 5.1. Tiền điều kiện chung

Seed khách C001 và tài xế D101; xe máy (`bike`) tại SG, 5 km; đồng hồ cố định 09:00 giờ VN ngày 04/11/2026; không mưa, không voucher ⇒ `fare = 35.000`. Mỗi execution reset DB và fake ví, dựng Express app mới; suite đóng PGlite ở cuối. Các ca chạy **tuần tự** trong file vì cùng dùng một DB — reset giữa chừng sẽ phá ca khác.

### 5.2. Bảng case

| ID / dataset | Requirement / risk | Hành động | Expected | Cleanup |
|---|---|---|---|---|
| IN-01 | R-QUOTE-03, R-IDEM-02 — replay tạo chuyến trùng | Báo giá; POST `/api/trips` với key `sys04-book-001`; gửi lại y hệt | Cả hai 201, body giống nhau, `idempotent-replayed: true`; SQL: 1 trip `pending`; quote trỏ đúng trip; events chỉ `requested`, `searching` | reset trước ca kế |
| IN-02 | FK schema — event mồ côi | INSERT `trip_events` với `trip_id='missing-trip'` | Reject code `23503`; bảng `trip_events` có 0 row | reset |
| IN-03 / none | R-PAY-03 — đường thu bình thường | Đưa chuyến tới `in_transit` qua API; complete | `completed/paid`; 1 call; ledger 35.000 | reset |
| IN-03 / lost_response | R-PAY-04 — đã thu nhưng mất phản hồi lần đầu | `failNext('lost_response')`; complete | 2 call cùng key; `paid`; ledger **chỉ** 35.000 | reset |
| IN-03 / decline | R-PAY-04/06 — retry nhầm lỗi nghiệp vụ | `setMode('decline')`; complete | 1 call; `failed`; thu 0; vẫn `completed`, HTTP 200 | reset |
| IN-03 / timeout | R-PAY-04/06 — hết lượt thử | `setMode('timeout')` cho mọi lần | 3 call cùng key; `failed`; thu 0; `completed` | reset |
| IN-03 / error | R-PAY-04/06 — lỗi tạm thời kéo dài | `setMode('error')` | 3 call cùng key; `failed`; thu 0; `completed` | reset |

Mọi dòng IN-03 còn kiểm: SQL `payment_status` khớp response, D101 trở lại `online`, và **đúng một** event `completed`.

Hai lần gọi không đồng nghĩa hai lần thu — phải nhìn ledger. Ngược lại, không thấy response không có nghĩa provider chưa thu: dataset `lost_response` tồn tại chính để nhắc điều đó.

### 5.3. Vì sao chọn `setMode` hay `failNext`

`failNext('lost_response')` chỉ làm hỏng **lần gọi đầu**; lần sau ví trả lời bình thường — đúng kịch bản «retry cứu được». `setMode('timeout')` làm hỏng **mọi lần**, nên chứng minh được *giới hạn* retry: nếu chỉ `failNext` một timeout, bạn mới thấy retry hoạt động chứ chưa thấy nó dừng đúng sau 3 lần.

## 6. Thực hành có thể chạy lại

Mã đầy đủ: [`articles/system-testing-integration-test/integration.test.ts`](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-integration-test). Dùng Node ≥ 22.12, tại `examples/toi-noi-testing`:

```sh
npm ci
npx vitest run articles/system-testing-integration-test
npm run typecheck
```

`npm ci` chỉ cần khi chuẩn bị dependency theo lockfile. Không cần PostgreSQL server: PGlite chạy PostgreSQL biên dịch sang WASM ngay trong tiến trình test. Expected: **7 tests pass** — hai ca riêng và năm dataset thanh toán.

### 6.1. Bước 1 — Lắp ráp và kiểm soát vòng đời

```ts
beforeAll(async () => {
  system = await createPgliteSystem({
    clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)),
    config: { payment: { timeoutMs: 50, maxRetries: 2 } },
  });
});
beforeEach(async () => {
  await system.reset();
  app = createApp(system.deps, { logErrors: false });
});
afterAll(async () => { await system?.db?.close(); });
```

`createPgliteSystem` tạo schema và seed; `beforeEach` reset rồi dựng app mới để trạng thái nằm trong process (bộ nhớ idempotency, fake ví) không rò giữa các ca. Đừng dùng `test.concurrent` với setup này. Muốn chạy song song, mỗi worker cần DB riêng hoặc namespace dữ liệu và cleanup thực sự cách ly.

Lưu ý `timeoutMs: 50`: lab rút deadline từ 3 giây (mặc định của SPEC) xuống 50 ms để kiểm **logic** đếm lần thử nhanh, giữ `maxRetries: 2`. Kết quả không chứng minh ngưỡng 3 giây hay bất kỳ SLA latency nào.

### 6.2. Bước 2 — Kích hoạt qua API, quan sát bằng SQL

IN-01 không insert trip trực tiếp; nó gọi báo giá và đặt chuyến thật, rồi truy vấn bảng:

```ts
const tripId = first.body.trip.id;
const trips = await system.db!.query('SELECT id, fare, payment_status FROM trips WHERE quote_id=$1', [qid]);
expect(trips.rows).toEqual([{ id: tripId, fare: 35_000, payment_status: 'pending' }]);
const quote = await system.db!.query('SELECT used_by_trip_id FROM quotes WHERE id=$1', [qid]);
expect(quote.rows).toEqual([{ used_by_trip_id: tripId }]);
const events = await system.db!.query('SELECT status FROM trip_events WHERE trip_id=$1 ORDER BY id', [tripId]);
expect(events.rows).toEqual([{ status: 'requested' }, { status: 'searching' }]);
```

Ba chi tiết đáng chú ý:

- **Query có tham số** (`$1`) thay vì ghép chuỗi — đúng cả về an toàn lẫn kiểu dữ liệu.
- `toEqual` với **mảng đầy đủ** chứ không `toContainEqual`: một row thừa phải làm test đỏ, vì «thừa» chính là lỗi replay đang săn.
- Event sắp theo `ORDER BY id` (cột identity) chứ không theo `occurred_at`: với đồng hồ cố định, hai event có thể cùng timestamp, khiến thứ tự theo thời gian không xác định.

### 6.3. Bước 3 — Tạo lỗi ở đúng ranh giới

```ts
async function inTransit() {
  const created = await book(await quoteId()).expect(201);
  const id: string = created.body.trip.id;
  await request(app).post(`/api/trips/${id}/accept`).set('x-user', 'driver:D101').send({ etaMinutes: 5 }).expect(200);
  for (const step of ['arrive', 'pickup', 'start']) {
    await request(app).post(`/api/trips/${id}/${step}`).set('x-user', 'driver:D101').send({}).expect(200);
  }
  return id;
}
```

Chuyến được đưa tới trạng thái đang chạy **qua API** rồi mới cấu hình lỗi ví — lỗi được tiêm đúng ở port thanh toán, không ở chỗ khác. Trong fake, `timeout` và `lost_response` đều trả về một promise không bao giờ xong; khác biệt là `lost_response` **ghi khoản thu vào ledger trước** khi im lặng:

```ts
const record = this.applyCharge(request); // idempotent theo khoá
if (failure === 'lost_response') return never();
return { status: 'succeeded', chargeId: record.chargeId };
```

Phía ứng dụng, `chargeWithRetry` bọc mỗi lần gọi bằng `withTimeout` (dựa trên `Promise.race`) và vòng lặp tối đa `1 + maxRetries` lần với cùng `request` — tức cùng khoá. Test không cần `sleep`: deadline của ứng dụng quyết định khi nào đi tiếp.

### 6.4. Bước 4 — Kiểm kết quả ở mọi phía

```ts
const res = await complete(id).expect(200);
expect(res.body.trip).toMatchObject({ status: 'completed', paymentStatus: status });
const stored = await system.db!.query('SELECT status, payment_status FROM trips WHERE id=$1', [id]);
expect(stored.rows).toEqual([{ status: 'completed', payment_status: status }]);
const driver = await system.db!.query("SELECT status FROM drivers WHERE id='D101'");
expect(driver.rows).toEqual([{ status: 'online' }]);
expect(system.payments.calls).toHaveLength(attempts);
for (const call of system.payments.calls) {
  expect(call).toMatchObject({ idempotencyKey: `trip:${id}:fare`, customerId: 'C001', amount: 35_000 });
}
expect(system.payments.totalCharged()).toBe(charged);
expect(system.payments.chargedKeys()).toEqual(charged ? [`trip:${id}:fare`] : []);
```

Khoá được dựng từ `id` của chính ca đó, không hardcode. Vòng `for` kiểm **từng** lần gọi — nếu lần retry thứ hai đổi khoá hay đổi số tiền, test đỏ. Decline/timeout/error vẫn kỳ vọng HTTP 200 theo R-PAY-06: hoàn tất chuyến và thu tiền là hai kết quả khác nhau. Test đòi 500 cho mọi lỗi ví sẽ áp đặt một hợp đồng sai.

### 6.5. Kết quả: mong đợi và thực tế

**Mong đợi** (từ thiết kế ở mục 5): 7 pass; số lần gọi theo dataset none/lost_response/decline/timeout/error là 1/2/1/3/3; ledger 35.000/35.000/0/0/0.

**Thực tế — lượt chạy lại ngày 13/11/2026** (Windows, Node v22.18.0, Vitest 3.2.7, lệnh `npx vitest run articles/system-testing-integration-test`):

```text
✓ articles/system-testing-integration-test/integration.test.ts (7 tests) 3750ms
  ✓ ... IN-03 · R-PAY-03/04/06: 'timeout' → 'failed', 3 lần gọi 356ms
Test Files  1 passed (1)
     Tests  7 passed (7)
  Duration  10.64s (transform 294ms, collect 5.69s, tests 3.75s, prepare 687ms)
```

Lượt chạy ghi trong báo cáo bài (cùng ngày, Node v24.15.0, chạy chung với suite SYS-01…03) cũng cho 7/7 pass, 2.628 ms cho file này. Thời gian khác nhau giữa hai lượt là bình thường — máy đang chạy việc khác, và đây **không** phải phép đo hiệu năng. Dataset `timeout` chậm nhất vì phải chờ ba deadline 50 ms cộng xử lý.

Đọc kết quả: replay trả lại response cũ, SQL có một chuyến và hai event khởi tạo; FK chặn event mồ côi với 23503; `lost_response` gọi hai lần nhưng chỉ trừ một lần 35.000; các lỗi kéo dài cho `completed/failed` và tài xế `online`. Không phát hiện defect sản phẩm trong lượt chạy.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn bằng chứng</span>
<p>PGlite là PostgreSQL <b>một kết nối</b> trong tiến trình — không thay một server có nhiều connection, nên tranh chấp và isolation chưa được kiểm. Ví, đồng hồ, thời tiết là fake: «lost_response thu một lần» chứng minh ứng dụng giữ nguyên khoá khi retry, <b>không</b> chứng minh provider thật dedupe theo khoá. Timeout 50 ms là cấu hình thử, không phải ngưỡng 3 giây của SPEC.</p>
</div>

## 7. Đọc thất bại và ra quyết định

| Dấu hiệu | Giả thuyết cần kiểm | Bằng chứng tiếp theo |
|---|---|---|
| Response đúng, SQL sai | Mapping/update chưa lưu | Row theo tripId và câu SQL của adapter |
| Replay tạo thêm event | Idempotency không bao toàn bộ handler | Phạm vi khoá, response đã lưu, số row trước/sau |
| lost_response thu hai lần | Khoá đổi giữa các lần thử, hoặc fake/provider không dedupe | Danh sách `calls` và ledger |
| decline có ba lần gọi | Gộp lỗi nghiệp vụ với lỗi tạm thời | Nhánh xử lý `ChargeResult` |
| IN-02 không lỗi | Schema/constraint chưa được nạp vào DB test | Kiểm schema thực của DB test |
| Test treo | Deadline hoặc cleanup không hoạt động | Timer, promise đang chờ; không chỉ tăng timeout của runner |

Khi ca đỏ, ghi: commit, ca/dataset, cấu hình timeout và retry, response, các row SQL và danh sách `calls` đã quan sát. Phân biệt **failed** (lỗi sản phẩm, có expected/actual và dữ liệu tái hiện) với **blocked** (môi trường: PGlite không khởi tạo, thiếu dependency). Khi mở rộng sang sandbox, không log token hay dữ liệu thanh toán thật.

Ví dụ quyết định: «IN-03/lost_response đỏ: `chargedKeys` có hai khoá khác nhau. Chặn merge thay đổi ở `payment.ts`; các suite không chạm thanh toán vẫn review được. Sau khi sửa, chạy confirmation dataset này, rồi cả bảy ca.»

## 8. Mở rộng nâng cao

### 8.1. Timeout không huỷ tác dụng phụ

`Promise.race` chỉ giới hạn thời gian **chờ** của caller, không huỷ request đang xử lý ở provider. `lost_response` minh hoạ việc tiền đã trừ dù caller hết giờ. Trong triển khai thật cần idempotency do provider bảo đảm, đối soát trạng thái giao dịch và cơ chế nhận kết quả muộn. Đừng suy ra «`failed` trong DB» luôn nghĩa là «chưa thu tiền»: nếu provider đã thu nhưng *mọi* phản hồi đều mất, hệ thống thật cần trạng thái «không xác định» và reconciliation; SPEC của lab giản lược thành `failed`.

### 8.2. Transaction cục bộ và giao dịch ngoài hệ thống

API → SQL → ví không nằm chung một transaction ACID. Có một cửa sổ: ví đã thu nhưng cập nhật `payment_status` thất bại. Suite hiện tại **chưa** tiêm lỗi ở điểm này. Thí nghiệm đề xuất: ép lệnh ghi `paymentStatus` lỗi sau khi charge thành công, quan sát ledger và DB lệch nhau, rồi thiết kế reconciliation hoặc outbox (xem [PostgreSQL: idempotency và outbox](/articles/postgresql-idempotency-outbox-transaction)). Không «sửa» bằng rollback SQL rồi giả định tiền ở hệ thống ngoài cũng tự rollback.

### 8.3. Contract testing và provider verification

Interface TypeScript kiểm hình dạng lời gọi nội bộ, không kiểm payload HTTP, mã lỗi provider, đơn vị tiền hay chữ ký. Một adapter gửi `35000` theo đồng trong khi provider hiểu là đơn vị nhỏ hơn sẽ làm mọi test ở port xanh mà tích hợp thật sai. Khi có adapter ngoài: lưu contract request/response, kiểm serialization bằng stub HTTP, và xác minh provider trên sandbox với version cụ thể. Fake cần được đối chiếu với contract định kỳ để tránh **drift** — fake dần khác provider mà không ai biết.

### 8.4. Schema evolution và nhiều phiên bản ứng dụng

DB tạo sạch từ `schema.sql` không chứng minh migration từ bản cũ đúng. Cần dựng schema cũ với dữ liệu đại diện, chạy migration, rồi kiểm app cũ và mới cùng đọc/ghi trong khoảng rolling deploy: nullability, default, backfill, đọc/ghi hai chiều. *Expand → migrate → contract* là một hướng: thêm trường tương thích, chuyển dữ liệu và consumer, sau đó mới bỏ trường cũ — mỗi bước có tiêu chí và test riêng.

### 8.5. Concurrency và ranh giới của PGlite

`Promise.all` trên PGlite không biến nó thành server nhiều connection. Kiểm hai tài xế nhận cùng chuyến (hay hai request đặt cùng quote, điều R-QUOTE-03 nói «kể cả khi hai yêu cầu tới cùng lúc») cần hai transaction độc lập, điểm đồng bộ khởi chạy, isolation level rõ ràng và invariant «chỉ một bên thắng». IN-01 replay **tuần tự**, không chứng minh xử lý hai request đang in-progress. Phần này cần PostgreSQL riêng và chưa chạy ở bài này.

### 8.6. Broker, outbox và quan sát

Nếu sau này hệ thống phát event `TripCompleted`, cần kiểm payload/version, correlation ID, retry, duplicate delivery và consumer idempotency. Oracle nên là trạng thái nghiệp vụ cuối cùng cộng số tác dụng phụ, không chỉ «message đã publish». Chờ có điều kiện với deadline thay cho `sleep` cố định. Lab chưa có broker hay outbox — đây là hướng thiết kế, không phải chức năng đang được kiểm.

## 9. Anti-pattern, giới hạn và đánh đổi

- **Mock repository rồi gọi là đã kiểm SQL** — schema, kiểu dữ liệu và ràng buộc bị bỏ qua hoàn toàn.
- **Assert toàn văn lỗi DB** — giòn theo ngôn ngữ, phiên bản; assert SQLSTATE.
- **Reset dữ liệu dùng chung khi test chạy song song** — nguồn flakiness kinh điển.
- **Retry cả decline** — sai hợp đồng, có thể gây nhiễu hoặc bị provider chặn.
- **Đếm calls mà không kiểm ledger** — bỏ sót thu trùng; ngược lại, kiểm ledger mà không đếm calls bỏ sót retry vô hạn.
- **Tăng timeout của runner để che treo** — phản hồi chậm hơn mà nguyên nhân vẫn còn.

Suite chưa kiểm: toàn bộ ràng buộc schema, durability sau restart, rollback khi lỗi giữa luồng, provider thật, JWT (header `x-user` là danh tính giả lập), browser, mạng, concurrency nhiều kết nối, migration. Khác biệt cốt lõi với SYS-03 không phải số ca nhiều hơn, mà là **oracle SQL và các adapter thật được phối hợp**.

Về chi phí: bảy ca chạy vài giây, đủ rẻ để nằm trong pipeline mỗi commit. Thêm PostgreSQL server thật cho concurrency, hay sandbox provider, tăng độ trung thực nhưng kéo theo hạ tầng, dữ liệu cô lập và thời gian — nên đặt ở suite riêng, chạy theo rủi ro thay đổi.

## 10. Bài tập và câu hỏi tự kiểm

### Bài tập

1. **Cơ bản.** Thêm dataset: một lần `error` rồi thành công.
2. **Trung bình.** Gửi lại POST tạo chuyến với cùng khoá nhưng `quoteId` khác.
3. **Trung bình.** Đặt chuyến `cash` rồi hoàn tất.
4. **Nâng cao.** Ép cập nhật SQL thất bại *sau* khi ví thu thành công. Mô tả trạng thái hệ thống và cách xử lý.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><b>1.</b> Dùng <code>system.payments.failNext('error')</code> (không phải <code>setMode</code>, để lỗi chỉ áp cho lần đầu). Expected: 2 calls cùng khoá <code>trip:&lt;id&gt;:fare</code>, <code>completed/paid</code> ở cả response và SQL, ledger 35.000.</p>
<p><b>2.</b> Theo R-IDEM-03: 422 <code>IDEMPOTENCY_KEY_REUSED</code>. Kiểm thêm bằng SQL: số row <code>trips</code> không tăng và quote mới có <code>used_by_trip_id</code> rỗng. Đây là ca khác hẳn replay cùng body của IN-01.</p>
<p><b>3.</b> Theo R-PAY-02: <code>paymentStatus = cash</code>, <code>payments.calls</code> rỗng, ledger 0, DB <code>completed</code>, D101 <code>online</code>. Không áp oracle <code>paid</code> của chuyến <code>vixanh</code>.</p>
<p><b>4.</b> Có cửa sổ lệch: ledger đã trừ 35.000, DB vẫn chưa ghi <code>paid</code> (hoặc request lỗi). Đáp án tốt nêu: số tiền thực đã thu, cách phát hiện (đối soát ledger/provider với DB), cách sửa (retry ghi DB, reconciliation job, outbox) — và <b>không</b> retry toàn luồng với khoá mới, vì sẽ thu lần hai.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** Dataset `lost_response` có 2 lần gọi ví. Vì sao ledger vẫn chỉ 35.000?

- a) Lần gọi thứ hai bị test bỏ qua
- b) Fake dedupe theo khoá idempotency, và ứng dụng giữ nguyên khoá khi retry
- c) Ứng dụng huỷ lần gọi đầu khi hết giờ
- d) PGlite rollback khoản thu đầu

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b>. <code>applyCharge</code> trả lại bản ghi cũ nếu khoá đã tồn tại. c) sai: <code>Promise.race</code> không huỷ thao tác phía ví. d) sai: ledger nằm ngoài DB. Lưu ý: điều này chứng minh hành vi của ứng dụng với fake, không chứng minh provider thật cũng dedupe.</p>
</details>

**Câu 2.** Vì sao IN-01 kiểm bằng <code>system.db.query</code> thay vì đọc lại qua API?

- a) API không có endpoint đọc chuyến
- b) Để oracle độc lập với mapping của adapter và thấy được row/event thừa mà API không lộ ra
- c) SQL nhanh hơn HTTP
- d) Để tránh phải gửi header <code>x-user</code>

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b>. Nếu adapter ghi sai và đọc sai cùng cách, round-trip qua API vẫn xanh. Bảng <code>trip_events</code> cũng không được API trả nguyên văn, nên event thừa do replay chỉ thấy qua SQL.</p>
</details>

**Câu 3.** Ví từ chối giao dịch khi hoàn tất chuyến. Hợp đồng đúng theo SPEC là gì?

- a) HTTP 500, chuyến giữ <code>in_transit</code>
- b) HTTP 402, chuyến <code>completed</code>
- c) HTTP 200, chuyến <code>completed</code>, <code>paymentStatus = failed</code>, 1 lần gọi
- d) HTTP 200, chuyến <code>completed</code>, <code>failed</code>, 3 lần gọi

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b> theo R-PAY-04 (decline không retry) và R-PAY-06 (thanh toán không chặn nghiệp vụ). d) là hành vi của timeout/error kéo dài.</p>
</details>

**Câu 4.** Bảy ca pass có chứng minh hai yêu cầu đặt cùng một quote đến *đồng thời* chỉ tạo một chuyến không?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>Không.</b> IN-01 gửi tuần tự, trên PGlite một kết nối. Yêu cầu «kể cả khi hai yêu cầu tới cùng lúc» của R-QUOTE-03 cần PostgreSQL nhiều kết nối, hai transaction thật và điểm đồng bộ — xem mục 8.5. Ràng buộc <code>UNIQUE</code> trên <code>trips.quote_id</code> trong schema là một lớp bảo vệ, nhưng cách ứng dụng phản ứng với vi phạm đó vẫn cần được kiểm.</p>
</details>

**Câu 5.** Vì sao lab đặt <code>timeoutMs: 50</code> trong khi SPEC nói 3 giây, và kết luận nào bị giới hạn bởi lựa chọn này?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Để kiểm logic đếm lần thử và giữ khoá trong thời gian ngắn (ba deadline 3 giây sẽ làm một dataset mất ~9 giây). Hệ quả: bài không chứng minh giá trị cấu hình mặc định là 3 giây, cũng không nói gì về latency thật của provider. Muốn khoá giá trị mặc định, thêm một kiểm tra cấu hình riêng hoặc dùng fake timer.</p>
</details>

## 11. Tài liệu tham khảo

Tra cứu và đối chiếu ngày 13/11/2026.

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — mục 2.2.1: component integration và system integration testing. https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- PostgreSQL 18 — *Constraints* (foreign key, unique): https://www.postgresql.org/docs/18/ddl-constraints.html
- PostgreSQL 18 — *Appendix A. Error Codes* (SQLSTATE 23503 `foreign_key_violation`): https://www.postgresql.org/docs/18/errcodes-appendix.html
- PGlite — *Getting Started*: PostgreSQL WASM trong tiến trình, query có tham số, in-memory vs filesystem. https://pglite.dev/docs/
- Lab Tới Nơi: `SPEC.md` (R-QUOTE-03, R-IDEM-02/03, R-PAY-02/03/04/06), `src/infra/pglite/schema.sql`, `src/infra/fake-payment-gateway.ts`, `src/app/payment.ts` và bộ test của bài — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [SYS-01 Smoke Test](/articles/system-testing-smoke-test) · [SYS-03 Functional Test](/articles/system-testing-functional-test) · [TEST-013 Component Integration Testing](/articles/component-integration-testing) · [TEST-015 System Integration Testing](/articles/system-integration-testing) · [Test double: dummy, stub, fake, mock, spy](/articles/test-doubles-dummy-stub-fake-mock-spy)

Bài tiếp theo: [SYS-05 · System Test](/articles/system-testing-system-test).
