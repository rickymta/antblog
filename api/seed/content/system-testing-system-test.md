---
slug: system-testing-system-test
title: "System Test: kiểm hành vi của toàn ứng dụng Tới Nơi trong ranh giới đã chọn"
summary: "Bài thực hành system testing: vẽ ranh giới hệ thống đang kiểm, tách system test khỏi E2E và UAT, thiết kế ba hành trình nhiều vai trò qua hai giao diện, chờ trạng thái bất đồng bộ bằng điều kiện thay vì sleep, đối chiếu UI với API và ghi rõ giới hạn bằng chứng của một lab Playwright trên server riêng."
category: testing
level: intermediate
tags: [testing, sys-05, system-testing, e2e, playwright, test-environment, observability, eventual-consistency]
language: vi
---

API hoàn tất chuyến trả `completed`, DB lưu đúng, nhưng trang khách vẫn hiện «Đang tìm tài xế» vì vòng hỏi lại trạng thái không chạy. Mọi bài integration của backend đều xanh, còn người dùng thì không biết chuyến đã kết thúc. Bài này trả lời: **muốn bắt loại lỗi đó, cần kiểm cái gì, qua đâu, trên môi trường nào, và được phép kết luận tới đâu.** Ta vẽ ranh giới hệ thống đang kiểm của lab Tới Nơi, thiết kế ba hành trình khách–tài xế qua hai giao diện, tự động hoá bằng Playwright trên một server riêng, rồi đọc kết quả như một người phải ký quyết định.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-05</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 1</td></tr>
<tr><th scope="row">Cấp độ</th><td>Trung cấp — biết HTTP, test case, Playwright cơ bản</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>System · Functional (có nhắc phần phi chức năng ở mục mở rộng)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-integration-test">SYS-04 Integration Test</a>; khái niệm cấp hệ thống theo ISTQB ở <a href="/articles/system-testing">TEST-014 System Testing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): 3 hành trình SY-01…03 qua trang <code>/</code> và <code>/driver.html</code>, server riêng cổng 3115 với PGlite — <code>examples/toi-noi-testing/articles/system-testing-system-test/</code>, config <code>system-testing.config.ts</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>ISTQB CTFL v4.0.1 mục 2.2.1; tài liệu Playwright (Web server, Assertions); <code>SPEC.md</code> của lab (R-UI, R-STATE, R-PAY, R-CANCEL, R-API, R-TEST)</td></tr>
</tbody>
</table>
</div>

## 1. Tình huống và mục tiêu

Ở [SYS-04](/articles/system-testing-integration-test) ta đã kiểm chỗ nối API–repository PGlite và cổng ví giả: SQL mapping đúng, ví được gọi đúng khoá idempotency. Những bằng chứng đó nằm **dưới** giao diện. Chúng không trả lời được các câu hỏi mà khách và tài xế thực sự gặp:

- Khách bấm «Đặt chuyến» xong có thấy chuyến chuyển sang «Đang tìm tài xế» không?
- Tài xế mở trang của mình có thấy đúng chuyến vừa đặt và đi được hết các bước không?
- Khi tài xế bấm «Hoàn thành», trang khách — không tải lại — có tự đổi sang «Hoàn thành» không?
- Khách sửa quãng đường sau khi đã có báo giá thì hệ thống có chặn đặt theo giá cũ không?

Mỗi câu cần ít nhất hai mảnh chạy cùng nhau: HTML/JS của trang, HTTP server, use case, kho dữ liệu, và đôi khi hai người dùng. Đó là lãnh thổ của system testing.

Sau bài này, bạn có thể:

1. Xác định **system under test** và ghi rõ cái gì nằm trong, cái gì được mô phỏng, cái gì nằm ngoài ranh giới.
2. Thiết kế hành trình nhiều vai trò từ rủi ro người dùng, với oracle độc lập cho tiền và trạng thái.
3. Kiểm kết quả ở **cả** UI lẫn API, và giải thích mỗi phía bắt được lỗi gì mà phía kia bỏ sót.
4. Chờ trạng thái bất đồng bộ bằng assertion có điều kiện, không dùng sleep cố định.
5. Phân biệt «ba hành trình lab xanh» với bằng chứng release production.

## 2. Khái niệm và các ranh giới dễ nhầm

Phần định nghĩa theo ISTQB — test basis, môi trường, ai thực hiện — đã có ở [TEST-014](/articles/system-testing). Ở đây ta chỉ chốt cách dùng trong series.

**System testing** đánh giá hành vi và năng lực của hệ thống hoàn chỉnh **trong phạm vi đã xác định**. Nó gồm cả chức năng và phi chức năng, và không đồng nghĩa với UI automation. «Hoàn chỉnh» luôn tương đối: bản MVP modular monolith của lab khác kiến trúc microservices mục tiêu của Tới Nơi. Vì vậy câu đầu tiên của mọi kế hoạch system test phải là: *hệ thống ở đây là gì?*

| Khái niệm | Đối tượng chính | Ví dụ Tới Nơi |
|---|---|---|
| Integration | Tương tác giữa thành phần/hệ thống | SQL mapping, retry ở cổng thanh toán (SYS-04) |
| System | Hệ thống được lắp ráp theo phạm vi đã chọn | Khách đặt, tài xế xử lý, khách thấy kết quả |
| E2E | Một luồng từ điểm bắt đầu tới kết quả cuối | Đặt → nhận → hoàn tất qua hai UI |
| UAT | Mức đáp ứng nhu cầu và tiêu chí chấp nhận nghiệp vụ | Đại diện nghiệp vụ chấp nhận quy trình |

Ba điểm hay nhầm:

- **E2E là cách tổ chức hành trình, không phải một cấp kiểm thử riêng.** Một test lỗi đầu vào ở trang khách cũng là system test dù không đi hết vòng đời chuyến (xem SY-03 bên dưới).
- **Browser test không tự động là system test.** Nếu mock mọi request, test chỉ kiểm giao diện — hữu ích, nhưng không cho bằng chứng về đường browser–server–DB.
- **System test xanh không phải UAT.** Test máy chỉ đối chiếu với đặc tả; còn «đúng nhu cầu» do người nghiệp vụ quyết định.

### 2.1. Ranh giới của bài này

Trong lab, hệ thống đang kiểm gồm Chrome headless → HTML/JS trang khách và trang tài xế → HTTP server Express chạy riêng → use case → repository PGlite. Đồng hồ, thời tiết, ví VíXanh và danh tính `x-user` là **mô phỏng**. Không có Kafka, GPS, ứng dụng mobile, reverse proxy, nhà cung cấp thanh toán thật hay hệ thống đăng nhập production (SPEC R-ARCH-02 liệt kê những thứ «chưa có trong lab»).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Ranh giới system under test của SYS-05: bên trái hai browser context của Chrome headless cho khách ở trang / và tài xế ở /driver.html. Ở giữa là khung hệ thống đang kiểm, server riêng cổng 3115 gồm web, API Express, use case và repository PGlite. Bên phải là các phần mô phỏng: đồng hồ dời được, thời tiết giả, ví VíXanh giả, danh tính x-user. Phía dưới là kênh điều khiển test /api/test/reset và /api/test/clock chỉ dùng cho setup, và danh sách những thứ nằm ngoài ranh giới: Kafka, GPS, mobile app, JWT, provider thanh toán thật.">
<defs>
<marker id="sys05-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<rect x="16" y="40" width="170" height="56" rx="8" class="f-phys-soft s-phys"/><text x="101" y="64" text-anchor="middle" class="t-b">Context khách</text><text x="101" y="83" text-anchor="middle" class="t-small">trang / · C001</text>
<rect x="16" y="140" width="170" height="56" rx="8" class="f-phys-soft s-phys"/><text x="101" y="164" text-anchor="middle" class="t-b">Context tài xế</text><text x="101" y="183" text-anchor="middle" class="t-small">/driver.html · D101</text>
<text x="101" y="118" text-anchor="middle" class="t-small t-phys">khách hỏi lại mỗi 2 giây</text>
<rect x="230" y="20" width="470" height="200" rx="10" class="f-bg s-pk dash"/>
<text x="246" y="42" class="t-b">Hệ thống đang kiểm — server riêng 127.0.0.1:3115</text>
<rect x="250" y="60" width="130" height="44" rx="8" class="f-pk-soft s-pk"/><text x="315" y="87" text-anchor="middle">web (HTML, JS)</text>
<rect x="395" y="60" width="130" height="44" rx="8" class="f-pk-soft s-pk"/><text x="460" y="87" text-anchor="middle">API Express</text>
<rect x="540" y="60" width="140" height="44" rx="8" class="f-pk-soft s-pk"/><text x="610" y="87" text-anchor="middle">use case</text>
<rect x="470" y="140" width="210" height="44" rx="8" class="f-pk-soft s-pk"/><text x="575" y="167" text-anchor="middle">repository PGlite</text>
<text x="350" y="160" text-anchor="middle" class="t-small">mọi thao tác nghiệp vụ</text>
<text x="350" y="176" text-anchor="middle" class="t-small">đi qua giao diện</text>
<line x1="186" y1="68" x2="246" y2="78" class="s-phys" stroke-width="1.6" marker-end="url(#sys05-arrow)"/>
<line x1="186" y1="168" x2="246" y2="92" class="s-phys" stroke-width="1.6" marker-end="url(#sys05-arrow)"/>
<line x1="380" y1="82" x2="391" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#sys05-arrow)"/>
<line x1="525" y1="82" x2="536" y2="82" class="s-pk" stroke-width="1.6" marker-end="url(#sys05-arrow)"/>
<line x1="610" y1="104" x2="610" y2="136" class="s-pk" stroke-width="1.6" marker-end="url(#sys05-arrow)"/>
<text x="822" y="28" text-anchor="middle" class="t-small t-warn">mô phỏng trong lab</text>
<rect x="740" y="38" width="164" height="36" rx="8" class="f-warn-soft s-warn"/><text x="822" y="61" text-anchor="middle">đồng hồ dời được</text>
<rect x="740" y="84" width="164" height="36" rx="8" class="f-warn-soft s-warn"/><text x="822" y="107" text-anchor="middle">thời tiết giả</text>
<rect x="740" y="130" width="164" height="36" rx="8" class="f-warn-soft s-warn"/><text x="822" y="153" text-anchor="middle">VíXanh giả</text>
<rect x="740" y="176" width="164" height="36" rx="8" class="f-warn-soft s-warn"/><text x="822" y="199" text-anchor="middle">danh tính x-user</text>
<line x1="700" y1="56" x2="736" y2="56" class="s-warn dash" stroke-width="1.4"/>
<line x1="700" y1="102" x2="736" y2="102" class="s-warn dash" stroke-width="1.4"/>
<line x1="700" y1="148" x2="736" y2="148" class="s-warn dash" stroke-width="1.4"/>
<line x1="700" y1="194" x2="736" y2="194" class="s-warn dash" stroke-width="1.4"/>
<rect x="250" y="236" width="430" height="28" rx="6" class="f-bg s-phys dash"/><text x="465" y="255" text-anchor="middle" class="t-small">kênh setup: POST /api/test/reset · /api/test/clock (chỉ khi bật test API)</text>
<text x="460" y="290" text-anchor="middle" class="t-small t-warn">ngoài ranh giới: Kafka · GPS · mobile app · JWT · provider thanh toán thật · reverse proxy</text>
</svg>
</div>
<figcaption><b>Hình 1 · Ranh giới system under test của SYS-05.</b> Hai browser context tách biệt thao tác với cùng một server chạy riêng. Kênh điều khiển test chỉ dùng để reset dữ liệu và đặt giờ, không dùng để đẩy chuyến tới trạng thái cuối. Mọi thứ ở hàng dưới cùng không được bài này kiểm.</figcaption>
</figure>

Viết ranh giới ra giấy có hai tác dụng. Trước khi chạy, nó cho biết test nào cần dựng gì. Sau khi chạy, nó giới hạn câu kết luận: «ba hành trình đạt **trong ranh giới Hình 1**», không phải «Tới Nơi hoạt động».

## 3. Chiến lược: ít hành trình nhưng oracle rõ

### 3.1. Chọn hành trình từ rủi ro người dùng

Browser test đắt: chậm hơn API test, nhiều điểm hỏng, khó chẩn đoán. Vì vậy ta không sao chép 24 ca giá của bài [Functional](/articles/system-testing-functional-test) lên trình duyệt — những tổ hợp tiền đó đã có phép kiểm nhanh. UI giữ các đường đại diện và các lỗi **chỉ xuất hiện khi ghép đủ mảnh**.

| Rủi ro người dùng | Vì sao cần cấp hệ thống | Hành trình |
|---|---|---|
| Chuyến không hoàn thành được qua hai ứng dụng | Cần hai UI, API và chuyển trạng thái cùng chạy | SY-01 |
| Khách không thấy tiến độ dù server đã cập nhật | Cần polling của trang khách và backend | SY-01 |
| Huỷ nhưng ghi phí sai hoặc vẫn huỷ được sau khi xong | Cần nút UI, quy tắc phí và trạng thái lưu | SY-02, SY-01 |
| Sửa dữ liệu nhưng vẫn đặt theo báo giá cũ | Cần sự kiện nhập liệu của trang và kiểm lỗi ở server | SY-03 |
| Khách khác đọc được chuyến | Quy tắc ở API, nhưng dữ liệu tạo qua UI thật | SY-01 (oracle bổ trợ) |

### 3.2. Test basis

Mỗi kỳ vọng trích từ SPEC của lab, không từ implementation:

- **R-UI-01** — trang khách: báo giá, đặt, trạng thái tự làm mới mỗi 2 giây, «Sửa bất kỳ ô nào sau khi báo giá ⇒ phải báo giá lại».
- **R-UI-02** — trang tài xế: tải chuyến đang chờ, nhận, bấm lần lượt các bước tới hoàn thành.
- **R-UI-04** — tiền hiển thị theo định dạng Việt Nam kèm « đ».
- **R-STATE-02** — các chuyển trạng thái hợp lệ `searching → accepted → arriving → picked_up → in_transit → completed`.
- **R-PAY-03** — chuyến `vixanh` hoàn thành thì thu đúng `fare`.
- **R-CANCEL-01**, **R-PAY-05** — huỷ khi `searching` phí 0 đ; phí 0 ⇒ `paymentStatus = none`.
- **R-API-05** — khách khác đọc chuyến nhận 404 `TRIP_NOT_FOUND`.
- **R-FARE-02** — quãng đường sau làm tròn phải thuộc [0,5; 100] km, ngoài khoảng ⇒ `DISTANCE_OUT_OF_RANGE`.

SY-03 chỉ chọn ô quãng đường để sửa. Bài **không** tuyên bố đã kiểm mọi ô của R-UI-01 — đó là một bài tập ở mục 10.

### 3.3. Oracle tiền độc lập

Xe máy 5 km, 09:00 giờ Việt Nam (ngoài cao điểm), không mưa, không voucher: 12.000 + 4.500 × 5 = 34.500 đ, làm tròn lên bội số 1.000 theo R-FARE-08 ⇒ **35.000 đ**. UI phải hiện đúng chuỗi `35.000 đ`, API phải trả `fare: 35000`.

Hai expected này được tính trên giấy từ bảng giá. Đừng đọc số trên UI rồi dùng chính số đó làm expected cho API: nếu calculator sai, cả hai phía sẽ sai giống nhau và test vẫn xanh.

## 4. Môi trường và quyền sở hữu dữ liệu kiểm thử

Bài có config riêng `system-testing.config.ts`. Nó tự khởi động server trên cổng 3115 với PGlite và **không tái sử dụng** server có sẵn. Nếu cổng đang bận, runner dừng thay vì chạy trên một server không rõ cấu hình rồi reset dữ liệu của nó. Đây là điều kiện để kết quả diễn giải được, không chỉ là tiện ích.

```ts
webServer: {
  command: 'npx tsx src/server.ts',
  url: 'http://127.0.0.1:3115/api/health',
  reuseExistingServer: false,
  timeout: 60_000,
  env: {
    PORT: '3115', NODE_ENV: 'test', TOINOI_TEST_API: '1',
    TOINOI_DB: 'pglite', TOINOI_RAIN: '',
    TOINOI_START_AT: '2026-11-04T09:00:00+07:00',
    TRIP_RATE_LIMIT_MAX: '5', TRIP_RATE_LIMIT_WINDOW_MS: '60000',
  },
},
```

| Cấu hình | Giá trị | Lý do |
|---|---|---|
| Browser | Chrome cài sẵn (`channel: 'chrome'`), headless | Chưa có matrix nhiều engine — để dành cho bài Compatibility |
| Dữ liệu | Seed C001, D101, SG, xe máy, 5 km; reset trước mỗi case | Mỗi case độc lập, chạy riêng hay chạy chung cùng kết quả |
| Giờ | Đặt về 09:00 VN 04/11/2026 trước mỗi case; đồng hồ tiếp tục chạy | Xa biên cao điểm 11:00 nên thời gian thao tác không đổi oracle tiền |
| DB | PGlite trong tiến trình server | Không dùng DB dùng chung |
| Rate limit | 5 lượt/60 giây | Mỗi case chỉ tạo tối đa một chuyến |
| Worker / retry | `workers: 1`, `retries: 0` | Tránh reset chéo, không che flakiness |
| Timeout | Test 30 giây, assertion 8 giây, khởi động server 60 giây | Budget local, không phải SLA |
| Bằng chứng | Trace và screenshot khi fail, dưới `reports/system-testing` | Không commit; có thể chứa payload |

`/api/test/reset` và `/api/test/clock` chỉ tồn tại khi `NODE_ENV=test` hoặc `TOINOI_TEST_API=1` (R-TEST-01). Ta chỉ dùng chúng cho **setup**, không dùng để đẩy chuyến thẳng tới `completed`. Các thao tác nghiệp vụ của hai vai trò đi qua giao diện; nếu tạo chuyến bằng API, ta mất bằng chứng nút «Đặt chuyến» hoạt động.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Không trỏ config này vào môi trường dùng chung</span>
<p><code>beforeEach</code> reset <b>toàn bộ</b> dữ liệu. Chạy nó trên staging dùng chung sẽ xoá dữ liệu của người khác. Config mặc định <code>npm run test:e2e</code> cũng phát hiện file spec này nhưng có thể dùng kho bộ nhớ và tái sử dụng server — kết quả đó <b>không</b> tương đương môi trường được báo cáo ở đây.</p>
</div>

## 5. Thiết kế ba kịch bản

| ID | Requirement / rủi ro | Tiền điều kiện | Hành động | Expected | Cleanup |
|---|---|---|---|---|---|
| SY-01 | R-UI-01/02, R-UI-04, R-STATE-02, R-PAY-03, R-API-05 | Seed sạch, 09:00 | Khách báo giá 5 km, chọn VíXanh, đặt; tài xế (context riêng) tải, nhận, arrive → pickup → start → complete | Giá `35.000 đ`; nhãn tài xế đúng từng bước; khách thấy D101 và «Hoàn thành»; nút huỷ disabled; API `completed`/`paid`/`35000`/`D101`; C002 đọc nhận 404 | Đóng context tài xế; context khách do runner đóng |
| SY-02 | R-CANCEL-01, R-PAY-05, R-UI-01 | Seed sạch, 09:00 | Khách đặt rồi huỷ ngay khi `searching` | UI «Đã huỷ», «Phí huỷ: 0 đ», nút huỷ disabled; API `cancelled`, `cancellationFee: 0`, `paymentStatus: none` | Reset ở case sau |
| SY-03 | R-UI-01, R-FARE-02 | Seed sạch | Báo giá 5 km, sửa thành 0,1 km, báo giá lại | Báo giá cũ ẩn, nút đặt disabled; response 422 `DISTANCE_OUT_OF_RANGE`; UI hiện lỗi chứa «Quãng đường» | Không tạo trip |

Hai quyết định thiết kế đáng chú ý:

- **Dùng ID thực nhận, không chọn dòng đầu.** SY-01 đọc `trip-id` trên trang khách rồi bấm đúng `accept-<id>` ở trang tài xế, sau đó assert `current-trip-id` bằng ID đó. Dùng `nth(0)` có thể nhận nhầm chuyến của case khác mà test vẫn xanh.
- **C002 là oracle bổ trợ qua API.** Đây không phải hành trình UI đầy đủ của người thứ ba và càng không phải pentest — phần phân quyền có bài riêng (SYS-10).

## 6. Thực hành và giải thích code

Mã: [`articles/system-testing-system-test/e2e/system.spec.ts`](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-system-test). Từ thư mục `examples/toi-noi-testing`, Node ≥ 22.12 và Google Chrome cài sẵn:

```sh
npm ci            # chỉ khi thiếu dependency
npx playwright test --config system-testing.config.ts
npm run typecheck
```

Không cần mở `npm start` trước: runner tự khởi động và dừng server. Expected: 3 test pass, không retry.

### 6.1. Bước 1 — Setup có kiểm kết quả

```ts
test.beforeEach(async ({ request }) => {
  expect((await request.post('/api/test/reset')).status()).toBe(200);
  expect((await request.post('/api/test/clock', { data: { now: '2026-11-04T09:00:00+07:00' } })).status()).toBe(200);
});
```

Cả hai lệnh đều assert HTTP 200. Nếu test API bị tắt, test dừng ở setup với lỗi rõ ràng thay vì chạy trên dữ liệu cũ rồi sinh triệu chứng khó hiểu ở bước 7. Reset DB không tự đặt lại giờ nên cần hai lệnh riêng.

### 6.2. Bước 2 — Đi qua UI và tách vai trò

```ts
async function book(page: Page) {
  await page.goto('/');
  await page.getByTestId('distance').fill('5');
  await page.getByTestId('payment-method').selectOption('vixanh');
  await page.getByTestId('quote-button').click();
  await expect(page.getByTestId('quote-total')).toHaveText('35.000 đ');
  await page.getByTestId('book-button').click();
  await expect(page.getByTestId('trip-status')).toHaveAttribute('data-status', 'searching');
  const id = await page.getByTestId('trip-id').textContent();
  expect(id).toBeTruthy();
  return id!;
}
```

Tài xế dùng `browser.newContext()` riêng để không chia sẻ cookie/storage nếu ứng dụng bổ sung chúng sau này. Nhưng danh tính lab vẫn là mã người dùng trong form — đây **không** phải kiểm đăng nhập. URL tài xế được suy từ origin của trang khách, để file test không bị khoá vào một cổng khác với server đang chạy:

```ts
await driver.goto(new URL('/driver.html', page.url()).href);
await driver.getByTestId('load-available').click();
await driver.getByTestId(`accept-${id}`).click();
await expect(driver.getByTestId('current-trip-id')).toHaveText(id);
await expect(page.getByTestId('trip-driver')).toHaveText('D101');
```

### 6.3. Bước 3 — Chờ điều kiện thay vì sleep

Trang khách hỏi lại server mỗi hai giây (`setInterval(..., 2000)` trong `src/web/customer.js`). Test không ngủ cố định rồi đọc DOM một lần; nó dùng assertion có chờ:

```ts
for (const [action, label] of [
  ['arrive', 'Tài xế đang tới'], ['pickup', 'Đã đón / lấy hàng'],
  ['start', 'Đang chở / giao'], ['complete', 'Hoàn thành'],
]) {
  await driver.getByTestId(`action-${action}`).click();
  await expect(driver.getByTestId('driver-trip-status')).toHaveText(label!);
}
await expect(page.getByTestId('trip-status')).toHaveText('Hoàn thành');
await expect(page.getByTestId('cancel-button')).toBeDisabled();
```

`toHaveText('Hoàn thành')` thử lại tới khi đạt hoặc hết 8 giây. Tám giây là budget của test local, **không** phải SLA cập nhật trạng thái (SPEC R-NFR-04 nói p95 dưới 2–3 giây và đó là phép đo khác). `retries: 0` không mâu thuẫn với assertion chờ: assertion đang chờ một trạng thái trong cùng một lần thực hiện, còn retry ở cấp test là chạy lại toàn bộ nghiệp vụ sau khi đã fail — thứ có thể che lỗi thật.

Ở SY-03, `waitForResponse` được đăng ký **trước** khi click để không bỏ lỡ response đến nhanh:

```ts
const response = page.waitForResponse(r => r.url().endsWith('/api/quotes') && r.request().method() === 'POST');
await page.getByTestId('quote-button').click();
const res = await response;
expect(res.status()).toBe(422);
expect((await res.json()).error.code).toBe('DISTANCE_OUT_OF_RANGE');
await expect(page.getByTestId('error')).toContainText('Quãng đường');
```

Chỉ thấy text lỗi trên trang không chứng minh server đã nhận giá trị mới — trang có thể tự kiểm phía client. Kiểm cả status/code lẫn thông báo UI cho biết lỗi đến từ đâu.

### 6.4. Bước 4 — Kiểm hai mặt của kết quả

```ts
const res = await request.get(`/api/trips/${id}`, { headers: { 'x-user': 'customer:C001' } });
expect(res.status()).toBe(200);
expect((await res.json()).trip).toMatchObject({ id, status: 'completed', paymentStatus: 'paid', fare: 35_000, driverId: 'D101' });
const other = await request.get(`/api/trips/${id}`, { headers: { 'x-user': 'customer:C002' } });
expect(other.status()).toBe(404);
expect((await other.json()).error.code).toBe('TRIP_NOT_FOUND');
```

| Chỉ kiểm… | Lỗi lọt qua |
|---|---|
| API | Polling hỏng, render sai nhãn, nút huỷ vẫn bấm được |
| Nhãn UI | UI hiện «Hoàn thành» sớm trước khi server lưu; chưa thu tiền |
| Cả hai | Vẫn chưa biết provider thật thu đúng — ngoài ranh giới |

SYS-04 đã kiểm sổ gọi ví giả. Bài này không có endpoint ledger, nên `paymentStatus: 'paid'` là trạng thái hệ thống ghi nhận, **không** phải đối soát tiền tại nhà cung cấp.

### 6.5. Kết quả

**Expected:** 3 test pass, không retry, typecheck exit 0.

**Observed — báo cáo SYS-05 ngày 13/11/2026** (Windows, Node v24.15.0, npm 11.12.1, Chrome 157.0.8103.51 headless):

| Kiểm tra | Kết quả thực đo |
|---|---|
| `npx playwright test --config system-testing.config.ts`, lượt cuối | PASS: 3 tests, 11,4 giây, không retry |
| SY-01 | PASS, 5,3 giây |
| SY-02 | PASS, 1,1 giây |
| SY-03 | PASS, 695 ms |
| `npm run typecheck` tại lab | PASS, exit 0 |

**Observed — chạy lại khi biên tập bài, cùng ngày** (Windows, Node v22.18.0, Playwright 1.64.1, Chrome 157.0.8103.78):

```text
Running 3 tests using 1 worker

  ok 1 … › SY-01 · khách đặt, tài xế hoàn tất, hai UI và API thống nhất (5.7s)
  ok 2 … › SY-02 · khách huỷ khi đang tìm tài xế, UI và API cùng ghi nhận phí 0 (540ms)
  ok 3 … › SY-03 · sửa dữ liệu làm vô hiệu báo giá cũ và nhận lỗi nghiệp vụ qua UI (483ms)

  3 passed (18.2s)
```

Tổng thời gian khác nhau giữa hai lượt (11,4 so với 18,2 giây) chủ yếu do khởi động server và máy; SY-01 dài nhất vì phải chờ ít nhất một chu kỳ polling. Không có lỗi sản phẩm được phát hiện. Lượt pass không sinh trace/screenshot vì config chỉ giữ chúng khi fail — bằng chứng ở đây là assertion và output runner.

## 7. Điều tra failure và tiêu chí kết thúc

| Failure | Phân loại ban đầu | Việc kiểm tiếp |
|---|---|---|
| Cổng 3115 bận / thiếu Chrome | Môi trường — blocked | Xem process đang giữ cổng hoặc cài browser; không tự trỏ sang server lạ |
| Báo giá không hiện | UI, mạng hoặc API | Trace: request có đi không, response gì, DOM ra sao — phân biệt click không xảy ra với server lỗi |
| API `completed` nhưng UI khách cũ | Polling/render hoặc lỗi mạng | Các GET lặp trong trace, response và thời điểm DOM đổi |
| Tài xế không thấy chuyến | Seed, bộ lọc hoặc trạng thái | ID vừa tạo, city/vehicle, response danh sách |
| Chạy riêng pass, cả suite fail | Reset / cô lập dữ liệu | Số worker, context chưa đóng, giờ và seed |

**Entry criteria:** môi trường riêng; Chrome và dependency có sẵn; test API bật đúng trong lab; SPEC đúng phiên bản.

**Exit criteria của bài tập:** ba case pass, không skip, không retry; typecheck đạt; giới hạn bằng chứng được ghi. Nếu có lỗi chập chờn, giữ trace và điều tra; không tăng timeout hay bật retry rồi gọi là ổn định.

Một báo cáo lỗi tốt ghi commit, cấu hình DB/browser, dữ liệu, **bước đầu tiên lệch**, expected/actual và artifact. Không cần chép lại mọi click khi trace đã có. Trace có thể chứa payload; khi dùng dữ liệu ngoài lab, quản lý quyền truy cập artifact theo độ nhạy của dữ liệu.

## 8. Mở rộng nâng cao

Các mục dưới đây là **thiết kế mở rộng, chưa được thực thi** trong lab — lab không có broker, nhiều worker hay staging.

### 8.1. Nhất quán cuối cùng và observability

Tình huống: Tới Nơi thật dùng Kafka; `POST /api/trips` trả `accepted` trước khi read model của trang khách cập nhật. Test chờ «Hoàn thành» thỉnh thoảng đỏ.

Cách giải:

1. Lấy deadline từ SLO đã thống nhất (ví dụ R-NFR-04 của SPEC), không tự chọn số.
2. Gắn **correlation ID** từ command tới event và read model; khi fail, tra theo ID thay vì đọc log toàn hệ thống.
3. Lưu lịch sử trạng thái quan sát được kèm thời điểm, để phân biệt «chậm hơn deadline» với «không bao giờ tới».
4. Không tăng sleep để che độ trễ không kiểm soát.

Hành trình lỗi (mất mạng, ví `decline`, báo giá hết hạn) có thể inject qua test control trước hành động — ví giả của lab có các chế độ `decline`, `error`, `timeout`, `lost_response` (R-TEST-03) — để kiểm UI không báo thanh toán thành công sai.

### 8.2. Chạy song song mà không xoá dữ liệu nhau

Một worker hợp với lab nhỏ nhưng thành nút thắt khi suite lớn. Mỗi worker cần tenant/DB riêng hoặc dữ liệu gắn run ID, tài khoản riêng và cleanup theo phạm vi. Reset toàn DB chỉ an toàn khi worker sở hữu độc quyền DB. Browser context tách biệt **không** làm DB tách biệt: hai test vẫn có thể tranh cùng tài xế D101.

### 8.3. System testing phi chức năng

Cùng môi trường có thể kiểm accessibility, compatibility, resilience hay performance, nhưng mỗi thứ cần mục tiêu và oracle riêng. Ba test hiện tại không đo p95/p99, không chứng minh usability hay WCAG. Locator click được nút không có nghĩa người thật hiểu được màn hình. Các phần đó có bài riêng trong lộ trình.

### 8.4. Production parity

Server chạy từ TypeScript source không chứng minh container/build artifact, migration hay reverse proxy đúng. Bước tiếp theo trên staging: deploy artifact được định danh, chạy luồng với payment sandbox và cấu hình tương đương, dùng dữ liệu tổng hợp. Đừng gọi staging «giống production» nếu đã tắt rate limit, auth hoặc cache mà không ghi lại khác biệt.

### 8.5. Chọn hành trình theo khả năng chẩn đoán

Giữ vài hành trình kiếm tiền/huỷ/khôi phục quan trọng ở browser; đẩy tổ hợp rộng xuống API/component. Khi một kịch bản dài có nhiều điểm lỗi độc lập, tách tại ranh giới mục tiêu để failure dễ giải thích. Nhưng đừng dùng API setup để bỏ qua chính bước UI mà mục tiêu cần kiểm.

## 9. Anti-pattern, giới hạn và đánh đổi

**Anti-pattern:**

- `nth(0)` để nhận «chuyến nào đó» — có thể kiểm nhầm dữ liệu.
- Mock toàn bộ API rồi gọi là đã chạy toàn hệ thống.
- Đọc trạng thái từ biến JS nội bộ thay vì giao diện công khai.
- Coi nút disabled là đủ bảo vệ: API vẫn phải tự kiểm quyền và chuyển trạng thái.
- Sleep cố định, retry toàn test, tăng timeout tới khi xanh.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn bằng chứng của SYS-05</span>
<p>Chỉ Chrome local được chạy. Chưa kiểm Safari/Firefox/mobile, GPS, đăng nhập JWT, provider thanh toán thật, mạng chậm, server restart, concurrency hay mọi ô làm vô hiệu báo giá. Đồng hồ là loại dời được và tiếp tục chạy, không đóng băng. PGlite trong một phiên không chứng minh dữ liệu bền sau crash. Ba case pass phản ánh ba hành trình trong ranh giới Hình 1 — không thay hồ sơ nghiệm thu.</p>
</div>

**Đánh đổi chi phí/phạm vi:** mỗi hành trình browser tốn vài giây và nhiều điểm hỏng hơn API test. Ba hành trình ở đây phủ ba rủi ro khó thấy ở cấp thấp hơn với tổng ~11–18 giây. Thêm 24 ca giá lên browser sẽ nhân thời gian mà gần như không thêm rủi ro mới được phủ.

## 10. Bài tập và câu hỏi tự kiểm

### Bài tập

**Bài 1 (cơ bản).** Sau khi báo giá, sửa ô voucher. Viết expected và chỉ ra cách thao tác để test thật sự kiểm sự kiện nhập liệu.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Theo R-UI-01, <code>quote-result</code> phải ẩn và <code>book-button</code> disabled. Dùng <code>fill()</code> trên ô voucher để phát sinh sự kiện <code>input</code> thật (trang gắn <code>invalidateQuote</code> vào sự kiện này). Không gọi thẳng <code>invalidateQuote()</code> qua <code>page.evaluate</code> — như vậy chỉ kiểm hàm, không kiểm việc trang có lắng nghe ô đó hay không.</p>
</details>

**Bài 2 (trung bình).** Nhập mã khách không tồn tại rồi báo giá. Expected là gì, và test này chứng minh được gì về bảo mật?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>API trả 401, UI hiện thông báo lỗi và không hiện báo giá hợp lệ. Test chỉ chứng minh hệ thống từ chối danh tính mô phỏng không hợp lệ; chưa kiểm credential, token hay phiên đăng nhập thật.</p>
</details>

**Bài 3 (trung bình).** Thiết kế hành trình: tài xế D101 huỷ nhận, rồi tài xế khác nhận lại.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Cần: trạng thái quay về <code>searching</code> sau khi tài xế huỷ; tài xế cũ trở lại online; trang khách hiện tài xế mới. Phải phân biệt tài xế huỷ với khách huỷ (khác quy tắc phí). Cần seed có tài xế thứ hai cùng thành phố và loại xe — kiểm SPEC trước khi chọn mã.</p>
</details>

**Bài 4 (nâng cao).** Chuyển suite sang hai worker. Chỉ đổi <code>workers: 2</code> có đủ không?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Không. <code>beforeEach</code> reset toàn DB nên worker này xoá dữ liệu của worker kia. Cần hai server/DB riêng (cổng khác nhau) hoặc chiến lược namespace dữ liệu theo worker, kèm tài khoản riêng để không tranh cùng tài xế, và rate limit tính riêng.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** API trả `paymentStatus: 'paid'` có chứng minh khách thấy chuyến hoàn thành không?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. Polling hoặc render của trang khách có thể hỏng. Cần assertion trên UI khách.</p>
</details>

**Câu 2.** Trang khách hiện «Hoàn thành» có chứng minh nhà cung cấp đã thu đúng tiền không?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. Ví là mô phỏng và nằm ngoài ranh giới bài này; cần kiểm ledger (SYS-04) và system integration với sandbox thật. Chỉ kết luận trong phạm vi đã quan sát.</p>
</details>

**Câu 3.** Vì sao SY-03 đăng ký <code>waitForResponse</code> trước khi click?

<details>
<summary><b>Đáp án</b></summary>
<p>Response có thể về trước khi lệnh chờ được đăng ký nếu đặt sau click; khi đó test treo tới timeout hoặc bắt nhầm response khác. Đăng ký trước đảm bảo bắt đúng POST <code>/api/quotes</code> do click gây ra.</p>
</details>

**Câu 4.** Một bộ Playwright mock mọi request API, chạy hết luồng đặt–hoàn thành và xanh. Có gọi đó là system test của Tới Nơi được không?

<details>
<summary><b>Đáp án</b></summary>
<p>Không theo nghĩa của bài: nó kiểm giao diện với dữ liệu giả, không cung cấp bằng chứng đường browser–server–DB. Có thể gọi là UI/component test của frontend.</p>
</details>

**Câu 5.** Vì sao chọn 09:00 làm giờ chuẩn thay vì 10:59?

<details>
<summary><b>Đáp án</b></summary>
<p>Đồng hồ tiếp tục chạy trong lúc test. Ở 10:59, vài chục giây thao tác có thể đẩy thời điểm báo giá sang 11:00 — giờ cao điểm (R-FARE-04) — làm giá đổi và test đỏ vì lý do không liên quan đến mục tiêu.</p>
</details>

## 11. Tài liệu tham khảo

Các nguồn ngoài được tra cứu 13/11/2026.

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024), mục 2.2.1 Test Levels — https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf. Ví dụ Tới Nơi và cách chọn hành trình là thiết kế riêng của series, không phải yêu cầu của syllabus.
- Playwright — Web server: https://playwright.dev/docs/test-webserver (quản lý server, `reuseExistingServer`).
- Playwright — Assertions: https://playwright.dev/docs/test-assertions (assertion tự chờ khác assertion giá trị tức thời).
- Lab Tới Nơi: `SPEC.md` (R-UI, R-STATE, R-FARE, R-PAY, R-CANCEL, R-API, R-TEST, R-NFR), `src/server.ts`, `src/web/customer.js`, `src/web/driver.js` và bộ test của bài — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-system-test
- Bài liên quan: [TEST-014 System Testing](/articles/system-testing) · [TEST-015 System Integration Testing](/articles/system-integration-testing) · [TEST-016 Acceptance Testing](/articles/acceptance-testing-tong-quan) · [SYS-01 Smoke Test](/articles/system-testing-smoke-test) · [SYS-03 Functional Test](/articles/system-testing-functional-test) · [SYS-04 Integration Test](/articles/system-testing-integration-test)

Bài tiếp theo: SYS-06 · Regression Test (đang biên tập).
