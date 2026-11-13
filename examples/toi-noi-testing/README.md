# Phòng thí nghiệm kiểm thử Tới Nơi

Nhóm bài mới: [Kiểm thử hệ thống — 16 chủ đề](../../docs/system-testing-roadmap.md). Lab đầu tiên: [SYS-01 Smoke Test](articles/system-testing-smoke-test/README.md), kiểm đường nghiệp vụ quan trọng qua API và PGlite.

Bản thu nhỏ của **Tới Nơi** — ứng dụng đặt xe và giao hàng hư cấu của blog — viết bằng TypeScript (Node 22), dùng chung cho loạt bài «Kiểm thử phần mềm theo ISTQB qua dự án Tới Nơi». Mọi test và script trong các bài đều chạy trên mã nguồn ở đây.

- **Đặc tả (test basis):** [`SPEC.md`](SPEC.md) — mọi quy tắc có mã cố định `R-…` để bài viết và ca kiểm thử trích dẫn.
- **Sản phẩm được kiểm thử:** `src/` — các bài **không sửa** `src/`; muốn minh hoạ lỗi thì chép một bản có lỗi vào thư mục bài của mình.
- **Kiến trúc:** modular monolith kiểu giai đoạn MVP của tài liệu kiến trúc Tới Nơi (SPEC mục 0). Không có Kafka, Redis, realtime gateway.

## Chạy nhanh

```bash
npm install          # chỉ lần đầu (lab đã cài sẵn khi viết bài)
npm test             # toàn bộ test vitest (smoke + test của các bài)
npm start            # máy chủ lab ở http://localhost:3100 — trang khách /, trang tài xế /driver.html
```

| Lệnh | Việc làm | Ghi chú |
|---|---|---|
| `npm test` | `vitest run` mọi `tests/**/*.test.ts` và `articles/**/*.test.ts` | Chạy một bài: `npx vitest run articles/<slug>` |
| `npm run test:coverage` | như trên + đo coverage v8 trên `src/` | Báo cáo HTML: `reports/coverage/index.html` |
| `npm run test:mutation` | Stryker đột biến `src/domain/**` | ~1–2 phút; báo cáo `reports/mutation/index.html` |
| `npm run test:e2e` | Playwright mọi `articles/**/e2e/**/*.spec.ts` | Tự khởi động máy chủ lab (xem dưới) |
| `npm run typecheck` | `tsc` strict, không sinh file | |
| `npm start` | máy chủ lab, kho trong bộ nhớ | biến môi trường ở đầu `src/server.ts` |

Đổi kho sang PostgreSQL (PGlite) khi chạy máy chủ:

```bash
TOINOI_DB=pglite npm start                       # bash
$env:TOINOI_DB='pglite'; npm start               # PowerShell
```

Biến môi trường khác của `src/server.ts`: `PORT` (3100), `TOINOI_START_AT` (dời đồng hồ, vd `2026-11-04T11:30:00+07:00` để rơi vào cao điểm), `TOINOI_RAIN` (vd `SG,HN`), `TRIP_RATE_LIMIT_MAX` (5; `0` = tắt — cần khi đo tải tạo chuyến), `TRIP_RATE_LIMIT_WINDOW_MS`, `NODE_ENV=test` hoặc `TOINOI_TEST_API=1` (bật API kiểm thử `/api/test/*`, SPEC R-TEST — mặc định TẮT).

## Cấu trúc thư mục

```text
SPEC.md                    đặc tả có mã yêu cầu
src/
  domain/                  quy tắc nghiệp vụ THUẦN (không I/O, không Date.now): cước, bưu kiện, đồ ăn,
                           voucher, phí huỷ, máy trạng thái, giờ Việt Nam, tiền
  app/                     use case: báo giá, đặt/nhận/huỷ chuyến, các bước của tài xế, thu tiền có retry;
                           ports.ts = các cổng (repository, ví, thời tiết, đồng hồ, id, idempotency)
  infra/
    memory/                kho trong bộ nhớ (mặc định — nhanh, tất định)
    pglite/                kho PostgreSQL thật qua PGlite + schema.sql
    fake-payment-gateway.ts  ví «VíXanh» giả có chế độ lỗi (decline/error/timeout/lost_response)
    fake-weather.ts, clock.ts, ids.ts, seed.ts
    compose.ts             createMemorySystem / createPgliteSystem — lắp ráp hệ thống cho test
  api/                     express 5: createApp(deps) (không mở cổng — dùng với supertest), zod,
                           idempotency, giới hạn tần suất, API kiểm thử
  web/                     trang khách + trang tài xế (HTML + JS thuần, có data-testid)
  server.ts                máy chủ lab
tests/smoke/               vài test khói mỗi lớp — KHÔNG phải bộ test đầy đủ (các bài sẽ viết)
articles/<slug>/           test/script của từng bài
reports/                   coverage, mutation, playwright (sinh ra, không commit)
```

## Quy ước thư mục bài `articles/<slug>/`

- Tên thư mục = slug của bài (vd `articles/ky-thuat-hop-den-co-ban/`).
- Test vitest: `*.test.ts` bất kỳ đâu trong thư mục bài ⇒ `npx vitest run articles/<slug>`.
- Test Playwright: đặt trong `articles/<slug>/e2e/*.spec.ts` ⇒ `npx playwright test articles/<slug>` (vitest bỏ qua thư mục `e2e`).
- Script khác (autocannon, k6…): đặt trong thư mục bài, lệnh chạy ghi trong `README.md` của bài.
- Mỗi thư mục bài có `README.md` ngắn: bài nào, lệnh chạy, kết quả mong đợi.
- Bản «có lỗi» để minh hoạ (vd `buggy-fare.ts`) nằm trong thư mục bài, không sửa `src/`.
- `articles/_smoke/` chỉ để kiểm cấu hình E2E, không thuộc bài nào.

## Viết test: các điểm tiêm (seam) có sẵn

```ts
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';

const clock = new FixedClock(vnTime(2026, 11, 4, 11, 30));   // 11:30 giờ VN — cao điểm
const system = createMemorySystem({ clock, config: { payment: { timeoutMs: 50, maxRetries: 2 } } });
system.weather.setRaining(['SG']);                          // thời tiết giả
system.payments.failNext('timeout', 'timeout');             // ví giả: 2 lần hết giờ rồi chạy bình thường
const app = createApp(system.deps, { logErrors: false });   // dùng với supertest
```

- **Đồng hồ:** domain nhận `Date`/`Clock` qua tham số; `FixedClock` có `set`, `advanceMinutes`, `advanceMs`. Giờ nghiệp vụ là UTC+7 tính tường minh trong `src/domain/time.ts` (`vnTime`, `isPeakTime`, `vnDate`) — kết quả không phụ thuộc múi giờ máy.
- **Id:** mặc định `SequentialIdGenerator` (`id-0001`, `id-0002`… dùng chung cho báo giá và chuyến) ⇒ kết quả tất định.
- **Cấu hình:** `config` ghi đè **từng khoá cấp một** của `DEFAULT_CONFIG` (`payment`, `tripRateLimit`, `searchTimeoutMinutes`, `quoteTtlMinutes`) — truyền cả object con.
- **Dữ liệu:** `labSeed()` (SPEC Phụ lục A) hoặc tự truyền `seed`; hàm `voucher({...})` dựng voucher có mặc định. `system.reset()` đưa về dữ liệu mẫu.
- **PostgreSQL:** `await createPgliteSystem({ clock })` — cùng API, thêm `system.db` để truy vấn SQL trực tiếp. Khởi tạo ~0,5–1 giây, nên tạo một lần (`beforeAll`) và `reset()` trong `beforeEach`.
- **E2E:** `playwright.config.ts` khởi động `npx tsx src/server.ts` với `NODE_ENV=test` (bật `/api/test/*`), đồng hồ dời về **09:00 giờ VN 04/11/2026** (ngoài cao điểm), tắt giới hạn tần suất. Gọi `POST /api/test/reset` trong `beforeEach`.

## Giới hạn của lab (bài viết cần nói rõ khi dùng)

- **PGlite là PostgreSQL chạy WASM với MỘT kết nối.** Các câu lệnh chạy tuần tự, nên không mô phỏng được hai giao dịch thật sự song song. Test «hai tài xế cùng nhận» trên PGlite chỉ kiểm được logic `UPDATE … WHERE status = 'searching'` khi các lệnh *xen kẽ* ở mức `await`; muốn kiểm khoá dòng và giao dịch đồng thời thật cần PostgreSQL thật (nhiều kết nối) — không có trong lab.
- **Không tải trình duyệt Playwright.** Cấu hình dùng `channel: 'chrome'` ⇒ máy chạy phải có Google Chrome cài sẵn.
- **Không có Docker, PostgreSQL thật, k6, JMeter.** Script cho các công cụ này ghi rõ «chưa chạy».
- **Một tiến trình:** giới hạn tần suất và khoá idempotency nằm trong bộ nhớ của tiến trình (mất khi khởi động lại; không chia sẻ giữa nhiều bản).
- **Báo giá không bao giờ bị dọn** khỏi kho trong bộ nhớ: chạy tải dài sẽ thấy bộ nhớ tăng dần (hệ thống thật cần dọn báo giá hết hạn).
- **Xác thực giản lược:** header `x-user: customer:<id>` / `driver:<id>` (SPEC R-API-03) — không phải cơ chế an toàn cho production.
- **Bản đồ giản lược:** client gửi thẳng quãng đường (SPEC R-GEN-06).
- **Đo hiệu năng** trên một máy dev chỉ minh hoạ phương pháp; luôn ghi cấu hình máy đo.

## Ví dụ: gọi API bằng curl

```bash
curl -s -X POST localhost:3100/api/quotes -H 'content-type: application/json' -H 'x-user: customer:C001' \
  -d '{"kind":"ride","cityCode":"SG","vehicle":"bike","distanceKm":7.4,"voucherCode":"GIAM20"}'
# lấy quoteId trong kết quả, rồi:
curl -s -X POST localhost:3100/api/trips -H 'content-type: application/json' -H 'x-user: customer:C001' \
  -H 'Idempotency-Key: dat-chuyen-0001' -d '{"quoteId":"<quoteId>"}'
```

Đo tải nhanh (xem bài hiệu năng để làm đúng phương pháp):

```bash
npx autocannon -c 20 -d 10 -m POST -H "content-type=application/json" -H "x-user=customer:C001" \
  -b '{"kind":"ride","cityCode":"SG","vehicle":"bike","distanceKm":7.4}' http://localhost:3100/api/quotes
```
