---
slug: system-testing-performance-test
title: "Performance Test: đo đúng câu hỏi trước khi tăng tải"
summary: "Bài thực hành performance testing trên lab Tới Nơi: đặt câu hỏi đo được, chọn ranh giới thời gian, tính p50/p95/p99, throughput, goodput và error rate với mẫu số rõ ràng, phân biệt closed và open model, lưu mẫu gốc và đọc một baseline 900 request trên loopback mà không biến nó thành tuyên bố về công suất production."
category: testing
level: intermediate
tags: [testing, sys-07, performance-testing, latency, percentile, throughput, baseline, slo, non-functional-testing]
language: vi
---

Khách mở Tới Nơi, bấm «Báo giá», chờ, rồi bấm lại. API báo giá trả đúng 35.000 đ, nhưng không ai trong đội trả lời được *nó trả nhanh tới đâu, trong điều kiện nào*. Một người nói «trên máy tôi nhanh lắm», người khác khoe «test đạt 2.000 request/giây» — không ai nói workload, dữ liệu, môi trường hay bao nhiêu request sai. Bài này trả lời: **làm sao biến cảm giác «nhanh/chậm» thành một phép đo có câu hỏi, có mẫu số, có bằng chứng gốc và có giới hạn được công bố**, qua một baseline nhỏ cho `POST /api/quotes` của lab Tới Nơi.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-07</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 1</td></tr>
<tr><th scope="row">Cấp độ</th><td>Trung cấp — biết HTTP, TypeScript, khái niệm test oracle; không cần cài k6</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>Phi chức năng — hiệu năng (time behaviour, resource utilization); thực hiện trên hệ thống lắp ráp qua HTTP</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-regression-test">SYS-06 Regression Test</a>; nền tảng ở <a href="/articles/non-functional-testing-tong-quan">Non-functional testing tổng quan</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): baseline closed model, 3 vòng × (50 warm-up + 300 đo) cho <code>POST /api/quotes</code>, oracle tiền 35.000 đ, lưu 900 mẫu gốc (R-API-07, R-FARE-01…08, R-NFR-03) — <code>examples/toi-noi-testing/articles/system-testing-performance-test/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>Grafana k6 (Load test types, Metrics, Open vs closed models); Node.js Performance measurement APIs; <code>SPEC.md</code> của lab</td></tr>
</tbody>
</table>
</div>

## 1. Tình huống và mục tiêu

### 1.1. Vì sao «nhanh trên máy tôi» không phải kết luận

Một câu khẳng định hiệu năng chỉ có nghĩa khi trả lời được năm câu hỏi: **đo cái gì** (endpoint nào, ranh giới thời gian từ đâu tới đâu), **dưới tải nào** (bao nhiêu request, gửi theo mô hình nào), **trên môi trường nào** (máy, DB, mạng), **đúng hay sai** (bao nhiêu response đạt oracle nghiệp vụ) và **so với mục tiêu nào** (ngưỡng đặt trước hay đặt sau khi thấy số). «2.000 request/giây» thiếu cả năm. Con số đó có thể là 2.000 response lỗi 500 trả rất nhanh, đo trong 0,2 giây, trên một máy dev không có DB.

Ở [SYS-06](/articles/system-testing-regression-test) ta bảo vệ hành vi đúng sau thay đổi. Bài này giữ nguyên nguyên tắc đó và thêm một chiều: **đúng và trong bao lâu**. Một response nhanh nhưng tổng tiền sai không phải thành công.

### 1.2. Đối tượng và điều kiện tiên quyết

Tester hoặc developer đã đọc SYS-01…06, biết đọc TypeScript, hiểu HTTP request/response và khái niệm oracle. Không cần k6 hay JMeter: lab dùng một script Node để mọi bước đo đều nhìn thấy được trong mã.

### 1.3. Mục tiêu đo được

Sau bài này, bạn có thể:

1. Viết **câu hỏi hiệu năng** đủ endpoint, workload, môi trường, oracle và ngưỡng — trước khi chạy.
2. Định nghĩa **latency** theo ranh giới đã chọn và giải thích nó gồm/không gồm những gì.
3. Tính **p50/p95/p99 theo nearest-rank**, throughput, goodput và error rate từ mẫu gốc, và chỉ ra vì sao không được lấy trung bình các p95.
4. Phân biệt **closed model** với **open model** và nhận ra khi nào kết quả có nguy cơ coordinated omission.
5. Đọc một baseline local và viết kết luận **không vượt quá** bằng chứng.

## 2. Khái niệm: performance là một nhóm câu hỏi

Performance testing đánh giá đặc tính thời gian, thông lượng và sử dụng tài nguyên **dưới điều kiện xác định**. Đây là tên chung; các biến thể khác nhau ở câu hỏi mà chúng trả lời:

| Biến thể | Câu hỏi | Ở series này |
|---|---|---|
| Performance baseline | Trong cấu hình và workload này, hệ thống đang có đặc tính gì? | SYS-07 (bài này) |
| Load testing | Ở tải dự kiến, có đạt mục tiêu không? | [SYS-08](/articles/system-testing-load-test) |
| Stress testing | Vượt tải thường hoặc gần giới hạn, hệ thống suy giảm thế nào và hồi phục ra sao? | [SYS-09](/articles/system-testing-stress-test) |
| Spike testing | Tải tăng đột ngột gây hậu quả gì? | Mở rộng của SYS-09 |
| Soak testing | Chạy dài có tích luỹ leak, backlog, mất ổn định không? | Mở rộng, chưa có lab |

Tài liệu k6 gọi một lượt tải thấp để kiểm script là *smoke test* — khác hoàn toàn smoke build ở [SYS-01](/articles/system-testing-smoke-test). Khi đọc tài liệu công cụ, luôn hỏi lại mục tiêu thay vì so tên.

Ba điểm hay nhầm:

- **Đối tượng đo quyết định kết luận.** Microbenchmark hàm tính cước không đại diện latency HTTP có DB. Một bài HTTP không chứng minh thời gian render UI. Chọn đối tượng trước, chọn công cụ sau.
- **Correctness vẫn là một phần của phép đo.** Response 200 nhanh nhưng `total` sai phải tính là lỗi. Tách lỗi transport (timeout, mất kết nối), HTTP (status) và nghiệp vụ (oracle) khi đọc kết quả.
- **Không phải mọi 4xx là hỏng.** Với endpoint có failure hợp lệ theo nghiệp vụ (ví dụ 422 khi quãng đường ngoài khoảng), định nghĩa *expected mix* trước khi chạy. Bài này cố ý dùng một input hợp lệ duy nhất nên mọi response khác 200/35.000 đều là lỗi.

### 2.1. Test basis: SPEC nói gì về hiệu năng?

SPEC của lab có mục R-NFR với các «mục tiêu khởi tạo để thảo luận, chưa phải cam kết». Liên quan trực tiếp bài này:

- **R-NFR-03** — p95 của `POST /api/quotes` dưới 500 ms, *trong hệ thống đích phụ thuộc bản đồ MapViet*.
- **R-NFR-07** — mức tải tham chiếu lúc cao điểm khoảng 70 chuyến tạo mới/giây và 30.000 truy vấn đọc/giây toàn hệ thống; SPEC ghi rõ đây *không phải mục tiêu* mà là dữ liệu dựng mô hình tải (dùng ở SYS-08).

SPEC cũng nói thẳng: bản lab chạy một tiến trình trên một máy nên chỉ dùng để minh hoạ phương pháp đo, không dùng để kết luận hệ thống đạt hay không. Đó là lý do bài này **không** kiểm R-NFR-03. Lab không có MapViet, không có DB server, không có mạng thật — đo nó rồi gọi là «đạt R-NFR-03» là sai đối tượng.

## 3. Định nghĩa phép đo và mẫu số

Một con số hiệu năng chỉ có nghĩa khi định nghĩa đi kèm. Bài chốt các định nghĩa sau và script tuân theo chúng.

**Latency** là thời gian một lần thực hiện trong ranh giới đã chọn. Trong lab, đồng hồ bắt đầu ngay trước `fetch` và dừng sau khi đọc/parse JSON và kiểm oracle. Vì vậy latency gồm client, loopback, server và xử lý response — không chỉ thời gian hàm backend, và không trùng hoàn toàn `http_req_duration` của k6.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 270" role="img" aria-label="Ranh giới đo latency của SYS-07: một tiến trình Node chứa cả client đo và server Express với kho bộ nhớ. Đồng hồ performance.now bắt đầu trước fetch, request đi qua loopback 127.0.0.1 tới API, use case tính cước và lưu quote, response quay lại, client parse JSON và kiểm oracle HTTP 200, total 35000, quoteId là chuỗi, rồi đồng hồ dừng. Bên dưới là closed model: worker chỉ gửi request tiếp khi request trước xong, concurrency bằng 1. Ngoài ranh giới: MapViet, PostgreSQL server, TLS, mạng WAN, browser render.">
<defs>
<marker id="sys07-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<rect x="16" y="16" width="888" height="170" rx="10" class="f-bg s-pk dash"/>
<text x="32" y="38" class="t-b">Một tiến trình Node — client đo và server dùng chung CPU, event loop</text>
<rect x="36" y="60" width="150" height="50" rx="8" class="f-phys-soft s-phys"/><text x="111" y="82" text-anchor="middle" class="t-b">t0 = now()</text><text x="111" y="100" text-anchor="middle" class="t-small">trước fetch</text>
<rect x="216" y="60" width="140" height="50" rx="8" class="f-phys-soft s-phys"/><text x="286" y="82" text-anchor="middle">loopback</text><text x="286" y="100" text-anchor="middle" class="t-small">127.0.0.1, cổng ngẫu nhiên</text>
<rect x="386" y="60" width="170" height="50" rx="8" class="f-pk-soft s-pk"/><text x="471" y="82" text-anchor="middle">API Express</text><text x="471" y="100" text-anchor="middle" class="t-small">POST /api/quotes</text>
<rect x="586" y="60" width="150" height="50" rx="8" class="f-pk-soft s-pk"/><text x="661" y="82" text-anchor="middle">cước + lưu quote</text><text x="661" y="100" text-anchor="middle" class="t-small">kho bộ nhớ</text>
<rect x="766" y="60" width="122" height="50" rx="8" class="f-phys-soft s-phys"/><text x="827" y="82" text-anchor="middle" class="t-b">t1 = now()</text><text x="827" y="100" text-anchor="middle" class="t-small">sau parse + oracle</text>
<line x1="186" y1="85" x2="212" y2="85" class="s-phys" stroke-width="1.6" marker-end="url(#sys07-arrow)"/>
<line x1="356" y1="85" x2="382" y2="85" class="s-pk" stroke-width="1.6" marker-end="url(#sys07-arrow)"/>
<line x1="556" y1="85" x2="582" y2="85" class="s-pk" stroke-width="1.6" marker-end="url(#sys07-arrow)"/>
<line x1="736" y1="85" x2="762" y2="85" class="s-phys" stroke-width="1.6" marker-end="url(#sys07-arrow)"/>
<line x1="111" y1="128" x2="827" y2="128" class="s-pk" stroke-width="1.6" marker-start="url(#sys07-arrow)" marker-end="url(#sys07-arrow)"/>
<text x="469" y="148" text-anchor="middle" class="t-b">latency của một mẫu = t1 − t0 (ms)</text>
<text x="469" y="170" text-anchor="middle" class="t-small">oracle: HTTP 200 · total 35.000 · quoteId là chuỗi · timeout 2.000 ms</text>
<rect x="36" y="200" width="380" height="30" rx="6" class="f-phys-soft s-phys"/><text x="226" y="220" text-anchor="middle" class="t-small">closed model: mẫu n+1 chỉ gửi khi mẫu n xong (concurrency 1)</text>
<rect x="440" y="200" width="448" height="30" rx="6" class="f-warn-soft s-warn"/><text x="664" y="220" text-anchor="middle" class="t-small">ngoài ranh giới: MapViet · PostgreSQL server · TLS · WAN · browser render</text>
<text x="460" y="258" text-anchor="middle" class="t-small t-warn">CPU và RSS đo cho cả tiến trình — không tách được phần của server</text>
</svg>
</div>
<figcaption><b>Hình 1 · Ranh giới đo của SYS-07.</b> Đồng hồ bao trùm client, loopback, server và bước kiểm oracle. Vì client và server chung một tiến trình, mọi số CPU/bộ nhớ là của cả hai. Hàng màu cam là những thứ R-NFR-03 cần nhưng lab không có.</figcaption>
</figure>

**Throughput hoàn tất** = số attempt hoàn tất / thời gian cửa sổ đo. **Goodput** — trong script là `validRps` — = số request đạt oracle / thời gian cửa sổ đo. **Error rate** = số attempt không đạt oracle / tổng attempt đo. Khi không có lỗi, throughput bằng goodput; khi server trả lỗi nhanh, throughput cao có thể che chất lượng thấp. Báo cáo luôn cần cả hai.

**Percentile** p95 là giá trị mà khoảng 95% mẫu không vượt quá, theo một phương pháp tính cụ thể. Script dùng *nearest-rank*: sắp N mẫu tăng dần, lấy phần tử ở vị trí ⌈0,95 × N⌉ (đánh số từ 1). Với N = 300: p95 là phần tử thứ 285, p99 là phần tử thứ 297. p99 trên 300 mẫu chỉ dựa vào 3 mẫu phía đuôi — nên p99 dao động mạnh giữa các lượt, kém ổn định hơn so với một thí nghiệm dài.

Hai quy tắc đi kèm:

- **Không lấy trung bình các p95.** Muốn p95 của 900 request phải gộp mẫu gốc (hoặc histogram tương thích) rồi tính lại. Mục 7.3 có ví dụ bằng chính dữ liệu của bài.
- **Mean thấp không loại trừ đuôi chậm.** Script ghi p50/p95/p99/max và toàn bộ mẫu, kèm status và lỗi của từng attempt.

## 4. Thiết kế trước khi chạy

### 4.1. Câu hỏi của bài

> Một request báo giá gửi nối tiếp qua HTTP loopback tới app dùng kho bộ nhớ có trả đúng kết quả không, và phân bố thời gian quan sát được là gì?

Đây là **baseline nhập môn**, không phải capacity test, không phải load test. Câu hỏi được viết trước; mọi con số sau đó chỉ trả lời câu hỏi này.

### 4.2. Bảng thiết kế

| Thành phần | Giá trị | Lý do |
|---|---|---|
| Endpoint | `POST /api/quotes` (R-API-07) | Endpoint đầu tiên khách chạm, không bị rate limit theo SPEC |
| Input | C001, `ride`, SG, `bike`, 5 km, không voucher | Một input cố định để phân bố không lẫn nhiều loại request |
| Oracle | HTTP 200, `total` 35.000, `quoteId` là chuỗi | 12.000 + 4.500 × 5 = 34.500, làm tròn lên 35.000 (R-FARE-01, R-FARE-08) — tính trên giấy |
| Giờ / thời tiết | 09:00 VN 04/11/2026 cố định (`FixedClock`), không mưa | Ngoài cao điểm, giá không đổi giữa các mẫu |
| Storage | Kho bộ nhớ; mỗi request vẫn lưu quote (R-QUOTE-01) | Đường code có ghi dữ liệu, không chỉ tính toán |
| Mô hình | Closed, concurrency 1, không think time | Câu hỏi về một luồng nối tiếp |
| Số vòng | 3 vòng trong cùng tiến trình; mỗi vòng 50 warm-up + 300 đo | Thấy được biến thiên giữa vòng |
| Dữ liệu | Reset trước warm-up, reset lần nữa trước khi đo | Quote của warm-up không làm lớn dataset đo |
| Timeout | 2.000 ms mỗi request | Request treo vẫn kết thúc và được tính là lỗi |
| Gate minh hoạ | Mỗi vòng: error rate = 0 và p95 < 1.000 ms | Đặt trước khi đo; xem cảnh báo bên dưới |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Ngưỡng 1.000 ms là giả định bài tập</span>
<p>Gate p95 &lt; 1.000 ms được đặt <b>trước</b> phép đo để có điều kiện pass/fail cho script. Nó không phải SLO của Tới Nơi, không phải chuẩn ngành, và cũng không phải R-NFR-03 (500 ms, trong hệ thống đích có MapViet). Đừng tạo SLO bằng cách lấy số vừa đo cộng một tỷ lệ tuỳ ý: mục tiêu phải đến từ nhu cầu người dùng và test basis, không từ những gì máy vừa làm được.</p>
</div>

### 4.3. Warm-up và reset: loại bỏ gì, giữ lại gì

Warm-up cho đường code, kết nối và JIT chạy nóng trước khi đo. Reset sau warm-up loại 50 quote của warm-up khỏi dataset đo. Nhưng reset **không** reset JIT, heap hay cache của tiến trình. Ba vòng là ba cửa sổ trong *cùng* một tiến trình, không phải ba lần khởi động độc lập. Cần ghi rõ điều này khi diễn giải xu hướng «vòng sau nhanh hơn».

### 4.4. Entry và exit criteria

- **Entry:** functional smoke đúng (SYS-01 xanh); môi trường, dữ liệu và câu hỏi đã ghi; không chủ động chạy tác vụ nặng khác cùng lúc.
- **Exit của bài tập:** script hoàn tất đủ mẫu; mẫu gốc được lưu; không có lỗi oracle; gate minh hoạ đạt; mọi giới hạn được công bố. Nếu không đạt, giữ số liệu và để exit code khác 0 — không nới ngưỡng để làm xanh.

## 5. Closed model và coordinated omission

Worker của lab chỉ gửi request tiếp theo khi request trước đã hoàn tất. Khi server chậm đi, tốc độ gửi **tự giảm theo**. Đó là *closed model*: hợp với câu hỏi «một luồng làm việc nối tiếp trải nghiệm gì», nhưng không mô phỏng dòng khách độc lập ngoài đời — khách không chờ khách khác xong mới bấm.

*Open model* lên lịch các arrival độc lập với thời gian phản hồi. Khi service chậm, hàng đợi hoặc số request đồng thời tăng lên, hoặc generator không tạo được đủ tải dự kiến (và phải báo điều đó). Nếu mục tiêu là giữ một arrival rate mà lại dùng closed model, kết quả bỏ qua phần thời gian chờ lẽ ra đã tích luỹ khi tải vẫn tiếp tục tới. Hiện tượng này thường được thảo luận dưới tên **coordinated omission**: chính lúc hệ thống chậm nhất, generator gửi ít nhất, nên phân bố đo được trông đẹp hơn thực tế.

| | Closed (bài này) | Open (SYS-08) |
|---|---|---|
| Biến điều khiển | Số worker / concurrency | Arrival rate (request/giây) |
| Server chậm thì | Tốc độ gửi giảm | Hàng đợi / concurrency tăng |
| Throughput là | **Output** của thí nghiệm | Input mong muốn; cần so offered với achieved |
| Hợp với câu hỏi | Trải nghiệm của một luồng nối tiếp | Hệ thống dưới dòng khách độc lập |

Concurrency 1 cũng **không** phải «một người dùng thật»: không có think time, không có hành trình nhiều endpoint, không có độ trễ mạng ngoài loopback. Throughput thu được là kết quả của thí nghiệm này — không phải offered RPS, càng không phải công suất tối đa.

## 6. Thực hành có thể chạy lại

Mã: [`articles/system-testing-performance-test/baseline.ts`](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-performance-test). Từ thư mục `examples/toi-noi-testing`, Node ≥ 22.12:

```sh
npm ci            # chỉ khi thiếu dependency
npx tsx articles/system-testing-performance-test/baseline.ts
npm run typecheck
```

Không cần mở server trước. Script lắp app thật với kho bộ nhớ, mở cổng ngẫu nhiên trên `127.0.0.1` và **không nhận URL bên ngoài** — không thể vô tình bắn tải vào production. Server được đóng trong `finally`.

### 6.1. Lắp hệ thống và cố định thời gian nghiệp vụ

```ts
const system = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
const server = createApp(system.deps, { logErrors: false }).listen(0, '127.0.0.1');
await once(server, 'listening');
```

`FixedClock` cố định **giờ nghiệp vụ** để giá không đổi. Nó không được dùng để đo thời gian: đồng hồ đứng yên thì mọi latency bằng 0. Hai loại đồng hồ có hai vai trò khác nhau.

### 6.2. Một mẫu đo: thời gian, status, oracle và lỗi

```ts
async function sample() {
  const start = performance.now();
  let status = 0;
  let valid = false;
  let error: string | null = null;
  try {
    const res = await fetch(url, {
      method: 'POST', headers: { 'content-type': 'application/json', 'x-user': 'customer:C001' },
      body: JSON.stringify({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 }),
      signal: AbortSignal.timeout(2_000),
    });
    status = res.status;
    const body = await res.json() as { quote?: { total?: number; quoteId?: string } };
    valid = status === 200 && body.quote?.total === 35_000 && typeof body.quote.quoteId === 'string';
    if (!valid) error = 'HTTP_OR_BUSINESS_ORACLE';
  } catch (cause) { error = cause instanceof Error ? cause.name : 'UNKNOWN'; }
  return { ms: performance.now() - start, status, valid, error };
}
```

Ba chi tiết đáng chú ý:

- `performance.now()` là đồng hồ đơn điệu cho khoảng thời gian, không bị ảnh hưởng khi giờ hệ thống bị chỉnh.
- Lỗi transport rơi vào `catch` nhưng **vẫn trả về một mẫu** có `ms`, `valid: false` và tên lỗi. Request timeout không bị bỏ khỏi phân bố mà không báo lỗi.
- `AbortSignal.timeout(2_000)` là deadline, không phải trần của số đo. Nếu event loop nghẽn, thời gian quan sát có thể vượt 2.000 ms; script không «kẹp» số đo về 2.000.

### 6.3. Warm-up có oracle, reset, rồi đo

```ts
await system.reset();
for (let i = 0; i < 50; i++) {
  if (!(await sample()).valid) throw new Error('Warm-up không đạt oracle; dừng đo');
}
await system.reset(); // warm-up không làm tăng kích thước dataset đo
```

Nếu warm-up sai oracle, script dừng ngay. Đo thời gian của một hệ thống đang trả sai là đo nhầm đối tượng.

### 6.4. Tính percentile và gate

```ts
const sorted = samples.map(s => s.ms).sort((a, b) => a - b);
const percentile = (p: number) => sorted[Math.ceil(p * sorted.length) - 1]!;
const failures = samples.filter(s => !s.valid).length;
```

`sorted` gồm **mọi attempt đo**, kể cả attempt lỗi (`latencyMsAllAttempts`). Gate được áp từng vòng: `failures === 0 && percentile(0.95) < 1_000`; nếu một vòng không đạt, `process.exitCode = 1`.

### 6.5. Đầu ra và tài nguyên

Script ghi `reports/performance/sys07-baseline.json` (thư mục `reports/` không được commit) gồm thời điểm UTC, thông tin máy, bảng thiết kế, summary và toàn bộ 900 mẫu. Bản evidence của lượt được báo cáo là `baseline-observed.json`, commit cùng lab; chạy lại **không** ghi đè nó.

CPU là delta user + system của **cả tiến trình** (client và server) — `combinedCpuMs`. RSS là ảnh chụp đầu và cuối mỗi vòng, **không** phải peak. Không suy ra memory leak chỉ từ việc RSS tăng: V8 có thể giữ bộ nhớ đã cấp phát, và mỗi vòng tích luỹ 300 quote theo thiết kế. Muốn chứng minh leak cần chạy dài, có bằng chứng heap/GC và so ở trạng thái dữ liệu tương đương.

## 7. Kết quả thực đo và cách đọc

### 7.1. Expected

Script hoàn tất exit 0; 3 vòng × 300 mẫu; 0 lỗi oracle; gate minh hoạ đạt cả 3 vòng; file JSON có đủ mẫu. Script **không** assert các con số latency cụ thể — chúng là quan sát, không phải kỳ vọng.

### 7.2. Observed — lượt được báo cáo

Lượt ngày 13/11/2026, kết thúc 20:52:47 giờ Việt Nam (13:52:47 UTC trong JSON). Windows 10.0.26300, Node v24.15.0, Intel Core i7-10700K 3,80 GHz, 16 logical CPU, RAM ~31,9 GiB. Không ghi nhận CPU nền của ứng dụng khác — đó là một hạn chế kiểm soát môi trường. Lệnh exit 0; 1.050 request (150 warm-up + 900 đo), không lỗi. `npm run typecheck` đạt.

| Vòng | Mẫu | Lỗi | p50 ms | p95 ms | p99 ms | Max ms | Thời gian đo ms | Valid RPS | CPU chung ms |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 300 | 0 | 0,688 | 1,139 | 1,642 | 5,408 | 226,1 | 1.326,9 | 313 |
| 2 | 300 | 0 | 0,551 | 0,950 | 1,771 | 6,452 | 188,4 | 1.592,3 | 234 |
| 3 | 300 | 0 | 0,440 | 0,691 | 1,331 | 6,317 | 154,8 | 1.938,0 | 218 |

RSS đầu/cuối (byte): vòng 1 139.264.000 / 164.614.144; vòng 2 171.724.800 / 177.557.504; vòng 3 178.257.920 / 182.837.248. Mọi response là HTTP 200.

### 7.3. Đọc bảng mà không nói quá

**Gate đạt, nhưng cửa sổ rất ngắn.** Mỗi vòng chỉ kéo dài 0,15–0,23 giây. Các giá trị RPS là số *chuẩn hoá* từ một cửa sổ ngắn; chúng không chứng minh duy trì được tốc độ đó trong một giây, một giờ, hay với DB và mạng thật. p95 dưới 1,2 ms không chứng minh gì về R-NFR-03.

**Vòng sau nhanh hơn — chưa biết vì sao.** p50 giảm từ 0,688 xuống 0,440 ms. Có thể do JIT, tái sử dụng kết nối, cache hoặc nhiễu máy; dữ liệu này chưa cô lập nguyên nhân nên không kết luận «phần mềm đã tối ưu». Cũng không được chọn riêng vòng 3 làm đại diện rồi bỏ hai vòng còn lại.

**Đuôi tồn tại dù median nhỏ.** Max 5–6 ms, gấp gần 10 lần p50. Tính lại từ `baseline-observed.json`: mỗi vòng có 2–3 mẫu vượt 2 ms. Với 300 mẫu, p99 chỉ là mẫu thứ 297 — thêm hoặc bớt một mẫu chậm là p99 đổi rõ rệt.

**Ví dụ không lấy trung bình p95 — bằng chính dữ liệu này.** Trung bình ba p95 là (1,139 + 0,950 + 0,691) / 3 ≈ **0,927 ms**. Gộp 900 mẫu gốc rồi tính nearest-rank, p95 thật của cả lượt là **0,990 ms**. Chênh lệch ở đây nhỏ vì ba vòng gần nhau; khi các vòng có phân bố khác hẳn (ví dụ một vòng 10 ms, một vòng 100 ms), trung bình các p95 có thể sai rất xa. Hai con số trên được tính lại từ file evidence đã lưu, không phải phép đo mới.

**CPU lớn hơn wall time không phải lỗi.** Vòng 1 đo 226 ms wall nhưng `combinedCpuMs` 313: tiến trình Node có hoạt động trên nhiều thread (GC, I/O). Và đó là CPU của client **cộng** server — không phải «server tốn 313 ms CPU».

### 7.4. Observed — chạy lại khi biên tập bài, không dùng để so sánh

Khi viết bài (13/11/2026, 15:47 UTC; Windows, Node v22.18.0), script được chạy lại **chỉ để xác nhận nó còn thực thi**. Lúc đó máy đang chạy song song nhiều bộ test khác, tức vi phạm entry criteria của chính bài này:

| Vòng | Mẫu | Lỗi | p95 ms | Valid RPS | Gate |
|---|---:|---:|---:|---:|---|
| 1 | 300 | 0 | 2,708 | 759 | đạt |
| 2 | 300 | 0 | 1,978 | 917 | đạt |
| 3 | 300 | 0 | 3,226 | 590 | đạt |

Exit 0, không lỗi oracle — script vẫn đúng. Nhưng p95 cao gấp 2–4 lần và RPS thấp hơn một nửa so với lượt được báo cáo. Đây **không** phải regression của phần mềm: đã đổi cùng lúc phiên bản Node và mức tải nền. Chính sự chênh lệch này là bài học: hai lượt chỉ so được khi workload, cấu hình và môi trường giống nhau, và khi tải nền được kiểm soát. Bảng 7.2 là bằng chứng của bài; bảng 7.4 chỉ là kiểm tra khả năng thực thi.

## 8. Điều tra và ra quyết định

| Quan sát | Giả thuyết | Bằng chứng cần thêm |
|---|---|---|
| RPS tăng nhưng `validRps` giảm | Server trả lỗi nhanh | Phân rã status/error, oracle nghiệp vụ |
| p99 tăng, median ổn | GC, lock, scheduler, dependency chậm | Trace/profile và timeline tài nguyên |
| CPU cao | Client hoặc server là nút thắt | Tách generator sang tiến trình/máy khác, đo từng phía |
| RSS tăng | Quote tích luỹ hoặc heap giữ allocation | Heap/GC, chạy dài hơn, so sau cleanup |
| Timeout tăng | Service hoặc generator nghẽn | Event loop delay, socket, hàng đợi DB, tải thực đạt |
| Hai lượt lệch nhau nhiều | Môi trường hoặc cấu hình khác | Diff phiên bản runtime, tải nền, cấu hình máy |

Ví dụ một kết luận đúng mức cho lượt 7.2:

> Trong lab bộ nhớ, loopback, closed model concurrency 1, 900 mẫu `POST /api/quotes` đều đạt oracle, p95 từng vòng 0,69–1,14 ms, đạt gate minh hoạ. Kết quả là baseline phương pháp; không đánh giá R-NFR-03, không phải công suất, chưa kiểm DB server, mạng thật hay tải đồng thời.

Ví dụ một kết luận **sai**: «API báo giá chịu được 1.900 RPS với p95 dưới 1 ms, đạt SLO.» Câu này sai ở cả mô hình tải (closed, không phải arrival rate), độ dài (0,15 giây), môi trường (không DB, không mạng) và mục tiêu (gate bài tập, không phải SLO).

## 9. Mở rộng nâng cao

Các mục dưới đây là **thiết kế mở rộng chưa được thực thi** trong lab.

### 9.1. Đo trên môi trường đại diện

Tình huống: đội muốn biết `POST /api/quotes` có đạt R-NFR-03 không trước đợt khuyến mãi.

Cách giải: tách load generator khỏi server (máy hoặc ít nhất tiến trình riêng); dùng PostgreSQL server với kích thước dữ liệu, index và connection pool đại diện; thay bản đồ giả bằng MapViet sandbox hoặc một stub có độ trễ được đo từ thực tế và ghi rõ là stub. Ghi phiên bản build, cấu hình, CPU/RAM, mạng, trạng thái cache, dependency. Chạy đủ dài để thấy steady state, và kiểm generator **không** bão hoà trước server — nếu CPU generator 100%, con số là giới hạn của generator.

### 9.2. SLI/SLO và workload mix

Một mục tiêu dùng được có đối tượng, cửa sổ thời gian, mức tải và tiêu chí lỗi. Ví dụ *giả định* để luyện viết: «báo giá hợp lệ p95 < 300 ms ở 50 arrival/giây trong 10 phút, lỗi < 1%». Các số này không phải cam kết sản phẩm. Khi có nhiều loại request (ride/parcel/food), đo và báo theo từng nhóm: percentile gộp có thể che một nhóm ít request nhưng rất chậm — ví dụ food với danh sách món dài.

### 9.3. Độ tin cậy thống kê

Ba vòng ngắn không đủ ước lượng tail latency ổn định. Tăng thời lượng và số mẫu, lặp ở điều kiện có kiểm soát (nhiều tiến trình độc lập, có thể nhiều máy), công bố biến thiên giữa các lượt. Không gộp số của các cấu hình khác nhau thành một SLO. So trước/sau một thay đổi cần cùng workload và môi trường; nếu đổi nhiều yếu tố một lúc như mục 7.4, không thể quy nguyên nhân cho yếu tố nào.

### 9.4. Little's Law để kiểm tính hợp lý

Trong điều kiện ổn định, L ≈ λW: số request trung bình trong hệ thống bằng throughput nhân thời gian trung bình trong cùng ranh giới. Với closed concurrency 1 và không think time, L = 1 nên λ ≈ 1/W. Kiểm bằng vòng 3: mean latency tính từ mẫu gốc là 0,515 ms, 1/0,000515 ≈ 1.940/giây — khớp `validRps` 1.938. Nếu hai số này lệch xa, có thể script tính sai cửa sổ hoặc có thời gian chết giữa các request. Little's Law dùng để phát hiện số liệu không nhất quán, không thay phép đo, và càng không cho phép nhân 1.938 với 16 CPU để «dự đoán công suất».

### 9.5. Profiling và tối ưu có kiểm chứng

Khi đã xác định một nút thắt, thay **một** yếu tố, chạy lại cùng bài đo và chạy lại functional/regression. Một «tối ưu» bỏ bước lưu quote sẽ làm latency đẹp hơn nhưng phá R-QUOTE-01 và luồng đặt từ quote. Theo dõi đồng thời độ đúng, tail latency, error rate và tài nguyên. Không dùng thời gian chạy suite Vitest làm proxy cho latency sản phẩm.

## 10. Anti-pattern, giới hạn và đánh đổi

**Anti-pattern:**

- Đo `/api/health` rồi suy ra khả năng báo giá hay tạo chuyến.
- Chỉ báo trung bình; hoặc lấy trung bình các p95.
- Bỏ request timeout/lỗi ra khỏi percentile cho đồ thị đẹp.
- Không kiểm body — đo tốc độ trả lỗi.
- Chạy generator cùng tiến trình rồi gọi CPU là «CPU server».
- Dùng vài trăm request trong 0,2 giây để tuyên bố công suất production.
- Đặt ngưỡng sau khi thấy số; nới ngưỡng khi đỏ.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn bằng chứng của SYS-07</span>
<p>Lab không có DB server, MapViet, TLS, mạng WAN, payment provider ngoài, nhiều người dùng hay tải mở. Không đo thời gian khởi động, không đo peak RSS, không chạy steady state dài, không chạy trên nhiều tiến trình/máy độc lập. Client và server chung tiến trình nên CPU/RSS không tách được. Không thực hiện Load hay Stress Test. Không có lỗi sản phẩm được phát hiện ở lượt đo — điều đó <b>không</b> có nghĩa là không có bottleneck.</p>
</div>

**Đánh đổi chi phí/phạm vi:** baseline này chạy dưới một giây và không cần hạ tầng, nên phù hợp để luyện phương pháp và phát hiện thay đổi thô (ví dụ một commit làm p95 local tăng 10 lần). Đổi lại, nó không trả lời được câu hỏi nào về production. Một phép đo đại diện cần môi trường riêng, dữ liệu, thời gian chạy và người đọc kết quả — tốn hơn nhiều, nên chỉ làm khi câu hỏi đáng giá đó.

## 11. Bài tập và câu hỏi tự kiểm

### Bài tập

**Bài 1 (cơ bản).** 100 request trong 2 giây, 4 response sai oracle. Tính throughput, goodput, error rate. Chỉ báo «50 RPS» bỏ mất điều gì?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Throughput = 100 / 2 = 50 RPS; goodput = 96 / 2 = 48 RPS; error rate = 4 / 100 = 4%. Chỉ báo 50 RPS che mất chất lượng kết quả: 4% khách nhận báo giá sai hoặc lỗi. Báo cáo cần đủ ba số và cách phân loại lỗi.</p>
</details>

**Bài 2 (trung bình).** Hai vòng có p95 lần lượt 10 ms và 100 ms, mỗi vòng 300 mẫu. p95 của 600 mẫu có bằng 55 ms không?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Không thể biết từ hai số đó. p95 của 600 mẫu là phần tử thứ 570 sau khi gộp và sắp xếp mẫu gốc; nó phụ thuộc toàn bộ phân bố. Nếu vòng 2 chậm đồng đều, p95 gộp có thể gần 100 ms. Cần mẫu gốc hoặc histogram cùng định nghĩa bucket. Mục 7.3 cho thấy ngay cả với ba vòng gần nhau, trung bình p95 (0,927 ms) đã khác p95 gộp (0,990 ms).</p>
</details>

**Bài 3 (trung bình).** Tăng số mẫu đo lên 3.000 mỗi vòng. Bài có tự trở thành load test ở tải dự kiến không?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Không. Vẫn là closed model concurrency 1 — chỉ dài hơn. Load test cần mô hình tải (arrival rate hoặc số VU có pacing), mức tải dự kiến lấy từ test basis (ví dụ dựng từ R-NFR-07), thời gian ramp-up/steady/cool-down và tiêu chí đạt. Tổng số request không định nghĩa được tải.</p>
</details>

**Bài 4 (nâng cao).** Thiết kế phép so sánh latency báo giá giữa kho bộ nhớ và PostgreSQL server.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Generator ở tiến trình riêng; server chạy hai lần với cùng build, chỉ đổi storage. PostgreSQL có schema/index của lab, dataset đại diện, connection pool cấu hình rõ. Cùng input mix, cùng warm-up, cùng số vòng, nhiều lượt độc lập. Đo CPU/RSS phía server và phía generator riêng. Nếu DB ở máy khác, mạng cũng đổi — phải đo hoặc tách phần mạng, không quy mọi chênh lệch cho DB. Kết luận ghi rõ cấu hình hai phía.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** p95 < 1 ms trong lab có chứng minh màn hình báo giá hiện nhanh cho khách không?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. Ranh giới đo dừng sau khi parse JSON trong tiến trình Node; không gồm mạng thật, trình duyệt, render hay MapViet. Cần phép đo khác với ranh giới khác.</p>
</details>

**Câu 2.** Gate minh hoạ đạt cả ba vòng. Có được ghi «đạt R-NFR-03» vào báo cáo release không?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. R-NFR-03 nói về hệ thống đích có MapViet, và SPEC nói rõ lab chỉ minh hoạ phương pháp. Gate 1.000 ms là giả định bài tập. Báo cáo chỉ được ghi kết luận trong ranh giới Hình 1.</p>
</details>

**Câu 3.** Vì sao script tính percentile trên mọi attempt, kể cả attempt lỗi?

<details>
<summary><b>Đáp án</b></summary>
<p>Bỏ attempt lỗi — thường là timeout, tức những mẫu chậm nhất — làm phân bố trông đẹp hơn trải nghiệm thật. Giữ chúng trong phân bố và tính riêng error rate cho thấy cả tốc độ lẫn chất lượng.</p>
</details>

**Câu 4.** Lượt chạy lại ở mục 7.4 có p95 cao gấp 2–4 lần. Có nên mở defect hiệu năng không?

<details>
<summary><b>Đáp án</b></summary>
<p>Chưa. Hai lượt khác phiên bản Node và lượt sau chạy khi máy đang tải nặng — vi phạm entry criteria. Cần chạy lại trong điều kiện có kiểm soát, cùng cấu hình với lượt gốc, nhiều lần. Chỉ khi chênh lệch vẫn còn mới điều tra như một regression.</p>
</details>

**Câu 5.** Trong closed model, khi server chậm đi gấp đôi, tốc độ gửi của generator thay đổi thế nào, và vì sao điều đó nguy hiểm khi mục tiêu là kiểm arrival rate?

<details>
<summary><b>Đáp án</b></summary>
<p>Tốc độ gửi giảm khoảng một nửa vì worker chờ response trước khi gửi tiếp. Nếu mục tiêu là «hệ thống dưới 50 arrival/giây», generator đã tự giảm tải đúng lúc hệ thống yếu nhất, nên không đo được thời gian chờ lẽ ra tích luỹ — nguy cơ coordinated omission. Khi đó cần open model và báo cả offered lẫn achieved load.</p>
</details>

## 12. Tài liệu tham khảo

Các nguồn ngoài được tra cứu 13/11/2026.

- Grafana k6 — Load test types: https://grafana.com/docs/k6/latest/testing-guides/test-types/ (phân biệt smoke tải thấp, average-load, stress, soak, spike, breakpoint).
- Grafana k6 — Metrics: https://grafana.com/docs/k6/latest/using-k6/metrics/ (nhóm metric request/error/duration; để đối chiếu với ranh giới đo của lab).
- Grafana k6 — Open and closed models: https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/open-vs-closed/ (quan hệ giữa thời gian iteration và tốc độ gửi).
- Node.js — Performance measurement APIs: https://nodejs.org/api/perf_hooks.html (`performance.now()`); lab đã chạy trên Node v24.15.0 (lượt báo cáo) và v22.18.0 (lượt kiểm thực thi).
- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024): https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf — thuật ngữ test type phi chức năng; thiết kế đo trong bài là hướng dẫn thực tiễn của series.
- Lab Tới Nơi: `SPEC.md` (R-API-07, R-FARE-01…08, R-QUOTE-01, R-NFR-03, R-NFR-07), `baseline.ts`, `baseline-observed.json` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-performance-test
- Bài liên quan: [Non-functional testing tổng quan](/articles/non-functional-testing-tong-quan) · [SYS-01 Smoke Test](/articles/system-testing-smoke-test) · [SYS-03 Functional Test](/articles/system-testing-functional-test) · [SYS-06 Regression Test](/articles/system-testing-regression-test)

Bài tiếp theo: [SYS-08 · Load Test](/articles/system-testing-load-test).
