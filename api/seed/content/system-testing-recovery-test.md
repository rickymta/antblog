---
slug: system-testing-recovery-test
title: "Recovery Test: hệ thống có thật sự trở lại sau sự cố, hay chỉ health check xanh?"
summary: "SYS-14 của series kiểm thử hệ thống: phân biệt retry, restart, restore, failover và đối soát; định nghĩa RTO/RPO đo được; inject lỗi ví VíXanh qua API Tới Nơi để kiểm phục hồi trong ngân sách retry, việc mới sau outage và khoản đã thu nhưng ghi failed; diễn tập pg_dump/pg_restore trên cluster PostgreSQL 18 riêng — kèm giới hạn bằng chứng."
category: testing
level: advanced
tags: [testing, sys-14, recovery-testing, rto-rpo, backup-restore, pg-dump, retry, idempotency, reconciliation, vitest]
language: vi
---

10:00, VíXanh ngừng trả lời. 10:07, VíXanh trở lại, `/api/health` của Tới Nơi vẫn xanh suốt thời gian đó, tài xế tiếp tục hoàn thành chuyến. Nhìn dashboard, mọi thứ "đã phục hồi". Nhưng trong bảy phút ấy có một chuyến mà ví **đã trừ tiền khách** mà không kịp trả lời, và trong DB chuyến đó mang nhãn `failed`. Nếu ai đó "sửa" bằng cách thu lại với một khoá mới, khách bị trừ hai lần. Bài này trả lời: **recovery test kiểm cái gì ngoài việc tiến trình sống lại, đo RTO/RPO thế nào cho trung thực, và thiết kế phép thử sao cho nó phơi ra khoảng trống chứ không che đi** — qua hai lab: lỗi dependency thanh toán trên API Tới Nơi và một restore drill PostgreSQL thật.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-14</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 2</td></tr>
<tr><th scope="row">Cấp độ</th><td>Advanced — QA/SDET và kỹ sư vận hành đã nắm transaction, idempotency và SQL</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>System test, đặc tính phi chức năng: khả năng phục hồi (recoverability) của dịch vụ và dữ liệu</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-data-integrity-test">SYS-13 · Data Integrity Test</a> · <a href="/articles/system-testing-integration-test">SYS-04 · Integration Test</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab A (code): 5 execution RC-01…03 — lỗi ví tạm thời, outage rồi việc mới, mất phản hồi kéo dài (R-PAY-03/04/06, R-NFR-05, R-STATE-03/06, R-ACCEPT-05). Lab B (script): logical backup/restore trên cluster PostgreSQL tạm — <code>examples/toi-noi-testing/articles/system-testing-recovery-test/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>PostgreSQL 18 · SQL Dump và Continuous Archiving/PITR; <code>SPEC.md</code>, <code>src/app/payment.ts</code>, <code>src/infra/fake-payment-gateway.ts</code> của lab</td></tr>
</tbody>
</table>
</div>

## 1. Bài toán và mục tiêu

### 1.1. Tình huống

Tới Nơi thu tiền chuyến `vixanh` khi tài xế bấm hoàn thành (R-PAY-03). Đường đi đó chạm ba thứ có thể hỏng độc lập: tiến trình ứng dụng, cơ sở dữ liệu và cổng ví bên ngoài. Mỗi thứ hỏng theo cách riêng, và "phục hồi" cũng mang nghĩa riêng:

- Ví lỗi vài trăm mili-giây rồi trở lại — phục hồi nghĩa là **retry đúng khoá**, không thu trùng.
- Ví ngừng vài phút — phục hồi nghĩa là **việc mới chạy được** sau khi ví trở lại, và việc cũ còn dở được **ghi nhận để xử lý**, không bị quên.
- Ai đó chạy nhầm `DELETE` trên DB — phục hồi nghĩa là **dựng lại dữ liệu từ bản sao**, biết chính xác mất gì, và DB khôi phục **ghi được** chứ không chỉ đọc được.

Một health check chỉ trả lời "tiến trình có phản hồi không". Recovery test phải trả lời "hệ thống đã trở về trạng thái nghiệp vụ chấp nhận được chưa, trong giới hạn thời gian và dữ liệu đã thống nhất".

### 1.2. Đối tượng và điều kiện tiên quyết

- Đã đọc [SYS-04](/articles/system-testing-integration-test) (timeout, retry, khoá idempotency `trip:<id>:fare`) và [SYS-13](/articles/system-testing-data-integrity-test) (đối soát bằng SQL).
- Lab A: Node.js ≥ 22.12 và dependency của `examples/toi-noi-testing` (PGlite chạy PostgreSQL dưới WASM, không cần server).
- Lab B: Windows + PowerShell, các binary PostgreSQL trên PATH: `initdb`, `pg_ctl`, `createdb`, `psql`, `pg_dump`, `pg_restore`. Hai lab **độc lập**: API của lab A không kết nối cluster của lab B.

### 1.3. Mục tiêu đo được

Sau bài này, bạn có thể:

1. Phân biệt retry, restart, restore, failover và reconciliation; nói được mỗi cơ chế **chưa** chứng minh điều gì.
2. Định nghĩa RTO/RPO kèm mốc đo cụ thể, không suy từ thời gian chạy test.
3. Viết recovery test theo chu trình bình thường → inject lỗi → gỡ lỗi → kiểm dữ liệu cũ **và** việc mới.
4. Nhận diện trạng thái "đã thu tiền nhưng chưa xác nhận" và giải thích vì sao không được retry mù.
5. Chạy một logical restore drill cô lập, đọc được marker nào trở lại, marker nào mất và vì sao.

## 2. Khái niệm và những chỗ dễ nhầm

### 2.1. Phục hồi cái gì?

| Cơ chế | Mục tiêu | Điều chưa được chứng minh chỉ bằng cơ chế đó |
|---|---|---|
| Retry | Thực hiện lại thao tác sau lỗi tạm thời | Không tạo tác dụng phụ trùng |
| Restart | Khởi động lại tiến trình | Dữ liệu bền vững, việc đang làm được tiếp tục |
| Restore | Dựng lại dữ liệu từ bản sao | Mọi giao dịch mới nhất đều có mặt |
| Failover | Chuyển phục vụ sang instance khác | Không split-brain, không mất dữ liệu |
| Reconciliation | Đối chiếu và xử lý trạng thái không khớp | Việc mới đã hoạt động bình thường |

Cột thứ ba quan trọng hơn cột thứ hai. Mỗi cơ chế giải một phần bài toán, và mỗi phần còn lại phải có phép kiểm riêng. Đội nào chỉ kiểm cột "mục tiêu" sẽ có một báo cáo toàn xanh và một khoảng trống không ai nhìn thấy.

### 2.2. Recovery test, backup job và resilience

**Recovery test** là phép kiểm chủ động: có failure model (hỏng cái gì, ở đâu, lúc nào), điều kiện bắt đầu/kết thúc và bằng chứng. Recovery test khác với việc "backup job báo success": job thành công chỉ chứng minh công cụ đã tạo ra một tệp. Phải **khôi phục, đọc, ghi và đối soát** mới có bằng chứng bản sao dùng được.

**Resilience** rộng hơn: chịu lỗi, suy giảm có kiểm soát (graceful degradation) và phục hồi. **Disaster recovery** xét thêm sự cố lớn như mất vùng hay mất hệ lưu trữ. Bài này nằm ở phần nhỏ nhất: một dependency giả và một logical restore cục bộ. Không diễn tập mất vùng, và bài sẽ nói rõ điều đó ở mọi chỗ cần nói.

### 2.3. Phân biệt với các bài trước

| Bài | Câu hỏi | Ví dụ khác biệt |
|---|---|---|
| [SYS-04 Integration](/articles/system-testing-integration-test) | Chỗ nối API – ví có hành xử đúng hợp đồng khi ví lỗi? | Một lần gọi lỗi ⇒ retry đúng khoá |
| [SYS-09 Stress](/articles/system-testing-stress-test) | Quá tải thì suy giảm và hồi phục ra sao? | Lỗi sinh ra **do tải** |
| [SYS-13 Data Integrity](/articles/system-testing-data-integrity-test) | Dữ liệu có nhất quán ở trạng thái hiện tại? | Đối soát chuyến – event – thu tiền |
| **SYS-14 Recovery** | Sau sự cố, hệ thống có **trở về** trạng thái nghiệp vụ chấp nhận được? | Outage xong, việc mới chạy; việc cũ còn dở được nhận diện |

SYS-04 kiểm một lỗi đơn lẻ. SYS-14 kiểm **chu trình**: có trước, trong và sau sự cố, và oracle nằm ở "sau".

## 3. RTO, RPO và cách đo trung thực

### 3.1. Định nghĩa có mốc

**RTO (Recovery Time Objective)** là mục tiêu thời gian phục hồi. Trước khi test phải chốt **mốc bắt đầu**: lúc nghiệp vụ ngừng đáp ứng, không phải lúc operator bắt đầu gõ lệnh restore. **Mốc kết thúc** là lúc nghiệp vụ đạt tiêu chí đã thống nhất, không phải lúc tiến trình vừa mở cổng.

**RPO (Recovery Point Objective)** là giới hạn mất dữ liệu, tính theo thời gian. Đo bằng các giao dịch **đã được xác nhận với người dùng** trước sự cố, so với điểm dữ liệu thực sự khôi phục được. Không dùng giờ sửa file backup. Và phải tìm **lỗ hổng** trong chuỗi giao dịch, không chỉ nhìn timestamp lớn nhất: dữ liệu khôi phục tới 09:57 nhưng thiếu một bản ghi lúc 09:40 vẫn là mất dữ liệu.

### 3.2. Ví dụ tính toán (giả định bài tập)

Giả định: RTO 15 phút, RPO 5 phút, **không thu trùng tiền**. Đây là ví dụ thiết kế, không phải SLO có trong SPEC.

- Sự cố lúc 10:00; nghiệp vụ hoạt động đủ tiêu chí lúc 10:12 ⇒ thời gian phục hồi **12 phút** (đạt RTO).
- Dữ liệu liên tục khôi phục tới 09:57 ⇒ khoảng mất **3 phút** (đạt RPO).
- Nhưng còn một khoản thu không rõ trạng thái ⇒ tiêu chí "không thu trùng, không thất thoát" **chưa** được chứng minh. Ghi tồn đọng; không tuyên bố "đã phục hồi hoàn toàn".

Ba con số cùng xanh không có nghĩa là xong, nếu tiêu chí thứ tư còn để ngỏ.

### 3.3. Timeline cần ghi

Năm mốc tối thiểu: `t_failure`, `t_detected`, `t_recovery_started`, `t_db_ready`, `t_business_verified`. Khoảng `t_failure → t_detected` thường dài nhất và hay bị bỏ khỏi báo cáo nhất. Đo duration trong một tiến trình bằng đồng hồ đơn điệu; ghép bằng chứng nhiều máy bằng UTC với đồng hồ đã đồng bộ.

`FixedClock` trong lab A giữ cho kết quả nghiệp vụ **tất định** (giá giờ thường lúc 09:00), không phải để đo thời gian sự cố. Đừng lấy nó làm đồng hồ RTO.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 250" role="img" aria-label="Dòng thời gian một sự cố. Trên trục: giao dịch cuối được backup lúc 09:57, sự cố lúc 10:00, phát hiện, bắt đầu phục hồi, DB sẵn sàng, và nghiệp vụ được xác minh lúc 10:12. RPO là khoảng từ điểm dữ liệu khôi phục tới sự cố; RTO là khoảng từ sự cố tới khi nghiệp vụ được xác minh. Chú thích: thời gian restore đo bởi script chỉ là một đoạn nhỏ bên trong RTO.">
<defs>
<marker id="sys14-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<line x1="40" y1="130" x2="890" y2="130" class="s-pk" stroke-width="2" marker-end="url(#sys14-arrow)"/>
<circle cx="140" cy="130" r="7" class="f-phys-soft s-phys"/><text x="140" y="160" text-anchor="middle" class="t-small">09:57</text><text x="140" y="176" text-anchor="middle" class="t-small">điểm dữ liệu khôi phục</text>
<circle cx="240" cy="130" r="8" class="f-warn-soft s-warn"/><text x="240" y="160" text-anchor="middle" class="t-b">t_failure</text><text x="240" y="176" text-anchor="middle" class="t-small">10:00</text>
<circle cx="360" cy="130" r="6" class="f-pk-soft s-pk"/><text x="360" y="160" text-anchor="middle" class="t-small">t_detected</text>
<circle cx="470" cy="130" r="6" class="f-pk-soft s-pk"/><text x="470" y="160" text-anchor="middle" class="t-small">t_recovery_started</text>
<circle cx="610" cy="130" r="6" class="f-pk-soft s-pk"/><text x="610" y="160" text-anchor="middle" class="t-small">t_db_ready</text>
<circle cx="780" cy="130" r="8" class="f-phys-soft s-phys"/><text x="780" y="160" text-anchor="middle" class="t-b">t_business_verified</text><text x="780" y="176" text-anchor="middle" class="t-small">10:12</text>
<rect x="140" y="60" width="100" height="34" rx="6" class="f-warn-soft s-warn"/><text x="190" y="82" text-anchor="middle">RPO thực: 3 phút</text>
<rect x="240" y="20" width="540" height="34" rx="6" class="f-pk-soft s-pk"/><text x="510" y="42" text-anchor="middle">RTO thực: 12 phút — tới khi nghiệp vụ đạt tiêu chí</text>
<rect x="490" y="80" width="110" height="30" rx="6" class="f-bg s-pk dash"/><text x="545" y="100" text-anchor="middle" class="t-small">restore + kiểm</text>
<text x="460" y="214" text-anchor="middle" class="t-small t-warn">restoreAndValidationMs của lab B chỉ là ô nét đứt — không phải RTO ứng dụng</text>
<text x="460" y="236" text-anchor="middle" class="t-small">khoản thu còn mơ hồ ⇒ tiêu chí "không thất thoát" vẫn mở dù RTO/RPO đều đạt</text>
</svg>
</div>
<figcaption><b>Hình 1 · RTO, RPO và thứ mà script thực sự đo.</b> Các mốc và con số là ví dụ giả định của mục 3.2. Phần phát hiện sự cố, cấp hạ tầng, đổi endpoint và kiểm hành trình nằm ngoài phép đo của lab B.</figcaption>
</figure>

## 4. Failure model, test basis và tiêu chí

### 4.1. Failure model

| Sự cố | Phép kiểm trong bài | Phạm vi |
|---|---|---|
| Ví lỗi tạm thời một lần | `error`/`timeout`/`lost_response` rồi thành công trong ngân sách retry | Đã tự động hoá (RC-01) |
| Ví ngừng rồi trở lại | Chuyến mới thu được; chuyến cũ vẫn `failed` | Đã tự động hoá (RC-02) |
| Mất phản hồi sau khi đã trừ tiền, cả ba lần | Ví đã thu, DB ghi `failed` | Đã tự động hoá (RC-03) |
| Xoá nhầm dữ liệu | Restore snapshot vào DB mới, kiểm marker và ghi mới | Script PostgreSQL riêng (lab B) |
| Mất tiến trình giữa commit và gọi ví | Restart, đối soát, replay có kiểm soát | Bài tập mở rộng, chưa chạy |
| Mất máy/vùng, thiếu WAL, failover | Cluster và storage riêng | Chưa chạy |

Viết failure model **trước** khi viết test buộc nhóm trả lời câu "chúng ta đang **không** kiểm cái gì" — hai dòng cuối bảng là phần quan trọng không kém bốn dòng đầu.

### 4.2. Test basis

Yêu cầu trong `SPEC.md` mà lab A dựa vào:

- **R-PAY-03** — chuyến `vixanh` hoàn thành ⇒ gọi VíXanh thu đúng `fare` với khoá `trip:<id>:fare`.
- **R-PAY-04 / R-NFR-05** — mỗi lần gọi hết giờ sau 3 giây; hết giờ hoặc lỗi tạm thời ⇒ thử lại tối đa 2 lần (tổng 3 lần gọi) **cùng khoá**; cổng đảm bảo cùng khoá chỉ trừ tiền một lần; "không bao giờ trừ tiền hai lần cho cùng một khoản".
- **R-PAY-06** — thu thất bại không chặn hoàn thành: chuyến vẫn `completed`, `paymentStatus = failed`, API trả 200.
- **R-STATE-03** — chuyển trạng thái không hợp lệ ⇒ `INVALID_TRANSITION` (409), chuyến giữ nguyên. Đây là lý do gọi `complete` lần hai cho chuyến đã `completed` bị 409.
- **R-STATE-06** — mỗi lần đổi trạng thái ghi một dòng `trip_events`.
- **R-ACCEPT-05** — chuyến `completed` ⇒ tài xế về `online`.

Lab B dùng bảng `recovery_probe` tổng hợp, **không** gán requirement Tới Nơi cho nó. Đó là thí nghiệm về cơ chế backup/restore, chưa phải restore dữ liệu nghiệp vụ.

### 4.3. Oracle nhiều phía

Sau sự cố, một nguồn duy nhất không đủ tin. Mỗi test case trong lab A kiểm đồng thời:

1. **HTTP** — mã trả về đúng R-PAY-06 / R-STATE-03.
2. **DB** — `status`, `payment_status`, số dòng `trip_events` `completed`, trạng thái tài xế.
3. **Phía ví** — số lần gọi, khoá đã dùng, khoá đã trừ tiền, tổng tiền đã trừ.

Phía thứ ba là phía mà dashboard ứng dụng **không** nhìn thấy, và cũng là nơi RC-03 lộ ra mâu thuẫn.

### 4.4. Tiêu chí bắt đầu và kết thúc

- **Entry:** dependency cài theo lockfile; SPEC đúng phiên bản; dữ liệu seed, không dữ liệu thật; với lab B: binary PostgreSQL có trên PATH và một cổng loopback trống.
- **Exit của bài tập:** 5 execution lab A pass, typecheck pass; lab B khôi phục đúng `1:35000`, ghi mới được, cluster đã dừng. Nếu RC-03 còn `failed` ⇒ ghi rõ "cần reconciliation". Exit này **không** cho phép tuyên bố release đạt RTO/RPO.

## 5. Thiết kế test case — lab A

### 5.1. Tiền điều kiện chung

Mỗi test case: reset seed, dựng Express app mới; khách C001, tài xế D101, 09:00 giờ VN ngày 04/11/2026, xe máy 5 km tại SG, trả bằng VíXanh ⇒ cước 35.000 đ (12.000 + 4.500 × 5 = 34.500, làm tròn lên 1.000 theo R-FARE-08). Chuyến được đưa tới `in_transit` qua đúng API tài xế trước khi inject lỗi. Cấu hình thanh toán rút gọn: timeout **30 ms**, tối đa **2** retry.

### 5.2. Bảng test case

| ID | Requirement/rủi ro | Hành động | Expected | Cleanup |
|---|---|---|---|---|
| RC-01 × 3 | R-PAY-03/04, R-NFR-05; lỗi thoáng qua | `failNext(mode)` với mode ∈ {`error`, `timeout`, `lost_response`}, rồi `complete` | 200; DB `completed/paid`; đúng 1 event `completed`; D101 `online`; 2 lần gọi **cùng khoá**; trừ đúng 35.000 một lần | reset seed |
| RC-02 | R-PAY-06, R-STATE-03; outage rồi việc mới | `setMode('timeout')`, complete chuyến cũ; `setMode('none')`, tạo và complete chuyến mới; complete lại chuyến cũ | Chuyến cũ `failed`, 3 lần gọi, 0 đ; chuyến mới `paid`; complete lại chuyến cũ ⇒ 409, **không** gọi ví thêm (tổng 4 lần gọi) | reset seed |
| RC-03 | R-PAY-04/06, R-NFR-05; mất phản hồi kéo dài | `setMode('lost_response')`, complete; gỡ lỗi; complete lại | 200, DB `failed`; ví **đã** trừ 35.000 với khoá của chuyến; complete lại ⇒ 409; vẫn 3 lần gọi, vẫn 35.000 | reset seed |

### 5.3. Vì sao ba chế độ lỗi khác nhau

Ví giả (`src/infra/fake-payment-gateway.ts`) phân biệt rõ ba kiểu lỗi — và vị trí của dòng trừ tiền quyết định tất cả:

```ts
const failure = this.queue.shift() ?? this.mode;
if (this.latencyMs > 0) await sleep(this.latencyMs);

if (failure === 'decline') return { status: 'declined', chargeId: null };
if (failure === 'error') throw new PaymentUnavailableError();
if (failure === 'timeout') return never();

const record = this.applyCharge(request); // idempotent theo khoá
if (failure === 'lost_response') return never();
return { status: 'succeeded', chargeId: record.chargeId };
```

- `error` và `timeout`: **chưa** trừ tiền. Retry là an toàn tự nhiên.
- `lost_response`: **đã** trừ tiền rồi mới im lặng. Đây là kiểu lỗi nguy hiểm nhất trong thực tế ("request tới nơi, response lạc đường"). Retry chỉ an toàn vì cùng khoá ⇒ `applyCharge` không trừ lần hai.

`failNext` tiêu thụ hàng đợi lỗi cho N lần gọi kế tiếp rồi quay về chế độ lâu dài; `setMode` đặt chế độ lâu dài. RC-01 cần "lỗi một lần rồi thôi" ⇒ `failNext`. RC-02/03 cần "ngừng hẳn cho tới khi gỡ" ⇒ `setMode`.

## 6. Thực hành A — phục hồi dependency thanh toán

### 6.1. Chạy lab

Từ `examples/toi-noi-testing`:

```powershell
npm ci
npm test -- articles/system-testing-recovery-test
npm run typecheck
```

### 6.2. Bước 1 — Lắp ráp với ngân sách retry rút gọn

```ts
beforeAll(async () => {
  system = await createPgliteSystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)),
    config: { payment: { timeoutMs: 30, maxRetries: 2 } } });
});
beforeEach(async () => { await system.reset(); app = createApp(system.deps, { logErrors: false }); });
afterAll(async () => { await system?.db?.close(); });
```

`timeoutMs: 30` thay vì 3 giây của SPEC để ba lần hết giờ chỉ mất ~90 ms. Hệ quả: lab kiểm **logic** đếm lần thử và giữ khoá, không chứng minh giá trị mặc định 3 giây, cũng không nói gì về latency thật của nhà cung cấp.

Retry nằm trong `chargeWithRetry` (`src/app/payment.ts`) — lỗi lập trình bị ném lại, không bị nuốt:

```ts
for (let attempt = 1; attempt <= maxAttempts; attempt++) {
  try {
    const result = await withTimeout(gateway.charge(request), options.timeoutMs);
    if (result.status === 'succeeded') {
      return { status: 'paid', attempts: attempt, failure: null, chargeId: result.chargeId };
    }
    return { status: 'failed', attempts: attempt, failure: 'declined', chargeId: null };
  } catch (error) {
    if (error instanceof PaymentTimeoutError) lastFailure = 'timeout';
    else if (error instanceof PaymentUnavailableError) lastFailure = 'unavailable';
    else throw error; // lỗi lập trình — không nuốt
  }
}
```

### 6.3. Bước 2 — Oracle DB dùng chung

```ts
async function stored(id: string, payment: string) {
  expect((await system.db!.query('SELECT status,payment_status FROM trips WHERE id=$1', [id])).rows)
    .toEqual([{ status: 'completed', payment_status: payment }]);
  expect((await system.db!.query("SELECT count(*)::int AS n FROM trip_events WHERE trip_id=$1 AND status='completed'", [id])).rows)
    .toEqual([{ n: 1 }]);
  expect((await system.db!.query("SELECT status FROM drivers WHERE id='D101'")).rows).toEqual([{ status: 'online' }]);
}
```

Đếm đúng **một** event `completed` chặn một lỗi phục hồi kinh điển: retry ở tầng sai (gọi lại cả use case thay vì chỉ gọi lại ví) sẽ ghi event hai lần. Kiểm tài xế `online` chặn lỗi ngược lại: chuyến xong nhưng tài xế bị kẹt `busy` vì lỗi thanh toán làm hỏng phần cập nhật sau.

### 6.4. Bước 3 — RC-01: lỗi một lần rồi phục hồi

```ts
it.each(['error', 'timeout', 'lost_response'] as const)(
  'RC-01 · one %s then recovery within retry budget', async mode => {
    const id = await inTransit('sys14-transient');
    system.payments.failNext(mode);
    await complete(id).expect(200);
    await stored(id, 'paid');
    expect(system.payments.calls).toHaveLength(2);
    expect(system.payments.calls.map(c => c.idempotencyKey)).toEqual([`trip:${id}:fare`, `trip:${id}:fare`]);
    expect(system.payments.totalCharged()).toBe(35_000);
    expect(system.payments.chargedKeys()).toEqual([`trip:${id}:fare`]);
  });
```

Với `lost_response`, lần gọi đầu **đã** trừ tiền; lần thứ hai cùng khoá trả về bản ghi cũ. Tổng vẫn 35.000. Nếu ai đó "cải tiến" retry bằng cách sinh khoá mới mỗi lần, assertion `totalCharged()` sẽ đỏ với 70.000 — đó là lỗi mà phép thử này tồn tại để bắt.

Chú ý: `calls` dài 2 chỉ đúng **vì** chỉ inject một lỗi. Không chuyển nguyên assertion "đúng N lần gọi" sang môi trường có retry ở nhiều tầng.

### 6.5. Bước 4 — RC-02: việc mới sau outage

```ts
const oldId = await inTransit('sys14-outage-old');
system.payments.setMode('timeout');
await complete(oldId).expect(200);
await stored(oldId, 'failed');
expect(system.payments.calls).toHaveLength(3);
expect(system.payments.totalCharged()).toBe(0);
system.payments.setMode('none');
const newId = await inTransit('sys14-outage-new');
await complete(newId).expect(200);
await stored(newId, 'paid');
await stored(oldId, 'failed');
await complete(oldId).expect(409);
expect(system.payments.calls).toHaveLength(4);
expect(system.payments.chargedKeys()).toEqual([`trip:${newId}:fare`]);
expect(system.payments.totalCharged()).toBe(35_000);
```

Hai điều được chứng minh cùng lúc. Một: hệ thống **nhận việc mới** sau outage — không bị kẹt trạng thái lỗi, không có circuit breaker nào mở mãi. Hai: chuyến cũ **không** tự được thu lại khi ví trở lại; gọi complete lần nữa bị chặn bởi R-STATE-03 và ví không nhận thêm lần gọi nào (4 = 3 của chuyến cũ + 1 của chuyến mới). Chuyến cũ là việc tồn đọng **có nhãn**, chờ quy trình xử lý.

### 6.6. Bước 5 — RC-03: khoảng trống được ghi nhận

```ts
const id = await inTransit('sys14-ambiguous');
system.payments.setMode('lost_response');
await complete(id).expect(200);
await stored(id, 'failed');
expect(system.payments.calls).toHaveLength(3);
expect(system.payments.chargedKeys()).toEqual([`trip:${id}:fare`]);
expect(system.payments.totalCharged()).toBe(35_000);
system.payments.setMode('none');
await complete(id).expect(409);
await stored(id, 'failed');
expect(system.payments.calls).toHaveLength(3);
expect(system.payments.totalCharged()).toBe(35_000);
```

Đây là test case quan trọng nhất của bài, và nó **pass vì nó ghi lại đúng một mâu thuẫn**: ví đã thu 35.000, DB nói `failed`. Theo R-PAY-06, API trả 200 và chuyến `completed` — đúng đặc tả. Nhưng `failed` ở đây nghĩa là "**không xác nhận được**", không phải "chắc chắn chưa thu".

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">RC-03 xanh không có nghĩa là đã phục hồi</span>
<p>Lab chưa có worker đối soát, chưa có API hỏi trạng thái giao dịch theo khoá ở phía nhà cung cấp, chưa có ledger thật. PASS của RC-03 là bằng chứng hệ thống <b>hành xử như SPEC mô tả</b> và khoảng trống được phơi ra, không phải bằng chứng nghiệp vụ tài chính đã phục hồi đủ. Đừng làm test "xanh hơn" bằng cách <code>UPDATE trips SET payment_status='paid'</code>: cần bằng chứng từ nhà cung cấp và một quy trình cập nhật có kiểm toán. Đừng cấp khoá mới để thu lại khoản cũ khi kết quả còn mơ hồ — đó chính là cách khách bị trừ hai lần.</p>
</div>

### 6.7. Kết quả: mong đợi và thực tế

**Mong đợi:** 5 execution pass (RC-01 × 3, RC-02, RC-03), không skip; typecheck exit 0.

**Thực tế — báo cáo gốc** (13/11/2026, Windows, Node.js 24.15.0, Vitest 3.2.7, PGlite 0.3.16):

```text
Test Files  1 passed (1)
     Tests  5 passed (5)
tổng 3,35 s · phần test 2,17 s · typecheck exit 0
```

**Thực tế — chạy lại khi biên tập bài** (13/11/2026, Windows 11, Node.js v22.18.0, Vitest 3.2.7), lệnh `npx vitest run articles/system-testing-recovery-test`:

```text
✓ RC-02 · new work recovers after outage; old failed payment remains unresolved  364ms
✓ RC-03 · persistent lost responses leave a charged-but-failed discrepancy       309ms

Test Files  1 passed (1)
     Tests  5 passed (5)
  Duration  5.38s (transform 362ms, collect 1.15s, tests 3.71s, prepare 187ms)
```

Lần chạy lại diễn ra khi máy đang chạy song song các lab khác, nên thời gian dài hơn báo cáo gốc. Thời gian ở đây chỉ để tham khảo, **không** phải số đo hiệu năng hay RTO. Typecheck không chạy lại trong lần biên tập; bài dẫn kết quả PASS từ báo cáo gốc.

## 7. Thực hành B — backup/restore thật trên dữ liệu tổng hợp

### 7.1. Chạy script

Từ root repository, trên Windows:

```powershell
& ./examples/toi-noi-testing/articles/system-testing-recovery-test/restore-drill.ps1
# Nếu cổng đang bận, chọn cổng khác:
& ./examples/toi-noi-testing/articles/system-testing-recovery-test/restore-drill.ps1 -Port 55485
```

Script tạo thư mục GUID dưới TEMP, `initdb` một cluster **mới**, chỉ listen `127.0.0.1`, cổng mặc định 55484. Script không dùng cluster PostgreSQL đang có, không xoá database nào của bạn. `--auth=trust` chỉ chấp nhận được vì đây là cluster loopback ngắn hạn chứa dữ liệu giả; không dùng cấu hình này ở môi trường dùng chung. Cluster dừng trong khối `finally`; thư mục bằng chứng được giữ lại.

### 7.2. Trình tự thí nghiệm

1. Tạo `source_drill`, bảng `recovery_probe`, commit **marker 1** (35.000).
2. `pg_dump -Fc` lấy logical backup.
3. Commit **marker 2** (42.000) **sau** backup; kiểm nguồn có đúng hai bản ghi.
4. Mô phỏng xoá nhầm: `DELETE` chỉ trong database vừa tạo.
5. Tạo `restored_drill` từ `template0`; `pg_restore --exit-on-error --single-transaction` vào DB mới.
6. Kiểm chính xác `1:35000`: marker 1 trở lại, marker 2 **không** có vì nằm ngoài snapshot.
7. Ghi **marker 3**, đếm lại để chứng minh DB khôi phục ghi được; lưu `result.json` và dừng cluster.

Phần lõi của script:

```powershell
$timer = [Diagnostics.Stopwatch]::StartNew()
Invoke-Pg 'createdb' ($conn + @('-T', 'template0', 'restored_drill')) | Out-Null
Invoke-Pg 'pg_restore' ($conn + @('--exit-on-error', '--single-transaction', '-d', 'restored_drill', $dumpPath)) | Out-Null
$actual = Invoke-Pg 'psql' ($conn + @('-X', '-At', '-v', 'ON_ERROR_STOP=1', '-d', 'restored_drill', '-c', "SELECT id::text || ':' || amount::text FROM recovery_probe ORDER BY id;"))
if ("$actual".Trim() -ne '1:35000') { throw "Unexpected restored markers: $actual" }
Invoke-Pg 'psql' ($conn + @('-X', '-v', 'ON_ERROR_STOP=1', '-d', 'restored_drill', '-c', 'INSERT INTO recovery_probe(id,amount) VALUES (3,15000);')) | Out-Null
$count = Invoke-Pg 'psql' ($conn + @('-X', '-At', '-d', 'restored_drill', '-c', 'SELECT count(*) FROM recovery_probe;'))
if ("$count".Trim() -ne '2') { throw 'Post-restore write check failed' }
$timer.Stop()
```

Ba lựa chọn đáng để ý:

- **Restore vào DB mới**, không `--clean` lên DB có sẵn. Nếu restore hỏng giữa chừng, nguồn (hay những gì còn lại của nó) vẫn nguyên để điều tra.
- `--exit-on-error --single-transaction`: restore được cả hoặc không gì cả. Không mở traffic cho một DB khôi phục một nửa.
- Oracle là **chuỗi chính xác** `1:35000`, không phải "count > 0". Cách này bắt được cả trường hợp restore thiếu dòng lẫn trường hợp restore "thừa" (ví dụ dump lấy nhầm thời điểm).

`Invoke-Pg` ném lỗi khi exit code khác 0, nên mọi bước đều dừng sớm nếu sai. Khi `pg_ctl start` lỗi vì cổng bận: đọc log, chọn cổng khác — **không** dừng tiến trình đang giữ cổng của người khác.

### 7.3. Kết quả: mong đợi và thực tế

**Mong đợi:** exit code 0; marker khôi phục là 1; marker 2 thiếu; ghi được marker 3; cluster dừng.

**Thực tế — từ báo cáo gốc** (13/11/2026, Windows, PostgreSQL 18.3, cluster mới trên 127.0.0.1:55484, backup custom format). Script **không** chạy lại khi biên tập bài này:

```json
{
  "postgres": "pg_dump (PostgreSQL) 18.3",
  "recoveredMarker": 1,
  "missingPostBackupMarker": 2,
  "newWriteMarker": 3,
  "restoreAndValidationMs": 774,
  "applicationRtoMeasured": false,
  "rpoTimeMeasured": false
}
```

Nguồn có hai marker trước `DELETE`; DB đích khôi phục đúng `1:35000`; insert marker 3 thành công, count cuối bằng 2; log xác nhận "database system is shut down".

Báo cáo cũng ghi trung thực rằng **hai lần thử đầu không hoàn thành**: trên Windows, PowerShell giữ pipe output của tiến trình nền khiến lệnh khởi động không trả về. Cách sửa (khởi động `pg_ctl` bằng `Start-Process` ẩn, output ra file, `WaitForExit` chỉ chờ `pg_ctl` chứ không chờ cây tiến trình PostgreSQL) chính là đoạn comment trong script. Kết quả trên thuộc lần chạy sau khi sửa; không lấy số từ lần lỗi.

### 7.4. Đọc con số 774 ms

`restoreAndValidationMs` tính từ trước `createdb` đích tới sau kiểm ghi mới. Con số này **không** phải RTO ứng dụng: không gồm phát hiện sự cố, cấp hạ tầng, đổi endpoint, hay kiểm hành trình Tới Nơi. Script cũng không đo RPO theo thời gian — chỉ xác nhận "một marker sau backup bị mất". Hai cờ `applicationRtoMeasured: false` và `rpoTimeMeasured: false` có mặt trong kết quả **chính là để** người đọc báo cáo sau này không hiểu nhầm.

Fixture này không restore schema hay dữ liệu Tới Nơi. Mở rộng đúng cách cần thêm quote, trip, trip_events, idempotency, đối chiếu theo khoá, số tiền và snapshot giá như SYS-13, rồi chạy hành trình nghiệp vụ trên backend PostgreSQL tương ứng.

## 8. Đọc thất bại và ra quyết định

| Dấu hiệu | Giả thuyết cần kiểm | Bằng chứng tiếp theo |
|---|---|---|
| RC-01 `totalCharged` = 70.000 | Retry sinh khoá mới, hoặc ví giả mất tính idempotent | `calls.map(c => c.idempotencyKey)`, diff `payment.ts` |
| RC-01 có 2 event `completed` | Retry ở tầng use case thay vì tầng gọi ví | Trình tự `trip_events`, call stack |
| RC-02 chuyến mới `failed` | Trạng thái lỗi bị giữ sau khi ví trở lại (cache, breaker) | Chế độ ví lúc gọi, số lần gọi của chuyến mới |
| RC-02 complete lại chuyến cũ trả 200 | Mất kiểm R-STATE-03 ⇒ nguy cơ thu lại | Trạng thái chuyến trước lệnh, `calls` sau lệnh |
| RC-03 `totalCharged` = 0 | Ví giả đổi hành vi `lost_response` — test mất ý nghĩa | Diff `fake-payment-gateway.ts` |
| Lab B: marker trả về `1:35000,2:42000` | Dump lấy sau marker 2, hoặc restore nhầm nguồn | Thứ tự lệnh, `probe.dump` timestamp, log |
| Lab B: start timeout | Cổng bận, pipe bị giữ | `start.err`, `postgres.log`, đổi `-Port` |

Lỗi môi trường (thiếu binary, cổng bận) báo **blocked** kèm log; lỗi sản phẩm báo **failed** với request, expected/actual, build và dữ liệu tái hiện. Một lần pass sau khi chạy lại không xoá lần lỗi trước.

Ví dụ quyết định viết được từ bài này: *"Retry trong ngân sách và tiếp nhận việc mới sau outage: đạt trong lab. Khoản thu mơ hồ sau mất phản hồi kéo dài: **chưa có cơ chế đối soát** — rủi ro tồn, cần người phụ trách và hạn xử lý trước khi bật VíXanh cho toàn bộ thành phố. Logical restore một bảng tổng hợp: đạt; chưa chứng minh RTO/RPO ứng dụng."*

## 9. Mở rộng nâng cao

### 9.1. Từ logical restore tới PITR

`pg_dump` cho một snapshot nhất quán của **một database**; roles và tablespaces cần kế hoạch riêng (`pg_dumpall --globals-only` hoặc tương đương). Có dump không có nghĩa có đủ cấu hình, chứng thư, quyền và secret để phục vụ ứng dụng — bài [Backup logic với pg_dump](/articles/postgresql-pg-dump-restore) đi sâu phần đối soát quyền.

PITR dùng **base backup + chuỗi WAL liên tục** để phục hồi tới một thời điểm chọn. Không thể lấy một `pg_dump` rồi replay WAL lên đó. Cần kiểm archive đủ, đúng timeline, đúng cấu hình phục hồi; thiếu một đoạn WAL có thể khiến không đạt được điểm đích. Phần này là mở rộng **chưa chạy** trong SYS-14; xem [Khôi phục theo thời điểm với base backup và WAL](/articles/postgresql-pitr-base-backup-wal).

Với `DELETE` nhầm lúc 10:00, phục hồi về 09:59:59 cũng **xoá luôn** các thay đổi hợp lệ sau đó trên những bảng khác. Cần kế hoạch đối soát và đưa lại phần dữ liệu cần giữ, không chỉ "restore là xong".

### 9.2. Failover

Kiểm **fencing** để primary cũ không tiếp tục ghi (split-brain), replication lag tại thời điểm chuyển, client reconnect, và tính duy nhất của tác dụng phụ ra ngoài (một khoản thu không được phát lại từ cả hai phía). Failback cần diễn tập riêng — nó hiếm khi đối xứng với failover.

### 9.3. Điểm cắt lỗi

Inject lỗi ở các điểm khác nhau cho ra oracle khác nhau:

| Điểm cắt | Câu hỏi oracle |
|---|---|
| Trước commit | Không có dấu vết gì? Client retry an toàn? |
| Sau commit, trước response | Client retry với cùng `Idempotency-Key` có nhận lại đúng kết quả (R-IDEM-02)? |
| Sau khi ví trừ tiền, trước khi lưu `paid` | Ai biết khoản thu tồn tại? Đối soát tìm ra nó bằng cách nào? |
| Worker đang giữ job | Job có được nhả, chạy lại đúng một lần? |

Retry cả giao dịch phải có giới hạn, backoff/jitter và theo dõi **retry amplification** (mỗi tầng retry 3 lần ⇒ 3 tầng = 27 lần gọi xuống đáy). Lab hiện **không** chứng minh các cơ chế này.

### 9.4. Bền vững qua restart

Khoá idempotency và kết quả của nó phải sống qua restart theo thiết kế (R-IDEM-05 ghi rõ bản lab giữ khoá trong vòng đời tiến trình; hệ thống đích tối thiểu 24 giờ). Ví giả lưu charge **trong RAM**, nên dựng lại ví giả không phải bằng chứng nhà cung cấp không thu trùng. Bằng chứng phải nằm **ngoài** tiến trình bị lỗi. Mẫu outbox/transaction cho phần này: [Idempotency, outbox và transaction trong PostgreSQL](/articles/postgresql-idempotency-outbox-transaction).

### 9.5. Backlog sau phục hồi

Dependency trở lại chưa đủ nếu hàng đợi vẫn phình. Đo tốc độ xử lý tồn, tuổi của job cũ nhất, lỗi lặp và công bằng giữa việc mới/cũ. Đặt **khoảng ổn định** sau phục hồi (ví dụ: 10 phút liên tục không tăng tồn) thay vì chấp nhận "một request thành công".

### 9.6. Backup hỏng và hạ tầng

Thử archive thiếu, sai checksum hoặc thiếu quyền giải mã — trên bản sao dành riêng cho diễn tập. Kết quả cần **fail rõ**, không mở traffic. Bài hiện chưa có thử nghiệm này. Lab B đặt backup và data trên **cùng máy**, nên không chứng minh chịu được mất ổ đĩa; diễn tập thật phải restore vào môi trường độc lập với nguồn.

## 10. Runbook, bằng chứng và anti-pattern

### 10.1. Runbook tối thiểu

Người điều phối, người xác nhận dữ liệu, phạm vi inject, điều kiện dừng, backup được chọn (ID/hash), cách chặn tác dụng phụ ra hệ ngoài (payment sandbox, tắt gửi SMS), timeline, tiêu chí mở traffic. Runbook chung cho vận hành PostgreSQL: [Thực hành bàn giao runbook phục hồi](/articles/postgresql-thuc-hanh-runbook-phuc-hoi).

### 10.2. Mẫu bằng chứng

| Trường | Nội dung cần lưu |
|---|---|
| Build và dữ liệu | Commit, runtime, phiên bản DB, seed, backup ID/hash |
| Sự cố | Failure mode, thời điểm inject/gỡ lỗi, request ID |
| Kết quả | HTTP + DB + hệ thanh toán, không chỉ health check |
| Phục hồi | Timeline 5 mốc, marker/khoá thiếu, khoản thu mơ hồ, số job tồn |
| Quyết định | Đạt/chưa đạt mục tiêu, giới hạn, việc phải xử lý và người chịu trách nhiệm |

### 10.3. Anti-pattern

- **Health check = phục hồi.** Health của lab chỉ trả `{status:'ok'}`; nó xanh suốt cả RC-02 lẫn RC-03.
- **Backup thành công = phục hồi được.** Chưa restore thì chưa biết.
- **Retry mù với khoá mới** khi kết quả mơ hồ ⇒ thu trùng.
- **Sửa tay `payment_status`** để báo cáo đẹp ⇒ mất dấu vết kiểm toán.
- **Gọi thời gian restore là RTO** ⇒ bỏ qua phần phát hiện và xác minh nghiệp vụ, thường là phần dài nhất.
- **`sleep` cố định chờ "hệ thống ổn"** thay vì điều kiện quan sát được có deadline.
- **Diễn tập trên cluster dùng chung** với `--clean` ⇒ biến bài tập thành sự cố thật.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn bằng chứng của SYS-14</span>
<p>Lab A: PGlite một kết nối, ví giả trong RAM, timeout 30 ms thay vì 3 giây — kiểm logic, không đo SLO và không kiểm crash giữa commit và gọi ví. Lab B: một bảng tổng hợp, cùng máy, logical dump — chưa restore schema Tới Nơi, chưa PITR, failover, backup hỏng, mất storage hay mất vùng. <code>restoreAndValidationMs</code> không phải RTO; RPO thời gian chưa được đo. Đánh đổi: lab rẻ, chạy lại được trong vài giây và phơi đúng khoảng trống đối soát; cái giá là mọi kết luận về môi trường thật vẫn cần một diễn tập riêng có hạ tầng tách biệt.</p>
</div>

## 11. Bài tập và câu hỏi tự kiểm

### Bài tập

**Bài 1 (cơ bản).** Thêm test case: `failNext('timeout', 'timeout')` rồi complete. Viết đầy đủ expected cho HTTP, DB và phía ví.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Hai lần hết giờ rồi thành công ở lần thứ ba — vẫn trong ngân sách (tối đa 3 lần gọi, R-PAY-04). Expected: 200; <code>stored(id, 'paid')</code>; <code>calls</code> dài 3, cả ba cùng khoá <code>trip:&lt;id&gt;:fare</code>; <code>totalCharged()</code> = 35.000; <code>chargedKeys()</code> chỉ có khoá của chuyến. Thêm <code>failNext</code> ba lần thì case chuyển thành "hết ngân sách" và expected đổi sang <code>failed</code>, giống RC-02.</p>
</details>

**Bài 2 (trung bình).** Đo RPO theo thời gian cho restore drill.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Lưu <b>bên ngoài</b> DB nguồn (ví dụ tệp JSON trong thư mục evidence) chuỗi marker đã được ACK cùng <code>committed_at</code> (bảng đã có cột này), mốc sự cố và danh sách marker khôi phục. RPO = mốc sự cố − thời điểm commit của marker liên tục cuối cùng có mặt. Tìm cả lỗ hổng giữa chuỗi. Không dùng giờ <code>pg_dump</code> kết thúc làm điểm dữ liệu: snapshot của dump được chụp lúc bắt đầu.</p>
</details>

**Bài 3 (nâng cao).** Thiết kế worker giải quyết RC-03 và phép thử cho nó.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Cần mở rộng port thanh toán (truy vấn giao dịch theo khoá) và SPEC trước — đây <b>không</b> phải hành vi sẵn có. Worker: chọn chuyến <code>completed</code> + <code>failed</code> + <code>vixanh</code>; hỏi ví theo khoá cũ <code>trip:&lt;id&gt;:fare</code>; nếu có giao dịch khớp số tiền và khách ⇒ cập nhật <code>paid</code> kèm bản ghi kiểm toán; nếu không ⇒ giữ <code>failed</code> và đưa vào hàng xử lý thủ công. Phép thử: restart worker giữa "đã hỏi ví" và "đã ghi DB" rồi chạy lại ⇒ vẫn đúng một cập nhật, không có lần thu mới, không cấp khoá mới.</p>
</details>

**Bài 4 (nâng cao).** Restore một fixture có chuyến đang `in_transit` rồi tiếp tục hành trình.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Đối chiếu sau restore: tài xế <code>busy</code> đúng một chuyến đang chạy (R-ACCEPT-05), snapshot giá giữ nguyên, <code>trip_events</code> đầy đủ theo R-STATE-06, bản ghi idempotency còn. Sau đó complete qua API với ví sandbox/giả, kiểm thu đúng một lần. Không trỏ cấu hình restore tới cổng thanh toán thật.</p>
</details>

**Bài 5 (nâng cao).** Thêm thử nghiệm archive không đọc được.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Sao chép <code>probe.dump</code>, cắt cụt hoặc đổi vài byte ở bản sao. <code>pg_restore --exit-on-error --single-transaction</code> vào DB mới phải exit khác 0 và DB đích không được dùng. Giữ dump gốc làm nhóm đối chứng để chứng minh lỗi đến từ archive chứ không từ môi trường.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** RC-03 pass. Có thể kết luận "lỗi mất phản hồi đã được xử lý" không?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>Không.</b> RC-03 chứng minh hệ thống hành xử theo SPEC (200, <code>completed</code>, <code>failed</code>, không thu thêm) và <b>phơi ra</b> mâu thuẫn ví đã thu – DB ghi failed. Không có cơ chế nào giải quyết mâu thuẫn đó trong lab. Kết luận đúng: "khoảng trống đối soát được xác nhận, cần xử lý".</p>
</details>

**Câu 2.** Vì sao RC-02 kiểm complete lại chuyến cũ trả 409 **và** số lần gọi ví vẫn là 4?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>409 (R-STATE-03) chứng minh API chặn chuyển trạng thái. Số lần gọi vẫn 4 chứng minh không có tác dụng phụ trước khi bị chặn — một hệ thống có thể trả 409 <b>sau khi</b> đã gọi ví. Hai assertion trả lời hai câu hỏi khác nhau.</p>
</details>

**Câu 3.** Restore drill báo `restoreAndValidationMs: 774`. Mục tiêu RTO là 15 phút. Đạt chưa?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>Chưa biết.</b> 774 ms chỉ gồm tạo DB đích, restore một bảng nhỏ và kiểm. RTO tính từ lúc nghiệp vụ ngừng tới lúc nghiệp vụ được xác minh — gồm phát hiện, cấp hạ tầng, đổi endpoint, kiểm hành trình. Script tự ghi <code>applicationRtoMeasured: false</code>.</p>
</details>

**Câu 4.** Vì sao script restore vào `restored_drill` mới thay vì `pg_restore --clean` lên `source_drill`?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Restore vào đích mới giữ nguyên nguồn để điều tra và so sánh; nếu restore hỏng giữa chừng, không có gì bị ghi đè. Trên môi trường dùng chung, <code>--clean</code> trỏ nhầm DB là cách biến diễn tập thành sự cố thật. Kèm <code>--single-transaction</code>, đích hoặc đầy đủ hoặc trống.</p>
</details>

**Câu 5.** Một đồng nghiệp đề xuất: "khi ví trở lại, tự động gọi lại thu tiền cho mọi chuyến `failed` với khoá mới `trip:<id>:fare:retry`". Điều gì sai?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Với chuyến như RC-03, ví <b>đã</b> thu bằng khoá cũ; khoá mới không được nhà cung cấp coi là cùng giao dịch ⇒ khách bị trừ hai lần, vi phạm R-NFR-05. Phải hỏi trạng thái theo <b>khoá cũ</b> trước, chỉ thu khi chắc chắn chưa có giao dịch, và có kiểm toán. Ngoài ra chuyến <code>failed</code> do <code>declined</code> không được retry theo R-PAY-04.</p>
</details>

## 12. Tài liệu tham khảo

Tra cứu và đối chiếu ngày 13/11/2026.

- PostgreSQL 18 — *SQL Dump* (`pg_dump`, custom format, `pg_restore`, giới hạn roles/tablespaces): https://www.postgresql.org/docs/18/backup-dump.html
- PostgreSQL 18 — *Continuous Archiving and Point-in-Time Recovery (PITR)*: https://www.postgresql.org/docs/18/continuous-archiving.html
- PostgreSQL 18 — *pg_restore* (`--exit-on-error`, `--single-transaction`): https://www.postgresql.org/docs/18/app-pgrestore.html
- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — test types, non-functional testing: https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- Lab Tới Nơi: `SPEC.md` (R-PAY-03/04/06, R-NFR-05, R-STATE-03/06, R-ACCEPT-05, R-IDEM-05), `src/app/payment.ts`, `src/infra/fake-payment-gateway.ts`, `articles/system-testing-recovery-test/` (`recovery.test.ts`, `restore-drill.ps1`) — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [SYS-04 Integration Test](/articles/system-testing-integration-test) · [SYS-13 Data Integrity Test](/articles/system-testing-data-integrity-test) · [Backup logic với pg_dump](/articles/postgresql-pg-dump-restore) · [PITR với base backup và WAL](/articles/postgresql-pitr-base-backup-wal) · [Runbook phục hồi PostgreSQL](/articles/postgresql-thuc-hanh-runbook-phuc-hoi) · [Idempotency, outbox và transaction](/articles/postgresql-idempotency-outbox-transaction)

Bài tiếp theo: [SYS-15 · UAT](/articles/system-testing-uat).
