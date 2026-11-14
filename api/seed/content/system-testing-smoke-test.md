---
slug: system-testing-smoke-test
title: "Smoke Test: bản build có đủ tốt để bắt đầu kiểm thử chuyên sâu?"
summary: "Smoke test là bộ kiểm nhỏ trả lời một câu hỏi duy nhất: build này có đáng để đội kiểm thử chuyên sâu tiếp không. Chọn ba case từ rủi ro cho Tới Nơi — health, chặn truy cập vô danh, đặt → hoàn tất → thu tiền — chạy qua API và PGlite, đọc thất bại và viết quyết định có bằng chứng."
category: testing
level: beginner
tags: [testing, sys-01, smoke-test, build-verification, release-gate, api-testing, pglite, vitest, test-oracle]
language: vi
---

Thứ Hai, Tới Nơi vừa đóng gói bản build mới. Trang chủ vẫn mở, `/api/health` vẫn trả 200 — nhưng câu lệnh ghi chuyến không còn khớp schema. Nếu đội kiểm thử lao ngay vào hàng trăm ca voucher, họ sẽ ghi nhận đi ghi nhận lại cùng một lỗi nền dưới hàng chục cái tên khác nhau, và mất nửa ngày mới hiểu ra. Bài này trả lời: **làm sao chọn vài phép kiểm đại diện để quyết định trong vài giây rằng build đủ tốt để kiểm tiếp, hay phải trả về sửa** — và vì sao "health xanh" chưa bao giờ đủ cho câu hỏi đó.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-01</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 1</td></tr>
<tr><th scope="row">Cấp độ</th><td>Beginner — tester mới biết HTTP và TypeScript cơ bản; mục 6 dành cho QA/SDET thiết kế release gate</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>Smoke là cách chọn phạm vi suite, không phải một test level; ở bài này các case chạy qua API với kho dữ liệu thật của lab (PGlite)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a> · <a href="/articles/tong-quan-5-test-level-istqb">TEST-011 Tổng quan 5 test level</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): 3 smoke case SM-01…03 — health, chặn danh tính thiếu, đặt → nhận → hoàn tất → đọc lại → kiểm thu tiền (R-API-15, R-API-03, R-STATE-02, R-PAY-03) — <code>examples/toi-noi-testing/articles/system-testing-smoke-test/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>ISTQB CTFL v4.0.1 (thuật ngữ test level/type, confirmation/regression); Grafana k6 Load test types (đối chiếu nghĩa "smoke" trong hiệu năng); <code>SPEC.md</code> của lab</td></tr>
</tbody>
</table>
</div>

## 1. Bài toán và mục tiêu

### 1.1. Tình huống

Tới Nơi là ứng dụng gọi xe, giao bưu kiện và đồ ăn. Lab của series là một *modular monolith*: web, API Express, use case, domain tính cước và trạng thái chuyến, kho dữ liệu PostgreSQL chạy dưới dạng PGlite. Ví điện tử VíXanh, dịch vụ thời tiết và đồng hồ được thay bằng bản giả có thể điều khiển.

Mỗi lần có build mới, đội có hai lựa chọn tốn kém nếu chọn sai:

- **Kiểm thử chuyên sâu ngay**: nếu build hỏng ở tầng nền (không ghi được chuyến, không thu được tiền), mọi ca chức năng phía sau đều đỏ với cùng một nguyên nhân. Thời gian điều tra bị nhân lên, báo cáo lỗi trùng lặp.
- **Trả build về ngay**: nếu không có bằng chứng cụ thể, developer nhận một câu "build hỏng" không tái hiện được.

Smoke test là bước nhỏ ở giữa: vài phép kiểm chọn lọc, chạy nhanh, cho ra quyết định *tiếp tục* hoặc *dừng* kèm bằng chứng tái hiện được.

### 1.2. Đối tượng và điều kiện tiên quyết

Bạn cần đọc được request/response HTTP (method, status code, JSON body) và TypeScript ở mức `async/await`. Không cần biết Vitest hay Supertest trước; bài giải thích những gì cần dùng. Máy cần Node ≥ 22.12; không cần cài PostgreSQL server.

### 1.3. Mục tiêu đo được

Sau bài này, bạn có thể:

1. Phân biệt health/liveness, readiness, smoke, sanity, confirmation và regression bằng câu hỏi mỗi loại trả lời.
2. Chọn smoke case từ rủi ro và test basis, ghi rõ ca nào bị loại và sẽ được kiểm ở đâu.
3. Viết một hành trình đi xuyên API và kho dữ liệu với oracle độc lập (không dùng code đang kiểm để tính expected).
4. Phân loại một thất bại là *blocked* (môi trường) hay *failed* (sản phẩm), và viết quyết định có phạm vi kết luận rõ.

## 2. Khái niệm và những điều dễ nhầm

### 2.1. Smoke test là gì

Trong series này, **smoke test** là một bộ kiểm tra nhỏ nhắm vào các khả năng thiết yếu của một build hoặc một môi trường. Nó thường chạy ngay sau build hoặc deploy, trước vòng kiểm thử sâu. Giá trị chính không nằm ở việc tìm nhiều lỗi, mà ở việc **phát hiện sớm những lỗi khiến phần còn lại của vòng kiểm thử trở nên vô nghĩa**.

Ba hệ quả của định nghĩa này:

- **Smoke không phải một test level.** Theo CTFL v4.0.1, test level là component, component integration, system, system integration và acceptance. Smoke là cách *chọn phạm vi* một suite; các case của nó có thể nằm ở cấp API, UI hay tích hợp. Ở bài này, case chạy qua API và chạm kho dữ liệu thật của lab.
- **Không bắt buộc chỉ có happy path.** Một hành trình "đặt xe thành công" là xương sống, nhưng cơ chế chặn truy cập vô danh có thể quan trọng đủ để nằm trong smoke: nếu nó hỏng, mọi kết quả phân quyền phía sau đều không đáng tin.
- **Smoke xanh cho phép kiểm tiếp, không cho phép release.** Đây là câu dễ quên nhất.

### 2.2. Bảng phân biệt

| Khái niệm | Câu hỏi trả lời | Ví dụ Tới Nơi |
|---|---|---|
| Health / liveness | Tiến trình có phản hồi không? | `GET /api/health` trả 200 |
| Readiness | Instance có đủ điều kiện nhận traffic không? | Kiểm dependency thiết yếu (DB, cấu hình) theo thiết kế |
| Smoke | Các khả năng thiết yếu có dùng được không? | Báo giá, đặt, nhận, hoàn tất, lưu và thu tiền |
| Sanity | Thay đổi cụ thể có hợp lý để kiểm tiếp không? | Sau sửa voucher, kiểm lỗi gốc và vùng liên quan |
| Confirmation | Lỗi đã sửa có còn tái hiện không? | Chạy lại đúng ca từng báo defect |
| Regression | Thay đổi có làm hỏng phần đang hoạt động không? | Voucher sửa xong nhưng tổng thanh toán bị sai |

Lưu ý dòng health: trong lab, route `/api/health` chỉ trả `{ "status": "ok" }` mà **không chạm vào cơ sở dữ liệu** (đúng R-API-15, dùng cho giám sát và để công cụ kiểm thử chờ máy chủ). Nghĩa là health xanh chỉ chứng minh tiến trình đang phản hồi HTTP — đúng tình huống mở bài: health xanh, ghi chuyến hỏng.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Tên gọi không thống nhất giữa các nhóm</span>
<p>Có đội gọi smoke là "build verification test", có đội dùng "sanity" cho cùng mục đích. Đừng tranh luận theo tên: ghi <b>mục tiêu và phạm vi</b> của suite trong test plan. Ngoài ra, "smoke test" trong tài liệu k6 là bài chạy tải rất thấp để kiểm script và hệ thống trước khi đo hiệu năng — mục đích khác hẳn smoke build ở đây (bài SYS-07, SYS-08 sẽ dùng nghĩa đó).</p>
</div>

### 2.3. Ưu và nhược điểm

Smoke cho phản hồi nhanh, rẻ, dễ tự động hoá và tạo một điểm quyết định rõ trong pipeline. Đổi lại, nó cố ý bỏ qua phần lớn rủi ro: biên, tổ hợp, lỗi hiếm, phi chức năng. Một smoke suite "phình" theo thời gian sẽ mất đúng giá trị duy nhất của nó — tốc độ. Vì vậy mỗi ca bị loại khỏi smoke phải có **nơi kiểm thay thế**, không phải được miễn.

## 3. Thiết kế từ rủi ro và test basis

### 3.1. Test basis và oracle

Test basis là tài liệu làm căn cứ để thiết kế test; ở lab là `examples/toi-noi-testing/SPEC.md`. Đọc SPEC **trước** khi đọc implementation, để expected đến từ yêu cầu chứ không phải từ code.

**Oracle** là căn cứ xác định kết quả đúng hay sai. Nếu ta gọi lại chính hàm tính cước để tạo expected, test và code có thể sai giống hệt nhau và test vẫn xanh. Vì thế oracle tiền trong bài là một con số tính tay từ SPEC:

- R-FARE-01: xe máy (`bike`) phí mở cửa 12.000 đ, 4.500 đ/km, cước tối thiểu 15.000 đ.
- 5 km ⇒ cước cơ bản = 12.000 + 4.500 × 5 = 34.500 đ.
- 09:00 giờ Việt Nam không thuộc [11:00; 13:00) hay [17:00; 19:00) ⇒ không nhân 1,25 (R-FARE-04). Không mưa (R-FARE-05), không voucher.
- 34.500 > 15.000 nên không nâng lên tối thiểu (R-FARE-07); làm tròn lên bội số 1.000 đ (R-FARE-08) ⇒ **35.000 đ**.

Con số 35.000 được viết thẳng vào test. Nếu một ngày test báo 36.000, ta phải quay về SPEC để hỏi "ai đúng", không được sửa expected cho khớp.

### 3.2. Ba câu hỏi chọn case

Với mỗi ứng viên, hỏi:

1. **Nếu hỏng, nó có chặn hoạt động cốt lõi không?** Không đặt được chuyến hay không thu được tiền thì vòng kiểm nào phía sau cũng vô nghĩa.
2. **Nó có đi qua thành phần dễ hỏng của build mới không?** Mapping DB, cấu hình bảng giá, middleware xác thực — những chỗ thay đổi thường xuyên.
3. **Chi phí chạy và điều tra có hợp với một gate không?** Case cần 10 phút hay khó chẩn đoán khi đỏ thì không hợp smoke.

Không chọn chỉ vì case chạy nhanh hoặc đã có sẵn.

### 3.3. Bảng case

| ID | Requirement / rủi ro | Precondition | Input / action | Expected | Cleanup |
|---|---|---|---|---|---|
| SM-01 | R-API-15; tiến trình không phản hồi | App mới dựng, seed đã reset | `GET /api/health` | 200, body đúng `{ status: 'ok' }` | Không cần |
| SM-02 | R-API-03; mất chặn truy cập | Như trên | `POST /api/quotes` báo giá hợp lệ, **không** gửi `x-user` | 401, `error.code = UNAUTHENTICATED` | Không cần |
| SM-03 | R-STATE-02, R-PAY-03, R-FARE-01…08; đường tiền bị đứt | Seed có C001, D101; đồng hồ 09:00 VN 04/11/2026; không mưa | Báo giá 5 km xe máy → đặt bằng VíXanh → D101 nhận → arrive → pickup → start → complete → đọc lại | Giá 35.000; chuyến `completed`/`paid` qua API và repository; ví được gọi đúng 1 lần với khoá `trip:<id>:fare`, tổng 35.000 | Reset seed ở case kế tiếp |

Mọi case reset seed và dựng Express app mới; PGlite được đóng sau cả suite.

### 3.4. Những gì cố ý bỏ ra ngoài

| Bị loại khỏi smoke | Vì sao | Kiểm ở đâu |
|---|---|---|
| Biên quãng đường 0,5 km / 100 km | Không chặn luồng cốt lõi | [SYS-03 Functional Test](/articles/system-testing-functional-test) |
| Tổ hợp voucher | Nhiều ca, giá trị theo rủi ro thay đổi | [SYS-02 Sanity](/articles/system-testing-sanity-test), SYS-03 |
| Retry, lỗi ví, idempotency khi lỗi | Cần inject lỗi, khó chẩn đoán nhanh | [SYS-04 Integration](/articles/system-testing-integration-test), SYS-14 Recovery |
| Giao diện trình duyệt | Chi phí chạy cao hơn | [SYS-05 System Test](/articles/system-testing-system-test) |

### 3.5. Tiêu chí bắt đầu và kết thúc

- **Entry criteria**: dependency đã cài theo lockfile; SPEC và seed đúng phiên bản; không dùng dữ liệu production.
- **Exit criteria (bài tập)**: cả ba case pass, không có case bị skip, exit code 0.

Exit criteria này cho phép **kiểm tiếp**, không cho phép release production.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 250" role="img" aria-label="Smoke test là cổng giữa build và vòng kiểm thử chuyên sâu. Build mới đi vào ba case SM-01 health, SM-02 chặn vô danh, SM-03 đặt đến thu tiền. Nếu cả ba pass thì sang kiểm thử chuyên sâu gồm sanity, functional, integration, system. Nếu có case đỏ thì phân loại blocked do môi trường hoặc failed do sản phẩm và trả build kèm bằng chứng.">
<defs>
<marker id="sys01-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<rect x="16" y="95" width="130" height="56" rx="8" class="f-phys-soft s-phys"/><text x="81" y="119" text-anchor="middle" class="t-b">Build mới</text><text x="81" y="138" text-anchor="middle" class="t-small">mã + cấu hình</text>
<rect x="190" y="20" width="300" height="210" rx="10" class="f-bg s-pk dash"/>
<text x="206" y="42" class="t-b">Smoke gate (vài giây)</text>
<rect x="210" y="56" width="260" height="44" rx="8" class="f-pk-soft s-pk"/><text x="340" y="83" text-anchor="middle">SM-01 · health 200</text>
<rect x="210" y="110" width="260" height="44" rx="8" class="f-pk-soft s-pk"/><text x="340" y="137" text-anchor="middle">SM-02 · thiếu x-user ⇒ 401</text>
<rect x="210" y="164" width="260" height="44" rx="8" class="f-pk-soft s-pk"/><text x="340" y="191" text-anchor="middle">SM-03 · đặt → hoàn tất → thu 35.000</text>
<line x1="146" y1="123" x2="186" y2="123" class="s-phys" stroke-width="1.6" marker-end="url(#sys01-arrow)"/>
<rect x="560" y="30" width="344" height="80" rx="8" class="f-pk-soft s-pk"/><text x="732" y="58" text-anchor="middle" class="t-b">Cả 3 pass ⇒ kiểm tiếp</text><text x="732" y="80" text-anchor="middle" class="t-small">sanity · functional · integration · system</text><text x="732" y="98" text-anchor="middle" class="t-small">chưa phải quyết định release</text>
<rect x="560" y="140" width="344" height="80" rx="8" class="f-warn-soft s-warn"/><text x="732" y="168" text-anchor="middle" class="t-b">Có case đỏ ⇒ dừng, phân loại</text><text x="732" y="190" text-anchor="middle" class="t-small">blocked (môi trường) hoặc failed (sản phẩm)</text><text x="732" y="208" text-anchor="middle" class="t-small t-warn">trả build kèm request, expected/actual, build ID</text>
<line x1="490" y1="90" x2="556" y2="70" class="s-pk" stroke-width="1.6" marker-end="url(#sys01-arrow)"/>
<line x1="490" y1="160" x2="556" y2="180" class="s-warn" stroke-width="1.6" marker-end="url(#sys01-arrow)"/>
</svg>
</div>
<figcaption><b>Hình 1 · Smoke là cổng quyết định "có kiểm tiếp không".</b> Ba case nhỏ đứng giữa build và vòng kiểm thử chuyên sâu. Nhánh xanh chỉ mở cửa cho các suite khác; nhánh đỏ phải đi kèm bằng chứng tái hiện được.</figcaption>
</figure>

## 4. Thực hành có thể chạy lại

### 4.1. Chuẩn bị và lệnh chạy

Mã đầy đủ: `examples/toi-noi-testing/articles/system-testing-smoke-test/smoke.test.ts` ([GitHub](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-smoke-test)). Từ thư mục `examples/toi-noi-testing`:

```bash
npm ci --no-audit --no-fund
npx vitest run articles/system-testing-smoke-test
npm run typecheck
```

Chỉ cần `npm ci` lần đầu. Các thành phần của bài:

- **PGlite** chạy PostgreSQL dưới WASM trong tiến trình test, nên không cần DB server và mỗi lần chạy có dữ liệu sạch.
- **Supertest** gửi HTTP request thẳng vào Express app mà không mở cổng mạng.
- **FixedClock** cố định 09:00 giờ VN ngày 04/11/2026, để giá không đổi theo giờ chạy test.
- **FakeWeather** mặc định không thành phố nào mưa; **ví VíXanh giả** ghi lại mọi lần gọi và số tiền thu.

Đó là các **điều kiện kiểm soát**, không phải hệ thống bên ngoài đã được kiểm chứng. Bài không kiểm DNS, TLS, reverse proxy hay VíXanh thật.

### 4.2. Dựng hệ thống và vòng đời test

```ts
describe('SYS-01 · release smoke qua API và PGlite', () => {
  let system: LabSystem;
  let app: ReturnType<typeof createApp>;
  beforeAll(async () => {
    system = await createPgliteSystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
  });
  beforeEach(async () => {
    await system.reset();
    app = createApp(system.deps, { logErrors: false });
  });
  afterAll(async () => { await system?.db?.close(); });
```

`beforeAll` khởi tạo PGlite một lần vì việc này tốn vài giây. `beforeEach` đưa dữ liệu về seed, đặt lại thời tiết và sổ ví, rồi dựng app mới. Nhờ vậy các case độc lập: chạy riêng SM-03 hay chạy cả ba theo thứ tự nào cũng cho cùng kết quả.

### 4.3. SM-01 và SM-02

```ts
it('SM-01 · tiến trình API phản hồi health đúng hợp đồng', async () => {
  const res = await request(app).get('/api/health').expect(200);
  expect(res.body).toEqual({ status: 'ok' });
});

it('SM-02 · R-API-03: thao tác nghiệp vụ thiếu danh tính bị chặn', async () => {
  const res = await request(app).post('/api/quotes').send({
    kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5,
  }).expect(401);
  expect(res.body.error.code).toBe('UNAUTHENTICATED');
});
```

SM-01 dùng `toEqual` vì hợp đồng health nhỏ và cố định. SM-02 gửi một báo giá **hợp lệ** nhưng thiếu header: nếu body sai, ta không biết 4xx đến từ xác thực hay validation. Theo R-VALID-01, xác thực được kiểm trước định dạng đầu vào, nên mã phải là `UNAUTHENTICATED`. Assert cả mã lỗi chứ không chỉ status 401.

### 4.4. SM-03 — hành trình đi xuyên tiền

Luồng chia ba đoạn. **Đoạn 1 — báo giá**, phát hiện cấu hình giá lệch:

```ts
const quote = await request(app).post('/api/quotes').set('x-user', 'customer:C001')
  .send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 }).expect(200);
expect(quote.body.quote.total).toBe(35_000);
```

**Đoạn 2 — tạo chuyến và chuyển trạng thái** bằng `quoteId` nhận được thật, không đoán ID:

```ts
const created = await request(app).post('/api/trips').set('x-user', 'customer:C001')
  .set('Idempotency-Key', 'sys01-smoke-001')
  .send({ quoteId: quote.body.quote.quoteId, paymentMethod: 'vixanh' }).expect(201);
const id: string = created.body.trip.id;
expect(id).toEqual(expect.any(String));
expect(created.body.trip.status).toBe('searching');
await request(app).post(`/api/trips/${id}/accept`).set('x-user', 'driver:D101')
  .send({ etaMinutes: 5 }).expect(200);
for (const step of ['arrive', 'pickup', 'start', 'complete']) {
  await request(app).post(`/api/trips/${id}/${step}`).set('x-user', 'driver:D101').send({}).expect(200);
}
```

`Idempotency-Key` là bắt buộc khi tạo chuyến (R-API-08); thiếu khoá sẽ nhận 400 và smoke đỏ vì lý do không liên quan. Các bước `arrive → pickup → start → complete` đi đúng bảng chuyển trạng thái R-STATE-02.

**Đoạn 3 — đọc lại và kiểm tác dụng phụ**:

```ts
const read = await request(app).get(`/api/trips/${id}`).set('x-user', 'customer:C001').expect(200);
expect(read.body.trip).toMatchObject({ id, status: 'completed', fare: 35_000, paymentStatus: 'paid' });
expect(await system.deps.trips.findById(id)).toMatchObject({ status: 'completed', paymentStatus: 'paid' });
expect(system.payments.chargedKeys()).toEqual([`trip:${id}:fare`]);
expect(system.payments.calls).toHaveLength(1);
expect(system.payments.totalCharged()).toBe(35_000);
```

Vì sao không dừng ở "mọi request đều 200"? Vì server hoàn toàn có thể trả thành công mà không lưu, hoặc lưu đúng mà thu sai tiền. Ba lớp assertion bắt ba kiểu hỏng khác nhau:

- **Đọc qua API**: khách C001 thấy chuyến của mình đã `completed`, `paid`, giá 35.000.
- **Đọc qua repository**: trạng thái thật sự nằm trong DB của phiên chạy, không chỉ trong response.
- **Sổ ví giả**: đúng khoá idempotency `trip:<id>:fare` theo R-PAY-03, gọi đúng một lần, tổng thu đúng 35.000.

Khoá ví được dựng từ `id` sinh trong test, không hardcode. `toMatchObject` chỉ so các trường quan trọng — snapshot cả response sẽ đỏ mỗi khi ai đó thêm một trường không liên quan.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Assertion chỉ đúng trong ngữ cảnh của nó</span>
<p><code>calls</code> có độ dài 1 là đúng cho happy path không inject lỗi. Khi có retry, nhiều lần gọi cùng một khoá idempotency có thể là hành vi <b>đúng</b>. Đừng chép nguyên assertion này sang bài Recovery (SYS-14) — ở đó, thứ cần kiểm là tổng tiền thu và số khoá duy nhất.</p>
</div>

### 4.5. Expected và observed

**Expected** (từ thiết kế): 3 test pass, exit code 0; thời gian thay đổi theo máy.

**Observed**, lần chạy lại ngày 13/11/2026 trên Windows 11, Node v22.18.0, Vitest 3.2.7:

```text
 ✓ articles/system-testing-smoke-test/smoke.test.ts (3 tests) 2728ms

 Test Files  1 passed (1)
      Tests  3 passed (3)
   Duration  7.69s (transform 372ms, setup 0ms, collect 3.78s, tests 2.73s, environment 0ms, prepare 430ms)
```

Lần chạy đầu khi soạn bài (cùng ngày, Node v24.15.0) chạy SYS-01 cùng bộ smoke có sẵn của lab: 4 file, 25 test pass trong 4,60 giây; riêng SYS-01 mất 3,028 giây, `npm run typecheck` exit 0. Phần lớn thời gian là khởi tạo và reset PGlite, không phải logic nghiệp vụ.

Lần thử đầu tiên **trước khi cài dependency** đã thất bại ở bước nạp config vì thiếu Vitest cục bộ; `npx` tự tải một Vitest khác ngoài lab và không giải quyết được import. Đây là lỗi chuẩn bị môi trường — được ghi là *blocked*, không phải defect sản phẩm. Cài theo lockfile rồi chạy lại mới cho kết quả trên.

## 5. Đọc thất bại và ra quyết định

### 5.1. Từ dấu hiệu tới giả thuyết

| Dấu hiệu | Giả thuyết cần kiểm | Bằng chứng tiếp theo |
|---|---|---|
| PGlite không khởi tạo | Dependency, WASM, thiếu tài nguyên | Stack trace, phiên bản Node, lockfile |
| Health xanh, tạo chuyến lỗi | Mapping/schema hoặc quy tắc nghiệp vụ | Response code, bước thất bại, log DB |
| Giá khác 35.000 | Seed, giờ, mưa, đặc tả hoặc regression | Diff cấu hình và test basis; **không** tự sửa expected |
| `completed` nhưng chưa `paid` | Đường thanh toán hỏng | Payment calls, khoá idempotency, trạng thái lưu |
| Chạy riêng pass, chạy chung fail | Trạng thái dùng chung hoặc cleanup thiếu | Seed, vòng đời app, thời điểm reset |

### 5.2. Blocked hay failed

- **Blocked**: test không chạy được tới điểm đánh giá vì môi trường (thiếu dependency, cổng bị chiếm, WASM lỗi). Báo kèm bằng chứng môi trường; chưa kết luận gì về sản phẩm.
- **Failed**: test chạy tới assertion và kết quả khác expected. Tạo defect với request, expected/actual, phiên bản build, dữ liệu tái hiện.

Không đổi mọi thất bại thành lỗi sản phẩm, cũng không rerun đến khi xanh. Một lần pass sau retry **không xoá** lần lỗi trước — nó là dấu hiệu flakiness cần điều tra.

### 5.3. Viết quyết định

Ví dụ khi đỏ:

> SM-03 fail ở bước kiểm `paymentStatus`: expected `paid`, actual `pending`; chuyến đã `completed`. Build #… , seed lab mặc định. Dừng vòng kiểm chức năng phụ thuộc thanh toán; review test design độc lập vẫn có thể tiếp tục. Sau khi sửa: chạy confirmation cho SM-03, rồi chạy lại toàn bộ smoke.

Ví dụ khi xanh: "Ba case pass. Đường health, chặn người dùng vô danh, báo giá → đặt → nhận → hoàn tất → đọc lại → thu tiền hoạt động trong lab API/PGlite với ví giả. Cho phép bắt đầu sanity và functional." Câu kết luận nêu **cấu hình** và **phạm vi**, không nói "hệ thống ổn".

## 6. Mở rộng nâng cao

### 6.1. Gate trước và sau triển khai

Suite trong bài phù hợp gate kiểm mã: các thành phần được lắp ráp trong một tiến trình. **Post-deploy smoke** thì phải gọi base URL thật của một môi trường cô lập, để phủ routing, TLS, biến môi trường và đúng artifact đang chạy. Khi đó phải ghi build ID và config version — kết quả của build cũ không phải bằng chứng cho build mới. Một pipeline hợp lý:

```text
build → smoke trong tiến trình (bài này) → deploy staging → post-deploy smoke
      → regression / risk suites → quyết định release
```

Bài này **chưa** chạy post-deploy smoke trên staging; SYS-16 Deployment Test sẽ quay lại phần đó.

### 6.2. Dữ liệu và tác dụng phụ ở môi trường dùng chung

Lab reset toàn bộ DB — không bao giờ làm vậy trên staging dùng chung. Thay vào đó: tenant kiểm thử riêng, dữ liệu gắn run ID, tài khoản dành riêng và cleanup giới hạn theo run ID. Bước thu tiền phải đi qua payment sandbox đã cấu hình. Nếu cleanup lỗi, lưu danh sách đối tượng còn sót để xử lý; đừng che lỗi bằng cách bỏ assertion.

### 6.3. Hệ phân tán và nhất quán cuối cùng

Ở Tới Nơi thật, nếu việc tạo chuyến đi qua message broker, API có thể trả *accepted* trước khi trạng thái đọc được cập nhật. Khi SPEC cho phép eventual consistency, smoke phải **chờ có điều kiện** tới một deadline đã thống nhất theo SLO, lưu lịch sử trạng thái và correlation ID để chẩn đoán. Không chèn `sleep` cố định để test xanh. Lab hiện không có broker nên bài chưa chứng minh cơ chế này; SYS-05 bàn kỹ hơn.

### 6.4. Ngân sách thời gian

Giả sử nhóm đặt mục tiêu nội bộ "smoke dưới hai phút" — đây là **giả định thiết kế**, không phải chuẩn chung. Đo p95 thời gian suite qua nhiều lần chạy, loại các case trùng rủi ro hoặc chuyển sang suite sâu. Theo dõi tỷ lệ báo động giả và số sự cố lọt qua gate để điều chỉnh; đừng dùng tổng số test làm thước đo chất lượng.

### 6.5. Chứng minh test biết bắt lỗi (thí nghiệm đề xuất)

Một test chưa từng đỏ là một test chưa được chứng minh. Đề xuất: trên nhánh bài tập, viết một adapter ví lỗi trong thư mục bài — báo thành công nhưng không ghi số tiền đã thu — và inject qua port. Kỳ vọng: SM-03 đỏ ở oracle tiền, SM-01 vẫn xanh. Bỏ lỗi đi, suite xanh lại. Thí nghiệm này **chưa được chạy** trong báo cáo hiện tại; khi làm, giữ nó trong thư mục bài, không sửa `src/` dùng chung của lab.

## 7. Anti-pattern, giới hạn bằng chứng và đánh đổi

### 7.1. Anti-pattern

- **Chỉ kiểm health**: bỏ sót DB và thanh toán — chính là tình huống mở bài.
- **Nhét regression vào smoke**: mất phản hồi nhanh, gate trở thành nút cổ chai.
- **Snapshot cả response**: test đỏ vì trường không liên quan, người ta quen bỏ qua đỏ.
- **Dùng giờ thực**: chạy lúc 11:30 giá nhân 1,25, test "ngẫu nhiên" đỏ.
- **Retry vô hạn**: che flakiness thay vì sửa nó.
- **Tắt assertion để qua gate**: gate còn đó nhưng không còn kiểm gì.

### 7.2. Giới hạn bằng chứng

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Ba case xanh chưa chứng minh</span>
<p>Suite dùng PGlite một kết nối: chưa kiểm concurrency PostgreSQL nhiều kết nối, độ bền sau restart, trình duyệt, mạng ngoài, JWT, nhà cung cấp thanh toán thật hay khả năng chịu tải. Header <code>x-user</code> là giản lược danh tính của lab (R-API-03), không phải bảo mật production. Đọc lại qua repository trong cùng phiên DB chứng minh dữ liệu ghi/đọc được ở phiên đó, không chứng minh backup/restore hay tồn tại sau sự cố máy.</p>
</div>

### 7.3. Đánh đổi chi phí và phạm vi

Mỗi case thêm vào smoke tăng độ phủ nhưng tăng thời gian và số lý do có thể đỏ. Ba case ở đây phủ ba lớp rủi ro khác nhau (tiến trình, kiểm soát truy cập, đường tiền) với khoảng 3 giây chạy. Thêm case thứ tư nên đi kèm câu hỏi: rủi ro mới nào mà ba case hiện có không bắt được?

## 8. Bài tập và câu hỏi tự kiểm

### Bài tập

**Bài 1 (cơ bản).** Bỏ header `x-user` khỏi bước báo giá trong SM-03 rồi chạy lại. Assertion nào đỏ đầu tiên và vì sao test dừng ở đó?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><code>.expect(200)</code> của báo giá đỏ ngay vì server trả 401 <code>UNAUTHENTICATED</code>; mọi bước sau không chạy, ví không bị gọi. Đây là hành vi tốt: test dừng ở đúng bước hỏng, thông báo chỉ thẳng nguyên nhân. Nhớ khôi phục header sau thí nghiệm.</p>
</details>

**Bài 2 (trung bình).** Thêm kiểm tra khách C002 không đọc được chuyến của C001. Expected là gì, và có nên giữ case này trong smoke?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Theo R-API-05, chuyến tồn tại nhưng không được xem trả <b>404 <code>TRIP_NOT_FOUND</code></b> — giống hệt chuyến không tồn tại, để không lộ sự tồn tại. Giữ trong smoke hay không phụ thuộc rủi ro của release: nếu build đụng tới middleware phân quyền hay truy vấn chuyến, đáng thêm; nếu không, để nó ở suite bảo mật (SYS-10).</p>
</details>

**Bài 3 (nâng cao).** Thiết kế post-deploy smoke cho staging có nhiều worker và nhiều đội cùng dùng.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Cần: base URL và build ID được ghi vào báo cáo; run ID gắn vào mọi dữ liệu tạo ra; tenant hoặc tài khoản riêng; payment sandbox; timeout có căn cứ (từ SLO, không đoán); chờ có điều kiện thay vì sleep; correlation ID trong log; cleanup chỉ xoá dữ liệu mang run ID của chính lần chạy, kể cả khi test đỏ.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** Ba case SM-01…03 đều pass. Build có đủ điều kiện release production không?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>Không.</b> Smoke xanh chỉ cho phép kiểm tiếp. Biên, tổ hợp, phi chức năng, tích hợp thật và tiêu chí acceptance chưa được phủ.</p>
</details>

**Câu 2.** SM-01 xanh nhưng SM-03 đỏ. Hai kết quả này có mâu thuẫn không?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>Không.</b> Chúng trả lời hai câu hỏi khác nhau. Health của lab không chạm DB, nên chỉ chứng minh tiến trình phản hồi; SM-03 chứng minh luồng nghiệp vụ đi được tới kho dữ liệu và ví.</p>
</details>

**Câu 3.** Vì sao expected 35.000 được viết cứng thay vì gọi hàm tính cước?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Để oracle độc lập với code đang kiểm. Gọi lại hàm tính cước thì test và code sai giống nhau vẫn xanh. 35.000 được tính tay từ R-FARE-01…08: 12.000 + 4.500 × 5 = 34.500, không cao điểm, không mưa, làm tròn lên 35.000.</p>
</details>

**Câu 4.** Test đỏ lần đầu, rerun thì xanh. Ghi nhận thế nào?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Ghi cả hai lần chạy. Lần đỏ là bằng chứng flakiness hoặc lỗi không ổn định; mở việc điều tra (trạng thái dùng chung, thời gian, môi trường). Không coi lần xanh sau retry là "đã ổn".</p>
</details>

**Câu 5.** Phân loại: `npx vitest` báo không tìm thấy module vì chưa chạy `npm ci`.

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>Blocked</b> do môi trường, không phải defect sản phẩm. Ghi phiên bản Node/npm và bước chuẩn bị bị thiếu, cài theo lockfile rồi chạy lại.</p>
</details>

## 9. Tài liệu tham khảo

Nguồn ngoài tra cứu 13/11/2026.

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — test levels, test types, confirmation và regression testing: https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf. Phần lựa chọn smoke và thiết kế pipeline trong bài là hướng dẫn thực tiễn, không phải yêu cầu của syllabus.
- Grafana k6 — *Load test types*: https://grafana.com/docs/k6/latest/testing-guides/test-types/ — đối chiếu nghĩa "smoke" trong kiểm thử hiệu năng.
- Lab Tới Nơi: `SPEC.md` (R-API-03, R-API-05, R-API-08, R-API-15, R-VALID-01, R-STATE-02, R-PAY-03, R-FARE-01…08) và mã bài — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-smoke-test
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-011 Tổng quan 5 test level](/articles/tong-quan-5-test-level-istqb) · [Quy trình kiểm thử](/articles/quy-trinh-kiem-thu-test-process) · [Confirmation testing và regression testing](/articles/confirmation-testing-va-regression-testing) · [System Testing (series ISTQB)](/articles/system-testing)

Bài tiếp theo: [SYS-02 · Sanity Test](/articles/system-testing-sanity-test).
