---
slug: system-testing-stress-test
title: "Stress Test: quan sát Tới Nơi suy giảm thế nào khi tải vượt mức dự kiến"
summary: "Bài thực hành stress testing trên lab Tới Nơi: phân biệt stress với load, spike, breakpoint và recovery; xác định thuộc tính phải giữ khi quá tải; thiết kế profile bước 30 → 150 → 600 → 30 yêu cầu/giây có trần in-flight, deadline và điều kiện dừng; đọc kết quả thực đo mà không biến 'chưa thấy điểm gãy' thành 'capacity 600/s'."
category: testing
level: advanced
tags: [testing, sys-09, stress-testing, performance-testing, open-model, backpressure, graceful-degradation, percentile, test-oracle, toi-noi]
language: vi
---

Tối thứ Sáu, Tới Nơi chạy khuyến mãi «giảm 50% chuyến đầu tuần». Lượng báo giá tăng gấp nhiều lần so với giờ cao điểm thường. Có ba kiểu hỏng rất khác nhau có thể xảy ra: hệ thống trả chậm dần, hệ thống bắt đầu từ chối yêu cầu, hoặc — tệ nhất — vẫn trả 200 nhưng giá sai. Khi chiến dịch kết thúc, traffic giảm nhưng hàng đợi có thể vẫn còn. Bài này trả lời: **làm sao thiết kế một thí nghiệm tải cao có ranh giới, biết trước điều gì được phép xấu đi và điều gì tuyệt đối không, rồi đọc kết quả mà không kết luận quá những gì đã đo.** Lab là một profile bước 30 → 150 → 600 → 30 yêu cầu/giây vào `POST /api/quotes`, có trần, deadline và điều kiện dừng, kèm kết quả thực đo ngày 13/11/2026.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-09</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 2</td></tr>
<tr><th scope="row">Cấp độ</th><td>Nâng cao — đã nắm latency, throughput, percentile, open model và dropped slots ở SYS-07/08</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>System · Non-functional (performance efficiency — hành vi dưới tải cao hơn mức thường)</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-load-test">SYS-08 Load Test</a> · <a href="/articles/system-testing-performance-test">SYS-07 Performance Test</a> · <a href="/articles/non-functional-testing-tong-quan">Non-functional testing tổng quan</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (script Node): 4 bước tải 30/s×3s → 150/s×4s → 600/s×4s → 30/s×3s (3.180 slots), trần 40 in-flight, deadline 2 s, dừng gửi sau 10 lỗi; probe báo giá và health sau tải (R-API-07, R-FARE-01…08, R-NFR-03, R-API-15) — <code>examples/toi-noi-testing/articles/system-testing-stress-test/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>Grafana k6 — Load test types và Stress testing (phân loại, cần baseline trước); <code>SPEC.md</code> của lab; evidence <code>stress-observed.json</code></td></tr>
</tbody>
</table>
</div>

## 1. Tình huống và mục tiêu

### 1.1. Câu hỏi thật sự của stress test

Câu hỏi «server có chết không?» là câu hỏi kém. Một hệ thống không chết nhưng trả giá 0 đồng cho mọi chuyến còn nguy hiểm hơn một hệ thống trả 503 rõ ràng. Với Tới Nơi trong đêm khuyến mãi, đội vận hành cần biết:

- **Suy giảm ở đâu:** latency tăng ở endpoint nào, từ mức tải nào; lỗi xuất hiện dưới dạng gì.
- **Giữ được gì:** giá báo vẫn đúng, không ghi trùng chuyến, không trừ tiền sai — kể cả khi chậm.
- **Trở lại thế nào:** khi tải giảm, hệ thống có quay về trạng thái phục vụ bình thường không, sau bao lâu, còn backlog không.

Stress test là thí nghiệm có chủ đích để quan sát ba điều đó **ở mức tải cao hơn mức thường đã xác định**, với giới hạn an toàn rõ ràng. Nó không phải cuộc thi làm sập máy.

### 1.2. Đối tượng và điều kiện tiên quyết

Bài dành cho QA/SDET và developer đã đọc [SYS-07](/articles/system-testing-performance-test) và [SYS-08](/articles/system-testing-load-test): hiểu p50/p95/p99, phân biệt offered load với achieved load, open model (lập lịch theo thời gian, không chờ response) với closed model (số người dùng ảo cố định), và hiểu vì sao dropped iteration ở generator khác với lỗi server.

Cần Node ≥ 22.12, dependency của lab đã cài bằng `npm ci`. Không cần PostgreSQL ngoài, không cần k6: lab dùng một script Node tự viết với PGlite.

### 1.3. Mục tiêu đo được

Sau bài này, bạn có thể:

1. Phân biệt stress với load, spike, soak, breakpoint và recovery test bằng **mục tiêu**, không bằng con số tải.
2. Viết ra trước khi chạy: thuộc tính bắt buộc giữ (oracle nghiệp vụ), thuộc tính được phép suy giảm (latency, drop có kiểm soát), và điều kiện dừng.
3. Đọc một báo cáo stress theo thứ tự đúng: `planned = sent + dropped` → lỗi trên request đã gửi → lý do drop → rồi mới đến latency và throughput.
4. Giải thích vì sao «600/s không lỗi» **không** được viết thành «capacity 600/s», và vì sao «health 200 sau tải» không phải bằng chứng phục hồi.

## 2. Stress, spike, breakpoint và những thứ dễ nhầm

### 2.1. Định nghĩa dùng trong series

**Stress testing** đánh giá hành vi của hệ thống dưới tải cao hơn mức thường đã xác định. Tài liệu k6 nhấn mạnh hai điểm mà bài này giữ nguyên: phải có baseline/load test trước để biết «bình thường» là gì, và mức tăng phụ thuộc rủi ro của hệ thống — không có tỷ lệ nhân cố định nào đúng cho mọi nơi.

| Loại kiểm | Mục tiêu | Yếu tố quyết định |
|---|---|---|
| Load | Đạt tiêu chí ở tải dự kiến | Mức vận hành mục tiêu, thời lượng ổn định |
| Stress | Quan sát hành vi khi tải cao hơn bình thường | Chấp nhận suy giảm, **vẫn giữ oracle dữ liệu** |
| Spike | Phản ứng với thay đổi tải đột ngột | Tốc độ tăng/giảm, không chỉ đỉnh |
| Soak | Ổn định trong thời gian dài | Rò rỉ bộ nhớ, tích lũy dữ liệu, kết nối |
| Breakpoint | Tìm giới hạn theo tiêu chí định trước | Tăng đủ xa, generator đủ mạnh, tiêu chí dừng rõ |
| Recovery | Khả năng phục hồi sau sự cố | Failure model, thời gian, toàn vẹn dữ liệu (SYS-14) |

Lab của bài là **stress dạng bước có giới hạn**: bốn mức tải cố định, mỗi mức drain xong mới sang mức sau. Nó không phải ramp tuyến tính, không phải spike production và không phải breakpoint.

### 2.2. Mức «bình thường» đến từ đâu

Mức 30/s là **normal giả định** kế thừa từ bài Load (profile 10 → 30 → 10/s), không phải telemetry production của Tới Nơi. SPEC có R-NFR-07 nêu mức tải tham chiếu của hệ thống đích — khoảng 70 chuyến tạo mới/giây lúc cao điểm — và ghi rõ đó «không phải mục tiêu, để dựng mô hình tải». Hai con số này không mâu thuẫn: lab chỉ có một endpoint báo giá trên một tiến trình, nên dùng mức nhỏ hơn và tự nhận là giả định.

150/s (gấp 5) và 600/s (gấp 20) được chọn để **học cách quan sát** mức cao hơn trong một máy, không phải để tuyên bố giới hạn của Tới Nơi.

### 2.3. «Suy giảm có kiểm soát» là một hợp đồng

Khi quá tải, hệ thống có nhiều cách phản ứng hợp lệ — miễn là có hợp đồng:

- **429** nghĩa là client vượt quota của mình; SPEC có quota cho tạo chuyến (R-API-13, 5 yêu cầu/60 giây/khách, kèm `Retry-After`), **không** có quota cho báo giá.
- **503** nghĩa là service tạm không nhận thêm; client nên lùi lại.
- **Timeout phía client** không cho biết server đã thực hiện thao tác hay chưa. Với báo giá thì vô hại; với đặt chuyến hay thanh toán thì phải đối soát.

Ngược lại, có những thứ **không bao giờ** được coi là «suy giảm chấp nhận được»: tính sai tiền, ghi trùng chuyến, báo thanh toán thành công khi chưa thu. Một stress test thiếu oracle nghiệp vụ sẽ không phân biệt được hai nhóm này.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Đừng nhầm với «smoke» của k6</span>
<p>Trong tài liệu k6, «smoke test» là chạy tải rất thấp để kiểm script trước khi đo. Đó khác với smoke build ở <a href="/articles/system-testing-smoke-test">SYS-01</a>. Tương tự, «recovery» trong bài này chỉ là quan sát sau khi giảm tải; recovery test theo failure model, RTO/RPO và restore nằm ở SYS-14.</p>
</div>

## 3. Test basis, rủi ro và oracle

### 3.1. Test basis

| Nguồn | Nội dung dùng trong bài |
|---|---|
| R-API-07 | `POST /api/quotes` cho khách, trả 200 với `quote.quoteId`, `quote.total` |
| R-API-03 | Header `x-user: customer:C001` là danh tính giản lược của lab |
| R-FARE-01…08 | Xe máy SG 5 km, giờ thường, không mưa, không voucher ⇒ 12.000 + 4.500 × 5 = 34.500, làm tròn lên bội số 1.000 ⇒ **35.000 đ** |
| R-QUOTE-01 | Mỗi báo giá thành công tạo một bản ghi lưu lại ⇒ dữ liệu tích lũy qua các bước |
| R-NFR-03 | p95 của `POST /api/quotes` dưới 500 ms (hệ thống đích) |
| R-API-15 | `GET /api/health` trả 200 `{ "status": "ok" }` |

Oracle tiền 35.000 tính tay từ bảng cước, không gọi lại calculator của sản phẩm. Ngưỡng 500 ms lấy từ R-NFR-03; trong lab nó là **reference envelope**, không phải cam kết hiệu năng của bản lab một tiến trình.

### 3.2. Rủi ro được nhắm tới

1. Dưới tải, response vẫn 200 nhưng nội dung sai (giá, thiếu `quoteId`) — **rủi ro dữ liệu**.
2. Latency đuôi (p99) tăng nhanh hơn p50 khi số yêu cầu đồng thời tăng — **rủi ro trải nghiệm**.
3. Sau khi giảm tải, hệ thống không về lại mức bình thường — **rủi ro phục hồi**.
4. Generator không giữ được lịch, khiến báo cáo «server chịu được» trong khi tải thật chưa được gửi — **rủi ro đo sai**.

Rủi ro 4 hay bị bỏ qua nhất. Một script closed-model chậm lại cùng server sẽ tự giảm tải và báo «không lỗi».

### 3.3. Tiêu chí bắt đầu và kết thúc

**Entry:** có kết quả SYS-07/08 trên cùng máy; dependency cài theo lockfile; SUT là server loopback do script tự dựng, không nhận URL ngoài; không chạy song song với tác vụ nặng khác.

**Exit của bài tập** (đúng như mã nguồn): exit code 0 khi (a) không có request lỗi ở bất kỳ bước nào, (b) baseline và return-low nằm trong envelope — không drop, không lỗi, p95 < 500 ms, (c) probe báo giá sau tải đúng oracle, (d) health trả 200. Ở elevated và high, p95 vượt 500 ms hoặc drop được **ghi nhận là suy giảm**, không tự làm exit 1.

Exit 0 vì thế chỉ có nghĩa các điều kiện bắt buộc đã đạt. Nó **không** nói mọi slot tải cao đã được gửi đủ. Nếu mục tiêu release yêu cầu «không drop ở 600/s», phải đưa tiêu chí đó vào gate **trước khi chạy**; đổi tiêu chí sau khi thấy kết quả là làm sai thí nghiệm.

## 4. Thiết kế thí nghiệm có ranh giới

### 4.1. Profile và các guard

| Mục | Thiết kế |
|---|---|
| SUT | Express + use case + repository + PGlite, cùng tiến trình với generator |
| Profile | baseline 30/s×3s → elevated 150/s×4s → high 600/s×4s → return-low 30/s×3s |
| Quy mô | 3.180 slots, cộng 20 warm-up, một quote probe cuối và một health |
| Request | C001, ride SG bike 5 km, không voucher, 09:00 giờ VN (FixedClock), không mưa |
| Lập lịch | Open slots: không chờ response để gửi slot tiếp theo |
| Trần | Tối đa 40 in-flight; slot trễ > 100 ms bị bỏ, không bắn bù |
| Dừng gửi | Sau 10 request lỗi, mọi slot sau ghi `failure-stop` |
| Deadline | 2 giây/request (`AbortSignal.timeout`) |
| Dữ liệu | Reset sau warm-up, giữ nguyên xuyên bốn bước |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Profile tải bước của SYS-09: trục ngang là thời gian theo lịch, trục dọc là số yêu cầu mỗi giây. Bốn bậc: baseline 30/s trong 3 giây, elevated 150/s trong 4 giây, high 600/s trong 4 giây, return-low 30/s trong 3 giây. Giữa các bậc có khoảng drain. Bên phải là các guard: trần 40 in-flight, bỏ slot trễ hơn 100 ms, deadline 2 giây mỗi request, ngừng gửi sau 10 lỗi. Sau profile là probe báo giá và health.">
<defs>
<marker id="sys09-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<line x1="60" y1="250" x2="640" y2="250" class="s-pk" stroke-width="1.6" marker-end="url(#sys09-arrow)"/>
<line x1="60" y1="250" x2="60" y2="30" class="s-pk" stroke-width="1.6" marker-end="url(#sys09-arrow)"/>
<text x="640" y="272" text-anchor="end" class="t-small">thời gian theo lịch (s)</text>
<text x="66" y="26" class="t-small">yêu cầu/giây</text>
<rect x="70" y="236" width="90" height="14" rx="2" class="f-pk-soft s-pk"/><text x="115" y="228" text-anchor="middle" class="t-small">baseline 30/s · 3s</text>
<rect x="172" y="200" width="120" height="50" rx="2" class="f-pk-soft s-pk"/><text x="232" y="192" text-anchor="middle" class="t-small">elevated 150/s · 4s</text>
<rect x="304" y="60" width="120" height="190" rx="2" class="f-warn-soft s-warn"/><text x="364" y="52" text-anchor="middle" class="t-b">high 600/s · 4s</text>
<rect x="436" y="236" width="90" height="14" rx="2" class="f-pk-soft s-pk"/><text x="481" y="228" text-anchor="middle" class="t-small">return-low 30/s · 3s</text>
<text x="166" y="266" text-anchor="middle" class="t-small t-phys">drain</text>
<text x="298" y="266" text-anchor="middle" class="t-small t-phys">drain</text>
<text x="430" y="266" text-anchor="middle" class="t-small t-phys">drain</text>
<rect x="540" y="196" width="92" height="44" rx="8" class="f-phys-soft s-phys"/><text x="586" y="214" text-anchor="middle" class="t-small">probe quote</text><text x="586" y="230" text-anchor="middle" class="t-small">+ health</text>
<rect x="670" y="30" width="234" height="210" rx="10" class="f-bg s-pk dash"/>
<text x="787" y="54" text-anchor="middle" class="t-b">Guard của generator</text>
<text x="686" y="86" class="t-small">• tối đa 40 request in-flight</text>
<text x="686" y="112" class="t-small">• slot trễ &gt; 100 ms ⇒ bỏ, không bù</text>
<text x="686" y="138" class="t-small">• deadline 2 s mỗi request</text>
<text x="686" y="164" class="t-small">• ≥ 10 lỗi ⇒ ngừng gửi slot mới</text>
<text x="686" y="190" class="t-small">• server loopback cổng ngẫu nhiên</text>
<text x="686" y="216" class="t-small">• không nhận URL bên ngoài</text>
<text x="460" y="292" text-anchor="middle" class="t-small t-warn">generator, server và PGlite cùng một tiến trình Node — giới hạn của máy có thể đến trước giới hạn của SUT</text>
</svg>
</div>
<figcaption><b>Hình 1 · Profile bước và guard của lab SYS-09.</b> Mỗi bậc drain hết request đang chạy rồi mới sang bậc sau, nên timeline thực dài hơn 14 giây theo lịch. Chiều cao các bậc chỉ minh hoạ tương đối.</figcaption>
</figure>

Hai hệ quả của thiết kế cần nói thẳng:

- **Drain giữa các bước** làm thí nghiệm dễ đọc (mỗi bucket chỉ chứa request của bước đó) nhưng che mất backlog liên tục. Đây không phải ramp không gián đoạn.
- **Safety stop dùng chung bộ đếm lỗi toàn cục.** Nếu đã kích hoạt ở bước high, các slot return-low cũng bị bỏ; probe cuối vẫn chạy một lần nhưng không thay thế được phase giảm tải.

### 4.2. Bảng case

Bản nháp chỉ mô tả các bước; bài này đặt ID để dễ báo cáo. Mọi case chạy trong một lần thực thi script.

| ID | Requirement / rủi ro | Precondition | Input | Action | Expected | Cleanup |
|---|---|---|---|---|---|---|
| ST-01 | Script và SUT sẵn sàng (rủi ro đo sai) | Server loopback mới, PGlite mới | 20 báo giá chuẩn | Gửi tuần tự | Cả 20 đúng oracle; lỗi ⇒ dừng toàn bộ | `system.reset()` |
| ST-02 | R-API-07, R-FARE, R-NFR-03 | Sau reset | 30/s × 3s | Open slots | 90 sent, 0 drop, 0 lỗi, p95 < 500 ms | Drain |
| ST-03 | Rủi ro dữ liệu, latency | Dữ liệu từ ST-02 | 150/s × 4s | Open slots | 0 lỗi bắt buộc; drop/p95 ghi nhận | Drain |
| ST-04 | Rủi ro dữ liệu, latency đuôi, đo sai | Dữ liệu tích lũy | 600/s × 4s | Open slots | 0 lỗi bắt buộc; drop/p95 ghi nhận | Drain |
| ST-05 | Rủi ro phục hồi | Ngay sau high | 30/s × 3s | Open slots | 90 sent, 0 drop, 0 lỗi, p95 < 500 ms | Drain |
| ST-06 | R-API-07, R-API-15 | Sau mọi bước | 1 báo giá + 1 health | Gửi đơn lẻ | Báo giá đúng oracle; health 200 `ok` | Đóng server và DB trong `finally` |

## 5. Lab từng bước

### 5.1. Chạy

Từ thư mục `examples/toi-noi-testing`:

```sh
npm ci
npx tsx articles/system-testing-stress-test/stress.ts
npm run typecheck
```

Script tự tạo server trên `127.0.0.1` cổng ngẫu nhiên, ghi raw JSON vào `reports/stress/sys09-stress.json` tương đối với thư mục chạy, in bản tóm tắt (không kèm samples) ra stdout và đóng server, DB trong `finally`. Chỉ nên chạy khi máy rảnh: mọi tác vụ nặng khác đều cạnh tranh CPU với cả generator lẫn SUT.

### 5.2. Mỗi request là một phép kiểm nghiệp vụ

```ts
const res = await fetch(url, {
  method: 'POST', headers: { 'content-type': 'application/json', 'x-user': 'customer:C001' },
  body: JSON.stringify({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 }),
  signal: AbortSignal.timeout(2_000),
});
status = res.status;
const body = await res.json() as { quote?: { total?: number; quoteId?: string } };
valid = status === 200 && body.quote?.total === 35_000 && typeof body.quote.quoteId === 'string';
if (!valid) error = 'HTTP_OR_BUSINESS_ORACLE';
```

Đây là điểm khác biệt quan trọng nhất so với một script «bắn request đếm status»: mỗi response phải đúng **cả HTTP lẫn tiền**. Request lỗi vẫn được tính vào percentile — latency của request thất bại không bị loại khỏi phân bố. Timeout và lỗi transport rơi vào nhánh `catch`, ghi tên lỗi (ví dụ `TimeoutError`).

Endpoint báo giá ghi dữ liệu (R-QUOTE-01), nên số bản ghi tăng dần qua bốn bước. Đó là chủ đích: dùng `GET /api/health` thay workload nghiệp vụ sẽ không chạm DB và không nói gì về suy giảm thật.

### 5.3. Ba lý do bỏ slot, theo thứ tự ưu tiên

```ts
const lagMs = performance.now() - due;
const reason = failureCount >= 10 ? 'failure-stop' : lagMs > maxLagMs ? 'scheduler-late' : active.size >= maxInFlight ? 'in-flight-cap' : null;
if (reason) { dropped.push({ slot, lagMs, reason }); continue; }
```

- `failure-stop`: guard an toàn, giới hạn tác động khi SUT đã hỏng.
- `scheduler-late`: **generator** không giữ được lịch. Slot này chưa bao giờ tới server, nên không được gọi là «server từ chối».
- `in-flight-cap`: số request đang chờ đã chạm trần — dấu hiệu request giữ slot lâu hơn, nhưng cũng là giới hạn do ta đặt.

Một lưu ý: dừng theo lỗi có độ trễ. Các request đang bay khi bộ đếm chạm 10 vẫn có thể lỗi tiếp, nên tổng lỗi có thể vượt 10. Không gọi 10 là trần tuyệt đối.

### 5.4. Drain, rồi mới tính số liệu từng bước

```ts
const end = start + stage.seconds * 1_000;
while (performance.now() < end) await sleep(Math.max(1, end - performance.now()));
await Promise.all(active); // drain before the next step; report includes drain time
const elapsedMsIncludingDrain = performance.now() - start;
const sorted = samples.map(s => s.latencyMs).sort((a, b) => a - b);
const p = (q: number) => sorted.length ? sorted[Math.ceil(q * sorted.length) - 1]! : null;
```

Percentile dùng nearest-rank trên toàn bộ samples của **từng bước**. Không gộp bốn bước thành một p95: con số gộp sẽ bị 2.400 request của bước high kéo, nhưng cũng có thể che bước tệ nhất khi bước đó ít request. Script còn ghi `completedInWindow` (số request hoàn tất trước khi cửa sổ lịch kết thúc) để không nhầm «gửi đủ 600/giây» với «hoàn tất 600/giây».

### 5.5. Gate cuối

```ts
if (results.some(r => r.errors > 0 || (['baseline', 'return-low'].includes(r.name) && !r.withinReferenceEnvelope)) || !postLoadProbe.valid || health.status !== 200) process.exitCode = 1;
```

Raw JSON được ghi **trước** khi quyết định exit code, để một lần chạy thất bại vẫn để lại bằng chứng điều tra.

## 6. Kết quả thực đo và cách đọc

### 6.1. Môi trường của lần chạy

Lần chạy làm evidence kết thúc lúc 21:29:50 giờ Việt Nam ngày 13/11/2026 (`measuredAtUtc` 2026-11-13T14:29:50.931Z): Windows 10.0.26300, Node v24.15.0, Intel Core i7-10700K, 16 logical CPU, RAM 34.245.918.720 byte; lệnh chạy từ root với `npx --prefix examples/toi-noi-testing tsx …`, exit 0, `npm run typecheck` đạt. File gốc đã được lưu thành `stress-observed.json` cạnh script; báo cáo đã kiểm độc lập rằng counts, error count và p95 khớp raw samples.

Khi biên tập bài (cùng ngày), script **không** được chạy lại: máy đang chạy song song các lab khác, mọi số đo mới sẽ bị nhiễm tải ngoài. Các con số dưới đây đều đọc từ `stress-observed.json`.

### 6.2. Expected và observed

Expected theo thiết kế: baseline và return-low trong envelope; không request lỗi ở bất kỳ bước nào; probe đúng 35.000 và health 200. Elevated/high: chưa có kỳ vọng định lượng — đó là phần cần quan sát.

Observed:

| Bước | Target | Sent / dropped / errors | p50 ms | p95 ms | p99 ms | Peak in-flight | Hoàn tất trong cửa sổ | Valid RPS gồm drain |
|---|---|---|---:|---:|---:|---:|---:|---:|
| baseline | 30/s × 3s | 90 / 0 / 0 | 3,11 | 10,49 | 11,36 | 1 | 90 | 29,99 |
| elevated | 150/s × 4s | 600 / 0 / 0 | 3,79 | 14,06 | 22,39 | 6 | 600 | 149,76 |
| high | 600/s × 4s | 2.400 / 0 / 0 | 5,47 | 15,15 | 22,80 | 24 | 2.393 | 598,34 |
| return-low | 30/s × 3s | 90 / 0 / 0 | 2,34 | 6,10 | 8,67 | 1 | 90 | 29,93 |

Probe báo giá sau tải: HTTP 200, đúng oracle, 2,83 ms. Health: 200 `{ "status": "ok" }`. Mọi response trong bốn bước có status 200. Cả bốn bước nằm trong reference envelope, `planned = sent + dropped` đúng ở mọi bước (3.180 = 3.180 + 0).

### 6.3. Đọc từng dòng, theo thứ tự đúng

1. **Đủ tải chưa?** Không có slot nào bị drop, kể cả `scheduler-late`: generator giữ được lịch ở mọi mức. Độ trễ lịch lớn nhất mỗi bước khoảng 15,6–20,2 ms, dưới ngưỡng 100 ms.
2. **Có lỗi không?** Không có request lỗi, không response sai tiền.
3. **Suy giảm thế nào?** p50 tăng từ 3,11 lên 5,47 ms, p99 từ 11,36 lên 22,80 ms; peak in-flight tăng từ 1 lên 24 — request giữ slot lâu hơn khi tải tăng. Đó là suy giảm có thật nhưng nhỏ, còn rất xa 500 ms.
4. **«600/s» nghĩa là gì?** 2.400 request được **gửi** trong 4 giây theo lịch; 2.393 hoàn tất trong cửa sổ, 7 hoàn tất ở phần đuôi (elapsed gồm drain ≈ 4.011,08 ms). Valid RPS gồm drain là 598,34 — không phải 600.
5. **Sau giảm tải?** Return-low có p95 6,10 ms, thấp hơn cả baseline. Không nên đọc thành «hệ thống nhanh hơn sau stress»; chênh lệch vài mili-giây trên một tiến trình có thể do JIT đã nóng, cache, hoặc nhiễu lịch của hệ điều hành. Lab chưa có dữ liệu để chọn giữa các giả thuyết này.

Một quan sát nhỏ đáng ghi: độ trễ lịch ~15,6 ms xuất hiện ngay cả ở 30/s, khi gần như không có tải. Một giả thuyết hợp lý là độ phân giải timer mặc định trên Windows; bài **chưa** kiểm chứng giả thuyết này. Nếu đúng, nó là đặc tính của generator trên máy này, không phải của SUT.

### 6.4. Kết luận được phép viết

> Trong lab một tiến trình (Node v24.15.0, i7-10700K, PGlite một kết nối), endpoint báo giá giữ đúng oracle tiền ở cả bốn mức 30/150/600/30 yêu cầu/giây; latency đuôi và in-flight tăng theo tải nhưng p95 vẫn ≤ 15,15 ms. **Chưa quan sát thấy điểm gãy** trong giới hạn đã thử. Các nhánh `in-flight-cap`, `scheduler-late`, `failure-stop` có trong mã nhưng **không được kích hoạt** trong lần chạy này.

Không được viết: «capacity Tới Nơi ≥ 600/s», «hệ thống phục hồi hoàn toàn», «đạt R-NFR-03 cho production». Không có lỗi sản phẩm được phát hiện; điều đó không chứng minh không có lỗi dưới tải khác.

## 7. Đọc kết quả theo nguyên nhân có thể kiểm chứng

Lần chạy đo được «sạch», nhưng một báo cáo stress hữu ích phải sẵn cách đọc cho lần chạy không sạch:

| Mẫu quan sát | Điều có thể suy ra | Chưa thể kết luận | Bằng chứng tiếp theo |
|---|---|---|---|
| Latency tăng, in-flight tăng | Request giữ slot lâu hơn | Chính xác CPU, SQL hay event loop nghẽn | Profile CPU, thời gian truy vấn, event loop lag tách riêng |
| Lag tăng, nhiều `scheduler-late` | Generator không giữ lịch | Server riêng biệt không chịu được target | Tách generator sang tiến trình/máy khác rồi lặp lại |
| Nhiều `in-flight-cap` | Request chậm hơn khả năng nhả slot | Server đã «từ chối» | Tăng trần chỉ khi máy còn tài nguyên; so p99 |
| Sent đủ, lỗi oracle tăng | Request tới nơi nhưng sai nghiệp vụ | Lỗi chỉ do tải | Phân loại lỗi, tái hiện ở tải thấp với cùng dữ liệu |
| `TimeoutError` | Client bỏ cuộc sau 2 s | Server không thực hiện thao tác | Đối soát dữ liệu phía server |
| Return-low đạt, probe đúng | Sau drain, luồng chọn hoạt động | Phục hồi đầy đủ mọi tài nguyên, RTO | Timeline liên tục, nhiều cửa sổ ổn định |
| Mọi mức đều đạt | Chưa thấy failure trong giới hạn thử | Đã tìm được capacity | Thiết kế thí nghiệm mới, không tăng tải vô hạn |

Đừng ép kết quả phải có lỗi để «đúng bài Stress». Nếu 600/s vẫn đạt, bước tiếp theo là một thí nghiệm mới với phạm vi và tài nguyên phù hợp — ưu tiên tách generator khỏi SUT trước khi nâng target.

**Ví dụ quyết định cho một lần chạy giả định không sạch:** «High: 2.400 planned, 2.010 sent, 390 drop (`in-flight-cap`), 0 lỗi oracle, p95 1.240 ms. Return-low đạt envelope, probe đúng. Kết luận: hệ thống giữ đúng dữ liệu nhưng không xử lý kịp 600/s trong cấu hình này; chưa phân biệt được giới hạn SUT với giới hạn máy. Đề xuất: tách generator, đo CPU từng tiến trình, lặp lại high.» Các con số trong ví dụ này là minh hoạ, **không** phải kết quả đã đo.

## 8. Mở rộng nâng cao

### 8.1. Graceful degradation và backpressure

Hệ thống thật có thể dùng hàng đợi hữu hạn, admission control, rate limit hoặc circuit breaker. Test cần xác định trước: khi nào hệ thống từ chối, response có giúp client retry đúng không (`Retry-After`, mã lỗi phân biệt được), và giao dịch **đã nhận** có hoàn tất đúng không. Hàng đợi không giới hạn thường cho throughput đẹp lúc đầu rồi latency tăng và cạn bộ nhớ — throughput cao ban đầu không chứng minh ổn định.

Ví dụ Tới Nơi: nếu báo giá phụ thuộc dịch vụ bản đồ MapViet (R-NFR-03 nói rõ điều này với hệ thống đích), một chiến lược suy giảm hợp lý là trả 503 nhanh khi MapViet quá tải, thay vì giữ kết nối tới khi client timeout. Stress test phải kiểm cả hai: thời gian từ chối và việc không có báo giá sai lọt ra.

### 8.2. Retry amplification

Khi nhiều client retry ngay lúc timeout, tải mới và tải retry cộng dồn — một đợt chậm nhỏ có thể tự khuếch đại thành sự cố. Workload cần tách **attempt** khỏi **logical transaction**, ghi retry count và idempotency key; kiểm backoff, jitter và retry budget khi có yêu cầu tương ứng. Lab hiện không retry request báo giá, nên **chưa mô phỏng retry storm**.

### 8.3. Không mất toàn vẹn dưới áp lực

Với đặt chuyến và thanh toán, kiểm invariant trong hoặc sau tải: số chuyến bằng số logical transaction thành công, không trừ tiền hai lần (R-NFR-05 giới hạn 2 lần thử lại cùng khoá idempotency), sự kiện khớp trạng thái. Timeout không đồng nghĩa chưa ghi dữ liệu; test phải đối soát các transaction có kết quả không rõ, không được gửi khoá mới rồi coi đó là retry an toàn. Lab chỉ báo giá; tranh chấp ví và dispatch thuộc phạm vi SYS-13 và cần PostgreSQL nhiều kết nối.

### 8.4. Đo recovery có thời gian

Muốn có con số time-to-recover, phải định nghĩa trước trạng thái «đã phục hồi» — ví dụ giả định: ba cửa sổ 10 giây liên tiếp ở tải thường có 0 lỗi và p95 dưới ngưỡng, backlog về mức chấp nhận. Ghi thời điểm giảm tải và thời điểm đạt điều kiện để tính khoảng cách. Cần scheduler theo một timeline chung (không drain giữa high và low), gắn mỗi request với bước đã phát ra nó, và tách thời gian drain do harness khỏi hành vi hệ thống. Một probe xanh không chứng minh ổn định.

### 8.5. Autoscaling và capacity planning

Trên staging có autoscaler, ghi số replica, thời gian khởi động, readiness, độ sâu hàng đợi và saturation từng instance. Tải tăng nhanh hơn tốc độ scale có thể gây lỗi tạm thời dù capacity cuối đủ — đó chính là chỗ spike test khác stress test. Không lấy 600/s của một phiên PGlite rồi nhân số node để dự báo chi phí cluster; capacity planning cần workload đại diện (tỷ lệ báo giá/đặt/đọc), dữ liệu cỡ thật và margin rủi ro.

## 9. Anti-pattern, giới hạn và đánh đổi

**Anti-pattern hay gặp:**

- Chạy stress lên production hoặc endpoint bên ngoài khi chưa thuộc phạm vi được phép.
- Bỏ guard để «cố tìm crash»; tăng tải vô hạn đến khi máy treo.
- Gọi mọi drop là 503 của server; gọi generator chậm là server chậm.
- Báo «phục hồi hoàn toàn» vì health xanh.
- Dùng một p95 gộp che bước high; loại request timeout khỏi phân bố latency.
- Chỉ đếm status 200 mà không kiểm nội dung response.
- Sửa `src/` của sản phẩm để tạo ra lỗi cho đẹp báo cáo.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn bằng chứng của lab SYS-09</span>
<p>Generator, server và PGlite chạy chung một tiến trình và một event loop, một kết nối DB, dữ liệu nhỏ. Profile chỉ 14 giây theo lịch cộng drain và warm-up: không phải soak, không đo rò rỉ bộ nhớ, không đo RTO/RPO hay crash recovery. Không có telemetry CPU/DB tách client–server. Các nhánh drop và safety stop chỉ được chứng minh là <b>có trong mã</b>, chưa được kích hoạt bằng thực nghiệm. Script là Node tự viết, không phải k6 đã chạy. Không suy ra capacity production hay độ ổn định dài hạn.</p>
</div>

**Đánh đổi chi phí/phạm vi:** một profile ngắn, chạy local, rẻ và tái hiện được — đủ để luyện thiết kế oracle, guard và cách đọc. Muốn trả lời câu hỏi capacity thật, chi phí tăng mạnh: môi trường cô lập giống production, generator phân tán, dữ liệu cỡ thật, monitoring đầy đủ và thời gian quan sát dài. Hãy chọn mức đầu tư theo câu hỏi cần trả lời, không theo số request lớn nhất có thể bắn.

## 10. Bài tập và câu hỏi tự kiểm

### Bài tập

1. **Cơ bản.** Một lần chạy cho bước high có 10% slot bị drop, return-low và probe đạt, exit code 0. Viết một câu kết luận đúng.
2. **Trung bình.** Sau tải, health trả 200 nhưng probe báo giá trả `total` 34.500. Script phải làm gì và báo cáo nên viết thế nào?
3. **Trung bình.** Sửa script để mỗi bước ghi thêm số lượng drop theo từng lý do (`failure-stop`, `scheduler-late`, `in-flight-cap`). Vì sao con số này quan trọng hơn tổng số drop?
4. **Nâng cao.** Thêm retry phía client cho request báo giá. Thiết kế lại workload và báo cáo.
5. **Nâng cao.** Thiết kế quan sát recovery không drain giữa high và return-low.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p><b>1.</b> «Ở 600/s, 10% slot không được gửi; các request đã gửi không lỗi. Chưa xác định drop do generator hay do SUT giữ slot lâu; mức high <b>không đạt</b> nếu tiêu chí release là không drop.» Exit 0 chỉ nói điều kiện bắt buộc của bài đạt; phải đọc planned/sent và lý do drop.</p>
<p><b>2.</b> Script đã fail (<code>!postLoadProbe.valid</code> ⇒ exit 1). Báo cáo: tiến trình còn sống nhưng nghiệp vụ sai; 34.500 khớp cước trước bước làm tròn R-FARE-08, gợi ý lỗi ở bước làm tròn sau tải — cần tái hiện ở tải thấp, không gọi là «recovery thành công».</p>
<p><b>3.</b> Ví dụ: <code>dropReasons: Object.groupBy(dropped, d =&gt; d.reason)</code> rồi đếm, hoặc reduce tương đương nếu môi trường không hỗ trợ. <code>scheduler-late</code> chỉ về generator, <code>in-flight-cap</code> chỉ về thời gian giữ slot, <code>failure-stop</code> chỉ về lỗi đã xảy ra trước đó — ba hướng điều tra khác nhau.</p>
<p><b>4.</b> Cần retry budget, backoff có jitter, logical transaction ID tách khỏi attempt; báo cáo cả offered load mới và tải retry. Nếu chuyển sang đặt chuyến, mọi retry phải cùng <code>Idempotency-Key</code> và kiểm không tạo chuyến trùng.</p>
<p><b>5.</b> Một scheduler theo timeline chung; mỗi request mang nhãn bước đã phát ra nó; tính metrics theo cửa sổ thời gian cố định sau thời điểm giảm tải; định nghĩa trước điều kiện ổn định (ví dụ giả định ba cửa sổ liên tiếp đạt) và không gộp response đến muộn của high vào bucket return-low.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** Điều gì phân biệt stress test với load test?

- a) Stress test luôn dùng nhiều request hơn load test
- b) Load test kiểm tiêu chí ở tải dự kiến; stress test quan sát hành vi khi tải cao hơn mức thường đã xác định
- c) Stress test phải làm sập hệ thống
- d) Load test không cần oracle nghiệp vụ

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng.</b> a) số request tuyệt đối không phân biệt được: «cao hơn bình thường» là tương đối với baseline. c) sai: stress quan sát suy giảm có giới hạn, không bắt buộc crash. d) sai: cả hai đều cần oracle.</p>
</details>

**Câu 2.** Một slot bị ghi `scheduler-late`. Kết luận nào đúng?

- a) Server đã từ chối request
- b) Request đã gửi nhưng timeout
- c) Request chưa bao giờ được gửi vì generator không giữ được lịch
- d) Bộ đếm lỗi đã chạm 10

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng.</b> Slot bị bỏ trước khi gọi <code>send()</code>. a), b) mô tả request đã tới server. d) là <code>failure-stop</code>, có ưu tiên cao hơn trong biểu thức chọn lý do.</p>
</details>

**Câu 3.** Bước high có 2.400 sent, 2.393 hoàn tất trong cửa sổ, 0 lỗi. Phát biểu nào chính xác?

- a) Hệ thống xử lý xong 600 request mỗi giây trong 4 giây
- b) 2.400 request được gửi theo lịch 600/s; 7 hoàn tất sau khi cửa sổ 4 giây kết thúc
- c) Capacity của Tới Nơi là 600/s
- d) 7 request đã lỗi

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng.</b> a) nhầm tốc độ gửi với tốc độ hoàn tất. c) vượt quá bằng chứng: một máy, một tiến trình, PGlite, 4 giây. d) sai: 0 lỗi; chúng chỉ hoàn tất ở phần drain.</p>
</details>

**Câu 4.** Vì sao endpoint báo giá được chọn thay vì <code>GET /api/health</code> làm workload?

- a) Health không có trong SPEC
- b) Báo giá chạm domain, DB và có oracle tiền; health chỉ cho biết tiến trình còn phản hồi
- c) Health nhanh quá nên không đo được
- d) Báo giá không ghi dữ liệu nên an toàn hơn

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b) đúng.</b> a) sai: health là R-API-15. c) không phải lý do thiết kế. d) sai: R-QUOTE-01 lưu mỗi báo giá, dữ liệu tích lũy qua các bước — chính vì thế script reset sau warm-up.</p>
</details>

**Câu 5.** Return-low đạt envelope và probe đúng. Có được kết luận «hệ thống phục hồi hoàn toàn sau stress» không?

- a) Có, vì không còn lỗi
- b) Có, nếu health cũng trả 200
- c) Không: drain giữa các bước và một probe đơn chưa đo được time-to-recover hay trạng thái mọi tài nguyên
- d) Không, vì p95 return-low thấp hơn baseline là bất thường

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c) đúng.</b> Cần timeline liên tục, tiêu chí ổn định định nghĩa trước và quan sát nhiều cửa sổ. d) sai lý do: chênh vài mili-giây trong một tiến trình có nhiều giả thuyết (JIT nóng, cache, nhiễu OS), không tự chứng minh điều gì.</p>
</details>

## 11. Tài liệu tham khảo

- Grafana k6 — *Load test types*: https://grafana.com/docs/k6/latest/testing-guides/test-types/ — phân loại smoke, average-load, stress, soak, spike, breakpoint (tra cứu 13/11/2026).
- Grafana k6 — *Stress testing*: https://grafana.com/docs/k6/latest/testing-guides/test-types/stress-testing/ — tải cao hơn thông thường, cần baseline/load trước, mức tăng tùy hệ thống (tra cứu 13/11/2026). Bài dùng k6 làm nguồn thuật ngữ; script lab là Node tự viết.
- Phòng lab: `SPEC.md` (R-API-07, R-API-15, R-FARE-01…08, R-QUOTE-01, R-NFR-03, R-NFR-07) và mã bài — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-stress-test
- Evidence: `stress-observed.json` cùng thư mục, đo ngày 13/11/2026 (Node v24.15.0, Windows 10.0.26300, i7-10700K).
- Bài liên quan: [SYS-07 · Performance Test](/articles/system-testing-performance-test) · [SYS-08 · Load Test](/articles/system-testing-load-test) · [Non-functional testing tổng quan](/articles/non-functional-testing-tong-quan) · [SYS-13 · Data Integrity Test](/articles/system-testing-data-integrity-test) · [SYS-14 · Recovery Test](/articles/system-testing-recovery-test)

Bài tiếp theo: [SYS-10 · Security Test – Pen test](/articles/system-testing-security-pentest).
